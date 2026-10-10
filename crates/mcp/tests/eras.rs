//! Both revisions end to end, through the binary over stdio
//! (`03-design/mcp-server.md` §5, the first gate): the modern one in one
//! process, the handshake one in a fresh process, every result held to
//! its revision's shape.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index the replies they expect"
)]

use std::io::{BufRead as _, BufReader, Write as _};
use std::process::{Command, Stdio};

use serde_json::{Value, json};

const MODERN: &str = "2026-07-28";

/// Every reply the binary writes to `messages`, in order, after its stdin
/// closes.
fn session(args: &[&str], messages: &[Value]) -> Vec<Value> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_teistro-mcp"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("the binary starts");
    let mut stdin = child.stdin.take().unwrap();
    for message in messages {
        writeln!(stdin, "{message}").unwrap();
    }
    drop(stdin);
    let replies = BufReader::new(child.stdout.take().unwrap())
        .lines()
        .map(|line| serde_json::from_str(&line.unwrap()).expect("a reply is JSON"))
        .collect();
    assert!(child.wait().unwrap().success());
    replies
}

fn meta() -> Value {
    json!({
        "io.modelcontextprotocol/protocolVersion": MODERN,
        "io.modelcontextprotocol/clientInfo": {"name": "eras", "version": "0"},
        "io.modelcontextprotocol/clientCapabilities": {},
    })
}

fn numerology() -> Value {
    json!({"name": "Henry Elder", "date": {"year": 1872, "month": 1, "day": 17}})
}

