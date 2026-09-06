//! What a release ships, built and staged: one platform's artefacts, and
//! the packages that carry them.
//!
//! `cargo xtask package` runs on one machine and produces what that
//! platform ships: the shared library on its own, gzipped, for the Dart
//! installer; a bundle with the header and both libraries for a C
//! consumer; and the npm package that carries this platform's addon.
//! Every file is recorded in a manifest with its size and its SHA-256,
//! and the library's digest is recorded uncompressed, so that whoever
//! fetches it verifies the bits they will load rather than the framing
//! they arrived in.
//!
//! `cargo xtask package stage` runs once, after the matrix has produced
//! every platform's manifest: it merges them, writes the digest table the
//! Dart installer holds downloads to, and stages the two packages that
//! are published from one place — the Node package that depends on the
//! platform packages, and the Dart package that fetches from the release.
//!
//! Nothing here publishes. The commands write into `target/dist`, and the
//! release workflow is what uploads and publishes, so that the same steps
//! can be run and inspected on a laptop.

use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use flate2::Compression;
use flate2::write::GzEncoder;
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

use crate::binding::{LIBRARY_STEM, cargo, step};
use crate::hashes::hex;
use crate::node_binding::ADDON_STEM;
use crate::platform::{NPM_SCOPE, PLATFORMS, Platform};
use crate::release;
use crate::{read, rel};

/// The manifest's schema, versioned like every other file this repository
/// writes for someone else to read.
const SCHEMA: &str = "teistro-release/1";

/// Where the artefacts are written. Inside `target`, so that a clean
/// checkout has none and `cargo clean` removes them.
const DIST: &str = "target/dist";

/// The file name the addon takes inside its platform package. It is not
/// `index.node`, because a package holding one file should say what the
/// file is.
const ADDON_FILE: &str = "teistro.node";

/// The files every package carries whatever else is in it.
const LEGAL: [&str; 2] = ["LICENSE", "NOTICE"];

// ── one platform ───────────────────────────────────────────────────────────

/// Builds this platform's artefacts and writes its manifest.
///
/// `target` names a platform by either of the two names it has, the Rust
/// triple or the short one artefacts carry (`linux-x64`), because a
/// person reading a release page has the second and a person reading a
/// build log has the first. Without one, this machine is packaged.
pub(crate) fn build(root: &Path, target: Option<&str>) -> i32 {
    let platform = match target {
        Some(target) => {
            let Some(platform) = Platform::by_triple(target).or_else(|| Platform::by_name(target))
            else {
                eprintln!("{target} is not a platform the SDK ships; {}", shipped());
                return 2;
            };
            platform
        }
        None => Platform::host(),
    };
    if Platform::by_triple(platform.triple).is_none() {
        eprintln!(
            "this machine is not a platform the SDK ships (`{}`); {}",
            platform.triple,
            shipped()
        );
        return 2;
    }
    let version = release::version(root);
    let dist = root.join(DIST);
    let Ok(built) = compile(root, &platform) else {
        return 1;
    };
    match stage_platform(root, &dist, &platform, &version, &built) {
        Ok(manifest) => {
            println!(
                "{} {version} for {}: {} artefact(s) in {}",
                NPM_SCOPE,
                platform.name(),
                manifest["archives"].as_array().map_or(0, Vec::len) + 1,
                rel(root, &dist)
            );
            0
        }
        Err(err) => {
            println!("FAIL  {} could not be packaged: {err}", platform.name());
            1
        }
    }
}

/// The platforms the release matrix builds, as a sentence.
fn shipped() -> String {
    let names: Vec<String> = PLATFORMS.iter().map(Platform::name).collect();
    format!("the SDK ships {}", names.join(", "))
}

/// Builds the library and the addon for a target, and returns the
/// directory Cargo wrote them to.
///
/// The target is always named, even when it is the host, so that the
/// output directory is the same shape on every runner and a cross-built
/// artefact is never mistaken for a native one.
fn compile(root: &Path, platform: &Platform) -> Result<PathBuf, ()> {
    step(
        Command::new(cargo())
            .args([
                "build",
                "--release",
                "--quiet",
                "--target",
                platform.triple,
                "-p",
                "teistro-ffi",
                "-p",
                "teistro-node",
            ])
            .current_dir(root),
        "",
        &format!(
            "the library and the addon did not build for {}",
            platform.triple
        ),
    )?;
    Ok(root.join("target").join(platform.triple).join("release"))
}

