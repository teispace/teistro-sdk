//! The mobile platforms: the library cross-built for Android and iOS, the
//! slices the Flutter package's build hook bundles into an app
//! (`03-design/flutter-build-hook.md`).
//!
//! They are a table of their own, not rows of [`crate::platform::PLATFORMS`],
//! because everything that reads that table means a host the SDK runs on
//! by itself: an npm package, a wheel, a jar, the agent server, a consumer
//! check. A mobile slice ships none of those, so a row there would have to
//! be filtered out by every reader, and a reader that forgot would stage a
//! broken package; apart, no reader can reach one.
//!
//! A mobile row is the first the SDK does not build natively: no runner is
//! an Android device or an iPhone. The crate graph holds no C, so a cross
//! build needs only a linker, which the NDK and Xcode provide, and nothing
//! else changes: the same profile, the same `cargo auditable` build. What
//! the build cannot be trusted to have done is read back from the file
//! ([`check`]): an Android library needs no C++ runtime and, on a 64-bit
//! row, is aligned for 16 KB pages, as Google Play requires; an iOS one is
//! built for its platform and the deployment target the SDK states.

use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use flate2::Compression;
use flate2::write::GzEncoder;
use serde_json::{Value, json};

use crate::binding::{LIBRARY_STEM, cargo, step};
use crate::package::{DIST, SCHEMA, entry, manifest_name, to_json};
use crate::{release, sbom};

/// The operating system a mobile row is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Os {
    /// Android, at [`ANDROID_API`].
    Android,
    /// iOS, a device or the simulator, at [`IOS_FLOOR`].
    Ios,
}

/// One mobile platform: the slice a Flutter build of one ABI or SDK
/// bundles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Mobile {
    /// The name the release and the digest table carry it under, in the
    /// words the host rows use (`android-arm64`).
    pub(crate) name: &'static str,
    /// Cargo's target triple.
    pub(crate) triple: &'static str,
    /// Its operating system.
    pub(crate) os: Os,
    /// The GitHub runner that cross-builds it.
    pub(crate) runner: &'static str,
    /// The ELF machine (Android) or the Mach-O CPU type (iOS) the file
    /// must be built for.
    pub(crate) machine: u32,
    /// Android: whether every loadable segment must be aligned for 16 KB
    /// pages (the 64-bit ABIs, which Play requires it of). iOS: unused.
    pub(crate) sixteen_k: bool,
    /// iOS: the `LC_BUILD_VERSION` platform, the device's or the
    /// simulator's. Android: unused.
    pub(crate) apple_platform: u32,
    /// Where a Flutter build puts it: the APK's ABI directory
    /// (`lib/arm64-v8a`) on Android, the SDK it is built for on iOS.
    pub(crate) bundle: &'static str,
    /// iOS: the deployment target the slice is built for and held to,
    /// [`IOS_FLOOR`] unless the target cannot go that low. Android: unused.
    pub(crate) minos: (u32, u32),
}

/// The Android API level every Android slice is linked for: Flutter's
/// default `minSdkVersion`, and above Rust's own minimum for each target.
pub(crate) const ANDROID_API: u32 = 24;

/// The name of the slice built **without the built-in ephemeris**, beside
/// each row's default: what an app that plugs a real engine in bundles
/// (`hooks: user_defines: teistro_flutter: builtin: false`). It is the
/// row's name with this after it, in its archive's name and in the digest
/// table (`android-arm64-no-builtin`).
pub(crate) const LEAN: &str = "no-builtin";

/// Cargo's arguments for the [`LEAN`] slice: every module family, and no
/// ephemeris of the library's own.
pub(crate) const LEAN_FEATURES: [&str; 3] = ["--no-default-features", "--features", "full"];

/// The iOS deployment target every iOS slice is built for: below the
/// version Flutter states for an app (15), so every Flutter app accepts
/// it.
pub(crate) const IOS_FLOOR: (u32, u32) = (13, 0);

/// The arm64 simulator's: there is no arm64 simulator runtime before iOS
/// 14, and Rust's `aarch64-apple-ios-sim` raises a lower target to it, so
/// the slice says 14.0 whatever it is asked for. No app is narrowed by it:
/// an Apple silicon Mac runs no simulator older than 14.
const IOS_SIM_ARM64_FLOOR: (u32, u32) = (14, 0);

/// The file holding the NDK release the Android rows link with, Flutter's
/// own default, read by the workflows and named by a failing local build.
const NDK_VERSION: &str = "xtask/ndk.version";

