//! The JavaScript glue: a Rust module rendered from the description, so a
//! new entry point in the boundary crate reaches JavaScript by running the
//! generator and nothing else (ADR-0007). Two [`Backend`]s render it: napi
//! for the Node addon and wasm-bindgen for the wasm package
//! (`03-design/wasm-binding.md` §3). The marshalling is emitted once and
//! the backend supplies only the spellings that differ, so both expose
//! **the same `native` object**, member for member, and one `index.js`
//! serves both.
//!
//! Every decision is driven by a parameter's or a field's role, never by a
//! function's name: a handle becomes the class, a struct in an object the
//! glue reads, a struct out an object it writes, a blob a `Buffer`, an
//! owned string a `String`, a lent string a copy taken before the next
//! call frees it, a scalar out a returned number. Enums cross as the
//! strings `catalogue.js` names, so the addon and the typed surface agree
//! without a table in two places.
//!
//! An object that lends the C struct a buffer (an array, a string) reads
//! into a `Held*` value that owns those buffers; the C struct borrows them
//! and never outlives the call. The glue holds the `unsafe` calls, each
//! with a SAFETY comment; the layer above it is hand-written JavaScript.

use std::collections::BTreeSet;
use std::fmt::Write;

use crate::emit::{DocStyle, field_doc_with, line_comment};
use crate::model::{
    Api, EnumDef, FieldDef, FunctionDef, OpaqueDef, Role, Scalar, StructDef, StructRole, TypeRef,
};
use crate::names::{binding_type_name, camel, pascal, snake};
use crate::rules::{
    FieldRole, Handed, constructor, destructor, field_roles, has_handshake, method_name, methods,
    pointee_opaque, pointee_struct, results, returns_status, status_enum,
};

use super::ts::{UNKNOWN_MEMBER, member_value};

/// The crate a boundary item lives in, as the addon names it. A source
/// under `crates/<name>/src/<module>.rs` is `<alias>::<module>::<item>`.
const CRATES: [(&str, &str); 3] = [
    ("ffi", "ffi"),
    ("port-ephemeris", "port"),
    ("core", "core_"),
];

/// The toolkit the glue is written for. Everything that differs between
/// the two is a method here, so the table in `03-design/wasm-binding.md`
/// §3 is this `impl` and nowhere else.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    /// napi-rs, for the Node addon (`bindings/node/native`).
    Napi,
    /// wasm-bindgen, for the wasm package (`bindings/wasm/native`).
    ///
    /// Plain objects cross through `serde-wasm-bindgen` with camel-cased
    /// fields, as napi's `#[napi(object)]` crosses them; a 64-bit integer
    /// crosses as a number, as napi's does, where wasm-bindgen's own
    /// mapping would make it a `BigInt`; and a refusal is an `Error` with
    /// the call's record as `lastError`, as napi's is.
    WasmBindgen,
}

impl Backend {
    /// The `use` lines naming the toolkit.
    fn prelude(self) -> &'static str {
        match self {
            Backend::Napi => "use napi::bindgen_prelude::*;\nuse napi_derive::napi;\n",
            Backend::WasmBindgen => "use wasm_bindgen::prelude::*;\n",
        }
    }

    /// The Rust type bytes cross as: a `Buffer`, or the `Uint8Array`
    /// wasm-bindgen makes of a `Vec<u8>`.
    fn bytes(self) -> &'static str {
        match self {
            Backend::Napi => "Buffer",
            Backend::WasmBindgen => "Vec<u8>",
        }
    }

    /// The Rust type bytes are *taken* as. napi's `Buffer` is Node's
    /// alone; a `Uint8Array` is what both runtimes have, and a `Buffer`
    /// is one, so the layer passes bytes without naming `Buffer`.
    fn bytes_in(self) -> &'static str {
        match self {
            Backend::Napi => "Uint8Array",
            Backend::WasmBindgen => "Vec<u8>",
        }
    }

    /// Bytes owned as a `Vec<u8>`, as the crossing type.
    fn bytes_from(self, vec: &str) -> String {
        match self {
            Backend::Napi => format!("Buffer::from({vec})"),
            Backend::WasmBindgen => vec.to_string(),
        }
    }

    /// A scalar as a JavaScript number. napi crosses a 64-bit integer as a
    /// number; wasm-bindgen would cross it as a `BigInt`, so it is an
    /// `f64` here and the glue casts. A JavaScript number holds every
    /// value the SDK produces at those sizes exactly; one needing more
    /// would need the description to say so.
    fn scalar(self, scalar: Scalar) -> &'static str {
        match (self, scalar) {
            (_, Scalar::Bool) => "bool",
            (_, Scalar::I8 | Scalar::I16 | Scalar::I32) => "i32",
            (_, Scalar::U8 | Scalar::U16 | Scalar::U32) => "u32",
            (Backend::Napi, Scalar::I64 | Scalar::Isize | Scalar::U64 | Scalar::Usize) => "i64",
            (_, _) => "f64",
        }
    }

    /// The attributes of a plain object. napi's `Buffer` is neither
    /// `Clone` nor `Debug`, so a napi object holding one derives neither.
    fn object(self, plain: bool) -> &'static str {
        match (self, plain) {
            (Backend::Napi, true) => "#[napi(object)]\n#[derive(Clone, Debug)]\n",
            (Backend::Napi, false) => "#[napi(object)]\n",
            (Backend::WasmBindgen, _) => {
                "#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]\n#[serde(rename_all = \"camelCase\")]\n"
            }
        }
    }

    /// A field's own attributes in an object: none for napi; for serde,
    /// an absent value is left out rather than written as `undefined`, as
    /// napi leaves it out, and bytes cross as a `Uint8Array`.
    fn field(self, ty: &str) -> &'static str {
        match self {
            Backend::WasmBindgen if ty.starts_with("Option<") => {
                "    #[serde(skip_serializing_if = \"Option::is_none\")]\n"
            }
            Backend::WasmBindgen if ty == "Vec<u8>" => "    #[serde(with = \"serde_bytes\")]\n",
            Backend::Napi | Backend::WasmBindgen => "",
        }
    }

    /// The attribute on a class, and on its `impl`.
    fn class(self) -> &'static str {
        match self {
            Backend::Napi => "#[napi]",
            Backend::WasmBindgen => "#[wasm_bindgen]",
        }
    }

    /// The attribute on an exported function or method. napi camel-cases
    /// the Rust name itself; wasm-bindgen is told the name.
    fn export(self, rust_name: &str) -> String {
        match self {
            Backend::Napi => String::from("#[napi]"),
            Backend::WasmBindgen => format!("#[wasm_bindgen(js_name = {})]", camel(rust_name)),
        }
    }

    /// The attribute on the class's constructor.
    fn constructor(self) -> &'static str {
        match self {
            Backend::Napi => "#[napi(constructor)]",
            Backend::WasmBindgen => "#[wasm_bindgen(constructor)]",
        }
    }

    /// The attribute on a second way to build the class: a static method.
    fn factory(self, rust_name: &str) -> String {
        match self {
            Backend::Napi => String::from("#[napi(factory)]"),
            Backend::WasmBindgen => self.export(rust_name),
        }
    }

    /// Whether a call takes napi's environment. It is how napi builds an
    /// error object and lends the host provider its callback; wasm has
    /// neither need.
    fn env(self) -> bool {
        self == Backend::Napi
    }

    /// A parameter that is a plain object as the exported function takes
    /// it, and the line making the object of it. napi converts the
    /// argument itself; wasm-bindgen is handed a `JsValue`, which serde
    /// reads.
    fn object_param(self, name: &str, ty: &str) -> (String, String) {
        match self {
            Backend::Napi => (format!("{name}: {ty}"), String::new()),
            Backend::WasmBindgen => (
                format!("{name}: JsValue"),
                format!("        let {name}: {ty} = from_js({name})?;\n"),
            ),
        }
    }

    /// The type an object is returned as, and the expression returning
    /// it.
    fn returned_object(self, ty: &str, value: &str) -> (String, String) {
        match self {
            Backend::Napi => (ty.to_string(), value.to_string()),
            Backend::WasmBindgen => (String::from("JsValue"), format!("to_js(&{value})?")),
        }
    }

    /// How an error carrying a record is made, from `message` and
    /// `record` in scope: napi builds it in the environment; the wasm
    /// prelude's `thrown` sets `lastError` on a JavaScript `Error`.
    ///
    /// `err` wraps the error in `Err(…)`, for a body that returns it.
    fn thrown(self, indent: &str, err: bool) -> String {
        let (open, close) = if err { ("Err(", ")") } else { ("", "") };
        match self {
            Backend::Napi => format!(
                "{indent}let thrown = env.create_error(Error::from_reason(message)).and_then(|mut error| {{\n{indent}    error.set_named_property(\"lastError\", record)?;\n{indent}    Ok(Error::from(error.to_unknown()))\n{indent}}});\n{indent}{open}thrown.unwrap_or_else(|failed| failed){close}"
            ),
            Backend::WasmBindgen => format!("{indent}{open}thrown(message, &record){close}"),
        }
    }
}

