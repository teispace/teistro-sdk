//! `docs/05-testing/mobile-sizes-measured.md`: what each mobile slice
//! weighs, three ways, rendered from the record
//! `cargo xtask measure-mobile` writes (`docs/05-testing/mobile-sizes.json`).
//!
//! The three libraries are the ones an app chooses between:
//!
//! - **the SDK library** as the release ships it (`crate::mobile`), with
//!   the built-in ephemeris at its default tier;
//! - **the SDK without the built-in**, the library an app that plugs a
//!   real engine in bundles instead (`--no-default-features --features
//!   full`: every module family, no ephemeris of its own);
//! - **the Teimeris adapter** (`adapters/ephemeris-teimeris/rust`), its
//!   modern tier's data embedded, so the app ships no data file.
//!
//! An app on Teimeris bundles the second and the third; one on the
//! built-in bundles the first. The page prints both sums, and what leaving
//! the built-in out saves. Every library is cross-built here the way the
//! release builds the SDK's slice and held to the same floor ([`check`]),
//! so a figure on this page is of a library a phone would load.
//!
//! The record is measured on one machine, not on CI: the adapter links
//! Teimeris, which the SDK's runners do not check out. It names the commit,
//! the Teimeris commit and the toolchains it was measured with.

use std::fmt::Write as _;
use std::fs;
use std::io::Write as _;
use std::path::Path;
use std::process::Command;

use flate2::Compression;
use flate2::write::GzEncoder;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::binding::{LIBRARY_STEM, step};
use crate::generated::{Output, check as check_page, write};
use crate::mobile::{LEAN_FEATURES, MOBILE, Mobile, Os, check, cross, ndk_linker};

const PAGE: &str = "docs/05-testing/mobile-sizes-measured.md";
const RECORD: &str = "docs/05-testing/mobile-sizes.json";
const SCHEMA: &str = "teistro-mobile-sizes/1";

/// The adapter's own workspace, and the file stem its library has.
const ADAPTER: &str = "adapters/ephemeris-teimeris/rust";
const ADAPTER_STEM: &str = "teistro_ephemeris_teimeris";

/// The adapter's embedded tier: the modern one, the tier the adapter's
/// default `data: "embedded"` loads.
const ADAPTER_TIER: &str = "MODERN";

/// One library measured for a row, by the key the record carries it under.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Variant {
    Sdk,
    SdkNoBuiltin,
    Adapter,
}

/// The SDK built as the release's [`crate::mobile::LEAN`] slice is.
const LEAN_ARGS: [&str; 5] = [
    "-p",
    "teistro-ffi",
    LEAN_FEATURES[0],
    LEAN_FEATURES[1],
    LEAN_FEATURES[2],
];

const VARIANTS: [Variant; 3] = [Variant::Sdk, Variant::SdkNoBuiltin, Variant::Adapter];

impl Variant {
    fn key(self) -> &'static str {
        match self {
            Variant::Sdk => "sdk",
            Variant::SdkNoBuiltin => "sdk-no-builtin",
            Variant::Adapter => "adapter",
        }
    }

    /// Where Cargo builds it from, and what it is told after the target.
    fn build(self) -> (&'static str, &'static [&'static str]) {
        match self {
            Variant::Sdk => (".", &["-p", "teistro-ffi"]),
            Variant::SdkNoBuiltin => (".", &LEAN_ARGS),
            Variant::Adapter => (ADAPTER, &["--lib"]),
        }
    }

    fn file(self, row: Mobile) -> String {
        let stem = match self {
            Variant::Sdk | Variant::SdkNoBuiltin => LIBRARY_STEM,
            Variant::Adapter => ADAPTER_STEM,
        };
        match row.os {
            Os::Android => format!("lib{stem}.so"),
            Os::Ios => format!("lib{stem}.dylib"),
        }
    }
}

// ── the record ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    schema: String,
    rows: Vec<Row>,
}

/// One platform's three libraries, and what they were built from.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Row {
    platform: String,
    date: String,
    commit: String,
    teimeris: String,
    rustc: String,
    toolchain: String,
    sdk: Size,
    #[serde(rename = "sdk-no-builtin")]
    sdk_no_builtin: Size,
    adapter: Size,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Size {
    /// As Cargo wrote it, which is the slice the release ships.
    raw: u64,
    gzip: u64,
    /// As an app's build strips it ([`strip`]), which is what a phone
    /// carries.
    stripped: u64,
    stripped_gzip: u64,
}

