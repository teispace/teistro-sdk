//! A provider that remembers what it was already asked, so a batch asks
//! the ephemeris for each cell once.
//!
//! Measured rather than assumed (`03-design/batch-and-parallelism-measured.md`,
//! gated by `cargo xtask check-batching`, which holds the numbers): a range
//! of almanac days asks for a large share of its cells more than once, and
//! the share **rises with the batch**, which can only come from sharing
//! between its items:
//! consecutive days scan overlapping windows for the same crossings, and
//! since the scan's samples are aligned to one epoch
//! (`astro::events::SCAN_ANCHOR_JD`) they are the *same instants* rather
//! than nearby different ones.
//!
//! # What makes it sound
//!
//! A memo answers a repeat with the first answer, so it is correct
//! exactly when the provider would have given the same answer again. The
//! port has always asked every provider to say so —
//! [`Capabilities::deterministic`] — and nothing read it. This reads it:
//! a provider that does not declare determinism is **not cached**, and
//! [`CachingProvider::caching`] says which happened rather than leaving a
//! caller to assume.
//!
//! # What it remembers besides answers
//!
//! A refusal of the shape of a request: the provider's
//! [`ProviderError::Unsupported`], which the port defines as a frame (or an
//! option) the provider cannot produce. The SDK's completion asks a
//! provider for the frame it wants before completing that frame from the
//! native one, so a provider that produces its native frame alone is asked,
//! and refuses, once for every request in any other frame. The memo keeps
//! the refusal by everything but the instants — the frame, the time scale,
//! the bodies, the speeds, the observer — and gives it back without asking
//! again. No other error is kept: an instant out of range or a missing
//! file is a property of the instants or of the machine, not of the shape.
//!
//! # What it does not change
//!
//! The capabilities it reports are the inner provider's, unchanged, name
//! and all. They reach the provenance stamp of every value the SDK
//! produces, so a wrapper that renamed the provider would change what
//! every chart says about itself — and the point of a cache is that
//! nothing downstream can tell it is there.
//!
//! # What it costs
//!
//! One `BTreeMap` behind one lock, bounded by
//! [`CachingProvider::with_capacity`]. Past the bound the cache stops
//! admitting and keeps serving what it holds, so memory is a number a
//! caller chose rather than a function of how long the batch ran. A
//! `BTreeMap` and not a `HashMap` because the determinism lints forbid
//! unordered iteration in a computation crate, and a cache that iterates
//! in a defined order needs no exception.
//!
//! ```
//! use teistro_port_ephemeris::{
//!     Body, CachingProvider, EphemerisProvider, Frame, PositionRequest, TestProvider, TimeScale,
//! };
//!
//! let cached = CachingProvider::new(TestProvider::new());
//! let jds = [2_451_545.0, 2_451_546.0];
//! let request = PositionRequest::new(&jds, TimeScale::Ut1, &[Body::Sun], Frame::CANONICAL);
//! cached.positions(&request).expect("the test provider answers");
//! cached.positions(&request).expect("and answers again");
//! assert_eq!(cached.stats().hits, 2, "the second call touched no ephemeris");
//! assert_eq!(cached.stats().misses, 2);
//! ```

use core::sync::atomic::{AtomicU64, Ordering};
use std::collections::BTreeMap;
use std::sync::Mutex;

use teistro_core::catalogue::Ayanamsha;
use teistro_core::quantity::{JulianDay, Ut1};

use crate::angles::{Angles, AnglesRequest};
use crate::body::{Body, TimeScale};
use crate::capabilities::{Capabilities, Obliquity};
use crate::columns::{Cell, PositionColumns};
use crate::crossing::{CrossingRequest, Event};
use crate::error::ProviderError;
use crate::frame::Frame;
use crate::horizon::HorizonRequest;
use crate::provider::{EphemerisProvider, PositionRequest};

/// How many cells a cache holds before it stops admitting, by default.
///
/// The settings knob `provider.cache_cells` is the same number, and it is
/// declared there rather than here because that is where a consumer sets
/// it; this is the one place that reads it as a capacity.
pub const DEFAULT_CAPACITY: usize = teistro_core::settings::DEFAULT_CACHE_CELLS as usize;

