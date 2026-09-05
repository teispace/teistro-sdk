//! House cusps for every catalogued system
//! (`docs/03-design/astro-house-systems.md`): the twelve cusps and the
//! auxiliary points (ascendant, midheaven, vertex, equatorial ascendant,
//! the two co-ascendants, the polar ascendant) from the right ascension of
//! the meridian, the geographic latitude and the true obliquity, with the
//! polar policy deciding what a system that is undefined inside the polar
//! circle does. Cusps are tropical longitudes of date; a sidereal chart
//! subtracts its ayanamsha.
//!
//! Every quadrant system is one construction: the ecliptic point where a
//! great circle of a given pole height, meeting the equator at a given
//! right ascension, crosses the ecliptic ([`circle_point`]). The horizon
//! (pole height the latitude, meeting the equator 90° east of the
//! meridian) gives the ascendant; an hour circle (pole height zero) gives
//! the midheaven and the meridian houses; Regiomontanus, Campanus,
//! Topocentric, Koch and Placidus differ only in which circles they pick.
//!
//! ```
//! use teistro_astro::houses::{Input, houses};
//! use teistro_core::catalogue::HouseSystem;
//! use teistro_core::settings::PolarPolicy;
//!
//! // Kathmandu, ARMC 3h: the midheaven is 45° along the ecliptic in the east.
//! let input = Input { armc_deg: 45.0, latitude_deg: 27.7172, obliquity_deg: 23.44, sun_declination_deg: None, sidereal_offset_deg: 0.0 };
//! let placidus = houses(HouseSystem::Placidus, &input, PolarPolicy::Error).expect("defined");
//! assert!((placidus.angles.midheaven_deg - 47.459).abs() < 0.01);
//! assert_eq!(placidus.cusps[9], placidus.angles.midheaven_deg);
//! assert_eq!(placidus.cusps[0], placidus.angles.ascendant_deg);
//! ```

use serde::Serialize;
use teistro_core::angle::{difference_deg, normalise_deg};
use teistro_core::catalogue::{Degeneracy, HouseSystem};
use teistro_core::error::{Error, Status};
use teistro_core::quantity::{JulianDay, Place, Tt, Ut1};
use teistro_core::settings::PolarPolicy;

use crate::iau::{DEG2RAD, RAD2DEG};
use crate::sky;

/// The engines' snapping threshold: an angle within this of a cardinal value
/// is that value, so that latitude 0 and ARMC 0 give an ascendant of 90.
const VERY_SMALL: f64 = 1e-10;

/// The most iterations a Placidus cusp is given before the system is
/// declared undefined at the place.
const PLACIDUS_ITERATIONS: u32 = 100;

/// The Placidus iteration's convergence, degrees: a hundredth of an arcsecond.
const PLACIDUS_TOLERANCE_DEG: f64 = 1.0 / 360_000.0;

/// The Sun's declination cannot exceed the obliquity; Sunshine refuses more.
const SUN_DECLINATION_BOUND_DEG: f64 = 24.0;

/// What the computation needs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Input {
    /// The right ascension of the meridian (local apparent sidereal time),
    /// degrees.
    pub armc_deg: f64,
    /// The geographic latitude, degrees, north positive.
    pub latitude_deg: f64,
    /// The true obliquity of the ecliptic, degrees.
    pub obliquity_deg: f64,
    /// The Sun's declination, degrees, for the Sunshine system alone.
    pub sun_declination_deg: Option<f64>,
    /// The ayanamsha of a sidereal chart, degrees, or zero for a tropical
    /// one: the sign-based systems (whole sign, equal from 0° Aries) take
    /// their signs in the zodiac in use, and their cusps come back as the
    /// tropical longitudes of those sidereal sign boundaries so that every
    /// cusp is tropical and the chart subtracts the same ayanamsha from all.
    pub sidereal_offset_deg: f64,
}

/// The auxiliary points every system shares, tropical degrees of date.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Angles {
    /// The ascendant: where the ecliptic rises.
    pub ascendant_deg: f64,
    /// The midheaven: the ecliptic on the meridian.
    pub midheaven_deg: f64,
    /// The right ascension of the meridian, as given.
    pub armc_deg: f64,
    /// The vertex: where the ecliptic crosses the prime vertical in the west.
    pub vertex_deg: f64,
    /// The equatorial ascendant (east point): the ecliptic point with right
    /// ascension ARMC + 90°.
    pub equatorial_ascendant_deg: f64,
    /// Koch's co-ascendant.
    pub co_ascendant_koch_deg: f64,
    /// Munkasey's co-ascendant.
    pub co_ascendant_munkasey_deg: f64,
    /// Munkasey's polar ascendant.
    pub polar_ascendant_deg: f64,
}

/// What happened to a system undefined at the place.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Outcome {
    /// The system asked for, at the place asked for.
    Defined,
    /// Another system's cusps stand in for the one asked for, which is
    /// undefined inside the polar circle.
    Substituted {
        /// The system asked for.
        asked: HouseSystem,
    },
    /// The system asked for, at the nearest latitude where it is defined.
    Clamped {
        /// The latitude asked for, degrees.
        asked_latitude_deg: f64,
    },
}

/// The cusps and points of one system at one place and instant.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Houses {
    /// The system the cusps belong to (the substitute, when one stood in).
    pub system: HouseSystem,
    /// The twelve cusps, house 1 first, tropical degrees of date.
    pub cusps: [f64; 12],
    /// The auxiliary points.
    pub angles: Angles,
    /// Whether the system asked for was computed, substituted or clamped.
    pub outcome: Outcome,
}

/// Whether a latitude lies inside the polar circle for an obliquity, where
/// the midheaven can sink below the horizon.
#[must_use]
pub fn is_polar(latitude_deg: f64, obliquity_deg: f64) -> bool {
    latitude_deg.abs() >= 90.0 - obliquity_deg
}

/// The ecliptic longitude, degrees, where the great circle of pole height
/// `pole_deg` whose ascending intersection with the equator is at right
/// ascension `ra_deg` crosses the ecliptic of obliquity `eps`: the horizon
/// gives the ascendant, an hour circle the midheaven.
#[must_use]
pub fn circle_point(ra_deg: f64, pole_deg: f64, eps: &Obliquity) -> f64 {
    let (sin_ra, cos_ra) = (ra_deg * DEG2RAD).sin_cos();
    let denominator = eps.cos * cos_ra - eps.sin * (pole_deg * DEG2RAD).tan();
    let value = normalise_deg(sin_ra.atan2(denominator) * RAD2DEG);
    snap_cardinal(value)
}

/// Snaps an angle within the engines' threshold of 0°, 90°, 180° or 270°.
fn snap_cardinal(value: f64) -> f64 {
    for cardinal in [90.0, 180.0, 270.0] {
        if (value - cardinal).abs() < VERY_SMALL {
            return cardinal;
        }
    }
    if (value - 360.0).abs() < VERY_SMALL || value.abs() < VERY_SMALL {
        return 0.0;
    }
    value
}

