//! The Java binding's own tests (`03-design/java-binding.md` §11): the
//! generated FFM layer and the hand-written one compiled at the floor's
//! release with every lint an error, then the binding's tests run against
//! the real library on the JDK on the machine.
//!
//! Run by hand (`cargo xtask check-java`) and in the nightly matrix, with
//! the shared examples after the tests. The gate is skipped with a note
//! where no JDK is on the machine.
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
pub(crate) const PACKAGE: &str = "bindings/java";
/// Its source roots: what `cargo xtask gen ffi` writes, the typed messages
/// `cargo xtask gen intl` writes, and what is written by hand.
pub(crate) const SOURCES: [&str; 3] = ["generated", "messages", "src"];
/// The tests, patched into the module.
const TESTS: &str = "test";
/// The module every source is in.
pub(crate) const MODULE: &str = "com.teispace.teistro";
/// The test program.
const MAIN: &str = "com.teispace.teistro.BindingTest";
/// The release the binding is compiled for: FFM was final in Java 22.
pub(crate) const FLOOR: &str = "22";
/// Where the classes are written.
const CLASSES: &str = "target/java";

/// Every `.java` file under a directory.
pub(crate) fn java_files(dir: &Path, found: &mut Vec<PathBuf>) {
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

/// The files as a `javac` argument file, written into `classes` and
/// answered as its `@` argument.
///
/// The module's sources passed one by one outgrew the 32 767 characters
/// a Windows command line holds, and `javac` never started there. Each
/// path is quoted, with forward slashes, which `javac` reads on every
/// platform and which leave no backslash for its quoting to escape.
pub(crate) fn argfile(classes: &Path, name: &str, files: &[PathBuf]) -> Result<String, ()> {
    let file = classes.join(format!("{name}.args"));
    let mut text = String::new();
    for path in files {
        text.push('"');
        text.push_str(&path.to_string_lossy().replace('\\', "/"));
        text.push_str("\"\n");
    }
    std::fs::create_dir_all(classes)
        .and_then(|()| std::fs::write(&file, text))
        .map_err(|why| println!("FAIL  {}: {why}", file.display()))?;
    Ok(format!("@{}", file.display()))
}

/// `javac` at the floor's release with every lint an error.
pub(crate) fn javac(out: &Path) -> Command {
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

/// Compiles the module into `classes/main` at the floor's release with
/// every lint an error, and answers where.
fn compile_module(root: &Path, classes: &Path) -> Result<PathBuf, ()> {
    let package = root.join(PACKAGE);
    let main = classes.join("main");
    let _ = std::fs::remove_dir_all(classes);
    let mut sources = Vec::new();
    for dir in SOURCES {
        java_files(&package.join(dir), &mut sources);
    }
    if sources.is_empty() {
        println!("FAIL  {PACKAGE} has no sources; run `cargo xtask gen ffi`");
        return Err(());
    }
    let sources = argfile(classes, "main", &sources)?;
    step(
        javac(&main).arg(sources).current_dir(root),
        &format!("{PACKAGE} compiles at release {FLOOR} with every lint an error"),
        &format!("{PACKAGE} does not compile clean at release {FLOOR}"),
    )?;
    Ok(main)
}

/// Compiles the module into `classes/main` and one patched directory into
/// `classes/<name>`, each at the floor's release with every lint an error,
/// and answers the module's classes.
fn compile(root: &Path, classes: &Path, patch: &str) -> Result<PathBuf, ()> {
    let package = root.join(PACKAGE);
    let patched = classes.join(patch);
    let mut patch_sources = Vec::new();
    java_files(&package.join(patch), &mut patch_sources);
    if patch_sources.is_empty() {
        println!("FAIL  {PACKAGE}/{patch} holds no sources");
        return Err(());
    }
    let main = compile_module(root, classes)?;
    step(
        javac(&patched)
            .arg("--module-path")
            .arg(&main)
            .arg(format!(
                "--patch-module={MODULE}={}",
                package.join(patch).display()
            ))
            .arg(argfile(classes, patch, &patch_sources)?)
            .current_dir(root),
        "",
        &format!("{PACKAGE}/{patch} does not compile clean"),
    )?;
    Ok(main)
}

/// `java` running a class patched into the module against a named library.
fn patched(main: &Path, patch: &Path, class: &str, library: &Path) -> Command {
    let mut command = crate::binding::java_command();
    command
        .arg(format!("--enable-native-access={MODULE}"))
        .arg("--module-path")
        .arg(main)
        .arg(format!("--patch-module={MODULE}={}", patch.display()))
        .args(["-m", &format!("{MODULE}/{class}")])
        .env("TEISTRO_LIBRARY", library);
    command
}

/// Compiles the shared examples against the module as a consumer holds
/// it: on the module path, from the unnamed module, so an example reaches
/// only what the module exports. Each at the floor's release with every
/// lint an error, as the binding is.
pub(crate) fn compile_examples(root: &Path, examples: &[PathBuf]) -> Result<(), ()> {
    let classes = root.join(EXAMPLE_CLASSES);
    let main = compile_module(root, &classes)?;
    step(
        javac(&classes.join(EXAMPLES))
            .arg("--module-path")
            .arg(&main)
            .args(["--add-modules", MODULE])
            .arg(argfile(&classes, EXAMPLES, examples)?)
            .current_dir(root),
        "",
        &format!("{PACKAGE}/{EXAMPLES} does not compile clean"),
    )
}

/// The command that runs one compiled example, by its class, against the
/// named library, from the package's directory.
pub(crate) fn example(root: &Path, class: &str, library: &Path) -> Command {
    let classes = root.join(EXAMPLE_CLASSES);
    let mut command = crate::binding::java_command();
    command
        .arg(format!("--enable-native-access={MODULE}"))
        .arg("--module-path")
        .arg(classes.join("main"))
        .args(["--add-modules", MODULE])
        .arg("-cp")
        .arg(classes.join(EXAMPLES))
        .arg(class)
        .env("TEISTRO_LIBRARY", library)
        .current_dir(root.join(PACKAGE));
    command
}

/// The shared examples' directory.
const EXAMPLES: &str = "example";
/// Where the examples and the module they run against are compiled.
const EXAMPLE_CLASSES: &str = "target/java-examples";

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
    let outcome = outcome.and_then(|()| {
        crate::examples::Binding::Java
            .run(root, &crate::examples::Runtime::of(root))
            .map(drop)
    });
    i32::from(outcome.is_err())
}
