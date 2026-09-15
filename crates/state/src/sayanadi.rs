//! The Sayanadi avasthas: the twelve states a graha is in by the chart's
//! numbers, and the three sub-states of each (BPHS ch. 45 vv. 30 to 37).
//!
//! The graha's nakshatra number (Ashwini 1) is multiplied by its own number
//! (the Sun 1 to Saturn 7) and by the number of the navamsha it stands in
//! within its sign (1 to 9, as the translator's note settles against a
//! reading of degrees); the Moon's nakshatra number, the ghatis of birth and
//! the lagna's sign number (Aries 1) are added; the remainder of twelve is
//! the state, Shayana 1 to Nidra 12 and so nothing Nidra.
//!
//! The sub-state reads the native's name: the state's number squared, with
//! the anka of the name's first syllable added, leaves a remainder of
//! twelve; the graha's additive (the Sun 5, the Moon 2, Mars 2, Mercury 3,
//! Jupiter 5, Venus 3, Saturn 3, Rahu and Ketu 4) is added, and a remainder
//! of three is Drishti for 1, Cheshta for 2 and Vicheshta for nothing. A
//! chart has no name, so it carries the sub-state for each of the five
//! ankas and a caller picks theirs.
//!
//! Two numbers the verses leave open are knobs (crux C78): which count of
//! the ghatis of birth, and what Rahu and Ketu multiply by.

use serde::{Deserialize, Serialize};
use teistro_core::angle::Nas;
use teistro_core::catalogue::{AvasthaCheshta, AvasthaSayanadi, Graha, Nakshatra, Rashi};
use teistro_core::error::Error;
use teistro_core::settings::{SayanadiGhatis, SayanadiNodes};
use teistro_time::ghati::GhatiPala;

/// The anka of the first syllable of a name, 1 to 5 (the translator's note
/// to BPHS ch. 45 v. 37).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(try_from = "u8", into = "u8")]
pub struct Anka(u8);

impl Anka {
    /// The five ankas, 1 to 5.
    pub const ALL: [Anka; 5] = [Anka(1), Anka(2), Anka(3), Anka(4), Anka(5)];

    /// An anka.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` outside 1 to 5.
    ///
    /// ```
    /// use teistro_state::sayanadi::Anka;
    /// assert_eq!(Anka::try_new(3).map(Anka::get), Ok(3));
    /// assert!(Anka::try_new(6).is_err());
    /// ```
    pub fn try_new(anka: u8) -> Result<Anka, Error> {
        if (1..=5).contains(&anka) {
            Ok(Anka(anka))
        } else {
            Err(Error::invalid_arg(format!(
                "anka: {anka} is not a syllable's anka; it is 1 to 5"
            )))
        }
    }

    /// Its value, 1 to 5.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for Anka {
    type Error = Error;

    fn try_from(anka: u8) -> Result<Anka, Error> {
        Anka::try_new(anka)
    }
}

impl From<Anka> for u8 {
    fn from(anka: Anka) -> u8 {
        anka.0
    }
}

/// A graha's Sayanadi state, with its sub-state for every anka.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Sayanadi {
    /// The state.
    pub avastha: AvasthaSayanadi,
    /// The sub-state under a name whose first syllable's anka is 1 to 5, in
    /// that order.
    pub cheshtas: [AvasthaCheshta; 5],
}

impl Sayanadi {
    /// The sub-state under a name of this anka.
    ///
    /// ```
    /// use teistro_core::catalogue::{AvasthaCheshta, AvasthaSayanadi, Graha};
    /// use teistro_state::sayanadi::{Anka, Sayanadi};
    /// // Shayana (1): 1 + 1 = 2, and the Sun's 5 makes 7, a remainder of 1.
    /// let sun = Sayanadi::of_state(Graha::Sun, AvasthaSayanadi::Shayana);
    /// assert_eq!(sun.cheshta(Anka::ALL[0]), AvasthaCheshta::Drishti);
    /// ```
    #[must_use]
    pub fn cheshta(&self, anka: Anka) -> AvasthaCheshta {
        let index = usize::from(anka.get() - 1);
        self.cheshtas
            .get(index)
            .copied()
            .unwrap_or(AvasthaCheshta::Vicheshta)
    }

