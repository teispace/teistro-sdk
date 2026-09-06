//! How the description is read: a parameter's role from its type and
//! name, a field's visibility, the status enum, the constructor and the
//! methods of a handle. Every emitter asks here, so a classification is
//! made once and every binding agrees (ADR-0007: "roles from types and
//! names").

use crate::model::{
    Api, EnumDef, FieldDef, FunctionDef, OpaqueDef, ParamDef, Role, StructDef, StructRole, TypeRef,
};
use crate::names;

/// The one field every boundary struct carries for the size handshake.
pub const STRUCT_SIZE_FIELD: &str = "struct_size";

/// Whether a field is the size handshake.
#[must_use]
pub fn is_struct_size(field: &FieldDef) -> bool {
    field.name == STRUCT_SIZE_FIELD
}

/// Whether a field is padding.
#[must_use]
pub fn is_reserved(field: &FieldDef) -> bool {
    field.name.starts_with("reserved")
}

/// A field a binding shows: not the handshake, not padding, and not the
/// bookkeeping another field's role absorbs (an element count, a presence
/// flag).
#[must_use]
pub fn is_visible(field: &FieldDef) -> bool {
    !is_struct_size(field) && !is_reserved(field)
}

/// What a struct field is, which decides how every binding marshals it.
/// Read once by [`field_roles`] so the C header, the TypeScript surface
/// and the napi glue agree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldRole {
    /// The `struct_size` handshake; a binding sets it and never shows it.
    Handshake,
    /// Padding; zero.
    Reserved,
    /// A boolean carried as an integer.
    Flag,
    /// A set of an enum's members, bit `n` for the member with value `n`.
    BitSet {
        /// The enum's Rust name.
        enum_name: String,
    },
    /// A pointer to `count` elements of a scalar or an enum.
    Array {
        /// The field holding the element count.
        count: String,
    },
    /// The element count of the array field that names it.
    Count {
        /// That field.
        of: String,
    },
    /// A NUL-terminated UTF-8 string.
    Text,
    /// A value present only when a flag field says so.
    Optional {
        /// The flag.
        flag: String,
    },
    /// The flag that says whether another field is present.
    Presence {
        /// That field.
        of: String,
    },
    /// A struct carried by value.
    Nested {
        /// Its Rust name.
        name: String,
    },
    /// A caller-allocated array the library or a provider writes into.
    Column,
    /// A fixed-size run of bytes carried in the struct itself (a hash).
    FixedBytes {
        /// How many.
        len: usize,
    },
    /// A scalar, or an integer standing for an enum.
    Value,
}

impl FieldRole {
    /// Whether a binding shows the field: the bookkeeping roles another
    /// field absorbs are not part of any binding's surface.
    #[must_use]
    pub const fn is_shown(&self) -> bool {
        !matches!(
            self,
            FieldRole::Handshake
                | FieldRole::Reserved
                | FieldRole::Count { .. }
                | FieldRole::Presence { .. }
        )
    }
}

/// Every field's role, in field order.
#[must_use]
pub fn field_roles(api: &Api, s: &StructDef) -> Vec<FieldRole> {
    let counts: Vec<(&str, &str)> = s
        .fields
        .iter()
        .filter_map(|f| f.meta.len.as_deref().map(|len| (len, f.name.as_str())))
        .collect();
    let presences: Vec<(&str, &str)> = s
        .fields
        .iter()
        .filter_map(|f| {
            f.meta
                .present_if
                .as_deref()
                .map(|flag| (flag, f.name.as_str()))
        })
        .collect();
    s.fields
        .iter()
        .map(|f| field_role(api, s, f, &counts, &presences))
        .collect()
}

