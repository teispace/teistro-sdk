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
use teistro_core::catalogue::Graha;
use teistro_core::envelope::{Envelope, Hash, Provenance, content_hash};
use teistro_core::error::Error;
use teistro_core::key::KeyId;
use teistro_core::quantity::{Depth, JulianDay, Place, Utc};
use teistro_core::time::UtcOffset;
use teistro_dasha::{DashaCursor, DashaName, Timeline};
use teistro_research::{
    Cell, Counts, Design, EventTest, GroupTest, MAX_INSTANTS, MAX_REPLICATES, MatchedMatrix,
    Matrix, PairMatrix, Parallelism, ReplicateTest, SplitMix64, Tested, compare, counts, matched,
    replicated, timed,
};
use teistro_rules::{Evaluator, NetStatus, Rule, RuleResult, TransitSky};
use teistro_serial::Document;

use crate::context::Context;
use crate::reading::ChartRequest;
use crate::rule_request::{RuleSet, RulesReading};
use crate::rules_bridge::{RuleInputs, rule_periods};

/// The widest time uncertainty a birth may carry, either side, in
/// minutes: half a day, past which the record is not of one birth.
pub const MAX_UNCERTAINTY_MINUTES: f64 = 720.0;

/// How far apart, in minutes, an uncertain birth is read across its
/// window unless a study says otherwise: under half the shortest dwell of
/// a navamsa on the lagna (about five minutes at ordinary latitudes), so a
/// navamsa that comes and goes inside the window is seen. A study reading
/// a finer varga, or a lagna at a high latitude, sets a smaller step
/// ([`Study::stepping`]); the step is sealed into the study's
/// pre-registration either way.
pub const DEFAULT_UNCERTAINTY_STEP_MINUTES: f64 = 2.0;

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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
    /// either side: a study reads it across that window every
    /// [`Study::uncertainty_step_minutes`], and a rule that changes
    /// anywhere in it is unstable on the chart.
    #[must_use]
    pub const fn uncertain_by(mut self, minutes: f64) -> Birth {
        self.uncertainty_minutes = minutes;
        self
    }
}

