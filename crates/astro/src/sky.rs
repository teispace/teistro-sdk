//! The sky above raw positions: the obliquity record the SDK computes
//! itself, the rotation between the ecliptic and the equator, apparent
//! sidereal time at a place, and the apparent equatorial position of a
//! body that the rise and set solver reads.

use teistro_core::angle::normalise_deg;
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Longitude, Tt, Ut1};
use teistro_port_ephemeris::{Body, Obliquity};

use crate::iau::{self, DEG2RAD, RAD2DEG};

/// A spherical position: longitude or right ascension and latitude or
/// declination, degrees.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spherical {
    /// Longitude or right ascension, degrees.
    pub lon_deg: f64,
    /// Latitude or declination, degrees.
    pub lat_deg: f64,
}

/// The SDK's own obliquity and nutation at a TT instant, degrees: the
/// IAU 2006 mean obliquity and the IAU 2000B nutation.
///
/// ```
/// use teistro_astro::sky::obliquity;
/// use teistro_core::quantity::{JulianDay, Tt};
///
/// let at = JulianDay::<Tt>::literal(2_451_545.0);
/// let eps = obliquity(at);
/// assert!((eps.mean_deg - 23.439_279_444).abs() < 1e-8);
/// assert!(eps.nutation_lon_deg.abs() < 0.01);
/// ```
#[must_use]
pub fn obliquity(tt: JulianDay<Tt>) -> Obliquity {
    let (date1, date2) = tt.split();
    let mean = iau::obl06(date1, date2);
    let nutation = iau::nut00b(date1, date2);
    Obliquity {
        mean_deg: mean * RAD2DEG,
        true_deg: (mean + nutation.deps) * RAD2DEG,
        nutation_lon_deg: nutation.dpsi * RAD2DEG,
        nutation_obl_deg: nutation.deps * RAD2DEG,
    }
}

/// Greenwich apparent sidereal time, degrees, from the UT1 instant (the
/// Earth's rotation) and the TT instant (the precession and nutation):
/// the IAU 2000 mean sidereal time plus the equation of the equinoxes
/// with the IAU 2000B nutation, as ERFA's `gst00b` computes with the two
/// scales distinguished.
#[must_use]
pub fn greenwich_sidereal_time_deg(ut1: JulianDay<Ut1>, tt: JulianDay<Tt>) -> f64 {
    let (uta, utb) = ut1.split();
    let (tta, ttb) = tt.split();
    let gast = iau::anp(iau::gmst00(uta, utb, tta, ttb) + iau::ee00b(tta, ttb));
    normalise_deg(gast * RAD2DEG)
}

/// Local apparent sidereal time at a longitude, degrees.
///
/// ```
/// use teistro_astro::sky::sidereal_time_deg;
/// use teistro_core::quantity::{JulianDay, Longitude, Tt, Ut1};
///
/// // ERFA's reference instant, 2006-01-15 21:24:37.5 UTC as UT1 and TT alike.
/// let ut1 = JulianDay::<Ut1>::literal(2_400_000.5 + 53_736.0);
/// let tt = JulianDay::<Tt>::literal(2_400_000.5 + 53_736.0);
/// let greenwich = sidereal_time_deg(ut1, tt, Longitude::literal(0.0));
/// assert!((greenwich - 1.754_166_136_510_680_589f64.to_degrees()).abs() < 1e-9);
/// ```
#[must_use]
pub fn sidereal_time_deg(ut1: JulianDay<Ut1>, tt: JulianDay<Tt>, longitude: Longitude) -> f64 {
    normalise_deg(greenwich_sidereal_time_deg(ut1, tt) + longitude.get())
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

/// The apparent geocentric equatorial position of a body: what an
/// observer's horizon is reckoned against.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
pub struct Apparent {
    /// Right ascension, degrees, of date.
    pub ra_deg: f64,
    /// Declination, degrees, of date.
    pub dec_deg: f64,
    /// Distance from the Earth's centre, astronomical units.
    pub distance_au: f64,
}

/// A source of apparent geocentric equatorial positions of date: the
/// frame completion over a provider, or a classical model.
pub trait ApparentPositions: Send + Sync {
    /// The body's apparent position at a UT1 instant.
    ///
    /// # Errors
    ///
    /// An instant the source cannot answer for.
    fn apparent(&self, body: Body, ut1: JulianDay<Ut1>) -> Result<Apparent, Error>;

    /// The source's name for provenance stamps.
    fn describe(&self) -> String;
}

impl<S: ApparentPositions + ?Sized> ApparentPositions for &S {
    fn apparent(&self, body: Body, ut1: JulianDay<Ut1>) -> Result<Apparent, Error> {
        (**self).apparent(body, ut1)
    }

    fn describe(&self) -> String {
        (**self).describe()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use teistro_core::angle::difference_deg;

    use super::*;

    #[test]
    fn rotation_round_trips_and_keeps_the_ecliptic_pole() {
        let eps = obliquity(JulianDay::literal(2_460_000.5)).true_deg;
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
                difference_deg(back.lon_deg, lon).abs() < 1e-11,
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
    fn sidereal_time_advances_a_degree_in_four_minutes() {
        let ut1 = JulianDay::<Ut1>::literal(2_451_545.0);
        let tt = JulianDay::<Tt>::literal(2_451_545.0 + 64.184 / 86_400.0);
        let now = sidereal_time_deg(ut1, tt, Longitude::literal(85.324));
        let later = sidereal_time_deg(
            ut1.plus_days(4.0 / 1440.0).unwrap(),
            tt.plus_days(4.0 / 1440.0).unwrap(),
            Longitude::literal(85.324),
        );
        let advance = difference_deg(later, now);
        assert!((advance - 1.002_738).abs() < 1e-4, "{advance}");
        // Greenwich at J2000.0: GMST 280.46°, the textbook value.
        let greenwich = greenwich_sidereal_time_deg(ut1, tt);
        assert!((greenwich - 280.46).abs() < 0.01, "{greenwich}");
    }
}