#[test]
fn the_modern_revision_is_served_without_a_handshake() {
    let replies = session(
        &["--ephemeris", "NONE"],
        &[
            json!({"jsonrpc": "2.0", "id": 1, "method": "server/discover", "params": {"_meta": meta()}}),
            json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {"_meta": meta()}}),
            json!({"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"_meta": meta(),
                "name": "numerology.profile", "arguments": {"request": numerology()}}}),
            json!({"jsonrpc": "2.0", "id": 4, "method": "tools/list",
                "params": {"_meta": {"io.modelcontextprotocol/protocolVersion": "2024-01-01"}}}),
            json!({"jsonrpc": "2.0", "id": 5, "method": "tools/list", "params": {}}),
            json!({"jsonrpc": "2.0", "id": 6, "method": "tools/call", "params": {"_meta": meta(),
                "name": "chart.horoscope", "arguments": {}}}),
            json!({"jsonrpc": "2.0", "id": 7, "method": "resources/list", "params": {"_meta": meta()}}),
        ],
    );
    assert_eq!(replies.len(), 7);
    for (reply, id) in replies.iter().zip(1..) {
        assert_eq!(reply["jsonrpc"], "2.0");
        assert_eq!(reply["id"], id);
    }

    let discovered = &replies[0]["result"];
    assert_eq!(discovered["resultType"], "complete");
    assert_eq!(
        discovered["supportedVersions"],
        json!([MODERN, "2025-11-25"])
    );
    assert!(discovered["capabilities"]["tools"].is_object());
    assert_eq!(
        discovered["_meta"]["io.modelcontextprotocol/serverInfo"]["name"],
        "teistro-mcp"
    );
    assert!(discovered["ttlMs"].is_u64());

    let listed = &replies[1]["result"];
    assert_eq!(listed["resultType"], "complete");
    assert_eq!(listed["cacheScope"], "public");
    assert!(listed["ttlMs"].is_u64());
    let names: Vec<&str> = listed["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect();
    let mut sorted = names.clone();
    sorted.sort_unstable();
    assert_eq!(names, sorted, "a deterministic order");
    for tool in [
        "research.study",
        "almanac.pakshi",
        "chart.rashifal",
        "numerology.profile",
        "matching.naam",
        "settings.describe",
    ] {
        assert!(names.contains(&tool), "{tool}");
    }

    let called = &replies[2]["result"];
    assert_eq!(called["resultType"], "complete");
    assert_eq!(called["isError"], false);
    let structured = &called["structuredContent"];
    assert!(structured["value"].is_object());
    let text: Value = serde_json::from_str(called["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(&text, structured, "the text is the same JSON");

    let unsupported = &replies[3]["error"];
    assert_eq!(unsupported["code"], -32_022);
    assert_eq!(
        unsupported["data"]["supported"],
        json!([MODERN, "2025-11-25"])
    );
    assert_eq!(unsupported["data"]["requested"], "2024-01-01");
    let unnamed = &replies[4]["error"];
    assert_eq!(unnamed["code"], -32_022, "no revision and no handshake");
    assert_eq!(unnamed["data"]["requested"], Value::Null);
    assert_eq!(replies[5]["error"]["code"], -32_602, "an unknown tool");
    assert_eq!(replies[6]["error"]["code"], -32_601);
}

#[test]
fn the_handshake_selects_the_legacy_revision_for_the_process() {
    let replies = session(
        &["--ephemeris", "NONE"],
        &[
            json!({"jsonrpc": "2.0", "id": "a", "method": "initialize", "params": {
                "protocolVersion": "2025-11-25", "capabilities": {},
                "clientInfo": {"name": "eras", "version": "0"}}}),
            json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
            json!({"jsonrpc": "2.0", "id": "b", "method": "ping"}),
            json!({"jsonrpc": "2.0", "id": "c", "method": "tools/list"}),
            json!({"jsonrpc": "2.0", "id": "d", "method": "tools/call", "params": {
                "name": "numerology.profile", "arguments": {"request": numerology()}}}),
        ],
    );
    assert_eq!(replies.len(), 4, "the notification is not answered");
    let initialized = &replies[0]["result"];
    assert_eq!(initialized["protocolVersion"], "2025-11-25");
    assert_eq!(initialized["serverInfo"]["name"], "teistro-mcp");
    assert!(initialized["capabilities"]["tools"].is_object());
    assert!(initialized["instructions"].is_string());
    assert_eq!(replies[1]["result"], json!({}));
    let listed = &replies[2]["result"];
    assert!(listed["tools"].is_array());
    for field in ["resultType", "ttlMs", "cacheScope"] {
        assert!(
            listed.get(field).is_none(),
            "{field} is the modern revision's"
        );
    }
    let called = &replies[3]["result"];
    assert_eq!(called["isError"], false);
    assert!(called.get("resultType").is_none());
}

#[test]
fn a_refusal_is_a_tool_error_naming_its_field() {
    let replies = session(
        &["--ephemeris", "NONE"],
        &[
            json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"_meta": meta(),
                "name": "numerology.profile", "arguments": {"request":
                    {"name": "Henry", "date": {"year": 1872, "month": 1, "day": 17}, "rules": {"master": "NONE"}}}}}),
            json!({"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"_meta": meta(),
                "name": "numerology.profile", "arguments": {"request": numerology(), "profile": "vedic"}}}),
            json!({"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"_meta": meta(),
                "name": "numerology.profile", "arguments": {"request": numerology(), "colour": "red"}}}),
            json!({"jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": {"_meta": meta(),
                "name": "numerology.profile", "arguments": {"request": numerology(),
                    "settings": {"houses": {"chalit_systm": "SRIPATI"}}}}}),
            json!({"jsonrpc": "2.0", "id": 5, "method": "tools/call", "params": {"_meta": meta(),
                "name": "almanac.pakshi", "arguments": {"request": {
                    "first": {"year": 1984, "month": 10, "day": 30},
                    "latitudeDeg": 13.0827, "longitudeDeg": 80.2707, "altitudeM": 6,
                    "utcOffsetSeconds": 19800, "native": {"bird": "COCK"}}}}}),
        ],
    );
    let fields: Vec<(&str, &str)> = replies
        .iter()
        .map(|reply| {
            let result = &reply["result"];
            assert_eq!(result["isError"], true, "{reply}");
            let refusal = &result["structuredContent"];
            assert!(refusal["message"].is_string());
            (
                refusal["status"].as_str().unwrap(),
                refusal["field"].as_str().unwrap_or(""),
            )
        })
        .collect();
    assert_eq!(fields[0].1, "numerology.rules.master");
    assert_eq!(fields[1].1, "profile");
    assert_eq!(fields[2].1, "arguments.colour");
    assert!(
        fields[3].1.starts_with("settings.houses"),
        "{:?}",
        fields[3]
    );
    assert_eq!(fields[4].0, "CAPABILITY", "a position without an ephemeris");
}

#[test]
fn the_command_line_refuses_an_unknown_ephemeris() {
    let out = Command::new(env!("CARGO_BIN_EXE_teistro-mcp"))
        .args(["--ephemeris", "SWISS"])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("BUILTIN, SURYA_SIDDHANTA, NONE"));
}

#[test]
fn the_command_line_loads_a_plugin_and_lists_its_operations() {
    let path = teistro_test_adapter::library().unwrap();
    let path = path.to_string_lossy();
    let replies = session(
        &[
            "--plugin",
            &path,
            "--plugin-config",
            "{}",
            "--ephemeris",
            "NONE",
        ],
        &[
            json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"_meta": meta(),
                "name": "engine.tp_echo", "arguments": {"value": 7}}}),
        ],
    );
    let result = &replies[0]["result"];
    assert_eq!(result["isError"], false, "{result}");
    assert_eq!(result["structuredContent"]["value"]["value"], 7.0);
    assert_eq!(
        result["structuredContent"]["provenance"]["provider"]["name"],
        "test-provider"
    );
}

#[test]
fn the_command_line_refuses_a_plugin_it_cannot_load() {
    let out = Command::new(env!("CARGO_BIN_EXE_teistro-mcp"))
        .args(["--plugin", "/nowhere/libnothing.so"])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("/nowhere/libnothing.so"));

    let out = Command::new(env!("CARGO_BIN_EXE_teistro-mcp"))
        .args(["--plugin-config", "{}"])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2), "a configuration with no plugin");
}
