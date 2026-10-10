//! Streamable HTTP, stateless (`03-design/mcp-server.md` §7, P8): the
//! modern revision keeps no session, so one POST is one message and the
//! dispatcher stdio reads answers it unchanged.
//!
//! A request whose `_meta` carries a `progressToken` is answered as an
//! event stream, its progress first and its answer last; any other is
//! answered as JSON. Closing the connection cancels the call it carries,
//! which is how the revision cancels over HTTP. The headers the revision
//! requires are checked against the body before anything is computed,
//! and a browser's `Origin` must be a loopback one or one the operator
//! allowed. Authorization is left to whatever the operator puts in
//! front: the server holds no secret.
//!
//! A [`teistro::Context`] is neither `Send` nor `Sync`, so each worker
//! thread builds a server of its own and keeps its contexts.

use core::fmt::Write as _;
use std::collections::HashMap;
use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::net::{IpAddr, TcpListener, TcpStream};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError, mpsc};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use crate::{
    Fault, Interrupt, MODERN, PROGRESS_META, Server, UNSUPPORTED_VERSION, VERSION_META, failure,
    limits,
};

/// The headers disagreeing with the body they carry, or missing
/// (`HeaderMismatchError`).
const HEADER_MISMATCH: i64 = -32_020;

/// The most bytes a request's line and headers take.
const HEAD_BYTES: u64 = 16 << 10;

/// How long a peer may take to send its request, or to take an answer.
const PATIENCE: Duration = Duration::from_secs(30);

/// The most bytes read past once an answer is written.
const LINGER_BYTES: usize = 1 << 20;

/// How often a running call looks at whether its connection closed.
const LOOK: Duration = Duration::from_millis(50);

/// Builds the server a worker thread answers with.
pub type MakeServer = dyn Fn() -> Result<Server, teistro::Error> + Send + Sync;

/// How the HTTP transport serves.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Http {
    /// The path the endpoint answers at.
    pub path: String,
    /// The worker threads, each answering one connection at a time.
    pub workers: usize,
    /// The origins a browser may call from beside the loopback ones,
    /// each written as the browser sends it (`https://app.example`).
    pub origins: Vec<String>,
    /// The most bytes one message carries, as [`crate::Limits`] bound it.
    pub message_bytes: Option<usize>,
    /// The most messages one peer address sends a second, sustained,
    /// with twice as many allowed in a burst; past it a message is `429`.
    /// `None` leaves the rate to whatever stands in front.
    pub rate: Option<u32>,
}

impl Default for Http {
    /// `/mcp`, four workers, loopback origins only, the shipped message
    /// bound, and fifty messages a second from each peer.
    fn default() -> Http {
        Http {
            path: String::from("/mcp"),
            workers: 4,
            origins: Vec::new(),
            message_bytes: crate::Limits::default().message_bytes,
            rate: Some(50),
        }
    }
}

/// Answers every connection `listener` accepts, until it fails. Each
/// worker builds its server with `make` before the first connection is
/// taken, so a server that cannot be built is reported here rather than
/// to a caller.
///
/// # Errors
///
/// `make`'s refusal, or the listener failing.
pub fn serve(listener: &TcpListener, http: &Http, make: &Arc<MakeServer>) -> std::io::Result<()> {
    let workers = http.workers.max(1);
    let (send, connections) = mpsc::sync_channel::<TcpStream>(workers.saturating_mul(16));
    let connections = Arc::new(Mutex::new(connections));
    let buckets = Arc::new(Mutex::new(Buckets::default()));
    let (ready, started) = mpsc::channel();
    for _ in 0..workers {
        let (make, connections, ready, http, buckets) = (
            Arc::clone(make),
            Arc::clone(&connections),
            ready.clone(),
            http.clone(),
            Arc::clone(&buckets),
        );
        std::thread::spawn(move || work(&*make, &connections, &ready, &http, &buckets));
    }
    drop(ready);
    for outcome in started.iter().take(workers) {
        outcome.map_err(std::io::Error::other)?;
    }
    for stream in listener.incoming() {
        let Ok(stream) = stream else {
            // A connection the peer dropped before it was taken, or a
            // process out of descriptors for a moment.
            std::thread::sleep(LOOK);
            continue;
        };
        match send.try_send(stream) {
            Ok(()) => {}
            Err(mpsc::TrySendError::Full(stream)) => {
                plain(
                    &stream,
                    503,
                    "every worker is busy",
                    &[("Retry-After", "1")],
                );
            }
            Err(mpsc::TrySendError::Disconnected(_)) => {
                return Err(std::io::Error::other("every worker has stopped"));
            }
        }
    }
    Ok(())
}

