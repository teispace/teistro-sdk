//! The proleptic Julian calendar, astronomical years (year 0 exists),
//! integer only.

use teistro_core::catalogue::{Calendar, Era};
use teistro_core::error::Error;

use crate::date::{CalendarCapabilities, CalendarDate, CalendarSystem, check_day};
use crate::fixed::FixedDay;
use crate::gregorian::{YEAR_RANGE, check_year, common_month_length, era_of};

/// The proleptic Julian calendar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Julian;

/// Whether a year is a Julian leap year.
#[must_use]
pub const fn is_leap_year(year: i32) -> bool {
    year.rem_euclid(4) == 0
}

/// The length of a month, or `None` for a month outside 1 to 12.
#[must_use]
pub const fn month_length(year: i32, month: u8) -> Option<u8> {
    if month == 2 && is_leap_year(year) {
        return Some(29);
    }
    common_month_length(month)
}

/// The fixed day of a Julian date; the date is not validated. Julian
/// 1 January 1 CE is fixed day −1.
#[must_use]
pub const fn fixed_from_julian(year: i32, month: u8, day: u8) -> FixedDay {
    let y = year as i64 - 1;
    let m = month as i64;
    let correction = if m <= 2 {
        0
    } else if is_leap_year(year) {
        -1
    } else {
        -2
    };
    FixedDay::new(
        -2 + 365 * y + y.div_euclid(4) + (367 * m - 362).div_euclid(12) + correction + day as i64,
    )
}

/// The Julian date of a fixed day.
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "years are bounded by the fixed day range; the month and day are positive"
)]
pub const fn julian_from_fixed(fixed: FixedDay) -> (i32, u8, u8) {
    let approx = ((4 * (fixed.get() + 1) + 1464).div_euclid(1461)) as i32;
    let year = if fixed.get() < fixed_from_julian(approx, 1, 1).get() {
        approx - 1
    } else {
        approx
    };
    let prior_days = fixed.get() - fixed_from_julian(year, 1, 1).get();
    let correction = if fixed.get() < fixed_from_julian(year, 3, 1).get() {
        0
    } else if is_leap_year(year) {
        1
    } else {
        2
    };
    let month = ((12 * (prior_days + correction) + 373).div_euclid(367)) as u8;
    let day = (fixed.get() - fixed_from_julian(year, month, 1).get() + 1) as u8;
    (year, month, day)
}

impl CalendarSystem for Julian {
    fn id(&self) -> Calendar {
        Calendar::Julian
    }

    fn capabilities(&self) -> CalendarCapabilities {
        CalendarCapabilities {
            range: (
                fixed_from_julian(YEAR_RANGE.0, 1, 1),
                fixed_from_julian(YEAR_RANGE.1, 12, 31),
            ),
            needs_place: false,
            needs_ephemeris: false,
            eras: &[Era::CommonEra, Era::BeforeCommonEra],
        }
    }

    fn date_of(&self, fixed: FixedDay) -> Result<CalendarDate, Error> {
        let (year, month, day) = julian_from_fixed(fixed);
        check_year(Calendar::Julian, year)?;
        let (era, era_year) = era_of(year);
        Ok(CalendarDate::defined(Calendar::Julian, year, month, day).with_era(era, era_year))
    }

    fn fixed_of(&self, date: &CalendarDate) -> Result<FixedDay, Error> {
        check_year(Calendar::Julian, date.year)?;
        let length = month_length(date.year, date.month).unwrap_or(0);
        check_day(
            Calendar::Julian,
            date.year,
            date.month,
            date.day,
            12,
            length,
        )?;
        Ok(fixed_from_julian(date.year, date.month, date.day))
    }

    fn month_length(&self, year: i32, month: u8) -> Result<u8, Error> {
        check_year(Calendar::Julian, year)?;
        let length = month_length(year, month).unwrap_or(0);
        check_day(Calendar::Julian, year, month, 1, 12, length)?;
        Ok(length)
    }

    fn is_leap(&self, year: i32) -> bool {
        is_leap_year(year)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;
    use crate::gregorian::fixed_from_gregorian;

    #[test]
    fn known_dates_and_rules() {
        assert_eq!(fixed_from_julian(1, 1, 1), FixedDay::new(-1));
        // Julian 4 October 1582 is the day before Gregorian 15 October 1582.
        assert_eq!(
            fixed_from_julian(1582, 10, 4),
            fixed_from_gregorian(1582, 10, 15).plus_days(-1)
        );
        // The two calendars agree in the third century.
        assert_eq!(
            fixed_from_julian(200, 3, 1),
            fixed_from_gregorian(200, 3, 1)
        );
        assert_eq!(julian_from_fixed(FixedDay::new(-1)), (1, 1, 1));
        assert_eq!(julian_from_fixed(FixedDay::new(-2)), (0, 12, 31));
        assert!(is_leap_year(1900) && is_leap_year(0) && is_leap_year(-4) && !is_leap_year(-1));
        assert_eq!(Julian.month_length(1900, 2).unwrap(), 29);
        // Today's difference: thirteen days.
        assert_eq!(
            fixed_from_julian(2024, 1, 1),
            fixed_from_gregorian(2024, 1, 14)
        );
    }

    #[test]
    fn every_day_round_trips() {
        let mut fixed = fixed_from_julian(YEAR_RANGE.0, 1, 1).get();
        let last = fixed_from_julian(YEAR_RANGE.1, 12, 31).get();
        let mut previous = (YEAR_RANGE.0 - 1, 12u8, 31u8);
        while fixed <= last {
            let (y, m, d) = julian_from_fixed(FixedDay::new(fixed));
            assert_eq!(fixed_from_julian(y, m, d).get(), fixed, "{y}-{m}-{d}");
            assert!((y, m, d) > previous, "{y}-{m}-{d} after {previous:?}");
            assert!(d >= 1 && d <= month_length(y, m).unwrap_or(0));
            previous = (y, m, d);
            fixed += 1;
        }
    }
}
