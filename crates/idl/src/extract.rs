//! The extractor: the API description from the Rust source of the boundary
//! crates. The source of truth is the Rust: `#[unsafe(no_mangle)] extern
//! "C"` functions, `#[repr(C)]` structs, `#[repr(int)]` enums,
//! function-pointer type aliases, the opaque handle types, and the
//! constants marked `api: constant`. Roles are inferred from types and
//! naming conventions ([`crate::rules`]); units, ranges, examples and
//! enum links come from the `api:` line of a doc comment (ADR-0023). An
//! unrecognised shape is an error, so the build fails rather than a
//! binding silently missing an entry point
//! (`docs/02-architecture/06-api-conventions.md`, rule 9).

use core::fmt;
use std::collections::BTreeSet;
use std::path::Path;

use syn::{Attribute, Expr, Fields, File, Item, Lit, Meta as SynMeta, Type};

use crate::model::{
    Api, BlobSchema, CallbackDef, ConstantDef, EnumDef, EnumValue, FieldDef, FunctionDef, Meta,
    OpaqueDef, ParamDef, Role, SCHEMA, Scalar, StructDef, StructRole, TypeRef,
};
use crate::names::clean_doc;
use crate::rules::infer_role;

/// One Rust source file to read.
#[derive(Debug, Clone)]
pub struct Source {
    /// The repository-relative path, recorded on every item.
    pub relative: String,
    /// The text.
    pub text: String,
}

impl Source {
    /// Reads a file.
    ///
    /// # Errors
    ///
    /// When the file cannot be read.
    pub fn read(root: &Path, relative: &str) -> Result<Source, ExtractError> {
        let text = std::fs::read_to_string(root.join(relative))
            .map_err(|e| ExtractError::new(relative, e.to_string()))?;
        Ok(Source {
            relative: relative.to_string(),
            text,
        })
    }
}

/// Everything the extractor needs beside the sources: the enums the
/// catalogue contributes (with their kinds), the blob schemas the boundary
/// crate declares, and the versions.
#[derive(Debug, Clone, Default)]
pub struct Inputs {
    /// The prefix every symbol carries.
    pub prefix: String,
    /// The SDK version.
    pub sdk_version: String,
    /// The name of the constant holding the ABI version.
    pub abi_version_constant: String,
    /// The catalogue's kinds and any other enum built outside the sources.
    pub extra_enums: Vec<EnumDef>,
    /// The blob schemas.
    pub blobs: Vec<BlobSchema>,
}

/// Why extraction failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractError {
    /// The source, or the item, the problem is in.
    pub where_: String,
    /// What is wrong.
    pub detail: String,
}

impl ExtractError {
    /// An error at a place.
    #[must_use]
    pub fn new(where_: &str, detail: impl Into<String>) -> ExtractError {
        ExtractError {
            where_: where_.to_string(),
            detail: detail.into(),
        }
    }
}

impl fmt::Display for ExtractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.where_, self.detail)
    }
}

impl std::error::Error for ExtractError {}

/// Extracts the description from the sources.
///
/// # Errors
///
/// A file that does not parse, a missing ABI version constant, a
/// reference to a type the sources do not define, an `api:` link to an
/// enum or blob that does not exist, or a parameter shape no role fits.
pub fn extract(sources: &[Source], inputs: &Inputs) -> Result<Api, ExtractError> {
    let mut api = Api {
        schema: SCHEMA.to_string(),
        abi_version: 0,
        sdk_version: inputs.sdk_version.clone(),
        prefix: inputs.prefix.clone(),
        sources: sources.iter().map(|s| s.relative.clone()).collect(),
        constants: Vec::new(),
        enums: inputs.extra_enums.clone(),
        opaques: Vec::new(),
        callbacks: Vec::new(),
        structs: Vec::new(),
        functions: Vec::new(),
        blobs: inputs.blobs.clone(),
    };
    let mut abi_version = None;
    let mut candidates: Vec<OpaqueDef> = Vec::new();
    for source in sources {
        let file: File = syn::parse_file(&source.text)
            .map_err(|e| ExtractError::new(&source.relative, e.to_string()))?;
        for item in &file.items {
            collect(
                item,
                source,
                &mut api,
                &mut candidates,
                &inputs.abi_version_constant,
                &mut abi_version,
            )?;
        }
    }
    api.abi_version = abi_version.ok_or_else(|| {
        ExtractError::new(
            "sources",
            format!("`{}` is not defined", inputs.abi_version_constant),
        )
    })?;
    api.opaques = referenced_opaques(&api, candidates);
    resolve_names(&mut api)?;
    infer_roles(&mut api);
    check_links(&api)?;
    Ok(api)
}

