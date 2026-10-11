//! `check-flutter [android|ios]`: a Flutter app built on the release's
//! own mobile slices, and what it bundled read back (`docs/03-design/flutter-build-hook.md`,
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
//! On macOS with Xcode the app is given its iOS project too, and
//! `flutter build ios --release --no-codesign` runs the hook for the
//! device: the app's frameworks must hold one binary named for the
//! library, and it must be the `ios-arm64` slice at its floor.
//!
//! Needs the mobile slices (`cargo xtask package android-arm64`, the
//! other two, and `ios-arm64` on macOS) and a Flutter SDK with Android's;
//! skipped with a note without Flutter (ADR-0014).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Map, Value, json};

use crate::binding::{LIBRARY_STEM, present, step};
use crate::mobile::{MOBILE, Mobile, Os};
use crate::package::{DIST, copy_tree, manifest_name, prebuilt_table};

/// Where the packages and the app are staged.
const STAGE: &str = "target/flutter-check";

/// The APK a release build writes, under the app.
const APK: &str = "build/app/outputs/flutter-apk/app-release.apk";

/// The iOS row a device build bundles, and where the built app keeps its
/// frameworks, under the app.
const IOS_DEVICE: &str = "ios-arm64";
const IOS_FRAMEWORKS: &str = "build/ios/iphoneos/Runner.app/Frameworks";

/// The page size an aligned row's library must start on in the APK.
const PAGE: usize = 16 * 1024;

/// Which apps a run builds: both on a Mac with Xcode unless one is named,
/// Android alone elsewhere.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Targets {
    android: bool,
    ios: bool,
}

impl Targets {
    fn of(args: &[String], xcode: bool) -> Result<Targets, String> {
        match args {
            [] => Ok(Targets {
                android: true,
                ios: xcode,
            }),
            [one] if one == "android" => Ok(Targets {
                android: true,
                ios: false,
            }),
            [one] if one == "ios" => Ok(Targets {
                android: false,
                ios: true,
            }),
            _ => Err(format!(
                "`check-flutter` takes `android` or `ios`, or nothing for every app this host builds, not `{}`",
                args.join(" ")
            )),
        }
    }

    /// `flutter create`'s `--platforms`.
    fn platforms(self) -> String {
        let named: Vec<&str> = [(self.android, "android"), (self.ios, "ios")]
            .into_iter()
            .filter_map(|(wanted, name)| wanted.then_some(name))
            .collect();
        format!("--platforms={}", named.join(","))
    }

    /// The rows whose slices the apps bundle.
    fn rows(self) -> Vec<Mobile> {
        MOBILE
            .into_iter()
            .filter(|row| {
                (self.android && row.os == Os::Android) || (self.ios && row.name == IOS_DEVICE)
            })
            .collect()
    }
}

pub(crate) fn check(root: &Path, args: &[String]) -> i32 {
    let xcode = cfg!(target_os = "macos") && present("xcodebuild", "-version");
    let targets = match Targets::of(args, xcode) {
        Ok(targets) => targets,
        Err(why) => {
            println!("FAIL  {why}");
            return 1;
        }
    };
    if !present("flutter", "--version") {
        crate::skip::skip("no `flutter` on this machine; check-flutter builds an app with it");
        return 0;
    }
    if targets.ios && !xcode {
        crate::skip::skip("the iOS app is built on macOS with Xcode, and this host has neither");
        return 0;
    }
    let version = crate::release::version(root);
    let dist = root.join(DIST);
    let rows = targets.rows();
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
    let created = step(
        Command::new("flutter")
            .args([
                "create",
                &targets.platforms(),
                "--project-name",
                "teistro_example",
                "--org",
                "com.teispace.teistro",
                ".",
            ])
            .current_dir(&app),
        "",
        "`flutter create` could not give the example app its platform projects",
    )
    .and_then(|()| {
        // `create` adds a counter app's widget test, which tests a widget
        // this app does not have; the app is proved by what it bundles.
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
    });
    if created.is_err() {
        return 1;
    }
    let mut failed = false;
    if targets.android {
        let android: Vec<Mobile> = rows
            .iter()
            .copied()
            .filter(|row| row.os == Os::Android)
            .collect();
        failed |= android_app(&app, &android).is_err();
    }
    if targets.ios {
        failed |= ios_app(&app, &rows).is_err();
    }
    i32::from(failed)
}

