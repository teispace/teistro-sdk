//! A chart request read whole from one JSON record: when, where, the
//! sections, and every record beside them (`03-design/mcp-server.md`,
//! step 2). It is what [`ChartArea::compose`](crate::ChartArea::compose)
//! takes, spelt as every other request record is, so the agent server's
//! `chart.found` reads it and nothing else composes the request.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use teistro_core::catalogue::{Catalogued, ChartFrame, ChartKind, ChartLayout, DashaSystem, Varga};
use teistro_core::envelope::Convention;
use teistro_core::error::Error;
use teistro_core::key::KeyId;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::strict;
use teistro_port_geo::ResolvedPlace;
use teistro_time::ZoneSpec;

use crate::asked::Where;
use crate::{ChartRecords, ChartRequest, KeysArea};

/// A section a chart request asks for by a `true` flag, and the request
/// that reads it.
type Section = (&'static str, fn(ChartRequest) -> ChartRequest);

/// The sections beside the foundation, by the flag every binding writes.
const SECTIONS: [Section; 14] = [
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
    ("moment", ChartRequest::with_moment),
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
    /// The dashas and drawings as asked, kept while one names a member no
    /// catalogue has, for [`FoundRequest::resolved`] to read through a
    /// context's registries.
    unresolved: Option<Unresolved>,
    /// Where the observer's height came from, for a request on a resolved
    /// `place` (`03-design/geo-port.md` §6a).
    height: Option<Convention>,
}

/// The members a request asked for, in the order asked, where one of
/// them is a name only a context's registry can resolve.
#[derive(Clone, Debug)]
struct Unresolved {
    dashas: Vec<Keyed<DashaSystem>>,
    drawings: Vec<(Keyed<ChartLayout>, Varga, ChartFrame)>,
}

impl FoundRequest {
    /// Where the observer's height came from, for a request on a resolved
    /// `place`: the record adds it to the answer's provenance.
    pub(crate) fn height(&self) -> Option<&Convention> {
        self.height.as_ref()
    }

    /// What a JSON chart request is, for a caller choosing it by name (the
    /// agent server's `chart.found`).
    pub const DESCRIPTION: &'static str = "`request` is a chart request: `instant` (a UTC Julian day) or `instants` (a batch, a chart each); `latitudeDeg`, `longitudeDeg`, `altitudeM` (optional) and `utcOffsetSeconds`, or `zone`, a zone as `time.resolve` takes it, or in place of the coordinates and the zone a `place` as `geo.resolve` answers it (its coordinates, its zone or `utcOffsetSeconds` where it has none, and its ground height as the observer's unless `altitudeM` states one, kept in the request so a receipt replays without the provider, the provenance naming where the height came from), each birth then read at the offset the zone had then (Kathmandu +05:30 in 1985, +05:45 now) and each chart founded on a zone or local mean time carrying it as its foundation's `clock`; optional `kind`, `vargas`, `dashas` and `drawings` (`{layout, varga, countsFrom}`, `countsFrom` one of `LAGNA` (the default), `MOON`, `SUN` and `BHAVA`, the bhava chalit) by their catalogue keys, bare or full (`varga.D9` or `D9`), a dasha system or layout the server registered by its key, and the `theme` the drawings are written as SVG in; a `true` flag for each section beside the foundation (`aspects`, `points`, `houses`, `ashtakavarga`, `vimshopaka`, `vaiseshikamsa`, `shadbala`, `bhavaBala`, `dashaPhala`, `jaimini`, `avakahada`, `moment`, `outerPlanets`, `state`); and a record for each table read off the charts, each the record its area reads (`rules`, `interpret`, `varsha`, `gochar`, `hits`, `sadeSati`, `kp`, `prashna`, `remedies`, `lalkitab`, `rectification`, `dignities`, `fortitudes`, `lots`, `considerations`, `perfection`, `progressions`, `westernAspects`, `synastry`, `parallels`, `antiscia`, `midpoints`, `westernHouses`, `harmonic`, `matching`, `sarvatobhadra`, `tithiPravesha`, `dossier`, `masaPravesha`, `westernConfigurations`). The answer's `charts` are the chart documents, and each table is a list with a row a chart, empty where its record was not sent; its `provenance.time.clock` states the first birth's clock.";

    /// The sections beside the foundation a request asks for by a `true`
    /// flag, by the flag every binding writes (`shadbala`, `bhavaBala`).
    ///
    /// ```
    /// assert!(teistro::FoundRequest::sections().contains(&"jaimini"));
    /// ```
    #[must_use]
    pub fn sections() -> [&'static str; SECTIONS.len()] {
        SECTIONS.map(|(name, _)| name)
    }

    /// The request a binding writes as a chart request's options, read and
    /// checked: `instant` or `instants`; `latitudeDeg`, `longitudeDeg`,
    /// `altitudeM` and `utcOffsetSeconds` or a `zone`; `kind`, `vargas`, `dashas` and
    /// `drawings` by their catalogue keys, bare or full; a `true` flag a
    /// section (`shadbala`, `jaimini`, …); and each record by its name
    /// ([`ChartRecords::NAMES`]). A refusal names the field as written,
    /// a record's from the record's own root (`kp.clock`).
    ///
    /// A dasha system or a drawing's layout may also be one a context
    /// registered (`dasha_system.ACME_SAPTAKA` or `ACME_SAPTAKA`): only the
    /// context knows it, so such a request is [`FoundRequest::resolved`]
    /// before it is composed, and until then its `request` asks for no
    /// dashas or drawings at all.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, a value out of range, two records asking for one table, or
    /// both `utcOffsetSeconds` and `zone` (named `zone`);
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
        let (at, height) = asked.at()?;
        let at = match asked.kind {
            Some(kind) => at.with_kind(kind),
            None => at,
        };
        let request = sections
            .into_iter()
            .fold(at, |request, add| add(request))
            .with_vargas(asked.vargas);
        let drawings: Vec<(Keyed<ChartLayout>, Varga, ChartFrame)> = asked
            .drawings
            .into_iter()
            .map(|drawing| (drawing.layout, drawing.varga, drawing.counts_from))
            .collect();
        let catalogued = asked.dashas.iter().all(Keyed::is_catalogued)
            && drawings.iter().all(|(layout, _, _)| layout.is_catalogued());
        let (request, unresolved) = if catalogued {
            let request = request
                .with_dashas(asked.dashas.iter().filter_map(Keyed::catalogued))
                .with_drawings(drawings.iter().filter_map(|(layout, varga, from)| {
                    Some((layout.catalogued()?, *varga, *from))
                }));
            (request, None)
        } else {
            let unresolved = Unresolved {
                dashas: asked.dashas,
                drawings,
            };
            (request, Some(unresolved))
        };
        Ok(FoundRequest {
            instants,
            request,
            records: records.checked()?,
            unresolved,
            height,
        })
    }

    /// The request with every dasha system and layout a context registered
    /// read through `keys` ([`Context::keys`](crate::Context::keys)), in the
    /// order asked; a request naming only catalogued members comes back as
    /// it was.
    ///
    /// ```
    /// use teistro::{Context, FoundRequest};
    ///
    /// let sdk = Context::builder().build()?;
    /// let found = FoundRequest::from_json(
    ///     r#"{"instant": 2447000.25, "latitudeDeg": 27.7, "longitudeDeg": 85.3,
    ///         "utcOffsetSeconds": 20700, "dashas": ["ACME_SAPTAKA"]}"#,
    /// )?;
    /// let refused = found.resolved(sdk.keys()).unwrap_err();
    /// assert_eq!(refused.field(), Some("dashas[0]"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming the place asked (`dashas[1]`,
    /// `drawings[0].layout`) for a key neither the catalogue nor the
    /// context's registry has, or a key of another kind.
    pub fn resolved(self, keys: KeysArea<'_>) -> Result<FoundRequest, Error> {
        let Some(unresolved) = self.unresolved else {
            return Ok(self);
        };
        let dashas = unresolved
            .dashas
            .iter()
            .enumerate()
            .map(|(at, system)| system.id(keys, || format!("dashas[{at}]")))
            .collect::<Result<Vec<_>, _>>()?;
        let drawings = unresolved
            .drawings
            .iter()
            .enumerate()
            .map(|(at, (layout, varga, from))| {
                Ok((
                    layout.id(keys, || format!("drawings[{at}].layout"))?,
                    *varga,
                    *from,
                ))
            })
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(FoundRequest {
            request: self.request.with_dashas(dashas).with_drawings(drawings),
            unresolved: None,
            ..self
        })
    }
}

