//! The JSON records that cross the boundary as text: a result's provenance
//! and a positions result's completion steps.
//!
//! They cross as canonical JSON (a blob's `provenance` section, its
//! `steps`), and every binding parsed them into an untyped map, so a
//! consumer read `provenance["settings_hash"]` with nothing to say the
//! field existed. The shape is serde's, and serde's own schema of it
//! (schemars) is what this reads: [`from_schema`] turns a JSON Schema's
//! definitions into [`RecordDef`]s, and every emitter renders the same
//! records as its language's typed values with a decoder beside them.
//!
//! The conversion reads a closed subset of JSON Schema — objects,
//! string-key unions, unions tagged by one constant field, unions of a key
//! or a one-key object (serde's external tagging), a sentinel or a value,
//! catalogue members, maps, lists, pairs and the scalars — and **refuses**
//! anything else by its path, so a field of a new shape stops the
//! generator rather than crossing as whatever the nearest guess was. A
//! part that is a JSON value and nothing narrower (a rule's body) is read
//! as one only when [`Reading::raw`] lists it, and a listed part nothing
//! reaches is refused too.
//!
//! An object, a union or a set of keys written inline, with no definition
//! of its own, is named after where it is: its record's name and its
//! field's (`Partile`'s `WITHIN` is `PartileWithin`).
//!
//! ```
//! use teistro_idl::records::{RecordShape, RecordType, from_schema};
//!
//! let schema = serde_json::json!({"$defs": {
//!     "Stamp": {"type": "object", "description": "A stamp.",
//!         "properties": {"name": {"type": "string", "description": "Its name."},
//!                        "note": {"type": ["string", "null"], "description": "A note."}},
//!         "required": ["name", "note"]}
//! }});
//! let records = from_schema(&schema, &["Stamp"])?;
//! let RecordShape::Object { fields } = &records[0].shape else { unreachable!() };
//! assert_eq!(fields[0].ty, RecordType::Text);
//! assert!(fields[1].nullable);
//! # Ok::<(), String>(())
//! ```

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::names::pascal;

/// A record that crosses as JSON, named as its schema names it.
///
/// ```
/// # use teistro_idl::records::{RecordShape, RecordType, from_schema};
/// #
/// # let schema = serde_json::json!({"$defs": {
/// #     "Stamp": {"type": "object", "description": "A stamp.", "properties": {
/// #         "hash": {"type": "string", "description": "Its hash."},
/// #         "sure": {"$ref": "#/$defs/Sure"},
/// #         "how": {"anyOf": [{"$ref": "#/$defs/How"}, {"type": "null"}]},
/// #         "year": {"type": "integer", "format": "uint8"}
/// #     }, "required": ["hash", "sure", "how", "year"]},
/// #     "Sure": {"oneOf": [{"const": "VERIFIED", "type": "string", "description": "Checked."},
/// #                        {"const": "UNVERIFIED", "type": "string"}]},
/// #     "How": {"oneOf": [
/// #         {"type": "object", "properties": {"kind": {"const": "DEFINED"}}, "required": ["kind"]},
/// #         {"type": "object", "properties": {"kind": {"const": "TABULAR"}, "edition": {"type": "string"}}, "required": ["kind", "edition"]}
/// #     ]}
/// # }});
/// # let records = from_schema(&schema, &["Stamp"])?;
///
/// // Roots first, then what they reach, field by field in name order.
/// let names: Vec<&str> = records.iter().map(|record| record.name.as_str()).collect();
/// assert_eq!(names, ["Stamp", "How", "Sure"]);
/// assert_eq!(records[0].doc, "A stamp.");
/// # Ok::<(), String>(())
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordDef {
    /// Its name, as the Rust type's schema names it (`Provenance`).
    pub name: String,
    /// Its documentation's first paragraph.
    pub doc: String,
    /// What it is made of.
    pub shape: RecordShape,
}

/// What a record is made of.
///
/// ```
/// # use teistro_idl::records::{RecordShape, RecordType, from_schema};
/// #
/// # let schema = serde_json::json!({"$defs": {
/// #     "Stamp": {"type": "object", "description": "A stamp.", "properties": {
/// #         "hash": {"type": "string", "description": "Its hash."},
/// #         "sure": {"$ref": "#/$defs/Sure"},
/// #         "how": {"anyOf": [{"$ref": "#/$defs/How"}, {"type": "null"}]},
/// #         "year": {"type": "integer", "format": "uint8"}
/// #     }, "required": ["hash", "sure", "how", "year"]},
/// #     "Sure": {"oneOf": [{"const": "VERIFIED", "type": "string", "description": "Checked."},
/// #                        {"const": "UNVERIFIED", "type": "string"}]},
/// #     "How": {"oneOf": [
/// #         {"type": "object", "properties": {"kind": {"const": "DEFINED"}}, "required": ["kind"]},
/// #         {"type": "object", "properties": {"kind": {"const": "TABULAR"}, "edition": {"type": "string"}}, "required": ["kind", "edition"]}
/// #     ]}
/// # }});
/// # let records = from_schema(&schema, &["Stamp"])?;
///
/// assert!(matches!(&records[0].shape, RecordShape::Object { fields } if fields.len() == 4));
/// assert!(matches!(&records[1].shape, RecordShape::Tagged { tag, .. } if tag == "kind"));
/// assert!(matches!(&records[2].shape, RecordShape::Keys { values } if values.len() == 2));
/// # Ok::<(), String>(())
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecordShape {
    /// Named fields.
    Object {
        /// The fields, in name order, which is the canonical wire's and
        /// holds whatever order a JSON library keeps a map in.
        fields: Vec<RecordField>,
    },
    /// One of a closed set of keys, written as the key.
    Keys {
        /// The keys, in the order the type declares them.
        values: Vec<RecordKey>,
    },
    /// One of several records, told apart by one field whose value is a
    /// key.
    Tagged {
        /// The field that names the variant (`kind`).
        tag: String,
        /// The variants, in the order the type declares them.
        variants: Vec<RecordVariant>,
    },
    /// A key, or an object of one field named by its key and holding the
    /// variant's value (`"SAME_DEGREE"` or `{"WITHIN": {"orbDeg": 1}}`),
    /// which is how serde writes an enum by default.
    External {
        /// The variants, in the order the type declares them.
        variants: Vec<ExternalVariant>,
    },
    /// One of a few fixed strings, or a value of one type (`"birth"` or a
    /// place; a criterion's name or the criterion spelt out), or a member
    /// of one catalogue kind beside them (a natal point: `"LAGNA"`, a
    /// graha's key, or the point spelt out).
    ///
    /// The value may be one of several records, untagged, each with the
    /// keys that pick it (a festival rule spelt out: `windows` a sait,
    /// `vedha` an Ekadashi), the last picked when no other is.
    OrNamed {
        /// The strings, as the wire writes them.
        names: Vec<String>,
        /// The values it may be, in the order a reader tries them; one,
        /// with no keys, for a name or a value.
        values: Vec<NamedValue>,
        /// The catalogue kind whose member it may also be, by its key.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        member: Option<String>,
    },
    /// Named fields beside another record's own, all in one object, as
    /// serde writes a record it flattens (`#[serde(flatten)]`): a
    /// cancellation is a condition with its `label` beside the condition's
    /// parts. The record laid flat is held as a field named after it
    /// (`condition`), which the wire does not write: its parts are the
    /// object's.
    Beside {
        /// The fields beside the record, in name order.
        fields: Vec<RecordField>,
        /// The record laid flat, as a field named after it.
        laid: RecordField,
    },
}

/// The value of `values` a JSON object is read as: the first whose keys
/// it carries one of, or the last, as the SDK's reader picks it.
///
/// ```
/// use teistro_idl::records::{NamedValue, RecordType, picked};
///
/// let value = |name: &str, when: &[&str]| NamedValue {
///     ty: RecordType::Record { name: name.into() },
///     when: when.iter().map(|k| (*k).to_owned()).collect(),
/// };
/// let values = [value("Sait", &["windows"]), value("Rule", &[])];
/// let sait = serde_json::json!({"key": "X", "windows": []});
/// assert_eq!(picked(&values, &sait).map(|v| &v.ty), Some(&values[0].ty));
/// assert_eq!(picked(&values, &serde_json::json!({"key": "X"})).map(|v| &v.ty), Some(&values[1].ty));
/// ```
#[must_use]
pub fn picked<'v>(values: &'v [NamedValue], object: &Value) -> Option<&'v NamedValue> {
    values
        .iter()
        .find(|value| value.when.iter().any(|key| object.get(key).is_some()))
        .or_else(|| values.last())
}

