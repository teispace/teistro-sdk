//! The engine: a solar year from a solar model, a clock, a place and a
//! month-start rule. A Bikram Sambat year Y begins at the Mesha sankranti
//! falling in the Gregorian year Y − 57; each month begins at the next
//! sankranti placed on a civil day by the rule. The regional solar
//! calendars run the same engine from their own first sign and era
//! ([`YearStart`], `crate::regional_solar`).

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

/// The days a year's first search moves on per sign past Mesha: a month
/// less a little, so the search still starts before the sankranti.
const SEARCH_DAYS_PER_SIGN: i64 = 30;

/// The twelve sankrantis of a solar year and the civil day each begins a
/// month on.
///
/// ```
/// # use teistro_calendar::MonthStartRule;
/// # use teistro_calendar::bikram_sambat::engine::{Engine, KATHMANDU};
/// # use teistro_core::time::UtcOffset;
/// # use teistro_siddhanta::SuryaSiddhanta;
/// # let (text, clock) = (SuryaSiddhanta::text(), UtcOffset::literal(5, 45, 0));
/// // The frame the shipped table is computed in: the text's Sun, Nepal's
/// // clock and Kathmandu, months placed by punya-kala.
/// let engine = Engine::new(&text, &clock, KATHMANDU, MonthStartRule::Punyakala);
/// use teistro_calendar::gregorian::gregorian_from_fixed;
///
/// let row = engine.year(2081)?;
/// assert_eq!(gregorian_from_fixed(row.start), (2024, 4, 13));
/// assert_eq!(row.months, [31, 32, 31, 32, 31, 30, 30, 30, 29, 30, 29, 31]);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct YearRow {
    /// The year, as its era counts it (Bikram Sambat for the engine's
    /// default [`YearStart`]).
    pub year: i32,
    /// The first day of the year (1 Baisakh for Bikram Sambat).
    pub start: FixedDay,
    /// The month lengths, from the year's first month (Baisakh to Chaitra
    /// for Bikram Sambat).
    pub months: [u8; 12],
    /// The sankranti instants, from the year's first sign (Mesha to Meena
    /// for Bikram Sambat).
    pub sankrantis: [JulianDay<Utc>; 12],
}

impl YearRow {
    /// The days in the year.
    ///
    /// ```
    /// # use teistro_calendar::MonthStartRule;
    /// # use teistro_calendar::bikram_sambat::engine::{Engine, KATHMANDU};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_siddhanta::SuryaSiddhanta;
    /// # let (text, clock) = (SuryaSiddhanta::text(), UtcOffset::literal(5, 45, 0));
    /// // The frame the shipped table is computed in: the text's Sun, Nepal's
    /// // clock and Kathmandu, months placed by punya-kala.
    /// let engine = Engine::new(&text, &clock, KATHMANDU, MonthStartRule::Punyakala);
    ///
    /// assert_eq!(engine.year(2081)?.days(), 366);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn days(&self) -> u32 {
        self.months.iter().map(|m| u32::from(*m)).sum()
    }
}

/// Where a solar year begins and how it is numbered: the sign whose
/// sankranti opens it, and how far the Gregorian year that sankranti
/// falls in is ahead of the year's own number.
///
/// ```
/// use teistro_calendar::bikram_sambat::engine::YearStart;
///
/// // Bikram Sambat 2081 began at the Mesha sankranti of 2024.
/// assert_eq!(YearStart::BIKRAM_SAMBAT.gregorian_year(2081), 2024);
/// // A year opening at Simha counts its months from Simha.
/// let simha = YearStart { sign: 4, gregorian_offset: 824 };
/// assert_eq!((simha.sign_of(0), simha.sign_of(8), simha.sign_of(12)), (4, 0, 4));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct YearStart {
    /// The sign whose sankranti begins the year, 0 for Mesha.
    pub sign: u8,
    /// The Gregorian year the opening sankranti falls in, less the year's
    /// number.
    pub gregorian_offset: i32,
}

impl YearStart {
    /// Bikram Sambat's: Mesha, in the Gregorian year Y − 57.
    pub const BIKRAM_SAMBAT: YearStart = YearStart {
        sign: 0,
        gregorian_offset: -YEAR_OFFSET,
    };

