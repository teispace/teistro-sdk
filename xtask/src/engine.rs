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
const OWNED_BY_THE_ADAPTER: [&str; 4] = [
    "tm_context_open",
    "tm_context_close",
    "tm_context_set_fetch",
    // Binds a data source to the context for that context's life. It is
    // outside the prefix below because it is named for the context and
    // not for the source, and it belongs here for the same reason as
    // the rest: the memory it binds is the adapter's.
    "tm_context_add_source",
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
    name.starts_with("tm_set_") || name.starts_with("tm_clear_") || ALSO_MUTATES.contains(&name)
}

/// Functions that change engine state without a name that says so.
///
/// The prefix rule above catches the engine's own `set`/`clear` families
/// and would silently miss these, which is worse than missing them
/// loudly: the column on the page exists so a consumer can see what a
/// call will do to every chart cast after it.
const ALSO_MUTATES: [&str; 1] = [
    // Loads a star catalogue into the context, so `tm_star_calc` after
    // it resolves names it did not resolve before.
    "tm_star_load_catalogue",
];

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

/// A public typedef over a plain arithmetic type, as the engine
/// describes it.
///
/// The engine has three — `tm_body`, `tm_flags`, `tm_ayanamsha` — and
/// each is an integer a caller passes and reads like any other. They are
/// read rather than listed here for the reason the engine's own
/// extractor gives: a name that is not in the description is
/// indistinguishable from `tm_context`, which is a handle and must never
/// be treated as a number, and a hand-written list of the difference is
/// right only until the next header.
#[derive(Debug, Deserialize)]
struct Alias {
    name: String,
    base: String,
}

#[derive(Debug, Deserialize)]
struct Idl {
    version: String,
    functions: Vec<Function>,
    #[serde(default)]
    aliases: Vec<Alias>,
    #[serde(default)]
    structs: Vec<Named>,
    #[serde(default)]
    enums: Vec<Named>,
    #[serde(default)]
    callbacks: Vec<Named>,
}

/// One function as the page prints it: the description it came from, so
/// every figure on the page is computed from the same reading, and the
/// reason the classifier gave.
struct Row<'a> {
    function: &'a Function,
    why: &'a str,
}

impl Row<'_> {
    fn name(&self) -> &str {
        &self.function.name
    }

    fn mutates(&self) -> bool {
        mutates_engine_state(&self.function.name)
    }
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

/// The engine's own names for the things it takes and answers.
///
/// One place, because the two questions the classifier asks — is this a
/// number, and is that number fractional — must be asked of the same
/// vocabulary, and the second is the one an alias would silently answer
/// wrong: a `scalar_out` of a base this does not resolve gets `0` for
/// its initial value whether or not `0.0` was meant.
struct Vocabulary {
    enums: Vec<String>,
    aliases: BTreeMap<String, String>,
}

impl Vocabulary {
    fn new(idl: &Idl) -> Self {
        Self {
            enums: idl.enums.iter().map(|e| e.name.clone()).collect(),
            aliases: idl
                .aliases
                .iter()
                .map(|a| (a.name.clone(), a.base.clone()))
                .collect(),
        }
    }

    /// A public alias followed to the arithmetic type it stands for;
    /// anything else unchanged.
    fn resolve<'a>(&'a self, base: &'a str) -> &'a str {
        self.aliases.get(base).map_or(base, String::as_str)
    }

    /// Whether a base crosses JSON as a number.
    fn is_number(&self, base: &str) -> bool {
        SCALARS.contains(&self.resolve(base)) || self.enums.iter().any(|e| e == base)
    }

    /// Whether a base crosses JSON as a number with a fractional part.
    fn is_float(&self, base: &str) -> bool {
        matches!(self.resolve(base), "double" | "float")
    }
}

fn plain(type_ref: &TypeRef, vocabulary: &Vocabulary) -> bool {
    type_ref.pointer == 0 && vocabulary.is_number(&type_ref.base)
}

fn scalar_out(type_ref: &TypeRef, vocabulary: &Vocabulary) -> bool {
    type_ref.pointer == 1 && vocabulary.is_number(&type_ref.base)
}

