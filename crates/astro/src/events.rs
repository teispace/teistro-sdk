//! Crossings and stations (`docs/03-design/astro-events-and-crossings.md`,
//! §4): when a body's longitude, a composite angle of two bodies'
//! longitudes (the tithi, the yoga, an aspect), or a body's speed crosses
//! a boundary. One kernel over the shared boundary solver: the quantity is
//! sampled at a step bounded by the fastest motion involved and the
//! lattice spacing, every lattice line passed between two samples is
//! narrowed to the tolerance by the shared solver, and a station is the
//! speed's sign change found the same way.
//!
//! ```
//! use teistro_astro::events::{Lattice, Quantity, Search};
//! use teistro_astro::{Completion, DeltaTModel};
//! use teistro_core::quantity::{JulianDay, Ut1};
//! use teistro_core::settings::OverridePolicy;
//! use teistro_port_ephemeris::{Body, Frame, TestProvider};
//!
//! let provider = TestProvider::new();
//! let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
//! let longitudes = completion.longitudes(Frame::CANONICAL);
//! // The Sun's sign ingresses in a year: twelve of them, about a month apart.
//! let ingresses = Search::new(&longitudes, Quantity::Longitude(Body::Sun), Lattice::SIGNS)
//!     .between(JulianDay::<Ut1>::literal(2_451_545.0), JulianDay::<Ut1>::literal(2_451_545.0 + 365.25))
//!     .expect("the test provider answers");
//! assert_eq!(ingresses.len(), 12);
//! ```

use core::cmp::Ordering;
use core::fmt;

use serde::Serialize;
use teistro_core::angle::{difference_deg, normalise_deg};
use teistro_core::error::{Error, Status};
use teistro_core::quantity::{JulianDay, Place, Ut1};
use teistro_core::settings::OverridePolicy;
use teistro_port_ephemeris::{
    Body, CrossingRequest, EphemerisProvider, Frame, Overrides, PositionRequest, ProviderError,
    TimeScale,
};

use crate::completion::{Completed, Completion, CompletionError, Implementation};
use crate::solve::{Caps, SolveError, first_zero, refine};

/// The tolerance a crossing is found to, days: a hundredth of a second,
/// a hundredth of the target the kernel is held to against the engines.
pub const TOLERANCE_DAYS: f64 = 1e-7;

/// The longest sampling step, days: no retrograde arc of a planet is
/// shorter than this, so a boundary crossed and re-crossed inside one step
/// is a body lingering within a fraction of an arcminute of it.
pub const STEP_CAP_DAYS: f64 = 1.0;

/// The spacing a single target is treated as having: a whole circle.
const SINGLE_TARGET_SPACING_DEG: f64 = 360.0;

/// The most samples one search takes, so a wide window with a small step
/// cannot run away.
const SAMPLE_CAP: u32 = 2_000_000;

/// A source of ecliptic longitudes and their rates, degrees and degrees a
/// day, at UT1 instants: the frame completion over a provider in the frame
/// a caller chooses, or a classical model.
pub trait Longitudes: Send + Sync {
    /// A body's longitude and its rate.
    ///
    /// # Errors
    ///
    /// An instant or a body the source cannot answer for.
    fn longitude_and_speed(&self, body: Body, ut1: JulianDay<Ut1>) -> Result<(f64, f64), Error>;

    /// Two bodies' longitudes and rates at one instant, in the order asked:
    /// what a composite quantity (the tithi, an aspect) and a visibility
    /// reading (the body and the Sun) need at every sample. A source that
    /// can answer two bodies in one request does so here, sharing the
    /// instant's obliquity, nutation and precession between them; the
    /// default reads them one by one.
    ///
    /// # Errors
    ///
    /// An instant or a body the source cannot answer for.
    fn longitude_and_speed_pair(
        &self,
        bodies: [Body; 2],
        ut1: JulianDay<Ut1>,
    ) -> Result<[(f64, f64); 2], Error> {
        let [first, second] = bodies;
        Ok([
            self.longitude_and_speed(first, ut1)?,
            self.longitude_and_speed(second, ut1)?,
        ])
    }

    /// The source's name for provenance stamps.
    fn describe(&self) -> String;
}

