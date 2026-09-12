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
    entry_points_reachable(root, &mut outcome);
    gate_runners(root, &mut outcome);
    python_in_utf8(root, &mut outcome);
    platform_runners(root, &mut outcome);

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
