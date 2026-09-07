//! The graha drishti: a body's gaze, counted in houses from the sign it
//! stands in.
//!
//! One table. A graha aspects the sign seven from its own fully, the
//! fourth and eighth at three quarters, the fifth and ninth at a half,
//! and the third and tenth at a quarter; Mars raises its fourth and
//! eighth to full, Jupiter its fifth and ninth, Saturn its third and
//! tenth. Nothing else is aspected, **the first included**: a graha does
//! not aspect the sign it stands in, which is why a conjunction is a
//! relation of its own ([`crate::conjunction`]) and not a drishti of no
//! houses.
//!
//! The count is from the **sign**, which is what the tradition's "the
//! seventh from it" means. Counted from the bhava it is a different
//! relation, and over the conformance corpus the two are
//! indistinguishable because every recorded placement is whole-sign
//! (`03-design/aspect-drishti-measured.md` §4).
//!
//! ```
//! use teistro_aspect::drishti::{Strength, between, quarters};
//! use teistro_core::catalogue::{Graha, Rashi};
//!
//! // Every graha aspects the seventh fully.
//! assert_eq!(between(Graha::Sun, Rashi::Aries, Rashi::Libra), Strength::Full);
//! // Mars alone reaches its fourth in full; the Sun only three quarters.
//! assert_eq!(between(Graha::Mars, Rashi::Aries, Rashi::Cancer), Strength::Full);
//! assert_eq!(
//!     between(Graha::Sun, Rashi::Aries, Rashi::Cancer),
//!     Strength::ThreeQuarters
//! );
//! // And nothing aspects the sign it stands in.
//! assert_eq!(quarters(Graha::Sun, 1), Strength::None);
//! ```

use serde::Serialize;
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::error::Error;
use teistro_core::settings::NodeAspects;

/// The signs of the zodiac, which is how many houses a count runs
/// through before it comes back to itself.
pub const HOUSES: u8 = 12;

/// The virupas a full aspect is worth, which is how the tradition
/// measures a drishti when it measures it at all.
pub const FULL_VIRUPAS: u16 = 60;

/// The table the SDK ships, by the key `aspect.drishti_table` names.
pub const PARASHARA: &str = "PARASHARA";

/// Every table the SDK ships, in the order [`table`] names them when it
/// refuses one it does not have.
pub const SHIPPED: [&str; 1] = [PARASHARA];

/// How strongly a graha looks at a sign, in the quarters the tradition
/// counts a drishti in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Strength {
    /// No aspect at all.
    None,
    /// A quarter aspect: the third and tenth.
    Quarter,
    /// A half aspect: the fifth and ninth.
    Half,
    /// A three-quarter aspect: the fourth and eighth.
    ThreeQuarters,
    /// A full aspect: the seventh, and a special graha's own two houses.
    Full,
}

impl Strength {
    /// Every member, weakest first.
    pub const ALL: [Strength; 5] = [
        Strength::None,
        Strength::Quarter,
        Strength::Half,
        Strength::ThreeQuarters,
        Strength::Full,
    ];

    /// The quarters of a full aspect this is, 0 to 4.
    #[must_use]
    pub const fn quarters(self) -> u8 {
        match self {
            Strength::None => 0,
            Strength::Quarter => 1,
            Strength::Half => 2,
            Strength::ThreeQuarters => 3,
            Strength::Full => 4,
        }
    }

    /// The one this many quarters make, or `None` past a full aspect.
    #[must_use]
    pub const fn of_quarters(quarters: u8) -> Option<Strength> {
        match quarters {
            0 => Some(Strength::None),
            1 => Some(Strength::Quarter),
            2 => Some(Strength::Half),
            3 => Some(Strength::ThreeQuarters),
            4 => Some(Strength::Full),
            _ => None,
        }
    }

    /// This aspect's value in virupas, **as the whole-sign table gives
    /// it**: a quarter is fifteen.
    ///
    /// This is not the sphuta drishti, which interpolates between the
    /// houses and which the SDK does not ship because no source in the
    /// project gives its construction (crux C45,
    /// `03-design/aspect-and-drishti.md` §8). It is the same unit and
    /// the same value at each house, which is the specification any
    /// future construction has to meet.
    #[must_use]
    pub const fn virupas(self) -> u16 {
        self.quarters() as u16 * (FULL_VIRUPAS / 4)
    }

