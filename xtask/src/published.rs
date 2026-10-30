//! `cargo xtask check-published VERSION`: every package of a release,
//! installed from its public registry into a throwaway project outside
//! the repository, and the binding's consumer run against it.
//!
//! `check-package` proves what a release *stages*; this proves what it
//! *published*: that npm, `PyPI`, pub.dev and Maven Central each serve the
//! version, that a consumer resolves it with nothing but the package's
//! name, and that what it installs answers the four facts every consumer
//! asserts. It is the release workflow's last job and runs only after a
//! publish, so it is never run on the unreleased version. A registry
//! takes a while to serve what was just published, so each install is
//! tried again for a few minutes before it fails.
//!
//! `teistro_flutter` is not installed here: what it bundles is checked
//! before the release, on the release's own slices (`check-flutter`), and
//! an install of it needs a Flutter app build.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use crate::binding::{present, step, tool};
use crate::platform::{NPM_SCOPE, Platform};
use crate::release::UNRELEASED;

/// How many times, and how far apart, an install from a registry is tried.
const ATTEMPTS: u32 = 6;
const BETWEEN: Duration = Duration::from_secs(60);

pub(crate) fn check(root: &Path, args: &[String]) -> i32 {
    let [version] = args else {
        println!(
            "FAIL  `check-published` takes the version a release published, as in `check-published 0.1.0`"
        );
        return 1;
    };
    if version == UNRELEASED || !crate::release::is_semver(version) {
        println!(
            "FAIL  {version} is not a released version; nothing is published from {UNRELEASED}"
        );
        return 1;
    }
    let platform = Platform::host();
    // Outside the repository, so that nothing resolves to its checkout.
    let into = std::env::temp_dir().join(format!("teistro-published-{}", std::process::id()));
    if into.exists() && fs::remove_dir_all(&into).is_err() {
        println!("FAIL  {} could not be emptied", into.display());
        return 1;
    }
    let outcomes = [
        node(root, &into, version),
        python(root, &into, &platform, version),
        dart(root, &into, version),
        crate::java_consumer::published(root, &into, &platform, version),
    ];
    let failed = outcomes.iter().filter(|outcome| outcome.is_err()).count();
    println!(
        "{} published packages installed and run for {}: {failed} failure(s)",
        outcomes.len(),
        platform.name()
    );
    let _ = fs::remove_dir_all(&into);
    i32::from(failed != 0)
}

/// Runs `install` until it succeeds or [`ATTEMPTS`] have failed, waiting
/// [`BETWEEN`] after each failure: a registry may not serve a version for
/// minutes after it was published.
fn retried(what: &str, install: impl FnMut() -> Result<(), String>) -> Result<(), ()> {
    tried(what, ATTEMPTS, BETWEEN, install)
}

/// [`retried`], with how many attempts and how long between them given.
fn tried(
    what: &str,
    attempts: u32,
    between: Duration,
    mut install: impl FnMut() -> Result<(), String>,
) -> Result<(), ()> {
    for attempt in 1..=attempts {
        match install() {
            Ok(()) => return Ok(()),
            Err(said) if attempt == attempts => {
                println!("FAIL  {what} did not install after {attempts} attempts:\n{said}");
            }
            Err(_) => {
                println!(
                    "note  {what} did not install (attempt {attempt} of {attempts}); trying again in {} s",
                    between.as_secs()
                );
                std::thread::sleep(between);
            }
        }
    }
    Err(())
}

