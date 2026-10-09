//! `teistro-mcp`: the server over stdio, one JSON-RPC message a line in
//! and one reply a line out; anything it logs goes to stderr
//! (`03-design/mcp-server.md`).

use std::io::{BufRead as _, Write as _};
use std::process::ExitCode;

use teistro_mcp::{Engine, LEGACY, MODERN, Server};

const USAGE: &str = "teistro-mcp: the Teistro SDK as Model Context Protocol tools, over stdio

usage: teistro-mcp [--ephemeris NAME]

  --ephemeris NAME   the ephemeris every tool computes with: BUILTIN (the
                     default), SURYA_SIDDHANTA or NONE
  --help             this text
  --version          the server's version and the revisions it speaks
";

fn main() -> ExitCode {
    let mut stderr = std::io::stderr();
    let engine = match engine(std::env::args().skip(1)) {
        Ok(Some(engine)) => engine,
        Ok(None) => return ExitCode::SUCCESS,
        Err(why) => {
            let _ = writeln!(stderr, "teistro-mcp: {why}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    let mut server = Server::new(engine);
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

/// The engine the command line names, `None` when it asked only for help
/// or the version, which are written here.
fn engine(mut args: impl Iterator<Item = String>) -> Result<Option<Engine>, String> {
    let mut engine = Engine::Builtin;
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
            _ => return Err(format!("no option `{arg}`")),
        }
    }
    Ok(Some(engine))
}
