//! The JSON requests a binding writes ([`crate::records::Requests`]),
//! rendered as each language's types, so a request is typed once, in Rust,
//! as serde reads it.
//!
//! A field keeps its wire name, which is what the binding hands through:
//! the runtime passes a request on unchanged, so its type is the wire's.
//! A field serde defaults is optional, and one that is an `Option` also
//! takes null; a Rust array is a tuple of its length. A closed set of keys is a literal
//! union, or the boundary enum's type where the request reuses one; a union
//! tagged by a field is one type per variant; a union tagged by its key is
//! the bare keys and a one-key object per variant with data; a catalogue
//! member is the member's full or bare key, and a registered key any key.
//!
//! ```
//! use teistro_idl::emit::requests::typescript_declarations;
//! use teistro_idl::model::{Api, SCHEMA};
//! use teistro_idl::records::{from_schema, Requests, RequestRoot};
//!
//! let schema = serde_json::json!({"$defs": {"Rules": {"type": "object",
//!     "properties": {"equalVarna": {"type": "string", "enum": ["WHOLE", "HALF"]}}}}});
//! let requests = Requests {
//!     roots: vec![RequestRoot { record: "matching".into(), name: "Rules".into() }],
//!     records: from_schema(&schema, &["Rules"]).unwrap(),
//!     enums: Default::default(),
//! };
//! let api = Api {
//!     schema: SCHEMA.into(), abi_version: 1, sdk_version: "0".into(), prefix: "ts_".into(),
//!     sources: vec![], constants: vec![], enums: vec![], opaques: vec![], callbacks: vec![],
//!     structs: vec![], functions: vec![], blobs: vec![], records: vec![], requests,
//! };
//! assert!(typescript_declarations(&api).contains("readonly equalVarna?: RulesEqualVarna;"));
//! ```

use std::collections::BTreeMap;
use std::fmt::Write;

use crate::emit::{block_comment, python, reserved, ts};
use crate::model::Api;
use crate::names::{binding_type_name, pascal};
use crate::records::{ExternalVariant, RecordField, RecordShape, RecordType, RecordVariant};

mod dart;
mod dart_deep;
mod examples;
pub use dart::{DART_FACADES, DART_SHADOWED, dart_declarations};
pub use dart_deep::dart_deep_test;
pub use examples::deep_examples;

/// What the TypeScript requests file says it is, after the header.
const TS_ABOUT: &str = "The JSON requests the layers hand through to the SDK, typed. Rendered from the façade's own schema of each request, which schemars reads from serde's description of the Rust types, so a request is typed as serde reads it: a field keeps its wire name and is optional where serde defaults it.";

/// What the Python requests file says it is, after the header.
const PY_ABOUT: &str = "The JSON requests the layer hands through to the SDK, typed as dictionaries. Rendered from the façade's own schema of each request, which schemars reads from serde's description of the Rust types, so a request is typed as serde reads it: a key keeps its wire name, and a key serde defaults may be left out. A catalogue member may be the enum member or its key; the layer writes it as its key.";

/// The Python layer's answer classes a request takes back where it takes
/// the record of the same name, so a caller hands back what it was given;
/// the layer writes each as its JSON (`_written`). Each must be a request
/// record (`cargo xtask gen ffi`'s tests hold it), and `check-python`'s
/// strict mypy reads each as the layer's.
pub const PY_TAKEN_BACK: [&str; 7] = [
    "AccidentalRules",
    "AccidentalScores",
    "AlmutenRules",
    "ConsiderationRules",
    "MuhurtaClauseKind",
    "NatalPoint",
    "PerfectionRules",
];

/// The name a variant of a union tagged by a field is given: the union's,
/// then the variant's key in the language's case, as an answer record's
/// (`planet-in-house` is `ConditionPlanetInHouse`).
fn variant_name(union: &str, variant: &RecordVariant) -> String {
    format!("{union}{}", key_part(&variant.key))
}

/// A wire key as part of a type's name (`planet-in-house` is
/// `PlanetInHouse`).
fn key_part(key: &str) -> String {
    pascal(&key.to_ascii_lowercase().replace('-', "_"))
}

/// Whether a wire name can stand bare as a TypeScript property.
fn is_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_' || first == '$')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

/// A property as TypeScript writes it: bare where it can be, quoted where
/// the wire names it otherwise.
fn ts_property(name: &str) -> String {
    if is_identifier(name) {
        name.to_owned()
    } else {
        format!("'{name}'")
    }
}

/// The most items a fixed-length list of scalars is written out as a
/// tuple with: the seven planets' orbs.
const TUPLE_MOST: u32 = 7;

/// What the emitters know besides a record.
struct Types<'a> {
    api: &'a Api,
    /// Each catalogue kind's type (`graha` is `Graha`).
    catalogue: BTreeMap<&'a str, String>,
    /// The boundary enums that are a catalogue kind, whose TypeScript type
    /// is the full keys (`sex.MALE`) where a request writes the bare ones.
    kinded: BTreeMap<&'a str, String>,
}

