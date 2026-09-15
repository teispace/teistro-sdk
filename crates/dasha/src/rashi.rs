//! The sign-based (Jaimini) dashas as rows over the K-rashi kernel
//! (`03-design/dasha-kernels.md`, measured in
//! `03-design/rashi-dashas-measured.md`).
//!
//! A rashi dasha runs the twelve signs, each for a number of years, and
//! divides each into twelve equal antardashas. A system is where it starts,
//! the order it visits the signs in, and how long a sign runs, so each is a
//! [`RashiRow`] and not a module.
//!
//! **Two kinds of odd.** A sign's years count to its lord forward from an
//! odd-*footed* sign (by threes from Aries), while the periods run forward
//! from an odd sign (Aries, Gemini, Leo…). Collapsing the two agrees with a
//! reference for some charts and not others, so they are the distinct types
//! [`Footedness`] and [`Parity`].
//!
//! Where the schools teach another reading — the direction by the ninth
//! house (crux C49), antardashas from the next sign (C50), the stronger dual
//! lord (C51), Drig's order (C52), the start sign and a second cycle (C53) —
//! the row takes the corpus's, and the crux names the rival.

use teistro_core::catalogue::{DashaSystem, Dignity, Graha, Rashi};
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::settings::{AfterCycle, YearLength};

use crate::tree::{Path, Period, Timeline};

/// The signs, in the zodiac's order.
const SIGNS: usize = 12;

/// Whether a sign is odd by number: Aries, Gemini, Leo and the rest.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Parity {
    /// Aries, Gemini, Leo, Libra, Sagittarius, Aquarius.
    Odd,
    /// Taurus, Cancer, Virgo, Scorpio, Capricorn, Pisces.
    Even,
}

impl Parity {
    /// A sign's parity.
    #[must_use]
    pub const fn of(sign: Rashi) -> Parity {
        if (sign as usize) % 2 == 0 {
            Parity::Odd
        } else {
            Parity::Even
        }
    }
}

/// Whether a sign is odd-footed, by threes from Aries: the count from a
/// sign to its lord runs forward from an odd-footed one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Footedness {
    /// Aries to Gemini, and Libra to Sagittarius.
    OddFooted,
    /// Cancer to Virgo, and Capricorn to Pisces.
    EvenFooted,
}

impl Footedness {
    /// A sign's footedness.
    #[must_use]
    pub const fn of(sign: Rashi) -> Footedness {
        if ((sign as usize) / 3) % 2 == 0 {
            Footedness::OddFooted
        } else {
            Footedness::EvenFooted
        }
    }
}

/// Which way a sequence runs through the zodiac.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    /// Aries to Taurus.
    Forward,
    /// Taurus to Aries.
    Back,
}

impl Direction {
    /// Forward from an odd sign, back from an even one.
    #[must_use]
    pub const fn of(parity: Parity) -> Direction {
        match parity {
            Parity::Odd => Direction::Forward,
            Parity::Even => Direction::Back,
        }
    }
}

/// The sign at a zodiac position counted from Aries, taken round the twelve.
fn sign_at(position: usize) -> Rashi {
    u16::try_from(position % SIGNS)
        .ok()
        .and_then(Rashi::from_id)
        .unwrap_or(Rashi::Aries)
}

/// The sign `count` signs from `sign` in `direction`.
#[must_use]
pub fn step(sign: Rashi, direction: Direction, count: usize) -> Rashi {
    let count = count % SIGNS;
    sign_at(match direction {
        Direction::Forward => sign as usize + count,
        Direction::Back => sign as usize + SIGNS - count,
    })
}

/// The steps from `from` to `to`, forward, 0 to 11.
fn forward(from: Rashi, to: Rashi) -> usize {
    (to as usize + SIGNS - from as usize) % SIGNS
}

/// What a rashi dasha reads of a chart.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RashiChart {
    /// The lagna's sign.
    pub lagna: Rashi,
    /// The arudha lagna's sign.
    pub arudha_lagna: Rashi,
    /// The navamsa lagna's sign.
    pub navamsa_lagna: Rashi,
    /// Each graha's sign, Sun to Ketu.
    pub signs: [Rashi; 9],
    /// Each graha's dignity, Sun to Ketu.
    pub dignities: [Dignity; 9],
}

