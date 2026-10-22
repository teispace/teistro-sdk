//! Streamable HTTP (`03-design/mcp-server.md` §7, P8), through the
//! binary on a loopback port: a POST answers as JSON, or as an event
//! stream when it carries a progress token; the headers the revision
//! requires are checked against the body; a foreign origin, another
//! method and another path are refused; and closing the connection
//! cancels the call it carries. The transport is served in this
//! process; the binary is started once, to show its options reach it.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index the replies they expect"
)]

use std::fmt::Write as _;
use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use teistro_mcp::http::{self, Http, MakeServer};
use teistro_mcp::{Engine, Limits, Server};

const MODERN: &str = "2026-07-28";

/// The transport serving on a loopback port of its own, in this process
/// so what it runs is measured; the thread lives until the tests end.
struct Served {
    address: String,
}

impl Served {
    /// Serves `engine` as the binary would with `http`, the message bound
    /// given to the server's limits as `--max-message-bytes` gives it.
    fn start(engine: Engine, http: Http) -> Served {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap().to_string();
        let limits = Limits {
            message_bytes: http.message_bytes,
            ..Limits::default()
        };
        let make: Arc<MakeServer> = Arc::new(move || Ok(Server::new(engine).with_limits(limits)));
        std::thread::spawn(move || http::serve(&listener, &http, &make));
        Served { address }
    }

    /// The shipped transport, every call a task for a client declaring
    /// the extension.
    fn tasking() -> Served {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap().to_string();
        let make: Arc<MakeServer> =
            Arc::new(|| Ok(Server::new(Engine::Builtin).with_task_patience(Some(Duration::ZERO))));
        std::thread::spawn(move || http::serve(&listener, &Http::default(), &make));
        Served { address }
    }

    /// The shipped transport over no ephemeris.
    fn plain() -> Served {
        Served::start(Engine::None, Http::default())
    }

    fn connect(&self) -> TcpStream {
        TcpStream::connect(&self.address).unwrap()
    }
}

/// A response's status, headers (names lowercased) and body.
struct Response {
    status: u16,
    headers: Vec<(String, String)>,
    body: String,
}

impl Response {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    fn json(&self) -> Value {
        serde_json::from_str(&self.body).unwrap_or_else(|_| panic!("not JSON: {}", self.body))
    }
}

fn send(stream: &mut TcpStream, method: &str, path: &str, headers: &[(&str, &str)], body: &str) {
    let mut request = format!(
        "{method} {path} HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n",
        body.len()
    );
    for (name, value) in headers {
        let _ = write!(request, "{name}: {value}\r\n");
    }
    request.push_str("\r\n");
    request.push_str(body);
    stream.write_all(request.as_bytes()).unwrap();
}

fn read(stream: TcpStream) -> Response {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    let status = line.split(' ').nth(1).unwrap().parse().unwrap();
    let mut headers = Vec::new();
    loop {
        line.clear();
        reader.read_line(&mut line).unwrap();
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        let (name, value) = line.split_once(':').unwrap();
        headers.push((name.to_ascii_lowercase(), value.trim().to_owned()));
    }
    let mut body = String::new();
    reader.read_to_string(&mut body).unwrap();
    Response {
        status,
        headers,
        body,
    }
}

fn meta() -> Value {
    json!({ "io.modelcontextprotocol/protocolVersion": MODERN,
            "io.modelcontextprotocol/clientCapabilities": {} })
}

/// The headers the revision requires for `message`.
fn required(message: &Value) -> Vec<(&'static str, String)> {
    let mut headers = vec![
        ("Content-Type", String::from("application/json")),
        (
            "Accept",
            String::from("application/json, text/event-stream"),
        ),
        ("MCP-Protocol-Version", String::from(MODERN)),
        ("Mcp-Method", message["method"].as_str().unwrap().to_owned()),
    ];
    if let Some(name) = message["params"]["name"].as_str() {
        headers.push(("Mcp-Name", name.to_owned()));
    }
    if let Some(task) = message["params"]["taskId"].as_str() {
        headers.push(("Mcp-Name", task.to_owned()));
    }
    headers
}

/// POSTs `message` with the required headers, changed by `change`.
fn post_with(served: &Served, message: &Value, change: &[(&str, Option<&str>)]) -> Response {
    let mut headers: Vec<(&str, String)> = required(message);
    for (name, value) in change {
        headers.retain(|(key, _)| key != name);
        if let Some(value) = value {
            headers.push((name, (*value).to_owned()));
        }
    }
    let headers: Vec<(&str, &str)> = headers.iter().map(|(k, v)| (*k, v.as_str())).collect();
    let mut stream = served.connect();
    send(&mut stream, "POST", "/mcp", &headers, &message.to_string());
    read(stream)
}

