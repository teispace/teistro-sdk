//! The 2026-07-28 revision's remaining patterns (`03-design/mcp-server.md`
//! D7, D10): a missing input asked for in a form rather than refused, for
//! a client that takes one; the tool list in one order whatever order its
//! tools arrived in; and a request's trace context carried without
//! changing its answer.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index the replies they expect"
)]

use serde_json::{Value, json};
use teistro_mcp::{Engine, Server, Tool};

const KEY: &str = "teistro.missing";

fn meta(capabilities: &Value) -> Value {
    json!({ "io.modelcontextprotocol/protocolVersion": "2026-07-28",
            "io.modelcontextprotocol/clientCapabilities": capabilities })
}

fn ask(server: &mut Server, method: &str, mut params: Value, capabilities: &Value) -> Value {
    if params.get("_meta").is_none() {
        params["_meta"] = meta(capabilities);
    }
    let message = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
    serde_json::from_str(&server.handle(&message.to_string()).unwrap()).unwrap()
}

fn chart(
    server: &mut Server,
    request: &Value,
    capabilities: &Value,
    responses: Option<Value>,
) -> Value {
    let mut params = json!({ "name": "chart.found", "arguments": { "request": request } });
    if let Some(responses) = responses {
        params["inputResponses"] = responses;
    }
    ask(server, "tools/call", params, capabilities)["result"].clone()
}

/// **A birth's missing place, then its instant, is asked for**: first
/// the fields the record's schema requires and the call lacks, in one
/// form whose fields a form can carry, then the instant the SDK refuses a
/// chart without; the retry's answers complete the call.
#[test]
fn a_missing_input_is_asked_for_and_the_retry_is_answered() {
    let mut server = Server::new(Engine::Builtin);
    let forms = json!({ "elicitation": {} });
    let offset = json!({ "utcOffsetSeconds": 20_700 });
    let asked = chart(&mut server, &offset, &forms, None);
    assert_eq!(asked["resultType"], "input_required", "{asked}");
    assert!(asked.get("isError").is_none());
    assert!(asked["_meta"]["io.modelcontextprotocol/serverInfo"].is_object());
    let form = &asked["inputRequests"][KEY];
    assert_eq!(form["method"], "elicitation/create");
    assert_eq!(form["params"]["mode"], "form");
    let schema = &form["params"]["requestedSchema"];
    assert_eq!(
        schema["required"],
        json!(["latitudeDeg", "longitudeDeg"]),
        "{schema}"
    );
    assert_eq!(schema["properties"]["latitudeDeg"]["type"], "number");
    assert!(schema["properties"]["latitudeDeg"].get("format").is_none());
    assert!(
        schema["properties"]["latitudeDeg"]
            .get("description")
            .is_none()
    );
    let message = form["params"]["message"].as_str().unwrap();
    assert!(
        message.starts_with("`chart.found` needs `latitudeDeg` and `longitudeDeg`: "),
        "{message}"
    );

    let place = json!({ KEY: { "action": "accept", "content": { "latitudeDeg": 27.7, "longitudeDeg": 85.3 } } });
    let asked = chart(&mut server, &offset, &forms, Some(place));
    assert_eq!(asked["resultType"], "input_required", "{asked}");
    let schema = &asked["inputRequests"][KEY]["params"]["requestedSchema"];
    assert_eq!(schema["required"], json!(["instant"]), "{schema}");
    assert_eq!(schema["properties"]["instant"]["type"], "number");
    assert!(
        schema["properties"]["instant"]["description"]
            .as_str()
            .unwrap()
            .contains("Julian day")
    );

    let placed = json!({ "latitudeDeg": 27.7, "longitudeDeg": 85.3, "utcOffsetSeconds": 20_700 });
    let instant = json!({ KEY: { "action": "accept", "content": { "instant": 2_460_000.5 } } });
    let answered = chart(&mut server, &placed, &forms, Some(instant));
    assert_eq!(answered["resultType"], "complete", "{answered}");
    assert_eq!(answered["isError"], false, "{answered}");
}

/// **A form left short is asked again; one turned down gets the
/// refusal**, and so does a client that takes no form, or only one by
/// URL.
#[test]
fn a_declined_form_or_a_client_without_forms_gets_the_refusal() {
    let mut server = Server::new(Engine::Builtin);
    let place = json!({ "latitudeDeg": 27.7, "longitudeDeg": 85.3, "utcOffsetSeconds": 20_700 });
    let forms = json!({ "elicitation": { "form": {} } });
    let short = json!({ KEY: { "action": "accept", "content": {} } });
    let again = chart(&mut server, &place, &forms, Some(short));
    assert_eq!(again["resultType"], "input_required", "{again}");
    assert_eq!(
        again["inputRequests"][KEY]["params"]["requestedSchema"]["required"],
        json!(["instant"])
    );
    for declined in ["decline", "cancel"] {
        let turned = json!({ KEY: { "action": declined } });
        let refused = chart(&mut server, &place, &forms, Some(turned));
        assert_eq!(refused["resultType"], "complete");
        assert_eq!(refused["isError"], true);
        assert_eq!(refused["structuredContent"]["field"], "instant");
    }
    for capabilities in [json!({}), json!({ "elicitation": { "url": {} } })] {
        let refused = chart(&mut server, &place, &capabilities, None);
        assert_eq!(refused["isError"], true, "{refused}");
        assert!(refused.get("inputRequests").is_none());
    }
}

