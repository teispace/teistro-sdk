//! The Node binding's own tests: the generated decoders against blobs the
//! library really produced, and the generated types against a consumer
//! that type-checks at maximum strictness (ADR-0023).
//!
//! Run by hand (`cargo xtask check-node`) and in the nightly matrix; the
//! fast check stays Rust-only (ADR-0014). The TypeScript step is skipped
//! with a note when no compiler is on the machine, because a type-check
//! needs one and the decoder tests do not.

use std::path::{Path, PathBuf};
use std::process::Command;

const FIXTURES: &str = "target/tsrb";
const TESTS: &str = "bindings/node/test/";
const TSCONFIG: &str = "bindings/node/typecheck/tsconfig.json";

fn cargo() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| String::from("cargo"))
}

/// The TypeScript compiler, when the machine has one: `TSC`, a local
/// install beside the consumer, or one npm has already fetched.
fn typescript(root: &Path) -> Option<(String, Vec<String>)> {
    if let Ok(tsc) = std::env::var("TSC") {
        return Some((tsc, Vec::new()));
    }
    let local: PathBuf = root.join("bindings/node/typecheck/node_modules/.bin/tsc");
    if local.exists() {
        return Some((local.display().to_string(), Vec::new()));
    }
    let npx = Command::new("npx")
        .args(["--no-install", "tsc", "--version"])
        .current_dir(root)
        .output();
    match npx {
        Ok(output) if output.status.success() => Some((
            String::from("npx"),
            ["--no-install", "tsc"]
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
        )),
        _ => None,
    }
}

/// Runs a step and reports it: `Ok(())` when it passed, `Err(())` when it
/// did not, with the line the gate prints either way.
fn step(command: &mut Command, passed: &str, failed: &str) -> Result<(), ()> {
    let status = command.status();
    if status.is_ok_and(|s| s.success()) {
        if !passed.is_empty() {
            println!("ok    {passed}");
        }
        Ok(())
    } else {
        println!("FAIL  {failed}");
        Err(())
    }
}

pub(crate) fn check(root: &Path) -> i32 {
    if Command::new("node").arg("--version").output().is_err() {
        eprintln!("no `node` on this machine; the Node binding's tests need it");
        return 0;
    }
    let fixtures = root.join(FIXTURES);
    let outcome = step(
        Command::new(cargo())
            .args([
                "run",
                "--quiet",
                "-p",
                "teistro-ffi",
                "--example",
                "blob_fixtures",
                "--",
            ])
            .arg(&fixtures)
            .current_dir(root),
        "",
        "the blob fixtures did not build",
    )
    .and_then(|()| {
        step(
            Command::new("node")
                .args(["--test", TESTS])
                .arg(&fixtures)
                .current_dir(root),
            &format!("{TESTS} decodes what the library produced"),
            &format!("{TESTS} did not pass"),
        )
    });
    if outcome.is_err() {
        return 1;
    }
    let Some((tsc, args)) = typescript(root) else {
        println!(
            "skip  {TSCONFIG}: no TypeScript compiler (set TSC, or `npm install typescript` in bindings/node/typecheck)"
        );
        return 0;
    };
    let checked = step(
        Command::new(&tsc)
            .args(&args)
            .args(["-p", TSCONFIG])
            .current_dir(root),
        &format!("{TSCONFIG} type-checks at maximum strictness"),
        &format!("{TSCONFIG} does not type-check"),
    );
    i32::from(checked.is_err())
}
