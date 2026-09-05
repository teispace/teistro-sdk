//! The model: a graha's place at an instant, the tradition's trace of the
//! computation, and the text's precession, declination and ascensional
//! difference that give sunrise and sunset.

use core::fmt;

use teistro_core::catalogue::Graha;
use teistro_core::error::Error;
use teistro_core::quantity::{Degrees, JulianDay, Latitude, Ut1};

use crate::equation::{
    Epicycle, FourStep, four_step, manda_equation_deg, manda_motion_deg_per_day,
};
use crate::mean::{Ahargana, Cycle, Motion};
use crate::params::{Parameters, Planet};
use crate::trig::{Bhuja, RADIUS, Trig};

/// A graha's place: the sidereal longitude in the text's own frame and
/// the daily motion.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
pub struct Position {
    /// The sidereal longitude, degrees in `[0, 360)`.
    pub longitude: Degrees,
    /// The daily motion in degrees, negative when retrograde.
    pub speed_deg_per_day: f64,
}

/// The tradition's intermediate figures, for checking a hand computation
/// or a printed panchanga against the model.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Trace {
    /// The graha.
    pub graha: Graha,
    /// The day count.
    pub ahargana: Ahargana,
    /// The mean place (madhyama), degrees.
    pub mean_deg: f64,
    /// The apsis (mandocca), degrees; the node for Rahu and Ketu.
    pub apsis_deg: f64,
    /// The conjunction (sighrocca) for a star planet.
    pub conjunction_deg: Option<f64>,
    /// The manda equation applied, degrees, signed.
    pub manda_equation_deg: f64,
    /// The place after the manda equation, degrees.
    pub manda_corrected_deg: f64,
    /// The sighra equation applied, for a star planet.
    pub sighra_equation_deg: Option<f64>,
    /// The sighra hypotenuse, for a star planet.
    pub karna: Option<f64>,
    /// The true place, degrees in `[0, 360)`.
    pub longitude_deg: f64,
    /// The daily motion, degrees.
    pub speed_deg_per_day: f64,
}

/// A day's arc at a latitude: when the Sun rises and sets, and the
/// figures behind them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DayArc {
    /// Sunrise, the Sun's centre on the horizon in mean time.
    pub sunrise: JulianDay<Ut1>,
    /// Sunset.
    pub sunset: JulianDay<Ut1>,
    /// The ascensional difference (cara) at sunrise, degrees, positive when
    /// the day is longer than the night.
    pub ascensional_difference_deg: f64,
    /// The Sun's declination at sunrise, degrees, north positive.
    pub declination_deg: f64,
}

/// The Surya Siddhanta over a set of parameters and a trigonometry.
#[derive(Clone, Debug, PartialEq)]
pub struct SuryaSiddhanta {
    params: Parameters,
    trig: Trig,
}

impl SuryaSiddhanta {
    /// The text's parameters with the text's table: the classical path.
    #[must_use]
    pub const fn text() -> SuryaSiddhanta {
        SuryaSiddhanta {
            params: Parameters::TEXT,
            trig: Trig::Table,
        }
    }

    /// A model over any parameters (a tradition's bija applied, say) and
    /// either trigonometry.
    #[must_use]
    pub const fn new(params: Parameters, trig: Trig) -> SuryaSiddhanta {
        SuryaSiddhanta { params, trig }
    }

    /// The parameters.
    #[must_use]
    pub const fn parameters(&self) -> &Parameters {
        &self.params
    }

    /// The trigonometry.
    #[must_use]
    pub const fn trig(&self) -> Trig {
        self.trig
    }

    /// The model's name for provenance stamps.
    #[must_use]
    pub fn describe(&self) -> String {
        let params = if self.params == Parameters::TEXT {
            "the text"
        } else {
            "adjusted parameters"
        };
        let trig = match self.trig {
            Trig::Table => "the sine table",
            Trig::Exact => "exact trigonometry",
        };
        format!("Surya Siddhanta ({params}, {trig})")
    }

    /// The day count at an instant.
    #[must_use]
    pub fn ahargana(&self, at: JulianDay<Ut1>) -> Ahargana {
        Ahargana::at(at.get(), &self.params)
    }

    /// The mean place of a planet's own revolutions (the conjunction's for
    /// Mercury and Venus), degrees.
    #[must_use]
    pub fn mean_deg(&self, planet: Planet, at: Ahargana) -> f64 {
        at.mean_degrees(self.params.motion(planet), &self.params)
    }

