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

/// One member of a surface, as the ergonomic layer declares it.
struct Member {
    name: String,
    /// The entry points its own body calls, in the order they appear.
    reaches: Vec<String>,
}

/// One thing a consumer reads members off: an area, or the context
/// itself.
struct Surface {
    /// What a consumer writes — `calendar` for `sdk.calendar`, or
    /// `(root)` for the context.
    name: String,
    /// The class the layer declares it with, so a reader can find it.
    class: String,
    members: Vec<Member>,
}

/// The name the context is read under, which is not an area.
const ROOT: &str = "(root)";

/// The layer's surfaces, and what each member of each reaches.
///
/// Read rather than listed, so that an area added to the layer is an
/// area this page reports and an operation moved between two shows as
/// moved. **Refuses** rather than guesses when a class is not where it
/// expects: a parse that silently found nothing would report a surface
/// of no members and call every rule proven.
fn surfaces(source: &str, entry_points: &[String]) -> Result<Vec<Surface>, String> {
    let reached: Vec<(String, String)> = entry_points
        .iter()
        .map(|name| (format!(".{}(", camel(name)), name.clone()))
        .collect();

    // The wiring is the mapping: the constructor says which class each
    // area is, so the page cannot name an area the layer does not build.
    let context = class_body(source, "export class Context {")?;
    let mut out = vec![Surface {
        name: ROOT.to_owned(),
        class: "Context".to_owned(),
        members: members_of(context, &reached)?,
    }];
    for line in context.lines() {
        let Some((area, class)) = wired(line) else {
            continue;
        };
        let body = class_body(source, &format!("\nclass {class} "))?;
        out.push(Surface {
            name: area,
            class,
            members: members_of(body, &reached)?,
        });
    }
    if out.len() == 1 {
        return Err(format!("{NODE} wires no areas onto its context"));
    }
    Ok(out)
}

/// The area a constructor line wires, and the class it wires it with.
///
/// `this.calendar = new CalendarArea(reach);` and the engine's
/// `this.#engine = new Engine(reach);`, which is the same wiring behind a
/// private field because reading it is what asks the engine a question.
fn wired(line: &str) -> Option<(String, String)> {
    let rest = line.trim().strip_prefix("this.")?;
    let (name, rest) = rest.split_once(" = new ")?;
    let class = rest.split_once('(')?.0;
    Some((
        name.trim_start_matches('#').to_owned(),
        class.trim().to_owned(),
    ))
}

/// A class's body, from the line that opens it to the `}` at column zero.
fn class_body<'a>(source: &'a str, opens: &str) -> Result<&'a str, String> {
    let at = source
        .find(opens)
        .ok_or_else(|| format!("{NODE} does not declare `{}`", opens.trim()))?;
    let after = source
        .get(at..)
        .and_then(|rest| rest.find('\n').map(|line| at + line + 1))
        .ok_or_else(|| format!("{NODE}'s `{}` has no body", opens.trim()))?;
    let body = source
        .get(after..)
        .ok_or_else(|| format!("{NODE} ends inside `{}`", opens.trim()))?;
    let end = body
        .find("\n}\n")
        .ok_or_else(|| format!("{NODE}'s `{}` does not end at column zero", opens.trim()))?;
    body.get(..end)
        .ok_or_else(|| format!("{NODE}'s `{}` could not be read", opens.trim()))
}

