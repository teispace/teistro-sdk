//! One version for the whole SDK, the gate that holds every manifest to
//! it, and the command that moves it.
//!
//! The version is declared once, in the workspace's `[workspace.package]`
//! table, and every crate takes it from there. Three files outside Cargo's
//! reach have to repeat it — the Node package's manifest, the Dart
//! package's, and the API description the bindings are generated from —
//! and a release where they disagree ships a library that refuses its own
//! generated types at load time (`refuseBuild`). `check-versions` is the
//! gate that catches that on the pull request instead, and
//! `cargo xtask version X` is how the version moves, so that nobody has to
//! know the list.
//!
//! Two rules ride along, because they are the same fact in another form:
//! the Node package lists a platform package for every platform the
//! release matrix builds, pinned to this exact version; and a repository
//! carrying the unreleased version marks both packages unpublishable, so
//! that publishing takes a deliberate bump and not a slip of the hand.

use std::fs;
use std::path::Path;

use regex::Regex;
use serde_json::{Map, Value};

use crate::platform::PLATFORMS;
use crate::{read, rel};

/// The version the repository carries until a release is cut. Nothing may
/// be published while it says this.
pub(crate) const UNRELEASED: &str = "0.0.0";

const WORKSPACE: &str = "Cargo.toml";
const NODE_MANIFEST: &str = "bindings/node/package.json";
const DART_MANIFEST: &str = "bindings/dart/pubspec.yaml";
const API: &str = "idl/api.json";
const PREBUILT: &str = "bindings/dart/lib/src/prebuilt.dart";
const CHANGELOG: &str = "CHANGELOG.md";

/// The SDK's version: what the workspace table declares.
///
/// # Panics
///
/// When the workspace manifest has no version, which would mean the
/// checkout is not this repository.
pub(crate) fn version(root: &Path) -> String {
    let text = read(&root.join(WORKSPACE));
    workspace_version(&text)
        .unwrap_or_else(|| panic!("{WORKSPACE} has no version in [workspace.package]"))
}

/// The version in a workspace manifest's `[workspace.package]` table.
fn workspace_version(text: &str) -> Option<String> {
    let table = text.split("[workspace.package]").nth(1)?;
    let line = table
        .lines()
        .take_while(|line| !line.starts_with('['))
        .find_map(|line| line.strip_prefix("version = "))?;
    Some(line.trim().trim_matches('"').to_string())
}

/// Whether a string is a version this project will release: three numbers
/// without leading zeros, and an optional pre-release of dot-separated
/// alphanumeric parts. Build metadata is not accepted, because npm and pub
/// treat it differently and the SDK would not be one version any more.
fn is_semver(text: &str) -> bool {
    let (core, pre) = text
        .split_once('-')
        .map_or((text, None), |(c, p)| (c, Some(p)));
    let numbers: Vec<&str> = core.split('.').collect();
    let numeric = |part: &str| {
        !part.is_empty()
            && part.chars().all(|c| c.is_ascii_digit())
            && (part == "0" || !part.starts_with('0'))
    };
    if numbers.len() != 3 || !numbers.iter().all(|part| numeric(part)) {
        return false;
    }
    match pre {
        None => true,
        Some(pre) => {
            !pre.is_empty()
                && pre.split('.').all(|part| {
                    !part.is_empty() && part.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
                })
        }
    }
}

// ── check-versions ─────────────────────────────────────────────────────────

/// Every manifest declares the workspace's version, the Node package lists
/// the platform packages the matrix builds, and an unreleased repository
/// cannot be published.
pub(crate) fn check(root: &Path) -> i32 {
    let wanted = version(root);
    let mut failures = Vec::new();
    if !is_semver(&wanted) {
        failures.push(format!(
            "{WORKSPACE} declares `{wanted}`, which is not a version this project releases (three numbers, optionally a pre-release)"
        ));
    }
    let released = wanted != UNRELEASED;

    let node = match parse_manifest(root, NODE_MANIFEST) {
        Ok(value) => Some(value),
        Err(failure) => {
            failures.push(failure);
            None
        }
    };
    if let Some(node) = &node {
        failures.extend(check_node(node, &wanted, released));
    }
    failures.extend(check_dart(root, &wanted, released));
    failures.extend(check_api(root, &wanted));
    failures.extend(check_changelog(root, &wanted, released));

    for failure in &failures {
        println!("FAIL  {failure}");
    }
    println!(
        "one version, {wanted}, across four manifests and {} platform packages: {} failure(s)",
        PLATFORMS.len(),
        failures.len()
    );
    i32::from(!failures.is_empty())
}