impl<'a> Types<'a> {
    fn of(api: &'a Api) -> Types<'a> {
        let catalogue = api
            .enums
            .iter()
            .filter_map(|e| Some((e.kind.as_deref()?, binding_type_name(&e.name))))
            .collect();
        let kinded = api
            .enums
            .iter()
            .filter(|e| e.kind.is_some())
            .map(|e| (e.name.as_str(), binding_type_name(&e.name)))
            .collect();
        Types {
            api,
            catalogue,
            kinded,
        }
    }

    /// A catalogue kind's type, as `catalogue.d.ts` names it.
    fn kind(&self, catalogue: &str) -> String {
        let name = self.catalogue.get(catalogue).unwrap_or_else(|| {
            unreachable!(
                "`cargo xtask gen ffi` refuses a request naming no catalogue kind (`{catalogue}`)"
            )
        });
        format!("catalogue.{name}")
    }

    /// A named request record, or the boundary enum it reuses: its bare
    /// keys where the enum is a catalogue kind.
    fn record(&self, name: &str) -> String {
        match self.api.requests.enums.get(name) {
            Some(boundary) => match self.kinded.get(boundary.as_str()) {
                Some(kind) => format!("Bare<catalogue.{kind}>"),
                None => format!("catalogue.{}", binding_type_name(boundary)),
            },
            None => name.to_owned(),
        }
    }

    /// Whether a list of `len` items of `of` is typed as a tuple: a few
    /// scalars, a key of a closed set among them.
    fn is_tuple(&self, of: &RecordType, len: u32) -> bool {
        len <= TUPLE_MOST
            && match of {
                RecordType::Bool
                | RecordType::Integer { .. }
                | RecordType::Number
                | RecordType::Text
                | RecordType::Member { .. } => true,
                RecordType::Record { name } => self.api.requests.records.iter().any(|record| {
                    record.name == *name && matches!(record.shape, RecordShape::Keys { .. })
                }),
                _ => false,
            }
    }

    /// A field's doc, and how many items it holds where serde reads a
    /// fixed number that is not a tuple (`Exactly 12 items, each of 5.`).
    fn with_lengths(&self, doc: &str, ty: &RecordType) -> String {
        let mut lengths = Vec::new();
        let mut at = ty;
        while let RecordType::List { of, len: Some(len) } = at {
            if self.is_tuple(of, *len) {
                break;
            }
            lengths.push(len.to_string());
            at = of;
        }
        match lengths.split_first() {
            None => doc.to_owned(),
            Some((outer, inner)) => {
                let each = inner.iter().fold(String::new(), |mut out, n| {
                    let _ = write!(out, ", each of {n}");
                    out
                });
                format!("{doc}\n\nExactly {outer} items{each}.")
            }
        }
    }

    fn ty(&self, ty: &RecordType) -> String {
        match ty {
            RecordType::Bool => "boolean".into(),
            RecordType::Integer { .. } | RecordType::Number => "number".into(),
            RecordType::Text => "string".into(),
            RecordType::Json => "unknown".into(),
            RecordType::Record { name } => self.record(name),
            RecordType::Member { catalogue } => format!("Member<{}>", self.kind(catalogue)),
            // A registered key is any key; the members are named so an
            // editor offers them.
            RecordType::Keyed { catalogue } => {
                format!("Member<{}> | (string & {{}})", self.kind(catalogue))
            }
            // A few scalars of a fixed number are a tuple, as serde reads
            // a Rust array and as a person writes them out; a longer list,
            // or one of records, which code builds, is an array whose
            // field's doc says how many.
            RecordType::List { of, len: Some(len) } if self.is_tuple(of, *len) => {
                let item = self.ty(of);
                let items: Vec<&str> = (0..*len).map(|_| item.as_str()).collect();
                format!("readonly [{}]", items.join(", "))
            }
            RecordType::List { of, .. } => match of.as_ref() {
                // `readonly` binds to an array or tuple literal, and `|`
                // looser than `[]`, so either item goes in parentheses.
                RecordType::List { .. } | RecordType::Pair { .. } | RecordType::Keyed { .. } => {
                    format!("readonly ({})[]", self.ty(of))
                }
                _ => format!("readonly {}[]", self.ty(of)),
            },
            RecordType::Pair { first, second } => {
                format!("readonly [{}, {}]", self.ty(first), self.ty(second))
            }
            RecordType::Map { of } => format!("Readonly<Record<string, {}>>", self.ty(of)),
        }
    }

    fn fields(&self, out: &mut String, fields: &[RecordField]) {
        for field in fields {
            out.push_str(&block_comment(
                &self.with_lengths(&field.doc, &field.ty),
                "  ",
            ));
            let optional = if field.nullable { "?" } else { "" };
            let null = if field.null { " | null" } else { "" };
            let _ = writeln!(
                out,
                "  readonly {}{optional}: {}{null};",
                ts_property(&field.name),
                self.ty(&field.ty)
            );
        }
    }
}

/// `requests.d.ts`: every request record as a TypeScript type.
///
/// ```
/// # use teistro_idl::Api;
/// use teistro_idl::emit::requests;
///
/// // With no requests described, the file is its header and the helper
/// // a member's type is spelled with.
/// # let root = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
/// # let api: Api = serde_json::from_str(&std::fs::read_to_string(root.join("idl/api.json"))?)?;
/// assert!(requests::typescript_declarations(&api).contains("export type Member<Full>"));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[must_use]
pub fn typescript_declarations(api: &Api) -> String {
    let types = Types::of(api);
    let mut out = ts::preamble(api, "//");
    out.push_str(&block_comment(TS_ABOUT, ""));
    out.push_str("\nimport type * as catalogue from './catalogue.js';\n\n");
    out.push_str(&block_comment(
        "A catalogue member's bare key (`SUN` for `graha.SUN`).",
        "",
    ));
    out.push_str(
        "export type Bare<Full> = Full extends `${string}.${infer Key}` ? Key : never;\n\n",
    );
    out.push_str(&block_comment(
        "A catalogue member as a request takes it: its full key (`graha.SUN`) or its bare one (`SUN`). The type an answer gives a member is one, so a member read from an answer is asked back as it is; its `unknown` is refused by the SDK, by field.",
        "",
    ));
    out.push_str("export type Member<Full> = Full | Bare<Full>;\n");
    for record in &api.requests.records {
        if api.requests.enums.contains_key(&record.name) {
            continue;
        }
        out.push('\n');
        match &record.shape {
            RecordShape::Object { fields } => {
                out.push_str(&block_comment(&record.doc, ""));
                let _ = writeln!(out, "export interface {} {{", record.name);
                types.fields(&mut out, fields);
                out.push_str("}\n");
            }
            RecordShape::Keys { values } => {
                let listed: Vec<String> = values
                    .iter()
                    .map(|value| format!("`{}`: {}", value.key, value.doc))
                    .collect();
                out.push_str(&block_comment(
                    &format!("{}\n\n{}", record.doc, listed.join("\n")),
                    "",
                ));
                let keys: Vec<String> = values.iter().map(|v| format!("'{}'", v.key)).collect();
                let _ = writeln!(out, "export type {} = {};", record.name, keys.join(" | "));
            }
            RecordShape::Tagged { tag, variants } => {
                let names: Vec<String> = variants
                    .iter()
                    .map(|variant| variant_name(&record.name, variant))
                    .collect();
                out.push_str(&block_comment(&record.doc, ""));
                let _ = writeln!(out, "export type {} = {};", record.name, names.join(" | "));
                for (variant, name) in variants.iter().zip(&names) {
                    out.push('\n');
                    out.push_str(&block_comment(&variant.doc, ""));
                    let _ = writeln!(out, "export interface {name} {{");
                    let _ = writeln!(out, "  readonly {}: '{}';", ts_property(tag), variant.key);
                    types.fields(&mut out, &variant.fields);
                    out.push_str("}\n");
                }
            }
            RecordShape::External { variants } => {
                let listed: Vec<String> = variants
                    .iter()
                    .map(|variant| format!("`{}`: {}", variant.key, variant.doc))
                    .collect();
                out.push_str(&block_comment(
                    &format!("{}\n\n{}", record.doc, listed.join("\n")),
                    "",
                ));
                let arms: Vec<String> = variants
                    .iter()
                    .map(|variant| match &variant.payload {
                        None => format!("'{}'", variant.key),
                        Some(payload) => format!(
                            "{{ readonly {}: {} }}",
                            ts_property(&variant.key),
                            types.ty(payload)
                        ),
                    })
                    .collect();
                let _ = writeln!(
                    out,
                    "export type {} =\n  | {};",
                    record.name,
                    arms.join("\n  | ")
                );
            }
            RecordShape::OrNamed {
                names,
                values,
                member,
            } => {
                out.push_str(&block_comment(&record.doc, ""));
                let mut arms: Vec<String> = names.iter().map(|name| format!("'{name}'")).collect();
                if let Some(kind) = member {
                    arms.push(format!("Member<{}>", types.kind(kind)));
                }
                arms.extend(values.iter().map(|value| types.ty(&value.ty)));
                let _ = writeln!(out, "export type {} = {};", record.name, arms.join(" | "));
            }
        }
    }
    out
}

/// Python: a key that a class body can declare, which is neither a keyword
/// nor anything but an identifier.
fn py_declarable(name: &str) -> bool {
    is_identifier(name) && !name.contains('$') && !reserved::is_reserved(name, reserved::PYTHON)
}

/// A string literal Python reads back as `text`.
fn py_quoted(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}

impl Types<'_> {
    /// A key of a closed set, as a `Literal`.
    fn py_literal(keys: &[String]) -> String {
        let quoted: Vec<String> = keys.iter().map(|key| format!("'{key}'")).collect();
        format!("Literal[{}]", quoted.join(", "))
    }

