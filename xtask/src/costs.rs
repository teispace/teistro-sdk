//! `costs-measured.md`: what each record entry point costs relative to a
//! plain chart, in instructions counted under callgrind (SR-54), rendered
//! from a record so the page is measured and never written by hand.
//!
//! The entry points are `teistro_scenario::entries`: a plain chart (the
//! unit) and every record's example request from the fixture the agent
//! server's tests send, the engine's records left out (a consumer's own
//! engine answers them). Each is run on a fresh context over the built-in
//! ephemeris: once, to price its first call, and twice, to price a call
//! on a context that has answered it before; the context alone is the
//! floor taken out of both.
//!
//! - `cargo xtask costs --measure FILE` counts them (Linux with valgrind,
//!   as `bench` does) and writes what it counted to `FILE`;
//! - `cargo xtask costs --from FILE` writes the record,
//!   `docs/05-testing/costs.json`, from such a file and renders the page;
//!   `costs --render` renders it from the record as it stands;
//! - `cargo xtask check-costs` holds the page to the record and the record
//!   to the entry points (the same names and requests, in order),
//!   everywhere; with `--from FILE`, the benchmarks workflow holds a fresh
//!   count to the record: a relative cost more than [`DRIFT`] from it is
//!   stale, and the record is re-measured from that run's file.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};
use teistro_scenario::{Entry, UNIT, UNPRICED, entries};

use crate::binding::{build, present};
use crate::generated::{Output, check, write};

/// The record the page is rendered from.
const RECORD: &str = "docs/05-testing/costs.json";
/// The page.
const PAGE: &str = "docs/05-testing/costs-measured.md";
/// The record's and the counted file's schema.
const SCHEMA: &str = "teistro-costs/1";
/// How far a fresh relative cost may move from the record's before the
/// record is stale: a tenth. The page prices entry points against one
/// another to the first figure or two, so a smaller move does not change
/// what a reader plans by, and a larger one does.
const DRIFT: f64 = 0.10;

/// What one entry point costs, in instructions above the context alone.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Counted {
    name: String,
    /// The first call on a fresh context.
    first: u64,
    /// A second call of the same request on the same context.
    again: u64,
}

/// What `costs --measure` writes: the counts and what counted them.
#[derive(Debug, Serialize, Deserialize)]
struct Measured {
    schema: String,
    rustc: String,
    target: String,
    /// The context alone, built over the built-in ephemeris.
    floor: u64,
    entries: Vec<Counted>,
}

/// One entry point in the record: what it asked and what it cost.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Priced {
    name: String,
    record: String,
    request: serde_json::Value,
    first: u64,
    again: u64,
}

/// The record.
#[derive(Debug, Serialize, Deserialize)]
struct Record {
    schema: String,
    commit: String,
    rustc: String,
    target: String,
    floor: u64,
    entries: Vec<Priced>,
}

impl Record {
    /// The unit's two counts.
    fn unit(&self) -> Result<(u64, u64), String> {
        self.entries
            .iter()
            .find(|e| e.name == UNIT)
            .map(|e| (e.first, e.again))
            .ok_or_else(|| format!("{RECORD} has no `{UNIT}`"))
    }
}

/// `costs` and `check-costs`.
pub(crate) fn command(root: &Path, command: &str, args: &[String]) -> i32 {
    let flag = |name: &str| {
        args.iter()
            .position(|arg| arg == name)
            .and_then(|at| args.get(at + 1))
            .map(Path::new)
    };
    let result = match command {
        "costs" if args.iter().any(|arg| arg == "--render") => render_record(root),
        "costs" => match (flag("--measure"), flag("--from")) {
            (Some(out), None) => measure(root, out),
            (None, Some(from)) => record_from(root, from),
            _ => {
                eprintln!(
                    "usage: cargo xtask costs --measure FILE | costs --from FILE | costs --render | check-costs [--from FILE]"
                );
                return 2;
            }
        },
        _ => check_costs(root, flag("--from")),
    };
    match result {
        Ok(code) => code,
        Err(why) => {
            println!("FAIL  {why}");
            1
        }
    }
}

// ── measuring ──────────────────────────────────────────────────────────────