/// The Node manifest: the version, the platform packages, and whether it
/// may be published.
fn check_node(node: &Value, wanted: &str, released: bool) -> Vec<String> {
    let mut failures = Vec::new();
    if node["version"].as_str() != Some(wanted) {
        failures.push(format!(
            "{NODE_MANIFEST} declares version {}, the workspace declares {wanted}",
            node["version"]
        ));
    }
    let private = node["private"].as_bool().unwrap_or(false);
    if released && private {
        failures.push(format!(
            "{NODE_MANIFEST} is `private` at version {wanted}; a released package is published, so the field is removed by `cargo xtask version`"
        ));
    }
    if !released && !private {
        failures.push(format!(
            "{NODE_MANIFEST} is not `private` at the unreleased version; nothing may be published from {UNRELEASED}"
        ));
    }
    let declared = node["optionalDependencies"].as_object();
    let expected: Vec<(String, String)> = PLATFORMS
        .iter()
        .map(|p| (p.npm_package(), wanted.to_string()))
        .collect();
    match declared {
        None => failures.push(format!(
            "{NODE_MANIFEST} lists no optionalDependencies; the loader finds the addon in a platform package, so all {} are listed",
            PLATFORMS.len()
        )),
        Some(map) => {
            for (name, version) in &expected {
                match map.get(name).and_then(Value::as_str) {
                    Some(found) if found == version => {}
                    Some(found) => failures.push(format!(
                        "{NODE_MANIFEST} depends on {name} {found}, which is not this build's {version}"
                    )),
                    None => failures.push(format!(
                        "{NODE_MANIFEST} does not list {name}, which the release matrix builds"
                    )),
                }
            }
            for name in map.keys() {
                if !expected.iter().any(|(expected, _)| expected == name) {
                    failures.push(format!(
                        "{NODE_MANIFEST} lists {name}, which no platform in the table builds"
                    ));
                }
            }
        }
    }
    failures
}

/// The Dart manifest and the table its installer verifies downloads
/// against, which is written at release time and carries the version it
/// was written for.
fn check_dart(root: &Path, wanted: &str, released: bool) -> Vec<String> {
    let mut failures = Vec::new();
    let text = read(&root.join(DART_MANIFEST));
    let declared = text
        .lines()
        .find_map(|line| line.strip_prefix("version: "))
        .map(str::trim);
    if declared != Some(wanted) {
        failures.push(format!(
            "{DART_MANIFEST} declares version {}, the workspace declares {wanted}",
            declared.unwrap_or("nothing")
        ));
    }
    let unpublishable = text.lines().any(|line| line.trim() == "publish_to: none");
    if released && unpublishable {
        failures.push(format!(
            "{DART_MANIFEST} says `publish_to: none` at version {wanted}; a released package is published"
        ));
    }
    if !released && !unpublishable {
        failures.push(format!(
            "{DART_MANIFEST} does not say `publish_to: none` at the unreleased version"
        ));
    }
    let prebuilt = root.join(PREBUILT);
    if prebuilt.is_file() {
        let text = read(&prebuilt);
        let quoted = format!("'{wanted}'");
        if !text.contains(&quoted) {
            failures.push(format!(
                "{PREBUILT} does not name version {wanted}; run `cargo xtask version {wanted}`"
            ));
        }
    } else {
        failures.push(format!(
            "{PREBUILT} is missing; the Dart installer verifies a download against it"
        ));
    }
    failures
}

