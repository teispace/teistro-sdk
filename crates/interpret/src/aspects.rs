//! Which graha looks at which, and how strongly
//! (`03-design/interpret-composers.md` §4).
//!
//! **The first composer that spends translation debt.** Four shipped free
//! because the packs held a message nobody read and the fifth exhausted
//! them; the drishti was the largest thing the SDK computes and no locale
//! could say a word about — not "looks at", not a grading, not the
//! relation. So `sdk.aspect` is a namespace of two messages written for
//! this composer, in English and in Nepali, from the tradition's own
//! vocabulary rather than invented prose: पाद, अर्ध, त्रिपाद and
//! पूर्ण दृष्टि for the quarters a drishti is counted in, and
//! परस्पर दृष्टि for a mutual one. They await the native review the
//! roadmap already requires for `ne` and `hi`, exactly as `sdk.reading`'s
//! six do.
//!
//! The grading belongs to the message and not to this module.
//! [`Strength`](teistro_aspect::Strength) is not a catalogued entity, so
//! `cast` selects on the slot the way `sdk.reading`'s `lifeClass` does,
//! with the arms spelled as `Strength::key()` writes them.
//!
//! What it does not say: how near either end stands to a sign edge, which
//! is a statement about how much an ayanamsha that moved would change the
//! reading rather than about the native; and which drishti table the
//! settings named, which is a setting. Neither has a message in any
//! locale, and neither is a sentence a reading wants.

use teistro_aspect::{Drishti, Strength, mutual_pairs};
use teistro_intl::messages::sdk::aspect;

use crate::Plan;

