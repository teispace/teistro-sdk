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
//!
//! **Where BPHS decides, the text is the default.** Chapter 46 vv. 158 to 166
//! give a sign's strength — a sign holding an exalted graha, then more grahas,
//! then a dual over a fixed over a movable sign — which settles the stronger
//! lord of Scorpio and Aquarius (C51), and vv. 179 to 184 start Mandooka,
//! Shoola and Trikona from the stronger of their signs (C53). [`RashiRules`]
//! carries both, and its [`RashiRules::RECORDING_ENGINE`] is the corpus's
//! reading.

use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{DashaSystem, Dignity, Graha, Rashi};
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{Depth, JulianDay, Utc};
use teistro_core::settings::{AfterCycle, DualLord, RashiStart, YearLength};

use crate::row::{DashaName, julian, three};
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
    /// The sign the Brahma graha occupies, which the Sthira dasa starts
    /// from; `None` where the chart's rule finds no Brahma
    /// ([`crate::jaimini::brahma`]).
    pub brahma: Option<Rashi>,
}

impl RashiChart {
    /// The sign a graha stands in.
    pub(crate) fn sign_of(&self, graha: Graha) -> Rashi {
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

/// The rashi dashas' two readings BPHS and the recording engine part on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct RashiRules {
    /// How the stronger lord of Scorpio and Aquarius is found (crux C51).
    pub dual_lord: DualLord,
    /// Where the systems that start from a stronger sign begin (crux C53).
    pub start: RashiStart,
}

impl RashiRules {
    /// BPHS ch. 46's readings, the settings' defaults.
    pub const BPHS: RashiRules = RashiRules {
        dual_lord: DualLord::Bphs,
        start: RashiStart::Stronger,
    };

    /// The conformance corpus's recording engine's readings.
    pub const RECORDING_ENGINE: RashiRules = RashiRules {
        dual_lord: DualLord::Kendra,
        start: RashiStart::Lagna,
    };

