//! `cargo xtask check-adapter [--adapter DIR]`: an ephemeris adapter built
//! and held to what every provider is held to (`05-testing/01-quality-bar.md`,
//! "provider conformance kit"), and loaded as a plugin by every binding.
//!
//! The SDK is engine-agnostic, so this gate names no engine. It takes the
//! adapter as a directory laid out by language, as the engine's own
//! repository keeps it (`docs/06-cicd/adapter-gate.md`):
//!
//! ```text
//! DIR/rust     the port over the engine: a standalone Cargo crate with a
//!              `cdylib`, its kit binary `<package>-kit` and a committed lock
//! DIR/node     the npm package           DIR/python  the Python package
//! DIR/dart     the Dart package          DIR/flutter the Flutter package
//! DIR/java     the Java module
//! ```
//!
//! What a gate needs to know about the engine that the layout cannot say
//! is listed in the crate's manifest, under
//! `[package.metadata.teistro-adapter]`, and read from there rather than
//! guessed ([`Described`]).
//!
//! The adapter is named by `--adapter DIR`, else by `TEISTRO_ADAPTER_DIR`;
//! the SDK holds none of its own. Every other gate that reads an
//! adapter's packages (`check-node`, `check-python`, `check-dart`,
//! `check-java`, `check-package`) finds it the same way, by [`Adapter::find`].
//!
//! Every Cargo run in the adapter passes `--locked`, so its `Cargo.lock`,
//! which is committed, is held to what the SDK's crates now ask for: a
//! crate gaining a dependency fails here instead of leaving the lock
//! rewritten in whichever tree a gate ran. `cargo xtask relock-adapter
//! [--adapter DIR]` writes it again.
//!
//! Four steps, each a failure of its own:
//!
//! 0. the lock is current;
//! 1. the adapter's tests, in release as timing tests expect;
//! 2. its kit binary: the checks under the published bounds, the report
//!    written to `target/<engine>-kit/`;
//! 3. the Node, Python, Dart and Java gates once more, with
//!    [`LIBRARY_VARIABLE`] naming the built library and [`DIR_VARIABLE`]
//!    the adapter, so each binding's plugin test loads the engine instead
//!    of skipping and each adapter package is checked against the SDK.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

use crate::binding::{cargo, step};
use crate::platform::Platform;

/// Names the adapter's directory to every gate that reads one.
pub(crate) const DIR_VARIABLE: &str = "TEISTRO_ADAPTER_DIR";
/// Names the adapter's built plugin library to each binding's plugin test.
pub(crate) const LIBRARY_VARIABLE: &str = "TEISTRO_ADAPTER_LIBRARY";
/// Names the engine the plugin tests expect the library to report.
pub(crate) const ENGINE_VARIABLE: &str = "TEISTRO_ADAPTER_ENGINE";

/// The binding gates whose suites load the plugin when it is named.
const BINDINGS: [&str; 4] = ["check-node", "check-python", "check-dart", "check-java"];

/// What the adapter's manifest says of itself.
#[derive(Debug, Deserialize)]
struct Manifest {
    package: Package,
}

#[derive(Debug, Deserialize)]
struct Package {
    name: String,
    #[serde(default)]
    metadata: Metadata,
}

#[derive(Debug, Default, Deserialize)]
struct Metadata {
    #[serde(rename = "teistro-adapter")]
    adapter: Option<Described>,
}

/// `[package.metadata.teistro-adapter]`: what a gate needs to know about
/// the engine that the directory's layout cannot say. Listed, not
/// inferred.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct Described {
    /// The name the engine's manifest reports (`ctx.engine.manifest.engine`)
    /// and a provider's stamp carries.
    engine: String,
    /// The variable the adapter's own packages read to find its library,
    /// set beside [`LIBRARY_VARIABLE`] so an installed package finds it.
    library_variable: Option<String>,
    /// The Java module the adapter's `java/` package declares.
    java_module: Option<String>,
    /// The engine's checkout, relative to `rust/`, whose commit a
    /// measurement records.
    engine_checkout: Option<String>,
    /// The environment a build with the engine's data compiled in needs,
    /// as a phone has it.
    #[serde(default)]
    embedded_env: BTreeMap<String, String>,
    /// The variables such a build must not inherit from a developer's shell.
    #[serde(default)]
    embedded_unset: Vec<String>,
}

