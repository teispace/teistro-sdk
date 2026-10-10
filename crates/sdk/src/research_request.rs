//! A study as a binding sends it (`03-design/research.md` §2): one record
//! naming which of the four studies it is, its births or subjects, its
//! rules and what the study reads, refused by where it stands under
//! `research`.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use teistro_core::catalogue::{Catalogued as _, DashaSystem};
use teistro_core::envelope::Envelope;
use teistro_core::error::Error;
use teistro_core::quantity::{Depth, JulianDay, Utc};
use teistro_research::{Counts, Design, EventTest, GroupTest, ReplicateTest, Tested};

use crate::area::ResearchArea;
use crate::area::research::{Birth, EventShuffle, EventStudy, Holds, Recombine, Study, Subject};
use crate::asked::{offset_of, place_of};
use crate::reading::ChartRequest;
use crate::rule_request::{RuleRequest, RuleSet};

/// The record's name where a binding sends it, which a refusal is named
/// under.
const RESEARCH: &str = "research";

/// Which study a request runs, and what that study alone reads.
#[derive(Clone, Debug, PartialEq)]
pub enum ResearchStudy {
    /// How often each rule holds in each group ([`ResearchArea::counts`]).
    Counts {
        /// The births.
        births: Vec<Birth>,
        /// Who is in which group.
        design: Design,
    },
    /// Whether the groups differ on each rule ([`ResearchArea::compare`]).
    Compare {
        /// The births.
        births: Vec<Birth>,
        /// Who is in which group.
        design: Design,
        /// The permutation test.
        test: GroupTest,
    },
    /// Whether each rule is commoner than in the sample's recombined
    /// population ([`ResearchArea::expected`]).
    Expected {
        /// The births.
        births: Vec<Birth>,
        /// How the population is refounded.
        control: Recombine,
        /// How the sample is read against it.
        test: ReplicateTest,
    },
    /// Whether each rule is delivered at the subjects' own events more
    /// often than at shuffled ones ([`ResearchArea::timed`]).
    Timed {
        /// The subjects.
        subjects: Vec<Subject>,
        /// The dasha whose periods deliver.
        dasha: DashaSystem,
        /// How deep its periods are read.
        depth: Depth,
        /// What the null keeps.
        shuffle: EventShuffle,
        /// Each subject's stratum, when events move only inside one.
        strata: Option<Vec<u32>>,
        /// The permutation test.
        test: EventTest,
    },
}

/// A study as a binding asks it: its rules, when a rule holds, and the
/// study itself.
#[derive(Clone, Debug, PartialEq)]
pub struct ResearchRequest {
    /// The predicates, one family for the corrections.
    pub rules: RuleSet,
    /// When a rule counts as holding.
    pub holds: Holds,
    /// The study and what it reads.
    pub study: ResearchStudy,
}

/// What a study answers: the counts, or a test's rows, each with the
/// study's provenance, whose input hash is its pre-registration.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum ResearchAnswer {
    /// A `COUNTS` study's table.
    Counts(Envelope<Counts>),
    /// Any other study's rows.
    Tested(Envelope<Tested>),
}

/// What a study's value is, for its schema: the counts or a test's rows.
#[cfg(feature = "schema")]
#[derive(schemars::JsonSchema)]
#[schemars(untagged)]
#[allow(dead_code, reason = "a schema's shape, never built")]
pub(crate) enum Studied {
    /// A `COUNTS` study's table.
    Counts(Counts),
    /// Any other study's rows.
    Tested(Tested),
}

/// The four studies, as a request names them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum StudyKind {
    Counts,
    Compare,
    Expected,
    Timed,
}

/// A birth as a binding writes it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct BirthAsked {
    instant: f64,
    latitude_deg: f64,
    longitude_deg: f64,
    #[serde(default)]
    altitude_m: f64,
    utc_offset_seconds: i32,
    #[serde(default)]
    uncertainty_minutes: f64,
}

/// A subject as a binding writes it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SubjectAsked {
    birth: BirthAsked,
    event: f64,
}

