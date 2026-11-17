//! The sunrise-anchored local day at a place: sunrise, sunset and the
//! next sunrise from a solar model, with the polar policies
//! (`docs/03-design/time-and-timezone.md`, §3.3 and §4).

use core::fmt;

use teistro_calendar::solar::{DayArc, DayLight, SolarModel, civil_day_light};
use teistro_calendar::{CalendarDate, CalendarSystem, FixedDay};
use teistro_core::catalogue::Vara;
use teistro_core::coverage::Clipped;
use teistro_core::error::{Error, Status};
use teistro_core::quantity::{JulianDay, Place, Utc};
use teistro_core::settings::{Air, PolarDayPolicy, SunriseConvention};
use teistro_core::time::LocalClock;

mod season;

/// Which polar state a day is in.
///
/// ```
/// use teistro_calendar::{CalendarDate, Gregorian};
/// use teistro_core::catalogue::Calendar;
/// use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
/// use teistro_core::settings::PolarDayPolicy;
/// use teistro_core::time::UtcOffset;
/// use teistro_siddhanta::SuryaSiddhanta;
/// use teistro_time::{DayState, PolarKind, local_day};
///
/// // Tromsø at midwinter: the Sun does not rise.
/// let tromso = Place::new(Latitude::literal(69.6492), Longitude::literal(18.9553), Altitude::literal(0.0));
/// let date = CalendarDate::defined(Calendar::Gregorian, 2024, 12, 21);
/// let (text, clock) = (SuryaSiddhanta::text(), UtcOffset::literal(1, 0, 0));
/// assert!(local_day(&text, &Gregorian, &clock, &tromso, &date, PolarDayPolicy::Undefined).is_err());
/// let day = local_day(&text, &Gregorian, &clock, &tromso, &date, PolarDayPolicy::CivilMidnight)?;
/// assert_eq!(day.state, DayState::Polar { kind: PolarKind::Night, policy: PolarDayPolicy::CivilMidnight });
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PolarKind {
    /// The Sun does not set.
    Day,
    /// The Sun does not rise.
    Night,
}

/// Whether the day had a sunrise, and what was done when it had not.
///
/// ```
/// # use teistro_calendar::{CalendarDate, Gregorian};
/// # use teistro_core::catalogue::Calendar;
/// # use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
/// # use teistro_core::settings::PolarDayPolicy;
/// # use teistro_core::time::UtcOffset;
/// # use teistro_siddhanta::SuryaSiddhanta;
/// # let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
/// # let date = CalendarDate::defined(Calendar::Gregorian, 2024, 1, 1);
/// # let clock = UtcOffset::literal(5, 45, 0);
/// // Monday 1 January 2024 at Kathmandu, sunrise to sunrise.
/// let day = teistro_time::local_day(&SuryaSiddhanta::text(), &Gregorian, &clock, &kathmandu, &date, PolarDayPolicy::Undefined)?;
/// use teistro_time::DayState;
///
/// assert_eq!(day.state, DayState::Normal);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "state", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DayState {
    /// Sunrise and sunset occurred.
    Normal,
    /// No horizon crossing; the policy synthesised the bounds.
    Polar {
        /// Day or night.
        kind: PolarKind,
        /// The policy applied.
        policy: PolarDayPolicy,
    },
}

/// A local day.
///
/// ```
/// # use teistro_calendar::{CalendarDate, Gregorian};
/// # use teistro_core::catalogue::Calendar;
/// # use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
/// # use teistro_core::settings::PolarDayPolicy;
/// # use teistro_core::time::UtcOffset;
/// # use teistro_siddhanta::SuryaSiddhanta;
/// # let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
/// # let date = CalendarDate::defined(Calendar::Gregorian, 2024, 1, 1);
/// # let clock = UtcOffset::literal(5, 45, 0);
/// // Monday 1 January 2024 at Kathmandu, sunrise to sunrise.
/// let day = teistro_time::local_day(&SuryaSiddhanta::text(), &Gregorian, &clock, &kathmandu, &date, PolarDayPolicy::Undefined)?;
/// use teistro_core::catalogue::Vara;
///
/// assert_eq!(day.vara, Vara::Somavara);
/// assert!(day.sunrise.get() < day.sunset.get() && day.sunset.get() < day.next_sunrise.get());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct LocalDay {
    /// The place.
    pub place: Place,
    /// The civil date.
    pub date: CalendarDate,
    /// The day's vara: the weekday of the date, which the sunrise-anchored
    /// reckoning keeps for the whole day from sunrise to sunrise.
    pub vara: Vara,
    /// Sunrise, or what the policy put in its place.
    pub sunrise: JulianDay<Utc>,
    /// Sunset.
    pub sunset: JulianDay<Utc>,
    /// The next sunrise.
    pub next_sunrise: JulianDay<Utc>,
    /// Whether the day is normal or polar.
    pub state: DayState,
    /// The sunrise convention the arc was reckoned by.
    pub convention: SunriseConvention,
    /// The model that gave the arc, for provenance.
    pub model: String,
    /// `END` when the next sunrise lies past the ephemeris's coverage
    /// (SR-136, C2244): `next_sunrise` is then the coverage's last instant
    /// and not a sunrise, the day's night is not known, and what divides
    /// it is left out of the day. The last civil day of the coverage is
    /// such a day wherever its next sunrise falls after the coverage
    /// ends. Absent on every other day, and then left out of its document.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clipped: Option<Clipped>,
}

impl LocalDay {
    /// The daylight, in days.
    ///
    /// ```
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::Calendar;
    /// # use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
    /// # use teistro_core::settings::PolarDayPolicy;
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_siddhanta::SuryaSiddhanta;
    /// # let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
    /// # let date = CalendarDate::defined(Calendar::Gregorian, 2024, 1, 1);
    /// # let clock = UtcOffset::literal(5, 45, 0);
    /// // Monday 1 January 2024 at Kathmandu, sunrise to sunrise.
    /// let day = teistro_time::local_day(&SuryaSiddhanta::text(), &Gregorian, &clock, &kathmandu, &date, PolarDayPolicy::Undefined)?;
    /// // A Kathmandu winter day: between ten and eleven hours of light.
    /// let hours = day.daylight_days() * 24.0;
    /// assert!((10.0..11.0).contains(&hours), "{hours}");
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn daylight_days(&self) -> f64 {
        self.sunset.get() - self.sunrise.get()
    }

