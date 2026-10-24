//! `docs/05-testing/app-speed-measured.md`: what an app pays, request by
//! request, rendered from the records `teistro-ephemeris-teimeris-app-speed`
//! writes (one file per machine in `docs/05-testing/app-speed/`), and the
//! gate on them.
//!
//! The budgets are the performance page's (`02-architecture/09-performance-
//! architecture.md`), written for a native provider, so they bind the two
//! setups over the Teimeris engine. Wall time is noisy, so a median fails
//! only past **twice** its budget. The built-in ephemeris is printed beside
//! them with its ratio to Teimeris and is not held to the native budget.
//!
//! A record also carries each setup's month grid on 1, 2, 4, … threads,
//! a context each. Every thread count must answer what one thread did,
//! and up to [`FLOOR_THREADS`] threads, fewer than the machine offers, each
//! setup must keep [`FLOOR`] of linear throughput: past that a machine's
//! threads are efficiency cores or a core's second thread, and a count
//! taking every core shares them with the system, none of which a context
//! can be held to. At least
//! one record must carry the measure, so the claim stands on a machine.

use std::fmt::Write as _;
use std::path::Path;

use serde::Deserialize;

use crate::generated::{Output, check, write};
use crate::measured::{Figure, MIN_RUNS, ratio};

const PAGE: &str = "docs/05-testing/app-speed-measured.md";
const RECORDED: &str = "docs/05-testing/app-speed";

/// How far past its budget a median may run before the gate fails it.
const CEILING: f64 = 2.0;

/// The least share of linear throughput a thread count up to
/// [`FLOOR_THREADS`] keeps: a context that waited on another's lock would
/// fall to a half at two threads and a quarter at four.
const FLOOR: f64 = 0.75;
const FLOOR_THREADS: usize = 4;

/// The thread counts a record's floor holds: up to [`FLOOR_THREADS`], and
/// fewer than the machine offers. A count taking every core shares them
/// with the system, so what it loses is the machine's rather than a lock's:
/// the four-core Android emulator kept 0.72 of linear at four threads over
/// Teimeris and 0.95 at two, where a lock would keep a half. A record
/// taken before the threads were measured offers none.
fn floor_holds(threads: usize, parallelism: usize) -> bool {
    threads <= FLOOR_THREADS && threads < parallelism
}

/// The setups the gate holds, and the order the page prints them in.
const GATED: [&str; 2] = ["drik-teimeris", "committee-teimeris"];
const SETUPS: [(&str, &str); 3] = [
    ("drik-teimeris", "drik, Teimeris"),
    (
        "committee-teimeris",
        "committee, Surya Siddhanta + Teimeris",
    ),
    ("drik-builtin", "drik, built-in"),
];

/// A request, what it asks, its budget and the target a consumer asked for
/// (milliseconds; `None` where the page states none).
struct Request {
    key: &'static str,
    asks: &'static str,
    budget: Option<f64>,
    target: Option<f64>,
}

const REQUESTS: [Request; 11] = [
    Request {
        key: "open",
        asks: "a context opened, the engine with it",
        budget: None,
        target: Some(3.0),
    },
    Request {
        key: "day",
        asks: "one almanac day",
        budget: Some(2.0),
        target: Some(0.5),
    },
    Request {
        key: "month",
        asks: "a month grid, 42 days in one call",
        budget: Some(60.0),
        target: Some(15.0),
    },
    Request {
        key: "year",
        asks: "a year of almanac days",
        budget: None,
        target: None,
    },
    Request {
        key: "festivals",
        asks: "a year of the Nepal pack's festivals",
        budget: None,
        target: Some(80.0),
    },
    Request {
        key: "muhurta",
        asks: "a 90-day marriage muhurta search",
        budget: Some(200.0),
        target: None,
    },
    Request {
        key: "chart",
        asks: "a birth chart with its Vimshottari to depth 3",
        budget: None,
        target: None,
    },
    Request {
        key: "parity-chart",
        asks: "the full parity chart: every section and varga, the baseline's parity-tier dashas to depth 3, every shipped rule",
        budget: Some(10.0),
        target: None,
    },
    Request {
        key: "widest-chart",
        asks: "the parity chart with every dasha a chart gives",
        budget: None,
        target: None,
    },
    Request {
        key: "matching",
        asks: "Ashta Koota between two cast charts",
        budget: None,
        target: None,
    },
    Request {
        key: "rectify",
        asks: "a day-long window rectified at the default one-minute seed",
        budget: Some(1000.0),
        target: None,
    },
];

#[derive(Deserialize)]
struct Timing {
    setup: String,
    request: String,
    runs: usize,
    median_ms: f64,
    min_ms: f64,
    max_ms: f64,
}