/// Renders `generated.rs` for a backend's crate.
#[must_use]
pub fn render(api: &Api, backend: Backend) -> String {
    let b = backend;
    let built = built_for(api, b);
    let api = &built;
    let mut out = String::new();
    let _ = writeln!(
        out,
        "//! Generated by `cargo xtask gen ffi` from the Teistro SDK's boundary\n//! crates; do not edit. ABI version {}, SDK {}.\n//!\n//! The `unsafe` calls into the C ABI live here, each with a SAFETY\n//! comment; the contract is `idl/api.json`.\n#![allow(\n    unsafe_code,\n    missing_docs,\n    missing_debug_implementations,\n    unreachable_pub,\n    dead_code,\n    unused_mut,\n    clippy::all,\n    clippy::pedantic,\n    reason = \"generated glue; the contract is the API description\"\n)]\n\nuse core::ffi::c_char;\nuse core::ffi::CStr;\nuse core::ptr;\n\n{}use teistro_core as core_;\nuse teistro_ffi as ffi;\nuse teistro_port_ephemeris as port;\n",
        api.abi_version,
        api.sdk_version,
        b.prelude()
    );
    if b == Backend::WasmBindgen {
        out.push_str(WASM_PRELUDE);
    }
    out.push_str(HELPERS);
    render_blob_helper(&mut out, b);
    out.push_str(STRING_HELPER);
    let enums = enums_at_the_boundary(api);
    for e in api.enums.iter().filter(|e| enums.contains(&e.name)) {
        render_enum_conversions(&mut out, e);
    }
    let objects = objects_at_the_boundary(api);
    for s in api.structs.iter().filter(|s| objects.contains(&s.name)) {
        render_object(&mut out, api, s, b);
    }
    render_last_error_object(&mut out, api, b);
    for f in &api.functions {
        if results(api, f).len() > 1 {
            render_result_object(&mut out, api, f, b);
        }
    }
    for o in &api.opaques {
        render_class(&mut out, api, o, b);
    }
    for f in free_functions(api) {
        render_free_function(&mut out, api, f, b);
    }
    out
}

/// The description as the backend's target builds it. A wasm module has
/// no plugin loader (ADR-0029), so the functions the description marks
/// native-only are not in it, nor is a handle only they make or take.
fn built_for(api: &Api, b: Backend) -> Api {
    let mut api = api.clone();
    if b == Backend::WasmBindgen {
        api.functions.retain(|f| !f.native_only);
        let taken: BTreeSet<String> = api
            .functions
            .iter()
            .flat_map(|f| &f.params)
            .filter_map(|p| pointee_opaque(&api, p).map(|o| o.name.clone()))
            .collect();
        api.opaques.retain(|o| taken.contains(&o.name));
    }
    api
}

/// What the wasm glue needs that napi's prelude has: its `Result` and
/// `Error::from_reason`, so the marshalling is the same text for both,
/// and serde's crossing of a plain object each way.
const WASM_PRELUDE: &str = r#"
/// A refusal, as JavaScript receives it: an `Error`.
pub struct Error(JsValue);

/// What every exported call returns.
pub type Result<T> = core::result::Result<T, Error>;

impl Error {
    /// An `Error` with the sentence given.
    pub fn from_reason(reason: impl Into<String>) -> Error {
        Error(js_sys::Error::new(&reason.into()).into())
    }
}

impl From<Error> for JsValue {
    fn from(error: Error) -> JsValue {
        error.0
    }
}

impl From<serde_wasm_bindgen::Error> for Error {
    fn from(error: serde_wasm_bindgen::Error) -> Error {
        Error::from_reason(error.to_string())
    }
}

/// A plain object read from JavaScript; an absent optional one is `None`.
fn from_js<T: serde::de::DeserializeOwned>(value: JsValue) -> Result<T> {
    Ok(serde_wasm_bindgen::from_value(value)?)
}

/// A plain object as JavaScript receives it; `None` is `undefined`.
fn to_js<T: serde::Serialize>(value: &T) -> Result<JsValue> {
    Ok(serde_wasm_bindgen::to_value(value)?)
}

/// A refusal carrying the call's whole record as `lastError`, which the
/// ergonomic layer rethrows as a `TeistroError`.
fn thrown(message: String, record: &LastError) -> Error {
    let error = js_sys::Error::new(&message);
    // A record that cannot be written leaves the sentence, which is the
    // part a reader needs; `LastError` is strings and integers, so it
    // always can.
    if let Ok(record) = to_js(record) {
        let _ = js_sys::Reflect::set(&error, &JsValue::from_str("lastError"), &record);
    }
    Error(error.into())
}
"#;

/// What every call shares: reading the library's strings and buffers.
const HELPERS: &str = r"
/// A string the library lent or returned, copied before the next call
/// frees or replaces it. Named apart from anything a parameter may be
/// called, because a generated body binds its parameters by their own
/// names (an entry point takes a `text`).
///
/// # Safety
///
/// `ptr` must be null or a NUL-terminated string valid for this call.
unsafe fn lent_text(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    // SAFETY: non-null, and NUL-terminated by the library's contract.
    Some(unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned())
}

/// # Safety
///
/// `ptr` must be null or valid for `len` reads for this call.
unsafe fn slice_or_empty<'a, T>(ptr: *const T, len: usize) -> &'a [T] {
    if ptr.is_null() || len == 0 {
        &[]
    } else {
        // SAFETY: as documented.
        unsafe { core::slice::from_raw_parts(ptr, len) }
    }
}

";

/// The blob reader, in the backend's bytes.
fn render_blob_helper(out: &mut String, b: Backend) {
    let _ = writeln!(
        out,
        "/// The bytes of a blob the library filled, copied into a {} and the\n/// blob freed, so nothing of the library's outlives the call.\nfn take_blob(blob: &mut ffi::blob::TsBlob) -> {} {{\n    // SAFETY: the library wrote `len` bytes at `data`, or left it null.\n    let bytes = unsafe {{ slice_or_empty(blob.data.cast_const(), blob.len) }}.to_vec();\n    // SAFETY: a descriptor this call passed to the library, freed once.\n    unsafe {{ ffi::blob::ts_blob_free(&raw mut *blob) }};\n    {}\n}}",
        match b {
            Backend::Napi => "Buffer",
            Backend::WasmBindgen => "byte array",
        },
        b.bytes(),
        b.bytes_from("bytes")
    );
}

const STRING_HELPER: &str = r"
/// The text of a string the library allocated, copied and the string freed.
fn take_string(string: &mut ffi::string::TsString) -> String {
    // SAFETY: the library wrote a NUL-terminated string, or left it null.
    let text = unsafe { lent_text(string.data.cast_const().cast()) }.unwrap_or_default();
    // SAFETY: a descriptor this call passed to the library, freed once.
    unsafe { ffi::string::ts_string_free(&raw mut *string) };
    text
}
";

// ── What crosses ───────────────────────────────────────────────────────────

fn enums_at_the_boundary(api: &Api) -> BTreeSet<String> {
    let mut used = BTreeSet::new();
    if let Some(status) = status_enum(api) {
        used.insert(status.name.clone());
    }
    let note = |ty: &TypeRef, used: &mut BTreeSet<String>| {
        if let TypeRef::Enum { name } = ty {
            used.insert(name.clone());
        }
    };
    for f in &api.functions {
        for p in &f.params {
            note(&p.ty, &mut used);
        }
        if let Some(r) = &f.returns {
            note(r, &mut used);
        }
    }
    let objects = objects_at_the_boundary(api);
    for s in api.structs.iter().filter(|s| objects.contains(&s.name)) {
        for f in &s.fields {
            for name in [f.meta.enum_name.as_deref(), f.meta.bitset.as_deref()]
                .into_iter()
                .flatten()
            {
                used.insert(name.to_string());
            }
            note(&f.ty, &mut used);
        }
    }
    used
}