/// The parameter roles the marshaller carries.
///
/// One list, read by the classifier and by nothing else: a role added
/// here without an arm in [`arm`] would generate code that does not
/// compile, which is the failure mode to want.
const ROLES_KNOWN: [&str; 6] = [
    "handle",
    "value",
    "scalar_out",
    "string_in",
    "string_out",
    "string_cap",
];

/// Whether a parameter is one the marshaller does not report and the
/// caller does not pass: the context it is called on, and the capacity
/// of a buffer the marshalling itself sizes.
fn bookkeeping(role: &str) -> bool {
    matches!(role, "handle" | "string_cap")
}

/// Where one function stands, and why.
fn standing(function: &Function, vocabulary: &Vocabulary) -> (Standing, &'static str) {
    if OWNED_BY_THE_ADAPTER.contains(&function.name.as_str())
        || function.name.starts_with(OWNED_PREFIX)
    {
        return (Standing::Owned, "the adapter's own context or data");
    }
    // A handle the adapter is not holding cannot be filled in from
    // anywhere: the arm has one context and no way to name another. Every
    // such function today is excluded above, so this classifies nothing —
    // it is here so that one the engine adds later is queued rather than
    // silently generated with the wrong pointer.
    if function
        .params
        .iter()
        .any(|p| p.role == "handle" && p.type_ref.base != "tm_context")
    {
        return (Standing::Unlearned, "a handle the adapter does not hold");
    }
    let roles: Vec<&str> = function.params.iter().map(|p| p.role.as_str()).collect();
    // The **hardest** thing in the way, not the first one in parameter
    // order. A function that takes an array of structs is queued behind
    // structs and not behind arrays, and saying otherwise put a sentence
    // on the page — "arrays are the next tranche and nearly free" — that
    // was true of the shape and false of the payoff.
    if let Some(reason) = function
        .params
        .iter()
        .filter_map(|param| blocked_by(param, vocabulary))
        .max_by_key(|reason| reason.rank())
    {
        return (Standing::Unlearned, reason.wording());
    }
    let args_plain = function
        .params
        .iter()
        .filter(|p| p.role == "value")
        .all(|p| plain(&p.type_ref, vocabulary));
    let outs_plain = function
        .params
        .iter()
        .filter(|p| p.role == "scalar_out")
        .all(|p| scalar_out(&p.type_ref, vocabulary));
    let strings_plain = function
        .params
        .iter()
        .filter(|p| matches!(p.role.as_str(), "string_in" | "string_out"))
        .all(|p| p.type_ref.pointer == 1 && p.type_ref.base == "char");
    let returns_plain = function.returns.pointer == 0
        && (vocabulary.is_number(&function.returns.base) || function.returns.base == "void");
    // A buffer the engine fills is only fillable by its own protocol —
    // fill, and be told the length it wanted — so a `string_out` without
    // a capacity beside it, or one whose function answers something other
    // than that length, is a shape this generator has not been shown.
    if fills_a_string(function) {
        let paired = roles.contains(&"string_cap");
        let answers_length = function.returns.pointer == 0 && function.returns.base == "size_t";
        if !(paired && answers_length) {
            return (
                Standing::Unlearned,
                "a buffer it fills by no protocol this generator knows",
            );
        }
    } else if roles.contains(&"string_cap") {
        return (Standing::Unlearned, "a capacity with no buffer beside it");
    }
    let returns_known = returns_plain || returns_a_string(function);
    if args_plain && outs_plain && strings_plain && returns_known {
        (Standing::Callable, "shapes the marshaller carries")
    } else if !returns_known {
        (Standing::Unlearned, "a pointer it returns")
    } else {
        (Standing::Unlearned, "a type that is not a plain scalar")
    }
}

