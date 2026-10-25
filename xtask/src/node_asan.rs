//! The Node addon under the address sanitizer: the binding's own suite run
//! unchanged against an addon whose every memory access is checked
//! (`05-testing/01-quality-bar.md`, "sanitizers").
//!
//! The `ffi` crate's tests already run under the sanitizer, but they call
//! the C ABI from Rust. The addon's generated glue sits between Node's values
//! and that ABI: it copies strings and arrays into the C structs a call
//! borrows and reads result blobs back out, and that glue runs only
//! inside Node.
//!
//! **Node itself is not rebuilt.** The addon is compiled with
//! `-Zsanitizer=address` and Node is started with the sanitizer's runtime
//! loaded ahead of everything else (`LD_PRELOAD`, or `DYLD_INSERT_LIBRARIES`
//! on macOS), so the runtime owns `malloc` and checks every access the addon
//! and the SDK beneath it make. V8's own heap is not checked, which is not
//! what this gate is for. Leak detection is off, because Node frees little
//! at exit and every report would be Node's own; the leak tests in each
//! binding hold the addon to giving memory back.
//!
//! **A run that checked nothing must not read as a clean one.** The gate
//! refuses an addon that calls no `__asan_report_*` function, and the
//! runtime is named by its path, so a missing runtime fails at load rather
//! than leaving the addon uninstrumented.
//!
//! Linux (glibc) and Apple-silicon macOS only: the runtime Linux loads is
//! clang's (`clang -print-file-name`, or `TEISTRO_ASAN_RUNTIME`), and the
//! one macOS loads is the Rust nightly's own. Run in verify's
//! `sanitizers` job; needs a nightly toolchain and Node.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::binding::{FIXTURES, blob_fixtures, present, step};
use crate::node_binding::{ADDON_STEM, TESTS};
use crate::platform::Platform;

/// The toolchain `-Zsanitizer` needs.
const NIGHTLY: &str = "+nightly";

/// What every instrumented access calls when it finds an error, and so
/// what an instrumented addon must import.
const REPORT: &[u8] = b"__asan_report_";

/// The runtime's options: stop at the first error, and leave leaks to the
/// bindings' leak tests.
const OPTIONS: &str = "halt_on_error=1:detect_leaks=0";

pub(crate) fn check(root: &Path) -> i32 {
    if !present("node", "--version") {
        crate::skip::skip("no `node` on this machine; the addon under ASan needs it");
        return 0;
    }
    let host = Platform::host();
    let preload = match (host.os, host.libc) {
        ("linux", Some("glibc")) => "LD_PRELOAD",
        ("darwin", _) if host.cpu == "arm64" => "DYLD_INSERT_LIBRARIES",
        _ => {
            crate::skip::skip(format_args!(
                "the Node addon under ASan runs on glibc Linux and Apple-silicon macOS, not {}",
                host.name()
            ));
            return 0;
        }
    };
    let Ok(addon) = build(root, &host) else {
        return 1;
    };
    let Ok(runtime) = runtime(&host) else {
        return 1;
    };
    let fixtures = root.join(FIXTURES);
    if blob_fixtures(root, &fixtures).is_err() {
        return 1;
    }
    let Ok(files) = suite(root) else {
        return 1;
    };
    let mut failed = 0;
    for file in &files {
        // Each file in Node itself, not under `node --test`: the runner
        // starts a process per file, and macOS's loader takes
        // `DYLD_INSERT_LIBRARIES` out of the environment it hands on, so
        // the runner's children would load the addon without the runtime.
        let run = step(
            Command::new("node")
                .arg(file)
                .env("TEISTRO_ADDON", &addon)
                .env("TEISTRO_SANITIZER", "address")
                .env("TEISTRO_FIXTURES", &fixtures)
                .env(preload, &runtime)
                .env("ASAN_OPTIONS", OPTIONS)
                .current_dir(root.join("bindings/node")),
            &format!("bindings/node/{file} passes with the addon under AddressSanitizer"),
            &format!("bindings/node/{file} failed with the addon under AddressSanitizer"),
        );
        failed += i32::from(run.is_err());
    }
    i32::from(failed > 0)
}

/// The suite's files, as `node_binding::SUITE` names them, relative to
/// the package.
fn suite(root: &Path) -> Result<Vec<String>, ()> {
    let entries = std::fs::read_dir(root.join(TESTS)).map_err(|why| {
        println!("FAIL  {TESTS} could not be read: {why}");
    })?;
    let mut files: Vec<String> = entries
        .filter_map(|entry| entry.ok()?.file_name().into_string().ok())
        .filter(|name| name.ends_with(".test.mjs"))
        .map(|name| format!("test/{name}"))
        .collect();
    files.sort();
    if files.is_empty() {
        println!("FAIL  {TESTS} holds no `*.test.mjs` file");
        return Err(());
    }
    Ok(files)
}

