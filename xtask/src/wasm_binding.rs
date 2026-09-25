//! The wasm binding's own gate: the module built for `wasm32-unknown-unknown`,
//! bound by the `wasm-bindgen` CLI the lockfile pins, and the **Node
//! binding's own test suite** run against it unchanged
//! (`03-design/wasm-binding.md` §5).
//!
//! That is the whole acceptance test, and a strong one, because the design
//! promises one thing: the wasm module exposes the same `native` object as
//! the napi addon, so one `index.js` serves both. `index.js` loads whatever
//! `TEISTRO_ADDON` names with `require`, which reads the module wasm-bindgen
//! writes for Node as readily as a `.node` file; so every call the suite
//! makes — charts, almanacs, positions, the locale engine, rules,
//! drawings, a provider written in JavaScript, refusals with their records,
//! `dispose` — goes through the wasm glue, and a member the backend spelt
//! differently fails a test that already exists.
//!
//! Run by hand (`cargo xtask check-wasm`) and in the nightly matrix.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::binding::{blob_fixtures, cargo, present, step};

/// The crate the module is built from, and the file Cargo names it.
const PACKAGE: &str = "teistro-wasm";
const MODULE: &str = "target/wasm32-unknown-unknown/release/teistro_wasm.wasm";
/// Where wasm-bindgen writes the module and its glue for Node.
const BOUND: &str = "target/wasm/nodejs";
/// The glue's entry point, which the suite loads through `TEISTRO_ADDON`.
const ENTRY: &str = "target/wasm/nodejs/teistro_wasm.js";
/// Where the CLI is installed when the machine has none of the right
/// version: inside the build tree, so nothing outside the repository is
/// touched and a cache of `target/` keeps it.
const TOOLS: &str = "target/tools";
const FIXTURES: &str = "target/tsrb";
const TESTS: &str = "bindings/node/test/";

/// The `wasm-bindgen` version the lockfile resolved. The CLI must be that
/// version exactly: the two halves of the bindings agree on a schema only
/// within one release, and a mismatched pair fails with an error about the
/// schema rather than about the version.
fn locked_version(root: &Path) -> Option<String> {
    let lock = std::fs::read_to_string(root.join("Cargo.lock")).ok()?;
    let mut lines = lock.lines();
    while let Some(line) = lines.next() {
        if line == r#"name = "wasm-bindgen""# {
            return lines
                .next()?
                .strip_prefix(r#"version = ""#)?
                .strip_suffix('"')
                .map(str::to_string);
        }
    }
    None
}

/// The CLI's own version, as it prints it (`wasm-bindgen 0.2.129`).
fn cli_version(cli: &Path) -> Option<String> {
    let output = Command::new(cli).arg("--version").output().ok()?;
    String::from_utf8(output.stdout)
        .ok()?
        .split_whitespace()
        .nth(1)
        .map(str::to_string)
}

/// A `wasm-bindgen` CLI of the locked version: `WASM_BINDGEN` when set,
/// then the one on the path, then one this gate installed before, and
/// otherwise one it installs now with `cargo install --locked`.
fn wasm_bindgen(root: &Path, wanted: &str) -> Result<PathBuf, ()> {
    let installed = root
        .join(TOOLS)
        .join("bin")
        .join(format!("wasm-bindgen{}", std::env::consts::EXE_SUFFIX));
    let candidates = [
        std::env::var_os("WASM_BINDGEN").map(PathBuf::from),
        Some(PathBuf::from("wasm-bindgen")),
        Some(installed.clone()),
    ];
    if let Some(found) = candidates
        .into_iter()
        .flatten()
        .find(|cli| cli_version(cli).as_deref() == Some(wanted))
    {
        return Ok(found);
    }
    step(
        Command::new(cargo())
            .args(["install", "--quiet", "--locked", "wasm-bindgen-cli"])
            .args(["--version", wanted, "--root"])
            .arg(root.join(TOOLS))
            .current_dir(root),
        &format!("wasm-bindgen {wanted} installed under {TOOLS}"),
        &format!("wasm-bindgen {wanted} could not be installed"),
    )?;
    Ok(installed)
}

pub(crate) fn check(root: &Path) -> i32 {
    if !present("node", "--version") {
        eprintln!("no `node` on this machine; the wasm binding's tests need it");
        return 0;
    }
    let Some(wanted) = locked_version(root) else {
        println!("FAIL  Cargo.lock names no `wasm-bindgen`; the wasm crate has lost its binding");
        return 1;
    };
    let outcome = wasm_bindgen(root, &wanted)
        .and_then(|cli| {
            step(
                Command::new(cargo())
                    .args(["build", "--quiet", "--release", "-p", PACKAGE])
                    .args(["--target", "wasm32-unknown-unknown"])
                    .current_dir(root),
                "",
                "the wasm module did not build",
            )?;
            step(
                Command::new(cli)
                    .args(["--target", "nodejs", "--out-dir", BOUND, MODULE])
                    .current_dir(root),
                &format!("{MODULE} bound for Node by wasm-bindgen {wanted}"),
                "wasm-bindgen could not bind the module",
            )
        })
        .and_then(|()| blob_fixtures(root, &root.join(FIXTURES)))
        .and_then(|()| {
            step(
                Command::new("node")
                    .args(["--test", TESTS])
                    .arg(root.join(FIXTURES))
                    .env("TEISTRO_ADDON", root.join(ENTRY))
                    // A plugin is a shared library, which a wasm module
                    // cannot open (ADR-0029); the suite's plugin tests
                    // skip without an adapter, and here there is none.
                    .env_remove("TEISTRO_TEIMERIS_ADAPTER")
                    .current_dir(root),
                &format!("{TESTS} passes against the wasm module, unchanged"),
                &format!("{TESTS} did not pass against the wasm module"),
            )
        });
    if outcome.is_err() {
        return 1;
    }
    if let Ok(meta) = std::fs::metadata(root.join(BOUND).join("teistro_wasm_bg.wasm")) {
        let tenths = meta.len() / 100_000;
        println!(
            "info  the module is {}.{} MB unoptimised; the profile binaries and their size gate are step 7",
            tenths / 10,
            tenths % 10
        );
    }
    0
}

#[cfg(test)]
mod tests {
    use super::locked_version;

    /// The version comes from the lockfile the workspace builds with, so
    /// the CLI can never be pinned apart from the library.
    #[test]
    fn the_cli_is_pinned_to_the_locked_library() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let version = locked_version(&root);
        assert!(
            version
                .as_deref()
                .is_some_and(|v| v.split('.').count() == 3),
            "{version:?}"
        );
    }
}
