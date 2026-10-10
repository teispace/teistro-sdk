//! `teistro-mcp`: the server over stdio, one JSON-RPC message a line in
//! and one reply a line out; anything it logs goes to stderr
//! (`03-design/mcp-server.md`).

use std::io::{BufRead as _, Write as _};
use std::process::ExitCode;

use teistro_mcp::{Adapter, Engine, LEGACY, MODERN, Server};

const USAGE: &str = "teistro-mcp: the Teistro SDK as Model Context Protocol tools, over stdio

usage: teistro-mcp [--ephemeris NAME] [--plugin PATH [--plugin-config JSON]]

  --ephemeris NAME       the ephemeris every tool computes with: BUILTIN (the
                         default), SURYA_SIDDHANTA or NONE; with a plugin, the
                         fallback where the plugin does not answer
  --plugin PATH          an engine's adapter, the shared library its package
                         ships, computed with first; its own operations become
                         tools named `engine.<name>`
  --plugin-config JSON   the adapter's own options
  --help                 this text
  --version              the server's version and the revisions it speaks
";

/// What the command line asked for.
struct Options {
    engine: Engine,
    plugin: Option<String>,
    plugin_config: Option<String>,
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
    for line in std::io::stdin().lock().lines() {
        let line = match line {
            Ok(line) => line,
            Err(why) => {
                let _ = writeln!(stderr, "teistro-mcp: stdin: {why}");
                return ExitCode::FAILURE;
            }
        };
        if line.trim().is_empty() {
            continue;
        }
        if let Some(reply) = server.handle(&line) {
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

/// The server the options name: the plugin loaded ahead of the engine
/// when there is one.
fn server(options: &Options) -> Result<Server, teistro::Error> {
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
    }))
}
