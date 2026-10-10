//! Resources and completions (`03-design/mcp-server.md` §7, P2 and P3):
//! every listed resource reads as its list says, each is the same answer
//! its tool gives, every value a completion offers reads, and what the
//! server does not have is refused under each revision's code.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index the replies they expect"
)]

use serde_json::{Value, json};
use teistro_core::catalogue::Kind;
use teistro_mcp::{Engine, Server};

const MODERN: &str = "2026-07-28";

/// The whole reply to `method`, asked under the stateless revision.
fn ask(server: &mut Server, method: &str, mut params: Value) -> Value {
    params["_meta"] = json!({ "io.modelcontextprotocol/protocolVersion": MODERN });
    let message = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
    serde_json::from_str(&server.handle(&message.to_string()).unwrap()).unwrap()
}

fn read(server: &mut Server, uri: &str) -> Value {
    ask(server, "resources/read", json!({ "uri": uri }))
}

/// A resource's text, read as JSON.
fn read_json(server: &mut Server, uri: &str) -> Value {
    let reply = read(server, uri);
    let text = reply["result"]["contents"][0]["text"]
        .as_str()
        .unwrap_or_else(|| panic!("{uri}: {reply}"));
    serde_json::from_str(text).unwrap_or_else(|why| panic!("{uri} is not JSON: {why}"))
}

fn call(server: &mut Server, name: &str, arguments: &Value) -> Value {
    let reply = ask(
        server,
        "tools/call",
        json!({ "name": name, "arguments": arguments }),
    );
    reply["result"]["structuredContent"].clone()
}

/// **A list says what a read answers**: every listed resource reads, once,
/// as the type and size the list gives, and its text is JSON; the list
/// and every read may be kept a day.
#[test]
fn every_listed_resource_reads_as_its_list_says() {
    let mut server = Server::new(Engine::None);
    let listed = ask(&mut server, "resources/list", json!({}));
    assert_eq!(listed["result"]["ttlMs"], 86_400_000);
    assert_eq!(listed["result"]["cacheScope"], "public");
    let resources = listed["result"]["resources"].as_array().unwrap();
    let mut uris: Vec<&str> = resources
        .iter()
        .map(|resource| resource["uri"].as_str().unwrap())
        .collect();
    uris.dedup();
    assert_eq!(uris.len(), resources.len(), "a URI is listed twice");
    for resource in resources {
        let uri = resource["uri"].as_str().unwrap();
        let reply = read(&mut server, uri);
        let result = &reply["result"];
        assert_eq!(result["cacheScope"], "public", "{uri}");
        let contents = result["contents"].as_array().unwrap();
        assert_eq!(contents.len(), 1, "{uri}");
        assert_eq!(contents[0]["uri"], uri);
        assert_eq!(contents[0]["mimeType"], resource["mimeType"], "{uri}");
        let text = contents[0]["text"].as_str().unwrap();
        assert_eq!(resource["size"], text.len(), "{uri}");
        let parsed: Value = serde_json::from_str(text).unwrap();
        assert!(parsed.get("error").is_none(), "{uri}: {parsed}");
    }
}

/// **The catalogue served is the catalogue compiled**: a resource for
/// every kind the binary knows, under its name and number, and none it
/// does not.
#[test]
fn every_catalogue_kind_is_a_resource_under_its_name_and_number() {
    let mut server = Server::new(Engine::None);
    let index = read_json(&mut server, "teistro://catalogue");
    let kinds: Vec<(String, u64)> = index["kinds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|kind| {
            (
                kind["kind"].as_str().unwrap().to_owned(),
                kind["number"].as_u64().unwrap(),
            )
        })
        .collect();
    for kind in Kind::ALL {
        let number = u64::from(kind as u8);
        assert!(
            kinds.contains(&(kind.name().to_owned(), number)),
            "{} ({number}) is not served",
            kind.name()
        );
        let whole = read_json(&mut server, &format!("teistro://catalogue/{}", kind.name()));
        assert_eq!(whole["kind"], kind.name());
        assert_eq!(whole["number"], number);
    }
    for (name, _) in &kinds {
        assert!(
            Kind::from_name(name).is_some(),
            "{name} is served and not compiled"
        );
    }
}