/// An ephemeris adapter, laid out by language under one directory.
#[derive(Debug)]
pub(crate) struct Adapter {
    /// The directory, absolute.
    dir: PathBuf,
    /// How a message names it: relative to the repository when inside it.
    shown: String,
    /// The Rust crate's package name.
    package: String,
    described: Described,
}

impl Adapter {
    /// The adapter a gate reads: `explicit`, else [`DIR_VARIABLE`].
    /// `Ok(None)` when none is named; an adapter named and not readable is
    /// an error.
    pub(crate) fn find(root: &Path, explicit: Option<&Path>) -> Result<Option<Adapter>, String> {
        let named = explicit
            .map(Path::to_path_buf)
            .or_else(|| std::env::var_os(DIR_VARIABLE).map(PathBuf::from));
        named.map(|dir| Adapter::read(root, &dir)).transpose()
    }

    /// The adapter at `dir`, relative to the repository or absolute.
    fn read(root: &Path, dir: &Path) -> Result<Adapter, String> {
        let dir = root.join(dir);
        let shown = crate::rel(root, &dir);
        let path = dir.join("rust/Cargo.toml");
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("{shown}/rust/Cargo.toml is not an adapter's manifest: {e}"))?;
        let manifest: Manifest = toml::from_str(&text)
            .map_err(|e| format!("{shown}/rust/Cargo.toml does not parse: {e}"))?;
        let described = manifest.package.metadata.adapter.ok_or_else(|| {
            format!(
                "{shown}/rust/Cargo.toml has no `[package.metadata.teistro-adapter]`, which \
                 names the engine (`docs/06-cicd/adapter-gate.md`)"
            )
        })?;
        Ok(Adapter {
            dir,
            shown,
            package: manifest.package.name,
            described,
        })
    }

    /// An adapter described in memory, for a test that needs one but not
    /// its files.
    #[cfg(test)]
    pub(crate) fn example() -> Adapter {
        Adapter {
            dir: PathBuf::from("/nowhere/example"),
            shown: String::from("/nowhere/example"),
            package: String::from("teistro-ephemeris-example"),
            described: Described {
                engine: String::from("example"),
                library_variable: None,
                java_module: None,
                engine_checkout: None,
                embedded_env: BTreeMap::new(),
                embedded_unset: Vec::new(),
            },
        }
    }

    /// One language's package: `rust`, `node`, `python`, `dart`, `java`
    /// or `flutter`.
    pub(crate) fn part(&self, language: &str) -> PathBuf {
        self.dir.join(language)
    }

    /// How a message names one language's package.
    pub(crate) fn shown(&self, language: &str) -> String {
        format!("{}/{language}", self.shown)
    }

    /// The engine's name, as its manifest reports it.
    pub(crate) fn engine(&self) -> &str {
        &self.described.engine
    }

    /// One of the crate's binaries by what follows the package's name:
    /// `kit` is `<package>-kit`.
    pub(crate) fn binary(&self, suffix: &str) -> String {
        format!("{}-{suffix}", self.package)
    }

    /// The library's file stem, as Cargo names the `cdylib`.
    pub(crate) fn stem(&self) -> String {
        self.package.replace('-', "_")
    }

    /// The Java module the adapter's `java/` package declares.
    pub(crate) fn java_module(&self) -> Option<&str> {
        self.described.java_module.as_deref()
    }

    /// The engine's checkout, whose commit a measurement records.
    pub(crate) fn engine_checkout(&self) -> Option<PathBuf> {
        self.described
            .engine_checkout
            .as_ref()
            .map(|relative| self.part("rust").join(relative))
    }

    /// A build with the engine's data compiled in, as a phone has it.
    pub(crate) fn embedded(&self, command: &mut Command) {
        command.envs(&self.described.embedded_env);
        for name in &self.described.embedded_unset {
            command.env_remove(name);
        }
    }

    /// The variable the adapter's own packages read to find its library,
    /// set to `library`, when the adapter names one.
    pub(crate) fn own_library(&self, command: &mut Command, library: &std::ffi::OsStr) {
        if let Some(own) = &self.described.library_variable {
            command.env(own, library);
        }
    }

    /// Cargo in the adapter's crate, held to its committed lock and
    /// building into its own `target/`, where [`Adapter::library`] reads
    /// the plugin, whatever `CARGO_TARGET_DIR` a caller's shell names.
    pub(crate) fn cargo(&self) -> Command {
        let dir = self.part("rust");
        let mut command = Command::new(cargo());
        command
            .arg("--locked")
            .env("CARGO_TARGET_DIR", dir.join("target"))
            .current_dir(dir);
        command
    }

    /// The crate's path dependencies that are not there: an engine's
    /// checkout beside the SDK, say, which a machine may not have.
    fn missing(&self) -> Vec<String> {
        let rust = self.part("rust");
        let Ok(text) = std::fs::read_to_string(rust.join("Cargo.toml")) else {
            return Vec::new();
        };
        let Ok(value) = toml::from_str::<toml::Value>(&text) else {
            return Vec::new();
        };
        ["dependencies", "build-dependencies", "dev-dependencies"]
            .iter()
            .filter_map(|table| value.get(table).and_then(toml::Value::as_table))
            .flat_map(|table| table.values())
            .filter_map(|dependency| dependency.get("path").and_then(toml::Value::as_str))
            .filter(|path| !rust.join(path).join("Cargo.toml").is_file())
            .map(str::to_owned)
            .collect()
    }

    /// The adapter's plugin library, built, and its path.
    fn library(&self) -> Result<PathBuf, ()> {
        step(
            self.cargo()
                .args(["build", "--quiet", "--release", "--lib"]),
            "",
            "the adapter's plugin library did not build",
        )?;
        let path = self
            .part("rust")
            .join("target/release")
            .join(Platform::host().shared(&self.stem()));
        if path.is_file() {
            Ok(path)
        } else {
            println!(
                "FAIL  the adapter built, and {} is not there",
                path.display()
            );
            Err(())
        }
    }
}