impl NamedValue {
    /// What a binding's class for this value adds to its union's name:
    /// `Value` for the only value, its record's name for one of several
    /// (`FestivalRuleItemSaitRule`).
    ///
    /// ```
    /// use teistro_idl::records::{NamedValue, RecordType};
    ///
    /// let sait = NamedValue { ty: RecordType::Record { name: "SaitRule".into() }, when: vec!["windows".into()] };
    /// assert_eq!((sait.class_suffix(true), sait.class_suffix(false)), ("SaitRule", "Value"));
    /// ```
    #[must_use]
    pub fn class_suffix(&self, several: bool) -> &str {
        match &self.ty {
            RecordType::Record { name } if several => name,
            _ => "Value",
        }
    }
}

/// One value a [`RecordShape::OrNamed`] may be, and the keys of which any
/// one present picks it; none for the last, or the only, value.
///
/// ```
/// use teistro_idl::records::{RecordShape, from_schema};
///
/// // A pack's name, or a rule spelt out: a sait has `windows`, and
/// // anything else is a rule.
/// let schema = serde_json::json!({"$defs": {
///     "Item": {"anyOf": [
///         {"type": "string", "enum": ["NEPAL"]},
///         {"$ref": "#/$defs/Sait", "x-teistro-when": ["windows"]},
///         {"$ref": "#/$defs/Rule"}
///     ]},
///     "Sait": {"type": "object", "properties": {"windows": {"type": "integer"}}, "required": ["windows"]},
///     "Rule": {"type": "object", "properties": {"key": {"type": "string"}}, "required": ["key"]}
/// }});
/// let records = from_schema(&schema, &["Item"])?;
/// let item = records.iter().find(|r| r.name == "Item").ok_or("read")?;
/// let RecordShape::OrNamed { values, .. } = &item.shape else { return Err("or-named".into()) };
/// assert_eq!(values.iter().map(|v| v.when.clone()).collect::<Vec<_>>(), [vec!["windows".to_owned()], vec![]]);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NamedValue {
    /// The value's type.
    pub ty: RecordType,
    /// The keys of which any one present picks it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub when: Vec<String>,
}

/// One variant of a [`RecordShape::External`] record.
///
/// ```
/// use teistro_idl::records::{RecordShape, RecordType, from_schema};
///
/// // How serde writes an enum by default: a bare key, or a key holding a value.
/// let schema = serde_json::json!({"$defs": {"Orb": {"oneOf": [
///     {"type": "string", "const": "SAME_DEGREE", "description": "Exact."},
///     {"type": "object", "properties": {"DAYS": {"type": "number"}}, "required": ["DAYS"], "additionalProperties": false}
/// ]}}});
/// let records = from_schema(&schema, &["Orb"])?;
/// let RecordShape::External { variants } = &records[0].shape else { return Err("external".into()) };
/// assert_eq!((variants[0].key.as_str(), &variants[0].payload), ("SAME_DEGREE", &None));
/// assert_eq!(variants[1].payload, Some(RecordType::Number));
/// # Ok::<(), String>(())
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExternalVariant {
    /// The key that names it (`WITHIN`).
    pub key: String,
    /// What it means.
    pub doc: String,
    /// What the key holds; `None` for a variant written as the bare key.
    pub payload: Option<RecordType>,
}

/// One key of a [`RecordShape::Keys`] record.
///
/// ```
/// # use teistro_idl::records::{RecordShape, RecordType, from_schema};
/// #
/// # let schema = serde_json::json!({"$defs": {
/// #     "Stamp": {"type": "object", "description": "A stamp.", "properties": {
/// #         "hash": {"type": "string", "description": "Its hash."},
/// #         "sure": {"$ref": "#/$defs/Sure"},
/// #         "how": {"anyOf": [{"$ref": "#/$defs/How"}, {"type": "null"}]},
/// #         "year": {"type": "integer", "format": "uint8"}
/// #     }, "required": ["hash", "sure", "how", "year"]},
/// #     "Sure": {"oneOf": [{"const": "VERIFIED", "type": "string", "description": "Checked."},
/// #                        {"const": "UNVERIFIED", "type": "string"}]},
/// #     "How": {"oneOf": [
/// #         {"type": "object", "properties": {"kind": {"const": "DEFINED"}}, "required": ["kind"]},
/// #         {"type": "object", "properties": {"kind": {"const": "TABULAR"}, "edition": {"type": "string"}}, "required": ["kind", "edition"]}
/// #     ]}
/// # }});
/// # let records = from_schema(&schema, &["Stamp"])?;
///
/// let RecordShape::Keys { values } = &records[2].shape else { return Err("keys".into()) };
/// assert_eq!((values[0].key.as_str(), values[0].doc.as_str()), ("VERIFIED", "Checked."));
/// # Ok::<(), String>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordKey {
    /// The key, as the wire writes it (`VERIFIED`).
    pub key: String,
    /// What it means.
    pub doc: String,
}

/// One variant of a [`RecordShape::Tagged`] record.
///
/// ```
/// # use teistro_idl::records::{RecordShape, RecordType, from_schema};
/// #
/// # let schema = serde_json::json!({"$defs": {
/// #     "Stamp": {"type": "object", "description": "A stamp.", "properties": {
/// #         "hash": {"type": "string", "description": "Its hash."},
/// #         "sure": {"$ref": "#/$defs/Sure"},
/// #         "how": {"anyOf": [{"$ref": "#/$defs/How"}, {"type": "null"}]},
/// #         "year": {"type": "integer", "format": "uint8"}
/// #     }, "required": ["hash", "sure", "how", "year"]},
/// #     "Sure": {"oneOf": [{"const": "VERIFIED", "type": "string", "description": "Checked."},
/// #                        {"const": "UNVERIFIED", "type": "string"}]},
/// #     "How": {"oneOf": [
/// #         {"type": "object", "properties": {"kind": {"const": "DEFINED"}}, "required": ["kind"]},
/// #         {"type": "object", "properties": {"kind": {"const": "TABULAR"}, "edition": {"type": "string"}}, "required": ["kind", "edition"]}
/// #     ]}
/// # }});
/// # let records = from_schema(&schema, &["Stamp"])?;
///
/// let RecordShape::Tagged { variants, .. } = &records[1].shape else { return Err("tagged".into()) };
/// // The tag is not among a variant's fields.
/// assert!(variants[0].fields.is_empty());
/// assert_eq!((variants[1].key.as_str(), variants[1].fields[0].name.as_str()), ("TABULAR", "edition"));
/// # Ok::<(), String>(())
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordVariant {
    /// The tag's value that names it (`TABULAR`).
    pub key: String,
    /// What it means.
    pub doc: String,
    /// Its fields besides the tag.
    pub fields: Vec<RecordField>,
}

/// One field of a record.
///
/// ```
/// # use teistro_idl::records::{RecordShape, RecordType, from_schema};
/// #
/// # let schema = serde_json::json!({"$defs": {
/// #     "Stamp": {"type": "object", "description": "A stamp.", "properties": {
/// #         "hash": {"type": "string", "description": "Its hash."},
/// #         "sure": {"$ref": "#/$defs/Sure"},
/// #         "how": {"anyOf": [{"$ref": "#/$defs/How"}, {"type": "null"}]},
/// #         "year": {"type": "integer", "format": "uint8"}
/// #     }, "required": ["hash", "sure", "how", "year"]},
/// #     "Sure": {"oneOf": [{"const": "VERIFIED", "type": "string", "description": "Checked."},
/// #                        {"const": "UNVERIFIED", "type": "string"}]},
/// #     "How": {"oneOf": [
/// #         {"type": "object", "properties": {"kind": {"const": "DEFINED"}}, "required": ["kind"]},
/// #         {"type": "object", "properties": {"kind": {"const": "TABULAR"}, "edition": {"type": "string"}}, "required": ["kind", "edition"]}
/// #     ]}
/// # }});
/// # let records = from_schema(&schema, &["Stamp"])?;
///
/// let RecordShape::Object { fields } = &records[0].shape else { return Err("an object".into()) };
/// // In name order; a union with null is nullable.
/// let names: Vec<&str> = fields.iter().map(|field| field.name.as_str()).collect();
/// assert_eq!(names, ["hash", "how", "sure", "year"]);
/// assert!(fields[1].nullable && !fields[0].nullable);
/// assert!(fields[1].null, "written as null, not only left out");
/// assert_eq!(fields[2].ty, RecordType::Record { name: "Sure".into() });
/// # Ok::<(), String>(())
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordField {
    /// Its name as the wire writes it (`settings_hash`); each emitter
    /// cases it for its language.
    pub name: String,
    /// What it holds.
    pub doc: String,
    /// Its type.
    pub ty: RecordType,
    /// Whether it may be null or absent; a decoder answers null for both.
    pub nullable: bool,
    /// Whether the wire may hold null for it, where [`RecordField::nullable`]
    /// may mean only absent: a request's `Option` takes null, a field serde
    /// defaults does not.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub null: bool,
    /// The value serde gives it when it is left out, as the schema states
    /// it, where that is not null: what a binding with constant defaults
    /// (Dart) writes in the constructor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
}

