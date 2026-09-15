//! The Kalachakra dasha: a nakshatra-pada-seeded progression of signs
//! (`03-design/dasha-kernels.md`, measured in
//! `03-design/kalachakra-measured.md`).
//!
//! The Moon's nakshatra picks one of four pada tables and its pada one row
//! of nine signs; each sign runs its fixed years, the first only for the
//! part of the pada still ahead; and each mahadasha divides among the nine
//! by their years, beginning from its own place among them. Three cycles
//! run, the second and third the nine again.
//!
//! Every fork the sources read differently at is a knob, with the corpus's
//! recording engine's reading as its default: which table a nakshatra takes
//! (crux C54), how the balance is taken (C55), and what follows the ninth
//! mahadasha (C56). A temporal balance reads the Moon's time in its
//! nakshatra in quarters, as that engine does (C57), and nothing is built
//! below the antardashas (C58).

use serde::{Deserialize, Serialize};
use teistro_core::angle::Nas;
use teistro_core::catalogue::{Nakshatra, Rashi};
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::settings::{
    AfterCycle, Balance, KalachakraAfterNinth, KalachakraBalance, KalachakraMembership, YearLength,
};

use crate::balance::{BalanceAtBirth, Written, elapsed_in};
use crate::tree::{Birth, Path, Period, Timeline};

/// The signs of one pada row.
const NINE: usize = 9;
/// The cycles a Kalachakra runs.
const CYCLES: usize = 3;
/// The mahadashas of the three cycles.
const MAHADASHAS: usize = NINE * CYCLES;

/// Each sign's years, Aries to Pisces: its lord's share of the paramayus.
pub const SIGN_YEARS: [u8; 12] = [7, 16, 9, 21, 5, 9, 16, 7, 10, 4, 4, 10];

const fn row(signs: [u8; NINE]) -> [u8; NINE] {
    signs
}

/// The four pada tables, each four rows of nine sign indices (Aries 0),
/// which the corpus's recording engine and the published chakra tables read
/// agree on sign for sign.
const TABLES: [[[u8; NINE]; 4]; 4] = [
    // Savya, first table (Ashwini's).
    [
        row([0, 1, 2, 3, 4, 5, 6, 7, 8]),
        row([9, 10, 11, 7, 6, 5, 3, 4, 2]),
        row([1, 0, 11, 10, 9, 8, 0, 1, 2]),
        row([3, 4, 5, 6, 7, 8, 9, 10, 11]),
    ],
    // Savya, second table (Bharani's).
    [
        row([7, 6, 5, 3, 4, 2, 1, 0, 11]),
        row([10, 9, 8, 0, 1, 2, 3, 4, 5]),
        row([6, 7, 8, 9, 10, 11, 7, 6, 5]),
        row([3, 4, 2, 1, 0, 11, 10, 9, 8]),
    ],
    // Apasavya, first table (Rohini's).
    [
        row([8, 9, 10, 11, 0, 1, 2, 4, 3]),
        row([5, 6, 7, 11, 10, 9, 8, 7, 6]),
        row([5, 4, 3, 2, 1, 0, 8, 9, 10]),
        row([11, 0, 1, 2, 4, 3, 5, 6, 7]),
    ],
    // Apasavya, second table (Mrigashira's).
    [
        row([11, 10, 9, 8, 7, 6, 5, 4, 3]),
        row([2, 1, 0, 8, 9, 10, 11, 0, 1]),
        row([2, 4, 3, 5, 6, 7, 11, 10, 9]),
        row([8, 7, 6, 5, 4, 3, 2, 1, 0]),
    ],
];

/// The table each nakshatra takes in the corpus's recording engine's lists,
/// Ashwini to Revati.
const LISTED: [u8; 27] = [
    0, 1, 0, 2, 3, 3, 0, 1, 0, 2, 3, 3, 0, 1, 0, 2, 3, 3, 0, 1, 0, 2, 3, 3, 0, 1, 1,
];

/// The choices a Kalachakra is computed under.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct KalachakraRules {
    /// How the balance is measured.
    pub balance: Balance,
    /// The length of a year.
    pub year_length: YearLength,
    /// What is answered past the end of the three cycles.
    pub after_cycle: AfterCycle,
    /// Which pada table a nakshatra takes.
    pub membership: KalachakraMembership,
    /// How the balance at birth is taken.
    pub balance_of: KalachakraBalance,
    /// What follows the ninth mahadasha.
    pub after_ninth: KalachakraAfterNinth,
}