/// The members a class body declares, and what each reaches.
fn members_of(body: &str, spellings: &[(String, String)]) -> Result<Vec<Member>, String> {
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
        return Err(format!("{NODE} declares a surface with no members"));
    }
    // A constructor is not an operation, and a `get`/`set` pair is one
    // member however many times the layer declares it: what the page is
    // counting is what a consumer writes.
    let mut out: Vec<Member> = Vec::with_capacity(offsets.len());
    for (index, (offset, name)) in offsets.iter().enumerate() {
        if name == "constructor" {
            continue;
        }
        let stop = offsets.get(index + 1).map_or(body.len(), |(next, _)| *next);
        let text = body.get(*offset..stop).unwrap_or_default();
        let reaches: Vec<String> = spellings
            .iter()
            .filter(|(spelling, _)| text.contains(spelling.as_str()))
            .map(|(_, entry)| entry.clone())
            .collect();
        if let Some(already) = out.iter_mut().find(|member| member.name == *name) {
            already.reaches.extend(reaches);
            continue;
        }
        out.push(Member {
            name: name.clone(),
            reaches,
        });
    }
    if out.is_empty() {
        return Err(format!("{NODE} declares a surface with nothing on it"));
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

/// The three parity runners, whose lists of canonical paths are the only
/// place the surface's shape is written down for all of the bindings.
///
/// Each holds its own copy, in its own syntax, and `check-parity`
/// compares what they *print* rather than what they list -- so three
/// runners that all forget the same new operation agree perfectly and
/// the gate says nothing. That is the hole this property closes, and it
/// is the same shape as several generators keeping the same private
/// list.
const RUNNERS: [&str; 3] = [
    "bindings/node/parity.mjs",
    "bindings/dart/bin/parity.dart",
    "bindings/python/parity.py",
];

/// The canonical paths a runner lists, whatever its language's quotes
/// and brackets look like: the first element of each row is the path,
/// and a path is the only quoted string in these files shaped
/// `<area>.<operation>`.
fn listed(source: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for line in source.lines() {
        // The row shape, and the same one in all three: an opening
        // bracket for the language's tuple or array, then the path as a
        // quoted string. Read per line and anchored at its start, because
        // scanning the file for quotes goes out of phase on the first
        // apostrophe in a comment.
        let row = line.trim_start();
        let Some(row) = row.strip_prefix('[').or_else(|| row.strip_prefix('(')) else {
            continue;
        };
        let Some(quote) = row.chars().next().filter(|c| *c == '\'' || *c == '"') else {
            continue;
        };
        let after = row.get(quote.len_utf8()..).unwrap_or("");
        let Some(end) = after.find(quote) else {
            continue;
        };
        let quoted = after.get(..end).unwrap_or("");
        if quoted.contains('.')
            && quoted
                .chars()
                .all(|c| c.is_ascii_lowercase() || c == '_' || c == '.' || c == '(' || c == ')')
        {
            out.insert(quoted.to_owned());
        }
    }
    out
}

/// An operation's canonical path: the area, and the operation in the
/// snake case the runners key by.
fn canonical(area: &str, member: &str) -> String {
    let mut operation = String::with_capacity(member.len() + 2);
    for letter in member.chars() {
        if letter.is_ascii_uppercase() {
            operation.push('_');
            operation.extend(letter.to_lowercase());
        } else {
            operation.push(letter);
        }
    }
    format!("{area}.{operation}")
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
    let surfaces = surfaces(&node, &entry_points)?;
    let mut runners: Vec<(&str, BTreeSet<String>)> = Vec::with_capacity(RUNNERS.len());
    for runner in RUNNERS {
        let text = std::fs::read_to_string(root.join(runner))
            .map_err(|error| format!("{runner} is not readable: {error}"))?;
        let paths = listed(&text);
        if paths.is_empty() {
            return Err(format!("{runner} lists no canonical surface paths"));
        }
        runners.push((runner, paths));
    }
    Ok(vec![Output::new(PAGE, page(&api, &surfaces, &runners))])
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

/// Whether the root counts as reaching a module.
///
/// Two questions, and they are not the same one: "is this module reached
/// at all" includes the root, and "can a consumer find this module's
/// operations under two names" is about the areas alone.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Reach {
    Anywhere,
    AreasOnly,
}

/// What area, if any, reaches a boundary module.
fn areas_of<'a>(
    surfaces: &'a [Surface],
    module: &BTreeMap<&str, &'a str>,
    reach: Reach,
) -> BTreeMap<&'a str, BTreeSet<&'a str>> {
    let mut out: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for surface in surfaces {
        if reach == Reach::AreasOnly && surface.name == ROOT {
            continue;
        }
        for member in &surface.members {
            for entry in &member.reaches {
                if let Some(found) = module.get(entry.as_str()) {
                    out.entry(found).or_default().insert(&surface.name);
                }
            }
        }
    }
    out
}

fn page(api: &Api, surfaces: &[Surface], runners: &[(&str, BTreeSet<String>)]) -> String {
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
    let by_module = areas_of(surfaces, &module, Reach::Anywhere);
    let by_area = areas_of(surfaces, &module, Reach::AreasOnly);

    let mut out = String::new();
    let _ = writeln!(out, "# The surface areas, measured\n");
    let _ = writeln!(
        out,
        "Status: `generated` by `cargo xtask areas`, gated by `check-areas`. Do not edit. Read from `{API}`, the boundary's own description, and from `{NODE}`, the one ergonomic layer whose areas and the entry points they reach can both be read from one file.\n"
    );
    let _ = writeln!(
        out,
        "The surface is `sdk.<area>.<operation>` (ADR-0030, designed in [`surface-areas.md`](surface-areas.md)). This measured the grouping **before** it was built, and falsified the rule the ADR words it by: five boundary modules were never reached by anything a consumer called, one member reached two, and the areas had to be chosen with the derivation as evidence rather than derived from it. That argument is made and [the design page](surface-areas.md) keeps it.\n"
    );
    let _ = writeln!(
        out,
        "What this page holds now is **the built thing**: the areas the layer wires, what each reaches, and the properties a namespaced surface has to keep as the remaining phases add to it.\n"
    );

    out.push_str(&claims_section(
        api,
        surfaces,
        &per_module,
        &by_module,
        &by_area,
        runners,
    ));
    out.push_str(&strays_section(api));
    out.push_str(&areas_section(surfaces, &module));
    out.push_str(&modules_section(&per_module, &by_module));
    out.push_str(&limits_section());
    out
}

