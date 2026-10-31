//! The C binding's own test: builds the library, compiles
//! `bindings/c/tests/smoke.c` against the generated header with warnings
//! as errors, and runs it. It proves what no Rust test can, that a C
//! compiler agrees with the header about struct layouts, enum values and
//! the calling convention, and that the library links. It then holds the
//! digest reader (`bindings/c/digest`) to `03-design/digest-format.md`.
//!
//! Run by hand (`cargo xtask check-c`) and in the nightly matrix; the fast
//! check stays Rust-only (ADR-0014).

use std::path::Path;
use std::process::Command;

use crate::binding::{FIXTURES, blob_fixtures, build, present};
use crate::platform::Platform;

const SMOKE: &str = "bindings/c/tests/smoke.c";
const HEADER_DIR: &str = "bindings/c/include";
const DIGEST_DIR: &str = "bindings/c/digest";
const DIGEST_SOURCE: &str = "bindings/c/digest/teistro_digest.c";
const DIGEST_TEST: &str = "bindings/c/tests/digest_read.c";

pub(crate) fn check(root: &Path) -> i32 {
    let cc = crate::binding::c_compiler();
    if !present(&cc, "--version") {
        crate::skip::skip(format_args!(
            "no `{cc}` on this machine; the C binding's test needs a C compiler"
        ));
        return 0;
    }
    if build(root, "teistro-ffi", "the library").is_err() {
        return 1;
    }
    let out = crate::binding::executable(&root.join("target/release"), "teistro-c-smoke");
    let library = root.join("target/release");
    // The shared library, and on Windows its import library by path:
    // `-lteistro_ffi` would find the static one sitting beside it.
    let compiled = Command::new(&cc)
        .args(["-std=c11", "-Wall", "-Wextra", "-Wpedantic", "-Werror"])
        .arg("-I")
        .arg(root.join(HEADER_DIR))
        .arg("-o")
        .arg(&out)
        .arg(root.join(SMOKE))
        .args(crate::binding::shared_link(&Platform::host(), &library))
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
        }
        _ => {
            println!("FAIL  {SMOKE} did not pass");
            return 1;
        }
    }
    i32::from(digest_reader(root, &cc).is_err())
}

/// The digest reader: nothing to link, and the library's own reader's
/// answer field for field; every refusal by name; the mutator under the
/// sanitizers.
fn digest_reader(root: &Path, cc: &str) -> Result<(), ()> {
    let release = root.join("target/release");
    let fixtures = root.join(FIXTURES);
    blob_fixtures(root, &fixtures)?;
    let bytes = fixtures.join("digest.tsrb");
    freestanding(root, cc, &release)?;
    let reader = crate::binding::executable(&release, "teistro-digest-read");
    let warned = ["-std=c11", "-Wall", "-Wextra", "-Wpedantic", "-Werror"];
    let compile = |sanitized: bool| {
        let mut command = Command::new(cc);
        command.args(warned);
        if sanitized {
            command.arg("-fsanitize=address,undefined").arg("-g");
        }
        command
            .arg("-I")
            .arg(root.join(DIGEST_DIR))
            .arg("-o")
            .arg(&reader)
            .arg(root.join(DIGEST_TEST))
            .arg(root.join(DIGEST_SOURCE))
            .stderr(std::process::Stdio::null());
        command.status().is_ok_and(|status| status.success())
    };
    if !compile(true) {
        crate::skip::skip(format_args!(
            "`{cc}` has no sanitizers here, so the digest reader runs without them"
        ));
        if !compile(false) {
            println!("FAIL  {DIGEST_TEST} does not compile");
            return Err(());
        }
    }
    let run = |mode: &str| {
        Command::new(&reader)
            .arg(mode)
            .arg(&bytes)
            .env("ASAN_OPTIONS", "symbolize=0:detect_leaks=0")
            .output()
    };
    let Ok(dumped) = run("dump") else {
        println!("FAIL  {DIGEST_TEST} did not run");
        return Err(());
    };
    let wanted = std::fs::read(fixtures.join("digest.dump")).map_err(|why| {
        println!("FAIL  the digest's dump: {why}");
    })?;
    if !dumped.status.success() || dumped.stdout != wanted {
        let line = dumped
            .stdout
            .split(|byte| *byte == b'\n')
            .zip(wanted.split(|byte| *byte == b'\n'))
            .position(|(got, want)| got != want);
        println!("FAIL  the C reader and the library's differ on the digest, at line {line:?}");
        return Err(());
    }
    println!("ok    the C reader reads every field the library's reader does");
    for mode in ["refuse", "mutate"] {
        match run(mode) {
            Ok(output) if output.status.success() => {
                let said = String::from_utf8_lossy(&output.stdout);
                println!("ok    the digest reader, {mode}: {}", said.trim());
            }
            Ok(output) => {
                println!(
                    "FAIL  the digest reader, {mode}: {}{}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
                return Err(());
            }
            Err(why) => {
                println!("FAIL  the digest reader, {mode}: {why}");
                return Err(());
            }
        }
    }
    Ok(())
}

/// Compiles the reader alone, with no C library, and holds its object to
/// what `03-design/digest-format.md` §7 says: no symbol from anywhere else,
/// so no allocation and no dependency.
fn freestanding(root: &Path, cc: &str, release: &Path) -> Result<(), ()> {
    let object = release.join("teistro-digest.o");
    let compiled = Command::new(cc)
        .args([
            "-std=c11",
            "-O2",
            "-Wall",
            "-Wextra",
            "-Wpedantic",
            "-Wconversion",
            "-Werror",
            "-ffreestanding",
            "-fno-stack-protector",
            "-fno-builtin",
            "-c",
        ])
        .arg("-o")
        .arg(&object)
        .arg(root.join(DIGEST_SOURCE))
        .status();
    if !compiled.is_ok_and(|status| status.success()) {
        println!("FAIL  {DIGEST_SOURCE} does not compile freestanding, warnings as errors");
        return Err(());
    }
    if !present("nm", "--version") {
        crate::skip::skip(format_args!(
            "no `nm` here, so the digest reader's symbols go unread"
        ));
        return Ok(());
    }
    let Ok(symbols) = Command::new("nm").arg("-u").arg(&object).output() else {
        println!("FAIL  `nm` did not run on the digest reader");
        return Err(());
    };
    let symbols = String::from_utf8_lossy(&symbols.stdout);
    let needed: Vec<&str> = symbols
        .split_whitespace()
        .filter(|word| *word != "U")
        .collect();
    if !needed.is_empty() {
        println!("FAIL  the digest reader needs {needed:?}");
        return Err(());
    }
    println!("ok    the digest reader links nothing");
    Ok(())
}
