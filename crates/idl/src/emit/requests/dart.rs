//! Dart: every request record as a class the layer writes, in a part of
//! `teistro.dart` so the records share its value equality (`_Value`).
//!
//! An object is a `final class` with a `const` constructor: a field serde
//! requires is `required`, one serde defaults takes the schema's default
//! where Dart can write it as a constant, and is nullable and left off the
//! wire otherwise, so serde gives it its own. A closed key set is an `enum`
//! with its `key`; a union is a `sealed class` of one `final class` per
//! variant. Each writes its JSON as `_wire`, a catalogue member as its key.

use std::fmt::Write;

use serde_json::Value;

use super::{Types, key_part, variant_name};
use crate::emit::dart::{doc, identifier, preamble};
use crate::model::Api;
use crate::names::binding_type_name;
use crate::records::{ExternalVariant, RecordField, RecordKey, RecordShape, RecordType};

/// What the Dart requests file says it is, after the header.
const DART_ABOUT: &str = "The JSON requests the layer hands through to the SDK, as classes. Rendered from the façade's own schema of each request, which schemars reads from serde's description of the Rust types, so a request is typed as serde reads it: a field serde defaults takes the schema's default, and a catalogue member is written as its key.";

/// Why a façade is kept: it names presets the layer gives (`AlmutenRules.lilly`).
const PRESET: &str = "names presets the layer gives (`AlmutenRules.lilly`)";
/// Why a façade is kept: it is built positionally, and the layer decodes it as an answer too.
const POSITIONAL: &str = "is built positionally, and the layer decodes it as an answer too";
/// Why a façade is kept: it is built by factories that check what they are given.
const METHODS: &str = "is built by factories that check what they are given";
/// Why a façade is kept: it is a union the layer builds through its own constructors.
const UNION: &str = "a union the layer builds through its own constructors";
/// Why a façade is kept: it names its parts by the layer's own types, clearer than the schema's (`PakshiClock` for `Clock`).
const NAMES: &str = "names its parts by the layer's own types, clearer than the schema's (`PakshiClock` for `Clock`)";
/// Why a façade is kept: it takes its parts in the layer's own shape (a place as an `Observer`, a table by its parts).
const SHAPE: &str =
    "takes its parts in the layer's own shape (a place as an `Observer`, a table by its parts)";

/// The request records the Dart layer keeps by hand, which this file
/// leaves out because `teistro.dart` declares them: each takes a shape of
/// its own (presets, a decoder, a method) and writes its JSON as `_wire`.
/// `cargo xtask check-lints` holds each to a reason
/// (`layer-does-not-twin-a-request`).
pub const DART_FACADES: &[(&str, &str)] = &[
    ("AccidentalRules", PRESET),
    ("AccidentalScores", PRESET),
    ("AlmutenRules", PRESET),
    ("AnnualPlace", UNION),
    ("AntisciaRequest", NAMES),
    ("AshauchEvent", UNION),
    ("AshauchLength", UNION),
    ("AshtakavargaAyusRules", NAMES),
    ("AttendantFacts", SHAPE),
    ("AyurdayaRules", PRESET),
    ("BaselineDashaRules", SHAPE),
    ("BaselineRectificationRequest", SHAPE),
    ("BirthDate", POSITIONAL),
    ("BirthFacts", SHAPE),
    ("CircumstanceRequest", SHAPE),
    ("ConsiderationRules", PRESET),
    ("DignityRequest", SHAPE),
    ("DignityScores", PRESET),
    ("DossierRequest", SHAPE),
    ("FestivalRequest", METHODS),
    ("FirstYearRequest", NAMES),
    ("HarmonicRequest", POSITIONAL),
    ("HitRequest", METHODS),
    ("Interval", POSITIONAL),
    ("KujaLifts", PRESET),
    ("LalKitabCycle", SHAPE),
    ("LalKitabRequest", NAMES),
    ("LifeEvent", NAMES),
    ("LotRequest", PRESET),
    ("MasaPraveshaRequest", NAMES),
    ("MatchingRequest", SHAPE),
    ("MuhurtaRequest", METHODS),
    ("NatalPoint", UNION),
    ("NumerologyRules", PRESET),
    ("OrbModel", UNION),
    ("Partner", NAMES),
    ("PerfectionRequest", PRESET),
    ("PerfectionRules", PRESET),
    ("PrashnaRequest", SHAPE),
    ("PrashnaRules", NAMES),
    ("ProgressionContacts", SHAPE),
    ("ProgressionRate", PRESET),
    ("ProgressionsRequest", NAMES),
    ("PurifierRules", SHAPE),
    ("PurifyRequest", SHAPE),
    ("RasmiRules", PRESET),
    ("ReckonedPlace", SHAPE),
    ("RectificationRequest", NAMES),
    ("ResearchDesign", SHAPE),
    ("ResearchReferents", SHAPE),
    ("ResearchSubject", SHAPE),
    ("RuleRequest", NAMES),
    ("SarvatobhadraMotion", UNION),
    ("ThreePairsRules", NAMES),
    ("VarshaRequest", NAMES),
    ("VarsheshaRules", SHAPE),
    ("WesternAspectRequest", SHAPE),
    ("YearClock", UNION),
    ("YogaRules", SHAPE),
];