impl RashiChart {
    /// The sign a graha stands in.
    fn sign_of(&self, graha: Graha) -> Rashi {
        self.signs
            .get(graha as usize)
            .copied()
            .unwrap_or(Rashi::Aries)
    }

    /// A graha's dignity.
    fn dignity_of(&self, graha: Graha) -> Dignity {
        self.dignities
            .get(graha as usize)
            .copied()
            .unwrap_or(Dignity::Neutral)
    }
}

/// Each sign's first Jaimini lord, Aries to Pisces.
const FIRST_LORDS: [Graha; SIGNS] = [
    Graha::Mars,
    Graha::Venus,
    Graha::Mercury,
    Graha::Moon,
    Graha::Sun,
    Graha::Mercury,
    Graha::Venus,
    Graha::Ketu,
    Graha::Jupiter,
    Graha::Saturn,
    Graha::Rahu,
    Graha::Jupiter,
];

/// A sign's first Jaimini lord: Ketu for Scorpio and Rahu for Aquarius.
#[must_use]
pub fn first_lord(sign: Rashi) -> Graha {
    FIRST_LORDS
        .get(sign as usize)
        .copied()
        .unwrap_or(Graha::Mars)
}

/// A dual-lorded sign's second lord: Mars for Scorpio, Saturn for Aquarius.
#[must_use]
pub const fn second_lord(sign: Rashi) -> Option<Graha> {
    match sign {
        Rashi::Scorpio => Some(Graha::Mars),
        Rashi::Aquarius => Some(Graha::Saturn),
        _ => None,
    }
}

/// The stronger of a sign's lords: the one in a kendra from the sign when
/// only one is, else the first (crux C51 names the ladder other schools use).
#[must_use]
pub fn stronger_lord(chart: &RashiChart, sign: Rashi) -> Graha {
    let first = first_lord(sign);
    let Some(second) = second_lord(sign) else {
        return first;
    };
    let in_kendra = |graha: Graha| forward(sign, chart.sign_of(graha)) % 3 == 0;
    if in_kendra(second) && !in_kendra(first) {
        second
    } else {
        first
    }
}

/// The years from a sign to its lord: the signs counted forward from an
/// odd-footed sign and back from an even-footed one, twelve when the lord
/// is in the sign.
#[must_use]
pub fn counted_years(chart: &RashiChart, sign: Rashi, lord: Graha) -> u8 {
    let at = chart.sign_of(lord);
    let distance = match Footedness::of(sign) {
        Footedness::OddFooted => forward(sign, at),
        Footedness::EvenFooted => forward(at, sign),
    };
    if distance == 0 {
        12
    } else {
        u8::try_from(distance).unwrap_or(12)
    }
}

/// Where a system starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Start {
    /// The lagna.
    Lagna,
    /// The arudha lagna.
    ArudhaLagna,
    /// The navamsa lagna.
    NavamsaLagna,
}

/// The order a system visits the signs in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Order {
    /// Every sign in turn from the start, forward from an odd start and back
    /// from an even one.
    Consecutive,
    /// The trine groups from the start's, forward, each group's three signs
    /// forward from an odd start and reversed from an even one.
    TrineGroups,
    /// The ninth, tenth and eleventh houses from the start, each followed by
    /// the signs it aspects in the zodiac's order, then any sign left out in
    /// the zodiac's order (crux C52).
    DrishtiChain,
    /// Back two signs at a time from the start, six times, then the same
    /// from the sign before the start.
    Leap,
}

/// How long a sign's period runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Length {
    /// The count to the sign's stronger lord.
    CountToLord,
    /// The same, a year more when that lord is exalted and a year less when
    /// debilitated, from one year to twelve.
    CountToLordByDignity,
    /// The same years for every sign.
    Fixed(u8),
    /// By the sign's modality.
    ByModality {
        /// A movable sign's years.
        movable: u8,
        /// A fixed sign's years.
        fixed: u8,
        /// A dual sign's years.
        dual: u8,
    },
}

