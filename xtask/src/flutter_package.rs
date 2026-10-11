//! `check-flutter`: a Flutter app built on the release's own Android
//! slices, and the APK read back (`docs/03-design/flutter-build-hook.md`,
//! §5 step 6).
//!
//! The packages are staged as a release stages them, with a digest table
//! written from the mobile manifests in `target/dist`, so the hook checks
//! real digests. The example app is given a `prebuilt_dir` pointing at the
//! archives there, so nothing is fetched. `flutter build apk --release`
//! then runs the hook once per ABI, and the APK must hold each ABI's
//! library, stored uncompressed and 16 KB-aligned where the row requires
//! it, and passing the same floor check `package` held the slice to:
//! whatever Gradle did to the file on the way in (it strips it), it is
//! still the slice for that ABI, needing nothing beyond bionic.
//!
//! Needs the mobile slices (`cargo xtask package android-arm64`, and the
//! other two) and a Flutter SDK with Android's; skipped with a note
//! without Flutter (ADR-0014).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Map, Value, json};

use crate::binding::{present, step};
use crate::mobile::{MOBILE, Mobile, Os};
use crate::package::{DIST, copy_tree, manifest_name, prebuilt_table};

/// Where the packages and the app are staged.
const STAGE: &str = "target/flutter-check";

/// The APK a release build writes, under the app.
const APK: &str = "build/app/outputs/flutter-apk/app-release.apk";

/// The page size an aligned row's library must start on in the APK.
const PAGE: usize = 16 * 1024;

pub(crate) fn check(root: &Path) -> i32 {
    if !present("flutter", "--version") {
        crate::skip::skip("no `flutter` on this machine; check-flutter builds an app with it");
        return 0;
    }
    let version = crate::release::version(root);
    let dist = root.join(DIST);
    let rows: Vec<Mobile> = MOBILE
        .into_iter()
        .filter(|row| row.os == Os::Android)
        .collect();
    let mut mobile = Map::new();
    for row in &rows {
        let path = dist.join(manifest_name(&version, row.name));
        let Some(manifest) = fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        else {
            println!(
                "FAIL  no {} manifest at {}; build the slice first: `cargo xtask package {}`",
                row.name,
                path.display(),
                row.name
            );
            return 1;
        };
        mobile.insert(row.name.to_owned(), manifest);
    }
    let app = match stage(root, &version, &dist, &json!({ "mobile": mobile })) {
        Ok(app) => app,
        Err(err) => {
            println!("FAIL  the packages could not be staged for the app: {err}");
            return 1;
        }
    };
    let built = step(
        Command::new("flutter")
            .args([
                "create",
                "--platforms=android",
                "--project-name",
                "teistro_example",
                "--org",
                "com.teispace.teistro",
                ".",
            ])
            .current_dir(&app),
        "",
        "`flutter create` could not give the example app its Android project",
    )
    .and_then(|()| {
        // `create` adds a counter app's widget test, which tests a widget
        // this app does not have; the app is proved by the APK instead.
        fs::remove_dir_all(app.join("test")).or_else(|err| {
            if err.kind() == io::ErrorKind::NotFound {
                Ok(())
            } else {
                println!("FAIL  the template's test could not be removed: {err}");
                Err(())
            }
        })
    })
    .and_then(|()| {
        step(
            Command::new("flutter").arg("analyze").current_dir(&app),
            "the example app analyses clean against the staged packages",
            "the example app does not analyse clean",
        )
    })
    .and_then(|()| {
        step(
            Command::new("flutter")
                .args(["build", "apk", "--release"])
                .current_dir(&app),
            "`flutter build apk --release` ran the hook for every ABI",
            "`flutter build apk --release` failed",
        )
    });
    if built.is_err() {
        return 1;
    }
    match fs::read(app.join(APK)) {
        Ok(apk) => i32::from(inspect(&apk, &rows).is_err()),
        Err(err) => {
            println!("FAIL  no APK at {APK}: {err}");
            1
        }
    }
}