    /// The night, in days.
    ///
    /// ```
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::Calendar;
    /// # use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
    /// # use teistro_core::settings::PolarDayPolicy;
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_siddhanta::SuryaSiddhanta;
    /// # let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
    /// # let date = CalendarDate::defined(Calendar::Gregorian, 2024, 1, 1);
    /// # let clock = UtcOffset::literal(5, 45, 0);
    /// // Monday 1 January 2024 at Kathmandu, sunrise to sunrise.
    /// let day = teistro_time::local_day(&SuryaSiddhanta::text(), &Gregorian, &clock, &kathmandu, &date, PolarDayPolicy::Undefined)?;
    /// assert!((day.daylight_days() + day.night_days() - 1.0).abs() < 0.01);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn night_days(&self) -> f64 {
        self.next_sunrise.get() - self.sunset.get()
    }

    /// The air the horizon was refracted through, resolved at the place,
    /// when the convention named one (`03-design/horizon-atmosphere.md`);
    /// `None` for the almanac's fixed 34′ or no refraction.
    ///
    /// ```
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::Calendar;
    /// # use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
    /// # use teistro_core::settings::PolarDayPolicy;
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_siddhanta::SuryaSiddhanta;
    /// # let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
    /// # let date = CalendarDate::defined(Calendar::Gregorian, 2024, 1, 1);
    /// # let clock = UtcOffset::literal(5, 45, 0);
    /// // Monday 1 January 2024 at Kathmandu, sunrise to sunrise.
    /// let day = teistro_time::local_day(&SuryaSiddhanta::text(), &Gregorian, &clock, &kathmandu, &date, PolarDayPolicy::Undefined)?;
    /// // The text's sunrise is its own, refracted through no named air.
    /// assert_eq!(day.air(), None);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn air(&self) -> Option<Air> {
        match self.convention {
            SunriseConvention::Atmospheric { air, .. } => Some(air.at(self.place.altitude)),
            SunriseConvention::Named { .. } | SunriseConvention::Custom { .. } => None,
        }
    }

    /// Whether an instant lies in this day, from its sunrise to the next.
    ///
    /// ```
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::Calendar;
    /// # use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
    /// # use teistro_core::settings::PolarDayPolicy;
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_siddhanta::SuryaSiddhanta;
    /// # let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
    /// # let date = CalendarDate::defined(Calendar::Gregorian, 2024, 1, 1);
    /// # let clock = UtcOffset::literal(5, 45, 0);
    /// // Monday 1 January 2024 at Kathmandu, sunrise to sunrise.
    /// let day = teistro_time::local_day(&SuryaSiddhanta::text(), &Gregorian, &clock, &kathmandu, &date, PolarDayPolicy::Undefined)?;
    /// assert!(day.contains(day.sunrise));
    /// assert!(!day.contains(day.next_sunrise));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn contains(&self, instant: JulianDay<Utc>) -> bool {
        instant.get() >= self.sunrise.get() && instant.get() < self.next_sunrise.get()
    }
}

impl fmt::Display for LocalDay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at {}: sunrise {}, sunset {}, next sunrise {}",
            self.date, self.place, self.sunrise, self.sunset, self.next_sunrise
        )
    }
}

/// The instant a fixed day's local civil midnight falls at.
///
/// Two modules need it — the polar synthesis here, and the daily
/// panchanga, whose window is the civil day under a midnight boundary and
/// whose moonrise may be searched from one — so it is public rather than
/// written twice.
///
/// ```
/// use teistro_calendar::gregorian::fixed_from_gregorian;
/// use teistro_core::time::UtcOffset;
/// use teistro_time::local_day::local_midnight;
///
/// // Kathmandu's midnight is 18:15 UTC the day before.
/// let midnight = local_midnight(&UtcOffset::literal(5, 45, 0), fixed_from_gregorian(2024, 1, 1))?;
/// assert!((midnight.get() - (2_460_310.5 - 5.75 / 24.0)).abs() < 1e-9);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// A fixed day outside the Julian day's range.
pub fn local_midnight(clock: &dyn LocalClock, day: FixedDay) -> Result<JulianDay<Utc>, Error> {
    let jd = day.jd_at_midnight()?;
    Ok(JulianDay::try_new(jd.get() - clock.offset_at(jd).days())?)
}

/// The next sunrise after a day whose following day has none, under the
/// policy: the following day's own synthesised sunrise, so the two days
/// touch.
fn next_sunrise_after(
    model: &dyn SolarModel,
    clock: &dyn LocalClock,
    place: &Place,
    day: FixedDay,
    policy: PolarDayPolicy,
    kind: PolarKind,
) -> Result<JulianDay<Utc>, Error> {
    synthesised(model, clock, place, day.plus_days(1), kind, policy).map(|bounds| bounds.sunrise)
}