fn measure(root: &Path, out: &Path) -> Result<i32, String> {
    if !present("valgrind", "--version") {
        eprintln!(
            "no `valgrind` on this machine; instruction counts need it (Linux, `apt install valgrind`)"
        );
        return Ok(0);
    }
    build(root, "teistro-scenario", "the scenario")
        .map_err(|()| "the scenario did not build".to_owned())?;
    let binary = root.join("target/release/teistro-scenario");
    let into = root.join("target/callgrind");
    fs::create_dir_all(&into).map_err(|e| format!("{}: {e}", into.display()))?;
    let count = |name: &str, calls: &str| {
        crate::bench::counted(
            &binary,
            &["entry", name, calls],
            &format!("entry.{name}.{calls}"),
            &into,
        )
        .ok_or_else(|| format!("callgrind could not count `{name}` answered {calls} times"))
    };
    let floor = count(UNIT, "0")?;
    let mut counted = Vec::new();
    for entry in entries()? {
        let once = count(&entry.name, "1")?;
        let twice = count(&entry.name, "2")?;
        let first = once.saturating_sub(floor);
        let again = twice.saturating_sub(once);
        println!("{:<28} {first:>16} {again:>16}", entry.name);
        counted.push(Counted {
            name: entry.name,
            first,
            again,
        });
    }
    let measured = Measured {
        schema: SCHEMA.to_owned(),
        rustc: output(Command::new("rustc").arg("-V")),
        target: format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS),
        floor,
        entries: counted,
    };
    let text = serde_json::to_string_pretty(&measured).map_err(|e| e.to_string())? + "\n";
    fs::write(out, text).map_err(|e| format!("{}: {e}", out.display()))?;
    println!("wrote {}", out.display());
    Ok(0)
}

fn output(command: &mut Command) -> String {
    command
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_owned())
        .unwrap_or_default()
}

fn read_measured(from: &Path) -> Result<Measured, String> {
    let text = fs::read_to_string(from).map_err(|e| format!("{}: {e}", from.display()))?;
    let measured: Measured =
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", from.display()))?;
    if measured.schema != SCHEMA {
        return Err(format!(
            "{} is `{}`, not `{SCHEMA}`",
            from.display(),
            measured.schema
        ));
    }
    Ok(measured)
}

// ── the record ─────────────────────────────────────────────────────────────

fn read_record(root: &Path) -> Result<Record, String> {
    let text = fs::read_to_string(root.join(RECORD)).map_err(|e| format!("{RECORD}: {e}"))?;
    let record: Record = serde_json::from_str(&text).map_err(|e| format!("{RECORD}: {e}"))?;
    if record.schema != SCHEMA {
        return Err(format!("{RECORD} is `{}`, not `{SCHEMA}`", record.schema));
    }
    Ok(record)
}

/// The record a counted file makes: every entry point priced, in
/// [`entries`]' order, with the request it was priced on.
fn priced(measured: Measured, commit: String) -> Result<Record, String> {
    let mut entries_priced = Vec::new();
    for entry in entries()? {
        let counted = measured
            .entries
            .iter()
            .find(|c| c.name == entry.name)
            .ok_or_else(|| format!("the counted file has no `{}`", entry.name))?;
        entries_priced.push(Priced {
            name: entry.name,
            record: entry.record,
            request: entry.request,
            first: counted.first,
            again: counted.again,
        });
    }
    Ok(Record {
        schema: SCHEMA.to_owned(),
        commit,
        rustc: measured.rustc,
        target: measured.target,
        floor: measured.floor,
        entries: entries_priced,
    })
}

fn record_from(root: &Path, from: &Path) -> Result<i32, String> {
    let measured = read_measured(from)?;
    let commit = output(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(root),
    );
    let record = priced(measured, commit)?;
    let text = serde_json::to_string_pretty(&record).map_err(|e| e.to_string())? + "\n";
    let page = render(&record)?;
    Ok(write(
        root,
        &[Output::new(RECORD, text), Output::new(PAGE, page)],
    ))
}

fn render_record(root: &Path) -> Result<i32, String> {
    let record = read_record(root)?;
    Ok(write(root, &[Output::new(PAGE, render(&record)?)]))
}

// ── the gate ───────────────────────────────────────────────────────────────

fn check_costs(root: &Path, from: Option<&Path>) -> Result<i32, String> {
    let record = read_record(root)?;
    let mut failures = covers(&record, &entries()?);
    failures += check(
        root,
        &[Output::new(PAGE, render(&record)?)],
        "cargo xtask costs --render",
    );
    if let Some(from) = from {
        let fresh = priced(read_measured(from)?, record.commit.clone())?;
        let drifted = drift(&record, &fresh)?;
        for line in &drifted {
            println!("FAIL  {line}");
        }
        if drifted.is_empty() {
            println!(
                "ok    every relative cost within {:.0}% of {RECORD}",
                DRIFT * 100.0
            );
        } else {
            println!(
                "      {RECORD} is stale: re-record it with `cargo xtask costs --from {}`",
                from.display()
            );
            failures += 1;
        }
    }
    Ok(i32::from(failures > 0))
}

