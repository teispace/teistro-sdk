//! The IAU routines frame completion needs, and the coordinate rotation.
//!
//! Ported from ERFA (BSD-3-Clause, the `NumFOCUS` Foundation, derived with
//! permission from SOFA), as ADR-0021 plans for the whole `astro` layer;
//! the notice is in the repository's `NOTICE`. The provenance table:
//!
//! | here | ERFA | revision | change |
//! |---|---|---|---|
//! | [`mean_obliquity_iau2006_rad`] | `eraObl06` | 2021 May 11 | one `f64` Julian Day instead of the two-part date |
//! | [`nutation_iau2000b_rad`] | `eraNut00b` | 2021 May 11 | one `f64` Julian Day; the 77-term table verbatim in [`nutation`] |
//!
//! The one-part date costs about 1e-9 day of resolution, a nanosecond of
//! nutation argument; the SDK's `astro` crate uses the two-part form.
//!
//! Delta T is the Espenak and Meeus (2006) polynomial fit, cited in
//! [`delta_t_seconds_approx`]; it is a stand-in for the SDK's Delta T
//! models and is what the `sdk-only` policy uses in this spike.

pub mod nutation;

use crate::model::Obliquity;

/// J2000.0 as a Julian Day.
pub const DJ00: f64 = 2_451_545.0;
/// Days per Julian century.
pub const DJC: f64 = 36_525.0;
/// Arcseconds to radians.
pub const DAS2R: f64 = core::f64::consts::PI / 648_000.0;
/// Milliarcseconds to radians.
pub const DMAS2R: f64 = DAS2R / 1e3;
/// Arcseconds in a full circle.
pub const TURNAS: f64 = 1_296_000.0;
/// Degrees to radians.
pub const DEG2RAD: f64 = core::f64::consts::PI / 180.0;
/// Radians to degrees.
pub const RAD2DEG: f64 = 180.0 / core::f64::consts::PI;

/// Julian centuries since J2000.0 of a TT Julian Day.
#[must_use]
pub fn centuries_tt(jd_tt: f64) -> f64 {
    (jd_tt - DJ00) / DJC
}

/// Mean obliquity of the ecliptic, IAU 2006 precession model, radians.
/// Port of `eraObl06`.
#[must_use]
pub fn mean_obliquity_iau2006_rad(jd_tt: f64) -> f64 {
    let t = centuries_tt(jd_tt);
    (84_381.406
        + (-46.836_769
            + (-0.000_183_1 + (0.002_003_40 + (-0.000_000_576 + (-0.000_000_043_4) * t) * t) * t)
                * t)
            * t)
        * DAS2R
}

/// Nutation in longitude and obliquity, IAU 2000B model, radians.
/// Port of `eraNut00b`: the 77 luni-solar terms plus the fixed offsets in
/// lieu of the planetary terms (Luzum 2001), accurate to about a
/// milliarcsecond over 1995 to 2050.
#[must_use]
pub fn nutation_iau2000b_rad(jd_tt: f64) -> Nutation {
    let t = centuries_tt(jd_tt);
    let arg = |a: f64, b: f64| (a + b * t).rem_euclid(TURNAS) * DAS2R;
    // Fundamental arguments of Simon et al. (1994), as the model uses them.
    let el = arg(485_868.249_036, 1_717_915_923.217_8);
    let elp = arg(1_287_104.793_05, 129_596_581.048_1);
    let f = arg(335_779.526_232, 1_739_527_262.847_8);
    let d = arg(1_072_260.703_69, 1_602_961_601.209_0);
    let om = arg(450_160.398_036, -6_962_890.543_1);
    let mut dp = 0.0;
    let mut de = 0.0;
    // Smallest terms first, as the reference does, for the same rounding.
    for term in nutation::TERMS.iter().rev() {
        let a = (f64::from(term.nl) * el
            + f64::from(term.nlp) * elp
            + f64::from(term.nf) * f
            + f64::from(term.nd) * d
            + f64::from(term.nom) * om)
            .rem_euclid(core::f64::consts::TAU);
        let (sarg, carg) = a.sin_cos();
        dp += (term.ps + term.pst * t) * sarg + term.pc * carg;
        de += (term.ec + term.ect * t) * carg + term.es * sarg;
    }
    // Units of 0.1 microarcsecond to radians, then the planetary offsets.
    let u2r = DAS2R / 1e7;
    Nutation {
        dpsi_rad: dp * u2r + (-0.135 * DMAS2R),
        deps_rad: de * u2r + (0.388 * DMAS2R),
    }
}