/// The arc a civil day has under a policy: the sky's where the Sun rises
/// and sets, and otherwise what `polar_day_policy` stands in for it, the
/// sunrise and sunset [`local_day`] gives the same date. A rule that
/// reads a day's arc reads this one, so it and the day's own row cannot
/// disagree (`03-design/calendar-bikram-sambat.md` §6).
///
/// ```
/// use teistro_calendar::gregorian::fixed_from_gregorian;
/// use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
/// use teistro_core::settings::PolarDayPolicy;
/// use teistro_core::time::UtcOffset;
/// use teistro_siddhanta::SuryaSiddhanta;
/// use teistro_time::local_day::{local_midnight, standing_arc};
///
/// let tromso = Place::new(Latitude::literal(69.6492), Longitude::literal(18.9553), Altitude::literal(0.0));
/// let (text, clock) = (SuryaSiddhanta::text(), UtcOffset::literal(1, 0, 0));
/// let solstice = fixed_from_gregorian(2024, 6, 21);
/// // Under the midnight sun, CIVIL_MIDNIGHT stands the civil day in.
/// let arc = standing_arc(&text, &clock, &tromso, solstice, PolarDayPolicy::CivilMidnight)?;
/// assert_eq!(arc.sunrise, local_midnight(&clock, solstice)?);
/// // UNDEFINED stands nothing in, and says so.
/// let refused = standing_arc(&text, &clock, &tromso, solstice, PolarDayPolicy::Undefined).unwrap_err();
/// assert_eq!(refused.field(), Some("day.polar_day_policy"));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// The model's refusal; a day with no arc under `UNDEFINED`,
/// `UNSUPPORTED` on `day.polar_day_policy`.
pub fn standing_arc(
    model: &dyn SolarModel,
    clock: &dyn LocalClock,
    place: &Place,
    day: FixedDay,
    policy: PolarDayPolicy,
) -> Result<DayArc, Error> {
    let kind = match civil_day_light(model, clock, place, day)? {
        DayLight::Arc(arc) => return Ok(arc),
        DayLight::AlwaysUp => PolarKind::Day,
        DayLight::NeverUp => PolarKind::Night,
    };
    let bounds = synthesised(model, clock, place, day, kind, policy)?;
    Ok(DayArc {
        sunrise: bounds.sunrise,
        sunset: bounds.sunset,
    })
}

/// The bounds a policy gives a day: sunrise, sunset, next sunrise.
struct Bounds {
    sunrise: JulianDay<Utc>,
    sunset: JulianDay<Utc>,
    next_sunrise: JulianDay<Utc>,
}

/// The local day of a civil date at a place.
///
/// ```
/// # use teistro_calendar::{CalendarDate, Gregorian};
/// # use teistro_core::catalogue::Calendar;
/// # use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
/// # use teistro_core::settings::PolarDayPolicy;
/// # use teistro_core::time::UtcOffset;
/// # use teistro_siddhanta::SuryaSiddhanta;
/// # let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
/// # let date = CalendarDate::defined(Calendar::Gregorian, 2024, 1, 1);
/// # let clock = UtcOffset::literal(5, 45, 0);
/// // Monday 1 January 2024 at Kathmandu, sunrise to sunrise.
/// let day = teistro_time::local_day(&SuryaSiddhanta::text(), &Gregorian, &clock, &kathmandu, &date, PolarDayPolicy::Undefined)?;
/// // The day is named by its civil date and runs from that date's sunrise.
/// assert_eq!(day.date, date);
/// assert!(day.sunrise.get() > 2_460_310.5 - 5.75 / 24.0);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// A date the calendar does not have; the model's refusal; a polar day
/// under the `UNDEFINED` policy (`UNSUPPORTED`, naming the policies that
/// synthesise a day).
pub fn local_day(
    model: &dyn SolarModel,
    calendar: &dyn CalendarSystem,
    clock: &dyn LocalClock,
    place: &Place,
    date: &CalendarDate,
    policy: PolarDayPolicy,
) -> Result<LocalDay, Error> {
    local_day_carrying(
        model,
        calendar,
        clock,
        place,
        date,
        policy,
        &mut Carried::default(),
    )
}

/// What one local day leaves for the next: the next civil day's light at
/// the place, which is that day's own.
///
/// A day reads its own light and the next day's, so a range asking day
/// after day would reckon every light but the ends twice. A carry spends
/// it once. It holds for one model and one clock; the day and the place
/// are checked, and a carry from any other day or place is reckoned
/// afresh, so a stale one costs a reading and never changes an answer.
///
/// ```
/// # use teistro_calendar::{CalendarDate, Gregorian};
/// # use teistro_core::catalogue::Calendar;
/// # use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
/// # use teistro_core::settings::PolarDayPolicy;
/// # use teistro_core::time::UtcOffset;
/// # use teistro_siddhanta::SuryaSiddhanta;
/// # let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
/// # let date = CalendarDate::defined(Calendar::Gregorian, 2024, 1, 1);
/// # let clock = UtcOffset::literal(5, 45, 0);
/// use teistro_time::local_day::{Carried, local_day, local_day_carrying};
///
/// let text = SuryaSiddhanta::text();
/// let mut carried = Carried::default();
/// let first = local_day_carrying(&text, &Gregorian, &clock, &kathmandu, &date, PolarDayPolicy::Undefined, &mut carried)?;
/// assert_eq!(first, local_day(&text, &Gregorian, &clock, &kathmandu, &date, PolarDayPolicy::Undefined)?);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct Carried(Option<(FixedDay, Place, DayLight)>);

/// [`local_day`], taking the day's light from `carried` when the day
/// before left it there and leaving the next day's for the day after.
/// The answer is [`local_day`]'s to the bit: a light is a function of the
/// model, the clock, the place and the day, and a carry is used only for
/// the same day and place.
///
/// ```
/// use teistro_calendar::{CalendarDate, Gregorian};
/// use teistro_core::catalogue::Calendar;
/// use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
/// use teistro_core::settings::PolarDayPolicy;
/// use teistro_core::time::UtcOffset;
/// use teistro_siddhanta::SuryaSiddhanta;
/// use teistro_time::local_day::{Carried, local_day, local_day_carrying};
///
/// let text = SuryaSiddhanta::text();
/// let clock = UtcOffset::literal(5, 45, 0);
/// let place = Place::new(
///     Latitude::literal(27.7172),
///     Longitude::literal(85.324),
///     Altitude::literal(1400.0),
/// );
/// let policy = PolarDayPolicy::Undefined;
/// let mut carried = Carried::default();
/// for day in 1..=3 {
///     let date = CalendarDate::defined(Calendar::Gregorian, 2024, 1, day);
///     let ranged = local_day_carrying(&text, &Gregorian, &clock, &place, &date, policy, &mut carried)?;
///     assert_eq!(ranged, local_day(&text, &Gregorian, &clock, &place, &date, policy)?);
/// }
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// As [`local_day`].
pub fn local_day_carrying(
    model: &dyn SolarModel,
    calendar: &dyn CalendarSystem,
    clock: &dyn LocalClock,
    place: &Place,
    date: &CalendarDate,
    policy: PolarDayPolicy,
    carried: &mut Carried,
) -> Result<LocalDay, Error> {
    let day = calendar.fixed_of(date)?;
    let today = match carried.0.take() {
        Some((left_for, at, light)) if left_for == day && at == *place => light,
        _ => civil_day_light(model, clock, place, day)?,
    };
    let tomorrow = within_coverage(civil_day_light(model, clock, place, day.plus_days(1)))?;
    carried.0 = tomorrow.map(|light| (day.plus_days(1), *place, light));
    local_day_lit(model, clock, place, (date, day), policy, (today, tomorrow))
}