/// Which lord a mahadasha names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NamedLord {
    /// The stronger of a dual-lorded sign's two.
    Stronger,
    /// The first, Ketu or Rahu.
    First,
}

/// A sign-based system.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RashiRow {
    /// Which system the row is.
    pub system: DashaSystem,
    /// Where it starts.
    pub start: Start,
    /// The order it visits the signs in.
    pub order: Order,
    /// How long a sign's period runs.
    pub length: Length,
    /// Which lord a mahadasha names.
    pub named_lord: NamedLord,
}

impl RashiRow {
    /// The mahadasha signs for a chart, in order.
    #[must_use]
    pub fn sequence(&self, chart: &RashiChart) -> [Rashi; SIGNS] {
        let start = match self.start {
            Start::Lagna => chart.lagna,
            Start::ArudhaLagna => chart.arudha_lagna,
            Start::NavamsaLagna => chart.navamsa_lagna,
        };
        let direction = Direction::of(Parity::of(start));
        let mut out = [start; SIGNS];
        let mut fill = |signs: &mut dyn Iterator<Item = Rashi>| {
            for (slot, sign) in out.iter_mut().zip(signs) {
                *slot = sign;
            }
        };
        match self.order {
            Order::Consecutive => fill(&mut (0..SIGNS).map(|k| step(start, direction, k))),
            Order::TrineGroups => fill(&mut (0..4).flat_map(|g| {
                let base = sign_at((start as usize % 4 + g) % 4);
                let trine = [
                    base,
                    step(base, Direction::Forward, 4),
                    step(base, Direction::Forward, 8),
                ];
                match direction {
                    Direction::Forward => trine,
                    Direction::Back => [trine[2], trine[1], trine[0]],
                }
            })),
            Order::DrishtiChain => {
                let mut seen = [false; SIGNS];
                let anchors = [8, 9, 10].map(|house| step(start, Direction::Forward, house));
                let mut chain = anchors
                    .into_iter()
                    .flat_map(|anchor| core::iter::once(anchor).chain(aspected(anchor)))
                    .chain(Rashi::ALL)
                    .filter(|sign| {
                        seen.get_mut(*sign as usize)
                            .is_some_and(|s| !core::mem::replace(s, true))
                    });
                fill(&mut chain);
            }
            Order::Leap => fill(&mut (0..SIGNS).map(|k| {
                let seed = if k < 6 {
                    start
                } else {
                    step(start, Direction::Back, 1)
                };
                step(seed, Direction::Back, 2 * (k % 6))
            })),
        }
        out
    }

    /// A sign's period in years for a chart.
    #[must_use]
    pub fn years(&self, chart: &RashiChart, sign: Rashi) -> u8 {
        match self.length {
            Length::Fixed(years) => years,
            Length::ByModality {
                movable,
                fixed,
                dual,
            } => match (sign as usize) % 3 {
                0 => movable,
                1 => fixed,
                _ => dual,
            },
            Length::CountToLord => counted_years(chart, sign, stronger_lord(chart, sign)),
            Length::CountToLordByDignity => {
                let lord = stronger_lord(chart, sign);
                let counted = counted_years(chart, sign, lord);
                match chart.dignity_of(lord) {
                    Dignity::Exalted | Dignity::DeepExalted => (counted + 1).min(12),
                    Dignity::Debilitated | Dignity::DeepDebilitated => {
                        counted.saturating_sub(1).max(1)
                    }
                    _ => counted,
                }
            }
        }
    }

    /// The lord a mahadasha of `sign` names.
    #[must_use]
    pub fn lord(&self, chart: &RashiChart, sign: Rashi) -> Graha {
        match self.named_lord {
            NamedLord::Stronger => stronger_lord(chart, sign),
            NamedLord::First => first_lord(sign),
        }
    }
}

