//! The schemas the rule language writes by hand, since its readers take
//! more shapes than a derive states: each under the name a tool's answer
//! refers to it by, saying what its reader takes.

#![cfg(feature = "schema")]
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they read"
)]

use schemars::SchemaGenerator;
use serde_json::{Value, json};
use teistro_rules::longevity::Reasons;
use teistro_rules::{
    Body, BodyRef, BodySubject, Cancellation, Condition, Karaka, SignRef, Subject,
};

#[test]
fn every_hand_written_schema_is_named_and_states_its_shapes() {
    let mut generator = SchemaGenerator::default();
    let named = [
        ("RuleBody", generator.subschema_for::<Body>()),
        ("Karaka", generator.subschema_for::<Karaka>()),
        ("BodySubject", generator.subschema_for::<BodySubject>()),
        ("BodyRef", generator.subschema_for::<BodyRef>()),
        ("SignRef", generator.subschema_for::<SignRef>()),
        ("Subject", generator.subschema_for::<Subject>()),
    ];
    let reasons = generator.subschema_for::<Reasons>();
    let definitions = generator.definitions().clone();
    for (name, reference) in &named {
        assert_eq!(
            reference.get("$ref"),
            Some(&json!(format!("#/$defs/{name}"))),
            "{name}"
        );
        let schema = &definitions[*name];
        let description = schema["description"].as_str().unwrap_or_default();
        assert!(!description.is_empty(), "{name}");
    }
    assert_eq!(definitions["RuleBody"]["type"], "string");
    // A maraka's reasons are a set: each once, each a reason.
    assert_eq!(reasons.get("$ref"), Some(&json!("#/$defs/MarakaReasons")));
    let set = &definitions["MarakaReasons"];
    assert_eq!(
        (&set["type"], &set["uniqueItems"]),
        (&json!("array"), &json!(true))
    );
    let item = set["items"]["$ref"]
        .as_str()
        .unwrap()
        .trim_start_matches("#/$defs/");
    assert!(
        definitions[item]["oneOf"].is_array() || definitions[item]["enum"].is_array(),
        "{item}"
    );
    let karakas: Vec<Value> = ["AK", "AmK", "BK", "MK", "PK", "GK", "DK", "PiK"]
        .into_iter()
        .map(Value::from)
        .collect();
    assert_eq!(definitions["Karaka"]["enum"], Value::Array(karakas));
    // A sign or body reference is a key, or a house's number, or an object
    // of named parts: never stricter than the reader, which judges the parts.
    for name in ["BodySubject", "BodyRef", "SignRef", "Subject"] {
        let shapes = definitions[name]["anyOf"].as_array().unwrap();
        let types: Vec<&str> = shapes.iter().filter_map(|s| s["type"].as_str()).collect();
        assert!(
            types.contains(&"string") && types.contains(&"object"),
            "{name}: {types:?}"
        );
    }
}

/// A cancellation is derived, not written by hand: its label beside every
/// condition's own parts, so a binding typed from it carries the condition.
#[test]
fn a_cancellation_is_a_condition_with_its_label_beside_it() {
    let mut generator = schemars::generate::SchemaSettings::draft2020_12().into_generator();
    let reference = generator.subschema_for::<Cancellation>();
    generator.subschema_for::<Condition>();
    assert_eq!(reference.get("$ref"), Some(&json!("#/$defs/Cancellation")));
    let definitions = generator.definitions().clone();
    let cancellation = &definitions["Cancellation"];
    assert_eq!(cancellation["type"], "object");
    assert_eq!(
        cancellation["properties"]["label"]["type"],
        json!(["string", "null"])
    );
    // Every condition, each without a bar on the label beside it; the
    // object as a whole takes nothing else.
    let laid = cancellation["oneOf"].as_array().unwrap();
    let conditions = definitions["Condition"]["oneOf"].as_array().unwrap();
    assert_eq!(laid.len(), conditions.len());
    for (beside, condition) in laid.iter().zip(conditions) {
        assert_eq!(
            beside["properties"]["type"],
            condition["properties"]["type"]
        );
        assert!(beside.get("additionalProperties").is_none(), "{beside}");
    }
    assert_eq!(cancellation["unevaluatedProperties"], false);
}
