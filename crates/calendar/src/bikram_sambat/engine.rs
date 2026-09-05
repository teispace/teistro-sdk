//! The engine: a Bikram Sambat year from a solar model, a clock, a place
//! and a month-start rule. Year Y begins at the Mesha sankranti falling
//! in the Gregorian year Y − 57; each month begins at the next sankranti
//! placed on a civil day by the rule.

use core::fmt;

use teistro_core::error::Error;
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro_core::time::LocalClock;

use crate::fixed::FixedDay;
use crate::gregorian::fixed_from_gregorian;
use crate::solar::{MonthStartRule, SolarModel, find_sankranti};

/// Bikram Sambat year Y begins in Gregorian year Y − 57.
pub const YEAR_OFFSET: i32 = 57;

/// Kathmandu, the meridian the national calendar is reckoned for.
pub const KATHMANDU: Place = Place::new(
    Latitude::literal(27.7172),
    Longitude::literal(85.324),
    Altitude::literal(1400.0),
);

/// The days from one sankranti after which the next is searched for: the
/// shortest month is 29 days, so the search starts inside the month and
/// never skips a sign.
const NEXT_SEARCH_FROM_DAYS: f64 = 25.0;

/// The twelve sankrantis of a Bikram Sambat year and the civil day each
/// begins a month on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct YearRow {
    /// The Bikram Sambat year.
    pub year: i32,
    /// The first day, 1 Baisakh.
    pub start: FixedDay,
    /// The month lengths, Baisakh to Chaitra.
    pub months: [u8; 12],
    /// The sankranti instants, Mesha to Meena.
    pub sankrantis: [JulianDay<Utc>; 12],
}

impl YearRow {
    /// The days in the year.
    #[must_use]
    pub fn days(&self) -> u32 {
        self.months.iter().map(|m| u32::from(*m)).sum()
    }
}

/// The engine over a model, a clock, a place and a rule.
pub struct Engine<'a> {
    model: &'a dyn SolarModel,
    clock: &'a dyn LocalClock,
    place: Place,
    rule: MonthStartRule,
}

impl fmt::Debug for Engine<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.describe())
    }
}

