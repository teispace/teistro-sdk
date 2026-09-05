//! Precession as a catalogue of models over the ported IAU routines: the
//! matrix from the mean equator and equinox of J2000.0 to the mean of a
//! date, its inverse, the rotation between two dates, and the mean
//! obliquity each model is consistent with
//! (`docs/03-design/astro-timescales-and-frames.md`).
//!
//! The default is Vondrák, Capitaine and Wallace (2011), valid over two
//! hundred millennia either side of J2000.0, which the ayanamsha catalogue
//! needs for epochs in the first millennium BCE; IAU 2006 is the modern
//! short-term model; IAU 1976 and Newcomb are the older rotation-angle
//! formulations kept because several published ayanamsha constants were
//! fitted with them.
//!
//! ```
//! use teistro_astro::precession::{PrecessionModel, mean_obliquity_deg, to_date};
//! use teistro_core::quantity::{JulianDay, Tt};
//!
//! let tt = JulianDay::<Tt>::literal(2_451_545.0 + 36_525.0);
//! // The J2000.0 equinox seen from 2100: the equinox has moved west, so the
//! // old one lies about 1.39° east along the equator.
//! let x = to_date(PrecessionModel::Vondrak2011, tt, [1.0, 0.0, 0.0]);
//! assert!((x[1].atan2(x[0]).to_degrees() - 1.28).abs() < 0.02);
//! assert!((mean_obliquity_deg(PrecessionModel::Iau2006, tt) - 23.426).abs() < 0.001);
//! ```

use serde::{Deserialize, Serialize};
use teistro_core::quantity::{JulianDay, Tt};

use crate::iau::vector::{Matrix3, Vector3, ir, rx, rxp, ry, rz, trxp};
use crate::iau::{DAS2R, DJ00, DJC, DJY, RAD2DEG, ltp, obl06, obl80, p06};

/// A precession model.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PrecessionModel {
    /// Vondrák, Capitaine and Wallace (2011): the long-term model, the
    /// default.
    #[default]
    Vondrak2011,
    /// IAU 2006 (Capitaine, Wallace and Chapront 2003, P03).
    Iau2006,
    /// IAU 1976 (Lieske, Lederle, Fricke and Morando 1977), the
    /// rotation-angle formulation.
    Iau1976,
    /// Newcomb's precession in Kinoshita's (1975) form, referred to B1850.
    Newcomb,
}

impl PrecessionModel {
    /// The stable key.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            PrecessionModel::Vondrak2011 => "VONDRAK_2011",
            PrecessionModel::Iau2006 => "IAU_2006",
            PrecessionModel::Iau1976 => "IAU_1976",
            PrecessionModel::Newcomb => "NEWCOMB",
        }
    }
}

/// The Julian epoch of a TT instant.
fn julian_epoch(tt: JulianDay<Tt>) -> f64 {
    let (date1, date2) = tt.split();
    2000.0 + ((date1 - DJ00) + date2) / DJY
}

/// The three equatorial precession angles (ζ, z, θ), radians, of the
/// rotation-angle models.
fn rotation_angles(model: PrecessionModel, tt: JulianDay<Tt>) -> (f64, f64, f64) {
    let (date1, date2) = tt.split();
    match model {
        PrecessionModel::Iau1976 => {
            // Lieske et al. (1977), Julian centuries from J2000.0.
            let t = ((date1 - DJ00) + date2) / DJC;
            let zeta = ((0.017_998 * t + 0.301_88) * t + 2306.2181) * t;
            let z = ((0.018_203 * t + 1.094_68) * t + 2306.2181) * t;
            let theta = ((-0.041_833 * t - 0.426_65) * t + 2004.3109) * t;
            (zeta * DAS2R, z * DAS2R, theta * DAS2R)
        }
        PrecessionModel::Newcomb => {
            // Kinoshita (1975): tropical millennia from B1850.
            const B1850: f64 = 2_396_758.203_581_0;
            const MILLENNIUM: f64 = 365_242.198_782;
            let t1 = (DJ00 - B1850) / MILLENNIUM;
            let t2 = ((date1 - B1850) + date2) / MILLENNIUM;
            let dt = t2 - t1;
            let dt2 = dt * dt;
            let dt3 = dt2 * dt;
            let z1 = 23_035.554_8 + 139.720 * t1 + 0.069 * t1 * t1;
            let zeta = z1 * dt + (30.242 - 0.269 * t1) * dt2 + 17.996 * dt3;
            let z = z1 * dt + (109.478 - 0.387 * t1) * dt2 + 18.324 * dt3;
            let theta = (20_051.125 - 85.294 * t1 - 0.365 * t1 * t1) * dt
                + (-42.647 - 0.365 * t1) * dt2
                - 41.802 * dt3;
            (zeta * DAS2R, z * DAS2R, theta * DAS2R)
        }
        // The matrix models never reach here.
        PrecessionModel::Vondrak2011 | PrecessionModel::Iau2006 => (0.0, 0.0, 0.0),
    }
}

