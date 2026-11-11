//! The parts every request record a binding writes spells the same way: a
//! civil day as `{year, month, day}`, an instant as a UTC Julian day, a place as `latitudeDeg`,
//! `longitudeDeg` and `altitudeM`, and an offset in seconds, each refused
//! by its own field.

use serde::{Deserialize, Serialize};
use teistro_calendar::CalendarDate;
use teistro_core::catalogue::Calendar;
use teistro_core::envelope::Convention;
use teistro_core::error::Error;
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro_core::time::UtcOffset;
use teistro_port_geo::ResolvedPlace;
use teistro_time::ZoneSpec;

use crate::DayClock;

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

/// Where a request stands and on which clock, as it wrote them out or
/// as a resolved `place` gives them (`03-design/geo-port.md` §6), with
/// where the observer's height came from when a place was resolved.
pub(crate) struct Placed {
    /// The place, its height the observer's.
    pub(crate) place: Place,
    /// The clock.
    pub(crate) clock: DayClock,
    /// Where the height came from, for the provenance: `None` for a
    /// place written out, whose height is the request's own.
    pub(crate) height: Option<Convention>,
}

/// The parts of a request that say where and on which clock, each as the
/// request read it.
pub(crate) struct Where<'a> {
    pub(crate) latitude_deg: Option<f64>,
    pub(crate) longitude_deg: Option<f64>,
    pub(crate) altitude_m: Option<f64>,
    pub(crate) utc_offset_seconds: Option<i32>,
    pub(crate) zone: Option<ZoneSpec>,
    pub(crate) place: Option<&'a ResolvedPlace>,
}

impl Where<'_> {
    /// The place and clock: the coordinates written out, or a resolved
    /// `place` with nothing beside it but an `altitudeM` that wins over
    /// its height and an offset where it has no zone.
    pub(crate) fn placed(self) -> Result<Placed, Error> {
        let Some(place) = self.place else {
            let (Some(latitude), Some(longitude)) = (self.latitude_deg, self.longitude_deg) else {
                let field = if self.latitude_deg.is_none() {
                    "latitudeDeg"
                } else {
                    "longitudeDeg"
                };
                return Err(Error::invalid_arg("a request names where it is reckoned")
                    .with_field(field)
                    .with_hint(
                        "write `latitudeDeg` and `longitudeDeg`, or a `place` as `geo.resolve` answers it",
                    ));
            };
            return Ok(Placed {
                place: place_of(latitude, longitude, self.altitude_m.unwrap_or_default())?,
                clock: DayClock::asked(self.utc_offset_seconds, self.zone)?,
                height: None,
            });
        };
        for (field, given) in [
            ("latitudeDeg", self.latitude_deg.is_some()),
            ("longitudeDeg", self.longitude_deg.is_some()),
            ("zone", self.zone.is_some()),
        ] {
            if given {
                return Err(Error::invalid_arg(format!(
                    "`{field}` is the resolved place's, so it is not written beside `place`"
                ))
                .with_field(field)
                .with_hint("leave it out, or write the place out without `place`"));
            }
        }
        resolved(place, self.altitude_m, self.utc_offset_seconds)
    }
}

/// A resolved place's coordinates, its clock (its zone's, or the offset
/// where it has none), and the observer's height: `altitude_m` when the
/// caller states one, which wins, else the provider's ground height, else
/// sea level, each recorded as where the height came from.
pub(crate) fn resolved(
    place: &ResolvedPlace,
    altitude_m: Option<f64>,
    offset_seconds: Option<i32>,
) -> Result<Placed, Error> {
    let point = place
        .checked(teistro_core::error::Status::InvalidArg)
        .map_err(|why| why.under("place"))?;
    let clock = match (&place.zone, offset_seconds) {
        (Some(zone), None) => DayClock::Zone(ZoneSpec::iana(zone.clone())),
        (None, Some(seconds)) => DayClock::Offset(offset_of(seconds)?),
        (Some(_), Some(_)) => {
            return Err(Error::invalid_arg(
                "the resolved place has a zone, so the request states no offset",
            )
            .with_field("utcOffsetSeconds")
            .with_hint("leave the offset out, or write the place out without `place`"));
        }
        (None, None) => {
            return Err(Error::invalid_arg(
                "the resolved place has no zone, so the request states its clock",
            )
            .with_field("place.zone")
            .with_hint("write `utcOffsetSeconds` beside the place"));
        }
    };
    let ground = place.height.as_ref().map(|height| height.metres);
    let metres = altitude_m.or(ground).unwrap_or_default();
    let field = if altitude_m.is_some() {
        "altitudeM"
    } else {
        "place.height.metres"
    };
    let altitude = Altitude::try_new(metres).map_err(|why| Error::from(why).with_field(field))?;
    Ok(Placed {
        place: Place::new(point.latitude, point.longitude, altitude),
        clock,
        height: Some(height_from(place, altitude_m)),
    })
}

/// Where the observer's height came from: the caller's `altitudeM`, the
/// provider's ground height, or sea level where it knows none
/// (`03-design/geo-port.md` §6a).
fn height_from(place: &ResolvedPlace, altitude_m: Option<f64>) -> Convention {
    let source = &place.source;
    let provider = format!(
        "the geo provider {} {} (data {})",
        source.name, source.version, source.data_version
    );
    let (metres, reason) = match (altitude_m, &place.height) {
        (Some(stated), Some(ground)) => (
            stated,
            format!(
                "stated by the caller as `altitudeM`, over {provider}'s ground height of {} m",
                ground.metres
            ),
        ),
        (Some(stated), None) => (
            stated,
            format!("stated by the caller as `altitudeM`; {provider} knows no height here"),
        ),
        (None, Some(ground)) => (
            ground.metres,
            format!(
                "{provider}: {}, at {} m resolution, above {}",
                ground.source, ground.resolution_m, ground.datum
            ),
        ),
        (None, None) => (
            0.0,
            format!("{provider} knows no height here, so sea level"),
        ),
    };
    Convention {
        knob: String::from(HEIGHT_KNOB),
        value: format!("{metres} m"),
        reason,
    }
}

/// The knob a resolved place's height is recorded under.
pub(crate) const HEIGHT_KNOB: &str = "observer.height";

/// An offset from its seconds, refused as `utcOffsetSeconds`.
pub(crate) fn offset_of(seconds: i32) -> Result<UtcOffset, Error> {
    UtcOffset::try_from_seconds(seconds)
        .map_err(|why| Error::from(why).with_field("utcOffsetSeconds"))
}

/// A UTC Julian day a request names, refused by its field.
pub(crate) fn instant(jd: f64, field: &str) -> Result<JulianDay<Utc>, Error> {
    JulianDay::<Utc>::try_new(jd).map_err(|why| Error::from(why).with_field(field))
}
