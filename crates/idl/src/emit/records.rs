//! The JSON records that cross as text ([`crate::records`]), rendered as
//! each language's typed values with a decoder from the wire's JSON.
//!
//! A record's fields are named as the language names things — `settingsHash`
//! in TypeScript and Dart, `settings_hash` in Python — and a decoder turns
//! the wire's canonical JSON into them: null for a field that is null or
//! absent, a record's own decoder for a record, a list item by item, a pair
//! as the language's pair. A union tagged by `kind` is one type per variant
//! and a union of them; a closed set of keys is the language's own union of
//! strings (a TypeScript literal union, a Python `Literal`, a Dart enum).
//!
//! ```
//! use teistro_idl::emit::records::typescript_declarations;
//! use teistro_idl::model::{Api, SCHEMA};
//! use teistro_idl::records::from_schema;
//!
//! let schema = serde_json::json!({"$defs": {"Stamp": {"type": "object",
//!     "properties": {"settings_hash": {"type": "string"}}, "required": ["settings_hash"]}}});
//! let api = Api {
//!     schema: SCHEMA.into(), abi_version: 1, sdk_version: "0".into(), prefix: "ts_".into(),
//!     sources: vec![], constants: vec![], enums: vec![], opaques: vec![], callbacks: vec![],
//!     structs: vec![], functions: vec![], blobs: vec![],
//!     records: from_schema(&schema, &["Stamp"]).unwrap(),
//! };
//! assert!(typescript_declarations(&api).contains("readonly settingsHash: string;"));
//! ```

use std::fmt::Write;

use crate::emit::{block_comment, dart, python, reserved, ts};
use crate::model::Api;
use crate::names::{camel, pascal, snake};
use crate::records::{RecordField, RecordShape, RecordType, RecordVariant};

/// What every records file says it is, after the generator's header.
const ABOUT: &str = "The JSON records that cross as text inside a result blob — a result's provenance, a positions result's completion steps — typed, with a decoder from their canonical JSON. Rendered from the `records` of idl/api.json, which schemars reads from serde's own description of the Rust types.";

/// The name a variant of a tagged union is given: the union's, then the
/// variant's key in the language's case (`CalendarResolutionTabular`).
fn variant_name(union: &str, variant: &RecordVariant) -> String {
    format!("{union}{}", pascal(&variant.key.to_ascii_lowercase()))
}

/// Whether a type decodes to itself: no record anywhere inside it, so the
/// parsed JSON is already the value. A key union is a record too, so its
/// decoder refuses a key the SDK does not write in every language alike.
fn decodes_to_itself(ty: &RecordType) -> bool {
    match ty {
        RecordType::Bool | RecordType::Integer | RecordType::Number | RecordType::Text => true,
        RecordType::Record { .. } => false,
        RecordType::List { of } => decodes_to_itself(of),
        RecordType::Pair { first, second } => decodes_to_itself(first) && decodes_to_itself(second),
    }
}

/// Whether a named record is a union of keys, which decodes from a bare
/// string rather than an object.
fn is_keys(api: &Api, name: &str) -> bool {
    api.record_named(name)
        .is_some_and(|record| matches!(record.shape, RecordShape::Keys { .. }))
}

// ── TypeScript ─────────────────────────────────────────────────────────────

/// A field as TypeScript names it.
fn ts_field(name: &str) -> String {
    camel(name)
}

fn ts_type(ty: &RecordType) -> String {
    match ty {
        RecordType::Bool => "boolean".into(),
        RecordType::Integer | RecordType::Number => "number".into(),
        RecordType::Text => "string".into(),
        RecordType::Record { name } => name.clone(),
        RecordType::List { of } => match of.as_ref() {
            // `readonly` binds to an array or tuple literal, so a list of
            // them puts the item in parentheses.
            RecordType::List { .. } | RecordType::Pair { .. } => {
                format!("readonly ({})[]", ts_type(of))
            }
            _ => format!("readonly {}[]", ts_type(of)),
        },
        RecordType::Pair { first, second } => {
            format!("readonly [{}, {}]", ts_type(first), ts_type(second))
        }
    }
}

fn ts_fields(out: &mut String, fields: &[RecordField]) {
    for field in fields {
        out.push_str(&block_comment(&field.doc, "  "));
        let nullable = if field.nullable { " | null" } else { "" };
        let _ = writeln!(
            out,
            "  readonly {}: {}{nullable};",
            ts_field(&field.name),
            ts_type(&field.ty)
        );
    }
}