/// The obliquity with its trigonometry, computed once.
#[derive(Clone, Copy, Debug)]
pub struct Obliquity {
    deg: f64,
    sin: f64,
    cos: f64,
    tan: f64,
}

impl Obliquity {
    fn new(deg: f64) -> Obliquity {
        let (sin, cos) = (deg * DEG2RAD).sin_cos();
        Obliquity {
            deg,
            sin,
            cos,
            tan: sin / cos,
        }
    }
}

fn sind(x: f64) -> f64 {
    (x * DEG2RAD).sin()
}
fn cosd(x: f64) -> f64 {
    (x * DEG2RAD).cos()
}
fn tand(x: f64) -> f64 {
    (x * DEG2RAD).tan()
}
fn asind(x: f64) -> f64 {
    x.clamp(-1.0, 1.0).asin() * RAD2DEG
}
fn acosd(x: f64) -> f64 {
    x.clamp(-1.0, 1.0).acos() * RAD2DEG
}
fn atand(x: f64) -> f64 {
    x.atan() * RAD2DEG
}

/// The ecliptic longitude of a point on the equator at right ascension
/// `ra_deg` (the Morinus projection, along circles through the ecliptic
/// poles).
fn equator_point_longitude(ra_deg: f64, eps: &Obliquity) -> f64 {
    let (sin_ra, cos_ra) = (ra_deg * DEG2RAD).sin_cos();
    normalise_deg((sin_ra * eps.cos).atan2(cos_ra) * RAD2DEG)
}

/// The right ascension of a point on the ecliptic at longitude `lon_deg`.
fn ecliptic_point_right_ascension(lon_deg: f64, eps: &Obliquity) -> f64 {
    let (sin_lon, cos_lon) = (lon_deg * DEG2RAD).sin_cos();
    normalise_deg((sin_lon * eps.cos).atan2(cos_lon) * RAD2DEG)
}

/// A spherical point rotated about the x axis by `angle_deg`: longitude and
/// latitude in, longitude and latitude out, degrees.
fn rotate_x(lon_deg: f64, lat_deg: f64, angle_deg: f64) -> (f64, f64) {
    let (sin_lon, cos_lon) = (lon_deg * DEG2RAD).sin_cos();
    let (sin_lat, cos_lat) = (lat_deg * DEG2RAD).sin_cos();
    let (sin_a, cos_a) = (angle_deg * DEG2RAD).sin_cos();
    let x = cos_lat * cos_lon;
    let y = cos_lat * sin_lon * cos_a + sin_lat * sin_a;
    let z = -cos_lat * sin_lon * sin_a + sin_lat * cos_a;
    let rxy = (x * x + y * y).sqrt();
    let lon = normalise_deg(y.atan2(x) * RAD2DEG);
    let lat = if rxy == 0.0 {
        if z >= 0.0 { 90.0 } else { -90.0 }
    } else {
        (z / rxy).atan() * RAD2DEG
    };
    (lon, lat)
}

/// The working frame of one computation.
struct Frame {
    armc: f64,
    latitude: f64,
    eps: Obliquity,
    tan_latitude: f64,
    ascendant: f64,
    midheaven: f64,
    sidereal_offset: f64,
}

impl Frame {
    fn new(input: &Input) -> Frame {
        // The poles are singular; step just inside them.
        let mut latitude = input.latitude_deg;
        if (latitude.abs() - 90.0).abs() < VERY_SMALL {
            latitude = if latitude < 0.0 {
                -90.0 + VERY_SMALL
            } else {
                90.0 - VERY_SMALL
            };
        }
        let eps = Obliquity::new(input.obliquity_deg);
        let armc = normalise_deg(input.armc_deg);
        Frame {
            armc,
            latitude,
            eps,
            tan_latitude: tand(latitude),
            ascendant: circle_point(armc + 90.0, latitude, &eps),
            midheaven: circle_point(armc, 0.0, &eps),
            sidereal_offset: input.sidereal_offset_deg,
        }
    }

    /// Inside the polar circle the midheaven can sink below the horizon, and
    /// then the rising point the formula gives is the descendant: the signed
    /// distance from the midheaven to the ascendant is negative.
    fn ascendant_is_descendant(&self) -> bool {
        difference_deg(self.ascendant, self.midheaven) < 0.0
    }

    /// The ascendant turned to the east when the midheaven is below the horizon.
    fn eastern_ascendant(&self) -> f64 {
        if self.ascendant_is_descendant() {
            normalise_deg(self.ascendant + 180.0)
        } else {
            self.ascendant
        }
    }

    fn point(&self, ra_deg: f64, pole_deg: f64) -> f64 {
        circle_point(ra_deg, pole_deg, &self.eps)
    }

    fn inside_polar_circle(&self) -> bool {
        is_polar(self.latitude, self.eps.deg)
    }
}

/// The point opposite a longitude.
fn opposite(deg: f64) -> f64 {
    normalise_deg(deg + 180.0)
}

/// Fills cusps 4 to 9 as the opposites of 10, 11, 12, 1, 2, 3.
fn oppose(cusps: &mut [f64; 12]) {
    cusps[3] = opposite(cusps[9]);
    cusps[4] = opposite(cusps[10]);
    cusps[5] = opposite(cusps[11]);
    cusps[6] = opposite(cusps[0]);
    cusps[7] = opposite(cusps[1]);
    cusps[8] = opposite(cusps[2]);
}

/// Twelve equal houses from a first cusp.
fn equal_from(first: f64) -> [f64; 12] {
    let mut cusps = [0.0; 12];
    let mut value = first;
    for cusp in &mut cusps {
        *cusp = normalise_deg(value);
        value += 30.0;
    }
    cusps
}

/// Twelve values at right ascensions ARMC + 30 n, n = 1 to 12, mapped by
/// `project`, arranged with house 1 first (ARMC + 30 is house 11).
fn every_thirty_degrees(armc: f64, project: impl Fn(f64) -> f64) -> [f64; 12] {
    let mut cusps = [0.0; 12];
    let mut ra = armc;
    for cusp in &mut cusps {
        ra += 30.0;
        *cusp = project(ra);
    }
    cusps.rotate_left(2);
    cusps
}

/// Porphyry's cusps: the quadrants between the eastern ascendant and the
/// midheaven trisected.
fn porphyry(frame: &Frame) -> [f64; 12] {
    let ascendant = frame.eastern_ascendant();
    let mc = frame.midheaven;
    let quadrant = normalise_deg(ascendant - mc);
    let mut cusps = [0.0; 12];
    cusps[0] = ascendant;
    cusps[9] = mc;
    cusps[1] = normalise_deg(ascendant + (180.0 - quadrant) / 3.0);
    cusps[2] = normalise_deg(ascendant + (180.0 - quadrant) / 3.0 * 2.0);
    cusps[10] = normalise_deg(mc + quadrant / 3.0);
    cusps[11] = normalise_deg(mc + quadrant / 3.0 * 2.0);
    oppose(&mut cusps);
    cusps
}

