//! Planetary phenomena (`docs/03-design/astro-planetary-phenomena.md`):
//! what a body looks like from the Earth. The elongation from the Sun,
//! the phase angle and the illuminated fraction, the apparent disc and
//! the horizontal parallax, and the visual magnitude under the models the
//! Astronomical Almanac uses (Mallama and Hilton 2018 for the planets,
//! Allen 1976 with Samaha's crescent for the Moon, the inverse-square
//! Sun, the IAU 1986 polynomial for Pluto). Read either from the frame
//! completion over a provider, which fetches the geometry, or from a
//! geometry the caller supplies, so the arithmetic is one function and the
//! ephemeris another.
//!
//! ```
//! use teistro_astro::phenomena::phenomena;
//! use teistro_astro::{Completion, DeltaTModel};
//! use teistro_core::quantity::{JulianDay, Tt};
//! use teistro_core::settings::OverridePolicy;
//! use teistro_port_ephemeris::{Body, TestProvider};
//!
//! let provider = TestProvider::new();
//! let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
//! let mars = phenomena(&completion, Body::Mars, JulianDay::<Tt>::J2000).expect("phenomena");
//! let phase = mars.phase.expect("a planet has a phase");
//! assert!((0.0..=180.0).contains(&mars.elongation_deg));
//! assert!((0.0..=1.0).contains(&phase.illuminated_fraction));
//! assert!(mars.magnitude.is_some());
//! ```

use teistro_core::angle::normalise_deg;
use teistro_core::error::{Error, Status};
use teistro_core::quantity::{JulianDay, Tt};
use teistro_port_ephemeris::{
    Body, Cell, Centre, DistanceUnit, EphemerisProvider, Frame, PositionRequest, ProviderError,
    TimeScale,
};

use crate::completion::{Completion, CompletionError};
use crate::iau::vector::{Vector3, pdp, pm, s2c};
use crate::iau::{AULT, DAYSEC, DEG2RAD, DJ00, DJC, RAD2DEG};
use crate::rise_set::{AU_KM, Disc, EARTH_EQUATORIAL_RADIUS_KM};

/// A position in ecliptic coordinates of date with its distance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EclipticPosition {
    /// Longitude, degrees.
    pub lon_deg: f64,
    /// Latitude, degrees.
    pub lat_deg: f64,
    /// Distance, astronomical units; zero for a point without one.
    pub dist_au: f64,
}

impl EclipticPosition {
    /// A position from its parts.
    #[must_use]
    pub const fn new(lon_deg: f64, lat_deg: f64, dist_au: f64) -> EclipticPosition {
        EclipticPosition {
            lon_deg,
            lat_deg,
            dist_au,
        }
    }

    /// The direction as a unit vector.
    fn direction(&self) -> Vector3 {
        s2c(self.lon_deg * DEG2RAD, self.lat_deg * DEG2RAD)
    }

    /// The position as a vector, au.
    fn vector(&self) -> Vector3 {
        self.direction().map(|component| component * self.dist_au)
    }

    fn check(&self, name: &str) -> Result<(), Error> {
        if !(self.lon_deg.is_finite() && self.lat_deg.is_finite() && self.dist_au.is_finite()) {
            return Err(Error::invalid_arg(format!(
                "the {name} position must be finite ({}, {}, {})",
                self.lon_deg, self.lat_deg, self.dist_au
            ))
            .with_field(name));
        }
        if self.dist_au < 0.0 {
            return Err(Error::invalid_arg(format!(
                "the {name} distance cannot be negative ({} au)",
                self.dist_au
            ))
            .with_field(name));
        }
        Ok(())
    }
}

/// The geometry a body's phenomena are read from: the body and the Sun as
/// the observer sees them (apparent, ecliptic of date), and the body from
/// the Sun at the instant the light now arriving left it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Geometry {
    /// The body from the observer.
    pub body: EclipticPosition,
    /// The Sun from the observer.
    pub sun: EclipticPosition,
    /// The body from the Sun at the retarded instant; `None` to take the
    /// difference of the two apparent vectors instead, which a provider
    /// without heliocentric positions leaves as the only way.
    pub body_from_sun: Option<EclipticPosition>,
}

