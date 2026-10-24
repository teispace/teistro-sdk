//! `docs/05-testing/micro-measured.md`: the kernels held to the performance
//! page's budgets (`03-design/micro-budgets.md`), rendered from the records
//! `cargo xtask measure-micro` writes, one file per machine in
//! `docs/05-testing/micro/`, and the gate on them.
//!
//! The figures are the crates' own `criterion` benches, read back from the
//! samples `criterion` saves, never from what it prints. [`BUDGETS`] is the
//! one list of what is held and to how much; the performance page's table
//! is its prose, and the gate fails when the two disagree.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::binding::cargo;
use crate::generated::{Output, check, write};
use crate::measured::{Figure, us, version};

const PAGE: &str = "docs/05-testing/micro-measured.md";
const RECORDED: &str = "docs/05-testing/micro";
const PERFORMANCE: &str = "docs/02-architecture/09-performance-architecture.md";
/// Where `criterion` writes while measuring, emptied first, so a sample an
/// earlier run left is never read as this one's.
const CRITERION_HOME: &str = "target/micro/criterion";

/// The gate fails a median past this many times its budget, as
/// `check-app-speed` does: wall time on a busy machine is noisy, and the
/// instruction-count gate is what holds a small regression.
const CEILING: f64 = 2.0;

/// The performance page's "900 rules under 2 ms", as the rate it is.
const RULE_US: f64 = 2000.0 / 900.0;

/// How much time an operation is allowed.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Allowance {
    /// Microseconds for each iteration.
    Each(f64),
    /// Microseconds for each element the bench says one iteration holds.
    PerElement(f64),
}

/// One budgeted operation.
#[derive(Clone, Copy, Debug)]
struct Budget {
    key: &'static str,
    package: &'static str,
    bench: &'static str,
    /// The `criterion` id, matched against `full_id`.
    id: &'static str,
    allowance: Allowance,
    /// The operation's row in the performance page's table, by its first
    /// cell, and the budget that row must state.
    row: &'static str,
    states: &'static str,
}

const DASHA_ROW: &str = "`dasha_at(instant, depth 5)`";
const VARGA_ROW: &str = "one varga chart; all standard vargas";

/// What is held, and to how much.
const BUDGETS: [Budget; 8] = [
    dasha("dasha-at-vimshottari", "vimshottari: at(t, 5), per instant"),
    dasha("dasha-at-ashtottari", "ashtottari: at(t, 5), per instant"),
    dasha(
        "dasha-at-registered",
        "a registered row: at(t, 5), per instant",
    ),
    dasha("dasha-at-chara", "chara: at(t, 5), per instant"),
    dasha("dasha-at-kalachakra", "kalachakra: at(t, 5), per instant"),
    Budget {
        key: "varga-one",
        package: "teistro-vargas",
        bench: "vargas",
        id: "varga: D9 chart",
        allowance: Allowance::Each(5.0),
        row: VARGA_ROW,
        states: "5 µs",
    },
    Budget {
        key: "varga-all",
        package: "teistro-vargas",
        bench: "vargas",
        id: "varga: every catalogued chart",
        allowance: Allowance::Each(100.0),
        row: VARGA_ROW,
        states: "100 µs",
    },
    Budget {
        key: "rules",
        package: "teistro-rules",
        bench: "rules",
        id: "rules/the shipped set over one chart",
        allowance: Allowance::PerElement(RULE_US),
        row: "900 rules",
        states: "2 ms",
    },
];

const fn dasha(key: &'static str, id: &'static str) -> Budget {
    Budget {
        key,
        package: "teistro-dasha",
        bench: "dasha",
        id,
        allowance: Allowance::Each(20.0),
        row: DASHA_ROW,
        states: "20 µs",
    }
}

impl Budget {
    /// The budget in microseconds, given the elements an iteration held.
    fn allowed(&self, elements: Option<u64>) -> Option<f64> {
        match self.allowance {
            Allowance::Each(us) => Some(us),
            #[expect(
                clippy::cast_precision_loss,
                reason = "a count of rules, far below 2^52"
            )]
            Allowance::PerElement(us) => elements.map(|n| us * n as f64),
        }
    }
}

/// One operation's figure, in microseconds an iteration.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
struct Measured {
    key: String,
    /// The samples `criterion` took, each one run.
    samples: usize,
    median_us: f64,
    min_us: f64,
    max_us: f64,
    /// The elements one iteration held, when the bench says.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    elements: Option<u64>,
}