impl<S: Longitudes + ?Sized> Longitudes for &S {
    fn longitude_and_speed(&self, body: Body, ut1: JulianDay<Ut1>) -> Result<(f64, f64), Error> {
        (**self).longitude_and_speed(body, ut1)
    }

    fn longitude_and_speed_pair(
        &self,
        bodies: [Body; 2],
        ut1: JulianDay<Ut1>,
    ) -> Result<[(f64, f64); 2], Error> {
        (**self).longitude_and_speed_pair(bodies, ut1)
    }

    fn describe(&self) -> String {
        (**self).describe()
    }
}

/// The frame completion as a source of longitudes in one frame, geocentric
/// or topocentric.
pub struct FrameLongitudes<'c, P: EphemerisProvider + ?Sized> {
    completion: &'c Completion<'c, P>,
    frame: Frame,
    observer: Option<Place>,
}

impl<P: EphemerisProvider + ?Sized> Completion<'_, P> {
    /// This completion as a source of longitudes in a frame.
    #[must_use]
    pub fn longitudes(&self, frame: Frame) -> FrameLongitudes<'_, P> {
        FrameLongitudes {
            completion: self,
            frame,
            observer: None,
        }
    }
}

impl<P: EphemerisProvider + ?Sized> fmt::Debug for FrameLongitudes<'_, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FrameLongitudes")
            .field("provider", &self.completion.capabilities().identity.name)
            .field("frame", &self.frame)
            .field("observer", &self.observer)
            .finish()
    }
}

impl<P: EphemerisProvider + ?Sized> FrameLongitudes<'_, P> {
    /// The observer, for a topocentric frame.
    #[must_use]
    pub fn with_observer(mut self, place: Place) -> Self {
        self.observer = Some(place);
        self
    }
}

impl<P: EphemerisProvider + ?Sized> FrameLongitudes<'_, P> {
    /// One request for the bodies at the instant, completed in the frame.
    fn request(&self, bodies: &[Body], ut1: JulianDay<Ut1>) -> Result<Completed, Error> {
        let jds = [ut1.get()];
        let mut request = PositionRequest::new(&jds, TimeScale::Ut1, bodies, self.frame);
        request.observer = self.observer;
        Ok(self.completion.positions(&request)?)
    }
}

/// The longitude and rate in one completed cell, or the refusal it holds.
fn reading(
    done: &Completed,
    index: usize,
    body: Body,
    ut1: JulianDay<Ut1>,
) -> Result<(f64, f64), Error> {
    let cell = done
        .columns
        .at(0, index)
        .ok_or_else(|| Error::new(Status::Provider, format!("no cell for {}", body.key())))?;
    if !cell.is_ok() {
        return Err(Error::new(
            Status::Provider,
            format!("{} at JD {}: {:?}", body.key(), ut1.get(), cell.status),
        ));
    }
    Ok((cell.lon, cell.lon_speed))
}

impl<P: EphemerisProvider + ?Sized> Longitudes for FrameLongitudes<'_, P> {
    fn longitude_and_speed(&self, body: Body, ut1: JulianDay<Ut1>) -> Result<(f64, f64), Error> {
        let done = self.request(&[body], ut1)?;
        reading(&done, 0, body, ut1)
    }

    fn longitude_and_speed_pair(
        &self,
        bodies: [Body; 2],
        ut1: JulianDay<Ut1>,
    ) -> Result<[(f64, f64); 2], Error> {
        let done = self.request(&bodies, ut1)?;
        let [first, second] = bodies;
        Ok([
            reading(&done, 0, first, ut1)?,
            reading(&done, 1, second, ut1)?,
        ])
    }

    fn describe(&self) -> String {
        format!(
            "{} in {}",
            self.completion.capabilities().identity.name,
            self.frame
        )
    }
}

pub use teistro_port_ephemeris::crossing::{Direction, Event, Lattice, Quantity};

/// The greatest rate the quantity reaches, degrees a day: a body's own,
/// or the coefficients' weighted sum of the two bodies' for a composite.
#[must_use]
pub fn quantity_rate_deg_per_day(quantity: Quantity) -> f64 {
    match quantity {
        Quantity::Longitude(body) | Quantity::Speed(body) => greatest_rate(body),
        Quantity::Composite {
            a,
            first,
            b,
            second,
        } => a.abs() * greatest_rate(first) + b.abs() * greatest_rate(second),
    }
}

