//! The falsification pass over **the consumption surface's areas**: what
//! `sdk.<area>.<operation>` would group by, measured before the four
//! binding layers are restructured around it.
//!
//! `cargo xtask areas` writes the page; `check-areas` regenerates it in
//! memory and fails on any difference.
//!
//! # Why this is a pass and not a paragraph
//!
//! ADR-0030 decides that the surface is namespaced and says the areas are
//! "derived from the boundary modules the reference site already groups
//! by". That is a **proposal about a grouping**, and it is exactly the
//! kind of claim this project measures before building: the restructuring
//! it asks for touches four binding layers, four reference surfaces and
//! every example, and it is cheap now and breaking after v1.
//!
//! What can be measured is where each thing actually sits today: which
//! boundary module every entry point belongs to, and which of them a
//! consumer-facing member of the context actually reaches. A module no
//! member reaches is not an area however neatly it groups the C surface,
//! and a member that reaches two is a member the rule cannot place.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

use serde::Deserialize;

use crate::generated::{Output, check, write};
use crate::measure::{Claim, table};

/// The boundary's own description, which every binding is written or
/// generated against.
const API: &str = "idl/api.json";

/// The one ergonomic layer with a surface this pass can read.
///
/// Node rather than Dart or Python because its layer declares the class
/// in one file and reaches the boundary by a name derived from the entry
/// point's own, so the mapping is a transformation and not a guess. What
/// the other two do differently is a thing this page does not measure and
/// says so.
const NODE: &str = "bindings/node/lib/index.js";

const PAGE: &str = "docs/03-design/surface-areas-measured.md";

/// One entry point of the C boundary.
#[derive(Debug, Deserialize)]
struct Function {
    name: String,
    source: String,
}

#[derive(Debug, Deserialize)]
struct Api {
    functions: Vec<Function>,
}

/// A boundary module: the file the entry points were extracted from,
/// which is what the reference site already groups by.
fn module_of(source: &str) -> &str {
    source
        .rsplit('/')
        .next()
        .and_then(|file| file.strip_suffix(".rs"))
        .unwrap_or(source)
}

/// The name the Node layer reaches an entry point by.
///
/// A transformation of the entry point's own name rather than a table:
/// `ts_time_delta_t` is `timeDeltaT`, and a table of forty-six would be a
/// second place for the mapping to be wrong.
fn camel(name: &str) -> String {
    let mut parts = name.strip_prefix("ts_").unwrap_or(name).split('_');
    let mut out = parts.next().unwrap_or_default().to_string();
    for part in parts {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.push(first.to_ascii_uppercase());
            out.push_str(chars.as_str());
        }
    }
    out
}

/// One member of the context, as the ergonomic layer declares it.
struct Member {
    name: String,
    /// The entry points its own body calls, in the order they appear.
    reaches: Vec<String>,
}

/// The members of Node's `Context`, and what each reaches.
///
/// Read rather than listed, so that a member added to the layer is a
/// member this page reports. **Refuses** rather than guesses when the
/// class is not where it expects: a parse that silently found nothing
/// would report a surface of no members and call the rule proven.
fn members(source: &str, entry_points: &[String]) -> Result<Vec<Member>, String> {
    const OPEN: &str = "\nexport class Context {\n";
    let start = source
        .find(OPEN)
        .ok_or_else(|| format!("{NODE} does not declare `export class Context`"))?
        + OPEN.len();
    let body = &source[start..];
    let end = body
        .find("\n}\n")
        .ok_or_else(|| format!("{NODE}'s `Context` class does not end at column zero"))?;
    let body = &body[..end];

    // A member is declared at one indent and nothing else in the class
    // is: a line of two spaces, a name, and either `(` for a method or
    // `=` for a field. Accessors carry `get`/`set` and are members twice,
    // which is what the layer declares and what the page should say.
    let mut offsets: Vec<(usize, String)> = Vec::new();
    for (offset, line) in line_offsets(body) {
        if let Some(name) = member_name(line) {
            offsets.push((offset, name));
        }
    }
    if offsets.is_empty() {
        return Err(format!("{NODE}'s `Context` class declares no members"));
    }

    let reached: Vec<(String, String)> = entry_points
        .iter()
        .map(|name| (format!(".{}(", camel(name)), name.clone()))
        .collect();
    let mut out = Vec::with_capacity(offsets.len());
    for (index, (offset, name)) in offsets.iter().enumerate() {
        let stop = offsets.get(index + 1).map_or(body.len(), |(next, _)| *next);
        let text = body.get(*offset..stop).unwrap_or_default();
        out.push(Member {
            name: name.clone(),
            reaches: reached
                .iter()
                .filter(|(spelling, _)| text.contains(spelling.as_str()))
                .map(|(_, entry)| entry.clone())
                .collect(),
        });
    }
    Ok(out)
}