/// Everything that decides a cell's value, so that two requests share an
/// answer exactly when they are asking the same question.
///
/// `speeds` is part of it although a cell computed with speeds carries
/// everything a cell computed without them does: a provider is free to
/// take a different path when speeds are not wanted, and a memo that
/// assumed otherwise would be wrong on a provider that did.
///
/// [`crate::counting::CountingProvider`] counts distinct cells by the same
/// key, so a cell it calls asked for twice is exactly one a memo could
/// have answered.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(crate) struct CellKey {
    jd: u64,
    scale: u32,
    body: u16,
    frame: u32,
    speeds: bool,
    observer: Option<[u64; 3]>,
}

/// A cell and the frame it was answered in.
///
/// The frame is stored because a provider may answer in its own native
/// frame rather than the one asked for, and the completion reads which
/// it got. A cache that dropped it would hand back cells labelled with
/// the frame that was wanted rather than the one that was given.
#[derive(Clone, Copy, Debug)]
struct Stored {
    cell: Cell,
    frame: Frame,
}

/// What a cache did, for a caller that wants to know whether it helped.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CacheStats {
    /// Cells answered from the cache.
    pub hits: u64,
    /// Cells that had to be asked for.
    pub misses: u64,
    /// Cells the cache could not admit because it was full.
    pub refused: u64,
    /// Cells missed in requests the provider refused. Nothing of them is
    /// kept as an answer; a refusal as unsupported is kept as a refusal
    /// of the request's shape (see `remembered`).
    pub failed: u64,
    /// Cells of requests refused from memory, without asking: a shape the
    /// provider refused as unsupported before (a frame it cannot produce,
    /// asked first under `prefer-native`).
    pub remembered: u64,
    /// Cells held.
    pub stored: u64,
}

impl CacheStats {
    /// The share of cells answered without touching the ephemeris, in
    /// `[0, 1]`; zero when nothing was asked for.
    #[must_use]
    pub fn hit_share(&self) -> f64 {
        let asked = self.hits + self.misses;
        if asked == 0 {
            return 0.0;
        }
        #[allow(
            clippy::cast_precision_loss,
            reason = "a ratio of counts, for reporting"
        )]
        {
            self.hits as f64 / asked as f64
        }
    }
}

/// A provider that answers a repeated cell from memory.
///
/// Wraps any provider, including a borrowed one — `EphemerisProvider` is
/// implemented for `&P`, so `CachingProvider::new(&provider)` leaves the
/// provider where it is.
#[derive(Debug)]
pub struct CachingProvider<P> {
    inner: P,
    cells: Mutex<BTreeMap<CellKey, Stored>>,
    /// The shapes the provider refused as unsupported, with what it said.
    shapes: Mutex<BTreeMap<ShapeKey, String>>,
    capacity: usize,
    caching: bool,
    hits: AtomicU64,
    misses: AtomicU64,
    refused: AtomicU64,
    failed: AtomicU64,
    remembered: AtomicU64,
}

impl<P: EphemerisProvider> CachingProvider<P> {
    /// A cache over a provider, holding [`DEFAULT_CAPACITY`] cells.
    ///
    /// A provider that does not declare `deterministic` is wrapped but
    /// not cached: every call goes through, and [`Self::caching`] is
    /// false.
    #[must_use]
    pub fn new(inner: P) -> CachingProvider<P> {
        Self::with_capacity(inner, DEFAULT_CAPACITY)
    }

    /// A cache holding at most `capacity` cells. Zero disables it, which
    /// is the same as not wrapping at all but keeps a caller's code the
    /// one shape.
    #[must_use]
    pub fn with_capacity(inner: P, capacity: usize) -> CachingProvider<P> {
        let caching = capacity > 0 && inner.capabilities().deterministic;
        CachingProvider {
            inner,
            cells: Mutex::new(BTreeMap::new()),
            shapes: Mutex::new(BTreeMap::new()),
            capacity,
            caching,
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            refused: AtomicU64::new(0),
            failed: AtomicU64::new(0),
            remembered: AtomicU64::new(0),
        }
    }

