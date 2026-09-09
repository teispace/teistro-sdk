//! The sky above raw positions: the obliquity record the SDK computes
//! itself, the rotation between the ecliptic and the equator, apparent
//! sidereal time at a place, and the apparent equatorial position of a
//! body that the rise and set solver reads.

use teistro_core::angle::{difference_deg, normalise_deg};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Longitude, Place, Tt, Ut1};
use teistro_port_ephemeris::{Body, Obliquity};

use crate::delta_t::{DeltaTModel, delta_t};
use crate::iau::vector::Vector3;
use crate::iau::{self, DEG2RAD, RAD2DEG};
use crate::scale::tt_from_ut1;

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
/// the IAU 2006 mean sidereal time plus the equation of the equinoxes
/// with the IAU 2006 mean obliquity and the IAU 2000B nutation, the
/// equinox-based expression of IERS Conventions 2010 (ERFA's `gst06a`
/// differs by the IAU 2000A nutation, under a milliarcsecond), so the
/// meridian agrees with the precession the frames are built with.
#[must_use]
pub fn greenwich_sidereal_time_deg(ut1: JulianDay<Ut1>, tt: JulianDay<Tt>) -> f64 {
    let (uta, utb) = ut1.split();
    let (tta, ttb) = tt.split();
    normalise_deg(iau::gst06b(uta, utb, tta, ttb) * RAD2DEG)
}

/// The local mean midnight that begins the local mean day an instant
/// falls in, at a longitude: the instant whose local mean time (UT1 plus
/// four minutes a degree of longitude) is 0h. The day the panchanga, the
/// rise and set solver's `day` and the visibility scan all reckon by.
///
/// ```
/// use teistro_astro::sky::local_mean_midnight;
/// use teistro_core::quantity::{JulianDay, Longitude, Ut1};
///
/// // 2024-06-21 12:00 UT at Kathmandu (85.324° E): the local day began
/// // at 2024-06-20 18:18:42 UT.
/// let noon = JulianDay::<Ut1>::literal(2_460_483.0);
/// let midnight = local_mean_midnight(noon, Longitude::literal(85.324));
/// assert!((midnight.get() - (2_460_482.5 - 85.324 / 360.0)).abs() < 1e-9);
/// ```
#[must_use]
pub fn local_mean_midnight(at: JulianDay<Ut1>, longitude: Longitude) -> JulianDay<Ut1> {
    let offset = longitude.get() / 360.0;
    let local = at.get() + offset;
    let midnight_local = (local - 0.5).floor() + 0.5;
    JulianDay::try_new(midnight_local - offset).unwrap_or(at)
}

/// Local apparent sidereal time at a longitude, degrees.
///
/// ```
/// use teistro_astro::sky::sidereal_time_deg;
/// use teistro_core::quantity::{JulianDay, Longitude, Place, Tt, Ut1};
///
/// // ERFA's reference instant, 2006-01-15 21:24:37.5 UTC as UT1 and TT alike.
/// let ut1 = JulianDay::<Ut1>::literal(2_400_000.5 + 53_736.0);
/// let tt = JulianDay::<Tt>::literal(2_400_000.5 + 53_736.0);
/// let greenwich = sidereal_time_deg(ut1, tt, Longitude::literal(0.0));
/// // ERFA's `gst06a` at that instant, which the IAU 2000A nutation
/// // separates from the SDK's 2000B by 0.3 milliarcseconds.
/// assert!((greenwich - 1.754_166_137_675_019_159f64.to_degrees()).abs() < 3e-7);
/// ```
#[must_use]
pub fn sidereal_time_deg(ut1: JulianDay<Ut1>, tt: JulianDay<Tt>, longitude: Longitude) -> f64 {
    normalise_deg(greenwich_sidereal_time_deg(ut1, tt) + longitude.get())
}

/// Where an observer stands and how fast the Earth's rotation carries
/// them, in the frame the sky's own positions are in: the true equator
/// and equinox of date, astronomical units and astronomical units a day,
/// measured from the centre of the Earth.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Observer {
    /// The station's geocentric position, astronomical units.
    pub position_au: Vector3,
    /// Its velocity, astronomical units a day.
    pub velocity_au_per_day: Vector3,
    /// Its acceleration, astronomical units a day squared. A station on
    /// a turning Earth is always accelerating towards the axis, and the
    /// aberration its velocity causes turns with it: two arcseconds a
    /// day, which is what a topocentric speed would be wrong by if this
    /// were left out.
    pub acceleration_au_per_day2: Vector3,
}