/// Every line of a body with the offset it starts at.
fn line_offsets(body: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut offset = 0;
    for line in body.split('\n') {
        out.push((offset, line));
        offset += line.len() + 1;
    }
    out
}

/// The name a member-declaring line declares, if it declares one.
fn member_name(line: &str) -> Option<String> {
    let rest = line.strip_prefix("  ")?;
    if rest.starts_with(' ') || rest.starts_with('*') || rest.starts_with('/') {
        return None;
    }
    let rest = rest
        .strip_prefix("get ")
        .or_else(|| rest.strip_prefix("set "))
        .or_else(|| rest.strip_prefix("async "))
        .unwrap_or(rest);
    let name: String = rest
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '$')
        .collect();
    if name.is_empty() {
        return None;
    }
    let after = rest.get(name.len()..)?.trim_start();
    // A declaration is followed by its parameter list or its initialiser;
    // anything else at this indent is a statement inside a member, which
    // the layer does not have but a future one might.
    (after.starts_with('(') || after.starts_with('=')).then_some(name)
}

/// The page, from one reading of both files.
fn outputs(root: &Path) -> Result<Vec<Output>, String> {
    let text = std::fs::read_to_string(root.join(API))
        .map_err(|error| format!("{API} is not readable: {error}"))?;
    let api: Api =
        serde_json::from_str(&text).map_err(|error| format!("{API} does not parse: {error}"))?;
    let node = std::fs::read_to_string(root.join(NODE))
        .map_err(|error| format!("{NODE} is not readable: {error}"))?;
    let entry_points: Vec<String> = api.functions.iter().map(|f| f.name.clone()).collect();
    let members = members(&node, &entry_points)?;
    Ok(vec![Output::new(PAGE, page(&api, &members))])
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
        Ok(outputs) => check(root, &outputs, "cargo xtask areas"),
        Err(message) => {
            eprintln!("{message}");
            1
        }
    }
}

fn page(api: &Api, members: &[Member]) -> String {
    let module: BTreeMap<&str, &str> = api
        .functions
        .iter()
        .map(|f| (f.name.as_str(), module_of(&f.source)))
        .collect();
    let mut per_module: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for function in &api.functions {
        per_module
            .entry(module_of(&function.source))
            .or_default()
            .push(&function.name);
    }
    let reached: BTreeSet<&str> = members
        .iter()
        .flat_map(|m| m.reaches.iter())
        .filter_map(|entry| module.get(entry.as_str()).copied())
        .collect();
    let unreached: Vec<&str> = per_module
        .keys()
        .copied()
        .filter(|name| !reached.contains(name))
        .collect();

    let mut out = String::new();
    let _ = writeln!(out, "# The surface areas, measured\n");
    let _ = writeln!(
        out,
        "Status: `generated` by `cargo xtask areas`, gated by `check-areas`. Do not edit. Read from `{API}`, the boundary's own description, and from `{NODE}`, the one ergonomic layer whose members and the entry points they reach can both be read from one file.\n"
    );
    let _ = writeln!(
        out,
        "ADR-0030 decides that the surface becomes `sdk.<area>.<operation>` and says the areas are \"derived from the boundary modules the reference site already groups by\". That is a proposal about a grouping, and this is the measurement of it — taken before four binding layers, four reference surfaces and every example are restructured around it, because the change is cheap now and breaking after v1.\n"
    );

    out.push_str(&claims_section(api, members, &per_module, &unreached));
    out.push_str(&strays_section(api));
    out.push_str(&modules_section(&per_module, &reached, members, &module));
    out.push_str(&members_section(members, &module));
    out.push_str(&limits_section());
    out
}

