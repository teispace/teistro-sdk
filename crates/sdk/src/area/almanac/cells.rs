//! A range of calendar cells as a request (`03-design/month-cells.md`,
//! SR-84): what a month grid prints of each date, its festivals only when
//! asked, far cheaper than a full day per date.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use teistro_calendar::CalendarDate;
use teistro_core::catalogue::Calendar;
use teistro_core::envelope::{Envelope, content_hash};
use teistro_core::error::Error;
use teistro_core::quantity::Place;
use teistro_core::time::UtcOffset;
use teistro_panchanga::cell::{self, DayCell};
use teistro_panchanga::festival::Observances;
use teistro_time::ZoneSpec;

use super::AlmanacArea;
use crate::DayClock;
use crate::asked::{DayAsked, place_of};
use crate::festival_request::FestivalRequest;

/// The field a cells request's refusals are named under.
const CELLS: &str = "cells";

/// A range of calendar cells, asked.
///
/// ```
/// use teistro::CellsRequest;
/// use teistro::catalogue::Calendar;
///
/// let asked = CellsRequest::from_json(
///     r#"{"first": {"year": 2083, "month": 1, "day": 1}, "last": {"year": 2083, "month": 1, "day": 31},
///         "calendar": "BIKRAM_SAMBAT", "latitudeDeg": 27.7172, "longitudeDeg": 85.324,
///         "zone": {"kind": "IANA", "zone": "Asia/Kathmandu"}}"#,
/// )?;
/// assert_eq!(asked.first.calendar, Calendar::BikramSambat);
/// assert!(asked.festivals.is_none());
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct CellsRequest {
    /// The range's first date.
    pub first: CalendarDate,
    /// Its last date, the first again for one; at most
    /// [`MOST_DAYS`](teistro_panchanga::almanac::MOST_DAYS) dates in all.
    pub last: CalendarDate,
    /// Where the dates are reckoned.
    pub place: Place,
    /// The clock they are reckoned on.
    pub clock: DayClock,
    /// The festivals to list on each cell; none when not asked.
    pub festivals: Option<FestivalRequest>,
}

/// The request as a binding writes it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "CellsRequest"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CellsAsked {
    first: DayAsked,
    /// The last date; the first when left out. At most 366 dates in all.
    #[serde(default)]
    last: Option<DayAsked>,
    /// The calendar the dates are written in; Gregorian when left out.
    #[serde(default)]
    calendar: Option<Calendar>,
    latitude_deg: f64,
    longitude_deg: f64,
    #[serde(default)]
    altitude_m: f64,
    /// The clock's offset; or `zone`, not both.
    #[serde(default)]
    utc_offset_seconds: Option<i32>,
    /// The zone, each date on its own offset; or `utcOffsetSeconds`, not
    /// both.
    #[serde(default)]
    zone: Option<ZoneSpec>,
    /// The festivals to list on each cell, as a festival request; none
    /// when left out.
    #[serde(default)]
    #[cfg_attr(
        feature = "schema",
        schemars(with = "Option<crate::festival_request::Asked>")
    )]
    festivals: Option<Value>,
}

impl CellsRequest {
    /// The record a binding sends, as JSON: `first` and optional `last` as
    /// `{year, month, day}` in `calendar` (Gregorian when left out);
    /// `latitudeDeg`, `longitudeDeg` and `altitudeM`; `utcOffsetSeconds` or
    /// `zone`; and optionally `festivals`, a festival request.
    ///
    /// ```
    /// use teistro::CellsRequest;
    ///
    /// let refused = CellsRequest::from_json(r#"{"first": {"year": 2026, "month": 11, "day": 1}}"#)
    ///     .unwrap_err();
    /// assert_eq!(refused.field(), Some("cells"));
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, a place or offset out of range, both or neither of the
    /// clock's fields, or a festival request its own reader refuses; each
    /// named under `cells`.
    pub fn from_json(text: &str) -> Result<CellsRequest, Error> {
        let asked: CellsAsked = teistro_core::strict::read(text, CELLS)?;
        asked.request().map_err(|why| why.under(CELLS))
    }
}

impl CellsAsked {
    fn request(self) -> Result<CellsRequest, Error> {
        let calendar = self.calendar.unwrap_or(Calendar::Gregorian);
        Ok(CellsRequest {
            first: self.first.in_calendar(calendar),
            last: self.last.unwrap_or(self.first).in_calendar(calendar),
            place: place_of(self.latitude_deg, self.longitude_deg, self.altitude_m)?,
            clock: DayClock::asked(self.utc_offset_seconds, self.zone)?,
            festivals: self
                .festivals
                .map(|festivals| FestivalRequest::from_json(&festivals.to_string()))
                .transpose()?,
        })
    }
}

