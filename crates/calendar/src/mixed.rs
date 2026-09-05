//! The mixed civil calendar: Julian until a transition day, Gregorian
//! from it, with the days skipped at the transition refused.

use teistro_core::catalogue::{Calendar, Era};
use teistro_core::error::Error;

use crate::date::{CalendarCapabilities, CalendarDate, CalendarSystem, nonexistent};
use crate::fixed::FixedDay;
use crate::gregorian::{Gregorian, check_year, fixed_from_gregorian};
use crate::julian::Julian;

/// A named transition: the first Gregorian day and the last Julian day.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Transition {
    /// The key (`GREGORIAN_1582`).
    pub key: &'static str,
    /// The first day reckoned in the Gregorian calendar.
    pub first_gregorian: FixedDay,
    /// The source of the date.
    pub source: &'static str,
}

/// The papal reform: Julian 4 October 1582 is followed by Gregorian
/// 15 October 1582.
pub const GREGORIAN_1582: Mixed = Mixed {
    transition: Transition {
        key: "GREGORIAN_1582",
        first_gregorian: fixed_from_gregorian(1582, 10, 15),
        source: "Inter gravissimas, 1582",
    },
};

/// Britain and its colonies: Julian 2 September 1752 is followed by
/// Gregorian 14 September 1752.
pub const BRITAIN_1752: Mixed = Mixed {
    transition: Transition {
        key: "BRITAIN_1752",
        first_gregorian: fixed_from_gregorian(1752, 9, 14),
        source: "Calendar (New Style) Act 1750",
    },
};

/// Russia: Julian 31 January 1918 is followed by Gregorian 14 February 1918.
pub const RUSSIA_1918: Mixed = Mixed {
    transition: Transition {
        key: "RUSSIA_1918",
        first_gregorian: fixed_from_gregorian(1918, 2, 14),
        source: "Sovnarkom decree of 24 January 1918",
    },
};

/// The mixed calendar with one transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mixed {
    /// The transition.
    pub transition: Transition,
}

impl Mixed {
    /// The shipped transitions by key.
    #[must_use]
    pub fn named(key: &str) -> Option<Mixed> {
        match key {
            "GREGORIAN_1582" => Some(GREGORIAN_1582),
            "BRITAIN_1752" => Some(BRITAIN_1752),
            "RUSSIA_1918" => Some(RUSSIA_1918),
            _ => None,
        }
    }

    fn is_gregorian(&self, fixed: FixedDay) -> bool {
        fixed >= self.transition.first_gregorian
    }
}

impl CalendarSystem for Mixed {
    fn id(&self) -> Calendar {
        Calendar::Mixed
    }

    fn capabilities(&self) -> CalendarCapabilities {
        CalendarCapabilities {
            range: (
                Julian.capabilities().range.0,
                Gregorian.capabilities().range.1,
            ),
            needs_place: false,
            needs_ephemeris: false,
            eras: &[Era::CommonEra, Era::BeforeCommonEra],
        }
    }

    fn date_of(&self, fixed: FixedDay) -> Result<CalendarDate, Error> {
        let mut date = if self.is_gregorian(fixed) {
            Gregorian.date_of(fixed)?
        } else {
            Julian.date_of(fixed)?
        };
        date.calendar = Calendar::Mixed;
        Ok(date)
    }

    fn fixed_of(&self, date: &CalendarDate) -> Result<FixedDay, Error> {
        check_year(Calendar::Mixed, date.year)?;
        let as_gregorian =
            CalendarDate::defined(Calendar::Gregorian, date.year, date.month, date.day);
        let as_julian = CalendarDate::defined(Calendar::Julian, date.year, date.month, date.day);
        let gregorian = Gregorian.fixed_of(&as_gregorian);
        if let Ok(fixed) = gregorian {
            if self.is_gregorian(fixed) {
                return Ok(fixed);
            }
        }
        let julian = Julian.fixed_of(&as_julian)?;
        if !self.is_gregorian(julian) {
            return Ok(julian);
        }
        let last_julian = Julian.date_of(self.transition.first_gregorian.plus_days(-1))?;
        let first_gregorian = Gregorian.date_of(self.transition.first_gregorian)?;
        Err(nonexistent(format!(
            "{}-{:02}-{:02} falls in the {} transition: {}-{:02}-{:02} is followed by {}-{:02}-{:02}",
            date.year,
            date.month,
            date.day,
            self.transition.key,
            last_julian.year,
            last_julian.month,
            last_julian.day,
            first_gregorian.year,
            first_gregorian.month,
            first_gregorian.day
        ))
        .with_field("day"))
    }