/// Every drishti the chart holds, and every pair that looks back.
///
/// The casts come first, in the relations' own order, and the mutual pairs
/// after them, so the same chart always gives the same plan. A pair that
/// looks at each other therefore has two `cast` items **and** a `mutual`
/// one: *parasparadrishti* is a named condition the texts read as one
/// thing, which a consumer would otherwise derive by scanning — the same
/// reason `occupants` stands beside `grahaInRashi`.
///
/// ```
/// # use teistro_aspect::{Drishti, Strength};
/// # use teistro_core::boundary::Boundaries;
/// # use teistro_core::catalogue::Graha;
/// # let edge = Boundaries { sign_deg: 15.0, nakshatra_deg: 6.0, pada_deg: 3.0 };
/// # let at = |from, to, houses, strength| Drishti {
/// #     from, to, houses, strength, from_edge: edge, to_edge: edge,
/// # };
/// let relations = [
///     at(Graha::Saturn, Graha::Jupiter, 7, Strength::Full),
///     at(Graha::Jupiter, Graha::Saturn, 7, Strength::Full),
/// ];
/// let plan = teistro_interpret::aspects(&relations);
/// assert_eq!(plan.len(), 3, "two casts and the pair they make");
/// assert_eq!(plan.items[0].key, "sdk.aspect.cast");
/// assert_eq!(plan.items[2].key, "sdk.aspect.mutual");
/// ```
#[must_use]
pub fn aspects(relations: &[Drishti]) -> Plan {
    let mut plan = Plan::default();
    for relation in relations {
        // `Aspects` never stores a relation of no strength, so this branch
        // costs nothing — but the message's catch-all arm is `FULL`, and a
        // catch-all that would say "fully" of no aspect at all is worth one
        // branch to make unreachable rather than merely unlikely.
        if relation.strength == Strength::None {
            continue;
        }
        plan.say(&aspect::Cast {
            from: relation.from,
            to: relation.to,
            strength: String::from(relation.strength.key()),
        });
    }
    for pair in mutual_pairs(relations) {
        plan.say(&aspect::Mutual {
            first: pair.first,
            second: pair.second,
        });
    }
    plan
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        clippy::indexing_slicing,
        reason = "tests unwrap what they build and fail by panicking"
    )]

    use teistro_core::boundary::Boundaries;
    use teistro_core::catalogue::Graha;
    use teistro_intl::Value;

    /// A body well away from every edge. There is no `Default` for this
    /// and there should not be: zero would say "exactly on a boundary".
    const MID_SIGN: Boundaries = Boundaries {
        sign_deg: 15.0,
        nakshatra_deg: 6.0,
        pada_deg: 3.0,
    };

    use super::*;
    use crate::{Item, KEYS};

    /// One relation, with only the fields the composer reads carrying
    /// meaning: the edges are the record's own, so a field added to
    /// `Drishti` does not reach this file with a number it would invent.
    fn at(from: Graha, to: Graha, houses: u8, strength: Strength) -> Drishti {
        Drishti {
            from,
            to,
            houses,
            strength,
            // Nothing here reads an edge: they say how near a boundary
            // the reading stands, which is a fact about the ayanamsha
            // rather than about the native, and no message carries it.
            from_edge: MID_SIGN,
            to_edge: MID_SIGN,
        }
    }

    /// Saturn and Jupiter look at each other fully; Mars looks at Saturn
    /// and is not looked back at.
    fn relations() -> Vec<Drishti> {
        vec![
            at(Graha::Saturn, Graha::Jupiter, 7, Strength::Full),
            at(Graha::Jupiter, Graha::Saturn, 7, Strength::Full),
            at(Graha::Mars, Graha::Saturn, 4, Strength::ThreeQuarters),
        ]
    }

    #[test]
    fn every_drishti_is_cast_and_every_pair_said_once() {
        let plan = aspects(&relations());
        assert_eq!(plan.len(), 4, "three casts and one pair");
        assert_eq!(
            plan.items[0],
            Item::of(&aspect::Cast {
                from: Graha::Saturn,
                to: Graha::Jupiter,
                strength: String::from("FULL"),
            })
        );
        assert_eq!(
            plan.items[2],
            Item::of(&aspect::Cast {
                from: Graha::Mars,
                to: Graha::Saturn,
                strength: String::from("THREE_QUARTERS"),
            })
        );
        assert_eq!(plan.items[3].key, "sdk.aspect.mutual");
    }

    /// The strength crosses as the key the enum writes, which is what the
    /// message's arms are spelled with. A mismatch here renders the
    /// catch-all, so this is the test that keeps a grading honest.
    #[test]
    fn a_strength_crosses_as_its_own_key() {
        for strength in [
            Strength::Quarter,
            Strength::Half,
            Strength::ThreeQuarters,
            Strength::Full,
        ] {
            let plan = aspects(&[at(Graha::Sun, Graha::Moon, 7, strength)]);
            assert_eq!(
                plan.items[0].params.get("strength"),
                Some(&Value::Str(String::from(strength.key()))),
                "{strength:?}"
            );
        }
    }

    /// A relation of no strength is not an aspect, and the message's
    /// catch-all would say "fully" of it.
    #[test]
    fn no_strength_is_not_said_at_all() {
        let plan = aspects(&[at(Graha::Sun, Graha::Moon, 2, Strength::None)]);
        assert!(plan.is_empty(), "{plan:?}");
    }

    /// One direction is not a pair, however strong it is.
    #[test]
    fn a_one_sided_look_makes_no_pair() {
        let plan = aspects(&[at(Graha::Mars, Graha::Saturn, 4, Strength::ThreeQuarters)]);
        assert_eq!(plan.len(), 1);
        assert_eq!(plan.items[0].key, "sdk.aspect.cast");
    }

    /// The edges and the house count are on every relation and in no item:
    /// they say how near a boundary the reading stands, which is a fact
    /// about the ayanamsha rather than about the native.
    #[test]
    fn it_does_not_say_what_no_locale_can_say() {
        let written = serde_json::to_string(&aspects(&relations())).unwrap();
        for claim in ["edge", "sign_deg", "houses", "nakshatra"] {
            assert!(!written.contains(claim), "`{claim}` is in {written}");
        }
    }

    #[test]
    fn it_emits_only_listed_keys() {
        for key in aspects(&relations()).keys() {
            assert!(KEYS.contains(&key), "`{key}` is not in KEYS");
        }
    }

    #[test]
    fn a_plan_is_the_same_bytes_every_time() {
        let once = serde_json::to_string(&aspects(&relations())).unwrap();
        let twice = serde_json::to_string(&aspects(&relations())).unwrap();
        assert_eq!(once, twice);
        let read: Plan = serde_json::from_str(&once).unwrap();
        assert_eq!(read, aspects(&relations()));
    }
}