fn recorded(root: &Path) -> Result<Record, String> {
    let text = fs::read_to_string(root.join(RECORD))
        .map_err(|e| format!("{RECORD}: {e}; run `cargo xtask measure-mobile`"))?;
    let record: Record = serde_json::from_str(&text).map_err(|e| format!("{RECORD}: {e}"))?;
    if record.schema != SCHEMA {
        return Err(format!(
            "{RECORD}: schema {:?}, not {SCHEMA:?}",
            record.schema
        ));
    }
    Ok(record)
}

/// Every mobile row the record lacks, or holds twice, or holds that the
/// table does not name, as sentences.
fn gaps(record: &Record) -> Vec<String> {
    let mut found = Vec::new();
    for row in MOBILE {
        match record
            .rows
            .iter()
            .filter(|r| r.platform == row.name)
            .count()
        {
            1 => {}
            0 => found.push(format!(
                "{RECORD} has no `{}`; run `cargo xtask measure-mobile {}`",
                row.name, row.name
            )),
            n => found.push(format!("{RECORD} holds `{}` {n} times", row.name)),
        }
    }
    for row in &record.rows {
        if Mobile::by_either(&row.platform).is_none() {
            found.push(format!(
                "{RECORD} holds `{}`, which is no mobile row",
                row.platform
            ));
        }
    }
    found
}

// ── measuring ──────────────────────────────────────────────────────────────

/// `measure-mobile [ROW...]`: cross-builds each row's three libraries one
/// after another, checks each, and writes its sizes into the record (the
/// rows not named keep theirs), then the page.
pub(crate) fn measure(root: &Path, names: &[String]) -> i32 {
    let rows: Vec<Mobile> = if names.is_empty() {
        MOBILE.to_vec()
    } else {
        let mut rows = Vec::new();
        for name in names {
            let Some(row) = Mobile::by_either(name) else {
                println!(
                    "FAIL  `{name}` is no mobile row; the rows are {}",
                    MOBILE.map(|row| row.name).join(", ")
                );
                return 1;
            };
            rows.push(row);
        }
        rows
    };
    let mut kept: Vec<Value> = match fs::read_to_string(root.join(RECORD)) {
        Ok(text) => match serde_json::from_str::<Value>(&text) {
            Ok(value) => value
                .get("rows")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default(),
            Err(err) => {
                println!("FAIL  {RECORD}: {err}");
                return 1;
            }
        },
        Err(_) => Vec::new(),
    };
    let Ok(about) = About::read(root, rows.iter().any(|row| row.os == Os::Ios)) else {
        return 1;
    };
    for row in rows {
        let Ok(sizes) = measure_row(root, row) else {
            return 1;
        };
        let mut entry = json!({
            "platform": row.name,
            "date": about.date,
            "commit": about.commit,
            "teimeris": about.teimeris,
            "rustc": about.rustc,
            "toolchain": match row.os {
                Os::Android => about.ndk.clone(),
                Os::Ios => about.xcode.clone(),
            },
        });
        for (variant, size) in VARIANTS.iter().zip(sizes) {
            if let Some(map) = entry.as_object_mut() {
                map.insert(
                    variant.key().to_owned(),
                    json!({
                        "raw": size.raw,
                        "gzip": size.gzip,
                        "stripped": size.stripped,
                        "stripped_gzip": size.stripped_gzip,
                    }),
                );
            }
        }
        kept.retain(|r| r.get("platform").and_then(Value::as_str) != Some(row.name));
        kept.push(entry);
        let order = |r: &Value| {
            MOBILE
                .iter()
                .position(|m| r.get("platform").and_then(Value::as_str) == Some(m.name))
                .unwrap_or(MOBILE.len())
        };
        kept.sort_by_key(order);
        let text = match serde_json::to_string_pretty(&json!({ "schema": SCHEMA, "rows": kept })) {
            Ok(text) => text,
            Err(err) => {
                println!("FAIL  {RECORD}: {err}");
                return 1;
            }
        };
        // Written after each row, so a later row's failure keeps what the
        // earlier ones measured.
        if let Err(err) = fs::write(root.join(RECORD), format!("{text}\n")) {
            println!("FAIL  {RECORD}: {err}");
            return 1;
        }
        println!("ok    {}: measured into {RECORD}", row.name);
    }
    generate(root)
}