/// `EM_AARCH64`, `EM_ARM` and `EM_X86_64`.
const EM_AARCH64: u32 = 183;
const EM_ARM: u32 = 40;
const EM_X86_64: u32 = 62;
/// `CPU_TYPE_ARM64` and `CPU_TYPE_X86_64`.
const CPU_TYPE_ARM64: u32 = 0x0100_000c;
const CPU_TYPE_X86_64: u32 = 0x0100_0007;
/// `PLATFORM_IOS` and `PLATFORM_IOSSIMULATOR`.
const PLATFORM_IOS: u32 = 2;
const PLATFORM_IOSSIMULATOR: u32 = 7;

/// Every mobile platform the release builds: Android's three ABIs a
/// default `flutter build apk` asks for, then iOS's device and its two
/// simulators.
pub(crate) const MOBILE: [Mobile; 6] = [
    Mobile {
        name: "android-arm64",
        triple: "aarch64-linux-android",
        os: Os::Android,
        runner: "ubuntu-latest",
        machine: EM_AARCH64,
        sixteen_k: true,
        apple_platform: 0,
        bundle: "arm64-v8a",
        minos: (0, 0),
    },
    // 32-bit Arm: no runner can run it, and a default `flutter build apk`
    // asks for it, so it ships with the build's checks and a hash-matrix
    // row under qemu (`flutter-build-hook.md` §6, O2).
    Mobile {
        name: "android-arm",
        triple: "armv7-linux-androideabi",
        os: Os::Android,
        runner: "ubuntu-latest",
        machine: EM_ARM,
        sixteen_k: false,
        apple_platform: 0,
        bundle: "armeabi-v7a",
        minos: (0, 0),
    },
    Mobile {
        name: "android-x64",
        triple: "x86_64-linux-android",
        os: Os::Android,
        runner: "ubuntu-latest",
        machine: EM_X86_64,
        sixteen_k: true,
        apple_platform: 0,
        bundle: "x86_64",
        minos: (0, 0),
    },
    Mobile {
        name: "ios-arm64",
        triple: "aarch64-apple-ios",
        os: Os::Ios,
        runner: "macos-latest",
        machine: CPU_TYPE_ARM64,
        sixteen_k: false,
        apple_platform: PLATFORM_IOS,
        bundle: "iphoneos",
        minos: IOS_FLOOR,
    },
    Mobile {
        name: "ios-sim-arm64",
        triple: "aarch64-apple-ios-sim",
        os: Os::Ios,
        runner: "macos-latest",
        machine: CPU_TYPE_ARM64,
        sixteen_k: false,
        apple_platform: PLATFORM_IOSSIMULATOR,
        bundle: "iphonesimulator",
        minos: IOS_SIM_ARM64_FLOOR,
    },
    Mobile {
        name: "ios-sim-x64",
        triple: "x86_64-apple-ios",
        os: Os::Ios,
        runner: "macos-latest",
        machine: CPU_TYPE_X86_64,
        sixteen_k: false,
        apple_platform: PLATFORM_IOSSIMULATOR,
        bundle: "iphonesimulator",
        minos: IOS_FLOOR,
    },
];

impl Mobile {
    /// The row a name or a triple names.
    pub(crate) fn by_either(name: &str) -> Option<Mobile> {
        MOBILE
            .into_iter()
            .find(|row| row.name == name || row.triple == name)
    }

    /// The library's file name: what Flutter bundles, the same for every
    /// ABI and SDK of one operating system, as it requires.
    pub(crate) fn library(&self) -> String {
        match self.os {
            Os::Android => format!("lib{LIBRARY_STEM}.so"),
            Os::Ios => format!("lib{LIBRARY_STEM}.dylib"),
        }
    }
}

/// The libraries an Android slice may link against: Bionic's own, which
/// every device carries. A C++ runtime (`libc++_shared.so`) would have to
/// be bundled beside it, and the crate graph has no C++.
const ANDROID_NEEDED: [&str; 4] = ["libc.so", "libm.so", "libdl.so", "liblog.so"];

// ── the build ──────────────────────────────────────────────────────────────

/// Cross-builds one mobile row's library, checks it, and writes its gzipped
/// slice, its bill of materials and its manifest into `target/dist`.
pub(crate) fn build(root: &Path, row: Mobile) -> i32 {
    let version = release::version(root);
    let dist = root.join(DIST);
    let Ok(built) = compile(root, row, &[]) else {
        return 1;
    };
    if let Err(err) = stage(root, &dist, row, &version, &built) {
        println!("FAIL  {} could not be packaged: {err}", row.name);
        return 1;
    }
    // Built second, over the first in Cargo's target directory: the
    // default slice is already staged, its digest taken from the file.
    let Ok(built) = compile(root, row, &LEAN_FEATURES) else {
        return 1;
    };
    match stage_lean(root, &dist, row, &version, &built) {
        Ok(()) => {
            println!(
                "ok    {} {version} for {}: the library, the one without the built-in, their bills and the manifest in {}",
                LIBRARY_STEM,
                row.name,
                crate::rel(root, &dist)
            );
            0
        }
        Err(err) => {
            println!("FAIL  {} could not be packaged: {err}", row.name);
            1
        }
    }
}

