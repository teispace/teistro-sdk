//! The Java package as Maven Central takes it (`03-design/java-binding.md`
//! §9): one default jar carrying every platform's library, one classifier
//! jar per platform carrying its own, the sources and javadoc jars, the
//! POM, and each file's checksums, in Maven's repository layout under
//! `target/dist/maven`. Nothing here signs or uploads:
//! `cargo xtask publish maven` does, from this layout.
//!
//! Written without Maven or Gradle: `javac` and `javadoc` from the JDK,
//! and the jars by `crate::zip`, so the entry order and dates are fixed
//! and two stagings of one commit write the same bytes. The classes of
//! the default jar and of every classifier jar are the same bytes, since
//! the one table that differs per release (`Prebuilt.java`, each
//! library's digest) is compiled in once.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use md5::Md5;
use serde_json::{Value, json};
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};

use crate::binding::{LIBRARY_STEM, present};
use crate::hashes::hex;
use crate::java_binding::{FLOOR, PACKAGE, SOURCES, argfile, java_files, javac};
use crate::package::verified_library;
use crate::platform::Platform;
use crate::rel;
use crate::zip::{self, Entry};

/// The Maven coordinate's group.
pub(crate) const GROUP: &str = "com.teispace";
/// The Maven coordinate's artifact.
pub(crate) const ARTIFACT: &str = "teistro";
/// The repository layout's root under `target/dist`.
pub(crate) const MAVEN: &str = "maven";
/// Where the build's intermediate trees go under `target/dist`.
const BUILD: &str = "maven-build";
/// The checked-in digest table the stage rewrites.
const PREBUILT: &str = "com/teispace/teistro/Prebuilt.java";
/// The line the table follows, which the rewrite keeps everything above.
const PREBUILT_TABLE: &str = "    static final Map<String, String> DIGESTS";
/// The checksum files Maven reads beside each file, by extension.
pub(crate) const CHECKSUMS: [&str; 4] = ["md5", "sha1", "sha256", "sha512"];

/// The directory the version's files are in, inside the layout.
pub(crate) fn version_dir(maven: &Path, version: &str) -> PathBuf {
    let mut dir = maven.to_path_buf();
    for part in GROUP.split('.') {
        dir.push(part);
    }
    dir.join(ARTIFACT).join(version)
}

/// Stages the layout from the libraries the platforms' manifests list.
/// Answers the files written and the `maven` record the merged manifest
/// carries, or nothing when the machine has no JDK (a skip, which a strict
/// run refuses).
pub(crate) fn stage(
    root: &Path,
    dist: &Path,
    version: &str,
    merged: &Value,
) -> io::Result<Option<(Vec<String>, Value)>> {
    if !(present("javac", "--version") && present("javadoc", "--version")) {
        crate::skip::skip("the Java package: no javac and javadoc on PATH");
        return Ok(None);
    }
    let build = dist.join(BUILD);
    if build.exists() {
        fs::remove_dir_all(&build)?;
    }
    let sources = build.join("sources");
    let classes = build.join("classes");
    let javadoc = build.join("javadoc");
    stage_sources(root, &sources, merged)?;
    let mut files = Vec::new();
    java_files(&sources, &mut files);
    files.sort();
    let args = argfile(&build, "main", &files).map_err(|()| io::Error::other("the argfile"))?;
    run(
        javac(&classes)
            .args(["--module-version", version])
            .arg(&args),
        "javac",
    )?;
    run(
        Command::new("javadoc")
            .args(["--release", FLOOR, "-encoding", "UTF-8", "-Xdoclint:all"])
            .args(["-Werror", "-quiet", "--date"])
            .arg(commit_time(root)?)
            .arg("-d")
            .arg(&javadoc)
            .arg(&args),
        "javadoc",
    )?;

    let platforms = platforms(dist, merged)?;
    let shared = shared_entries(root, &classes)?;
    let out = version_dir(&dist.join(MAVEN), version);
    if out.exists() {
        fs::remove_dir_all(&out)?;
    }
    fs::create_dir_all(&out)?;
    let base = format!("{ARTIFACT}-{version}");
    let mut written = Vec::new();
    let mut record = Vec::new();
    let mut put = |name: String, bytes: Vec<u8>| -> io::Result<()> {
        let path = out.join(&name);
        fs::write(&path, &bytes)?;
        record.push(json!({
            "file": rel(dist, &path),
            "bytes": bytes.len(),
            "sha256": hex(&Sha256::digest(&bytes)),
        }));
        for (extension, digest) in checksums(&bytes) {
            fs::write(out.join(format!("{name}.{extension}")), digest)?;
        }
        println!("{name}: {} bytes", bytes.len());
        written.push(rel(root, &path));
        Ok(())
    };
    put(format!("{base}.pom"), pom(root, version)?.into_bytes())?;
    put(
        format!("{base}.jar"),
        runtime_jar(version, &shared, &platforms.iter().collect::<Vec<_>>())?,
    )?;
    for one in &platforms {
        put(
            format!("{base}-{}.jar", one.0.name()),
            runtime_jar(version, &shared, &[one])?,
        )?;
    }
    put(
        format!("{base}-sources.jar"),
        jar(version, &legal(root, tree(&sources)?)?, Vec::new())?,
    )?;
    put(
        format!("{base}-javadoc.jar"),
        jar(version, &legal(root, tree(&javadoc)?)?, Vec::new())?,
    )?;
    let maven = json!({
        "coordinate": format!("{GROUP}:{ARTIFACT}:{version}"),
        "javac": javac_version()?,
        "platforms": platforms.iter().map(|one| one.0.name()).collect::<Vec<_>>(),
        "files": record,
    });
    Ok(Some((written, maven)))
}

