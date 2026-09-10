//! The provider conformance kit, run against the built-in ephemeris.
//!
//! `crates/ephemeris-kit` is the checklist every provider of this port
//! has to pass: that its capabilities describe it, that its cells are
//! finite, that a repeated request gives identical bits when it claims
//! determinism, that a batch answers what singles answer, and that a
//! body it declares is a body it computes.
//!
//! Phase 3's exit asks for the kit against **every tier**, and the tiers
//! are cargo features, so this test is the kit against whichever one is
//! compiled. Running it three times is running it three times.
//!
//! The kit is where a provider's own tests stop being enough: a provider
//! author can only test what they thought of, and the kit is what
//! everyone else thought of.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::print_stdout,
    reason = "a test fails by panicking, and the report is worth reading"
)]

use teistro_ephemeris_builtin::provider::Builtin;
use teistro_ephemeris_builtin::tables::TIER_NAME;
use teistro_ephemeris_kit::{Bounds, run};

#[test]
fn the_built_in_ephemeris_passes_the_provider_kit() {
    let provider = Builtin::new();
    let report = run(&provider, &Bounds::DEFAULT);
    println!("\n{}\n", report.markdown());
    let failed: Vec<&str> = report
        .checks
        .iter()
        .filter(|check| check.ran() && !check.passed)
        .map(|check| check.name)
        .collect();
    assert!(
        failed.is_empty(),
        "the {TIER_NAME} tier fails the kit: {}",
        failed.join(", ")
    );
}

/// Turning the bija off must not turn conformance off with it: the
/// correction changes what the Moon says, never whether the provider
/// keeps its contract.
#[test]
fn the_uncorrected_theory_passes_the_kit_too() {
    let report = run(&Builtin::without_bija(), &Bounds::DEFAULT);
    let failed: Vec<&str> = report
        .checks
        .iter()
        .filter(|check| check.ran() && !check.passed)
        .map(|check| check.name)
        .collect();
    assert!(failed.is_empty(), "without the bija: {}", failed.join(", "));
}