    /// Whether this is a full aspect.
    #[must_use]
    pub const fn is_full(self) -> bool {
        matches!(self, Strength::Full)
    }

    /// Whether there is any aspect at all.
    #[must_use]
    pub const fn is_any(self) -> bool {
        !matches!(self, Strength::None)
    }
}

/// Which house a graha aspects, and with what strength, before the three
/// special grahas raise their own.
const TABLE: [(u8, Strength); 7] = [
    (3, Strength::Quarter),
    (4, Strength::ThreeQuarters),
    (5, Strength::Half),
    (7, Strength::Full),
    (8, Strength::ThreeQuarters),
    (9, Strength::Half),
    (10, Strength::Quarter),
];

/// The three grahas with a full aspect beyond the seventh, and the two
/// houses each raises.
pub const SPECIAL: [(Graha, [u8; 2]); 3] = [
    (Graha::Mars, [4, 8]),
    (Graha::Jupiter, [5, 9]),
    (Graha::Saturn, [3, 10]),
];

/// The houses a graha aspects beyond the seventh, which is empty for
/// every graha but Mars, Jupiter and Saturn.
#[must_use]
pub fn special(graha: Graha) -> &'static [u8] {
    SPECIAL
        .iter()
        .find(|(who, _)| *who == graha)
        .map_or(&[], |(_, houses)| houses.as_slice())
}

/// The two shadow grahas, whose special houses are a settings choice
/// rather than a fixed table.
pub const NODES: [Graha; 2] = [Graha::Rahu, Graha::Ketu];

/// The houses a node aspects fully beyond the seventh, under each
/// reading `aspect.node_aspects` offers.
///
/// The knob makes a node a special graha with a *chosen* pair of houses,
/// exactly as Mars, Jupiter and Saturn have fixed ones. Under
/// [`NodeAspects::None`] it has none and takes the general table like
/// everything else; under the other two it raises its own pair to full.
/// [`NodeAspects::ThreeSevenEleven`] reaches the eleventh, which is
/// outside the general table, so a node under that reading aspects eight
/// houses and not seven.
///
/// Nothing in the corpus prefers a reading — there is no recorded aspect
/// to compare — so the root's `NONE` stays the default because it claims
/// least (`03-design/aspect-drishti-measured.md` §6), and a reading this
/// crate has not been taught claims least too rather than guessing.
#[must_use]
pub const fn node_special(reading: NodeAspects) -> &'static [u8] {
    match reading {
        NodeAspects::FiveSevenNine => &[5, 9],
        NodeAspects::ThreeSevenEleven => &[3, 11],
        // `NodeAspects` is non-exhaustive, so a reading added to the
        // catalogue and not yet read here claims least rather than
        // guessing at houses nobody has written down.
        NodeAspects::None | _ => &[],
    }
}

/// The houses a graha aspects fully beyond the seventh, with a node's
/// read from the settings.
#[must_use]
pub fn special_under(graha: Graha, reading: NodeAspects) -> &'static [u8] {
    if NODES.contains(&graha) {
        node_special(reading)
    } else {
        special(graha)
    }
}

/// How strongly a graha aspects the sign that is `houses` from its own,
/// with a node's special houses read from the settings.
#[must_use]
pub fn quarters_under(graha: Graha, houses: u8, reading: NodeAspects) -> Strength {
    if special_under(graha, reading).contains(&houses) {
        return Strength::Full;
    }
    if NODES.contains(&graha) {
        return general(houses);
    }
    quarters(graha, houses)
}

/// How strongly a graha standing in one sign aspects another, with a
/// node's special houses read from the settings.
#[must_use]
pub fn between_under(graha: Graha, from: Rashi, to: Rashi, reading: NodeAspects) -> Strength {
    quarters_under(graha, house_count(from, to), reading)
}

