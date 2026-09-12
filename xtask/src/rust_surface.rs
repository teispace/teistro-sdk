//! The falsification pass over **what a Rust consumer would have to
//! assemble**, measured before a Rust façade is designed.
//!
//! `cargo xtask rust-surface` writes the page; `check-rust-surface`
//! regenerates it in memory and fails on any difference.
//!
//! # Why this is a pass and not a paragraph
//!
//! ADR-0030 §9 leaves "Rust's own consumer surface" to the Rust
//! binding's own page, and the obvious proposal is that it mirrors the
//! other three: seven areas and a root over one context. Whether that is
//! worth building depends on a question nobody had asked — **how far is
//! a Rust consumer from it today?** — and that is measurable from the
//! source rather than arguable.
//!
//! What can be measured: every boundary module names the SDK crates it
//! calls, and [`crate::areas`] already reads which entry points each
//! area's operations reach. Composing the two gives the crates behind
//! each area, the crates behind a whole context, and — the sharp one —
//! any pair of crates that **only** the boundary brings together, which
//! is assembly a Rust consumer cannot reach at all without going
//! through C.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

use crate::generated::{Output, check, write};
use crate::measure::{Claim, table};

/// The boundary's own description, which says which module each entry
/// point was extracted from.
const API: &str = "idl/api.json";

/// The boundary crate, whose modules are read for what they call.
const BOUNDARY: &str = "crates/ffi/src";

/// The Node layer, which is the one ergonomic surface a pass can read;
/// [`crate::areas`] says why it and not the other two.
const NODE: &str = "bindings/node/lib/index.js";

const PAGE: &str = "docs/03-design/rust-consumer-surface-measured.md";

/// Where the SDK's crates live, read for what each depends on.
const CRATES: &str = "crates";

/// The boundary crate itself, which is not a consumer.
const FFI: &str = "ffi";

/// The boundary modules a **context's own life** needs, which no area's
/// operations reach and every consumer uses first.
///
/// The areas are operations *on* a context; building one is
/// `ts_context_new` and its provider, and that is where the settings,
/// the locale and the ephemeris are composed. Leaving these out
/// understated the whole measurement: the two crates only the boundary
/// holds are both reached from here.
const A_CONTEXT_ITSELF: [&str; 2] = ["context", "provider"];

/// Crates a boundary module names that are not an SDK crate a consumer
/// would depend on.
///
/// `teistro_ffi` is the boundary naming itself, for its own build
/// information; `teistro_idl` is the description the generators read,
/// which a consumer of the SDK has no reason to hold.
const NOT_A_CONSUMER_DEPENDENCY: [&str; 2] = ["teistro_ffi", "teistro_idl"];

#[derive(Debug, serde::Deserialize)]
struct Function {
    name: String,
    source: String,
}

#[derive(Debug, serde::Deserialize)]
struct Api {
    functions: Vec<Function>,
}

/// The boundary module an entry point was extracted from.
fn module_of(source: &str) -> &str {
    source
        .rsplit('/')
        .next()
        .and_then(|file| file.strip_suffix(".rs"))
        .unwrap_or(source)
}

/// A crate name as a manifest spells it, from the way Rust source does.
fn dashed(name: &str) -> String {
    name.replace('_', "-")
}

/// Every SDK crate each boundary module calls.
///
/// Read off the source rather than from the manifest, because the
/// manifest says what the *crate* depends on and this page is about what
/// each **operation** needs: `teistro-chart` is a dependency of the
/// boundary whether or not the calendar's entry points touch it.
fn reached(root: &Path) -> Result<BTreeMap<String, BTreeSet<String>>, String> {
    let dir = root.join(BOUNDARY);
    let entries =
        std::fs::read_dir(&dir).map_err(|error| format!("{BOUNDARY} is not readable: {error}"))?;
    let mut out: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|kind| kind != "rs") {
            continue;
        }
        let Some(module) = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
        else {
            continue;
        };
        let text = std::fs::read_to_string(&path)
            .map_err(|error| format!("{} is not readable: {error}", path.display()))?;
        out.insert(module, crates_named(&text));
    }
    if out.is_empty() {
        return Err(format!("{BOUNDARY} holds no modules"));
    }
    Ok(out)
}