fn collect(
    item: &Item,
    source: &Source,
    api: &mut Api,
    candidates: &mut Vec<OpaqueDef>,
    abi_constant: &str,
    abi_version: &mut Option<u32>,
) -> Result<(), ExtractError> {
    match item {
        Item::Const(c) if matches!(c.vis, syn::Visibility::Public(_)) => {
            collect_const(c, source, api, abi_constant, abi_version)
        }
        Item::Enum(e) if matches!(e.vis, syn::Visibility::Public(_)) => {
            collect_enum(e, source, api);
            Ok(())
        }
        Item::Struct(s) if matches!(s.vis, syn::Visibility::Public(_)) => {
            collect_struct(s, source, api, candidates)
        }
        Item::Type(t) if matches!(t.vis, syn::Visibility::Public(_)) => {
            collect_callback(t, source, api)
        }
        Item::Fn(f) if is_exported(&f.attrs) => collect_function(f, source, api),
        _ => Ok(()),
    }
}

fn collect_const(
    c: &syn::ItemConst,
    source: &Source,
    api: &mut Api,
    abi_constant: &str,
    abi_version: &mut Option<u32>,
) -> Result<(), ExtractError> {
    let (doc, _, flags) = doc_of(&c.attrs);
    let name = c.ident.to_string();
    let value = int_value(&c.expr);
    if name == abi_constant {
        *abi_version = value.and_then(|v| u32::try_from(v).ok());
    }
    if !flags.constant {
        return Ok(());
    }
    let ty = match &*c.ty {
        Type::Path(p) => p
            .path
            .segments
            .last()
            .and_then(|s| Scalar::from_rust(&s.ident.to_string())),
        _ => None,
    };
    let (Some(ty), Some(value)) = (ty, value) else {
        return Err(ExtractError::new(
            &source.relative,
            format!("constant `{name}` must be an integer scalar with a literal value"),
        ));
    };
    api.constants.push(ConstantDef {
        name,
        doc,
        ty,
        value,
        source: source.relative.clone(),
    });
    Ok(())
}

fn collect_enum(e: &syn::ItemEnum, source: &Source, api: &mut Api) {
    let Some(repr) = repr_of(&e.attrs).and_then(|r| Scalar::from_rust(&r)) else {
        return;
    };
    let (doc, _, _) = doc_of(&e.attrs);
    let mut next = 0i64;
    let values = e
        .variants
        .iter()
        .map(|v| {
            let (doc, _, flags) = doc_of(&v.attrs);
            let value = v
                .discriminant
                .as_ref()
                .and_then(|(_, expr)| int_value(expr))
                .unwrap_or(next);
            next = value + 1;
            EnumValue {
                name: v.ident.to_string(),
                value,
                doc,
                key: None,
                deprecated: flags.deprecated,
            }
        })
        .collect();
    api.enums.push(EnumDef {
        name: e.ident.to_string(),
        doc,
        repr,
        kind: None,
        values,
        source: source.relative.clone(),
    });
}

