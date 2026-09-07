//! What every derived point is: a catalogue key and a longitude, with
//! the sign it falls in and how near it stands to the edge of one.
//!
//! A point is not a body, but everything above reads it as one — it has
//! a longitude, it falls in a sign, a house holds it, a drishti reaches
//! it — so it carries the same things a placed graha does, including
//! the distance to the boundary that decides its sign
//! (`03-design/derived-points.md` §9).

use serde::Serialize;
use teistro_core::angle::Nas;
use teistro_core::boundary::Boundaries;
use teistro_core::catalogue::{Point, Rashi};
use teistro_core::error::Error;
use teistro_core::quantity::Degrees;

/// Where a derived point stands.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Derived {
    /// Which point, by the catalogue's own key.
    pub point: Point,
    /// Its longitude in the chart's zodiac, degrees.
    pub longitude_deg: f64,
    /// The sign it falls in.
    pub sign: Rashi,
    /// How near it stands to a sign, nakshatra or pada edge, which is
    /// what a different ayanamsha would move it across.
    pub boundaries: Boundaries,
}

impl Derived {
    /// A point at a longitude, which is wrapped into the circle.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` for a longitude that is not a finite number.
    pub fn at(point: Point, longitude_deg: f64) -> Result<Derived, Error> {
        if !longitude_deg.is_finite() {
            return Err(Error::invalid_arg(format!(
                "{} needs a finite longitude, not {longitude_deg}",
                point.key()
            ))
            .with_field("points.longitude_deg"));
        }
        let angle = Nas::from_degrees(Degrees::try_new(longitude_deg.rem_euclid(360.0))?);
        Ok(Derived {
            point,
            longitude_deg: angle.to_degrees(),
            sign: angle.sign(),
            boundaries: Boundaries::of(angle),
        })
    }

    /// How far into its sign it stands, degrees.
    #[must_use]
    pub fn in_sign_deg(&self) -> f64 {
        self.longitude_deg - f64::from(self.sign as u16) * 30.0
    }

    /// Whether its sign would survive an ayanamsha moved by a caller's
    /// own tolerance, degrees.
    ///
    /// The tolerance is the caller's because it is a property of its
    /// provider's accuracy and not of the tradition — the same argument
    /// `state::boundary` and `aspect` make.
    #[must_use]
    pub fn is_firm(&self, tolerance_deg: f64) -> bool {
        self.boundaries.sign_deg > tolerance_deg
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::float_cmp,
        reason = "tests fail by panicking, and two longitudes computed the same way are equal or the code is wrong"
    )]

    use super::Derived;
    use teistro_core::catalogue::{Point, Rashi};

    #[test]
    fn a_point_carries_its_sign_and_its_edge() {
        let found = Derived::at(Point::Dhuma, 45.5).unwrap();
        assert_eq!(found.point, Point::Dhuma);
        assert_eq!(found.sign, Rashi::Taurus);
        assert!((found.in_sign_deg() - 15.5).abs() < 1e-9);
        assert!((found.boundaries.sign_deg - 14.5).abs() < 1e-9);
        assert!(found.is_firm(1.0) && !found.is_firm(15.0));
    }

    #[test]
    fn a_longitude_is_wrapped_into_the_circle() {
        let over = Derived::at(Point::Yogi, 370.0).unwrap();
        let under = Derived::at(Point::Yogi, -350.0).unwrap();
        assert!((over.longitude_deg - 10.0).abs() < 1e-9);
        assert_eq!(over.longitude_deg, under.longitude_deg);
        assert_eq!(over.sign, Rashi::Aries);
        // The very start of the zodiac is on every edge at once.
        let start = Derived::at(Point::Yogi, 360.0).unwrap();
        assert!(start.longitude_deg.abs() < 1e-9);
        assert!(!start.is_firm(0.0), "it is on the edge, not past it");
    }

    #[test]
    fn a_longitude_that_is_not_a_number_is_refused_by_field() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let error = Derived::at(Point::Gulika, bad).expect_err("not a longitude");
            assert_eq!(error.field(), Some("points.longitude_deg"));
            assert!(error.message.contains("GULIKA"), "{error}");
        }
    }
}
