//! The period tree of a nakshatra-seeded dasha, read without building it
//! (`03-design/dasha-kernels.md`, "The cursor").
//!
//! A tree six levels deep is 9⁶ periods a cycle, so nothing here builds
//! one. A period is found by its path, a period's children are computed
//! when asked for, and [`Dasha::at`] descends to the periods running at an
//! instant without allocating.
//!
//! **A boundary is its parent's start plus an exact share of its length.**
//! The share is whole years over the cycle's years, so every boundary is
//! one multiplication and one division from its parent's ends, the first
//! child starts on the parent's start and the last ends on the parent's
//! end: children partition their parent by construction, and no error
//! accumulates down the tree. The corpus's own boundaries agree to a
//! quarter of a millisecond and are not reproducible to the bit under any
//! order of float arithmetic (`03-design/dasha-measured.md`), so this is
//! chosen for its invariants and compared to the corpus's tolerance.

use core::fmt;

use teistro_core::angle::Nas;
use teistro_core::catalogue::{DashaSystem, Graha, Nakshatra, Rashi};
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{Depth, JulianDay, Utc};
use teistro_core::settings::{
    AfterCycle, AshtottariGrouping, Balance, BirthPeriod, SeedOverflow, YearLength,
};

use crate::balance::{BalanceAtBirth, Written, spatial, temporal};
use crate::row::{Seat, UduRow};

/// The deepest a path goes: the six named levels, from mahadasha to deha.
pub const MAX_DEPTH: usize = 6;

/// What a dasha is computed from: the birth and the Moon at it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Birth {
    /// The instant of birth.
    pub instant: JulianDay<Utc>,
    /// The Moon's sidereal longitude at it, exactly.
    pub moon: Nas,
    /// The Moon's stay in its nakshatra, from entering to leaving, which a
    /// temporal balance reads; nothing for a spatial one.
    pub moon_span: Option<Interval>,
}

impl Birth {
    /// The Moon's nakshatra span a temporal balance reads.
    ///
    /// # Errors
    ///
    /// No span, named `moon_span`.
    pub fn span(&self) -> Result<Interval, Error> {
        self.moon_span.ok_or_else(|| {
            Error::invalid_arg("a temporal balance reads the Moon's nakshatra span")
                .with_field("moon_span")
        })
    }
}

/// The choices a dasha is computed under, which the settings' `dasha`
/// group holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Rules {
    /// How the balance is measured.
    pub balance: Balance,
    /// The length of a year.
    pub year_length: YearLength,
    /// How the birth period is divided among its sub-periods.
    pub birth_period: BirthPeriod,
    /// What is answered past the end of the cycle.
    pub after_cycle: AfterCycle,
    /// What a seed outside a conditional cycle does.
    pub seed_overflow: SeedOverflow,
    /// How Ashtottari's lords share the nakshatras, which chooses its row.
    /// A document written before it was a choice ran the recording
    /// engine's, and reads back as that.
    #[serde(default = "three_each")]
    pub ashtottari_grouping: AshtottariGrouping,
}

const fn three_each() -> AshtottariGrouping {
    AshtottariGrouping::ThreeEach
}

impl Rules {
    /// The rules the settings' `dasha` group gives a system: its own year
    /// length, and the group's balance, birth period, cycle end and overflow.
    #[must_use]
    pub fn of(settings: &teistro_core::settings::Dasha, system: DashaSystem) -> Rules {
        Rules {
            balance: settings.balance,
            year_length: settings
                .year_length
                .get(&system)
                .copied()
                .unwrap_or(YearLength::Julian36525),
            birth_period: settings.birth_period,
            after_cycle: settings.after_cycle,
            seed_overflow: settings.seed_overflow,
            ashtottari_grouping: settings.ashtottari_grouping,
        }
    }

