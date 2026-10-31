//! `cargo xtask check-teimeris-adapter`: the Teimeris adapter built and
//! held to what every provider is held to (`05-testing/01-quality-bar.md`,
//! "provider conformance kit"), and loaded as a plugin by every binding.
//!
//! The adapter lives outside the workspace and links Teimeris from a
//! checkout beside the SDK's (`../teimeris`, the path its `Cargo.toml`
//! names), so no other gate builds it. Teimeris is a private repository:
//! verify's `teimeris` job clones it there with a read token, and a
//! machine without the checkout skips this gate, which a strict run
//! refuses.
//!
//! Every Cargo run in the adapter passes `--locked`, so its `Cargo.lock`,
//! which is committed, is held to what the SDK's crates now ask for: a
//! crate gaining a dependency fails here instead of leaving the lock
//! rewritten in whichever tree a gate ran. `cargo xtask
//! relock-teimeris-adapter` writes it again.
//!
//! Four steps, each a failure of its own:
//!
//! 0. the lock is current;
//! 1. the adapter's tests, in release as its timing tests expect;
//! 2. its kit binary: the eighteen checks under the published bounds and
//!    the façade's two chart checks, the report written to
//!    `target/teimeris-kit/teimeris.json`;
//! 3. the Node, Python, Dart and Java gates once more, with
//!    `TEISTRO_TEIMERIS_ADAPTER` naming the built library, so each
//!    binding's plugin test loads the engine instead of skipping.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::binding::{cargo, step};
use crate::platform::Platform;
use crate::skip::skip;

pub(crate) const ADAPTER: &str = "adapters/ephemeris-teimeris/rust";
const KIT: &str = "teistro-ephemeris-teimeris-kit";
const REPORT: &str = "target/teimeris-kit";
/// The adapter's library stem, as Cargo names the `cdylib`.
const STEM: &str = "teistro_ephemeris_teimeris";
/// What each binding reads to find the plugin.
const PLUGIN: &str = "TEISTRO_TEIMERIS_ADAPTER";
/// The binding gates whose suites load the plugin when it is named.
const BINDINGS: [&str; 4] = ["check-node", "check-python", "check-dart", "check-java"];

pub(crate) fn check(root: &Path) -> i32 {
    let source = teimeris(root);
    if !source.join("bindings/rust/teimeris/Cargo.toml").is_file() {
        skip(format!(
            "the Teimeris adapter: no Teimeris checkout at {} (a private repository; verify's \
             `teimeris` job clones it with a read token)",
            source.display()
        ));
        return 0;
    }
    let dir = root.join(ADAPTER);
    let release = |args: &[&str]| {
        let mut command = cargo_in(root);
        command.args(args);
        command
    };
    if step(
        release(&["metadata", "--quiet", "--format-version", "1"])
            .stdout(std::process::Stdio::null()),
        "the adapter's Cargo.lock is what the SDK's crates ask for",
        "the adapter's Cargo.lock is stale: run `cargo xtask relock-teimeris-adapter` and commit it",
    )
    .is_err()
    {
        return 1;
    }
    let mut failed = 0;
    if step(
        &mut release(&["test", "--quiet", "--release"]),
        "the adapter's own tests",
        "the adapter's tests failed",
    )
    .is_err()
    {
        failed += 1;
    }
    let report = root.join(REPORT);
    if std::fs::create_dir_all(&report).is_err() {
        println!("FAIL  cannot make {}", report.display());
        return 1;
    }
    let out = report.to_string_lossy().into_owned();
    if step(
        &mut release(&[
            "run",
            "--quiet",
            "--release",
            "--bin",
            KIT,
            "--",
            "--out",
            &out,
        ]),
        "the provider kit over Teimeris, every check inside its published bound",
        "the provider kit failed over Teimeris",
    )
    .is_err()
    {
        failed += 1;
    }
    let Ok(plugin) = library(&dir, &release) else {
        return failed + 1;
    };
    let Ok(xtask) = std::env::current_exe() else {
        println!("FAIL  cannot name this xtask to run the binding gates");
        return failed + 1;
    };
    for gate in BINDINGS {
        if step(
            Command::new(&xtask)
                .arg(gate)
                .env(PLUGIN, &plugin)
                .current_dir(root),
            &format!("{gate} with the Teimeris plugin loaded"),
            &format!("{gate} failed with the Teimeris plugin loaded"),
        )
        .is_err()
        {
            failed += 1;
        }
    }
    failed
}

/// Cargo in the adapter, held to its committed lock and building into its
/// own `target/`, where [`library`] reads the plugin, whatever
/// `CARGO_TARGET_DIR` a caller's shell names.
pub(crate) fn cargo_in(root: &Path) -> Command {
    let dir = root.join(ADAPTER);
    let mut command = Command::new(cargo());
    command
        .arg("--locked")
        .env("CARGO_TARGET_DIR", dir.join("target"))
        .current_dir(dir);
    command
}

/// `cargo xtask relock-teimeris-adapter`: the adapter's `Cargo.lock`
/// resolved again against the SDK's crates, keeping every version it
/// already pins.
pub(crate) fn relock(root: &Path) -> i32 {
    if !teimeris(root)
        .join("bindings/rust/teimeris/Cargo.toml")
        .is_file()
    {
        println!(
            "FAIL  no Teimeris checkout at {}: the lock cannot be resolved",
            teimeris(root).display()
        );
        return 1;
    }
    let outcome = step(
        Command::new(cargo())
            .args(["metadata", "--quiet", "--format-version", "1"])
            .stdout(std::process::Stdio::null())
            .current_dir(root.join(ADAPTER)),
        &format!("{ADAPTER}/Cargo.lock resolved"),
        "the adapter did not resolve",
    );
    i32::from(outcome.is_err())
}

/// Where the adapter's `Cargo.toml` reaches Teimeris: beside the SDK.
fn teimeris(root: &Path) -> PathBuf {
    root.join("..").join("teimeris")
}

/// The adapter's plugin library, built, and its path.
fn library(dir: &Path, release: &impl Fn(&[&str]) -> Command) -> Result<PathBuf, ()> {
    step(
        &mut release(&["build", "--quiet", "--release", "--lib"]),
        "",
        "the adapter's plugin library did not build",
    )?;
    let path = dir
        .join("target/release")
        .join(Platform::host().shared(STEM));
    if path.is_file() {
        Ok(path)
    } else {
        println!(
            "FAIL  the adapter built, and {} is not there",
            path.display()
        );
        Err(())
    }
}