fn field_role(
    api: &Api,
    s: &StructDef,
    f: &FieldDef,
    counts: &[(&str, &str)],
    presences: &[(&str, &str)],
) -> FieldRole {
    if is_struct_size(f) {
        return FieldRole::Handshake;
    }
    if is_reserved(f) {
        return FieldRole::Reserved;
    }
    if let Some((_, of)) = counts.iter().find(|(len, _)| *len == f.name) {
        return FieldRole::Count {
            of: (*of).to_string(),
        };
    }
    if let Some((_, of)) = presences.iter().find(|(flag, _)| *flag == f.name) {
        return FieldRole::Presence {
            of: (*of).to_string(),
        };
    }
    if let Some(flag) = &f.meta.present_if {
        return FieldRole::Optional { flag: flag.clone() };
    }
    if let Some(enum_name) = &f.meta.bitset {
        return FieldRole::BitSet {
            enum_name: enum_name.clone(),
        };
    }
    if f.meta.flag {
        return FieldRole::Flag;
    }
    if let Some(count) = &f.meta.len {
        return FieldRole::Array {
            count: count.clone(),
        };
    }
    match &f.ty {
        TypeRef::Pointer { to, mutable } => match &**to {
            TypeRef::Char => FieldRole::Text,
            _ if *mutable && s.role == StructRole::Columns => FieldRole::Column,
            _ => FieldRole::Value,
        },
        TypeRef::Array { of, len }
            if matches!(
                **of,
                TypeRef::Scalar {
                    scalar: crate::model::Scalar::U8
                }
            ) =>
        {
            FieldRole::FixedBytes { len: *len }
        }
        TypeRef::Struct { name } => {
            let _ = api;
            FieldRole::Nested { name: name.clone() }
        }
        _ => FieldRole::Value,
    }
}

/// Whether a struct carries `struct_size`.
#[must_use]
pub fn has_handshake(s: &StructDef) -> bool {
    s.fields.iter().any(is_struct_size)
}

/// Whether a parameter name marks an output (`out_context`, `out`).
#[must_use]
pub fn is_out_name(name: &str) -> bool {
    name == "out" || name.starts_with("out_")
}

/// Whether a parameter name marks an element count of the pointer before
/// it (`len`, `jd_count`, `bytes_len`).
#[must_use]
pub fn is_length_name(name: &str) -> bool {
    name == "len" || name == "count" || name.ends_with("_len") || name.ends_with("_count")
}

/// The role of a parameter, from its type, its name and the description's
/// knowledge of what its pointee is. `previous` is the parameter before
/// it, which decides whether a count is a `Length`.
#[must_use]
pub fn infer_role(api: &Api, param_name: &str, ty: &TypeRef, previous: Option<&ParamDef>) -> Role {
    match ty {
        TypeRef::Pointer { to, mutable } => match &**to {
            TypeRef::Pointer { to: inner, .. } if matches!(**inner, TypeRef::Opaque { .. }) => {
                Role::HandleOut
            }
            TypeRef::Opaque { .. } => Role::Handle,
            TypeRef::Void => Role::UserData,
            TypeRef::Char => Role::StringIn,
            TypeRef::Struct { name } => {
                let role = api.struct_named(name).map(|s| s.role).unwrap_or_default();
                let out = is_out_name(param_name);
                match role {
                    StructRole::Vtable => Role::VtableIn,
                    StructRole::Blob if out => Role::BlobOut,
                    StructRole::Blob => Role::BlobFree,
                    StructRole::OwnedString if out => Role::StringOut,
                    StructRole::OwnedString => Role::StringFree,
                    StructRole::BorrowedString => Role::StrOut,
                    StructRole::Object | StructRole::Columns if out => Role::StructOut,
                    StructRole::Object | StructRole::Columns => Role::StructIn,
                }
            }
            TypeRef::Scalar { scalar } if *mutable && is_out_name(param_name) => {
                let _ = scalar;
                Role::ScalarOut
            }
            TypeRef::Scalar {
                scalar: crate::model::Scalar::U8,
            } if !*mutable => Role::BytesIn,
            TypeRef::Scalar { .. } | TypeRef::Enum { .. } if !*mutable => Role::ArrayIn,
            _ => Role::Value,
        },
        TypeRef::Scalar { .. }
            if is_length_name(param_name)
                && previous.is_some_and(|p| matches!(p.role, Role::BytesIn | Role::ArrayIn)) =>
        {
            Role::Length
        }
        _ => Role::Value,
    }
}