/// The type of a record's field.
///
/// ```
/// use teistro_idl::records::RecordType;
///
/// let grahas = RecordType::List { of: Box::new(RecordType::Member { catalogue: "graha".into() }), len: None };
/// assert_eq!(serde_json::to_string(&grahas)?, r#"{"kind":"LIST","of":{"kind":"MEMBER","catalogue":"graha"}}"#);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecordType {
    /// `true` or `false`.
    Bool,
    /// A whole number.
    Integer {
        /// Its width, when the schema states one (`format: uint8`).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        width: Option<Width>,
    },
    /// A double.
    Number,
    /// Text, including a named string such as a hash.
    Text,
    /// Another record, by name.
    Record {
        /// The record's name.
        name: String,
    },
    /// A list of one type.
    List {
        /// What each item is.
        of: Box<RecordType>,
        /// How many items it holds, where that is fixed (a Rust array:
        /// `minItems` equal to `maxItems`); absent for any number.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        len: Option<u32>,
    },
    /// Two values of fixed types, written as a two-item array.
    Pair {
        /// The first's type.
        first: Box<RecordType>,
        /// The second's type.
        second: Box<RecordType>,
    },
    /// A member of a catalogue kind, written as its key, bare or full.
    Member {
        /// The catalogue kind (`graha`).
        catalogue: String,
    },
    /// A member of a catalogue kind, or a key a context registered under
    /// it, written as the key, bare or full.
    Keyed {
        /// The catalogue kind (`dasha_system`).
        catalogue: String,
    },
    /// An object whose keys are text and whose values are of one type.
    Map {
        /// What each value is.
        of: Box<RecordType>,
    },
    /// Any JSON value: a part [`Reading::raw`] lists.
    Json,
}

/// A whole number's width, as the schema's `format` states it.
///
/// ```
/// # use teistro_idl::records::{RecordShape, RecordType, from_schema};
/// #
/// # let schema = serde_json::json!({"$defs": {
/// #     "Stamp": {"type": "object", "description": "A stamp.", "properties": {
/// #         "hash": {"type": "string", "description": "Its hash."},
/// #         "sure": {"$ref": "#/$defs/Sure"},
/// #         "how": {"anyOf": [{"$ref": "#/$defs/How"}, {"type": "null"}]},
/// #         "year": {"type": "integer", "format": "uint8"}
/// #     }, "required": ["hash", "sure", "how", "year"]},
/// #     "Sure": {"oneOf": [{"const": "VERIFIED", "type": "string", "description": "Checked."},
/// #                        {"const": "UNVERIFIED", "type": "string"}]},
/// #     "How": {"oneOf": [
/// #         {"type": "object", "properties": {"kind": {"const": "DEFINED"}}, "required": ["kind"]},
/// #         {"type": "object", "properties": {"kind": {"const": "TABULAR"}, "edition": {"type": "string"}}, "required": ["kind", "edition"]}
/// #     ]}
/// # }});
/// # let records = from_schema(&schema, &["Stamp"])?;
///
/// use teistro_idl::records::Width;
///
/// let RecordShape::Object { fields } = &records[0].shape else { return Err("an object".into()) };
/// // The schema's `format: uint8`.
/// assert_eq!(fields[3].ty, RecordType::Integer { width: Some(Width { signed: false, bits: 8 }) });
/// # Ok::<(), String>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Width {
    /// Whether it may be negative.
    pub signed: bool,
    /// How many bits it has.
    pub bits: u8,
}

/// The JSON request records a binding writes, read from the façade's own
/// schemas of them, so a binding can type a request as serde reads it.
///
/// ```
/// use teistro_idl::records::Requests;
///
/// // Handed to the emitters in memory and never written into `api.json`.
/// let none = Requests::default();
/// assert!(none.roots.is_empty() && none.records.is_empty() && none.enums.is_empty());
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Requests {
    /// Each record entry point's request.
    pub roots: Vec<RequestRoot>,
    /// Every record the roots reach, roots first.
    pub records: Vec<RecordDef>,
    /// The request records that are a boundary enum (`Sex`): the same
    /// keys, so a binding uses the enum it already has. By the record's
    /// name, the enum's (Rust) name.
    pub enums: BTreeMap<String, String>,
}

/// A record entry point and the record its request is.
///
/// ```
/// use teistro_idl::records::RequestRoot;
///
/// let found = RequestRoot { record: "chart.found".into(), name: "ChartFoundRequest".into() };
/// assert_eq!(serde_json::to_value(&found)?["name"], "ChartFoundRequest");
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestRoot {
    /// The entry point (`chart.found`).
    pub record: String,
    /// The request's record (`ChartFoundRequest`).
    pub name: String,
}

/// How a schema is read.
///
/// ```
/// use teistro_idl::records::{RecordShape, RecordType, Reading, read};
///
/// let schema = serde_json::json!({"$defs": {"Rule": {"type": "object", "properties": {
///     "key": {"type": "string"}, "body": true
/// }, "required": ["key", "body"]}}});
/// // `body` is any JSON value: refused unless it is listed raw.
/// assert!(read(&schema, &["Rule"], Reading::default()).is_err());
/// let records = read(&schema, &["Rule"], Reading { raw: &["Rule.body"] })?;
/// let RecordShape::Object { fields } = &records[0].shape else { return Err("an object".into()) };
/// assert_eq!(fields[0].ty, RecordType::Json);
/// # Ok::<(), String>(())
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct Reading<'a> {
    /// The parts read as a JSON value: a definition's name (`Subject`) or
    /// a field's path (`ThemePatch.style`). Each must be reached.
    pub raw: &'a [&'a str],
}

/// The records `roots` name and every record they reach, from a JSON
/// Schema's `$defs`, roots first and the rest in the order they are
/// reached, under the default [`Reading`] (no part read raw).
///
/// A definition that is a bare scalar with a constraint (a hash's pattern)
/// is not a record: a field referring to it takes the scalar's type.
///
/// ```
/// # use teistro_idl::records::{RecordShape, RecordType, from_schema};
/// #
/// # let schema = serde_json::json!({"$defs": {
/// #     "Stamp": {"type": "object", "description": "A stamp.", "properties": {
/// #         "hash": {"type": "string", "description": "Its hash."},
/// #         "sure": {"$ref": "#/$defs/Sure"},
/// #         "how": {"anyOf": [{"$ref": "#/$defs/How"}, {"type": "null"}]},
/// #         "year": {"type": "integer", "format": "uint8"}
/// #     }, "required": ["hash", "sure", "how", "year"]},
/// #     "Sure": {"oneOf": [{"const": "VERIFIED", "type": "string", "description": "Checked."},
/// #                        {"const": "UNVERIFIED", "type": "string"}]},
/// #     "How": {"oneOf": [
/// #         {"type": "object", "properties": {"kind": {"const": "DEFINED"}}, "required": ["kind"]},
/// #         {"type": "object", "properties": {"kind": {"const": "TABULAR"}, "edition": {"type": "string"}}, "required": ["kind", "edition"]}
/// #     ]}
/// # }});
/// # let records = from_schema(&schema, &["Stamp"])?;
///
/// // A root the schema lacks is named.
/// assert_eq!(from_schema(&schema, &["Nothing"]).unwrap_err(), "Nothing: no such definition");
/// assert_eq!(records.len(), 3);
/// # Ok::<(), String>(())
/// ```
///
/// # Errors
///
/// A root the schema does not define, a reference to a definition that is
/// not there, or any construct outside the subset, each named by its path
/// (`Provenance.module_versions`).
pub fn from_schema(schema: &Value, roots: &[&str]) -> Result<Vec<RecordDef>, String> {
    read(schema, roots, Reading::default())
}