    /// The apsis of a planet, degrees.
    #[must_use]
    pub fn apsis_deg(&self, planet: Planet, at: Ahargana) -> f64 {
        at.mean_degrees(self.params.apsis(planet), &self.params)
    }

    /// The Moon's ascending node, degrees.
    #[must_use]
    pub fn node_deg(&self, at: Ahargana) -> f64 {
        at.mean_degrees(self.params.moon_node, &self.params)
    }

    /// A manda-only body (the Sun or the Moon): the true place and the
    /// figures on the way.
    fn manda_only(&self, planet: Planet, at: Ahargana) -> Trace {
        let mean = self.mean_deg(planet, at);
        let apsis = self.apsis_deg(planet, at);
        let epicycle = self.params.manda_epicycle(planet);
        let kendra = apsis - mean;
        let equation = manda_equation_deg(self.trig, epicycle, kendra);
        let longitude = (mean + equation).rem_euclid(360.0);
        let speed = manda_motion_deg_per_day(
            self.trig,
            epicycle,
            kendra,
            self.params.motion(planet).degrees_per_day(&self.params),
            self.params.apsis(planet).degrees_per_day(&self.params),
        );
        Trace {
            graha: graha_of(planet),
            ahargana: at,
            mean_deg: mean,
            apsis_deg: apsis,
            conjunction_deg: None,
            manda_equation_deg: equation,
            manda_corrected_deg: longitude,
            sighra_equation_deg: None,
            karna: None,
            longitude_deg: longitude,
            speed_deg_per_day: speed,
        }
    }

    /// A star planet's four steps at a count, with its mean place, apsis
    /// and conjunction.
    fn star_steps(&self, planet: Planet, at: Ahargana) -> (FourStep, f64, f64, f64) {
        let sun_mean = self.mean_deg(Planet::Sun, at);
        let own = self.mean_deg(planet, at);
        let (mean, conjunction) = if planet.is_inferior() {
            (sun_mean, own)
        } else {
            (own, sun_mean)
        };
        let apsis = self.apsis_deg(planet, at);
        let manda = self.params.manda_epicycle(planet);
        let sighra = self
            .params
            .sighra_epicycle(planet)
            .unwrap_or(Epicycle::new(0, 0));
        (
            four_step(self.trig, manda, sighra, mean, apsis, conjunction),
            mean,
            apsis,
            conjunction,
        )
    }

    /// A star planet: the four steps, and the daily motion as the change
    /// over the day centred on the instant (the text's rule for the
    /// sighra motion, II.50 to 51, is not yet implemented).
    fn star(&self, planet: Planet, at: Ahargana) -> Trace {
        let (steps, mean, apsis, conjunction) = self.star_steps(planet, at);
        let before = self.star_steps(planet, at.plus(-0.5)).0.true_deg;
        let after = self.star_steps(planet, at.plus(0.5)).0.true_deg;
        Trace {
            graha: graha_of(planet),
            ahargana: at,
            mean_deg: mean,
            apsis_deg: apsis,
            conjunction_deg: Some(conjunction),
            manda_equation_deg: steps.manda_equation_deg,
            manda_corrected_deg: steps.manda_corrected_deg,
            sighra_equation_deg: Some(steps.sighra.equation_deg),
            karna: Some(steps.sighra.karna),
            longitude_deg: steps.true_deg,
            speed_deg_per_day: signed_difference(after, before),
        }
    }

    /// The node, direct or opposite.
    fn node(&self, graha: Graha, at: Ahargana) -> Trace {
        let node = self.node_deg(at);
        let longitude = if graha == Graha::Ketu {
            (node + 180.0).rem_euclid(360.0)
        } else {
            node
        };
        Trace {
            graha,
            ahargana: at,
            mean_deg: longitude,
            apsis_deg: node,
            conjunction_deg: None,
            manda_equation_deg: 0.0,
            manda_corrected_deg: longitude,
            sighra_equation_deg: None,
            karna: None,
            longitude_deg: longitude,
            speed_deg_per_day: self.params.moon_node.degrees_per_day(&self.params),
        }
    }

