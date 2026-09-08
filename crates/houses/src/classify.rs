//! What kind of house a house is, and who rules it.
//!
//! Four classifications, all universally attested and none of them in
//! the catalogue: the three **quadrants** partition the twelve, and the
//! trikona, dusthana and upachaya cut across them. They ship here
//! because `strength` needs them for the Dig Bala and the Bhava Bala and
//! `rules` needs them for every yoga stated in terms of a kendra or a
//! dusthana, and both would otherwise write their own copy
//! (`03-design/houses-service.md` §6).
//!
//! ```
//! use teistro_houses::classify::{Quadrant, is_dusthana, is_trikona, quadrant};
//!
//! assert_eq!(quadrant(1), Some(Quadrant::Kendra));
//! assert_eq!(quadrant(3), Some(Quadrant::Apoklima));
//! assert!(is_trikona(9) && !is_trikona(10));
//! assert!(is_dusthana(8));
//! // Nothing outside the twelve is a house.
//! assert_eq!(quadrant(0), None);
//! assert_eq!(quadrant(13), None);
//! ```

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Rashi};

/// The houses of a chart.
pub const HOUSES: u8 = 12;

/// The angular houses, where a body is most able to act.
pub const KENDRA: [u8; 4] = [1, 4, 7, 10];

/// The succedent houses.
pub const PANAPARA: [u8; 4] = [2, 5, 8, 11];

/// The cadent houses.
pub const APOKLIMA: [u8; 4] = [3, 6, 9, 12];

/// The trines, which the tradition counts most fortunate.
pub const TRIKONA: [u8; 3] = [1, 5, 9];

/// The houses of difficulty.
pub const DUSTHANA: [u8; 3] = [6, 8, 12];

/// The houses that grow better with time.
pub const UPACHAYA: [u8; 4] = [3, 6, 10, 11];

/// Which third of the wheel a house stands in.
///
/// These three **partition** the twelve, which is why they are one enum
/// and the overlapping classifications are predicates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Quadrant {
    /// Angular: the 1st, 4th, 7th and 10th.
    Kendra,
    /// Succedent: the 2nd, 5th, 8th and 11th.
    Panapara,
    /// Cadent: the 3rd, 6th, 9th and 12th.
    Apoklima,
}

impl Quadrant {
    /// Every quadrant, in the order the wheel meets them.
    pub const ALL: [Quadrant; 3] = [Quadrant::Kendra, Quadrant::Panapara, Quadrant::Apoklima];

    /// The houses this quadrant holds.
    #[must_use]
    pub const fn houses(self) -> [u8; 4] {
        match self {
            Quadrant::Kendra => KENDRA,
            Quadrant::Panapara => PANAPARA,
            Quadrant::Apoklima => APOKLIMA,
        }
    }
}

/// Which quadrant a house stands in, or `None` outside the twelve.
///
/// A number outside 1 to 12 is not an error: it is arithmetic that
/// cannot arise from a chart, and a predicate has nowhere to report one.
#[must_use]
pub fn quadrant(house: u8) -> Option<Quadrant> {
    match house {
        0 => None,
        house if house > HOUSES => None,
        // The wheel repeats every three houses from the first.
        house => Quadrant::ALL.get(usize::from(house - 1) % 3).copied(),
    }
}

/// Whether a house is a trine.
#[must_use]
pub fn is_trikona(house: u8) -> bool {
    TRIKONA.contains(&house)
}

/// Whether a house is one of difficulty.
#[must_use]
pub fn is_dusthana(house: u8) -> bool {
    DUSTHANA.contains(&house)
}

/// Whether a house grows better with time.
#[must_use]
pub fn is_upachaya(house: u8) -> bool {
    UPACHAYA.contains(&house)
}

/// The lord of a house: the lord of the sign it is centred in.
///
/// The **madhya** and not the sandhi, because under an unequal division
/// a house can begin in one sign and be centred in another, and it is
/// the sign it is centred in that the tradition means by "the house's
/// sign". Nothing in the corpus records a house lord, so this is stated
/// rather than measured (`03-design/houses-service.md` §6).
#[must_use]
pub fn lord_of(sign: Rashi) -> Graha {
    sign.attributes().lord
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "tests fail by panicking"
    )]

    use super::{
        APOKLIMA, DUSTHANA, HOUSES, KENDRA, PANAPARA, Quadrant, TRIKONA, UPACHAYA, is_dusthana,
        is_trikona, is_upachaya, lord_of, quadrant,
    };
    use teistro_core::catalogue::{Graha, Rashi};

    #[test]
    fn the_three_quadrants_partition_the_twelve() {
        let mut seen = Vec::new();
        for house in 1..=HOUSES {
            let found = quadrant(house).expect("a house");
            assert!(found.houses().contains(&house), "{house}");
            seen.push(house);
        }
        assert_eq!(seen.len(), usize::from(HOUSES));
        // Each quadrant holds four, and no house is in two.
        for quadrant in Quadrant::ALL {
            assert_eq!(quadrant.houses().len(), 4, "{quadrant:?}");
            for other in Quadrant::ALL {
                if quadrant == other {
                    continue;
                }
                for house in quadrant.houses() {
                    assert!(!other.houses().contains(&house), "{house} in two");
                }
            }
        }
        assert_eq!(KENDRA.len() + PANAPARA.len() + APOKLIMA.len(), 12);
    }

    #[test]
    fn nothing_outside_the_twelve_is_a_house() {
        assert_eq!(quadrant(0), None);
        assert_eq!(quadrant(13), None);
        assert_eq!(quadrant(u8::MAX), None);
        for house in [0, 13, 100, u8::MAX] {
            assert!(!is_trikona(house) && !is_dusthana(house) && !is_upachaya(house));
        }
    }

    #[test]
    fn the_overlapping_kinds_are_the_attested_ones() {
        for house in 1..=HOUSES {
            assert_eq!(is_trikona(house), TRIKONA.contains(&house), "{house}");
            assert_eq!(is_dusthana(house), DUSTHANA.contains(&house), "{house}");
            assert_eq!(is_upachaya(house), UPACHAYA.contains(&house), "{house}");
        }
        // They cut across the quadrants rather than following them: the
        // trines are two cadent houses and one angular, and the sixth is
        // both a dusthana and an upachaya.
        assert_eq!(quadrant(1), Some(Quadrant::Kendra));
        assert_eq!(quadrant(5), Some(Quadrant::Panapara));
        assert_eq!(quadrant(9), Some(Quadrant::Apoklima));
        assert!(is_dusthana(6) && is_upachaya(6));
        // And the first is a trine and an angle at once, which is why the
        // tradition calls it the strongest house there is.
        assert!(is_trikona(1) && quadrant(1) == Some(Quadrant::Kendra));
    }

    #[test]
    fn every_sign_has_a_lord_and_the_luminaries_have_one_each() {
        for sign in Rashi::ALL {
            let lord = lord_of(sign);
            assert!(
                Rashi::ALL.iter().any(|other| lord_of(*other) == lord),
                "{sign:?}"
            );
        }
        assert_eq!(lord_of(Rashi::Leo), Graha::Sun, "the Sun rules one sign");
        assert_eq!(lord_of(Rashi::Cancer), Graha::Moon, "and the Moon one");
        assert_eq!(lord_of(Rashi::Aries), Graha::Mars);
        assert_eq!(lord_of(Rashi::Scorpio), Graha::Mars, "and Mars two");
    }
}
