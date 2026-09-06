//! The determinism lints: the rules a computation crate must keep that no
//! compiler checks (`05-testing/01-quality-bar.md`, "determinism lints").
//!
//! Each is a property of the source rather than of a run, so each is read
//! off the source: an unordered collection whose iteration could reach an
//! output, a read of the clock or the environment inside a computation,
//! an `unsafe` allowance outside the two places that may have one, and
//! the classification functions that must stay integer arithmetic.
//!
//! A line that must break a rule says so with a `lint:` marker naming the
//! rule and the reason; the gate prints those, so an allowance is an
//! inventory rather than a silence.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// The crates whose code computes an answer: what they iterate, read and
/// round is what a chart is made of. The tooling crates (`idl`, `xtask`)
/// and the boundary (`ffi`) are held to the compiler's lints alone.
const COMPUTATION: [&str; 8] = [
    "core",
    "calendar",
    "time",
    "astro",
    "siddhanta",
    "port-ephemeris",
    "port-timezone",
    "intl",
];

/// The crates that may hold `unsafe` code, and so may downgrade the
/// workspace's `forbid` on it: the port's C vtable, the boundary crate
/// and the Node addon. Everything else inherits `forbid`, which the
/// compiler then enforces; what this rule watches is a manifest quietly
/// changing its mind.
const UNSAFE_CRATES: [&str; 3] = [
    "crates/port-ephemeris",
    "crates/ffi",
    "bindings/node/native",
];

/// The classification functions of `core::angle`: exact integer
/// arithmetic on nanoarcseconds, which `const fn` guarantees, because
/// stable Rust has no floating-point arithmetic in a `const fn`
/// (ADR-0016).
const CLASSIFIERS: [&str; 7] = [
    "division_index",
    "sign_index",
    "nakshatra_index",
    "pada",
    "pada_global",
    "in_sign",
    "in_division",
];

/// One thing the gate found: where, and what to say about it.
struct Finding {
    file: String,
    line: usize,
    text: String,
    rule: &'static str,
}

/// A rule, its matches and the allowances it was given.
#[derive(Default)]
struct Outcome {
    failures: Vec<Finding>,
    allowed: Vec<Finding>,
}

/// Every `.rs` file under a directory, sorted.
fn sources(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

/// The lines of a file that are not inside a `#[cfg(test)]` item: a test
/// may read the clock and iterate a hash map, because what it computes
/// reaches nobody.
fn outside_tests(text: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut depth = 0isize;
    let mut skipping = false;
    let mut skip_depth = 0isize;
    let mut pending = false;
    for (number, line) in text.lines().enumerate() {
        let opens = isize::try_from(line.matches('{').count()).unwrap_or(0);
        let closes = isize::try_from(line.matches('}').count()).unwrap_or(0);
        if line.trim_start().starts_with("#[cfg(test)]") {
            pending = true;
        }
        if !skipping && pending && opens > 0 {
            skipping = true;
            skip_depth = depth;
            pending = false;
        }
        if !skipping {
            out.push((number + 1, line));
        }
        depth += opens - closes;
        if skipping && depth <= skip_depth {
            skipping = false;
        }
    }
    out
}

/// Whether a line asks to be excused, and from which rule.
fn excused(line: &str, rule: &str) -> bool {
    line.contains(&format!("lint: {rule}"))
}

/// A rule over the computation crates' lines.
fn scan(root: &Path, rule: &'static str, needles: &[&str], outcome: &mut Outcome) {
    let mut seen_files: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for crate_name in COMPUTATION {
        let dir = root.join("crates").join(crate_name).join("src");
        for path in sources(&dir) {
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            let shown = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .display()
                .to_string();
            // A file may excuse itself once, in its header, when every
            // use in it has the same reason.
            let file_excused = text
                .lines()
                .take_while(|line| line.starts_with("//!") || line.trim().is_empty())
                .any(|line| line.contains(&format!("lint: {rule}")));
            for (number, line) in outside_tests(&text) {
                if line.trim_start().starts_with("//") {
                    continue;
                }
                let Some(needle) = needles.iter().find(|needle| line.contains(**needle)) else {
                    continue;
                };
                let finding = Finding {
                    file: shown.clone(),
                    line: number,
                    text: format!("`{needle}` in {}", line.trim()),
                    rule,
                };
                if file_excused && !seen_files.insert(shown.clone()) {
                    // One line of inventory per file, not one per use.
                    continue;
                }
                if file_excused || excused(line, rule) {
                    outcome.allowed.push(finding);
                } else {
                    outcome.failures.push(finding);
                }
            }
        }
    }
}

/// The `unsafe` inventory: exactly the crates that may hold unsafe code
/// downgrade the workspace's `forbid`, and each says why in its manifest.
fn unsafe_inventory(root: &Path, outcome: &mut Outcome) {
    let mut manifests: Vec<PathBuf> = Vec::new();
    for dir in [root.join("crates"), root.join("bindings")] {
        let mut stack = vec![dir];
        while let Some(here) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&here) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && path.file_name().is_some_and(|n| n != "target") {
                    stack.push(path);
                } else if path.file_name().is_some_and(|n| n == "Cargo.toml") {
                    manifests.push(path);
                }
            }
        }
    }
    manifests.sort();
    for manifest in manifests {
        let Ok(text) = std::fs::read_to_string(&manifest) else {
            continue;
        };
        let downgraded = text
            .lines()
            .any(|line| line.trim_start().starts_with("unsafe_code =") && !line.contains("forbid"));
        let crate_dir = manifest
            .parent()
            .and_then(|dir| dir.strip_prefix(root).ok())
            .map(|dir| dir.display().to_string())
            .unwrap_or_default();
        let expected = UNSAFE_CRATES.contains(&crate_dir.as_str());
        match (downgraded, expected) {
            (true, true) => outcome.allowed.push(Finding {
                file: format!("{crate_dir}/Cargo.toml"),
                line: 1,
                text: String::from("holds unsafe code, and says so in its manifest"),
                rule: "unsafe-inventory",
            }),
            (true, false) => outcome.failures.push(Finding {
                file: format!("{crate_dir}/Cargo.toml"),
                line: 1,
                text: String::from(
                    "downgrades the workspace's `forbid` on unsafe code; only the port, the boundary and the addon may",
                ),
                rule: "unsafe-inventory",
            }),
            (false, true) => outcome.failures.push(Finding {
                file: format!("{crate_dir}/Cargo.toml"),
                line: 1,
                text: String::from(
                    "is on the list of crates that may hold unsafe code but forbids it; take it off the list",
                ),
                rule: "unsafe-inventory",
            }),
            (false, false) => {}
        }
    }
}