/// Every operation the layer declares, against every runner's list.
///
/// A miss is a (runner, operation) pair, because one runner having it and
/// two not is two thirds of a gap rather than none. The other direction
/// is counted too: a row a runner lists that the layer no longer
/// declares is what goes stale when an operation is renamed rather than
/// added.
fn listing_claim(surfaces: &[Surface], runners: &[(&str, BTreeSet<String>)]) -> Claim {
    let operations: Vec<String> = surfaces
        .iter()
        .flat_map(|surface| {
            surface
                .members
                .iter()
                .map(move |member| canonical(&surface.name, &member.name))
        })
        .collect();
    let declared: BTreeSet<&str> = operations.iter().map(String::as_str).collect();
    let mut unlisted: Vec<String> = Vec::new();
    let mut stale: Vec<String> = Vec::new();
    for (runner, paths) in runners {
        let file = runner.rsplit('/').next().unwrap_or(runner);
        for operation in &operations {
            if !paths.contains(operation) {
                unlisted.push(format!("{operation} by {file}"));
            }
        }
        for path in paths {
            if !declared.contains(path.as_str()) {
                stale.push(format!("{path} by {file}"));
            }
        }
    }
    Claim::counted(
        "every operation the layer declares is listed by every parity runner",
        unlisted.len() + stale.len(),
        operations.len() * runners.len(),
    )
    .with_note(match (unlisted.is_empty(), stale.is_empty()) {
        (true, true) => {
            String::from("so an operation added to one binding cannot go unheld in the other two")
        }
        (false, true) => format!("unlisted: {}", named(&unlisted)),
        (true, false) => format!("listed and gone: {}", named(&stale)),
        (false, false) => format!(
            "unlisted: {}; listed and gone: {}",
            named(&unlisted),
            named(&stale)
        ),
    })
}

/// The properties the surface has to keep.
fn claims_section(
    api: &Api,
    surfaces: &[Surface],
    per_module: &BTreeMap<&str, Vec<&str>>,
    by_module: &BTreeMap<&str, BTreeSet<&str>>,
    by_area: &BTreeMap<&str, BTreeSet<&str>>,
    runners: &[(&str, BTreeSet<String>)],
) -> String {
    let carries = api
        .functions
        .iter()
        .filter(|function| carries_its_module(function))
        .count();
    let strays: Vec<&str> = api
        .functions
        .iter()
        .filter(|function| !carries_its_module(function))
        .map(|function| function.name.as_str())
        .collect();

    let unreached: Vec<&str> = per_module
        .keys()
        .copied()
        .filter(|name| !by_module.contains_key(name))
        .collect();
    // Over the areas alone. The root is not an area, and `positions`
    // reaching `ts_frame_pack` from it does not give a consumer a second
    // place to look for anything: there is no `sdk.frame.pack`.
    let split: Vec<&str> = by_area
        .iter()
        .filter(|(_, areas)| areas.len() > 1)
        .map(|(module, _)| *module)
        .collect();
    let empty: Vec<&str> = surfaces
        .iter()
        .filter(|surface| {
            surface.name != ROOT && surface.members.iter().all(|m| m.reaches.is_empty())
        })
        .map(|surface| surface.name.as_str())
        .collect();
    let repeats = repeating(surfaces);

    let claims = [
        Claim::counted(
            "a boundary module a consumer reaches is reached from one area",
            split.len(),
            by_area.len(),
        )
        .with_note("so no operation can be looked for under two names"),
        Claim::counted(
            "every area holds an operation that reaches the boundary",
            empty.len(),
            surfaces.len() - 1,
        ),
        Claim::counted(
            "no operation's name repeats its own area's",
            repeats.len(),
            surfaces
                .iter()
                .filter(|s| s.name != ROOT)
                .map(|s| s.members.len())
                .sum(),
        )
        .with_note("which is what the namespace is for"),
        Claim::counted(
            "every boundary module is reached, or is the caller's memory or the context's life",
            unreached.len().saturating_sub(PLUMBING.len()),
            per_module.len(),
        )
        .with_note(format!("unreached: {}", named(&unreached))),
        Claim::counted(
            "an entry point's name already carries its own module",
            api.functions.len() - carries,
            api.functions.len(),
        )
        .with_note(format!("the exceptions are {}", named(&strays))),
        listing_claim(surfaces, runners),
    ];

    let mut out = String::new();
    let _ = writeln!(out, "## The properties\n");
    out.push_str(&table(&claims));
    let _ = writeln!(out);
    if repeats.is_empty() {
        let _ = writeln!(
            out,
            "**No operation spells its own area.** Several did on the flat surface — `convertTime`, because `convert` was taken by the calendar, is the one to remember — and each gave the word back when the namespace took it; [`surface-areas.md`](surface-areas.md) lists them. This row is the one that decays quietly as operations are added, which is why it is gated.\n"
        );
    } else {
        let _ = writeln!(
            out,
            "**{} {} its own area inside its own name**: {}. The namespace already carries the word, so saying it twice is what the flat surface had to do and this one does not.\n",
            repeats.len(),
            if repeats.len() == 1 {
                "operation spells"
            } else {
                "operations spell"
            },
            named(&repeats)
        );
    }
    if !unreached.is_empty() {
        let _ = writeln!(
            out,
            "The unreached modules are `{}`. {} of them are the plumbing the rule allows for — the C caller's memory, the context's own life, and the library itself — and a rule that made an area of each would put them on the surface with nothing ever to be found under them.\n",
            unreached.join("`, `"),
            PLUMBING.len()
        );
    }
    out
}