/// The signs a sign aspects by rashi drishti, in the zodiac's order: a
/// movable sign the fixed signs but the one after it, a fixed sign the
/// movable signs but the one before it, a dual sign the other duals.
#[must_use]
pub fn aspected(sign: Rashi) -> [Rashi; 3] {
    let target = match (sign as usize) % 3 {
        0 => 1,
        1 => 0,
        _ => 2,
    };
    let mut out = [sign; 3];
    let aspects = Rashi::ALL.into_iter().filter(|&other| {
        let adjacent = forward(sign, other) == 1 || forward(other, sign) == 1;
        other != sign && (other as usize) % 3 == target && !adjacent
    });
    for (slot, other) in out.iter_mut().zip(aspects) {
        *slot = other;
    }
    out
}

const fn row(
    system: DashaSystem,
    start: Start,
    order: Order,
    length: Length,
    named_lord: NamedLord,
) -> RashiRow {
    RashiRow {
        system,
        start,
        order,
        length,
        named_lord,
    }
}

/// Chara: every sign from the lagna, each counted to its stronger lord.
pub const CHARA: RashiRow = row(
    DashaSystem::Chara,
    Start::Lagna,
    Order::Consecutive,
    Length::CountToLord,
    NamedLord::Stronger,
);
/// Narayana: Chara with the lord's exaltation and debilitation.
pub const NARAYANA: RashiRow = row(
    DashaSystem::Narayana,
    Start::Lagna,
    Order::Consecutive,
    Length::CountToLordByDignity,
    NamedLord::Stronger,
);
/// Padanadhamsa: Chara from the arudha lagna.
pub const PADANADHAMSA: RashiRow = row(
    DashaSystem::Padanadhamsa,
    Start::ArudhaLagna,
    Order::Consecutive,
    Length::CountToLord,
    NamedLord::Stronger,
);
/// Trikona: the trine groups from the lagna's.
pub const TRIKONA: RashiRow = row(
    DashaSystem::Trikona,
    Start::Lagna,
    Order::TrineGroups,
    Length::CountToLord,
    NamedLord::Stronger,
);
/// Drig: the ninth, tenth and eleventh houses and what each aspects.
pub const DRIG: RashiRow = row(
    DashaSystem::Drig,
    Start::Lagna,
    Order::DrishtiChain,
    Length::CountToLord,
    NamedLord::Stronger,
);
/// Shoola: nine years a sign from the lagna.
pub const SHOOLA: RashiRow = row(
    DashaSystem::Shoola,
    Start::Lagna,
    Order::Consecutive,
    Length::Fixed(9),
    NamedLord::First,
);
/// Niryana Shoola: Shoola from the navamsa lagna.
pub const NIRYANA_SHOOLA: RashiRow = row(
    DashaSystem::NiryanaShoola,
    Start::NavamsaLagna,
    Order::Consecutive,
    Length::Fixed(9),
    NamedLord::First,
);
/// Mandooka: leaping back two signs, seven, eight or nine years by modality.
pub const MANDOOKA: RashiRow = row(
    DashaSystem::Mandooka,
    Start::Lagna,
    Order::Leap,
    Length::ByModality {
        movable: 7,
        fixed: 8,
        dual: 9,
    },
    NamedLord::Stronger,
);

/// Every sign-based row this build implements.
pub const RASHI_ROWS: &[RashiRow] = &[
    CHARA,
    NARAYANA,
    PADANADHAMSA,
    TRIKONA,
    DRIG,
    SHOOLA,
    NIRYANA_SHOOLA,
    MANDOOKA,
];

/// The sign-based row of a system, when this build implements one.
#[must_use]
pub fn rashi_row(system: DashaSystem) -> Option<&'static RashiRow> {
    RASHI_ROWS.iter().find(|row| row.system == system)
}

/// A sign-based dasha of one chart.
#[derive(Clone, Debug, PartialEq)]
pub struct RashiDasha {
    row: &'static RashiRow,
    chart: RashiChart,
    birth: JulianDay<Utc>,
    after_cycle: AfterCycle,
    signs: [Rashi; SIGNS],
    lords: [Graha; SIGNS],
    /// Where each mahadasha begins, days after birth, and the cycle's end.
    offsets: [f64; SIGNS + 1],
}

