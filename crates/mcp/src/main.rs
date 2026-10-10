//! `teistro-mcp`: the server over stdio, one JSON-RPC message a line in
//! and one reply a line out, or over Streamable HTTP with `--http`;
//! anything it logs goes to stderr (`03-design/mcp-server.md`).

use std::io::{BufRead, Write as _};
use std::net::TcpListener;
use std::process::ExitCode;
use std::sync::Arc;

use teistro_mcp::http::{self, Http, MakeServer};
use teistro_mcp::{Adapter, Detail, Engine, Interrupt, LEGACY, Limits, MODERN, Server};

const USAGE: &str = "teistro-mcp: the Teistro SDK as Model Context Protocol tools, over stdio
or Streamable HTTP

usage: teistro-mcp [--ephemeris NAME] [--plugin PATH [--plugin-config JSON]]
                   [--schemas DETAIL] [--max-message-bytes N] [--max-items N]
                   [--max-days N] [--pack PATH]...
                   [--http ADDRESS [--http-workers N] [--http-rate N]
                                   [--allow-origin ORIGIN]...]

  --ephemeris NAME       the ephemeris every tool computes with: BUILTIN (the
                         default), SURYA_SIDDHANTA or NONE; with a plugin, the
                         fallback where the plugin does not answer
  --plugin PATH          an engine's adapter, the shared library its package
                         ships, computed with first; its own operations become
                         tools named `engine.<name>`
  --plugin-config JSON   the adapter's own options
  --pack PATH            an interpretation or locale pack (`.tpack` or
                         `.tbundle`, as `teistro-intl build` writes it) loaded
                         into every context, in the order given; a locale it
                         brings is one a call may name. Read and verified at
                         start; repeatable
  --schemas DETAIL       how much schema the tool list states: lean (the
                         default; the records a request carries by name are
                         answered by `schema.describe`, and no answer schema
                         is listed) or full (everything, for a client that
                         validates structured content)
  --max-message-bytes N  the longest message read, 16777216 unless told; a
                         longer one is refused unread
  --max-items N          the most members of any array in a record, 1000
                         unless told: a batch's instants, a corpus's charts
  --max-days N           the most days a record's ranges span, 366 unless
                         told
                         (each takes `none` to bound nothing; a request past
                         a bound is refused naming its field and the bound)
  --http ADDRESS         serve Streamable HTTP at http://ADDRESS/mcp rather
                         than stdio (`127.0.0.1:8080`, or port 0 for any): the
                         2026-07-28 revision, stateless; authorization is left
                         to whatever stands in front
  --http-workers N       the calls answered at once over HTTP, 4 unless told
  --http-rate N          the messages a second one peer address may send, 50
                         unless told, with a burst of twice that; past it a
                         message is 429 (`none` leaves it to a proxy in front)
  --allow-origin ORIGIN  a browser origin that may call beside the loopback
                         ones, as the browser sends it; repeatable
  --help                 this text
  --version              the server's version and the revisions it speaks
";

/// What the command line asked for.
struct Options {
    engine: Engine,
    plugin: Option<String>,
    plugin_config: Option<String>,
    detail: Detail,
    limits: Limits,
    /// The address `--http` serves at, and how.
    http: Option<(String, Http)>,
    /// Each `--pack` by its path, read.
    packs: Vec<(String, Arc<[u8]>)>,
}

fn main() -> ExitCode {
    let mut stderr = std::io::stderr();
    let options = match options(std::env::args().skip(1)) {
        Ok(Some(options)) => options,
        Ok(None) => return ExitCode::SUCCESS,
        Err(why) => {
            let _ = writeln!(stderr, "teistro-mcp: {why}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    if let Some((address, http)) = options.http.clone() {
        return serve_http(options, &address, &http);
    }
    let server = match server(&options) {
        Ok(server) => server,
        Err(why) => {
            let _ = writeln!(stderr, "teistro-mcp: {why}");
            return ExitCode::FAILURE;
        }
    };
    let mut server = server.with_notify(|notification| {
        // A progress line beside the replies; a closed stdout ends the
        // server at its next reply.
        let mut stdout = std::io::stdout().lock();
        let _ = writeln!(stdout, "{notification}").and_then(|()| stdout.flush());
    });
    let lines = reading(server.interrupt(), options.limits.message_bytes);
    for line in lines {
        let line = match line {
            Ok(line) => line,
            Err(why) => {
                let _ = writeln!(stderr, "teistro-mcp: stdin: {why}");
                return ExitCode::FAILURE;
            }
        };
        if let Some(reply) = server.handle_bytes(&line) {
            let mut stdout = std::io::stdout().lock();
            if writeln!(stdout, "{reply}")
                .and_then(|()| stdout.flush())
                .is_err()
            {
                // The host closed its end: nothing is left to answer.
                return ExitCode::SUCCESS;
            }
        }
    }
    ExitCode::SUCCESS
}

/// Every line of stdin, read on a thread of its own so a cancellation is
/// heard while a call computes: a `notifications/cancelled` is handed to
/// `interrupt` as it arrives, and every other line is queued for the
/// server in order.
fn reading(
    interrupt: Interrupt,
    most: Option<usize>,
) -> std::sync::mpsc::Receiver<std::io::Result<Vec<u8>>> {
    let (send, lines) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut stdin = std::io::stdin().lock();
        loop {
            let mut line = Vec::new();
            match read_line(&mut stdin, most, &mut line) {
                Ok(false) => return,
                Ok(true) => {}
                Err(why) => {
                    let _ = send.send(Err(why));
                    return;
                }
            }
            if line.trim_ascii().is_empty() {
                continue;
            }
            if let Some(id) = cancelled(&line) {
                interrupt.cancel(&id);
                continue;
            }
            if send.send(Ok(line)).is_err() {
                return;
            }
        }
    });
    lines
}