/// [`from_schema`] under a [`Reading`].
///
/// ```
/// # use teistro_idl::records::{RecordShape, RecordType, from_schema};
/// #
/// # let schema = serde_json::json!({"$defs": {
/// #     "Stamp": {"type": "object", "description": "A stamp.", "properties": {
/// #         "hash": {"type": "string", "description": "Its hash."},
/// #         "sure": {"$ref": "#/$defs/Sure"},
/// #         "how": {"anyOf": [{"$ref": "#/$defs/How"}, {"type": "null"}]},
/// #         "year": {"type": "integer", "format": "uint8"}
/// #     }, "required": ["hash", "sure", "how", "year"]},
/// #     "Sure": {"oneOf": [{"const": "VERIFIED", "type": "string", "description": "Checked."},
/// #                        {"const": "UNVERIFIED", "type": "string"}]},
/// #     "How": {"oneOf": [
/// #         {"type": "object", "properties": {"kind": {"const": "DEFINED"}}, "required": ["kind"]},
/// #         {"type": "object", "properties": {"kind": {"const": "TABULAR"}, "edition": {"type": "string"}}, "required": ["kind", "edition"]}
/// #     ]}
/// # }});
/// # let records = from_schema(&schema, &["Stamp"])?;
///
/// use teistro_idl::records::{Reading, read};
///
/// // A part listed raw that nothing reaches is refused too.
/// let unused = read(&schema, &["Stamp"], Reading { raw: &["Stamp.nothing"] }).unwrap_err();
/// assert!(unused.starts_with("listed as raw and reached by nothing"));
/// # Ok::<(), String>(())
/// ```
///
/// # Errors
///
/// As [`from_schema`], and a raw part nothing reaches, or an inline type
/// whose name another definition already has.
pub fn read(
    schema: &Value,
    roots: &[&str],
    reading: Reading<'_>,
) -> Result<Vec<RecordDef>, String> {
    let defs = schema
        .get("$defs")
        .and_then(Value::as_object)
        .ok_or("the schema has no $defs")?;
    let mut reader = Reader {
        defs,
        raw: reading.raw,
        used: BTreeSet::new(),
        inline: BTreeMap::new(),
        reached: Vec::new(),
    };
    let mut queue: VecDeque<String> = roots.iter().map(|root| (*root).to_owned()).collect();
    let mut seen: BTreeSet<String> = queue.iter().cloned().collect();
    let mut records = Vec::new();
    while let Some(name) = queue.pop_front() {
        let def = match defs.get(&name) {
            Some(def) => def.clone(),
            None => reader
                .inline
                .get(&name)
                .cloned()
                .ok_or_else(|| format!("{name}: no such definition"))?,
        };
        let shape = reader.shape_of(&name, &def)?;
        records.push(RecordDef {
            doc: doc_of(&def),
            name,
            shape,
        });
        for next in std::mem::take(&mut reader.reached) {
            if seen.insert(next.clone()) {
                queue.push_back(next);
            }
        }
    }
    let unused: Vec<&&str> = reading
        .raw
        .iter()
        .filter(|part| !reader.used.contains(**part))
        .collect();
    if !unused.is_empty() {
        return Err(format!(
            "listed as raw and reached by nothing: {unused:?}; take each off the list"
        ));
    }
    Ok(records)
}

/// What a read keeps between definitions.
struct Reader<'s> {
    defs: &'s Map<String, Value>,
    raw: &'s [&'s str],
    used: BTreeSet<String>,
    /// The inline types named so far, by the name they were given.
    inline: BTreeMap<String, Value>,
    /// The records reached since the queue last took them.
    reached: Vec<String>,
}

impl<'s> Reader<'s> {
    /// Whether `part` is listed raw, noting that it was reached.
    fn is_raw(&mut self, part: &str) -> bool {
        let listed = self.raw.contains(&part);
        if listed {
            self.used.insert(part.to_owned());
        }
        listed
    }

    /// A definition's shape, noting the records its fields reach.
    fn shape_of(&mut self, at: &str, def: &Value) -> Result<RecordShape, String> {
        if def.get("type").and_then(Value::as_str) == Some("object")
            && def.get("properties").is_some()
        {
            if let Some(options) = def.get("oneOf").and_then(Value::as_array) {
                return self.beside_of(at, def, options);
            }
            return Ok(RecordShape::Object {
                fields: self.fields_of(at, def, None)?,
            });
        }
        if let Some(keys) = def.get("enum").and_then(Value::as_array) {
            let values = keys
                .iter()
                .map(|key| {
                    key.as_str()
                        .map(|key| RecordKey {
                            key: key.to_owned(),
                            doc: String::new(),
                        })
                        .ok_or_else(|| format!("{at}: a key that is not text"))
                })
                .collect::<Result<_, _>>()?;
            return Ok(RecordShape::Keys { values });
        }
        if let Some(options) = def.get("anyOf").and_then(Value::as_array) {
            return self.named_of(at, options);
        }
        let Some(options) = def.get("oneOf").and_then(Value::as_array) else {
            return Err(format!("{at}: neither an object nor a oneOf"));
        };
        if options.iter().all(|option| option.get("const").is_some()) {
            let values = options
                .iter()
                .map(|option| {
                    option
                        .get("const")
                        .and_then(Value::as_str)
                        .map(|key| RecordKey {
                            key: key.to_owned(),
                            doc: doc_of(option),
                        })
                        .ok_or_else(|| format!("{at}: a key that is not text"))
                })
                .collect::<Result<_, _>>()?;
            return Ok(RecordShape::Keys { values });
        }
        if options.iter().all(|option| external_key(option).is_some()) {
            let variants = options
                .iter()
                .map(|option| {
                    let (key, payload) = external_key(option).unwrap_or_default();
                    let payload = match payload {
                        Some(payload) => Some(
                            self.type_of(
                                &format!("{at}.{key}"),
                                payload,
                                &format!("{at}{}", part(&key)),
                            )?
                            .0,
                        ),
                        None => None,
                    };
                    Ok(ExternalVariant {
                        doc: doc_of(option),
                        key,
                        payload,
                    })
                })
                .collect::<Result<_, String>>()?;
            return Ok(RecordShape::External { variants });
        }
        let tag = tag_of(at, options)?;
        let variants = options
            .iter()
            .map(|option| {
                let key = option
                    .pointer(&format!("/properties/{tag}/const"))
                    .and_then(Value::as_str)
                    .ok_or_else(|| format!("{at}: a variant without its {tag}"))?
                    .to_owned();
                Ok(RecordVariant {
                    fields: self.fields_of(&format!("{at}.{key}"), option, Some(&tag))?,
                    doc: doc_of(option),
                    key,
                })
            })
            .collect::<Result<_, String>>()?;
        Ok(RecordShape::Tagged { tag, variants })
    }

    /// Fields beside a flattened record's variants. schemars writes the
    /// flattened record's variants inline, each opened to the fields beside
    /// it, so the record is found by content: the one definition whose own
    /// `oneOf` they are, once neither bars a key the other adds.
    fn beside_of(
        &mut self,
        at: &str,
        def: &Value,
        options: &[Value],
    ) -> Result<RecordShape, String> {
        let variants = opened(&Value::Array(options.to_vec()));
        let mut found = self.defs.iter().filter(|(name, other)| {
            name.as_str() != at && other.get("oneOf").map(opened).as_ref() == Some(&variants)
        });
        let (Some((name, laid_def)), None) = (found.next(), found.next()) else {
            return Err(format!(
                "{at}: fields with a `oneOf` beside them (a flattened enum) that is not one definition's own, which the reader does not model"
            ));
        };
        let name = name.clone();
        let doc = doc_of(laid_def);
        // Each binding writes the laid record's variants with the fields
        // beside them, so it is a union tagged by a field.
        tag_of(&name, options)?;
        let mut own = def.clone();
        if let Some(object) = own.as_object_mut() {
            object.remove("oneOf");
            if object.get("unevaluatedProperties") == Some(&Value::Bool(false)) {
                object.remove("unevaluatedProperties");
            }
        }
        let fields = self.fields_of(at, &own, None)?;
        let parts: BTreeSet<&str> = options
            .iter()
            .filter_map(|option| option.get("properties")?.as_object())
            .flat_map(|properties| properties.keys().map(String::as_str))
            .collect();
        if let Some(clash) = fields.iter().find(|f| parts.contains(f.name.as_str())) {
            return Err(format!(
                "{at}.{}: a field the flattened {name} writes too",
                clash.name
            ));
        }
        let ty = self.referred(at, &format!("#/$defs/{name}"))?;
        let mut chars = name.chars();
        let field = chars
            .next()
            .map(|first| first.to_lowercase().chain(chars).collect())
            .unwrap_or_default();
        Ok(RecordShape::Beside {
            fields,
            laid: RecordField {
                name: field,
                doc,
                ty,
                nullable: false,
                null: false,
                default: None,
            },
        })
    }

    /// A union of fixed strings and one other type, and at most one
    /// catalogue kind's members beside them.
    fn named_of(&mut self, at: &str, options: &[Value]) -> Result<RecordShape, String> {
        let mut names = Vec::new();
        let mut members = Vec::new();
        let mut others = Vec::new();
        for option in options {
            if let Some(fixed) = fixed_strings(option).or_else(|| self.named_strings(option)) {
                names.extend(fixed);
            } else if let Some(kind) = self.kind_of(option) {
                members.push(kind);
            } else {
                others.push(option);
            }
        }
        let member = match members.as_slice() {
            [] => None,
            [kind] => Some(kind.clone()),
            _ => return Err(format!("{at}: an anyOf of two catalogue kinds")),
        };
        if names.is_empty() || others.is_empty() {
            return Err(format!(
                "{at}: an anyOf that is neither one type or null nor fixed strings or one \
                 type; list it as raw if it is a JSON value"
            ));
        }
        let several = others.len() > 1;
        let last = others.len() - 1;
        let values = others
            .iter()
            .enumerate()
            .map(|(index, option)| {
                let when = names_of(option.get("x-teistro-when"))
                    .into_iter()
                    .collect::<Vec<_>>();
                let inline = if several {
                    format!("{at}Value{index}")
                } else {
                    format!("{at}Value")
                };
                let ty = self.type_of(&format!("{at}.value"), option, &inline)?.0;
                // Several values are told apart by their keys, so each is
                // a record, and each but the last names the keys that pick
                // it; the last is what is left.
                if several && !matches!(ty, RecordType::Record { .. }) {
                    return Err(format!("{at}: one of several values is not a record"));
                }
                match (several && index != last, when.is_empty()) {
                    (true, true) => Err(format!(
                        "{at}: value {index} of several says no keys that pick it (`x-teistro-when`)"
                    )),
                    (false, false) => Err(format!(
                        "{at}: the last or only value names keys, but it is what is left"
                    )),
                    _ => Ok(NamedValue { ty, when }),
                }
            })
            .collect::<Result<Vec<_>, String>>()?;
        // A member is read from a string, so the value beside it must not
        // be one, or a key could read as either.
        if member.is_some()
            && values
                .iter()
                .any(|value| !matches!(value.ty, RecordType::Record { .. }))
        {
            return Err(format!(
                "{at}: a catalogue member beside a value that is not a record"
            ));
        }
        Ok(RecordShape::OrNamed {
            names,
            values,
            member,
        })
    }

