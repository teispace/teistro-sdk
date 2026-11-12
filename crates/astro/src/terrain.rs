//! The rise and set behind the terrain (`docs/03-design/terrain-horizon.md`,
//! SR-91): a horizon profile, the terrain's apparent elevation by azimuth,
//! and the instants the Sun's and the Moon's limb first clears it and last
//! sinks behind it, beside the astronomical rise and set on a level
//! horizon, which they never replace.
//!
//! The event is the rise and set solver's ([`Solver::with_terrain`]): the
//! profile's elevation at the body's azimuth takes the place of the
//! convention's altitude, and the refraction is reckoned at that apparent
//! altitude rather than at zero.
//!
//! ```
//! use teistro_astro::terrain::{HorizonProfile, HorizonSample, visible_day};
//! use teistro_astro::day_sky::SkyState;
//! use teistro_astro::{Completion, DeltaTModel};
//! use teistro_core::quantity::{JulianDay, Place, Ut1};
//! use teistro_core::settings::OverridePolicy;
//! use teistro_port_ephemeris::{Horizon, TestProvider};
//!
//! let provider = TestProvider::new();
//! let delta_t = DeltaTModel::TableThenModel;
//! let sky = Completion::new(&provider, OverridePolicy::SdkOnly, delta_t);
//! let kathmandu = Place::try_from_degrees(27.7172, 85.324, 1_400.0)?;
//! // Hills two degrees high all round.
//! let hills = HorizonProfile::try_new(vec![HorizonSample { azimuth_deg: 0.0, elevation_deg: 2.0 }])?;
//! // The local mean day of 2024-06-21 there.
//! let midnight = JulianDay::<Ut1>::literal(2_460_482.5 - 85.324 / 360.0);
//! let day = visible_day(&sky, kathmandu, Horizon::UPPER_LIMB_REFRACTION, delta_t, midnight, &hills)?;
//! let (rise, seen) = (day.sun.rise.expect("a sunrise"), day.sun.visible_rise.expect("a visible one"));
//! // The Sun clears the hills some ten minutes after it clears the sea's horizon.
//! let minutes = (seen.at.get() - rise.get()) * 1_440.0;
//! assert!(minutes > 8.0 && minutes < 16.0, "{minutes}");
//! assert_eq!(day.sun.visible_state, SkyState::Crosses);
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

use std::fmt;

use serde::{Deserialize, Serialize};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Place, Ut1};
use teistro_port_ephemeris::{Body, Horizon};

use crate::day_sky::SkyState;
use crate::delta_t::DeltaTModel;
use crate::rise_set::{DayEvents, HorizonEvent, Solver, bennett_refraction_arcmin};
use crate::sky::ApparentPositions;

/// The most samples a profile holds: one every hundredth of a degree,
/// finer than any elevation model resolves at a horizon's distance.
///
/// ```
/// use teistro_astro::terrain::{HorizonProfile, HorizonSample, MAX_HORIZON_SAMPLES};
///
/// // A sample every hundredth of a degree, and one more.
/// let fine = |count: u32| (0..count).map(|i| HorizonSample { azimuth_deg: f64::from(i) / 100.0, elevation_deg: 1.0 }).collect();
/// assert!(HorizonProfile::try_new(fine(36_000)).is_ok());
/// assert_eq!(MAX_HORIZON_SAMPLES, 36_000);
/// // The profile itself is refused, so a record names it by its own field.
/// assert_eq!(HorizonProfile::try_new(fine(36_001)).unwrap_err().field(), None);
/// ```
pub const MAX_HORIZON_SAMPLES: usize = 36_000;

