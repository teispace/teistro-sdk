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
use crate::solve::{Caps, SolveError, first_zero, refine_known, refine_with_rates};

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

    /// A body's longitude and rate at **many instants at once**, written
    /// into `out` in the order asked.
    ///
    /// The companion of [`Longitudes::longitude_and_speed_pair`], one
    /// axis over. A search's scan knows every instant it will visit
    /// before it visits the first, and an ephemeris call is the
    /// expensive thing in this library, so a source that can answer a
    /// grid in one request does so here; the default reads them one at
    /// a time, as every source did before this existed.
    ///
    /// **The contract an override takes on:** the grid answers what the
    /// walk would have answered, value for value, to the bit. A source
    /// whose grid disagreed with its own scalar reading would move
    /// every instant the SDK publishes and nothing would say so, which
    /// is why `tests/events.rs` holds both against each other.
    ///
    /// `out` is cleared first, and is a buffer rather than a return so
    /// that a search reuses one allocation across its chunks.
    ///
    /// # Errors
    ///
    /// An instant or a body the source cannot answer for.
    fn longitudes_and_speeds(
        &self,
        body: Body,
        ut1: &[JulianDay<Ut1>],
        out: &mut Vec<(f64, f64)>,
    ) -> Result<(), Error> {
        out.clear();
        out.reserve(ut1.len());
        for at in ut1 {
            out.push(self.longitude_and_speed(body, *at)?);
        }
        Ok(())
    }

    /// Two bodies' longitudes and rates at many instants at once: what a
    /// composite quantity's scan needs. As
    /// [`Longitudes::longitudes_and_speeds`], and under the same
    /// contract.
    ///
    /// # Errors
    ///
    /// An instant or a body the source cannot answer for.
    fn longitudes_and_speeds_pair(
        &self,
        bodies: [Body; 2],
        ut1: &[JulianDay<Ut1>],
        out: &mut Vec<[(f64, f64); 2]>,
    ) -> Result<(), Error> {
        out.clear();
        out.reserve(ut1.len());
        for at in ut1 {
            out.push(self.longitude_and_speed_pair(bodies, *at)?);
        }
        Ok(())
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

    // The grids too: a provided method not forwarded here answers a
    // reference's grid an instant at a time, whatever the source can do.
    fn longitudes_and_speeds(
        &self,
        body: Body,
        ut1: &[JulianDay<Ut1>],
        out: &mut Vec<(f64, f64)>,
    ) -> Result<(), Error> {
        (**self).longitudes_and_speeds(body, ut1, out)
    }

    fn longitudes_and_speeds_pair(
        &self,
        bodies: [Body; 2],
        ut1: &[JulianDay<Ut1>],
        out: &mut Vec<[(f64, f64); 2]>,
    ) -> Result<(), Error> {
        (**self).longitudes_and_speeds_pair(bodies, ut1, out)
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
        self.grid(bodies, &[ut1.get()])
    }

    /// One request for the bodies at every instant: the shape the port
    /// was built for, and the reason a scan asks for its samples at
    /// once rather than one round trip at a time.
    fn grid(&self, bodies: &[Body], jds: &[f64]) -> Result<Completed, Error> {
        let mut request = PositionRequest::new(jds, TimeScale::Ut1, bodies, self.frame);
        request.observer = self.observer;
        Ok(self.completion.positions(&request)?)
    }
}

/// The longitude and rate in one completed cell, or the refusal it holds.
fn reading(
    done: &Completed,
    row: usize,
    index: usize,
    body: Body,
    ut1: JulianDay<Ut1>,
) -> Result<(f64, f64), Error> {
    let cell = done
        .columns
        .at(row, index)
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
        reading(&done, 0, 0, body, ut1)
    }

    fn longitude_and_speed_pair(
        &self,
        bodies: [Body; 2],
        ut1: JulianDay<Ut1>,
    ) -> Result<[(f64, f64); 2], Error> {
        let done = self.request(&bodies, ut1)?;
        let [first, second] = bodies;
        Ok([
            reading(&done, 0, 0, first, ut1)?,
            reading(&done, 0, 1, second, ut1)?,
        ])
    }

    fn longitudes_and_speeds(
        &self,
        body: Body,
        ut1: &[JulianDay<Ut1>],
        out: &mut Vec<(f64, f64)>,
    ) -> Result<(), Error> {
        out.clear();
        if ut1.is_empty() {
            return Ok(());
        }
        let jds: Vec<f64> = ut1.iter().map(|at| at.get()).collect();
        let done = self.grid(&[body], &jds)?;
        out.reserve(ut1.len());
        for (row, at) in ut1.iter().enumerate() {
            out.push(reading(&done, row, 0, body, *at)?);
        }
        Ok(())
    }

    fn longitudes_and_speeds_pair(
        &self,
        bodies: [Body; 2],
        ut1: &[JulianDay<Ut1>],
        out: &mut Vec<[(f64, f64); 2]>,
    ) -> Result<(), Error> {
        out.clear();
        if ut1.is_empty() {
            return Ok(());
        }
        let jds: Vec<f64> = ut1.iter().map(|at| at.get()).collect();
        let done = self.grid(&bodies, &jds)?;
        let [first, second] = bodies;
        out.reserve(ut1.len());
        for (row, at) in ut1.iter().enumerate() {
            out.push([
                reading(&done, row, 0, first, *at)?,
                reading(&done, row, 1, second, *at)?,
            ]);
        }
        Ok(())
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

/// The quantity at an instant from a source of longitudes: an angle
/// reduced to `[0, 360)`, or a rate in degrees a day.
///
/// Public because a caller that has already searched a lattice often has
/// to name what lies *between* two boundaries — which tithi runs between
/// two tithi boundaries — and reading the quantity there is how it does
/// so, with the same evaluation the search used.
///
/// # Errors
///
/// The source's own refusal for the instant or the bodies.
pub fn value_of<S: Longitudes + ?Sized>(
    quantity: Quantity,
    source: &S,
    ut1: JulianDay<Ut1>,
) -> Result<f64, Error> {
    Ok(evaluate(quantity, source, ut1)?.0)
}

/// The index of the last anchored sample at or before an instant.
///
/// `floor` rather than `round`, so the window's first bracket always
/// contains its start.
fn grid_index(at: f64, step: f64) -> i64 {
    // An index is a window's days over a step of a few minutes at least,
    // well inside an `i64`, and a whole number after `floor`.
    #[allow(
        clippy::cast_possible_truncation,
        reason = "a grid index is a whole number far inside the range"
    )]
    let index = ((at - SCAN_ANCHOR_JD) / step).floor() as i64;
    index
}

