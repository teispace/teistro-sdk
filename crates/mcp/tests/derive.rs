//! A call's scope is **derived** from one base context per provider, not
//! built (`03-design/mcp-server.md`, rank 8 of the boundary audit): a new
//! settings patch opens no engine, the derived context answers as a built
//! one does, and the cache lets go of the scope used longest ago rather
//! than of every scope at once.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index the replies they expect"
)]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::{Value, json};
use teistro::{Context, Ephemeris};
use teistro_mcp::{Engine, OpenEngine, Server};
use teistro_port_ephemeris::EphemerisProvider;

fn call(server: &mut Server, name: &str, arguments: &Value) -> Value {
    let reply = server
        .handle(
            &json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {
                "_meta": {"io.modelcontextprotocol/protocolVersion": "2026-07-28", "io.modelcontextprotocol/clientCapabilities": {}},
                "name": name, "arguments": arguments}})
            .to_string(),
        )
        .unwrap();
    let reply: Value = serde_json::from_str(&reply).unwrap();
    reply["result"].clone()
}

/// The fixture's `chart.found` request, which every binding's tests read.
fn chart_request() -> Value {
    let fixture: Vec<Value> =
        serde_json::from_str(include_str!("../../sdk/tests/requests.json")).unwrap();
    fixture
        .into_iter()
        .find(|example| example["record"] == "chart.found")
        .unwrap()["request"]
        .clone()
}

/// A server over the built-in, opened through a hook that counts each
/// time the engine is opened.
fn counted() -> (Server, Arc<AtomicUsize>) {
    let opens = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&opens);
    let open: OpenEngine = Box::new(move || {
        counter.fetch_add(1, Ordering::SeqCst);
        let provider: Box<dyn EphemerisProvider> = Ephemeris::Builtin.open()?.unwrap();
        Ok(provider)
    });
    (Server::with_engine(Engine::Builtin, open).unwrap(), opens)
}

#[test]
fn a_second_patch_opens_no_engine_and_answers_as_a_built_context() {
    let (mut server, opens) = counted();
    // The server opened the engine once to read what it offers.
    let started = opens.load(Ordering::SeqCst);
    let request = chart_request();
    let mean = call(
        &mut server,
        "chart.found",
        &json!({"request": request, "detail": "full", "settings": {"frame": {"node": "MEAN"}}}),
    );
    assert_eq!(mean["isError"], false, "{mean}");
    // The base context opens it once more, and only once.
    let based = opens.load(Ordering::SeqCst);
    assert_eq!(based, started + 1);
    let true_node = call(
        &mut server,
        "chart.found",
        &json!({"request": request, "detail": "full", "settings": {"frame": {"node": "TRUE"}}, "locale": "en-Latn"}),
    );
    assert_eq!(true_node["isError"], false, "{true_node}");
    let profiled = call(
        &mut server,
        "chart.found",
        &json!({"request": request, "detail": "full", "profile": "kp-default"}),
    );
    assert_eq!(profiled["isError"], false, "{profiled}");
    assert_eq!(
        opens.load(Ordering::SeqCst),
        based,
        "a patch reopened the engine"
    );

    // Each answers as a context built under the same scope does.
    for (answered, profile, patch) in [
        (&mean, None, r#"{"frame": {"node": "MEAN"}}"#),
        (&true_node, None, r#"{"frame": {"node": "TRUE"}}"#),
        (&profiled, Some("kp-default"), "{}"),
    ] {
        let mut builder = Context::builder()
            .ephemeris([Ephemeris::Builtin])
            .settings_json(patch);
        if let Some(profile) = profile {
            builder = builder.profile(profile);
        }
        let built = builder.build().unwrap();
        assert_eq!(
            answered["structuredContent"]["provenance"]["settings_hash"],
            built.settings_hash().to_string(),
            "{patch}"
        );
    }
    assert_ne!(
        mean["structuredContent"]["provenance"]["settings_hash"],
        true_node["structuredContent"]["provenance"]["settings_hash"]
    );
}

#[test]
fn a_patch_is_refused_by_its_knob() {
    let (mut server, _) = counted();
    let request = chart_request();
    let unknown = call(
        &mut server,
        "chart.found",
        &json!({"request": request, "settings": {"frame": {"nod": "MEAN"}}}),
    );
    assert_eq!(unknown["isError"], true);
    assert_eq!(unknown["structuredContent"]["field"], "settings.frame.nod");
    let incoherent = call(
        &mut server,
        "chart.found",
        &json!({"request": request, "settings": {"day": {"sunrise": {"kind": "CUSTOM", "altitude_deg": 95.0}}}}),
    );
    assert_eq!(incoherent["isError"], true, "{incoherent}");
    assert!(
        incoherent["structuredContent"]["field"]
            .as_str()
            .unwrap()
            .starts_with("settings."),
        "{incoherent}"
    );
}

#[test]
fn a_full_cache_lets_go_of_the_scope_used_longest_ago() {
    let (mut server, opens) = counted();
    let request = chart_request();
    let ask = |server: &mut Server, cells: u32| {
        let answered = call(
            server,
            "chart.found",
            &json!({"request": request, "settings": {"provider": {"cache_cells": cells}}}),
        );
        assert_eq!(answered["isError"], false, "{answered}");
    };
    for cells in 1..=17 {
        ask(&mut server, cells);
    }
    // Sixteen kept: the first let go, not every one.
    assert!(format!("{server:?}").contains("contexts: 16"), "{server:?}");
    assert_eq!(
        opens.load(Ordering::SeqCst),
        2,
        "every scope derives from one base"
    );
}
