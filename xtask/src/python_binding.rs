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

use crate::binding::{FIXTURES, blob_fixtures, library, present, python_command, step};
use crate::binding_coverage;
use crate::examples::{Binding, Runtime};
use crate::platform::Platform;

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
/// The file whose whole purpose is to be wrong: every line marked
/// `# expect:` must be reported, which is how the Python half of Phase
/// 1's "a swapped latitude and longitude does not compile" is proved.
const WRONG: &str = "typecheck/wrong.py";
/// The pinned type checker, read by nothing else: the gate installs it
/// and no workflow has a step for it.
const REQUIREMENTS: &str = "bindings/python/typecheck/requirements.txt";

/// Whether a program exists under a name an operating system gives it.
///
/// Windows writes `mypy.exe` where a Unix writes `mypy`, and
/// `Path::exists` on a name without its extension misses it -- the same
/// shape as npm's `.cmd` shims, which `binding::tool` answers for a tool
/// that is spawned rather than looked for on disk.
fn exists_with_extension(path: &Path) -> bool {
    path.with_extension("exe").exists()
}

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
    let platform = Platform::host();
    let venv = root.join("bindings/python/.venv");
    let local = venv.join(platform.venv_bin()).join("mypy");
    // Installed from the pinned requirements beside the typecheck when
    // there is none, which is what `check-node` does with its own
    // compiler: a gate that skips because the machine has no checker is
    // a gate nobody notices skipping, and an unpinned one is a checker
    // that differs between machines.
    if !local.exists() && !exists_with_extension(&local) {
        let made = python_command(python)
            .args(["-m", "venv", ".venv"])
            .current_dir(root.join("bindings/python"))
            .status()
            .is_ok_and(|status| status.success());
        if made {
            let _ = python_command(venv.join(platform.venv_bin()).join("pip"))
                .args(["install", "--disable-pip-version-check", "--quiet", "-r"])
                .arg(root.join(REQUIREMENTS))
                .status();
        }
    }
    if local.exists() || exists_with_extension(&local) {
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

/// The coverage tool's pinned requirement (`03-design/binding-coverage.md`).
const COVERAGE_REQUIREMENTS: &str = "xtask/coverage-requirements.txt";

/// The suite's interpreter, and the report it owes when a run asked for
/// the bindings' coverage.
struct Suite {
    command: Command,
    report: Option<(PathBuf, PathBuf)>,
}

impl Suite {
    /// Writes coverage.py's LCOV report of the run, when one was asked.
    fn report(&self, package: &Path) -> Result<(), ()> {
        let Some((python, report)) = &self.report else {
            return Ok(());
        };
        step(
            python_command(python)
                .env("COVERAGE_FILE", report.with_extension("coverage"))
                .args(["-m", "coverage", "lcov", "--quiet", "-o"])
                .arg(report)
                .current_dir(package),
            &format!("{PACKAGE}'s coverage written to {}", report.display()),
            &format!("{PACKAGE}'s coverage was not written"),
        )
    }
}

/// `python -m unittest`, or under coverage.py, branches counted and the
/// package alone measured, when a run asked for the bindings' reports:
/// coverage.py is installed from its pin into the binding's own venv.
fn measured_suite(root: &Path, python: &str) -> Result<Suite, String> {
    let Some(dir) = binding_coverage::asked(root)? else {
        return Ok(Suite {
            command: python_command(python),
            report: None,
        });
    };
    let venv = root.join("bindings/python/.venv");
    let local = venv.join(Platform::host().venv_bin()).join("python");
    if !local.exists() && !exists_with_extension(&local) {
        step(
            python_command(python)
                .args(["-m", "venv", ".venv"])
                .current_dir(root.join("bindings/python")),
            "",
            "the binding's venv could not be made",
        )
        .map_err(|()| String::from("no venv for coverage.py"))?;
    }
    step(
        python_command(&local)
            .args([
                "-m",
                "pip",
                "install",
                "--disable-pip-version-check",
                "--quiet",
                "-r",
            ])
            .arg(root.join(COVERAGE_REQUIREMENTS)),
        "",
        "coverage.py could not be installed from its pin",
    )
    .map_err(|()| String::from("no coverage.py"))?;
    binding_coverage::name_tool(
        &dir,
        "python",
        "coverage.py",
        python_command(&local).args(["-m", "coverage", "--version"]),
    )?;
    // The data file beside the report, not in the package's tree.
    let data = dir.join("python.coverage");
    let mut command = python_command(&local);
    command
        .args(["-m", "coverage", "run", "--branch", "--source=teistro"])
        .env("COVERAGE_FILE", &data);
    Ok(Suite {
        command,
        report: Some((local, dir.join("python.lcov"))),
    })
}

/// Hands a feed the binding wrote to a parser that is not ours, at its pin
/// (`03-design/icalendar.md` §6). An install that fails is a failure: this
/// is the claim the feed holds to RFC 5545.
fn ical_black_box(root: &Path, python: &str, package: &Path, library: &Path) -> Result<(), ()> {
    let parser = root.join("target/python-blackbox");
    step(
        python_command(python)
            .args(["-m", "pip", "install", "--disable-pip-version-check"])
            .args(["--quiet", "--upgrade", "--target"])
            .arg(&parser)
            .arg("-r")
            .arg(root.join(PACKAGE).join("blackbox/requirements.txt")),
        "",
        "the feed's parser could not be installed from its pin",
    )?;
    let path = std::env::join_paths([package, parser.as_path()]).map_err(|_| ())?;
    step(
        python_command(python)
            .arg("blackbox/ical_feed.py")
            .env("TEISTRO_LIBRARY", library)
            .env("PYTHONPATH", path)
            .current_dir(package),
        "a feed reads whole in a parser that is not ours",
        "a parser that is not ours refused the feed",
    )
}

pub(crate) fn check(root: &Path) -> i32 {
    let python = interpreter();
    if !present(&python, "--version") {
        crate::skip::skip(format_args!(
            "no `{python}` on this machine; the Python binding's tests need it"
        ));
        return 0;
    }
    let package = root.join(PACKAGE);
    let fixtures = root.join(FIXTURES);
    let Ok(library) = library(root) else {
        return 1;
    };
    let mut suite = match measured_suite(root, &python) {
        Ok(suite) => suite,
        Err(why) => {
            println!("FAIL  {why}");
            return 1;
        }
    };
    let outcome = blob_fixtures(root, &fixtures)
        .and_then(|()| {
            step(
                suite
                    .command
                    .args(["-m", "unittest", "discover", "-s", "tests", "-t", "."])
                    .env("TEISTRO_LIBRARY", &library)
                    .env("TEISTRO_FIXTURES", &fixtures)
                    .env("PYTHONPATH", &package)
                    .current_dir(&package),
                &format!("{PACKAGE}/tests passes against the library it was built for"),
                &format!("{PACKAGE}/tests did not pass"),
            )
        })
        .and_then(|()| suite.report(&package))
        .and_then(|()| ical_black_box(root, &python, &package, &library))
        .and_then(|()| Binding::Python.run(root, &Runtime::of(root)).map(drop));
    if outcome.is_err() {
        return 1;
    }
    let Some(checker) = type_checker(root, &python) else {
        crate::skip::skip(format_args!(
            "{PACKAGE}: the pinned type checker is not installed and could not be (needs `{python} -m venv`; or set MYPY)"
        ));
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