    /// The fixed strings a `$ref` names, where it names a def of nothing
    /// else that is no catalogue kind: schemars writes a unit enum used
    /// as one option of a union by reference, as an `enum`, or as a
    /// `oneOf` of constants when its keys are documented.
    fn named_strings(&self, option: &Value) -> Option<Vec<String>> {
        let name = option.get("$ref")?.as_str()?.strip_prefix("#/$defs/")?;
        let def = self.defs.get(name)?;
        if def.get("x-teistro-kind").is_some() {
            return None;
        }
        fixed_strings(def).or_else(|| {
            def.get("oneOf")?
                .as_array()?
                .iter()
                .map(|option| Some(option.get("const")?.as_str()?.to_owned()))
                .collect()
        })
    }

    /// The catalogue kind a `$ref` names, if it names one.
    fn kind_of(&self, option: &Value) -> Option<String> {
        let name = option.get("$ref")?.as_str()?.strip_prefix("#/$defs/")?;
        Some(
            self.defs
                .get(name)?
                .get("x-teistro-kind")?
                .as_str()?
                .to_owned(),
        )
    }

    /// An object's fields in name order, leaving out the tag. Sorted here
    /// and not taken as the map keeps them, because that order is a
    /// feature of the JSON library (`preserve_order`) and a generated file
    /// must not change with a build's features.
    fn fields_of(
        &mut self,
        at: &str,
        object: &Value,
        tag: Option<&str>,
    ) -> Result<Vec<RecordField>, String> {
        read_whole(at, object)?;
        // serde's flattened enum: the variants beside the fields, which a
        // record of fields would read as less than it is.
        if let Some(keyword) = ["oneOf", "anyOf"]
            .into_iter()
            .find(|k| object.get(*k).is_some())
        {
            return Err(format!(
                "{at}: fields with a `{keyword}` beside them (a flattened enum), which the reader does not model"
            ));
        }
        let mut properties = object
            .get("properties")
            .and_then(Value::as_object)
            .ok_or_else(|| format!("{at}: an object with no properties"))?
            .clone();
        let mut required: BTreeSet<String> = names_of(object.get("required"));
        // A variant that wraps a record (serde's internally tagged newtype,
        // `Custom(Table)`) refers to it beside its tag, and serde writes
        // that record's fields beside the tag: they are the variant's.
        if let Some(reference) = object.get("$ref").and_then(Value::as_str) {
            let wrapped = self.wrapped(at, reference)?;
            for (name, schema) in wrapped
                .get("properties")
                .and_then(Value::as_object)
                .into_iter()
                .flatten()
            {
                properties
                    .entry(name.clone())
                    .or_insert_with(|| schema.clone());
            }
            required.extend(names_of(wrapped.get("required")));
        }
        let record = at.split('.').next().unwrap_or(at);
        let mut named: Vec<(&String, &Value)> = properties.iter().collect();
        named.sort_by(|a, b| a.0.cmp(b.0));
        named
            .into_iter()
            .filter(|(name, _)| Some(name.as_str()) != tag)
            .map(|(name, schema)| {
                let path = format!("{at}.{name}");
                let inline = format!("{record}{}", part(name));
                let (ty, nullable) = self.type_of(&path, schema, &inline)?;
                Ok(RecordField {
                    name: name.clone(),
                    doc: doc_of(schema),
                    ty,
                    nullable: nullable || !required.contains(name),
                    null: nullable,
                    default: schema
                        .get("default")
                        .filter(|value| !value.is_null())
                        .cloned(),
                })
            })
            .collect()
    }

    /// A field's type, and whether it may be null; `inline` is the name an
    /// inline type there takes.
    fn type_of(
        &mut self,
        at: &str,
        schema: &Value,
        inline: &str,
    ) -> Result<(RecordType, bool), String> {
        if self.is_raw(at) {
            return Ok((RecordType::Json, false));
        }
        read_whole(at, schema)?;
        if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
            return Ok((self.referred(at, reference)?, false));
        }
        if let Some(options) = schema.get("anyOf").and_then(Value::as_array) {
            let (nulls, others): (Vec<&Value>, Vec<&Value>) = options
                .iter()
                .partition(|option| option.get("type").and_then(Value::as_str) == Some("null"));
            let nullable = match nulls.len() {
                0 => false,
                1 => true,
                _ => return Err(format!("{at}: an anyOf with two nulls")),
            };
            if let [one] = others.as_slice() {
                return Ok((self.type_of(at, one, inline)?.0, nullable));
            }
            if let Some(catalogue) = self.keyed(&others) {
                return Ok((RecordType::Keyed { catalogue }, nullable));
            }
            let mut union = schema.clone();
            if let Some(options) = union.get_mut("anyOf") {
                *options = Value::Array(others.into_iter().cloned().collect());
            }
            return Ok((self.named_inline(at, &union, inline)?, nullable));
        }
        if schema.get("oneOf").is_some() || schema.get("enum").is_some() {
            return Ok((self.named_inline(at, schema, inline)?, false));
        }
        let (name, nullable) = match schema.get("type") {
            Some(Value::String(name)) => (name.as_str(), false),
            Some(Value::Array(names)) => match names.as_slice() {
                [Value::String(name), Value::String(null)] if null == "null" => {
                    (name.as_str(), true)
                }
                _ => return Err(format!("{at}: a type list that is not one type or null")),
            },
            _ => {
                return Err(format!(
                    "{at}: no type; list it as raw if it is any JSON value"
                ));
            }
        };
        let ty = match name {
            "boolean" => RecordType::Bool,
            "integer" => RecordType::Integer {
                width: width_of(schema),
            },
            "number" => RecordType::Number,
            "string" => RecordType::Text,
            "array" => self.array_of(at, schema, inline)?,
            "object" if schema.get("properties").is_some() => {
                self.named_inline(at, schema, inline)?
            }
            "object" => match schema.get("additionalProperties") {
                Some(values @ Value::Object(_)) => RecordType::Map {
                    of: Box::new(self.type_of(&format!("{at}{{}}"), values, inline)?.0),
                },
                _ => {
                    return Err(format!(
                        "{at}: an object open to any field; list it as raw if it is one"
                    ));
                }
            },
            other => return Err(format!("{at}: type `{other}`, which no record field is")),
        };
        Ok((ty, nullable))
    }

    /// An inline type, named `inline` and read as a definition of its own.
    fn named_inline(
        &mut self,
        at: &str,
        schema: &Value,
        inline: &str,
    ) -> Result<RecordType, String> {
        if self.defs.contains_key(inline) {
            return Err(format!(
                "{at}: an inline type here would be named {inline}, which a definition is; \
                 give one of them another name"
            ));
        }
        match self.inline.get(inline) {
            Some(known) if known != schema => {
                return Err(format!(
                    "{at}: two inline types would be named {inline}; give one a definition"
                ));
            }
            Some(_) => {}
            None => {
                self.inline.insert(inline.to_owned(), schema.clone());
            }
        }
        self.reached.push(inline.to_owned());
        Ok(RecordType::Record {
            name: inline.to_owned(),
        })
    }

    /// A list of one type, or a pair written as a two-item array.
    fn array_of(&mut self, at: &str, schema: &Value, inline: &str) -> Result<RecordType, String> {
        if let Some(items) = schema.get("prefixItems").and_then(Value::as_array) {
            return match items.as_slice() {
                [first, second] => Ok(RecordType::Pair {
                    first: Box::new(self.type_of(&format!("{at}[0]"), first, inline)?.0),
                    second: Box::new(self.type_of(&format!("{at}[1]"), second, inline)?.0),
                }),
                _ => Err(format!("{at}: a tuple that is not a pair")),
            };
        }
        let items = schema
            .get("items")
            .ok_or_else(|| format!("{at}: an array with no items"))?;
        let bound = |key: &str| schema.get(key).and_then(Value::as_u64);
        let len = match (bound("minItems"), bound("maxItems")) {
            (Some(min), Some(max)) if min == max => {
                Some(u32::try_from(min).map_err(|_| format!("{at}: an array of {min} items"))?)
            }
            _ => None,
        };
        Ok(RecordType::List {
            of: Box::new(self.type_of(&format!("{at}[]"), items, inline)?.0),
            len,
        })
    }

    /// The catalogue a union of a catalogue member and a free key reads: a
    /// member, or a key a context registered.
    fn keyed(&self, options: &[&Value]) -> Option<String> {
        let [member, key] = options else {
            return None;
        };
        let name = member.get("$ref")?.as_str()?.strip_prefix("#/$defs/")?;
        let catalogue = self.defs.get(name)?.get("x-teistro-kind")?.as_str()?;
        (key.get("type").and_then(Value::as_str) == Some("string") && key.get("pattern").is_some())
            .then(|| catalogue.to_owned())
    }

    /// The type a `$ref` names: a record, a catalogue member, a map, a
    /// raw part, or the scalar a constrained scalar definition is.
    /// The record a reference beside an object's own fields names,
    /// through any alias; refused unless it is a record with fields, since
    /// a variant would otherwise lose what serde writes.
    fn wrapped(&self, at: &str, reference: &str) -> Result<&'s Value, String> {
        let name = reference
            .strip_prefix("#/$defs/")
            .ok_or_else(|| format!("{at}: a reference outside $defs: {reference}"))?;
        let def = self
            .defs
            .get(name)
            .ok_or_else(|| format!("{at}: a reference to {name}, which is not defined"))?;
        if let Some(alias) = def.get("$ref").and_then(Value::as_str) {
            return self.wrapped(at, alias);
        }
        if def.get("properties").and_then(Value::as_object).is_none() {
            return Err(format!(
                "{at}: wraps {name}, which has no fields to stand beside the tag"
            ));
        }
        read_whole(at, def)?;
        Ok(def)
    }

    fn referred(&mut self, at: &str, reference: &str) -> Result<RecordType, String> {
        let name = reference
            .strip_prefix("#/$defs/")
            .ok_or_else(|| format!("{at}: a reference outside $defs: {reference}"))?;
        if self.is_raw(name) {
            return Ok(RecordType::Json);
        }
        let def = self
            .defs
            .get(name)
            .ok_or_else(|| format!("{at}: a reference to {name}, which is not defined"))?;
        if let Some(alias) = def.get("$ref").and_then(Value::as_str) {
            return self.referred(at, alias);
        }
        if let Some(kind) = def.get("x-teistro-kind").and_then(Value::as_str) {
            return Ok(RecordType::Member {
                catalogue: kind.to_owned(),
            });
        }
        let record = def.get("enum").is_some()
            || def.get("oneOf").is_some()
            || def.get("anyOf").is_some()
            || def.get("properties").is_some();
        match def.get("type").and_then(Value::as_str) {
            Some("string") if !record => Ok(RecordType::Text),
            Some("integer") => Ok(RecordType::Integer {
                width: width_of(def),
            }),
            Some("number") => Ok(RecordType::Number),
            Some("boolean") => Ok(RecordType::Bool),
            Some("object" | "array") if !record => Ok(self.type_of(at, def, name)?.0),
            _ => {
                self.reached.push(name.to_owned());
                Ok(RecordType::Record {
                    name: name.to_owned(),
                })
            }
        }
    }
}

