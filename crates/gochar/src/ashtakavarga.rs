//! A transit judged by the natal bindus of the sign it crosses
//! (`03-design/gochar-ashtakavarga.md`).
//!
//! Phaladeepika ch. 23, read on the printed page: a graha's transit over a
//! sign fares by the bindus its own Ashtakavarga put there (v. 11) — from
//! none, death, through loss, expense, dread and fear, to wealth from five
//! and sovereignty at eight; a bindu bears its fruit while the graha
//! crosses the eighth of the sign that belongs to the contributor who gave
//! it, the kakshya (vv. 16 to 19); and a sign whose sarvashtakavarga
//! exceeds 28 is auspicious to transit, below it distressing (v. 20).
//!
//! The crate reads the natal **prastara** as data — for each graha and
//! sign, which contributors gave it a bindu — so it depends on nothing it
//! did not already; the SDK's façade computes it from the natal chart.
//!
//! ```
//! use teistro_core::catalogue::{Graha, Rashi};
//! use teistro_gochar::Transit;
//! use teistro_gochar::ashtakavarga::{
//!     AshtakavargaRules, KakshyaLord, SarvaStanding, transits, LAGNA,
//! };
//!
//! // A prastara whose every cell holds Saturn's and the lagna's bindus.
//! let prastara = [[(1 << Graha::Saturn as u8) | LAGNA; 12]; 7];
//! let mut at = [Transit::new(Rashi::Aries, 1.0); 9];
//! at[Graha::Sun as usize] = Transit::new(Rashi::Leo, 29.0);
//! let read = transits(&prastara, &at, AshtakavargaRules::TEXT);
//! let sun = &read[0];
//! assert_eq!(sun.bindus, 2);
//! // Two bindus is expense by v. 11, not good.
//! assert!(!sun.good);
//! // The last eighth of a sign is the lagna's, and the lagna gave one.
//! assert_eq!(sun.kakshya.lord, KakshyaLord::Lagna);
//! assert!(sun.kakshya_bindu);
//! // Seven grahas of two bindus each: 14, short of 28.
//! assert_eq!((sun.sarva, sun.sarva_standing), (14, SarvaStanding::Below));
//! ```

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Graha;
use teistro_core::settings::{AshtakavargaGoodFrom, Settings};

use crate::Transit;

/// The contributors' bit for the lagna in a prastara cell: the seven grahas
/// take bits 0 to 6 by their ids.
pub const LAGNA: u8 = 1 << 7;

/// For each graha, Sun to Saturn, and each sign, Aries to Pisces, the
/// contributors that gave it a bindu: bit `n` the graha with id `n`, and
/// [`LAGNA`] the lagna.
pub type Prastara = [[u8; 12]; 7];

/// The sarvashtakavarga v. 20 measures a sign against: more is auspicious,
/// less distressing (अष्टाक्ष, eight and two read right to left).
pub const SARVA_MEASURE: u16 = 28;

/// The readings the text leaves open, from the settings' `gochar` group.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AshtakavargaRules {
    /// How many bindus make a transit good (crux C141).
    pub good_from: AshtakavargaGoodFrom,
}

impl AshtakavargaRules {
    /// The text's: good from five, since v. 11 makes four a fear.
    pub const TEXT: AshtakavargaRules = AshtakavargaRules {
        good_from: AshtakavargaGoodFrom::Five,
    };

    /// The readings the settings' `gochar` group gives.
    #[must_use]
    pub const fn of(settings: &Settings) -> AshtakavargaRules {
        AshtakavargaRules {
            good_from: settings.gochar.ashtakavarga_good_from,
        }
    }

    /// The fewest bindus that make a transit good.
    #[must_use]
    pub const fn threshold(self) -> u8 {
        match self.good_from {
            AshtakavargaGoodFrom::Four => 4,
            // `FIVE`, and a reading added later until it says otherwise.
            _ => 5,
        }
    }
}

/// Who lords an eighth of a sign (vv. 18 and 19), in the orbits' order
/// from the sign's start.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum KakshyaLord {
    /// The first eighth.
    Saturn,
    /// The second.
    Jupiter,
    /// The third.
    Mars,
    /// The fourth.
    Sun,
    /// The fifth.
    Venus,
    /// The sixth.
    Mercury,
    /// The seventh.
    Moon,
    /// The last.
    Lagna,
}

impl KakshyaLord {
    /// The eight in a sign's order.
    pub const ALL: [KakshyaLord; 8] = [
        KakshyaLord::Saturn,
        KakshyaLord::Jupiter,
        KakshyaLord::Mars,
        KakshyaLord::Sun,
        KakshyaLord::Venus,
        KakshyaLord::Mercury,
        KakshyaLord::Moon,
        KakshyaLord::Lagna,
    ];

