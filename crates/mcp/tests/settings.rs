//! `settings.explain` and `settings.diff` as tools
//! (`03-design/settings-and-profiles.md` §3.5): a call's scope explains
//! as a context built with the same profile and patch does, its patch
//! named as the context's layer, and a diff names the context it is
//! compared with as `other`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index the replies they expect"
)]

use serde_json::{Value, json};
use teistro::Context;
use teistro::settings::SettingsQuery;
use teistro_mcp::{Engine, Server};

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

#[test]
fn a_scope_explains_as_the_context_built_with_it() {
    let mut server = Server::new(Engine::Builtin);
    let patch = r#"{"frame": {"node": "MEAN"}}"#;
    let answered = call(
        &mut server,
        "settings.explain",
        &json!({"request": {}, "profile": "nepali-drik",
                "settings": serde_json::from_str::<Value>(patch).unwrap(), "locale": "ne-Deva-NP"}),
    );
    assert_eq!(answered["isError"], false, "{answered}");
    let value = &answered["structuredContent"]["value"];

    let built = Context::builder()
        .profile("nepali-drik")
        .settings_json(patch)
        .locale("ne-Deva-NP")
        .build()
        .unwrap();
    let expected = built.explain_settings(&SettingsQuery::default()).unwrap();
    assert_eq!(value, &serde_json::to_value(&expected).unwrap());

    let node = value["knobs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|knob| knob["knob"] == "frame.node")
        .unwrap();
    // The call's patch is the context's layer, and its note Nepali.
    assert_eq!(node["layer"], "CONTEXT");
    assert_eq!(node["citation"], Value::Null);
    assert!(node["effect"].as_str().unwrap().contains("राहु"));
}

#[test]
fn a_diff_names_the_scope_it_is_compared_with() {
    // No ephemeris, which every profile opens under: the committee's
    // Surya Siddhanta is refused over a modern one.
    let mut server = Server::new(Engine::None);
    let diff = call(
        &mut server,
        "settings.diff",
        &json!({"request": {"knobs": ["frame"]}, "profile": "nepali-committee",
                "other": {"profile": "nepali-drik"}}),
    );
    assert_eq!(diff["isError"], false, "{diff}");
    let value = &diff["structuredContent"]["value"];
    assert_eq!(value["otherProfile"], "nepali-drik");
    let knobs: Vec<&str> = value["knobs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|knob| knob["knob"].as_str().unwrap())
        .collect();
    assert_eq!(
        knobs,
        ["frame.ayanamsha", "frame.centre", "frame.siddhanta"]
    );

    let alone = call(&mut server, "settings.diff", &json!({"request": {}}));
    assert_eq!(alone["structuredContent"]["value"]["knobs"], json!([]));

    let refused = call(
        &mut server,
        "settings.explain",
        &json!({"request": {"knobs": ["frame.nod"]}}),
    );
    assert_eq!(refused["isError"], true);
    assert!(
        refused["structuredContent"]["field"]
            .as_str()
            .is_some_and(|field| field.ends_with("knobs[0]")),
        "{refused}"
    );
}