    /// The rules a consumer's system runs under: its definition's own year
    /// length, and the settings group's balance, birth period, cycle end and
    /// overflow, as a catalogued system takes them.
    #[must_use]
    pub fn of_definition(
        settings: &teistro_core::settings::Dasha,
        definition: &crate::definition::DashaDefinition,
    ) -> Rules {
        Rules {
            balance: settings.balance,
            year_length: definition.year_length(),
            birth_period: settings.birth_period,
            after_cycle: settings.after_cycle,
            seed_overflow: settings.seed_overflow,
            ashtottari_grouping: settings.ashtottari_grouping,
        }
    }
}

/// Where a period sits: the cycle it runs in, and its place among its
/// parent's children at every level.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Path {
    cycle: u32,
    indices: [u8; MAX_DEPTH],
    depth: u8,
}

impl Path {
    /// The mahadasha at `index` of `cycle`.
    #[must_use]
    pub const fn root(cycle: u32, index: u8) -> Path {
        let mut indices = [0; MAX_DEPTH];
        indices[0] = index;
        Path {
            cycle,
            indices,
            depth: 1,
        }
    }

    /// The cycle, 0 for the one birth falls in.
    #[must_use]
    pub const fn cycle(self) -> u32 {
        self.cycle
    }

    /// How many levels deep: 1 for a mahadasha.
    #[must_use]
    pub const fn depth(self) -> usize {
        self.depth as usize
    }

    /// The place at each level, from the mahadasha down. A child's place is
    /// its place in its parent's sequence, so under the elapsed reading the
    /// first child after birth may not be 0.
    #[must_use]
    pub fn indices(&self) -> &[u8] {
        self.indices.get(..self.depth()).unwrap_or(&[])
    }

    /// The child at `index`, or nothing at the deepest level.
    #[must_use]
    pub fn child(self, index: u8) -> Option<Path> {
        let mut path = self;
        *path.indices.get_mut(self.depth())? = index;
        path.depth += 1;
        Some(path)
    }
}

impl fmt::Display for Path {
    /// `0/2/5`, and `2:0/2/5` in a later cycle.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.cycle > 0 {
            write!(f, "{}:", self.cycle)?;
        }
        for (level, index) in self.indices().iter().enumerate() {
            if level > 0 {
                f.write_str("/")?;
            }
            write!(f, "{index}")?;
        }
        Ok(())
    }
}

impl fmt::Debug for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Path({self})")
    }
}

/// One period of the tree.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Period {
    /// Its lord.
    pub lord: Graha,
    /// The sign it is the period of, in a sign-based dasha; nothing in a
    /// nakshatra-seeded one, whose periods are their lords'.
    pub sign: Option<Rashi>,
    /// Where it sits.
    pub path: Path,
    /// When it runs.
    pub interval: Interval,
    /// The span its sub-periods are shares of: its own interval, except
    /// under the elapsed reading, where a period running at birth began
    /// before it.
    whole: Interval,
    /// Its lord's place in the row, which its children start from. Kept
    /// rather than looked up by graha, which a row naming one graha twice
    /// would make ambiguous.
    seat: usize,
}

impl Period {
    /// A period that runs its whole span, as every sign-based one does.
    pub(crate) const fn of_sign(
        sign: Rashi,
        lord: Graha,
        path: Path,
        interval: Interval,
    ) -> Period {
        Period {
            lord,
            sign: Some(sign),
            path,
            interval,
            whole: interval,
            seat: 0,
        }
    }

    /// A period of a year's ring (`annual`), which states its own span
    /// and the span its sub-periods divide.
    pub(crate) const fn of_share(
        lord: Graha,
        sign: Option<Rashi>,
        path: Path,
        interval: Interval,
        whole: Interval,
        seat: usize,
    ) -> Period {
        Period {
            lord,
            sign,
            path,
            interval,
            whole,
            seat,
        }
    }

    /// The span its sub-periods are shares of.
    #[must_use]
    pub const fn whole(&self) -> Interval {
        self.whole
    }
}

