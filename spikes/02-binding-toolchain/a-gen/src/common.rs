//! Rules every emitter shares, so a naming, documentation or classification
//! decision is made once and every binding agrees with the others.

use std::fmt::Write;

use crate::model::{
    Api, CallbackDef, EnumDef, FieldDef, FunctionDef, ParamDef, Role, StructDef, TypeRef,
    strip_prefix,
};

/// The binding-facing name of a C type: `TspSettings` becomes `Settings`.
pub(crate) fn binding_name(api: &Api, name: &str) -> String {
    strip_prefix(name, &api.prefix)
}

/// `PascalCase` to `snake_case`.
pub(crate) fn snake(name: &str) -> String {
    separate(name, '_')
}

/// `PascalCase` to `kebab-case`.
pub(crate) fn kebab(name: &str) -> String {
    separate(name, '-')
}

fn separate(name: &str, separator: char) -> String {
    let mut out = String::with_capacity(name.len() + 4);
    for (i, ch) in name.chars().enumerate() {
        if ch.is_ascii_uppercase() && i > 0 {
            out.push(separator);
        }
        out.push(ch.to_ascii_lowercase());
    }
    out
}

/// A struct that carries a callback pointer: a provider vtable.
pub(crate) fn is_vtable(s: &StructDef) -> bool {
    s.fields
        .iter()
        .any(|f| matches!(f.ty, TypeRef::Callback { .. }))
}

/// A struct a binding exposes as a plain object: neither a vtable nor a blob.
pub(crate) fn is_object(s: &StructDef) -> bool {
    s.blob.is_none() && !is_vtable(s)
}

/// A field a binding shows: not the handshake size, not padding.
pub(crate) fn is_visible(f: &FieldDef) -> bool {
    f.meta.role.as_deref() != Some("struct_size") && f.name != "reserved"
}

/// A function with no handle in play: a static function of the module.
pub(crate) fn is_free_function(f: &FunctionDef) -> bool {
    !f.params
        .iter()
        .any(|p| matches!(p.role, Role::Handle | Role::HandleOut | Role::BlobInOut))
}

/// The function that creates the opaque handle.
pub(crate) fn constructor(api: &Api) -> Option<&FunctionDef> {
    api.functions
        .iter()
        .find(|f| f.params.iter().any(|p| p.role == Role::HandleOut))
}

/// The function that frees the opaque handle.
pub(crate) fn destructor(api: &Api) -> Option<&FunctionDef> {
    api.functions.iter().find(|f| {
        f.name.ends_with("_free") && f.params.first().is_some_and(|p| p.role == Role::Handle)
    })
}

/// The methods of the handle, in declaration order.
pub(crate) fn methods(api: &Api) -> Vec<&FunctionDef> {
    api.functions
        .iter()
        .filter(|f| {
            f.params.first().is_some_and(|p| p.role == Role::Handle) && !f.name.ends_with("_free")
        })
        .collect()
}

/// The snake-case method name of a handle function: `tsp_context_settings`
/// becomes `settings`, `tsp_chart_compute` becomes `chart_compute`.
pub(crate) fn method_name(api: &Api, opaque: &str, f: &FunctionDef) -> String {
    let handle_prefix = format!("{}{}_", api.prefix, snake(&binding_name(api, opaque)));
    f.name
        .strip_prefix(&handle_prefix)
        .map_or_else(|| strip_prefix(&f.name, &api.prefix), ToString::to_string)
}

/// The struct behind a pointer parameter, if it points at one.
pub(crate) fn pointee_struct(p: &ParamDef) -> Option<&str> {
    match &p.ty {
        TypeRef::Pointer { to, .. } => match &**to {
            TypeRef::Struct { name } => Some(name),
            _ => None,
        },
        _ => None,
    }
}

/// The opaque type behind a handle parameter, through one or two pointers.
pub(crate) fn pointee_opaque(p: &ParamDef) -> Option<&str> {
    fn walk(ty: &TypeRef) -> Option<&str> {
        match ty {
            TypeRef::Pointer { to, .. } => walk(to),
            TypeRef::Opaque { name } => Some(name),
            _ => None,
        }
    }
    walk(&p.ty)
}

/// A callback's shape: its value arguments and the struct it fills.
pub(crate) struct CallbackShape<'a> {
    /// The arguments the host function receives, in order.
    pub args: Vec<&'a ParamDef>,
    /// The struct the host function returns, by name.
    pub out: &'a str,
    /// The parameter that receives it.
    pub out_param: &'a ParamDef,
}

/// The shape of a callback, `None` when it fills no struct.
pub(crate) fn callback_shape(c: &CallbackDef) -> Option<CallbackShape<'_>> {
    let out_param = c.params.iter().find(|p| p.role == Role::StructOut)?;
    let out = pointee_struct(out_param)?;
    Some(CallbackShape {
        args: c.params.iter().filter(|p| p.role == Role::Value).collect(),
        out,
        out_param,
    })
}

