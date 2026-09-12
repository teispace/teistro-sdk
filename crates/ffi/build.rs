//! Records what this build is, for `ts_build_info`: the commit it
//! came from and whether the tree was clean, the target, the profile and
//! the compiler. A binding refuses a library that is not the build its
//! own half was generated for
//! (`02-architecture/07-binding-architecture.md`, "Loading and
//! identity").

// A build script stops the build by panicking and speaks to cargo through
// stdout, so the library lints against both are allowed here.
#![allow(
    clippy::print_stdout,
    clippy::expect_used,
    clippy::panic,
    reason = "a build script"
)]

use std::path::PathBuf;
use std::process::Command;

/// What `git` says about the tree this is built from, or `None` when the
/// build is not from a checkout (a published crate, a vendored source).
fn commit(root: &std::path::Path) -> Option<(String, bool)> {
    let git = |args: &[&str]| -> Option<String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .ok()?;
        output
            .status
            .success()
            .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
    };
    let head = git(&["rev-parse", "HEAD"])?;
    let dirty = !git(&["status", "--porcelain"])?.is_empty();
    Some((head, dirty))
}

/// The build's identity as the JSON `ts_build_info` hands out. Written
/// here rather than assembled at run time, so it costs nothing and cannot
/// disagree with the library it is compiled into.
fn build_info(out: &std::path::Path, root: &std::path::Path) {
    let (commit, dirty) = commit(root).unwrap_or_else(|| (String::from("unknown"), false));
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| String::from("unknown"));
    let target = std::env::var("TARGET").unwrap_or_else(|_| String::from("unknown"));
    let debug = std::env::var("DEBUG").is_ok_and(|v| v != "false");
    let optimised = std::env::var("OPT_LEVEL").is_ok_and(|v| v != "0");
    let rustc = std::env::var("RUSTC")
        .ok()
        .and_then(|rustc| Command::new(rustc).arg("--version").output().ok())
        .map_or_else(
            || String::from("unknown"),
            |out| String::from_utf8_lossy(&out.stdout).trim().to_string(),
        );
    // The sanitizers a build may carry, which a loader refuses by
    // default: an instrumented library answers, but not at the speed or
    // in the memory a consumer expects.
    let sanitizer = std::env::var("CARGO_ENCODED_RUSTFLAGS")
        .unwrap_or_default()
        .split('\u{1f}')
        .find_map(|flag| flag.strip_prefix("-Zsanitizer=").map(str::to_string))
        .unwrap_or_default();
    let json = format!(
        "{{\"sdk\":\"{sdk}\",\"abi\":{abi},\"catalogue\":{catalogue},\"commit\":\"{commit}\",\"dirty\":{dirty},\"profile\":\"{profile}\",\"target\":\"{target}\",\"debug_assertions\":{debug},\"optimised\":{optimised},\"sanitizer\":\"{sanitizer}\",\"rustc\":\"{rustc}\"}}",
        sdk = std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| String::from("0.0.0")),
        abi = 1,
        catalogue = 1,
    );
    let path = out.join("buildinfo.json");
    std::fs::write(&path, &json).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
}

fn main() {
    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("cargo sets OUT_DIR"));
    // **The locale bundles are the façade's now.** This script used to
    // build them from `i18n/` as well, and `crates/sdk/build.rs` does
    // that — so the library still carries every shipped locale
    // (ADR-0010), through the context it now composes with. What is left
    // here is what only the C library has: which build a binding is
    // talking to.
    // The repository, from this crate's own manifest: `crates/ffi` is
    // two levels down. Derived rather than asked of `teistro-intl`,
    // which this script no longer needs now that the bundles are the
    // façade's -- so it has no build dependency at all.
    let manifest = PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR").expect("cargo sets CARGO_MANIFEST_DIR"),
    );
    let repository = manifest
        .parent()
        .and_then(std::path::Path::parent)
        .map_or_else(|| manifest.clone(), std::path::Path::to_path_buf);
    println!(
        "cargo:rerun-if-changed={}",
        repository.join(".git/HEAD").display()
    );
    build_info(&out, &repository);
}