/// Where the Sun-to-body leg of the geometry came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeliocentricLeg {
    /// The provider's heliocentric position at the retarded instant.
    Provider,
    /// The difference of the apparent body and Sun vectors: within a few
    /// arcseconds of phase angle, the Sun's own light time being ignored.
    FromGeocentric,
}

/// The phase of a body: the angle at the body between the Sun and the
/// observer, and the fraction of its disc that is lit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Phase {
    /// The phase angle, degrees, 0 at full and 180 at new.
    pub angle_deg: f64,
    /// The illuminated fraction, `(1 + cos i) / 2`.
    pub illuminated_fraction: f64,
}

/// What a body looks like from the observer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Phenomena {
    /// The body.
    pub body: Body,
    /// The angular distance from the Sun, degrees in `[0, 180]`; the Sun's
    /// own is zero.
    pub elongation_deg: f64,
    /// The phase; `None` for the Sun, which has none, and for the lunar
    /// points, which have no disc.
    pub phase: Option<Phase>,
    /// The apparent semidiameter and the horizontal parallax; a point's are
    /// zero.
    pub disc: Disc,
    /// The visual magnitude; `None` for a point, or where the model's fit
    /// does not reach (Venus within a degree of the Sun's far side).
    pub magnitude: Option<f64>,
    /// How the Sun-to-body leg was obtained.
    pub heliocentric_leg: HeliocentricLeg,
}

impl Phenomena {
    /// The apparent diameter, degrees.
    #[must_use]
    pub fn apparent_diameter_deg(&self) -> f64 {
        self.disc.semidiameter_deg * 2.0
    }

    /// The phenomena from a geometry at a TT instant (the instant only
    /// enters Saturn's ring tilt and Neptune's slow drift).
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming the position for a non-finite value or a
    /// negative distance.
    pub fn from_geometry(
        body: Body,
        geometry: &Geometry,
        tt: JulianDay<Tt>,
    ) -> Result<Phenomena, Error> {
        geometry.body.check("body")?;
        geometry.sun.check("sun")?;
        if let Some(leg) = &geometry.body_from_sun {
            leg.check("body_from_sun")?;
        }
        let elongation_deg = if body == Body::Sun {
            0.0
        } else {
            angle_deg(&geometry.body.direction(), &geometry.sun.direction())
        };
        let disc = Disc::of(body, geometry.body.dist_au);
        let has_phase = body != Body::Sun && body.has_distance() && geometry.body.dist_au > 0.0;
        let (from_sun, heliocentric_leg) = geometry.body_from_sun.map_or_else(
            || {
                let [bx, by, bz] = geometry.body.vector();
                let [sx, sy, sz] = geometry.sun.vector();
                ([bx - sx, by - sy, bz - sz], HeliocentricLeg::FromGeocentric)
            },
            |leg| (leg.vector(), HeliocentricLeg::Provider),
        );
        let phase = has_phase.then(|| {
            let angle_deg = angle_deg(&geometry.body.vector(), &from_sun);
            Phase {
                angle_deg,
                illuminated_fraction: f64::midpoint(1.0, (angle_deg * DEG2RAD).cos()),
            }
        });
        let magnitude = magnitude(body, geometry, phase, &from_sun, tt);
        Ok(Phenomena {
            body,
            elongation_deg,
            phase,
            disc,
            magnitude,
            heliocentric_leg,
        })
    }
}

/// The angle between two vectors, degrees, clamped against rounding.
fn angle_deg(a: &Vector3, b: &Vector3) -> f64 {
    let (ma, mb) = (pm(a), pm(b));
    if ma == 0.0 || mb == 0.0 {
        return 0.0;
    }
    (pdp(a, b) / (ma * mb)).clamp(-1.0, 1.0).acos() * RAD2DEG
}

