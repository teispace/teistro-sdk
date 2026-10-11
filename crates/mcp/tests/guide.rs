//! The site's guide to the agent server (`site/content/docs/mcp.mdx`) is
//! read here, so what it shows is what the server does: each call it
//! shows is made, and the structured content it shows beside it is the
//! answer, value for value; each option it names is one the program
//! reads.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index the replies they expect"
)]

use std::process::Command;

use serde_json::{Value, json};
use teistro_mcp::{Engine, Server};

const GUIDE: &str = "../../site/content/docs/mcp.mdx";

fn guide() -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(GUIDE)).unwrap()
}

/// Each fenced block the guide titles `title`, in order, as JSON.
fn blocks(text: &str, title: &str) -> Vec<Value> {
    let fence = format!("```json title=\"{title}\"");
    let mut found = Vec::new();
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        if line.trim() == fence {
            let body: Vec<&str> = lines
                .by_ref()
                .take_while(|line| line.trim() != "```")
                .collect();
            found.push(
                serde_json::from_str(&body.join("\n"))
                    .unwrap_or_else(|why| panic!("a `{title}` block is not JSON: {why}")),
            );
        }
    }
    found
}

/// **Each call the guide shows answers what it shows**: the call made
/// over the built-in ephemeris under the 2026-07-28 revision, its
/// structured content the block beside it, refusals included.
#[test]
fn every_call_the_guide_shows_answers_what_it_shows() {
    let text = guide();
    let calls = blocks(&text, "tools/call");
    let answers = blocks(&text, "structuredContent");
    assert!(!calls.is_empty(), "the guide shows no call");
    assert_eq!(calls.len(), answers.len(), "a call a shown answer");
    let mut server = Server::new(Engine::Builtin);
    for (call, shown) in calls.into_iter().zip(answers) {
        let mut params = call.clone();
        params["_meta"] = json!({ "io.modelcontextprotocol/protocolVersion": "2026-07-28",
                                  "io.modelcontextprotocol/clientCapabilities": {} });
        let message =
            json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": params });
        let reply: Value =
            serde_json::from_str(&server.handle(&message.to_string()).unwrap()).unwrap();
        assert_eq!(
            reply["result"]["structuredContent"], shown,
            "{}: the guide shows another answer",
            call["name"]
        );
    }
}

/// **Each option the guide names is one the program reads**, as its own
/// `--help` lists them.
#[test]
fn every_option_the_guide_names_is_the_programs() {
    let help = Command::new(env!("CARGO_BIN_EXE_teistro-mcp"))
        .arg("--help")
        .output()
        .unwrap();
    let help = String::from_utf8_lossy(&help.stdout).into_owned();
    let text = guide();
    let mut named = 0;
    for (at, _) in text.match_indices("`--") {
        let option: String = text[at + 1..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect();
        named += 1;
        assert!(
            help.contains(&format!("{option} ")) || help.contains(&format!("{option}\n")),
            "the guide names `{option}`, which `teistro-mcp --help` does not"
        );
    }
    assert!(named > 0, "the guide names no option");
}
