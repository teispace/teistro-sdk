//! How near a body stands to a classification boundary.
//!
//! A body within a hair of a sign, nakshatra or pada edge is one whose
//! classification would flip under a slightly different ayanamsha, and
//! the recording engine flags it on one threshold for all three, which
//! the corpus brackets to between 21 and 43 arcseconds
//! (`03-design/state-tables-measured.md` §8).
//!
//! It lives in `core` because two modules want it and it is a fact about
//! the canonical angle rather than about either of them: `state` reports
//! how safe a dignity is, and `aspect` how safe a whole-sign drishti is
//! — a relation that changes all at once at a sign edge
//! (`03-design/aspect-and-drishti.md` §7).
//!
//! **The SDK ships no threshold.** It reports the distance, which is
//! always a fact, and a caller asks whether that is inside whatever
//! tolerance its own provider claims. A constant compiled into the
//! library could not answer the question for two providers of different
//! accuracy, and the corpus's own tolerance file already frames it that
//! way: a classification within the longitude tolerance of a boundary is
//! an edge case and not a failure.

use serde::{Deserialize, Serialize};

use crate::angle::Nas;

/// How far a body stands from the nearest boundary of each division,
/// degrees.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Boundaries {
    /// To the nearer edge of its sign.
    pub sign_deg: f64,
    /// To the nearer edge of its nakshatra.
    pub nakshatra_deg: f64,
    /// To the nearer edge of its pada.
    pub pada_deg: f64,
}

impl Boundaries {
    /// The distances for a longitude.
    #[must_use]
    pub fn of(longitude: Nas) -> Boundaries {
        Boundaries {
            sign_deg: to_edge(longitude, Nas::PER_SIGN),
            nakshatra_deg: to_edge(longitude, Nas::PER_NAKSHATRA),
            pada_deg: to_edge(longitude, Nas::PER_NAKSHATRA / 4),
        }
    }

    /// The nearest of the three, degrees: what a caller checking "is this
    /// classification safe at all" wants.
    #[must_use]
    pub fn nearest_deg(&self) -> f64 {
        self.sign_deg.min(self.nakshatra_deg).min(self.pada_deg)
    }

    /// Whether any classification is inside a caller's own tolerance,
    /// degrees — which is the caller's to state, because it is the
    /// provider's accuracy and not a constant of the tradition.
    #[must_use]
    pub fn is_near(&self, tolerance_deg: f64) -> bool {
        self.nearest_deg() <= tolerance_deg
    }
}

/// How far an angle is from the nearer edge of a division, degrees.
///
/// Integer arithmetic on the canonical angle: the division widths are
/// exact in nanoarcseconds and `360/27` is not a double.
fn to_edge(longitude: Nas, width: i64) -> f64 {
    if width <= 0 {
        return 0.0;
    }
    let inside = longitude.get().rem_euclid(width);
    Nas::new(inside.min(width - inside)).to_degrees()
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "tests fail by panicking"
    )]

    use super::Boundaries;
    use crate::angle::Nas;
    use crate::quantity::Degrees;

    fn at(degrees: f64) -> Nas {
        Nas::from_degrees(Degrees::try_new(degrees.rem_euclid(360.0)).expect("finite"))
    }

    #[test]
    fn a_body_on_a_sign_edge_is_on_every_edge() {
        // Aries 0° opens a sign, a nakshatra and a pada at once.
        let found = Boundaries::of(at(0.0));
        assert!(found.sign_deg.abs() < 1e-12);
        assert!(found.nakshatra_deg.abs() < 1e-12);
        assert!(found.pada_deg.abs() < 1e-12);
        assert!(found.nearest_deg().abs() < 1e-12);
        assert!(found.is_near(1e-9));
    }

    #[test]
    fn the_middle_of_a_sign_is_as_far_as_it_gets() {
        let found = Boundaries::of(at(15.0));
        assert!((found.sign_deg - 15.0).abs() < 1e-9);
        // A nakshatra is 13°20' and a pada a quarter of that, so the
        // distances shrink with the division.
        assert!(found.nakshatra_deg < found.sign_deg);
        assert!(found.pada_deg <= found.nakshatra_deg);
        assert!(!found.is_near(0.01));
    }

    #[test]
    fn a_distance_is_to_the_nearer_edge_either_side() {
        let before = Boundaries::of(at(29.99));
        let after = Boundaries::of(at(30.01));
        assert!((before.sign_deg - 0.01).abs() < 1e-9, "{before:?}");
        assert!((after.sign_deg - 0.01).abs() < 1e-9, "{after:?}");
        // And the tolerance is the caller's to choose.
        assert!(before.is_near(0.02));
        assert!(!before.is_near(0.005));
    }

    #[test]
    fn every_longitude_of_the_circle_has_a_distance() {
        for tenth in 0..3600 {
            let found = Boundaries::of(at(f64::from(tenth) / 10.0));
            assert!(found.sign_deg >= 0.0 && found.sign_deg <= 15.0);
            assert!(found.nakshatra_deg >= 0.0 && found.nakshatra_deg <= 360.0 / 54.0);
            assert!(found.pada_deg >= 0.0 && found.pada_deg <= 360.0 / 216.0);
            assert!(found.nearest_deg() <= found.sign_deg);
        }
    }
}
