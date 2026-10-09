//! `sdk.research`: counts and permutation tests over a batch of births
//! (`03-design/research.md`).
//!
//! The area founds the batch, evaluates every rule of a [`RuleSet`] on
//! every chart once into a chart-by-predicate matrix, and hands the
//! matrix to `teistro-research`, which reads no sky. The births are
//! grouped by place and clock and each group is read in one crossing, so
//! a study of many births at a few places costs a few crossings.

use std::collections::BTreeMap;

use serde::Serialize;
use teistro_core::catalogue::DashaSystem;
use teistro_core::envelope::{Envelope, Hash, Provenance, content_hash};
use teistro_core::error::Error;
use teistro_core::key::KeyId;
use teistro_core::quantity::{Depth, JulianDay, Place, Utc};
use teistro_core::time::UtcOffset;
use teistro_dasha::{DashaCursor, Timeline};
use teistro_research::{
    Cell, Counts, Design, EventTest, GroupTest, MAX_REPLICATES, Matrix, PairMatrix, Parallelism,
    ReplicateTest, SplitMix64, Tested, compare, counts, replicated, timed,
};
use teistro_rules::{Evaluator, NetStatus, Rule, RuleResult};
use teistro_serial::Document;

use crate::context::Context;
use crate::reading::ChartRequest;
use crate::rule_request::{RuleSet, RulesReading};
use crate::rules_bridge::{RuleInputs, rule_periods};

/// The widest time uncertainty a birth may carry, either side, in
/// minutes: half a day, past which the record is not of one birth.
pub const MAX_UNCERTAINTY_MINUTES: f64 = 720.0;

/// One birth of a study: when, where, under which clock, and how far its
/// recorded time is trusted.
///
/// ```
/// use teistro::research::Birth;
/// use teistro::UtcOffset;
/// use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
///
/// let kathmandu = Place::new(
///     Latitude::try_new(27.7172)?,
///     Longitude::try_new(85.324)?,
///     Altitude::try_new(1400.0)?,
/// );
/// let offset = UtcOffset::try_from_seconds(20_700)?;
/// let birth = Birth::new(JulianDay::<Utc>::literal(2_447_000.25), kathmandu, offset)
///     .uncertain_by(30.0);
/// assert_eq!(birth.uncertainty_minutes, 30.0);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Birth {
    /// The instant, UTC.
    pub instant: JulianDay<Utc>,
    /// Where.
    pub place: Place,
    /// The local clock's offset from UTC.
    pub offset: UtcOffset,
    /// How far either side of `instant` the recorded time may be wrong,
    /// in minutes; zero when it is trusted. A rule whose answer at either
    /// edge differs from its answer at the instant is counted unstable on
    /// this chart, never decided (`research.md` §1.7).
    pub uncertainty_minutes: f64,
}

impl Birth {
    /// A birth whose recorded time is trusted.
    #[must_use]
    pub const fn new(instant: JulianDay<Utc>, place: Place, offset: UtcOffset) -> Birth {
        Birth {
            instant,
            place,
            offset,
            uncertainty_minutes: 0.0,
        }
    }

    /// The same birth with its recorded time trusted only to `minutes`
    /// either side.
    #[must_use]
    pub const fn uncertain_by(mut self, minutes: f64) -> Birth {
        self.uncertainty_minutes = minutes;
        self
    }
}

/// When a rule counts as holding on a chart.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Holds {
    /// It is formed and its cancellations do not undo it: present, and
    /// not fully cancelled. A rule is its conditions *and* its
    /// cancellations, so this is the default.
    #[default]
    Standing,
    /// It is formed, whatever its cancellations say.
    Formed,
}

/// The study a call reads: its births, what each chart is read for, and
/// its predicates.
#[derive(Clone, Copy, Debug)]
pub struct Study<'s> {
    /// The births, in the order the design labels them.
    pub births: &'s [Birth],
    /// What each chart is read with: its sections and options. Each
    /// birth's place and clock replace the request's own.
    pub request: &'s ChartRequest,
    /// The predicates: every rule of the set is one, named by its key, in
    /// the set's order, and the set is one family for the corrections.
    pub rules: &'s RuleSet,
    /// When a rule counts as holding.
    pub holds: Holds,
}

