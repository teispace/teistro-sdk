//! Coverage floors (`05-testing/01-quality-bar.md`, "coverage floors"):
//! each library crate's line coverage, held to a floor that only rises.
//!
//! - verify's `coverage` job runs every library crate's tests under
//!   `cargo llvm-cov` and hands the JSON summary to `check-coverage --from
//!   FILE`, which fails when a crate falls below its floor, when a measured
//!   crate has no floor, and when a floored crate went unmeasured;
//! - `cargo xtask coverage --from FILE` raises each crate's floor to the
//!   run's figure, rounded down to a whole percent so a run's noise does
//!   not fail the next, and never lowers one; a crate that no longer
//!   exists leaves the record;
//! - `cargo xtask check-coverage` holds the page to the record, and the
//!   record to the one on `main` where git can read it, so a floor lowered
//!   by hand fails.
//!
//! The floors are whole percents of lines. A crate is the files under its
//! `crates/<name>/src/` but its generated tables; tests, examples and
//! benches are not measured, and neither is a crate in [`NOT_MEASURED`].

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::generated::{Output, check, write};

/// The record the floors are kept in.
const RECORD: &str = "docs/05-testing/coverage.json";
/// The page rendered from it.
const PAGE: &str = "docs/05-testing/coverage-measured.md";
/// The record's schema.
const SCHEMA: &str = "teistro-coverage/1";
/// The line coverage the quality bar asks of a computation crate.
const TARGET: u8 = 90;
/// Crates under `crates/` that are not measured, each with why.
const NOT_MEASURED: [(&str, &str); 1] = [(
    "scenario",
    "the fixed workload the determinism and instruction-count gates walk, run by those \
     gates rather than by tests of its own",
)];

/// The floors, a whole percent of lines to a crate.
#[derive(Debug, Serialize, Deserialize)]
struct Record {
    schema: String,
    floors: BTreeMap<String, u8>,
}

/// A crate's lines: how many count, and how many ran.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Lines {
    count: u64,
    covered: u64,
}

impl Lines {
    /// The share that ran, a percent; a crate with no lines is all covered.
    #[allow(
        clippy::cast_precision_loss,
        reason = "line counts far below 2^52, read to a percent"
    )]
    fn percent(self) -> f64 {
        if self.count == 0 {
            100.0
        } else {
            self.covered as f64 * 100.0 / self.count as f64
        }
    }

    /// The percent rounded down to a whole one, the floor a run supports.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a percent between 0 and 100"
    )]
    fn floor(self) -> u8 {
        self.percent().floor().clamp(0.0, 100.0) as u8
    }
}

/// `coverage --from FILE` and `check-coverage [--from FILE]`.
pub(crate) fn command(root: &Path, command: &str, args: &[String]) -> i32 {
    let from = args
        .iter()
        .position(|arg| arg == "--from")
        .and_then(|at| args.get(at + 1));
    let result = match (command, from) {
        ("coverage", Some(from)) => record(root, Path::new(from)),
        ("coverage", None) => {
            eprintln!("usage: cargo xtask coverage --from FILE | check-coverage [--from FILE]");
            return 2;
        }
        (_, from) => check_coverage(root, from.map(Path::new)),
    };
    match result {
        Ok(code) => code,
        Err(why) => {
            println!("FAIL  {why}");
            1
        }
    }
}

fn read_record(root: &Path) -> Result<Record, String> {
    let text = fs::read_to_string(root.join(RECORD)).map_err(|e| format!("{RECORD}: {e}"))?;
    parse_record(&text)
}

fn parse_record(text: &str) -> Result<Record, String> {
    let record: Record = serde_json::from_str(text).map_err(|e| format!("{RECORD}: {e}"))?;
    if record.schema != SCHEMA {
        return Err(format!("{RECORD} is `{}`, not `{SCHEMA}`", record.schema));
    }
    Ok(record)
}

/// Each crate's lines in an `llvm-cov export` summary, by the directory
/// under `crates/` its file sits in.
fn measured(export: &Value) -> Result<BTreeMap<String, Lines>, String> {
    let files = export
        .pointer("/data/0/files")
        .and_then(Value::as_array)
        .ok_or("not an llvm-cov export: no data[0].files")?;
    let mut by: BTreeMap<String, Lines> = BTreeMap::new();
    for file in files {
        let name = file
            .get("filename")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .replace('\\', "/");
        let Some(name) = crate_of(&name) else {
            continue;
        };
        let lines = file.pointer("/summary/lines");
        let read = |key: &str| lines.and_then(|l| l.get(key)).and_then(Value::as_u64);
        let (Some(count), Some(covered)) = (read("count"), read("covered")) else {
            return Err(format!("{name}: a file summary without line counts"));
        };
        let entry = by.entry(name.to_owned()).or_default();
        entry.count += count;
        entry.covered += covered;
    }
    Ok(by)
}

