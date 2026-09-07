//! The orb engine: an angular separation measured against a stated
//! tolerance, and whether the pair is closing on the angle or leaving
//! it.
//!
//! This knows nothing of signs, houses or grahas. It is the shared
//! geometry that `tajika` needs for ithasala and ishrafa, that a future
//! `western` needs for the Ptolemaic aspects, and that the classical
//! conjunction does **not** need — a conjunction in this tradition is
//! co-presence in a sign, with no orb at all
//! (`03-design/aspect-and-drishti.md` §6).
//!
//! Keeping them apart is deliberate. An engine that tried to be both
//! would need an orb parameter half its callers must pass as "not
//! applicable".
//!
//! ```
//! use teistro_aspect::orb::{Angle, Moving, hits};
//!
//! // Two bodies eight degrees short of opposition, the gap still opening
//! // towards it because the leading body is the faster.
//! let first = Moving::new(10.0, 0.1);
//! let second = Moving::new(182.0, 1.0);
//! let found = hits(first, second, &Angle::PTOLEMAIC, 10.0).expect("a finite orb");
//! assert_eq!(found.len(), 1);
//! assert_eq!(found[0].angle.key, "OPPOSITION");
//! assert!(found[0].applying, "the faster body is closing on it");
//! assert!((found[0].from_exact_deg - 8.0).abs() < 1e-9);
//! ```

use serde::Serialize;
use teistro_core::error::Error;

/// The widest orb the engine accepts, degrees. Past a right angle an
/// "orb" no longer separates one angle from its neighbours.
pub const WIDEST_ORB_DEG: f64 = 90.0;

/// A body as this engine sees it: where it is and how fast it is going.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Moving {
    /// Longitude, degrees.
    pub longitude_deg: f64,
    /// Motion in longitude, degrees a day; negative when retrograde.
    pub speed_deg_per_day: f64,
}

impl Moving {
    /// A body at a longitude, moving at a rate.
    #[must_use]
    pub const fn new(longitude_deg: f64, speed_deg_per_day: f64) -> Moving {
        Moving {
            longitude_deg,
            speed_deg_per_day,
        }
    }

    /// A body whose motion is not known or does not matter, which no
    /// [`Hit`] from it will call applying.
    #[must_use]
    pub const fn still(longitude_deg: f64) -> Moving {
        Moving::new(longitude_deg, 0.0)
    }
}

/// An angle two bodies may stand at.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Angle {
    /// Its name, which a value carries so a caller need not compare
    /// floating-point degrees to find out which angle it is.
    pub key: &'static str,
    /// The separation it names, degrees, 0 to 180.
    pub degrees: f64,
}

impl Angle {
    /// Bodies at the same longitude.
    pub const CONJUNCTION: Angle = Angle {
        key: "CONJUNCTION",
        degrees: 0.0,
    };
    /// A sixth of the circle.
    pub const SEXTILE: Angle = Angle {
        key: "SEXTILE",
        degrees: 60.0,
    };
    /// A quarter of the circle.
    pub const SQUARE: Angle = Angle {
        key: "SQUARE",
        degrees: 90.0,
    };
    /// A third of the circle.
    pub const TRINE: Angle = Angle {
        key: "TRINE",
        degrees: 120.0,
    };
    /// Half the circle.
    pub const OPPOSITION: Angle = Angle {
        key: "OPPOSITION",
        degrees: 180.0,
    };

    /// The five Ptolemaic angles, which the Tajika aspects are read on
    /// too.
    pub const PTOLEMAIC: [Angle; 5] = [
        Angle::CONJUNCTION,
        Angle::SEXTILE,
        Angle::SQUARE,
        Angle::TRINE,
        Angle::OPPOSITION,
    ];
}

/// One angle two bodies stand within an orb of.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Hit {
    /// Which angle.
    pub angle: Angle,
    /// How far apart the bodies are, degrees, 0 to 180.
    pub apart_deg: f64,
    /// How far from exact the angle is, degrees; zero when it is exact.
    pub from_exact_deg: f64,
    /// Whether the separation is closing on the angle rather than
    /// leaving it. Always false when neither body moves.
    pub applying: bool,
}