/// The callback the provider vtable carries, with the vtable and its field.
pub(crate) fn provider_callback(api: &Api) -> Option<(&StructDef, &FieldDef, &CallbackDef)> {
    let vtable = api.structs.iter().find(|s| is_vtable(s))?;
    let field = vtable
        .fields
        .iter()
        .find(|f| matches!(f.ty, TypeRef::Callback { .. }))?;
    let TypeRef::Callback { name } = &field.ty else {
        return None;
    };
    let callback = api.callbacks.iter().find(|c| &c.name == name)?;
    Some((vtable, field, callback))
}

/// The binding-facing name of the provider function type: `PositionProvider`.
pub(crate) fn provider_type_name(api: &Api, c: &CallbackDef) -> String {
    format!(
        "{}Provider",
        binding_name(api, &c.name).trim_end_matches("Fn")
    )
}

/// The status enum: the one with an `Ok` value.
pub(crate) fn status_enum(api: &Api) -> Option<&EnumDef> {
    api.enums
        .iter()
        .find(|e| e.values.iter().any(|v| v.name == "Ok"))
}

/// The function returning a static C string for a status, if any.
pub(crate) fn message_function(api: &Api) -> Option<&FunctionDef> {
    api.functions
        .iter()
        .find(|f| matches!(&f.returns, Some(TypeRef::Pointer { to, .. }) if matches!(**to, TypeRef::Char)))
}

/// The function returning the provider's last error code, if any.
pub(crate) fn provider_code_function(api: &Api) -> Option<&FunctionDef> {
    api.functions
        .iter()
        .find(|f| f.name.ends_with("last_provider_code"))
}

/// Whether a function reports through the status enum.
pub(crate) fn returns_status(f: &FunctionDef) -> bool {
    matches!(&f.returns, Some(TypeRef::Enum { .. }))
}

/// The scalar a function returns, if it returns one.
pub(crate) fn returned_scalar(f: &FunctionDef) -> Option<&str> {
    match &f.returns {
        Some(TypeRef::Scalar { name }) => Some(name),
        _ => None,
    }
}

/// How a field's metadata is spelled inside its documentation.
#[derive(Clone, Copy)]
pub(crate) enum DocStyle {
    /// `Unit: deg. Range: [1,5]. Example: 3.` on one line.
    Prose,
    /// `@unit deg`, `@range [1,5]`, `@example 3`, one per line.
    JsDoc,
}

/// A field's documentation with its unit, range and example appended.
pub(crate) fn field_doc(f: &FieldDef, style: DocStyle) -> String {
    let mut text = f.doc.clone();
    let tags = [
        ("unit", &f.meta.unit),
        ("range", &f.meta.range),
        ("example", &f.meta.example),
    ];
    let mut first = true;
    for (tag, value) in tags {
        let Some(value) = value else { continue };
        match style {
            DocStyle::Prose => {
                let separator = if first { "\n" } else { " " };
                let label = format!("{}{}", tag[..1].to_ascii_uppercase(), &tag[1..]);
                let _ = write!(text, "{separator}{label}: {value}.");
            }
            DocStyle::JsDoc => {
                let _ = write!(text, "\n@{tag} {value}");
            }
        }
        first = false;
    }
    text
}

/// A comment where every line carries the same marker, as in `/// ` for
/// Rust and Dart.
pub(crate) fn line_comment(text: &str, indent: &str, marker: &str) -> String {
    let mut out = String::new();
    for line in text.lines() {
        let _ = writeln!(out, "{indent}{marker}{line}");
    }
    out
}

/// A `/** … */` block, as in C and JavaScript documentation comments; empty
/// text renders nothing.
pub(crate) fn block_comment(text: &str, indent: &str) -> String {
    if text.trim().is_empty() {
        return String::new();
    }
    let mut out = format!("{indent}/**\n");
    for line in text.lines() {
        let line = line.replace("*/", "* /");
        if line.is_empty() {
            let _ = writeln!(out, "{indent} *");
        } else {
            let _ = writeln!(out, "{indent} * {line}");
        }
    }
    let _ = writeln!(out, "{indent} */");
    out
}

/// Whether the host writes a struct: a `StructIn` parameter or a callback's
/// out struct. Only such structs need a "to C" conversion.
pub(crate) fn is_written_by_host(api: &Api, s: &StructDef) -> bool {
    let as_input = api
        .functions
        .iter()
        .flat_map(|f| f.params.iter())
        .any(|p| p.role == Role::StructIn && pointee_struct(p) == Some(s.name.as_str()));
    let as_callback_out = api
        .callbacks
        .iter()
        .any(|c| callback_shape(c).is_some_and(|shape| shape.out == s.name));
    as_input || as_callback_out
}

/// Whether the host reads a struct back: a `StructOut` parameter. Only such
/// structs need a "from C" conversion.
pub(crate) fn is_read_by_host(api: &Api, s: &StructDef) -> bool {
    api.functions
        .iter()
        .flat_map(|f| f.params.iter())
        .any(|p| p.role == Role::StructOut && pointee_struct(p) == Some(s.name.as_str()))
}
