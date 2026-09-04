//! Extracts the API description from the Rust source of the C ABI crate.
//!
//! The source of truth is the Rust file: `#[unsafe(no_mangle)] extern "C"`
//! functions, `#[repr(C)]` structs, `#[repr(int)]` enums, function-pointer
//! type aliases, the opaque handle struct, and the blob schema constants.
//! Roles are inferred from types and naming conventions (an `out_` prefix,
//! a `Vtable` suffix, `user_data`), the same rules Teimeris's extractor
//! applies to its C headers; units, ranges, examples and enum links come
//! from the `api:` line of a doc comment.

use std::collections::BTreeMap;

use syn::{Attribute, Expr, Fields, File, Item, Lit, Meta as SynMeta, Type};

use crate::model::{
    Api, BlobSchema, CallbackDef, ColumnDef, EnumDef, EnumValue, FieldDef, FunctionDef, Meta,
    OpaqueDef, ParamDef, Role, SectionSchema, StructDef, TypeRef,
};

/// Extracts the description from one Rust source file.
///
/// # Errors
///
/// When the file does not parse, or a schema constant is missing.
pub(crate) fn extract(source_path: &str, source: &str) -> Result<Api, String> {
    let file: File = syn::parse_file(source).map_err(|err| format!("{source_path}: {err}"))?;
    let mut consts: BTreeMap<String, ConstValue> = BTreeMap::new();
    let mut enums = Vec::new();
    let mut structs = Vec::new();
    let mut callbacks = Vec::new();
    let mut opaques = Vec::new();
    let mut raw_functions = Vec::new();

    for item in &file.items {
        match item {
            Item::Const(c) => {
                if let Some(value) = const_value(&c.expr) {
                    consts.insert(c.ident.to_string(), value);
                }
            }
            Item::Enum(e) => {
                if let Some(repr) = repr_of(&e.attrs) {
                    let (doc, _) = doc_of(&e.attrs);
                    let values = e
                        .variants
                        .iter()
                        .map(|v| {
                            let (doc, _) = doc_of(&v.attrs);
                            EnumValue {
                                name: v.ident.to_string(),
                                value: v
                                    .discriminant
                                    .as_ref()
                                    .and_then(|(_, expr)| int_value(expr))
                                    .unwrap_or_default(),
                                doc,
                            }
                        })
                        .collect();
                    enums.push(EnumDef {
                        name: e.ident.to_string(),
                        repr,
                        doc,
                        values,
                    });
                }
            }
            Item::Struct(s) => {
                let (doc, meta) = doc_of(&s.attrs);
                if repr_of(&s.attrs).as_deref() == Some("C") {
                    let fields = match &s.fields {
                        Fields::Named(named) => named
                            .named
                            .iter()
                            .map(|f| {
                                let (doc, meta) = doc_of(&f.attrs);
                                FieldDef {
                                    name: f
                                        .ident
                                        .as_ref()
                                        .map(ToString::to_string)
                                        .unwrap_or_default(),
                                    ty: type_ref(&f.ty),
                                    doc,
                                    meta,
                                }
                            })
                            .collect(),
                        _ => Vec::new(),
                    };
                    structs.push(StructDef {
                        name: s.ident.to_string(),
                        doc,
                        handshake: meta.handshake.clone(),
                        blob: meta.blob.clone(),
                        fields,
                    });
                } else if matches!(s.vis, syn::Visibility::Public(_))
                    && has_private_fields(&s.fields)
                {
                    opaques.push(OpaqueDef {
                        name: s.ident.to_string(),
                        doc,
                    });
                }
            }
            Item::Type(t) => {
                if let Type::FnPtr(bare) = &*t.ty {
                    let (doc, _) = doc_of(&t.attrs);
                    let params = bare
                        .inputs
                        .iter()
                        .map(|arg| {
                            let name = arg
                                .name
                                .as_ref()
                                .map(|(ident, _)| ident.to_string())
                                .unwrap_or_default();
                            let ty = type_ref(&arg.ty);
                            ParamDef {
                                role: infer_role(&name, &ty),
                                name,
                                ty,
                            }
                        })
                        .collect();
                    let returns = match &bare.output {
                        syn::ReturnType::Default => TypeRef::Void,
                        syn::ReturnType::Type(_, ty) => type_ref(ty),
                    };
                    callbacks.push(CallbackDef {
                        name: t.ident.to_string(),
                        doc,
                        params,
                        returns,
                    });
                }
            }
            Item::Fn(f) if is_exported(&f.attrs) => raw_functions.push(f),
            _ => {}
        }
    }

    let functions = raw_functions
        .into_iter()
        .map(|f| {
            let (doc, _) = doc_of(&f.attrs);
            let (doc, safety) = split_safety(&doc);
            let params = f
                .sig
                .inputs
                .iter()
                .filter_map(|input| match input {
                    syn::FnArg::Typed(pat) => {
                        let name = match &*pat.pat {
                            syn::Pat::Ident(ident) => ident.ident.to_string(),
                            _ => String::from("_"),
                        };
                        let ty = type_ref(&pat.ty);
                        Some(ParamDef {
                            role: infer_role(&name, &ty),
                            name,
                            ty,
                        })
                    }
                    syn::FnArg::Receiver(_) => None,
                })
                .collect();
            let returns = match &f.sig.output {
                syn::ReturnType::Default => None,
                syn::ReturnType::Type(_, ty) => Some(type_ref(ty)),
            };
            FunctionDef {
                name: f.sig.ident.to_string(),
                doc,
                safety: if matches!(f.sig.safety, syn::Safety::Unsafe(_)) {
                    Some(safety)
                } else {
                    None
                },
                params,
                returns,
            }
        })
        .collect();

    let abi_version = consts
        .get("TSP_ABI_VERSION")
        .and_then(ConstValue::as_int)
        .ok_or("TSP_ABI_VERSION is missing")?;
    let blob = BlobSchema {
        magic: consts
            .get("TSPB_MAGIC")
            .and_then(ConstValue::as_int)
            .ok_or("TSPB_MAGIC is missing")?,
        version: consts
            .get("TSPB_VERSION")
            .and_then(ConstValue::as_int)
            .ok_or("TSPB_VERSION is missing")?,
        sections: vec![
            section(&consts, 1, "chart", "fixed", "CHART_HEADER_FIELDS")?,
            section(&consts, 2, "positions", "columns", "POSITION_COLUMNS")?,
            section(&consts, 3, "dasha", "columns", "DASHA_COLUMNS")?,
        ],
    };

    let mut api = Api {
        abi_version,
        prefix: String::from("tsp_"),
        source: source_path.to_string(),
        enums,
        structs,
        callbacks,
        opaques,
        functions,
        blob,
    };
    resolve_names(&mut api);
    Ok(api)
}