/// **Only a field a form can carry and the call lacks is asked for**: a
/// value the call wrote wrongly, a field under one, an object, and a
/// refusal of anything but an argument are each refused as before.
#[test]
fn a_refusal_a_form_cannot_answer_stays_a_refusal() {
    let mut server = Server::new(Engine::Builtin);
    let forms = json!({ "elicitation": {} });
    let wrong = json!({ "instant": "noon", "latitudeDeg": 27.7, "longitudeDeg": 85.3, "utcOffsetSeconds": 0 });
    let refused = chart(&mut server, &wrong, &forms, None);
    assert_eq!(refused["isError"], true, "{refused}");
    let nested = json!({ "instant": 2_460_000.5, "latitudeDeg": 27.7, "longitudeDeg": 85.3,
                         "utcOffsetSeconds": 0, "theme": { "unknown": 1 } });
    let refused = chart(&mut server, &nested, &forms, None);
    assert_eq!(refused["isError"], true, "{refused}");
    // A position with no ephemeris to compute it is not a missing input.
    let mut bare = Server::new(Engine::None);
    let whole = json!({ "instant": 2_460_000.5, "latitudeDeg": 27.7, "longitudeDeg": 85.3, "utcOffsetSeconds": 0 });
    let refused = chart(&mut bare, &whole, &forms, None);
    assert_eq!(refused["isError"], true, "{refused}");
    // A program's own tool is not a record, so it is not asked for.
    let mut own = Server::new(Engine::None)
        .with_tool(greeter("acme.greet"))
        .unwrap();
    let reply = ask(
        &mut own,
        "tools/call",
        json!({ "name": "acme.greet", "arguments": {} }),
        &forms,
    );
    assert_eq!(reply["result"]["isError"], true, "{reply}");
}

fn greeter(name: &str) -> Tool {
    Tool::new(
        name,
        "Greets a querent by name.",
        json!({ "type": "object", "properties": { "name": { "type": "string" } }, "required": ["name"] }),
        |call| {
            let name: Option<String> = call.argument("name")?;
            name.map(|name| json!({ "greeting": name }))
                .ok_or_else(|| teistro::Error::invalid_arg("a name").with_field("arguments.name"))
        },
    )
}

/// **The tool list is one order** (the revision's SHOULD, for a host's
/// prompt cache): name order, and the same bytes whichever order a
/// program registered its tools in and however often it is asked.
#[test]
fn the_tool_list_is_one_order_whatever_order_tools_arrive_in() {
    let listed = |names: [&str; 3]| {
        let mut server = Server::new(Engine::None);
        for name in names {
            server = server.with_tool(greeter(name)).unwrap();
        }
        let first = ask(&mut server, "tools/list", json!({}), &json!({}));
        let again = ask(&mut server, "tools/list", json!({}), &json!({}));
        assert_eq!(first, again);
        first["result"]["tools"].clone()
    };
    let one = listed(["acme.c", "acme.a", "acme.b"]);
    let other = listed(["acme.b", "acme.c", "acme.a"]);
    assert_eq!(one.to_string(), other.to_string());
    let names: Vec<&str> = one
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|tool| tool["name"].as_str())
        .collect();
    assert!(names.windows(2).all(|pair| pair[0] < pair[1]), "{names:?}");
}

/// **A request's trace context changes nothing** (SEP-414): the SDK has
/// no spans to join, so `traceparent`, `tracestate` and `baggage` in
/// `_meta` are accepted and the answer is the one without them.
#[test]
fn a_trace_context_is_accepted_and_changes_nothing() {
    let mut server = Server::new(Engine::None);
    let call = json!({ "name": "numerology.profile", "arguments": { "request":
        { "name": "Asha", "date": { "year": 1990, "month": 5, "day": 17 } } } });
    let plain = ask(&mut server, "tools/call", call.clone(), &json!({}));
    let mut traced = call;
    let mut context = meta(&json!({}));
    context["traceparent"] = json!("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01");
    context["tracestate"] = json!("teistro=1");
    context["baggage"] = json!("tenant=a");
    traced["_meta"] = context;
    let answered = ask(&mut server, "tools/call", traced, &json!({}));
    assert_eq!(plain, answered);
}
