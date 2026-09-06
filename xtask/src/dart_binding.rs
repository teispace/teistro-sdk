//! The Dart binding's own tests: the generated declarations and the
//! ergonomic layer against the real library, the generated decoders
//! against blobs the library produced, and the package analysed at the
//! strictness `analysis_options.yaml` sets (ADR-0023).
//!
//! Run by hand (`cargo xtask check-dart`) and in the nightly matrix; the
//! fast check stays Rust-only (ADR-0014). The whole gate is skipped with a
//! note when no Dart toolchain is on the machine.
//!
//! The generated files carry `// dart format off`, so `dart format` leaves
//! them as the generator laid them out and the format check covers the
//! hand-written layer and the tests.

use std::path::Path;
use std::process::Command;

use crate::binding::{blob_fixtures, library, present, step};

const PACKAGE: &str = "bindings/dart";
/// The file whose whole purpose is to be wrong: every line marked
/// `// expect:` must be reported, which is how the Dart half of Phase 1's
/// "a swapped latitude and longitude does not compile" is proved.
const WRONG: &str = "typecheck/wrong.dart";
const FIXTURES: &str = "target/tsrb";
/// The README's example, run so the two cannot drift.
const EXAMPLE: &str = "example/teistro_example.dart";

/// Analyses the file of wrong usages and holds it to what it expects:
/// every `// expect: <text>` line must be answered by an error carrying
/// that text, and no error may go unexpected.
fn wrong_usages(package: &Path) -> Result<(), ()> {
    let source = std::fs::read_to_string(package.join(WRONG)).map_err(|e| {
        println!("FAIL  {PACKAGE}/{WRONG}: {e}");
    })?;
    let expected: Vec<&str> = source
        .lines()
        .filter_map(|line| line.trim().strip_prefix("// expect:"))
        .map(str::trim)
        .collect();
    let output = Command::new("dart")
        .args(["analyze", WRONG])
        .current_dir(package)
        .output()
        .map_err(|e| {
            println!("FAIL  {PACKAGE}/{WRONG} could not be analysed: {e}");
        })?;
    let report = String::from_utf8_lossy(&output.stdout);
    let errors: Vec<&str> = report
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("error -"))
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
            "FAIL  {PACKAGE}/{WRONG} expects {} error(s) and got {}",
            expected.len(),
            errors.len()
        );
        failures += 1;
    }
    if failures > 0 {
        return Err(());
    }
    println!(
        "ok    {PACKAGE}/{WRONG}: {} wrong usage(s) do not compile",
        expected.len()
    );
    Ok(())
}

pub(crate) fn check(root: &Path) -> i32 {
    if !present("dart", "--version") {
        eprintln!("no `dart` on this machine; the Dart binding's tests need it");
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
                Command::new("dart")
                    .args(["pub", "get"])
                    .current_dir(&package),
                "",
                &format!("{PACKAGE}: `dart pub get` failed"),
            )
        })
        .and_then(|()| {
            step(
                Command::new("dart")
                    .args(["analyze", "--fatal-infos"])
                    .current_dir(&package),
                &format!("{PACKAGE} analyses clean"),
                &format!("{PACKAGE} does not analyse clean"),
            )
        })
        .and_then(|()| {
            step(
                Command::new("dart")
                    .args(["format", "--set-exit-if-changed", "."])
                    .current_dir(&package),
                &format!("{PACKAGE} is formatted"),
                &format!("{PACKAGE} is not formatted; run `dart format .`"),
            )
        })
        .and_then(|()| wrong_usages(&package))
        .and_then(|()| {
            step(
                Command::new("dart")
                    .args(["run", EXAMPLE])
                    .env("TEISTRO_LIBRARY", &library)
                    .current_dir(&package),
                &format!("{PACKAGE}/{EXAMPLE} runs"),
                &format!("{PACKAGE}/{EXAMPLE} did not run"),
            )
        })
        .and_then(|()| {
            step(
                Command::new("dart")
                    .arg("test")
                    .env("TEISTRO_LIBRARY", &library)
                    .env("TEISTRO_FIXTURES", &fixtures)
                    .current_dir(&package),
                &format!("{PACKAGE}/test reads what the library produced"),
                &format!("{PACKAGE}/test did not pass"),
            )
        });
    i32::from(outcome.is_err())
}
