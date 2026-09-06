//! Bikram Sambat: the official table as the authority publishes it (rank 2
//! until the source memo cites the publication), the computed extension
//! either side of it from the SDK's own engine, every date saying which
//! answered, and the engine itself (`docs/03-design/calendar-bikram-sambat.md`,
//! `docs/calendars/bikram-sambat.md`).
//!
//! ```
//! use teistro_calendar::{BikramSambat, CalendarSystem, Gregorian};
//! use teistro_core::envelope::CalendarResolution;
//!
//! let bs = BikramSambat::shipped();
//! let new_year = bs.to_fixed_ymd(2081, 1, 1).expect("in the table");
//! assert_eq!(Gregorian.date_of(new_year).map(|d| (d.year, d.month, d.day)).ok(), Some((2024, 4, 13)));
//! let date = bs.date_of(new_year).expect("in the table");
//! assert!(matches!(date.resolution, CalendarResolution::Tabular { .. }));
//! // A year the authority has not published is computed and says so.
//! let early = bs.to_fixed_ymd(1750, 3, 15).expect("in the computed span");
//! assert!(matches!(bs.date_of(early).expect("in the table").resolution, CalendarResolution::Computed { .. }));
//! ```

use std::borrow::Cow;
use std::sync::OnceLock;

use teistro_core::catalogue::{Calendar, Era};
use teistro_core::envelope::{CalendarResolution, MonthDay};
use teistro_core::error::Error;

use crate::date::{
    CalendarCapabilities, CalendarDate, CalendarSystem, EraNumber, check_day, out_of_range,
};
use crate::fixed::FixedDay;
use crate::gregorian::fixed_from_gregorian;
use crate::solar::MonthStartRule;

pub mod engine;
pub mod fit;
#[rustfmt::skip]
mod generated;

pub use engine::{Engine, KATHMANDU, YEAR_OFFSET, YearRow};
pub use fit::{Divergence, FitReport, fit};
pub use generated::{FIRST_YEAR, LAST_YEAR, OFFICIAL_FIRST_YEAR, OFFICIAL_LAST_YEAR};

/// The month-start rule the shipped table is computed under: chosen by
/// the measurement `cargo xtask calendars bs-fit` publishes in the source
/// memo, and regenerated with it.
pub const SHIPPED_RULE: MonthStartRule = generated::RULE;

/// The span the shipped table covers.
pub const SHIPPED_SPAN: (i32, i32) = (FIRST_YEAR, LAST_YEAR);

/// A year inside the official span where the engine's answer differs
/// from the table's: the engine's 1 Baisakh relative to the table's, and
/// the engine's month lengths, so a date can report both labels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelRow {
    /// The year.
    pub year: i32,
    /// The engine's 1 Baisakh less the table's, in days.
    pub start_offset: i8,
    /// The engine's month lengths.
    pub months: [u8; 12],
}

/// The table the calendar runs over: what `cargo xtask gen calendars`
/// writes and `check-calendars` holds to the engine.
#[derive(Debug)]
pub struct Table {
    /// The first year of the rows.
    pub first_year: i32,
    /// Month lengths per year, Baisakh to Chaitra.
    pub month_lengths: &'static [[u8; 12]],
    /// A year whose 1 Baisakh is known as a Gregorian date; every other
    /// year's start follows from the rows in both directions.
    pub anchor: (i32, (i32, u8, u8)),
    /// The official span inside the rows.
    pub official: (i32, i32),
    /// Who publishes the official span.
    pub authority: &'static str,
    /// The official span's edition.
    pub edition: &'static str,
    /// The frame the computed rows were made under: model, clock, place
    /// and rule.
    pub frame: &'static str,
    /// The months inside the official span where the engine differs.
    pub divergences: &'static [Divergence],
    /// The engine's rows for the years with a divergence.
    pub model_rows: &'static [ModelRow],
}

