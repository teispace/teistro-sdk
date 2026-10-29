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
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use serde_json::{Value, json};
use teistro::Ephemeris;
use teistro_core::catalogue::Ayanamsha;
use teistro_core::quantity::{JulianDay, Ut1};
use teistro_mcp::{Engine, Interrupt, OpenEngine, Server};
use teistro_port_ephemeris::capabilities::{Capabilities, Obliquity};
use teistro_port_ephemeris::{
    Angles, AnglesRequest, CrossingEvent, CrossingRequest, EphemerisProvider, HorizonRequest,
    PositionColumns, PositionRequest, ProviderError, TimeScale,
};

fn meta(token: Option<&str>) -> Value {
    let mut meta = json!({ "io.modelcontextprotocol/protocolVersion": "2026-07-28",
        "io.modelcontextprotocol/clientCapabilities": {} });
    if let Some(token) = token {
        meta["progressToken"] = json!(token);
    }
    meta
}

/// A year of days at Kathmandu, about the most one call may ask (366):
/// long past the first progress notification (`EVERY`, 250 ms), so a
/// token hears progress and a cancel lands while the call computes. A
/// quarter once took seconds; the cheaper days made it end inside the
/// window under load.
fn days(id: u64, token: Option<&str>) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "method": "tools/call", "params": {
        "_meta": meta(token), "name": "almanac.days", "arguments": { "request": {
            "first": { "year": 2026, "month": 1, "day": 1 },
            "last": { "year": 2026, "month": 12, "day": 31 },
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
    assert!(
        !progress.is_empty(),
        "no progress before the answer: {lines:?}"
    );
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
/// computing, it is never answered and the next request is, through the
/// binary; and in-process it asks its engine for nothing past the part in
/// hand when the cancel came, counted in instants rather than timed.
///
/// The claim was once a time: from the cancel to the next answer, under
/// a quarter of a whole call timed afterwards on the same server. That
/// compared two moments of a machine's load, and a cold call with one the
/// cancelled call had warmed, so it failed under the chain's four-way
/// test load and passed alone. Counted where the engine is asked, with
/// the cancel given from inside the first progress notification, the
/// stop does not depend on how loaded the machine is.
#[test]
fn a_call_cancelled_while_it_computes_stops_unanswered() {
    let mut session = Session::start();
    session.send(&days(7, Some("cancel-me")));
    let first = session.next();
    progress(&first, "cancel-me");
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
        progress(&line, "cancel-me");
    }
    let after = session.finish();
    assert!(after.is_empty(), "the cancelled call went on: {after:?}");

    // In-process, the cancel is given from inside the first progress
    // notification, so no latency falls between the two, and the work is
    // counted where the engine is asked rather than timed.
    let cancelled = counted(7, true);
    assert_eq!(cancelled.answer, None, "a cancelled call was answered");
    let at = cancelled.at_cancel.unwrap();
    // The premise, on a fresh server (what one call caches shortens the
    // next): the same call uncancelled asks the engine for many parts
    // more than it had when the cancel came, so one left running would
    // be counted asking for them.
    let whole = counted(9, false);
    assert!(whole.answer.is_some(), "the whole call was not answered");
    assert!(
        whole.asked > at + 4 * PART,
        "the call had asked {at} of its {} instants at its first progress, too late to tell a \
         stopped call from one left running",
        whole.asked
    );
    // Stopped, it asks at most the part whose turn passed the cancel.
    assert!(
        cancelled.asked <= at + PART,
        "the cancelled call ran on: it asked the engine for {} instants after its cancel at \
         {at}, a whole call for {}",
        cancelled.asked - at,
        whole.asked
    );
    assert_eq!(
        cancelled.heard, 1,
        "a cancelled call reported progress after its cancel"
    );
}

/// The most instants the server asks its engine for in one request
/// (`watch::PART`); a cancel is seen between two.
const PART: u64 = 512;

/// What [`counted`] saw of one call.
struct Counted {
    /// The engine instants it asked for: a position request's instants,
    /// and one for any other request.
    asked: u64,
    /// How many it had asked for when the cancel was given, if one was.
    at_cancel: Option<u64>,
    /// The progress notifications it reported.
    heard: usize,
    answer: Option<String>,
}

/// A year of days asked of a fresh in-process server whose engine counts
/// what it is asked for, under a progress token, and cancelled from
/// inside its first progress notification when `cancel`.
fn counted(id: u64, cancel: bool) -> Counted {
    let asked = Arc::new(AtomicU64::new(0));
    let open_asked = Arc::clone(&asked);
    let open: OpenEngine = Box::new(move || {
        let inner = Ephemeris::Builtin.open()?.unwrap();
        let provider: Box<dyn EphemerisProvider> = Box::new(Asked {
            inner,
            asked: Arc::clone(&open_asked),
        });
        Ok(provider)
    });
    let heard = Arc::new(Mutex::new((0_usize, None::<u64>)));
    let interrupt = Arc::new(OnceLock::<Interrupt>::new());
    let (sink, stop, seen) = (
        Arc::clone(&heard),
        Arc::clone(&interrupt),
        Arc::clone(&asked),
    );
    let mut server = Server::with_engine(Engine::Builtin, open)
        .unwrap()
        .with_notify(move |line| {
            progress(&serde_json::from_str(&line).unwrap(), "counted");
            let mut heard = sink.lock().unwrap();
            heard.0 += 1;
            if cancel && heard.0 == 1 {
                heard.1 = Some(seen.load(Ordering::SeqCst));
                stop.get().unwrap().cancel(&json!(id));
            }
        });
    interrupt.set(server.interrupt()).unwrap();
    let answer = server.handle(&days(id, Some("counted")).to_string());
    let (heard, at_cancel) = *heard.lock().unwrap();
    Counted {
        asked: asked.load(Ordering::SeqCst),
        at_cancel,
        heard,
        answer,
    }
}

/// An engine that counts what it is asked for, as the server's watch
/// counts it, and answers as the one it wraps.
struct Asked {
    inner: Box<dyn EphemerisProvider>,
    asked: Arc<AtomicU64>,
}

impl Asked {
    fn ask(&self, instants: usize) {
        self.asked
            .fetch_add(u64::try_from(instants.max(1)).unwrap(), Ordering::SeqCst);
    }
}

impl EphemerisProvider for Asked {
    fn capabilities(&self) -> Capabilities {
        self.inner.capabilities()
    }

    fn positions(&self, request: &PositionRequest<'_>) -> Result<PositionColumns, ProviderError> {
        self.ask(request.jds.len());
        self.inner.positions(request)
    }

    fn obliquity(&self, jd: f64, scale: TimeScale) -> Result<Obliquity, ProviderError> {
        self.ask(1);
        self.inner.obliquity(jd, scale)
    }

    fn delta_t_seconds(&self, jd_ut1: f64) -> Result<f64, ProviderError> {
        self.ask(1);
        self.inner.delta_t_seconds(jd_ut1)
    }

    fn ayanamsha_deg(
        &self,
        jd: f64,
        scale: TimeScale,
        ayanamsha: Ayanamsha,
    ) -> Result<f64, ProviderError> {
        self.ask(1);
        self.inner.ayanamsha_deg(jd, scale, ayanamsha)
    }

    fn dut1_seconds(&self, jd_utc: f64) -> Result<f64, ProviderError> {
        self.ask(1);
        self.inner.dut1_seconds(jd_utc)
    }

    fn horizon_event(
        &self,
        request: &HorizonRequest,
    ) -> Result<Option<JulianDay<Ut1>>, ProviderError> {
        self.ask(1);
        self.inner.horizon_event(request)
    }

    fn angles(&self, request: &AnglesRequest) -> Result<Angles, ProviderError> {
        self.ask(1);
        self.inner.angles(request)
    }

    fn crossings(&self, request: &CrossingRequest) -> Result<Vec<CrossingEvent>, ProviderError> {
        self.ask(1);
        self.inner.crossings(request)
    }

    fn native_manifest(&self) -> Result<String, ProviderError> {
        self.inner.native_manifest()
    }

    fn native_call(&self, function: &str, arguments_json: &str) -> Result<String, ProviderError> {
        self.ask(1);
        self.inner.native_call(function, arguments_json)
    }
}

/// The engine instants a progress notification for `token` reports.
fn progress(line: &Value, token: &str) -> u64 {
    assert_eq!(line["method"], "notifications/progress", "{line}");
    assert_eq!(line["params"]["progressToken"], token, "{line}");
    line["params"]["progress"].as_u64().unwrap()
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
