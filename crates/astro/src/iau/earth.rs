//! The Earth as a body an observer stands on: the reference ellipsoids,
//! the geodetic to geocentric transformation, the polar motion matrix and
//! the position and velocity of a terrestrial station, ported from ERFA
//! as [`super`] describes.
//!
//! These are the routines the completion's topocentric step is built on
//! (`docs/03-design/astro-timescales-and-frames.md`, §4). ERFA's station
//! routine takes the Earth rotation angle and answers in the celestial
//! intermediate system; the SDK's frames are equinox-based, so
//! [`sky::observer`](crate::sky::observer) passes Greenwich apparent
//! sidereal time instead and reads the answer in the true equator and
//! equinox of date. The substitution is a rotation about the same axis by
//! a different angle, which is what both quantities are.

use super::vector::{self, Matrix3, Vector3};
use super::{D2PI, DAS2R, DAYSEC, DJ00, DJC};

/// A reference ellipsoid the geodetic coordinates of a place are measured
/// against (`eraEform`'s identifiers, whose invalid values cannot be
/// written here).
///
/// ```
/// use teistro_astro::iau::earth::Ellipsoid;
///
/// let (a, f) = Ellipsoid::Wgs84.parameters();
/// assert!((a - 6_378_137.0).abs() < 1e-10);
/// assert!((f - 0.335_281_066_474_748_072e-2).abs() < 1e-18);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Ellipsoid {
    /// WGS84, the ellipsoid GPS and every place in the SDK is given on.
    #[default]
    Wgs84,
    /// GRS80.
    Grs80,
    /// WGS72.
    Wgs72,
}

impl Ellipsoid {
    /// Every ellipsoid, in ERFA's identifier order.
    pub const ALL: [Ellipsoid; 3] = [Ellipsoid::Wgs84, Ellipsoid::Grs80, Ellipsoid::Wgs72];

    /// The equatorial radius in metres and the flattening (`eraEform`).
    /// ERFA returns these through pointers with a status for an
    /// identifier it does not know; the enumeration makes that status
    /// unreachable, so there is none.
    #[must_use]
    pub const fn parameters(self) -> (f64, f64) {
        match self {
            Ellipsoid::Wgs84 => (6_378_137.0, 1.0 / 298.257_223_563),
            Ellipsoid::Grs80 => (6_378_137.0, 1.0 / 298.257_222_101),
            Ellipsoid::Wgs72 => (6_378_135.0, 1.0 / 298.26),
        }
    }

    /// The name stamped in provenance.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Ellipsoid::Wgs84 => "WGS84",
            Ellipsoid::Grs80 => "GRS80",
            Ellipsoid::Wgs72 => "WGS72",
        }
    }
}

/// Geodetic coordinates to a geocentric vector on a given ellipsoid
/// (`eraGd2gce`): longitude east of Greenwich and geodetic latitude in
/// radians, height above the ellipsoid and the equatorial radius in
/// metres, and the answer in metres in the terrestrial frame.
///
/// `None` for a flattening the transformation cannot invert, which is
/// ERFA's `-1`. The quantity it guards is a sum of squares, so no
/// flattening and latitude a caller can write actually reaches it; the
/// refusal is kept because the signature takes both.
#[must_use]
#[allow(
    clippy::many_single_char_names,
    reason = "ERFA's own names, kept so the C and the Rust read side by side"
)]
pub fn gd2gce(a: f64, f: f64, elong: f64, phi: f64, height: f64) -> Option<Vector3> {
    // Functions of geodetic latitude.
    let sp = phi.sin();
    let cp = phi.cos();
    let w = 1.0 - f;
    let w = w * w;
    let d = cp * cp + w * sp * sp;
    if d <= 0.0 {
        return None;
    }
    let ac = a / d.sqrt();
    let as_ = w * ac;
    // Geocentric vector.
    let r = (ac + height) * cp;
    Some([r * elong.cos(), r * elong.sin(), (as_ + height) * sp])
}

/// Geodetic coordinates to a geocentric vector on a named ellipsoid
/// (`eraGd2gc`), metres. ERFA's two error returns — an unknown
/// identifier and a flattening it cannot invert — are an enumeration and
/// an ellipsoid whose flattening always inverts, so this cannot fail.
#[must_use]
pub fn gd2gc(ellipsoid: Ellipsoid, elong: f64, phi: f64, height: f64) -> Vector3 {
    let (a, f) = ellipsoid.parameters();
    gd2gce(a, f, elong, phi, height).unwrap_or([0.0, 0.0, 0.0])
}