/// The quantity at an instant from a source of longitudes: an angle reduced
/// to `[0, 360)`, or a rate in degrees a day.
fn evaluate<S: Longitudes + ?Sized>(
    quantity: Quantity,
    source: &S,
    ut1: JulianDay<Ut1>,
) -> Result<f64, Error> {
    Ok(match quantity {
        Quantity::Longitude(body) => source.longitude_and_speed(body, ut1)?.0,
        Quantity::Speed(body) => source.longitude_and_speed(body, ut1)?.1,
        Quantity::Composite {
            a,
            first,
            b,
            second,
        } => {
            let [(x, _), (y, _)] = source.longitude_and_speed_pair([first, second], ut1)?;
            normalise_deg(a * x + b * y)
        }
    })
}

/// The spacing the kernel steps by: a single target is one line a circle
/// apart from itself.
fn spacing_deg(lattice: &Lattice) -> f64 {
    if lattice.is_single() {
        SINGLE_TARGET_SPACING_DEG
    } else {
        lattice.step_deg
    }
}

/// The `k`th line of the lattice on the unwrapped curve.
fn line_deg(lattice: &Lattice, k: i64) -> f64 {
    // A line index is a small integer.
    #[allow(
        clippy::cast_precision_loss,
        reason = "lattice indices are small integers"
    )]
    let k = k as f64;
    lattice.origin_deg + k * spacing_deg(lattice)
}

/// The greatest geocentric rate of longitude a body reaches, degrees a
/// day, with a margin: the Moon at perigee, Mercury at inferior
/// conjunction, the true node's swings; bodies this table does not know
/// take Mercury's.
#[must_use]
pub fn greatest_rate(body: Body) -> f64 {
    match body {
        Body::Sun => 1.03,
        Body::Moon => 15.5,
        Body::Venus => 1.3,
        Body::Mars => 0.9,
        Body::Jupiter => 0.3,
        Body::Saturn => 0.15,
        Body::Uranus => 0.07,
        Body::Neptune | Body::Pluto => 0.05,
        Body::MeanNode | Body::MeanApogee => 0.12,
        Body::TrueNode | Body::OsculatingApogee => 0.6,
        // Mercury, the fastest planet, and any body the port adds later.
        _ => 2.3,
    }
}

/// Which way a station turns.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StationKind {
    /// Direct motion ends: the speed passes from positive to negative.
    Retrograde,
    /// Retrograde motion ends: the speed passes from negative to positive.
    Direct,
}

/// A station: the instant a body's longitude stands still.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Station {
    /// The instant, UT1.
    pub instant: JulianDay<Ut1>,
    /// The body's longitude there, degrees.
    pub longitude_deg: f64,
    /// Which way it turns.
    pub kind: StationKind,
    /// How many times the source was asked.
    pub evaluations: u32,
}

/// The events of a crossing search over the completion, with who found
/// them.
#[derive(Clone, Debug, PartialEq)]
pub struct Crossings {
    /// The events, in time order.
    pub events: Vec<Event>,
    /// The provider's own search, or the SDK's kernel.
    pub implementation: Implementation,
}

