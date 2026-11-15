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
//! The rest of the horizon is the convention's model (`day.horizon`,
//! [`HorizonModel`]): the formula a named air refracts through (Bennett,
//! Saemundsson or Sinclair), the disc's size and radii, the rising body's
//! latitude, the dip of the horizon from the observer's height (none,
//! geometric, or refracted) and a raised or lowered horizon. Its standard
//! is what the solver has always reckoned: no dip, since the almanacs and
//! the panchanga reckon from sea level.

use core::cell::Cell;
use core::fmt;
use teistro_core::math;

use serde::Serialize;
use teistro_core::angle::difference_deg;
use teistro_core::error::Error;
use teistro_core::quantity::{Altitude, JulianDay, Place, Ut1};
use teistro_core::settings::{
    Atmosphere, Dip, DiscSize, HorizonModel, Radii, RefractionFormula, RisingLatitude,
};
use teistro_port_ephemeris::{
    Body, Conventions, DiscPoint, Horizon, HorizonEventKind, HorizonRequest, Refraction,
};

use crate::day_sky::horizontal;
use crate::delta_t::DeltaTModel;
use crate::iau::{DEG2RAD, RAD2DEG};
use crate::lockstep::{Lockstep, finished};
use crate::scale::tt_of;
use crate::sky::{
    Apparent, ApparentPositions, Spherical, ecliptic_to_equatorial, equatorial_to_ecliptic,
    obliquity, sidereal_time_deg,
};
use crate::solve::{Caps, SCAN_CHUNK, Scan, first_zero_gridded_async};
use crate::terrain::{HorizonProfile, refraction_scale};

/// The standard refraction at the horizon, arcminutes: the value the
/// almanacs reckon rising and setting by (*Astronomical Almanac*,
/// section A; Meeus, chapter 15).
///
/// ```
/// use teistro_astro::rise_set::{STANDARD_HORIZON_REFRACTION_ARCMIN, horizon_refraction_arcmin};
/// use teistro_core::quantity::Altitude;
/// use teistro_port_ephemeris::Refraction;
///
/// // The almanac's refraction, whatever the height.
/// let everest = Altitude::try_new(8_849.0)?;
/// assert_eq!(horizon_refraction_arcmin(Refraction::Standard, everest), STANDARD_HORIZON_REFRACTION_ARCMIN);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
pub const STANDARD_HORIZON_REFRACTION_ARCMIN: f64 = 34.0;
/// The Earth's equatorial radius, kilometres (WGS 84), for the horizontal
/// parallax.
///
/// ```
/// use teistro_astro::rise_set::{AU_KM, Disc, EARTH_EQUATORIAL_RADIUS_KM};
/// use teistro_port_ephemeris::Body;
///
/// // The Moon's horizontal parallax at her mean distance, near a degree.
/// let moon = Disc::of(Body::Moon, 384_400.0 / AU_KM);
/// assert!((moon.parallax_deg - (EARTH_EQUATORIAL_RADIUS_KM / 384_400.0).asin().to_degrees()).abs() < 1e-12);
/// assert!((moon.parallax_deg - 0.951).abs() < 0.001);
/// ```
pub const EARTH_EQUATORIAL_RADIUS_KM: f64 = 6_378.137;
/// The astronomical unit, kilometres (IAU 2012 Resolution B2).
///
/// ```
/// use teistro_astro::rise_set::{AU_KM, Disc};
/// use teistro_port_ephemeris::Body;
///
/// // Jupiter at five au: some twenty arcseconds across.
/// let jupiter = Disc::of(Body::Jupiter, 5.0);
/// assert!((jupiter.semidiameter_deg * 3600.0 - 69_911.0 / (5.0 * AU_KM) * 206_264.8).abs() < 0.01);
/// ```
pub const AU_KM: f64 = 149_597_870.7;
/// The Sun's nominal radius, kilometres (IAU 2015 Resolution B3): the one
/// solar radius the SDK reckons by, for a sunrise's limb and an eclipse's
/// disc alike. The eclipse module once kept the older 696 000 km.
///
/// ```
/// use teistro_astro::rise_set::{SUN_RADIUS_KM, radius_km};
/// use teistro_port_ephemeris::Body;
///
/// assert_eq!(radius_km(Body::Sun), SUN_RADIUS_KM);
/// assert_eq!(teistro_astro::eclipse::SUN_RADIUS_KM, SUN_RADIUS_KM);
/// ```
pub const SUN_RADIUS_KM: f64 = 695_700.0;
/// The Moon's mean radius, kilometres (Archinal et al. 2018).
pub const MOON_RADIUS_KM: f64 = 1_737.4;
/// The Moon's mean distance a disc of fixed size is reckoned at,
/// astronomical units: the Swiss Ephemeris's `SE_BIT_FIXED_DISC_SIZE`
/// value, 0.00257 au (384 463 km), which an engine's mean disc takes too.
pub const MOON_MEAN_DISTANCE_AU: f64 = 0.002_57;
/// The Earth's equatorial radius, metres (WGS 84), for the geometric dip:
/// the tangent to the sphere from the observer's height.
pub const DIP_EARTH_RADIUS_M: f64 = 6_378_137.0;
/// The Earth's equatorial radius, metres (IERS Conventions 2010), for the
/// refracted dip, as the Swiss Ephemeris's `swe_refrac_extended` takes it.
pub const REFRACTED_DIP_EARTH_RADIUS_M: f64 = 6_378_136.6;
/// The standard atmosphere's lapse rate, kelvin per metre, which the
/// refracted dip's refraction coefficient is reckoned from.
pub const LAPSE_RATE_K_PER_M: f64 = 0.0065;
/// The lowest horizon Bennett's formula is read at, degrees: its pole is
/// at −4.4°, so a horizon dipped and lowered past −4° is refused.
pub const BENNETT_LOWEST_DEG: f64 = -4.0;
/// How fast the hour angle grows, degrees per day of UT1: a rotation plus
/// the day's share of the year.
///
/// ```
/// use teistro_astro::rise_set::HOUR_ANGLE_RATE_DEG_PER_DAY;
///
/// // A sidereal day is a turn of the hour angle: four minutes short of the solar day.
/// let sidereal_day_s = 360.0 / HOUR_ANGLE_RATE_DEG_PER_DAY * 86_400.0;
/// assert!((sidereal_day_s - 86_164.09).abs() < 0.01);
/// ```
pub const HOUR_ANGLE_RATE_DEG_PER_DAY: f64 = 360.985_647;
/// The tolerance an event is found to, days: under a hundredth of a
/// second, inside `f64`'s resolution and far inside any convention.
///
/// ```
/// # use teistro_astro::rise_set::Solver;
/// # use teistro_astro::{Completion, DeltaTModel};
/// # use teistro_core::quantity::{JulianDay, Place, Ut1};
/// # use teistro_core::settings::OverridePolicy;
/// # use teistro_port_ephemeris::{Body, Horizon, TestProvider};
/// #
/// # let provider = TestProvider::new();
/// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let place = Place::try_from_degrees(27.7172, 85.324, 0.0)?;
/// # let solver = Solver::new(&sky, Body::Sun, place, Horizon::UPPER_LIMB_REFRACTION, DeltaTModel::TableThenModel);
/// # // The local mean day of 2024-06-21 there.
/// # let midnight = JulianDay::<Ut1>::literal(2_460_482.5 - 85.324 / 360.0);
///
/// use teistro_astro::rise_set::TOLERANCE_DAYS;
/// use teistro_port_ephemeris::HorizonEventKind;
///
/// // The rise is found to within the tolerance of where the altitude crosses.
/// let rise = solver.event(HorizonEventKind::Rise, midnight, 1.0)?.expect("a sunrise");
/// let before = solver.altitude_deg(JulianDay::literal(rise.instant.get() - TOLERANCE_DAYS))?;
/// let after = solver.altitude_deg(JulianDay::literal(rise.instant.get() + TOLERANCE_DAYS))?;
/// assert!(before < after);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
pub const TOLERANCE_DAYS: f64 = 1e-7;
/// The scan step of the fallback search, days: ten minutes, so that a
/// body above the horizon for a quarter of an hour is still seen.
const SCAN_STEP_DAYS: f64 = 1.0 / 144.0;
/// The fewest bracket steps the scan will take, whatever the span: two
/// and three-quarter days of ten-minute steps.
const LEAST_SCAN_STEPS: u32 = 400;
/// The shared bisection's cap, which the span does not change.
const SCAN_REFINEMENTS: u32 = 64;

/// The scan's caps for a span, in days.
///
/// A cap exists to stop a runaway, not to bound the search: a caller that
/// asks for a longer span means it. This was a constant 400 — described
/// as "a day of ten-minute steps", though 400 of them is two and
/// three-quarter days — and a polar day's **synthesised** arc is longer
/// than that, so the scan met the constant rather than the horizon and
/// refused with `NOT_CONVERGED`. A Moon that does not rise at 69.65°N is
/// an answer; a step budget is not. Measured by
/// `panchanga-at-the-boundary-measured.md`, which could not reach a polar
/// day at all until this was sized from the span.
#[must_use]
fn scan_caps(span_days: f64) -> Caps {
    let needed = if span_days.is_finite() && span_days > 0.0 {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a span is finite and positive here, and saturates rather than wraps"
        )]
        let steps = (span_days / SCAN_STEP_DAYS).ceil() as u32;
        steps
    } else {
        0
    };
    Caps {
        bracket_steps: needed.saturating_add(2).max(LEAST_SCAN_STEPS),
        refinements: SCAN_REFINEMENTS,
    }
}
/// Passes of the iteration before the scan takes over.
const MAX_ITERATIONS: u32 = 12;

/// The fastest the altitude of a body less the altitude its rise or set is
/// at can change, degrees a day. The Earth's turn moves an altitude at
/// `ω cos φ sin A`, no more than `ω`, 360.99° a day; a body's own motion
/// moves it no more than its angle across the sky, under 17° a day for
/// the Moon at perigee and less for every other body; and the event
/// altitude moves with the Moon's distance by hundredths of a degree a
/// day. Four hundred and twenty holds all three with room.
const ALTITUDE_RATE_CEILING_DEG_PER_DAY: f64 = 420.0;

/// The most anything but the Earth's turn moves the gap between a body's
/// altitude and its event altitude, degrees a day: the body's own motion
/// across the sky, under 17° a day for the Moon, and the event altitude's
/// drift with its distance.
const OWN_RATE_CEILING_DEG_PER_DAY: f64 = 20.0;

/// How far from the horizon a body counts as low, degrees: within it the
/// slope bound below holds.
const LOW_ALTITUDE_DEG: f64 = 12.0;

/// The most the gap's slope changes within [`LOW_ALTITUDE_DEG`] of the
/// horizon, degrees a day squared. The turn bends the altitude by
/// `(d²sin h + sin h · h′²) / cos h`, under `1.24 ω²` (2 810° a day²)
/// there, and the body's own motion adds a few hundred.
const SLOPE_CHANGE_CEILING_DEG_PER_DAY2: f64 = 5000.0;

/// The longest stretch one slope reading proves, days: so short that a
/// body starting within half of [`LOW_ALTITUDE_DEG`] cannot climb out of
/// it at [`ALTITUDE_RATE_CEILING_DEG_PER_DAY`].
const LOW_STRETCH_DAYS: f64 = LOW_ALTITUDE_DEG / 2.0 / ALTITUDE_RATE_CEILING_DEG_PER_DAY;

/// How near the event altitude a reading may come before
/// [`Solver::proves_none`] stops proving and the walk decides, degrees.
const ONE_SIDE_FLOOR_DEG: f64 = 0.25;

/// The most readings [`Solver::proves_none`] takes before handing the
/// window to the walk. A body far from the horizon needs a handful; one
/// that skims it for hours, as the Moon can near the polar circles, is
/// the walk's.
const ONE_SIDE_READINGS: u32 = 48;
/// Below this value of the altitude's rate factor (cos δ cos φ sin H) the
/// iteration's correction is not trusted and the scan answers.
const MIN_RATE_FACTOR: f64 = 1e-3;

/// How far a body's declination strays from the straight line between
/// two readings a day apart, degrees.
///
/// The companion of [`crate::events::greatest_rate`], one coordinate
/// over, and it exists for one purpose: bounding where a body's altitude
/// can reach over a window, which is how a rise or a set is shown **not
/// to happen** without walking the window to find out
/// (`03-design/horizon-absence-measured.md`).
///
/// **Not the day's motion**, which was tried and is hopeless: the Moon's
/// declination moves six degrees in a day, and a margin that wide puts
/// every temperate latitude inside the range, proves nothing at all, and
/// costs two readings to find that out. It took an almanac from 19 632
/// calls to 20 032. What a margin has to cover is the *stray*, and over
/// a day a smooth curve barely leaves its chord.
///
/// Measured rather than assumed, at seven interior points of every
/// window over thirteen latitudes and thirty-four days; the page checks
/// that these cover what it saw.
///
/// `None` for a body the sweep has not measured, and there the fast path
/// does not apply at all — a bound that might be too small is worse than
/// no bound, so an unmeasured body walks the window as it always did.
///
/// ```
/// use teistro_astro::rise_set::declination_chord_margin;
/// use teistro_port_ephemeris::Body;
///
/// assert_eq!(declination_chord_margin(Body::Moon), Some(0.5));
/// // Unmeasured: its windows are walked.
/// assert_eq!(declination_chord_margin(Body::Venus), None);
/// ```
#[must_use]
pub const fn declination_chord_margin(body: Body) -> Option<f64> {
    match body {
        Body::Sun => Some(0.01),
        Body::Moon => Some(0.5),
        _ => None,
    }
}