/// A field's or a key's name as part of a type's name (`orbDeg` is `OrbDeg`,
/// `WITHIN` is `Within`, `house-weighted` is `HouseWeighted`).
fn part(name: &str) -> String {
    let name = name.replace('-', "_");
    if name.chars().any(|c| c.is_ascii_lowercase()) {
        pascal(&name)
    } else {
        pascal(&name.to_ascii_lowercase())
    }
}

/// The strings an option fixes: a constant, or a string `enum`.
fn fixed_strings(option: &Value) -> Option<Vec<String>> {
    if let Some(key) = option.get("const").and_then(Value::as_str) {
        return Some(vec![key.to_owned()]);
    }
    option
        .get("enum")?
        .as_array()?
        .iter()
        .map(|key| key.as_str().map(str::to_owned))
        .collect()
}

/// The key and the payload of an external variant: a bare string constant,
/// or an object of exactly one required field and nothing else.
fn external_key(option: &Value) -> Option<(String, Option<&Value>)> {
    if let Some(key) = option.get("const").and_then(Value::as_str) {
        return Some((key.to_owned(), None));
    }
    let properties = option.get("properties")?.as_object()?;
    let [(key, payload)] = properties.iter().collect::<Vec<_>>()[..] else {
        return None;
    };
    let closed = option.get("additionalProperties") == Some(&Value::Bool(false));
    let required = option
        .get("required")
        .and_then(Value::as_array)
        .is_some_and(|names| names.as_slice() == [Value::String(key.clone())]);
    (closed && required && payload.get("const").is_none()).then(|| (key.clone(), Some(payload)))
}

/// A whole number's width from its `format`.
fn width_of(schema: &Value) -> Option<Width> {
    let format = schema.get("format")?.as_str()?;
    let (signed, bits) = match format {
        "int8" => (true, 8),
        "int16" => (true, 16),
        "int32" => (true, 32),
        "int64" | "int" => (true, 64),
        "uint8" => (false, 8),
        "uint16" => (false, 16),
        "uint32" => (false, 32),
        "uint64" | "uint" => (false, 64),
        _ => return None,
    };
    Some(Width { signed, bits })
}

/// The one property every option of a union fixes to a constant.
fn tag_of(at: &str, options: &[Value]) -> Result<String, String> {
    let first = options
        .first()
        .and_then(|option| option.get("properties"))
        .and_then(Value::as_object)
        .ok_or_else(|| format!("{at}: a union whose first option is not an object"))?;
    first
        .iter()
        .filter(|(_, schema)| schema.get("const").is_some())
        .map(|(name, _)| name)
        .find(|name| {
            options.iter().all(|option| {
                option
                    .pointer(&format!("/properties/{name}/const"))
                    .is_some()
            })
        })
        .cloned()
        .ok_or_else(|| format!("{at}: a union with no field that tags every option"))
}

/// The keywords that would change what a value may be and that the reader
/// does not model: a schema carrying one would be read as less than it
/// says, as a `$ref` beside a variant's tag once was (every binding's
/// `ThresholdsCustom` lost its table), so it is refused instead.
const UNREAD: [&str; 8] = [
    "allOf",
    "not",
    "if",
    "patternProperties",
    "dependentRequired",
    "dependentSchemas",
    "unevaluatedProperties",
    "contains",
];