/// The calendar over a table.
#[derive(Debug)]
pub struct BikramSambat {
    table: &'static Table,
    /// The fixed day of 1 Baisakh of each year, one more entry than years.
    year_starts: Vec<FixedDay>,
}

static SHIPPED: OnceLock<BikramSambat> = OnceLock::new();

impl BikramSambat {
    /// The shipped calendar over the checked-in table.
    pub fn shipped() -> &'static BikramSambat {
        SHIPPED.get_or_init(|| BikramSambat::over(&generated::TABLE))
    }

    /// A calendar over a table.
    #[must_use]
    pub fn over(table: &'static Table) -> BikramSambat {
        let year_days = |months: &[u8; 12]| months.iter().map(|d| i64::from(*d)).sum::<i64>();
        let (anchor_year, (y, m, d)) = table.anchor;
        let anchor_day = fixed_from_gregorian(y, m, d);
        let anchor_index = usize::try_from(anchor_year - table.first_year).unwrap_or(0);
        let rows = table.month_lengths;
        let mut year_starts = vec![FixedDay::default(); rows.len() + 1];
        let mut start = anchor_day;
        for index in anchor_index..rows.len() {
            if let (Some(slot), Some(months)) = (year_starts.get_mut(index), rows.get(index)) {
                *slot = start;
                start = start.plus_days(year_days(months));
            }
        }
        if let Some(last) = year_starts.last_mut() {
            *last = start;
        }
        let mut start = anchor_day;
        for index in (0..anchor_index).rev() {
            if let (Some(months), Some(slot)) = (rows.get(index), year_starts.get_mut(index)) {
                start = start.plus_days(-year_days(months));
                *slot = start;
            }
        }
        BikramSambat { table, year_starts }
    }

    /// The table.
    #[must_use]
    pub const fn table(&self) -> &'static Table {
        self.table
    }

    /// The first and last year of the table.
    #[must_use]
    pub fn years(&self) -> (i32, i32) {
        let count = i32::try_from(self.table.month_lengths.len()).unwrap_or(0);
        (self.table.first_year, self.table.first_year + count - 1)
    }

    /// The official span.
    #[must_use]
    pub const fn official_years(&self) -> (i32, i32) {
        self.table.official
    }

    /// The official span's edition.
    #[must_use]
    pub const fn edition(&self) -> &'static str {
        self.table.edition
    }

    /// The frame the computed rows were made under.
    #[must_use]
    pub const fn frame(&self) -> &'static str {
        self.table.frame
    }

    /// The months inside the official span where the engine differs from
    /// the table: the public record of where computation and the official
    /// almanac disagree.
    #[must_use]
    pub const fn divergences(&self) -> &'static [Divergence] {
        self.table.divergences
    }

    /// The official rows: each year of the published span with its
    /// twelve month lengths, what a measurement compares an engine with.
    #[must_use]
    pub fn official_rows(&self) -> Vec<(i32, [u8; 12])> {
        let (first, last) = self.table.official;
        (first..=last)
            .filter_map(|year| self.row(year).ok().map(|months| (year, *months)))
            .collect()
    }

    /// The fixed day of 1 Baisakh of a year in the table.
    #[must_use]
    pub fn year_start(&self, year: i32) -> Option<FixedDay> {
        let index = usize::try_from(year - self.table.first_year).ok()?;
        if index >= self.table.month_lengths.len() {
            return None;
        }
        self.year_starts.get(index).copied()
    }

    fn row(&self, year: i32) -> Result<&'static [u8; 12], Error> {
        let (first, last) = self.years();
        usize::try_from(year - first)
            .ok()
            .and_then(|i| self.table.month_lengths.get(i))
            .ok_or_else(|| {
                out_of_range(format!(
                    "Bikram Sambat covers {first} to {last} BS, not {year}"
                ))
                .with_field("year")
            })
    }

    fn is_official(&self, year: i32) -> bool {
        year >= self.table.official.0 && year <= self.table.official.1
    }

    /// The month and day of a day offset into a year's rows, if inside.
    fn label(months: &[u8; 12], mut remaining: i64) -> Option<MonthDay> {
        if remaining < 0 {
            return None;
        }
        for (m, length) in months.iter().enumerate() {
            if remaining < i64::from(*length) {
                return Some(MonthDay {
                    month: u8::try_from(m + 1).ok()?,
                    day: u8::try_from(remaining + 1).ok()?,
                });
            }
            remaining -= i64::from(*length);
        }
        None
    }

    /// The resolution of a date: tabular inside the official span, unless
    /// the engine labels the day differently, which is reported;
    /// computed outside it.
    fn resolution(
        &self,
        year: i32,
        start: FixedDay,
        fixed: FixedDay,
        tabular: MonthDay,
    ) -> CalendarResolution {
        if !self.is_official(year) {
            return CalendarResolution::Computed {
                model: Cow::Borrowed(self.table.frame),
            };
        }
        let model_label = self
            .table
            .model_rows
            .iter()
            .find(|row| row.year == year)
            .and_then(|row| {
                let model_start = start.plus_days(i64::from(row.start_offset));
                BikramSambat::label(&row.months, model_start.days_until(fixed))
            });
        match model_label {
            Some(computed) if computed != tabular => CalendarResolution::Divergent {
                tabular,
                computed,
                model: Cow::Borrowed(self.table.frame),
            },
            _ => CalendarResolution::Tabular {
                authority: Cow::Borrowed(self.table.authority),
                edition: Cow::Borrowed(self.table.edition),
            },
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
        let after = || {
            out_of_range(format!(
                "{fixed} is after the last day of {last_year} BS, the last year of the table"
            ))
        };
        let (Some(start), Some(months)) = (
            self.year_starts.get(index),
            self.table.month_lengths.get(index),
        ) else {
            return Err(after());
        };
        let year = first_year + i32::try_from(index).unwrap_or(0);
        let tabular = BikramSambat::label(months, start.days_until(fixed)).ok_or_else(after)?;
        Ok(CalendarDate {
            calendar: Calendar::BikramSambat,
            year,
            month: tabular.month,
            day: tabular.day,
            era: Some(EraNumber {
                era: Era::Vikrama,
                year,
            }),
            resolution: self.resolution(year, *start, fixed, tabular),
        })
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
        let start = self.year_start(date.year).unwrap_or_default();
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

    use teistro_siddhanta::SuryaSiddhanta;
    use teistro_time::zones;

    use super::*;
    use crate::gregorian::{Gregorian, gregorian_from_fixed};

    #[test]
    fn the_epoch_and_known_new_years() {
        let bs = BikramSambat::shipped();
        assert_eq!(bs.years(), (FIRST_YEAR, LAST_YEAR));
        assert_eq!(
            bs.official_years(),
            (OFFICIAL_FIRST_YEAR, OFFICIAL_LAST_YEAR)
        );
        // 1 Baisakh 1970 BS is 13 April 1913, the table's anchor.
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
        // 2 Magh 1990 BS is 15 January 1934, the day of the great earthquake.
        assert_eq!(
            gregorian_from_fixed(bs.to_fixed_ymd(1990, 10, 2).unwrap()),
            (1934, 1, 15)
        );
        let day = Gregorian.to_fixed_ymd(2015, 4, 14).unwrap();
        let date = bs.date_of(day).unwrap();
        assert_eq!((date.year, date.month, date.day), (2072, 1, 1));
        assert!(matches!(
            date.resolution,
            CalendarResolution::Tabular { .. }
        ));
        let computed = bs
            .date_of(bs.to_fixed_ymd(1900, 5, 3).unwrap())
            .unwrap()
            .resolution;
        assert!(
            matches!(computed, CalendarResolution::Computed { ref model } if model == bs.frame())
        );
        assert_eq!(bs.month_length(2072, 1).unwrap(), 31);
        assert!(
            bs.to_fixed_ymd(2072, 1, 32)
                .unwrap_err()
                .message
                .contains("days 1 to 31")
        );
        assert!(
            bs.to_fixed_ymd(1600, 1, 1)
                .unwrap_err()
                .message
                .contains("covers")
        );
        assert!(bs.to_fixed_ymd(2072, 13, 1).is_err());
        assert!(bs.year_start(1600).is_none() && bs.year_start(LAST_YEAR + 1).is_none());
        assert_eq!(bs.year_start(1970), Some(epoch_1970));
        assert_eq!(bs.table().anchor.0, 1970);
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

    #[test]
    fn divergences_are_reported_and_only_where_the_table_says() {
        let bs = BikramSambat::shipped();
        let divergences = bs.divergences();
        // Every divergence names an official year and a real difference,
        // and has a model row to label dates by.
        for d in divergences {
            assert!(bs.is_official(d.year), "{d:?}");
            assert_ne!(d.tabular, d.computed, "{d:?}");
            assert!(
                bs.table().model_rows.iter().any(|row| row.year == d.year),
                "{d:?}"
            );
        }
        // A date in a month after a divergent boundary carries both labels.
        if let Some(first) = divergences.first() {
            let year = first.year;
            let mut found = false;
            let start = bs.year_start(year).unwrap();
            let end = bs.year_start(year + 1).unwrap();
            let mut day = start;
            while day < end {
                let date = bs.date_of(day).unwrap();
                if let CalendarResolution::Divergent {
                    tabular,
                    computed,
                    model,
                } = &date.resolution
                {
                    assert_eq!((tabular.month, tabular.day), (date.month, date.day));
                    assert_ne!(tabular, computed);
                    assert_eq!(model, bs.frame());
                    found = true;
                }
                day = day.plus_days(1);
            }
            assert!(found, "{year} has a divergence but no divergent date");
        }
        // A year without a divergence is tabular on every day.
        let clean = (OFFICIAL_FIRST_YEAR..=OFFICIAL_LAST_YEAR)
            .find(|y| !divergences.iter().any(|d| d.year == *y))
            .unwrap();
        let start = bs.year_start(clean).unwrap();
        for offset in [0, 100, 200, 300, 364] {
            let date = bs.date_of(start.plus_days(offset)).unwrap();
            assert!(
                matches!(date.resolution, CalendarResolution::Tabular { .. }),
                "{date}"
            );
        }
    }

    #[test]
    fn the_shipped_table_is_what_the_engine_computes_under_its_frame() {
        // The computed rows either side of the official span come from the
        // shipped frame; a sample of years must reproduce exactly, and the
        // official span's divergences must be the engine's.
        let bs = BikramSambat::shipped();
        let text = SuryaSiddhanta::text();
        let engine = Engine::new(&text, zones::nepal(), KATHMANDU, SHIPPED_RULE);
        assert_eq!(engine.describe(), bs.frame());
        for year in [
            FIRST_YEAR,
            1800,
            OFFICIAL_FIRST_YEAR - 1,
            OFFICIAL_LAST_YEAR + 1,
            2200,
            LAST_YEAR,
        ] {
            let row = engine.year(year).unwrap();
            let shipped: Vec<u8> = (1..=12)
                .map(|m| bs.month_length(year, m).unwrap())
                .collect();
            assert_eq!(row.months.to_vec(), shipped, "{year}");
        }
        let official: Vec<(i32, [u8; 12])> = (OFFICIAL_FIRST_YEAR..=OFFICIAL_LAST_YEAR)
            .map(|y| (y, *bs.row(y).unwrap()))
            .collect();
        let computed = engine
            .span(OFFICIAL_FIRST_YEAR, OFFICIAL_LAST_YEAR)
            .unwrap();
        let report = fit(
            &engine.describe(),
            &computed,
            &official,
            bs.year_start(OFFICIAL_FIRST_YEAR).unwrap(),
        );
        assert_eq!(report.divergences, bs.divergences());
        assert!(report.drift_within_a_day(), "{report}");
    }
}
