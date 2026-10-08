//! The Java binding's own tests (`03-design/java-binding.md` §11): the
//! generated FFM layer and the hand-written one compiled at the floor's
//! release with every lint an error, then the binding's tests run against
//! the real library on the JDK on the machine.
//!
//! Run by hand (`cargo xtask check-java`) and in the nightly matrix. The
//! gate is skipped with a note where no JDK is on the machine.
//!
//! The tests are patched into the module rather than compiled beside it,
//! so they reach its package-private parts as a test in the same package
//! would, and the library they run against is named outright: a named
//! library is the only one the loader tries, so the run cannot pass
//! against a stale build it found by itself.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::binding::{library, present, step};

/// The binding's tree.
const PACKAGE: &str = "bindings/java";
/// Its source roots: what `cargo xtask gen ffi` writes, and what is
/// written by hand.
const SOURCES: [&str; 2] = ["generated", "src"];
/// The tests, patched into the module.
const TESTS: &str = "test";
/// The module every source is in.
const MODULE: &str = "com.teispace.teistro";
/// The test program.
const MAIN: &str = "com.teispace.teistro.BindingTest";
/// The release the binding is compiled for: FFM was final in Java 22.
const FLOOR: &str = "22";
/// Where the classes are written.
const CLASSES: &str = "target/java";

/// Every `.java` file under a directory.
fn java_files(dir: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            java_files(&path, found);
        } else if path.extension().is_some_and(|ext| ext == "java") {
            found.push(path);
        }
    }
}

/// `javac` at the floor's release with every lint an error.
fn javac(out: &Path) -> Command {
    let mut command = Command::new("javac");
    command
        .args([
            "--release",
            FLOOR,
            "-Xlint:all",
            "-Werror",
            "-encoding",
            "UTF-8",
            "-d",
        ])
        .arg(out);
    command
}

pub(crate) fn check(root: &Path) -> i32 {
    if !present("javac", "--version") || !present("java", "--version") {
        crate::skip::skip(
            "no JDK on this machine; the Java binding's tests need `javac` and `java`",
        );
        return 0;
    }
    let package = root.join(PACKAGE);
    let classes = root.join(CLASSES);
    let main = classes.join("main");
    let tests = classes.join("test");
    let _ = std::fs::remove_dir_all(&classes);
    let Ok(library) = library(root) else {
        return 1;
    };
    let mut sources = Vec::new();
    for dir in SOURCES {
        java_files(&package.join(dir), &mut sources);
    }
    let mut test_sources = Vec::new();
    java_files(&package.join(TESTS), &mut test_sources);
    if sources.is_empty() || test_sources.is_empty() {
        println!("FAIL  {PACKAGE} has no sources or no tests; run `cargo xtask gen ffi`");
        return 1;
    }
    let outcome = step(
        javac(&main).args(&sources).current_dir(root),
        &format!("{PACKAGE} compiles at release {FLOOR} with every lint an error"),
        &format!("{PACKAGE} does not compile clean at release {FLOOR}"),
    )
    .and_then(|()| {
        step(
            javac(&tests)
                .arg("--module-path")
                .arg(&main)
                .arg(format!(
                    "--patch-module={MODULE}={}",
                    package.join(TESTS).display()
                ))
                .args(&test_sources)
                .current_dir(root),
            "",
            &format!("{PACKAGE}/{TESTS} does not compile clean"),
        )
    })
    .and_then(|()| {
        step(
            Command::new("java")
                .arg(format!("--enable-native-access={MODULE}"))
                .arg("--module-path")
                .arg(&main)
                .arg(format!("--patch-module={MODULE}={}", tests.display()))
                .args(["-m", &format!("{MODULE}/{MAIN}")])
                .env("TEISTRO_LIBRARY", &library)
                .current_dir(root),
            &format!("{PACKAGE}'s tests pass against the real library"),
            &format!("{PACKAGE}'s tests did not pass"),
        )
    });
    i32::from(outcome.is_err())
}