/// The periods running at an instant, from the mahadasha down.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Chain {
    periods: [Option<Period>; MAX_DEPTH],
    len: usize,
}

impl Chain {
    pub(crate) const EMPTY: Chain = Chain {
        periods: [None; MAX_DEPTH],
        len: 0,
    };

    pub(crate) fn push(&mut self, period: Period) {
        if let Some(slot) = self.periods.get_mut(self.len) {
            *slot = Some(period);
            self.len += 1;
        }
    }

    /// How many levels it holds; none before birth or past the cycle's end.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Whether no period runs.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The periods, from the mahadasha down.
    pub fn iter(&self) -> impl Iterator<Item = &Period> {
        self.periods.iter().take(self.len).flatten()
    }

    /// The deepest period, when one runs.
    #[must_use]
    pub fn deepest(&self) -> Option<&Period> {
        self.iter().last()
    }
}

/// A dasha's periods, read without building the tree: the mahadasha running
/// at an instant and a period's children, from which the chain at an
/// instant and a window's periods follow the same way for every kind of
/// dasha.
pub trait Timeline {
    /// How many children a period has at most.
    fn breadth(&self) -> usize;

    /// The mahadasha at `index` of `cycle`, 0 for the one birth falls in; nothing
    /// past the cycle's mahadashas, or past its end when the rules end it.
    fn mahadasha(&self, cycle: u32, index: usize) -> Option<Period>;

    /// The mahadashas of the birth cycle, in order.
    fn mahadashas(&self) -> impl Iterator<Item = Period> + '_ {
        (0..).map_while(|index| self.mahadasha(0, index))
    }

    /// The mahadasha running at a Julian day (UTC), with its cycle; nothing
    /// before birth, or past the cycle's end when the rules end it.
    fn mahadasha_at(&self, instant: f64) -> Option<Period>;

    /// The child of `parent` at `index` in its sequence, or nothing past
    /// the sequence or the deepest level, or when it is over before birth.
    fn child(&self, parent: &Period, index: usize) -> Option<Period>;

    /// The children of `parent` that run, in order.
    fn children<'a>(&'a self, parent: &'a Period) -> impl Iterator<Item = Period> + 'a {
        (0..self.breadth()).filter_map(move |index| self.child(parent, index))
    }

    /// The periods running at `instant`, from the mahadasha down to `depth`
    /// levels; empty before birth, and past the end of the cycle when the
    /// rules end it. Allocates nothing.
    fn at(&self, instant: JulianDay<Utc>, depth: Depth) -> Chain {
        let mut chain = Chain::EMPTY;
        let jd = instant.get();
        let Some(mut period) = self.mahadasha_at(jd) else {
            return chain;
        };
        chain.push(period);
        while chain.len() < usize::from(depth.get()) {
            let Some(next) = (0..self.breadth())
                .filter_map(|index| self.child(&period, index))
                .find(|child| child.interval.from.get() <= jd && jd < child.interval.to.get())
            else {
                break;
            };
            chain.push(next);
            period = next;
        }
        chain
    }

    /// Every period of the birth cycle to `depth` levels that overlaps
    /// `window`, depth first in time order: the tree materialised, pruned to
    /// the window, which is the cost a caller asks for by asking.
    fn periods(&self, window: Interval, depth: Depth) -> Vec<Period> {
        fn collect<T: Timeline + ?Sized>(
            timeline: &T,
            period: Period,
            window: Interval,
            depth: usize,
            out: &mut Vec<Period>,
        ) {
            if !period.interval.overlaps(window) {
                return;
            }
            out.push(period);
            if period.path.depth() < depth {
                for index in 0..timeline.breadth() {
                    if let Some(child) = timeline.child(&period, index) {
                        collect(timeline, child, window, depth, out);
                    }
                }
            }
        }
        let mut out = Vec::new();
        for maha in self.mahadashas() {
            collect(self, maha, window, usize::from(depth.get()), &mut out);
        }
        out
    }
}