    /// Whether cells are being remembered: the capacity is positive and
    /// the provider declares that identical requests get identical bits.
    #[must_use]
    pub const fn caching(&self) -> bool {
        self.caching
    }

    /// The provider underneath.
    pub const fn inner(&self) -> &P {
        &self.inner
    }

    /// What the cache has done so far.
    #[must_use]
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            refused: self.refused.load(Ordering::Relaxed),
            failed: self.failed.load(Ordering::Relaxed),
            remembered: self.remembered.load(Ordering::Relaxed),
            stored: self
                .cells
                .lock()
                .map_or(0, |cells| u64::try_from(cells.len()).unwrap_or(u64::MAX)),
        }
    }

    /// Forgets everything and zeroes the counts.
    pub fn clear(&self) {
        if let Ok(mut cells) = self.cells.lock() {
            cells.clear();
        }
        if let Ok(mut shapes) = self.shapes.lock() {
            shapes.clear();
        }
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
        self.refused.store(0, Ordering::Relaxed);
        self.failed.store(0, Ordering::Relaxed);
        self.remembered.store(0, Ordering::Relaxed);
    }
}

/// Everything of a request but its instants: what a refusal as
/// unsupported is a property of.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct ShapeKey {
    scale: u32,
    frame: u32,
    speeds: bool,
    observer: Option<[u64; 3]>,
    bodies: Vec<u16>,
}

impl ShapeKey {
    fn of(request: &PositionRequest<'_>) -> ShapeKey {
        ShapeKey {
            scale: request.scale.id(),
            frame: request.frame.to_bits(),
            speeds: request.speeds,
            observer: request.observer.map(place_bits),
            bodies: request.bodies.iter().map(|body| body.id()).collect(),
        }
    }
}

/// A place to the bit, for a key.
fn place_bits(place: teistro_core::quantity::Place) -> [u64; 3] {
    [
        place.latitude.get().to_bits(),
        place.longitude.get().to_bits(),
        place.altitude.get().to_bits(),
    ]
}

impl CellKey {
    /// The key of one cell of a request.
    pub(crate) fn of(request: &PositionRequest<'_>, jd: f64, body: Body) -> CellKey {
        CellKey {
            jd: jd.to_bits(),
            scale: request.scale.id(),
            body: body.id(),
            frame: request.frame.to_bits(),
            speeds: request.speeds,
            observer: request.observer.map(place_bits),
        }
    }
}

impl<P: EphemerisProvider> CachingProvider<P> {
    /// What the provider said when it refused this request's shape as
    /// unsupported, if it has.
    fn refused_before(&self, request: &PositionRequest<'_>) -> Option<String> {
        let shapes = self.shapes.lock().ok()?;
        shapes.get(&ShapeKey::of(request)).cloned()
    }

    /// Keeps a refusal as unsupported against the request's shape, within
    /// the same bound as the cells.
    fn remember_refusal(&self, request: &PositionRequest<'_>, what: &str) {
        if let Ok(mut shapes) = self.shapes.lock() {
            if shapes.len() < self.capacity {
                shapes.insert(ShapeKey::of(request), what.to_owned());
            }
        }
    }

    /// The cached answer, or `None` when any cell of the request is
    /// missing, along with what was found.
    fn gather(&self, request: &PositionRequest<'_>) -> Vec<Option<Stored>> {
        let mut found = Vec::with_capacity(request.cell_count());
        let Ok(cells) = self.cells.lock() else {
            return vec![None; request.cell_count()];
        };
        for jd in request.jds {
            for body in request.bodies {
                found.push(cells.get(&CellKey::of(request, *jd, *body)).copied());
            }
        }
        found
    }
}

impl<P: EphemerisProvider> EphemerisProvider for CachingProvider<P> {
    fn capabilities(&self) -> Capabilities {
        // The inner provider's, unchanged: they reach every provenance
        // stamp, and a cache nothing downstream can tell is there must
        // not be visible in one.
        self.inner.capabilities()
    }

