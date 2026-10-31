//! The façade's record entry points: every area a binding reaches with one
//! JSON record, read by the request type's own reader and answered as
//! canonical JSON (`03-design/mcp-server.md` D3).
//!
//! One table, so that a caller dispatching records by name, the agent
//! server first, keeps no list of the SDK's operations of its own. Each
//! entry names the boundary function that crosses the same record, whose
//! documentation in `idl/api.json` is the record's description.

use teistro_core::envelope::{Envelope, Provenance, canonical_json};
use teistro_core::error::Error;

use crate::Context;

/// One record entry point.
///
/// ```
/// # use teistro::{Context, Ephemeris};
/// # fn main() -> Result<(), teistro::Error> {
/// let sdk = Context::builder().ephemeris([Ephemeris::None]).build()?;
/// let convert = teistro::records::record("calendar.convert").expect("every build carries it");
/// let answered = convert.answer(
///     &sdk,
///     r#"{"date": {"year": 2026, "month": 10, "day": 10}, "into": "calendar.BIKRAM_SAMBAT"}"#,
/// )?;
/// assert_eq!(convert.boundary, "ts_calendar_convert");
/// assert_eq!(answered.value["weekday"], "SATURDAY");
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Copy)]
pub struct Record {
    /// Its name, `area.operation`, as a tool is called.
    pub name: &'static str,
    /// The boundary function that crosses the same record and documents it.
    pub boundary: &'static str,
    /// A title for a person choosing it.
    pub title: &'static str,
    /// What the record is, where the boundary function's documentation
    /// describes a C request rather than a JSON record.
    pub reads: Option<&'static str>,
    run: fn(&Context, &str) -> Result<Answered, Error>,
    #[cfg_attr(
        not(feature = "schema"),
        expect(dead_code, reason = "read only by the `schema` feature's methods")
    )]
    schemas: Schemas,
}

/// Writes one type's JSON Schema into a generator, answering the
/// reference to it.
#[cfg(feature = "schema")]
type SchemaFn = fn(&mut schemars::SchemaGenerator) -> schemars::Schema;

/// A record's request and answer as JSON Schema, each derived from the
/// type serde reads or writes; empty without the `schema` feature, and
/// either one absent where its type does not carry one yet.
#[derive(Clone, Copy, Default)]
struct Schemas {
    #[cfg(feature = "schema")]
    request: Option<SchemaFn>,
    #[cfg(feature = "schema")]
    answer: Option<SchemaFn>,
    /// The records the request carries inside it by name, each read by
    /// its own reader: what a caller may ask the schema of one at a time.
    #[cfg(feature = "schema")]
    parts: &'static [&'static str],
}

/// What a record's schema says of a part whose reader reads no single
/// type yet: an object, which the reader checks; never stricter than it.
#[cfg(feature = "schema")]
pub(crate) const UNSTATED: &str = "its schema is not stated yet; the record's own reader checks it";

/// A part whose schema is not stated yet, by the name it is written under.
#[cfg(feature = "schema")]
pub(crate) fn unstated(name: &str) -> schemars::Schema {
    let said = format!("The `{name}` record: {UNSTATED}.");
    schemars::json_schema!({ "type": "object", "description": said })
}

/// A theme as a record writes it: a shipped theme to start from and the
/// changes laid over it, each group naming only what it changes, so no
/// part is required.
#[cfg(all(feature = "schema", feature = "svg"))]
pub(crate) struct ThemePatch;

#[cfg(all(feature = "schema", feature = "svg"))]
impl schemars::JsonSchema for ThemePatch {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("ThemePatch")
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "object",
            "properties": {
                "extends": {
                    "type": "string",
                    "description": "The shipped theme the changes are laid over; the light one when left out.",
                },
                "style": { "type": "object", "description": "The style's changes." },
                "content": { "type": "object", "description": "The content's changes." },
            },
        })
    }
}