#[derive(Deserialize)]
struct Record {
    machine: String,
    os: String,
    arch: String,
    commit: String,
    teimeris: String,
    /// `embedded` or `files`: where the engine read its data. Records
    /// taken before the field was added read their data from files.
    #[serde(default = "files")]
    data: String,
    memo: String,
    timings: Vec<Timing>,
    /// The threads the machine offered; 0 in a record taken before the
    /// threads were measured.
    #[serde(default)]
    parallelism: usize,
    #[serde(default)]
    scaling: Vec<Scaling>,
}

/// A setup's month grid on a number of threads, a context each.
#[derive(Deserialize)]
struct Scaling {
    setup: String,
    threads: usize,
    months: u32,
    trials: usize,
    median_ms: f64,
    per_second: f64,
    efficiency: f64,
    identical: bool,
}

fn files() -> String {
    String::from("files")
}

impl Timing {
    /// The timing as the results schema holds it.
    fn figure(&self) -> Figure {
        Figure {
            runs: self.runs,
            median: self.median_ms,
            min: self.min_ms,
            max: self.max_ms,
        }
    }
}

impl Record {
    fn timing(&self, setup: &str, request: &str) -> Option<&Timing> {
        self.timings
            .iter()
            .find(|t| t.setup == setup && t.request == request)
    }
}

fn recorded(root: &Path) -> Result<Vec<(String, Record)>, String> {
    crate::measured::records(root, RECORDED, "teistro-ephemeris-teimeris-app-speed")
}

/// Every median a gated setup holds past its ceiling, as sentences.
fn breaches(records: &[(String, Record)]) -> Vec<String> {
    let mut found = Vec::new();
    for (name, record) in records {
        // A median is a claim only with the runs it was taken from and a
        // spread that holds it.
        for t in &record.timings {
            if let Some(fault) = t.figure().fault() {
                found.push(format!(
                    "{name}: `{}` under `{}`, in ms, is {fault}",
                    t.request, t.setup
                ));
            }
        }
        for row in &record.scaling {
            if row.trials < MIN_RUNS || row.months == 0 {
                found.push(format!(
                    "{name}: `{}` on {} threads is {} trials of {} months: \
                     a throughput needs {MIN_RUNS} trials of at least one month",
                    row.setup, row.threads, row.trials, row.months
                ));
            }
        }
        for request in &REQUESTS {
            let Some(budget) = request.budget else {
                continue;
            };
            for setup in GATED {
                match record.timing(setup, request.key) {
                    Some(t) if t.median_ms > CEILING * budget => found.push(format!(
                        "{name}: `{}` under `{setup}` took {:.3} ms, past {CEILING}x its {budget} ms budget",
                        request.key, t.median_ms
                    )),
                    Some(_) => {}
                    None => found.push(format!(
                        "{name}: no `{}` timing under `{setup}`; re-record it",
                        request.key
                    )),
                }
            }
        }
        for row in &record.scaling {
            if !row.identical {
                found.push(format!(
                    "{name}: `{}` on {} threads answered other than on one: a context shares state",
                    row.setup, row.threads
                ));
            }
            if floor_holds(row.threads, record.parallelism) && row.efficiency < FLOOR {
                found.push(format!(
                    "{name}: `{}` on {} threads kept {:.3} of linear throughput, under its floor of {FLOOR}",
                    row.setup, row.threads, row.efficiency
                ));
            }
        }
    }
    if records.iter().all(|(_, record)| record.scaling.is_empty()) {
        found.push(String::from(
            "no record measures contexts on threads; re-record one with the app-speed binary",
        ));
    }
    found
}

/// The table of a record's thread counts, or a line saying it has none.
fn threads(record: &Record) -> String {
    let mut out = String::new();
    if record.scaling.is_empty() {
        let _ = writeln!(
            out,
            "Contexts on threads: not measured in this record.
"
        );
        return out;
    }
    let _ = writeln!(
        out,
        "Contexts on threads, the `month` request's grid on each, a context a thread, of the \
         {} threads this machine offers. **Efficiency** is the throughput over the one \
         thread's times the count (1 is linear); up to {FLOOR_THREADS} threads, and fewer than \
         the machine offers, it is held to {FLOOR}. **Alike** is whether every thread's month hashed as one thread's did.\n",
        record.parallelism
    );
    let _ = writeln!(
        out,
        "| setup | threads | median trial, ms | months a second | efficiency | alike |"
    );
    let _ = writeln!(out, "|---|---:|---:|---:|---:|---|");
    for row in &record.scaling {
        let under = row.threads <= FLOOR_THREADS && row.efficiency < FLOOR;
        let _ = writeln!(
            out,
            "| {} | {} | {} ({} trials of {} months) | {:.2} | {:.3}{} | {} |",
            row.setup,
            row.threads,
            ms(row.median_ms),
            row.trials,
            row.months,
            row.per_second,
            row.efficiency,
            if under { " **under**" } else { "" },
            if row.identical { "yes" } else { "**no**" }
        );
    }
    out.push('\n');
    out
}