    /// A graha's sub-states in a state, for every anka.
    #[must_use]
    pub fn of_state(graha: Graha, avastha: AvasthaSayanadi) -> Sayanadi {
        Sayanadi {
            avastha,
            cheshtas: Anka::ALL.map(|anka| cheshta(graha, avastha, anka)),
        }
    }
}

/// What a chart gives every graha's Sayanadi alike.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SayanadiChart {
    /// The Moon's nakshatra.
    pub moon: Nakshatra,
    /// The ishtakaal: how far into the chart's day birth is.
    pub ishtakaal: GhatiPala,
    /// The lagna's sign.
    pub lagna: Rashi,
}

/// The sub-state a graha in a state is in under a name of an anka.
#[must_use]
pub fn cheshta(graha: Graha, avastha: AvasthaSayanadi, anka: Anka) -> AvasthaCheshta {
    let number = u32::from(avastha.id()) + 1;
    let additive = match graha {
        Graha::Sun | Graha::Jupiter => 5,
        Graha::Moon | Graha::Mars => 2,
        Graha::Rahu | Graha::Ketu => 4,
        _ => 3,
    };
    let remainder = ((number * number + u32::from(anka.get())) % 12 + additive) % 3;
    AvasthaCheshta::ALL
        .into_iter()
        .find(|member| u32::from(member.attributes().remainder) == remainder)
        .unwrap_or(AvasthaCheshta::Vicheshta)
}