/// The Sun's mean visual magnitude at one astronomical unit (the
/// Astronomical Almanac's −26.86 with its 1 392 000 km disc; the ratio of
/// discs scales it, so the radius cancels).
const SUN_MAGNITUDE_AT_1_AU: f64 = -26.86;

/// Where Allen's lunar formula gives way to Samaha's cubic for the thin
/// crescent: the phase angle where the two agree.
const LUNAR_STITCH_DEG: f64 = 147.138_546_5;

/// The visual magnitude of a body from its phase angle, its distances from
/// the Sun and the observer and, for Saturn, the tilt of its rings.
fn magnitude(
    body: Body,
    geometry: &Geometry,
    phase: Option<Phase>,
    from_sun: &Vector3,
    tt: JulianDay<Tt>,
) -> Option<f64> {
    let delta = geometry.body.dist_au;
    if body == Body::Sun {
        if delta <= 0.0 {
            return None;
        }
        // The Sun varies with distance alone: the disc's ratio to the disc at
        // one astronomical unit, squared.
        let disc = Disc::of(Body::Sun, delta).semidiameter_deg;
        let at_unit = Disc::of(Body::Sun, 1.0).semidiameter_deg;
        let fac = (disc / at_unit).powi(2);
        return Some(SUN_MAGNITUDE_AT_1_AU - 2.5 * fac.log10());
    }
    let phase = phase?;
    let r = pm(from_sun);
    if r <= 0.0 || delta <= 0.0 {
        return None;
    }
    let a = phase.angle_deg;
    let a2 = a * a;
    // The 5 log(rΔ) distance term every model shares, au.
    let distance_term = 5.0 * (r * delta).log10();
    let value = match body {
        Body::Moon => {
            // Allen (1976) with the Earth-Moon distance in Earth radii, and
            // Samaha's cubic for the crescent beyond the stitch.
            let distances = 5.0 * (delta * r * AU_KM / EARTH_EQUATORIAL_RADIUS_KM).log10();
            if a <= LUNAR_STITCH_DEG {
                -21.62 + 0.026 * a.abs() + 4e-9 * a.powi(4) + distances
            } else {
                -4.5444 - 2.5 * (180.0 - a).powi(3).log10() + distances
            }
        }
        // Mallama and Hilton (2018), sixth order in the phase angle.
        Body::Mercury => {
            -0.613 + a * 6.3280e-2 - a2 * 1.6336e-3 + a2 * a * 3.3644e-5 - a2 * a2 * 3.4265e-7
                + a2 * a2 * a * 1.6893e-9
                - a2 * a2 * a2 * 3.0334e-12
                + distance_term
        }
        Body::Venus => {
            if a > 179.0 {
                // Beyond the published fit's range.
                return None;
            }
            let base = if a <= 163.7 {
                -4.384 - a * 1.044e-3 + a2 * 3.687e-4 - a2 * a * 2.814e-6 + a2 * a2 * 8.938e-9
            } else {
                236.058_28 - a * 2.819_14 + a2 * 8.390_34e-3
            };
            base + distance_term
        }
        Body::Mars => {
            // The terms for the terrain in view are omitted: they change the
            // brightness by tenths of a magnitude within hours.
            let base = if a <= 50.0 {
                -1.601 + a * 0.022_67 - a2 * 0.000_130_2
            } else {
                -0.367 - a * 0.025_73 + a2 * 0.000_344_5
            };
            base + distance_term
        }
        Body::Jupiter => -9.395 - a * 3.7e-4 + a2 * 6.16e-4 + distance_term,
        Body::Saturn => {
            // The rings' tilt to the Earth and to the Sun, averaged: most of
            // Saturn's variation, nearly a magnitude between edge-on and open.
            // The ring pole is Meeus's (chapter 45), referred to the ecliptic
            // of date, at the retarded instant.
            let retarded = tt.get() - delta * AULT / DAYSEC;
            let t = (retarded - DJ00) / DJC;
            let inclination = (28.075_216 - 0.012_998 * t + 0.000_004 * t * t) * DEG2RAD;
            let node = (169.508_470 + 1.394_681 * t + 0.000_412 * t * t) * DEG2RAD;
            let tilt = |lon_deg: f64, lat_deg: f64| {
                let (lon, lat) = (lon_deg * DEG2RAD, lat_deg * DEG2RAD);
                (inclination.sin() * lat.cos() * (lon - node).sin() - inclination.cos() * lat.sin())
                    .clamp(-1.0, 1.0)
                    .asin()
            };
            let (helio_lon, helio_lat) = ecliptic_of(from_sun);
            let from_earth = tilt(geometry.body.lon_deg, geometry.body.lat_deg);
            let from_sun_tilt = tilt(helio_lon, helio_lat);
            let sin_b = f64::midpoint(from_earth, from_sun_tilt).sin().abs();
            -8.914 - 1.825 * sin_b + 0.026 * a - 0.378 * sin_b * (-2.25 * a).exp() + distance_term
        }
        Body::Uranus => {
            // The sub-Earth latitude term is folded into its mean, −0.05.
            -7.110 + a * 6.587e-3 + a2 * 1.045e-4 + distance_term - 0.05
        }
        Body::Neptune => {
            // A step of the calendar: the observed brightness drifted from
            // −6.89 in 1980 to −7.00 in 2000 and settled; the slope keeps the
            // curve continuous.
            const YEAR_1980: f64 = 2_444_239.5;
            const YEAR_2000: f64 = 2_451_544.5;
            let base = if tt.get() < YEAR_1980 {
                -6.89
            } else if tt.get() <= YEAR_2000 {
                -6.89 - 0.0055 * (tt.get() - YEAR_1980) / 365.25
            } else {
                -7.00
            };
            base + distance_term
        }
        // The IAU 1986 polynomial with its phase terms at zero.
        Body::Pluto => -1.00 + distance_term,
        _ => return None,
    };
    Some(value)
}

