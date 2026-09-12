//! The Rust façade's own gate: the eight examples run, and the crate's
//! tests and doctests pass under the features the examples need.
//!
//! Run by hand (`cargo xtask check-rust`) and in the nightly matrix. The
//! fast check already compiles and lints these targets — `cargo clippy
//! --workspace --all-targets` sees an example — so what this gate adds
//! is the thing a compiler cannot say: that each program *runs*, and so
//! the README cannot describe a surface nobody has executed. That was
//! the lesson the install page taught on 12 September, and the four
//! other bindings each have this gate for the same reason.
//!
//! It is the only binding gate that needs no shared library: Rust
//! composes the crates, so there is nothing to build and load
//! (`03-design/rust-consumer-surface.md` §3).

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::binding::{cargo, step};

/// The crate the examples belong to, as Cargo names it.
const PACKAGE: &str = "teistro";
/// Where the examples live. **Every** file there is run, so a scenario
/// added to the directory is gated by having been added — the failure a
/// list in this file would eventually have is that someone writes an
/// example and forgets to list it.
const EXAMPLES: &str = "crates/sdk/examples";
/// The one example this gate leaves alone: it prints the parity report,
/// and `check-parity` runs it as one of four runners. Running it here
/// too would pay twice for one proof and bury this gate's output under
/// five hundred lines of `key<TAB>value`.
///
/// A name and not a list, because the rule is "the example another gate
/// already runs" and there is exactly one.
const RUN_BY_CHECK_PARITY: &str = "parity";

/// Every example in name order, by target name.
fn found(root: &Path) -> Result<Vec<String>, ()> {
    let directory = root.join(EXAMPLES);
    let mut names: Vec<String> = std::fs::read_dir(&directory)
        .map_err(|e| println!("FAIL  {EXAMPLES}: {e}"))?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|kind| kind == "rs"))
        .filter_map(|path: PathBuf| {
            path.file_stem()
                .map(|stem| stem.to_string_lossy().to_string())
        })
        .filter(|name| name != RUN_BY_CHECK_PARITY)
        .collect();
    names.sort();
    if names.is_empty() {
        println!("FAIL  {EXAMPLES} holds no examples");
        return Err(());
    }
    Ok(names)
}

/// Runs every example, in name order, and says how many.
///
/// An example is a program a reader is invited to copy, so it is held to
/// the same bar as a test: it must run against the crate this build
/// produced. **Release**, because the built-in ephemeris is a truncated
/// VSOP87 and ELP2000 and a debug build of it takes minutes over a year
/// of the sky — which is also what the README tells a reader to do.
fn examples(root: &Path, names: &[String]) -> Result<(), ()> {
    for name in names {
        step(
            Command::new(cargo())
                .args([
                    "run",
                    "--quiet",
                    "--release",
                    "-p",
                    PACKAGE,
                    "--example",
                    name,
                ])
                .current_dir(root),
            "",
            &format!("{EXAMPLES}/{name}.rs did not run"),
        )?;
    }
    println!("ok    {EXAMPLES}: {} example(s) run", names.len());
    Ok(())
}

/// The crate's own tests and doctests.
///
/// Here rather than left to `cargo test --workspace` because this gate
/// is what a maintainer runs when they have touched the façade, and
/// because the doctests are the crate's front door: the first thing a
/// consumer copies is the example in `lib.rs`.
fn tests(root: &Path) -> Result<(), ()> {
    step(
        Command::new(cargo())
            .args(["test", "--quiet", "--release", "-p", PACKAGE])
            .current_dir(root),
        &format!("{PACKAGE}: the surface tests and doctests pass"),
        &format!("{PACKAGE}'s own tests did not pass"),
    )
}

pub(crate) fn check(root: &Path) -> i32 {
    let Ok(names) = found(root) else {
        return 1;
    };
    if tests(root).is_err() || examples(root, &names).is_err() {
        return 1;
    }
    0
}