/// `T`'s schema, inlined, with `extra`'s parts beside its own: the shape
/// of a reader that takes those parts out by name and reads the rest as
/// `T`, so the whole is as strict as `T` and no stricter.
#[cfg(feature = "schema")]
pub(crate) fn beside<T: schemars::JsonSchema>(
    generator: &mut schemars::SchemaGenerator,
    extra: Vec<(&str, schemars::Schema)>,
) -> schemars::Schema {
    let mut schema = T::json_schema(generator);
    if let Some(properties) = schema
        .get_mut("properties")
        .and_then(serde_json::Value::as_object_mut)
    {
        for (name, part) in extra {
            properties.insert(name.to_owned(), part.to_value());
        }
    }
    schema
}

/// `schema` as a definition of its own under `name`, answered as a
/// reference to it: a schema written by hand ([`beside`]) for a field of a
/// request is otherwise inlined, and a binding has to name its type after
/// the field it sits in.
#[cfg(all(feature = "schema", feature = "western"))]
pub(crate) fn defined(
    generator: &mut schemars::SchemaGenerator,
    name: &str,
    schema: schemars::Schema,
) -> schemars::Schema {
    let path = generator.settings().definitions_path.clone();
    let path = path.strip_prefix('#').unwrap_or(&path);
    let path = path.strip_suffix('/').unwrap_or(path);
    generator
        .definitions_mut()
        .insert(name.to_owned(), schema.to_value());
    schemars::Schema::new_ref(format!("#{path}/{name}"))
}

/// The schemas of a record whose request reader reads `$request` and
/// whose answer serde writes from `$answer`; `schemas!()` for one whose
/// types carry none yet.
macro_rules! schemas {
    () => {
        Schemas::default()
    };
    (fn $request:path => $answer:ty, parts: $parts:expr) => {
        Schemas {
            #[cfg(feature = "schema")]
            request: Some($request),
            #[cfg(feature = "schema")]
            answer: Some(schemars::SchemaGenerator::subschema_for::<$answer>),
            #[cfg(feature = "schema")]
            parts: &$parts,
        }
    };
    ($request:ty => $answer:ty) => {
        Schemas {
            #[cfg(feature = "schema")]
            request: Some(schemars::SchemaGenerator::subschema_for::<$request>),
            #[cfg(feature = "schema")]
            answer: Some(schemars::SchemaGenerator::subschema_for::<$answer>),
            #[cfg(feature = "schema")]
            parts: &[],
        }
    };
}

impl core::fmt::Debug for Record {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Record")
            .field("name", &self.name)
            .field("boundary", &self.boundary)
            .finish_non_exhaustive()
    }
}

impl Record {
    /// The record `json` read and answered under `context`.
    ///
    /// ```
    /// # use teistro::{Context, Ephemeris};
    /// # fn main() -> Result<(), teistro::Error> {
    /// let sdk = Context::builder().ephemeris([Ephemeris::None]).build()?;
    /// let resolve = teistro::records::record("time.resolve").expect("every build carries it");
    /// // A field the record does not have is refused, not ignored.
    /// let refused = resolve.answer(&sdk, r#"{"when": "noon"}"#).unwrap_err();
    /// assert_eq!(refused.status, teistro::Status::InvalidArg);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Whatever the record's reader refuses, named under its area, and
    /// whatever the area refuses once it runs.
    pub fn answer(&self, context: &Context, json: &str) -> Result<Answered, Error> {
        (self.run)(context, json)
    }

    /// The record's request as JSON Schema, written into `generator`:
    /// the reference to it, its definitions left in the generator. Never
    /// stricter than the reader, because it is derived from the type the
    /// reader deserialises. `None` where that type carries no schema yet.
    #[cfg(feature = "schema")]
    #[must_use]
    pub fn request_schema(
        &self,
        generator: &mut schemars::SchemaGenerator,
    ) -> Option<schemars::Schema> {
        self.schemas.request.map(|schema| schema(generator))
    }

    /// The records the request carries inside it by name, each read by
    /// its own reader (`chart.found`'s `kp`, `almanac.days`'s `muhurta`):
    /// the parts a caller may ask the schema of one at a time, where the
    /// whole would be more than a tool list should carry.
    #[cfg(feature = "schema")]
    #[must_use]
    pub fn parts(&self) -> &'static [&'static str] {
        self.schemas.parts
    }

    /// The record's answer value as JSON Schema, as
    /// [`Record::request_schema`] writes the request's.
    #[cfg(feature = "schema")]
    #[must_use]
    pub fn answer_schema(
        &self,
        generator: &mut schemars::SchemaGenerator,
    ) -> Option<schemars::Schema> {
        self.schemas.answer.map(|schema| schema(generator))
    }
}