/// Whole-sign houses from the eastern ascendant's sign, in the zodiac in
/// use: the sidereal ascendant's sign boundary, returned as a tropical
/// longitude.
fn whole_sign(frame: &Frame) -> [f64; 12] {
    let sidereal_ascendant = normalise_deg(frame.eastern_ascendant() - frame.sidereal_offset);
    equal_from(sidereal_ascendant - sidereal_ascendant % 30.0 + frame.sidereal_offset)
}

/// Turns the quadrant cusps (1, 2, 3, 10, 11, 12) by 180° when the
/// midheaven is below the horizon inside the polar circle, as the circle
/// systems do.
fn turn_quadrant_cusps(frame: &Frame, cusps: &mut [f64; 12], all: bool) {
    if frame.inside_polar_circle() && frame.ascendant_is_descendant() {
        for (i, cusp) in cusps.iter_mut().enumerate() {
            if all || !(3..9).contains(&i) {
                *cusp = normalise_deg(*cusp + 180.0);
            }
        }
    }
}

/// A quadrant system's four intermediate cusps from the circles it picks:
/// (right ascension, pole height) for cusps 11, 12, 2 and 3.
fn circle_system(frame: &Frame, picks: [(f64, f64); 4], turn_all: bool) -> [f64; 12] {
    let mut cusps = [0.0; 12];
    cusps[0] = frame.ascendant;
    cusps[9] = frame.midheaven;
    cusps[10] = frame.point(picks[0].0, picks[0].1);
    cusps[11] = frame.point(picks[1].0, picks[1].1);
    cusps[1] = frame.point(picks[2].0, picks[2].1);
    cusps[2] = frame.point(picks[3].0, picks[3].1);
    turn_quadrant_cusps(frame, &mut cusps, turn_all);
    oppose(&mut cusps);
    cusps
}

fn regiomontanus(frame: &Frame) -> [f64; 12] {
    let th = frame.armc;
    let f1 = atand(frame.tan_latitude * 0.5);
    let f2 = atand(frame.tan_latitude * cosd(30.0));
    circle_system(
        frame,
        [
            (th + 30.0, f1),
            (th + 60.0, f2),
            (th + 120.0, f2),
            (th + 150.0, f1),
        ],
        false,
    )
}

/// Campanus's circles from a latitude and meridian: the prime vertical cut
/// into 30° arcs, great circles through the north and south points.
fn campanus_picks(latitude: f64, th: f64) -> [(f64, f64); 4] {
    let f1 = asind(sind(latitude) / 2.0);
    let f2 = asind(3f64.sqrt() / 2.0 * sind(latitude));
    let cos_latitude = cosd(latitude);
    let (xh1, xh2) = if cos_latitude.abs() == 0.0 {
        let pole = if latitude > 0.0 { 90.0 } else { 270.0 };
        (pole, pole)
    } else {
        (
            atand(3f64.sqrt() / cos_latitude),
            atand(1.0 / 3f64.sqrt() / cos_latitude),
        )
    };
    [
        (th + 90.0 - xh1, f1),
        (th + 90.0 - xh2, f2),
        (th + 90.0 + xh2, f2),
        (th + 90.0 + xh1, f1),
    ]
}

fn campanus(frame: &Frame) -> [f64; 12] {
    circle_system(frame, campanus_picks(frame.latitude, frame.armc), false)
}

fn topocentric(frame: &Frame) -> [f64; 12] {
    let th = frame.armc;
    let f1 = atand(frame.tan_latitude / 3.0);
    let f2 = atand(frame.tan_latitude * 2.0 / 3.0);
    circle_system(
        frame,
        [
            (th + 30.0, f1),
            (th + 60.0, f2),
            (th + 120.0, f2),
            (th + 150.0, f1),
        ],
        true,
    )
}

/// Alcabitius: the eastern ascendant's semi-diurnal and semi-nocturnal arcs,
/// measured on the equator, trisected, the points carried to the ecliptic
/// along hour circles.
fn alcabitius(frame: &Frame) -> [f64; 12] {
    let ascendant = frame.eastern_ascendant();
    let th = frame.armc;
    let declination = asind(sind(ascendant) * frame.eps.sin);
    let diurnal = acosd(-frame.tan_latitude * tand(declination));
    let nocturnal = 180.0 - diurnal;
    let mut cusps = [0.0; 12];
    cusps[0] = ascendant;
    cusps[9] = frame.midheaven;
    cusps[10] = frame.point(th + diurnal / 3.0, 0.0);
    cusps[11] = frame.point(th + 2.0 * diurnal / 3.0, 0.0);
    cusps[1] = frame.point(th + 180.0 - 2.0 * nocturnal / 3.0, 0.0);
    cusps[2] = frame.point(th + 180.0 - nocturnal / 3.0, 0.0);
    oppose(&mut cusps);
    cusps
}

/// Koch: the midheaven's semi-arcs trisected in time, the ascendants at
/// those times; undefined inside the polar circle.
fn koch(frame: &Frame) -> Option<[f64; 12]> {
    if frame.inside_polar_circle() {
        return None;
    }
    let th = frame.armc;
    let sina = (sind(frame.midheaven) * frame.eps.sin / cosd(frame.latitude)).clamp(-1.0, 1.0);
    let cosa = (1.0 - sina * sina).sqrt();
    let c = atand(frame.tan_latitude / cosa);
    let ad3 = asind(sind(c) * sina) / 3.0;
    let latitude = frame.latitude;
    let mut cusps = [0.0; 12];
    cusps[0] = frame.ascendant;
    cusps[9] = frame.midheaven;
    cusps[10] = frame.point(th + 30.0 - 2.0 * ad3, latitude);
    cusps[11] = frame.point(th + 60.0 - ad3, latitude);
    cusps[1] = frame.point(th + 120.0 + ad3, latitude);
    cusps[2] = frame.point(th + 150.0 + 2.0 * ad3, latitude);
    oppose(&mut cusps);
    Some(cusps)
}