/// The API description carries the version the bindings were generated
/// from, and every binding refuses a library that is not that build.
fn check_api(root: &Path, wanted: &str) -> Vec<String> {
    match parse_manifest(root, API) {
        Err(failure) => vec![failure],
        Ok(api) => {
            if api["sdk_version"].as_str() == Some(wanted) {
                Vec::new()
            } else {
                vec![format!(
                    "{API} was generated from {}, the workspace declares {wanted}; run `cargo xtask gen ffi`",
                    api["sdk_version"]
                )]
            }
        }
    }
}

/// A released version has an entry in the changelog, because the entry is
/// where "does this move any number" is answered.
fn check_changelog(root: &Path, wanted: &str, released: bool) -> Vec<String> {
    if !released {
        return Vec::new();
    }
    let text = read(&root.join(CHANGELOG));
    let heading =
        Regex::new(&format!(r"(?m)^## {}\b", regex::escape(wanted))).expect("valid regex");
    if heading.is_match(&text) {
        Vec::new()
    } else {
        vec![format!(
            "{CHANGELOG} has no `## {wanted}` entry; every release answers whether it moves any number"
        )]
    }
}

fn parse_manifest(root: &Path, path: &str) -> Result<Value, String> {
    let full = root.join(path);
    let text = fs::read_to_string(&full)
        .map_err(|err| format!("cannot read {}: {err}", rel(root, &full)))?;
    serde_json::from_str(&text).map_err(|err| format!("{path} is not valid JSON: {err}"))
}

/// The changelog's entry for a version, which becomes the release's
/// notes. Printed rather than written, so that a caller decides where it
/// goes and a person can read it before a release exists.
pub(crate) fn changelog_entry(root: &Path, wanted: &str) -> i32 {
    let text = read(&root.join(CHANGELOG));
    let heading = Regex::new(&format!(r"^## {}\b", regex::escape(wanted))).expect("valid regex");
    let mut lines = text.lines().skip_while(|line| !heading.is_match(line));
    let Some(first) = lines.next() else {
        eprintln!("{CHANGELOG} has no `## {wanted}` entry");
        return 1;
    };
    println!("{first}");
    for line in lines.take_while(|line| !line.starts_with("## ")) {
        println!("{line}");
    }
    0
}

/// The tag a release is cut from names the version the repository
/// carries, which is the one thing a workflow cannot check for itself:
/// a tag pushed at the wrong commit would otherwise build and publish a
/// version nobody bumped.
pub(crate) fn check_tag(root: &Path, tag: &str) -> i32 {
    let wanted = version(root);
    let expected = format!("v{wanted}");
    if tag == expected {
        println!("ok    the tag {tag} is the version the repository carries");
        return 0;
    }
    println!(
        "FAIL  the tag is {tag}, the repository carries {wanted} (expected {expected}); move the tag or run `cargo xtask version`"
    );
    if wanted == UNRELEASED {
        println!("note  {UNRELEASED} is the unreleased version: nothing is published from it");
    }
    1
}

// ── version X ──────────────────────────────────────────────────────────────

/// Moves the whole repository to a version: the workspace table, both
/// package manifests, the platform packages they depend on, the
/// publishability of each, and the table the Dart installer verifies
/// against. What is generated from the version — the API description and
/// the catalogues the bindings carry — is regenerated afterwards, and the
/// gate says so if it was not.
pub(crate) fn set(root: &Path, wanted: &str) -> i32 {
    if !is_semver(wanted) {
        eprintln!(
            "`{wanted}` is not a version this project releases: three numbers without leading zeros, optionally a pre-release (`1.2.0`, `1.2.0-rc.1`)"
        );
        return 2;
    }
    let released = wanted != UNRELEASED;
    let mut written = Vec::new();

    let manifest = root.join(WORKSPACE);
    let text = read(&manifest);
    let Some(current) = workspace_version(&text) else {
        eprintln!("{WORKSPACE} has no version in [workspace.package]");
        return 1;
    };
    let updated = text.replacen(
        &format!("version = \"{current}\""),
        &format!("version = \"{wanted}\""),
        1,
    );
    write(&manifest, &updated, &mut written, root);

    set_node(root, wanted, released, &mut written);
    set_dart(root, wanted, released, &mut written);

    for path in &written {
        println!("wrote {path}");
    }
    println!(
        "the SDK is {wanted}; run `cargo xtask gen ffi` and `cargo check --workspace`, then `cargo xtask check-versions`"
    );
    0
}

