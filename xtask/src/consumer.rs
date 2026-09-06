//! The proof that what a release ships is usable: the artefacts are
//! built, the packages staged, each installed into a throwaway project,
//! and a consumer that knows nothing but the published names is run
//! against it.
//!
//! Everything else about the bindings is tested from inside the
//! repository, against files by their paths. That proves the code and not
//! the package: an export left out of `files`, a subpath that resolves in
//! a checkout and not in an install, an addon nobody depends on, a header
//! the bundle forgot — none of them can fail a test that imports
//! `../lib/index.js`. They all fail here.
//!
//! Each consumer asserts the same four facts the C smoke test prints, so
//! a package that loads but answers differently fails here rather than in
//! the field. A toolchain that is missing skips its own step with a note,
//! as the other bindings' gates do (ADR-0014).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::binding::{LIBRARY_STEM, present, step};
use crate::package;
use crate::platform::{NPM_SCOPE, Platform};
use crate::release;
use crate::{read, rel};

/// Where the throwaway projects are built, inside `target` so that
/// `cargo clean` takes them away.
const CHECK: &str = "target/dist/check";

pub(crate) fn check(root: &Path) -> i32 {
    if package::build(root, None) != 0 {
        return 1;
    }
    if package::stage(root, true) != 0 {
        return 1;
    }
    let platform = Platform::host();
    let version = release::version(root);
    let dist = root.join("target/dist");
    let check = root.join(CHECK);
    if check.exists() && fs::remove_dir_all(&check).is_err() {
        println!("FAIL  {CHECK} could not be emptied");
        return 1;
    }

    let outcomes = [
        c_consumer(root, &dist, &check, &platform, &version),
        node_consumer(root, &dist, &check, &platform),
        dart_consumer(root, &dist, &check, &platform, &version),
    ];
    let failed = outcomes.iter().filter(|outcome| outcome.is_err()).count();
    println!(
        "three packages installed and run for {}: {failed} failure(s)",
        platform.name()
    );
    i32::from(failed != 0)
}

// ── C ──────────────────────────────────────────────────────────────────────

/// Unpacks the C bundle and builds the smoke test against it, both ways a
/// C consumer links: against the static library, and against the shared
/// one with the loader pointed at the bundle.
fn c_consumer(
    root: &Path,
    dist: &Path,
    check: &Path,
    platform: &Platform,
    version: &str,
) -> Result<(), ()> {
    let cc = std::env::var("CC").unwrap_or_else(|_| String::from("cc"));
    if !present(&cc, "--version") {
        println!("skip  the C bundle: no `{cc}` on this machine");
        return Ok(());
    }
    let into = check.join("c");
    let stem = format!("teistro-c-{version}-{}", platform.name());
    unpack(&dist.join(format!("{stem}.tar.gz")), &into).map_err(|err| {
        println!("FAIL  the C bundle did not unpack: {err}");
    })?;
    let bundle = into.join(&stem);
    let include = bundle.join("include");
    let lib = bundle.join("lib");
    for expected in ["include/teistro.h", "LICENSE", "NOTICE", "README.md"] {
        if !bundle.join(expected).is_file() {
            println!("FAIL  the C bundle has no {expected}");
            return Err(());
        }
    }

    let smoke = root.join("bindings/c/tests/smoke.c");
    let statically = into.join("smoke-static");
    step(
        Command::new(&cc)
            .args(["-std=c11", "-Wall", "-Wextra", "-Wpedantic", "-Werror"])
            .arg("-I")
            .arg(&include)
            .arg("-o")
            .arg(&statically)
            .arg(&smoke)
            .arg(lib.join(platform.static_library(LIBRARY_STEM))),
        "",
        "the C bundle's static library does not link",
    )?;
    step(
        &mut Command::new(&statically),
        "the C bundle links statically and answers",
        "the C bundle's static build did not pass",
    )?;

    let dynamically = into.join("smoke-shared");
    step(
        Command::new(&cc)
            .args(["-std=c11", "-Wall", "-Wextra", "-Wpedantic", "-Werror"])
            .arg("-I")
            .arg(&include)
            .arg("-o")
            .arg(&dynamically)
            .arg(&smoke)
            .arg("-L")
            .arg(&lib)
            .arg("-lteistro_ffi"),
        "",
        "the C bundle's shared library does not link",
    )?;
    step(
        Command::new(&dynamically).env(loader_variable(platform), &lib),
        "the C bundle links dynamically and answers",
        "the C bundle's shared build did not pass",
    )
}

/// The environment variable each platform's loader reads.
fn loader_variable(platform: &Platform) -> &'static str {
    match platform.os {
        "darwin" => "DYLD_LIBRARY_PATH",
        "win32" => "PATH",
        _ => "LD_LIBRARY_PATH",
    }
}

/// Unpacks a `.tar.gz` in process, so that the gate needs no `tar` on the
/// machine and behaves the same on every platform.
fn unpack(archive: &Path, into: &Path) -> std::io::Result<()> {
    fs::create_dir_all(into)?;
    let file = fs::File::open(archive)?;
    tar::Archive::new(flate2::read::GzDecoder::new(file)).unpack(into)
}

// ── Node ───────────────────────────────────────────────────────────────────