/// A shape the marshaller does not carry, ordered by how much work it
/// is to learn.
///
/// The order is the point: a function held up by several of these is
/// held up by the hardest, and grouping it under an easier one would
/// promise that learning the easy one released it.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Blocker {
    /// An array whose elements are numbers: the fill protocol the string
    /// tranche already runs, with a width instead of a byte.
    NumberArray,
    /// A handle the caller would then have to hold and eventually free.
    HandleOut,
    /// Bytes with no meaning of their own.
    Opaque,
    /// A struct, whether passed alone or as an array's element.
    Struct,
    /// Something the description says and this generator has never seen.
    Unknown,
}

impl Blocker {
    /// How hard, for choosing between several on one function.
    fn rank(self) -> u8 {
        match self {
            Self::NumberArray => 0,
            Self::HandleOut => 1,
            Self::Opaque => 2,
            Self::Struct => 3,
            Self::Unknown => 4,
        }
    }

    /// What the page's table calls it.
    fn wording(self) -> &'static str {
        match self {
            Self::NumberArray => "an array of numbers",
            Self::HandleOut => "a handle it creates",
            Self::Opaque => "opaque bytes",
            Self::Struct => "a struct",
            Self::Unknown => "a role the marshaller does not know",
        }
    }
}

/// What stands in the way of one parameter, if anything does.
fn blocked_by(param: &Param, vocabulary: &Vocabulary) -> Option<Blocker> {
    if ROLES_KNOWN.contains(&param.role.as_str()) {
        return None;
    }
    Some(match param.role.as_str() {
        "struct_in" | "struct_out" | "out_struct_size" => Blocker::Struct,
        // An array is its elements. One of numbers is the protocol
        // already running; one of structs is the struct job wearing a
        // count, and belongs in that group where its size can be seen.
        "array_in" | "array_out" | "array_out_parallel" => {
            if vocabulary.is_number(&param.type_ref.base) {
                Blocker::NumberArray
            } else {
                Blocker::Struct
            }
        }
        // A length or a capacity is the marshaller's own bookkeeping and
        // never the reason: whatever it counts is beside it and says so.
        "array_len" | "array_cap" => Blocker::NumberArray,
        "handle_out" => Blocker::HandleOut,
        "opaque" => Blocker::Opaque,
        _ => Blocker::Unknown,
    })
}

/// Whether the function's own return value is a string it lends.
fn returns_a_string(function: &Function) -> bool {
    function.returns.pointer == 1 && function.returns.base == "char"
}

/// Whether the function fills a buffer of the caller's with a string.
fn fills_a_string(function: &Function) -> bool {
    has_role(function, "string_out")
}

/// The roles that describe an array, for the one question the page asks
/// that the classifier does not.
const ARRAY_ROLES: [&str; 5] = [
    "array_in",
    "array_len",
    "array_out",
    "array_cap",
    "array_out_parallel",
];

/// Whether any parameter plays the given role.
fn has_role(function: &Function, role: &str) -> bool {
    function.params.iter().any(|p| p.role == role)
}

/// How many rows describe a function satisfying a predicate, as the
/// page's prose needs it.
///
/// Over the description rather than over the row, so the predicates are
/// the same ones the classifier used and no second definition of "takes
/// a string" can drift away from the first.
fn count_where(rows: &[Row<'_>], predicate: impl Fn(&Function) -> bool) -> usize {
    rows.iter().filter(|row| predicate(row.function)).count()
}

/// What a callable function carries beyond numbers, for the table.
///
/// Read from the description rather than from the arm, so the column
/// cannot describe code that is no longer generated.
fn carries(function: &Function) -> &'static str {
    match (
        has_role(function, "string_in"),
        fills_a_string(function),
        returns_a_string(function),
    ) {
        (true, _, _) => "a string it reads",
        (_, true, _) => "a string it fills",
        (_, _, true) => "a string it lends",
        _ => "",
    }
}