/// The mean radius of a body, kilometres, for its apparent semidiameter:
/// the Sun's nominal radius (IAU 2015 Resolution B3), the others from the
/// IAU Working Group on Cartographic Coordinates (Archinal et al., 2018);
/// the lunar points have no disc.
///
/// ```
/// use teistro_astro::rise_set::radius_km;
/// use teistro_port_ephemeris::Body;
///
/// assert_eq!(radius_km(Body::Moon), 1_737.4);
/// // A lunar point has no disc.
/// assert_eq!(radius_km(Body::MeanNode), 0.0);
/// ```
#[must_use]
pub fn radius_km(body: Body) -> f64 {
    match body {
        Body::Sun => SUN_RADIUS_KM,
        Body::Moon => MOON_RADIUS_KM,
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Disc {
    /// The semidiameter, degrees.
    pub semidiameter_deg: f64,
    /// The horizontal parallax, degrees.
    pub parallax_deg: f64,
}

impl Disc {
    /// The disc of a body at a distance in astronomical units; a point
    /// (a node, an apogee) has neither size nor parallax.
    ///
    /// ```
    /// use teistro_astro::rise_set::Disc;
    /// use teistro_port_ephemeris::Body;
    ///
    /// // The Sun at perihelion is larger than at one au.
    /// assert!(Disc::of(Body::Sun, 0.983).semidiameter_deg > Disc::of(Body::Sun, 1.0).semidiameter_deg);
    /// assert_eq!(Disc::of(Body::TrueNode, 0.002).parallax_deg, 0.0);
    /// ```
    #[must_use]
    pub fn of(body: Body, distance_au: f64) -> Disc {
        Disc::under(body, distance_au, &HorizonModel::STANDARD)
    }

    /// The disc of a body at a distance under a horizon model: the radii
    /// it names, and the actual disc, the disc at the body's mean distance
    /// or a fixed semidiameter. The parallax is always the actual one.
    ///
    /// ```
    /// use teistro_astro::rise_set::Disc;
    /// use teistro_core::settings::{DiscSize, HorizonModel, Radii};
    /// use teistro_port_ephemeris::Body;
    ///
    /// let fixed = HorizonModel { disc_size: DiscSize::Fixed { semidiameter_arcmin: 16.0 }, ..HorizonModel::STANDARD };
    /// assert_eq!(Disc::under(Body::Sun, 0.983, &fixed).semidiameter_deg, 16.0 / 60.0);
    /// let mean = HorizonModel { disc_size: DiscSize::MeanDistance, ..HorizonModel::STANDARD };
    /// assert_eq!(Disc::under(Body::Sun, 0.983, &mean).semidiameter_deg, Disc::of(Body::Sun, 1.0).semidiameter_deg);
    /// // The older solar radius, by name.
    /// let older = HorizonModel { radii: Radii::Custom { sun_km: 696_000.0, moon_km: 1_737.4 }, ..HorizonModel::STANDARD };
    /// assert!(Disc::under(Body::Sun, 1.0, &older).semidiameter_deg > Disc::of(Body::Sun, 1.0).semidiameter_deg);
    /// ```
    #[must_use]
    pub fn under(body: Body, distance_au: f64, model: &HorizonModel) -> Disc {
        if !body.has_distance() || distance_au <= 0.0 {
            return Disc {
                semidiameter_deg: 0.0,
                parallax_deg: 0.0,
            };
        }
        let radius = match (model.radii, body) {
            (Radii::Custom { sun_km, .. }, Body::Sun) => sun_km,
            (Radii::Custom { moon_km, .. }, Body::Moon) => moon_km,
            _ => radius_km(body),
        };
        let distance_km = distance_au * AU_KM;
        let semidiameter = |at_km: f64| math::asin((radius / at_km).min(1.0)) * RAD2DEG;
        let semidiameter_deg = match (model.disc_size, body) {
            (
                DiscSize::Fixed {
                    semidiameter_arcmin,
                },
                _,
            ) if radius > 0.0 => semidiameter_arcmin / 60.0,
            (DiscSize::MeanDistance, Body::Sun) => semidiameter(AU_KM),
            (DiscSize::MeanDistance, Body::Moon) => semidiameter(MOON_MEAN_DISTANCE_AU * AU_KM),
            _ => semidiameter(distance_km),
        };
        Disc {
            semidiameter_deg,
            parallax_deg: math::asin((EARTH_EQUATORIAL_RADIUS_KM / distance_km).min(1.0)) * RAD2DEG,
        }
    }
}

/// A horizon reckoned once at a place: the apparent altitude of the
/// horizon itself (the model's raised or lowered horizon with its dip),
/// and the refraction at it, both in degrees.
///
/// ```
/// use teistro_astro::rise_set::{Reckoned, reckon};
/// use teistro_core::quantity::Altitude;
/// use teistro_port_ephemeris::Horizon;
///
/// // The almanac's sunrise: the ideal horizon and its 34′.
/// let sea = Altitude::try_new(0.0)?;
/// assert_eq!(reckon(&Horizon::UPPER_LIMB_REFRACTION, sea)?, Reckoned { height_deg: 0.0, refraction_deg: 34.0 / 60.0 });
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Reckoned {
    /// The horizon's apparent altitude, degrees: the model's altitude plus
    /// the dip (negative).
    pub height_deg: f64,
    /// The refraction at the horizon, degrees.
    pub refraction_deg: f64,
}

/// The dip of the horizon from a height under a convention, degrees
/// (negative), for a ray through `air` when the dip is refracted.
///
/// The geometric dip is the tangent to the sphere, `−acos(R / (R + h))`;
/// the refracted one is Sinclair's through the near-ground layer's
/// refraction coefficient (Thom 1973, in Reijs's metric form, as the Swiss
/// Ephemeris's `swe_refrac_extended` reckons it), `−acos(1 / (1 + h / R))
/// · √(1 − 1.8480 k P / T²)` with `k = (0.0342 + λ) / (0.154 · 0.0238)`.
///
/// ```
/// use teistro_astro::rise_set::dip_deg;
/// use teistro_core::quantity::Altitude;
/// use teistro_core::settings::{Atmosphere, Dip};
///
/// let kathmandu = Altitude::try_new(1400.0)?;
/// let geometric = dip_deg(Dip::Geometric, kathmandu, Atmosphere::STANDARD);
/// // About 1.2° from 1400 m, and the air lifts the horizon a little.
/// assert!((geometric + 1.2).abs() < 0.01);
/// let refracted = dip_deg(Dip::Refracted, kathmandu, Atmosphere::STANDARD);
/// assert!(refracted > geometric && refracted < 0.0);
/// assert_eq!(dip_deg(Dip::None, kathmandu, Atmosphere::STANDARD), 0.0);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[must_use]
pub fn dip_deg(dip: Dip, height: Altitude, air: Atmosphere) -> f64 {
    let h = height.get().max(0.0);
    match dip {
        Dip::Geometric => -math::acos(DIP_EARTH_RADIUS_M / (DIP_EARTH_RADIUS_M + h)) * RAD2DEG,
        Dip::Refracted => {
            let air = air.at(height);
            let kelvin = 273.15 + air.temperature_c;
            let coefficient = (0.0342 + LAPSE_RATE_K_PER_M) / (0.154 * 0.0238);
            let d = 1.0 - 1.8480 * coefficient * air.pressure_hpa / kelvin / kelvin;
            -math::acos(1.0 / (1.0 + h / REFRACTED_DIP_EARTH_RADIUS_M))
                * RAD2DEG
                * d.max(0.0).sqrt()
        }
        _ => 0.0,
    }
}

/// Saemundsson's refraction at a **true** altitude, arcminutes, at the air
/// it was fitted to (1010 hPa, 10 °C): `1.02 / tan(h + 10.3° / (h +
/// 5.11))` (Sæmundsson 1986; Meeus, eq. 16.4).
///
/// ```
/// use teistro_astro::rise_set::saemundsson_refraction_arcmin;
///
/// assert!((saemundsson_refraction_arcmin(-0.5) - 33.0).abs() < 1.0);
/// ```
#[must_use]
pub fn saemundsson_refraction_arcmin(true_deg: f64) -> f64 {
    1.02 / math::tan((true_deg + 10.3 / (true_deg + 5.11)) * DEG2RAD)
}

/// Sinclair's refraction at an **apparent** altitude through an air,
/// arcminutes (Sinclair 1982, as the Swiss Ephemeris reckons it): `(34.46 +
/// 4.23 a + 0.004 a²) / (1 + 0.505 a + 0.0845 a²)` up to 17.904°, where it
/// meets `0.97 / tan a`, scaled by `(P − 80) / 930 / (1 + 0.00008 (r + 39)
/// (T − 10))`.
///
/// ```
/// use teistro_astro::rise_set::sinclair_refraction_arcmin;
///
/// // At the horizon through the standard sea-level air: 33.6′.
/// assert!((sinclair_refraction_arcmin(0.0, 1013.25, 15.0) - 33.593).abs() < 1e-3);
/// ```
#[must_use]
pub fn sinclair_refraction_arcmin(apparent_deg: f64, pressure_hpa: f64, temperature_c: f64) -> f64 {
    let a = apparent_deg;
    let r = if a > 17.904_104_638_432 {
        0.97 / math::tan(a * DEG2RAD)
    } else {
        (34.46 + 4.23 * a + 0.004 * a * a) / (1.0 + 0.505 * a + 0.0845 * a * a)
    };
    (pressure_hpa - 80.0) / 930.0 / (1.0 + 0.000_08 * (r + 39.0) * (temperature_c - 10.0)) * r
}

/// The horizon a convention names at a height: its apparent altitude
/// (the model's altitude and the dip) and the refraction there, the
/// almanac's stated 34′ for the standard refraction and the model's
/// formula through a named air.
///
/// Bennett's formula is read at the horizon's own altitude with the dip,
/// as an engine reads it, and Saemundsson's at the true altitude whose
/// apparent one is the horizon's (a fixed point, which contracts by some
/// 0.06 a step there).
///
/// ```
/// use teistro_astro::rise_set::reckon;
/// use teistro_core::quantity::Altitude;
/// use teistro_core::settings::{Atmosphere, Dip, HorizonModel, RefractionFormula};
/// use teistro_port_ephemeris::{Horizon, Refraction};
///
/// let air = Horizon { refraction: Refraction::Atmosphere(Atmosphere::STANDARD), ..Horizon::UPPER_LIMB_REFRACTION };
/// let kathmandu = Altitude::try_new(1400.0)?;
/// let bennett = reckon(&air, kathmandu)?;
/// for formula in [RefractionFormula::Saemundsson, RefractionFormula::Sinclair] {
///     let other = reckon(&air.with_model(HorizonModel { refraction: formula, ..HorizonModel::STANDARD }), kathmandu)?;
///     // The three formulas agree at the horizon to under a minute of arc.
///     assert!((other.refraction_deg - bennett.refraction_deg).abs() * 60.0 < 1.0);
/// }
/// // A dipped horizon is lower and refracts more.
/// let dipped = reckon(&air.with_model(HorizonModel { dip: Dip::Geometric, ..HorizonModel::STANDARD }), kathmandu)?;
/// assert!(dipped.height_deg < -1.0 && dipped.refraction_deg > bennett.refraction_deg);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// `INVALID_ARG` for Bennett's formula read below −4°, past which its
/// pole at −4.4° makes it meaningless.
pub fn reckon(horizon: &Horizon, height: Altitude) -> Result<Reckoned, Error> {
    let model = horizon.model;
    let air = match horizon.refraction {
        Refraction::Atmosphere(air) => air,
        // The dip's air when the refraction names none.
        _ => Atmosphere::STANDARD,
    };
    let height_deg = model.altitude_deg + dip_deg(model.dip, height, air);
    let refraction_deg = match horizon.refraction {
        Refraction::Atmosphere(air) => {
            let air = air.at(height);
            match model.refraction {
                RefractionFormula::Saemundsson => {
                    let factor = air_factor(air.pressure_hpa, air.temperature_c);
                    let mut true_deg = height_deg - 0.5;
                    for _ in 0..12 {
                        true_deg =
                            height_deg - saemundsson_refraction_arcmin(true_deg) * factor / 60.0;
                    }
                    height_deg - true_deg
                }
                RefractionFormula::Sinclair => {
                    sinclair_refraction_arcmin(height_deg, air.pressure_hpa, air.temperature_c)
                        / 60.0
                }
                _ => {
                    if height_deg < BENNETT_LOWEST_DEG {
                        return Err(Error::invalid_arg(format!(
                            "Bennett's refraction is not defined at a horizon of {height_deg:.3}° (its pole is at -4.4°)"
                        ))
                        .with_field("day.horizon"));
                    }
                    horizon_refraction_at_arcmin(height_deg, air.pressure_hpa, air.temperature_c)
                        / 60.0
                }
            }
        }
        other => horizon_refraction_arcmin(other, height) / 60.0,
    };
    Ok(Reckoned {
        height_deg,
        refraction_deg,
    })
}

/// The point of the ecliptic a body stands over, as right ascension and
/// declination: its ecliptic latitude set to zero, the rest kept. The
/// Hindu rising (`RisingLatitude::Zero`) is the rising of that point.
///
/// ```
/// use teistro_astro::rise_set::on_the_ecliptic;
/// use teistro_astro::sky::{Apparent, Spherical, equatorial_to_ecliptic};
///
/// let moon = Apparent { ra_deg: 120.0, dec_deg: 25.0, distance_au: 0.0026 };
/// let point = on_the_ecliptic(&moon, 23.44);
/// let ecliptic = equatorial_to_ecliptic(Spherical { lon_deg: point.ra_deg, lat_deg: point.dec_deg }, 23.44);
/// assert!(ecliptic.lat_deg.abs() < 1e-9 && point.distance_au == moon.distance_au);
/// ```
#[must_use]
pub fn on_the_ecliptic(apparent: &Apparent, obliquity_deg: f64) -> Apparent {
    let ecliptic = equatorial_to_ecliptic(
        Spherical {
            lon_deg: apparent.ra_deg,
            lat_deg: apparent.dec_deg,
        },
        obliquity_deg,
    );
    let point = ecliptic_to_equatorial(
        Spherical {
            lon_deg: ecliptic.lon_deg,
            lat_deg: 0.0,
        },
        obliquity_deg,
    );
    Apparent {
        ra_deg: point.lon_deg,
        dec_deg: point.lat_deg,
        distance_au: apparent.distance_au,
    }
}

/// Meeus's scaling of a refraction from the air it was fitted to (1010
/// hPa, 10 °C) to another, `(P / 1010) · (283 / (273 + T))`.
fn air_factor(pressure_hpa: f64, temperature_c: f64) -> f64 {
    (pressure_hpa / 1010.0) * (283.0 / (273.0 + temperature_c))
}

/// Bennett's refraction at an apparent altitude through an air, arcminutes,
/// in the order [`horizon_refraction_arcmin`] has always multiplied it.
fn horizon_refraction_at_arcmin(apparent_deg: f64, pressure_hpa: f64, temperature_c: f64) -> f64 {
    /// The air Bennett's formula was fitted to.
    const REFERENCE_HPA: f64 = 1010.0;
    const REFERENCE_K: f64 = 283.0;
    const CELSIUS_TO_K: f64 = 273.0;
    bennett_refraction_arcmin(apparent_deg)
        * (pressure_hpa / REFERENCE_HPA)
        * (REFERENCE_K / (CELSIUS_TO_K + temperature_c))
}

/// The refraction at the horizon, arcminutes, as the SDK's solver
/// reckons it: none, the almanac's 34′, or Bennett's through a named air
/// resolved at the observer's height (`03-design/horizon-atmosphere.md`).
///
/// Bennett's formula at an apparent altitude of zero is `cot(7.31° / 4.4)`
/// (Meeus, eq. 16.4), about 34.48′ at the air it was fitted to, 1010 hPa
/// and 10 °C; another air scales it by `(P / 1010) · (283 / (273 + T))`
/// (Meeus, ch. 16), Meeus's 273 kept rather than 273.15.
///
/// ```
/// use teistro_astro::rise_set::horizon_refraction_arcmin;
/// use teistro_core::quantity::Altitude;
/// use teistro_core::settings::Atmosphere;
/// use teistro_port_ephemeris::Refraction;
///
/// let sea = Altitude::try_new(0.0).unwrap();
/// let reference = Refraction::Atmosphere(Atmosphere::given(1010.0, 10.0));
/// assert!((horizon_refraction_arcmin(reference, sea) - 34.4775).abs() < 1e-4);
/// // The engines' standard air at 1400 m refracts some six arcminutes less.
/// let kathmandu = Altitude::try_new(1400.0).unwrap();
/// let standard = Refraction::Atmosphere(Atmosphere::STANDARD);
/// assert!((horizon_refraction_arcmin(standard, kathmandu) - 28.72).abs() < 0.01);
/// assert_eq!(horizon_refraction_arcmin(Refraction::Standard, kathmandu), 34.0);
/// ```
#[must_use]
pub fn horizon_refraction_arcmin(refraction: Refraction, height: Altitude) -> f64 {
    match refraction {
        Refraction::None => 0.0,
        Refraction::Standard => STANDARD_HORIZON_REFRACTION_ARCMIN,
        Refraction::Atmosphere(air) => {
            let air = air.at(height);
            horizon_refraction_at_arcmin(0.0, air.pressure_hpa, air.temperature_c)
        }
    }
}

/// Bennett's refraction at an apparent altitude, arcminutes, at the air
/// it was fitted to (1010 hPa, 10 °C): `cot(h + 7.31° / (h + 4.4))`
/// (Meeus, eq. 16.4), fitted to apparent altitudes from 0° to 90° and
/// good to 0.07′ there. [`horizon_refraction_arcmin`] reads it at zero;
/// a skyline above the horizon reads it at the skyline's elevation
/// ([`crate::terrain::refraction_scale`]).
///
/// ```
/// use teistro_astro::rise_set::bennett_refraction_arcmin;
///
/// assert!((bennett_refraction_arcmin(0.0) - 34.4775).abs() < 1e-4);
/// // The higher the ray comes in, the less the air bends it.
/// assert!(bennett_refraction_arcmin(2.0) < 19.0 && bennett_refraction_arcmin(45.0) < 1.0);
/// ```
#[must_use]
pub fn bennett_refraction_arcmin(apparent_deg: f64) -> f64 {
    1.0 / math::tan((apparent_deg + 7.31 / (apparent_deg + 4.4)) * DEG2RAD)
}

/// The geocentric altitude of the body's centre at the event under a
/// convention, for an observer at a height, degrees: the disc point's
/// target altitude on the horizon the model reckons ([`reckon`]), less the
/// refraction, less the semidiameter for the upper limb (plus it for the
/// lower), plus the horizontal parallax.
///
/// ```
/// use teistro_astro::rise_set::{centre_altitude_deg, Disc};
/// use teistro_core::quantity::Altitude;
/// use teistro_port_ephemeris::{Body, Horizon};
///
/// let sun = Disc::of(Body::Sun, 1.0);
/// let sea = Altitude::try_new(0.0).unwrap();
/// let almanac = centre_altitude_deg(&Horizon::UPPER_LIMB_REFRACTION, &sun, sea)?;
/// assert!((almanac + 50.0 / 60.0).abs() < 0.01); // about -0°50′, Meeus's value
/// assert_eq!(centre_altitude_deg(&Horizon::CENTRE_NO_REFRACTION, &sun, sea)?, sun.parallax_deg);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// As [`reckon`].
pub fn centre_altitude_deg(horizon: &Horizon, disc: &Disc, height: Altitude) -> Result<f64, Error> {
    Ok(event_altitude_deg(horizon, disc, reckon(horizon, height)?))
}

/// [`centre_altitude_deg`] with the horizon already reckoned, degrees,
/// which the solver does once rather than at every sample.
fn event_altitude_deg(horizon: &Horizon, disc: &Disc, reckoned: Reckoned) -> f64 {
    let limb = match horizon.disc {
        DiscPoint::Centre => 0.0,
        DiscPoint::UpperLimb => -disc.semidiameter_deg,
        DiscPoint::LowerLimb => disc.semidiameter_deg,
    };
    horizon.altitude_deg + reckoned.height_deg - reckoned.refraction_deg + limb + disc.parallax_deg
}

/// Which method answered.
///
/// ```
/// # use teistro_astro::rise_set::Solver;
/// # use teistro_astro::{Completion, DeltaTModel};
/// # use teistro_core::quantity::{JulianDay, Place, Ut1};
/// # use teistro_core::settings::OverridePolicy;
/// # use teistro_port_ephemeris::{Body, Horizon, TestProvider};
/// #
/// # let provider = TestProvider::new();
/// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let place = Place::try_from_degrees(27.7172, 85.324, 0.0)?;
/// # let solver = Solver::new(&sky, Body::Sun, place, Horizon::UPPER_LIMB_REFRACTION, DeltaTModel::TableThenModel);
/// # // The local mean day of 2024-06-21 there.
/// # let midnight = JulianDay::<Ut1>::literal(2_460_482.5 - 85.324 / 360.0);
///
/// use teistro_astro::rise_set::Method;
/// use teistro_port_ephemeris::HorizonEventKind;
///
/// // A temperate sunrise is the iteration's.
/// let rise = solver.event(HorizonEventKind::Rise, midnight, 1.0)?.expect("a sunrise");
/// assert_eq!(rise.method, Method::Iterated);
/// assert_eq!(serde_json::to_string(&Method::Scanned)?, r#""SCANNED""#);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Method {
    /// The hour-angle estimate iterated to convergence.
    Iterated,
    /// The window scanned for the altitude's sign change and bisected.
    Scanned,
    /// The ephemeris engine's own search (the rise and set override),
    /// which the policy chose (boundary audit, rank 6).
    Native,
}

/// A found horizon event.
///
/// ```
/// # use teistro_astro::rise_set::Solver;
/// # use teistro_astro::{Completion, DeltaTModel};
/// # use teistro_core::quantity::{JulianDay, Place, Ut1};
/// # use teistro_core::settings::OverridePolicy;
/// # use teistro_port_ephemeris::{Body, Horizon, TestProvider};
/// #
/// # let provider = TestProvider::new();
/// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let place = Place::try_from_degrees(27.7172, 85.324, 0.0)?;
/// # let solver = Solver::new(&sky, Body::Sun, place, Horizon::UPPER_LIMB_REFRACTION, DeltaTModel::TableThenModel);
/// # // The local mean day of 2024-06-21 there.
/// # let midnight = JulianDay::<Ut1>::literal(2_460_482.5 - 85.324 / 360.0);
///
/// use teistro_port_ephemeris::HorizonEventKind;
///
/// let set = solver.event(HorizonEventKind::Set, midnight, 1.0)?.expect("a sunset");
/// // Kathmandu's midsummer sunset falls in the local evening, and took a few readings.
/// let hours = (set.instant.get() - midnight.get()) * 24.0;
/// assert!(hours > 18.0 && hours < 20.0);
/// assert!(set.evaluations > 0);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
///
/// ```
/// # use teistro_astro::rise_set::Solver;
/// # use teistro_astro::{Completion, DeltaTModel};
/// # use teistro_core::quantity::{JulianDay, Place, Ut1};
/// # use teistro_core::settings::OverridePolicy;
/// # use teistro_port_ephemeris::{Body, Horizon, TestProvider};
/// #
/// # let provider = TestProvider::new();
/// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let place = Place::try_from_degrees(69.65, 18.96, 0.0)?;
/// # let solver = Solver::new(&sky, Body::Sun, place, Horizon::UPPER_LIMB_REFRACTION, DeltaTModel::TableThenModel);
/// # // The local mean day of 2024-06-21 there.
/// # let midnight = JulianDay::<Ut1>::literal(2_460_482.5 - 18.96 / 360.0);
///
/// // Tromsø at midsummer: a polar day, the Sun above all day and neither rising nor setting.
/// let day = solver.day(midnight)?;
/// assert_eq!((day.rise, day.set, day.above_at_midday), (None, None, true));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
    ///
    /// ```
    /// # use teistro_astro::rise_set::Solver;
    /// # use teistro_astro::{Completion, DeltaTModel};
    /// # use teistro_core::quantity::{JulianDay, Place, Ut1};
    /// # use teistro_core::settings::OverridePolicy;
    /// # use teistro_port_ephemeris::{Body, Horizon, TestProvider};
    /// #
    /// # let provider = TestProvider::new();
    /// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
    /// # let place = Place::try_from_degrees(27.7172, 85.324, 0.0)?;
    /// # let solver = Solver::new(&sky, Body::Sun, place, Horizon::UPPER_LIMB_REFRACTION, DeltaTModel::TableThenModel);
    /// # // The local mean day of 2024-06-21 there.
    /// # let midnight = JulianDay::<Ut1>::literal(2_460_482.5 - 85.324 / 360.0);
    ///
    /// let (rise, set) = solver.day(midnight)?.arc().expect("a sunrise and a sunset");
    /// assert!(rise.get() < set.get());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
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

/// One reading of the sky at an instant, in the solver's own terms.
#[allow(
    clippy::struct_field_names,
    reason = "each field is a quantity in degrees and says so"
)]
#[derive(Clone, Copy)]
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
    /// The horizon at the place, reckoned once ([`reckon`]), or why it
    /// cannot be, which every reading then answers.
    reckoned: Result<Reckoned, Error>,
    /// The terrain the event is reckoned against in place of the
    /// convention's altitude ([`Solver::with_terrain`]).
    terrain: Option<&'a HorizonProfile>,
    delta_t: DeltaTModel,
    chunk: usize,
    absence_check: bool,
    /// Whether to try proving no event of the kind asked for in a window
    /// before walking it ([`Solver::proves_none`]).
    none_proof: bool,
    /// The last instant read alone and its reading, by the instant's
    /// bits. A day's moonrise and moonset searches open at the same
    /// instant and are polled one after the other, so the second reads
    /// the first's reading rather than reckoning the sidereal time again;
    /// a reading is a function of the instant alone, so it is the same
    /// to the bit.
    last: Cell<Option<(u64, Sample)>>,
}