/// The anchored sample at an index: `anchor + k × step`, by
/// multiplication, so the same index is the same instant to the last bit
/// whatever window reached it.
fn grid_instant(index: i64, step: f64) -> f64 {
    // Exact: a grid index is far below 2^53.
    #[allow(
        clippy::cast_precision_loss,
        reason = "a grid index is far below 2^53 and converts exactly"
    )]
    let index = index as f64;
    step.mul_add(index, SCAN_ANCHOR_JD)
}

/// The index of the first grid sample at or after `to`, and not before
/// `first`: where a scan that began at `first` stops.
fn end_index(to: f64, step: f64, first: i64) -> i64 {
    let mut index = grid_index(to, step).max(first);
    // `floor` and `mul_add` round apart by a bit now and then, so the
    // index is settled against the instants it names.
    while grid_instant(index, step) < to {
        index += 1;
    }
    while index > first && grid_instant(index - 1, step) >= to {
        index -= 1;
    }
    index
}

/// The share of a body's shortest run between stations a scan may stride
/// across: under a whole run, so no two stations share a bracket, with a
/// quarter to spare for the frames and the eras the measurement did not
/// span (`station-runs-measured.md`).
const RUN_SHARE: f64 = 0.75;

/// The most a scan's bracket may carry a quantity, degrees, so a bracket
/// unwrapped from its own earlier sample can never be half a circle out.
const QUARTER_CIRCLE_DEG: f64 = 90.0;

/// The shortest a body's longitude runs one way between two stations,
/// days, at or under the least measured over the built-in ephemeris's six
/// centuries (`03-design/station-runs-measured.md`, which holds these
/// against it both ways): Mercury's retrogression at its shortest,
/// Venus's, and so on out to Pluto's.
///
/// Infinite for a body that never turns ([`least_rate`] bounds it), and
/// `None` for one that turns within days or that the table does not
/// know: the true node and the osculating apogee swing back and forth in
/// hours, so their scan keeps the fine step.
#[must_use]
pub fn shortest_run_days(body: Body) -> Option<f64> {
    if least_rate(body).is_some() {
        return Some(f64::INFINITY);
    }
    match body {
        Body::Mercury => Some(19.5),
        Body::Venus => Some(40.5),
        Body::Mars => Some(59.5),
        Body::Jupiter => Some(117.0),
        Body::Saturn => Some(133.5),
        Body::Uranus => Some(148.5),
        Body::Neptune | Body::Pluto => Some(156.0),
        _ => None,
    }
}

/// One sample of a scan: its index on the fine grid, the quantity's raw
/// value there and its rate.
#[derive(Clone, Copy, Debug)]
struct Sample {
    index: i64,
    raw: f64,
    rate: f64,
}

/// How near a line must pass a bracket's end, degrees, to count as inside
/// it when the scan decides where to look: a bracket's arithmetic and a
/// cell's round apart in the last bits, so a line met on a sample is
/// looked for on both sides of it rather than on neither.
const NEAR_LINE_DEG: f64 = 1e-6;

/// A scan in progress: the search, its fine grid, the last cell whose
/// crossings were found, and each lattice's events so far.
struct Scan<'a, 's, S: Longitudes + ?Sized> {
    search: &'a Search<'s, S>,
    step: f64,
    /// The lower index of the last cell refined; cells are refined once,
    /// in time order.
    done: i64,
    /// Scratch for a grid's instants and answers, reused.
    instants: Vec<JulianDay<Ut1>>,
    values: Vec<(f64, f64)>,
    singles: Vec<(f64, f64)>,
    pairs: Vec<[(f64, f64); 2]>,
    events: Vec<Vec<Event>>,
}

impl<S: Longitudes + ?Sized> Scan<'_, '_, S> {
    /// The quantity at several samples of the fine grid, as one grid.
    fn read(&mut self, indices: impl Iterator<Item = i64>) -> Result<Vec<Sample>, Error> {
        let indices: Vec<i64> = indices.collect();
        self.instants.clear();
        self.instants.extend(
            indices
                .iter()
                .map(|index| JulianDay::literal(grid_instant(*index, self.step))),
        );
        if self.instants.is_empty() {
            return Ok(Vec::new());
        }
        evaluate_many(
            self.search.quantity,
            self.search.source,
            &self.instants,
            &mut self.values,
            &mut self.singles,
            &mut self.pairs,
        )?;
        Ok(indices
            .iter()
            .zip(&self.values)
            .map(|(&index, &(raw, rate))| Sample { index, raw, rate })
            .collect())
    }