impl<'s> Study<'s> {
    /// A study of `births`, each read as `request` asks, of every rule in
    /// `rules`, a rule counting while it stands.
    #[must_use]
    pub const fn new(
        births: &'s [Birth],
        request: &'s ChartRequest,
        rules: &'s RuleSet,
    ) -> Study<'s> {
        Study {
            births,
            request,
            rules,
            holds: Holds::Standing,
        }
    }

    /// The same study, a rule counting as `holds` says.
    #[must_use]
    pub const fn holding(mut self, holds: Holds) -> Study<'s> {
        self.holds = holds;
        self
    }
}

/// What a study call seals into its input hash, which is the study's
/// pre-registration: publish it before the data are collected
/// (`research.md` §1.7).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StudyInput<'a, T: Serialize> {
    births: &'a [Birth],
    rules: Vec<&'a str>,
    holds: Holds,
    design: &'a Design,
    #[serde(skip_serializing_if = "Option::is_none")]
    test: Option<&'a T>,
    /// Each crossing's own input hash, which covers what the charts were
    /// read with.
    read: Vec<Hash>,
}

/// The crossings' provenance: the first one's, which every crossing
/// shares but for its input, and every crossing's input hash.
struct Read {
    provenance: Provenance,
    inputs: Vec<Hash>,
}

/// `sdk.research`: counts and permutation tests over a batch of births.
#[derive(Clone, Copy, Debug)]
pub struct ResearchArea<'a> {
    context: &'a Context,
}

impl<'a> ResearchArea<'a> {
    pub(crate) fn of(context: &'a Context) -> ResearchArea<'a> {
        ResearchArea { context }
    }

    /// The context this area was read off.
    #[must_use]
    pub fn context(self) -> &'a Context {
        self.context
    }

    /// How often each rule holds in each group, with the charts it cannot
    /// be read on and those it is unstable on counted apart. No null and
    /// no shuffle.
    ///
    /// # Errors
    ///
    /// As [`ResearchArea::compare`] refuses a study and a design.
    pub fn counts(self, study: Study<'_>, design: &Design) -> Result<Envelope<Counts>, Error> {
        let (matrix, read) = self.matrix(study)?;
        let value = counts(&matrix, design)?;
        Ok(seal(value, read, study, design, None::<&GroupTest>))
    }

    /// Whether the design's groups differ on each rule, the labels
    /// permuted (within strata when the design has them), with the
    /// family's corrections and the effect sizes.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming its field: no birth (`births`); a birth's
    /// uncertainty outside 0 to [`MAX_UNCERTAINTY_MINUTES`]
    /// (`births[i].uncertaintyMinutes`); a rule the language cannot
    /// evaluate (`rules[i]`); whatever reading the births refuses; and
    /// whatever the design and the test refuse
    /// ([`teistro_research::compare`]).
    pub fn compare(
        self,
        study: Study<'_>,
        design: &Design,
        test: &GroupTest,
    ) -> Result<Envelope<Tested>, Error> {
        let (matrix, read) = self.matrix(study)?;
        let value = compare(&matrix, design, test)?;
        Ok(seal(value, read, study, design, Some(&test.registered())))
    }

