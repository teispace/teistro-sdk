//! The Sun for solar calendars: a model that answers the Sun's sidereal
//! longitude at an instant and the day's arc at a place, the sankranti
//! finder over the shared solver, and the month-start rules of the
//! traditions (`docs/03-design/calendar-bikram-sambat.md`, §4).
//!
//! A solar calendar is a set of sankrantis (the Sun entering a sign)
//! placed on civil days by a rule under a clock; everything above the
//! model is arithmetic on instants, so the same engine serves any solar
//! calendar and any model: the Surya Siddhanta (`siddhanta`) or modern
//! astronomy through the ephemeris port ([`DrikSun`]).

pub mod drik;
pub mod rule;
pub mod sankranti;
mod siddhanta;

use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Place, Ut1, Utc};
use teistro_core::settings::SunriseConvention;
use teistro_core::time::{LocalClock, LocalMeanTime};

use crate::fixed::FixedDay;

pub use drik::DrikSun;
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

    /// The sunrise convention the model reckons its arc by: the text's
    /// centre on the geometric horizon for the classical model, the
    /// profile's for the drik model.
    fn convention(&self) -> SunriseConvention;

    /// Whether the day's sunrise is a classical astronomy's own
    /// **definition** rather than the sky's: the text's model itself, or
    /// a model over a provider that defines it
    /// (`03-design/classical-chart.md` §4). A chart stamps who gave its
    /// day from this.
    fn defines_sunrise(&self) -> bool {
        false
    }

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

/// The day a solar model reckons a civil date by.
///
/// A model counts days in the local mean time of the place's longitude, which
/// is the civil date wherever the clock keeps within half a day of it — and
/// is not where a clock was moved across the date line: Samoa's UTC+14 at
/// 171.8° W is 25½ hours from its mean time, so its civil 31 December 2011
/// is 30 December by the Sun. The mean-time day in force at the civil day's
/// own noon is the one whose sunrise falls in that civil day, and it is the
/// civil date itself everywhere else.
#[must_use]
pub fn mean_time_day(clock: &dyn LocalClock, place: &Place, civil: FixedDay) -> FixedDay {
    let Ok(midnight) = civil.jd_at_midnight() else {
        return civil;
    };
    let noon = midnight.get() + 0.5 - clock.offset_at(midnight).days();
    FixedDay::from_local_jd(noon + LocalMeanTime::new(place.longitude).offset().days()).0
}

/// The instant a mean-time day begins at a place: its midnight in the
/// local mean time of the longitude, **exactly**.
///
/// Not through [`LocalMeanTime`]'s offset, which a `UtcOffset` holds in
/// whole seconds: a classical text reckons its day from the exact mean
/// midnight (`SuryaSiddhanta::local_mean_midnight`), and a midnight
/// rounded to the second put the text's own sunrise, counted by the
/// solar model, up to half a second from the same sunrise counted by
/// its provider (`03-design/classical-chart-measured.md`).
///
/// # Errors
///
/// A day outside the Julian day range.
pub fn local_mean_midnight(day: FixedDay, place: &Place) -> Result<JulianDay<Ut1>, Error> {
    Ok(JulianDay::try_new(
        day.jd_at_midnight()?.get() - place.longitude.get() / 360.0,
    )?)
}

/// Sunrise and sunset of a civil date under a clock, however far the clock
/// keeps from the place's mean time ([`mean_time_day`]).
///
/// # Errors
///
/// As [`SolarModel::day_light`].
pub fn civil_day_light(
    model: &dyn SolarModel,
    clock: &dyn LocalClock,
    place: &Place,
    civil: FixedDay,
) -> Result<DayLight, Error> {
    model.day_light(mean_time_day(clock, place, civil), place)
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

    fn convention(&self) -> SunriseConvention {
        (**self).convention()
    }

    fn defines_sunrise(&self) -> bool {
        (**self).defines_sunrise()
    }
}
