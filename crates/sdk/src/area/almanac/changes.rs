//! A range's change points as a request (`03-design/change-points.md`,
//! SR-07), and the clock a range or a chart is asked on: a bare offset, or
//! a zone (`03-design/time-and-timezone.md` §3.4).

use serde::{Deserialize, Serialize};
use teistro_calendar::CalendarDate;
use teistro_core::catalogue::Calendar;
use teistro_core::envelope::{Envelope, Provenance, content_hash};
use teistro_core::error::Error;
use teistro_core::quantity::Place;
use teistro_core::time::UtcOffset;
use teistro_panchanga::change::{ChangeKind, ChangePoints};
use teistro_time::ZoneSpec;

use super::{AlmanacAnswer, AlmanacArea, AlmanacRequest};
use crate::asked::{DayAsked, offset_of, place_of};

/// The field a change-points request's refusals are named under.
const CHANGES: &str = "changes";

/// The clock a request is asked on: a range of days, or a chart's births.
///
/// ```
/// use teistro::{DayClock, UtcOffset, ZoneSpec};
///
/// let refused = DayClock::asked(Some(20_700), Some(ZoneSpec::iana("Asia/Kathmandu"))).unwrap_err();
/// assert_eq!(refused.field(), Some("zone"));
/// assert!(DayClock::asked(None, None).is_err());
/// // An offset is the clock it states.
/// assert_eq!(DayClock::from(UtcOffset::UTC), DayClock::Offset(UtcOffset::UTC));
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum DayClock {
    /// An offset the caller stated.
    Offset(UtcOffset),
    /// A zone, each day or birth on its own offset.
    Zone(ZoneSpec),
}

impl From<UtcOffset> for DayClock {
    fn from(offset: UtcOffset) -> DayClock {
        DayClock::Offset(offset)
    }
}

impl DayClock {
    /// The clock a request's two fields name: one of them.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on `zone` for both, on `utcOffsetSeconds` for neither
    /// or an offset out of range.
    pub fn asked(seconds: Option<i32>, zone: Option<ZoneSpec>) -> Result<DayClock, Error> {
        match (seconds, zone) {
            (Some(seconds), None) => Ok(DayClock::Offset(offset_of(seconds)?)),
            (None, Some(zone)) => Ok(DayClock::Zone(zone)),
            (Some(_), Some(_)) => Err(Error::invalid_arg(
                "a request is on `utcOffsetSeconds` or a `zone`, not both",
            )
            .with_field("zone")
            .with_hint("write the zone and leave the offset out: each instant is read on its own")),
            (None, None) => Err(Error::invalid_arg(
                "a request names its clock: `utcOffsetSeconds` or a `zone`",
            )
            .with_field("utcOffsetSeconds")),
        }
    }

    /// The zone, when this clock is one: what an input hash seals beside
    /// the offset it read.
    ///
    /// ```
    /// use teistro::{DayClock, UtcOffset, ZoneSpec};
    ///
    /// assert_eq!(DayClock::from(UtcOffset::UTC).zone(), None);
    /// let nepal = ZoneSpec::iana("Asia/Kathmandu");
    /// assert_eq!(DayClock::Zone(nepal.clone()).zone(), Some(&nepal));
    /// ```
    #[must_use]
    pub const fn zone(&self) -> Option<&ZoneSpec> {
        match self {
            DayClock::Offset(_) => None,
            DayClock::Zone(zone) => Some(zone),
        }
    }

    /// The offset, when this clock is one stated: what a reckoning that
    /// takes a bare offset is handed.
    ///
    /// ```
    /// use teistro::{DayClock, UtcOffset, ZoneSpec};
    ///
    /// assert_eq!(DayClock::from(UtcOffset::UTC).offset(), Some(UtcOffset::UTC));
    /// assert_eq!(DayClock::Zone(ZoneSpec::iana("Asia/Kathmandu")).offset(), None);
    /// ```
    #[must_use]
    pub const fn offset(&self) -> Option<UtcOffset> {
        match self {
            DayClock::Offset(offset) => Some(*offset),
            DayClock::Zone(_) => None,
        }
    }

    /// Writes this clock into a request's input record as the request
    /// asked it: `utcOffsetSeconds` for an offset stated, the record then
    /// byte for byte what it was, or `zone` (C1252).
    pub(crate) fn seal_into(&self, input: &mut serde_json::Value) {
        if let Some(record) = input.as_object_mut() {
            match self {
                DayClock::Offset(offset) => {
                    record.insert(String::from("utcOffsetSeconds"), offset.seconds().into());
                }
                DayClock::Zone(zone) => {
                    record.insert(String::from("zone"), serde_json::json!(zone));
                }
            }
        }
    }

    /// Joins a zone to an answer whose input hash a reckoning wrote on the
    /// one offset it was handed, so a request by zone and one by that
    /// offset seal apart; an offset leaves the hash as it was (C1252).
    pub(crate) fn seal_answer(&self, provenance: &mut Provenance) {
        if let DayClock::Zone(zone) = self {
            provenance.input_hash = content_hash(&serde_json::json!({
                "inputHash": provenance.input_hash.to_string(),
                "zone": zone,
            }));
        }
    }

