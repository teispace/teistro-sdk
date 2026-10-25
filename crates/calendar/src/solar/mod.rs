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
///
/// ```
/// # use teistro_calendar::bikram_sambat::engine::KATHMANDU;
/// # use teistro_core::time::UtcOffset;
/// # use teistro_siddhanta::SuryaSiddhanta;
/// # let (text, clock) = (SuryaSiddhanta::text(), UtcOffset::literal(5, 45, 0));
/// use teistro_calendar::gregorian::fixed_from_gregorian;
/// use teistro_calendar::solar::civil_day_light;
///
/// let arc = civil_day_light(&text, &clock, &KATHMANDU, fixed_from_gregorian(2024, 1, 1))?.arc().unwrap();
/// // A winter day at Kathmandu: about ten and a quarter hours of light.
/// let hours = (arc.sunset.get() - arc.sunrise.get()) * 24.0;
/// assert!((10.0..10.5).contains(&hours), "{hours}");
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DayArc {
    /// Sunrise.
    pub sunrise: JulianDay<Utc>,
    /// Sunset.
    pub sunset: JulianDay<Utc>,
}

impl DayArc {
    /// The instant a given fraction of the daylight has elapsed.
    ///
    /// ```
    /// # use teistro_calendar::bikram_sambat::engine::KATHMANDU;
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_siddhanta::SuryaSiddhanta;
    /// # let (text, clock) = (SuryaSiddhanta::text(), UtcOffset::literal(5, 45, 0));
    /// use teistro_calendar::gregorian::fixed_from_gregorian;
    /// use teistro_calendar::solar::civil_day_light;
    ///
    /// let arc = civil_day_light(&text, &clock, &KATHMANDU, fixed_from_gregorian(2024, 1, 1))?.arc().unwrap();
    /// assert_eq!(arc.at_fraction(0.0), arc.sunrise.get());
    /// assert!((arc.at_fraction(1.0) - arc.sunset.get()).abs() < 1e-9);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn at_fraction(&self, fraction: f64) -> f64 {
        self.sunrise.get() + (self.sunset.get() - self.sunrise.get()) * fraction
    }
}

/// What the Sun does on a civil day at a place.
///
/// ```
/// use teistro_calendar::gregorian::fixed_from_gregorian;
/// use teistro_siddhanta::SuryaSiddhanta;
/// use teistro_calendar::solar::{DayLight, SolarModel};
/// use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
///
/// // Tromsø, 69.6° N: no sunrise at midwinter, no sunset at midsummer.
/// let tromso = Place::new(Latitude::literal(69.6492), Longitude::literal(18.9553), Altitude::literal(0.0));
/// let text = SuryaSiddhanta::text();
/// assert_eq!(text.day_light(fixed_from_gregorian(2024, 12, 21), &tromso)?, DayLight::NeverUp);
/// assert_eq!(text.day_light(fixed_from_gregorian(2024, 6, 21), &tromso)?, DayLight::AlwaysUp);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
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
    ///
    /// ```
    /// use teistro_calendar::solar::DayLight;
    ///
    /// assert_eq!(DayLight::NeverUp.arc(), None);
    /// ```
    #[must_use]
    pub const fn arc(self) -> Option<DayArc> {
        match self {
            DayLight::Arc(arc) => Some(arc),
            DayLight::AlwaysUp | DayLight::NeverUp => None,
        }
    }
}

/// A model of the Sun a solar calendar can be computed from.
///
/// ```
/// use teistro_siddhanta::SuryaSiddhanta;
/// use teistro_calendar::solar::SolarModel;
///
/// // The text's own Sun: its sidereal longitude, in its own zodiac.
/// let text = SuryaSiddhanta::text();
/// let longitude = text.sidereal_sun_deg(2_460_310.5)?;
/// // On 1 January the Sun is in Dhanu, the ninth sign (240° to 270°).
/// assert!((240.0..270.0).contains(&longitude), "{longitude}");
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
pub trait SolarModel: Send + Sync {
    /// The Sun's sidereal longitude in degrees at a Universal Time Julian
    /// day, in the model's own sidereal frame (the text's for the Surya
    /// Siddhanta, the profile's ayanamsha for a modern ephemeris).
    ///
    /// ```
    /// use teistro_siddhanta::SuryaSiddhanta;
    /// use teistro_calendar::solar::SolarModel;
    ///
    /// let text = SuryaSiddhanta::text();
    /// // About a degree a day.
    /// let step = text.sidereal_sun_deg(2_460_311.5)? - text.sidereal_sun_deg(2_460_310.5)?;
    /// assert!((0.95..1.05).contains(&step), "{step}");
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// An instant the model cannot answer for (a modern ephemeris outside
    /// its data); the classical model never fails.
    fn sidereal_sun_deg(&self, jd_ut: f64) -> Result<f64, Error>;

