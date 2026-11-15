//! A polar season under `NEAREST_EVENT`: its two ends, and the days
//! between them (`03-design/time-and-timezone.md` §4, C1900).
//!
//! Each civil date in the season gets its own day. Its sunrise is the
//! real sunrise at either end of the season, carried across the season
//! in equal steps: an end is the nearest day with a real arc, or, where a
//! polar night meets a polar day with no arc between (near the pole),
//! the seam's mean midnight (a midnight sun begins) or mean noon (a polar
//! night begins). Every day of a season is then the same length, the
//! season's span over its days, which is within minutes of a day at any
//! latitude, and each day's next sunrise is the following day's sunrise,
//! to the bit. Its daylight is carried the same way, from the end arcs'
//! daylight, or a whole day (a midnight sun) or none (a polar night) at
//! a seam.

use teistro_calendar::FixedDay;
use teistro_calendar::solar::{
    DayLight, SolarModel, civil_day_light, local_mean_midnight, mean_time_day,
};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Place};
use teistro_core::time::LocalClock;

use super::{Bounds, PolarKind};

/// How far a season's ends are searched for, in days each way. A polar
/// season on Earth is shorter than a year (about 194 days at the pole
/// with refraction and the dip of a high horizon), so a search this long
/// finds both ends, and a season with neither end inside a year (a
/// test model's endless night) stands on its natural hour every day.
const REACH_DAYS: i64 = 366;

/// One end of a season: the day it stands on, that day's sunrise, and
/// its daylight as a fraction of a day.
#[derive(Clone, Copy, Debug)]
struct End {
    day: FixedDay,
    sunrise: f64,
    daylight: f64,
}

/// A season's ends as a date inside it finds them.
#[derive(Clone, Copy, Debug)]
enum Ends {
    /// Both ends: the days between are carried from one to the other.
    Both(End, End),
    /// One end only: the days keep its hour.
    One(End),
}

/// A polar season seen from a date inside it.
#[derive(Clone, Copy, Debug)]
pub(super) struct Season(Ends);

/// Days between two fixed days, as a float.
#[allow(
    clippy::cast_precision_loss,
    reason = "days within twice the reach, far below 2^52"
)]
fn days_between(from: FixedDay, to: FixedDay) -> f64 {
    (to.get() - from.get()) as f64
}

/// The seam's natural sunrise for the season it opens: the mean midnight
/// of its day under a midnight sun, when the Sun is lowest and nearest
/// rising, and the mean noon under a polar night, when it is highest.
fn seam(
    clock: &dyn LocalClock,
    place: &Place,
    day: FixedDay,
    opens: PolarKind,
) -> Result<f64, Error> {
    let midnight = local_mean_midnight(mean_time_day(clock, place, day), place)?.get();
    Ok(match opens {
        PolarKind::Day => midnight,
        PolarKind::Night => midnight + 0.5,
    })
}

/// The daylight a season stands on at a seam: the whole day under a
/// midnight sun, none under a polar night.
const fn natural_daylight(kind: PolarKind) -> f64 {
    match kind {
        PolarKind::Day => 1.0,
        PolarKind::Night => 0.0,
    }
}

impl Season {
    /// The season holding `day`, a polar day of `kind`.
    ///
    /// # Errors
    ///
    /// The model's refusal on a day searched.
    pub(super) fn of(
        model: &dyn SolarModel,
        clock: &dyn LocalClock,
        place: &Place,
        day: FixedDay,
        kind: PolarKind,
    ) -> Result<Self, Error> {
        let before = end(model, clock, place, (day, kind), -1)?;
        let after = end(model, clock, place, (day, kind), 1)?;
        let ends = match (before, after) {
            (Some(before), Some(after)) => Ends::Both(before, after),
            (Some(one), None) | (None, Some(one)) => Ends::One(one),
            (None, None) => Ends::One(End {
                day,
                sunrise: seam(clock, place, day, kind)?,
                daylight: natural_daylight(kind),
            }),
        };
        Ok(Self(ends))
    }

    /// The sunrise of a day of the season (or of its ends).
    pub(super) fn sunrise(self, day: FixedDay) -> f64 {
        match self.0 {
            Ends::Both(before, _) if day == before.day => before.sunrise,
            Ends::Both(_, after) if day == after.day => after.sunrise,
            Ends::Both(before, after) => {
                let share = days_between(before.day, day) / days_between(before.day, after.day);
                before.sunrise + (after.sunrise - before.sunrise) * share
            }
            Ends::One(end) => end.sunrise + days_between(end.day, day),
        }
    }

    /// The daylight of a day of the season, as a fraction of the day.
    pub(super) fn daylight(self, day: FixedDay) -> f64 {
        let daylight = match self.0 {
            Ends::Both(before, after) => {
                let share = days_between(before.day, day) / days_between(before.day, after.day);
                before.daylight + (after.daylight - before.daylight) * share
            }
            Ends::One(end) => end.daylight,
        };
        daylight.clamp(0.0, 1.0)
    }
}

/// The season's end on one side of `day` (`step` −1 or +1): the nearest
/// day with a real arc, or the seam where the other polar kind begins.
fn end(
    model: &dyn SolarModel,
    clock: &dyn LocalClock,
    place: &Place,
    (day, kind): (FixedDay, PolarKind),
    step: i64,
) -> Result<Option<End>, Error> {
    for distance in 1..=REACH_DAYS {
        let candidate = day.plus_days(step * distance);
        let other = match civil_day_light(model, clock, place, candidate)? {
            DayLight::Arc(arc) => {
                return Ok(Some(End {
                    day: candidate,
                    sunrise: arc.sunrise.get(),
                    daylight: arc.sunset.get() - arc.sunrise.get(),
                }));
            }
            DayLight::AlwaysUp => PolarKind::Day,
            DayLight::NeverUp => PolarKind::Night,
        };
        if other == kind {
            continue;
        }
        // The other polar kind with no arc between: the seam is the first
        // day of whichever season comes later, on the hour that season
        // opens on, so both seasons find the same instant.
        let (at, opens) = if step < 0 {
            (candidate.plus_days(1), kind)
        } else {
            (candidate, other)
        };
        return Ok(Some(End {
            day: at,
            sunrise: seam(clock, place, at, opens)?,
            daylight: natural_daylight(kind),
        }));
    }
    Ok(None)
}

/// A synthesised day's bounds from its season.
///
/// # Errors
///
/// An instant outside the Julian day range.
pub(super) fn bounds(season: Season, day: FixedDay) -> Result<Bounds, Error> {
    let sunrise = season.sunrise(day);
    let next = season.sunrise(day.plus_days(1));
    let sunset = sunrise + (next - sunrise) * season.daylight(day);
    Ok(Bounds {
        sunrise: JulianDay::try_new(sunrise)?,
        sunset: JulianDay::try_new(sunset)?,
        next_sunrise: JulianDay::try_new(next)?,
    })
}