/// The request a `notifications/cancelled` line cancels, or None for any
/// other line; only a line naming the method is parsed here.
fn cancelled(line: &[u8]) -> Option<serde_json::Value> {
    const METHOD: &[u8] = b"notifications/cancelled";
    if !line.windows(METHOD.len()).any(|window| window == METHOD) {
        return None;
    }
    let message: serde_json::Value = serde_json::from_slice(line).ok()?;
    if message.get("method")?.as_str()? != "notifications/cancelled" || message.get("id").is_some()
    {
        return None;
    }
    message.get("params")?.get("requestId").cloned()
}

/// The next line of `input` into `line`, without its newline; false at
/// the end of input. A line longer than `most` keeps its first `most + 1`
/// bytes and the rest is read past, so the server refuses it without
/// holding it.
fn read_line(
    input: &mut impl BufRead,
    most: Option<usize>,
    line: &mut Vec<u8>,
) -> std::io::Result<bool> {
    line.clear();
    let room = most.map_or(usize::MAX, |most| most.saturating_add(1));
    let mut read_any = false;
    loop {
        let buffer = match input.fill_buf() {
            Ok(buffer) => buffer,
            Err(why) if why.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(why) => return Err(why),
        };
        if buffer.is_empty() {
            return Ok(read_any);
        }
        read_any = true;
        let (taken, ended) = match buffer.iter().position(|byte| *byte == b'\n') {
            Some(at) => (at, true),
            None => (buffer.len(), false),
        };
        let kept = taken.min(room.saturating_sub(line.len()));
        line.extend_from_slice(buffer.get(..kept).unwrap_or_default());
        input.consume(if ended { taken + 1 } else { taken });
        if ended {
            return Ok(true);
        }
    }
}

/// Serves HTTP at `address` until the listener fails; the first line on
/// stderr names the endpoint, port and all.
fn serve_http(options: Options, address: &str, http: &Http) -> ExitCode {
    let mut stderr = std::io::stderr();
    let listener = match TcpListener::bind(address) {
        Ok(listener) => listener,
        Err(why) => {
            let _ = writeln!(stderr, "teistro-mcp: --http {address}: {why}");
            return ExitCode::FAILURE;
        }
    };
    let bound = listener.local_addr();
    let shown = bound
        .as_ref()
        .map_or_else(|_| address.to_owned(), ToString::to_string);
    let _ = writeln!(stderr, "teistro-mcp: serving http://{shown}{}", http.path);
    if !bound.is_ok_and(|bound| bound.ip().is_loopback()) {
        let _ = writeln!(
            stderr,
            "teistro-mcp: listening beyond loopback; the server checks no credentials"
        );
    }
    let make: Arc<MakeServer> = Arc::new(move || server(&options));
    match http::serve(&listener, http, &make) {
        Ok(()) => ExitCode::SUCCESS,
        Err(why) => {
            let _ = writeln!(stderr, "teistro-mcp: {why}");
            ExitCode::FAILURE
        }
    }
}

/// The server the options name: the plugin loaded ahead of the engine
/// when there is one.
fn server(options: &Options) -> Result<Server, teistro::Error> {
    let mut server = engine(options)?
        .with_detail(options.detail)
        .with_limits(options.limits);
    for (path, bytes) in &options.packs {
        server = server.with_pack(Arc::clone(bytes)).map_err(|refusal| {
            teistro::Error::new(teistro::Status::Pack, format!("--pack {path}: {refusal}"))
                .with_field("bytes")
        })?;
    }
    Ok(server)
}

