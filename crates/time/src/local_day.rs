//! The sunrise-anchored local day at a place: sunrise, sunset and the
//! next sunrise from a solar model, with the polar policies
//! (`docs/03-design/time-and-timezone.md`, §3.3 and §4).

use core::fmt;

use teistro_calendar::solar::{DayLight, SolarModel};
use teistro_calendar::{CalendarDate, CalendarSystem, FixedDay};
use teistro_core::catalogue::Vara;
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Place, Utc};
use teistro_core::settings::{PolarDayPolicy, SunriseConvention};
use teistro_core::time::LocalClock;

/// How far a nearest-event search looks, in days: past half a year the
/// other polar season has begun.
const NEAREST_SEARCH_DAYS: i64 = 190;

/// Which polar state a day is in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PolarKind {
    /// The Sun does not set.
    Day,
    /// The Sun does not rise.
    Night,
}

/// Whether the day had a sunrise, and what was done when it had not.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
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
    #[must_use]
    pub fn daylight_days(&self) -> f64 {
        self.sunset.get() - self.sunrise.get()
    }

    /// The night, in days.
    #[must_use]
    pub fn night_days(&self) -> f64 {
        self.next_sunrise.get() - self.sunset.get()
    }

    /// Whether an instant lies in this day, from its sunrise to the next.
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

/// The first day after `day` with a sunrise.
fn first_arc_after(
    model: &dyn SolarModel,
    place: &Place,
    day: FixedDay,
) -> Result<Option<(FixedDay, teistro_calendar::solar::DayArc)>, Error> {
    for distance in 1..=NEAREST_SEARCH_DAYS {
        let candidate = day.plus_days(distance);
        if let DayLight::Arc(arc) = model.day_light(candidate, place)? {
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
        PolarDayPolicy::NearestEvent => first_arc_after(model, place, day)?
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

/// The bounds a policy gives a day: sunrise, sunset, next sunrise.
struct Bounds {
    sunrise: JulianDay<Utc>,
    sunset: JulianDay<Utc>,
    next_sunrise: JulianDay<Utc>,
}

/// The nearest day to `day` with a sunrise, searching both ways.
fn nearest_arc(
    model: &dyn SolarModel,
    place: &Place,
    day: FixedDay,
) -> Result<Option<(FixedDay, teistro_calendar::solar::DayArc)>, Error> {
    for distance in 1..=NEAREST_SEARCH_DAYS {
        for candidate in [day.plus_days(distance), day.plus_days(-distance)] {
            if let DayLight::Arc(arc) = model.day_light(candidate, place)? {
                return Ok(Some((candidate, arc)));
            }
        }
    }
    Ok(None)
}

/// The local day of a civil date at a place.
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
    let day = calendar.fixed_of(date)?;
    let today = model.day_light(day, place)?;
    let tomorrow = model.day_light(day.plus_days(1), place)?;
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
                next,
                DayState::Polar { kind, policy },
            )
        }
        (polar, _) => {
            let kind = polar_kind(polar);
            let bounds = synthesised(model, clock, place, day, kind, policy)?;
            (
                bounds.sunrise,
                bounds.sunset,
                bounds.next_sunrise,
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
            let midnight = |d: FixedDay| -> Result<JulianDay<Utc>, Error> {
                let jd = d.jd_at_midnight()?;
                Ok(JulianDay::try_new(jd.get() - clock.offset_at(jd).days())?)
            };
            let start = midnight(day)?;
            let end = midnight(day.plus_days(1))?;
            Ok(Bounds {
                sunrise: start,
                sunset: end,
                next_sunrise: end,
            })
        }
        PolarDayPolicy::NearestEvent => {
            let (found_day, arc) =
                nearest_arc(model, place, day)?.ok_or_else(|| no_sunrise_within(day, place))?;
            let next_sunrise = first_arc_after(model, place, found_day)?
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
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use teistro_calendar::solar::DayArc;
    use teistro_calendar::{Gregorian, gregorian};
    use teistro_core::catalogue::Calendar;
    use teistro_core::quantity::{Altitude, Latitude, Longitude};
    use teistro_core::time::UtcOffset;
    use teistro_siddhanta::SuryaSiddhanta;

    use super::*;
    use crate::zones;

    const KATHMANDU: Place = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );

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
