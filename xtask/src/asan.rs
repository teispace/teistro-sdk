//! The bindings' native layers under the address sanitizer: each binding's
//! own suite run unchanged against an instrumented build, so every memory
//! access it makes into the SDK is checked
//! (`05-testing/01-quality-bar.md`, "sanitizers").
//!
//! The `ffi` crate's tests already run under the sanitizer, but they call
//! the C ABI from Rust. A binding crosses it its own way: the Node addon's
//! generated glue copies Node's strings and arrays into the C structs a
//! call borrows and reads result blobs back out, and Python's `ctypes`
//! declarations lay out the same structs a second time. Neither runs
//! anywhere but inside its interpreter.
//!
//! **The interpreters are not rebuilt.** The addon and the C library are
//! compiled with `-Zsanitizer=address` and the interpreter is started with
//! the sanitizer's runtime loaded ahead of everything else (`LD_PRELOAD`,
//! or `DYLD_INSERT_LIBRARIES` on macOS), so the runtime owns `malloc` and
//! checks every access the SDK makes. The interpreters' own objects are
//! not checked, which is not what this gate is for. Leak detection is off,
//! because an interpreter frees little at exit and every report would be
//! its own; the leak tests in each binding hold the SDK to giving memory
//! back.
//!
//! **A run that checked nothing must not read as a clean one.** The gate
//! refuses a build that calls no `__asan_report_*` function, the runtime is
//! named by its path, so a missing runtime fails at load, and each binding
//! loads a sanitizer build only when `TEISTRO_SANITIZER` names it.
//!
//! Linux (glibc) and Apple-silicon macOS only: the runtime Linux loads is
//! clang's (`clang -print-file-name`, or `TEISTRO_ASAN_RUNTIME`), and the
//! one macOS loads is the Rust nightly's own. Run in verify's
//! `sanitizers` job; needs a nightly toolchain, Node and Python.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::binding::{
    FIXTURES, LIBRARY_STEM, blob_fixtures, present, python, python_command, step,
};
use crate::node_binding::{ADDON_STEM, TESTS};
use crate::platform::Platform;

/// The toolchain `-Zsanitizer` needs.
const NIGHTLY: &str = "+nightly";

/// What every instrumented access calls when it finds an error, and so
/// what an instrumented build must import.
const REPORT: &[u8] = b"__asan_report_";

/// The runtime's options: stop at the first error, and leave leaks to the
/// bindings' leak tests.
const OPTIONS: &str = "halt_on_error=1:detect_leaks=0";

/// The sanitizer each binding is told it is loading.
const SANITIZER: &str = "address";

/// The bindings this gate runs, by the name a caller picks one with.
const BINDINGS: [&str; 2] = ["node", "python"];

/// What every run under the sanitizer shares.
struct Instrumented {
    /// The variable that loads the runtime first.
    preload: &'static str,
    runtime: PathBuf,
    fixtures: PathBuf,
}

impl Instrumented {
    /// `command` with the runtime loaded and the build acknowledged.
    fn apply<'c>(&self, command: &'c mut Command) -> &'c mut Command {
        command
            .env(self.preload, &self.runtime)
            .env("ASAN_OPTIONS", OPTIONS)
            .env("TEISTRO_SANITIZER", SANITIZER)
            .env("TEISTRO_FIXTURES", &self.fixtures)
    }
}

/// `check-asan [BINDING...]`: the named bindings, or every one.
pub(crate) fn check(root: &Path, args: &[String]) -> i32 {
    let asked: Vec<&str> = if args.is_empty() {
        BINDINGS.to_vec()
    } else {
        args.iter().map(String::as_str).collect()
    };
    if let Some(unknown) = asked.iter().find(|name| !BINDINGS.contains(name)) {
        println!(
            "FAIL  no binding `{unknown}` runs under the sanitizer; the bindings are {}",
            BINDINGS.join(", ")
        );
        return 1;
    }
    let host = Platform::host();
    let preload = match (host.os, host.libc) {
        ("linux", Some("glibc")) => "LD_PRELOAD",
        ("darwin", _) if host.cpu == "arm64" => "DYLD_INSERT_LIBRARIES",
        _ => {
            crate::skip::skip(format_args!(
                "the bindings under ASan run on glibc Linux and Apple-silicon macOS, not {}",
                host.name()
            ));
            return 0;
        }
    };
    let Ok(runtime) = runtime(&host) else {
        return 1;
    };
    let fixtures = root.join(FIXTURES);
    if blob_fixtures(root, &fixtures).is_err() {
        return 1;
    }
    let shared = Instrumented {
        preload,
        runtime,
        fixtures,
    };
    let mut failed = 0;
    for binding in asked {
        let ran = match binding {
            "node" => node(root, &host, &shared),
            _ => python_suite(root, &host, &shared),
        };
        failed += i32::from(ran.is_err());
    }
    i32::from(failed > 0)
}