fn ms(value: f64) -> String {
    if value >= 100.0 {
        format!("{value:.0}")
    } else if value >= 10.0 {
        format!("{value:.1}")
    } else if value >= 1.0 {
        format!("{value:.2}")
    } else {
        format!("{value:.3}")
    }
}

fn limit(value: Option<f64>) -> String {
    value.map_or_else(|| String::from("—"), |v| format!("{} ms", ms(v)))
}

fn cell(record: &Record, setup: &str, request: &Request) -> String {
    let Some(t) = record.timing(setup, request.key) else {
        return String::from("not recorded");
    };
    let over = GATED.contains(&setup)
        && request
            .budget
            .is_some_and(|budget| t.median_ms > CEILING * budget);
    let mark = if over { " **over**" } else { "" };
    format!("{}{mark}", ms(t.median_ms))
}

fn section(name: &str, record: &Record) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "## `{name}`\n");
    let _ = writeln!(
        out,
        "{} ({}, {}), commit `{}`, Teimeris {}, its data {}. Memo: {}.\n",
        record.machine,
        record.os,
        record.arch,
        record.commit,
        record.teimeris,
        if record.data == "embedded" {
            "built in"
        } else {
            "read from files"
        },
        record.memo
    );
    let _ = write!(out, "| request | budget | target |");
    for (_, label) in SETUPS {
        let _ = write!(out, " {label} |");
    }
    let _ = writeln!(out, " built-in ÷ Teimeris |");
    let _ = writeln!(out, "|---|---:|---:|---:|---:|---:|---:|");
    for request in &REQUESTS {
        let ratio = match (
            record.timing("drik-builtin", request.key),
            record.timing("drik-teimeris", request.key),
        ) {
            (Some(b), Some(t)) => ratio(&b.figure(), &t.figure()).render(|r| format!("{r:.1}×")),
            _ => String::from("—"),
        };
        let _ = write!(
            out,
            "| {} ({}) | {} | {} |",
            request.key,
            request.asks,
            limit(request.budget),
            limit(request.target)
        );
        for (setup, _) in SETUPS {
            let _ = write!(out, " {} |", cell(record, setup, request));
        }
        let _ = writeln!(out, " {ratio} |");
    }
    let _ = writeln!(out, "\nMedians in milliseconds. The spread of every row:\n");
    let _ = writeln!(out, "| setup | request | runs | fastest | slowest |");
    let _ = writeln!(out, "|---|---|---:|---:|---:|");
    for t in &record.timings {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} |",
            t.setup,
            t.request,
            t.runs,
            ms(t.min_ms),
            ms(t.max_ms)
        );
    }
    out
}

fn page(records: &[(String, Record)]) -> String {
    let mut out = String::from("# What an app pays\n\n");
    let _ = writeln!(
        out,
        "Status: `generated` by `cargo xtask app-speed` from `{RECORDED}/*.json`, gated by \
         `check-app-speed`. Do not edit. Each record is written by \
         `teistro-ephemeris-teimeris-app-speed` (`adapters/ephemeris-teimeris/rust`), which times \
         the calls an almanac or chart screen makes through the Rust façade, each on a fresh \
         context so that the call memo starts empty.\n"
    );
    let _ = writeln!(
        out,
        "The **budget** is the performance page's (`02-architecture/09-performance-architecture.md`), \
         written for a native provider: the gate fails a median past {CEILING}× its budget under \
         either setup over Teimeris, because wall time is noisy and a tight ceiling would fail on \
         a busy runner rather than on a regression. The **target** is the faster figure a \
         consumer's app needs on a low-end phone, scaled to a laptop core, and is reported, not \
         gated. The built-in ephemeris needs no engine and is held to accuracy, not to the native \
         budget; its ratio to Teimeris is the column on the right, which says `within spread` where \
         the two setups' fastest-to-slowest ranges overlap, since no ratio is measured there. A \
         median taken from fewer than {MIN_RUNS} runs, or outside its own range, fails the gate.\n"
    );
    let _ = writeln!(
        out,
        "```text\ncd adapters/ephemeris-teimeris/rust\ncargo run --release --features app-speed \\\n  --bin teistro-ephemeris-teimeris-app-speed \\\n  > ../../../{RECORDED}/desktop-$(uname -s)-$(uname -m).json\ncargo xtask app-speed\n```\n"
    );
    let over = breaches(records);
    if over.is_empty() {
        let _ = writeln!(
            out,
            "Every gated median is inside {CEILING}× its budget, and every thread count \
             answers alike and keeps its floor.\n"
        );
    } else {
        let _ = writeln!(out, "**Past the ceiling:**\n");
        for line in &over {
            let _ = writeln!(out, "- {line}");
        }
        out.push('\n');
    }
    for (name, record) in records {
        out.push_str(&section(name, record));
        out.push('\n');
        out.push_str(&threads(record));
    }
    out.truncate(out.trim_end().len());
    out.push('\n');
    out
}