    /// The range on this clock, and what is asked beside it.
    pub(crate) fn asked_on(
        &self,
        almanac: AlmanacArea<'_>,
        (first, last): (&CalendarDate, &CalendarDate),
        place: &Place,
        beside: &AlmanacRequest,
    ) -> Result<AlmanacAnswer, Error> {
        match self {
            DayClock::Offset(offset) => almanac.asked(first, last, place, *offset, beside),
            DayClock::Zone(zone) => almanac.in_zone(first, last, place, zone, beside),
        }
    }
}

/// A range's change points, asked.
///
/// ```
/// use teistro::{ChangeKind, ChangesRequest};
///
/// let asked = ChangesRequest::from_json(
///     r#"{"first": {"year": 2026, "month": 11, "day": 1}, "latitudeDeg": 27.7172,
///         "longitudeDeg": 85.324, "utcOffsetSeconds": 20700, "kinds": ["TITHI"]}"#,
/// )?;
/// assert_eq!(asked.first, asked.last);
/// assert_eq!(asked.kinds, [ChangeKind::Tithi]);
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct ChangesRequest {
    /// The range's first day.
    pub first: CalendarDate,
    /// Its last day, the first again for one day.
    pub last: CalendarDate,
    /// Where the days are reckoned.
    pub place: Place,
    /// The clock they are reckoned on.
    pub clock: DayClock,
    /// The kinds wanted; every kind when empty.
    pub kinds: Vec<ChangeKind>,
}

/// The request as a binding writes it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "ChangesRequest"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ChangesAsked {
    first: DayAsked,
    /// The last day; the first when left out.
    #[serde(default)]
    last: Option<DayAsked>,
    /// The calendar the days are written in; Gregorian when left out.
    #[serde(default)]
    calendar: Option<Calendar>,
    latitude_deg: f64,
    longitude_deg: f64,
    #[serde(default)]
    altitude_m: f64,
    /// The clock's offset; or `zone`, not both.
    #[serde(default)]
    utc_offset_seconds: Option<i32>,
    /// The zone; or `utcOffsetSeconds`, not both.
    #[serde(default)]
    zone: Option<ZoneSpec>,
    /// The kinds wanted; every kind when left out.
    #[serde(default)]
    kinds: Vec<ChangeKind>,
}

impl ChangesRequest {
    /// The record a binding sends, as JSON: `first` and optional `last` as
    /// `{year, month, day}` in `calendar` (Gregorian when left out);
    /// `latitudeDeg`, `longitudeDeg` and `altitudeM`; `utcOffsetSeconds` or
    /// `zone`; and `kinds`, every kind when left out.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, a place or offset out of range, or both or neither of the
    /// clock's fields; each named under `changes`.
    pub fn from_json(text: &str) -> Result<ChangesRequest, Error> {
        let asked: ChangesAsked = teistro_core::strict::read(text, CHANGES)?;
        asked.request().map_err(|why| why.under(CHANGES))
    }
}

impl ChangesAsked {
    fn request(self) -> Result<ChangesRequest, Error> {
        let calendar = self.calendar.unwrap_or(Calendar::Gregorian);
        Ok(ChangesRequest {
            first: self.first.in_calendar(calendar),
            last: self.last.unwrap_or(self.first).in_calendar(calendar),
            place: place_of(self.latitude_deg, self.longitude_deg, self.altitude_m)?,
            clock: DayClock::asked(self.utc_offset_seconds, self.zone)?,
            kinds: self.kinds,
        })
    }
}

impl AlmanacArea<'_> {
    /// **Every instant in a range at which something changes**: the tithi,
    /// nakshatra, yoga, karana, the Moon's or Sun's sign or the vara, one
    /// list sorted by instant, each saying what ended and what began
    /// (`03-design/change-points.md`). Reads the spans a range finds, and
    /// asks for no rise or set.
    ///
    /// ```no_run
    /// use teistro::{ChangesRequest, Context};
    ///
    /// let sdk = Context::builder().build()?;
    /// let asked = ChangesRequest::from_json(
    ///     r#"{"first": {"year": 2026, "month": 11, "day": 1}, "latitudeDeg": 27.7172,
    ///         "longitudeDeg": 85.324, "zone": {"kind": "IANA", "zone": "Asia/Kathmandu"}}"#,
    /// )?;
    /// for point in sdk.almanac().changes(&asked)?.value.points {
    ///     println!("{} {:?}: {} to {}", point.at, point.kind, point.from, point.to);
    /// }
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::of`]; a zone the database does not have.
    pub fn changes(self, asked: &ChangesRequest) -> Result<Envelope<ChangePoints>, Error> {
        let found = |area: AlmanacArea<'_>, offset: UtcOffset| {
            area.with_almanac(asked.first.calendar, offset, |almanac| {
                almanac.changes(&asked.first, &asked.last, &asked.place)
            })
        };
        let found = match &asked.clock {
            DayClock::Offset(offset) => found(self, *offset)?,
            DayClock::Zone(zone) => {
                let clock = zone.clock(self.context.zones())?;
                found(self.on(&clock), UtcOffset::UTC)?
            }
        };
        let Envelope { value, provenance } = found;
        let points = value
            .points
            .into_iter()
            .filter(|point| asked.kinds.is_empty() || asked.kinds.contains(&point.kind))
            .collect();
        Ok(Envelope::sealing(
            ChangePoints {
                points,
                window: value.window,
            },
            provenance,
        ))
    }
}