/// Each peer address's token bucket: what it may still send, and when
/// that was last counted.
#[derive(Default)]
struct Buckets {
    peers: HashMap<IpAddr, (f64, Instant)>,
}

impl Buckets {
    /// The peers remembered before the idle ones are forgotten.
    const REMEMBERED: usize = 4_096;

    /// Whether `peer` may send a message at `now`, `rate` a second with a
    /// burst of twice that, spending one when it may.
    fn take(&mut self, peer: IpAddr, rate: u32, now: Instant) -> bool {
        let rate = f64::from(rate.max(1));
        let burst = rate * 2.0;
        if self.peers.len() >= Self::REMEMBERED {
            self.peers
                .retain(|_, (_, last)| now.duration_since(*last).as_secs_f64() * rate < burst);
        }
        let (tokens, last) = self.peers.entry(peer).or_insert((burst, now));
        *tokens = (*tokens + now.duration_since(*last).as_secs_f64() * rate).min(burst);
        *last = now;
        if *tokens >= 1.0 {
            *tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

type Sink = Arc<Mutex<Option<TcpStream>>>;

fn locked<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A worker's server, its progress written as events to the stream the
/// running call answers on.
fn built(make: &MakeServer, sink: &Sink) -> Result<Server, String> {
    let sink = Arc::clone(sink);
    let server = make().map_err(|why| why.to_string())?;
    Ok(server.with_notify(move |notification| {
        if let Some(stream) = locked(&sink).as_mut() {
            let _ = write!(stream, "event: message\ndata: {notification}\n\n")
                .and_then(|()| stream.flush());
        }
    }))
}

fn work(
    make: &MakeServer,
    connections: &Mutex<mpsc::Receiver<TcpStream>>,
    ready: &mpsc::Sender<Result<(), String>>,
    http: &Http,
    buckets: &Mutex<Buckets>,
) {
    let sink: Sink = Arc::new(Mutex::new(None));
    let mut server = match built(make, &sink) {
        Ok(server) => server,
        Err(why) => {
            let _ = ready.send(Err(why));
            return;
        }
    };
    let _ = ready.send(Ok(()));
    loop {
        let Ok(stream) = locked(connections).recv() else {
            return;
        };
        // A panic answering one connection closes it and costs the
        // worker its contexts, not the worker.
        let answered = catch_unwind(AssertUnwindSafe(|| {
            exchange(&mut server, &sink, &stream, http, buckets);
        }));
        *locked(&sink) = None;
        linger(&stream);
        if answered.is_err() {
            match built(make, &sink) {
                Ok(fresh) => server = fresh,
                Err(_) => return,
            }
        }
    }
}

/// A request's line and headers.
struct Head {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
}

impl Head {
    /// Every value of the header `name`, matched ignoring case.
    fn all<'a>(&'a self, name: &str) -> impl Iterator<Item = &'a str> {
        let name = name.to_ascii_lowercase();
        self.headers
            .iter()
            .filter(move |(key, _)| key.eq_ignore_ascii_case(&name))
            .map(|(_, value)| value.as_str())
    }

    fn one(&self, name: &str) -> Option<&str> {
        self.all(name).next()
    }
}

/// An HTTP refusal: the status and a sentence saying why.
struct Refusal(u16, &'static str);

fn read_head(reader: &mut impl std::io::BufRead) -> Result<Head, Refusal> {
    let mut limited = reader.take(HEAD_BYTES);
    let mut line = String::new();
    let mut next = |line: &mut String| -> Result<(), Refusal> {
        line.clear();
        match limited.read_line(line) {
            Ok(0) => Err(Refusal(400, "the request ended inside its headers")),
            Ok(_) if !line.ends_with('\n') => {
                Err(Refusal(431, "the request's headers are too long"))
            }
            Ok(_) => {
                let kept = line.trim_end_matches(['\r', '\n']).len();
                line.truncate(kept);
                Ok(())
            }
            Err(_) => Err(Refusal(400, "the request's headers are not text")),
        }
    };
    next(&mut line)?;
    let mut parts = line.split(' ');
    let (Some(method), Some(target), Some(version), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(Refusal(
            400,
            "the request line is not `METHOD target HTTP/1.1`",
        ));
    };
    if !matches!(version, "HTTP/1.1" | "HTTP/1.0") {
        return Err(Refusal(505, "the server speaks HTTP/1.1"));
    }
    let path = target.split_once('?').map_or(target, |(path, _)| path);
    let (method, path) = (method.to_owned(), path.to_owned());
    let mut headers = Vec::new();
    loop {
        next(&mut line)?;
        if line.is_empty() {
            return Ok(Head {
                method,
                path,
                headers,
            });
        }
        if line.starts_with([' ', '\t']) {
            return Err(Refusal(400, "a header is folded onto the next line"));
        }
        let Some((name, value)) = line.split_once(':') else {
            return Err(Refusal(400, "a header is not `Name: value`"));
        };
        headers.push((name.trim().to_owned(), value.trim().to_owned()));
    }
}

/// Whether a browser at `origin` may call: a loopback page, or one the
/// operator named.
fn allowed(origin: &str, http: &Http) -> bool {
    if http.origins.iter().any(|allowed| allowed == origin) {
        return true;
    }
    let Some(rest) = origin
        .strip_prefix("http://")
        .or_else(|| origin.strip_prefix("https://"))
    else {
        return false;
    };
    let host = if let Some(bracketed) = rest.strip_prefix('[') {
        bracketed.split_once(']').map(|(host, _)| host)
    } else {
        rest.split(':').next()
    };
    matches!(host, Some("localhost" | "127.0.0.1" | "::1"))
}

/// Whether the media type `wanted` is one the `Accept` header takes;
/// an absent header takes everything.
fn accepts(head: &Head, wanted: &str) -> bool {
    let mut listed = head
        .all("accept")
        .flat_map(|value| value.split(','))
        .peekable();
    if listed.peek().is_none() {
        return true;
    }
    listed.any(|range| {
        let range = range.split(';').next().unwrap_or_default().trim();
        range == wanted
            || range == "*/*"
            || range == "application/*" && wanted.starts_with("application/")
    })
}

fn mismatch(message: String, field: &str) -> Fault {
    Fault {
        code: HEADER_MISMATCH,
        message,
        data: Some(json!({ "header": field })),
    }
}

/// Whether the headers the revision requires say what the body does.
fn agree(head: &Head, body: &Value) -> Result<(), Fault> {
    let version = head.one("mcp-protocol-version").ok_or_else(|| {
        mismatch(
            format!("a request names its revision in `MCP-Protocol-Version`, `{MODERN}`"),
            "MCP-Protocol-Version",
        )
    })?;
    if version != MODERN {
        return Err(Fault {
            code: UNSUPPORTED_VERSION,
            message: format!("over HTTP the server speaks revision {MODERN}, not {version}"),
            data: Some(json!({ "supported": [MODERN], "requested": version })),
        });
    }
    let params = body.get("params");
    let named = params
        .and_then(|params| params.get("_meta"))
        .and_then(|meta| meta.get(VERSION_META))
        .and_then(Value::as_str);
    if named.is_some_and(|named| named != version) {
        return Err(mismatch(
            format!("`MCP-Protocol-Version` says {version} and the body's `_meta` does not"),
            "MCP-Protocol-Version",
        ));
    }
    let method = body.get("method").and_then(Value::as_str);
    if head.one("mcp-method") != method {
        return Err(mismatch(
            String::from("`Mcp-Method` names the body's `method`"),
            "Mcp-Method",
        ));
    }
    let field = match method {
        Some("tools/call" | "prompts/get") => "name",
        Some("resources/read") => "uri",
        _ => return Ok(()),
    };
    let name = params
        .and_then(|params| params.get(field))
        .and_then(Value::as_str);
    let header = head.one("mcp-name").map(decoded);
    if header.as_deref() != name {
        return Err(mismatch(
            format!("`Mcp-Name` names the body's `params.{field}`"),
            "Mcp-Name",
        ));
    }
    Ok(())
}

/// A header value as the revision encodes one it cannot send raw,
/// `=?base64?…?=`, decoded; any other value as it stands, and an
/// encoding that does not decode as itself, so it disagrees with the
/// body it should name.
fn decoded(value: &str) -> std::borrow::Cow<'_, str> {
    value
        .strip_prefix("=?base64?")
        .and_then(|rest| rest.strip_suffix("?="))
        .and_then(base64)
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .map_or(std::borrow::Cow::Borrowed(value), std::borrow::Cow::Owned)
}

/// Standard Base64 with its padding, as RFC 4648 §4 writes it.
fn base64(text: &str) -> Option<Vec<u8>> {
    let sextet = |byte: u8| -> Option<u32> {
        Some(u32::from(match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        }))
    };
    let bytes = text.as_bytes();
    if bytes.len() % 4 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(bytes.len() / 4 * 3);
    for (at, quad) in bytes.chunks(4).enumerate() {
        let last = at + 1 == bytes.len() / 4;
        let padding = quad.iter().rev().take_while(|byte| **byte == b'=').count();
        if padding > 2 || (padding > 0 && !last) {
            return None;
        }
        let mut word = 0_u32;
        for byte in quad.get(..4 - padding)? {
            word = word << 6 | sextet(*byte)?;
        }
        word <<= 6 * u32::try_from(padding).ok()?;
        let [_, high, middle, low] = word.to_be_bytes();
        out.extend_from_slice([high, middle, low].get(..3 - padding)?);
    }
    Some(out)
}

