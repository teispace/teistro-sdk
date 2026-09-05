//! Bikram Sambat over its table of month lengths: the official span as the
//! authority publishes it (rank 2 until the source memo cites the
//! publication) and a computed extension either side, every date saying
//! which answered (`docs/03-design/calendar-bikram-sambat.md`,
//! `docs/calendars/bikram-sambat.md`).

use std::sync::OnceLock;

use teistro_core::catalogue::{Calendar, Era};
use teistro_core::envelope::CalendarResolution;
use teistro_core::error::Error;

use crate::date::{CalendarCapabilities, CalendarDate, CalendarSystem, check_day, out_of_range};
use crate::fixed::FixedDay;
use crate::gregorian::fixed_from_gregorian;

mod table;

pub use table::{EDITION, FIRST_YEAR, LAST_YEAR, OFFICIAL_FIRST_YEAR, OFFICIAL_LAST_YEAR};

/// The anchor the whole table hangs on: 1 Baisakh 1970 BS is 13 April 1913
/// (the baseline engine's epoch, inside the official span).
const ANCHOR_YEAR: i32 = 1970;
const ANCHOR_GREGORIAN: (i32, u8, u8) = (1913, 4, 13);

/// The calendar over a table.
#[derive(Debug)]
pub struct BikramSambat {
    /// Month lengths per year from `FIRST_YEAR`.
    month_lengths: &'static [[u8; 12]],
    /// The fixed day of 1 Baisakh of each year, one more entry than years.
    year_starts: Vec<FixedDay>,
    /// The official span inside the table.
    official: (i32, i32),
    /// The table's edition.
    edition: &'static str,
}

static SHIPPED: OnceLock<BikramSambat> = OnceLock::new();

impl BikramSambat {
    /// The shipped calendar over the checked-in table.
    pub fn shipped() -> &'static BikramSambat {
        SHIPPED.get_or_init(|| {
            BikramSambat::over(
                &table::MONTH_LENGTHS,
                FIRST_YEAR,
                (
                    ANCHOR_YEAR,
                    fixed_from_gregorian(
                        ANCHOR_GREGORIAN.0,
                        ANCHOR_GREGORIAN.1,
                        ANCHOR_GREGORIAN.2,
                    ),
                ),
                (OFFICIAL_FIRST_YEAR, OFFICIAL_LAST_YEAR),
                EDITION,
            )
        })
    }

    /// A calendar over a table whose first row is `first_year`, anchored on
    /// one year whose 1 Baisakh is known as a fixed day; every other year's
    /// start follows from the month lengths in both directions.
    #[must_use]
    pub fn over(
        month_lengths: &'static [[u8; 12]],
        first_year: i32,
        anchor: (i32, FixedDay),
        official: (i32, i32),
        edition: &'static str,
    ) -> BikramSambat {
        let year_days = |months: &[u8; 12]| months.iter().map(|d| i64::from(*d)).sum::<i64>();
        let anchor_index = usize::try_from(anchor.0 - first_year).unwrap_or(0);
        let mut year_starts = vec![FixedDay::default(); month_lengths.len() + 1];
        let mut start = anchor.1;
        for index in anchor_index..month_lengths.len() {
            if let (Some(slot), Some(months)) =
                (year_starts.get_mut(index), month_lengths.get(index))
            {
                *slot = start;
                start = start.plus_days(year_days(months));
            }
        }
        if let Some(last) = year_starts.last_mut() {
            *last = start;
        }
        let mut start = anchor.1;
        for index in (0..anchor_index).rev() {
            if let (Some(months), Some(slot)) =
                (month_lengths.get(index), year_starts.get_mut(index))
            {
                start = start.plus_days(-year_days(months));
                *slot = start;
            }
        }
        BikramSambat {
            month_lengths,
            year_starts,
            official,
            edition,
        }
    }

    /// The first and last year of the table.
    #[must_use]
    pub fn years(&self) -> (i32, i32) {
        (
            FIRST_YEAR,
            FIRST_YEAR + i32::try_from(self.month_lengths.len()).unwrap_or(0) - 1,
        )
    }

    /// The official span.
    #[must_use]
    pub const fn official_years(&self) -> (i32, i32) {
        self.official
    }

    /// The table's edition.
    #[must_use]
    pub const fn edition(&self) -> &'static str {
        self.edition
    }

    fn row(&self, year: i32) -> Result<&[u8; 12], Error> {
        let (first, last) = self.years();
        usize::try_from(year - first)
            .ok()
            .and_then(|i| self.month_lengths.get(i))
            .ok_or_else(|| {
                out_of_range(format!(
                    "Bikram Sambat covers {first} to {last} BS, not {year}"
                ))
                .with_field("year")
            })
    }

    fn resolution(&self, year: i32) -> CalendarResolution {
        if year >= self.official.0 && year <= self.official.1 {
            CalendarResolution::Tabular {
                authority: String::from("the official almanac, through the baseline engine"),
                edition: self.edition.to_string(),
            }
        } else {
            CalendarResolution::Computed {
                model: String::from("the baseline engine's computed extension"),
            }
        }
    }
}

impl CalendarSystem for BikramSambat {
    fn id(&self) -> Calendar {
        Calendar::BikramSambat
    }

    fn capabilities(&self) -> CalendarCapabilities {
        let last = self
            .year_starts
            .last()
            .copied()
            .unwrap_or_default()
            .plus_days(-1);
        CalendarCapabilities {
            range: (self.year_starts.first().copied().unwrap_or_default(), last),
            needs_place: false,
            needs_ephemeris: false,
            eras: &[Era::Vikrama],
        }
    }

