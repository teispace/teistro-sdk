//! The day a chart belongs to, which is not its civil date.
//!
//! A panchanga day runs from sunrise to sunrise. An instant before the
//! civil date's sunrise belongs to the day that began the morning before,
//! and the vara, the hora sequence, the ishtakaal and the sunrise that
//! anchors the lagna all move back a day with it.
//!
//! The corpus was recorded to prove this. Chart `c001` is a birth at
//! 05:30 in Kathmandu on 14 April 1990, where sunrise is 05:30:44: its
//! recorded arc runs from the previous evening's sunset, `is_day_birth`
//! is false, and `lagna_sunrise_jd` is the previous morning's sunrise. An
//! implementation that takes the civil date and looks up its sunrise is a
//! day out for every instant between midnight and sunrise — a quarter of
//! the clock — and every one of those charts looks plausible.
//!
//! `time::local_day` answers "what is the arc of *this date*". This
//! module answers the inverse, "what arc holds *this instant*", which is
//! the question a chart actually asks.

use teistro_calendar::CalendarSystem;
use teistro_calendar::solar::SolarModel;
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Place, Utc};
use teistro_core::settings::PolarDayPolicy;
use teistro_core::time::LocalClock;
use teistro_time::local_day::{LocalDay, local_day};

/// Which part of a day's arc holds an instant.
///
/// Two members and not three. "Pre-sunrise" is not a third part of a day:
/// it is the night of the day before, and [`ChartDay`] has already moved
/// to that day by the time this is answered. Naming it so stops "does
/// pre-sunrise count as night?" from being asked once per module.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DayPart {
    /// Between the day's sunrise and its sunset.
    Daylight,
    /// Between the day's sunset and the next sunrise. A birth in the
    /// small hours is here, in the night of the day before.
    Night,
}

impl DayPart {
    /// Whether this is the daylight part, which is what "day birth"
    /// means in the texts and in every recorded fixture.
    #[must_use]
    pub const fn is_daylight(self) -> bool {
        matches!(self, DayPart::Daylight)
    }
}

/// The day a chart belongs to, and where in it the chart's instant falls.
///
/// [`LocalDay`] already carries the arc, the vara, the polar state and
/// the convention the arc was reckoned by; what this adds is the two
/// things a day cannot know about an instant it does not have.
#[derive(Clone, Debug, PartialEq)]
pub struct ChartDay {
    /// The day, which may be the civil date before the instant's.
    pub day: LocalDay,
    /// Which part of its arc holds the instant.
    pub part: DayPart,
    /// How far through that part the instant is: 0 at its start and
    /// approaching 1 at its end.
    pub elapsed: f64,
}

impl ChartDay {
    /// The sunrise that anchors the lagna: the one that opened this day,
    /// which for an instant before the civil date's sunrise is the
    /// previous morning's.
    #[must_use]
    pub const fn lagna_sunrise(&self) -> JulianDay<Utc> {
        self.day.sunrise
    }

    /// The bounds of the part holding the instant, as the recorded
    /// fixtures report them: sunrise to sunset by day, sunset to the next
    /// sunrise by night.
    #[must_use]
    pub const fn part_bounds(&self) -> (JulianDay<Utc>, JulianDay<Utc>) {
        match self.part {
            DayPart::Daylight => (self.day.sunrise, self.day.sunset),
            DayPart::Night => (self.day.sunset, self.day.next_sunrise),
        }
    }

    /// The length of the part holding the instant, in days.
    #[must_use]
    pub fn part_days(&self) -> f64 {
        let (from, to) = self.part_bounds();
        to.get() - from.get()
    }
}