/// The server computing with the engine the options name: the plugin
/// loaded ahead of it when there is one.
fn engine(options: &Options) -> Result<Server, teistro::Error> {
    let Some(path) = &options.plugin else {
        return Ok(Server::new(options.engine));
    };
    #[allow(
        unsafe_code,
        reason = "loading the adapter the operator named, the one unsafe call in this crate"
    )]
    // SAFETY: the operator named this library on the server's own command
    // line, which is the trust the binary itself runs with.
    let adapter = unsafe { Adapter::load(path, options.plugin_config.as_deref()) }?;
    Server::with_plugin(options.engine, adapter)
}

/// The options the command line names, `None` when it asked only for
/// help or the version, which are written here.
fn options(mut args: impl Iterator<Item = String>) -> Result<Option<Options>, String> {
    let mut engine = Engine::Builtin;
    let (mut plugin, mut plugin_config) = (None, None);
    let mut detail = Detail::default();
    let mut limits = Limits::default();
    let (mut address, mut workers, mut origins) = (None, None, Vec::new());
    let mut rate = None;
    let mut packs = Vec::new();
    let mut stdout = std::io::stdout();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                let _ = write!(stdout, "{USAGE}");
                return Ok(None);
            }
            "--version" | "-V" => {
                let _ = writeln!(
                    stdout,
                    "teistro-mcp {} (revisions {MODERN} and {LEGACY})",
                    env!("CARGO_PKG_VERSION")
                );
                return Ok(None);
            }
            "--ephemeris" => {
                let name = args.next().ok_or("`--ephemeris` takes a name")?;
                engine = Engine::from_name(&name).ok_or_else(|| {
                    format!(
                        "no ephemeris `{name}`; the names are {}",
                        Engine::NAMES.join(", ")
                    )
                })?;
            }
            "--plugin" => plugin = Some(args.next().ok_or("`--plugin` takes a path")?),
            "--plugin-config" => {
                plugin_config = Some(args.next().ok_or("`--plugin-config` takes JSON")?);
            }
            "--schemas" => {
                let name = args.next().ok_or("`--schemas` takes a detail")?;
                detail = Detail::from_name(&name).ok_or_else(|| {
                    format!(
                        "no schema detail `{name}`; the details are {}",
                        Detail::NAMES.join(", ")
                    )
                })?;
            }
            "--max-message-bytes" => limits.message_bytes = bound(&arg, args.next())?,
            "--max-items" => limits.items = bound(&arg, args.next())?,
            "--max-days" => limits.days = bound(&arg, args.next())?,
            "--http" => address = Some(args.next().ok_or("`--http` takes an address")?),
            "--http-workers" => {
                let count = args.next().ok_or("`--http-workers` takes a count")?;
                workers = Some(
                    count
                        .parse::<usize>()
                        .ok()
                        .filter(|count| *count > 0)
                        .ok_or_else(|| format!("`--http-workers` takes a count, not `{count}`"))?,
                );
            }
            "--pack" => {
                let path = args.next().ok_or("`--pack` takes a path")?;
                // Read once, here: every worker's server is built from
                // the same bytes and none reads the disk again.
                let bytes = std::fs::read(&path).map_err(|why| format!("--pack {path}: {why}"))?;
                packs.push((path, Arc::<[u8]>::from(bytes)));
            }
            "--http-rate" => rate = Some(bound::<u32>(&arg, args.next())?),
            "--allow-origin" => {
                origins.push(args.next().ok_or("`--allow-origin` takes an origin")?);
            }
            _ => return Err(format!("no option `{arg}`")),
        }
    }
    if plugin_config.is_some() && plugin.is_none() {
        return Err(String::from("`--plugin-config` configures a `--plugin`"));
    }
    if address.is_none() && (workers.is_some() || rate.is_some() || !origins.is_empty()) {
        return Err(String::from(
            "`--http-workers`, `--http-rate` and `--allow-origin` configure `--http`",
        ));
    }
    let http = address.map(|address| {
        let shipped = Http::default();
        let http = Http {
            workers: workers.unwrap_or(shipped.workers),
            origins,
            message_bytes: limits.message_bytes,
            rate: rate.unwrap_or(shipped.rate),
            ..shipped
        };
        (address, http)
    });
    Ok(Some(Options {
        engine,
        plugin,
        plugin_config,
        detail,
        limits,
        http,
        packs,
    }))
}

/// A bound as the command line writes it: a count, or `none`.
fn bound<T: std::str::FromStr>(option: &str, value: Option<String>) -> Result<Option<T>, String> {
    let value = value.ok_or_else(|| format!("`{option}` takes a count or `none`"))?;
    if value == "none" {
        return Ok(None);
    }
    value
        .parse()
        .map(Some)
        .map_err(|_| format!("`{option}` takes a count or `none`, not `{value}`"))
}
