//! The limits (`03-design/mcp-server.md` §7, P5): a range of days, an
//! array and a message past their bounds are refused naming the field,
//! the bound and the option moving it; at the bound they answer; and a
//! window of life, which spans decades by design, is not a range of days.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index the replies they expect"
)]

use std::io::{BufRead as _, BufReader, Write as _};
use std::process::{Command, Stdio};

use serde_json::{Value, json};
use teistro_mcp::{Engine, Limits, Server};

fn meta() -> Value {
    json!({ "io.modelcontextprotocol/protocolVersion": "2026-07-28" })
}

fn ask(server: &mut Server, method: &str, mut params: Value) -> Value {
    params["_meta"] = meta();
    let message = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
    serde_json::from_str(&server.handle(&message.to_string()).unwrap()).unwrap()
}

/// A tool's structured content, and whether it is a refusal.
fn call(server: &mut Server, name: &str, request: &Value) -> (Value, bool) {
    let reply = ask(
        server,
        "tools/call",
        json!({ "name": name, "arguments": { "request": request } }),
    );
    let result = &reply["result"];
    (
        result["structuredContent"].clone(),
        result["isError"]
            .as_bool()
            .unwrap_or_else(|| panic!("{reply}")),
    )
}

fn day(day: u32) -> Value {
    json!({ "year": 2026, "month": 10, "day": day })
}

/// `request` at Kathmandu, on its clock.
fn placed(mut request: Value) -> Value {
    request["latitudeDeg"] = json!(27.7172);
    request["longitudeDeg"] = json!(85.324);
    request["utcOffsetSeconds"] = json!(20_700);
    request
}

/// A range of days from the tenth to `last` at Kathmandu, with `beside`.
fn range(last: u32, beside: &Value) -> Value {
    let mut range = placed(beside.clone());
    range["first"] = day(10);
    range["last"] = day(last);
    range
}

/// A request built around a range of days ending on the given day.
type Request = Box<dyn Fn(u32) -> Value>;

/// Every record reading a range of days, its request built around the
/// range, and the field its range's `last` is named by.
fn ranged() -> Vec<(&'static str, Request, &'static str)> {
    vec![
        (
            "almanac.days",
            Box::new(|last| range(last, &json!({}))),
            "last",
        ),
        (
            "almanac.pakshi",
            Box::new(|last| range(last, &json!({ "native": { "bird": "OWL" } }))),
            "last",
        ),
        (
            "chart.rashifal",
            Box::new(|last| json!({ "periods": [range(last, &json!({}))], "baseline": "DAILY" })),
            "periods[0].last",
        ),
    ]
}

/// **A range of days is bounded** (P5): each record reading days answers
/// a range at the bound, and one a day past it is a `LIMIT` naming the
/// range's `last`, the bound and the option.
#[test]
fn each_range_of_days_answers_at_the_bound_and_is_refused_past_it() {
    let limits = Limits {
        days: Some(3),
        ..Limits::default()
    };
    let mut server = Server::new(Engine::Builtin).with_limits(limits);
    for (name, request, field) in ranged() {
        let (answer, refused) = call(&mut server, name, &request(12));
        assert!(!refused, "{name} over three days: {answer}");
        let (refusal, refused) = call(&mut server, name, &request(13));
        assert!(refused, "{name} over four days: {refusal}");
        assert_eq!(refusal["status"], "LIMIT", "{name}: {refusal}");
        assert_eq!(refusal["field"], field, "{name}: {refusal}");
        assert!(refusal["message"].as_str().unwrap().contains("at most 3"));
        assert!(refusal["hint"].as_str().unwrap().contains("--max-days"));
    }
}

/// The days of every range a record carries are summed: two periods of
/// two days each pass a bound of three at the second.
#[test]
fn the_days_of_every_range_are_summed() {
    let limits = Limits {
        days: Some(3),
        ..Limits::default()
    };
    let mut server = Server::new(Engine::None).with_limits(limits);
    let request = json!({ "periods": [range(11, &json!({})), range(11, &json!({}))] });
    let (refusal, refused) = call(&mut server, "chart.rashifal", &request);
    assert!(refused);
    assert_eq!(refusal["status"], "LIMIT", "{refusal}");
    assert_eq!(refusal["field"], "periods[1].last");
}

/// **A window of life is not a range of days**: a progressions window of
/// eighty years is read under the shipped bounds, which hold a year of
/// days.
#[test]
fn a_window_of_life_is_not_a_range_of_days() {
    let mut server = Server::new(Engine::None);
    let request = placed(json!({ "instant": 2_447_987.5,
        "progressions": { "contacts": { "from": 2_447_987.5, "to": 2_477_207.5 } } }));
    let (answer, _) = call(&mut server, "chart.found", &request);
    assert_ne!(answer["status"], "LIMIT", "{answer}");
}