/// The precession matrix from the mean equator and equinox of J2000.0 to
/// the mean equator and equinox of a TT date, without the frame bias.
#[must_use]
pub fn matrix(model: PrecessionModel, tt: JulianDay<Tt>) -> Matrix3 {
    match model {
        PrecessionModel::Vondrak2011 => ltp::ltp(julian_epoch(tt)),
        PrecessionModel::Iau2006 => {
            let (date1, date2) = tt.split();
            p06::bp06(date1, date2).1
        }
        PrecessionModel::Iau1976 | PrecessionModel::Newcomb => {
            // The 323 Euler rotation, as ERFA's `pmat76` composes it.
            let (zeta, z, theta) = rotation_angles(model, tt);
            let mut r = ir();
            rz(-zeta, &mut r);
            ry(theta, &mut r);
            rz(-z, &mut r);
            r
        }
    }
}

/// A J2000.0 mean equatorial vector precessed to the mean equator and
/// equinox of a TT date.
#[must_use]
pub fn to_date(model: PrecessionModel, tt: JulianDay<Tt>, v: Vector3) -> Vector3 {
    rxp(&matrix(model, tt), &v)
}

/// A mean-of-date equatorial vector precessed back to J2000.0.
#[must_use]
pub fn to_j2000(model: PrecessionModel, tt: JulianDay<Tt>, v: Vector3) -> Vector3 {
    trxp(&matrix(model, tt), &v)
}

/// A mean-of-date equatorial vector carried from one TT date to another.
#[must_use]
pub fn between(
    model: PrecessionModel,
    from: JulianDay<Tt>,
    to: JulianDay<Tt>,
    v: Vector3,
) -> Vector3 {
    to_date(model, to, to_j2000(model, from, v))
}

/// The mean obliquity of the ecliptic consistent with a precession model,
/// radians: Vondrák's own series, IAU 2006's polynomial, IAU 1980's for
/// IAU 1976, and Newcomb's referred to 1850.
#[must_use]
pub fn mean_obliquity_rad(model: PrecessionModel, tt: JulianDay<Tt>) -> f64 {
    let (date1, date2) = tt.split();
    match model {
        PrecessionModel::Vondrak2011 => ltp::ltpeps(julian_epoch(tt)).1,
        PrecessionModel::Iau2006 => obl06(date1, date2),
        PrecessionModel::Iau1976 => obl80(date1, date2),
        PrecessionModel::Newcomb => {
            let t = ((date1 - 2_396_758.0) + date2) / DJC;
            (((0.0017 * t - 0.0085) * t - 46.837) * t + 84_451.68) * DAS2R
        }
    }
}

/// The mean obliquity consistent with a model, degrees.
#[must_use]
pub fn mean_obliquity_deg(model: PrecessionModel, tt: JulianDay<Tt>) -> f64 {
    mean_obliquity_rad(model, tt) * RAD2DEG
}

/// Rotates a mean equatorial vector of date into the ecliptic of the same
/// date (about the x axis by the obliquity).
#[must_use]
pub fn equatorial_to_ecliptic(v: Vector3, obliquity_rad: f64) -> Vector3 {
    let mut r = ir();
    rx(obliquity_rad, &mut r);
    rxp(&r, &v)
}

