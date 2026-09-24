//! The Rust façade's own gate: the examples run, and the crate's
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

use std::path::Path;
use std::process::Command;

use crate::binding::{cargo, step};
use crate::examples::{Binding, Runtime};

/// The crate the tests belong to, as Cargo names it.
const PACKAGE: &str = "teistro";

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

/// The examples, run as `check-parity` runs them to compare them with the
/// bindings' — every file in the directory but the parity runner, which
/// that gate runs as one of four runners.
fn examples(root: &Path) -> Result<(), ()> {
    Binding::Rust.run(root, &Runtime::of(root)).map(drop)
}

pub(crate) fn check(root: &Path) -> i32 {
    if tests(root).is_err() || examples(root).is_err() {
        return 1;
    }
    0
}