/// The crate a source file belongs to: `<name>` of `…/crates/<name>/src/…`.
/// A file under a `generated/` directory belongs to none: it is a table its
/// generator's own check holds, and its rows of constants say nothing about
/// whether code is tested. Neither does a crate in [`NOT_MEASURED`].
fn crate_of(path: &str) -> Option<&str> {
    let (_, rest) = path.rsplit_once("/crates/")?;
    let (name, rest) = rest.split_once('/')?;
    (rest.starts_with("src/")
        && !rest.contains("/generated/")
        && !NOT_MEASURED.iter().any(|(skipped, _)| *skipped == name))
    .then_some(name)
}

fn read_export(from: &Path) -> Result<BTreeMap<String, Lines>, String> {
    let text = fs::read_to_string(from).map_err(|e| format!("{}: {e}", from.display()))?;
    let export: Value =
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", from.display()))?;
    measured(&export)
}

/// The floors a run supports: each raised to the run's whole percent,
/// none lowered, a new crate added and a crate whose directory is gone
/// dropped.
fn raised(
    old: &BTreeMap<String, u8>,
    run: &BTreeMap<String, Lines>,
    exists: impl Fn(&str) -> bool,
) -> BTreeMap<String, u8> {
    let mut floors: BTreeMap<String, u8> = old
        .iter()
        .filter(|(name, _)| exists(name))
        .map(|(name, floor)| (name.clone(), *floor))
        .collect();
    for (name, lines) in run {
        let floor = floors.entry(name.clone()).or_insert(0);
        *floor = (*floor).max(lines.floor());
    }
    floors
}

/// What a run says against the floors: a crate below its floor, a measured
/// crate with none, a floored crate unmeasured.
fn findings(floors: &BTreeMap<String, u8>, run: &BTreeMap<String, Lines>) -> Vec<String> {
    let mut found = Vec::new();
    for (name, floor) in floors {
        match run.get(name) {
            None => found.push(format!(
                "`{name}` has a floor of {floor}% but the run measured no line of it"
            )),
            Some(lines) if lines.percent() < f64::from(*floor) => found.push(format!(
                "`{name}` covers {:.2}% of its lines, under its floor of {floor}%: add the \
                 tests the change left out",
                lines.percent()
            )),
            Some(_) => {}
        }
    }
    for name in run.keys().filter(|name| !floors.contains_key(*name)) {
        found.push(format!(
            "`{name}` has no floor; record it with `cargo xtask coverage --from FILE`"
        ));
    }
    found
}

fn record(root: &Path, from: &Path) -> Result<i32, String> {
    let run = read_export(from)?;
    let old = read_record(root).map(|r| r.floors).unwrap_or_default();
    let floors = raised(&old, &run, |name| root.join("crates").join(name).is_dir());
    let record = Record {
        schema: SCHEMA.to_owned(),
        floors,
    };
    let text = serde_json::to_string_pretty(&record).map_err(|e| e.to_string())? + "\n";
    Ok(write(
        root,
        &[
            Output::new(RECORD, text),
            Output::new(PAGE, render(&record)),
        ],
    ))
}

fn check_coverage(root: &Path, from: Option<&Path>) -> Result<i32, String> {
    let record = read_record(root)?;
    let mut failures = check(
        root,
        &[Output::new(PAGE, render(&record))],
        "cargo xtask coverage --from FILE",
    );
    let main = Command::new("git")
        .args(["show", &format!("origin/main:{RECORD}")])
        .current_dir(root)
        .output()
        .ok()
        .filter(|out| out.status.success());
    match main.map(|out| parse_record(&String::from_utf8_lossy(&out.stdout))) {
        Some(Ok(main)) => {
            for (name, was) in &main.floors {
                if let Some(now) = record.floors.get(name).filter(|now| *now < was) {
                    println!("FAIL  `{name}`'s floor fell from {was}% on main to {now}%");
                    failures += 1;
                }
            }
        }
        Some(Err(why)) => {
            println!("note  main's record is unreadable ({why}), so no floor is compared with it");
        }
        None => {
            println!("note  git cannot read main's record here, so no floor is compared with it");
        }
    }
    if let Some(from) = from {
        let run = read_export(from)?;
        for finding in findings(&record.floors, &run) {
            println!("FAIL  {finding}");
            failures += 1;
        }
        if failures == 0 {
            println!("ok    {} crates at or over their floors", run.len());
        }
    }
    Ok(i32::from(failures != 0))
}

