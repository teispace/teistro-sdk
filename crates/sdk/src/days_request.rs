//! A range of days and what is asked beside them, read whole from one
//! JSON record (`03-design/mcp-server.md`, step 2): what
//! [`AlmanacArea::asked`](crate::AlmanacArea::asked) takes, spelt as every
//! other request record is, so the agent server's `almanac.days` reads it
//! and nothing else composes the request.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use teistro_calendar::CalendarDate;
use teistro_core::catalogue::Calendar;
use teistro_core::error::Error;
use teistro_core::quantity::Place;
use teistro_core::strict;
use teistro_core::time::UtcOffset;

use crate::asked::{DayAsked, offset_of, place_of};
use crate::{AlmanacRequest, FestivalRequest};

/// A range of days read from JSON: the first and last day, the place and
/// clock, and what is asked beside the days.
///
/// ```
/// use teistro::DaysRequest;
///
/// let days = DaysRequest::from_json(
///     r#"{"first": {"year": 2026, "month": 10, "day": 10}, "latitudeDeg": 27.7172,
///         "longitudeDeg": 85.324, "utcOffsetSeconds": 20700, "years": true}"#,
/// )?;
/// assert_eq!(days.first, days.last);
/// assert!(days.beside.years());
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct DaysRequest {
    /// The range's first day.
    pub first: CalendarDate,
    /// Its last day, the first again for one day.
    pub last: CalendarDate,
    /// Where the days are reckoned.
    pub place: Place,
    /// The clock they are reckoned on.
    pub offset: UtcOffset,
    /// What is asked beside the days.
    pub beside: AlmanacRequest,
}

impl DaysRequest {
    /// What a JSON days request is, for a caller choosing it by name (the
    /// agent server's `almanac.days`).
    pub const DESCRIPTION: &'static str = "`request` is a range of days: `first` and optional `last` as `{year, month, day}` in `calendar` (a catalogue key, Gregorian when left out), `latitudeDeg`, `longitudeDeg`, `altitudeM` (optional) and `utcOffsetSeconds`; and beside the days, optionally, a `muhurta` search and a `festivals` reckoning, each the record its area reads, and `true` flags for the lunar `years`, the `eclipses` and each day's `nepalSambat` date. The answer's `days` are every day's panchanga, and each section asked is its own `{value, provenance}`.";

    /// The request a binding writes for a range of days, read and checked;
    /// a refusal names the field as written, a record's from the record's
    /// own root (`festivals.packs`).
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read or a value out of range; `CAPABILITY` for a `muhurta` search in
    /// a build without the family.
    pub fn from_json(text: &str) -> Result<DaysRequest, Error> {
        let Value::Object(mut fields) = strict::parse(text, "")? else {
            return Err(Error::invalid_arg("a days request is a JSON object").with_hint(
                "e.g. {\"first\": {\"year\": 2026, \"month\": 10, \"day\": 10}, \"latitudeDeg\": 27.7, \"longitudeDeg\": 85.3, \"utcOffsetSeconds\": 20700}",
            ));
        };
        let mut beside = AlmanacRequest::new();
        if let Some(muhurta) = fields.remove(MUHURTA) {
            beside = with_muhurta(beside, &muhurta.to_string())?;
        }
        if let Some(festivals) = fields.remove(FESTIVALS) {
            beside = beside.with_festivals(FestivalRequest::from_json(&festivals.to_string())?);
        }
        let asked: Asked = strict::read_value(&Value::Object(fields), "")?;
        let calendar = asked.calendar.unwrap_or(Calendar::Gregorian);
        let flags: [(bool, Asking); 3] = [
            (asked.years, AlmanacRequest::with_years),
            (asked.eclipses, AlmanacRequest::with_eclipses),
            (asked.nepal_sambat, AlmanacRequest::with_nepal_sambat),
        ];
        for (asked, add) in flags {
            if asked {
                beside = add(beside);
            }
        }
        Ok(DaysRequest {
            first: asked.first.in_calendar(calendar),
            last: asked.last.unwrap_or(asked.first).in_calendar(calendar),
            place: place_of(asked.latitude_deg, asked.longitude_deg, asked.altitude_m)?,
            offset: offset_of(asked.utc_offset_seconds)?,
            beside,
        })
    }
}

/// A section asked beside the days, as the request adds it.
type Asking = fn(AlmanacRequest) -> AlmanacRequest;

/// The record a muhurta search is written as.
const MUHURTA: &str = "muhurta";

/// The record a festival reckoning is written as.
const FESTIVALS: &str = "festivals";

/// `beside` with the muhurta search `json` asks for.
#[cfg(feature = "muhurta")]
fn with_muhurta(beside: AlmanacRequest, json: &str) -> Result<AlmanacRequest, Error> {
    Ok(beside.with_muhurta(crate::MuhurtaRequest::from_json(json)?))
}

/// A muhurta search, refused by a build without the family.
#[cfg(not(feature = "muhurta"))]
fn with_muhurta(_beside: AlmanacRequest, _json: &str) -> Result<AlmanacRequest, Error> {
    Err(Error::left_out(MUHURTA).with_field(MUHURTA))
}

/// The records a range of days carries inside it by name.
#[cfg(feature = "schema")]
pub(crate) const PARTS: [&str; 2] = [MUHURTA, FESTIVALS];

/// What `almanac.days` answers: the days, their own hashes, and each
/// section asked beside them, laid flat.
#[derive(Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub(crate) struct DaysWritten<'a> {
    pub(crate) days: &'a [teistro_panchanga::Panchanga],
    pub(crate) day_hashes: &'a [teistro_core::envelope::Hash],
    #[serde(flatten)]
    pub(crate) sections: crate::AlmanacSections,
}

/// The record's schema: the request as [`Asked`] reads it, with the
/// muhurta search and the festival reckoning beside it.
#[cfg(feature = "schema")]
pub(crate) fn schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
    #[cfg(feature = "muhurta")]
    let muhurta = generator.subschema_for::<crate::muhurta_request::Asked>();
    #[cfg(not(feature = "muhurta"))]
    let muhurta = crate::records::unstated(MUHURTA);
    let festivals = generator.subschema_for::<crate::festival_request::Asked>();
    crate::records::beside::<Asked>(generator, vec![(MUHURTA, muhurta), (FESTIVALS, festivals)])
}

/// The request as a binding writes it, the records taken out.
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Asked {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    calendar: Option<Calendar>,
    first: DayAsked,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last: Option<DayAsked>,
    latitude_deg: f64,
    longitude_deg: f64,
    #[serde(default)]
    altitude_m: f64,
    utc_offset_seconds: i32,
    #[serde(default)]
    years: bool,
    #[serde(default)]
    eclipses: bool,
    #[serde(default)]
    nepal_sambat: bool,
}
