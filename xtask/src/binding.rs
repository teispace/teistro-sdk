//! What the bindings' gates share: running a step and reporting it,
//! building the library each one loads, and writing the blob fixtures
//! their decoders read. `check-c`, `check-node` and `check-dart` differ
//! only in the toolchain they drive, so everything else is here once.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::platform::Platform;

/// The crate that builds the SDK's shared library, as Cargo names its
/// artefacts.
pub(crate) const LIBRARY_STEM: &str = "teistro_ffi";

/// The native libraries a C consumer must link beside the SDK's own,
/// **as the toolchain reports them**.
///
/// Not a list written here. `libteistro_ffi.a` carries Rust's standard
/// library, and what that needs is a property of the target and the
/// toolchain version: `-lm` on Linux and MinGW where the maths functions
/// are separate, and on Windows some thirty more — `ws2_32`, `userenv`,
/// `ntdll`, `__chkstk`. A list copied from one toolchain is a list that
/// is wrong for another, and a list written from memory is a guess in a
/// consumer's link line.
///
/// So the compiler is asked. `rustc --print native-static-libs` answers
/// for the build in front of it, which is the same answer
/// `bindings/c/README.md` tells a consumer to get.
///
/// Falls back to `-lm` alone when the question cannot be asked, because
/// that is the one flag every non-macOS platform certainly needs and a
/// gate that silently linked nothing extra is how this was missed for
/// days.
pub(crate) fn c_link_flags(root: &Path) -> Vec<String> {
    let asked = Command::new("cargo")
        .args([
            "rustc",
            "-q",
            "-p",
            "teistro-ffi",
            "--crate-type",
            "staticlib",
            "--",
            "--print",
            "native-static-libs",
        ])
        .current_dir(root)
        .output();
    let Ok(output) = asked else {
        return vec![String::from("-lm")];
    };
    let said = String::from_utf8_lossy(&output.stderr);
    let Some(line) = said
        .lines()
        .find_map(|line| line.split_once("native-static-libs:"))
        .map(|(_, tail)| tail)
    else {
        return vec![String::from("-lm")];
    };
    // Deduplicated, order kept: rustc repeats a library that more than
    // one crate asked for, and a linker does not need it twice.
    let mut seen = std::collections::BTreeSet::new();
    let flags: Vec<String> = line
        .split_whitespace()
        .filter(|flag| seen.insert((*flag).to_string()))
        .map(str::to_string)
        .collect();
    if flags.is_empty() {
        return vec![String::from("-lm")];
    }
    flags
}

/// Cargo, as the environment names it.
pub(crate) fn cargo() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| String::from("cargo"))
}

/// Whether a tool is on this machine, which decides whether a gate runs
/// or says why it is skipping (ADR-0014: the fast check stays Rust-only).
pub(crate) fn present(tool: &str, version: &str) -> bool {
    Command::new(tool).arg(version).output().is_ok()
}

/// Runs a step and reports it: `Ok(())` when it passed, `Err(())` when it
/// did not, with the line the gate prints either way.
pub(crate) fn step(command: &mut Command, passed: &str, failed: &str) -> Result<(), ()> {
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

/// The file name this platform gives the SDK's shared library.
pub(crate) fn library_artefact() -> String {
    Platform::host().shared(LIBRARY_STEM)
}

/// Builds a crate in release, quietly.
pub(crate) fn build(root: &Path, package: &str, what: &str) -> Result<(), ()> {
    step(
        Command::new(cargo())
            .args(["build", "--quiet", "--release", "-p", package])
            .current_dir(root),
        "",
        &format!("{what} did not build"),
    )
}

/// The shared library, built and its path returned.
pub(crate) fn library(root: &Path) -> Result<PathBuf, ()> {
    build(root, "teistro-ffi", "the library")?;
    Ok(root.join("target/release").join(library_artefact()))
}

/// Writes the result blobs a binding's decoders are tested against.
pub(crate) fn blob_fixtures(root: &Path, into: &Path) -> Result<(), ()> {
    step(
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
            .arg(into)
            .current_dir(root),
        "",
        "the blob fixtures did not build",
    )
}