fn measure_row(root: &Path, row: Mobile) -> Result<[Size; 3], ()> {
    let mut sizes = [Size {
        raw: 0,
        gzip: 0,
        stripped: 0,
        stripped_gzip: 0,
    }; 3];
    for (variant, size) in VARIANTS.iter().zip(sizes.iter_mut()) {
        let (dir, args) = variant.build();
        let dir = root.join(dir);
        let mut command = cross(root, row, &dir, args)?;
        if *variant == Variant::Adapter {
            // Teimeris built from its source with its data compiled in,
            // never a host library a developer's shell points at.
            command
                .env("TM_EMBED_TIER", ADAPTER_TIER)
                .env_remove("TEIMERIS_LIB_DIR");
        }
        step(
            &mut command,
            "",
            &format!("{} did not build for {}", variant.key(), row.triple),
        )?;
        let file = dir
            .join("target")
            .join(row.triple)
            .join("release")
            .join(variant.file(row));
        let bytes = fs::read(&file).map_err(|err| println!("FAIL  {}: {err}", file.display()))?;
        check(row, &bytes).map_err(|why| println!("FAIL  {}: {why}", file.display()))?;
        let stripped = strip(root, row, &file)?;
        check(row, &stripped)
            .map_err(|why| println!("FAIL  {} stripped: {why}", file.display()))?;
        let gzip = |bytes: &[u8]| gzipped(bytes).map_err(|err| println!("FAIL  gzip: {err}"));
        *size = Size {
            raw: bytes.len() as u64,
            gzip: gzip(&bytes)?,
            stripped: stripped.len() as u64,
            stripped_gzip: gzip(&stripped)?,
        };
        println!(
            "ok    {} {}: {} bytes, {} gzipped; stripped {}, {} gzipped",
            row.name,
            variant.key(),
            size.raw,
            size.gzip,
            size.stripped,
            size.stripped_gzip
        );
    }
    Ok(sizes)
}

/// The library as an app's build strips it, written beside it and read
/// back: on Android what the Gradle plugin's native-library stripping runs
/// (the NDK's `llvm-strip --strip-unneeded`), on iOS the local symbols
/// removed (`strip -x`), which keeps every symbol the loader resolves.
fn strip(root: &Path, row: Mobile, file: &Path) -> Result<Vec<u8>, ()> {
    let out = file.with_extension("stripped");
    let mut command = match row.os {
        Os::Android => {
            let linker = ndk_linker(root, row).map_err(|why| println!("FAIL  {why}"))?;
            let tool = linker.with_file_name(if cfg!(windows) {
                "llvm-strip.exe"
            } else {
                "llvm-strip"
            });
            let mut command = Command::new(tool);
            command.arg("--strip-unneeded");
            command
        }
        Os::Ios => {
            let mut command = Command::new("xcrun");
            command.args(["strip", "-x"]);
            command
        }
    };
    command.arg("-o").arg(&out).arg(file);
    step(
        &mut command,
        "",
        &format!("{} could not be stripped", file.display()),
    )?;
    let bytes = fs::read(&out).map_err(|err| println!("FAIL  {}: {err}", out.display()))?;
    let _ = fs::remove_file(&out);
    Ok(bytes)
}

/// The gzipped size at the level the release compresses its slices at.
fn gzipped(bytes: &[u8]) -> std::io::Result<u64> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(bytes)?;
    Ok(encoder.finish()?.len() as u64)
}

/// What a measurement was taken from.
struct About {
    date: String,
    commit: String,
    teimeris: String,
    rustc: String,
    ndk: String,
    xcode: String,
}

