//! The Python binding's mechanical layer, rendered from the description:
//! the `ctypes` declarations that match the C header name for name, the
//! catalogue as `IntEnum`s, a frozen dataclass per boundary struct with
//! its own marshalling, the context as a handle with a finaliser, and the
//! result-blob decoders as `memoryview` columns.
//!
//! The same roles drive it as drive the C header, the Node addon and the
//! Dart layer, so the four cannot disagree: a flag is a `bool`, a bit set
//! a `frozenset`, an array of an enum's ids the members themselves, an
//! optional value `Optional[…]`, and the counts and presence flags a C
//! struct carries never reach the surface.
//!
//! Two things are Python's own, and both come out of the falsification
//! pass in `03-design/binding-surface-measured.md`:
//!
//! 1. **`from` is a keyword here and nowhere else.** It is a struct field
//!    and a parameter of two entry points, so the emitter applies PEP 8's
//!    rule and writes `from_`. The Dart emitter's own rule never fires on
//!    any of the three, and Python's never fires on the member Dart has
//!    to rename, because a member here is its catalogue key upper-cased
//!    and every Python keyword is lower-case.
//! 2. **Nothing checks a `ctypes` layout.** The C header asserts all 25
//!    struct sizes at compile time; a `ctypes` declaration is trusted. So
//!    the emitter writes the sizes it computed, for both targets, and the
//!    binding's tests assert `ctypes.sizeof` against them on the machine
//!    the library was really built on.
//!
//! Three files, so a package's parts are found where a Python reader
//! looks: `catalogue.py` (the enums), `_ffi.py` (the declarations, the
//! value classes and the context) and `_blob.py` (the decoders).

use std::collections::BTreeSet;
use std::fmt::Write;

use crate::emit::{DocStyle, field_doc_with, line_comment, reserved};
use crate::layout::{Target, struct_layout};
use crate::model::{
    Api, BlobSchema, EnumDef, EnumValue, FieldDef, FunctionDef, OpaqueDef, ParamDef, Role, Scalar,
    SectionKind, SectionSchema, StructDef, StructRole, TypeRef,
};
use crate::names::{binding_type_name, c_type_name, kebab, method_name, pascal, screaming, snake};
use crate::rules::{
    FieldRole, Handed, constant_key, constants, constructor, destructor, field_roles,
    has_handshake, is_free_function, methods, pointee_struct, results, returned_scalar,
    returns_status,
};

/// A Python identifier for a field, a parameter or a method: snake case,
/// with PEP 8's trailing underscore where the name is a keyword.
fn identifier(name: &str) -> String {
    reserved::renamed(&snake(name), reserved::PYTHON, "_")
}

/// A Python name for an enum member: its catalogue key upper-cased where
/// it has one, its variant in screaming snake case otherwise. Every
/// Python keyword is lower-case, so this spelling cannot collide today;
/// the rule is applied anyway, because a member added later might be
/// spelled another way and a silent collision is a file that will not
/// parse.
fn member(value: &EnumValue) -> String {
    let spelled = value.key.clone().unwrap_or_else(|| screaming(&value.name));
    reserved::renamed(&spelled, reserved::PYTHON, "_")
}

/// The member a value from a newer library falls into.
const UNKNOWN: &str = "UNKNOWN";

/// A docstring of a boundary item's own documentation, with rustdoc's
/// link syntax removed.
fn docstring(text: &str, indent: &str) -> String {
    written(&crate::names::clean_doc(text), indent)
}

/// A field's docstring: its own text cleaned, and the unit, range,
/// example and enum the `api:` line carries appended **after** the
/// cleaning, because a range is written `[-90,90]` and the rustdoc
/// cleaner would take it for a link and eat the brackets.
fn field_docstring(field: &FieldDef, indent: &str) -> String {
    let mut cleaned = field.clone();
    cleaned.doc = crate::names::clean_doc(&field.doc);
    written(
        &field_doc_with(&cleaned, DocStyle::Prose, field.meta.enum_name.as_deref()),
        indent,
    )
}

/// A docstring of text that is already as it should read. Empty text
/// renders nothing.
fn written(text: &str, indent: &str) -> String {
    let text = crate::emit::without_rust_examples(text);
    if text.trim().is_empty() {
        return String::new();
    }
    // A backslash would escape the character after it and a triple quote
    // would close the string; neither is meaningful in the boundary's own
    // documentation, so both are defused rather than carried through.
    let text = text.replace('\\', "\\\\").replace("\"\"\"", "'''");
    let mut lines = text.lines();
    let Some(first) = lines.next() else {
        return String::new();
    };
    let rest: Vec<&str> = lines.collect();
    // One line closes on itself, unless it ends in the quote that would
    // then run into the three that close the string.
    if rest.is_empty() && !first.ends_with('"') {
        return format!("{indent}\"\"\"{first}\"\"\"\n");
    }
    let mut out = format!("{indent}\"\"\"{first}\n");
    for line in rest {
        if line.is_empty() {
            out.push('\n');
        } else {
            let _ = writeln!(out, "{indent}{line}");
        }
    }
    let _ = writeln!(out, "{indent}\"\"\"");
    out
}

/// A comment where every line carries `# `.
fn comment(text: &str, indent: &str) -> String {
    line_comment(&crate::names::clean_doc(text), indent, "# ")
}

fn preamble(api: &Api, what: &str) -> String {
    format!(
        "# Generated by `cargo xtask gen ffi` from the Teistro SDK's boundary\n# crates; do not edit. ABI version {}, SDK {}.\n#\n{}#\n# The description this file was rendered from ships as idl/api.json, and\n# the generator lays this file out, so `cargo xtask check-ffi` can\n# regenerate it on a machine with no Python and still compare it byte for\n# byte.\n\nfrom __future__ import annotations\n\n",
        api.abi_version,
        api.sdk_version,
        comment(what, "")
    )
}

// ── The catalogue ──────────────────────────────────────────────────────────

/// `catalogue.py`: every enum as an `IntEnum` carrying the id the C
/// boundary uses and the key every pack, fixture and serialised result
/// spells.
#[must_use]
pub fn catalogue(api: &Api) -> String {
    let mut out = preamble(
        api,
        "The catalogue: every enum of the boundary, as an `IntEnum` whose value is\nthe id the C ABI carries and whose `key` is the word every pack, fixture\nand serialised result uses.",
    );
    out.push_str("import enum\nfrom typing import Optional\n\n\n");
    out.push_str(CATALOGUE_BASE);
    for e in &api.enums {
        render_enum(&mut out, e);
    }
    render_key_tables(&mut out, api);
    out
}

/// The two bases every generated enum uses. A catalogued kind's members
/// come from a registry that may grow, so an id this build does not know
/// is `UNKNOWN` rather than an exception; a closed enum has no such
/// member, and an id outside it raises, because a value outside a closed
/// set is a fault and not a state.
const CATALOGUE_BASE: &str = r#"class Member(enum.IntEnum):
    """An enum of the boundary: an id the C ABI carries, and a key.

    The value is the id, so a member may be passed wherever the boundary
    wants a number. `key` is the word every pack, fixture and serialised
    result spells the member with.
    """

    def __bool__(self) -> bool:
        """A member is always a value, whatever its id.

        `IntEnum` inherits `int.__bool__`, which makes the member with id
        zero falsy — and the member with id zero is `Status.OK`,
        `Graha.SUN`, `Era.VIKRAMA` and the first member of every other
        enum. `if graha:` has to mean "there is a graha", not "the graha
        is not the Sun".
        """
        return True

    @property
    def id(self) -> int:
        """The id the C boundary carries, which is the member's value."""
        return int(self)

    @property
    def key(self) -> str:
        """The key every pack, fixture and serialised result spells."""
        return _KEYS[type(self).__name__][int(self)]

    @classmethod
    def by_key(cls, key: str) -> Optional["Member"]:
        """The member with a key, or `None` for one this build lacks.

        A full key (`graha.SUN`) and a bare one (`SUN`) both find it.
        """
        wanted = key.rsplit(".", 1)[-1]
        for found in cls:
            if found.key == wanted:
                return found
        return None


class Catalogued(Member):
    """A member of a catalogue kind, whose members a registry may add to.

    An id this build does not know is `UNKNOWN`, so a caller matching on
    the result stays exhaustive against a newer library.
    """

    @classmethod
    def _missing_(cls, value: object) -> "Catalogued":
        return cls(-1)

    @property
    def full_key(self) -> str:
        """The key with its kind, as every pack and fixture spells it."""
        return f"{_KINDS[type(self).__name__]}.{self.key}"


"#;

fn render_enum(out: &mut String, e: &EnumDef) {
    let name = binding_type_name(&e.name);
    let catalogued = e.kind.is_some();
    let base = if catalogued { "Catalogued" } else { "Member" };
    let _ = writeln!(out, "class {name}({base}):");
    out.push_str(&docstring(&e.doc, "    "));
    for value in &e.values {
        let doc = if value.deprecated {
            format!("{}\n\nDeprecated.", value.doc)
        } else {
            value.doc.clone()
        };
        let _ = writeln!(out, "\n    {} = {}", member(value), value.value);
        out.push_str(&docstring(&doc, "    "));
    }
    if catalogued {
        let _ = writeln!(out, "\n    {UNKNOWN} = -1");
        out.push_str(&docstring(
            "A member this build does not know: from a newer library, or\nregistered at run time.",
            "    ",
        ));
    }
    out.push_str("\n\n");
}