/// What a record answers: its value, and its provenance where the area
/// seals one. An answer that reads no sky and no setting, a numerology
/// profile or a match of two names, has none.
///
/// ```
/// # use teistro::{Context, Ephemeris};
/// # fn main() -> Result<(), teistro::Error> {
/// let sdk = Context::builder().ephemeris([Ephemeris::None]).build()?;
/// let convert = teistro::records::record("calendar.convert").expect("every build carries it");
/// let answered = convert.answer(
///     &sdk,
///     r#"{"date": {"year": 2026, "month": 10, "day": 10}, "into": "calendar.BIKRAM_SAMBAT"}"#,
/// )?;
/// // The answer speaks serde's bare spelling of a catalogue member.
/// assert_eq!(answered.value["date"]["calendar"], "BIKRAM_SAMBAT");
/// assert_eq!(answered.value["date"]["year"], 2083);
/// assert!(answered.provenance.is_none(), "a date converts without a sky");
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Answered {
    /// The value, as serde writes it.
    pub value: serde_json::Value,
    /// How it was produced.
    pub provenance: Option<Provenance>,
}

impl Answered {
    fn sealed<T: serde::Serialize>(answer: Envelope<T>) -> Result<Answered, Error> {
        Ok(Answered {
            value: written(&answer.value)?,
            provenance: Some(answer.provenance),
        })
    }

    fn plain<T: serde::Serialize>(value: &T) -> Result<Answered, Error> {
        Ok(Answered {
            value: written(value)?,
            provenance: None,
        })
    }

    /// The answer as canonical JSON: `{value, provenance}` where it is
    /// sealed, the value alone where it is not.
    ///
    /// ```
    /// # use teistro::{Context, Ephemeris};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let sdk = Context::builder().ephemeris([Ephemeris::None]).build()?;
    /// let convert = teistro::records::record("calendar.convert").expect("every build carries it");
    /// let answered = convert.answer(
    ///     &sdk,
    ///     r#"{"date": {"year": 2026, "month": 10, "day": 10}, "into": "calendar.BIKRAM_SAMBAT"}"#,
    /// )?;
    /// let json = answered.json();
    /// assert!(json.starts_with(r#"{"date":"#), "{json}");
    /// assert_eq!(serde_json::from_str::<serde_json::Value>(&json)?, answered.value);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn json(&self) -> String {
        match &self.provenance {
            Some(provenance) => canonical_json(&Envelope::new(&self.value, provenance.clone())),
            None => canonical_json(&self.value),
        }
    }
}

fn written<T: serde::Serialize>(value: &T) -> Result<serde_json::Value, Error> {
    serde_json::to_value(value)
        .map_err(|why| Error::internal(format!("an answer serde cannot write: {why}")))
}