impl About {
    fn read(root: &Path, ios: bool) -> Result<About, ()> {
        let run = |program: &str, args: &[&str], dir: &Path| -> Result<String, ()> {
            Command::new(program)
                .args(args)
                .current_dir(dir)
                .output()
                .ok()
                .filter(|out| out.status.success())
                .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_owned())
                .ok_or_else(|| println!("FAIL  `{program} {}` did not answer", args.join(" ")))
        };
        let teimeris = root.join(ADAPTER).join("../../../../teimeris");
        let dirty = |dir: &Path| {
            run(
                "git",
                &["status", "--porcelain", "--untracked-files=no"],
                dir,
            )
        };
        let commit = |dir: &Path| -> Result<String, ()> {
            let head = run("git", &["rev-parse", "--short=12", "HEAD"], dir)?;
            Ok(if dirty(dir)?.is_empty() {
                head
            } else {
                format!("{head}+changes")
            })
        };
        Ok(About {
            date: run("git", &["show", "-s", "--format=%cs", "HEAD"], root)?,
            commit: commit(root)?,
            teimeris: commit(&teimeris)?,
            rustc: run("rustc", &["-V"], root)?,
            ndk: format!(
                "Android NDK {}",
                fs::read_to_string(root.join("xtask/ndk.version"))
                    .map_err(|err| println!("FAIL  xtask/ndk.version: {err}"))?
                    .trim()
            ),
            // Asked only when an iOS row is measured: a Linux machine
            // measuring Android has no Xcode to name.
            xcode: if ios {
                run("xcodebuild", &["-version"], root)?
                    .lines()
                    .next()
                    .unwrap_or_default()
                    .to_owned()
            } else {
                String::new()
            },
        })
    }
}

// ── the page ───────────────────────────────────────────────────────────────

/// A byte count the way `SIZES.md` prints it, thousands apart by a space.
fn bytes(value: u64) -> String {
    let digits = value.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(' ');
        }
        out.push(c);
    }
    out
}

fn share(part: u64, whole: u64) -> String {
    if whole == 0 {
        return String::from("–");
    }
    #[allow(clippy::cast_precision_loss, reason = "a share printed to one decimal")]
    let ratio = part as f64 / whole as f64 * 100.0;
    format!("{ratio:.1}%")
}

fn table(out: &mut String, rows: &[&Row], pick: fn(Size) -> u64) {
    out.push_str(
        "| platform | SDK library | without the built-in | saved | Teimeris adapter | on Teimeris | against the SDK library |\n\
         |---|--:|--:|--:|--:|--:|--:|\n",
    );
    for row in rows {
        let sdk = pick(row.sdk);
        let bare = pick(row.sdk_no_builtin);
        let adapter = pick(row.adapter);
        let on_teimeris = bare + adapter;
        let against = if on_teimeris >= sdk {
            format!("+{}", bytes(on_teimeris - sdk))
        } else {
            format!("−{}", bytes(sdk - on_teimeris))
        };
        let _ = writeln!(
            out,
            "| `{}` | {} | {} | {} ({}) | {} | {} | {} |",
            row.platform,
            bytes(sdk),
            bytes(bare),
            bytes(sdk.saturating_sub(bare)),
            share(sdk.saturating_sub(bare), sdk),
            bytes(adapter),
            bytes(on_teimeris),
            against,
        );
    }
}