/// The classification functions are `const fn`, which in stable Rust
/// cannot compute in floating point.
fn exact_classification(root: &Path, outcome: &mut Outcome) {
    let path = root.join("crates/core/src/angle.rs");
    let Ok(text) = std::fs::read_to_string(&path) else {
        outcome.failures.push(Finding {
            file: String::from("crates/core/src/angle.rs"),
            line: 1,
            text: String::from("cannot be read"),
            rule: "exact-classification",
        });
        return;
    };
    for name in CLASSIFIERS {
        let signature = format!(" fn {name}(");
        let Some((number, line)) = text
            .lines()
            .enumerate()
            .map(|(index, line)| (index + 1, line))
            .find(|(_, line)| line.contains(&signature))
        else {
            continue;
        };
        let finding = Finding {
            file: String::from("crates/core/src/angle.rs"),
            line: number,
            text: format!("`{name}` classifies"),
            rule: "exact-classification",
        };
        if line.contains("const fn") {
            outcome.allowed.push(finding);
        } else {
            outcome.failures.push(Finding {
                text: format!(
                    "`{name}` is not a `const fn`, so nothing stops it computing in floating point"
                ),
                ..finding
            });
        }
    }
}

pub(crate) fn check(root: &Path) -> i32 {
    let mut outcome = Outcome::default();
    scan(
        root,
        "deterministic-iteration",
        &["HashMap", "HashSet", "hash_map", "hash_set"],
        &mut outcome,
    );
    scan(
        root,
        "ambient-input",
        &[
            "SystemTime::now",
            "Instant::now",
            "std::env::",
            "env::var",
            "env::args",
            "std::process::id",
        ],
        &mut outcome,
    );
    unsafe_inventory(root, &mut outcome);
    exact_classification(root, &mut outcome);

    let mut report = String::new();
    for rule in [
        "deterministic-iteration",
        "ambient-input",
        "unsafe-inventory",
        "exact-classification",
    ] {
        let failures = outcome.failures.iter().filter(|f| f.rule == rule).count();
        let allowed: Vec<&Finding> = outcome.allowed.iter().filter(|f| f.rule == rule).collect();
        if failures == 0 {
            println!("ok    {rule}: {} allowed", allowed.len());
        } else {
            println!("FAIL  {rule}: {failures} unallowed");
        }
        for finding in allowed {
            let _ = writeln!(
                report,
                "      {}:{} {}",
                finding.file, finding.line, finding.text
            );
        }
    }
    print!("{report}");
    for finding in &outcome.failures {
        println!(
            "FAIL  {}:{} {} ({})",
            finding.file, finding.line, finding.text, finding.rule
        );
    }
    i32::from(!outcome.failures.is_empty())
}