/// A member asked for by its key: a catalogued one, bare or full, or a
/// name only a context's registry can resolve.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Keyed<T> {
    Catalogued(T),
    Named(String),
}

impl<T: Catalogued> Keyed<T> {
    fn is_catalogued(&self) -> bool {
        matches!(self, Keyed::Catalogued(_))
    }

    fn catalogued(&self) -> Option<T> {
        match self {
            Keyed::Catalogued(member) => Some(*member),
            Keyed::Named(_) => None,
        }
    }

    /// The member's id: a catalogued one's own, or the one `keys` gives a
    /// registered key, refused at `field` for a key of another kind or
    /// one nobody has.
    fn id(&self, keys: KeysArea<'_>, field: impl Fn() -> String) -> Result<KeyId, Error> {
        let name = match self {
            Keyed::Catalogued(member) => return Ok(member.key_id()),
            Keyed::Named(name) => name,
        };
        let kind = T::KIND.name();
        let full = match name.split_once('.') {
            None => format!("{kind}.{name}"),
            Some((asked, _)) if asked == kind => name.clone(),
            Some((asked, _)) => {
                return Err(
                    Error::invalid_arg(format!("`{name}` is a `{asked}`, not a `{kind}`"))
                        .with_field(field())
                        .with_hint(format!("name a `{kind}` here, bare or as `{kind}.KEY`")),
                );
            }
        };
        keys.id(&full).map_err(|refusal| {
            let hint = refusal.hint().map_or_else(
                || format!("a catalogued `{kind}`, or one the context registered"),
                str::to_owned,
            );
            refusal.with_field(field()).with_hint(hint)
        })
    }
}

