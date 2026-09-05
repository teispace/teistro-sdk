//! The Sun for solar calendars: a model that answers the Sun's sidereal
//! longitude at an instant and the day's arc at a place, the sankranti
//! finder over the shared solver, and the month-start rules of the
//! traditions (`docs/03-design/calendar-bikram-sambat.md`, §4).
//!
//! A solar calendar is a set of sankrantis (the Sun entering a sign)
//! placed on civil days by a rule under a clock; everything above the
//! model is arithmetic on instants, so the same engine serves any solar
//! calendar and any model, classical or modern.

pub mod rule;
pub mod sankranti;
mod siddhanta;

use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Place, Utc};

use crate::fixed::FixedDay;

pub use rule::MonthStartRule;
pub use sankranti::{MEAN_SOLAR_RATE_DEG_PER_DAY, Sankranti, TOLERANCE_DAYS, find_sankranti};

/// A day's sunrise and sunset at a place, as UTC instants.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DayArc {
    /// Sunrise.
    pub sunrise: JulianDay<Utc>,
    /// Sunset.
    pub sunset: JulianDay<Utc>,
}

impl DayArc {
    /// The instant a given fraction of the daylight has elapsed.
    #[must_use]
    pub fn at_fraction(&self, fraction: f64) -> f64 {
        self.sunrise.get() + (self.sunset.get() - self.sunrise.get()) * fraction
    }
}

/// What the Sun does on a civil day at a place.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DayLight {
    /// It rises and sets.
    Arc(DayArc),
    /// It never sets (polar day).
    AlwaysUp,
    /// It never rises (polar night).
    NeverUp,
}

impl DayLight {
    /// The arc, when there is one.
    #[must_use]
    pub const fn arc(self) -> Option<DayArc> {
        match self {
            DayLight::Arc(arc) => Some(arc),
            DayLight::AlwaysUp | DayLight::NeverUp => None,
        }
    }
}

/// A model of the Sun a solar calendar can be computed from.
pub trait SolarModel: Send + Sync {
    /// The Sun's sidereal longitude in degrees at a Universal Time Julian
    /// day, in the model's own sidereal frame (the text's for the Surya
    /// Siddhanta, the profile's ayanamsha for a modern ephemeris).
    ///
    /// # Errors
    ///
    /// An instant the model cannot answer for (a modern ephemeris outside
    /// its data); the classical model never fails.
    fn sidereal_sun_deg(&self, jd_ut: f64) -> Result<f64, Error>;

    /// What the Sun does on a civil day at a place: its sunrise and
    /// sunset, or which polar state the day is in.
    ///
    /// # Errors
    ///
    /// As [`SolarModel::sidereal_sun_deg`].
    fn day_light(&self, day: FixedDay, place: &Place) -> Result<DayLight, Error>;

    /// The model's name for provenance stamps.
    fn describe(&self) -> String;

    /// Sunrise and sunset of a civil day at a place, or `None` where the
    /// Sun neither rises nor sets that day.
    ///
    /// # Errors
    ///
    /// As [`SolarModel::day_light`].
    fn day_arc(&self, day: FixedDay, place: &Place) -> Result<Option<DayArc>, Error> {
        Ok(self.day_light(day, place)?.arc())
    }
}

impl<M: SolarModel + ?Sized> SolarModel for &M {
    fn sidereal_sun_deg(&self, jd_ut: f64) -> Result<f64, Error> {
        (**self).sidereal_sun_deg(jd_ut)
    }

    fn day_light(&self, day: FixedDay, place: &Place) -> Result<DayLight, Error> {
        (**self).day_light(day, place)
    }

    fn describe(&self) -> String {
        (**self).describe()
    }
}