/// The addon built with the address sanitizer for the host, its `.node` copy's path
/// returned once it is shown to be instrumented.
///
/// Built for the host's triple named explicitly, so the flag reaches the
/// addon and the SDK and not the build scripts and procedural macros,
/// which run on the build machine and are no part of what Node loads.
fn build(root: &Path, host: &Platform) -> Result<PathBuf, ()> {
    step(
        Command::new("cargo")
            .args([NIGHTLY, "build", "--quiet", "-p", "teistro-node"])
            .args(["--target", host.triple])
            .env("RUSTFLAGS", flags(host))
            .current_dir(root),
        "",
        "the Node addon did not build under AddressSanitizer",
    )?;
    let addon = root
        .join("target")
        .join(host.triple)
        .join("debug")
        .join(host.shared(ADDON_STEM));
    let bytes = std::fs::read(&addon).map_err(|why| {
        println!("FAIL  {} could not be read: {why}", addon.display());
    })?;
    if !bytes.windows(REPORT.len()).any(|window| window == REPORT) {
        println!(
            "FAIL  {} calls no `__asan_report_*` function: it was built without the sanitizer",
            addon.display()
        );
        return Err(());
    }
    println!("ok    the Node addon is instrumented by AddressSanitizer");
    // Node loads a native addon only from a `.node` file, and reads any
    // other name as JavaScript.
    let named = addon.with_file_name(format!("{ADDON_STEM}_asan.node"));
    std::fs::copy(&addon, &named).map_err(|why| {
        println!("FAIL  {} could not be copied: {why}", named.display());
    })?;
    Ok(named)
}

/// The compiler flags that instrument the addon.
///
/// **Globals are left unpadded on macOS.** Instrumented there, the
/// initializer table napi's `ctor` writes into `__mod_init_func` no
/// longer points at functions, and Apple's linker refuses the addon
/// ("initializer pointer has no target"); `-Clink-dead-code` did not
/// help, and leaving globals out did. Heap and stack accesses are checked
/// either way, and Linux, the gate's CI row, keeps the globals' checks.
fn flags(host: &Platform) -> &'static str {
    if host.os == "darwin" {
        "-Zsanitizer=address -Cllvm-args=-asan-globals=0"
    } else {
        "-Zsanitizer=address"
    }
}

/// The sanitizer's runtime to load ahead of Node, by its path.
fn runtime(host: &Platform) -> Result<PathBuf, ()> {
    let path = if let Some(named) = std::env::var_os("TEISTRO_ASAN_RUNTIME") {
        PathBuf::from(named)
    } else if host.os == "darwin" {
        // The runtime the addon was linked against, from the toolchain
        // that built it.
        let sysroot = output(Command::new("rustc").args([NIGHTLY, "--print", "sysroot"]))?;
        Path::new(&sysroot)
            .join("lib/rustlib")
            .join(host.triple)
            .join("lib/librustc-nightly_rt.asan.dylib")
    } else {
        // The Rust toolchain ships its Linux runtime only as a static
        // archive, which an executable links and a shared library cannot,
        // so the runtime is clang's. `-print-file-name` echoes the bare
        // name back when clang has none, which the existence check below
        // refuses.
        let arch = if host.cpu == "arm64" {
            "aarch64"
        } else {
            "x86_64"
        };
        PathBuf::from(output(
            Command::new("clang").arg(format!("-print-file-name=libclang_rt.asan-{arch}.so")),
        )?)
    };
    if path.is_absolute() && path.is_file() {
        Ok(path)
    } else {
        println!(
            "FAIL  no AddressSanitizer runtime at `{}` (install clang, or set TEISTRO_ASAN_RUNTIME)",
            path.display()
        );
        Err(())
    }
}

/// What `command` printed, trimmed, when it ran and succeeded.
fn output(command: &mut Command) -> Result<String, ()> {
    match command.output() {
        Ok(out) if out.status.success() => {
            Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
        }
        Ok(out) => {
            println!(
                "FAIL  `{}` failed: {}",
                command.get_program().to_string_lossy(),
                String::from_utf8_lossy(&out.stderr).trim()
            );
            Err(())
        }
        Err(why) => {
            println!(
                "FAIL  `{}` did not start: {why}",
                command.get_program().to_string_lossy()
            );
            Err(())
        }
    }
}
