//! The rise and set solver: when a body reaches a horizon convention at a
//! place, from a source of apparent positions, with polar days and nights
//! reported as absences rather than guessed
//! (`docs/03-design/astro-events-and-crossings.md`, §4).
//!
//! The method is Meeus's (*Astronomical Algorithms*, chapter 15) iterated
//! to convergence: from the body's apparent right ascension and
//! declination and the local apparent sidereal time, the hour angle at
//! which the body's centre stands at the event altitude gives a first
//! instant, and each pass corrects it by the altitude error over the
//! altitude's rate. Where that rate vanishes (a grazing event near the
//! polar circles) or the iteration does not settle, the solver scans the
//! window for the altitude's sign change and bisects it, through the
//! shared solver, and says which method answered.
//!
//! The event altitude follows the convention: the disc point's target
//! altitude, less the refraction at the horizon, less the semidiameter
//! for the upper limb (more for the lower), plus the horizontal parallax,
//! since the observer sees the body lower than the Earth's centre does.
//! The observer's height above the ground is not applied: the almanacs
//! and the panchanga reckon from sea level, and a dip is a custom
//! altitude of the convention when a caller wants one.

use core::fmt;

use serde::Serialize;
use teistro_core::angle::difference_deg;
use teistro_core::error::{Error, Status};
use teistro_core::quantity::{JulianDay, Place, Ut1};
use teistro_port_ephemeris::{Body, DiscPoint, Horizon, HorizonEventKind, Refraction};

use crate::delta_t::DeltaTModel;
use crate::iau::{DEG2RAD, RAD2DEG};
use crate::scale::tt_of;
use crate::sky::{ApparentPositions, sidereal_time_deg};
use crate::solve::{Caps, SolveError, first_zero};

/// The standard refraction at the horizon, arcminutes: the value the
/// almanacs reckon rising and setting by (*Astronomical Almanac*,
/// section A; Meeus, chapter 15).
pub const STANDARD_HORIZON_REFRACTION_ARCMIN: f64 = 34.0;
/// The Earth's equatorial radius, kilometres (WGS 84), for the horizontal
/// parallax.
pub const EARTH_EQUATORIAL_RADIUS_KM: f64 = 6_378.137;
/// The astronomical unit, kilometres (IAU 2012 Resolution B2).
pub const AU_KM: f64 = 149_597_870.7;
/// How fast the hour angle grows, degrees per day of UT1: a rotation plus
/// the day's share of the year.
pub const HOUR_ANGLE_RATE_DEG_PER_DAY: f64 = 360.985_647;
/// The tolerance an event is found to, days: under a hundredth of a
/// second, inside `f64`'s resolution and far inside any convention.
pub const TOLERANCE_DAYS: f64 = 1e-7;
/// The scan step of the fallback search, days: ten minutes, so that a
/// body above the horizon for a quarter of an hour is still seen.
const SCAN_STEP_DAYS: f64 = 1.0 / 144.0;
/// The scan's caps: a day of ten-minute steps, and the shared bisection.
const SCAN_CAPS: Caps = Caps {
    bracket_steps: 400,
    refinements: 64,
};
/// Passes of the iteration before the scan takes over.
const MAX_ITERATIONS: u32 = 12;
/// Below this value of the altitude's rate factor (cos δ cos φ sin H) the
/// iteration's correction is not trusted and the scan answers.
const MIN_RATE_FACTOR: f64 = 1e-3;

/// The mean radius of a body, kilometres, for its apparent semidiameter:
/// the Sun's nominal radius (IAU 2015 Resolution B3), the others from the
/// IAU Working Group on Cartographic Coordinates (Archinal et al., 2018);
/// the lunar points have no disc.
#[must_use]
pub fn radius_km(body: Body) -> f64 {
    match body {
        Body::Sun => 695_700.0,
        Body::Moon => 1_737.4,
        Body::Mercury => 2_439.7,
        Body::Venus => 6_051.8,
        Body::Mars => 3_389.5,
        Body::Jupiter => 69_911.0,
        Body::Saturn => 58_232.0,
        Body::Uranus => 25_362.0,
        Body::Neptune => 24_622.0,
        Body::Pluto => 1_188.3,
        _ => 0.0,
    }
}