/// A day's light, or `None` when it lies past the ephemeris's coverage
/// (an `OUT_OF_RANGE` refusal): what the next civil day's light is to a
/// day, whose own is then cut at the coverage's end (SR-136).
///
/// ```
/// use teistro_calendar::solar::DayLight;
/// use teistro_core::error::{Error, Status};
/// use teistro_time::local_day::within_coverage;
///
/// assert_eq!(within_coverage(Ok(DayLight::NeverUp))?, Some(DayLight::NeverUp));
/// assert_eq!(within_coverage(Err(Error::new(Status::OutOfRange, "past the end")))?, None);
/// assert!(within_coverage(Err(Error::invalid_arg("a bad place"))).is_err());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// Any other refusal, as it came.
pub fn within_coverage(light: Result<DayLight, Error>) -> Result<Option<DayLight>, Error> {
    match light {
        Ok(light) => Ok(Some(light)),
        Err(error) if error.status == Status::OutOfRange => Ok(None),
        Err(error) => Err(error),
    }
}

/// The local day of a civil date from its light and the next civil
/// day's, already found ([`civil_day_light`], or many at once through
/// [`teistro_calendar::solar::civil_day_lights`]). The answer is
/// [`local_day`]'s for the same lights, to the bit; a polar day still
/// asks the model for the days its policy reaches for.
///
/// ```
/// # use teistro_calendar::{CalendarDate, Gregorian};
/// # use teistro_core::catalogue::Calendar;
/// # use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
/// # use teistro_core::settings::PolarDayPolicy;
/// # use teistro_core::time::UtcOffset;
/// # use teistro_siddhanta::SuryaSiddhanta;
/// # let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
/// # let date = CalendarDate::defined(Calendar::Gregorian, 2024, 1, 1);
/// # let clock = UtcOffset::literal(5, 45, 0);
/// use teistro_calendar::gregorian::fixed_from_gregorian;
/// use teistro_calendar::solar::civil_day_light;
/// use teistro_time::local_day::{local_day, local_day_lit};
///
/// let text = SuryaSiddhanta::text();
/// let fixed = fixed_from_gregorian(2024, 1, 1);
/// let lights = (
///     civil_day_light(&text, &clock, &kathmandu, fixed)?,
///     Some(civil_day_light(&text, &clock, &kathmandu, fixed.plus_days(1))?),
/// );
/// let lit = local_day_lit(&text, &clock, &kathmandu, (&date, fixed), PolarDayPolicy::Undefined, lights)?;
/// assert_eq!(lit, local_day(&text, &Gregorian, &clock, &kathmandu, &date, PolarDayPolicy::Undefined)?);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// As [`local_day`].
pub fn local_day_lit(
    model: &dyn SolarModel,
    clock: &dyn LocalClock,
    place: &Place,
    (date, day): (&CalendarDate, FixedDay),
    policy: PolarDayPolicy,
    (today, tomorrow): (DayLight, Option<DayLight>),
) -> Result<LocalDay, Error> {
    let Some(tomorrow) = tomorrow else {
        return cut_at_the_end(model, place, (date, day), today);
    };
    let (sunrise, sunset, next_sunrise, state) = match (today, tomorrow) {
        (DayLight::Arc(arc), DayLight::Arc(next)) => {
            (arc.sunrise, arc.sunset, next.sunrise, DayState::Normal)
        }
        (DayLight::Arc(arc), polar) => {
            // The last day before a polar season: the next sunrise is the
            // first after this one under the policy.
            let kind = polar_kind(polar);
            let next = next_sunrise_after(model, clock, place, day, policy, kind)?;
            (
                arc.sunrise,
                arc.sunset,
                not_before(next, arc.sunset, policy),
                DayState::Polar { kind, policy },
            )
        }
        (polar, next) => {
            let kind = polar_kind(polar);
            let bounds = synthesised(model, clock, place, day, kind, policy)?;
            // The last day of a polar season under `CIVIL_MIDNIGHT` runs to
            // the first real sunrise, not to midnight: the real day begins
            // at that sunrise, so ending at midnight left the small hours
            // between in no day at all. That sunrise can come before the
            // midnight (a day's sunrise is found by its own rule, not the
            // clock's date), so the synthesised sunset stops at it.
            let next_sunrise = match (policy, next) {
                (PolarDayPolicy::CivilMidnight, DayLight::Arc(arc)) => arc.sunrise,
                _ => bounds.next_sunrise,
            };
            // The first day of a season begins at its midnight, which is
            // its predecessor's next sunrise; but not before that day's
            // real sunset, which can fall after the midnight.
            let sunrise = match policy {
                PolarDayPolicy::CivilMidnight => {
                    match civil_day_light(model, clock, place, day.plus_days(-1))? {
                        DayLight::Arc(before) => not_before(bounds.sunrise, before.sunset, policy),
                        _ => bounds.sunrise,
                    }
                }
                _ => bounds.sunrise,
            };
            (
                sunrise,
                not_after(bounds.sunset, next_sunrise, policy),
                next_sunrise,
                DayState::Polar { kind, policy },
            )
        }
    };
    Ok(LocalDay {
        place: *place,
        date: date.clone(),
        vara: day.weekday().vara(),
        sunrise,
        sunset,
        next_sunrise,
        state,
        convention: model.convention(),
        model: model.describe(),
        clipped: None,
    })
}