    /// What the Sun does on a civil day at a place: its sunrise and
    /// sunset, or which polar state the day is in.
    ///
    /// ```
    /// # use teistro_calendar::bikram_sambat::engine::KATHMANDU;
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_siddhanta::SuryaSiddhanta;
    /// # let (text, clock) = (SuryaSiddhanta::text(), UtcOffset::literal(5, 45, 0));
    /// use teistro_calendar::gregorian::fixed_from_gregorian;
    /// use teistro_calendar::solar::SolarModel;
    ///
    /// let light = text.day_light(fixed_from_gregorian(2024, 1, 1), &KATHMANDU)?;
    /// assert!(light.arc().is_some());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`SolarModel::sidereal_sun_deg`].
    fn day_light(&self, day: FixedDay, place: &Place) -> Result<DayLight, Error>;

    /// [`SolarModel::day_light`] for many days at one place, each day's
    /// answer its own: what [`SolarModel::day_light`] answers for it, to
    /// the bit. A model that can share the work between days (the drik
    /// model searches them in lockstep, one request a round) does; the
    /// default asks day by day.
    ///
    /// ```
    /// # use teistro_calendar::bikram_sambat::engine::KATHMANDU;
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_siddhanta::SuryaSiddhanta;
    /// # let (text, clock) = (SuryaSiddhanta::text(), UtcOffset::literal(5, 45, 0));
    /// use teistro_calendar::gregorian::fixed_from_gregorian;
    /// use teistro_calendar::solar::SolarModel;
    ///
    /// let days = [fixed_from_gregorian(2024, 1, 1), fixed_from_gregorian(2024, 1, 2)];
    /// let each = text.day_lights(&days, &KATHMANDU);
    /// assert_eq!(each[1].as_ref().ok(), text.day_light(days[1], &KATHMANDU).as_ref().ok());
    /// ```
    fn day_lights(&self, days: &[FixedDay], place: &Place) -> Vec<Result<DayLight, Error>> {
        days.iter().map(|day| self.day_light(*day, place)).collect()
    }

    /// The model's name for provenance stamps.
    ///
    /// ```
    /// use teistro_siddhanta::SuryaSiddhanta;
    /// use teistro_calendar::solar::SolarModel;
    ///
    /// assert_eq!(SuryaSiddhanta::text().describe(), "Surya Siddhanta (the text, the sine table)");
    /// ```
    fn describe(&self) -> String;

    /// The sunrise convention the model reckons its arc by: the text's
    /// centre on the geometric horizon for the classical model, the
    /// profile's for the drik model.
    ///
    /// ```
    /// use teistro_siddhanta::SuryaSiddhanta;
    /// use teistro_calendar::solar::SolarModel;
    /// use teistro_core::settings::{Sunrise, SunriseConvention};
    ///
    /// // The text's sunrise: the Sun's centre on the horizon, no refraction.
    /// assert_eq!(SuryaSiddhanta::text().convention(), SunriseConvention::Named { which: Sunrise::CentreNoRefraction });
    /// ```
    fn convention(&self) -> SunriseConvention;

    /// Whether the day's sunrise is a classical astronomy's own
    /// **definition** rather than the sky's: the text's model itself, or
    /// a model over a provider that defines it
    /// (`03-design/classical-chart.md` §4). A chart stamps who gave its
    /// day from this.
    ///
    /// ```
    /// use teistro_siddhanta::SuryaSiddhanta;
    /// use teistro_calendar::solar::SolarModel;
    ///
    /// assert!(SuryaSiddhanta::text().defines_sunrise());
    /// ```
    fn defines_sunrise(&self) -> bool {
        false
    }