/// What a command printed when it failed, or nothing when it passed.
fn ran(command: &mut Command) -> Result<(), String> {
    let out = command
        .output()
        .map_err(|err| format!("{command:?} did not start: {err}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ))
    }
}

fn project(into: &Path, name: &str) -> Result<PathBuf, ()> {
    let dir = into.join(name);
    fs::create_dir_all(&dir).map_err(|err| println!("FAIL  {}: {err}", dir.display()))?;
    Ok(dir)
}

fn copied(root: &Path, from: &str, dir: &Path) -> Result<(), ()> {
    let source = root.join(from);
    let name = source.file_name().unwrap_or_default();
    fs::copy(&source, dir.join(name))
        .map(drop)
        .map_err(|err| println!("FAIL  {from} did not copy: {err}"))
}

/// `@teistro/sdk` from npm, which brings the platform package npm chose.
fn node(root: &Path, into: &Path, version: &str) -> Result<(), ()> {
    let Some(npm) = tool("npm", "--version") else {
        crate::skip::skip("the published Node package: no `npm` on this machine");
        return Ok(());
    };
    let dir = project(into, "node")?;
    crate::consumer::write(
        &dir.join("package.json"),
        "{\"name\": \"teistro-published-check\", \"private\": true, \"type\": \"module\"}\n",
    )?;
    let package = format!("{NPM_SCOPE}@{version}");
    retried(&format!("{package} from npm"), || {
        ran(Command::new(&npm)
            .args(["install", "--no-audit", "--no-fund", "--silent", &package])
            .current_dir(&dir))
    })?;
    copied(root, "bindings/node/packaging/consumer.mjs", &dir)?;
    step(
        Command::new("node")
            .arg("consumer.mjs")
            .current_dir(&dir)
            .env_remove("TEISTRO_ADDON"),
        &format!("{package} from npm answers as the library does"),
        &format!("{package} from npm did not answer"),
    )
}

/// `teistro` from `PyPI`, which brings this platform's wheel.
fn python(root: &Path, into: &Path, platform: &Platform, version: &str) -> Result<(), ()> {
    let python = crate::binding::python();
    if !present(&python, "--version") {
        crate::skip::skip(format_args!(
            "the published Python package: no `{python}` on this machine"
        ));
        return Ok(());
    }
    let dir = project(into, "python")?;
    step(
        crate::binding::python_command(&python)
            .args(["-m", "venv", ".venv"])
            .current_dir(&dir),
        "",
        "the Python environment could not be created",
    )?;
    let venv = dir.join(".venv").join(platform.venv_bin());
    // PyPI holds the release under Python's spelling of its version.
    let spelled = crate::release::python_version(version).unwrap_or_else(|| version.to_owned());
    let wanted = format!("teistro=={spelled}");
    retried(&format!("{wanted} from PyPI"), || {
        ran(crate::binding::python_command(venv.join("pip"))
            .args([
                "install",
                "--disable-pip-version-check",
                "--quiet",
                "--no-cache-dir",
                &wanted,
            ])
            .current_dir(&dir))
    })?;
    copied(root, "bindings/python/packaging/consumer.py", &dir)?;
    step(
        crate::binding::python_command(venv.join("python"))
            .arg("consumer.py")
            .current_dir(&dir)
            .env_remove("TEISTRO_LIBRARY"),
        &format!("{wanted} from PyPI answers as the library does"),
        &format!("{wanted} from PyPI did not answer"),
    )
}

/// `teistro` from pub.dev, and the library its installer fetches from the
/// release, checked against the digests the package carries.
fn dart(root: &Path, into: &Path, version: &str) -> Result<(), ()> {
    if !present("dart", "--version") {
        crate::skip::skip("the published Dart package: no `dart` on this machine");
        return Ok(());
    }
    let dir = project(into, "dart")?;
    crate::consumer::write(
        &dir.join("pubspec.yaml"),
        &format!(
            "name: teistro_published_check\npublish_to: none\n\nenvironment:\n  sdk: ^3.7.0\n\ndependencies:\n  teistro: {version}\n"
        ),
    )?;
    retried(&format!("teistro {version} from pub.dev"), || {
        ran(Command::new("dart").args(["pub", "get"]).current_dir(&dir))
    })?;
    step(
        Command::new("dart")
            .args(["run", "teistro:install"])
            .current_dir(&dir)
            .env_remove("TEISTRO_LIBRARY"),
        "",
        "the published Dart package's installer did not fetch the release's library",
    )?;
    for file in ["consumer.dart", "facts.dart"] {
        copied(root, &format!("bindings/dart/packaging/{file}"), &dir)?;
    }
    step(
        Command::new("dart")
            .args(["run", "consumer.dart"])
            .current_dir(&dir)
            .env_remove("TEISTRO_LIBRARY"),
        &format!("teistro {version} from pub.dev answers as the library does"),
        &format!("teistro {version} from pub.dev did not answer"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_is_checked_at_the_unreleased_version() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        assert_eq!(check(&root, &[UNRELEASED.to_owned()]), 1);
        assert_eq!(check(&root, &["v1.0".to_owned()]), 1);
        assert_eq!(check(&root, &[]), 1);
    }

    #[test]
    fn an_install_is_tried_until_it_passes_and_no_further() {
        let mut tries = 0;
        let late = tried("a package", 3, Duration::ZERO, || {
            tries += 1;
            if tries < 3 {
                Err("not yet".into())
            } else {
                Ok(())
            }
        });
        assert_eq!((late, tries), (Ok(()), 3));
        let mut tries = 0;
        let never = tried("a package", 3, Duration::ZERO, || {
            tries += 1;
            Err("never".into())
        });
        assert_eq!((never, tries), (Err(()), 3));
    }
}
