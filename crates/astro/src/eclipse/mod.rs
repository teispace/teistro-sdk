//! Eclipses (`docs/03-design/eclipses.md`): every solar and lunar eclipse
//! in a window, read from the Sun's and the Moon's apparent places alone.
//! A lunar eclipse's kind, gamma, umbral and penumbral magnitudes and its
//! six contacts under a named rule for the Earth's shadow; a solar
//! eclipse's kind (partial, annular, total or hybrid), gamma, magnitude
//! and the place on the Earth where it is greatest. Held against NASA's
//! Five Millennium Canon for 1900 to 2100 (`eclipses-measured.md`).
//!
//! The geometry is the shadow's: each body is placed by the path its light
//! took ([`ShadowSource`]). A solar eclipse reads both bodies astrometric,
//! the Moon's shadow being where the light left the Moon; a lunar eclipse
//! reads both apparent, the Earth's shadow falling where the Earth was
//! when the light that casts it passed, which is the Moon's aberration.
//!
//! [`Eclipses::solar_seen`] and [`Eclipses::lunar_seen`] give an eclipse
//! as one place sees it: the local contacts, each with the body's
//! altitude, and the stretch of the eclipse the body stands above a
//! horizon convention, which is what a sutak asks.
//!
//! ```
//! use teistro_astro::eclipse::{Eclipses, LunarKind};
//! use teistro_astro::{Completion, DeltaTModel};
//! use teistro_core::quantity::{JulianDay, Ut1};
//! use teistro_core::settings::OverridePolicy;
//! use teistro_port_ephemeris::TestProvider;
//!
//! let provider = TestProvider::new();
//! let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
//! let eclipses = Eclipses::new(&sky, DeltaTModel::TableThenModel);
//! // A year holds at least two lunar eclipses, penumbral ones counted.
//! let from = JulianDay::<Ut1>::literal(2_451_545.0);
//! let to = JulianDay::<Ut1>::literal(2_451_545.0 + 365.25);
//! let lunar = eclipses.lunar_between(from, to).expect("the test provider answers");
//! assert!(lunar.len() >= 2);
//! for eclipse in &lunar {
//!     assert!(eclipse.penumbral_magnitude > 0.0);
//!     assert_eq!(eclipse.kind == LunarKind::Total, eclipse.umbral_magnitude >= 1.0);
//! }
//! ```

use serde::Serialize;
use teistro_core::angle::normalise_deg;
use teistro_core::error::{Error, Status};
use teistro_core::math;
use teistro_core::quantity::{JulianDay, Latitude, Longitude, Ut1};
use teistro_core::settings::EclipseShadow;
use teistro_port_ephemeris::{
    Astronomy, Body, Coordinates, Corrections, EphemerisProvider, Frame, PositionRequest, TimeScale,
};

use crate::completion::{Completion, apparent_cell};
use crate::delta_t::DeltaTModel;
use crate::iau::vector::{Vector3, pdp, pm, pn, pxp, s2c, sxp};
use crate::iau::{DEG2RAD, RAD2DEG};
use crate::rise_set::{AU_KM, EARTH_EQUATORIAL_RADIUS_KM};
use crate::scale::tt_of;
use crate::sky::{Apparent, ApparentPositions, greenwich_sidereal_time_deg};
use crate::solve::{Caps, SolveError, minimum, refine};

mod local;

pub use local::{EclipsesHere, LocalMoment, LunarHere, LunarView, SolarHere, SolarView, Visible};

/// The Sun's radius, km (IAU 2015 nominal).
pub const SUN_RADIUS_KM: f64 = 696_000.0;

/// The Moon's radius in Earth radii for a lunar eclipse and the penumbra
/// of a solar one (IAU 1982).
pub const MOON_RADIUS: f64 = 0.272_507_6;

/// The Moon's radius in Earth radii for the umbra of a solar eclipse:
/// the mean radius under the limb's valleys, which NASA's canon takes so
/// that a total eclipse is total wherever the Sun is wholly hidden. With
/// [`MOON_RADIUS`] instead, four total eclipses and one annular eclipse
/// of 1900 to 2100 read partial.
pub const MOON_RADIUS_UMBRA: f64 = 0.272_281;

/// The Earth's polar radius over its equatorial one, `1 − 1/298.257`.
pub const EARTH_AXIS_RATIO: f64 = 0.996_647_189_335;

/// The mean synodic month, days (Meeus, *Astronomical Algorithms* 49.1).
const SYNODIC_MONTH_DAYS: f64 = 29.530_588_861;

/// The mean new moon of 2000 January 6 (Meeus 49.1), as a JD.
const MEAN_NEW_MOON_JD: f64 = 2_451_550.097_66;

/// How far either side of a mean syzygy the true one is searched, days:
/// the true syzygy is within 0.6 days of the mean, and the Moon's
/// distance from the shadow's axis falls monotonically toward it from a
/// day out.
const SEED_REACH_DAYS: f64 = 1.0;

