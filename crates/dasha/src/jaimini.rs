//! Jaimini's significators the sign dashas start from: the **karakamsha**,
//! the Atmakaraka's navamsha sign, and the **Brahma graha**, the planet the
//! Sthira dasa starts from (`03-design/jaimini-significators.md`).
//!
//! Both are read from what a sign dasha already reads of a chart — the
//! signs, the dignities, the lagna — with the one or two things more each
//! needs passed beside it: the Atmakaraka and the navamshas for the
//! karakamsha, each graha's degrees for Brahma. So they sit here, beside
//! [`stronger_sign`](crate::rashi::stronger_sign), whose sign strength
//! (BPHS ch. 46 vv. 158 to 166) Brahma is chosen by, and not in the rules,
//! which no dasha may depend on.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::settings::{BrahmaRule, DualLord, NodeCoLordship};

use crate::rashi::{RashiChart, first_lord, second_lord, stronger_lord, stronger_sign};

/// Signs in the zodiac.
const SIGNS: usize = 12;

/// The grahas a chart places, in the catalogue's order.
const GRAHAS: [Graha; 9] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
    Graha::Rahu,
    Graha::Ketu,
];

/// The sign `steps` forward of another.
fn ahead(from: Rashi, steps: usize) -> Rashi {
    Rashi::ALL
        .get((from as usize + steps) % SIGNS)
        .copied()
        .unwrap_or(from)
}

/// The steps from one sign forward to another, 0 to 11.
const fn steps(from: Rashi, to: Rashi) -> usize {
    (to as usize + SIGNS - from as usize) % SIGNS
}

/// The house one sign is counted from another, 1 to 12.
fn house(from: Rashi, to: Rashi) -> u8 {
    u8::try_from(steps(from, to) + 1).unwrap_or(1)
}

/// Whether a sign is odd: Aries, Gemini, Leo, Libra, Sagittarius, Aquarius.
const fn odd(sign: Rashi) -> bool {
    (sign as usize) % 2 == 0
}

/// The karakamsha: the navamsha sign the Atmakaraka occupies (BPHS ch. 33
/// v. 1), and every graha's house counted from it in **both** charts, since
/// the schools read its houses in the rasi chart or in the navamsha itself
/// (crux C130) and a consumer reading one wants the other named.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Karakamsha {
    /// The Atmakaraka, under the profile's chara karaka scheme.
    pub atmakaraka: Graha,
    /// The karakamsha: the Atmakaraka's navamsha sign.
    pub sign: Rashi,
    /// Each graha's house from the karakamsha, counted in the rasi chart,
    /// the Sun to Ketu.
    pub in_rasi: [u8; 9],
    /// Each graha's house from the karakamsha, counted in the navamsha, the
    /// Sun to Ketu.
    pub in_navamsha: [u8; 9],
}

/// The karakamsha of a chart.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_dasha::jaimini::karakamsha;
///
/// let signs = [Rashi::Aries; 9];
/// let mut navamshas = [Rashi::Taurus; 9];
/// navamshas[Graha::Sun as usize] = Rashi::Leo;
/// let reading = karakamsha(Graha::Sun, &signs, &navamshas);
/// assert_eq!(reading.sign, Rashi::Leo);
/// // Aries is the 9th sign from Leo in the rasi chart, and the Sun's own
/// // navamsha is the 1st from itself in the navamsha.
/// assert_eq!(reading.in_rasi[Graha::Sun as usize], 9);
/// assert_eq!(reading.in_navamsha[Graha::Sun as usize], 1);
/// ```
#[must_use]
pub fn karakamsha(atmakaraka: Graha, signs: &[Rashi; 9], navamshas: &[Rashi; 9]) -> Karakamsha {
    let sign = navamshas
        .get(atmakaraka as usize)
        .copied()
        .unwrap_or(Rashi::Aries);
    Karakamsha {
        atmakaraka,
        sign,
        in_rasi: signs.map(|at| house(sign, at)),
        in_navamsha: navamshas.map(|at| house(sign, at)),
    }
}

/// A chart's Jaimini significators: what `sdk.chart().jaimini` answers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct JaiminiReading {
    /// The karakamsha, with every graha's house from it in both charts.
    pub karakamsha: Karakamsha,
    /// The Brahma graha, or why there is none.
    pub brahma: Brahma,
}