/// Nutation in longitude and obliquity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Nutation {
    /// Nutation in longitude, radians.
    pub dpsi_rad: f64,
    /// Nutation in obliquity, radians.
    pub deps_rad: f64,
}

/// The SDK's own obliquity and nutation at a TT instant, degrees.
#[must_use]
pub fn obliquity(jd_tt: f64) -> Obliquity {
    let mean = mean_obliquity_iau2006_rad(jd_tt);
    let nut = nutation_iau2000b_rad(jd_tt);
    Obliquity {
        mean_deg: mean * RAD2DEG,
        true_deg: (mean + nut.deps_rad) * RAD2DEG,
        nutation_lon_deg: nut.dpsi_rad * RAD2DEG,
        nutation_obl_deg: nut.deps_rad * RAD2DEG,
    }
}

/// Delta T in seconds from the polynomial fits of Espenak and Meeus
/// (2006, "Five Millennium Canon of Solar Eclipses", NASA/TP-2006-214141),
/// with the 2050 to 2150 expression of the same source and the long-term
/// parabola outside. Good to a few seconds after 2005, which moves the
/// nutation argument by a microarcsecond; the SDK's Delta T models replace
/// it.
#[must_use]
pub fn delta_t_seconds_approx(jd_ut1: f64) -> f64 {
    let y = 2000.0 + (jd_ut1 - DJ00) / 365.25;
    if (1900.0..1920.0).contains(&y) {
        let t = y - 1900.0;
        -2.79 + 1.494_119 * t - 0.059_893_9 * t * t + 0.006_196_6 * t.powi(3)
            - 0.000_197 * t.powi(4)
    } else if (1920.0..1941.0).contains(&y) {
        let t = y - 1920.0;
        21.20 + 0.844_93 * t - 0.076_100 * t * t + 0.002_093_6 * t.powi(3)
    } else if (1941.0..1961.0).contains(&y) {
        let t = y - 1950.0;
        29.07 + 0.407 * t - t * t / 233.0 + t.powi(3) / 2547.0
    } else if (1961.0..1986.0).contains(&y) {
        let t = y - 1975.0;
        45.45 + 1.067 * t - t * t / 260.0 - t.powi(3) / 718.0
    } else if (1986.0..2005.0).contains(&y) {
        let t = y - 2000.0;
        63.86 + 0.3345 * t - 0.060_374 * t * t
            + 0.001_727_5 * t.powi(3)
            + 0.000_651_814 * t.powi(4)
            + 0.000_023_735_99 * t.powi(5)
    } else if (2005.0..2050.0).contains(&y) {
        let t = y - 2000.0;
        62.92 + 0.322_17 * t + 0.005_589 * t * t
    } else if (2050.0..2150.0).contains(&y) {
        let u = (y - 1820.0) / 100.0;
        -20.0 + 32.0 * u * u - 0.5628 * (2150.0 - y)
    } else {
        let u = (y - 1820.0) / 100.0;
        -20.0 + 32.0 * u * u
    }
}

/// TT from UT1 and Delta T.
#[must_use]
pub fn tt_from_ut1(jd_ut1: f64, delta_t_seconds: f64) -> f64 {
    jd_ut1 + delta_t_seconds / 86_400.0
}

/// Degrees into `[0, 360)`.
#[must_use]
pub fn normalise_deg(deg: f64) -> f64 {
    let wrapped = deg.rem_euclid(360.0);
    if wrapped >= 360.0 { 0.0 } else { wrapped }
}

/// A spherical position: longitude or right ascension and latitude or
/// declination, degrees.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spherical {
    /// Longitude or right ascension, degrees.
    pub lon_deg: f64,
    /// Latitude or declination, degrees.
    pub lat_deg: f64,
}

