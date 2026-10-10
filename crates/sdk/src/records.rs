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
        Record {
            name: "almanac.days",
            boundary: "ts_panchanga_days",
            title: "Every day of a range at a place, and what is asked beside them",
            reads: Some(crate::DaysRequest::DESCRIPTION),
            run: days,
            schemas: schemas!(fn crate::days_request::schema => crate::days_request::DaysWritten<'static>, parts: crate::days_request::PARTS),
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
    all.extend(time_records());
    all.extend(engine_records());
    all.sort_by_key(|record| record.name);
    all
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
fn time_records() -> [Record; 4] {
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
    ]
}

/// The record entry point called `name`, if this build carries it.
#[must_use]
pub fn record(name: &str) -> Option<Record> {
    records().into_iter().find(|record| record.name == name)
}

/// `almanac.days`: the range's days founded once, with their own hashes
/// and every section asked beside them, each sealed as the boundary seals
/// it ([`AlmanacAnswer::sections`](crate::AlmanacAnswer::sections)); the
/// days' provenance is the answer's.
fn days(context: &Context, json: &str) -> Result<Answered, Error> {
    let asked = crate::DaysRequest::from_json(json)?;
    let answer = context.almanac().asked(
        &asked.first,
        &asked.last,
        &asked.place,
        asked.offset,
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