    /// Whether a crossing or a station may lie between two samples: the
    /// rate changes sign, or a line of some lattice lies between the two
    /// values (within [`NEAR_LINE_DEG`] of either). Between samples closer
    /// than the body's shortest run there is at most one station, so the
    /// same sign at both ends means none, and the curve runs one way.
    fn busy(&self, lo: Sample, hi: Sample) -> bool {
        if (lo.rate > 0.0) != (hi.rate > 0.0) {
            return true;
        }
        let (from_deg, to_deg) = (
            lo.raw,
            hi.raw + turn_deg(hi.raw - lo.raw, self.search.quantity.wraps()),
        );
        let (below, above) = if to_deg >= from_deg {
            (from_deg - NEAR_LINE_DEG, to_deg + NEAR_LINE_DEG)
        } else {
            (to_deg - NEAR_LINE_DEG, from_deg + NEAR_LINE_DEG)
        };
        self.search
            .lattices
            .iter()
            .any(|lattice| !lines_between(lattice, below, above).is_empty())
    }

    /// Every crossing between two samples of the scan, in time order.
    ///
    /// A cell of the fine grid is refined as a fine scan refines it. A
    /// wider bracket costs its two ends unless [`Scan::busy`]; a busy one
    /// the curve runs through one way is [narrowed](Scan::narrow) onto
    /// each line it passes, and one that holds a station is read at every
    /// `√n`th sample as one grid and each part looked into the same way.
    /// Every cell a fine scan would find a crossing in is refined, by the
    /// same arithmetic from the same two samples, so the answers are the
    /// fine scan's to the bit.
    fn bracket(&mut self, lo: Sample, hi: Sample) -> Result<(), Error> {
        let span = hi.index - lo.index;
        if span == 1 {
            return self.cell(lo, hi);
        }
        if !self.busy(lo, hi) {
            return Ok(());
        }
        if (lo.rate > 0.0) == (hi.rate > 0.0) && self.unwrapped(lo, hi) - lo.raw != 0.0 {
            return self.narrow(lo, hi);
        }
        // The square root of a small whole number, to the nearest.
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_precision_loss,
            reason = "a span is a small whole number of steps"
        )]
        let every = ((span as f64).sqrt().round() as i64).max(1);
        let mut coarse = vec![lo];
        coarse.extend(
            self.read(
                (1..)
                    .map(|k| lo.index + k * every)
                    .take_while(|i| *i < hi.index),
            )?,
        );
        coarse.push(hi);
        for pair in coarse.windows(2) {
            if let (Some(&a), Some(&b)) = (pair.first(), pair.get(1)) {
                self.bracket(a, b)?;
            }
        }
        Ok(())
    }

    /// A sample's value unwrapped onto `from`'s turn: a bracket is shorter
    /// than a quarter circle's motion, so the nearest turn is the one.
    fn unwrapped(&self, from: Sample, sample: Sample) -> f64 {
        sample.raw + turn_deg(sample.raw - from.raw, self.search.quantity.wraps())
    }

    /// Every crossing of a bracket the curve runs through one way.
    ///
    /// Each line the bracket passes is narrowed onto: the samples known
    /// strictly below it and strictly above it (by [`NEAR_LINE_DEG`], in
    /// the direction of travel) close in, a round at a time, on a guess
    /// from the values either side — the secant's, or the midpoint's when
    /// the last round did not halve the gap — with the four samples round
    /// the guess read as one grid for every line at once. A line is placed
    /// when every sample between its two is known; the curve being
    /// monotone, no cell outside them can pass it. Then every cell between
    /// a placed line's samples is refined, in time order.
    fn narrow(&mut self, lo: Sample, hi: Sample) -> Result<(), Error> {
        let sense = if self.unwrapped(lo, hi) > lo.raw {
            1.0
        } else {
            -1.0
        };
        let key = |scan: &Self, sample: Sample| sense * scan.unwrapped(lo, sample);
        let (below, above) = (
            lo.raw.min(self.unwrapped(lo, hi)) - NEAR_LINE_DEG,
            lo.raw.max(self.unwrapped(lo, hi)) + NEAR_LINE_DEG,
        );
        let mut lines: Vec<f64> = Vec::new();
        for lattice in &self.search.lattices {
            lines.extend(
                lines_between(lattice, below, above)
                    .into_iter()
                    .map(|index| sense * line_deg(lattice, index)),
            );
        }
        let mut known = vec![lo, hi];
        // The gap each line was narrowed from, twice the first so that the
        // first round guesses by the secant.
        let mut widths = vec![2 * (hi.index - lo.index); lines.len()];
        let mut placed: Vec<(i64, i64)> = Vec::new();
        while !lines.is_empty() {
            let mut wanted: Vec<i64> = Vec::new();
            let (mut open, mut open_widths) = (Vec::new(), Vec::new());
            for (&line, &width) in lines.iter().zip(&widths) {
                let a = known
                    .iter()
                    .rposition(|sample| key(self, *sample) < line - NEAR_LINE_DEG)
                    .unwrap_or(0);
                let b = known
                    .iter()
                    .skip(a + 1)
                    .position(|sample| key(self, *sample) > line + NEAR_LINE_DEG)
                    .map_or(known.len() - 1, |offset| a + 1 + offset);
                let (Some(&first), Some(&last)) = (known.get(a), known.get(b)) else {
                    continue;
                };
                if b - a == usize::try_from(last.index - first.index).unwrap_or(usize::MAX) {
                    placed.push((first.index, last.index));
                    continue;
                }
                let gap = last.index - first.index;
                let halved = gap * 2 <= width;
                let guess = if halved {
                    let (from, to) = (key(self, first), key(self, last));
                    // A share of a gap of a few hundred steps.
                    #[allow(
                        clippy::cast_possible_truncation,
                        clippy::cast_precision_loss,
                        reason = "a gap is a small whole number of steps"
                    )]
                    let ahead = (((line - from) / (to - from)) * gap as f64).floor() as i64;
                    first.index + ahead
                } else {
                    first.index + gap / 2
                };
                let inside = |index: &i64| *index > first.index && *index < last.index;
                let unknown = |index: &i64| known.iter().all(|sample| sample.index != *index);
                let mut round: Vec<i64> = (guess..=guess + 1)
                    .filter(|index| inside(index) && unknown(index))
                    .collect();
                if round.is_empty() {
                    round = (first.index + 1..last.index)
                        .filter(|index| unknown(index))
                        .collect();
                }
                for index in round {
                    if let Err(at) = wanted.binary_search(&index) {
                        wanted.insert(at, index);
                    }
                }
                open.push(line);
                open_widths.push(gap);
            }
            for sample in self.read(wanted.into_iter())? {
                let at = known.partition_point(|earlier| earlier.index < sample.index);
                known.insert(at, sample);
            }
            (lines, widths) = (open, open_widths);
        }
        for pair in known.windows(2) {
            if let (Some(&a), Some(&b)) = (pair.first(), pair.get(1))
                && b.index - a.index == 1
                && placed
                    .iter()
                    .any(|(from, to)| a.index >= *from && b.index <= *to)
            {
                self.cell(a, b)?;
            }
        }
        Ok(())
    }

    /// One cell of the fine grid, exactly as a fine scan refines it: the
    /// curve unwrapped across the cell alone, from its own earlier sample,
    /// and every line of every lattice it passes refined. A sum carried
    /// from the window's first sample would round differently for every
    /// start, and the same cell would refine to different bits in two
    /// windows that both hold it.
    fn cell(&mut self, lo: Sample, hi: Sample) -> Result<(), Error> {
        if lo.index <= self.done {
            return Ok(());
        }
        self.done = lo.index;
        let search = self.search;
        let (t_lo, t_hi) = (
            grid_instant(lo.index, self.step),
            grid_instant(hi.index, self.step),
        );
        let (unwrapped_lo, unwrapped_hi) = (
            lo.raw,
            hi.raw + turn_deg(hi.raw - lo.raw, search.quantity.wraps()),
        );
        if unwrapped_hi - unwrapped_lo != 0.0 {
            for (lattice, found) in search.lattices.iter().zip(self.events.iter_mut()) {
                search.cross(
                    lattice,
                    (t_lo, lo.raw, unwrapped_lo, lo.rate),
                    (t_hi, unwrapped_hi, hi.rate),
                    found,
                )?;
            }
        }
        Ok(())
    }
}

