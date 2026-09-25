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

/// Under `sdk-only` a chart over the text is its native places and nothing
/// else it offers: its obliquity, its ayanamsha and its sunrise are the
/// text's, and none of them may reach a chart the policy says the SDK
/// makes. Under `prefer-native` they must, or the identity proves nothing.
#[test]
fn under_sdk_only_a_chart_over_the_text_is_its_places_alone() {
    use teistro_port_ephemeris::EphemerisProvider;

    let open = || -> Box<dyn EphemerisProvider> { Box::new(SiddhantaProvider::text()) };
    let check = teistro_ephemeris_kit::sdk_only::check(&open);
    println!("{}", check.detail);
    assert!(check.passed, "{}", check.detail);
    assert!(
        check.detail.contains("overrides make them part"),
        "the text's overrides must reach a chart under `prefer-native`: {}",
        check.detail
    );
}