/// Why a chart has no Brahma graha under the rule asked for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NoBrahma {
    /// Under the verses, no lord of the 6th, 8th or 12th stands in an odd
    /// sign behind the sign counted from; they give no fallback (C128).
    NoLordQualifies,
    /// Saturn or a node qualified, and no planet stands in the 6th sign
    /// from it to take its place (C127).
    NoPlanetInTheSixth,
    /// Under the translator's note, no planet stands in the 8th and none in
    /// an odd sign within the six signs behind.
    NoPlanetQualifies,
}

/// The Brahma graha of a chart, and how it was found (BPHS ch. 46 vv. 170
/// to 173).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Brahma {
    /// The rule it was found under.
    pub rule: BrahmaRule,
    /// The stronger of the lagna and the 7th, which the houses are counted
    /// from.
    pub counted_from: Rashi,
    /// The planets that met the rule's marks, in the catalogue's order: the
    /// field a reader checks a choice against.
    pub qualified: Vec<Graha>,
    /// The Brahma graha, or `None` where the rule finds none.
    pub graha: Option<Graha>,
    /// Saturn or the node that qualified and passed Brahma-hood to the
    /// planet in the 6th from it (C127), when one did.
    pub passed_from: Option<Graha>,
    /// Why there is none, when there is none.
    pub none: Option<NoBrahma>,
}

impl Brahma {
    /// The sign the Brahma graha stands in, which the Sthira dasa starts
    /// from.
    #[must_use]
    pub fn sign(&self, chart: &RashiChart) -> Option<Rashi> {
        self.graha.map(|graha| chart.sign_of(graha))
    }
}

/// A graha's degrees in its sign as the verses weigh them: a node's are
/// counted from the end of the sign, since it moves backward through it.
fn weight(graha: Graha, degrees: &[f64; 9]) -> f64 {
    let at = degrees.get(graha as usize).copied().unwrap_or(0.0);
    if matches!(graha, Graha::Rahu | Graha::Ketu) {
        30.0 - at
    } else {
        at
    }
}

/// The one of several with the most degrees, and on equal degrees the one
/// in the stronger sign (v. 173), else the first in the catalogue's order.
fn most_degrees(chart: &RashiChart, degrees: &[f64; 9], among: &[Graha]) -> Option<Graha> {
    among.iter().copied().reduce(|best, next| {
        match weight(next, degrees).total_cmp(&weight(best, degrees)) {
            core::cmp::Ordering::Greater => next,
            core::cmp::Ordering::Less => best,
            core::cmp::Ordering::Equal => {
                let (at_best, at_next) = (chart.sign_of(best), chart.sign_of(next));
                if stronger_sign(chart, at_best, at_next) == Some(at_next) {
                    next
                } else {
                    best
                }
            }
        }
    })
}

/// The lords of a sign under the settings' co-lordship of Scorpio and
/// Aquarius: the traditional lord alone, the stronger of the two by the
/// chapter's own rule, or both.
fn lords(chart: &RashiChart, sign: Rashi, co_lordship: NodeCoLordship) -> Vec<Graha> {
    let Some(second) = second_lord(sign) else {
        return vec![first_lord(sign)];
    };
    // The first Jaimini lord of a dual-lorded sign is its node, the second
    // its traditional lord.
    match co_lordship {
        NodeCoLordship::Both => vec![second, first_lord(sign)],
        NodeCoLordship::StrongerLord => vec![stronger_lord(chart, sign, DualLord::Bphs)],
        _ => vec![second],
    }
}

/// Whether a sign is among the six counted back from another: that sign
/// and the five before it (C124).
const fn behind(sign: Rashi, from: Rashi) -> bool {
    steps(sign, from) < 6
}

/// The planets standing in a sign, in the catalogue's order.
fn in_sign(chart: &RashiChart, sign: Rashi) -> Vec<Graha> {
    GRAHAS
        .into_iter()
        .filter(|graha| chart.signs.get(*graha as usize) == Some(&sign))
        .collect()
}

