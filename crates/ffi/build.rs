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

use std::path::{Path, PathBuf};
use std::process::Command;

/// What `git` says about the tree this is built from, or `None` when the
/// build is not from a checkout (a published crate, a vendored source).
fn commit(root: &Path) -> Option<(String, bool)> {
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
fn build_info(out: &Path, identity: Option<(String, bool)>) {
    let (commit, dirty) = identity.unwrap_or_else(|| (String::from("unknown"), false));
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

/// The file `cargo package` writes into a crate packaged from a git
/// checkout: the commit and whether the tree was dirty.
const VCS_INFO: &str = ".cargo_vcs_info.json";

/// The commit a packaged crate was cut from, as its [`VCS_INFO`] records
/// it, or `None` when this is not a packaged crate (or one packaged
/// outside git). Read with no JSON parser, so the script keeps no build
/// dependency: `"sha1": "<hex>"`, and `"dirty": true` only when it was.
fn packaged(manifest: &Path) -> Option<(String, bool)> {
    let text = std::fs::read_to_string(manifest.join(VCS_INFO)).ok()?;
    let sha1 = text
        .split_once("\"sha1\"")?
        .1
        .split('"')
        .nth(1)?
        .to_string();
    let dirty = text.split_once("\"dirty\"").is_some_and(|(_, rest)| {
        rest.trim_start()
            .trim_start_matches(':')
            .trim_start()
            .starts_with("true")
    });
    Some((sha1, dirty))
}

fn main() {
    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("cargo sets OUT_DIR"));
    // **The locale bundles are the façade's now.** This script used to
    // build them from `i18n/` as well, and `crates/sdk/build.rs` does
    // that — so the library still carries every shipped locale
    // (ADR-0010), through the context it now composes with. What is left
    // here is what only the C library has: which build a binding is
    // talking to.
    let manifest = PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR").expect("cargo sets CARGO_MANIFEST_DIR"),
    );
    // Built from its package (a registry, a vendored source, the bundle
    // `cargo xtask check-packages` builds), the crate says which commit it
    // was cut from, and nothing outside the crate is read: its parent is
    // whatever directory the package was unpacked into, which may be
    // someone else's checkout.
    if let Some(identity) = packaged(&manifest) {
        println!(
            "cargo:rerun-if-changed={}",
            manifest.join(VCS_INFO).display()
        );
        build_info(&out, Some(identity));
        return;
    }
    // In the repository, `crates/ffi` is two levels down, and git says
    // what the tree is. Anything else is a source with no history to ask.
    let repository = manifest
        .parent()
        .and_then(Path::parent)
        .filter(|root| manifest.ends_with("crates/ffi") && root.join("Cargo.toml").is_file());
    let Some(repository) = repository else {
        build_info(&out, None);
        return;
    };
    watch(repository);
    build_info(&out, commit(repository));
}

/// Tells cargo when the stamp goes stale, so it is written again. HEAD
/// alone changes only when the branch does: a commit moves the branch's
/// ref, staging moves the index, and an edit to what the library is
/// built from changes the tree without touching git at all, so each is
/// watched. Git names its own paths, which a worktree keeps elsewhere.
fn watch(root: &Path) {
    let git = |args: &[&str]| -> Option<PathBuf> {
        let output = Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .ok()?;
        output
            .status
            .success()
            .then(|| root.join(String::from_utf8_lossy(&output.stdout).trim()))
    };
    let branch = Command::new("git")
        .args(["symbolic-ref", "-q", "HEAD"])
        .current_dir(root)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string());
    let mut paths: Vec<PathBuf> = ["HEAD", "index", "packed-refs"]
        .into_iter()
        .chain(branch.as_deref())
        .filter_map(|name| git(&["rev-parse", "--git-path", name]))
        .collect();
    paths.extend(
        ["crates", "catalogue", "Cargo.toml", "Cargo.lock"]
            .into_iter()
            .map(|source| root.join(source)),
    );
    for path in paths {
        println!("cargo:rerun-if-changed={}", path.display());
    }
}
