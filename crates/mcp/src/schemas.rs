//! Each tool's schemas (`03-design/mcp-server.md` §7, P1): the record a
//! tool reads and the envelope it answers, as JSON Schema 2020-12 derived
//! from the types serde reads and writes, so never stricter than the
//! reader; and `schema.describe`, which answers them in full.
//!
//! A tool list is read into a model's context whole, and `chart.found`'s
//! record carries every chart record's schema inside it, several hundred
//! kilobytes with the answer's. So a list states its schemas in one of
//! two details ([`Detail`]): **lean**, the default, where each record a
//! request carries by name (`kp`, `muhurta`, …) is one line saying to ask
//! `schema.describe` for it and no answer schema is listed, and **full**,
//! everything inline, for a client that validates structured content.

use std::collections::BTreeSet;

use schemars::SchemaGenerator;
use schemars::generate::SchemaSettings;
use serde_json::{Map, Value, json};
use teistro::Error;
use teistro::records::Record;
use teistro_core::envelope::Provenance;
use teistro_core::settings::{DEFAULT_PROFILE, SHIPPED_PROFILES};

use crate::tools::annotations;

/// The tool answering a tool's schemas in full, or one part's.
pub(crate) const DESCRIBE: &str = "schema.describe";

/// How much of each record's schema a tool list states.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Detail {
    /// Each record a request carries by name stated by a line naming
    /// `schema.describe`, and no answer schema: a list a model reads.
    #[default]
    Lean,
    /// Every schema inline, the answers' included: a list a client
    /// validates against.
    Full,
}

impl Detail {
    /// The names [`Detail::from_name`] reads.
    pub const NAMES: [&'static str; 2] = ["lean", "full"];

    /// The detail called `name`.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Detail> {
        match name {
            "lean" => Some(Detail::Lean),
            "full" => Some(Detail::Full),
            _ => None,
        }
    }
}

/// `object` with `key` set to `value`.
pub(crate) fn with(mut object: Value, key: &str, value: Value) -> Value {
    if let Value::Object(fields) = &mut object {
        fields.insert(key.to_owned(), value);
    }
    object
}

/// A generator writing JSON Schema 2020-12, the revision's dialect, its
/// definitions under `$defs` at the schema's root.
pub(crate) fn generator() -> SchemaGenerator {
    SchemaSettings::draft2020_12().into_generator()
}

/// A schema whose references point into `generator`'s definitions, made
/// whole: those definitions laid under its own `$defs`.
fn rooted(schema: Value, generator: &mut SchemaGenerator) -> Value {
    let definitions = generator.take_definitions(true);
    if definitions.is_empty() {
        schema
    } else {
        with(schema, "$defs", Value::Object(definitions))
    }
}

/// A record tool's arguments, in full: `request` the record's own schema
/// where its reader's type carries one, and the context's three beside
/// it.
pub(crate) fn input(record: &Record) -> Value {
    let mut generator = generator();
    let described = format!(
        "The record `{}` reads, as the tool's description gives it.",
        record.boundary
    );
    let request = match record.request_schema(&mut generator) {
        Some(schema) => {
            let mut schema = schema.to_value();
            if let Value::Object(object) = &mut schema {
                object
                    .entry("description")
                    .or_insert_with(|| json!(described));
            }
            schema
        }
        None => json!({ "type": "object", "description": described }),
    };
    let schema = json!({
        "type": "object",
        "properties": {
            "request": request,
            "profile": {
                "type": "string",
                "enum": SHIPPED_PROFILES,
                "default": DEFAULT_PROFILE,
                "description": "The shipped profile the settings resolve from.",
            },
            "settings": {
                "type": "object",
                "description": "A patch over the profile, every group and knob optional; \
                                `settings.describe` answers its JSON Schema.",
            },
            "locale": {
                "type": "string",
                "description": "The locale a message renders in, as a BCP 47 tag.",
            },
        },
        "required": ["request"],
        "additionalProperties": false,
    });
    rooted(schema, &mut generator)
}

/// A record tool's structured content, the envelope: `value` the
/// answer's own schema, and `provenance` where the area seals one. None
/// where the answer's type carries no schema.
pub(crate) fn output(record: &Record) -> Option<Value> {
    let mut generator = generator();
    let value = record.answer_schema(&mut generator)?.to_value();
    let provenance = generator.subschema_for::<Provenance>().to_value();
    let schema = json!({
        "type": "object",
        "properties": { "value": value, "provenance": provenance },
        "required": ["value"],
        "additionalProperties": false,
    });
    Some(rooted(schema, &mut generator))
}

/// The request's own properties, where its schema is inline.
fn request_parts(input: &mut Value) -> Option<&mut Map<String, Value>> {
    input
        .get_mut("properties")?
        .get_mut("request")?
        .get_mut("properties")?
        .as_object_mut()
}

