//! The Bangladeshi national calendar, Bengali era (Bangabda): the revised
//! Bengali calendar Bangla Academy keeps, in force from 1 Kartik 1426, 17
//! October 2019. Pure arithmetic on the Gregorian calendar, integer only.
//!
//! A Bengali year begins on 14 April, 593 Gregorian years after the year it
//! is numbered from. Boishakh to Ashwin have 31 days, Kartik to Magh 30,
//! Falgun 29, or 30 when the Gregorian year it falls in is a leap year,
//! and Choitro 30; so every month begins on the same Gregorian date each
//! year, Falgun on 14 February and Poush on 16 December. A date before the
//! revision is reckoned by it, as the Gregorian calendar is proleptic: the
//! 1987 rules it replaced gave Ashwin 30 days and Falgun 30 or 31, and
//! their dates are not these (`calendars/bangladeshi.md`). Years are
//! astronomical, as the Gregorian years they follow.

use teistro_core::catalogue::{Calendar, Era};
use teistro_core::error::Error;

use crate::date::{
    CalendarCapabilities, CalendarDate, CalendarSystem, check_day, check_fixed, out_of_range,
};
use crate::fixed::FixedDay;
use crate::gregorian::{self, YEAR_RANGE, fixed_from_gregorian};

/// The Bangladeshi national calendar.
///
/// ```
/// use teistro_calendar::bangladeshi::Bangladeshi;
/// use teistro_calendar::{CalendarSystem, Gregorian};
///
/// // 21 February is 8 Falgun, and 16 December 1 Poush, every year.
/// for year in [2024, 2025] {
///     let ekushey = Bangladeshi.date_of(Gregorian.to_fixed_ymd(year, 2, 21)?)?;
///     assert_eq!((ekushey.month, ekushey.day), (11, 8));
///     let victory = Bangladeshi.date_of(Gregorian.to_fixed_ymd(year, 12, 16)?)?;
///     assert_eq!((victory.month, victory.day), (9, 1));
/// }
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Bangladeshi;

/// How many years the Gregorian year a Bengali year begins in is ahead of
/// it.
pub const OFFSET: i32 = 593;

/// The Bengali years converted: every one that begins and ends inside the
/// Gregorian calendar's own years.
pub const BENGALI_RANGE: (i32, i32) = (YEAR_RANGE.0 - OFFSET, YEAR_RANGE.1 - OFFSET - 1);

/// Whether a Bengali year is a leap year: when the Gregorian year its
/// Falgun falls in, the one after the year it begins in, is one.
///
/// ```
/// use teistro_calendar::bangladeshi::is_leap_year;
///
/// // 1430's Falgun fell in February 2024.
/// assert!(is_leap_year(1430) && !is_leap_year(1431));
/// ```
#[must_use]
pub const fn is_leap_year(year: i32) -> bool {
    gregorian::is_leap_year(year + OFFSET + 1)
}

/// The length of a month, or `None` for a month outside 1 to 12.
///
/// ```
/// use teistro_calendar::bangladeshi::month_length;
///
/// assert_eq!((month_length(1431, 6), month_length(1431, 7)), (Some(31), Some(30)));
/// assert_eq!((month_length(1430, 11), month_length(1431, 11)), (Some(30), Some(29)));
/// ```
#[must_use]
pub const fn month_length(year: i32, month: u8) -> Option<u8> {
    match month {
        1..=6 => Some(31),
        11 if is_leap_year(year) => Some(30),
        11 => Some(29),
        7..=10 | 12 => Some(30),
        _ => None,
    }
}

/// The fixed day of 1 Boishakh of a Bengali year.
///
/// ```
/// use teistro_calendar::gregorian::fixed_from_gregorian;
/// use teistro_calendar::bangladeshi::new_year;
///
/// assert_eq!(new_year(1431), fixed_from_gregorian(2024, 4, 14));
/// ```
#[must_use]
pub const fn new_year(year: i32) -> FixedDay {
    fixed_from_gregorian(year + OFFSET, 4, 14)
}

/// The days of a Bengali year before a month begins; the month is not
/// validated.
const fn days_before(year: i32, month: u8) -> i64 {
    let m = month as i64;
    match m {
        ..=1 => 0,
        2..=7 => 31 * (m - 1),
        8..=11 => 31 * 6 + 30 * (m - 7),
        _ => 31 * 6 + 30 * 4 + if is_leap_year(year) { 30 } else { 29 },
    }
}

/// The fixed day of a Bengali date; the date is not validated.
///
/// ```
/// use teistro_calendar::gregorian::fixed_from_gregorian;
/// use teistro_calendar::bangladeshi::fixed_from_bangladeshi;
///
/// assert_eq!(fixed_from_bangladeshi(1431, 9, 1), fixed_from_gregorian(2024, 12, 16));
/// ```
#[must_use]
pub const fn fixed_from_bangladeshi(year: i32, month: u8, day: u8) -> FixedDay {
    new_year(year).plus_days(days_before(year, month) + day as i64 - 1)
}

/// The Bengali date of a fixed day.
///
/// ```
/// use teistro_calendar::gregorian::fixed_from_gregorian;
/// use teistro_calendar::bangladeshi::bangladeshi_from_fixed;
///
/// assert_eq!(bangladeshi_from_fixed(fixed_from_gregorian(2025, 4, 13)), (1431, 12, 30));
/// ```
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a day of the year is below 366, so its month and day fit a u8"
)]
pub const fn bangladeshi_from_fixed(fixed: FixedDay) -> (i32, u8, u8) {
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
    if year < BENGALI_RANGE.0 || year > BENGALI_RANGE.1 {
        return Err(out_of_range(format!(
            "{} covers Bengali years {} to {}, not {year}",
            Calendar::Bangladeshi.key(),
            BENGALI_RANGE.0,
            BENGALI_RANGE.1
        ))
        .with_field("year"));
    }
    Ok(())
}

