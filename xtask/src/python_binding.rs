//! The Python binding's own tests: the generated `ctypes` layer and the
//! ergonomic layer against the real library, the generated decoders
//! against blobs the library produced, the struct sizes against what the
//! interpreter lays out, and the package type-checked at the strictness
//! `pyproject.toml` sets (ADR-0023).
//!
//! Run by hand (`cargo xtask check-python`) and in the nightly matrix;
//! the fast check stays Rust-only (ADR-0014). The whole gate is skipped
//! with a note when no Python is on the machine, and the type-check step
//! alone is skipped when no checker is, because a type-check needs one
//! and the tests do not.
//!
//! The size step is the one this binding has and the others do not. The C
//! header asserts every struct's size at compile time; a `ctypes`
//! declaration is trusted, so the only place the question can be settled
//! is on the machine the library was really built on
//! (`03-design/binding-surface-measured.md` §3).

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::binding::{blob_fixtures, library, present, python_command, step};

const PACKAGE: &str = "bindings/python";
/// The Teimeris adapter's own package, which the SDK does not depend on
/// and which depends on the SDK.
///
/// Type-checked here rather than in a gate of its own, because what it
/// needs is this ecosystem's checker and this ecosystem's strictness: an
/// adapter package that did not type-check against the SDK's own
/// declarations would be a broken package however green the SDK's gate
/// was. Most of what it holds is the generated façade (ADR-0030).
const ADAPTER: &str = "adapters/ephemeris-teimeris/python";
const FIXTURES: &str = "target/tsrb";
/// The file whose whole purpose is to be wrong: every line marked
/// `# expect:` must be reported, which is how the Python half of Phase
/// 1's "a swapped latitude and longitude does not compile" is proved.
const WRONG: &str = "typecheck/wrong.py";
/// Where the examples live. **Every** file there is run, so a scenario
/// added to the directory is gated by having been added — the failure a
/// list in this file would eventually have is that someone writes an
/// example and forgets to list it.
const EXAMPLES: &str = "example";

/// The interpreter to use: `PYTHON` when the environment names one, else
/// `python3`.
fn interpreter() -> String {
    crate::binding::python()
}

/// The type checker, when the machine has one: `MYPY`, a local install
/// beside the package, or one the interpreter can import.
fn type_checker(root: &Path, python: &str) -> Option<(String, Vec<String>)> {
    if let Ok(mypy) = std::env::var("MYPY") {
        return Some((mypy, Vec::new()));
    }
    let local: PathBuf = root.join("bindings/python/.venv/bin/mypy");
    if local.exists() {
        return Some((local.display().to_string(), Vec::new()));
    }
    let importable = python_command(python)
        .args(["-c", "import mypy"])
        .output()
        .is_ok_and(|output| output.status.success());
    importable.then(|| {
        (
            python.to_string(),
            ["-m", "mypy"].iter().map(|s| (*s).to_string()).collect(),
        )
    })
}

/// Type-checks the file of wrong usages and holds it to what it expects:
/// every `# expect: <text>` line must be answered by an error carrying
/// that text, and no error may go unexpected.
fn wrong_usages(package: &Path, checker: &(String, Vec<String>)) -> Result<(), ()> {
    let source = std::fs::read_to_string(package.join(WRONG)).map_err(|e| {
        println!("FAIL  {PACKAGE}/{WRONG}: {e}");
    })?;
    let expected: Vec<&str> = source
        .lines()
        .filter_map(|line| line.trim().strip_prefix("# expect:"))
        .map(str::trim)
        .collect();
    let output = Command::new(&checker.0)
        .args(&checker.1)
        .args(["--no-error-summary", "--no-color-output", WRONG])
        .current_dir(package)
        .output()
        .map_err(|e| {
            println!("FAIL  {PACKAGE}/{WRONG} could not be checked: {e}");
        })?;
    let report = String::from_utf8_lossy(&output.stdout);
    let errors: Vec<&str> = report
        .lines()
        .map(str::trim)
        .filter(|line| line.contains(" error: "))
        .collect();
    let mut failures = 0;
    for want in &expected {
        if !errors.iter().any(|error| error.contains(want)) {
            println!("FAIL  {PACKAGE}/{WRONG} expects `{want}`, which was not reported");
            failures += 1;
        }
    }
    if errors.len() != expected.len() {
        println!(
            "FAIL  {PACKAGE}/{WRONG} expects {} error(s) and got {}:",
            expected.len(),
            errors.len()
        );
        for error in &errors {
            println!("      {error}");
        }
        failures += 1;
    }
    if failures > 0 {
        return Err(());
    }
    println!(
        "ok    {PACKAGE}/{WRONG}: {} wrong usage(s) do not type-check",
        expected.len()
    );
    Ok(())
}