/// The ecliptic longitude and latitude, degrees, of a vector.
fn ecliptic_of(v: &Vector3) -> (f64, f64) {
    let (lon, lat) = crate::iau::vector::c2s(v);
    (normalise_deg(lon * RAD2DEG), lat * RAD2DEG)
}

/// A body's phenomena at a TT instant over the frame completion: the body
/// and the Sun apparent and geocentric from the provider, and the body's
/// heliocentric position at the retarded instant when the provider gives
/// one (`HeliocentricLeg::Provider`), else the difference of the two
/// (`FromGeocentric`).
///
/// # Errors
///
/// `UNSUPPORTED` naming the provider when its distances are not in
/// astronomical units (a classical astronomy's mean distances give no
/// magnitude), `INVALID_ARG` for a body the provider does not carry, or the
/// provider's own error.
pub fn phenomena<P: EphemerisProvider + ?Sized>(
    completion: &Completion<'_, P>,
    body: Body,
    tt: JulianDay<Tt>,
) -> Result<Phenomena, Error> {
    let capabilities = completion.capabilities();
    if capabilities.distance_unit != DistanceUnit::AstronomicalUnits {
        return Err(Error::new(
            Status::Unsupported,
            format!(
                "the {} provider gives distances in {:?}; the phenomena need astronomical units",
                capabilities.identity.name, capabilities.distance_unit
            ),
        )
        .with_field("provider"));
    }
    let jds = [tt.get()];
    let bodies = [body, Body::Sun];
    let request = PositionRequest::new(&jds, TimeScale::Tt, &bodies, Frame::CANONICAL);
    let done = completion.positions(&request)?;
    let read = |index: usize, name: &str| -> Result<EclipticPosition, Error> {
        let cell = done
            .columns
            .at(0, index)
            .filter(Cell::is_ok)
            .ok_or_else(|| Error::new(Status::Provider, format!("no position for the {name}")))?;
        Ok(EclipticPosition::new(cell.lon, cell.lat, cell.dist))
    };
    let body_position = read(0, body.key())?;
    let sun = read(1, "Sun")?;
    let body_from_sun = if body != Body::Sun && body.has_distance() && body_position.dist_au > 0.0 {
        heliocentric_leg(completion, body, tt, body_position.dist_au)?
    } else {
        None
    };
    Phenomena::from_geometry(
        body,
        &Geometry {
            body: body_position,
            sun,
            body_from_sun,
        },
        tt,
    )
}