/// A range's calendar cells.
///
/// ```no_run
/// use teistro::{CellsRequest, Context};
///
/// let sdk = Context::builder().build()?;
/// let asked = CellsRequest::from_json(
///     r#"{"first": {"year": 2026, "month": 11, "day": 1}, "latitudeDeg": 27.7172,
///         "longitudeDeg": 85.324, "utcOffsetSeconds": 20700}"#,
/// )?;
/// let answer = sdk.almanac().cells(&asked)?.value;
/// // One date, and no festivals were asked.
/// assert_eq!(answer.cells.len(), 1);
/// assert!(answer.observances.is_none());
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DayCells {
    /// One cell a date, in order.
    pub cells: Vec<DayCell>,
    /// The observances the cells' `festivals` name, as
    /// [`AlmanacArea::festivals`] answers them over the range; left out
    /// when no festivals were asked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observances: Option<Observances>,
}

/// What a cells answer's input hash seals: the request, its clock as the
/// offset the reckoning ran on (a zone is joined by
/// [`DayClock::seal_answer`]).
#[derive(Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
struct CellsInput<'r> {
    from: String,
    to: String,
    place: Place,
    utc_offset_seconds: i32,
    festivals: Option<&'r FestivalRequest>,
}

impl AlmanacArea<'_> {
    /// **What a month grid prints of every date in a range**: each date's
    /// vara and sunrise, the tithi at the sunrise and its paksha, the
    /// lunar month and its kind, the tithis ending before the next sunrise
    /// and whether one is skipped (kshaya) or repeated (vriddhi), and, when
    /// asked, the festivals kept on it (`03-design/month-cells.md`). Each
    /// sunrise and tithi is the one [`AlmanacArea::of`] gives that date, to
    /// the bit; no Moon rise or set, period or muhurta is computed.
    ///
    /// ```no_run
    /// use teistro::{CellsRequest, Context};
    ///
    /// let sdk = Context::builder().build()?;
    /// let asked = CellsRequest::from_json(
    ///     r#"{"first": {"year": 2026, "month": 11, "day": 1}, "last": {"year": 2026, "month": 11, "day": 30},
    ///         "latitudeDeg": 27.7172, "longitudeDeg": 85.324,
    ///         "zone": {"kind": "IANA", "zone": "Asia/Kathmandu"}}"#,
    /// )?;
    /// for cell in sdk.almanac().cells(&asked)?.value.cells {
    ///     println!("{} {:?} {:?}", cell.date, cell.tithi, cell.kind);
    /// }
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::of`], `OUT_OF_RANGE` on `to` past 366 dates; as
    /// [`AlmanacArea::festivals`] for the festivals; a zone the database
    /// does not have.
    pub fn cells(self, asked: &CellsRequest) -> Result<Envelope<DayCells>, Error> {
        self.unclocked().clocked_sealed(
            &asked.clock,
            || Self::noon_of_date(&asked.first),
            |almanac, offset| almanac.cells_on(asked, offset),
        )
    }

    /// [`AlmanacArea::cells`] on `offset`, or on the zone this area is on.
    fn cells_on(
        self,
        asked: &CellsRequest,
        offset: UtcOffset,
    ) -> Result<Envelope<DayCells>, Error> {
        let (first, last) = (&asked.first, &asked.last);
        let Envelope {
            value: days,
            mut provenance,
        } = self.with_almanac(first.calendar, offset, |almanac| {
            almanac.cell_days(first, last, &asked.place)
        })?;
        let mut cells = cell::cells(&days)?;
        let observances = match &asked.festivals {
            Some(request) => {
                request.check()?;
                // The rules read more of a day than a cell does, so the
                // range's festival days are founded as
                // `almanac.festivals` founds them, and reckoned the same.
                let festival_days = self.with_almanac(first.calendar, offset, |almanac| {
                    almanac.festival_days(first, last, &asked.place)
                })?;
                let found =
                    self.reckoning(first, last, &asked.place, offset, request, festival_days)?;
                cell::with_festivals(&mut cells, &found.value);
                // The festivals' conventions, the widened days among them.
                provenance = found.provenance;
                Some(found.value)
            }
            None => None,
        };
        provenance.input_hash = content_hash(&CellsInput {
            from: first.to_string(),
            to: last.to_string(),
            place: asked.place,
            utc_offset_seconds: offset.seconds(),
            festivals: asked.festivals.as_ref(),
        });
        Ok(Envelope::sealing(
            DayCells { cells, observances },
            provenance,
        ))
    }
}
