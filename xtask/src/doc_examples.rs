//! Every public item of a listed crate that rustdoc says can carry an
//! example carries one (`05-testing/01-quality-bar.md`, "documentation").
//!
//! rustdoc counts the items itself, under a lint only a nightly compiler
//! has (`missing_doc_code_examples`), so the check runs that lint as an
//! error on the nightly the coverage job already pins
//! ([`crate::coverage::COUNTED_BY`]): a moving compiler could start
//! counting items the crate never changed. The examples themselves run
//! as doctests on stable, under `cargo test --workspace`.
//!
//! **A list, raised by hand.** A crate joins [`EXAMPLED`] once it reaches
//! every item, and is held there from then on; the crates not yet on it
//! are the quality bar's open work, not a pass.

use std::path::Path;
use std::process::Command;

use crate::binding::step;
use crate::coverage::COUNTED_BY;

/// The crates held to an example on every item rustdoc counts.
const EXAMPLED: &[&str] = &[
    "teistro",
    "teistro-aspect",
    "teistro-calendar",
    "teistro-chart",
    "teistro-core",
    "teistro-dasha",
    "teistro-geometry",
    "teistro-gochar",
    "teistro-hellenistic",
    "teistro-houses",
    "teistro-interpret",
    "teistro-kp",
    "teistro-lalkitab",
    "teistro-matching",
    "teistro-muhurta",
    "teistro-numerology",
    "teistro-pakshi",
    "teistro-panchanga",
    "teistro-points",
    "teistro-port-ephemeris",
    "teistro-port-timezone",
    "teistro-prashna",
    "teistro-rashifal",
    "teistro-rectification",
    "teistro-remedies",
    "teistro-render-svg",
    "teistro-research",
    "teistro-serial",
    "teistro-siddhanta",
    "teistro-state",
    "teistro-strength",
    "teistro-time",
    "teistro-vargas",
    "teistro-western",
];

pub(crate) fn check(root: &Path) -> i32 {
    let mut failed = 0;
    for krate in EXAMPLED {
        let run = step(
            Command::new("cargo")
                .arg(format!("+{COUNTED_BY}"))
                .args(["rustdoc", "--quiet", "-p", krate, "--lib", "--"])
                .arg("-Zcrate-attr=feature(rustdoc_missing_doc_code_examples)")
                .args(["-D", "rustdoc::missing-doc-code-examples"])
                .current_dir(root),
            &format!("{krate}: every item rustdoc counts has an example"),
            &format!("{krate}: an item rustdoc counts has no example"),
        );
        failed += i32::from(run.is_err());
    }
    i32::from(failed > 0)
}