/// One Placidus cusp: the ecliptic point that has covered the given fraction
/// (one third or two thirds, the divisor 3 or 1.5) of its semi-diurnal arc
/// on the circle meeting the equator at `rectasc`; the pole height depends
/// on the answer's declination, so it iterates. `None` when it does not
/// settle, which happens at the polar circle.
fn placidus_cusp(frame: &Frame, rectasc: f64, divisor: f64, initial_pole: f64) -> Option<f64> {
    let rectasc = normalise_deg(rectasc);
    let pole_for = |cusp: f64| -> Option<f64> {
        let tan_declination = tand(asind(frame.eps.sin * sind(cusp)));
        if tan_declination.abs() < VERY_SMALL {
            return Some(f64::NAN); // signals the equator case below
        }
        let arc = frame.tan_latitude * tan_declination;
        if arc.abs() > 1.0 {
            return None;
        }
        Some(atand(sind(asind(arc) / divisor) / tan_declination))
    };
    let mut pole = match pole_for(frame.point(rectasc, initial_pole))? {
        p if p.is_nan() => return Some(rectasc),
        p => p,
    };
    let mut cusp = frame.point(rectasc, pole);
    let mut previous = cusp;
    for iteration in 1..=PLACIDUS_ITERATIONS {
        pole = match pole_for(cusp)? {
            p if p.is_nan() => return Some(rectasc),
            p => p,
        };
        cusp = frame.point(rectasc, pole);
        if iteration > 1 && difference_deg(cusp, previous).abs() < PLACIDUS_TOLERANCE_DEG {
            return Some(cusp);
        }
        previous = cusp;
    }
    None
}

/// Placidus: the semi-diurnal arcs trisected in time; undefined inside the
/// polar circle, and close to it the iteration may not settle.
fn placidus(frame: &Frame) -> Option<[f64; 12]> {
    if frame.inside_polar_circle() {
        return None;
    }
    let th = frame.armc;
    let a = asind(frame.tan_latitude * frame.eps.tan);
    let fh1 = atand(sind(a / 3.0) / frame.eps.tan);
    let fh2 = atand(sind(a * 2.0 / 3.0) / frame.eps.tan);
    let mut cusps = [0.0; 12];
    cusps[0] = frame.ascendant;
    cusps[9] = frame.midheaven;
    cusps[10] = placidus_cusp(frame, th + 30.0, 3.0, fh1)?;
    cusps[11] = placidus_cusp(frame, th + 60.0, 1.5, fh2)?;
    cusps[1] = placidus_cusp(frame, th + 120.0, 1.5, fh2)?;
    cusps[2] = placidus_cusp(frame, th + 150.0, 3.0, fh1)?;
    oppose(&mut cusps);
    Some(cusps)
}

/// Meridian (axial rotation): the ecliptic points with right ascensions
/// ARMC + 30 n, along hour circles.
fn meridian(frame: &Frame) -> [f64; 12] {
    every_thirty_degrees(frame.armc, |ra| frame.point(ra, 0.0))
}

/// Morinus: the equator points ARMC + 30 n carried into ecliptic longitude.
fn morinus(frame: &Frame) -> [f64; 12] {
    every_thirty_degrees(frame.armc, |ra| equator_point_longitude(ra, &frame.eps))
}

/// Carter's poli-equatorial houses: hour circles at the ascendant's right
/// ascension plus 30 n.
fn carter(frame: &Frame) -> [f64; 12] {
    let ascendant = frame.eastern_ascendant();
    let ra = ecliptic_point_right_ascension(ascendant, &frame.eps);
    let mut cusps = [0.0; 12];
    cusps[0] = ascendant;
    cusps[1] = frame.point(ra + 30.0, 0.0);
    cusps[2] = frame.point(ra + 60.0, 0.0);
    cusps[9] = frame.point(ra + 270.0, 0.0);
    cusps[10] = frame.point(ra + 300.0, 0.0);
    cusps[11] = frame.point(ra + 330.0, 0.0);
    oppose(&mut cusps);
    cusps
}

/// Horizon (azimuthal) houses: Campanus's geometry applied to the horizon by
/// swapping the pole for the zenith.
fn horizon(frame: &Frame) -> [f64; 12] {
    let mut latitude = if frame.latitude > 0.0 {
        90.0 - frame.latitude
    } else {
        -90.0 - frame.latitude
    };
    if (latitude.abs() - 90.0).abs() < VERY_SMALL {
        latitude = if latitude < 0.0 {
            -90.0 + VERY_SMALL
        } else {
            90.0 - VERY_SMALL
        };
    }
    let th = normalise_deg(frame.armc + 180.0);
    let picks = campanus_picks(latitude, th);
    let mut cusps = [0.0; 12];
    cusps[10] = frame.point(picks[0].0, picks[0].1);
    cusps[11] = frame.point(picks[1].0, picks[1].1);
    cusps[0] = frame.point(th + 90.0, latitude);
    cusps[1] = frame.point(picks[2].0, picks[2].1);
    cusps[2] = frame.point(picks[3].0, picks[3].1);
    cusps[9] = frame.midheaven;
    if is_polar(latitude, frame.eps.deg) && frame.ascendant_is_descendant() {
        for (i, cusp) in cusps.iter_mut().enumerate() {
            if !(3..9).contains(&i) {
                *cusp = normalise_deg(*cusp + 180.0);
            }
        }
    }
    for (i, cusp) in cusps.iter_mut().enumerate() {
        if matches!(i, 0 | 1 | 2 | 10 | 11) {
            *cusp = opposite(*cusp);
        }
    }
    oppose(&mut cusps);
    cusps
}

/// Krusinski-Pisa-Goelzer: the great circle through the ascendant and the
/// zenith cut into twelve, carried to the ecliptic along hour circles.
fn krusinski(frame: &Frame) -> [f64; 12] {
    let ascendant = frame.eastern_ascendant();
    let eps = frame.eps.deg;
    let th = frame.armc;
    let latitude = frame.latitude;
    // The ascendant into the horizontal frame: ecliptic to equatorial, the
    // meridian to the origin, the pole to the zenith.
    let (lon, lat) = rotate_x(ascendant, 0.0, -eps);
    let (horizon_lon, _) = rotate_x(lon - (th - 90.0), lat, -(90.0 - latitude));
    let mut eastern = [0.0; 6];
    let mut along = 0.0;
    for cusp in &mut eastern {
        // The n-th point on the ascendant-zenith circle, back to horizontal,
        // rotated to the meridian, back to equatorial.
        let (lon, lat) = rotate_x(along, 0.0, 90.0);
        let (lon, _) = rotate_x(lon + horizon_lon, lat, 90.0 - latitude);
        let ra = normalise_deg(lon + (th - 90.0));
        *cusp = frame.point(ra, 0.0);
        along += 30.0;
    }
    let mut cusps = [0.0; 12];
    let (first, second) = cusps.split_at_mut(6);
    first.copy_from_slice(&eastern);
    for (cusp, east) in second.iter_mut().zip(eastern) {
        *cusp = opposite(east);
    }
    cusps
}