/// The HTTP status a JSON-RPC reply is sent under.
fn status(code: Option<i64>) -> u16 {
    match code {
        None => 200,
        Some(crate::METHOD_NOT_FOUND) => 404,
        Some(_) => 400,
    }
}

fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        202 => "Accepted",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        406 => "Not Acceptable",
        411 => "Length Required",
        413 => "Content Too Large",
        415 => "Unsupported Media Type",
        429 => "Too Many Requests",
        431 => "Request Header Fields Too Large",
        501 => "Not Implemented",
        503 => "Service Unavailable",
        505 => "HTTP Version Not Supported",
        _ => "Internal Server Error",
    }
}

fn respond(mut stream: &TcpStream, status: u16, kind: &str, body: &[u8], extra: &[(&str, &str)]) {
    let mut head = format!(
        "HTTP/1.1 {status} {}\r\nContent-Type: {kind}\r\nContent-Length: {}\r\n\
         Cache-Control: no-store\r\nConnection: close\r\n",
        reason(status),
        body.len()
    );
    for (name, value) in extra {
        let _ = write!(head, "{name}: {value}\r\n");
    }
    head.push_str("\r\n");
    let _ = stream
        .write_all(head.as_bytes())
        .and_then(|()| stream.write_all(body))
        .and_then(|()| stream.flush());
}

