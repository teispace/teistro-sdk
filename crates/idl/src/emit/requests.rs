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

use crate::emit::{block_comment, ts};
use crate::model::Api;
use crate::names::{binding_type_name, pascal};
use crate::records::{RecordField, RecordShape, RecordType, RecordVariant};

/// What the TypeScript requests file says it is, after the header.
const TS_ABOUT: &str = "The JSON requests the layers hand through to the SDK, typed. Rendered from the façade's own schema of each request, which schemars reads from serde's description of the Rust types, so a request is typed as serde reads it: a field keeps its wire name and is optional where serde defaults it.";

/// The name a variant of a union tagged by a field is given: the union's,
/// then the variant's key in the language's case, as an answer record's
/// (`planet-in-house` is `ConditionPlanetInHouse`).
fn variant_name(union: &str, variant: &RecordVariant) -> String {
    format!(
        "{union}{}",
        pascal(&variant.key.to_ascii_lowercase().replace('-', "_"))
    )
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

/// What the TypeScript emitter knows besides a record.
struct TsTypes<'a> {
    api: &'a Api,
    /// Each catalogue kind's type (`graha` is `Graha`).
    catalogue: BTreeMap<&'a str, String>,
    /// The boundary enums that are a catalogue kind, whose TypeScript type
    /// is the full keys (`sex.MALE`) where a request writes the bare ones.
    kinded: BTreeMap<&'a str, String>,
}

impl<'a> TsTypes<'a> {
    fn of(api: &'a Api) -> TsTypes<'a> {
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
        TsTypes {
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
    let types = TsTypes::of(api);
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
            RecordShape::OrNamed { names, of, member } => {
                out.push_str(&block_comment(&record.doc, ""));
                let mut arms: Vec<String> = names.iter().map(|name| format!("'{name}'")).collect();
                if let Some(kind) = member {
                    arms.push(format!("Member<{}>", types.kind(kind)));
                }
                arms.push(types.ty(of));
                let _ = writeln!(out, "export type {} = {};", record.name, arms.join(" | "));
            }
        }
    }
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
    fn rendered() -> String {
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
                "with-dash": {"type": "string"}
            }, "required": ["orbs", "how", "with-dash"]},
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
        let api = Api {
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
        };
        typescript_declarations(&api)
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
        let types = TsTypes::of(&api);
        assert_eq!(types.record("Ephemeris"), "catalogue.Ephemeris");
        assert_eq!(types.record("Rules"), "Rules");
        assert!(is_identifier("_a$1") && !is_identifier("1a") && !is_identifier("a-b"));
    }
}
