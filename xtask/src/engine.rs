//! The falsification pass over **the engine passthrough**: which of an
//! engine's own functions the SDK can offer at `sdk.engine.*`, which it
//! must not, and which it cannot yet.
//!
//! `cargo xtask engine` writes the page; `check-engine` regenerates it in
//! memory and fails on any difference, so a figure on it is a figure this
//! build produces from the engine's own description.
//!
//! # Why this is a pass and not a paragraph
//!
//! ADR-0030 puts the engine's own functions at `sdk.engine.*` for the
//! flexibility of reaching what the SDK has not ported. The port's own
//! contract says how: "the marshalling belongs to the adapter, generated
//! from the engine's manifest". Generated from a description means the
//! coverage is a **measurement** — it changes when the engine does — and
//! a measurement that is not gated drifts.
//!
//! The classification matters more than the count. A function this
//! adapter refuses to expose is not the same as one it has not learned to
//! marshal, and neither is a defect: the first is a boundary and the
//! second is a queue.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use serde::Deserialize;

use crate::generated::{Output, check, write};
use crate::measure::{count, spelled};

/// The engine's own description, vendored beside the adapter.
///
/// Vendored rather than read from a sibling checkout: the pass has to run
/// in CI, where no engine is installed, and a coverage figure that could
/// only be produced on one machine is not a gate. Refreshing it is a copy
/// and a regeneration, which is a deliberate act with a diff.
const IDL: &str = "adapters/ephemeris-teimeris/rust/data/teimeris.idl";

const PAGE: &str = "docs/03-design/engine-passthrough-measured.md";

/// The generated marshalling, beside the adapter that uses it.
const DISPATCH: &str = "adapters/ephemeris-teimeris/rust/src/dispatch.rs";

/// What the SDK owns and a caller must not take from it.
///
/// These open, close or rebind the very things the adapter is holding: a
/// consumer who called one through the passthrough would close the
/// context every other call depends on, or swap the data underneath it.
/// Excluded whatever their shape, and named here rather than discovered
/// by the marshaller failing to handle them, because the reason is a
/// boundary and not a difficulty.
const OWNED_BY_THE_ADAPTER: [&str; 3] = [
    "tm_context_open",
    "tm_context_close",
    "tm_context_set_fetch",
];

/// The prefix of the data-source family, all of which bind memory the
/// adapter owns for the life of its context.
const OWNED_PREFIX: &str = "tm_data_source_";

/// A function that changes engine state the SDK's own provenance does not
/// observe: after one of these a chart's recorded settings and the engine
/// answering it have quietly parted company.
///
/// **Offered rather than refused**, because reaching what the SDK has not
/// ported is the whole point of the namespace, and marked so that a
/// consumer chooses it knowingly.
fn mutates_engine_state(name: &str) -> bool {
    name.starts_with("tm_set_") || name.starts_with("tm_clear_")
}

#[derive(Debug, Deserialize)]
struct TypeRef {
    base: String,
    #[serde(default)]
    pointer: u32,
}

#[derive(Debug, Deserialize)]
struct Param {
    name: String,
    #[serde(rename = "type")]
    type_ref: TypeRef,
    #[serde(default)]
    role: String,
}

#[derive(Debug, Deserialize)]
struct Function {
    name: String,
    returns: TypeRef,
    params: Vec<Param>,
}

#[derive(Debug, Deserialize)]
struct Named {
    name: String,
}

#[derive(Debug, Deserialize)]
struct Idl {
    version: String,
    functions: Vec<Function>,
    #[serde(default)]
    structs: Vec<Named>,
    #[serde(default)]
    enums: Vec<Named>,
    #[serde(default)]
    callbacks: Vec<Named>,
}

/// Where a function stands with respect to the passthrough.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Standing {
    /// Callable at `sdk.engine.*` today.
    Callable,
    /// The adapter owns what it touches; never offered.
    Owned,
    /// The marshaller has not learned its shape yet.
    Unlearned,
}

