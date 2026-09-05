//! The model: a graha's place at an instant, the tradition's trace of the
//! computation, and the text's precession, declination and ascensional
//! difference that give sunrise and sunset.

use core::fmt;

use teistro_core::catalogue::Graha;
use teistro_core::error::Error;
use teistro_core::quantity::{Degrees, JulianDay, Latitude, Longitude, Ut1};

use crate::equation::{
    Epicycle, FourStep, four_step, latitude_deg, manda_equation_deg, manda_motion_deg_per_day,
    sighra_motion_deg_per_day,
};
use crate::lagna::{ASU_PER_DAY, Lagna, RisingTimes};
use crate::mean::{Ahargana, Cycle, Motion};
use crate::params::{Parameters, Planet};
use crate::trig::{Bhuja, RADIUS, Trig};

/// A graha's place: the sidereal longitude in the text's own frame, the
/// latitude, and the daily motion.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
pub struct Position {
    /// The sidereal longitude, degrees in `[0, 360)`.
    pub longitude: Degrees,
    /// The latitude, degrees, north positive (II.56 to 57); zero for the
    /// Sun and the nodes.
    pub latitude_deg: f64,
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
    /// The node the latitude is reckoned from, degrees, for a body that
    /// has one.
    pub node_deg: Option<f64>,
    /// The latitude, degrees, north positive.
    pub latitude_deg: f64,
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
        // The Moon's latitude (II.57): its distance from its node, over the
        // radius; the Sun has none.
        let (node_deg, latitude) = match (
            self.params.node(planet),
            self.params.extreme_latitude_arcmin(planet),
        ) {
            (Some(node), Some(extreme)) => {
                let node_deg = at.mean_degrees(node, &self.params);
                (
                    Some(node_deg),
                    latitude_deg(self.trig, longitude - node_deg, extreme, RADIUS),
                )
            }
            _ => (None, 0.0),
        };
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
            node_deg,
            latitude_deg: latitude,
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