    /// A named request record, or the boundary enum it reuses: the enum's
    /// member or its key, or the keys alone where the enum is a catalogue
    /// kind, whose member the layer writes as its full key.
    fn py_record(&self, name: &str) -> String {
        let Some(boundary) = self.api.requests.enums.get(name) else {
            return if PY_TAKEN_BACK.contains(&name) {
                format!("Union[{name}, _layer.{name}]")
            } else {
                name.to_owned()
            };
        };
        let keys: Vec<String> = self
            .api
            .requests
            .records
            .iter()
            .find(|record| record.name == name)
            .and_then(|record| match &record.shape {
                RecordShape::Keys { values } => {
                    Some(values.iter().map(|value| value.key.clone()).collect())
                }
                _ => None,
            })
            .unwrap_or_default();
        if self.kinded.contains_key(boundary.as_str()) {
            Self::py_literal(&keys)
        } else {
            format!(
                "Union[_catalogue.{}, {}]",
                binding_type_name(boundary),
                Self::py_literal(&keys)
            )
        }
    }

    /// A catalogue member: the enum's member or its key.
    fn py_member(&self, catalogue: &str) -> String {
        format!(
            "Union[_catalogue.{}, str]",
            &self.kind(catalogue)["catalogue.".len()..]
        )
    }

    fn py(&self, ty: &RecordType) -> String {
        match ty {
            RecordType::Bool => "bool".into(),
            RecordType::Integer { .. } => "int".into(),
            RecordType::Number => "float".into(),
            RecordType::Text => "str".into(),
            RecordType::Json => "Any".into(),
            RecordType::Record { name } => self.py_record(name),
            RecordType::Member { catalogue } | RecordType::Keyed { catalogue } => {
                self.py_member(catalogue)
            }
            RecordType::List { of, .. } => format!("Sequence[{}]", self.py(of)),
            RecordType::Pair { first, second } => {
                format!("Tuple[{}, {}]", self.py(first), self.py(second))
            }
            RecordType::Map { of } => format!("Mapping[str, {}]", self.py(of)),
        }
    }

