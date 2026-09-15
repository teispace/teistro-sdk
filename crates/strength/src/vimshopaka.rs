//! The Vimshopaka: each graha's strength out of 20 across the divisional
//! charts, under the four schemes (`03-design/vimshopaka-measured.md`).
//!
//! The schemes and their weights are BPHS ch. 7 vv. 17 to 25's, which the
//! conformance corpus's recording engine shares. How a graha is scored in
//! one varga is read two ways, and each is a setting ([`Vimshopaka`], crux
//! C63):
//!
//! - [`Vimshopaka::Bphs`], ch. 7 vv. 9 to 16: 20 in exaltation or its own
//!   sign, else 18, 15, 10, 7 or 5 by its compound relationship with the
//!   sign's lord, the temporary half taken where both stand in the rasi
//!   chart; a varga's share is those points times its weight over 20;
//! - [`Vimshopaka::SaptavargajaVirupas`], the recording engine's: the
//!   Saptavargaja virupas by natural friendship alone (45 in exaltation, 30
//!   in moolatrikona or the own sign, 15, 7.5 or 3.75 by friend, neutral or
//!   enemy, nothing in debilitation) over 45, rounded half up to hundredths.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Rashi, Relationship, Varga};
use teistro_core::settings::Vimshopaka;
use teistro_state::dignity::{FRIENDLY_HOUSES, compound, natural};

use crate::ashtakavarga::GRAHAS;

/// The sixteen vargas the schemes read, in the order a
/// [`VimshopakaChart`] holds them.
pub const VARGAS: [Varga; 16] = [
    Varga::D1,
    Varga::D2,
    Varga::D3,
    Varga::D4,
    Varga::D7,
    Varga::D9,
    Varga::D10,
    Varga::D12,
    Varga::D16,
    Varga::D20,
    Varga::D24,
    Varga::D27,
    Varga::D30,
    Varga::D40,
    Varga::D45,
    Varga::D60,
];

/// The four schemes' weights by varga in [`VARGAS`]' order, in halves of a
/// point, each summing to 20 points (BPHS ch. 7 vv. 17 to 25): the
/// shadvarga, saptavarga, dashavarga and shodashavarga.
pub const WEIGHTS: [[u8; 16]; 4] = [
    [12, 4, 8, 0, 0, 10, 0, 4, 0, 0, 0, 0, 2, 0, 0, 0],
    [10, 4, 6, 0, 5, 9, 0, 4, 0, 0, 0, 0, 2, 0, 0, 0],
    [6, 3, 3, 0, 3, 3, 3, 3, 3, 0, 0, 0, 3, 0, 0, 10],
    [7, 2, 2, 1, 1, 6, 1, 1, 4, 1, 1, 1, 2, 1, 1, 8],
];

/// What a Vimshopaka reads of a chart: the seven grahas' signs in each of
/// the sixteen vargas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VimshopakaChart {
    /// Each varga's row in [`VARGAS`]' order, the grahas Sun to Saturn.
    pub signs: [[Rashi; 7]; 16],
}

impl VimshopakaChart {
    /// A chart from each graha's sign in each varga.
    #[must_use]
    pub fn from_fn(sign: impl Fn(Varga, Graha) -> Rashi) -> VimshopakaChart {
        VimshopakaChart {
            signs: VARGAS.map(|varga| GRAHAS.map(|graha| sign(varga, graha))),
        }
    }
}

/// One graha's Vimshopaka, each out of 20.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GrahaVimshopaka {
    /// Which graha.
    pub graha: Graha,
    /// Over the six vargas.
    pub shadvarga: f64,
    /// Over the seven.
    pub saptavarga: f64,
    /// Over the ten.
    pub dashavarga: f64,
    /// Over the sixteen.
    pub shodashavarga: f64,
}

/// A chart's Vimshopaka.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VimshopakaReading {
    /// How each varga was scored.
    pub scoring: Vimshopaka,
    /// Each graha's, Sun to Saturn.
    pub grahas: Vec<GrahaVimshopaka>,
}

/// The text's points out of 20 for a graha in a varga sign, `rasi` the
/// seven grahas' rasi signs.
fn text_points(graha: Graha, sign: Rashi, rasi: &[Rashi; 7]) -> f64 {
    let attributes = graha.attributes();
    if attributes.exaltation.is_some_and(|at| at.sign == sign) || attributes.own.contains(&sign) {
        return 20.0;
    }
    let lord = sign.attributes().lord;
    let at = |g: Graha| {
        GRAHAS
            .iter()
            .zip(rasi)
            .find_map(|(x, s)| (*x == g).then_some(*s))
    };
    let temporary = match (at(graha), at(lord)) {
        (Some(from), Some(to)) => {
            let distance = (to as u8 + 12 - from as u8) % 12 + 1;
            if FRIENDLY_HOUSES.contains(&distance) {
                Relationship::Friend
            } else {
                Relationship::Enemy
            }
        }
        _ => Relationship::Neutral,
    };
    match compound(natural(graha, sign), temporary) {
        Relationship::GreatFriend => 18.0,
        Relationship::Friend => 15.0,
        Relationship::Enemy => 7.0,
        Relationship::GreatEnemy => 5.0,
        _ => 10.0,
    }
}