    /// The tradition's figures for a graha at an instant.
    ///
    /// # Errors
    ///
    /// `UNSUPPORTED` for a graha the text does not model (Uranus, Neptune,
    /// Pluto).
    pub fn trace(&self, graha: Graha, at: JulianDay<Ut1>) -> Result<Trace, Error> {
        let count = self.ahargana(at);
        Ok(match graha {
            Graha::Sun => self.manda_only(Planet::Sun, count),
            Graha::Moon => self.manda_only(Planet::Moon, count),
            Graha::Mars => self.star(Planet::Mars, count),
            Graha::Mercury => self.star(Planet::Mercury, count),
            Graha::Jupiter => self.star(Planet::Jupiter, count),
            Graha::Venus => self.star(Planet::Venus, count),
            Graha::Saturn => self.star(Planet::Saturn, count),
            Graha::Rahu | Graha::Ketu => self.node(graha, count),
            other => {
                return Err(Error::unsupported(format!(
                    "the Surya Siddhanta does not model {}",
                    other.key()
                ))
                .with_field("graha")
                .with_hint(
                    "the text knows the Sun, the Moon, the five star planets and the node",
                ));
            }
        })
    }

    /// A graha's place at an instant.
    ///
    /// # Errors
    ///
    /// As [`SuryaSiddhanta::trace`].
    pub fn graha(&self, graha: Graha, at: JulianDay<Ut1>) -> Result<Position, Error> {
        self.trace(graha, at).map(Position::from)
    }

    /// The Sun's place: the mean place with the manda equation (II.45).
    #[must_use]
    pub fn sun(&self, at: JulianDay<Ut1>) -> Position {
        Position::from(self.manda_only(Planet::Sun, self.ahargana(at)))
    }

    /// The Sun's sidereal longitude at a Universal Time Julian day given
    /// as a number, the form a root finder calls.
    #[must_use]
    pub fn sun_longitude_deg(&self, jd_ut: f64) -> f64 {
        self.manda_only(Planet::Sun, Ahargana::at(jd_ut, &self.params))
            .longitude_deg
    }

    /// The Moon's place.
    #[must_use]
    pub fn moon(&self, at: JulianDay<Ut1>) -> Position {
        Position::from(self.manda_only(Planet::Moon, self.ahargana(at)))
    }

    /// The nine grahas the text models, in the catalogue's order.
    #[must_use]
    pub fn all(&self, at: JulianDay<Ut1>) -> [(Graha, Position); 9] {
        let count = self.ahargana(at);
        let mut sun = self.manda_only(Planet::Sun, count);
        sun.graha = Graha::Sun;
        [
            (Graha::Sun, Position::from(sun)),
            (
                Graha::Moon,
                Position::from(self.manda_only(Planet::Moon, count)),
            ),
            (Graha::Mars, Position::from(self.star(Planet::Mars, count))),
            (
                Graha::Mercury,
                Position::from(self.star(Planet::Mercury, count)),
            ),
            (
                Graha::Jupiter,
                Position::from(self.star(Planet::Jupiter, count)),
            ),
            (
                Graha::Venus,
                Position::from(self.star(Planet::Venus, count)),
            ),
            (
                Graha::Saturn,
                Position::from(self.star(Planet::Saturn, count)),
            ),
            (Graha::Rahu, Position::from(self.node(Graha::Rahu, count))),
            (Graha::Ketu, Position::from(self.node(Graha::Ketu, count))),
        ]
    }

    /// The text's precession (III.9 to 12): the equinoxes librate 600
    /// times an age; the reduced arc of the libration's argument, three
    /// tenths of it, is the ayanamsha, at most 27°. The sign is taken as
    /// the tradition applies the text today: zero at the start of the
    /// Kali age and again 3600 years later (Shaka 421), positive since,
    /// 54″ a year; so the tropical longitude is the sidereal one plus this.
    #[must_use]
    pub fn ayanamsha_deg(&self, at: JulianDay<Ut1>) -> f64 {
        let motion = Motion::direct(
            u64::from(self.params.ayana_revolutions_per_yuga),
            Cycle::Yuga,
        );
        let argument = self.ahargana(at).mean_degrees(motion, &self.params);
        let bhuja = Bhuja::of(argument);
        let magnitude = bhuja.arc_deg * f64::from(self.params.ayana_extent_deg) / 90.0;
        if bhuja.sine_positive {
            -magnitude
        } else {
            magnitude
        }
    }