/// The scalar bases a value crosses JSON as without ceremony.
const SCALARS: [&str; 11] = [
    "double", "float", "int32_t", "uint32_t", "int64_t", "uint64_t", "size_t", "int", "unsigned",
    "bool", "char",
];

fn plain(type_ref: &TypeRef, enums: &[String]) -> bool {
    type_ref.pointer == 0
        && (SCALARS.contains(&type_ref.base.as_str()) || enums.contains(&type_ref.base))
}

fn scalar_out(type_ref: &TypeRef, enums: &[String]) -> bool {
    type_ref.pointer == 1
        && (SCALARS.contains(&type_ref.base.as_str()) || enums.contains(&type_ref.base))
}

/// Where one function stands, and why.
fn standing(function: &Function, enums: &[String]) -> (Standing, &'static str) {
    if OWNED_BY_THE_ADAPTER.contains(&function.name.as_str())
        || function.name.starts_with(OWNED_PREFIX)
    {
        return (Standing::Owned, "the adapter's own context or data");
    }
    let roles: Vec<&str> = function.params.iter().map(|p| p.role.as_str()).collect();
    let shape_known = roles
        .iter()
        .all(|role| matches!(*role, "handle" | "value" | "scalar_out"));
    if !shape_known {
        let unknown = roles
            .iter()
            .find(|role| !matches!(**role, "handle" | "value" | "scalar_out"))
            .copied()
            .unwrap_or("?");
        return (
            Standing::Unlearned,
            match unknown {
                "struct_in" | "struct_out" | "out_struct_size" => "a struct",
                "array_in" | "array_len" | "array_out" | "array_cap" | "array_out_parallel" => {
                    "an array"
                }
                "string_out" | "string_cap" => "a string it fills",
                "string_in" => "a string it reads",
                "handle_out" => "a handle it creates",
                "opaque" => "opaque bytes",
                _ => "a role the marshaller does not know",
            },
        );
    }
    let args_plain = function
        .params
        .iter()
        .filter(|p| p.role == "value")
        .all(|p| plain(&p.type_ref, enums));
    let outs_plain = function
        .params
        .iter()
        .filter(|p| p.role == "scalar_out")
        .all(|p| scalar_out(&p.type_ref, enums));
    let returns_plain = function.returns.pointer == 0
        && (SCALARS.contains(&function.returns.base.as_str())
            || enums.contains(&function.returns.base)
            || function.returns.base == "void");
    if args_plain && outs_plain && returns_plain {
        (Standing::Callable, "scalars and enums only")
    } else if function.returns.pointer == 1 && function.returns.base == "char" {
        (Standing::Unlearned, "a string it returns")
    } else if !returns_plain {
        (Standing::Unlearned, "a pointer it returns")
    } else {
        (Standing::Unlearned, "a type that is not a plain scalar")
    }
}

