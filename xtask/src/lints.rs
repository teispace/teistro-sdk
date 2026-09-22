//! The determinism lints: the rules a computation crate must keep that no
//! compiler checks (`05-testing/01-quality-bar.md`, "determinism lints").
//!
//! Each is a property of the source rather than of a run, so each is read
//! off the source: an unordered collection whose iteration could reach an
//! output, a read of the clock or the environment inside a computation,
//! an `unsafe` allowance outside the two places that may have one, the
//! classification functions that must stay integer arithmetic, and a
//! settings knob that ships and resolves and is read by nobody.
//!
//! Two of them are properties of the repository rather than of a crate,
//! and are here for the same reason: a workflow file that GitHub cannot
//! parse, and a gate that is declared and that no workflow runs. Neither
//! is compiled by anything, and both silence other gates when they
//! break.
//!
//! A line that must break a rule says so with a `lint:` marker naming the
//! rule and the reason; the gate prints those, so an allowance is an
//! inventory rather than a silence.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// The crates whose code computes an answer: what they iterate, read and
/// round is what a chart is made of. The tooling crates (`idl`, `xtask`)
/// and the two surfaces (`ffi`, the C boundary, and `sdk`, the Rust
/// façade) are held to the compiler's lints alone — a surface composes
/// what these compute and rounds nothing itself.
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
/// workspace's `forbid` on it: the port's C vtable, the boundary crate,
/// the Node addon, and the counting allocator the tests install, which is
/// never published. Everything else inherits `forbid`, which the compiler
/// then enforces; what this rule watches is a manifest quietly changing
/// its mind.
const UNSAFE_CRATES: [&str; 6] = [
    "crates/port-ephemeris",
    "crates/ffi",
    "crates/test-allocator",
    "bindings/node/native",
    // The adapters call a C engine directly. They are outside the
    // workspace, which is why they were outside this inventory until
    // 2026-09-12 — and being outside the workspace is no reason to be
    // outside the inventory, because an adapter is now a shared library
    // a consumer loads into their own process (ADR-0029).
    "adapters/ephemeris-teimeris/rust",
    "adapters/ephemeris-sweph/rust",
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
pub(crate) fn sources(root: &Path) -> Vec<PathBuf> {
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
    for dir in [
        root.join("crates"),
        root.join("bindings"),
        root.join("adapters"),
    ] {
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

/// Where the knobs are declared and where a patch is applied: a mention
/// here is not a reader, because this is the layer whose job is to carry
/// them.
const SETTINGS_LAYER: [&str; 3] = [
    "settings/mod.rs",
    "settings/profiles.rs",
    "settings/knobs.rs",
];

/// Every settings knob has a reader outside the settings layer.
///
/// A knob that ships, resolves and is read by nobody is a bug whether or
/// not anything crashes, and three had been found by hand in as many
/// modules before this rule existed: `state.combustion_orbs`, which made
/// a chart founded on the SDK's own default profile fail
/// (`05-testing/01-golden-vectors.md`, entry 23);
/// `houses.module_overrides`, which quietly gave a KP reading whole-sign
/// houses; and `output.precision`, which did nothing at all.
///
/// The knob list comes from `core` itself
/// ([`teistro_core::settings::Settings::knob_paths`]), so a group added
/// to the document is watched without a second list to remember.
///
/// A knob whose module is not written yet says so where it is declared,
/// with a `lint: knob-has-a-reader` marker naming what will read it. The
/// gate prints those, so a deferral is an inventory rather than a
/// silence — **and an allowance that is no longer needed is itself a
/// finding**, so the inventory cannot rot.
fn knob_readers(root: &Path, outcome: &mut Outcome) {
    const RULE: &str = "knob-has-a-reader";
    // A chain broken across lines by the formatter still reads as one
    // access, so the search is over the text with its whitespace gone.
    let mut haystacks: Vec<String> = Vec::new();
    for directory in ["crates", "bindings", "xtask"] {
        for path in sources(&root.join(directory)) {
            let shown = path.display().to_string();
            if SETTINGS_LAYER.iter().any(|layer| shown.ends_with(layer)) {
                continue;
            }
            if let Ok(text) = std::fs::read_to_string(&path) {
                haystacks.push(text.split_whitespace().collect());
            }
        }
    }

    let declarations = root.join("crates/core/src/settings/mod.rs");
    let source = std::fs::read_to_string(&declarations).unwrap_or_default();
    let shown = declarations
        .strip_prefix(root)
        .unwrap_or(&declarations)
        .display()
        .to_string();

    for (group, knob) in teistro_core::settings::Settings::knob_paths() {
        let access = format!(".{group}.{knob}");
        let read = haystacks.iter().any(|text| text.contains(&access));
        let (line, marker) = declaration_of(&source, knob);
        match (read, marker) {
            (false, None) => outcome.failures.push(Finding {
                file: shown.clone(),
                line,
                text: format!(
                    "`{group}.{knob}` has no reader; give it one, or say `lint: {RULE}` \
                     where it is declared with what will read it"
                ),
                rule: RULE,
            }),
            (false, Some(reason)) => outcome.allowed.push(Finding {
                file: shown.clone(),
                line,
                text: format!("{group}.{knob}: {reason}"),
                rule: RULE,
            }),
            (true, Some(_)) => outcome.failures.push(Finding {
                file: shown.clone(),
                line,
                text: format!(
                    "`{group}.{knob}` is read now, so its `lint: {RULE}` allowance is stale \
                     and should go"
                ),
                rule: RULE,
            }),
            (true, None) => {}
        }
    }
}

/// Where a knob is declared, and the reason its allowance gives if it
/// has one.
///
/// The marker goes in the knob's own doc comment, so it is read by
/// whoever reads the knob rather than living in a list elsewhere.
fn declaration_of(source: &str, knob: &str) -> (usize, Option<String>) {
    let wanted = format!("{knob}: ");
    let mut reason: Option<String> = None;
    for (index, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("/// lint: knob-has-a-reader") {
            reason = Some(rest.trim_start_matches([' ', '—', '-']).trim().to_string());
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("///") {
            // A reason may run over several lines; it reads as one.
            if let Some(started) = reason.as_mut() {
                let rest = rest.trim();
                if !rest.is_empty() {
                    started.push(' ');
                    started.push_str(rest);
                }
            }
            continue;
        }
        if trimmed.starts_with(&wanted) {
            return (index + 1, reason);
        }
        reason = None;
    }
    (0, None)
}

/// The workflow files, sorted: the only source in the repository that
/// GitHub parses and rustc does not, and the only place a gate is
/// actually run.
fn workflows(root: &Path) -> Vec<PathBuf> {
    let dir = root.join(".github").join("workflows");
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .extension()
                .is_some_and(|extension| extension == "yml" || extension == "yaml")
            {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

/// The gate a workflow line runs, if it runs one. The name is taken as a
/// whole word rather than as a substring, because `check-doc` is a
/// prefix of `check-docs`: a substring test would let a renamed gate
/// pass while nothing ran it, which is the failure the rule exists for.
fn gate_run_by(line: &str) -> Option<&str> {
    let rest = line.split_once("cargo xtask ")?.1;
    rest.split([' ', '"', '\''])
        .next()
        .filter(|name| name.starts_with("check-"))
}

/// The gate an arm of `xtask`'s entry point declares.
fn gate_declared_by(line: &str) -> Option<&str> {
    let rest = line.trim_start().strip_prefix("Some(\"")?;
    let name = rest.split_once('"')?.0;
    name.starts_with("check-").then_some(name)
}

/// Which line of the entry point holds a pass's row, so a finding points
/// at something a reader can open. Zero when it cannot be found, which
/// is a worse message and not a wrong one.
fn pass_row(root: &Path, name: &str) -> usize {
    let entry = root.join("xtask").join("src").join("main.rs");
    let Ok(text) = std::fs::read_to_string(&entry) else {
        return 0;
    };
    let needle = format!("(\"{name}\", ");
    text.lines()
        .position(|line| line.trim_start().starts_with(&needle))
        .map_or(0, |index| index + 1)
}

/// Every workflow file parses.
///
/// A step name with an unquoted colon, a slipped indent, a duplicated
/// key: GitHub answers a file it cannot read by failing the run in zero
/// seconds, without starting a single step and without naming what is
/// wrong. A suite of thirty gates then stops running, and the only sign
/// of it is a red tick that says nothing. Nothing but a parse catches
/// that before the push, and rustc never reads these files.
fn workflows_parse(root: &Path, outcome: &mut Outcome) {
    for path in workflows(root) {
        let shown = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .display()
            .to_string();
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        if let Err(error) = serde_yaml_ng::from_str::<serde_yaml_ng::Value>(&text) {
            outcome.failures.push(Finding {
                file: shown,
                line: error.location().map_or(0, |place| place.line()),
                text: error.to_string(),
                rule: "workflow-parses",
            });
        }
    }
}

/// Every gate is run by a workflow.
///
/// This is `knob-has-a-reader` one level up. A pass that regenerates a
/// page in memory and fails on any difference holds that page only for
/// as long as something runs it; declared and unwired, it is a gate the
/// documentation claims and the repository does not have. A gate meant
/// to be run by hand says so on its arm, so the exception is an
/// inventory rather than a silence.
///
/// Two sources, because `xtask` declares its gates two ways. The
/// hand-written arms spell `Some("check-...")` and are found by reading
/// the file; the generated pages are rows of [`crate::PASSES`] whose
/// gate name is `check-` and the row's own, and were **not covered at
/// all** until this asked the table — nineteen pages whose gates the
/// rule could not see, four of which no workflow ran.
fn gate_runners(root: &Path, outcome: &mut Outcome) {
    let mut wired: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for path in workflows(root) {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        for line in text.lines() {
            if let Some(name) = gate_run_by(line) {
                wired.insert(name.to_owned());
            }
        }
    }
    for (name, _, _) in crate::PASSES {
        let gate = format!("check-{name}");
        if wired.contains(&gate) {
            continue;
        }
        outcome.failures.push(Finding {
            file: "xtask/src/main.rs".to_owned(),
            line: pass_row(root, name),
            text: format!("`cargo xtask {gate}` is declared and no workflow runs it"),
            rule: "gate-has-a-runner",
        });
    }
    let entry = root.join("xtask").join("src").join("main.rs");
    let Ok(text) = std::fs::read_to_string(&entry) else {
        return;
    };
    for (number, line) in text.lines().enumerate() {
        let Some(name) = gate_declared_by(line) else {
            continue;
        };
        if wired.contains(name) {
            continue;
        }
        let finding = Finding {
            file: "xtask/src/main.rs".to_owned(),
            line: number + 1,
            text: format!("`cargo xtask {name}` is declared and no workflow runs it"),
            rule: "gate-has-a-runner",
        };
        if excused(line, "gate-has-a-runner") {
            outcome.allowed.push(finding);
        } else {
            outcome.failures.push(finding);
        }
    }
}

/// The hand-written surface of each binding that names the composers.
///
/// A `PlanRequest` crosses the boundary as **JSON**, not as a struct, so
/// none of these is generated from the API description the way the rest of
/// a binding is: each language spells the record itself. That is three
/// copies of one list, which is the shape this repository keeps finding,
/// and nothing held them together until a composer was added twice and
/// each binding had to be remembered.
const PLAN_SURFACES: [(&str, [&str; 2]); 3] = [
    (
        "bindings/node/lib/index.d.ts",
        ["interface PlanRequest {", "interface Plans {"],
    ),
    (
        "bindings/dart/lib/teistro.dart",
        ["final class PlanRequest {", "final class PlanRequest {"],
    ),
    (
        "bindings/python/teistro/__init__.py",
        ["class PlanRequest(TypedDict", "class Plans(TypedDict"],
    ),
];

/// The line an anchor is on, one-based, for a finding to point at.
///
/// The newlines before it, and not the lines: an anchor that starts in the
/// first column leaves a trailing newline that `lines` does not count and
/// an indented one does not, so counting lines is right for the boundary
/// declarations and one too many for a row inside a list.
fn line_of(text: &str, anchor: &str) -> usize {
    text.find(anchor).map_or(1, |at| {
        text.get(..at).unwrap_or_default().matches('\n').count() + 1
    })
}

/// The declaration an anchor opens: from the anchor to the first line
/// after it that starts in the first column, which ends a block in each of
/// the three languages — a `}` for TypeScript and Dart, the next `class`
/// or `def` for Python.
///
/// Dart names its request and its record of plans in one class, so both
/// anchors are the same line there and the block is read twice.
fn block_from<'t>(text: &'t str, anchor: &str) -> Option<&'t str> {
    let start = text.find(anchor)?;
    let body = text.get(start..)?;
    let mut end = body.len();
    for (at, _) in body.match_indices('\n') {
        let rest = body.get(at + 1..).unwrap_or_default();
        if rest
            .chars()
            .next()
            .is_some_and(|first| !first.is_whitespace())
        {
            end = at + 1 + rest.find('\n').unwrap_or(rest.len());
            break;
        }
    }
    body.get(..end)
}

/// That every composer a request can name reaches every binding.
///
/// It checks that the member is **named twice**, which is the floor every
/// binding meets: once on the request a caller fills in and once on the
/// record of plans it gets back. One of the two alone is a composer that
/// can be asked for and never read, or read and never asked for. How each
/// spells them is its own business and its own test asserts it.
///
/// What this catches is the failure that actually happens — a composer
/// added to `PlanRequest` and to one or two of the three surfaces — which
/// no other gate sees: `check-parity` compares the values a scenario
/// answers, and a composer nobody can ask for answers nothing.
fn composers_reach_every_binding(root: &Path, outcome: &mut Outcome) {
    const RULE: &str = "composer-reaches-every-binding";
    for (surface, anchors) in PLAN_SURFACES {
        let path = root.join(surface);
        let Ok(text) = std::fs::read_to_string(&path) else {
            outcome.failures.push(Finding {
                file: surface.to_owned(),
                line: 1,
                text: String::from("names the composers and could not be read"),
                rule: RULE,
            });
            continue;
        };
        for anchor in anchors {
            let Some(block) = block_from(&text, anchor) else {
                outcome.failures.push(Finding {
                    file: surface.to_owned(),
                    line: 1,
                    text: format!("`{anchor}` is not in this binding any more"),
                    rule: RULE,
                });
                continue;
            };
            for member in teistro::PlanRequest::MEMBERS {
                if block.contains(member) {
                    continue;
                }
                outcome.failures.push(Finding {
                    file: surface.to_owned(),
                    line: line_of(&text, anchor),
                    text: format!(
                        "`PlanRequest::MEMBERS` has `{member}` and `{anchor}` does not name it"
                    ),
                    rule: RULE,
                });
            }
        }
    }
}

/// The register of questions, and the tracker that must name the open ones.
const REGISTER: (&str, &str) = ("docs/QUESTIONS.md", "docs/STATUS.md");

/// The phrases that count the open questions in prose, which is the one
/// thing about them that has gone stale twice.
///
/// A count beside a register that grows is the rot this repository keeps
/// finding, and this instance is worse than most: the sentence sits in the
/// **first numbered step of "How to resume"**, which is the first thing a
/// reader is told to trust. It said "none is open" with one open, then
/// "one question is open" with two.
/// Three and not more: the last two subsume every counted phrasing, and
/// a longer list would report one occurrence twice.
const COUNTED_IN_PROSE: [&str; 3] = ["none is open", "question is open", "questions are open"];

/// That the tracker names every question the register still has open, and
/// counts none of them.
///
/// Both halves matter. A question opened and not mentioned leaves a reader
/// resuming from a document that does not know about it; a **count** is
/// right on the day it is written and wrong on the next.
fn open_questions_are_named(root: &Path, outcome: &mut Outcome) {
    const RULE: &str = "open-question-is-named";
    let (register, tracker) = REGISTER;
    let (Ok(questions), Ok(status)) = (
        std::fs::read_to_string(root.join(register)),
        std::fs::read_to_string(root.join(tracker)),
    ) else {
        return;
    };
    for (at, line) in questions.lines().enumerate() {
        let Some(rest) = line.strip_prefix("## ") else {
            continue;
        };
        if !line.ends_with(": `open`") {
            continue;
        }
        let Some(number) = rest.split('.').next() else {
            continue;
        };
        if !status.contains(number) {
            outcome.failures.push(Finding {
                file: register.to_owned(),
                line: at + 1,
                text: format!("`{number}` is open and {tracker} never names it"),
                rule: RULE,
            });
        }
    }
    for phrase in COUNTED_IN_PROSE {
        if status.contains(phrase) {
            outcome.failures.push(Finding {
                file: tracker.to_owned(),
                line: line_of(&status, phrase),
                text: format!(
                    "`{phrase}` counts the open questions in prose, and a count is right for one \
                     day; name them instead and let {register} be the authority"
                ),
                rule: RULE,
            });
        }
    }
}

/// The tracker's table of what is built, and where its rows come from.
const CRATE_TABLE: (&str, &str) = ("docs/STATUS.md", "| crate | what it is |");

/// That every crate in the workspace is named in the tracker's table, and
/// that the table names no crate that is not there.
///
/// A hand-kept list beside a directory that grows is the rot this
/// repository keeps finding in its own prose: the table was missing
/// **eight** crates on 2026-09-22 — `rules`, `interpret`, `dasha`,
/// `strength`, `sdk`, `geometry`, `render-svg` and `ephemeris-builtin`,
/// which is most of what Phases 4 to 6 produced — and nothing noticed
/// because nothing compared it to `crates/`. The gate list two items above
/// it went stale the same way and became a pointer; this one is a table
/// with a sentence a crate, so it is held instead.
///
/// A row may name more than one crate (`` `time`, `port-timezone` ``), so
/// every backticked name in a row's **first** cell counts. Only the first:
/// the `chart` row's description names `day`, `bhava`, `zodiac` and
/// `foundation`, which are modules and not crates, and a looser read
/// reports four crates that do not exist.
fn crates_are_listed(root: &Path, outcome: &mut Outcome) {
    const RULE: &str = "crate-is-listed";
    let (tracker, heading) = CRATE_TABLE;
    let Ok(text) = std::fs::read_to_string(root.join(tracker)) else {
        return;
    };
    if !text.contains(heading) {
        outcome.failures.push(Finding {
            file: tracker.to_owned(),
            line: 1,
            text: format!("`{heading}` is not in the tracker any more"),
            rule: RULE,
        });
        return;
    }
    let listed: std::collections::BTreeSet<&str> = text
        .lines()
        .skip_while(|line| !line.contains(heading))
        .map(str::trim_start)
        .take_while(|line| line.starts_with('|'))
        .filter_map(|row| row.split('|').nth(1))
        .flat_map(|cell| cell.split('`').skip(1).step_by(2))
        .collect();
    let line = line_of(&text, heading);
    let Ok(entries) = std::fs::read_dir(root.join("crates")) else {
        return;
    };
    let mut present: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for entry in entries.flatten() {
        if entry.path().join("Cargo.toml").is_file()
            && let Some(name) = entry.file_name().to_str()
        {
            present.insert(name.to_owned());
        }
    }
    for name in &present {
        if !listed.contains(name.as_str()) {
            outcome.failures.push(Finding {
                file: tracker.to_owned(),
                line,
                text: format!("`crates/{name}` is built and the tracker's table never names it"),
                rule: RULE,
            });
        }
    }
    for name in &listed {
        if !present.contains(*name) {
            outcome.failures.push(Finding {
                file: tracker.to_owned(),
                line,
                text: format!("the tracker's table names `{name}` and `crates/` has no such crate"),
                rule: RULE,
            });
        }
    }
}

/// Every boundary module the description is read from.
///
/// The API description is extracted from a hand-written list of sources
/// ([`teistro_idl::sdk::SOURCES`]), and the C header and all three
/// bindings are generated from that description. A module of the `ffi`
/// crate that holds entry points and is **not** on that list is
/// therefore invisible: it compiles, it exports its symbols, and no
/// binding has ever heard of it. Nothing compared the list against the
/// crate, which is `gate-has-a-runner` a third time — an artefact that
/// exists and nothing reads — so this does.
///
/// A module with no `#[unsafe(no_mangle)]` entry point has nothing to
/// describe and is not expected on the list.
fn boundary_sources(root: &Path, outcome: &mut Outcome) {
    const RULE: &str = "boundary-is-described";
    let listed: std::collections::BTreeSet<&str> =
        teistro_idl::sdk::SOURCES.iter().copied().collect();
    for path in sources(&root.join("crates/ffi/src")) {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        if !text.contains("#[unsafe(no_mangle)]") {
            continue;
        }
        let shown = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .display()
            .to_string()
            .replace('\\', "/");
        if listed.contains(shown.as_str()) {
            continue;
        }
        let excused = text
            .lines()
            .take_while(|line| line.starts_with("//!") || line.trim().is_empty())
            .any(|line| line.contains(&format!("lint: {RULE}")));
        let finding = Finding {
            file: shown,
            line: 1,
            text: String::from(
                "holds entry points and is not in `teistro_idl::sdk::SOURCES`, \
                 so no binding has heard of them",
            ),
            rule: RULE,
        };
        if excused {
            outcome.allowed.push(finding);
        } else {
            outcome.failures.push(finding);
        }
    }
}

/// Every entry point of the boundary is reached by some emitter.
///
/// `boundary-is-described` holds that a *file* of entry points is on the
/// description's source list. This holds the next thing along: that each
/// **function** on that list is placed by one of the rules the emitters
/// group by — a free function, an opaque's constructor, one of its
/// factories, a method of one, its destructor or its last-error reader.
/// A function none of them matches is described, generated into every
/// `extern` declaration, and reachable from no binding.
///
/// It was written because one had been in exactly that state:
/// `ts_context_new_with_provider` takes a `handle` of `TsProvider` that
/// is not its first parameter, so it is nobody's method; and `TsContext`
/// already had a constructor, so it is not that either. The plugin route
/// the whole of ADR-0029 is about therefore stopped at the boundary in
/// every binding but Rust, and nothing said so.
fn entry_points_reachable(root: &Path, outcome: &mut Outcome) {
    const RULE: &str = "entry-point-is-reachable";
    let Ok(text) = std::fs::read_to_string(root.join("idl").join("api.json")) else {
        return;
    };
    let Ok(api) = serde_json::from_str::<teistro_idl::model::Api>(&text) else {
        return;
    };
    let mut placed: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for function in &api.functions {
        if teistro_idl::rules::is_free_function(function) {
            placed.insert(&function.name);
        }
    }
    for opaque in &api.opaques {
        for found in [
            teistro_idl::rules::constructor(&api, opaque),
            teistro_idl::rules::destructor(&api, opaque),
            teistro_idl::rules::last_error(&api, opaque),
        ]
        .into_iter()
        .flatten()
        {
            placed.insert(&found.name);
        }
        for method in teistro_idl::rules::methods(&api, opaque) {
            placed.insert(&method.name);
        }
        // A second way in, beside the constructor: the rule this lint's
        // own first run called for.
        for factory in teistro_idl::rules::factories(&api, opaque) {
            placed.insert(&factory.name);
        }
    }
    for function in &api.functions {
        if placed.contains(function.name.as_str()) {
            continue;
        }
        let deferred = AWAITING_AN_EMITTER
            .iter()
            .find(|(name, _)| *name == function.name);
        let finding = Finding {
            file: function.source.clone(),
            line: 1,
            text: match deferred {
                Some((name, awaiting)) => format!("`{name}` is unplaced: {awaiting}"),
                None => format!(
                    "`{}` matches no emitter rule — not a free function, and no opaque's \
                     constructor, factory, method, destructor or last-error reader — so it \
                     is described and reachable from no binding",
                    function.name
                ),
            },
            rule: RULE,
        };
        if deferred.is_some() {
            outcome.allowed.push(finding);
        } else {
            outcome.failures.push(finding);
        }
    }
}

/// Entry points no emitter rule places yet, each with what will place
/// it.
///
/// **An inventory rather than a silence**, which is what
/// `knob-has-a-reader` keeps for the same reason: the gate prints every
/// row on every run, so a deferral is read rather than forgotten, and a
/// row that has been placed since becomes a stale allowance — itself a
/// failure of that rule's own kind.
///
/// Empty, and it was not: `ts_context_new_with_provider` sat here for
/// exactly as long as it took to write `rules::factories` and teach the
/// three emitters a second way in. Leaving the list in place is the
/// point — the next one has somewhere to be declared.
const AWAITING_AN_EMITTER: [(&str, &str); 0] = [];

/// Python is spawned in one place, and that place puts it in UTF-8 mode.
///
/// Two halves of one property, both read off `xtask`'s own source: the
/// interpreter is named once, and no module builds a Python `Command`
/// of its own. `binding::python_command` is that place, and the reason
/// it has to be one place is in its doc comment -- `PYTHONUTF8`, which
/// every program that prints what this SDK returns needs on Windows.
///
/// Born of a fix that went into one gate when four needed it: the
/// binding's examples were fixed and `check-parity`'s runner failed on
/// the next run with the same `UnicodeEncodeError`. A rule over the
/// source catches the third and fourth without another matrix run.
fn python_in_utf8(root: &Path, outcome: &mut Outcome) {
    const RULE: &str = "python-runs-in-utf8-mode";
    for path in sources(&root.join("xtask/src")) {
        // Not this file: a rule cannot be written without naming what it
        // looks for, and its own needles are not Python being spawned.
        if path.file_name().is_some_and(|name| name == "lints.rs") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let shown = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .display()
            .to_string();
        for (number, line) in outside_tests(&text) {
            if line.trim_start().starts_with("//") {
                continue;
            }
            let builds_one =
                line.contains("Command::new(") && line.to_lowercase().contains("python");
            let names_the_variable = line.contains(r#"env::var("PYTHON")"#);
            if !builds_one && !names_the_variable {
                continue;
            }
            let finding = Finding {
                file: shown.clone(),
                line: number,
                text: format!("`{}`", line.trim()),
                rule: RULE,
            };
            if excused(line, RULE) {
                outcome.allowed.push(finding);
            } else {
                outcome.failures.push(finding);
            }
        }
    }
}

/// Every target of the façade crate that names a feature-gated
/// ephemeris declares the feature it needs.
///
/// That a tier feature turns on the base feature it refines.
///
/// A crate whose sources read `#[cfg(feature = "builtin-ephemeris")]` and
/// whose manifest offers `builtin-compact`, `builtin-standard` and
/// `builtin-full` has four features and one guard: a tier that forwards to
/// a dependency without also naming the crate's own base feature builds a
/// library with a built-in ephemeris underneath and no way to ask for it.
///
/// That is not hypothetical either. `teistro-ffi`'s tiers forwarded only —
/// `builtin-standard = ["teistro/builtin-standard"]` — so every tier build
/// refused `TsEphemeris::Builtin` as `UNSUPPORTED`, and the verify matrix's
/// three tier jobs failed on it. The façade next door had the same four
/// features written the right way, which is the shape this project keeps
/// meeting: one rule, two places, right in one of them.
///
/// The rule reads both the sources and the manifest, so a crate that stops
/// gating on the feature stops being held to it, and one that starts cannot
/// be added without the line.
fn tiers_turn_on_their_base(root: &Path, outcome: &mut Outcome) {
    const RULE: &str = "a-tier-turns-on-its-base";
    const BASE: &str = "builtin-ephemeris";
    for crate_dir in ["crates/ffi", "crates/sdk"] {
        let manifest = root.join(crate_dir).join("Cargo.toml");
        let Ok(text) = std::fs::read_to_string(&manifest) else {
            continue;
        };
        // Only a crate that actually gates on the base feature can be
        // broken by a tier that leaves it off.
        if !gates_on(&root.join(crate_dir).join("src"), BASE) {
            continue;
        }
        for (at, line) in text.lines().enumerate() {
            let Some((name, rest)) = line.split_once('=') else {
                continue;
            };
            let name = name.trim();
            if !name.starts_with("builtin-") || name == BASE || !rest.contains('[') {
                continue;
            }
            if !rest.contains(BASE) {
                outcome.failures.push(Finding {
                    file: format!("{crate_dir}/Cargo.toml"),
                    line: at + 1,
                    text: format!(
                        "`{name}` forwards a tier without turning on `{BASE}`, so this \
                         crate's own `cfg` stays off and the built-in cannot be asked for"
                    ),
                    rule: RULE,
                });
            }
        }
    }
}

/// Whether any source under `dir` gates on `feature`.
fn gates_on(dir: &Path, feature: &str) -> bool {
    let needle = format!("feature = \"{feature}\"");
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    entries.flatten().any(|entry| {
        let path = entry.path();
        if path.is_dir() {
            return gates_on(&path, feature);
        }
        path.extension().is_some_and(|ext| ext == "rs")
            && std::fs::read_to_string(&path).is_ok_and(|text| text.contains(&needle))
    })
}

/// `Ephemeris::Builtin` exists only under `builtin-ephemeris`, so an
/// example or a test that names it and does **not** carry
/// `required-features` breaks a `--no-default-features` build of the
/// crate instead of being skipped by it. That is not hypothetical: the
/// eight examples and `tests/surface.rs` all had the hole, and nobody
/// saw it because nothing had ever built this crate without its
/// default — the ephemeris tier matrix builds
/// `teistro-ephemeris-builtin` and `teistro-ffi`, not this.
///
/// The rule reads the source rather than a list, so an example that
/// stops naming the built-in stops needing the line, and a ninth that
/// names it cannot be added without one.
fn targets_declare_their_features(root: &Path, outcome: &mut Outcome) {
    const RULE: &str = "target-declares-the-feature-it-needs";
    /// The feature the variant lives behind, and the variant.
    const GATED: (&str, &str) = ("builtin-ephemeris", "Ephemeris::Builtin");
    let manifest = root.join("crates/sdk/Cargo.toml");
    let Ok(manifest_text) = std::fs::read_to_string(&manifest) else {
        outcome.failures.push(Finding {
            file: String::from("crates/sdk/Cargo.toml"),
            line: 0,
            text: String::from("the façade's manifest could not be read"),
            rule: RULE,
        });
        return;
    };
    for (kind, directory) in [
        ("example", "crates/sdk/examples"),
        ("test", "crates/sdk/tests"),
    ] {
        for path in sources(&root.join(directory)) {
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            if !text.contains(GATED.1) {
                continue;
            }
            // A shared module (`tests/common/mod.rs`) is no target of its
            // own: every target that declares it (`mod common;`) names what
            // it names, so each of those must require the feature.
            let targets: Vec<String> = if path.file_name().is_some_and(|f| f == "mod.rs") {
                let Some(module) = path
                    .parent()
                    .and_then(Path::file_name)
                    .map(|m| m.to_string_lossy().to_string())
                else {
                    continue;
                };
                sources(&root.join(directory))
                    .into_iter()
                    .filter(|target| target.parent() == Some(&root.join(directory)))
                    .filter(|target| {
                        std::fs::read_to_string(target)
                            .is_ok_and(|body| body.contains(&format!("mod {module};")))
                    })
                    .filter_map(|target| {
                        target
                            .file_stem()
                            .map(|stem| stem.to_string_lossy().to_string())
                    })
                    .collect()
            } else {
                path.file_stem()
                    .map(|stem| stem.to_string_lossy().to_string())
                    .into_iter()
                    .collect()
            };
            for name in targets {
                // The manifest section for this target, and whether it
                // requires the feature. Read as text because the question is
                // whether two lines sit together, which is what a reader
                // checking the manifest by eye would look for.
                let header = format!("[[{kind}]]\nname = \"{name}\"");
                let requires = manifest_text.split_once(&header).is_some_and(|(_, after)| {
                    after
                        .split("\n[")
                        .next()
                        .is_some_and(|section| section.contains(GATED.0))
                });
                if requires {
                    continue;
                }
                let shown = path
                    .strip_prefix(root)
                    .unwrap_or(&path)
                    .display()
                    .to_string();
                outcome.failures.push(Finding {
                    file: shown,
                    line: 0,
                    text: format!(
                        "names `{}` but no `[[{kind}]] name = \"{name}\"` requires `{}`",
                        GATED.1, GATED.0
                    ),
                    rule: RULE,
                });
            }
        }
    }
}

/// Every type a document-carrying crate serialises can describe itself.
///
/// The document schema is generated from `schemars::JsonSchema`, and a
/// type that serialises without one either breaks the generator (when the
/// document reaches it) or waits to (when a field is added that does).
/// So the rule is not "the types the document reaches" but **every type
/// that derives `Serialize` in a crate with the `schema` feature**, which
/// a reader can check without knowing the document
/// (`docs/03-design/document-schema.md` §7).
///
/// Two forms, because a type serialises in two ways: a derive, which must
/// carry the `cfg_attr` derive beside it, and a hand-written
/// `impl Serialize for T`, which must have a hand-written schema for the
/// same `T` in the same crate — `impl schemars::JsonSchema for T` or the
/// core crate's `hand_schema!(T, …)`.
fn serialised_types_describe_themselves(root: &Path, outcome: &mut Outcome) {
    const RULE: &str = "serialised-type-describes-itself";
    const DERIVE: &str = "derive(schemars::JsonSchema)";
    let Ok(crates) = std::fs::read_dir(root.join("crates")) else {
        return;
    };
    let mut crates: Vec<PathBuf> = crates.flatten().map(|entry| entry.path()).collect();
    crates.sort();
    let hand_impl = regex::Regex::new(
        r"impl(?:<[^>]*>)?\s+(?:serde::)?Serialize\s+for\s+([A-Za-z_][A-Za-z0-9_]*)",
    )
    .unwrap_or_else(|error| panic!("the pattern compiles: {error}"));
    for krate in crates {
        let has_feature = std::fs::read_to_string(krate.join("Cargo.toml"))
            .is_ok_and(|manifest| manifest.lines().any(|line| line.starts_with("schema = ")));
        if !has_feature {
            continue;
        }
        let files: Vec<(PathBuf, String)> = sources(&krate.join("src"))
            .into_iter()
            .filter_map(|path| std::fs::read_to_string(&path).ok().map(|text| (path, text)))
            .collect();
        let described = |name: &str| {
            files.iter().any(|(_, text)| {
                text.contains(&format!("impl schemars::JsonSchema for {name} "))
                    || text.contains(&format!("hand_schema!({name},"))
            })
        };
        for (path, text) in &files {
            let shown = path
                .strip_prefix(root)
                .unwrap_or(path)
                .display()
                .to_string();
            let lines: Vec<&str> = text.lines().collect();
            let mut index = 0;
            while index < lines.len() {
                let line = lines[index].trim_start();
                if !line.starts_with("#[derive(") {
                    index += 1;
                    continue;
                }
                // The derive, which may run over several lines.
                let start = index;
                let mut derive = String::from(line);
                while !derive.contains(")]") && index + 1 < lines.len() {
                    index += 1;
                    derive.push_str(lines[index].trim());
                }
                index += 1;
                if !derive.contains("Serialize") {
                    continue;
                }
                // The attributes between the derive and the item.
                let mut carries = false;
                while index < lines.len() {
                    let next = lines[index].trim_start();
                    if !(next.starts_with("#[") || next.starts_with("//")) {
                        break;
                    }
                    carries |= next.contains(DERIVE);
                    index += 1;
                }
                if !carries {
                    outcome.failures.push(Finding {
                        file: shown.clone(),
                        line: start + 1,
                        text: format!(
                            "derives `Serialize` without `#[cfg_attr(feature = \"schema\", {DERIVE})]`"
                        ),
                        rule: RULE,
                    });
                }
            }
            for found in hand_impl.captures_iter(text) {
                let name = &found[1];
                if described(name) {
                    continue;
                }
                let line = text[..found.get(0).map_or(0, |m| m.start())]
                    .lines()
                    .count()
                    + 1;
                outcome.failures.push(Finding {
                    file: shown.clone(),
                    line,
                    text: format!(
                        "serialises `{name}` by hand with no `JsonSchema` for it in this crate"
                    ),
                    rule: RULE,
                });
            }
        }
    }
}

/// Every platform row of a workflow matrix runs on the runner the
/// platform table names.
///
/// `xtask/src/platform.rs` says it is "the only place any of that is
/// written", and it was not: the workflows kept their own copy of which
/// runner builds which platform, and when GitHub retired the
/// `macos-13` image the table and two workflows had to be corrected in
/// three places. One of them was missed for an afternoon, and the
/// symptom was not a failure -- `bindings (darwin-x64)` sat in `queued`
/// for as long as anyone let it, so eleven dispatches of the verify
/// matrix never reached a conclusion at all.
///
/// The same shape as `knob-has-a-reader` and the parity runners' list:
/// a description that claims to be the only one, read by generators
/// that each keep a copy.
///
/// A matrix row that names no `platform` is not covered, and that is
/// deliberate rather than an oversight: `hash-matrix.yml` chooses
/// runners to compare two architectures and two operating systems, in
/// Rust's own arch vocabulary (`linux-x86_64`), and which runners those
/// are is that workflow's decision and not the shipping table's.
fn platform_runners(root: &Path, outcome: &mut Outcome) {
    const RULE: &str = "runner-matches-the-platform-table";
    for path in workflows(root) {
        let shown = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .display()
            .to_string();
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        // A workflow that does not parse is `workflow-parses`'s to
        // report; this rule says nothing about it.
        let Ok(doc) = serde_yaml_ng::from_str::<serde_yaml_ng::Value>(&text) else {
            continue;
        };
        for entry in matrix_rows(&doc) {
            let (Some(platform), Some(os)) = (
                entry.get("platform").and_then(serde_yaml_ng::Value::as_str),
                entry.get("os").and_then(serde_yaml_ng::Value::as_str),
            ) else {
                continue;
            };
            let complaint = match crate::platform::Platform::by_name(platform) {
                Some(row) if row.runner == os => continue,
                Some(row) => format!(
                    "`{platform}` runs on `{os}`; the platform table says `{}`",
                    row.runner
                ),
                None => format!("`{platform}` is not a row of the platform table"),
            };
            let line = text
                .lines()
                .position(|line| line.contains(&format!("platform: {platform}")))
                .map_or(0, |at| at + 1);
            outcome.failures.push(Finding {
                file: shown.clone(),
                line,
                text: complaint,
                rule: RULE,
            });
        }
    }
}

/// Every `strategy.matrix.include` entry of every job of a workflow.
fn matrix_rows(doc: &serde_yaml_ng::Value) -> Vec<&serde_yaml_ng::Value> {
    let mut out = Vec::new();
    let Some(jobs) = doc.get("jobs").and_then(serde_yaml_ng::Value::as_mapping) else {
        return out;
    };
    for (_, job) in jobs {
        let include = job
            .get("strategy")
            .and_then(|strategy| strategy.get("matrix"))
            .and_then(|matrix| matrix.get("include"))
            .and_then(serde_yaml_ng::Value::as_sequence);
        if let Some(entries) = include {
            out.extend(entries.iter());
        }
    }
    out
}

/// The condition language's kinds are listed once, and the list is complete.
///
/// `Condition::kind` names each predicate and `language::KINDS` lists them
/// for whoever walks the language — the prose pass reports one rendering for
/// each kind, and a kind missing from the list would simply not be reported.
/// A private copy of a list is how a generator's description goes stale, so
/// this holds the two against each other both ways.
fn predicates_are_listed(root: &Path, outcome: &mut Outcome) {
    const RULE: &str = "every-predicate-is-listed";
    const FILE: &str = "crates/rules/src/language.rs";
    let Ok(text) = std::fs::read_to_string(root.join(FILE)) else {
        outcome.failures.push(Finding {
            file: String::from(FILE),
            line: 1,
            text: String::from("cannot be read"),
            rule: RULE,
        });
        return;
    };
    let quoted = regex::Regex::new(r#""([a-z-]+)""#)
        .unwrap_or_else(|error| panic!("the pattern compiles: {error}"));
    let between = |start: &str, end: &str| -> Vec<String> {
        text.split_once(start)
            .and_then(|(_, rest)| rest.split_once(end))
            .map(|(block, _)| {
                quoted
                    .captures_iter(block)
                    .filter_map(|found| found.get(1).map(|name| String::from(name.as_str())))
                    .collect()
            })
            .unwrap_or_default()
    };
    let listed = between("pub const KINDS: [&str; ", "];");
    let named = between("pub const fn kind(&self)", "\n    }");
    let line = text
        .lines()
        .position(|line| line.contains("pub const KINDS"))
        .map_or(1, |at| at + 1);
    for (left, right, what) in [
        (&listed, &named, "listed but no predicate answers it"),
        (&named, &listed, "answered by a predicate but not listed"),
    ] {
        for kind in left {
            if !right.contains(kind) {
                outcome.failures.push(Finding {
                    file: String::from(FILE),
                    line,
                    text: format!("`{kind}` is {what}"),
                    rule: RULE,
                });
            }
        }
    }
    outcome.allowed.push(Finding {
        file: String::from(FILE),
        line,
        text: format!("{} predicates listed", listed.len()),
        rule: RULE,
    });
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
    knob_readers(root, &mut outcome);
    boundary_sources(root, &mut outcome);
    workflows_parse(root, &mut outcome);
    tiers_turn_on_their_base(root, &mut outcome);
    entry_points_reachable(root, &mut outcome);
    gate_runners(root, &mut outcome);
    python_in_utf8(root, &mut outcome);
    platform_runners(root, &mut outcome);
    targets_declare_their_features(root, &mut outcome);
    serialised_types_describe_themselves(root, &mut outcome);
    predicates_are_listed(root, &mut outcome);
    composers_reach_every_binding(root, &mut outcome);
    crates_are_listed(root, &mut outcome);
    open_questions_are_named(root, &mut outcome);

    let mut report = String::new();
    for rule in [
        "deterministic-iteration",
        "ambient-input",
        "unsafe-inventory",
        "exact-classification",
        "knob-has-a-reader",
        "boundary-is-described",
        "workflow-parses",
        "gate-has-a-runner",
        "entry-point-is-reachable",
        "python-runs-in-utf8-mode",
        "runner-matches-the-platform-table",
        "target-declares-the-feature-it-needs",
        "a-tier-turns-on-its-base",
        "serialised-type-describes-itself",
        "every-predicate-is-listed",
        "composer-reaches-every-binding",
        "crate-is-listed",
        "open-question-is-named",
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

#[cfg(test)]
mod tests {
    use super::{declaration_of, gate_declared_by, gate_run_by};

    /// The shape of a `group!` body, which is what the rule reads.
    const SOURCE: &str = "group!(
    Vargas, VargasPatch {
        /// The convention for an unattested chart.
        unattested_dn: UnattestedDn,
        /// The bala scheme.
        /// lint: knob-has-a-reader — `strength`, Phase 5.
        bala_scheme: BalaScheme,
        /// The eras.
        /// lint: knob-has-a-reader — the first line,
        /// and the second, which reads on.
        eras: BTreeSet<Era>,
    }
);
";

    #[test]
    fn a_knob_without_a_marker_has_no_reason() {
        let (line, reason) = declaration_of(SOURCE, "unattested_dn");
        assert_eq!(line, 4, "the line the knob is declared on");
        assert_eq!(reason, None);
    }

    #[test]
    fn a_marker_gives_its_reason_and_belongs_to_the_knob_below_it() {
        let (line, reason) = declaration_of(SOURCE, "bala_scheme");
        assert_eq!(line, 7);
        assert_eq!(reason.as_deref(), Some("`strength`, Phase 5."));
        // And it does not leak upwards to the knob before it.
        assert_eq!(declaration_of(SOURCE, "unattested_dn").1, None);
    }

    #[test]
    fn a_reason_may_run_over_several_lines_and_reads_as_one() {
        let (_, reason) = declaration_of(SOURCE, "eras");
        assert_eq!(
            reason.as_deref(),
            Some("the first line, and the second, which reads on.")
        );
    }

    #[test]
    fn a_knob_that_is_not_there_is_not_found() {
        assert_eq!(declaration_of(SOURCE, "no_such_knob"), (0, None));
    }

    #[test]
    fn a_run_line_names_the_gate_it_runs() {
        assert_eq!(
            gate_run_by("        run: cargo xtask check-batching"),
            Some("check-batching")
        );
        // With its arguments, and quoted, as the DCO and tag gates are run.
        assert_eq!(
            gate_run_by(r#"        run: cargo xtask check-tag "${GITHUB_REF_NAME}""#),
            Some("check-tag")
        );
    }

    #[test]
    fn a_gate_is_matched_as_a_whole_word() {
        // `check-doc` is a prefix of `check-docs`: a substring test would
        // call it wired, which is the whole failure the rule exists for.
        assert_eq!(
            gate_run_by("        run: cargo xtask check-docs"),
            Some("check-docs")
        );
        assert_ne!(
            gate_run_by("        run: cargo xtask check-docs"),
            Some("check-doc")
        );
    }

    #[test]
    fn a_line_that_runs_no_gate_names_none() {
        assert_eq!(gate_run_by("        run: cargo xtask gen ffi"), None);
        assert_eq!(gate_run_by("        run: cargo test --workspace"), None);
    }

    #[test]
    fn an_arm_names_the_gate_it_declares() {
        assert_eq!(
            gate_declared_by(
                "        Some(\"check-surface\") => surface::check_generated(&repo_root()),"
            ),
            Some("check-surface")
        );
    }

    #[test]
    fn an_arm_that_is_not_a_gate_declares_none() {
        assert_eq!(
            gate_declared_by("        Some(\"surface\") => surface::generate(&repo_root()),"),
            None
        );
        assert_eq!(gate_declared_by("    let name = Some(\"check-x\");"), None);
    }
}

#[cfg(test)]
mod serialised_type_describes_itself {
    #![allow(clippy::unwrap_used, reason = "a test fails by panicking")]

    use super::*;

    /// A crate with the `schema` feature, written under a fresh
    /// directory, and the rule run over it.
    fn findings(source: &str) -> Vec<String> {
        let root = std::env::temp_dir().join(format!(
            "teistro-lint-{}-{}",
            std::process::id(),
            source.len()
        ));
        let krate = root.join("crates/sample");
        std::fs::create_dir_all(krate.join("src")).unwrap();
        std::fs::write(
            krate.join("Cargo.toml"),
            "[features]\nschema = [\"dep:schemars\"]\n",
        )
        .unwrap();
        std::fs::write(krate.join("src/lib.rs"), source).unwrap();
        let mut outcome = Outcome::default();
        serialised_types_describe_themselves(&root, &mut outcome);
        std::fs::remove_dir_all(&root).unwrap();
        outcome
            .failures
            .iter()
            .map(|f| format!("{}:{}", f.line, f.text))
            .collect()
    }

    #[test]
    fn a_serialised_type_without_a_schema_is_found_in_both_forms() {
        let found = findings(
            "#[derive(Debug,\n    serde::Serialize)]\n#[serde(rename_all = \"lowercase\")]\npub enum Bare { One }\n\n\
             pub struct Hand;\nimpl serde::Serialize for Hand {}\n",
        );
        assert_eq!(found.len(), 2, "{found:?}");
        assert!(found[0].starts_with("1:derives `Serialize`"), "{found:?}");
        assert!(found[1].contains("serialises `Hand` by hand"), "{found:?}");
    }

    #[test]
    fn a_type_that_describes_itself_passes_whatever_sits_between() {
        let found = findings(
            "#[derive(serde::Serialize)]\n/// A comment.\n#[serde(tag = \"kind\")]\n\
             #[cfg_attr(feature = \"schema\", derive(schemars::JsonSchema))]\npub enum Tagged { One }\n\n\
             pub struct Hand;\nimpl serde::Serialize for Hand {}\nhand_schema!(Hand, \"Hand\", {});\n\
             #[derive(Debug, Clone)]\npub struct NotSerialised;\n",
        );
        assert!(found.is_empty(), "{found:?}");
    }
}
