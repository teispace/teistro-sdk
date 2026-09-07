//! Conjunction: co-presence in a sign.
//!
//! In this tradition a conjunction has **no orb**. Two bodies are
//! together if they stand in the same sign, however far apart in it they
//! are, and that is what a yoga means by "with". The degree-measured
//! kind belongs to [`crate::orb`], which a Tajika or western reading
//! wants and which this is deliberately not
//! (`03-design/aspect-and-drishti.md` §6).
//!
//! ```
//! use teistro_aspect::conjunction::{together, within};
//! use teistro_core::catalogue::Rashi;
//!
//! // Twenty-eight degrees apart and still together, because one sign.
//! assert!(together(Rashi::Aries, Rashi::Aries));
//! assert!(!together(Rashi::Aries, Rashi::Taurus));
//! // Two degrees apart across a sign edge: not together, but near.
//! assert!(within(29.0, 31.0, 5.0));
//! ```

use teistro_core::catalogue::Rashi;

/// Whether two bodies share a sign, which is what the tradition means by
/// a conjunction.
#[must_use]
pub const fn together(first: Rashi, second: Rashi) -> bool {
    first as u16 == second as u16
}

/// Whether two longitudes are within an orb of each other, degrees:
/// the degree-measured reading, which is not the classical one.
///
/// A negative or non-finite orb is `false` rather than an error, because
/// this is a predicate and there is no answer to report an error
/// through; [`crate::orb::hits`] is the form that refuses one by name.
#[must_use]
pub fn within(first_deg: f64, second_deg: f64, orb_deg: f64) -> bool {
    orb_deg.is_finite()
        && orb_deg >= 0.0
        && crate::orb::separation(first_deg, second_deg) <= orb_deg
}

#[cfg(test)]
mod tests {
    use super::{together, within};
    use teistro_core::catalogue::Rashi;

    #[test]
    fn a_conjunction_is_a_sign_and_not_a_distance() {
        assert!(together(Rashi::Aries, Rashi::Aries));
        assert!(!together(Rashi::Aries, Rashi::Taurus));
        for sign in Rashi::ALL {
            assert!(together(sign, sign), "{sign:?}");
        }
    }

    #[test]
    fn two_bodies_at_opposite_ends_of_one_sign_are_still_together() {
        // 0°01' Aries and 29°59' Aries are together; 29°59' Aries and
        // 0°01' Taurus are not, though they are two hundred times nearer.
        assert!(together(Rashi::Aries, Rashi::Aries));
        assert!(!together(Rashi::Aries, Rashi::Taurus));
        assert!(!within(0.016, 29.983, 5.0), "and by orb the reverse");
        assert!(within(29.983, 30.016, 5.0));
    }

    #[test]
    fn an_orb_that_makes_no_sense_is_no_conjunction() {
        assert!(!within(0.0, 0.0, -1.0));
        assert!(!within(0.0, 0.0, f64::NAN));
        assert!(within(0.0, 0.0, 0.0), "exactness is inside every orb");
        assert!(within(359.0, 1.0, 2.5), "over the end of the circle");
    }
}