/// **A resource is its tool's answer**: the profiles and the settings
/// schema as `settings.describe` answers them, a tool's schemas as
/// `schema.describe` does, and the document schema as the SDK carries it.
#[test]
fn each_resource_is_the_answer_its_tool_gives() {
    let mut server = Server::new(Engine::None);
    let described = call(&mut server, "settings.describe", &json!({}));
    assert_eq!(
        read_json(&mut server, "teistro://settings/schema"),
        described["settings"]
    );
    for profile in described["profiles"].as_array().unwrap() {
        let uri = format!("teistro://profiles/{}", profile["id"].as_str().unwrap());
        assert_eq!(&read_json(&mut server, &uri), profile, "{uri}");
    }
    let document: Value = serde_json::from_str(teistro::schema::DOCUMENT).unwrap();
    assert_eq!(
        read_json(&mut server, "teistro://document/schema"),
        document
    );
    for record in teistro::records::records() {
        let uri = format!("teistro://tools/{}/schema", record.name);
        let described = call(
            &mut server,
            "schema.describe",
            &json!({ "tool": record.name }),
        );
        assert_eq!(read_json(&mut server, &uri), described, "{uri}");
    }
}

/// **What the server does not have is refused, not empty**: `-32602`
/// with the URI under the stateless revision, `-32002` under the
/// handshake's, and a cursor the server never gave on every list.
#[test]
fn a_resource_the_server_lacks_is_refused_under_each_revision() {
    let mut server = Server::new(Engine::None);
    for uri in [
        "teistro://nothing",
        "teistro://catalogue/nothing",
        "teistro://catalogue/../settings/schema",
        "teistro://profiles/",
        "teistro://tools/chart.nothing/schema",
        "teistro://tools/settings.describe/schema",
        "file:///etc/passwd",
        "settings/schema",
    ] {
        let reply = read(&mut server, uri);
        assert_eq!(reply["error"]["code"], -32_602, "{uri}: {reply}");
        assert_eq!(reply["error"]["data"]["uri"], uri);
    }
    let reply = ask(&mut server, "resources/read", json!({}));
    assert_eq!(reply["error"]["code"], -32_602, "{reply}");
    for method in ["resources/list", "resources/templates/list", "tools/list"] {
        let reply = ask(&mut server, method, json!({ "cursor": "2" }));
        assert_eq!(reply["error"]["code"], -32_602, "{method}: {reply}");
    }

    let mut legacy = Server::new(Engine::None);
    legacy
        .handle(r#"{"jsonrpc":"2.0","id":0,"method":"initialize","params":{}}"#)
        .unwrap();
    let reply: Value = serde_json::from_str(
        &legacy
            .handle(r#"{"jsonrpc":"2.0","id":1,"method":"resources/read","params":{"uri":"teistro://nothing"}}"#)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(reply["error"]["code"], -32_002, "{reply}");
    let reply: Value = serde_json::from_str(
        &legacy
            .handle(r#"{"jsonrpc":"2.0","id":2,"method":"resources/list","params":{}}"#)
            .unwrap(),
    )
    .unwrap();
    assert!(reply["result"]["resources"].is_array(), "{reply}");
    assert!(reply["result"].get("ttlMs").is_none(), "{reply}");
}

/// **Every value a completion offers reads**: each template's argument,
/// completed from nothing, offers every value it takes, and each one put
/// in the template is a resource that reads.
#[test]
fn every_completed_value_reads_through_its_template() {
    let mut server = Server::new(Engine::None);
    let templates = ask(&mut server, "resources/templates/list", json!({}));
    let templates = templates["result"]["resourceTemplates"].as_array().unwrap();
    for template in templates {
        let uri = template["uriTemplate"].as_str().unwrap();
        let open = uri.find('{').unwrap();
        let close = uri.find('}').unwrap();
        let argument = &uri[open + 1..close];
        let reply = ask(
            &mut server,
            "completion/complete",
            json!({ "ref": { "type": "ref/resource", "uri": uri },
                    "argument": { "name": argument, "value": "" } }),
        );
        let completion = &reply["result"]["completion"];
        let values = completion["values"].as_array().unwrap();
        assert!(!values.is_empty(), "{uri}: {reply}");
        assert_eq!(
            completion["total"],
            values.len(),
            "{uri} has more than a page"
        );
        for value in values {
            let filled = format!(
                "{}{}{}",
                &uri[..open],
                value.as_str().unwrap(),
                &uri[close + 1..]
            );
            let reply = read(&mut server, &filled);
            assert!(reply.get("result").is_some(), "{filled}: {reply}");
        }
    }
}

/// **A completion ranks what was typed**: what it begins first, then what
/// it is inside, then its letters in order, and nothing else; case is
/// ignored.
#[test]
fn a_completion_ranks_the_typed_prefix_first() {
    let mut server = Server::new(Engine::None);
    let mut complete = |uri: &str, argument: &str, typed: &str| -> Vec<String> {
        let reply = ask(
            &mut server,
            "completion/complete",
            json!({ "ref": { "type": "ref/resource", "uri": uri },
                    "argument": { "name": argument, "value": typed } }),
        );
        reply["result"]["completion"]["values"]
            .as_array()
            .unwrap_or_else(|| panic!("{reply}"))
            .iter()
            .map(|value| value.as_str().unwrap().to_owned())
            .collect()
    };
    let kinds = complete("teistro://catalogue/{kind}", "kind", "NAK");
    assert_eq!(
        kinds.first().map(String::as_str),
        Some("nakshatra"),
        "{kinds:?}"
    );
    let avasthas = complete("teistro://catalogue/{kind}", "kind", "avastha");
    assert!(
        avasthas.iter().all(|kind| kind.starts_with("avastha")),
        "{avasthas:?}"
    );
    let tools = complete("teistro://tools/{tool}/schema", "tool", "found");
    assert_eq!(
        tools.first().map(String::as_str),
        Some("chart.found"),
        "{tools:?}"
    );
    let scattered = complete("teistro://tools/{tool}/schema", "tool", "chfd");
    assert!(
        scattered.contains(&String::from("chart.found")),
        "{scattered:?}"
    );
    assert_eq!(
        complete("teistro://catalogue/{kind}", "kind", "zzzz"),
        Vec::<String>::new()
    );
}

/// A completion request names the field it gets wrong.
#[test]
fn a_completion_is_refused_by_the_field_it_gets_wrong() {
    let mut server = Server::new(Engine::None);
    for (params, field) in [
        (json!({ "argument": { "name": "kind" } }), "ref"),
        (
            json!({ "ref": { "type": "ref/resource", "uri": "teistro://catalogue/{kind}" } }),
            "argument",
        ),
        (
            json!({ "ref": { "type": "ref/resource", "uri": "teistro://nothing/{x}" },
                 "argument": { "name": "x", "value": "" } }),
            "ref.uri",
        ),
        (
            json!({ "ref": { "type": "ref/resource", "uri": "teistro://catalogue/{kind}" },
                 "argument": { "name": "id", "value": "" } }),
            "argument.name",
        ),
        (
            json!({ "ref": { "type": "ref/resource", "uri": "teistro://catalogue/{kind}" },
                 "argument": { "name": "kind", "value": 3 } }),
            "argument.value",
        ),
        (
            json!({ "ref": { "type": "ref/nothing" }, "argument": { "name": "x" } }),
            "ref.type",
        ),
    ] {
        let reply = ask(&mut server, "completion/complete", params.clone());
        assert_eq!(reply["error"]["code"], -32_602, "{params}: {reply}");
        assert_eq!(reply["error"]["data"]["field"], field, "{params}: {reply}");
    }
}

/// `server/discover` offers what the server answers.
#[test]
fn discovery_offers_resources_and_completions() {
    let mut server = Server::new(Engine::None);
    let reply = ask(&mut server, "server/discover", json!({}));
    let capabilities = &reply["result"]["capabilities"];
    assert!(capabilities["resources"].is_object(), "{reply}");
    assert!(capabilities["completions"].is_object(), "{reply}");
}