/// The lowest terrain elevation a profile takes, degrees, exclusive: the
/// dip of the sea's horizon from the highest summit is some 3°, and a
/// profile is the terrain's angle, not a depression below it.
///
/// ```
/// use teistro_astro::terrain::{HorizonProfile, HorizonSample, LOWEST_TERRAIN_DEG};
///
/// let at = |elevation_deg| HorizonProfile::try_new(vec![HorizonSample { azimuth_deg: 0.0, elevation_deg }]);
/// assert!(at(LOWEST_TERRAIN_DEG + 0.01).is_ok());
/// assert_eq!(at(LOWEST_TERRAIN_DEG).unwrap_err().field(), Some("[0].elevationDeg"));
/// ```
pub const LOWEST_TERRAIN_DEG: f64 = -5.0;

/// The highest terrain elevation a profile takes, degrees, exclusive: a
/// wall steeper than this hides the sky itself, not the horizon, and no
/// Sun stands behind it for long.
///
/// ```
/// use teistro_astro::terrain::{HIGHEST_TERRAIN_DEG, HorizonProfile, HorizonSample};
///
/// let at = |elevation_deg| HorizonProfile::try_new(vec![HorizonSample { azimuth_deg: 0.0, elevation_deg }]);
/// assert!(at(HIGHEST_TERRAIN_DEG - 0.01).is_ok());
/// assert!(at(HIGHEST_TERRAIN_DEG).is_err());
/// ```
pub const HIGHEST_TERRAIN_DEG: f64 = 45.0;

/// One sample of a horizon profile: the terrain's apparent elevation at
/// an azimuth, as an observer at the place sees it.
///
/// ```
/// use teistro_astro::terrain::HorizonSample;
///
/// let east = HorizonSample { azimuth_deg: 90.0, elevation_deg: 2.0 };
/// assert_eq!(serde_json::to_string(&east)?, r#"{"azimuthDeg":90.0,"elevationDeg":2.0}"#);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HorizonSample {
    /// The azimuth, degrees in `[0, 360)`, from north through east.
    pub azimuth_deg: f64,
    /// The terrain's apparent elevation there, degrees, above −5 and
    /// below 45: the angle the skyline stands at above the level
    /// horizon, as seen, terrestrial refraction and the Earth's curve
    /// included.
    pub elevation_deg: f64,
}

/// A horizon profile: the terrain's apparent elevation by azimuth, as an
/// ordered list of samples, read between them linearly in azimuth and
/// round through north.
///
/// Built only by [`HorizonProfile::try_new`], which refuses a profile
/// with no sample or more than [`MAX_HORIZON_SAMPLES`], an azimuth outside
/// `[0, 360)` or not strictly after the one before, and an elevation
/// outside ([`LOWEST_TERRAIN_DEG`], [`HIGHEST_TERRAIN_DEG`]).
///
/// ```
/// use teistro_astro::terrain::{HorizonProfile, HorizonSample};
///
/// let sample = |azimuth_deg, elevation_deg| HorizonSample { azimuth_deg, elevation_deg };
/// let ridge = HorizonProfile::try_new(vec![sample(80.0, 1.0), sample(100.0, 3.0)])?;
/// assert_eq!(ridge.elevation_at(90.0), 2.0);
/// assert_eq!(ridge.samples().len(), 2);
/// assert_eq!(serde_json::to_string(&ridge)?, r#"[{"azimuthDeg":80.0,"elevationDeg":1.0},{"azimuthDeg":100.0,"elevationDeg":3.0}]"#);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct HorizonProfile {
    samples: Vec<HorizonSample>,
}

