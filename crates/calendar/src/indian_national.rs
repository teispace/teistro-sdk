//! The Indian national calendar, Saka era: the civil calendar the
//! Calendar Reform Committee proposed (*Report of the Calendar Reform
//! Committee*, 1955, part C) and India adopted on 22 March 1957, 1 Chaitra
//! 1879. Pure arithmetic on the Gregorian calendar, integer only.
//!
//! A Saka year begins 78 Gregorian years after the year it is numbered
//! from: 1 Chaitra is 22 March, or 21 March when that Gregorian year is a
//! leap year, and the Saka year is a leap year exactly then. Chaitra has 30
//! days, 31 in a leap year; Vaishakha to Bhadra have 31; Ashvina to Phalguna
//! have 30. Years are astronomical, as the Gregorian years they follow.

use teistro_core::catalogue::{Calendar, Era};
use teistro_core::error::Error;

use crate::date::{
    CalendarCapabilities, CalendarDate, CalendarSystem, check_day, check_fixed, out_of_range,
};
use crate::fixed::FixedDay;
use crate::gregorian::{self, YEAR_RANGE, fixed_from_gregorian};

/// The Indian national calendar.
///
/// ```
/// use teistro_calendar::indian_national::IndianNational;
/// use teistro_calendar::{CalendarSystem, Gregorian};
///
/// // 1 January 2024 is 11 Pausha 1945 Saka.
/// let saka = IndianNational.date_of(Gregorian.to_fixed_ymd(2024, 1, 1)?)?;
/// assert_eq!((saka.year, saka.month, saka.day), (1945, 10, 11));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IndianNational;

/// How many years the Gregorian year a Saka year begins in is ahead of it.
pub const OFFSET: i32 = 78;

/// The Saka years converted: every one that begins and ends inside the
/// Gregorian calendar's own years.
pub const SAKA_RANGE: (i32, i32) = (YEAR_RANGE.0 - OFFSET, YEAR_RANGE.1 - OFFSET - 1);

/// Whether a Saka year is a leap year: when the Gregorian year it begins in
/// is one.
///
/// ```
/// use teistro_calendar::indian_national::is_leap_year;
///
/// // Saka 1946 began in 2024, a Gregorian leap year.
/// assert!(is_leap_year(1946) && !is_leap_year(1945));
/// ```
#[must_use]
pub const fn is_leap_year(year: i32) -> bool {
    gregorian::is_leap_year(year + OFFSET)
}

/// The length of a month, or `None` for a month outside 1 to 12.
///
/// ```
/// use teistro_calendar::indian_national::month_length;
///
/// assert_eq!(month_length(1946, 1), Some(31));
/// assert_eq!(month_length(1945, 1), Some(30));
/// assert_eq!((month_length(1946, 6), month_length(1946, 7)), (Some(31), Some(30)));
/// ```
#[must_use]
pub const fn month_length(year: i32, month: u8) -> Option<u8> {
    match month {
        1 if is_leap_year(year) => Some(31),
        1 | 7..=12 => Some(30),
        2..=6 => Some(31),
        _ => None,
    }
}

/// The fixed day of 1 Chaitra of a Saka year.
///
/// ```
/// use teistro_calendar::gregorian::fixed_from_gregorian;
/// use teistro_calendar::indian_national::new_year;
///
/// // 1 Chaitra is 21 March in a leap year and 22 March otherwise.
/// assert_eq!(new_year(1946), fixed_from_gregorian(2024, 3, 21));
/// assert_eq!(new_year(1947), fixed_from_gregorian(2025, 3, 22));
/// ```
#[must_use]
pub const fn new_year(year: i32) -> FixedDay {
    let gregorian = year + OFFSET;
    let day = if gregorian::is_leap_year(gregorian) {
        21
    } else {
        22
    };
    fixed_from_gregorian(gregorian, 3, day)
}

/// The days of a Saka year before a month begins; the month is not
/// validated.
const fn days_before(year: i32, month: u8) -> i64 {
    let m = month as i64;
    let chaitra = if is_leap_year(year) { 31 } else { 30 };
    match m {
        ..=1 => 0,
        2..=7 => chaitra + 31 * (m - 2),
        _ => chaitra + 31 * 5 + 30 * (m - 7),
    }
}

/// The fixed day of a Saka date; the date is not validated.
///
/// ```
/// use teistro_calendar::gregorian::fixed_from_gregorian;
/// use teistro_calendar::indian_national::fixed_from_indian_national;
///
/// assert_eq!(fixed_from_indian_national(1945, 10, 11), fixed_from_gregorian(2024, 1, 1));
/// ```
#[must_use]
pub const fn fixed_from_indian_national(year: i32, month: u8, day: u8) -> FixedDay {
    new_year(year).plus_days(days_before(year, month) + day as i64 - 1)
}