/// The apparent size and the horizontal parallax of a body at a distance,
/// degrees.
///
/// ```
/// use teistro_astro::rise_set::Disc;
/// use teistro_port_ephemeris::Body;
///
/// let sun = Disc::of(Body::Sun, 1.0);
/// // 959.23″ from the IAU 2015 nominal radius; the almanacs' older 959.63″
/// // came from a radius of 696 000 km.
/// assert!((sun.semidiameter_deg * 3600.0 - 959.23).abs() < 0.01);
/// assert!((sun.parallax_deg * 3600.0 - 8.794).abs() < 0.01);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Disc {
    /// The semidiameter, degrees.
    pub semidiameter_deg: f64,
    /// The horizontal parallax, degrees.
    pub parallax_deg: f64,
}

impl Disc {
    /// The disc of a body at a distance in astronomical units; a point
    /// (a node, an apogee) has neither size nor parallax.
    #[must_use]
    pub fn of(body: Body, distance_au: f64) -> Disc {
        if !body.has_distance() || distance_au <= 0.0 {
            return Disc {
                semidiameter_deg: 0.0,
                parallax_deg: 0.0,
            };
        }
        let distance_km = distance_au * AU_KM;
        Disc {
            semidiameter_deg: (radius_km(body) / distance_km).min(1.0).asin() * RAD2DEG,
            parallax_deg: (EARTH_EQUATORIAL_RADIUS_KM / distance_km).min(1.0).asin() * RAD2DEG,
        }
    }
}

/// The geocentric altitude of the body's centre at the event under a
/// convention, degrees: the disc point's target altitude, less the
/// refraction, less the semidiameter for the upper limb (plus it for the
/// lower), plus the horizontal parallax.
///
/// ```
/// use teistro_astro::rise_set::{centre_altitude_deg, Disc};
/// use teistro_port_ephemeris::{Body, Horizon};
///
/// let sun = Disc::of(Body::Sun, 1.0);
/// let almanac = centre_altitude_deg(&Horizon::UPPER_LIMB_REFRACTION, &sun);
/// assert!((almanac + 50.0 / 60.0).abs() < 0.01); // about -0°50′, Meeus's value
/// assert_eq!(centre_altitude_deg(&Horizon::CENTRE_NO_REFRACTION, &sun), sun.parallax_deg);
/// ```
#[must_use]
pub fn centre_altitude_deg(horizon: &Horizon, disc: &Disc) -> f64 {
    let refraction = match horizon.refraction {
        Refraction::Standard => STANDARD_HORIZON_REFRACTION_ARCMIN / 60.0,
        Refraction::None => 0.0,
    };
    let limb = match horizon.disc {
        DiscPoint::Centre => 0.0,
        DiscPoint::UpperLimb => -disc.semidiameter_deg,
        DiscPoint::LowerLimb => disc.semidiameter_deg,
    };
    horizon.altitude_deg - refraction + limb + disc.parallax_deg
}

/// Which method answered.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Method {
    /// The hour-angle estimate iterated to convergence.
    Iterated,
    /// The window scanned for the altitude's sign change and bisected.
    Scanned,
}

/// A found horizon event.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct HorizonEvent {
    /// The instant, UT1.
    pub instant: JulianDay<Ut1>,
    /// Which method answered.
    pub method: Method,
    /// How many apparent positions were read.
    pub evaluations: u32,
}

/// The rise of a body inside one day and the set that follows it (the
/// day's arc), and whether it stood above the horizon at the day's
/// middle, which names a day without an arc as a polar day or a polar
/// night.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct DayEvents {
    /// The rise, when there is one.
    pub rise: Option<HorizonEvent>,
    /// The set, when there is one.
    pub set: Option<HorizonEvent>,
    /// Whether the body was above the event altitude at the middle of the
    /// day.
    pub above_at_midday: bool,
}

impl DayEvents {
    /// The sunrise and the sunset when the day has both, in order.
    #[must_use]
    pub fn arc(&self) -> Option<(JulianDay<Ut1>, JulianDay<Ut1>)> {
        match (self.rise, self.set) {
            (Some(rise), Some(set)) if rise.instant.get() <= set.instant.get() => {
                Some((rise.instant, set.instant))
            }
            _ => None,
        }
    }
}

