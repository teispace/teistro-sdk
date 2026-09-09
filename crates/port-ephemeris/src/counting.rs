//! What the SDK asked a provider for: a wrapper that counts the calls
//! and the cells behind them.
//!
//! An ephemeris call is the expensive thing in this library — a file
//! read, a series evaluation, a lock — and the port's shape says a
//! caller should ask for a **grid** rather than a cell at a time
//! (`docs/03-design/ephemeris-port-and-adapters.md`, §3). Whether the
//! SDK's own layers keep to that is a countable question, and this is
//! what counts it: wrap any provider, do the work, read the tally.
//!
//! It is public because an adapter author wants the same answer. An
//! ephemeris that costs a millisecond a call is fine at one call a chart
//! and unusable at eight hundred, and this says which it is being asked
//! for without a profiler.
//!
//! The counters are atomic, so the wrapper is `Send + Sync` like the
//! provider it wraps and a batch counted across threads still adds up.
//! Counting costs one relaxed atomic add per call, which is nothing
//! beside the call.

use core::sync::atomic::{AtomicU64, Ordering};
use std::collections::BTreeSet;
use std::sync::Mutex;

use teistro_core::catalogue::Ayanamsha;
use teistro_core::quantity::{JulianDay, Ut1};

use crate::body::TimeScale;
use crate::capabilities::{Capabilities, Obliquity};
use crate::columns::PositionColumns;
use crate::crossing::{CrossingRequest, Event};
use crate::error::ProviderError;
use crate::horizon::HorizonRequest;
use crate::provider::{EphemerisProvider, PositionRequest};

/// What a provider was asked for, and how much of it.
///
/// `cells` is what makes the difference between a batch and a loop
/// visible: a hundred charts asking for one grid of a hundred instants
/// and a hundred charts asking one at a time both report a hundred times
/// the cells, and only the first reports one call.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProviderCalls {
    /// Calls to `positions`.
    pub positions: u64,
    /// Cells those calls asked for: instants times bodies, summed.
    pub cells: u64,
    /// The largest single call, in cells; one means the caller never
    /// batched anything.
    pub widest: u64,
    /// Calls to the obliquity override.
    pub obliquity: u64,
    /// Calls to the Delta T override.
    pub delta_t: u64,
    /// Calls to the ayanamsha override.
    pub ayanamsha: u64,
    /// Calls to the UT1 offset override.
    pub dut1: u64,
    /// Calls to the rise, set and transit override.
    pub horizon: u64,
    /// Calls to the crossing override.
    pub crossings: u64,
    /// Distinct cells asked for — one instant, one body, one frame —
    /// counted only when [`CountingProvider::watching_repeats`] built the
    /// wrapper. `cells` less this is what a cache would have answered.
    pub distinct_cells: u64,
}

impl ProviderCalls {
    /// Every call, of whatever kind.
    #[must_use]
    pub const fn total(&self) -> u64 {
        self.positions
            + self.obliquity
            + self.delta_t
            + self.ayanamsha
            + self.dut1
            + self.horizon
            + self.crossings
    }

    /// The share of the cells that were asked for more than once, which
    /// is what a memo inside one call would save. Nought when repeats
    /// were not watched.
    #[must_use]
    pub fn repeat_share(&self) -> f64 {
        if self.cells == 0 || self.distinct_cells == 0 {
            return 0.0;
        }
        #[allow(
            clippy::cast_precision_loss,
            reason = "a ratio of counts, for reporting"
        )]
        {
            1.0 - (self.distinct_cells as f64 / self.cells as f64)
        }
    }

    /// The mean width of a `positions` call, in cells: one when every
    /// call asked for a single cell, and the batch size when the caller
    /// asked once for the lot.
    #[must_use]
    pub fn mean_width(&self) -> f64 {
        if self.positions == 0 {
            return 0.0;
        }
        #[allow(
            clippy::cast_precision_loss,
            reason = "a ratio of counts, for reporting"
        )]
        {
            self.cells as f64 / self.positions as f64
        }
    }
}