/// The request records named like a type another of the binding's files
/// already declares (a catalogue enum, an answer record), which a
/// declaration here would shadow: a caller reaches the request's own
/// through the field that takes it, typed as the JSON it is.
pub const DART_SHADOWED: &[&str] = &["CivilTime", "Partile", "Siege", "Terms", "ZoneSpec"];

/// Whether the layer keeps `name` by hand.
pub(super) fn is_facade(name: &str) -> bool {
    DART_FACADES.iter().any(|(facade, _)| *facade == name)
}

/// A Dart string literal.
pub(super) fn quoted(text: &str) -> String {
    let escaped = text
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('$', "\\$")
        .replace('\n', "\\n");
    format!("'{escaped}'")
}

/// A key as a Dart enum member (`DAY_AND_NIGHT` is `dayAndNight`).
pub(super) fn member(key: &str) -> String {
    identifier(&key.replace('-', "_"))
}

impl Types<'_> {
    /// Whether a record is written as a key: a key set, or the boundary
    /// enum it reuses.
    pub(super) fn is_key(&self, name: &str) -> bool {
        self.api.requests.enums.contains_key(name)
            || self.api.requests.records.iter().any(|record| {
                record.name == name && matches!(record.shape, RecordShape::Keys { .. })
            })
    }

    /// The Dart type a catalogue kind is.
    fn dart_kind(&self, catalogue: &str) -> String {
        self.catalogue.get(catalogue).cloned().unwrap_or_else(|| {
            unreachable!(
                "`cargo xtask gen ffi` refuses a request naming no catalogue kind (`{catalogue}`)"
            )
        })
    }

    fn dart(&self, ty: &RecordType) -> String {
        match ty {
            RecordType::Bool => "bool".into(),
            RecordType::Integer { .. } => "int".into(),
            RecordType::Number => "double".into(),
            RecordType::Text => "String".into(),
            RecordType::Json => "Object?".into(),
            RecordType::Record { name } if DART_SHADOWED.contains(&name.as_str()) => {
                "Object?".into()
            }
            RecordType::Record { name } => match self.api.requests.enums.get(name) {
                Some(boundary) => binding_type_name(boundary),
                None => name.clone(),
            },
            RecordType::Member { catalogue } => self.dart_kind(catalogue),
            RecordType::Keyed { catalogue } => format!("KeyOf<{}>", self.dart_kind(catalogue)),
            RecordType::List { of, .. } => format!("List<{}>", self.dart(of)),
            RecordType::Pair { first, second } => {
                format!("({}, {})", self.dart(first), self.dart(second))
            }
            RecordType::Map { of } => format!("Map<String, {}>", self.dart(of)),
        }
    }

    /// `value` as its JSON; `depth` names a nested list's item apart from
    /// its container's.
    fn wire(&self, ty: &RecordType, value: &str, depth: usize) -> String {
        match ty {
            RecordType::Bool
            | RecordType::Integer { .. }
            | RecordType::Number
            | RecordType::Text
            | RecordType::Json => value.to_owned(),
            RecordType::Record { name } if DART_SHADOWED.contains(&name.as_str()) => {
                value.to_owned()
            }
            RecordType::Record { name } if self.is_key(name) => format!("{value}.key"),
            RecordType::Record { .. } => format!("{value}._wire"),
            RecordType::Member { .. } => format!("{value}.key"),
            RecordType::Keyed { .. } => format!("{value}.fullKey"),
            RecordType::List { of, .. } => {
                let item = format!("e{depth}");
                let written = self.wire(of, &item, depth + 1);
                if written == item {
                    value.to_owned()
                } else {
                    format!("[for (final {item} in {value}) {written}]")
                }
            }
            RecordType::Pair { first, second } => format!(
                "[{}, {}]",
                self.wire(first, &format!("{value}.$1"), depth),
                self.wire(second, &format!("{value}.$2"), depth)
            ),
            RecordType::Map { of } => {
                let item = format!("v{depth}");
                let written = self.wire(of, &item, depth + 1);
                if written == item {
                    value.to_owned()
                } else {
                    format!("{value}.map((k{depth}, {item}) => MapEntry(k{depth}, {written}))")
                }
            }
        }
    }

    /// A schema's default as a Dart constant of `ty`, where Dart can write
    /// one; `None` leaves the field nullable for serde to default.
    pub(super) fn constant(&self, ty: &RecordType, value: &Value) -> Option<String> {
        // A façade's constructor is the layer's, and a shadowed record is
        // JSON here: neither is built from the schema.
        if let RecordType::Record { name } = ty
            && (is_facade(name) || DART_SHADOWED.contains(&name.as_str()))
            && !self.is_key(name)
        {
            return None;
        }
        match (ty, value) {
            (RecordType::Bool, Value::Bool(b)) => Some(b.to_string()),
            (RecordType::Integer { .. }, Value::Number(n)) if n.is_i64() || n.is_u64() => {
                Some(n.to_string())
            }
            (RecordType::Number, Value::Number(n)) => {
                let text = n.to_string();
                Some(if text.contains(['.', 'e', 'E']) {
                    text
                } else {
                    format!("{text}.0")
                })
            }
            (RecordType::Text, Value::String(s)) => Some(quoted(s)),
            (RecordType::Member { catalogue }, Value::String(key)) => {
                let bare = key.rsplit('.').next().unwrap_or(key);
                self.enum_member(Some(catalogue), None, bare)
            }
            (RecordType::Record { name }, Value::String(key)) => {
                match self.api.requests.enums.get(name) {
                    Some(boundary) => self.enum_member(None, Some(boundary), key),
                    None => self
                        .api
                        .requests
                        .records
                        .iter()
                        .find(|record| record.name == *name)
                        .and_then(|record| match &record.shape {
                            RecordShape::Keys { values } => values
                                .iter()
                                .any(|v| v.key == *key)
                                .then(|| format!("{name}.{}", member(key))),
                            RecordShape::External { variants } => variants
                                .iter()
                                .any(|v| v.key == *key && v.payload.is_none())
                                .then(|| format!("{name}{}()", key_part(key))),
                            _ => None,
                        }),
                }
            }
            (RecordType::Record { name }, Value::Object(given)) => {
                let record = self
                    .api
                    .requests
                    .records
                    .iter()
                    .find(|record| record.name == *name)?;
                let RecordShape::Object { fields } = &record.shape else {
                    return None;
                };
                if fields
                    .iter()
                    .any(|field| !field.nullable && !given.contains_key(&field.name))
                {
                    return None;
                }
                let mut named = Vec::new();
                for (key, item) in given {
                    let field = fields.iter().find(|field| field.name == *key)?;
                    if item.is_null() {
                        continue;
                    }
                    named.push(format!(
                        "{}: {}",
                        identifier(key),
                        self.constant(&field.ty, item)?
                    ));
                }
                Some(format!("{name}({})", named.join(", ")))
            }
            (RecordType::List { of, .. }, Value::Array(items)) => {
                let items: Option<Vec<String>> =
                    items.iter().map(|item| self.constant(of, item)).collect();
                Some(format!("[{}]", items?.join(", ")))
            }
            (RecordType::Map { of }, Value::Object(entries)) => {
                let entries: Option<Vec<String>> = entries
                    .iter()
                    .map(|(key, item)| {
                        Some(format!("{}: {}", quoted(key), self.constant(of, item)?))
                    })
                    .collect();
                Some(format!("{{{}}}", entries?.join(", ")))
            }
            _ => None,
        }
    }

    /// A boundary enum's member by its key: the catalogue kind's, or the
    /// enum named.
    pub(super) fn enum_member(
        &self,
        kind: Option<&str>,
        name: Option<&str>,
        key: &str,
    ) -> Option<String> {
        let e = self.api.enums.iter().find(|e| {
            kind.is_some_and(|kind| e.kind.as_deref() == Some(kind))
                || name.is_some_and(|name| e.name == name)
        })?;
        let value = e.values.iter().find(|value| value.key == key)?;
        Some(format!(
            "{}.{}",
            binding_type_name(&e.name),
            identifier(&value.name)
        ))
    }

    /// A class's fields, constructor, JSON and equality; `tag` is the
    /// field and key a tagged union's variant writes first.
    fn dart_class(
        &self,
        out: &mut String,
        name: &str,
        doc_text: &str,
        extends: &str,
        tag: Option<(&str, &str)>,
        fields: &[RecordField],
    ) {
        let _ = writeln!(
            out,
            "{}final class {name} extends {extends} {{",
            doc(doc_text, "")
        );
        let mut params = Vec::new();
        let mut declared = String::new();
        let mut written = Vec::new();
        let mut equal = Vec::new();
        if let Some((tag, key)) = tag {
            written.push(format!("{}: {}", quoted(tag), quoted(key)));
        }
        for field in fields {
            let field_name = identifier(&field.name);
            let ty = self.dart(&field.ty);
            let constant = field
                .default
                .as_ref()
                .and_then(|value| self.constant(&field.ty, value));
            let declared_ty = if !field.nullable {
                params.push(format!("required this.{field_name}"));
                ty
            } else if let Some(constant) = constant {
                // A list, a map or a constructor call is constant only
                // when it says so; an enum member or a literal already is.
                let prefix = if constant.starts_with(['[', '{']) || constant.ends_with(')') {
                    "const "
                } else {
                    ""
                };
                params.push(format!("this.{field_name} = {prefix}{constant}"));
                ty
            } else {
                params.push(format!("this.{field_name}"));
                if ty.ends_with('?') {
                    ty
                } else {
                    format!("{ty}?")
                }
            };
            // A raw part's `Object?` cannot tell JSON's null from a part not
            // given, so an optional one is left off the wire when null: serde
            // then gives its default, where a null it may not take (a theme
            // patch's `style`) would be refused. A required one is written.
            let nullable = declared_ty.ends_with('?')
                && (field.nullable || !matches!(field.ty, RecordType::Json));
            let _ = writeln!(
                declared,
                "\n{}  final {declared_ty} {field_name};",
                doc(&field.doc, "  ")
            );
            let key = quoted(&field.name);
            if nullable {
                let value = format!("{field_name}Value");
                written.push(format!(
                    "if ({field_name} case final {value}?) {key}: {}",
                    self.wire(&field.ty, &value, 0)
                ));
            } else {
                written.push(format!("{key}: {}", self.wire(&field.ty, &field_name, 0)));
            }
            equal.push(field_name);
        }
        let constructor = if params.is_empty() {
            format!("  const {name}();")
        } else {
            format!("  const {name}({{\n    {},\n  }});", params.join(",\n    "))
        };
        let _ = writeln!(out, "{constructor}");
        out.push_str(&declared);
        let wire = if written.is_empty() {
            String::from("const <String, Object?>{}")
        } else {
            format!(
                "<String, Object?>{{\n    {},\n  }}",
                written.join(",\n    ")
            )
        };
        let _ = writeln!(
            out,
            "\n  @override\n  Map<String, Object?> get _wire => {wire};\n\n  @override\n  List<Object?> get _fields => [{}];\n}}",
            equal.join(", ")
        );
    }

    fn dart_external(
        &self,
        out: &mut String,
        name: &str,
        doc_text: &str,
        variants: &[ExternalVariant],
    ) {
        sealed(out, name, doc_text);
        for variant in variants {
            let class = format!("{name}{}", key_part(&variant.key));
            match &variant.payload {
                None => dart_unit(out, &class, &variant.doc, name, &variant.key),
                Some(payload) => dart_one(
                    out,
                    &class,
                    &variant.doc,
                    name,
                    &self.dart(payload),
                    &format!(
                        "{{{}: {}}}",
                        quoted(&variant.key),
                        self.wire(payload, "value", 0)
                    ),
                ),
            }
        }
    }
}

