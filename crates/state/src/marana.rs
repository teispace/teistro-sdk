//! Marana karaka sthana: the place where a graha brings death near.
//!
//! *Jataka Parijata* ch. 17 vv. 34 to 36 (V. Subrahmanya Sastri, vol. II
//! p. 982) name eight: Jupiter in the 3rd, Mars in the 7th, Saturn in the
//! *janma*, Rahu in the 9th sign, the Moon in the 8th, the Sun in the
//! 12th, Mercury in the 7th and Venus in the *śatru* sign. v. 36 says a
//! graha there joined or aspected by a malefic, in an enemy's or its
//! debilitation sign, or weak, brings the native grief. Those
//! aggravations are other states the chart already carries; this is the
//! place.
//!
//! Houses are counted by sign from the lagna's sign, as two of the eight
//! words (*navamarāśi*, *śatrurāśi*) say outright. Two words are read two
//! ways (crux C768): Saturn's *janma* is the first house, the lagna, by
//! the list it stands in, or "the natal star" as the translator renders
//! it (`state.marana_saturn`); Venus's *śatru* sign is the sixth by the
//! parallel *navamarāśi*, or an enemy's sign as the translator renders it
//! (`state.marana_venus`). The verses name neither Ketu nor the outer
//! planets, and the SDK gives them no place rather than a modern one.
//!
//! ```
//! use teistro_core::catalogue::{Graha, Relationship};
//! use teistro_core::settings::knobs::{MaranaSaturn, MaranaVenus};
//! use teistro_state::marana::{Standing, holds};
//!
//! let in_the = |house| Standing { house, natural: Relationship::Neutral, in_birth_star: false };
//! let reading = (MaranaSaturn::First, MaranaVenus::Sixth);
//! assert_eq!(holds(Graha::Sun, in_the(12), reading.0, reading.1), Some(true));
//! assert_eq!(holds(Graha::Sun, in_the(11), reading.0, reading.1), Some(false));
//! // Ketu has no place in the verses.
//! assert_eq!(holds(Graha::Ketu, in_the(4), reading.0, reading.1), None);
//! ```

use teistro_core::catalogue::{Graha, Relationship};
use teistro_core::settings::knobs::{MaranaSaturn, MaranaVenus};

/// Where a graha stands, as much as the verses read.
///
/// ```
/// use teistro_core::catalogue::Relationship;
/// use teistro_state::marana::Standing;
///
/// let standing = Standing { house: 6, natural: Relationship::Enemy, in_birth_star: false };
/// assert_eq!(standing.house, 6);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Standing {
    /// The house it is in, counted by sign from the lagna's sign, 1 to 12.
    pub house: u8,
    /// How it stands to its sign's lord by the natural friendships.
    pub natural: Relationship,
    /// Whether it stands in the Moon's nakshatra.
    pub in_birth_star: bool,
}

/// The house *Jataka Parijata* ch. 17 vv. 34 and 35 name for a graha, with
/// Saturn's *janma* read as the first and Venus's *śatru* sign as the
/// sixth; `None` for a graha the verses do not name.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_state::marana::house;
///
/// assert_eq!(house(Graha::Jupiter), Some(3));
/// assert_eq!(house(Graha::Uranus), None);
/// ```
#[must_use]
pub const fn house(graha: Graha) -> Option<u8> {
    match graha {
        Graha::Sun => Some(12),
        Graha::Moon => Some(8),
        Graha::Mars | Graha::Mercury => Some(7),
        Graha::Jupiter => Some(3),
        Graha::Venus => Some(6),
        Graha::Saturn => Some(1),
        Graha::Rahu => Some(9),
        _ => None,
    }
}

