//! The Node binding's own tests: the addon built and loaded, the whole
//! surface exercised through the ergonomic layer, the generated decoders
//! against blobs the library really produced, and the generated types
//! against a consumer that type-checks at maximum strictness (ADR-0023).
//!
//! Run by hand (`cargo xtask check-node`) and in the nightly matrix; the
//! fast check stays Rust-only (ADR-0014). The TypeScript step is skipped
//! with a note when no compiler is on the machine, because a type-check
//! needs one and the tests do not.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::binding::{blob_fixtures, build, present, step};

const FIXTURES: &str = "target/tsrb";
const TESTS: &str = "bindings/node/test/";
const TSCONFIG: &str = "bindings/node/typecheck/tsconfig.json";
/// Where the addon is loaded from: Node requires the `.node` suffix, so
/// the cdylib Cargo builds is copied there.
pub(crate) const ADDON: &str = "bindings/node/native/index.node";

/// The name Cargo gives the addon on this platform.
pub(crate) const ADDON_ARTEFACT: &str = if cfg!(target_os = "macos") {
    "libteistro_node.dylib"
} else if cfg!(target_os = "windows") {
    "teistro_node.dll"
} else {
    "libteistro_node.so"
};

/// Builds the addon and puts it where the loader looks.
fn build_addon(root: &Path) -> Result<(), ()> {
    build(root, "teistro-node", "the Node addon")?;
    let built = root.join("target/release").join(ADDON_ARTEFACT);
    let addon = root.join(ADDON);
    std::fs::copy(&built, &addon).map(|_| ()).map_err(|e| {
        println!(
            "FAIL  {} could not be copied to {ADDON}: {e}",
            built.display()
        );
    })
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

pub(crate) fn check(root: &Path) -> i32 {
    if !present("node", "--version") {
        eprintln!("no `node` on this machine; the Node binding's tests need it");
        return 0;
    }
    let fixtures = root.join(FIXTURES);
    if build_addon(root).is_err() {
        return 1;
    }
    let outcome = blob_fixtures(root, &fixtures).and_then(|()| {
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