    /// Sunrise and sunset of a civil day at a place, or `None` where the
    /// Sun neither rises nor sets that day.
    ///
    /// ```
    /// # use teistro_calendar::bikram_sambat::engine::KATHMANDU;
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_siddhanta::SuryaSiddhanta;
    /// # let (text, clock) = (SuryaSiddhanta::text(), UtcOffset::literal(5, 45, 0));
    /// use teistro_calendar::gregorian::fixed_from_gregorian;
    /// use teistro_calendar::solar::SolarModel;
    ///
    /// let day = fixed_from_gregorian(2024, 1, 1);
    /// // The text has a `day_arc` of its own, so the trait's is named.
    /// let arc = SolarModel::day_arc(&text, day, &KATHMANDU)?;
    /// assert_eq!(arc, SolarModel::day_light(&text, day, &KATHMANDU)?.arc());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
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
///
/// ```
/// use teistro_calendar::gregorian::fixed_from_gregorian;
/// use teistro_calendar::bikram_sambat::engine::KATHMANDU;
/// use teistro_calendar::solar::mean_time_day;
/// use teistro_core::time::UtcOffset;
///
/// // Nepal's clock keeps within a few minutes of Kathmandu's mean time.
/// let day = fixed_from_gregorian(2024, 1, 1);
/// assert_eq!(mean_time_day(&UtcOffset::literal(5, 45, 0), &KATHMANDU, day), day);
/// ```
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
/// ```
/// use teistro_calendar::gregorian::fixed_from_gregorian;
/// use teistro_calendar::bikram_sambat::engine::KATHMANDU;
/// use teistro_calendar::solar::local_mean_midnight;
///
/// // 85.324° east: mean midnight is 5 h 41 min 18 s before UT's.
/// let midnight = local_mean_midnight(fixed_from_gregorian(2024, 1, 1), &KATHMANDU)?;
/// assert!((midnight.get() - (2_460_310.5 - 85.324 / 360.0)).abs() < 1e-9);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
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
/// ```
/// # use teistro_calendar::bikram_sambat::engine::KATHMANDU;
/// # use teistro_core::time::UtcOffset;
/// # use teistro_siddhanta::SuryaSiddhanta;
/// # let (text, clock) = (SuryaSiddhanta::text(), UtcOffset::literal(5, 45, 0));
/// use teistro_calendar::gregorian::fixed_from_gregorian;
/// use teistro_calendar::solar::civil_day_light;
///
/// let arc = civil_day_light(&text, &clock, &KATHMANDU, fixed_from_gregorian(2024, 1, 1))?.arc().unwrap();
/// // Sunrise a little before 07:00 on Nepal's clock.
/// let local_hours = (arc.sunrise.get() - 2_460_310.5) * 24.0 + 5.75;
/// assert!((6.75..7.0).contains(&local_hours), "{local_hours}");
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
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

/// [`civil_day_light`] for many civil dates, through
/// [`SolarModel::day_lights`] so that a model sharing work between days
/// shares it; each date's answer is [`civil_day_light`]'s.
///
/// ```
/// # use teistro_calendar::bikram_sambat::engine::KATHMANDU;
/// # use teistro_core::time::UtcOffset;
/// # use teistro_siddhanta::SuryaSiddhanta;
/// # let (text, clock) = (SuryaSiddhanta::text(), UtcOffset::literal(5, 45, 0));
/// use teistro_calendar::gregorian::fixed_from_gregorian;
/// use teistro_calendar::solar::{civil_day_light, civil_day_lights};
///
/// let days = [fixed_from_gregorian(2024, 1, 1), fixed_from_gregorian(2024, 6, 21)];
/// let lights = civil_day_lights(&text, &clock, &KATHMANDU, &days);
/// assert_eq!(lights[1].as_ref().ok(), civil_day_light(&text, &clock, &KATHMANDU, days[1]).as_ref().ok());
/// ```
#[must_use]
pub fn civil_day_lights(
    model: &dyn SolarModel,
    clock: &dyn LocalClock,
    place: &Place,
    civil: &[FixedDay],
) -> Vec<Result<DayLight, Error>> {
    let days: Vec<FixedDay> = civil
        .iter()
        .map(|day| mean_time_day(clock, place, *day))
        .collect();
    model.day_lights(&days, place)
}

impl<M: SolarModel + ?Sized> SolarModel for &M {
    fn sidereal_sun_deg(&self, jd_ut: f64) -> Result<f64, Error> {
        (**self).sidereal_sun_deg(jd_ut)
    }

    fn day_light(&self, day: FixedDay, place: &Place) -> Result<DayLight, Error> {
        (**self).day_light(day, place)
    }

    fn day_lights(&self, days: &[FixedDay], place: &Place) -> Vec<Result<DayLight, Error>> {
        (**self).day_lights(days, place)
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