    /// Its bit in a prastara cell.
    #[must_use]
    pub const fn bit(self) -> u8 {
        match self {
            KakshyaLord::Saturn => 1 << Graha::Saturn as u8,
            KakshyaLord::Jupiter => 1 << Graha::Jupiter as u8,
            KakshyaLord::Mars => 1 << Graha::Mars as u8,
            KakshyaLord::Sun => 1 << Graha::Sun as u8,
            KakshyaLord::Venus => 1 << Graha::Venus as u8,
            KakshyaLord::Mercury => 1 << Graha::Mercury as u8,
            KakshyaLord::Moon => 1 << Graha::Moon as u8,
            KakshyaLord::Lagna => LAGNA,
        }
    }
}

/// The eighth of a sign a transit stands in, 3°45′ each (v. 16).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Kakshya {
    /// Which eighth, 1 to 8.
    pub index: u8,
    /// Its lord.
    pub lord: KakshyaLord,
}

impl Kakshya {
    /// The eighth `degrees` into a sign fall in; a value outside 0 to 30 is
    /// taken into it.
    #[must_use]
    pub fn at(degrees: f64) -> Kakshya {
        let into = degrees.rem_euclid(30.0);
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "degrees in [0, 30) over 3.75 is an eighth 0 to 7"
        )]
        let at = ((into / 3.75) as usize).min(7);
        Kakshya {
            index: u8::try_from(at + 1).unwrap_or(8),
            lord: KakshyaLord::ALL
                .get(at)
                .copied()
                .unwrap_or(KakshyaLord::Lagna),
        }
    }
}

/// Where a sign's sarvashtakavarga stands against v. 20's 28.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SarvaStanding {
    /// More than 28: auspicious to transit.
    Above,
    /// Exactly 28, which the verse does not judge (crux C142).
    Even,
    /// Fewer than 28: distressing.
    Below,
}

impl SarvaStanding {
    /// Where `sarva` stands.
    #[must_use]
    pub const fn of(sarva: u16) -> SarvaStanding {
        if sarva > SARVA_MEASURE {
            SarvaStanding::Above
        } else if sarva == SARVA_MEASURE {
            SarvaStanding::Even
        } else {
            SarvaStanding::Below
        }
    }
}

/// One graha's transit judged by the natal Ashtakavarga.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AshtakavargaTransit {
    /// Which graha, Sun to Saturn.
    pub graha: Graha,
    /// The bindus its own Ashtakavarga put in the sign it transits, 0 to 8
    /// (v. 11), unreduced (crux C143).
    pub bindus: u8,
    /// Whether they reach the settings' threshold (crux C141).
    pub good: bool,
    /// The eighth of the sign it stands in (vv. 16 to 19).
    pub kakshya: Kakshya,
    /// Whether that eighth's lord gave a bindu to the sign in this graha's
    /// Ashtakavarga, so that a bindu bears its fruit now.
    pub kakshya_bindu: bool,
    /// The sign's sarvashtakavarga, the seven grahas' bindus together.
    pub sarva: u16,
    /// Where it stands against 28 (v. 20).
    pub sarva_standing: SarvaStanding,
}