impl Measured {
    fn figure(&self) -> Figure {
        Figure {
            runs: self.samples,
            median: self.median_us,
            min: self.min_us,
            max: self.max_us,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    os: String,
    arch: String,
    commit: String,
    rustc: String,
    figures: Vec<Measured>,
}

impl Record {
    fn figure(&self, key: &str) -> Option<&Measured> {
        self.figures.iter().find(|m| m.key == key)
    }
}

/// What `criterion` saves beside a bench's samples: only what is read.
#[derive(Deserialize)]
struct Saved {
    full_id: String,
    /// `{"Elements": n}` when the bench counts elements; another unit, or
    /// none, is no count.
    #[serde(default)]
    throughput: Option<serde_json::Value>,
}

impl Saved {
    fn elements(&self) -> Option<u64> {
        self.throughput.as_ref()?.get("Elements")?.as_u64()
    }
}

/// A bench's samples: `times[i]` nanoseconds for `iters[i]` iterations.
#[derive(Deserialize)]
struct Samples {
    iters: Vec<f64>,
    times: Vec<f64>,
}

/// The figure `samples` make for `key`, each sample a run.
fn measured(key: &str, samples: &Samples, elements: Option<u64>) -> Result<Measured, String> {
    if samples.iters.len() != samples.times.len() || samples.iters.is_empty() {
        return Err(format!(
            "{key}: {} sample counts against {} times",
            samples.iters.len(),
            samples.times.len()
        ));
    }
    let mut each: Vec<f64> = samples
        .iters
        .iter()
        .zip(&samples.times)
        .map(|(iters, nanos)| nanos / iters / 1e3)
        .collect();
    each.sort_by(f64::total_cmp);
    let middle = each.len() / 2;
    let median = if each.len() % 2 == 1 {
        each[middle]
    } else {
        f64::midpoint(each[middle - 1], each[middle])
    };
    Ok(Measured {
        key: key.to_string(),
        samples: each.len(),
        median_us: median,
        min_us: each[0],
        max_us: each[each.len() - 1],
        elements,
    })
}

/// Every `new/benchmark.json` under `dir`, with the directory it is in.
fn saved(dir: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            saved(&path, found);
        } else if path.file_name().is_some_and(|n| n == "benchmark.json")
            && path
                .parent()
                .and_then(Path::file_name)
                .is_some_and(|n| n == "new")
        {
            found.push(path);
        }
    }
}

/// The figure of every budget `criterion` wrote under `home`, matched by
/// its full id. An id written twice, or a budget never written, fails.
fn read_back(home: &Path) -> Result<Vec<Measured>, String> {
    let mut paths = Vec::new();
    saved(home, &mut paths);
    let mut by_id: Vec<(Saved, PathBuf)> = Vec::new();
    for path in paths {
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let id: Saved =
            serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        by_id.push((id, path.with_file_name("sample.json")));
    }
    BUDGETS
        .iter()
        .map(|budget| {
            let mut matching = by_id.iter().filter(|(id, _)| id.full_id == budget.id);
            let (id, samples) = matching
                .next()
                .ok_or_else(|| format!("{}: `criterion` wrote no `{}`", budget.key, budget.id))?;
            if matching.next().is_some() {
                return Err(format!("{}: `{}` was written twice", budget.key, budget.id));
            }
            let text = std::fs::read_to_string(samples)
                .map_err(|e| format!("{}: {e}", samples.display()))?;
            let samples: Samples =
                serde_json::from_str(&text).map_err(|e| format!("{}: {e}", samples.display()))?;
            measured(budget.key, &samples, id.elements())
        })
        .collect()
}

/// A regular expression matching exactly the ids in `ids`.
fn filter(ids: &[&str]) -> String {
    let escaped: Vec<String> = ids
        .iter()
        .map(|id| {
            id.chars().fold(String::new(), |mut out, c| {
                if "\\^$.|?*+()[]{}".contains(c) {
                    out.push('\\');
                }
                out.push(c);
                out
            })
        })
        .collect();
    format!("^({})$", escaped.join("|"))
}