/// The structs a function or a callback reads or writes, and everything
/// those nest. A callback's structs are there because a host-implemented
/// port is answered in this binding's own language, so the shapes it is
/// asked in and answers with are part of the surface.
fn objects_at_the_boundary(api: &Api) -> BTreeSet<String> {
    let mut reachable = BTreeSet::new();
    let params = api
        .functions
        .iter()
        .flat_map(|f| f.params.iter())
        .chain(api.callbacks.iter().flat_map(|c| c.params.iter()));
    for p in params {
        if matches!(p.role, Role::StructIn | Role::StructOut) {
            if let Some(s) = pointee_struct(api, p) {
                reachable.insert(s.name.clone());
            }
        }
    }
    loop {
        let named: Vec<String> = api
            .structs
            .iter()
            .filter(|s| reachable.contains(&s.name))
            .flat_map(|s| s.fields.iter())
            .filter_map(|f| match &f.ty {
                TypeRef::Struct { name } => Some(name.clone()),
                _ => None,
            })
            .collect();
        let mut grown = false;
        for name in named {
            grown |= reachable.insert(name);
        }
        if !grown {
            break;
        }
    }
    // The error struct crosses as `LastError`, a shape of its own; a
    // struct holding an array of structs is described rather than built
    // (a binding would have to keep every element's own buffers alive for
    // the call, which is the adapter's business, not an object's).
    if let Some(error) = last_error_struct(api) {
        reachable.remove(&error.name);
    }
    let nested_arrays: Vec<String> = api
        .structs
        .iter()
        .filter(|s| reachable.contains(&s.name))
        .filter(|s| {
            s.fields.iter().zip(field_roles(api, s)).any(|(f, role)| {
                matches!(role, FieldRole::Array { .. })
                    && matches!(f.ty.pointee(), Some(TypeRef::Struct { .. }))
            })
        })
        .map(|s| s.name.clone())
        .collect();
    for name in nested_arrays {
        reachable.remove(&name);
    }
    let referenced: BTreeSet<String> = api
        .structs
        .iter()
        .filter(|s| reachable.contains(&s.name))
        .flat_map(|s| s.fields.iter())
        .filter_map(|f| f.ty.pointee().or(Some(&f.ty)))
        .filter_map(|ty| match ty {
            TypeRef::Struct { name } => Some(name.clone()),
            _ => None,
        })
        .collect();
    let orphans: Vec<String> = reachable
        .iter()
        .filter(|name| !referenced.contains(*name) && !named_by_a_call(api, name))
        .cloned()
        .collect();
    for name in orphans {
        reachable.remove(&name);
    }
    reachable
}

/// Whether a function or a callback names a struct directly.
fn named_by_a_call(api: &Api, name: &str) -> bool {
    api.functions
        .iter()
        .flat_map(|f| f.params.iter())
        .chain(api.callbacks.iter().flat_map(|c| c.params.iter()))
        .any(|p| matches!(p.ty.pointee(), Some(TypeRef::Struct { name: n }) if n == name))
}

/// A function with no handle in play and no memory to reclaim.
fn free_functions(api: &Api) -> Vec<&FunctionDef> {
    api.functions
        .iter()
        .filter(|f| {
            !f.params.iter().any(|p| {
                matches!(
                    p.role,
                    Role::Handle
                        | Role::HandleOut
                        | Role::BlobFree
                        | Role::StringFree
                        | Role::ErrorFree
                )
            })
        })
        .collect()
}

/// A call into the boundary, wrapped in `unsafe` only when the entry
/// point carries a Safety contract.
fn call_expression(f: &FunctionDef, args: &str) -> String {
    let call = format!("{}({args})", path_of(&f.source, &f.name));
    if f.safety.is_some() {
        format!("unsafe {{ {call} }}")
    } else {
        call
    }
}

/// The Rust path of a boundary item, from the source it was read in.
fn path_of(source: &str, name: &str) -> String {
    let mut parts = source.split('/').skip(1);
    let krate = parts.next().unwrap_or_default();
    let alias = CRATES
        .iter()
        .find(|(dir, _)| *dir == krate)
        .map_or("ffi", |(_, alias)| *alias);
    let module = source
        .rsplit('/')
        .next()
        .and_then(|file| file.strip_suffix(".rs"))
        .filter(|m| *m != "lib")
        .unwrap_or_default();
    if module.is_empty() {
        format!("{alias}::{name}")
    } else {
        format!("{alias}::{module}::{name}")
    }
}

/// The Rust path of a struct or an opaque the description names.
fn struct_path(api: &Api, name: &str) -> String {
    let source = api
        .struct_named(name)
        .map(|s| s.source.as_str())
        .or_else(|| api.opaque_named(name).map(|o| o.source.as_str()));
    source.map_or_else(|| format!("ffi::{name}"), |s| path_of(s, name))
}

/// The struct the last-error reader writes; it crosses as `LastError`,
/// which is a shape of its own, so it is not also a plain object.
fn last_error_struct(api: &Api) -> Option<&StructDef> {
    let reader = api
        .functions
        .iter()
        .find(|f| f.name.ends_with("_last_error"))?;
    reader
        .params
        .iter()
        .find(|p| p.role == Role::StructOut)
        .and_then(|p| pointee_struct(api, p))
}

// ── Results ────────────────────────────────────────────────────────────────

impl Handed<'_> {
    fn js_type(&self, api: &Api, b: Backend) -> String {
        match self {
            Handed::Returned(scalar) => b.scalar(*scalar).to_string(),
            Handed::Struct(p) => pointee_struct(api, p)
                .map_or_else(|| String::from("Unknown"), |s| binding_type_name(&s.name)),
            Handed::Blob(_) => b.bytes().to_string(),
            Handed::Owned(_) | Handed::Lent(_) => String::from("String"),
            Handed::Scalar(p) => {
                p.ty.pointee()
                    .and_then(TypeRef::as_scalar)
                    .map_or_else(|| String::from("f64"), |s| b.scalar(s).to_string())
            }
        }
    }

    fn expression(&self, api: &Api) -> String {
        match self {
            Handed::Returned(_) => String::from("value as _"),
            // SAFETY of the read is argued at the call site's block.
            Handed::Struct(p) => format!(
                "unsafe {{ {}::write(&{}) }}",
                binding_type_name(
                    &pointee_struct(api, p).map_or_else(String::new, |s| s.name.clone())
                ),
                snake(&p.name)
            ),
            Handed::Blob(p) => format!("take_blob(&mut {})", snake(&p.name)),
            Handed::Owned(p) => format!("take_string(&mut {})", snake(&p.name)),
            // SAFETY of the read is argued at the call site's block.
            Handed::Lent(p) => format!(
                "unsafe {{ lent_text({}.data) }}.unwrap_or_default()",
                snake(&p.name)
            ),
            Handed::Scalar(p) => format!("{} as _", snake(&p.name)),
        }
    }
}

// ── Types ──────────────────────────────────────────────────────────────────

/// The element of an array field.
fn element(field: &FieldDef) -> Scalar {
    field
        .ty
        .pointee()
        .and_then(TypeRef::as_scalar)
        .unwrap_or(Scalar::U8)
}

/// The crossing type of a struct field, from its role.
fn js_field(field: &FieldDef, role: &FieldRole, b: Backend) -> String {
    let base = match role {
        FieldRole::Flag => String::from("bool"),
        FieldRole::BitSet { .. } => String::from("Vec<String>"),
        FieldRole::Array { .. } | FieldRole::Column => {
            if field.meta.enum_name.is_some() {
                String::from("Vec<String>")
            } else {
                format!("Vec<{}>", b.scalar(element(field)))
            }
        }
        FieldRole::FixedBytes { .. } => b.bytes().to_string(),
        FieldRole::Text => String::from("String"),
        FieldRole::Nested { name } => binding_type_name(name),
        FieldRole::Optional { .. } => match &field.ty {
            TypeRef::Struct { name } => format!("Option<{}>", binding_type_name(name)),
            _ => format!("Option<{}>", plain(field, b)),
        },
        _ => plain(field, b),
    };
    if field.meta.nullable && !base.starts_with("Option<") {
        format!("Option<{base}>")
    } else {
        base
    }
}

fn plain(field: &FieldDef, b: Backend) -> String {
    if field.meta.enum_name.is_some() {
        return String::from("String");
    }
    match &field.ty {
        TypeRef::Scalar { scalar } => b.scalar(*scalar).to_string(),
        TypeRef::Enum { .. } => String::from("String"),
        TypeRef::Pointer { to, .. } if matches!(**to, TypeRef::Char) => String::from("String"),
        _ => String::from("u32"),
    }
}

// ── Enums ──────────────────────────────────────────────────────────────────

fn enum_fns(e: &EnumDef) -> (String, String) {
    names_of(&e.name)
}

/// The pair of conversions for an enum named by an `api:` link. The
/// extractor refuses a link to an enum it does not have, so the name is
/// always one the description defines; deriving the names from it keeps
/// the generator total.
fn enum_fns_named(name: &str) -> (String, String) {
    names_of(name)
}

fn names_of(rust_name: &str) -> (String, String) {
    let base = snake(&binding_type_name(rust_name));
    (format!("{base}_from_str"), format!("{base}_to_str"))
}