/// Builds the library for the row through `cargo auditable`, with the
/// linker and the link arguments its operating system needs, and answers
/// where Cargo wrote it.
fn compile(root: &Path, row: Mobile, features: &[&str]) -> Result<PathBuf, ()> {
    let mut args = vec!["-p", "teistro-ffi"];
    args.extend_from_slice(features);
    let mut command = cross(root, row, root, &args)?;
    step(
        &mut command,
        "",
        &format!(
            "the library did not build for {} (`rustup target add {}`; every row builds \
             through cargo-auditable)",
            row.triple, row.triple
        ),
    )?;
    Ok(root.join("target").join(row.triple).join("release"))
}

/// The `cargo auditable build --release` of a crate under `dir` for the
/// row, with `args` after the target: the linker and link arguments its
/// operating system needs, and on Android the NDK's clang and archiver as
/// the C compiler too, for a crate that compiles C of its own (the
/// Teimeris adapter, `crate::mobile_sizes`).
pub(crate) fn cross(root: &Path, row: Mobile, dir: &Path, args: &[&str]) -> Result<Command, ()> {
    let mut command = Command::new(cargo());
    command
        .args([
            "auditable",
            "build",
            "--release",
            "--quiet",
            "--target",
            row.triple,
        ])
        .args(args)
        .current_dir(dir);
    let flags_var = format!(
        "CARGO_TARGET_{}_RUSTFLAGS",
        row.triple.to_ascii_uppercase().replace('-', "_")
    );
    match row.os {
        Os::Android => {
            let linker = ndk_linker(root, row).map_err(|why| println!("FAIL  {why}"))?;
            let cc_suffix = row.triple.replace('-', "_");
            if let Some(bin) = linker.parent() {
                command.env(format!("AR_{cc_suffix}"), bin.join("llvm-ar"));
            }
            command.env(format!("CC_{cc_suffix}"), &linker);
            command.env(
                format!(
                    "CARGO_TARGET_{}_LINKER",
                    row.triple.to_ascii_uppercase().replace('-', "_")
                ),
                linker,
            );
            // NDK r28 aligns for 16 KB already; the flag states the rule the
            // check below holds, so a different NDK cannot quietly drop it.
            if row.sixteen_k {
                command.env(flags_var, "-C link-arg=-Wl,-z,max-page-size=16384");
            }
        }
        Os::Ios => {
            command.env(
                "IPHONEOS_DEPLOYMENT_TARGET",
                format!("{}.{}", row.minos.0, row.minos.1),
            );
            // Room for the install name Flutter writes when it wraps the
            // library in a framework, whatever its length.
            command.env(flags_var, "-C link-arg=-Wl,-headerpad_max_install_names");
        }
    }
    Ok(command)
}

/// The NDK's clang for the row at [`ANDROID_API`], from the pinned NDK: in
/// `$ANDROID_NDK_HOME`, or under the SDK (`$ANDROID_HOME`, then the
/// default macOS and Linux places) by its version.
/// The first of `candidates` that is an NDK at the pinned release.
///
/// One at another release is passed over, not refused: a runner's
/// `ANDROID_NDK_HOME` names the image's own NDK (GitHub's Ubuntu image
/// says 27.3), while the pinned one is installed beside it. A mismatch is
/// reported only when no candidate is the pinned release.
fn pinned_ndk(candidates: impl Iterator<Item = PathBuf>, pinned: &str) -> Result<PathBuf, String> {
    let mut passed = Vec::new();
    for dir in candidates {
        let Ok(properties) = fs::read_to_string(dir.join("source.properties")) else {
            continue;
        };
        let revision = properties
            .lines()
            .find_map(|line| line.strip_prefix("Pkg.Revision"))
            .map(|rest| rest.trim_start_matches([' ', '=']).trim().to_owned());
        if revision.as_deref() == Some(pinned) {
            return Ok(dir);
        }
        passed.push(format!(
            "{} is {}",
            dir.display(),
            revision.as_deref().unwrap_or("of no stated version")
        ));
    }
    Err(format!(
        "no Android NDK {pinned} was found ({NDK_VERSION}){}; install it with \
         `sdkmanager \"ndk;{pinned}\"`, or name it with ANDROID_NDK_HOME",
        if passed.is_empty() {
            String::new()
        } else {
            format!(", and {}", passed.join(", "))
        }
    ))
}