/// The TIO locator s′ in radians at a TT date (`eraSp00`): the
/// positioning of the terrestrial intermediate origin on the equator,
/// −47 microarcseconds a century, which is the only part of it that is
/// predictable.
#[must_use]
pub fn sp00(date1: f64, date2: f64) -> f64 {
    // Interval between fundamental epoch J2000.0 and current date (JC).
    let t = ((date1 - DJ00) + date2) / DJC;
    // Approximate s'.
    -47e-6 * t * DAS2R
}

/// The polar motion matrix (`eraPom00`) from the pole's coordinates and
/// the TIO locator, all radians: the rotation from the international
/// terrestrial system to the terrestrial intermediate one.
#[must_use]
pub fn pom00(xp: f64, yp: f64, sp: f64) -> Matrix3 {
    // Construct the matrix.
    let mut rpom = vector::ir();
    vector::rz(sp, &mut rpom);
    vector::ry(-xp, &mut rpom);
    vector::rx(-yp, &mut rpom);
    rpom
}

/// The Earth's rotation rate in radians per UT1 second, the constant
/// `eraPvtob` carries: the sidereal rate, so a station returns to the
/// same direction in a sidereal day rather than a solar one.
#[allow(
    clippy::excessive_precision,
    reason = "the ratio is quoted as ERFA writes it"
)]
pub const ROTATION_RATE: f64 = 1.002_737_811_911_354_48 * D2PI / DAYSEC;