fn render_enum_conversions(out: &mut String, e: &EnumDef) {
    let (from, to) = enum_fns(e);
    let repr = e.repr.rust_name();
    let name = binding_type_name(&e.name);
    let _ = writeln!(
        out,
        "/// A `{name}` from the string `catalogue.js` names it by.\npub fn {from}(value: &str) -> Result<{repr}> {{\n    match value {{"
    );
    for v in &e.values {
        let _ = writeln!(
            out,
            "        {:?} => Ok({}),",
            member_value(e.kind.as_deref(), &v.key),
            v.value
        );
    }
    let _ = writeln!(
        out,
        "        other => Err(Error::from_reason(format!(\"`{{other}}` is not a {name}\"))),\n    }}\n}}\n"
    );
    let _ = writeln!(
        out,
        "/// The string for a `{name}`; a value from a newer library is\n/// `{UNKNOWN_MEMBER}`.\npub fn {to}(value: {repr}) -> String {{\n    match value {{"
    );
    for v in &e.values {
        let _ = writeln!(
            out,
            "        {} => {:?},",
            v.value,
            member_value(e.kind.as_deref(), &v.key)
        );
    }
    let _ = writeln!(
        out,
        "        _ => {UNKNOWN_MEMBER:?},\n    }}\n    .to_string()\n}}\n"
    );
}

// ── Objects ────────────────────────────────────────────────────────────────

fn doc(text: &str, indent: &str) -> String {
    line_comment(text, indent, "/// ")
}

fn render_object(out: &mut String, api: &Api, s: &StructDef, b: Backend) {
    let name = binding_type_name(&s.name);
    let roles = field_roles(api, s);
    let c = struct_path(api, &s.name);
    let plain = roles
        .iter()
        .all(|r| !matches!(r, FieldRole::FixedBytes { .. }));
    let _ = writeln!(
        out,
        "{}{}pub struct {name} {{",
        doc(&s.doc, ""),
        b.object(plain)
    );
    for (f, role) in s.fields.iter().zip(&roles) {
        if !role.is_shown() {
            continue;
        }
        let ty = js_field(f, role, b);
        let _ = writeln!(
            out,
            "{}{}    pub {}: {ty},",
            doc(
                &field_doc_with(f, DocStyle::Prose, f.meta.enum_name.as_deref()),
                "    "
            ),
            b.field(&ty),
            snake(&f.name),
        );
    }
    let _ = writeln!(out, "}}\n");
    render_held(out, api, s, &roles, &name, &c);
    render_read_write(out, api, s, &roles, &name, &c, b);
}

/// The value that owns whatever the C struct points at.
fn render_held(
    out: &mut String,
    api: &Api,
    s: &StructDef,
    roles: &[FieldRole],
    name: &str,
    c: &str,
) {
    let _ = writeln!(
        out,
        "/// What a `{name}` lends the C struct built from it: the buffers its\n/// pointers point into, alive for as long as this value is.\npub struct Held{name} {{"
    );
    for (f, role) in s.fields.iter().zip(roles) {
        let ty = match role {
            FieldRole::Array { .. } => format!("Vec<{}>", element(f).rust_name()),
            FieldRole::Text => String::from("Option<std::ffi::CString>"),
            FieldRole::Optional { .. } => match &f.ty {
                TypeRef::Struct { name } => format!("Option<Held{}>", binding_type_name(name)),
                _ => continue,
            },
            FieldRole::Nested { name } => format!("Held{}", binding_type_name(name)),
            FieldRole::FixedBytes { len } => format!("[u8; {len}]"),
            FieldRole::Handshake
            | FieldRole::Reserved
            | FieldRole::Count { .. }
            | FieldRole::Presence { .. } => continue,
            _ => c_field_type(api, f),
        };
        let _ = writeln!(out, "    {}: {ty},", snake(&f.name));
    }
    let _ = writeln!(
        out,
        "}}\n\nimpl Held{name} {{\n    /// The C struct, borrowing this value's buffers.\n    pub fn as_c(&self) -> {c} {{\n        {c} {{"
    );
    for (f, role) in s.fields.iter().zip(roles) {
        let field = snake(&f.name);
        let value = match role {
            FieldRole::Handshake => format!("core::mem::size_of::<{c}>() as u32"),
            FieldRole::Reserved => String::from("Default::default()"),
            FieldRole::Count { of } => format!("self.{}.len()", snake(of)),
            FieldRole::Presence { of } => format!("u8::from(self.{}.is_some())", snake(of)),
            FieldRole::Array { .. } => {
                // A column the library writes into wants a `*mut`; the
                // buffer is this value's, so the cast hands out what it
                // owns rather than something borrowed elsewhere.
                if matches!(&f.ty, TypeRef::Pointer { mutable: true, .. }) {
                    format!("self.{field}.as_ptr().cast_mut()")
                } else {
                    format!("self.{field}.as_ptr()")
                }
            }
            FieldRole::Text => format!("self.{field}.as_ref().map_or(ptr::null(), |s| s.as_ptr())"),
            FieldRole::Optional { .. } => match &f.ty {
                TypeRef::Struct { name } => format!(
                    "self.{field}.as_ref().map_or_else(\n                // SAFETY: the struct is plain data, so all-zero is a value.\n                || unsafe {{ core::mem::zeroed() }},\n                Held{}::as_c,\n            )",
                    binding_type_name(name)
                ),
                _ => format!("self.{field}"),
            },
            FieldRole::Nested { .. } => format!("self.{field}.as_c()"),
            _ => format!("self.{field}"),
        };
        let _ = writeln!(out, "            {field}: {value},");
    }
    let _ = writeln!(out, "        }}\n    }}\n}}\n");
}

/// The C type of a field the held value stores as it is.
fn c_field_type(api: &Api, f: &FieldDef) -> String {
    let _ = api;
    match &f.ty {
        TypeRef::Scalar { scalar } => scalar.rust_name().to_string(),
        TypeRef::Enum { name } => api
            .enum_named(name)
            .map_or_else(|| String::from("u32"), |e| e.repr.rust_name().to_string()),
        TypeRef::Array { of, len } => {
            let inner = match &**of {
                TypeRef::Scalar { scalar } => scalar.rust_name(),
                _ => "u8",
            };
            format!("[{inner}; {len}]")
        }
        _ => String::from("u32"),
    }
}

fn render_read_write(
    out: &mut String,
    api: &Api,
    s: &StructDef,
    roles: &[FieldRole],
    name: &str,
    c: &str,
    b: Backend,
) {
    let _ = writeln!(
        out,
        "impl {name} {{\n    /// The buffers and values the C struct is built from.\n    pub fn read(&self) -> Result<Held{name}> {{\n        Ok(Held{name} {{"
    );
    for (f, role) in s.fields.iter().zip(roles) {
        let field = snake(&f.name);
        let value = match role {
            FieldRole::Handshake
            | FieldRole::Reserved
            | FieldRole::Count { .. }
            | FieldRole::Presence { .. } => continue,
            FieldRole::Flag => format!("u8::from(self.{field})"),
            FieldRole::BitSet { enum_name } => {
                let (from, _) = enum_fns_named(enum_name);
                format!(
                    "self.{field}\n                .iter()\n                .try_fold(0u32, |bits, name| Ok::<u32, Error>(bits | (1 << {from}(name)?)))?"
                )
            }
            FieldRole::Array { .. } => {
                if let Some(enum_name) = &f.meta.enum_name {
                    let (from, _) = enum_fns_named(enum_name);
                    format!("self.{field}.iter().map(|v| {from}(v)).collect::<Result<Vec<_>>>()?")
                } else {
                    let element = element(f).rust_name();
                    format!("self.{field}.iter().map(|v| *v as {element}).collect()")
                }
            }
            FieldRole::Text => {
                let value = if f.meta.nullable {
                    format!("self.{field}.as_deref()")
                } else {
                    format!("Some(self.{field}.as_str())")
                };
                format!(
                    "{value}\n                .map(|s| std::ffi::CString::new(s).map_err(|e| Error::from_reason(e.to_string())))\n                .transpose()?"
                )
            }
            FieldRole::Optional { .. } => match &f.ty {
                TypeRef::Struct { .. } => {
                    format!(
                        "self.{field}.as_ref().map({name_of_read}).transpose()?",
                        name_of_read = "|v| v.read()"
                    )
                }
                _ => format!("self.{field}.unwrap_or_default()"),
            },
            FieldRole::Nested { .. } => format!("self.{field}.read()?"),
            FieldRole::Column => format!("self.{field}.iter().map(|v| *v as _).collect()"),
            FieldRole::FixedBytes { len } => format!(
                "self.{field}\n                .{}()\n                .try_into()\n                .map_err(|_| Error::from_reason(\"`{field}` takes exactly {len} bytes\"))?",
                match b {
                    Backend::Napi => "as_ref",
                    Backend::WasmBindgen => "as_slice",
                }
            ),
            FieldRole::Value => {
                if let Some(enum_name) = &f.meta.enum_name {
                    let repr = api.enum_named(enum_name).map_or(Scalar::U32, |e| e.repr);
                    let (from, _) = enum_fns_named(enum_name);
                    let none = format!("{}::MAX", repr.rust_name());
                    if f.meta.nullable {
                        format!(
                            "match self.{field}.as_deref() {{\n                Some(v) => {from}(v)?,\n                None => {none},\n            }}"
                        )
                    } else {
                        format!("{from}(&self.{field})?")
                    }
                } else {
                    let scalar = f.ty.as_scalar().unwrap_or(Scalar::U32);
                    format!("self.{field} as {}", scalar.rust_name())
                }
            }
        };
        let _ = writeln!(out, "            {field}: {value},");
    }
    let _ = writeln!(
        out,
        "        }})\n    }}\n\n    /// The object a call filled.\n    ///\n    /// # Safety\n    ///\n    /// Every pointer in `raw` must be valid as the struct documents, for\n    /// the length of this call.\n    pub unsafe fn write(raw: &{c}) -> Self {{\n        {name} {{"
    );
    for (f, role) in s.fields.iter().zip(roles) {
        if !role.is_shown() {
            continue;
        }
        let _ = writeln!(
            out,
            "            {}: {},",
            snake(&f.name),
            write_value(api, f, role, b)
        );
    }
    let _ = writeln!(out, "        }}\n    }}\n}}\n");
}