/// The Node suite against the addon built under the sanitizer.
fn node(root: &Path, host: &Platform, shared: &Instrumented) -> Result<(), ()> {
    if !present("node", "--version") {
        crate::skip::skip("no `node` on this machine; the addon under ASan needs it");
        return Ok(());
    }
    let addon = build(root, host, "teistro-node", ADDON_STEM, "the Node addon")?;
    let mut failed = false;
    for file in node_suite(root)? {
        // Each file in Node itself, not under `node --test`: the runner
        // starts a process per file, and macOS's loader takes
        // `DYLD_INSERT_LIBRARIES` out of the environment it hands on, so
        // the runner's children would load the addon without the runtime.
        failed |= step(
            shared
                .apply(&mut Command::new("node"))
                .arg(&file)
                .env("TEISTRO_ADDON", &addon)
                .current_dir(root.join("bindings/node")),
            &format!("bindings/node/{file} passes with the addon under AddressSanitizer"),
            &format!("bindings/node/{file} failed with the addon under AddressSanitizer"),
        )
        .is_err();
    }
    if failed { Err(()) } else { Ok(()) }
}

/// The suite's files, as `node_binding::SUITE` names them, relative to
/// the package.
fn node_suite(root: &Path) -> Result<Vec<String>, ()> {
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

/// The Python suite against the C library built under the sanitizer: one
/// interpreter for the whole suite, since `unittest` starts no other.
fn python_suite(root: &Path, host: &Platform, shared: &Instrumented) -> Result<(), ()> {
    let interpreter = python();
    if !present(&interpreter, "--version") {
        crate::skip::skip(format_args!(
            "no `{interpreter}` on this machine; the library under ASan needs it"
        ));
        return Ok(());
    }
    let library = build(root, host, "teistro-ffi", LIBRARY_STEM, "the C library")?;
    let package = root.join("bindings/python");
    let executable = python_executable(&interpreter)?;
    step(
        shared
            .apply(&mut python_command(&executable))
            .args(["-m", "unittest", "discover", "-s", "tests", "-t", "."])
            .env("TEISTRO_LIBRARY", &library)
            .env("PYTHONPATH", &package)
            .current_dir(&package),
        "bindings/python/tests passes with the library under AddressSanitizer",
        "bindings/python/tests failed with the library under AddressSanitizer",
    )
}

/// The binary that runs Python, rather than the one a command names.
///
/// **A framework build re-executes.** Homebrew's and python.org's macOS
/// `python3` start `Python.app` inside the framework, and macOS's loader
/// takes `DYLD_INSERT_LIBRARIES` out of what it hands on, so the library
/// loaded without the runtime ("Interceptors are not working"). The
/// framework's own binary is asked of the interpreter; anywhere else the
/// interpreter is its own binary.
fn python_executable(interpreter: &str) -> Result<String, ()> {
    output(python_command(interpreter).args([
        "-c",
        "import os, sys; app = os.path.join(sys.base_prefix, \
         'Resources/Python.app/Contents/MacOS/Python'); \
         print(app if os.path.isfile(app) else sys.executable)",
    ]))
}

/// `package`'s shared library built with the address sanitizer for the
/// host, its path returned once it is shown to be instrumented.
///
/// Built for the host's triple named explicitly, so the flag reaches the
/// library and the SDK and not the build scripts and procedural macros,
/// which run on the build machine and are no part of what is loaded.
fn build(
    root: &Path,
    host: &Platform,
    package: &str,
    stem: &str,
    what: &str,
) -> Result<PathBuf, ()> {
    step(
        Command::new("cargo")
            .args([NIGHTLY, "build", "--quiet", "-p", package])
            .args(["--target", host.triple])
            .env("RUSTFLAGS", flags(host))
            .current_dir(root),
        "",
        &format!("{what} did not build under AddressSanitizer"),
    )?;
    let built = root
        .join("target")
        .join(host.triple)
        .join("debug")
        .join(host.shared(stem));
    let bytes = std::fs::read(&built).map_err(|why| {
        println!("FAIL  {} could not be read: {why}", built.display());
    })?;
    if !bytes.windows(REPORT.len()).any(|window| window == REPORT) {
        println!(
            "FAIL  {} calls no `__asan_report_*` function: it was built without the sanitizer",
            built.display()
        );
        return Err(());
    }
    println!("ok    {what} is instrumented by AddressSanitizer");
    Ok(built)
}

/// The compiler flags that instrument a build.
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

/// The sanitizer's runtime to load ahead of the interpreter, by its path.
fn runtime(host: &Platform) -> Result<PathBuf, ()> {
    let path = if let Some(named) = std::env::var_os("TEISTRO_ASAN_RUNTIME") {
        PathBuf::from(named)
    } else if host.os == "darwin" {
        // The runtime the builds were linked against, from the toolchain
        // that built them.
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