/// The seven grahas' transits, Sun to Saturn, judged by the natal
/// `prastara`; the nodes, which have no Ashtakavarga, are not read.
#[must_use]
pub fn transits(
    prastara: &Prastara,
    transits: &[Transit; 9],
    rules: AshtakavargaRules,
) -> [AshtakavargaTransit; 7] {
    let sarva = |sign: usize| -> u16 {
        prastara
            .iter()
            .filter_map(|row| row.get(sign))
            .map(|cell| u16::try_from(cell.count_ones()).unwrap_or(0))
            .sum()
    };
    let seven = [
        Graha::Sun,
        Graha::Moon,
        Graha::Mars,
        Graha::Mercury,
        Graha::Jupiter,
        Graha::Venus,
        Graha::Saturn,
    ];
    seven.map(|graha| {
        let transit = transits
            .get(graha as usize)
            .copied()
            .unwrap_or(Transit::new(teistro_core::catalogue::Rashi::Aries, 0.0));
        let sign = transit.sign as usize % 12;
        let cell = prastara
            .get(graha as usize)
            .and_then(|row| row.get(sign))
            .copied()
            .unwrap_or(0);
        let bindus = u8::try_from(cell.count_ones()).unwrap_or(0);
        let kakshya = Kakshya::at(transit.degrees);
        let total = sarva(sign);
        AshtakavargaTransit {
            graha,
            bindus,
            good: bindus >= rules.threshold(),
            kakshya,
            kakshya_bindu: cell & kakshya.lord.bit() != 0,
            sarva: total,
            sarva_standing: SarvaStanding::of(total),
        }
    })
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index what they built"
    )]

    use super::*;
    use teistro_core::catalogue::Rashi;

    #[test]
    fn a_sign_s_eighths_run_in_the_orbits_order_to_the_lagna() {
        // v. 19: Saturn first, Jupiter second, and so on; the lagna last.
        for (at, lord) in KakshyaLord::ALL.iter().enumerate() {
            let from = 3.75 * f64::from(u8::try_from(at).unwrap());
            for degrees in [from, from + 1.0, from + 3.749] {
                let kakshya = Kakshya::at(degrees);
                assert_eq!(
                    (kakshya.index, kakshya.lord),
                    (u8::try_from(at + 1).unwrap(), *lord),
                    "{degrees}"
                );
            }
        }
        assert_eq!(Kakshya::at(30.0).lord, KakshyaLord::Saturn);
        assert_eq!(Kakshya::at(29.999_999).lord, KakshyaLord::Lagna);
    }

    #[test]
    fn a_kakshya_s_lord_is_the_contributor_whose_bindu_it_times() {
        // Every lord's bit is its contributor's, each distinct, the lagna's
        // the eighth.
        let mut bits: Vec<u8> = KakshyaLord::ALL.iter().map(|lord| lord.bit()).collect();
        bits.sort_unstable();
        bits.dedup();
        assert_eq!(bits.len(), 8);
        assert_eq!(bits.iter().fold(0, |all, bit| all | bit), u8::MAX);
    }

    /// v. 11 makes four a fear, so the text's threshold is five; the other
    /// reading is four (C141).
    #[test]
    fn a_transit_is_good_from_five_bindus_or_as_the_settings_say() {
        let mut prastara = [[0_u8; 12]; 7];
        for (count, sign) in (0..=8_u8).zip(Rashi::ALL) {
            prastara[0][sign as usize] = u8::try_from((1_u16 << count) - 1).unwrap();
        }
        for (count, sign) in (0..=8_u8).zip(Rashi::ALL) {
            let mut at = [Transit::new(Rashi::Aries, 0.0); 9];
            at[0] = Transit::new(sign, 0.0);
            let text = transits(&prastara, &at, AshtakavargaRules::TEXT)[0];
            let four = transits(
                &prastara,
                &at,
                AshtakavargaRules {
                    good_from: AshtakavargaGoodFrom::Four,
                },
            )[0];
            assert_eq!(text.bindus, count);
            assert_eq!(text.good, count >= 5, "{count}");
            assert_eq!(four.good, count >= 4, "{count}");
        }
    }

    #[test]
    fn the_sarvashtakavarga_stands_above_even_or_below_28() {
        assert_eq!(SarvaStanding::of(29), SarvaStanding::Above);
        assert_eq!(SarvaStanding::of(28), SarvaStanding::Even);
        assert_eq!(SarvaStanding::of(27), SarvaStanding::Below);
        // Four bindus in each of the seven rows is 28 exactly.
        let prastara = [[0b1111_u8; 12]; 7];
        let at = [Transit::new(Rashi::Gemini, 12.0); 9];
        let read = transits(&prastara, &at, AshtakavargaRules::TEXT);
        assert!(
            read.iter()
                .all(|one| (one.sarva, one.sarva_standing) == (28, SarvaStanding::Even))
        );
    }

    #[test]
    fn each_graha_reads_its_own_row_and_the_nodes_none() {
        let mut prastara = [[0_u8; 12]; 7];
        for (graha, row) in prastara.iter_mut().enumerate() {
            row[Rashi::Libra as usize] = u8::try_from((1_u16 << (graha + 1)) - 1).unwrap();
        }
        let at = [Transit::new(Rashi::Libra, 5.0); 9];
        let read = transits(&prastara, &at, AshtakavargaRules::TEXT);
        assert_eq!(read.len(), 7);
        for (graha, one) in read.iter().enumerate() {
            assert_eq!(one.graha as usize, graha);
            assert_eq!(usize::from(one.bindus), graha + 1);
        }
        // 5° is the second eighth, Jupiter's; only the rows of five bits
        // or more, Jupiter's own on, hold Jupiter's bit.
        assert!(
            read.iter()
                .all(|one| one.kakshya.lord == KakshyaLord::Jupiter)
        );
        let jupiter = 1 << Graha::Jupiter as u8;
        for one in read {
            assert_eq!(
                one.kakshya_bindu,
                prastara[one.graha as usize][6] & jupiter != 0
            );
        }
    }
}