fn collect_struct(
    s: &syn::ItemStruct,
    source: &Source,
    api: &mut Api,
    candidates: &mut Vec<OpaqueDef>,
) -> Result<(), ExtractError> {
    let (doc, _, flags) = doc_of(&s.attrs);
    if repr_of(&s.attrs).as_deref() != Some("C") {
        if has_private_fields(&s.fields) {
            candidates.push(OpaqueDef {
                name: s.ident.to_string(),
                doc,
                source: source.relative.clone(),
            });
        }
        return Ok(());
    }
    let fields = match &s.fields {
        Fields::Named(named) => named
            .named
            .iter()
            .map(|f| {
                let (doc, meta, _) = doc_of(&f.attrs);
                type_ref(&f.ty, &source.relative).map(|ty| FieldDef {
                    name: f
                        .ident
                        .as_ref()
                        .map(ToString::to_string)
                        .unwrap_or_default(),
                    ty,
                    doc,
                    meta,
                })
            })
            .collect::<Result<Vec<_>, _>>()?,
        _ => Vec::new(),
    };
    let role = match flags.role.as_deref() {
        Some("owned_string") => StructRole::OwnedString,
        Some("borrowed_string") => StructRole::BorrowedString,
        Some("blob") => StructRole::Blob,
        Some("vtable") => StructRole::Vtable,
        Some(other) => {
            return Err(ExtractError::new(
                &source.relative,
                format!("struct `{}`: unknown role `{other}`", s.ident),
            ));
        }
        None if s.ident.to_string().ends_with("Vtable") => StructRole::Vtable,
        None => StructRole::Object,
    };
    api.structs.push(StructDef {
        name: s.ident.to_string(),
        doc,
        role,
        fields,
        source: source.relative.clone(),
    });
    Ok(())
}

