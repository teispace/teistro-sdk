//! Every published crate builds from its own package.
//!
//! A consumer compiles the SDK from what `cargo package` ships: a registry,
//! a vendored source, or a bundle in which each crate is trimmed to its
//! `cargo package --list`. A build script, an `include_str!` or an
//! `env!("CARGO_MANIFEST_DIR")` path that reaches out of its crate into the
//! repository (`../../i18n`) builds in a checkout and nowhere else, and no
//! other gate sees it, because every other gate builds the checkout.
//!
//! `cargo xtask check-packages` packages every crate a consumer compiles —
//! the workspace crates `teistro` and `teistro-ffi` reach through normal
//! and build dependencies, read from `cargo metadata` rather than listed —
//! unpacks each `.crate` into a directory outside the repository, and
//! builds them there as a workspace of their own, each sibling resolved by
//! `[patch.crates-io]` to its unpacked package and nothing to the
//! checkout. A path that climbs out of an unpacked crate lands in a
//! directory the repository is not in.
//!
//! **Why not `cargo package`'s own verification**, which unpacks and
//! builds each package with its siblings from a local-registry overlay:
//! the overlay takes only a package that may be published
//! (`publish != false`), and every crate here says `publish = false`
//! until the first release (`docs/06-cicd/03-release-process.md`), so
//! the overlay is empty and the second crate's lockfile fails to resolve
//! the first. The gate packages with `--no-verify --exclude-lockfile`,
//! which needs no registry, and builds the packages itself, from the
//! repository's `Cargo.lock` so the versions are the ones every other gate
//! builds.
//!
//! Packaging also needs every internal dependency to carry a version, the
//! one a registry resolves once the `path` is dropped: each is the
//! workspace's own, exactly, which [`check_requirements`] holds for
//! `check-versions` and [`set_requirements`] moves for `cargo xtask
//! version`.
//!
//! Nothing here publishes: `cargo package` writes `.crate` files into the
//! temporary directory, which is removed afterwards with the build.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

use crate::binding::{cargo, step};

/// What a consumer compiles: the Rust façade and the C library. Every
/// crate they reach is published with them.
pub(crate) const ROOTS: [&str; 2] = ["teistro", "teistro-ffi"];

/// Where the crates that may carry an internal dependency live.
const CRATES: &str = "crates";

/// The dependency tables a package keeps. `dev-dependencies` are left
/// out: `cargo package` drops a development dependency with no version,
/// so the test-only crates need never be published.
const KEPT_TABLES: [&str; 2] = ["dependencies", "build-dependencies"];

// ── check-packages ─────────────────────────────────────────────────────────

/// Packages every published crate, builds them from their packages alone
/// in a directory outside the repository, and reports each `.crate`'s
/// size.
pub(crate) fn check(root: &Path) -> i32 {
    let published = match published(root) {
        Ok(published) => published,
        Err(failure) => {
            println!("FAIL  {failure}");
            return 1;
        }
    };
    let scratch = std::env::temp_dir().join(format!("teistro-packages-{}", std::process::id()));
    if let Some(failure) = outside(root, &scratch) {
        println!("FAIL  {failure}");
        return 1;
    }
    let _ = fs::remove_dir_all(&scratch);
    println!(
        "packaging {} crates, each built from its package in {}",
        published.len(),
        scratch.display()
    );
    let failures = packaged_and_built(root, &scratch, &published);
    let _ = fs::remove_dir_all(&scratch);
    println!(
        "{} published crates from their packages: {failures} failure(s)",
        published.len()
    );
    i32::from(failures != 0)
}

/// The failures of packaging `published` into `scratch`, unpacking each
/// package and building them all from there.
fn packaged_and_built(root: &Path, scratch: &Path, published: &BTreeSet<String>) -> usize {
    let target = scratch.join("target");
    let mut package = Command::new(cargo());
    package
        .current_dir(root)
        .args([
            "package",
            "--no-verify",
            "--exclude-lockfile",
            "--allow-dirty",
            "--target-dir",
        ])
        .arg(&target);
    for name in published {
        package.args(["-p", name]);
    }
    if step(&mut package, "", "the published crates do not package").is_err() {
        return 1;
    }
    let crates = match sizes(&target.join("package"), published) {
        Ok(crates) => crates,
        Err(failures) => return failures,
    };
    let sources = scratch.join("src");
    let members = match unpack(&crates, &sources) {
        Ok(members) => members,
        Err(failure) => {
            println!("FAIL  {failure}");
            return 1;
        }
    };
    let lock = fs::copy(root.join("Cargo.lock"), sources.join("Cargo.lock"));
    let manifest = fs::write(sources.join("Cargo.toml"), workspace(&members));
    if let Err(why) = lock.map(drop).and(manifest) {
        println!("FAIL  the packages' workspace cannot be written: {why}");
        return 1;
    }
    let mut build = Command::new(cargo());
    build
        .current_dir(&sources)
        .args(["build", "--workspace", "--target-dir"])
        .arg(&target)
        // The queue and CI may point every build at the checkout's target;
        // `--target-dir` already wins, and nothing else of the checkout's
        // configuration should reach a build that stands for a consumer's.
        .env_remove("CARGO_TARGET_DIR");
    usize::from(
        step(
            &mut build,
            "every published crate builds from its own package",
            "a published crate does not build from its own package: a build script, `include_str!` or `CARGO_MANIFEST_DIR` path reaches outside its crate",
        )
        .is_err(),
    )
}