/// The request as a binding writes it. `rules`, `test` and `control` stay
/// JSON until the study is known, because each is read by the reader the
/// study gives it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct RequestAsked {
    study: StudyKind,
    #[serde(default)]
    births: Option<Vec<BirthAsked>>,
    #[serde(default)]
    subjects: Option<Vec<SubjectAsked>>,
    rules: Value,
    #[serde(default)]
    holds: Holds,
    #[serde(default)]
    design: Option<Design>,
    #[serde(default)]
    test: Option<Value>,
    #[serde(default)]
    control: Option<Value>,
    #[serde(default)]
    dasha: Option<String>,
    #[serde(default)]
    depth: Option<u8>,
    #[serde(default)]
    shuffle: Option<EventShuffle>,
    #[serde(default)]
    strata: Option<Vec<u32>>,
}

impl BirthAsked {
    fn birth(self) -> Result<Birth, Error> {
        let instant = JulianDay::<Utc>::try_new(self.instant)
            .map_err(|why| Error::from(why).with_field("instant"))?;
        let place = place_of(self.latitude_deg, self.longitude_deg, self.altitude_m)?;
        Ok(
            Birth::new(instant, place, offset_of(self.utc_offset_seconds)?)
                .uncertain_by(self.uncertainty_minutes),
        )
    }
}

/// Each element read by `read`, a refusal named by its index under
/// `field`.
fn each<A: Copy, T>(
    field: &str,
    asked: &[A],
    read: impl Fn(A) -> Result<T, Error>,
) -> Result<Vec<T>, Error> {
    asked
        .iter()
        .enumerate()
        .map(|(index, &one)| read(one).map_err(|why| why.under(&format!("{field}[{index}]"))))
        .collect()
}

/// A list a study reads, refused by name when it is empty.
fn any<'a, T>(asked: &'a [T], field: &str) -> Result<&'a [T], Error> {
    if asked.is_empty() {
        return Err(
            Error::invalid_arg(format!("a study needs {field}, and this one has none"))
                .with_field(field),
        );
    }
    Ok(asked)
}

/// A field a study reads, refused by name when it is missing.
fn needed<T>(given: Option<T>, field: &str, study: &str) -> Result<T, Error> {
    given.ok_or_else(|| {
        Error::invalid_arg(format!(
            "a {study} study reads `{field}`, and it is missing"
        ))
        .with_field(field)
    })
}

/// A record read strictly from its JSON, its `seed` given as a number or,
/// past what a JavaScript number holds exactly, as a decimal string.
fn seeded<T: Serialize + serde::de::DeserializeOwned>(
    mut given: Value,
    field: &str,
) -> Result<T, Error> {
    if let Some(seed) = given.get_mut("seed")
        && let Value::String(text) = seed
    {
        let number: u64 = text.parse().map_err(|_| {
            Error::invalid_arg(format!(
                "a seed is a whole number from 0 to {}, not `{text}`",
                u64::MAX
            ))
            .with_field(format!("{field}.seed"))
        })?;
        *seed = Value::from(number);
    }
    teistro_core::strict::read_value(&given, field)
}

