//! Where each function stands, and what each parameter asks of the
//! marshaller: the classification every generated file is read from.

use core::fmt;

use super::idl::{Extent, Field, Function, Made, Param, TypeRef, Vocabulary};

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
pub(crate) const ROLES_KNOWN: [&str; 14] = [
    "handle",
    "value",
    "scalar_out",
    "string_in",
    "string_out",
    "string_cap",
    "struct_in",
    "struct_out",
    "out_struct_size",
    "array_in",
    "array_len",
    "array_out",
    "array_cap",
    "array_out_parallel",
];

/// Which side of a call a parameter is on, as a consumer sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Crossing {
    /// The marshaller supplies it and never reports it: the context, a
    /// buffer's capacity, a struct's extent, an array's length, and the
    /// count of an array whose own length already says it.
    Bookkeeping,
    /// The caller supplies it.
    Takes,
    /// The answer reports it.
    Gives,
}

/// Which side of a call one parameter is on.
///
/// Per function and not per role, because two roles change sides with
/// the output beside them: an `asked` output's capacity is the caller's
/// question — how many eclipses to find — and an output count is
/// bookkeeping exactly when it counts an array the answer already holds.
pub(crate) fn crossing(function: &Function, param: &Param) -> Crossing {
    match param.role.as_str() {
        "handle" | "string_cap" | "out_struct_size" | "array_len" => Crossing::Bookkeeping,
        "array_cap" => {
            let asked = output_of(function, param)
                .and_then(|output| sizing(function, output))
                .is_some_and(|sized| matches!(sized, Sizing::Asked { .. }));
            if asked {
                Crossing::Takes
            } else {
                Crossing::Bookkeeping
            }
        }
        "scalar_out" if counts_an_output(function, &param.name) => Crossing::Bookkeeping,
        "scalar_out" | "string_out" | "struct_out" | "array_out" | "array_out_parallel" => {
            Crossing::Gives
        }
        _ => Crossing::Takes,
    }
}

/// The output array a capacity belongs to.
fn output_of<'a>(function: &'a Function, capacity: &Param) -> Option<&'a Param> {
    let of = capacity.of.as_deref()?;
    function
        .params
        .iter()
        .find(|param| param.role == "array_out" && param.name == of)
}

/// Whether a scalar out-parameter is the count of an output array.
fn counts_an_output(function: &Function, name: &str) -> bool {
    function.params.iter().any(|param| {
        matches!(
            &param.extent,
            Some(Extent::Asked { count } | Extent::Total { count }) if count == name
        )
    })
}

/// Whether the function's return value is the count of an output array.
pub(crate) fn returns_the_count(function: &Function) -> bool {
    counts_an_output(function, RETURN)
}

/// The extent's spelling of "the function's own return value".
const RETURN: &str = "return";

