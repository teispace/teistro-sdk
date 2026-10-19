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

use std::fmt::Write as _;
use std::path::Path;

use serde::Deserialize;

use crate::generated::{Output, check, write};

const PAGE: &str = "docs/05-testing/app-speed-measured.md";
const RECORDED: &str = "docs/05-testing/app-speed";

/// How far past its budget a median may run before the gate fails it.
const CEILING: f64 = 2.0;

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

const REQUESTS: [Request; 8] = [
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
        budget: Some(10.0),
        target: None,
    },
    Request {
        key: "matching",
        asks: "Ashta Koota between two cast charts",
        budget: None,
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
    memo: String,
    timings: Vec<Timing>,
}

impl Record {
    fn timing(&self, setup: &str, request: &str) -> Option<&Timing> {
        self.timings
            .iter()
            .find(|t| t.setup == setup && t.request == request)
    }
}

fn recorded(root: &Path) -> Result<Vec<(String, Record)>, String> {
    let dir = root.join(RECORDED);
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .map_err(|e| format!("{RECORDED}: {e}"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .filter_map(|path| Some(path.file_stem()?.to_string_lossy().into_owned()))
        .collect();
    names.sort();
    if names.is_empty() {
        return Err(format!(
            "{RECORDED} holds no record; run `teistro-ephemeris-teimeris-app-speed`"
        ));
    }
    names
        .into_iter()
        .map(|name| {
            let path = dir.join(format!("{name}.json"));
            let text = std::fs::read_to_string(&path).map_err(|e| format!("{name}.json: {e}"))?;
            let record: Record =
                serde_json::from_str(&text).map_err(|e| format!("{RECORDED}/{name}.json: {e}"))?;
            Ok((name, record))
        })
        .collect()
}

/// Every median a gated setup holds past its ceiling, as sentences.
fn breaches(records: &[(String, Record)]) -> Vec<String> {
    let mut found = Vec::new();
    for (name, record) in records {
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
    }
    found
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
        "{} ({}, {}), commit `{}`, Teimeris {}. Memo: {}.\n",
        record.machine, record.os, record.arch, record.commit, record.teimeris, record.memo
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
            (Some(b), Some(t)) if t.median_ms > 0.0 => format!("{:.1}×", b.median_ms / t.median_ms),
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
         budget; its ratio to Teimeris is the column on the right.\n"
    );
    let _ = writeln!(
        out,
        "```text\ncd adapters/ephemeris-teimeris/rust\ncargo run --release --features app-speed \\\n  --bin teistro-ephemeris-teimeris-app-speed \\\n  > ../../../{RECORDED}/desktop-$(uname -s)-$(uname -m).json\ncargo xtask app-speed\n```\n"
    );
    let over = breaches(records);
    if over.is_empty() {
        let _ = writeln!(out, "Every gated median is inside {CEILING}× its budget.\n");
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
            memo: String::from("cold"),
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
}
