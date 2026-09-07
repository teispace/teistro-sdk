//! The five upagrahas the Sun casts, which are a chain and not five
//! offsets.
//!
//! Dhuma is 133°20′ ahead of the Sun; Vyatipata is Dhuma reflected about
//! the start of the zodiac; Parivesha is Vyatipata opposed; Indrachapa
//! is Parivesha reflected; Upaketu is 16°40′ past Indrachapa.
//!
//! **Every one is exact** against the conformance corpus — not near,
//! exact, to the last bit of a double, on all 71 fixtures that record
//! them (`03-design/points-measured.md` §2). The project's research page
//! marks the whole family "verify"; that pass is the verification.
//!
//! The chain matters *as a chain*. Two of its five steps are
//! reflections, and a reflection does not compose into an offset:
//! **Dhuma, Indrachapa and Upaketu advance with the Sun while Vyatipata
//! and Parivesha retreat from it.** Written as five constants added to
//! the Sun, two of them would carry the wrong sign and the code would
//! still look right — which is why [`advances`] states the direction
//! and a test measures it rather than trusting the reading.
//!
//! ```
//! use teistro_points::solar::chain;
//! use teistro_core::catalogue::Point;
//!
//! let found = chain(0.0).expect("a finite longitude");
//! assert_eq!(found[0].point, Point::Dhuma);
//! assert!((found[0].longitude_deg - (133.0 + 20.0 / 60.0)).abs() < 1e-9);
//! // Vyatipata is Dhuma reflected, so it retreats as the Sun advances.
//! let later = chain(1.0).expect("a finite longitude");
//! assert!(later[1].longitude_deg < found[1].longitude_deg);
//! assert_eq!(teistro_points::solar::advances(Point::Vyatipata), Some(false));
//! ```

use teistro_core::catalogue::Point;
use teistro_core::error::Error;

use crate::derived::Derived;

/// How far ahead of the Sun Dhuma stands, degrees.
pub const DHUMA_FROM_SUN_DEG: f64 = 133.0 + 20.0 / 60.0;

/// How far past Indrachapa Upaketu stands, degrees.
pub const UPAKETU_FROM_INDRACHAPA_DEG: f64 = 16.0 + 40.0 / 60.0;

/// The five, in the order the chain builds them.
pub const CHAIN: [Point; 5] = [
    Point::Dhuma,
    Point::Vyatipata,
    Point::Parivesha,
    Point::Indrachapa,
    Point::Upaketu,
];

/// The five upagrahas the Sun casts, in the order the chain builds them.
///
/// # Errors
///
/// `INVALID_ARG` for a longitude that is not a finite number.
pub fn chain(sun_deg: f64) -> Result<[Derived; 5], Error> {
    let dhuma = sun_deg + DHUMA_FROM_SUN_DEG;
    let vyatipata = reflected(dhuma);
    let parivesha = vyatipata + 180.0;
    let indrachapa = reflected(parivesha);
    let upaketu = indrachapa + UPAKETU_FROM_INDRACHAPA_DEG;
    Ok([
        Derived::at(Point::Dhuma, dhuma)?,
        Derived::at(Point::Vyatipata, vyatipata)?,
        Derived::at(Point::Parivesha, parivesha)?,
        Derived::at(Point::Indrachapa, indrachapa)?,
        Derived::at(Point::Upaketu, upaketu)?,
    ])
}

/// A longitude reflected about the start of the zodiac, which is what
/// the tradition means by "subtract it from the circle".
fn reflected(degrees: f64) -> f64 {
    360.0 - degrees.rem_euclid(360.0)
}

/// Whether a member of the chain advances with the Sun or retreats from
/// it: two reflections make two of them run backwards.
///
/// `None` for a point that is not in the chain.
#[must_use]
pub fn advances(point: Point) -> Option<bool> {
    match point {
        // An odd number of reflections behind it, so it runs backwards.
        Point::Vyatipata | Point::Parivesha => Some(false),
        Point::Dhuma | Point::Indrachapa | Point::Upaketu => Some(true),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::float_cmp,
        reason = "tests fail by panicking, index a fixed-length chain and compare longitudes computed the same way"
    )]

    use super::{CHAIN, DHUMA_FROM_SUN_DEG, UPAKETU_FROM_INDRACHAPA_DEG, advances, chain};
    use teistro_core::catalogue::Point;

    #[test]
    fn the_chain_is_the_five_in_order() {
        let found = chain(0.0).unwrap();
        for (derived, point) in found.iter().zip(CHAIN) {
            assert_eq!(derived.point, point);
        }
        assert!((found[0].longitude_deg - DHUMA_FROM_SUN_DEG).abs() < 1e-9);
        assert!((found[1].longitude_deg - (360.0 - DHUMA_FROM_SUN_DEG)).abs() < 1e-9);
        assert!((found[2].longitude_deg - (180.0 - DHUMA_FROM_SUN_DEG)).abs() < 1e-9);
        assert!((found[3].longitude_deg - (180.0 + DHUMA_FROM_SUN_DEG)).abs() < 1e-9);
        assert!(
            (found[4].longitude_deg - (180.0 + DHUMA_FROM_SUN_DEG + UPAKETU_FROM_INDRACHAPA_DEG))
                .abs()
                < 1e-9
        );
    }

    #[test]
    fn two_of_them_retreat_as_the_sun_advances() {
        // Two reflections: Vyatipata is behind one, Parivesha behind one,
        // Indrachapa and Upaketu behind two, which cancel.
        let here = chain(10.0).unwrap();
        let there = chain(11.0).unwrap();
        for (first, second) in here.iter().zip(&there) {
            let moved = (second.longitude_deg - first.longitude_deg).rem_euclid(360.0);
            let forward = (moved - 1.0).abs() < 1e-9;
            let backward = (moved - 359.0).abs() < 1e-9;
            assert!(forward || backward, "{:?}: {moved}", first.point);
            assert_eq!(
                advances(first.point),
                Some(forward),
                "{:?} moved {moved}",
                first.point
            );
        }
        assert_eq!(advances(Point::SreeLagna), None, "not in the chain");
    }

    #[test]
    fn every_degree_of_the_sun_gives_five_longitudes() {
        for tenth in 0..3600 {
            let sun = f64::from(tenth) / 10.0;
            let found = chain(sun).unwrap();
            for derived in &found {
                assert!(
                    (0.0..360.0).contains(&derived.longitude_deg),
                    "{:?} at {sun}",
                    derived.point
                );
            }
            // Dhuma and Indrachapa are opposed, and so are Vyatipata and
            // Parivesha: the chain's two reflections make two pairs.
            let apart = |a: usize, b: usize| {
                (found[a].longitude_deg - found[b].longitude_deg)
                    .abs()
                    .rem_euclid(360.0)
            };
            assert!((apart(0, 3) - 180.0).abs() < 1e-9, "at {sun}");
            assert!((apart(1, 2) - 180.0).abs() < 1e-9, "at {sun}");
        }
    }

    #[test]
    fn a_sun_that_is_not_a_number_is_refused() {
        assert!(chain(f64::NAN).is_err());
        assert!(chain(f64::INFINITY).is_err());
        // And a longitude outside the circle is wrapped, not refused.
        let over = chain(370.0).unwrap();
        let under = chain(10.0).unwrap();
        for (a, b) in over.iter().zip(&under) {
            assert!((a.longitude_deg - b.longitude_deg).abs() < 1e-9);
        }
    }
}
