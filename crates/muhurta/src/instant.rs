//! The clauses of an instant: what the lagna and the grahas' places from
//! it say, as Raman's twenty-one Mahadoshas and their neutralisations
//! state them (*Muhurtha*, ch. V) and his marriage chapter adds (ch. IX).
//!
//! Houses are counted by sign from the lagna's sign, as the texts count
//! them. The malefics are the natural five, the Sun, Mars, Saturn and the
//! nodes; the waning Moon and an afflicted Mercury, which Raman's glossary
//! adds, are conditions of a chart and not of a graha, and are not read.
//!
//! A clause here says nothing of when it holds: the caller passes the
//! window it judged, which is a window over which none of these changes
//! (`03-design/muhurta.md` §4.3).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Nakshatra, Rashi, Tithi, Vara};
use teistro_core::interval::Interval;
use teistro_panchanga::omen::panchaka_remainder;

use crate::clause::{Clause, ClauseKind};

/// The nine grahas, in the catalogue's order, which is the order of
/// [`Sky::grahas`].
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

/// The natural malefics.
const MALEFICS: [Graha; 5] = [
    Graha::Sun,
    Graha::Mars,
    Graha::Saturn,
    Graha::Rahu,
    Graha::Ketu,
];

/// The sky at an instant, in the chart's zodiac.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Sky {
    /// The lagna's longitude, degrees.
    pub lagna_deg: f64,
    /// The nine grahas' longitudes, degrees, in the catalogue's order.
    pub grahas: [f64; 9],
}

/// The limbs running at the instant, which the panchaka remainder reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Limbs {
    /// The tithi.
    pub tithi: Tithi,
    /// The vara.
    pub vara: Vara,
    /// The Moon's nakshatra.
    pub nakshatra: Nakshatra,
}

impl Sky {
    /// The lagna's sign.
    #[must_use]
    pub fn lagna(&self) -> Rashi {
        sign_of(self.lagna_deg)
    }

    /// The lagna's navamsa sign: the ninths of the zodiac counted on from
    /// Aries, which is the Parashari D9.
    #[must_use]
    pub fn navamsa(&self) -> Rashi {
        sign_of(self.lagna_deg * 9.0)
    }

    /// The house a graha stands in, counted by sign from the lagna's, 1
    /// to 12.
    #[must_use]
    pub fn house_of(&self, graha: Graha) -> u8 {
        let at = self
            .grahas
            .get(usize::from(graha.id()))
            .copied()
            .unwrap_or(self.lagna_deg);
        let steps = (sign_of(at).id() + 12 - self.lagna().id()) % 12;
        u8::try_from(steps + 1).unwrap_or(1)
    }

    /// The grahas of a list that stand in a house.
    fn in_house(&self, house: u8, among: &[Graha]) -> Vec<Graha> {
        among
            .iter()
            .copied()
            .filter(|g| self.house_of(*g) == house)
            .collect()
    }
}

/// Every clause of an instant, each over the window `at` the caller
/// judged.
///
/// `limbs` gives the panchaka remainder, and `native_lagna` the ashtama
/// lagna; either may be absent, and its clause is then not read.
///
/// ```
/// use teistro_core::interval::Interval;
/// use teistro_muhurta::ClauseKind;
/// use teistro_muhurta::instant::{Sky, clauses};
///
/// // Libra rising, Mars in Taurus (the 8th), Venus in Pisces (the 6th),
/// // Jupiter in Libra (the lagna); everyone else in Capricorn.
/// let mut grahas = [285.0; 9];
/// grahas[2] = 45.0; // Mars
/// grahas[5] = 345.0; // Venus
/// grahas[4] = 190.0; // Jupiter
/// let sky = Sky { lagna_deg: 185.0, grahas };
/// let found = clauses(&sky, None, None, Interval::literal(0.0, 1.0));
/// assert!(found.iter().any(|c| c.kind == ClauseKind::MarsInEighth {}));
/// assert!(found.iter().any(|c| c.kind == ClauseKind::VenusInSixth {}));
/// // And Jupiter in the lagna is the neutraliser Raman ranks highest.
/// assert!(found.iter().any(|c| matches!(c.kind, ClauseKind::BeneficInLagna { .. }) && c.favourable()));
/// ```
#[must_use]
pub fn clauses(
    sky: &Sky,
    limbs: Option<&Limbs>,
    native_lagna: Option<Rashi>,
    at: Interval,
) -> Vec<Clause> {
    let mut kinds = Vec::new();
    let others: Vec<Graha> = GRAHAS
        .iter()
        .copied()
        .filter(|g| *g != Graha::Moon)
        .collect();

    let (second, twelfth) = (sky.in_house(2, &MALEFICS), sky.in_house(12, &MALEFICS));
    if !second.is_empty() && !twelfth.is_empty() {
        kinds.push(ClauseKind::Kartari { second, twelfth });
    }
    let moon = sky.house_of(Graha::Moon);
    if matches!(moon, 6 | 8 | 12) {
        kinds.push(ClauseKind::MoonInDusthana { house: moon });
    }
    let with = sky.in_house(moon, &others);
    if !with.is_empty() {
        kinds.push(ClauseKind::MoonJoined { with });
    }
    if sky.house_of(Graha::Venus) == 6 {
        kinds.push(ClauseKind::VenusInSixth {});
    }
    if sky.house_of(Graha::Mars) == 8 {
        kinds.push(ClauseKind::MarsInEighth {});
    }
    if let Some(birth) = native_lagna {
        if (sky.lagna().id() + 12 - birth.id()) % 12 == 7 {
            kinds.push(ClauseKind::AshtamaLagna {});
        }
    }
    let navamsa = sky.navamsa();
    let lord = navamsa.attributes().lord;
    if MALEFICS.contains(&lord) {
        kinds.push(ClauseKind::Kunavamsa { navamsa, lord });
    }
    let remainder = limbs.and_then(|limbs| {
        panchaka_remainder(limbs.tithi, limbs.vara, limbs.nakshatra, sky.lagna())
    });
    if let Some(panchaka) = remainder {
        kinds.push(ClauseKind::PanchakaRemainder { panchaka });
    }
    let by = sky.in_house(7, &GRAHAS);
    if !by.is_empty() {
        kinds.push(ClauseKind::SeventhOccupied { by });
    }
    let grahas = sky.in_house(1, &MALEFICS);
    if !grahas.is_empty() {
        kinds.push(ClauseKind::MaleficInLagna { grahas });
    }
    neutralisers(sky, &mut kinds);
    kinds.into_iter().map(|kind| Clause { kind, at }).collect()
}

