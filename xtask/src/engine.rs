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
pub(crate) struct Param {
    name: String,
    #[serde(rename = "type")]
    type_ref: TypeRef,
    #[serde(default)]
    role: String,
    /// Whether the engine takes a null here. Its extractor marks every
    /// single struct input so, because whether null is allowed is in the
    /// header's prose and not in the type — and the engine refuses a null
    /// it does not allow with its own status, so passing one is safe.
    #[serde(default)]
    optional: bool,
}

impl Param {
    /// Whether a caller may leave this out, which crosses as null.
    fn nullable(&self) -> bool {
        self.optional && self.role == "struct_in"
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Function {
    name: String,
    returns: TypeRef,
    params: Vec<Param>,
}

#[derive(Debug, Deserialize)]
struct Named {
    name: String,
}

/// One field of a public struct, as the engine describes it.
#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Field {
    pub(crate) name: String,
    #[serde(rename = "type")]
    pub(crate) base: String,
    #[serde(default)]
    pub(crate) pointer: u32,
    #[serde(default, rename = "const")]
    pub(crate) constant: bool,
}

/// A public struct, with the field list its marshalling is generated
/// from.
#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Shape {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) fields: Vec<Field>,
}

impl Shape {
    /// The fields that cross, which is every field but the struct's own
    /// extent (`03-design/engine-passthrough.md` §2).
    pub(crate) fn crossing(&self) -> impl Iterator<Item = &Field> {
        self.fields.iter().filter(|field| field.name != STRUCT_SIZE)
    }

    /// Whether the struct carries its own extent as its first field.
    fn sized(&self) -> bool {
        self.fields
            .first()
            .is_some_and(|field| field.name == STRUCT_SIZE)
    }
}

/// The field a versioned struct carries its own extent in.
///
/// **Bookkeeping, and never crosses.** The engine reads a struct only as
/// far as this says, so a caller who could set it could tell the engine
/// the struct was larger than the one on the stack. Its only correct
/// value is the size of the struct the generated arm declared, which the
/// arm already knows.
const STRUCT_SIZE: &str = "struct_size";

