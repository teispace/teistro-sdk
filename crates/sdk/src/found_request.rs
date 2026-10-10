//! A chart request read whole from one JSON record: when, where, the
//! sections, and every record beside them (`03-design/mcp-server.md`,
//! step 2). It is what [`ChartArea::compose`](crate::ChartArea::compose)
//! takes, spelt as every other request record is, so the agent server's
//! `chart.found` reads it and nothing else composes the request.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use teistro_core::catalogue::{ChartKind, ChartLayout, DashaSystem, Varga};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::strict;

use crate::asked::{offset_of, place_of};
use crate::{ChartRecords, ChartRequest};

/// A section a chart request asks for by a `true` flag, and the request
/// that reads it.
type Section = (&'static str, fn(ChartRequest) -> ChartRequest);

/// The sections beside the foundation, by the flag every binding writes.
const SECTIONS: [Section; 13] = [
    ("aspects", ChartRequest::with_aspects),
    ("points", ChartRequest::with_points),
    ("houses", ChartRequest::with_houses),
    ("ashtakavarga", ChartRequest::with_ashtakavarga),
    ("vimshopaka", ChartRequest::with_vimshopaka),
    ("vaiseshikamsa", ChartRequest::with_vaiseshikamsa),
    ("shadbala", ChartRequest::with_shadbala),
    ("bhavaBala", ChartRequest::with_bhava_bala),
    ("dashaPhala", ChartRequest::with_dasha_phala),
    ("jaimini", ChartRequest::with_jaimini),
    ("avakahada", ChartRequest::with_avakahada),
    ("outerPlanets", ChartRequest::with_outer_planets),
    ("state", ChartRequest::with_state),
];

/// A chart request read from JSON: the births, the request every chart
/// is founded and read under, and the records beside it.
///
/// ```
/// use teistro::FoundRequest;
///
/// let found = FoundRequest::from_json(
///     r#"{"instant": 2447000.25, "latitudeDeg": 27.7172, "longitudeDeg": 85.324,
///         "utcOffsetSeconds": 20700, "vargas": ["varga.D9"], "shadbala": true,
///         "lots": {}}"#,
/// )?;
/// assert_eq!(found.instants.len(), 1);
/// assert!(found.records.lots.is_some());
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug)]
pub struct FoundRequest {
    /// The births, a chart each, in UTC Julian days.
    pub instants: Vec<JulianDay<Utc>>,
    /// Where, on which clock, of which kind, with which sections.
    pub request: ChartRequest,
    /// The records beside the sections, checked together.
    pub records: ChartRecords,
}

impl FoundRequest {
    /// What a JSON chart request is, for a caller choosing it by name (the
    /// agent server's `chart.found`).
    pub const DESCRIPTION: &'static str = "`request` is a chart request: `instant` (a UTC Julian day) or `instants` (a batch, a chart each); `latitudeDeg`, `longitudeDeg`, `altitudeM` (optional) and `utcOffsetSeconds`; optional `kind`, `vargas`, `dashas` and `drawings` (`{layout, varga}`) by their catalogue keys, bare or full (`varga.D9` or `D9`); a `true` flag for each section beside the foundation (`aspects`, `points`, `houses`, `ashtakavarga`, `vimshopaka`, `vaiseshikamsa`, `shadbala`, `bhavaBala`, `dashaPhala`, `jaimini`, `avakahada`, `outerPlanets`, `state`); and a record for each table read off the charts, each the record its area reads (`rules`, `interpret`, `varsha`, `gochar`, `hits`, `sadeSati`, `kp`, `prashna`, `remedies`, `lalkitab`, `rectification`, `dignities`, `fortitudes`, `lots`, `considerations`, `perfection`, `progressions`, `westernAspects`, `synastry`, `parallels`, `antiscia`, `midpoints`, `westernHouses`, `harmonic`, `matching`). The answer's `charts` are the chart documents, and each table is a list with a row a chart, empty where its record was not sent.";

