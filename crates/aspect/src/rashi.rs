//! The Jaimini rashi drishti: a relation between **signs**, with no body
//! in it.
//!
//! A movable sign aspects the three fixed signs but the one next to it; a
//! fixed sign the three movable but the one before it; a dual sign the
//! other three dual signs. So every sign aspects exactly three, and —
//! unlike a graha drishti — **the relation always looks back**.
//!
//! That mutuality is the whole difference between the two systems, and
//! the reason this module keeps them apart rather than offering one
//! where a caller asked for the other: over the corpus's 6696 ordered
//! pairs of bodies the two agree on 4053 and each sees relations the
//! other does not (`03-design/aspect-drishti-measured.md` §3, §4).
//!
//! ```
//! use teistro_aspect::rashi::{aspects, aspected};
//! use teistro_core::catalogue::Rashi;
//!
//! // Aries is movable and reaches the fixed signs but the one next to it.
//! assert_eq!(aspected(Rashi::Aries), [Rashi::Leo, Rashi::Scorpio, Rashi::Aquarius]);
//! assert!(!aspects(Rashi::Aries, Rashi::Taurus));
//! // And whatever it reaches, reaches back.
//! assert!(aspects(Rashi::Leo, Rashi::Aries));
//! ```

use teistro_core::catalogue::{Modality, Rashi};

/// How many signs a sign aspects, which the rule makes the same for
/// every one of the twelve.
pub const ASPECTED: usize = 3;

/// Whether one sign aspects another.
#[must_use]
pub fn aspects(from: Rashi, to: Rashi) -> bool {
    let ahead = crate::drishti::house_count(from, to);
    match (from.attributes().modality, to.attributes().modality) {
        // The movable sign misses the one immediately after it.
        (Modality::Chara, Modality::Sthira) => ahead != 2,
        // The fixed sign misses the one immediately before it.
        (Modality::Sthira, Modality::Chara) => ahead != 12,
        // A dual sign reaches the other dual signs and not itself.
        (Modality::Dwiswabhava, Modality::Dwiswabhava) => from != to,
        _ => false,
    }
}

/// The three signs a sign aspects, in zodiacal order from Aries.
///
/// The rule gives every sign exactly three, which
/// `every_sign_aspects_exactly_three_and_never_itself` proves over all
/// twelve, so the array is always filled.
#[must_use]
pub fn aspected(sign: Rashi) -> [Rashi; ASPECTED] {
    let mut found = [sign; ASPECTED];
    for (slot, other) in found
        .iter_mut()
        .zip(Rashi::ALL.into_iter().filter(|other| aspects(sign, *other)))
    {
        *slot = other;
    }
    found
}

/// The three signs that aspect a sign, which are the three it aspects:
/// the relation is symmetric.
#[must_use]
pub fn aspecting(sign: Rashi) -> [Rashi; ASPECTED] {
    aspected(sign)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::{ASPECTED, aspected, aspecting, aspects};
    use teistro_core::catalogue::{Modality, Rashi};

    #[test]
    fn every_sign_aspects_exactly_three_and_never_itself() {
        for sign in Rashi::ALL {
            let seen = Rashi::ALL.iter().filter(|to| aspects(sign, **to)).count();
            assert_eq!(seen, ASPECTED, "{sign:?}");
            assert!(!aspects(sign, sign), "{sign:?}");
            assert_eq!(aspected(sign).len(), ASPECTED);
        }
    }

    #[test]
    fn the_relation_always_looks_back() {
        let mut pairs = 0;
        for from in Rashi::ALL {
            for to in Rashi::ALL {
                if aspects(from, to) {
                    pairs += 1;
                    assert!(aspects(to, from), "{from:?} and {to:?}");
                }
            }
        }
        assert_eq!(pairs, 36, "twelve signs times three");
        for sign in Rashi::ALL {
            assert_eq!(aspecting(sign), aspected(sign), "{sign:?}");
        }
    }

    #[test]
    fn a_movable_sign_reaches_the_fixed_signs_but_the_next() {
        assert_eq!(
            aspected(Rashi::Aries),
            [Rashi::Leo, Rashi::Scorpio, Rashi::Aquarius]
        );
        assert!(!aspects(Rashi::Aries, Rashi::Taurus), "the next one");
        for to in aspected(Rashi::Aries) {
            assert_eq!(to.attributes().modality, Modality::Sthira);
        }
    }

    #[test]
    fn a_fixed_sign_reaches_the_movable_but_the_one_before() {
        assert_eq!(
            aspected(Rashi::Taurus),
            [Rashi::Cancer, Rashi::Libra, Rashi::Capricorn]
        );
        assert!(!aspects(Rashi::Taurus, Rashi::Aries), "the one before");
        for to in aspected(Rashi::Taurus) {
            assert_eq!(to.attributes().modality, Modality::Chara);
        }
    }

    #[test]
    fn a_dual_sign_reaches_the_other_dual_signs() {
        assert_eq!(
            aspected(Rashi::Gemini),
            [Rashi::Virgo, Rashi::Sagittarius, Rashi::Pisces]
        );
        for to in aspected(Rashi::Gemini) {
            assert_eq!(to.attributes().modality, Modality::Dwiswabhava);
        }
        assert!(!aspects(Rashi::Gemini, Rashi::Leo), "no fixed sign");
    }

    #[test]
    fn it_is_not_the_graha_drishti() {
        use crate::drishti::{between, quarters};
        use teistro_core::catalogue::Graha;

        // The seventh is a full graha drishti and no rashi drishti at
        // all, because opposite signs share a modality.
        assert!(quarters(Graha::Sun, 7).is_full());
        assert!(!aspects(Rashi::Aries, Rashi::Libra));
        // And a rashi drishti reaches the fifth, which the Sun sees at a
        // half.
        assert!(aspects(Rashi::Aries, Rashi::Leo));
        assert!(!between(Graha::Sun, Rashi::Aries, Rashi::Leo).is_full());
    }
}
