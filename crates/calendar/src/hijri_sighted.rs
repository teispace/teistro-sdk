//! The sighted Hijri calendar's months as a table: each month's first
//! day at a place, as the crescent reckoning found them
//! (`docs/03-design/hijri-sighted.md`).
//!
//! A sighted month begins on the day after the evening its new crescent
//! is seen; a month whose crescent is not seen on the evening of its 29th
//! completes thirty days. Which evening sees it is a question of the sky
//! at the place, so the months are reckoned by the almanac, which has an
//! ephemeris and a place, and handed here as a table that converts days
//! like any other calendar over the span it covers. The months are named
//! by the tabular month whose first day lies nearest, which is never more
//! than three days away.
//!
//! ```
//! use teistro_calendar::gregorian::fixed_from_gregorian;
//! use teistro_calendar::hijri_sighted::{SightedMonth, SightedMonths};
//! use teistro_calendar::CalendarSystem;
//!
//! // Ramadan and Shawwal 1446 as a sighting began them, and Dhu al-Qa'da's
//! // first day closing Shawwal.
//! let months = SightedMonths::new(
//!     vec![
//!         SightedMonth { year: 1446, month: 9, first: fixed_from_gregorian(2025, 3, 2), seen: true },
//!         SightedMonth { year: 1446, month: 10, first: fixed_from_gregorian(2025, 3, 31), seen: true },
//!     ],
//!     fixed_from_gregorian(2025, 4, 30),
//! )?;
//! let eid = months.date_of(fixed_from_gregorian(2025, 3, 31))?;
//! assert_eq!((eid.year, eid.month, eid.day), (1446, 10, 1));
//! // Ramadan had 29 days: its 29th's crescent was seen.
//! assert_eq!(months.month_length(1446, 9)?, 29);
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Calendar, Era};
use teistro_core::error::Error;

use crate::date::{CalendarCapabilities, CalendarDate, CalendarSystem, check_day, out_of_range};
use crate::fixed::FixedDay;
use crate::hijri::fixed_from_hijri;

/// The most days a sighted month's first day stands from the tabular
/// one of the same name. A sighting moves a month a day or two from the
/// table; three is a reckoning that has lost its place.
pub const MOST_FROM_TABLE: i64 = 3;

/// One sighted month: its name and its first day.
///
/// ```
/// use teistro_calendar::gregorian::fixed_from_gregorian;
/// use teistro_calendar::hijri_sighted::SightedMonth;
///
/// let shawwal = SightedMonth { year: 1446, month: 10, first: fixed_from_gregorian(2025, 3, 31), seen: true };
/// assert!(shawwal.seen);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SightedMonth {
    /// The Hijri year.
    pub year: i32,
    /// The month, 1 (Muharram) to 12 (Dhu al-Hijja).
    pub month: u8,
    /// Its first day.
    pub first: FixedDay,
    /// Whether it began on a crescent seen on the evening before; `false`
    /// when the month before completed thirty days unseen.
    pub seen: bool,
}

/// The sighted months over a span of days.
///
/// ```
/// use teistro_calendar::gregorian::fixed_from_gregorian;
/// use teistro_calendar::hijri_sighted::{SightedMonth, SightedMonths};
/// use teistro_calendar::CalendarSystem;
///
/// let shawwal = SightedMonth { year: 1446, month: 10, first: fixed_from_gregorian(2025, 3, 31), seen: true };
/// let table = SightedMonths::new(vec![shawwal], fixed_from_gregorian(2025, 4, 30))?;
/// // Shawwal 1446 ran thirty days, Dhu al-Qa'da beginning on 30 April.
/// assert_eq!(table.month_length(1446, 10)?, 30);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SightedMonths {
    months: Vec<SightedMonth>,
    end: FixedDay,
}