/// What a struct's fields are made of: its **worst** field, because a
/// struct crosses only as completely as its hardest field does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Made {
    /// Numbers, enums, and nested structs that are plain themselves.
    Plain,
    /// A string, which has to outlive the call or be copied out of it.
    Strings,
    /// A pointer to another struct: an optional nested object.
    Pointers,
    /// A function pointer, a list of strings, or bytes: not an object.
    Other,
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
    structs: Vec<Shape>,
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
pub(crate) struct Vocabulary {
    enums: Vec<String>,
    aliases: BTreeMap<String, String>,
    structs: BTreeMap<String, Shape>,
    /// The structs' names in declaration order, which a map forgets.
    order: Vec<String>,
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
            structs: idl
                .structs
                .iter()
                .map(|shape| (shape.name.clone(), shape.clone()))
                .collect(),
            order: idl.structs.iter().map(|shape| shape.name.clone()).collect(),
        }
    }

    /// Every struct, in the order the engine declares them.
    pub(crate) fn declared(&self) -> impl Iterator<Item = &Shape> {
        self.order.iter().filter_map(|name| self.structs.get(name))
    }

    /// The struct a base names, if it names one.
    pub(crate) fn shape(&self, base: &str) -> Option<&Shape> {
        self.structs.get(base)
    }

    /// What a struct is made of; `Other` for a name that is not one.
    pub(crate) fn made_of(&self, base: &str) -> Made {
        self.made_within(base, &mut Vec::new())
    }

    fn made_within<'a>(&'a self, base: &str, seen: &mut Vec<&'a str>) -> Made {
        let Some(shape) = self.structs.get(base) else {
            return Made::Other;
        };
        // A struct that contains itself has no finite JSON object. The
        // engine has none; the guard keeps this total rather than a
        // stack overflow the day one appears.
        if seen.contains(&shape.name.as_str()) {
            return Made::Other;
        }
        seen.push(&shape.name);
        let worst = shape
            .fields
            .iter()
            .map(|field| self.made_of_field(field, seen))
            .max()
            .unwrap_or(Made::Plain);
        seen.pop();
        worst
    }

    fn made_of_field<'a>(&'a self, field: &Field, seen: &mut Vec<&'a str>) -> Made {
        let base = field.base.as_str();
        match field.pointer {
            0 if self.is_number(base) => Made::Plain,
            0 if self.structs.contains_key(base) => self.made_within(base, seen),
            1 if base == "char" && field.constant => Made::Strings,
            1 if self.structs.contains_key(base) => {
                Made::Pointers.max(self.made_within(base, seen))
            }
            _ => Made::Other,
        }
    }

    /// A public alias followed to the arithmetic type it stands for;
    /// anything else unchanged.
    fn resolve<'a>(&'a self, base: &'a str) -> &'a str {
        self.aliases.get(base).map_or(base, String::as_str)
    }

    /// Whether a base crosses JSON as a number.
    pub(crate) fn is_number(&self, base: &str) -> bool {
        SCALARS.contains(&self.resolve(base)) || self.enums.iter().any(|e| e == base)
    }

    /// Whether a base crosses JSON as a number with a fractional part.
    pub(crate) fn is_float(&self, base: &str) -> bool {
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
const ROLES_KNOWN: [&str; 9] = [
    "handle",
    "value",
    "scalar_out",
    "string_in",
    "string_out",
    "string_cap",
    "struct_in",
    "struct_out",
    "out_struct_size",
];

/// Whether a parameter is one the marshaller does not report and the
/// caller does not pass: the context it is called on, and the capacity
/// of a buffer the marshalling itself sizes.
fn bookkeeping(role: &str) -> bool {
    matches!(role, "handle" | "string_cap" | "out_struct_size")
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Blocker {
    /// An array whose elements are numbers: the fill protocol the string
    /// tranche already runs, with a width instead of a byte.
    NumberArray,
    /// An array of plain structs: the array protocol over a shape the
    /// marshaller already carries.
    StructArray,
    /// A struct carrying a string, which must outlive the call going in
    /// and be copied coming out.
    StringStruct,
    /// A struct pointing at another: a nested object that may be absent.
    PointerStruct,
    /// A handle the caller would then have to hold and eventually free.
    HandleOut,
    /// Bytes with no meaning of their own.
    Opaque,
    /// A struct carrying a function pointer, a list of strings or bytes,
    /// which no JSON object describes.
    OtherStruct,
    /// Something the description says and this generator has never seen.
    Unknown,
}

impl Blocker {
    /// Every blocker, in rank order, which is the order the page lists
    /// them in.
    const ALL: [Self; 8] = [
        Self::NumberArray,
        Self::StructArray,
        Self::StringStruct,
        Self::PointerStruct,
        Self::HandleOut,
        Self::Opaque,
        Self::OtherStruct,
        Self::Unknown,
    ];

    /// How hard, for choosing between several on one function.
    fn rank(self) -> usize {
        Self::ALL
            .iter()
            .position(|one| *one == self)
            .unwrap_or_else(|| unreachable!("every blocker is in ALL"))
    }

    /// What the page's table calls it.
    fn wording(self) -> &'static str {
        match self {
            Self::NumberArray => "an array of numbers",
            Self::StructArray => "an array of structs",
            Self::StringStruct => "a struct carrying a string",
            Self::PointerStruct => "a struct pointing at another",
            Self::HandleOut => "a handle it creates",
            Self::Opaque => "opaque bytes",
            Self::OtherStruct => "a struct no JSON object describes",
            Self::Unknown => "a role the marshaller does not know",
        }
    }

    /// What stands in the way of a struct, if anything does.
    const fn of_struct(made: Made) -> Option<Self> {
        match made {
            Made::Plain => None,
            Made::Strings => Some(Self::StringStruct),
            Made::Pointers => Some(Self::PointerStruct),
            Made::Other => Some(Self::OtherStruct),
        }
    }
}

/// What stands in the way of one parameter, if anything does.
fn blocked_by(param: &Param, vocabulary: &Vocabulary) -> Option<Blocker> {
    let base = param.type_ref.base.as_str();
    match param.role.as_str() {
        // A struct crosses when every field does, so what blocks it is
        // its worst field: that is the shape the generator would learn
        // next. "A struct" alone was one row over eighty-one functions
        // and four different jobs.
        "struct_in" | "struct_out" => Blocker::of_struct(vocabulary.made_of(base)),
        // An array is its elements. One of numbers is the protocol
        // already running; one of structs is blocked by its element's
        // worst field, or by the array protocol once that is plain.
        "array_in" | "array_out" | "array_out_parallel" => Some(if vocabulary.is_number(base) {
            Blocker::NumberArray
        } else {
            Blocker::of_struct(vocabulary.made_of(base)).unwrap_or(Blocker::StructArray)
        }),
        // A length or a capacity is the marshaller's own bookkeeping and
        // never the reason: whatever it counts is beside it and says so.
        "array_len" | "array_cap" => Some(Blocker::NumberArray),
        "handle_out" => Some(Blocker::HandleOut),
        "opaque" => Some(Blocker::Opaque),
        known if ROLES_KNOWN.contains(&known) => None,
        _ => Some(Blocker::Unknown),
    }
}

/// One callable function as **the manifest describes it**, which is the
/// only description a consumer ever sees.
///
/// Read once and shared, because the manifest and the typed façades must
/// agree about every name, role and type: a façade that typed an
/// argument the manifest calls something else would type-check a call
/// that the dispatch then refuses by name.
pub(crate) struct Described<'a> {
    /// The engine's own name, which `call` takes.
    pub(crate) name: &'a str,
    /// What the caller supplies: the key, and the manifest's word for
    /// its type.
    pub(crate) takes: Vec<(&'a str, &'a str)>,
    /// What comes back, keyed as the answer keys it. `return` is in here
    /// when the function's own return value is reported.
    pub(crate) gives: Vec<(&'a str, &'a str)>,
    /// Whether a call changes engine state the SDK's provenance does not
    /// record.
    pub(crate) mutates: bool,
    /// The keys in `takes` a caller may leave out or pass as null.
    nullable: Vec<&'a str>,
}

