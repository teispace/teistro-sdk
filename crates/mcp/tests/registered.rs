//! The operator's own layouts and dasha systems (`03-design/mcp-server.md`
//! §7, P9): registered on the server, reached by their keys in a chart
//! request, listed by `settings.describe`, and refused at the start when
//! the registry would refuse them.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index the replies they expect"
)]

use std::process::{Command, Stdio};

use serde_json::{Value, json};
use teistro::catalogue::Nakshatra;
use teistro::dasha::{UduDefinition, VIMSHOTTARI};
use teistro::{Layout, Layouts};
use teistro_mcp::{Detail, Engine, Server};

/// Vimshottari's table under the operator's key, so its periods have an
/// exact twin in the catalogue's own system.
fn twin() -> UduDefinition {
    UduDefinition {
        lords: VIMSHOTTARI.lords.to_vec(),
        sources: vec![String::from("Vimshottari's table, for the test")],
        ..UduDefinition::of("ACME_VIMSHOTTARI", Nakshatra::Ashwini)
    }
}

/// The East Indian chart under the operator's key.
fn odia() -> Layout {
    let mut layout = Layouts::new().get("EAST_INDIAN").cloned().unwrap();
    layout.key = String::from("ACME_ODIA");
    layout
}

fn server() -> Server {
    Server::new(Engine::Builtin)
        .with_layout(odia())
        .unwrap()
        .with_dasha_system(twin())
        .unwrap()
}

fn ask(server: &mut Server, method: &str, params: Value) -> Value {
    let mut params = params;
    params["_meta"] = json!({ "io.modelcontextprotocol/protocolVersion": "2026-07-28",
                              "io.modelcontextprotocol/clientCapabilities": {} });
    let message = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
    serde_json::from_str(&server.handle(&message.to_string()).unwrap()).unwrap()
}

/// The structured content of a call, and whether it is a refusal.
fn call(server: &mut Server, name: &str, arguments: &Value) -> (Value, bool) {
    let reply = ask(
        server,
        "tools/call",
        json!({ "name": name, "arguments": arguments }),
    );
    let result = &reply["result"];
    (
        result["structuredContent"].clone(),
        result["isError"]
            .as_bool()
            .unwrap_or_else(|| panic!("{reply}")),
    )
}

/// A chart at Kathmandu with `extra` beside it.
fn found(extra: &Value) -> Value {
    let mut request = json!({ "instant": 2_447_995.489_583_333_5, "latitudeDeg": 27.7172,
        "longitudeDeg": 85.324, "altitudeM": 1400, "utcOffsetSeconds": 20_700 });
    for (key, value) in extra.as_object().unwrap() {
        request[key] = value.clone();
    }
    json!({ "profile": "conformance-baseline", "request": request })
}

/// **A registered dasha system is reached by its key** (P9), bare or
/// full, and answers as its twin in the catalogue does, period for
/// period, in the order asked.
#[test]
fn a_registered_dasha_system_is_reached_by_its_key() {
    let mut server = server();
    for key in ["ACME_VIMSHOTTARI", "dasha_system.ACME_VIMSHOTTARI"] {
        let (answer, refused) = call(
            &mut server,
            "chart.found",
            &found(&json!({ "dashas": [key, "VIMSHOTTARI"] })),
        );
        assert!(!refused, "{answer}");
        let dashas = &answer["value"]["charts"][0]["dashas"];
        assert_eq!(dashas[0]["system"], "ACME_VIMSHOTTARI", "{key}");
        assert_eq!(dashas[1]["system"], "VIMSHOTTARI", "{key}");
        assert_eq!(dashas[0]["periods"], dashas[1]["periods"], "{key}");
        assert_eq!(dashas[0]["balance"], dashas[1]["balance"], "{key}");
    }
}

/// **A registered layout draws** (P9): a drawing naming it is the shipped
/// layout it copies, under its own key.
#[test]
fn a_registered_layout_draws() {
    let mut server = server();
    let drawn = |server: &mut Server, layout: &str| {
        let (answer, refused) = call(
            server,
            "chart.found",
            &found(&json!({ "drawings": [{ "layout": layout, "varga": "D1" }], "theme": {} })),
        );
        assert!(!refused, "{answer}");
        answer["value"]["svgs"].clone()
    };
    let own = drawn(&mut server, "chart_layout.ACME_ODIA");
    let shipped = drawn(&mut server, "EAST_INDIAN");
    assert!(own.to_string().contains("<svg"), "{own}");
    assert_eq!(own, shipped, "the layout it copies, drawn alike");
}

