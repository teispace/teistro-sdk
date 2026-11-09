//! `crates/sdk/src/records/answers.json`: every record's answer schema,
//! kept to the keywords that say where a field is, which the SDK checks a
//! caller's field mask against without linking schemars
//! (`teistro::records::Record::fields`).
//!
//! One generator writes every answer, so the definitions they share are
//! written once. Descriptions, examples, bounds and formats go: a mask
//! asks where a field is, never what it means or what it may hold.

use std::path::Path;

use schemars::generate::SchemaSettings;
use serde_json::{Map, Value, json};

use crate::generated::{Output, check, write};

/// The file the SDK embeds.
const FILE: &str = "crates/sdk/src/records/answers.json";

/// The keywords that place a field: what `Walk` in the SDK's
/// `records/detail.rs` reads, and the stating ones it tells an open value
/// from a closed one by.
const KEPT: [&str; 13] = [
    "type",
    "properties",
    "items",
    "additionalProperties",
    "anyOf",
    "oneOf",
    "allOf",
    "not",
    "$ref",
    "const",
    "enum",
    "required",
    "prefixItems",
];

/// A schema kept to [`KEPT`], all the way down.
fn shape(schema: Value) -> Value {
    let Value::Object(object) = schema else {
        return schema;
    };
    let mut kept = Map::new();
    for (key, value) in object {
        if !KEPT.contains(&key.as_str()) {
            continue;
        }
        let value = match (key.as_str(), value) {
            ("properties", Value::Object(properties)) => Value::Object(
                properties
                    .into_iter()
                    .map(|(name, inner)| (name, shape(inner)))
                    .collect(),
            ),
            ("items" | "additionalProperties" | "not", inner) => shape(inner),
            ("anyOf" | "oneOf" | "allOf" | "prefixItems", Value::Array(branches)) => {
                Value::Array(branches.into_iter().map(shape).collect())
            }
            (_, other) => other,
        };
        kept.insert(key, value);
    }
    Value::Object(kept)
}

/// The file's text.
fn table() -> String {
    let mut generator = SchemaSettings::draft2020_12().into_generator();
    let mut answers = Map::new();
    for record in teistro::records::records() {
        if let Some(schema) = record.answer_schema(&mut generator) {
            answers.insert(record.name.to_owned(), shape(schema.to_value()));
        }
    }
    let definitions: Map<String, Value> = generator
        .take_definitions(true)
        .into_iter()
        .map(|(name, schema)| (name, shape(schema)))
        .collect();
    let mut text = serde_json::to_string(&json!({
        "answers": answers,
        "definitions": definitions,
    }))
    .unwrap_or_default();
    text.push('\n');
    text
}

pub(crate) fn generate(root: &Path) -> i32 {
    write(root, &[Output::new(FILE, table())])
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    i32::from(check(root, &[Output::new(FILE, table())], "cargo xtask answers") != 0)
}