/// The adapter whose packages a binding gate checks against the SDK, if
/// one is named: `Ok(None)`, said once, when there is
/// none, since a checkout of the SDK alone has no adapter to check, and a
/// failure when one is named and cannot be read.
pub(crate) fn for_packages(root: &Path, language: &str) -> Result<Option<Adapter>, ()> {
    match Adapter::find(root, None) {
        Ok(Some(adapter)) if adapter.part(language).is_dir() => Ok(Some(adapter)),
        Ok(_) => {
            println!("      no adapter's {language} package named by {DIR_VARIABLE}: none checked");
            Ok(None)
        }
        Err(why) => {
            println!("FAIL  {why}");
            Err(())
        }
    }
}

/// `--adapter DIR` from a command's arguments, or an error naming what
/// else it was given.
fn named(args: &[String]) -> Result<Option<PathBuf>, String> {
    match args {
        [] => Ok(None),
        [flag, dir] if flag == "--adapter" => Ok(Some(PathBuf::from(dir))),
        other => Err(format!(
            "expected `--adapter DIR` or nothing, not `{}`",
            other.join(" ")
        )),
    }
}

/// The adapter a command names, with its path dependencies present; `Err`
/// with the exit code otherwise.
fn ready(root: &Path, args: &[String]) -> Result<Adapter, i32> {
    let explicit = named(args).map_err(|why| {
        println!("FAIL  {why}");
        1
    })?;
    let adapter = match Adapter::find(root, explicit.as_deref()) {
        Ok(Some(adapter)) => adapter,
        Ok(None) => {
            println!("FAIL  no adapter: name one with `--adapter DIR` or {DIR_VARIABLE}");
            return Err(1);
        }
        Err(why) => {
            println!("FAIL  {why}");
            return Err(1);
        }
    };
    let missing = adapter.missing();
    if missing.is_empty() {
        return Ok(adapter);
    }
    println!(
        "FAIL  the adapter at {}: its path dependencies {} are not there (the engine is checked \
         out from its own repository)",
        adapter.shown,
        missing.join(", ")
    );
    Err(1)
}

