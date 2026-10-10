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

use crate::binding::{LIBRARY_STEM, StaticLink, present, step, tool};
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
        python_consumer(root, &dist, &check, &platform, &version),
        crate::java_consumer::check(root, &dist, &check, &platform, &version),
        adapter_consumer(root, &dist, &check, &platform),
        mcp_consumer(&dist, &check, &platform, &version),
    ];
    let recorded = crate::sizes::write_fragment(
        root,
        &platform.name(),
        crate::sizes::platform_rows(root, &platform, &version),
    );
    let failed =
        outcomes.iter().filter(|outcome| outcome.is_err()).count() + usize::from(recorded.is_err());
    println!(
        "{} packages installed and run for {}: {failed} failure(s)",
        outcomes.len(),
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
    let cc = crate::binding::c_compiler();
    if !present(&cc, "--version") {
        crate::skip::skip(format_args!("the C bundle: no `{cc}` on this machine"));
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
    for expected in [
        "include/teistro.h",
        "LICENSE",
        "NOTICE",
        "README.md",
        crate::sbom::FILE,
    ] {
        if !bundle.join(expected).is_file() {
            println!("FAIL  the C bundle has no {expected}");
            return Err(());
        }
    }

    let smoke = root.join("bindings/c/tests/smoke.c");
    match crate::binding::static_link(platform) {
        StaticLink::With(beside) => {
            let statically = crate::binding::executable(&into, "smoke-static");
            step(
                Command::new(&cc)
                    .args(["-std=c11", "-Wall", "-Wextra", "-Wpedantic", "-Werror"])
                    .arg("-I")
                    .arg(&include)
                    .arg("-o")
                    .arg(&statically)
                    .arg(&smoke)
                    .arg(lib.join(platform.static_library(LIBRARY_STEM)))
                    .args(beside),
                "",
                "the C bundle's static library does not link",
            )?;
            step(
                &mut Command::new(&statically),
                "the C bundle links statically and answers",
                "the C bundle's static build did not pass",
            )?;
        }
        // Said, not skipped in silence: the bundle still ships the
        // static library and a consumer with the matching compiler
        // links it, but this gate's `cc` is not that compiler.
        StaticLink::Refused(why) => {
            crate::skip::excused(
                format_args!("the C bundle's static library, with `{cc}`: {why}"),
                "the gap `06-cicd/02-build-matrix.md` admits",
            );
        }
    }

    let dynamically = crate::binding::executable(&into, "smoke-shared");
    step(
        Command::new(&cc)
            .args(["-std=c11", "-Wall", "-Wextra", "-Wpedantic", "-Werror"])
            .arg("-I")
            .arg(&include)
            .arg("-o")
            .arg(&dynamically)
            .arg(&smoke)
            .args(crate::binding::shared_link(platform, &lib)),
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

// ── the agent server ───────────────────────────────────────────────────────

/// Unpacks the agent server's archive and runs the program from there, as
/// a host configured with its path would: its version, then one session
/// under each revision it speaks, each answer held to its shape. A musl
/// row runs it inside Alpine, which proves the static build.
fn mcp_consumer(dist: &Path, check: &Path, platform: &Platform, version: &str) -> Result<(), ()> {
    let into = check.join("mcp");
    unpack(
        &dist.join(package::mcp_archive_name(version, &platform.name())),
        &into,
    )
    .map_err(|err| println!("FAIL  the agent server's archive did not unpack: {err}"))?;
    let unpacked = into.join("teistro-mcp");
    let program = unpacked.join(platform.program("teistro-mcp"));
    for expected in ["README.md", "LICENSE", "NOTICE", crate::sbom::FILE] {
        if !unpacked.join(expected).is_file() {
            println!("FAIL  the agent server's archive carries no {expected}");
            return Err(());
        }
    }
    let shown = Command::new(&program)
        .arg("--version")
        .output()
        .map_err(|err| println!("FAIL  {} did not run: {err}", program.display()))?;
    let shown = String::from_utf8_lossy(&shown.stdout).into_owned();
    for said in [
        format!("teistro-mcp {version}"),
        String::from("2026-07-28"),
        String::from("2025-11-25"),
    ] {
        if !shown.contains(&said) {
            println!("FAIL  `teistro-mcp --version` does not say {said}: {shown}");
            return Err(());
        }
    }
    let meta = serde_json::json!({ "io.modelcontextprotocol/protocolVersion": "2026-07-28",
                                   "io.modelcontextprotocol/clientCapabilities": {} });
    let convert = serde_json::json!({ "name": "time.convert",
        "arguments": { "request": { "jd": 2_451_545.0, "from": "UTC", "to": "TT" } } });
    let mut call = convert.clone();
    call["_meta"] = meta.clone();
    let modern = session(
        &program,
        &[
            serde_json::json!({ "jsonrpc": "2.0", "id": 1, "method": "server/discover", "params": { "_meta": meta } }),
            serde_json::json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": { "_meta": meta } }),
            serde_json::json!({ "jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": call }),
        ],
    )?;
    let legacy = session(
        &program,
        &[
            serde_json::json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
                "protocolVersion": "2025-11-25", "capabilities": {},
                "clientInfo": { "name": "check-package", "version": version } } }),
            serde_json::json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }),
            serde_json::json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": convert }),
        ],
    )?;
    let listed = modern
        .get(1)
        .and_then(|reply| reply["result"]["tools"].as_array())
        .is_some_and(|tools| tools.iter().any(|tool| tool["name"] == "time.convert"));
    let held = [
        (
            "discovery names the revision",
            modern.first().is_some_and(|r| r["result"].is_object()),
        ),
        ("the list names `time.convert`", listed),
        (
            "a modern call answers",
            modern
                .get(2)
                .is_some_and(|r| r["result"]["isError"] == false),
        ),
        (
            "the handshake agrees the revision",
            legacy
                .first()
                .is_some_and(|r| r["result"]["protocolVersion"] == "2025-11-25"),
        ),
        (
            "a legacy call answers",
            legacy
                .get(1)
                .is_some_and(|r| r["result"]["isError"] == false),
        ),
    ];
    for (what, holds) in held {
        if !holds {
            println!("FAIL  the unpacked agent server: {what} ({modern:?} {legacy:?})");
            return Err(());
        }
    }
    println!("ok    the agent server unpacked and answered under both revisions");
    Ok(())
}