/// The bracket a minimum is first narrowed to before it is judged,
/// days.
const COARSE_DAYS: f64 = 0.01;

/// The tolerance a greatest eclipse and a contact are found to, days.
pub const TOLERANCE_DAYS: f64 = 1e-7;

/// The fastest the Moon closes on the Earth's shadow axis, radians a day
/// (under 15° a day, the Moon's fastest less the Sun's slowest): bounds
/// how far a coarse minimum can sit above the true one.
const LUNAR_CLOSING_RATE: f64 = 16.0 * DEG2RAD;

/// The same for the Moon's shadow axis across the fundamental plane,
/// Earth radii a day: 15° a day at 60 Earth radii is 15.7.
const SOLAR_CLOSING_RATE: f64 = 16.0;

/// How long a lunar eclipse's penumbral phase lasts at most, either side
/// of greatest, days: about 3 hours, so a contact is bracketed by a
/// quarter day.
const CONTACT_REACH_DAYS: f64 = 0.25;

/// How long a search looks ahead for the next eclipse, days: a year holds
/// at least two of each kind, penumbral lunar ones counted.
pub const NEXT_REACH_DAYS: f64 = 366.0;

/// How the Earth's shadow is enlarged by its atmosphere.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub enum ShadowRule {
    /// Danjon (1951): the Earth's radius grows by 1/85, so 1% is added to
    /// the Moon's parallax. NASA's canon and the French almanac.
    #[default]
    Danjon,
    /// Chauvenet (1891): both shadows' radii grow by 1/50. The
    /// Astronomical Almanac until 2015; magnitudes 0.008 larger in the
    /// umbra and 0.028 in the penumbra.
    Chauvenet,
}

impl ShadowRule {
    /// The umbra's and the penumbra's angular radii, radians, at the
    /// Moon's distance, from the Moon's and the Sun's horizontal
    /// parallaxes and the Sun's semidiameter.
    fn radii(self, moon_parallax: f64, sun_parallax: f64, sun_semidiameter: f64) -> (f64, f64) {
        let umbra = moon_parallax + sun_parallax - sun_semidiameter;
        let penumbra = moon_parallax + sun_parallax + sun_semidiameter;
        match self {
            ShadowRule::Danjon => {
                let grown = 0.01 * moon_parallax;
                (umbra + grown, penumbra + grown)
            }
            ShadowRule::Chauvenet => (1.02 * umbra, 1.02 * penumbra),
        }
    }

    /// The rule `panchanga.eclipse_shadow` names.
    ///
    /// # Errors
    ///
    /// A member this release of the SDK does not compute (`Unsupported`,
    /// on the knob).
    pub fn of_setting(knob: EclipseShadow) -> Result<ShadowRule, Error> {
        match knob {
            EclipseShadow::Danjon => Ok(ShadowRule::Danjon),
            EclipseShadow::Chauvenet => Ok(ShadowRule::Chauvenet),
            other => Err(Error::new(
                Status::Unsupported,
                format!("the eclipse search does not know the shadow rule {other} yet"),
            )
            .with_field("panchanga.eclipse_shadow")),
        }
    }

    /// The rule's key.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            ShadowRule::Danjon => "DANJON",
            ShadowRule::Chauvenet => "CHAUVENET",
        }
    }
}

/// How much of the Moon the Earth's shadow takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum LunarKind {
    /// The penumbra only.
    Penumbral,
    /// Part of the Moon in the umbra.
    Partial,
    /// The whole Moon in the umbra.
    Total,
}

/// How much of the Sun the Moon hides, and how.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum SolarKind {
    /// The umbra and the antumbra miss the Earth.
    Partial,
    /// The Moon's disc inside the Sun's: the antumbra reaches the Earth.
    Annular,
    /// The Sun wholly hidden: the umbra reaches the Earth.
    Total,
    /// Annular at the path's ends and total at its middle, read at the
    /// greatest eclipse: the umbra reaches the Earth's surface there and
    /// not the fundamental plane, or the other way about.
    Hybrid,
}

/// A lunar eclipse's contacts, UT1: the Moon's limb meeting each
/// shadow's edge. A contact the eclipse does not have is `None`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct LunarContacts {
    /// The Moon enters the penumbra.
    pub p1: JulianDay<Ut1>,
    /// The Moon enters the umbra; a partial or total eclipse's.
    pub u1: Option<JulianDay<Ut1>>,
    /// The Moon is wholly in the umbra; a total eclipse's.
    pub u2: Option<JulianDay<Ut1>>,
    /// The Moon begins to leave the umbra; a total eclipse's.
    pub u3: Option<JulianDay<Ut1>>,
    /// The Moon leaves the umbra; a partial or total eclipse's.
    pub u4: Option<JulianDay<Ut1>>,
    /// The Moon leaves the penumbra.
    pub p4: JulianDay<Ut1>,
}