    /// Every rule on every chart, the births read a place at a time.
    fn matrix(self, study: Study<'_>) -> Result<(Matrix, Read), Error> {
        check(study)?;
        let rules = study.rules.rules();
        let mut columns = vec![vec![Cell::Absent; study.births.len()]; rules.len()];
        let mut read: Option<Read> = None;
        for group in by_place(study.births) {
            let Some(first) = group.first().and_then(|&i| study.births.get(i)) else {
                continue;
            };
            let request = study.request.placed(first.place, first.offset);
            let PlaceReads {
                instants,
                owners,
                edges,
            } = instants_of(study.births, &group)?;
            let answered = self.read_rules(&instants, &owners, &request, study.rules)?;
            let standing: Vec<Vec<Option<bool>>> = answered
                .value
                .iter()
                .map(|(_, reading)| held_by(reading, rules, study.holds))
                .collect();
            for (position, &chart) in group.iter().enumerate() {
                let centre = standing.get(position);
                let sides = edges.iter().position(|&i| i == chart).map(|k| {
                    let at = group.len() + 2 * k;
                    [standing.get(at), standing.get(at + 1)]
                });
                for (j, column) in columns.iter_mut().enumerate() {
                    let at = |reading: Option<&Vec<Option<bool>>>| {
                        reading.and_then(|r| r.get(j).copied().flatten())
                    };
                    let cell = match (at(centre), sides) {
                        (None, _) => Cell::Unreadable,
                        (Some(holds), Some(sides))
                            if sides.iter().any(|&side| at(side) != Some(holds)) =>
                        {
                            Cell::Unstable
                        }
                        (Some(true), _) => Cell::Present,
                        (Some(false), _) => Cell::Absent,
                    };
                    if let Some(slot) = column.get_mut(chart) {
                        *slot = cell;
                    }
                }
            }
            match &mut read {
                None => {
                    read = Some(Read {
                        inputs: answered.inputs,
                        provenance: answered.provenance,
                    });
                }
                Some(read) => read.inputs.extend(answered.inputs),
            }
        }
        let mut matrix = Matrix::new(study.births.len());
        for (rule, column) in rules.iter().zip(&columns) {
            matrix.push(rule.key.clone(), column)?;
        }
        let read = read.ok_or_else(|| {
            Error::invalid_arg("a study needs births, and this one has none").with_field("births")
        })?;
        Ok((matrix, read))
    }
}

impl ResearchArea<'_> {
    /// One place's charts with what they answer by rule, `owners` naming
    /// the birth each instant belongs to (`births[3]`).
    ///
    /// A batch in which one chart cannot have an input a rule reads (a
    /// polar birth's special lagnas) comes back read without it for every
    /// chart, so such a batch is read again a chart at a time and only the
    /// charts that lack the input count as unreadable. A batch that is
    /// refused is read a chart at a time too, so the refusal names the
    /// birth that caused it.
    fn read_rules<'r>(
        self,
        instants: &[JulianDay<Utc>],
        owners: &[String],
        request: &ChartRequest,
        set: &'r RuleSet,
    ) -> Result<Answered<'r>, Error> {
        let chart = self.context.chart();
        let alone = |k: usize, instant: &JulianDay<Utc>| {
            chart
                .readings_with_rules(std::slice::from_ref(instant), request, set)
                .map_err(|error| {
                    let owner = owners.get(k).map_or("a birth", String::as_str);
                    let hint = match error.hint() {
                        Some(hint) => format!("{owner} is the birth refused; {hint}"),
                        None => format!("{owner} is the birth refused"),
                    };
                    error.with_hint(hint)
                })
        };
        let batch = match chart.readings_with_rules(instants, request, set) {
            Ok(batch) => batch,
            Err(error) => {
                // The chart that refuses refuses alone; if none does, the
                // batch's own refusal is the answer.
                for (k, instant) in instants.iter().enumerate() {
                    alone(k, instant)?;
                }
                return Err(error);
            }
        };
        let lacking = batch
            .value
            .iter()
            .any(|(_, reading)| !reading.unreadable.is_empty());
        if !lacking || instants.len() < 2 {
            return Ok(Answered {
                inputs: vec![batch.provenance.input_hash],
                provenance: batch.provenance,
                value: batch.value,
            });
        }
        let mut value = Vec::with_capacity(instants.len());
        let mut inputs = Vec::with_capacity(instants.len());
        for (k, instant) in instants.iter().enumerate() {
            let one = alone(k, instant)?;
            inputs.push(one.provenance.input_hash);
            value.extend(one.value);
        }
        Ok(Answered {
            value,
            inputs,
            provenance: batch.provenance,
        })
    }
}