/// The twelve sectors of the Ascendant Parallel Circle system (Koppejan),
/// each cusp its own great circle; the houses are not opposed in pairs.
fn apc(frame: &Frame) -> [f64; 12] {
    let ph = frame.latitude * DEG2RAD;
    let e = frame.eps.deg * DEG2RAD;
    let az = frame.armc * DEG2RAD;
    let tan_ph = ph.tan();
    let (sin_az, cos_az) = az.sin_cos();
    let (sin_e, cos_e) = e.sin_cos();
    let (kv, dasc) = if frame.latitude.abs() > 90.0 - VERY_SMALL {
        (0.0, 0.0)
    } else {
        let tpte = tan_ph * e.tan();
        let kv = (tpte * cos_az / (1.0 + tpte * sin_az)).atan();
        let dasc = if frame.latitude.abs() < VERY_SMALL {
            ((90.0 - VERY_SMALL) * DEG2RAD).copysign(ph)
        } else {
            (kv.sin() / tan_ph).atan()
        };
        (kv, dasc)
    };
    let tdtp = dasc.tan() * tan_ph;
    let setp = sin_e * tan_ph;
    let mut cusps = [0.0; 12];
    let mut n = 0.0f64;
    for cusp in &mut cusps {
        n += 1.0;
        // Houses 1 to 7 lie below the horizon in this construction.
        let (k, span) = if n < 8.0 {
            (n - 1.0, core::f64::consts::FRAC_PI_2 - kv)
        } else {
            (n - 13.0, core::f64::consts::FRAC_PI_2 + kv)
        };
        let mut a = kv + az + core::f64::consts::FRAC_PI_2 + k * span / 3.0;
        a %= core::f64::consts::TAU;
        if a.abs() < 1e-13 {
            a = 0.0;
        }
        if a < 0.0 {
            a += core::f64::consts::TAU;
        }
        let value = (tdtp * sin_az + a.sin())
            .atan2(cos_e * (tdtp * cos_az + a.cos()) + setp * (az - a).sin());
        *cusp = normalise_deg(value * RAD2DEG);
    }
    // The construction's midheaven drifts near the pole; the true one stands.
    cusps[9] = frame.midheaven;
    cusps[3] = normalise_deg(frame.midheaven + 180.0);
    if frame.inside_polar_circle() && frame.ascendant_is_descendant() {
        for cusp in &mut cusps {
            *cusp = normalise_deg(*cusp + 180.0);
        }
    }
    cusps
}

/// Sripati: Porphyry's sectors with the cusps at the middle of each sector
/// (the Porphyry cusps are the bhava madhyas, the reported cusps the
/// sandhis).
fn sripati(frame: &Frame) -> [f64; 12] {
    let ascendant = frame.eastern_ascendant();
    let mc = frame.midheaven;
    let quadrant = normalise_deg(ascendant - mc);
    let s1 = (180.0 - quadrant) / 3.0;
    let s4 = quadrant / 3.0;
    let mut cusps = [0.0; 12];
    cusps[0] = normalise_deg(ascendant - s4 * 0.5);
    cusps[1] = normalise_deg(ascendant + s1 * 0.5);
    cusps[2] = normalise_deg(ascendant + s1 * 1.5);
    cusps[9] = normalise_deg(mc - s1 * 0.5);
    cusps[10] = normalise_deg(mc + s4 * 0.5);
    cusps[11] = normalise_deg(mc + s4 * 1.5);
    oppose(&mut cusps);
    cusps
}

/// Pullen's sinusoidal delta: house widths in a quadrant vary linearly
/// about 30°, the quadrant's excess spread as 1 : 3 : 3 : 1 … in Pullen's
/// delta form.
fn pullen_sinusoidal_delta(frame: &Frame) -> [f64; 12] {
    let ascendant = frame.eastern_ascendant();
    let mc = frame.midheaven;
    let quadrant = normalise_deg(ascendant - mc);
    let night = 180.0 - quadrant;
    let mut cusps = [0.0; 12];
    cusps[0] = ascendant;
    cusps[9] = mc;
    let d = (quadrant - 90.0) / 4.0;
    if quadrant <= 30.0 {
        cusps[10] = normalise_deg(mc + quadrant / 2.0);
        cusps[11] = cusps[10];
    } else {
        cusps[10] = normalise_deg(mc + 30.0 + d);
        cusps[11] = normalise_deg(mc + 60.0 + 3.0 * d);
    }
    let d = (night - 90.0) / 4.0;
    if night <= 30.0 {
        cusps[1] = normalise_deg(ascendant + night / 2.0);
        cusps[2] = cusps[1];
    } else {
        cusps[1] = normalise_deg(ascendant + 30.0 + d);
        cusps[2] = normalise_deg(ascendant + 60.0 + 3.0 * d);
    }
    oppose(&mut cusps);
    cusps
}

/// Pullen's sinusoidal ratio: house widths in a quadrant form a geometric
/// progression x, xr, xr³, xr⁴ with the ratio from Pullen's closed form of
/// the quartic.
fn pullen_sinusoidal_ratio(frame: &Frame) -> [f64; 12] {
    let ascendant = frame.eastern_ascendant();
    let mc = frame.midheaven;
    let quadrant = normalise_deg(ascendant - mc);
    let q = if quadrant > 90.0 {
        180.0 - quadrant
    } else {
        quadrant
    };
    let (x, xr, xr3, xr4) = if q < 1e-30 {
        (0.0, 0.0, 0.0, 180.0)
    } else {
        let third = 1.0 / 3.0;
        let two23 = 4f64.powf(third);
        let cc = (180.0 - q) / q;
        let ccr = (cc * cc - cc).powf(third);
        let cqx = (two23 * ccr + 1.0).sqrt();
        let r1 = 0.5 * cqx;
        let r2 = 0.5 * (-2.0 * (1.0 - 2.0 * cc) / cqx - two23 * ccr + 2.0).sqrt();
        let r = r1 + r2 - 0.5;
        let x = q / (2.0 * r + 1.0);
        let xr = r * x;
        let xr3 = xr * r * r;
        (x, xr, xr3, xr3 * r)
    };
    let mut cusps = [0.0; 12];
    cusps[0] = ascendant;
    cusps[9] = mc;
    if quadrant > 90.0 {
        cusps[10] = normalise_deg(mc + xr3);
        cusps[11] = normalise_deg(cusps[10] + xr4);
        cusps[1] = normalise_deg(ascendant + xr);
        cusps[2] = normalise_deg(cusps[1] + x);
    } else {
        cusps[10] = normalise_deg(mc + xr);
        cusps[11] = normalise_deg(cusps[10] + x);
        cusps[1] = normalise_deg(ascendant + xr3);
        cusps[2] = normalise_deg(cusps[1] + xr4);
    }
    oppose(&mut cusps);
    cusps
}