/// Writes everything one platform ships, and its manifest.
fn stage_platform(
    root: &Path,
    dist: &Path,
    platform: &Platform,
    version: &str,
    built: &Path,
) -> io::Result<Value> {
    fs::create_dir_all(dist)?;
    let shared = built.join(platform.shared(LIBRARY_STEM));
    let addon = built.join(platform.shared(ADDON_STEM));

    let library = gzipped_library(dist, platform, version, &shared)?;
    let bundle = c_bundle(root, dist, platform, version, built)?;
    let package = npm_platform_package(root, dist, platform, version, &addon)?;

    let manifest = json!({
        "schema": SCHEMA,
        "version": version,
        "platform": platform.name(),
        "triple": platform.triple,
        // Uncompressed, because it is what a consumer loads and what the
        // Dart installer checks after it has unpacked the download.
        "library": entry(&shared, &platform.shared(LIBRARY_STEM))?,
        "addon": entry(&addon, ADDON_FILE)?,
        "archives": [library, bundle],
        "npm": package,
    });
    let path = dist.join(manifest_name(version, &platform.name()));
    fs::write(&path, format!("{}\n", to_json(&manifest)))?;
    Ok(manifest)
}

/// The per-platform manifest's file name, which `stage` looks for.
fn manifest_name(version: &str, platform: &str) -> String {
    format!("teistro-{version}-{platform}.json")
}

/// The shared library on its own, gzipped: the smallest thing that can be
/// downloaded and loaded, and what the Dart installer fetches. Gzip alone,
/// with no archive around it, so that unpacking it needs nothing but the
/// decompressor every language already has.
fn gzipped_library(
    dist: &Path,
    platform: &Platform,
    version: &str,
    shared: &Path,
) -> io::Result<Value> {
    // Versioned and platformed before the extension, so that the name
    // sorts beside its siblings on a release page and still ends in the
    // suffix a decompressor expects.
    let shared_name = platform.shared(LIBRARY_STEM);
    let (stem, extension) = shared_name
        .rsplit_once('.')
        .unwrap_or((shared_name.as_str(), "so"));
    let name = format!("{stem}-{version}-{}.{extension}.gz", platform.name());
    let path = dist.join(&name);
    let mut encoder = GzEncoder::new(File::create(&path)?, Compression::best());
    io::copy(&mut File::open(shared)?, &mut encoder)?;
    encoder.finish()?;
    entry(&path, &name)
}

/// The bundle a C consumer unpacks: the header, both libraries, and the
/// terms they are under. A `.tar.gz` on every platform, Windows included,
/// because Windows has carried `tar` since 2018 and one archive format is
/// one code path here and one instruction in the documentation.
fn c_bundle(
    root: &Path,
    dist: &Path,
    platform: &Platform,
    version: &str,
    built: &Path,
) -> io::Result<Value> {
    let stem = format!("teistro-c-{version}-{}", platform.name());
    let name = format!("{stem}.tar.gz");
    let path = dist.join(&name);
    let encoder = GzEncoder::new(File::create(&path)?, Compression::best());
    let mut archive = tar::Builder::new(encoder);

    append(
        &mut archive,
        &format!("{stem}/include/teistro.h"),
        &root.join("bindings/c/include/teistro.h"),
    )?;
    let mut libraries = vec![
        platform.shared(LIBRARY_STEM),
        platform.static_library(LIBRARY_STEM),
    ];
    libraries.extend(platform.import_library(LIBRARY_STEM));
    for library in &libraries {
        append(
            &mut archive,
            &format!("{stem}/lib/{library}"),
            &built.join(library),
        )?;
    }
    append(
        &mut archive,
        &format!("{stem}/README.md"),
        &root.join("bindings/c/README.md"),
    )?;
    for legal in LEGAL {
        append(&mut archive, &format!("{stem}/{legal}"), &root.join(legal))?;
    }
    archive.into_inner()?.finish()?;
    entry(&path, &name)
}

