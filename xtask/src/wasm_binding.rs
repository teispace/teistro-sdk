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
//! **The gate** runs the staged package these ways:
//!
//! 1. **The Node binding's whole suite, unchanged**, from inside the
//!    staged package, so `../lib/index.js` is the package's and its
//!    `#native` resolves through the package's own `node` condition to
//!    the wasm loader. Every call the suite makes goes through the wasm
//!    glue *and* the loader a consumer gets. Then **each profile's
//!    subpath** (`03-design/wasm-profiles.md`), both entries under Node:
//!    what the profile keeps answers as the full module does, to the bit,
//!    and the chart area is refused as a capability.
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
//! 5. **In Cloudflare's workerd**, from the installed package: a Worker
//!    that imports it by name, bundled by the pinned Wrangler as a
//!    deploy bundles it (which resolves `#native` through the `workerd`
//!    condition) and run by `workerd test`. Its answer too must equal
//!    Node's to the bit.
//! 6. **Shaken by the bundlers**, from the installed package: esbuild,
//!    Vite and webpack, pinned in `bindings/wasm/bundlers`, each bundle
//!    one member of `/catalogue` and must ship no module and at most
//!    [`SHAKEN_MOST`] bytes; each also bundles the entry and each
//!    profile's subpath, which must ship their own module and no other,
//!    so the first measurement is known to be able to see one.
//! 7. **Each module held to its size budget** (`docs/05-testing/sizes.json`), both ways:
//!    over it fails, and so does more than 5% under it, since a budget
//!    that loose would let the saving go unnoticed. What it measured,
//!    the modules, their glue and every bundle, is written as this run's
//!    sizes fragment for `SIZES.md` (`xtask/src/sizes.rs`).
//!
//! Run by hand (`cargo xtask check-wasm`) and in the nightly matrix. The
//! browser step needs Chrome (`CHROME`, or where it installs) and prints
//! `skip` without it; the Workers step installs its own tools and skips
//! only without npm.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Value, json};

use crate::binding::{blob_fixtures, cargo, pinned_npm_tool, present, step, tool};
use crate::platform::NPM_WASM;
use crate::sbom;
use crate::sizes::{self, Allowed, Artefact, Row};

/// The crate the module is built from, and the file Cargo names it.
const PACKAGE: &str = "teistro-wasm";
/// The Cargo profile the module is built with: `release`, optimised for
/// size with fat LTO, as the workspace manifest measures.
const PROFILE: &str = "wasm";
const MODULE: &str = "target/wasm32-unknown-unknown/wasm/teistro_wasm.wasm";
/// Where the package is staged.
pub(crate) const STAGED: &str = "target/wasm/package";
/// Where the throwaway project that installs it is built.
const CONSUMER: &str = "target/wasm/consumer";
/// The Workers check: its Worker, its runner, and the pinned tools.
const WORKERD: &str = "bindings/wasm/workerd";
/// The tree-shaking check: its runner and the pinned bundlers.
const BUNDLERS: &str = "bindings/wasm/bundlers";
/// The most a bundle importing one member of `/catalogue` may weigh. The
/// three bundlers wrote 278 to 288 bytes on 2026-10-08, and 38 to 68 kB
/// before the generated tables were marked pure: the budget sits between
/// the two, near enough to the first to catch a table that stops shaking.
const SHAKEN_MOST: u64 = 1024;
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
/// The package's loaders by the import condition that picks each, **in
/// the order a resolver tries them**: the first condition a host sets
/// wins, so the Workers one comes before `node` (a Worker bundled with
/// Node compatibility sets both) and `default` is last, since it matches
/// everything.
const LOADERS: [(&str, &str); 3] = [
    ("workerd", "native.workerd.js"),
    ("node", "native.node.js"),
    ("default", "native.web.js"),
];
/// The files every package carries whatever else is in it.
const LEGAL: [&str; 2] = ["LICENSE", "NOTICE"];

/// A profile's module, shipped beside the full one under a subpath of its
/// own (`03-design/wasm-profiles.md`): the same layer over a module built
/// with fewer families.
pub(crate) struct Profile {
    /// The subpath, the directory under `wasm/` the module is bound into,
    /// and the wasm crate's feature that builds it.
    pub(crate) name: &'static str,
}

