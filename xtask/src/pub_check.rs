//! `cargo xtask check-pub [--dist DIR]`: the staged pub packages through
//! pub's own validation, uploading nothing (`dart pub publish --dry-run`).
//!
//! Each package is validated from a copy outside the repository, because
//! pub publishes only what git would track and the staging directory is
//! ignored: run in place, every package reads as "the pubspec is hidden".
//! The release's publish step copies them out the same way.
//!
//! A dry-run exits non-zero on a warning as well as an error, and the
//! release publishes with `--force`, which overrides warnings but not
//! errors; so this reads what pub found instead of its exit code. Every
//! error fails, and so does every warning not listed in [`ACCEPTED`] with
//! its reason. `teistro_flutter` is validated against the staged
//! `teistro` through `pubspec_overrides.yaml`, since the exact version it
//! depends on is not on pub.dev until the release publishes it.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::binding::present;

/// The packages, in the order the release publishes them.
const PACKAGES: [&str; 2] = ["teistro", "teistro_flutter"];

/// Warnings a package is published with, each with why.
const ACCEPTED: [(&str, &str, &str); 1] = [(
    "teistro_flutter",
    "Your dependency on \"teistro\" should allow more than one version",
    "the hook checks the native slices against one version's digest table, so no other \
     version of `teistro` will do (`03-design/flutter-build-hook.md`)",
)];

/// What pub found, by how grave it is.
#[derive(Debug, Default, PartialEq, Eq)]
struct Findings {
    errors: Vec<String>,
    warnings: Vec<String>,
    hints: Vec<String>,
}

/// Reads a dry-run's report: each `* ` item under the heading that says
/// what it is (an error, a potential issue, a hint), with its
/// continuation lines.
fn findings(report: &str) -> Findings {
    let mut found = Findings::default();
    let mut section: Option<&mut Vec<String>> = None;
    for line in report.lines() {
        if line.starts_with("Package validation found the following") {
            section = Some(if line.contains("error") {
                &mut found.errors
            } else if line.contains("potential issue") {
                &mut found.warnings
            } else {
                &mut found.hints
            });
            continue;
        }
        let Some(items) = section.as_mut() else {
            continue;
        };
        if let Some(item) = line.strip_prefix("* ") {
            items.push(item.trim().to_string());
        } else if line.starts_with("  ") && !line.trim().is_empty() {
            if let Some(last) = items.last_mut() {
                last.push(' ');
                last.push_str(line.trim());
            }
        } else if !line.trim().is_empty() {
            section = None;
        }
    }
    found
}

/// The warnings `package` may not be published with.
fn refused<'w>(package: &str, warnings: &'w [String]) -> Vec<&'w String> {
    warnings
        .iter()
        .filter(|warning| {
            !ACCEPTED
                .iter()
                .any(|(name, text, _)| *name == package && warning.starts_with(text))
        })
        .collect()
}

/// A fresh directory outside every repository for the copies.
fn scratch() -> Result<PathBuf, String> {
    let dir = std::env::temp_dir().join(format!("teistro-check-pub-{}", std::process::id()));
    if dir.exists() {
        fs::remove_dir_all(&dir).map_err(|e| format!("cannot clear {}: {e}", dir.display()))?;
    }
    fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    Ok(dir)
}

fn copy_tree(from: &Path, to: &Path) -> Result<(), String> {
    fs::create_dir_all(to).map_err(|e| format!("cannot create {}: {e}", to.display()))?;
    let entries = fs::read_dir(from).map_err(|e| format!("cannot read {}: {e}", from.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("cannot read {}: {e}", from.display()))?;
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)
                .map_err(|e| format!("cannot copy {}: {e}", entry.path().display()))?;
        }
    }
    Ok(())
}

/// One package's dry-run from its copy, and what it found.
fn dry_run(copy: &Path) -> Result<Findings, String> {
    let output = Command::new("dart")
        .args(["pub", "publish", "--dry-run"])
        .current_dir(copy)
        .output()
        .map_err(|e| format!("`dart pub publish --dry-run` did not start: {e}"))?;
    let report = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let found = findings(&report);
    if !output.status.success()
        && found.errors.is_empty()
        && found.warnings.is_empty()
        && found.hints.is_empty()
    {
        return Err(format!(
            "`dart pub publish --dry-run` failed and reported nothing it found:\n{report}"
        ));
    }
    Ok(found)
}