/// The table before any graha's specials are applied.
fn general(houses: u8) -> Strength {
    TABLE
        .iter()
        .find(|(house, _)| *house == houses)
        .map_or(Strength::None, |(_, strength)| *strength)
}

/// How strongly a graha aspects the sign that is `houses` from its own,
/// counting inclusively from one.
///
/// A count outside 1 to 12 is [`Strength::None`] rather than an error:
/// no two signs produce one, so it is arithmetic that cannot arise from
/// a chart.
#[must_use]
pub fn quarters(graha: Graha, houses: u8) -> Strength {
    if special(graha).contains(&houses) {
        return Strength::Full;
    }
    general(houses)
}

/// Which house of the first sign the second is, counting inclusively
/// from one: the count the whole tradition reckons a relation by.
#[must_use]
pub fn house_count(from: Rashi, to: Rashi) -> u8 {
    let (from, to) = (index(from), index(to));
    (to + HOUSES - from) % HOUSES + 1
}

/// The house the target counts the source as, given the house the source
/// counts the target as: the same relation read from the other end.
///
/// The seventh looks back at the seventh, which is why a mutual full
/// aspect is nearly always that one; the fourth looks back at the tenth,
/// which is the exception Mars and Saturn make
/// (`03-design/aspect-and-drishti.md` §5).
#[must_use]
pub const fn looking_back(houses: u8) -> u8 {
    if houses == 0 || houses > HOUSES {
        return houses;
    }
    (HOUSES + 1 - houses) % HOUSES + 1
}

/// How strongly a graha standing in one sign aspects another.
#[must_use]
pub fn between(graha: Graha, from: Rashi, to: Rashi) -> Strength {
    quarters(graha, house_count(from, to))
}

/// Whether two grahas standing in two signs aspect each other fully.
#[must_use]
pub fn mutually_full(first: Graha, first_sign: Rashi, second: Graha, second_sign: Rashi) -> bool {
    between(first, first_sign, second_sign).is_full()
        && between(second, second_sign, first_sign).is_full()
}

/// The table a settings key names, matched without regard to case.
///
/// The SDK ships one, and names it in the refusal rather than leaving a
/// caller to guess. A profile that names another fails here and not in
/// a consumer's application.
///
/// # Errors
///
/// `UNSUPPORTED` for a table the SDK does not ship, naming the ones it
/// does.
pub fn table(key: &str) -> Result<&'static str, Error> {
    SHIPPED
        .into_iter()
        .find(|shipped| key.eq_ignore_ascii_case(shipped))
        .ok_or_else(|| {
            Error::unsupported(format!(
                "no drishti table `{key}`; the SDK ships {}",
                SHIPPED
                    .iter()
                    .map(|shipped| format!("`{shipped}`"))
                    .collect::<Vec<_>>()
                    .join(" and ")
            ))
            .with_field("aspect.drishti_table")
        })
}

