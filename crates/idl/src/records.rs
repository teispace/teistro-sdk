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
    /// A key, or an object of one field named by its key and holding the
    /// variant's value (`"SAME_DEGREE"` or `{"WITHIN": {"orbDeg": 1}}`),
    /// which is how serde writes an enum by default.
    External {
        /// The variants, in the order the type declares them.
        variants: Vec<ExternalVariant>,
    },
    /// One of a few fixed strings, or a value of one type (`"birth"` or a
    /// place; a criterion's name or the criterion spelt out).
    OrNamed {
        /// The strings, as the wire writes them.
        names: Vec<String>,
        /// The value's type.
        of: RecordType,
    },
}

/// One variant of a [`RecordShape::External`] record.
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Width {
    /// Whether it may be negative.
    pub signed: bool,
    /// How many bits it has.
    pub bits: u8,
}

/// How a schema is read.
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

impl Reader<'_> {
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

    /// A union of fixed strings and one other type.
    fn named_of(&mut self, at: &str, options: &[Value]) -> Result<RecordShape, String> {
        let mut names = Vec::new();
        let mut others = Vec::new();
        for option in options {
            match fixed_strings(option) {
                Some(fixed) => names.extend(fixed),
                None => others.push(option),
            }
        }
        match others.as_slice() {
            [other] if !names.is_empty() => Ok(RecordShape::OrNamed {
                names,
                of: self
                    .type_of(&format!("{at}.value"), other, &format!("{at}Value"))?
                    .0,
            }),
            _ => Err(format!(
                "{at}: an anyOf that is neither one type or null nor fixed strings or one \
                 type; list it as raw if it is a JSON value"
            )),
        }
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
        let properties = object
            .get("properties")
            .and_then(Value::as_object)
            .ok_or_else(|| format!("{at}: an object with no properties"))?;
        let required: BTreeSet<&str> = object
            .get("required")
            .and_then(Value::as_array)
            .map(|names| names.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default();
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
                    nullable: nullable || !required.contains(name.as_str()),
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
        Ok(RecordType::List {
            of: Box::new(self.type_of(&format!("{at}[]"), items, inline)?.0),
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
                })
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
                })
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
        let RecordShape::OrNamed { names, of } = &named("AskMatters").shape else {
            panic!("a name or a value")
        };
        assert_eq!(names, &["all"]);
        assert!(matches!(of, RecordType::List { .. }));
        let RecordShape::OrNamed { names, of } = &named("Place").shape else {
            panic!("a name or a value")
        };
        assert_eq!(
            (names.as_slice(), of),
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
}
