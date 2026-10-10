//! Packs (`03-design/mcp-server.md` §7, P9): a pack the operator names is
//! loaded into every context the server builds, every answer's provenance
//! names it, `settings.describe` lists it, and one that is not a pack
//! fails the start rather than a call.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index the replies they expect"
)]

use std::process::{Command, Stdio};

use serde_json::{Value, json};
use teistro::{Status, Tree};
use teistro_mcp::{Engine, Server};

/// The states corpus's English entity names, as a pack.
fn pack() -> Vec<u8> {
    let states = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packs/states");
    let tree = Tree::load(&states).expect("the states corpus");
    let english = tree.locales.get("en-Latn").expect("an English corpus");
    teistro::pack::build(english, "sdk.entity").expect("a pack")
}

fn call(server: &mut Server, name: &str, arguments: &Value) -> Value {
    let message = json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {
        "_meta": { "io.modelcontextprotocol/protocolVersion": "2026-07-28",
                   "io.modelcontextprotocol/clientCapabilities": {} },
        "name": name, "arguments": arguments } });
    let reply: Value = serde_json::from_str(&server.handle(&message.to_string()).unwrap()).unwrap();
    let result = &reply["result"];
    assert_eq!(result["isError"], false, "{reply}");
    result["structuredContent"].clone()
}

/// A chart at Kathmandu under `profile` with its placements said: an
/// answer whose words a pack shapes.
fn said(profile: &str) -> Value {
    json!({ "profile": profile, "request": {
        "instant": 2_447_987.5, "latitudeDeg": 27.7, "longitudeDeg": 85.3,
        "utcOffsetSeconds": 20_700, "interpret": { "placements": true } } })
}

/// **An answer names the packs that shaped its words** (ADR-0020):
/// each context the server builds, under any profile, loads the pack, a
/// chart said in words stamps it, and a server without one stamps none.
#[test]
fn every_context_loads_the_packs_and_every_answer_names_them() {
    let bytes = pack();
    let mut server = Server::new(Engine::Builtin).with_pack(bytes).unwrap();
    let loaded = server.packs()[0].clone();
    for profile in ["nepali-default", "parashari-classical"] {
        let answer = call(&mut server, "chart.found", &said(profile));
        let packs = &answer["provenance"]["packs"];
        assert_eq!(
            packs.as_array().map(Vec::len),
            Some(1),
            "{profile}: {answer}"
        );
        assert_eq!(packs[0]["id"], "en-Latn/sdk.entity");
        assert_eq!(packs[0]["hash"], loaded.sha256);
    }
    let mut bare = Server::new(Engine::Builtin);
    let answer = call(&mut bare, "chart.found", &said("nepali-default"));
    assert_eq!(answer["provenance"]["packs"], json!([]), "{answer}");
}

/// A model can tell what the operator loaded: `settings.describe` lists
/// each pack's locale, namespaces, entries and digest.
#[test]
fn settings_describe_lists_the_packs() {
    let mut server = Server::new(Engine::None).with_pack(pack()).unwrap();
    let described = call(&mut server, "settings.describe", &json!({}));
    let listed = &described["packs"][0];
    let loaded = server.packs()[0].clone();
    assert_eq!(listed["locale"], "en-Latn");
    assert_eq!(listed["namespaces"], json!(["sdk.entity"]));
    assert_eq!(listed["entries"], loaded.entries);
    assert_eq!(listed["sha256"], loaded.sha256);
}

/// **A bad pack fails the start** (P9): bytes that are not a pack are
/// `PACK` naming `bytes` when given, and through the binary a missing
/// file is a usage error and a file that is not a pack a failure, each
/// naming the path.
#[test]
fn a_pack_that_does_not_verify_fails_the_start() {
    let refused = Server::new(Engine::None)
        .with_pack(&b"TPK1 not a pack"[..])
        .unwrap_err();
    assert_eq!(refused.status, Status::Pack, "{refused}");
    assert_eq!(refused.field(), Some("bytes"));
    let run = |path: &str| {
        Command::new(env!("CARGO_BIN_EXE_teistro-mcp"))
            .args(["--pack", path])
            .stdin(Stdio::null())
            .output()
            .unwrap()
    };
    let missing = run("/nonexistent/readings.tpack");
    assert_eq!(missing.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&missing.stderr).contains("--pack /nonexistent/readings.tpack")
    );
    let junk = std::env::temp_dir().join(format!("teistro-mcp-junk-{}.tpack", std::process::id()));
    std::fs::write(&junk, b"TPK1 not a pack").unwrap();
    let path = junk.to_string_lossy().into_owned();
    let failed = run(&path);
    let _ = std::fs::remove_file(&junk);
    assert_eq!(failed.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&failed.stderr).contains(&format!("--pack {path}")));
}