/// The day an instant belongs to at a place.
///
/// The civil date is where the search starts and not where it ends: if
/// the instant precedes that date's sunrise it belongs to the day before,
/// and that day's arc is the one returned.
///
/// # Errors
///
/// Whatever `time::local_day` refuses — a date the calendar does not
/// have, the solar model's own refusal, or a polar day under the
/// `UNDEFINED` policy, which names the policies that synthesise one.
pub fn chart_day(
    model: &dyn SolarModel,
    calendar: &dyn CalendarSystem,
    clock: &dyn LocalClock,
    place: &Place,
    instant: JulianDay<Utc>,
    policy: PolarDayPolicy,
) -> Result<ChartDay, Error> {
    let civil = teistro_calendar::solar::rule::local_day(clock, instant).0;
    let date = calendar.date_of(civil)?;
    let today = local_day(model, calendar, clock, place, &date, policy)?;

    // Before this morning's sunrise the instant belongs to yesterday's
    // day, whose night is still running.
    if instant.get() < today.sunrise.get() {
        let yesterday_date = calendar.date_of(civil.plus_days(-1))?;
        let yesterday = local_day(model, calendar, clock, place, &yesterday_date, policy)?;
        return Ok(within(yesterday, instant));
    }
    Ok(within(today, instant))
}

