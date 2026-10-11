//! A civil time, an instant and a date read whole from one JSON record
//! each (`03-design/mcp-server.md`, step 3): what
//! [`TimeArea`](crate::TimeArea) and [`CalendarArea`](crate::CalendarArea)
//! take, spelt as every other request record is, so the agent server's
//! `time.*` and `calendar.convert` read them and nothing else composes
//! the request.
//!
//! A zone is written as [`ZoneSpec`] serialises, which is how a stored
//! chart keeps it: `{"kind": "IANA", "zone": "Asia/Kathmandu"}`,
//! `{"kind": "LOCAL_MEAN", "longitude": 85.324}` or
//! `{"kind": "FIXED", "offset": 20700}`.

use serde::{Deserialize, Serialize};
use teistro_calendar::CalendarDate;
use teistro_calendar::fixed::Weekday;
use teistro_core::catalogue::Calendar;
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::strict;
use teistro_time::civil::{CivilDateTime, CivilTime};
use teistro_time::zone::{ZoneResolution, ZoneSpec};

use crate::Scale;
use crate::asked::DayAsked;

/// A civil date and time in a zone, to resolve to an instant: what
/// [`TimeArea::resolve`](crate::TimeArea::resolve) takes.
///
/// ```
/// use teistro::ResolveRequest;
///
/// let asked = ResolveRequest::from_json(
///     r#"{"date": {"year": 1990, "month": 4, "day": 5}, "time": {"hour": 4, "minute": 30, "second": 0},
///         "zone": {"kind": "IANA", "zone": "Asia/Kathmandu"}}"#,
/// )?;
/// assert!(asked.civil.time.is_some());
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct ResolveRequest {
    /// The civil date and, when known, the time.
    pub civil: CivilDateTime,
    /// The zone it was read in.
    pub zone: ZoneSpec,
}

impl ResolveRequest {
    /// What a JSON resolve request is, for a caller choosing it by name.
    pub const DESCRIPTION: &'static str = "`request` is a civil date and time in a zone: `date` as `{year, month, day}` in `calendar` (a catalogue key, Gregorian when left out), `time` as `{hour, minute, second, nanos}` (left out when the time is not known, and the settings' unknown-time policy supplies one), and `zone` as `{\"kind\": \"IANA\", \"zone\": \"Asia/Kathmandu\"}`, `{\"kind\": \"LOCAL_MEAN\", \"longitude\": 85.324}` or `{\"kind\": \"FIXED\", \"offset\": 20700}`. The answer is the UTC instant as a Julian day, the zone's resolution (the offset, the database's version, the abbreviation, what the daylight-saving policy did and every warning) and the civil time resolved.";

    /// The request read and checked; a refusal names the field written.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, or a value out of range.
    pub fn from_json(text: &str) -> Result<ResolveRequest, Error> {
        let asked: ResolveAsked = strict::read(text, "")?;
        let date = asked
            .date
            .in_calendar(asked.calendar.unwrap_or(Calendar::Gregorian));
        Ok(ResolveRequest {
            civil: CivilDateTime {
                date,
                time: asked.time,
            },
            zone: asked.zone,
        })
    }
}

/// An instant read on a zone's clock, in a calendar: what
/// [`TimeArea::civil_of`](crate::TimeArea::civil_of) takes.
#[derive(Clone, Debug, PartialEq)]
pub struct CivilRequest {
    /// The instant, a UTC Julian day.
    pub instant: JulianDay<Utc>,
    /// The zone whose clock reads it.
    pub zone: ZoneSpec,
    /// The calendar the date is written in.
    pub calendar: Calendar,
}

impl CivilRequest {
    /// What a JSON civil request is, for a caller choosing it by name.
    pub const DESCRIPTION: &'static str = "`request` is an instant to read on a zone's clock: `instant` as a UTC Julian day, `zone` as `time.resolve` writes it, and `calendar` (a catalogue key, Gregorian when left out). The answer is the civil date and time and the zone's resolution.";

    /// The request read and checked; a refusal names the field written.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, or an instant out of range.
    pub fn from_json(text: &str) -> Result<CivilRequest, Error> {
        let asked: CivilAsked = strict::read(text, "")?;
        Ok(CivilRequest {
            instant: JulianDay::try_new(asked.instant)
                .map_err(|why| Error::from(why).with_field("instant"))?,
            zone: asked.zone,
            calendar: asked.calendar.unwrap_or(Calendar::Gregorian),
        })
    }
}

/// An instant to carry from one time scale into another: what
/// [`TimeArea::convert`](crate::TimeArea::convert) takes.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct ScaleRequest {
    /// The instant as a Julian day in `from`.
    pub jd: f64,
    /// The scale it is in.
    pub from: Scale,
    /// The scale wanted.
    pub to: Scale,
}

impl ScaleRequest {
    /// What a JSON scale request is, for a caller choosing it by name.
    pub const DESCRIPTION: &'static str = "`request` is an instant to carry between time scales: `jd`, a Julian day in `from`, and `from` and `to`, each `UT1`, `TT` or `UTC`. The answer is the Julian day in `to` and what was applied: the Delta T with its model, source and uncertainty where one was needed, whether UTC was extended before leap seconds began, and the DUT1.";

    /// The request read and checked; a refusal names the field written.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, or a scale it does not name.
    pub fn from_json(text: &str) -> Result<ScaleRequest, Error> {
        strict::read(text, "")
    }
}

/// A date to write in another calendar: what
/// [`CalendarArea::convert`](crate::CalendarArea::convert) takes.
#[derive(Clone, Debug, PartialEq)]
pub struct CalendarRequest {
    /// The date, in its own calendar.
    pub date: CalendarDate,
    /// The calendar to write it in.
    pub into: Calendar,
}

impl CalendarRequest {
    /// What a JSON calendar request is, for a caller choosing it by name.
    pub const DESCRIPTION: &'static str = "`request` is a date to write in another calendar: `date` as `{year, month, day}` in `calendar` (a catalogue key, Gregorian when left out) and `into`, the calendar wanted. The answer is the date in `into`, with its era and how it was resolved, and its weekday.";

    /// The request read and checked; a refusal names the field written.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, or a calendar it does not name.
    pub fn from_json(text: &str) -> Result<CalendarRequest, Error> {
        let asked: CalendarAsked = strict::read(text, "")?;
        Ok(CalendarRequest {
            date: asked
                .date
                .in_calendar(asked.calendar.unwrap_or(Calendar::Gregorian)),
            into: asked.into,
        })
    }
}

/// An instant read on a zone's clock: what `time.civil` answers.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CivilReading {
    /// The civil date and time.
    pub civil: CivilDateTime,
    /// The zone's resolution at the instant.
    pub zone: ZoneResolution,
}

/// A date written in another calendar: what `calendar.convert` answers.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CalendarReading {
    /// The date in the calendar asked for.
    pub date: CalendarDate,
    /// Its weekday.
    pub weekday: Weekday,
}

/// A resolve request as a binding writes it.
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub(crate) struct ResolveAsked {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    calendar: Option<Calendar>,
    date: DayAsked,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    time: Option<CivilTime>,
    zone: ZoneSpec,
}

/// A civil request as a binding writes it.
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub(crate) struct CivilAsked {
    instant: f64,
    zone: ZoneSpec,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    calendar: Option<Calendar>,
}

/// A calendar request as a binding writes it.
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub(crate) struct CalendarAsked {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    calendar: Option<Calendar>,
    date: DayAsked,
    into: Calendar,
}