/// How the marshaller sizes one output array.
#[derive(Clone, Debug)]
pub(crate) enum Sizing<'a> {
    /// As long as these measures multiplied, in layout order.
    Inputs(Vec<Measure<'a>>),
    /// As long as `function` answers when called with `args` — once, or
    /// once per element of the array an `Each` argument names, taking the
    /// largest answer and multiplying it by that array's length.
    Called {
        function: &'a str,
        args: Vec<Measure<'a>>,
        /// The input array the answer is multiplied by the length of.
        times: Option<&'a str>,
    },
    /// The caller says how many to find, under `capacity`, and `count`
    /// receives how many were.
    Asked { capacity: &'a str, count: &'a str },
    /// The engine says how many there are; the marshaller asks again
    /// when there are more than it made room for.
    Total { count: Counted<'a> },
}

/// One number an output's length is computed from.
#[derive(Clone, Debug)]
pub(crate) enum Measure<'a> {
    /// The length of the input array of this name.
    Length(&'a str),
    /// A value parameter of this name.
    Value(&'a str),
    /// A field of a struct input: the parameter, and the field's name.
    Field(&'a Param, &'a str),
    /// A field of every element of an input array of structs.
    Each(&'a Param, &'a str),
}

/// Where a `total` output's count comes back.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Counted<'a> {
    /// In a scalar out-parameter of that name.
    Param(&'a str),
    /// As the function's own return value.
    Return,
}

/// How to size an output array, or `None` when the marshaller cannot.
///
/// `None` for an extent the engine leaves unstated, and for a length it
/// states in terms this marshaller cannot evaluate without calling
/// something else first — a struct input's field, or a value. Both are
/// queued rather than guessed: an output sized wrongly is a truncated
/// answer at best and a refusal the caller cannot fix at worst.
pub(crate) fn sizing<'a>(function: &'a Function, output: &'a Param) -> Option<Sizing<'a>> {
    let capacity = || {
        function
            .params
            .iter()
            .find(|param| param.role == "array_cap" && param.of.as_deref() == Some(&output.name))
            .map(|param| param.name.as_str())
    };
    match output.extent.as_ref()? {
        Extent::Length { of } => {
            measures(function, core::slice::from_ref(of), false).map(Sizing::Inputs)
        }
        Extent::Product { of } => measures(function, of, false).map(Sizing::Inputs),
        Extent::Call {
            function: called,
            of,
            reduce,
            times,
        } => {
            let args = measures(function, of, true)?;
            let spread = args.iter().any(|arg| matches!(arg, Measure::Each(..)));
            // Only the one combination the engine states: the largest
            // answer, times the length of the array it was taken over. An
            // extent asking for anything else is queued, not guessed.
            let times = match (spread, reduce.as_deref(), times) {
                (false, None, None) => None,
                (true, Some("max"), Some(times)) => {
                    let array = function
                        .params
                        .iter()
                        .find(|param| param.role == "array_len" && &param.name == times)?
                        .of
                        .as_deref()?;
                    let over = args.iter().all(|arg| match arg {
                        Measure::Each(param, _) => param.name == array,
                        Measure::Value(_) => true,
                        _ => false,
                    });
                    over.then_some(array)?.into()
                }
                _ => return None,
            };
            Some(Sizing::Called {
                function: called,
                args,
                times,
            })
        }
        Extent::Asked { count } => Some(Sizing::Asked {
            capacity: capacity()?,
            count,
        }),
        Extent::Total { count } => Some(Sizing::Total {
            count: if count == RETURN {
                Counted::Return
            } else {
                Counted::Param(count)
            },
        }),
        Extent::Unstated { .. } => None,
    }
}

/// What each name in an extent measures, or `None` when any of them is
/// something the marshaller cannot read before the call.
///
/// A length names an input array's length parameter or a field of a
/// struct input (`req.day_count`); a call's argument names a value or a
/// field. At most one struct input may be named, because a nullable one
/// makes the whole measure zero when it is absent, and two would need a
/// rule for one present and one not that no extent needs.
fn measures<'a>(
    function: &'a Function,
    names: &'a [String],
    values: bool,
) -> Option<Vec<Measure<'a>>> {
    let found = names
        .iter()
        .map(|name| {
            let (head, field) = match name.split_once('.') {
                Some((head, field)) => (head, Some(field)),
                None => (name.as_str(), None),
            };
            let (head, each) = match head.strip_suffix("[]") {
                Some(array) => (array, true),
                None => (head, false),
            };
            let param = function.params.iter().find(|param| param.name == head)?;
            match (field, param.role.as_str()) {
                (Some(field), "array_in") if each && values => Some(Measure::Each(param, field)),
                (Some(field), "struct_in") if !each => Some(Measure::Field(param, field)),
                (None, "array_len") if !values => param.of.as_deref().map(Measure::Length),
                (None, "value") if values => Some(Measure::Value(&param.name)),
                _ => None,
            }
        })
        .collect::<Option<Vec<_>>>()?;
    let mut roots = found.iter().filter_map(|measure| match measure {
        Measure::Field(param, _) | Measure::Each(param, _) => Some(param.name.as_str()),
        _ => None,
    });
    let first = roots.next();
    roots.all(|root| Some(root) == first).then_some(found)
}

/// The outputs an engine sizes by being asked twice, which the arm can
/// run at most once per call.
pub(crate) fn totals(function: &Function) -> impl Iterator<Item = (&Param, Counted<'_>)> {
    function
        .params
        .iter()
        .filter(|param| param.role == "array_out")
        .filter_map(|param| match sizing(function, param) {
            Some(Sizing::Total { count }) => Some((param, count)),
            _ => None,
        })
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
        .filter_map(|param| blocked_by(function, param, vocabulary))
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
    // Asking again is a protocol that reruns the whole call, so a call can
    // run it for one thing: a filled string, or one output of unknown
    // length. None asks for two; one that did would be queued here rather
    // than generated with one of them wrong.
    if totals(function).count() + usize::from(fills_a_string(function)) > 1 {
        return (
            Standing::Unlearned,
            "two outputs that each need the call run again",
        );
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
    /// An output array whose length the description does not state in
    /// terms of the call's own inputs.
    UnsizedOutput,
    /// An output written beside another at the same length, and optional.
    ParallelOutput,
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
    pub(crate) const ALL: [Self; 6] = [
        Self::UnsizedOutput,
        Self::ParallelOutput,
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
            Self::UnsizedOutput => "an output whose length it cannot compute",
            Self::ParallelOutput => "an optional output parallel to another",
            Self::HandleOut => "a handle it creates",
            Self::Opaque => "opaque bytes",
            Self::OtherStruct => "a struct no JSON object describes",
            Self::Unknown => "a role the marshaller does not know",
        }
    }

    /// What stands in the way of a struct, if anything does: only a field
    /// no JSON object describes. Numbers, strings and pointers to other
    /// structs all cross.
    const fn of_struct(made: Made) -> Option<Self> {
        match made {
            Made::Plain | Made::Strings | Made::Pointers => None,
            Made::Other => Some(Self::OtherStruct),
        }
    }

    /// What stands in the way of an array's element, if anything does:
    /// a number crosses, a struct crosses as structs do, and anything
    /// else — a byte of an encoded blob — is bytes.
    fn of_element(base: &str, vocabulary: &Vocabulary) -> Option<Self> {
        if vocabulary.is_number(base) {
            None
        } else if vocabulary.shape(base).is_some() {
            Self::of_struct(vocabulary.made_of(base))
        } else {
            Some(Self::Opaque)
        }
    }
}