    fn positions(&self, request: &PositionRequest<'_>) -> Result<PositionColumns, ProviderError> {
        if !self.caching || request.jds.is_empty() || request.bodies.is_empty() {
            return self.inner.positions(request);
        }
        if let Some(what) = self.refused_before(request) {
            self.remembered.fetch_add(
                u64::try_from(request.cell_count()).unwrap_or(0),
                Ordering::Relaxed,
            );
            return Err(ProviderError::Unsupported { what });
        }
        let found = self.gather(request);
        let hits = found.iter().filter(|slot| slot.is_some()).count();
        let misses = found.len() - hits;
        self.hits
            .fetch_add(u64::try_from(hits).unwrap_or(0), Ordering::Relaxed);
        self.misses
            .fetch_add(u64::try_from(misses).unwrap_or(0), Ordering::Relaxed);

        if misses == 0 {
            return Ok(assemble(request, &found, &[], &BTreeMap::new(), &[]));
        }

        // Ask only for what is missing: the instants that lack a cell, by
        // the bodies that are missing at any of them. That is a grid, so
        // it is one call however scattered the gaps are, and it over-asks
        // only where a body was already known at an instant another body
        // was not.
        let (wanted_jds, wanted_bodies) = missing(request, &found);
        let sub = PositionRequest {
            jds: &wanted_jds,
            bodies: &wanted_bodies,
            ..*request
        };
        let answered = self.inner.positions(&sub).inspect_err(|error| {
            self.failed
                .fetch_add(u64::try_from(misses).unwrap_or(0), Ordering::Relaxed);
            if let ProviderError::Unsupported { what } = error {
                self.remember_refusal(&sub, what);
            }
        })?;

        let mut where_jd: BTreeMap<u64, usize> = BTreeMap::new();
        for (row, jd) in wanted_jds.iter().enumerate() {
            where_jd.insert(jd.to_bits(), row);
        }

        if let Ok(mut cells) = self.cells.lock() {
            let mut refused = 0u64;
            for (row, jd) in wanted_jds.iter().enumerate() {
                for (column, body) in wanted_bodies.iter().enumerate() {
                    let Some(cell) = answered.at(row, column) else {
                        continue;
                    };
                    if cells.len() >= self.capacity {
                        refused += 1;
                        continue;
                    }
                    cells.insert(
                        CellKey::of(request, *jd, *body),
                        Stored {
                            cell,
                            frame: answered.frame,
                        },
                    );
                }
            }
            self.refused.fetch_add(refused, Ordering::Relaxed);
        }

        Ok(assemble(
            request,
            &found,
            &wanted_bodies,
            &where_jd,
            &[answered],
        ))
    }

    fn obliquity(&self, jd: f64, scale: TimeScale) -> Result<Obliquity, ProviderError> {
        self.inner.obliquity(jd, scale)
    }

    fn delta_t_seconds(&self, jd_ut1: f64) -> Result<f64, ProviderError> {
        self.inner.delta_t_seconds(jd_ut1)
    }

    fn ayanamsha_deg(
        &self,
        jd: f64,
        scale: TimeScale,
        ayanamsha: Ayanamsha,
    ) -> Result<f64, ProviderError> {
        self.inner.ayanamsha_deg(jd, scale, ayanamsha)
    }

    fn dut1_seconds(&self, jd_utc: f64) -> Result<f64, ProviderError> {
        self.inner.dut1_seconds(jd_utc)
    }

    fn horizon_event(
        &self,
        request: &HorizonRequest,
    ) -> Result<Option<JulianDay<Ut1>>, ProviderError> {
        self.inner.horizon_event(request)
    }

    fn angles(&self, request: &AnglesRequest) -> Result<Angles, ProviderError> {
        self.inner.angles(request)
    }

    fn crossings(&self, request: &CrossingRequest) -> Result<Vec<Event>, ProviderError> {
        self.inner.crossings(request)
    }

    fn native_manifest(&self) -> Result<String, ProviderError> {
        self.inner.native_manifest()
    }

    fn native_call(&self, function: &str, arguments_json: &str) -> Result<String, ProviderError> {
        self.inner.native_call(function, arguments_json)
    }
}