/// The angle between two longitudes, degrees, the shorter way round.
#[must_use]
pub fn separation(first: f64, second: f64) -> f64 {
    let apart = (first - second).abs().rem_euclid(360.0);
    apart.min(360.0 - apart)
}

/// Every angle the two bodies stand within `orb_deg` of.
///
/// The result is in the order the angles were given, so a caller that
/// passes [`Angle::PTOLEMAIC`] gets them from conjunction to opposition.
///
/// # Errors
///
/// `INVALID_ARG` for an orb that is negative, not finite or wider than
/// [`WIDEST_ORB_DEG`], and for a longitude or speed that is not finite.
pub fn hits(
    first: Moving,
    second: Moving,
    angles: &[Angle],
    orb_deg: f64,
) -> Result<Vec<Hit>, Error> {
    check_orb(orb_deg)?;
    check_body(first, "aspect.first")?;
    check_body(second, "aspect.second")?;
    let apart = separation(first.longitude_deg, second.longitude_deg);
    // A day is short enough that no pair crosses an angle within it, and
    // long enough that the change is well above the last bit of a
    // double.
    let later = separation(
        first.longitude_deg + first.speed_deg_per_day,
        second.longitude_deg + second.speed_deg_per_day,
    );
    Ok(angles
        .iter()
        .filter_map(|angle| {
            let from_exact = (apart - angle.degrees).abs();
            (from_exact <= orb_deg).then(|| Hit {
                angle: *angle,
                apart_deg: apart,
                from_exact_deg: from_exact,
                applying: (later - angle.degrees).abs() < from_exact,
            })
        })
        .collect())
}

/// The one angle of a set the bodies stand nearest, within the orb.
///
/// # Errors
///
/// As [`hits`].
pub fn nearest(
    first: Moving,
    second: Moving,
    angles: &[Angle],
    orb_deg: f64,
) -> Result<Option<Hit>, Error> {
    let mut found = hits(first, second, angles, orb_deg)?;
    found.sort_by(|a, b| {
        a.from_exact_deg
            .partial_cmp(&b.from_exact_deg)
            .unwrap_or(core::cmp::Ordering::Equal)
    });
    Ok(found.into_iter().next())
}

fn check_orb(orb_deg: f64) -> Result<(), Error> {
    if orb_deg.is_finite() && (0.0..=WIDEST_ORB_DEG).contains(&orb_deg) {
        return Ok(());
    }
    Err(
        Error::invalid_arg(format!(
            "an orb of {orb_deg}° is outside 0 to {WIDEST_ORB_DEG}; past a right angle an orb no longer separates one angle from its neighbours"
        ))
        .with_field("aspect.orb_deg"),
    )
}