/// Why each figure or record is no claim, as sentences.
fn faults(records: &[(String, Record)]) -> Vec<String> {
    let mut found = Vec::new();
    for (name, record) in records {
        found.extend(crate::measured::dirty(name, &record.commit));
        for budget in &BUDGETS {
            let Some(m) = record.figure(budget.key) else {
                found.push(format!(
                    "{name}: no `{}` figure; re-record with `cargo xtask measure-micro`",
                    budget.key
                ));
                continue;
            };
            if let Some(fault) = m.figure().fault() {
                found.push(format!("{name}: `{}`, in µs, is {fault}", budget.key));
            }
            match budget.allowed(m.elements) {
                None => found.push(format!(
                    "{name}: `{}` has no element count, and its budget is a rate",
                    budget.key
                )),
                Some(allowed) if m.median_us > CEILING * allowed => found.push(format!(
                    "{name}: `{}` takes {} µs, past {CEILING}× its budget of {} µs",
                    budget.key,
                    us(m.median_us),
                    us(allowed)
                )),
                Some(_) => {}
            }
        }
    }
    found
}

/// Where the performance page's table and [`BUDGETS`] disagree.
fn page_disagrees(performance: &str) -> Vec<String> {
    BUDGETS
        .iter()
        .filter_map(|budget| {
            let prefix = format!("| {} |", budget.row);
            match performance.lines().find(|line| line.starts_with(&prefix)) {
                None => Some(format!(
                    "{PERFORMANCE} has no row `{}`, which `{}` is held to",
                    budget.row, budget.key
                )),
                Some(line) if !line.contains(budget.states) => Some(format!(
                    "{PERFORMANCE}'s row `{}` does not state `{}`, `{}`'s budget in {}",
                    budget.row,
                    budget.states,
                    budget.key,
                    file!()
                )),
                Some(_) => None,
            }
        })
        .collect()
}

fn section(name: &str, record: &Record) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "## `{name}`\n\n{} {}, commit `{}`; {}.\n",
        record.os, record.arch, record.commit, record.rustc
    );
    let _ = writeln!(
        out,
        "| operation | bench | budget | median | of its budget | samples | fastest | slowest |"
    );
    let _ = writeln!(out, "|---|---|---:|---:|---:|---:|---:|---:|");
    for budget in &BUDGETS {
        let Some(m) = record.figure(budget.key) else {
            let _ = writeln!(
                out,
                "| `{}` | `{}` | | not recorded | | | | |",
                budget.key, budget.id
            );
            continue;
        };
        let allowed = budget.allowed(m.elements);
        let share = allowed.map_or_else(
            || String::from("—"),
            |a| format!("{:.0}%", 100.0 * m.median_us / a),
        );
        let named = match (budget.allowance, m.elements) {
            (Allowance::PerElement(rate), Some(n)) => format!(
                "{} ({n} × {})",
                allowed.map_or_else(String::new, us),
                us(rate)
            ),
            _ => allowed.map_or_else(|| String::from("—"), us),
        };
        let _ = writeln!(
            out,
            "| `{}` | `{}` | {named} | {} | {share} | {} | {} | {} |",
            budget.key,
            budget.id,
            us(m.median_us),
            m.samples,
            us(m.min_us),
            us(m.max_us)
        );
    }
    out
}

fn page(records: &[(String, Record)]) -> String {
    let mut out = String::from("# The kernels against their budgets\n\n");
    let _ = writeln!(
        out,
        "Status: `generated` by `cargo xtask micro` from `{RECORDED}/*.json`, gated by \
         `check-micro`. Do not edit. Each record is written by `cargo xtask measure-micro`, which \
         runs the crates' own `criterion` benches in release, one crate at a time, and reads back \
         the samples `criterion` saved (`03-design/micro-budgets.md`).\n"
    );
    let _ = writeln!(
        out,
        "The **budget** is the performance page's (`{PERFORMANCE}`), and the gate fails a median \
         past {CEILING}× it, because wall time is noisy and a small regression is the \
         instruction-count gate's to catch. A figure is in microseconds an iteration, and each of \
         `criterion`'s samples is one run: a median from fewer than three, or outside its own \
         range, fails, and so does a record missing an operation or measured from a tree with \
         changes. The rules' budget is a rate, {} µs a rule, times the rules the bench evaluated, \
         since the shipped set grows with every chapter.\n",
        us(RULE_US)
    );
    let found = faults(records);
    if found.is_empty() {
        let _ = writeln!(out, "Every median is inside {CEILING}× its budget.\n");
    } else {
        let _ = writeln!(out, "**Not a claim:**\n");
        for line in &found {
            let _ = writeln!(out, "- {line}");
        }
        out.push('\n');
    }
    for (name, record) in records {
        out.push_str(&section(name, record));
        out.push('\n');
    }
    out.truncate(out.trim_end().len());
    out.push('\n');
    out
}

