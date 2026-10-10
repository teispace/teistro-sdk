//! The parts every request record a binding writes spells the same way: a
//! civil day as `{year, month, day}`, a place as `latitudeDeg`,
//! `longitudeDeg` and `altitudeM`, and an offset in seconds, each refused
//! by its own field.

use serde::{Deserialize, Serialize};
use teistro_calendar::CalendarDate;
use teistro_core::catalogue::Calendar;
use teistro_core::error::Error;
use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
use teistro_core::time::UtcOffset;

/// A civil day as a request names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub(crate) struct DayAsked {
    pub(crate) year: i32,
    pub(crate) month: u8,
    pub(crate) day: u8,
}

impl DayAsked {
    /// The date in `calendar`.
    pub(crate) fn in_calendar(self, calendar: Calendar) -> CalendarDate {
        CalendarDate::defined(calendar, self.year, self.month, self.day)
    }
}

/// A place from its three numbers, each refused by its own field.
pub(crate) fn place_of(
    latitude_deg: f64,
    longitude_deg: f64,
    altitude_m: f64,
) -> Result<Place, Error> {
    Ok(Place::new(
        Latitude::try_new(latitude_deg)
            .map_err(|why| Error::from(why).with_field("latitudeDeg"))?,
        Longitude::try_new(longitude_deg)
            .map_err(|why| Error::from(why).with_field("longitudeDeg"))?,
        Altitude::try_new(altitude_m).map_err(|why| Error::from(why).with_field("altitudeM"))?,
    ))
}

/// An offset from its seconds, refused as `utcOffsetSeconds`.
pub(crate) fn offset_of(seconds: i32) -> Result<UtcOffset, Error> {
    UtcOffset::try_from_seconds(seconds)
        .map_err(|why| Error::from(why).with_field("utcOffsetSeconds"))
}