fn plain(stream: &TcpStream, status: u16, why: &str, extra: &[(&str, &str)]) {
    respond(
        stream,
        status,
        "text/plain; charset=utf-8",
        format!("{why}\n").as_bytes(),
        extra,
    );
}

fn json(stream: &TcpStream, status: u16, reply: &str) {
    respond(stream, status, "application/json", reply.as_bytes(), &[]);
}

/// A request the endpoint does not take: its status, why, and a header
/// the refusal carries.
struct Turned(u16, String, Option<(&'static str, &'static str)>);

impl Turned {
    fn new(status: u16, why: &str) -> Turned {
        Turned(status, why.to_owned(), None)
    }

    fn write(&self, stream: &TcpStream) {
        let Turned(status, why, extra) = self;
        plain(stream, *status, why, extra.as_slice());
    }
}

/// A connection's request: its head and its message.
fn received(stream: &TcpStream, http: &Http) -> Result<(Head, Vec<u8>), Turned> {
    let mut reader = BufReader::new(stream);
    let head = read_head(&mut reader).map_err(|Refusal(status, why)| Turned::new(status, why))?;
    if head.path != http.path {
        return Err(Turned::new(404, "the server answers at its MCP path only"));
    }
    if head
        .one("origin")
        .is_some_and(|origin| !allowed(origin, http))
    {
        return Err(Turned::new(403, "this origin may not call the server"));
    }
    if head.method != "POST" {
        // A stateless server opens no stream of its own and keeps no
        // session to delete.
        return Err(Turned(
            405,
            String::from("each message is a POST"),
            Some(("Allow", "POST")),
        ));
    }
    let sized = "a message is sent with its `Content-Length`";
    if head.one("transfer-encoding").is_some() {
        return Err(Turned::new(501, sized));
    }
    let length = {
        let mut lengths = head.all("content-length");
        match (lengths.next().map(str::parse::<usize>), lengths.next()) {
            (None, _) => return Err(Turned::new(411, sized)),
            (Some(Ok(length)), None) => length,
            _ => return Err(Turned::new(400, "the `Content-Length` is not one count")),
        }
    };
    if let Some(most) = http.message_bytes.filter(|most| length > *most) {
        return Err(Turned(413, limits::too_long(most), None));
    }
    let kind = head.one("content-type").unwrap_or_default();
    if !kind
        .split(';')
        .next()
        .is_some_and(|kind| kind.trim().eq_ignore_ascii_case("application/json"))
    {
        return Err(Turned::new(415, "a message is `application/json`"));
    }
    if !accepts(&head, "application/json") && !accepts(&head, "text/event-stream") {
        return Err(Turned::new(
            406,
            "the server answers JSON or an event stream",
        ));
    }
    // The capacity is not taken on the peer's word.
    let mut message = Vec::with_capacity(length.min(1 << 20));
    let read = (&mut reader)
        .take(u64::try_from(length).unwrap_or(u64::MAX))
        .read_to_end(&mut message);
    if read.is_err() || message.len() != length {
        return Err(Turned::new(
            400,
            "the message is shorter than its `Content-Length`",
        ));
    }
    Ok((head, message))
}