impl CalendarSystem for Bangladeshi {
    fn id(&self) -> Calendar {
        Calendar::Bangladeshi
    }

    fn capabilities(&self) -> CalendarCapabilities {
        CalendarCapabilities {
            range: (
                new_year(BENGALI_RANGE.0),
                new_year(BENGALI_RANGE.1 + 1).plus_days(-1),
            ),
            needs_place: false,
            needs_ephemeris: false,
            eras: &[Era::Bengali],
        }
    }

    fn date_of(&self, fixed: FixedDay) -> Result<CalendarDate, Error> {
        check_fixed(Calendar::Bangladeshi, self.capabilities().range, fixed)?;
        let (year, month, day) = bangladeshi_from_fixed(fixed);
        Ok(
            CalendarDate::defined(Calendar::Bangladeshi, year, month, day)
                .with_era(Era::Bengali, year),
        )
    }

    fn fixed_of(&self, date: &CalendarDate) -> Result<FixedDay, Error> {
        crate::date::check_no_leap(date)?;
        check_year(date.year)?;
        let length = month_length(date.year, date.month).unwrap_or(0);
        check_day(
            Calendar::Bangladeshi,
            date.year,
            date.month,
            date.day,
            12,
            length,
        )?;
        Ok(fixed_from_bangladeshi(date.year, date.month, date.day))
    }

    fn month_length(&self, year: i32, month: u8) -> Result<u8, Error> {
        check_year(year)?;
        let length = month_length(year, month).unwrap_or(0);
        check_day(Calendar::Bangladeshi, year, month, 1, 12, length)?;
        Ok(length)
    }

    fn is_leap(&self, year: i32) -> bool {
        is_leap_year(year)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;
    use crate::gregorian::gregorian_from_fixed;

    fn gregorian_of(year: i32, month: u8, day: u8) -> (i32, u8, u8) {
        gregorian_from_fixed(fixed_from_bangladeshi(year, month, day))
    }

    /// Each month's first day, as Bangla Academy's 2019 table sets it in
    /// every year: Boishakh 14 April to Choitro 15 March.
    const FIRSTS: [(u8, u8); 12] = [
        (4, 14),
        (5, 15),
        (6, 15),
        (7, 16),
        (8, 16),
        (9, 16),
        (10, 17),
        (11, 16),
        (12, 16),
        (1, 15),
        (2, 14),
        (3, 15),
    ];

    #[test]
    fn every_month_begins_on_its_gregorian_date_in_every_year() {
        for year in 1400..=1500 {
            for (month, &(on_month, on_day)) in (1..=12).zip(&FIRSTS) {
                let (_, m, d) = gregorian_of(year, month, 1);
                assert_eq!((m, d), (on_month, on_day), "{year}-{month}");
            }
        }
    }

    #[test]
    fn the_revisions_first_day_and_the_national_days() {
        // In force from 1 Kartik 1426, 17 October 2019.
        assert_eq!(gregorian_of(1426, 7, 1), (2019, 10, 17));
        // Pohela Boishakh 1433, 14 April 2026.
        assert_eq!(gregorian_of(1433, 1, 1), (2026, 4, 14));
        // Ekushey: 21 February is 8 Falgun; Victory day, 16 December, 1
        // Poush; Independence day, 26 March, 12 Choitro.
        assert_eq!(gregorian_of(1431, 11, 8), (2025, 2, 21));
        assert_eq!(gregorian_of(1432, 9, 1), (2025, 12, 16));
        assert_eq!(gregorian_of(1432, 12, 12), (2026, 3, 26));
        // Falgun 1430 runs through 29 February 2024, its 30th day.
        assert!(is_leap_year(1430) && !is_leap_year(1431));
        assert_eq!(month_length(1430, 11), Some(30));
        assert_eq!(gregorian_of(1430, 11, 30), (2024, 3, 14));
        assert_eq!(gregorian_of(1430, 12, 1), (2024, 3, 15));
        assert_eq!(gregorian_of(1431, 11, 29), (2025, 3, 14));
        let refused = Bangladeshi
            .fixed_of(&CalendarDate::defined(Calendar::Bangladeshi, 1431, 11, 30))
            .unwrap_err();
        assert!(
            refused.message.contains("days 1 to 29"),
            "{}",
            refused.message
        );
    }

    #[test]
    fn every_day_round_trips() {
        let (first, last) = Bangladeshi.capabilities().range;
        let mut previous = (BENGALI_RANGE.0 - 1, 12u8, 30u8);
        let mut fixed = first.get();
        while fixed <= last.get() {
            let (y, m, d) = bangladeshi_from_fixed(FixedDay::new(fixed));
            assert_eq!(fixed_from_bangladeshi(y, m, d).get(), fixed, "{y}-{m}-{d}");
            assert!((y, m, d) > previous, "{y}-{m}-{d} after {previous:?}");
            assert!(d >= 1 && d <= month_length(y, m).unwrap_or(0));
            previous = (y, m, d);
            fixed += 1;
        }
        assert_eq!(previous, (BENGALI_RANGE.1, 12, 30));
    }
}