/// A lunar eclipse.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct LunarEclipse {
    /// The instant the Moon's centre is nearest the shadow's axis, UT1.
    pub greatest: JulianDay<Ut1>,
    /// The kind.
    pub kind: LunarKind,
    /// The Moon's centre from the shadow's axis at the greatest eclipse,
    /// Earth radii, positive when the Moon passes north of it.
    pub gamma: f64,
    /// The fraction of the Moon's diameter in the umbra; at least 1 in a
    /// total eclipse, negative when the Moon misses the umbra.
    pub umbral_magnitude: f64,
    /// The fraction of the Moon's diameter in the penumbra.
    pub penumbral_magnitude: f64,
    /// The contacts.
    pub contacts: LunarContacts,
    /// The rule the shadow was enlarged by.
    pub shadow: ShadowRule,
}

impl LunarEclipse {
    /// How long the Moon is in the umbra wholly (the totality), partly
    /// or at all (the penumbral phase), days; `None` for a phase the
    /// eclipse does not have.
    #[must_use]
    pub fn durations(&self) -> [Option<f64>; 3] {
        let span = |from: Option<JulianDay<Ut1>>, to: Option<JulianDay<Ut1>>| {
            Some(to?.get() - from?.get())
        };
        let c = &self.contacts;
        [
            span(c.u2, c.u3),
            span(c.u1, c.u4),
            span(Some(c.p1), Some(c.p4)),
        ]
    }
}

/// A point on the Earth: a geodetic latitude and a longitude, east
/// positive.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct GroundPoint {
    /// Geodetic latitude.
    pub latitude: Latitude,
    /// Longitude, east positive.
    pub longitude: Longitude,
}

/// A solar eclipse.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SolarEclipse {
    /// The instant the shadow's axis passes nearest the Earth's centre,
    /// UT1.
    pub greatest: JulianDay<Ut1>,
    /// The kind.
    pub kind: SolarKind,
    /// The axis from the Earth's centre at the greatest eclipse, Earth
    /// radii, positive when it passes north; under 1 the eclipse is
    /// central (for a sphere).
    pub gamma: f64,
    /// At the point of greatest eclipse: the Moon's apparent diameter
    /// over the Sun's in a central eclipse, the fraction of the Sun's
    /// diameter hidden in a partial one.
    pub magnitude: f64,
    /// Where the eclipse is greatest: under the axis, or for an axis that
    /// misses the Earth, the point of the limb nearest it.
    pub point: GroundPoint,
}

/// How a body is placed for a shadow.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Seen {
    /// Corrected for light time and for nothing else: where the light now
    /// reaching the Earth's centre left the body.
    Astrometric,
    /// Every correction: where the body is seen.
    Apparent,
}

impl Seen {
    /// The geocentric frame of date this placing asks for.
    #[must_use]
    pub const fn frame(self) -> Frame {
        let corrections = match self {
            Seen::Astrometric => Corrections {
                light_time: true,
                aberration: false,
                deflection: false,
                nutation: true,
            },
            Seen::Apparent => Corrections::APPARENT,
        };
        Frame {
            coordinates: Coordinates::Equatorial,
            corrections,
            ..Frame::CANONICAL
        }
    }
}

/// A source of the Sun's and the Moon's places of date, geocentric on the
/// true equator, placed as a shadow asks.
///
/// **Which placing each eclipse takes is the light's path.** A solar
/// eclipse's shadow reaching the Earth now is sunlight that passed the
/// Moon 1.3 seconds ago, so both bodies are astrometric. A lunar
/// eclipse's shadow was cast past the Earth where the Earth stood 1.3
/// seconds before the Moon is seen, and from the Earth now that offset is
/// the Moon's aberration, the same angle as the Sun's: so both bodies are
/// apparent. Measured against NASA's canon, a lunar eclipse read with the
/// Sun astrometric comes 38 seconds late and with both astrometric 77;
/// these placings agree with it to a second or two
/// (`docs/03-design/eclipses.md` §4.1).
pub trait ShadowSource: Send + Sync {
    /// The body's place at a UT1 instant.
    ///
    /// # Errors
    ///
    /// An instant or a body the source cannot answer, or a source that
    /// cannot place a body as asked.
    fn place(&self, body: Body, ut1: JulianDay<Ut1>, seen: Seen) -> Result<Apparent, Error>;

    /// The source's name for provenance stamps.
    fn describe(&self) -> String;
}

impl<P: EphemerisProvider + ?Sized> ShadowSource for Completion<'_, P> {
    fn place(&self, body: Body, ut1: JulianDay<Ut1>, seen: Seen) -> Result<Apparent, Error> {
        if self.capabilities().astronomy == Astronomy::Classical {
            return Err(Error::new(
                Status::Unsupported,
                "a classical text's eclipse is its own method (Surya Siddhanta IV to VI), \
                 not a modern shadow over its places (C188)",
            )
            .with_hint("find eclipses over a modern ephemeris"));
        }
        let jds = [ut1.get()];
        let bodies = [body];
        let request =
            PositionRequest::new(&jds, TimeScale::Ut1, &bodies, seen.frame()).without_speeds();
        apparent_cell(&self.positions(&request)?, 0, body, ut1)
    }

    fn describe(&self) -> String {
        ApparentPositions::describe(self)
    }
}