/// Appends one file under a name, with the header fields a build machine
/// would otherwise vary: no owner, no modification time, one mode. Two
/// runs of the same source produce the same archive.
fn append<W: io::Write>(archive: &mut tar::Builder<W>, name: &str, from: &Path) -> io::Result<()> {
    let data = fs::read(from)
        .map_err(|err| io::Error::other(format!("cannot read {}: {err}", from.display())))?;
    let mut header = tar::Header::new_gnu();
    header.set_size(data.len() as u64);
    header.set_mode(0o644);
    header.set_mtime(0);
    header.set_uid(0);
    header.set_gid(0);
    header.set_entry_type(tar::EntryType::Regular);
    archive.append_data(&mut header, name, data.as_slice())
}

/// Stages the npm package that carries this platform's addon: the addon,
/// the terms, a manifest npm can match against a host, and a readme that
/// says what to install instead.
fn npm_platform_package(
    root: &Path,
    dist: &Path,
    platform: &Platform,
    version: &str,
    addon: &Path,
) -> io::Result<Value> {
    let name = platform.npm_package();
    let directory = dist.join("npm").join(&name);
    fs::create_dir_all(&directory)?;
    fs::copy(addon, directory.join(ADDON_FILE))?;
    for legal in LEGAL {
        fs::copy(root.join(legal), directory.join(legal))?;
    }

    let mut manifest = Map::new();
    manifest.insert("name".to_string(), json!(name));
    manifest.insert("version".to_string(), json!(version));
    manifest.insert(
        "description".to_string(),
        json!(format!(
            "The Teistro SDK's prebuilt Node addon for {}. Install {NPM_SCOPE}, which depends on this.",
            described(platform)
        )),
    );
    manifest.insert("license".to_string(), json!("Apache-2.0"));
    manifest.insert(
        "repository".to_string(),
        json!({
            "type": "git",
            "url": "git+https://github.com/teispace/teistro-sdk.git",
            "directory": "bindings/node",
        }),
    );
    manifest.insert("os".to_string(), json!([platform.os]));
    manifest.insert("cpu".to_string(), json!([platform.cpu]));
    if let Some(libc) = platform.libc {
        manifest.insert("libc".to_string(), json!([libc]));
    }
    manifest.insert("engines".to_string(), json!({ "node": ">=20" }));
    let mut files = vec![json!(ADDON_FILE)];
    files.extend(LEGAL.map(|legal| json!(legal)));
    manifest.insert("files".to_string(), Value::Array(files));
    fs::write(
        directory.join("package.json"),
        format!("{}\n", to_json(&Value::Object(manifest))),
    )?;
    fs::write(
        directory.join("README.md"),
        format!(
            "# {name}\n\nThe Teistro SDK's prebuilt Node addon for {}.\n\nThis package holds one file and no code. Install [`{NPM_SCOPE}`](https://www.npmjs.com/package/{NPM_SCOPE}) instead: it depends on this package for the host it is installed on, and loads the addon from it.\n\nApache-2.0. The sources are at <https://github.com/teispace/teistro-sdk>.\n",
            described(platform)
        ),
    )?;
    let addon = entry(&directory.join(ADDON_FILE), ADDON_FILE)?;
    Ok(json!({
        "package": name,
        "directory": rel(root, &directory),
        "addon": addon,
    }))
}

/// A platform in words, for a description a person reads.
fn described(platform: &Platform) -> String {
    let os = match platform.os {
        "darwin" => "macOS",
        "win32" => "Windows",
        _ => "Linux",
    };
    let cpu = match platform.cpu {
        "arm64" => "arm64",
        _ => "x86-64",
    };
    match platform.libc {
        Some("musl") => format!("{os} on {cpu} (musl)"),
        _ => format!("{os} on {cpu}"),
    }
}

// ── every platform ─────────────────────────────────────────────────────────

