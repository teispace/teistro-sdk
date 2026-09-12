//! The parity gate: one scenario through every binding, and the reports
//! compared. It is the gate the architecture asks for, that the bindings
//! are one SDK and not three
//! (`02-architecture/07-binding-architecture.md`).
//!
//! Each runner (`bindings/node/parity.mjs`,
//! `bindings/dart/bin/parity.dart`, `bindings/python/parity.py`) walks the
//! same scenario through its own ergonomic layer and prints
//! `key<TAB>value` lines sorted by key. Nothing here says what a value
//! should be: the point is that the bindings agree with **each other**, so
//! a fact written into this file could only weaken it.
//!
//! # Values, and shape
//!
//! Most of what a runner prints is what its binding **answered**. The
//! `surface.*` lines are what it is **shaped** like: each key is a
//! canonical `area.operation` path and the member the runner references
//! beside it is that binding's own spelling of it. A binding that moved
//! an operation to another area, or renamed one, prints a key the others
//! do not, and the comparison below reports it as missing on one side and
//! extra on the other.
//!
//! That is the gap `03-design/surface-areas.md` named: three bindings held
//! to the same values and to no shape at all, so a namespaced surface
//! could drift into three groupings and this gate could not tell a
//! deliberate difference from a mistake.
//!
//! Every report is compared against the first that ran, which is what
//! makes the gate work on a machine with only some of the toolchains: two
//! present are still compared, and a third joins without the others
//! changing.
//!
//! Values are compared as text, except that two numbers are compared as
//! numbers within a relative tolerance, because nine decimals is where
//! two languages' formatting may disagree in the last digit and that is
//! not a difference between the bindings.
//!
//! Run by hand (`cargo xtask check-parity`) and in the nightly matrix; a
//! toolchain that is missing is reported and its runner skipped, and a
//! run with fewer than two reports is nothing to compare.

use std::path::Path;
use std::process::Command;

use crate::binding::{build, library, library_artefact, present};

const NODE: &str = "bindings/node/parity.mjs";
const DART: &str = "bindings/dart/bin/parity.dart";
const PYTHON: &str = "bindings/python/parity.py";
/// The Rust surface's runner, which is an example of its own crate
/// rather than a script beside a binding: a Rust consumer runs an
/// example, and `cargo` already knows how.
const RUST: &str = "crates/sdk/examples/parity.rs";
/// Where a number's last digit is allowed to differ. Both reports print
/// nine decimals, so two languages rounding the same double can differ by
/// one in that place and no more; the tolerance is absolute rather than
/// relative, because a relative one on a Julian day would swallow a tenth
/// of a second.
const TOLERANCE: f64 = 2e-9;

/// One binding's report: its lines, in the order it printed them.
struct Report {
    binding: &'static str,
    lines: Vec<(String, String)>,
    /// The keys this report is allowed not to print, or empty when it
    /// must print them all.
    ///
    /// **The Rust surface cannot print the same key set, by design**: a
    /// Rust consumer has no ABI, no build handshake and no result blob,
    /// because Cargo resolved the versions, `Drop` freed the memory and
    /// the crates handed back their own types
    /// (`03-design/rust-consumer-surface.md` §6).
    ///
    /// This was a **count** for four passes, and deliberately weaker
    /// than a list: at 549 absences a list would have had to name detail
    /// the runner simply did not print yet, a graha at a time. At nine
    /// it is a list, which is the stronger gate §7 was waiting for — an
    /// absence that stops being deliberate is now a failure rather than
    /// a number that stopped shrinking.
    absences: &'static [&'static str],
}

/// Every key the Rust runner does not print, and why each one is not a
/// gap.
///
/// Nine, and they are of three kinds. `abi` and the `build-*` keys are
/// the **boundary's own handshake**: there is no ABI between a Rust
/// consumer and the SDK, and nothing to hand-shake, because Cargo
/// resolved the graph — `sdk`, `catalogue-version` and `default-profile`
/// the runner *does* print, from constants. `provenance-fnv` is the
/// positions envelope's canonical JSON, whose input hash is of the
/// boundary's **decoded** request record; a Rust consumer holds the
/// `PositionRequest` itself, and the three fields of that envelope
/// anyone reads — the profile, the settings hash and the provider's
/// frame — the runner prints. And `surface.(root).dispose` is the one
/// operation this surface cannot have: a `Context` is dropped, so
/// listing it would be a disagreement where §6 intends an absence.
const RUST_ABSENCES: [&str; 9] = [
    "abi",
    "build-abi",
    "build-catalogue",
    "build-commit",
    "build-dirty",
    "build-sdk",
    "build-target",
    "provenance-fnv",
    "surface.(root).dispose",
];

