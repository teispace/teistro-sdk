//! The conformance kit over the classical provider: the text answers the
//! port honestly, its speeds declared as a rule, its distances as mean
//! distances, and its own sunrise as the rise and set override.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::print_stdout,
    reason = "tests fail by panicking and print the report under --nocapture"
)]

use teistro_ephemeris_kit::kit::{Bounds, run};
use teistro_siddhanta::SiddhantaProvider;

#[test]
fn the_text_passes_the_kit() {
    let report = run(&SiddhantaProvider::text(), &Bounds::DEFAULT);
    // `cargo test -p teistro-siddhanta --test kit -- --nocapture` prints
    // the report, whose informational rows measure the text's distance
    // from modern astronomy.
    println!("{}", report.markdown());
    assert!(report.passed, "{}", report.markdown());
    let speed = report.check("speed_consistency").unwrap();
    assert!(speed.detail.contains("rule"), "{}", speed.detail);
    assert!(report.check("override_rise_set_geometric").unwrap().ran());
    assert!(report.check("override_obliquity").unwrap().ran());
    assert!(report.check("override_ayanamsha").unwrap().ran());
}
