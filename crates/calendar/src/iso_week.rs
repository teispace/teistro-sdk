//! The ISO week date: a week-year, a week 1 to 53, and a weekday Monday
//! 1 to Sunday 7. As a calendar, the month field carries the week and the
//! day field the weekday.

use teistro_core::catalogue::{Calendar, Era};
use teistro_core::error::Error;

use crate::date::{
    CalendarCapabilities, CalendarDate, CalendarSystem, check_day, check_fixed, nonexistent,
};
use crate::fixed::{FixedDay, Weekday};
use crate::gregorian::{YEAR_RANGE, check_year, fixed_from_gregorian, year_from_fixed};

/// The ISO week calendar.
///
/// ```
/// use teistro_calendar::{CalendarDate, CalendarSystem, IsoWeek};
/// use teistro_calendar::gregorian::fixed_from_gregorian;
/// use teistro_core::catalogue::Calendar;
///
/// // Sunday 31 December 2023 is the seventh day of week 52 of 2023.
/// let date = IsoWeek.date_of(fixed_from_gregorian(2023, 12, 31))?;
/// assert_eq!((date.year, date.month, date.day), (2023, 52, 7));
/// assert!(IsoWeek.fixed_of(&CalendarDate::defined(Calendar::IsoWeek, 2024, 53, 1)).is_err());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IsoWeek;

/// An ISO week date.
///
/// ```
/// use teistro_calendar::IsoWeekDate;
/// use teistro_calendar::gregorian::fixed_from_gregorian;
/// use teistro_calendar::iso_week::iso_from_fixed;
///
/// // 1 January 2021 was a Friday in the last week of 2020.
/// assert_eq!(iso_from_fixed(fixed_from_gregorian(2021, 1, 1)), IsoWeekDate { week_year: 2020, week: 53, weekday: 5 });
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct IsoWeekDate {
    /// The week-year.
    pub week_year: i32,
    /// The week, 1 to 53.
    pub week: u8,
    /// The weekday, Monday 1 to Sunday 7.
    pub weekday: u8,
}

/// The fixed day of an ISO week date; not validated.
///
/// ```
/// use teistro_calendar::gregorian::fixed_from_gregorian;
/// use teistro_calendar::iso_week::fixed_from_iso;
///
/// assert_eq!(fixed_from_iso(2024, 1, 1), fixed_from_gregorian(2024, 1, 1));
/// assert_eq!(fixed_from_iso(2025, 1, 1), fixed_from_gregorian(2024, 12, 30));
/// ```
#[must_use]
pub const fn fixed_from_iso(week_year: i32, week: u8, weekday: u8) -> FixedDay {
    // The Sunday strictly before 28 December of the previous year, then
    // whole weeks: Reingold and Dershowitz's `nth-kday`.
    let anchor = fixed_from_gregorian(week_year - 1, 12, 28)
        .plus_days(-1)
        .on_or_before(Weekday::Sunday);
    anchor.plus_days(7 * week as i64 + weekday as i64)
}

/// The ISO week date of a fixed day.
///
/// ```
/// use teistro_calendar::IsoWeekDate;
/// use teistro_calendar::gregorian::fixed_from_gregorian;
/// use teistro_calendar::iso_week::iso_from_fixed;
///
/// assert_eq!(iso_from_fixed(fixed_from_gregorian(2024, 12, 31)), IsoWeekDate { week_year: 2025, week: 1, weekday: 2 });
/// ```
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a week below fifty-four and a weekday below eight; the month and day are positive"
)]
pub const fn iso_from_fixed(fixed: FixedDay) -> IsoWeekDate {
    let approx = year_from_fixed(fixed.plus_days(-3));
    let week_year = if fixed.get() >= fixed_from_iso(approx + 1, 1, 1).get() {
        approx + 1
    } else {
        approx
    };
    let week = ((fixed.get() - fixed_from_iso(week_year, 1, 1).get()).div_euclid(7) + 1) as u8;
    let weekday = fixed.weekday().iso_number();
    IsoWeekDate {
        week_year,
        week,
        weekday,
    }
}