fn write_value(api: &Api, f: &FieldDef, role: &FieldRole, b: Backend) -> String {
    let field = &f.name;
    match role {
        FieldRole::Flag => format!("raw.{field} != 0"),
        FieldRole::BitSet { enum_name } => {
            let repr = api.enum_named(enum_name).map_or(Scalar::U32, |e| e.repr);
            let (_, to) = enum_fns_named(enum_name);
            format!(
                "(0..32)\n                .filter(|bit| raw.{field} & (1u32 << bit) != 0)\n                .map(|bit| {to}(bit as {}))\n                .collect()",
                repr.rust_name()
            )
        }
        FieldRole::Array { count } => {
            let converted = f.meta.enum_name.as_ref().map_or_else(
                || String::from("*v as _"),
                |enum_name| {
                    let (_, to) = enum_fns_named(enum_name);
                    format!("{to}(*v)")
                },
            );
            let constify = if matches!(&f.ty, TypeRef::Pointer { mutable: true, .. }) {
                ".cast_const()"
            } else {
                ""
            };
            format!(
                "unsafe {{ slice_or_empty(raw.{field}{constify}, raw.{}) }}\n                .iter()\n                .map(|v| {converted})\n                .collect()",
                snake(count)
            )
        }
        FieldRole::Column => format!("Vec::new() /* `{field}` is filled by the caller */"),
        FieldRole::FixedBytes { .. } => b.bytes_from(&format!("raw.{field}.to_vec()")),
        FieldRole::Text => {
            let read = format!("unsafe {{ lent_text(raw.{field}) }}");
            if f.meta.nullable {
                read
            } else {
                format!("{read}.unwrap_or_default()")
            }
        }
        FieldRole::Optional { flag } => match &f.ty {
            TypeRef::Struct { name } => format!(
                "(raw.{} != 0).then(|| unsafe {{ {}::write(&raw.{field}) }})",
                snake(flag),
                binding_type_name(name)
            ),
            _ => format!("(raw.{} != 0).then_some(raw.{field} as _)", snake(flag)),
        },
        FieldRole::Nested { name } => {
            format!(
                "unsafe {{ {}::write(&raw.{field}) }}",
                binding_type_name(name)
            )
        }
        FieldRole::Value => {
            if let Some(enum_name) = &f.meta.enum_name {
                let repr = api.enum_named(enum_name).map_or(Scalar::U32, |e| e.repr);
                let (_, to) = enum_fns_named(enum_name);
                if f.meta.nullable {
                    format!(
                        "(raw.{field} != {}::MAX).then(|| {to}(raw.{field}))",
                        repr.rust_name()
                    )
                } else {
                    format!("{to}(raw.{field})")
                }
            } else {
                format!("raw.{field} as _")
            }
        }
        FieldRole::Handshake
        | FieldRole::Reserved
        | FieldRole::Count { .. }
        | FieldRole::Presence { .. } => String::new(),
    }
}

/// The fields of `LastError`, as the ergonomic layer reads them.
const LAST_ERROR_FIELDS: [(&str, &str); 8] = [
    ("status", "String"),
    ("code", "i32"),
    ("provider_code", "i32"),
    ("message", "Option<String>"),
    ("detail", "Option<String>"),
    ("field", "Option<String>"),
    ("hint", "Option<String>"),
    ("message_key", "Option<String>"),
];

fn render_last_error_object(out: &mut String, api: &Api, b: Backend) {
    let Some(status) = status_enum(api) else {
        return;
    };
    let (_, to) = enum_fns(status);
    let Some(error) = api.structs.iter().find(|s| s.role == StructRole::Error) else {
        return;
    };
    let c = struct_path(api, &error.name);
    let fields = LAST_ERROR_FIELDS
        .iter()
        .fold(String::new(), |mut fields, (name, ty)| {
            let _ = writeln!(fields, "{}    pub {name}: {ty},", b.field(ty));
            fields
        });
    let _ = writeln!(
        out,
        "/// The outcome of the last call on a context, as the ergonomic layer\n/// rethrows it: the status by name and by code, the provider's own code,\n/// and the message, detail, field, hint and message key the library gave.\n{}pub struct LastError {{\n{fields}}}\n\nimpl LastError {{\n    /// # Safety\n    ///\n    /// Every pointer in `raw` must be a string the context lent for this\n    /// call, or null.\n    unsafe fn of(raw: &{c}) -> Self {{\n        // SAFETY: the caller's contract.\n        unsafe {{\n            LastError {{\n                status: {to}(raw.status),\n                code: raw.status,\n                provider_code: raw.provider_code,\n                message: lent_text(raw.message),\n                detail: lent_text(raw.detail),\n                field: lent_text(raw.field),\n                hint: lent_text(raw.hint),\n                message_key: lent_text(raw.key),\n            }}\n        }}\n    }}\n}}\n",
        b.object(true)
    );
    let Some(free) = crate::rules::error_free(api) else {
        return;
    };
    let _ = writeln!(
        out,
        "/// A refusal to build a handle, as the error this addon throws: the\n/// library's own sentence, with its whole record kept as `lastError` so\n/// the ergonomic layer rethrows it as the same `TeistroError` a context's\n/// refusal becomes. The record's strings are the library's, released here.\nfn refused({}raw: &mut {c}) -> Error {{\n    // SAFETY: a record the library wrote for this call, or left zeroed.\n    let record = unsafe {{ LastError::of(raw) }};\n    // SAFETY: the same record, released once; a lent or zeroed one is ignored.\n    {};\n    let message = record.message.clone().unwrap_or_else(|| {{\n        // SAFETY: the library returns a static NUL-terminated string.\n        unsafe {{ lent_text(ffi::ts_status_message(record.code)) }}.unwrap_or_default()\n    }});\n{}\n}}\n",
        if b.env() { "env: &Env, " } else { "" },
        call_expression(free, "&raw mut *raw"),
        b.thrown("    ", false)
    );
}

fn render_result_object(out: &mut String, api: &Api, f: &FunctionDef, b: Backend) {
    let name = format!(
        "{}Result",
        pascal(f.name.strip_prefix(&api.prefix).unwrap_or(&f.name))
    );
    let _ = writeln!(
        out,
        "/// What `{}` hands back.\n{}pub struct {name} {{",
        f.name,
        b.object(true)
    );
    for r in results(api, f) {
        let ty = r.js_type(api, b);
        let _ = writeln!(out, "{}    pub {}: {ty},", b.field(&ty), r.name());
    }
    let _ = writeln!(out, "}}\n");
}

// ── The class ──────────────────────────────────────────────────────────────

/// One call's Rust body.
struct Call {
    params: Vec<String>,
    setup: String,
    args: Vec<String>,
    finish: String,
    returns: String,
}