/// Sunshine (Makransky's system in Treindl's construction): the Sun's own
/// diurnal and nocturnal arcs trisected, so the houses move with the
/// season; the only system that needs the date. `None` where the Sun
/// neither rises nor sets or the construction collapses.
fn sunshine(frame: &Frame, sun_declination: f64) -> Option<[f64; 12]> {
    let latitude = frame.latitude;
    let ramc = frame.armc;
    let mut ascendant = frame.ascendant;
    let mut mc = frame.midheaven;
    if frame.ascendant_is_descendant() {
        ascendant = normalise_deg(ascendant + 180.0);
        mc = normalise_deg(mc + 180.0);
    }
    // The offsets along the Sun's path: thirds of its semi-arcs.
    let arg = tand(sun_declination) * tand(latitude);
    let ad = if arg >= 1.0 {
        90.0 - VERY_SMALL
    } else if arg <= -1.0 {
        -90.0 + VERY_SMALL
    } else {
        asind(arg)
    };
    let nocturnal = 90.0 - ad;
    let diurnal = 90.0 + ad;
    let offsets: [(usize, f64); 8] = [
        (2, -2.0 * nocturnal / 3.0),
        (3, -nocturnal / 3.0),
        (5, nocturnal / 3.0),
        (6, 2.0 * nocturnal / 3.0),
        (8, -2.0 * diurnal / 3.0),
        (9, -diurnal / 3.0),
        (11, diurnal / 3.0),
        (12, 2.0 * diurnal / 3.0),
    ];
    let (sin_lat, cos_lat) = (sind(latitude), cosd(latitude));
    let (cos_dec, tan_dec) = (cosd(sun_declination), tand(sun_declination));
    let mc_declination = atand(sind(ramc) * frame.eps.tan);
    let mc_under_horizon = (latitude - mc_declination).abs() > 90.0;
    let mut cusps = [0.0; 12];
    cusps[0] = ascendant;
    cusps[9] = mc;
    cusps[3] = normalise_deg(mc + 180.0);
    cusps[6] = normalise_deg(ascendant + 180.0);
    let mut sum = 0.0;
    for (house, offset) in offsets {
        let xhs = 2.0 * asind(cos_dec * sind(offset / 2.0));
        let alpha = acosd(tan_dec * tand(xhs / 2.0));
        let (alpha2, b) = if house > 7 {
            (180.0 - alpha, 90.0 - latitude + sun_declination)
        } else {
            (alpha, 90.0 - latitude - sun_declination)
        };
        let cos_c = cosd(xhs) * cosd(b) + sind(xhs) * sind(b) * cosd(alpha2);
        let c = acosd(cos_c);
        if c < 1e-6 {
            return None;
        }
        let sin_zd = sind(xhs) * sind(alpha2) / sind(c);
        let zd = asind(sin_zd);
        let rax = atand(cos_lat * tand(zd));
        let mut pole = asind(sin_zd * sin_lat);
        let a = if house <= 6 {
            pole = -pole;
            normalise_deg(rax + ramc + 180.0)
        } else {
            normalise_deg(ramc + rax)
        };
        let cusp = frame.point(a, pole);
        sum += cusp;
        if let Some(slot) = cusps.get_mut(house - 1) {
            *slot = cusp;
        }
    }
    // A cusp that is not a number is not a cusp: the construction collapsed.
    if sum.is_nan() {
        return None;
    }
    if mc_under_horizon {
        for (house, _) in offsets {
            if let Some(slot) = cusps.get_mut(house - 1) {
                *slot = opposite(*slot);
            }
        }
    }
    Some(cusps)
}

/// The systems that turn the midheaven with the ascendant when the
/// midheaven is below the horizon inside the polar circle, so that their
/// reported midheaven is the cusp they built on; the others keep the
/// meridian's own point.
fn turns_midheaven(system: HouseSystem) -> bool {
    matches!(
        system,
        HouseSystem::Campanus
            | HouseSystem::Regiomontanus
            | HouseSystem::Topocentric
            | HouseSystem::Apc
            | HouseSystem::Sunshine
    )
}

/// The auxiliary points of a frame, as the system reports them.
fn angles(frame: &Frame, system: HouseSystem) -> Angles {
    let th = frame.armc;
    let latitude = frame.latitude;
    let turned = frame.inside_polar_circle() && frame.ascendant_is_descendant();
    let midheaven = if turned && turns_midheaven(system) {
        opposite(frame.midheaven)
    } else {
        frame.midheaven
    };
    let complement = if latitude >= 0.0 {
        90.0 - latitude
    } else {
        -90.0 - latitude
    };
    let mut vertex = frame.point(th - 90.0, complement);
    // In the tropics the vertex wanders as the ascendant does inside the
    // polar circle; it is held in the western hemisphere.
    if latitude.abs() <= frame.eps.deg && difference_deg(vertex, frame.midheaven) > 0.0 {
        vertex = normalise_deg(vertex + 180.0);
    }
    Angles {
        ascendant_deg: frame.eastern_ascendant(),
        midheaven_deg: midheaven,
        armc_deg: th,
        vertex_deg: vertex,
        equatorial_ascendant_deg: frame.point(th + 90.0, 0.0),
        co_ascendant_koch_deg: normalise_deg(frame.point(th - 90.0, latitude) + 180.0),
        co_ascendant_munkasey_deg: frame.point(th + 90.0, complement),
        polar_ascendant_deg: frame.point(th - 90.0, latitude),
    }
}

/// The cusps of a system at a frame, or `None` where the system is undefined.
fn cusps_of(system: HouseSystem, frame: &Frame, sun_declination: Option<f64>) -> Option<[f64; 12]> {
    Some(match system {
        HouseSystem::WholeSign => whole_sign(frame),
        HouseSystem::Placidus => placidus(frame)?,
        HouseSystem::Koch => koch(frame)?,
        HouseSystem::Regiomontanus => regiomontanus(frame),
        HouseSystem::Campanus => campanus(frame),
        HouseSystem::Equal => equal_from(frame.eastern_ascendant()),
        HouseSystem::Meridian => meridian(frame),
        HouseSystem::Alcabitius => alcabitius(frame),
        HouseSystem::Porphyry => porphyry(frame),
        HouseSystem::Topocentric => topocentric(frame),
        HouseSystem::Morinus => morinus(frame),
        HouseSystem::Sripati => sripati(frame),
        HouseSystem::EqualMc => equal_from(frame.midheaven + 90.0),
        HouseSystem::EqualAries => equal_from(frame.sidereal_offset),
        HouseSystem::Vehlow => equal_from(frame.eastern_ascendant() - 15.0),
        HouseSystem::Carter => carter(frame),
        HouseSystem::Horizon => horizon(frame),
        HouseSystem::Sunshine => sunshine(frame, sun_declination?)?,
        HouseSystem::PullenSd => pullen_sinusoidal_delta(frame),
        HouseSystem::PullenSr => pullen_sinusoidal_ratio(frame),
        HouseSystem::Krusinski => krusinski(frame),
        HouseSystem::Apc => apc(frame),
        // A system the catalogue adds before this crate learns it.
        _ => return None,
    })
}