/// The page and the marshalling, from one reading of the description, so
/// a figure on the page and the code that answers it cannot disagree.
fn outputs(root: &Path) -> Result<Vec<Output>, String> {
    let text = std::fs::read_to_string(root.join(IDL))
        .map_err(|error| format!("{IDL} is not readable: {error}"))?;
    let idl: Idl =
        serde_json::from_str(&text).map_err(|error| format!("{IDL} does not parse: {error}"))?;
    let enums: Vec<String> = idl.enums.iter().map(|e| e.name.clone()).collect();
    let classified: Vec<(&Function, Standing, &'static str)> = idl
        .functions
        .iter()
        .map(|function| {
            let (standing, why) = standing(function, &enums);
            (function, standing, why)
        })
        .collect();
    Ok(vec![
        Output::new(PAGE, page(&idl, &classified)),
        Output::new(DISPATCH, dispatch(&idl, &classified)),
    ])
}

/// Runs the classification and writes both.
pub(crate) fn generate(root: &Path) -> i32 {
    match outputs(root) {
        Ok(outputs) => write(root, &outputs),
        Err(message) => {
            eprintln!("{message}");
            1
        }
    }
}

/// Regenerates both in memory and fails on any difference.
pub(crate) fn check_generated(root: &Path) -> i32 {
    match outputs(root) {
        Ok(outputs) => check(root, &outputs, "cargo xtask engine"),
        Err(message) => {
            eprintln!("{message}");
            1
        }
    }
}

/// The measurement, from a reading already done.
fn page(idl: &Idl, classified: &[(&Function, Standing, &'static str)]) -> String {
    let mut by_standing: BTreeMap<Standing, Vec<(&str, &'static str, bool)>> = BTreeMap::new();
    for (function, standing, why) in classified {
        by_standing.entry(*standing).or_default().push((
            function.name.as_str(),
            why,
            mutates_engine_state(&function.name),
        ));
    }
    let callable = by_standing.get(&Standing::Callable).map_or(0, Vec::len);
    let owned = by_standing.get(&Standing::Owned).map_or(0, Vec::len);
    let unlearned = by_standing.get(&Standing::Unlearned).map_or(0, Vec::len);

    let mut out = String::new();
    let _ = writeln!(out, "# The engine passthrough, measured\n");
    let _ = writeln!(
        out,
        "Status: `generated` by `cargo xtask engine`, gated by `check-engine`. Do not edit. Read from `{IDL}`, the engine's own generated description, vendored beside the adapter at version `{}`. The same reading writes `{DISPATCH}`, so a figure here and the code that answers it cannot disagree.\n",
        idl.version
    );
    let _ = writeln!(
        out,
        "ADR-0030 puts an engine's own functions at `sdk.engine.*`, so that a consumer can reach what the SDK has not ported. The port says how — \"the marshalling belongs to the adapter, generated from the engine's manifest\" — and this is the measurement that generation is sized from.\n"
    );
    let _ = writeln!(
        out,
        "The engine describes **{} functions**, beside {} structs, {} enums and {} callbacks.\n",
        count(idl.functions.len()),
        count(idl.structs.len()),
        count(idl.enums.len()),
        count(idl.callbacks.len()),
    );

    let _ = writeln!(out, "| standing | functions | what it means |");
    let _ = writeln!(out, "|---|---:|---|");
    let _ = writeln!(
        out,
        "| **callable** | {callable} | offered at `sdk.engine.*` today |"
    );
    let _ = writeln!(
        out,
        "| **the adapter's own** | {owned} | never offered, whatever their shape |"
    );
    let _ = writeln!(
        out,
        "| **not yet marshalled** | {unlearned} | a queue for the generator, not a refusal |"
    );
    let _ = writeln!(out);

    out.push_str(&owned_section(by_standing.get(&Standing::Owned)));
    out.push_str(&callable_section(by_standing.get(&Standing::Callable)));
    out.push_str(&queue_section(by_standing.get(&Standing::Unlearned)));
    out.push_str(&limits_section());
    out
}

/// The Rust spelling of an IDL type, as the `sys` crate names it.
///
/// The engine's enums are `c_int` aliases in `sys`, so a name that is not
/// a scalar is an alias and passes through unchanged: the cast does the
/// work and nothing here has to carry a table of forty enums.
fn rust_type(base: &str) -> String {
    match base {
        "double" => "f64".to_string(),
        "float" => "f32".to_string(),
        "int32_t" => "i32".to_string(),
        "uint32_t" => "u32".to_string(),
        "int64_t" => "i64".to_string(),
        "uint64_t" => "u64".to_string(),
        "size_t" => "usize".to_string(),
        "int" => "::core::ffi::c_int".to_string(),
        "bool" => "bool".to_string(),
        other => format!("sys::{other}"),
    }
}

/// Whether a base crosses JSON as a number with a fractional part.
fn is_float(base: &str) -> bool {
    matches!(base, "double" | "float")
}

/// The marshalling: one arm per callable function, generated.
fn dispatch(idl: &Idl, classified: &[(&Function, Standing, &'static str)]) -> String {
    let callable: Vec<&Function> = classified
        .iter()
        .filter(|(_, standing, _)| *standing == Standing::Callable)
        .map(|(function, _, _)| *function)
        .collect();

    let mut out = String::new();
    let _ = writeln!(
        out,
        "//! The engine's own functions, reachable by name. **Generated by\n\
         //! `cargo xtask engine`. Do not edit.**\n\
         //!\n\
         //! One arm per function the marshaller can carry, from the engine's\n\
         //! own description at version `{}`. What is here and what is not is\n\
         //! measured in `docs/03-design/engine-passthrough-measured.md`: the\n\
         //! functions this adapter will never offer are excluded by what they\n\
         //! touch, and the rest are a queue for the generator.\n\
         //!\n\
         //! This is the adapter's only `unsafe`. Everything else goes through\n\
         //! the safe binding; these are the calls that binding does not wrap,\n\
         //! which is the whole reason a consumer would reach for them.\n\
         \n\
         #![allow(\n    \
         unsafe_code,\n    \
         clippy::too_many_lines,\n    \
         reason = \"generated: one arm per engine function\"\n\
         )]\n\
         \n\
         use serde_json::{{Map, Value, json}};\n\
         use teimeris::sys;\n\
         use teistro_port_ephemeris::ProviderError;\n\
         \n\
         use crate::passthrough::{{narrow, number, status}};\n",
        idl.version
    );

    // The manifest: what a caller may ask for, and the role of each
    // parameter, which is what `sdk.engine.signature(name)` reads.
    let mut functions_json = Vec::new();
    for function in &callable {
        let params: Vec<String> = function
            .params
            .iter()
            .filter(|p| p.role != "handle")
            .map(|p| {
                let role = if p.role == "scalar_out" { "out" } else { "in" };
                format!(
                    "{{\"name\":\"{}\",\"role\":\"{role}\",\"type\":\"{}\"}}",
                    p.name, p.type_ref.base
                )
            })
            .collect();
        let returns = if function.returns.base == "tm_status" {
            "status".to_string()
        } else if function.returns.base == "void" {
            "void".to_string()
        } else {
            function.returns.base.clone()
        };
        functions_json.push(format!(
            "{{\"name\":\"{}\",\"params\":[{}],\"returns\":\"{returns}\",\"mutatesEngineState\":{}}}",
            function.name,
            params.join(","),
            mutates_engine_state(&function.name)
        ));
    }
    let _ = writeln!(
        out,
        "/// What this adapter offers of the engine's own surface, as the\n\
         /// port's `native_manifest` answers it.\n\
         ///\n\
         /// `mutatesEngineState` is the flag a consumer needs and no other\n\
         /// manifest would carry: after such a call the engine answers under\n\
         /// settings the SDK's provenance does not record.\n\
         pub(crate) const MANIFEST: &str = r#\"{{\"engine\":\"teimeris\",\"version\":\"{}\",\"functions\":[{}]}}\"#;\n",
        idl.version,
        functions_json.join(",")
    );

    let _ = writeln!(
        out,
        "/// Calls one of them by name.\n\
         ///\n\
         /// # Errors\n\
         ///\n\
         /// `Unsupported` for a name this adapter does not offer, `Invalid`\n\
         /// for an argument missing or of the wrong kind, and the engine's\n\
         /// own refusal otherwise.\n\
         pub(crate) fn call(\n    \
         context: *mut sys::tm_context,\n    \
         function: &str,\n    \
         args: &Map<String, Value>,\n\
         ) -> Result<Value, ProviderError> {{\n    \
         match function {{"
    );
    for function in &callable {
        out.push_str(&arm(function));
    }
    let _ = writeln!(
        out,
        "        other => Err(ProviderError::unsupported(format!(\n            \
         \"the engine offers no `{{other}}` through this adapter; \\\n             \
         `native_manifest` lists what it does\"\n        ))),\n    \
         }}\n\
         }}"
    );
    out
}

/// One function's arm: read the arguments, call, answer.
fn arm(function: &Function) -> String {
    let mut out = String::new();
    let name = &function.name;
    let _ = writeln!(out, "        \"{name}\" => {{");

    let mut call_args: Vec<String> = Vec::new();
    let mut outs: Vec<&str> = Vec::new();
    for param in &function.params {
        match param.role.as_str() {
            "handle" => call_args.push("context".to_string()),
            "value" => {
                let ty = rust_type(&param.type_ref.base);
                // A float is read as one; anything else is **narrowed** to
                // the width the engine declares, and refused when it does
                // not fit rather than silently keeping the low bits.
                let _ = if is_float(&param.type_ref.base) {
                    // `number` answers `f64` already; only a `float`
                    // parameter needs narrowing, and that one is lossy by
                    // the engine's own declaration rather than by ours.
                    let cast = if ty == "f64" {
                        String::new()
                    } else {
                        format!(" as {ty}")
                    };
                    writeln!(
                        out,
                        "            let {}: {ty} = number(args, \"{}\")?{cast};",
                        param.name, param.name
                    )
                } else {
                    writeln!(
                        out,
                        "            let {}: {ty} = narrow(args, \"{}\")?;",
                        param.name, param.name
                    )
                };
                call_args.push(param.name.clone());
            }
            "scalar_out" => {
                let ty = rust_type(&param.type_ref.base);
                let zero = if is_float(&param.type_ref.base) {
                    "0.0"
                } else {
                    "0"
                };
                let _ = writeln!(out, "            let mut {}: {ty} = {zero};", param.name);
                call_args.push(format!("&raw mut {}", param.name));
                outs.push(param.name.as_str());
            }
            _ => unreachable!("only a callable function reaches here"),
        }
    }

    let call = format!("sys::{name}({})", call_args.join(", "));
    let binding = if function.returns.base == "void" {
        String::new()
    } else {
        "let answered = ".to_string()
    };
    let _ = writeln!(
        out,
        "            // SAFETY: the context is the adapter's own, live for\n            \
         // this call; every out-parameter is a local of the declared width.\n            \
         {binding}unsafe {{ {call} }};"
    );
    if function.returns.base == "tm_status" {
        let _ = writeln!(out, "            status(answered, \"{name}\")?;");
    }
    let mut fields: Vec<String> = outs
        .iter()
        // No cast at all: `serde_json` serialises every primitive
        // numeric, and the widths here are the engine's own. A cast would
        // be a claim about which of them was declared.
        .map(|name| format!("\"{name}\": {name}"))
        .collect();
    if !matches!(function.returns.base.as_str(), "tm_status" | "void") {
        fields.push("\"return\": answered".to_string());
    }
    let _ = writeln!(out, "            Ok(json!({{{}}}))", fields.join(", "));
    let _ = writeln!(out, "        }}");
    out
}

fn owned_section(rows: Option<&Vec<(&str, &'static str, bool)>>) -> String {
    let mut out = String::new();
    let Some(rows) = rows else { return out };
    let _ = writeln!(out, "## What the adapter will not hand over\n");
    let _ = writeln!(
        out,
        "**{} functions, excluded by what they touch rather than by what they cost.** Each opens, closes or rebinds the context the adapter is holding, or the data bound to it for that context's life. A consumer who called one through the passthrough would close the context every other call depends on, or change the files underneath it.\n",
        spelled(rows.len())
    );
    let _ = writeln!(
        out,
        "They are named here rather than left to the marshaller to fail on, because the reason is a **boundary** and not a difficulty — teaching the generator more shapes must never bring them in.\n"
    );
    for (name, _, _) in rows {
        let _ = writeln!(out, "- `{name}`");
    }
    let _ = writeln!(out);
    out
}

fn callable_section(rows: Option<&Vec<(&str, &'static str, bool)>>) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "## What is callable\n");
    let Some(rows) = rows else {
        let _ = writeln!(out, "Nothing yet.\n");
        return out;
    };
    let mutating: Vec<&str> = rows
        .iter()
        .filter(|(_, _, mutates)| *mutates)
        .map(|(name, _, _)| *name)
        .collect();
    let _ = writeln!(
        out,
        "**{} functions**, every one of them taking and returning scalars and enums alone, which is the shape a JSON object carries without a marshaller having to know anything else.\n",
        spelled(rows.len())
    );
    let _ = writeln!(out, "| function | changes engine state |");
    let _ = writeln!(out, "|---|---|");
    for (name, _, mutates) in rows {
        let _ = writeln!(
            out,
            "| `{name}` | {} |",
            if *mutates { "**yes**" } else { "" }
        );
    }
    let _ = writeln!(out);
    if !mutating.is_empty() {
        let _ = writeln!(
            out,
            "### The {} that change engine state\n",
            spelled(mutating.len())
        );
        let _ = writeln!(
            out,
            "These are **offered rather than refused**, and the column above is why the distinction is drawn at all. Reaching what the SDK has not ported is the point of the namespace; but after one of these the engine is answering under settings the SDK's own provenance does not record, so a chart cast afterwards says it was computed one way and was computed another.\n"
        );
        let _ = writeln!(
            out,
            "A consumer who wants the change *recorded* has the settings for it (ADR-0013's override policy), and one who wants it anyway can have it and knows what it costs. What the SDK will not do is make the choice quietly on their behalf.\n"
        );
    }
    out
}

fn queue_section(rows: Option<&Vec<(&str, &'static str, bool)>>) -> String {
    let mut out = String::new();
    let Some(rows) = rows else { return out };
    let _ = writeln!(out, "## What the marshaller has not learned\n");
    let mut by_reason: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (name, why, _) in rows {
        by_reason.entry(why).or_default().push(name);
    }
    let _ = writeln!(
        out,
        "**{} functions**, grouped by what stands in the way. This is a queue rather than a refusal: each group is one shape the generator has to learn, and learning one brings its whole group in at once.\n",
        spelled(rows.len())
    );
    let _ = writeln!(out, "| what it takes or returns | functions | examples |");
    let _ = writeln!(out, "|---|---:|---|");
    for (why, names) in &by_reason {
        let examples: Vec<String> = names.iter().take(3).map(|n| format!("`{n}`")).collect();
        let _ = writeln!(out, "| {why} | {} | {} |", names.len(), examples.join(", "));
    }
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "The order to learn them in is **not** the order of that table by size. A string the engine *returns* is one `CStr` read and brings its whole group at once; a string it *fills* and an array it fills share one protocol — ask the size, allocate, ask again — so whichever is learned first brings the other nearly free; a struct is fifty-seven field lists and is the largest group because it is the largest job. Opaque bytes are last and may stay there: what crosses is a buffer, and a buffer has no meaning in a JSON object.\n"
    );
    out
}

fn limits_section() -> String {
    let mut out = String::new();
    let _ = writeln!(out, "## What this does not measure\n");
    let _ = writeln!(
        out,
        "**Whether a callable function answers correctly.** This reads a description and classifies shapes; it does not call anything. What the marshalling produces is tested against the engine where the adapter's own tests run, and a function's presence here is a claim about its *shape* alone.\n"
    );
    let _ = writeln!(
        out,
        "**Any engine but this one.** The classification is of one vendored description. Another engine that answers `native_manifest` reaches `sdk.engine.call` by the dynamic route with no generation at all, and gets no typed façade until someone generates one from its description.\n"
    );
    let _ = writeln!(
        out,
        "**Whether a consumer should use any of it.** A call through this namespace is to a named engine and does not survive changing it — which is why the namespace is called `engine` and not `ephemeris` (ADR-0030). What proves universal is promoted into the port, and then it is portable and this page is no longer where it lives.\n"
    );
    out
}
