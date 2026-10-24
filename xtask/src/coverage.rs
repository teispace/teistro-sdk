//! Coverage floors (`05-testing/01-quality-bar.md`, "coverage floors"):
//! each library crate's line and branch coverage, each held to a floor that
//! only rises.
//!
//! - verify's `coverage` job runs every library crate's tests under
//!   `cargo llvm-cov --branch`, on the pinned nightly branch coverage needs,
//!   and hands the JSON summary to `check-coverage --from
//!   FILE`, which fails when a crate falls below either floor, when a
//!   measured crate has no floor, and when a floored crate went unmeasured,
//!   its branches included;
//! - `cargo xtask coverage --from FILE` raises each crate's floor to the
//!   run's figure, rounded down to a whole percent so a run's noise does
//!   not fail the next, and never lowers one; a crate that no longer
//!   exists leaves the record;
//! - `cargo xtask check-coverage` holds the page to the record, and the
//!   record to the one on `main` where git can read it, so a floor lowered
//!   by hand fails.
//!
//! The floors are whole percents of lines and of branches. A crate is the files under its
//! `crates/<name>/src/` but its generated tables; tests, examples and
//! benches are not measured, and neither is a crate in [`NOT_MEASURED`].

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::generated::{Output, check, on_main, write};

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

/// The floors, a whole percent of lines to a crate and a whole percent of
/// branches. A record from before branches were measured has no branch
/// floors.
#[derive(Debug, Serialize, Deserialize)]
struct Record {
    schema: String,
    floors: BTreeMap<String, u8>,
    #[serde(default)]
    branch_floors: BTreeMap<String, u8>,
}

/// What a run measured of a crate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Measured {
    lines: Counts,
    branches: Counts,
}

/// How many of a crate's lines or branches count, and how many ran.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Counts {
    count: u64,
    covered: u64,
}

impl Counts {
    fn add(&mut self, other: Counts) {
        self.count += other.count;
        self.covered += other.covered;
    }