/// Copies the binding's source roots into one tree and writes the
/// release's digest table into it.
fn stage_sources(root: &Path, into: &Path, merged: &Value) -> io::Result<()> {
    let package = root.join(PACKAGE);
    let mut seen = BTreeMap::new();
    for source in SOURCES {
        for (name, bytes) in tree(&package.join(source))? {
            if let Some(other) = seen.insert(name.clone(), source) {
                return Err(io::Error::other(format!(
                    "{name} is in both {other} and {source}"
                )));
            }
            let path = into.join(&name);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, bytes)?;
        }
    }
    let checked_in = fs::read_to_string(into.join(PREBUILT))?;
    fs::write(into.join(PREBUILT), prebuilt_table(&checked_in, merged)?)
}

/// The checked-in `Prebuilt.java` with the release's table in place of the
/// empty one.
pub(crate) fn prebuilt_table(checked_in: &str, merged: &Value) -> io::Result<String> {
    let (head, _) = checked_in.split_once(PREBUILT_TABLE).ok_or_else(|| {
        io::Error::other(format!("{PREBUILT} no longer declares `{PREBUILT_TABLE}`"))
    })?;
    let mut rows: Vec<String> = merged["platforms"]
        .as_object()
        .into_iter()
        .flatten()
        .map(|(name, platform)| {
            format!(
                "            Map.entry(\"{name}\", \"{}\")",
                platform["library"]["sha256"].as_str().unwrap_or_default()
            )
        })
        .collect();
    rows.sort();
    Ok(format!(
        "{head}{PREBUILT_TABLE} = Map.ofEntries(\n{});\n}}\n",
        rows.join(",\n")
    ))
}

/// Each platform's library, verified against its manifest and deflated
/// once, with its bill of materials.
fn platforms(dist: &Path, merged: &Value) -> io::Result<Vec<(Platform, Entry, Vec<u8>)>> {
    let mut found = Vec::new();
    for (name, manifest) in merged["platforms"].as_object().into_iter().flatten() {
        let platform = Platform::by_name(name)
            .ok_or_else(|| io::Error::other(format!("{name} is not a shipped platform")))?;
        let bytes = verified_library(dist, name, manifest)?;
        let bill = manifest["sboms"][0]["file"]
            .as_str()
            .ok_or_else(|| io::Error::other(format!("{name}'s manifest names no library bill")))?;
        found.push((platform, zip::pack(&bytes)?, fs::read(dist.join(bill))?));
    }
    Ok(found)
}

/// A platform's entries in a jar: its library and its bill.
fn native_entries(one: &(Platform, Entry, Vec<u8>)) -> Vec<(String, Entry)> {
    let (platform, library, bill) = one;
    let dir = format!("native/{}", platform.name());
    vec![
        (
            format!("{dir}/{}", platform.shared(LIBRARY_STEM)),
            library.clone(),
        ),
        (
            format!("{dir}/{}", crate::sbom::FILE),
            Entry::Plain(bill.clone()),
        ),
    ]
}

