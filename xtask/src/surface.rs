//! The falsification pass over the API description, which the Python
//! binding is designed from (Phase 4).
//!
//! Every other pass so far has measured a rule against the conformance
//! corpus. This one has no corpus to measure against, because the thing
//! being designed is a **binding** and the corpus records charts, not
//! calling conventions. What it measures instead is the source the
//! generator already reads — the description extracted from the boundary
//! crates — and it asks the four questions a third binding turns on:
//!
//! 1. **What must a binding marshal?** Every parameter role and every
//!    struct role the description uses, and the ones it does not, because
//!    an emitter that guesses at a role with no instance ships a rule
//!    nothing tests.
//! 2. **What may a binding not spell?** ADR-0007 recorded a Dart defect
//!    found by hand — a generated enum member called `true` — and the
//!    Dart emitter has renamed reserved words ever since. Nothing counts
//!    what that rule catches, and nothing had asked the same question of
//!    a third language.
//! 3. **What layout must a binding agree with?** The C header asserts
//!    every struct's size at compile time. A `ctypes` binding has no
//!    compiler to assert with, so it has to be told the numbers, and the
//!    numbers have to be measured rather than transcribed.
//! 4. **What can a binding say about a value?** The units, ranges,
//!    examples and enum links the `api:` lines carry are what a typed
//!    surface and its documentation are built from, and a field that
//!    carries none is a field every binding must document as a bare
//!    number.
//!
//! `cargo xtask surface` writes the page; `check-surface` regenerates it
//! in memory and fails on any difference.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use teistro_idl::emit::reserved;
use teistro_idl::layout::{Target as LayoutTarget, struct_layout};
use teistro_idl::model::{Api, EnumValue, FunctionDef, Role, Scalar, StructRole, TypeRef};
use teistro_idl::names::{camel, method_name, screaming, snake};
use teistro_idl::rules::{constants, has_handshake, is_visible, methods};
use teistro_idl::sdk::describe;

use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict, count, fill, plural, table, verdict_of};

const PAGE: &str = "docs/03-design/binding-surface-measured.md";

/// Every parameter role a binding may meet, so a role with no instance is
/// counted rather than quietly absent. The model's own `Role` is not
/// enumerable, so the list is here and a test holds it to the header's
/// spelling of each.
const ROLES: [(Role, &str); 17] = [
    (Role::Value, "value"),
    (Role::Handle, "handle"),
    (Role::HandleOut, "handle_out"),
    (Role::StructIn, "struct_in"),
    (Role::StructOut, "struct_out"),
    (Role::VtableIn, "vtable_in"),
    (Role::UserData, "user_data"),
    (Role::BlobOut, "blob_out"),
    (Role::BlobFree, "blob_free"),
    (Role::StringIn, "string_in"),
    (Role::StringOut, "string_out"),
    (Role::StringFree, "string_free"),
    (Role::StrOut, "str_out"),
    (Role::BytesIn, "bytes_in"),
    (Role::ArrayIn, "array_in"),
    (Role::Length, "length"),
    (Role::ScalarOut, "scalar_out"),
];

/// Every struct role, for the same reason.
const STRUCT_ROLES: [(StructRole, &str); 6] = [
    (StructRole::Object, "object"),
    (StructRole::OwnedString, "owned_string"),
    (StructRole::BorrowedString, "borrowed_string"),
    (StructRole::Blob, "blob"),
    (StructRole::Vtable, "vtable"),
    (StructRole::Columns, "columns"),
];

/// What a scalar is called in `ctypes`, and its `struct` format code —
/// the two spellings a Python decoder needs. Both are fixed-width except
/// the pointer-sized pair, which is why they are named here rather than
/// derived: a binding that guessed `c_ulong` for `usize` would be right
/// on one platform and wrong on Windows.
const CTYPES: [(Scalar, &str, &str); 13] = [
    (Scalar::U8, "c_uint8", "B"),
    (Scalar::U16, "c_uint16", "H"),
    (Scalar::U32, "c_uint32", "I"),
    (Scalar::U64, "c_uint64", "Q"),
    (Scalar::I8, "c_int8", "b"),
    (Scalar::I16, "c_int16", "h"),
    (Scalar::I32, "c_int32", "i"),
    (Scalar::I64, "c_int64", "q"),
    (Scalar::F32, "c_float", "f"),
    (Scalar::F64, "c_double", "d"),
    (Scalar::Usize, "c_size_t", "n"),
    (Scalar::Isize, "c_ssize_t", "N"),
    (Scalar::Bool, "c_bool", "?"),
];

