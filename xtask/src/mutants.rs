//! Mutation testing (`05-testing/01-quality-bar.md`, "mutation testing"):
//! each kernel crate's caught share, held to a floor that only rises, and
//! its surviving mutants, each a missing test, named so that no new one
//! slips in.
//!
//! - `.github/workflows/mutants.yml` runs `cargo mutants` on each crate in
//!   [`KERNEL`], in the shards its row asks, from the matrix `cargo xtask
//!   mutants --matrix` prints, so the table here is the only list;
//! - `cargo xtask check-mutants --from DIR` reads every shard's
//!   `outcomes.json` under `DIR/<crate>/<shard>/mutants.out/` and fails when
//!   a shard is missing, when a crate's caught share is under its floor,
//!   and when a mutant survives that the record does not name;
//! - `cargo xtask mutants --from DIR` records the crates the run holds:
//!   each floor raised to the run's whole percent and never lowered, and
//!   the survivors replaced by the run's;
//! - `cargo xtask check-mutants` alone holds the page to the record, and
//!   the floors to `main`'s.
//!
//! A mutant is caught when the tests fail or time out under it; one that
//! does not build is not counted. A survivor is named by its file, change
//! and function without its line and column, so that moving code does not
//! read as a new survivor. A mutant that cannot change what the code
//! computes is listed, with why, in `.cargo/mutants.toml`, and is never
//! generated.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::coverage::crate_of;
use crate::generated::{Output, check, on_main, write};

/// The record the floors and survivors are kept in.
const RECORD: &str = "docs/05-testing/mutants.json";
/// The page rendered from it.
const PAGE: &str = "docs/05-testing/mutants-measured.md";
/// The record's schema.
const SCHEMA: &str = "teistro-mutants/1";
/// The caught share the quality bar asks of a kernel crate.
const TARGET: u8 = 80;
/// The kernel crates (the glossary's "kernel": the udu and rashi dashas,
/// the vargas, the bala schemes and the rules), each with the shards its
/// mutants are run in, so that no shard outlasts a runner's six hours.
const KERNEL: [(&str, u32); 4] = [("dasha", 3), ("rules", 3), ("strength", 2), ("vargas", 1)];
/// The directory a run that was not sharded is kept in, beside or instead
/// of the numbered shards.
const WHOLE: &str = "whole";

/// One kernel crate as the last recorded run left it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
struct Measured {
    /// The caught share's floor, a whole percent.
    floor: u8,
    /// The surviving mutants, by [`key`], in order.
    survivors: Vec<String>,
}

/// The record: each kernel crate's floor and survivors.
#[derive(Debug, Serialize, Deserialize)]
struct Record {
    schema: String,
    crates: BTreeMap<String, Measured>,
}

/// One crate's run, its shards together.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Run {
    caught: u32,
    timeouts: Vec<String>,
    survivors: Vec<String>,
}

impl Run {
    /// The mutants that built.
    fn viable(&self) -> u32 {
        self.caught + u32::try_from(self.survivors.len()).unwrap_or(u32::MAX)
    }

    /// The share caught, a percent; a crate with no viable mutant is all
    /// caught.
    fn percent(&self) -> f64 {
        let viable = self.viable();
        if viable == 0 {
            100.0
        } else {
            f64::from(self.caught) * 100.0 / f64::from(viable)
        }
    }

    /// The percent rounded down to a whole one, the floor a run supports.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a percent between 0 and 100"
    )]
    fn floor(&self) -> u8 {
        self.percent().floor().clamp(0.0, 100.0) as u8
    }
}

/// `mutants --from DIR`, `mutants --matrix` and `check-mutants [--from DIR]`.
pub(crate) fn command(root: &Path, command: &str, args: &[String]) -> i32 {
    let from = args
        .iter()
        .position(|arg| arg == "--from")
        .and_then(|at| args.get(at + 1));
    let matrix = args.iter().any(|arg| arg == "--matrix");
    let result = match (command, from) {
        ("mutants", _) if matrix => {
            println!("{}", self::matrix());
            Ok(0)
        }
        ("mutants", Some(from)) => record(root, Path::new(from)),
        ("mutants", None) => {
            eprintln!(
                "usage: cargo xtask mutants --from DIR | mutants --matrix | check-mutants [--from DIR]"
            );
            return 2;
        }
        (_, from) => check_mutants(root, from.map(Path::new)),
    };
    match result {
        Ok(code) => code,
        Err(why) => {
            println!("FAIL  {why}");
            1
        }
    }
}