impl Described<'_> {
    /// Whether a key in `takes` may be left out.
    pub(crate) fn nullable(&self, key: &str) -> bool {
        self.nullable.contains(&key)
    }
}

/// The manifest's word for a parameter's type.
fn declared_type(param: &Param) -> &str {
    if matches!(param.role.as_str(), "string_in" | "string_out") {
        "string"
    } else {
        param.type_ref.base.as_str()
    }
}

/// The manifest's word for what a function answers with.
///
/// A word about the answer and not about C: `status` is already not a
/// type name, and a function whose return the fill protocol consumes
/// puts nothing under `return` either.
fn declared_return(function: &Function) -> &str {
    if function.returns.base == "tm_status" {
        "status"
    } else if function.returns.base == "void" || fills_a_string(function) {
        "void"
    } else if returns_a_string(function) {
        "string"
    } else {
        function.returns.base.as_str()
    }
}

/// One callable function, described once.
pub(crate) fn describe(function: &Function) -> Described<'_> {
    let mut takes = Vec::new();
    let mut gives = Vec::new();
    let mut nullable = Vec::new();
    for param in &function.params {
        if bookkeeping(&param.role) {
            continue;
        }
        if param.nullable() {
            nullable.push(param.name.as_str());
        }
        let entry = (param.name.as_str(), declared_type(param));
        if matches!(
            param.role.as_str(),
            "scalar_out" | "string_out" | "struct_out"
        ) {
            gives.push(entry);
        } else {
            takes.push(entry);
        }
    }
    let returns = declared_return(function);
    if !matches!(returns, "status" | "void") {
        gives.push(("return", returns));
    }
    Described {
        name: &function.name,
        takes,
        gives,
        mutates: mutates_engine_state(&function.name),
        nullable,
    }
}

/// Whether the function's own return value is a string it lends.
fn returns_a_string(function: &Function) -> bool {
    function.returns.pointer == 1 && function.returns.base == "char"
}

/// Whether the function fills a buffer of the caller's with a string.
fn fills_a_string(function: &Function) -> bool {
    has_role(function, "string_out")
}

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
fn carries(function: &Function) -> String {
    [
        (has_role(function, "string_in"), "a string it reads"),
        (fills_a_string(function), "a string it fills"),
        (returns_a_string(function), "a string it lends"),
        (has_role(function, "struct_in"), "a struct it reads"),
        (has_role(function, "struct_out"), "a struct it fills"),
    ]
    .into_iter()
    .filter_map(|(yes, clause)| yes.then_some(clause))
    .collect::<Vec<_>>()
    .join(", ")
}

/// Whether a function carries a string in any of the three ways.
fn carries_a_string(function: &Function) -> bool {
    has_role(function, "string_in") || fills_a_string(function) || returns_a_string(function)
}