/// Packs the two staged packages exactly as `npm publish` would, installs
/// them into an empty project, and runs the consumer there.
fn node_consumer(root: &Path, dist: &Path, check: &Path, platform: &Platform) -> Result<(), ()> {
    if !present("npm", "--version") {
        println!("skip  the Node packages: no `npm` on this machine");
        return Ok(());
    }
    let into = check.join("node");
    let tarballs = into.join("tarballs");
    fs::create_dir_all(&tarballs).map_err(|err| println!("FAIL  {CHECK}/node: {err}"))?;

    let staged = dist.join("npm");
    for package in [NPM_SCOPE.to_string(), platform.npm_package()] {
        step(
            Command::new("npm")
                .args(["pack", "--silent", "--pack-destination"])
                .arg(&tarballs)
                .arg(staged.join(&package))
                .current_dir(root),
            "",
            &format!("{package} did not pack"),
        )?;
    }
    let packed: Vec<PathBuf> = fs::read_dir(&tarballs)
        .map_err(|err| println!("FAIL  {CHECK}/node/tarballs: {err}"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "tgz"))
        .collect();
    if packed.len() != 2 {
        println!("FAIL  npm packed {} tarball(s), not two", packed.len());
        return Err(());
    }

    write(
        &into.join("package.json"),
        "{\n  \"name\": \"teistro-packaging-check\",\n  \"private\": true,\n  \"type\": \"module\"\n}\n",
    )?;
    step(
        Command::new("npm")
            .args(["install", "--silent", "--no-audit", "--no-fund"])
            .args(&packed)
            .current_dir(&into),
        "",
        "the Node packages did not install",
    )?;

    // Copied rather than run where it lives: a script inside the
    // repository would resolve `@teistro/sdk` to the repository itself,
    // which is the one thing this gate is not testing.
    let consumer = into.join("consumer.mjs");
    fs::copy(root.join("bindings/node/packaging/consumer.mjs"), &consumer)
        .map_err(|err| println!("FAIL  the Node consumer did not copy: {err}"))?;
    step(
        Command::new("node")
            .arg(&consumer)
            .current_dir(&into)
            .env_remove("TEISTRO_ADDON"),
        "the installed Node package answers as the library does",
        "the installed Node package did not answer",
    )
}

// ── Dart ───────────────────────────────────────────────────────────────────

/// Builds a project that depends on the staged package, installs the
/// library from the archive the release would publish, and runs the
/// consumer with nothing in the environment to help it.
fn dart_consumer(
    root: &Path,
    dist: &Path,
    check: &Path,
    platform: &Platform,
    version: &str,
) -> Result<(), ()> {
    if !present("dart", "--version") {
        println!("skip  the Dart package: no `dart` on this machine");
        return Ok(());
    }
    let into = check.join("dart");
    fs::create_dir_all(&into).map_err(|err| println!("FAIL  {CHECK}/dart: {err}"))?;
    let staged = dist.join("pub/teistro");
    let sdk = read(&root.join("bindings/dart/pubspec.yaml"))
        .lines()
        .find_map(|line| line.trim().strip_prefix("sdk: ").map(str::to_string))
        .unwrap_or_else(|| String::from("^3.7.0"));
    write(
        &into.join("pubspec.yaml"),
        &format!(
            "name: teistro_packaging_check\npublish_to: none\n\nenvironment:\n  sdk: {sdk}\n\ndependencies:\n  teistro:\n    path: {}\n",
            staged.display()
        ),
    )?;
    fs::copy(
        root.join("bindings/dart/packaging/consumer.dart"),
        into.join("consumer.dart"),
    )
    .map_err(|err| println!("FAIL  the Dart consumer did not copy: {err}"))?;

    step(
        Command::new("dart").args(["pub", "get"]).current_dir(&into),
        "",
        "the Dart package did not resolve",
    )?;

    let shared = platform.shared(LIBRARY_STEM);
    let (stem, extension) = shared.rsplit_once('.').unwrap_or((shared.as_str(), "so"));
    let archive = dist.join(format!(
        "{stem}-{version}-{}.{extension}.gz",
        platform.name()
    ));
    step(
        Command::new("dart")
            .args(["run", "teistro:install", "--from"])
            .arg(&archive)
            .current_dir(&into)
            .env_remove("TEISTRO_LIBRARY"),
        "",
        "the Dart installer did not install the library it was given",
    )?;
    let installed = into.join(format!(".dart_tool/teistro/{version}/{shared}"));
    if !installed.is_file() {
        println!(
            "FAIL  the Dart installer wrote no {}",
            rel(root, &installed)
        );
        return Err(());
    }

    step(
        Command::new("dart")
            .args(["run", "consumer.dart"])
            .current_dir(&into)
            .env_remove("TEISTRO_LIBRARY"),
        "the installed Dart package answers as the library does",
        "the installed Dart package did not answer",
    )
}

/// Writes a file, reporting where it could not.
fn write(path: &Path, text: &str) -> Result<(), ()> {
    fs::create_dir_all(path.parent().unwrap_or(path))
        .and_then(|()| fs::write(path, text))
        .map_err(|err| println!("FAIL  cannot write {}: {err}", path.display()))
}