    /// The readings the settings' `dasha` group gives.
    #[must_use]
    pub const fn of(settings: &teistro_core::settings::Settings) -> RashiRules {
        RashiRules {
            dual_lord: settings.dasha.dual_lord,
            start: settings.dasha.rashi_start,
        }
    }
}

/// How many of the nine grahas stand in a sign.
fn occupants(chart: &RashiChart, sign: Rashi) -> usize {
    chart.signs.iter().filter(|at| **at == sign).count()
}

/// Whether an exalted graha stands in a sign.
fn holds_exalted(chart: &RashiChart, sign: Rashi) -> bool {
    chart
        .signs
        .iter()
        .zip(chart.dignities)
        .any(|(at, dignity)| {
            *at == sign && matches!(dignity, Dignity::Exalted | Dignity::DeepExalted)
        })
}

/// A sign's modality's rank in strength: a dual sign over a fixed over a
/// movable one.
const fn modality_rank(sign: Rashi) -> usize {
    match (sign as usize) % 3 {
        0 => 0,
        1 => 1,
        _ => 2,
    }
}

/// The stronger of two signs by BPHS ch. 46 vv. 158 to 166, or `None` when
/// the verses leave them equal: the one holding an exalted graha when only
/// one does, else the one holding more grahas, else a dual sign over a fixed
/// over a movable one.
///
/// ```
/// use teistro_core::catalogue::{Dignity, Rashi};
/// use teistro_dasha::rashi::{RashiChart, stronger_sign};
///
/// // The Sun alone in Leo, and no graha in Aries.
/// let mut signs = [Rashi::Taurus; 9];
/// signs[0] = Rashi::Leo;
/// let chart = RashiChart {
///     lagna: Rashi::Aries,
///     arudha_lagna: Rashi::Aries,
///     navamsa_lagna: Rashi::Aries,
///     signs,
///     dignities: [Dignity::Neutral; 9],
///     brahma: None,
/// };
/// assert_eq!(stronger_sign(&chart, Rashi::Aries, Rashi::Leo), Some(Rashi::Leo));
/// assert_eq!(stronger_sign(&chart, Rashi::Aries, Rashi::Libra), None);
/// ```
#[must_use]
pub fn stronger_sign(chart: &RashiChart, a: Rashi, b: Rashi) -> Option<Rashi> {
    let by = |score: &dyn Fn(Rashi) -> usize| match score(a).cmp(&score(b)) {
        core::cmp::Ordering::Greater => Some(a),
        core::cmp::Ordering::Less => Some(b),
        core::cmp::Ordering::Equal => None,
    };
    by(&|sign| usize::from(holds_exalted(chart, sign)))
        .or_else(|| by(&|sign| occupants(chart, sign)))
        .or_else(|| by(&modality_rank))
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

/// The stronger of a sign's lords under a rule (crux C51).
///
/// - [`DualLord::Bphs`], ch. 46 vv. 158 to 166: a lord standing in the sign
///   counts to the other (both there, twelve years either way); else the
///   lord in the stronger sign by [`stronger_sign`]; and when the signs are
///   equal, the lord the greater count reaches.
/// - [`DualLord::Kendra`], the recording engine: the one in a kendra from the
///   sign when only one is, else the first.
#[must_use]
pub fn stronger_lord(chart: &RashiChart, sign: Rashi, rule: DualLord) -> Graha {
    let first = first_lord(sign);
    let Some(second) = second_lord(sign) else {
        return first;
    };
    let (at_first, at_second) = (chart.sign_of(first), chart.sign_of(second));
    if rule == DualLord::Kendra {
        let in_kendra = |at: Rashi| forward(sign, at) % 3 == 0;
        return if in_kendra(at_second) && !in_kendra(at_first) {
            second
        } else {
            first
        };
    }
    if at_first == sign {
        return second;
    }
    if at_second == sign {
        return first;
    }
    let second_wins = match stronger_sign(chart, at_first, at_second) {
        Some(stronger) => stronger == at_second,
        None => counted_years(chart, sign, second) > counted_years(chart, sign, first),
    };
    if second_wins { second } else { first }
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Start {
    /// The lagna.
    Lagna,
    /// The arudha lagna.
    ArudhaLagna,
    /// The navamsa lagna.
    NavamsaLagna,
    /// The sign the Brahma graha occupies (BPHS ch. 46 v. 169).
    Brahma,
}

impl Start {
    /// Whether every chart gives this start. The lagnas always stand; the
    /// Brahma graha is one the verses find on about half of charts, and a
    /// system starting from it is refused on the rest (C125).
    ///
    /// ```
    /// use teistro_dasha::Start;
    ///
    /// assert!(Start::Lagna.every_chart_gives());
    /// assert!(!Start::Brahma.every_chart_gives());
    /// ```
    #[must_use]
    pub const fn every_chart_gives(self) -> bool {
        !matches!(self, Start::Brahma)
    }
}

/// The order a system visits the signs in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
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
    /// Every sign in turn from the start, forward whatever the start's
    /// parity: the Sthira dasa's order, the Sanskrit stating none (C129).
    Forward,
}

/// How long a sign's period runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NamedLord {
    /// The stronger of a dual-lorded sign's two.
    Stronger,
    /// The first, Ketu or Rahu.
    First,
}

/// A sign-based system.
///
/// Borrowed where it is shipped and owned where a consumer registered it,
/// exactly as [`UduRow`](crate::UduRow) is: a shipped row clones without
/// allocating, which the allocation tests hold, and a registered one owns
/// its key and its houses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RashiRow {
    /// Which system the row is: a catalogued one, or a consumer's by the
    /// key it was registered under.
    pub system: DashaName,
    /// Where it starts.
    pub start: Start,
    /// The order it visits the signs in.
    pub order: Order,
    /// How long a sign's period runs.
    pub length: Length,
    /// Which lord a mahadasha names.
    pub named_lord: NamedLord,
    /// The houses from the lagna BPHS starts the system from the strongest of,
    /// under [`RashiStart::Stronger`]; empty when it names no such start.
    pub stronger_of: Cow<'static, [u8]>,
}

