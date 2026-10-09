//! A tool answers as the façade does (`03-design/mcp-server.md` §5, the
//! second gate): every record tool is called with an example of its
//! record, and its structured content is the façade's own answer, value
//! and provenance, input hash included.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index the replies they expect"
)]

use serde_json::{Value, json};
use teistro::{Context, Ephemeris};
use teistro_mcp::{Engine, Server};

/// One record of each tool's, the examples its area's own documentation
/// and tests send.
fn examples() -> Vec<(&'static str, Value)> {
    let birth = |instant: f64| {
        json!({"instant": instant, "latitudeDeg": 27.7172, "longitudeDeg": 85.324,
            "altitudeM": 1400, "utcOffsetSeconds": 20700, "uncertaintyMinutes": 0})
    };
    vec![
        ("matching.naam", json!({"bride": "सीता", "groom": "राम"})),
        (
            "numerology.profile",
            json!({"name": "Henry Elder", "date": {"year": 1872, "month": 1, "day": 17},
                "rules": {"masters": "NONE"}}),
        ),
        (
            "almanac.pakshi",
            json!({"first": {"year": 1984, "month": 10, "day": 30},
                "last": {"year": 1984, "month": 11, "day": 1},
                "latitudeDeg": 13.0827, "longitudeDeg": 80.2707, "altitudeM": 6,
                "utcOffsetSeconds": 19800, "native": {"bird": "COCK"}}),
        ),
        (
            "chart.rashifal",
            json!({"periods": [{"first": {"year": 2026, "month": 10, "day": 4},
                "last": {"year": 2026, "month": 10, "day": 10},
                "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1400,
                "utcOffsetSeconds": 20700}], "baseline": "WEEKLY"}),
        ),
        (
            "research.study",
            json!({"study": "COUNTS", "rules": {"shipped": ["YOGAS"]}, "holds": "FORMED",
                "births": [birth(2_446_000.25), birth(2_447_100.5), birth(2_448_300.75)],
                "design": {"groups": [0, 1, 1]}}),
        ),
    ]
}

fn call(server: &mut Server, name: &str, arguments: &Value) -> Value {
    let reply = server
        .handle(
            &json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {
                "_meta": {"io.modelcontextprotocol/protocolVersion": "2026-07-28"},
                "name": name, "arguments": arguments}})
            .to_string(),
        )
        .unwrap();
    let reply: Value = serde_json::from_str(&reply).unwrap();
    reply["result"].clone()
}

#[test]
fn every_record_tool_has_an_example_and_every_example_a_tool() {
    let mut tools: Vec<&str> = teistro::records::records().iter().map(|r| r.name).collect();
    let mut examples: Vec<&str> = examples().iter().map(|(name, _)| *name).collect();
    tools.sort_unstable();
    examples.sort_unstable();
    assert_eq!(tools, examples);
}

#[test]
fn a_tool_answers_as_the_facade_does() {
    let mut server = Server::new(Engine::Builtin);
    for profile in [None, Some("conformance-baseline")] {
        let mut builder = Context::builder().ephemeris([Ephemeris::Builtin]);
        if let Some(profile) = profile {
            builder = builder.profile(profile);
        }
        let context = builder.build().unwrap();
        for (name, request) in examples() {
            let mut arguments = json!({"request": request});
            if let Some(profile) = profile {
                arguments["profile"] = json!(profile);
            }
            let result = call(&mut server, name, &arguments);
            assert_eq!(result["isError"], false, "{name}: {result}");
            let answered = teistro::records::record(name)
                .unwrap()
                .answer(&context, &request.to_string())
                .unwrap();
            let structured = &result["structuredContent"];
            assert_eq!(structured["value"], answered.value, "{name}");
            match answered.provenance {
                Some(provenance) => {
                    assert_eq!(
                        structured["provenance"],
                        serde_json::to_value(provenance).unwrap()
                    );
                }
                None => assert!(structured.get("provenance").is_none(), "{name}"),
            }
        }
    }
}

#[test]
fn a_tool_reads_the_settings_and_the_locale_it_names() {
    let mut server = Server::new(Engine::Builtin);
    let (name, request) = examples().remove(2);
    let centred = |server: &mut Server, centre: &str| {
        let result = call(
            server,
            name,
            &json!({"request": request, "settings": {"frame": {"centre": centre}}}),
        );
        assert_eq!(result["isError"], false, "{result}");
        result["structuredContent"]["provenance"]["settings_hash"].clone()
    };
    let geocentric = centred(&mut server, "GEOCENTRIC");
    assert_ne!(
        geocentric,
        centred(&mut server, "TOPOCENTRIC"),
        "the patch is applied"
    );
    assert_eq!(
        geocentric,
        centred(&mut server, "GEOCENTRIC"),
        "and the context kept"
    );
    let unknown = call(
        &mut server,
        name,
        &json!({"request": request, "locale": "xx-Zzzz"}),
    );
    assert_eq!(unknown["isError"], true);
    assert_eq!(unknown["structuredContent"]["field"], "locale");
}

#[test]
fn every_record_tool_is_described_by_its_boundary_function() {
    let mut server = Server::new(Engine::None);
    let reply = server
        .handle(r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28"}}}"#)
        .unwrap();
    let reply: Value = serde_json::from_str(&reply).unwrap();
    for tool in reply["result"]["tools"].as_array().unwrap() {
        let description = tool["description"].as_str().unwrap();
        assert_eq!(tool["annotations"]["readOnlyHint"], true);
        assert_eq!(tool["inputSchema"]["type"], "object");
        if tool["name"] != "settings.describe" {
            assert!(
                description.contains("`request` is"),
                "{}: {description}",
                tool["name"]
            );
            assert!(!description.contains("request_json"), "{}", tool["name"]);
        }
    }
}

#[test]
fn settings_describe_answers_the_profiles_and_the_patch_schema() {
    let mut server = Server::new(Engine::None);
    let result = call(&mut server, "settings.describe", &json!({}));
    assert_eq!(result["isError"], false);
    let described = &result["structuredContent"];
    assert_eq!(
        described["defaultProfile"],
        teistro::settings::DEFAULT_PROFILE
    );
    let ids: Vec<&str> = described["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|profile| profile["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, teistro::settings::SHIPPED_PROFILES);
    let schema = &described["settings"];
    assert!(schema["$schema"].as_str().unwrap().contains("2020-12"));
    assert!(schema["properties"]["frame"].is_object(), "{schema}");
    let refused = call(
        &mut server,
        "settings.describe",
        &json!({"profile": "kp-default"}),
    );
    assert_eq!(refused["isError"], true);
    assert_eq!(refused["structuredContent"]["field"], "arguments.profile");
}