impl HorizonProfile {
    /// The profile of `samples`, checked; a refusal names the sample by
    /// its index, as `[2].azimuthDeg`, so a reader of a record names it
    /// under the record's own field ([`Error::under`]).
    ///
    /// ```
    /// use teistro_astro::terrain::{HorizonProfile, HorizonSample};
    ///
    /// let sample = |azimuth_deg, elevation_deg| HorizonSample { azimuth_deg, elevation_deg };
    /// let field = |samples| HorizonProfile::try_new(samples).unwrap_err().field().map(str::to_owned);
    /// assert_eq!(field(vec![]), None);
    /// assert_eq!(field(vec![sample(360.0, 0.0)]).as_deref(), Some("[0].azimuthDeg"));
    /// assert_eq!(field(vec![sample(10.0, 0.0), sample(10.0, 1.0)]).as_deref(), Some("[1].azimuthDeg"));
    /// assert_eq!(field(vec![sample(20.0, 0.0), sample(10.0, 1.0)]).as_deref(), Some("[1].azimuthDeg"));
    /// assert_eq!(field(vec![sample(10.0, f64::NAN)]).as_deref(), Some("[0].elevationDeg"));
    /// assert!(HorizonProfile::try_new(vec![sample(0.0, 0.0), sample(359.9, 2.0)]).is_ok());
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG`, naming the sample and its field, or no field (the
    /// profile itself) for one with no sample or too many.
    pub fn try_new(samples: Vec<HorizonSample>) -> Result<HorizonProfile, Error> {
        if samples.is_empty() {
            return Err(Error::invalid_arg(
                "a horizon profile holds at least one sample of the terrain's elevation",
            )
            .with_hint(
                "a level horizon is one sample: [{\"azimuthDeg\": 0, \"elevationDeg\": 0}]",
            ));
        }
        if samples.len() > MAX_HORIZON_SAMPLES {
            return Err(Error::invalid_arg(format!(
                "a horizon profile holds at most {MAX_HORIZON_SAMPLES} samples, not {}",
                samples.len()
            )));
        }
        let mut before: Option<f64> = None;
        for (index, sample) in samples.iter().enumerate() {
            let azimuth = sample.azimuth_deg;
            if !(azimuth.is_finite() && (0.0..360.0).contains(&azimuth)) {
                return Err(Error::invalid_arg(format!(
                    "an azimuth is degrees in [0, 360), not {azimuth}"
                ))
                .with_field(format!("[{index}].azimuthDeg")));
            }
            if let Some(last) = before.filter(|last| azimuth <= *last) {
                return Err(Error::invalid_arg(format!(
                    "a profile's azimuths run strictly upward from north, and {azimuth} follows {last}"
                ))
                .with_field(format!("[{index}].azimuthDeg"))
                .with_hint("sort the samples by azimuth and keep one per azimuth"));
            }
            before = Some(azimuth);
            let elevation = sample.elevation_deg;
            if !(elevation.is_finite()
                && elevation > LOWEST_TERRAIN_DEG
                && elevation < HIGHEST_TERRAIN_DEG)
            {
                return Err(Error::invalid_arg(format!(
                    "a terrain's elevation is degrees above {LOWEST_TERRAIN_DEG} and below {HIGHEST_TERRAIN_DEG}, not {elevation}"
                ))
                .with_field(format!("[{index}].elevationDeg")));
            }
        }
        Ok(HorizonProfile { samples })
    }

    /// The samples, in azimuth order.
    ///
    /// ```
    /// use teistro_astro::terrain::{HorizonProfile, HorizonSample};
    ///
    /// let level = HorizonProfile::try_new(vec![HorizonSample { azimuth_deg: 0.0, elevation_deg: 0.0 }])?;
    /// assert_eq!(level.samples()[0].elevation_deg, 0.0);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn samples(&self) -> &[HorizonSample] {
        &self.samples
    }

    /// The terrain's elevation at an azimuth, degrees: the samples'
    /// elevation read linearly between the two either side of it, the
    /// last and the first joined through north; one sample is a level
    /// skyline at its elevation. An azimuth outside `[0, 360)` is read
    /// round the circle.
    ///
    /// ```
    /// use teistro_astro::terrain::{HorizonProfile, HorizonSample};
    ///
    /// let sample = |azimuth_deg, elevation_deg| HorizonSample { azimuth_deg, elevation_deg };
    /// // Two samples either side of north: the profile wraps through it.
    /// let north = HorizonProfile::try_new(vec![sample(10.0, 4.0), sample(350.0, 2.0)])?;
    /// assert!((north.elevation_at(0.0) - 3.0).abs() < 1e-12);
    /// assert!((north.elevation_at(360.0) - 3.0).abs() < 1e-12);
    /// assert_eq!(north.elevation_at(10.0), 4.0);
    /// assert_eq!(north.elevation_at(180.0), 3.0);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn elevation_at(&self, azimuth_deg: f64) -> f64 {
        let azimuth = azimuth_deg.rem_euclid(360.0);
        // The first sample past the azimuth; the segment ends there.
        let after = self
            .samples
            .partition_point(|sample| sample.azimuth_deg <= azimuth);
        let (from, to, span) = match (after.checked_sub(1), self.samples.get(after)) {
            (Some(before), Some(next)) => match self.samples.get(before) {
                Some(from) => (from, next, next.azimuth_deg - from.azimuth_deg),
                None => return self.level(),
            },
            // Before the first sample or past the last: the segment
            // through north, from the last to the first.
            _ => match (self.samples.last(), self.samples.first()) {
                (Some(last), Some(first)) => {
                    (last, first, first.azimuth_deg + 360.0 - last.azimuth_deg)
                }
                _ => return self.level(),
            },
        };
        let along = (azimuth - from.azimuth_deg).rem_euclid(360.0) / span;
        along.mul_add(to.elevation_deg - from.elevation_deg, from.elevation_deg)
    }

    /// The first sample's elevation: what a lookup answers where it found
    /// no segment, which a checked profile never leaves it.
    fn level(&self) -> f64 {
        self.samples
            .first()
            .map_or(0.0, |sample| sample.elevation_deg)
    }
}

