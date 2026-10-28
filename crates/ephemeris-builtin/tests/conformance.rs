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

#[path = "../../core/tests/support/conformance.rs"]
mod conformance;

#[path = "../../astro/tests/common/mod.rs"]
mod accuracy;

use conformance::Tally;
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
    let mut tally = Tally::new(
        "baseline",
        format!("positions over the built-in {TIER_NAME} tier"),
    );
    let fields = report.results.iter().flat_map(|result| &result.fields);
    tally.agreed(fields.filter(|field| field.within).count());
    for (divergence, count) in judged.explained_by {
        tally.explained(divergence.name, count);
    }
    tally.record();
}

/// The recorded variants, each founded under the settings the corpus's
/// manifest stored for it as the compatibility shim translates them
/// (`03-design/compat-shim.md` item 2), over this tier and against its
/// class's band. No variant's settings are written here; a profile the
/// shim refuses is reported apart, and only the bija correction is (C777).
#[test]
fn the_built_in_ephemeris_reproduces_the_recorded_variants() {
    use teistro_ephemeris_kit::corpus::Corpus;
    use teistro_ephemeris_kit::variants;
    use teistro_port_ephemeris::EphemerisProvider;

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
    let corpus = Corpus::open(&root).unwrap_or_else(|why| panic!("{why}"));
    let open = || -> Box<dyn EphemerisProvider> { Box::new(Builtin::new()) };
    let class = format!("builtin-{TIER_NAME}");
    let run = variants::run(&corpus, &open, &class).unwrap_or_else(|why| panic!("{why}"));
    println!("\n{}\n", run.report.markdown());
    for refusal in &run.refused {
        println!(
            "refused `{}` ({} variants): {}",
            refusal.profile,
            refusal.fixtures.len(),
            refusal.why
        );
    }
    let target = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/kit");
    run.report
        .write(&target, &format!("variants-{TIER_NAME}"))
        .unwrap_or_else(|why| panic!("{why}"));
    let refused: Vec<&str> = run
        .refused
        .iter()
        .map(|refusal| refusal.profile.as_str())
        .collect();
    assert_eq!(refused, ["surya-bija"]);
    let judged = run.report.against(&variants::KNOWN);
    assert!(
        judged.holds(),
        "the {TIER_NAME} tier against the recorded variants:\nunexplained: {:#?}\nidle: {:#?}",
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

/// The tier's geometric positions against DE440's own, CSPICE's states over
/// NAIF's kernel (evidence rank 1), under the band the corpus gives the
/// tier's class: what the tier's theory misses the modern ephemeris by,
/// with no light time, aberration or Delta T between them. Every miss is a
/// measured divergence from DE440 and every one that applies explains one.
#[test]
fn the_built_in_ephemeris_holds_to_de440_within_its_tiers_band() {
    use teistro_ephemeris_kit::corpus::Corpus;
    use teistro_ephemeris_kit::jpl;

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
    let corpus = Corpus::open(&root).unwrap_or_else(|why| panic!("{why}"));
    let class = format!("builtin-{TIER_NAME}");
    let run =
        jpl::geometric(&corpus, &Builtin::new(), &class).unwrap_or_else(|why| panic!("{why}"));
    for worst in run.report.worst() {
        println!(
            "{}: {:e} against {:e} at {}",
            worst.field, worst.difference, worst.tolerance, worst.fixture
        );
    }
    println!("outside the tier's range: {} instants", run.outside.len());
    let target = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/kit");
    run.report
        .write(&target, &format!("jpl-{TIER_NAME}"))
        .unwrap_or_else(|why| panic!("{why}"));
    let judged = run.report.against(&jpl::KNOWN);
    assert!(
        judged.holds(),
        "the {TIER_NAME} tier against DE440:\nunexplained: {:#?}\nidle: {:#?}",
        judged.unexplained,
        judged
            .idle
            .iter()
            .map(|divergence| divergence.name)
            .collect::<Vec<_>>()
    );
    record_accuracy(&run.report);
    let mut tally = Tally::new(
        "jpl",
        format!("geometric positions over the built-in {TIER_NAME} tier"),
    );
    let fields = run.report.results.iter().flat_map(|result| &result.fields);
    tally.agreed(fields.filter(|field| field.within).count());
    for (divergence, count) in judged.explained_by {
        tally.explained(divergence.name, count);
    }
    tally.record();
}

/// The accuracy document's `ephemeris` row: the worst difference from
/// DE440 a group of bodies shows, in arcseconds or kilometres, under its
/// band. A body enters a group only when every one of its values is
/// inside the band, so a bound the document prints is one each value
/// meets; the others' misses are the divergences [`jpl::KNOWN`] explains,
/// which `CONFORMANCE.md` counts. The bodies are named from the report,
/// so the document says which ones it measured.
///
/// [`jpl::KNOWN`]: teistro_ephemeris_kit::jpl::KNOWN
fn record_accuracy(report: &teistro_ephemeris_kit::corpus::CorpusReport) {
    use std::collections::BTreeMap;

    /// A body's values in a group: the worst, how many, the band and
    /// whether every one was inside it.
    type Body = (f64, usize, f64, bool);
    let mut groups: BTreeMap<&str, BTreeMap<String, Body>> = BTreeMap::new();
    let mut order: Vec<String> = Vec::new();
    for field in report.results.iter().flat_map(|result| &result.fields) {
        let path = field.path.trim_start_matches("jpl.geometric.");
        let (outer, path) = path
            .strip_prefix("outer.")
            .map_or((false, path), |rest| (true, rest));
        let (body, quantity) = path.split_once('.').expect("a body and a quantity");
        let group = match (quantity, outer) {
            ("direction_deg", false) => "direction, inner",
            ("direction_deg", true) => "direction, outer",
            ("distance_km", _) => "distance",
            _ => panic!("an unknown DE440 field {}", field.path),
        };
        if !order.iter().any(|known| known == body) {
            order.push(body.to_owned());
        }
        let entry = groups
            .entry(group)
            .or_default()
            .entry(body.to_owned())
            .or_insert((0.0, 0, field.tolerance, true));
        entry.0 = entry.0.max(field.difference);
        entry.1 += 1;
        entry.3 &= field.within;
    }
    for (group, bodies) in groups {
        let (mut named, mut worst, mut count, mut band) = (Vec::new(), 0.0_f64, 0, 0.0);
        for body in &order {
            if let Some(&(body_worst, body_count, body_band, true)) = bodies.get(body) {
                named.push(name(body));
                worst = worst.max(body_worst);
                count += body_count;
                band = body_band;
            }
        }
        if named.is_empty() {
            continue;
        }
        let (what, scale, unit) = if group == "distance" {
            ("distance", 1.0, " km")
        } else {
            ("direction", 3600.0, "″")
        };
        accuracy::record(
            "ephemeris",
            &format!(
                "the {TIER_NAME} tier's geometric {what} against DE440 of {}",
                listed(&named)
            ),
            worst * scale,
            unit,
            band * scale,
            count,
        );
    }
}

/// Names as a sentence lists them: `a`, `a and b`, `a, b and c`.
fn listed(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [only] => only.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

/// A body as the document names it.
fn name(body: &str) -> String {
    let mut chars = body.chars();
    chars.next().map_or_else(String::new, |first| {
        let rest: String = chars.as_str().to_lowercase();
        match body {
            "SUN" | "MOON" => format!("the {first}{rest}"),
            _ => format!("{first}{rest}"),
        }
    })
}