impl<T: Catalogued> Serialize for Keyed<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Keyed::Catalogued(member) => serializer.serialize_str(member.key()),
            Keyed::Named(name) => serializer.serialize_str(name),
        }
    }
}

impl<'de, T: Catalogued> Deserialize<'de> for Keyed<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let key = <std::borrow::Cow<'_, str>>::deserialize(deserializer)?;
        Ok(T::from_either_key(&key)
            .map_or_else(|_| Keyed::Named(key.into_owned()), Keyed::Catalogued))
    }
}

/// The catalogue's keys, or a key a context registered: what the reader
/// above accepts, and never less.
#[cfg(feature = "schema")]
impl<T: Catalogued + schemars::JsonSchema> schemars::JsonSchema for Keyed<T> {
    fn inline_schema() -> bool {
        true
    }

    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Owned(format!("Keyed{}", T::schema_name()))
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({ "anyOf": [
            generator.subschema_for::<T>(),
            { "type": "string", "pattern": REGISTERED_KEY,
              "description": "a member the context registered, by its key, bare or full" },
        ] })
    }
}

/// A key a context may register, bare or under its kind.
#[cfg(feature = "schema")]
const REGISTERED_KEY: &str = "^([a-z_]+\\.)?[A-Z][A-Z0-9_]*$";

/// The parts a tool list names rather than states: each record read off
/// the charts, then the resolved place.
#[cfg(feature = "schema")]
pub(crate) const PARTS: [&str; ChartRecords::NAMES.len() + 1] = {
    let mut parts = [crate::asked::PLACE; ChartRecords::NAMES.len() + 1];
    let mut slots: &mut [&str] = &mut parts;
    let mut names: &[&str] = &ChartRecords::NAMES;
    while let ([slot, after @ ..], [name, more @ ..]) = (slots, names) {
        *slot = *name;
        slots = after;
        names = more;
    }
    parts
};

/// The record's schema: the request as [`Asked`] reads it, each section
/// a flag beside it, and each record by its own reader's schema.
#[cfg(feature = "schema")]
pub(crate) fn schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
    let mut extra: Vec<(&str, schemars::Schema)> = SECTIONS
        .iter()
        .map(|(name, _)| (*name, schemars::json_schema!({ "type": "boolean" })))
        .collect();
    for name in ChartRecords::NAMES {
        let schema =
            ChartRecords::schema(name, generator).unwrap_or_else(|| crate::records::unstated(name));
        extra.push((name, schema));
    }
    crate::records::beside::<Asked>(generator, extra)
}

/// The request as a binding writes it, the sections and records taken out.
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Asked {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    instant: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    instants: Option<Vec<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    latitude_deg: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    longitude_deg: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    altitude_m: Option<f64>,
    /// A place as `geo.resolve` answered it, in place of `latitudeDeg`,
    /// `longitudeDeg` and `zone`: its coordinates, its zone, and its ground
    /// height as the observer's unless `altitudeM` states one, kept in the
    /// request so a receipt replays without the provider.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    place: Option<ResolvedPlace>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    utc_offset_seconds: Option<i32>,
    /// The zone, in place of `utcOffsetSeconds`: each birth read at the
    /// zone's offset then.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    zone: Option<ZoneSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    kind: Option<ChartKind>,
    #[serde(default)]
    vargas: Vec<Varga>,
    #[serde(default)]
    dashas: Vec<Keyed<DashaSystem>>,
    #[serde(default)]
    drawings: Vec<DrawingAsked>,
}

/// A drawing: a layout of a divisional chart, its houses counted from the
/// lagna unless `countsFrom` names the Moon, the Sun or the bhavas.
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
struct DrawingAsked {
    layout: Keyed<ChartLayout>,
    varga: Varga,
    #[serde(default = "lagna", rename = "countsFrom")]
    counts_from: ChartFrame,
}

const fn lagna() -> ChartFrame {
    ChartFrame::Lagna
}

impl Asked {
    /// Where and on which clock: the coordinates, height and clock written
    /// out, or a resolved `place`, with where its height came from.
    fn at(&self) -> Result<(ChartRequest, Option<Convention>), Error> {
        let placed = Where {
            latitude_deg: self.latitude_deg,
            longitude_deg: self.longitude_deg,
            altitude_m: self.altitude_m,
            utc_offset_seconds: self.utc_offset_seconds,
            zone: self.zone.clone(),
            place: self.place.as_ref(),
        }
        .placed()?;
        Ok((ChartRequest::on(placed.place, placed.clock), placed.height))
    }

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
