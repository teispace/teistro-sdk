//! What a shipped file is made of, said twice: inside the file, and beside
//! it.
//!
//! **Inside.** Every shared library and addon is built through
//! `cargo auditable`, which links the crates that went into it into a
//! `.dep-v0` section, so a scanner holding only the file can still say
//! which versions of which crates it carries. Nothing in a build that
//! skipped the wrapper says so, so [`embedded`] reads the section's name
//! back out of the file: the name sits in the section table of an ELF,
//! Mach-O or PE file alike, and a file built without the wrapper does not
//! carry it.
//!
//! **Beside.** A `CycloneDX` bill of materials per artefact, written from
//! `cargo tree` for the artefact's own package and target and from the
//! lock file's checksums ([`bill`]). It lists every crate reachable from
//! the package through normal dependencies, which is what is compiled
//! into the file: a build script's and a test's dependencies run on the
//! build machine and ship nothing. `cargo tree` rather than
//! `cargo metadata`, because the second resolves features for the whole
//! workspace at once, and so reaches crates another member's features
//! switch on (the library's would have listed `clap`, which it does not
//! compile). It carries no timestamp and no serial number, and every list
//! is sorted, so two builds of one commit write the same bytes, as the
//! archives beside it do.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

use serde_json::{Value, json};

use crate::binding::cargo;

/// The section `cargo auditable` links its dependency list into.
const SECTION: &[u8] = b".dep-v0";

/// The pinned `cargo-auditable`, which the workflows install from the same
/// file a failing local build names.
pub(crate) const AUDITABLE_VERSION: &str = "xtask/cargo-auditable.version";

/// The file name every package carries its bill of materials under.
pub(crate) const FILE: &str = "sbom.cdx.json";

/// The `CycloneDX` version the bills are written in.
const SPEC_VERSION: &str = "1.5";

/// Refuses a file that carries no `cargo auditable` dependency list.
pub(crate) fn embedded(files: &[&Path]) -> Result<(), String> {
    for file in files {
        let bytes = fs::read(file).map_err(|err| format!("{}: {err}", file.display()))?;
        if !bytes.windows(SECTION.len()).any(|window| window == SECTION) {
            return Err(format!(
                "{} carries no `.dep-v0` dependency list: it was not built through `cargo auditable` \
                 (`cargo install --locked cargo-auditable@{}`)",
                file.display(),
                auditable_version(file)
            ));
        }
        println!("audit  {} lists its crates", file.display());
    }
    Ok(())
}

/// The pinned version, for a message; the file's own path when it cannot
/// be read, which is where the version would have been.
fn auditable_version(near: &Path) -> String {
    near.ancestors()
        .map(|dir| dir.join(AUDITABLE_VERSION))
        .find_map(|path| fs::read_to_string(path).ok())
        .map_or_else(
            || AUDITABLE_VERSION.to_string(),
            |text| text.trim().to_string(),
        )
}