/// A jar that loads: the shared entries and the platforms' natives, read
/// back by its central directory before it is answered, so a jar whose
/// manifest is not first or whose natives are not exactly the platforms'
/// is never written.
fn runtime_jar(
    version: &str,
    shared: &BTreeMap<String, Vec<u8>>,
    platforms: &[&(Platform, Entry, Vec<u8>)],
) -> io::Result<Vec<u8>> {
    let natives: Vec<Vec<(String, Entry)>> =
        platforms.iter().map(|one| native_entries(one)).collect();
    let mut expected: Vec<String> = natives
        .iter()
        .flatten()
        .map(|(name, _)| name.clone())
        .collect();
    expected.sort();
    let bytes = jar(version, shared, natives)?;
    let names = zip::names(&bytes)?;
    let found: Vec<&String> = names
        .iter()
        .filter(|name| name.starts_with("native/"))
        .collect();
    if names.first().map(String::as_str) != Some(MANIFEST)
        || found != expected.iter().collect::<Vec<_>>()
    {
        return Err(io::Error::other(format!(
            "a jar read back as {names:?}, not the manifest first and the natives {expected:?}"
        )));
    }
    Ok(bytes)
}

/// The jar manifest's entry name.
const MANIFEST: &str = "META-INF/MANIFEST.MF";

/// What every runtime jar carries: the classes, the terms and the IDL.
fn shared_entries(root: &Path, classes: &Path) -> io::Result<BTreeMap<String, Vec<u8>>> {
    let mut entries = legal(root, tree(classes)?)?;
    entries.insert(
        "META-INF/teistro/api.json".to_owned(),
        fs::read(root.join("idl/api.json"))?,
    );
    Ok(entries)
}

/// `entries` with the licence and notice under `META-INF`.
fn legal(
    root: &Path,
    mut entries: BTreeMap<String, Vec<u8>>,
) -> io::Result<BTreeMap<String, Vec<u8>>> {
    for file in ["LICENSE", "NOTICE"] {
        entries.insert(format!("META-INF/{file}"), fs::read(root.join(file))?);
    }
    Ok(entries)
}

/// A jar: the manifest first, where `JarInputStream` looks for it, then
/// every other entry in name order. The natives' entries sort with the
/// rest.
pub(crate) fn jar(
    version: &str,
    entries: &BTreeMap<String, Vec<u8>>,
    natives: Vec<Vec<(String, Entry)>>,
) -> io::Result<Vec<u8>> {
    let mut sorted: BTreeMap<String, Entry> = entries
        .iter()
        .map(|(name, bytes)| (name.clone(), Entry::Plain(bytes.clone())))
        .collect();
    sorted.extend(natives.into_iter().flatten());
    let manifest = Entry::Plain(manifest(version));
    zip::write(
        std::iter::once((MANIFEST, &manifest))
            .chain(sorted.iter().map(|(name, entry)| (name.as_str(), entry))),
    )
}

/// The jar manifest: CRLF lines, each under 72 bytes, a blank line last.
/// No `Automatic-Module-Name` (the jar has a `module-info.class`) and no
/// `Enable-Native-Access` (read only from an executable jar's manifest).
pub(crate) fn manifest(version: &str) -> Vec<u8> {
    [
        "Manifest-Version: 1.0".to_owned(),
        "Created-By: teistro-xtask".to_owned(),
        format!("Implementation-Title: {ARTIFACT}"),
        format!("Implementation-Version: {version}"),
        "Implementation-Vendor: Teispace".to_owned(),
        String::new(),
        String::new(),
    ]
    .join("\r\n")
    .into_bytes()
}

/// Every file under a directory, keyed by its path with `/` separators.
fn tree(dir: &Path) -> io::Result<BTreeMap<String, Vec<u8>>> {
    fn walk(dir: &Path, base: &Path, into: &mut BTreeMap<String, Vec<u8>>) -> io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                walk(&path, base, into)?;
            } else {
                let name = path
                    .strip_prefix(base)
                    .map_err(io::Error::other)?
                    .components()
                    .map(|part| part.as_os_str().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("/");
                into.insert(name, fs::read(&path)?);
            }
        }
        Ok(())
    }
    let mut found = BTreeMap::new();
    walk(dir, dir, &mut found)?;
    Ok(found)
}