/// The SDK crates a module's source names.
fn crates_named(text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut rest = text;
    while let Some(at) = rest.find("teistro_") {
        let tail = rest.get(at..).unwrap_or("");
        let name: String = tail
            .chars()
            .take_while(|c| c.is_ascii_lowercase() || *c == '_' || c.is_ascii_digit())
            .collect();
        if !NOT_A_CONSUMER_DEPENDENCY.contains(&name.as_str()) {
            out.insert(name.clone());
        }
        rest = tail.get(name.len()..).unwrap_or("");
    }
    out
}

/// Which SDK crates depend on each SDK crate, **not counting the
/// boundary**.
///
/// This is what decides whether an integration exists outside
/// `crates/ffi`. A crate the boundary needs and nothing else depends on
/// is one whose composition with the rest is written once, at the C
/// boundary, and therefore reachable from Rust only through C.
fn dependants(root: &Path) -> Result<BTreeMap<String, BTreeSet<String>>, String> {
    let dir = root.join(CRATES);
    let entries =
        std::fs::read_dir(&dir).map_err(|error| format!("{CRATES} is not readable: {error}"))?;
    let mut out: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut found = 0_usize;
    for entry in entries.flatten() {
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        let manifest = entry.path().join("Cargo.toml");
        if !manifest.is_file() {
            continue;
        }
        found += 1;
        out.entry(format!("teistro-{name}")).or_default();
        if name == FFI {
            continue;
        }
        let text = std::fs::read_to_string(&manifest)
            .map_err(|error| format!("{} is not readable: {error}", manifest.display()))?;
        for line in text.lines() {
            let Some(depended) = line.split_whitespace().next() else {
                continue;
            };
            if depended.starts_with("teistro-") && line.contains("path") {
                out.entry(depended.to_owned())
                    .or_default()
                    .insert(name.clone());
            }
        }
    }
    if found == 0 {
        return Err(format!("{CRATES} holds no crates"));
    }
    Ok(out)
}

/// The page, from one reading of the description, the boundary and the
/// Node layer.
fn outputs(root: &Path) -> Result<Vec<Output>, String> {
    let text = std::fs::read_to_string(root.join(API))
        .map_err(|error| format!("{API} is not readable: {error}"))?;
    let api: Api =
        serde_json::from_str(&text).map_err(|error| format!("{API} does not parse: {error}"))?;
    let node = std::fs::read_to_string(root.join(NODE))
        .map_err(|error| format!("{NODE} is not readable: {error}"))?;
    let entry_points: Vec<String> = api.functions.iter().map(|f| f.name.clone()).collect();
    let surfaces = crate::areas::surfaces_of(&node, &entry_points)?;
    let per_module = reached(root)?;
    let dependants = dependants(root)?;
    Ok(vec![Output::new(
        PAGE,
        page(&api, &surfaces, &per_module, &dependants),
    )])
}

pub(crate) fn generate(root: &Path) -> i32 {
    match outputs(root) {
        Ok(outputs) => write(root, &outputs),
        Err(message) => {
            eprintln!("{message}");
            1
        }
    }
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    match outputs(root) {
        Ok(outputs) => check(root, &outputs, "cargo xtask rust-surface"),
        Err(message) => {
            eprintln!("{message}");
            1
        }
    }
}

/// What each area's operations need, in SDK crates.
fn per_area<'a>(
    surfaces: &'a [crate::areas::Surface],
    module: &BTreeMap<&str, &str>,
    reached: &BTreeMap<String, BTreeSet<String>>,
) -> BTreeMap<&'a str, BTreeSet<String>> {
    let mut out: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for surface in surfaces {
        let entry = out.entry(surface.name.as_str()).or_default();
        for member in &surface.members {
            for point in &member.reaches {
                if let Some(found) = module.get(point.as_str())
                    && let Some(crates) = reached.get(*found)
                {
                    entry.extend(crates.iter().cloned());
                }
            }
        }
    }
    out
}