/// A profile is named by its sample count, as a solver's description and
/// an answer's applied convention name it.
///
/// ```
/// use teistro_astro::terrain::{HorizonProfile, HorizonSample};
///
/// let sample = |azimuth_deg| HorizonSample { azimuth_deg, elevation_deg: 1.0 };
/// assert_eq!(HorizonProfile::try_new(vec![sample(0.0)])?.to_string(), "1 sample");
/// assert_eq!(HorizonProfile::try_new(vec![sample(0.0), sample(90.0)])?.to_string(), "2 samples");
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
impl fmt::Display for HorizonProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let count = self.samples.len();
        write!(f, "{count} sample{}", if count == 1 { "" } else { "s" })
    }
}

/// What the refraction at the horizon is multiplied by at an apparent
/// altitude: Bennett's refraction there over Bennett's at zero (Meeus,
/// eq. 16.4), so a convention's own horizon refraction — the almanac's
/// 34′, or an air's — carries its scale to a skyline above it.
///
/// Below zero it is one: Bennett's formula is fitted to apparent
/// altitudes from 0° to 90°, and under the level horizon the ray's path
/// runs through air below the observer that a profile does not describe,
/// so the horizon's own refraction is held (`terrain-horizon.md` §3).
///
/// ```
/// use teistro_astro::terrain::refraction_scale;
///
/// assert_eq!(refraction_scale(0.0), 1.0);
/// assert_eq!(refraction_scale(-2.0), 1.0);
/// // At two degrees the air bends the ray about half as much as at the horizon.
/// assert!((refraction_scale(2.0) - 0.528).abs() < 0.001, "{}", refraction_scale(2.0));
/// ```
#[must_use]
pub fn refraction_scale(apparent_deg: f64) -> f64 {
    if apparent_deg <= 0.0 {
        1.0
    } else {
        bennett_refraction_arcmin(apparent_deg) / bennett_refraction_arcmin(0.0)
    }
}

