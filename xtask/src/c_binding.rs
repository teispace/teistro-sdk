//! The C binding's own test: builds the library, compiles
//! `bindings/c/tests/smoke.c` against the generated header with warnings
//! as errors, and runs it. It proves what no Rust test can, that a C
//! compiler agrees with the header about struct layouts, enum values and
//! the calling convention, and that the library links.
//!
//! Run by hand (`cargo xtask check-c`) and in the nightly matrix; the fast
//! check stays Rust-only (ADR-0014).

use std::path::Path;
use std::process::Command;

use crate::binding::{build, present};

const SMOKE: &str = "bindings/c/tests/smoke.c";
const HEADER_DIR: &str = "bindings/c/include";

/// The compiler to use: `CC` when the environment names one, else `cc`.
fn compiler() -> String {
    std::env::var("CC").unwrap_or_else(|_| String::from("cc"))
}

pub(crate) fn check(root: &Path) -> i32 {
    let cc = compiler();
    if !present(&cc, "--version") {
        eprintln!("no `{cc}` on this machine; the C binding's test needs a C compiler");
        return 0;
    }
    if build(root, "teistro-ffi", "the library").is_err() {
        return 1;
    }
    let out = root.join("target/release/teistro-c-smoke");
    let library = root.join("target/release");
    let compiled = Command::new(&cc)
        .args(["-std=c11", "-Wall", "-Wextra", "-Wpedantic", "-Werror"])
        .arg("-I")
        .arg(root.join(HEADER_DIR))
        .arg("-o")
        .arg(&out)
        .arg(root.join(SMOKE))
        .arg("-L")
        .arg(&library)
        .arg("-lteistro_ffi")
        .status();
    match compiled {
        Ok(status) if status.success() => {}
        _ => {
            println!("FAIL  {SMOKE} does not compile against {HEADER_DIR}/teistro.h");
            return 1;
        }
    }
    match Command::new(&out).status() {
        Ok(status) if status.success() => {
            println!("ok    {SMOKE} compiles under C11 and passes");
            0
        }
        _ => {
            println!("FAIL  {SMOKE} did not pass");
            1
        }
    }
}
