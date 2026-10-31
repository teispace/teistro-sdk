//! The sunrise-anchored local day at a place: sunrise, sunset and the
//! next sunrise from a solar model, with the polar policies
//! (`docs/03-design/time-and-timezone.md`, §3.3 and §4).

use core::fmt;

use teistro_calendar::solar::{DayArc, DayLight, SolarModel, civil_day_light};
use teistro_calendar::{CalendarDate, CalendarSystem, FixedDay};
use teistro_core::catalogue::Vara;
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Place, Utc};
use teistro_core::settings::{Air, PolarDayPolicy, SunriseConvention};
use teistro_core::time::LocalClock;

/// How far a nearest-event search looks, in days: past half a year the
/// other polar season has begun.
const NEAREST_SEARCH_DAYS: i64 = 190;

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

/// The first day after `day` with a sunrise.
fn first_arc_after(
    model: &dyn SolarModel,
    clock: &dyn LocalClock,
    place: &Place,
    day: FixedDay,
) -> Result<Option<(FixedDay, teistro_calendar::solar::DayArc)>, Error> {
    for distance in 1..=NEAREST_SEARCH_DAYS {
        let candidate = day.plus_days(distance);
        if let DayLight::Arc(arc) = civil_day_light(model, clock, place, candidate)? {
            return Ok(Some((candidate, arc)));
        }
    }
    Ok(None)
}

/// The next sunrise after a day whose following day has none, under the
/// policy: the first sunrise the season allows, or the next civil
/// midnight.
fn next_sunrise_after(
    model: &dyn SolarModel,
    clock: &dyn LocalClock,
    place: &Place,
    day: FixedDay,
    policy: PolarDayPolicy,
    kind: PolarKind,
) -> Result<JulianDay<Utc>, Error> {
    match policy {
        PolarDayPolicy::NearestEvent => first_arc_after(model, clock, place, day)?
            .map(|(_, arc)| arc.sunrise)
            .ok_or_else(|| no_sunrise_within(day, place)),
        _ => synthesised(model, clock, place, day.plus_days(1), kind, policy)
            .map(|bounds| bounds.sunrise),
    }
}

fn no_sunrise_within(day: FixedDay, place: &Place) -> Error {
    Error::unsupported(format!(
        "no sunrise within {NEAREST_SEARCH_DAYS} days of {day} at {place}"
    ))
    .with_field("day.polar_day_policy")
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
/// The model's refusal; a day with no arc under `UNDEFINED`, or with no
/// sunrise within the policy's reach, `UNSUPPORTED` on
/// `day.polar_day_policy`.
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

/// The nearest day to `day` with a sunrise, searching both ways.
fn nearest_arc(
    model: &dyn SolarModel,
    clock: &dyn LocalClock,
    place: &Place,
    day: FixedDay,
) -> Result<Option<(FixedDay, teistro_calendar::solar::DayArc)>, Error> {
    for distance in 1..=NEAREST_SEARCH_DAYS {
        for candidate in [day.plus_days(distance), day.plus_days(-distance)] {
            if let DayLight::Arc(arc) = civil_day_light(model, clock, place, candidate)? {
                return Ok(Some((candidate, arc)));
            }
        }
    }
    Ok(None)
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
    let tomorrow = civil_day_light(model, clock, place, day.plus_days(1))?;
    carried.0 = Some((day.plus_days(1), *place, tomorrow));
    local_day_lit(model, clock, place, (date, day), policy, (today, tomorrow))
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
///     civil_day_light(&text, &clock, &kathmandu, fixed.plus_days(1))?,
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
    (today, tomorrow): (DayLight, DayLight),
) -> Result<LocalDay, Error> {
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
            let (found_day, arc) = nearest_arc(model, clock, place, day)?
                .ok_or_else(|| no_sunrise_within(day, place))?;
            let next_sunrise = first_arc_after(model, clock, place, found_day)?
                .map_or(arc.sunset, |(_, next)| next.sunrise);
            Ok(Bounds {
                sunrise: arc.sunrise,
                sunset: arc.sunset,
                next_sunrise,
            })
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
        // Day 150 is nearer to day 99 (51 away) than to day 200 (50 away)?
        // No: 200 is 50 away, so the next season's first sunrise answers.
        let (found, _) = FixedDay::from_jd(nearest.sunrise);
        assert_eq!(found, FixedDay::new(200));
        // The last day before the night borrows its next sunrise.
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
        assert_eq!(FixedDay::from_jd(edge.next_sunrise).0, FixedDay::new(200));
        let (y, m, d) = gregorian::gregorian_from_fixed(FixedDay::new(150));
        assert_eq!((y, m, d), (1, 5, 30));
    }
}