    /// The share that ran, a percent; a crate with none is all covered.
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

/// Each crate's lines and branches in an `llvm-cov export` summary, by
/// the directory under `crates/` its file sits in.
fn measured(export: &Value) -> Result<BTreeMap<String, Measured>, String> {
    let files = export
        .pointer("/data/0/files")
        .and_then(Value::as_array)
        .ok_or("not an llvm-cov export: no data[0].files")?;
    let mut by: BTreeMap<String, Measured> = BTreeMap::new();
    for file in files {
        let name = file
            .get("filename")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .replace('\\', "/");
        let Some(name) = crate_of(&name) else {
            continue;
        };
        let counts = |kind: &str| {
            let summary = file.pointer(&format!("/summary/{kind}"));
            let read = |key: &str| summary.and_then(|s| s.get(key)).and_then(Value::as_u64);
            match (read("count"), read("covered")) {
                (Some(count), Some(covered)) => Ok(Counts { count, covered }),
                _ => Err(format!("{name}: a file summary without {kind} counts")),
            }
        };
        let (lines, branches) = (counts("lines")?, counts("branches")?);
        let entry = by.entry(name.to_owned()).or_default();
        entry.lines.add(lines);
        entry.branches.add(branches);
    }
    Ok(by)
}

/// Each crate's lines, from a run.
fn lines_of(run: &BTreeMap<String, Measured>) -> BTreeMap<String, Counts> {
    run.iter()
        .map(|(name, m)| (name.clone(), m.lines))
        .collect()
}

/// Each crate's branches, from a run that measured them: none from a run
/// without `--branch`, whose every crate counts no branch at all and would
/// otherwise read as wholly covered.
fn branches_of(run: &BTreeMap<String, Measured>) -> BTreeMap<String, Counts> {
    if run.values().all(|m| m.branches.count == 0) {
        return BTreeMap::new();
    }
    run.iter()
        .map(|(name, m)| (name.clone(), m.branches))
        .collect()
}

/// The crate a source file belongs to: `<name>` of `…/crates/<name>/src/…`.
/// A file under a `generated/` directory belongs to none: it is a table its
/// generator's own check holds, and its rows of constants say nothing about
/// whether code is tested. Neither does a crate in [`NOT_MEASURED`].
pub(crate) fn crate_of(path: &str) -> Option<&str> {
    let (_, rest) = path.rsplit_once("/crates/")?;
    let (name, rest) = rest.split_once('/')?;
    (rest.starts_with("src/")
        && !rest.contains("/generated/")
        && !NOT_MEASURED.iter().any(|(skipped, _)| *skipped == name))
    .then_some(name)
}

fn read_export(from: &Path) -> Result<BTreeMap<String, Measured>, String> {
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
    run: &BTreeMap<String, Counts>,
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

/// What a run says against one kind of floor (`lines` or `branches`): a
/// crate below its floor, a measured crate with none, a floored crate
/// unmeasured.
fn findings(
    kind: &str,
    floors: &BTreeMap<String, u8>,
    run: &BTreeMap<String, Counts>,
) -> Vec<String> {
    let mut found = Vec::new();
    for (name, floor) in floors {
        match run.get(name) {
            None => found.push(format!(
                "`{name}` has a {kind} floor of {floor}% but the run measured none of its {kind}"
            )),
            Some(counts) if counts.percent() < f64::from(*floor) => found.push(format!(
                "`{name}` covers {:.2}% of its {kind}, under its floor of {floor}%: add the \
                 tests the change left out",
                counts.percent()
            )),
            Some(_) => {}
        }
    }
    for name in run.keys().filter(|name| !floors.contains_key(*name)) {
        found.push(format!(
            "`{name}` has no {kind} floor; record it with `cargo xtask coverage --from FILE`"
        ));
    }
    found
}

fn record(root: &Path, from: &Path) -> Result<i32, String> {
    let run = read_export(from)?;
    let old = read_record(root).ok();
    let exists = |name: &str| root.join("crates").join(name).is_dir();
    let (old_lines, old_branches) =
        old.map_or_else(Default::default, |old| (old.floors, old.branch_floors));
    let record = Record {
        schema: SCHEMA.to_owned(),
        floors: raised(&old_lines, &lines_of(&run), exists),
        branch_floors: raised(&old_branches, &branches_of(&run), exists),
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

/// Each floor of `now` lower than `main`'s, as sentences.
fn lowered(kind: &str, main: &BTreeMap<String, u8>, now: &BTreeMap<String, u8>) -> Vec<String> {
    main.iter()
        .filter_map(|(name, was)| {
            now.get(name)
                .filter(|now| *now < was)
                .map(|now| format!("`{name}`'s {kind} floor fell from {was}% on main to {now}%"))
        })
        .collect()
}

fn check_coverage(root: &Path, from: Option<&Path>) -> Result<i32, String> {
    let record = read_record(root)?;
    let mut failures = check(
        root,
        &[Output::new(PAGE, render(&record))],
        "cargo xtask coverage --from FILE",
    );
    let mut fail = |finding: String| {
        println!("FAIL  {finding}");
        failures += 1;
    };
    match on_main(root, RECORD).map(|text| parse_record(&text)) {
        Some(Ok(main)) => {
            lowered("line", &main.floors, &record.floors)
                .into_iter()
                .chain(lowered(
                    "branch",
                    &main.branch_floors,
                    &record.branch_floors,
                ))
                .for_each(&mut fail);
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
        findings("lines", &record.floors, &lines_of(&run))
            .into_iter()
            .chain(findings(
                "branches",
                &record.branch_floors,
                &branches_of(&run),
            ))
            .for_each(&mut fail);
        if failures == 0 {
            println!(
                "ok    {} crates at or over their line and branch floors",
                run.len()
            );
        }
    }
    Ok(i32::from(failures != 0))
}

fn render(record: &Record) -> String {
    let at = |name: &str, floor: u8| {
        floor >= TARGET
            && record
                .branch_floors
                .get(name)
                .is_some_and(|branch| *branch >= TARGET)
    };
    let at_target = record
        .floors
        .iter()
        .filter(|(name, floor)| at(name, **floor))
        .count();
    let mut rows = String::new();
    for (name, floor) in &record.floors {
        let branch = record
            .branch_floors
            .get(name)
            .map_or_else(|| String::from("not yet measured"), |b| format!("{b}%"));
        let mark = if at(name, *floor) { "yes" } else { "no" };
        let _ = writeln!(rows, "| `{name}` | {floor}% | {branch} | {mark} |");
    }
    format!(
        "# Coverage floors, measured\n\n\
         Status: `generated` by `cargo xtask coverage --from FILE` from verify's `coverage` \
         job; `check-coverage` holds this page to [`coverage.json`](coverage.json), and verify \
         holds every run to the floors.\n\n\
         Each crate has two floors: the share of its lines its tests run and the share of its \
         branches, each rounded down to a whole percent, from the last run recorded. Branches \
         are llvm's, measured on the pinned nightly `cargo llvm-cov --branch` needs. A crate's \
         generated tables are not counted, since their generators' checks hold them{skipped} \
         ([`01-quality-bar.md`](01-quality-bar.md), \"coverage floors\"). A floor only rises: a \
         run below it fails, recording raises it to what the run reached, and a floor lowered \
         by hand fails against `main`'s. The quality bar asks {TARGET}% of lines and of \
         branches of a computation crate; {at_target} of the {total} crates have reached \
         both.\n\n\
         | crate | lines | branches | at {TARGET}% |\n|---|---:|---:|---|\n{rows}",
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

    fn lines(count: u64, covered: u64) -> Counts {
        Counts { count, covered }
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
            {"filename": "/w/crates/core/src/a.rs", "summary": {
                "lines": {"count": 10, "covered": 9}, "branches": {"count": 4, "covered": 3}}},
            {"filename": "/w/crates/core/src/b.rs", "summary": {
                "lines": {"count": 10, "covered": 10}, "branches": {"count": 0, "covered": 0}}},
            {"filename": "/home/.cargo/registry/serde/src/lib.rs", "summary": {
                "lines": {"count": 5, "covered": 0}, "branches": {"count": 2, "covered": 0}}},
        ]}]});
        let run = measured(&export).unwrap();
        assert_eq!(run.len(), 1);
        assert_eq!(run["core"].lines, lines(20, 19));
        assert_eq!(run["core"].lines.floor(), 95);
        assert_eq!(branches_of(&run)["core"], lines(4, 3));
        assert_eq!(branches_of(&run)["core"].floor(), 75);
    }

    /// A run without `--branch` counts no branch anywhere, which is no
    /// measurement of branches rather than every branch covered.
    #[test]
    fn a_run_without_branches_measures_none() {
        let run: BTreeMap<String, Measured> = [(
            "core".into(),
            Measured {
                lines: lines(10, 9),
                branches: lines(0, 0),
            },
        )]
        .into();
        assert!(branches_of(&run).is_empty());
        let floors: BTreeMap<String, u8> = [("core".into(), 80)].into();
        let found = findings("branches", &floors, &branches_of(&run)).join("\n");
        assert!(
            found.contains("`core` has a branches floor of 80% but the run measured none"),
            "{found}"
        );
        assert_eq!(raised(&floors, &branches_of(&run), |_| true), floors);
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
        let run: BTreeMap<String, Counts> = [
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
        let run: BTreeMap<String, Counts> = [
            ("core".into(), lines(1000, 899)),
            ("new".into(), lines(1, 1)),
        ]
        .into();
        let found = findings("lines", &floors, &run).join("\n");
        assert!(
            found.contains("`core` covers 89.90% of its lines"),
            "{found}"
        );
        assert!(
            found.contains("`rules` has a lines floor of 80%"),
            "{found}"
        );
        assert!(found.contains("`new` has no lines floor"), "{found}");
        let run: BTreeMap<String, Counts> = [
            ("core".into(), lines(10, 9)),
            ("rules".into(), lines(10, 8)),
        ]
        .into();
        assert_eq!(findings("lines", &floors, &run), Vec::<String>::new());
    }

    /// The page says which crates have both floors at the target, and that
    /// a crate whose branches were never measured has no branch floor yet.
    #[test]
    fn the_page_marks_a_crate_at_the_target_only_on_both_floors() {
        let record = Record {
            schema: SCHEMA.into(),
            floors: [("a".into(), 95), ("b".into(), 95), ("c".into(), 95)].into(),
            branch_floors: [("a".into(), 91), ("b".into(), 85)].into(),
        };
        let page = render(&record);
        assert!(page.contains("| `a` | 95% | 91% | yes |"), "{page}");
        assert!(page.contains("| `b` | 95% | 85% | no |"), "{page}");
        assert!(
            page.contains("| `c` | 95% | not yet measured | no |"),
            "{page}"
        );
        assert!(
            page.contains("1 of the 3 crates have reached both"),
            "{page}"
        );
        assert_eq!(
            lowered("branch", &[("a".into(), 91)].into(), &record.branch_floors),
            Vec::<String>::new()
        );
        assert_eq!(
            lowered("branch", &[("b".into(), 90)].into(), &record.branch_floors),
            ["`b`'s branch floor fell from 90% on main to 85%"]
        );
    }
}