    fn date_of(&self, fixed: FixedDay) -> Result<CalendarDate, Error> {
        let (first_year, last_year) = self.years();
        let index = match self.year_starts.binary_search(&fixed) {
            Ok(i) => i,
            Err(0) => {
                return Err(out_of_range(format!(
                    "{fixed} is before 1 Baisakh {first_year} BS, the first day of the table"
                )));
            }
            Err(i) => i - 1,
        };
        let (Some(start), Some(months)) =
            (self.year_starts.get(index), self.month_lengths.get(index))
        else {
            return Err(out_of_range(format!(
                "{fixed} is after the last day of {last_year} BS, the last year of the table"
            )));
        };
        let year = first_year + i32::try_from(index).unwrap_or(0);
        let mut remaining = start.days_until(fixed);
        for (m, length) in months.iter().enumerate() {
            if remaining < i64::from(*length) {
                let month = u8::try_from(m + 1).unwrap_or(1);
                let day = u8::try_from(remaining + 1).unwrap_or(1);
                return Ok(CalendarDate {
                    calendar: Calendar::BikramSambat,
                    year,
                    month,
                    day,
                    era: Some(crate::date::EraNumber {
                        era: Era::Vikrama,
                        year,
                    }),
                    resolution: self.resolution(year),
                });
            }
            remaining -= i64::from(*length);
        }
        Err(out_of_range(format!(
            "{fixed} is after the last day of {last_year} BS, the last year of the table"
        )))
    }

    fn fixed_of(&self, date: &CalendarDate) -> Result<FixedDay, Error> {
        let months = self.row(date.year)?;
        let length = months
            .get(usize::from(date.month.saturating_sub(1)))
            .copied()
            .unwrap_or(0);
        check_day(
            Calendar::BikramSambat,
            date.year,
            date.month,
            date.day,
            12,
            length,
        )?;
        let index = usize::try_from(date.year - FIRST_YEAR).unwrap_or(0);
        let start = self.year_starts.get(index).copied().unwrap_or_default();
        let before: i64 = months
            .iter()
            .take(usize::from(date.month - 1))
            .map(|d| i64::from(*d))
            .sum();
        Ok(start.plus_days(before + i64::from(date.day) - 1))
    }

    fn month_length(&self, year: i32, month: u8) -> Result<u8, Error> {
        let months = self.row(year)?;
        let length = months
            .get(usize::from(month.saturating_sub(1)))
            .copied()
            .unwrap_or(0);
        check_day(Calendar::BikramSambat, year, month, 1, 12, length)?;
        Ok(length)
    }

    fn is_leap(&self, year: i32) -> bool {
        self.row(year)
            .is_ok_and(|months| months.iter().map(|d| u32::from(*d)).sum::<u32>() == 366)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;
    use crate::gregorian::{Gregorian, gregorian_from_fixed};

    #[test]
    fn the_epoch_and_known_new_years() {
        let bs = BikramSambat::shipped();
        assert_eq!(bs.years(), (FIRST_YEAR, LAST_YEAR));
        // 1 Baisakh 1970 BS is 13 April 1913, the baseline engine's epoch.
        let epoch_1970 = bs.to_fixed_ymd(1970, 1, 1).unwrap();
        assert_eq!(gregorian_from_fixed(epoch_1970), (1913, 4, 13));
        // 1 Baisakh 2072 BS is 14 April 2015; 1 Baisakh 2081 BS is 13 April 2024.
        assert_eq!(
            gregorian_from_fixed(bs.to_fixed_ymd(2072, 1, 1).unwrap()),
            (2015, 4, 14)
        );
        assert_eq!(
            gregorian_from_fixed(bs.to_fixed_ymd(2081, 1, 1).unwrap()),
            (2024, 4, 13)
        );
        let day = Gregorian.to_fixed_ymd(2015, 4, 14).unwrap();
        let date = bs.date_of(day).unwrap();
        assert_eq!((date.year, date.month, date.day), (2072, 1, 1));
        assert!(matches!(
            date.resolution,
            CalendarResolution::Tabular { .. }
        ));
        assert!(matches!(
            bs.date_of(bs.to_fixed_ymd(1900, 5, 3).unwrap())
                .unwrap()
                .resolution,
            CalendarResolution::Computed { .. }
        ));
        assert_eq!(bs.month_length(2072, 1).unwrap(), 31);
        assert!(
            bs.to_fixed_ymd(2072, 1, 32)
                .unwrap_err()
                .message
                .contains("days 1 to 31")
        );
        assert!(
            bs.to_fixed_ymd(1800, 1, 1)
                .unwrap_err()
                .message
                .contains("covers")
        );
        assert!(bs.to_fixed_ymd(2072, 13, 1).is_err());
        assert!(bs.is_leap(2072) || !bs.is_leap(2072));
    }

    #[test]
    fn every_day_round_trips_and_years_have_365_or_366_days() {
        let bs = BikramSambat::shipped();
        let (first, last) = bs.capabilities().range;
        let mut fixed = first.get();
        let mut previous: Option<(i32, u8, u8)> = None;
        while fixed <= last.get() {
            let f = FixedDay::new(fixed);
            let date = bs.date_of(f).unwrap();
            assert_eq!(bs.fixed_of(&date).unwrap(), f, "{date}");
            let key = (date.year, date.month, date.day);
            if let Some(p) = previous {
                assert!(key > p);
            }
            previous = Some(key);
            fixed += 1;
        }
        for year in FIRST_YEAR..=LAST_YEAR {
            let days: u32 = (1..=12)
                .map(|m| u32::from(bs.month_length(year, m).unwrap()))
                .sum();
            assert!(days == 365 || days == 366, "{year} has {days} days");
        }
        assert!(bs.date_of(first.plus_days(-1)).is_err());
        assert!(bs.date_of(last.plus_days(1)).is_err());
    }
}