pub(crate) fn check(root: &Path, args: &[String]) -> i32 {
    let adapter = match ready(root, args) {
        Ok(adapter) => adapter,
        Err(code) => return code,
    };
    let engine = adapter.engine().to_owned();
    if step(
        adapter
            .cargo()
            .args(["metadata", "--quiet", "--format-version", "1"])
            .stdout(std::process::Stdio::null()),
        "the adapter's Cargo.lock is what the SDK's crates ask for",
        "the adapter's Cargo.lock is stale: run `cargo xtask relock-adapter --adapter DIR` and commit it",
    )
    .is_err()
    {
        return 1;
    }
    let mut failed = 0;
    if step(
        adapter.cargo().args(["test", "--quiet", "--release"]),
        "the adapter's own tests",
        "the adapter's tests failed",
    )
    .is_err()
    {
        failed += 1;
    }
    let report = root.join("target").join(format!("{engine}-kit"));
    if std::fs::create_dir_all(&report).is_err() {
        println!("FAIL  cannot make {}", report.display());
        return 1;
    }
    let out = report.to_string_lossy().into_owned();
    if step(
        adapter.cargo().args([
            "run",
            "--quiet",
            "--release",
            "--bin",
            &adapter.binary("kit"),
            "--",
            "--out",
            &out,
        ]),
        &format!("the provider kit over {engine}, every check inside its published bound"),
        &format!("the provider kit failed over {engine}"),
    )
    .is_err()
    {
        failed += 1;
    }
    let Ok(plugin) = adapter.library() else {
        return failed + 1;
    };
    let Ok(xtask) = std::env::current_exe() else {
        println!("FAIL  cannot name this xtask to run the binding gates");
        return failed + 1;
    };
    for gate in BINDINGS {
        let mut command = Command::new(&xtask);
        command
            .arg(gate)
            .env(DIR_VARIABLE, &adapter.dir)
            .env(LIBRARY_VARIABLE, &plugin)
            .env(ENGINE_VARIABLE, &engine)
            .current_dir(root);
        adapter.own_library(&mut command, plugin.as_os_str());
        if step(
            &mut command,
            &format!("{gate} with the {engine} plugin loaded"),
            &format!("{gate} failed with the {engine} plugin loaded"),
        )
        .is_err()
        {
            failed += 1;
        }
    }
    failed
}

/// `cargo xtask relock-adapter [--adapter DIR]`: the adapter's
/// `Cargo.lock` resolved again against the SDK's crates, keeping every
/// version it already pins.
pub(crate) fn relock(root: &Path, args: &[String]) -> i32 {
    let adapter = match ready(root, args) {
        Ok(adapter) => adapter,
        Err(0) => {
            println!("FAIL  the lock cannot be resolved without the adapter's path dependencies");
            return 1;
        }
        Err(code) => return code,
    };
    let outcome = step(
        Command::new(cargo())
            .args(["metadata", "--quiet", "--format-version", "1"])
            .stdout(std::process::Stdio::null())
            .current_dir(adapter.part("rust")),
        &format!("{}/Cargo.lock resolved", adapter.shown("rust")),
        "the adapter did not resolve",
    );
    i32::from(outcome.is_err())
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{Adapter, named};

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn an_adapter_is_named_by_flag_or_not_at_all() {
        assert_eq!(named(&args(&[])), Ok(None));
        assert_eq!(
            named(&args(&["--adapter", "../engine/teistro"])),
            Ok(Some(PathBuf::from("../engine/teistro")))
        );
        assert!(named(&args(&["--adapter"])).is_err());
        assert!(named(&args(&["somewhere"])).is_err());
    }

    /// Every name a gate used to spell comes out of the adapter's own
    /// description and its package's name, whatever the engine.
    #[test]
    fn an_adapter_describes_itself() {
        let dir = std::env::temp_dir().join(format!("teistro-adapter-{}", std::process::id()));
        let rust = dir.join("rust");
        std::fs::create_dir_all(&rust).expect("a scratch directory");
        std::fs::write(
            rust.join("Cargo.toml"),
            "[package]\nname = \"teistro-ephemeris-example\"\n\n\
             [package.metadata.teistro-adapter]\nengine = \"example\"\n\
             java-module = \"com.example.ephemeris\"\n\
             embedded-env = { EXAMPLE_TIER = \"MODERN\" }\n",
        )
        .expect("a scratch manifest");
        let found = Adapter::find(Path::new("/"), Some(&dir));
        std::fs::remove_dir_all(&dir).expect("the scratch directory removed");
        let Ok(Some(adapter)) = found else {
            panic!("the adapter does not read: {found:?}");
        };
        assert_eq!(adapter.engine(), "example");
        assert_eq!(adapter.binary("kit"), "teistro-ephemeris-example-kit");
        assert_eq!(adapter.stem(), "teistro_ephemeris_example");
        assert_eq!(adapter.java_module(), Some("com.example.ephemeris"));
        assert!(adapter.shown("node").ends_with("/node"));
        assert_eq!(adapter.missing(), Vec::<String>::new());
    }

    #[test]
    fn a_directory_without_a_manifest_is_refused() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let refused = Adapter::find(root, Some(Path::new("no-such-adapter")));
        assert!(refused.is_err_and(|why| why.contains("no-such-adapter/rust/Cargo.toml")));
    }
}