pub(crate) fn ndk_linker(root: &Path, row: Mobile) -> Result<PathBuf, String> {
    let pinned = fs::read_to_string(root.join(NDK_VERSION))
        .map_err(|err| format!("{NDK_VERSION}: {err}"))?
        .trim()
        .to_owned();
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let candidates = [
        std::env::var_os("ANDROID_NDK_HOME").map(PathBuf::from),
        std::env::var_os("ANDROID_HOME").map(|sdk| PathBuf::from(sdk).join("ndk").join(&pinned)),
        home.as_ref()
            .map(|home| home.join("Library/Android/sdk/ndk").join(&pinned)),
        home.as_ref()
            .map(|home| home.join("Android/Sdk/ndk").join(&pinned)),
    ];
    let ndk = pinned_ndk(candidates.into_iter().flatten(), &pinned)?;
    let host = if cfg!(target_os = "macos") {
        "darwin-x86_64"
    } else if cfg!(target_os = "windows") {
        "windows-x86_64"
    } else {
        "linux-x86_64"
    };
    // The NDK names armv7's clang `armv7a-linux-androideabi`, where Rust
    // names the target `armv7-linux-androideabi`.
    let clang = row.triple.replacen("armv7-", "armv7a-", 1);
    let linker = ndk
        .join("toolchains/llvm/prebuilt")
        .join(host)
        .join("bin")
        .join(format!(
            "{clang}{ANDROID_API}-clang{}",
            if cfg!(windows) { ".cmd" } else { "" }
        ));
    if linker.is_file() {
        Ok(linker)
    } else {
        Err(format!("the NDK has no {}", linker.display()))
    }
}

/// Checks the built library and writes what the row ships: the gzipped
/// slice the build hook downloads, its bill of materials, and a manifest
/// `package stage` merges.
fn stage(root: &Path, dist: &Path, row: Mobile, version: &str, built: &Path) -> io::Result<()> {
    fs::create_dir_all(dist)?;
    let library = built.join(row.library());
    let bytes = fs::read(&library)
        .map_err(|err| io::Error::other(format!("{}: {err}", library.display())))?;
    check(row, &bytes).map_err(|why| io::Error::other(format!("{}: {why}", library.display())))?;
    println!("floor  {} is {}", library.display(), floor(row));
    sbom::embedded(&[&library]).map_err(io::Error::other)?;
    let bill = dist.join(format!("teistro-{version}-{}-library.cdx.json", row.name));
    sbom::write(root, row.triple, "teistro-ffi", &bill).map_err(io::Error::other)?;

    let file = row.library();
    let (stem, extension) = file.rsplit_once('.').unwrap_or((file.as_str(), "so"));
    let name = format!("{stem}-{version}-{}.{extension}.gz", row.name);
    let gzipped = dist.join(&name);
    let mut encoder = GzEncoder::new(File::create(&gzipped)?, Compression::best());
    io::copy(&mut bytes.as_slice(), &mut encoder)?;
    encoder.finish()?;

    let manifest: Value = json!({
        "schema": SCHEMA,
        "version": version,
        "platform": row.name,
        "triple": row.triple,
        "mobile": true,
        "library": entry(&library, &file)?,
        "archives": [entry(&gzipped, &name)?],
        "sboms": [entry(&bill, &bill.file_name().map_or_else(String::new, |n| n.to_string_lossy().into_owned()))?],
    });
    fs::write(
        dist.join(manifest_name(version, row.name)),
        format!("{}\n", to_json(&manifest)),
    )
}

/// Checks the [`LEAN`] library and adds it to the row's manifest under
/// `variants`, with its gzipped slice and its own bill, read with the
/// features it was built with.
fn stage_lean(
    root: &Path,
    dist: &Path,
    row: Mobile,
    version: &str,
    built: &Path,
) -> io::Result<()> {
    let library = built.join(row.library());
    let bytes = fs::read(&library)
        .map_err(|err| io::Error::other(format!("{}: {err}", library.display())))?;
    check(row, &bytes).map_err(|why| io::Error::other(format!("{}: {why}", library.display())))?;
    sbom::embedded(&[&library]).map_err(io::Error::other)?;
    let name = format!("{}-{LEAN}", row.name);
    let bill = dist.join(format!("teistro-{version}-{name}-library.cdx.json"));
    sbom::write_with(root, row.triple, "teistro-ffi", &LEAN_FEATURES, &bill)
        .map_err(io::Error::other)?;

    let file = row.library();
    let (stem, extension) = file.rsplit_once('.').unwrap_or((file.as_str(), "so"));
    let archive = format!("{stem}-{version}-{name}.{extension}.gz");
    let gzipped = dist.join(&archive);
    let mut encoder = GzEncoder::new(File::create(&gzipped)?, Compression::best());
    io::copy(&mut bytes.as_slice(), &mut encoder)?;
    encoder.finish()?;

    let path = dist.join(manifest_name(version, row.name));
    let mut manifest: Value = serde_json::from_str(&fs::read_to_string(&path)?)
        .map_err(|err| io::Error::other(format!("{}: {err}", path.display())))?;
    let variant = json!({
        "library": entry(&library, &file)?,
        "archives": [entry(&gzipped, &archive)?],
        "sboms": [entry(&bill, &bill.file_name().map_or_else(String::new, |n| n.to_string_lossy().into_owned()))?],
    });
    manifest
        .as_object_mut()
        .ok_or_else(|| io::Error::other(format!("{} is not an object", path.display())))?
        .insert(String::from("variants"), json!({ LEAN: variant }));
    fs::write(&path, format!("{}\n", to_json(&manifest)))
}