/// The struct behind a pointer parameter, if it points at one.
#[must_use]
pub fn pointee_struct<'a>(api: &'a Api, p: &ParamDef) -> Option<&'a StructDef> {
    match p.ty.pointee()? {
        TypeRef::Struct { name } => api.struct_named(name),
        _ => None,
    }
}

/// The opaque type behind a handle parameter, through one or two pointers.
#[must_use]
pub fn pointee_opaque<'a>(api: &'a Api, p: &ParamDef) -> Option<&'a OpaqueDef> {
    fn walk(ty: &TypeRef) -> Option<&str> {
        match ty {
            TypeRef::Pointer { to, .. } => walk(to),
            TypeRef::Opaque { name } => Some(name),
            _ => None,
        }
    }
    api.opaque_named(walk(&p.ty)?)
}

/// A function with no handle in play: a static function of the module.
#[must_use]
pub fn is_free_function(f: &FunctionDef) -> bool {
    !f.params
        .iter()
        .any(|p| matches!(p.role, Role::Handle | Role::HandleOut))
}

/// The function that creates an opaque type's handle.
#[must_use]
pub fn constructor<'a>(api: &'a Api, opaque: &OpaqueDef) -> Option<&'a FunctionDef> {
    api.functions.iter().find(|f| {
        f.params.iter().any(|p| {
            p.role == Role::HandleOut
                && pointee_opaque(api, p).is_some_and(|o| o.name == opaque.name)
        })
    })
}

/// The function that frees an opaque type's handle.
#[must_use]
pub fn destructor<'a>(api: &'a Api, opaque: &OpaqueDef) -> Option<&'a FunctionDef> {
    api.functions.iter().find(|f| {
        f.name.ends_with("_free")
            && f.params.len() == 1
            && f.params.first().is_some_and(|p| {
                p.role == Role::Handle
                    && pointee_opaque(api, p).is_some_and(|o| o.name == opaque.name)
            })
    })
}

/// The methods of an opaque type, in declaration order: every function
/// whose first parameter is its handle, the destructor excepted.
#[must_use]
pub fn methods<'a>(api: &'a Api, opaque: &OpaqueDef) -> Vec<&'a FunctionDef> {
    api.functions
        .iter()
        .filter(|f| {
            f.params.first().is_some_and(|p| {
                p.role == Role::Handle
                    && pointee_opaque(api, p).is_some_and(|o| o.name == opaque.name)
            }) && !f.name.ends_with("_free")
        })
        .collect()
}

/// The binding-facing method name of a handle function.
#[must_use]
pub fn method_name(api: &Api, opaque: &OpaqueDef, f: &FunctionDef) -> String {
    names::method_name(&api.prefix, &opaque.name, &f.name)
}

/// The status enum: the one with an `Ok` member at zero.
#[must_use]
pub fn status_enum(api: &Api) -> Option<&EnumDef> {
    api.enums
        .iter()
        .find(|e| e.values.iter().any(|v| v.name == "Ok" && v.value == 0))
}

/// Whether a function reports through the status enum.
#[must_use]
pub fn returns_status(api: &Api, f: &FunctionDef) -> bool {
    match (&f.returns, status_enum(api)) {
        (Some(TypeRef::Enum { name }), Some(status)) => *name == status.name,
        _ => false,
    }
}

/// The function returning a static C string for a status, if any.
#[must_use]
pub fn message_function(api: &Api) -> Option<&FunctionDef> {
    let status = status_enum(api)?;
    api.functions.iter().find(|f| {
        matches!(&f.returns, Some(TypeRef::Pointer { to, .. }) if matches!(**to, TypeRef::Char))
            && f.params.len() == 1
            && f.params
                .first()
                .is_some_and(|p| matches!(&p.ty, TypeRef::Enum { name } if *name == status.name))
    })
}

