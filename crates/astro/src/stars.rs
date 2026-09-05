//! The star table (`docs/03-design/astro-star-table.md`): where a
//! catalogued star, the galactic centre or a galactic pole stands at a
//! date, from its ICRS astrometry. The catalogue (`teistro_core::catalogue::Star`)
//! carries the SIMBAD astrometry of every member at epoch J2000.0 with its
//! source; this module carries a direction to the equator and ecliptic of
//! date: proper motion and parallax, the light deflection by the Sun, the
//! aberration from the Earth's barycentric velocity (the SDK's own Earth
//! ephemeris, so a star's place is the same over every provider), the frame
//! bias, precession under the chosen model and, when asked, nutation. It
//! is what the star-anchored ayanamshas stand on.
//!
//! ```
//! use teistro_astro::stars::{Options, place_of};
//! use teistro_core::catalogue::Star;
//! use teistro_core::quantity::{JulianDay, Tt};
//!
//! // Spica's apparent place at J2000.0: 23°50′ into Libra, two degrees
//! // south of the ecliptic.
//! let spica = place_of(Star::Spica, JulianDay::<Tt>::J2000, &Options::APPARENT).expect("a place");
//! assert!((spica.lon_deg - 203.84).abs() < 0.01, "{}", spica.lon_deg);
//! assert!((spica.lat_deg + 2.05).abs() < 0.01, "{}", spica.lat_deg);
//! ```

use teistro_core::angle::normalise_deg;
use teistro_core::catalogue::{Nakshatra, Star};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Tt};

use crate::iau::apparent::{ab, ldsun, numat, pmpx};
use crate::iau::epv00::epv00;
use crate::iau::p06::bp06;
use crate::iau::vector::{c2s, pm, pn, rxp};
use crate::iau::{AULT, DAYSEC, DEG2RAD, DJM00, DJM0, DJY, DMAS2R, RAD2DEG, nut00b};
use crate::precession::{self, PrecessionModel};

/// The ICRS astrometry of a star: its direction at an epoch, its space
/// motion and its distance, in the units the catalogues publish.
///
/// ```
/// use teistro_astro::stars::Astrometry;
/// use teistro_core::catalogue::Star;
///
/// // From the catalogue, or built for a star the catalogue does not carry.
/// let spica = Astrometry::of(Star::Spica);
/// assert!((spica.parallax_mas - 13.06).abs() < 1e-9);
/// let custom = Astrometry::icrs(201.298, -11.161).with_proper_motion(-42.35, -30.67).with_parallax(13.06);
/// assert_eq!(custom.radial_velocity_km_s, 0.0);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Astrometry {
    /// Right ascension at the epoch, degrees.
    pub ra_deg: f64,
    /// Declination at the epoch, degrees.
    pub dec_deg: f64,
    /// Proper motion in right ascension, the great-circle rate (already
    /// multiplied by the cosine of the declination), milliarcseconds a year.
    pub pm_ra_mas_yr: f64,
    /// Proper motion in declination, milliarcseconds a year.
    pub pm_dec_mas_yr: f64,
    /// Annual parallax, milliarcseconds; zero for a star without one, which
    /// is then placed at infinity.
    pub parallax_mas: f64,
    /// Radial velocity, km/s, positive receding.
    pub radial_velocity_km_s: f64,
    /// The epoch of the position (the catalogue's rows are J2000.0).
    pub epoch: JulianDay<Tt>,
}

impl Astrometry {
    /// A fixed direction at J2000.0 with no motion, parallax or velocity.
    #[must_use]
    pub const fn icrs(ra_deg: f64, dec_deg: f64) -> Astrometry {
        Astrometry {
            ra_deg,
            dec_deg,
            pm_ra_mas_yr: 0.0,
            pm_dec_mas_yr: 0.0,
            parallax_mas: 0.0,
            radial_velocity_km_s: 0.0,
            epoch: JulianDay::J2000,
        }
    }

    /// The catalogue's astrometry of a member, at J2000.0.
    #[must_use]
    pub fn of(star: Star) -> Astrometry {
        let a = &star.attributes().astrometry;
        Astrometry {
            ra_deg: a.ra_deg,
            dec_deg: a.dec_deg,
            pm_ra_mas_yr: a.pm_ra_mas_yr,
            pm_dec_mas_yr: a.pm_dec_mas_yr,
            parallax_mas: a.parallax_mas,
            radial_velocity_km_s: a.radial_velocity_km_s,
            epoch: JulianDay::J2000,
        }
    }

    /// With a proper motion (the right ascension rate with cos δ applied),
    /// milliarcseconds a year.
    #[must_use]
    pub const fn with_proper_motion(mut self, pm_ra_mas_yr: f64, pm_dec_mas_yr: f64) -> Astrometry {
        self.pm_ra_mas_yr = pm_ra_mas_yr;
        self.pm_dec_mas_yr = pm_dec_mas_yr;
        self
    }