/// Whether this build has a construction for the system.
fn known(system: HouseSystem) -> bool {
    matches!(
        system,
        HouseSystem::WholeSign
            | HouseSystem::Placidus
            | HouseSystem::Koch
            | HouseSystem::Regiomontanus
            | HouseSystem::Campanus
            | HouseSystem::Equal
            | HouseSystem::Meridian
            | HouseSystem::Alcabitius
            | HouseSystem::Porphyry
            | HouseSystem::Topocentric
            | HouseSystem::Morinus
            | HouseSystem::Sripati
            | HouseSystem::EqualMc
            | HouseSystem::EqualAries
            | HouseSystem::Vehlow
            | HouseSystem::Carter
            | HouseSystem::Horizon
            | HouseSystem::Sunshine
            | HouseSystem::PullenSd
            | HouseSystem::PullenSr
            | HouseSystem::Krusinski
            | HouseSystem::Apc
    )
}

/// The cusps and points of a house system from the meridian, the latitude
/// and the obliquity, the polar policy deciding what a system undefined
/// inside the polar circle does.
///
/// # Errors
///
/// `UNSUPPORTED` for a system this build has no construction for; `INVALID_ARG`
/// for a non-finite input, a Sunshine request without the Sun's declination
/// or one outside ±24°; `UNSUPPORTED` naming the system and the latitude
/// when it is undefined there and the policy is `ERROR`.
pub fn houses(system: HouseSystem, input: &Input, policy: PolarPolicy) -> Result<Houses, Error> {
    if !known(system) {
        return Err(Error::new(
            Status::Unsupported,
            format!(
                "the {} house system has no construction in this build",
                system.key()
            ),
        )
        .with_field("houses.placement_system"));
    }
    for (name, value) in [
        ("armc", input.armc_deg),
        ("latitude", input.latitude_deg),
        ("obliquity", input.obliquity_deg),
        ("sidereal_offset", input.sidereal_offset_deg),
    ] {
        if !value.is_finite() {
            return Err(
                Error::invalid_arg(format!("the houses' {name} must be finite")).with_field(name),
            );
        }
    }
    let sun_declination = match (system, input.sun_declination_deg) {
        (HouseSystem::Sunshine, None) => {
            return Err(Error::invalid_arg(
                "the SUNSHINE house system trisects the Sun's own arcs and needs the Sun's declination",
            )
            .with_field("sun_declination_deg"));
        }
        (HouseSystem::Sunshine, Some(dec))
            if dec.is_nan() || dec.abs() > SUN_DECLINATION_BOUND_DEG =>
        {
            return Err(Error::invalid_arg(format!(
                "the Sun's declination {dec}° is outside ±{SUN_DECLINATION_BOUND_DEG}°"
            ))
            .with_field("sun_declination_deg"));
        }
        (_, dec) => dec,
    };
    let frame = Frame::new(input);
    if let Some(cusps) = cusps_of(system, &frame, sun_declination) {
        return Ok(Houses {
            system,
            cusps,
            angles: angles(&frame, system),
            outcome: Outcome::Defined,
        });
    }
    // Undefined at the place: the policy decides.
    match policy {
        PolarPolicy::FallbackPorphyry | PolarPolicy::FallbackWholeSign => {
            let substitute = if policy == PolarPolicy::FallbackPorphyry {
                HouseSystem::Porphyry
            } else {
                HouseSystem::WholeSign
            };
            let cusps = cusps_of(substitute, &frame, None).unwrap_or_else(|| porphyry(&frame));
            Ok(Houses {
                system: substitute,
                cusps,
                angles: angles(&frame, substitute),
                outcome: Outcome::Substituted { asked: system },
            })
        }
        PolarPolicy::Clamp => {
            let edge = 90.0 - input.obliquity_deg - 1e-6;
            let clamped = Input {
                latitude_deg: edge.copysign(input.latitude_deg),
                ..*input
            };
            let clamped_frame = Frame::new(&clamped);
            let cusps = cusps_of(system, &clamped_frame, sun_declination)
                .ok_or_else(|| undefined(system, input.latitude_deg, "even at the polar circle"))?;
            Ok(Houses {
                system,
                cusps,
                angles: angles(&clamped_frame, system),
                outcome: Outcome::Clamped {
                    asked_latitude_deg: input.latitude_deg,
                },
            })
        }
        // `ERROR`, and a policy core adds before this crate learns it.
        _ => Err(undefined(system, input.latitude_deg, "")),
    }
}

fn undefined(system: HouseSystem, latitude_deg: f64, note: &str) -> Error {
    let degeneracy = match system.attributes().degeneracy {
        Degeneracy::PolarUndefined => "inside the polar circle the arc it divides does not exist",
        _ => "the construction does not settle there",
    };
    Error::new(
        Status::Unsupported,
        format!(
            "the {} house system is undefined at latitude {latitude_deg}°{}: {degeneracy}; choose the FALLBACK_PORPHYRY, FALLBACK_WHOLE_SIGN or CLAMP polar policy, or a system without the degeneracy",
            system.key(),
            if note.is_empty() { String::new() } else { format!(" {note}") }
        ),
    )
    .with_field("houses.polar_policy")
}

/// The houses at a UT1 instant and a place: the meridian from apparent
/// sidereal time and the true obliquity from the SDK's own record.
///
/// # Errors
///
/// As [`houses`].
pub fn houses_at(
    system: HouseSystem,
    ut1: JulianDay<Ut1>,
    tt: JulianDay<Tt>,
    place: &Place,
    chart: &ChartFrame,
    policy: PolarPolicy,
) -> Result<Houses, Error> {
    let input = Input {
        armc_deg: sky::sidereal_time_deg(ut1, tt, place.longitude),
        latitude_deg: place.latitude.get(),
        obliquity_deg: sky::obliquity(tt).true_deg,
        sun_declination_deg: chart.sun_declination_deg,
        sidereal_offset_deg: chart.sidereal_offset_deg,
    };
    houses(system, &input, policy)
}

