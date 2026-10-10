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
        },
        Record {
            name: "almanac.days",
            boundary: "ts_panchanga_days",
            title: "Every day of a range at a place, and what is asked beside them",
            reads: Some(crate::DaysRequest::DESCRIPTION),
            run: days,
        },
        #[cfg(feature = "chart")]
        Record {
            name: "chart.found",
            boundary: "ts_chart_found",
            title: "Found charts and read every table asked of them",
            reads: Some(crate::FoundRequest::DESCRIPTION),
            run: |context, json| {
                let found = crate::FoundRequest::from_json(json)?;
                let composed =
                    context
                        .chart()
                        .compose(&found.instants, &found.request, &found.records)?;
                Ok(Answered {
                    value: written(&composed)?,
                    provenance: Some(composed.founded.provenance.clone()),
                })
            },
        },
        #[cfg(feature = "numerology")]
        Record {
            name: "numerology.profile",
            boundary: "ts_numerology_profile",
            title: "A name and a birth date read under numerology",
            reads: None,
            run: |_, json| Answered::plain(&crate::NumerologyRequest::from_json(json)?.answer()?),
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
    let mut value = serde_json::Map::new();
    value.insert(String::from("days"), written(&answer.days.value)?);
    value.insert(String::from("dayHashes"), written(&answer.day_hashes)?);
    if let serde_json::Value::Object(sections) = written(&answer.sections()?)? {
        value.extend(sections);
    }
    Ok(Answered {
        value: serde_json::Value::Object(value),
        provenance: Some(answer.days.provenance),
    })
}