    /// With a parallax, milliarcseconds.
    #[must_use]
    pub const fn with_parallax(mut self, parallax_mas: f64) -> Astrometry {
        self.parallax_mas = parallax_mas;
        self
    }

    /// With a radial velocity, km/s, positive receding.
    #[must_use]
    pub const fn with_radial_velocity(mut self, km_s: f64) -> Astrometry {
        self.radial_velocity_km_s = km_s;
        self
    }

    /// Stated at another epoch (Gaia's rows are J2016.0).
    #[must_use]
    pub const fn at_epoch(mut self, epoch: JulianDay<Tt>) -> Astrometry {
        self.epoch = epoch;
        self
    }

    fn check(&self) -> Result<(), Error> {
        for (name, value) in [
            ("ra_deg", self.ra_deg),
            ("dec_deg", self.dec_deg),
            ("pm_ra_mas_yr", self.pm_ra_mas_yr),
            ("pm_dec_mas_yr", self.pm_dec_mas_yr),
            ("parallax_mas", self.parallax_mas),
            ("radial_velocity_km_s", self.radial_velocity_km_s),
        ] {
            if !value.is_finite() {
                return Err(Error::invalid_arg(format!(
                    "a star's {name} must be a finite number, not {value}"
                ))
                .with_field(name));
            }
        }
        if self.dec_deg.abs() >= 90.0 {
            return Err(Error::invalid_arg(format!(
                "a star's declination must lie inside ±90°, not {}",
                self.dec_deg
            ))
            .with_field("dec_deg")
            .with_hint("a star at a pole has no right ascension rate"));
        }
        if self.parallax_mas < 0.0 {
            return Err(Error::invalid_arg(format!(
                "a star's parallax cannot be negative ({})",
                self.parallax_mas
            ))
            .with_field("parallax_mas")
            .with_hint("give zero for a star without a measured parallax"));
        }
        Ok(())
    }
}

impl From<Star> for Astrometry {
    fn from(star: Star) -> Astrometry {
        Astrometry::of(star)
    }
}

/// Which effects a place carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "four independent switches, each meaningful on its own, as the port's corrections"
)]
pub struct Corrections {
    /// The true equator and equinox of date rather than the mean.
    pub nutation: bool,
    /// The aberration from the Earth's barycentric velocity.
    pub aberration: bool,
    /// The light deflection by the Sun.
    pub deflection: bool,
    /// The annual parallax from the Earth's barycentric position.
    pub parallax: bool,
}

impl Corrections {
    /// Everything: the apparent place on the true equator of date.
    pub const APPARENT: Corrections = Corrections {
        nutation: true,
        aberration: true,
        deflection: true,
        parallax: true,
    };

    /// The apparent place on the mean equator of date (what a mean
    /// ayanamsha reads).
    pub const MEAN: Corrections = Corrections {
        nutation: false,
        ..Corrections::APPARENT
    };

    /// The direction itself on the mean equator of date: proper motion and
    /// precession only.
    pub const GEOMETRIC: Corrections = Corrections {
        nutation: false,
        aberration: false,
        deflection: false,
        parallax: false,
    };
}

/// How a place is computed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Options {
    /// The precession model carrying the direction to the date.
    pub precession: PrecessionModel,
    /// The corrections applied.
    pub corrections: Corrections,
}

impl Options {
    /// The apparent place under the default precession model.
    pub const APPARENT: Options = Options {
        precession: PrecessionModel::Vondrak2011,
        corrections: Corrections::APPARENT,
    };

    /// The apparent place on the mean equator of date.
    pub const MEAN: Options = Options {
        corrections: Corrections::MEAN,
        ..Options::APPARENT
    };

    /// The geometric direction on the mean equator of date.
    pub const GEOMETRIC: Options = Options {
        corrections: Corrections::GEOMETRIC,
        ..Options::APPARENT
    };

    /// The same corrections under another precession model.
    #[must_use]
    pub const fn with_precession(mut self, model: PrecessionModel) -> Options {
        self.precession = model;
        self
    }
}

impl Default for Options {
    fn default() -> Options {
        Options::APPARENT
    }
}

/// A place on the equator and ecliptic of date, degrees.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Place {
    /// Ecliptic longitude, `[0, 360)`.
    pub lon_deg: f64,
    /// Ecliptic latitude.
    pub lat_deg: f64,
    /// Right ascension, `[0, 360)`.
    pub ra_deg: f64,
    /// Declination.
    pub dec_deg: f64,
}

/// A catalogued member's place at a TT instant.
///
/// # Errors
///
/// As [`place`].
pub fn place_of(star: Star, tt: JulianDay<Tt>, options: &Options) -> Result<Place, Error> {
    place(&Astrometry::of(star), tt, options)
}