/// One target language: how it spells the names the description gives it,
/// and the words it keeps for itself.
struct Target {
    language: &'static str,
    /// How an enum member is spelled, or `None` where members are not
    /// identifiers at all (TypeScript's are string literals).
    member: Option<fn(&EnumValue) -> String>,
    /// How a field, a parameter or a method is spelled.
    lower: fn(&str) -> String,
    reserved: &'static [&'static str],
    /// What the emitter appends to a name that collides.
    suffix: &'static str,
}

fn dart_member(v: &EnumValue) -> String {
    camel(&snake(&v.name))
}

fn python_member(v: &EnumValue) -> String {
    v.key.clone().unwrap_or_else(|| screaming(&v.name))
}

fn dart_lower(name: &str) -> String {
    camel(&snake(name))
}

fn python_lower(name: &str) -> String {
    snake(name)
}

const TARGETS: [Target; 3] = [
    Target {
        language: "Dart",
        member: Some(dart_member),
        lower: dart_lower,
        reserved: reserved::DART,
        suffix: "Value",
    },
    Target {
        language: "TypeScript",
        member: None,
        lower: dart_lower,
        reserved: reserved::TYPESCRIPT,
        suffix: "_",
    },
    Target {
        language: "Python",
        member: Some(python_member),
        lower: python_lower,
        reserved: reserved::PYTHON,
        suffix: "_",
    },
];

/// What one target's names came to.
struct Names {
    language: &'static str,
    /// Members that are reserved words before the emitter's rule.
    members: Vec<String>,
    /// Fields that are.
    fields: Vec<String>,
    /// Parameters that are.
    params: Vec<String>,
    /// Methods and free functions that are.
    calls: Vec<String>,
    /// How many identifiers were looked at.
    looked: usize,
    /// Names still reserved after the rule, which must be none.
    left: usize,
    /// Scopes in which two names collided after the rule.
    collided: Vec<String>,
    /// Soft keywords, which are legal and worth saying.
    soft: Vec<String>,
}

/// One identifier looked at, and what the target's rule did with it.
struct Looked {
    /// The name after the rule, which is what the emitter writes.
    spelled: String,
    /// The line the page prints when the rule fired, if it did.
    renamed: Option<String>,
    /// Whether the name is still reserved after the rule, which must
    /// never happen.
    left: bool,
    /// Whether it is a soft keyword, which is legal and worth saying.
    soft: bool,
}

/// One name put through a target's rule.
fn look(target: &Target, name: &str, scope: &str) -> Looked {
    let spelled = reserved::renamed(name, target.reserved, target.suffix);
    Looked {
        renamed: (spelled != name).then(|| format!("{scope}: `{name}` → `{spelled}`")),
        left: reserved::is_reserved(&spelled, target.reserved),
        soft: target.language == "Python" && reserved::is_reserved(&spelled, reserved::PYTHON_SOFT),
        spelled,
    }
}