/// A nakshatra-seeded dasha of one birth.
#[derive(Clone, Debug, PartialEq)]
pub struct Dasha {
    row: UduRow,
    rules: Rules,
    birth: JulianDay<Utc>,
    seed: Nakshatra,
    seat: Seat,
    balance: BalanceAtBirth,
    year_days: f64,
    /// Where each mahadasha of the birth cycle begins, and where the last
    /// ends: `mahadashas + 1` instants.
    first: Vec<f64>,
    /// The offset of each mahadasha into a whole cycle, and the cycle's
    /// length: `mahadashas + 1` day counts.
    full: Vec<f64>,
}

impl Dasha {
    /// The dasha of a birth under a row and its rules.
    ///
    /// # Errors
    ///
    /// A row its checks refuse; a seed outside a conditional cycle when the
    /// rules refuse one, named `seed_overflow`; a temporal balance with no
    /// Moon span, or one that does not hold the birth, named `moon_span`.
    pub fn new(row: &UduRow, birth: &Birth, rules: Rules) -> Result<Dasha, Error> {
        row.validate()?;
        let seed = birth.moon.nakshatra();
        let seat = row.seat(row.wheel.segment(birth.moon).place);
        if seat.overflow && rules.seed_overflow == SeedOverflow::Reject {
            return Err(Error::invalid_arg(format!(
                "the Moon's nakshatra is outside the nakshatras {} covers",
                row.system
            ))
            .with_field("seed_overflow")
            .with_hint("the WRAP_TO_START overflow starts such a seed at the first lord"));
        }
        let remaining = match rules.balance {
            Balance::Temporal => temporal(row, seat, birth.instant, birth.span()?)?,
            _ => spatial(row, birth.moon, seat),
        };
        let year_days = rules.year_length.days();
        let years = |at: usize| row.scaled_years(seat.lord + at);
        let days = remaining * years(0) * year_days;

        let mahadashas = row.mahadashas();
        let mut first = Vec::with_capacity(mahadashas + 1);
        let mut full = Vec::with_capacity(mahadashas + 1);
        let (mut at, mut offset) = (birth.instant.get(), 0.0);
        for index in 0..mahadashas {
            first.push(at);
            full.push(offset);
            let length = years(index) * year_days;
            at += if index == 0 { days } else { length };
            offset += length;
        }
        first.push(at);
        full.push(offset);

        Ok(Dasha {
            row: row.clone(),
            rules,
            birth: birth.instant,
            seed,
            seat,
            balance: BalanceAtBirth {
                method: rules.balance,
                remaining,
                days,
                written: Written::of(days, year_days),
            },
            year_days,
            first,
            full,
        })
    }

    /// The row the dasha runs.
    #[must_use]
    pub const fn row(&self) -> &UduRow {
        &self.row
    }

    /// The rules it runs under.
    #[must_use]
    pub const fn rules(&self) -> Rules {
        self.rules
    }

    /// The nakshatra the Moon stood in, which seeds it.
    #[must_use]
    pub const fn seed(&self) -> Nakshatra {
        self.seed
    }

    /// Where the Moon's nakshatra seats it: the first lord, and whether the
    /// seed overflowed a conditional cycle.
    #[must_use]
    pub const fn seat(&self) -> Seat {
        self.seat
    }

    /// The balance at birth.
    #[must_use]
    pub const fn balance(&self) -> BalanceAtBirth {
        self.balance
    }

    /// When the birth cycle ends.
    #[must_use]
    pub fn cycle_end(&self) -> JulianDay<Utc> {
        JulianDay::literal(self.first.last().copied().unwrap_or(self.birth.get()))
    }

    fn lords(&self) -> usize {
        self.row.lords.len()
    }

    /// The lord's place in the row at `index` round it.
    fn seat_of(&self, index: usize) -> usize {
        index % self.lords().max(1)
    }