/// One place's readings: the charts with their rule answers, every
/// crossing's input hash, and the first crossing's provenance.
struct Answered<'r> {
    value: Vec<(Document, RulesReading<'r>)>,
    inputs: Vec<Hash>,
    provenance: Provenance,
}

/// Refuses what no reading could answer.
fn check(study: Study<'_>) -> Result<(), Error> {
    for (index, birth) in study.births.iter().enumerate() {
        let minutes = birth.uncertainty_minutes;
        if !(0.0..=MAX_UNCERTAINTY_MINUTES).contains(&minutes) {
            return Err(Error::invalid_arg(format!(
                "a birth's time uncertainty is 0 to {MAX_UNCERTAINTY_MINUTES} minutes either side, not {minutes}"
            ))
            .with_field(format!("births[{index}].uncertaintyMinutes")));
        }
    }
    for (index, rule) in study.rules.rules().iter().enumerate() {
        if !rule.is_evaluable() {
            return Err(Error::invalid_arg(format!(
                "rule `{}` is computed by its author rather than by the rule language, so a study cannot count it",
                rule.key
            ))
            .with_field(format!("rules[{index}]")));
        }
    }
    Ok(())
}

/// The instants one place's charts are read at: each birth's own, then
/// both edges of every uncertain one.
fn instants_of(births: &[Birth], group: &[usize]) -> Result<PlaceReads, Error> {
    let mut instants = Vec::new();
    let mut owners = Vec::new();
    for &i in group {
        if let Some(birth) = births.get(i) {
            instants.push(birth.instant);
            owners.push(format!("births[{i}]"));
        }
    }
    let mut edges = Vec::new();
    for &i in group {
        let Some(birth) = births
            .get(i)
            .filter(|birth| birth.uncertainty_minutes > 0.0)
        else {
            continue;
        };
        let days = birth.uncertainty_minutes / 1440.0;
        let edge = |days: f64| {
            birth.instant.plus_days(days).map_err(|error| {
                Error::from(error).with_field(format!("births[{i}].uncertaintyMinutes"))
            })
        };
        instants.push(edge(-days)?);
        instants.push(edge(days)?);
        for side in ["earliest", "latest"] {
            owners.push(format!("births[{i}], at its {side}"));
        }
        edges.push(i);
    }
    Ok(PlaceReads {
        instants,
        owners,
        edges,
    })
}

/// The instants one place's charts are read at, the birth each belongs
/// to, and the uncertain births in the order their edges follow.
struct PlaceReads {
    instants: Vec<JulianDay<Utc>>,
    owners: Vec<String>,
    edges: Vec<usize>,
}

/// Whether each rule holds on one reading; none where the chart cannot
/// have an input the rule reads.
fn held_by(reading: &RulesReading<'_>, rules: &[Rule], holds: Holds) -> Vec<Option<bool>> {
    rules
        .iter()
        .map(|rule| {
            if !reading.unreadable.is_empty() && rule.reads_points() {
                return None;
            }
            Some(reading.present.iter().any(|present| {
                std::ptr::eq(present.rule, rule)
                    && match holds {
                        Holds::Formed => true,
                        Holds::Standing => present.result.status != Some(NetStatus::FullyCancelled),
                    }
            }))
        })
        .collect()
}

/// A test as a study registers it: everything that decides the answer,
/// and not the thread count, which the answer does not depend on.
trait Registered: Serialize + Clone {
    fn registered(&self) -> Self;
}

impl Registered for GroupTest {
    fn registered(&self) -> GroupTest {
        GroupTest {
            parallelism: Parallelism::One,
            ..self.clone()
        }
    }
}

impl Registered for EventTest {
    fn registered(&self) -> EventTest {
        EventTest {
            parallelism: Parallelism::One,
            ..self.clone()
        }
    }
}

/// Seals a study's answer under its first crossing's provenance and the
/// study's own input hash.
fn seal<T: Serialize, Q: Serialize>(
    value: T,
    read: Read,
    study: Study<'_>,
    design: &Design,
    test: Option<&Q>,
) -> Envelope<T> {
    let mut provenance = read.provenance;
    provenance.input_hash = content_hash(&StudyInput {
        births: study.births,
        rules: study
            .rules
            .rules()
            .iter()
            .map(|rule| rule.key.as_str())
            .collect(),
        holds: study.holds,
        design,
        test,
        read: read.inputs,
    });
    Envelope::sealing(value, provenance)
}