/// One connection's request and its answer.
fn exchange(
    server: &mut Server,
    sink: &Sink,
    stream: &TcpStream,
    http: &Http,
    buckets: &Mutex<Buckets>,
) {
    let _ = stream.set_read_timeout(Some(PATIENCE));
    let _ = stream.set_write_timeout(Some(PATIENCE));
    let (head, message) = match received(stream, http) {
        Ok(received) => received,
        Err(turned) => return turned.write(stream),
    };
    // Counted once the message is read, so the refusal reaches the peer.
    if let (Some(rate), Ok(peer)) = (http.rate, stream.peer_addr()) {
        if !locked(buckets).take(peer.ip(), rate, Instant::now()) {
            let wait = [("Retry-After", "1")];
            return plain(stream, 429, "too many messages; slow down", &wait);
        }
    }
    let body: Value = serde_json::from_slice(&message).unwrap_or(Value::Null);
    if body.is_object() {
        if let Err(fault) = agree(&head, &body) {
            let id = body.get("id").unwrap_or(&Value::Null);
            return json(stream, 400, &failure(id, &fault));
        }
    }
    let Some(id) = body.get("id").cloned() else {
        // A notification, or a message the dispatcher refuses unread.
        return match server.handle_coded(&message) {
            Some((reply, code)) => json(stream, status(code), &reply),
            None => respond(stream, 202, "application/json", b"", &[]),
        };
    };
    let progress = body
        .get("params")
        .and_then(|params| params.get("_meta"))
        .is_some_and(|meta| meta.get(PROGRESS_META).is_some());
    // A subscription's acknowledgment goes before its closure, so it is
    // a stream too.
    let listening = body.get("method").and_then(Value::as_str) == Some("subscriptions/listen");
    let streamed = (progress || listening) && accepts(&head, "text/event-stream");
    if streamed {
        let mut writer = stream;
        let opened = writer
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\n\
                  Cache-Control: no-store\r\nX-Accel-Buffering: no\r\nConnection: close\r\n\r\n",
            )
            .and_then(|()| writer.flush());
        if opened.is_err() {
            return;
        }
        *locked(sink) = stream.try_clone().ok();
    }
    let interrupt = server.interrupt();
    let done = AtomicBool::new(false);
    let answered = std::thread::scope(|scope| {
        let watching = scope.spawn(|| watch_close(stream, &done, &interrupt, &id));
        let answered = server.handle_coded(&message);
        done.store(true, Ordering::Relaxed);
        let _ = watching.join();
        answered
    });
    *locked(sink) = None;
    // A call stopped because its connection closed has no one to answer.
    let Some((reply, code)) = answered else {
        return;
    };
    if streamed {
        let mut writer = stream;
        let _ = write!(writer, "event: message\ndata: {reply}\n\n").and_then(|()| writer.flush());
    } else {
        json(stream, status(code), &reply);
    }
}