impl fmt::Debug for Solver<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.describe())
    }
}

impl<'a> Solver<'a> {
    /// A solver.
    ///
    /// ```
    /// # use teistro_astro::rise_set::Solver;
    /// # use teistro_astro::{Completion, DeltaTModel};
    /// # use teistro_core::quantity::{JulianDay, Place, Ut1};
    /// # use teistro_core::settings::OverridePolicy;
    /// # use teistro_port_ephemeris::{Body, Horizon, TestProvider};
    /// #
    /// # let provider = TestProvider::new();
    /// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
    /// # let place = Place::try_from_degrees(27.7172, 85.324, 0.0)?;
    /// # let solver = Solver::new(&sky, Body::Sun, place, Horizon::UPPER_LIMB_REFRACTION, DeltaTModel::TableThenModel);
    /// # // The local mean day of 2024-06-21 there.
    /// # let midnight = JulianDay::<Ut1>::literal(2_460_482.5 - 85.324 / 360.0);
    ///
    /// // The convention it was built with, and a stamp naming it all.
    /// assert_eq!(solver.horizon(), Horizon::UPPER_LIMB_REFRACTION);
    /// assert!(solver.describe().starts_with("SUN "));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn new(
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
            reckoned: reckon(&horizon, place.altitude),
            terrain: None,
            delta_t,
            chunk: SCAN_CHUNK,
            absence_check: true,
            none_proof: true,
            last: Cell::new(None),
        }
    }

    /// Whether to try proving a rise or a set **absent** before looking
    /// for it.
    ///
    /// On by default. Two readings of the sky bound where the altitude
    /// can reach over the window, and where that bound is on one side of
    /// the event's target there is no crossing and nothing to look for
    /// — which saves the iteration and the walk behind it, and an
    /// almanac spends most of its horizon work proving absences
    /// (`03-design/horizon-absence-measured.md`).
    ///
    /// Turning it off is what the measured page's own sweep does, so
    /// that the rule is compared against a search that does not use it;
    /// a caller who wants the walk's answer by the walk's route can do
    /// the same.
    ///
    /// ```
    /// # use teistro_astro::rise_set::Solver;
    /// # use teistro_astro::{Completion, DeltaTModel};
    /// # use teistro_core::quantity::{JulianDay, Place, Ut1};
    /// # use teistro_core::settings::OverridePolicy;
    /// # use teistro_port_ephemeris::{Body, Horizon, TestProvider};
    /// #
    /// # let provider = TestProvider::new();
    /// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
    /// # let place = Place::try_from_degrees(69.65, 18.96, 0.0)?;
    /// # let solver = Solver::new(&sky, Body::Sun, place, Horizon::UPPER_LIMB_REFRACTION, DeltaTModel::TableThenModel);
    /// # // The local mean day of 2024-06-21 there.
    /// # let midnight = JulianDay::<Ut1>::literal(2_460_482.5 - 18.96 / 360.0);
    ///
    /// use teistro_port_ephemeris::HorizonEventKind;
    ///
    /// // No sunset at midsummer, proved or walked: the same answer.
    /// let proved = solver.event(HorizonEventKind::Set, midnight, 1.0)?;
    /// let walked = Solver::new(&sky, Body::Sun, place, Horizon::UPPER_LIMB_REFRACTION, DeltaTModel::TableThenModel)
    ///     .with_absence_check(false)
    ///     .event(HorizonEventKind::Set, midnight, 1.0)?;
    /// assert_eq!((proved, walked), (None, None));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub const fn with_absence_check(mut self, on: bool) -> Solver<'a> {
        self.absence_check = on;
        self
    }

    /// Whether to try proving that a window holds no event of the kind
    /// asked for before walking it ([`Solver::proves_no_event`]).
    ///
    /// On by default, and an almanac's event loop is what it is for: the
    /// search that ends the loop has no event to find in what is left of
    /// the day, and proving that from a dozen readings replaces a walk of
    /// hundreds (`03-design/horizon-absence-measured.md` §7). The answer
    /// is the walk's either way. Turning it off is what that page's sweep
    /// does to measure the proof against the walk.
    ///
    /// ```
    /// # use teistro_astro::rise_set::Solver;
    /// # use teistro_astro::{Completion, DeltaTModel};
    /// # use teistro_core::quantity::{JulianDay, Place, Ut1};
    /// # use teistro_core::settings::OverridePolicy;
    /// # use teistro_port_ephemeris::{Body, Horizon, TestProvider};
    /// #
    /// # let provider = TestProvider::new();
    /// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
    /// # let place = Place::try_from_degrees(27.7172, 85.324, 0.0)?;
    /// # let solver = Solver::new(&sky, Body::Sun, place, Horizon::UPPER_LIMB_REFRACTION, DeltaTModel::TableThenModel);
    /// # // The local mean day of 2024-06-21 there.
    /// # let midnight = JulianDay::<Ut1>::literal(2_460_482.5 - 85.324 / 360.0);
    ///
    /// use teistro_port_ephemeris::HorizonEventKind;
    ///
    /// let walk = || Solver::new(&sky, Body::Sun, place, Horizon::UPPER_LIMB_REFRACTION, DeltaTModel::TableThenModel);
    /// // The rise is the same whichever route found it.
    /// let with = walk().event(HorizonEventKind::Rise, midnight, 1.0)?.map(|rise| rise.instant);
    /// let without = walk().with_none_proof(false).event(HorizonEventKind::Rise, midnight, 1.0)?.map(|rise| rise.instant);
    /// assert_eq!(with, without);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub const fn with_none_proof(mut self, on: bool) -> Solver<'a> {
        self.none_proof = on;
        self
    }

    /// The rise and set **behind a terrain** (`03-design/terrain-horizon.md`):
    /// the profile's elevation at the body's azimuth takes the place of the
    /// convention's altitude, and the convention's refraction is carried
    /// to that apparent altitude by Bennett's formula
    /// ([`crate::terrain::refraction_scale`]). The disc point and the air
    /// stay the convention's.
    ///
    /// The azimuth moves while the body climbs, so the event's altitude
    /// is a function of the instant; the search is the scan, which finds
    /// the **first** crossing a ten-minute grid brackets, rather than the
    /// iteration, which settles on whichever crossing is nearest its
    /// estimate. The absence proofs bound a fixed target and are not
    /// tried; the walk answers instead.
    ///
    /// ```
    /// # use teistro_astro::rise_set::Solver;
    /// # use teistro_astro::{Completion, DeltaTModel};
    /// # use teistro_core::quantity::{JulianDay, Place, Ut1};
    /// # use teistro_core::settings::OverridePolicy;
    /// # use teistro_port_ephemeris::{Body, Horizon, TestProvider};
    /// #
    /// # let provider = TestProvider::new();
    /// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
    /// # let place = Place::try_from_degrees(27.7172, 85.324, 0.0)?;
    /// # let solver = Solver::new(&sky, Body::Sun, place, Horizon::UPPER_LIMB_REFRACTION, DeltaTModel::TableThenModel);
    /// # // The local mean day of 2024-06-21 there.
    /// # let midnight = JulianDay::<Ut1>::literal(2_460_482.5 - 85.324 / 360.0);
    ///
    /// use teistro_astro::rise_set::Method;
    /// use teistro_astro::terrain::{HorizonProfile, HorizonSample};
    /// use teistro_port_ephemeris::HorizonEventKind;
    ///
    /// let hills = HorizonProfile::try_new(vec![HorizonSample { azimuth_deg: 0.0, elevation_deg: 3.0 }])?;
    /// let behind = Solver::new(&sky, Body::Sun, place, Horizon::UPPER_LIMB_REFRACTION, DeltaTModel::TableThenModel)
    ///     .with_terrain(&hills);
    /// let level = solver.event(HorizonEventKind::Rise, midnight, 1.0)?.expect("a sunrise");
    /// let seen = behind.event(HorizonEventKind::Rise, midnight, 1.0)?.expect("over the hills");
    /// assert!(seen.instant.get() > level.instant.get());
    /// assert_eq!(seen.method, Method::Scanned);
    /// assert!(behind.describe().contains("behind a terrain of 1 sample"));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub const fn with_terrain(mut self, profile: &'a HorizonProfile) -> Solver<'a> {
        self.terrain = Some(profile);
        self
    }

    /// How many of the scan's instants to ask the source for at once.
    ///
    /// The default is [`crate::solve::SCAN_CHUNK`]. One is the walk this
    /// replaced — every instant its own round trip — which is what the
    /// grid is held against in the tests. Zero is read as one.
    ///
    /// ```
    /// # use teistro_astro::rise_set::Solver;
    /// # use teistro_astro::{Completion, DeltaTModel};
    /// # use teistro_core::quantity::{JulianDay, Place, Ut1};
    /// # use teistro_core::settings::OverridePolicy;
    /// # use teistro_port_ephemeris::{Body, Horizon, TestProvider};
    /// #
    /// # let provider = TestProvider::new();
    /// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
    /// # let place = Place::try_from_degrees(69.65, 18.96, 0.0)?;
    /// # let solver = Solver::new(&sky, Body::Sun, place, Horizon::UPPER_LIMB_REFRACTION, DeltaTModel::TableThenModel);
    /// # // The local mean day of 2024-06-21 there.
    /// # let midnight = JulianDay::<Ut1>::literal(2_460_482.5 - 18.96 / 360.0);
    ///
    /// use teistro_port_ephemeris::HorizonEventKind;
    ///
    /// // Zero is read as one; the answer is the grid's either way.
    /// let walked = Solver::new(&sky, Body::Sun, place, Horizon::UPPER_LIMB_REFRACTION, DeltaTModel::TableThenModel).with_chunk(0);
    /// assert_eq!(walked.day(midnight)?, solver.day(midnight)?);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub const fn with_chunk(mut self, instants: usize) -> Solver<'a> {
        self.chunk = if instants == 0 { 1 } else { instants };
        self
    }

    /// The convention.
    ///
    /// ```
    /// # use teistro_astro::rise_set::Solver;
    /// # use teistro_astro::{Completion, DeltaTModel};
    /// # use teistro_core::quantity::{JulianDay, Place, Ut1};
    /// # use teistro_core::settings::OverridePolicy;
    /// # use teistro_port_ephemeris::{Body, Horizon, TestProvider};
    /// #
    /// # let provider = TestProvider::new();
    /// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
    /// # let place = Place::try_from_degrees(27.7172, 85.324, 0.0)?;
    /// # let solver = Solver::new(&sky, Body::Sun, place, Horizon::UPPER_LIMB_REFRACTION, DeltaTModel::TableThenModel);
    /// # // The local mean day of 2024-06-21 there.
    /// # let midnight = JulianDay::<Ut1>::literal(2_460_482.5 - 85.324 / 360.0);
    ///
    /// assert_eq!(solver.horizon().refraction, Horizon::UPPER_LIMB_REFRACTION.refraction);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub const fn horizon(&self) -> Horizon {
        self.horizon
    }

    /// The stamp: body, convention, place and source.
    ///
    /// ```
    /// # use teistro_astro::rise_set::Solver;
    /// # use teistro_astro::{Completion, DeltaTModel};
    /// # use teistro_core::quantity::{JulianDay, Place, Ut1};
    /// # use teistro_core::settings::OverridePolicy;
    /// # use teistro_port_ephemeris::{Body, Horizon, TestProvider};
    /// #
    /// # let provider = TestProvider::new();
    /// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
    /// # let place = Place::try_from_degrees(27.7172, 85.324, 0.0)?;
    /// # let solver = Solver::new(&sky, Body::Sun, place, Horizon::UPPER_LIMB_REFRACTION, DeltaTModel::TableThenModel);
    /// # // The local mean day of 2024-06-21 there.
    /// # let midnight = JulianDay::<Ut1>::literal(2_460_482.5 - 85.324 / 360.0);
    ///
    /// // Body, convention, place and the source it reads.
    /// assert!(solver.describe().contains("test-provider"));
    /// assert_eq!(format!("{solver:?}"), solver.describe());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn describe(&self) -> String {
        let terrain = self.terrain.map_or_else(String::new, |profile| {
            format!(" behind a terrain of {profile}")
        });
        format!(
            "{} {}{terrain} at {} from {}",
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
    /// ```
    /// # use teistro_astro::rise_set::Solver;
    /// # use teistro_astro::{Completion, DeltaTModel};
    /// # use teistro_core::quantity::{JulianDay, Place, Ut1};
    /// # use teistro_core::settings::OverridePolicy;
    /// # use teistro_port_ephemeris::{Body, Horizon, TestProvider};
    /// #
    /// # let provider = TestProvider::new();
    /// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
    /// # let place = Place::try_from_degrees(27.7172, 85.324, 0.0)?;
    /// # let solver = Solver::new(&sky, Body::Sun, place, Horizon::UPPER_LIMB_REFRACTION, DeltaTModel::TableThenModel);
    /// # // The local mean day of 2024-06-21 there.
    /// # let midnight = JulianDay::<Ut1>::literal(2_460_482.5 - 85.324 / 360.0);
    ///
    /// // Below the horizon at local midnight, high at local noon.
    /// assert!(solver.altitude_deg(midnight)? < 0.0);
    /// let noon = JulianDay::<Ut1>::literal(midnight.get() + 0.5);
    /// assert!(solver.altitude_deg(noon)? > 80.0);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// The source's refusal, or a Delta T model that cannot answer.
    pub fn altitude_deg(&self, ut1: JulianDay<Ut1>) -> Result<f64, Error> {
        let mut evaluations = 0;
        Ok(self.sample(ut1.get(), &mut evaluations)?.altitude_deg)
    }

    /// The geocentric azimuth of the body's centre at an instant, degrees
    /// in `[0, 360)` from north through east, as the solver reads it: the
    /// azimuth a terrain's elevation is read at ([`Solver::with_terrain`]).
    ///
    /// ```
    /// # use teistro_astro::rise_set::Solver;
    /// # use teistro_astro::{Completion, DeltaTModel};
    /// # use teistro_core::quantity::{JulianDay, Place, Ut1};
    /// # use teistro_core::settings::OverridePolicy;
    /// # use teistro_port_ephemeris::{Body, Horizon, TestProvider};
    /// #
    /// # let provider = TestProvider::new();
    /// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
    /// # let place = Place::try_from_degrees(27.7172, 85.324, 0.0)?;
    /// # let solver = Solver::new(&sky, Body::Sun, place, Horizon::UPPER_LIMB_REFRACTION, DeltaTModel::TableThenModel);
    /// # // The local mean day of 2024-06-21 there.
    /// # let midnight = JulianDay::<Ut1>::literal(2_460_482.5 - 85.324 / 360.0);
    ///
    /// use teistro_port_ephemeris::HorizonEventKind;
    ///
    /// // The midsummer Sun rises north of east.
    /// let rise = solver.event(HorizonEventKind::Rise, midnight, 1.0)?.expect("a sunrise");
    /// let azimuth = solver.azimuth_deg(rise.instant)?;
    /// assert!(azimuth > 55.0 && azimuth < 70.0, "{azimuth}");
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`Solver::altitude_deg`].
    pub fn azimuth_deg(&self, ut1: JulianDay<Ut1>) -> Result<f64, Error> {
        let mut evaluations = 0;
        let sample = self.sample(ut1.get(), &mut evaluations)?;
        Ok(horizontal(
            sample.hour_angle_deg,
            sample.dec_deg,
            self.place.latitude.get(),
        )
        .1)
    }

    /// Whether the body stands above the altitude its rise and set are
    /// at: which side of the convention's horizon it is on, which is what
    /// names a window without a crossing as always above or always below.
    ///
    /// ```
    /// # use teistro_astro::rise_set::Solver;
    /// # use teistro_astro::{Completion, DeltaTModel};
    /// # use teistro_core::quantity::{JulianDay, Place, Ut1};
    /// # use teistro_core::settings::OverridePolicy;
    /// # use teistro_port_ephemeris::{Body, Horizon, TestProvider};
    /// #
    /// # let provider = TestProvider::new();
    /// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
    /// # let place = Place::try_from_degrees(27.7172, 85.324, 0.0)?;
    /// # let solver = Solver::new(&sky, Body::Sun, place, Horizon::UPPER_LIMB_REFRACTION, DeltaTModel::TableThenModel);
    /// # // The local mean day of 2024-06-21 there.
    /// # let midnight = JulianDay::<Ut1>::literal(2_460_482.5 - 85.324 / 360.0);
    ///
    /// assert!(!solver.is_above(midnight)?);
    /// assert!(solver.is_above(JulianDay::<Ut1>::literal(midnight.get() + 0.5))?);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`Solver::altitude_deg`].
    pub fn is_above(&self, ut1: JulianDay<Ut1>) -> Result<bool, Error> {
        let mut evaluations = 0;
        let sample = self.sample(ut1.get(), &mut evaluations)?;
        Ok(sample.altitude_deg > sample.target_deg)
    }

    /// The sky at an instant: altitude, target, hour angle, declination.
    /// The sky at many instants at once, in the solver's own terms.
    ///
    /// One request where the walk made one per instant. The readings are
    /// the walk's own — the same source, the same instants — so a scan
    /// that reads them here answers exactly what a scan that read them
    /// one at a time answered.
    fn samples(&self, instants: &[f64], evaluations: &mut u32) -> Result<Vec<Sample>, Error> {
        let ut1: Vec<JulianDay<Ut1>> = instants
            .iter()
            .map(|t| JulianDay::<Ut1>::literal(*t))
            .collect();
        let mut apparent = Vec::with_capacity(ut1.len());
        self.sky.apparent_many(self.body, &ut1, &mut apparent)?;
        *evaluations += u32::try_from(ut1.len()).unwrap_or(u32::MAX);
        ut1.iter()
            .zip(&apparent)
            .map(|(at, apparent)| self.reading(*at, apparent))
            .collect()
    }

    fn sample(&self, t: f64, evaluations: &mut u32) -> Result<Sample, Error> {
        let ut1 = JulianDay::<Ut1>::try_new(t)?;
        *evaluations += 1;
        if let Some(kept) = self.kept(t) {
            return Ok(kept);
        }
        let apparent = self.sky.apparent(self.body, ut1)?;
        self.keep(t, self.reading(ut1, &apparent)?)
    }

    /// The reading [`Solver::keep`] last kept, when it was of `t`.
    fn kept(&self, t: f64) -> Option<Sample> {
        self.last
            .get()
            .and_then(|(bits, sample)| (bits == t.to_bits()).then_some(sample))
    }

    /// Keeps a reading of `t` as the last read alone, and answers it.
    #[expect(
        clippy::unnecessary_wraps,
        reason = "it closes the paths that read, each of which may fail"
    )]
    fn keep(&self, t: f64, sample: Sample) -> Result<Sample, Error> {
        self.last.set(Some((t.to_bits(), sample)));
        Ok(sample)
    }

    /// [`Solver::sample`] on the spot, or in `round`'s request.
    async fn sample_in(
        &self,
        round: Option<&Lockstep<'a>>,
        t: f64,
        evaluations: &mut u32,
    ) -> Result<Sample, Error> {
        let Some(round) = round else {
            return self.sample(t, evaluations);
        };
        let ut1 = JulianDay::<Ut1>::try_new(t)?;
        *evaluations += 1;
        if let Some(kept) = self.kept(t) {
            return Ok(kept);
        }
        let apparent = round.apparent_at(self.sky, self.body, ut1).await?;
        // Kept by another search between the asking and the answer: the
        // round's searches that opened at this instant answer after one
        // another, and only the first reckons it.
        if let Some(kept) = self.kept(t) {
            return Ok(kept);
        }
        self.keep(t, self.reading(ut1, &apparent)?)
    }

    /// [`Solver::samples`] on the spot, or in `round`'s request.
    async fn samples_in(
        &self,
        round: Option<&Lockstep<'a>>,
        instants: &[f64],
        evaluations: &mut u32,
    ) -> Result<Vec<Sample>, Error> {
        let Some(round) = round else {
            return self.samples(instants, evaluations);
        };
        let ut1: Vec<JulianDay<Ut1>> = instants
            .iter()
            .map(|t| JulianDay::<Ut1>::literal(*t))
            .collect();
        let apparent = round.apparent(self.sky, self.body, &ut1).await?;
        *evaluations += u32::try_from(ut1.len()).unwrap_or(u32::MAX);
        ut1.iter()
            .zip(&apparent)
            .map(|(at, apparent)| self.reading(*at, apparent))
            .collect()
    }

    /// One apparent position read as the solver reads it: the altitude
    /// above the horizon, the altitude the event is at, the hour angle
    /// and the declination. Shared by the walk and the grid so that
    /// neither can drift from the other.
    fn reading(&self, ut1: JulianDay<Ut1>, apparent: &Apparent) -> Result<Sample, Error> {
        let reckoned = self.reckoned.clone()?;
        let (tt, _) = tt_of(ut1, self.delta_t)?;
        let sidereal = sidereal_time_deg(ut1, tt, self.place.longitude);
        // The rising of the ecliptic point the body stands over, when the
        // convention takes the body's latitude as zero (the Hindu rising).
        let apparent = match self.horizon.model.latitude {
            RisingLatitude::Zero => on_the_ecliptic(apparent, obliquity(tt).true_deg),
            _ => *apparent,
        };
        let hour_angle_deg = difference_deg(sidereal, apparent.ra_deg);
        let (sin_phi, cos_phi) = math::sin_cos(self.place.latitude.get() * DEG2RAD);
        let (sin_dec, cos_dec) = math::sin_cos(apparent.dec_deg * DEG2RAD);
        let sin_alt = sin_phi * sin_dec + cos_phi * cos_dec * math::cos(hour_angle_deg * DEG2RAD);
        let disc = Disc::under(self.body, apparent.distance_au, &self.horizon.model);
        let target_deg = match self.terrain {
            None => event_altitude_deg(&self.horizon, &disc, reckoned),
            Some(profile) => {
                let (_, azimuth) =
                    horizontal(hour_angle_deg, apparent.dec_deg, self.place.latitude.get());
                let skyline = profile.elevation_at(azimuth);
                // The skyline is the horizon: neither the model's altitude
                // nor a dip enters it.
                event_altitude_deg(
                    &Horizon {
                        altitude_deg: skyline,
                        ..self.horizon
                    },
                    &disc,
                    Reckoned {
                        height_deg: 0.0,
                        refraction_deg: reckoned.refraction_deg * refraction_scale(skyline),
                    },
                )
            }
        };
        Ok(Sample {
            altitude_deg: math::asin(sin_alt.clamp(-1.0, 1.0)) * RAD2DEG,
            target_deg,
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
                let (sin_phi, cos_phi) = math::sin_cos(self.place.latitude.get() * DEG2RAD);
                let (sin_dec, cos_dec) = math::sin_cos(sample.dec_deg * DEG2RAD);
                let denominator = cos_phi * cos_dec;
                if denominator.abs() < f64::EPSILON {
                    return None;
                }
                let cos_h =
                    (math::sin(sample.target_deg * DEG2RAD) - sin_phi * sin_dec) / denominator;
                if !(-1.0..=1.0).contains(&cos_h) {
                    return None;
                }
                let h = math::acos(cos_h) * RAD2DEG;
                Some(if kind == HorizonEventKind::Rise {
                    -h
                } else {
                    h
                })
            }
        }
    }

    /// Whether the readings alone prove that the window starting `from`
    /// and lasting `window_days` holds no event of `kind`, without
    /// walking it: the proof [`Solver::event`] tries before a walk
    /// (see [`Solver::with_none_proof`]). `false` says only that the
    /// proof could not settle the window, never that an event is in it.
    ///
    /// ```
    /// # use teistro_astro::rise_set::Solver;
    /// # use teistro_astro::{Completion, DeltaTModel};
    /// # use teistro_core::quantity::{JulianDay, Place, Ut1};
    /// # use teistro_core::settings::OverridePolicy;
    /// # use teistro_port_ephemeris::{Body, Horizon, TestProvider};
    /// #
    /// # let provider = TestProvider::new();
    /// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
    /// # let place = Place::try_from_degrees(69.65, 18.96, 0.0)?;
    /// # let solver = Solver::new(&sky, Body::Sun, place, Horizon::UPPER_LIMB_REFRACTION, DeltaTModel::TableThenModel);
    /// # // The local mean day of 2024-06-21 there.
    /// # let midnight = JulianDay::<Ut1>::literal(2_460_482.5 - 18.96 / 360.0);
    ///
    /// use teistro_port_ephemeris::HorizonEventKind;
    ///
    /// // Two readings bound the Sun's altitude at Tromsø above the horizon all day.
    /// assert!(solver.proves_no_event(HorizonEventKind::Set, midnight, 1.0)?);
    /// assert!(solver.proves_no_event(HorizonEventKind::Set, midnight, 0.0).is_err());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A window that is not a positive number of days, the source's
    /// refusal, or a Delta T model that cannot answer.
    pub fn proves_no_event(
        &self,
        kind: HorizonEventKind,
        from: JulianDay<Ut1>,
        window_days: f64,
    ) -> Result<bool, Error> {
        if !(window_days.is_finite() && window_days > 0.0) {
            return Err(Error::invalid_arg(format!(
                "a search window is a positive number of days, not {window_days}"
            ))
            .with_field("window_days"));
        }
        if !matches!(kind, HorizonEventKind::Rise | HorizonEventKind::Set) {
            return Ok(false);
        }
        let start = from.get();
        let first = self.sample(start, &mut 0)?;
        spot(self.proves_none(None, kind, (start, first), start + window_days, &mut 0))
    }

    /// The next event of a kind at or after `from`, inside a window of so
    /// many days, or `None` when there is none inside it (a polar day or
    /// night, a circumpolar body).
    ///
    /// ```
    /// # use teistro_astro::rise_set::Solver;
    /// # use teistro_astro::{Completion, DeltaTModel};
    /// # use teistro_core::quantity::{JulianDay, Place, Ut1};
    /// # use teistro_core::settings::OverridePolicy;
    /// # use teistro_port_ephemeris::{Body, Horizon, TestProvider};
    /// #
    /// # let provider = TestProvider::new();
    /// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
    /// # let place = Place::try_from_degrees(27.7172, 85.324, 0.0)?;
    /// # let solver = Solver::new(&sky, Body::Sun, place, Horizon::UPPER_LIMB_REFRACTION, DeltaTModel::TableThenModel);
    /// # // The local mean day of 2024-06-21 there.
    /// # let midnight = JulianDay::<Ut1>::literal(2_460_482.5 - 85.324 / 360.0);
    ///
    /// use teistro_port_ephemeris::HorizonEventKind;
    ///
    /// let rise = solver.event(HorizonEventKind::Rise, midnight, 1.0)?.expect("a sunrise");
    /// let transit = solver.event(HorizonEventKind::Transit, midnight, 1.0)?.expect("a noon");
    /// assert!(rise.instant.get() < transit.instant.get());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
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
        spot(self.event_with(None, kind, from, window_days))
    }

    /// [`Solver::event`] run beside other searches in lockstep, its
    /// readings asked for in `round`'s requests ([`Lockstep::run`]). It
    /// takes [`Solver::event`]'s steps on [`Solver::event`]'s readings,
    /// so it answers [`Solver::event`]'s answer to the bit.
    ///
    /// ```
    /// # use teistro_astro::rise_set::Solver;
    /// # use teistro_astro::{Completion, DeltaTModel};
    /// # use teistro_core::quantity::{JulianDay, Place, Ut1};
    /// # use teistro_core::settings::OverridePolicy;
    /// # use teistro_port_ephemeris::{Body, Horizon, TestProvider};
    /// #
    /// # let provider = TestProvider::new();
    /// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
    /// # let place = Place::try_from_degrees(27.7172, 85.324, 0.0)?;
    /// # let solver = Solver::new(&sky, Body::Sun, place, Horizon::UPPER_LIMB_REFRACTION, DeltaTModel::TableThenModel);
    /// # // The local mean day of 2024-06-21 there.
    /// # let midnight = JulianDay::<Ut1>::literal(2_460_482.5 - 85.324 / 360.0);
    ///
    /// use teistro_astro::lockstep::Lockstep;
    /// use teistro_port_ephemeris::HorizonEventKind;
    ///
    /// // The rise and the set searched side by side: the answers on the spot, to the bit.
    /// let round = Lockstep::new();
    /// let both = round.run([HorizonEventKind::Rise, HorizonEventKind::Set].map(|kind| solver.event_in(&round, kind, midnight, 1.0)));
    /// assert_eq!(both[0], solver.event(HorizonEventKind::Rise, midnight, 1.0));
    /// assert_eq!(both[1], solver.event(HorizonEventKind::Set, midnight, 1.0));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`Solver::event`].
    pub async fn event_in(
        &self,
        round: &Lockstep<'a>,
        kind: HorizonEventKind,
        from: JulianDay<Ut1>,
        window_days: f64,
    ) -> Result<Option<HorizonEvent>, Error> {
        self.event_with(Some(round), kind, from, window_days).await
    }

    /// The event search, reading on the spot or in a round.
    async fn event_with(
        &self,
        round: Option<&Lockstep<'a>>,
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
        // The engine's own search, when the sky offers one and no terrain
        // stands in for the horizon (the rise and set override).
        if self.terrain.is_none() {
            let request = HorizonRequest {
                body: self.body,
                kind,
                place: self.place,
                from,
                window_days,
                horizon: self.horizon,
                conventions: Conventions::STANDARD.with_delta_t(self.delta_t),
            };
            if let Some(found) = self.sky.native_horizon_event(&request) {
                return found.map(|instant| {
                    instant.map(|instant| HorizonEvent {
                        instant,
                        method: Method::Native,
                        evaluations: 0,
                    })
                });
            }
        }
        let start = from.get();
        let end = start + window_days;
        let mut evaluations = 0u32;
        // The window's first reading, which the iteration starts from and
        // the scan's proofs start from too: read once for both.
        let first = self.sample_in(round, start, &mut evaluations).await?;
        // Behind a terrain the first crossing is wanted, which the scan
        // finds and the iteration does not promise ([`Solver::with_terrain`]).
        let iterated = if self.terrain.is_some() {
            None
        } else {
            self.iterate(round, kind, (start, first), end, &mut evaluations)
                .await?
        };
        match iterated {
            Some(instant) => Ok(Some(HorizonEvent {
                instant: JulianDay::try_new(instant)?,
                method: Method::Iterated,
                evaluations,
            })),
            None => {
                self.scan(round, kind, (start, first), end, evaluations)
                    .await
            }
        }
    }

    /// Whether the body certainly reaches no crossing of the event's
    /// target inside the window.
    ///
    /// An altitude is `sin h = sin φ sin δ + cos φ cos δ cos H`, and over
    /// a rotation `cos H` sweeps −1 to 1, so for a fixed declination the
    /// altitude lies between its two transits:
    ///
    /// ```text
    /// h ≤ 90° − |φ − δ|      upper transit, cos H = 1
    /// h ≥ |φ + δ| − 90°      lower transit, cos H = −1
    /// ```
    ///
    /// If the upper bound is under the target the body never reaches it;
    /// if the lower bound is over it the body never falls to it. Either
    /// way there is no crossing, and no rise and no set.
    ///
    /// **Both bounds take the declination kindest to an event**, because
    /// an absence is only proved when *no* declination the window allows
    /// permits one. Getting either the wrong way round claims absences
    /// that are not: taking the largest `|φ + δ|` rather than the
    /// smallest called fifty-three events absent when the rule was
    /// measured, and widening the margin made that worse rather than
    /// better — which is what said the bound was inverted rather than
    /// merely tight.
    ///
    /// The bounds hold for a *fixed* declination and it moves, so the
    /// range is what the two readings saw widened by
    /// [`declination_chord_margin`]. A body with no measured margin gets
    /// no fast path.
    ///
    /// Only a rise or a set: a transit is a crossing of the hour angle,
    /// which happens once a rotation whatever the altitude does.
    ///
    /// Conservative in the one direction that matters. The transits bound
    /// a **whole** rotation, so a window shorter than one can hold no
    /// event where these allow one; this therefore proves fewer absences
    /// than exist and never one that does not — 60% of them over the
    /// measured sweep, and none of 1768 searches wrongly.
    /// Whether the window certainly holds no crossing of the kind asked
    /// for, proved from a few readings rather than walked: no rise for a
    /// rise, no set for a set. The other kind may happen; a moonset in
    /// what is left of the day does not stop a proof that no second
    /// moonrise is in it.
    ///
    /// The gap `g` (the altitude less the event altitude) moves no faster
    /// than [`ALTITUDE_RATE_CEILING_DEG_PER_DAY`], so a reading `|g|`
    /// degrees from the event altitude proves the gap keeps its sign for
    /// `|g|` over that rate, and the next reading is taken there. Far from
    /// the horizon a reading covers hours.
    ///
    /// Close to it the reading's slope decides. The Earth's turn moves
    /// the altitude at `−ω cos φ cos δ sin H / cos h`, nothing else moves
    /// the gap by more than [`OWN_RATE_CEILING_DEG_PER_DAY`], and within
    /// [`LOW_ALTITUDE_DEG`] of the horizon the slope changes by no more
    /// than [`SLOPE_CHANGE_CEILING_DEG_PER_DAY2`]. So at a slope `s` the
    /// gap is monotone for `s` over that ceiling, and crosses at most
    /// once, in the slope's direction. A crossing in the direction not
    /// asked for is allowed through; one in the direction asked for, or
    /// a slope too shallow to tell, leaves the window to the walk.
    ///
    /// When the proof passes the window's end the walk would find no
    /// crossing of its kind, so the answer is the walk's: no event.
    async fn proves_none(
        &self,
        round: Option<&Lockstep<'a>>,
        kind: HorizonEventKind,
        (start, first): (f64, Sample),
        end: f64,
        evaluations: &mut u32,
    ) -> Result<bool, Error> {
        let upward = match kind {
            HorizonEventKind::Rise => true,
            HorizonEventKind::Set => false,
            _ => return Ok(false),
        };
        // The rate ceilings bound the altitude's motion, not a skyline's
        // under a moving azimuth: no proof behind a terrain.
        if self.terrain.is_some() {
            return Ok(false);
        }
        let (_, cos_phi) = math::sin_cos(self.place.latitude.get() * DEG2RAD);
        let mut t = start;
        for step in 0..ONE_SIDE_READINGS {
            let reading = if step == 0 {
                // The window's first reading, counted as the walk's.
                *evaluations += 1;
                first
            } else {
                self.sample_in(round, t, evaluations).await?
            };
            let gap = reading.altitude_deg - reading.target_deg;
            let reach = if gap.abs() >= 2.0 * ONE_SIDE_FLOOR_DEG {
                // Short of the full reach by the floor, so the next
                // reading is inside the stretch this one proved, and a
                // gap closing on zero hands over to the slope below
                // rather than closing on the floor in ever smaller steps.
                (gap.abs() - ONE_SIDE_FLOOR_DEG) / ALTITUDE_RATE_CEILING_DEG_PER_DAY
            } else {
                if reading.altitude_deg.abs() > LOW_ALTITUDE_DEG / 2.0 {
                    return Ok(false);
                }
                let turn = -HOUR_ANGLE_RATE_DEG_PER_DAY
                    * cos_phi
                    * math::cos(reading.dec_deg * DEG2RAD)
                    * math::sin(reading.hour_angle_deg * DEG2RAD)
                    / math::cos(reading.altitude_deg * DEG2RAD);
                let steep = turn.abs() - OWN_RATE_CEILING_DEG_PER_DAY;
                if steep <= 0.0 {
                    return Ok(false);
                }
                let rising = turn > 0.0;
                // A crossing of the asked-for kind is possible only when
                // the gap moves that way from the far side of zero.
                let towards = rising == upward;
                if towards && !(if upward { gap > 0.0 } else { gap < 0.0 }) {
                    return Ok(false);
                }
                (steep / SLOPE_CHANGE_CEILING_DEG_PER_DAY2).min(LOW_STRETCH_DAYS)
            };
            if t + reach >= end {
                return Ok(true);
            }
            t += reach;
        }
        Ok(false)
    }

    async fn cannot_cross(
        &self,
        round: Option<&Lockstep<'a>>,
        kind: HorizonEventKind,
        (start, first): (f64, Sample),
        end: f64,
        evaluations: &mut u32,
    ) -> Result<bool, Error> {
        // The transits bound a fixed target, not a skyline's under a
        // moving azimuth: no bound behind a terrain.
        if !matches!(kind, HorizonEventKind::Rise | HorizonEventKind::Set) || self.terrain.is_some()
        {
            return Ok(false);
        }
        // A whole rotation, or nothing. The transits bound the altitude
        // over one, so a shorter window can hold no event where they
        // allow one and the bound can never prove an absence in it — it
        // would only cost two readings to say nothing.
        //
        // This is not a refinement, it is what makes the fast path pay.
        // An almanac's event loop searches a **partial** window every
        // time it ends, which is the very case that dominates its
        // absences, and at any temperate latitude the Moon rises every
        // day so the bound would never fire there anyway. Without this
        // guard the check cost an almanac 400 calls over fifty days and
        // saved none of them.
        if end - start < 360.0 / HOUR_ANGLE_RATE_DEG_PER_DAY {
            return Ok(false);
        }
        let Some(margin) = declination_chord_margin(self.body) else {
            return Ok(false);
        };
        // The window's first reading, counted as this bound's.
        *evaluations += 1;
        let last = self.sample_in(round, end, evaluations).await?;

        let (dec_lo, dec_hi) = (
            first.dec_deg.min(last.dec_deg) - margin,
            first.dec_deg.max(last.dec_deg) + margin,
        );
        let latitude = self.place.latitude.get();
        // The highest an upper transit could reach: the declination
        // nearest the latitude, since `90° − |φ − δ|` grows as they close.
        let nearest = latitude.clamp(dec_lo, dec_hi);
        let highest = 90.0 - (latitude - nearest).abs();
        // The deepest a lower transit could fall: the declination making
        // `|φ + δ|` smallest, since `|φ + δ| − 90°` falls as it shrinks.
        let shallowest = (-latitude).clamp(dec_lo, dec_hi);
        let lowest = (latitude + shallowest).abs() - 90.0;
        // The kindest target either reading offers, so an absence holds
        // for both.
        let (target_lo, target_hi) = (
            first.target_deg.min(last.target_deg),
            first.target_deg.max(last.target_deg),
        );
        Ok(highest < target_lo || lowest > target_hi)
    }

    /// Meeus's iteration from the hour-angle estimate: `Some(instant)`
    /// when it converged inside the window, `Ok(None)` when it did not
    /// settle or its rate factor vanished, which hands the search to the
    /// scan.
    async fn iterate(
        &self,
        round: Option<&Lockstep<'a>>,
        kind: HorizonEventKind,
        (start, first): (f64, Sample),
        end: f64,
        evaluations: &mut u32,
    ) -> Result<Option<f64>, Error> {
        let sidereal_day = 360.0 / HOUR_ANGLE_RATE_DEG_PER_DAY;
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
            let sample = self.sample_in(round, t, evaluations).await?;
            let correction = match kind {
                HorizonEventKind::Transit | HorizonEventKind::Antitransit => {
                    difference_deg(target_hour_angle, sample.hour_angle_deg)
                        / HOUR_ANGLE_RATE_DEG_PER_DAY
                }
                HorizonEventKind::Rise | HorizonEventKind::Set => {
                    let (_, cos_phi) = math::sin_cos(self.place.latitude.get() * DEG2RAD);
                    let cos_dec = math::cos(sample.dec_deg * DEG2RAD);
                    let factor = cos_dec * cos_phi * math::sin(sample.hour_angle_deg * DEG2RAD);
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
    async fn scan(
        &self,
        round: Option<&Lockstep<'a>>,
        kind: HorizonEventKind,
        (start, first): (f64, Sample),
        end: f64,
        evaluations: u32,
    ) -> Result<Option<HorizonEvent>, Error> {
        // Before walking, try to show there is nothing to walk for. Two
        // readings against a hundred and forty-four, and an almanac's
        // event loop ends by searching for a rise that is not there,
        // twice a day (`03-design/horizon-absence-measured.md`).
        //
        // **Here rather than before the iteration**, which is where it
        // was first put and where it cost more than it saved: an event
        // that iterates cleanly is the common case at any temperate
        // latitude, and making it pay two readings for a proof it does
        // not need took an almanac from 19 632 calls to 22 020. Only a
        // search that is about to walk can be worth two readings, so only
        // a search that is about to walk pays them.
        let mut counted = evaluations;
        if self.absence_check
            && self
                .cannot_cross(round, kind, (start, first), end, &mut counted)
                .await?
        {
            return Ok(None);
        }
        if self.none_proof
            && self
                .proves_none(round, kind, (start, first), end, &mut counted)
                .await?
        {
            return Ok(None);
        }
        let evaluations = counted;

        let quantity = |sample: &Sample| -> f64 {
            match kind {
                HorizonEventKind::Rise | HorizonEventKind::Set => {
                    sample.altitude_deg - sample.target_deg
                }
                HorizonEventKind::Transit => sample.hour_angle_deg,
                HorizonEventKind::Antitransit => difference_deg(sample.hour_angle_deg, 180.0),
            }
        };
        // The scan's instants are known before the first is asked for, so
        // they are asked for a grid at a time; only the narrowing stays
        // serial. This is where the SDK's remaining ephemeris calls were:
        // the almanac's event loop proves a window holds no more rises by
        // scanning the whole of it, twice a day
        // (`07-roadmap/02-plan-performance-and-passthrough.md`, A1d).
        let counted = core::cell::Cell::new(evaluations);
        let grid = async |instants: &[f64], out: &mut Vec<f64>| -> Result<(), Error> {
            let mut used = counted.get();
            let samples = self.samples_in(round, instants, &mut used).await?;
            counted.set(used);
            out.clear();
            out.extend(samples.iter().map(&quantity));
            Ok(())
        };
        let one = async |t: f64| -> Result<f64, Error> {
            let mut used = counted.get();
            let sample = self.sample_in(round, t, &mut used).await?;
            counted.set(used);
            Ok(quantity(&sample))
        };
        let upward = kind != HorizonEventKind::Set;
        let scan = Scan::new(start, end, SCAN_STEP_DAYS, upward, TOLERANCE_DAYS)
            .with_caps(scan_caps(end - start))
            .with_chunk(self.chunk);
        let crossing = first_zero_gridded_async(grid, one, &scan)
            .await
            .map_err(|error| {
                error.into_error(|| {
                    format!(
                        "the {} of {} at {} from JD {start}",
                        kind.key().to_ascii_lowercase(),
                        self.body.key(),
                        self.place
                    )
                })
            })?;
        let count = counted.get();
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
    /// ```
    /// # use teistro_astro::rise_set::Solver;
    /// # use teistro_astro::{Completion, DeltaTModel};
    /// # use teistro_core::quantity::{JulianDay, Place, Ut1};
    /// # use teistro_core::settings::OverridePolicy;
    /// # use teistro_port_ephemeris::{Body, Horizon, TestProvider};
    /// #
    /// # let provider = TestProvider::new();
    /// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
    /// # let place = Place::try_from_degrees(27.7172, 85.324, 0.0)?;
    /// # let solver = Solver::new(&sky, Body::Sun, place, Horizon::UPPER_LIMB_REFRACTION, DeltaTModel::TableThenModel);
    /// # // The local mean day of 2024-06-21 there.
    /// # let midnight = JulianDay::<Ut1>::literal(2_460_482.5 - 85.324 / 360.0);
    ///
    /// let day = solver.day(midnight)?;
    /// // A temperate day: a rise, a set, and the Sun up at its middle.
    /// assert!(day.rise.is_some() && day.set.is_some() && day.above_at_midday);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`Solver::event`].
    pub fn day(&self, midnight: JulianDay<Ut1>) -> Result<DayEvents, Error> {
        spot(self.day_with(None, midnight))
    }

    /// [`Solver::day`] run beside other searches in lockstep
    /// ([`Solver::event_in`]), answering [`Solver::day`]'s answer to the
    /// bit.
    ///
    /// ```
    /// # use teistro_astro::rise_set::Solver;
    /// # use teistro_astro::{Completion, DeltaTModel};
    /// # use teistro_core::quantity::{JulianDay, Place, Ut1};
    /// # use teistro_core::settings::OverridePolicy;
    /// # use teistro_port_ephemeris::{Body, Horizon, TestProvider};
    /// #
    /// # let provider = TestProvider::new();
    /// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
    /// # let place = Place::try_from_degrees(27.7172, 85.324, 0.0)?;
    /// # let solver = Solver::new(&sky, Body::Sun, place, Horizon::UPPER_LIMB_REFRACTION, DeltaTModel::TableThenModel);
    /// # // The local mean day of 2024-06-21 there.
    /// # let midnight = JulianDay::<Ut1>::literal(2_460_482.5 - 85.324 / 360.0);
    ///
    /// use teistro_astro::lockstep::Lockstep;
    ///
    /// // Three days in one run, each the day `day` answers.
    /// let days = [0.0, 1.0, 2.0].map(|d| JulianDay::<Ut1>::literal(midnight.get() + d));
    /// let round = Lockstep::new();
    /// let found = round.run(days.map(|day| solver.day_in(&round, day)));
    /// for (day, events) in days.iter().zip(found) {
    ///     assert_eq!(events?, solver.day(*day)?);
    /// }
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`Solver::day`].
    pub async fn day_in(
        &self,
        round: &Lockstep<'a>,
        midnight: JulianDay<Ut1>,
    ) -> Result<DayEvents, Error> {
        self.day_with(Some(round), midnight).await
    }

    async fn day_with(
        &self,
        round: Option<&Lockstep<'a>>,
        midnight: JulianDay<Ut1>,
    ) -> Result<DayEvents, Error> {
        let rise = self
            .event_with(round, HorizonEventKind::Rise, midnight, 1.0)
            .await?;
        let set = match rise {
            Some(rise) => {
                self.event_with(round, HorizonEventKind::Set, rise.instant, 1.0)
                    .await?
            }
            None => {
                self.event_with(round, HorizonEventKind::Set, midnight, 1.0)
                    .await?
            }
        };
        let mut count = 0u32;
        let midday = self
            .sample_in(round, midnight.get() + 0.5, &mut count)
            .await?;
        Ok(DayEvents {
            rise,
            set,
            above_at_midday: midday.altitude_deg > midday.target_deg,
        })
    }
}

/// A search read on the spot: one poll finishes it, since nothing it
/// awaits waits; a search that did wait is a defect, reported rather than
/// spun on.
fn spot<T>(search: impl core::future::Future<Output = Result<T, Error>>) -> Result<T, Error> {
    finished(search).unwrap_or_else(|| {
        Err(Error::internal(
            "a horizon search read on the spot waited for a round",
        ))
    })
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
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

    impl Solver<'_> {
        /// The scan alone, read on the spot.
        fn scan_now(
            &self,
            kind: HorizonEventKind,
            start: f64,
            end: f64,
            evaluations: u32,
        ) -> Result<Option<HorizonEvent>, Error> {
            let first = self.sample(start, &mut 0)?;
            spot(self.scan(None, kind, (start, first), end, evaluations))
        }

        /// The absence proof alone, read on the spot.
        fn proves_none_now(
            &self,
            kind: HorizonEventKind,
            start: f64,
            end: f64,
            evaluations: &mut u32,
        ) -> Result<bool, Error> {
            let first = self.sample(start, &mut 0)?;
            spot(self.proves_none(None, kind, (start, first), end, evaluations))
        }
    }

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
                .scan_now(kind, from.get(), from.get() + 1.0, 0)
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
            .scan_now(HorizonEventKind::Transit, from.get(), from.get() + 1.0, 0)
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
    fn a_proved_absence_is_what_the_walk_finds() {
        // Partial windows, as an almanac's event loop leaves them, from
        // the equator to past the polar circle, for the Sun and the Moon:
        // with the proof and without it the solver answers the same, to
        // the bit, rise and set alike.
        let provider = TestProvider::new();
        let sky = Completion::new(
            &provider,
            OverridePolicy::SdkOnly,
            DeltaTModel::TableThenModel,
        );
        let mut proved = 0_u32;
        for latitude in [-45.0, 0.0, 27.7, 50.0, 60.0, 64.0, 66.0, 70.0, 78.0] {
            for body in [Body::Moon, Body::Sun] {
                let at = place(latitude, 85.3);
                let solver = || {
                    Solver::new(
                        &sky,
                        body,
                        at,
                        Horizon::UPPER_LIMB_REFRACTION,
                        DeltaTModel::TableThenModel,
                    )
                };
                let walker = solver().with_none_proof(false);
                let solver = solver();
                for day in 0..20 {
                    for (offset, length) in [(0.13, 0.62), (0.47, 0.91), (0.71, 0.33)] {
                        let from =
                            JulianDay::<Ut1>::literal(2_460_310.5 + f64::from(day) * 1.37 + offset);
                        for kind in [HorizonEventKind::Rise, HorizonEventKind::Set] {
                            let fast = solver.event(kind, from, length).unwrap();
                            let slow = walker.event(kind, from, length).unwrap();
                            assert_eq!(
                                fast.map(|e| e.instant.get().to_bits()),
                                slow.map(|e| e.instant.get().to_bits()),
                                "{body:?} {kind:?} at {latitude} from {from} for {length}"
                            );
                            let mut count = 0;
                            if fast.is_none()
                                && solver
                                    .proves_none_now(
                                        kind,
                                        from.get(),
                                        from.get() + length,
                                        &mut count,
                                    )
                                    .unwrap()
                            {
                                proved += 1;
                            }
                        }
                    }
                }
            }
        }
        // Most windows without an event are proved rather than walked.
        assert!(proved > 300, "{proved}");
    }

    #[test]
    fn an_air_refracts_the_horizon_by_the_place_and_the_weather() {
        use teistro_core::settings::{Atmosphere, Sunrise, SunriseConvention};
        let provider = TestProvider::new();
        let sky = Completion::new(
            &provider,
            OverridePolicy::SdkOnly,
            DeltaTModel::TableThenModel,
        );
        let at = |metres: f64| Place {
            altitude: Altitude::literal(metres),
            ..place(27.7172, 85.324)
        };
        let midnight = JulianDay::literal(2_460_482.5 - 85.324 / 360.0);
        let air = |air| {
            Horizon::from_convention(SunriseConvention::Atmospheric {
                which: Sunrise::UpperLimbRefraction,
                air,
            })
        };
        let rise = |horizon: Horizon, place: Place| {
            Solver::new(&sky, Body::Sun, place, horizon, DeltaTModel::TableThenModel)
                .day(midnight)
                .unwrap()
                .arc()
                .unwrap()
                .0
                .get()
        };
        let seconds = |later: f64, earlier: f64| (later - earlier) * 86_400.0;
        // At 1400 m the standard air is 856 hPa, which refracts 28.7′ and
        // not the almanac's 34′: the Sun clears the horizon later, by what
        // the engines' own search moves with the height (26 s measured).
        let height = at(1400.0);
        let thin = rise(air(Atmosphere::STANDARD), height);
        let almanac = rise(Horizon::UPPER_LIMB_REFRACTION, height);
        assert!(
            (20.0..35.0).contains(&seconds(thin, almanac)),
            "{}",
            seconds(thin, almanac)
        );
        // An air given as the standard resolves to is the same air.
        let resolved = Atmosphere::STANDARD.at(height.altitude);
        let given = rise(
            air(Atmosphere::given(
                resolved.pressure_hpa,
                resolved.temperature_c,
            )),
            height,
        );
        assert_eq!(given.to_bits(), thin.to_bits());
        // At sea level Bennett's air refracts 0.48′ more than the
        // almanac's: a second or two earlier, never the other way.
        let sea = at(0.0);
        let bennett = rise(air(Atmosphere::given(1010.0, 10.0)), sea);
        let almanac = rise(Horizon::UPPER_LIMB_REFRACTION, sea);
        assert!(
            (0.5..4.0).contains(&seconds(almanac, bennett)),
            "{}",
            seconds(almanac, bennett)
        );
        // Cold dense air refracts more, and the Sun is seen sooner.
        let cold = rise(air(Atmosphere::given(1040.0, -30.0)), sea);
        assert!(cold < bennett);
    }

    #[test]
    fn a_disc_follows_the_models_size_and_radii() {
        let standard = HorizonModel::STANDARD;
        // The actual disc at the body's distance, from the IAU 2015 radius.
        let sun = Disc::under(Body::Sun, 1.0, &standard);
        assert!((sun.semidiameter_deg * 3600.0 - 959.2).abs() < 0.1);
        assert!(sun.parallax_deg > 0.0);
        // A custom radius is the body's own: the Sun's for the Sun, the
        // Moon's for the Moon, and a planet keeps its catalogue radius.
        let custom = HorizonModel { radii: Radii::Custom { sun_km: 696_000.0, moon_km: 1_738.0 }, ..standard };
        assert!(Disc::under(Body::Sun, 1.0, &custom).semidiameter_deg > sun.semidiameter_deg);
        let moon = Disc::under(Body::Moon, MOON_MEAN_DISTANCE_AU, &standard);
        assert!(Disc::under(Body::Moon, MOON_MEAN_DISTANCE_AU, &custom).semidiameter_deg > moon.semidiameter_deg);
        assert_eq!(Disc::under(Body::Mars, 1.5, &custom), Disc::under(Body::Mars, 1.5, &standard));
        // At the mean distance whatever the actual one.
        let mean = HorizonModel { disc_size: DiscSize::MeanDistance, ..standard };
        assert_eq!(Disc::under(Body::Sun, 0.983, &mean).semidiameter_deg, sun.semidiameter_deg);
        assert_eq!(Disc::under(Body::Moon, 0.0024, &mean).semidiameter_deg, moon.semidiameter_deg);
        // A fixed angle for any body with a size, and none for a point.
        let fixed = HorizonModel { disc_size: DiscSize::Fixed { semidiameter_arcmin: 16.0 }, ..standard };
        assert!((Disc::under(Body::Moon, 0.0024, &fixed).semidiameter_deg - 16.0 / 60.0).abs() < 1e-12);
        let none = Disc::under(Body::MeanNode, 1.0, &fixed);
        assert_eq!((none.semidiameter_deg, none.parallax_deg), (0.0, 0.0));
        let nowhere = Disc::under(Body::Sun, 0.0, &standard);
        assert_eq!((nowhere.semidiameter_deg, nowhere.parallax_deg), (0.0, 0.0));
    }

    #[test]
    fn the_dip_and_the_refraction_formulas_reckon_as_their_sources_do() {
        let air = Atmosphere::STANDARD;
        let sea = Altitude::literal(0.0);
        let hill = Altitude::literal(1_400.0);
        assert_eq!(dip_deg(Dip::None, hill, air), 0.0);
        assert_eq!(dip_deg(Dip::Geometric, sea, air), 0.0);
        let geometric = dip_deg(Dip::Geometric, hill, air);
        // About 1.2 degrees from 1,400 m, and refraction lifts it a little.
        assert!((geometric + 1.2).abs() < 0.05, "{geometric}");
        let refracted = dip_deg(Dip::Refracted, hill, air);
        assert!(refracted > geometric && refracted < 0.0, "{refracted}");
        // Below the ground is the ground.
        assert_eq!(dip_deg(Dip::Geometric, Altitude::literal(-100.0), air), 0.0);
        // Saemundsson near the horizon, Sinclair on both sides of its seam.
        assert!((saemundsson_refraction_arcmin(0.0) - 29.0).abs() < 1.0);
        let low = sinclair_refraction_arcmin(17.9, 1013.25, 15.0);
        let high = sinclair_refraction_arcmin(17.91, 1013.25, 15.0);
        assert!((low - high).abs() < 0.01, "{low} {high}");
        assert!(sinclair_refraction_arcmin(45.0, 1013.25, 15.0) < 1.0);
        assert_eq!(horizon_refraction_arcmin(Refraction::None, hill), 0.0);
        assert_eq!(horizon_refraction_arcmin(Refraction::Standard, hill), STANDARD_HORIZON_REFRACTION_ARCMIN);
        assert!(horizon_refraction_arcmin(Refraction::Atmosphere(air), sea) > 30.0);
    }

    #[test]
    fn each_formula_reckons_the_horizon_and_bennett_refuses_past_its_pole() {
        let hill = Altitude::literal(1_400.0);
        let named = |refraction: RefractionFormula, dip: Dip, altitude_deg: f64| Horizon {
            refraction: Refraction::Atmosphere(Atmosphere::STANDARD),
            model: HorizonModel { refraction, dip, altitude_deg, ..HorizonModel::STANDARD },
            ..Horizon::UPPER_LIMB_REFRACTION
        };
        let mut seen = Vec::new();
        for formula in [RefractionFormula::Bennett, RefractionFormula::Saemundsson, RefractionFormula::Sinclair] {
            let reckoned = reckon(&named(formula, Dip::None, 0.0), hill).unwrap();
            assert_eq!(reckoned.height_deg, 0.0);
            // Each near half a degree at the horizon, thinner air at height.
            assert!((0.3..0.6).contains(&reckoned.refraction_deg), "{formula:?}: {}", reckoned.refraction_deg);
            seen.push(reckoned.refraction_deg);
        }
        assert!(seen.windows(2).any(|pair| pair[0] != pair[1]), "three formulas, three answers");
        // The dip lowers the horizon, and the altitude raises it.
        let dipped = reckon(&named(RefractionFormula::Bennett, Dip::Geometric, 0.5), hill).unwrap();
        assert!((dipped.height_deg - (0.5 + dip_deg(Dip::Geometric, hill, Atmosphere::STANDARD))).abs() < 1e-12);
        // Bennett's pole: refused, naming the setting; the others answer.
        let error = reckon(&named(RefractionFormula::Bennett, Dip::None, -4.2), hill).unwrap_err();
        assert_eq!(error.field(), Some("day.horizon"));
        assert!(reckon(&named(RefractionFormula::Sinclair, Dip::None, -4.2), hill).is_ok());
        // A refraction that names no air is the convention's own.
        let standard = reckon(&Horizon::UPPER_LIMB_REFRACTION, hill).unwrap();
        assert!((standard.refraction_deg - STANDARD_HORIZON_REFRACTION_ARCMIN / 60.0).abs() < 1e-12);
        let geometric = reckon(&Horizon::CENTRE_NO_REFRACTION, hill).unwrap();
        assert_eq!(geometric.refraction_deg, 0.0);
    }

    #[test]
    fn the_hindu_rising_is_the_ecliptic_points() {
        let provider = TestProvider::new();
        let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
        let kathmandu = place(27.7172, 85.324);
        let midnight = JulianDay::literal(2_460_482.5 - 85.324 / 360.0);
        let rise = |latitude| {
            let horizon = Horizon {
                model: HorizonModel { latitude, ..HorizonModel::STANDARD },
                ..Horizon::UPPER_LIMB_REFRACTION
            };
            Solver::new(&sky, Body::Moon, kathmandu, horizon, DeltaTModel::TableThenModel)
                .event(HorizonEventKind::Rise, midnight, 2.0)
                .unwrap()
                .unwrap()
                .instant
                .get()
        };
        // The test sky's Moon stands off the ecliptic, so the point under
        // it rises at another instant, within minutes of it.
        let (body, point) = (rise(RisingLatitude::Body), rise(RisingLatitude::Zero));
        assert!(body != point && (body - point).abs() * 1_440.0 < 30.0, "{body} {point}");
        let on = on_the_ecliptic(&Apparent { ra_deg: 10.0, dec_deg: -20.0, distance_au: 1.0 }, 23.44);
        let back = equatorial_to_ecliptic(Spherical { lon_deg: on.ra_deg, lat_deg: on.dec_deg }, 23.44);
        assert!(back.lat_deg.abs() < 1e-9);
    }

    /// The test sky, declaring the rise and set override and answering
    /// every question with one instant.
    struct Answering;

    impl teistro_port_ephemeris::EphemerisProvider for Answering {
        fn capabilities(&self) -> teistro_port_ephemeris::Capabilities {
            teistro_port_ephemeris::Capabilities {
                overrides: teistro_port_ephemeris::Overrides::RISE_SET,
                ..TestProvider::new().capabilities()
            }
        }

        fn positions(
            &self,
            request: &teistro_port_ephemeris::PositionRequest<'_>,
        ) -> Result<teistro_port_ephemeris::PositionColumns, teistro_port_ephemeris::ProviderError> {
            TestProvider::new().positions(request)
        }

        fn horizon_event(
            &self,
            request: &HorizonRequest,
        ) -> Result<Option<JulianDay<Ut1>>, teistro_port_ephemeris::ProviderError> {
            Ok(Some(JulianDay::literal(request.from.get() + 0.25)))
        }
    }

    #[test]
    fn under_prefer_native_an_engine_that_declares_rise_and_set_answers_it() {
        let provider = Answering;
        let midnight = JulianDay::literal(2_460_482.5);
        let kathmandu = place(27.7172, 85.324);
        for (policy, native) in [(OverridePolicy::PreferNative, true), (OverridePolicy::SdkOnly, false)] {
            let sky = Completion::new(&provider, policy, DeltaTModel::TableThenModel);
            // Through a reference too, as an area holds its sky.
            let held = &sky;
            let found = Solver::new(&held, Body::Sun, kathmandu, Horizon::UPPER_LIMB_REFRACTION, DeltaTModel::TableThenModel)
                .event(HorizonEventKind::Rise, midnight, 1.0)
                .unwrap()
                .unwrap();
            assert_eq!(found.method == Method::Native, native, "{policy:?}");
            if native {
                assert_eq!((found.instant.get(), found.evaluations), (midnight.get() + 0.25, 0));
            }
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
    fn a_month_of_moonrises_in_lockstep_is_the_month_read_alone() {
        let provider = TestProvider::new();
        let sky = Completion::new(
            &provider,
            OverridePolicy::SdkOnly,
            DeltaTModel::TableThenModel,
        );
        let solver = Solver::new(
            &sky,
            Body::Moon,
            place(27.7172, 85.324),
            Horizon::UPPER_LIMB_REFRACTION,
            DeltaTModel::TableThenModel,
        );
        let days: Vec<JulianDay<Ut1>> = (0..30)
            .map(|d| JulianDay::literal(2_460_482.5 + f64::from(d)))
            .collect();
        for kind in [
            HorizonEventKind::Rise,
            HorizonEventKind::Set,
            HorizonEventKind::Transit,
        ] {
            let alone: Vec<_> = days
                .iter()
                .map(|day| solver.event(kind, *day, 1.0))
                .collect();
            let lockstep = Lockstep::new();
            let together = lockstep.run(
                days.iter()
                    .map(|day| solver.event_in(&lockstep, kind, *day, 1.0)),
            );
            assert_eq!(alone, together, "{kind:?}");
            // The Moon's day is longer than the Earth's, so one day of
            // the month has none of each.
            let none = alone
                .iter()
                .filter(|event| event.as_ref().unwrap().is_none())
                .count();
            assert_eq!(none, 1, "{kind:?}");
        }
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
        let sea = Altitude::try_new(0.0).unwrap();
        let lower = centre_altitude_deg(&Horizon::LOWER_LIMB_REFRACTION, &sun, sea).unwrap();
        let upper = centre_altitude_deg(&Horizon::UPPER_LIMB_REFRACTION, &sun, sea).unwrap();
        assert!((lower - upper - 2.0 * sun.semidiameter_deg).abs() < 1e-12);
        assert_eq!(radius_km(Body::MeanApogee), 0.0);
    }

    /// The gridded scan answers what the walk answered, to the bit.
    ///
    /// The scan asks for its instants a chunk at a time (A1d of
    /// `07-roadmap/02-plan-performance-and-passthrough.md`), and a chunk
    /// of one is the walk it replaced. The instants are the same
    /// instants and the values the same values, so the bracket handed to
    /// the narrowing is the same bracket and the crossing is the same
    /// crossing — not close, identical.
    #[test]
    fn a_gridded_scan_answers_what_the_walk_answered_to_the_bit() {
        let star = FixedStar { dec_deg: 20.35 };
        let from = JulianDay::<Ut1>::literal(2_451_545.0);
        for kind in [
            HorizonEventKind::Rise,
            HorizonEventKind::Set,
            HorizonEventKind::Transit,
            HorizonEventKind::Antitransit,
        ] {
            let walked = Solver::new(
                &star,
                Body::MeanNode,
                place(69.6, 18.9),
                Horizon::CENTRE_NO_REFRACTION,
                DeltaTModel::TableThenModel,
            )
            .with_chunk(1)
            .scan_now(kind, from.get(), from.get() + 1.0, 0)
            .unwrap()
            .unwrap();
            for chunk in [2usize, 7, 32, 144, 10_000] {
                let gridded = Solver::new(
                    &star,
                    Body::MeanNode,
                    place(69.6, 18.9),
                    Horizon::CENTRE_NO_REFRACTION,
                    DeltaTModel::TableThenModel,
                )
                .with_chunk(chunk)
                .scan_now(kind, from.get(), from.get() + 1.0, 0)
                .unwrap()
                .unwrap();
                assert_eq!(
                    gridded.instant.get().to_bits(),
                    walked.instant.get().to_bits(),
                    "{kind} chunk {chunk}: {} against {}",
                    gridded.instant,
                    walked.instant
                );
                assert_eq!(gridded.method, walked.method);
                // A chunk may carry instants past the one that brackets
                // the crossing, which the walk would never have asked
                // for, so the readings taken can exceed the walk's by up
                // to a chunk less one. What must not differ is the
                // answer, and it does not.
                let extra = gridded.evaluations.saturating_sub(walked.evaluations);
                assert!(
                    gridded.evaluations >= walked.evaluations
                        && usize::try_from(extra).unwrap_or(usize::MAX) < chunk,
                    "{kind} chunk {chunk}: {} readings against the walk's {}",
                    gridded.evaluations,
                    walked.evaluations
                );
            }
        }
    }

    /// An absence costs a walk of the whole window, which is what the
    /// almanac's event loop does twice a day; the grid is what makes it
    /// affordable.
    /// The public absence proof settles a star that never rises, declines
    /// a transit (which always happens) and refuses an empty window by
    /// name; at the pole a star's altitude never changes, so the hour
    /// angle of a rise is undefined and no rise is found; a star a
    /// twentieth of a degree from the celestial pole, seen from the
    /// equator, crosses the horizon too slowly for the iteration and the
    /// scan finds its rise instead; and the lockstep day is the day.
    #[test]
    fn the_edge_skies_are_each_answered_by_a_fixed_example() {
        let from = JulianDay::<Ut1>::literal(2_451_545.0 - 0.5);
        let southern = FixedStar { dec_deg: -80.0 };
        let tromso = Solver::new(
            &southern,
            Body::MeanNode,
            place(69.6, 18.9),
            Horizon::CENTRE_NO_REFRACTION,
            DeltaTModel::TableThenModel,
        );
        assert!(
            tromso
                .proves_no_event(HorizonEventKind::Rise, from, 1.0)
                .unwrap()
        );
        assert!(
            !tromso
                .proves_no_event(HorizonEventKind::Transit, from, 1.0)
                .unwrap()
        );
        let err = tromso
            .proves_no_event(HorizonEventKind::Rise, from, 0.0)
            .unwrap_err();
        assert_eq!(err.field(), Some("window_days"));

        let star = FixedStar { dec_deg: 30.0 };
        let pole = Solver::new(
            &star,
            Body::MeanNode,
            place(90.0, 0.0),
            Horizon::CENTRE_NO_REFRACTION,
            DeltaTModel::TableThenModel,
        );
        assert!(
            pole.event(HorizonEventKind::Rise, from, 1.0)
                .unwrap()
                .is_none()
        );

        let near_pole = FixedStar { dec_deg: 89.95 };
        let equator = Solver::new(
            &near_pole,
            Body::MeanNode,
            place(0.0, 0.0),
            Horizon::CENTRE_NO_REFRACTION,
            DeltaTModel::TableThenModel,
        );
        let transit = equator
            .event(HorizonEventKind::Transit, from, 1.0)
            .unwrap()
            .unwrap();
        let rise = equator
            .event(HorizonEventKind::Rise, from, 1.0)
            .unwrap()
            .unwrap();
        let quarter = 90.0 / HOUR_ANGLE_RATE_DEG_PER_DAY;
        let early = transit.instant.get() - rise.instant.get();
        assert!(
            (early - quarter).abs() < 1e-3,
            "{early} days before transit"
        );
        assert_eq!(rise.method, Method::Scanned);

        let equator_star = FixedStar { dec_deg: 0.0 };
        let solver = Solver::new(
            &equator_star,
            Body::MeanNode,
            place(0.0, 0.0),
            Horizon::CENTRE_NO_REFRACTION,
            DeltaTModel::TableThenModel,
        );
        let days = [0.0, 1.0, 2.0].map(|d| JulianDay::<Ut1>::literal(from.get() + d));
        let round = Lockstep::new();
        let found = round.run(days.map(|day| solver.day_in(&round, day)));
        for (day, events) in days.iter().zip(found) {
            assert_eq!(events.unwrap(), solver.day(*day).unwrap());
        }
    }

    #[test]
    fn proving_an_absence_walks_the_window_and_the_grid_carries_it() {
        // A star that never rises at this latitude: the scan finds
        // nothing and has to look everywhere to say so.
        let star = FixedStar { dec_deg: -80.0 };
        let from = JulianDay::<Ut1>::literal(2_451_545.0);
        let solver = |chunk: usize| {
            Solver::new(
                &star,
                Body::MeanNode,
                place(69.6, 18.9),
                Horizon::CENTRE_NO_REFRACTION,
                DeltaTModel::TableThenModel,
            )
            .with_chunk(chunk)
        };
        for chunk in [1usize, 32, 10_000] {
            let absent = solver(chunk)
                .scan_now(HorizonEventKind::Rise, from.get(), from.get() + 1.0, 0)
                .unwrap();
            assert!(absent.is_none(), "chunk {chunk}: it never rises");
        }
    }
}