/// Rotates an ecliptic vector of date into the mean equator of the same
/// date.
#[must_use]
pub fn ecliptic_to_equatorial(v: Vector3, obliquity_rad: f64) -> Vector3 {
    let mut r = ir();
    rx(-obliquity_rad, &mut r);
    rxp(&r, &v)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and read fixed matrices"
    )]

    use crate::iau::vector::{c2s, pdp};

    use super::*;

    const ALL: [PrecessionModel; 4] = [
        PrecessionModel::Vondrak2011,
        PrecessionModel::Iau2006,
        PrecessionModel::Iau1976,
        PrecessionModel::Newcomb,
    ];

    #[test]
    fn every_model_is_the_identity_at_j2000_and_orthogonal_elsewhere() {
        let j2000 = JulianDay::<Tt>::literal(DJ00);
        for model in ALL {
            let m = matrix(model, j2000);
            for (i, row) in m.iter().enumerate() {
                for (j, cell) in row.iter().enumerate() {
                    let expected = if i == j { 1.0 } else { 0.0 };
                    assert!((cell - expected).abs() < 1e-7, "{model:?} {i}{j} {cell}");
                }
            }
            let tt = JulianDay::<Tt>::literal(DJ00 + 3.0 * DJC);
            let m = matrix(model, tt);
            for i in 0..3 {
                for j in 0..3 {
                    let dot = pdp(&m[i], &m[j]);
                    let expected = if i == j { 1.0 } else { 0.0 };
                    assert!((dot - expected).abs() < 1e-12, "{model:?} rows {i} {j}");
                }
            }
            // Round trip through J2000.
            let v = [0.3, -0.5, 0.8];
            let back = to_j2000(model, tt, to_date(model, tt, v));
            for k in 0..3 {
                assert!((back[k] - v[k]).abs() < 1e-14, "{model:?} round trip");
            }
        }
    }

    #[test]
    fn the_models_agree_on_the_general_precession_over_a_century() {
        // The J2000.0 equinox seen a century later: the equinox moves west,
        // so the old one lies 1.396° east along the ecliptic of date, the
        // textbook 50.3″ a year of general precession.
        let tt = JulianDay::<Tt>::literal(DJ00 + DJC);
        for model in ALL {
            let x = to_date(model, tt, [1.0, 0.0, 0.0]);
            let ecl = equatorial_to_ecliptic(x, mean_obliquity_rad(model, tt));
            let (lon, _) = c2s(&ecl);
            let deg = lon.to_degrees();
            assert!((deg - 1.396).abs() < 0.002, "{model:?}: {deg}");
        }
        // Vondrák and IAU 2006 agree to a few milliarcseconds over that
        // century (measured 4.3 mas), the two solutions' own difference.
        let a = to_date(PrecessionModel::Vondrak2011, tt, [0.0, 1.0, 0.0]);
        let b = to_date(PrecessionModel::Iau2006, tt, [0.0, 1.0, 0.0]);
        let apart = ((pdp(&a, &b)).clamp(-1.0, 1.0)).acos() / DAS2R;
        assert!(apart < 0.01, "{apart}\"");
    }

    #[test]
    fn the_obliquities_are_consistent_and_between_is_a_composition() {
        let tt = JulianDay::<Tt>::literal(DJ00 - 20.0 * DJC);
        let vondrak = mean_obliquity_deg(PrecessionModel::Vondrak2011, tt);
        let iau = mean_obliquity_deg(PrecessionModel::Iau2006, tt);
        assert!((vondrak - iau).abs() < 0.002, "{vondrak} {iau}");
        assert!(
            (mean_obliquity_deg(PrecessionModel::Newcomb, JulianDay::literal(DJ00)) - 23.4393)
                .abs()
                < 0.0005
        );
        let from = JulianDay::<Tt>::literal(DJ00 - DJC);
        let to = JulianDay::<Tt>::literal(DJ00 + DJC);
        let v = [0.6, 0.0, 0.8];
        let direct = between(PrecessionModel::Vondrak2011, from, to, v);
        let stepwise = to_date(
            PrecessionModel::Vondrak2011,
            to,
            to_j2000(PrecessionModel::Vondrak2011, from, v),
        );
        for (a, b) in direct.iter().zip(stepwise.iter()) {
            assert!((a - b).abs() < 1e-15, "{a} {b}");
        }
        assert_eq!(PrecessionModel::default().key(), "VONDRAK_2011");
    }
}