/// The births' indices grouped by place and clock, groups in the order of
/// their first birth and indices ascending inside each.
fn by_place(births: &[Birth]) -> Vec<Vec<usize>> {
    let mut groups: Vec<(Place, UtcOffset, Vec<usize>)> = Vec::new();
    for (index, birth) in births.iter().enumerate() {
        match groups
            .iter_mut()
            .find(|(place, offset, _)| *place == birth.place && *offset == birth.offset)
        {
            Some((_, _, members)) => members.push(index),
            None => groups.push((birth.place, birth.offset, vec![index])),
        }
    }
    groups.into_iter().map(|(_, _, members)| members).collect()
}

/// One subject of an event study: a birth and the dated event of its life
/// the study is about.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Subject {
    /// The birth.
    pub birth: Birth,
    /// When the event happened, UTC.
    pub event: JulianDay<Utc>,
}

/// What the shuffled-event null keeps (`research.md` §1.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventShuffle {
    /// Event dates move among people: the calendar of events is kept, each
    /// person's age at the event is not. A pairing that puts an event
    /// before a birth is what `afterBirth` decides.
    EventDates,
    /// Ages at the event move among people: person `i` is read at its
    /// birth plus person `j`'s age. The ages are kept; the calendar is not.
    AgesAtEvent,
}

/// An event study: its subjects, how each chart is read, the rules whose
/// delivery is tested, the dasha that delivers them and the shuffle.
#[derive(Clone, Copy, Debug)]
pub struct EventStudy<'s> {
    /// The subjects.
    pub subjects: &'s [Subject],
    /// What each chart is read with; each birth's place and clock replace
    /// the request's, and the dasha below is added to its dashas.
    pub request: &'s ChartRequest,
    /// The rules; a rule holds at an instant when it is formed (or
    /// standing, as `holds` says) and a running period delivers it
    /// ([`teistro_rules::Evaluator::delivery`]).
    pub rules: &'s RuleSet,
    /// The dasha whose periods deliver.
    pub dasha: DashaSystem,
    /// How deep the delivering periods are read: 1 is the mahadasha alone,
    /// 2 adds the antardasha, and a rule holds when any of them delivers.
    pub depth: Depth,
    /// What the null keeps.
    pub shuffle: EventShuffle,
    /// When a rule counts as formed.
    pub holds: Holds,
}

impl<'s> EventStudy<'s> {
    /// A study of when each rule is delivered by `dasha`'s periods down to
    /// the antardasha under the null `shuffle` keeps, a rule counting while
    /// it stands. The shuffle has no default: the two keep different
    /// margins, and a dasha reads the age that a date shuffle moves (C364).
    #[must_use]
    pub fn new(
        subjects: &'s [Subject],
        request: &'s ChartRequest,
        rules: &'s RuleSet,
        dasha: DashaSystem,
        shuffle: EventShuffle,
    ) -> EventStudy<'s> {
        EventStudy {
            subjects,
            request,
            rules,
            dasha,
            depth: Depth::try_new(2).unwrap_or(Depth::MIN),
            shuffle,
            holds: Holds::Standing,
        }
    }

    /// The same study, periods read to `depth`.
    #[must_use]
    pub const fn to_depth(mut self, depth: Depth) -> EventStudy<'s> {
        self.depth = depth;
        self
    }

    /// The same study under another null.
    #[must_use]
    pub const fn shuffled(mut self, shuffle: EventShuffle) -> EventStudy<'s> {
        self.shuffle = shuffle;
        self
    }

    /// The same study, a rule counting as `holds` says.
    #[must_use]
    pub const fn holding(mut self, holds: Holds) -> EventStudy<'s> {
        self.holds = holds;
        self
    }
}