/// The neutralisations that read the sky (Raman, ch. V, 6, 8, 10 and 11).
fn neutralisers(sky: &Sky, kinds: &mut Vec<ClauseKind>) {
    let grahas = sky.in_house(1, &[Graha::Venus, Graha::Mercury, Graha::Jupiter]);
    if !grahas.is_empty() {
        kinds.push(ClauseKind::BeneficInLagna { grahas });
    }
    let grahas: Vec<Graha> = sky
        .in_house(1, &GRAHAS)
        .into_iter()
        .filter(|g| exalted(sky, *g))
        .collect();
    if !grahas.is_empty() {
        kinds.push(ClauseKind::ExaltedInLagna { grahas });
    }
    let grahas = sky.in_house(11, &[Graha::Sun, Graha::Moon]);
    if !grahas.is_empty() {
        kinds.push(ClauseKind::LuminaryInEleventh { grahas });
    }
    let grahas: Vec<Graha> = [Graha::Jupiter, Graha::Venus]
        .into_iter()
        .filter(|g| matches!(sky.house_of(*g), 1 | 4 | 7 | 10))
        .collect();
    let malefics_placed = MALEFICS
        .iter()
        .all(|g| matches!(sky.house_of(*g), 3 | 6 | 11));
    if !grahas.is_empty() && malefics_placed {
        kinds.push(ClauseKind::KendraBenefics { grahas });
    }
}

/// Whether a graha stands in its sign of exaltation.
fn exalted(sky: &Sky, graha: Graha) -> bool {
    let at = sky.grahas.get(usize::from(graha.id())).copied();
    match (graha.attributes().exaltation, at) {
        (Some(exaltation), Some(at)) => sign_of(at) == exaltation.sign,
        _ => false,
    }
}

/// The sign a longitude falls in.
fn sign_of(deg: f64) -> Rashi {
    let index = deg.rem_euclid(360.0) / 30.0;
    // The floor of a value in [0, 12) is a sign index; the cast cannot
    // truncate anything the floor has not.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a floor in 0..12"
    )]
    let id = index.floor() as u16;
    Rashi::from_id(id.min(11)).unwrap_or(Rashi::Aries)
}

#[cfg(test)]
#[allow(clippy::expect_used, reason = "tests fail by panicking")]
mod tests {
    use super::{Limbs, Sky, clauses};
    use crate::clause::ClauseKind;
    use teistro_core::catalogue::{Graha, Nakshatra, Panchaka, Rashi, Tithi, Vara};
    use teistro_core::interval::Interval;

    /// Every graha in one sign, the lagna in another.
    fn sky(lagna_deg: f64, all: f64) -> Sky {
        Sky {
            lagna_deg,
            grahas: [all; 9],
        }
    }

    fn at() -> Interval {
        Interval::literal(10.0, 10.5)
    }

    fn kinds(sky: &Sky) -> Vec<ClauseKind> {
        clauses(sky, None, None, at())
            .into_iter()
            .map(|c| c.kind)
            .collect()
    }

