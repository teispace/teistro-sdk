//! The progressed angles: how far the meridian has turned (C237).
//!
//! Every method moves the birth's meridian by the Sun's motion over the
//! progression's span of sky, mean or true, along the equator or along the
//! ecliptic; the ascendant is then the one the turned meridian rises with
//! at the birthplace. The quotidian is the chart at the instant itself.

use teistro_astro::houses::{Obliquity, circle_point};
use teistro_astro::sky::{Spherical, ecliptic_to_equatorial};
use teistro_core::error::Error;

use crate::progression::TROPICAL_YEAR_DAYS;

/// How the progressed midheaven moves (C237).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AngleMethod {
    /// The mean Sun's motion in right ascension: the chart cast for the
    /// birth's clock time on the progressed day, Leo's own map (p. 35),
    /// whose sidereal time is the birth's plus 3m 56.555s a day.
    #[default]
    NaibodRightAscension,
    /// The mean Sun's motion along the ecliptic, added to the midheaven's
    /// longitude.
    NaibodLongitude,
    /// The true Sun's motion along the ecliptic, added to the midheaven's
    /// longitude: the solar arc.
    SolarArcLongitude,
    /// The true Sun's motion in right ascension, added to the meridian.
    SolarArcRightAscension,
    /// The chart at the progressed instant, whose meridian turns a full
    /// circle and a degree for each day of sky (Leo's figure A′, p. 296).
    Quotidian,
}

/// The Sun at one instant: tropical longitude and the obliquity of date,
/// degrees.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SunAt {
    /// Tropical ecliptic longitude.
    pub longitude_deg: f64,
    /// The true obliquity of the ecliptic at that instant.
    pub obliquity_deg: f64,
}

impl SunAt {
    fn right_ascension_deg(self) -> f64 {
        ecliptic_to_equatorial(
            Spherical {
                lon_deg: self.longitude_deg,
                lat_deg: 0.0,
            },
            self.obliquity_deg,
        )
        .lon_deg
    }
}

/// What the progressed meridian is turned from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Meridian {
    /// The birth's right ascension of the midheaven, degrees.
    pub armc_deg: f64,
    /// The birth's obliquity, degrees: the midheaven's longitude is read
    /// with it.
    pub obliquity_deg: f64,
}

impl Meridian {
    /// The birth's midheaven, tropical longitude, degrees: the point of
    /// the ecliptic whose right ascension is the meridian's.
    fn midheaven_deg(self) -> f64 {
        circle_point(self.armc_deg, 0.0, &Obliquity::new(self.obliquity_deg))
    }
}

