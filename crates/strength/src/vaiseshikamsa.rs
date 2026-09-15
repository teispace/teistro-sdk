//! The Vaiseshikamsa: the names a graha earns by the count of good vargas it
//! holds in each scheme of divisions (BPHS ch. 6 vv. 42 to 53).
//!
//! A varga is good when the graha stands there in its exaltation sign, its
//! moolatrikona sign or its own sign, or in a sign owned by the lord of a
//! kendra from the arudha lagna. Two to six good vargas of the shadvarga name
//! Kimshuka to Kundala, and the saptavarga adds Mukuta for seven; the
//! dashavarga runs Parijata to Shridhama and the shodashavarga Bhedaka to
//! Shrivallabha. The verses withhold the auspiciousness of a combust, defeated
//! or weak graha, or one in a bad avastha such as Shayana; the caller says
//! which grahas are impaired, and the names stand beside the flag (crux C77).
//!
//! The four schemes are the Vimshopaka's, which weighs the same vargas.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Rashi, Vaiseshikamsa};

use crate::ashtakavarga::GRAHAS;
use crate::vimshopaka::{VimshopakaChart, WEIGHTS};

/// The shadvarga's and saptavarga's names from two good vargas.
const SAPTAVARGA_LADDER: [Vaiseshikamsa; 6] = [
    Vaiseshikamsa::Kimshuka,
    Vaiseshikamsa::Vyanjana,
    Vaiseshikamsa::Chamara,
    Vaiseshikamsa::Chatra,
    Vaiseshikamsa::Kundala,
    Vaiseshikamsa::Mukuta,
];

/// The dashavarga's names from two good vargas.
const DASHAVARGA_LADDER: [Vaiseshikamsa; 9] = [
    Vaiseshikamsa::Parijata,
    Vaiseshikamsa::Uttama,
    Vaiseshikamsa::Gopura,
    Vaiseshikamsa::Simhasana,
    Vaiseshikamsa::Paravata,
    Vaiseshikamsa::Devaloka,
    Vaiseshikamsa::Brahmaloka,
    Vaiseshikamsa::Shakravahana,
    Vaiseshikamsa::Shridhama,
];

/// The shodashavarga's names from two good vargas.
const SHODASHAVARGA_LADDER: [Vaiseshikamsa; 15] = [
    Vaiseshikamsa::Bhedaka,
    Vaiseshikamsa::Kusuma,
    Vaiseshikamsa::Nagapushpa,
    Vaiseshikamsa::Kanduka,
    Vaiseshikamsa::Kerala,
    Vaiseshikamsa::Kalpavriksha,
    Vaiseshikamsa::Chandanavana,
    Vaiseshikamsa::Purnachandra,
    Vaiseshikamsa::Uchchaishrava,
    Vaiseshikamsa::Dhanvantari,
    Vaiseshikamsa::Suryakanta,
    Vaiseshikamsa::Vidruma,
    Vaiseshikamsa::Chakrasimhasana,
    Vaiseshikamsa::Goloka,
    Vaiseshikamsa::Shrivallabha,
];

/// What a Vaiseshikamsa reads of a chart.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VaiseshikamsaChart {
    /// The seven grahas' signs in the sixteen vargas, as the Vimshopaka
    /// holds them.
    pub signs: VimshopakaChart,
    /// The arudha lagna's sign, whose kendras' lords make a sign good.
    pub arudha_lagna: Rashi,
    /// Which grahas, Sun to Saturn, are combust, defeated, in Shayana or
    /// otherwise impaired, and so earn their names without their
    /// auspiciousness.
    pub impaired: [bool; 7],
}

/// A graha's standing in one scheme.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Standing {
    /// How many of the scheme's vargas are good for it.
    pub good_vargas: u8,
    /// The name that count earns, from two good vargas.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub name: Option<Vaiseshikamsa>,
}

/// One graha's Vaiseshikamsa.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GrahaVaiseshikamsa {
    /// Which graha.
    pub graha: Graha,
    /// Over the six vargas.
    pub shadvarga: Standing,
    /// Over the seven.
    pub saptavarga: Standing,
    /// Over the ten.
    pub dashavarga: Standing,
    /// Over the sixteen.
    pub shodashavarga: Standing,
    /// Whether it is impaired, its names then not auspicious.
    pub impaired: bool,
}

/// A chart's Vaiseshikamsa.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VaiseshikamsaReading {
    /// Each graha's, Sun to Saturn.
    pub grahas: Vec<GrahaVaiseshikamsa>,
}

/// Whether a sign is good for a graha: its exaltation, moolatrikona or own
/// sign, or owned by one of the lords of the arudha lagna's kendras.
fn good(graha: Graha, sign: Rashi, kendra_lords: [Graha; 4]) -> bool {
    let attributes = graha.attributes();
    attributes.exaltation.is_some_and(|e| e.sign == sign)
        || attributes.moolatrikona.is_some_and(|m| m.sign == sign)
        || attributes.own.contains(&sign)
        || kendra_lords.contains(&sign.attributes().lord)
}

/// The name a count of good vargas earns on a ladder starting at two.
fn name(ladder: &[Vaiseshikamsa], good_vargas: u8) -> Option<Vaiseshikamsa> {
    ladder
        .iter()
        .find(|member| member.attributes().good_vargas == good_vargas)
        .copied()
}