/// Rotates ecliptic longitude and latitude to right ascension and
/// declination with an obliquity, all degrees.
#[must_use]
pub fn ecliptic_to_equatorial(p: Spherical, obliquity_deg: f64) -> Spherical {
    let (sl, cl) = (p.lon_deg * DEG2RAD).sin_cos();
    let (sb, cb) = (p.lat_deg * DEG2RAD).sin_cos();
    let (se, ce) = (obliquity_deg * DEG2RAD).sin_cos();
    let ra = (sl * ce - (sb / cb) * se).atan2(cl);
    let dec = (sb * ce + cb * se * sl).asin();
    Spherical {
        lon_deg: normalise_deg(ra * RAD2DEG),
        lat_deg: dec * RAD2DEG,
    }
}

/// Rotates right ascension and declination to ecliptic longitude and
/// latitude with an obliquity, all degrees.
#[must_use]
pub fn equatorial_to_ecliptic(p: Spherical, obliquity_deg: f64) -> Spherical {
    let (sa, ca) = (p.lon_deg * DEG2RAD).sin_cos();
    let (sd, cd) = (p.lat_deg * DEG2RAD).sin_cos();
    let (se, ce) = (obliquity_deg * DEG2RAD).sin_cos();
    let lon = (sa * ce + (sd / cd) * se).atan2(ca);
    let lat = (sd * ce - cd * se * sa).asin();
    Spherical {
        lon_deg: normalise_deg(lon * RAD2DEG),
        lat_deg: lat * RAD2DEG,
    }
}

/// The smaller signed difference between two angles in degrees.
#[must_use]
pub fn angle_difference_deg(a: f64, b: f64) -> f64 {
    let d = (a - b).rem_euclid(360.0);
    if d > 180.0 { d - 360.0 } else { d }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, reason = "a test fails by panicking")]

    use super::*;

    #[test]
    fn mean_obliquity_at_j2000_is_the_iau_2006_constant() {
        let eps = mean_obliquity_iau2006_rad(DJ00) / DAS2R;
        assert!((eps - 84_381.406).abs() < 1e-9, "{eps}");
    }

    #[test]
    fn nutation_at_j2000_matches_the_published_value() {
        // ERFA t_erfa_c: eraNut00b(2400000.5, 53736.0) gives
        // dpsi = -0.9632552291148362783e-5, deps = 0.4063197106621159367e-4.
        let n = nutation_iau2000b_rad(2_400_000.5 + 53_736.0);
        assert!(
            (n.dpsi_rad - (-9.632_552_291_148_363e-6)).abs() < 1e-13,
            "{}",
            n.dpsi_rad
        );
        assert!(
            (n.deps_rad - 4.063_197_106_621_159e-5).abs() < 1e-13,
            "{}",
            n.deps_rad
        );
    }

    #[test]
    fn rotation_round_trips_and_keeps_the_ecliptic_pole() {
        let eps = obliquity(2_460_000.5).true_deg;
        for (lon, lat) in [(0.0, 0.0), (123.456, -5.5), (359.9, 4.2), (270.0, 60.0)] {
            let eq = ecliptic_to_equatorial(
                Spherical {
                    lon_deg: lon,
                    lat_deg: lat,
                },
                eps,
            );
            let back = equatorial_to_ecliptic(eq, eps);
            assert!(
                angle_difference_deg(back.lon_deg, lon).abs() < 1e-11,
                "{lon} {}",
                back.lon_deg
            );
            assert!((back.lat_deg - lat).abs() < 1e-11);
        }
        let pole = ecliptic_to_equatorial(
            Spherical {
                lon_deg: 90.0,
                lat_deg: 0.0,
            },
            eps,
        );
        assert!((pole.lat_deg - eps).abs() < 1e-11);
    }

    #[test]
    fn delta_t_is_continuous_enough_and_positive_now() {
        let now = delta_t_seconds_approx(2_460_000.5);
        assert!((60.0..80.0).contains(&now), "{now}");
        let before = delta_t_seconds_approx(DJ00 - 0.5);
        let after = delta_t_seconds_approx(DJ00 + 0.5);
        assert!((before - after).abs() < 0.01);
    }
}