/// The recording engine's fraction of full strength for a graha in a varga
/// sign: its Saptavargaja virupas by natural friendship over 45.
fn engine_fraction(graha: Graha, sign: Rashi) -> f64 {
    let attributes = graha.attributes();
    let virupas = if attributes.exaltation.is_some_and(|at| at.sign == sign) {
        45.0
    } else if attributes.debilitation.is_some_and(|at| at.sign == sign) {
        0.0
    } else if attributes
        .moolatrikona
        .is_some_and(|span| span.sign == sign)
        || attributes.own.contains(&sign)
    {
        30.0
    } else {
        match natural(graha, sign) {
            Relationship::Friend => 15.0,
            Relationship::Enemy => 3.75,
            _ => 7.5,
        }
    };
    virupas / 45.0
}

impl VimshopakaReading {
    /// A chart's Vimshopaka, each varga scored under `scoring`.
    #[must_use]
    pub fn of(chart: &VimshopakaChart, scoring: Vimshopaka) -> VimshopakaReading {
        let [rasi, ..] = chart.signs;
        let grahas = GRAHAS
            .iter()
            .enumerate()
            .map(|(index, graha)| {
                // Each varga's fraction of full strength, scored once.
                let fractions = chart.signs.map(|row| {
                    let sign = row.get(index).copied().unwrap_or(Rashi::Aries);
                    if scoring == Vimshopaka::SaptavargajaVirupas {
                        engine_fraction(*graha, sign)
                    } else {
                        text_points(*graha, sign, &rasi) / 20.0
                    }
                });
                let [shadvarga, saptavarga, dashavarga, shodashavarga] = WEIGHTS.map(|weights| {
                    let sum = fractions
                        .iter()
                        .zip(weights)
                        .filter(|(_, halves)| *halves != 0)
                        .fold(0.0, |sum, (fraction, halves)| {
                            sum + fraction * (f64::from(halves) / 2.0)
                        });
                    if scoring == Vimshopaka::SaptavargajaVirupas {
                        // The engine's own arithmetic: normalised by the
                        // total weight and back, rounded half up.
                        ((sum / 20.0 * 20.0) * 100.0 + 0.5).floor() / 100.0
                    } else {
                        sum
                    }
                });
                GrahaVimshopaka {
                    graha: *graha,
                    shadvarga,
                    saptavarga,
                    dashavarga,
                    shodashavarga,
                }
            })
            .collect();
        VimshopakaReading { scoring, grahas }
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::indexing_slicing,
        clippy::float_cmp,
        reason = "tests index what they built and compare exact sums"
    )]

    use super::*;

    #[test]
    fn every_scheme_weighs_20() {
        for weights in WEIGHTS {
            assert_eq!(weights.iter().map(|h| u32::from(*h)).sum::<u32>(), 40);
        }
    }

    #[test]
    fn a_graha_exalted_in_every_varga_scores_20_under_both() {
        let chart = VimshopakaChart::from_fn(|_, graha| {
            graha
                .attributes()
                .exaltation
                .map_or(Rashi::Aries, |at| at.sign)
        });
        for scoring in [Vimshopaka::Bphs, Vimshopaka::SaptavargajaVirupas] {
            for graha in VimshopakaReading::of(&chart, scoring).grahas {
                assert_eq!(
                    [
                        graha.shadvarga,
                        graha.saptavarga,
                        graha.dashavarga,
                        graha.shodashavarga
                    ],
                    [20.0; 4],
                    "{:?} {scoring:?}",
                    graha.graha
                );
            }
        }
    }

    #[test]
    fn a_friend_s_varga_is_a_third_of_the_text_s_under_the_engine() {
        // The Sun in Cancer, the Moon's sign: a natural friend. Under the
        // text the Moon's rasi place decides the temporary half too.
        assert_eq!(engine_fraction(Graha::Sun, Rashi::Cancer), 15.0 / 45.0);
        let mut rasi = [Rashi::Aries; 7];
        // The Moon in the 2nd from the Sun: a temporary friend as well.
        rasi[1] = Rashi::Taurus;
        assert_eq!(text_points(Graha::Sun, Rashi::Cancer, &rasi), 18.0);
        // In the 7th: a temporary enemy, so neutral.
        rasi[1] = Rashi::Libra;
        assert_eq!(text_points(Graha::Sun, Rashi::Cancer, &rasi), 10.0);
        // Debilitation takes nothing from the text's own ladder, and all
        // from the engine's.
        assert_eq!(engine_fraction(Graha::Sun, Rashi::Libra), 0.0);
        assert!(text_points(Graha::Sun, Rashi::Libra, &rasi) > 0.0);
    }
}