/// The Sun and the Moon as the shadow sees them at one instant:
/// equatorial vectors of date, Earth radii.
#[derive(Clone, Copy, Debug)]
struct Pair {
    sun: Vector3,
    moon: Vector3,
}

/// A vector's offset across an axis, in the plane through the Earth's
/// centre perpendicular to it: `x` eastward, `y` toward the celestial
/// north, and the axis's declination's cosine.
fn across(point: &Vector3, axis: &Vector3) -> (f64, f64, f64) {
    let north = [0.0, 0.0, 1.0];
    let (cos_d, ey) = pn(&sub(&north, &sxp(pdp(&north, axis), axis)));
    let ex = pxp(&ey, axis);
    (pdp(point, &ex), pdp(point, &ey), cos_d)
}

fn sub(a: &Vector3, b: &Vector3) -> Vector3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// The angle between two vectors, radians, well conditioned near 0 and π.
fn angle(a: &Vector3, b: &Vector3) -> f64 {
    math::atan2(pm(&pxp(a, b)), pdp(a, b))
}

/// The lunar quantities at one instant.
#[derive(Clone, Copy, Debug)]
struct LunarAt {
    /// The Moon's centre from the antisolar point, radians.
    distance: f64,
    /// The umbra's and the penumbra's radii, radians.
    umbra: f64,
    penumbra: f64,
    /// The Moon's semidiameter, radians.
    moon_semidiameter: f64,
    /// The signed distance of the Moon's centre from the axis, Earth radii.
    gamma: f64,
}

impl LunarAt {
    fn of(pair: &Pair, rule: ShadowRule) -> LunarAt {
        let (moon_r, moon_u) = pn(&pair.moon);
        let (sun_r, sun_u) = pn(&pair.sun);
        let axis = sxp(-1.0, &sun_u);
        let distance = angle(&moon_u, &axis);
        let (umbra, penumbra) = rule.radii(
            math::asin(1.0 / moon_r),
            math::asin(1.0 / sun_r),
            math::asin(SUN_RADIUS_KM / EARTH_EQUATORIAL_RADIUS_KM / sun_r),
        );
        let (_, y, _) = across(&pair.moon, &axis);
        LunarAt {
            distance,
            umbra,
            penumbra,
            moon_semidiameter: math::asin(MOON_RADIUS / moon_r),
            gamma: (moon_r * math::sin(distance)).copysign(y),
        }
    }

    fn magnitude(&self, radius: f64) -> f64 {
        (radius + self.moon_semidiameter - self.distance) / (2.0 * self.moon_semidiameter)
    }
}

/// The solar quantities at one instant: the Moon's shadow axis and where
/// it crosses the fundamental plane.
#[derive(Clone, Copy, Debug)]
struct SolarAt {
    /// The axis, from the Sun through the Moon, unit.
    axis: Vector3,
    /// Where the axis crosses the fundamental plane.
    x: f64,
    y: f64,
    /// The axis's declination's cosine.
    cos_d: f64,
    /// The Moon's distance from the plane, toward the Sun, Earth radii.
    z: f64,
    /// The Sun's distance from the Moon, Earth radii.
    sun_from_moon: f64,
}

impl SolarAt {
    fn of(pair: &Pair) -> SolarAt {
        let (sun_from_moon, toward) = pn(&sub(&pair.moon, &pair.sun));
        let (x, y, cos_d) = across(&pair.moon, &toward);
        SolarAt {
            axis: toward,
            x,
            y,
            cos_d,
            z: -pdp(&pair.moon, &toward),
            sun_from_moon,
        }
    }

    /// The axis from the Earth's centre, Earth radii.
    fn offset(&self) -> f64 {
        math::hypot(self.x, self.y)
    }

    /// The umbra's radius at a distance below the Moon toward the Earth,
    /// Earth radii: positive inside the umbra's cone, negative past its
    /// vertex in the antumbra.
    fn umbra_at(&self, below_moon: f64) -> f64 {
        let vertex = self.sun_from_moon * MOON_RADIUS_UMBRA / (sun_radius() - MOON_RADIUS_UMBRA);
        MOON_RADIUS_UMBRA * (1.0 - below_moon / vertex)
    }

    /// The penumbra's radius in the fundamental plane, Earth radii.
    fn penumbra(&self) -> f64 {
        let vertex = self.sun_from_moon * MOON_RADIUS / (sun_radius() + MOON_RADIUS);
        MOON_RADIUS * (1.0 + self.z / vertex)
    }
}

