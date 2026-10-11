//! Cancellation and progress (`03-design/mcp-server.md` §7, P6), through
//! the binary: a call carrying a progress token hears increasing progress
//! before its answer and one without hears none; a call cancelled while it
//! computes is stopped and never answered, and the next request is; a
//! request cancelled before it begins is not answered either.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index the replies they expect"
)]

use std::io::{BufRead as _, BufReader, Lines, Write as _};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::Instant;

use serde_json::{Value, json};
use teistro_mcp::{Engine, Server};

fn meta(token: Option<&str>) -> Value {
    let mut meta = json!({ "io.modelcontextprotocol/protocolVersion": "2026-07-28",
        "io.modelcontextprotocol/clientCapabilities": {} });
    if let Some(token) = token {
        meta["progressToken"] = json!(token);
    }
    meta
}

/// A quarter of days at Kathmandu: seconds of work in a debug build.
fn days(id: u64, token: Option<&str>) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "method": "tools/call", "params": {
        "_meta": meta(token), "name": "almanac.days", "arguments": { "request": {
            "first": { "year": 2026, "month": 1, "day": 1 },
            "last": { "year": 2026, "month": 3, "day": 31 },
            "latitudeDeg": 27.7, "longitudeDeg": 85.3, "utcOffsetSeconds": 20_700 } } } })
}

fn ping(id: u64) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "method": "ping", "params": { "_meta": meta(None) } })
}

struct Session {
    child: Child,
    stdin: Option<ChildStdin>,
    stdout: Lines<BufReader<ChildStdout>>,
}

impl Session {
    fn start() -> Session {
        let mut child = Command::new(env!("CARGO_BIN_EXE_teistro-mcp"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("the binary starts");
        let stdin = child.stdin.take();
        let stdout = BufReader::new(child.stdout.take().unwrap()).lines();
        Session {
            child,
            stdin,
            stdout,
        }
    }

    fn send(&mut self, message: &Value) {
        let stdin = self.stdin.as_mut().unwrap();
        writeln!(stdin, "{message}").unwrap();
        stdin.flush().unwrap();
    }

    fn next(&mut self) -> Value {
        serde_json::from_str(&self.stdout.next().unwrap().unwrap()).unwrap()
    }

    /// Every line left once stdin closes.
    fn finish(mut self) -> Vec<Value> {
        drop(self.stdin.take());
        let rest = self
            .stdout
            .by_ref()
            .map(|line| serde_json::from_str(&line.unwrap()).unwrap())
            .collect();
        assert!(self.child.wait().unwrap().success());
        rest
    }
}

/// **Progress is heard** (P6): a call carrying a token hears progress
/// under it, increasing, before its answer; one without hears none.
#[test]
fn a_call_with_a_token_hears_increasing_progress_before_its_answer() {
    let mut session = Session::start();
    session.send(&days(1, Some("quarter")));
    session.send(&days(2, None));
    let lines = session.finish();
    let answered = lines.iter().position(|line| line["id"] == 1).unwrap();
    let progress: Vec<f64> = lines[..answered]
        .iter()
        .map(|line| {
            assert_eq!(line["method"], "notifications/progress", "{line}");
            assert_eq!(line["params"]["progressToken"], "quarter");
            line["params"]["progress"].as_f64().unwrap()
        })
        .collect();
    assert!(!progress.is_empty(), "no progress before the answer");
    assert!(
        progress.windows(2).all(|pair| pair[0] < pair[1]),
        "{progress:?}"
    );
    assert_eq!(lines[answered]["result"]["isError"], false);
    let rest = &lines[answered + 1..];
    assert_eq!(rest.len(), 1, "the untokened call hears nothing: {rest:?}");
    assert_eq!(rest[0]["id"], 2);
}

/// **A cancelled call stops** (P6): cancelled once its progress shows it
/// computing, it is never answered, and the next request is answered at
/// once rather than after the work it would have done.
#[test]
fn a_call_cancelled_while_it_computes_stops_unanswered() {
    let mut session = Session::start();
    let started = Instant::now();
    session.send(&days(7, Some("cancel-me")));
    assert_eq!(session.next()["method"], "notifications/progress");
    session.send(
        &json!({ "jsonrpc": "2.0", "method": "notifications/cancelled",
                          "params": { "requestId": 7, "reason": "changed my mind" } }),
    );
    session.send(&ping(8));
    loop {
        let line = session.next();
        assert_ne!(line["id"], 7, "a cancelled call was answered: {line}");
        if line["id"] == 8 {
            break;
        }
        assert_eq!(line["method"], "notifications/progress", "{line}");
    }
    let stopped = started.elapsed();
    let whole = Instant::now();
    session.send(&days(9, None));
    let rest = session.finish();
    let whole = whole.elapsed();
    assert_eq!(rest.last().unwrap()["id"], 9);
    assert!(
        stopped < whole,
        "the cancelled call ran on: stopped after {stopped:?}, a whole call took {whole:?}"
    );
}

/// A request cancelled before it begins is never answered, and a
/// cancellation of an id no request carries changes nothing.
#[test]
fn a_request_cancelled_before_it_begins_is_not_answered() {
    let mut server = Server::new(Engine::None);
    let cancel = |id: u64| {
        json!({ "jsonrpc": "2.0", "method": "notifications/cancelled",
                "params": { "requestId": id } })
        .to_string()
    };
    assert_eq!(server.handle(&cancel(3)), None);
    assert_eq!(server.handle(&ping(3).to_string()), None);
    assert!(
        server.handle(&ping(3).to_string()).is_some(),
        "a cancellation is spent once"
    );
    assert_eq!(server.handle(&cancel(99)), None);
    assert!(server.handle(&ping(4).to_string()).is_some());
    server.interrupt().cancel(&json!("five"));
    let five = json!({ "jsonrpc": "2.0", "id": "five", "method": "ping",
                       "params": { "_meta": meta(None) } });
    assert_eq!(server.handle(&five.to_string()), None);
}