/// The observer at a place and an instant ([`Observer`]), from ERFA's
/// station routine on the WGS84 ellipsoid with Greenwich apparent
/// sidereal time for the rotation angle, so the answer is in the same
/// true equator and equinox of date as the positions it is subtracted
/// from.
///
/// The pole's coordinates and the TIO locator are passed as zero: the
/// SDK holds no Earth orientation parameters, and polar motion moves a
/// station by under fifteen metres, which turns the Moon's direction by
/// eight milliarcseconds and a planet's by less than a microarcsecond.
/// A consumer who has them can call [`iau::earth::pvtob`] with them.
///
/// ```
/// use teistro_astro::sky::observer;
/// use teistro_core::quantity::{JulianDay, Place, Tt, Ut1};
///
/// let place = Place::try_from_degrees(27.7172, 85.324, 1_400.0).expect("Kathmandu");
/// let at = observer(place, JulianDay::<Ut1>::J2000, JulianDay::<Tt>::literal(2_451_545.0));
/// // A station stands about an Earth radius out, 4.3e-5 astronomical units.
/// let radius = at.position_au.iter().map(|c| c * c).sum::<f64>().sqrt();
/// assert!((radius - 4.26e-5).abs() < 2e-7, "{radius}");
/// ```
#[must_use]
pub fn observer(place: Place, ut1: JulianDay<Ut1>, tt: JulianDay<Tt>) -> Observer {
    let theta = greenwich_sidereal_time_deg(ut1, tt) * DEG2RAD;
    let pv = iau::earth::pvtob(
        place.longitude.get() * DEG2RAD,
        place.latitude.get() * DEG2RAD,
        place.altitude.get(),
        0.0,
        0.0,
        0.0,
        theta,
    );
    let position_au = pv[0].map(|m| m / iau::DAU);
    let turn = iau::earth::ROTATION_RATE * iau::DAYSEC;
    Observer {
        position_au,
        velocity_au_per_day: pv[1].map(|m| m * iau::DAYSEC / iau::DAU),
        // The rotation is rigid, so the acceleration is the centripetal
        // one about the polar axis and needs no differencing.
        acceleration_au_per_day2: [
            -turn * turn * position_au[0],
            -turn * turn * position_au[1],
            0.0,
        ],
    }
}

/// Where the Earth is and how fast, as the aberration needs it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EarthAt {
    /// The Earth's barycentric velocity in the true equator and equinox
    /// of date, astronomical units a day: the velocity the annual
    /// aberration is computed from.
    pub velocity_au_per_day: Vector3,
    /// The Sun's distance from the Earth, astronomical units, which the
    /// aberration's own gravitational term is divided by.
    pub sun_distance_au: f64,
}

/// The Earth's barycentric velocity and the Sun's distance at an
/// instant.
///
/// `eraEpv00` answers in the celestial frame, so the IAU 2006
/// bias-precession matrix and the IAU 2000B nutation matrix carry it to
/// the frame the positions it is used with are in. It is the velocity
/// the annual aberration is computed from: the completion's topocentric
/// step takes that aberration off a provider's apparent direction before
/// displacing it and puts it back after, which is worth a third of an
/// arcsecond on the Moon and nothing on anything else
/// (`docs/03-design/topocentric-measured.md`, §2).
///
/// ```
/// use teistro_astro::sky::earth_at;
/// use teistro_core::quantity::{JulianDay, Tt};
///
/// let at = earth_at(JulianDay::<Tt>::literal(2_451_545.0));
/// // The Earth travels about 0.0172 astronomical units a day, and stood
/// // 0.983 astronomical units from the Sun at perihelion in 2000.
/// let speed = at.velocity_au_per_day.iter().map(|c| c * c).sum::<f64>().sqrt();
/// assert!((speed - 0.0172).abs() < 3e-4, "{speed}");
/// assert!((at.sun_distance_au - 0.9833).abs() < 1e-3, "{}", at.sun_distance_au);
/// ```
#[must_use]
pub fn earth_at(tt: JulianDay<Tt>) -> EarthAt {
    let (date1, date2) = tt.split();
    let state = iau::epv00::epv00(date1, date2);
    let nutation = iau::nut00b(date1, date2);
    let to_of_date = iau::vector::rxr(
        &iau::apparent::numat(iau::obl06(date1, date2), nutation.dpsi, nutation.deps),
        &iau::p06::pmat06(date1, date2),
    );
    EarthAt {
        velocity_au_per_day: iau::vector::rxp(&to_of_date, &state.barycentric.velocity),
        sun_distance_au: iau::vector::pm(&state.heliocentric.position),
    }
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

/// The equation of time at a UT1 instant, seconds: apparent solar time less
/// mean solar time, from the Greenwich apparent sidereal time and the Sun's
/// apparent right ascension in `sky`. Positive when the sundial is ahead
/// of the clock (about +16.4 minutes in early November, −14.2 in
/// mid-February).
///
/// ```
/// use teistro_astro::sky::equation_of_time_seconds;
/// use teistro_astro::{Completion, DeltaTModel};
/// use teistro_core::quantity::{JulianDay, Ut1};
/// use teistro_core::settings::OverridePolicy;
/// use teistro_port_ephemeris::TestProvider;
///
/// let provider = TestProvider::new();
/// let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// let e = equation_of_time_seconds(&sky, JulianDay::<Ut1>::J2000, DeltaTModel::TableThenModel).expect("the Sun");
/// assert!(e.abs() < 17.0 * 60.0);
/// ```
///
/// # Errors
///
/// The sky's error for the Sun, or a Delta T model that cannot answer.
pub fn equation_of_time_seconds(
    sky: &dyn ApparentPositions,
    ut1: JulianDay<Ut1>,
    delta_t_model: DeltaTModel,
) -> Result<f64, Error> {
    let tt = tt_from_ut1(ut1, &delta_t(ut1, delta_t_model)?);
    let sidereal = greenwich_sidereal_time_deg(ut1, tt);
    // The hour angle of the mean Sun at Greenwich: the time since midnight.
    let since_midnight_deg = (ut1.get() + 0.5).rem_euclid(1.0) * 360.0;
    let sun = sky.apparent(Body::Sun, ut1)?;
    // The true Sun's hour angle less the mean Sun's, folded to a half turn,
    // at four minutes a degree.
    let apart_deg = difference_deg(sidereal - sun.ra_deg, since_midnight_deg + 180.0);
    Ok(apart_deg * 240.0)
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