/// A constant the extractor understands: an integer or an array of strings.
#[derive(Debug, Clone)]
enum ConstValue {
    Int(u32),
    Strings(Vec<String>),
}

impl ConstValue {
    fn as_int(&self) -> Option<u32> {
        match self {
            ConstValue::Int(v) => Some(*v),
            ConstValue::Strings(_) => None,
        }
    }
}

fn const_value(expr: &Expr) -> Option<ConstValue> {
    match expr {
        Expr::Lit(lit) => match &lit.lit {
            Lit::Int(int) => int.base10_parse::<u32>().ok().map(ConstValue::Int),
            _ => None,
        },
        Expr::Array(array) => {
            let strings: Option<Vec<String>> = array
                .elems
                .iter()
                .map(|e| match e {
                    Expr::Lit(lit) => match &lit.lit {
                        Lit::Str(s) => Some(s.value()),
                        _ => None,
                    },
                    _ => None,
                })
                .collect();
            strings.map(ConstValue::Strings)
        }
        _ => None,
    }
}

fn section(
    consts: &BTreeMap<String, ConstValue>,
    id: u32,
    name: &str,
    kind: &str,
    const_name: &str,
) -> Result<SectionSchema, String> {
    let Some(ConstValue::Strings(entries)) = consts.get(const_name) else {
        return Err(format!(
            "{const_name} is missing or not an array of strings"
        ));
    };
    let fields = entries
        .iter()
        .map(|entry| {
            let (name, ty) = entry.split_once(':').unwrap_or((entry, "u8"));
            ColumnDef {
                name: name.to_string(),
                ty: ty.to_string(),
            }
        })
        .collect();
    Ok(SectionSchema {
        id,
        name: name.to_string(),
        kind: kind.to_string(),
        fields,
    })
}

fn int_value(expr: &Expr) -> Option<i64> {
    match expr {
        Expr::Lit(lit) => match &lit.lit {
            Lit::Int(int) => int.base10_parse::<i64>().ok(),
            _ => None,
        },
        Expr::Unary(unary) => match unary.op {
            syn::UnOp::Neg(_) => int_value(&unary.expr).map(|v| -v),
            _ => None,
        },
        _ => None,
    }
}

fn repr_of(attrs: &[Attribute]) -> Option<String> {
    attrs.iter().find_map(|attr| {
        if !attr.path().is_ident("repr") {
            return None;
        }
        let mut repr = None;
        let _ = attr.parse_nested_meta(|meta| {
            repr = meta.path.get_ident().map(ToString::to_string);
            Ok(())
        });
        repr
    })
}

fn is_exported(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| match &attr.meta {
        SynMeta::List(list) => {
            list.path.is_ident("unsafe") && list.tokens.to_string().contains("no_mangle")
        }
        SynMeta::Path(path) => path.is_ident("no_mangle"),
        SynMeta::NameValue(_) => false,
    })
}

fn has_private_fields(fields: &Fields) -> bool {
    match fields {
        Fields::Named(named) => named
            .named
            .iter()
            .any(|f| !matches!(f.vis, syn::Visibility::Public(_))),
        Fields::Unnamed(_) | Fields::Unit => false,
    }
}