/// When a rule counts as holding on a chart.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
    /// How far apart, in minutes, an uncertain birth is read across its
    /// window ([`DEFAULT_UNCERTAINTY_STEP_MINUTES`]). A change shorter than
    /// the step can still be missed, which is why the answer seals it.
    pub uncertainty_step_minutes: f64,
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
            uncertainty_step_minutes: DEFAULT_UNCERTAINTY_STEP_MINUTES,
        }
    }

    /// The same study, an uncertain birth read every `minutes` across its
    /// window rather than every [`DEFAULT_UNCERTAINTY_STEP_MINUTES`].
    #[must_use]
    pub const fn stepping(mut self, minutes: f64) -> Study<'s> {
        self.uncertainty_step_minutes = minutes;
        self
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
struct StudyInput<'a, T: Serialize> {
    births: &'a [Birth],
    rules: Vec<&'a str>,
    holds: Holds,
    /// The step an uncertain birth was read at; absent from a study whose
    /// births are all exact, which it does not touch.
    #[serde(skip_serializing_if = "Option::is_none")]
    uncertainty_step_minutes: Option<f64>,
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
        study
            .rules
            .refuse_transits("a study of births without events")?;
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
                windows,
            } = instants_of(study.births, &group, study.uncertainty_step_minutes)?;
            let answered = self.read_rules(&instants, &owners, &request, study.rules)?;
            let standing: Vec<Vec<Option<bool>>> = answered
                .value
                .iter()
                .map(|(_, reading)| held_by(reading, rules, study.holds))
                .collect();
            for (position, &chart) in group.iter().enumerate() {
                let centre = standing.get(position);
                let window = windows
                    .get(position)
                    .cloned()
                    .flatten()
                    .and_then(|range| standing.get(range));
                for (j, column) in columns.iter_mut().enumerate() {
                    let at = |reading: Option<&Vec<Option<bool>>>| {
                        reading.and_then(|r| r.get(j).copied().flatten())
                    };
                    let cell = match (at(centre), window) {
                        (None, _) => Cell::Unreadable,
                        (Some(holds), Some(window))
                            if window
                                .iter()
                                .any(|reading| at(Some(reading)) != Some(holds)) =>
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
                .answered(std::slice::from_ref(instant), request, set)
                .map(|answered| Envelope::sealing(answered.value, answered.provenance))
                .map_err(|error| {
                    let owner = owners.get(k).map_or("a birth", String::as_str);
                    let hint = match error.hint() {
                        Some(hint) => format!("{owner} is the birth refused; {hint}"),
                        None => format!("{owner} is the birth refused"),
                    };
                    error.with_hint(hint)
                })
        };
        let batch = match chart
            .answered(instants, request, set)
            .map(|answered| Envelope::sealing(answered.value, answered.provenance))
        {
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
    let step = study.uncertainty_step_minutes;
    if !(step.is_finite() && step > 0.0 && step <= MAX_UNCERTAINTY_MINUTES) {
        return Err(Error::invalid_arg(format!(
            "an uncertain birth is read every so many minutes across its window, more than 0 and at most \
             {MAX_UNCERTAINTY_MINUTES}, not {step}"
        ))
        .with_field("uncertaintyStepMinutes"));
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

/// The instants one place's charts are read at: each birth's own, then,
/// for every uncertain one, its window read every `step` minutes out from
/// the birth to both edges, the edges themselves included.
///
/// Reading the edges alone called a rule stable that changed and changed
/// back inside the window: across the widest window, a whole day, the
/// lagna passes every sign and comes back to its own.
fn instants_of(births: &[Birth], group: &[usize], step: f64) -> Result<PlaceReads, Error> {
    let mut instants = Vec::new();
    let mut owners = Vec::new();
    for &i in group {
        if let Some(birth) = births.get(i) {
            instants.push(birth.instant);
            owners.push(format!("births[{i}]"));
        }
    }
    let mut windows = Vec::with_capacity(group.len());
    for &i in group {
        let Some(birth) = births
            .get(i)
            .filter(|birth| birth.uncertainty_minutes > 0.0)
        else {
            windows.push(None);
            continue;
        };
        let start = instants.len();
        for minutes in window_offsets(birth.uncertainty_minutes, step) {
            let at = birth.instant.plus_days(minutes / 1440.0).map_err(|error| {
                Error::from(error).with_field(format!("births[{i}].uncertaintyMinutes"))
            })?;
            instants.push(at);
            owners.push(format!("births[{i}], {minutes:+} minutes from it"));
        }
        windows.push(Some(start..instants.len()));
    }
    Ok(PlaceReads {
        instants,
        owners,
        windows,
    })
}

/// The minutes from an uncertain birth its window is read at: every
/// `step` out to `uncertainty` on both sides, each edge exactly, the birth
/// itself not again.
fn window_offsets(uncertainty: f64, step: f64) -> Vec<f64> {
    let mut offsets = Vec::new();
    let mut k = 1.0_f64;
    loop {
        let minutes = (k * step).min(uncertainty);
        offsets.push(-minutes);
        offsets.push(minutes);
        if minutes >= uncertainty {
            return offsets;
        }
        k += 1.0;
    }
}

/// The instants one place's charts are read at, the birth each belongs
/// to, and where each uncertain birth's window is among them.
struct PlaceReads {
    instants: Vec<JulianDay<Utc>>,
    owners: Vec<String>,
    /// One per birth of the place, in its order: the readings of its
    /// window, none for a birth known to the minute.
    windows: Vec<Option<std::ops::Range<usize>>>,
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
            max_pairs: teistro_research::DEFAULT_MAX_PAIRS,
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
        uncertainty_step_minutes: study
            .births
            .iter()
            .any(|birth| birth.uncertainty_minutes > 0.0)
            .then_some(study.uncertainty_step_minutes),
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Subject {
    /// The birth.
    pub birth: Birth,
    /// When the event happened, UTC.
    pub event: JulianDay<Utc>,
}

/// What the shuffled-event null keeps (`research.md` §1.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventShuffle {
    /// Event dates move among people: the calendar of events is kept, each
    /// person's age at the event is not. A pairing that puts an event
    /// before a birth is what `afterBirth` decides.
    EventDates,
    /// Ages at the event move among people: person `i` is read at its
    /// birth plus person `j`'s age. The ages are kept; the calendar is not.
    AgesAtEvent,
    /// Each person is their own control (the case-crossover design,
    /// C367): the event is as likely at any of the person's referent
    /// instants ([`Referents`]), so nothing fixed about the person, the
    /// chart included, can make a rule look delivered. Nobody's event
    /// moves to anybody else.
    SelfMatched,
}

/// Where a self-matched study reads each person besides their event
/// (`research.md` §1.4, C367): time-stratified referents (Janes,
/// Sheppard and Lumley, *Epidemiology* 16, 2005). The life is cut into
/// windows of `window_days` from birth; the event's window is read at
/// the event and every `step_days` either side of it inside the window.
/// The windows are fixed before the event is known, which is what keeps
/// the referents from biasing the estimate, and none starts before birth.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Referents {
    /// Each window's length in days; a person's instants are those of
    /// their event's window.
    pub window_days: f64,
    /// The days between a person's instants.
    pub step_days: f64,
}

impl Referents {
    /// Referents every `step_days` inside windows of `window_days`.
    #[must_use]
    pub const fn new(window_days: f64, step_days: f64) -> Referents {
        Referents {
            window_days,
            step_days,
        }
    }

    /// `INVALID_ARG` naming the field when a length is not a positive
    /// number of days, or a window holds more than [`MAX_INSTANTS`]
    /// instants.
    fn check(self) -> Result<(), Error> {
        for (value, field) in [
            (self.window_days, "referents.windowDays"),
            (self.step_days, "referents.stepDays"),
        ] {
            if !(value.is_finite() && value > 0.0) {
                return Err(Error::invalid_arg(format!(
                    "{value} is not a positive number of days"
                ))
                .with_field(field));
            }
        }
        if self.window_days / self.step_days > f64::from(MAX_INSTANTS) {
            return Err(Error::invalid_arg(format!(
                "a window of {} days read every {} days holds more than {MAX_INSTANTS} instants; lengthen the step or shorten the window",
                self.window_days, self.step_days
            ))
            .with_field("referents.stepDays"));
        }
        Ok(())
    }

    /// A subject's instants: its event first, then the others of its
    /// event's window, earliest first.
    fn of(self, subject: &Subject) -> Vec<JulianDay<Utc>> {
        let (born, event) = (subject.birth.instant, subject.event);
        let age = event.get() - born.get();
        let start = (age / self.window_days).floor() * self.window_days;
        let end = start + self.window_days;
        let before = ((age - start) / self.step_days).floor();
        let mut instants = vec![event];
        let mut k = -before;
        loop {
            let at = age + k * self.step_days;
            if at >= end {
                break;
            }
            if k != 0.0
                && at >= start
                && let Ok(instant) = born.plus_days(at)
            {
                instants.push(instant);
            }
            k += 1.0;
        }
        instants
    }
}

/// An event study: its subjects, how each chart is read, the rules whose
/// delivery is tested, the dasha that delivers them and the shuffle.
#[derive(Clone, Debug)]
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
    /// The dasha whose periods deliver: a catalogued system, or one the
    /// context registered.
    pub dasha: DashaName,
    /// How deep the delivering periods are read: 1 is the mahadasha alone,
    /// 2 adds the antardasha, and a rule holds when any of them delivers.
    pub depth: Depth,
    /// What the null keeps.
    pub shuffle: EventShuffle,
    /// When a rule counts as formed.
    pub holds: Holds,
    /// Where each person is read besides their event, which
    /// [`EventShuffle::SelfMatched`] requires and the others refuse.
    pub referents: Option<Referents>,
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
        dasha: impl Into<DashaName>,
        shuffle: EventShuffle,
    ) -> EventStudy<'s> {
        EventStudy {
            subjects,
            request,
            rules,
            dasha: dasha.into(),
            depth: Depth::try_new(2).unwrap_or(Depth::MIN),
            shuffle,
            holds: Holds::Standing,
            referents: None,
        }
    }

    /// The same study self-matched: each person read at their event and
    /// at `referents`, the null drawing which of them is the event.
    #[must_use]
    pub const fn self_matched(mut self, referents: Referents) -> EventStudy<'s> {
        self.shuffle = EventShuffle::SelfMatched;
        self.referents = Some(referents);
        self
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
struct EventInput<'a> {
    subjects: &'a [Subject],
    rules: Vec<&'a str>,
    dasha: KeyId,
    depth: u8,
    shuffle: EventShuffle,
    #[serde(skip_serializing_if = "Option::is_none")]
    referents: Option<Referents>,
    holds: Holds,
    strata: Option<&'a [u32]>,
    test: &'a EventTest,
    read: Vec<Hash>,
}

