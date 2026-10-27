//! Long work as a task (`03-design/mcp-server.md` §7, P7): the
//! `io.modelcontextprotocol/tasks` extension, in process. A client that
//! declares it gets a call it outlasts as a task handle, which `tasks/get`
//! polls to the same answer the call gives inline and `tasks/cancel`
//! stops; a client that does not, or a call quicker than the server's
//! patience, is answered as before. Refusals carry the extension's codes.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index the replies they expect"
)]

use std::time::{Duration, Instant};

use serde_json::{Value, json};
use teistro_mcp::{Engine, Server};

const EXTENSION: &str = "io.modelcontextprotocol/tasks";

fn meta(tasks: bool) -> Value {
    let capabilities = if tasks {
        json!({ "extensions": { EXTENSION: {} } })
    } else {
        json!({})
    };
    json!({ "io.modelcontextprotocol/protocolVersion": "2026-07-28",
        "io.modelcontextprotocol/clientCapabilities": capabilities })
}

fn ask(server: &mut Server, method: &str, params: Value) -> Value {
    let mut message = json!({ "jsonrpc": "2.0", "id": 1, "method": method });
    message["params"] = params;
    let reply = server
        .handle(&message.to_string())
        .expect("a request is answered");
    serde_json::from_str(&reply).unwrap()
}

/// A quarter of days at Kathmandu: seconds of work in a debug build.
fn quarter(tasks: bool) -> Value {
    json!({ "_meta": meta(tasks), "name": "almanac.days", "arguments": { "request": {
        "first": { "year": 2026, "month": 1, "day": 1 },
        "last": { "year": 2026, "month": 3, "day": 31 },
        "latitudeDeg": 27.7, "longitudeDeg": 85.3, "utcOffsetSeconds": 20_700 } } })
}

fn days(tasks: bool) -> Value {
    json!({ "_meta": meta(tasks), "name": "almanac.days", "arguments": { "request": {
        "first": { "year": 2026, "month": 1, "day": 1 },
        "last": { "year": 2026, "month": 1, "day": 2 },
        "latitudeDeg": 27.7, "longitudeDeg": 85.3, "utcOffsetSeconds": 20_700 } } })
}

fn task(server: &mut Server, method: &str, id: &str, tasks: bool) -> Value {
    ask(
        server,
        method,
        json!({ "_meta": meta(tasks), "taskId": id }),
    )
}