    fn month_length(&self, year: i32, month: u8) -> Result<u8, Error> {
        let gregorian_start = Gregorian.to_fixed_ymd(year, month, 1)?;
        if self.is_gregorian(gregorian_start) {
            return Gregorian.month_length(year, month);
        }
        let julian_length = Julian.month_length(year, month)?;
        let julian_end = Julian.to_fixed_ymd(year, month, julian_length)?;
        if !self.is_gregorian(julian_end) {
            return Ok(julian_length);
        }
        // The transition month: the Julian days before the transition plus
        // the Gregorian days from it.
        let julian_start = Julian.to_fixed_ymd(year, month, 1)?;
        let gregorian_end =
            Gregorian.to_fixed_ymd(year, month, Gregorian.month_length(year, month)?)?;
        let julian_days = julian_start.days_until(self.transition.first_gregorian);
        let gregorian_days = self.transition.first_gregorian.days_until(gregorian_end) + 1;
        Ok(u8::try_from(julian_days + gregorian_days).unwrap_or(u8::MAX))
    }

    fn is_leap(&self, year: i32) -> bool {
        if self.is_gregorian(fixed_from_gregorian(year, 3, 1)) {
            Gregorian.is_leap(year)
        } else {
            Julian.is_leap(year)
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    #[test]
    fn the_papal_transition() {
        let before = GREGORIAN_1582.to_fixed_ymd(1582, 10, 4).unwrap();
        let after = GREGORIAN_1582.to_fixed_ymd(1582, 10, 15).unwrap();
        assert_eq!(before.plus_days(1), after);
        let gap = GREGORIAN_1582.to_fixed_ymd(1582, 10, 10).unwrap_err();
        assert!(gap.message.contains("transition"), "{gap}");
        assert_eq!(GREGORIAN_1582.date_of(before).unwrap().day, 4);
        assert_eq!(GREGORIAN_1582.date_of(after).unwrap().day, 15);
        assert!(GREGORIAN_1582.is_leap(1500) && !GREGORIAN_1582.is_leap(1700));
        assert_eq!(GREGORIAN_1582.month_length(1582, 10).unwrap(), 21);
        assert_eq!(GREGORIAN_1582.month_length(1582, 9).unwrap(), 30);
        assert_eq!(GREGORIAN_1582.month_length(1583, 2).unwrap(), 28);
        assert_eq!(
            BRITAIN_1752.to_fixed_ymd(1752, 9, 2).unwrap().plus_days(1),
            BRITAIN_1752.to_fixed_ymd(1752, 9, 14).unwrap()
        );
        assert_eq!(BRITAIN_1752.month_length(1752, 9).unwrap(), 19);
        assert!(RUSSIA_1918.to_fixed_ymd(1918, 2, 5).is_err());
        assert_eq!(Mixed::named("BRITAIN_1752"), Some(BRITAIN_1752));
    }

    #[test]
    fn every_day_round_trips_and_is_monotonic() {
        for calendar in [GREGORIAN_1582, BRITAIN_1752, RUSSIA_1918] {
            let (first, last) = calendar.capabilities().range;
            let mut fixed = first.get();
            let mut previous: Option<(i32, u8, u8)> = None;
            while fixed <= last.get() {
                let f = FixedDay::new(fixed);
                let date = calendar.date_of(f).unwrap();
                assert_eq!(calendar.fixed_of(&date).unwrap(), f, "{date}");
                let key = (date.year, date.month, date.day);
                if let Some(p) = previous {
                    assert!(key > p, "{key:?} after {p:?}");
                }
                previous = Some(key);
                fixed += 1;
            }
        }
    }
}