fn check_body(body: Moving, field: &str) -> Result<(), Error> {
    if body.longitude_deg.is_finite() && body.speed_deg_per_day.is_finite() {
        return Ok(());
    }
    Err(Error::invalid_arg(String::from(
        "a longitude and a speed must both be finite numbers",
    ))
    .with_field(field.to_string()))
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index their own results"
    )]

    use super::{Angle, Moving, WIDEST_ORB_DEG, hits, nearest, separation};

    #[test]
    fn a_separation_is_the_shorter_way_round() {
        assert!((separation(10.0, 350.0) - 20.0).abs() < 1e-9);
        assert!((separation(350.0, 10.0) - 20.0).abs() < 1e-9);
        assert!((separation(0.0, 180.0) - 180.0).abs() < 1e-9);
        assert!(separation(5.0, 5.0).abs() < 1e-9);
        assert!((separation(-10.0, 10.0) - 20.0).abs() < 1e-9);
    }

    #[test]
    fn an_angle_within_the_orb_is_a_hit_and_one_outside_is_not() {
        let found = hits(
            Moving::still(0.0),
            Moving::still(115.0),
            &Angle::PTOLEMAIC,
            6.0,
        )
        .unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].angle.key, "TRINE");
        assert!((found[0].from_exact_deg - 5.0).abs() < 1e-9);
        assert!((found[0].apart_deg - 115.0).abs() < 1e-9);
        assert!(!found[0].applying, "neither body moves");

        let none = hits(
            Moving::still(0.0),
            Moving::still(105.0),
            &Angle::PTOLEMAIC,
            6.0,
        )
        .unwrap();
        assert!(none.is_empty(), "{none:?}");
    }

    #[test]
    fn a_wide_orb_can_catch_two_angles_and_the_nearest_picks_one() {
        // Seventy-five degrees is fifteen from a sextile and fifteen from
        // a square.
        let found = hits(
            Moving::still(0.0),
            Moving::still(75.0),
            &Angle::PTOLEMAIC,
            16.0,
        )
        .unwrap();
        assert_eq!(found.len(), 2, "{found:?}");
        assert_eq!(found[0].angle.key, "SEXTILE", "in the order given");
        assert_eq!(found[1].angle.key, "SQUARE");
        let one = nearest(
            Moving::still(0.0),
            Moving::still(74.0),
            &Angle::PTOLEMAIC,
            16.0,
        )
        .unwrap()
        .expect("within the orb");
        assert_eq!(one.angle.key, "SEXTILE", "fourteen against sixteen");
    }

    #[test]
    fn applying_is_the_separation_closing_on_the_angle() {
        // 172° apart and opening: the leading body is the faster, so the
        // separation grows towards the opposition.
        let closing = hits(
            Moving::new(10.0, 0.1),
            Moving::new(182.0, 1.0),
            &[Angle::OPPOSITION],
            10.0,
        )
        .unwrap();
        assert!((closing[0].apart_deg - 172.0).abs() < 1e-9, "{closing:?}");
        assert!(closing[0].applying, "{closing:?}");

        // Past the opposition — 185° round one way, so 175° the shorter
        // — and still opening, which now takes it away from the angle.
        let leaving = hits(
            Moving::new(10.0, 0.1),
            Moving::new(195.0, 1.0),
            &[Angle::OPPOSITION],
            10.0,
        )
        .unwrap();
        assert!((leaving[0].apart_deg - 175.0).abs() < 1e-9, "{leaving:?}");
        assert!(!leaving[0].applying, "{leaving:?}");

        // The same the other way about, for a conjunction: a faster body
        // behind a slower one is closing on it.
        let catching = hits(
            Moving::new(10.0, 1.0),
            Moving::new(13.0, 0.1),
            &[Angle::CONJUNCTION],
            5.0,
        )
        .unwrap();
        assert!(catching[0].applying, "{catching:?}");
        let past = hits(
            Moving::new(13.0, 1.0),
            Moving::new(10.0, 0.1),
            &[Angle::CONJUNCTION],
            5.0,
        )
        .unwrap();
        assert!(!past[0].applying, "{past:?}");
    }

    #[test]
    fn a_conjunction_by_orb_is_a_separation_and_not_a_sign() {
        let found = hits(
            Moving::still(359.0),
            Moving::still(2.0),
            &[Angle::CONJUNCTION],
            5.0,
        )
        .unwrap();
        assert_eq!(found.len(), 1, "over the end of the circle");
        assert!((found[0].apart_deg - 3.0).abs() < 1e-9);
    }

    #[test]
    fn an_orb_or_a_body_that_makes_no_sense_is_refused_by_field() {
        for orb in [-1.0, f64::NAN, f64::INFINITY, WIDEST_ORB_DEG + 0.1] {
            let error = hits(Moving::still(0.0), Moving::still(1.0), &[], orb)
                .expect_err("{orb} is not an orb");
            assert_eq!(error.field(), Some("aspect.orb_deg"), "{orb}");
        }
        let error = hits(Moving::still(f64::NAN), Moving::still(1.0), &[], 1.0)
            .expect_err("not a longitude");
        assert_eq!(error.field(), Some("aspect.first"));
        let error = hits(Moving::still(0.0), Moving::new(1.0, f64::NAN), &[], 1.0)
            .expect_err("not a speed");
        assert_eq!(error.field(), Some("aspect.second"));
        // Nought is an orb, and the widest is too.
        assert!(hits(Moving::still(0.0), Moving::still(0.0), &[], 0.0).is_ok());
        assert!(hits(Moving::still(0.0), Moving::still(0.0), &[], WIDEST_ORB_DEG).is_ok());
    }
}
