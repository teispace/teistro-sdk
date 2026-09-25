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
//! The conversion reads a small, closed subset of JSON Schema — objects,
//! string-key unions, unions tagged by one constant field, lists, pairs and
//! the scalars — and **refuses** anything else by its path, so a field of
//! a new shape stops the generator rather than crossing as whatever the
//! nearest guess was.
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

use std::collections::{BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// A record that crosses as JSON, named as its schema names it.
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
}

/// One key of a [`RecordShape::Keys`] record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordKey {
    /// The key, as the wire writes it (`VERIFIED`).
    pub key: String,
    /// What it means.
    pub doc: String,
}

/// One variant of a [`RecordShape::Tagged`] record.
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
}

/// The type of a record's field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecordType {
    /// `true` or `false`.
    Bool,
    /// A whole number.
    Integer,
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
    },
    /// Two values of fixed types, written as a two-item array.
    Pair {
        /// The first's type.
        first: Box<RecordType>,
        /// The second's type.
        second: Box<RecordType>,
    },
}

/// The records `roots` name and every record they reach, from a JSON
/// Schema's `$defs`, roots first and the rest in the order they are
/// reached.
///
/// A definition that is a bare scalar with a constraint (a hash's pattern)
/// is not a record: a field referring to it takes the scalar's type.
///
/// # Errors
///
/// A root the schema does not define, a reference to a definition that is
/// not there, or any construct outside the subset, each named by its path
/// (`Provenance.module_versions`).
pub fn from_schema(schema: &Value, roots: &[&str]) -> Result<Vec<RecordDef>, String> {
    let defs = schema
        .get("$defs")
        .and_then(Value::as_object)
        .ok_or("the schema has no $defs")?;
    let mut queue: VecDeque<String> = roots.iter().map(|root| (*root).to_owned()).collect();
    let mut seen: BTreeSet<String> = queue.iter().cloned().collect();
    let mut records = Vec::new();
    while let Some(name) = queue.pop_front() {
        let def = defs
            .get(&name)
            .ok_or_else(|| format!("{name}: no such definition"))?;
        let mut reached = Vec::new();
        let shape = shape_of(&name, def, defs, &mut reached)?;
        records.push(RecordDef {
            doc: doc_of(def),
            name,
            shape,
        });
        for next in reached {
            if seen.insert(next.clone()) {
                queue.push_back(next);
            }
        }
    }
    Ok(records)
}

/// A definition's shape, noting the records its fields reach.
fn shape_of(
    at: &str,
    def: &Value,
    defs: &Map<String, Value>,
    reached: &mut Vec<String>,
) -> Result<RecordShape, String> {
    if def.get("type").and_then(Value::as_str) == Some("object") {
        return Ok(RecordShape::Object {
            fields: fields_of(at, def, defs, reached, None)?,
        });
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
                fields: fields_of(&format!("{at}.{key}"), option, defs, reached, Some(&tag))?,
                doc: doc_of(option),
                key,
            })
        })
        .collect::<Result<_, String>>()?;
    Ok(RecordShape::Tagged { tag, variants })
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