    fn lord(&self, seat: usize) -> Option<Graha> {
        self.row.lords.get(seat).map(|lord| lord.graha)
    }

    fn cycle_days(&self) -> f64 {
        self.full.last().copied().unwrap_or_default()
    }
}

impl Timeline for Dasha {
    fn breadth(&self) -> usize {
        self.lords()
    }

    /// The mahadasha at `index` of `cycle`, or nothing past the end of the
    /// cycle when the rules end it.
    fn mahadasha(&self, cycle: u32, index: usize) -> Option<Period> {
        let mahadashas = self.row.mahadashas();
        if index >= mahadashas || (cycle > 0 && self.rules.after_cycle == AfterCycle::End) {
            return None;
        }
        let seat = self.seat_of(self.seat.lord + index);
        let lord = self.lord(seat)?;
        let path = Path::root(cycle, u8::try_from(index).ok()?);
        if cycle == 0 {
            let interval = Interval::literal(*self.first.get(index)?, *self.first.get(index + 1)?);
            let whole = if index == 0 && self.rules.birth_period == BirthPeriod::Elapsed {
                let length = self.row.scaled_years(self.seat.lord) * self.year_days;
                Interval::literal(interval.to.get() - length, interval.to.get())
            } else {
                interval
            };
            return Some(Period {
                lord,
                sign: None,
                path,
                interval,
                whole,
                seat,
            });
        }
        let start = self.first.get(mahadashas)? + f64::from(cycle - 1) * self.cycle_days();
        let interval = Interval::literal(
            start + self.full.get(index)?,
            start + self.full.get(index + 1)?,
        );
        Some(Period {
            lord,
            sign: None,
            path,
            interval,
            whole: interval,
            seat,
        })
    }

    fn child(&self, parent: &Period, index: usize) -> Option<Period> {
        let lords = self.lords();
        if index >= lords {
            return None;
        }
        let path = parent.path.child(u8::try_from(index).ok()?)?;
        let first = parent.seat;
        let total = self.row.total_years();
        let shares = |count: usize| -> u32 {
            self.row
                .lords
                .iter()
                .cycle()
                .skip(first)
                .take(count)
                .map(|lord| u32::from(lord.years))
                .sum()
        };
        let whole = parent.whole;
        let boundary = |share: u32| -> f64 {
            match share {
                0 => whole.from.get(),
                s if s == total => whole.to.get(),
                s => whole.from.get() + whole.days() * f64::from(s) / f64::from(total),
            }
        };
        let child_whole = Interval::literal(boundary(shares(index)), boundary(shares(index + 1)));
        if child_whole.to.get() <= parent.interval.from.get() {
            return None;
        }
        let interval = Interval::literal(
            child_whole.from.get().max(parent.interval.from.get()),
            child_whole.to.get(),
        );
        let seat = self.seat_of(first + index);
        Some(Period {
            lord: self.lord(seat)?,
            sign: None,
            path,
            interval,
            whole: child_whole,
            seat,
        })
    }