/// `records.d.ts`: every record as a TypeScript type, and its decoder's
/// declaration.
#[must_use]
pub fn typescript_declarations(api: &Api) -> String {
    let mut out = ts::preamble(api, "//");
    out.push_str(&block_comment(ABOUT, ""));
    for record in &api.records {
        out.push('\n');
        match &record.shape {
            RecordShape::Object { fields } => {
                out.push_str(&block_comment(&record.doc, ""));
                let _ = writeln!(out, "export interface {} {{", record.name);
                ts_fields(&mut out, fields);
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
                    let _ = writeln!(out, "  readonly {}: '{}';", ts_field(tag), variant.key);
                    ts_fields(&mut out, &variant.fields);
                    out.push_str("}\n");
                }
            }
        }
        out.push('\n');
        out.push_str(&block_comment(
            &format!(
                "A `{}` from its JSON, as a blob carries it or a stored document holds it.",
                record.name
            ),
            "",
        ));
        let _ = writeln!(
            out,
            "export declare function decode{}(json: unknown): {};",
            record.name, record.name
        );
    }
    out
}

/// The expression that decodes `expr`, of type `ty`, in JavaScript.
fn js_decode(ty: &RecordType, expr: &str) -> String {
    if decodes_to_itself(ty) {
        return expr.to_owned();
    }
    match ty {
        RecordType::Record { name } => format!("decode{name}({expr})"),
        RecordType::List { of } => format!("{expr}.map((item) => {})", js_decode(of, "item")),
        RecordType::Pair { first, second } => format!(
            "[{}, {}]",
            js_decode(first, &format!("{expr}[0]")),
            js_decode(second, &format!("{expr}[1]"))
        ),
        _ => expr.to_owned(),
    }
}

fn js_fields(out: &mut String, fields: &[RecordField], indent: &str) {
    for field in fields {
        let wire = format!("json.{}", field.name);
        let decoded = js_decode(&field.ty, &wire);
        let value = if field.nullable {
            format!("{wire} == null ? null : {decoded}")
        } else {
            decoded
        };
        let _ = writeln!(out, "{indent}{}: {value},", ts_field(&field.name));
    }
}

/// `records.js`: every record's decoder.
#[must_use]
pub fn javascript_decoders(api: &Api) -> String {
    let mut out = ts::preamble(api, "//");
    out.push_str(&block_comment(ABOUT, ""));
    for record in &api.records {
        let name = &record.name;
        match &record.shape {
            RecordShape::Object { fields } => {
                let _ = writeln!(out, "\n/** A `{name}` from its JSON. */");
                let _ = writeln!(out, "export function decode{name}(json) {{");
                out.push_str("  return {\n");
                js_fields(&mut out, fields, "    ");
                out.push_str("  };\n}\n");
            }
            RecordShape::Keys { values } => {
                let keys: Vec<String> = values.iter().map(|v| format!("'{}'", v.key)).collect();
                let _ = writeln!(
                    out,
                    "\nconst {} = new Set([{}]);",
                    keys_constant(name),
                    keys.join(", ")
                );
                let _ = writeln!(out, "\n/** A `{name}` from its JSON: the key itself. */");
                let _ = writeln!(out, "export function decode{name}(json) {{");
                let _ = writeln!(
                    out,
                    "  if (!{}.has(json)) throw new TypeError(`not a {name}: ${{json}}`);",
                    keys_constant(name)
                );
                out.push_str("  return json;\n}\n");
            }
            RecordShape::Tagged { tag, variants } => {
                let _ = writeln!(out, "\n/** A `{name}` from its JSON, by its `{tag}`. */");
                let _ = writeln!(out, "export function decode{name}(json) {{");
                let _ = writeln!(out, "  switch (json.{tag}) {{");
                for variant in variants {
                    let _ = writeln!(out, "    case '{}':", variant.key);
                    out.push_str("      return {\n");
                    let _ = writeln!(out, "        {}: '{}',", ts_field(tag), variant.key);
                    js_fields(&mut out, &variant.fields, "        ");
                    out.push_str("      };\n");
                }
                out.push_str("    default:\n");
                let _ = writeln!(
                    out,
                    "      throw new TypeError(`not a {name}: {tag} ${{json.{tag}}}`);"
                );
                out.push_str("  }\n}\n");
            }
        }
    }
    out
}

/// The module constant holding a key union's keys.
fn keys_constant(name: &str) -> String {
    format!("{}_KEYS", crate::names::screaming(name))
}

