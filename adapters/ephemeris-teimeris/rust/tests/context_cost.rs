//! What a second engine context costs.
//!
//! The SDK's parallelism is capped at 2.2x by one lock
//! (`parallelism_probe.rs`): a `teimeris::Context` is `Send` and not
//! `Sync`, so this adapter holds `Mutex<Context>` and every ephemeris
//! call queues. The engine's own rule says the way out — "N contexts
//! from N threads with no locking" — which makes one question decide
//! whether a provider per thread is worth building: **what does a second
//! context cost?**
//!
//! Opening one reads the ephemeris index and whatever else the profile
//! wants. If that is milliseconds it is nothing beside a batch; if it is
//! hundreds, a thread has to earn its context back before it saves
//! anything, and the threshold is a different number.
//!
//! Run by hand with the engine present.
#![allow(
    clippy::print_stdout,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::arithmetic_side_effects,
    reason = "a measurement prints what it measured and fails by panicking"
)]

use std::time::Instant;

use teistro_ephemeris_teimeris::{TeimerisProvider, data_dir_from_env};
use teistro_port_ephemeris::{Body, EphemerisProvider, Frame, PositionRequest, TimeScale};

#[test]
fn what_a_second_context_costs() {
    let dir = data_dir_from_env();

    // The first one warms whatever the process caches; the ones after it
    // are what a thread would actually pay.
    let at = Instant::now();
    let first = TeimerisProvider::open(&dir).unwrap_or_else(|e| panic!("{e}"));
    let first_open = at.elapsed();

    let mut opens = Vec::new();
    let mut providers = Vec::new();
    for _ in 0..7 {
        let at = Instant::now();
        let provider = TeimerisProvider::open(&dir).unwrap_or_else(|e| panic!("{e}"));
        opens.push(at.elapsed());
        providers.push(provider);
    }
    let total: std::time::Duration = opens.iter().sum();
    let each = total / u32::try_from(opens.len()).unwrap_or(1);

    // And what a call costs, so the two can be compared: a context is
    // worth opening when a thread's share of the work exceeds it.
    let jds = [2_451_545.0];
    let bodies = [Body::Sun, Body::Moon, Body::Mars];
    let request = PositionRequest::new(&jds, TimeScale::Ut1, &bodies, Frame::CANONICAL);
    first.positions(&request).expect("it answers");
    let at = Instant::now();
    for _ in 0..1000 {
        first.positions(&request).expect("it answers");
    }
    let per_call = at.elapsed() / 1000;

    println!("a second engine context");
    println!("  first open      {first_open:?}");
    println!("  each after it   {each:?}  (seven of them, {total:?})");
    println!(
        "  one positions call ({} cells)  {per_call:?}",
        bodies.len()
    );
    println!(
        "  a context is worth {} calls",
        each.as_secs_f64() / per_call.as_secs_f64()
    );

    // What the numbers decide, asserted so the conclusion cannot rot
    // quietly: a context has to be worth thousands of calls, and the
    // largest batch the SDK measures is 8 174 of them
    // (`03-design/batch-and-parallelism-measured.md`, fifty almanac days
    // with the memo). A thread that opened its own context would pay
    // more for it than the whole batch costs, so a provider per thread
    // loses for any single batch and only a pool that outlives many of
    // them can win — which is the consumer's to keep, not the SDK's to
    // make.
    let worth = each.as_secs_f64() / per_call.as_secs_f64();
    assert!(
        worth > 8_174.0,
        "a context costs {worth} calls; if this ever falls below the \
         largest measured batch, a provider per thread is worth \
         revisiting"
    );

    assert_eq!(providers.len(), 7, "seven more contexts opened at once");
    // Each of them answers: a second context is a working engine and not
    // a handle that shares the first one's state.
    for provider in &providers {
        assert!(provider.positions(&request).is_ok());
    }
}