/// The four digests Maven reads beside a file, by extension: lower-case
/// hex, the digest alone, no newline.
pub(crate) fn checksums(bytes: &[u8]) -> [(&'static str, String); 4] {
    [
        (CHECKSUMS[0], hex(&Md5::digest(bytes))),
        (CHECKSUMS[1], hex(&Sha1::digest(bytes))),
        (CHECKSUMS[2], hex(&Sha256::digest(bytes))),
        (CHECKSUMS[3], hex(&Sha512::digest(bytes))),
    ]
}

/// The POM, with every element Central requires, the licence and the
/// repository read from the workspace's `[workspace.package]`.
pub(crate) fn pom(root: &Path, version: &str) -> io::Result<String> {
    let manifest: toml::Table = fs::read_to_string(root.join("Cargo.toml"))?
        .parse()
        .map_err(io::Error::other)?;
    let field = |key: &str| {
        manifest["workspace"]["package"][key]
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| io::Error::other(format!("[workspace.package] has no {key}")))
    };
    Ok(pom_text(version, &field("license")?, &field("repository")?))
}

/// The POM's text for a version, a licence and a repository URL.
pub(crate) fn pom_text(version: &str, licence: &str, repository: &str) -> String {
    let ssh = repository.replacen("https://github.com/", "ssh://git@github.com/", 1);
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://maven.apache.org/POM/4.0.0 https://maven.apache.org/xsd/maven-4.0.0.xsd">
  <modelVersion>4.0.0</modelVersion>
  <groupId>{GROUP}</groupId>
  <artifactId>{ARTIFACT}</artifactId>
  <version>{version}</version>
  <packaging>jar</packaging>
  <name>Teistro SDK for Java</name>
  <description>The Teistro SDK for Java: an ephemeris-agnostic astrology engine over the C ABI through the Foreign Function and Memory API, with the catalogue, the calendars, the time layer, the locale engine and the astronomy. Java {FLOOR} or later.</description>
  <url>{repository}</url>
  <licenses>
    <license>
      <name>{licence}</name>
      <url>https://www.apache.org/licenses/LICENSE-2.0.txt</url>
      <distribution>repo</distribution>
    </license>
  </licenses>
  <developers>
    <developer>
      <id>teispace</id>
      <name>Teispace</name>
      <email>support@teispace.com</email>
      <organization>Teispace</organization>
      <organizationUrl>https://teispace.com</organizationUrl>
    </developer>
  </developers>
  <scm>
    <connection>scm:git:{repository}.git</connection>
    <developerConnection>scm:git:{ssh}.git</developerConnection>
    <url>{repository}/tree/v{version}</url>
    <tag>v{version}</tag>
  </scm>
  <issueManagement>
    <system>GitHub</system>
    <url>{repository}/issues</url>
  </issueManagement>
</project>
"#
    )
}