    #[test]
    fn kartari_needs_a_malefic_on_both_sides() {
        // Aries rising; Saturn in Taurus (2nd), Mars in Pisces (12th).
        let mut s = sky(5.0, 125.0);
        s.grahas[6] = 35.0;
        s.grahas[2] = 355.0;
        assert!(kinds(&s).contains(&ClauseKind::Kartari {
            second: vec![Graha::Saturn],
            twelfth: vec![Graha::Mars],
        }));
        // Only one side hemmed is no kartari.
        s.grahas[2] = 125.0;
        assert!(
            !kinds(&s)
                .iter()
                .any(|k| matches!(k, ClauseKind::Kartari { .. }))
        );
    }

    #[test]
    fn the_moon_alone_in_the_sixth_is_one_clause_and_joined_is_another() {
        // Aries rising; the Moon in Virgo (6th), the others in Leo.
        let mut s = sky(5.0, 125.0);
        s.grahas[1] = 155.0;
        let k = kinds(&s);
        assert!(k.contains(&ClauseKind::MoonInDusthana { house: 6 }));
        assert!(!k.iter().any(|k| matches!(k, ClauseKind::MoonJoined { .. })));
        // Mercury joins it.
        s.grahas[3] = 170.0;
        assert!(kinds(&s).contains(&ClauseKind::MoonJoined {
            with: vec![Graha::Mercury]
        }));
    }

    #[test]
    fn ashtama_lagna_is_the_eighth_from_the_birth_lagna() {
        // Raman's example (ch. V): a groom born with Aquarius rising and a
        // bride with Capricorn; Virgo and Leo are the 8th from them.
        let virgo = sky(155.0, 125.0);
        let found = |birth| {
            clauses(&virgo, None, Some(birth), at())
                .into_iter()
                .any(|c| c.kind == ClauseKind::AshtamaLagna {})
        };
        assert!(found(Rashi::Aquarius));
        assert!(!found(Rashi::Capricorn));
        assert!(
            clauses(&sky(125.0, 0.0), None, Some(Rashi::Capricorn), at())
                .iter()
                .any(|c| c.kind == ClauseKind::AshtamaLagna {})
        );
    }

    #[test]
    fn kunavamsa_reads_the_lagnas_ninth_part() {
        // 0° to 3°20′ of Aries is the Aries navamsa, Mars's: kunavamsa.
        let k = kinds(&sky(1.0, 125.0));
        assert!(k.contains(&ClauseKind::Kunavamsa {
            navamsa: Rashi::Aries,
            lord: Graha::Mars
        }));
        // 3°20′ to 6°40′ is Taurus, Venus's: none.
        assert!(
            !kinds(&sky(5.0, 125.0))
                .iter()
                .any(|k| matches!(k, ClauseKind::Kunavamsa { .. }))
        );
    }

    #[test]
    fn the_panchaka_remainder_reads_the_lagna() {
        // Raman's worked example: Aslesha, the 13th tithi, Sunday, Virgo.
        let limbs = Limbs {
            tithi: Tithi::ShuklaTrayodashi,
            vara: Vara::Ravivara,
            nakshatra: Nakshatra::Ashlesha,
        };
        let virgo = clauses(&sky(155.0, 125.0), Some(&limbs), None, at());
        assert!(virgo.iter().any(|c| c.kind
            == ClauseKind::PanchakaRemainder {
                panchaka: Panchaka::Agni
            }));
    }

    #[test]
    fn an_exalted_graha_in_the_lagna_neutralises() {
        // Capricorn rising with Mars in it: Mars is exalted in Capricorn.
        let mut s = sky(275.0, 125.0);
        s.grahas[2] = 290.0;
        let found = clauses(&s, None, None, at());
        let exalted = found
            .iter()
            .find(|c| matches!(c.kind, ClauseKind::ExaltedInLagna { .. }))
            .expect("Mars exalted in the lagna");
        assert!(exalted.favourable());
        // The same Mars is also a malefic in the lagna: both are reported.
        assert!(found.iter().any(|c| c.kind
            == ClauseKind::MaleficInLagna {
                grahas: vec![Graha::Mars]
            }));
    }

    #[test]
    fn kendra_benefics_need_every_malefic_in_an_upachaya() {
        // Aries rising; Jupiter in Cancer (4th); the Sun, Mars and Saturn
        // in Gemini (3rd), the nodes in Virgo (6th).
        let mut s = sky(5.0, 65.0);
        s.grahas[4] = 95.0;
        s.grahas[1] = 215.0;
        s.grahas[3] = 215.0;
        s.grahas[5] = 215.0;
        s.grahas[7] = 155.0;
        s.grahas[8] = 155.0;
        assert!(kinds(&s).contains(&ClauseKind::KendraBenefics {
            grahas: vec![Graha::Jupiter]
        }));
        // Ketu moved to the 7th breaks it.
        s.grahas[8] = 185.0;
        assert!(
            !kinds(&s)
                .iter()
                .any(|k| matches!(k, ClauseKind::KendraBenefics { .. }))
        );
    }

    #[test]
    fn every_clause_carries_the_window_it_was_judged_over() {
        let found = clauses(&sky(1.0, 1.0), None, None, at());
        assert!(!found.is_empty());
        assert!(found.iter().all(|c| c.at == at()));
    }
}