#[allow(
    clippy::too_many_lines,
    reason = "one arm per parameter role; splitting the table would hide it"
)]
fn build_call(api: &Api, f: &FunctionDef, receiver: Option<&str>, b: Backend) -> Call {
    let mut params = Vec::new();
    let mut setup = String::new();
    let mut args = Vec::new();
    let mut last_pointer = String::new();
    // A struct a *way in* takes may be omitted for every default, which
    // is what the C signature says of it (`options` may be null). That is
    // true of a factory as much as of the constructor, and reading it off
    // the primary one alone made the factory demand what the constructor
    // does not.
    let optional_struct = builds_a_handle(api, f);
    for p in &f.params {
        let name = snake(&p.name);
        match p.role {
            // The handle of the class being rendered is `self`; any
            // other opaque's is a parameter, and the class that holds it
            // is what the caller passes. A factory is the only shape
            // where the second case arises, and it arose unplaced —
            // `rules::factories` says why.
            Role::Handle => {
                let held = pointee_opaque(api, p).map(|o| o.name.clone());
                if held.as_deref() == receiver || receiver.is_none() {
                    args.push(String::from("self.handle"));
                } else {
                    let class = binding_type_name(held.as_deref().unwrap_or_default());
                    params.push(format!("{name}: &{class}"));
                    args.push(format!("{name}.handle"));
                }
            }
            Role::HandleOut => {
                let opaque = pointee_opaque(api, p).map_or_else(String::new, |o| o.name.clone());
                let _ = writeln!(
                    setup,
                    "        let mut handle: *mut {} = ptr::null_mut();",
                    struct_path(api, &opaque)
                );
                args.push(String::from("&raw mut handle"));
            }
            Role::Value => {
                // A parameter that names an enum, by its type or by an
                // `api:` line, crosses as the string `catalogue.js` names.
                let named = match &p.ty {
                    TypeRef::Enum { name } => Some(name.as_str()),
                    _ => p.meta.enum_name.as_deref(),
                };
                match (named, &p.ty) {
                    (Some(e), _) => {
                        let (from, _) = enum_fns_named(e);
                        let cast =
                            p.ty.as_scalar()
                                .map_or_else(String::new, |s| format!(" as {}", s.rust_name()));
                        params.push(format!("{name}: String"));
                        let _ = writeln!(setup, "        let {name} = {from}(&{name})?{cast};");
                        args.push(name);
                    }
                    (None, TypeRef::Scalar { scalar }) => {
                        params.push(format!("{name}: {}", b.scalar(*scalar)));
                        args.push(format!("{name} as {}", scalar.rust_name()));
                    }
                    _ => {}
                }
            }
            Role::StructIn => {
                let Some(s) = pointee_struct(api, p) else {
                    continue;
                };
                let ty = binding_type_name(&s.name);
                if optional_struct {
                    let (param, read) = b.object_param(&name, &format!("Option<{ty}>"));
                    params.push(param);
                    setup.push_str(&read);
                    let _ = writeln!(
                        setup,
                        "        let held_{name} = {name}.map(|v| v.read()).transpose()?;\n        let raw_{name} = held_{name}.as_ref().map(Held{ty}::as_c);\n        let {name} = raw_{name}.as_ref().map_or(ptr::null(), |v| &raw const *v);"
                    );
                } else {
                    let (param, read) = b.object_param(&name, &ty);
                    params.push(param);
                    setup.push_str(&read);
                    let _ = writeln!(
                        setup,
                        "        let held_{name} = {name}.read()?;\n        let raw_{name} = held_{name}.as_c();\n        let {name} = &raw const raw_{name};"
                    );
                }
                args.push(name);
            }
            Role::StructOut => {
                let Some(s) = pointee_struct(api, p) else {
                    continue;
                };
                let _ = writeln!(
                    setup,
                    "{}",
                    zeroed(&struct_path(api, &s.name), &name, has_handshake(s))
                );
                args.push(format!("&raw mut {name}"));
            }
            Role::BlobOut => {
                let _ = writeln!(
                    setup,
                    "        let mut {name} = ffi::blob::TsBlob::empty();"
                );
                args.push(format!("&raw mut {name}"));
            }
            Role::StringOut => {
                let _ = writeln!(
                    setup,
                    "        let mut {name} = ffi::string::TsString::empty();"
                );
                args.push(format!("&raw mut {name}"));
            }
            Role::StrOut => {
                let _ = writeln!(
                    setup,
                    "        let mut {name} = ffi::string::TsStr {{ data: ptr::null(), len: 0 }};"
                );
                args.push(format!("&raw mut {name}"));
            }
            Role::ScalarOut => {
                let scalar =
                    p.ty.pointee()
                        .and_then(TypeRef::as_scalar)
                        .unwrap_or(Scalar::F64);
                let _ = writeln!(
                    setup,
                    "        let mut {name}: {} = Default::default();",
                    scalar.rust_name()
                );
                args.push(format!("&raw mut {name}"));
            }
            Role::StringIn if p.meta.nullable => {
                params.push(format!("{name}: Option<String>"));
                let _ = writeln!(
                    setup,
                    "        let {name} = {name}\n            .map(|s| std::ffi::CString::new(s).map_err(|e| Error::from_reason(e.to_string())))\n            .transpose()?;\n        let {name} = {name}.as_ref().map_or(ptr::null(), |s| s.as_ptr());"
                );
                args.push(name);
            }
            Role::StringIn => {
                params.push(format!("{name}: String"));
                let _ = writeln!(
                    setup,
                    "        let {name} = std::ffi::CString::new({name}).map_err(|e| Error::from_reason(e.to_string()))?;"
                );
                args.push(format!("{name}.as_ptr()"));
            }
            Role::BytesIn => {
                params.push(format!("{name}: {}", b.bytes_in()));
                last_pointer.clone_from(&name);
                args.push(format!("{name}.as_ptr()"));
            }
            Role::ArrayIn => {
                params.push(format!("{name}: Vec<f64>"));
                last_pointer.clone_from(&name);
                args.push(format!("{name}.as_ptr()"));
            }
            Role::Length => args.push(format!("{last_pointer}.len()")),
            Role::VtableIn => {
                let (info, read) =
                    b.object_param("provider", "Option<crate::provider::ProviderInfo>");
                params.push(info);
                params.push(String::from(match b {
                    Backend::Napi => "provider_positions: Option<Function<FnArgs<(PositionRequest,)>, Option<PositionColumns>>>",
                    Backend::WasmBindgen => "provider_positions: Option<js_sys::Function>",
                }));
                setup.push_str(&read);
                let _ = writeln!(
                    setup,
                    "        let host = match (provider, provider_positions) {{\n            (Some(info), Some(callback)) => Some(crate::provider::Host::bind(info, &callback)?),\n            (None, None) => None,\n            _ => {{\n                return Err(Error::from_reason(\n                    \"a provider needs both its description and its positions callback\",\n                ));\n            }}\n        }};\n        let (host_vtable, user_data) = crate::provider::parts(host.as_ref());\n        let {name} = host_vtable.as_ref().map_or(ptr::null(), |v| &raw const *v);"
                );
                args.push(name);
            }
            Role::UserData => args.push(String::from("user_data")),
            Role::BlobFree | Role::StringFree | Role::ErrorFree => {}
        }
    }
    let (returns, finish) = call_outputs(api, f, b);
    Call {
        params,
        setup,
        args,
        finish,
        returns,
    }
}

/// What a call hands back, and the lines that build it: nothing, the one
/// value it produces, or an object of the several it produces.
fn call_outputs(api: &Api, f: &FunctionDef, b: Backend) -> (String, String) {
    let handed = results(api, f);
    let single = if handed.len() == 1 {
        handed.first()
    } else {
        None
    };
    let mut finish = String::new();
    let returns = match (handed.is_empty(), single) {
        (true, _) => {
            finish.push_str("        Ok(())\n");
            String::from("()")
        }
        (false, Some(one @ Handed::Struct(_))) => {
            let (returns, value) = b.returned_object(&one.js_type(api, b), &one.expression(api));
            let _ = writeln!(finish, "        Ok({value})");
            returns
        }
        (false, Some(one)) => {
            let _ = writeln!(finish, "        Ok({})", one.expression(api));
            one.js_type(api, b)
        }
        (false, None) => {
            let object = format!(
                "{}Result",
                pascal(f.name.strip_prefix(&api.prefix).unwrap_or(&f.name))
            );
            let mut built = format!("{object} {{\n");
            for r in &handed {
                let _ = writeln!(built, "            {}: {},", r.name(), r.expression(api));
            }
            built.push_str("        }");
            let (returns, value) = b.returned_object(&object, "handed");
            if b == Backend::Napi {
                let _ = writeln!(finish, "        Ok({built})");
            } else {
                let _ = writeln!(finish, "        let handed = {built};\n        Ok({value})");
            }
            returns
        }
    };
    (returns, finish)
}

/// Whether a function is a way in: the constructor of an opaque, or one
/// of its factories.
fn builds_a_handle(api: &Api, f: &FunctionDef) -> bool {
    api.opaques.iter().any(|o| {
        constructor(api, o).is_some_and(|c| c.name == f.name)
            || crate::rules::factories(api, o)
                .iter()
                .any(|x| x.name == f.name)
    })
}

fn zeroed(path: &str, binding: &str, handshake: bool) -> String {
    let size = if handshake {
        format!("\n        {binding}.struct_size = core::mem::size_of::<{path}>() as u32;")
    } else {
        String::new()
    };
    format!(
        "        // SAFETY: every field is a plain integer, float or pointer, so\n        // all-zero is a valid value; a size, where the struct has one, is set\n        // before the call reads it.\n        let mut {binding}: {path} = unsafe {{ core::mem::zeroed() }};{size}"
    )
}