/// The Sun's radius in Earth radii.
fn sun_radius() -> f64 {
    SUN_RADIUS_KM / EARTH_EQUATORIAL_RADIUS_KM
}

/// A vector with the Earth made a unit sphere: the polar axis stretched.
fn rounded(v: &Vector3) -> Vector3 {
    [v[0], v[1], v[2] / EARTH_AXIS_RATIO]
}

/// The inverse of [`rounded`].
fn flattened(v: &Vector3) -> Vector3 {
    [v[0], v[1], v[2] * EARTH_AXIS_RATIO]
}

/// The distance from a point of the fundamental plane to the Earth's
/// outline there, an ellipse with semi-axes 1 and `rho`: Newton's method
/// on the ellipse's parameter from the point's own direction.
fn outline_distance(x: f64, y: f64, rho: f64) -> f64 {
    let mut theta = math::atan2(y / rho, x);
    for _ in 0..16 {
        let (s, c) = math::sin_cos(theta);
        // Half the squared distance's derivative and its own.
        let slope = (x - c) * s - (y - rho * s) * rho * c;
        let curve = (x - c) * c + s * s + (y - rho * s) * rho * s + rho * rho * c * c;
        let step = slope / curve;
        theta -= step;
        if step.abs() < 1e-15 {
            break;
        }
    }
    let (s, c) = math::sin_cos(theta);
    math::hypot(x - c, y - rho * s)
}

/// The geodetic place of a point on the Earth's surface given as an
/// equatorial vector of date, Earth radii, at an instant's sidereal time.
fn ground(surface: &Vector3, sidereal_deg: f64) -> Result<GroundPoint, Error> {
    let equatorial = math::hypot(surface[0], surface[1]);
    let latitude = math::atan2(surface[2], EARTH_AXIS_RATIO * EARTH_AXIS_RATIO * equatorial);
    let ra = math::atan2(surface[1], surface[0]) * RAD2DEG;
    let east = normalise_deg(ra - sidereal_deg + 180.0) - 180.0;
    Ok(GroundPoint {
        latitude: Latitude::try_new(latitude * RAD2DEG)?,
        longitude: Longitude::try_new(east)?,
    })
}

/// Every eclipse in a window over a source of apparent places.
///
/// `Eclipses` borrows the source and holds the two choices an answer
/// depends on, the Delta T model the sidereal time is read under and the
/// rule for the Earth's shadow. A lunar eclipse reports the rule it was
/// found under.
pub struct Eclipses<'s, S: ShadowSource + ?Sized> {
    sky: &'s S,
    delta_t: DeltaTModel,
    shadow: ShadowRule,
}

impl<S: ShadowSource + ?Sized> core::fmt::Debug for Eclipses<'_, S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Eclipses")
            .field("sky", &self.sky.describe())
            .field("delta_t", &self.delta_t)
            .field("shadow", &self.shadow)
            .finish()
    }
}

/// Which syzygy a search walks: the new moons for the Sun, the full moons
/// for the Moon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Syzygy {
    New,
    Full,
}

impl<'s, S: ShadowSource + ?Sized> Eclipses<'s, S> {
    /// The eclipses over a source, under Danjon's shadow.
    #[must_use]
    pub const fn new(sky: &'s S, delta_t: DeltaTModel) -> Eclipses<'s, S> {
        Eclipses {
            sky,
            delta_t,
            shadow: ShadowRule::Danjon,
        }
    }

    /// The same, with the Earth's shadow enlarged by another rule.
    #[must_use]
    pub const fn with_shadow(self, shadow: ShadowRule) -> Eclipses<'s, S> {
        Eclipses { shadow, ..self }
    }

    /// The rule the Earth's shadow is enlarged by.
    #[must_use]
    pub const fn shadow(&self) -> ShadowRule {
        self.shadow
    }

    /// Every lunar eclipse whose greatest moment falls in `[from, to)`,
    /// in order.
    ///
    /// # Errors
    ///
    /// A window that does not run forward (`InvalidArg` on `to`), or an
    /// instant the source cannot answer.
    pub fn lunar_between(
        &self,
        from: JulianDay<Ut1>,
        to: JulianDay<Ut1>,
    ) -> Result<Vec<LunarEclipse>, Error> {
        between(from, to, Syzygy::Full, |seed| self.lunar_near(seed))
    }

    /// Every solar eclipse whose greatest moment falls in `[from, to)`, in
    /// order.
    ///
    /// # Errors
    ///
    /// As [`Eclipses::lunar_between`].
    pub fn solar_between(
        &self,
        from: JulianDay<Ut1>,
        to: JulianDay<Ut1>,
    ) -> Result<Vec<SolarEclipse>, Error> {
        between(from, to, Syzygy::New, |seed| self.solar_near(seed))
    }