fn page(record: &Record) -> String {
    let mut rows: Vec<&Row> = record.rows.iter().collect();
    rows.sort_by_key(|r| MOBILE.iter().position(|m| m.name == r.platform));
    let mut out = String::new();
    out.push_str("# Mobile sizes\n\n");
    out.push_str(
        "Status: generated by `cargo xtask mobile-sizes` from `mobile-sizes.json`, which `cargo xtask measure-mobile` records; held by `cargo xtask check-mobile-sizes`. Do not edit by hand.\n\n",
    );
    let _ = writeln!(
        out,
        "What each mobile slice weighs, in bytes, for the three libraries an app chooses between. \
         **The SDK library** is the slice the release ships, the built-in ephemeris inside it. \
         **Without the built-in** is the same library built with `--no-default-features --features full`: \
         every module family, no ephemeris of its own, the one an app that plugs a real engine in bundles. \
         **The Teimeris adapter** is `{ADAPTER}` with Teimeris's `{ADAPTER_TIER}` data compiled in, \
         so it needs no data file. **On Teimeris** is the sum an app on Teimeris bundles, the second and the third; \
         the last column sets it against the SDK library alone.\n"
    );
    out.push_str(
        "Each library is cross-built the way the release builds its slice (`xtask/src/mobile.rs`) and held to the same floor before it is measured: the ELF machine, Bionic's libraries alone and 16 KB pages on Android, the platform and deployment target on iOS. It is measured twice. **As built** is the file Cargo writes, the slice the release ships. **Stripped** is what an app's build makes of it, held to the same floor again: on Android the Gradle plugin strips every native library (the NDK's `llvm-strip --strip-unneeded`), and on iOS the local symbols are removed (`strip -x`). Every symbol the loader resolves stays. Read back from a Flutter release build on 2026-10-20 (`adapters/ephemeris-teimeris/flutter/example`): an APK stores both libraries uncompressed at exactly their stripped size here, and an iOS release app's framework is already stripped as far, larger only by the code signature its build adds (about 140 KB on the SDK library).\n\n",
    );
    out.push_str("## Stripped, gzipped\n\nWhat a download carries, near enough: an app store compresses the package it serves, and gzip at the best level stands in for its compressor.\n\n");
    table(&mut out, &rows, |s| s.stripped_gzip);
    out.push_str("\n## Stripped\n\nWhat the installed app holds: Android keeps a library uncompressed in the APK so it can be mapped in place, and an iOS app's frameworks are stored as they are.\n\n");
    table(&mut out, &rows, |s| s.stripped);
    out.push_str("\n## As built\n\nThe slices as Cargo writes them, which the release gzips: the symbols are still in them.\n\n");
    out.push_str("| platform | SDK library | gzipped | without the built-in | gzipped | Teimeris adapter | gzipped |\n|---|--:|--:|--:|--:|--:|--:|\n");
    for row in &rows {
        let _ = writeln!(
            out,
            "| `{}` | {} | {} | {} | {} | {} | {} |",
            row.platform,
            bytes(row.sdk.raw),
            bytes(row.sdk.gzip),
            bytes(row.sdk_no_builtin.raw),
            bytes(row.sdk_no_builtin.gzip),
            bytes(row.adapter.raw),
            bytes(row.adapter.gzip),
        );
    }
    out.push_str("\n## Measured with\n\nMeasured on one machine, not on CI: the adapter links Teimeris, which the SDK's runners do not check out. A commit marked `+changes` was measured with edits on top of it.\n\n");
    out.push_str(
        "| platform | date | commit | Teimeris | rustc | toolchain |\n|---|---|---|---|---|---|\n",
    );
    for row in &rows {
        let _ = writeln!(
            out,
            "| `{}` | {} | `{}` | `{}` | {} | {} |",
            row.platform, row.date, row.commit, row.teimeris, row.rustc, row.toolchain
        );
    }
    out
}

fn outputs(root: &Path) -> Result<Vec<Output>, i32> {
    let record = recorded(root).map_err(|message| {
        println!("FAIL  {message}");
        1
    })?;
    let gaps = gaps(&record);
    if !gaps.is_empty() {
        for gap in &gaps {
            println!("FAIL  {gap}");
        }
        return Err(1);
    }
    Ok(vec![Output::new(PAGE, page(&record))])
}

/// Writes the page.
pub(crate) fn generate(root: &Path) -> i32 {
    match outputs(root) {
        Ok(outputs) => write(root, &outputs),
        Err(code) => code,
    }
}

/// Regenerates the page in memory and fails on any difference, or on a
/// mobile row the record lacks.
pub(crate) fn check_generated(root: &Path) -> i32 {
    match outputs(root) {
        Ok(outputs) => check_page(root, &outputs, "cargo xtask mobile-sizes"),
        Err(code) => code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_byte_count_is_grouped_by_thousands() {
        assert_eq!(bytes(0), "0");
        assert_eq!(bytes(999), "999");
        assert_eq!(bytes(1_000), "1 000");
        assert_eq!(bytes(4_453_568), "4 453 568");
    }

    #[test]
    fn the_sdk_variants_measure_the_slice_the_release_ships() {
        for row in MOBILE {
            assert_eq!(Variant::Sdk.file(row), row.library());
            assert_eq!(Variant::SdkNoBuiltin.file(row), row.library());
            let adapter = Variant::Adapter.file(row);
            let extension = row.library().rsplit_once('.').map(|(_, e)| e.to_owned());
            assert_eq!(
                adapter.rsplit_once('.').map(|(_, e)| e.to_owned()),
                extension
            );
        }
    }

    #[test]
    fn a_record_missing_a_row_is_named() {
        let record = Record {
            schema: String::from(SCHEMA),
            rows: Vec::new(),
        };
        assert_eq!(gaps(&record).len(), MOBILE.len());
    }
}