/// A provider that counts what it is asked for and answers with the one
/// it wraps.
///
/// ```
/// use teistro_port_ephemeris::{Body, CountingProvider, EphemerisProvider, Frame, PositionRequest, TestProvider, TimeScale};
///
/// let counted = CountingProvider::new(TestProvider::new());
/// let jds = [2_451_545.0, 2_451_546.0];
/// let request = PositionRequest::new(&jds, TimeScale::Ut1, &[Body::Sun, Body::Moon], Frame::CANONICAL);
/// counted.positions(&request).expect("the canonical frame");
/// let calls = counted.calls();
/// assert_eq!((calls.positions, calls.cells, calls.widest), (1, 4, 4));
/// // Four cells in one call: the caller asked for a grid.
/// assert!((calls.mean_width() - 4.0).abs() < 1e-12);
/// ```
#[derive(Debug, Default)]
pub struct CountingProvider<P> {
    inner: P,
    positions: AtomicU64,
    cells: AtomicU64,
    widest: AtomicU64,
    obliquity: AtomicU64,
    delta_t: AtomicU64,
    ayanamsha: AtomicU64,
    dut1: AtomicU64,
    horizon: AtomicU64,
    crossings: AtomicU64,
    /// The cells seen, when repeats are watched. A set behind a lock,
    /// because this is a measurement and not a hot path; `new` leaves it
    /// empty and never touches it.
    seen: Option<Mutex<BTreeSet<(u64, u16, u32)>>>,
}

impl<P> CountingProvider<P> {
    /// Wraps a provider with its counters at nought.
    pub const fn new(inner: P) -> CountingProvider<P> {
        CountingProvider {
            inner,
            positions: AtomicU64::new(0),
            cells: AtomicU64::new(0),
            widest: AtomicU64::new(0),
            obliquity: AtomicU64::new(0),
            delta_t: AtomicU64::new(0),
            ayanamsha: AtomicU64::new(0),
            dut1: AtomicU64::new(0),
            horizon: AtomicU64::new(0),
            crossings: AtomicU64::new(0),
            seen: None,
        }
    }

    /// The same wrapper, remembering which cells it has been asked for,
    /// so [`ProviderCalls::distinct_cells`] says how much of the work was
    /// asked for twice. Costs a lock and a set insertion per cell, which
    /// is why it is not the default.
    #[must_use]
    pub fn watching_repeats(self) -> CountingProvider<P> {
        CountingProvider {
            seen: Some(Mutex::new(BTreeSet::new())),
            ..self
        }
    }

    /// The provider underneath.
    pub const fn inner(&self) -> &P {
        &self.inner
    }

    /// The tally so far.
    pub fn calls(&self) -> ProviderCalls {
        let read = |counter: &AtomicU64| counter.load(Ordering::Relaxed);
        ProviderCalls {
            positions: read(&self.positions),
            cells: read(&self.cells),
            widest: read(&self.widest),
            obliquity: read(&self.obliquity),
            delta_t: read(&self.delta_t),
            ayanamsha: read(&self.ayanamsha),
            dut1: read(&self.dut1),
            horizon: read(&self.horizon),
            crossings: read(&self.crossings),
            distinct_cells: self
                .seen
                .as_ref()
                .map_or(0, |seen| seen.lock().map_or(0, |seen| seen.len() as u64)),
        }
    }

    /// Sets every counter back to nought, so one wrapper can measure one
    /// operation after another.
    pub fn reset(&self) {
        if let Some(seen) = self.seen.as_ref() {
            if let Ok(mut seen) = seen.lock() {
                seen.clear();
            }
        }
        for counter in [
            &self.positions,
            &self.cells,
            &self.widest,
            &self.obliquity,
            &self.delta_t,
            &self.ayanamsha,
            &self.dut1,
            &self.horizon,
            &self.crossings,
        ] {
            counter.store(0, Ordering::Relaxed);
        }
    }
}

impl<P: EphemerisProvider> EphemerisProvider for CountingProvider<P> {
    fn capabilities(&self) -> Capabilities {
        self.inner.capabilities()
    }