impl RashiDasha {
    /// The dasha of a chart under a row.
    ///
    /// # Errors
    ///
    /// A year length of no days, named `year_length`.
    pub fn new(
        row: &'static RashiRow,
        chart: &RashiChart,
        birth: JulianDay<Utc>,
        year_length: YearLength,
        after_cycle: AfterCycle,
    ) -> Result<RashiDasha, Error> {
        let year_days = year_length.days();
        if year_days.is_nan() || year_days <= 0.0 {
            return Err(Error::invalid_arg("a year of no days").with_field("year_length"));
        }
        let signs = row.sequence(chart);
        let mut lords = [Graha::Sun; SIGNS];
        let mut offsets = [0.0; SIGNS + 1];
        let mut total = 0.0;
        for ((lord, sign), end) in lords.iter_mut().zip(signs).zip(offsets.iter_mut().skip(1)) {
            *lord = row.lord(chart, sign);
            total += f64::from(row.years(chart, sign)) * year_days;
            *end = total;
        }
        Ok(RashiDasha {
            row,
            chart: *chart,
            birth,
            after_cycle,
            signs,
            lords,
            offsets,
        })
    }

    /// The row the dasha runs.
    #[must_use]
    pub const fn row(&self) -> &'static RashiRow {
        self.row
    }

    /// The chart it reads.
    #[must_use]
    pub const fn chart(&self) -> &RashiChart {
        &self.chart
    }

    /// The mahadasha signs, in order.
    #[must_use]
    pub const fn signs(&self) -> [Rashi; SIGNS] {
        self.signs
    }

    /// When the birth cycle ends.
    #[must_use]
    pub fn cycle_end(&self) -> JulianDay<Utc> {
        JulianDay::literal(self.birth.get() + self.cycle_days())
    }

    /// The mahadasha at `index` of `cycle`, or nothing past the end of the
    /// cycle when the rules end it.
    #[must_use]
    pub fn mahadasha(&self, cycle: u32, index: usize) -> Option<Period> {
        if index >= SIGNS || (cycle > 0 && self.after_cycle == AfterCycle::End) {
            return None;
        }
        let base = self.birth.get() + f64::from(cycle) * self.cycle_days();
        Some(Period::of_sign(
            *self.signs.get(index)?,
            *self.lords.get(index)?,
            Path::root(cycle, u8::try_from(index).ok()?),
            Interval::literal(
                base + self.offsets.get(index)?,
                base + self.offsets.get(index + 1)?,
            ),
        ))
    }

    /// A whole cycle's days.
    fn cycle_days(&self) -> f64 {
        self.offsets.last().copied().unwrap_or_default()
    }
}

impl Timeline for RashiDasha {
    fn breadth(&self) -> usize {
        SIGNS
    }