fn outputs(root: &Path) -> Result<(Vec<Output>, Vec<String>), i32> {
    let performance = std::fs::read_to_string(root.join(PERFORMANCE)).map_err(|e| {
        eprintln!("{PERFORMANCE}: {e}");
        1
    })?;
    match crate::measured::records::<Record>(root, RECORDED, "cargo xtask measure-micro") {
        Ok(records) => {
            let mut found = faults(&records);
            found.extend(page_disagrees(&performance));
            Ok((vec![Output::new(PAGE, page(&records))], found))
        }
        Err(message) => {
            eprintln!("{message}");
            Err(1)
        }
    }
}

/// Writes the page.
pub(crate) fn generate(root: &Path) -> i32 {
    match outputs(root) {
        Ok((outputs, _)) => write(root, &outputs),
        Err(code) => code,
    }
}

/// Regenerates the page in memory, fails on any difference, and fails a
/// figure past its budget, a record that lacks one, or a budget the
/// performance page states otherwise.
pub(crate) fn check_generated(root: &Path) -> i32 {
    match outputs(root) {
        Ok((outputs, found)) => {
            for line in &found {
                println!("FAIL  {line}");
            }
            let page = check(root, &outputs, "cargo xtask micro");
            if found.is_empty() { page } else { 1 }
        }
        Err(code) => code,
    }
}

/// Runs every budgeted bench, one crate after another, and writes this
/// machine's record and the page.
pub(crate) fn measure(root: &Path) -> i32 {
    match benches(root) {
        Ok(figures) => {
            let record = Record {
                os: String::from(std::env::consts::OS),
                arch: String::from(std::env::consts::ARCH),
                commit: crate::measured::commit(root),
                rustc: version("rustc", "--version"),
                figures,
            };
            match crate::measured::write_record(root, RECORDED, &record) {
                Ok(()) => generate(root),
                Err(message) => {
                    println!("FAIL  {message}");
                    1
                }
            }
        }
        Err(message) => {
            println!("FAIL  {message}");
            1
        }
    }
}