// ── Python ─────────────────────────────────────────────────────────────────

fn py_field(name: &str) -> String {
    reserved::renamed(&snake(name), reserved::PYTHON, "_")
}

fn py_type(ty: &RecordType) -> String {
    match ty {
        RecordType::Bool => "bool".into(),
        RecordType::Integer => "int".into(),
        RecordType::Number => "float".into(),
        RecordType::Text => "str".into(),
        RecordType::Record { name } => name.clone(),
        RecordType::List { of } => format!("tuple[{}, ...]", py_type(of)),
        RecordType::Pair { first, second } => {
            format!("tuple[{}, {}]", py_type(first), py_type(second))
        }
    }
}

/// The expression that decodes `expr`, of type `ty`, in Python.
fn py_decode(ty: &RecordType, expr: &str) -> String {
    match ty {
        RecordType::Number => format!("float({expr})"),
        RecordType::Bool | RecordType::Integer | RecordType::Text => expr.to_owned(),
        RecordType::Record { name } => format!("decode_{}({expr})", snake(name)),
        RecordType::List { of } => {
            format!("tuple({} for item in {expr})", py_decode(of, "item"))
        }
        RecordType::Pair { first, second } => format!(
            "({}, {})",
            py_decode(first, &format!("{expr}[0]")),
            py_decode(second, &format!("{expr}[1]"))
        ),
    }
}

fn py_class(
    out: &mut String,
    name: &str,
    doc: &str,
    fields: &[RecordField],
    tag: Option<(&str, &str)>,
) {
    let _ = writeln!(out, "\n\n@dataclass(frozen=True)\nclass {name}:");
    out.push_str(&python::docstring(doc, "    "));
    for field in fields {
        let nullable = if field.nullable { " | None" } else { "" };
        let _ = writeln!(
            out,
            "\n    {}: {}{nullable}",
            py_field(&field.name),
            py_type(&field.ty)
        );
        out.push_str(&python::docstring(&field.doc, "    "));
    }
    if let Some((tag, key)) = tag {
        let _ = writeln!(
            out,
            "\n    {}: Literal[\"{key}\"] = \"{key}\"",
            py_field(tag)
        );
        let _ = writeln!(out, "    \"\"\"Which variant this is.\"\"\"");
    }
}

fn py_construct(name: &str, fields: &[RecordField], indent: &str) -> String {
    let mut out = format!("{name}(\n");
    for field in fields {
        let wire = format!("raw.get(\"{}\")", field.name);
        let value = if field.nullable {
            format!(
                "None if {wire} is None else {}",
                py_decode(&field.ty, &format!("raw[\"{}\"]", field.name))
            )
        } else {
            py_decode(&field.ty, &format!("raw[\"{}\"]", field.name))
        };
        let _ = writeln!(out, "{indent}    {}={value},", py_field(&field.name));
    }
    let _ = write!(out, "{indent})");
    out
}