fn collect_callback(t: &syn::ItemType, source: &Source, api: &mut Api) -> Result<(), ExtractError> {
    let Type::BareFn(bare) = &*t.ty else {
        return Ok(());
    };
    let (doc, _, _) = doc_of(&t.attrs);
    let params = bare
        .inputs
        .iter()
        .map(|arg| {
            let name = arg
                .name
                .as_ref()
                .map(|(ident, _)| ident.to_string())
                .unwrap_or_default();
            type_ref(&arg.ty, &source.relative).map(|ty| ParamDef {
                name,
                ty,
                role: Role::Value,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let returns = match &bare.output {
        syn::ReturnType::Default => TypeRef::Void,
        syn::ReturnType::Type(_, ty) => type_ref(ty, &source.relative)?,
    };
    api.callbacks.push(CallbackDef {
        name: t.ident.to_string(),
        doc,
        params,
        returns,
        source: source.relative.clone(),
    });
    Ok(())
}

fn collect_function(f: &syn::ItemFn, source: &Source, api: &mut Api) -> Result<(), ExtractError> {
    let (doc, meta, _) = doc_of(&f.attrs);
    let (doc, safety) = split_safety(&doc);
    let params = f
        .sig
        .inputs
        .iter()
        .filter_map(|input| match input {
            syn::FnArg::Typed(pat) => Some(pat),
            syn::FnArg::Receiver(_) => None,
        })
        .map(|pat| {
            let name = match &*pat.pat {
                syn::Pat::Ident(ident) => ident.ident.to_string(),
                _ => String::from("_"),
            };
            type_ref(&pat.ty, &source.relative).map(|ty| ParamDef {
                name,
                ty,
                role: Role::Value,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let returns = match &f.sig.output {
        syn::ReturnType::Default => None,
        syn::ReturnType::Type(_, ty) => Some(type_ref(ty, &source.relative)?),
    };
    api.functions.push(FunctionDef {
        name: f.sig.ident.to_string(),
        doc,
        safety: f.sig.unsafety.is_some().then_some(safety),
        params,
        returns,
        meta,
        source: source.relative.clone(),
    });
    Ok(())
}

/// The opaque candidates that some function or struct points at; a private
/// struct nobody references is an implementation detail.
fn referenced_opaques(api: &Api, candidates: Vec<OpaqueDef>) -> Vec<OpaqueDef> {
    let mut referenced = BTreeSet::new();
    let mut visit = |ty: &TypeRef| {
        let mut ty = ty;
        while let TypeRef::Pointer { to, .. } = ty {
            ty = to;
        }
        if let TypeRef::Struct { name } = ty {
            referenced.insert(name.clone());
        }
    };
    for f in &api.functions {
        f.params.iter().for_each(|p| visit(&p.ty));
        if let Some(r) = &f.returns {
            visit(r);
        }
    }
    for c in &api.callbacks {
        c.params.iter().for_each(|p| visit(&p.ty));
    }
    for s in &api.structs {
        s.fields.iter().for_each(|f| visit(&f.ty));
    }
    candidates
        .into_iter()
        .filter(|o| referenced.contains(&o.name))
        .collect()
}

/// Whether an item's doc lines ask for something beyond metadata.
#[derive(Debug, Default)]
struct Flags {
    constant: bool,
    deprecated: bool,
    role: Option<String>,
}

/// The doc lines of an item, the `api:` metadata pulled out of them, and
/// the flags; rustdoc links are flattened to plain text.
fn doc_of(attrs: &[Attribute]) -> (String, Meta, Flags) {
    let mut lines = Vec::new();
    let mut meta = Meta::default();
    let mut flags = Flags::default();
    for attr in attrs {
        if attr.path().is_ident("deprecated") {
            flags.deprecated = true;
            continue;
        }
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
            parse_meta(rest.trim_end_matches('`').trim(), &mut meta, &mut flags);
        } else {
            lines.push(clean_doc(trimmed));
        }
    }
    while lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
    while lines.first().is_some_and(String::is_empty) {
        lines.remove(0);
    }
    (lines.join("\n"), meta, flags)
}

fn parse_meta(text: &str, meta: &mut Meta, flags: &mut Flags) {
    for pair in text.split_whitespace() {
        match pair.split_once('=') {
            Some(("unit", value)) => meta.unit = Some(value.to_string()),
            Some(("range", value)) => meta.range = Some(value.to_string()),
            Some(("example", value)) => meta.example = Some(value.to_string()),
            Some(("enum", value)) => meta.enum_name = Some(value.to_string()),
            Some(("blob", value)) => meta.blob = Some(value.to_string()),
            Some(("since", value)) => meta.since = value.parse().ok(),
            Some(("role", value)) => flags.role = Some(value.to_string()),
            None if pair == "nullable" => meta.nullable = true,
            None if pair == "constant" => flags.constant = true,
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

/// A constant integer expression: a literal, a negation, or the shifts,
/// bit operations and sums a flag set is written with (`1 << 2`, `A | B`).
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
        Expr::Binary(binary) => {
            let (left, right) = (int_value(&binary.left)?, int_value(&binary.right)?);
            match binary.op {
                syn::BinOp::Shl(_) => u32::try_from(right).ok().and_then(|r| left.checked_shl(r)),
                syn::BinOp::Shr(_) => u32::try_from(right).ok().and_then(|r| left.checked_shr(r)),
                syn::BinOp::BitOr(_) => Some(left | right),
                syn::BinOp::BitAnd(_) => Some(left & right),
                syn::BinOp::Add(_) => left.checked_add(right),
                syn::BinOp::Sub(_) => left.checked_sub(right),
                syn::BinOp::Mul(_) => left.checked_mul(right),
                _ => None,
            }
        }
        Expr::Group(group) => int_value(&group.expr),
        Expr::Paren(paren) => int_value(&paren.expr),
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
            let name = meta.path.get_ident().map(ToString::to_string);
            if name
                .as_deref()
                .is_some_and(|n| n == "C" || Scalar::from_rust(n).is_some())
            {
                repr = name;
            }
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

/// A first pass at a type: every path that is not a scalar, `c_void`,
/// `c_char` or `Option<fn>` is a `Struct` placeholder resolved once every
/// item is known.
fn type_ref(ty: &Type, where_: &str) -> Result<TypeRef, ExtractError> {
    match ty {
        Type::Ptr(ptr) => Ok(TypeRef::Pointer {
            to: Box::new(type_ref(&ptr.elem, where_)?),
            mutable: ptr.mutability.is_some(),
        }),
        Type::Array(array) => {
            let len = int_value(&array.len)
                .and_then(|v| usize::try_from(v).ok())
                .ok_or_else(|| ExtractError::new(where_, "an array length must be a literal"))?;
            Ok(TypeRef::Array {
                of: Box::new(type_ref(&array.elem, where_)?),
                len,
            })
        }
        Type::Path(path) => {
            let last = path
                .path
                .segments
                .last()
                .ok_or_else(|| ExtractError::new(where_, "an empty type path"))?;
            let name = last.ident.to_string();
            if name == "c_void" {
                return Ok(TypeRef::Void);
            }
            if name == "c_char" {
                return Ok(TypeRef::Char);
            }
            if let Some(scalar) = Scalar::from_rust(&name) {
                return Ok(TypeRef::scalar(scalar));
            }
            if name == "Option" {
                let inner = match &last.arguments {
                    syn::PathArguments::AngleBracketed(args) => {
                        args.args.first().and_then(|a| match a {
                            syn::GenericArgument::Type(Type::Path(p)) => {
                                p.path.segments.last().map(|s| s.ident.to_string())
                            }
                            _ => None,
                        })
                    }
                    _ => None,
                };
                return inner.map(|name| TypeRef::Callback { name }).ok_or_else(|| {
                    ExtractError::new(
                        where_,
                        "`Option` at the boundary wraps a callback alias only",
                    )
                });
            }
            Ok(TypeRef::Struct { name })
        }
        other => Err(ExtractError::new(
            where_,
            format!("a type the boundary cannot carry: `{}`", quote_type(other)),
        )),
    }
}

fn quote_type(ty: &Type) -> String {
    use syn::__private::ToTokens;
    ty.to_token_stream().to_string()
}

/// Turns `Struct { name }` placeholders into enum, opaque or callback
/// references now that every item is known, and refuses an unknown name.
fn resolve_names(api: &mut Api) -> Result<(), ExtractError> {
    let enums: BTreeSet<String> = api.enums.iter().map(|e| e.name.clone()).collect();
    let opaques: BTreeSet<String> = api.opaques.iter().map(|o| o.name.clone()).collect();
    let callbacks: BTreeSet<String> = api.callbacks.iter().map(|c| c.name.clone()).collect();
    let structs: BTreeSet<String> = api.structs.iter().map(|s| s.name.clone()).collect();
    let resolve = |ty: &mut TypeRef, where_: &str| -> Result<(), ExtractError> {
        resolve_type(ty, &enums, &opaques, &callbacks, &structs, where_)
    };
    for s in &mut api.structs {
        for f in &mut s.fields {
            resolve(&mut f.ty, &format!("{}.{}", s.name, f.name))?;
        }
    }
    for c in &mut api.callbacks {
        for p in &mut c.params {
            resolve(&mut p.ty, &format!("{}({})", c.name, p.name))?;
        }
        resolve(&mut c.returns, &c.name)?;
    }
    for f in &mut api.functions {
        for p in &mut f.params {
            resolve(&mut p.ty, &format!("{}({})", f.name, p.name))?;
        }
        if let Some(r) = &mut f.returns {
            resolve(r, &f.name)?;
        }
    }
    Ok(())
}

fn resolve_type(
    ty: &mut TypeRef,
    enums: &BTreeSet<String>,
    opaques: &BTreeSet<String>,
    callbacks: &BTreeSet<String>,
    structs: &BTreeSet<String>,
    where_: &str,
) -> Result<(), ExtractError> {
    match ty {
        TypeRef::Struct { name } => {
            if enums.contains(name) {
                *ty = TypeRef::Enum { name: name.clone() };
            } else if opaques.contains(name) {
                *ty = TypeRef::Opaque { name: name.clone() };
            } else if callbacks.contains(name) {
                *ty = TypeRef::Callback { name: name.clone() };
            } else if !structs.contains(name) {
                return Err(ExtractError::new(
                    where_,
                    format!("`{name}` is not a boundary type the sources define"),
                ));
            }
            Ok(())
        }
        TypeRef::Callback { name } if !callbacks.contains(name) => Err(ExtractError::new(
            where_,
            format!("`{name}` is not a callback alias the sources define"),
        )),
        TypeRef::Pointer { to, .. } | TypeRef::Array { of: to, .. } => {
            resolve_type(to, enums, opaques, callbacks, structs, where_)
        }
        _ => Ok(()),
    }
}

/// Roles for every function and callback parameter, now that struct roles
/// and opaque types are known.
fn infer_roles(api: &mut Api) {
    let snapshot = api.clone();
    for f in &mut api.functions {
        let mut previous: Option<ParamDef> = None;
        for p in &mut f.params {
            p.role = infer_role(&snapshot, &p.name, &p.ty, previous.as_ref());
            previous = Some(p.clone());
        }
    }
    for c in &mut api.callbacks {
        let mut previous: Option<ParamDef> = None;
        for p in &mut c.params {
            p.role = infer_role(&snapshot, &p.name, &p.ty, previous.as_ref());
            previous = Some(p.clone());
        }
    }
}

/// Every `enum=` names an enum, every `blob=` a schema, every blob-out
/// function a schema, and every name is unique.
fn check_links(api: &Api) -> Result<(), ExtractError> {
    let mut names = BTreeSet::new();
    for name in api
        .enums
        .iter()
        .map(|e| e.name.as_str())
        .chain(api.structs.iter().map(|s| s.name.as_str()))
        .chain(api.opaques.iter().map(|o| o.name.as_str()))
        .chain(api.callbacks.iter().map(|c| c.name.as_str()))
    {
        if !names.insert(name) {
            return Err(ExtractError::new(name, "defined twice across the sources"));
        }
    }
    let mut symbols = BTreeSet::new();
    for f in &api.functions {
        if !symbols.insert(f.name.as_str()) {
            return Err(ExtractError::new(&f.name, "exported twice"));
        }
        if let Some(blob) = &f.meta.blob {
            if api.blob_named(blob).is_none() {
                return Err(ExtractError::new(
                    &f.name,
                    format!("no blob schema `{blob}`"),
                ));
            }
        }
        if f.params.iter().any(|p| p.role == Role::BlobOut) && f.meta.blob.is_none() {
            return Err(ExtractError::new(
                &f.name,
                "returns a blob but names no schema (`api: blob=<name>`)",
            ));
        }
    }
    for s in &api.structs {
        for f in &s.fields {
            if let Some(e) = &f.meta.enum_name {
                if api.enum_named(e).is_none() {
                    return Err(ExtractError::new(
                        &format!("{}.{}", s.name, f.name),
                        format!("`api: enum={e}` names no enum"),
                    ));
                }
            }
        }
    }
    for b in &api.blobs {
        for s in &b.sections {
            for c in &s.fields {
                if let Some(e) = &c.enum_name {
                    if api.enum_named(e).is_none() {
                        return Err(ExtractError::new(
                            &format!("{}.{}.{}", b.name, s.name, c.name),
                            format!("names no enum `{e}`"),
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use super::*;

    const SAMPLE: &str = r#"
/// The ABI version.
///
/// `api: constant`
pub const TS_ABI_VERSION: u32 = 1;

/// Selects the test provider.
///
/// `api: constant`
pub const TS_CONTEXT_TEST_PROVIDER: u32 = 1 << 0;

/// Not exported.
pub const HIDDEN: u32 = 5;

/// The status.
#[repr(i32)]
pub enum Status {
    /// Fine.
    Ok = 0,
    /// Bad.
    InvalidArg = -1,
}

/// A context.
pub struct TsContext {
    inner: u8,
}

/// Options.
#[repr(C)]
pub struct TsOptions {
    /// The size.
    pub struct_size: u32,
    /// Flags.
    /// `api: example=0`
    pub flags: u32,
    /// The id. See [`Status`].
    /// `api: nullable`
    pub profile: *const c_char,
    /// Zero.
    pub reserved: [u8; 4],
}

/// An owned string.
///
/// `api: role=owned_string`
#[repr(C)]
pub struct TsString {
    /// Bytes.
    pub data: *mut u8,
    /// Length.
    pub len: usize,
}

/// A callback.
pub type PositionsFn = unsafe extern "C" fn(user_data: *mut c_void, out: *mut TsOptions) -> i32;

/// The vtable.
#[repr(C)]
pub struct ProviderVtable {
    /// Size.
    pub struct_size: u32,
    /// Positions.
    pub positions: Option<PositionsFn>,
}

/// Creates a context.
///
/// # Safety
///
/// Pointers valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_context_new(options: *const TsOptions, provider: *const ProviderVtable, user_data: *mut c_void, out_context: *mut *mut TsContext) -> Status { todo!() }

/// Frees.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_context_free(context: *mut TsContext) {}

/// Renders.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_render(context: *const TsContext, key: *const c_char, bytes: *const u8, bytes_len: usize, out_text: *mut TsString) -> Status { todo!() }

/// Frees a string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_string_free(string: *mut TsString) {}

/// The version.
#[unsafe(no_mangle)]
pub extern "C" fn ts_abi_version() -> u32 { TS_ABI_VERSION }
"#;

    fn inputs() -> Inputs {
        Inputs {
            prefix: "ts_".into(),
            sdk_version: "0.0.0".into(),
            abi_version_constant: "TS_ABI_VERSION".into(),
            extra_enums: vec![],
            blobs: vec![],
        }
    }

    #[test]
    fn the_sample_extracts_with_roles_and_metadata() {
        let source = Source {
            relative: "sample.rs".into(),
            text: SAMPLE.into(),
        };
        let api = extract(&[source], &inputs()).unwrap();
        assert_eq!(api.abi_version, 1);
        assert_eq!(
            api.constants
                .iter()
                .map(|c| (c.name.as_str(), c.value))
                .collect::<Vec<_>>(),
            [("TS_ABI_VERSION", 1), ("TS_CONTEXT_TEST_PROVIDER", 1)]
        );
        assert_eq!(api.enums[0].values[1].value, -1);
        assert_eq!(
            api.opaques
                .iter()
                .map(|o| o.name.as_str())
                .collect::<Vec<_>>(),
            ["TsContext"]
        );
        let options = api.struct_named("TsOptions").unwrap();
        assert_eq!(options.fields[1].meta.example.as_deref(), Some("0"));
        assert!(options.fields[2].meta.nullable);
        assert_eq!(options.fields[2].doc, "The id. See `Status`.");
        assert_eq!(
            options.fields[3].ty,
            TypeRef::Array {
                of: Box::new(TypeRef::scalar(Scalar::U8)),
                len: 4
            }
        );
        assert_eq!(
            api.struct_named("TsString").unwrap().role,
            StructRole::OwnedString
        );
        assert_eq!(
            api.struct_named("ProviderVtable").unwrap().role,
            StructRole::Vtable
        );
        assert_eq!(
            api.struct_named("ProviderVtable").unwrap().fields[1].ty,
            TypeRef::Callback {
                name: "PositionsFn".into()
            }
        );
        let new = api.function_named("ts_context_new").unwrap();
        assert_eq!(new.safety.as_deref(), Some("Pointers valid."));
        assert_eq!(new.doc, "Creates a context.");
        assert_eq!(
            new.params.iter().map(|p| p.role).collect::<Vec<_>>(),
            [
                Role::StructIn,
                Role::VtableIn,
                Role::UserData,
                Role::HandleOut
            ]
        );
        assert_eq!(
            new.returns,
            Some(TypeRef::Enum {
                name: "Status".into()
            })
        );
        let render = api.function_named("ts_render").unwrap();
        assert_eq!(
            render.params.iter().map(|p| p.role).collect::<Vec<_>>(),
            [
                Role::Handle,
                Role::StringIn,
                Role::BytesIn,
                Role::Length,
                Role::StringOut
            ]
        );
        assert_eq!(
            api.function_named("ts_string_free").unwrap().params[0].role,
            Role::StringFree
        );
        assert_eq!(api.callbacks[0].params[1].role, Role::StructOut);
        assert!(
            api.function_named("ts_abi_version")
                .unwrap()
                .safety
                .is_none()
        );
    }

    #[test]
    fn unknown_types_and_missing_links_fail_the_build() {
        let missing_abi = Source {
            relative: "a.rs".into(),
            text: "#[unsafe(no_mangle)] pub extern \"C\" fn ts_x() {}".into(),
        };
        assert!(
            extract(&[missing_abi], &inputs())
                .unwrap_err()
                .detail
                .contains("TS_ABI_VERSION")
        );
        let unknown = Source {
            relative: "b.rs".into(),
            text: "/// `api: constant`\npub const TS_ABI_VERSION: u32 = 1;\n#[unsafe(no_mangle)] pub unsafe extern \"C\" fn ts_x(a: *const Nope) {}".into(),
        };
        assert!(
            extract(&[unknown], &inputs())
                .unwrap_err()
                .detail
                .contains("Nope")
        );
        let blob = Source {
            relative: "c.rs".into(),
            text: "/// `api: constant`\npub const TS_ABI_VERSION: u32 = 1;\n/// `api: role=blob`\n#[repr(C)] pub struct TsBlob { pub data: *mut u8 }\n#[unsafe(no_mangle)] pub unsafe extern \"C\" fn ts_x(out_blob: *mut TsBlob) {}".into(),
        };
        assert!(
            extract(&[blob], &inputs())
                .unwrap_err()
                .detail
                .contains("blob")
        );
    }
}