fn post(served: &Served, message: &Value) -> Response {
    post_with(served, message, &[])
}

fn numerology(id: u64) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "method": "tools/call", "params": { "_meta": meta(),
        "name": "numerology.profile", "arguments": { "request":
            { "name": "Henry Elder", "date": { "year": 1872, "month": 1, "day": 17 } } } } })
}

/// A year of days at Kathmandu, about the most one call may ask (366):
/// long past the first progress notification (`EVERY`, 250 ms), so a
/// token hears progress and a cancel lands while the call computes. A
/// quarter once took seconds; the cheaper days made it end inside the
/// window under load.
fn days(id: u64, token: Option<&str>) -> Value {
    let mut meta = meta();
    if let Some(token) = token {
        meta["progressToken"] = json!(token);
    }
    json!({ "jsonrpc": "2.0", "id": id, "method": "tools/call", "params": {
        "_meta": meta, "name": "almanac.days", "arguments": { "request": {
            "first": { "year": 2026, "month": 1, "day": 1 },
            "last": { "year": 2026, "month": 12, "day": 31 },
            "latitudeDeg": 27.7, "longitudeDeg": 85.3, "utcOffsetSeconds": 20_700 } } } })
}

/// **A POST is one message** (P8): a call answers as JSON naming its
/// server, and a notification is accepted with no body.
#[test]
fn a_post_answers_as_json_and_a_notification_is_accepted() {
    let served = Served::plain();
    let answer = post(&served, &numerology(1));
    assert_eq!(answer.status, 200, "{}", answer.body);
    assert_eq!(answer.header("content-type"), Some("application/json"));
    let reply = answer.json();
    assert_eq!(reply["id"], 1);
    assert_eq!(reply["result"]["isError"], false, "{reply}");
    assert_eq!(
        reply["result"]["_meta"]["io.modelcontextprotocol/serverInfo"]["name"],
        "teistro-mcp"
    );
    let notification = json!({ "jsonrpc": "2.0", "method": "notifications/initialized",
                                "params": { "_meta": meta() } });
    let accepted = post(&served, &notification);
    assert_eq!(accepted.status, 202);
    assert_eq!(accepted.body, "");
    let unknown =
        json!({ "jsonrpc": "2.0", "id": 2, "method": "no/such", "params": { "_meta": meta() } });
    let unknown = post(&served, &unknown);
    assert_eq!(unknown.status, 404);
    assert_eq!(unknown.json()["error"]["code"], -32_601);
}

/// **The headers say what the body does** (P8): a revision, method or
/// name missing or disagreeing is `-32020` under 400, before anything
/// is computed; a revision the transport does not speak is `-32022`.
#[test]
fn the_required_headers_are_checked_against_the_body() {
    let served = Served::plain();
    for (change, header) in [
        (("MCP-Protocol-Version", None), "MCP-Protocol-Version"),
        (("Mcp-Method", Some("tools/list")), "Mcp-Method"),
        (("Mcp-Method", None), "Mcp-Method"),
        (("Mcp-Name", Some("chart.found")), "Mcp-Name"),
        (("Mcp-Name", None), "Mcp-Name"),
    ] {
        let refused = post_with(&served, &numerology(3), &[change]);
        assert_eq!(refused.status, 400, "{change:?}: {}", refused.body);
        let reply = refused.json();
        assert_eq!(reply["id"], 3);
        assert_eq!(reply["error"]["code"], -32_020, "{change:?}: {reply}");
        assert_eq!(reply["error"]["data"]["header"], header);
    }
    let legacy = post_with(
        &served,
        &numerology(4),
        &[("MCP-Protocol-Version", Some("2025-11-25"))],
    );
    assert_eq!(legacy.status, 400);
    assert_eq!(legacy.json()["error"]["code"], -32_022);
    let mut disagreeing = numerology(5);
    disagreeing["params"]["_meta"]["io.modelcontextprotocol/protocolVersion"] = json!("2025-11-25");
    let refused = post(&served, &disagreeing);
    assert_eq!(refused.json()["error"]["code"], -32_020, "{}", refused.body);
}