/// Whether an entry point's name begins with the module it lives in.
fn carries_its_module(function: &Function) -> bool {
    let stem = function.name.strip_prefix("ts_").unwrap_or(&function.name);
    let module = module_of(&function.source);
    stem == module || stem.starts_with(&format!("{module}_"))
}

/// The boundary modules that exist for the C caller's memory, the
/// context's own life, or the library itself, and are therefore not
/// areas.
///
/// Named rather than discovered, because the reason is a **kind** and not
/// a count: a module nothing happens to reach this week is not the same
/// as one nothing should ever reach.
const PLUMBING: [&str; 5] = ["blob", "context", "lib", "provider", "string"];

/// The operations whose names still spell their own area.
fn repeating(surfaces: &[Surface]) -> Vec<String> {
    let mut out = Vec::new();
    for surface in surfaces {
        if surface.name == ROOT {
            continue;
        }
        let stem = surface.name.strip_suffix('s').unwrap_or(&surface.name);
        for member in &surface.members {
            let spelling = member.name.to_ascii_lowercase();
            if spelling != surface.name
                && (spelling.contains(&surface.name) || spelling.contains(stem))
            {
                out.push(format!("{}.{}", surface.name, member.name));
            }
        }
    }
    out
}

/// The entry points whose names do not carry their module for a reason
/// `03-design/surface-areas.md` records, so the page can tell a decision
/// from an oversight while the rule goes on counting both.
const DECLARED: [&str; 1] = ["ts_context_new_with_provider"];