/// Each `.crate` unpacked under `into`, as the directory names of the
/// unpacked packages.
fn unpack(crates: &[PathBuf], into: &Path) -> Result<Vec<String>, String> {
    fs::create_dir_all(into).map_err(|why| format!("{}: {why}", into.display()))?;
    let mut members = Vec::new();
    for file in crates {
        let reader = fs::File::open(file).map_err(|why| format!("{}: {why}", file.display()))?;
        tar::Archive::new(flate2::read::GzDecoder::new(reader))
            .unpack(into)
            .map_err(|why| format!("{} does not unpack: {why}", file.display()))?;
        let stem = file
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .ok_or_else(|| format!("{} has no name", file.display()))?;
        members.push(stem);
    }
    Ok(members)
}

/// The manifest of the workspace the unpacked packages build in: each a
/// member, and each sibling a package asks for by version taken from its
/// unpacked directory rather than from crates.io.
fn workspace(members: &[String]) -> String {
    use std::fmt::Write as _;

    let mut text = String::from(
        "# Written by `cargo xtask check-packages`: the published crates,\n\
         # each unpacked from its .crate, built as a consumer builds them.\n\
         [workspace]\nresolver = \"3\"\nmembers = [\n",
    );
    for member in members {
        let _ = writeln!(text, "    {member:?},");
    }
    text.push_str("]\n\n[patch.crates-io]\n");
    for member in members {
        let name = package_name(member);
        let _ = writeln!(text, "{name} = {{ path = {member:?} }}");
    }
    text
}

/// A package's name from its unpacked directory, `teistro-core-1.0.0-rc.1`
/// being `teistro-core`: the name ends where a part begins with a digit.
fn package_name(directory: &str) -> &str {
    directory
        .match_indices('-')
        .find(|(at, _)| directory[at + 1..].starts_with(|c: char| c.is_ascii_digit()))
        .map_or(directory, |(at, _)| &directory[..at])
}

/// `None` when `target` is outside the repository, else why it may not
/// be used: inside it, a path climbing out of an unpacked crate could
/// still find the repository's files and pass.
fn outside(root: &Path, target: &Path) -> Option<String> {
    let root = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let parent = target.parent().map(Path::to_path_buf).unwrap_or_default();
    let parent = fs::canonicalize(&parent).unwrap_or(parent);
    parent.starts_with(&root).then(|| {
        format!(
            "the temporary directory {} is inside the repository {}; set TMPDIR elsewhere",
            target.display(),
            root.display()
        )
    })
}

/// Each published crate's `.crate`, its compressed size printed, or the
/// number of published crates with no file.
fn sizes(dir: &Path, published: &BTreeSet<String>) -> Result<Vec<PathBuf>, usize> {
    let mut found: BTreeMap<String, (PathBuf, u64)> = BTreeMap::new();
    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path
            .extension()
            .is_some_and(|extension| extension == "crate")
        {
            let size = entry.metadata().map_or(0, |metadata| metadata.len());
            let stem = path
                .file_stem()
                .map(|stem| stem.to_string_lossy().into_owned())
                .unwrap_or_default();
            found.insert(stem, (path, size));
        }
    }
    let mut crates = Vec::new();
    let mut failures = 0;
    for name in published {
        if let Some((stem, (path, size))) =
            found.iter().find(|(stem, _)| package_name(stem) == name)
        {
            println!("      {stem}.crate {:.1} KiB", kib(*size));
            crates.push(path.clone());
        } else {
            println!("FAIL  {name} has no .crate in {}", dir.display());
            failures += 1;
        }
    }
    if failures == 0 {
        Ok(crates)
    } else {
        Err(failures)
    }
}

/// Bytes as KiB, for a report line.
#[expect(clippy::cast_precision_loss, reason = "a size printed to a tenth")]
fn kib(bytes: u64) -> f64 {
    bytes as f64 / 1024.0
}