/// A visible rise or set: its instant, and the azimuth and the terrain's
/// elevation the limb met it at.
///
/// ```
/// use teistro_astro::terrain::VisibleEvent;
/// use teistro_core::quantity::JulianDay;
///
/// let seen = VisibleEvent { at: JulianDay::literal(2_460_482.5), azimuth_deg: 62.5, terrain_deg: 2.0 };
/// assert_eq!(serde_json::to_value(seen)?["terrainDeg"], 2.0);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct VisibleEvent {
    /// The instant, UT1 read as UTC.
    pub at: JulianDay<Ut1>,
    /// The body's azimuth then, degrees in `[0, 360)`, from north
    /// through east.
    pub azimuth_deg: f64,
    /// The profile's elevation at that azimuth, degrees.
    pub terrain_deg: f64,
}

/// One body's day on a level horizon and behind the terrain.
///
/// `rise` and `set` are the astronomical events under the day's own
/// convention, as [`Solver::day`] finds them: the rise inside the civil
/// day and the set after it. `visible_rise` and `visible_set` are the
/// same search with the profile as the horizon, and `visible_state` says
/// why a day has neither: a body that never clears the terrain is
/// `ALWAYS_BELOW`, one that never sinks behind it `ALWAYS_ABOVE`.
///
/// ```
/// use teistro_astro::day_sky::SkyState;
/// use teistro_astro::terrain::VisibleBody;
///
/// let hidden = VisibleBody { rise: None, set: None, visible_rise: None, visible_set: None, visible_state: SkyState::AlwaysBelow };
/// assert_eq!(serde_json::to_value(hidden)?["visibleState"], "ALWAYS_BELOW");
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct VisibleBody {
    /// The astronomical rise, on the level horizon.
    pub rise: Option<JulianDay<Ut1>>,
    /// The astronomical set that follows it, or the day's when there is
    /// no rise.
    pub set: Option<JulianDay<Ut1>>,
    /// The first instant the limb clears the terrain.
    pub visible_rise: Option<VisibleEvent>,
    /// The first instant after it the limb sinks behind the terrain.
    pub visible_set: Option<VisibleEvent>,
    /// Whether the day holds a visible rise or set, and if not, which
    /// side of the terrain the body stayed on.
    pub visible_state: SkyState,
}

/// The Sun's and the Moon's day behind a terrain, from a local midnight.
///
/// ```
/// use teistro_astro::day_sky::SkyState;
/// use teistro_astro::terrain::{VisibleBody, VisibleDay};
/// use teistro_core::quantity::JulianDay;
///
/// let hidden = VisibleBody { rise: None, set: None, visible_rise: None, visible_set: None, visible_state: SkyState::AlwaysBelow };
/// let day = VisibleDay { from: JulianDay::literal(2_460_482.5), sun: hidden, moon: hidden };
/// let written = serde_json::to_value(day)?;
/// assert_eq!(written["sun"]["visibleRise"], serde_json::Value::Null);
/// assert_eq!(written["moon"]["visibleState"], "ALWAYS_BELOW");
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct VisibleDay {
    /// The midnight the day's search opens at, UT1 read as UTC.
    pub from: JulianDay<Ut1>,
    /// The Sun.
    pub sun: VisibleBody,
    /// The Moon.
    pub moon: VisibleBody,
}