/// Whether a function carries a struct either way.
fn carries_a_struct(function: &Function) -> bool {
    has_role(function, "struct_in") || has_role(function, "struct_out")
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
    let callable: Vec<&Function> = classified
        .iter()
        .filter(|(_, standing, _)| *standing == Standing::Callable)
        .map(|(function, _, _)| *function)
        .collect();
    let mut outputs = vec![
        Output::new(PAGE, page(&idl, &classified, &vocabulary)),
        Output::new(DISPATCH, dispatch(&idl, &classified, &vocabulary)),
    ];
    // The typed façades, from the same reading: a façade that typed an
    // argument the dispatch would refuse by name is the one thing
    // generating both from one description makes impossible (ADR-0030).
    outputs.extend(crate::facade::outputs(&idl.version, &callable, &vocabulary));
    Ok(outputs)
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
        vocabulary,
    ));
    out.push_str(&facade_section(
        by_standing
            .get(&Standing::Callable)
            .map_or(&[][..], |rows| rows.as_slice()),
    ));
    out.push_str(&limits_section());
    out
}

/// What shape the typed façade hands an answer back in, which is a
/// measurement rather than a taste.
fn facade_section(rows: &[Row<'_>]) -> String {
    let mut out = String::new();
    if rows.is_empty() {
        return out;
    }
    let mut by_count: BTreeMap<usize, Vec<&str>> = BTreeMap::new();
    for row in rows {
        by_count
            .entry(describe(row.function).gives.len())
            .or_default()
            .push(row.name());
    }
    let one = by_count.get(&1).map_or(0, Vec::len);
    let none = by_count.get(&0).map_or(0, Vec::len);
    let many: Vec<&str> = by_count
        .iter()
        .filter(|(count, _)| **count > 1)
        .flat_map(|(_, names)| names.iter().copied())
        .collect();
    let listed = many
        .iter()
        .map(|name| format!("`{name}`"))
        .collect::<Vec<_>>()
        .join(", ");

    let _ = writeln!(out, "## What the typed façade hands back\n");
    let _ = writeln!(
        out,
        "ADR-0030 puts a typed façade in the **adapter's** package, generated from this same reading so that it cannot type an argument the dispatch would refuse by name. What a method answers with is decided here, by counting:\n"
    );
    let _ = writeln!(out, "| values answered | functions | the façade's answer |");
    let _ = writeln!(out, "|---:|---:|---|");
    let _ = writeln!(
        out,
        "| 1 | {one} | **the value itself** — a number, a string, a struct |"
    );
    let _ = writeln!(out, "| 0 | {none} | nothing |");
    let _ = writeln!(
        out,
        "| more | {} | a record: an object, a Dart record, a `TypedDict` |",
        many.len()
    );
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "**{one} of the {} answer with exactly one value**, so a façade that always returned an object would have made every one of them an indexing exercise for the sake of {}. Those {} get a record apiece — {listed} — which is {} types per target rather than {}.\n",
        rows.len(),
        many.len(),
        many.len(),
        many.len(),
        rows.len()
    );
    let _ = writeln!(
        out,
        "The same count settles a question every target would otherwise have raised. `return` — the key a function's own return value comes back under — **never appears beside another key**: every one of those {} is a status-returning function with out-parameters. So no record field is ever named `return`, and no target has to rename a keyword it could not spell.\n",
        many.len()
    );
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
         {PASSTHROUGH_IMPORTS}\n",
        idl.version
    );

    // The manifest: what a caller may ask for, and the role of each
    // parameter, which is what `sdk.engine.signature(name)` reads.
    let mut functions_json = Vec::new();
    for function in &callable {
        let described = describe(function);
        // The manifest keeps `role` as `in`/`out` because that is what a
        // consumer reading `sdk.engine.signature(name)` needs, and the
        // description above is already grouped that way.
        let params: Vec<String> = described
            .takes
            .iter()
            .map(|(name, declared)| {
                // `optional` only where it is true, so the manifest of a
                // function with none reads as it always has.
                let optional = if described.nullable(name) {
                    ",\"optional\":true"
                } else {
                    ""
                };
                format!("{{\"name\":\"{name}\",\"role\":\"in\",\"type\":\"{declared}\"{optional}}}")
            })
            .chain(
                described
                    .gives
                    .iter()
                    .filter(|(name, _)| *name != "return")
                    .map(|(name, declared)| {
                        format!("{{\"name\":\"{name}\",\"role\":\"out\",\"type\":\"{declared}\"}}")
                    }),
            )
            .collect();
        functions_json.push(format!(
            "{{\"name\":\"{}\",\"params\":[{}],\"returns\":\"{}\",\"mutatesEngineState\":{}}}",
            described.name,
            params.join(","),
            declared_return(function),
            described.mutates
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
    out.push_str(&marshalling(&callable, vocabulary));
    out.replace(PASSTHROUGH_IMPORTS, &passthrough_imports(&out))
}

/// Where the `use` of the passthrough's helpers goes, filled in once the
/// file is written.
const PASSTHROUGH_IMPORTS: &str = "// the passthrough's helpers";

/// The helpers the generated file actually calls, and only those.
///
/// Read off the file rather than kept in step with the arms by hand: a
/// helper an engine version never needs — `object`, when every struct it
/// takes is optional — would otherwise be an unused import in the one
/// file nobody edits.
fn passthrough_imports(generated: &str) -> String {
    /// Every helper, with the text a use of it contains.
    const HELPERS: [(&str, &str); 9] = [
        ("Within", "Within<"),
        ("borrowed", "borrowed(answered"),
        ("fill", "fill(\""),
        ("narrow", "narrow(args"),
        ("number", "number(args"),
        ("object", " object(args"),
        ("optional_object", "optional_object(args"),
        ("status", "status(answered"),
        ("text", "text(args"),
    ];
    let used: Vec<&str> = HELPERS
        .iter()
        .filter(|(_, marker)| generated.contains(marker))
        .map(|(name, _)| *name)
        .collect();
    format!("use crate::passthrough::{{{}}};", used.join(", "))
}

/// The structs one role reaches, with every struct they nest, in the
/// order the engine declares them.
///
/// Emitted for exactly these and no others, so a reader is generated
/// only where something reads one: a struct nothing callable mentions
/// would be dead code in the adapter and a name in four façades.
///
/// # Panics
///
/// When a struct nests one the engine declares after it. The façades
/// that need declaration order rely on the engine's header order, and
/// this is where that assumption is checked rather than believed.
pub(crate) fn reached<'v>(
    callable: &[&Function],
    roles: &[&str],
    vocabulary: &'v Vocabulary,
) -> Vec<&'v Shape> {
    let mut names: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    let mut pending: Vec<&str> = callable
        .iter()
        .flat_map(|function| function.params.iter())
        .filter(|param| roles.contains(&param.role.as_str()))
        .map(|param| param.type_ref.base.as_str())
        .collect();
    while let Some(name) = pending.pop() {
        let Some(shape) = vocabulary.shape(name) else {
            continue;
        };
        if names.insert(&shape.name) {
            pending.extend(
                shape
                    .fields
                    .iter()
                    .filter(|field| field.pointer == 0)
                    .map(|field| field.base.as_str()),
            );
        }
    }
    let ordered: Vec<&Shape> = vocabulary
        .declared()
        .filter(|shape| names.contains(shape.name.as_str()))
        .collect();
    for (at, shape) in ordered.iter().enumerate() {
        for field in shape.fields.iter().filter(|field| field.pointer == 0) {
            if let Some(nested) = ordered.iter().position(|one| one.name == field.base) {
                assert!(
                    nested < at,
                    "`{}` nests `{}`, which the engine declares after it",
                    shape.name,
                    field.base
                );
            }
        }
    }
    ordered
}