    /// A star planet: the four steps, the daily motion by the text's rule
    /// (II.47 to 51: the motion corrected for the apsis at the third
    /// step's anomaly, then the conjunction's equation of motion through
    /// the last hypotenuse), and the latitude (II.56 to 57: the distance
    /// from the node, which the conjunction's equation moves along with
    /// the planet for Mars, Jupiter and Saturn, and which for Mercury and
    /// Venus is the conjunction's distance from the node moved by the
    /// apsis's equation the other way; over the last hypotenuse).
    fn star(&self, planet: Planet, at: Ahargana) -> Trace {
        let (steps, mean, apsis, conjunction) = self.star_steps(planet, at);
        let own_motion = self.params.motion(planet).degrees_per_day(&self.params);
        let sun_motion = self
            .params
            .motion(Planet::Sun)
            .degrees_per_day(&self.params);
        // The motion of the mean place: the Sun's for an inferior planet,
        // whose own revolutions are its conjunction's.
        let (mean_motion, conjunction_motion) = if planet.is_inferior() {
            (sun_motion, own_motion)
        } else {
            (own_motion, sun_motion)
        };
        let equated = manda_motion_deg_per_day(
            self.trig,
            self.params.manda_epicycle(planet),
            steps.manda_anomaly_deg,
            mean_motion,
            self.params.apsis(planet).degrees_per_day(&self.params),
        );
        let speed = sighra_motion_deg_per_day(equated, conjunction_motion, steps.sighra.karna);
        let node_deg = self
            .params
            .node(planet)
            .map(|node| at.mean_degrees(node, &self.params));
        let latitude = match (node_deg, self.params.extreme_latitude_arcmin(planet)) {
            (Some(node), Some(extreme)) => {
                let distance = if planet.is_inferior() {
                    conjunction + steps.manda_equation_deg - node
                } else {
                    steps.manda_corrected_deg - node
                };
                latitude_deg(self.trig, distance, extreme, steps.sighra.karna)
            }
            _ => 0.0,
        };
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
            node_deg,
            latitude_deg: latitude,
            speed_deg_per_day: speed,
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
            node_deg: Some(node),
            latitude_deg: 0.0,
            speed_deg_per_day: self.params.moon_node.degrees_per_day(&self.params),
        }
    }

    /// The Moon's apogee (its apsis, the mandocca), for the port's mean
    /// apogee.
    fn moon_apogee(&self, at: Ahargana) -> Trace {
        let apsis = self.apsis_deg(Planet::Moon, at);
        Trace {
            graha: Graha::Moon,
            ahargana: at,
            mean_deg: apsis,
            apsis_deg: apsis,
            conjunction_deg: None,
            manda_equation_deg: 0.0,
            manda_corrected_deg: apsis,
            sighra_equation_deg: None,
            karna: None,
            longitude_deg: apsis,
            node_deg: None,
            latitude_deg: 0.0,
            speed_deg_per_day: self.params.moon_apsis.degrees_per_day(&self.params),
        }
    }

    /// The Moon's apogee at an instant: the apsis of I.34, which moves
    /// direct.
    #[must_use]
    pub fn moon_apogee_position(&self, at: JulianDay<Ut1>) -> Position {
        Position::from(self.moon_apogee_trace(at))
    }

    /// The Moon's apogee as a trace.
    #[must_use]
    pub fn moon_apogee_trace(&self, at: JulianDay<Ut1>) -> Trace {
        self.moon_apogee(self.ahargana(at))
    }

    /// The node of a planet at a count, degrees: the Moon's from I.34, a
    /// star planet's from I.43 to 44; `None` for the Sun.
    #[must_use]
    pub fn planet_node_deg(&self, planet: Planet, at: Ahargana) -> Option<f64> {
        self.params
            .node(planet)
            .map(|node| at.mean_degrees(node, &self.params))
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
    /// The local mean midnight that begins the local mean day an instant
    /// falls in, at a longitude.
    #[must_use]
    pub fn local_mean_midnight(at: JulianDay<Ut1>, longitude: Longitude) -> JulianDay<Ut1> {
        teistro_astro::sky::local_mean_midnight(at, longitude)
    }

    /// The times of rising of the signs at a latitude (III.42 to 45), or
    /// `None` where the text's rule has no answer.
    #[must_use]
    pub fn rising_times(&self, latitude: Latitude) -> Option<RisingTimes> {
        RisingTimes::at(self, latitude)
    }

    /// The horoscope point at an instant and a place (III.46 to 49): the
    /// Sun's tropical longitude at the sunrise of the local mean day the
    /// instant falls in, carried forward by the time since sunrise in
    /// respirations through the signs' rising times at the latitude (or
    /// back, before sunrise), and the point on the meridian from the
    /// Sun's hour angle through the rising times at Lanka; both returned
    /// with the text's precession taken off again.
    ///
    /// # Errors
    ///
    /// `UNSUPPORTED` at a latitude where the Sun does not rise or set on
    /// the day, or where a sign never rises, naming the place.
    pub fn lagna(
        &self,
        at: JulianDay<Ut1>,
        latitude: Latitude,
        longitude: Longitude,
    ) -> Result<Lagna, Error> {
        let midnight = SuryaSiddhanta::local_mean_midnight(at, longitude);
        let unsupported = |what: &str| {
            Error::unsupported(format!(
                "the text's horoscope point needs {what} at {latitude} {longitude} on {midnight}"
            ))
            .with_field("place")
        };
        let arc = self
            .day_arc(midnight, latitude)
            .ok_or_else(|| unsupported("a sunrise"))?;
        let rising = self
            .rising_times(latitude)
            .ok_or_else(|| unsupported("every sign to rise"))?;
        let ayanamsha = self.ayanamsha_deg(arc.sunrise);
        let sun_tropical =
            (self.sun_longitude_deg(arc.sunrise.get()) + ayanamsha).rem_euclid(360.0);
        let elapsed_asu = (at.get() - arc.sunrise.get()) * ASU_PER_DAY;
        let tropical = rising.point_after(sun_tropical, elapsed_asu);
        // The hour angle from local mean noon, the middle of the text's
        // symmetric day, through the right ascensions.
        let noon = midnight.get() + 0.5;
        let hour_angle_asu = (at.get() - noon) * ASU_PER_DAY;
        let meridian_tropical =
            RisingTimes::lanka(&self.params).point_after(sun_tropical, hour_angle_asu);
        Ok(Lagna {
            sidereal_deg: (tropical - ayanamsha).rem_euclid(360.0),
            tropical_deg: tropical,
            meridian_sidereal_deg: (meridian_tropical - ayanamsha).rem_euclid(360.0),
            sun_tropical_deg: sun_tropical,
            elapsed_asu,
            rising,
        })
    }

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
            latitude_deg: trace.latitude_deg,
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

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use proptest::prelude::*;
    use teistro_core::angle::difference_deg as signed_difference;

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

    /// Midnight of 1 January 1860 at Washington (77°3′ W), the instant
    /// of every worked example in Burgess's notes: 25 nadis 28 vinadis
    /// after midnight on the meridian of Ujjain.
    fn burgess_instant() -> JulianDay<Ut1> {
        jd(2_400_410.5 + (77.0 + 3.0 / 60.0) / 360.0)
    }

    #[test]
    fn burgess_day_count_and_mean_sun_for_1860() {
        // Burgess under I.48 to 53: the sum of days to the beginning of
        // 1 January 1860 at midnight on the prime meridian is
        // 714 404 108 572 from the creation, 1 811 945 from the Kali age;
        // the mean Sun then is 8s 17°48′7″.
        let text = SuryaSiddhanta::text();
        let ujjain_midnight = jd(2_400_410.5 - (75.0 + 47.0 / 60.0) / 360.0);
        let count = text.ahargana(ujjain_midnight);
        assert_eq!(count.days, 1_811_945);
        assert!(count.fraction.abs() < 1e-6, "{}", count.fraction);
        assert_eq!(
            u64::try_from(count.days).unwrap() + Parameters::TEXT.elapsed_days_at_kali,
            714_404_108_572
        );
        let sun = text.mean_deg(Planet::Sun, count);
        let expected = 240.0 + 17.0 + 48.0 / 60.0 + 7.0 / 3600.0;
        assert!(
            (sun - expected).abs() * 3600.0 < 2.0,
            "{sun} against {expected}"
        );
        // Under III.9 to 12 for the Washington instant: a precession of
        // 20°24′39″.
        let ayanamsha = text.ayanamsha_deg(burgess_instant());
        let expected = 20.0 + 24.0 / 60.0 + 39.0 / 3600.0;
        assert!((ayanamsha - expected).abs() * 3600.0 < 5.0, "{ayanamsha}");
    }

    #[test]
    fn burgess_daily_motions_for_1860() {
        // Under II.47 to 49: the Moon's mean anomaly 10s 18°46′15″ (the
        // apsis 327°50′24″ less the Moon 9°4′9″ of Burgess's table of mean
        // places at Washington), the difference of sines 174, the equation
        // 53′31″, the true motion 737′4″; the Sun's equation of motion
        // +2′18″ and true motion 61′26″. Under II.50 to 51: Mars's equation for the apsis
        // −3′41″, equated motion 27′45″, synodic motion 31′23″, hypotenuse
        // 3984, equation for the conjunction +4′18″, true motion 32′3″.
        let text = SuryaSiddhanta::text();
        let at = burgess_instant();
        let moon = text.trace(Graha::Moon, at).unwrap();
        let anomaly = (moon.apsis_deg - moon.mean_deg).rem_euclid(360.0);
        assert!(
            (anomaly - (300.0 + 18.0 + 46.25 / 60.0)).abs() < 0.01,
            "anomaly {anomaly}: mean {} apsis {} at count {:?}",
            moon.mean_deg,
            moon.apsis_deg,
            moon.ahargana
        );
        // The mean places of Burgess's table, sidereal: the Moon
        // 350°59′1″, its apsis 309°45′16″, its node 294°24′43″.
        assert!((moon.mean_deg - (350.0 + 59.0 / 60.0 + 1.0 / 3600.0)).abs() * 3600.0 < 5.0);
        assert!((moon.apsis_deg - (309.0 + 45.0 / 60.0 + 16.0 / 3600.0)).abs() * 3600.0 < 5.0);
        assert!(
            (moon.node_deg.unwrap() - (294.0 + 24.0 / 60.0 + 43.0 / 3600.0)).abs() * 3600.0 < 5.0
        );
        let expected_moon = (737.0 + 4.0 / 60.0) / 60.0;
        assert!(
            (moon.speed_deg_per_day - expected_moon).abs() * 3600.0 < 20.0,
            "{} against {expected_moon}",
            moon.speed_deg_per_day * 60.0
        );
        let sun = text.sun(at);
        let expected_sun = (61.0 + 26.0 / 60.0) / 60.0;
        assert!(
            (sun.speed_deg_per_day - expected_sun).abs() * 3600.0 < 6.0,
            "{}",
            sun.speed_deg_per_day * 60.0
        );
        let mars = text.trace(Graha::Mars, at).unwrap();
        assert!(
            (mars.karna.unwrap() - 3984.0).abs() < 6.0,
            "{:?}",
            mars.karna
        );
        let expected_mars = (32.0 + 3.0 / 60.0) / 60.0;
        assert!(
            (mars.speed_deg_per_day - expected_mars).abs() * 3600.0 < 30.0,
            "{} against {expected_mars}",
            mars.speed_deg_per_day * 60.0
        );
        // Jupiter and Saturn were retrograde at that time ("as the last
        // table shows").
        assert!(text.trace(Graha::Jupiter, at).unwrap().speed_deg_per_day < 0.0);
        assert!(text.trace(Graha::Saturn, at).unwrap().speed_deg_per_day < 0.0);
    }

    #[test]
    fn burgess_latitudes_for_1860() {
        // Under II.56 to 58: the Moon 53°14′ from its node, the sine 2754,
        // the latitude 3°36′ north; Mercury 3s 24°14′ from its node, the
        // sine 3134, the latitude 2°4′ north.
        let text = SuryaSiddhanta::text();
        let at = burgess_instant();
        let moon = text.trace(Graha::Moon, at).unwrap();
        let distance = (moon.longitude_deg - moon.node_deg.unwrap()).rem_euclid(360.0);
        assert!((distance - (53.0 + 14.0 / 60.0)).abs() < 0.05, "{distance}");
        assert!(
            (moon.latitude_deg - 3.6).abs() < 0.02,
            "{}",
            moon.latitude_deg
        );
        let mercury = text.trace(Graha::Mercury, at).unwrap();
        assert!(
            (mercury.latitude_deg - (2.0 + 4.0 / 60.0)).abs() < 0.05,
            "{}",
            mercury.latitude_deg
        );
        // Every latitude stays within the extreme, and the Sun has none.
        for graha in [
            Graha::Moon,
            Graha::Mars,
            Graha::Mercury,
            Graha::Jupiter,
            Graha::Venus,
            Graha::Saturn,
        ] {
            let trace = text.trace(graha, at).unwrap();
            assert!(
                trace.latitude_deg.abs() <= 4.6,
                "{graha}: {}",
                trace.latitude_deg
            );
            assert!(trace.node_deg.is_some());
        }
        assert!(text.sun(at).latitude_deg.abs() < f64::EPSILON);
        assert!(text.trace(Graha::Rahu, at).unwrap().latitude_deg.abs() < f64::EPSILON);
    }

    #[test]
    fn the_lagna_is_the_sun_at_sunrise_and_advances_through_the_day() {
        let text = SuryaSiddhanta::text();
        let kathmandu = (Latitude::literal(27.7172), Longitude::literal(85.324));
        let midnight = SuryaSiddhanta::local_mean_midnight(jd_of(2024, 4, 13), kathmandu.1);
        assert!((midnight.get() - (2_460_413.5 - 85.324 / 360.0)).abs() < 1e-9);
        let arc = text.day_arc(midnight, kathmandu.0).unwrap();
        // At sunrise the horoscope point is the Sun.
        let at_sunrise = text.lagna(arc.sunrise, kathmandu.0, kathmandu.1).unwrap();
        let sun = text.sun_longitude_deg(arc.sunrise.get());
        assert!(
            (at_sunrise.sidereal_deg - sun).abs() < 1e-6,
            "{} {sun}",
            at_sunrise.sidereal_deg
        );
        assert!(at_sunrise.elapsed_asu.abs() < 1e-6);
        // Six hours later it is about a quadrant on; at noon the meridian
        // point is the Sun; before sunrise it is behind the Sun.
        let later = text
            .lagna(
                arc.sunrise.plus_days(0.25).unwrap(),
                kathmandu.0,
                kathmandu.1,
            )
            .unwrap();
        let advance = (later.sidereal_deg - sun).rem_euclid(360.0);
        assert!(advance > 75.0 && advance < 105.0, "{advance}");
        let noon = midnight.plus_days(0.5).unwrap();
        let at_noon = text.lagna(noon, kathmandu.0, kathmandu.1).unwrap();
        let meridian_gap =
            (at_noon.meridian_sidereal_deg - text.sun_longitude_deg(noon.get())).rem_euclid(360.0);
        assert!(meridian_gap < 0.5 || meridian_gap > 359.5, "{meridian_gap}");
        let before = text
            .lagna(
                arc.sunrise.plus_days(-1.0 / 24.0).unwrap(),
                kathmandu.0,
                kathmandu.1,
            )
            .unwrap();
        assert!(before.elapsed_asu < 0.0);
        let behind = (sun - before.sidereal_deg).rem_euclid(360.0);
        assert!(behind > 5.0 && behind < 25.0, "{behind}");
        assert_eq!(before.rising, text.rising_times(kathmandu.0).unwrap());
        // Above the polar circle the text has no answer.
        assert!(
            text.lagna(jd_of(2024, 6, 21), Latitude::literal(70.0), kathmandu.1)
                .is_err()
        );
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