/// The Brahma graha of a chart under a rule.
///
/// `degrees` are each graha's degrees within its sign, the Sun to Ketu.
/// BPHS's worked example: an Aquarius lagna holding Mercury, Jupiter and
/// Venus, stronger than an empty Leo; of the lords of the 6th, 8th and
/// 12th, only the 8th lord Mercury stands in an odd sign behind Aquarius,
/// in Aquarius itself.
///
/// ```
/// use teistro_core::catalogue::{Dignity, Graha, Rashi};
/// use teistro_core::settings::{BrahmaRule, NodeCoLordship};
/// use teistro_dasha::jaimini::brahma;
/// use teistro_dasha::rashi::RashiChart;
///
/// let chart = RashiChart {
///     lagna: Rashi::Aquarius,
///     arudha_lagna: Rashi::Aquarius,
///     navamsa_lagna: Rashi::Aquarius,
///     signs: [
///         Rashi::Capricorn, Rashi::Gemini, Rashi::Taurus, Rashi::Aquarius, Rashi::Aquarius,
///         Rashi::Aquarius, Rashi::Scorpio, Rashi::Gemini, Rashi::Sagittarius,
///     ],
///     dignities: [Dignity::Neutral; 9],
///     brahma: None,
/// };
/// let degrees = [29.6, 22.1, 0.9, 13.1, 13.7, 20.1, 13.4, 13.9, 13.9];
/// let found = brahma(&chart, &degrees, NodeCoLordship::None, BrahmaRule::Verses);
/// assert_eq!(found.counted_from, Rashi::Aquarius);
/// assert_eq!(found.graha, Some(Graha::Mercury));
/// ```
#[must_use]
pub fn brahma(
    chart: &RashiChart,
    degrees: &[f64; 9],
    co_lordship: NodeCoLordship,
    rule: BrahmaRule,
) -> Brahma {
    let seventh = ahead(chart.lagna, 6);
    let counted_from = stronger_sign(chart, chart.lagna, seventh).unwrap_or(chart.lagna);
    let marked = |graha: &Graha| {
        let at = chart.sign_of(*graha);
        odd(at) && behind(at, counted_from)
    };
    let found = |qualified: Vec<Graha>, graha: Option<Graha>, none: NoBrahma| Brahma {
        rule,
        counted_from,
        graha,
        passed_from: None,
        none: graha.is_none().then_some(none),
        qualified,
    };
    if rule == BrahmaRule::TranslatorsNote {
        let eighth = ahead(counted_from, 7);
        let lord = lords(chart, eighth, co_lordship);
        if let Some(own) = lord
            .iter()
            .copied()
            .find(|graha| chart.sign_of(*graha) == eighth)
        {
            return found(vec![own], Some(own), NoBrahma::NoPlanetQualifies);
        }
        let there = in_sign(chart, eighth);
        if !there.is_empty() {
            let chosen = most_degrees(chart, degrees, &there);
            return found(there, chosen, NoBrahma::NoPlanetQualifies);
        }
        let near: Vec<Graha> = GRAHAS.into_iter().filter(marked).collect();
        let chosen = most_degrees(chart, degrees, &near);
        return found(near, chosen, NoBrahma::NoPlanetQualifies);
    }

    let mut candidates: Vec<Graha> = [6, 8, 12]
        .into_iter()
        .flat_map(|h| lords(chart, ahead(counted_from, h - 1), co_lordship))
        .collect();
    candidates.sort_by_key(|graha| *graha as usize);
    candidates.dedup();
    let qualified: Vec<Graha> = candidates.into_iter().filter(marked).collect();
    let Some(chosen) = most_degrees(chart, degrees, &qualified) else {
        return found(qualified, None, NoBrahma::NoLordQualifies);
    };
    if !matches!(chosen, Graha::Saturn | Graha::Rahu | Graha::Ketu) {
        return found(qualified, Some(chosen), NoBrahma::NoLordQualifies);
    }
    let sixth = ahead(chart.sign_of(chosen), 5);
    let heir = most_degrees(chart, degrees, &in_sign(chart, sixth));
    Brahma {
        passed_from: Some(chosen),
        ..found(qualified, heir, NoBrahma::NoPlanetInTheSixth)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::indexing_slicing, reason = "tests")]

    use teistro_core::catalogue::Dignity;

    use super::*;

    /// BPHS ch. 46's worked example after v. 173.
    fn example() -> (RashiChart, [f64; 9]) {
        (
            RashiChart {
                lagna: Rashi::Aquarius,
                arudha_lagna: Rashi::Aquarius,
                navamsa_lagna: Rashi::Aquarius,
                signs: [
                    Rashi::Capricorn,
                    Rashi::Gemini,
                    Rashi::Taurus,
                    Rashi::Aquarius,
                    Rashi::Aquarius,
                    Rashi::Aquarius,
                    Rashi::Scorpio,
                    Rashi::Gemini,
                    Rashi::Sagittarius,
                ],
                dignities: [Dignity::Neutral; 9],
                brahma: None,
            },
            [
                29.0 + 36.0 / 60.0,
                22.0 + 7.0 / 60.0,
                56.0 / 60.0,
                13.0 + 6.0 / 60.0,
                13.0 + 42.0 / 60.0,
                20.0 + 4.0 / 60.0,
                13.0 + 24.0 / 60.0,
                13.0 + 56.0 / 60.0,
                13.0 + 56.0 / 60.0,
            ],
        )
    }

    #[test]
    fn the_worked_example_finds_mercury_under_every_co_lordship() {
        let (chart, degrees) = example();
        for co in [
            NodeCoLordship::None,
            NodeCoLordship::StrongerLord,
            NodeCoLordship::Both,
        ] {
            let found = brahma(&chart, &degrees, co, BrahmaRule::Verses);
            assert_eq!(found.graha, Some(Graha::Mercury), "{co:?}");
            assert_eq!(found.qualified, vec![Graha::Mercury], "{co:?}");
            assert_eq!(found.sign(&chart), Some(Rashi::Aquarius));
            assert_eq!(found.none, None);
        }
    }

    #[test]
    fn saturn_qualifying_passes_brahma_to_the_planet_in_its_sixth() {
        // A Leo lagna, stronger than an empty Aquarius, with Saturn (the 6th
        // lord from Leo, Capricorn's) in the odd Gemini, among the six signs
        // behind Leo, and Venus in Scorpio, the 6th sign from Gemini.
        let mut signs = [Rashi::Taurus; 9];
        signs[Graha::Sun as usize] = Rashi::Leo;
        signs[Graha::Saturn as usize] = Rashi::Gemini;
        signs[Graha::Venus as usize] = Rashi::Scorpio;
        let chart = RashiChart {
            lagna: Rashi::Leo,
            arudha_lagna: Rashi::Leo,
            navamsa_lagna: Rashi::Leo,
            signs,
            dignities: [Dignity::Neutral; 9],
            brahma: None,
        };
        let found = brahma(&chart, &[10.0; 9], NodeCoLordship::None, BrahmaRule::Verses);
        assert_eq!(found.passed_from, Some(Graha::Saturn));
        assert_eq!(found.graha, Some(Graha::Venus));
        // With nobody in Scorpio there is no heir, and the chart says so.
        let mut empty = chart;
        empty.signs[Graha::Venus as usize] = Rashi::Taurus;
        let none = brahma(&empty, &[10.0; 9], NodeCoLordship::None, BrahmaRule::Verses);
        assert_eq!(none.graha, None);
        assert_eq!(none.none, Some(NoBrahma::NoPlanetInTheSixth));
    }

    #[test]
    fn no_qualifying_lord_is_no_brahma_under_the_verses_and_one_under_the_note() {
        // Everything in the even Taurus: no lord stands in an odd sign.
        let chart = RashiChart {
            lagna: Rashi::Aries,
            arudha_lagna: Rashi::Aries,
            navamsa_lagna: Rashi::Aries,
            signs: [Rashi::Taurus; 9],
            dignities: [Dignity::Neutral; 9],
            brahma: None,
        };
        let degrees = [5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 3.0, 3.0];
        let verses = brahma(&chart, &degrees, NodeCoLordship::None, BrahmaRule::Verses);
        assert_eq!(verses.graha, None);
        assert_eq!(verses.none, Some(NoBrahma::NoLordQualifies));
        // The note's rule: the 8th from Taurus, the stronger sign here, is
        // Sagittarius, empty; nothing stands in an odd sign behind Taurus;
        // so it finds none either, and says which rule's none it is.
        let note = brahma(
            &chart,
            &degrees,
            NodeCoLordship::None,
            BrahmaRule::TranslatorsNote,
        );
        assert_eq!(note.none, Some(NoBrahma::NoPlanetQualifies));
    }

    #[test]
    fn a_nodes_degrees_run_from_the_end_of_its_sign() {
        assert!(
            (weight(Graha::Rahu, &[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 10.0, 0.0]) - 20.0).abs()
                < 1e-12
        );
        assert!((weight(Graha::Sun, &[10.0; 9]) - 10.0).abs() < 1e-12);
    }

    #[test]
    fn the_karakamsha_counts_its_houses_in_both_charts() {
        let (chart, _) = example();
        let mut navamshas = [Rashi::Aries; 9];
        navamshas[Graha::Sun as usize] = Rashi::Cancer;
        let reading = karakamsha(Graha::Sun, &chart.signs, &navamshas);
        assert_eq!(reading.sign, Rashi::Cancer);
        // Mercury in Aquarius is the 8th from Cancer in the rasi chart, and
        // its navamsha Aries the 10th in the navamsha.
        assert_eq!(reading.in_rasi[Graha::Mercury as usize], 8);
        assert_eq!(reading.in_navamsha[Graha::Mercury as usize], 10);
    }
}