    /// The Gregorian year a year's opening sankranti falls in.
    ///
    /// ```
    /// use teistro_calendar::bikram_sambat::engine::YearStart;
    ///
    /// assert_eq!(YearStart { sign: 0, gregorian_offset: 78 }.gregorian_year(1946), 2024);
    /// ```
    #[must_use]
    pub const fn gregorian_year(self, year: i32) -> i32 {
        year + self.gregorian_offset
    }

    /// The sign of a year's month counted from 0, wrapping past Meena.
    ///
    /// ```
    /// use teistro_calendar::bikram_sambat::engine::YearStart;
    ///
    /// assert_eq!(YearStart::BIKRAM_SAMBAT.sign_of(11), 11);
    /// ```
    #[must_use]
    pub fn sign_of(self, index: usize) -> u8 {
        u8::try_from((usize::from(self.sign) + index % 12) % 12).unwrap_or(0)
    }
}

/// The engine over a model, a clock, a place and a rule.
///
/// ```
/// # use teistro_calendar::MonthStartRule;
/// # use teistro_calendar::bikram_sambat::engine::{Engine, KATHMANDU};
/// # use teistro_core::time::UtcOffset;
/// # use teistro_siddhanta::SuryaSiddhanta;
/// # let (text, clock) = (SuryaSiddhanta::text(), UtcOffset::literal(5, 45, 0));
/// // The frame the shipped table is computed in: the text's Sun, Nepal's
/// // clock and Kathmandu, months placed by punya-kala.
/// let engine = Engine::new(&text, &clock, KATHMANDU, MonthStartRule::Punyakala);
/// use teistro_calendar::BikramSambat;
///
/// // Computed, 2081 is the year the committee printed.
/// let official = BikramSambat::shipped().official_rows();
/// let printed = official.iter().find(|(year, _)| *year == 2081).unwrap().1;
/// assert_eq!(engine.year(2081)?.months, printed);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
pub struct Engine<'a> {
    model: &'a dyn SolarModel,
    clock: &'a dyn LocalClock,
    place: Place,
    rule: MonthStartRule,
    start: YearStart,
}

impl fmt::Debug for Engine<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.describe())
    }
}