/// Merges the platforms' manifests and stages the two packages that are
/// published once: the Node package that depends on the platform
/// packages, and the Dart package whose installer fetches from the
/// release.
///
/// A release stages every platform: a Dart installer whose table is
/// missing a row would tell that platform's users to build from source
/// after they had installed a release built for them. `partial` is for
/// trying the packaging on one machine, and says so in what it prints.
pub(crate) fn stage(root: &Path, partial: bool) -> i32 {
    let version = release::version(root);
    let dist = root.join(DIST);
    let mut platforms = Map::new();
    let mut missing = Vec::new();
    for platform in PLATFORMS {
        let path = dist.join(manifest_name(&version, &platform.name()));
        match fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        {
            Some(manifest) => {
                platforms.insert(platform.name(), manifest);
            }
            None => missing.push(rel(root, &path)),
        }
    }
    if !missing.is_empty() {
        for path in &missing {
            println!(
                "{}  no manifest at {path}",
                if partial { "note" } else { "FAIL" }
            );
        }
        println!(
            "the matrix has not produced every platform: {} of {} present",
            platforms.len(),
            PLATFORMS.len()
        );
        if !partial {
            return 1;
        }
    }
    let merged = json!({
        "schema": SCHEMA,
        "version": version,
        "platforms": platforms,
    });
    match write_stage(root, &dist, &version, &merged) {
        Ok(paths) => {
            for path in &paths {
                println!("wrote {path}");
            }
            println!(
                "{version} staged for {} of {} platform(s){}; nothing is published by this command",
                merged["platforms"]
                    .as_object()
                    .map_or(0, serde_json::Map::len),
                PLATFORMS.len(),
                if partial { ", a partial stage" } else { "" }
            );
            0
        }
        Err(err) => {
            println!("FAIL  the release could not be staged: {err}");
            1
        }
    }
}

/// Writes the merged manifest, the checksum list and both packages.
fn write_stage(root: &Path, dist: &Path, version: &str, merged: &Value) -> io::Result<Vec<String>> {
    let mut written = Vec::new();
    let manifest = dist.join("manifest.json");
    fs::write(&manifest, format!("{}\n", to_json(merged)))?;
    written.push(rel(root, &manifest));

    let checksums = dist.join("checksums.txt");
    fs::write(&checksums, checksum_list(merged))?;
    written.push(rel(root, &checksums));

    written.push(stage_node(root, dist)?);
    written.push(stage_dart(root, dist, version, merged)?);
    Ok(written)
}

/// Every archive's digest, in the format `sha256sum -c` reads, so that a
/// download can be checked with the tool already on the machine.
fn checksum_list(merged: &Value) -> String {
    let mut lines = Vec::new();
    if let Some(platforms) = merged["platforms"].as_object() {
        for platform in platforms.values() {
            for archive in platform["archives"].as_array().into_iter().flatten() {
                lines.push(format!(
                    "{}  {}",
                    archive["sha256"].as_str().unwrap_or_default(),
                    archive["file"].as_str().unwrap_or_default()
                ));
            }
        }
    }
    lines.sort();
    format!("{}\n", lines.join("\n"))
}

/// Stages the Node package a consumer installs: the generated and
/// hand-written layers, the manifest that names the platform packages,
/// and the terms. No addon: it comes from whichever platform package npm
/// installed.
fn stage_node(root: &Path, dist: &Path) -> io::Result<String> {
    let directory = dist.join("npm").join(NPM_SCOPE);
    if directory.exists() {
        fs::remove_dir_all(&directory)?;
    }
    fs::create_dir_all(directory.join("lib"))?;
    let source = root.join("bindings/node");
    copy_tree(&source.join("lib"), &directory.join("lib"))?;
    for file in ["package.json", "README.md"] {
        fs::copy(source.join(file), directory.join(file))?;
    }
    for legal in LEGAL {
        fs::copy(root.join(legal), directory.join(legal))?;
    }
    Ok(rel(root, &directory))
}