/// A schema with every bar on keys it does not name taken off, however
/// deep: what schemars leaves of a record it flattens, so the record can be
/// matched to its own definition.
fn opened(schema: &Value) -> Value {
    match schema {
        Value::Object(object) => Value::Object(
            object
                .iter()
                .filter(|(key, value)| {
                    !(matches!(
                        key.as_str(),
                        "additionalProperties" | "unevaluatedProperties"
                    ) && **value == Value::Bool(false))
                })
                .map(|(key, value)| (key.clone(), opened(value)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(opened).collect()),
        other => other.clone(),
    }
}

/// Refuses a schema with a keyword [`UNREAD`] names.
fn read_whole(at: &str, schema: &Value) -> Result<(), String> {
    match UNREAD
        .iter()
        .find(|keyword| schema.get(**keyword).is_some())
    {
        Some(keyword) => Err(format!(
            "{at}: `{keyword}`, which the reader does not model; describe it without, or teach the reader"
        )),
        None => Ok(()),
    }
}

/// The names a schema's `required` lists.
fn names_of(required: Option<&Value>) -> BTreeSet<String> {
    required
        .and_then(Value::as_array)
        .map(|names| {
            names
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

/// A schema's description, first paragraph only, as one line.
fn doc_of(schema: &Value) -> String {
    schema
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .split("\n\n")
        .next()
        .unwrap_or_default()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        clippy::panic,
        reason = "tests fail by panicking"
    )]

    use serde_json::json;

    use super::*;

    fn schema() -> Value {
        json!({"$defs": {
            "Root": {"type": "object", "description": "The root.\n\nMore.", "properties": {
                "hash": {"$ref": "#/$defs/Hash", "description": "A hash."},
                "pairs": {"type": "array", "items": {"type": "array", "minItems": 2, "maxItems": 2,
                    "prefixItems": [{"type": "string"}, {"$ref": "#/$defs/Version"}]}},
                "how": {"anyOf": [{"$ref": "#/$defs/How"}, {"type": "null"}]},
                "sure": {"$ref": "#/$defs/Sure"},
                "count": {"type": "integer"}
            }, "required": ["hash", "pairs", "how", "sure"]},
            "Hash": {"type": "string", "pattern": "^[0-9a-f]{64}$"},
            "Version": {"type": "object", "properties": {"major": {"type": "integer"}}, "required": ["major"]},
            "How": {"oneOf": [
                {"type": "object", "description": "Defined.", "properties": {"kind": {"const": "DEFINED"}}, "required": ["kind"]},
                {"type": "object", "properties": {"kind": {"const": "TABULAR"}, "edition": {"type": "string"}}, "required": ["kind", "edition"]}
            ]},
            "Sure": {"oneOf": [{"const": "VERIFIED", "type": "string"}, {"const": "UNVERIFIED", "type": "string"}]}
        }})
    }

    #[test]
    fn every_shape_of_the_subset_is_read() {
        let records = from_schema(&schema(), &["Root"]).unwrap();
        let names: Vec<&str> = records.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(
            names,
            ["Root", "How", "Version", "Sure"],
            "roots first, Hash inlined"
        );
        assert_eq!(records[0].doc, "The root.");
        let RecordShape::Object { fields } = &records[0].shape else {
            panic!("an object")
        };
        let order: Vec<&str> = fields.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(
            order,
            ["count", "hash", "how", "pairs", "sure"],
            "name order"
        );
        assert!(fields[0].nullable, "a field not required may be absent");
        assert_eq!(fields[1].ty, RecordType::Text);
        assert_eq!(fields[0].ty, RecordType::Integer { width: None });
        assert!(fields[2].nullable && !fields[4].nullable);
        assert_eq!(
            fields[3].ty,
            RecordType::List {
                of: Box::new(RecordType::Pair {
                    first: Box::new(RecordType::Text),
                    second: Box::new(RecordType::Record {
                        name: "Version".into()
                    }),
                }),
                len: None,
            }
        );
        let RecordShape::Tagged { tag, variants } = &records[1].shape else {
            panic!("a tagged union")
        };
        assert_eq!(tag, "kind");
        assert!(variants[0].fields.is_empty(), "{:?}", variants[0].fields);
        assert_eq!(variants[1].fields[0].name, "edition");
        assert!(matches!(&records[3].shape, RecordShape::Keys { values } if values.len() == 2));
    }

    /// The request side's constructs: serde's external tagging, a name
    /// or a value, catalogue members and registered keys, maps, widths,
    /// aliases, and the types written inline, named after where they are.
    fn requests() -> Value {
        json!({"$defs": {
            "Ask": {"type": "object", "properties": {
                "partile": {"$ref": "#/$defs/Partile"},
                "place": {"$ref": "#/$defs/Place"},
                "graha": {"$ref": "#/$defs/Graha"},
                "point": {"$ref": "#/$defs/Point"},
                "systems": {"type": "array", "items": {"anyOf": [
                    {"$ref": "#/$defs/Graha"},
                    {"type": "string", "pattern": "^[A-Z]+$"}]}},
                "weights": {"type": "object", "additionalProperties": {"type": "integer", "format": "uint16"}},
                "year": {"type": "integer", "format": "uint8"},
                "matters": {"anyOf": [
                    {"anyOf": [{"const": "all"}, {"type": "array", "items": {"type": "integer"}}]},
                    {"type": "null"}]},
                "rules": true,
                "how": {"type": "string", "enum": ["FAST", "SLOW"]}
            }, "required": ["partile"]},
            "Partile": {"oneOf": [
                {"type": "string", "const": "SAME_DEGREE", "description": "Exact."},
                {"type": "object", "properties": {"WITHIN": {"type": "object",
                    "properties": {"orbDeg": {"type": "number"}}, "required": ["orbDeg"]}},
                 "required": ["WITHIN"], "additionalProperties": false},
                {"type": "object", "properties": {"DAYS": {"type": "number"}},
                 "required": ["DAYS"], "additionalProperties": false}
            ]},
            "Place": {"anyOf": [{"const": "birth"}, {"$ref": "#/$defs/Residence"}]},
            "Residence": {"type": "object", "properties": {"lat": {"type": "number"}}, "required": ["lat"]},
            "Graha": {"anyOf": [{"type": "string", "enum": ["SUN"]}], "x-teistro-kind": "graha"},
            "Point": {"$ref": "#/$defs/Residence"}
        }})
    }

    /// serde's internally tagged newtype variant, as schemars describes
    /// it: the wrapped record by reference, the tag beside it.
    fn wrapping(table: &serde_json::Value) -> serde_json::Value {
        json!({"$defs": {
            "Thresholds": {"oneOf": [
                {"type": "object", "properties": {"kind": {"type": "string", "const": "PTOLEMY"}},
                 "required": ["kind"]},
                {"$ref": "#/$defs/Table", "type": "object",
                 "properties": {"kind": {"type": "string", "const": "CUSTOM"}}, "required": ["kind"]}
            ]},
            "Table": table,
            "Pair": {"type": "object", "properties": {"direct": {"type": "number"}}, "required": ["direct"]}
        }})
    }

    #[test]
    fn a_variant_wrapping_a_record_has_its_fields_beside_the_tag() {
        let table = json!({"type": "object", "properties": {
            "venus": {"anyOf": [{"$ref": "#/$defs/Pair"}, {"type": "null"}]},
            "mars": {"$ref": "#/$defs/Pair"}
        }, "required": ["mars"]});
        let records = read(&wrapping(&table), &["Thresholds"], Reading { raw: &[] }).unwrap();
        let tagged = records.iter().find(|r| r.name == "Thresholds").unwrap();
        let RecordShape::Tagged { tag, variants } = &tagged.shape else {
            return assert_eq!(tagged.shape, RecordShape::Keys { values: vec![] });
        };
        assert_eq!(tag, "kind");
        let custom = variants.iter().find(|v| v.key == "CUSTOM").unwrap();
        let fields: Vec<(&str, bool)> = custom
            .fields
            .iter()
            .map(|f| (f.name.as_str(), f.nullable))
            .collect();
        assert_eq!(fields, vec![("mars", false), ("venus", true)]);
        // A wrapped type with no fields would lose what serde writes.
        let refused = read(
            &wrapping(&json!({"type": "string"})),
            &["Thresholds"],
            Reading { raw: &[] },
        );
        assert!(refused.is_err_and(|e| e.contains("no fields to stand beside the tag")));
    }

    #[test]
    fn a_keyword_the_reader_does_not_model_is_refused() {
        let table = json!({"type": "object", "properties": {"mars": {"$ref": "#/$defs/Pair"}},
            "allOf": [{"required": ["mars"]}]});
        let refused = read(&wrapping(&table), &["Thresholds"], Reading { raw: &[] });
        assert!(refused.is_err_and(|e| e.contains("`allOf`, which the reader does not model")));
    }

    #[test]
    fn several_values_are_told_apart_by_their_keys_or_refused() {
        let union = |sait: Value, rule: Value| {
            json!({"$defs": {
                "Item": {"anyOf": [{"type": "string", "enum": ["NEPAL"]}, sait, rule]},
                "Sait": {"type": "object", "properties": {"windows": {"type": "integer"}}},
                "Rule": {"type": "object", "properties": {"key": {"type": "string"}}}
            }})
        };
        let sait = json!({"$ref": "#/$defs/Sait", "x-teistro-when": ["windows"]});
        let rule = json!({"$ref": "#/$defs/Rule"});
        let records = from_schema(&union(sait.clone(), rule.clone()), &["Item"]).unwrap();
        let item = records.iter().find(|r| r.name == "Item").unwrap();
        let RecordShape::OrNamed { values, .. } = &item.shape else {
            panic!("a name or a value")
        };
        let windows = json!({"windows": 1});
        assert_eq!(picked(values, &windows), values.first());
        assert_eq!(picked(values, &json!({"key": "X"})), values.last());
        let refused = |sait: &Value, rule: &Value| {
            from_schema(&union(sait.clone(), rule.clone()), &["Item"]).unwrap_err()
        };
        assert!(refused(&rule, &rule).contains("value 0 of several says no keys"));
        assert!(refused(&sait, &sait).contains("the last or only value names keys"));
        assert!(
            refused(&sait, &json!({"type": "string"}))
                .contains("one of several values is not a record")
        );
    }

    #[test]
    fn every_request_construct_is_read() {
        let records = read(
            &requests(),
            &["Ask"],
            Reading {
                raw: &["Ask.rules"],
            },
        )
        .unwrap();
        let named = |name: &str| records.iter().find(|r| r.name == name).unwrap();
        let RecordShape::Object { fields } = &named("Ask").shape else {
            panic!("an object")
        };
        let field = |name: &str| fields.iter().find(|f| f.name == name).unwrap();
        assert_eq!(
            field("graha").ty,
            RecordType::Member {
                catalogue: "graha".into()
            }
        );
        assert_eq!(
            field("systems").ty,
            RecordType::List {
                of: Box::new(RecordType::Keyed {
                    catalogue: "graha".into()
                }),
                len: None,
            }
        );
        assert_eq!(
            field("weights").ty,
            RecordType::Map {
                of: Box::new(RecordType::Integer {
                    width: Some(Width {
                        signed: false,
                        bits: 16
                    })
                })
            }
        );
        assert_eq!(
            field("year").ty,
            RecordType::Integer {
                width: Some(Width {
                    signed: false,
                    bits: 8
                })
            }
        );
        assert_eq!(field("rules").ty, RecordType::Json);
        assert_eq!(
            field("point").ty,
            RecordType::Record {
                name: "Residence".into()
            },
            "an alias"
        );
        assert_eq!(
            field("how").ty,
            RecordType::Record {
                name: "AskHow".into()
            }
        );
        assert!(
            matches!(&named("AskHow").shape, RecordShape::Keys { values } if values.len() == 2)
        );
    }

    #[test]
    fn every_request_union_is_read() {
        let records = read(
            &requests(),
            &["Ask"],
            Reading {
                raw: &["Ask.rules"],
            },
        )
        .unwrap();
        let named = |name: &str| records.iter().find(|r| r.name == name).unwrap();
        let RecordShape::Object { fields } = &named("Ask").shape else {
            panic!("an object")
        };
        let field = |name: &str| fields.iter().find(|f| f.name == name).unwrap();
        assert_eq!(
            field("matters").ty,
            RecordType::Record {
                name: "AskMatters".into()
            }
        );
        assert!(field("matters").nullable);
        let RecordShape::OrNamed { names, values, .. } = &named("AskMatters").shape else {
            panic!("a name or a value")
        };
        assert_eq!(names, &["all"]);
        assert!(
            matches!(values.as_slice(), [NamedValue { ty: RecordType::List { .. }, when }] if when.is_empty())
        );
        let RecordShape::OrNamed { names, values, .. } = &named("Place").shape else {
            panic!("a name or a value")
        };
        assert_eq!(
            (names.as_slice(), &values[0].ty),
            (
                &["birth".to_owned()][..],
                &RecordType::Record {
                    name: "Residence".into()
                }
            )
        );
        let RecordShape::External { variants } = &named("Partile").shape else {
            panic!("externally tagged")
        };
        let keys: Vec<&str> = variants.iter().map(|v| v.key.as_str()).collect();
        assert_eq!(keys, ["SAME_DEGREE", "WITHIN", "DAYS"]);
        assert_eq!(variants[0].payload, None);
        assert_eq!(
            variants[1].payload,
            Some(RecordType::Record {
                name: "PartileWithin".into()
            })
        );
        assert_eq!(variants[2].payload, Some(RecordType::Number));
        assert!(
            matches!(&named("PartileWithin").shape, RecordShape::Object { fields } if fields[0].name == "orbDeg")
        );
    }

    #[test]
    fn a_raw_part_is_listed_both_ways() {
        assert_eq!(
            read(&requests(), &["Ask"], Reading::default()).unwrap_err(),
            "Ask.rules: no type; list it as raw if it is any JSON value"
        );
        assert_eq!(
            read(
                &requests(),
                &["Ask"],
                Reading {
                    raw: &["Ask.rules", "Ask.nothing"]
                }
            )
            .unwrap_err(),
            "listed as raw and reached by nothing: [\"Ask.nothing\"]; take each off the list"
        );
    }

    #[test]
    fn an_inline_type_never_takes_a_definitions_name() {
        let mut clash = requests();
        clash["$defs"]["AskHow"] = json!({"type": "object", "properties": {}});
        assert!(
            read(
                &clash,
                &["Ask"],
                Reading {
                    raw: &["Ask.rules"]
                }
            )
            .unwrap_err()
            .starts_with(
                "Ask.how: an inline type here would be named AskHow, which a definition is"
            )
        );
    }

    #[test]
    fn a_construct_outside_the_subset_is_refused_by_its_path() {
        let mut odd = schema();
        odd["$defs"]["Root"]["properties"]["count"] = json!({"type": "object"});
        let refused = from_schema(&odd, &["Root"]).unwrap_err();
        assert_eq!(
            refused,
            "Root.count: an object open to any field; list it as raw if it is one"
        );
        assert!(
            from_schema(&schema(), &["Nothing"])
                .unwrap_err()
                .contains("no such definition")
        );
        let mut untagged = schema();
        untagged["$defs"]["How"]["oneOf"][1]["properties"]["kind"] = json!({"type": "string"});
        assert!(
            from_schema(&untagged, &["Root"])
                .unwrap_err()
                .contains("no field that tags every option")
        );
    }

    #[test]
    fn an_array_of_a_fixed_length_keeps_its_length() {
        let mut fixed = schema();
        fixed["$defs"]["Root"]["properties"]["count"] = json!({"type": "array",
            "items": {"type": "number"}, "minItems": 7, "maxItems": 7});
        fixed["$defs"]["Root"]["properties"]["hash"] = json!({"type": "array",
            "items": {"type": "number"}, "minItems": 1});
        let records = from_schema(&fixed, &["Root"]).unwrap();
        let RecordShape::Object { fields } = &records[0].shape else {
            panic!("an object")
        };
        let len = |name: &str| match &fields.iter().find(|f| f.name == name).unwrap().ty {
            RecordType::List { len, .. } => *len,
            _ => panic!("a list"),
        };
        assert_eq!((len("count"), len("hash")), (Some(7), None));
        // Written only where it is fixed, so a description without one
        // reads as it did.
        let open = RecordType::List {
            of: Box::new(RecordType::Number),
            len: None,
        };
        assert_eq!(
            serde_json::to_string(&open).unwrap(),
            r#"{"kind":"LIST","of":{"kind":"NUMBER"}}"#
        );
    }

    /// A cancellation as schemars writes it: a label, and a condition's
    /// variants inline, opened to the label, beside it.
    fn cancellation_schema(label: &str, variants: &serde_json::Value) -> serde_json::Value {
        json!({"$defs": {
            "Cancellation": {"type": "object", "description": "A cancellation.",
                "properties": {label: {"type": ["string", "null"]}},
                "oneOf": variants, "unevaluatedProperties": false},
            "Condition": {"description": "A condition.", "oneOf": [
                {"type": "object", "properties": {"type": {"const": "combust"},
                    "planet": {"type": "string"}}, "required": ["type", "planet"],
                    "additionalProperties": false},
                {"type": "object", "properties": {"type": {"const": "day"}},
                    "required": ["type"], "additionalProperties": false}
            ]}
        }})
    }

    fn opened_conditions() -> serde_json::Value {
        json!([
            {"type": "object", "properties": {"type": {"const": "combust"},
                "planet": {"type": "string"}}, "required": ["type", "planet"]},
            {"type": "object", "properties": {"type": {"const": "day"}}, "required": ["type"]}
        ])
    }

    #[test]
    fn a_record_laid_flat_is_the_definition_whose_variants_it_repeats() {
        let records = from_schema(
            &cancellation_schema("label", &opened_conditions()),
            &["Cancellation"],
        )
        .unwrap();
        let names: Vec<&str> = records.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, ["Cancellation", "Condition"]);
        let RecordShape::Beside { fields, laid } = &records[0].shape else {
            panic!("fields beside a record: {:?}", records[0].shape)
        };
        assert_eq!(fields.len(), 1);
        assert!(fields[0].name == "label" && fields[0].nullable && fields[0].null);
        assert_eq!(laid.name, "condition");
        assert_eq!(laid.doc, "A condition.");
        assert!(!laid.nullable);
        assert_eq!(
            laid.ty,
            RecordType::Record {
                name: "Condition".into()
            }
        );
        // Variants no definition has are not guessed at.
        let unmatched = json!([{"type": "object", "properties": {"type": {"const": "night"}}}]);
        let refused =
            from_schema(&cancellation_schema("label", &unmatched), &["Cancellation"]).unwrap_err();
        assert!(refused.contains("not one definition's own"), "{refused}");
        // A field the laid record writes too would be written twice.
        let refused = from_schema(
            &cancellation_schema("planet", &opened_conditions()),
            &["Cancellation"],
        )
        .unwrap_err();
        assert!(
            refused.contains("Cancellation.planet: a field the flattened Condition writes too"),
            "{refused}"
        );
        // A laid record no field tags could not have its variants written.
        let mut untagged = cancellation_schema(
            "label",
            &json!([{"type": "object", "properties": {"AT": {"type": "number"}}}]),
        );
        untagged["$defs"]["Condition"]["oneOf"] = json!([{"type": "object", "properties": {"AT": {"type": "number"}},
            "additionalProperties": false}]);
        let refused = from_schema(&untagged, &["Cancellation"]).unwrap_err();
        assert!(
            refused.contains("Condition: a union with no field that tags every option"),
            "{refused}"
        );
    }

    #[test]
    fn a_schema_is_opened_however_deep_and_keeps_what_is_not_a_bar() {
        let barred = json!({"additionalProperties": false, "items": [{"unevaluatedProperties": false,
            "additionalProperties": {"type": "integer"}}], "x": 1});
        assert_eq!(
            opened(&barred),
            json!({"items": [{"additionalProperties": {"type": "integer"}}], "x": 1})
        );
    }
}