    /// The first lunar eclipse at or after `from`.
    ///
    /// # Errors
    ///
    /// `NotConverged` when none falls within [`NEXT_REACH_DAYS`], which
    /// only a source whose range ends can cause, or an instant the source
    /// cannot answer.
    pub fn next_lunar(&self, from: JulianDay<Ut1>) -> Result<LunarEclipse, Error> {
        first(from, Syzygy::Full, |seed| self.lunar_near(seed))?
            .ok_or_else(|| not_found("lunar", from))
    }

    /// The first solar eclipse at or after `from`.
    ///
    /// # Errors
    ///
    /// As [`Eclipses::next_lunar`].
    pub fn next_solar(&self, from: JulianDay<Ut1>) -> Result<SolarEclipse, Error> {
        first(from, Syzygy::New, |seed| self.solar_near(seed))?
            .ok_or_else(|| not_found("solar", from))
    }

    /// The Sun and the Moon at an instant, Earth radii, each placed as
    /// the syzygy's shadow asks ([`ShadowSource`]).
    fn pair(&self, at: f64, syzygy: Syzygy) -> Result<Pair, Error> {
        let ut1 = JulianDay::try_new(at)?;
        let seen = match syzygy {
            Syzygy::New => Seen::Astrometric,
            Syzygy::Full => Seen::Apparent,
        };
        let radii = |body, seen| -> Result<Vector3, Error> {
            let place = self.sky.place(body, ut1, seen)?;
            Ok(sxp(
                place.distance_au * AU_KM / EARTH_EQUATORIAL_RADIUS_KM,
                &s2c(place.ra_deg * DEG2RAD, place.dec_deg * DEG2RAD),
            ))
        };
        Ok(Pair {
            sun: radii(Body::Sun, seen)?,
            moon: radii(Body::Moon, seen)?,
        })
    }

    /// The least of a quantity of the pair near a seed: narrowed coarsely,
    /// judged by `close` at the least the quantity could reach inside the
    /// coarse bracket (closing at `rate` a day), then refined; `None` when
    /// `close` refuses.
    fn least(
        &self,
        seed: f64,
        syzygy: Syzygy,
        (quantity, rate): (impl Fn(&Pair) -> f64, f64),
        close: impl FnOnce(f64, &Pair) -> bool,
    ) -> Result<Option<(f64, Pair)>, Error> {
        let f = |at: f64| self.pair(at, syzygy).map(|pair| quantity(&pair));
        let sought = || format!("the syzygy near JD {seed}");
        let coarse = minimum(
            f,
            seed - SEED_REACH_DAYS,
            seed + SEED_REACH_DAYS,
            COARSE_DAYS,
            Caps::DEFAULT,
        )
        .map_err(|error| error.into_error(sought))?;
        if !close(
            coarse.value - rate * coarse.width,
            &self.pair(coarse.instant, syzygy)?,
        ) {
            return Ok(None);
        }
        let fine = minimum(
            f,
            coarse.instant - coarse.width,
            coarse.instant + coarse.width,
            TOLERANCE_DAYS,
            Caps::DEFAULT,
        )
        .map_err(|error| error.into_error(sought))?;
        Ok(Some((fine.instant, self.pair(fine.instant, syzygy)?)))
    }

    fn lunar_near(&self, seed: f64) -> Result<Option<LunarEclipse>, Error> {
        let rule = self.shadow;
        let Some((at, pair)) = self.least(
            seed,
            Syzygy::Full,
            (|pair| LunarAt::of(pair, rule).distance, LUNAR_CLOSING_RATE),
            |least, pair| {
                let lunar = LunarAt::of(pair, rule);
                least < lunar.penumbra + lunar.moon_semidiameter
            },
        )?
        else {
            return Ok(None);
        };
        let lunar = LunarAt::of(&pair, rule);
        let penumbral_magnitude = lunar.magnitude(lunar.penumbra);
        if penumbral_magnitude <= 0.0 {
            return Ok(None);
        }
        let umbral_magnitude = lunar.magnitude(lunar.umbra);
        let kind = if umbral_magnitude >= 1.0 {
            LunarKind::Total
        } else if umbral_magnitude > 0.0 {
            LunarKind::Partial
        } else {
            LunarKind::Penumbral
        };
        // A contact: where the Moon's distance from the axis meets a
        // shadow's edge less or more its semidiameter, on one side.
        let contact = |edge: fn(&LunarAt) -> f64, after: bool| -> Result<JulianDay<Ut1>, Error> {
            let gap = |t: f64| {
                let here = LunarAt::of(&self.pair(t, Syzygy::Full)?, rule);
                let gap = here.distance - edge(&here);
                Ok::<_, Error>(if after { gap } else { -gap })
            };
            let (lo, hi) = if after {
                (at, at + CONTACT_REACH_DAYS)
            } else {
                (at - CONTACT_REACH_DAYS, at)
            };
            let found = refine(gap, lo, hi, TOLERANCE_DAYS, Caps::DEFAULT).map_err(
                |error: SolveError<Error>| {
                    error.into_error(|| format!("the lunar eclipse's contact near JD {at}"))
                },
            )?;
            Ok(JulianDay::try_new(found.instant)?)
        };
        let penumbra: fn(&LunarAt) -> f64 = |a| a.penumbra + a.moon_semidiameter;
        let umbra: fn(&LunarAt) -> f64 = |a| a.umbra + a.moon_semidiameter;
        let inner: fn(&LunarAt) -> f64 = |a| a.umbra - a.moon_semidiameter;
        let partial = kind != LunarKind::Penumbral;
        let total = kind == LunarKind::Total;
        let contacts = LunarContacts {
            p1: contact(penumbra, false)?,
            u1: partial.then(|| contact(umbra, false)).transpose()?,
            u2: total.then(|| contact(inner, false)).transpose()?,
            u3: total.then(|| contact(inner, true)).transpose()?,
            u4: partial.then(|| contact(umbra, true)).transpose()?,
            p4: contact(penumbra, true)?,
        };
        Ok(Some(LunarEclipse {
            greatest: JulianDay::try_new(at)?,
            kind,
            gamma: lunar.gamma,
            umbral_magnitude,
            penumbral_magnitude,
            contacts,
            shadow: rule,
        }))
    }