    fn positions(&self, request: &PositionRequest<'_>) -> Result<PositionColumns, ProviderError> {
        self.positions.fetch_add(1, Ordering::Relaxed);
        let cells = request.cell_count() as u64;
        self.cells.fetch_add(cells, Ordering::Relaxed);
        self.widest.fetch_max(cells, Ordering::Relaxed);
        if let Some(seen) = self.seen.as_ref() {
            if let Ok(mut seen) = seen.lock() {
                let frame = request.frame.to_bits();
                for jd in request.jds {
                    for body in request.bodies {
                        seen.insert((jd.to_bits(), body.id(), frame));
                    }
                }
            }
        }
        self.inner.positions(request)
    }

    fn obliquity(&self, jd: f64, scale: TimeScale) -> Result<Obliquity, ProviderError> {
        self.obliquity.fetch_add(1, Ordering::Relaxed);
        self.inner.obliquity(jd, scale)
    }

    fn delta_t_seconds(&self, jd_ut1: f64) -> Result<f64, ProviderError> {
        self.delta_t.fetch_add(1, Ordering::Relaxed);
        self.inner.delta_t_seconds(jd_ut1)
    }

    fn ayanamsha_deg(
        &self,
        jd: f64,
        scale: TimeScale,
        ayanamsha: Ayanamsha,
    ) -> Result<f64, ProviderError> {
        self.ayanamsha.fetch_add(1, Ordering::Relaxed);
        self.inner.ayanamsha_deg(jd, scale, ayanamsha)
    }

    fn dut1_seconds(&self, jd_utc: f64) -> Result<f64, ProviderError> {
        self.dut1.fetch_add(1, Ordering::Relaxed);
        self.inner.dut1_seconds(jd_utc)
    }

    fn horizon_event(
        &self,
        request: &HorizonRequest,
    ) -> Result<Option<JulianDay<Ut1>>, ProviderError> {
        self.horizon.fetch_add(1, Ordering::Relaxed);
        self.inner.horizon_event(request)
    }

    fn crossings(&self, request: &CrossingRequest) -> Result<Vec<Event>, ProviderError> {
        self.crossings.fetch_add(1, Ordering::Relaxed);
        self.inner.crossings(request)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;
    use crate::body::Body;
    use crate::frame::Frame;
    use crate::test_provider::TestProvider;

    #[test]
    fn a_loop_and_a_batch_ask_for_the_same_cells_and_a_different_number_of_calls() {
        let jds: Vec<f64> = (0..10).map(|day| 2_451_545.0 + f64::from(day)).collect();
        let bodies = [Body::Sun, Body::Moon];

        let batched = CountingProvider::new(TestProvider);
        let request = PositionRequest::new(&jds, TimeScale::Ut1, &bodies, Frame::CANONICAL);
        batched.positions(&request).unwrap();

        let looped = CountingProvider::new(TestProvider);
        for jd in &jds {
            let one = std::slice::from_ref(jd);
            let request = PositionRequest::new(one, TimeScale::Ut1, &bodies, Frame::CANONICAL);
            looped.positions(&request).unwrap();
        }

        assert_eq!(batched.calls().cells, looped.calls().cells);
        assert_eq!(batched.calls().positions, 1);
        assert_eq!(looped.calls().positions, 10);
        assert_eq!(batched.calls().widest, 20);
        assert_eq!(looped.calls().widest, 2);
        assert!((looped.calls().mean_width() - 2.0).abs() < 1e-12);
        assert_eq!(batched.calls().total(), 1);
    }

    #[test]
    fn the_counters_start_at_nought_and_go_back_to_it() {
        let counted = CountingProvider::new(TestProvider);
        assert_eq!(counted.calls(), ProviderCalls::default());
        // No calls is a width of nought rather than a division by zero.
        assert!(ProviderCalls::default().mean_width().abs() < 1e-12);
        let jds = [2_451_545.0];
        let request = PositionRequest::new(&jds, TimeScale::Ut1, &[Body::Sun], Frame::CANONICAL);
        counted.positions(&request).unwrap();
        counted.obliquity(2_451_545.0, TimeScale::Ut1).ok();
        assert_eq!(counted.calls().total(), 2);
        counted.reset();
        assert_eq!(counted.calls(), ProviderCalls::default());
        assert_eq!(counted.inner().capabilities().bodies.len(), 8);
    }
}