/// What stands in the way of one parameter, if anything does.
pub(crate) fn blocked_by(
    function: &Function,
    param: &Param,
    vocabulary: &Vocabulary,
) -> Option<Blocker> {
    let base = param.type_ref.base.as_str();
    match param.role.as_str() {
        // A struct crosses when every field does, so what blocks it is
        // its worst field: that is the shape the generator would learn
        // next. "A struct" alone was one row over eighty-one functions
        // and four different jobs.
        "struct_in" | "struct_out" => Blocker::of_struct(vocabulary.made_of(base)),
        // An array is its elements, and an output array is also its
        // length: one the marshaller cannot size cannot be allocated.
        "array_in" => Blocker::of_element(base, vocabulary),
        "array_out" => Blocker::of_element(base, vocabulary).or_else(|| {
            sizing(function, param)
                .is_none()
                .then_some(Blocker::UnsizedOutput)
        }),
        // A twin written at its output's length: learned exactly when that
        // output is sized before the call, which a gathered one is not.
        "array_out_parallel" => Blocker::of_element(base, vocabulary).or_else(|| {
            let twin = param
                .of
                .as_deref()
                .and_then(|of| function.params.iter().find(|one| one.name == of));
            match twin.and_then(|twin| sizing(function, twin)) {
                Some(Sizing::Total { .. }) | None => Some(Blocker::ParallelOutput),
                Some(_) => None,
            }
        }),
        "handle_out" => Some(Blocker::HandleOut),
        "opaque" => Some(Blocker::Opaque),
        known if ROLES_KNOWN.contains(&known) => None,
        _ => Some(Blocker::Unknown),
    }
}

/// A type as the manifest and the façades name it: the engine's own
/// word, and whether a list of them crosses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Declared<'a> {
    /// The engine's word — `double`, `tm_datetime` — or `string`.
    pub(crate) base: &'a str,
    /// Whether the value is an array of `base`.
    pub(crate) list: bool,
    /// Whether the value may be null: a struct field that is a pointer.
    pub(crate) nullable: bool,
}

impl<'a> Declared<'a> {
    /// One value of a type.
    pub(crate) const fn one(base: &'a str) -> Self {
        Self {
            base,
            list: false,
            nullable: false,
        }
    }

    /// A struct field's type: a `const char *` is a string and a pointer
    /// to a struct is that struct, both of which may be null; anything
    /// else is itself.
    pub(crate) fn field(field: &'a Field) -> Self {
        match (field.pointer, field.base.as_str()) {
            (1, "char") => Self {
                nullable: true,
                ..Self::one("string")
            },
            (1, base) => Self {
                nullable: true,
                ..Self::one(base)
            },
            _ => Self::one(&field.base),
        }
    }
}

impl fmt::Display for Declared<'_> {
    /// `double`, or `double[]` for a list, which is what the manifest
    /// says a parameter's type is.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = if self.list { "[]" } else { "" };
        write!(f, "{}{suffix}", self.base)
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
    /// What the caller supplies: the key, and its type.
    pub(crate) takes: Vec<(&'a str, Declared<'a>)>,
    /// What comes back, keyed as the answer keys it. `return` is in here
    /// when the function's own return value is reported.
    pub(crate) gives: Vec<(&'a str, Declared<'a>)>,
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
pub(crate) fn declared_type(param: &Param) -> Declared<'_> {
    match param.role.as_str() {
        "string_in" | "string_out" => Declared::one("string"),
        "array_in" | "array_out" | "array_out_parallel" => Declared {
            list: true,
            ..Declared::one(&param.type_ref.base)
        },
        _ => Declared::one(&param.type_ref.base),
    }
}

/// The manifest's word for what a function answers with.
///
/// A word about the answer and not about C: `status` is already not a
/// type name, and a function whose return a protocol consumes — the fill
/// protocol's length, an output's count — puts nothing under `return`.
pub(crate) fn declared_return(function: &Function) -> &str {
    if function.returns.base == "tm_status" {
        "status"
    } else if function.returns.base == "void"
        || fills_a_string(function)
        || returns_the_count(function)
    {
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
        let entry = (param.name.as_str(), declared_type(param));
        match crossing(function, param) {
            Crossing::Bookkeeping => {}
            Crossing::Takes => {
                if param.nullable() {
                    nullable.push(param.name.as_str());
                }
                takes.push(entry);
            }
            Crossing::Gives => gives.push(entry),
        }
    }
    let returns = declared_return(function);
    if !matches!(returns, "status" | "void") {
        gives.push(("return", Declared::one(returns)));
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
        (has_role(function, "array_in"), "an array it reads"),
        (
            has_role(function, "array_out") || has_role(function, "array_out_parallel"),
            "an array it fills",
        ),
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

/// Whether a function carries an array either way.
pub(crate) fn carries_an_array(function: &Function) -> bool {
    has_role(function, "array_in") || has_role(function, "array_out")
}