/// What the floor check holds the row to, as a sentence.
fn floor(row: Mobile) -> String {
    match row.os {
        Os::Android => format!(
            "built for API {ANDROID_API}, needing only {}{}",
            ANDROID_NEEDED.join(", "),
            if row.sixteen_k {
                ", aligned for 16 KB pages"
            } else {
                ""
            }
        ),
        Os::Ios => format!(
            "built for {} {}.{}",
            if row.apple_platform == PLATFORM_IOS {
                "iOS"
            } else {
                "the iOS simulator"
            },
            row.minos.0,
            row.minos.1
        ),
    }
}

// ── reading the file back ──────────────────────────────────────────────────

/// Holds a built library to its row: the architecture, and on Android the
/// libraries it needs and its segments' alignment, on iOS the platform and
/// the deployment target its build version records.
pub(crate) fn check(row: Mobile, file: &[u8]) -> Result<(), String> {
    match row.os {
        Os::Android => android(row, &Elf::read(file)?),
        Os::Ios => ios(row, &MachO::read(file)?),
    }
}

fn android(row: Mobile, elf: &Elf) -> Result<(), String> {
    if elf.machine != row.machine {
        return Err(format!(
            "it is built for ELF machine {}, and {} is {}",
            elf.machine, row.name, row.machine
        ));
    }
    let foreign: Vec<&str> = elf
        .needed
        .iter()
        .map(String::as_str)
        .filter(|name| !ANDROID_NEEDED.contains(name))
        .collect();
    if !foreign.is_empty() {
        return Err(format!(
            "it needs {}, which a device does not carry; it may need only {}",
            foreign.join(", "),
            ANDROID_NEEDED.join(", ")
        ));
    }
    if row.sixteen_k {
        if let Some(align) = elf.load_aligns.iter().find(|align| **align < 0x4000) {
            return Err(format!(
                "a loadable segment is aligned to {align:#x} bytes, and Google Play asks a 64-bit \
                 library for 16 KB (0x4000)"
            ));
        }
    }
    Ok(())
}

fn ios(row: Mobile, macho: &MachO) -> Result<(), String> {
    if macho.cpu != row.machine {
        return Err(format!(
            "it is built for Mach-O CPU type {:#x}, and {} is {:#x}",
            macho.cpu, row.name, row.machine
        ));
    }
    let Some((platform, minos)) = macho.build_version else {
        return Err("it records no LC_BUILD_VERSION, so no platform or deployment target".into());
    };
    if platform != row.apple_platform {
        return Err(format!(
            "it is built for Apple platform {platform}, and {} is {}",
            row.name, row.apple_platform
        ));
    }
    let wanted = (row.minos.0 << 16) | (row.minos.1 << 8);
    if minos != wanted {
        return Err(format!(
            "its deployment target is {}.{}, and the SDK states {}.{}",
            minos >> 16,
            (minos >> 8) & 0xff,
            row.minos.0,
            row.minos.1
        ));
    }
    Ok(())
}

/// What the floor check reads of an ELF file: its machine, the libraries
/// its dynamic section needs, and each loadable segment's alignment. Both
/// classes are read, because Android's 32-bit Arm row is ELF32; only the
/// little-endian layout, the only one Android ships.
#[derive(Debug, PartialEq, Eq)]
struct Elf {
    machine: u32,
    needed: Vec<String>,
    load_aligns: Vec<u64>,
}

const PT_LOAD: u32 = 1;
const PT_DYNAMIC: u32 = 2;
const DT_NEEDED: u64 = 1;
const DT_STRTAB: u64 = 5;