/// The rules, and what the two descriptions said about them.
fn claims_section(
    api: &Api,
    members: &[Member],
    per_module: &BTreeMap<&str, Vec<&str>>,
    unreached: &[&str],
) -> String {
    let carries_its_module = |f: &&Function| {
        let stem = f.name.strip_prefix("ts_").unwrap_or(&f.name);
        let module = module_of(&f.source);
        stem == module || stem.starts_with(&format!("{module}_"))
    };
    let strays: Vec<&str> = api
        .functions
        .iter()
        .filter(|f| !carries_its_module(f))
        .map(|f| f.name.as_str())
        .collect();
    let carries = api
        .functions
        .iter()
        .filter(|f| {
            carries_its_module(f)
        })
        .count();
    let none = members.iter().filter(|m| m.reaches.is_empty()).count();
    let several = members
        .iter()
        .filter(|m| m.reaches.len() > 1)
        .collect::<Vec<_>>();
    let spread = members
        .iter()
        .filter(|m| {
            m.reaches
                .iter()
                .filter_map(|entry| {
                    api.functions
                        .iter()
                        .find(|f| f.name == *entry)
                        .map(|f| module_of(&f.source))
                })
                .collect::<BTreeSet<_>>()
                .len()
                > 1
        })
        .count();

    let claims = [
        Claim::counted(
            "every boundary module is an area a consumer sees",
            unreached.len(),
            per_module.len(),
        )
        .with_note(format!("never reached: {}", named(unreached))),
        Claim::counted(
            "every member of the context reaches the boundary",
            none,
            members.len(),
        )
        .with_note("each of them is a cached value, a decoded result or a delegate"),
        Claim::counted(
            "a member that reaches the boundary reaches one module",
            spread,
            members.len() - none,
        ),
        Claim::counted(
            "an entry point's name already carries its own module",
            api.functions.len() - carries,
            api.functions.len(),
        )
        .with_note(format!("the exceptions are {}", named(&strays))),
    ];

    let mut out = String::new();
    let _ = writeln!(out, "## The rules\n");
    out.push_str(&table(&claims));
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "**The grouping is real and the rule that derives it is not.** {} of the {} boundary modules are never reached by anything a consumer calls, and they are not an oversight: each exists for the C caller's memory or for the context's own life, which is not an operation anybody namespaces. A rule that made an area of every module would put {} of them on the surface with nothing ever to be found under them.\n",
        unreached.len(),
        per_module.len(),
        unreached.len()
    );
    if several.is_empty() {
        let _ = writeln!(
            out,
            "No member reaches more than one entry point, so nothing here has to be split between two areas.\n"
        );
    } else {
        let _ = writeln!(
            out,
            "**{} of the {} members that reach the boundary at all {} more than one entry point**, and that is where a grouping derived from the boundary does fail: {}. The failure is one of kind rather than of grouping — `ts_frame_pack` marshals the frame the request is expressed in and is not an operation a consumer would look for under an area — but the rule as ADR-0030 words it does not know that, so the areas cannot be *derived* and have to be **chosen with the derivation as evidence**.\n",
            several.len(),
            members.len() - none,
            if several.len() == 1 { "reaches" } else { "reach" },
            several
                .iter()
                .map(|m| {
                    let reaches: Vec<&str> = m.reaches.iter().map(String::as_str).collect();
                    format!("`{}` reaches {}", m.name, named(&reaches))
                })
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    out
}

/// The entry points whose names do not carry their module, which are two
/// different things and the page should say which.
fn strays_section(api: &Api) -> String {
    let mut by_module: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for function in &api.functions {
        let stem = function.name.strip_prefix("ts_").unwrap_or(&function.name);
        let module = module_of(&function.source);
        if stem != module && !stem.starts_with(&format!("{module}_")) {
            by_module.entry(module).or_default().push(&function.name);
        }
    }
    let library = by_module.get("lib").map_or(0, Vec::len);
    let mismatched: Vec<&str> = by_module
        .iter()
        .filter(|(module, _)| **module != "lib")
        .flat_map(|(_, names)| names.iter().copied())
        .collect();
    let mut out = String::new();
    if library + mismatched.len() == 0 {
        return out;
    }
    let _ = writeln!(out, "### The names that do not carry their module\n");
    let _ = writeln!(
        out,
        "{library} of them are `lib`'s, and `lib` is not a module in the sense the others are: it is the library itself, and `ts_sdk_version` would gain nothing by becoming `ts_lib_sdk_version`. The rule does not apply to it, and this page counts it as a miss rather than writing the exception into the rule — a rule with its exceptions inside it cannot be falsified by anything.\n"
    );
    let _ = writeln!(
        out,
        "The remaining {} are **inconsistencies rather than exceptions**, and each is a line to fix while the surface is being restructured anyway: {}.\n",
        mismatched.len(),
        named(&mismatched)
    );
    out
}

fn modules_section(
    per_module: &BTreeMap<&str, Vec<&str>>,
    reached: &BTreeSet<&str>,
    members: &[Member],
    module: &BTreeMap<&str, &str>,
) -> String {
    let mut per_area: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for member in members {
        for entry in &member.reaches {
            if let Some(area) = module.get(entry.as_str()) {
                per_area.entry(area).or_default().push(&member.name);
            }
        }
    }
    let mut out = String::new();
    let _ = writeln!(out, "## What each boundary module holds\n");
    let _ = writeln!(
        out,
        "| module | entry points | members that reach it | an area |"
    );
    let _ = writeln!(out, "|---|---:|---:|---|");
    for (name, entries) in per_module {
        let mut reaching = per_area.get(name).cloned().unwrap_or_default();
        reaching.sort_unstable();
        reaching.dedup();
        let _ = writeln!(
            out,
            "| `{name}` | {} | {} | {} |",
            entries.len(),
            reaching.len(),
            if reached.contains(name) { "**yes**" } else { "" }
        );
    }
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "**Entry points are not a measure of an area's size to a consumer, and that is the second finding.** `calendar` has the most of them and `chart`, `positions` and `panchanga` have one each — and those three are the operations the SDK exists for. One entry point serves a whole family there, because a chart request carries what would otherwise have been a dozen calls. An area sized by its entry points would rank the surface almost backwards.\n"
    );
    out
}