/// The body from the Sun at the retarded instant, from a provider that
/// answers heliocentric requests; `None` when it does not.
fn heliocentric_leg<P: EphemerisProvider + ?Sized>(
    completion: &Completion<'_, P>,
    body: Body,
    tt: JulianDay<Tt>,
    distance_au: f64,
) -> Result<Option<EclipticPosition>, Error> {
    let retarded = [tt.get() - distance_au * AULT / DAYSEC];
    let bodies = [body];
    let request = PositionRequest::new(
        &retarded,
        TimeScale::Tt,
        &bodies,
        Frame::CANONICAL.with_centre(Centre::Heliocentric),
    );
    match completion.positions(&request) {
        Ok(done) => Ok(done
            .columns
            .at(0, 0)
            .filter(|cell| cell.is_ok() && cell.dist > 0.0)
            .map(|cell| EclipticPosition::new(cell.lon, cell.lat, cell.dist))),
        Err(
            CompletionError::Unsupported { .. }
            | CompletionError::Provider {
                error: ProviderError::Unsupported { .. },
            },
        ) => Ok(None),
        Err(error) => Err(error.into()),
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::float_cmp,
        reason = "tests fail by panicking and compare exact zeros"
    )]

    use teistro_core::settings::OverridePolicy;
    use teistro_port_ephemeris::TestProvider;

    use super::*;
    use crate::delta_t::DeltaTModel;

    const J2000: JulianDay<Tt> = JulianDay::J2000;

    fn geometry(body_lon: f64, body_dist: f64, sun_lon: f64) -> Geometry {
        Geometry {
            body: EclipticPosition::new(body_lon, 0.0, body_dist),
            sun: EclipticPosition::new(sun_lon, 0.0, 1.0),
            body_from_sun: None,
        }
    }

    /// The Moon opposite the Sun is full and lit, at quadrature half lit.
    #[test]
    fn the_moons_phases_follow_its_elongation() {
        let full =
            Phenomena::from_geometry(Body::Moon, &geometry(180.0, 0.002_57, 0.0), J2000).unwrap();
        assert!((full.elongation_deg - 180.0).abs() < 1e-9);
        let phase = full.phase.unwrap();
        assert!(phase.angle_deg < 0.01, "{}", phase.angle_deg);
        assert!(phase.illuminated_fraction > 0.999_99);
        // Allen's full Moon at the mean distance: −12.7 or so.
        assert!(
            (full.magnitude.unwrap() + 12.7).abs() < 0.2,
            "{:?}",
            full.magnitude
        );
        assert!((full.disc.semidiameter_deg * 60.0 - 15.5).abs() < 0.3);
        assert!((full.disc.parallax_deg * 60.0 - 57.0).abs() < 0.5);
        assert_eq!(full.heliocentric_leg, HeliocentricLeg::FromGeocentric);

        let quarter =
            Phenomena::from_geometry(Body::Moon, &geometry(90.0, 0.002_57, 0.0), J2000).unwrap();
        let phase = quarter.phase.unwrap();
        assert!((phase.angle_deg - 90.0).abs() < 0.2, "{}", phase.angle_deg);
        assert!((phase.illuminated_fraction - 0.5).abs() < 0.002);

        // A crescent a day old reads Samaha's branch and is faint.
        let crescent =
            Phenomena::from_geometry(Body::Moon, &geometry(12.0, 0.002_57, 0.0), J2000).unwrap();
        assert!(crescent.phase.unwrap().angle_deg > LUNAR_STITCH_DEG);
        assert!(
            crescent.magnitude.unwrap() > -8.0,
            "{:?}",
            crescent.magnitude
        );
    }

    /// Venus at greatest elongation, half lit at −4.4; within a degree of the
    /// Sun's far side its fit gives out.
    #[test]
    fn venus_is_bright_at_greatest_elongation_and_unfitted_at_inferior_conjunction() {
        // Venus 0.72 au from the Sun, seen 46.3° from it at 0.69 au: the
        // phase angle is 90° and a little.
        let venus =
            Phenomena::from_geometry(Body::Venus, &geometry(46.3, 0.694, 0.0), J2000).unwrap();
        let phase = venus.phase.unwrap();
        assert!((phase.angle_deg - 90.0).abs() < 2.0, "{}", phase.angle_deg);
        assert!(
            (venus.magnitude.unwrap() + 4.4).abs() < 0.3,
            "{:?}",
            venus.magnitude
        );
        assert!((venus.disc.semidiameter_deg * 3600.0 - 12.0).abs() < 1.0);
        // Nearly between the Earth and the Sun.
        let inferior =
            Phenomena::from_geometry(Body::Venus, &geometry(0.3, 0.277, 0.0), J2000).unwrap();
        assert!(inferior.phase.unwrap().angle_deg > 179.0);
        assert!(inferior.magnitude.is_none());
        // Just outside that degree, the crescent branch answers.
        let thin = Phenomena::from_geometry(Body::Venus, &geometry(3.0, 0.28, 0.0), J2000).unwrap();
        let angle = thin.phase.unwrap().angle_deg;
        assert!((163.7..179.0).contains(&angle), "{angle}");
        assert!(thin.magnitude.is_some());
    }

    /// The outer planets at opposition: Jupiter −2.7, Saturn brighter with
    /// its rings open than edge-on, Neptune −7.0 of absolute magnitude after
    /// 2000 and −6.89 before 1980, Pluto the IAU polynomial.
    #[test]
    fn the_outer_planets_at_opposition() {
        let jupiter =
            Phenomena::from_geometry(Body::Jupiter, &geometry(180.0, 4.2, 0.0), J2000).unwrap();
        assert!(
            (jupiter.magnitude.unwrap() + 2.7).abs() < 0.2,
            "{:?}",
            jupiter.magnitude
        );
        assert!(jupiter.phase.unwrap().angle_deg < 0.01);
        // Saturn at 8.5 au: the ring tilt depends on where along its orbit.
        let open =
            Phenomena::from_geometry(Body::Saturn, &geometry(80.0, 8.5, 260.0), J2000).unwrap();
        let edge =
            Phenomena::from_geometry(Body::Saturn, &geometry(170.0, 8.5, 350.0), J2000).unwrap();
        assert!(
            open.magnitude.unwrap() < edge.magnitude.unwrap(),
            "{:?} {:?}",
            open.magnitude,
            edge.magnitude
        );
        assert!(
            (open.magnitude.unwrap() + 0.4).abs() < 0.6,
            "{:?}",
            open.magnitude
        );
        let neptune_now =
            Phenomena::from_geometry(Body::Neptune, &geometry(180.0, 29.0, 0.0), J2000).unwrap();
        let neptune_1970 = Phenomena::from_geometry(
            Body::Neptune,
            &geometry(180.0, 29.0, 0.0),
            JulianDay::literal(2_440_587.5),
        )
        .unwrap();
        assert!(
            (neptune_1970.magnitude.unwrap() - neptune_now.magnitude.unwrap() - 0.11).abs() < 1e-9
        );
        assert!(
            (neptune_now.magnitude.unwrap() - 7.8).abs() < 0.2,
            "{:?}",
            neptune_now.magnitude
        );
        let pluto =
            Phenomena::from_geometry(Body::Pluto, &geometry(180.0, 30.0, 0.0), J2000).unwrap();
        assert!(
            (pluto.magnitude.unwrap() - 13.9).abs() < 0.2,
            "{:?}",
            pluto.magnitude
        );
    }

    /// The Sun has no phase and no elongation, its magnitude follows the
    /// inverse square; a node has no disc, phase or magnitude but an
    /// elongation.
    #[test]
    fn the_sun_and_the_points() {
        let sun = Phenomena::from_geometry(Body::Sun, &geometry(10.0, 1.0, 10.0), J2000).unwrap();
        assert_eq!(sun.elongation_deg, 0.0);
        assert!(sun.phase.is_none());
        assert!((sun.magnitude.unwrap() - SUN_MAGNITUDE_AT_1_AU).abs() < 1e-9);
        let near =
            Phenomena::from_geometry(Body::Sun, &geometry(10.0, 0.983, 10.0), J2000).unwrap();
        assert!(
            (near.magnitude.unwrap() - (SUN_MAGNITUDE_AT_1_AU + 5.0 * 0.983f64.log10())).abs()
                < 1e-6
        );
        let node = Geometry {
            body: EclipticPosition::new(100.0, 0.0, 0.0),
            sun: EclipticPosition::new(10.0, 0.0, 1.0),
            body_from_sun: None,
        };
        let node = Phenomena::from_geometry(Body::MeanNode, &node, J2000).unwrap();
        assert!((node.elongation_deg - 90.0).abs() < 1e-9);
        assert!(node.phase.is_none() && node.magnitude.is_none());
        assert_eq!(node.disc.semidiameter_deg, 0.0);
        // A given heliocentric leg is used as given.
        // The body beyond the Earth on the far side from the Sun: from the
        // Sun it lies the same way as from the Earth, so the phase angle is
        // zero.
        let given = Geometry {
            body_from_sun: Some(EclipticPosition::new(180.0, 0.0, 1.5)),
            ..geometry(180.0, 0.5, 0.0)
        };
        let mars = Phenomena::from_geometry(Body::Mars, &given, J2000).unwrap();
        assert_eq!(mars.heliocentric_leg, HeliocentricLeg::Provider);
        assert!(mars.phase.unwrap().angle_deg < 1e-9);
        let bad = Geometry {
            body: EclipticPosition::new(f64::NAN, 0.0, 1.0),
            ..geometry(0.0, 1.0, 0.0)
        };
        assert_eq!(
            Phenomena::from_geometry(Body::Mars, &bad, J2000)
                .unwrap_err()
                .field(),
            Some("body")
        );
    }

    /// Over the test provider, which answers no heliocentric request, the
    /// leg comes from the geocentric vectors and every quantity is sound.
    #[test]
    fn the_completion_supplies_the_geometry() {
        let provider = TestProvider::new();
        let completion = Completion::new(
            &provider,
            OverridePolicy::SdkOnly,
            DeltaTModel::TableThenModel,
        );
        for body in [
            Body::Moon,
            Body::Mercury,
            Body::Venus,
            Body::Mars,
            Body::Jupiter,
            Body::Saturn,
        ] {
            let seen = phenomena(&completion, body, J2000).unwrap();
            assert_eq!(seen.body, body);
            assert!(
                (0.0..=180.0).contains(&seen.elongation_deg),
                "{body:?} {seen:?}"
            );
            let phase = seen.phase.unwrap();
            assert!(
                (0.0..=180.0).contains(&phase.angle_deg),
                "{body:?} {seen:?}"
            );
            assert!((0.0..=1.0).contains(&phase.illuminated_fraction));
            assert!(seen.disc.semidiameter_deg > 0.0);
            assert!(seen.magnitude.is_some(), "{body:?}");
            assert_eq!(seen.heliocentric_leg, HeliocentricLeg::FromGeocentric);
        }
        let sun = phenomena(&completion, Body::Sun, J2000).unwrap();
        assert!(sun.phase.is_none() && sun.elongation_deg == 0.0);
        let node = phenomena(&completion, Body::MeanNode, J2000).unwrap();
        assert!(node.phase.is_none() && node.magnitude.is_none());
    }
}
