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
use crate::binding_coverage;

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

/// `javadoc` at the floor's release with every doclint an error: what the
/// Maven package's javadoc jar is built with, so a link to a name that is
/// gone fails here as it fails there.
pub(crate) fn javadoc(out: &Path) -> Command {
    let mut command = Command::new("javadoc");
    command
        .args(["--release", FLOOR, "-encoding", "UTF-8", "-Xdoclint:all"])
        .args(["-Werror", "-quiet", "-d"])
        .arg(out);
    command
}

/// Compiles the module into `classes/main` at the floor's release with
/// every lint an error, documents it with every doclint an error, and
/// answers where.
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
        javac(&main).arg(&sources).current_dir(root),
        &format!("{PACKAGE} compiles at release {FLOOR} with every lint an error"),
        &format!("{PACKAGE} does not compile clean at release {FLOOR}"),
    )?;
    step(
        javadoc(&classes.join("javadoc"))
            .arg(sources)
            .current_dir(root),
        &format!("{PACKAGE}'s javadoc builds with every doclint an error"),
        &format!("{PACKAGE}'s javadoc does not build clean"),
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
    patched_under(main, patch, class, library, &[])
}

/// [`patched`] with JVM options before the module, such as an agent.
fn patched_under(
    main: &Path,
    patch: &Path,
    class: &str,
    library: &Path,
    options: &[String],
) -> Command {
    let mut command = crate::binding::java_command();
    command
        .args(options)
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

/// The FFI-cost harness (`03-design/ffi-cost.md`), compiled as the shared
/// examples are.
pub(crate) const BENCH: &str = "bindings/java/bench/FfiCost.java";

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

/// The Teimeris adapter's Java package.
const ADAPTER: &str = "adapters/ephemeris-teimeris/java";

/// The file whose whole purpose is to be wrong: every `// expect:` line
/// must be answered by an error on the statement after it, which is how
/// the Java half of "a swapped latitude and longitude does not compile"
/// and of "a request is typed" is proved.
const WRONG: &str = "typecheck/Wrong.java";

/// Compiles the file of wrong usages against the module and holds it to
/// what it expects: each `// expect: <text>` is answered by exactly one
/// error on the lines up to the next expectation, carrying that text in
/// its message or the detail javac indents under it, and no error falls
/// before the first.
fn wrong_usages(root: &Path, classes: &Path, main: &Path) -> Result<(), ()> {
    let file = root.join(PACKAGE).join(WRONG);
    let source = std::fs::read_to_string(&file).map_err(|e| {
        println!("FAIL  {PACKAGE}/{WRONG}: {e}");
    })?;
    let expected: Vec<(usize, &str)> = source
        .lines()
        .enumerate()
        .filter_map(|(at, line)| Some((at + 1, line.trim().strip_prefix("// expect:")?.trim())))
        .collect();
    let output = Command::new("javac")
        .args([
            "--release",
            FLOOR,
            "-encoding",
            "UTF-8",
            "-Xmaxerrs",
            "1000",
            "-d",
        ])
        .arg(classes.join("wrong"))
        .arg("--module-path")
        .arg(main)
        .args(["--add-modules", MODULE])
        .arg(&file)
        .current_dir(root)
        .output()
        .map_err(|e| println!("FAIL  {PACKAGE}/{WRONG} could not be compiled: {e}"))?;
    let errors = javac_errors(&String::from_utf8_lossy(&output.stderr));
    let failures = held_to(&expected, &errors);
    for failure in &failures {
        println!("FAIL  {PACKAGE}/{WRONG}: {failure}");
    }
    if failures.is_empty() && !output.status.success() {
        println!(
            "ok    {PACKAGE}/{WRONG}: {} wrong usage(s) do not compile",
            expected.len()
        );
        Ok(())
    } else {
        if failures.is_empty() {
            println!("FAIL  {PACKAGE}/{WRONG} compiled, and every line in it is wrong");
        }
        Err(())
    }
}

/// javac's errors, each its line and its whole block: the message and the
/// detail lines indented under it (`required:`, `reason: argument
/// mismatch; ...`).
fn javac_errors(report: &str) -> Vec<(usize, String)> {
    let mut errors: Vec<(usize, String)> = Vec::new();
    let mut open = false;
    for line in report.lines() {
        let head = line.split_once(": error: ").and_then(|(place, message)| {
            let (_, number) = place.rsplit_once(':')?;
            Some((number.parse::<usize>().ok()?, message))
        });
        if let Some((number, message)) = head {
            errors.push((number, message.to_owned()));
            open = true;
        } else if line.contains(": warning: ")
            || line.ends_with(" error")
            || line.ends_with(" errors")
        {
            open = false;
        } else if open {
            if let Some((_, block)) = errors.last_mut() {
                block.push('\n');
                block.push_str(line.trim());
            }
        }
    }
    errors
}

/// What is wrong with `errors` against `expected`, each as a sentence.
fn held_to(expected: &[(usize, &str)], errors: &[(usize, String)]) -> Vec<String> {
    let mut failures = Vec::new();
    for (index, (at, want)) in expected.iter().enumerate() {
        let until = expected.get(index + 1).map_or(usize::MAX, |next| next.0);
        let answered: Vec<&String> = errors
            .iter()
            .filter(|(line, _)| line > at && *line < until)
            .map(|(_, block)| block)
            .collect();
        match answered.as_slice() {
            [one] if one.contains(want) => {}
            [one] => failures.push(format!(
                "line {at} expects `{want}`, and javac said `{}`",
                one.lines().next().unwrap_or_default()
            )),
            [] => failures.push(format!("line {at} expects `{want}`, which compiled")),
            many => failures.push(format!(
                "line {at} expects one error, and javac reported {}",
                many.len()
            )),
        }
    }
    let first = expected.first().map_or(usize::MAX, |first| first.0);
    for (line, block) in errors.iter().filter(|(line, _)| *line < first) {
        failures.push(format!(
            "line {line} is no expected error: `{}`",
            block.lines().next().unwrap_or_default()
        ));
    }
    failures
}

/// Compiles the Teimeris adapter's module against the binding's, and its
/// typecheck consumer against both, each at the floor's release with every
/// lint an error: the gate the other bindings' adapters get from their own
/// type checkers. The consumer is compiled and never run, since it needs
/// the engine's data.
fn adapter(root: &Path, classes: &Path, main: &Path) -> Result<(), ()> {
    let package = root.join(ADAPTER);
    let module = classes.join("adapter");
    let mut sources = Vec::new();
    java_files(&package.join("src"), &mut sources);
    step(
        javac(&module)
            .arg("--module-path")
            .arg(main)
            .arg(argfile(classes, "adapter", &sources)?)
            .current_dir(root),
        "",
        &format!("{ADAPTER} does not compile clean"),
    )?;
    let mut consumer = Vec::new();
    java_files(&package.join("typecheck"), &mut consumer);
    let separator = if cfg!(windows) { ";" } else { ":" };
    step(
        javac(&classes.join("adapter-typecheck"))
            .arg("--module-path")
            .arg(format!("{}{separator}{}", main.display(), module.display()))
            .args(["--add-modules", "com.teispace.teistro.teimeris"])
            .arg(argfile(classes, "adapter-typecheck", &consumer)?)
            .current_dir(root),
        &format!("{ADAPTER} and its README's consumer compile against the binding"),
        &format!("{ADAPTER}'s typecheck consumer does not compile"),
    )
}

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
    let fixtures = root.join(crate::binding::FIXTURES);
    let coverage = match binding_coverage::asked(root).and_then(|dir| {
        dir.map(|dir| binding_coverage::Jacoco::fetched(root, &dir))
            .transpose()
    }) {
        Ok(coverage) => coverage,
        Err(why) => {
            println!("FAIL  {why}");
            return 1;
        }
    };
    let options = coverage
        .as_ref()
        .map_or_else(Vec::new, |jacoco| vec![jacoco.agent_option()]);
    let outcome = crate::binding::blob_fixtures(root, &fixtures)
        .and_then(|()| compile(root, &classes, TESTS))
        .and_then(|main| {
            step(
                patched_under(&main, &classes.join(TESTS), MAIN, &library, &options)
                    .env("TEISTRO_FIXTURES", &fixtures)
                    .current_dir(root),
                &format!("{PACKAGE}'s tests pass against the real library"),
                &format!("{PACKAGE}'s tests did not pass"),
            )?;
            if let Some(jacoco) = &coverage {
                jacoco.report(root, &main)?;
            }
            adapter(root, &classes, &main)?;
            wrong_usages(root, &classes, &main)?;
            // Run only by `cargo xtask measure-ffi-cost`, so compiled here
            // where every change to the binding meets it.
            compile_examples(root, &[root.join(BENCH)])
        });
    let outcome = outcome.and_then(|()| {
        crate::examples::Binding::Java
            .run(root, &crate::examples::Runtime::of(root))
            .map(drop)
    });
    i32::from(outcome.is_err())
}