/// The instants and bodies a request still needs: the instants that lack
/// at least one cell, and the bodies that are missing at any instant,
/// each in the order the request named them.
fn missing(request: &PositionRequest<'_>, found: &[Option<Stored>]) -> (Vec<f64>, Vec<Body>) {
    let mut jds = Vec::new();
    let mut bodies = Vec::new();
    for (row, jd) in request.jds.iter().enumerate() {
        let mut row_missing = false;
        for (column, body) in request.bodies.iter().enumerate() {
            if found
                .get(row * request.bodies.len() + column)
                .is_some_and(Option::is_some)
            {
                continue;
            }
            row_missing = true;
            if !bodies.contains(body) {
                bodies.push(*body);
            }
        }
        if row_missing {
            jds.push(*jd);
        }
    }
    (jds, bodies)
}

/// The request's answer, cell by cell, from what the cache held and what
/// the provider just gave.
fn assemble(
    request: &PositionRequest<'_>,
    found: &[Option<Stored>],
    wanted_bodies: &[Body],
    where_jd: &BTreeMap<u64, usize>,
    answered: &[PositionColumns],
) -> PositionColumns {
    let frame = found
        .iter()
        .flatten()
        .map(|stored| stored.frame)
        .next()
        .or_else(|| answered.first().map(|columns| columns.frame))
        .unwrap_or(request.frame);
    let mut columns = PositionColumns::new(request.jds.len(), request.bodies.len(), frame);
    for (row, jd) in request.jds.iter().enumerate() {
        for (column, body) in request.bodies.iter().enumerate() {
            let cell = found
                .get(row * request.bodies.len() + column)
                .copied()
                .flatten()
                .map(|stored| stored.cell)
                .or_else(|| {
                    let sub_row = *where_jd.get(&jd.to_bits())?;
                    let sub_column = wanted_bodies.iter().position(|wanted| wanted == body)?;
                    answered.first()?.at(sub_row, sub_column)
                });
            if let Some(cell) = cell {
                columns.set_at(row, column, cell);
            }
        }
    }
    columns
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::float_cmp,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and read their own small grids"
    )]

    use super::*;
    use crate::counting::CountingProvider;
    use crate::test_provider::TestProvider;
    use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};

    const BODIES: [Body; 2] = [Body::Sun, Body::Moon];

    fn grid(jds: &[f64]) -> Vec<f64> {
        jds.to_vec()
    }

    fn place() -> Place {
        Place::new(
            Latitude::literal(27.7172),
            Longitude::literal(85.3240),
            Altitude::literal(1400.0),
        )
    }

    /// A provider that says its answers may differ between calls.
    #[derive(Debug)]
    struct Wobbly(TestProvider);

    impl EphemerisProvider for Wobbly {
        fn capabilities(&self) -> Capabilities {
            Capabilities {
                deterministic: false,
                ..self.0.capabilities()
            }
        }
        fn positions(
            &self,
            request: &PositionRequest<'_>,
        ) -> Result<PositionColumns, ProviderError> {
            self.0.positions(request)
        }
    }

    #[test]
    fn a_repeated_cell_is_answered_from_memory() {
        let inner = CountingProvider::new(TestProvider::new());
        let cached = CachingProvider::new(inner);
        let jds = grid(&[2_451_545.0, 2_451_546.0, 2_451_547.0]);
        let request = PositionRequest::new(&jds, TimeScale::Ut1, &BODIES, Frame::CANONICAL);
        let first = cached.positions(&request).unwrap();
        let second = cached.positions(&request).unwrap();
        assert_eq!(first, second, "the same answer, cell for cell");
        assert_eq!(cached.inner().calls().positions, 1, "one call, not two");
        let stats = cached.stats();
        assert_eq!(stats.misses, 6);
        assert_eq!(stats.hits, 6);
        assert_eq!(stats.stored, 6);
        assert_eq!(stats.refused, 0);
        assert!((stats.hit_share() - 0.5).abs() < 1e-12);
    }

    /// A deterministic provider whose data is missing at every instant.
    #[derive(Debug)]
    struct Missing;

    impl EphemerisProvider for Missing {
        fn capabilities(&self) -> Capabilities {
            TestProvider::new().capabilities()
        }
        fn positions(&self, _: &PositionRequest<'_>) -> Result<PositionColumns, ProviderError> {
            Err(ProviderError::DataMissing {
                detail: "no file".to_owned(),
            })
        }
    }

    fn sidereal() -> Frame {
        Frame::CANONICAL.with_zodiac(crate::frame::Zodiac::sidereal(
            teistro_core::catalogue::Ayanamsha::Lahiri,
        ))
    }

    #[test]
    fn a_shape_refused_as_unsupported_is_refused_again_from_memory() {
        // The test provider produces its canonical frame alone, so a
        // sidereal request is refused, and the refusal is the shape's:
        // other instants of the same shape are refused without asking.
        let inner = CountingProvider::new(TestProvider::new()).watching_repeats();
        let cached = CachingProvider::new(inner);
        let (first, later) = (grid(&[2_451_545.0, 2_451_546.0]), grid(&[2_451_600.0]));
        let asked = |jds: &[f64]| {
            cached.positions(&PositionRequest::new(
                jds,
                TimeScale::Ut1,
                &BODIES,
                sidereal(),
            ))
        };
        let refusal = asked(&first).unwrap_err();
        assert!(matches!(refusal, ProviderError::Unsupported { .. }));
        assert_eq!(
            asked(&first).unwrap_err(),
            refusal,
            "the provider's own words"
        );
        assert_eq!(asked(&later).unwrap_err(), refusal);
        let stats = cached.stats();
        assert_eq!((stats.misses, stats.failed, stats.remembered), (4, 4, 6));
        assert_eq!(stats.stored, 0, "a refusal is not an answer");
        assert_eq!(cached.inner().calls().positions, 1);
        // Another shape is its own question: the same frame with one body
        // is asked, and the canonical frame is answered.
        let one = cached.positions(&PositionRequest::new(
            &first,
            TimeScale::Ut1,
            &[Body::Sun],
            sidereal(),
        ));
        assert!(matches!(one, Err(ProviderError::Unsupported { .. })));
        assert_eq!(cached.inner().calls().positions, 2);
        let canonical = PositionRequest::new(&first, TimeScale::Ut1, &BODIES, Frame::CANONICAL);
        assert!(cached.positions(&canonical).is_ok());
        // Forgetting forgets the refusals too.
        cached.clear();
        assert!(asked(&first).is_err());
        assert_eq!(cached.inner().calls().positions, 4);
    }

    #[test]
    fn an_error_other_than_unsupported_is_asked_again() {
        // Data missing is a property of the machine, not of the shape.
        let cached = CachingProvider::new(CountingProvider::new(Missing));
        let jds = grid(&[2_451_545.0]);
        let request = PositionRequest::new(&jds, TimeScale::Ut1, &BODIES, Frame::CANONICAL);
        for _ in 0..2 {
            assert!(matches!(
                cached.positions(&request),
                Err(ProviderError::DataMissing { .. })
            ));
        }
        let stats = cached.stats();
        assert_eq!((stats.failed, stats.remembered), (4, 0));
        assert_eq!(cached.inner().calls().positions, 2);
    }

    #[test]
    fn the_counter_tells_cells_apart_as_the_memo_does() {
        // One instant and body with and without speeds are two questions
        // to a memo, so they are two distinct cells to the counter.
        let counter = CountingProvider::new(TestProvider::new()).watching_repeats();
        let jds = grid(&[2_451_545.0]);
        let mut request = PositionRequest::new(&jds, TimeScale::Ut1, &BODIES, Frame::CANONICAL);
        counter.positions(&request).unwrap();
        request.speeds = !request.speeds;
        counter.positions(&request).unwrap();
        counter.positions(&request).unwrap();
        let calls = counter.calls();
        assert_eq!((calls.cells, calls.distinct_cells), (6, 4));
    }

    #[test]
    fn an_overlapping_grid_asks_only_for_what_is_missing() {
        // This is the shape a batch actually makes: consecutive days scan
        // windows that share all but their ends, and since the samples are
        // anchored they share the *instants* and not merely the span.
        let inner = CountingProvider::new(TestProvider::new()).watching_repeats();
        let cached = CachingProvider::new(inner);
        let first = grid(&[2_451_545.0, 2_451_546.0, 2_451_547.0, 2_451_548.0]);
        let second = grid(&[2_451_546.0, 2_451_547.0, 2_451_548.0, 2_451_549.0]);
        cached
            .positions(&PositionRequest::new(
                &first,
                TimeScale::Ut1,
                &BODIES,
                Frame::CANONICAL,
            ))
            .unwrap();
        cached
            .positions(&PositionRequest::new(
                &second,
                TimeScale::Ut1,
                &BODIES,
                Frame::CANONICAL,
            ))
            .unwrap();
        let calls = cached.inner().calls();
        assert_eq!(calls.positions, 2, "one call each");
        // Eight cells asked for, but the second call carried only the two
        // that were new.
        assert_eq!(calls.cells, 8 + 2, "the second call was one instant wide");
        assert_eq!(cached.stats().hits, 6);
    }

    #[test]
    fn the_cached_answer_is_the_uncached_answer_to_the_bit() {
        let plain = TestProvider::new();
        let cached = CachingProvider::new(TestProvider::new());
        let jds = grid(&[2_451_545.0, 2_451_545.25, 2_451_546.5]);
        for request in [
            PositionRequest::new(&jds, TimeScale::Ut1, &BODIES, Frame::CANONICAL),
            PositionRequest::new(&jds, TimeScale::Tt, &BODIES, Frame::CANONICAL),
            PositionRequest::new(&jds, TimeScale::Ut1, &BODIES, Frame::CANONICAL).without_speeds(),
        ] {
            let want = plain.positions(&request).unwrap();
            // Twice, so the second is served from the cache.
            assert_eq!(cached.positions(&request).unwrap(), want);
            let got = cached.positions(&request).unwrap();
            assert_eq!(got.frame, want.frame);
            for index in 0..want.len() {
                let (a, b) = (got.cell(index).unwrap(), want.cell(index).unwrap());
                assert_eq!(a.lon.to_bits(), b.lon.to_bits(), "cell {index}");
                assert_eq!(a.lon_speed.to_bits(), b.lon_speed.to_bits(), "cell {index}");
                assert_eq!(a.status, b.status);
                assert_eq!(a.source, b.source);
            }
        }
    }

    #[test]
    fn everything_that_decides_a_cell_is_part_of_its_key() {
        let cached = CachingProvider::new(CountingProvider::new(TestProvider::new()));
        let jds = grid(&[2_451_545.0]);
        let base = PositionRequest::new(&jds, TimeScale::Ut1, &[Body::Sun], Frame::CANONICAL);
        let variants = [
            base,
            // A different scale is a different question.
            PositionRequest {
                scale: TimeScale::Tt,
                ..base
            },
            // So is a different observer. The frame stays canonical
            // because the test provider answers no other; what is being
            // checked is that the observer reaches the key at all, and a
            // topocentric frame would only be a second reason to miss.
            PositionRequest {
                observer: Some(place()),
                ..base
            },
            // And so is asking without speeds.
            base.without_speeds(),
        ];
        for request in variants {
            cached.positions(&request).unwrap();
        }
        assert_eq!(
            cached.inner().calls().positions,
            variants.len() as u64,
            "no variant was mistaken for another"
        );
        assert_eq!(cached.stats().hits, 0);
        // And each is remembered on its own.
        for request in variants {
            cached.positions(&request).unwrap();
        }
        assert_eq!(cached.inner().calls().positions, variants.len() as u64);
        assert_eq!(cached.stats().hits, variants.len() as u64);
    }

    #[test]
    fn a_provider_that_does_not_declare_determinism_is_not_cached() {
        let cached = CachingProvider::new(Wobbly(TestProvider::new()));
        assert!(!cached.caching(), "a memo over it would not be sound");
        let jds = grid(&[2_451_545.0]);
        let request = PositionRequest::new(&jds, TimeScale::Ut1, &BODIES, Frame::CANONICAL);
        cached.positions(&request).unwrap();
        cached.positions(&request).unwrap();
        assert_eq!(cached.stats(), CacheStats::default(), "nothing was counted");
    }

    #[test]
    fn a_full_cache_stops_admitting_and_keeps_answering() {
        let inner = CountingProvider::new(TestProvider::new());
        let cached = CachingProvider::with_capacity(inner, 2);
        let jds = grid(&[2_451_545.0, 2_451_546.0, 2_451_547.0]);
        let request = PositionRequest::new(&jds, TimeScale::Ut1, &BODIES, Frame::CANONICAL);
        let want = cached.positions(&request).unwrap();
        assert_eq!(cached.stats().stored, 2);
        assert_eq!(cached.stats().refused, 4);
        // The answer is still whole, and still right, from a cache that
        // holds a third of it.
        let got = cached.positions(&request).unwrap();
        assert_eq!(got, want);
        assert_eq!(cached.stats().hits, 2, "the two it kept");
    }

    #[test]
    fn a_capacity_of_nothing_is_a_cache_that_does_nothing() {
        let cached = CachingProvider::with_capacity(CountingProvider::new(TestProvider::new()), 0);
        assert!(!cached.caching());
        let jds = grid(&[2_451_545.0]);
        let request = PositionRequest::new(&jds, TimeScale::Ut1, &BODIES, Frame::CANONICAL);
        cached.positions(&request).unwrap();
        cached.positions(&request).unwrap();
        assert_eq!(cached.inner().calls().positions, 2);
    }

    #[test]
    fn clearing_forgets_everything_and_zeroes_the_counts() {
        let inner = CountingProvider::new(TestProvider::new());
        let cached = CachingProvider::new(inner);
        let jds = grid(&[2_451_545.0]);
        let request = PositionRequest::new(&jds, TimeScale::Ut1, &BODIES, Frame::CANONICAL);
        cached.positions(&request).unwrap();
        cached.positions(&request).unwrap();
        assert_eq!(cached.stats().hits, 2);
        cached.clear();
        assert_eq!(cached.stats(), CacheStats::default());
        cached.positions(&request).unwrap();
        assert_eq!(cached.inner().calls().positions, 2, "it asked again");
    }

    #[test]
    fn the_capabilities_are_the_inner_providers_unchanged() {
        // They reach every provenance stamp; a cache must be invisible.
        let plain = TestProvider::new();
        let cached = CachingProvider::new(TestProvider::new());
        assert_eq!(
            cached.capabilities().identity.name,
            plain.capabilities().identity.name
        );
        assert_eq!(
            cached.capabilities().describe(),
            plain.capabilities().describe()
        );
    }
}