/// The key tables the bases read, written once at the end of the module,
/// so a member's key is data rather than a method generated per enum.
fn render_key_tables(out: &mut String, api: &Api) {
    out.push_str("# The key of every member, by enum and id, and the kind of every\n# catalogued enum. The two bases above read them; nothing else should.\n_KEYS: dict[str, dict[int, str]] = {\n");
    for e in &api.enums {
        let _ = writeln!(out, "    \"{}\": {{", binding_type_name(&e.name));
        for value in &e.values {
            let key = value.key.clone().unwrap_or_else(|| kebab(&value.name));
            let _ = writeln!(out, "        {}: \"{key}\",", value.value);
        }
        if e.kind.is_some() {
            let _ = writeln!(out, "        -1: \"{UNKNOWN}\",");
        }
        out.push_str("    },\n");
    }
    out.push_str("}\n\n_KINDS: dict[str, str] = {\n");
    for e in &api.enums {
        if let Some(kind) = &e.kind {
            let _ = writeln!(out, "    \"{}\": \"{kind}\",", binding_type_name(&e.name));
        }
    }
    out.push_str("}\n");
}

// ── Types ──────────────────────────────────────────────────────────────────

/// The `ctypes` type of a scalar. Every one is fixed-width: `c_long` is 8
/// bytes on Linux and 4 on Windows, and a generated binding exists to
/// make that class of mistake impossible.
const fn ctypes_scalar(scalar: Scalar) -> &'static str {
    match scalar {
        Scalar::U8 => "ctypes.c_uint8",
        Scalar::U16 => "ctypes.c_uint16",
        Scalar::U32 => "ctypes.c_uint32",
        Scalar::U64 => "ctypes.c_uint64",
        Scalar::I8 => "ctypes.c_int8",
        Scalar::I16 => "ctypes.c_int16",
        Scalar::I32 => "ctypes.c_int32",
        Scalar::I64 => "ctypes.c_int64",
        Scalar::F32 => "ctypes.c_float",
        Scalar::F64 => "ctypes.c_double",
        Scalar::Usize => "ctypes.c_size_t",
        Scalar::Isize => "ctypes.c_ssize_t",
        Scalar::Bool => "ctypes.c_bool",
    }
}

/// The `struct`/`memoryview` format code of a scalar, which a blob column
/// is cast to.
const fn format_code(scalar: Scalar) -> &'static str {
    match scalar {
        Scalar::U8 => "B",
        Scalar::U16 => "H",
        Scalar::U32 => "I",
        Scalar::U64 => "Q",
        Scalar::I8 => "b",
        Scalar::I16 => "h",
        Scalar::I32 => "i",
        Scalar::I64 => "q",
        Scalar::F32 => "f",
        Scalar::F64 => "d",
        Scalar::Usize => "n",
        Scalar::Isize => "N",
        Scalar::Bool => "?",
    }
}

/// What a scalar reads back as in Python.
const fn python_scalar(scalar: Scalar) -> &'static str {
    match scalar {
        Scalar::F32 | Scalar::F64 => "float",
        Scalar::Bool => "bool",
        _ => "int",
    }
}

/// The private `ctypes.Structure` of a boundary struct.
fn struct_name(rust_name: &str) -> String {
    format!("_{}Struct", binding_type_name(rust_name))
}

/// The `ctypes` spelling of a type as it appears in a declaration.
fn ctypes_type(api: &Api, ty: &TypeRef) -> String {
    match ty {
        TypeRef::Scalar { scalar } => ctypes_scalar(*scalar).to_string(),
        TypeRef::Enum { name } => api.enum_named(name).map_or_else(
            || String::from("ctypes.c_int32"),
            |e| ctypes_scalar(e.repr).to_string(),
        ),
        TypeRef::Struct { name } => struct_name(name),
        TypeRef::Opaque { name } => format!("_{}", binding_type_name(name)),
        TypeRef::Callback { name } => binding_type_name(name),
        TypeRef::Pointer { to, .. } => match &**to {
            TypeRef::Void => String::from("ctypes.c_void_p"),
            TypeRef::Char => String::from("ctypes.c_char_p"),
            inner => format!("ctypes.POINTER({})", ctypes_type(api, inner)),
        },
        TypeRef::Array { of, len } => format!("{} * {len}", ctypes_type(api, of)),
        TypeRef::Void => String::from("None"),
        TypeRef::Char => String::from("ctypes.c_char"),
    }
}

/// The struct the library hands a blob back in.
fn blob_struct(api: &Api) -> String {
    api.structs
        .iter()
        .find(|s| s.role == StructRole::Blob)
        .map_or_else(|| String::from("ctypes.c_void_p"), |s| struct_name(&s.name))
}

// ── The declarations ───────────────────────────────────────────────────────

/// `_ffi.py`: the declarations, the branded quantities, a value class per
/// boundary struct, the exception, the sizes, and the context.
#[must_use]
pub fn declarations(api: &Api) -> String {
    let mut out = preamble(
        api,
        "The `ctypes` layer: the declarations that match the C header name for\nname, a frozen value class per boundary struct with its own marshalling,\nthe exception every refusal is raised as, and the context handle.",
    );
    out.push_str("import ctypes\nimport weakref\nfrom dataclasses import dataclass\nfrom types import TracebackType\nfrom typing import Any, Final, NamedTuple, Optional, Sequence\n\n");
    render_catalogue_import(&mut out, api);
    render_constants(&mut out, api);
    render_sizes(&mut out, api);
    render_opaques(&mut out, api);
    // A callback points at a struct and a vtable holds callbacks, so the
    // three come out in the one order that resolves: the structs no
    // callback needs, then the callbacks, then the structs that hold one.
    render_structs(&mut out, api, false);
    render_callbacks(&mut out, api);
    render_structs(&mut out, api, true);
    render_exception(&mut out, api);
    out.push_str(HELPERS);
    render_brands(&mut out, api);
    for s in &api.structs {
        if shown(api, s) {
            render_value_class(&mut out, api, s);
        }
    }
    render_results(&mut out, api);
    render_library(&mut out, api);
    for opaque in &api.opaques {
        render_context(&mut out, api, opaque);
    }
    render_free_functions(&mut out, api);
    out
}

/// Every enum by name, so nothing is imported with a wildcard: a strict
/// type checker cannot see through one, and a reader cannot either.
fn render_catalogue_import(out: &mut String, api: &Api) {
    let mut names: Vec<String> = api
        .enums
        .iter()
        .map(|e| binding_type_name(&e.name))
        .collect();
    names.push(String::from("Member"));
    names.sort_unstable();
    names.dedup();
    out.push_str("from .catalogue import (\n");
    for name in names {
        let _ = writeln!(out, "    {name},");
    }
    out.push_str(")\n\n");
}

fn render_constants(out: &mut String, api: &Api) {
    for c in constants(api) {
        out.push_str(&comment(&c.doc, ""));
        let _ = writeln!(out, "{}: Final = {}\n", constant_key(c), c.value);
    }
    let _ = writeln!(
        out,
        "# The ABI and the SDK version these declarations were generated from. A\n# library that answers otherwise is refused when it is opened.\nGENERATED_ABI_VERSION: Final = {}\nGENERATED_SDK_VERSION: Final = \"{}\"\n",
        api.abi_version, api.sdk_version
    );
}

/// The struct sizes on both targets, and the one this interpreter is.
///
/// The falsification pass measured them: ten of the structs hold no
/// pointer and are the same size everywhere, and the rest are not. A
/// binding that shipped a single number would be right on one target and
/// silently wrong on the other, and `ctypes` has no compiler to catch it,
/// so the table is generated and the binding's tests assert
/// `ctypes.sizeof` against it on the machine the library was really built
/// on.
fn render_sizes(out: &mut String, api: &Api) {
    for (label, target) in [("64", Target::LP64), ("32", Target::ILP32)] {
        let _ = writeln!(out, "_SIZES_{label}: Final[dict[str, int]] = {{");
        for s in &api.structs {
            if let Ok(layout) = struct_layout(api, s, target) {
                let _ = writeln!(
                    out,
                    "    \"{}\": {},",
                    c_type_name(&api.prefix, &s.name),
                    layout.size
                );
            }
        }
        out.push_str("}\n\n");
    }
    out.push_str("# The size the C compiler gives every boundary struct on this target, as\n# the description computes it. `ctypes` lays a struct out by the same\n# rules, and the binding's tests hold the two to each other, because\n# nothing else can: the C header asserts these at compile time and a\n# `ctypes` declaration is simply trusted\n# (`03-design/binding-surface-measured.md` \u{a7}3).\nSIZES: Final[dict[str, int]] = (\n    _SIZES_64 if ctypes.sizeof(ctypes.c_void_p) == 8 else _SIZES_32\n)\n\n");
}

fn render_opaques(out: &mut String, api: &Api) {
    for o in &api.opaques {
        let _ = writeln!(
            out,
            "class _{}(ctypes.Structure):",
            binding_type_name(&o.name)
        );
        out.push_str(&docstring(
            &format!(
                "{}\n\nAn incomplete type: it is only ever a pointer, and giving it a class\nof its own keeps it apart from every other handle.",
                o.doc
            ),
            "    ",
        ));
        out.push_str("\n\n");
    }
}