/// The profiles the package ships beside its full module. `panchanga` is
/// the calendars, the almanac and the muhurta search: what a patro needs,
/// at three fifths of the full module gzipped.
pub(crate) const PROFILES: [Profile; 1] = [Profile { name: "panchanga" }];

impl Profile {
    /// The import the profile's entry reads its native half from.
    fn import(&self) -> String {
        format!("#native-{}", self.name)
    }

    /// The profile's loader for one of [`LOADERS`]: `native.web.js`
    /// becomes `native-panchanga.web.js`.
    fn loader(&self, loader: &str) -> String {
        format!(
            "native-{}.{}",
            self.name,
            loader.strip_prefix("native.").unwrap_or(loader)
        )
    }
}

/// `text` with every `from` replaced by `to`, refused when there is none:
/// a generated file made by rewriting another must fail when the source
/// stops saying what the rewrite expects, not ship it unchanged.
fn rewritten(text: &str, from: &str, to: &str, what: &str) -> io::Result<String> {
    if text.contains(from) {
        Ok(text.replace(from, to))
    } else {
        Err(io::Error::other(format!("{what} no longer says `{from}`")))
    }
}

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
        let loaders: serde_json::Map<String, Value> = LOADERS
            .iter()
            .map(|(condition, file)| ((*condition).to_owned(), json!(format!("./lib/{file}"))))
            .collect();
        let mut imports = serde_json::Map::new();
        imports.insert("#native".into(), Value::Object(loaders));
        for profile in &PROFILES {
            let loaders: serde_json::Map<String, Value> = LOADERS
                .iter()
                .map(|(condition, file)| {
                    (
                        (*condition).to_owned(),
                        json!(format!("./lib/{}", profile.loader(file))),
                    )
                })
                .collect();
            imports.insert(profile.import(), Value::Object(loaders));
        }
        fields.insert("imports".into(), Value::Object(imports));
        // A profile's entry is the package's own layer, so its types are
        // the entry's: a call it leaves out is still typed, and refused.
        if let Some(exports) = fields.get_mut("exports").and_then(Value::as_object_mut) {
            for profile in &PROFILES {
                exports.insert(
                    format!("./{}", profile.name),
                    json!({
                        "types": "./lib/index.d.ts",
                        "default": format!("./lib/{}.js", profile.name),
                    }),
                );
            }
        }
        fields.insert(
            "files".into(),
            json!([
                "lib/",
                "wasm/",
                "README.md",
                "LICENSE",
                "NOTICE",
                sbom::FILE
            ]),
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
    // Each profile's entry and loaders, written from the package's own so
    // that nothing of the layer is kept twice: the entry reads its native
    // half from the profile's import, and each loader its module from the
    // profile's directory.
    let index = fs::read_to_string(lib.join("index.js"))?;
    for profile in &PROFILES {
        fs::write(
            lib.join(format!("{}.js", profile.name)),
            rewritten(
                &index,
                "from '#native';",
                &format!("from '{}';", profile.import()),
                "the layer's index.js",
            )?,
        )?;
        for (_, loader) in LOADERS {
            let text = fs::read_to_string(lib.join(loader))?;
            fs::write(
                lib.join(profile.loader(loader)),
                rewritten(
                    &text,
                    "'../wasm/teistro_wasm",
                    &format!("'../wasm/{}/teistro_wasm", profile.name),
                    loader,
                )?,
            )?;
        }
    }
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
    // One module for every host, so one bill, for its own target. The
    // module is not built through `cargo auditable`: the bill is what says
    // what it carries.
    sbom::write(
        root,
        "wasm32-unknown-unknown",
        PACKAGE,
        &directory.join(sbom::FILE),
    )
    .map_err(io::Error::other)
}

