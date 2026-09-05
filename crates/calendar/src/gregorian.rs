//! The proleptic Gregorian calendar over Reingold and Dershowitz's
//! arithmetic, astronomical years, integer only.

use teistro_core::catalogue::{Calendar, Era};
use teistro_core::error::Error;

use crate::date::{CalendarCapabilities, CalendarDate, CalendarSystem, check_day, out_of_range};
use crate::fixed::FixedDay;

/// The proleptic Gregorian calendar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Gregorian;

/// The supported years, a limit and not a truth of the arithmetic.
pub const YEAR_RANGE: (i32, i32) = (-9999, 9999);

/// The length of a month of a common year, shared with the Julian calendar.
pub(crate) const fn common_month_length(month: u8) -> Option<u8> {
    Some(match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => 28,
        _ => return None,
    })
}

/// Whether a year is a Gregorian leap year.
#[must_use]
pub const fn is_leap_year(year: i32) -> bool {
    year.rem_euclid(4) == 0 && (year.rem_euclid(100) != 0 || year.rem_euclid(400) == 0)
}

/// The length of a month, or `None` for a month outside 1 to 12.
#[must_use]
pub const fn month_length(year: i32, month: u8) -> Option<u8> {
    if month == 2 && is_leap_year(year) {
        return Some(29);
    }
    common_month_length(month)
}

/// The fixed day of a Gregorian date; the date is not validated.
#[must_use]
pub const fn fixed_from_gregorian(year: i32, month: u8, day: u8) -> FixedDay {
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
        365 * y + y.div_euclid(4) - y.div_euclid(100)
            + y.div_euclid(400)
            + (367 * m - 362).div_euclid(12)
            + correction
            + day as i64,
    )
}

/// The Gregorian year a fixed day falls in.
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    reason = "years are bounded by the fixed day range"
)]
pub const fn year_from_fixed(fixed: FixedDay) -> i32 {
    let d0 = fixed.get() - 1;
    let n400 = d0.div_euclid(146_097);
    let d1 = d0.rem_euclid(146_097);
    let n100 = d1.div_euclid(36_524);
    let d2 = d1.rem_euclid(36_524);
    let n4 = d2.div_euclid(1461);
    let d3 = d2.rem_euclid(1461);
    let n1 = d3.div_euclid(365);
    let year = 400 * n400 + 100 * n100 + 4 * n4 + n1;
    (if n100 == 4 || n1 == 4 { year } else { year + 1 }) as i32
}

/// The Gregorian date of a fixed day.
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a month below thirteen and a day below thirty-two; the month and day are positive"
)]
pub const fn gregorian_from_fixed(fixed: FixedDay) -> (i32, u8, u8) {
    let year = year_from_fixed(fixed);
    let prior_days = fixed.get() - fixed_from_gregorian(year, 1, 1).get();
    let correction = if fixed.get() < fixed_from_gregorian(year, 3, 1).get() {
        0
    } else if is_leap_year(year) {
        1
    } else {
        2
    };
    let month = ((12 * (prior_days + correction) + 373).div_euclid(367)) as u8;
    let day = (fixed.get() - fixed_from_gregorian(year, month, 1).get() + 1) as u8;
    (year, month, day)
}

/// The era view of an astronomical year.
#[must_use]
pub const fn era_of(year: i32) -> (Era, i32) {
    if year > 0 {
        (Era::CommonEra, year)
    } else {
        (Era::BeforeCommonEra, 1 - year)
    }
}

pub(crate) fn check_year(calendar: Calendar, year: i32) -> Result<(), Error> {
    if year < YEAR_RANGE.0 || year > YEAR_RANGE.1 {
        return Err(out_of_range(format!(
            "{} covers years {} to {}, not {year}",
            calendar.key(),
            YEAR_RANGE.0,
            YEAR_RANGE.1
        ))
        .with_field("year"));
    }
    Ok(())
}