/// Whether a graha stands in its marana karaka sthana, under the two
/// readings the knobs choose; `None` for a graha the verses do not name.
///
/// ```
/// use teistro_core::catalogue::{Graha, Relationship};
/// use teistro_core::settings::knobs::{MaranaSaturn, MaranaVenus};
/// use teistro_state::marana::{Standing, holds};
///
/// // Venus in Leo, the sign of her natural enemy the Sun, in the 2nd.
/// let venus = Standing { house: 2, natural: Relationship::Enemy, in_birth_star: false };
/// assert_eq!(holds(Graha::Venus, venus, MaranaSaturn::First, MaranaVenus::Sixth), Some(false));
/// assert_eq!(holds(Graha::Venus, venus, MaranaSaturn::First, MaranaVenus::EnemySign), Some(true));
/// ```
#[must_use]
pub fn holds(
    graha: Graha,
    standing: Standing,
    saturn: MaranaSaturn,
    venus: MaranaVenus,
) -> Option<bool> {
    let place = house(graha)?;
    Some(match graha {
        Graha::Saturn if saturn == MaranaSaturn::BirthStar => standing.in_birth_star,
        Graha::Venus if venus == MaranaVenus::EnemySign => standing.natural == Relationship::Enemy,
        _ => standing.house == place,
    })
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    const DEFAULTS: (MaranaSaturn, MaranaVenus) = (MaranaSaturn::First, MaranaVenus::Sixth);

    fn standing(house: u8) -> Standing {
        Standing {
            house,
            natural: Relationship::Neutral,
            in_birth_star: false,
        }
    }

    #[test]
    fn each_named_graha_holds_in_exactly_its_verses_house() {
        let verse = [
            (Graha::Jupiter, 3),
            (Graha::Mars, 7),
            (Graha::Saturn, 1),
            (Graha::Rahu, 9),
            (Graha::Moon, 8),
            (Graha::Sun, 12),
            (Graha::Mercury, 7),
            (Graha::Venus, 6),
        ];
        for (graha, place) in verse {
            for house in 1..=12 {
                assert_eq!(
                    holds(graha, standing(house), DEFAULTS.0, DEFAULTS.1),
                    Some(house == place),
                    "{graha} in {house}"
                );
            }
        }
    }

    #[test]
    fn the_unnamed_have_no_place() {
        for graha in [Graha::Ketu, Graha::Uranus, Graha::Neptune, Graha::Pluto] {
            for house in 1..=12 {
                assert_eq!(holds(graha, standing(house), DEFAULTS.0, DEFAULTS.1), None);
            }
        }
    }

    #[test]
    fn saturn_in_the_birth_star_reads_the_star_not_the_house() {
        let in_star = Standing {
            in_birth_star: true,
            ..standing(5)
        };
        let star = MaranaSaturn::BirthStar;
        assert_eq!(holds(Graha::Saturn, in_star, star, DEFAULTS.1), Some(true));
        assert_eq!(
            holds(Graha::Saturn, standing(1), star, DEFAULTS.1),
            Some(false)
        );
        // The knob is Saturn's alone.
        assert_eq!(
            holds(Graha::Moon, standing(8), star, DEFAULTS.1),
            Some(true)
        );
    }

    #[test]
    fn venus_in_an_enemy_sign_reads_the_friendship_not_the_house() {
        let enemy = MaranaVenus::EnemySign;
        let hostile = Standing {
            natural: Relationship::Enemy,
            ..standing(6)
        };
        assert_eq!(holds(Graha::Venus, hostile, DEFAULTS.0, enemy), Some(true));
        assert_eq!(
            holds(Graha::Venus, standing(6), DEFAULTS.0, enemy),
            Some(false)
        );
        // The knob is Venus's alone: Mars in the 7th is in his place
        // whatever its lord is to him.
        let mars = Standing {
            natural: Relationship::Enemy,
            ..standing(7)
        };
        assert_eq!(holds(Graha::Mars, mars, DEFAULTS.0, enemy), Some(true));
        assert_eq!(holds(Graha::Mars, hostile, DEFAULTS.0, enemy), Some(false));
    }
}