/// Where an instant falls in a day whose arc already holds it.
fn within(day: LocalDay, instant: JulianDay<Utc>) -> ChartDay {
    let at = instant.get();
    let (part, from, to) = if at < day.sunset.get() {
        (DayPart::Daylight, day.sunrise.get(), day.sunset.get())
    } else {
        (DayPart::Night, day.sunset.get(), day.next_sunrise.get())
    };
    let span = to - from;
    // A polar policy may synthesise a part of zero length; a fraction of
    // nothing is the start of it rather than a division by zero.
    let elapsed = if span > 0.0 { (at - from) / span } else { 0.0 };
    ChartDay { day, part, elapsed }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::{DayPart, chart_day};
    use teistro_calendar::Gregorian;
    use teistro_calendar::fixed::FixedDay;
    use teistro_calendar::solar::{DayLight, SolarModel};
    use teistro_core::error::Error;
    use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
    use teistro_core::settings::{PolarDayPolicy, Sunrise, SunriseConvention};
    use teistro_core::time::UtcOffset;

    /// A Sun that rises at 06:00 UTC and sets at 18:00 UTC every day, so
    /// the arcs are exact and the assertions are about the day chosen
    /// rather than about the solver.
    #[derive(Debug)]
    struct SixToSix;

    impl SolarModel for SixToSix {
        fn sidereal_sun_deg(&self, _jd_ut: f64) -> Result<f64, Error> {
            Ok(0.0)
        }

        fn day_light(&self, day: FixedDay, _place: &Place) -> Result<DayLight, Error> {
            // A fixed day's noon is its Julian day at .5; 06:00 is .25
            // before it and 18:00 .25 after.
            let midnight = FixedDay::JD_EPOCH + f64::from(i32::try_from(day.get()).unwrap_or(0));
            Ok(DayLight::Arc(teistro_calendar::solar::DayArc {
                sunrise: JulianDay::<Utc>::literal(midnight + 0.25),
                sunset: JulianDay::<Utc>::literal(midnight + 0.75),
            }))
        }

        fn describe(&self) -> String {
            String::from("six-to-six")
        }

        fn convention(&self) -> SunriseConvention {
            Sunrise::CentreNoRefraction.into()
        }
    }

    fn place() -> Place {
        Place::new(
            Latitude::literal(0.0),
            Longitude::literal(0.0),
            Altitude::literal(0.0),
        )
    }

    /// The instant `hours` after the midnight opening fixed day `day`.
    fn at(day: i64, hours: f64) -> JulianDay<Utc> {
        JulianDay::<Utc>::literal(
            FixedDay::JD_EPOCH + f64::from(i32::try_from(day).unwrap_or(0)) + hours / 24.0,
        )
    }

    fn day_of(instant: JulianDay<Utc>) -> super::ChartDay {
        chart_day(
            &SixToSix,
            &Gregorian,
            &UtcOffset::UTC,
            &place(),
            instant,
            PolarDayPolicy::Undefined,
        )
        .unwrap_or_else(|e| panic!("{e}"))
    }

    #[test]
    fn a_birth_before_sunrise_belongs_to_the_day_before() {
        let civil = 730_000_i64;
        // 03:00 — the small hours, before the 06:00 sunrise.
        let small_hours = day_of(at(civil, 3.0));
        assert_eq!(small_hours.part, DayPart::Night);
        assert!(!small_hours.part.is_daylight());
        // The day it belongs to opened the previous morning, and its
        // sunrise is the one that anchors the lagna.
        let expected = at(civil - 1, 6.0);
        assert!(
            (small_hours.lagna_sunrise().get() - expected.get()).abs() < 1e-9,
            "the anchor is yesterday's sunrise"
        );
        // Three quarters through the twelve-hour night: 18:00 to 06:00,
        // and 03:00 is nine hours in.
        assert!(
            (small_hours.elapsed - 0.75).abs() < 1e-9,
            "{}",
            small_hours.elapsed
        );
    }

    #[test]
    fn a_birth_after_sunrise_belongs_to_its_own_day() {
        let civil = 730_000_i64;
        let noon = day_of(at(civil, 12.0));
        assert_eq!(noon.part, DayPart::Daylight);
        assert!((noon.lagna_sunrise().get() - at(civil, 6.0).get()).abs() < 1e-9);
        assert!(
            (noon.elapsed - 0.5).abs() < 1e-9,
            "half way from 06:00 to 18:00"
        );
        assert!((noon.part_days() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn the_boundary_instants_fall_on_the_right_side() {
        let civil = 730_000_i64;
        // Sunrise itself opens its own day's daylight.
        let sunrise = day_of(at(civil, 6.0));
        assert_eq!(sunrise.part, DayPart::Daylight);
        assert!(sunrise.elapsed.abs() < 1e-12);
        assert!((sunrise.lagna_sunrise().get() - at(civil, 6.0).get()).abs() < 1e-9);

        // A moment before it is still the night of the day before.
        let before = day_of(at(civil, 6.0 - 1e-6));
        assert_eq!(before.part, DayPart::Night);
        assert!((before.lagna_sunrise().get() - at(civil - 1, 6.0).get()).abs() < 1e-9);

        // Sunset opens the night of its own day.
        let sunset = day_of(at(civil, 18.0));
        assert_eq!(sunset.part, DayPart::Night);
        assert!(sunset.elapsed.abs() < 1e-12);
        assert!((sunset.lagna_sunrise().get() - at(civil, 6.0).get()).abs() < 1e-9);
    }

    #[test]
    fn the_night_runs_past_midnight_into_the_next_civil_date() {
        let civil = 730_000_i64;
        // 23:00 on the civil date, and 01:00 on the next: the same night
        // of the same panchanga day.
        let evening = day_of(at(civil, 23.0));
        let small_hours = day_of(at(civil + 1, 1.0));
        assert_eq!(evening.part, DayPart::Night);
        assert_eq!(small_hours.part, DayPart::Night);
        assert_eq!(
            evening.day.date, small_hours.day.date,
            "one night, one day, across the civil midnight"
        );
        assert!(
            small_hours.elapsed > evening.elapsed,
            "the night runs forward"
        );
    }

    #[test]
    fn the_parts_bounds_are_the_ones_the_fixtures_report() {
        let civil = 730_000_i64;
        let (from, to) = day_of(at(civil, 12.0)).part_bounds();
        assert!((from.get() - at(civil, 6.0).get()).abs() < 1e-9);
        assert!((to.get() - at(civil, 18.0).get()).abs() < 1e-9);

        let (from, to) = day_of(at(civil, 20.0)).part_bounds();
        assert!((from.get() - at(civil, 18.0).get()).abs() < 1e-9);
        assert!((to.get() - at(civil + 1, 6.0).get()).abs() < 1e-9);
    }
}