/// The quantity at every instant, in the order asked: [`evaluate`] over
/// a grid, written into `out`.
///
/// One request where the walk made one per instant. The values are the
/// walk's own — the same source, the same instants, combined the same
/// way — so a scan that reads them here answers exactly what a scan
/// that read them one at a time answered.
fn evaluate_many<S: Longitudes + ?Sized>(
    quantity: Quantity,
    source: &S,
    ut1: &[JulianDay<Ut1>],
    out: &mut Vec<(f64, f64)>,
    singles: &mut Vec<(f64, f64)>,
    pairs: &mut Vec<[(f64, f64); 2]>,
) -> Result<(), Error> {
    out.clear();
    out.reserve(ut1.len());
    match quantity {
        Quantity::Longitude(body) => {
            source.longitudes_and_speeds(body, ut1, singles)?;
            out.extend_from_slice(singles);
        }
        Quantity::Speed(body) => {
            source.longitudes_and_speeds(body, ut1, singles)?;
            out.extend(singles.iter().map(|(_, speed)| (*speed, NO_RATE)));
        }
        Quantity::Composite {
            a,
            first,
            b,
            second,
        } => {
            source.longitudes_and_speeds_pair([first, second], ut1, pairs)?;
            out.extend(pairs.iter().map(|&[x, y]| composite(a, x, b, y)));
        }
    }
    if out.len() != ut1.len() {
        return Err(Error::new(
            Status::Provider,
            format!(
                "a source answered {} of {} instants asked for as a grid",
                out.len(),
                ut1.len()
            ),
        ));
    }
    Ok(())
}

/// The rate a quantity reads with when its source gives none: a speed's
/// own rate is an acceleration, which no source answers.
const NO_RATE: f64 = f64::NAN;

/// A composite angle and its rate from its two bodies' readings.
fn composite(a: f64, (x, dx): (f64, f64), b: f64, (y, dy): (f64, f64)) -> (f64, f64) {
    (normalise_deg(a * x + b * y), a * dx + b * dy)
}

