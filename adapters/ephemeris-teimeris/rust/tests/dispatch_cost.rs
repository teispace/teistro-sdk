//! What a JSON dispatch would cost, measured before it is built.
//!
//! The engine's own conventions say to benchmark before adding, because
//! two planned APIs there died on a benchmark that ran before any code
//! existed. The SDK's passthrough needs a call by name with arguments as
//! JSON (`port_ephemeris::native`), and the question that decides its
//! shape is not whether it can be written but what it costs beside the
//! call it wraps.
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

/// A representative call: the arguments an engine function takes and the
/// answer it gives, in the shape the port's `native_call` relays.
const ARGUMENTS: &str = r#"{"jd_ut":2451545.0,"body":1,"flags":258,"observer":{"longitude":85.324,"latitude":27.7172,"altitude":1400.0}}"#;
const ANSWER: &str = r#"{"status":0,"longitude":280.368166,"latitude":0.000123,"distance":0.983327,"speed_longitude":1.019041,"speed_latitude":0.000004,"speed_distance":-0.000178}"#;

#[test]
fn what_a_json_dispatch_would_cost_beside_the_call_it_wraps() {
    let provider = TeimerisProvider::open(&data_dir_from_env()).unwrap_or_else(|e| panic!("{e}"));
    let jds = [2_451_545.0];
    let bodies = [Body::Sun, Body::Moon, Body::Mars];
    let request = PositionRequest::new(&jds, TimeScale::Ut1, &bodies, Frame::CANONICAL);
    provider.positions(&request).expect("it answers");

    let rounds = 10_000;
    let at = Instant::now();
    for _ in 0..rounds {
        provider.positions(&request).expect("it answers");
    }
    let per_call = at.elapsed() / rounds;

    // What the relay would add: parse the arguments, and parse the answer
    // the adapter hands back. The adapter's own marshalling — turning
    // parsed JSON into C arguments — is not counted here because it does
    // not exist yet; this is the floor, and a floor is what decides
    // whether the shape is affordable at all.
    let at = Instant::now();
    for _ in 0..rounds {
        let arguments: serde_json::Value = serde_json::from_str(ARGUMENTS).expect("arguments");
        std::hint::black_box(&arguments);
        let answer: serde_json::Value = serde_json::from_str(ANSWER).expect("answer");
        std::hint::black_box(&answer);
    }
    let per_round_trip = at.elapsed() / rounds;

    // And what it costs to write the answer, which the adapter must do.
    let answer: serde_json::Value = serde_json::from_str(ANSWER).expect("answer");
    let at = Instant::now();
    for _ in 0..rounds {
        std::hint::black_box(answer.to_string());
    }
    let per_write = at.elapsed() / rounds;

    let overhead = per_round_trip + per_write;
    println!("a call by name, against the call it wraps");
    println!("  one positions call, three cells   {per_call:?}");
    println!("  parsing arguments and answer      {per_round_trip:?}");
    println!("  writing the answer                {per_write:?}");
    println!("  the relay's floor                 {overhead:?}");
    println!(
        "  which is {:.2}x the call it wraps",
        overhead.as_secs_f64() / per_call.as_secs_f64()
    );

    assert!(per_call.as_nanos() > 0, "the engine was actually called");
    assert!(overhead.as_nanos() > 0, "the relay was actually measured");
}