/// Writes the bill of materials for `package` built for `triple` to
/// `path`.
pub(crate) fn write(root: &Path, triple: &str, package: &str, path: &Path) -> Result<(), String> {
    let output = Command::new(cargo())
        .args([
            "tree",
            "--quiet",
            "--locked",
            "--package",
            package,
            "--edges",
            "normal",
            "--target",
            triple,
            "--prefix",
            "depth",
            "--format",
            "{p}|{l}",
        ])
        .current_dir(root)
        .output()
        .map_err(|err| format!("cargo tree did not run: {err}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo tree failed for {package} on {triple}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let lock =
        fs::read_to_string(root.join("Cargo.lock")).map_err(|err| format!("Cargo.lock: {err}"))?;
    let tree = parse(&String::from_utf8_lossy(&output.stdout))?;
    let text = serde_json::to_string_pretty(&bill(&tree, &checksums(&lock))?)
        .map_err(|err| err.to_string())?;
    fs::write(path, format!("{text}\n")).map_err(|err| format!("{}: {err}", path.display()))
}

/// One crate of a tree: its licence, and the crates it depends on, each
/// by `name@version`.
#[derive(Debug, Default, PartialEq)]
struct Crate {
    name: String,
    version: String,
    licence: Option<String>,
    deps: BTreeSet<String>,
}

/// A dependency tree, every crate by `name@version`, and the one it is
/// the tree of.
#[derive(Debug)]
struct Tree {
    subject: String,
    crates: BTreeMap<String, Crate>,
}

/// `cargo tree --prefix depth --format '{p}|{l}'`'s lines: each a depth,
/// then `name vVERSION`, a source or `(proc-macro)` in parentheses, `|`,
/// the licence, and ` (*)` where a crate already shown is shown again. A
/// crate's parent is the nearest line above it one level shallower.
fn parse(text: &str) -> Result<Tree, String> {
    let mut crates: BTreeMap<String, Crate> = BTreeMap::new();
    let mut path: Vec<String> = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let digits = line.chars().take_while(char::is_ascii_digit).count();
        let depth: usize = line[..digits]
            .parse()
            .map_err(|_| format!("a tree line without a depth: {line}"))?;
        let rest = line[digits..].trim_end_matches(" (*)");
        let (package, licence) = rest.split_once('|').unwrap_or((rest, ""));
        let mut words = package.split_whitespace();
        let (Some(name), Some(version)) = (words.next(), words.next()) else {
            return Err(format!("a tree line without a crate: {line}"));
        };
        let version = version.trim_start_matches('v');
        let key = format!("{name}@{version}");
        path.truncate(depth);
        if path.len() != depth {
            return Err(format!("a tree line deeper than its parent: {line}"));
        }
        if let Some(parent) = path.last() {
            crates
                .entry(parent.clone())
                .or_default()
                .deps
                .insert(key.clone());
        }
        let entry = crates.entry(key.clone()).or_default();
        entry.name = name.to_string();
        entry.version = version.to_string();
        let licence = licence.trim();
        if !licence.is_empty() {
            entry.licence = Some(licence.to_string());
        }
        path.push(key);
    }
    let subject = text
        .lines()
        .next()
        .and_then(|line| {
            let line = line.trim_start_matches('0');
            let mut words = line.split_whitespace();
            Some(format!(
                "{}@{}",
                words.next()?,
                words.next()?.trim_start_matches('v')
            ))
        })
        .ok_or("cargo tree printed nothing")?;
    Ok(Tree { subject, crates })
}

/// Each registry crate's checksum in a lock file, by `name@version`.
fn checksums(lock: &str) -> BTreeMap<String, String> {
    let mut found = BTreeMap::new();
    for block in lock.split("[[package]]") {
        let field = |key: &str| {
            block.lines().find_map(|line| {
                let (name, value) = line.split_once(" = ")?;
                (name.trim() == key).then(|| value.trim().trim_matches('"').to_string())
            })
        };
        if let (Some(name), Some(version), Some(checksum)) =
            (field("name"), field("version"), field("checksum"))
        {
            found.insert(format!("{name}@{version}"), checksum);
        }
    }
    found
}

/// The bill of materials for a tree: every crate in it a component, the
/// tree's own crate the bill's subject, and every edge a dependency.
fn bill(tree: &Tree, checksums: &BTreeMap<String, String>) -> Result<Value, String> {
    let component = |key: &str| -> Result<Value, String> {
        let one = tree
            .crates
            .get(key)
            .ok_or_else(|| format!("{key} is not in the tree"))?;
        let mut entry = json!({
            "type": "library",
            "bom-ref": key,
            "name": one.name,
            "version": one.version,
            "purl": format!("pkg:cargo/{}@{}", one.name, one.version),
        });
        if let Some(licence) = &one.licence {
            entry["licenses"] = json!([{ "expression": licence }]);
        }
        if let Some(checksum) = checksums.get(key) {
            entry["hashes"] = json!([{ "alg": "SHA-256", "content": checksum }]);
        }
        Ok(entry)
    };
    let components = tree
        .crates
        .keys()
        .filter(|key| **key != tree.subject)
        .map(|key| component(key))
        .collect::<Result<Vec<_>, _>>()?;
    let dependencies: Vec<Value> = tree
        .crates
        .iter()
        .map(|(key, one)| json!({ "ref": key, "dependsOn": one.deps }))
        .collect();
    Ok(json!({
        "bomFormat": "CycloneDX",
        "specVersion": SPEC_VERSION,
        "version": 1,
        "metadata": { "component": component(&tree.subject)? },
        "components": components,
        "dependencies": dependencies,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What `cargo tree` prints: a path crate, a proc-macro, a crate shown
    /// twice, and one with no licence.
    const TREE: &str = "0teistro-ffi v0.1.0 (/repo/crates/ffi)|Apache-2.0
1serde v1.0.0|MIT OR Apache-2.0
2serde_derive v1.0.0 (proc-macro)|MIT OR Apache-2.0
3itoa v1.0.0|MIT
1itoa v1.0.0|MIT (*)
1quiet v0.1.0|
";

    #[test]
    fn a_tree_is_read_with_its_edges() {
        let tree = parse(TREE).unwrap();
        assert_eq!(tree.subject, "teistro-ffi@0.1.0");
        let keys: Vec<&str> = tree.crates.keys().map(String::as_str).collect();
        assert_eq!(
            keys,
            [
                "itoa@1.0.0",
                "quiet@0.1.0",
                "serde@1.0.0",
                "serde_derive@1.0.0",
                "teistro-ffi@0.1.0"
            ]
        );
        let deps: Vec<&str> = tree.crates["teistro-ffi@0.1.0"]
            .deps
            .iter()
            .map(String::as_str)
            .collect();
        // A crate shown again is still an edge of its second parent.
        assert_eq!(deps, ["itoa@1.0.0", "quiet@0.1.0", "serde@1.0.0"]);
        assert_eq!(tree.crates["quiet@0.1.0"].licence, None);
    }

    #[test]
    fn a_bill_names_every_crate_once_and_its_subject_apart() {
        let checksums = BTreeMap::from([("serde@1.0.0".to_string(), "ab".to_string())]);
        let bill = bill(&parse(TREE).unwrap(), &checksums).unwrap();
        let names: Vec<&str> = bill["components"]
            .as_array()
            .unwrap()
            .iter()
            .map(|one| one["name"].as_str().unwrap())
            .collect();
        assert_eq!(names, ["itoa", "quiet", "serde", "serde_derive"]);
        assert_eq!(bill["metadata"]["component"]["name"], "teistro-ffi");
        assert_eq!(bill["components"][2]["hashes"][0]["content"], "ab");
        assert_eq!(bill["components"][2]["purl"], "pkg:cargo/serde@1.0.0");
        assert!(bill["components"][1].get("licenses").is_none());
        assert_eq!(bill["dependencies"][2]["ref"], "serde@1.0.0");
        assert_eq!(
            bill["dependencies"][2]["dependsOn"],
            json!(["serde_derive@1.0.0"])
        );
    }

    #[test]
    fn a_lock_files_checksums_are_read_by_name_and_version() {
        let lock = "[[package]]\nname = \"serde\"\nversion = \"1.0.0\"\nsource = \"registry+x\"\nchecksum = \"ab\"\n\n[[package]]\nname = \"teistro\"\nversion = \"0.1.0\"\n";
        assert_eq!(
            checksums(lock),
            BTreeMap::from([("serde@1.0.0".to_string(), "ab".to_string())])
        );
    }

    /// The library's own bill, from this workspace: what it compiles in for
    /// its target, each registry crate with the lock file's checksum.
    #[test]
    fn the_librarys_bill_is_its_own() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let path =
            std::env::temp_dir().join(format!("teistro-ffi-{}.cdx.json", std::process::id()));
        write(root, "x86_64-unknown-linux-gnu", "teistro-ffi", &path).unwrap();
        let bill: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        fs::remove_file(&path).unwrap();
        let components = bill["components"].as_array().unwrap();
        let named = |name: &str| components.iter().any(|one| one["name"] == name);
        assert!(named("teistro") && named("serde"));
        assert!(!named("teistro-ffi"), "the subject is the bill's own");
        assert!(
            !components
                .iter()
                .any(|one| one["name"].as_str().unwrap().starts_with("windows")),
            "a Linux file carries no Windows crate"
        );
        assert!(
            !named("clap"),
            "another member's features switch on nothing here"
        );
        for one in components
            .iter()
            .filter(|one| !one["name"].as_str().unwrap().starts_with("teistro"))
        {
            assert!(
                one["hashes"][0]["content"].is_string(),
                "{} carries its checksum",
                one["name"]
            );
        }
    }

    #[test]
    fn a_file_without_the_section_is_refused() {
        let dir = std::env::temp_dir().join(format!("teistro-sbom-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let with = dir.join("with");
        let without = dir.join("without");
        fs::write(&with, b"\x7fELF....dep-v0..").unwrap();
        fs::write(&without, b"\x7fELF........").unwrap();
        assert!(embedded(&[&with]).is_ok());
        assert!(
            embedded(&[&without])
                .unwrap_err()
                .contains("cargo auditable")
        );
        fs::remove_dir_all(&dir).unwrap();
    }
}