/// Whether a week-year has 53 weeks.
///
/// ```
/// use teistro_calendar::iso_week::is_long_year;
///
/// assert!(is_long_year(2020) && is_long_year(2026));
/// assert!(!is_long_year(2024));
/// ```
#[must_use]
pub const fn is_long_year(week_year: i32) -> bool {
    let next = fixed_from_iso(week_year + 1, 1, 1).get();
    let this = fixed_from_iso(week_year, 1, 1).get();
    (next - this) / 7 == 53
}

impl CalendarSystem for IsoWeek {
    fn id(&self) -> Calendar {
        Calendar::IsoWeek
    }

    fn capabilities(&self) -> CalendarCapabilities {
        CalendarCapabilities {
            range: (
                fixed_from_iso(YEAR_RANGE.0, 1, 1),
                fixed_from_iso(YEAR_RANGE.1 + 1, 1, 1).plus_days(-1),
            ),
            needs_place: false,
            needs_ephemeris: false,
            eras: &[Era::CommonEra, Era::BeforeCommonEra],
        }
    }

    fn date_of(&self, fixed: FixedDay) -> Result<CalendarDate, Error> {
        check_fixed(Calendar::IsoWeek, self.capabilities().range, fixed)?;
        let date = iso_from_fixed(fixed);
        check_year(Calendar::IsoWeek, date.week_year)?;
        Ok(CalendarDate::defined(
            Calendar::IsoWeek,
            date.week_year,
            date.week,
            date.weekday,
        ))
    }

    fn fixed_of(&self, date: &CalendarDate) -> Result<FixedDay, Error> {
        crate::date::check_no_leap(date)?;
        check_year(Calendar::IsoWeek, date.year)?;
        let weeks = if is_long_year(date.year) { 53 } else { 52 };
        if date.month == 0 || date.month > weeks {
            return Err(nonexistent(format!(
                "ISO week-year {} has weeks 1 to {weeks}, not {}",
                date.year, date.month
            ))
            .with_field("month"));
        }
        check_day(Calendar::IsoWeek, date.year, date.month, date.day, weeks, 7)?;
        Ok(fixed_from_iso(date.year, date.month, date.day))
    }

    fn month_length(&self, year: i32, month: u8) -> Result<u8, Error> {
        check_year(Calendar::IsoWeek, year)?;
        let weeks = if is_long_year(year) { 53 } else { 52 };
        check_day(Calendar::IsoWeek, year, month, 1, weeks, 7)?;
        Ok(7)
    }

    fn is_leap(&self, year: i32) -> bool {
        is_long_year(year)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    fn iso_of(year: i32, month: u8, day: u8) -> IsoWeekDate {
        iso_from_fixed(fixed_from_gregorian(year, month, day))
    }

    #[test]
    fn the_edges_of_the_week_year() {
        assert_eq!(
            iso_of(2020, 12, 31),
            IsoWeekDate {
                week_year: 2020,
                week: 53,
                weekday: 4
            }
        );
        assert_eq!(
            iso_of(2021, 1, 3),
            IsoWeekDate {
                week_year: 2020,
                week: 53,
                weekday: 7
            }
        );
        assert_eq!(
            iso_of(2021, 1, 4),
            IsoWeekDate {
                week_year: 2021,
                week: 1,
                weekday: 1
            }
        );
        assert_eq!(
            iso_of(2024, 12, 30),
            IsoWeekDate {
                week_year: 2025,
                week: 1,
                weekday: 1
            }
        );
        assert_eq!(
            iso_of(2000, 1, 1),
            IsoWeekDate {
                week_year: 1999,
                week: 52,
                weekday: 6
            }
        );
        assert!(is_long_year(2020) && is_long_year(2015) && !is_long_year(2021));
        assert!(IsoWeek.to_fixed_ymd(2021, 53, 1).is_err());
        assert!(IsoWeek.to_fixed_ymd(2020, 53, 7).is_ok());
        assert!(IsoWeek.to_fixed_ymd(2020, 1, 8).is_err());
    }

    #[test]
    fn every_day_round_trips() {
        let (first, last) = IsoWeek.capabilities().range;
        let mut fixed = first.get();
        while fixed <= last.get() {
            let f = FixedDay::new(fixed);
            let date = iso_from_fixed(f);
            assert_eq!(
                fixed_from_iso(date.week_year, date.week, date.weekday),
                f,
                "{date:?}"
            );
            assert!(date.week >= 1 && date.week <= 53 && date.weekday >= 1 && date.weekday <= 7);
            fixed += 1;
        }
    }
}