/// Closes a connection once its answer is written: the server's side
/// first, then whatever the peer still sends is read past for a moment,
/// so a refusal sent before its body was read reaches the peer rather
/// than being lost to a reset.
fn linger(mut stream: &TcpStream) {
    let _ = stream.shutdown(std::net::Shutdown::Write);
    let _ = stream.set_read_timeout(Some(LOOK.saturating_mul(4)));
    let mut past = [0_u8; 4096];
    let mut left = LINGER_BYTES;
    while left > 0 {
        match stream.read(&mut past) {
            Ok(0) | Err(_) => return,
            Ok(read) => left = left.saturating_sub(read),
        }
    }
}

/// Stops the call `id` once its connection closes: the revision's
/// cancellation over HTTP. Bytes the peer sends after its message are
/// read past, since every connection carries one.
fn watch_close(mut stream: &TcpStream, done: &AtomicBool, interrupt: &Interrupt, id: &Value) {
    let _ = stream.set_read_timeout(Some(LOOK));
    let mut past = [0_u8; 256];
    while !done.load(Ordering::Relaxed) {
        match stream.read(&mut past) {
            Ok(0) => return interrupt.stop_running(id),
            Ok(_) => {}
            Err(why)
                if matches!(
                    why.kind(),
                    std::io::ErrorKind::WouldBlock
                        | std::io::ErrorKind::TimedOut
                        | std::io::ErrorKind::Interrupted
                ) => {}
            Err(_) => return interrupt.stop_running(id),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    fn head(text: &str) -> Result<Head, Refusal> {
        read_head(&mut std::io::Cursor::new(text.as_bytes().to_vec()))
    }

    #[test]
    fn a_head_is_read_and_its_headers_matched_ignoring_case() {
        let read = head("POST /mcp?x=1 HTTP/1.1\r\nMcp-Method: ping\r\nAccept: a, b\r\n\r\n")
            .ok()
            .unwrap();
        assert_eq!((read.method.as_str(), read.path.as_str()), ("POST", "/mcp"));
        assert_eq!(read.one("MCP-METHOD"), Some("ping"));
        for refused in [
            "POST /mcp\r\n\r\n",
            "POST /mcp HTTP/2\r\n\r\n",
            "POST /mcp HTTP/1.1\r\nFolded: a\r\n b\r\n\r\n",
            "POST /mcp HTTP/1.1\r\nno colon\r\n\r\n",
            "POST /mcp HTTP/1.1\r\nHost: x\r\n",
        ] {
            assert!(head(refused).is_err(), "{refused:?}");
        }
        let long = format!("POST /mcp HTTP/1.1\r\nX: {}\r\n\r\n", "a".repeat(20_000));
        assert!(matches!(head(&long), Err(Refusal(431, _))));
    }

    #[test]
    fn a_peer_past_its_rate_waits_and_another_peer_does_not() {
        let mut buckets = Buckets::default();
        let (one, two) = (IpAddr::from([127, 0, 0, 1]), IpAddr::from([127, 0, 0, 2]));
        let start = Instant::now();
        assert!((0..4).all(|_| buckets.take(one, 2, start)));
        assert!(!buckets.take(one, 2, start), "a burst is twice the rate");
        assert!(buckets.take(two, 2, start));
        assert!(buckets.take(one, 2, start + Duration::from_millis(500)));
        assert!(!buckets.take(one, 2, start + Duration::from_millis(500)));
    }

    #[test]
    fn an_encoded_name_is_decoded_and_a_broken_one_kept() {
        assert_eq!(decoded("chart.found"), "chart.found");
        assert_eq!(decoded("=?base64?Y2hhcnQuZm91bmQ=?="), "chart.found");
        assert_eq!(decoded("=?base64?YQ==?="), "a");
        assert_eq!(decoded("=?base64?YWI=?="), "ab");
        for broken in ["=?base64?Y2h?=", "=?base64?Y=Q=?=", "=?base64?****?="] {
            assert_eq!(decoded(broken), broken);
        }
    }

    #[test]
    fn only_a_loopback_or_named_origin_is_allowed() {
        let http = Http {
            origins: vec![String::from("https://app.example")],
            ..Http::default()
        };
        for origin in [
            "http://localhost:3000",
            "http://127.0.0.1",
            "https://[::1]:8443",
            "https://app.example",
        ] {
            assert!(allowed(origin, &http), "{origin}");
        }
        for origin in [
            "null",
            "https://evil.example",
            "http://localhost.evil.example",
            "https://app.example.evil",
            "http://127.0.0.1.evil",
        ] {
            assert!(!allowed(origin, &http), "{origin}");
        }
    }
}