/// **Only the endpoint answers, and only to a page it trusts** (P8): a
/// foreign `Origin` is 403, a loopback or allowed one answers, another
/// method is 405, another path 404, another body type 415 and a body
/// past the bound 413.
#[test]
fn a_foreign_origin_another_method_or_path_is_refused() {
    let served = Served::start(
        Engine::None,
        Http {
            origins: vec![String::from("https://app.example")],
            message_bytes: Some(4096),
            ..Http::default()
        },
    );
    let call = numerology(6);
    for origin in ["https://evil.example", "null"] {
        let refused = post_with(&served, &call, &[("Origin", Some(origin))]);
        assert_eq!(refused.status, 403, "{origin}");
    }
    for origin in ["http://localhost:5173", "https://app.example"] {
        let answered = post_with(&served, &call, &[("Origin", Some(origin))]);
        assert_eq!(answered.status, 200, "{origin}: {}", answered.body);
    }
    for method in ["GET", "DELETE"] {
        let mut stream = served.connect();
        send(&mut stream, method, "/mcp", &[], "");
        let refused = read(stream);
        assert_eq!(refused.status, 405, "{method}");
        assert_eq!(refused.header("allow"), Some("POST"));
    }
    let mut stream = served.connect();
    send(&mut stream, "POST", "/elsewhere", &[], "");
    assert_eq!(read(stream).status, 404);
    let text = post_with(&served, &call, &[("Content-Type", Some("text/plain"))]);
    assert_eq!(text.status, 415);
    let mut long = numerology(7);
    long["params"]["padding"] = json!("a".repeat(8_192));
    let long = post(&served, &long);
    assert_eq!(long.status, 413);
    assert!(long.body.contains("--max-message-bytes"), "{}", long.body);
}

/// **Progress is an event stream** (P8): a call carrying a token is
/// answered as events, increasing progress first and its answer last,
/// unbuffered by a proxy.
#[test]
fn a_call_with_a_token_is_answered_as_an_event_stream() {
    let served = Served::start(Engine::Builtin, Http::default());
    let answer = post(&served, &days(8, Some("quarter")));
    assert_eq!(answer.status, 200);
    assert_eq!(answer.header("content-type"), Some("text/event-stream"));
    assert_eq!(answer.header("x-accel-buffering"), Some("no"));
    let events: Vec<Value> = answer
        .body
        .lines()
        .filter_map(|line| line.strip_prefix("data: "))
        .map(|data| serde_json::from_str(data).unwrap())
        .collect();
    let (last, progress) = events.split_last().unwrap();
    assert_eq!(last["id"], 8, "{last}");
    assert_eq!(last["result"]["isError"], false);
    assert!(!progress.is_empty(), "no progress before the answer");
    let counts: Vec<f64> = progress
        .iter()
        .map(|event| {
            assert_eq!(event["method"], "notifications/progress");
            assert_eq!(event["params"]["progressToken"], "quarter");
            event["params"]["progress"].as_f64().unwrap()
        })
        .collect();
    assert!(
        counts.windows(2).all(|pair| pair[0] < pair[1]),
        "{counts:?}"
    );
}

/// **Closing the stream cancels** (P8): a call whose connection closes
/// once its progress shows it computing stops, and the one worker is
/// free for the next call sooner than a whole call takes.
#[test]
fn closing_the_connection_cancels_its_call() {
    let served = Served::start(
        Engine::Builtin,
        Http {
            workers: 1,
            ..Http::default()
        },
    );
    let message = days(9, Some("cancel-me"));
    let headers = required(&message);
    let headers: Vec<(&str, &str)> = headers.iter().map(|(k, v)| (*k, v.as_str())).collect();
    let mut stream = served.connect();
    send(&mut stream, "POST", "/mcp", &headers, &message.to_string());
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    while !line.starts_with("data: ") {
        line.clear();
        assert!(reader.read_line(&mut line).unwrap() > 0, "the stream ended");
    }
    // Timed from the close: the cold work before the first progress is
    // not what a cancel saves.
    let started = Instant::now();
    drop(reader);
    let next = post(&served, &numerology(10));
    let stopped = started.elapsed();
    assert_eq!(next.json()["id"], 10);
    let whole = Instant::now();
    let answered = post(&served, &days(11, None));
    let whole = whole.elapsed();
    assert_eq!(answered.json()["id"], 11);
    assert!(
        stopped < whole,
        "the cancelled call ran on: stopped after {stopped:?}, a whole call took {whole:?}"
    );
}