fn check_one(dist: &Path, scratch: &Path, package: &str) -> Result<(), String> {
    let staged = dist.join("pub").join(package);
    if !staged.join("pubspec.yaml").is_file() {
        return Err(format!(
            "{} has no staged package; run `cargo xtask package stage` first",
            staged.display()
        ));
    }
    let copy = scratch.join(package);
    copy_tree(&staged, &copy)?;
    if package != "teistro" {
        fs::write(
            copy.join("pubspec_overrides.yaml"),
            "dependency_overrides:\n  teistro:\n    path: ../teistro\n",
        )
        .map_err(|e| format!("cannot write {package}'s overrides: {e}"))?;
    }
    let found = dry_run(&copy)?;
    for hint in &found.hints {
        println!("note  {package}: {hint}");
    }
    for (name, text, why) in ACCEPTED {
        if name == package && found.warnings.iter().any(|w| w.starts_with(text)) {
            println!("note  {package} is published with \"{text}\": {why}");
        }
    }
    let refused = refused(package, &found.warnings);
    if found.errors.is_empty() && refused.is_empty() {
        println!("ok    {package} passes pub's validation, nothing uploaded");
        return Ok(());
    }
    let why: String = found
        .errors
        .iter()
        .map(|error| format!("\n      error: {error}"))
        .chain(
            refused
                .iter()
                .map(|warning| format!("\n      warning: {warning}")),
        )
        .collect();
    Err(format!("{package} would not publish as staged:{why}"))
}

pub(crate) fn check(root: &Path, args: &[String]) -> i32 {
    let dist = match args {
        [] => root.join("target/dist"),
        [flag, dir] if flag == "--dist" => PathBuf::from(dir),
        _ => {
            println!("FAIL  `check-pub` takes --dist DIR, not {}", args.join(" "));
            return 1;
        }
    };
    if !present("dart", "--version") {
        crate::skip::skip("no `dart` on this machine; `check-pub` needs it");
        return 0;
    }
    let scratch = match scratch() {
        Ok(dir) => dir,
        Err(why) => {
            println!("FAIL  {why}");
            return 1;
        }
    };
    let mut failed = 0;
    for package in PACKAGES {
        if let Err(why) = check_one(&dist, &scratch, package) {
            println!("FAIL  {why}");
            failed += 1;
        }
    }
    let _ = fs::remove_dir_all(&scratch);
    i32::from(failed > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A dry-run's report as pub 3.10 writes it.
    const REPORT: &str = "\
Validating package...
Package validation found the following error:
* You must have a LICENSE file in the root directory.
  An open-source license helps ensure people can legally use your code.

Package validation found the following 2 potential issues:
* Your dependency on \"teistro\" should allow more than one version. For example:

  dependencies:
    teistro: ^0.0.0

* Please add a `CHANGELOG.md` to your package.

Package validation found the following hint:
* Non-dev dependencies are overridden in pubspec_overrides.yaml.
The server may enforce additional checks.

Package has 2 warnings and 1 hint.
";

    #[test]
    fn a_report_is_read_by_its_headings() {
        let found = findings(REPORT);
        assert_eq!(found.errors.len(), 1);
        assert!(found.errors[0].starts_with("You must have a LICENSE file"));
        assert!(found.errors[0].ends_with("use your code."));
        assert_eq!(found.warnings.len(), 2);
        assert_eq!(found.hints.len(), 1);
    }

    #[test]
    fn only_a_listed_warning_is_accepted_and_only_for_its_package() {
        let found = findings(REPORT);
        let flutter = refused("teistro_flutter", &found.warnings);
        assert_eq!(flutter.len(), 1);
        assert!(flutter[0].starts_with("Please add a `CHANGELOG.md`"));
        assert_eq!(refused("teistro", &found.warnings).len(), 2);
    }

    #[test]
    fn a_clean_report_finds_nothing() {
        assert_eq!(
            findings("Validating package...\nPackage has 0 warnings.\n"),
            Findings::default()
        );
    }
}