    /// A field's type as a `TypedDict` declares it: `Required` where serde
    /// reads it always, `Optional` where it takes null.
    fn py_field(&self, field: &RecordField) -> String {
        let ty = self.py(&field.ty);
        let ty = if field.null {
            format!("Optional[{ty}]")
        } else {
            ty
        };
        if field.nullable {
            ty
        } else {
            format!("Required[{ty}]")
        }
    }

    /// A union tagged by its key: the bare keys' `Literal` and a one-key
    /// dictionary per variant with data, named for the union and the key
    /// and private, since a caller writes the dictionary.
    fn py_external(&self, out: &mut String, name: &str, doc: &str, variants: &[ExternalVariant]) {
        let mut arms = Vec::new();
        let bare: Vec<String> = variants
            .iter()
            .filter(|variant| variant.payload.is_none())
            .map(|variant| variant.key.clone())
            .collect();
        if !bare.is_empty() {
            arms.push(Types::py_literal(&bare));
        }
        let mut wrappers = String::new();
        for variant in variants {
            let Some(payload) = &variant.payload else {
                continue;
            };
            // A one-key object per variant with data, named for the
            // union and the key; a caller writes the dictionary.
            let wrapper = format!("_{name}{}", key_part(&variant.key));
            arms.push(wrapper.clone());
            wrappers.push_str("\n\n");
            let field = RecordField {
                name: variant.key.clone(),
                doc: variant.doc.clone(),
                ty: payload.clone(),
                nullable: false,
                null: false,
                default: None,
            };
            self.py_typed_dict(
                &mut wrappers,
                &wrapper,
                "",
                None,
                std::slice::from_ref(&field),
            );
        }
        let _ = writeln!(
            out,
            "{name}: TypeAlias = {}",
            py_quoted(&format!("Union[{}]", arms.join(", ")))
        );
        let listed: Vec<String> = variants
            .iter()
            .map(|variant| format!("`{}`: {}", variant.key, variant.doc))
            .collect();
        out.push_str(&python::docstring(
            &format!("{}\n\n{}", doc, listed.join("\n")),
            "",
        ));
        out.push_str(&wrappers);
    }

    /// A `TypedDict` of these fields, after a tag where a union is tagged
    /// by one: a class where every key can be declared, and the call form
    /// where one cannot (`from`), with its doc as a comment.
    fn py_typed_dict(
        &self,
        out: &mut String,
        name: &str,
        doc: &str,
        tag: Option<(&str, &str)>,
        fields: &[RecordField],
    ) {
        let mut entries: Vec<(String, String, String)> = Vec::new();
        if let Some((tag, key)) = tag {
            entries.push((
                tag.to_owned(),
                format!("Required[Literal['{key}']]"),
                String::new(),
            ));
        }
        for field in fields {
            entries.push((
                field.name.clone(),
                self.py_field(field),
                self.with_lengths(&field.doc, &field.ty),
            ));
        }
        if entries.iter().all(|(key, _, _)| py_declarable(key)) {
            let _ = writeln!(out, "class {name}(TypedDict, total=False):");
            let docstring = python::docstring(doc, "    ");
            out.push_str(&docstring);
            if entries.is_empty() {
                if docstring.is_empty() {
                    out.push_str("    pass\n");
                }
                return;
            }
            for (at, (key, ty, field_doc)) in entries.iter().enumerate() {
                if !docstring.is_empty() || at > 0 {
                    out.push('\n');
                }
                let _ = writeln!(out, "    {key}: {ty}");
                out.push_str(&python::docstring(field_doc, "    "));
            }
        } else {
            out.push_str(&crate::emit::line_comment(
                &crate::names::clean_doc(doc),
                "",
                "# ",
            ));
            let _ = writeln!(out, "{name} = TypedDict(\n    \"{name}\",\n    {{");
            for (key, ty, field_doc) in &entries {
                out.push_str(&crate::emit::line_comment(
                    &crate::names::clean_doc(field_doc),
                    "        ",
                    "# ",
                ));
                let _ = writeln!(out, "        {}: {},", py_quoted(key), py_quoted(ty));
            }
            out.push_str("    },\n    total=False,\n)\n");
        }
    }
}