/// The quantity at an instant, as the search reads it, with its rate in
/// degrees a day ([`NO_RATE`] for a speed).
fn evaluate<S: Longitudes + ?Sized>(
    quantity: Quantity,
    source: &S,
    ut1: JulianDay<Ut1>,
) -> Result<(f64, f64), Error> {
    Ok(match quantity {
        Quantity::Longitude(body) => source.longitude_and_speed(body, ut1)?,
        Quantity::Speed(body) => (source.longitude_and_speed(body, ut1)?.1, NO_RATE),
        Quantity::Composite {
            a,
            first,
            b,
            second,
        } => {
            let [x, y] = source.longitude_and_speed_pair([first, second], ut1)?;
            composite(a, x, b, y)
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

/// The least speed of longitude a body ever has, degrees a day, with a
/// margin the other way from [`greatest_rate`]'s: the Sun at aphelion,
/// the Moon at apogee, the mean node's and the mean apogee's steady
/// drift.
///
/// `None` for a body whose longitude can **turn**. A retrograding body
/// passes through zero speed and may cross a line and come back, so how
/// long it can stand between two lines has no bound a table can give;
/// a caller that needs one for such a body has to say what it will
/// accept. The mean node is here although it moves backwards: it never
/// turns, so its dwell is bounded like any other steady motion.
#[must_use]
pub fn least_rate(body: Body) -> Option<f64> {
    match body {
        Body::Sun => Some(0.95),
        Body::Moon => Some(11.7),
        Body::MeanNode => Some(0.052),
        Body::MeanApogee => Some(0.110),
        // Every planet stations, the true node and the osculating
        // apogee swing, and a body the port adds later is unknown.
        _ => None,
    }
}

/// The least rate a quantity advances at, degrees a day, or `None` when
/// it can stand still or turn.
///
/// A composite is bounded term by term: a term with a positive
/// coefficient is slowest at its body's least rate, and one with a
/// negative coefficient is most negative at its body's *greatest*, which
/// is a bound whether that body turns or not. The tithi's elongation is
/// the case that matters — the Moon's least less the Sun's greatest, ten
/// and a half degrees a day.
#[must_use]
pub fn quantity_least_rate(quantity: Quantity) -> Option<f64> {
    match quantity {
        Quantity::Longitude(body) => least_rate(body),
        // A speed's own rate of change is not in this table.
        Quantity::Speed(_) => None,
        Quantity::Composite {
            a,
            first,
            b,
            second,
        } => {
            let slowest = |coefficient: f64, body: Body| -> Option<f64> {
                if coefficient > 0.0 {
                    least_rate(body).map(|rate| coefficient * rate)
                } else {
                    Some(coefficient * greatest_rate(body))
                }
            };
            let total = slowest(a, first)? + slowest(b, second)?;
            (total > 0.0).then_some(total)
        }
    }
}

/// The longest a quantity can stand between two lines of a lattice,
/// days: the lattice's spacing at the quantity's least rate.
///
/// The companion of the rule that sizes a search's *step*. The step is
/// sized by how fast the quantity can move, so that no line is passed
/// twice between two samples; this is sized by how slowly it can move,
/// so that a caller searching `[from - dwell, to + dwell]` sees every
/// span touching `[from, to]` with both its own bounds rather than the
/// window's.
///
/// `None` when the quantity can stand still or turn
/// ([`quantity_least_rate`]); such a caller chooses a reach and lives
/// with the truncation past it.
#[must_use]
pub fn longest_dwell_days(quantity: Quantity, lattice: &Lattice) -> Option<f64> {
    quantity_least_rate(quantity).map(|rate| spacing_deg(lattice) / rate)
}

/// Which way a station turns.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StationKind {
    /// Direct motion ends: the speed passes from positive to negative.
    Retrograde,
    /// Retrograde motion ends: the speed passes from negative to positive.
    Direct,
}

/// A station: the instant a body's longitude stands still.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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

/// The epoch a scan's samples are aligned to: J2000.0.
///
/// A scan used to step from the caller's own `from`, so where its
/// samples fell — and therefore which bracket the refinement was handed
/// — depended on where the question started. Two callers asking about
/// the same crossing from different windows got answers up to two
/// milliseconds apart, and a crossing's instant was a property of the
/// question as much as of the sky.
///
/// Aligning every sample to one epoch makes a narrower window's samples
/// a **subset** of a wider one's, so any two windows that both contain a
/// crossing bracket it identically and refine it to the same bits. That
/// is worth having on its own, and it is what lets a range search once
/// and slice the answer per day (`07-roadmap/02-plan-performance-and-passthrough.md`,
/// A1c) without the slices disagreeing with the searches they replace.
///
/// Samples are `anchor + k × step` for a whole `k`, computed by
/// multiplication rather than by accumulating steps, so the same `k`
/// gives the same instant to the last bit however the scan reached it.
pub const SCAN_ANCHOR_JD: f64 = 2_451_545.0;

/// How many of a scan's instants are asked for in one grid by default.
///
/// The scan visits every instant between its ends, so every sample it
/// asks for is one it needs and a chunk wastes nothing; the cap is
/// there for memory alone, since a search over a millennium at a
/// half-day step is three quarters of a million instants. Five hundred
/// and twelve holds a two-year ingress search or a century of solar
/// sign changes in one request, and keeps the buffer under thirty
/// kilobytes.
pub const SCAN_CHUNK: usize = 512;

/// A search for the crossings of a quantity over a lattice, or over
/// several at once from one scan.
pub struct Search<'s, S: Longitudes + ?Sized> {
    source: &'s S,
    quantity: Quantity,
    /// Never empty: every constructor gives at least one.
    lattices: Vec<Lattice>,
    tolerance_days: f64,
    step_days: Option<f64>,
    caps: Caps,
    chunk: usize,
}

impl<S: Longitudes + ?Sized> fmt::Debug for Search<'_, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Search")
            .field("source", &self.source.describe())
            .field("quantity", &self.quantity)
            .field("lattices", &self.lattices)
            .field("tolerance_days", &self.tolerance_days)
            .field("step_days", &self.step_days())
            .field("caps", &self.caps)
            .finish()
    }
}

impl<'s, S: Longitudes + ?Sized> Search<'s, S> {
    /// A search with the default tolerance, the step rule and caps.
    #[must_use]
    pub fn new(source: &'s S, quantity: Quantity, lattice: Lattice) -> Search<'s, S> {
        Search::each(source, quantity, lattice, [])
    }

    /// A search over several lattices of one quantity, **scanned once**:
    /// each sample is tested against every lattice, so a caller asking
    /// after a body's signs, its nakshatras and its aspects to ten natal
    /// points pays for one walk over the window and not twelve. The step
    /// is the finest any of them needs, and only the refinement of a
    /// crossing found is paid per lattice.
    ///
    /// [`Search::between_each`] answers each lattice's crossings apart;
    /// [`Search::between`], all of them in time order.
    #[must_use]
    pub fn each(
        source: &'s S,
        quantity: Quantity,
        first: Lattice,
        more: impl IntoIterator<Item = Lattice>,
    ) -> Search<'s, S> {
        Search {
            source,
            quantity,
            lattices: std::iter::once(first).chain(more).collect(),
            tolerance_days: TOLERANCE_DAYS,
            step_days: None,
            caps: Caps::DEFAULT,
            chunk: SCAN_CHUNK,
        }
    }