/// The Sun's and the Moon's rise and set inside the day that begins at
/// `midnight`, on the level horizon under `horizon` and behind the
/// terrain of `profile` under the same convention's limb and air.
///
/// ```
/// use teistro_astro::terrain::{HorizonProfile, HorizonSample, visible_day};
/// use teistro_astro::day_sky::SkyState;
/// use teistro_astro::{Completion, DeltaTModel};
/// use teistro_core::quantity::{JulianDay, Place, Ut1};
/// use teistro_core::settings::OverridePolicy;
/// use teistro_port_ephemeris::{Horizon, TestProvider};
///
/// let provider = TestProvider::new();
/// let delta_t = DeltaTModel::TableThenModel;
/// let sky = Completion::new(&provider, OverridePolicy::SdkOnly, delta_t);
/// let kathmandu = Place::try_from_degrees(27.7172, 85.324, 1_400.0)?;
/// // A wall of 44 degrees: the midsummer Sun at Kathmandu culminates near 86,
/// // so it clears it; the hidden day is the one it never does.
/// let wall = HorizonProfile::try_new(vec![HorizonSample { azimuth_deg: 0.0, elevation_deg: 44.0 }])?;
/// let midnight = JulianDay::<Ut1>::literal(2_460_482.5 - 85.324 / 360.0);
/// let day = visible_day(&sky, kathmandu, Horizon::UPPER_LIMB_REFRACTION, delta_t, midnight, &wall)?;
/// assert_eq!(day.sun.visible_state, SkyState::Crosses);
/// assert!(day.sun.visible_rise.expect("over the wall").at.get() > day.sun.rise.expect("a sunrise").get());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// As [`Solver::day`].
pub fn visible_day(
    source: &dyn ApparentPositions,
    place: Place,
    horizon: Horizon,
    delta_t: DeltaTModel,
    midnight: JulianDay<Ut1>,
    profile: &HorizonProfile,
) -> Result<VisibleDay, Error> {
    let body = |body: Body| -> Result<VisibleBody, Error> {
        let level = Solver::new(source, body, place, horizon, delta_t).day(midnight)?;
        let terrain = Solver::new(source, body, place, horizon, delta_t).with_terrain(profile);
        let seen = terrain.day(midnight)?;
        let visible = |event: Option<HorizonEvent>| -> Result<Option<VisibleEvent>, Error> {
            event
                .map(|event| {
                    let azimuth_deg = terrain.azimuth_deg(event.instant)?;
                    Ok(VisibleEvent {
                        at: event.instant,
                        azimuth_deg,
                        terrain_deg: profile.elevation_at(azimuth_deg),
                    })
                })
                .transpose()
        };
        Ok(VisibleBody {
            rise: level.rise.map(|rise| rise.instant),
            set: level.set.map(|set| set.instant),
            visible_rise: visible(seen.rise)?,
            visible_set: visible(seen.set)?,
            visible_state: state_of(&seen),
        })
    };
    Ok(VisibleDay {
        from: midnight,
        sun: body(Body::Sun)?,
        moon: body(Body::Moon)?,
    })
}

