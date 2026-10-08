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
/// Its source roots: what `cargo xtask gen ffi` writes, the typed messages
/// `cargo xtask gen intl` writes, and what is written by hand.
const SOURCES: [&str; 3] = ["generated", "messages", "src"];
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

/// Compiles the module into `classes/main` and one patched directory into
/// `classes/<name>`, each at the floor's release with every lint an error,
/// and answers the module's classes.
fn compile(root: &Path, classes: &Path, patch: &str) -> Result<PathBuf, ()> {
    let package = root.join(PACKAGE);
    let main = classes.join("main");
    let patched = classes.join(patch);
    let _ = std::fs::remove_dir_all(classes);
    let mut sources = Vec::new();
    for dir in SOURCES {
        java_files(&package.join(dir), &mut sources);
    }
    let mut patch_sources = Vec::new();
    java_files(&package.join(patch), &mut patch_sources);
    if sources.is_empty() || patch_sources.is_empty() {
        println!("FAIL  {PACKAGE} has no sources or no {patch}; run `cargo xtask gen ffi`");
        return Err(());
    }
    step(
        javac(&main).args(&sources).current_dir(root),
        &format!("{PACKAGE} compiles at release {FLOOR} with every lint an error"),
        &format!("{PACKAGE} does not compile clean at release {FLOOR}"),
    )?;
    step(
        javac(&patched)
            .arg("--module-path")
            .arg(&main)
            .arg(format!(
                "--patch-module={MODULE}={}",
                package.join(patch).display()
            ))
            .args(&patch_sources)
            .current_dir(root),
        "",
        &format!("{PACKAGE}/{patch} does not compile clean"),
    )?;
    Ok(main)
}

/// `java` running a class patched into the module against a named library.
fn patched(main: &Path, patch: &Path, class: &str, library: &Path) -> Command {
    let mut command = Command::new("java");
    command
        .arg(format!("--enable-native-access={MODULE}"))
        .arg("--module-path")
        .arg(main)
        .arg(format!("--patch-module={MODULE}={}", patch.display()))
        .args(["-m", &format!("{MODULE}/{class}")])
        .env("TEISTRO_LIBRARY", library);
    command
}

/// The parity runner, compiled and ready to run: `Parity.java` patched
/// into the module as the tests are, so it walks the scenario through the
/// binding a consumer would hold. `None` when it does not compile, which
/// the parity gate counts as a runner tried and failed.
pub(crate) fn parity(root: &Path, library: &Path) -> Option<Command> {
    let classes = root.join(PARITY_CLASSES);
    let main = compile(root, &classes, PARITY).ok()?;
    let mut command = patched(&main, &classes.join(PARITY), PARITY_MAIN, library);
    command.current_dir(root);
    Some(command)
}

/// The parity runner's directory, patched into the module.
const PARITY: &str = "parity";
/// The parity program.
const PARITY_MAIN: &str = "com.teispace.teistro.ParityRunner";
/// Where the parity runner's classes are written, apart from the tests'.
const PARITY_CLASSES: &str = "target/java-parity";

pub(crate) fn check(root: &Path) -> i32 {
    if !present("javac", "--version") || !present("java", "--version") {
        crate::skip::skip(
            "no JDK on this machine; the Java binding's tests need `javac` and `java`",
        );
        return 0;
    }
    let classes = root.join(CLASSES);
    let Ok(library) = library(root) else {
        return 1;
    };
    let outcome = compile(root, &classes, TESTS).and_then(|main| {
        step(
            patched(&main, &classes.join(TESTS), MAIN, &library).current_dir(root),
            &format!("{PACKAGE}'s tests pass against the real library"),
            &format!("{PACKAGE}'s tests did not pass"),
        )
    });
    i32::from(outcome.is_err())
}
