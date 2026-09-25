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
    clippy::panic,
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

/// The recorded charts, founded over this tier, against the band the
/// corpus's own tolerance file gives the tier's class: the corpus check
/// ADR-0022 asks of every tier, run three times by the three tier jobs.
/// Every miss is one of the kit's known divergences, each measured and
/// explained, and every divergence that applies to this tier explains one.
#[test]
fn the_built_in_ephemeris_reproduces_the_corpus_within_its_tiers_band() {
    use teistro_ephemeris_kit::corpus::{self, Corpus};
    use teistro_port_ephemeris::EphemerisProvider;

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
    let corpus = Corpus::open(&root).unwrap_or_else(|why| panic!("{why}"));
    let open = || -> Box<dyn EphemerisProvider> { Box::new(Builtin::new()) };
    let class = format!("builtin-{TIER_NAME}");
    let report = corpus::run(&corpus, &open, &class).unwrap_or_else(|why| panic!("{why}"));
    println!("\n{}\n", report.markdown());
    // Kept where a reader can argue with it: every field, not the summary.
    let target = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/kit");
    report
        .write(&target, &format!("corpus-{TIER_NAME}"))
        .unwrap_or_else(|why| panic!("{why}"));
    let judged = report.against(&corpus::KNOWN);
    assert!(
        judged.holds(),
        "the {TIER_NAME} tier against the corpus:\nunexplained: {:#?}\nidle: {:#?}",
        judged.unexplained,
        judged
            .idle
            .iter()
            .map(|divergence| divergence.name)
            .collect::<Vec<_>>()
    );
}

/// Under `sdk-only` a chart over the built-in ephemeris is its native
/// positions and nothing else the provider offers.
///
/// A property of the SDK's completion rather than of a tier's tables, so it
/// is compiled at the standard tier alone, the one fast-check builds: a
/// whole reading at the full tier costs eight seconds unoptimised, and
/// running the same proof three times proves it once.
#[cfg(feature = "standard")]
#[test]
fn a_chart_over_the_built_in_ephemeris_is_its_native_positions_under_sdk_only() {
    use teistro_port_ephemeris::EphemerisProvider;

    let open = || -> Box<dyn EphemerisProvider> { Box::new(Builtin::new()) };
    let check = teistro_ephemeris_kit::sdk_only::check(&open);
    println!("{}", check.detail);
    assert!(check.passed, "{}", check.detail);
}