#[cfg(test)]
mod native_tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index their own fixtures"
    )]

    use super::*;
    use crate::counting::CountingProvider;
    use crate::test_provider::TestProvider;

    /// A wrapper must not swallow what it wraps.
    ///
    /// The cache and the counter both stand between a consumer and their
    /// engine. If either dropped the engine's own operations, wrapping a
    /// provider — which the boundary now does from settings — would
    /// quietly close the door this whole passthrough opens.
    #[test]
    fn a_wrapped_provider_keeps_its_engines_own_operations() {
        let stacked = CachingProvider::new(CountingProvider::new(TestProvider::new()));
        assert!(stacked.capabilities().native, "declared through both");
        let native = stacked.native().expect("reachable through both");
        assert_eq!(native.manifest().unwrap().engine, "test-provider");
        let answer = native
            .call("tp_echo", &serde_json::json!({ "value": 2.5 }))
            .unwrap();
        assert_eq!(answer["value"], 2.5);
    }

    /// And through a borrow and a box, which is how the boundary holds one.
    #[test]
    fn a_borrowed_and_a_boxed_provider_keep_them_too() {
        let provider = TestProvider::new();
        let borrowed = &provider;
        assert_eq!(
            borrowed.native_call("tp_echo", r#"{"value":1}"#).unwrap(),
            provider.native_call("tp_echo", r#"{"value":1}"#).unwrap()
        );
        let boxed: Box<dyn EphemerisProvider> = Box::new(TestProvider::new());
        assert!(boxed.native_manifest().is_ok());
    }
}