/// A reader and a writer for every struct a callable function passes.
fn marshalling(callable: &[&Function], vocabulary: &Vocabulary) -> String {
    let mut out = String::new();
    for shape in reached(callable, &["struct_in"], vocabulary) {
        out.push_str(&reader(shape, vocabulary));
    }
    for shape in reached(callable, &["struct_out"], vocabulary) {
        out.push_str(&writer(shape, vocabulary));
    }
    out
}

/// One struct read out of the object a caller passed, every field
/// required.
fn reader(shape: &Shape, vocabulary: &Vocabulary) -> String {
    let name = &shape.name;
    let mut fields = Vec::new();
    if shape.sized() {
        fields.push(format!(
            "        {STRUCT_SIZE}: core::mem::size_of::<sys::{name}>(),"
        ));
    }
    for field in shape.crossing() {
        let key = &field.name;
        let read = if vocabulary.shape(&field.base).is_some() {
            format!("read_{}(&within.nested(\"{key}\")?)?", field.base)
        } else if vocabulary.is_float(&field.base) {
            let ty = rust_type(&field.base);
            if ty == "f64" {
                format!("within.number(\"{key}\")?")
            } else {
                format!("within.number(\"{key}\")? as {ty}")
            }
        } else {
            format!("within.narrow(\"{key}\")?")
        };
        fields.push(format!("        {key}: {read},"));
    }
    format!(
        "\n/// A `{name}` read out of the object a caller passed.\n\
         fn read_{name}(within: &Within<'_>) -> Result<sys::{name}, ProviderError> {{\n    \
         Ok(sys::{name} {{\n{}\n    }})\n}}\n",
        fields.join("\n")
    )
}

