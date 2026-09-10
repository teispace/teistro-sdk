//! How much of a batch is inside the engine, and how much is the SDK's
//! own arithmetic: the measurement the parallelism step needs before it
//! is designed (`07-roadmap/02-plan-performance-and-passthrough.md`, A3).
//!
//! It matters because of a rule the engine states about itself: a
//! `teimeris::Context` is `Send` and deliberately **not** `Sync`, because
//! "N contexts may be used concurrently from N threads with no locking —
//! which is a statement about N contexts, not about one shared between
//! them". So this adapter holds `Mutex<Context>`, and **every ephemeris
//! call the SDK makes takes one lock**. Threads over a batch that shares
//! one provider cannot overlap any of that; they can only overlap what
//! the SDK does between the calls.
//!
//! What is printed is the split. Nothing here is asserted as a timing —
//! a timing belongs to the machine — but the counts beside it are, so
//! the shape of the workload is held even though its speed is not. Run
//! by hand with the engine present.
#![allow(
    clippy::print_stdout,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::arithmetic_side_effects,
    reason = "a measurement prints what it measured and fails by panicking"
)]

use std::sync::Mutex;
use std::time::{Duration, Instant};

use teistro_core::catalogue::Ayanamsha;
use teistro_core::quantity::{JulianDay, Ut1};
use teistro_ephemeris_teimeris::{TeimerisProvider, data_dir_from_env};
use teistro_port_ephemeris::crossing::{CrossingRequest, Event};
use teistro_port_ephemeris::{
    Capabilities, EphemerisProvider, HorizonRequest, Obliquity, PositionColumns, PositionRequest,
    ProviderError, TimeScale,
};

/// A provider that times what its inner one spends, so the share inside
/// the engine can be told from the share outside it.
struct Timed {
    inner: TeimerisProvider,
    spent: Mutex<Duration>,
    calls: Mutex<u64>,
}

impl EphemerisProvider for Timed {
    fn capabilities(&self) -> Capabilities {
        self.inner.capabilities()
    }
    fn positions(&self, request: &PositionRequest<'_>) -> Result<PositionColumns, ProviderError> {
        let at = Instant::now();
        let answer = self.inner.positions(request);
        *self.spent.lock().unwrap() += at.elapsed();
        *self.calls.lock().unwrap() += 1;
        answer
    }
    fn obliquity(&self, jd: f64, s: TimeScale) -> Result<Obliquity, ProviderError> {
        let at = Instant::now();
        let answer = self.inner.obliquity(jd, s);
        *self.spent.lock().unwrap() += at.elapsed();
        answer
    }
    fn delta_t_seconds(&self, jd: f64) -> Result<f64, ProviderError> {
        self.inner.delta_t_seconds(jd)
    }
    fn ayanamsha_deg(&self, jd: f64, s: TimeScale, a: Ayanamsha) -> Result<f64, ProviderError> {
        self.inner.ayanamsha_deg(jd, s, a)
    }
    fn dut1_seconds(&self, jd: f64) -> Result<f64, ProviderError> {
        self.inner.dut1_seconds(jd)
    }
    fn horizon_event(&self, r: &HorizonRequest) -> Result<Option<JulianDay<Ut1>>, ProviderError> {
        self.inner.horizon_event(r)
    }
    fn crossings(&self, r: &CrossingRequest) -> Result<Vec<Event>, ProviderError> {
        self.inner.crossings(r)
    }
}

/// A year of the Moon's sign ingresses: the shape a panchanga is made
/// of, and the most numerous crossing search the SDK does.
#[test]
fn how_much_of_a_search_is_inside_the_engines_lock() {
    use teistro_astro::events::{Lattice, Quantity, Search};
    use teistro_astro::{Completion, DeltaTModel};
    use teistro_core::settings::OverridePolicy;
    use teistro_port_ephemeris::{Body, CachingProvider, Frame};

    let provider = Timed {
        inner: TeimerisProvider::open(&data_dir_from_env()).unwrap_or_else(|e| panic!("{e}")),
        spent: Mutex::new(Duration::ZERO),
        calls: Mutex::new(0),
    };
    let cached = CachingProvider::new(&provider);
    let completion = Completion::new(
        &cached,
        OverridePolicy::PreferNative,
        DeltaTModel::TableThenModel,
    );
    let longitudes = completion.longitudes(Frame::CANONICAL);
    let from = JulianDay::<Ut1>::literal(2_451_545.0);
    let to = JulianDay::<Ut1>::literal(2_451_545.0 + 365.25);

    let at = Instant::now();
    // The shape a panchanga is made of: a lattice search over a year for
    // the Moon, whose crossings are the most numerous the SDK does.
    let events = Search::new(&longitudes, Quantity::Longitude(Body::Moon), Lattice::SIGNS)
        .between(from, to)
        .expect("the engine answers");
    let whole = at.elapsed();
    let inside = *provider.spent.lock().unwrap();
    let calls = *provider.calls.lock().unwrap();
    let outside = whole.checked_sub(inside).unwrap_or(Duration::ZERO);

    println!("a year of the Moon's sign ingresses through the real engine");
    println!("  crossings found   {}", events.len());
    println!("  whole search      {whole:?}");
    println!(
        "  inside the engine {inside:?}  ({:.1}%)",
        100.0 * inside.as_secs_f64() / whole.as_secs_f64()
    );
    println!(
        "  outside it        {:?}  ({:.1}%)",
        outside,
        100.0 * outside.as_secs_f64() / whole.as_secs_f64()
    );
    println!("  positions calls   {calls}");
    println!("  cache             {:?}", cached.stats());

    // The timing is the machine's; the workload is not, and it is what
    // makes the timing mean anything.
    assert_eq!(events.len(), 160, "the Moon crosses a sign every 2.3 days");
    assert!(
        calls > 500,
        "the search really did reach the engine: {calls}"
    );
    assert!(
        inside > Duration::ZERO && inside < whole,
        "some of it inside the engine and some outside"
    );
}