impl KalachakraRules {
    /// The rules the settings give the Kalachakra.
    #[must_use]
    pub fn of(settings: &teistro_core::settings::Settings) -> KalachakraRules {
        KalachakraRules {
            balance: settings.dasha.balance,
            year_length: settings
                .dasha
                .year_length
                .get(&teistro_core::catalogue::DashaSystem::Kalachakra)
                .copied()
                .unwrap_or(YearLength::Julian36525),
            after_cycle: settings.dasha.after_cycle,
            membership: settings.dasha.kalachakra_membership,
            balance_of: settings.dasha.kalachakra_balance,
            after_ninth: settings.dasha.kalachakra_after_ninth,
        }
    }
}

fn sign(index: u8) -> Rashi {
    Rashi::from_id(u16::from(index % 12)).unwrap_or(Rashi::Aries)
}

fn years(sign: Rashi) -> f64 {
    SIGN_YEARS
        .get(sign as usize)
        .map_or(0.0, |years| f64::from(*years))
}

/// The table a nakshatra takes under a membership.
fn table(nakshatra: u8, membership: KalachakraMembership) -> &'static [[u8; NINE]; 4] {
    let at = match membership {
        KalachakraMembership::Triad => {
            let savya = (nakshatra / 3) % 2 == 0;
            let middle = nakshatra % 3 == 1;
            match (savya, middle) {
                (true, false) => 0,
                (true, true) => 1,
                (false, false) => 2,
                (false, true) => 3,
            }
        }
        _ => LISTED.get(usize::from(nakshatra)).copied().unwrap_or(0),
    };
    TABLES.get(usize::from(at)).unwrap_or(&TABLES[0])
}

/// The nine signs a nakshatra's pada (from 0) runs, under a membership.
#[must_use]
pub fn pada_row(nakshatra: Nakshatra, pada: u8, membership: KalachakraMembership) -> [Rashi; NINE] {
    table(u8::try_from(nakshatra.id() % 27).unwrap_or(0), membership)
        .get(usize::from(pada.min(3)))
        .copied()
        .unwrap_or_default()
        .map(sign)
}

/// The Kalachakra of one birth.
#[derive(Clone, Debug, PartialEq)]
pub struct KalachakraDasha {
    rules: KalachakraRules,
    birth: JulianDay<Utc>,
    seed: Nakshatra,
    pada: u8,
    /// The three cycles' nine signs.
    nines: [[Rashi; NINE]; CYCLES],
    /// How many of the first cycle's signs the balance skipped.
    skipped: usize,
    balance: BalanceAtBirth,
    /// Where each running mahadasha begins, days after birth, and the end.
    offsets: [f64; MAHADASHAS + 1],
}