impl SightedMonths {
    /// The table of `months`, consecutive and in order, the last ending
    /// the day before `end`.
    ///
    /// ```
    /// use teistro_calendar::gregorian::fixed_from_gregorian;
    /// use teistro_calendar::hijri_sighted::{SightedMonth, SightedMonths};
    ///
    /// let shawwal = SightedMonth { year: 1446, month: 10, first: fixed_from_gregorian(2025, 3, 31), seen: true };
    /// // A month of 31 days is no Hijri month.
    /// assert!(SightedMonths::new(vec![shawwal], fixed_from_gregorian(2025, 5, 1)).is_err());
    /// ```
    ///
    /// # Errors
    ///
    /// `INTERNAL` for an empty table, months out of order or not
    /// consecutive, a month other than 29 or 30 days, or one more than
    /// [`MOST_FROM_TABLE`] days from the table's month of its name.
    pub fn new(months: Vec<SightedMonth>, end: FixedDay) -> Result<SightedMonths, Error> {
        let Some(first) = months.first() else {
            return Err(Error::internal("a sighted Hijri table holds no month"));
        };
        let mut expected = (first.year, first.month);
        for (at, month) in months.iter().enumerate() {
            let next = months.get(at + 1).map_or(end, |next| next.first);
            let length = month.first.days_until(next);
            if (month.year, month.month) != expected || !(29..=30).contains(&length) {
                return Err(Error::internal(format!(
                    "sighted Hijri month {}-{:02} of {length} days, after {}-{:02}: a table's months follow each other and run 29 or 30 days",
                    month.year, month.month, expected.0, expected.1
                )));
            }
            let table = fixed_from_hijri(month.year, month.month, 1);
            if table.days_until(month.first).abs() > MOST_FROM_TABLE {
                return Err(Error::internal(format!(
                    "sighted Hijri month {}-{:02} begins {} days from the table's",
                    month.year,
                    month.month,
                    table.days_until(month.first)
                )));
            }
            expected = if month.month == 12 {
                (month.year + 1, 1)
            } else {
                (month.year, month.month + 1)
            };
        }
        Ok(SightedMonths { months, end })
    }

    /// The months, in order.
    ///
    /// ```
    /// use teistro_calendar::gregorian::fixed_from_gregorian;
    /// use teistro_calendar::hijri_sighted::{SightedMonth, SightedMonths};
    ///
    /// let shawwal = SightedMonth { year: 1446, month: 10, first: fixed_from_gregorian(2025, 3, 31), seen: true };
    /// let table = SightedMonths::new(vec![shawwal], fixed_from_gregorian(2025, 4, 29))?;
    /// assert_eq!(table.months(), [shawwal]);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn months(&self) -> &[SightedMonth] {
        &self.months
    }

    /// The first day after the last month.
    ///
    /// ```
    /// use teistro_calendar::gregorian::fixed_from_gregorian;
    /// use teistro_calendar::hijri_sighted::{SightedMonth, SightedMonths};
    ///
    /// let shawwal = SightedMonth { year: 1446, month: 10, first: fixed_from_gregorian(2025, 3, 31), seen: true };
    /// let table = SightedMonths::new(vec![shawwal], fixed_from_gregorian(2025, 4, 29))?;
    /// assert_eq!(table.end(), fixed_from_gregorian(2025, 4, 29));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub const fn end(&self) -> FixedDay {
        self.end
    }

    /// The month a day falls in and its first day after.
    fn month_of(&self, fixed: FixedDay) -> Option<(&SightedMonth, FixedDay)> {
        let at = self
            .months
            .partition_point(|month| month.first.get() <= fixed.get());
        let month = self.months.get(at.checked_sub(1)?)?;
        let next = self.months.get(at).map_or(self.end, |next| next.first);
        (fixed.get() < next.get()).then_some((month, next))
    }

    fn named(&self, year: i32, month: u8) -> Result<(&SightedMonth, FixedDay), Error> {
        let at = self
            .months
            .iter()
            .position(|held| (held.year, held.month) == (year, month))
            .ok_or_else(|| self.uncovered(&format!("{year}-{month:02}")))?;
        let held = self
            .months
            .get(at)
            .ok_or_else(|| Error::internal("a sighted month came apart"))?;
        let next = self.months.get(at + 1).map_or(self.end, |next| next.first);
        Ok((held, next))
    }

    fn uncovered(&self, what: &str) -> Error {
        let first = self.months.first().map_or(0, |month| month.first.get());
        out_of_range(format!(
            "the sighted Hijri months reckoned cover fixed days {first} to {}, not {what}",
            self.end.get() - 1
        ))
        .with_hint("the months are reckoned over a request's own days and a year before them")
    }
}