impl Elf {
    fn read(file: &[u8]) -> Result<Elf, String> {
        if file.get(..4) != Some(b"\x7fELF".as_slice()) {
            return Err("not an ELF file".into());
        }
        if file.get(5) != Some(&1) {
            return Err("not a little-endian ELF file, the only layout Android ships".into());
        }
        let wide = match file.get(4) {
            Some(1) => false,
            Some(2) => true,
            _ => return Err("an ELF file of no known class".into()),
        };
        let word = |at: usize| -> Result<u64, String> {
            if wide {
                read_u64(file, at)
            } else {
                read_u32(file, at).map(u64::from)
            }
        };
        let machine = u32::from(read_u16(file, 0x12)?);
        let (phoff, phentsize, phnum) = if wide {
            (
                read_u64(file, 0x20)?,
                read_u16(file, 0x36)?,
                read_u16(file, 0x38)?,
            )
        } else {
            (
                u64::from(read_u32(file, 0x1c)?),
                read_u16(file, 0x2a)?,
                read_u16(file, 0x2c)?,
            )
        };
        let mut load_aligns = Vec::new();
        let mut segments = Vec::new();
        let mut dynamic = None;
        for index in 0..usize::from(phnum) {
            let at = index
                .checked_mul(usize::from(phentsize))
                .and_then(|offset| offset.checked_add(to_usize(phoff).ok()?))
                .ok_or("the program headers run past the file")?;
            let kind = read_u32(file, at)?;
            // ELF64: type, flags, offset, vaddr, paddr, filesz, memsz, align.
            // ELF32: type, offset, vaddr, paddr, filesz, memsz, flags, align.
            let (offset, vaddr, filesz, align) = if wide {
                (
                    read_u64(file, at + 0x08)?,
                    read_u64(file, at + 0x10)?,
                    read_u64(file, at + 0x20)?,
                    read_u64(file, at + 0x30)?,
                )
            } else {
                (
                    u64::from(read_u32(file, at + 0x04)?),
                    u64::from(read_u32(file, at + 0x08)?),
                    u64::from(read_u32(file, at + 0x10)?),
                    u64::from(read_u32(file, at + 0x1c)?),
                )
            };
            if kind == PT_LOAD {
                load_aligns.push(align);
                segments.push((vaddr, offset, filesz));
            } else if kind == PT_DYNAMIC {
                dynamic = Some((offset, filesz));
            }
        }
        let mut needed = Vec::new();
        if let Some((offset, size)) = dynamic {
            let entry = if wide { 16 } else { 8 };
            let mut names = Vec::new();
            let mut strtab = None;
            let start = to_usize(offset)?;
            for index in 0..to_usize(size)? / entry {
                let at = start + index * entry;
                let tag = word(at)?;
                let value = word(at + entry / 2)?;
                match tag {
                    0 => break,
                    DT_NEEDED => names.push(value),
                    DT_STRTAB => strtab = Some(value),
                    _ => {}
                }
            }
            // The string table is named by its address; the segment that
            // maps it gives its place in the file.
            let strtab = strtab.ok_or("a dynamic section with no string table")?;
            let base = segments
                .iter()
                .find(|(vaddr, _, filesz)| strtab >= *vaddr && strtab < vaddr + filesz)
                .map(|(vaddr, offset, _)| strtab - vaddr + offset)
                .ok_or("the string table lies in no loaded segment")?;
            for name in names {
                needed.push(read_string(file, to_usize(base + name)?)?);
            }
        }
        Ok(Elf {
            machine,
            needed,
            load_aligns,
        })
    }
}

/// What the floor check reads of a thin Mach-O file: its CPU type, and the
/// platform and deployment target `LC_BUILD_VERSION` records.
#[derive(Debug, PartialEq, Eq)]
struct MachO {
    cpu: u32,
    build_version: Option<(u32, u32)>,
}

const MH_MAGIC_64: u32 = 0xfeed_facf;
const LC_BUILD_VERSION: u32 = 0x32;

impl MachO {
    fn read(file: &[u8]) -> Result<MachO, String> {
        if read_u32(file, 0)? != MH_MAGIC_64 {
            return Err(
                "not a thin 64-bit little-endian Mach-O file, the only layout iOS ships".into(),
            );
        }
        let cpu = read_u32(file, 4)?;
        let commands = read_u32(file, 0x10)?;
        let mut at = 0x20;
        let mut build_version = None;
        for _ in 0..commands {
            let command = read_u32(file, at)?;
            let size = to_usize(u64::from(read_u32(file, at + 4)?))?;
            if command == LC_BUILD_VERSION {
                build_version = Some((read_u32(file, at + 8)?, read_u32(file, at + 12)?));
            }
            if size == 0 {
                return Err("a load command of no size".into());
            }
            at += size;
        }
        Ok(MachO { cpu, build_version })
    }
}