/// `_requests.py`: every request record as a `TypedDict`, a `Literal` or
/// a union of them.
///
/// ```
/// # use teistro_idl::Api;
/// use teistro_idl::emit::requests;
///
/// // The description on disk carries no requests, so the file is its
/// // header and its imports.
/// # let root = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
/// # let api: Api = serde_json::from_str(&std::fs::read_to_string(root.join("idl/api.json"))?)?;
/// assert!(requests::python_declarations(&api).contains("from . import catalogue as _catalogue"));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[must_use]
pub fn python_declarations(api: &Api) -> String {
    let types = Types::of(api);
    let mut out = python::preamble(api, PY_ABOUT);
    out.push_str("from typing import TYPE_CHECKING, Any, Literal, Mapping, Optional, Required, Sequence, Tuple, TypeAlias, TypedDict, Union\n\nfrom . import catalogue as _catalogue\n\nif TYPE_CHECKING:\n    import teistro as _layer\n");
    let mut exported: Vec<String> = Vec::new();
    let mut body = String::new();
    for record in &api.requests.records {
        if api.requests.enums.contains_key(&record.name) {
            continue;
        }
        let name = &record.name;
        body.push_str("\n\n");
        match &record.shape {
            RecordShape::Object { fields } => {
                exported.push(name.clone());
                types.py_typed_dict(&mut body, name, &record.doc, None, fields);
            }
            RecordShape::Keys { values } => {
                exported.push(name.clone());
                let keys: Vec<String> = values.iter().map(|value| value.key.clone()).collect();
                let _ = writeln!(body, "{name}: TypeAlias = {}", Types::py_literal(&keys));
                let listed: Vec<String> = values
                    .iter()
                    .map(|value| format!("`{}`: {}", value.key, value.doc))
                    .collect();
                body.push_str(&python::docstring(
                    &format!("{}\n\n{}", record.doc, listed.join("\n")),
                    "",
                ));
            }
            RecordShape::Tagged { tag, variants } => {
                let names: Vec<String> = variants
                    .iter()
                    .map(|variant| variant_name(name, variant))
                    .collect();
                exported.push(name.clone());
                let _ = writeln!(
                    body,
                    "{name}: TypeAlias = {}",
                    py_quoted(&format!("Union[{}]", names.join(", ")))
                );
                body.push_str(&python::docstring(&record.doc, ""));
                for (variant, variant_class) in variants.iter().zip(&names) {
                    exported.push(variant_class.clone());
                    body.push_str("\n\n");
                    types.py_typed_dict(
                        &mut body,
                        variant_class,
                        &variant.doc,
                        Some((tag, &variant.key)),
                        &variant.fields,
                    );
                }
            }
            RecordShape::External { variants } => {
                exported.push(name.clone());
                types.py_external(&mut body, name, &record.doc, variants);
            }
            RecordShape::OrNamed {
                names,
                values,
                member,
            } => {
                exported.push(name.clone());
                let mut arms = vec![Types::py_literal(names)];
                if let Some(kind) = member {
                    arms.push(types.py_member(kind));
                }
                arms.extend(values.iter().map(|value| types.py(&value.ty)));
                let _ = writeln!(
                    body,
                    "{name}: TypeAlias = {}",
                    py_quoted(&format!("Union[{}]", arms.join(", ")))
                );
                body.push_str(&python::docstring(&record.doc, ""));
            }
        }
    }
    let listed: Vec<String> = exported
        .iter()
        .map(|name| format!("    \"{name}\",\n"))
        .collect();
    let _ = write!(out, "\n__all__ = [\n{}]\n", listed.concat());
    out.push_str(&body);
    out
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::unwrap_used, reason = "a test fails by panicking")]

    use super::*;
    use crate::model::{EnumDef, SCHEMA};
    use crate::records::{RecordDef, RecordKey, Requests, from_schema};
    use serde_json::json;

    fn kind(name: &str, kind: Option<&str>, key: &str) -> EnumDef {
        serde_json::from_value(json!({"name": name, "doc": "", "repr": "u16", "kind": kind,
            "values": [{"name": "One", "value": 0, "doc": "", "key": key}], "source": ""}))
        .unwrap()
    }

    /// Every shape a request takes, read from a schema as the façade's are.
    fn fixture() -> Api {
        let schema = json!({"$defs": {
            "Ask": {"type": "object", "properties": {
                "orbs": {"type": "array", "items": {"type": "number"}, "minItems": 7, "maxItems": 7},
                "houses": {"type": "array", "items": {"type": "number"}, "minItems": 12, "maxItems": 12},
                "table": {"type": "array", "minItems": 2, "maxItems": 2, "items":
                    {"type": "array", "items": {"$ref": "#/$defs/Entry"}, "minItems": 5, "maxItems": 5}},
                "moves": {"type": "array", "items": {"$ref": "#/$defs/Move"}, "minItems": 3, "maxItems": 3},
                "lord": {"anyOf": [{"$ref": "#/$defs/Graha"}, {"type": "null"}]},
                "lords": {"type": "array", "items": {"anyOf": [{"$ref": "#/$defs/Graha"},
                    {"type": "string", "pattern": "^.+$"}]}},
                "pairs": {"type": "array", "items": {"type": "array", "prefixItems":
                    [{"type": "string"}, {"type": "integer"}], "minItems": 2, "maxItems": 2}},
                "by": {"type": "object", "additionalProperties": {"type": "boolean"}},
                "sex": {"$ref": "#/$defs/Sex"},
                "how": {"$ref": "#/$defs/How"},
                "tag": {"$ref": "#/$defs/Tag"},
                "point": {"$ref": "#/$defs/Point"},
                "with-dash": {"type": "string"},
                "speed": {"$ref": "#/$defs/Move", "default": "FAST"},
                "weight": {"type": "number", "default": 2},
                "first": {"$ref": "#/$defs/Entry", "default": {"end": 1.5}},
                "gender": {"$ref": "#/$defs/Sex", "default": "MALE"},
                "after": {"$ref": "#/$defs/Tag", "default": "SAME"},
                "rule": {"$ref": "#/$defs/Rule"},
                "rules": {"type": "array", "items": {"$ref": "#/$defs/Rule"}},
                "gate": {"$ref": "#/$defs/Gate"}
            }, "required": ["orbs", "how", "with-dash"]},
            "Rule": {"anyOf": [{"type": "string", "enum": ["PACK"]},
                {"$ref": "#/$defs/Spelt", "x-teistro-when": ["windows"]}, {"$ref": "#/$defs/Entry"}]},
            "Spelt": {"type": "object", "properties": {"windows": {"type": "integer"}}, "required": ["windows"]},
            "Gate": {"anyOf": [{"$ref": "#/$defs/Level"}, {"type": "boolean"}]},
            "Level": {"oneOf": [{"const": "LOW", "description": "Low."}, {"const": "HIGH"}]},
            "Entry": {"type": "object", "properties": {"end": {"type": "number"}}, "required": ["end"]},
            "Move": {"type": "string", "enum": ["MEAN", "FAST"]},
            "Graha": {"type": "string", "x-teistro-kind": "graha"},
            "Sex": {"type": "string", "enum": ["MALE"]},
            "How": {"oneOf": [
                {"type": "object", "properties": {"kind": {"const": "planet-in-house"}}, "required": ["kind"]},
                {"type": "object", "properties": {"kind": {"const": "AT"}, "deg": {"type": "number"}},
                    "required": ["kind", "deg"]}]},
            "Tag": {"oneOf": [{"const": "SAME"},
                {"type": "object", "properties": {"WITHIN": {"type": "number"}}, "required": ["WITHIN"],
                    "additionalProperties": false}]},
            "Point": {"anyOf": [{"enum": ["LAGNA"]}, {"$ref": "#/$defs/Graha"}, {"$ref": "#/$defs/Entry"}]}
        }});
        let mut requests = Requests {
            roots: vec![],
            records: from_schema(&schema, &["Ask"]).unwrap(),
            enums: BTreeMap::default(),
        };
        requests.enums.insert("Sex".into(), "Sex".into());
        requests.records.push(RecordDef {
            name: "Ephemeris".into(),
            doc: String::new(),
            shape: RecordShape::Keys {
                values: vec![RecordKey {
                    key: "NONE".into(),
                    doc: String::new(),
                }],
            },
        });
        requests
            .enums
            .insert("Ephemeris".into(), "TsEphemeris".into());
        Api {
            schema: SCHEMA.into(),
            abi_version: 1,
            sdk_version: "0".into(),
            prefix: "ts_".into(),
            sources: vec![],
            constants: vec![],
            enums: vec![
                kind("Graha", Some("graha"), "SUN"),
                kind("Sex", Some("sex"), "MALE"),
                kind("TsEphemeris", None, "NONE"),
            ],
            opaques: vec![],
            callbacks: vec![],
            structs: vec![],
            functions: vec![],
            blobs: vec![],
            records: vec![],
            requests,
        }
    }

    fn rendered() -> String {
        typescript_declarations(&fixture())
    }

    #[test]
    fn a_short_list_of_scalars_is_a_tuple_and_a_long_one_says_its_length() {
        let out = rendered();
        assert!(out.contains(
            "readonly orbs: readonly [number, number, number, number, number, number, number];"
        ));
        assert!(
            out.contains("readonly moves?: readonly [Move, Move, Move];"),
            "a key is a scalar"
        );
        assert!(out.contains("readonly houses?: readonly number[];"));
        assert!(out.contains("Exactly 12 items."));
        assert!(out.contains("readonly table?: readonly (readonly Entry[])[];"));
        assert!(out.contains("Exactly 2 items, each of 5."));
    }

    #[test]
    fn each_field_is_typed_as_serde_reads_it() {
        let out = rendered();
        assert!(
            out.contains("readonly lord?: Member<catalogue.Graha> | null;"),
            "an Option takes null"
        );
        assert!(
            out.contains("readonly lords?: readonly (Member<catalogue.Graha> | (string & {}))[];")
        );
        assert!(out.contains("readonly pairs?: readonly (readonly [string, number])[];"));
        assert!(out.contains("readonly by?: Readonly<Record<string, boolean>>;"));
        assert!(
            out.contains("readonly 'with-dash': string;"),
            "a wire name that is no identifier is quoted"
        );
        assert!(
            out.contains("readonly sex?: Bare<catalogue.Sex>;"),
            "a kind's enum is its bare keys"
        );
        assert!(
            !out.contains("export type Sex"),
            "a reused enum is not declared again"
        );
        assert!(!out.contains("export type Ephemeris"));
    }

    #[test]
    fn each_union_is_typed_as_serde_reads_it() {
        let out = rendered();
        assert!(out.contains("export type How = HowPlanetInHouse | HowAt;"));
        assert!(out.contains("  readonly kind: 'planet-in-house';"));
        assert!(out.contains("export type Tag =\n  | 'SAME'\n  | { readonly WITHIN: number };"));
        assert!(out.contains("export type Point = 'LAGNA' | Member<catalogue.Graha> | Entry;"));
        assert!(out.contains("export type Move = 'MEAN' | 'FAST';"));
    }

    #[test]
    fn a_reused_enum_without_a_kind_is_the_boundary_type() {
        let mut api: Api = serde_json::from_value(json!({"schema": SCHEMA, "abi_version": 1,
            "sdk_version": "0", "prefix": "ts_", "sources": [], "constants": [],
            "enums": [], "opaques": [], "callbacks": [], "structs": [], "functions": [],
            "blobs": [], "records": []}))
        .unwrap();
        api.requests
            .enums
            .insert("Ephemeris".into(), "TsEphemeris".into());
        let types = Types::of(&api);
        assert_eq!(types.record("Ephemeris"), "catalogue.Ephemeris");
        assert_eq!(types.record("Rules"), "Rules");
        assert!(is_identifier("_a$1") && !is_identifier("1a") && !is_identifier("a-b"));
    }

    #[test]
    fn dart_writes_each_field_as_serde_reads_it() {
        let out = dart_declarations(&fixture());
        assert!(out.contains("part of '../teistro.dart';"));
        assert!(
            out.contains("    required this.orbs,\n"),
            "a required field"
        );
        assert!(
            out.contains("    this.speed = Move.fast,\n")
                && out.contains("    this.weight = 2.0,\n")
                && out.contains("    this.first = const Entry(end: 1.5),\n")
                && out.contains("    this.gender = Sex.one,\n")
                && out.contains("    this.after = const TagSame(),\n"),
            "a default the schema states is the constructor's"
        );
        assert!(
            out.contains("  final Graha? lord;\n")
                && out.contains("if (lord case final lordValue?) 'lord': lordValue.key"),
            "an Option is nullable and left off the wire when null"
        );
        assert!(
            out.contains("'lords': [for (final e0 in lords) e0.fullKey]")
                || out.contains("if (lords case final lordsValue?) 'lords': [for (final e0 in lordsValue) e0.fullKey]"),
            "a registered key is written in full"
        );
        assert!(out.contains("'with-dash': withDash"));
        assert!(out.contains("final List<(String, int)>? pairs;"));
        assert!(
            out.contains("[for (final e0 in pairsValue) [e0.$1, e0.$2]]"),
            "a pair crosses as a two-item array"
        );
        assert!(out.contains("'how': how._wire"));
        assert!(out.contains("enum Move implements _Keyed {\n"));
        assert!(!out.contains("enum Sex"), "a reused enum is the boundary's");
    }

    #[test]
    fn dart_writes_each_union_as_serde_reads_it() {
        let out = dart_declarations(&fixture());
        assert!(out.contains("sealed class How extends _Request {"));
        assert!(
            out.contains("final class HowPlanetInHouse extends How {")
                && out.contains("'kind': 'planet-in-house'"),
            "a variant writes its tag"
        );
        assert!(
            out.contains("final class TagSame extends Tag {")
                && out.contains("Object? get _wire => 'SAME';")
        );
        assert!(
            out.contains("Object? get _wire => <String, Object?>{'WITHIN': value};")
                || out.contains("Object? get _wire => {'WITHIN': value};")
        );
        assert!(
            out.contains("final class PointLagna extends Point {")
                && out.contains("final class PointMember extends Point {")
                && out.contains("final class PointValue extends Point {"),
            "a name, a member and the value are each a variant"
        );
    }

    #[test]
    fn dart_compares_a_part_the_layer_keeps_by_hand_by_what_it_writes() {
        // `Interval` is a façade: the layer's class compares by identity, so
        // a generated request holding one compares it by its JSON.
        let mut api = fixture();
        let schema = json!({"$defs": {
            "Hold": {"type": "object", "properties": {
                "window": {"$ref": "#/$defs/Interval"},
                "windows": {"type": "array", "items": {"$ref": "#/$defs/Interval"}},
                "end": {"$ref": "#/$defs/Entry"}
            }, "required": ["window", "end"]},
            "Interval": {"type": "object", "properties": {"from": {"type": "number"},
                "to": {"type": "number"}}, "required": ["from", "to"]},
            "Entry": {"type": "object", "properties": {"end": {"type": "number"}}, "required": ["end"]}
        }});
        let known: Vec<String> = api
            .requests
            .records
            .iter()
            .map(|r| r.name.clone())
            .collect();
        api.requests.records.extend(
            from_schema(&schema, &["Hold"])
                .unwrap()
                .into_iter()
                .filter(|record| !known.contains(&record.name)),
        );
        let out = dart_declarations(&api);
        assert!(
            !out.contains("final class Interval "),
            "the layer declares it"
        );
        assert!(
            out.contains("List<Object?> get _fields => [end, jsonEncode(window._wire), if (windows case final windowsValue?) jsonEncode([for (final e0 in windowsValue) e0._wire]) else null];"),
            "a generated part compares as itself, a hand-kept one by its JSON: {out}"
        );
    }

    #[test]
    fn python_types_each_field_as_serde_reads_it() {
        let out = python_declarations(&fixture());
        // A key no class body can declare (`with-dash`) puts the record in
        // the call form, each type a string.
        assert!(out.contains("        \"orbs\": \"Required[Sequence[float]]\",\n"));
        assert!(
            out.contains("        \"lord\": \"Optional[Union[_catalogue.Graha, str]]\",\n"),
            "an Option takes null"
        );
        assert!(out.contains("        \"pairs\": \"Sequence[Tuple[str, int]]\",\n"));
        assert!(out.contains("        \"by\": \"Mapping[str, bool]\",\n"));
        assert!(
            out.contains("        \"sex\": \"Literal['MALE']\",\n"),
            "a kind's enum is its bare keys"
        );
        assert!(out.contains("Exactly 12 items."));
        assert!(
            !out.contains("\nSex: TypeAlias"),
            "a reused enum is not declared again"
        );
        assert!(out.contains("Ask = TypedDict(\n    \"Ask\",\n    {\n"));
        assert!(out.contains("class Entry(TypedDict, total=False):\n    end: Required[float]\n"));
        assert!(out.contains("        \"with-dash\": \"Required[str]\",\n"));
    }

    #[test]
    fn python_types_each_union_as_serde_reads_it() {
        let out = python_declarations(&fixture());
        assert!(out.contains("How: TypeAlias = \"Union[HowPlanetInHouse, HowAt]\""));
        assert!(out.contains("class HowAt(TypedDict, total=False):\n"));
        assert!(out.contains("    kind: Required[Literal['AT']]\n"));
        assert!(out.contains("Tag: TypeAlias = \"Union[Literal['SAME'], _TagWithin]\""));
        assert!(
            out.contains(
                "class _TagWithin(TypedDict, total=False):\n    WITHIN: Required[float]\n"
            )
        );
        assert!(out.contains(
            "Point: TypeAlias = \"Union[Literal['LAGNA'], Union[_catalogue.Graha, str], Entry]\""
        ));
        assert!(out.contains("Move: TypeAlias = Literal['MEAN', 'FAST']"));
        assert!(out.contains("    \"How\",\n"), "every record is exported");
        assert!(
            !out.contains("    \"_TagWithin\",\n"),
            "a wrapper is private"
        );
    }

    #[test]
    fn a_union_of_names_and_several_records_is_written_and_picked_by_its_keys() {
        let mut api = fixture();
        api.requests.roots.push(crate::records::RequestRoot {
            record: "ask".into(),
            name: "Ask".into(),
        });
        let ts = typescript_declarations(&api);
        assert!(
            ts.contains("export type Rule = 'PACK' | Spelt | Entry;"),
            "{ts}"
        );
        assert!(
            ts.contains("export type Gate = 'LOW' | 'HIGH' | boolean;"),
            "{ts}"
        );
        let py = python_declarations(&api);
        assert!(
            py.contains(r#"Rule: TypeAlias = "Union[Literal['PACK'], Spelt, Entry]""#),
            "{py}"
        );
        assert!(
            py.contains(r#"Gate: TypeAlias = "Union[Literal['LOW', 'HIGH'], bool]""#),
            "{py}"
        );
        let dart = dart_declarations(&api);
        for class in [
            "final class RulePack extends Rule",
            "final class RuleSpelt extends Rule",
            "final class RuleEntry extends Rule",
            "final class GateValue extends Gate",
        ] {
            assert!(dart.contains(class), "{class} in {dart}");
        }
        assert!(
            dart.contains("/// The value, picked by `windows`."),
            "{dart}"
        );
        assert!(
            dart.contains("/// The value, when it carries none of the others' keys."),
            "{dart}"
        );
        let deep = deep_examples(&api, &json!([]));
        assert_eq!(
            deep.pointer("/0/request/rules"),
            Some(&json!([{"windows": 1}, {"end": 1.5}]))
        );
        assert_eq!(deep.pointer("/0/request/gate"), Some(&json!(true)));
        let test = dart_deep_test(&api, &deep);
        assert!(
            test.contains("rules: [RuleSpelt(Spelt(windows: 1)), RuleEntry(Entry(end: 1.5))],"),
            "{test}"
        );
        assert!(test.contains("gate: GateValue(true),"), "{test}");
        let java = crate::emit::java::render(&api);
        let file = |name: &str| {
            java.iter()
                .find(|f| f.path.ends_with(&format!("/request/{name}.java")))
                .map(|f| f.text.clone())
                .unwrap_or_default()
        };
        let rule = file("Rule");
        assert!(
            rule.contains("permits Rule.Key, RuleSpelt, RuleEntry"),
            "{rule}"
        );
        assert!(
            rule.contains("static Rule value(Spelt value)")
                && rule.contains("static Rule value(Entry value)"),
            "{rule}"
        );
        assert!(
            rule.contains(
                "if (object.containsKey(\"windows\")) {\n                return RuleSpelt.of(raw);"
            ),
            "{rule}"
        );
        assert!(rule.contains("return RuleEntry.of(raw);\n    }"), "{rule}");
        let gate = file("Gate");
        assert!(
            gate.contains("if (raw instanceof String) {\n            return Key.of(raw);"),
            "{gate}"
        );
        assert!(gate.contains("static Gate value(boolean value)"), "{gate}");
        assert!(file("GateValue").contains("Read.bool(raw, \"Gate.value\")"));
    }

    #[test]
    fn python_takes_back_the_layers_answer_where_it_takes_the_record() {
        let mut api = fixture();
        api.requests.records.push(RecordDef {
            name: "NatalPoint".into(),
            doc: String::new(),
            shape: RecordShape::Object { fields: vec![] },
        });
        let types = Types::of(&api);
        assert_eq!(
            types.py_record("NatalPoint"),
            "Union[NatalPoint, _layer.NatalPoint]"
        );
        assert_eq!(
            types.py_record("Ephemeris"),
            "Union[_catalogue.Ephemeris, Literal['NONE']]"
        );
        assert!(py_declarable("orbs") && !py_declarable("from") && !py_declarable("a-b"));
        assert_eq!(py_quoted("a\"b"), "\"a\\\"b\"");
    }

    #[test]
    fn dart_builds_each_deep_example_with_the_generated_classes() {
        let mut api = fixture();
        api.requests.roots.push(crate::records::RequestRoot {
            record: "ask".into(),
            name: "Ask".into(),
        });
        let deep = deep_examples(&api, &json!([]));
        let out = dart_deep_test(&api, &deep);
        assert!(out.contains("  test('ask: Ask', () {\n"), "{out}");
        assert!(out.contains("      written: Ask(\n"), "the root is built");
        assert!(
            out.contains("lord: Graha.one,"),
            "a member is its enum's member"
        );
        assert!(out.contains("how: HowAt(deg: 1.5),"), "the widest variant");
        assert!(
            out.contains("after: TagWithin(1.5),"),
            "a variant with a value"
        );
        assert!(
            out.contains("point: PointValue(Entry(end: 1.5)),"),
            "a value beside names"
        );
        assert!(
            out.contains("sex: Sex.one,"),
            "a reused enum is the boundary's"
        );
        assert!(
            out.contains("pairs: [('x', 1)],"),
            "a pair is a Dart record"
        );
        assert!(out.contains("by: {'x': true},"));
        assert!(out.contains("      ).toJson(),\n"));
        assert!(out.contains("dropping: [],") && out.contains("members: 2,"));
        assert!(out.contains("The cases compare 2 of the examples' 2 members."));
    }
}
