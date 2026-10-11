//! A program's own tools (`03-design/mcp-server.md` §7, P9): listed and
//! answered beside the SDK's records, refused at registration for a name
//! the SDK holds or reserves, and failing as a tool rather than as the
//! server.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index the replies they expect"
)]

use serde_json::{Value, json};
use teistro::Error;
use teistro_mcp::{Engine, Limits, Server, Tool};

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

fn greet() -> Tool {
    Tool::new(
        "acme.greet",
        "Greets a querent by name.",
        json!({ "type": "object", "properties": { "name": { "type": "string" } },
                "required": ["name"] }),
        |call| {
            let name: String = call
                .argument("name")?
                .ok_or_else(|| Error::invalid_arg("a name").with_field("arguments.name"))?;
            Ok(json!({ "greeting": format!("Namaste, {name}") }))
        },
    )
    .titled("Greet")
    .read_only()
}

/// **A program's tool is a tool** (P9): listed in name order with its
/// schema and annotations, answered with its structured content, and a
/// refusal its handler gives is the tool's result with `isError`.
#[test]
fn a_programs_tool_is_listed_and_answered() {
    let mut server = Server::new(Engine::None).with_tool(greet()).unwrap();
    let listed = ask(&mut server, "tools/list", json!({}));
    let tools = listed["result"]["tools"].as_array().unwrap();
    let names: Vec<&str> = tools
        .iter()
        .filter_map(|tool| tool["name"].as_str())
        .collect();
    let mut sorted = names.clone();
    sorted.sort_unstable();
    assert_eq!(names, sorted, "the list stays in name order");
    let tool = tools
        .iter()
        .find(|tool| tool["name"] == "acme.greet")
        .unwrap();
    assert_eq!(tool["title"], "Greet");
    assert_eq!(tool["annotations"]["readOnlyHint"], true);
    let (answer, refused) = call(&mut server, "acme.greet", &json!({ "name": "Asha" }));
    assert!(!refused, "{answer}");
    assert_eq!(answer["greeting"], "Namaste, Asha");
    let (refusal, refused) = call(&mut server, "acme.greet", &json!({}));
    assert!(refused);
    assert_eq!(refusal["field"], "arguments.name", "{refusal}");
}

/// **A name the SDK holds or reserves is refused** at registration: an
/// exact SDK tool, a namespace it lists tools under (so an upgrade can
/// never shadow a program's tool), one already given, a name the
/// protocol does not allow, and a schema that is not an object.
#[test]
fn a_name_the_sdk_holds_or_reserves_is_refused() {
    let named = |name: &str| Tool::new(name, "x", json!({ "type": "object" }), |_| Ok(json!({})));
    for name in [
        "chart.found",
        "chart.mine",
        "engine.mine",
        "settings.describe",
        "acme greet",
        "",
    ] {
        let refused = Server::new(Engine::None)
            .with_tool(named(name))
            .unwrap_err();
        assert_eq!(refused.field(), Some("tool.name"), "{name}: {refused}");
    }
    let twice = Server::new(Engine::None)
        .with_tool(greet())
        .unwrap()
        .with_tool(greet());
    assert_eq!(twice.unwrap_err().field(), Some("tool.name"));
    let untyped = Tool::new("acme.x", "x", json!({ "type": "string" }), |_| {
        Ok(json!({}))
    });
    let refused = Server::new(Engine::None).with_tool(untyped).unwrap_err();
    assert_eq!(refused.field(), Some("tool.inputSchema"));
    let claims = Tool::new(
        "acme.x",
        "x",
        json!({ "type": "object", "properties": { "locale": { "type": "string" } } }),
        |_| Ok(json!({})),
    )
    .in_context();
    let refused = Server::new(Engine::None).with_tool(claims).unwrap_err();
    assert_eq!(refused.field(), Some("tool.inputSchema.properties.locale"));
}

/// **A tool in context reaches the server's context** (P9): its listed
/// schema gains `profile`, `settings` and `locale`, its handler sees its
/// own arguments without them, and the context it reaches is the one
/// they name; a tool not in context is refused one.
#[test]
fn a_tool_in_context_reaches_the_context_its_arguments_name() {
    let profiled = Tool::new(
        "acme.profile",
        "The profile the call computes under.",
        json!({ "type": "object", "properties": { "note": { "type": "string" } } }),
        |call| {
            let context = call.context()?;
            Ok(json!({ "profile": context.profile(),
                       "arguments": call.arguments() }))
        },
    )
    .in_context();
    let reaches = Tool::new(
        "acme.bare",
        "No context.",
        json!({ "type": "object" }),
        |call| call.context().map(|_| json!({})),
    );
    let mut server = Server::new(Engine::None)
        .with_tool(profiled)
        .unwrap()
        .with_tool(reaches)
        .unwrap();
    let listed = ask(&mut server, "tools/list", json!({}));
    let tool = listed["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|tool| tool["name"] == "acme.profile")
        .unwrap()
        .clone();
    for scope in ["profile", "settings", "locale", "note"] {
        assert!(
            tool["inputSchema"]["properties"].get(scope).is_some(),
            "{scope}: {tool}"
        );
    }
    let (answer, refused) = call(
        &mut server,
        "acme.profile",
        &json!({ "profile": "parashari-classical", "note": "n" }),
    );
    assert!(!refused, "{answer}");
    assert_eq!(answer["profile"], "parashari-classical");
    assert_eq!(answer["arguments"], json!({ "note": "n" }));
    let (refusal, refused) = call(&mut server, "acme.bare", &json!({}));
    assert!(refused);
    assert_eq!(refusal["status"], "INTERNAL", "{refusal}");
}

/// **A tool fails as a tool** (P9): a handler that panics is the tool's
/// `INTERNAL` refusal and the server answers the next call; the request
/// bounds hold for a program's arguments as for a record's.
#[test]
fn a_panicking_or_oversized_call_fails_as_the_tool() {
    let panics = Tool::new(
        "acme.panics",
        "Panics.",
        json!({ "type": "object" }),
        |_| panic!("inside the handler"),
    );
    let limits = Limits {
        items: Some(2),
        ..Limits::default()
    };
    let mut server = Server::new(Engine::None)
        .with_limits(limits)
        .with_tool(panics)
        .unwrap()
        .with_tool(greet())
        .unwrap();
    let (refusal, refused) = call(&mut server, "acme.panics", &json!({}));
    assert!(refused);
    assert_eq!(refusal["status"], "INTERNAL", "{refusal}");
    let (answer, refused) = call(&mut server, "acme.greet", &json!({ "name": "Ravi" }));
    assert!(!refused, "{answer}");
    let (refusal, refused) = call(
        &mut server,
        "acme.greet",
        &json!({ "name": "Ravi", "extra": [1, 2, 3] }),
    );
    assert!(refused);
    assert_eq!(refusal["status"], "LIMIT", "{refusal}");
}