    fn mahadashas(&self) -> impl Iterator<Item = Period> + '_ {
        (0..SIGNS).filter_map(|index| self.mahadasha(0, index))
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
            .position(|(from, to)| *from <= within && within < *to)?;
        self.mahadasha(cycle, index)
    }

    /// Twelve equal antardashas from the period's own sign, forward from an
    /// odd sign and back from an even one, each naming its sign's first lord
    /// (crux C50 names the reading that begins from the next sign).
    fn child(&self, parent: &Period, index: usize) -> Option<Period> {
        let sign = parent.sign?;
        if index >= SIGNS {
            return None;
        }
        let place = u8::try_from(index).ok()?;
        let path = parent.path.child(place)?;
        let child = step(sign, Direction::of(Parity::of(sign)), index);
        let whole = parent.interval;
        let bound = |k: u8| match k {
            0 => whole.from.get(),
            12 => whole.to.get(),
            k => whole.from.get() + whole.days() * f64::from(k) / 12.0,
        };
        Some(Period::of_sign(
            child,
            first_lord(child),
            path,
            Interval::literal(bound(place), bound(place + 1)),
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

    use super::*;

    fn chart(lagna: Rashi) -> RashiChart {
        RashiChart {
            lagna,
            arudha_lagna: Rashi::Gemini,
            navamsa_lagna: Rashi::Aquarius,
            signs: [
                Rashi::Aries,
                Rashi::Scorpio,
                Rashi::Aquarius,
                Rashi::Aries,
                Rashi::Gemini,
                Rashi::Aquarius,
                Rashi::Capricorn,
                Rashi::Capricorn,
                Rashi::Cancer,
            ],
            dignities: [Dignity::Neutral; 9],
        }
    }

    #[test]
    fn footedness_and_parity_are_different_odds() {
        let odd_footed: Vec<Rashi> = Rashi::ALL
            .into_iter()
            .filter(|&s| Footedness::of(s) == Footedness::OddFooted)
            .collect();
        assert_eq!(
            odd_footed,
            [
                Rashi::Aries,
                Rashi::Taurus,
                Rashi::Gemini,
                Rashi::Libra,
                Rashi::Scorpio,
                Rashi::Sagittarius
            ]
        );
        assert_eq!(Parity::of(Rashi::Taurus), Parity::Even);
        assert_eq!(Footedness::of(Rashi::Taurus), Footedness::OddFooted);
    }

    #[test]
    fn every_order_visits_every_sign_once_from_every_start() {
        for row in RASHI_ROWS {
            for lagna in Rashi::ALL {
                let mut sequence = row.sequence(&RashiChart {
                    arudha_lagna: lagna,
                    navamsa_lagna: lagna,
                    ..chart(lagna)
                });
                sequence.sort();
                assert_eq!(sequence, Rashi::ALL, "{:?} from {lagna:?}", row.system);
            }
        }
    }

    #[test]
    fn every_length_is_one_to_twelve_years() {
        for row in RASHI_ROWS {
            for lagna in Rashi::ALL {
                let c = chart(lagna);
                for sign in Rashi::ALL {
                    let years = row.years(&c, sign);
                    assert!(
                        (1..=12).contains(&years),
                        "{:?} {sign:?}: {years}",
                        row.system
                    );
                }
            }
        }
    }

    #[test]
    fn a_sign_counts_to_its_lord_by_footedness_and_its_own_sign_is_twelve() {
        let c = chart(Rashi::Pisces);
        // Aries is odd-footed and Mars stands in Aquarius: ten signs on.
        assert_eq!(counted_years(&c, Rashi::Aries, Graha::Mars), 10);
        // Cancer is even-footed and the Moon in Scorpio: four signs back.
        assert_eq!(counted_years(&c, Rashi::Cancer, Graha::Moon), 8);
        // Saturn in Capricorn, its own sign.
        assert_eq!(counted_years(&c, Rashi::Capricorn, Graha::Saturn), 12);
        // Aquarius: Rahu and Saturn both in Capricorn, neither in a kendra.
        assert_eq!(stronger_lord(&c, Rashi::Aquarius), Graha::Rahu);
        // Scorpio: Mars in Aquarius is a kendra from it, Ketu in Cancer is not.
        assert_eq!(stronger_lord(&c, Rashi::Scorpio), Graha::Mars);
    }

    #[test]
    fn antardashas_divide_their_mahadasha_equally_from_its_own_sign() {
        let c = chart(Rashi::Pisces);
        let dasha = RashiDasha::new(
            &CHARA,
            &c,
            JulianDay::literal(2_451_545.0),
            YearLength::Julian36525,
            AfterCycle::End,
        )
        .unwrap();
        let first = dasha.mahadashas().next().unwrap();
        assert_eq!(first.sign, Some(Rashi::Pisces));
        let children: Vec<Period> = dasha.children(&first).collect();
        assert_eq!(children.len(), 12);
        assert_eq!(children[0].sign, Some(Rashi::Pisces));
        assert_eq!(
            children[1].sign,
            Some(Rashi::Aquarius),
            "Pisces is even: backwards"
        );
        assert_eq!(
            children[1].lord,
            Graha::Rahu,
            "an antardasha names the first lord"
        );
        assert_eq!(children[0].interval.from, first.interval.from);
        assert_eq!(children[11].interval.to, first.interval.to);
        assert!(
            dasha.mahadasha_at(dasha.cycle_end().get()).is_none(),
            "the cycle ends"
        );
    }
}