/// What an event study seals into its input hash.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EventInput<'a> {
    subjects: &'a [Subject],
    rules: Vec<&'a str>,
    dasha: KeyId,
    depth: u8,
    shuffle: EventShuffle,
    holds: Holds,
    strata: Option<&'a [u32]>,
    test: &'a EventTest,
    read: Vec<Hash>,
}

/// Gauquelin's control: the sample refounded `replicates` times, each
/// subject's date and place kept and its clock time taken from another
/// subject, drawn inside strata (`research.md` §1.3).
#[derive(Clone, Debug, PartialEq, Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Recombine {
    /// The study's seed; required.
    pub seed: u64,
    /// How many replicates, 1 to [`teistro_research::MAX_REPLICATES`];
    /// each refounds every birth.
    pub replicates: u32,
    /// Each birth's stratum, when clock times move only inside one (a
    /// birth decade: the hour births peaked at moved with obstetrics).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strata: Option<Vec<u32>>,
}

impl ResearchArea<'_> {
    /// Whether each rule is delivered at the subjects' own events more
    /// (or less) often than at events shuffled among them.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming its field: an event before its own birth
    /// (`subjects[i].event`); a rule the language cannot evaluate
    /// (`rules[i]`); whatever reading the births or the dasha refuses; and
    /// whatever [`teistro_research::timed`] refuses.
    pub fn timed(
        self,
        study: EventStudy<'_>,
        strata: Option<&[u32]>,
        test: &EventTest,
    ) -> Result<Envelope<Tested>, Error> {
        let births: Vec<Birth> = study.subjects.iter().map(|s| s.birth).collect();
        for (index, subject) in study.subjects.iter().enumerate() {
            if subject.event.get() < subject.birth.instant.get() {
                return Err(Error::invalid_arg(
                    "an event before its own birth cannot be delivered by a period of that life",
                )
                .with_field(format!("subjects[{index}].event")));
            }
        }
        check(Study::new(&births, study.request, study.rules))?;
        let mut pairs = pairings(study)?;
        let (charts, read, provenance) = self.lives(study, &births)?;
        let readings = study.rules.readings().readings();
        let evaluators: Vec<Option<(Evaluator<'_>, &Lived)>> = charts
            .iter()
            .map(|lived| lived.as_ref().map(|l| (l.inputs.evaluator(readings), l)))
            .collect();
        for (r, rule) in study.rules.rules().iter().enumerate() {
            pairs.push(rule.key.clone(), |i, j| match evaluators.get(i) {
                Some(Some((evaluator, lived))) => delivered(
                    study,
                    (r, rule),
                    evaluator,
                    lived,
                    event_instant(study, i, j),
                ),
                _ => Cell::Unreadable,
            })?;
        }
        let value = timed(&pairs, strata, test)?;
        let mut provenance = provenance.ok_or_else(|| {
            Error::invalid_arg("an event study needs subjects, and this one has none")
                .with_field("subjects")
        })?;
        let registered = test.registered();
        provenance.input_hash = content_hash(&EventInput {
            subjects: study.subjects,
            rules: study
                .rules
                .rules()
                .iter()
                .map(|rule| rule.key.as_str())
                .collect(),
            dasha: KeyId::from(study.dasha),
            depth: study.depth.get(),
            shuffle: study.shuffle,
            holds: study.holds,
            strata,
            test: &registered,
            read,
        });
        Ok(Envelope::sealing(value, provenance))
    }

    /// Every subject's chart, rule results and dasha, read a place at a
    /// time, with every crossing's input hash and the first's provenance.
    fn lives(self, study: EventStudy<'_>, births: &[Birth]) -> Result<Lives, Error> {
        let mut charts: Vec<Option<Lived>> = (0..births.len()).map(|_| None).collect();
        let mut read = Vec::new();
        let mut provenance = None;
        let wanted = KeyId::from(study.dasha);
        let mut dashas: Vec<KeyId> = study.request.dashas().to_vec();
        if !dashas.contains(&wanted) {
            dashas.push(wanted);
        }
        for group in by_place(births) {
            let Some(head) = group.first().and_then(|&i| births.get(i)) else {
                continue;
            };
            let request = study
                .request
                .placed(head.place, head.offset)
                .with_dashas(dashas.iter().copied());
            let instants: Vec<JulianDay<Utc>> = group
                .iter()
                .filter_map(|&i| births.get(i))
                .map(|b| b.instant)
                .collect();
            let owners: Vec<String> = group.iter().map(|i| format!("subjects[{i}]")).collect();
            let answered = self.read_rules(&instants, &owners, &request, study.rules)?;
            read.extend(answered.inputs);
            for (&i, (document, reading)) in group.iter().zip(answered.value) {
                let life = self.lived(study, &document, &reading)?;
                if let Some(slot) = charts.get_mut(i) {
                    *slot = Some(life);
                }
            }
            provenance.get_or_insert(answered.provenance);
        }
        Ok((charts, read, provenance))
    }

    /// One subject's chart as an event study reads it.
    fn lived(
        self,
        study: EventStudy<'_>,
        document: &Document,
        reading: &RulesReading<'_>,
    ) -> Result<Lived, Error> {
        let standing = held_by(reading, study.rules.rules(), study.holds);
        let results: Vec<Option<RuleResult>> = study
            .rules
            .rules()
            .iter()
            .zip(&standing)
            .map(|(rule, stands)| {
                stands.filter(|&h| h).and_then(|_| {
                    reading
                        .present
                        .iter()
                        .find(|p| std::ptr::eq(p.rule, rule))
                        .map(|p| p.result.clone())
                })
            })
            .collect();
        Ok(Lived {
            inputs: RuleInputs::of(document)?,
            cursor: self.context.chart().dasha(document, study.dasha)?,
            results,
            unreadable: standing.iter().map(Option::is_none).collect(),
        })
    }

    /// Whether each rule is commoner (or rarer) in this sample than in its
    /// own recombined population: the sample refounded with clock times
    /// shuffled among its subjects, date and place kept (§1.3).
    ///
    /// # Errors
    ///
    /// As [`ResearchArea::counts`] refuses a study; `strata` not one per
    /// birth (`control.strata`); and whatever
    /// [`teistro_research::replicated`] refuses.
    pub fn expected(
        self,
        study: Study<'_>,
        control: &Recombine,
        test: &ReplicateTest,
    ) -> Result<Envelope<Tested>, Error> {
        let n = study.births.len();
        if control.strata.as_ref().is_some_and(|s| s.len() != n) {
            return Err(Error::invalid_arg(format!(
                "{} strata for {n} births; give one stratum per birth",
                control.strata.as_ref().map_or(0, Vec::len)
            ))
            .with_field("control.strata"));
        }
        if control.replicates == 0 || control.replicates > MAX_REPLICATES {
            return Err(Error::invalid_arg(format!(
                "{} replicates is outside 1 to {MAX_REPLICATES}",
                control.replicates
            ))
            .with_field("control.replicates"));
        }
        let one = Design::new(vec![0; n]);
        let (matrix, read) = self.matrix(study)?;
        let observed = matrix.tallies();
        let names: Vec<String> = study.rules.rules().iter().map(|r| r.key.clone()).collect();
        let mut replicates = Vec::new();
        for r in 0..control.replicates {
            let births = recombined(study.births, control, r);
            let trusted: Vec<Birth> = births.iter().map(|b| b.uncertain_by(0.0)).collect();
            let (matrix, _) = self.matrix(Study {
                births: &trusted,
                ..study
            })?;
            replicates.push(
                matrix
                    .tallies()
                    .into_iter()
                    .map(|c| (c.present, c.present + c.absent))
                    .collect(),
            );
        }
        let value =
            replicated(&names, &observed, &replicates, test).map_err(|error| {
                match error.field() {
                    Some("replicates") => error.with_field("control.replicates"),
                    _ => error,
                }
            })?;
        Ok(seal(value, read, study, &one, Some(&(control, test))))
    }
}