/// The workflow's matrix: a row a shard, with the package it mutates.
fn matrix() -> Value {
    let include: Vec<Value> = KERNEL
        .iter()
        .flat_map(|(name, shards)| {
            (0..*shards).map(move |shard| {
                serde_json::json!({
                    "crate": name,
                    "package": format!("teistro-{name}"),
                    "shard": shard,
                    "shards": shards,
                })
            })
        })
        .collect();
    serde_json::json!({ "include": include })
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

/// A mutant's name without its line and column: `crates/x/src/a.rs:12:5:
/// replace + with - in f` is `crates/x/src/a.rs: replace + with - in f`.
fn key(name: &str) -> String {
    let Some((head, change)) = name.split_once(": ") else {
        return name.to_owned();
    };
    let file = head
        .rsplitn(3, ':')
        .last()
        .filter(|_| head.matches(':').count() >= 2)
        .unwrap_or(head);
    format!("{file}: {change}")
}

/// What one `outcomes.json` says, added into `runs` by crate.
fn read_outcomes(outcomes: &Value, runs: &mut BTreeMap<String, Run>) -> Result<(), String> {
    let list = outcomes
        .get("outcomes")
        .and_then(Value::as_array)
        .ok_or("not cargo-mutants' outcomes: no `outcomes` list")?;
    for outcome in list {
        let Some(mutant) = outcome.pointer("/scenario/Mutant") else {
            // The baseline, which every shard runs first.
            continue;
        };
        let text = |key: &str| mutant.get(key).and_then(Value::as_str).unwrap_or_default();
        let (file, name) = (text("file").replace('\\', "/"), text("name"));
        let rooted = format!("/{file}");
        let Some(owner) = crate_of(&rooted) else {
            return Err(format!("`{name}` is in no crate's sources"));
        };
        let run = runs.entry(owner.to_owned()).or_default();
        match outcome.get("summary").and_then(Value::as_str) {
            Some("CaughtMutant") => run.caught += 1,
            Some("Timeout") => {
                run.caught += 1;
                run.timeouts.push(key(name));
            }
            Some("MissedMutant") => run.survivors.push(key(name)),
            Some("Unviable") => {}
            other => return Err(format!("`{name}` ended as {other:?}, which is no verdict")),
        }
    }
    Ok(())
}

/// Every crate a run under `from` holds, each from its numbered shards, all
/// of them, or from one whole run. A crate whose directory is absent is not
/// in the run; a shard that is absent fails it, since a shard that crashed
/// and one that never started leave the same absence.
fn read_run(from: &Path) -> Result<BTreeMap<String, Run>, String> {
    let mut runs = BTreeMap::new();
    for (name, shards) in KERNEL {
        let dir = from.join(name);
        if !dir.is_dir() {
            continue;
        }
        let outcomes = |part: &str| dir.join(part).join("mutants.out").join("outcomes.json");
        let parts: Vec<String> = if outcomes(WHOLE).is_file() {
            vec![WHOLE.to_owned()]
        } else {
            (0..shards).map(|shard| shard.to_string()).collect()
        };
        let mut crate_runs = BTreeMap::new();
        for part in &parts {
            let path = outcomes(part);
            let text = fs::read_to_string(&path).map_err(|e| {
                format!(
                    "`{name}`'s shard {part} of {shards} left no outcomes at {}: {e}",
                    path.display()
                )
            })?;
            let value: Value =
                serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
            read_outcomes(&value, &mut crate_runs)?;
        }
        let run = crate_runs.remove(name).unwrap_or_default();
        if let Some(other) = crate_runs.keys().next() {
            return Err(format!("`{name}`'s run mutated `{other}` too"));
        }
        runs.insert(name.to_owned(), run);
    }
    for run in runs.values_mut() {
        run.survivors.sort();
        run.timeouts.sort();
    }
    Ok(runs)
}

/// The record a run supports: each crate it holds at its floor raised to
/// the run's and with the run's survivors; the crates it does not hold as
/// they were; a crate no longer a kernel's dropped.
fn recorded(
    old: &BTreeMap<String, Measured>,
    run: &BTreeMap<String, Run>,
) -> BTreeMap<String, Measured> {
    let mut crates: BTreeMap<String, Measured> = old
        .iter()
        .filter(|(name, _)| KERNEL.iter().any(|(kernel, _)| kernel == name))
        .map(|(name, measured)| (name.clone(), measured.clone()))
        .collect();
    for (name, run) in run {
        let entry = crates.entry(name.clone()).or_default();
        entry.floor = entry.floor.max(run.floor());
        entry.survivors.clone_from(&run.survivors);
    }
    crates
}

/// What a run says against the record: a kernel crate unmeasured, one
/// under its floor or with no record, a survivor the record does not name.
/// Survivors are counted, so a second mutant like a named one is new too.
fn findings(record: &BTreeMap<String, Measured>, run: &BTreeMap<String, Run>) -> Vec<String> {
    let mut found = Vec::new();
    for (name, _) in KERNEL {
        let Some(run) = run.get(name) else {
            found.push(format!(
                "`{name}` is a kernel crate the run did not measure"
            ));
            continue;
        };
        let Some(measured) = record.get(name) else {
            found.push(format!(
                "`{name}` has no record; record it with `cargo xtask mutants --from DIR`"
            ));
            continue;
        };
        if run.percent() < f64::from(measured.floor) {
            found.push(format!(
                "`{name}` caught {:.2}% of its mutants, under its floor of {}%",
                run.percent(),
                measured.floor
            ));
        }
        let mut named: BTreeMap<&str, usize> = BTreeMap::new();
        for survivor in &measured.survivors {
            *named.entry(survivor).or_default() += 1;
        }
        for survivor in &run.survivors {
            match named.get_mut(survivor.as_str()) {
                Some(left) if *left > 0 => *left -= 1,
                _ => found.push(format!(
                    "`{survivor}` survived: a test should fail under it, or it goes in \
                     `.cargo/mutants.toml` with why it cannot change the answer"
                )),
            }
        }
    }
    found
}

fn record(root: &Path, from: &Path) -> Result<i32, String> {
    let run = read_run(from)?;
    if run.is_empty() {
        return Err(format!("{} holds no kernel crate's run", from.display()));
    }
    let old = read_record(root).map(|r| r.crates).unwrap_or_default();
    let record = Record {
        schema: SCHEMA.to_owned(),
        crates: recorded(&old, &run),
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

fn check_mutants(root: &Path, from: Option<&Path>) -> Result<i32, String> {
    let record = read_record(root)?;
    let mut failures = check(
        root,
        &[Output::new(PAGE, render(&record))],
        "cargo xtask mutants --from DIR",
    );
    match on_main(root, RECORD).map(|text| parse_record(&text)) {
        Some(Ok(main)) => {
            for (name, was) in &main.crates {
                if let Some(now) = record.crates.get(name).filter(|now| now.floor < was.floor) {
                    println!(
                        "FAIL  `{name}`'s floor fell from {}% on main to {}%",
                        was.floor, now.floor
                    );
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
        let run = read_run(from)?;
        for (name, run) in &run {
            for timeout in &run.timeouts {
                println!("note  `{name}`: `{timeout}` timed out, which counts as caught");
            }
        }
        for finding in findings(&record.crates, &run) {
            println!("FAIL  {finding}");
            failures += 1;
        }
        if failures == 0 {
            println!("ok    {} kernel crates at or over their floors", run.len());
        }
    }
    Ok(i32::from(failures != 0))
}

fn render(record: &Record) -> String {
    let mut rows = String::new();
    let mut lists = String::new();
    for (name, measured) in &record.crates {
        let mark = if measured.floor >= TARGET {
            "yes"
        } else {
            "no"
        };
        let _ = writeln!(
            rows,
            "| `{name}` | {}% | {mark} | {} |",
            measured.floor,
            measured.survivors.len()
        );
        if measured.survivors.is_empty() {
            continue;
        }
        let _ = writeln!(lists, "\n## `{name}`\n");
        for survivor in &measured.survivors {
            let _ = writeln!(lists, "- `{survivor}`");
        }
    }
    let at_target = record.crates.values().filter(|m| m.floor >= TARGET).count();
    format!(
        "# Mutation testing, measured\n\n\
         Status: `generated` by `cargo xtask mutants --from DIR` from the `mutants` \
         workflow's run; `check-mutants` holds this page to [`mutants.json`](mutants.json), and \
         the workflow holds every run to it.\n\n\
         Each kernel crate's floor is the share of its mutants that its tests catch, rounded \
         down to a whole percent; a mutant that does not build is not counted, and one listed \
         in `.cargo/mutants.toml`, with why it cannot change what the code computes, is never \
         made ([`01-quality-bar.md`](01-quality-bar.md), \"mutation testing\"). A floor only \
         rises. Every survivor is a missing test, and a run fails on one this page does not \
         name, so the list below only shrinks. The quality bar asks {TARGET}% of a kernel \
         crate; {at_target} of the {total} have reached it.\n\n\
         | crate | floor | at {TARGET}% | survivors |\n|---|---:|---|---:|\n{rows}{lists}",
        total = record.crates.len(),
    )
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    fn run(caught: u32, survivors: &[&str]) -> Run {
        Run {
            caught,
            timeouts: Vec::new(),
            survivors: survivors.iter().map(|s| (*s).to_owned()).collect(),
        }
    }

    #[test]
    fn a_survivor_is_named_without_its_place() {
        assert_eq!(
            key("crates/strength/src/shadbala/dig.rs:17:48: replace + with - in of"),
            "crates/strength/src/shadbala/dig.rs: replace + with - in of"
        );
        // A change whose own text holds a colon keeps it.
        assert_eq!(
            key("crates/a/src/b.rs:1:2: replace f -> Vec<u8> with vec![::std::default()]"),
            "crates/a/src/b.rs: replace f -> Vec<u8> with vec![::std::default()]"
        );
    }

    #[test]
    fn outcomes_count_by_the_crate_that_holds_each_mutant() {
        let mutant = |line: u32, summary: &str| {
            serde_json::json!({
                "scenario": {"Mutant": {
                    "file": "crates/vargas/src/change.rs",
                    "name": format!("crates/vargas/src/change.rs:{line}:5: replace a with b in f"),
                }},
                "summary": summary,
            })
        };
        let outcomes = serde_json::json!({"outcomes": [
            {"scenario": "Baseline", "summary": "Success"},
            mutant(1, "CaughtMutant"),
            mutant(2, "Timeout"),
            mutant(3, "MissedMutant"),
            mutant(4, "Unviable"),
        ]});
        let mut runs = BTreeMap::new();
        read_outcomes(&outcomes, &mut runs).unwrap();
        let vargas = &runs["vargas"];
        assert_eq!(vargas.caught, 2);
        assert_eq!(vargas.viable(), 3);
        assert_eq!(vargas.floor(), 66);
        assert_eq!(
            vargas.survivors,
            ["crates/vargas/src/change.rs: replace a with b in f"]
        );
        let odd = serde_json::json!({"outcomes": [mutant(1, "Failure")]});
        assert!(read_outcomes(&odd, &mut runs).is_err());
    }

    /// A crate's shards are read together, and a missing shard fails.
    #[test]
    fn every_shard_is_read_and_a_missing_one_fails() {
        let dir = std::env::temp_dir().join(format!("xtask-mutants-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let write_shard = |part: &str, summary: &str| {
            let out = dir.join("strength").join(part).join("mutants.out");
            fs::create_dir_all(&out).unwrap();
            let outcomes = serde_json::json!({"outcomes": [{
                "scenario": {"Mutant": {
                    "file": "crates/strength/src/a.rs",
                    "name": format!("crates/strength/src/a.rs:1:1: replace {part} in f"),
                }},
                "summary": summary,
            }]});
            fs::write(out.join("outcomes.json"), outcomes.to_string()).unwrap();
        };
        write_shard("0", "CaughtMutant");
        let missing = read_run(&dir).unwrap_err();
        assert!(missing.contains("shard 1 of 2"), "{missing}");
        write_shard("1", "MissedMutant");
        let both = read_run(&dir).unwrap();
        assert_eq!(both.keys().collect::<Vec<_>>(), ["strength"]);
        assert_eq!(both["strength"].viable(), 2);
        // A whole run stands for the shards.
        write_shard(WHOLE, "CaughtMutant");
        assert_eq!(read_run(&dir).unwrap()["strength"].survivors.len(), 0);
        let _ = fs::remove_dir_all(&dir);
    }

    /// A floor rises to the run and never falls; the survivors are the
    /// run's; an unmeasured crate keeps its record.
    #[test]
    fn a_record_follows_the_run() {
        let old: BTreeMap<String, Measured> = [
            (
                "strength".into(),
                Measured {
                    floor: 90,
                    survivors: vec!["a".into(), "b".into()],
                },
            ),
            ("dasha".into(), Measured::default()),
            ("gone".into(), Measured::default()),
        ]
        .into();
        let now: BTreeMap<String, Run> = [
            ("strength".into(), run(87, &["a"])),
            ("vargas".into(), run(9, &["c"])),
        ]
        .into();
        let crates = recorded(&old, &now);
        assert_eq!(
            crates.keys().collect::<Vec<_>>(),
            ["dasha", "strength", "vargas"]
        );
        assert_eq!(crates["strength"].floor, 98);
        assert_eq!(crates["strength"].survivors, ["a"]);
        assert_eq!(crates["vargas"].floor, 90);
        let old = crates;
        let crates = recorded(&old, &[("strength".into(), run(1, &["a", "b"]))].into());
        assert_eq!(crates["strength"].floor, 98, "a floor does not fall");
    }

    #[test]
    fn a_new_survivor_or_a_fall_or_an_unmeasured_crate_is_refused() {
        let measured = |floor, survivors: &[&str]| Measured {
            floor,
            survivors: survivors.iter().map(|s| (*s).to_owned()).collect(),
        };
        let record: BTreeMap<String, Measured> = [
            ("dasha".into(), measured(80, &[])),
            ("rules".into(), measured(80, &["r"])),
            ("strength".into(), measured(95, &["a"])),
        ]
        .into();
        let now: BTreeMap<String, Run> = [
            ("dasha".into(), run(1, &[])),
            ("rules".into(), run(9, &["r", "r"])),
            ("strength".into(), run(18, &["a", "z"])),
            ("vargas".into(), run(1, &[])),
        ]
        .into();
        let found = findings(&record, &now).join("\n");
        assert!(
            found.contains("`r` survived"),
            "a second like a named one: {found}"
        );
        assert!(found.contains("`z` survived"), "{found}");
        assert!(found.contains("`strength` caught 90.00%"), "{found}");
        assert!(found.contains("`vargas` has no record"), "{found}");
        assert_eq!(found.lines().count(), 4, "{found}");
        let fewer: BTreeMap<String, Run> = [("dasha".into(), run(1, &[]))].into();
        let found = findings(&record, &fewer).join("\n");
        assert!(
            found.contains("`rules` is a kernel crate the run did not measure"),
            "{found}"
        );
        let record: BTreeMap<String, Measured> = KERNEL
            .iter()
            .map(|(name, _)| ((*name).to_owned(), measured(50, &["s"])))
            .collect();
        let fine: BTreeMap<String, Run> = KERNEL
            .iter()
            .map(|(name, _)| ((*name).to_owned(), run(1, &[])))
            .collect();
        assert_eq!(findings(&record, &fine), Vec::<String>::new());
    }

    #[test]
    fn the_matrix_is_a_row_a_shard() {
        let matrix = matrix();
        let rows = matrix["include"].as_array().unwrap();
        let shards: u32 = KERNEL.iter().map(|(_, shards)| shards).sum();
        assert_eq!(rows.len(), usize::try_from(shards).unwrap());
        assert_eq!(rows[0]["package"], "teistro-dasha");
        assert_eq!(rows[2]["shard"], 2);
    }
}
