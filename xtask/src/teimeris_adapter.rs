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
//! Three steps, each a failure of its own:
//!
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

const ADAPTER: &str = "adapters/ephemeris-teimeris/rust";
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
        let mut command = Command::new(cargo());
        command.args(args).current_dir(&dir);
        command
    };
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
