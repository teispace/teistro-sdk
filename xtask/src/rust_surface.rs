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

/// The entry points a **context's own life** needs, which no area's
/// operations reach and every consumer uses first.
///
/// The areas are operations *on* a context; building one is these, and
/// that is where the settings, the locale and the ephemeris are
/// composed. Leaving them out understated the whole measurement: the
/// crates only the boundary holds are reached from here.
const A_CONTEXT_ITSELF: [&str; 2] = ["ts_context_new", "ts_context_new_with_provider"];

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

/// What one function of the boundary crate names.
struct Body {
    /// The SDK crates its own lines reach, whether by a path or by a
    /// name the file imported.
    crates: BTreeSet<String>,
    /// The boundary's own functions it calls, each as the module
    /// declaring it and its name. **Resolved, not bare**: two modules
    /// may each declare a `place`, and a closure keyed by the bare name
    /// alone attributed one module's crates to the other -- `intl` came
    /// out reaching `teistro-panchanga`, and a date conversion came out
    /// reaching the chart.
    calls: BTreeSet<(String, String)>,
}

/// Every SDK-crate item a file brings into scope, by the name its own
/// lines then call it.
///
/// Rust code reaches a crate through `use`, so a function's body usually
/// names no crate at all: `shipped(calendar)` is `teistro_calendar`'s
/// and says so nowhere. Reading the imports is what makes a per-function
/// measurement possible, and reading them is why this **refuses** a glob
/// — `use teistro_x::*` would put names in scope that no line here can
/// enumerate, and a measurement that silently under-attributed would
/// read as a Rust consumer being closer than they are.
fn imports(file: &str, text: &str) -> Result<Scope, String> {
    let mut out = Scope::default();
    let mut statement = String::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if statement.is_empty() && !trimmed.starts_with("use ") {
            continue;
        }
        statement.push_str(trimmed);
        if !statement.ends_with(';') {
            statement.push(' ');
            continue;
        }
        let whole = std::mem::take(&mut statement);
        let Some(rest) = whole
            .trim_end_matches(';')
            .strip_prefix("use ")
            .map(str::trim)
        else {
            continue;
        };
        // `use crate::<module>::{…}` says where a bare call resolves;
        // without it, `with_context` in `calendar.rs` is a name no
        // module declares and the closure stops at the boundary's own
        // front door.
        if let Some(inside) = rest.strip_prefix("crate::") {
            if inside.contains('*') {
                return Err(format!(
                    "{file} imports `{rest}`, and a glob puts names in scope this page cannot enumerate"
                ));
            }
            if let Some((module, _)) = inside.split_once("::") {
                for item in items_of(inside) {
                    out.helpers.insert(item, module.to_owned());
                }
            }
            continue;
        }
        let Some(name) = rest.strip_prefix("teistro_") else {
            continue;
        };
        let Some((tail, _)) = name.split_once("::") else {
            continue;
        };
        let of = format!("teistro_{tail}");
        if rest.contains('*') {
            return Err(format!(
                "{file} imports `{rest}`, and a glob puts names in scope this page cannot enumerate"
            ));
        }
        for item in items_of(rest) {
            out.crates.insert(item, of.clone());
        }
    }
    if !statement.is_empty() {
        return Err(format!("{file} ends inside a `use` statement"));
    }
    Ok(out)
}

/// What one boundary module has in scope: the SDK-crate items it
/// imported, and where each of the boundary's own helpers it imported
/// is declared.
#[derive(Default)]
struct Scope {
    /// An imported item's name, to the SDK crate it came from.
    crates: BTreeMap<String, String>,
    /// An imported helper's name, to the boundary module declaring it.
    helpers: BTreeMap<String, String>,
}

/// The names a `use` statement's tail brings into scope, each under the
/// spelling the source then uses: the braced list, or the last segment,
/// and the alias where there is one.
fn items_of(rest: &str) -> Vec<String> {
    let named = |one: &str| -> String {
        let one = one.trim();
        one.rsplit(" as ")
            .next()
            .unwrap_or(one)
            .rsplit("::")
            .next()
            .unwrap_or(one)
            .trim()
            .to_owned()
    };
    match rest.split_once('{') {
        Some((_, list)) => list
            .trim_end_matches('}')
            .split(',')
            .filter(|one| !one.trim().is_empty())
            .map(named)
            .collect(),
        None => vec![named(rest)],
    }
}