impl Report {
    fn read(binding: &'static str, output: &str, absences: &'static [&'static str]) -> Report {
        let lines = output
            .lines()
            .filter_map(|line| line.split_once('\t'))
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect();
        Report {
            binding,
            lines,
            absences,
        }
    }
}

/// Whether two values say the same thing, as text or as numbers.
fn agree(left: &str, right: &str) -> bool {
    if left == right {
        return true;
    }
    match (left.parse::<f64>(), right.parse::<f64>()) {
        (Ok(a), Ok(b)) => (a - b).abs() <= TOLERANCE,
        _ => false,
    }
}

/// Compares two reports and prints every difference; the count is the
/// number of keys that disagree, are missing, or are extra.
fn compare(left: &Report, right: &Report) -> usize {
    let mut differences = 0;
    let mut left_keys: Vec<&str> = left.lines.iter().map(|(k, _)| k.as_str()).collect();
    let mut right_keys: Vec<&str> = right.lines.iter().map(|(k, _)| k.as_str()).collect();
    left_keys.sort_unstable();
    right_keys.sort_unstable();
    let mut absent: Vec<&str> = Vec::new();
    for key in &left_keys {
        if !right_keys.contains(key) {
            if right.absences.contains(key) {
                absent.push(key);
                continue;
            }
            println!(
                "      {key}: {} has it, {} does not",
                left.binding, right.binding
            );
            differences += 1;
        }
    }
    // Named, not counted, and **the list is exhaustive both ways**: an
    // absence not on it failed above, and one on it that the runner has
    // started printing fails here. A declared absence that stops being
    // deliberate is a failed gate rather than a number nobody watched.
    for declared in right.absences {
        if right_keys.contains(declared) {
            println!(
                "      {declared}: declared absent from {} and printed anyway; take it off the list",
                right.binding
            );
            differences += 1;
        } else if !left_keys.contains(declared) {
            println!(
                "      {declared}: declared absent from {} and {} does not print it either; the entry is stale",
                right.binding, left.binding
            );
            differences += 1;
        }
    }
    if !absent.is_empty() {
        println!(
            "      {} does not print {}, which §6 of `03-design/rust-consumer-surface.md` accounts for",
            right.binding,
            absent.join(", ")
        );
    }
    for key in &right_keys {
        if !left_keys.contains(key) {
            println!(
                "      {key}: {} has it, {} does not",
                right.binding, left.binding
            );
            differences += 1;
        }
    }
    for (key, value) in &left.lines {
        let Some((_, other)) = right.lines.iter().find(|(k, _)| k == key) else {
            continue;
        };
        if !agree(value, other) {
            println!(
                "      {key}: {} says `{value}`, {} says `{other}`",
                left.binding, right.binding
            );
            differences += 1;
        }
    }
    differences
}

/// Runs one binding's report, or `None` when the runner failed, which is
/// printed either way. Each caller sets the directory its runner expects.
fn run(
    binding: &'static str,
    command: &mut Command,
    absences: &'static [&'static str],
) -> Option<Report> {
    match command.output() {
        Ok(output) if output.status.success() => {
            let text = String::from_utf8_lossy(&output.stdout).to_string();
            let report = Report::read(binding, &text, absences);
            if report.lines.is_empty() {
                println!("FAIL  the {binding} runner printed no report");
                return None;
            }
            println!(
                "ok    the {binding} runner printed {} values",
                report.lines.len()
            );
            Some(report)
        }
        Ok(output) => {
            println!(
                "FAIL  the {binding} runner failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            );
            None
        }
        Err(error) => {
            println!("FAIL  the {binding} runner did not start: {error}");
            None
        }
    }
}

pub(crate) fn check(root: &Path) -> i32 {
    let has_node = present("node", "--version");
    let has_dart = present("dart", "--version");
    let python = crate::binding::python();
    let has_python = present(&python, "--version");
    let ran = has_node || has_dart || has_python;
    if !ran {
        eprintln!(
            "no `node`, `dart` or `{python}` on this machine; the parity gate needs two of them"
        );
        return 0;
    }
    if library(root).is_err() {
        return 1;
    }
    let mut reports = Vec::new();
    if has_node {
        if build(root, "teistro-node", "the Node addon").is_err() {
            return 1;
        }
        let built = root
            .join("target/release")
            .join(super::node_binding::addon_artefact());
        if std::fs::copy(&built, root.join(super::node_binding::ADDON)).is_err() {
            println!("FAIL  the Node addon could not be copied into the package");
            return 1;
        }
        reports.extend(run(
            "Node",
            Command::new("node").arg(NODE).current_dir(root),
            &[],
        ));
    } else {
        println!("skip  {NODE}: no `node` on this machine");
    }
    if has_dart {
        let library = root.join("target/release").join(library_artefact());
        reports.extend(run(
            "Dart",
            Command::new("dart")
                .args(["run", "bin/parity.dart"])
                .env("TEISTRO_LIBRARY", &library)
                .current_dir(root.join("bindings/dart")),
            &[],
        ));
    } else {
        println!("skip  {DART}: no `dart` on this machine");
    }
    if has_python {
        let library = root.join("target/release").join(library_artefact());
        reports.extend(run(
            "Python",
            crate::binding::python_command(&python)
                .arg("parity.py")
                .env("TEISTRO_LIBRARY", &library)
                .env("PYTHONPATH", root.join("bindings/python"))
                .current_dir(root.join("bindings/python")),
            &[],
        ));
    } else {
        println!("skip  {PYTHON}: no `{python}` on this machine");
    }
    // The Rust surface's own runner. No `present` check: it is an
    // example of a workspace crate, so a machine that can run this gate
    // can run it.
    println!("run   {RUST}");
    reports.extend(run(
        "Rust",
        Command::new(crate::binding::cargo())
            .args(["run", "--quiet", "-p", "teistro", "--example", "parity"])
            .current_dir(root),
        &RUST_ABSENCES,
    ));

    if reports.len() < 2 {
        println!("skip  nothing to compare: {} report(s)", reports.len());
        return i32::from(reports.is_empty() && ran);
    }
    // Every report against the first, so a machine with two toolchains
    // still gates the pair it has and a third joins without the others
    // changing.
    let first = &reports[0];
    let mut differences = 0;
    for other in &reports[1..] {
        let found = compare(first, other);
        if found == 0 {
            // The count compared, not the first report's: a report
            // agreeing on 665 of 674 must not read as 674.
            println!(
                "ok    the {} and {} bindings agree on every one of {} values",
                first.binding,
                other.binding,
                other.lines.len().min(first.lines.len())
            );
        }
        differences += found;
    }
    if differences == 0 {
        println!("ok    {} bindings agree, value for value", reports.len());
        0
    } else {
        println!("FAIL  the bindings disagree on {differences} value(s)");
        1
    }
}