fn render_class(out: &mut String, api: &Api, opaque: &OpaqueDef, b: Backend) {
    let name = binding_type_name(&opaque.name);
    let c = struct_path(api, &opaque.name);
    let host = takes_host(api, opaque);
    // A host-implemented port is answered in this binding's own language,
    // so the adapter that wraps it into the vtable is hand-written, in
    // `crate::provider` (`02-architecture/07-binding-architecture.md`).
    // The class owns it, because the vtable points into it for as long as
    // the handle lives.
    let field = if host {
        "\n    /// The host-implemented provider, alive while the handle is.\n    host: Option<crate::provider::Host>,"
    } else {
        ""
    };
    let _ = writeln!(
        out,
        "{}{class}\npub struct {name} {{\n    handle: *mut {c},{field}\n}}\n\n// SAFETY: a context is used by one thread at a time, which is the contract\n// the boundary documents; a worker thread builds its own.\nunsafe impl Send for {name} {{}}\n\n{class}\nimpl {name} {{",
        doc(&opaque.doc, ""),
        class = b.class()
    );
    if let Some(ctor) = constructor(api, opaque) {
        render_constructor(out, api, ctor, &name, host, b);
    }
    for factory in crate::rules::factories(api, opaque) {
        render_factory(out, api, factory, opaque, &name, host, b);
    }
    render_last_error_method(out, api, opaque, b);
    render_check(out, api, opaque, b);
    if host {
        render_host_helpers(out, b);
    }
    for m in methods(api, opaque) {
        render_method(out, api, opaque, m, host, b);
    }
    if let Some(free) = destructor(api, opaque) {
        render_dispose(out, free, b);
    }
    let _ = writeln!(out, "}}\n");
    if let Some(free) = destructor(api, opaque) {
        // The handle is nulled by `dispose`, so `Drop` frees only what is
        // still there. The freeing entry point ignores a null anyway; the
        // check is here so a reader does not have to know that.
        let _ = writeln!(
            out,
            "impl Drop for {name} {{\n    fn drop(&mut self) {{\n        if self.handle.is_null() {{\n            return;\n        }}\n        // SAFETY: the handle came from the constructor and is dropped once.\n        {};\n    }}\n}}\n",
            call_expression(free, "self.handle")
        );
    }
}

/// The explicit release every binding ships beside its finaliser.
///
/// ADR-0007's fourth finding: a result that waits for the collector can
/// exhaust memory, so a handle is freed when the caller says so and not
/// only when the garbage collector gets to it. The handle is nulled
/// rather than left dangling, and the boundary refuses a null handle with
/// `INVALID_ARG`, so a call after `dispose` is a clean refusal.
fn render_dispose(out: &mut String, free: &FunctionDef, b: Backend) {
    let _ = writeln!(
        out,
        "\n    /// Frees the handle's native memory now, rather than when the\n    /// collector gets to it. Calling it twice is allowed, and a call on a\n    /// disposed handle is refused with `INVALID_ARG`.\n    {}\n    pub fn dispose(&mut self) {{\n        if self.handle.is_null() {{\n            return;\n        }}\n        // SAFETY: the handle came from the constructor and is freed once;\n        // nulling it here is what makes that true.\n        {};\n        self.handle = std::ptr::null_mut();\n    }}",
        b.export("dispose"),
        call_expression(free, "self.handle")
    );
}

fn render_constructor(
    out: &mut String,
    api: &Api,
    ctor: &FunctionDef,
    name: &str,
    host: bool,
    b: Backend,
) {
    let mut call = build_call(api, ctor, None, b);
    if b.env() {
        call.params.insert(0, String::from("env: Env"));
    }
    let built = if host { ", host" } else { "" };
    let _ = writeln!(
        out,
        "{}    {}\n    pub fn new({}) -> Result<Self> {{\n{}        // SAFETY: every pointer is valid for the call; the handle is owned\n        // from here and freed once, in `Drop`.\n        let status = {};\n        if status != core_::Status::Ok {{\n            return Err(refused({}&mut out_error));\n        }}\n        Ok({name} {{ handle{built} }})\n    }}\n",
        doc(&ctor.doc, "    "),
        b.constructor(),
        call.params.join(", "),
        call.setup,
        call_expression(ctor, &call.args.join(", ")),
        refused_env(b)
    );
}

/// The environment `refused` takes, where the backend has one.
fn refused_env(b: Backend) -> &'static str {
    if b.env() { "&env, " } else { "" }
}

/// A second way to build the class: a static factory.
///
/// `#[napi(factory)]` and not a second `#[napi(constructor)]`, because a
/// JavaScript class has one of the latter. Everything else is the
/// constructor's body — `build_call` renders the roles the same way, and
/// a `handle` of another opaque becomes a parameter of that class rather
/// than `self`.
fn render_factory(
    out: &mut String,
    api: &Api,
    f: &FunctionDef,
    opaque: &OpaqueDef,
    name: &str,
    host: bool,
    b: Backend,
) {
    let mut call = build_call(api, f, Some(&opaque.name), b);
    if b.env() {
        call.params.insert(0, String::from("env: Env"));
    }
    // A factory does not bind a host-implemented port: the provider it is
    // given is already a handle the boundary owns. The field is still on
    // the struct, so it is filled with nothing.
    let built = if host { ", host: None" } else { "" };
    let method = crate::rules::method_name(api, opaque, f);
    let _ = writeln!(
        out,
        "{}    {}\n    pub fn {method}({}) -> Result<Self> {{\n{}        // SAFETY: every pointer is valid for the call; the handle is owned\n        // from here and freed once, in `Drop`.\n        let status = {};\n        if status != core_::Status::Ok {{\n            return Err(refused({}&mut out_error));\n        }}\n        Ok({name} {{ handle{built} }})\n    }}\n",
        doc(&f.doc, "    "),
        b.factory(&method),
        call.params.join(", "),
        call.setup,
        call_expression(f, &call.args.join(", ")),
        refused_env(b)
    );
}

/// Whether an opaque's constructor takes a host-implemented port.
fn takes_host(api: &Api, opaque: &OpaqueDef) -> bool {
    constructor(api, opaque)
        .is_some_and(|ctor| ctor.params.iter().any(|p| p.role == Role::VtableIn))
}

/// The two lines every call of a class with a host runs: the environment
/// is lent for the length of the call and taken back after it, so a
/// callback that escaped finds nothing to call into.
fn render_host_helpers(out: &mut String, b: Backend) {
    let (param, arg) = if b.env() {
        ("env: Env", "env")
    } else {
        ("", "")
    };
    let _ = writeln!(
        out,
        "    /// Lends the environment to the host provider for one call.\n    fn enter(&self{}{param}) {{\n        if let Some(host) = self.host.as_ref() {{\n            host.enter({arg});\n        }}\n    }}\n\n    /// Takes it back, and reports what the provider threw.\n    fn leave(&self) -> Result<()> {{\n        match self.host.as_ref() {{\n            Some(host) => host.leave(),\n            None => Ok(()),\n        }}\n    }}\n",
        if b.env() { ", " } else { "" }
    );
}

fn render_last_error_method(out: &mut String, api: &Api, opaque: &OpaqueDef, b: Backend) {
    // This opaque's own reader, matched by the handle it takes: a second
    // opaque type without one must not inherit the first's.
    let Some(reader) = crate::rules::last_error(api, opaque) else {
        return;
    };
    let Some(s) = reader
        .params
        .iter()
        .find(|p| p.role == Role::StructOut)
        .and_then(|p| pointee_struct(api, p))
    else {
        return;
    };
    let _ = writeln!(
        out,
        "    /// The outcome of the last call on this context, which the layer\n    /// above rethrows with its field, its hint and its code.\n    {}fn last_error(&self) -> Option<LastError> {{\n{}\n        // SAFETY: a live handle and a valid struct with its size set.\n        let status = {};\n        if status != core_::Status::Ok || raw.status == 0 {{\n            return None;\n        }}\n        // SAFETY: the strings are the ones this context lent just now.\n        Some(unsafe {{ LastError::of(&raw) }})\n    }}\n{}",
        match b {
            Backend::Napi => "#[napi]\n    pub ",
            Backend::WasmBindgen => "",
        },
        zeroed(&struct_path(api, &s.name), "raw", has_handshake(s)),
        call_expression(reader, "self.handle, &raw mut raw"),
        // wasm-bindgen exports what a method returns as it is, so the
        // record goes out through serde from a method of its own; the
        // Rust one stays, for `check`.
        match b {
            Backend::Napi => "",
            Backend::WasmBindgen => {
                "\n    /// The same, as JavaScript reads it: `undefined` when the last call\n    /// succeeded.\n    #[wasm_bindgen(js_name = lastError)]\n    pub fn last_error_js(&self) -> Result<JsValue> {\n        to_js(&self.last_error())\n    }\n"
            }
        }
    );
}