/// Whether a struct holds a function pointer, which the callbacks have to
/// be declared before.
fn holds_callback(s: &StructDef) -> bool {
    s.fields
        .iter()
        .any(|f| matches!(f.ty, TypeRef::Callback { .. }))
}

fn render_structs(out: &mut String, api: &Api, with_callbacks: bool) {
    // A struct may hold another by value, and a `ctypes.Structure` reads
    // its fields when the class is defined, so the definitions come out in
    // dependency order.
    for s in ordered(api) {
        if holds_callback(s) != with_callbacks {
            continue;
        }
        let _ = writeln!(out, "class {}(ctypes.Structure):", struct_name(&s.name));
        out.push_str(&docstring(
            &format!(
                "{}\n\nThe C layout, field for field. `{}` is the value class over it.",
                s.doc,
                binding_type_name(&s.name)
            ),
            "    ",
        ));
        out.push_str("\n    _fields_ = [\n");
        for f in &s.fields {
            let _ = writeln!(
                out,
                "        (\"{}\", {}),",
                identifier(&f.name),
                ctypes_type(api, &f.ty)
            );
        }
        out.push_str("    ]\n\n\n");
    }
}

/// The structs in an order where a struct comes after everything it holds
/// by value.
fn ordered(api: &Api) -> Vec<&StructDef> {
    let mut done: Vec<&str> = Vec::new();
    let mut out: Vec<&StructDef> = Vec::new();
    // A boundary struct nests one or two levels, so repeated passes are
    // simpler than a graph and cannot loop for ever: each pass places at
    // least one struct until none is left that can be placed.
    for _ in 0..api.structs.len() {
        if out.len() == api.structs.len() {
            break;
        }
        for s in &api.structs {
            if done.contains(&s.name.as_str()) {
                continue;
            }
            let ready = s.fields.iter().all(|f| match &f.ty {
                TypeRef::Struct { name } => done.contains(&name.as_str()),
                _ => true,
            });
            if ready {
                done.push(&s.name);
                out.push(s);
            }
        }
    }
    // A cycle is impossible in a `#[repr(C)]` value type, but a struct
    // left over is emitted rather than dropped.
    for s in &api.structs {
        if !done.contains(&s.name.as_str()) {
            out.push(s);
        }
    }
    out
}

fn render_callbacks(out: &mut String, api: &Api) {
    for c in &api.callbacks {
        let returns = match &c.returns {
            TypeRef::Void => String::from("None"),
            other => ctypes_type(api, other),
        };
        let params = c.params.iter().map(|p| ctypes_type(api, &p.ty));
        out.push_str(&comment(&c.doc, ""));
        let _ = writeln!(
            out,
            "{} = ctypes.CFUNCTYPE(\n    {},\n)\n",
            binding_type_name(&c.name),
            std::iter::once(returns)
                .chain(params)
                .collect::<Vec<_>>()
                .join(",\n    ")
        );
    }
}

fn render_exception(out: &mut String, api: &Api) {
    let fields = error_fields(api);
    let mut params = String::new();
    let mut sets = String::new();
    let mut shown = String::new();
    for field in &fields {
        let _ = writeln!(params, "        {field}: str = \"\",");
        let _ = writeln!(sets, "        self.{field} = {field}");
        // The detail is printed on its own line rather than labelled: it
        // is the boundary's own code for what happened, and reads as one.
        if field != "detail" {
            let _ = writeln!(
                shown,
                "        if self.{field}:\n            parts.append(f\"{field}: {{self.{field}}}\")"
            );
        }
    }
    let _ = writeln!(
        out,
        "class TeistroError(Exception):\n    \"\"\"A refusal from the library, with everything it said about it.\n\n    `status` is the boundary's own code, so a caller may match on it\n    rather than on the message; the rest are filled when the library\n    named them.\n    \"\"\"\n\n    def __init__(\n        self,\n        status: Status,\n        message: str,\n        *,\n{params}    ) -> None:\n        super().__init__(message)\n        self.status = status\n        self.message = message\n{sets}\n    def __str__(self) -> str:\n        parts = [self.message]\n        if self.detail:\n            parts.append(self.detail)\n{shown}        return \"\\n  \".join(parts)\n"
    );
}

/// The text fields the boundary's error struct carries, which are the
/// ones the exception holds. Read from the description rather than
/// listed, so a field added there reaches the exception.
fn error_fields(api: &Api) -> Vec<String> {
    api.structs
        .iter()
        .find(|s| s.name.ends_with("Error"))
        .map(|s| {
            s.fields
                .iter()
                .filter(|f| f.name != "message" && matches!(f.ty.pointee(), Some(TypeRef::Char)))
                .map(|f| identifier(&f.name))
                .collect()
        })
        .unwrap_or_default()
}

/// The helpers the generated code calls, written once.
const HELPERS: &str = r#"
def _text(pointer: Any) -> str:
    """A NUL-terminated UTF-8 string the library lent, as a `str`."""
    if not pointer:
        return ""
    raw = pointer if isinstance(pointer, bytes) else ctypes.string_at(pointer)
    return raw.decode("utf-8")


def _c_value(value: Any) -> Any:
    """A field's value as `ctypes` takes it.

    An enum becomes its id, a bool a number, `None` a zero, and anything
    else itself.
    """
    if value is None:
        return 0
    if isinstance(value, Member):
        return int(value)
    if isinstance(value, bool):
        return 1 if value else 0
    return value


def _lent_string(raw: Any) -> str:
    """A string the library lent until the next call, copied out now."""
    if not raw.data:
        return ""
    return ctypes.string_at(raw.data, raw.len).decode("utf-8")


"#;

/// The branded quantities as `float` subclasses: a `Latitude` is a float
/// at run time and its own type to a checker, so a latitude cannot be
/// passed where a longitude is wanted, and the constructor checks the
/// range the description states (ADR-0023, and Phase 1's exit criterion).
///
/// A `float` subclass rather than a `NewType` because it is both: a
/// `NewType` gives a checker the distinction and can carry no check, and
/// a validating factory beside it would be a second name for one idea.
fn render_brands(out: &mut String, api: &Api) {
    for (brand, field) in crate::emit::ts::brands(api) {
        let name = pascal(&brand);
        let unit = field.meta.unit.as_deref().unwrap_or("");
        let where_ = if unit.is_empty() {
            String::new()
        } else {
            format!(" in {unit}")
        };
        let range = field.meta.range.as_deref().unwrap_or("");
        let check = match crate::emit::ts::brand_range(field) {
            Some((low, high)) => format!(
                "        if not {low} <= value <= {high}:\n            raise ValueError(\n                f\"a {brand} is {range} and {{value}} is not\"\n            )\n"
            ),
            None => String::new(),
        };
        let _ = writeln!(
            out,
            "class {name}(float):\n    \"\"\"A {brand}{where_}{}.\n\n    Its own type, so it cannot be passed where another quantity is\n    wanted, and a float at run time, so it crosses the boundary as one.\n    \"\"\"\n\n    __slots__ = ()\n\n    def __new__(cls, value: float) -> \"{name}\":\n{check}        return super().__new__(cls, value)\n",
            if range.is_empty() {
                String::new()
            } else {
                format!(", {range}")
            }
        );
    }
}

/// A struct a binding shows as a value class, by the same rule the Dart
/// emitter uses: a plain object or a set of caller-allocated columns,
/// with at least one field of its own, and no array of structs.
fn shown(api: &Api, s: &StructDef) -> bool {
    matches!(s.role, StructRole::Object | StructRole::Columns)
        && s.fields
            .iter()
            .any(|f| f.name != "struct_size" && !f.name.starts_with("reserved"))
        && !s.fields.iter().zip(field_roles(api, s)).any(|(f, role)| {
            matches!(role, FieldRole::Array { .. })
                && matches!(f.ty.pointee(), Some(TypeRef::Struct { .. }))
        })
}

/// The Python type of a field, from its role.
fn value_type(field: &FieldDef, role: &FieldRole) -> String {
    let base = match role {
        FieldRole::Flag => String::from("bool"),
        FieldRole::BitSet { enum_name } => {
            format!("frozenset[{}]", binding_type_name(enum_name))
        }
        FieldRole::Array { .. } | FieldRole::Column => match &field.meta.enum_name {
            Some(name) => format!("Sequence[{}]", binding_type_name(name)),
            None => format!("Sequence[{}]", python_scalar(element(field))),
        },
        FieldRole::FixedBytes { .. } => String::from("bytes"),
        FieldRole::Text => String::from("str"),
        FieldRole::Nested { name } => binding_type_name(name),
        FieldRole::Optional { .. } => match &field.ty {
            TypeRef::Struct { name } => binding_type_name(name),
            other => plain_value(field, other),
        },
        _ => plain_value(field, &field.ty),
    };
    if field.meta.nullable || matches!(role, FieldRole::Optional { .. }) {
        format!("Optional[{base}]")
    } else {
        base
    }
}

fn plain_value(field: &FieldDef, ty: &TypeRef) -> String {
    if let Some(brand) = &field.meta.brand {
        return pascal(brand);
    }
    if let Some(name) = &field.meta.enum_name {
        return binding_type_name(name);
    }
    match ty {
        TypeRef::Scalar { scalar } => python_scalar(*scalar).to_string(),
        TypeRef::Enum { name } => binding_type_name(name),
        TypeRef::Pointer { to, .. } if matches!(**to, TypeRef::Char) => String::from("str"),
        _ => String::from("int"),
    }
}