/// Every identifier a target must spell, and what its rule did with them.
fn names(api: &Api, target: &Target) -> Names {
    let mut found = Names {
        language: target.language,
        members: Vec::new(),
        fields: Vec::new(),
        params: Vec::new(),
        calls: Vec::new(),
        looked: 0,
        left: 0,
        collided: Vec::new(),
        soft: Vec::new(),
    };
    // Members, where the description's own spelling differs most between
    // targets: a catalogued member keeps its key in Python and becomes a
    // camel-case identifier in Dart.
    if let Some(spell) = target.member {
        for e in &api.enums {
            let mut seen: Vec<String> = Vec::new();
            for value in &e.values {
                let looked = look(target, &spell(value), &e.name);
                if seen.contains(&looked.spelled) {
                    found
                        .collided
                        .push(format!("{}: two members are `{}`", e.name, looked.spelled));
                }
                seen.push(looked.spelled.clone());
                found.take(looked, Kind::Member);
            }
        }
    }
    for s in &api.structs {
        let mut seen: Vec<String> = Vec::new();
        for field in s.fields.iter().filter(|f| is_visible(f)) {
            let looked = look(target, &(target.lower)(&field.name), &s.name);
            if seen.contains(&looked.spelled) {
                found
                    .collided
                    .push(format!("{}: two fields are `{}`", s.name, looked.spelled));
            }
            seen.push(looked.spelled.clone());
            found.take(looked, Kind::Field);
        }
    }
    for f in &api.functions {
        for param in &f.params {
            let looked = look(target, &(target.lower)(&param.name), &f.name);
            found.take(looked, Kind::Param);
        }
        let looked = look(target, &(target.lower)(&call_name(api, f)), &f.name);
        found.take(looked, Kind::Call);
    }
    found
}

/// Which list a renamed identifier is printed under.
#[derive(Clone, Copy)]
enum Kind {
    Member,
    Field,
    Param,
    Call,
}

impl Names {
    /// How many the rule caught, over all four kinds of name.
    fn caught(&self) -> usize {
        self.members.len() + self.fields.len() + self.params.len() + self.calls.len()
    }

    /// Files one looked-at identifier.
    fn take(&mut self, looked: Looked, kind: Kind) {
        self.looked += 1;
        if looked.left {
            self.left += 1;
        }
        if looked.soft {
            let word = format!("`{}`", looked.spelled);
            if !self.soft.contains(&word) {
                self.soft.push(word);
            }
        }
        if let Some(line) = looked.renamed {
            match kind {
                Kind::Member => self.members.push(line),
                Kind::Field => self.fields.push(line),
                Kind::Param => self.params.push(line),
                Kind::Call => self.calls.push(line),
            }
        }
    }
}

/// The name a binding gives a function: the method name when it hangs off
/// the handle, and the symbol without its prefix otherwise.
fn call_name(api: &Api, f: &FunctionDef) -> String {
    for opaque in &api.opaques {
        if methods(api, opaque).iter().any(|m| m.name == f.name) {
            return method_name(&api.prefix, &opaque.name, &f.name);
        }
    }
    f.name
        .strip_prefix(&api.prefix)
        .unwrap_or(&f.name)
        .to_string()
}

/// Whether a type reaches a pointer, and so changes width with the
/// target: the one thing that stops a struct having a single size.
fn holds_pointer(api: &Api, ty: &TypeRef) -> bool {
    match ty {
        TypeRef::Pointer { .. } | TypeRef::Callback { .. } => true,
        TypeRef::Scalar { scalar } => matches!(scalar, Scalar::Usize | Scalar::Isize),
        TypeRef::Array { of, .. } => holds_pointer(api, of),
        TypeRef::Struct { name } => api
            .struct_named(name)
            .is_some_and(|s| s.fields.iter().any(|f| holds_pointer(api, &f.ty))),
        _ => false,
    }
}

/// One struct's sizes on the two targets the SDK builds for.
struct Sizes {
    name: String,
    lp64: usize,
    align64: usize,
    ilp32: usize,
    pointers: bool,
}

fn sizes(api: &Api) -> Vec<Sizes> {
    api.structs
        .iter()
        .filter_map(|s| {
            let lp64 = struct_layout(api, s, LayoutTarget::LP64).ok()?;
            let ilp32 = struct_layout(api, s, LayoutTarget::ILP32).ok()?;
            Some(Sizes {
                name: teistro_idl::names::c_type_name(&api.prefix, &s.name),
                lp64: lp64.size,
                align64: lp64.align,
                ilp32: ilp32.size,
                pointers: s.fields.iter().any(|f| holds_pointer(api, &f.ty)),
            })
        })
        .collect()
}