fn benches(root: &Path) -> Result<Vec<Measured>, String> {
    let home = root.join(CRITERION_HOME);
    if home.exists() {
        std::fs::remove_dir_all(&home).map_err(|e| format!("{CRITERION_HOME}: {e}"))?;
    }
    let mut targets: Vec<(&str, &str)> = BUDGETS.iter().map(|b| (b.package, b.bench)).collect();
    targets.dedup();
    for (package, bench) in targets {
        let ids: Vec<&str> = BUDGETS
            .iter()
            .filter(|b| b.package == package && b.bench == bench)
            .map(|b| b.id)
            .collect();
        let status = Command::new(cargo())
            .args(["bench", "--quiet", "-p", package, "--bench", bench, "--"])
            .arg(filter(&ids))
            .env("CRITERION_HOME", &home)
            .current_dir(root)
            .status()
            .map_err(|e| format!("`cargo bench -p {package}` did not start: {e}"))?;
        if !status.success() {
            return Err(format!("`cargo bench -p {package} --bench {bench}` failed"));
        }
    }
    read_back(&home)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(figures: Vec<Measured>) -> Record {
        Record {
            os: "macos".into(),
            arch: "aarch64".into(),
            commit: "abc".into(),
            rustc: "rustc 1".into(),
            figures,
        }
    }

    fn within(budget: &Budget) -> Measured {
        let elements = matches!(budget.allowance, Allowance::PerElement(_)).then_some(900);
        let allowed = budget.allowed(elements).unwrap_or(1.0);
        Measured {
            key: budget.key.into(),
            samples: 100,
            median_us: allowed,
            min_us: allowed / 2.0,
            max_us: allowed * 3.0,
            elements,
        }
    }

    #[test]
    fn a_sample_is_a_run_and_its_median_is_taken_per_iteration() {
        let samples = Samples {
            iters: vec![10.0, 20.0, 30.0, 40.0],
            times: vec![10_000.0, 60_000.0, 60_000.0, 80_000.0],
        };
        let m = measured("k", &samples, Some(3)).expect("a figure");
        assert_eq!((m.samples, m.min_us, m.max_us), (4, 1.0, 3.0));
        assert!((m.median_us - 2.0).abs() < 1e-12, "{}", m.median_us);
        assert_eq!(m.elements, Some(3));
        let odd = Samples {
            iters: vec![1.0, 1.0, 1.0],
            times: vec![3000.0, 1000.0, 2000.0],
        };
        assert!((measured("k", &odd, None).expect("a figure").median_us - 2.0).abs() < 1e-12);
        let torn = Samples {
            iters: vec![1.0],
            times: vec![],
        };
        assert!(measured("k", &torn, None).is_err());
    }

    #[test]
    fn a_bench_is_read_back_by_its_full_id_and_not_its_directory() {
        let home = std::env::temp_dir().join(format!("teistro-micro-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&home);
        for (n, budget) in BUDGETS.iter().enumerate() {
            // A directory `criterion` might name it, which the reader ignores.
            let dir = home.join(format!("dir{n}")).join("new");
            std::fs::create_dir_all(&dir).expect("a directory");
            let throughput = if matches!(budget.allowance, Allowance::PerElement(_)) {
                r#","throughput":{"Elements":4396}"#
            } else {
                ""
            };
            std::fs::write(
                dir.join("benchmark.json"),
                format!(
                    r#"{{"group_id":"g","full_id":"{}"{throughput}}}"#,
                    budget.id
                ),
            )
            .expect("written");
            std::fs::write(
                dir.join("sample.json"),
                r#"{"sampling_mode":"Linear","iters":[1.0,2.0,3.0],"times":[1000.0,2000.0,3000.0]}"#,
            )
            .expect("written");
        }
        let read = read_back(&home).expect("every budget read back");
        assert_eq!(read.len(), BUDGETS.len());
        let rules = read.iter().find(|m| m.key == "rules").expect("the rules");
        assert_eq!(rules.elements, Some(4396));
        assert!(read.iter().all(|m| (m.median_us - 1.0).abs() < 1e-12));
        // A budget `criterion` never wrote is named.
        std::fs::remove_dir_all(home.join("dir0")).expect("removed");
        let missing = read_back(&home).expect_err("a missing bench fails");
        assert!(missing.contains(BUDGETS[0].id), "{missing}");
        let _ = std::fs::remove_dir_all(&home);
    }

    #[test]
    fn a_median_past_twice_its_budget_fails_and_a_rate_scales_with_its_count() {
        let whole = record(BUDGETS.iter().map(within).collect());
        assert_eq!(faults(&[("m".into(), whole)]), Vec::<String>::new());

        let mut figures: Vec<Measured> = BUDGETS.iter().map(within).collect();
        figures[0].median_us = 41.0;
        figures[0].max_us = 50.0;
        let found = faults(&[("m".into(), record(figures))]);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(
            found[0].contains("past 2× its budget of 20.0 µs"),
            "{found:?}"
        );

        let rules = BUDGETS
            .iter()
            .find(|b| b.key == "rules")
            .expect("the rules");
        assert!((rules.allowed(Some(900)).expect("a rate") - 2000.0).abs() < 1e-9);
        assert!((rules.allowed(Some(4396)).expect("a rate") - 4396.0 * RULE_US).abs() < 1e-9);
        let mut figures: Vec<Measured> = BUDGETS.iter().map(within).collect();
        for m in &mut figures {
            m.elements = None;
        }
        let found = faults(&[("m".into(), record(figures))]);
        assert!(
            found.iter().any(|f| f.contains("no element count")),
            "{found:?}"
        );
    }

    #[test]
    fn a_record_missing_an_operation_or_taken_dirty_is_no_claim() {
        let mut dirty = record(BUDGETS.iter().skip(1).map(within).collect());
        dirty.commit = "abc-dirty".into();
        let found = faults(&[("m".into(), dirty)]);
        assert!(
            found.iter().any(|f| f.contains("tree with changes")),
            "{found:?}"
        );
        assert!(
            found.iter().any(|f| f.contains(BUDGETS[0].key)),
            "{found:?}"
        );
    }

    #[test]
    fn the_filter_matches_exactly_the_ids_it_names() {
        assert_eq!(filter(&["a: at(t, 5)", "b.c"]), r"^(a: at\(t, 5\)|b\.c)$");
    }

    #[test]
    fn the_performance_page_states_every_budget_the_list_holds() {
        let page = std::fs::read_to_string(crate::repo_root().join(PERFORMANCE))
            .expect("the performance page");
        assert_eq!(page_disagrees(&page), Vec::<String>::new());
        let changed = page.replace("| 20 µs,", "| 25 µs,");
        assert!(
            page_disagrees(&changed)
                .iter()
                .any(|f| f.contains("does not state `20 µs`")),
            "a budget changed on the page alone is found"
        );
    }
}