/// Whether a day crossed its horizon, and if not which side it stayed on.
fn state_of(day: &DayEvents) -> SkyState {
    if day.rise.is_some() || day.set.is_some() {
        SkyState::Crosses
    } else if day.above_at_midday {
        SkyState::AlwaysAbove
    } else {
        SkyState::AlwaysBelow
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::float_cmp,
        reason = "tests fail by panicking and compare values they wrote"
    )]

    use teistro_core::quantity::{Altitude, Latitude, Longitude};
    use teistro_core::settings::OverridePolicy;
    use teistro_port_ephemeris::{HorizonEventKind, TestProvider};

    use super::*;
    use crate::completion::Completion;
    use crate::rise_set::HOUR_ANGLE_RATE_DEG_PER_DAY;
    use crate::sky::Apparent;

    fn sample(azimuth_deg: f64, elevation_deg: f64) -> HorizonSample {
        HorizonSample {
            azimuth_deg,
            elevation_deg,
        }
    }

    fn level(elevation_deg: f64) -> HorizonProfile {
        HorizonProfile::try_new(vec![sample(0.0, elevation_deg)]).unwrap()
    }

    fn place(lat: f64, lon: f64) -> Place {
        Place::new(
            Latitude::literal(lat),
            Longitude::literal(lon),
            Altitude::literal(0.0),
        )
    }

    /// A star on the equator, on the meridian at J2000.0 noon: seen from
    /// the equator it rises due east, straight up, at the Earth's rate.
    struct EquatorStar;

    impl ApparentPositions for EquatorStar {
        fn apparent(&self, _body: Body, _ut1: JulianDay<Ut1>) -> Result<Apparent, Error> {
            Ok(Apparent {
                ra_deg: 280.46,
                dec_deg: 0.0,
                distance_au: 0.0,
            })
        }

        fn describe(&self) -> String {
            String::from("a star on the equator")
        }
    }

    #[test]
    fn a_star_rising_straight_up_clears_a_ridge_by_the_ridge_over_the_turn() {
        // Altitude = 90° − |H| exactly, so a ridge of e degrees in the east
        // is reached e / ω later, with no refraction to bend it.
        fn solver<'a>(star: &'a EquatorStar, terrain: Option<&'a HorizonProfile>) -> Solver<'a> {
            let solver = Solver::new(
                star,
                Body::MeanNode,
                place(0.0, 0.0),
                Horizon::CENTRE_NO_REFRACTION,
                DeltaTModel::TableThenModel,
            );
            match terrain {
                Some(profile) => solver.with_terrain(profile),
                None => solver,
            }
        }
        let star = EquatorStar;
        let from = JulianDay::<Ut1>::literal(2_451_545.0 - 0.5);
        let rise = |terrain: Option<&HorizonProfile>| {
            solver(&star, terrain)
                .event(HorizonEventKind::Rise, from, 1.0)
                .unwrap()
                .unwrap()
                .instant
                .get()
        };
        // Only the east is raised: a ridge from 80° to 100°, 3° high.
        let ridge = HorizonProfile::try_new(vec![
            sample(70.0, 0.0),
            sample(80.0, 3.0),
            sample(100.0, 3.0),
            sample(110.0, 0.0),
        ])
        .unwrap();
        let delay = rise(Some(&ridge)) - rise(None);
        let expected = 3.0 / HOUR_ANGLE_RATE_DEG_PER_DAY;
        assert!(
            (delay - expected).abs() < 1e-6,
            "{delay} against {expected}"
        );
        // The azimuth it met the ridge at.
        let seen = solver(&star, Some(&ridge))
            .azimuth_deg(JulianDay::literal(rise(Some(&ridge))))
            .unwrap();
        assert!((seen - 90.0).abs() < 1e-3, "{seen}");
        // A level profile of zero is the level horizon's rise.
        assert!((rise(Some(&level(0.0))) - rise(None)).abs() < 2e-7);
    }

    #[test]
    fn a_zero_profile_is_the_conventions_own_rise_and_set() {
        let provider = TestProvider::new();
        let sky = Completion::new(
            &provider,
            OverridePolicy::SdkOnly,
            DeltaTModel::TableThenModel,
        );
        let kathmandu = place(27.7172, 85.324);
        let midnight = JulianDay::<Ut1>::literal(2_460_482.5 - 85.324 / 360.0);
        for horizon in [
            Horizon::UPPER_LIMB_REFRACTION,
            Horizon::CENTRE_NO_REFRACTION,
            Horizon::LOWER_LIMB_REFRACTION,
        ] {
            let day = visible_day(
                &sky,
                kathmandu,
                horizon,
                DeltaTModel::TableThenModel,
                midnight,
                &level(0.0),
            )
            .unwrap();
            for body in [day.sun, day.moon] {
                let pairs = [(body.rise, body.visible_rise), (body.set, body.visible_set)];
                for (level, seen) in pairs {
                    // A Moon with no rise in the day has none behind a
                    // level skyline either.
                    assert_eq!(level.is_some(), seen.is_some(), "{horizon}");
                    let (Some(level), Some(seen)) = (level, seen) else {
                        continue;
                    };
                    // The iteration and the scan settle to the solver's
                    // tolerance each, from either side.
                    assert!(
                        (seen.at.get() - level.get()).abs() < 2.0 * crate::rise_set::TOLERANCE_DAYS,
                        "{horizon}: {} against {}",
                        seen.at.get(),
                        level.get()
                    );
                    assert_eq!(seen.terrain_deg, 0.0);
                }
            }
        }
    }

    #[test]
    fn a_wall_the_winter_sun_never_tops_hides_the_day() {
        let provider = TestProvider::new();
        let sky = Completion::new(
            &provider,
            OverridePolicy::SdkOnly,
            DeltaTModel::TableThenModel,
        );
        // Oslo at midwinter: the Sun culminates under seven degrees.
        let oslo = place(59.91, 10.75);
        let midnight = JulianDay::<Ut1>::literal(2_460_665.5 - 10.75 / 360.0);
        let day = visible_day(
            &sky,
            oslo,
            Horizon::UPPER_LIMB_REFRACTION,
            DeltaTModel::TableThenModel,
            midnight,
            &level(10.0),
        )
        .unwrap();
        assert!(day.sun.rise.is_some() && day.sun.set.is_some());
        assert_eq!(
            (
                day.sun.visible_rise,
                day.sun.visible_set,
                day.sun.visible_state
            ),
            (None, None, SkyState::AlwaysBelow)
        );
    }

    #[test]
    fn the_midnight_sun_sinks_behind_a_northern_ridge() {
        let provider = TestProvider::new();
        let sky = Completion::new(
            &provider,
            OverridePolicy::SdkOnly,
            DeltaTModel::TableThenModel,
        );
        // Tromsø at midsummer: the Sun stays some three degrees up at
        // midnight, so a level horizon has no set, and a ridge of five
        // degrees round the north hides it for the night.
        let tromso = place(69.65, 18.96);
        let midnight = JulianDay::<Ut1>::literal(2_460_482.5 - 18.96 / 360.0);
        let ridge = HorizonProfile::try_new(vec![
            sample(0.0, 5.0),
            sample(60.0, 5.0),
            sample(90.0, 0.0),
            sample(270.0, 0.0),
            sample(300.0, 5.0),
        ])
        .unwrap();
        let day = visible_day(
            &sky,
            tromso,
            Horizon::UPPER_LIMB_REFRACTION,
            DeltaTModel::TableThenModel,
            midnight,
            &ridge,
        )
        .unwrap();
        assert_eq!((day.sun.rise, day.sun.set), (None, None));
        let rise = day.sun.visible_rise.unwrap();
        let set = day.sun.visible_set.unwrap();
        assert_eq!(day.sun.visible_state, SkyState::Crosses);
        // Out from behind the ridge in the north-east, and behind it again
        // in the north-west the next night.
        assert!(
            rise.azimuth_deg < 60.0 && rise.terrain_deg == 5.0,
            "{rise:?}"
        );
        assert!(set.azimuth_deg > 300.0 && set.terrain_deg == 5.0, "{set:?}");
        assert!(set.at.get() > rise.at.get());
        // The wrap through north is read as one ridge: an azimuth either
        // side of it stands on the ridge's height.
        assert_eq!(ridge.elevation_at(359.0), 5.0);
        assert_eq!(ridge.elevation_at(1.0), 5.0);
    }

    #[test]
    fn a_profile_wraps_through_north() {
        let profile = HorizonProfile::try_new(vec![sample(30.0, 6.0), sample(330.0, 0.0)]).unwrap();
        // From 330° at 0 to 30° (390°) at 6: 3 at north, 1.5 at 345°.
        assert!((profile.elevation_at(0.0) - 3.0).abs() < 1e-12);
        assert!((profile.elevation_at(345.0) - 1.5).abs() < 1e-12);
        assert!((profile.elevation_at(15.0) - 4.5).abs() < 1e-12);
        assert!((profile.elevation_at(-15.0) - 1.5).abs() < 1e-12);
        // Between the two, the long way: 30° to 330° at 6 down to 0.
        assert!((profile.elevation_at(180.0) - 3.0).abs() < 1e-12);
        // A flat profile reads its height to the bit everywhere.
        let flat = HorizonProfile::try_new(vec![sample(0.0, 2.0), sample(180.0, 2.0)]).unwrap();
        for azimuth in [0.0, 0.1, 90.0, 179.9, 180.0, 270.0, 359.99] {
            assert_eq!(flat.elevation_at(azimuth), 2.0);
        }
    }
}
