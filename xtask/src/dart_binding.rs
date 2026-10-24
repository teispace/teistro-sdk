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

use crate::binding::{FIXTURES, blob_fixtures, library, present, step};
use crate::examples::{Binding, Runtime};

const PACKAGE: &str = "bindings/dart";
/// The Teimeris adapter's own package, which the SDK does not depend on
/// and which depends on the SDK.
///
/// Analysed here rather than in a gate of its own, because what it needs
/// is this ecosystem's analyser and this ecosystem's strictness: an
/// adapter package that did not resolve against the SDK's own types
/// would be a broken package however green the SDK's gate was. Most of
/// what it holds is the generated façade (ADR-0030).
const ADAPTER: &str = "adapters/ephemeris-teimeris/dart";
/// The Flutter build-hook package (`docs/03-design/flutter-build-hook.md`),
/// plain Dart, so this gate's toolchain is all it needs: the hook's
/// platform names, floors, cache and refusals, against the public
/// `package:teistro/prebuilt.dart` it reads its table and its check from.
const FLUTTER: &str = "bindings/flutter";
/// The Teimeris adapter's Flutter package: its hook is the SDK's bundler
/// with the adapter's release, and its `teimeris()` names the library the
/// loader opens inside an app.
const FLUTTER_ADAPTER: &str = "adapters/ephemeris-teimeris/flutter";
/// The file whose whole purpose is to be wrong: every line marked
/// `// expect:` must be reported, which is how the Dart half of Phase 1's
/// "a swapped latitude and longitude does not compile" is proved.
const WRONG: &str = "typecheck/wrong.dart";
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

/// A package beside the binding, resolved, analysed and formatted, and its
/// own tests run when it has them.
fn companion(root: &Path, name: &str, analysed: &str, tests: Option<&str>) -> Result<(), ()> {
    let package = root.join(name);
    // Not the example's: an example may be a Flutter app, which only
    // `flutter pub` resolves, and this gate is plain Dart's.
    step(
        Command::new("dart")
            .args(["pub", "get", "--no-example"])
            .current_dir(&package),
        "",
        &format!("{name}: `dart pub get` failed"),
    )
    .and_then(|()| {
        step(
            Command::new("dart")
                .args(["analyze", "--fatal-infos"])
                .current_dir(&package),
            &format!("{name}: {analysed}"),
            &format!("{name} does not analyse clean"),
        )
    })
    .and_then(|()| {
        step(
            Command::new("dart")
                .args(["format", "--set-exit-if-changed", "."])
                .current_dir(&package),
            &format!("{name} is formatted"),
            &format!("{name} is not formatted; run `dart format .`"),
        )
    })
    .and_then(|()| match tests {
        Some(said) => step(
            Command::new("dart").arg("test").current_dir(&package),
            &format!("{name}/test: {said}"),
            &format!("{name}/test did not pass"),
        ),
        None => Ok(()),
    })
}

/// The adapter package: resolved, analysed and formatted.
///
/// Its façade is generated from the engine's own description, so what
/// this holds is that the generated extension composes with the SDK's
/// `Engine` — which nothing else can say, because the SDK does not know
/// the adapter exists.
fn adapter(root: &Path) -> Result<(), ()> {
    companion(
        root,
        ADAPTER,
        "the typed engine façade composes with the SDK",
        None,
    )
}

/// The Flutter hook package: what a build bundles, and every refusal.
fn flutter_hook(root: &Path) -> Result<(), ()> {
    companion(
        root,
        FLUTTER,
        "the build hook composes with the SDK's prebuilt table",
        Some("the hook names, checks, caches and refuses slices"),
    )
}

/// The adapter's Flutter package: its release and what each platform's
/// loader is handed.
fn flutter_adapter(root: &Path) -> Result<(), ()> {
    companion(
        root,
        FLUTTER_ADAPTER,
        "the adapter's hook composes with the SDK's bundler",
        Some("the adapter opens embedded, by the name each loader finds"),
    )
}

pub(crate) fn check(root: &Path) -> i32 {
    if !present("dart", "--version") {
        crate::skip::skip("no `dart` on this machine; the Dart binding's tests need it");
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
        .and_then(|()| adapter(root))
        .and_then(|()| flutter_hook(root))
        .and_then(|()| flutter_adapter(root))
        .and_then(|()| wrong_usages(&package))
        .and_then(|()| Binding::Dart.run(root, &Runtime::of(root)).map(drop))
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
