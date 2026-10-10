//! `teistro-mcp`: the server over stdio, one JSON-RPC message a line in
//! and one reply a line out; anything it logs goes to stderr
//! (`03-design/mcp-server.md`).

use std::io::{BufRead, Write as _};
use std::process::ExitCode;

use teistro_mcp::{Adapter, Detail, Engine, LEGACY, Limits, MODERN, Server};

const USAGE: &str = "teistro-mcp: the Teistro SDK as Model Context Protocol tools, over stdio

usage: teistro-mcp [--ephemeris NAME] [--plugin PATH [--plugin-config JSON]]
                   [--schemas DETAIL] [--max-message-bytes N] [--max-items N]
                   [--max-days N]

  --ephemeris NAME       the ephemeris every tool computes with: BUILTIN (the
                         default), SURYA_SIDDHANTA or NONE; with a plugin, the
                         fallback where the plugin does not answer
  --plugin PATH          an engine's adapter, the shared library its package
                         ships, computed with first; its own operations become
                         tools named `engine.<name>`
  --plugin-config JSON   the adapter's own options
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
    let mut server = match server(&options) {
        Ok(server) => server,
        Err(why) => {
            let _ = writeln!(stderr, "teistro-mcp: {why}");
            return ExitCode::FAILURE;
        }
    };
    let mut stdout = std::io::stdout().lock();
    let mut stdin = std::io::stdin().lock();
    let mut line = Vec::new();
    loop {
        match read_line(&mut stdin, options.limits.message_bytes, &mut line) {
            Ok(false) => return ExitCode::SUCCESS,
            Ok(true) => {}
            Err(why) => {
                let _ = writeln!(stderr, "teistro-mcp: stdin: {why}");
                return ExitCode::FAILURE;
            }
        }
        if line.trim_ascii().is_empty() {
            continue;
        }
        if let Some(reply) = server.handle_bytes(&line) {
            if writeln!(stdout, "{reply}")
                .and_then(|()| stdout.flush())
                .is_err()
            {
                // The host closed its end: nothing is left to answer.
                return ExitCode::SUCCESS;
            }
        }
    }
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

/// The server the options name: the plugin loaded ahead of the engine
/// when there is one.
fn server(options: &Options) -> Result<Server, teistro::Error> {
    let Some(path) = &options.plugin else {
        return Ok(Server::new(options.engine)
            .with_detail(options.detail)
            .with_limits(options.limits));
    };
    #[allow(
        unsafe_code,
        reason = "loading the adapter the operator named, the one unsafe call in this crate"
    )]
    // SAFETY: the operator named this library on the server's own command
    // line, which is the trust the binary itself runs with.
    let adapter = unsafe { Adapter::load(path, options.plugin_config.as_deref()) }?;
    Ok(Server::with_plugin(options.engine, adapter)?
        .with_detail(options.detail)
        .with_limits(options.limits))
}

/// The options the command line names, `None` when it asked only for
/// help or the version, which are written here.
fn options(mut args: impl Iterator<Item = String>) -> Result<Option<Options>, String> {
    let mut engine = Engine::Builtin;
    let (mut plugin, mut plugin_config) = (None, None);
    let mut detail = Detail::default();
    let mut limits = Limits::default();
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
            _ => return Err(format!("no option `{arg}`")),
        }
    }
    if plugin_config.is_some() && plugin.is_none() {
        return Err(String::from("`--plugin-config` configures a `--plugin`"));
    }
    Ok(Some(Options {
        engine,
        plugin,
        plugin_config,
        detail,
        limits,
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