/// `_records.py`: every record as a frozen dataclass or a `Literal`, and
/// a `decode_*` function for each.
#[must_use]
pub fn python_records(api: &Api) -> String {
    let mut out = python::preamble(api, ABOUT);
    out.push_str(
        "from dataclasses import dataclass\nfrom typing import Any, Literal, Mapping, Union\n",
    );
    let mut exported: Vec<String> = Vec::new();
    for record in &api.records {
        exported.push(record.name.clone());
        if let RecordShape::Tagged { variants, .. } = &record.shape {
            exported.extend(
                variants
                    .iter()
                    .map(|variant| variant_name(&record.name, variant)),
            );
        }
        exported.push(format!("decode_{}", snake(&record.name)));
    }
    let listed: Vec<String> = exported
        .iter()
        .map(|name| format!("    \"{name}\",\n"))
        .collect();
    let _ = write!(out, "\n__all__ = [\n{}]\n", listed.concat());
    for record in &api.records {
        let name = &record.name;
        let decoder = format!("decode_{}", snake(name));
        match &record.shape {
            RecordShape::Object { fields } => {
                py_class(&mut out, name, &record.doc, fields, None);
                let _ = writeln!(
                    out,
                    "\n\ndef {decoder}(raw: Mapping[str, Any]) -> {name}:\n    \"\"\"A `{name}` from its JSON.\"\"\"\n    return {}",
                    py_construct(name, fields, "    ")
                );
            }
            RecordShape::Keys { values } => {
                let keys: Vec<String> = values.iter().map(|v| format!("\"{}\"", v.key)).collect();
                let listed: Vec<String> = values
                    .iter()
                    .map(|value| format!("`{}`: {}", value.key, value.doc))
                    .collect();
                let _ = writeln!(out, "\n\n{name} = Literal[{}]", keys.join(", "));
                out.push_str(&python::docstring(
                    &format!("{}\n\n{}", record.doc, listed.join("\n")),
                    "",
                ));
                let _ = writeln!(
                    out,
                    "\n_{snaked}_KEYS = frozenset(({keys},))\n\n\ndef {decoder}(raw: Any) -> {name}:\n    \"\"\"A `{name}` from its JSON: the key itself, refused if the SDK does not write it.\"\"\"\n    if raw not in _{snaked}_KEYS:\n        raise ValueError(f\"not a {name}: {{raw!r}}\")\n    return raw  # type: ignore[no-any-return]",
                    snaked = snake(name).to_ascii_uppercase(),
                    keys = keys.join(", ")
                );
            }
            RecordShape::Tagged { tag, variants } => {
                let names: Vec<String> = variants
                    .iter()
                    .map(|variant| variant_name(name, variant))
                    .collect();
                for (variant, variant_name) in variants.iter().zip(&names) {
                    py_class(
                        &mut out,
                        variant_name,
                        &variant.doc,
                        &variant.fields,
                        Some((tag, &variant.key)),
                    );
                }
                let _ = writeln!(out, "\n\n{name} = Union[{}]", names.join(", "));
                out.push_str(&python::docstring(&record.doc, ""));
                let _ = writeln!(
                    out,
                    "\n\ndef {decoder}(raw: Mapping[str, Any]) -> {name}:\n    \"\"\"A `{name}` from its JSON, by its `{tag}`.\"\"\"\n    which = raw[\"{tag}\"]"
                );
                for (variant, variant_name) in variants.iter().zip(&names) {
                    let _ = writeln!(
                        out,
                        "    if which == \"{}\":\n        return {}",
                        variant.key,
                        py_construct(variant_name, &variant.fields, "        ")
                    );
                }
                let _ = writeln!(
                    out,
                    "    raise ValueError(f\"not a {name}: {tag} {{which!r}}\")"
                );
            }
        }
    }
    out
}

// ── Dart ───────────────────────────────────────────────────────────────────

fn dart_field(name: &str) -> String {
    dart::identifier(name)
}

fn dart_type(ty: &RecordType) -> String {
    match ty {
        RecordType::Bool => "bool".into(),
        RecordType::Integer => "int".into(),
        RecordType::Number => "double".into(),
        RecordType::Text => "String".into(),
        RecordType::Record { name } => name.clone(),
        RecordType::List { of } => format!("List<{}>", dart_type(of)),
        RecordType::Pair { first, second } => {
            format!("({}, {})", dart_type(first), dart_type(second))
        }
    }
}

/// The expression that decodes `expr`, an `Object?` of type `ty`, in Dart.
fn dart_decode(api: &Api, ty: &RecordType, expr: &str) -> String {
    match ty {
        RecordType::Bool => format!("{expr}! as bool"),
        RecordType::Integer => format!("({expr}! as num).toInt()"),
        RecordType::Number => format!("({expr}! as num).toDouble()"),
        RecordType::Text => format!("{expr}! as String"),
        RecordType::Record { name } => {
            if is_keys(api, name) {
                format!("{name}.fromJson({expr})")
            } else {
                format!("{name}.fromJson({expr}! as Map<String, Object?>)")
            }
        }
        RecordType::List { of } => format!(
            "[for (final item in {expr}! as List<Object?>) {}]",
            dart_decode(api, of, "item")
        ),
        RecordType::Pair { first, second } => format!(
            "_pair({expr}, (a) => {}, (b) => {})",
            dart_decode(api, first, "a"),
            dart_decode(api, second, "b")
        ),
    }
}