/// One struct written into the object a caller gets back.
fn writer(shape: &Shape, vocabulary: &Vocabulary) -> String {
    let name = &shape.name;
    let fields: Vec<String> = shape
        .crossing()
        .map(|field| {
            let key = &field.name;
            if vocabulary.shape(&field.base).is_some() {
                format!("        \"{key}\": write_{}(&value.{key}),", field.base)
            } else {
                format!("        \"{key}\": value.{key},")
            }
        })
        .collect();
    format!(
        "\n/// A `{name}` as the object a caller gets back.\n\
         fn write_{name}(value: &sys::{name}) -> Value {{\n    \
         json!({{\n{}\n    }})\n}}\n",
        fields.join("\n")
    )
}

/// One function's arm: read the arguments, call, answer.
fn arm(function: &Function, vocabulary: &Vocabulary) -> String {
    let mut out = String::new();
    let name = &function.name;
    let _ = writeln!(out, "        \"{name}\" => {{");

    let mut call_args: Vec<String> = Vec::new();
    // Each out-parameter as the answer's `"key": expression`.
    let mut outs: Vec<String> = Vec::new();
    let structs: Vec<&Param> = function
        .params
        .iter()
        .filter(|p| matches!(p.role.as_str(), "struct_in" | "struct_out"))
        .collect();
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
                outs.push(format!("\"{0}\": {0}", param.name));
            }
            "struct_in" | "struct_out" | "out_struct_size" => {
                struct_param(param, &structs, &mut out, &mut call_args, &mut outs);
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
    let fields = answer(function, outs, buffer);
    let _ = writeln!(out, "            Ok(json!({{{}}}))", fields.join(", "));
    let _ = writeln!(out, "        }}");
    out
}

/// What an arm answers with, as `"key": expression` pairs: its
/// out-parameters, then a filled string or the function's own return.
fn answer(function: &Function, outs: Vec<String>, buffer: Option<&str>) -> Vec<String> {
    // No cast on a number: `serde_json` serialises every primitive
    // numeric, and the widths here are the engine's own. A cast would be
    // a claim about which of them was declared.
    let mut fields = outs;
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
    fields
}