/// What a chart brings to the houses beside its instant and place.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ChartFrame {
    /// The ayanamsha of a sidereal chart, degrees; zero for a tropical one.
    pub sidereal_offset_deg: f64,
    /// The Sun's declination, degrees, when the Sunshine system is asked for.
    pub sun_declination_deg: Option<f64>,
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::indexing_slicing,
        clippy::float_cmp,
        reason = "tests fail by panicking, read twelve cusps and compare snapped cardinal values exactly"
    )]

    use super::*;

    const EPS: f64 = 23.4393;

    fn input(armc: f64, latitude: f64) -> Input {
        Input {
            armc_deg: armc,
            latitude_deg: latitude,
            obliquity_deg: EPS,
            sun_declination_deg: Some(10.0),
            sidereal_offset_deg: 0.0,
        }
    }

    #[test]
    fn the_angles_at_the_equator_and_the_equinox_are_the_textbook_values() {
        // ARMC 0 at the equator: the midheaven is 0° Aries, the ascendant 90°.
        let h = houses(HouseSystem::Equal, &input(0.0, 0.0), PolarPolicy::Error).unwrap();
        assert_eq!(h.angles.midheaven_deg, 0.0);
        assert_eq!(h.angles.ascendant_deg, 90.0);
        assert_eq!(h.cusps[0], 90.0);
        assert_eq!(h.cusps[9], 0.0);
        // ARMC 90: the midheaven is 90°, the ascendant 180° everywhere.
        let h = houses(
            HouseSystem::Porphyry,
            &input(90.0, 45.0),
            PolarPolicy::Error,
        )
        .unwrap();
        assert!((h.angles.midheaven_deg - 90.0).abs() < 1e-9);
        assert!((h.angles.ascendant_deg - 180.0).abs() < 1e-9);
    }

    #[test]
    fn every_system_computes_at_a_temperate_latitude_with_the_angles_in_place() {
        let input = input(258.0 + 40.0 / 60.0, 27.7172);
        for system in HouseSystem::ALL {
            let h = houses(system, &input, PolarPolicy::Error).unwrap();
            assert_eq!(h.outcome, Outcome::Defined, "{system:?}");
            for cusp in h.cusps {
                assert!((0.0..360.0).contains(&cusp), "{system:?} {cusp}");
            }
            let quadrant = [
                HouseSystem::Placidus,
                HouseSystem::Koch,
                HouseSystem::Regiomontanus,
                HouseSystem::Campanus,
                HouseSystem::Porphyry,
                HouseSystem::Topocentric,
                HouseSystem::Alcabitius,
                HouseSystem::PullenSd,
                HouseSystem::PullenSr,
                HouseSystem::Krusinski,
                HouseSystem::Equal,
                HouseSystem::Carter,
                HouseSystem::Sunshine,
                HouseSystem::Apc,
            ];
            if quadrant.contains(&system) {
                assert!(
                    (h.cusps[0] - h.angles.ascendant_deg).abs() < 1e-9,
                    "{system:?}"
                );
            }
            if quadrant.contains(&system)
                && system != HouseSystem::Equal
                && system != HouseSystem::Carter
            {
                assert!(
                    (h.cusps[9] - h.angles.midheaven_deg).abs() < 1e-9,
                    "{system:?}"
                );
            }
            // Opposite cusps oppose, except for the systems whose houses do not.
            if !matches!(system, HouseSystem::Apc | HouseSystem::Sunshine) {
                for i in 0..6 {
                    assert!(
                        (difference_deg(h.cusps[i + 6], h.cusps[i]).abs() - 180.0).abs() < 1e-9,
                        "{system:?} house {}",
                        i + 1
                    );
                }
            }
        }
    }

    #[test]
    fn the_polar_policies_decide_an_undefined_system() {
        let tromso = input(120.0, 69.6492);
        let error = houses(HouseSystem::Placidus, &tromso, PolarPolicy::Error).unwrap_err();
        assert_eq!(error.status, Status::Unsupported);
        assert!(
            error.message.contains("PLACIDUS") && error.message.contains("69.6492"),
            "{error}"
        );
        let porphyry = houses(
            HouseSystem::Placidus,
            &tromso,
            PolarPolicy::FallbackPorphyry,
        )
        .unwrap();
        assert_eq!(porphyry.system, HouseSystem::Porphyry);
        assert_eq!(
            porphyry.outcome,
            Outcome::Substituted {
                asked: HouseSystem::Placidus
            }
        );
        assert_eq!(
            porphyry.cusps,
            houses(HouseSystem::Porphyry, &tromso, PolarPolicy::Error)
                .unwrap()
                .cusps
        );
        let whole = houses(HouseSystem::Koch, &tromso, PolarPolicy::FallbackWholeSign).unwrap();
        assert_eq!(whole.system, HouseSystem::WholeSign);
        let clamped = houses(HouseSystem::Placidus, &tromso, PolarPolicy::Clamp).unwrap();
        assert_eq!(clamped.system, HouseSystem::Placidus);
        assert!(
            matches!(clamped.outcome, Outcome::Clamped { asked_latitude_deg } if asked_latitude_deg == 69.6492)
        );
        // Regiomontanus is defined inside the polar circle.
        let regio = houses(HouseSystem::Regiomontanus, &tromso, PolarPolicy::Error).unwrap();
        assert_eq!(regio.outcome, Outcome::Defined);
    }

    #[test]
    fn sunshine_needs_the_sun_and_refuses_an_impossible_declination() {
        let mut input = input(30.0, 40.0);
        input.sun_declination_deg = None;
        let error = houses(HouseSystem::Sunshine, &input, PolarPolicy::Error).unwrap_err();
        assert_eq!(error.status, Status::InvalidArg);
        assert_eq!(error.field(), Some("sun_declination_deg"));
        input.sun_declination_deg = Some(30.0);
        assert_eq!(
            houses(HouseSystem::Sunshine, &input, PolarPolicy::Error)
                .unwrap_err()
                .status,
            Status::InvalidArg
        );
        input.sun_declination_deg = Some(23.0);
        assert_eq!(
            houses(HouseSystem::Sunshine, &input, PolarPolicy::Error)
                .unwrap()
                .outcome,
            Outcome::Defined
        );
    }

    #[test]
    fn the_sign_systems_take_their_signs_in_the_zodiac_in_use() {
        let mut sidereal = input(258.0, 27.7172);
        sidereal.sidereal_offset_deg = 23.72;
        let whole = houses(HouseSystem::WholeSign, &sidereal, PolarPolicy::Error).unwrap();
        // The sidereal cusps are sign boundaries; the tropical ones carry the offset.
        let sidereal_cusp = normalise_deg(whole.cusps[0] - 23.72);
        assert!((sidereal_cusp % 30.0).abs() < 1e-9, "{sidereal_cusp}");
        let sidereal_ascendant = normalise_deg(whole.angles.ascendant_deg - 23.72);
        assert!(sidereal_cusp <= sidereal_ascendant && sidereal_ascendant < sidereal_cusp + 30.0);
        let aries = houses(HouseSystem::EqualAries, &sidereal, PolarPolicy::Error).unwrap();
        assert!((aries.cusps[0] - 23.72).abs() < 1e-9);
    }

    #[test]
    fn a_non_finite_input_is_refused_by_name() {
        let mut bad = input(0.0, 10.0);
        bad.latitude_deg = f64::NAN;
        let error = houses(HouseSystem::Equal, &bad, PolarPolicy::Error).unwrap_err();
        assert_eq!(error.field(), Some("latitude"));
    }
}