/// The doc lines of an item, and the `api:` metadata pulled out of them.
fn doc_of(attrs: &[Attribute]) -> (String, Meta) {
    let mut lines = Vec::new();
    let mut meta = Meta::default();
    for attr in attrs {
        let SynMeta::NameValue(nv) = &attr.meta else {
            continue;
        };
        if !nv.path.is_ident("doc") {
            continue;
        }
        let Expr::Lit(lit) = &nv.value else { continue };
        let Lit::Str(s) = &lit.lit else { continue };
        let line = s.value();
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("`api:") {
            parse_meta(rest.trim_end_matches('`').trim(), &mut meta);
        } else {
            lines.push(trimmed.to_string());
        }
    }
    while lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
    (lines.join("\n"), meta)
}

fn parse_meta(text: &str, meta: &mut Meta) {
    for pair in text.split_whitespace() {
        let Some((key, value)) = pair.split_once('=') else {
            continue;
        };
        let value = Some(value.to_string());
        match key {
            "unit" => meta.unit = value,
            "range" => meta.range = value,
            "example" => meta.example = value,
            "enum" => meta.enum_name = value,
            "role" => meta.role = value,
            "callback" => meta.callback = value,
            "handshake" => meta.handshake = value,
            "blob" => meta.blob = value,
            _ => {}
        }
    }
}

fn split_safety(doc: &str) -> (String, String) {
    match doc.split_once("# Safety") {
        Some((before, after)) => (before.trim().to_string(), after.trim().to_string()),
        None => (doc.to_string(), String::new()),
    }
}

/// A first pass at a type; names are resolved to enum, struct, opaque or
/// callback once every item is known.
fn type_ref(ty: &Type) -> TypeRef {
    match ty {
        Type::Ptr(ptr) => TypeRef::Pointer {
            to: Box::new(type_ref(&ptr.elem)),
            mutable: matches!(ptr.mutability, syn::PointerMutability::Mut(_)),
        },
        Type::Path(path) => {
            let last = path.path.segments.last();
            let name = last.map(|s| s.ident.to_string()).unwrap_or_default();
            match name.as_str() {
                "c_void" => TypeRef::Void,
                "c_char" => TypeRef::Char,
                "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" | "f32" | "f64"
                | "usize" | "isize" | "bool" => TypeRef::Scalar { name },
                "Option" => {
                    let inner = last.and_then(|s| match &s.arguments {
                        syn::PathArguments::AngleBracketed(args) => {
                            args.args.first().and_then(|a| match a {
                                syn::GenericArgument::Type(Type::Path(p)) => {
                                    p.path.segments.last().map(|s| s.ident.to_string())
                                }
                                _ => None,
                            })
                        }
                        _ => None,
                    });
                    TypeRef::Callback {
                        name: inner.unwrap_or_default(),
                    }
                }
                _ => TypeRef::Struct { name },
            }
        }
        _ => TypeRef::Void,
    }
}

fn infer_role(name: &str, ty: &TypeRef) -> Role {
    match ty {
        TypeRef::Pointer { to, .. } => match &**to {
            TypeRef::Pointer { .. } => Role::HandleOut,
            TypeRef::Void => Role::UserData,
            TypeRef::Struct { name: target } => {
                if target.ends_with("Vtable") {
                    Role::VtableIn
                } else if target.ends_with("Blob") {
                    if name.starts_with("out_") {
                        Role::BlobOut
                    } else {
                        Role::BlobInOut
                    }
                } else if target.ends_with("Context") {
                    Role::Handle
                } else if name.starts_with("out_") {
                    Role::StructOut
                } else {
                    Role::StructIn
                }
            }
            _ => Role::Value,
        },
        _ => Role::Value,
    }
}

/// Turns `Struct { name }` placeholders into enum, opaque or callback
/// references now that every item is known.
fn resolve_names(api: &mut Api) {
    let enum_names: Vec<String> = api.enums.iter().map(|e| e.name.clone()).collect();
    let opaque_names: Vec<String> = api.opaques.iter().map(|o| o.name.clone()).collect();
    let callback_names: Vec<String> = api.callbacks.iter().map(|c| c.name.clone()).collect();
    let resolve = |ty: &mut TypeRef| resolve_type(ty, &enum_names, &opaque_names, &callback_names);
    for s in &mut api.structs {
        for f in &mut s.fields {
            resolve(&mut f.ty);
        }
    }
    for c in &mut api.callbacks {
        for p in &mut c.params {
            resolve(&mut p.ty);
        }
        resolve(&mut c.returns);
    }
    for f in &mut api.functions {
        for p in &mut f.params {
            resolve(&mut p.ty);
        }
        if let Some(r) = &mut f.returns {
            resolve(r);
        }
    }
}

fn resolve_type(ty: &mut TypeRef, enums: &[String], opaques: &[String], callbacks: &[String]) {
    match ty {
        TypeRef::Struct { name } => {
            if enums.contains(name) {
                *ty = TypeRef::Enum { name: name.clone() };
            } else if opaques.contains(name) {
                *ty = TypeRef::Opaque { name: name.clone() };
            } else if callbacks.contains(name) {
                *ty = TypeRef::Callback { name: name.clone() };
            }
        }
        TypeRef::Pointer { to, .. } => resolve_type(to, enums, opaques, callbacks),
        _ => {}
    }
}
