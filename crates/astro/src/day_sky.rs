//! The day's sky (`docs/03-design/day-sky.md`): the paths of the Sun and
//! the Moon over a window at a place, their rises, transits and sets as
//! exact points on those paths, and the Moon's lit shape as it stands in
//! that sky.
//!
//! The ephemeris is read at Chebyshev nodes only. Right ascension,
//! declination and distance are smooth over days, so a fit answers every
//! point and every search of the window for the price of its nodes, and
//! the step asked does not change what the ephemeris is asked. A search
//! that leaves the fit reads the source; nothing is extrapolated.
//!
//! ```
//! use teistro_astro::day_sky::{SkyPointKind, SkyState, SkyStep, day_sky};
//! use teistro_astro::{Completion, DeltaTModel};
//! use teistro_core::quantity::{JulianDay, Place, Ut1};
//! use teistro_core::settings::OverridePolicy;
//! use teistro_port_ephemeris::{Horizon, TestProvider};
//!
//! let provider = TestProvider::new();
//! let delta_t = DeltaTModel::TableThenModel;
//! let source = Completion::new(&provider, OverridePolicy::SdkOnly, delta_t);
//! let place = Place::try_from_degrees(27.7172, 85.324, 1_400.0)?;
//! // The local mean day of 2024-06-21 there.
//! let from = JulianDay::<Ut1>::literal(2_460_482.5 - 85.324 / 360.0);
//! let to = JulianDay::<Ut1>::literal(from.get() + 1.0);
//! let sky = day_sky(&source, place, Horizon::UPPER_LIMB_REFRACTION, delta_t, (from, to), SkyStep::default())?;
//! assert_eq!(sky.sun.state, SkyState::Crosses);
//! // A noon, with its altitude, marked on the path.
//! let noon = sky.sun.points(SkyPointKind::Transit).next().expect("a noon");
//! assert!(noon.altitude_deg > 80.0);
//! // The Moon's points carry her shape; the Sun's have none.
//! assert!(sky.moon.track.iter().all(|point| point.moon.is_some()));
//! assert!(sky.sun.track.iter().all(|point| point.moon.is_none()));
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

use serde::Serialize;
use teistro_core::angle::{difference_deg, normalise_deg};
use teistro_core::chebyshev::{evaluate, lobatto};
use teistro_core::error::Error;
use teistro_core::math;
use teistro_core::quantity::{JulianDay, Place, Ut1};
use teistro_port_ephemeris::{Body, Horizon, HorizonEventKind};

use crate::delta_t::DeltaTModel;
use crate::iau;
use crate::iau::vector::{Vector3, c2s, pdp, s2c, sxp};
use crate::iau::{DEG2RAD, RAD2DEG};
use crate::phenomena::{bright_limb_deg, parallactic_angle_deg, phase_of};
use crate::rise_set::Solver;
use crate::scale::tt_of;
use crate::sky::{Apparent, ApparentPositions, obliquity_parts, observer_at};

/// The nodes each body's fit is read at. Measured against the source by
/// this module's test and `crates/sdk/tests/day_sky.rs`.
const FIT_NODES: usize = 11;

/// How far the fit reaches before the window, days: a search can settle a
/// tolerance before its start.
const FIT_LEAD_DAYS: f64 = 1.0 / 48.0;

/// How far the fit reaches past the window, days: the solver's iteration
/// starts from the next hour angle it wants, up to a rotation on, and
/// only then finds the event lies outside.
const FIT_TRAIL_DAYS: f64 = 1.1;

/// The least gap between two events of a kind, days, which a search for
/// the next steps past: about a quarter of an hour.
const NEXT_EVENT_GAP_DAYS: f64 = 0.01;

/// How close to a window's edge an event counts as on it, days, a tenth
/// of a second: a day's sunrise opens its window to the bit, and a search
/// over the fit can settle a hair either side, which must not lose it or
/// hand it to the day before.
const EDGE_DAYS: f64 = 1e-6;

/// The minutes in a day.
const MINUTES_A_DAY: f64 = 1_440.0;

/// The step of a track, whole minutes from 1 to 240, ten by default.
///
/// ```
/// use teistro_astro::day_sky::SkyStep;
///
/// assert_eq!(SkyStep::default().minutes(), 10);
/// assert_eq!(SkyStep::of_minutes(30)?.minutes(), 30);
/// // A step of nothing, or of more than four hours, is refused and named.
/// assert_eq!(SkyStep::of_minutes(0).unwrap_err().field(), Some("sky_step_minutes"));
/// assert!(SkyStep::of_minutes(241).is_err());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SkyStep(u16);

