//! The Node binding's own tests: the addon built and loaded, the whole
//! surface exercised through the ergonomic layer, the generated decoders
//! against blobs the library really produced, and the generated types
//! against a consumer that type-checks at maximum strictness (ADR-0023).
//!
//! Run by hand (`cargo xtask check-node`) and in the nightly matrix; the
//! fast check stays Rust-only (ADR-0014). The TypeScript step is skipped
//! with a note when no compiler is on the machine, because a type-check
//! needs one and the tests do not.

use std::path::Path;
use std::process::Command;

use crate::binding::{blob_fixtures, build, pinned_npm_tool, present, step, tool};
use crate::examples::{Binding, Runtime};
use crate::platform::Platform;

const FIXTURES: &str = "target/tsrb";
const TESTS: &str = "bindings/node/test/";
/// Where the examples live. **Every** file there is run, so a scenario
/// added to the directory is gated by having been added.
/// Where the pinned TypeScript compiler and the strict consumer live.
const TYPECHECK: &str = "bindings/node/typecheck";
const TSCONFIG: &str = "bindings/node/typecheck/tsconfig.json";
/// The layer's members against its declarations, from `bindings/node`.
const SURFACE: &str = "typecheck/surface.mjs";
/// The Teimeris adapter's own package, which the SDK does not depend on
/// and which depends on the SDK.
///
/// Checked here rather than in a gate of its own, because what it needs
/// is this ecosystem's type checker and this ecosystem's strictness: an
/// adapter package that did not compile against the SDK's declarations
/// would be a broken package however green the SDK's own gate was. The
/// generated façade is most of what it holds (ADR-0030).
const ADAPTER_TSCONFIG: &str = "adapters/ephemeris-teimeris/node/tsconfig.json";
/// Where the addon is loaded from: Node requires the `.node` suffix, so
/// the cdylib Cargo builds is copied there.
pub(crate) const ADDON: &str = "bindings/node/native/index.node";

/// The crate that builds the addon, as Cargo names its artefacts.
pub(crate) const ADDON_STEM: &str = "teistro_node";

/// The name Cargo gives the addon on this platform.
pub(crate) fn addon_artefact() -> String {
    Platform::host().shared(ADDON_STEM)
}

/// Builds the addon and puts it where the loader looks.
fn build_addon(root: &Path) -> Result<(), ()> {
    build(root, "teistro-node", "the Node addon")?;
    let built = root.join("target/release").join(addon_artefact());
    let addon = root.join(ADDON);
    std::fs::copy(&built, &addon).map(|_| ()).map_err(|e| {
        println!(
            "FAIL  {} could not be copied to {ADDON}: {e}",
            built.display()
        );
    })
}

/// The TypeScript compiler, when the machine has one: `TSC`, a local
/// install beside the consumer, or one npm has already fetched.
///
/// **The local one is run as what it is**, a JavaScript program, by
/// `node` and not by the `.bin` shim beside it. npm writes that shim as
/// `tsc.cmd` on Windows, and a shim goes through `cmd.exe` -- which is
/// how `The system cannot find the path specified.` became the whole of
/// a win32 failure, with nothing in it naming a tool. `node` is an
/// executable on every platform and `typescript/bin/tsc` is the same
/// file on every platform, so this path has no shim in it at all.
///
/// `npx` stays as the last resort and still goes through [`tool`],
/// because there it is the only thing on offer.
fn typescript(root: &Path) -> Option<(String, Vec<String>)> {
    if let Ok(tsc) = std::env::var("TSC") {
        return Some((tsc, Vec::new()));
    }
    // Installed from the lock file beside it, which is what the site's
    // gate does with its own: a version pinned in the repository means
    // every machine and every runner type-checks with the same compiler,
    // rather than whichever one a runner image happens to carry.
    if let Some(installed) = pinned_npm_tool(&root.join(TYPECHECK), "typescript") {
        let tsc = installed.join("bin/tsc");
        return Some((String::from("node"), vec![tsc.display().to_string()]));
    }
    let npx = tool("npx", "--version")?;
    let fetched = Command::new(&npx)
        .args(["--no-install", "tsc", "--version"])
        .current_dir(root)
        .output();
    match fetched {
        Ok(output) if output.status.success() => Some((
            npx,
            ["--no-install", "tsc"]
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
        )),
        _ => None,
    }
}

pub(crate) fn check(root: &Path) -> i32 {
    if !present("node", "--version") {
        eprintln!("no `node` on this machine; the Node binding's tests need it");
        return 0;
    }
    let fixtures = root.join(FIXTURES);
    if build_addon(root).is_err() {
        return 1;
    }
    let outcome = blob_fixtures(root, &fixtures).and_then(|()| {
        step(
            Command::new("node")
                .args(["--test", TESTS])
                .env("TEISTRO_FIXTURES", &fixtures)
                .current_dir(root),
            &format!("{TESTS} decodes what the library produced"),
            &format!("{TESTS} did not pass"),
        )
    });
    if outcome.is_err() {
        return 1;
    }
    if Binding::Node.run(root, &Runtime::of(root)).is_err() {
        return 1;
    }
    let Some((tsc, args)) = typescript(root) else {
        // The compiler is pinned in `typecheck/package.json` and this
        // gate installs it, so reaching here means npm could not be run
        // or the install failed -- not that the machine was expected to
        // have brought its own.
        println!(
            "skip  {TSCONFIG}: the pinned TypeScript compiler is not installed and could not be (needs `npm`; or set TSC)"
        );
        return 0;
    };
    let checked = step(
        Command::new(&tsc)
            .args(&args)
            .args(["-p", TSCONFIG])
            .current_dir(root),
        &format!("{TSCONFIG} type-checks at maximum strictness"),
        &format!("{TSCONFIG} does not type-check"),
    );
    if checked.is_err() {
        return 1;
    }
    // The declarations are hand-written, and `tsc` proves only what a file
    // uses; this proves every member of every exported class, measured on
    // real instances, is declared and none is declared that is not there.
    let surface = step(
        Command::new("node")
            .arg(SURFACE)
            .current_dir(root.join("bindings/node")),
        "bindings/node/lib/index.d.ts declares the layer member for member",
        "bindings/node/lib/index.d.ts and the layer disagree",
    );
    if surface.is_err() {
        return 1;
    }
    let adapter = step(
        Command::new(&tsc)
            .args(&args)
            .args(["-p", ADAPTER_TSCONFIG])
            .current_dir(root),
        &format!("{ADAPTER_TSCONFIG}: the typed engine façade composes with the SDK"),
        &format!("{ADAPTER_TSCONFIG} does not type-check"),
    );
    i32::from(adapter.is_err())
}