/// Polls until the task ends, as the extension asks, within a minute.
fn ended(server: &mut Server, id: &str) -> Value {
    let started = Instant::now();
    loop {
        let polled = task(server, "tasks/get", id, true);
        let result = &polled["result"];
        assert_eq!(result["resultType"], "complete", "{polled}");
        assert_eq!(result["taskId"], id);
        if result["status"] != "working" {
            return result.clone();
        }
        assert!(result["ttlMs"].is_null(), "a working task is unlimited");
        assert!(
            started.elapsed() < Duration::from_secs(60),
            "the task never ended"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// **Discovery names the extension** (P7) when the server makes tasks,
/// and not when its operator turned them off.
#[test]
fn discovery_names_the_extension_only_when_tasks_are_made() {
    let mut server = Server::new(Engine::Builtin);
    let found = ask(
        &mut server,
        "server/discover",
        json!({ "_meta": meta(false) }),
    );
    assert_eq!(
        found["result"]["capabilities"]["extensions"][EXTENSION],
        json!({})
    );
    let mut server = Server::new(Engine::Builtin).with_task_patience(None);
    let found = ask(
        &mut server,
        "server/discover",
        json!({ "_meta": meta(false) }),
    );
    assert!(found["result"]["capabilities"].get("extensions").is_none());
    let refused = task(&mut server, "tasks/get", "0", true);
    assert_eq!(refused["error"]["code"], -32_601, "{refused}");
}

/// **A task answers what the call does** (P7): outlasting no patience at
/// all, the call is a handle with the extension's fields, and polled to
/// its end it carries the call's own result; a client not declaring the
/// extension is answered inline by the same server.
#[test]
fn a_call_outlasting_the_patience_is_a_task_polled_to_the_inline_answer() {
    let mut server = Server::new(Engine::Builtin).with_task_patience(Some(Duration::ZERO));
    let inline = ask(&mut server, "tools/call", days(false));
    assert_eq!(inline["result"]["resultType"], "complete", "{inline}");

    let created = ask(&mut server, "tools/call", days(true));
    let handle = &created["result"];
    assert_eq!(handle["resultType"], "task", "{created}");
    let id = handle["taskId"].as_str().unwrap().to_owned();
    assert_eq!(id.len(), 32);
    assert!(id.bytes().all(|byte| byte.is_ascii_hexdigit()));
    for field in ["status", "createdAt", "lastUpdatedAt", "pollIntervalMs"] {
        assert!(handle.get(field).is_some(), "{field} in {handle}");
    }
    assert!(handle["_meta"]["io.modelcontextprotocol/serverInfo"].is_object());

    let done = ended(&mut server, &id);
    assert_eq!(done["status"], "completed", "{done}");
    assert_eq!(
        done["result"]["structuredContent"],
        inline["result"]["structuredContent"]
    );
    assert_eq!(done["result"]["isError"], false);
    assert!(done["ttlMs"].as_u64().unwrap() >= 3_600_000);
    // Kept, and read again the same.
    assert_eq!(ended(&mut server, &id), done);
}

/// **A quick call stays a call** (P7): within the patience, a declaring
/// client is answered as any other.
#[test]
fn a_call_within_the_patience_is_answered_inline() {
    let mut server = Server::new(Engine::Builtin).with_task_patience(Some(Duration::from_secs(60)));
    let answered = ask(&mut server, "tools/call", days(true));
    assert_eq!(answered["result"]["resultType"], "complete", "{answered}");
    assert_eq!(answered["result"]["isError"], false);
}

/// **A refusal is a completed task, a fault a failed one** (P7): a tool
/// error is the call's result, `isError` and all, and a protocol error
/// is the task's `error`.
#[test]
fn a_tool_error_completes_and_a_protocol_error_fails() {
    let mut server = Server::new(Engine::Builtin).with_task_patience(Some(Duration::ZERO));
    let mut wrong = days(true);
    wrong["arguments"]["request"]["latitudeDeg"] = json!(91);
    let created = ask(&mut server, "tools/call", wrong);
    let done = ended(&mut server, created["result"]["taskId"].as_str().unwrap());
    assert_eq!(done["status"], "completed", "{done}");
    assert_eq!(done["result"]["isError"], true);

    let mut unknown = days(true);
    unknown["name"] = json!("no.such.tool");
    let created = ask(&mut server, "tools/call", unknown);
    let done = ended(&mut server, created["result"]["taskId"].as_str().unwrap());
    assert_eq!(done["status"], "failed", "{done}");
    assert_eq!(done["error"]["code"], -32_602);
}

/// **A task is stopped** (P7): `tasks/cancel` is acknowledged at once and
/// the working call stops at its next request of the engine.
#[test]
fn a_cancelled_task_ends_cancelled() {
    let mut server = Server::new(Engine::Builtin).with_task_patience(Some(Duration::ZERO));
    let created = ask(&mut server, "tools/call", quarter(true));
    let id = created["result"]["taskId"].as_str().unwrap().to_owned();
    let acknowledged = task(&mut server, "tasks/cancel", &id, true);
    assert_eq!(
        acknowledged["result"]["resultType"], "complete",
        "{acknowledged}"
    );
    assert_eq!(ended(&mut server, &id)["status"], "cancelled");
}

/// **The extension's refusals** (P7): an unknown task is `-32602` naming
/// `taskId`, for every method; a client not declaring the extension is
/// `-32021` naming it; `tasks/update` has nothing to update and is
/// acknowledged.
#[test]
fn the_task_methods_refuse_as_the_extension_says() {
    let mut server = Server::new(Engine::Builtin).with_task_patience(Some(Duration::ZERO));
    for method in ["tasks/get", "tasks/update", "tasks/cancel"] {
        let unknown = task(&mut server, method, "feedface", true);
        assert_eq!(unknown["error"]["code"], -32_602, "{method}: {unknown}");
        assert_eq!(unknown["error"]["data"]["field"], "taskId");
        let undeclared = task(&mut server, method, "feedface", false);
        assert_eq!(
            undeclared["error"]["code"], -32_021,
            "{method}: {undeclared}"
        );
        assert_eq!(
            undeclared["error"]["data"]["requiredCapabilities"]["extensions"][EXTENSION],
            json!({})
        );
    }
    let created = ask(&mut server, "tools/call", days(true));
    let id = created["result"]["taskId"].as_str().unwrap().to_owned();
    let updated = task(&mut server, "tasks/update", &id, true);
    assert_eq!(updated["result"]["resultType"], "complete", "{updated}");
    ended(&mut server, &id);
}

/// **A long session keeps a bounded table** (P7): a server making a task
/// of every call, three hundred of them ended one after another, keeps
/// the newest 256 readable and has let the oldest go, so a client that
/// never stops calling does not grow the server without bound.
#[test]
fn a_long_session_keeps_the_newest_ended_tasks_and_lets_the_oldest_go() {
    const CALLS: usize = 300;
    let mut server = Server::new(Engine::Builtin).with_task_patience(Some(Duration::ZERO));
    let convert = json!({ "_meta": meta(true), "name": "time.convert", "arguments": { "request": {
        "jd": 2_451_545.0, "from": "UT1", "to": "TT" } } });
    let mut ids = Vec::with_capacity(CALLS);
    for _ in 0..CALLS {
        let created = ask(&mut server, "tools/call", convert.clone());
        let id = created["result"]["taskId"].as_str().unwrap().to_owned();
        assert_eq!(ended(&mut server, &id)["status"], "completed");
        ids.push(id);
    }
    let oldest = task(&mut server, "tasks/get", &ids[0], true);
    assert!(
        oldest.get("error").is_some(),
        "the oldest was let go: {oldest}"
    );
    let newest = ids.len() - 256;
    for id in &ids[newest..] {
        assert_eq!(
            ended(&mut server, id)["status"],
            "completed",
            "{id} is kept"
        );
    }
}