/// The Saka date of a fixed day.
///
/// ```
/// use teistro_calendar::gregorian::fixed_from_gregorian;
/// use teistro_calendar::indian_national::indian_national_from_fixed;
///
/// assert_eq!(indian_national_from_fixed(fixed_from_gregorian(2024, 3, 21)), (1946, 1, 1));
/// ```
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a day of the year is below 366, so its month and day fit a u8"
)]
pub const fn indian_national_from_fixed(fixed: FixedDay) -> (i32, u8, u8) {
    let guess = gregorian::year_from_fixed(fixed) - OFFSET;
    let year = if fixed.get() < new_year(guess).get() {
        guess - 1
    } else {
        guess
    };
    let into = fixed.get() - new_year(year).get();
    let mut month = 12;
    while days_before(year, month) > into {
        month -= 1;
    }
    let day = (into - days_before(year, month) + 1) as u8;
    (year, month, day)
}

fn check_year(year: i32) -> Result<(), Error> {
    if year < SAKA_RANGE.0 || year > SAKA_RANGE.1 {
        return Err(out_of_range(format!(
            "{} covers Saka years {} to {}, not {year}",
            Calendar::IndianNational.key(),
            SAKA_RANGE.0,
            SAKA_RANGE.1
        ))
        .with_field("year"));
    }
    Ok(())
}

impl CalendarSystem for IndianNational {
    fn id(&self) -> Calendar {
        Calendar::IndianNational
    }

    fn capabilities(&self) -> CalendarCapabilities {
        CalendarCapabilities {
            range: (
                new_year(SAKA_RANGE.0),
                new_year(SAKA_RANGE.1 + 1).plus_days(-1),
            ),
            needs_place: false,
            needs_ephemeris: false,
            eras: &[Era::Shaka],
        }
    }

    fn date_of(&self, fixed: FixedDay) -> Result<CalendarDate, Error> {
        check_fixed(Calendar::IndianNational, self.capabilities().range, fixed)?;
        let (year, month, day) = indian_national_from_fixed(fixed);
        Ok(
            CalendarDate::defined(Calendar::IndianNational, year, month, day)
                .with_era(Era::Shaka, year),
        )
    }

    fn fixed_of(&self, date: &CalendarDate) -> Result<FixedDay, Error> {
        crate::date::check_no_leap(date)?;
        check_year(date.year)?;
        let length = month_length(date.year, date.month).unwrap_or(0);
        check_day(
            Calendar::IndianNational,
            date.year,
            date.month,
            date.day,
            12,
            length,
        )?;
        Ok(fixed_from_indian_national(date.year, date.month, date.day))
    }

    fn month_length(&self, year: i32, month: u8) -> Result<u8, Error> {
        check_year(year)?;
        let length = month_length(year, month).unwrap_or(0);
        check_day(Calendar::IndianNational, year, month, 1, 12, length)?;
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
    use crate::gregorian::gregorian_from_fixed;

    fn gregorian_of(year: i32, month: u8, day: u8) -> (i32, u8, u8) {
        gregorian_from_fixed(fixed_from_indian_national(year, month, day))
    }

    #[test]
    fn the_committees_definition_and_published_dates() {
        // The calendar's first day: 1 Chaitra 1879, 22 March 1957.
        assert_eq!(gregorian_of(1879, 1, 1), (1957, 3, 22));
        // A leap year begins a day early: 1 Chaitra 1946, 21 March 2024.
        assert_eq!(gregorian_of(1946, 1, 1), (2024, 3, 21));
        assert!(is_leap_year(1946) && !is_leap_year(1947) && !is_leap_year(1822));
        assert_eq!(month_length(1946, 1), Some(31));
        // India's independence, Shravana 24, 1869.
        assert_eq!(gregorian_of(1869, 5, 24), (1947, 8, 15));
        // Each month's first day in a common year, as the committee's
        // table sets them: Vaishakha 21 April, Ashvina 23 September,
        // Phalguna 20 February.
        assert_eq!(gregorian_of(1947, 2, 1), (2025, 4, 21));
        assert_eq!(gregorian_of(1947, 7, 1), (2025, 9, 23));
        assert_eq!(gregorian_of(1947, 12, 1), (2026, 2, 20));
        // The year's last day is the day before the next 1 Chaitra.
        assert_eq!(gregorian_of(1947, 12, 30), (2026, 3, 21));
        assert_eq!(gregorian_of(1948, 1, 1), (2026, 3, 22));
        let refused = IndianNational
            .fixed_of(&CalendarDate::defined(
                Calendar::IndianNational,
                1947,
                1,
                31,
            ))
            .unwrap_err();
        assert!(
            refused.message.contains("days 1 to 30"),
            "{}",
            refused.message
        );
    }

    #[test]
    fn every_day_round_trips() {
        let (first, last) = IndianNational.capabilities().range;
        let mut previous = (SAKA_RANGE.0 - 1, 12u8, 30u8);
        let mut fixed = first.get();
        while fixed <= last.get() {
            let (y, m, d) = indian_national_from_fixed(FixedDay::new(fixed));
            assert_eq!(
                fixed_from_indian_national(y, m, d).get(),
                fixed,
                "{y}-{m}-{d}"
            );
            assert!((y, m, d) > previous, "{y}-{m}-{d} after {previous:?}");
            assert!(d >= 1 && d <= month_length(y, m).unwrap_or(0));
            previous = (y, m, d);
            fixed += 1;
        }
        assert_eq!(previous, (SAKA_RANGE.1, 12, 30));
    }
}