impl RashiRow {
    /// The sign the system starts from under `rules`.
    ///
    /// # Errors
    ///
    /// `UNSUPPORTED` for a system that starts from the Brahma graha over a
    /// chart whose rule found none, naming the knob that supplies another
    /// rule: a start from nowhere is refused, never taken from the lagna.
    pub fn start_sign(&self, chart: &RashiChart, rules: RashiRules) -> Result<Rashi, Error> {
        if rules.start == RashiStart::Stronger && !self.stronger_of.is_empty() {
            // The strongest of the houses named, the earlier one on a tie.
            let house = |h: u8| step(chart.lagna, Direction::Forward, usize::from(h.max(1) - 1));
            return Ok(self
                .stronger_of
                .iter()
                .map(|h| house(*h))
                .reduce(|best, next| {
                    if stronger_sign(chart, best, next) == Some(next) {
                        next
                    } else {
                        best
                    }
                })
                .unwrap_or(chart.lagna));
        }
        Ok(match self.start {
            Start::Lagna => chart.lagna,
            Start::ArudhaLagna => chart.arudha_lagna,
            Start::NavamsaLagna => chart.navamsa_lagna,
            Start::Brahma => chart.brahma.ok_or_else(|| {
                Error::unsupported(format!(
                    "{} starts from the Brahma graha, and this chart has none under its rule",
                    self.system
                ))
                .with_field("jaimini.brahma")
                .with_hint(
                    "the verses of BPHS ch. 46 find none on many charts and give no fallback; \
                     TRANSLATORS_NOTE is the rule that supplies one",
                )
            })?,
        })
    }