/// **A batch is bounded** (P5): an array at the bound is read, one past
/// it is a `LIMIT` naming the array by its path.
#[test]
fn an_array_is_read_at_the_bound_and_refused_past_it() {
    let limits = Limits {
        items: Some(2),
        ..Limits::default()
    };
    let mut server = Server::new(Engine::Builtin).with_limits(limits);
    let found = |instants: &[f64]| placed(json!({ "instants": instants }));
    let (answer, refused) = call(
        &mut server,
        "chart.found",
        &found(&[2_447_987.5, 2_447_988.5]),
    );
    assert!(!refused, "{answer}");
    let (refusal, refused) = call(
        &mut server,
        "chart.found",
        &found(&[2_447_987.5, 2_447_988.5, 2_447_989.5]),
    );
    assert!(refused);
    assert_eq!(refusal["status"], "LIMIT", "{refusal}");
    assert_eq!(refusal["field"], "instants");
    assert!(refusal["hint"].as_str().unwrap().contains("--max-items"));
    let nested = json!({ "periods": [range(10, &json!({ "events": ["SUN", "MOON", "MARS"] }))] });
    let (refusal, _) = call(&mut server, "chart.rashifal", &nested);
    assert_eq!(refusal["field"], "periods[0].events", "{refusal}");
}

/// Unbounded limits refuse nothing a reader would read.
#[test]
fn unbounded_limits_refuse_nothing() {
    let mut server = Server::new(Engine::None).with_limits(Limits::UNBOUNDED);
    let (answer, _) = call(
        &mut server,
        "chart.rashifal",
        &json!({ "periods": [{
        "first": { "year": 1900, "month": 1, "day": 1 },
        "last": { "year": 2100, "month": 1, "day": 1 },
        "latitudeDeg": 27.7, "longitudeDeg": 85.3, "utcOffsetSeconds": 20_700 }] }),
    );
    assert_ne!(answer["status"], "LIMIT", "{answer}");
}

/// **A prompt writes no call past a bound**: the day panchanga's range
/// is refused at `prompts/get`, named as the prompt's `last`.
#[test]
fn a_prompt_past_a_bound_names_its_argument() {
    let mut server = Server::new(Engine::None);
    let reply = ask(
        &mut server,
        "prompts/get",
        json!({ "name": "day-panchanga", "arguments": {
            "date": "2026-01-01", "last": "2028-01-01", "zone": "+05:45",
            "latitude": "27.7", "longitude": "85.3" } }),
    );
    assert_eq!(reply["error"]["code"], -32_602, "{reply}");
    assert_eq!(reply["error"]["data"]["argument"], "last", "{reply}");
    assert!(
        reply["error"]["message"]
            .as_str()
            .unwrap()
            .contains("--max-days")
    );
}

/// **A message is bounded unread**: a line past the bound is refused with
/// a null id and the server reads the next one; `none` lifts the bound.
#[test]
fn a_message_past_the_bound_is_refused_and_the_next_is_read() {
    let replies = |args: &[&str]| -> Vec<Value> {
        let mut child = Command::new(env!("CARGO_BIN_EXE_teistro-mcp"))
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("the binary starts");
        let mut stdin = child.stdin.take().unwrap();
        let long = json!({ "jsonrpc": "2.0", "id": 1, "method": "ping",
                           "params": { "_meta": meta(), "padding": "a".repeat(4_096) } });
        let ping = json!({ "jsonrpc": "2.0", "id": 2, "method": "ping",
                           "params": { "_meta": meta() } });
        writeln!(stdin, "{long}\n{ping}").unwrap();
        drop(stdin);
        let replies = BufReader::new(child.stdout.take().unwrap())
            .lines()
            .map(|line| serde_json::from_str(&line.unwrap()).unwrap())
            .collect();
        assert!(child.wait().unwrap().success());
        replies
    };
    let bounded = replies(&["--max-message-bytes", "1024"]);
    assert_eq!(bounded[0]["id"], Value::Null, "{bounded:?}");
    assert_eq!(bounded[0]["error"]["code"], -32_600);
    assert_eq!(bounded[0]["error"]["data"]["limit"], 1_024);
    assert_eq!(bounded[1]["id"], 2, "{bounded:?}");
    assert!(bounded[1].get("result").is_some());
    let unbounded = replies(&["--max-message-bytes", "none"]);
    assert_eq!(unbounded[0]["id"], 1, "{unbounded:?}");
    assert!(unbounded[0].get("result").is_some());
}

/// A bound the command line cannot read is refused before the server
/// starts.
#[test]
fn the_command_line_refuses_a_bound_it_cannot_read() {
    let out = Command::new(env!("CARGO_BIN_EXE_teistro-mcp"))
        .args(["--max-days", "a year"])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("a count or `none`"));
}