fn dart_class(
    out: &mut String,
    api: &Api,
    name: &str,
    doc: &str,
    fields: &[RecordField],
    parent: Option<(&str, &str, &str)>,
) {
    out.push('\n');
    out.push_str(&dart::doc(doc, ""));
    match parent {
        Some((union, _, _)) => {
            let _ = writeln!(out, "final class {name} extends {union} {{");
        }
        None => {
            let _ = writeln!(out, "final class {name} {{");
        }
    }
    let _ = writeln!(out, "  /// A {name} with every field named.");
    if fields.is_empty() {
        let _ = writeln!(out, "  const {name}();");
    } else {
        let named: Vec<String> = fields
            .iter()
            .map(|field| format!("required this.{}", dart_field(&field.name)))
            .collect();
        let _ = writeln!(out, "  const {name}({{{}}});", named.join(", "));
    }
    let _ = writeln!(out, "\n  /// A {name} from its JSON.");
    if fields.is_empty() {
        let _ = writeln!(
            out,
            "  factory {name}.fromJson(Map<String, Object?> json) => const {name}();"
        );
    } else {
        let _ = writeln!(
            out,
            "  factory {name}.fromJson(Map<String, Object?> json) => {name}("
        );
        for field in fields {
            let wire = format!("json['{}']", field.name);
            let decoded = dart_decode(api, &field.ty, &wire);
            let value = if field.nullable {
                format!("{wire} == null ? null : {decoded}")
            } else {
                decoded
            };
            let _ = writeln!(out, "        {}: {value},", dart_field(&field.name));
        }
        out.push_str("      );\n");
    }
    for field in fields {
        out.push('\n');
        out.push_str(&dart::doc(&field.doc, "  "));
        let nullable = if field.nullable { "?" } else { "" };
        let _ = writeln!(
            out,
            "  final {}{nullable} {};",
            dart_type(&field.ty),
            dart_field(&field.name)
        );
    }
    if let Some((_, tag, key)) = parent {
        let _ = writeln!(
            out,
            "\n  @override\n  String get {} => '{key}';",
            dart_field(tag)
        );
    }
    out.push_str("}\n");
}

/// `records.dart`: every record as a Dart class, sealed family or enum,
/// each decoded by `fromJson`.
#[must_use]
pub fn dart_records(api: &Api) -> String {
    let mut out = dart::preamble(api, "");
    out.push_str(&dart::doc(ABOUT, ""));
    out.push_str("library;\n\n/// A pair from a two-item JSON array, each item by its own decoder.\n(A, B) _pair<A, B>(Object? json, A Function(Object?) first, B Function(Object?) second) {\n  final items = json! as List<Object?>;\n  return (first(items[0]), second(items[1]));\n}\n");
    for record in &api.records {
        let name = &record.name;
        match &record.shape {
            RecordShape::Object { fields } => {
                dart_class(&mut out, api, name, &record.doc, fields, None);
            }
            RecordShape::Keys { values } => {
                out.push('\n');
                out.push_str(&dart::doc(&record.doc, ""));
                let _ = writeln!(out, "enum {name} {{");
                for (index, value) in values.iter().enumerate() {
                    out.push_str(&dart::doc(&value.doc, "  "));
                    let end = if index + 1 == values.len() { ";" } else { "," };
                    let _ = writeln!(
                        out,
                        "  {}('{}'){end}",
                        dart_field(&value.key.to_ascii_lowercase()),
                        value.key
                    );
                }
                let _ = writeln!(
                    out,
                    "\n  const {name}(this.key);\n\n  /// The key, as the wire writes it.\n  final String key;\n\n  /// The {name} a key names.\n  static {name} fromJson(Object? json) => values.firstWhere(\n        (value) => value.key == json,\n        orElse: () => throw FormatException('not a {name}: $json'),\n      );\n}}"
                );
            }
            RecordShape::Tagged { tag, variants } => {
                out.push('\n');
                out.push_str(&dart::doc(&record.doc, ""));
                let _ = writeln!(out, "sealed class {name} {{\n  const {name}();\n");
                let _ = writeln!(out, "  /// A {name} from its JSON, by its `{tag}`.");
                let _ = writeln!(
                    out,
                    "  factory {name}.fromJson(Map<String, Object?> json) => switch (json['{tag}']) {{"
                );
                for variant in variants {
                    let _ = writeln!(
                        out,
                        "        '{}' => {}.fromJson(json),",
                        variant.key,
                        variant_name(name, variant)
                    );
                }
                let _ = writeln!(
                    out,
                    "        final other => throw FormatException('not a {name}: {tag} $other'),\n      }};\n\n  /// Which variant this is, as the wire's `{tag}` names it.\n  String get {};\n}}",
                    dart_field(tag)
                );
                for variant in variants {
                    dart_class(
                        &mut out,
                        api,
                        &variant_name(name, variant),
                        &variant.doc,
                        &variant.fields,
                        Some((name, tag, &variant.key)),
                    );
                }
            }
        }
    }
    out
}