impl ResearchRequest {
    /// The record a binding sends, as JSON: `study`, one of `COUNTS`,
    /// `COMPARE`, `EXPECTED` and `TIMED`; `rules`, the record a chart
    /// request's rules are (`RuleRequest::from_json`); `holds`, optional;
    /// and what the study reads. `COUNTS` and `COMPARE` read `births` and
    /// `design`, `COMPARE` a `test` as well; `EXPECTED` reads `births`,
    /// `control` and an optional `test`; `TIMED` reads `subjects`, `dasha`,
    /// `shuffle`, a `test` and optionally `depth` and `strata`. A birth is
    /// `{instant, latitudeDeg, longitudeDeg, altitudeM, utcOffsetSeconds,
    /// uncertaintyMinutes}`, the instant a Julian day in UTC and the last
    /// and the altitude optional, and a subject `{birth, event}`. A seed is
    /// a number or a decimal string.
    ///
    /// ```
    /// use teistro::research::Contrast;
    /// use teistro::{ResearchRequest, ResearchStudy};
    ///
    /// let asked = ResearchRequest::from_json(
    ///     r#"{"study": "COMPARE", "rules": {"shipped": ["YOGAS"]},
    ///         "births": [
    ///           {"instant": 2447000.25, "latitudeDeg": 27.7, "longitudeDeg": 85.3, "utcOffsetSeconds": 20700},
    ///           {"instant": 2448000.75, "latitudeDeg": 27.7, "longitudeDeg": 85.3, "utcOffsetSeconds": 20700}],
    ///         "design": {"groups": [0, 1]},
    ///         "test": {"seed": "18446744073709551615", "permutations": 999,
    ///                  "contrast": {"kind": "CASE_VS_REST", "case": 1}}}"#,
    /// )?;
    /// let ResearchStudy::Compare { test, .. } = asked.study else { unreachable!() };
    /// assert_eq!(test.seed, u64::MAX);
    /// assert_eq!(test.contrast, Contrast::CaseVsRest { case: 1 });
    /// let stray = ResearchRequest::from_json(
    ///     r#"{"study": "COUNTS", "rules": {}, "births": [], "design": {"groups": []}, "dasha": "VIMSHOTTARI"}"#,
    /// )
    /// .unwrap_err();
    /// assert_eq!(stray.field(), Some("research.dasha"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, a field the study does not read or one it reads and is
    /// missing, a birth's place, offset or instant out of range, a depth
    /// outside 1 to 6, and whatever the rules, the design and the test
    /// refuse, each named under `research`, as `research.births[2].instant`.
    pub fn from_json(text: &str) -> Result<ResearchRequest, Error> {
        let asked: RequestAsked = teistro_core::strict::read(text, RESEARCH)?;
        asked.request().map_err(|why| why.under(RESEARCH))
    }
}

impl RequestAsked {
    /// Every field the study does not read, which is refused rather than
    /// ignored: a stray `design` on an event study is a study that is not
    /// the one its author meant.
    fn strays(&self) -> Result<(), Error> {
        let reads: &[&str] = match self.study {
            StudyKind::Counts => &["births", "design"],
            StudyKind::Compare => &["births", "design", "test"],
            StudyKind::Expected => &["births", "control", "test"],
            StudyKind::Timed => &["subjects", "dasha", "depth", "shuffle", "strata", "test"],
        };
        let given = [
            ("births", self.births.is_some()),
            ("subjects", self.subjects.is_some()),
            ("design", self.design.is_some()),
            ("test", self.test.is_some()),
            ("control", self.control.is_some()),
            ("dasha", self.dasha.is_some()),
            ("depth", self.depth.is_some()),
            ("shuffle", self.shuffle.is_some()),
            ("strata", self.strata.is_some()),
        ];
        match given
            .iter()
            .find(|(field, present)| *present && !reads.contains(field))
        {
            Some((field, _)) => Err(Error::invalid_arg(format!(
                "a {} study does not read `{field}`",
                self.name()
            ))
            .with_field(*field)),
            None => Ok(()),
        }
    }