    /// The mahadasha signs for a chart, in order.
    ///
    /// # Errors
    ///
    /// As [`RashiRow::start_sign`].
    pub fn sequence(&self, chart: &RashiChart, rules: RashiRules) -> Result<[Rashi; SIGNS], Error> {
        let start = self.start_sign(chart, rules)?;
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
            Order::Forward => fill(&mut (0..SIGNS).map(|k| step(start, Direction::Forward, k))),
        }
        Ok(out)
    }

    /// A sign's period in years for a chart.
    #[must_use]
    pub fn years(&self, chart: &RashiChart, sign: Rashi, rules: RashiRules) -> u8 {
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
            Length::CountToLord => {
                counted_years(chart, sign, stronger_lord(chart, sign, rules.dual_lord))
            }
            Length::CountToLordByDignity => {
                let lord = stronger_lord(chart, sign, rules.dual_lord);
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
    pub fn lord(&self, chart: &RashiChart, sign: Rashi, rules: RashiRules) -> Graha {
        match self.named_lord {
            NamedLord::Stronger => stronger_lord(chart, sign, rules.dual_lord),
            NamedLord::First => first_lord(sign),
        }
    }
}

impl RashiRow {
    /// Whether this row is a catalogued system's.
    #[must_use]
    pub fn is(&self, system: DashaSystem) -> bool {
        self.system.catalogued() == Some(system)
    }

    /// The whole-table invariants a row must meet, shipped or registered.
    ///
    /// A sign-based row has no lords and no seed to get wrong, so what is
    /// left is the two places a number can be: the years a fixed or
    /// modality length gives, and the houses a stronger start counts from.
    /// Both are refused by field, exactly as `UduRow::validate` does, so a
    /// registered row is refused by the same field a shipped one would be.
    ///
    /// # Errors
    ///
    /// A length of no years, naming `length`; a house outside one to
    /// twelve, or fewer than two to be strongest of, naming `stronger_of`.
    pub fn validate(&self) -> Result<(), Error> {
        let refuse = |field: &str, message: String| {
            Err(Error::invalid_arg(message).with_field(field.to_owned()))
        };
        match self.length {
            Length::Fixed(0) => {
                return refuse("length", String::from("a sign of no years"));
            }
            Length::ByModality {
                movable,
                fixed,
                dual,
            } if movable == 0 || fixed == 0 || dual == 0 => {
                return refuse(
                    "length",
                    String::from("a movable, fixed or dual sign of no years"),
                );
            }
            _ => {}
        }
        if let Some((at, house)) = self
            .stronger_of
            .iter()
            .enumerate()
            .find(|(_, house)| **house == 0 || **house > 12)
        {
            return refuse(
                &format!("stronger_of[{at}]"),
                format!("house {house} is not one of the twelve"),
            );
        }
        if self.stronger_of.len() == 1 {
            return refuse(
                "stronger_of",
                String::from("the strongest of one house is that house; name two or none"),
            );
        }
        let mut seen = self.stronger_of.to_vec();
        seen.sort_unstable();
        seen.dedup();
        if seen.len() < self.stronger_of.len() {
            return refuse(
                "stronger_of",
                String::from("a house named twice is no stronger for it"),
            );
        }
        Ok(())
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
    stronger_of: &'static [u8],
) -> RashiRow {
    RashiRow {
        system: DashaName::Catalogued(system),
        start,
        order,
        length,
        named_lord,
        stronger_of: Cow::Borrowed(stronger_of),
    }
}

/// Chara: every sign from the lagna, each counted to its stronger lord.
pub const CHARA: RashiRow = row(
    DashaSystem::Chara,
    Start::Lagna,
    Order::Consecutive,
    Length::CountToLord,
    NamedLord::Stronger,
    &[],
);
/// Narayana: Chara with the lord's exaltation and debilitation.
pub const NARAYANA: RashiRow = row(
    DashaSystem::Narayana,
    Start::Lagna,
    Order::Consecutive,
    Length::CountToLordByDignity,
    NamedLord::Stronger,
    &[],
);
/// Padanadhamsa: Chara from the arudha lagna.
pub const PADANADHAMSA: RashiRow = row(
    DashaSystem::Padanadhamsa,
    Start::ArudhaLagna,
    Order::Consecutive,
    Length::CountToLord,
    NamedLord::Stronger,
    &[],
);
/// Trikona: the trine groups from the lagna's, or under BPHS from the
/// strongest trine's (ch. 46 vv. 183 and 184).
pub const TRIKONA: RashiRow = row(
    DashaSystem::Trikona,
    Start::Lagna,
    Order::TrineGroups,
    Length::CountToLord,
    NamedLord::Stronger,
    &[1, 5, 9],
);
/// Drig: the ninth, tenth and eleventh houses and what each aspects.
pub const DRIG: RashiRow = row(
    DashaSystem::Drig,
    Start::Lagna,
    Order::DrishtiChain,
    Length::CountToLord,
    NamedLord::Stronger,
    &[],
);
/// Shoola: nine years a sign from the lagna, or under BPHS from the stronger
/// of the second and the eighth (ch. 46 vv. 181 and 182).
pub const SHOOLA: RashiRow = row(
    DashaSystem::Shoola,
    Start::Lagna,
    Order::Consecutive,
    Length::Fixed(9),
    NamedLord::First,
    &[2, 8],
);
/// Niryana Shoola: Shoola from the navamsa lagna.
pub const NIRYANA_SHOOLA: RashiRow = row(
    DashaSystem::NiryanaShoola,
    Start::NavamsaLagna,
    Order::Consecutive,
    Length::Fixed(9),
    NamedLord::First,
    &[],
);
/// Mandooka: leaping back two signs, seven, eight or nine years by modality,
/// from the lagna, or under BPHS from the stronger of the lagna and the
/// seventh (ch. 46 vv. 179 and 180).
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
    &[1, 7],
);

/// A consumer's own sign-based system, the shape [`UduDefinition`] has for
/// the nakshatra-seeded kernel.
///
/// A sign-based system is four choices and a list of houses — where it
/// starts, the order it visits the signs in, how long a sign runs, which
/// lord a mahadasha names — so a consumer holding a text that states them
/// registers the row and asks for it by key, with no change to this crate.
/// That is the whole point of it: without one, a school's Sthira or
/// Varnada is shut to a consumer who **has** the text as firmly as it is
/// to this build, which is a dead end in the SDK rather than a gap in the
/// sources (`03-design/dasha-coverage-measured.md`).
///
/// [`UduDefinition`]: crate::UduDefinition
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_dasha::{Length, NamedLord, Order, RashiDefinition, Start};
///
/// // A Sthira reckoned from the lagna rather than the Brahma graha, every
/// // sign in turn, seven, eight or nine years by the sign's modality.
/// let sthira = RashiDefinition {
///     length: Length::ByModality { movable: 7, fixed: 8, dual: 9 },
///     ..RashiDefinition::of("ACME_STHIRA")
/// };
/// assert_eq!(sthira.row().start, Start::Lagna);
/// assert_eq!(sthira.row().order, Order::Consecutive);
/// assert_eq!(sthira.row().named_lord, NamedLord::Stronger);
/// assert_eq!(Rashi::ALL.len(), 12);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct RashiDefinition {
    /// The key it is registered under, in the key grammar and not one the
    /// catalogue has.
    pub key: String,
    /// Where the row comes from.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<String>,
    /// Where it starts; the lagna by default.
    #[serde(default = "lagna")]
    pub start: Start,
    /// The order it visits the signs in; every sign in turn by default.
    #[serde(default = "consecutive")]
    pub order: Order,
    /// How long a sign's period runs; the count to its stronger lord by
    /// default, which is what most of the family does.
    #[serde(default = "count_to_lord")]
    pub length: Length,
    /// Which lord a mahadasha names; the stronger of a dual-lorded sign's
    /// two by default.
    #[serde(default = "stronger")]
    pub named_lord: NamedLord,
    /// The houses from the lagna to start from the strongest of, under
    /// [`RashiStart::Stronger`]; none by default, which starts from
    /// [`RashiDefinition::start`] itself.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stronger_of: Vec<u8>,
    /// The length of its year; the Julian year by default, as every
    /// catalogued system takes unless the settings say otherwise.
    #[serde(default = "julian")]
    pub year_length: YearLength,
    /// How many levels of periods a reading carries; three by default.
    #[serde(default = "three")]
    pub depth: Depth,
}