fn element(field: &FieldDef) -> Scalar {
    field
        .ty
        .pointee()
        .and_then(TypeRef::as_scalar)
        .unwrap_or(Scalar::U8)
}

fn render_value_class(out: &mut String, api: &Api, s: &StructDef) {
    let name = binding_type_name(&s.name);
    let roles = field_roles(api, s);
    let shown_fields: Vec<(&FieldDef, &FieldRole)> = s
        .fields
        .iter()
        .zip(&roles)
        .filter(|(_, role)| role.is_shown())
        .collect();
    let _ = writeln!(out, "@dataclass(frozen=True)\nclass {name}:");
    out.push_str(&docstring(&s.doc, "    "));
    // A field with a default has to follow every field without one, so
    // the optional fields come last whatever order the struct declares
    // them in; every call names its arguments, so nothing else notices.
    let optional = |f: &FieldDef, role: &FieldRole| {
        f.meta.nullable || matches!(role, FieldRole::Optional { .. })
    };
    let (required, defaulted): (Vec<_>, Vec<_>) = shown_fields
        .iter()
        .partition(|(f, role)| !optional(f, role));
    for (f, role) in required.iter().chain(defaulted.iter()) {
        let _ = writeln!(
            out,
            "\n    {}: {}{}",
            identifier(&f.name),
            value_type(f, role),
            if optional(f, role) { " = None" } else { "" }
        );
        out.push_str(&field_docstring(f, "    "));
    }
    render_write(out, s, &roles);
    render_read(out, s, &roles, &name);
    out.push_str("\n\n");
}

/// `_into` and `_to_c`: the value written into the C struct a call takes
/// by pointer. Anything the struct points at is kept alive in `owned`,
/// which the caller holds until the call has returned.
fn render_write(out: &mut String, s: &StructDef, roles: &[FieldRole]) {
    let c = struct_name(&s.name);
    let name = binding_type_name(&s.name);
    let _ = writeln!(
        out,
        "\n    def _into(self, raw: {c}, owned: list[Any]) -> None:\n        \"\"\"Writes this value into a C struct, which may be one held inside\n        another rather than one of its own.\n\n        Anything the struct points at is appended to `owned`, which the\n        caller keeps alive until the call has returned.\n        \"\"\""
    );
    let mut wrote = false;
    if has_handshake(s) {
        let _ = writeln!(out, "        raw.struct_size = ctypes.sizeof({c})");
        wrote = true;
    }
    for (f, role) in s.fields.iter().zip(roles) {
        let field = identifier(&f.name);
        let line = match role {
            FieldRole::Handshake | FieldRole::Reserved => continue,
            FieldRole::Flag => format!("raw.{field} = 1 if self.{field} else 0"),
            FieldRole::BitSet { .. } => format!(
                "raw.{field} = 0\n        for _member in self.{field}:\n            raw.{field} |= 1 << int(_member)"
            ),
            FieldRole::Count { of } => format!("raw.{field} = len(self.{})", identifier(of)),
            FieldRole::Presence { of } => {
                format!("raw.{field} = 0 if self.{} is None else 1", identifier(of))
            }
            FieldRole::Array { .. } | FieldRole::Column => {
                let scalar = ctypes_scalar(element(f));
                format!(
                    "_{field} = ({scalar} * len(self.{field}))(\n            *(_c_value(_v) for _v in self.{field})\n        )\n        owned.append(_{field})\n        raw.{field} = ctypes.cast(_{field}, ctypes.POINTER({scalar}))"
                )
            }
            FieldRole::FixedBytes { len } => {
                format!("raw.{field} = (ctypes.c_uint8 * {len})(*self.{field}[:{len}])")
            }
            FieldRole::Text => format!(
                "_{field} = None if self.{field} is None else self.{field}.encode(\"utf-8\")\n        owned.append(_{field})\n        raw.{field} = _{field}"
            ),
            FieldRole::Nested { .. } => format!("self.{field}._into(raw.{field}, owned)"),
            FieldRole::Optional { .. } => match &f.ty {
                TypeRef::Struct { name } => format!(
                    "if self.{field} is not None:\n            self.{field}._into(raw.{field}, owned)"
                )
                .replace("{name}", &binding_type_name(name)),
                _ => format!("raw.{field} = _c_value(self.{field})"),
            },
            FieldRole::Value => format!("raw.{field} = _c_value(self.{field})"),
        };
        let _ = writeln!(out, "        {line}");
        wrote = true;
    }
    if !wrote {
        out.push_str("        return None\n");
    }
    let _ = writeln!(
        out,
        "\n    def _to_c(self, owned: list[Any]) -> {c}:\n        \"\"\"This value as a fresh C struct, ready to be passed by pointer.\"\"\"\n        raw = {c}()\n        self._into(raw, owned)\n        return raw\n\n    @classmethod\n    def _empty(cls) -> {name}:\n        \"\"\"The zero value, for a nested struct a caller left out.\"\"\"\n        return cls._of({c}())"
    );
}

/// `_of`: the value read back out of a C struct the library filled.
fn render_read(out: &mut String, s: &StructDef, roles: &[FieldRole], name: &str) {
    let c = struct_name(&s.name);
    let _ = writeln!(
        out,
        "\n    @classmethod\n    def _of(cls, raw: {c}) -> {name}:\n        \"\"\"The value the library wrote into a C struct.\"\"\"\n        return cls("
    );
    for (f, role) in s.fields.iter().zip(roles).filter(|(_, r)| r.is_shown()) {
        let field = identifier(&f.name);
        let read = match role {
            FieldRole::Flag => format!("raw.{field} != 0"),
            FieldRole::BitSet { enum_name } => format!(
                "frozenset(\n                {}(_bit)\n                for _bit in range(32)\n                if raw.{field} & (1 << _bit)\n            )",
                binding_type_name(enum_name)
            ),
            FieldRole::Array { count } => {
                let count = identifier(count);
                match &f.meta.enum_name {
                    Some(enum_name) => format!(
                        "[\n                {}(raw.{field}[_i]) for _i in range(raw.{count})\n            ]\n            if raw.{field}\n            else [],",
                        binding_type_name(enum_name)
                    ),
                    None => format!(
                        "[raw.{field}[_i] for _i in range(raw.{count})]\n            if raw.{field}\n            else [],"
                    ),
                }
            }
            FieldRole::Column => String::from("[],"),
            FieldRole::FixedBytes { len } => format!("bytes(raw.{field}[:{len}]),"),
            FieldRole::Text => format!("_text(raw.{field}),"),
            FieldRole::Nested { name } => {
                format!("{}._of(raw.{field}),", binding_type_name(name))
            }
            FieldRole::Optional { flag } => {
                let flag = identifier(flag);
                match &f.ty {
                    TypeRef::Struct { name } => format!(
                        "{}._of(raw.{field}) if raw.{flag} else None,",
                        binding_type_name(name)
                    ),
                    other => format!("{} if raw.{flag} else None,", read_plain(f, other, &field)),
                }
            }
            _ => format!("{},", read_plain(f, &f.ty, &field)),
        };
        // An array's reader already carries its own trailing comma,
        // because it spans lines and the comma belongs after the `else`.
        let read = read.strip_suffix(',').unwrap_or(&read).to_string();
        let _ = writeln!(out, "            {field}={read},");
    }
    out.push_str("        )\n");
}

fn read_plain(field: &FieldDef, ty: &TypeRef, name: &str) -> String {
    if let Some(brand) = &field.meta.brand {
        return format!("{}(raw.{name})", pascal(brand));
    }
    if let Some(enum_name) = &field.meta.enum_name {
        return format!("{}(raw.{name})", binding_type_name(enum_name));
    }
    match ty {
        TypeRef::Enum { name: enum_name } => {
            format!("{}(raw.{name})", binding_type_name(enum_name))
        }
        TypeRef::Pointer { to, .. } if matches!(**to, TypeRef::Char) => {
            format!("_text(raw.{name})")
        }
        TypeRef::Scalar { scalar } if scalar.is_float() => format!("float(raw.{name})"),
        _ => format!("raw.{name}"),
    }
}

// ── What a call answers with ───────────────────────────────────────────────

/// The name of the `NamedTuple` a call with more than one result answers
/// with: `ts_time_civil` gives a `CivilResult` of a civil time and a zone
/// resolution, which is typed where a dictionary would not be.
fn result_type(api: &Api, f: &FunctionDef) -> String {
    let bare = api
        .opaques
        .iter()
        .find(|o| methods(api, o).iter().any(|m| m.name == f.name))
        .map_or_else(
            || {
                f.name
                    .strip_prefix(&api.prefix)
                    .unwrap_or(&f.name)
                    .to_string()
            },
            |o| method_name(&api.prefix, &o.name, &f.name),
        );
    format!("{}Result", pascal(&snake(&bare)))
}

/// One `NamedTuple` per call that hands back more than one thing.
fn render_results(out: &mut String, api: &Api) {
    for f in &api.functions {
        let handed = results(api, f);
        if handed.len() < 2 {
            continue;
        }
        let _ = writeln!(out, "class {}(NamedTuple):", result_type(api, f));
        out.push_str(&docstring(
            &format!("What `{}` hands back.", f.name),
            "    ",
        ));
        for hand in &handed {
            let _ = writeln!(out, "\n    {}: {}", hand.name(), handed_type(api, hand));
            out.push_str(&docstring(&handed_doc(api, hand), "    "));
        }
        out.push_str("\n\n");
    }
}