    fn solar_near(&self, seed: f64) -> Result<Option<SolarEclipse>, Error> {
        let Some((at, pair)) = self.least(
            seed,
            Syzygy::New,
            (|pair| SolarAt::of(pair).offset(), SOLAR_CLOSING_RATE),
            |least, pair| least < 1.0 + SolarAt::of(pair).penumbra(),
        )?
        else {
            return Ok(None);
        };
        let solar = SolarAt::of(&pair);
        let ut1 = JulianDay::<Ut1>::try_new(at)?;
        let (tt, _) = tt_of(ut1, self.delta_t)?;
        let sidereal = greenwich_sidereal_time_deg(ut1, tt);
        let umbra = solar.umbra_at(solar.z);
        let rho =
            (1.0 - (1.0 - EARTH_AXIS_RATIO * EARTH_AXIS_RATIO) * solar.cos_d * solar.cos_d).sqrt();
        let inside = solar.x * solar.x + (solar.y / rho) * (solar.y / rho) < 1.0;
        // The axis and the Moon with the Earth made a unit sphere, where
        // the surface point under the axis and the limb nearest it are
        // plain geometry; tangency and intersection survive the stretch.
        let (moon, sun) = (rounded(&pair.moon), rounded(&pair.sun));
        let (_, axis) = pn(&sub(&moon, &sun));
        let offset = sub(&moon, &sxp(pdp(&moon, &axis), &axis));
        let (kind, magnitude, surface) = if inside {
            // Never negative but by rounding: the two inside tests are
            // the same test, before and after the stretch.
            let reach = (1.0 - pdp(&offset, &offset)).max(0.0);
            let below = -pdp(&moon, &axis) - reach.sqrt();
            let surface = flattened(&sub(&moon, &sxp(-below, &axis)));
            let umbra_there = solar.umbra_at(pdp(&sub(&surface, &pair.moon), &solar.axis));
            let moon_disc = math::asin(MOON_RADIUS_UMBRA / pm(&sub(&pair.moon, &surface)));
            let sun_disc = math::asin(sun_radius() / pm(&sub(&pair.sun, &surface)));
            let kind = match (umbra_there > 0.0, umbra > 0.0) {
                (true, true) => SolarKind::Total,
                (false, false) => SolarKind::Annular,
                _ => SolarKind::Hybrid,
            };
            (kind, moon_disc / sun_disc, surface)
        } else {
            let delta = outline_distance(solar.x, solar.y, rho);
            let penumbra = solar.penumbra();
            let magnitude = (penumbra - delta) / (penumbra - umbra);
            if magnitude <= 0.0 {
                return Ok(None);
            }
            let kind = if delta < umbra.abs() {
                if umbra > 0.0 {
                    SolarKind::Total
                } else {
                    SolarKind::Annular
                }
            } else {
                SolarKind::Partial
            };
            let (_, limb) = pn(&offset);
            (kind, magnitude, flattened(&limb))
        };
        Ok(Some(SolarEclipse {
            greatest: ut1,
            kind,
            gamma: solar.offset().copysign(solar.y),
            magnitude,
            point: ground(&surface, sidereal)?,
        }))
    }
}

/// The eclipses of one syzygy kind with the greatest moment in the
/// window.
fn between<E: Greatest>(
    from: JulianDay<Ut1>,
    to: JulianDay<Ut1>,
    syzygy: Syzygy,
    mut near: impl FnMut(f64) -> Result<Option<E>, Error>,
) -> Result<Vec<E>, Error> {
    if to.get() <= from.get() {
        return Err(Error::invalid_arg(format!(
            "the eclipse window must run forward, not from {} to {}",
            from.get(),
            to.get()
        ))
        .with_field("to"));
    }
    let mut found = Vec::new();
    for seed in seeds(from.get(), to.get(), syzygy) {
        if let Some(eclipse) = near(seed)? {
            if (from.get()..to.get()).contains(&eclipse.greatest()) {
                found.push(eclipse);
            }
        }
    }
    Ok(found)
}