/// **A subscription is a stream** (P8): its acknowledgment and its
/// graceful closure are events of one response.
#[test]
fn a_subscription_is_acknowledged_and_closed_on_one_stream() {
    let served = Served::plain();
    let listen = json!({ "jsonrpc": "2.0", "id": 12, "method": "subscriptions/listen",
                         "params": { "_meta": meta(), "notifications": { "toolsListChanged": true } } });
    let answer = post(&served, &listen);
    assert_eq!(answer.header("content-type"), Some("text/event-stream"));
    let events: Vec<Value> = answer
        .body
        .lines()
        .filter_map(|line| line.strip_prefix("data: "))
        .map(|data| serde_json::from_str(data).unwrap())
        .collect();
    assert_eq!(events.len(), 2, "{events:?}");
    assert_eq!(
        events[0]["method"],
        "notifications/subscriptions/acknowledged"
    );
    assert_eq!(events[1]["id"], 12);
    assert_eq!(events[1]["result"]["resultType"], "complete");
}

/// **A peer past its rate waits** (P8): a burst past twice the rate is
/// `429` with a `Retry-After`.
#[test]
fn a_peer_past_its_rate_is_told_to_wait() {
    let served = Served::start(
        Engine::None,
        Http {
            rate: Some(1),
            ..Http::default()
        },
    );
    let ping =
        json!({ "jsonrpc": "2.0", "id": 13, "method": "ping", "params": { "_meta": meta() } });
    let statuses: Vec<u16> = (0..3).map(|_| post(&served, &ping).status).collect();
    assert_eq!(statuses, [200, 200, 429]);
}

/// **An encoded name is read decoded** (P8): `Mcp-Name` in the
/// revision's Base64 form agrees with the body it names.
#[test]
fn an_encoded_name_header_is_decoded() {
    let served = Served::plain();
    let answered = post_with(
        &served,
        &numerology(14),
        &[("Mcp-Name", Some("=?base64?bnVtZXJvbG9neS5wcm9maWxl?="))],
    );
    assert_eq!(answered.status, 200, "{}", answered.body);
}

/// **The binary serves the transport** (P8): `--http` names the endpoint
/// on stderr, port and all, and its options reach the transport, the
/// rate here.
#[test]
fn the_binary_serves_the_endpoint_its_options_describe() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_teistro-mcp"))
        .args([
            "--http",
            "127.0.0.1:0",
            "--ephemeris",
            "NONE",
            "--http-rate",
            "1",
        ])
        .stdin(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the binary starts");
    let mut stderr = BufReader::new(child.stderr.take().unwrap());
    let mut line = String::new();
    stderr.read_line(&mut line).unwrap();
    let address = line
        .trim()
        .strip_prefix("teistro-mcp: serving http://")
        .and_then(|rest| rest.strip_suffix("/mcp"))
        .map(str::to_owned);
    let statuses: Option<Vec<u16>> = address.as_ref().map(|address| {
        let served = Served {
            address: address.clone(),
        };
        (0..3)
            .map(|_| post(&served, &numerology(15)).status)
            .collect()
    });
    let _ = child.kill();
    let _ = child.wait();
    assert!(address.is_some(), "no address in {line:?}");
    assert_eq!(statuses, Some(vec![200, 200, 429]));
}

/// **A task is the process's** (P7): begun on one worker, it is polled to
/// its end through whichever worker serves each poll, its `Mcp-Name` the
/// task's id.
#[test]
fn a_task_is_polled_through_every_worker() {
    let served = Served::tasking();
    let mut call = days(1, None);
    call["params"]["_meta"]["io.modelcontextprotocol/clientCapabilities"] =
        json!({ "extensions": { "io.modelcontextprotocol/tasks": {} } });
    let created = post(&served, &call).json();
    assert_eq!(created["result"]["resultType"], "task", "{created}");
    let id = created["result"]["taskId"].as_str().unwrap().to_owned();
    let started = Instant::now();
    loop {
        let poll = json!({ "jsonrpc": "2.0", "id": 2, "method": "tasks/get", "params": {
            "_meta": call["params"]["_meta"].clone(), "taskId": id } });
        let polled = post(&served, &poll);
        assert_eq!(polled.status, 200, "{}", polled.body);
        let task = polled.json()["result"].clone();
        if task["status"] == "completed" {
            assert_eq!(task["result"]["isError"], false, "{task}");
            break;
        }
        assert_eq!(task["status"], "working", "{task}");
        assert!(
            started.elapsed() < Duration::from_secs(120),
            "the task never ended"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}