/// **A key no registry has is refused by its place** (P9): one nobody
/// registered, and one of another kind, each naming where it was asked.
#[test]
fn a_key_no_registry_has_is_refused_by_its_place() {
    let mut server = server();
    for (extra, field) in [
        (
            json!({ "dashas": ["VIMSHOTTARI", "ACME_NOPE"] }),
            "dashas[1]",
        ),
        (json!({ "dashas": ["chart_layout.ACME_ODIA"] }), "dashas[0]"),
        (
            json!({ "drawings": [{ "layout": "ACME_VIMSHOTTARI", "varga": "D1" }] }),
            "drawings[0].layout",
        ),
    ] {
        let (refusal, refused) = call(&mut server, "chart.found", &found(&extra));
        assert!(refused, "{extra}: {refusal}");
        assert_eq!(refusal["field"], field, "{refusal}");
    }
    // A server that registered nothing refuses the same key the same way.
    let (refusal, refused) = call(
        &mut Server::new(Engine::Builtin),
        "chart.found",
        &found(&json!({ "dashas": ["ACME_VIMSHOTTARI"] })),
    );
    assert!(refused);
    assert_eq!(refusal["field"], "dashas[0]", "{refusal}");
}

/// What a request may name, the schema takes, and `settings.describe`
/// lists what the operator registered.
#[test]
fn the_schema_takes_a_registered_key_and_describe_lists_them() {
    let mut server = server().with_detail(Detail::Full);
    let listed = ask(&mut server, "tools/list", json!({}));
    let tool = listed["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|tool| tool["name"] == "chart.found")
        .unwrap()
        .clone();
    let arguments = found(
        &json!({ "dashas": ["ACME_VIMSHOTTARI", "dasha_system.VIMSHOTTARI"],
        "drawings": [{ "layout": "chart_layout.ACME_ODIA", "varga": "D9" }] }),
    );
    let mut schemas = boon::Schemas::new();
    let mut compiler = boon::Compiler::new();
    compiler
        .add_resource("urn:teistro:tool", tool["inputSchema"].clone())
        .unwrap();
    let index = compiler.compile("urn:teistro:tool", &mut schemas).unwrap();
    schemas
        .validate(&arguments, index)
        .unwrap_or_else(|why| panic!("{why:#}"));
    let (described, _) = call(&mut server, "settings.describe", &json!({}));
    assert_eq!(
        described["registered"],
        json!({ "dashaSystems": ["ACME_VIMSHOTTARI"], "chartLayouts": ["ACME_ODIA"] })
    );
}

/// **A definition the registry refuses fails the start** (P9): a key the
/// SDK ships, or one given twice, refused naming the field; through the
/// binary a file that does not read is a usage error and a row the
/// registry refuses a failure, each naming the file.
#[test]
fn a_definition_the_registry_refuses_fails_the_start() {
    let mut shipped = odia();
    shipped.key = String::from("EAST_INDIAN");
    let refused = Server::new(Engine::None).with_layout(shipped).unwrap_err();
    assert!(refused.field().is_some(), "{refused}");
    let twice = server().with_dasha_system(twin()).unwrap_err();
    assert!(twice.field().is_some(), "{twice}");

    let run = |option: &str, text: &str| {
        let path = std::env::temp_dir().join(format!(
            "teistro-mcp-{}-{}.json",
            option.trim_start_matches('-'),
            std::process::id()
        ));
        std::fs::write(&path, text).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_teistro-mcp"))
            .args([option, &path.to_string_lossy()])
            .stdin(Stdio::null())
            .output()
            .unwrap();
        let _ = std::fs::remove_file(&path);
        (
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
        )
    };
    let (code, stderr) = run("--dashas", "[1]");
    assert_eq!(code, Some(2), "{stderr}");
    assert!(stderr.contains("--dashas "), "{stderr}");
    let mut taken = odia();
    taken.key = String::from("WESTERN_WHEEL");
    let (code, stderr) = run("--layouts", &serde_json::to_string(&vec![taken]).unwrap());
    assert_eq!(code, Some(1), "{stderr}");
    assert!(stderr.contains("--layouts "), "{stderr}");
}