    /// How many of the scan's instants to ask for in one grid.
    ///
    /// The default is [`SCAN_CHUNK`]. One is the walk this replaced —
    /// every instant its own round trip — which is what the grid is
    /// held against in the tests. Zero is read as one.
    #[must_use]
    pub const fn with_chunk(mut self, instants: usize) -> Self {
        self.chunk = if instants == 0 { 1 } else { instants };
        self
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

    /// The step the search samples at: half the finest lattice's spacing
    /// at the quantity's greatest rate, never more than a day, so no line
    /// is passed twice between two samples and no retrograde arc is
    /// stepped over.
    #[must_use]
    pub fn step_days(&self) -> f64 {
        self.step_days.unwrap_or_else(|| {
            let rate = quantity_rate_deg_per_day(self.quantity);
            let finest = self
                .lattices
                .iter()
                .map(spacing_deg)
                .fold(f64::INFINITY, f64::min);
            (finest / rate * 0.5).min(STEP_CAP_DAYS)
        })
    }

    /// The longest a value can stand between two lattice lines, days:
    /// the lattice's spacing at the quantity's least rate.
    ///
    /// This is [`Search::step_days`]'s companion, and it answers the
    /// other question a caller has about a window. The step is sized by
    /// how *fast* the quantity can move, so that no line is passed twice
    /// between two samples; the reach is sized by how *slowly* it can
    /// move, so that a span already running when the window opens is
    /// found rather than truncated. A caller that searches `[from -
    /// reach, to + reach]` sees every span that touches `[from, to]`
    /// with both its own bounds.
    ///
    /// `None` when the quantity can stand still or turn, whose dwell no
    /// table bounds ([`quantity_least_rate`]); such a caller chooses a
    /// reach and lives with the truncation past it.
    #[must_use]
    pub fn longest_dwell_days(&self) -> Option<f64> {
        // The longest of any one lattice's, since a caller widens its
        // window by the most any of them can need.
        self.lattices
            .iter()
            .map(|lattice| longest_dwell_days(self.quantity, lattice))
            .try_fold(0.0_f64, |longest, dwell| dwell.map(|d| longest.max(d)))
    }

    fn check(&self) -> Result<(), Error> {
        if self.lattices.iter().any(|lattice| {
            !(lattice.origin_deg.is_finite() && lattice.step_deg.is_finite())
                || lattice.step_deg < 0.0
        }) {
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

    /// Every crossing between two instants, of every lattice, in time
    /// order.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` for an empty or reversed window or a bad tolerance,
    /// step or lattice; the source's error; `NOT_CONVERGED` should a
    /// bracket fail to narrow.
    pub fn between(&self, from: JulianDay<Ut1>, to: JulianDay<Ut1>) -> Result<Vec<Event>, Error> {
        let mut each = self.between_each(from, to)?;
        if each.len() == 1 {
            return Ok(each.pop().unwrap_or_default());
        }
        let mut all: Vec<Event> = each.into_iter().flatten().collect();
        all.sort_by(|a, b| a.instant.get().total_cmp(&b.instant.get()));
        Ok(all)
    }

    /// Every crossing between two instants, lattice by lattice in the
    /// order they were given, each in time order, from one scan.
    ///
    /// # Errors
    ///
    /// As [`Search::between`].
    pub fn between_each(
        &self,
        from: JulianDay<Ut1>,
        to: JulianDay<Ut1>,
    ) -> Result<Vec<Vec<Event>>, Error> {
        self.check()?;
        check_window("search", from, to)?;
        // The scan is compiled once, over any source, rather than once for
        // every source type a caller searches: it is the bulk of this
        // module and a wasm module pays for each copy, while the one
        // indirect call it adds is per request and not per instant.
        let source: &dyn Longitudes = &self.source;
        Search {
            source,
            quantity: self.quantity,
            lattices: self.lattices.clone(),
            tolerance_days: self.tolerance_days,
            step_days: self.step_days,
            caps: self.caps,
            chunk: self.chunk,
        }
        .scan(from, to)
    }
}

impl Search<'_, dyn Longitudes + '_> {
    /// [`Search::between_each`] over a checked window.
    fn scan(&self, from: JulianDay<Ut1>, to: JulianDay<Ut1>) -> Result<Vec<Vec<Event>>, Error> {
        let step = self.step_days();
        let stride = self.stride();
        // The samples are the anchored grid's, from the last one at or
        // before the window to the first one at or after it, so that every
        // cell is a whole step and two windows that both hold a crossing
        // hand the refinement the same one. Each instant is computed from
        // its own index rather than by adding a step to the last, because
        // an accumulated sum depends on where it started and a product
        // does not. A slow body is scanned every `stride` samples of that
        // grid and the grid is read only where a crossing or a station is
        // (§4, "a slow body's scan"); the window's two ends stay the fine
        // grid's, so the scan asks for nothing further outside the window
        // than a fine scan would.
        let first = grid_index(from.get(), step);
        let last = end_index(to.get(), step, first);
        let mut scan = Scan {
            search: self,
            step,
            done: first - 1,
            instants: Vec::new(),
            values: Vec::new(),
            singles: Vec::new(),
            pairs: Vec::new(),
            events: vec![Vec::new(); self.lattices.len()],
        };

        // The scan visits every sample between the ends, and knows all of
        // them before it asks for the first, so it asks for them a grid at
        // a time. Only the refinement below stays serial: each of its
        // steps is chosen from the answer to the last, so there is nothing
        // to ask for in advance.
        let mut indices: Vec<i64> = Vec::with_capacity(self.chunk);
        let mut instants: Vec<JulianDay<Ut1>> = Vec::with_capacity(self.chunk);
        let mut values: Vec<(f64, f64)> = Vec::with_capacity(self.chunk);
        let (mut singles, mut pairs) = (Vec::new(), Vec::new());
        let mut previous: Option<Sample> = None;
        let mut index = first;
        let mut samples = 0u32;
        let mut done = false;
        while !done {
            indices.clear();
            instants.clear();
            while indices.len() < self.chunk {
                indices.push(index);
                instants.push(JulianDay::literal(grid_instant(index, step)));
                if index >= last {
                    done = true;
                    break;
                }
                if samples >= SAMPLE_CAP {
                    return Err(Error::new(
                        Status::NotConverged,
                        format!("the crossing search took more than {SAMPLE_CAP} samples"),
                    ));
                }
                samples += 1;
                index = ((index.div_euclid(stride) + 1) * stride).min(last);
            }
            evaluate_many(
                self.quantity,
                self.source,
                &instants,
                &mut values,
                &mut singles,
                &mut pairs,
            )?;
            for (&index, (raw, rate)) in indices.iter().zip(values.iter().copied()) {
                let sample = Sample { index, raw, rate };
                if let Some(earlier) = previous {
                    scan.bracket(earlier, sample)?;
                }
                previous = Some(sample);
            }
        }
        let mut events = scan.events;
        // A cell may reach outside the window at either end, since the
        // ends are grid points rather than the caller's own instants. A
        // crossing found out there is a real crossing and not this
        // window's, so it is dropped rather than reported.
        for found in &mut events {
            found.retain(|event| {
                event.instant.get() >= from.get() && event.instant.get() <= to.get()
            });
        }
        Ok(events)
    }
}

impl<S: Longitudes + ?Sized> Search<'_, S> {
    /// How many steps of the fine grid the scan strides between samples:
    /// one, unless the quantity is a body's longitude whose shortest run
    /// between stations is known, and no step was chosen by hand.
    ///
    /// Between two samples less than the body's shortest run apart there
    /// is at most one station, so the rates at the two ends say whether
    /// there is one; and away from a station the longitude is monotone,
    /// so the lines between the ends are every line crossed, once each.
    /// The stride is held under three quarters of the shortest run, and
    /// under the finest lattice's spacing (or a quarter circle) at the
    /// greatest rate, so a bracket passes a line or so and never half a
    /// circle.
    fn stride(&self) -> i64 {
        let Quantity::Longitude(body) = self.quantity else {
            return 1;
        };
        let (None, Some(run)) = (self.step_days, shortest_run_days(body)) else {
            return 1;
        };
        let finest = self
            .lattices
            .iter()
            .map(spacing_deg)
            .fold(QUARTER_CIRCLE_DEG, f64::min);
        let coarse = (run * RUN_SHARE).min(finest / greatest_rate(body));
        let steps = (coarse / self.step_days()).floor();
        // A stride is a small whole number of steps: a body's run over a
        // fraction of a day at most, or a quarter circle at its rate.
        #[allow(
            clippy::cast_possible_truncation,
            reason = "a stride is a small whole number of steps"
        )]
        let steps = steps as i64;
        steps.max(1)
    }