/// The Android app built for release, and the APK held to its rows.
fn android_app(app: &Path, rows: &[Mobile]) -> Result<(), ()> {
    step(
        Command::new("flutter")
            .args(["build", "apk", "--release"])
            .current_dir(app),
        "`flutter build apk --release` ran the hook for every ABI",
        "`flutter build apk --release` failed",
    )?;
    let apk = fs::read(app.join(APK)).map_err(|err| {
        println!("FAIL  no APK at {APK}: {err}");
    })?;
    inspect(&apk, rows)
}

/// The iOS app built for a device, unsigned, and the library the hook
/// bundled read back out of it: one framework holding it, and that
/// framework's binary the device slice, held to its floor as `package`
/// held it.
fn ios_app(app: &Path, rows: &[Mobile]) -> Result<(), ()> {
    let Some(row) = rows.iter().find(|row| row.name == IOS_DEVICE) else {
        println!("FAIL  no {IOS_DEVICE} row to hold the iOS app to");
        return Err(());
    };
    step(
        Command::new("flutter")
            .args(["build", "ios", "--release", "--no-codesign"])
            .current_dir(app),
        "`flutter build ios --release --no-codesign` ran the hook for the device",
        "`flutter build ios --release --no-codesign` failed",
    )?;
    let frameworks = app.join(IOS_FRAMEWORKS);
    let mut found = Vec::new();
    collect_named(&frameworks, LIBRARY_STEM, &mut found);
    let binaries: Vec<&PathBuf> = found
        .iter()
        .filter(|path| path.is_file() && path.extension().is_none())
        .collect();
    let [binary] = binaries[..] else {
        println!(
            "FAIL  {IOS_FRAMEWORKS} holds {} binaries named for {LIBRARY_STEM}, not one: {binaries:?}",
            binaries.len()
        );
        return Err(());
    };
    let bytes = fs::read(binary).map_err(|err| {
        println!("FAIL  {} cannot be read: {err}", binary.display());
    })?;
    let shown = binary.strip_prefix(app).unwrap_or(binary).display();
    match arm64_slice(&bytes).and_then(|slice| crate::mobile::check(*row, slice).map(|()| slice)) {
        Ok(slice) => {
            println!(
                "ok    {shown}: {} bytes, a universal file of the {IOS_DEVICE} slice alone, at its floor",
                slice.len()
            );
            Ok(())
        }
        Err(why) => {
            println!("FAIL  {shown}: {why}");
            Err(())
        }
    }
}

/// `CPU_TYPE_ARM64`, as a universal file's header names it.
const CPU_ARM64: u32 = 0x0100_000c;

