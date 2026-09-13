//! The engine's own description, as the classifier and the generators
//! read it: the model, and the vocabulary that names its types.

use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct TypeRef {
    pub(crate) base: String,
    #[serde(default)]
    pub(crate) pointer: u32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Param {
    pub(crate) name: String,
    #[serde(rename = "type")]
    pub(crate) type_ref: TypeRef,
    #[serde(default)]
    pub(crate) role: String,
    /// Whether the engine takes a null here. Its extractor marks every
    /// single struct input so, because whether null is allowed is in the
    /// header's prose and not in the type — and the engine refuses a null
    /// it does not allow with its own status, so passing one is safe.
    #[serde(default)]
    pub(crate) optional: bool,
    /// The parameter a length or a capacity belongs to.
    #[serde(default)]
    pub(crate) of: Option<String>,
    /// How long an output array is, as the engine records it.
    #[serde(default)]
    pub(crate) extent: Option<Extent>,
}

/// How long an output array is — which its C type cannot say, and which
/// the engine's extractor lists for every one.
///
/// `double *out, size_t out_capacity` is one declaration for four
/// contracts, and a marshaller has to know which before it can size the
/// buffer or tell how much of it is answer.
#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub(crate) enum Extent {
    /// As long as `of`: an input array's length, or a struct input's
    /// field spelled `req.day_count`.
    Length { of: String },
    /// As long as every name in `of` multiplied, in layout order.
    Product { of: Vec<String> },
    /// The capacity is the caller's question — "the next N eclipses" —
    /// and `count` receives how many were written.
    Asked { count: String },
    /// `count` receives, or as `return` the function returns, how many
    /// there are, even beyond the capacity.
    Total { count: String },
    /// Decided by something no parameter holds, and `why` says what.
    Unstated { why: String },
}

impl Param {
    /// Whether a caller may leave this out, which crosses as null.
    pub(crate) fn nullable(&self) -> bool {
        self.optional && self.role == "struct_in"
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Function {
    pub(crate) name: String,
    pub(crate) returns: TypeRef,
    pub(crate) params: Vec<Param>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Named {
    pub(crate) name: String,
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
    pub(crate) fn sized(&self) -> bool {
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
pub(crate) const STRUCT_SIZE: &str = "struct_size";

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
pub(crate) struct Alias {
    pub(crate) name: String,
    pub(crate) base: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Idl {
    pub(crate) version: String,
    pub(crate) functions: Vec<Function>,
    #[serde(default)]
    pub(crate) aliases: Vec<Alias>,
    #[serde(default)]
    pub(crate) structs: Vec<Shape>,
    #[serde(default)]
    pub(crate) enums: Vec<Named>,
    #[serde(default)]
    pub(crate) callbacks: Vec<Named>,
}

/// The scalar bases a value crosses JSON as without ceremony.
pub(crate) const SCALARS: [&str; 11] = [
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
    pub(crate) enums: Vec<String>,
    pub(crate) aliases: BTreeMap<String, String>,
    pub(crate) structs: BTreeMap<String, Shape>,
    /// The structs' names in declaration order, which a map forgets.
    pub(crate) order: Vec<String>,
}

impl Vocabulary {
    pub(crate) fn new(idl: &Idl) -> Self {
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

    /// Whether reading a struct needs somewhere to keep what it points at:
    /// a string or a pointer in it, or in a struct it holds by value.
    pub(crate) fn needs_keep(&self, base: &str) -> bool {
        self.structs.contains_key(base) && self.made_of(base) != Made::Plain
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