/// The page and the marshalling, from one reading of the description, so
/// a figure on the page and the code that answers it cannot disagree.
fn outputs(root: &Path) -> Result<Vec<Output>, String> {
    let text = std::fs::read_to_string(root.join(IDL))
        .map_err(|error| format!("{IDL} is not readable: {error}"))?;
    let idl: Idl =
        serde_json::from_str(&text).map_err(|error| format!("{IDL} does not parse: {error}"))?;
    let vocabulary = Vocabulary::new(&idl);
    let classified: Vec<(&Function, Standing, &'static str)> = idl
        .functions
        .iter()
        .map(|function| {
            let (standing, why) = standing(function, &vocabulary);
            (function, standing, why)
        })
        .collect();
    Ok(vec![
        Output::new(PAGE, page(&idl, &classified, &vocabulary)),
        Output::new(DISPATCH, dispatch(&idl, &classified, &vocabulary)),
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
fn page(
    idl: &Idl,
    classified: &[(&Function, Standing, &'static str)],
    vocabulary: &Vocabulary,
) -> String {
    let mut by_standing: BTreeMap<Standing, Vec<Row<'_>>> = BTreeMap::new();
    for (function, standing, why) in classified {
        by_standing
            .entry(*standing)
            .or_default()
            .push(Row { function, why });
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
    out.push_str(&queue_section(
        by_standing.get(&Standing::Unlearned),
        idl.structs.len(),
        vocabulary,
    ));
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

/// The marshalling: one arm per callable function, generated.
fn dispatch(
    idl: &Idl,
    classified: &[(&Function, Standing, &'static str)],
    vocabulary: &Vocabulary,
) -> String {
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
         use crate::passthrough::{{borrowed, fill, narrow, number, status, text}};\n",
        idl.version
    );

    // The manifest: what a caller may ask for, and the role of each
    // parameter, which is what `sdk.engine.signature(name)` reads.
    let mut functions_json = Vec::new();
    for function in &callable {
        let params: Vec<String> = function
            .params
            .iter()
            .filter(|p| !bookkeeping(&p.role))
            .map(|p| {
                let role = if matches!(p.role.as_str(), "scalar_out" | "string_out") {
                    "out"
                } else {
                    "in"
                };
                let declared = if matches!(p.role.as_str(), "string_in" | "string_out") {
                    "string"
                } else {
                    p.type_ref.base.as_str()
                };
                format!("{{\"name\":\"{}\",\"role\":\"{role}\",\"type\":\"{declared}\"}}", p.name)
            })
            .collect();
        // `returns` is a word about the answer and not about C: `status`
        // is already not a type name, and a function whose return the
        // fill protocol consumes puts nothing under `return` either.
        let returns = if function.returns.base == "tm_status" {
            "status".to_string()
        } else if function.returns.base == "void" || fills_a_string(function) {
            "void".to_string()
        } else if returns_a_string(function) {
            "string".to_string()
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
        out.push_str(&arm(function, vocabulary));
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
fn arm(function: &Function, vocabulary: &Vocabulary) -> String {
    let mut out = String::new();
    let name = &function.name;
    let _ = writeln!(out, "        \"{name}\" => {{");

    let mut call_args: Vec<String> = Vec::new();
    let mut outs: Vec<&str> = Vec::new();
    // The buffer of a fill, if there is one: its name is what the string
    // comes back under, and the call is made inside a closure so the
    // protocol can run it twice.
    let mut buffer: Option<&str> = None;
    for param in &function.params {
        match param.role.as_str() {
            "handle" => call_args.push("context".to_string()),
            "string_in" => {
                // Bound, not inlined: the `CString` must outlive the call
                // and a temporary would be dropped at the semicolon.
                let _ = writeln!(
                    out,
                    "            let {} = text(args, \"{}\")?;",
                    param.name, param.name
                );
                call_args.push(format!("{}.as_ptr()", param.name));
            }
            "string_out" => {
                buffer = Some(param.name.as_str());
                call_args.push("buffer".to_string());
            }
            "string_cap" => call_args.push("capacity".to_string()),
            "value" => {
                let ty = rust_type(&param.type_ref.base);
                // A float is read as one; anything else is **narrowed** to
                // the width the engine declares, and refused when it does
                // not fit rather than silently keeping the low bits.
                let _ = if vocabulary.is_float(&param.type_ref.base) {
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
                let zero = if vocabulary.is_float(&param.type_ref.base) {
                    "0.0"
                } else {
                    "0"
                };
                let _ = writeln!(out, "            let mut {}: {ty} = {zero};", param.name);
                call_args.push(format!("&raw mut {}", param.name));
                outs.push(param.name.as_str());
            }
            other => unreachable!("a callable function has no `{other}` parameter"),
        }
    }

    let call = format!("sys::{name}({})", call_args.join(", "));
    let safety = safety(function);
    if let Some(buffer) = buffer {
        // The fill protocol owns the buffer and the count, and runs the
        // call again only if the first answer did not fit.
        let indented = safety.replace('\n', "\n                ");
        let _ = writeln!(
            out,
            "            let {buffer} = fill(\"{name}\", |buffer, capacity| {{\n                \
             {indented}\n                unsafe {{ {call} }}\n            }})?;"
        );
    } else {
        let binding = if function.returns.base == "void" {
            String::new()
        } else {
            "let answered = ".to_string()
        };
        let indented = safety.replace('\n', "\n            ");
        let _ = writeln!(
            out,
            "            {indented}\n            {binding}unsafe {{ {call} }};"
        );
        if function.returns.base == "tm_status" {
            let _ = writeln!(out, "            status(answered, \"{name}\")?;");
        }
    }
    let mut fields: Vec<String> = outs
        .iter()
        // No cast at all: `serde_json` serialises every primitive
        // numeric, and the widths here are the engine's own. A cast would
        // be a claim about which of them was declared.
        .map(|name| format!("\"{name}\": {name}"))
        .collect();
    // A filled string comes back under its parameter's own name, like
    // every other out-parameter. The length the function answered is the
    // protocol's and is not reported: the string it describes is already
    // here in full.
    if let Some(buffer) = buffer {
        fields.push(format!("\"{buffer}\": {buffer}"));
    } else if returns_a_string(function) {
        // `None` for null, which is how the engine says there is no such
        // name, and which JSON has a word for.
        fields.push("\"return\": borrowed(answered)".to_string());
    } else if !matches!(function.returns.base.as_str(), "tm_status" | "void") {
        fields.push("\"return\": answered".to_string());
    }
    let _ = writeln!(out, "            Ok(json!({{{}}}))", fields.join(", "));
    let _ = writeln!(out, "        }}");
    out
}

/// The SAFETY note for one arm, naming **what that arm actually
/// passes** and nothing else.
///
/// A generated note that lists an out-parameter the call does not have,
/// or a context it is not given, is worse than none: the whole reason
/// this file is allowed its `unsafe` is that the note beside each call
/// can be checked against the call.
fn safety(function: &Function) -> String {
    let mut clauses: Vec<&str> = Vec::new();
    if has_role(function, "handle") {
        clauses.push("the context is the adapter's own and live for this call");
    }
    if has_role(function, "scalar_out") {
        clauses.push("every out-parameter is a local of the width the engine declares");
    }
    if has_role(function, "string_in") {
        clauses.push("every string argument is a `CString` that outlives the call");
    }
    if fills_a_string(function) {
        clauses.push(
            "the buffer and its capacity are the fill protocol's own, and it passes the length \
             of what it allocated",
        );
    }
    if clauses.is_empty() {
        // Nothing is lent and nothing is written through: the arm passes
        // numbers the engine declared and that is the whole of it.
        clauses.push("the call takes nothing but values of the widths the engine declares");
    }
    wrapped(&format!("SAFETY: {}.", clauses.join("; ")))
}

/// A note as comment lines no wider than the project's own limit.
///
/// Unindented; the caller indents to the depth its arm sits at.
fn wrapped(note: &str) -> String {
    /// What is left for text once `            // ` is in front of it.
    const ROOM: usize = 60;

    let mut lines: Vec<String> = vec![String::new()];
    for word in note.split_whitespace() {
        let line = lines.last_mut().unwrap_or_else(|| unreachable!("never empty"));
        if line.is_empty() {
            line.push_str(word);
        } else if line.len() + 1 + word.len() <= ROOM {
            line.push(' ');
            line.push_str(word);
        } else {
            lines.push(word.to_string());
        }
    }
    lines
        .iter()
        .map(|line| format!("// {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn owned_section(rows: Option<&Vec<Row<'_>>>) -> String {
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
    for row in rows {
        let _ = writeln!(out, "- `{}`", row.name());
    }
    let _ = writeln!(out);
    out
}

fn callable_section(rows: Option<&Vec<Row<'_>>>) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "## What is callable\n");
    let Some(rows) = rows else {
        let _ = writeln!(out, "Nothing yet.\n");
        return out;
    };
    let mutating = count_where(rows, |function| mutates_engine_state(&function.name));
    // Counted from the same rows the table below prints, because the
    // sentence is a claim about them: an arm that carries a string is a
    // different amount of generated code from one that does not, and a
    // paragraph asserting "scalars alone" outlived the truth of it once
    // already.
    let takes = count_where(rows, |function| has_role(function, "string_in"));
    let fills = count_where(rows, fills_a_string);
    let lends = count_where(rows, returns_a_string);
    let scalar_only = rows.len() - takes - fills - lends;
    let _ = writeln!(
        out,
        "**{} functions**. {} of them take and answer scalars and enums alone, which is the shape a JSON object carries without a marshaller having to know anything else. The other {} carry a string: {} read one the caller passes, {} fill a buffer of the marshaller's, and {} answer with one the engine lends and the marshaller copies before anything else can move it.\n",
        spelled(rows.len()),
        spelled(scalar_only),
        spelled(takes + fills + lends),
        spelled(takes),
        spelled(fills),
        spelled(lends),
    );
    let _ = writeln!(out, "| function | carries | changes engine state |");
    let _ = writeln!(out, "|---|---|---|");
    for row in rows {
        let _ = writeln!(
            out,
            "| `{}` | {} | {} |",
            row.name(),
            carries(row.function),
            if row.mutates() { "**yes**" } else { "" }
        );
    }
    let _ = writeln!(out);
    if mutating > 0 {
        let _ = writeln!(out, "### The {} that change engine state\n", spelled(mutating));
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

fn queue_section(
    rows: Option<&Vec<Row<'_>>>,
    structs: usize,
    vocabulary: &Vocabulary,
) -> String {
    let mut out = String::new();
    let Some(rows) = rows else { return out };
    let _ = writeln!(out, "## What the marshaller has not learned\n");
    let mut by_reason: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for row in rows {
        by_reason.entry(row.why).or_default().push(row.name());
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
    // Grouped by the HARDEST thing in the way, which is what makes the
    // sizes mean anything: an array of structs counted as an array told
    // this page that arrays were the next tranche and nearly free, and
    // the payoff would have been three functions.
    let by_struct = by_reason.get("a struct").map_or(0, Vec::len);
    let by_array = by_reason.get("an array of numbers").map_or(0, Vec::len);
    let _ = writeln!(
        out,
        "**{} of the {} are behind structs**, and that is the finding. Nothing else in the queue is a tranche: an array of numbers would release {}, and the rest are ones and twos. A struct is the largest group because it is the largest job — {} field lists, each read out of the description and written into and out of a JSON object, several carrying a `struct_size` the engine reads before it writes — and it is now also the *only* group whose learning changes the shape of this page.\n",
        by_struct,
        rows.len(),
        spelled(by_array),
        spelled(structs)
    );
    // Counted from the rows: a function grouped under structs that also
    // has an array parameter is one this grouping moved.
    let moved = count_where(rows, |function| {
        function.params.iter().any(|param| {
            ARRAY_ROLES.contains(&param.role.as_str())
                && blocked_by(param, vocabulary) == Some(Blocker::Struct)
        })
    });
    let _ = writeln!(
        out,
        "This table used to group an array by the fact that it was an array, and said on that basis that arrays were the next tranche and nearly free. They are nearly free and they are not a tranche: **{moved} of them are arrays of structs**, which is the struct job wearing a count. Grouping by the hardest thing in the way rather than by the first one in parameter order is what made that visible.\n"
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