/// The day whose next sunrise lies past the coverage (SR-136, C2244): its
/// own sunrise and sunset, and the coverage's last instant where the next
/// sunrise would be, marked `END`. A day with no arc of its own there has
/// no night to stand in for and is refused `OUT_OF_RANGE` on `date`.
fn cut_at_the_end(
    model: &dyn SolarModel,
    place: &Place,
    (date, day): (&CalendarDate, FixedDay),
    today: DayLight,
) -> Result<LocalDay, Error> {
    let end = model.coverage().to;
    let arc = today
        .arc()
        .filter(|arc| arc.sunset.get() <= end && end.is_finite());
    let Some(arc) = arc else {
        return Err(Error::new(
            Status::OutOfRange,
            format!("{date}: the day's light reaches past the ephemeris's coverage"),
        )
        .with_field("date"));
    };
    Ok(LocalDay {
        place: *place,
        date: date.clone(),
        vara: day.weekday().vara(),
        sunrise: arc.sunrise,
        sunset: arc.sunset,
        next_sunrise: JulianDay::try_new(end)?,
        state: DayState::Normal,
        convention: model.convention(),
        model: model.describe(),
        clipped: Some(Clipped::End),
    })
}

/// Under `CIVIL_MIDNIGHT`, a synthesised bound moved past a real event
/// it would otherwise precede (later of the two).
fn not_before(
    synthesised: JulianDay<Utc>,
    real: JulianDay<Utc>,
    policy: PolarDayPolicy,
) -> JulianDay<Utc> {
    if policy == PolarDayPolicy::CivilMidnight && real.get() > synthesised.get() {
        real
    } else {
        synthesised
    }
}

/// Under `CIVIL_MIDNIGHT`, a synthesised bound held back to a real event
/// it would otherwise follow (earlier of the two).
fn not_after(
    synthesised: JulianDay<Utc>,
    real: JulianDay<Utc>,
    policy: PolarDayPolicy,
) -> JulianDay<Utc> {
    if policy == PolarDayPolicy::CivilMidnight && real.get() < synthesised.get() {
        real
    } else {
        synthesised
    }
}

fn polar_kind(light: DayLight) -> PolarKind {
    match light {
        DayLight::NeverUp => PolarKind::Night,
        DayLight::Arc(_) | DayLight::AlwaysUp => PolarKind::Day,
    }
}