/// The Python type of one thing a call hands back.
fn handed_type(api: &Api, hand: &Handed<'_>) -> String {
    match hand {
        Handed::Returned(scalar) => python_scalar(*scalar).to_string(),
        Handed::Struct(p) => pointee_struct(api, p)
            .map_or_else(|| String::from("Any"), |s| binding_type_name(&s.name)),
        Handed::Blob(_) => String::from("bytes"),
        Handed::Owned(_) | Handed::Lent(_) => String::from("str"),
        Handed::Scalar(p) => {
            p.ty.pointee()
                .and_then(TypeRef::as_scalar)
                .map_or("int", python_scalar)
                .to_string()
        }
    }
}

fn handed_doc(api: &Api, hand: &Handed<'_>) -> String {
    match hand {
        Handed::Returned(_) => String::from("What the call returned."),
        Handed::Struct(p) => {
            pointee_struct(api, p).map_or_else(|| format!("The `{}`.", p.name), |s| s.doc.clone())
        }
        Handed::Blob(p) | Handed::Owned(p) | Handed::Lent(p) | Handed::Scalar(p) => {
            format!("The call's `{}`.", p.name)
        }
    }
}

/// How a call reads one result back, given the expression for the library.
fn handed_read(api: &Api, hand: &Handed<'_>, lib: &str) -> String {
    match hand {
        Handed::Returned(scalar) => format!("{}(value)", python_scalar(*scalar)),
        Handed::Struct(p) => {
            let raw = format!("_{}", identifier(&p.name));
            pointee_struct(api, p).map_or(raw.clone(), |s| {
                format!("{}._of({raw})", binding_type_name(&s.name))
            })
        }
        Handed::Blob(p) => format!("_take_blob({lib}, _{})", identifier(&p.name)),
        Handed::Owned(p) => format!("_take_string({lib}, _{})", identifier(&p.name)),
        Handed::Lent(p) => format!("_lent_string(_{})", identifier(&p.name)),
        Handed::Scalar(p) => format!("_{}.value", identifier(&p.name)),
    }
}

// ── Marshalling ────────────────────────────────────────────────────────────

/// What marshalling a call comes to.
struct Marshalled {
    /// The parameters it takes in Python, after the receiver.
    signature: String,
    /// The lines that set its arguments up, unindented.
    setup: Vec<String>,
    /// The arguments, in order.
    args: Vec<String>,
    /// The lines that read the results back, unindented.
    reads: Vec<String>,
    /// The type it answers with, and the expression it answers.
    returns: String,
    result: String,
    /// Whether anything it allocates has to outlive the call.
    owns: bool,
    /// Whether the call's own return value has to be bound: a scalar, or
    /// the static string four of the entry points answer with.
    binds: bool,
}

/// What one parameter contributes to a call.
#[derive(Default)]
struct Marshalling {
    signature: String,
    setup: Vec<String>,
    args: Vec<String>,
    owns: bool,
}

/// Marshals one call. A role with no rule here is a **generation-time
/// refusal** rather than a guess: the falsification pass counted the
/// roles the description actually uses, and `array_in` has no instance,
/// so a rule written for it would be a rule nothing exercises
/// (`03-design/binding-surface-measured.md` §1).
fn marshal(api: &Api, f: &FunctionDef, lib: &str, receiver: Option<&str>) -> Marshalled {
    let mut signature = String::new();
    let mut setup: Vec<String> = Vec::new();
    let mut args: Vec<String> = Vec::new();
    let mut owns = false;
    for (index, p) in f.params.iter().enumerate() {
        let one = parameter(api, f, p, index, receiver);
        signature.push_str(&one.signature);
        setup.extend(one.setup);
        args.extend(one.args);
        owns |= one.owns;
    }
    finish(api, f, lib, signature, setup, args, owns)
}

/// The arms, one per role, so the whole of the marshalling reads as the
/// list of roles the pass counted.
fn parameter(
    api: &Api,
    f: &FunctionDef,
    p: &ParamDef,
    index: usize,
    receiver: Option<&str>,
) -> Marshalling {
    let mut signature = String::new();
    let mut setup: Vec<String> = Vec::new();
    let mut args: Vec<String> = Vec::new();
    let mut owns = false;
    {
        let name = identifier(&p.name);
        match p.role {
            // The handle of the class being rendered is `self`; any other
            // opaque's is a parameter, and the class that holds it is what
            // the caller passes (`rules::factories`).
            Role::Handle => {
                let held = crate::rules::pointee_opaque(api, p).map(|o| o.name.clone());
                if held.as_deref() == receiver || receiver.is_none() {
                    args.push(String::from("self._raw"));
                } else {
                    let class = format!(
                        "Teistro{}",
                        binding_type_name(held.as_deref().unwrap_or_default())
                    );
                    signature.push_str(&format!(", {name}: {class}"));
                    args.push(format!("{name}._raw"));
                }
            }
            Role::HandleOut => args.push(String::from("ctypes.byref(handle)")),
            // The pointer the SDK hands back to every callback. A provider
            // written in Python closes over itself, so nothing has to be
            // passed back and it is null rather than a fabricated address.
            Role::UserData => args.push(String::from("None")),
            Role::VtableIn => {
                let _ = write!(signature, ", provider: Any");
                args.push(String::from("provider"));
            }
            Role::Value => {
                let _ = write!(signature, ", {name}: {}", value_param_type(p));
                args.push(format!("_c_value({name})"));
            }
            Role::StructIn => {
                let ty = pointee_struct(api, p)
                    .map_or_else(|| String::from("Any"), |s| binding_type_name(&s.name));
                let _ = write!(signature, ", {name}: {ty}");
                setup.push(format!("_{name} = {name}._to_c(owned)"));
                args.push(format!("ctypes.byref(_{name})"));
                owns = true;
            }
            Role::StructOut | Role::StringOut | Role::StrOut => {
                let of = pointee_struct(api, p);
                let raw =
                    of.map_or_else(|| String::from("ctypes.c_void_p"), |s| struct_name(&s.name));
                setup.push(format!("_{name} = {raw}()"));
                // The caller of a struct the library writes sets the
                // handshake, because the library checks it before it
                // writes and refuses a size that is not its own.
                if of.is_some_and(has_handshake) {
                    setup.push(format!("_{name}.struct_size = ctypes.sizeof({raw})"));
                }
                args.push(format!("ctypes.byref(_{name})"));
            }
            Role::ScalarOut => {
                let scalar =
                    p.ty.pointee()
                        .and_then(TypeRef::as_scalar)
                        .map_or("ctypes.c_int32", ctypes_scalar);
                setup.push(format!("_{name} = {scalar}()"));
                args.push(format!("ctypes.byref(_{name})"));
            }
            Role::BlobOut => {
                setup.push(format!("_{name} = {}()", blob_struct(api)));
                args.push(format!("ctypes.byref(_{name})"));
            }
            Role::StringIn => {
                let _ = write!(signature, ", {name}: str");
                setup.push(format!("_{name} = {name}.encode(\"utf-8\")"));
                setup.push(format!("owned.append(_{name})"));
                args.push(format!("_{name}"));
                owns = true;
            }
            Role::BytesIn => {
                let _ = write!(signature, ", {name}: bytes");
                setup.push(format!(
                    "_{name} = (ctypes.c_uint8 * len({name})).from_buffer_copy({name})"
                ));
                setup.push(format!("owned.append(_{name})"));
                args.push(format!("_{name}"));
                owns = true;
            }
            Role::Length => {
                let of = f
                    .params
                    .get(index.wrapping_sub(1))
                    .map_or_else(String::new, |before| identifier(&before.name));
                args.push(format!("len({of})"));
            }
            Role::StringFree | Role::BlobFree => args.push(format!("ctypes.byref({name})")),
            // Not a rule left out: the falsification pass counted the
            // roles the description uses and this one has **no
            // instance**, so a rule written for it would be a rule
            // nothing exercises. The generator stops and says which
            // parameter it could not place, which is a build that fails
            // rather than a marshalling that is quietly wrong.
            Role::ArrayIn => unreachable("array_in", &p.name, &f.name),
        }
    }
    Marshalling {
        signature,
        setup,
        args,
        owns,
    }
}

/// A role the description has no instance of.
///
/// A generator that cannot place a parameter must **stop**: the
/// alternative is a file that compiles and marshals the wrong bytes. The
/// panic is the build failing, with the parameter named, and it is the
/// only one in this crate.
#[expect(
    clippy::panic,
    reason = "a generator that cannot place a parameter stops the build rather than emitting a wrong marshalling"
)]
fn unreachable(role: &str, param: &str, function: &str) -> ! {
    panic!(
        "no Python rule for the `{role}` parameter `{param}` of `{function}`: the \
         description had no instance of that role when the emitter was written \
         (03-design/binding-surface-measured.md \u{a7}1)"
    )
}