/// Stages `teistro` with a digest table written from `merged`, then the
/// hook package and its example beside it, the example told to read the
/// archives from `dist`; returns the example's directory.
fn stage(root: &Path, version: &str, dist: &Path, merged: &Value) -> io::Result<PathBuf> {
    let stage = root.join(STAGE);
    if stage.exists() {
        fs::remove_dir_all(&stage)?;
    }
    // The same directory names as the repository's, so that the hook
    // package's `path: ../dart` and the example's `path: ..` resolve to
    // the staged copies unchanged.
    let dart = stage.join("dart");
    let flutter = stage.join("flutter");
    for (from, to, parts) in [
        (
            root.join("bindings/dart"),
            &dart,
            &["lib", "pubspec.yaml", "analysis_options.yaml"][..],
        ),
        (
            root.join("bindings/flutter"),
            &flutter,
            &[
                "lib",
                "hook",
                "example",
                "pubspec.yaml",
                "analysis_options.yaml",
            ][..],
        ),
    ] {
        fs::create_dir_all(to)?;
        for part in parts {
            let source = from.join(part);
            if source.is_dir() {
                copy_tree(&source, &to.join(part))?;
            } else {
                fs::copy(&source, to.join(part))?;
            }
        }
    }
    fs::write(
        dart.join("lib/src/prebuilt.dart"),
        prebuilt_table(root, version, merged),
    )?;
    let app = flutter.join("example");
    let pubspec = app.join("pubspec.yaml");
    let mut text = fs::read_to_string(&pubspec)?;
    // A JSON string is a YAML string, quoted whatever the path holds.
    let quoted = serde_json::to_string(&dist.display().to_string()).map_err(io::Error::other)?;
    text.push_str("\nhooks:\n  user_defines:\n    teistro_flutter:\n      prebuilt_dir: ");
    text.push_str(&quoted);
    text.push('\n');
    fs::write(&pubspec, text)?;
    Ok(app)
}

/// Holds the APK to what the hook was to bundle: each row's library in
/// its ABI's directory and nowhere else, stored, aligned where the row
/// requires it, and passing the row's floor check.
fn inspect(apk: &[u8], rows: &[Mobile]) -> Result<(), ()> {
    let names = crate::zip::names(apk).map_err(|err| {
        println!("FAIL  the APK cannot be read: {err}");
    })?;
    let mut failures = 0;
    let expected: Vec<String> = rows
        .iter()
        .map(|row| format!("lib/{}/{}", row.bundle, row.library()))
        .collect();
    for name in names.iter().filter(|name| name.contains("teistro_ffi")) {
        if !expected.contains(name) {
            println!("FAIL  the APK holds {name}, which no row bundles");
            failures += 1;
        }
    }
    for (row, name) in rows.iter().zip(&expected) {
        match held(apk, *row, name) {
            Ok(said) => println!("ok    {name}: {said}"),
            Err(why) => {
                println!("FAIL  {name}: {why}");
                failures += 1;
            }
        }
    }
    if failures == 0 { Ok(()) } else { Err(()) }
}

/// One row's library in the APK, held to its row.
fn held(apk: &[u8], row: Mobile, name: &str) -> Result<String, String> {
    let (at, stored) = crate::zip::placed(apk, name)
        .map_err(|err| err.to_string())?
        .ok_or_else(|| format!("the APK holds no library for {}", row.name))?;
    if !stored {
        return Err("it is deflated, so the loader cannot map it from the APK".into());
    }
    if row.sixteen_k && at % PAGE != 0 {
        return Err(format!(
            "it starts at {at:#x}, not on a 16 KB page, which Play requires of {}",
            row.bundle
        ));
    }
    let file = crate::zip::read(apk, name)
        .map_err(|err| err.to_string())?
        .ok_or("the entry vanished between two reads")?;
    crate::mobile::check(row, &file)?;
    Ok(format!(
        "{} bytes, stored at {at:#x}, {}",
        file.len(),
        if row.sixteen_k {
            "page-aligned and the slice for its ABI"
        } else {
            "the slice for its ABI"
        }
    ))
}
