//! The wasm package, staged, and its gate (`03-design/wasm-binding.md`
//! §5).
//!
//! **Staged, not kept.** The package is the Node package's own layer —
//! every file of `bindings/node/lib` but its addon loader — with the wasm
//! package's two loaders beside it, the module bound for the web, and a
//! manifest derived from the Node package's with only what differs
//! overridden. Nothing of the layer is kept twice, so the two packages
//! cannot drift; `#native` in `index.js` is the seam, mapped to the addon
//! in one manifest and to the module in the other.
//!
//! **The gate** runs the staged package three ways:
//!
//! 1. **The Node binding's whole suite, unchanged**, from inside the
//!    staged package, so `../lib/index.js` is the package's and its
//!    `#native` resolves through the package's own `node` condition to
//!    the wasm loader. Every call the suite makes goes through the wasm
//!    glue *and* the loader a consumer gets.
//! 2. **In a headless browser**, unbundled, with `#native` mapped to the
//!    web loader as a bundler maps it from the `default` condition. A Node
//!    built-in anywhere on that path could not resolve there, so loading
//!    at all proves there is none; the page runs a probe and posts its
//!    answer back.
//! 3. **The same probe under Node**, whose answer the browser's must equal
//!    to the bit: longitudes as exact text, the settings hash, the
//!    refusal of a plugin.
//! 4. **As a consumer installs it**: packed as `npm publish` would pack
//!    it, installed into an empty project, and asked the four facts the
//!    Node package's consumer asks, by the same file — so an export left
//!    out of `files` or a loader the tarball lacks fails here and not in
//!    the field.
//!
//! Run by hand (`cargo xtask check-wasm`) and in the nightly matrix. The
//! browser step needs Chrome (`CHROME`, or where it installs) and prints
//! `skip` without it.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Value, json};

use crate::binding::{blob_fixtures, cargo, present, step, tool};
use crate::platform::NPM_WASM;

/// The crate the module is built from, and the file Cargo names it.
const PACKAGE: &str = "teistro-wasm";
const MODULE: &str = "target/wasm32-unknown-unknown/release/teistro_wasm.wasm";
/// Where the package is staged.
const STAGED: &str = "target/wasm/package";
/// Where the throwaway project that installs it is built.
const CONSUMER: &str = "target/wasm/consumer";
/// The Node package, whose layer and manifest the wasm package is made of.
const NODE: &str = "bindings/node";
/// The Node package's loader, the one file of its layer the wasm package
/// replaces.
const NODE_LOADER: &str = "addon.js";
/// The wasm package's own sources: its loaders, its README, the browser
/// check.
const WASM: &str = "bindings/wasm";
/// Where the CLI is installed when the machine has none of the right
/// version: inside the build tree, so nothing outside the repository is
/// touched and a cache of `target/` keeps it.
const TOOLS: &str = "target/tools";
const FIXTURES: &str = "target/tsrb";
/// The files every package carries whatever else is in it.
const LEGAL: [&str; 2] = ["LICENSE", "NOTICE"];

/// The `wasm-bindgen` version the lockfile resolved. The CLI must be that
/// version exactly: the two halves of the bindings agree on a schema only
/// within one release, and a mismatched pair fails with an error about the
/// schema rather than about the version.
fn locked_version(root: &Path) -> Option<String> {
    let lock = fs::read_to_string(root.join("Cargo.lock")).ok()?;
    let mut lines = lock.lines();
    while let Some(line) = lines.next() {
        if line == r#"name = "wasm-bindgen""# {
            return lines
                .next()?
                .strip_prefix(r#"version = ""#)?
                .strip_suffix('"')
                .map(str::to_string);
        }
    }
    None
}

/// The CLI's own version, as it prints it (`wasm-bindgen 0.2.129`).
fn cli_version(cli: &Path) -> Option<String> {
    let output = Command::new(cli).arg("--version").output().ok()?;
    String::from_utf8(output.stdout)
        .ok()?
        .split_whitespace()
        .nth(1)
        .map(str::to_string)
}