/// Every function the boundary crate declares, and what each names,
/// keyed by the module declaring it.
///
/// **Methods too, and that was a correction.** The first reading took
/// module-level functions only, on the ground that a method of
/// `TsCalendarDate` converts between a C struct and a crate's type and
/// is marshalling a Rust consumer neither has nor wants. True of that
/// one and false of the one that matters: `TsContext::new` *is* the
/// assembly — settings, locale and ephemeris meet there — so
/// `ts_context_new` came out reaching two crates when it reaches five,
/// and the page understated its own finding.
///
/// A body runs from its `fn` line to the first `}` at the same
/// indentation, which the source being `rustfmt`-clean makes exact.
fn bodies(root: &Path) -> Result<BTreeMap<(String, String), Body>, String> {
    let dir = root.join(BOUNDARY);
    let entries =
        std::fs::read_dir(&dir).map_err(|error| format!("{BOUNDARY} is not readable: {error}"))?;
    let mut files: Vec<(String, String)> = Vec::new();
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
        files.push((module, text));
    }
    if files.is_empty() {
        return Err(format!("{BOUNDARY} holds no modules"));
    }
    files.sort();
    let modules: BTreeSet<String> = files.iter().map(|(module, _)| module.clone()).collect();

    let mut out: BTreeMap<(String, String), Body> = BTreeMap::new();
    for (module, text) in &files {
        let scope = imports(module, text)?;
        let declared: BTreeSet<String> = text
            .lines()
            .filter_map(|line| function_name(line).map(|(name, _)| name))
            .collect();
        let lines: Vec<&str> = text.lines().collect();
        for (at, line) in lines.iter().enumerate() {
            let Some((name, indent)) = function_name(line) else {
                continue;
            };
            let closes = format!("{}}}", " ".repeat(indent));
            let body: Vec<&str> = lines
                .iter()
                .skip(at + 1)
                .take_while(|line| **line != closes)
                .copied()
                .collect();
            if at + 1 + body.len() >= lines.len() {
                return Err(format!(
                    "{module}.rs's `{name}` has no `}}` at its own indentation"
                ));
            }
            let read = named_in(module, &body, &scope, &declared, &modules);
            // A name a module declares twice -- a `#[cfg]` pair, an
            // `impl` on two types -- reaches the union of both, which is
            // what a reading keyed by the bare name can honestly say.
            let entry = out.entry((module.clone(), name)).or_insert_with(|| Body {
                crates: BTreeSet::new(),
                calls: BTreeSet::new(),
            });
            entry.crates.extend(read.crates);
            entry.calls.extend(read.calls);
        }
    }
    Ok(out)
}

/// The name a function declaration gives and the indentation it sits
/// at, when a line is one.
fn function_name(line: &str) -> Option<(String, usize)> {
    let indent = line.len() - line.trim_start().len();
    let mut rest = line.trim_start();
    for modifier in [
        "pub(crate) ",
        "pub(super) ",
        "pub ",
        "default ",
        "const ",
        "async ",
        "unsafe ",
        "extern \"C\" ",
    ] {
        if rest.starts_with("fn ") {
            break;
        }
        rest = rest.strip_prefix(modifier).unwrap_or(rest);
    }
    let after = rest.strip_prefix("fn ")?;
    let name: String = after
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    (!name.is_empty()).then_some((name, indent))
}

/// The crates and the boundary functions a body's lines name.
///
/// A called name resolves three ways, in this order: a
/// `crate::<module>::<name>` path, a helper the file imported from
/// another module, or a function this module declares itself. Anything
/// else is a method, a variable or a macro, and is not followed.
fn named_in(
    module: &str,
    body: &[&str],
    scope: &Scope,
    declared: &BTreeSet<String>,
    modules: &BTreeSet<String>,
) -> Body {
    let mut crates = BTreeSet::new();
    let mut calls = BTreeSet::new();
    for line in body {
        let code = line.split_once("//").map_or(*line, |(before, _)| before);
        let words = words(code);
        for (at, word) in words.iter().enumerate() {
            if word.starts_with("teistro_") {
                crates.insert(word.clone());
                continue;
            }
            if let Some(of) = scope.crates.get(word) {
                crates.insert(of.clone());
            }
            // `crate::support::null` -- the module is named on the line.
            if modules.contains(word)
                && let Some(next) = words.get(at + 1)
            {
                calls.insert((word.clone(), next.clone()));
                continue;
            }
            if let Some(where_from) = scope.helpers.get(word) {
                calls.insert((where_from.clone(), word.clone()));
            } else if declared.contains(word) {
                calls.insert((module.to_owned(), word.clone()));
            }
        }
    }
    crates.retain(|one| !NOT_A_CONSUMER_DEPENDENCY.contains(&one.as_str()));
    Body { crates, calls }
}