impl SkyStep {
    /// The longest step, minutes.
    pub const MOST_MINUTES: u16 = 240;

    /// The step of so many minutes.
    ///
    /// ```
    /// use teistro_astro::day_sky::SkyStep;
    ///
    /// assert_eq!(SkyStep::of_minutes(SkyStep::MOST_MINUTES)?.minutes(), 240);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming `sky_step_minutes`, outside 1 to 240.
    pub fn of_minutes(minutes: u16) -> Result<SkyStep, Error> {
        if (1..=Self::MOST_MINUTES).contains(&minutes) {
            Ok(SkyStep(minutes))
        } else {
            Err(Error::invalid_arg(format!(
                "a sky track's step is 1 to {} minutes, not {minutes}",
                Self::MOST_MINUTES
            ))
            .with_field("sky_step_minutes"))
        }
    }

    /// The minutes.
    ///
    /// ```
    /// use teistro_astro::day_sky::SkyStep;
    ///
    /// assert_eq!(SkyStep::of_minutes(5)?.minutes(), 5);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub const fn minutes(self) -> u16 {
        self.0
    }
}

impl Default for SkyStep {
    fn default() -> SkyStep {
        SkyStep(10)
    }
}

/// What a point on a track is.
///
/// ```
/// use teistro_astro::day_sky::SkyPointKind;
///
/// assert_eq!(serde_json::to_string(&SkyPointKind::Transit)?, r#""TRANSIT""#);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SkyPointKind {
    /// A reading at the step.
    Sample,
    /// The exact rise, under the day's horizon convention.
    Rise,
    /// The exact meridian passage, the hour angle's zero (crux C970).
    Transit,
    /// The exact set.
    Set,
}

/// How a window stood to the horizon.
///
/// ```
/// use teistro_astro::day_sky::SkyState;
///
/// assert_eq!(serde_json::to_string(&SkyState::AlwaysAbove)?, r#""ALWAYS_ABOVE""#);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SkyState {
    /// The window holds a rise or a set.
    Crosses,
    /// It holds neither, and the body stays above the horizon.
    AlwaysAbove,
    /// It holds neither, and the body stays below the horizon.
    AlwaysBelow,
}

/// The Moon's lit shape at an instant, seen from the place.
///
/// To draw the disc as it stands in the sky, turn the bright limb from the
/// zenith by `bright_limb_deg - parallactic_angle_deg`
/// (`03-design/day-sky.md` §1).
///
/// ```
/// use teistro_astro::day_sky::MoonShape;
///
/// let half = MoonShape {
///     illuminated_fraction: 0.5,
///     phase_angle_deg: 90.0,
///     waxing: true,
///     bright_limb_deg: 270.0,
///     parallactic_angle_deg: 30.0,
/// };
/// assert_eq!(half.bright_limb_from_zenith_deg(), 240.0);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct MoonShape {
    /// The fraction of the disc lit, 0 to 1, from the place (crux C971).
    pub illuminated_fraction: f64,
    /// The angle at the Moon between the place and the Sun, degrees: 0 at
    /// full, 180 at new.
    pub phase_angle_deg: f64,
    /// Whether the Moon is growing: the Earth's-centre sign of her
    /// longitude less the Sun's, which agrees with the paksha (crux C972).
    pub waxing: bool,
    /// The position angle of the bright limb's midpoint, degrees in
    /// `[0, 360)`, from the disc's north point through the east.
    pub bright_limb_deg: f64,
    /// The position angle of the zenith, degrees in `(-180, 180]`, from
    /// the disc's north point through the east.
    pub parallactic_angle_deg: f64,
}

impl MoonShape {
    /// The bright limb's angle from the top of the disc as it stands in
    /// the sky, degrees in `[0, 360)`, through the left: `χ − q`.
    ///
    /// ```
    /// # use teistro_astro::day_sky::MoonShape;
    /// # let shape = MoonShape { illuminated_fraction: 0.5, phase_angle_deg: 90.0, waxing: false, bright_limb_deg: 10.0, parallactic_angle_deg: 30.0 };
    /// assert_eq!(shape.bright_limb_from_zenith_deg(), 340.0);
    /// ```
    #[must_use]
    pub fn bright_limb_from_zenith_deg(&self) -> f64 {
        normalise_deg(self.bright_limb_deg - self.parallactic_angle_deg)
    }
}