impl CalendarSystem for Gregorian {
    fn id(&self) -> Calendar {
        Calendar::Gregorian
    }

    fn capabilities(&self) -> CalendarCapabilities {
        CalendarCapabilities {
            range: (
                fixed_from_gregorian(YEAR_RANGE.0, 1, 1),
                fixed_from_gregorian(YEAR_RANGE.1, 12, 31),
            ),
            needs_place: false,
            needs_ephemeris: false,
            eras: &[Era::CommonEra, Era::BeforeCommonEra],
        }
    }

    fn date_of(&self, fixed: FixedDay) -> Result<CalendarDate, Error> {
        let (year, month, day) = gregorian_from_fixed(fixed);
        check_year(Calendar::Gregorian, year)?;
        let (era, era_year) = era_of(year);
        Ok(CalendarDate::defined(Calendar::Gregorian, year, month, day).with_era(era, era_year))
    }

    fn fixed_of(&self, date: &CalendarDate) -> Result<FixedDay, Error> {
        check_year(Calendar::Gregorian, date.year)?;
        let length = month_length(date.year, date.month).unwrap_or(0);
        check_day(
            Calendar::Gregorian,
            date.year,
            date.month,
            date.day,
            12,
            length,
        )?;
        Ok(fixed_from_gregorian(date.year, date.month, date.day))
    }

    fn month_length(&self, year: i32, month: u8) -> Result<u8, Error> {
        check_year(Calendar::Gregorian, year)?;
        let length = month_length(year, month).unwrap_or(0);
        check_day(Calendar::Gregorian, year, month, 1, 12, length)?;
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

    #[test]
    fn known_dates_and_rules() {
        assert_eq!(fixed_from_gregorian(1, 1, 1), FixedDay::EPOCH);
        assert_eq!(fixed_from_gregorian(2000, 1, 1), FixedDay::new(730_120));
        assert_eq!(gregorian_from_fixed(FixedDay::new(730_120)), (2000, 1, 1));
        assert_eq!(fixed_from_gregorian(1582, 10, 15), FixedDay::new(577_736));
        assert_eq!(gregorian_from_fixed(FixedDay::new(0)), (0, 12, 31));
        assert!(
            is_leap_year(2000) && !is_leap_year(1900) && !is_leap_year(2100) && is_leap_year(2024)
        );
        assert!(is_leap_year(0) && is_leap_year(-4) && !is_leap_year(-100) && is_leap_year(-400));
        assert_eq!(month_length(2024, 2), Some(29));
        assert_eq!(month_length(2023, 2), Some(28));
        assert_eq!(month_length(2023, 13), None);
        assert_eq!(era_of(0), (Era::BeforeCommonEra, 1));
        assert_eq!(era_of(-43), (Era::BeforeCommonEra, 44));
        let date = Gregorian.date_of(FixedDay::new(730_120)).unwrap();
        assert_eq!(date.era.map(|e| e.year), Some(2000));
        assert!(
            Gregorian
                .to_fixed_ymd(2023, 2, 29)
                .unwrap_err()
                .message
                .contains("days 1 to 28")
        );
        assert!(Gregorian.to_fixed_ymd(10_000, 1, 1).is_err());
    }

    #[test]
    fn every_day_round_trips() {
        let mut fixed = fixed_from_gregorian(YEAR_RANGE.0, 1, 1).get();
        let last = fixed_from_gregorian(YEAR_RANGE.1, 12, 31).get();
        let mut previous = (YEAR_RANGE.0 - 1, 12u8, 31u8);
        while fixed <= last {
            let (y, m, d) = gregorian_from_fixed(FixedDay::new(fixed));
            assert_eq!(fixed_from_gregorian(y, m, d).get(), fixed, "{y}-{m}-{d}");
            assert!((y, m, d) > previous, "{y}-{m}-{d} after {previous:?}");
            assert!(d >= 1 && d <= month_length(y, m).unwrap_or(0));
            previous = (y, m, d);
            fixed += 1;
        }
    }
}