/// A variant holding one value, which `wire` writes as its JSON.
fn dart_one(out: &mut String, name: &str, doc_text: &str, extends: &str, ty: &str, wire: &str) {
    let _ = writeln!(
        out,
        "\n{}final class {name} extends {extends} {{\n  const {name}(this.value);\n\n  /// The value.\n  final {ty} value;\n\n  @override\n  Object? get _wire => {wire};\n\n  @override\n  List<Object?> get _fields => [value];\n}}",
        doc(doc_text, "")
    );
}

/// A variant that is its key alone.
fn dart_unit(out: &mut String, name: &str, doc_text: &str, extends: &str, key: &str) {
    let _ = writeln!(
        out,
        "\n{}final class {name} extends {extends} {{\n  const {name}();\n\n  @override\n  Object? get _wire => {};\n\n  @override\n  List<Object?> get _fields => const [];\n}}",
        doc(doc_text, ""),
        quoted(key)
    );
}

/// A closed key set as an enum the layer's decoders read by key
/// (`_keyedIn`).
fn dart_enum(out: &mut String, name: &str, doc_text: &str, values: &[RecordKey]) {
    let _ = writeln!(out, "{}enum {name} implements _Keyed {{", doc(doc_text, ""));
    let last = values.len().saturating_sub(1);
    for (i, value) in values.iter().enumerate() {
        let end = if i == last { ";" } else { "," };
        let _ = writeln!(
            out,
            "{}  {}({}){end}",
            doc(&value.doc, "  "),
            member(&value.key),
            quoted(&value.key)
        );
    }
    let _ = writeln!(
        out,
        "\n  const {name}(this.key);\n\n  /// The key the wire spells it with.\n  @override\n  final String key;\n}}"
    );
}