/// The place of a direction with the given astrometry at a TT instant.
///
/// The catalogue direction is carried to the date by its space motion,
/// seen from the Earth's centre for the parallax, bent by the Sun,
/// aberrated by the Earth's barycentric velocity (both from the SDK's own
/// Earth ephemeris), rotated from the ICRS by the frame bias, precessed
/// under the model, nutated when asked, and read on the equator and the
/// ecliptic of date with the obliquity the model is consistent with.
///
/// # Errors
///
/// `INVALID_ARG` naming the field for a non-finite parameter, a
/// declination at a pole or a negative parallax.
pub fn place(
    astrometry: &Astrometry,
    tt: JulianDay<Tt>,
    options: &Options,
) -> Result<Place, Error> {
    astrometry.check()?;
    let corrections = options.corrections;
    let (date1, date2) = tt.split();
    let earth = epv00(date1, date2);
    // Proper motion and parallax, seen from the Earth's centre.
    let observer = if corrections.parallax {
        earth.barycentric.position
    } else {
        [0.0; 3]
    };
    let years = ((date1 - astrometry.epoch.get()) + date2) / DJY;
    let dec = astrometry.dec_deg * DEG2RAD;
    let ra_rate = astrometry.pm_ra_mas_yr * DMAS2R / dec.cos();
    let mut direction = pmpx(
        astrometry.ra_deg * DEG2RAD,
        dec,
        ra_rate,
        astrometry.pm_dec_mas_yr * DMAS2R,
        astrometry.parallax_mas * 1e-3,
        astrometry.radial_velocity_km_s,
        years,
        &observer,
    );
    // The Sun's deflection and the aberration, both wanting the Sun to
    // Earth distance.
    let (sun_distance, from_sun) = pn(&earth.heliocentric.position);
    if corrections.deflection {
        direction = ldsun(&direction, &from_sun, sun_distance);
    }
    if corrections.aberration {
        let velocity = earth
            .barycentric
            .velocity
            .map(|component| component * AULT / DAYSEC);
        let speed = pm(&velocity);
        direction = ab(
            &direction,
            &velocity,
            sun_distance,
            (1.0 - speed * speed).sqrt(),
        );
    }
    // The frame bias to the J2000.0 mean equator, then precession to the
    // mean equator of date, then nutation when asked.
    let (bias, _, _) = bp06(DJM0, DJM00);
    let mut equatorial = precession::to_date(options.precession, tt, rxp(&bias, &direction));
    let mut obliquity = precession::mean_obliquity_rad(options.precession, tt);
    if corrections.nutation {
        let nutation = nut00b(date1, date2);
        equatorial = rxp(&numat(obliquity, nutation.dpsi, nutation.deps), &equatorial);
        obliquity += nutation.deps;
    }
    let (ra, declination) = c2s(&equatorial);
    let (lon, lat) = c2s(&precession::equatorial_to_ecliptic(equatorial, obliquity));
    Ok(Place {
        lon_deg: normalise_deg(lon * RAD2DEG),
        lat_deg: lat * RAD2DEG,
        ra_deg: normalise_deg(ra * RAD2DEG),
        dec_deg: declination * RAD2DEG,
    })
}