/// What a search failed with.
#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    /// The event was found.
    Found(HorizonEvent),
    /// No such event inside the window.
    Absent,
}

/// One reading of the sky at an instant, in the solver's own terms.
#[allow(
    clippy::struct_field_names,
    reason = "each field is a quantity in degrees and says so"
)]
struct Sample {
    /// The body's altitude above the horizon, degrees, geocentric.
    altitude_deg: f64,
    /// The altitude the event is at, degrees.
    target_deg: f64,
    /// The hour angle, degrees, in (-180, 180].
    hour_angle_deg: f64,
    /// The declination, degrees.
    dec_deg: f64,
}

/// The solver over a source of apparent positions, for one body at one
/// place under one horizon convention.
///
/// ```
/// use teistro_astro::rise_set::Solver;
/// use teistro_astro::{Completion, DeltaTModel};
/// use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
/// use teistro_core::settings::OverridePolicy;
/// use teistro_port_ephemeris::{Body, Horizon, TestProvider};
///
/// let provider = TestProvider::new();
/// let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
/// let solver = Solver::new(&sky, Body::Sun, kathmandu, Horizon::CENTRE_NO_REFRACTION, DeltaTModel::TableThenModel);
/// // The local mean day of 2024-06-21 at Kathmandu.
/// let midnight = JulianDay::literal(2_460_482.5 - 85.324 / 360.0);
/// let day = solver.day(midnight).expect("the test provider answers");
/// let (rise, set) = day.arc().expect("a sunrise and a sunset");
/// assert!(set.get() - rise.get() > 0.5 && set.get() - rise.get() < 0.6);
/// ```
pub struct Solver<'a> {
    sky: &'a dyn ApparentPositions,
    body: Body,
    place: Place,
    horizon: Horizon,
    delta_t: DeltaTModel,
}

impl fmt::Debug for Solver<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.describe())
    }
}