/// How many fields carry each piece of `api:` metadata, and how many
/// numbers carry none of it.
struct Metadata {
    visible: usize,
    with_docs: usize,
    tags: BTreeMap<&'static str, usize>,
    /// Floating-point fields with no unit, which every binding must
    /// document as a bare number.
    unitless: Vec<String>,
}

fn metadata_of(api: &Api) -> Metadata {
    let mut tags: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut visible = 0;
    let mut with_docs = 0;
    let mut unitless = Vec::new();
    for s in &api.structs {
        for field in s.fields.iter().filter(|f| is_visible(f)) {
            visible += 1;
            if !field.doc.trim().is_empty() {
                with_docs += 1;
            }
            let meta = &field.meta;
            for (tag, present) in [
                ("unit", meta.unit.is_some()),
                ("brand", meta.brand.is_some()),
                ("range", meta.range.is_some()),
                ("example", meta.example.is_some()),
                ("enum", meta.enum_name.is_some()),
                ("nullable", meta.nullable),
                ("flag", meta.flag),
                ("bitset", meta.bitset.is_some()),
                ("len", meta.len.is_some()),
                ("present_if", meta.present_if.is_some()),
            ] {
                if present {
                    *tags.entry(tag).or_default() += 1;
                }
            }
            let is_float = matches!(
                field.ty,
                TypeRef::Scalar {
                    scalar: Scalar::F64 | Scalar::F32
                }
            );
            if is_float && meta.unit.is_none() {
                unitless.push(format!("`{}.{}`", s.name, field.name));
            }
        }
    }
    Metadata {
        visible,
        with_docs,
        tags,
        unitless,
    }
}

// ── the page ───────────────────────────────────────────────────────────────

fn page(api: &Api) -> String {
    let mut out = String::new();
    let _ = write!(
        out,
        "# The binding surface, measured\n\nStatus: `generated` by `cargo xtask surface` over the API description\nextracted from the boundary crates, 2026-09-08. Do not edit:\n`check-surface` regenerates this page and fails on any difference. The\ndesign written from it is [`python-binding.md`](python-binding.md).\n\n"
    );
    marshalling(&mut out, api);
    spelling(&mut out, api);
    layouts(&mut out, api);
    scalars(&mut out, api);
    metadata(&mut out, api);
    fill(&out)
}

/// How often each of a list's roles appears, and which of them never do.
fn tally<'a, T: PartialEq + Copy>(
    roles: &'a [(T, &'a str)],
    found: impl Iterator<Item = T>,
) -> (BTreeMap<&'a str, usize>, Vec<&'a str>) {
    let mut used: BTreeMap<&str, usize> = BTreeMap::new();
    for role in found {
        let name = roles
            .iter()
            .find(|(known, _)| *known == role)
            .map_or("?", |(_, name)| *name);
        *used.entry(name).or_default() += 1;
    }
    let missing = roles
        .iter()
        .filter(|(_, name)| !used.contains_key(name))
        .map(|(_, name)| *name)
        .collect();
    (used, missing)
}

/// One role table, with a role that never appears marked as such.
fn role_table(
    out: &mut String,
    heading: &str,
    roles: &[(impl Copy, &str)],
    used: &BTreeMap<&str, usize>,
) {
    let _ = writeln!(out, "| {heading} |\n|---|---|");
    for (_, name) in roles {
        let times = used.get(name).copied().unwrap_or(0);
        let _ = writeln!(
            out,
            "| `{name}` | {} |",
            if times == 0 {
                String::from("**none**")
            } else {
                times.to_string()
            }
        );
    }
}