impl<P: EphemerisProvider + ?Sized> Completion<'_, P> {
    /// Every crossing of a quantity over a lattice inside a window, by the
    /// override policy: a provider that declares the crossings override
    /// answers under `PREFER_NATIVE` and `NATIVE_ONLY`, the SDK's kernel
    /// over the provider's positions otherwise; under `PREFER_NATIVE` a
    /// request the provider refuses (a topocentric frame, say) falls to the
    /// kernel.
    ///
    /// ```
    /// use teistro_astro::events::Lattice;
    /// use teistro_astro::{Completion, DeltaTModel};
    /// use teistro_core::quantity::{JulianDay, Ut1};
    /// use teistro_core::settings::OverridePolicy;
    /// use teistro_port_ephemeris::{Body, CrossingRequest, Frame, Quantity, TestProvider};
    ///
    /// let provider = TestProvider::new();
    /// let completion = Completion::new(&provider, OverridePolicy::PreferNative, DeltaTModel::TableThenModel);
    /// let found = completion.crossings(&CrossingRequest {
    ///     quantity: Quantity::Longitude(Body::Sun),
    ///     lattice: Lattice::SIGNS,
    ///     from: JulianDay::<Ut1>::J2000,
    ///     to: JulianDay::<Ut1>::literal(2_451_545.0 + 365.25),
    ///     tolerance_days: 1e-7,
    ///     frame: Frame::CANONICAL,
    ///     observer: None,
    /// }).expect("the Sun's ingresses");
    /// assert_eq!(found.events.len(), 12);
    /// ```
    ///
    /// # Errors
    ///
    /// The policy's refusal (`NATIVE_ONLY` without the override), the
    /// provider's error, or the kernel's (`INVALID_ARG` for a bad window).
    pub fn crossings(&self, request: &CrossingRequest) -> Result<Crossings, Error> {
        let implementation = self.choose(Overrides::CROSSINGS, "crossings")?;
        if implementation == Implementation::Native {
            match self.provider().crossings(request) {
                Ok(events) => {
                    return Ok(Crossings {
                        events,
                        implementation,
                    });
                }
                Err(ProviderError::Unsupported { .. })
                    if self.policy() == OverridePolicy::PreferNative => {}
                Err(error) => return Err(CompletionError::from(error).into()),
            }
        }
        let mut longitudes = self.longitudes(request.frame);
        if let Some(place) = request.observer {
            longitudes = longitudes.with_observer(place);
        }
        let events = Search::new(&longitudes, request.quantity, request.lattice)
            .with_tolerance_days(request.tolerance_days)
            .between(request.from, request.to)?;
        Ok(Crossings {
            events,
            implementation: Implementation::Sdk,
        })
    }
}

/// A search for the crossings of a quantity over a lattice.
pub struct Search<'s, S: Longitudes + ?Sized> {
    source: &'s S,
    quantity: Quantity,
    lattice: Lattice,
    tolerance_days: f64,
    step_days: Option<f64>,
    caps: Caps,
}

impl<S: Longitudes + ?Sized> fmt::Debug for Search<'_, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Search")
            .field("source", &self.source.describe())
            .field("quantity", &self.quantity)
            .field("lattice", &self.lattice)
            .field("tolerance_days", &self.tolerance_days)
            .field("step_days", &self.step_days())
            .field("caps", &self.caps)
            .finish()
    }
}