impl CalendarSystem for SightedMonths {
    fn id(&self) -> Calendar {
        Calendar::HijriSighted
    }

    fn capabilities(&self) -> CalendarCapabilities {
        let first = self.months.first().map_or(self.end, |month| month.first);
        CalendarCapabilities {
            range: (first, self.end.plus_days(-1)),
            needs_place: true,
            needs_ephemeris: true,
            eras: &[Era::Hijri],
        }
    }

    fn date_of(&self, fixed: FixedDay) -> Result<CalendarDate, Error> {
        let (month, _) = self.month_of(fixed).ok_or_else(|| {
            self.uncovered(&format!("fixed day {}", fixed.get()))
                .with_field("fixed")
        })?;
        let day = u8::try_from(month.first.days_until(fixed) + 1)
            .map_err(|_| Error::internal("a sighted month ran past 255 days"))?;
        Ok(
            CalendarDate::defined(Calendar::HijriSighted, month.year, month.month, day)
                .with_era(Era::Hijri, month.year),
        )
    }

    fn fixed_of(&self, date: &CalendarDate) -> Result<FixedDay, Error> {
        crate::date::check_no_leap(date)?;
        let length = self.month_length(date.year, date.month)?;
        check_day(
            Calendar::HijriSighted,
            date.year,
            date.month,
            date.day,
            12,
            length,
        )?;
        let (month, _) = self.named(date.year, date.month)?;
        Ok(month.first.plus_days(i64::from(date.day) - 1))
    }

    fn month_length(&self, year: i32, month: u8) -> Result<u8, Error> {
        check_day(Calendar::HijriSighted, year, month, 1, 12, 30)?;
        let (held, next) = self.named(year, month)?;
        u8::try_from(held.first.days_until(next))
            .map_err(|_| Error::internal("a sighted month ran past 255 days"))
    }

    /// Whether Dhu al-Hijja has 30 days, where the table holds it.
    fn is_leap(&self, year: i32) -> bool {
        self.month_length(year, 12).is_ok_and(|length| length == 30)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;
    use crate::gregorian::fixed_from_gregorian;

    fn months() -> SightedMonths {
        let month = |month, (m, d), seen| SightedMonth {
            year: 1446,
            month,
            first: fixed_from_gregorian(2025, m, d),
            seen,
        };
        SightedMonths::new(
            vec![
                month(9, (3, 2), true),
                month(10, (3, 31), true),
                month(11, (4, 30), false),
            ],
            fixed_from_gregorian(2025, 5, 29),
        )
        .unwrap()
    }

    #[test]
    fn every_day_converts_and_converts_back() {
        let table = months();
        let (first, last) = table.capabilities().range;
        for fixed in first.get()..=last.get() {
            let date = table.date_of(FixedDay::new(fixed)).unwrap();
            assert_eq!(table.fixed_of(&date).unwrap().get(), fixed);
        }
        assert_eq!(table.month_length(1446, 10).unwrap(), 30);
        assert_eq!(table.month_length(1446, 11).unwrap(), 29);
        assert!(table.date_of(first.plus_days(-1)).is_err());
        assert!(table.date_of(last.plus_days(1)).is_err());
        assert!(table.month_length(1446, 12).is_err());
        assert!(!table.is_leap(1446));
        let refused = table
            .fixed_of(&CalendarDate::defined(Calendar::HijriSighted, 1446, 11, 30))
            .unwrap_err();
        assert_eq!(refused.field(), Some("day"));
    }

    #[test]
    fn a_table_that_loses_its_place_is_refused() {
        let shawwal = |first| SightedMonth {
            year: 1446,
            month: 10,
            first,
            seen: true,
        };
        // Four days from the table's 31 March.
        let far = fixed_from_gregorian(2025, 4, 4);
        assert!(SightedMonths::new(vec![shawwal(far)], far.plus_days(29)).is_err());
        assert!(SightedMonths::new(Vec::new(), far).is_err());
        // Out of order.
        let first = fixed_from_gregorian(2025, 3, 31);
        let mut later = shawwal(first.plus_days(30));
        later.month = 12;
        assert!(SightedMonths::new(vec![shawwal(first), later], first.plus_days(59)).is_err());
    }
}