#[cfg(test)]
mod tests {
    use super::{held_to, javac_errors};

    const REPORT: &str = "\
typecheck/Wrong.java:10: error: incompatible types: Latitude cannot be converted to Longitude
        new Observer(new Latitude(1), new Longitude(2), new Altitude(3));
                     ^
typecheck/Wrong.java:13: error: method kp in class Builder cannot be applied to given types;
        ChartOptions.builder().kp(Map.of(\"number\", 108));
                              ^
  required: KpRequest
  found:    Map<String,Integer>
  reason: argument mismatch; Map<String,Integer> cannot be converted to KpRequest
2 errors
";

    #[test]
    fn an_error_is_read_with_the_detail_under_it() {
        let errors = javac_errors(REPORT);
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].0, 10);
        assert!(
            errors[1].1.contains("cannot be converted to KpRequest"),
            "{:?}",
            errors[1]
        );
    }

    #[test]
    fn each_expectation_is_answered_by_one_error_after_it() {
        let errors = javac_errors(REPORT);
        let expected = [
            (9, "Latitude cannot be converted to Longitude"),
            (12, "cannot be converted to KpRequest"),
        ];
        assert_eq!(held_to(&expected, &errors), Vec::<String>::new());
        // A usage that compiled, one answered wrongly, and an error nothing expects.
        let failures = held_to(
            &[(9, "cannot be converted to Altitude"), (11, "x")],
            &errors,
        );
        assert_eq!(failures.len(), 2, "{failures:?}");
        let stray = held_to(&[(12, "cannot be converted to KpRequest")], &errors);
        assert_eq!(stray.len(), 1, "{stray:?}");
        assert!(
            stray[0].contains("line 10 is no expected error"),
            "{stray:?}"
        );
        let silent = held_to(&[(1, "anything")], &[]);
        assert!(silent[0].contains("which compiled"), "{silent:?}");
    }
}
