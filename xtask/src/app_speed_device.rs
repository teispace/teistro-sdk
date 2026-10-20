//! `cargo xtask app-speed-device ROW`: `teistro-ephemeris-teimeris-app-speed`
//! run where an app runs, its record written beside the desktop ones in
//! `docs/05-testing/app-speed/` and the page regenerated.
//!
//! The binary is cross-built for the row the way its libraries are
//! (`crate::mobile::cross`), with Teimeris's data built in, as a phone
//! has it. It runs on the Android device or emulator `adb` reaches, or on
//! the booted iOS simulator through `simctl spawn`, and what it prints is
//! the record. An iPhone itself needs a signed app to run code, so the
//! `ios-arm64` row is refused here; the simulator runs the Mac's own
//! processor, and its figures are the Mac's, not a phone's, which the
//! record says by naming the simulator.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

use crate::binding::step;
use crate::mobile::{Mobile, Os, cross};

const RECORDED: &str = "docs/05-testing/app-speed";
const ADAPTER: &str = "adapters/ephemeris-teimeris/rust";
const BINARY: &str = "teistro-ephemeris-teimeris-app-speed";

/// Where the binary is put on an Android device: the one directory a
/// shell user may write to and execute from.
const ANDROID_PATH: &str = "/data/local/tmp/teistro-app-speed";

/// Builds, runs and records the row named.
pub(crate) fn run(root: &Path, name: Option<&str>) -> i32 {
    let Some(row) = name.and_then(Mobile::by_either) else {
        println!(
            "FAIL  name the row to run on: android-arm64, android-arm, android-x64 or \
             ios-sim-arm64, ios-sim-x64"
        );
        return 1;
    };
    // The device's SDK, as Flutter names where the slice goes.
    if row.os == Os::Ios && row.bundle == "iphoneos" {
        println!(
            "FAIL  {} is an iPhone, which runs only a signed app's code; run the simulator's \
             row, or time the app itself",
            row.name
        );
        return 1;
    }
    let dir = root.join(ADAPTER);
    let Ok(mut command) = cross(
        root,
        row,
        &dir,
        &["--features", "app-speed", "--bin", BINARY],
    ) else {
        return 1;
    };
    command
        .env("TM_EMBED_TIER", "MODERN")
        .env_remove("TEIMERIS_LIB_DIR");
    if step(
        &mut command,
        "",
        &format!("{BINARY} did not build for {}", row.triple),
    )
    .is_err()
    {
        return 1;
    }
    let binary = dir
        .join("target")
        .join(row.triple)
        .join("release")
        .join(BINARY);
    let commit = Command::new("git")
        .args(["rev-parse", "--short=8", "HEAD"])
        .current_dir(root)
        .output()
        .ok()
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_owned())
        .unwrap_or_default();
    let printed = match row.os {
        Os::Android => android(row, &binary, &commit),
        Os::Ios => simulator(&binary, &commit),
    };
    let Ok(printed) = printed.map_err(|why| println!("FAIL  {why}")) else {
        return 1;
    };
    let record: Value = match serde_json::from_str(&printed) {
        Ok(record) => record,
        Err(err) => {
            println!("FAIL  the run printed no record ({err}):\n{printed}");
            return 1;
        }
    };
    let machine = record["machine"].as_str().unwrap_or("device");
    let file = root
        .join(RECORDED)
        .join(format!("{}-{}.json", row.name, slug(machine)));
    if let Err(err) = fs::write(&file, format!("{}\n", printed.trim_end())) {
        println!("FAIL  {}: {err}", file.display());
        return 1;
    }
    println!("ok    {machine}: recorded into {}", crate::rel(root, &file));
    crate::app_speed::generate(root)
}

/// A machine's name as a file name: letters and digits, the rest one `-`.
fn slug(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_owned()
}

fn adb() -> PathBuf {
    let sdk = std::env::var_os("ANDROID_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join("Library/Android/sdk"))
        });
    sdk.map(|sdk| sdk.join("platform-tools/adb"))
        .filter(|adb| adb.is_file())
        .unwrap_or_else(|| PathBuf::from("adb"))
}

/// What a command printed on success, or why it did not succeed.
fn printed(command: &mut Command) -> Result<String, String> {
    let shown = format!("{command:?}");
    let out = command
        .output()
        .map_err(|err| format!("{shown} did not start: {err}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        Err(format!(
            "{shown} failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

fn android(row: Mobile, binary: &Path, commit: &str) -> Result<String, String> {
    let adb = adb();
    let state = printed(Command::new(&adb).arg("get-state"))
        .map_err(|why| format!("no Android device or emulator is reachable ({why}); start one"))?;
    if state.trim() != "device" {
        return Err(format!(
            "the Android device is `{}`, not ready",
            state.trim()
        ));
    }
    // A device runs each ABI it lists; the binary is one of them.
    let abis = printed(Command::new(&adb).args(["shell", "getprop", "ro.product.cpu.abilist"]))?;
    if !abis.trim().split(',').any(|abi| abi == row.bundle) {
        return Err(format!(
            "the device runs {}, not {}; run the row it does",
            abis.trim(),
            row.bundle
        ));
    }
    printed(Command::new(&adb).arg("push").arg(binary).arg(ANDROID_PATH))?;
    printed(Command::new(&adb).args(["shell", "chmod", "755", ANDROID_PATH]))?;
    printed(Command::new(&adb).args(["shell", &format!("TEISTRO_COMMIT={commit} {ANDROID_PATH}")]))
}

fn simulator(binary: &Path, commit: &str) -> Result<String, String> {
    let booted = printed(Command::new("xcrun").args(["simctl", "list", "devices", "booted"]))?;
    if !booted.contains("(Booted)") {
        return Err(String::from(
            "no iOS simulator is booted; boot one (`xcrun simctl boot <device>`)",
        ));
    }
    printed(
        Command::new("xcrun")
            .args(["simctl", "spawn", "booted"])
            .arg(binary)
            .env("SIMCTL_CHILD_TEISTRO_COMMIT", commit),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_machine_becomes_a_file_name() {
        assert_eq!(
            slug("Android emulator, `sdk_gphone64_arm64`"),
            "android-emulator-sdk-gphone64-arm64"
        );
        assert_eq!(
            slug("iOS simulator, iPhone 18 Pro"),
            "ios-simulator-iphone-18-pro"
        );
    }
}