    /// The step the scan samples at, days: [`Search::step_days`] times
    /// the stride a slow body's shortest run allows. A crossing is still
    /// refined on the fine step's grid, which is why the two are apart.
    #[must_use]
    pub fn scan_step_days(&self) -> f64 {
        // A stride is a small whole number.
        #[allow(
            clippy::cast_precision_loss,
            reason = "a stride is a small whole number of steps"
        )]
        let stride = self.stride() as f64;
        self.step_days() * stride
    }

    /// Every line of one lattice the curve passed between two samples,
    /// refined and pushed onto `found`: `lo` is the earlier sample's
    /// instant, raw value, unwrapped value and rate, `hi` the later's
    /// instant, unwrapped value and rate.
    fn cross(
        &self,
        lattice: &Lattice,
        (t_lo, raw_lo, unwrapped_lo, rate_lo): (f64, f64, f64, f64),
        (t_hi, unwrapped_hi, rate_hi): (f64, f64, f64),
        found: &mut Vec<Event>,
    ) -> Result<(), Error> {
        let wraps = self.quantity.wraps();
        let rising = unwrapped_hi > unwrapped_lo;
        let signed = |distance: f64| if rising { distance } else { -distance };
        for k in lines_between(lattice, unwrapped_lo, unwrapped_hi) {
            let line = line_deg(lattice, k);
            // The signed distance to the line along the unwrapped curve,
            // negative before it, so the bracket has the shape the solver
            // expects, and its rate signed the same way. The curve is
            // unwrapped exactly as the samples were, so the bracket's ends
            // carry the values the lattice test saw and a line met at a
            // sample still brackets.
            let gap = |t: f64| -> Result<(f64, f64), Error> {
                let (value, rate) = evaluate(self.quantity, self.source, JulianDay::literal(t))?;
                let advance = if wraps {
                    difference_deg(value, raw_lo)
                } else {
                    value - raw_lo
                };
                Ok((signed(unwrapped_lo + advance - line), signed(rate)))
            };
            // The ends are the two samples, whose distances and rates the
            // scan already holds, so neither is asked for again. A speed
            // comes with no rate of its own, so its line is narrowed
            // without one.
            let lo = (t_lo, signed(unwrapped_lo - line), signed(rate_lo));
            let hi = (t_hi, signed(unwrapped_hi - line), signed(rate_hi));
            let refined = if matches!(self.quantity, Quantity::Speed(_)) {
                refine_known(
                    |t| gap(t).map(|(value, _)| value),
                    (lo.0, lo.1),
                    (hi.0, hi.1),
                    self.tolerance_days,
                    self.caps,
                )
            } else {
                refine_with_rates(gap, lo, hi, self.tolerance_days, self.caps)
            }
            .map_err(solve_error)?;
            found.push(Event {
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
        Ok(())
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

/// The whole circle to add to a later sample so it follows an earlier one
/// `step_deg` behind it (raw later less raw earlier) by less than half a
/// circle: none for a quantity that does not wrap.
fn turn_deg(step_deg: f64, wraps: bool) -> f64 {
    if !wraps {
        0.0
    } else if step_deg > 180.0 {
        -360.0
    } else if step_deg <= -180.0 {
        360.0
    } else {
        0.0
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
        clippy::expect_used,
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

    /// A crossing is the same bits in every window that holds it, however
    /// far before it the window opens: the samples are the anchored grid's,
    /// and each bracket is unwrapped from its own sample rather than by a
    /// sum carried from the window's first.
    #[test]
    fn a_crossing_is_the_same_bits_in_every_window_that_holds_it() {
        let provider = TestProvider::new();
        let completion = Completion::new(
            &provider,
            OverridePolicy::SdkOnly,
            DeltaTModel::TableThenModel,
        );
        let longitudes = completion.longitudes(Frame::CANONICAL);
        let end = J2000.plus_days(400.0).unwrap();
        let moon = Search::new(
            &longitudes,
            Quantity::Longitude(Body::Moon),
            Lattice::NAKSHATRAS,
        );
        let reference = moon.between(J2000, end).unwrap();
        for earlier in [1.0, 37.5, 1_000.25, 12_345.0] {
            let wider = moon
                .between(J2000.plus_days(-earlier).unwrap(), end)
                .unwrap();
            let held: Vec<_> = wider
                .iter()
                .filter(|event| event.instant.get() >= J2000.get())
                .map(|event| (event.instant.get().to_bits(), event.boundary_deg.to_bits()))
                .collect();
            let alone: Vec<_> = reference
                .iter()
                .map(|event| (event.instant.get().to_bits(), event.boundary_deg.to_bits()))
                .collect();
            assert_eq!(held, alone, "a window opening {earlier} days earlier");
        }
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
            let elongation = value_of(Quantity::ELONGATION, &longitudes, event.instant).unwrap();
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

    #[test]
    fn a_body_that_never_turns_has_a_least_rate_and_one_that_can_does_not() {
        assert!(super::least_rate(Body::Sun).is_some());
        assert!(super::least_rate(Body::Moon).is_some());
        assert!(
            super::least_rate(Body::MeanNode).is_some(),
            "it moves backwards, but it never turns"
        );
        for turns in [Body::Mercury, Body::Mars, Body::Pluto, Body::TrueNode] {
            assert!(super::least_rate(turns).is_none(), "{turns:?}");
        }
    }

    #[test]
    fn the_least_rate_is_under_the_greatest_for_every_body_that_has_one() {
        for body in [Body::Sun, Body::Moon, Body::MeanNode, Body::MeanApogee] {
            let least = super::least_rate(body).expect("a body that never turns");
            assert!(
                least > 0.0 && least < super::greatest_rate(body),
                "{body:?}: {least}"
            );
        }
    }

    #[test]
    fn a_sign_takes_the_sun_a_month_and_the_moon_two_and_a_half_days() {
        let sun = super::longest_dwell_days(Quantity::Longitude(Body::Sun), &Lattice::SIGNS)
            .expect("the Sun never turns");
        let moon = super::longest_dwell_days(Quantity::Longitude(Body::Moon), &Lattice::SIGNS)
            .expect("the Moon never turns");
        assert!((31.0..32.0).contains(&sun), "{sun}");
        assert!((2.5..2.7).contains(&moon), "{moon}");
        assert!(
            super::longest_dwell_days(Quantity::Longitude(Body::Mars), &Lattice::SIGNS).is_none(),
            "Mars stations, so nothing bounds its dwell"
        );
    }

    #[test]
    fn the_elongations_least_rate_is_the_moons_less_the_suns() {
        // The tithi's quantity: twelve degrees of the Moon less the Sun.
        // Its slowest is the Moon at apogee against the Sun at perihelion,
        // and the panchanga's own constant for the longest a limb runs
        // (a day and a half) has to cover the dwell that follows from it.
        let elongation = Quantity::Composite {
            a: 1.0,
            first: Body::Moon,
            b: -1.0,
            second: Body::Sun,
        };
        let rate = super::quantity_least_rate(elongation).expect("neither turns");
        assert!((10.0..11.0).contains(&rate), "{rate}");
        let dwell = super::longest_dwell_days(elongation, &Lattice::TITHIS).expect("neither turns");
        assert!(
            dwell < 1.5,
            "a day and a half covers the slowest tithi: {dwell}"
        );
    }

    #[test]
    fn a_composite_with_a_body_that_turns_has_no_least_rate() {
        assert!(
            super::quantity_least_rate(Quantity::Composite {
                a: 1.0,
                first: Body::Mars,
                b: -1.0,
                second: Body::Sun,
            })
            .is_none(),
            "the term with the positive coefficient can stand still"
        );
    }
}