/// The yogatara, the junction star, of a nakshatra as the catalogue
/// identifies it.
///
/// ```
/// use teistro_astro::stars::yogatara;
/// use teistro_core::catalogue::{Nakshatra, Star};
///
/// assert_eq!(yogatara(Nakshatra::Chitra), Some(Star::Spica));
/// assert_eq!(yogatara(Nakshatra::Rohini), Some(Star::Aldebaran));
/// ```
#[must_use]
pub fn yogatara(nakshatra: Nakshatra) -> Option<Star> {
    Star::ALL
        .into_iter()
        .find(|star| star.attributes().yogatara_of == Some(nakshatra))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use teistro_core::angle::difference_deg;

    use super::*;

    const J2000: JulianDay<Tt> = JulianDay::J2000;

    /// At J2000.0 without corrections, a star's place is its ICRS direction
    /// turned by the frame bias alone: a few hundredths of an arcsecond.
    #[test]
    fn the_geometric_place_at_j2000_is_the_catalogue_direction() {
        let spica = Astrometry::of(Star::Spica);
        let place = super::place(&spica, J2000, &Options::GEOMETRIC).unwrap();
        assert!(
            difference_deg(place.ra_deg, spica.ra_deg).abs() * 3600.0 < 0.03,
            "{place:?}"
        );
        assert!(
            (place.dec_deg - spica.dec_deg).abs() * 3600.0 < 0.03,
            "{place:?}"
        );
        assert!((place.lon_deg - 203.84).abs() < 0.01, "{}", place.lon_deg);
    }

    /// The aberration moves a place by up to twenty arcseconds and the
    /// deflection by a fraction of one; nutation by up to seventeen.
    #[test]
    fn the_corrections_are_the_size_the_textbooks_give() {
        let spica = Astrometry::of(Star::Spica);
        let geometric = place(&spica, J2000, &Options::GEOMETRIC).unwrap();
        let mean = place(&spica, J2000, &Options::MEAN).unwrap();
        let apparent = place(&spica, J2000, &Options::APPARENT).unwrap();
        let aberration = difference_deg(mean.lon_deg, geometric.lon_deg).abs() * 3600.0;
        assert!((1.0..=21.0).contains(&aberration), "{aberration}");
        let nutation = difference_deg(apparent.lon_deg, mean.lon_deg).abs() * 3600.0;
        assert!((1.0..=18.0).contains(&nutation), "{nutation}");
        // A place nutated but not aberrated is neither of the two above.
        let only_nutation = Options {
            corrections: Corrections {
                nutation: true,
                ..Corrections::GEOMETRIC
            },
            ..Options::APPARENT
        };
        let nutated = place(&spica, J2000, &only_nutation).unwrap();
        assert!(difference_deg(nutated.lon_deg, geometric.lon_deg).abs() * 3600.0 > 1.0);
    }

    /// Precession carries a place forward by the general precession, about
    /// 50.3″ a year, and proper motion adds the star's own drift.
    #[test]
    fn precession_and_proper_motion_move_a_place_over_time() {
        let later = JulianDay::<Tt>::literal(J2000.get() + 100.0 * DJY);
        let spica = place_of(Star::Spica, later, &Options::GEOMETRIC).unwrap();
        let spica_now = place_of(Star::Spica, J2000, &Options::GEOMETRIC).unwrap();
        // Arcseconds a year: the general precession, 50.29″, less Spica's own
        // slow westward motion.
        let advance = difference_deg(spica.lon_deg, spica_now.lon_deg) * 36.0;
        assert!((50.1..=50.4).contains(&advance), "{advance}″ a year");
        // Arcturus runs 2.28″ a year across the sky.
        let fixed = Astrometry::icrs(
            Astrometry::of(Star::Arcturus).ra_deg,
            Astrometry::of(Star::Arcturus).dec_deg,
        );
        let moving = place(&Astrometry::of(Star::Arcturus), later, &Options::GEOMETRIC).unwrap();
        let still = place(&fixed, later, &Options::GEOMETRIC).unwrap();
        let drift = ((difference_deg(moving.ra_deg, still.ra_deg)
            * moving.dec_deg.to_radians().cos())
        .powi(2)
            + (moving.dec_deg - still.dec_deg).powi(2))
        .sqrt()
            * 3600.0;
        assert!((225.0..=232.0).contains(&drift), "{drift}″ a century");
    }

    /// The nearest star shows the largest parallax: about three quarters of
    /// an arcsecond of shift between the equinoxes.
    #[test]
    fn parallax_shifts_the_nearest_star() {
        let alpha_cen = Astrometry::of(Star::RigilKentaurus);
        let with = Options {
            corrections: Corrections {
                parallax: true,
                ..Corrections::GEOMETRIC
            },
            ..Options::APPARENT
        };
        let march = JulianDay::<Tt>::literal(2_451_623.5);
        let with_parallax = place(&alpha_cen, march, &with).unwrap();
        let without = place(&alpha_cen, march, &Options::GEOMETRIC).unwrap();
        let shift = ((difference_deg(with_parallax.ra_deg, without.ra_deg)
            * without.dec_deg.to_radians().cos())
        .powi(2)
            + (with_parallax.dec_deg - without.dec_deg).powi(2))
        .sqrt()
            * 3600.0;
        assert!((0.3..=0.75).contains(&shift), "{shift}″");
    }

    #[test]
    fn every_nakshatra_has_a_yogatara_and_bad_astrometry_is_refused_by_name() {
        for nakshatra in Nakshatra::ALL {
            assert!(yogatara(nakshatra).is_some(), "{}", nakshatra.key());
        }
        assert_eq!(yogatara(Nakshatra::Revati), Some(Star::Revati));
        let pole = Astrometry::icrs(0.0, 90.0);
        let err = place(&pole, J2000, &Options::APPARENT).unwrap_err();
        assert_eq!(err.field(), Some("dec_deg"));
        let nan = Astrometry::icrs(f64::NAN, 0.0);
        assert_eq!(
            place(&nan, J2000, &Options::APPARENT).unwrap_err().field(),
            Some("ra_deg")
        );
        let negative = Astrometry::icrs(0.0, 0.0).with_parallax(-1.0);
        assert_eq!(
            place(&negative, J2000, &Options::APPARENT)
                .unwrap_err()
                .field(),
            Some("parallax_mas")
        );
    }
}