/// What `javac --version` prints, which the release records beside the
/// classes it compiled.
fn javac_version() -> io::Result<String> {
    let out = Command::new("javac").arg("--version").output()?;
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

/// HEAD's commit time, which dates the javadoc so a commit stages the
/// same bytes twice. (`--date` stamps every page with it; `javadoc`
/// refuses it beside `-notimestamp`.)
fn commit_time(root: &Path) -> io::Result<String> {
    let out = Command::new("git")
        .args(["log", "-1", "--format=%cI"])
        .current_dir(root)
        .output()?;
    let text = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    if !out.status.success() || text.is_empty() {
        return Err(io::Error::other("git gave no commit time for HEAD"));
    }
    Ok(text)
}

/// Runs a JDK tool, its output shown only when it fails.
fn run(command: &mut Command, what: &str) -> io::Result<()> {
    let out = command.output()?;
    if out.status.success() {
        return Ok(());
    }
    Err(io::Error::other(format!(
        "{what} failed:\n{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn merged(names: &[&str]) -> Value {
        let platforms: serde_json::Map<String, Value> = names
            .iter()
            .map(|name| {
                (
                    (*name).to_owned(),
                    json!({ "library": { "sha256": format!("{name}-digest") } }),
                )
            })
            .collect();
        json!({ "platforms": platforms })
    }

    #[test]
    fn the_pom_holds_every_element_central_requires() {
        let pom = pom_text(
            "1.2.3",
            "Apache-2.0",
            "https://github.com/teispace/teistro-sdk",
        );
        for element in [
            "<groupId>com.teispace</groupId>",
            "<artifactId>teistro</artifactId>",
            "<version>1.2.3</version>",
            "<name>",
            "<description>",
            "<url>https://github.com/teispace/teistro-sdk</url>",
            "<license>",
            "<name>Apache-2.0</name>",
            "<developer>",
            "<email>support@teispace.com</email>",
            "<organization>",
            "<organizationUrl>",
            "<connection>scm:git:https://github.com/teispace/teistro-sdk.git</connection>",
            "<developerConnection>scm:git:ssh://git@github.com/teispace/teistro-sdk.git</developerConnection>",
            "<url>https://github.com/teispace/teistro-sdk/tree/v1.2.3</url>",
        ] {
            assert!(pom.contains(element), "{element}");
        }
        assert!(
            !pom.contains("<dependencies>"),
            "the jar depends on nothing"
        );
    }

    #[test]
    fn the_manifest_is_first_and_crlf_and_short() {
        let text = String::from_utf8(manifest("1.2.3")).unwrap();
        assert!(text.ends_with("\r\n\r\n"));
        for line in text.split("\r\n") {
            assert!(line.len() < 72, "{line}");
            assert!(!line.contains('\n'));
        }
        let entries = BTreeMap::from([("a.class".to_owned(), b"x".to_vec())]);
        let bytes = jar("1.2.3", &entries, Vec::new()).unwrap();
        assert_eq!(zip::names(&bytes).unwrap()[0], "META-INF/MANIFEST.MF");
    }

    #[test]
    fn a_default_jar_carries_every_native_and_a_classifier_one() {
        let classes = BTreeMap::from([("module-info.class".to_owned(), b"m".to_vec())]);
        let native = |name: &str| {
            let platform = Platform::by_name(name).unwrap();
            native_entries(&(platform, zip::pack(b"lib").unwrap(), b"bill".to_vec()))
        };
        let all = jar(
            "1.0.0",
            &classes,
            vec![native("linux-x64"), native("darwin-arm64")],
        )
        .unwrap();
        let one = jar("1.0.0", &classes, vec![native("darwin-arm64")]).unwrap();
        let natives = |bytes: &[u8]| -> Vec<String> {
            zip::names(bytes)
                .unwrap()
                .into_iter()
                .filter(|name| name.starts_with("native/"))
                .collect()
        };
        assert_eq!(
            natives(&all),
            [
                "native/darwin-arm64/libteistro_ffi.dylib",
                "native/darwin-arm64/sbom.cdx.json",
                "native/linux-x64/libteistro_ffi.so",
                "native/linux-x64/sbom.cdx.json",
            ]
        );
        assert_eq!(
            natives(&one),
            [
                "native/darwin-arm64/libteistro_ffi.dylib",
                "native/darwin-arm64/sbom.cdx.json"
            ]
        );
        assert_eq!(
            jar("1.0.0", &classes, vec![native("darwin-arm64")]).unwrap(),
            one,
            "the same inputs, the same bytes"
        );
    }

    #[test]
    fn the_checksums_are_mavens_form() {
        let [md5, sha1, ..] = checksums(b"abc");
        assert_eq!(md5, ("md5", "900150983cd24fb0d6963f7d28e17f72".to_owned()));
        assert_eq!(
            sha1,
            (
                "sha1",
                "a9993e364706816aba3e25717850c26c9cd0d89d".to_owned()
            )
        );
    }

    #[test]
    fn the_digest_table_lists_the_merged_platforms_sorted() {
        let checked_in = "package x;\n\nfinal class Prebuilt {\n    static final Map<String, String> DIGESTS = Map.of();\n}\n";
        let table = prebuilt_table(checked_in, &merged(&["linux-x64", "darwin-arm64"])).unwrap();
        let darwin = table.find("darwin-arm64").unwrap();
        let linux = table.find("linux-x64").unwrap();
        assert!(darwin < linux, "sorted:\n{table}");
        assert!(table.contains("Map.entry(\"linux-x64\", \"linux-x64-digest\")"));
        assert!(table.ends_with("));\n}\n"), "{table}");
        assert!(prebuilt_table("no table", &merged(&[])).is_err());
    }

    #[test]
    fn the_layout_is_mavens() {
        assert_eq!(
            version_dir(Path::new("m"), "1.2.3"),
            Path::new("m/com/teispace/teistro/1.2.3")
        );
    }
}