/// The scalar a function returns, if it returns one.
#[must_use]
pub fn returned_scalar(f: &FunctionDef) -> Option<crate::model::Scalar> {
    f.returns.as_ref().and_then(TypeRef::as_scalar)
}

/// Whether the host writes a struct: a `StructIn` parameter or a
/// callback's out struct. Only such structs need a "to C" conversion in a
/// binding.
#[must_use]
pub fn is_written_by_host(api: &Api, s: &StructDef) -> bool {
    let as_input = api.functions.iter().flat_map(|f| f.params.iter()).any(|p| {
        p.role == Role::StructIn && pointee_struct(api, p).is_some_and(|t| t.name == s.name)
    });
    let as_callback_out = api.callbacks.iter().flat_map(|c| c.params.iter()).any(|p| {
        p.role == Role::StructOut && pointee_struct(api, p).is_some_and(|t| t.name == s.name)
    });
    as_input || as_callback_out || is_nested_in_written(api, s)
}

/// Whether the host reads a struct back: a `StructOut` parameter of a
/// function, or a struct nested in one.
#[must_use]
pub fn is_read_by_host(api: &Api, s: &StructDef) -> bool {
    api.functions.iter().flat_map(|f| f.params.iter()).any(|p| {
        p.role == Role::StructOut && pointee_struct(api, p).is_some_and(|t| t.name == s.name)
    }) || api.structs.iter().any(|outer| {
        outer.name != s.name && contains_struct(outer, &s.name) && is_read_by_host(api, outer)
    })
}

fn is_nested_in_written(api: &Api, s: &StructDef) -> bool {
    api.structs.iter().any(|outer| {
        outer.name != s.name && contains_struct(outer, &s.name) && is_written_by_host(api, outer)
    })
}