/// What the call answers with, once its parameters are placed.
fn finish(
    api: &Api,
    f: &FunctionDef,
    lib: &str,
    signature: String,
    setup: Vec<String>,
    args: Vec<String>,
    owns: bool,
) -> Marshalled {
    let handed = results(api, f);
    let reads: Vec<String> = handed
        .iter()
        .map(|hand| format!("{} = {}", hand.name(), handed_read(api, hand, lib)))
        .collect();
    // Four entry points answer with a static NUL-terminated string,
    // which is not a scalar and so is not one of the results the shared
    // rules count; it is still the only thing the call hands back.
    let text =
        matches!(&f.returns, Some(TypeRef::Pointer { to, .. }) if matches!(**to, TypeRef::Char));
    let (returns, result) = match handed.as_slice() {
        [] if text => (String::from("str"), String::from("_text(value)")),
        [] => (String::from("None"), String::from("None")),
        // A returned scalar is coerced once, where it is read: `ctypes`
        // answers `Any` and the declared return type has to be true.
        [one] => (handed_type(api, one), one.name()),
        many => (
            result_type(api, f),
            format!(
                "{}({})",
                result_type(api, f),
                many.iter()
                    .map(|h| format!("{0}={0}", h.name()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        ),
    };
    Marshalled {
        signature,
        setup,
        args,
        reads,
        returns,
        result,
        owns,
        binds: text || returned_scalar(f).is_some(),
    }
}

/// The Python type of a by-value parameter, from its metadata.
fn value_param_type(p: &ParamDef) -> String {
    if let Some(enum_name) = &p.meta.enum_name {
        return binding_type_name(enum_name);
    }
    match &p.ty {
        TypeRef::Enum { name } => binding_type_name(name),
        TypeRef::Scalar { scalar } => python_scalar(*scalar).to_string(),
        _ => String::from("int"),
    }
}

/// A call written at a width a Python reader is used to: on one line
/// when it fits, and one argument to a line when it does not.
fn call_text(lib: &str, f: &FunctionDef, args: &[String], indent: &str) -> String {
    let one_line = format!("{lib}.{}({})", f.name, args.join(", "));
    if indent.len() + one_line.len() + "status = Status()".len() <= 79 || args.is_empty() {
        return one_line;
    }
    format!(
        "{lib}.{}(\n{indent}    {},\n{indent})",
        f.name,
        args.join(&format!(",\n{indent}    "))
    )
}

/// The body of a call: the setup, the call itself, the refusal and the
/// results, at one indent.
fn render_body(
    out: &mut String,
    api: &Api,
    f: &FunctionDef,
    m: &Marshalled,
    lib: &str,
    indent: &str,
    refuse: &str,
) {
    if m.owns {
        let _ = writeln!(out, "{indent}owned: list[Any] = []");
    }
    for line in &m.setup {
        let _ = writeln!(out, "{indent}{line}");
    }
    let call = call_text(lib, f, &m.args, indent);
    if returns_status(api, f) {
        let _ = writeln!(
            out,
            "{indent}status = Status({call})\n{indent}if status != Status.OK:\n{indent}    {refuse}"
        );
    } else if m.binds {
        let _ = writeln!(out, "{indent}value = {call}");
    } else {
        let _ = writeln!(out, "{indent}{call}");
    }
    if m.owns {
        let _ = writeln!(out, "{indent}owned.clear()");
    }
    for line in &m.reads {
        let _ = writeln!(out, "{indent}{line}");
    }
    let _ = writeln!(out, "{indent}return {}", m.result);
}

// ── The library ────────────────────────────────────────────────────────────

fn render_library(out: &mut String, api: &Api) {
    out.push_str("class TeistroLibrary:\n    \"\"\"Every entry point of an open library, with its signature set.\n\n    `ctypes` believes whatever `argtypes` and `restype` say, so both are\n    generated from the description rather than written, and a call that\n    would have been given the wrong width is an error here instead of a\n    wrong number later.\n    \"\"\"\n\n    def __init__(self, library: ctypes.CDLL) -> None:\n        self.library = library\n");
    for f in &api.functions {
        let argtypes: Vec<String> = f.params.iter().map(|p| ctypes_type(api, &p.ty)).collect();
        let restype = match &f.returns {
            None | Some(TypeRef::Void) => String::from("None"),
            Some(other) => ctypes_type(api, other),
        };
        let _ = writeln!(
            out,
            "        self.{0}: Any = library.{0}\n        self.{0}.argtypes = [{1}]\n        self.{0}.restype = {restype}",
            f.name,
            if argtypes.is_empty() {
                String::new()
            } else {
                format!(
                    "\n            {},\n        ",
                    argtypes.join(",\n            ")
                )
            }
        );
    }
    out.push_str("\n\n");
}

// ── The context ────────────────────────────────────────────────────────────

fn render_context(out: &mut String, api: &Api, opaque: &OpaqueDef) {
    let name = binding_type_name(&opaque.name);
    let handle = format!("_{name}");
    let free = format!("_free_{}", snake(&name));
    let _ = writeln!(
        out,
        "class Teistro{name}:\n    \"\"\"A live context, and every call that takes one.\n\n    The handle is freed by `close`, by leaving a `with` block, or by the\n    finaliser when neither happened; a result that waits for the\n    collector can exhaust memory (ADR-0007, finding 4), so the explicit\n    forms are the ones to use.\n    \"\"\"\n\n    def __init__(\n        self, lib: TeistroLibrary, handle: \"ctypes._Pointer[{handle}]\"\n    ) -> None:\n        self._lib = lib\n        self._handle: Optional[\"ctypes._Pointer[{handle}]\"] = handle\n        self._finalise = weakref.finalize(self, {free}, lib, handle)\n"
    );
    if let Some(ctor) = constructor(api, opaque) {
        render_constructor(out, api, ctor, &name, &handle);
    }
    for factory in crate::rules::factories(api, opaque) {
        render_factory(out, api, factory, opaque, &name, &handle);
    }
    let _ = writeln!(
        out,
        "    @property\n    def _raw(self) -> \"ctypes._Pointer[{handle}]\":\n        \"\"\"The live handle, or a refusal saying it was closed.\"\"\"\n        if self._handle is None:\n            raise TeistroError(\n                Status.INVALID_ARG,\n                \"this context has been closed\",\n                hint=\"open another one\",\n            )\n        return self._handle\n\n    def close(self) -> None:\n        \"\"\"Frees the context's native memory. Closing twice is allowed.\"\"\"\n        if self._handle is not None:\n            self._handle = None\n            self._finalise()\n\n    def __enter__(self) -> Teistro{name}:\n        return self\n\n    def __exit__(\n        self,\n        kind: Optional[type[BaseException]],\n        value: Optional[BaseException],\n        traceback: Optional[TracebackType],\n    ) -> None:\n        self.close()"
    );
    render_error_reader(out, api);
    for m in methods(api, opaque) {
        render_method(out, api, opaque, m);
    }
    out.push_str("\n\n");
    if let Some(freer) = destructor(api, opaque) {
        let _ = writeln!(
            out,
            "def {free}(lib: TeistroLibrary, handle: Any) -> None:\n    \"\"\"The finaliser: frees a handle whoever let it go.\"\"\"\n    if handle:\n        lib.{}(handle)\n\n",
            freer.name
        );
    }
}

fn render_constructor(out: &mut String, api: &Api, ctor: &FunctionDef, name: &str, handle: &str) {
    let m = marshal(api, ctor, "lib", None);
    let _ = writeln!(
        out,
        "    @classmethod\n    def _new(cls, lib: TeistroLibrary{}) -> Teistro{name}:",
        m.signature
    );
    out.push_str(&docstring(&ctor.doc, "        "));
    if m.owns {
        out.push_str("        owned: list[Any] = []\n");
    }
    let _ = writeln!(out, "        handle = ctypes.POINTER({handle})()");
    for line in &m.setup {
        let _ = writeln!(out, "        {line}");
    }
    let _ = writeln!(
        out,
        "        status = Status({})\n        if status != Status.OK:\n            _refuse(lib, status, {})\n        return cls(lib, handle)\n",
        call_text("lib", ctor, &m.args, "        "),
        ctor.params
            .iter()
            .find(|p| p.role == Role::StringOut)
            .map_or_else(
                || String::from("None"),
                |p| format!("_{}", identifier(&p.name))
            ),
    );
}

/// A second way in, beside the constructor: a class method named for
/// the entry point it calls.
///
/// `_new_with_provider` beside `_new`, because Python has one
/// `__init__` and a class method is how a second way in is spelled. The
/// body is the constructor's, and a `handle` of another opaque becomes a
/// parameter of that class rather than `self` (`rules::factories`).
fn render_factory(
    out: &mut String,
    api: &Api,
    f: &FunctionDef,
    opaque: &OpaqueDef,
    name: &str,
    handle: &str,
) {
    let method = identifier(&method_name(&api.prefix, &opaque.name, &f.name));
    let m = marshal(api, f, "lib", Some(&opaque.name));
    let _ = writeln!(
        out,
        "    @classmethod\n    def _{method}(cls, lib: TeistroLibrary{}) -> Teistro{name}:",
        m.signature
    );
    out.push_str(&docstring(&f.doc, "        "));
    if m.owns {
        out.push_str("        owned: list[Any] = []\n");
    }
    let _ = writeln!(out, "        handle = ctypes.POINTER({handle})()");
    for line in &m.setup {
        let _ = writeln!(out, "        {line}");
    }
    let _ = writeln!(
        out,
        "        status = Status({})\n        if status != Status.OK:\n            _refuse(lib, status, {})\n        return cls(lib, handle)\n",
        call_text("lib", f, &m.args, "        "),
        f.params
            .iter()
            .find(|p| p.role == Role::StringOut)
            .map_or_else(
                || String::from("None"),
                |p| format!("_{}", identifier(&p.name))
            ),
    );
}

/// The reader for the last error, which every refusal on a live handle
/// goes through: the library keeps it beside the context, and the
/// exception carries every field the boundary filled.
fn render_error_reader(out: &mut String, api: &Api) {
    let Some(reader) = api
        .functions
        .iter()
        .find(|f| f.name.ends_with("_last_error"))
    else {
        return;
    };
    let Some(error) = reader
        .params
        .iter()
        .find(|p| p.role == Role::StructOut)
        .and_then(|p| pointee_struct(api, p))
    else {
        return;
    };
    let mut reads = String::new();
    for field in error_fields(api) {
        let _ = writeln!(reads, "                    {field}=found.{field} or \"\",");
    }
    let _ = writeln!(
        out,
        "\n    def _raise(self, status: Status) -> None:\n        \"\"\"Raises what the library said about its last refusal.\"\"\"\n        raw = {0}()\n        raw.struct_size = ctypes.sizeof({0})\n        if self._handle is not None:\n            if self._lib.{1}(self._handle, ctypes.byref(raw)) == 0:\n                found = {2}._of(raw)\n                raise TeistroError(\n                    status,\n                    found.message or status.key,\n{reads}                )\n        raise TeistroError(status, status.key)",
        struct_name(&error.name),
        reader.name,
        binding_type_name(&error.name),
    );
}

fn render_method(out: &mut String, api: &Api, opaque: &OpaqueDef, f: &FunctionDef) {
    let name = identifier(&method_name(&api.prefix, &opaque.name, &f.name));
    let m = marshal(api, f, "self._lib", Some(&opaque.name));
    let _ = writeln!(
        out,
        "\n    def {name}(self{}) -> {}:",
        m.signature, m.returns
    );
    out.push_str(&docstring(&f.doc, "        "));
    render_body(
        out,
        api,
        f,
        &m,
        "self._lib",
        "        ",
        "self._raise(status)",
    );
}

/// The calls that hang off nothing: the versions, the status message and
/// the frame, which need no context.
fn render_free_functions(out: &mut String, api: &Api) {
    for f in &api.functions {
        // What is left when the handle's own calls are taken away: the
        // versions, the status message and the frame. The two freeing
        // entry points are left out too — the helpers below call them,
        // and a caller that freed a blob by hand would free it twice.
        let frees = f
            .params
            .iter()
            .any(|p| matches!(p.role, Role::StringFree | Role::BlobFree));
        if !is_free_function(f) || frees {
            continue;
        }
        let m = marshal(api, f, "lib", None);
        let _ = writeln!(
            out,
            "def {}(lib: TeistroLibrary{}) -> {}:",
            identifier(f.name.strip_prefix(&api.prefix).unwrap_or(&f.name)),
            m.signature,
            m.returns
        );
        out.push_str(&docstring(&f.doc, "    "));
        render_body(out, api, f, &m, "lib", "    ", "_refuse(lib, status, None)");
        out.push_str("\n\n");
    }
    out.push_str(&free_helpers(api));
}

/// The helpers a blob-returning or string-returning call uses. The names
/// of the two freeing entry points come from the description, so a
/// renamed symbol reaches them.
fn free_helpers(api: &Api) -> String {
    let string_free = api
        .functions
        .iter()
        .find(|f| f.params.iter().any(|p| p.role == Role::StringFree))
        .map_or_else(|| String::from("ts_string_free"), |f| f.name.clone());
    let blob_free = api
        .functions
        .iter()
        .find(|f| f.params.iter().any(|p| p.role == Role::BlobFree))
        .map_or_else(|| String::from("ts_blob_free"), |f| f.name.clone());
    let message = api
        .functions
        .iter()
        .find(|f| f.name.ends_with("_status_message"))
        .map_or_else(|| String::from("ts_status_message"), |f| f.name.clone());
    format!(
        r#"def _refuse(lib: TeistroLibrary, status: Status, detail: Any) -> None:
    """Raises a refusal from a call with no context to ask.

    `detail` is the owned string such a call fills when it has more to
    say than a code, which is freed here whether or not it is used.
    """
    said = "" if detail is None else _take_string(lib, detail)
    raise TeistroError(
        status, said or _text(lib.{message}(int(status)))
    )


def _take_string(lib: TeistroLibrary, raw: Any) -> str:
    """A string the library allocated, copied out and freed."""
    if not raw.data:
        return ""
    text = ctypes.string_at(raw.data, raw.len).decode("utf-8")
    lib.{string_free}(ctypes.byref(raw))
    return text


def _take_blob(lib: TeistroLibrary, raw: Any) -> bytes:
    """A result blob the library allocated, copied out and freed.

    One copy rather than none, deliberately: a decoded column is a view
    over these bytes, and a view over the library's own memory would read
    freed memory the moment the blob was let go.
    """
    if not raw.data:
        return b""
    blob = ctypes.string_at(raw.data, raw.len)
    lib.{blob_free}(ctypes.byref(raw))
    return blob
"#
    )
}

// ── The decoders ───────────────────────────────────────────────────────────

/// `_blob.py`: one decoder per result blob.
#[must_use]
pub fn decoders(api: &Api) -> String {
    let mut out = preamble(
        api,
        "The result-blob decoders: one per schema, reading the `TSRB` layout into\n`memoryview` columns over the blob's own bytes rather than copies.",
    );
    out.push_str("import struct\nfrom dataclasses import dataclass\nfrom typing import Final\n\n");
    out.push_str(BLOB_READER);
    // Shared across every blob, so a shape declared by two of them is
    // written once.
    let mut shapes = BTreeSet::new();
    for schema in &api.blobs {
        render_decoder(&mut out, schema, &mut shapes);
    }
    out
}

/// The reader every decoder shares.
const BLOB_READER: &str = r#"_MAGIC: Final = 0x42525354  # 'TSRB'
_VERSION: Final = 1
_HEADER_LENGTH: Final = 32
_ENTRY_LENGTH: Final = 16


class BlobError(ValueError):
    """Bytes that are not a blob this decoder reads."""


@dataclass(frozen=True)
class _Section:
    """One section of a blob, as its table of contents describes it."""

    offset: int
    length: int
    count: int


class _Blob:
    """A blob opened for reading: its bytes and its table of contents."""

    def __init__(self, raw: bytes, schema_id: int, schema_name: str) -> None:
        if len(raw) < _HEADER_LENGTH:
            raise BlobError("not a Teistro result blob: too short for a header")
        magic, version, count, total, identifier = struct.unpack_from(
            "<IIIII", raw, 0
        )
        if magic != _MAGIC:
            raise BlobError("not a Teistro result blob")
        if version != _VERSION:
            raise BlobError(
                f"blob layout version {version}, "
                f"this decoder reads version {_VERSION}"
            )
        if total != len(raw):
            raise BlobError(f"the header says {total} bytes, {len(raw)} were given")
        if identifier != schema_id:
            raise BlobError(
                f"blob schema {identifier}, expected {schema_id} ({schema_name})"
            )
        self.bytes = raw
        self.view = memoryview(raw)
        self.sections: dict[int, _Section] = {}
        for index in range(count):
            at = _HEADER_LENGTH + index * _ENTRY_LENGTH
            if at + _ENTRY_LENGTH > total:
                raise BlobError("the table of contents runs past the end of the blob")
            section_id, offset, length, rows = struct.unpack_from("<IIII", raw, at)
            if offset + length > total:
                raise BlobError(f"section {section_id} runs past the end of the blob")
            self.sections[section_id] = _Section(offset, length, rows)

    def section(self, section_id: int, name: str) -> _Section:
        """The section with an id, or a `BlobError` naming what is missing."""
        found = self.sections.get(section_id)
        if found is None:
            raise BlobError(f"the blob carries no `{name}` section")
        return found

    def fixed(self, at: _Section, slot: int, code: str) -> float:
        """One field of a fixed section, which gives every field a slot."""
        value = struct.unpack_from(f"<{code}", self.bytes, at.offset + slot * 8)[0]
        return float(value)

    def column(self, at: _Section, index: int, width: int, rows: int) -> memoryview:
        """The bytes of one column of a column section.

        The caller casts them to the column's own format code, which it
        knows as a literal; a format passed through here as a string
        would be one a type checker could not follow.
        """
        offset = at.offset + struct.unpack_from(
            "<I", self.bytes, at.offset + index * 4
        )[0]
        end = offset + rows * width
        if end > at.offset + at.length:
            raise BlobError("a column runs past the end of its section")
        return self.view[offset:end]

    def text(self, at: _Section) -> str:
        """A bytes section as the UTF-8 text the library wrote."""
        return self.bytes[at.offset : at.offset + at.count].decode("utf-8")


"#;

fn render_decoder(out: &mut String, schema: &BlobSchema, shapes: &mut BTreeSet<String>) {
    let name = pascal(&snake(&schema.name));
    for section in &schema.sections {
        if section.kind == SectionKind::Columns && section.shape.is_none() {
            render_column_class(out, &name, section);
        }
    }
    render_decoded_class(out, &name, schema, shapes);
    render_decode_function(out, &name, schema);
}

/// The dataclass a column section decodes into: the shape it names, so
/// two blobs carrying it share one class, or a name of the blob's own.
fn column_class(name: &str, section: &SectionSchema) -> String {
    section.shape.as_deref().map_or_else(
        || format!("{name}{}", pascal(&snake(&section.name))),
        |shape| pascal(&snake(shape)),
    )
}

fn render_column_class(out: &mut String, name: &str, section: &SectionSchema) {
    let _ = writeln!(
        out,
        "@dataclass(frozen=True)\nclass {}:",
        column_class(name, section)
    );
    let whose = if section.shape.is_some() {
        String::from("section, wherever a blob carries it")
    } else {
        format!("section of a {name} blob")
    };
    out.push_str(&docstring(
        &format!(
            "The `{}` {whose}: one column per field, each a view\nover the blob's bytes rather than a copy.\n\n{}",
            section.name, section.doc
        ),
        "    ",
    ));
    for column in &section.fields {
        let _ = writeln!(
            out,
            "\n    {}: memoryview[{}]",
            identifier(&column.name),
            python_scalar(column.scalar)
        );
        out.push_str(&docstring(&column.doc, "    "));
    }
    out.push_str("\n    length: int\n");
    out.push_str(&docstring("The number of rows every column holds.", "    "));
    out.push_str("\n\n");
}

/// A section that names a shape, as a dataclass of its own.
///
/// Two blobs carrying the same section — a chart's day and a panchanga's
/// — then share one type rather than repeating its fields in each
/// (`03-design/chart-at-the-boundary.md` §8). A section without a shape
/// stays inlined on the blob's own dataclass, as it always was.
///
/// A column section shapes the same way, into the dataclass
/// [`render_column_class`] writes; which kind a shape is follows the
/// section's own, and a shape used by two kinds is refused by the
/// schema's own check rather than emitted twice.
fn render_shape_classes(
    out: &mut String,
    blob: &str,
    schema: &BlobSchema,
    shapes: &mut BTreeSet<String>,
) {
    for section in &schema.sections {
        let Some(shape) = section.shape.as_deref() else {
            continue;
        };
        if !shapes.insert(shape.to_string()) {
            continue;
        }
        if section.kind == SectionKind::Columns {
            render_column_class(out, blob, section);
            continue;
        }
        let _ = writeln!(out, "@dataclass(frozen=True)\nclass {}:", pascal(shape));
        out.push_str(&docstring(&section.doc, "    "));
        for field in &section.fields {
            let _ = writeln!(
                out,
                "\n    {}: {}",
                identifier(&field.name),
                python_scalar(field.scalar)
            );
            out.push_str(&docstring(&field.doc, "    "));
        }
        out.push('\n');
    }
}

fn render_decoded_class(
    out: &mut String,
    name: &str,
    schema: &BlobSchema,
    shapes: &mut BTreeSet<String>,
) {
    render_shape_classes(out, name, schema, shapes);
    let _ = writeln!(out, "@dataclass(frozen=True)\nclass {name}:");
    out.push_str(&docstring(
        &format!("A decoded {name} blob.\n\n{}", schema.doc),
        "    ",
    ));
    for section in &schema.sections {
        match section.kind {
            SectionKind::Fixed if section.shape.is_some() => {
                let shape = section.shape.as_deref().unwrap_or_default();
                let _ = writeln!(
                    out,
                    "\n    {}: {}",
                    identifier(&section.name),
                    pascal(shape)
                );
                out.push_str(&docstring(&section.doc, "    "));
            }
            SectionKind::Fixed => {
                for field in &section.fields {
                    let _ = writeln!(
                        out,
                        "\n    {}: {}",
                        identifier(&field.name),
                        python_scalar(field.scalar)
                    );
                    out.push_str(&docstring(&field.doc, "    "));
                }
            }
            SectionKind::Columns => {
                let _ = writeln!(
                    out,
                    "\n    {}: {}",
                    identifier(&section.name),
                    column_class(name, section)
                );
                out.push_str(&docstring(&section.doc, "    "));
            }
            SectionKind::Bytes => {
                let _ = writeln!(out, "\n    {}: str", identifier(&section.name));
                out.push_str(&docstring(&section.doc, "    "));
            }
        }
    }
    out.push_str("\n\n");
}

/// Constructing a shaped section: one call rather than a field per
/// value, so the blob's dataclass carries one field for the shape.
fn render_shaped_construction(out: &mut String, section: &SectionSchema, at: &str) {
    let shape = section.shape.as_deref().unwrap_or_default();
    let _ = writeln!(
        out,
        "        {}={}(",
        identifier(&section.name),
        pascal(shape)
    );
    for (slot, field) in section.fields.iter().enumerate() {
        let read = format!(
            "blob.fixed({at}, {slot}, \"{}\")",
            format_code(field.scalar)
        );
        let _ = writeln!(
            out,
            "            {}={},",
            identifier(&field.name),
            if field.scalar.is_float() {
                read
            } else {
                format!("int({read})")
            }
        );
    }
    let _ = writeln!(out, "        ),");
}

fn render_decode_function(out: &mut String, name: &str, schema: &BlobSchema) {
    let _ = writeln!(
        out,
        "def decode_{}(raw: bytes) -> {name}:",
        snake(&schema.name)
    );
    out.push_str(&docstring(
        &format!(
            "Decodes a {name} blob.\n\nThe columns are views over `raw`, so the buffer must outlive the\nresult; a blob of another layout version or another schema is a\n`BlobError`."
        ),
        "    ",
    ));
    let _ = writeln!(
        out,
        "    blob = _Blob(raw, {}, \"{}\")",
        schema.id, schema.name
    );
    for section in &schema.sections {
        let _ = writeln!(
            out,
            "    at_{} = blob.section({}, \"{}\")",
            snake(&section.name),
            section.id,
            section.name
        );
    }
    let _ = writeln!(out, "    return {name}(");
    for section in &schema.sections {
        let at = format!("at_{}", snake(&section.name));
        match section.kind {
            SectionKind::Fixed if section.shape.is_some() => {
                render_shaped_construction(out, section, &at);
            }
            SectionKind::Fixed => {
                for (slot, field) in section.fields.iter().enumerate() {
                    let read = format!(
                        "blob.fixed({at}, {slot}, \"{}\")",
                        format_code(field.scalar)
                    );
                    let _ = writeln!(
                        out,
                        "        {}={},",
                        identifier(&field.name),
                        if field.scalar.is_float() {
                            read
                        } else {
                            format!("int({read})")
                        }
                    );
                }
            }
            SectionKind::Columns => {
                let _ = writeln!(
                    out,
                    "        {}={}(",
                    identifier(&section.name),
                    column_class(name, section)
                );
                for (index, column) in section.fields.iter().enumerate() {
                    let read = format!(
                        "{}=blob.column({at}, {index}, {}, {at}.count).cast(\"{}\"),",
                        identifier(&column.name),
                        column.scalar.width(8),
                        format_code(column.scalar)
                    );
                    // One line where it fits, and the argument on its own
                    // where it does not, so a long column name does not
                    // push the generated file past a Python reader's width.
                    if read.len() + 12 <= 79 {
                        let _ = writeln!(out, "            {read}");
                    } else {
                        let _ = writeln!(
                            out,
                            "            {}=blob.column(\n                {at}, {index}, {}, {at}.count\n            ).cast(\"{}\"),",
                            identifier(&column.name),
                            column.scalar.width(8),
                            format_code(column.scalar)
                        );
                    }
                }
                let _ = writeln!(out, "            length={at}.count,\n        ),");
            }
            SectionKind::Bytes => {
                let _ = writeln!(
                    out,
                    "        {}=blob.text({at}),",
                    identifier(&section.name)
                );
            }
        }
    }
    out.push_str("    )\n\n\n");
}

#[cfg(test)]
mod tests {
    use super::{ctypes_scalar, format_code, identifier, member, python_scalar, struct_name};
    use crate::model::{EnumValue, Scalar};

    fn value(name: &str, key: Option<&str>) -> EnumValue {
        EnumValue {
            name: name.to_string(),
            value: 0,
            doc: String::new(),
            key: key.map(str::to_string),
            deprecated: false,
        }
    }

    #[test]
    fn a_keyword_gets_pep_8s_trailing_underscore() {
        assert_eq!(identifier("from"), "from_");
        assert_eq!(identifier("jd_ut1"), "jd_ut1");
        assert_eq!(identifier("longitudeDeg"), "longitude_deg");
        // A member is its key upper-cased, and no Python keyword is, so
        // the member rule never fires where Dart's does.
        assert_eq!(member(&value("Return", Some("RETURN"))), "RETURN");
        assert_eq!(member(&value("None", Some("NONE"))), "NONE");
        assert_eq!(member(&value("InvalidArg", None)), "INVALID_ARG");
    }

    #[test]
    fn every_scalar_has_a_fixed_width_spelling_and_a_format_code() {
        for scalar in Scalar::ALL {
            let name = ctypes_scalar(scalar);
            assert!(name.starts_with("ctypes.c_"), "{name}");
            // `c_long` and `c_int` change width with the platform, and a
            // generated binding exists to make that impossible.
            assert!(
                !matches!(name, "ctypes.c_long" | "ctypes.c_int" | "ctypes.c_ulong"),
                "{name} is not fixed-width"
            );
            assert_eq!(format_code(scalar).len(), 1);
        }
        assert_eq!(python_scalar(Scalar::F64), "float");
        assert_eq!(python_scalar(Scalar::Bool), "bool");
        assert_eq!(python_scalar(Scalar::U16), "int");
        assert_eq!(struct_name("PositionRequestC"), "_PositionRequestStruct");
    }
}