/// Whether the record prices exactly the entry points, each on its request.
fn covers(record: &Record, wanted: &[Entry]) -> i32 {
    let recorded: Vec<(&str, &str, &serde_json::Value)> = record
        .entries
        .iter()
        .map(|e| (e.name.as_str(), e.record.as_str(), &e.request))
        .collect();
    let asked: Vec<(&str, &str, &serde_json::Value)> = wanted
        .iter()
        .map(|e| (e.name.as_str(), e.record.as_str(), &e.request))
        .collect();
    if recorded == asked {
        println!(
            "ok    {RECORD} prices the {} entry points, each on its example",
            asked.len()
        );
        return 0;
    }
    for (name, record_name, request) in &asked {
        match recorded.iter().find(|(n, _, _)| n == name) {
            None => println!("FAIL  {RECORD} does not price `{name}`"),
            Some((_, r, q)) if r != record_name || q != request => {
                println!("FAIL  `{name}`'s request is not the one {RECORD} priced");
            }
            Some(_) => {}
        }
    }
    for (name, _, _) in &recorded {
        if !asked.iter().any(|(n, _, _)| n == name) {
            println!("FAIL  {RECORD} prices `{name}`, which is no entry point");
        }
    }
    println!(
        "      re-record it: `cargo xtask costs --measure FILE` on Linux, then `costs --from FILE`"
    );
    1
}

/// Each relative cost of `fresh` more than [`DRIFT`] from the record's.
fn drift(record: &Record, fresh: &Record) -> Result<Vec<String>, String> {
    let (unit_first, unit_again) = record.unit()?;
    let (fresh_first, fresh_again) = fresh.unit()?;
    let mut drifted = Vec::new();
    for was in &record.entries {
        let Some(now) = fresh.entries.iter().find(|now| now.name == was.name) else {
            drifted.push(format!("`{}` was not counted", was.name));
            continue;
        };
        for (what, a, b) in [
            (
                "first call",
                relative(was.first, unit_first),
                relative(now.first, fresh_first),
            ),
            (
                "call again",
                relative(was.again, unit_again),
                relative(now.again, fresh_again),
            ),
        ] {
            let moved = if a > 0.0 { (b - a).abs() / a } else { b };
            if moved > DRIFT {
                drifted.push(format!(
                    "`{}` {what}: {} the unit, recorded {}",
                    was.name,
                    times(b),
                    times(a)
                ));
            }
        }
    }
    Ok(drifted)
}

#[expect(
    clippy::cast_precision_loss,
    reason = "a ratio of instruction counts read to three figures"
)]
fn relative(count: u64, unit: u64) -> f64 {
    if unit == 0 {
        0.0
    } else {
        count as f64 / unit as f64
    }
}

/// A ratio to three significant figures, `×` after it.
fn times(ratio: f64) -> String {
    if ratio >= 100.0 {
        format!("{ratio:.0}×")
    } else if ratio >= 10.0 {
        format!("{ratio:.1}×")
    } else if ratio >= 1.0 {
        format!("{ratio:.2}×")
    } else if ratio >= 0.1 {
        format!("{ratio:.3}×")
    } else if ratio >= 0.01 {
        format!("{ratio:.4}×")
    } else if ratio >= 0.001 {
        format!("{ratio:.5}×")
    } else {
        "<0.001×".to_owned()
    }
}

/// A count with its thousands apart by a space.
fn grouped(count: u64) -> String {
    let digits = count.to_string();
    let mut out = String::new();
    for (k, digit) in digits.chars().enumerate() {
        if k > 0 && (digits.len() - k) % 3 == 0 {
            out.push(' ');
        }
        out.push(digit);
    }
    out
}

// ── the page ───────────────────────────────────────────────────────────────