/// Gauquelin's control: the sample refounded `replicates` times, each
/// subject's date and place kept and its clock time taken from another
/// subject, drawn inside strata (`research.md` §1.3).
#[derive(Clone, Debug, PartialEq, Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
    /// (or less) often than at events shuffled among them. A rule that
    /// [reads transits](Rule::reads_transits) is read instead under the sky
    /// at each instant, founded once per place and instant, and its timing
    /// is the transit's own, so no period delivers it (crux C382).
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming its field: an event before its own birth
    /// (`subjects[i].event`); a rule the language cannot evaluate
    /// (`rules[i]`); more instants of the sky than `test.max_pairs`
    /// allows (`maxPairs`, with the count it would need); a registered
    /// dasha this context has no system for (`dasha`); whatever reading
    /// the births or the dasha refuses; and whatever
    /// [`teistro_research::timed`] refuses.
    pub fn timed(
        self,
        study: &EventStudy<'_>,
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
        let plan = Plan::of(study, strata)?;
        // Every instant each subject is read at: each pairing's, or its own
        // referents.
        let wanted: Vec<Vec<JulianDay<Utc>>> = match &plan {
            Plan::Paired(_) => (0..study.subjects.len())
                .map(|i| {
                    (0..study.subjects.len())
                        .filter_map(|j| event_instant(study, i, j))
                        .collect()
                })
                .collect(),
            Plan::Matched(referents) => study.subjects.iter().map(|s| referents.of(s)).collect(),
        };
        let skies = self.skies(study, &births, &wanted, test.max_pairs)?;
        let (charts, read, provenance) = self.lives(study, &births)?;
        let readings = study.rules.readings().readings();
        let evaluators: Vec<Option<(Evaluator<'_>, &Lived)>> = charts
            .iter()
            .map(|lived| lived.as_ref().map(|l| (l.inputs.evaluator(readings), l)))
            .collect();
        let cell =
            |r: usize, rule: &Rule, i: usize, at: Option<JulianDay<Utc>>| match evaluators.get(i) {
                Some(Some((evaluator, lived))) => match &skies {
                    Some(skies) if rule.reads_transits() => {
                        let sky = at.and_then(|at| skies.get(i, at));
                        in_transit(study, (r, rule), evaluator, lived, sky)
                    }
                    _ => delivered(study, (r, rule), evaluator, lived, at),
                },
                _ => Cell::Unreadable,
            };
        let value = match plan {
            Plan::Paired(mut pairs) => {
                for (r, rule) in study.rules.rules().iter().enumerate() {
                    pairs.push(rule.key.clone(), |i, j| {
                        cell(r, rule, i, event_instant(study, i, j))
                    })?;
                }
                timed(&pairs, strata, test)?
            }
            Plan::Matched(_) => {
                let instants = &wanted;
                let counts: Vec<u16> = instants
                    .iter()
                    .map(|own| u16::try_from(own.len()).unwrap_or(u16::MAX))
                    .collect();
                let mut matrix = MatchedMatrix::new(&counts)?;
                for (r, rule) in study.rules.rules().iter().enumerate() {
                    matrix.push(rule.key.clone(), |i, k| {
                        cell(
                            r,
                            rule,
                            i,
                            instants.get(i).and_then(|own| own.get(k)).copied(),
                        )
                    })?;
                }
                matched(&matrix, test)?
            }
        };
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
            dasha: self.dasha_id(&study.dasha)?,
            depth: study.depth.get(),
            shuffle: study.shuffle,
            referents: study.referents,
            holds: study.holds,
            strata,
            test: &registered,
            read,
        });
        Ok(Envelope::sealing(value, provenance))
    }

    /// The sky in transit at every instant a subject is read at, founded a
    /// place at a time, when a rule reads transits; none otherwise, so a
    /// study of natal rules alone pays for no sky. Refused above
    /// `max_pairs` instants, naming how many the study would need.
    fn skies(
        self,
        study: &EventStudy<'_>,
        births: &[Birth],
        wanted: &[Vec<JulianDay<Utc>>],
        max_pairs: u64,
    ) -> Result<Option<Skies>, Error> {
        if !study.rules.rules().iter().any(Rule::reads_transits) {
            return Ok(None);
        }
        let groups = by_place(births);
        // Each place's distinct instants, earliest first.
        let instants: Vec<Vec<JulianDay<Utc>>> = groups
            .iter()
            .map(|members| {
                let mut at: Vec<JulianDay<Utc>> = members
                    .iter()
                    .filter_map(|&i| wanted.get(i))
                    .flatten()
                    .copied()
                    .collect();
                at.sort_by(|a, b| a.get().total_cmp(&b.get()));
                at.dedup_by(|a, b| a.get().to_bits() == b.get().to_bits());
                at
            })
            .collect();
        let needed: u64 = instants.iter().map(|at| at.len() as u64).sum();
        if needed > max_pairs {
            return Err(Error::invalid_arg(format!(
                "a rule reads transits, and this study reads the sky at {needed} instants, above `maxPairs`, {max_pairs}"
            ))
            .with_field("test.maxPairs")
            .with_hint(format!(
                "raise `maxPairs` to {needed} to pay for it, or study fewer subjects; the age shuffle reads the sky at every pairing, the event-date shuffle at each event a place at a time"
            )));
        }
        let mut subject_place = vec![0; births.len()];
        let mut skies = Vec::with_capacity(groups.len());
        for (g, (members, at)) in groups.iter().zip(&instants).enumerate() {
            for &i in members {
                if let Some(slot) = subject_place.get_mut(i) {
                    *slot = g;
                }
            }
            let Some(place) = members
                .first()
                .and_then(|&i| births.get(i))
                .map(|b| b.place)
            else {
                continue;
            };
            let read = self
                .context
                .chart()
                .founding(UtcOffset::UTC, |founder| founder.longitudes(at, &place))?;
            let sky: BTreeMap<u64, TransitSky> =
                at.iter()
                    .zip(read.value)
                    .map(|(instant, longitudes)| {
                        let signs = Graha::ALL.into_iter().zip(longitudes).map(|(graha, l)| {
                            (graha, teistro_gochar::Transit::at_longitude(l).sign)
                        });
                        (instant.get().to_bits(), TransitSky::new(signs))
                    })
                    .collect();
            skies.push(sky);
        }
        Ok(Some(Skies {
            subject_place,
            skies,
        }))
    }

    /// Every subject's chart, rule results and dasha, read a place at a
    /// time, with every crossing's input hash and the first's provenance.
    /// The id a study's dasha is read under: a catalogued system's own, or
    /// the one this context registered the key under, refused at `dasha`
    /// when it registered none.
    fn dasha_id(self, dasha: &DashaName) -> Result<KeyId, Error> {
        match dasha {
            DashaName::Catalogued(system) => Ok(KeyId::from(*system)),
            DashaName::Registered(_) => self
                .context
                .keys()
                .id(&dasha.full_key())
                .map_err(|refusal| refusal.with_field("dasha")),
        }
    }

    fn lives(self, study: &EventStudy<'_>, births: &[Birth]) -> Result<Lives, Error> {
        let mut charts: Vec<Option<Lived>> = (0..births.len()).map(|_| None).collect();
        let mut read = Vec::new();
        let mut provenance = None;
        let wanted = self.dasha_id(&study.dasha)?;
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
        study: &EventStudy<'_>,
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
            cursor: self.context.chart().dasha(document, &study.dasha)?,
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

/// How an event study draws its null: events shuffled among the
/// subjects, or each subject's event among its own referents.
enum Plan {
    Paired(PairMatrix),
    Matched(Referents),
}

impl Plan {
    /// The study's null, its referents and strata checked against its
    /// shuffle.
    fn of(study: &EventStudy<'_>, strata: Option<&[u32]>) -> Result<Plan, Error> {
        match (study.shuffle, study.referents) {
            (EventShuffle::SelfMatched, None) => Err(Error::invalid_arg(
                "a self-matched study reads each person at referent instants; give `referents`",
            )
            .with_field("referents")),
            (EventShuffle::SelfMatched, Some(_)) if strata.is_some() => Err(Error::invalid_arg(
                "a self-matched study already matches each person to themselves; strata have nothing left to hold",
            )
            .with_field("strata")),
            (EventShuffle::SelfMatched, Some(referents)) => {
                referents.check()?;
                Ok(Plan::Matched(referents))
            }
            (_, Some(_)) => Err(Error::invalid_arg(
                "referents are read only by a SELF_MATCHED study",
            )
            .with_field("referents")),
            (_, None) => Ok(Plan::Paired(pairings(study)?)),
        }
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
fn event_instant(study: &EventStudy<'_>, i: usize, j: usize) -> Option<JulianDay<Utc>> {
    let (own, other) = (study.subjects.get(i)?, study.subjects.get(j)?);
    match study.shuffle {
        EventShuffle::EventDates => Some(other.event),
        EventShuffle::AgesAtEvent => own
            .birth
            .instant
            .plus_days(other.event.get() - other.birth.instant.get())
            .ok(),
        EventShuffle::SelfMatched => (i == j).then_some(own.event),
    }
}

/// The study's pairings, each that would read a subject before its own
/// birth forbidden.
fn pairings(study: &EventStudy<'_>) -> Result<PairMatrix, Error> {
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

/// The sky in transit at each instant a study reads, by subject.
struct Skies {
    /// Each subject's place, an index into `skies`.
    subject_place: Vec<usize>,
    /// Each place's skies, by the instant's bits.
    skies: Vec<BTreeMap<u64, TransitSky>>,
}

impl Skies {
    fn get(&self, subject: usize, at: JulianDay<Utc>) -> Option<&TransitSky> {
        let place = *self.subject_place.get(subject)?;
        self.skies.get(place)?.get(&at.get().to_bits())
    }
}

/// Whether rule `r`, which reads transits, holds of a subject's chart
/// under the sky at an instant. Its timing is the transit's own, so no
/// period is asked to deliver it (crux C382); `holds` still decides
/// whether its cancellations undo it.
fn in_transit(
    study: &EventStudy<'_>,
    (r, rule): (usize, &Rule),
    evaluator: &Evaluator<'_>,
    lived: &Lived,
    sky: Option<&TransitSky>,
) -> Cell {
    if lived.unreadable.get(r).copied().unwrap_or(true) {
        return Cell::Unreadable;
    }
    let Some(sky) = sky else {
        return Cell::Unreadable;
    };
    let result = evaluator.with_transits(sky).evaluate(rule);
    let held = result.present
        && match study.holds {
            Holds::Formed => true,
            Holds::Standing => result.status != Some(NetStatus::FullyCancelled),
        };
    if held { Cell::Present } else { Cell::Absent }
}

/// Whether rule `r` of a subject's chart is delivered at `at`: formed on
/// the chart, and a period running then delivers it.
fn delivered(
    study: &EventStudy<'_>,
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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    fn subject(age: f64) -> Subject {
        let born = 2_440_000.5;
        Subject {
            birth: Birth::new(
                JulianDay::<Utc>::literal(born),
                Place::try_from_degrees(28.6, 77.2, 0.0).unwrap(),
                UtcOffset::literal(5, 30, 0),
            ),
            event: JulianDay::<Utc>::literal(born + age),
        }
    }

    /// Each instant's age in days.
    fn ages(referents: Referents, age: f64) -> Vec<f64> {
        let subject = subject(age);
        referents
            .of(&subject)
            .iter()
            .map(|at| at.get() - subject.birth.instant.get())
            .collect()
    }

    #[test]
    fn referents_are_the_events_window_read_every_step_the_event_first() {
        let month = Referents::new(28.0, 7.0);
        // Day 30 of a life is day 2 of its second window, 28 to 56.
        assert_eq!(ages(month, 30.0), [30.0, 37.0, 44.0, 51.0]);
        assert_eq!(ages(month, 50.0), [50.0, 29.0, 36.0, 43.0]);
        // A window's first and last instants stay inside it.
        assert_eq!(ages(month, 28.0), [28.0, 35.0, 42.0, 49.0]);
        assert_eq!(ages(month, 55.5), [55.5, 34.5, 41.5, 48.5]);
        // The first window starts at birth, so nothing is read before it.
        assert_eq!(ages(month, 3.0), [3.0, 10.0, 17.0, 24.0]);
        // A window that does not divide by the step reads what fits.
        assert_eq!(ages(Referents::new(10.0, 4.0), 1.0), [1.0, 5.0, 9.0]);
        assert_eq!(ages(Referents::new(10.0, 4.0), 3.0), [3.0, 7.0]);
        // A step longer than the window leaves the event alone.
        assert_eq!(ages(Referents::new(5.0, 7.0), 12.0), [12.0]);
    }

    #[test]
    fn referents_refuse_lengths_that_are_not_days() {
        for (referents, field) in [
            (Referents::new(0.0, 7.0), "referents.windowDays"),
            (Referents::new(f64::NAN, 7.0), "referents.windowDays"),
            (Referents::new(28.0, -1.0), "referents.stepDays"),
            (Referents::new(28.0, f64::INFINITY), "referents.stepDays"),
            (Referents::new(1_001.0, 1.0), "referents.stepDays"),
        ] {
            assert_eq!(referents.check().unwrap_err().field(), Some(field));
        }
        assert!(Referents::new(1_000.0, 1.0).check().is_ok());
    }
}