fn members_section(members: &[Member], module: &BTreeMap<&str, &str>) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "## Where each member sits today\n");
    let _ = writeln!(out, "| member | reaches | module |");
    let _ = writeln!(out, "|---|---|---|");
    for member in members {
        let areas: BTreeSet<&str> = member
            .reaches
            .iter()
            .filter_map(|entry| module.get(entry.as_str()).copied())
            .collect();
        let _ = writeln!(
            out,
            "| `{}` | {} | {} |",
            member.name,
            if member.reaches.is_empty() {
                String::from("—")
            } else {
                member
                    .reaches
                    .iter()
                    .map(|entry| format!("`{entry}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            },
            areas
                .iter()
                .map(|area| format!("`{area}`"))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    let _ = writeln!(out);
    // A member whose name is *only* its area is not repeating it: it is
    // the operation the area is named after, and namespacing gives it a
    // verb rather than taking one away.
    let repeats: Vec<&Member> = members
        .iter()
        .filter(|member| {
            let spelling = member.name.to_ascii_lowercase();
            member
                .reaches
                .iter()
                .filter_map(|entry| module.get(entry.as_str()).copied())
                .any(|area| {
                    let stem = area.strip_suffix('s').unwrap_or(area);
                    spelling != area && (spelling.contains(area) || spelling.contains(stem))
                })
        })
        .collect();
    let _ = writeln!(
        out,
        "**{} of them already spell their own area inside their own name** — {} — which is what a flat surface costs when two areas want the same verb: `convert` was taken by the calendar, so the time's became `convertTime`. Under `sdk.<area>.<operation>` each of those loses the half that is now the namespace, and none of them needs a new word invented for it.\n",
        repeats.len(),
        repeats
            .iter()
            .map(|member| format!("`{}`", member.name))
            .collect::<Vec<_>>()
            .join(", ")
    );
    out
}

fn limits_section() -> String {
    let mut out = String::new();
    let _ = writeln!(out, "## What this does not measure\n");
    let _ = writeln!(
        out,
        "**The Dart and Python layers.** Only one of the three declares its surface and reaches the boundary by a name derived from the entry point's own, so only one can be read this way. `check-parity` already holds all three to the same values; what it does not hold them to is the same *shape*, which is the gap this namespacing closes and a thing a later pass should gate.\n"
    );
    let _ = writeln!(
        out,
        "**Whether these are the right area names.** This says which groupings the boundary supports and which it does not. `ephemeris` is the clearest case of a name the measurement cannot settle: ADR-0030 renames it `engine` on an argument about what a consumer needs to be warned of, and no count decides that.\n"
    );
    let _ = writeln!(
        out,
        "**What the remaining phases add.** Every area here is one the SDK already has. Dashas, strengths, rules, interpretation and the application modules are what make a flat surface untenable, and they are not in the description yet — so this page measures the case for namespacing at its weakest, which is the honest time to make it.\n"
    );
    out
}

/// A list of names, as prose.
fn named(names: &[&str]) -> String {
    names
        .iter()
        .map(|name| format!("`{name}`"))
        .collect::<Vec<_>>()
        .join(", ")
}