/// The entry points whose names do not carry their module, which are
/// three different things and the page should say which.
fn strays_section(api: &Api) -> String {
    let mut by_module: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for function in &api.functions {
        if !carries_its_module(function) {
            by_module
                .entry(module_of(&function.source))
                .or_default()
                .push(&function.name);
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
    let declared: Vec<&str> = mismatched
        .iter()
        .copied()
        .filter(|name| DECLARED.contains(name))
        .collect();
    let unexplained: Vec<&str> = mismatched
        .iter()
        .copied()
        .filter(|name| !DECLARED.contains(name))
        .collect();
    let _ = writeln!(out, "### The names that do not carry their module\n");
    let _ = writeln!(
        out,
        "{library} of them are `lib`'s, and `lib` is not a module in the sense the others are: it is the library itself, and `ts_sdk_version` would gain nothing by becoming `ts_lib_sdk_version`. The rule does not apply to it, and this page counts it as a miss rather than writing the exception into the rule — a rule with its exceptions inside it cannot be falsified by anything.\n"
    );
    if !declared.is_empty() {
        let _ = writeln!(
            out,
            "{} {} **declared in [`surface-areas.md`](surface-areas.md)** and still counted here for the same reason: {}. It belongs to `context`'s family by the name a consumer calls and to `provider`'s by what it depends on, and the name that should read well is the one a consumer calls.\n",
            declared.len(),
            if declared.len() == 1 {
                "is an exception"
            } else {
                "are exceptions"
            },
            named(&declared)
        );
    }
    if unexplained.is_empty() {
        let _ = writeln!(
            out,
            "Nothing else. The three that were inconsistencies were all the same one — a function named in the singular in a file named in the plural — and were fixed by renaming the **file**, which is not an ABI symbol, rather than the function, which is.\n"
        );
    } else {
        let _ = writeln!(
            out,
            "The remaining {} {} not accounted for: {}.\n",
            unexplained.len(),
            if unexplained.len() == 1 { "is" } else { "are" },
            named(&unexplained)
        );
    }
    out
}

/// Every area, its operations, and what each reaches.
fn areas_section(surfaces: &[Surface], module: &BTreeMap<&str, &str>) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "## The areas the layer wires\n");
    let operations: usize = surfaces
        .iter()
        .filter(|s| s.name != ROOT)
        .map(|s| s.members.len())
        .sum();
    let _ = writeln!(
        out,
        "**{} areas over {} operations, and a root.** An area is a *value*: built once with the context, frozen, and destructurable, which is what makes the grouping worth having rather than merely tidy.\n",
        surfaces.len() - 1,
        operations
    );
    for surface in surfaces {
        let title = if surface.name == ROOT {
            format!("### The root — `{}`\n", surface.class)
        } else {
            format!("### `sdk.{}` — `{}`\n", surface.name, surface.class)
        };
        let _ = writeln!(out, "{title}");
        let _ = writeln!(out, "| operation | reaches |");
        let _ = writeln!(out, "|---|---|");
        for member in &surface.members {
            let _ = writeln!(
                out,
                "| `{}` | {} |",
                member.name,
                if member.reaches.is_empty() {
                    String::from("—")
                } else {
                    member
                        .reaches
                        .iter()
                        .map(|entry| {
                            let area = module.get(entry.as_str()).copied().unwrap_or("?");
                            format!("`{entry}` ({area})")
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            );
        }
        let _ = writeln!(out);
    }
    out
}

fn modules_section(
    per_module: &BTreeMap<&str, Vec<&str>>,
    by_module: &BTreeMap<&str, BTreeSet<&str>>,
) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "## What each boundary module holds\n");
    let _ = writeln!(out, "| module | entry points | reached from |");
    let _ = writeln!(out, "|---|---:|---|");
    for (name, entries) in per_module {
        let _ = writeln!(
            out,
            "| `{name}` | {} | {} |",
            entries.len(),
            by_module.get(name).map_or_else(
                || String::from("—"),
                |areas| areas
                    .iter()
                    .map(|area| format!("`{area}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        );
    }
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "**Entry points do not measure an area's size to a consumer.** `calendar` has the most of them and `chart`, `positions` and `panchanga` have one each — and those three are the operations the SDK exists for. One entry point serves a whole family there, because a chart request carries what would otherwise have been a dozen calls. An area sized by its entry points would rank the surface almost backwards, which is why the design page sizes none of them.\n"
    );
    out
}

fn limits_section() -> String {
    let mut out = String::new();
    let _ = writeln!(out, "## What this does not measure\n");
    let _ = writeln!(
        out,
        "**The Dart and Python layers.** Only one of the three declares its surface and reaches the boundary by a name derived from the entry point's own, so only one can be read this way. What holds the other two to this one is `check-parity`, which compares a `surface.<area>.<operation>` line per operation from each runner: this page says what Node's shape *is*, and that gate says the other two share it.\n"
    );
    let _ = writeln!(
        out,
        "**Whether the area names are the right ones.** This holds the properties a namespaced surface must keep. `almanac` over `panchanga`, and `engine` over `ephemeris`, are arguments and not counts, and [`surface-areas.md`](surface-areas.md) makes them.\n"
    );
    let _ = writeln!(
        out,
        "**What the remaining phases add.** Every area here is one the SDK already has. Dashas, strengths, rules, interpretation and the application modules are what make a flat surface untenable, and an area invented before its operations exist is a slot that shapes the work to fit it.\n"
    );
    out
}

/// A list of names, as prose.
fn named<T: AsRef<str>>(names: &[T]) -> String {
    names
        .iter()
        .map(|name| format!("`{}`", name.as_ref()))
        .collect::<Vec<_>>()
        .join(", ")
}