/// A `wasm-bindgen` CLI of the locked version: `WASM_BINDGEN` when set,
/// then the one on the path, then one this gate installed before, and
/// otherwise one it installs now with `cargo install --locked`.
fn wasm_bindgen(root: &Path, wanted: &str) -> Result<PathBuf, ()> {
    let installed = root
        .join(TOOLS)
        .join("bin")
        .join(format!("wasm-bindgen{}", std::env::consts::EXE_SUFFIX));
    let candidates = [
        std::env::var_os("WASM_BINDGEN").map(PathBuf::from),
        Some(PathBuf::from("wasm-bindgen")),
        Some(installed.clone()),
    ];
    if let Some(found) = candidates
        .into_iter()
        .flatten()
        .find(|cli| cli_version(cli).as_deref() == Some(wanted))
    {
        return Ok(found);
    }
    step(
        Command::new(cargo())
            .args(["install", "--quiet", "--locked", "wasm-bindgen-cli"])
            .args(["--version", wanted, "--root"])
            .arg(root.join(TOOLS))
            .current_dir(root),
        &format!("wasm-bindgen {wanted} installed under {TOOLS}"),
        &format!("wasm-bindgen {wanted} could not be installed"),
    )?;
    Ok(installed)
}

/// The wasm package's manifest: the Node package's, with only what
/// differs overridden — its name and description, where it lives, the
/// import map that picks the loader, the files it carries — and the
/// Node package's platform dependencies and scripts dropped. The version,
/// the licence, the engines and every export come from the one manifest.
fn manifest(node: &Value) -> Value {
    let mut manifest = node.clone();
    if let Some(fields) = manifest.as_object_mut() {
        fields.insert("name".into(), json!(NPM_WASM));
        fields.insert(
            "description".into(),
            json!("Teistro SDK as WebAssembly: the same layer as @teistro/sdk over a wasm module, for browsers, workers and any JavaScript host without a prebuilt addon."),
        );
        fields.insert(
            "imports".into(),
            json!({ "#native": { "node": "./lib/native.node.js", "default": "./lib/native.web.js" } }),
        );
        fields.insert(
            "files".into(),
            json!(["lib/", "wasm/", "README.md", "LICENSE", "NOTICE"]),
        );
        fields.remove("optionalDependencies");
        fields.remove("scripts");
    }
    if let Some(repository) = manifest
        .get_mut("repository")
        .and_then(Value::as_object_mut)
    {
        repository.insert("directory".into(), json!(WASM));
    }
    manifest
}

/// Copies every file of a directory, but the ones named, into another.
fn copy_files(from: &Path, to: &Path, except: &[&str]) -> io::Result<()> {
    for entry in fs::read_dir(from)? {
        let path = entry?.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if path.is_file() && !except.contains(&name) {
            fs::copy(&path, to.join(name))?;
        }
    }
    Ok(())
}

/// Writes the package's files around a module already bound into
/// `<directory>/wasm`.
fn assemble(root: &Path, directory: &Path) -> io::Result<()> {
    let lib = directory.join("lib");
    fs::create_dir_all(&lib)?;
    copy_files(&root.join(NODE).join("lib"), &lib, &[NODE_LOADER])?;
    copy_files(&root.join(WASM).join("lib"), &lib, &[])?;
    let node: Value =
        serde_json::from_str(&fs::read_to_string(root.join(NODE).join("package.json"))?)
            .map_err(io::Error::other)?;
    let text = serde_json::to_string_pretty(&manifest(&node)).map_err(io::Error::other)?;
    fs::write(directory.join("package.json"), format!("{text}\n"))?;
    fs::copy(
        root.join(WASM).join("README.md"),
        directory.join("README.md"),
    )?;
    for legal in LEGAL {
        fs::copy(root.join(legal), directory.join(legal))?;
    }
    Ok(())
}

/// Builds the module, binds it for the web and stages the package at
/// `directory`, replacing whatever was there.
pub(crate) fn stage(root: &Path, directory: &Path) -> Result<(), ()> {
    let Some(wanted) = locked_version(root) else {
        println!("FAIL  Cargo.lock names no `wasm-bindgen`; the wasm crate has lost its binding");
        return Err(());
    };
    let cli = wasm_bindgen(root, &wanted)?;
    step(
        Command::new(cargo())
            .args(["build", "--quiet", "--release", "-p", PACKAGE])
            .args(["--target", "wasm32-unknown-unknown"])
            .current_dir(root),
        "",
        "the wasm module did not build",
    )?;
    if directory.exists() {
        fs::remove_dir_all(directory)
            .map_err(|e| println!("FAIL  {}: {e}", directory.display()))?;
    }
    // `web` output instantiates from a URL or from bytes, which is what
    // both loaders need; the layer's own declarations are the package's
    // types, so the glue's are not written.
    step(
        Command::new(cli)
            .args(["--target", "web", "--no-typescript", "--out-dir"])
            .arg(directory.join("wasm"))
            .arg(MODULE)
            .current_dir(root),
        "",
        "wasm-bindgen could not bind the module",
    )?;
    assemble(root, directory)
        .map_err(|e| println!("FAIL  the wasm package could not be staged: {e}"))
}