/// A sign's index, 0 for Aries.
fn index(sign: Rashi) -> u8 {
    u8::try_from(sign as u16).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "tests fail by panicking"
    )]

    use super::{
        FULL_VIRUPAS, NODES, PARASHARA, SHIPPED, SPECIAL, Strength, between, between_under,
        house_count, looking_back, mutually_full, node_special, quarters, quarters_under, special,
        special_under, table,
    };
    use teistro_core::catalogue::{Graha, Rashi};
    use teistro_core::settings::NodeAspects;

    #[test]
    fn the_table_is_the_classical_one() {
        assert_eq!(quarters(Graha::Sun, 7), Strength::Full);
        assert_eq!(quarters(Graha::Sun, 4), Strength::ThreeQuarters);
        assert_eq!(quarters(Graha::Sun, 8), Strength::ThreeQuarters);
        assert_eq!(quarters(Graha::Sun, 5), Strength::Half);
        assert_eq!(quarters(Graha::Sun, 9), Strength::Half);
        assert_eq!(quarters(Graha::Sun, 3), Strength::Quarter);
        assert_eq!(quarters(Graha::Sun, 10), Strength::Quarter);
        for house in [1, 2, 6, 11, 12] {
            assert_eq!(quarters(Graha::Sun, house), Strength::None, "{house}");
        }
    }

    #[test]
    fn nothing_aspects_the_sign_it_stands_in_and_nothing_outside_the_zodiac() {
        for graha in Graha::ALL {
            assert_eq!(quarters(graha, 1), Strength::None, "{graha:?}");
            assert_eq!(quarters(graha, 0), Strength::None, "{graha:?}");
            assert_eq!(quarters(graha, 13), Strength::None, "{graha:?}");
            assert_eq!(quarters(graha, 255), Strength::None, "{graha:?}");
        }
    }

    #[test]
    fn every_graha_aspects_seven_houses_and_the_specials_raise_two() {
        for graha in Graha::ALL {
            let seen = (1..=12).filter(|h| quarters(graha, *h).is_any()).count();
            assert_eq!(seen, 7, "{graha:?} aspects seven");
            let full = (1..=12).filter(|h| quarters(graha, *h).is_full()).count();
            assert_eq!(full, 1 + special(graha).len(), "{graha:?}");
        }
        for (graha, houses) in SPECIAL {
            for house in houses {
                assert_eq!(quarters(graha, house), Strength::Full, "{graha:?} {house}");
            }
        }
        assert!(special(Graha::Sun).is_empty(), "only three are special");
    }

    #[test]
    fn a_house_is_counted_inclusively_from_the_sign() {
        assert_eq!(house_count(Rashi::Aries, Rashi::Aries), 1);
        assert_eq!(house_count(Rashi::Aries, Rashi::Libra), 7);
        assert_eq!(house_count(Rashi::Libra, Rashi::Aries), 7);
        assert_eq!(house_count(Rashi::Pisces, Rashi::Aries), 2, "over the end");
        assert_eq!(house_count(Rashi::Aries, Rashi::Pisces), 12);
    }

    #[test]
    fn a_relation_read_from_the_other_end_is_its_complement() {
        assert_eq!(looking_back(7), 7, "the seventh looks back at itself");
        assert_eq!(looking_back(4), 10);
        assert_eq!(looking_back(10), 4);
        assert_eq!(looking_back(5), 9);
        assert_eq!(looking_back(1), 1);
        for house in 1..=12_u8 {
            assert_eq!(looking_back(looking_back(house)), house, "{house}");
        }
        // And it agrees with counting the other way round for real signs.
        for from in Rashi::ALL {
            for to in Rashi::ALL {
                assert_eq!(
                    looking_back(house_count(from, to)),
                    house_count(to, from),
                    "{from:?} {to:?}"
                );
            }
        }
    }

    #[test]
    fn a_mutual_full_aspect_is_the_seventh_or_mars_and_saturn() {
        let mut beyond = 0;
        for first in Graha::ALL {
            for second in Graha::ALL {
                for from in Rashi::ALL {
                    for to in Rashi::ALL {
                        if !mutually_full(first, from, second, to) {
                            continue;
                        }
                        if house_count(from, to) == 7 {
                            continue;
                        }
                        beyond += 1;
                        let pair = (first, second);
                        assert!(
                            pair == (Graha::Jupiter, Graha::Jupiter)
                                || pair == (Graha::Mars, Graha::Saturn)
                                || pair == (Graha::Saturn, Graha::Mars),
                            "{pair:?} across the {}th",
                            house_count(from, to)
                        );
                    }
                }
            }
        }
        // The measured page's own count (§2), recomputed here.
        assert_eq!(beyond, 48, "the pass measured forty-eight");
    }

    #[test]
    fn a_strength_is_quarters_and_virupas_of_the_same_thing() {
        assert_eq!(Strength::Full.virupas(), FULL_VIRUPAS);
        assert_eq!(Strength::ThreeQuarters.virupas(), 45);
        assert_eq!(Strength::Half.virupas(), 30);
        assert_eq!(Strength::Quarter.virupas(), 15);
        assert_eq!(Strength::None.virupas(), 0);
        for strength in Strength::ALL {
            assert_eq!(Strength::of_quarters(strength.quarters()), Some(strength));
            assert_eq!(
                u16::from(strength.quarters()) * 15,
                strength.virupas(),
                "{strength:?}"
            );
        }
        assert_eq!(Strength::of_quarters(5), None, "past a full aspect");
        assert!(Strength::Full > Strength::ThreeQuarters, "and it orders");
        assert!(Strength::Full.is_any() && !Strength::None.is_any());
    }

    #[test]
    fn a_table_the_sdk_does_not_ship_is_refused_by_name() {
        assert_eq!(table(PARASHARA).unwrap(), PARASHARA);
        assert_eq!(table("parashara").unwrap(), PARASHARA, "whatever the case");
        let error = table("SOMEONE_ELSES").expect_err("no such table");
        assert!(error.message.contains("SOMEONE_ELSES"), "{error}");
        for shipped in SHIPPED {
            assert!(error.message.contains(shipped), "{error}");
        }
        assert_eq!(error.field(), Some("aspect.drishti_table"));
    }

    #[test]
    fn a_node_takes_the_general_table_and_the_specials_the_settings_give_it() {
        for node in NODES {
            // The root's reading: no specials, so the general table.
            assert!(special_under(node, NodeAspects::None).is_empty());
            assert_eq!(
                quarters_under(node, 7, NodeAspects::None),
                Strength::Full,
                "{node:?} still aspects the seventh"
            );
            assert_eq!(
                quarters_under(node, 5, NodeAspects::None),
                Strength::Half,
                "{node:?} at a half, as everything else"
            );
            // The five-seven-nine reading raises its trines to full.
            assert_eq!(
                quarters_under(node, 5, NodeAspects::FiveSevenNine),
                Strength::Full
            );
            assert_eq!(
                quarters_under(node, 9, NodeAspects::FiveSevenNine),
                Strength::Full
            );
            assert_eq!(
                quarters_under(node, 4, NodeAspects::FiveSevenNine),
                Strength::ThreeQuarters,
                "and leaves the rest alone"
            );
            // The three-seven-eleven reading reaches a house the general
            // table does not, so a node then aspects eight.
            assert_eq!(
                quarters_under(node, 11, NodeAspects::ThreeSevenEleven),
                Strength::Full
            );
            assert_eq!(
                quarters(node, 11),
                Strength::None,
                "which nothing else does"
            );
            let seen = (1..=12)
                .filter(|h| quarters_under(node, *h, NodeAspects::ThreeSevenEleven).is_any())
                .count();
            assert_eq!(seen, 8, "{node:?}");
        }
        // A reading changes nothing for a body that is not a node.
        for reading in [
            NodeAspects::None,
            NodeAspects::FiveSevenNine,
            NodeAspects::ThreeSevenEleven,
        ] {
            for graha in Graha::ALL.into_iter().filter(|g| !NODES.contains(g)) {
                for house in 1..=12 {
                    assert_eq!(
                        quarters_under(graha, house, reading),
                        quarters(graha, house),
                        "{graha:?} {house} {reading:?}"
                    );
                }
            }
        }
        assert_eq!(node_special(NodeAspects::None).len(), 0);
        assert_eq!(node_special(NodeAspects::FiveSevenNine), &[5, 9]);
        assert_eq!(node_special(NodeAspects::ThreeSevenEleven), &[3, 11]);
        assert_eq!(
            between_under(
                Graha::Rahu,
                Rashi::Aries,
                Rashi::Leo,
                NodeAspects::FiveSevenNine
            ),
            Strength::Full
        );
    }

    #[test]
    fn between_two_signs_is_the_table_read_at_their_distance() {
        assert_eq!(
            between(Graha::Mars, Rashi::Aries, Rashi::Cancer),
            Strength::Full,
            "Mars's fourth"
        );
        assert_eq!(
            between(Graha::Saturn, Rashi::Aries, Rashi::Gemini),
            Strength::Full,
            "Saturn's third"
        );
        assert_eq!(
            between(Graha::Jupiter, Rashi::Aries, Rashi::Leo),
            Strength::Full,
            "Jupiter's fifth"
        );
        assert_eq!(
            between(Graha::Venus, Rashi::Aries, Rashi::Aries),
            Strength::None
        );
    }
}