/// Runs every example, in name order, and says how many.
///
/// An example is a program a reader is invited to copy, so it is held to
/// the same bar as a test: it must run, against the library this build
/// produced, and its output is shown when it does not.
fn examples(package: &Path, python: &str, library: &Path) -> Result<(), ()> {
    let directory = package.join(EXAMPLES);
    let mut found: Vec<PathBuf> = std::fs::read_dir(&directory)
        .map_err(|e| println!("FAIL  {PACKAGE}/{EXAMPLES}: {e}"))?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|kind| kind == "py"))
        .collect();
    found.sort();
    if found.is_empty() {
        println!("FAIL  {PACKAGE}/{EXAMPLES} holds no examples");
        return Err(());
    }
    for example in &found {
        let name = example
            .file_name()
            .map_or_else(String::new, |name| name.to_string_lossy().to_string());
        step(
            python_command(python)
                .arg(example)
                .env("TEISTRO_LIBRARY", library)
                .env("PYTHONPATH", package)
                .current_dir(package),
            "",
            &format!("{PACKAGE}/{EXAMPLES}/{name} did not run"),
        )?;
    }
    println!("ok    {PACKAGE}/{EXAMPLES}: {} example(s) run", found.len());
    Ok(())
}

pub(crate) fn check(root: &Path) -> i32 {
    let python = interpreter();
    if !present(&python, "--version") {
        eprintln!("no `{python}` on this machine; the Python binding's tests need it");
        return 0;
    }
    let package = root.join(PACKAGE);
    let fixtures = root.join(FIXTURES);
    let Ok(library) = library(root) else {
        return 1;
    };
    let outcome = blob_fixtures(root, &fixtures)
        .and_then(|()| {
            step(
                python_command(&python)
                    .args(["-m", "unittest", "discover", "-s", "tests", "-t", "."])
                    .env("TEISTRO_LIBRARY", &library)
                    .env("TEISTRO_FIXTURES", &fixtures)
                    .env("PYTHONPATH", &package)
                    .current_dir(&package),
                &format!("{PACKAGE}/tests passes against the library it was built for"),
                &format!("{PACKAGE}/tests did not pass"),
            )
        })
        .and_then(|()| examples(&package, &python, &library));
    if outcome.is_err() {
        return 1;
    }
    let Some(checker) = type_checker(root, &python) else {
        println!("skip  {PACKAGE}: no type checker (set MYPY, or `{python} -m pip install mypy`)");
        return 0;
    };
    let outcome = wrong_usages(&package, &checker)
        .and_then(|()| {
            step(
                Command::new(&checker.0)
                    .args(&checker.1)
                    .arg("--no-error-summary")
                    .current_dir(&package),
                &format!("{PACKAGE} type-checks in strict mode"),
                &format!("{PACKAGE} does not type-check"),
            )
        })
        .and_then(|()| {
            // `MYPYPATH` rather than an install: the SDK is a sibling
            // directory here, and the adapter depends on it by path for
            // as long as neither is published.
            step(
                Command::new(&checker.0)
                    .args(&checker.1)
                    .arg("--no-error-summary")
                    .env("MYPYPATH", package.as_os_str())
                    .current_dir(root.join(ADAPTER)),
                &format!("{ADAPTER}: the typed engine façade composes with the SDK"),
                &format!("{ADAPTER} does not type-check"),
            )
        });
    i32::from(outcome.is_err())
}