fn page(
    api: &Api,
    surfaces: &[crate::areas::Surface],
    reached: &BTreeMap<String, BTreeSet<String>>,
    dependants: &BTreeMap<String, BTreeSet<String>>,
) -> String {
    let module: BTreeMap<&str, &str> = api
        .functions
        .iter()
        .map(|function| (function.name.as_str(), module_of(&function.source)))
        .collect();
    let areas = per_area(surfaces, &module, reached);

    // A whole context: every crate every area needs, and the crates
    // building one needs before any area is read off it.
    let mut whole: BTreeSet<&str> = areas
        .values()
        .flat_map(|crates| crates.iter().map(String::as_str))
        .collect();
    for module in A_CONTEXT_ITSELF {
        if let Some(crates) = reached.get(module) {
            whole.extend(crates.iter().map(String::as_str));
        }
    }

    // The crates only the boundary brings together: a crate that no
    // single SDK crate depends on beside another is assembly a Rust
    // consumer has to do themselves, and the pair that says so loudest
    // is the one no consumer-facing crate holds at all.
    let mut only_the_boundary: Vec<&str> = whole
        .iter()
        .copied()
        .filter(|name| {
            dependants
                .get(&dashed(name))
                .is_some_and(BTreeSet::is_empty)
        })
        .collect();
    only_the_boundary.sort_unstable();

    let mut out = String::new();
    out.push_str(&header(api, surfaces, &whole));
    out.push_str(&claims_section(&areas, &whole, &only_the_boundary));
    out.push_str(&areas_section(&areas));
    out.push_str(&modules_section(reached));
    out.push_str(&limits_section());
    out
}

/// What the page opens with, computed from the rows under it.
fn header(api: &Api, surfaces: &[crate::areas::Surface], whole: &BTreeSet<&str>) -> String {
    let operations: usize = surfaces.iter().map(|s| s.members.len()).sum();
    let areas = surfaces
        .iter()
        .filter(|s| s.name != crate::areas::ROOT)
        .count();
    let mut out = String::new();
    let _ = writeln!(out, "# The Rust consumer surface, measured\n");
    let _ = writeln!(
        out,
        "Status: `generated` by `cargo xtask rust-surface`, gated by `check-rust-surface`. Do not edit. Read from `idl/api.json`, the boundary's own description; from `crates/ffi/src/`, for the SDK crates each of its modules calls; from `crates/*/Cargo.toml`, for which crates depend on which; and from `bindings/node/lib/index.js`, the one ergonomic layer whose areas and the entry points they reach can both be read from one file.\n"
    );
    let _ = writeln!(
        out,
        "ADR-0030 §9 leaves Rust's own consumer surface to the Rust binding's own page, and the obvious proposal is that it mirrors the other three: **{areas} areas and a root**, {operations} operations, over one context. What that proposal is worth depends on how far a Rust consumer is from it today, which is the thing this page measures rather than argues.\n"
    );
    let _ = writeln!(
        out,
        "The measurement is a composition of two readings the repository already has: the boundary's description says which module each of its {} entry points came from, and the Node layer says which entry points each area's operations reach ([`surface-areas-measured.md`](surface-areas-measured.md)). Each boundary module names the SDK crates it calls, so an area's operations name the crates behind them.\n",
        api.functions.len()
    );
    let _ = writeln!(
        out,
        "**A context and its areas need {} of the SDK's crates**: {}.\n",
        whole.len(),
        named(&whole.iter().map(|name| dashed(name)).collect::<Vec<_>>())
    );
    out
}