/// A union's sealed root, which each variant's class extends.
fn sealed(out: &mut String, name: &str, doc_text: &str) {
    let _ = writeln!(
        out,
        "{}sealed class {name} extends _Request {{\n  const {name}();\n}}",
        doc(doc_text, "")
    );
}

/// `requests.dart`: every request record as a Dart class, a part of
/// `teistro.dart`.
#[must_use]
pub fn dart_declarations(api: &Api) -> String {
    let types = Types::of(api);
    let mut out = preamble(api, "");
    out.push_str(&doc(DART_ABOUT, ""));
    out.push_str("part of '../teistro.dart';\n\n");
    out.push_str(&doc(
        "A request record: a value compared field by field, written as its JSON.",
        "",
    ));
    out.push_str("abstract base class _Request extends _Value {\n  const _Request();\n\n  /// Its JSON, as the SDK reads it.\n  Object? get _wire;\n\n  /// Its JSON as the SDK reads it: maps, lists, strings, numbers and\n  /// booleans, a catalogue member as its key. `jsonEncode` calls it, so a\n  /// request can be stored or logged as the text that crosses.\n  Object? toJson() => _wire;\n}\n");
    for record in &api.requests.records {
        let name = record.name.as_str();
        if api.requests.enums.contains_key(name) || is_facade(name) || DART_SHADOWED.contains(&name)
        {
            continue;
        }
        out.push('\n');
        match &record.shape {
            RecordShape::Object { fields } => {
                types.dart_class(&mut out, name, &record.doc, "_Request", None, fields);
            }
            RecordShape::Keys { values } => dart_enum(&mut out, name, &record.doc, values),
            RecordShape::Tagged { tag, variants } => {
                sealed(&mut out, name, &record.doc);
                for variant in variants {
                    out.push('\n');
                    types.dart_class(
                        &mut out,
                        &variant_name(name, variant),
                        &variant.doc,
                        name,
                        Some((tag, &variant.key)),
                        &variant.fields,
                    );
                }
            }
            RecordShape::External { variants } => {
                types.dart_external(&mut out, name, &record.doc, variants);
            }
            RecordShape::OrNamed {
                names,
                values,
                member: kind,
            } => {
                sealed(&mut out, name, &record.doc);
                for named in names {
                    dart_unit(
                        &mut out,
                        &format!("{name}{}", key_part(named)),
                        &format!("`{named}`."),
                        name,
                        named,
                    );
                }
                if let Some(kind) = kind {
                    dart_one(
                        &mut out,
                        &format!("{name}Member"),
                        "A catalogue member, written as its key.",
                        name,
                        &types.dart_kind(kind),
                        "value.key",
                    );
                }
                let several = values.len() > 1;
                for value in values {
                    let doc = match value.when.as_slice() {
                        [] if several => {
                            String::from("The value, when it carries none of the others' keys.")
                        }
                        [] => String::from("The value itself."),
                        keys => format!(
                            "The value, picked by {}.",
                            keys.iter()
                                .map(|k| format!("`{k}`"))
                                .collect::<Vec<_>>()
                                .join(" or ")
                        ),
                    };
                    dart_one(
                        &mut out,
                        &format!("{name}{}", value.class_suffix(several)),
                        &doc,
                        name,
                        &types.dart(&value.ty),
                        &types.wire(&value.ty, "value", 0),
                    );
                }
            }
        }
    }
    out
}