    /// The Sun's declination from its tropical longitude (II.28 with
    /// III.17): the sine of the longitude times the sine of the greatest
    /// declination over the radius; north in the half from Mesha.
    #[must_use]
    pub fn declination_deg(&self, tropical_longitude_deg: f64) -> f64 {
        let bhuja = Bhuja::of(tropical_longitude_deg);
        let sine = self.trig.sine(bhuja.arc_deg) * f64::from(self.params.obliquity_sine) / RADIUS;
        let arc = self.trig.arc(sine);
        if bhuja.sine_positive { arc } else { -arc }
    }

    /// The ascensional difference (III.14 to 17, III.34 to 35): the
    /// equinoctial shadow of a twelve-digit gnomon is twelve times the
    /// sine of the latitude over its cosine; the earth-sine is the sine
    /// of the declination times the shadow over twelve; the earth-sine
    /// times the radius over the day-radius (the cosine of the
    /// declination) is the sine of the difference. Degrees, positive when
    /// the day is longer than the night; `None` when the Sun neither
    /// rises nor sets.
    #[must_use]
    pub fn ascensional_difference_deg(
        &self,
        latitude: Latitude,
        declination_deg: f64,
    ) -> Option<f64> {
        let phi = latitude.get();
        let shadow = 12.0 * self.trig.sine(phi.abs()) / self.trig.sine(90.0 - phi.abs());
        let earth_sine = self.trig.sine(declination_deg.abs()) * shadow / 12.0;
        let day_radius = self.trig.sine(90.0 - declination_deg.abs());
        if day_radius <= 0.0 {
            return None;
        }
        let sine = earth_sine * RADIUS / day_radius;
        if sine > RADIUS {
            return None;
        }
        let arc = self.trig.arc(sine);
        let same_side = (phi >= 0.0) == (declination_deg >= 0.0);
        Some(if same_side { arc } else { -arc })
    }

    /// Sunrise and sunset of the local mean day that begins at
    /// `local_mean_midnight`, at a latitude (III.42 to 43): half the day
    /// is a quarter of the circle plus the ascensional difference, so the
    /// Sun rises that difference before six and sets it after eighteen in
    /// local mean time. The declination is taken at the rise and set
    /// instants themselves, each found in two passes. `None` where the
    /// Sun neither rises nor sets.
    #[must_use]
    pub fn day_arc(
        &self,
        local_mean_midnight: JulianDay<Ut1>,
        latitude: Latitude,
    ) -> Option<DayArc> {
        let midnight = local_mean_midnight.get();
        let event = |base: f64, sign: f64| -> Option<(f64, f64, f64)> {
            let mut fraction = base;
            let mut cara = 0.0;
            let mut declination = 0.0;
            for _ in 0..2 {
                let jd = midnight + fraction;
                let sun = self.sun_longitude_deg(jd);
                let ayanamsha = self.ayanamsha_deg(JulianDay::try_new(jd).ok()?);
                declination = self.declination_deg(sun + ayanamsha);
                cara = self.ascensional_difference_deg(latitude, declination)?;
                fraction = base + sign * cara / 360.0;
            }
            Some((fraction, cara, declination))
        };
        let (rise, cara, declination) = event(0.25, -1.0)?;
        let (set, _, _) = event(0.75, 1.0)?;
        Some(DayArc {
            sunrise: JulianDay::try_new(midnight + rise).ok()?,
            sunset: JulianDay::try_new(midnight + set).ok()?,
            ascensional_difference_deg: cara,
            declination_deg: declination,
        })
    }
}

impl SuryaSiddhanta {
    /// Whether, on a day without a sunrise, the Sun is above the horizon
    /// throughout (a polar day) rather than below it (a polar night): the
    /// declination at local mean noon has the latitude's sign.
    #[must_use]
    pub fn sun_up_all_day(&self, local_mean_midnight: JulianDay<Ut1>, latitude: Latitude) -> bool {
        let noon = local_mean_midnight.get() + 0.5;
        let sun = self.sun_longitude_deg(noon);
        let ayanamsha = JulianDay::try_new(noon).map_or(0.0, |at| self.ayanamsha_deg(at));
        let declination = self.declination_deg(sun + ayanamsha);
        (latitude.get() >= 0.0) == (declination >= 0.0)
    }
}