/// `input` with each of the record's parts a line naming
/// `schema.describe`, and the definitions only they reached dropped.
pub(crate) fn lean(mut input: Value, record: &Record) -> Value {
    if let Some(properties) = request_parts(&mut input) {
        for part in record.parts() {
            if properties.contains_key(*part) {
                let said = format!(
                    "The `{part}` record, read by its own reader: `{DESCRIBE}` with \
                     {{\"tool\": \"{}\", \"part\": \"{part}\"}} answers its schema.",
                    record.name
                );
                properties.insert(
                    (*part).to_owned(),
                    json!({ "type": "object", "description": said }),
                );
            }
        }
    }
    pruned(input)
}

/// One part of a record's request, whole: its schema and the definitions
/// it reaches.
fn part(record: &Record, name: &str) -> Option<Value> {
    let mut input = input(record);
    let definitions = input.get("$defs").cloned();
    let schema = request_parts(&mut input)?.remove(name)?;
    Some(pruned(match definitions {
        Some(definitions) => with(schema, "$defs", definitions),
        None => schema,
    }))
}

/// `schema` with only the definitions something outside `$defs` reaches,
/// directly or through another definition.
fn pruned(mut schema: Value) -> Value {
    let Some(Value::Object(definitions)) = schema.as_object_mut().and_then(|s| s.remove("$defs"))
    else {
        return schema;
    };
    let mut reached = BTreeSet::new();
    let mut pending = Vec::new();
    references(&schema, &mut pending);
    while let Some(name) = pending.pop() {
        if reached.insert(name.clone())
            && let Some(definition) = definitions.get(&name)
        {
            references(definition, &mut pending);
        }
    }
    let kept: Map<String, Value> = definitions
        .into_iter()
        .filter(|(name, _)| reached.contains(name))
        .collect();
    if kept.is_empty() {
        schema
    } else {
        with(schema, "$defs", Value::Object(kept))
    }
}

/// Every definition `value` refers to, by name.
fn references(value: &Value, into: &mut Vec<String>) {
    match value {
        Value::Object(object) => {
            for (key, inner) in object {
                match (key.as_str(), inner) {
                    ("$ref", Value::String(target)) => {
                        if let Some(name) = target.strip_prefix("#/$defs/") {
                            into.push(name.to_owned());
                        }
                    }
                    _ => references(inner, into),
                }
            }
        }
        Value::Array(items) => items.iter().for_each(|item| references(item, into)),
        _ => {}
    }
}

pub(crate) fn describe_tool() -> Value {
    let title = "A tool's schemas in full, or one part's";
    let tools: Vec<&str> = teistro::records::records()
        .iter()
        .map(|record| record.name)
        .collect();
    json!({
        "name": DESCRIBE,
        "title": title,
        "description": "A record tool's schemas in full: `input`, the arguments with the \
                        record inside them, and `output`, the `{value, provenance}` it \
                        answers. With `part`, one record a request carries by name \
                        (`chart.found`'s `kp`, `almanac.days`'s `muhurta`), which a tool \
                        list states by a line naming this tool.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "tool": { "type": "string", "enum": tools, "description": "The record tool." },
                "part": {
                    "type": "string",
                    "description": "One record the tool's request carries by name.",
                },
            },
            "required": ["tool"],
            "additionalProperties": false,
        },
        "annotations": annotations(title),
    })
}

/// `schema.describe`'s answer.
pub(crate) fn describe(arguments: &Map<String, Value>) -> Result<Value, Error> {
    if let Some(key) = arguments
        .keys()
        .find(|key| *key != "tool" && *key != "part")
    {
        return Err(Error::invalid_arg(format!("no argument `{key}`"))
            .with_field(format!("arguments.{key}"))
            .with_hint(format!("`{DESCRIBE}` reads `tool` and `part`")));
    }
    let name = match arguments.get("tool") {
        Some(Value::String(name)) => name.as_str(),
        _ => {
            return Err(
                Error::invalid_arg("name the record tool as a string").with_field("arguments.tool")
            );
        }
    };
    let record = teistro::records::record(name).ok_or_else(|| {
        Error::invalid_arg(format!("no record tool `{name}`"))
            .with_field("arguments.tool")
            .with_hint("`tools/list` names every one")
    })?;
    match arguments.get("part") {
        None => Ok(json!({
            "tool": name,
            "input": input(&record),
            "output": output(&record),
        })),
        Some(Value::String(wanted)) => {
            let schema = record
                .parts()
                .contains(&wanted.as_str())
                .then(|| part(&record, wanted))
                .flatten()
                .ok_or_else(|| {
                    Error::invalid_arg(format!("`{name}` carries no part `{wanted}`"))
                        .with_field("arguments.part")
                        .with_hint(if record.parts().is_empty() {
                            String::from("this tool's request carries no record by name")
                        } else {
                            format!("its parts are {}", record.parts().join(", "))
                        })
                })?;
            Ok(json!({ "tool": name, "part": wanted, "schema": schema }))
        }
        Some(_) => Err(Error::invalid_arg("`part` is a string").with_field("arguments.part")),
    }
}
