//! The tabular Hijri calendar: the arithmetic Islamic calendar of the
//! civil epoch, integer only (Reingold and Dershowitz, *Calendrical
//! Calculations*, ch. 7; ICU's `islamic-civil`).
//!
//! It is a rule, not a sighting. Twelve months alternate 30 and 29 days,
//! Muharram first; in eleven years of every thirty (the 2nd, 5th, 7th, 10th,
//! 13th, 16th, 18th, 21st, 24th, 26th and 29th) Dhu al-Hijja has 30 days.
//! 1 Muharram 1 AH is Friday 16 July 622 in the Julian calendar. Countries
//! that begin a month on a sighted crescent, as Nepal and India date Eid,
//! can part from it by a day or two; that answer is published, not
//! computed, and comes as data. Years are astronomical: before 1 AH is
//! year 0.

use teistro_core::catalogue::{Calendar, Era};
use teistro_core::error::Error;

use crate::date::{
    CalendarCapabilities, CalendarDate, CalendarSystem, check_day, check_fixed, out_of_range,
};
use crate::fixed::FixedDay;
use crate::julian::fixed_from_julian;

/// The tabular Hijri calendar.
///
/// ```
/// use teistro_calendar::hijri::HijriTabular;
/// use teistro_calendar::gregorian::fixed_from_gregorian;
/// use teistro_calendar::{CalendarDate, CalendarSystem};
/// use teistro_core::catalogue::Calendar;
///
/// // The arithmetic year 1446 begins on 8 July 2024; a sighted crescent
/// // may put it a day either side.
/// let new_year = HijriTabular.fixed_of(&CalendarDate::defined(Calendar::HijriTabular, 1446, 1, 1))?;
/// assert_eq!(new_year, fixed_from_gregorian(2024, 7, 8));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HijriTabular;

/// 1 Muharram 1 AH: Friday 16 July 622, Julian.
pub const EPOCH: FixedDay = fixed_from_julian(622, 7, 16);

/// The Hijri years converted: every one inside the Gregorian calendar's
/// own range, −9999 to 9999.
pub const HIJRI_RANGE: (i32, i32) = (-10_946, 9_665);

/// Whether a Hijri year is a leap year: its Dhu al-Hijja has 30 days.
///
/// ```
/// use teistro_calendar::hijri::is_leap_year;
///
/// // The 2nd, 5th, 7th … 29th years of each thirty are leap.
/// assert!(is_leap_year(2) && is_leap_year(29) && is_leap_year(32));
/// assert!(!is_leap_year(1) && !is_leap_year(30));
/// ```
#[must_use]
pub const fn is_leap_year(year: i32) -> bool {
    (14 + 11 * year as i64).rem_euclid(30) < 11
}

/// The length of a month, or `None` for a month outside 1 to 12.
///
/// ```
/// use teistro_calendar::hijri::month_length;
///
/// // The months alternate 30 and 29; Dhu al-Hijja has 30 in a leap year.
/// assert_eq!(month_length(1446, 1), Some(30));
/// assert_eq!(month_length(1446, 2), Some(29));
/// assert_eq!(month_length(2, 12), Some(30));
/// assert_eq!(month_length(1, 12), Some(29));
/// ```
#[must_use]
pub const fn month_length(year: i32, month: u8) -> Option<u8> {
    match month {
        12 if is_leap_year(year) => Some(30),
        1..=12 if month % 2 == 1 => Some(30),
        1..=12 => Some(29),
        _ => None,
    }
}

/// The fixed day of a Hijri date; the date is not validated.
///
/// ```
/// use teistro_calendar::hijri::{EPOCH, fixed_from_hijri};
///
/// assert_eq!(fixed_from_hijri(1, 1, 1), EPOCH);
/// ```
#[must_use]
pub const fn fixed_from_hijri(year: i32, month: u8, day: u8) -> FixedDay {
    let y = year as i64;
    let m = month as i64;
    EPOCH.plus_days(
        day as i64 - 1
            + 29 * (m - 1)
            + (6 * m - 1).div_euclid(11)
            + (y - 1) * 354
            + (3 + 11 * y).div_euclid(30),
    )
}

/// The Hijri date of a fixed day.
///
/// ```
/// use teistro_calendar::gregorian::fixed_from_gregorian;
/// use teistro_calendar::hijri::hijri_from_fixed;
///
/// assert_eq!(hijri_from_fixed(fixed_from_gregorian(2024, 7, 7)), (1445, 12, 30));
/// ```
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "years are bounded by the fixed day range; a day of the year is below 355"
)]
pub const fn hijri_from_fixed(fixed: FixedDay) -> (i32, u8, u8) {
    let year = ((30 * (fixed.get() - EPOCH.get()) + 10_646).div_euclid(10_631)) as i32;
    let prior = fixed.get() - fixed_from_hijri(year, 1, 1).get();
    let month = ((11 * prior + 330).div_euclid(325)) as u8;
    let day = (fixed.get() - fixed_from_hijri(year, month, 1).get() + 1) as u8;
    (year, month, day)
}

