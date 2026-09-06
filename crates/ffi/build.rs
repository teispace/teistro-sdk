//! Builds the SDK's locale bundles from `i18n/` into `OUT_DIR`, so the C
//! library carries every shipped locale and a consumer needs no files to
//! render the SDK's own messages (ADR-0010). `bundles.rs` lists them for
//! `include_bytes!`.
//!
//! It also records what this build is, for `ts_build_info`: the commit it
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

use std::fmt::Write as _;
use std::path::PathBuf;
use std::process::Command;

use teistro_intl::pack::build_bundle;
use teistro_intl::source::Tree;

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
    let root = teistro_intl::sdk_root();
    println!("cargo:rerun-if-changed={}", root.display());
    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("cargo sets OUT_DIR"));
    let tree = Tree::load(&root).unwrap_or_else(|e| panic!("the i18n/ sources load: {e}"));
    let mut entries = String::new();
    for (tag, locale) in &tree.locales {
        let bytes = build_bundle(locale).unwrap_or_else(|e| panic!("{tag}: {e}"));
        let path = out.join(format!("{tag}.tbundle"));
        std::fs::write(&path, bytes).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let _ = writeln!(
            entries,
            "    ({tag:?}, include_bytes!({:?})),",
            path.display()
        );
    }
    let listing = format!(
        "/// The SDK's locales as bundles, built from `i18n/` when the crate is compiled.\npub(crate) static BUNDLES: &[(&str, &[u8])] = &[\n{entries}];\n"
    );
    let path = out.join("bundles.rs");
    std::fs::write(&path, listing).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let repository = root.parent().unwrap_or(&root).to_path_buf();
    println!(
        "cargo:rerun-if-changed={}",
        repository.join(".git/HEAD").display()
    );
    build_info(&out, &repository);
}