/// Stages the Dart package: the sources as they are, and the digest table
/// its installer holds a download to, written from what the matrix built.
fn stage_dart(root: &Path, dist: &Path, version: &str, merged: &Value) -> io::Result<String> {
    let directory = dist.join("pub").join("teistro");
    if directory.exists() {
        fs::remove_dir_all(&directory)?;
    }
    fs::create_dir_all(&directory)?;
    let source = root.join("bindings/dart");
    copy_tree(&source.join("lib"), &directory.join("lib"))?;
    copy_tree(&source.join("example"), &directory.join("example"))?;
    // Only the installer, of the two commands in `bin/`: the other is the
    // parity harness, which belongs to this repository and not to a
    // consumer's project.
    fs::create_dir_all(directory.join("bin"))?;
    fs::copy(
        source.join("bin/install.dart"),
        directory.join("bin/install.dart"),
    )?;
    for file in ["pubspec.yaml", "README.md", "analysis_options.yaml"] {
        fs::copy(source.join(file), directory.join(file))?;
    }
    for legal in LEGAL {
        fs::copy(root.join(legal), directory.join(legal))?;
    }
    fs::write(
        directory.join("lib/src/prebuilt.dart"),
        prebuilt_table(root, version, merged),
    )?;
    Ok(rel(root, &directory))
}

/// The Dart file that says where a prebuilt library is and what it must
/// hash to. The header is the checked-in file's, so that the two differ in
/// the table alone and a reader can see what the release added.
fn prebuilt_table(root: &Path, version: &str, merged: &Value) -> String {
    let checked_in = read(&root.join("bindings/dart/lib/src/prebuilt.dart"));
    let head = checked_in
        .split_once("const Map<String, String> prebuiltDigests")
        .map_or(checked_in.clone(), |(head, _)| head.to_string())
        .replace(
            "const String prebuiltVersion = '0.0.0';",
            &format!("const String prebuiltVersion = '{version}';"),
        );
    let mut rows = Vec::new();
    if let Some(platforms) = merged["platforms"].as_object() {
        for (name, platform) in platforms {
            rows.push(format!(
                "  '{name}': '{}',",
                platform["library"]["sha256"].as_str().unwrap_or_default()
            ));
        }
    }
    rows.sort();
    format!(
        "{head}const Map<String, String> prebuiltDigests = <String, String>{{\n{}\n}};\n",
        rows.join("\n")
    )
}

// ── the small shared things ────────────────────────────────────────────────

/// One file's size and digest, under the name it is published as.
fn entry(path: &Path, name: &str) -> io::Result<Value> {
    let data = fs::read(path)
        .map_err(|err| io::Error::other(format!("cannot read {}: {err}", path.display())))?;
    Ok(json!({
        "file": name,
        "bytes": data.len(),
        "sha256": hex(&Sha256::digest(&data)),
    }))
}

/// Copies a directory recursively, skipping what a package never carries:
/// build output, dependency trees and the tooling's own caches.
fn copy_tree(from: &Path, to: &Path) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if matches!(
            name.as_ref(),
            "node_modules" | ".dart_tool" | "target" | "build" | ".packages"
        ) {
            continue;
        }
        let source = entry.path();
        let destination = to.join(name.as_ref());
        if source.is_dir() {
            copy_tree(&source, &destination)?;
        } else {
            fs::copy(&source, &destination)?;
        }
    }
    Ok(())
}

/// JSON as this repository writes it: two spaces, keys in the order they
/// were inserted.
fn to_json(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| String::from("{}"))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{checksum_list, described, manifest_name};
    use crate::platform::Platform;

    #[test]
    fn a_platform_reads_as_a_sentence() {
        let mac = Platform::by_name("darwin-arm64").expect("a shipped platform");
        assert_eq!(described(&mac), "macOS on arm64");
        let linux = Platform::by_name("linux-x64").expect("a shipped platform");
        assert_eq!(described(&linux), "Linux on x86-64");
    }

    #[test]
    fn the_manifest_is_named_for_its_platform() {
        assert_eq!(
            manifest_name("1.2.3", "win32-x64"),
            "teistro-1.2.3-win32-x64.json"
        );
    }

    #[test]
    fn checksums_are_what_sha256sum_reads() {
        let merged = json!({
            "platforms": {
                "linux-x64": { "archives": [ { "file": "b.gz", "sha256": "bb" } ] },
                "darwin-arm64": { "archives": [ { "file": "a.gz", "sha256": "aa" } ] },
            }
        });
        assert_eq!(checksum_list(&merged), "aa  a.gz\nbb  b.gz\n");
    }
}
