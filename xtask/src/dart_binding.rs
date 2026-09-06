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
const FIXTURES: &str = "target/tsrb";
/// The README's example, run so the two cannot drift.
const EXAMPLE: &str = "example/teistro_example.dart";

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