/// The replies `program` writes to `messages` over stdio, in order.
fn session(program: &Path, messages: &[serde_json::Value]) -> Result<Vec<serde_json::Value>, ()> {
    use std::io::Write as _;
    let mut child = Command::new(program)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .map_err(|err| println!("FAIL  {} did not start: {err}", program.display()))?;
    if let Some(mut stdin) = child.stdin.take() {
        for message in messages {
            writeln!(stdin, "{message}")
                .map_err(|err| println!("FAIL  the agent server read nothing: {err}"))?;
        }
    }
    let output = child
        .wait_with_output()
        .map_err(|err| println!("FAIL  the agent server did not finish: {err}"))?;
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| {
            serde_json::from_str(line).map_err(|err| {
                println!("FAIL  the agent server wrote a line that is not JSON ({err}): {line}");
            })
        })
        .collect()
}

// ── Node ───────────────────────────────────────────────────────────────────

/// Packs the two staged packages exactly as `npm publish` would, installs
/// them into an empty project, and runs the consumer there.
fn node_consumer(root: &Path, dist: &Path, check: &Path, platform: &Platform) -> Result<(), ()> {
    let Some(npm) = tool("npm", "--version") else {
        crate::skip::skip("the Node packages: no `npm` on this machine");
        return Ok(());
    };
    let into = check.join("node");
    let tarballs = into.join("tarballs");
    fs::create_dir_all(&tarballs).map_err(|err| println!("FAIL  {CHECK}/node: {err}"))?;

    let staged = dist.join("npm");
    for package in [NPM_SCOPE.to_string(), platform.npm_package()] {
        step(
            Command::new(&npm)
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
        Command::new(&npm)
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

// ── The adapter ────────────────────────────────────────────────────────────

/// Where the Teimeris adapter's npm package lives, which is the package
/// itself rather than a staged copy: it ships no per-platform binary yet,
/// so there is nothing to assemble.
const ADAPTER_NODE: &str = "adapters/ephemeris-teimeris/node";

/// The environment variable naming the engine's data directory, which a
/// checkout of the SDK does not have.
const ENGINE_DATA: &str = "TEISTRO_TEIMERIS_DATA";

/// The adapter package, installed beside the SDK's and run.
///
/// This is what ADR-0029 meant by `check-package` having to cover the
/// adapters. It tests the **package**: the name, the export map, the
/// `files` list, and the generated façade being inside it — none of
/// which the adapter's own type-check can fail, because that imports
/// `../index.js` by path.
///
/// It needs two things a checkout does not have, and skips rather than
/// fails without them: the adapter's built library
/// (`TEISTRO_TEIMERIS_ADAPTER`, as every plugin test uses) and the
/// engine's data (`TEISTRO_TEIMERIS_DATA`). A skip is not a pass and
/// says which it wanted.
fn adapter_consumer(root: &Path, dist: &Path, check: &Path, platform: &Platform) -> Result<(), ()> {
    let Some(npm) = tool("npm", "--version") else {
        crate::skip::skip("the adapter package: no `npm` on this machine");
        return Ok(());
    };
    let Some(library) = std::env::var_os("TEISTRO_TEIMERIS_ADAPTER") else {
        crate::skip::excused(
            "the adapter package: set TEISTRO_TEIMERIS_ADAPTER to its library",
            "the adapter is built in its own repository",
        );
        return Ok(());
    };
    let into = check.join("adapter-node");
    let tarballs = into.join("tarballs");
    fs::create_dir_all(&tarballs).map_err(|err| println!("FAIL  {CHECK}/adapter-node: {err}"))?;

    // The SDK's own packages, which the adapter depends on, and then the
    // adapter itself. Packed from where each lives: the SDK's from the
    // staged tree the release would publish, the adapter's from its own
    // directory, which is already the package.
    let staged = dist.join("npm");
    for package in [NPM_SCOPE.to_string(), platform.npm_package()] {
        step(
            Command::new(&npm)
                .args(["pack", "--silent", "--pack-destination"])
                .arg(&tarballs)
                .arg(staged.join(&package))
                .current_dir(root),
            "",
            &format!("{package} did not pack"),
        )?;
    }
    step(
        Command::new(&npm)
            .args(["pack", "--silent", "--pack-destination"])
            .arg(&tarballs)
            .arg(root.join(ADAPTER_NODE))
            .current_dir(root),
        "",
        "the adapter package did not pack",
    )?;
    let packed: Vec<PathBuf> = fs::read_dir(&tarballs)
        .map_err(|err| println!("FAIL  {CHECK}/adapter-node/tarballs: {err}"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "tgz"))
        .collect();
    if packed.len() != 3 {
        println!("FAIL  npm packed {} tarball(s), not three", packed.len());
        return Err(());
    }

    write(
        &into.join("package.json"),
        "{\n  \"name\": \"teistro-adapter-packaging-check\",\n  \"private\": true,\n  \"type\": \"module\"\n}\n",
    )?;
    step(
        Command::new(&npm)
            .args(["install", "--silent", "--no-audit", "--no-fund"])
            .args(&packed)
            .current_dir(&into),
        "",
        "the adapter package did not install",
    )?;

    // Copied rather than run where it lives, for the reason the SDK's own
    // consumer is: a script inside the repository would resolve the
    // package to the repository itself.
    let consumer = into.join("consumer.mjs");
    fs::copy(
        root.join(ADAPTER_NODE).join("packaging/consumer.mjs"),
        &consumer,
    )
    .map_err(|err| println!("FAIL  the adapter consumer did not copy: {err}"))?;
    let mut run = Command::new("node");
    run.arg(&consumer)
        .current_dir(&into)
        .env_remove("TEISTRO_ADDON")
        .env("TEISTRO_TEIMERIS_ADAPTER", &library);
    if let Some(data) = std::env::var_os(ENGINE_DATA) {
        run.env(ENGINE_DATA, data);
    }
    step(
        &mut run,
        "the installed adapter package answers as the engine does",
        "the installed adapter package did not answer",
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
    if platform.libc == Some("musl") {
        crate::skip::excused(
            "the Dart package",
            "the Dart SDK is built for glibc only, so no Dart runs on a musl host",
        );
        return Ok(());
    }
    if !present("dart", "--version") {
        crate::skip::skip("the Dart package: no `dart` on this machine");
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

// ── Python ─────────────────────────────────────────────────────────────────

/// Installs the staged package into a throwaway virtual environment, has
/// its own installer put the library where it looks, and runs a consumer
/// that knows nothing but the published names.
fn python_consumer(
    root: &Path,
    dist: &Path,
    check: &Path,
    platform: &Platform,
    version: &str,
) -> Result<(), ()> {
    let python = crate::binding::python();
    if !present(&python, "--version") {
        crate::skip::skip(format_args!(
            "the Python package: no `{python}` on this machine"
        ));
        return Ok(());
    }
    let into = check.join("python");
    fs::create_dir_all(&into).map_err(|err| println!("FAIL  {CHECK}/python: {err}"))?;
    let staged = dist.join("pypi/teistro");

    step(
        crate::binding::python_command(&python)
            .args(["-m", "venv", ".venv"])
            .current_dir(&into),
        "",
        "the Python environment could not be created",
    )?;
    let venv = into.join(".venv").join(platform.venv_bin());
    step(
        crate::binding::python_command(venv.join("pip"))
            .args(["install", "--disable-pip-version-check", "--quiet"])
            .arg(&staged)
            .current_dir(&into),
        "",
        "the Python package did not install",
    )?;

    let shared = platform.shared(LIBRARY_STEM);
    let (stem, extension) = shared.rsplit_once('.').unwrap_or((shared.as_str(), "so"));
    let archive = dist.join(format!(
        "{stem}-{version}-{}.{extension}.gz",
        platform.name()
    ));
    step(
        crate::binding::python_command(venv.join("teistro-install"))
            .arg("--from")
            .arg(&archive)
            .current_dir(&into)
            .env_remove("TEISTRO_LIBRARY"),
        "",
        "the Python installer did not install the library it was given",
    )?;
    let installed = into.join(format!(".teistro/{version}/{shared}"));
    if !installed.is_file() {
        println!(
            "FAIL  the Python installer wrote no {}",
            rel(root, &installed)
        );
        return Err(());
    }

    run_python_consumer(root, &into, &venv, "the installed Python package")?;
    python_wheel_consumer(root, dist, check, platform, version, &python)
}

/// Installs this platform's wheel into an environment of its own and runs
/// the consumer with nothing installed beside it: the library it answers
/// with is the one the wheel carries.
fn python_wheel_consumer(
    root: &Path,
    dist: &Path,
    check: &Path,
    platform: &Platform,
    version: &str,
    python: &str,
) -> Result<(), ()> {
    let into = check.join("python-wheel");
    fs::create_dir_all(&into).map_err(|err| println!("FAIL  {CHECK}/python-wheel: {err}"))?;
    let wheel = dist.join(format!(
        "pypi/wheels/teistro-{version}-py3-none-{}.whl",
        platform.wheel_tag
    ));
    if !wheel.is_file() {
        println!(
            "FAIL  no wheel was staged for {} at {}",
            platform.name(),
            rel(root, &wheel)
        );
        return Err(());
    }
    step(
        crate::binding::python_command(python)
            .args(["-m", "venv", ".venv"])
            .current_dir(&into),
        "",
        "the Python environment for the wheel could not be created",
    )?;
    let venv = into.join(".venv").join(platform.venv_bin());
    step(
        crate::binding::python_command(venv.join("pip"))
            .args(["install", "--disable-pip-version-check", "--quiet"])
            .arg(&wheel)
            .current_dir(&into),
        "",
        "the platform wheel did not install",
    )?;
    run_python_consumer(root, &into, &venv, "the platform wheel")
}

/// Runs the Python consumer in `into` with the environment at `venv`, the
/// library found only where the package itself looks.
fn run_python_consumer(root: &Path, into: &Path, venv: &Path, what: &str) -> Result<(), ()> {
    fs::copy(
        root.join("bindings/python/packaging/consumer.py"),
        into.join("consumer.py"),
    )
    .map_err(|err| println!("FAIL  the Python consumer did not copy: {err}"))?;
    step(
        crate::binding::python_command(venv.join("python"))
            .arg("consumer.py")
            .current_dir(into)
            .env_remove("TEISTRO_LIBRARY"),
        &format!("{what} answers as the library does"),
        &format!("{what} did not answer"),
    )
}

/// Writes a file, reporting where it could not.
pub(crate) fn write(path: &Path, text: &str) -> Result<(), ()> {
    fs::create_dir_all(path.parent().unwrap_or(path))
        .and_then(|()| fs::write(path, text))
        .map_err(|err| println!("FAIL  cannot write {}: {err}", path.display()))
}