/// One struct parameter's part of an arm: the local it binds, what the
/// call is passed, and what the answer reports.
fn struct_param(
    param: &Param,
    structs: &[&Param],
    out: &mut String,
    call_args: &mut Vec<String>,
    outs: &mut Vec<String>,
) {
    let name = &param.name;
    let base = &param.type_ref.base;
    match param.role.as_str() {
        "struct_in" if param.nullable() => {
            // Left out or null crosses as a null pointer, and the engine
            // decides whether that is allowed: an observer is optional
            // unless the flags ask for a topocentric answer, and a
            // datetime never is. Both are the engine's refusal to make.
            let _ = writeln!(
                out,
                "            let {name} = optional_object(args, \"{name}\")?\n                \
                 .map(|within| read_{base}(&within))\n                .transpose()?;"
            );
            call_args.push(format!(
                "{name}.as_ref().map_or(core::ptr::null(), core::ptr::from_ref)"
            ));
        }
        "struct_in" => {
            let _ = writeln!(
                out,
                "            let {name} = read_{base}(&object(args, \"{name}\")?)?;"
            );
            call_args.push(format!("&raw const {name}"));
        }
        "struct_out" => {
            // `default()` and not zeroed: it is the binding's own way of
            // filling the extent the engine reads before it writes.
            let _ = writeln!(
                out,
                "            let mut {name}: sys::{base} = sys::{base}::default();"
            );
            call_args.push(format!("&raw mut {name}"));
            outs.push(format!("\"{name}\": write_{base}(&{name})"));
        }
        // The one struct an extent measures. Asserted rather than
        // guessed: a function with two structs and one extent would have
        // to say which, and none does.
        _ => {
            let [measured] = structs else {
                panic!(
                    "`{name}` passes a struct's extent beside {} structs; the generator knows \
                     which struct it measures only when there is one",
                    structs.len()
                );
            };
            call_args.push(format!("core::mem::size_of_val(&{})", measured.name));
        }
    }
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
    if has_role(function, "struct_in") {
        clauses.push("every struct argument is a local of the type the engine declares, its extent filled by the arm");
    }
    if has_role(function, "struct_out") {
        clauses.push("every struct out-parameter is a local of the type the engine declares, its extent filled by `default`");
    }
    if has_role(function, "out_struct_size") {
        clauses.push("the extent passed is the size of that local");
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
        let line = lines
            .last_mut()
            .unwrap_or_else(|| unreachable!("never empty"));
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
    let strings = count_where(rows, carries_a_string);
    let structs = count_where(rows, carries_a_struct);
    let both = count_where(rows, |function| {
        carries_a_string(function) && carries_a_struct(function)
    });
    let scalar_only = rows.len() - strings - structs + both;
    let _ = writeln!(
        out,
        "**{} functions**. {} of them take and answer scalars and enums alone, which is the shape a JSON object carries without a marshaller having to know anything else.\n",
        spelled(rows.len()),
        spelled(scalar_only),
    );
    let _ = writeln!(
        out,
        "{} carry a string: {} read one the caller passes, {} fill a buffer of the marshaller's, and {} answer with one the engine lends and the marshaller copies before anything else can move it.\n",
        spelled(strings),
        spelled(takes),
        spelled(fills),
        spelled(lends),
    );
    let _ = writeln!(
        out,
        "{} carry a struct, which crosses as a JSON object keyed by the engine's own field names, nested as the struct nests; {} of them carry a string as well. Every field is required going in, and the struct's own `struct_size` crosses in neither direction — the arm fills it, because the engine reads the struct only as far as it says (`03-design/engine-passthrough.md`).\n",
        spelled(structs),
        spelled(both),
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
        let _ = writeln!(
            out,
            "### The {} that change engine state\n",
            spelled(mutating)
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

fn queue_section(rows: Option<&Vec<Row<'_>>>, vocabulary: &Vocabulary) -> String {
    let mut out = String::new();
    let Some(rows) = rows else { return out };
    let _ = writeln!(out, "## What the marshaller has not learned\n");
    let mut by_reason: BTreeMap<(usize, &str), Vec<&str>> = BTreeMap::new();
    for row in rows {
        // In rank order, easiest first, so the table reads as the order
        // of work; a reason that is not a blocker comes after them all.
        let rank = Blocker::ALL
            .iter()
            .position(|blocker| blocker.wording() == row.why)
            .unwrap_or(Blocker::ALL.len());
        by_reason
            .entry((rank, row.why))
            .or_default()
            .push(row.name());
    }
    let _ = writeln!(
        out,
        "**{} functions**, grouped by the hardest thing in the way and listed easiest first. This is a queue rather than a refusal: each group is one shape the generator has to learn, and learning one brings its whole group in at once.\n",
        spelled(rows.len())
    );
    let _ = writeln!(out, "| what it takes or returns | functions | examples |");
    let _ = writeln!(out, "|---|---:|---|");
    for ((_, why), names) in &by_reason {
        let examples: Vec<String> = names.iter().take(3).map(|n| format!("`{n}`")).collect();
        let _ = writeln!(out, "| {why} | {} | {} |", names.len(), examples.join(", "));
    }
    let _ = writeln!(out);
    let struct_rows = count_where(rows, |function| {
        function.params.iter().any(|param| {
            matches!(
                blocked_by(param, vocabulary),
                Some(
                    Blocker::StructArray
                        | Blocker::StringStruct
                        | Blocker::PointerStruct
                        | Blocker::OtherStruct
                )
            )
        })
    });
    let _ = writeln!(
        out,
        "**{struct_rows} of the {} still touch a struct**, and they are no longer one group. This table used to have a row reading *a struct* over eighty-one functions, because the classifier knew a struct only by the word. Reading each struct's field list split that row by the worst field in the way — and the largest of the pieces, the structs made of numbers alone, was also the easiest, and is callable above.\n",
        rows.len()
    );
    let _ = writeln!(
        out,
        "The order of work that follows is in `03-design/engine-passthrough.md` §5, with what each step releases; the figures there were measured by the same classification as this table.\n"
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