/// A point on a track.
///
/// ```
/// use teistro_astro::day_sky::{SkyPoint, SkyPointKind};
/// use teistro_core::quantity::JulianDay;
///
/// let point = SkyPoint {
///     at: JulianDay::literal(2_460_000.5),
///     kind: SkyPointKind::Sample,
///     altitude_deg: 30.0,
///     refracted_altitude_deg: 30.03,
///     azimuth_deg: 120.0,
///     moon: None,
/// };
/// assert!(point.refracted_altitude_deg > point.altitude_deg);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SkyPoint {
    /// The instant, UT1 read as UTC.
    pub at: JulianDay<Ut1>,
    /// A step's reading or an exact event.
    pub kind: SkyPointKind,
    /// The airless topocentric altitude of the centre, degrees.
    pub altitude_deg: f64,
    /// The altitude with the standard atmosphere's refraction, degrees;
    /// the airless altitude where the body stays below the horizon
    /// (crux C973).
    pub refracted_altitude_deg: f64,
    /// The topocentric azimuth, degrees in `[0, 360)`, from north through
    /// east.
    pub azimuth_deg: f64,
    /// The Moon's shape at the instant, on the Moon's track only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moon: Option<MoonShape>,
}

/// One body's day.
///
/// ```
/// use teistro_astro::day_sky::{BodySky, SkyPointKind, SkyState};
///
/// let empty = BodySky { state: SkyState::AlwaysBelow, track: Vec::new() };
/// assert_eq!(empty.points(SkyPointKind::Rise).count(), 0);
/// ```
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct BodySky {
    /// How the window stood to the horizon.
    pub state: SkyState,
    /// Every step's reading and every exact event, in time order.
    pub track: Vec<SkyPoint>,
}

impl BodySky {
    /// The track's points of a kind, in time order.
    ///
    /// ```
    /// # use teistro_astro::day_sky::{BodySky, SkyPoint, SkyPointKind, SkyState};
    /// # use teistro_core::quantity::JulianDay;
    /// # let at = |kind| SkyPoint { at: JulianDay::literal(2_460_000.5), kind, altitude_deg: 1.0, refracted_altitude_deg: 1.0, azimuth_deg: 1.0, moon: None };
    /// let sun = BodySky { state: SkyState::Crosses, track: vec![at(SkyPointKind::Sample), at(SkyPointKind::Transit)] };
    /// assert_eq!(sun.points(SkyPointKind::Transit).count(), 1);
    /// ```
    pub fn points(&self, kind: SkyPointKind) -> impl Iterator<Item = &SkyPoint> {
        self.track.iter().filter(move |point| point.kind == kind)
    }
}

/// The sky of one day at one place.
///
/// ```
/// # use teistro_astro::day_sky::{BodySky, DaySky, SkyState, SkyStep};
/// # use teistro_core::quantity::JulianDay;
/// # let none = BodySky { state: SkyState::AlwaysBelow, track: Vec::new() };
/// let sky = DaySky {
///     from: JulianDay::literal(2_460_000.5),
///     to: JulianDay::literal(2_460_001.5),
///     step_minutes: SkyStep::default().minutes(),
///     sun: none.clone(),
///     moon: none,
/// };
/// assert_eq!(sky.step_minutes, 10);
/// ```
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct DaySky {
    /// The window's start, UT1 read as UTC.
    pub from: JulianDay<Ut1>,
    /// The window's end; an event here is the next window's.
    pub to: JulianDay<Ut1>,
    /// The step asked, minutes.
    pub step_minutes: u16,
    /// The Sun.
    pub sun: BodySky,
    /// The Moon.
    pub moon: BodySky,
}

/// One body's right ascension, declination and distance over a span, as
/// Chebyshev series.
struct Fit {
    start: f64,
    length: f64,
    series: [[f64; FIT_NODES]; 3],
}