/// A graha's Sayanadi, or `None` for a body BPHS gives no number: the
/// outer planets.
#[must_use]
pub fn sayanadi(
    graha: Graha,
    longitude: Nas,
    chart: &SayanadiChart,
    ghatis: SayanadiGhatis,
    nodes: SayanadiNodes,
) -> Option<Sayanadi> {
    let number: u32 = match graha {
        Graha::Sun => 1,
        Graha::Moon => 2,
        Graha::Mars => 3,
        Graha::Mercury => 4,
        Graha::Jupiter => 5,
        Graha::Venus => 6,
        Graha::Saturn => 7,
        Graha::Rahu => 8,
        Graha::Ketu if nodes == SayanadiNodes::SharedWithRahu => 8,
        Graha::Ketu => 9,
        _ => return None,
    };
    let nakshatra = u32::from(longitude.nakshatra().id()) + 1;
    // The navamsha within the sign: every sign holds nine.
    let navamsha = longitude.division_index(108) % 9 + 1;
    let elapsed = u32::from(chart.ishtakaal.ghati);
    let ghati_count = if ghatis == SayanadiGhatis::Running {
        elapsed + u32::from(chart.ishtakaal.pala != 0 || chart.ishtakaal.vipala != 0)
    } else {
        elapsed
    };
    let sum = nakshatra * number * navamsha
        + u32::from(chart.moon.id())
        + 1
        + ghati_count
        + u32::from(chart.lagna.id())
        + 1;
    // A remainder of nothing is the twelfth, Nidra.
    let index = (sum + 11) % 12;
    let avastha = AvasthaSayanadi::ALL
        .get(usize::try_from(index).ok()?)
        .copied()?;
    Some(Sayanadi::of_state(graha, avastha))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests unwrap what they built")]

    use super::*;
    use teistro_core::quantity::Degrees;

    fn at(degrees: f64) -> Nas {
        Nas::from_degrees(Degrees::try_new(degrees).unwrap())
    }

    const CHART: SayanadiChart = SayanadiChart {
        moon: Nakshatra::Rohini,
        ishtakaal: GhatiPala {
            ghati: 20,
            pala: 30,
            vipala: 0,
        },
        lagna: Rashi::Leo,
    };

    #[test]
    fn the_verses_arithmetic_worked_by_hand() {
        // The Sun at 10° Leo: Magha (10), its fourth navamsha of Leo.
        // 10 x 1 x 4 = 40; Rohini 4, 20 ghatis, Leo 5: 69, a remainder of 9,
        // Bhojana.
        let sun = sayanadi(
            Graha::Sun,
            at(130.0),
            &CHART,
            SayanadiGhatis::Elapsed,
            SayanadiNodes::NineGrahaOrder,
        )
        .unwrap();
        assert_eq!(sun.avastha, AvasthaSayanadi::Bhojana);
        // The running ghati is the 21st: 70, a remainder of 10.
        let running = sayanadi(
            Graha::Sun,
            at(130.0),
            &CHART,
            SayanadiGhatis::Running,
            SayanadiNodes::NineGrahaOrder,
        )
        .unwrap();
        assert_eq!(running.avastha, AvasthaSayanadi::Nrityalipsa);
        // Bhojana 9: 81 + 1 = 82, a remainder of 10, the Sun's 5 makes 15,
        // nothing over three: Vicheshta. With anka 2, 83, 11, 16: Drishti.
        assert_eq!(
            sun.cheshta(Anka::try_new(1).unwrap()),
            AvasthaCheshta::Vicheshta
        );
        assert_eq!(
            sun.cheshta(Anka::try_new(2).unwrap()),
            AvasthaCheshta::Drishti
        );
    }

    #[test]
    fn a_remainder_of_nothing_is_nidra() {
        // Saturn at 0° Aries: Ashwini (1), first navamsha: 7. With Rohini 4,
        // 20 ghatis and Leo 5: 36, a remainder of nothing.
        let saturn = sayanadi(
            Graha::Saturn,
            at(0.0),
            &CHART,
            SayanadiGhatis::Elapsed,
            SayanadiNodes::NineGrahaOrder,
        )
        .unwrap();
        assert_eq!(saturn.avastha, AvasthaSayanadi::Nidra);
    }

    #[test]
    fn the_ninth_navamsha_of_a_sign_counts_nine() {
        // Mars at 29° Aries: Krittika (3), the ninth navamsha: 3 x 3 x 9 = 81;
        // with 29, 110, a remainder of 2, Upaveshana.
        let mars = sayanadi(
            Graha::Mars,
            at(29.0),
            &CHART,
            SayanadiGhatis::Elapsed,
            SayanadiNodes::NineGrahaOrder,
        )
        .unwrap();
        assert_eq!(mars.avastha, AvasthaSayanadi::Upaveshana);
    }

    #[test]
    fn ketu_multiplies_by_the_knob_s_number() {
        // Ketu at 0° Aries: 1 x 9 x 1 + 29 = 38, a remainder of 2; shared
        // with Rahu, 8 + 29 = 37, a remainder of 1.
        let ketu = |nodes| {
            sayanadi(Graha::Ketu, at(0.0), &CHART, SayanadiGhatis::Elapsed, nodes)
                .unwrap()
                .avastha
        };
        assert_eq!(
            ketu(SayanadiNodes::NineGrahaOrder),
            AvasthaSayanadi::Upaveshana
        );
        assert_eq!(
            ketu(SayanadiNodes::SharedWithRahu),
            AvasthaSayanadi::Shayana
        );
    }

    #[test]
    fn the_outer_planets_have_no_number() {
        let uranus = sayanadi(
            Graha::Uranus,
            at(0.0),
            &CHART,
            SayanadiGhatis::Elapsed,
            SayanadiNodes::NineGrahaOrder,
        );
        assert_eq!(uranus, None);
    }

    #[test]
    fn every_state_and_anka_leaves_a_sub_state_by_the_remainder() {
        for graha in [Graha::Sun, Graha::Moon, Graha::Mercury, Graha::Rahu] {
            for avastha in AvasthaSayanadi::ALL {
                for anka in Anka::ALL {
                    let number = u32::from(avastha.id()) + 1;
                    let expected = (number * number + u32::from(anka.get())) % 12;
                    let got = cheshta(graha, avastha, anka);
                    let additive = match graha {
                        Graha::Sun => 5,
                        Graha::Moon => 2,
                        Graha::Mercury => 3,
                        _ => 4,
                    };
                    assert_eq!(
                        u32::from(got.attributes().remainder),
                        (expected + additive) % 3
                    );
                }
            }
        }
    }

    #[test]
    fn an_anka_is_one_to_five() {
        assert!(Anka::try_new(0).is_err());
        assert!(Anka::try_new(6).is_err());
        assert_eq!(Anka::ALL.map(Anka::get), [1, 2, 3, 4, 5]);
        assert_eq!(serde_json::from_str::<Anka>("4").unwrap().get(), 4);
        assert!(serde_json::from_str::<Anka>("9").is_err());
    }
}