fn check_year(year: i32) -> Result<(), Error> {
    if year < HIJRI_RANGE.0 || year > HIJRI_RANGE.1 {
        return Err(out_of_range(format!(
            "{} covers Hijri years {} to {}, not {year}",
            Calendar::HijriTabular.key(),
            HIJRI_RANGE.0,
            HIJRI_RANGE.1
        ))
        .with_field("year"));
    }
    Ok(())
}

impl CalendarSystem for HijriTabular {
    fn id(&self) -> Calendar {
        Calendar::HijriTabular
    }

    fn capabilities(&self) -> CalendarCapabilities {
        CalendarCapabilities {
            range: (
                fixed_from_hijri(HIJRI_RANGE.0, 1, 1),
                fixed_from_hijri(HIJRI_RANGE.1 + 1, 1, 1).plus_days(-1),
            ),
            needs_place: false,
            needs_ephemeris: false,
            eras: &[Era::Hijri],
        }
    }

    fn date_of(&self, fixed: FixedDay) -> Result<CalendarDate, Error> {
        check_fixed(Calendar::HijriTabular, self.capabilities().range, fixed)?;
        let (year, month, day) = hijri_from_fixed(fixed);
        Ok(
            CalendarDate::defined(Calendar::HijriTabular, year, month, day)
                .with_era(Era::Hijri, year),
        )
    }

    fn fixed_of(&self, date: &CalendarDate) -> Result<FixedDay, Error> {
        crate::date::check_no_leap(date)?;
        check_year(date.year)?;
        let length = month_length(date.year, date.month).unwrap_or(0);
        check_day(
            Calendar::HijriTabular,
            date.year,
            date.month,
            date.day,
            12,
            length,
        )?;
        Ok(fixed_from_hijri(date.year, date.month, date.day))
    }

    fn month_length(&self, year: i32, month: u8) -> Result<u8, Error> {
        check_year(year)?;
        let length = month_length(year, month).unwrap_or(0);
        check_day(Calendar::HijriTabular, year, month, 1, 12, length)?;
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
    use crate::gregorian::{YEAR_RANGE, fixed_from_gregorian, gregorian_from_fixed};

    fn gregorian_of(year: i32, month: u8, day: u8) -> (i32, u8, u8) {
        gregorian_from_fixed(fixed_from_hijri(year, month, day))
    }

    #[test]
    fn the_civil_epoch_and_icus_islamic_civil() {
        // The epoch: Friday 16 July 622 Julian, 19 July 622 Gregorian.
        assert_eq!(gregorian_of(1, 1, 1), (622, 7, 19));
        assert_eq!(EPOCH.get().rem_euclid(7), 5, "a Friday");
        // ICU's islamic-civil, read through Node's Intl.
        assert_eq!(gregorian_of(1445, 12, 30), (2024, 7, 7));
        assert_eq!(gregorian_of(1446, 1, 1), (2024, 7, 8));
        assert_eq!(gregorian_of(1446, 9, 1), (2025, 3, 1));
        assert_eq!(gregorian_of(1420, 9, 24), (2000, 1, 1));
        assert_eq!(gregorian_of(1317, 10, 28), (1900, 3, 1));
        assert_eq!(gregorian_of(1524, 10, 29), (2100, 12, 31));
        // Eleven leap years in thirty, the cycle's own list.
        let leaps: Vec<i32> = (1..=30).filter(|y| is_leap_year(*y)).collect();
        assert_eq!(leaps, [2, 5, 7, 10, 13, 16, 18, 21, 24, 26, 29]);
        assert_eq!(
            fixed_from_hijri(31, 1, 1).get() - fixed_from_hijri(1, 1, 1).get(),
            10_631
        );
        let refused = HijriTabular
            .fixed_of(&CalendarDate::defined(Calendar::HijriTabular, 1446, 2, 30))
            .unwrap_err();
        assert!(
            refused.message.contains("days 1 to 29"),
            "{}",
            refused.message
        );
    }

    #[test]
    fn every_day_round_trips_inside_the_gregorian_range() {
        let (first, last) = HijriTabular.capabilities().range;
        assert!(first.get() >= fixed_from_gregorian(YEAR_RANGE.0, 1, 1).get());
        assert!(last.get() <= fixed_from_gregorian(YEAR_RANGE.1, 12, 31).get());
        let mut previous = (HIJRI_RANGE.0 - 1, 12u8, 30u8);
        let mut fixed = first.get();
        while fixed <= last.get() {
            let (y, m, d) = hijri_from_fixed(FixedDay::new(fixed));
            assert_eq!(fixed_from_hijri(y, m, d).get(), fixed, "{y}-{m}-{d}");
            assert!((y, m, d) > previous, "{y}-{m}-{d} after {previous:?}");
            assert!(d >= 1 && d <= month_length(y, m).unwrap_or(0));
            previous = (y, m, d);
            fixed += 1;
        }
        assert_eq!(previous.0, HIJRI_RANGE.1);
    }
}