impl<'a> Engine<'a> {
    /// An engine.
    #[must_use]
    pub const fn new(
        model: &'a dyn SolarModel,
        clock: &'a dyn LocalClock,
        place: Place,
        rule: MonthStartRule,
    ) -> Engine<'a> {
        Engine {
            model,
            clock,
            place,
            rule,
        }
    }

    /// The rule.
    #[must_use]
    pub const fn rule(&self) -> MonthStartRule {
        self.rule
    }

    /// The frame stamp: model, clock, place and rule, the text a computed
    /// date carries.
    #[must_use]
    pub fn describe(&self) -> String {
        format!(
            "{}; {}; {}; rule {}",
            self.model.describe(),
            self.clock.describe(),
            self.place,
            self.rule
        )
    }

    /// The thirteen sankrantis of a year: from its Mesha, in the Gregorian
    /// year `year - 57`, to the Mesha of the next year. The rules need no
    /// more, so a measurement finds these once and places them under every
    /// rule.
    ///
    /// # Errors
    ///
    /// The model's refusal or a search that does not converge.
    pub fn sankrantis(&self, year: i32) -> Result<[JulianDay<Utc>; 13], Error> {
        let mut instants = [JulianDay::<Utc>::J2000; 13];
        let mut from = fixed_from_gregorian(year - YEAR_OFFSET, 3, 1).jd_at_midnight()?;
        for (index, slot) in instants.iter_mut().enumerate() {
            let sign = u8::try_from(index % 12).unwrap_or(0);
            let found = find_sankranti(self.model, sign, from)?;
            *slot = found.instant;
            from = found.instant.plus_days(NEXT_SEARCH_FROM_DAYS)?;
        }
        Ok(instants)
    }

    /// A year.
    ///
    /// # Errors
    ///
    /// The model's refusal or a search that does not converge.
    pub fn year(&self, year: i32) -> Result<YearRow, Error> {
        let instants = self.sankrantis(year)?;
        self.year_from(year, &instants)
    }

    /// A year from its thirteen sankrantis, placed by the rule.
    ///
    /// # Errors
    ///
    /// A rule that needs a sunrise on a day without one.
    pub fn year_from(&self, year: i32, instants: &[JulianDay<Utc>; 13]) -> Result<YearRow, Error> {
        self.year_from_with(year, instants, &|_| self.rule)
    }

    /// A year from its thirteen sankrantis, each placed by the rule a
    /// function names for its sign (0 for Mesha), for measurements that
    /// try a rule on one sankranti at a time.
    ///
    /// # Errors
    ///
    /// As [`Engine::year_from`].
    pub fn year_from_with(
        &self,
        year: i32,
        instants: &[JulianDay<Utc>; 13],
        rule_for: &dyn Fn(u8) -> MonthStartRule,
    ) -> Result<YearRow, Error> {
        let mut starts = [FixedDay::default(); 13];
        for (index, (slot, instant)) in starts.iter_mut().zip(instants.iter()).enumerate() {
            let sign = u8::try_from(index % 12).unwrap_or(0);
            *slot =
                rule_for(sign).month_start(sign, *instant, self.clock, self.model, &self.place)?;
        }
        let mut months = [0u8; 12];
        for (index, length) in months.iter_mut().enumerate() {
            let (Some(this), Some(next)) = (starts.get(index), starts.get(index + 1)) else {
                continue;
            };
            *length = u8::try_from(this.days_until(*next)).map_err(|_| {
                Error::internal(format!(
                    "a month of {} days in {year} BS",
                    this.days_until(*next)
                ))
            })?;
        }
        let mut sankrantis = [JulianDay::<Utc>::J2000; 12];
        sankrantis.copy_from_slice(instants.get(..12).unwrap_or(&[]));
        Ok(YearRow {
            year,
            start: starts.first().copied().unwrap_or_default(),
            months,
            sankrantis,
        })
    }

    /// Every year of a span, in order.
    ///
    /// # Errors
    ///
    /// As [`Engine::year`], naming the first failing year.
    pub fn span(&self, first_year: i32, last_year: i32) -> Result<Vec<YearRow>, Error> {
        (first_year..=last_year)
            .map(|year| self.year(year))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use teistro_siddhanta::SuryaSiddhanta;
    use teistro_time::zones;

    use super::*;
    use crate::gregorian::gregorian_from_fixed;

    #[test]
    fn the_engine_computes_a_year_of_twelve_plausible_months() {
        let text = SuryaSiddhanta::text();
        let engine = Engine::new(
            &text,
            zones::nepal(),
            KATHMANDU,
            MonthStartRule::SankrantiDay,
        );
        let row = engine.year(2081).unwrap();
        assert_eq!(row.year, 2081);
        assert_eq!(gregorian_from_fixed(row.start), (2024, 4, 13));
        assert!(
            row.months.iter().all(|m| (29..=32).contains(m)),
            "{:?}",
            row.months
        );
        assert!(row.days() == 365 || row.days() == 366);
        assert!(
            row.sankrantis
                .windows(2)
                .all(|w| matches!(w, [a, b] if b.get() > a.get() + 28.0))
        );
        let span = engine.span(2080, 2082).unwrap();
        assert_eq!(span.len(), 3);
        let (first, second) = (span.first().unwrap(), span.get(1).unwrap());
        assert_eq!(
            first.start.plus_days(i64::from(first.days())),
            second.start,
            "years abut"
        );
        assert!(engine.describe().contains("SANKRANTI_DAY"));
        assert!(
            format!("{engine:?}").contains("Kathmandu") || engine.describe().contains("27.7172")
        );
    }
}