fn contains_struct(outer: &StructDef, inner: &str) -> bool {
    outer.fields.iter().any(|f| match &f.ty {
        TypeRef::Struct { name } => name == inner,
        TypeRef::Array { of, .. } => matches!(&**of, TypeRef::Struct { name } if name == inner),
        _ => false,
    })
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::expect_used,
        clippy::too_many_lines,
        reason = "tests fail by panicking; one description is built by hand"
    )]

    use super::*;
    use crate::model::{Meta, Scalar};

    fn api() -> Api {
        let mut api = Api {
            schema: crate::model::SCHEMA.to_string(),
            abi_version: 1,
            sdk_version: "0.0.0".into(),
            prefix: "ts_".into(),
            sources: vec![],
            constants: vec![],
            enums: vec![],
            opaques: vec![OpaqueDef {
                name: "TsContext".into(),
                doc: String::new(),
                source: String::new(),
            }],
            callbacks: vec![],
            structs: vec![
                StructDef {
                    name: "TsBlob".into(),
                    doc: String::new(),
                    role: StructRole::Blob,
                    fields: vec![],
                    source: String::new(),
                },
                StructDef {
                    name: "ProviderVtable".into(),
                    doc: String::new(),
                    role: StructRole::Vtable,
                    fields: vec![],
                    source: String::new(),
                },
                StructDef {
                    name: "TsOptions".into(),
                    doc: String::new(),
                    role: StructRole::Object,
                    fields: vec![],
                    source: String::new(),
                },
            ],
            functions: vec![],
            blobs: vec![],
        };
        let opaque = TypeRef::Opaque {
            name: "TsContext".into(),
        };
        let param = |name: &str, ty: TypeRef, previous: Option<&ParamDef>| ParamDef {
            role: infer_role(&api, name, &ty, previous),
            name: name.into(),
            ty,
            meta: Meta::default(),
        };
        let bytes = param(
            "bytes",
            TypeRef::pointer(TypeRef::scalar(Scalar::U8), false),
            None,
        );
        let len = param("len", TypeRef::scalar(Scalar::Usize), Some(&bytes));
        let functions = vec![
            FunctionDef {
                name: "ts_context_new".into(),
                doc: String::new(),
                safety: None,
                params: vec![
                    param(
                        "options",
                        TypeRef::pointer(
                            TypeRef::Struct {
                                name: "TsOptions".into(),
                            },
                            false,
                        ),
                        None,
                    ),
                    param(
                        "provider",
                        TypeRef::pointer(
                            TypeRef::Struct {
                                name: "ProviderVtable".into(),
                            },
                            false,
                        ),
                        None,
                    ),
                    param("user_data", TypeRef::pointer(TypeRef::Void, true), None),
                    param(
                        "out_context",
                        TypeRef::pointer(TypeRef::pointer(opaque.clone(), true), true),
                        None,
                    ),
                ],
                returns: None,
                meta: Meta::default(),
                source: String::new(),
            },
            FunctionDef {
                name: "ts_context_free".into(),
                doc: String::new(),
                safety: None,
                params: vec![param(
                    "context",
                    TypeRef::pointer(opaque.clone(), true),
                    None,
                )],
                returns: None,
                meta: Meta::default(),
                source: String::new(),
            },
            FunctionDef {
                name: "ts_positions".into(),
                doc: String::new(),
                safety: None,
                params: vec![
                    param("context", TypeRef::pointer(opaque.clone(), false), None),
                    param("key", TypeRef::pointer(TypeRef::Char, false), None),
                    bytes.clone(),
                    len,
                    param(
                        "out_id",
                        TypeRef::pointer(TypeRef::scalar(Scalar::U32), true),
                        None,
                    ),
                    param(
                        "out_blob",
                        TypeRef::pointer(
                            TypeRef::Struct {
                                name: "TsBlob".into(),
                            },
                            true,
                        ),
                        None,
                    ),
                ],
                returns: None,
                meta: Meta::default(),
                source: String::new(),
            },
            FunctionDef {
                name: "ts_blob_free".into(),
                doc: String::new(),
                safety: None,
                params: vec![param(
                    "blob",
                    TypeRef::pointer(
                        TypeRef::Struct {
                            name: "TsBlob".into(),
                        },
                        true,
                    ),
                    None,
                )],
                returns: None,
                meta: Meta::default(),
                source: String::new(),
            },
        ];
        api.functions = functions;
        api
    }

    #[test]
    fn roles_come_from_types_and_names() {
        let api = api();
        let roles: Vec<Vec<Role>> = api
            .functions
            .iter()
            .map(|f| f.params.iter().map(|p| p.role).collect())
            .collect();
        assert_eq!(
            roles,
            [
                vec![
                    Role::StructIn,
                    Role::VtableIn,
                    Role::UserData,
                    Role::HandleOut
                ],
                vec![Role::Handle],
                vec![
                    Role::Handle,
                    Role::StringIn,
                    Role::BytesIn,
                    Role::Length,
                    Role::ScalarOut,
                    Role::BlobOut
                ],
                vec![Role::BlobFree],
            ]
        );
    }

    #[test]
    fn handles_have_a_constructor_a_destructor_and_methods() {
        let api = api();
        let context = api.opaque_named("TsContext").expect("the opaque");
        assert_eq!(
            constructor(&api, context).map(|f| f.name.as_str()),
            Some("ts_context_new")
        );
        assert_eq!(
            destructor(&api, context).map(|f| f.name.as_str()),
            Some("ts_context_free")
        );
        let names: Vec<&str> = methods(&api, context)
            .iter()
            .map(|f| f.name.as_str())
            .collect();
        assert_eq!(names, ["ts_positions"]);
        let positions = api.function_named("ts_positions").expect("the function");
        assert_eq!(method_name(&api, context, positions), "positions");
        assert!(is_free_function(
            api.function_named("ts_blob_free").expect("the function")
        ));
        assert!(!is_free_function(positions));
        assert!(is_written_by_host(
            &api,
            api.struct_named("TsOptions").expect("the struct")
        ));
        assert!(!is_read_by_host(
            &api,
            api.struct_named("TsOptions").expect("the struct")
        ));
    }
}
