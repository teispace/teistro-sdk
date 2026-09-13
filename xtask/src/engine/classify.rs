//! Where each function stands, and what each parameter asks of the
//! marshaller: the classification every generated file is read from.

use super::idl::{Function, Made, Param, TypeRef, Vocabulary};

/// What the SDK owns and a caller must not take from it.
///
/// These open, close or rebind the very things the adapter is holding: a
/// consumer who called one through the passthrough would close the
/// context every other call depends on, or swap the data underneath it.
/// Excluded whatever their shape, and named here rather than discovered
/// by the marshaller failing to handle them, because the reason is a
/// boundary and not a difficulty.
pub(crate) const OWNED_BY_THE_ADAPTER: [&str; 4] = [
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
pub(crate) const OWNED_PREFIX: &str = "tm_data_source_";

/// A function that changes engine state the SDK's own provenance does not
/// observe: after one of these a chart's recorded settings and the engine
/// answering it have quietly parted company.
///
/// **Offered rather than refused**, because reaching what the SDK has not
/// ported is the whole point of the namespace, and marked so that a
/// consumer chooses it knowingly.
pub(crate) fn mutates_engine_state(name: &str) -> bool {
    name.starts_with("tm_set_") || name.starts_with("tm_clear_") || ALSO_MUTATES.contains(&name)
}

/// Functions that change engine state without a name that says so.
///
/// The prefix rule above catches the engine's own `set`/`clear` families
/// and would silently miss these, which is worse than missing them
/// loudly: the column on the page exists so a consumer can see what a
/// call will do to every chart cast after it.
pub(crate) const ALSO_MUTATES: [&str; 1] = [
    // Loads a star catalogue into the context, so `tm_star_calc` after
    // it resolves names it did not resolve before.
    "tm_star_load_catalogue",
];

/// Where a function stands with respect to the passthrough.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Standing {
    /// Callable at `sdk.engine.*` today.
    Callable,
    /// The adapter owns what it touches; never offered.
    Owned,
    /// The marshaller has not learned its shape yet.
    Unlearned,
}

pub(crate) fn plain(type_ref: &TypeRef, vocabulary: &Vocabulary) -> bool {
    type_ref.pointer == 0 && vocabulary.is_number(&type_ref.base)
}

pub(crate) fn scalar_out(type_ref: &TypeRef, vocabulary: &Vocabulary) -> bool {
    type_ref.pointer == 1 && vocabulary.is_number(&type_ref.base)
}

/// The parameter roles the marshaller carries.
///
/// One list, read by the classifier and by nothing else: a role added
/// here without an arm in [`arm`] would generate code that does not
/// compile, which is the failure mode to want.
pub(crate) const ROLES_KNOWN: [&str; 9] = [
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
pub(crate) fn bookkeeping(role: &str) -> bool {
    matches!(role, "handle" | "string_cap" | "out_struct_size")
}

/// Where one function stands, and why.
pub(crate) fn standing(function: &Function, vocabulary: &Vocabulary) -> (Standing, &'static str) {
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
pub(crate) enum Blocker {
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
    pub(crate) const ALL: [Self; 8] = [
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
    pub(crate) fn wording(self) -> &'static str {
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
pub(crate) fn blocked_by(param: &Param, vocabulary: &Vocabulary) -> Option<Blocker> {
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
pub(crate) fn declared_type(param: &Param) -> &str {
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
pub(crate) fn declared_return(function: &Function) -> &str {
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
pub(crate) fn returns_a_string(function: &Function) -> bool {
    function.returns.pointer == 1 && function.returns.base == "char"
}

/// Whether the function fills a buffer of the caller's with a string.
pub(crate) fn fills_a_string(function: &Function) -> bool {
    has_role(function, "string_out")
}

/// Whether any parameter plays the given role.
pub(crate) fn has_role(function: &Function, role: &str) -> bool {
    function.params.iter().any(|p| p.role == role)
}

/// What a callable function carries beyond numbers, for the table.
///
/// Read from the description rather than from the arm, so the column
/// cannot describe code that is no longer generated.
pub(crate) fn carries(function: &Function) -> String {
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
pub(crate) fn carries_a_string(function: &Function) -> bool {
    has_role(function, "string_in") || fills_a_string(function) || returns_a_string(function)
}

/// Whether a function carries a struct either way.
pub(crate) fn carries_a_struct(function: &Function) -> bool {
    has_role(function, "struct_in") || has_role(function, "struct_out")
}