/// Builds the module with the wasm crate's `features` (its defaults when
/// there are none) and binds it for the web into `out`.
fn module(root: &Path, cli: &Path, features: Option<&str>, out: &Path) -> Result<(), ()> {
    let mut build = Command::new(cargo());
    build
        .args(["build", "--quiet", "--profile", PROFILE, "-p", PACKAGE])
        .args(["--target", "wasm32-unknown-unknown"]);
    if let Some(features) = features {
        build.args(["--no-default-features", "--features", features]);
    }
    step(
        build.current_dir(root),
        "",
        &format!(
            "the wasm module did not build ({})",
            features.unwrap_or("full")
        ),
    )?;
    // `web` output instantiates from a URL or from bytes, which is what
    // both loaders need; the layer's own declarations are the package's
    // types, so the glue's are not written. The name section is a debugger's
    // and was 47% of the module; a refusal reaches the caller as a
    // `TeistroError` with its record, not as a stack of function names.
    step(
        Command::new(cli)
            .args(["--target", "web", "--no-typescript"])
            .args(["--remove-name-section", "--remove-producers-section"])
            .arg("--out-dir")
            .arg(out)
            .arg(MODULE)
            .current_dir(root),
        "",
        "wasm-bindgen could not bind the module",
    )
}

/// Builds the modules, binds each for the web and stages the package at
/// `directory`, replacing whatever was there. The profiles are built after
/// the full module, one at a time, since each build writes the same file.
pub(crate) fn stage(root: &Path, directory: &Path) -> Result<(), ()> {
    let Some(wanted) = locked_version(root) else {
        println!("FAIL  Cargo.lock names no `wasm-bindgen`; the wasm crate has lost its binding");
        return Err(());
    };
    let cli = wasm_bindgen(root, &wanted)?;
    if directory.exists() {
        fs::remove_dir_all(directory)
            .map_err(|e| println!("FAIL  {}: {e}", directory.display()))?;
    }
    let wasm = directory.join("wasm");
    module(root, &cli, None, &wasm)?;
    for profile in &PROFILES {
        module(root, &cli, Some(profile.name), &wasm.join(profile.name))?;
    }
    assemble(root, directory)
        .map_err(|e| println!("FAIL  the wasm package could not be staged: {e}"))
}

/// Whether the toolchain can build for the module's target: its standard
/// library is where `rustc` says a target's libraries go.
pub(crate) fn target_installed() -> bool {
    Command::new("rustc")
        .args([
            "--print",
            "target-libdir",
            "--target",
            "wasm32-unknown-unknown",
        ])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .is_some_and(|dir| Path::new(dir.trim()).is_dir())
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

/// A probe script under `bindings/wasm`, run by Node: what it printed,
/// read as the probe's answer or the error it reported.
fn probe_by(root: &Path, script: &str, args: &[&std::ffi::OsStr]) -> Result<Value, String> {
    Command::new("node")
        .arg(root.join(WASM).join(script))
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|e| e.to_string())
        .and_then(|output| answer_of(&output.stdout))
}

/// The probe under Node, through the staged package's own `node` loader:
/// the answer every other host is held to.
fn node_answer(root: &Path, staged: &Path) -> Result<Value, ()> {
    probe_by(root, "browser/node.mjs", &[staged.as_os_str()])
        .map_err(|error| println!("FAIL  the probe did not run under Node: {error}"))
}

/// A host's answer held equal to Node's, to the bit.
fn held_to_node(
    host: &str,
    proof: &str,
    answer: Result<Value, String>,
    node: &Value,
) -> Result<(), ()> {
    match answer {
        Ok(answer) if answer == *node => {
            println!(
                "ok    {proof}, and answers as Node does to the bit ({})",
                answer["target"].as_str().unwrap_or_default()
            );
            Ok(())
        }
        Ok(answer) => {
            println!("FAIL  {host} and Node answer differently:\n  {host} {answer}\n  node {node}");
            Err(())
        }
        Err(error) => {
            println!("FAIL  the package did not run in {host}: {error}");
            Err(())
        }
    }
}

/// The probe in a headless browser, unbundled.
fn browser(root: &Path, staged: &Path, node: &Value) -> Result<(), ()> {
    let Some(chrome) = chrome() else {
        crate::skip::skip("the browser check needs Chrome (set CHROME to its binary)");
        return Ok(());
    };
    held_to_node(
        "the browser",
        "the package loads in a headless browser with no Node built-in",
        probe_by(
            root,
            "browser/run.mjs",
            &[staged.as_os_str(), chrome.as_os_str()],
        ),
        node,
    )
}

