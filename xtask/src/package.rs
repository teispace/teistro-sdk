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
//! `cargo xtask package wasm` stages the wasm package, which is one
//! artefact for every host and so is built once rather than per platform.
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
use crate::platform::{NPM_MCP, NPM_SCOPE, NPM_WASM, PLATFORMS, Platform};
use crate::{read, rel};
use crate::{release, sbom};

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

/// The agent server's program, as Cargo names its binary target.
const MCP_PROGRAM: &str = "teistro-mcp";

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

/// Builds the library, the addon and the agent server for a target, and
/// returns the directory Cargo wrote them to.
///
/// The target is always named, even when it is the host, so that the
/// output directory is the same shape on every runner and a cross-built
/// artefact is never mistaken for a native one.
///
/// A glibc row links with `cargo zigbuild` against its floor's glibc
/// (`x86_64-unknown-linux-gnu.2.28`) rather than the runner's, still on
/// its own runner, so the build is native and only the symbol versions it
/// asks for change; `floor::check` then reads them back. The output
/// directory drops the suffix, so it is the same as a plain build's.
///
/// Every row builds through `cargo auditable`, which passes any cargo
/// command through and links the crates that went in into each library
/// (`sbom::embedded` reads that back).
fn compile(root: &Path, platform: &Platform) -> Result<PathBuf, ()> {
    let (subcommand, target, hint) = match platform.glibc_floor {
        Some((major, minor)) => (
            "zigbuild",
            format!("{}.{major}.{minor}", platform.triple),
            format!(
                " (a glibc row links against GLIBC_{major}.{minor} with cargo-zigbuild: \
                 `pip install -r {ZIGBUILD_REQUIREMENTS}`)"
            ),
        ),
        None => ("build", platform.triple.to_string(), String::new()),
    };
    let auditable = fs::read_to_string(root.join(sbom::AUDITABLE_VERSION)).map_or_else(
        |_| String::from("the pinned version"),
        |text| text.trim().to_string(),
    );
    step(
        Command::new(cargo())
            .args([
                "auditable",
                subcommand,
                "--release",
                "--quiet",
                "--target",
                &target,
                "-p",
                "teistro-ffi",
                "-p",
                "teistro-node",
                "-p",
                "teistro-mcp",
            ])
            .current_dir(root),
        "",
        &format!(
            "the library, the addon and the agent server did not build for {}{hint} (every row builds through \
             cargo-auditable: `cargo install --locked cargo-auditable@{auditable}`)",
            platform.triple
        ),
    )?;
    Ok(root.join("target").join(platform.triple).join("release"))
}

/// The pinned cargo-zigbuild and zig, which the workflows install from
/// the same file a failing local build names.
const ZIGBUILD_REQUIREMENTS: &str = "xtask/zigbuild-requirements.txt";

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
    let server = built.join(platform.program(MCP_PROGRAM));
    crate::floor::check(platform, &[&shared, &addon, &server]).map_err(io::Error::other)?;
    sbom::embedded(&[&shared, &addon, &server]).map_err(io::Error::other)?;
    let library_bill = bill(root, dist, platform, version, "teistro-ffi", "library")?;
    let addon_bill = bill(root, dist, platform, version, ADDON_PACKAGE, "addon")?;
    let server_bill = bill(root, dist, platform, version, MCP_PROGRAM, "mcp")?;

    let library = gzipped_library(dist, platform, version, &shared)?;
    let bundle = c_bundle(root, dist, platform, version, built, &library_bill)?;
    let package = npm_platform_package(root, dist, platform, version, &addon, &addon_bill)?;
    let mcp = mcp_archive(root, dist, platform, version, &server, &server_bill)?;
    let mcp_package = npm_mcp_package(root, dist, platform, version, &server, &server_bill)?;

    let manifest = json!({
        "schema": SCHEMA,
        "version": version,
        "platform": platform.name(),
        "triple": platform.triple,
        // Uncompressed, because it is what a consumer loads and what the
        // Dart installer checks after it has unpacked the download.
        "library": entry(&shared, &platform.shared(LIBRARY_STEM))?,
        "addon": entry(&addon, ADDON_FILE)?,
        "archives": [library, bundle, mcp],
        "npm": package,
        "npmMcp": mcp_package,
        // What each file is made of, beside the files (`xtask/src/sbom.rs`).
        "sboms": [
            entry(&library_bill, &file_name(&library_bill))?,
            entry(&addon_bill, &file_name(&addon_bill))?,
            entry(&server_bill, &file_name(&server_bill))?,
        ],
    });
    let path = dist.join(manifest_name(version, &platform.name()));
    fs::write(&path, format!("{}\n", to_json(&manifest)))?;
    Ok(manifest)
}