/// The properties a Rust façade would or would not be adding.
fn claims_section(
    areas: &BTreeMap<&str, BTreeSet<String>>,
    whole: &BTreeSet<&str>,
    only_the_boundary: &[&str],
) -> String {
    // An area whose operations all come from one crate is an area a
    // consumer already has: `use teistro_calendar::…` is the façade.
    let mixed: Vec<&str> = areas
        .iter()
        .filter(|(name, crates)| **name != crate::areas::ROOT && crates.len() > 1)
        .map(|(name, _)| *name)
        .collect();
    let empty: Vec<&str> = areas
        .iter()
        .filter(|(_, crates)| crates.is_empty())
        .map(|(name, _)| *name)
        .collect();

    let claims = [
        Claim::counted(
            "an area's operations come from one SDK crate, so a Rust consumer already has the area",
            mixed.len(),
            areas.len().saturating_sub(1),
        )
        .with_note(format!(
            "more than one: {}",
            named(
                &mixed
                    .iter()
                    .map(|name| format!("{name} ({})", areas.get(name).map_or(0, BTreeSet::len)))
                    .collect::<Vec<_>>()
            )
        )),
        Claim::counted(
            "every area reaches the boundary, so every area names crates",
            empty.len(),
            areas.len(),
        )
        .with_note(if empty.is_empty() {
            String::from("so every row of the table below is a measurement and not a gap")
        } else {
            format!("naming none: {}", named(&empty))
        }),
        Claim::counted(
            "no crate a context needs is brought in by the boundary alone",
            only_the_boundary.len(),
            whole.len(),
        )
        .with_note(format!(
            "only `crates/ffi` depends on {}",
            named(
                &only_the_boundary
                    .iter()
                    .map(|name| dashed(name))
                    .collect::<Vec<_>>()
            )
        )),
    ];

    let mut out = String::new();
    let _ = writeln!(out, "## The properties\n");
    out.push_str(&table(&claims));
    let _ = writeln!(out);
    if only_the_boundary.is_empty() {
        let _ = writeln!(
            out,
            "**Every crate a context needs is composed somewhere other than the C boundary**, so a Rust consumer can assemble one with the crates alone.\n"
        );
    } else {
        let _ = writeln!(
            out,
            "**{} of the crates a context needs are held by `crates/ffi` and by nothing else** — {} — so the composition that makes a context is written once, at the C boundary, and a Rust consumer cannot reach it without going through C. This is the finding that decides the question the page was written to ask: a Rust façade would not be a convenience over crates a consumer already composes, it would be the *first* place that composition exists in Rust.\n",
            only_the_boundary.len(),
            named(
                &only_the_boundary
                    .iter()
                    .map(|name| dashed(name))
                    .collect::<Vec<_>>()
            )
        );
    }
    out
}

/// Every area, and the crates behind its operations.
fn areas_section(areas: &BTreeMap<&str, BTreeSet<String>>) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "## What each area needs\n");
    let _ = writeln!(out, "| area | crates | which |");
    let _ = writeln!(out, "|---|---|---|");
    for (name, crates) in areas {
        let _ = writeln!(
            out,
            "| `{name}` | {} | {} |",
            crates.len(),
            if crates.is_empty() {
                String::from("—")
            } else {
                named(&crates.iter().map(|one| dashed(one)).collect::<Vec<_>>())
            }
        );
    }
    let _ = writeln!(out);
    out
}

/// Every boundary module, and what it calls.
fn modules_section(reached: &BTreeMap<String, BTreeSet<String>>) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "## What each boundary module calls\n");
    let _ = writeln!(
        out,
        "Read off the source and not the manifest: the manifest says what the boundary *crate* depends on, and this page is about what each operation needs.\n"
    );
    let _ = writeln!(out, "| module | calls |");
    let _ = writeln!(out, "|---|---|");
    for (module, crates) in reached {
        let _ = writeln!(
            out,
            "| `{module}` | {} |",
            if crates.is_empty() {
                String::from("—")
            } else {
                named(&crates.iter().map(|one| dashed(one)).collect::<Vec<_>>())
            }
        );
    }
    let _ = writeln!(out);
    out
}

/// What this page does not measure, said rather than left to be
/// assumed.
fn limits_section() -> String {
    let mut out = String::new();
    let _ = writeln!(out, "## What this does not measure\n");
    let _ = writeln!(
        out,
        "- **What a Rust façade's signatures would look like.** This page counts crates, which decides *whether* the composition exists in Rust; it says nothing about whether a Rust consumer wants a context object, a builder, or free functions over a settings value. That is the design page's question."
    );
    let _ = writeln!(
        out,
        "- **What each crate's public API already offers.** A crate behind an area may already expose exactly the operation, or may expose the pieces it is built from. Counting crates cannot tell the two apart."
    );
    let _ = writeln!(
        out,
        "- **What the C caller's memory costs.** `blob`, `string` and `support` are listed below and add nothing to any area, because no area's operations reach them: they are the C caller's memory and its handshake, and a Rust consumer of the crates has neither — the crates hand back their own types. [`surface-areas-measured.md`](surface-areas-measured.md) calls them plumbing for the same reason. So a Rust façade is smaller than the boundary, not larger."
    );
    let _ = writeln!(out);
    out
}

/// Names in a sentence, each in backticks.
fn named<T: AsRef<str>>(names: &[T]) -> String {
    if names.is_empty() {
        return String::from("none");
    }
    names
        .iter()
        .map(|name| format!("`{}`", name.as_ref()))
        .collect::<Vec<_>>()
        .join(", ")
}