impl<'s, S: Longitudes + ?Sized> Search<'s, S> {
    /// A search with the default tolerance, the step rule and caps.
    #[must_use]
    pub const fn new(source: &'s S, quantity: Quantity, lattice: Lattice) -> Search<'s, S> {
        Search {
            source,
            quantity,
            lattice,
            tolerance_days: TOLERANCE_DAYS,
            step_days: None,
            caps: Caps::DEFAULT,
        }
    }

    /// The tolerance an instant is found to, days.
    #[must_use]
    pub const fn with_tolerance_days(mut self, tolerance_days: f64) -> Self {
        self.tolerance_days = tolerance_days;
        self
    }

    /// A sampling step instead of the rule's.
    #[must_use]
    pub const fn with_step_days(mut self, step_days: f64) -> Self {
        self.step_days = Some(step_days);
        self
    }

    /// The step the search samples at: half the lattice spacing at the
    /// quantity's greatest rate, never more than a day, so no line is
    /// passed twice between two samples and no retrograde arc is stepped
    /// over.
    #[must_use]
    pub fn step_days(&self) -> f64 {
        self.step_days.unwrap_or_else(|| {
            let rate = quantity_rate_deg_per_day(self.quantity);
            (spacing_deg(&self.lattice) / rate * 0.5).min(STEP_CAP_DAYS)
        })
    }

    fn check(&self) -> Result<(), Error> {
        if !(self.lattice.origin_deg.is_finite() && self.lattice.step_deg.is_finite())
            || self.lattice.step_deg < 0.0
        {
            return Err(Error::invalid_arg(
                "a lattice needs a finite origin and a non-negative step",
            )
            .with_field("lattice"));
        }
        for (name, value) in [
            ("tolerance_days", self.tolerance_days),
            ("step_days", self.step_days()),
        ] {
            if !(value.is_finite() && value > 0.0) {
                return Err(Error::invalid_arg(format!(
                    "the search's {name} must be a positive finite number, not {value}"
                ))
                .with_field(name));
            }
        }
        Ok(())
    }

    /// Every crossing between two instants, in time order.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` for an empty or reversed window or a bad tolerance,
    /// step or lattice; the source's error; `NOT_CONVERGED` should a
    /// bracket fail to narrow.
    pub fn between(&self, from: JulianDay<Ut1>, to: JulianDay<Ut1>) -> Result<Vec<Event>, Error> {
        self.check()?;
        check_window("search", from, to)?;
        let step = self.step_days();
        let wraps = self.quantity.wraps();
        let mut events = Vec::new();
        let sample = |t: f64| -> Result<f64, Error> {
            evaluate(self.quantity, self.source, JulianDay::literal(t))
        };
        // The quantity unwrapped along the samples, so a lattice line is a
        // level on a continuous curve.
        let mut t_lo = from.get();
        let mut raw_lo = sample(t_lo)?;
        let mut unwrapped_lo = raw_lo;
        let mut samples = 0u32;
        while t_lo < to.get() {
            if samples >= SAMPLE_CAP {
                return Err(Error::new(
                    Status::NotConverged,
                    format!("the crossing search took more than {SAMPLE_CAP} samples"),
                ));
            }
            samples += 1;
            let t_hi = (t_lo + step).min(to.get());
            let raw_hi = sample(t_hi)?;
            let delta = if wraps {
                difference_deg(raw_hi, raw_lo)
            } else {
                raw_hi - raw_lo
            };
            let unwrapped_hi = unwrapped_lo + delta;
            if delta != 0.0 {
                for k in lines_between(&self.lattice, unwrapped_lo, unwrapped_hi) {
                    let line = line_deg(&self.lattice, k);
                    let rising = delta > 0.0;
                    // The signed distance to the line along the unwrapped
                    // curve, negative before it, so the bracket has the shape
                    // the solver expects. The curve is unwrapped exactly as
                    // the samples were, so the bracket's ends carry the
                    // values the lattice test saw and a line met at a sample
                    // still brackets.
                    let gap = |t: f64| -> Result<f64, Error> {
                        let value = evaluate(self.quantity, self.source, JulianDay::literal(t))?;
                        let advance = if wraps {
                            difference_deg(value, raw_lo)
                        } else {
                            value - raw_lo
                        };
                        let distance = unwrapped_lo + advance - line;
                        Ok(if rising { distance } else { -distance })
                    };
                    let refined = refine(gap, t_lo, t_hi, self.tolerance_days, self.caps)
                        .map_err(solve_error)?;
                    events.push(Event {
                        instant: JulianDay::literal(refined.instant),
                        boundary_deg: if wraps { normalise_deg(line) } else { line },
                        direction: if rising {
                            Direction::Rising
                        } else {
                            Direction::Falling
                        },
                        evaluations: refined.evaluations,
                    });
                }
            }
            t_lo = t_hi;
            raw_lo = raw_hi;
            unwrapped_lo = unwrapped_hi;
        }
        Ok(events)
    }

    /// The first crossing at or after an instant within a window of days.
    ///
    /// # Errors
    ///
    /// As [`Search::between`]; a non-positive window is `INVALID_ARG`.
    pub fn next_within(
        &self,
        from: JulianDay<Ut1>,
        window_days: f64,
    ) -> Result<Option<Event>, Error> {
        if !(window_days.is_finite() && window_days > 0.0) {
            return Err(Error::invalid_arg(format!(
                "the search window must be a positive number of days, not {window_days}"
            ))
            .with_field("window_days"));
        }
        let to = from.plus_days(window_days)?;
        Ok(self.between(from, to)?.into_iter().next())
    }
}

/// The lattice indices whose lines lie in the open interval from `a` to
/// `b` (either order), the line at `b` included: the lines a monotone
/// curve passes between two samples.
fn lines_between(lattice: &Lattice, a: f64, b: f64) -> Vec<i64> {
    let spacing = spacing_deg(lattice);
    let (lo, hi, forward) = if b > a { (a, b, true) } else { (b, a, false) };
    let first = ((lo - lattice.origin_deg) / spacing).floor();
    let last = ((hi - lattice.origin_deg) / spacing).floor();
    // The indices are bounded by the window over the spacing, small numbers.
    #[allow(
        clippy::cast_possible_truncation,
        reason = "lattice indices are small integers"
    )]
    let (first, last) = (first as i64, last as i64);
    let mut lines: Vec<i64> = Vec::new();
    for k in first..=last {
        let line = line_deg(lattice, k);
        let inside = if forward {
            line > lo && line <= hi
        } else {
            line >= lo && line < hi
        };
        if inside {
            lines.push(k);
        }
    }
    if !forward {
        lines.reverse();
    }
    lines
}