/// The Node addon's package, whose bill the platform's npm package carries.
const ADDON_PACKAGE: &str = "teistro-node";

/// Writes the `CycloneDX` bill of materials for one of a platform's files
/// into `dist`, and returns where.
fn bill(
    root: &Path,
    dist: &Path,
    platform: &Platform,
    version: &str,
    package: &str,
    what: &str,
) -> io::Result<PathBuf> {
    let path = dist.join(format!(
        "teistro-{version}-{}-{what}.cdx.json",
        platform.name()
    ));
    sbom::write(root, platform.triple, package, &path).map_err(io::Error::other)?;
    Ok(path)
}

/// A path's last component, as a manifest names a file.
fn file_name(path: &Path) -> String {
    path.file_name()
        .map_or_else(String::new, |name| name.to_string_lossy().into_owned())
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
    bill: &Path,
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
    append(&mut archive, &format!("{stem}/{}", sbom::FILE), bill)?;
    archive.into_inner()?.finish()?;
    entry(&path, &name)
}

/// The agent server's archive (`03-design/mcp-server.md` §6 step 5): the
/// program, its README, the terms and its bill, under one directory so it
/// unpacks to a place of its own. A `.tar.gz` on every platform, as the C
/// bundle is.
fn mcp_archive(
    root: &Path,
    dist: &Path,
    platform: &Platform,
    version: &str,
    server: &Path,
    bill: &Path,
) -> io::Result<Value> {
    let name = mcp_archive_name(version, &platform.name());
    let path = dist.join(&name);
    let encoder = GzEncoder::new(File::create(&path)?, Compression::best());
    let mut archive = tar::Builder::new(encoder);
    append_mode(
        &mut archive,
        &format!("{MCP_PROGRAM}/{}", platform.program(MCP_PROGRAM)),
        server,
        0o755,
    )?;
    append(
        &mut archive,
        &format!("{MCP_PROGRAM}/README.md"),
        &root.join("crates/mcp/README.md"),
    )?;
    for legal in LEGAL {
        append(
            &mut archive,
            &format!("{MCP_PROGRAM}/{legal}"),
            &root.join(legal),
        )?;
    }
    append(&mut archive, &format!("{MCP_PROGRAM}/{}", sbom::FILE), bill)?;
    archive.into_inner()?.finish()?;
    entry(&path, &name)
}

/// The agent server's archive's file name, which `check-package` unpacks.
pub(crate) fn mcp_archive_name(version: &str, platform: &str) -> String {
    format!("{MCP_PROGRAM}-{version}-{platform}.tar.gz")
}

/// Appends one file under a name, with the header fields a build machine
/// would otherwise vary: no owner, no modification time, one mode. Two
/// runs of the same source produce the same archive.
fn append<W: io::Write>(archive: &mut tar::Builder<W>, name: &str, from: &Path) -> io::Result<()> {
    append_mode(archive, name, from, 0o644)
}

/// [`append`], with the mode a program needs to run once unpacked.
fn append_mode<W: io::Write>(
    archive: &mut tar::Builder<W>,
    name: &str,
    from: &Path,
    mode: u32,
) -> io::Result<()> {
    let data = fs::read(from)
        .map_err(|err| io::Error::other(format!("cannot read {}: {err}", from.display())))?;
    let mut header = tar::Header::new_gnu();
    header.set_size(data.len() as u64);
    header.set_mode(mode);
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
    bill: &Path,
) -> io::Result<Value> {
    let carried = Carried {
        package: platform.npm_package(),
        file: ADDON_FILE.to_string(),
        from: addon,
        what: "prebuilt Node addon",
        instead: NPM_SCOPE,
        source: "bindings/node",
    };
    let directory = platform_package(root, dist, platform, version, &carried, bill)?;
    let addon = entry(&directory.join(ADDON_FILE), ADDON_FILE)?;
    Ok(json!({
        "package": carried.package,
        "directory": rel(root, &directory),
        "addon": addon,
    }))
}