/// The arm64 slice of the binary Xcode leaves in a framework: a
/// universal file (`lipo`'s, 32- or 64-bit offsets) that must hold that
/// one slice and nothing else, since the device has nothing else to run.
fn arm64_slice(file: &[u8]) -> Result<&[u8], String> {
    let word = |at: usize| -> Result<u32, String> {
        file.get(at..at + 4)
            .map(|b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
            .ok_or_else(|| format!("the universal header is cut short at {at}"))
    };
    let wide = match word(0)? {
        0xcafe_babe => false,
        0xcafe_babf => true,
        _ => {
            return Err(
                "not a universal file, which is how Xcode leaves a framework's binary".into(),
            );
        }
    };
    let count = word(4)?;
    if count != 1 {
        return Err(format!(
            "it holds {count} slices; a device app holds arm64's alone"
        ));
    }
    let cpu = word(8)?;
    if cpu != CPU_ARM64 {
        return Err(format!(
            "its one slice is CPU type {cpu:#x}, not arm64's {CPU_ARM64:#x}"
        ));
    }
    let (offset, size) = if wide {
        let long = |at: usize| -> Result<u64, String> {
            Ok((u64::from(word(at)?) << 32) | u64::from(word(at + 4)?))
        };
        (long(16)?, long(24)?)
    } else {
        (u64::from(word(16)?), u64::from(word(20)?))
    };
    let start = usize::try_from(offset).map_err(|_| "the slice's offset is past memory")?;
    let len = usize::try_from(size).map_err(|_| "the slice's size is past memory")?;
    start
        .checked_add(len)
        .and_then(|end| file.get(start..end))
        .ok_or_else(|| format!("the slice at {start:#x}, {len} bytes, runs past the file's end"))
}

/// Every path under `dir` whose file name contains `needle`.
fn collect_named(dir: &Path, needle: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if entry.file_name().to_string_lossy().contains(needle) {
            out.push(path.clone());
        }
        if path.is_dir() {
            collect_named(&path, needle, out);
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

#[cfg(test)]
mod targets {
    use super::*;

    fn of(args: &[&str], xcode: bool) -> Result<Targets, String> {
        let args: Vec<String> = args.iter().map(ToString::to_string).collect();
        Targets::of(&args, xcode)
    }

    #[test]
    fn a_host_builds_every_app_it_can_unless_one_is_named() {
        let both = of(&[], true).unwrap();
        assert_eq!(both.platforms(), "--platforms=android,ios");
        let names: Vec<&str> = both.rows().iter().map(|row| row.name).collect();
        assert_eq!(
            names,
            ["android-arm64", "android-arm", "android-x64", "ios-arm64"]
        );
        assert_eq!(of(&[], false).unwrap().platforms(), "--platforms=android");
        let ios = of(&["ios"], false).unwrap();
        assert_eq!(ios.platforms(), "--platforms=ios");
        assert_eq!(ios.rows().len(), 1);
        assert_eq!(of(&["android"], true).unwrap().rows().len(), 3);
        assert!(of(&["web"], true).unwrap_err().contains("`web`"));
    }

    /// A universal file of the given slices, each `body` at its offset.
    fn universal(cpus: &[u32], body: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend(0xcafe_babe_u32.to_be_bytes());
        out.extend(u32::try_from(cpus.len()).unwrap().to_be_bytes());
        let header = 8 + 20 * cpus.len();
        for (i, cpu) in cpus.iter().enumerate() {
            let offset = u32::try_from(header + i * body.len()).unwrap();
            for word in [*cpu, 0, offset, u32::try_from(body.len()).unwrap(), 0] {
                out.extend(word.to_be_bytes());
            }
        }
        for _ in cpus {
            out.extend(body);
        }
        out
    }

    #[test]
    fn a_framework_binary_is_one_arm64_slice_in_a_universal_file() {
        let body = b"slice bytes";
        assert_eq!(arm64_slice(&universal(&[CPU_ARM64], body)).unwrap(), body);
        let two = arm64_slice(&universal(&[CPU_ARM64, 0x0100_0007], body)).unwrap_err();
        assert!(two.contains("2 slices"), "{two}");
        let x86 = arm64_slice(&universal(&[0x0100_0007], body)).unwrap_err();
        assert!(x86.contains("not arm64"), "{x86}");
        assert!(arm64_slice(&[0xfe, 0xed, 0xfa, 0xcf, 0, 0, 0, 0]).is_err());
        let mut cut = universal(&[CPU_ARM64], body);
        cut.truncate(cut.len() - 1);
        assert!(
            arm64_slice(&cut)
                .unwrap_err()
                .contains("past the file's end")
        );
    }
}