/// The position and velocity of a terrestrial observing station
/// (`eraPvtob`): metres and metres a second, in the celestial system the
/// rotation angle `theta` refers the station's meridian to.
///
/// `elong` and `phi` are the geodetic longitude east of Greenwich and
/// latitude in radians, `hm` the height above the WGS84 ellipsoid in
/// metres, `xp` and `yp` the pole's coordinates and `sp` the TIO locator
/// in radians. Pass the Earth rotation angle as ERFA does for a
/// celestial-intermediate answer, or Greenwich apparent sidereal time
/// for one in the true equator and equinox of date.
#[must_use]
#[allow(
    clippy::many_single_char_names,
    clippy::too_many_arguments,
    reason = "ERFA's own names and signature, kept so the C and the Rust read side by side"
)]
pub fn pvtob(elong: f64, phi: f64, hm: f64, xp: f64, yp: f64, sp: f64, theta: f64) -> [Vector3; 2] {
    // Earth rotation rate in radians per UT1 second.
    const OM: f64 = ROTATION_RATE;

    // Geodetic to geocentric transformation (WGS84).
    let xyzm = gd2gc(Ellipsoid::Wgs84, elong, phi, hm);
    // Polar motion and TIO position.
    let rpm = pom00(xp, yp, sp);
    let [x, y, z] = vector::trxp(&rpm, &xyzm);
    // Functions of ERA.
    let (sin, cos) = theta.sin_cos();
    [
        // Position.
        [cos * x - sin * y, sin * x + cos * y, z],
        // Velocity.
        [OM * (-sin * x - cos * y), OM * (cos * x - sin * y), 0.0],
    ]
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::excessive_precision,
        clippy::unreadable_literal,
        reason = "tests fail by panicking, and the reference values are quoted as ERFA prints them"
    )]

    use super::*;

    fn vvd(value: f64, expected: f64, tolerance: f64, name: &str) {
        assert!(
            (value - expected).abs() <= tolerance,
            "{name}: {value} against {expected}"
        );
    }

    #[test]
    fn eform_reproduces_erfa() {
        let (a, f) = Ellipsoid::Wgs84.parameters();
        vvd(a, 6378137.0, 1e-10, "a1");
        vvd(f, 0.3352810664747480720e-2, 1e-18, "f1");
        let (a, f) = Ellipsoid::Grs80.parameters();
        vvd(a, 6378137.0, 1e-10, "a2");
        vvd(f, 0.3352810681182318935e-2, 1e-18, "f2");
        let (a, f) = Ellipsoid::Wgs72.parameters();
        vvd(a, 6378135.0, 1e-10, "a3");
        vvd(f, 0.3352779454167504862e-2, 1e-18, "f3");
        // The identifier ERFA answers `-1` for cannot be written, and
        // every ellipsoid names itself for a stamp.
        assert_eq!(
            Ellipsoid::ALL.map(Ellipsoid::key),
            ["WGS84", "GRS80", "WGS72"]
        );
        assert_eq!(Ellipsoid::default(), Ellipsoid::Wgs84);
    }

    #[test]
    fn gd2gc_and_gd2gce_reproduce_erfa() {
        let (e, p, h) = (3.1, -0.5, 2500.0);
        let xyz = gd2gc(Ellipsoid::Wgs84, e, p, h);
        vvd(xyz[0], -5599000.5577049947, 1e-7, "1/1");
        vvd(xyz[1], 233011.67223479203, 1e-7, "2/1");
        vvd(xyz[2], -3040909.4706983363, 1e-7, "3/1");
        let xyz = gd2gc(Ellipsoid::Grs80, e, p, h);
        vvd(xyz[0], -5599000.5577260984, 1e-7, "1/2");
        vvd(xyz[1], 233011.6722356702949, 1e-7, "2/2");
        vvd(xyz[2], -3040909.4706095476, 1e-7, "3/2");
        let xyz = gd2gc(Ellipsoid::Wgs72, e, p, h);
        vvd(xyz[0], -5598998.7626301490, 1e-7, "1/3");
        vvd(xyz[1], 233011.5975297822211, 1e-7, "2/3");
        vvd(xyz[2], -3040908.6861467111, 1e-7, "3/3");
        let xyz = gd2gce(6378136.0, 0.0033528, e, p, h).unwrap_or_default();
        vvd(xyz[0], -5598999.6665116328, 1e-7, "1");
        vvd(xyz[1], 233011.6351463057189, 1e-7, "2");
        vvd(xyz[2], -3040909.0517314132, 1e-7, "3");
        // ERFA's second status, which the sum of squares behind it makes
        // unreachable: the quantity it guards is `cos²φ + (1−f)² sin²φ`,
        // which is zero only for a flattening of one at a latitude whose
        // cosine is exactly zero, and no double is. The refusal is kept
        // because the signature takes an arbitrary flattening; nothing in
        // the sky can reach it.
        for step in 0..=36 {
            let phi = f64::from(step)
                .mul_add(core::f64::consts::PI / 36.0, -core::f64::consts::FRAC_PI_2);
            assert!(gd2gce(6378136.0, 1.0, e, phi, h).is_some());
        }
    }

    #[test]
    fn sp00_and_pom00_reproduce_erfa() {
        vvd(
            sp00(2400000.5, 52541.0),
            -0.6216698469981019309e-11,
            1e-12,
            "sp00",
        );
        let rpom = pom00(2.55060238e-7, 1.860359247e-6, -0.1367174580728891460e-10);
        vvd(rpom[0][0], 0.9999999999999674721, 1e-12, "11");
        vvd(rpom[0][1], -0.1367174580728846989e-10, 1e-16, "12");
        vvd(rpom[0][2], 0.2550602379999972345e-6, 1e-16, "13");
        vvd(rpom[1][0], 0.1414624947957029801e-10, 1e-16, "21");
        vvd(rpom[1][1], 0.9999999999982695317, 1e-12, "22");
        vvd(rpom[1][2], -0.1860359246998866389e-5, 1e-16, "23");
        vvd(rpom[2][0], -0.2550602379741215021e-6, 1e-16, "31");
        vvd(rpom[2][1], 0.1860359247002414021e-5, 1e-16, "32");
        vvd(rpom[2][2], 0.9999999999982370039, 1e-12, "33");
    }

    #[test]
    fn pvtob_reproduces_erfa() {
        let pv = pvtob(2.0, 0.5, 3000.0, 1e-6, -0.5e-6, 1e-8, 5.0);
        vvd(pv[0][0], 4225081.367071159207, 1e-5, "p(1)");
        vvd(pv[0][1], 3681943.215856198144, 1e-5, "p(2)");
        vvd(pv[0][2], 3041149.399241260785, 1e-5, "p(3)");
        vvd(pv[1][0], -268.4915389365998787, 1e-9, "v(1)");
        vvd(pv[1][1], 308.0977983288903123, 1e-9, "v(2)");
        vvd(pv[1][2], 0.0, 0.0, "v(3)");
    }

    #[test]
    fn the_station_turns_with_the_earth() {
        // Polar motion left out, a station on the equator at Greenwich
        // stands one equatorial radius out and moves eastward at the
        // equatorial speed; the pole stands still.
        let pv = pvtob(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        vvd(pv[0][0], 6_378_137.0, 1e-6, "equator");
        vvd(pv[1][1], 465.101, 1e-3, "eastward");
        let pole = pvtob(0.0, core::f64::consts::FRAC_PI_2, 0.0, 0.0, 0.0, 0.0, 1.0);
        assert!(vector::pm(&pole[1]) < 1e-9, "the pole does not move");
        // A quarter turn later the station has swung a quarter of the way
        // round, and its height goes straight into its distance.
        let quarter = pvtob(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, core::f64::consts::FRAC_PI_2);
        vvd(quarter[0][1], 6_378_137.0, 1e-6, "a quarter turn");
        let high = pvtob(0.0, 0.0, 1_000.0, 0.0, 0.0, 0.0, 0.0);
        vvd(high[0][0], 6_379_137.0, 1e-6, "a kilometre up");
    }
}
