//! The MCP Registry's entry (`server.json`, `03-design/mcp-server.md` §6
//! step 5): what the registry lists is what the program says of itself and
//! what npm installs, so neither can drift from the other unread.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index the documents they read"
)]

use serde_json::{Value, json};
use teistro_mcp::{Engine, Server};

fn read(file: &str) -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(file);
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// The server's identity, as every modern result names it.
fn identity() -> Value {
    let mut server = Server::new(Engine::None);
    let message = json!({ "jsonrpc": "2.0", "id": 1, "method": "server/discover", "params": {
        "_meta": { "io.modelcontextprotocol/protocolVersion": "2026-07-28",
                   "io.modelcontextprotocol/clientCapabilities": {} } } });
    let reply: Value = serde_json::from_str(&server.handle(&message.to_string()).unwrap()).unwrap();
    reply["result"]["_meta"]["io.modelcontextprotocol/serverInfo"].clone()
}

/// **The registry lists what the program says** (2025-12-11 schema): the
/// title, the description within the registry's 100 characters, the
/// website and the version, each the server's own; a name in the
/// namespace the release's GitHub login proves.
#[test]
fn the_entry_says_what_the_server_says_of_itself() {
    let entry = read("server.json");
    let said = identity();
    assert_eq!(entry["title"], said["title"], "{said}");
    assert_eq!(entry["description"], said["description"], "{said}");
    assert_eq!(entry["websiteUrl"], said["websiteUrl"]);
    assert_eq!(entry["version"], env!("CARGO_PKG_VERSION"));
    let description = entry["description"].as_str().unwrap();
    assert!(
        (1..=100).contains(&description.chars().count()),
        "{description}"
    );
    assert!(
        entry["name"]
            .as_str()
            .unwrap()
            .starts_with("io.github.teispace/")
    );
}

/// **The entry names what npm installs**: the launcher's package and
/// version, its `mcpName` the entry's name (how the registry proves the
/// package is this server's), and the choices it offers the ones the
/// program reads.
#[test]
fn the_entry_names_the_npm_launcher_and_its_choices() {
    let entry = read("server.json");
    let launcher = read("npm/package.json");
    let npm = entry["packages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|package| package["registryType"] == "npm")
        .unwrap();
    assert_eq!(npm["identifier"], launcher["name"]);
    assert_eq!(npm["version"], launcher["version"]);
    assert_eq!(npm["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(launcher["mcpName"], entry["name"]);
    assert_eq!(npm["transport"], json!({ "type": "stdio" }));
    let ephemeris = npm["packageArguments"]
        .as_array()
        .unwrap()
        .iter()
        .find(|argument| argument["name"] == "--ephemeris")
        .unwrap();
    assert_eq!(ephemeris["choices"], json!(Engine::NAMES));
}