fn render(record: &Record) -> Result<String, String> {
    let (unit_first, unit_again) = record.unit()?;
    let mut page = String::new();
    let short = record.commit.get(..12).unwrap_or(&record.commit);
    let _ = writeln!(page, "# What each entry point costs, measured\n");
    let _ = writeln!(
        page,
        "Status: `generated` by `cargo xtask costs --from FILE` from instructions counted under \
         callgrind (`cargo xtask costs --measure FILE`, Linux); `check-costs` holds this page to \
         [`costs.json`](costs.json), and the benchmarks workflow holds every pull request's count \
         to it within {:.0}%. Do not edit.\n",
        DRIFT * 100.0
    );
    let _ = writeln!(
        page,
        "Each entry point is a record, the name the C boundary, every binding and the agent \
         server's tools share, answered on the example request of \
         `crates/sdk/tests/requests.json` over the built-in ephemeris. The unit is a plain chart, \
         `chart.found` at Kathmandu with nothing read of it but the chart. **First call** is the \
         entry point's first answer on a fresh context, what a one-off call pays, the \
         ephemeris's segments decoded for it included; **again** is the same request a second \
         time on that context, what a consumer that keeps its context pays per call. Both are \
         instructions above the context alone ({floor} to build it), so they are counts, not \
         times: a relative cost is the same on a phone and a server where a time is not, and an \
         entry point's figure says how its work compares, not how long it takes on any \
         machine. The engine's records (`{UNPRICED}*`) answer from the consumer's own engine and \
         are not priced.\n",
        floor = grouped(record.floor)
    );
    let _ = writeln!(
        page,
        "Counted at `{short}`, `{}`, {}.\n",
        record.rustc, record.target
    );
    let _ = writeln!(
        page,
        "| entry point | first call | again | instructions, first call | instructions, again |"
    );
    let _ = writeln!(page, "|---|---:|---:|---:|---:|");
    for entry in &record.entries {
        let name = if entry.name == UNIT {
            format!("`{}` (the unit)", entry.record)
        } else {
            format!("`{}`", entry.name)
        };
        let _ = writeln!(
            page,
            "| {name} | {} | {} | {} | {} |",
            times(relative(entry.first, unit_first)),
            times(relative(entry.again, unit_again)),
            grouped(entry.first),
            grouped(entry.again)
        );
    }
    Ok(page)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(entries: Vec<Priced>) -> Record {
        Record {
            schema: SCHEMA.into(),
            commit: "abc".into(),
            rustc: "rustc 1".into(),
            target: "x86_64-linux".into(),
            floor: 10,
            entries,
        }
    }

    fn priced(name: &str, first: u64, again: u64) -> Priced {
        Priced {
            name: name.into(),
            record: "chart.found".into(),
            request: serde_json::json!({}),
            first,
            again,
        }
    }

    #[test]
    fn a_ratio_is_written_to_three_figures() {
        assert_eq!(times(123.4), "123×");
        assert_eq!(times(12.34), "12.3×");
        assert_eq!(times(1.0), "1.00×");
        assert_eq!(times(0.1234), "0.123×");
        assert_eq!(times(0.012_34), "0.0123×");
        assert_eq!(times(0.000_1), "<0.001×");
        assert_eq!(grouped(1_234_567), "1 234 567");
        assert_eq!(grouped(12), "12");
    }

    #[test]
    fn a_relative_cost_past_the_drift_is_stale_and_one_inside_it_is_not() {
        let was = record(vec![priced(UNIT, 100, 50), priced("a", 200, 100)]);
        // Everything twice as dear: no ratio moved.
        let doubled = record(vec![priced(UNIT, 200, 100), priced("a", 400, 200)]);
        assert_eq!(drift(&was, &doubled).unwrap(), Vec::<String>::new());
        let inside = record(vec![priced(UNIT, 100, 50), priced("a", 219, 109)]);
        assert_eq!(drift(&was, &inside).unwrap(), Vec::<String>::new());
        let past = record(vec![priced(UNIT, 100, 50), priced("a", 221, 100)]);
        let found = drift(&was, &past).unwrap();
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].contains("`a` first call"), "{found:?}");
    }

    #[test]
    fn the_record_covers_the_entry_points_or_names_what_differs() {
        let wanted = entries().unwrap();
        let mut exact = record(
            wanted
                .iter()
                .map(|e| Priced {
                    name: e.name.clone(),
                    record: e.record.clone(),
                    request: e.request.clone(),
                    first: 1,
                    again: 1,
                })
                .collect(),
        );
        assert_eq!(covers(&exact, &wanted), 0);
        exact.entries[1].request = serde_json::json!({"changed": true});
        assert_eq!(covers(&exact, &wanted), 1);
        exact.entries.pop();
        assert_eq!(covers(&exact, &wanted), 1);
    }

    #[test]
    fn the_page_prices_each_entry_against_the_unit() {
        let page = render(&record(vec![priced(UNIT, 100, 50), priced("a", 250, 25)])).unwrap();
        assert!(
            page.contains("| `chart.found` (the unit) | 1.00× | 1.00× | 100 | 50 |"),
            "{page}"
        );
        assert!(
            page.contains("| `a` | 2.50× | 0.500× | 250 | 25 |"),
            "{page}"
        );
        assert!(render(&record(vec![priced("a", 1, 1)])).is_err());
    }
}
