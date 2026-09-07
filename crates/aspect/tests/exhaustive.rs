//! The whole space, not a sample of it.
//!
//! The conformance corpus records no aspect, so this crate has no
//! recorded answer to be held to (`03-design/aspect-drishti-measured.md`
//! §1). What it has instead is a space small enough to exhaust: twelve
//! signs, twelve houses and twelve grahas, which is 1728 relations and
//! 11 664 ordered pairs of them. Every invariant the design states is
//! checked over all of it.
//!
//! The counts asserted here are the ones `cargo xtask aspect` publishes,
//! computed by a pass that never imports this crate. Two independent
//! derivations of the same tables have to keep agreeing.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    reason = "tests fail by panicking, index their own results and print counts under --nocapture"
)]

use teistro_aspect::conjunction;
use teistro_aspect::drishti::{
    self, HOUSES, SPECIAL, Strength, between, house_count, looking_back, mutually_full, quarters,
    quarters_under, special, special_under,
};
use teistro_aspect::rashi;
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::settings::NodeAspects;

/// The nine grahas a chart of this tradition carries. The catalogue also
/// holds the three outer planets, which take the same table and are
/// checked with the rest.
const CLASSICAL: [Graha; 9] = [
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

/// Every reading the settings offer for a node.
const READINGS: [NodeAspects; 3] = [
    NodeAspects::None,
    NodeAspects::FiveSevenNine,
    NodeAspects::ThreeSevenEleven,
];

#[test]
fn every_graha_against_every_house() {
    let mut cells = 0;
    for graha in Graha::ALL {
        let mut seen = 0;
        let mut full = 0;
        for houses in 1..=HOUSES {
            let strength = quarters(graha, houses);
            cells += 1;
            seen += usize::from(strength.is_any());
            full += usize::from(strength.is_full());
            // The first is never aspected, and a strength is one of five.
            if houses == 1 {
                assert_eq!(strength, Strength::None, "{graha:?} on its own sign");
            }
            assert!(Strength::ALL.contains(&strength), "{graha:?} {houses}");
            assert_eq!(
                Strength::of_quarters(strength.quarters()),
                Some(strength),
                "{graha:?} {houses}"
            );
        }
        assert_eq!(seen, 7, "{graha:?} aspects seven houses");
        assert_eq!(full, 1 + special(graha).len(), "{graha:?}");
    }
    assert_eq!(cells, Graha::ALL.len() * usize::from(HOUSES));
    println!("{cells} cells of the drishti table, all decided");
}

#[test]
fn a_relation_read_from_either_end_agrees_over_every_pair_of_signs() {
    let mut pairs = 0;
    for from in Rashi::ALL {
        for to in Rashi::ALL {
            pairs += 1;
            let there = house_count(from, to);
            let back = house_count(to, from);
            assert!((1..=HOUSES).contains(&there), "{from:?} {to:?}");
            assert_eq!(looking_back(there), back, "{from:?} {to:?}");
            assert_eq!(looking_back(back), there, "{from:?} {to:?}");
            // A sign is its own first, either way about.
            if from == to {
                assert_eq!(there, 1);
            }
            // And the seventh is the only self-complementary count.
            if there == back {
                assert!(there == 1 || there == 7, "{there}");
            }
        }
    }
    assert_eq!(pairs, 144);
}

#[test]
fn the_rashi_drishti_is_three_wide_mutual_and_never_reflexive() {
    let mut pairs = 0;
    for from in Rashi::ALL {
        let reached = rashi::aspected(from);
        assert_eq!(reached.len(), 3, "{from:?}");
        for to in Rashi::ALL {
            if !rashi::aspects(from, to) {
                continue;
            }
            pairs += 1;
            assert!(rashi::aspects(to, from), "{from:?} and {to:?}");
            assert_ne!(from, to, "no sign aspects itself");
            assert!(reached.contains(&to), "{from:?} lists {to:?}");
        }
    }
    assert_eq!(pairs, 36, "twelve signs times three");
}

#[test]
fn a_mutual_full_aspect_is_the_seventh_or_the_two_configurations() {
    let mut total = 0;
    let mut beyond = 0;
    let mut mars_saturn = 0;
    for first in CLASSICAL {
        for second in CLASSICAL {
            for from in Rashi::ALL {
                for to in Rashi::ALL {
                    if !mutually_full(first, from, second, to) {
                        continue;
                    }
                    total += 1;
                    if house_count(from, to) == 7 {
                        continue;
                    }
                    beyond += 1;
                    match (first, second) {
                        (Graha::Jupiter, Graha::Jupiter) => {}
                        (Graha::Mars, Graha::Saturn) => {
                            assert_eq!(house_count(from, to), 4);
                            mars_saturn += 1;
                        }
                        (Graha::Saturn, Graha::Mars) => {
                            assert_eq!(house_count(from, to), 10);
                            mars_saturn += 1;
                        }
                        pair => panic!("{pair:?} across the {}th", house_count(from, to)),
                    }
                }
            }
        }
    }
    // The pass's own numbers (§2 of the measured page).
    assert_eq!(total, 1020, "the pass measured a thousand and twenty");
    assert_eq!(beyond, 48, "and forty-eight beyond the seventh");
    assert_eq!(mars_saturn, 24, "half of them Mars and Saturn");
    println!("{total} mutual full pairs, {beyond} beyond the seventh");
}

#[test]
fn jupiter_never_faces_itself_in_a_chart() {
    // The Jupiter case of the finding above is arithmetic and not
    // astrology: a graha stands in one sign. The chart layer's `mutual`
    // takes bodies and not positions, so the case cannot be built —
    // this test is the reason that shape was chosen.
    for from in Rashi::ALL {
        for to in Rashi::ALL {
            if from == to {
                assert!(!mutually_full(Graha::Jupiter, from, Graha::Jupiter, to));
            } else {
                // Two different signs and the same body: impossible.
                assert_eq!(
                    mutually_full(Graha::Jupiter, from, Graha::Jupiter, to),
                    house_count(from, to) == 5
                        || house_count(from, to) == 7
                        || house_count(from, to) == 9,
                    "{from:?} {to:?}"
                );
            }
        }
    }
}

#[test]
fn a_node_reading_changes_nodes_and_nothing_else() {
    for reading in READINGS {
        for graha in Graha::ALL {
            let is_node = drishti::NODES.contains(&graha);
            for houses in 1..=HOUSES {
                let under = quarters_under(graha, houses, reading);
                if is_node {
                    let raised = special_under(graha, reading).contains(&houses);
                    if raised {
                        assert_eq!(under, Strength::Full, "{graha:?} {houses} {reading:?}");
                    } else {
                        // Everything else is the general table, which for
                        // a node is the table with no specials at all.
                        assert_eq!(
                            under,
                            quarters_under(graha, houses, NodeAspects::None),
                            "{graha:?} {houses} {reading:?}"
                        );
                    }
                } else {
                    assert_eq!(
                        under,
                        quarters(graha, houses),
                        "{graha:?} {houses} {reading:?}"
                    );
                }
            }
        }
    }
    // And the widest reading gives a node one house more than anything
    // else has, because the eleventh is outside the general table.
    for node in drishti::NODES {
        let seen = (1..=HOUSES)
            .filter(|h| quarters_under(node, *h, NodeAspects::ThreeSevenEleven).is_any())
            .count();
        assert_eq!(seen, 8, "{node:?}");
    }
}

#[test]
fn a_conjunction_is_a_sign_and_a_drishti_is_never_one() {
    for from in Rashi::ALL {
        for to in Rashi::ALL {
            let together = conjunction::together(from, to);
            assert_eq!(together, from == to);
            if together {
                for graha in Graha::ALL {
                    assert_eq!(
                        between(graha, from, to),
                        Strength::None,
                        "{graha:?} does not aspect its own sign"
                    );
                }
            }
        }
    }
}

#[test]
fn the_two_systems_are_different_relations_over_every_pair_of_signs() {
    let mut both = 0;
    let mut graha_only = 0;
    let mut rashi_only = 0;
    for from in Rashi::ALL {
        for to in Rashi::ALL {
            if from == to {
                continue;
            }
            // A graha with no special aspect is the fairest comparison.
            let graha = between(Graha::Sun, from, to).is_any();
            let sign = rashi::aspects(from, to);
            match (graha, sign) {
                (true, true) => both += 1,
                (true, false) => graha_only += 1,
                (false, true) => rashi_only += 1,
                (false, false) => {}
            }
        }
    }
    assert!(
        graha_only > 0 && rashi_only > 0,
        "neither contains the other"
    );
    println!("{both} shared, {graha_only} graha only, {rashi_only} rashi only");
}

#[test]
fn the_specials_are_three_grahas_and_two_houses_each() {
    assert_eq!(SPECIAL.len(), 3);
    for (graha, houses) in SPECIAL {
        assert_eq!(houses.len(), 2);
        assert_eq!(special(graha), houses.as_slice());
        // A special house is one the general table already reaches, so
        // the specials raise a value rather than adding a house.
        for house in houses {
            assert!(
                quarters(Graha::Sun, house).is_any(),
                "{graha:?} {house} is in the general table"
            );
        }
        // And no special is the seventh, which is full for everyone.
        assert!(!houses.contains(&7), "{graha:?}");
    }
    // Every graha aspects seven houses whatever its specials, because a
    // special raises rather than adds.
    for graha in Graha::ALL {
        assert_eq!(
            (1..=HOUSES)
                .filter(|h| quarters(graha, *h).is_any())
                .count(),
            7,
            "{graha:?}"
        );
    }
}