/// The bounds a policy gives a polar day: sunrise, sunset, next sunrise.
fn synthesised(
    model: &dyn SolarModel,
    clock: &dyn LocalClock,
    place: &Place,
    day: FixedDay,
    kind: PolarKind,
    policy: PolarDayPolicy,
) -> Result<Bounds, Error> {
    match policy {
        PolarDayPolicy::Undefined => Err(Error::unsupported(format!(
            "{day} at {place} has no sunrise (polar {}) and the polar-day policy is UNDEFINED",
            match kind {
                PolarKind::Day => "day",
                PolarKind::Night => "night",
            }
        ))
        .with_field("day.polar_day_policy")
        .with_hint("choose NEAREST_EVENT or CIVIL_MIDNIGHT to synthesise the day's bounds")),
        PolarDayPolicy::CivilMidnight => {
            // The civil day under the clock, as a day without a night.
            let start = local_midnight(clock, day)?;
            let end = local_midnight(clock, day.plus_days(1))?;
            Ok(Bounds {
                sunrise: start,
                sunset: end,
                next_sunrise: end,
            })
        }
        PolarDayPolicy::NearestEvent => {
            season::bounds(season::Season::of(model, clock, place, day, kind)?, day)
        }
        other => Err(crate::ghati::unknown_knob(
            "day.polar_day_policy",
            &format!("{other:?}"),
        )),
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use teistro_calendar::{Gregorian, gregorian};
    use teistro_core::catalogue::Calendar;
    use teistro_core::quantity::{Altitude, Latitude, Longitude};
    use teistro_core::time::UtcOffset;
    use teistro_siddhanta::SuryaSiddhanta;

    use super::*;
    use crate::zones;

    /// A standing arc is the sky's where it has one, and otherwise the
    /// same sunrise and sunset the day's own row gives under the policy.
    #[test]
    fn a_standing_arc_is_the_day_row_s_own() {
        let text = SuryaSiddhanta::text();
        let clock = UtcOffset::literal(1, 0, 0);
        let tromso = Place::new(
            Latitude::literal(69.6492),
            Longitude::literal(18.9553),
            Altitude::literal(0.0),
        );
        for (year, month, day) in [(2024, 7, 16), (2024, 1, 5), (2024, 3, 20)] {
            let date = CalendarDate::defined(Calendar::Gregorian, year, month, day);
            let fixed = gregorian::fixed_from_gregorian(year, month, day);
            for policy in [PolarDayPolicy::NearestEvent, PolarDayPolicy::CivilMidnight] {
                let row = local_day(&text, &Gregorian, &clock, &tromso, &date, policy).unwrap();
                let arc = standing_arc(&text, &clock, &tromso, fixed, policy).unwrap();
                assert_eq!((arc.sunrise, arc.sunset), (row.sunrise, row.sunset));
            }
        }
        let refused = standing_arc(
            &text,
            &clock,
            &tromso,
            gregorian::fixed_from_gregorian(2024, 1, 5),
            PolarDayPolicy::Undefined,
        )
        .unwrap_err();
        assert_eq!(refused.field(), Some("day.polar_day_policy"));
        assert!(refused.message.contains("polar night"), "{refused}");
    }

    const KATHMANDU: Place = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );

    /// Samoa moved to UTC+14 by skipping 30 December 2011, so its clock keeps
    /// 25½ hours from the mean time of its longitude. A civil day's sunrise
    /// must still fall inside that civil day, and the next one inside the next.
    #[test]
    fn a_civil_day_far_from_its_mean_time_rises_inside_itself() {
        const APIA: Place = Place::new(
            Latitude::literal(-13.8506),
            Longitude::literal(-171.7513),
            Altitude::literal(2.0),
        );
        let text = SuryaSiddhanta::text();
        let clock = UtcOffset::literal(14, 0, 0);
        for day in [30, 31] {
            let date = CalendarDate::defined(Calendar::Gregorian, 2011, 12, day);
            let local = local_day(
                &text,
                &Gregorian,
                &clock,
                &APIA,
                &date,
                PolarDayPolicy::Undefined,
            )
            .unwrap();
            let civil = Gregorian.fixed_of(&date).unwrap();
            let start = local_midnight(&clock, civil).unwrap().get();
            let end = local_midnight(&clock, civil.plus_days(1)).unwrap().get();
            for (name, at) in [("sunrise", local.sunrise), ("sunset", local.sunset)] {
                assert!(
                    start <= at.get() && at.get() < end,
                    "{name} of {date} at {at}"
                );
            }
            assert!(end <= local.next_sunrise.get(), "{date}");
        }
    }

    /// Through a polar night and out of it, under every policy that
    /// synthesises a day, a carried range is its days alone, to the bit.
    #[test]
    fn a_carried_range_is_its_days_through_a_polar_season() {
        const TROMSO: Place = Place::new(
            Latitude::literal(69.6492),
            Longitude::literal(18.9553),
            Altitude::literal(10.0),
        );
        let text = SuryaSiddhanta::text();
        let clock = UtcOffset::literal(1, 0, 0);
        for policy in [PolarDayPolicy::NearestEvent, PolarDayPolicy::CivilMidnight] {
            let mut carried = Carried::default();
            let start = Gregorian
                .fixed_of(&CalendarDate::defined(Calendar::Gregorian, 2023, 11, 15))
                .unwrap();
            for offset in 0..75 {
                let date = Gregorian.date_of(start.plus_days(offset)).unwrap();
                let ranged = local_day_carrying(
                    &text,
                    &Gregorian,
                    &clock,
                    &TROMSO,
                    &date,
                    policy,
                    &mut carried,
                )
                .unwrap();
                let alone = local_day(&text, &Gregorian, &clock, &TROMSO, &date, policy).unwrap();
                assert_eq!(ranged, alone, "{date} under {policy:?}");
            }
        }
    }

    /// A light carried for another day or another place is not taken:
    /// the answer is that day's own.
    #[test]
    fn a_light_carried_for_another_day_or_place_is_recomputed() {
        const POKHARA: Place = Place::new(
            Latitude::literal(28.2096),
            Longitude::literal(83.9856),
            Altitude::literal(820.0),
        );
        let text = SuryaSiddhanta::text();
        let clock = UtcOffset::literal(5, 45, 0);
        let policy = PolarDayPolicy::Undefined;
        let kathmandu = Place::new(
            Latitude::literal(27.7172),
            Longitude::literal(85.324),
            Altitude::literal(1400.0),
        );
        let first = CalendarDate::defined(Calendar::Gregorian, 2024, 6, 21);
        let far = CalendarDate::defined(Calendar::Gregorian, 2024, 9, 1);
        let next = CalendarDate::defined(Calendar::Gregorian, 2024, 6, 22);
        for (place, date) in [(kathmandu, &far), (POKHARA, &next)] {
            let mut carried = Carried::default();
            local_day_carrying(
                &text,
                &Gregorian,
                &clock,
                &kathmandu,
                &first,
                policy,
                &mut carried,
            )
            .unwrap();
            let ranged = local_day_carrying(
                &text,
                &Gregorian,
                &clock,
                &place,
                date,
                policy,
                &mut carried,
            )
            .unwrap();
            let alone = local_day(&text, &Gregorian, &clock, &place, date, policy).unwrap();
            assert_eq!(ranged, alone, "{date}");
        }
    }

    #[test]
    fn a_normal_day_at_kathmandu() {
        let text = SuryaSiddhanta::text();
        let date = CalendarDate::defined(Calendar::Gregorian, 2024, 6, 21);
        let day = local_day(
            &text,
            &Gregorian,
            zones::nepal(),
            &KATHMANDU,
            &date,
            PolarDayPolicy::Undefined,
        )
        .unwrap();
        assert_eq!(day.state, DayState::Normal);
        assert!((day.daylight_days() * 24.0 - 13.8).abs() < 0.1, "{day}");
        assert!((day.night_days() * 24.0 - 10.2).abs() < 0.1);
        assert!(day.contains(day.sunrise) && !day.contains(day.next_sunrise));
        assert!(day.model.starts_with("Surya Siddhanta"));
        assert_eq!(day.vara, Vara::Shukravara, "2024-06-21 is a Friday");
        assert_eq!(
            day.convention,
            teistro_core::settings::Sunrise::CentreNoRefraction.into()
        );
        assert!(day.to_string().contains("sunrise"));
    }

    /// A model with a polar night from day 100 to day 200 of its epoch.
    struct Polar;

    impl SolarModel for Polar {
        fn sidereal_sun_deg(&self, _jd_ut: f64) -> Result<f64, Error> {
            Ok(0.0)
        }

        fn day_light(&self, day: FixedDay, _place: &Place) -> Result<DayLight, Error> {
            let index = day.get();
            if (100..200).contains(&index) {
                return Ok(DayLight::NeverUp);
            }
            let midnight = day.jd_at_midnight()?;
            Ok(DayLight::Arc(DayArc {
                sunrise: midnight.plus_days(0.25)?,
                sunset: midnight.plus_days(0.75)?,
            }))
        }

        fn describe(&self) -> String {
            String::from("polar test model")
        }

        fn convention(&self) -> SunriseConvention {
            teistro_core::settings::Sunrise::CentreNoRefraction.into()
        }
    }

    /// A midnight sun from day 100 to day 199 whose seams cross civil
    /// midnight, as Tromsø's and Rovaniemi's do: day 99 sets after day
    /// 100's midnight, and day 200 rises before its own.
    struct SeamsPastMidnight;

    impl SolarModel for SeamsPastMidnight {
        fn sidereal_sun_deg(&self, _jd_ut: f64) -> Result<f64, Error> {
            Ok(0.0)
        }

        fn day_light(&self, day: FixedDay, _place: &Place) -> Result<DayLight, Error> {
            let midnight = day.jd_at_midnight()?;
            let (rise, set) = match day.get() {
                100..200 => return Ok(DayLight::AlwaysUp),
                99 => (0.05, 1.02),
                200 => (-0.01, 0.9),
                _ => (0.25, 0.75),
            };
            Ok(DayLight::Arc(DayArc {
                sunrise: midnight.plus_days(rise)?,
                sunset: midnight.plus_days(set)?,
            }))
        }

        fn describe(&self) -> String {
            String::from("seams past midnight")
        }

        fn convention(&self) -> SunriseConvention {
            teistro_core::settings::Sunrise::CentreNoRefraction.into()
        }
    }

    /// Under `CIVIL_MIDNIGHT` the days run in order and touch at both seams:
    /// a synthesised midnight gave way to neither the real sunset after it
    /// nor the real sunrise before it, so a night ended before it began.
    #[test]
    fn civil_midnight_days_touch_across_seams_past_midnight() {
        let place = Place::new(
            Latitude::literal(69.0),
            Longitude::literal(0.0),
            Altitude::literal(0.0),
        );
        let day = |n: i64| {
            let date = Gregorian.date_of(FixedDay::new(n)).unwrap();
            local_day(
                &SeamsPastMidnight,
                &Gregorian,
                &UtcOffset::UTC,
                &place,
                &date,
                PolarDayPolicy::CivilMidnight,
            )
            .unwrap_or_else(|e| panic!("day {n}: {e}"))
        };
        for n in 97..203 {
            let (this, next) = (day(n), day(n + 1));
            let (rise, set, ends) = (
                this.sunrise.get(),
                this.sunset.get(),
                this.next_sunrise.get(),
            );
            assert!(rise <= set && set <= ends, "day {n}: {rise} {set} {ends}");
            assert_eq!(ends, next.sunrise.get(), "day {n} and the next touch");
        }
        // The real events stand; the synthesised midnights give way.
        assert!(
            (day(99).sunset.get() - (FixedDay::new(99).jd_at_midnight().unwrap().get() + 1.02))
                .abs()
                < 1e-9
        );
        assert!(
            (day(199).next_sunrise.get()
                - (FixedDay::new(200).jd_at_midnight().unwrap().get() - 0.01))
                .abs()
                < 1e-9
        );
    }

    #[test]
    fn polar_days_follow_the_policy() {
        let place = Place::new(
            Latitude::literal(80.0),
            Longitude::literal(0.0),
            Altitude::literal(0.0),
        );
        let calendar = Gregorian;
        let day_150 = calendar.date_of(FixedDay::new(150)).unwrap();
        let clock = UtcOffset::UTC;
        let error = local_day(
            &Polar,
            &calendar,
            &clock,
            &place,
            &day_150,
            PolarDayPolicy::Undefined,
        )
        .unwrap_err();
        assert!(error.message.contains("polar night"));
        let midnight = local_day(
            &Polar,
            &calendar,
            &clock,
            &place,
            &day_150,
            PolarDayPolicy::CivilMidnight,
        )
        .unwrap();
        assert_eq!(
            midnight.state,
            DayState::Polar {
                kind: PolarKind::Night,
                policy: PolarDayPolicy::CivilMidnight
            }
        );
        assert!(
            (midnight.daylight_days() - 1.0).abs() < 1e-9 && midnight.night_days().abs() < 1e-9
        );
        let nearest = local_day(
            &Polar,
            &calendar,
            &clock,
            &place,
            &day_150,
            PolarDayPolicy::NearestEvent,
        )
        .unwrap();
        assert_eq!(
            nearest.state,
            DayState::Polar {
                kind: PolarKind::Night,
                policy: PolarDayPolicy::NearestEvent
            }
        );
        // Day 150 is its own day, carried from day 99's sunrise to day
        // 200's in equal steps (C1900): both rise at 06:00, so it does too.
        let (found, hour) = FixedDay::from_jd(nearest.sunrise);
        assert_eq!(found, FixedDay::new(150));
        assert!((hour - 0.25).abs() < 1e-9, "{hour}");
        assert!((nearest.next_sunrise.get() - nearest.sunrise.get() - 1.0).abs() < 1e-9);
        // The last day before the night runs to the night's first.
        let day_99 = calendar.date_of(FixedDay::new(99)).unwrap();
        let edge = local_day(
            &Polar,
            &calendar,
            &clock,
            &place,
            &day_99,
            PolarDayPolicy::NearestEvent,
        )
        .unwrap();
        assert!(matches!(
            edge.state,
            DayState::Polar {
                kind: PolarKind::Night,
                ..
            }
        ));
        assert_eq!(FixedDay::from_jd(edge.next_sunrise).0, FixedDay::new(100));
        let (y, m, d) = gregorian::gregorian_from_fixed(FixedDay::new(150));
        assert_eq!((y, m, d), (1, 5, 30));
    }

    /// The day of a fixed day under a policy, at a place on a clock.
    fn day_of(
        model: &dyn SolarModel,
        clock: &dyn LocalClock,
        place: &Place,
        day: FixedDay,
        policy: PolarDayPolicy,
    ) -> LocalDay {
        let date = Gregorian.date_of(day).unwrap();
        local_day(model, &Gregorian, clock, place, &date, policy)
            .unwrap_or_else(|e| panic!("{date} at {place}: {e}"))
    }

    /// What every day of a polar season must be under `NEAREST_EVENT`
    /// (SR-103): its own, 23 to 25 hours long, its sunset inside it, and
    /// touching the days either side to the bit.
    fn assert_its_own_day(
        model: &dyn SolarModel,
        clock: &dyn LocalClock,
        place: &Place,
        day: FixedDay,
    ) {
        let policy = PolarDayPolicy::NearestEvent;
        let (before, this, after) = (
            day_of(model, clock, place, day.plus_days(-1), policy),
            day_of(model, clock, place, day, policy),
            day_of(model, clock, place, day.plus_days(1), policy),
        );
        // The day before is held too when it is the season's last real
        // day, whose next sunrise is the season's first.
        for row in [&before, &this] {
            if matches!(row.state, DayState::Polar { .. }) {
                let hours = (row.next_sunrise.get() - row.sunrise.get()) * 24.0;
                assert!((23.0..=25.0).contains(&hours), "{row}: {hours} h");
            }
        }
        assert!(
            this.sunrise.get() <= this.sunset.get() && this.sunset.get() <= this.next_sunrise.get(),
            "{this}"
        );
        assert_eq!(
            before.next_sunrise, this.sunrise,
            "{this} meets the day before"
        );
        assert_eq!(
            this.next_sunrise, after.sunrise,
            "{this} meets the day after"
        );
    }

    /// Tromsø, 18 to 26 May 2025 (the consumer's SR-103): each date was
    /// one 68-day window; each is now a day of its own.
    #[test]
    fn tromso_s_first_polar_days_are_each_their_own() {
        let text = SuryaSiddhanta::text();
        let clock = UtcOffset::literal(2, 0, 0);
        let tromso = Place::new(
            Latitude::literal(69.6492),
            Longitude::literal(18.9553),
            Altitude::literal(0.0),
        );
        let first = gregorian::fixed_from_gregorian(2025, 5, 18);
        let mut sunrises = Vec::new();
        for offset in 0..9 {
            let day = first.plus_days(offset);
            assert_its_own_day(&text, &clock, &tromso, day);
            let row = day_of(&text, &clock, &tromso, day, PolarDayPolicy::NearestEvent);
            sunrises.push(FixedDay::from_jd(row.sunrise.plus_days(2.0 / 24.0).unwrap()).0);
        }
        // Each sunrise falls on its own civil date, on Tromsø's clock.
        let dates: Vec<FixedDay> = (0..9).map(|offset| first.plus_days(offset)).collect();
        assert_eq!(sunrises, dates);
    }

    /// A polar night meeting a midnight sun with no arc between, as near
    /// the pole: each season runs to the seam, which opens the later one
    /// on its own hour (mean midnight for a midnight sun), and the days
    /// touch across it.
    #[test]
    fn a_seam_with_no_arc_between_is_crossed_day_by_day() {
        struct Pole;
        impl SolarModel for Pole {
            fn sidereal_sun_deg(&self, _jd_ut: f64) -> Result<f64, Error> {
                Ok(0.0)
            }
            fn day_light(&self, day: FixedDay, _place: &Place) -> Result<DayLight, Error> {
                Ok(if day.get().rem_euclid(360) < 180 {
                    DayLight::NeverUp
                } else {
                    DayLight::AlwaysUp
                })
            }
            fn describe(&self) -> String {
                String::from("pole")
            }
            fn convention(&self) -> SunriseConvention {
                teistro_core::settings::Sunrise::CentreNoRefraction.into()
            }
        }
        let place = Place::new(
            Latitude::literal(89.9),
            Longitude::literal(0.0),
            Altitude::literal(0.0),
        );
        for n in [1, 90, 178, 179, 180, 181, 270, 358, 359, 360] {
            assert_its_own_day(&Pole, &UtcOffset::UTC, &place, FixedDay::new(n));
        }
        let seam = day_of(
            &Pole,
            &UtcOffset::UTC,
            &place,
            FixedDay::new(180),
            PolarDayPolicy::NearestEvent,
        );
        let (on, hour) = FixedDay::from_jd(seam.sunrise);
        assert!(on == FixedDay::new(180) && hour.abs() < 1e-9, "{seam}");
        // A midnight sun has no night; a polar night no daylight.
        assert_eq!(seam.sunset, seam.next_sunrise);
        let dark = day_of(
            &Pole,
            &UtcOffset::UTC,
            &place,
            FixedDay::new(90),
            PolarDayPolicy::NearestEvent,
        );
        assert_eq!(dark.sunrise, dark.sunset);
    }

    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(48))]

        /// SR-103: every civil date in a polar day or night gets its own
        /// day, 23 to 25 hours long, at any latitude from 66.6° to 89.9°,
        /// either hemisphere, any longitude, through either season and
        /// most often at its edges.
        #[test]
        fn every_polar_date_is_its_own_day(
            latitude in 66.6_f64..=89.9,
            south in proptest::bool::ANY,
            longitude in -180.0_f64..180.0,
            year in 1900_i32..2100,
            winter in proptest::bool::ANY,
            offset in -120_i64..=120,
        ) {
            let text = SuryaSiddhanta::text();
            #[allow(clippy::cast_possible_truncation, reason = "a whole hour within ±12")]
            let hours = (longitude / 15.0).round() as i32;
            let clock = UtcOffset::literal(hours, 0, 0);
            let place = Place::new(
                Latitude::literal(if south { -latitude } else { latitude }),
                Longitude::literal(longitude),
                Altitude::literal(0.0),
            );
            let solstice = gregorian::fixed_from_gregorian(year, if winter { 12 } else { 6 }, 21);
            // Walk from the offset back toward the solstice, which is
            // polar at these latitudes, to the first polar date.
            let step = if offset > 0 { -1 } else { 1 };
            let mut day = solstice.plus_days(offset);
            while civil_day_light(&text, &clock, &place, day).unwrap().arc().is_some() {
                day = day.plus_days(step);
            }
            assert_its_own_day(&text, &clock, &place, day);
        }
    }
}