/// The probe in Cloudflare's workerd, from the **installed** package,
/// bundled by the pinned Wrangler as a consumer's deploy bundles it.
///
/// The tools are pinned in `bindings/wasm/workerd/package.json` and
/// installed from its lock file, so this runs wherever npm does; it skips
/// only where the install cannot happen, and says so.
fn workerd(root: &Path, node: &Value) -> Result<(), ()> {
    if pinned_npm_tool(&root.join(WORKERD), "wrangler").is_none() {
        crate::skip::skip(
            "the Workers check: the pinned Wrangler is not installed and could not be (needs `npm`)",
        );
        return Ok(());
    }
    held_to_node(
        "workerd",
        "the installed package bundles with Wrangler and runs in workerd through its `workerd` loader",
        probe_by(root, "workerd/run.mjs", &[root.join(CONSUMER).as_os_str()]),
        node,
    )
}

/// The installed package bundled by each pinned bundler: one member of
/// `/catalogue` ships no module and stays under [`SHAKEN_MOST`]; the
/// entry ships the full module, and each profile's subpath its own module
/// and nothing else, told apart by what the emitted modules weigh.
fn shaken(root: &Path, staged: &Path, measured: &mut Vec<Row>) -> Result<(), ()> {
    let dir = root.join(BUNDLERS);
    for bundler in sizes::BUNDLERS {
        if pinned_npm_tool(&dir, bundler).is_none() {
            crate::skip::skip(format_args!(
                "the tree-shaking check: the pinned {bundler} is not installed and could not be (needs `npm`)"
            ));
            return Ok(());
        }
    }
    let weight = |bound: PathBuf| {
        fs::metadata(bound.join("teistro_wasm_bg.wasm"))
            .map(|meta| meta.len())
            .map_err(|e| println!("FAIL  the staged module could not be read: {e}"))
    };
    let wasm = staged.join("wasm");
    // Each entry's module: its weight, and the end of the path a bundle
    // that leaves the module to be served names it by.
    let mut modules = vec![(
        String::from("everything"),
        weight(wasm.clone())?,
        String::from("/wasm/teistro_wasm_bg.wasm"),
    )];
    for profile in &PROFILES {
        modules.push((
            profile.name.to_owned(),
            weight(wasm.join(profile.name))?,
            format!("/wasm/{}/teistro_wasm_bg.wasm", profile.name),
        ));
    }
    let rows = probe_by(root, "bundlers/run.mjs", &[root.join(CONSUMER).as_os_str()]).map_err(
        |error| println!("FAIL  the bundlers did not bundle the installed package: {error}"),
    )?;
    let rows = rows.as_array().cloned().unwrap_or_default();
    if rows.is_empty() {
        println!("FAIL  the bundlers reported no bundle");
        return Err(());
    }
    let mut failed = false;
    for row in &rows {
        let bundler = row["bundler"].as_str().unwrap_or("?");
        let entry = row["entry"].as_str().unwrap_or("?");
        let bytes = row["bytes"].as_u64().unwrap_or(u64::MAX);
        let wasm = row["wasm"].as_bool().unwrap_or(false);
        let shipped = row["wasmBytes"].as_u64().unwrap_or(0);
        let references: Vec<&str> = row["references"]
            .as_array()
            .map(|paths| paths.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default();
        measured.push(Row {
            subject: sizes::bundle_subject(bundler, entry),
            artefact: Artefact::Bundle,
            raw: Some(bytes),
            gzip: None,
        });
        let wanted = modules
            .iter()
            .find(|(name, _, _)| name == entry)
            .map(|(_, weight, path)| (*weight, path.as_str()));
        let complaint = match (entry, wanted) {
            ("catalogue", _) if wasm => Some(String::from("ships the module")),
            ("catalogue", _) if bytes > SHAKEN_MOST => {
                Some(format!("weighs {bytes} bytes, over {SHAKEN_MOST}"))
            }
            ("catalogue", _) => None,
            (_, None) => Some(String::from("is an entry the check does not know")),
            _ if !wasm => Some(String::from(
                "does not ship the module, so the check cannot see one",
            )),
            (_, Some((weight, _))) if shipped > 0 && shipped != weight => Some(format!(
                "ships {shipped} bytes of module where its own weighs {weight}"
            )),
            (_, Some((_, path)))
                if shipped == 0
                    && (references.is_empty() || !references.iter().all(|r| r.ends_with(path))) =>
            {
                Some(format!(
                    "emits no module and names {references:?} where its own is `…{path}`"
                ))
            }
            _ => None,
        };
        match complaint {
            Some(why) => {
                failed = true;
                println!("FAIL  {bundler} bundling `{entry}` {why}");
            }
            None => println!(
                "ok    {bundler} bundles `{entry}` in {bytes} bytes{}",
                if shipped > 0 {
                    format!(", with its module of {shipped} bytes")
                } else if wasm {
                    format!(", naming its module at {}", references.join(", "))
                } else {
                    String::from(", without the module")
                }
            ),
        }
    }
    if failed { Err(()) } else { Ok(()) }
}

/// Each profile's entry under Node, from the staged package: what it
/// keeps answers as the full module does, to the bit, and a call into the
/// chart area, which every profile leaves out, is refused as a capability
/// naming the family.
fn profiles(root: &Path, staged: &Path) -> Result<(), ()> {
    let mut failed = false;
    for profile in &PROFILES {
        let name = profile.name;
        let answer = match probe_by(
            root,
            "profile.mjs",
            &[staged.as_os_str(), std::ffi::OsStr::new(name)],
        ) {
            Ok(answer) => answer,
            Err(error) => {
                println!("FAIL  the `{name}` entry did not run under Node: {error}");
                failed = true;
                continue;
            }
        };
        if answer["full"] == answer["profile"] && answer["full"].is_object() {
            println!(
                "ok    `{NPM_WASM}/{name}` answers the calendars, the almanac and the muhurta search as the full module does, to the bit"
            );
        } else {
            println!(
                "FAIL  `{NPM_WASM}/{name}` and the full module answer differently:\n  {name} {}\n  full {}",
                answer["profile"], answer["full"]
            );
            failed = true;
        }
        let refusal = &answer["refusal"];
        let message = refusal["message"].as_str().unwrap_or_default();
        if refusal["status"] == "CAPABILITY" && message.contains("`chart`") {
            println!("ok    `{NPM_WASM}/{name}` refuses a chart as a capability: {message}");
        } else {
            println!(
                "FAIL  `{NPM_WASM}/{name}` did not refuse a chart as a capability naming `chart`: {refusal}"
            );
            failed = true;
        }
    }
    if failed { Err(()) } else { Ok(()) }
}

/// The staged package packed, installed into an empty project, and run by
/// the Node package's own consumer.
fn consumer(root: &Path, staged: &Path) -> Result<(), ()> {
    let Some(npm) = tool("npm", "--version") else {
        crate::skip::skip("the installed wasm package: no `npm` on this machine");
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
        crate::skip::skip("no `node` on this machine; the wasm binding's tests need it");
        return 0;
    }
    let staged = root.join(STAGED);
    let tests = staged.join("test");
    let mut bundles = Vec::new();
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
                    .args(["--test", crate::node_binding::SUITE])
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
        .and_then(|()| profiles(root, &staged))
        .and_then(|()| node_answer(root, &staged))
        .and_then(|node| {
            browser(root, &staged, &node)
                .and_then(|()| consumer(root, &staged))
                .and_then(|()| workerd(root, &node))
                .and_then(|()| shaken(root, &staged, &mut bundles))
        });
    // The suite was copied in to run from inside the package; it is not
    // part of what a consumer installs.
    let _ = fs::remove_dir_all(&tests);
    if outcome.is_err() {
        return 1;
    }
    let held = size(root, &staged);
    let mut rows = sizes::wasm_rows(&staged);
    rows.append(&mut bundles);
    let recorded = sizes::write_fragment(root, "wasm", rows);
    i32::from(held.is_err() || recorded.is_err())
}

/// A size as a reader reads it: megabytes to two places.
fn megabytes(bytes: u64) -> String {
    let hundredths = bytes.div_ceil(10_000);
    format!("{}.{:02} MB", hundredths / 100, hundredths % 100)
}

/// Each shipped module held to the budget the sizes record sets it: the
/// full one, then each profile's.
fn size(root: &Path, staged: &Path) -> Result<(), ()> {
    let wasm = staged.join("wasm");
    let mut failed = false;
    for subject in std::iter::once(sizes::FULL).chain(PROFILES.iter().map(|p| p.name)) {
        let (module, bound) = if subject == sizes::FULL {
            (String::from("the module"), wasm.clone())
        } else {
            (format!("the `{subject}` module"), wasm.join(subject))
        };
        failed |= sizes::budget(root, subject, Artefact::Module)
            .map_err(|why| println!("FAIL  {why}"))
            .and_then(|allowed| held_to_budget(&module, &bound, allowed, subject))
            .is_err();
    }
    if failed { Err(()) } else { Ok(()) }
}

/// One module held to its budget, both ways: over it is a regression, and
/// more than 5% under it is a budget that no longer protects the saving,
/// so the gate says what to write instead. Gzip is the proxy for what a
/// browser downloads; the raw size is what it compiles.
fn held_to_budget(module: &str, bound: &Path, allowed: Allowed, subject: &str) -> Result<(), ()> {
    let bytes = fs::read(bound.join("teistro_wasm_bg.wasm"))
        .map_err(|e| println!("FAIL  {module} could not be read: {e}"))?;
    let gzip = sizes::gzip_best(&bytes);
    let mut failed = false;
    for (name, measured, allowed) in [
        ("raw", bytes.len(), allowed.raw),
        ("gzip", gzip.len(), allowed.gzip),
    ] {
        let measured = u64::try_from(measured).unwrap_or(u64::MAX);
        let ratio = sizes::ratio(measured, allowed);
        let suggested = sizes::suggested(measured);
        let at = format!("`{subject}`'s `{name}` in {}", sizes::RECORD);
        if measured > allowed {
            println!(
                "FAIL  {module}'s {name} is {} ({measured}), over its budget of {} by {:.1}%; if the growth is wanted, set {at} to {suggested}, or re-record with `cargo xtask sizes --from DIR --why SENTENCE`",
                megabytes(measured),
                megabytes(allowed),
                (ratio - 1.0) * 100.0
            );
            failed = true;
        } else if ratio < sizes::SLACK {
            println!(
                "FAIL  {module}'s {name} is {} ({measured}), {:.1}% under its budget of {}; lower {at} to {suggested} so the saving is kept",
                megabytes(measured),
                (1.0 - ratio) * 100.0,
                megabytes(allowed)
            );
            failed = true;
        } else {
            // The exact figure on a pass too, so a budget can be re-measured
            // from any run on the runner that enforces it.
            println!(
                "ok    {module}'s {name} is {} ({measured}) of its {} budget, {:.1}% left",
                megabytes(measured),
                megabytes(allowed),
                (1.0 - ratio) * 100.0
            );
        }
    }
    if failed { Err(()) } else { Ok(()) }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::indexing_slicing, reason = "a test fails by panicking")]

    use serde_json::{Value, json};

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
        for kept in ["version", "engines"] {
            assert_eq!(wasm[kept], node[kept], "{kept}");
        }
        assert_eq!(wasm["repository"]["directory"], "bindings/wasm");
        let conditions: Vec<(&String, &Value)> = wasm["imports"]["#native"]
            .as_object()
            .map(|loaders| loaders.iter().collect())
            .unwrap_or_default();
        let order: Vec<&str> = conditions
            .iter()
            .map(|(condition, _)| condition.as_str())
            .collect();
        assert_eq!(
            order,
            ["workerd", "node", "default"],
            "the order a resolver tries them"
        );
        assert_eq!(wasm["imports"]["#native"]["default"], "./lib/native.web.js");
        // A profile is a subpath of the same layer over its own loaders.
        assert_eq!(
            wasm["imports"]["#native-panchanga"]["workerd"],
            "./lib/native-panchanga.workerd.js"
        );
        assert_eq!(
            wasm["exports"]["./panchanga"]["default"],
            "./lib/panchanga.js"
        );
        assert_eq!(wasm["exports"]["./panchanga"]["types"], "./lib/index.d.ts");
        assert_eq!(wasm["exports"]["."], node["exports"]["."]);
        assert!(wasm.get("optionalDependencies").is_none() && wasm.get("scripts").is_none());
    }

    /// Each loader the manifest names exists and its `platformPackage()`
    /// names the package the manifest is published as.
    #[test]
    fn the_loaders_name_the_package_they_are_in() {
        let lib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../bindings/wasm/lib");
        for (_, loader) in super::LOADERS {
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