    /// The mahadasha running at an instant, with its cycle.
    fn mahadasha_at(&self, instant: f64) -> Option<Period> {
        if instant < self.birth.get() {
            return None;
        }
        let end = *self.first.get(self.row.mahadashas())?;
        let (cycle, offsets, base): (u32, &[f64], f64) = if instant < end {
            (0, &self.first, 0.0)
        } else {
            if self.rules.after_cycle == AfterCycle::End || self.cycle_days() <= 0.0 {
                return None;
            }
            let into = instant - end;
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "a non-negative whole number of cycles, far below u32::MAX for any date"
            )]
            let cycle = (into / self.cycle_days()).floor() as u32 + 1;
            (
                cycle,
                &self.full,
                end + f64::from(cycle - 1) * self.cycle_days(),
            )
        };
        let index = offsets
            .iter()
            .zip(offsets.iter().skip(1))
            .position(|(from, to)| base + from <= instant && instant < base + to)?;
        self.mahadasha(cycle, index)
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::float_cmp,
        reason = "tests fail by panicking, index their own fixtures and compare exact values"
    )]

    use teistro_core::quantity::Degrees;

    use super::*;
    use crate::row::VIMSHOTTARI;

    fn rules() -> Rules {
        Rules {
            balance: Balance::Spatial,
            year_length: YearLength::Julian36525,
            birth_period: BirthPeriod::Compressed,
            after_cycle: AfterCycle::End,
            seed_overflow: SeedOverflow::WrapToStart,
            ashtottari_grouping: teistro_core::settings::AshtottariGrouping::ThreeEach,
        }
    }

    /// A Moon a third of the way into Anuradha, Saturn's nakshatra.
    fn birth() -> Birth {
        let degrees = 16.0 * 360.0 / 27.0 + 360.0 / 81.0;
        Birth {
            instant: JulianDay::literal(2_447_995.5),
            moon: Nas::from_degrees(Degrees::try_new(degrees).unwrap()),
            moon_span: None,
        }
    }

    fn dasha(rules: Rules) -> Dasha {
        Dasha::new(&VIMSHOTTARI, &birth(), rules).unwrap()
    }

    #[test]
    fn the_first_period_runs_the_balance_and_the_rest_run_whole() {
        let dasha = dasha(rules());
        assert_eq!(dasha.seat().lord, 7, "Saturn");
        let balance = dasha.balance();
        assert!((balance.remaining - 2.0 / 3.0).abs() < 1e-12);
        assert!((balance.days - 2.0 / 3.0 * 19.0 * 365.25).abs() < 1e-9);
        let mahas: Vec<_> = dasha.mahadashas().collect();
        assert_eq!(mahas.len(), 9);
        assert_eq!(mahas[0].lord, Graha::Saturn);
        assert_eq!(mahas[1].lord, Graha::Mercury);
        assert_eq!(mahas[0].interval.from, birth().instant);
        assert!((mahas[1].interval.days() - 17.0 * 365.25).abs() < 1e-9);
        for pair in mahas.windows(2) {
            assert_eq!(pair[0].interval.to, pair[1].interval.from);
        }
    }

    #[test]
    fn children_partition_their_parent_exactly_at_every_level() {
        let dasha = dasha(rules());
        let mut period = dasha.mahadasha(0, 3).unwrap();
        for level in 2..=MAX_DEPTH {
            let kids: Vec<_> = dasha.children(&period).collect();
            assert_eq!(kids.len(), 9, "level {level}");
            assert_eq!(kids[0].lord, period.lord, "from its own lord");
            assert_eq!(kids[0].interval.from, period.interval.from);
            assert_eq!(kids[8].interval.to, period.interval.to);
            for pair in kids.windows(2) {
                assert_eq!(pair[0].interval.to, pair[1].interval.from);
            }
            period = kids[5];
        }
        assert_eq!(period.path.depth(), MAX_DEPTH);
        assert_eq!(dasha.child(&period, 0), None, "no level past the sixth");
    }

    #[test]
    fn the_birth_period_is_compressed_by_default_and_elapsed_on_request() {
        let compressed = dasha(rules());
        let maha = compressed.mahadasha(0, 0).unwrap();
        let kids: Vec<_> = compressed.children(&maha).collect();
        assert_eq!(kids.len(), 9);
        assert!((kids[0].interval.days() - maha.interval.days() * 19.0 / 120.0).abs() < 1e-9);

        let elapsed = dasha(Rules {
            birth_period: BirthPeriod::Elapsed,
            ..rules()
        });
        let maha = elapsed.mahadasha(0, 0).unwrap();
        assert!((maha.whole().days() - 19.0 * 365.25).abs() < 1e-9);
        assert_eq!(maha.interval, compressed.mahadasha(0, 0).unwrap().interval);
        let kids: Vec<_> = elapsed.children(&maha).collect();
        assert!(
            kids.len() < 9,
            "a third of the period was over before birth"
        );
        assert_eq!(
            kids[0].interval.from,
            birth().instant,
            "the running one is cut at birth"
        );
        assert!(
            kids[0].path.indices()[1] > 0,
            "and keeps its place in the sequence"
        );
        assert_eq!(kids.last().unwrap().interval.to, maha.interval.to);
        let last = kids.last().unwrap();
        // The sequence from Saturn ends on Jupiter's sixteen years.
        assert_eq!(last.lord, Graha::Jupiter);
        assert!((last.interval.days() - 19.0 * 365.25 * 16.0 / 120.0).abs() < 1e-6);
    }

    #[test]
    fn at_descends_to_the_running_periods_and_agrees_with_the_tree() {
        let dasha = dasha(rules());
        let depth = Depth::try_new(5).unwrap();
        let instant = JulianDay::literal(2_457_995.489_583_333_5);
        let chain = dasha.at(instant, depth);
        assert_eq!(chain.len(), 5);
        let mut parent: Option<Period> = None;
        for period in chain.iter() {
            assert!(period.interval.contains(instant));
            if let Some(parent) = parent {
                assert_eq!(
                    dasha.child(&parent, usize::from(*period.path.indices().last().unwrap())),
                    Some(*period)
                );
            }
            parent = Some(*period);
        }
        assert_eq!(chain.deepest().unwrap().path.depth(), 5);
        assert!(
            dasha.at(JulianDay::literal(2_447_995.0), depth).is_empty(),
            "before birth"
        );
    }

    #[test]
    fn the_cycle_ends_by_default_and_repeats_on_request() {
        let depth = Depth::try_new(2).unwrap();
        let ended = dasha(rules());
        let past = ended.cycle_end().plus_days(10.0).unwrap();
        assert!(ended.at(past, depth).is_empty());
        assert_eq!(ended.mahadasha(1, 0), None);

        let repeating = dasha(Rules {
            after_cycle: AfterCycle::Repeat,
            ..rules()
        });
        let chain = repeating.at(past, depth);
        assert_eq!(chain.len(), 2);
        let maha = chain.iter().next().unwrap();
        assert_eq!(
            (maha.path.cycle(), maha.lord),
            (1, Graha::Saturn),
            "from the first lord, in full"
        );
        assert!((maha.interval.days() - 19.0 * 365.25).abs() < 1e-9);
        assert_eq!(maha.interval.from, repeating.cycle_end());
        assert_eq!(maha.path.to_string(), "1:0");
    }

    #[test]
    fn a_window_materialises_only_what_overlaps_it() {
        let dasha = dasha(rules());
        let maha = dasha.mahadasha(0, 2).unwrap();
        let window = Interval::literal(
            maha.interval.from.get() + 1.0,
            maha.interval.from.get() + 2.0,
        );
        let found = dasha.periods(window, Depth::try_new(3).unwrap());
        assert!(found.iter().all(|p| p.interval.overlaps(window)));
        assert_eq!(found.iter().filter(|p| p.path.depth() == 1).count(), 1);
        assert_eq!(found.iter().filter(|p| p.path.depth() == 3).count(), 1);
    }

    #[test]
    fn a_temporal_balance_needs_the_moon_span_and_the_other_refusals_name_their_fields() {
        let temporal = Rules {
            balance: Balance::Temporal,
            ..rules()
        };
        let err = Dasha::new(&VIMSHOTTARI, &birth(), temporal).unwrap_err();
        assert_eq!(err.field(), Some("moon_span"));
        let with = Birth {
            moon_span: Some(Interval::literal(2_447_995.0, 2_447_996.0)),
            ..birth()
        };
        let dasha = Dasha::new(&VIMSHOTTARI, &with, temporal).unwrap();
        assert!((dasha.balance().remaining - 0.5).abs() < 1e-12);
        assert_eq!(dasha.balance().method, Balance::Temporal);
    }
}