fn outputs(root: &Path) -> Result<(Vec<Output>, Vec<String>), i32> {
    match recorded(root) {
        Ok(records) => Ok((vec![Output::new(PAGE, page(&records))], breaches(&records))),
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
/// gated median past its ceiling.
pub(crate) fn check_generated(root: &Path) -> i32 {
    match outputs(root) {
        Ok((outputs, over)) => {
            for line in &over {
                println!("FAIL  {line}");
            }
            let page = check(root, &outputs, "cargo xtask app-speed");
            if over.is_empty() { page } else { 1 }
        }
        Err(code) => code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(setup: &str, request: &str, median_ms: f64) -> Record {
        Record {
            machine: String::from("m"),
            os: String::from("o"),
            arch: String::from("a"),
            commit: String::from("c"),
            teimeris: String::from("t"),
            data: files(),
            memo: String::from("cold"),
            parallelism: 0,
            scaling: Vec::new(),
            timings: vec![Timing {
                setup: setup.into(),
                request: request.into(),
                runs: 3,
                median_ms,
                min_ms: median_ms,
                max_ms: median_ms,
            }],
        }
    }

    #[test]
    fn a_gated_median_past_twice_its_budget_fails_and_one_inside_does_not() {
        let inside = record("drik-teimeris", "day", 3.9);
        let past = record("committee-teimeris", "day", 4.1);
        let builtin = record("drik-builtin", "day", 400.0);
        let over = |r: Record| {
            breaches(&[(String::from("x"), r)])
                .into_iter()
                .filter(|line| line.contains("`day`") && line.contains("past"))
                .count()
        };
        assert_eq!(over(inside), 0);
        assert_eq!(over(past), 1);
        assert_eq!(
            over(builtin),
            0,
            "the built-in is not held to the native budget"
        );
    }

    #[test]
    fn a_missing_gated_timing_is_named() {
        let lines = breaches(&[(String::from("x"), record("drik-builtin", "open", 1.0))]);
        assert!(
            lines
                .iter()
                .any(|l| l.contains("no `day` timing under `drik-teimeris`"))
        );
    }

    fn scaled(threads: usize, efficiency: f64, identical: bool) -> Record {
        let mut record = record("drik-teimeris", "day", 1.0);
        record.parallelism = 8;
        record.scaling = vec![Scaling {
            setup: String::from("drik-teimeris"),
            threads,
            months: 3,
            trials: 5,
            median_ms: 60.0,
            per_second: 50.0,
            efficiency,
            identical,
        }];
        record
    }

    #[test]
    fn a_thread_count_under_its_floor_or_answering_otherwise_fails() {
        let threads = |r: Record| {
            breaches(&[(String::from("x"), r)])
                .into_iter()
                .filter(|line| line.contains("threads"))
                .count()
        };
        assert_eq!(threads(scaled(4, 0.80, true)), 0);
        assert_eq!(threads(scaled(4, 0.70, true)), 1);
        assert_eq!(
            threads(scaled(8, 0.50, true)),
            0,
            "past four threads the efficiency is reported, not held"
        );
        assert_eq!(threads(scaled(8, 0.90, false)), 1);
        let every_core = |threads, efficiency| {
            let mut record = scaled(threads, efficiency, true);
            record.parallelism = 4;
            record
        };
        assert_eq!(
            threads(every_core(4, 0.70)),
            0,
            "a count taking every core is reported, not held"
        );
        assert_eq!(
            threads(every_core(2, 0.50)),
            1,
            "a lock still shows below it"
        );
        assert_eq!(
            threads(record("drik-teimeris", "day", 1.0)),
            1,
            "a claim no record measures fails"
        );
    }

    #[test]
    fn a_median_without_its_runs_or_outside_its_spread_fails() {
        let mut few = record("drik-builtin", "open", 1.0);
        few.timings[0].runs = 2;
        let mut outside = record("drik-builtin", "open", 1.0);
        outside.timings[0].max_ms = 0.5;
        let named = |r: Record| {
            breaches(&[(String::from("x"), r)])
                .into_iter()
                .filter(|line| line.contains("a median needs"))
                .count()
        };
        assert_eq!(named(record("drik-builtin", "open", 1.0)), 0);
        assert_eq!(named(few), 1);
        assert_eq!(named(outside), 1);
    }
}