impl Fit {
    /// The fit through the source's readings at the span's nodes, the
    /// right ascension unwrapped so it has no seam.
    fn read(
        source: &dyn ApparentPositions,
        body: Body,
        start: f64,
        length: f64,
    ) -> Result<Fit, Error> {
        let half = length / 2.0;
        let middle = start + half;
        #[expect(
            clippy::cast_precision_loss,
            reason = "a node's number is tens at most"
        )]
        let instants: Vec<JulianDay<Ut1>> = (0..FIT_NODES)
            .map(|node| {
                let angle = core::f64::consts::PI * node as f64 / (FIT_NODES - 1) as f64;
                JulianDay::literal(half.mul_add(math::cos(angle), middle))
            })
            .collect();
        let mut readings = Vec::with_capacity(FIT_NODES);
        source.apparent_many(body, &instants, &mut readings)?;
        let mut unwrapped = None;
        let mut next = readings.iter();
        let series = lobatto::<FIT_NODES, 3>(
            |_| {
                let Some(reading) = next.next() else {
                    return [0.0; 3];
                };
                let ra = unwrapped.map_or(reading.ra_deg, |last: f64| {
                    last + difference_deg(reading.ra_deg, last)
                });
                unwrapped = Some(ra);
                [ra, reading.dec_deg, reading.distance_au]
            },
            start,
            length,
        );
        Ok(Fit {
            start,
            length,
            series,
        })
    }

    /// The fit's position at an instant inside its span, which `contains`
    /// has checked.
    fn at(&self, t: f64) -> Apparent {
        let x = 2.0 * (t - self.start) / self.length - 1.0;
        let [ra, dec, distance] = &self.series;
        Apparent {
            ra_deg: normalise_deg(evaluate(ra, x).0),
            dec_deg: evaluate(dec, x).0,
            distance_au: evaluate(distance, x).0,
        }
    }

    fn contains(&self, t: f64) -> bool {
        (self.start..=self.start + self.length).contains(&t)
    }
}

/// The source with the Sun and the Moon answered from their fits: what
/// the window's searches read.
struct Fitted<'a> {
    source: &'a dyn ApparentPositions,
    sun: Fit,
    moon: Fit,
}

impl Fitted<'_> {
    fn fit_of(&self, body: Body, t: f64) -> Option<&Fit> {
        let fit = match body {
            Body::Sun => &self.sun,
            Body::Moon => &self.moon,
            _ => return None,
        };
        fit.contains(t).then_some(fit)
    }
}

impl ApparentPositions for Fitted<'_> {
    fn apparent(&self, body: Body, ut1: JulianDay<Ut1>) -> Result<Apparent, Error> {
        match self.fit_of(body, ut1.get()) {
            Some(fit) => Ok(fit.at(ut1.get())),
            None => self.source.apparent(body, ut1),
        }
    }

    fn describe(&self) -> String {
        self.source.describe()
    }
}

/// One instant's frame, which every body read at it shares.
struct Stance {
    /// The local apparent sidereal time, degrees.
    sidereal_deg: f64,
    /// The observer's geocentric position in the true equator of date,
    /// astronomical units.
    station: Vector3,
}

/// A body's place seen from a station.
struct Seen {
    /// Its geocentric vector, astronomical units.
    geocentric: Vector3,
    /// Its topocentric vector.
    topocentric: Vector3,
    ra_deg: f64,
    dec_deg: f64,
    hour_angle_deg: f64,
}

/// What a window's points are read from.
///
/// The nutation is read once, at the window's middle: over a day or two
/// it moves the sidereal time by under a hundredth of an arcsecond, and
/// a reading per point was a third of what the flag cost.
struct Reader<'a> {
    fitted: &'a Fitted<'a>,
    place: Place,
    delta_t: DeltaTModel,
    mean_obliquity: f64,
    longitude_nutation: f64,
    ecliptic_pole: Vector3,
}

impl Reader<'_> {
    fn stance(&self, at: JulianDay<Ut1>) -> Result<Stance, Error> {
        let (tt, _) = tt_of(at, self.delta_t)?;
        let (uta, utb) = at.split();
        let (tta, ttb) = tt.split();
        let greenwich = iau::gst06b_with(
            uta,
            utb,
            tta,
            ttb,
            self.mean_obliquity,
            self.longitude_nutation,
        );
        Ok(Stance {
            sidereal_deg: normalise_deg(greenwich * RAD2DEG + self.place.longitude.get()),
            station: observer_at(self.place, greenwich).position_au,
        })
    }

    fn seen(&self, body: Body, at: JulianDay<Ut1>, stance: &Stance) -> Result<Seen, Error> {
        let apparent = self.fitted.apparent(body, at)?;
        let geocentric = sxp(
            apparent.distance_au,
            &s2c(apparent.ra_deg * DEG2RAD, apparent.dec_deg * DEG2RAD),
        );
        let topocentric = minus(&geocentric, &stance.station);
        let (ra, dec) = c2s(&topocentric);
        let ra_deg = ra * RAD2DEG;
        Ok(Seen {
            geocentric,
            topocentric,
            ra_deg,
            dec_deg: dec * RAD2DEG,
            hour_angle_deg: difference_deg(stance.sidereal_deg, ra_deg),
        })
    }

    /// The point of `body` at an instant.
    fn point(
        &self,
        body: Body,
        kind: SkyPointKind,
        at: JulianDay<Ut1>,
        stance: &Stance,
    ) -> Result<SkyPoint, Error> {
        let seen = self.seen(body, at, stance)?;
        let latitude = self.place.latitude.get();
        let (altitude_deg, azimuth_deg) = horizontal(seen.hour_angle_deg, seen.dec_deg, latitude);
        let moon = if body == Body::Moon {
            let sun = self.seen(Body::Sun, at, stance)?;
            Some(shape(&seen, &sun, self.ecliptic_pole, latitude))
        } else {
            None
        };
        Ok(SkyPoint {
            at,
            kind,
            altitude_deg,
            refracted_altitude_deg: refracted_deg(altitude_deg),
            azimuth_deg,
            moon,
        })
    }
}