impl KalachakraDasha {
    /// The Kalachakra of a birth under its rules.
    ///
    /// # Errors
    ///
    /// A temporal balance with no Moon span, or one that does not hold the
    /// birth, named `moon_span`; a year length of no days, named
    /// `year_length`.
    pub fn new(birth: &Birth, rules: KalachakraRules) -> Result<KalachakraDasha, Error> {
        let year_days = rules.year_length.days();
        if year_days.is_nan() || year_days <= 0.0 {
            return Err(Error::invalid_arg("a year of no days").with_field("year_length"));
        }
        let moon = birth.moon;
        let seed = moon.nakshatra();
        let in_nakshatra = moon.in_nakshatra().get();
        let per_pada = Nas::PER_NAKSHATRA / 4;
        let pada = u8::try_from((in_nakshatra / per_pada).min(3)).unwrap_or(3);
        #[expect(
            clippy::cast_precision_loss,
            reason = "both are below a pada's nanoarcseconds, 1.2e13, far inside the 2^53 a double holds exactly"
        )]
        let spatial = (in_nakshatra % per_pada) as f64 / per_pada as f64;
        let elapsed = match rules.balance {
            Balance::Temporal => {
                // The Moon's time in its nakshatra, in quarters (crux C57).
                let quarters = elapsed_in(birth.instant, birth.span()?)? * 4.0;
                (quarters - quarters.floor()).clamp(0.0, 1.0)
            }
            _ => spatial,
        };

        let first = pada_row(seed, pada, rules.membership);
        let next = if rules.after_ninth == KalachakraAfterNinth::Repeat {
            first
        } else {
            let mut reversed = first;
            reversed.reverse();
            reversed
        };
        let nines = [first, next, first];

        let (skipped, first_years) = match rules.balance_of {
            KalachakraBalance::WholePada => {
                let total: f64 = first.iter().copied().map(years).sum();
                let mut gone = elapsed * total;
                let mut skipped = 0;
                while skipped + 1 < NINE && first.get(skipped).is_some_and(|s| gone >= years(*s)) {
                    gone -= first.get(skipped).copied().map_or(0.0, years);
                    skipped += 1;
                }
                (
                    skipped,
                    first.get(skipped).copied().map_or(0.0, years) - gone,
                )
            }
            _ => (
                0,
                (1.0 - elapsed) * first.first().copied().map_or(0.0, years),
            ),
        };
        let first_sign_years = first.get(skipped).copied().map_or(0.0, years);
        let days = first_years * year_days;

        let mut offsets = [0.0; MAHADASHAS + 1];
        let mut total = 0.0;
        for (k, end) in offsets.iter_mut().skip(1).enumerate() {
            let length = match k {
                0 => days,
                k if k + skipped < MAHADASHAS => {
                    let (cycle, place) = ((k + skipped) / NINE, (k + skipped) % NINE);
                    nines
                        .get(cycle)
                        .and_then(|nine| nine.get(place))
                        .copied()
                        .map_or(0.0, years)
                        * year_days
                }
                _ => 0.0,
            };
            total += length;
            *end = total;
        }
        Ok(KalachakraDasha {
            rules,
            birth: birth.instant,
            seed,
            pada,
            nines,
            skipped,
            balance: BalanceAtBirth {
                method: rules.balance,
                remaining: if first_sign_years > 0.0 {
                    first_years / first_sign_years
                } else {
                    0.0
                },
                days,
                written: Written::of(days, year_days),
            },
            offsets,
        })
    }

    /// The rules it runs under.
    #[must_use]
    pub const fn rules(&self) -> KalachakraRules {
        self.rules
    }

    /// The nakshatra the Moon stood in.
    #[must_use]
    pub const fn seed(&self) -> Nakshatra {
        self.seed
    }

    /// The Moon's pada in its nakshatra, from 0.
    #[must_use]
    pub const fn pada(&self) -> u8 {
        self.pada
    }

    /// The balance at birth.
    #[must_use]
    pub const fn balance(&self) -> BalanceAtBirth {
        self.balance
    }

    /// How many mahadashas run in a cycle of this birth: the 27 less any the
    /// balance skipped.
    fn running(&self) -> usize {
        MAHADASHAS - self.skipped
    }

    fn cycle_days(&self) -> f64 {
        self.offsets
            .get(self.running())
            .copied()
            .unwrap_or_default()
    }

    /// The sign at a running mahadasha's index, with its nine and its place.
    fn place(&self, index: usize) -> Option<(Rashi, [Rashi; NINE], usize)> {
        let at = index + self.skipped;
        let nine = *self.nines.get(at / NINE)?;
        let place = at % NINE;
        Some((*nine.get(place)?, nine, place))
    }
}

impl Timeline for KalachakraDasha {
    fn breadth(&self) -> usize {
        NINE
    }

    /// The mahadasha at `index` of `cycle`, or nothing past the end when the
    /// rules end it.
    fn mahadasha(&self, cycle: u32, index: usize) -> Option<Period> {
        if index >= self.running() || (cycle > 0 && self.rules.after_cycle == AfterCycle::End) {
            return None;
        }
        let (sign, _, _) = self.place(index)?;
        let base = self.birth.get() + f64::from(cycle) * self.cycle_days();
        Some(Period::of_sign(
            sign,
            sign.attributes().lord,
            Path::root(cycle, u8::try_from(index).ok()?),
            Interval::literal(
                base + self.offsets.get(index)?,
                base + self.offsets.get(index + 1)?,
            ),
        ))
    }