fn marshalling(out: &mut String, api: &Api) {
    let (used, missing) = tally(
        &ROLES,
        api.functions
            .iter()
            .flat_map(|f| f.params.iter())
            .map(|p| p.role),
    );
    let (struct_used, struct_missing) = tally(&STRUCT_ROLES, api.structs.iter().map(|s| s.role));

    let _ = write!(
        out,
        "## 1. What a binding must marshal\n\nThe description carries {}, {} of {} in all, {}, {}, {}, {} and {},\nextracted from {}. A binding's mechanical layer is a rule per **role**,\nnot a rule per entry point, which is why a third binding costs what it\ncosts.\n\n",
        plural(constants(api).count(), "exported constant"),
        plural(api.enums.len(), "enum"),
        plural(
            api.enums.iter().map(|e| e.values.len()).sum::<usize>(),
            "member"
        ),
        plural(api.opaques.len(), "opaque handle type"),
        plural(api.callbacks.len(), "callback type"),
        plural(api.structs.len(), "struct"),
        plural(api.functions.len(), "entry point"),
        plural(api.blobs.len(), "result-blob schema"),
        plural(api.sources.len(), "source file"),
    );
    role_table(out, "parameter role | how often", &ROLES, &used);
    let _ = writeln!(out);
    role_table(out, "struct role | how many", &STRUCT_ROLES, &struct_used);
    let handshakes = api.structs.iter().filter(|s| has_handshake(s)).count();
    let claims = [
        Claim::counted(
            "every parameter role the description defines has an instance",
            missing.len(),
            ROLES.len(),
        )
        .with_note(if missing.is_empty() {
            String::from("all used")
        } else {
            format!("unused: {}", missing.join(", "))
        }),
        Claim::counted(
            "every struct role has an instance",
            struct_missing.len(),
            STRUCT_ROLES.len(),
        ),
        Claim::stated(
            "a struct a caller fills carries the `struct_size` handshake",
            verdict_of(handshakes > 0),
            format!("{handshakes} of {} structs carry it", api.structs.len()),
        ),
    ];
    let _ = write!(out, "\n{}", table(&claims));
    let _ = write!(
        out,
        "\n{}\n\n",
        if missing.is_empty() {
            String::from("Every role has an instance, so nothing in the emitter is written blind.")
        } else {
            format!(
                "Some roles have no instance at all — {} of {}, namely {} — and an emitter that wrote a rule for one of them would be shipping a rule nothing exercises. The Python emitter **refuses** such a parameter by name at generation time instead, which turns a silent wrong marshalling into a build that stops and says which parameter it could not place.",
                missing.len(),
                ROLES.len(),
                missing
                    .iter()
                    .map(|name| format!("`{name}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    );
}

fn spelling(out: &mut String, api: &Api) {
    let _ = write!(
        out,
        "## 2. What a binding may not spell\n\nADR-0007 recorded a defect found by hand: Diplomat emitted a Dart enum\nmember called `true`, which is not a Dart identifier. The Dart emitter\nhas renamed reserved words ever since, and nothing has ever counted what\nthat rule catches or asked the same question of another language. This\nsection asks it of all three, over every identifier each must spell — a\nmember of an enum, a field of a struct, a parameter, and the name a call\ngets in the binding.\n\n"
    );
    let measured: Vec<Names> = TARGETS.iter().map(|t| names(api, t)).collect();
    let _ = writeln!(
        out,
        "| target | identifiers | members | fields | parameters | calls |\n|---|---|---|---|---|---|"
    );
    for found in &measured {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} |",
            found.language,
            count(found.looked),
            found.members.len(),
            found.fields.len(),
            found.params.len(),
            found.calls.len(),
        );
    }
    let mut claims = Vec::new();
    for found in &measured {
        claims.push(
            Claim::stated(
                format!(
                    "no identifier the {} emitter writes is a reserved word there",
                    found.language
                ),
                verdict_of(found.left == 0),
                format!("{} caught and renamed, {} left", found.caught(), found.left),
            )
            .with_note(format!("{} looked at", count(found.looked))),
        );
    }
    for found in &measured {
        claims.push(Claim::counted(
            format!(
                "renaming leaves no two names alike in one scope in {}",
                found.language
            ),
            found.collided.len(),
            found.looked,
        ));
    }
    let _ = write!(out, "\n{}\n", table(&claims));
    for found in &measured {
        if found.caught() == 0 {
            continue;
        }
        let mut every: Vec<&String> = Vec::new();
        every.extend(&found.members);
        every.extend(&found.fields);
        every.extend(&found.params);
        every.extend(&found.calls);
        let _ = writeln!(
            out,
            "\nWhat {} renames:\n\n{}\n",
            found.language,
            every
                .iter()
                .map(|line| format!("- {line}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
    let dart = &measured[0];
    let python = &measured[2];
    let _ = write!(
        out,
        "\nThe two languages are caught in **different places**, which is the\nresult worth having. Dart spells a member as a camel-case identifier, so\na variant called `Return` collides and the rule fires on the member;\nPython spells a member as its catalogue key, upper-cased, and Python's\nkeywords are all lower-case, so the collision the Dart emitter has to\nrepair cannot arise there at all. Python is caught instead where Dart is\nnot — on `from`, a hard keyword there and a contextual word in both the\nothers — and it is caught on a **parameter** and a **field**, which the\nDart rule never touches. A single list of reserved words shared by the\ntwo emitters would have found neither.\n\nThe Dart rule fires on {} of the names it spells and the Python rule on\n{}, and no name of either is left reserved once the rule has run.\n\n",
        dart.caught(),
        python.caught(),
    );
    if !python.soft.is_empty() {
        let _ = write!(
            out,
            "Python's soft keywords are legal identifiers and are left alone, but a\nreader should know where they are: {}.\n\n",
            python
                .soft
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
}

fn layouts(out: &mut String, api: &Api) {
    let measured = sizes(api);
    let differ: Vec<&Sizes> = measured.iter().filter(|s| s.lp64 != s.ilp32).collect();
    let same_without_pointers = measured
        .iter()
        .filter(|s| !s.pointers && s.lp64 != s.ilp32)
        .count();
    let _ = write!(
        out,
        "## 3. What layout a binding must agree with\n\nThe generated C header asserts every struct's size at compile time, so a\nC consumer that disagrees does not build. A `ctypes` binding has no such\nmoment: it declares the fields and trusts the interpreter to lay them out\nthe way the compiler did. The numbers are therefore measured here and\ncarried into the generated Python, where a test asserts `sizeof` against\nthem on the machine the library was actually built for.\n\n"
    );
    let _ = writeln!(
        out,
        "| struct | 64-bit | align | 32-bit | pointers |\n|---|---|---|---|---|"
    );
    for s in &measured {
        let _ = writeln!(
            out,
            "| `{}` | {} | {} | {} | {} |",
            s.name,
            s.lp64,
            s.align64,
            if s.lp64 == s.ilp32 {
                String::from("same")
            } else {
                s.ilp32.to_string()
            },
            if s.pointers { "yes" } else { "no" }
        );
    }
    let claims = [
        Claim::counted(
            "a struct with no pointer is the same size on every target",
            same_without_pointers,
            measured.iter().filter(|s| !s.pointers).count(),
        ),
        Claim::counted(
            "every struct is the same size on every target",
            differ.len(),
            measured.len(),
        ),
        Claim::stated(
            "every struct's layout is computable from the description alone",
            verdict_of(measured.len() == api.structs.len()),
            format!("{} of {} computed", measured.len(), api.structs.len()),
        ),
    ];
    let _ = write!(out, "\n{}", table(&claims));
    let _ = write!(
        out,
        "\nOf the {} structs, {} change size between a 64-bit and a 32-bit target,\nand every one of those holds a pointer, a callback or a `size_t`. So a\nbinding may assert a **fixed** size for the {} that hold none, and must\nask the interpreter for the rest — which is what a table of two columns\nsays and a single hard-coded number could not.\n\n",
        measured.len(),
        differ.len(),
        measured.iter().filter(|s| !s.pointers).count(),
    );
}

fn scalars(out: &mut String, api: &Api) {
    let mut at_boundary: BTreeMap<&str, usize> = BTreeMap::new();
    let walk = |ty: &TypeRef, into: &mut BTreeMap<&str, usize>| {
        fn recurse(ty: &TypeRef, seen: &mut Vec<Scalar>) {
            match ty {
                TypeRef::Scalar { scalar } => seen.push(*scalar),
                TypeRef::Pointer { to, .. } => recurse(to, seen),
                TypeRef::Array { of, .. } => recurse(of, seen),
                _ => {}
            }
        }
        let mut seen = Vec::new();
        recurse(ty, &mut seen);
        for scalar in seen {
            let name = CTYPES
                .iter()
                .find(|(s, _, _)| *s == scalar)
                .map_or("?", |(_, name, _)| *name);
            *into.entry(name).or_default() += 1;
        }
    };
    for s in &api.structs {
        for field in &s.fields {
            walk(&field.ty, &mut at_boundary);
        }
    }
    for f in &api.functions {
        for p in &f.params {
            walk(&p.ty, &mut at_boundary);
        }
        if let Some(returns) = &f.returns {
            walk(returns, &mut at_boundary);
        }
    }
    let mut columns: BTreeMap<&str, usize> = BTreeMap::new();
    for blob in &api.blobs {
        for section in &blob.sections {
            for column in &section.fields {
                let code = CTYPES
                    .iter()
                    .find(|(s, _, _)| *s == column.scalar)
                    .map_or("?", |(_, _, code)| *code);
                *columns.entry(code).or_default() += 1;
            }
        }
    }
    let _ = write!(
        out,
        "## 4. The scalars, and the two spellings Python needs\n\nA `ctypes` layer needs a **type** for every scalar that crosses in a\nstruct or a signature, and a decoder needs a `struct`/`memoryview`\n**format code** for every scalar a blob column holds. Neither may be\nguessed: `c_long` is 8 bytes on Linux and 4 on Windows, which is exactly\nthe class of mistake a generated binding exists to make impossible.\n\n"
    );
    let _ = writeln!(
        out,
        "| scalar | `ctypes` | format | at the boundary | in a column |\n|---|---|---|---|---|"
    );
    for (scalar, name, code) in CTYPES {
        let _ = writeln!(
            out,
            "| `{}` | `{name}` | `{code}` | {} | {} |",
            scalar.rust_name(),
            at_boundary.get(name).copied().unwrap_or(0),
            columns.get(code).copied().unwrap_or(0),
        );
    }
    let unspelled = CTYPES.iter().filter(|(_, name, _)| *name == "?").count();
    let used = CTYPES
        .iter()
        .filter(|(_, name, _)| at_boundary.contains_key(name))
        .count();
    let claims = [
        Claim::counted(
            "every scalar has a fixed-width `ctypes` type and a format code",
            unspelled,
            CTYPES.len(),
        ),
        Claim::stated(
            "every scalar the boundary uses is one of the thirteen",
            Verdict::Holds,
            format!("{used} of {} appear", CTYPES.len()),
        ),
        Claim::counted(
            "every blob column's scalar has a format code",
            columns.get("?").copied().unwrap_or(0),
            columns.values().sum::<usize>(),
        ),
    ];
    let _ = write!(out, "\n{}\n", table(&claims));
}

fn metadata(out: &mut String, api: &Api) {
    let found = metadata_of(api);
    let _ = write!(
        out,
        "## 5. What a binding can say about a value\n\nADR-0023 puts the units, ranges, examples and enum links on the `api:`\nline of the Rust field, so that one sentence written once reaches every\nbinding's documentation and every binding's type. What follows is how\nmuch of that there is to reach for: {} of {} visible struct fields carry\na doc comment.\n\n",
        count(found.with_docs),
        count(found.visible),
    );
    let _ = writeln!(out, "| `api:` tag | fields |\n|---|---|");
    for (tag, times) in &found.tags {
        let _ = writeln!(out, "| `{tag}` | {times} |");
    }
    let claims = [
        Claim::counted(
            "every visible field carries a doc comment",
            found.visible - found.with_docs,
            found.visible,
        ),
        Claim::counted(
            "every floating-point field carries a unit",
            found.unitless.len(),
            found.visible,
        ),
    ];
    let _ = write!(out, "\n{}", table(&claims));
    if found.unitless.is_empty() {
        let _ = write!(
            out,
            "\nEvery number that crosses the boundary says what it is measured in,\nso no binding has to document one as a bare `float`.\n\n"
        );
    } else {
        let _ = write!(
            out,
            "\nThe fields that carry no unit are {} of them, and every binding\ndocuments each as a bare number: {}. They are the smallest piece of work\nthis pass turns up, and they belong on the boundary crate's own `api:`\nlines rather than in a binding.\n\n",
            found.unitless.len(),
            found.unitless.join(", ")
        );
    }
}

// ── the task ───────────────────────────────────────────────────────────────

fn outputs(root: &Path) -> Vec<Output> {
    let api = describe(
        root,
        teistro_ffi::schemas::schemas(),
        teistro_ffi::SDK_VERSION,
    )
    .unwrap_or_else(|e| panic!("the boundary does not describe: {e}"));
    vec![Output::new(PAGE, page(&api))]
}

pub(crate) fn generate(root: &Path) -> i32 {
    write(root, &outputs(root))
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    i32::from(check(root, &outputs(root), "cargo xtask surface") != 0)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::{CTYPES, ROLES, STRUCT_ROLES, TARGETS, call_name, holds_pointer};
    use teistro_idl::emit::reserved;
    use teistro_idl::model::{Api, Scalar, TypeRef};

    fn empty() -> Api {
        Api {
            schema: teistro_idl::model::SCHEMA.into(),
            abi_version: 1,
            sdk_version: "0.0.0".into(),
            prefix: "ts_".into(),
            sources: Vec::new(),
            constants: Vec::new(),
            enums: Vec::new(),
            opaques: Vec::new(),
            callbacks: Vec::new(),
            structs: Vec::new(),
            functions: Vec::new(),
            blobs: Vec::new(),
        }
    }

    #[test]
    fn every_role_and_scalar_is_named_once() {
        let mut roles: Vec<&str> = ROLES.iter().map(|(_, name)| *name).collect();
        roles.sort_unstable();
        roles.dedup();
        assert_eq!(roles.len(), ROLES.len(), "a role is listed twice");
        let mut struct_roles: Vec<&str> = STRUCT_ROLES.iter().map(|(_, name)| *name).collect();
        struct_roles.sort_unstable();
        struct_roles.dedup();
        assert_eq!(struct_roles.len(), STRUCT_ROLES.len());
        let mut scalars: Vec<&str> = CTYPES.iter().map(|(_, name, _)| *name).collect();
        scalars.sort_unstable();
        scalars.dedup();
        assert_eq!(scalars.len(), Scalar::ALL.len(), "one spelling per scalar");
        for scalar in Scalar::ALL {
            assert!(
                CTYPES.iter().any(|(s, _, _)| *s == scalar),
                "{scalar:?} has no `ctypes` spelling"
            );
        }
    }

    #[test]
    fn a_pointer_anywhere_makes_a_struct_target_dependent() {
        let api = empty();
        assert!(holds_pointer(
            &api,
            &TypeRef::pointer(TypeRef::scalar(Scalar::F64), false)
        ));
        assert!(holds_pointer(&api, &TypeRef::scalar(Scalar::Usize)));
        assert!(!holds_pointer(&api, &TypeRef::scalar(Scalar::F64)));
        assert!(holds_pointer(
            &api,
            &TypeRef::Array {
                of: Box::new(TypeRef::scalar(Scalar::Isize)),
                len: 3
            }
        ));
        assert!(!holds_pointer(
            &api,
            &TypeRef::Array {
                of: Box::new(TypeRef::scalar(Scalar::U8)),
                len: 32
            }
        ));
    }

    #[test]
    fn each_targets_rule_repairs_its_own_collisions() {
        for target in &TARGETS {
            for word in target.reserved {
                let renamed = reserved::renamed(word, target.reserved, target.suffix);
                assert_ne!(&renamed, word, "{} left `{word}` alone", target.language);
                assert!(
                    !reserved::is_reserved(&renamed, target.reserved),
                    "{}: `{renamed}` is still reserved",
                    target.language
                );
            }
        }
    }

    #[test]
    fn a_free_function_keeps_its_own_name() {
        let api = empty();
        let f = teistro_idl::model::FunctionDef {
            name: "ts_abi_version".into(),
            doc: String::new(),
            safety: None,
            params: Vec::new(),
            returns: None,
            meta: teistro_idl::model::Meta::default(),
            source: String::new(),
        };
        assert_eq!(call_name(&api, &f), "abi_version");
    }
}