/// The Node manifest: the version, the platform packages at that version,
/// and `private` while there is nothing to publish.
fn set_node(root: &Path, wanted: &str, released: bool, written: &mut Vec<String>) {
    let path = root.join(NODE_MANIFEST);
    let Ok(Value::Object(mut node)) = parse_manifest(root, NODE_MANIFEST) else {
        eprintln!("{NODE_MANIFEST} is not an object");
        return;
    };
    node.insert("version".to_string(), Value::String(wanted.to_string()));
    if released {
        node.shift_remove("private");
    } else {
        // Ordered where npm's own `init` puts it, after the description,
        // so a reader meets the field before the rest of the manifest.
        node.insert("private".to_string(), Value::Bool(true));
    }
    let mut platforms = Map::new();
    for platform in PLATFORMS {
        platforms.insert(platform.npm_package(), Value::String(wanted.to_string()));
    }
    node.insert("optionalDependencies".to_string(), Value::Object(platforms));
    let text = format!(
        "{}\n",
        serde_json::to_string_pretty(&Value::Object(node)).expect("a manifest serialises")
    );
    write(&path, &text, written, root);
}

/// The Dart manifest and the installer's table.
fn set_dart(root: &Path, wanted: &str, released: bool, written: &mut Vec<String>) {
    let path = root.join(DART_MANIFEST);
    let text = read(&path);
    // The field is dropped wherever it stands and written back beside the
    // version, so that moving to and from the unreleased version is the
    // same edit in both directions.
    let mut lines: Vec<String> = text
        .lines()
        .filter(|line| line.trim() != "publish_to: none")
        .map(|line| {
            if line.starts_with("version: ") {
                format!("version: {wanted}")
            } else {
                line.to_string()
            }
        })
        .collect();
    if !released {
        if let Some(at) = lines.iter().position(|line| line.starts_with("version: ")) {
            lines.insert(at + 1, "publish_to: none".to_string());
        }
    }
    write(&path, &format!("{}\n", lines.join("\n")), written, root);

    let prebuilt = root.join(PREBUILT);
    let table = read(&prebuilt);
    let line = Regex::new(r"(?m)^const String prebuiltVersion = '[^']*';$").expect("valid regex");
    let updated = line.replace(
        &table,
        format!("const String prebuiltVersion = '{wanted}';").as_str(),
    );
    write(&prebuilt, &updated, written, root);
}

fn write(path: &Path, text: &str, written: &mut Vec<String>, root: &Path) {
    if read_or_empty(path) == text {
        return;
    }
    match fs::write(path, text) {
        Ok(()) => written.push(rel(root, path)),
        Err(err) => eprintln!("cannot write {}: {err}", rel(root, path)),
    }
}

fn read_or_empty(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{is_semver, workspace_version};

    #[test]
    fn the_workspace_version_is_read_from_its_own_table() {
        let text = "[workspace]\nmembers = []\n\n[workspace.package]\nversion = \"1.2.3\"\nedition = \"2024\"\n\n[other]\nversion = \"9.9.9\"\n";
        assert_eq!(workspace_version(text).as_deref(), Some("1.2.3"));
    }

    #[test]
    fn a_manifest_without_the_table_has_no_version() {
        assert_eq!(workspace_version("[package]\nversion = \"1.0.0\"\n"), None);
    }

    #[test]
    fn versions_this_project_releases() {
        for good in [
            "0.0.0",
            "1.2.3",
            "0.1.0",
            "1.0.0-rc.1",
            "2.0.0-alpha",
            "10.20.30",
        ] {
            assert!(is_semver(good), "{good} is a version");
        }
        for bad in [
            "1.2",
            "1.2.3.4",
            "v1.2.3",
            "01.2.3",
            "1.2.3+build",
            "1.2.3-",
            "",
            "1.2.x",
        ] {
            assert!(!is_semver(bad), "{bad} is not a version");
        }
    }
}