/// One subject's chart as an event study reads it.
struct Lived {
    inputs: RuleInputs,
    cursor: DashaCursor,
    /// Each rule's result where it holds.
    results: Vec<Option<RuleResult>>,
    /// Whether each rule could not be read on the chart.
    unreadable: Vec<bool>,
}

/// Every subject's life, the crossings' input hashes and the first
/// crossing's provenance.
type Lives = (Vec<Option<Lived>>, Vec<Hash>, Option<Provenance>);

/// The instant subject `i` is read at under subject `j`'s event, as the
/// study's shuffle pairs them.
fn event_instant(study: EventStudy<'_>, i: usize, j: usize) -> Option<JulianDay<Utc>> {
    let (own, other) = (study.subjects.get(i)?, study.subjects.get(j)?);
    match study.shuffle {
        EventShuffle::EventDates => Some(other.event),
        EventShuffle::AgesAtEvent => own
            .birth
            .instant
            .plus_days(other.event.get() - other.birth.instant.get())
            .ok(),
    }
}

/// The study's pairings, each that would read a subject before its own
/// birth forbidden.
fn pairings(study: EventStudy<'_>) -> Result<PairMatrix, Error> {
    let n = study.subjects.len();
    let mut pairs = PairMatrix::new(n).map_err(|error| error.under("subjects"))?;
    for (i, subject) in study.subjects.iter().enumerate() {
        let born = subject.birth.instant.get();
        for j in 0..n {
            if event_instant(study, i, j).is_none_or(|at| at.get() < born) {
                pairs.forbid(i, j);
            }
        }
    }
    Ok(pairs)
}

