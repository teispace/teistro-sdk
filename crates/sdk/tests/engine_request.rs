//! The engine passthrough read from JSON (the agent server's
//! `engine.manifest` and `engine.call`): the engine's own operations,
//! answered sealed with a provenance naming the engine.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index the answers they expect"
)]

use teistro::records::record;
use teistro::{Context, EngineCall, Ephemeris, ManifestRequest, Status, content_hash};

fn over_the_test_provider() -> Context {
    Context::builder()
        .ephemeris([Ephemeris::Test])
        .build()
        .unwrap()
}

#[test]
fn each_record_is_the_request_it_spells() {
    let call = EngineCall::from_json(r#"{"name": "tp_echo", "arguments": {"value": 2}}"#).unwrap();
    assert_eq!(call.name, "tp_echo");
    assert_eq!(call.arguments["value"], 2);
    let bare = EngineCall::from_json(r#"{"name": "tp_sum"}"#).unwrap();
    assert!(bare.arguments.is_empty(), "no arguments is an empty object");
    ManifestRequest::from_json("{}").unwrap();
}

#[test]
fn a_refusal_names_the_field_written() {
    for (refused, field) in [
        (
            EngineCall::from_json(r#"{"name": "tp_sum", "args": {}}"#).unwrap_err(),
            "args",
        ),
        (
            EngineCall::from_json(r#"{"arguments": {}}"#).unwrap_err(),
            "name",
        ),
        (
            ManifestRequest::from_json(r#"{"name": "tp_sum"}"#).unwrap_err(),
            "name",
        ),
    ] {
        assert_eq!(refused.field(), Some(field), "{refused}");
    }
}

#[test]
fn the_engine_answers_sealed_and_by_its_own_name() {
    let context = over_the_test_provider();
    let manifest = record("engine.manifest")
        .unwrap()
        .answer(&context, "{}")
        .unwrap();
    let names: Vec<&str> = manifest.value["functions"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|function| function["name"].as_str())
        .collect();
    assert_eq!(names, ["tp_echo", "tp_sum"]);

    let summed = record("engine.call")
        .unwrap()
        .answer(
            &context,
            r#"{"name": "tp_sum", "arguments": {"values": [1, 2.5]}}"#,
        )
        .unwrap();
    assert_eq!(summed.value["total"], 3.5);
    let provenance = summed.provenance.unwrap();
    assert_eq!(provenance.content_hash, content_hash(&summed.value));
    assert_eq!(provenance.provider.name, "test-provider");
    assert_eq!(provenance.provider.flags_used, ["engine.tp_sum"]);

    let unknown = record("engine.call")
        .unwrap()
        .answer(&context, r#"{"name": "tp_product"}"#)
        .unwrap_err();
    assert_eq!(unknown.status, Status::Unsupported, "{unknown}");
}

#[test]
fn an_engine_with_no_operations_of_its_own_refuses_by_name() {
    let classical = Context::builder()
        .ephemeris([Ephemeris::SuryaSiddhanta])
        .build()
        .unwrap();
    let refused = record("engine.manifest")
        .unwrap()
        .answer(&classical, "{}")
        .unwrap_err();
    assert_eq!(refused.status, Status::Unsupported, "{refused}");

    // No ephemeris at all is the capability refusal, naming the option.
    let bare = Context::builder().build().unwrap();
    let refused = record("engine.call")
        .unwrap()
        .answer(&bare, r#"{"name": "tp_sum"}"#)
        .unwrap_err();
    assert_eq!(refused.status, Status::Capability, "{refused}");
    assert_eq!(refused.field(), Some("ephemeris"));
}