const fn lagna() -> Start {
    Start::Lagna
}

const fn consecutive() -> Order {
    Order::Consecutive
}

const fn count_to_lord() -> Length {
    Length::CountToLord
}

const fn stronger() -> NamedLord {
    NamedLord::Stronger
}

impl RashiDefinition {
    /// A definition with a key and every other field its default: Chara's
    /// row under another name, which a caller then changes.
    #[must_use]
    pub fn of(key: impl Into<String>) -> RashiDefinition {
        RashiDefinition {
            key: key.into(),
            sources: Vec::new(),
            start: lagna(),
            order: consecutive(),
            length: count_to_lord(),
            named_lord: stronger(),
            stronger_of: Vec::new(),
            year_length: julian(),
            depth: three(),
        }
    }

    /// The row the kernel runs, its houses owned.
    #[must_use]
    pub fn row(&self) -> RashiRow {
        RashiRow {
            system: DashaName::Registered(self.key.clone()),
            start: self.start,
            order: self.order,
            length: self.length,
            named_lord: self.named_lord,
            stronger_of: Cow::Owned(self.stronger_of.clone()),
        }
    }
}

impl teistro_core::registry::Definition for RashiDefinition {
    fn key(&self) -> &str {
        &self.key
    }

    fn validate(&self) -> Result<(), Error> {
        self.row().validate()
    }
}

/// Sthira: seven, eight or nine years by modality, from the sign the Brahma
/// graha occupies, every sign in turn forward (BPHS ch. 46 vv. 168 to
/// 173; `03-design/jaimini-significators.md`). The coverage page stated it
/// from the lagna; the Sanskrit starts it from Brahma, and states no order,
/// which the translation's "reverse from even signs" supplies and this
/// does not (C129).
pub const STHIRA: RashiRow = row(
    DashaSystem::Sthira,
    Start::Brahma,
    Order::Forward,
    Length::ByModality {
        movable: 7,
        fixed: 8,
        dual: 9,
    },
    NamedLord::Stronger,
    &[],
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
    STHIRA,
];

/// The sign-based row of a system, when this build implements one.
#[must_use]
pub fn rashi_row(system: DashaSystem) -> Option<&'static RashiRow> {
    RASHI_ROWS.iter().find(|row| row.is(system))
}