    fn name(&self) -> &'static str {
        match self.study {
            StudyKind::Counts => "COUNTS",
            StudyKind::Compare => "COMPARE",
            StudyKind::Expected => "EXPECTED",
            StudyKind::Timed => "TIMED",
        }
    }

    fn births(&self) -> Result<Vec<Birth>, Error> {
        let asked = needed(self.births.as_deref(), "births", self.name())?;
        each("births", any(asked, "births")?, BirthAsked::birth)
    }

    fn test<T: Serialize + serde::de::DeserializeOwned>(&self) -> Result<T, Error> {
        seeded(needed(self.test.clone(), "test", self.name())?, "test")
    }

    fn request(self) -> Result<ResearchRequest, Error> {
        self.strays()?;
        let rules = RuleRequest::from_json(&self.rules.to_string())
            .map_err(|why| why.under("rules"))?
            .rule_set()
            .map_err(|why| why.under("rules"))?;
        let study = match self.study {
            StudyKind::Counts => ResearchStudy::Counts {
                births: self.births()?,
                design: needed(self.design.clone(), "design", self.name())?,
            },
            StudyKind::Compare => ResearchStudy::Compare {
                births: self.births()?,
                design: needed(self.design.clone(), "design", self.name())?,
                test: self.test()?,
            },
            StudyKind::Expected => ResearchStudy::Expected {
                births: self.births()?,
                control: seeded(
                    needed(self.control.clone(), "control", self.name())?,
                    "control",
                )?,
                test: match &self.test {
                    None => ReplicateTest::default(),
                    Some(_) => self.test()?,
                },
            },
            StudyKind::Timed => ResearchStudy::Timed {
                subjects: each(
                    "subjects",
                    any(
                        needed(self.subjects.as_deref(), "subjects", self.name())?,
                        "subjects",
                    )?,
                    |subject: SubjectAsked| {
                        Ok(Subject {
                            birth: subject.birth.birth().map_err(|why| why.under("birth"))?,
                            event: JulianDay::<Utc>::try_new(subject.event)
                                .map_err(|why| Error::from(why).with_field("event"))?,
                        })
                    },
                )?,
                dasha: DashaSystem::from_either_key(&needed(
                    self.dasha.clone(),
                    "dasha",
                    self.name(),
                )?)
                .map_err(|why| Error::invalid_arg(why.to_string()).with_field("dasha"))?,
                depth: Depth::try_new(self.depth.unwrap_or(2))
                    .map_err(|why| Error::from(why).with_field("depth"))?,
                shuffle: needed(self.shuffle, "shuffle", self.name())?,
                strata: self.strata.clone(),
                test: self.test()?,
            },
        };
        Ok(ResearchRequest {
            rules,
            holds: self.holds,
            study,
        })
    }
}

/// A study of `births` under the request's rules and its `holds`.
fn study_of<'s>(
    births: &'s [Birth],
    request: &'s ChartRequest,
    asked: &'s ResearchRequest,
) -> Study<'s> {
    Study::new(births, request, &asked.rules).holding(asked.holds)
}

impl ResearchArea<'_> {
    /// Runs a study a binding sent ([`ResearchRequest::from_json`]), each
    /// chart read with nothing but the rules' own inputs at its birth's
    /// place and clock.
    ///
    /// # Errors
    ///
    /// Whatever the study's own call refuses: [`ResearchArea::counts`],
    /// [`ResearchArea::compare`], [`ResearchArea::expected`] or
    /// [`ResearchArea::timed`].
    pub fn request(self, asked: &ResearchRequest) -> Result<ResearchAnswer, Error> {
        // Each birth's place and clock replace the template's, so the
        // first's is as good as any; a study with none is refused as the
        // façade refuses it.
        let template = |first: Option<&Birth>, field: &str| {
            first
                .map(|birth| ChartRequest::at(birth.place, birth.offset))
                .ok_or_else(|| {
                    Error::invalid_arg(format!("a study needs {field}, and this one has none"))
                        .with_field(field)
                })
        };
        match &asked.study {
            ResearchStudy::Counts { births, design } => {
                let request = template(births.first(), "births")?;
                self.counts(study_of(births, &request, asked), design)
                    .map(ResearchAnswer::Counts)
            }
            ResearchStudy::Compare {
                births,
                design,
                test,
            } => {
                let request = template(births.first(), "births")?;
                self.compare(study_of(births, &request, asked), design, test)
                    .map(ResearchAnswer::Tested)
            }
            ResearchStudy::Expected {
                births,
                control,
                test,
            } => {
                let request = template(births.first(), "births")?;
                self.expected(study_of(births, &request, asked), control, test)
                    .map(ResearchAnswer::Tested)
            }
            ResearchStudy::Timed {
                subjects,
                dasha,
                depth,
                shuffle,
                strata,
                test,
            } => {
                let request = template(subjects.first().map(|s| &s.birth), "subjects")?;
                let events = EventStudy::new(subjects, &request, &asked.rules, *dasha, *shuffle)
                    .to_depth(*depth)
                    .holding(asked.holds);
                self.timed(events, strata.as_deref(), test)
                    .map(ResearchAnswer::Tested)
            }
        }
    }
}