/// An object's fields in name order, leaving out the tag. Sorted here and
/// not taken as the map keeps them, because that order is a feature of the
/// JSON library (`preserve_order`) and a generated file must not change
/// with a build's features.
fn fields_of(
    at: &str,
    object: &Value,
    defs: &Map<String, Value>,
    reached: &mut Vec<String>,
    tag: Option<&str>,
) -> Result<Vec<RecordField>, String> {
    let properties = object
        .get("properties")
        .and_then(Value::as_object)
        .ok_or_else(|| format!("{at}: an object with no properties"))?;
    let required: BTreeSet<&str> = object
        .get("required")
        .and_then(Value::as_array)
        .map(|names| names.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let mut named: Vec<(&String, &Value)> = properties.iter().collect();
    named.sort_by(|a, b| a.0.cmp(b.0));
    named
        .into_iter()
        .filter(|(name, _)| Some(name.as_str()) != tag)
        .map(|(name, schema)| {
            let path = format!("{at}.{name}");
            let (ty, nullable) = type_of(&path, schema, defs, reached)?;
            Ok(RecordField {
                name: name.clone(),
                doc: doc_of(schema),
                ty,
                nullable: nullable || !required.contains(name.as_str()),
            })
        })
        .collect()
}

/// A field's type, and whether it may be null.
fn type_of(
    at: &str,
    schema: &Value,
    defs: &Map<String, Value>,
    reached: &mut Vec<String>,
) -> Result<(RecordType, bool), String> {
    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        return Ok((referred(at, reference, defs, reached)?, false));
    }
    if let Some(options) = schema.get("anyOf").and_then(Value::as_array) {
        let (nulls, others): (Vec<&Value>, Vec<&Value>) = options
            .iter()
            .partition(|option| option.get("type").and_then(Value::as_str) == Some("null"));
        return match (nulls.len(), others.as_slice()) {
            (1, [one]) => Ok((type_of(at, one, defs, reached)?.0, true)),
            _ => Err(format!("{at}: an anyOf that is not one type or null")),
        };
    }
    let (name, nullable) = match schema.get("type") {
        Some(Value::String(name)) => (name.as_str(), false),
        Some(Value::Array(names)) => match names.as_slice() {
            [Value::String(name), Value::String(null)] if null == "null" => (name.as_str(), true),
            _ => return Err(format!("{at}: a type list that is not one type or null")),
        },
        _ => return Err(format!("{at}: no type")),
    };
    let ty = match name {
        "boolean" => RecordType::Bool,
        "integer" => RecordType::Integer,
        "number" => RecordType::Number,
        "string" => RecordType::Text,
        "array" => array_of(at, schema, defs, reached)?,
        other => return Err(format!("{at}: type `{other}`, which no record field is")),
    };
    Ok((ty, nullable))
}

/// A list of one type, or a pair written as a two-item array.
fn array_of(
    at: &str,
    schema: &Value,
    defs: &Map<String, Value>,
    reached: &mut Vec<String>,
) -> Result<RecordType, String> {
    if let Some(items) = schema.get("prefixItems").and_then(Value::as_array) {
        return match items.as_slice() {
            [first, second] => Ok(RecordType::Pair {
                first: Box::new(type_of(&format!("{at}[0]"), first, defs, reached)?.0),
                second: Box::new(type_of(&format!("{at}[1]"), second, defs, reached)?.0),
            }),
            _ => Err(format!("{at}: a tuple that is not a pair")),
        };
    }
    let items = schema
        .get("items")
        .ok_or_else(|| format!("{at}: an array with no items"))?;
    Ok(RecordType::List {
        of: Box::new(type_of(&format!("{at}[]"), items, defs, reached)?.0),
    })
}

/// The type a `$ref` names: a record, or the scalar a constrained scalar
/// definition is.
fn referred(
    at: &str,
    reference: &str,
    defs: &Map<String, Value>,
    reached: &mut Vec<String>,
) -> Result<RecordType, String> {
    let name = reference
        .strip_prefix("#/$defs/")
        .ok_or_else(|| format!("{at}: a reference outside $defs: {reference}"))?;
    let def = defs
        .get(name)
        .ok_or_else(|| format!("{at}: a reference to {name}, which is not defined"))?;
    match def.get("type").and_then(Value::as_str) {
        Some("string") => Ok(RecordType::Text),
        Some("integer") => Ok(RecordType::Integer),
        Some("number") => Ok(RecordType::Number),
        Some("boolean") => Ok(RecordType::Bool),
        _ => {
            reached.push(name.to_owned());
            Ok(RecordType::Record {
                name: name.to_owned(),
            })
        }
    }
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
        assert!(fields[2].nullable && !fields[4].nullable);
        assert_eq!(
            fields[3].ty,
            RecordType::List {
                of: Box::new(RecordType::Pair {
                    first: Box::new(RecordType::Text),
                    second: Box::new(RecordType::Record {
                        name: "Version".into()
                    }),
                })
            }
        );
        let RecordShape::Tagged { tag, variants } = &records[1].shape else {
            panic!("a tagged union")
        };
        assert_eq!(tag, "kind");
        assert!(variants[0].fields.is_empty());
        assert_eq!(variants[1].fields[0].name, "edition");
        assert!(matches!(&records[3].shape, RecordShape::Keys { values } if values.len() == 2));
    }

    #[test]
    fn a_construct_outside_the_subset_is_refused_by_its_path() {
        let mut odd = schema();
        odd["$defs"]["Root"]["properties"]["count"] = json!({"type": "object"});
        let refused = from_schema(&odd, &["Root"]).unwrap_err();
        assert_eq!(
            refused,
            "Root.count: type `object`, which no record field is"
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
}