fn render(record: &Record) -> String {
    let at_target = record.floors.values().filter(|f| **f >= TARGET).count();
    let mut rows = String::new();
    for (name, floor) in &record.floors {
        let mark = if *floor >= TARGET { "yes" } else { "no" };
        let _ = writeln!(rows, "| `{name}` | {floor}% | {mark} |");
    }
    format!(
        "# Coverage floors, measured\n\n\
         Status: `generated` by `cargo xtask coverage --from FILE` from verify's `coverage` \
         job; `check-coverage` holds this page to [`coverage.json`](coverage.json), and verify \
         holds every run to the floors.\n\n\
         Each crate's floor is the share of its lines its tests run, rounded down to a whole \
         percent, from the last run recorded; a crate's generated tables are not counted, \
         since their generators' checks hold them{skipped} ([`01-quality-bar.md`](01-quality-bar.md), \
         \"coverage floors\"). A floor only rises: a run below it fails, recording raises it \
         to what the run reached, and a floor lowered by hand fails against `main`'s. The \
         quality bar asks {TARGET}% of a computation crate; {at_target} of the {total} crates \
         have reached it.\n\n\
         | crate | floor | at {TARGET}% |\n|---|---:|---|\n{rows}",
        total = record.floors.len(),
        skipped = NOT_MEASURED
            .iter()
            .fold(String::new(), |mut out, (name, why)| {
                let _ = write!(out, ", and `{name}` is not measured: {why}");
                out
            }),
    )
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    fn lines(count: u64, covered: u64) -> Lines {
        Lines { count, covered }
    }

    #[test]
    fn a_file_belongs_to_the_crate_whose_src_holds_it() {
        assert_eq!(crate_of("/w/crates/core/src/lib.rs"), Some("core"));
        assert_eq!(crate_of("C:/w/crates/rules/src/eval/x.rs"), Some("rules"));
        assert_eq!(crate_of("/w/crates/core/tests/a.rs"), None);
        assert_eq!(crate_of("/w/xtask/src/main.rs"), None);
        assert_eq!(
            crate_of("/w/crates/core/src/catalogue/generated/star.rs"),
            None
        );
        assert_eq!(crate_of("/w/crates/scenario/src/lib.rs"), None);
        let export = serde_json::json!({"data": [{"files": [
            {"filename": "/w/crates/core/src/a.rs", "summary": {"lines": {"count": 10, "covered": 9}}},
            {"filename": "/w/crates/core/src/b.rs", "summary": {"lines": {"count": 10, "covered": 10}}},
            {"filename": "/home/.cargo/registry/serde/src/lib.rs", "summary": {"lines": {"count": 5, "covered": 0}}},
        ]}]});
        let run = measured(&export).unwrap();
        assert_eq!(run.len(), 1);
        assert_eq!(run["core"], lines(20, 19));
        assert_eq!(run["core"].floor(), 95);
    }

    /// A floor rises to the run and never falls; a gone crate leaves.
    #[test]
    fn a_floor_only_rises() {
        let old: BTreeMap<String, u8> = [
            ("core".into(), 90),
            ("gone".into(), 50),
            ("rules".into(), 80),
        ]
        .into();
        let run: BTreeMap<String, Lines> = [
            ("core".into(), lines(1000, 874)),
            ("rules".into(), lines(1000, 857)),
            ("new".into(), lines(3, 2)),
        ]
        .into();
        let floors = raised(&old, &run, |name| name != "gone");
        assert_eq!(
            floors,
            [
                ("core".into(), 90),
                ("new".into(), 66),
                ("rules".into(), 85)
            ]
            .into()
        );
    }

    #[test]
    fn a_run_under_its_floor_or_beside_the_record_is_refused() {
        let floors: BTreeMap<String, u8> = [("core".into(), 90), ("rules".into(), 80)].into();
        let run: BTreeMap<String, Lines> = [
            ("core".into(), lines(1000, 899)),
            ("new".into(), lines(1, 1)),
        ]
        .into();
        let found = findings(&floors, &run).join("\n");
        assert!(found.contains("`core` covers 89.90%"), "{found}");
        assert!(found.contains("`rules` has a floor of 80%"), "{found}");
        assert!(found.contains("`new` has no floor"), "{found}");
        let run: BTreeMap<String, Lines> = [
            ("core".into(), lines(10, 9)),
            ("rules".into(), lines(10, 8)),
        ]
        .into();
        assert_eq!(findings(&floors, &run), Vec::<String>::new());
    }
}