fn bytes<const N: usize>(file: &[u8], at: usize) -> Result<[u8; N], String> {
    file.get(at..at + N)
        .and_then(|slice| slice.try_into().ok())
        .ok_or_else(|| format!("the file ends before byte {}", at + N))
}

fn read_u16(file: &[u8], at: usize) -> Result<u16, String> {
    bytes(file, at).map(u16::from_le_bytes)
}

fn read_u32(file: &[u8], at: usize) -> Result<u32, String> {
    bytes(file, at).map(u32::from_le_bytes)
}

fn read_u64(file: &[u8], at: usize) -> Result<u64, String> {
    bytes(file, at).map(u64::from_le_bytes)
}

fn to_usize(value: u64) -> Result<usize, String> {
    usize::try_from(value).map_err(|_| format!("{value} is past any file this machine can read"))
}

fn read_string(file: &[u8], at: usize) -> Result<String, String> {
    let rest = file.get(at..).ok_or("a name past the end of the file")?;
    let end = rest
        .iter()
        .position(|byte| *byte == 0)
        .ok_or("a name with no end")?;
    String::from_utf8(rest[..end].to_vec()).map_err(|_| "a name that is not UTF-8".into())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    /// A minimal ELF64 shared object: one `PT_LOAD` of the given alignment
    /// mapping the whole file, and a `PT_DYNAMIC` naming the given
    /// libraries through a string table inside it.
    fn elf64(machine: u16, align: u64, needed: &[&str]) -> Vec<u8> {
        let mut strings = vec![0_u8];
        let mut offsets = Vec::new();
        for name in needed {
            offsets.push(strings.len() as u64);
            strings.extend_from_slice(name.as_bytes());
            strings.push(0);
        }
        let phoff = 0x40_u64;
        let dynamic_at = phoff + 2 * 0x38;
        let mut dynamic = Vec::new();
        let strtab_at = dynamic_at + 16 * (offsets.len() as u64 + 2);
        for offset in &offsets {
            dynamic.extend_from_slice(&DT_NEEDED.to_le_bytes());
            dynamic.extend_from_slice(&offset.to_le_bytes());
        }
        dynamic.extend_from_slice(&DT_STRTAB.to_le_bytes());
        dynamic.extend_from_slice(&strtab_at.to_le_bytes());
        dynamic.extend_from_slice(&[0; 16]);
        let total = strtab_at + strings.len() as u64;

        let mut file = vec![0_u8; 0x40];
        file[..4].copy_from_slice(b"\x7fELF");
        file[4] = 2;
        file[5] = 1;
        file[0x12..0x14].copy_from_slice(&machine.to_le_bytes());
        file[0x20..0x28].copy_from_slice(&phoff.to_le_bytes());
        file[0x36..0x38].copy_from_slice(&0x38_u16.to_le_bytes());
        file[0x38..0x3a].copy_from_slice(&2_u16.to_le_bytes());
        let header = |kind: u32, offset: u64, size: u64, align: u64| {
            let mut h = vec![0_u8; 0x38];
            h[..4].copy_from_slice(&kind.to_le_bytes());
            h[0x08..0x10].copy_from_slice(&offset.to_le_bytes());
            h[0x10..0x18].copy_from_slice(&offset.to_le_bytes());
            h[0x20..0x28].copy_from_slice(&size.to_le_bytes());
            h[0x28..0x30].copy_from_slice(&size.to_le_bytes());
            h[0x30..0x38].copy_from_slice(&align.to_le_bytes());
            h
        };
        file.extend(header(PT_LOAD, 0, total, align));
        file.extend(header(PT_DYNAMIC, dynamic_at, dynamic.len() as u64, 8));
        file.extend(dynamic);
        file.extend(strings);
        file
    }

    /// A minimal thin Mach-O with one `LC_BUILD_VERSION`.
    fn macho(cpu: u32, platform: u32, minos: u32) -> Vec<u8> {
        let mut file = Vec::new();
        for word in [MH_MAGIC_64, cpu, 0, 6, 1, 24, 0, 0] {
            file.extend_from_slice(&word.to_le_bytes());
        }
        for word in [LC_BUILD_VERSION, 24, platform, minos, 0, 0] {
            file.extend_from_slice(&word.to_le_bytes());
        }
        file
    }

    fn row(name: &str) -> Mobile {
        Mobile::by_either(name).unwrap()
    }

    #[test]
    fn the_flutter_hook_holds_an_app_to_the_floors_the_slices_are_built_for() {
        let hook = std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../bindings/flutter/lib/src/bundle.dart"),
        )
        .expect("the hook's source");
        for line in [
            format!("const int androidFloor = {ANDROID_API};"),
            format!("const int iosFloor = {};", IOS_FLOOR.0),
            format!("const String withoutBuiltinSuffix = '{LEAN}';"),
        ] {
            assert!(hook.contains(&line), "bundle.dart must say `{line}`");
        }
        assert_eq!(IOS_FLOOR.1, 0, "the hook compares whole iOS versions");
    }

    #[test]
    fn an_ndk_at_another_release_is_passed_over_for_the_pinned_one() {
        let scratch = std::env::temp_dir().join(format!("teistro-ndk-{}", std::process::id()));
        let ndk = |name: &str, revision: &str| {
            let dir = scratch.join(name);
            fs::create_dir_all(&dir).unwrap();
            fs::write(
                dir.join("source.properties"),
                format!("Pkg.Desc = Android NDK\nPkg.Revision = {revision}\n"),
            )
            .unwrap();
            dir
        };
        let image = ndk("image", "27.3.13750724");
        let pinned = ndk("pinned", "28.2.13676358");
        let missing = scratch.join("missing");
        assert_eq!(
            pinned_ndk(
                [image.clone(), missing.clone(), pinned.clone()].into_iter(),
                "28.2.13676358"
            ),
            Ok(pinned)
        );
        let refused = pinned_ndk([image, missing].into_iter(), "28.2.13676358").unwrap_err();
        assert!(refused.contains("is 27.3.13750724"), "{refused}");
        fs::remove_dir_all(&scratch).unwrap();
    }

    #[test]
    fn every_mobile_row_is_named_once_and_apart_from_the_hosts() {
        let mut names: Vec<&str> = MOBILE.iter().map(|row| row.name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), MOBILE.len());
        for row in MOBILE {
            assert!(crate::platform::Platform::by_triple(row.triple).is_none());
            assert!(crate::platform::Platform::by_name(row.name).is_none());
            assert_eq!(Mobile::by_either(row.triple), Some(row));
        }
    }

    #[test]
    fn an_android_slice_needing_only_bionic_and_aligned_passes() {
        let file = elf64(183, 0x4000, &["libc.so", "libdl.so", "libm.so"]);
        assert_eq!(check(row("android-arm64"), &file), Ok(()));
    }

    #[test]
    fn an_android_slice_aligned_for_4_kb_is_refused_on_a_64_bit_row() {
        let file = elf64(183, 0x1000, &["libc.so"]);
        let refused = check(row("android-arm64"), &file).unwrap_err();
        assert!(refused.contains("16 KB"), "{refused}");
    }

    #[test]
    fn an_android_slice_needing_a_cpp_runtime_is_refused() {
        let file = elf64(62, 0x4000, &["libc.so", "libc++_shared.so"]);
        let refused = check(row("android-x64"), &file).unwrap_err();
        assert!(refused.contains("libc++_shared.so"), "{refused}");
    }

    #[test]
    fn an_android_slice_of_another_architecture_is_refused() {
        let file = elf64(62, 0x4000, &["libc.so"]);
        assert!(check(row("android-arm64"), &file).is_err());
    }

    #[test]
    fn an_ios_slice_is_held_to_its_platform_and_deployment_target() {
        let target = (IOS_FLOOR.0 << 16) | (IOS_FLOOR.1 << 8);
        let device = macho(CPU_TYPE_ARM64, PLATFORM_IOS, target);
        assert_eq!(check(row("ios-arm64"), &device), Ok(()));
        // The same bits are not a simulator's.
        assert!(check(row("ios-sim-arm64"), &device).is_err());
        let later = macho(CPU_TYPE_ARM64, PLATFORM_IOS, 17 << 16);
        let refused = check(row("ios-arm64"), &later).unwrap_err();
        assert!(refused.contains("17.0"), "{refused}");
        let intel = macho(CPU_TYPE_X86_64, PLATFORM_IOSSIMULATOR, target);
        assert_eq!(check(row("ios-sim-x64"), &intel), Ok(()));
        assert!(check(row("ios-sim-arm64"), &intel).is_err());
        // The arm64 simulator is held to the 14.0 its target cannot go below.
        let fourteen = IOS_SIM_ARM64_FLOOR.0 << 16;
        let silicon = macho(CPU_TYPE_ARM64, PLATFORM_IOSSIMULATOR, fourteen);
        assert_eq!(check(row("ios-sim-arm64"), &silicon), Ok(()));
        let asked = macho(CPU_TYPE_ARM64, PLATFORM_IOSSIMULATOR, target);
        assert!(check(row("ios-sim-arm64"), &asked).is_err());
    }
}