/// The identifiers a line holds, **one per path segment**.
///
/// `Place::new` is two words, and it has to be: the file imported
/// `Place` and not `Place::new`, so keeping the path whole made every
/// associated function invisible — which read as a Rust consumer being
/// closer to the operation than they are, and was the bug that made
/// `ts_chart_found` appear not to reach `teistro-chart` at all.
fn words(line: &str) -> Vec<String> {
    line.split(|letter: char| !letter.is_alphanumeric() && letter != '_')
        .filter(|word| !word.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Every SDK crate a function reaches, following the boundary's own
/// functions until nothing new is added.
///
/// A closure and not one hop: `ts_calendar_convert`'s own body names no
/// crate, calls `system_of`, which calls `shipped`, which is
/// `teistro-calendar`'s. A one-hop reading would have called that
/// operation crate-free and every conclusion drawn from it would have
/// been wrong in the consumer's favour.
fn closure(
    bodies: &BTreeMap<(String, String), Body>,
) -> BTreeMap<(String, String), BTreeSet<String>> {
    let mut out: BTreeMap<(String, String), BTreeSet<String>> = bodies
        .iter()
        .map(|(who, body)| (who.clone(), body.crates.clone()))
        .collect();
    loop {
        let mut grew = false;
        for (who, body) in bodies {
            let mut gained: BTreeSet<String> = BTreeSet::new();
            for called in &body.calls {
                if called == who {
                    continue;
                }
                if let Some(theirs) = out.get(called) {
                    gained.extend(theirs.iter().cloned());
                }
            }
            if let Some(mine) = out.get_mut(who) {
                let before = mine.len();
                mine.extend(gained);
                grew |= mine.len() != before;
            }
        }
        if !grew {
            break;
        }
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
    let bodies = bodies(root)?;
    let closed = closure(&bodies);
    // Down to the entry points, each in the module its own description
    // names. Every one is a literal `extern "C" fn` of this crate, so
    // one the reader did not find is a reader that has stopped working
    // rather than a boundary that has grown a new shape.
    let mut per_function: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut missing: Vec<&str> = Vec::new();
    for function in &api.functions {
        let who = (
            module_of(&function.source).to_owned(),
            function.name.clone(),
        );
        match closed.get(&who) {
            Some(crates) => {
                per_function.insert(function.name.clone(), crates.clone());
            }
            None => missing.push(function.name.as_str()),
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "{BOUNDARY} declares no function for {}",
            named(&missing)
        ));
    }
    let dependants = dependants(root)?;
    Ok(vec![Output::new(
        PAGE,
        page(&api, &surfaces, &per_function, &dependants),
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
    per_function: &BTreeMap<String, BTreeSet<String>>,
) -> BTreeMap<&'a str, BTreeSet<String>> {
    let mut out: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for surface in surfaces {
        let entry = out.entry(surface.name.as_str()).or_default();
        for member in &surface.members {
            for point in &member.reaches {
                if let Some(crates) = per_function.get(point) {
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
    per_function: &BTreeMap<String, BTreeSet<String>>,
    dependants: &BTreeMap<String, BTreeSet<String>>,
) -> String {
    let areas = per_area(surfaces, per_function);

    // A whole context: every crate every area needs, and the crates
    // building one needs before any area is read off it.
    let mut whole: BTreeSet<&str> = areas
        .values()
        .flat_map(|crates| crates.iter().map(String::as_str))
        .collect();
    for point in A_CONTEXT_ITSELF {
        if let Some(crates) = per_function.get(point) {
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
    out.push_str(&claims_section(
        api,
        &areas,
        &whole,
        &only_the_boundary,
        per_function,
    ));
    out.push_str(&areas_section(&areas));
    out.push_str(&entry_points_section(api, per_function));
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
        "Status: `generated` by `cargo xtask rust-surface`, gated by `check-rust-surface`. Do not edit. Read from `idl/api.json`, the boundary's own description; from `crates/ffi/src/`, for what every function of it names and calls; from `crates/*/Cargo.toml`, for which crates depend on which; and from `bindings/node/lib/index.js`, the one ergonomic layer whose areas and the entry points they reach can both be read from one file.\n"
    );
    let _ = writeln!(
        out,
        "ADR-0030 §9 leaves Rust's own consumer surface to the Rust binding's own page, and the obvious proposal is that it mirrors the other three: **{areas} areas and a root**, {operations} operations, over one context. What that proposal is worth depends on how far a Rust consumer is from it today, which is the thing this page measures rather than argues.\n"
    );
    let _ = writeln!(
        out,
        "**The method**, because the number is only as good as it. Every function of the boundary crate is read for the SDK crates it names — by a path, or by a name its module imported — and for the boundary's own functions it calls, resolved through `use crate::<module>::…` and `crate::<module>::<name>` rather than by bare name. Those calls are then closed over until nothing new is added, so a function whose own lines name no crate still reaches whatever it calls: `ts_calendar_convert` names none, calls `system_of`, which calls `teistro-calendar`'s `shipped`. Which of the {} entry points each area's operations reach comes from the Node layer, as [`surface-areas-measured.md`](surface-areas-measured.md) reads it.\n",
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

/// The entry points whose work is composition, and those whose work is
/// none of it.
///
/// An entry point reaching one crate is an operation a Rust consumer
/// already has: the boundary's own lines around it are the marshalling,
/// and a façade over it would be a rename. One reaching two or more is
/// composition, which is work. One reaching none at all is the C
/// caller's memory and has nothing to port.
fn tallies<'a>(
    api: &'a Api,
    per_function: &BTreeMap<String, BTreeSet<String>>,
) -> (Vec<&'a str>, Vec<&'a str>) {
    let of = |wanted: fn(&BTreeSet<String>) -> bool| -> Vec<&'a str> {
        api.functions
            .iter()
            .filter(|function| per_function.get(&function.name).is_some_and(wanted))
            .map(|function| function.name.as_str())
            .collect()
    };
    (of(|crates| crates.len() > 1), of(BTreeSet::is_empty))
}

/// The properties a Rust façade would or would not be adding.
fn claims_section(
    api: &Api,
    areas: &BTreeMap<&str, BTreeSet<String>>,
    whole: &BTreeSet<&str>,
    only_the_boundary: &[&str],
    per_function: &BTreeMap<String, BTreeSet<String>>,
) -> String {
    let (composed, marshalling) = tallies(api, per_function);
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
            "an entry point's work reaches one SDK crate, so a façade over it is a rename",
            composed.len(),
            api.functions.len(),
        )
        .with_note(format!(
            "{} reach two or more; {} reach none at all, and those are the C caller's memory: {}",
            composed.len(),
            marshalling.len(),
            named(&marshalling)
        )),
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

/// Every entry point, and the SDK crates its work reaches.
///
/// The closure, not the body: a function whose own lines name no crate
/// still reaches whatever the boundary helpers it calls reach.
fn entry_points_section(api: &Api, per_function: &BTreeMap<String, BTreeSet<String>>) -> String {
    let mut rows: Vec<(&str, &str, &BTreeSet<String>)> = api
        .functions
        .iter()
        .filter_map(|function| {
            per_function
                .get(&function.name)
                .map(|crates| (function.name.as_str(), module_of(&function.source), crates))
        })
        .collect();
    // Widest first: the rows that decide the question are the ones a
    // façade would have the most to compose.
    rows.sort_by(|left, right| {
        right
            .2
            .len()
            .cmp(&left.2.len())
            .then_with(|| left.0.cmp(right.0))
    });

    let mut out = String::new();
    let _ = writeln!(out, "## What each entry point reaches\n");
    let _ = writeln!(
        out,
        "Widest first. Read through the boundary's own helpers, because a body that names no crate still reaches whatever it calls: `ts_calendar_convert` names none, calls `system_of`, which calls `teistro-calendar`'s `shipped`.\n"
    );
    let _ = writeln!(out, "| entry point | module | crates | which |");
    let _ = writeln!(out, "|---|---|---|---|");
    for (name, module, crates) in rows {
        let _ = writeln!(
            out,
            "| `{name}` | `{module}` | {} | {} |",
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
        "- **A crate reached by inference alone.** A name resolves here through an import, a path, or the module it is declared in; a method called on a value whose type is never written, in a module that imports nothing of that crate, is invisible. Nothing in the boundary is written that way today — the readings agree with a second, coarser one that attributes a whole module's imports to each of its entry points — but a body that grew that shape would read low rather than loudly, so it is said here."
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