/// A sign-based dasha of one chart.
#[derive(Clone, Debug, PartialEq)]
pub struct RashiDasha {
    row: RashiRow,
    chart: RashiChart,
    birth: JulianDay<Utc>,
    after_cycle: AfterCycle,
    rules: RashiRules,
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
    /// A year length of no days, named `year_length`; a start the chart
    /// cannot give ([`RashiRow::start_sign`]).
    pub fn new(
        row: &RashiRow,
        chart: &RashiChart,
        birth: JulianDay<Utc>,
        year_length: YearLength,
        after_cycle: AfterCycle,
        rules: RashiRules,
    ) -> Result<RashiDasha, Error> {
        let year_days = year_length.days();
        if year_days.is_nan() || year_days <= 0.0 {
            return Err(Error::invalid_arg("a year of no days").with_field("year_length"));
        }
        let signs = row.sequence(chart, rules)?;
        let mut lords = [Graha::Sun; SIGNS];
        let mut offsets = [0.0; SIGNS + 1];
        let mut total = 0.0;
        for ((lord, sign), end) in lords.iter_mut().zip(signs).zip(offsets.iter_mut().skip(1)) {
            *lord = row.lord(chart, sign, rules);
            total += f64::from(row.years(chart, sign, rules)) * year_days;
            *end = total;
        }
        Ok(RashiDasha {
            row: row.clone(),
            chart: *chart,
            birth,
            after_cycle,
            rules,
            signs,
            lords,
            offsets,
        })
    }

    /// The row the dasha runs.
    #[must_use]
    pub const fn row(&self) -> &RashiRow {
        &self.row
    }

    /// The readings it was built under.
    #[must_use]
    pub const fn rules(&self) -> RashiRules {
        self.rules
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

    /// A whole cycle's days.
    fn cycle_days(&self) -> f64 {
        self.offsets.last().copied().unwrap_or_default()
    }
}

impl Timeline for RashiDasha {
    fn breadth(&self) -> usize {
        SIGNS
    }