    /// The request a binding writes as a chart request's options, read and
    /// checked: `instant` or `instants`; `latitudeDeg`, `longitudeDeg`,
    /// `altitudeM` and `utcOffsetSeconds`; `kind`, `vargas`, `dashas` and
    /// `drawings` by their catalogue keys, bare or full; a `true` flag a
    /// section (`shadbala`, `jaimini`, …); and each record by its name
    /// ([`ChartRecords::NAMES`]). A refusal names the field as written,
    /// a record's from the record's own root (`kp.clock`).
    ///
    /// A dasha system is a catalogued one: a context's registered systems
    /// are reached through [`ChartRequest::with_dashas`].
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, a value out of range, or two records asking for one table;
    /// `CAPABILITY` for a record of a family this build leaves out.
    pub fn from_json(text: &str) -> Result<FoundRequest, Error> {
        let Value::Object(mut fields) = strict::parse(text, "")? else {
            return Err(Error::invalid_arg("a chart request is a JSON object")
                .with_hint("e.g. {\"instant\": 2447000.25, \"latitudeDeg\": 27.7, \"longitudeDeg\": 85.3, \"utcOffsetSeconds\": 20700}"));
        };
        let mut records = ChartRecords::default();
        for name in ChartRecords::NAMES {
            if let Some(record) = fields.remove(name) {
                records.read(name, &record.to_string())?;
            }
        }
        let mut sections = Vec::new();
        for (name, add) in SECTIONS {
            match fields.remove(name) {
                None | Some(Value::Bool(false)) => {}
                Some(Value::Bool(true)) => sections.push(add),
                Some(_) => {
                    return Err(Error::invalid_arg(format!("`{name}` is a flag"))
                        .with_field(name)
                        .with_hint("write true to read the section, or leave it out"));
                }
            }
        }
        let asked: Asked = strict::read_value(&Value::Object(fields), "")?;
        let instants = asked.instants()?;
        let at = ChartRequest::at(
            place_of(asked.latitude_deg, asked.longitude_deg, asked.altitude_m)?,
            offset_of(asked.utc_offset_seconds)?,
        );
        let at = match asked.kind {
            Some(kind) => at.with_kind(kind),
            None => at,
        };
        let request = sections.into_iter().fold(at, |request, add| add(request));
        Ok(FoundRequest {
            instants,
            request: request
                .with_vargas(asked.vargas)
                .with_dashas(asked.dashas)
                .with_drawings(
                    asked
                        .drawings
                        .into_iter()
                        .map(|drawing| (drawing.layout, drawing.varga)),
                ),
            records: records.checked()?,
        })
    }
}

/// The request as a binding writes it, the sections and records taken out.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Asked {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    instant: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    instants: Option<Vec<f64>>,
    latitude_deg: f64,
    longitude_deg: f64,
    #[serde(default)]
    altitude_m: f64,
    utc_offset_seconds: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    kind: Option<ChartKind>,
    #[serde(default)]
    vargas: Vec<Varga>,
    #[serde(default)]
    dashas: Vec<DashaSystem>,
    #[serde(default)]
    drawings: Vec<DrawingAsked>,
}

/// A drawing: a layout of a divisional chart.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DrawingAsked {
    layout: ChartLayout,
    varga: Varga,
}

impl Asked {
    /// The births, from `instant` or `instants` and never both.
    fn instants(&self) -> Result<Vec<JulianDay<Utc>>, Error> {
        let (asked, field): (&[f64], &str) = match (&self.instant, &self.instants) {
            (Some(one), None) => (core::slice::from_ref(one), "instant"),
            (None, Some(many)) if !many.is_empty() => (many, "instants"),
            (Some(_), Some(_)) => {
                return Err(Error::invalid_arg("a chart request names its births twice")
                    .with_field("instants")
                    .with_hint(
                        "write `instant` for one birth or `instants` for a batch, not both",
                    ));
            }
            (None, _) => {
                return Err(Error::invalid_arg("no birth to found a chart at")
                    .with_field("instant")
                    .with_hint("name a UTC Julian day as `instant`, or a list as `instants`"));
            }
        };
        asked
            .iter()
            .enumerate()
            .map(|(at, jd)| {
                JulianDay::<Utc>::try_new(*jd).map_err(|why| {
                    let field = if field == "instant" {
                        field.to_owned()
                    } else {
                        format!("{field}[{at}]")
                    };
                    Error::from(why).with_field(field)
                })
            })
            .collect()
    }
}