impl VaiseshikamsaReading {
    /// A chart's Vaiseshikamsa.
    #[must_use]
    pub fn of(chart: &VaiseshikamsaChart) -> VaiseshikamsaReading {
        let kendra_lords = [0_u16, 3, 6, 9].map(|step| {
            Rashi::from_id((chart.arudha_lagna as u16 + step) % 12)
                .unwrap_or(Rashi::Aries)
                .attributes()
                .lord
        });
        let grahas = GRAHAS
            .iter()
            .zip(chart.impaired)
            .enumerate()
            .map(|(index, (graha, impaired))| {
                let goods: Vec<bool> = chart
                    .signs
                    .signs
                    .iter()
                    .map(|row| {
                        row.get(index)
                            .is_some_and(|sign| good(*graha, *sign, kendra_lords))
                    })
                    .collect();
                let [shad, sapta, dasha, shodasha] = WEIGHTS.map(|weights| {
                    let count = weights
                        .iter()
                        .zip(&goods)
                        .filter(|(weight, is_good)| **weight != 0 && **is_good)
                        .count();
                    u8::try_from(count).unwrap_or(u8::MAX)
                });
                let standing = |good_vargas: u8, ladder: &[Vaiseshikamsa]| Standing {
                    good_vargas,
                    name: name(ladder, good_vargas),
                };
                GrahaVaiseshikamsa {
                    graha: *graha,
                    shadvarga: standing(shad, SAPTAVARGA_LADDER.get(..5).unwrap_or_default()),
                    saptavarga: standing(sapta, &SAPTAVARGA_LADDER),
                    dashavarga: standing(dasha, &DASHAVARGA_LADDER),
                    shodashavarga: standing(shodasha, &SHODASHAVARGA_LADDER),
                    impaired,
                }
            })
            .collect();
        VaiseshikamsaReading { grahas }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::indexing_slicing, reason = "tests index what they built")]

    use super::*;

    fn chart(sign: impl Fn(usize, Graha) -> Rashi, arudha_lagna: Rashi) -> VaiseshikamsaChart {
        let mut signs = [[Rashi::Aries; 7]; 16];
        for (v, row) in signs.iter_mut().enumerate() {
            for (slot, graha) in row.iter_mut().zip(GRAHAS) {
                *slot = sign(v, graha);
            }
        }
        VaiseshikamsaChart {
            signs: VimshopakaChart { signs },
            arudha_lagna,
            impaired: [false; 7],
        }
    }

    #[test]
    fn a_graha_at_home_in_every_varga_crowns_every_ladder() {
        // The Sun in Leo throughout; the arudha lagna's kendras' lords make
        // nothing else good, since only Leo is asked.
        let chart = chart(|_, _| Rashi::Leo, Rashi::Taurus);
        let sun = VaiseshikamsaReading::of(&chart).grahas[0];
        assert_eq!(
            [
                sun.shadvarga.good_vargas,
                sun.saptavarga.good_vargas,
                sun.dashavarga.good_vargas,
                sun.shodashavarga.good_vargas
            ],
            [6, 7, 10, 16]
        );
        assert_eq!(sun.shadvarga.name, Some(Vaiseshikamsa::Kundala));
        assert_eq!(sun.saptavarga.name, Some(Vaiseshikamsa::Mukuta));
        assert_eq!(sun.dashavarga.name, Some(Vaiseshikamsa::Shridhama));
        assert_eq!(sun.shodashavarga.name, Some(Vaiseshikamsa::Shrivallabha));
    }

    #[test]
    fn the_arudha_lagna_s_kendra_lords_make_a_sign_good() {
        // The Moon in Aries in the rasi chart and Libra in every other varga:
        // neither is her exaltation, moolatrikona or own sign.
        let moon = |al| {
            let signs = |v: usize, _| if v == 0 { Rashi::Aries } else { Rashi::Libra };
            VaiseshikamsaReading::of(&chart(signs, al)).grahas[1]
        };
        // Gemini's kendras are Mercury's and Jupiter's: nothing is good.
        assert_eq!(moon(Rashi::Gemini).shodashavarga.good_vargas, 0);
        assert_eq!(moon(Rashi::Gemini).shodashavarga.name, None);
        // Cancer's are the Moon's, Venus's, Saturn's and Mars's: all sixteen.
        assert_eq!(moon(Rashi::Cancer).shodashavarga.good_vargas, 16);
    }

    #[test]
    fn one_good_varga_earns_no_name() {
        // The Moon at home in Cancer in the rasi chart alone, in Gemini
        // elsewhere, under Aries's kendras (Mars, the Moon, Venus, Saturn).
        let signs = |v: usize, _| if v == 0 { Rashi::Cancer } else { Rashi::Gemini };
        let moon = VaiseshikamsaReading::of(&chart(signs, Rashi::Aries)).grahas[1];
        assert_eq!(moon.shadvarga.good_vargas, 1);
        assert_eq!(moon.shadvarga.name, None);
    }

    #[test]
    fn the_shadvarga_stops_at_kundala() {
        // Six good vargas in the shadvarga name Kundala; its ladder has no
        // Mukuta for a count it cannot reach.
        assert_eq!(name(&SAPTAVARGA_LADDER[..5], 7), None);
        assert_eq!(name(&SAPTAVARGA_LADDER, 7), Some(Vaiseshikamsa::Mukuta));
        assert_eq!(name(&DASHAVARGA_LADDER, 1), None);
    }
}