/// The check every method runs: a status other than `Ok` becomes an error
/// carrying the library's own sentence.
fn render_check(out: &mut String, api: &Api, opaque: &OpaqueDef, b: Backend) {
    // An opaque type with a last-error reader gets the library's own
    // sentence; one without gets the status's, because there is nothing
    // to read it from and inventing a reader is what stopped the addon
    // compiling when the second opaque type arrived.
    //
    // **The record travels on the error.** The layer above used to read the
    // context's last error for any exception its call threw, so an argument
    // the layer itself refused before reaching the library was reported as
    // whatever the library had refused last. An error that carries the record
    // of the call that failed cannot be confused with one that has none.
    let body = if crate::rules::last_error(api, opaque).is_some() {
        format!(
            "        let record = self.last_error();\n        let message = record\n            .as_ref()\n            .and_then(|e| e.message.clone())\n            .unwrap_or_else(|| {{\n                // SAFETY: the library returns a static NUL-terminated string.\n                unsafe {{ lent_text(ffi::ts_status_message(status.code())) }}.unwrap_or_default()\n            }});\n        let Some(record) = record else {{\n            return Err(Error::from_reason(message));\n        }};\n{}",
            b.thrown("        ", true)
        )
    } else {
        format!(
            "{}        // SAFETY: the library returns a static NUL-terminated string.\n        let message =\n            unsafe {{ lent_text(ffi::ts_status_message(status.code())) }}.unwrap_or_default();\n        Err(Error::from_reason(message))",
            if b.env() {
                "        let _ = env;\n"
            } else {
                ""
            }
        )
    };
    let _ = writeln!(
        out,
        "    /// Turns a failed call into an error whose message is the library's\n    /// own sentence, with the call's record attached as `lastError`.\n    fn check(&self, {}status: core_::Status) -> Result<()> {{\n        if status == core_::Status::Ok {{\n            return Ok(());\n        }}\n{body}\n    }}\n",
        if b.env() { "env: &Env, " } else { "" }
    );
}

fn render_method(
    out: &mut String,
    api: &Api,
    opaque: &OpaqueDef,
    m: &FunctionDef,
    host: bool,
    b: Backend,
) {
    if m.name.ends_with("_last_error") {
        return;
    }
    let method = snake(&method_name(api, opaque, m));
    let call = build_call(api, m, Some(&opaque.name), b);
    let called = call_expression(m, &call.args.join(", "));
    // A call that may reach the host provider brackets itself: the
    // environment is lent before it and taken back after, and what the
    // provider threw is reported in place of the status it turned into.
    let (lend, take) = match (host, b.env()) {
        (true, true) => ("        self.enter(env);\n", "        self.leave()?;\n"),
        (true, false) => ("        self.enter();\n", "        self.leave()?;\n"),
        (false, _) => ("", ""),
    };
    // The environment is taken by a call that lends it to the provider, and
    // by one that can fail, whose error carries its own record.
    let status = returns_status(api, m);
    let env_param = if b.env() && (host || status) {
        "env: Env, "
    } else {
        ""
    };
    let env_arg = if b.env() { "&env, " } else { "" };
    let body = if status {
        format!("        let status = {called};\n{take}        self.check({env_arg}status)?;")
    } else {
        format!("        let value = {called};\n{take}")
    };
    let _ = writeln!(
        out,
        "{}    {}\n    pub fn {method}(&self, {env_param}{}) -> Result<{}> {{\n{}{lend}        // SAFETY: the handle is live and every pointer is valid for the call.\n{body}\n{}    }}\n",
        doc(&m.doc, "    "),
        b.export(&method),
        call.params.join(", "),
        call.returns,
        call.setup,
        call.finish
    );
}

fn render_free_function(out: &mut String, api: &Api, f: &FunctionDef, b: Backend) {
    let name = snake(f.name.strip_prefix(&api.prefix).unwrap_or(&f.name));
    let call = build_call(api, f, None, b);
    let called = call_expression(f, &call.args.join(", "));
    if matches!(&f.returns, Some(TypeRef::Pointer { to, .. }) if matches!(**to, TypeRef::Char)) {
        let _ = writeln!(
            out,
            "{}{}\npub fn {name}({}) -> Result<String> {{\n{}    // SAFETY: the library returns a static NUL-terminated string.\n    Ok(unsafe {{ lent_text({called}) }}.unwrap_or_default())\n}}\n",
            doc(&f.doc, ""),
            b.export(&name),
            call.params.join(", "),
            call.setup.replace("        ", "    ")
        );
        return;
    }
    let setup = call.setup.replace("        ", "    ");
    let finish = call.finish.replace("        ", "    ");
    let body = if returns_status(api, f) {
        format!(
            "    let status = {called};\n    if status != core_::Status::Ok {{\n        // SAFETY: the library returns a static NUL-terminated string.\n        let message = unsafe {{ lent_text(ffi::ts_status_message(status.code())) }}.unwrap_or_default();\n        return Err(Error::from_reason(format!(\"{}: {{message}}\")));\n    }}",
            f.name
        )
    } else {
        format!("    let value = {called};")
    };
    let _ = writeln!(
        out,
        "{}{}\npub fn {name}({}) -> Result<{}> {{\n{setup}    // SAFETY: every pointer is valid for the call.\n{body}\n{finish}}}\n",
        doc(&f.doc, ""),
        b.export(&name),
        call.params.join(", "),
        call.returns
    );
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "a test fails by panicking")]

    use std::collections::BTreeSet;

    use super::{Backend, render};
    use crate::model::Api;
    use crate::names::camel;

    /// The description the bindings are generated from.
    fn api() -> Api {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../idl/api.json");
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    /// The JavaScript name of every function and method a rendering
    /// exports: napi's camel-cased from the Rust name after `#[napi]` or
    /// `#[napi(factory)]`, wasm-bindgen's as its `js_name` states it.
    fn exported(glue: &str) -> BTreeSet<String> {
        let mut names = BTreeSet::new();
        let mut lines = glue.lines().map(str::trim);
        while let Some(line) = lines.next() {
            if let Some(name) = line
                .strip_prefix("#[wasm_bindgen(js_name = ")
                .and_then(|rest| rest.strip_suffix(")]"))
            {
                names.insert(name.to_string());
            } else if matches!(line, "#[napi]" | "#[napi(factory)]") {
                let next = lines.next().unwrap_or_default();
                if let Some(rest) = next.strip_prefix("pub fn ") {
                    let rust = rest.split('(').next().unwrap_or_default();
                    names.insert(camel(rust));
                }
            }
        }
        names
    }

    /// The design's one promise: both backends expose the same members,
    /// so one `index.js` serves both. The only difference is the plugin
    /// loader, which a wasm module cannot have (ADR-0029).
    #[test]
    fn both_backends_export_the_same_members_but_the_loader() {
        let api = api();
        let napi = exported(&render(&api, Backend::Napi));
        let wasm = exported(&render(&api, Backend::WasmBindgen));
        let only_napi: Vec<&String> = napi.difference(&wasm).collect();
        let only_wasm: Vec<&String> = wasm.difference(&napi).collect();
        assert_eq!(only_napi, ["newWithProvider"]);
        assert!(only_wasm.is_empty(), "{only_wasm:?}");
        assert!(napi.len() > 40, "{} members read", napi.len());
    }

    /// No native-only symbol reaches the wasm glue, and every one reaches
    /// napi's.
    #[test]
    fn a_native_only_function_is_left_out_of_the_wasm_glue_alone() {
        let api = api();
        let napi = render(&api, Backend::Napi);
        let wasm = render(&api, Backend::WasmBindgen);
        let native: Vec<&str> = api
            .functions
            .iter()
            .filter(|f| f.native_only)
            .map(|f| f.name.as_str())
            .collect();
        assert_eq!(native.len(), 3);
        for name in native {
            assert!(napi.contains(name), "{name} missing from napi");
            assert!(!wasm.contains(name), "{name} in the wasm glue");
        }
    }

    /// The signatures after an attribute starting `attribute` that name a
    /// 64-bit integer, each read up to and including its opening brace.
    fn wide_signatures(glue: &str, attribute: &str) -> Vec<String> {
        let mut found = Vec::new();
        let mut lines = glue.lines().map(str::trim);
        while let Some(line) = lines.next() {
            if !line.starts_with(attribute) {
                continue;
            }
            let mut signature = String::new();
            for part in lines.by_ref() {
                signature.push_str(part);
                if part.ends_with('{') || part.ends_with(';') {
                    break;
                }
            }
            if ["i64", "u64", "isize", "usize"].iter().any(|wide| {
                signature.contains(&format!(": {wide}")) || signature.contains(&format!("<{wide}>"))
            }) {
                found.push(signature);
            }
        }
        found
    }

    /// wasm-bindgen would cross a 64-bit integer as a `BigInt`, where napi
    /// and `index.js` use a number, so no exported wasm signature names
    /// one. napi's do, which is what proves the reading sees them.
    #[test]
    fn no_wasm_signature_crosses_a_bigint() {
        let api = api();
        let wasm = wide_signatures(&render(&api, Backend::WasmBindgen), "#[wasm_bindgen(");
        assert!(wasm.is_empty(), "{wasm:#?}");
        let napi = wide_signatures(&render(&api, Backend::Napi), "#[napi");
        assert!(!napi.is_empty(), "the reading found no napi `i64` either");
    }
}