impl<'a> Engine<'a> {
    /// An engine.
    ///
    /// ```
    /// # use teistro_calendar::MonthStartRule;
    /// # use teistro_calendar::bikram_sambat::engine::{Engine, KATHMANDU};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_siddhanta::SuryaSiddhanta;
    /// # let (text, clock) = (SuryaSiddhanta::text(), UtcOffset::literal(5, 45, 0));
    /// // The frame the shipped table is computed in: the text's Sun, Nepal's
    /// // clock and Kathmandu, months placed by punya-kala.
    /// let engine = Engine::new(&text, &clock, KATHMANDU, MonthStartRule::Punyakala);
    ///
    /// assert_eq!(engine.rule(), MonthStartRule::Punyakala);
    /// ```
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
            start: YearStart::BIKRAM_SAMBAT,
        }
    }

    /// The engine with another year start: a regional solar calendar's
    /// first sign and era in place of Bikram Sambat's.
    ///
    /// ```
    /// # use teistro_calendar::MonthStartRule;
    /// # use teistro_calendar::bikram_sambat::engine::{Engine, KATHMANDU, YearStart};
    /// # use teistro_calendar::gregorian::gregorian_from_fixed;
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_siddhanta::SuryaSiddhanta;
    /// # let (text, clock) = (SuryaSiddhanta::text(), UtcOffset::literal(5, 30, 0));
    /// // A year opening at the Simha sankranti, numbered 824 behind the
    /// // Gregorian year it begins in.
    /// let simha = YearStart { sign: 4, gregorian_offset: 824 };
    /// let engine = Engine::new(&text, &clock, KATHMANDU, MonthStartRule::SankrantiDay).with_year_start(simha);
    /// assert_eq!(engine.year_start(), simha);
    /// let row = engine.year(1200)?;
    /// assert_eq!(gregorian_from_fixed(row.start).0, 2024);
    /// assert_eq!(gregorian_from_fixed(row.start).1, 8);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub const fn with_year_start(mut self, start: YearStart) -> Engine<'a> {
        self.start = start;
        self
    }

    /// The year start.
    ///
    /// ```
    /// # use teistro_calendar::MonthStartRule;
    /// # use teistro_calendar::bikram_sambat::engine::{Engine, KATHMANDU, YearStart};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_siddhanta::SuryaSiddhanta;
    /// # let (text, clock) = (SuryaSiddhanta::text(), UtcOffset::literal(5, 45, 0));
    /// let engine = Engine::new(&text, &clock, KATHMANDU, MonthStartRule::Punyakala);
    ///
    /// assert_eq!(engine.year_start(), YearStart::BIKRAM_SAMBAT);
    /// ```
    #[must_use]
    pub const fn year_start(&self) -> YearStart {
        self.start
    }

    /// The rule.
    ///
    /// ```
    /// # use teistro_calendar::MonthStartRule;
    /// # use teistro_calendar::bikram_sambat::engine::{Engine, KATHMANDU};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_siddhanta::SuryaSiddhanta;
    /// # let (text, clock) = (SuryaSiddhanta::text(), UtcOffset::literal(5, 45, 0));
    /// // The frame the shipped table is computed in: the text's Sun, Nepal's
    /// // clock and Kathmandu, months placed by punya-kala.
    /// let engine = Engine::new(&text, &clock, KATHMANDU, MonthStartRule::Punyakala);
    ///
    /// assert!(engine.rule().needs_day_arc());
    /// ```
    #[must_use]
    pub const fn rule(&self) -> MonthStartRule {
        self.rule
    }

    /// The frame stamp: model, clock, place and rule, the text a computed
    /// date carries.
    ///
    /// ```
    /// # use teistro_calendar::MonthStartRule;
    /// # use teistro_calendar::bikram_sambat::engine::{Engine, KATHMANDU};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_siddhanta::SuryaSiddhanta;
    /// # let (text, clock) = (SuryaSiddhanta::text(), UtcOffset::literal(5, 45, 0));
    /// // The frame the shipped table is computed in: the text's Sun, Nepal's
    /// // clock and Kathmandu, months placed by punya-kala.
    /// let engine = Engine::new(&text, &clock, KATHMANDU, MonthStartRule::Punyakala);
    ///
    /// assert_eq!(
    ///     engine.describe(),
    ///     "Surya Siddhanta (the text, the sine table); +05:45; 27.7172°N 85.324°E 1400 m; rule PUNYAKALA"
    /// );
    /// ```
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

    /// The thirteen sankrantis of a year: from its first sign's, in the
    /// Gregorian year [`YearStart::gregorian_year`] names (Mesha in
    /// `year - 57` for Bikram Sambat), to the next year's. The rules need
    /// no more, so a measurement finds these once and places them under
    /// every rule.
    ///
    /// ```
    /// # use teistro_calendar::MonthStartRule;
    /// # use teistro_calendar::bikram_sambat::engine::{Engine, KATHMANDU};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_siddhanta::SuryaSiddhanta;
    /// # let (text, clock) = (SuryaSiddhanta::text(), UtcOffset::literal(5, 45, 0));
    /// // The frame the shipped table is computed in: the text's Sun, Nepal's
    /// // clock and Kathmandu, months placed by punya-kala.
    /// let engine = Engine::new(&text, &clock, KATHMANDU, MonthStartRule::Punyakala);
    ///
    /// let instants = engine.sankrantis(2081)?;
    /// // Mesha to Mesha: a sidereal year of the text's, 365.26 days.
    /// let year = instants[12].get() - instants[0].get();
    /// assert!((365.25..365.27).contains(&year), "{year}");
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// The model's refusal or a search that does not converge.
    pub fn sankrantis(&self, year: i32) -> Result<[JulianDay<Utc>; 13], Error> {
        let mut instants = [JulianDay::<Utc>::J2000; 13];
        // From 1 March for Mesha, a month on for each later first sign:
        // inside the month before the sankranti in every century the
        // calendars cover, so the search finds this year's.
        let from_day = fixed_from_gregorian(self.start.gregorian_year(year), 3, 1)
            .plus_days(i64::from(self.start.sign) * SEARCH_DAYS_PER_SIGN);
        let mut from = from_day.jd_at_midnight()?;
        for (index, slot) in instants.iter_mut().enumerate() {
            let sign = self.start.sign_of(index);
            let found = find_sankranti(self.model, sign, from)?;
            *slot = found.instant;
            from = found.instant.plus_days(NEXT_SEARCH_FROM_DAYS)?;
        }
        Ok(instants)
    }

    /// A year.
    ///
    /// ```
    /// # use teistro_calendar::MonthStartRule;
    /// # use teistro_calendar::bikram_sambat::engine::{Engine, KATHMANDU};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_siddhanta::SuryaSiddhanta;
    /// # let (text, clock) = (SuryaSiddhanta::text(), UtcOffset::literal(5, 45, 0));
    /// // The frame the shipped table is computed in: the text's Sun, Nepal's
    /// // clock and Kathmandu, months placed by punya-kala.
    /// let engine = Engine::new(&text, &clock, KATHMANDU, MonthStartRule::Punyakala);
    ///
    /// assert_eq!(engine.year(2081)?.year, 2081);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
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
    /// ```
    /// # use teistro_calendar::MonthStartRule;
    /// # use teistro_calendar::bikram_sambat::engine::{Engine, KATHMANDU};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_siddhanta::SuryaSiddhanta;
    /// # let (text, clock) = (SuryaSiddhanta::text(), UtcOffset::literal(5, 45, 0));
    /// // The frame the shipped table is computed in: the text's Sun, Nepal's
    /// // clock and Kathmandu, months placed by punya-kala.
    /// let engine = Engine::new(&text, &clock, KATHMANDU, MonthStartRule::Punyakala);
    ///
    /// // Found once, placed by any rule.
    /// let instants = engine.sankrantis(2081)?;
    /// assert_eq!(engine.year_from(2081, &instants)?, engine.year(2081)?);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
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
    /// ```
    /// # use teistro_calendar::MonthStartRule;
    /// # use teistro_calendar::bikram_sambat::engine::{Engine, KATHMANDU};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_siddhanta::SuryaSiddhanta;
    /// # let (text, clock) = (SuryaSiddhanta::text(), UtcOffset::literal(5, 45, 0));
    /// // The frame the shipped table is computed in: the text's Sun, Nepal's
    /// // clock and Kathmandu, months placed by punya-kala.
    /// let engine = Engine::new(&text, &clock, KATHMANDU, MonthStartRule::Punyakala);
    ///
    /// // Every sign under punya-kala is the year the engine's own rule gives.
    /// let instants = engine.sankrantis(2081)?;
    /// let each = engine.year_from_with(2081, &instants, &|_| MonthStartRule::Punyakala)?;
    /// assert_eq!(each, engine.year(2081)?);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
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
            let sign = self.start.sign_of(index);
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
                    "a month of {} days in year {year}",
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
    /// ```
    /// # use teistro_calendar::MonthStartRule;
    /// # use teistro_calendar::bikram_sambat::engine::{Engine, KATHMANDU};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_siddhanta::SuryaSiddhanta;
    /// # let (text, clock) = (SuryaSiddhanta::text(), UtcOffset::literal(5, 45, 0));
    /// // The frame the shipped table is computed in: the text's Sun, Nepal's
    /// // clock and Kathmandu, months placed by punya-kala.
    /// let engine = Engine::new(&text, &clock, KATHMANDU, MonthStartRule::Punyakala);
    ///
    /// let years = engine.span(2081, 2083)?;
    /// assert_eq!(years.iter().map(|row| row.year).collect::<Vec<_>>(), [2081, 2082, 2083]);
    /// // Each year begins where the one before ended.
    /// assert_eq!(years[0].start.plus_days(i64::from(years[0].days())), years[1].start);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
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
#[cfg_attr(coverage_nightly, coverage(off))]
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