/// Whether rule `r` of a subject's chart is delivered at `at`: formed on
/// the chart, and a period running then delivers it.
fn delivered(
    study: EventStudy<'_>,
    (r, rule): (usize, &Rule),
    evaluator: &Evaluator<'_>,
    lived: &Lived,
    at: Option<JulianDay<Utc>>,
) -> Cell {
    if lived.unreadable.get(r).copied().unwrap_or(true) {
        return Cell::Unreadable;
    }
    let Some(at) = at else {
        return Cell::Unreadable;
    };
    let Some(Some(result)) = lived.results.get(r) else {
        return Cell::Absent;
    };
    let chain = lived.cursor.at(at, study.depth);
    if chain.is_empty() {
        return Cell::Unreadable;
    }
    if evaluator
        .delivery(rule, result, rule_periods(&chain))
        .highest()
        .is_some()
    {
        Cell::Present
    } else {
        Cell::Absent
    }
}

/// Replicate `r` of a sample: each birth's civil date and place with the
/// clock time of the birth the replicate's shuffle pairs it with.
fn recombined(births: &[Birth], control: &Recombine, r: u32) -> Vec<Birth> {
    let mut rng = SplitMix64::for_permutation(control.seed, u64::from(r));
    let mut order: Vec<usize> = (0..births.len()).collect();
    match &control.strata {
        None => rng.shuffle(&mut order),
        Some(strata) => {
            let mut by_key: BTreeMap<u32, Vec<usize>> = BTreeMap::new();
            for (i, &key) in strata.iter().enumerate() {
                by_key.entry(key).or_default().push(i);
            }
            for positions in by_key.values() {
                let mut drawn = positions.clone();
                rng.shuffle(&mut drawn);
                for (&at, &from) in positions.iter().zip(&drawn) {
                    if let Some(slot) = order.get_mut(at) {
                        *slot = from;
                    }
                }
            }
        }
    }
    births
        .iter()
        .zip(&order)
        .map(|(birth, &from)| {
            let donor = births.get(from).unwrap_or(birth);
            let local = |b: &Birth| b.instant.get() + f64::from(b.offset.seconds()) / 86_400.0;
            // A civil day runs from midnight, half a Julian day past its
            // noon; the clock time is the fraction past that midnight.
            let midnight = |jd: f64| (jd + 0.5).floor() - 0.5;
            let own = local(birth);
            let theirs = local(donor);
            let at = midnight(own) + (theirs - midnight(theirs))
                - f64::from(birth.offset.seconds()) / 86_400.0;
            Birth {
                instant: JulianDay::literal(at),
                ..*birth
            }
        })
        .collect()
}