/// Stages the npm package that carries this platform's agent server, which
/// `@teistro/mcp`'s launcher runs.
fn npm_mcp_package(
    root: &Path,
    dist: &Path,
    platform: &Platform,
    version: &str,
    server: &Path,
    bill: &Path,
) -> io::Result<Value> {
    let carried = Carried {
        package: platform.npm_mcp_package(),
        file: platform.program(MCP_PROGRAM),
        from: server,
        what: "prebuilt agent server (`teistro-mcp`)",
        instead: NPM_MCP,
        source: "crates/mcp/npm",
    };
    let directory = platform_package(root, dist, platform, version, &carried, bill)?;
    Ok(json!({
        "package": carried.package,
        "directory": rel(root, &directory),
        "program": entry(&directory.join(&carried.file), &carried.file)?,
    }))
}

/// One file a platform's npm package carries, and the package a consumer
/// installs instead, which depends on it.
struct Carried<'a> {
    package: String,
    file: String,
    from: &'a Path,
    what: &'static str,
    instead: &'static str,
    source: &'static str,
}

/// Writes a platform package: the file, the terms, the bill, a manifest
/// npm can match against a host, and a readme that says what to install
/// instead. Returns its directory.
fn platform_package(
    root: &Path,
    dist: &Path,
    platform: &Platform,
    version: &str,
    carried: &Carried<'_>,
    bill: &Path,
) -> io::Result<PathBuf> {
    let name = &carried.package;
    let directory = dist.join("npm").join(name);
    fs::create_dir_all(&directory)?;
    // `fs::copy` keeps the mode, so a program stays runnable, and npm
    // keeps it in the tarball.
    fs::copy(carried.from, directory.join(&carried.file))?;
    for legal in LEGAL {
        fs::copy(root.join(legal), directory.join(legal))?;
    }
    fs::copy(bill, directory.join(sbom::FILE))?;

    let (what, instead) = (carried.what, carried.instead);
    let mut manifest = Map::new();
    manifest.insert("name".to_string(), json!(name));
    manifest.insert("version".to_string(), json!(version));
    manifest.insert(
        "description".to_string(),
        json!(format!(
            "The Teistro SDK's {what} for {}. Install {instead}, which depends on this.",
            described(platform)
        )),
    );
    manifest.insert("license".to_string(), json!("Apache-2.0"));
    manifest.insert(
        "repository".to_string(),
        json!({
            "type": "git",
            "url": "git+https://github.com/teispace/teistro-sdk.git",
            "directory": carried.source,
        }),
    );
    manifest.insert("os".to_string(), json!([platform.os]));
    manifest.insert("cpu".to_string(), json!([platform.cpu]));
    if let Some(libc) = platform.libc {
        manifest.insert("libc".to_string(), json!([libc]));
    }
    // The SDK's own floor, which `node-is-tested-at-its-floor` holds every
    // package to, so a platform package never promises an older Node.
    let floor = serde_json::from_str::<Value>(&read(&root.join("bindings/node/package.json")))
        .ok()
        .and_then(|node| node["engines"]["node"].as_str().map(str::to_owned))
        .ok_or_else(|| io::Error::other("bindings/node/package.json states no `engines.node`"))?;
    manifest.insert("engines".to_string(), json!({ "node": floor }));
    let mut files = vec![json!(carried.file), json!(sbom::FILE)];
    files.extend(LEGAL.map(|legal| json!(legal)));
    manifest.insert("files".to_string(), Value::Array(files));
    fs::write(
        directory.join("package.json"),
        format!("{}\n", to_json(&Value::Object(manifest))),
    )?;
    fs::write(
        directory.join("README.md"),
        format!(
            "# {name}\n\nThe Teistro SDK's {what} for {}.\n\nThis package holds one file and no code. Install [`{instead}`](https://www.npmjs.com/package/{instead}) instead: it depends on this package for the host it is installed on, and loads the file from it.\n\nApache-2.0. The sources are at <https://github.com/teispace/teistro-sdk>.\n",
            described(platform)
        ),
    )?;
    Ok(directory)
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
/// Builds the wasm module and stages its package where the release
/// collects the npm packages, beside the platform ones.
pub(crate) fn wasm(root: &Path) -> i32 {
    let directory = wasm_package(root);
    match crate::wasm_binding::stage(root, &directory) {
        Ok(()) => {
            println!("wrote {}", rel(root, &directory));
            0
        }
        Err(()) => 1,
    }
}

/// Where the wasm package is staged.
fn wasm_package(root: &Path) -> PathBuf {
    root.join(DIST).join("npm").join(NPM_WASM)
}

pub(crate) fn stage(root: &Path, partial: bool) -> i32 {
    let version = release::version(root);
    let dist = root.join(DIST);
    let mut platforms = Map::new();
    let mut missing = Vec::new();
    // The wasm package is built by its own job, not a platform's, so it is
    // checked for here as a platform's manifest is: a release without it
    // would publish every package but one.
    if !wasm_package(root).join("package.json").is_file() {
        missing.push(rel(root, &wasm_package(root)));
    }
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
                "{}  {path} is missing",
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

/// Writes the merged manifest, the checksum list and every package.
///
/// The Java package is staged first, since the merged manifest records
/// it (`maven`) and the checksum list lists its files.
fn write_stage(root: &Path, dist: &Path, version: &str, merged: &Value) -> io::Result<Vec<String>> {
    let mut written = Vec::new();
    let mut merged = merged.clone();
    if let Some((jars, record)) = crate::java_package::stage(root, dist, version, &merged)? {
        written.extend(jars);
        merged["maven"] = record;
    }
    let merged = &merged;
    let manifest = dist.join("manifest.json");
    fs::write(&manifest, format!("{}\n", to_json(merged)))?;
    written.push(rel(root, &manifest));

    let checksums = dist.join("checksums.txt");
    fs::write(&checksums, checksum_list(merged))?;
    written.push(rel(root, &checksums));

    written.push(stage_node(root, dist)?);
    written.push(stage_mcp_node(root, dist)?);
    written.push(stage_dart(root, dist, version, merged)?);
    written.push(stage_python(root, dist, version, merged)?);
    written.extend(python_wheels(root, dist, version, merged)?);
    Ok(written)
}

/// Where the platform wheels are staged, beside the Python package they
/// are built from.
const WHEELS: &str = "pypi/wheels";

/// A wheel for every platform the matrix built, each carrying that
/// platform's library (`wheel.rs`). The library is the one the platform's
/// gzipped archive holds, checked against the digest its manifest
/// recorded before it is packed, so a wheel carries the bits the release
/// lists.
fn python_wheels(
    root: &Path,
    dist: &Path,
    version: &str,
    merged: &Value,
) -> io::Result<Vec<String>> {
    let staged = dist.join("pypi").join("teistro");
    let into = dist.join(WHEELS);
    if into.exists() {
        fs::remove_dir_all(&into)?;
    }
    let mut written = Vec::new();
    for (name, manifest) in merged["platforms"].as_object().into_iter().flatten() {
        let platform = Platform::by_name(name)
            .ok_or_else(|| io::Error::other(format!("{name} is not a shipped platform")))?;
        let library = verified_library(dist, name, manifest)?;
        let wheel = crate::wheel::write(&staged, &into, version, &platform, &library)?;
        written.push(rel(root, &into.join(wheel)));
    }
    Ok(written)
}

/// A platform's library, as its gzipped archive holds it, checked against
/// the digest its manifest recorded: what every package that carries the
/// library carries, so each carries the bits the release lists.
pub(crate) fn verified_library(dist: &Path, name: &str, manifest: &Value) -> io::Result<Vec<u8>> {
    let archive = manifest["archives"][0]["file"]
        .as_str()
        .ok_or_else(|| io::Error::other(format!("{name}'s manifest names no library archive")))?;
    let mut library = Vec::new();
    io::copy(
        &mut flate2::read::GzDecoder::new(File::open(dist.join(archive))?),
        &mut library,
    )?;
    let recorded = manifest["library"]["sha256"].as_str().unwrap_or_default();
    if hex(&Sha256::digest(&library)) != recorded {
        return Err(io::Error::other(format!(
            "{archive} does not hold the library {name}'s manifest records"
        )));
    }
    Ok(library)
}

/// Every archive's and every bill's digest, in the format `sha256sum -c`
/// reads, so that a download can be checked with the tool already on the
/// machine.
fn checksum_list(merged: &Value) -> String {
    let mut lines = Vec::new();
    if let Some(platforms) = merged["platforms"].as_object() {
        for platform in platforms.values() {
            let files = ["archives", "sboms"]
                .into_iter()
                .flat_map(|list| platform[list].as_array().into_iter().flatten());
            for archive in files {
                lines.push(format!(
                    "{}  {}",
                    archive["sha256"].as_str().unwrap_or_default(),
                    archive["file"].as_str().unwrap_or_default()
                ));
            }
        }
    }
    for file in merged["maven"]["files"].as_array().into_iter().flatten() {
        lines.push(format!(
            "{}  {}",
            file["sha256"].as_str().unwrap_or_default(),
            file["file"].as_str().unwrap_or_default()
        ));
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

/// Stages the agent server's npm launcher: its manifest, which names the
/// platform packages, the launcher, the server's README and the terms. No
/// program: it comes from whichever platform package npm installed.
fn stage_mcp_node(root: &Path, dist: &Path) -> io::Result<String> {
    let directory = dist.join("npm").join(NPM_MCP);
    if directory.exists() {
        fs::remove_dir_all(&directory)?;
    }
    fs::create_dir_all(directory.join("bin"))?;
    let source = root.join("crates/mcp/npm");
    fs::copy(source.join("package.json"), directory.join("package.json"))?;
    fs::copy(
        source.join("bin/teistro-mcp.js"),
        directory.join("bin/teistro-mcp.js"),
    )?;
    fs::copy(
        root.join("crates/mcp/README.md"),
        directory.join("README.md"),
    )?;
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

/// Stages the Python package: the sources as they are, and the digest
/// table its installer holds a download to, written from what the matrix
/// built.
fn stage_python(root: &Path, dist: &Path, version: &str, merged: &Value) -> io::Result<String> {
    let directory = dist.join("pypi").join("teistro");
    if directory.exists() {
        fs::remove_dir_all(&directory)?;
    }
    fs::create_dir_all(&directory)?;
    let source = root.join("bindings/python");
    copy_tree(&source.join("teistro"), &directory.join("teistro"))?;
    copy_tree(&source.join("example"), &directory.join("example"))?;
    for file in ["pyproject.toml", "README.md"] {
        fs::copy(source.join(file), directory.join(file))?;
    }
    for legal in LEGAL {
        fs::copy(root.join(legal), directory.join(legal))?;
    }
    fs::write(
        directory.join("teistro/_prebuilt.py"),
        python_prebuilt_table(root, version, merged),
    )?;
    Ok(rel(root, &directory))
}

/// The digests of every platform's library, as the release recorded them.
fn digest_rows(merged: &Value, quote: char) -> Vec<String> {
    let mut rows = Vec::new();
    if let Some(platforms) = merged["platforms"].as_object() {
        for (name, platform) in platforms {
            rows.push(format!(
                "  {quote}{name}{quote}: {quote}{}{quote},",
                platform["library"]["sha256"].as_str().unwrap_or_default()
            ));
        }
    }
    rows.sort();
    rows
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
    format!(
        "{head}const Map<String, String> prebuiltDigests = <String, String>{{\n{}\n}};\n",
        digest_rows(merged, '\'').join("\n")
    )
}

/// The same table for Python, from the same manifest and the same
/// checked-in header.
fn python_prebuilt_table(root: &Path, version: &str, merged: &Value) -> String {
    let checked_in = read(&root.join("bindings/python/teistro/_prebuilt.py"));
    let head = checked_in
        .split_once("PREBUILT_DIGESTS: Final[dict[str, str]]")
        .map_or(checked_in.clone(), |(head, _)| head.to_string())
        .replace(
            "PREBUILT_VERSION: Final = \"0.0.0\"",
            &format!("PREBUILT_VERSION: Final = \"{version}\""),
        );
    format!(
        "{head}PREBUILT_DIGESTS: Final[dict[str, str]] = {{\n{}\n}}\n",
        digest_rows(merged, '"').join("\n")
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
pub(crate) fn copy_tree(from: &Path, to: &Path) -> io::Result<()> {
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
                "linux-x64": {
                    "archives": [ { "file": "b.gz", "sha256": "bb" } ],
                    "sboms": [ { "file": "b.cdx.json", "sha256": "cc" } ],
                },
                "darwin-arm64": { "archives": [ { "file": "a.gz", "sha256": "aa" } ] },
            }
        });
        assert_eq!(
            checksum_list(&merged),
            "aa  a.gz\nbb  b.gz\ncc  b.cdx.json\n"
        );
    }
}