/// Every record entry point this build carries, in name order.
///
/// ```
/// let names: Vec<&str> = teistro::records::records().iter().map(|record| record.name).collect();
/// assert!(names.contains(&"calendar.convert"));
/// assert!(names.is_sorted());
/// ```
#[must_use]
pub fn records() -> Vec<Record> {
    let mut all: Vec<Record> = vec![
        #[cfg(feature = "chart")]
        Record {
            name: "matching.naam",
            boundary: "ts_naam_milan",
            title: "Match two names by their first syllables",
            reads: None,
            run: |_, json| Answered::plain(&crate::NaamRequest::from_json(json)?.answer()?),
            schemas: schemas!(crate::NaamRequest => crate::NaamMilan),
        },
        #[cfg(feature = "chart")]
        Record {
            name: "chart.found",
            boundary: "ts_chart_found",
            title: "Found charts and read every table asked of them",
            reads: Some(crate::FoundRequest::DESCRIPTION),
            run: |context, json| {
                let found = crate::FoundRequest::from_json(json)?.resolved(context.keys())?;
                let composed =
                    context
                        .chart()
                        .compose(&found.instants, &found.request, &found.records)?;
                Ok(Answered {
                    value: written(&composed)?,
                    provenance: Some(composed.founded.provenance.clone()),
                })
            },
            schemas: schemas!(fn crate::found_request::schema => crate::Composed<'static>, parts: crate::ChartRecords::NAMES),
        },
        #[cfg(feature = "numerology")]
        Record {
            name: "numerology.profile",
            boundary: "ts_numerology_profile",
            title: "A name and a birth date read under numerology",
            reads: None,
            run: |_, json| Answered::plain(&crate::NumerologyRequest::from_json(json)?.answer()?),
            schemas: schemas!(crate::NumerologyRequest => teistro_numerology::Profile),
        },
        #[cfg(feature = "pakshi")]
        Record {
            name: "almanac.pakshi",
            boundary: "ts_pakshi",
            title: "A native's bird over days, by Pancha Pakshi",
            reads: None,
            run: |context, json| {
                let asked = crate::PakshiRequest::from_json(json)?;
                Answered::sealed(context.almanac().pakshi_request(&asked)?)
            },
            schemas: schemas!(crate::pakshi_request::RequestAsked => Vec<crate::PakshiDay>),
        },
        #[cfg(feature = "rashifal")]
        Record {
            name: "chart.rashifal",
            boundary: "ts_rashifal",
            title: "A period read for each of the twelve signs",
            reads: None,
            run: |context, json| {
                let asked = crate::RashifalBatch::from_json(json)?;
                Answered::sealed(context.chart().rashifal_answers(&asked)?)
            },
            schemas: schemas!(crate::rashifal_request::BatchAsked => Vec<crate::RashifalAnswer>),
        },
        #[cfg(feature = "research")]
        Record {
            name: "research.study",
            boundary: "ts_research",
            title: "Count or test rules over a batch of births",
            reads: None,
            run: |context, json| {
                let asked = crate::ResearchRequest::from_json(json)?;
                match context.research().request(&asked)? {
                    crate::ResearchAnswer::Counts(table) => Answered::sealed(table),
                    crate::ResearchAnswer::Tested(tested) => Answered::sealed(tested),
                }
            },
            schemas: schemas!(crate::research_request::RequestAsked => crate::research_request::Studied),
        },
    ];
    all.extend(intl_records());
    #[cfg(feature = "chart")]
    all.extend(naming_records());
    all.extend(almanac_records());
    all.extend(almanac_lookup_records());
    all.extend(time_records());
    all.extend(engine_records());
    all.sort_by_key(|record| record.name);
    all
}

/// The locale engine's records.
fn intl_records() -> [Record; 3] {
    [
        Record {
            name: "intl.inWords",
            boundary: "ts_intl_in_words",
            title: "A number or an amount of money in words and figures",
            reads: None,
            run: |context, json| {
                Answered::plain(
                    &context
                        .intl()
                        .in_words(&crate::InWordsRequest::from_json(json)?)?,
                )
            },
            schemas: schemas!(crate::InWordsRequest => crate::InWords),
        },
        Record {
            name: "intl.time",
            boundary: "ts_intl_time",
            title: "A time of day as text and parts, in the locale's pattern",
            reads: None,
            run: |context, json| {
                Answered::plain(
                    &context
                        .intl()
                        .time(&crate::TimeTextRequest::from_json(json)?)?,
                )
            },
            schemas: schemas!(crate::TimeTextRequest => crate::CivilText),
        },
        Record {
            name: "intl.date",
            boundary: "ts_intl_date",
            title: "A date as text and parts, in the locale's pattern",
            reads: None,
            run: |context, json| {
                Answered::plain(
                    &context
                        .intl()
                        .date(&crate::DateTextRequest::from_json(json)?)?,
                )
            },
            schemas: schemas!(crate::DateTextRequest => crate::CivilText),
        },
    ]
}

/// The naming of a child's records: the five `NamingArea` operations a
/// request answers whole. `load` is none, since what it answers is the
/// context it changes (`03-design/namakarana.md` §10).
#[cfg(feature = "chart")]
fn naming_records() -> [Record; 5] {
    [
        Record {
            name: "naming.syllables",
            boundary: "ts_naming_syllables",
            title: "The syllables a pada, a star or a sign names a child by",
            reads: None,
            run: |context, json| {
                Answered::plain(&crate::NamingSyllablesRequest::from_json(json)?.answer(context)?)
            },
            schemas: schemas!(crate::NamingSyllablesRequest => Vec<teistro_matching::PadaSyllable>),
        },
        Record {
            name: "naming.read",
            boundary: "ts_naming_read",
            title: "Names read into the cakra, each with its clauses",
            reads: None,
            run: |context, json| {
                Answered::plain(&crate::NamingReadRequest::from_json(json)?.answer(context))
            },
            schemas: schemas!(crate::NamingReadRequest => Vec<crate::NameRead>),
        },
        Record {
            name: "naming.check",
            boundary: "ts_naming_check",
            title: "Names checked against a birth's pada",
            reads: None,
            run: |context, json| {
                Answered::plain(&crate::NamingCheckRequest::from_json(json)?.answer(context)?)
            },
            schemas: schemas!(crate::NamingCheckRequest => Vec<crate::NameChecked>),
        },
        Record {
            name: "naming.of",
            boundary: "ts_naming_of",
            title: "The names a birth gives",
            reads: None,
            run: |context, json| {
                Answered::plain(&crate::NamingOfRequest::from_json(json)?.answer(context)?)
            },
            schemas: schemas!(crate::NamingOfRequest => teistro_matching::Names),
        },
        Record {
            name: "naming.suggest",
            boundary: "ts_naming_suggest",
            title: "The names a book suggests for a birth",
            reads: None,
            run: |context, json| {
                Answered::plain(&crate::NamingSuggestRequest::from_json(json)?.answer(context)?)
            },
            schemas: schemas!(crate::NamingSuggestRequest => teistro_matching::Suggestions),
        },
    ]
}

/// The almanac's records every build carries over a range or a death: its
/// change points, an ashauch, a first year, its days with what is asked
/// beside them, and its festivals as a feed.
fn almanac_records() -> [Record; 6] {
    [
        Record {
            name: "almanac.anniversaries",
            boundary: "ts_almanac_anniversaries",
            title: "Anniversaries kept by the tithi over a span of years, for many people in one call, each day naming the rule that chose it",
            reads: None,
            run: |context, json| {
                let asked = crate::AnniversariesRequest::from_json(json)?;
                Answered::sealed(context.almanac().anniversaries_in_full(&asked)?)
            },
            schemas: schemas!(crate::area::AnniversariesAsked => teistro_panchanga::festival::Observances),
        },
        Record {
            name: "almanac.changes",
            boundary: "ts_almanac_changes",
            title: "Every instant in a range at which the tithi, nakshatra, yoga, karana, the Moon's or Sun's sign or the vara changes, one sorted list",
            reads: None,
            run: |context, json| {
                let asked = crate::ChangesRequest::from_json(json)?;
                Answered::sealed(context.almanac().changes(&asked)?)
            },
            schemas: schemas!(crate::area::ChangesAsked => crate::ChangePoints),
        },
        Record {
            name: "almanac.ashauch",
            boundary: "ts_almanac_ashauch",
            title: "The period of impurity after a birth or a death, and the clause that set it",
            reads: None,
            run: |context, json| {
                let asked = crate::AshauchQuery::from_json(json)?;
                Answered::sealed(context.almanac().ashauch_query(&asked)?)
            },
            schemas: schemas!(crate::ashauch_request::AshauchAsked => crate::ashauch::Ashauch),
        },
        Record {
            name: "almanac.firstYear",
            boundary: "ts_almanac_first_year",
            title: "A death's first year: the sixteen shraddhas, the sapindana and the first abdika, each on its day",
            reads: None,
            run: |context, json| {
                let asked = crate::FirstYearQuery::from_json(json)?;
                Answered::sealed(context.almanac().first_year_query(&asked)?)
            },
            schemas: schemas!(crate::first_year_request::FirstYearAsked => crate::first_year::FirstYearSchedule),
        },
        Record {
            name: "almanac.days",
            boundary: "ts_panchanga_days",
            title: "Every day of a range at a place, and what is asked beside them",
            reads: Some(crate::DaysRequest::DESCRIPTION),
            run: days,
            schemas: schemas!(fn crate::days_request::schema => crate::days_request::DaysWritten<'static>, parts: crate::days_request::PARTS),
        },
        Record {
            name: "almanac.festival_calendar",
            boundary: "ts_almanac_festival_calendar",
            title: "A range's festivals as an iCalendar feed",
            reads: None,
            run: festival_calendar,
            schemas: schemas!(crate::area::FestivalCalendarAsked => String),
        },
    ]
}

/// The rest of the almanac's records: what a pack holds, a digest, the next
/// day of a rule, why a published day differs, and a sankalpa's time.
fn almanac_lookup_records() -> [Record; 6] {
    [
        Record {
            name: "almanac.festivalPack",
            boundary: "ts_almanac_festival_pack",
            title: "Every rule of a festival pack with what it is: kind, tradition, communities and regions, and the fields no source gives",
            reads: None,
            run: |context, json| {
                let asked = crate::PackRequest::from_json(json)?;
                Answered::plain(&context.almanac().festival_pack(&asked))
            },
            schemas: schemas!(crate::area::PackAsked => crate::PackListing),
        },
        Record {
            name: "almanac.digest",
            boundary: "ts_almanac_digest",
            title: "A range of days as a compact digest a small process reads with no engine",
            reads: None,
            run: digest,
            schemas: schemas!(crate::area::DigestAsked => DigestWritten),
        },
        Record {
            name: "almanac.next",
            boundary: "ts_almanac_next",
            title: "The next day a festival rule or a kind of day (Ekadashi, Purnima, Amavasya, Sankranti) falls on after an instant at a place",
            reads: None,
            run: |context, json| {
                let asked = crate::NextRequest::from_json(json)?;
                Answered::sealed(context.almanac().next_in_full(&asked)?)
            },
            schemas: schemas!(crate::area::NextAsked => crate::NextOccurrence),
        },
        Record {
            name: "almanac.sentences",
            boundary: "ts_almanac_sentences",
            title: "One day's facts as short declarative sentences, every value taken from the day's answer, in a locale that carries the templates",
            reads: None,
            run: |context, json| {
                let asked = crate::SentencesRequest::from_json(json)?;
                Answered::sealed(context.almanac().sentences(&asked)?)
            },
            schemas: schemas!(crate::area::SentencesAsked => crate::DaySentences),
        },
        Record {
            name: "almanac.publishedReasons",
            boundary: "ts_almanac_published_reasons",
            title: "Why each published festival day differs from the rule's: the place, the sky or the shipped rule that gives it, or the publisher's own choice",
            reads: None,
            // The agent server computes over the one engine its command
            // line chose (`mcp-server.md` D6), so no other sky is tried;
            // the places and the shipped packs are.
            run: |context, json| {
                let asked = crate::PublishedReasonsRequest::from_json(json)?;
                Answered::sealed(context.almanac().published_reasons_request(&[], &asked)?)
            },
            schemas: schemas!(crate::area::ReasonsAsked => crate::PublishedReasons),
        },
        Record {
            name: "almanac.sankalpa",
            boundary: "ts_almanac_sankalpa",
            title: "The time a sankalpa recites at an instant and a place",
            reads: None,
            run: |context, json| {
                let asked = crate::SankalpaRequest::from_json(json)?;
                Answered::sealed(context.almanac().sankalpa_request(&asked)?)
            },
            schemas: schemas!(crate::sankalpa_request::SankalpaAsked => crate::Sankalpa),
        },
    ]
}

/// The engine passthrough (ADR-0030): what the engine says it offers,
/// and one of its own operations called by name. Each answer is sealed
/// with a provenance naming the engine, because nothing of the SDK
/// computed it.
fn engine_records() -> [Record; 2] {
    [
        Record {
            name: "engine.call",
            boundary: "ts_ephemeris_call",
            title: "One of the engine's own operations, called by name",
            reads: Some(crate::EngineCall::DESCRIPTION),
            run: |context, json| {
                let asked = crate::EngineCall::from_json(json)?;
                let arguments = serde_json::Value::Object(asked.arguments.clone());
                let value = context.engine().call(&asked.name, &arguments)?;
                engine_answer(context, &asked, &asked.name, value)
            },
            schemas: schemas!(crate::EngineCall => serde_json::Value),
        },
        Record {
            name: "engine.manifest",
            boundary: "ts_ephemeris_manifest",
            title: "Everything the engine offers of its own, with each operation's parameters",
            reads: Some(crate::ManifestRequest::DESCRIPTION),
            run: |context, json| {
                let asked = crate::ManifestRequest::from_json(json)?;
                let manifest = context.engine().manifest()?;
                engine_answer(context, &asked, "manifest", written(&manifest)?)
            },
            schemas: schemas!(crate::ManifestRequest => crate::NativeManifest),
        },
    ]
}

/// An engine's answer sealed: the request's hash, and the engine's
/// identity with the operation as its one step.
fn engine_answer<T: serde::Serialize>(
    context: &Context,
    asked: &T,
    step: &str,
    value: serde_json::Value,
) -> Result<Answered, Error> {
    let provenance = context.stamped(
        teistro_core::envelope::content_hash(asked),
        teistro_port_ephemeris::Frame::CANONICAL,
        vec![format!("engine.{step}")],
    );
    Answered::sealed(Envelope::sealing(value, provenance))
}

/// The clock and calendar records: what `TimeArea` and `CalendarArea`
/// answer, each read from its own request record.
fn time_records() -> [Record; 6] {
    [
        Record {
            name: "time.resolve",
            boundary: "ts_time_resolve",
            title: "A civil date and time in a zone, as an instant",
            reads: Some(crate::ResolveRequest::DESCRIPTION),
            run: |context, json| {
                let asked = crate::ResolveRequest::from_json(json)?;
                Answered::plain(&context.time().resolve(&asked.civil, &asked.zone)?)
            },
            schemas: schemas!(crate::time_request::ResolveAsked => crate::Resolved),
        },
        Record {
            name: "time.civil",
            boundary: "ts_time_civil",
            title: "An instant read on a zone's clock, in a calendar",
            reads: Some(crate::CivilRequest::DESCRIPTION),
            run: |context, json| {
                let asked = crate::CivilRequest::from_json(json)?;
                let (civil, zone) =
                    context
                        .time()
                        .civil_of(asked.instant, &asked.zone, asked.calendar)?;
                Answered::plain(&crate::CivilReading { civil, zone })
            },
            schemas: schemas!(crate::time_request::CivilAsked => crate::CivilReading),
        },
        Record {
            name: "time.convert",
            boundary: "ts_time_convert",
            title: "An instant carried between UT1, TT and UTC, and what was applied",
            reads: Some(crate::ScaleRequest::DESCRIPTION),
            run: |context, json| {
                let asked = crate::ScaleRequest::from_json(json)?;
                Answered::plain(&context.time().convert(asked.jd, asked.from, asked.to)?)
            },
            schemas: schemas!(crate::ScaleRequest => crate::scale::Conversion),
        },
        Record {
            name: "calendar.convert",
            boundary: "ts_calendar_convert",
            title: "A date written in another calendar, with its weekday",
            reads: Some(crate::CalendarRequest::DESCRIPTION),
            run: |context, json| {
                let asked = crate::CalendarRequest::from_json(json)?;
                let calendars = context.calendar();
                let date = calendars.convert(&asked.date, asked.into)?;
                let weekday = calendars.weekday_of(&date)?;
                Answered::plain(&crate::CalendarReading { date, weekday })
            },
            schemas: schemas!(crate::time_request::CalendarAsked => crate::CalendarReading),
        },
        Record {
            name: "calendar.add",
            boundary: "ts_calendar_add",
            title: "A date moved by years, months and days, with its weekday",
            reads: Some(crate::AddRequest::DESCRIPTION),
            run: |context, json| {
                let asked = crate::AddRequest::from_json(json)?;
                let calendars = context.calendar();
                let date = calendars.add(&asked.date, asked.span, asked.overflow)?;
                let weekday = calendars.weekday_of(&date)?;
                Answered::plain(&crate::CalendarReading { date, weekday })
            },
            schemas: schemas!(crate::time_request::AddAsked => crate::CalendarReading),
        },
        Record {
            name: "calendar.between",
            boundary: "ts_calendar_between",
            title: "The years, months and days between two dates, counted as an age",
            reads: Some(crate::BetweenRequest::DESCRIPTION),
            run: |context, json| {
                let asked = crate::BetweenRequest::from_json(json)?;
                Answered::plain(&context.calendar().between(
                    &asked.from,
                    &asked.to,
                    asked.overflow,
                )?)
            },
            schemas: schemas!(crate::time_request::BetweenAsked => crate::DateSpan),
        },
    ]
}

/// The record entry point called `name`, if this build carries it.
///
/// ```
/// let convert = teistro::records::record("calendar.convert").expect("every build carries it");
/// assert_eq!(convert.name, "calendar.convert");
/// assert!(teistro::records::record("calendar.unknown").is_none());
/// ```
#[must_use]
pub fn record(name: &str) -> Option<Record> {
    records().into_iter().find(|record| record.name == name)
}

/// `almanac.festival_calendar`: the feed's text, sealed with the festival
/// answer's provenance (`03-design/icalendar.md`).
fn festival_calendar(context: &Context, json: &str) -> Result<Answered, Error> {
    let asked = crate::FestivalCalendarRequest::from_json(json)?;
    Answered::sealed(context.almanac().festival_calendar_request(&asked)?)
}

/// What a tool answers for a digest, whose bytes a text answer cannot
/// hold as they are.
#[derive(serde::Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
struct DigestWritten {
    /// The format's major version.
    format: u32,
    /// How many days it holds.
    days: u32,
    /// Its length in bytes.
    size: usize,
    /// Its seal: the last 64 bytes, SHA-256 in hexadecimal.
    seal: String,
    /// The bytes, as standard Base64.
    base64: String,
}

/// `almanac.digest`: the bytes, for a tool, as text
/// (`03-design/digest-format.md` §6).
fn digest(context: &Context, json: &str) -> Result<Answered, Error> {
    let asked = crate::DigestRequest::from_json(json)?;
    let answer = context.almanac().digest(&asked)?;
    let seal = answer.value.len().saturating_sub(64);
    let written = DigestWritten {
        format: crate::DIGEST_FORMAT_MAJOR,
        days: asked.days,
        size: answer.value.len(),
        seal: String::from_utf8_lossy(answer.value.get(seal..).unwrap_or_default()).into_owned(),
        base64: base64(&answer.value),
    };
    Answered::sealed(Envelope {
        value: written,
        provenance: answer.provenance,
    })
}

/// Standard Base64 with padding (RFC 4648 §4).
fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let at = |index: usize| u32::from(chunk.get(index).copied().unwrap_or(0));
        let triple = (at(0) << 16) | (at(1) << 8) | at(2);
        for position in 0..4_usize {
            let sextet = (triple >> (18 - 6 * position)) & 63;
            let letter = ALPHABET.get(sextet as usize).copied().unwrap_or(b'=');
            out.push(if position <= chunk.len() {
                char::from(letter)
            } else {
                '='
            });
        }
    }
    out
}

/// `almanac.days`: the range's days founded once, with their own hashes
/// and every section asked beside them, each sealed as the boundary seals
/// it ([`AlmanacAnswer::sections`](crate::AlmanacAnswer::sections)); the
/// days' provenance is the answer's.
fn days(context: &Context, json: &str) -> Result<Answered, Error> {
    let asked = crate::DaysRequest::from_json(json)?;
    let answer = asked.clock.asked_on(
        context.almanac(),
        (&asked.first, &asked.last),
        &asked.place,
        &asked.beside,
    )?;
    let value = written(&crate::days_request::DaysWritten {
        days: &answer.days.value,
        day_hashes: &answer.day_hashes,
        sections: answer.sections()?,
    })?;
    Ok(Answered {
        value,
        provenance: Some(answer.days.provenance),
    })
}