impl<'a> Solver<'a> {
    /// A solver.
    pub const fn new(
        sky: &'a dyn ApparentPositions,
        body: Body,
        place: Place,
        horizon: Horizon,
        delta_t: DeltaTModel,
    ) -> Solver<'a> {
        Solver {
            sky,
            body,
            place,
            horizon,
            delta_t,
        }
    }

    /// The convention.
    #[must_use]
    pub const fn horizon(&self) -> Horizon {
        self.horizon
    }

    /// The stamp: body, convention, place and source.
    #[must_use]
    pub fn describe(&self) -> String {
        format!(
            "{} {} at {} from {}",
            self.body.key(),
            self.horizon,
            self.place,
            self.sky.describe()
        )
    }

    /// The geocentric altitude of the body's centre above the horizon at
    /// an instant, degrees, as the solver reads it: what a visibility
    /// criterion compares the Sun's depression against.
    ///
    /// # Errors
    ///
    /// The source's refusal, or a Delta T model that cannot answer.
    pub fn altitude_deg(&self, ut1: JulianDay<Ut1>) -> Result<f64, Error> {
        let mut evaluations = 0;
        Ok(self.sample(ut1.get(), &mut evaluations)?.altitude_deg)
    }

    /// The sky at an instant: altitude, target, hour angle, declination.
    fn sample(&self, t: f64, evaluations: &mut u32) -> Result<Sample, Error> {
        let ut1 = JulianDay::<Ut1>::try_new(t)?;
        *evaluations += 1;
        let apparent = self.sky.apparent(self.body, ut1)?;
        let (tt, _) = tt_of(ut1, self.delta_t)?;
        let sidereal = sidereal_time_deg(ut1, tt, self.place.longitude);
        let hour_angle_deg = difference_deg(sidereal, apparent.ra_deg);
        let (sin_phi, cos_phi) = (self.place.latitude.get() * DEG2RAD).sin_cos();
        let (sin_dec, cos_dec) = (apparent.dec_deg * DEG2RAD).sin_cos();
        let sin_alt = sin_phi * sin_dec + cos_phi * cos_dec * (hour_angle_deg * DEG2RAD).cos();
        Ok(Sample {
            altitude_deg: sin_alt.clamp(-1.0, 1.0).asin() * RAD2DEG,
            target_deg: centre_altitude_deg(
                &self.horizon,
                &Disc::of(self.body, apparent.distance_au),
            ),
            hour_angle_deg,
            dec_deg: apparent.dec_deg,
        })
    }

    /// The hour angle at which the body stands at the event altitude, for
    /// a rise (negative) or a set (positive), from one sample; `None`
    /// where the body never reaches it that day.
    fn event_hour_angle_deg(&self, kind: HorizonEventKind, sample: &Sample) -> Option<f64> {
        match kind {
            HorizonEventKind::Transit => Some(0.0),
            HorizonEventKind::Antitransit => Some(180.0),
            HorizonEventKind::Rise | HorizonEventKind::Set => {
                let (sin_phi, cos_phi) = (self.place.latitude.get() * DEG2RAD).sin_cos();
                let (sin_dec, cos_dec) = (sample.dec_deg * DEG2RAD).sin_cos();
                let denominator = cos_phi * cos_dec;
                if denominator.abs() < f64::EPSILON {
                    return None;
                }
                let cos_h = ((sample.target_deg * DEG2RAD).sin() - sin_phi * sin_dec) / denominator;
                if !(-1.0..=1.0).contains(&cos_h) {
                    return None;
                }
                let h = cos_h.acos() * RAD2DEG;
                Some(if kind == HorizonEventKind::Rise {
                    -h
                } else {
                    h
                })
            }
        }
    }

    /// The next event of a kind at or after `from`, inside a window of so
    /// many days, or `None` when there is none inside it (a polar day or
    /// night, a circumpolar body).
    ///
    /// # Errors
    ///
    /// The source's refusal, a Delta T model that cannot answer, or a
    /// scan that does not converge (`NOT_CONVERGED`).
    pub fn event(
        &self,
        kind: HorizonEventKind,
        from: JulianDay<Ut1>,
        window_days: f64,
    ) -> Result<Option<HorizonEvent>, Error> {
        if !(window_days.is_finite() && window_days > 0.0) {
            return Err(Error::invalid_arg(format!(
                "a search window is a positive number of days, not {window_days}"
            ))
            .with_field("window_days"));
        }
        let start = from.get();
        let end = start + window_days;
        let mut evaluations = 0u32;
        match self.iterate(kind, start, end, &mut evaluations)? {
            Some(instant) => Ok(Some(HorizonEvent {
                instant: JulianDay::try_new(instant)?,
                method: Method::Iterated,
                evaluations,
            })),
            None => self.scan(kind, start, end, evaluations),
        }
    }

    /// Meeus's iteration from the hour-angle estimate: `Some(instant)`
    /// when it converged inside the window, `Ok(None)` when it did not
    /// settle or its rate factor vanished, which hands the search to the
    /// scan.
    fn iterate(
        &self,
        kind: HorizonEventKind,
        start: f64,
        end: f64,
        evaluations: &mut u32,
    ) -> Result<Option<f64>, Error> {
        let sidereal_day = 360.0 / HOUR_ANGLE_RATE_DEG_PER_DAY;
        let first = self.sample(start, evaluations)?;
        let Some(target_hour_angle) = self.event_hour_angle_deg(kind, &first) else {
            return Ok(None);
        };
        // The first instant at which the hour angle reaches the target,
        // at or after the start.
        let advance = (target_hour_angle - first.hour_angle_deg).rem_euclid(360.0);
        let mut t = start + advance / HOUR_ANGLE_RATE_DEG_PER_DAY;
        if t > end + sidereal_day {
            return Ok(None);
        }
        let mut retried = false;
        for _ in 0..MAX_ITERATIONS {
            let sample = self.sample(t, evaluations)?;
            let correction = match kind {
                HorizonEventKind::Transit | HorizonEventKind::Antitransit => {
                    difference_deg(target_hour_angle, sample.hour_angle_deg)
                        / HOUR_ANGLE_RATE_DEG_PER_DAY
                }
                HorizonEventKind::Rise | HorizonEventKind::Set => {
                    let (_, cos_phi) = (self.place.latitude.get() * DEG2RAD).sin_cos();
                    let cos_dec = (sample.dec_deg * DEG2RAD).cos();
                    let factor = cos_dec * cos_phi * (sample.hour_angle_deg * DEG2RAD).sin();
                    if factor.abs() < MIN_RATE_FACTOR {
                        return Ok(None);
                    }
                    (sample.altitude_deg - sample.target_deg)
                        / (HOUR_ANGLE_RATE_DEG_PER_DAY * factor)
                }
            };
            t += correction;
            if correction.abs() < TOLERANCE_DAYS {
                if t < start - TOLERANCE_DAYS && !retried {
                    // The event settled just before the window: the next
                    // one is a sidereal day on.
                    retried = true;
                    t += sidereal_day;
                    continue;
                }
                if t < start - TOLERANCE_DAYS || t > end + TOLERANCE_DAYS {
                    return Ok(None);
                }
                return Ok(Some(t));
            }
        }
        Ok(None)
    }

    /// The scan: the altitude less the target (a rise upward, a set
    /// downward), or the hour angle less the target (a transit), over the
    /// window, bisected where it changes sign.
    fn scan(
        &self,
        kind: HorizonEventKind,
        start: f64,
        end: f64,
        evaluations: u32,
    ) -> Result<Option<HorizonEvent>, Error> {
        let mut count = evaluations;
        let quantity = |t: f64| -> Result<f64, Error> {
            let sample = self.sample(t, &mut count)?;
            Ok(match kind {
                HorizonEventKind::Rise | HorizonEventKind::Set => {
                    sample.altitude_deg - sample.target_deg
                }
                HorizonEventKind::Transit => sample.hour_angle_deg,
                HorizonEventKind::Antitransit => difference_deg(sample.hour_angle_deg, 180.0),
            })
        };
        let upward = kind != HorizonEventKind::Set;
        let crossing = first_zero(
            quantity,
            start,
            end,
            SCAN_STEP_DAYS,
            upward,
            TOLERANCE_DAYS,
            SCAN_CAPS,
        )
        .map_err(|error| match error {
            SolveError::Evaluation(inner) => inner,
            other => Error::new(
                Status::NotConverged,
                format!(
                    "the {} of {} at {} from JD {start} was not found: {other}",
                    kind.key().to_ascii_lowercase(),
                    self.body.key(),
                    self.place
                ),
            ),
        })?;
        Ok(match crossing {
            Some(found) => Some(HorizonEvent {
                instant: JulianDay::try_new(found.instant)?,
                method: Method::Scanned,
                evaluations: count,
            }),
            None => None,
        })
    }

    /// The rise inside the day that begins at `midnight` (a local mean
    /// midnight for a civil day at the place), the set that follows that
    /// rise (the day's arc, which past the polar circles may end after the
    /// next civil midnight), or the set inside the day when there is no
    /// rise, and whether the body stood above the event altitude at the
    /// day's middle.
    ///
    /// # Errors
    ///
    /// As [`Solver::event`].
    pub fn day(&self, midnight: JulianDay<Ut1>) -> Result<DayEvents, Error> {
        let rise = self.event(HorizonEventKind::Rise, midnight, 1.0)?;
        let set = match rise {
            Some(rise) => self.event(HorizonEventKind::Set, rise.instant, 1.0)?,
            None => self.event(HorizonEventKind::Set, midnight, 1.0)?,
        };
        let mut count = 0u32;
        let midday = self.sample(midnight.get() + 0.5, &mut count)?;
        Ok(DayEvents {
            rise,
            set,
            above_at_midday: midday.altitude_deg > midday.target_deg,
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::float_cmp,
        reason = "tests fail by panicking and compare exact zeros"
    )]

    use teistro_core::quantity::{Altitude, Latitude, Longitude};
    use teistro_core::settings::OverridePolicy;
    use teistro_port_ephemeris::TestProvider;

    use super::*;
    use crate::completion::Completion;

    fn place(lat: f64, lon: f64) -> Place {
        Place::new(
            Latitude::literal(lat),
            Longitude::literal(lon),
            Altitude::literal(0.0),
        )
    }

    /// A sky whose one body sits at a fixed declination and a right
    /// ascension that keeps it on the meridian at J2000.0 noon: a star.
    struct FixedStar {
        dec_deg: f64,
    }

    impl ApparentPositions for FixedStar {
        fn apparent(
            &self,
            _body: Body,
            _ut1: JulianDay<Ut1>,
        ) -> Result<crate::sky::Apparent, Error> {
            Ok(crate::sky::Apparent {
                ra_deg: 280.46,
                dec_deg: self.dec_deg,
                distance_au: 0.0,
            })
        }

        fn describe(&self) -> String {
            String::from("a fixed star")
        }
    }

    #[test]
    fn a_star_on_the_equator_rises_and_sets_six_hours_from_transit() {
        let star = FixedStar { dec_deg: 0.0 };
        let solver = Solver::new(
            &star,
            Body::MeanNode,
            place(0.0, 0.0),
            Horizon::CENTRE_NO_REFRACTION,
            DeltaTModel::TableThenModel,
        );
        let from = JulianDay::literal(2_451_545.0 - 0.5);
        let transit = solver
            .event(HorizonEventKind::Transit, from, 1.0)
            .unwrap()
            .unwrap();
        let rise = solver
            .event(HorizonEventKind::Rise, from, 1.0)
            .unwrap()
            .unwrap();
        let set = solver
            .event(HorizonEventKind::Set, from, 1.0)
            .unwrap()
            .unwrap();
        let quarter = 90.0 / HOUR_ANGLE_RATE_DEG_PER_DAY;
        assert!((transit.instant.get() - rise.instant.get() - quarter).abs() < 1e-6);
        assert!((set.instant.get() - transit.instant.get() - quarter).abs() < 1e-6);
        assert_eq!(rise.method, Method::Iterated);
        assert!(rise.evaluations < 8, "{}", rise.evaluations);
        let anti = solver
            .event(HorizonEventKind::Antitransit, from, 1.0)
            .unwrap()
            .unwrap();
        assert!((anti.instant.get() - transit.instant.get()).abs() - 0.5 < 0.01);
        assert!(solver.describe().contains("MEAN_NODE"));
        assert!(solver.event(HorizonEventKind::Rise, from, 0.0).is_err());
    }

    #[test]
    fn a_circumpolar_star_never_sets_and_a_southern_one_never_rises() {
        let star = FixedStar { dec_deg: 80.0 };
        let tromso = place(69.6, 18.9);
        let solver = Solver::new(
            &star,
            Body::MeanNode,
            tromso,
            Horizon::CENTRE_NO_REFRACTION,
            DeltaTModel::TableThenModel,
        );
        let from = JulianDay::literal(2_451_545.0);
        assert!(
            solver
                .event(HorizonEventKind::Set, from, 1.0)
                .unwrap()
                .is_none()
        );
        assert!(
            solver
                .event(HorizonEventKind::Rise, from, 1.0)
                .unwrap()
                .is_none()
        );
        let day = solver.day(from).unwrap();
        assert!(day.rise.is_none() && day.set.is_none() && day.above_at_midday);
        assert!(day.arc().is_none());
        let southern = FixedStar { dec_deg: -80.0 };
        let solver = Solver::new(
            &southern,
            Body::MeanNode,
            tromso,
            Horizon::CENTRE_NO_REFRACTION,
            DeltaTModel::TableThenModel,
        );
        let day = solver.day(from).unwrap();
        assert!(day.rise.is_none() && !day.above_at_midday);
    }

    #[test]
    fn a_grazing_star_is_found_and_the_scan_agrees_with_the_iteration() {
        // At 69.6°N a body at declination 20.35° dips a twentieth of a
        // degree below the horizon: the altitude's rate at the event is
        // small, and the scan, the safety net when it vanishes, must land
        // where the iteration does.
        let star = FixedStar { dec_deg: 20.35 };
        let solver = Solver::new(
            &star,
            Body::MeanNode,
            place(69.6, 18.9),
            Horizon::CENTRE_NO_REFRACTION,
            DeltaTModel::TableThenModel,
        );
        let from = JulianDay::literal(2_451_545.0);
        for kind in [HorizonEventKind::Rise, HorizonEventKind::Set] {
            let iterated = solver.event(kind, from, 1.0).unwrap().unwrap();
            let mut count = 0;
            let sample = solver.sample(iterated.instant.get(), &mut count).unwrap();
            assert!(
                (sample.altitude_deg - sample.target_deg).abs() < 1e-4,
                "{sample:?}"
            );
            let scanned = solver
                .scan(kind, from.get(), from.get() + 1.0, 0)
                .unwrap()
                .unwrap();
            assert_eq!(scanned.method, Method::Scanned);
            assert!(
                (scanned.instant.get() - iterated.instant.get()).abs() < 2.0 * TOLERANCE_DAYS,
                "{kind}: scanned {} iterated {}",
                scanned.instant,
                iterated.instant
            );
            assert!(scanned.evaluations > iterated.evaluations);
        }
        let transit = solver
            .scan(HorizonEventKind::Transit, from.get(), from.get() + 1.0, 0)
            .unwrap()
            .unwrap();
        let iterated = solver
            .event(HorizonEventKind::Transit, from, 1.0)
            .unwrap()
            .unwrap();
        assert!((transit.instant.get() - iterated.instant.get()).abs() < 2.0 * TOLERANCE_DAYS);
    }

    impl fmt::Debug for Sample {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "alt {} target {} H {}",
                self.altitude_deg, self.target_deg, self.hour_angle_deg
            )
        }
    }

    #[test]
    fn the_test_providers_sun_gives_a_kathmandu_day_under_every_convention() {
        let provider = TestProvider::new();
        let sky = Completion::new(
            &provider,
            OverridePolicy::SdkOnly,
            DeltaTModel::TableThenModel,
        );
        let kathmandu = place(27.7172, 85.324);
        let midnight = JulianDay::literal(2_460_482.5 - 85.324 / 360.0);
        let mut previous_rise = f64::NEG_INFINITY;
        for horizon in [
            Horizon::UPPER_LIMB_REFRACTION,
            Horizon::LOWER_LIMB_REFRACTION,
            Horizon::CENTRE_NO_REFRACTION,
        ] {
            let solver = Solver::new(
                &sky,
                Body::Sun,
                kathmandu,
                horizon,
                DeltaTModel::TableThenModel,
            );
            let day = solver.day(midnight).unwrap();
            let (rise, set) = day.arc().unwrap();
            // The upper limb with refraction rises first (the centre 50′
            // down), then the lower limb with refraction (18′ down), and the
            // centre on the geometric horizon last.
            assert!(rise.get() > previous_rise, "{horizon}");
            previous_rise = rise.get();
            assert!(day.above_at_midday);
            assert!(
                set.get() - rise.get() > 0.5,
                "{horizon}: {}",
                set.get() - rise.get()
            );
        }
        let twilight =
            Horizon::from_convention(teistro_core::settings::SunriseConvention::Custom {
                altitude_deg: -6.0,
            });
        let solver = Solver::new(
            &sky,
            Body::Sun,
            kathmandu,
            twilight,
            DeltaTModel::TableThenModel,
        );
        let dawn = solver
            .event(HorizonEventKind::Rise, midnight, 1.0)
            .unwrap()
            .unwrap();
        assert!(dawn.instant.get() < previous_rise);
        assert!(solver.horizon().altitude_deg < 0.0);
    }

    #[test]
    fn discs_and_target_altitudes_follow_the_convention() {
        let moon = Disc::of(Body::Moon, 0.002_57);
        assert!((moon.parallax_deg - 0.951).abs() < 0.01, "{moon:?}");
        assert!((moon.semidiameter_deg / moon.parallax_deg - 0.2724).abs() < 1e-3);
        let point = Disc::of(Body::TrueNode, 1.0);
        assert_eq!(point.semidiameter_deg, 0.0);
        assert_eq!(Disc::of(Body::Sun, 0.0).parallax_deg, 0.0);
        let sun = Disc::of(Body::Sun, 1.0);
        let lower = centre_altitude_deg(&Horizon::LOWER_LIMB_REFRACTION, &sun);
        let upper = centre_altitude_deg(&Horizon::UPPER_LIMB_REFRACTION, &sun);
        assert!((lower - upper - 2.0 * sun.semidiameter_deg).abs() < 1e-12);
        assert_eq!(radius_km(Body::MeanApogee), 0.0);
    }
}