fn minus(left: &Vector3, right: &Vector3) -> Vector3 {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

fn cross(left: &Vector3, right: &Vector3) -> Vector3 {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
}

/// The altitude and the azimuth from north through east, degrees, of a
/// direction at an hour angle (Meeus, ch. 13).
fn horizontal(hour_angle_deg: f64, dec_deg: f64, latitude_deg: f64) -> (f64, f64) {
    let (sin_h, cos_h) = math::sin_cos(hour_angle_deg * DEG2RAD);
    let (sin_dec, cos_dec) = math::sin_cos(dec_deg * DEG2RAD);
    let (sin_phi, cos_phi) = math::sin_cos(latitude_deg * DEG2RAD);
    let sin_alt = sin_phi * sin_dec + cos_phi * cos_dec * cos_h;
    let azimuth = math::atan2(
        -cos_dec * sin_h,
        cos_phi * sin_dec - sin_phi * cos_dec * cos_h,
    );
    (
        math::asin(sin_alt.clamp(-1.0, 1.0)) * RAD2DEG,
        normalise_deg(azimuth * RAD2DEG),
    )
}

/// The airless altitude refracted under the standard atmosphere,
/// degrees (Sæmundsson 1986, Meeus eq. 16.4), or the airless altitude
/// where the body stays below the horizon (crux C973).
fn refracted_deg(altitude_deg: f64) -> f64 {
    // Sæmundsson's fit turns over below about −3°, and anything that low
    // is below the horizon however it is refracted.
    const FLOOR_DEG: f64 = -2.0;
    if altitude_deg < FLOOR_DEG {
        return altitude_deg;
    }
    let lifted = altitude_deg + 10.3 / (altitude_deg + 5.11);
    let refraction_deg = 1.02 / math::tan(lifted * DEG2RAD) / 60.0;
    let refracted = altitude_deg + refraction_deg;
    if refracted < 0.0 {
        altitude_deg
    } else {
        refracted
    }
}

/// The Moon's shape, from her place and the Sun's as the station sees
/// them, and the Earth's-centre sign of her elongation.
fn shape(moon: &Seen, sun: &Seen, ecliptic_pole: Vector3, latitude_deg: f64) -> MoonShape {
    let phase = phase_of(
        &moon.topocentric,
        &minus(&moon.topocentric, &sun.topocentric),
    );
    let (sun_ra, sun_dec) = (sun.ra_deg, sun.dec_deg);
    MoonShape {
        illuminated_fraction: phase.illuminated_fraction,
        phase_angle_deg: phase.angle_deg,
        waxing: pdp(&cross(&sun.geocentric, &moon.geocentric), &ecliptic_pole) > 0.0,
        bright_limb_deg: bright_limb_deg((moon.ra_deg, moon.dec_deg), (sun_ra, sun_dec)),
        parallactic_angle_deg: parallactic_angle_deg(
            moon.hour_angle_deg,
            moon.dec_deg,
            latitude_deg,
        ),
    }
}

/// Every event of a kind in `[from, to)`, within [`EDGE_DAYS`] of its
/// edges.
fn events_in(
    solver: &Solver<'_>,
    kind: HorizonEventKind,
    from: f64,
    to: f64,
    out: &mut Vec<JulianDay<Ut1>>,
) -> Result<(), Error> {
    let end = to - EDGE_DAYS;
    let mut start = from - EDGE_DAYS;
    while start < end {
        let Some(event) = solver.event(kind, JulianDay::literal(start), end - start)? else {
            break;
        };
        let at = event.instant;
        if at.get() < end {
            out.push(at);
        }
        start = at.get() + NEXT_EVENT_GAP_DAYS;
    }
    Ok(())
}

/// One body's day over a window.
fn body_sky(
    reader: &Reader<'_>,
    body: Body,
    horizon: Horizon,
    (from, to): (JulianDay<Ut1>, JulianDay<Ut1>),
    samples: &[(JulianDay<Ut1>, Stance)],
) -> Result<BodySky, Error> {
    let solver = Solver::new(reader.fitted, body, reader.place, horizon, reader.delta_t);
    let mut track = Vec::with_capacity(samples.len() + 6);
    for (at, stance) in samples {
        track.push(reader.point(body, SkyPointKind::Sample, *at, stance)?);
    }
    let mut crossed = false;
    for (event, kind) in [
        (HorizonEventKind::Rise, SkyPointKind::Rise),
        (HorizonEventKind::Transit, SkyPointKind::Transit),
        (HorizonEventKind::Set, SkyPointKind::Set),
    ] {
        let mut instants = Vec::new();
        events_in(&solver, event, from.get(), to.get(), &mut instants)?;
        crossed |= kind != SkyPointKind::Transit && !instants.is_empty();
        for at in instants {
            track.push(reader.point(body, kind, at, &reader.stance(at)?)?);
        }
    }
    track.sort_by(|left, right| left.at.get().total_cmp(&right.at.get()));
    let state = if crossed {
        SkyState::Crosses
    } else if solver.is_above(from)? {
        SkyState::AlwaysAbove
    } else {
        SkyState::AlwaysBelow
    };
    Ok(BodySky { state, track })
}

/// The sky of a window at a place: `from` to `to` in UT1, each body's path
/// at `step` and its events under `horizon`.
///
/// ```
/// # use teistro_astro::day_sky::{SkyState, SkyStep, day_sky};
/// # use teistro_astro::{Completion, DeltaTModel};
/// # use teistro_core::quantity::{JulianDay, Place, Ut1};
/// # use teistro_core::settings::OverridePolicy;
/// # use teistro_port_ephemeris::{Horizon, TestProvider};
/// # let provider = TestProvider::new();
/// # let delta_t = DeltaTModel::TableThenModel;
/// # let source = Completion::new(&provider, OverridePolicy::SdkOnly, delta_t);
/// let tromso = Place::try_from_degrees(69.65, 18.96, 0.0)?;
/// // The local mean day of 2024-06-21 there: the midnight Sun.
/// let from = JulianDay::<Ut1>::literal(2_460_482.5 - 18.96 / 360.0);
/// let to = JulianDay::<Ut1>::literal(from.get() + 1.0);
/// let sky = day_sky(&source, tromso, Horizon::UPPER_LIMB_REFRACTION, delta_t, (from, to), SkyStep::of_minutes(60)?)?;
/// assert_eq!(sky.sun.state, SkyState::AlwaysAbove);
/// // A step of an hour over a day, both ends, and the noon.
/// assert_eq!(sky.sun.track.len(), 26);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// `INVALID_ARG` for a window the wrong way round or empty; the source's
/// refusal, or a Delta T model that cannot answer.
pub fn day_sky(
    source: &dyn ApparentPositions,
    place: Place,
    horizon: Horizon,
    delta_t: DeltaTModel,
    (from, to): (JulianDay<Ut1>, JulianDay<Ut1>),
    step: SkyStep,
) -> Result<DaySky, Error> {
    let days = to.get() - from.get();
    if !(days.is_finite() && days > 0.0) {
        return Err(Error::invalid_arg(format!(
            "a sky window runs forwards, not JD {} to {}",
            from.get(),
            to.get()
        ))
        .with_field("window"));
    }
    let start = from.get() - FIT_LEAD_DAYS;
    let length = FIT_LEAD_DAYS + days + FIT_TRAIL_DAYS;
    let fitted = Fitted {
        source,
        sun: Fit::read(source, Body::Sun, start, length)?,
        moon: Fit::read(source, Body::Moon, start, length)?,
    };
    let middle = JulianDay::<Ut1>::literal(from.get() + days / 2.0);
    let (tt, _) = tt_of(middle, delta_t)?;
    let (obliquity, mean_obliquity, longitude_nutation) = obliquity_parts(tt);
    let (sin_e, cos_e) = math::sin_cos(obliquity.true_deg * DEG2RAD);
    let reader = Reader {
        fitted: &fitted,
        place,
        delta_t,
        mean_obliquity,
        longitude_nutation,
        ecliptic_pole: [0.0, -sin_e, cos_e],
    };
    let stepped = f64::from(step.minutes()) / MINUTES_A_DAY;
    let mut samples = Vec::new();
    let mut count = 0.0;
    loop {
        let at = from.get() + count * stepped;
        if at >= to.get() {
            break;
        }
        let instant = JulianDay::literal(at);
        samples.push((instant, reader.stance(instant)?));
        count += 1.0;
    }
    samples.push((to, reader.stance(to)?));
    Ok(DaySky {
        from,
        to,
        step_minutes: step.minutes(),
        sun: body_sky(&reader, Body::Sun, horizon, (from, to), &samples)?,
        moon: body_sky(&reader, Body::Moon, horizon, (from, to), &samples)?,
    })
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index what they asked for"
    )]

    use teistro_core::settings::OverridePolicy;
    use teistro_port_ephemeris::TestProvider;

    use super::*;
    use crate::Completion;

    const DELTA_T: DeltaTModel = DeltaTModel::TableThenModel;

    fn window(start: f64) -> (JulianDay<Ut1>, JulianDay<Ut1>) {
        (JulianDay::literal(start), JulianDay::literal(start + 1.0))
    }

    /// Meeus, example 13.b: Venus from the US Naval Observatory, 1987
    /// April 10, 19:21 UT. `H = 64.3521°`, `δ = −6.7199°`, `φ = 38.9214°`;
    /// `A = 68.0337°` from the south, `h = 15.1249°`.
    #[test]
    fn altitude_and_azimuth_reproduce_meeus_13b() {
        let (altitude, azimuth) = horizontal(64.352_133, -6.719_892, 38.921_389);
        assert!((altitude - 15.124_9).abs() < 1e-3, "{altitude}");
        assert!((azimuth - 248.033_7).abs() < 1e-3, "{azimuth}");
    }

    #[test]
    fn refraction_is_saemundsson_above_the_horizon_and_nothing_below() {
        // Meeus, ch. 16: about 34′ on the horizon and 1′ at 45°.
        assert!((refracted_deg(0.0) - 0.48).abs() < 0.01);
        assert!((refracted_deg(45.0) - 45.0 - 1.0 / 60.0).abs() < 1e-3);
        // Still below the horizon once refracted, and far below.
        assert_eq!(refracted_deg(-1.0), -1.0);
        assert_eq!(refracted_deg(-40.0), -40.0);
    }

    #[test]
    fn a_window_the_wrong_way_round_is_refused() {
        let provider = TestProvider::new();
        let source = Completion::new(&provider, OverridePolicy::SdkOnly, DELTA_T);
        let place = Place::try_from_degrees(0.0, 0.0, 0.0).unwrap();
        let (from, to) = window(2_460_000.5);
        let refused = day_sky(
            &source,
            place,
            Horizon::UPPER_LIMB_REFRACTION,
            DELTA_T,
            (to, from),
            SkyStep::default(),
        )
        .unwrap_err();
        assert_eq!(refused.field(), Some("window"));
    }

    #[test]
    fn the_fit_answers_what_the_source_answers_across_the_seam() {
        let provider = TestProvider::new();
        let source = Completion::new(&provider, OverridePolicy::SdkOnly, DELTA_T);
        let (start, length) = (2_460_000.5, 2.2);
        for body in [Body::Sun, Body::Moon] {
            let fit = Fit::read(&source, body, start, length).unwrap();
            for tenth in 0..=22 {
                let t = start + f64::from(tenth) / 10.0;
                let want = source.apparent(body, JulianDay::literal(t)).unwrap();
                let got = fit.at(t);
                assert!(difference_deg(got.ra_deg, want.ra_deg).abs() < 1e-5);
                assert!((got.dec_deg - want.dec_deg).abs() < 1e-5);
                assert!((got.distance_au - want.distance_au).abs() < 1e-9);
            }
            assert!(!fit.contains(start + length + 1.0));
        }
    }

    #[test]
    fn a_search_past_the_fit_reads_the_source() {
        let provider = TestProvider::new();
        let source = Completion::new(&provider, OverridePolicy::SdkOnly, DELTA_T);
        let (start, length) = (2_460_000.5, 1.0);
        let fitted = Fitted {
            source: &source,
            sun: Fit::read(&source, Body::Sun, start, length).unwrap(),
            moon: Fit::read(&source, Body::Moon, start, length).unwrap(),
        };
        let beyond = JulianDay::literal(start + 3.0);
        assert_eq!(
            fitted.apparent(Body::Moon, beyond).unwrap(),
            source.apparent(Body::Moon, beyond).unwrap()
        );
        assert_eq!(
            fitted.apparent(Body::Mars, beyond).unwrap(),
            source.apparent(Body::Mars, beyond).unwrap()
        );
        assert_eq!(fitted.describe(), source.describe());
    }

    #[test]
    fn events_fall_in_a_half_open_window_and_neighbours_share_none() {
        let provider = TestProvider::new();
        let source = Completion::new(&provider, OverridePolicy::SdkOnly, DELTA_T);
        let place = Place::try_from_degrees(27.7172, 85.324, 0.0).unwrap();
        let horizon = Horizon::UPPER_LIMB_REFRACTION;
        let start = 2_460_482.5 - 85.324 / 360.0;
        let [first, second] = [window(start), window(start + 1.0)]
            .map(|w| day_sky(&source, place, horizon, DELTA_T, w, SkyStep::default()).unwrap());
        let exact = |sky: &DaySky| -> Vec<f64> {
            [&sky.sun, &sky.moon]
                .iter()
                .flat_map(|body| body.track.iter())
                .filter(|point| point.kind != SkyPointKind::Sample)
                .map(|point| point.at.get())
                .collect()
        };
        for at in exact(&first) {
            assert!(!exact(&second).contains(&at));
            assert!(at >= start - EDGE_DAYS && at < start + 1.0);
        }
        // Sorted, opening at the window's start and closing at its end.
        let track = &first.sun.track;
        assert!(
            track
                .windows(2)
                .all(|pair| pair[0].at.get() <= pair[1].at.get())
        );
        assert_eq!(track[0].at, first.from);
        assert_eq!(track[track.len() - 1].at, first.to);
    }

    /// A source that counts what it is asked.
    struct Counted<'a> {
        source: &'a dyn ApparentPositions,
        reads: core::sync::atomic::AtomicUsize,
    }

    impl ApparentPositions for Counted<'_> {
        fn apparent(&self, body: Body, ut1: JulianDay<Ut1>) -> Result<Apparent, Error> {
            self.reads
                .fetch_add(1, core::sync::atomic::Ordering::Relaxed);
            self.source.apparent(body, ut1)
        }

        fn describe(&self) -> String {
            self.source.describe()
        }
    }

    /// The price of a day's sky is its nodes, whatever the step.
    #[test]
    fn the_step_does_not_change_what_the_source_is_asked() {
        let provider = TestProvider::new();
        let source = Completion::new(&provider, OverridePolicy::SdkOnly, DELTA_T);
        let place = Place::try_from_degrees(27.7172, 85.324, 1_400.0).unwrap();
        let reads = |minutes| {
            let counted = Counted {
                source: &source,
                reads: core::sync::atomic::AtomicUsize::new(0),
            };
            day_sky(
                &counted,
                place,
                Horizon::UPPER_LIMB_REFRACTION,
                DELTA_T,
                window(2_460_482.5 - 85.324 / 360.0),
                SkyStep::of_minutes(minutes).unwrap(),
            )
            .unwrap();
            counted.reads.into_inner()
        };
        assert_eq!(reads(1), 2 * FIT_NODES);
        assert_eq!(reads(240), 2 * FIT_NODES);
    }

    #[test]
    fn a_polar_night_is_said() {
        let provider = TestProvider::new();
        let source = Completion::new(&provider, OverridePolicy::SdkOnly, DELTA_T);
        let place = Place::try_from_degrees(69.65, 18.96, 0.0).unwrap();
        // The local mean day of 2024-12-21.
        let sky = day_sky(
            &source,
            place,
            Horizon::UPPER_LIMB_REFRACTION,
            DELTA_T,
            window(2_460_665.5 - 18.96 / 360.0),
            SkyStep::of_minutes(60).unwrap(),
        )
        .unwrap();
        assert_eq!(sky.sun.state, SkyState::AlwaysBelow);
        assert_eq!(sky.sun.points(SkyPointKind::Rise).count(), 0);
        assert_eq!(sky.sun.points(SkyPointKind::Transit).count(), 1);
    }
}