    fn mahadasha_at(&self, instant: f64) -> Option<Period> {
        let into = instant - self.birth.get();
        let cycle_days = self.cycle_days();
        if into < 0.0 || cycle_days <= 0.0 {
            return None;
        }
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a non-negative whole number of cycles, far below u32::MAX for any date"
        )]
        let cycle = (into / cycle_days).floor() as u32;
        let within = into - f64::from(cycle) * cycle_days;
        let index = self
            .offsets
            .iter()
            .zip(self.offsets.iter().skip(1))
            .take(self.running())
            .position(|(from, to)| *from <= within && within < *to)?;
        self.mahadasha(cycle, index)
    }

    /// Nine antardashas from the mahadasha's own place among its nine, each
    /// its sign's years over the nine's; nothing below them (crux C58).
    fn child(&self, parent: &Period, index: usize) -> Option<Period> {
        if parent.path.depth() != 1 || index >= NINE {
            return None;
        }
        let (_, nine, place) = self.place(usize::from(*parent.path.indices().first()?))?;
        let total: f64 = nine.iter().copied().map(years).sum();
        let whole = parent.interval;
        let share = |k: usize| -> f64 {
            (0..k)
                .map(|step| nine.get((place + step) % NINE).copied().map_or(0.0, years))
                .sum::<f64>()
        };
        let bound = |k: usize| match k {
            0 => whole.from.get(),
            NINE => whole.to.get(),
            k => whole.from.get() + whole.days() * share(k) / total,
        };
        let child = *nine.get((place + index) % NINE)?;
        Some(Period::of_sign(
            child,
            child.attributes().lord,
            parent.path.child(u8::try_from(index).ok()?)?,
            Interval::literal(bound(index), bound(index + 1)),
        ))
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index what they built"
    )]

    use teistro_core::quantity::Degrees;
    use teistro_core::settings::YearLength;

    use super::*;

    fn rules() -> KalachakraRules {
        KalachakraRules {
            balance: Balance::Spatial,
            year_length: YearLength::Julian36525,
            after_cycle: AfterCycle::End,
            membership: KalachakraMembership::Listed,
            balance_of: KalachakraBalance::FirstSign,
            after_ninth: KalachakraAfterNinth::Reverse,
        }
    }

    fn birth(degrees: f64) -> Birth {
        Birth {
            instant: JulianDay::literal(2_451_545.0),
            moon: Nas::from_degrees(Degrees::try_new(degrees).unwrap()),
            moon_span: None,
        }
    }

    #[test]
    fn every_table_row_holds_nine_signs_and_every_nakshatra_a_table() {
        for tables in TABLES {
            for pada in tables {
                assert!(pada.iter().all(|s| *s < 12));
            }
        }
        // Both memberships give each chakra's two tables to its own
        // nakshatras: savya tables to savya triads, apasavya to apasavya.
        for n in 0..27_u8 {
            let savya = (n / 3) % 2 == 0;
            for membership in [KalachakraMembership::Listed, KalachakraMembership::Triad] {
                let at = TABLES
                    .iter()
                    .position(|t| t == table(n, membership))
                    .unwrap();
                assert_eq!(at < 2, savya, "{n} {membership:?}");
            }
        }
        // The memberships differ exactly where the measurement says.
        let differ: Vec<u8> = (0..27)
            .filter(|&n| {
                table(n, KalachakraMembership::Listed) != table(n, KalachakraMembership::Triad)
            })
            .collect();
        assert_eq!(differ, [5, 11, 17, 23, 26]);
    }

    #[test]
    fn the_first_sign_runs_the_rest_of_the_pada_and_antardashas_partition_it() {
        // Ashwini's first pada, a quarter through: Aries runs three quarters of
        // its seven years, then Taurus its sixteen.
        let dasha = KalachakraDasha::new(&birth(360.0 / 108.0 / 4.0), rules()).unwrap();
        let mds: Vec<Period> = dasha.mahadashas().collect();
        assert_eq!(mds.len(), 27);
        assert_eq!(mds[0].sign, Some(Rashi::Aries));
        assert!((mds[0].interval.days() - 0.75 * 7.0 * 365.25).abs() < 1e-6);
        assert_eq!(mds[1].sign, Some(Rashi::Taurus));
        // The second cycle runs the nine reversed.
        assert_eq!(mds[9].sign, Some(Rashi::Sagittarius));
        let children: Vec<Period> = dasha.children(&mds[1]).collect();
        assert_eq!(children.len(), 9);
        assert_eq!(children[0].sign, Some(Rashi::Taurus), "from its own place");
        assert_eq!(children[8].sign, Some(Rashi::Aries), "round to the first");
        assert_eq!(children[0].interval.from, mds[1].interval.from);
        assert_eq!(children[8].interval.to, mds[1].interval.to);
        assert!(
            dasha.children(&children[0]).next().is_none(),
            "nothing below"
        );
    }

    #[test]
    fn the_whole_pada_balance_skips_the_signs_its_elapsed_span_covers() {
        // Ashwini's first pada, a tenth through: a tenth of its hundred years
        // covers Aries's seven, and three of Taurus's sixteen are gone.
        let whole = KalachakraRules {
            balance_of: KalachakraBalance::WholePada,
            ..rules()
        };
        let dasha = KalachakraDasha::new(&birth(360.0 / 108.0 / 10.0), whole).unwrap();
        let first = dasha.mahadashas().next().unwrap();
        assert_eq!(first.sign, Some(Rashi::Taurus));
        assert!((first.interval.days() - 13.0 * 365.25).abs() < 1e-6);
        assert_eq!(dasha.mahadashas().count(), 26);
        // The cyclic school runs the nine again in order.
        let repeat = KalachakraRules {
            after_ninth: KalachakraAfterNinth::Repeat,
            ..rules()
        };
        let again = KalachakraDasha::new(&birth(1.0), repeat).unwrap();
        assert_eq!(again.mahadasha(0, 9).unwrap().sign, Some(Rashi::Aries));
    }
}