/// Chrome, when the machine has it: `CHROME`, then where each platform
/// installs it, then the names a Linux runner puts on the path.
fn chrome() -> Option<PathBuf> {
    let named = std::env::var_os("CHROME").map(PathBuf::from);
    let installed = [
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/usr/bin/google-chrome",
        "/usr/bin/google-chrome-stable",
        "/usr/bin/chromium",
        "/usr/bin/chromium-browser",
    ]
    .map(PathBuf::from);
    named
        .into_iter()
        .chain(installed)
        .find(|path| path.is_file())
}

/// A probe's answer as a script printed it, or the error it reported.
fn answer_of(printed: &[u8]) -> Result<Value, String> {
    let report: Value = serde_json::from_slice(printed)
        .map_err(|e| format!("not a report ({e}): {}", String::from_utf8_lossy(printed)))?;
    report
        .get("answer")
        .cloned()
        .ok_or_else(|| report["error"].as_str().unwrap_or("no answer").to_string())
}

/// The probe in a headless browser and under Node, held equal.
fn browser(root: &Path, staged: &Path) -> Result<(), ()> {
    let Some(chrome) = chrome() else {
        println!("skip  the browser check needs Chrome (set CHROME to its binary)");
        return Ok(());
    };
    let check = root.join(WASM).join("browser");
    let run = |script: &str, args: &[&std::ffi::OsStr]| {
        Command::new("node")
            .arg(check.join(script))
            .args(args)
            .current_dir(root)
            .output()
            .map_err(|e| e.to_string())
            .and_then(|output| answer_of(&output.stdout))
    };
    let in_browser = run("run.mjs", &[staged.as_os_str(), chrome.as_os_str()]);
    let in_node = run("node.mjs", &[staged.as_os_str()]);
    match (in_browser, in_node) {
        (Ok(browser), Ok(node)) if browser == node => {
            println!(
                "ok    the package loads in a headless browser with no Node built-in, and answers as Node does to the bit ({})",
                browser["target"].as_str().unwrap_or_default()
            );
            Ok(())
        }
        (Ok(browser), Ok(node)) => {
            println!(
                "FAIL  the browser and Node answer differently:\n  browser {browser}\n  node    {node}"
            );
            Err(())
        }
        (Err(error), _) => {
            println!("FAIL  the package did not run in the browser: {error}");
            Err(())
        }
        (_, Err(error)) => {
            println!("FAIL  the probe did not run under Node: {error}");
            Err(())
        }
    }
}