/// The workspace crates [`ROOTS`] reach through normal and build
/// dependencies, the roots among them, as `cargo metadata` describes the
/// workspace.
fn published(root: &Path) -> Result<BTreeSet<String>, String> {
    let output = Command::new(cargo())
        .current_dir(root)
        .args(["metadata", "--format-version", "1", "--no-deps", "--quiet"])
        .output()
        .map_err(|why| format!("`cargo metadata` did not start: {why}"))?;
    if !output.status.success() {
        return Err(format!(
            "`cargo metadata` failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let metadata: Value = serde_json::from_slice(&output.stdout)
        .map_err(|why| format!("`cargo metadata` is not JSON: {why}"))?;
    let edges = internal_edges(&metadata);
    closure(&edges, &ROOTS)
}

/// Each workspace package's internal dependencies a package keeps (normal
/// and build, never development), by name.
fn internal_edges(metadata: &Value) -> BTreeMap<String, BTreeSet<String>> {
    let packages = metadata["packages"].as_array().cloned().unwrap_or_default();
    packages
        .iter()
        .filter_map(|package| {
            let name = package["name"].as_str()?.to_owned();
            let deps = package["dependencies"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|dep| dep["path"].is_string() && dep["kind"].as_str() != Some("dev"))
                .filter_map(|dep| dep["name"].as_str().map(str::to_owned))
                .collect();
            Some((name, deps))
        })
        .collect()
}

/// Everything `roots` reach in `edges`, the roots included; a root or a
/// dependency the workspace does not have is an error.
fn closure(
    edges: &BTreeMap<String, BTreeSet<String>>,
    roots: &[&str],
) -> Result<BTreeSet<String>, String> {
    let mut seen = BTreeSet::new();
    let mut stack: Vec<String> = roots.iter().map(|root| (*root).to_owned()).collect();
    while let Some(name) = stack.pop() {
        let deps = edges
            .get(&name)
            .ok_or_else(|| format!("the workspace has no package `{name}`"))?;
        if seen.insert(name) {
            stack.extend(deps.iter().cloned());
        }
    }
    Ok(seen)
}

// ── the requirement each internal dependency carries ───────────────────────

/// The requirement every internal dependency carries at `version`: the
/// SDK is one version, so a crate takes its siblings at exactly theirs.
fn requirement(version: &str) -> String {
    format!("={version}")
}

/// Every crate manifest under `crates/`, sorted.
fn manifests(root: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = fs::read_dir(root.join(CRATES))
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path().join("Cargo.toml"))
        .filter(|path| path.is_file())
        .collect();
    found.sort();
    found
}

/// Each internal dependency a package keeps whose requirement is not
/// exactly `version`: `cargo package` refuses one with none, and one with
/// another would resolve a sibling of another release.
pub(crate) fn check_requirements(root: &Path, version: &str) -> Vec<String> {
    let wanted = requirement(version);
    let mut failures = Vec::new();
    for path in manifests(root) {
        let relative = crate::rel(root, &path);
        let manifest: toml::Table = match crate::read(&path).parse() {
            Ok(manifest) => manifest,
            Err(why) => {
                failures.push(format!("{relative} does not parse: {why}"));
                continue;
            }
        };
        for (table, name, dep) in kept_dependencies(&manifest) {
            if dep.get("path").is_none() {
                continue;
            }
            match dep.get("version").and_then(toml::Value::as_str) {
                Some(found) if found == wanted => {}
                Some(found) => failures.push(format!(
                    "{relative} [{table}] takes {name} at `{found}`, not the workspace's `{wanted}`"
                )),
                None => failures.push(format!(
                    "{relative} [{table}] takes {name} by path alone; a package needs `version = \"{wanted}\"` beside the path"
                )),
            }
        }
    }
    failures
}

/// Every dependency in a table a package keeps, target-specific tables
/// included, with the table's name.
fn kept_dependencies(manifest: &toml::Table) -> Vec<(String, &str, &toml::Table)> {
    let mut found = Vec::new();
    for table in KEPT_TABLES {
        take(&mut found, table, manifest.get(table));
    }
    let targets = manifest.get("target").and_then(toml::Value::as_table);
    for (target, tables) in targets.into_iter().flatten() {
        for table in KEPT_TABLES {
            take(
                &mut found,
                &format!("target.{target}.{table}"),
                tables.get(table),
            );
        }
    }
    found
}

/// Each dependency of one table, labelled with the table's name.
fn take<'a>(
    found: &mut Vec<(String, &'a str, &'a toml::Table)>,
    label: &str,
    table: Option<&'a toml::Value>,
) {
    for (name, dep) in table.and_then(toml::Value::as_table).into_iter().flatten() {
        if let Some(dep) = dep.as_table() {
            found.push((label.to_owned(), name.as_str(), dep));
        }
    }
}

/// Moves every internal requirement from `current` to `wanted`, for
/// `cargo xtask version`, appending each manifest it wrote.
pub(crate) fn set_requirements(root: &Path, current: &str, wanted: &str) -> Vec<PathBuf> {
    let from = format!("version = \"{}\"", requirement(current));
    let to = format!("version = \"{}\"", requirement(wanted));
    let mut written = Vec::new();
    for path in manifests(root) {
        let text = crate::read(&path);
        if text.contains(&from) {
            fs::write(&path, text.replace(&from, &to))
                .unwrap_or_else(|why| panic!("cannot write {}: {why}", path.display()));
            written.push(path);
        }
    }
    written
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edges(pairs: &[(&str, &[&str])]) -> BTreeMap<String, BTreeSet<String>> {
        pairs
            .iter()
            .map(|(name, deps)| {
                (
                    (*name).to_owned(),
                    deps.iter().map(|dep| (*dep).to_owned()).collect(),
                )
            })
            .collect()
    }

    #[test]
    fn the_published_set_is_what_the_roots_reach() {
        let graph = edges(&[
            ("teistro", &["teistro-core", "teistro-intl"]),
            ("teistro-ffi", &["teistro"]),
            ("teistro-intl", &["teistro-core"]),
            ("teistro-core", &[]),
            ("teistro-mcp", &["teistro"]),
        ]);
        let set = closure(&graph, &ROOTS).unwrap();
        let names: Vec<&str> = set.iter().map(String::as_str).collect();
        assert_eq!(
            names,
            ["teistro", "teistro-core", "teistro-ffi", "teistro-intl"]
        );
    }

    #[test]
    fn a_dependency_the_workspace_lacks_is_an_error() {
        let graph = edges(&[("teistro", &["teistro-gone"]), ("teistro-ffi", &[])]);
        assert!(
            closure(&graph, &ROOTS)
                .unwrap_err()
                .contains("teistro-gone")
        );
    }

    #[test]
    fn development_dependencies_are_not_followed() {
        let metadata = serde_json::json!({"packages": [
            {"name": "a", "dependencies": [
                {"name": "b", "path": "/b", "kind": null},
                {"name": "c", "path": "/c", "kind": "build"},
                {"name": "d", "path": "/d", "kind": "dev"},
                {"name": "serde", "kind": null}
            ]}
        ]});
        let found = internal_edges(&metadata);
        let deps: Vec<&str> = found["a"].iter().map(String::as_str).collect();
        assert_eq!(deps, ["b", "c"]);
    }

    #[test]
    fn a_requirement_is_checked_in_every_kept_table() {
        let manifest: toml::Table = r#"
            [dependencies]
            a = { path = "../a", version = "=1.0.0" }
            b = { path = "../b" }
            serde = "1"
            [build-dependencies]
            c = { path = "../c", version = "1.0.0" }
            [dev-dependencies]
            d = { path = "../d" }
            [target.'cfg(unix)'.dependencies]
            e = { path = "../e" }
        "#
        .parse()
        .unwrap();
        let names: Vec<&str> = kept_dependencies(&manifest)
            .into_iter()
            .filter(|(_, _, dep)| dep.get("path").is_some())
            .map(|(_, name, _)| name)
            .collect();
        assert_eq!(names, ["a", "b", "c", "e"]);
    }

    #[test]
    fn a_package_is_named_by_its_directory() {
        assert_eq!(package_name("teistro-core-1.0.0-rc.1"), "teistro-core");
        assert_eq!(package_name("teistro-1.0.0"), "teistro");
        assert_eq!(
            package_name("teistro-port-ephemeris-2.3.4"),
            "teistro-port-ephemeris"
        );
    }

    #[test]
    fn the_workspace_patches_every_member_to_its_package() {
        let text = workspace(&[String::from("teistro-core-1.0.0")]);
        let manifest: toml::Table = text.parse().unwrap();
        assert_eq!(
            manifest["workspace"]["members"][0].as_str(),
            Some("teistro-core-1.0.0")
        );
        assert_eq!(
            manifest["patch"]["crates-io"]["teistro-core"]["path"].as_str(),
            Some("teistro-core-1.0.0")
        );
    }

    #[test]
    fn a_target_inside_the_repository_is_refused() {
        let root = std::env::temp_dir();
        assert!(outside(&root, &root.join("pkgs")).is_some());
        assert!(outside(&root.join("repo"), &root.join("pkgs")).is_none());
    }
}