impl From<Trace> for Position {
    fn from(trace: Trace) -> Position {
        Position {
            longitude: Degrees::try_new(trace.longitude_deg).unwrap_or_default(),
            speed_deg_per_day: trace.speed_deg_per_day,
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({:+.4}°/day)",
            self.longitude, self.speed_deg_per_day
        )
    }
}

/// The catalogue key of a planet.
const fn graha_of(planet: Planet) -> Graha {
    match planet {
        Planet::Sun => Graha::Sun,
        Planet::Moon => Graha::Moon,
        Planet::Mars => Graha::Mars,
        Planet::Mercury => Graha::Mercury,
        Planet::Jupiter => Graha::Jupiter,
        Planet::Venus => Graha::Venus,
        Planet::Saturn => Graha::Saturn,
    }
}

/// `after - before` along the shorter arc, in `(-180, 180]`.
fn signed_difference(after: f64, before: f64) -> f64 {
    let d = (after - before).rem_euclid(360.0);
    if d > 180.0 { d - 360.0 } else { d }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use proptest::prelude::*;

    use super::*;

    fn jd(value: f64) -> JulianDay<Ut1> {
        JulianDay::try_new(value).unwrap()
    }

    /// 0h UT on a proleptic Gregorian date, for the tests' dates.
    fn jd_of(year: i32, month: u32, day: u32) -> JulianDay<Ut1> {
        let (y, m) = if month <= 2 {
            (year - 1, month + 12)
        } else {
            (year, month)
        };
        let a = y.div_euclid(100);
        let b = 2 - a + a.div_euclid(4);
        jd((365.25 * f64::from(y + 4716)).floor()
            + (30.6001 * f64::from(m + 1)).floor()
            + f64::from(day)
            + f64::from(b)
            - 1524.5)
    }

    #[test]
    fn the_sun_crosses_mesha_in_mid_april_and_moves_about_a_degree_a_day() {
        let text = SuryaSiddhanta::text();
        // Around 13 April 2024 the text's Sun enters Mesha (1 Baisakh
        // 2081 BS is 13 April 2024).
        let before = text.sun(jd_of(2024, 4, 12)).longitude.get();
        let after = text.sun(jd_of(2024, 4, 15)).longitude.get();
        assert!(before > 358.0, "{before}");
        assert!(after < 2.0, "{after}");
        let sun = text.sun(jd_of(2024, 4, 13));
        assert!((sun.speed_deg_per_day - 0.98).abs() < 0.03, "{sun}");
        assert!(sun.to_string().contains("°/day"));
        // In early January the Sun is fastest, in early July slowest.
        let january = text.sun(jd_of(2024, 1, 3)).speed_deg_per_day;
        let july = text.sun(jd_of(2024, 7, 5)).speed_deg_per_day;
        assert!(january > 1.0 && july < 0.96, "{january} {july}");
    }

    #[test]
    fn the_tradition_worked_example_is_reproduced() {
        // A hand computation of the tradition for 31 October 1994 at
        // Kathmandu (10:49 local, +5:45) reaches a mean Sun of 6 signs
        // 15°46′30″, an apsis of 77°17′39″, a manda equation of 1°55′6″
        // and a true Sun of 6 signs 13°51′. Its count of days, 1 861 191,
        // is the text's count to that day's midnight; the figures follow
        // from the text at the next midnight, the tradition counting the
        // current day as elapsed.
        let text = SuryaSiddhanta::text();
        let midnight = text.ahargana(jd_of(1994, 10, 31));
        assert_eq!(midnight.days, 1_861_191, "{midnight}");
        let count = midnight.plus(1.0 - midnight.fraction);
        assert_eq!(count.days, 1_861_192);
        let mean = text.mean_deg(Planet::Sun, count);
        assert!((mean - (195.0 + 46.5 / 60.0)).abs() < 0.002, "{mean}");
        let apsis = text.apsis_deg(Planet::Sun, count);
        assert!((apsis - (77.0 + 17.65 / 60.0)).abs() < 0.001, "{apsis}");
        let trace = text
            .trace(Graha::Sun, jd(text.parameters().epoch_jd_ut + 1_861_192.0))
            .unwrap();
        assert!(
            (trace.manda_equation_deg + (1.0 + 55.1 / 60.0)).abs() < 0.003,
            "{}",
            trace.manda_equation_deg
        );
        assert!(
            (trace.longitude_deg - (193.0 + 51.5 / 60.0)).abs() < 0.01,
            "{}",
            trace.longitude_deg
        );
        assert!((trace.speed_deg_per_day - 1.0033).abs() < 0.001);
    }

    #[test]
    fn every_graha_answers_and_the_unknown_ones_are_refused() {
        let text = SuryaSiddhanta::text();
        let at = jd_of(2024, 1, 15);
        let all = text.all(at);
        assert_eq!(all.len(), 9);
        for (graha, position) in all {
            assert!((0.0..360.0).contains(&position.longitude.get()), "{graha}");
            let again = text.graha(graha, at).unwrap();
            assert_eq!(again, position, "{graha}");
        }
        let rahu = text.graha(Graha::Rahu, at).unwrap();
        let ketu = text.graha(Graha::Ketu, at).unwrap();
        assert!(
            (signed_difference(ketu.longitude.get(), rahu.longitude.get()).abs() - 180.0).abs()
                < 1e-9
        );
        assert!(rahu.speed_deg_per_day < 0.0);
        let error = text.graha(Graha::Pluto, at).unwrap_err();
        assert_eq!(error.field(), Some("graha"));
        assert!(error.message.contains("PLUTO"));
        let mercury = text.trace(Graha::Mercury, at).unwrap();
        assert!(mercury.conjunction_deg.is_some() && mercury.karna.is_some());
        let elongation =
            signed_difference(mercury.longitude_deg, text.sun(at).longitude.get()).abs();
        assert!(elongation < 30.0, "{elongation}");
        let moon = text.moon(at);
        assert!(
            moon.speed_deg_per_day > 11.0 && moon.speed_deg_per_day < 15.5,
            "{moon}"
        );
        assert!(text.describe().contains("the text"));
        assert_eq!(
            SuryaSiddhanta::new(Parameters::TEXT, Trig::Exact).describe(),
            "Surya Siddhanta (the text, exact trigonometry)"
        );
    }

    #[test]
    fn precession_is_zero_at_shaka_421_and_54_seconds_a_year_since() {
        let text = SuryaSiddhanta::text();
        // 499 CE, the year of Shaka 421.
        assert!(text.ayanamsha_deg(jd_of(499, 3, 21)).abs() < 0.02);
        // 2024: 1525 years × 54″ = 22.875°.
        let now = text.ayanamsha_deg(jd_of(2024, 3, 21));
        assert!((now - 22.875).abs() < 0.02, "{now}");
        // 200 BCE: negative, the equinox ahead of the sidereal zero.
        let early = text.ayanamsha_deg(jd_of(-199, 3, 21));
        assert!((early + 10.485).abs() < 0.02, "{early}");
        // The start of the Kali age: zero.
        assert!(text.ayanamsha_deg(jd(text.parameters().epoch_jd_ut)).abs() < 1e-6);
    }

    #[test]
    fn declination_and_ascensional_difference_follow_the_text() {
        let text = SuryaSiddhanta::text();
        // At the solstice the declination is the obliquity's 24°.
        assert!((text.declination_deg(90.0) - 24.0).abs() < 0.02);
        assert!((text.declination_deg(270.0) + 24.0).abs() < 0.02);
        assert!(text.declination_deg(0.0).abs() < 1e-9);
        assert!(text.declination_deg(200.0) < 0.0);
        // At the equator there is no ascensional difference.
        let equator = Latitude::try_new(0.0).unwrap();
        assert!(
            text.ascensional_difference_deg(equator, 20.0)
                .unwrap()
                .abs()
                < 1e-9
        );
        // At Kathmandu with the Sun at 24° north: asin(tan 27.7° tan 24°).
        let kathmandu = Latitude::try_new(27.7172).unwrap();
        let cara = text.ascensional_difference_deg(kathmandu, 24.0).unwrap();
        let expected = (27.7172f64.to_radians().tan() * 24.0f64.to_radians().tan())
            .asin()
            .to_degrees();
        assert!((cara - expected).abs() < 0.1, "{cara} vs {expected}");
        assert!(text.ascensional_difference_deg(kathmandu, -24.0).unwrap() < 0.0);
        // South of the equator the sign flips.
        let sydney = Latitude::try_new(-33.9).unwrap();
        assert!(text.ascensional_difference_deg(sydney, 24.0).unwrap() < 0.0);
        // Above the polar circle in summer the Sun does not set.
        let tromso = Latitude::try_new(69.6).unwrap();
        assert!(text.ascensional_difference_deg(tromso, 24.0).is_none());
    }

    #[test]
    fn the_day_arc_at_kathmandu_is_long_in_june_and_short_in_december() {
        let text = SuryaSiddhanta::text();
        let kathmandu = Latitude::try_new(27.7172).unwrap();
        let lmt_offset = 85.324 / 360.0;
        let june = text
            .day_arc(jd(jd_of(2024, 6, 21).get() - lmt_offset), kathmandu)
            .unwrap();
        let june_length = (june.sunset.get() - june.sunrise.get()) * 24.0;
        assert!(june_length > 13.5 && june_length < 14.0, "{june_length}");
        let december = text
            .day_arc(jd(jd_of(2024, 12, 21).get() - lmt_offset), kathmandu)
            .unwrap();
        let december_length = (december.sunset.get() - december.sunrise.get()) * 24.0;
        assert!(
            december_length > 10.0 && december_length < 10.5,
            "{december_length}"
        );
        assert!(june.ascensional_difference_deg > 0.0 && december.ascensional_difference_deg < 0.0);
        assert!(june.declination_deg > 23.0 && december.declination_deg < -23.0);
        // Sunrise in local mean time: about 5:07 in June (6h less the
        // difference of about 53 minutes).
        let sunrise_local = (june.sunrise.get() + lmt_offset + 0.5).rem_euclid(1.0) * 24.0;
        assert!((sunrise_local - 5.10).abs() < 0.05, "{sunrise_local}");
        // The equator has a six-hour sunrise in mean time all year.
        let equator = Latitude::try_new(0.0).unwrap();
        let arc = text.day_arc(jd_of(2024, 3, 1), equator).unwrap();
        assert!(((arc.sunrise.get() - jd_of(2024, 3, 1).get()) * 24.0 - 6.0).abs() < 1e-9);
        // Polar: none.
        let tromso = Latitude::try_new(69.6).unwrap();
        assert!(text.day_arc(jd_of(2024, 6, 21), tromso).is_none());
        assert!(text.sun_up_all_day(jd_of(2024, 6, 21), tromso));
        assert!(!text.sun_up_all_day(jd_of(2024, 12, 21), tromso));
    }

    #[test]
    fn exact_trigonometry_stays_within_the_tables_precision() {
        let table = SuryaSiddhanta::text();
        let exact = SuryaSiddhanta::new(Parameters::TEXT, Trig::Exact);
        for day in 0..40 {
            let at = jd(2_460_000.5 + f64::from(day) * 9.0);
            for (graha, a) in table.all(at) {
                let b = exact.graha(graha, at).unwrap();
                let gap = signed_difference(a.longitude.get(), b.longitude.get()).abs();
                let limit = if matches!(graha, Graha::Mars | Graha::Venus) {
                    0.6
                } else {
                    0.1
                };
                assert!(gap < limit, "{graha} at {at}: {gap}");
            }
        }
    }

    #[test]
    fn the_classical_path_is_deterministic_to_the_bit() {
        // A golden value: the text's Sun at J2000.0 through the table.
        // A change here is a change in every number the model produces
        // and needs a calculation-version entry.
        let sun = SuryaSiddhanta::text().sun_longitude_deg(2_451_545.0);
        assert_eq!(sun.to_bits(), 0x4070_0452_25F3_190C, "{sun}");
    }

    proptest! {
        #[test]
        fn longitudes_are_in_range_and_continuous(jd_value in 2_300_000.0f64..2_600_000.0) {
            let text = SuryaSiddhanta::text();
            let at = jd(jd_value);
            let later = jd(jd_value + 0.01);
            for (graha, position) in text.all(at) {
                prop_assert!((0.0..360.0).contains(&position.longitude.get()));
                let next = text.graha(graha, later).unwrap();
                let moved = signed_difference(next.longitude.get(), position.longitude.get()).abs();
                prop_assert!(moved < 0.2, "{graha} moved {moved} in 0.01 day");
            }
            let sun = text.sun(at);
            prop_assert!(sun.speed_deg_per_day > 0.94 && sun.speed_deg_per_day < 1.03);
        }
    }
}