/// The first eclipse of one syzygy kind at or after `from`, a year on at
/// most.
fn first<E: Greatest>(
    from: JulianDay<Ut1>,
    syzygy: Syzygy,
    mut near: impl FnMut(f64) -> Result<Option<E>, Error>,
) -> Result<Option<E>, Error> {
    let from = from.get();
    for seed in seeds(from, from + NEXT_REACH_DAYS, syzygy) {
        if let Some(eclipse) = near(seed)? {
            if eclipse.greatest() >= from {
                return Ok(Some(eclipse));
            }
        }
    }
    Ok(None)
}

/// An eclipse's greatest moment, for the window's filter.
trait Greatest {
    fn greatest(&self) -> f64;
}

impl Greatest for LunarEclipse {
    fn greatest(&self) -> f64 {
        self.greatest.get()
    }
}

impl Greatest for SolarEclipse {
    fn greatest(&self) -> f64 {
        self.greatest.get()
    }
}

/// The mean syzygies whose search reaches into `[from, to)`.
fn seeds(from: f64, to: f64, syzygy: Syzygy) -> impl Iterator<Item = f64> {
    let offset = match syzygy {
        Syzygy::New => 0.0,
        Syzygy::Full => 0.5,
    };
    let at = move |k: f64| MEAN_NEW_MOON_JD + (k + offset) * SYNODIC_MONTH_DAYS;
    let first = ((from - SEED_REACH_DAYS - at(0.0)) / SYNODIC_MONTH_DAYS).floor();
    (0..)
        .map(move |n| at(first + f64::from(n)))
        .skip_while(move |seed| seed + SEED_REACH_DAYS < from)
        .take_while(move |seed| seed - SEED_REACH_DAYS < to)
}

fn not_found(kind: &str, from: JulianDay<Ut1>) -> Error {
    Error::new(
        Status::NotConverged,
        format!(
            "no {kind} eclipse within {NEXT_REACH_DAYS} days of JD {}; the source's range may end there",
            from.get()
        ),
    )
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use super::*;

    #[test]
    fn the_shadow_rules_enlarge_as_their_sources_say() {
        // The Moon's parallax 57′, the Sun's 8.8″ and semidiameter 16′.
        let (pm, ps, ss) = (57.0 / 60.0, 8.8 / 3600.0, 16.0 / 60.0);
        let (u, p) = ShadowRule::Danjon.radii(pm, ps, ss);
        assert!((u - (1.01 * pm + ps - ss)).abs() < 1e-15);
        assert!((p - (1.01 * pm + ps + ss)).abs() < 1e-15);
        let (u, p) = ShadowRule::Chauvenet.radii(pm, ps, ss);
        assert!((u - 1.02 * (pm + ps - ss)).abs() < 1e-15);
        assert!((p - 1.02 * (pm + ps + ss)).abs() < 1e-15);
    }

    #[test]
    fn the_outline_distance_is_the_ellipses() {
        // On a circle it is the radial distance.
        assert!((outline_distance(1.5, 0.0, 1.0) - 0.5).abs() < 1e-15);
        assert!((outline_distance(0.6, 0.8, 1.0) - 0.0).abs() < 1e-15);
        // Over the pole of a squashed outline, the minor axis.
        assert!((outline_distance(0.0, 1.5, 0.99) - 0.51).abs() < 1e-12);
        // Off-axis, no point of the ellipse is nearer than the answer.
        let (x, y, rho) = (0.9, 1.1, 0.997);
        let found = outline_distance(x, y, rho);
        let nearest = (0..100_000)
            .map(|i| {
                let t = f64::from(i) / 100_000.0 * core::f64::consts::TAU;
                math::hypot(x - math::cos(t), y - rho * math::sin(t))
            })
            .fold(f64::INFINITY, f64::min);
        assert!(found <= nearest + 1e-12 && nearest - found < 1e-8);
    }

    #[test]
    fn the_seeds_cover_the_window_and_no_more() {
        let from = 2_460_000.0;
        let to = from + 365.25;
        let new: Vec<f64> = seeds(from, to, Syzygy::New).collect();
        assert!(new.len() == 12 || new.len() == 13, "{}", new.len());
        assert!(new[0] + SEED_REACH_DAYS >= from);
        assert!(new[0] - SYNODIC_MONTH_DAYS + SEED_REACH_DAYS < from);
        let full: Vec<f64> = seeds(from, to, Syzygy::Full).collect();
        for pair in full.windows(2) {
            assert!((pair[1] - pair[0] - SYNODIC_MONTH_DAYS).abs() < 1e-9);
        }
    }
}