/// The staged package packed, installed into an empty project, and run by
/// the Node package's own consumer.
fn consumer(root: &Path, staged: &Path) -> Result<(), ()> {
    let Some(npm) = tool("npm", "--version") else {
        println!("skip  the installed wasm package: no `npm` on this machine");
        return Ok(());
    };
    let project = root.join(CONSUMER);
    if project.exists() {
        fs::remove_dir_all(&project).map_err(|e| println!("FAIL  {CONSUMER}: {e}"))?;
    }
    fs::create_dir_all(&project).map_err(|e| println!("FAIL  {CONSUMER}: {e}"))?;
    step(
        Command::new(&npm)
            .args(["pack", "--silent", "--pack-destination"])
            .arg(&project)
            .arg(staged)
            .current_dir(root),
        "",
        "the wasm package did not pack",
    )?;
    let tarball = fs::read_dir(&project)
        .map_err(|e| println!("FAIL  {CONSUMER}: {e}"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.extension().is_some_and(|ext| ext == "tgz"))
        .ok_or_else(|| println!("FAIL  npm packed no tarball of the wasm package"))?;
    fs::write(
        project.join("package.json"),
        "{\n  \"name\": \"teistro-wasm-packaging-check\",\n  \"private\": true,\n  \"type\": \"module\"\n}\n",
    )
    .map_err(|e| println!("FAIL  {CONSUMER}: {e}"))?;
    step(
        Command::new(&npm)
            .args(["install", "--silent", "--no-audit", "--no-fund"])
            .arg(&tarball)
            .current_dir(&project),
        "",
        "the wasm package did not install",
    )?;
    // Copied rather than run where it lives: a script inside the
    // repository would resolve the package to the repository itself.
    let script = project.join("consumer.mjs");
    fs::copy(root.join(NODE).join("packaging/consumer.mjs"), &script)
        .map_err(|e| println!("FAIL  the consumer did not copy: {e}"))?;
    step(
        Command::new("node")
            .arg(&script)
            .env("TEISTRO_PACKAGE", NPM_WASM)
            .current_dir(&project),
        "the installed wasm package answers as the library does",
        "the installed wasm package did not answer",
    )
}

pub(crate) fn check(root: &Path) -> i32 {
    if !present("node", "--version") {
        eprintln!("no `node` on this machine; the wasm binding's tests need it");
        return 0;
    }
    let staged = root.join(STAGED);
    let tests = staged.join("test");
    let outcome = stage(root, &staged)
        .and_then(|()| blob_fixtures(root, &root.join(FIXTURES)))
        .and_then(|()| {
            fs::create_dir_all(&tests)
                .and_then(|()| copy_files(&root.join(NODE).join("test"), &tests, &[]))
                .map_err(|e| println!("FAIL  the suite could not be copied in: {e}"))
        })
        .and_then(|()| {
            step(
                Command::new("node")
                    .args(["--test", "test/"])
                    .env("TEISTRO_FIXTURES", root.join(FIXTURES))
                    // A plugin is a shared library, which a wasm module
                    // cannot open (ADR-0029); the suite's plugin tests
                    // skip without an adapter, and here there is none.
                    .env_remove("TEISTRO_TEIMERIS_ADAPTER")
                    .current_dir(&staged),
                &format!("{NODE}/test/ passes against the staged wasm package, unchanged"),
                &format!("{NODE}/test/ did not pass against the staged wasm package"),
            )
        })
        .and_then(|()| browser(root, &staged))
        .and_then(|()| consumer(root, &staged));
    // The suite was copied in to run from inside the package; it is not
    // part of what a consumer installs.
    let _ = fs::remove_dir_all(&tests);
    if outcome.is_err() {
        return 1;
    }
    if let Ok(meta) = fs::metadata(staged.join("wasm/teistro_wasm_bg.wasm")) {
        let tenths = meta.len() / 100_000;
        println!(
            "info  the module is {}.{} MB unoptimised; the profile binaries and their size gate are step 7",
            tenths / 10,
            tenths % 10
        );
    }
    0
}

#[cfg(test)]
mod tests {
    #![allow(clippy::indexing_slicing, reason = "a test fails by panicking")]

    use serde_json::json;

    use super::{locked_version, manifest};

    /// The wasm manifest is the Node one but for what it overrides: the
    /// exports, the version and the engines cannot drift apart.
    #[test]
    fn the_wasm_manifest_is_the_node_one_but_for_its_loader() {
        let node = json!({
            "name": "@teistro/sdk",
            "version": "1.2.3",
            "repository": { "type": "git", "directory": "bindings/node" },
            "exports": { ".": { "default": "./lib/index.js" } },
            "engines": { "node": ">=20" },
            "imports": { "#native": "./lib/addon.js" },
            "scripts": { "test": "node --test" },
            "optionalDependencies": { "@teistro/sdk-linux-x64": "1.2.3" },
        });
        let wasm = manifest(&node);
        assert_eq!(wasm["name"], "@teistro/sdk-wasm");
        for kept in ["version", "exports", "engines"] {
            assert_eq!(wasm[kept], node[kept], "{kept}");
        }
        assert_eq!(wasm["repository"]["directory"], "bindings/wasm");
        assert_eq!(wasm["imports"]["#native"]["default"], "./lib/native.web.js");
        assert!(wasm.get("optionalDependencies").is_none() && wasm.get("scripts").is_none());
    }

    /// Each loader's `platformPackage()` names the package the manifest
    /// is published as.
    #[test]
    fn the_loaders_name_the_package_they_are_in() {
        let lib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../bindings/wasm/lib");
        for loader in ["native.web.js", "native.node.js"] {
            let text = std::fs::read_to_string(lib.join(loader)).unwrap_or_default();
            assert!(
                text.contains(&format!("return '{}';", super::NPM_WASM)),
                "{loader}"
            );
        }
    }

    /// The version comes from the lockfile the workspace builds with, so
    /// the CLI can never be pinned apart from the library.
    #[test]
    fn the_cli_is_pinned_to_the_locked_library() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let version = locked_version(&root);
        assert!(
            version
                .as_deref()
                .is_some_and(|v| v.split('.').count() == 3),
            "{version:?}"
        );
    }
}