/// The right ascension of the progressed midheaven, degrees in `[0, 360)`,
/// by `method`, or `None` for [`AngleMethod::Quotidian`], whose meridian is
/// the progressed chart's own.
///
/// `sky_days` is the span of sky from birth to the progressed instant, and
/// `birth_sun` and `progressed_sun` the Sun at its two ends; the latter's
/// obliquity is the one the progressed angles are built on.
///
/// ```
/// use teistro_western::{AngleMethod, Meridian, SunAt, progressed_armc};
///
/// // Leo's progressed map (p. 35): 2h 52m 54s at birth, forty-six days on.
/// let birth = Meridian { armc_deg: 43.225, obliquity_deg: 23.46 };
/// let sun = SunAt { longitude_deg: 0.0, obliquity_deg: 23.46 };
/// let armc = progressed_armc(AngleMethod::NaibodRightAscension, birth, 46.0, sun, sun)?.unwrap();
/// // He casts it at 5h 54m 16s.
/// assert!((armc / 15.0 - (5.0 + 54.0 / 60.0 + 16.0 / 3600.0)).abs() < 1.0 / 3600.0);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// A value that is not finite, named by the field it came in.
pub fn progressed_armc(
    method: AngleMethod,
    birth: Meridian,
    sky_days: f64,
    birth_sun: SunAt,
    progressed_sun: SunAt,
) -> Result<Option<f64>, Error> {
    for (value, field) in [
        (birth.armc_deg, "armc_deg"),
        (birth.obliquity_deg, "obliquity_deg"),
        (sky_days, "sky_days"),
        (birth_sun.longitude_deg, "birth_sun"),
        (birth_sun.obliquity_deg, "birth_sun"),
        (progressed_sun.longitude_deg, "progressed_sun"),
        (progressed_sun.obliquity_deg, "progressed_sun"),
    ] {
        if !value.is_finite() {
            return Err(Error::invalid_arg(format!("{value} is not finite")).with_field(field));
        }
    }
    let mean_arc = sky_days * 360.0 / TROPICAL_YEAR_DAYS;
    let solar_arc = progressed_sun.longitude_deg - birth_sun.longitude_deg;
    // The moved midheaven goes back to the equator with the progressed
    // obliquity, the one the angles are built on, so the midheaven they
    // read off it is the birth's plus the arc exactly.
    let by_longitude = |arc: f64| {
        ecliptic_to_equatorial(
            Spherical {
                lon_deg: birth.midheaven_deg() + arc,
                lat_deg: 0.0,
            },
            progressed_sun.obliquity_deg,
        )
        .lon_deg
    };
    let armc = match method {
        AngleMethod::NaibodRightAscension => birth.armc_deg + mean_arc,
        AngleMethod::NaibodLongitude => by_longitude(mean_arc),
        AngleMethod::SolarArcLongitude => by_longitude(solar_arc),
        AngleMethod::SolarArcRightAscension => {
            birth.armc_deg + progressed_sun.right_ascension_deg() - birth_sun.right_ascension_deg()
        }
        AngleMethod::Quotidian => return Ok(None),
    };
    Ok(Some(armc.rem_euclid(360.0)))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    const EPS: f64 = 23.4393;

    fn armc(method: AngleMethod, from: f64, days: f64, sun: (f64, f64)) -> f64 {
        let at = |longitude_deg| SunAt {
            longitude_deg,
            obliquity_deg: EPS,
        };
        progressed_armc(
            method,
            Meridian {
                armc_deg: from,
                obliquity_deg: EPS,
            },
            days,
            at(sun.0),
            at(sun.1),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn at_an_equinox_and_a_solstice_ecliptic_and_equator_agree() {
        // A midheaven at 0° Aries moved 90° along the ecliptic and along
        // the equator ends at the same place, the solstice's.
        let along = armc(AngleMethod::SolarArcLongitude, 0.0, 91.0, (0.0, 90.0));
        let across = armc(AngleMethod::SolarArcRightAscension, 0.0, 91.0, (0.0, 90.0));
        assert!((along - 90.0).abs() < 1e-9 && (across - 90.0).abs() < 1e-9);
        // Between them they part: 30° of longitude from the equinox is less
        // than 30° of right ascension.
        let along = armc(AngleMethod::SolarArcLongitude, 0.0, 30.0, (0.0, 30.0));
        assert!((along - 27.9105).abs() < 1e-3, "{along}");
    }

    #[test]
    fn the_longitude_methods_move_the_midheaven_itself() {
        // From a meridian at 40° of right ascension the midheaven moves
        // along the ecliptic by exactly the arc.
        let midheaven = |armc: f64| circle_point(armc, 0.0, &Obliquity::new(EPS));
        let from = 40.0;
        let turned = armc(AngleMethod::SolarArcLongitude, from, 30.0, (100.0, 129.0));
        assert!((midheaven(turned) - midheaven(from) - 29.0).abs() < 1e-9);
        let turned = armc(AngleMethod::NaibodLongitude, from, 30.0, (0.0, 0.0));
        let mean = 30.0 * 360.0 / TROPICAL_YEAR_DAYS;
        assert!((midheaven(turned) - midheaven(from) - mean).abs() < 1e-9);
    }

    #[test]
    fn the_mean_methods_move_a_degree_a_year_and_the_solar_arc_the_suns() {
        let naibod = armc(AngleMethod::NaibodRightAscension, 350.0, 20.0, (0.0, 0.0));
        assert!((naibod - (350.0 + 20.0 * 360.0 / TROPICAL_YEAR_DAYS - 360.0)).abs() < 1e-9);
        // The solar arc in right ascension is the Sun's own: from 10° to 40°
        // of longitude the Sun's right ascension moves 29.18°.
        let solar = armc(
            AngleMethod::SolarArcRightAscension,
            100.0,
            30.0,
            (10.0, 40.0),
        );
        let ra = |lon: f64| {
            ecliptic_to_equatorial(
                Spherical {
                    lon_deg: lon,
                    lat_deg: 0.0,
                },
                EPS,
            )
            .lon_deg
        };
        assert!((solar - (100.0 + ra(40.0) - ra(10.0))).abs() < 1e-9);
    }

    #[test]
    fn the_quotidian_is_the_charts_own() {
        let sun = SunAt {
            longitude_deg: 0.0,
            obliquity_deg: EPS,
        };
        let meridian = Meridian {
            armc_deg: 10.0,
            obliquity_deg: EPS,
        };
        assert_eq!(
            progressed_armc(AngleMethod::Quotidian, meridian, 5.0, sun, sun).unwrap(),
            None
        );
    }

    #[test]
    fn a_value_that_is_not_finite_is_named() {
        let sun = SunAt {
            longitude_deg: f64::NAN,
            obliquity_deg: EPS,
        };
        let meridian = Meridian {
            armc_deg: 10.0,
            obliquity_deg: EPS,
        };
        let error =
            progressed_armc(AngleMethod::SolarArcLongitude, meridian, 5.0, sun, sun).unwrap_err();
        assert_eq!(error.field(), Some("birth_sun"));
    }
}
