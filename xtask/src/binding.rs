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

/// What a C consumer must link beside the SDK's own library.
///
/// **`-lm`**, because the astronomy calls `sin`, `atan2` and the rest,
/// and on Linux and MinGW the maths functions live in a separate `libm`
/// that the linker will not pull in by itself. On macOS they are in
/// libSystem and the flag is a harmless no-op — which is exactly why
/// this was missing: every gate that ran locally passed, and three of
/// the five platforms in the nightly matrix could not link at all.
///
/// Read by both C gates and stated in `bindings/c/README.md`, so the
/// line a consumer is told to run is the line the gates run.
pub(crate) const C_LINK_FLAGS: [&str; 1] = ["-lm"];

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