/// A window must run forward; a reversed, empty or unordered one is refused
/// by name.
pub(crate) fn check_window(
    what: &str,
    from: JulianDay<Ut1>,
    to: JulianDay<Ut1>,
) -> Result<(), Error> {
    if to.get().partial_cmp(&from.get()) == Some(Ordering::Greater) {
        return Ok(());
    }
    Err(Error::invalid_arg(format!(
        "the {what} window must run forward, not from {} to {}",
        from.get(),
        to.get()
    ))
    .with_field("to"))
}

fn solve_error(error: SolveError<Error>) -> Error {
    match error {
        SolveError::Evaluation(inner) => inner,
        other => Error::new(Status::NotConverged, other.to_string()),
    }
}

/// The stations of a body between two instants: where its rate of
/// longitude changes sign, each found by the sign change and refined to
/// the tolerance.
///
/// # Errors
///
/// As [`Search::between`].
pub fn stations(
    source: &dyn Longitudes,
    body: Body,
    from: JulianDay<Ut1>,
    to: JulianDay<Ut1>,
    tolerance_days: f64,
) -> Result<Vec<Station>, Error> {
    check_window("station", from, to)?;
    let mut stations = Vec::new();
    for (upward, kind) in [
        (true, StationKind::Direct),
        (false, StationKind::Retrograde),
    ] {
        let mut start = from.get();
        while start < to.get() {
            let found = first_zero(
                |t| {
                    source
                        .longitude_and_speed(body, JulianDay::literal(t))
                        .map(|(_, speed)| speed)
                },
                start,
                to.get(),
                STEP_CAP_DAYS,
                upward,
                tolerance_days,
                Caps {
                    bracket_steps: SAMPLE_CAP,
                    ..Caps::DEFAULT
                },
            )
            .map_err(solve_error)?;
            let Some(crossing) = found else {
                break;
            };
            let instant = JulianDay::literal(crossing.instant);
            let (longitude_deg, _) = source.longitude_and_speed(body, instant)?;
            stations.push(Station {
                instant,
                longitude_deg,
                kind,
                evaluations: crossing.evaluations,
            });
            start = crossing.instant + STEP_CAP_DAYS;
        }
    }
    stations.sort_by(|a, b| a.instant.get().total_cmp(&b.instant.get()));
    Ok(stations)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::float_cmp,
        clippy::indexing_slicing,
        reason = "tests fail by panicking, compare chosen constants and read small lists"
    )]

    use teistro_core::settings::OverridePolicy;
    use teistro_port_ephemeris::TestProvider;

    use crate::delta_t::DeltaTModel;

    use super::*;

    const J2000: JulianDay<Ut1> = JulianDay::literal(2_451_545.0);

    #[test]
    fn lines_between_lists_the_lattice_lines_a_curve_passes() {
        let signs = Lattice::SIGNS;
        assert_eq!(lines_between(&signs, 25.0, 65.0), vec![1, 2]);
        assert_eq!(lines_between(&signs, 65.0, 25.0), vec![2, 1]);
        assert_eq!(lines_between(&signs, 30.0, 59.0), Vec::<i64>::new());
        assert_eq!(lines_between(&signs, 29.0, 30.0), vec![1]);
        assert_eq!(lines_between(&signs, -5.0, 5.0), vec![0]);
        let single = Lattice::single(100.0);
        assert_eq!(lines_between(&single, 90.0, 110.0), vec![0]);
        assert_eq!(lines_between(&single, 110.0, 130.0), Vec::<i64>::new());
    }

    #[test]
    fn the_step_rule_bounds_the_step_by_the_rate_and_a_day() {
        let provider = TestProvider::new();
        let completion = Completion::new(
            &provider,
            OverridePolicy::SdkOnly,
            DeltaTModel::TableThenModel,
        );
        let longitudes = completion.longitudes(Frame::CANONICAL);
        let tithis = Search::new(&longitudes, Quantity::ELONGATION, Lattice::TITHIS);
        // 12° at 16.5° a day, halved: 0.36 of a day.
        assert!((tithis.step_days() - 12.0 / 16.53 * 0.5).abs() < 1e-9);
        let sun = Search::new(&longitudes, Quantity::Longitude(Body::Sun), Lattice::SIGNS);
        assert_eq!(sun.step_days(), STEP_CAP_DAYS);
        let single = Search::new(
            &longitudes,
            Quantity::Longitude(Body::Saturn),
            Lattice::single(10.0),
        );
        assert_eq!(single.step_days(), STEP_CAP_DAYS);
        assert!((quantity_rate_deg_per_day(Quantity::MOON_PLUS_SUN) - 16.53).abs() < 1e-9);
    }

    #[test]
    fn the_suns_ingresses_and_the_tithis_are_found_in_order_at_the_boundaries() {
        let provider = TestProvider::new();
        let completion = Completion::new(
            &provider,
            OverridePolicy::SdkOnly,
            DeltaTModel::TableThenModel,
        );
        let longitudes = completion.longitudes(Frame::CANONICAL);
        let year = J2000.plus_days(365.25).unwrap();
        let ingresses = Search::new(&longitudes, Quantity::Longitude(Body::Sun), Lattice::SIGNS)
            .between(J2000, year)
            .unwrap();
        assert_eq!(ingresses.len(), 12);
        for pair in ingresses.windows(2) {
            assert!(pair[1].instant.get() > pair[0].instant.get());
            assert!(
                (difference_deg(pair[1].boundary_deg, pair[0].boundary_deg) - 30.0).abs() < 1e-9
            );
        }
        for event in &ingresses {
            let (lon, _) = longitudes
                .longitude_and_speed(Body::Sun, event.instant)
                .unwrap();
            assert!(
                difference_deg(lon, event.boundary_deg).abs() < 1e-6,
                "{lon} {}",
                event.boundary_deg
            );
            assert_eq!(event.direction, Direction::Rising);
            assert_eq!(event.boundary_deg % 30.0, 0.0);
        }
        // A lunation's tithis: thirty, each a rising crossing of a 12° line.
        let tithis = Search::new(&longitudes, Quantity::ELONGATION, Lattice::TITHIS)
            .between(J2000, J2000.plus_days(29.53).unwrap())
            .unwrap();
        assert!((29..=31).contains(&tithis.len()), "{}", tithis.len());
        for event in &tithis {
            let elongation = evaluate(Quantity::ELONGATION, &longitudes, event.instant).unwrap();
            assert!(difference_deg(elongation, event.boundary_deg).abs() < 1e-6);
        }
        // The first one within a window is the first of the list.
        let first = Search::new(&longitudes, Quantity::ELONGATION, Lattice::TITHIS)
            .next_within(J2000, 5.0)
            .unwrap()
            .unwrap();
        assert_eq!(first, tithis[0]);
        // No stations for the test provider's Sun, which never turns.
        assert!(
            stations(&longitudes, Body::Sun, J2000, year, TOLERANCE_DAYS)
                .unwrap()
                .is_empty()
        );
    }

    /// A source that counts which reading each search takes.
    struct Counting<'a, P: EphemerisProvider + ?Sized> {
        inner: &'a FrameLongitudes<'a, P>,
        singles: std::sync::atomic::AtomicU32,
        pairs: std::sync::atomic::AtomicU32,
    }

    impl<P: EphemerisProvider + ?Sized> Longitudes for Counting<'_, P> {
        fn longitude_and_speed(
            &self,
            body: Body,
            ut1: JulianDay<Ut1>,
        ) -> Result<(f64, f64), Error> {
            self.singles
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            self.inner.longitude_and_speed(body, ut1)
        }

        fn longitude_and_speed_pair(
            &self,
            bodies: [Body; 2],
            ut1: JulianDay<Ut1>,
        ) -> Result<[(f64, f64); 2], Error> {
            self.pairs
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            self.inner.longitude_and_speed_pair(bodies, ut1)
        }

        fn describe(&self) -> String {
            self.inner.describe()
        }
    }

    #[test]
    fn a_composite_reads_its_two_bodies_in_one_request() {
        let provider = TestProvider::new();
        let completion = Completion::new(
            &provider,
            OverridePolicy::SdkOnly,
            DeltaTModel::TableThenModel,
        );
        let longitudes = completion.longitudes(Frame::CANONICAL);
        // The pair is the two single readings, bit for bit.
        let [moon, sun] = longitudes
            .longitude_and_speed_pair([Body::Moon, Body::Sun], J2000)
            .unwrap();
        assert_eq!(
            moon,
            longitudes.longitude_and_speed(Body::Moon, J2000).unwrap()
        );
        assert_eq!(
            sun,
            longitudes.longitude_and_speed(Body::Sun, J2000).unwrap()
        );
        // A body the provider does not carry is refused by name, in a pair too.
        let refused = longitudes
            .longitude_and_speed_pair([Body::Moon, Body::Pluto], J2000)
            .unwrap_err();
        assert!(refused.to_string().contains("PLUTO"), "{refused}");
        // The tithi search reads pairs alone; an ingress search singles alone.
        let counting = Counting {
            inner: &longitudes,
            singles: std::sync::atomic::AtomicU32::new(0),
            pairs: std::sync::atomic::AtomicU32::new(0),
        };
        let month = J2000.plus_days(29.53).unwrap();
        let tithis = Search::new(&counting, Quantity::ELONGATION, Lattice::TITHIS)
            .between(J2000, month)
            .unwrap();
        assert_eq!(tithis.len(), 30);
        let pairs = counting.pairs.load(std::sync::atomic::Ordering::Relaxed);
        assert!(pairs > 0 && counting.singles.load(std::sync::atomic::Ordering::Relaxed) == 0);
        let ingresses = Search::new(&counting, Quantity::Longitude(Body::Sun), Lattice::SIGNS)
            .between(J2000, month)
            .unwrap();
        assert_eq!(ingresses.len(), 1);
        assert_eq!(
            counting.pairs.load(std::sync::atomic::Ordering::Relaxed),
            pairs
        );
        assert!(counting.singles.load(std::sync::atomic::Ordering::Relaxed) > 0);
    }

    #[test]
    fn bad_windows_and_lattices_are_refused_by_name() {
        let provider = TestProvider::new();
        let completion = Completion::new(
            &provider,
            OverridePolicy::SdkOnly,
            DeltaTModel::TableThenModel,
        );
        let longitudes = completion.longitudes(Frame::CANONICAL);
        let search = Search::new(&longitudes, Quantity::Longitude(Body::Sun), Lattice::SIGNS);
        let backwards = search
            .between(J2000, JulianDay::literal(J2000.get() - 1.0))
            .unwrap_err();
        assert_eq!(backwards.status, Status::InvalidArg);
        assert_eq!(backwards.field(), Some("to"));
        let bad_step = Search::new(&longitudes, Quantity::Longitude(Body::Sun), Lattice::SIGNS)
            .with_step_days(0.0)
            .between(J2000, JulianDay::literal(J2000.get() + 1.0))
            .unwrap_err();
        assert_eq!(bad_step.field(), Some("step_days"));
        let bad_lattice = Search::new(
            &longitudes,
            Quantity::Longitude(Body::Sun),
            Lattice {
                origin_deg: 0.0,
                step_deg: -1.0,
            },
        )
        .between(J2000, JulianDay::literal(J2000.get() + 1.0))
        .unwrap_err();
        assert_eq!(bad_lattice.field(), Some("lattice"));
        assert_eq!(
            search.next_within(J2000, 0.0).unwrap_err().field(),
            Some("window_days")
        );
    }
}