    /// The mahadasha at `index` of `cycle`, or nothing past the end of the
    /// cycle when the rules end it.
    fn mahadasha(&self, cycle: u32, index: usize) -> Option<Period> {
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

    /// Twelve equal antardashas from the period's own sign, each naming its
    /// sign's first lord (crux C50 names the reading that begins from the
    /// next sign): forward under an [`Order::Forward`] row, as its periods
    /// run (C129), and otherwise forward from an odd sign and back from an
    /// even one.
    fn child(&self, parent: &Period, index: usize) -> Option<Period> {
        let sign = parent.sign?;
        if index >= SIGNS {
            return None;
        }
        let place = u8::try_from(index).ok()?;
        let path = parent.path.child(place)?;
        let direction = match self.row.order {
            Order::Forward => Direction::Forward,
            _ => Direction::of(Parity::of(sign)),
        };
        let child = step(sign, direction, index);
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
            brahma: Some(lagna),
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
        for rules in [RashiRules::BPHS, RashiRules::RECORDING_ENGINE] {
            for row in RASHI_ROWS {
                for lagna in Rashi::ALL {
                    let chart = RashiChart {
                        arudha_lagna: lagna,
                        navamsa_lagna: lagna,
                        ..chart(lagna)
                    };
                    let mut sequence = row.sequence(&chart, rules).unwrap();
                    sequence.sort();
                    assert_eq!(sequence, Rashi::ALL, "{:?} from {lagna:?}", row.system);
                }
            }
        }
    }

    #[test]
    fn every_length_is_one_to_twelve_years() {
        for row in RASHI_ROWS {
            for lagna in Rashi::ALL {
                let c = chart(lagna);
                for sign in Rashi::ALL {
                    for rules in [RashiRules::BPHS, RashiRules::RECORDING_ENGINE] {
                        let years = row.years(&c, sign, rules);
                        assert!(
                            (1..=12).contains(&years),
                            "{:?} {sign:?}: {years}",
                            row.system
                        );
                    }
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
        assert_eq!(
            stronger_lord(&c, Rashi::Aquarius, DualLord::Kendra),
            Graha::Rahu
        );
        // Scorpio: Mars in Aquarius is a kendra from it, Ketu in Cancer is not.
        assert_eq!(
            stronger_lord(&c, Rashi::Scorpio, DualLord::Kendra),
            Graha::Mars
        );
    }

    #[test]
    fn bphs_finds_the_stronger_lord_by_the_signs_they_stand_in() {
        const MARS: usize = 2;
        const JUPITER: usize = 4;
        const KETU: usize = 8;
        let base = chart(Rashi::Pisces);
        let lord = |c: &RashiChart| stronger_lord(c, Rashi::Scorpio, DualLord::Bphs);
        // Mars and Venus in Aquarius outnumber Ketu alone in Cancer.
        assert_eq!(lord(&base), Graha::Mars);
        // Mars in Scorpio itself: count to the other, where the engine keeps
        // the lord in the sign.
        let mut home = base;
        home.signs[MARS] = Rashi::Scorpio;
        assert_eq!(lord(&home), Graha::Ketu);
        assert_eq!(
            stronger_lord(&home, Rashi::Scorpio, DualLord::Kendra),
            Graha::Mars
        );
        // An exalted Jupiter beside Ketu settles it, however many are with Mars.
        let mut exalted = base;
        exalted.signs[JUPITER] = Rashi::Cancer;
        exalted.dignities[JUPITER] = Dignity::Exalted;
        assert_eq!(lord(&exalted), Graha::Ketu);
        // One graha each: the dual Gemini over the fixed Taurus.
        let mut modal = base;
        modal.signs = [Rashi::Aries; 9];
        modal.signs[MARS] = Rashi::Taurus;
        modal.signs[KETU] = Rashi::Gemini;
        assert_eq!(lord(&modal), Graha::Ketu);
        // Equal signs, one graha each and both movable: the lord the greater
        // count reaches. Scorpio is odd-footed, so it counts forward: Mars in
        // Libra eleven signs on, Ketu in Cancer eight.
        let mut even = base;
        even.signs = [Rashi::Aries; 9];
        even.signs[MARS] = Rashi::Libra;
        even.signs[KETU] = Rashi::Cancer;
        assert_eq!(counted_years(&even, Rashi::Scorpio, Graha::Mars), 11);
        assert_eq!(counted_years(&even, Rashi::Scorpio, Graha::Ketu), 8);
        assert_eq!(lord(&even), Graha::Mars);
    }

    #[test]
    fn bphs_starts_from_the_stronger_of_the_signs_it_names() {
        let c = chart(Rashi::Pisces);
        // Trikona: Pisces empty, Cancer holding Ketu, Scorpio the Moon; one
        // each, and the fixed Scorpio over the movable Cancer.
        assert_eq!(
            TRIKONA.start_sign(&c, RashiRules::BPHS).unwrap(),
            Rashi::Scorpio
        );
        assert_eq!(
            TRIKONA
                .start_sign(&c, RashiRules::RECORDING_ENGINE)
                .unwrap(),
            Rashi::Pisces
        );
        // Mandooka: Pisces and Virgo both empty and both dual, so the lagna;
        // the Sun in Virgo makes it the seventh.
        assert_eq!(
            MANDOOKA.start_sign(&c, RashiRules::BPHS).unwrap(),
            Rashi::Pisces
        );
        let mut sun = c;
        sun.signs[0] = Rashi::Virgo;
        assert_eq!(
            MANDOOKA.start_sign(&sun, RashiRules::BPHS).unwrap(),
            Rashi::Virgo
        );
        // Chara names no stronger start, and keeps the lagna.
        assert_eq!(
            CHARA.start_sign(&sun, RashiRules::BPHS).unwrap(),
            Rashi::Pisces
        );
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
            RashiRules::RECORDING_ENGINE,
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

    /// The Sthira dasa's antardashas run forward like its periods, from an
    /// even sign as from an odd one (C129), where the Chara's above run back.
    #[test]
    fn sthira_antardashas_run_forward_from_an_even_sign() {
        let c = RashiChart {
            brahma: Some(Rashi::Pisces),
            ..chart(Rashi::Aries)
        };
        let dasha = RashiDasha::new(
            &STHIRA,
            &c,
            JulianDay::literal(2_451_545.0),
            YearLength::Julian36525,
            AfterCycle::End,
            RashiRules::BPHS,
        )
        .unwrap();
        let first = dasha.mahadashas().next().unwrap();
        assert_eq!(first.sign, Some(Rashi::Pisces));
        let children: Vec<Option<Rashi>> = dasha.children(&first).map(|child| child.sign).collect();
        let forward: Vec<Option<Rashi>> = (0..SIGNS)
            .map(|k| Some(step(Rashi::Pisces, Direction::Forward, k)))
            .collect();
        assert_eq!(children, forward);
        assert_eq!(children[1], Some(Rashi::Aries));
    }
}
