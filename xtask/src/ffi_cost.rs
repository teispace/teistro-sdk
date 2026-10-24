//! `docs/05-testing/ffi-cost-measured.md`: what a binding pays to cross
//! the C ABI (`03-design/ffi-cost.md`), rendered from the records
//! `cargo xtask measure-ffi-cost` writes, one file per machine in
//! `docs/05-testing/ffi-cost/`, and the gate on them.
//!
//! No budget binds these figures yet: they are published so that one can
//! be set from them. The gate holds every figure to the results schema
//! (`measured.rs`), and every record to carry each key for every binding
//! [`BINDINGS`] names.

use std::fmt::Write as _;
use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::binding::{build, cargo};
use crate::generated::{Output, check, write};
use crate::measured::{Figure, difference, us, version};

const PAGE: &str = "docs/05-testing/ffi-cost-measured.md";
const RECORDED: &str = "docs/05-testing/ffi-cost";

/// The bindings a record must carry, in the order the page prints them,
/// each with what it is timed from.
const BINDINGS: [(&str, &str); 5] = [
    (
        "rust",
        "`crates/ffi/examples/ffi_cost.rs`, the C ABI from Rust",
    ),
    ("node", "`bindings/node/bench/ffi-cost.mjs`, the Node layer"),
    (
        "python",
        "`bindings/python/bench/ffi_cost.py`, the Python layer",
    ),
    (
        "dart",
        "`bindings/dart/bench/ffi_cost.dart`, the Dart layer",
    ),
    ("java", "`bindings/java/bench/FfiCost.java`, the Java layer"),
];

/// The keys every binding times, and what each asks.
const KEYS: [(&str, &str); 5] = [
    ("call-native", "1 × 3, test provider"),
    ("call-hosted", "1 × 3, hosted provider"),
    ("grid-native", "366 × 10, test provider"),
    ("grid-hosted", "366 × 10, hosted provider"),
    ("decode", "the 366 × 10 blob, decoded alone"),
];

/// The cells of the grid, which the hosted provider's grid cost is shared
/// over.
const GRID_CELLS: f64 = 3660.0;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Timed {
    binding: String,
    key: String,
    runs: usize,
    /// The calls one run timed.
    calls: u64,
    median_us: f64,
    min_us: f64,
    max_us: f64,
}

impl Timed {
    fn figure(&self) -> Figure {
        Figure {
            runs: self.runs,
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
    /// Each binding's runtime, as its own tool names it.
    runtimes: Vec<(String, String)>,
    figures: Vec<Timed>,
}

impl Record {
    fn figure(&self, binding: &str, key: &str) -> Option<&Timed> {
        self.figures
            .iter()
            .find(|t| t.binding == binding && t.key == key)
    }
}

fn recorded(root: &Path) -> Result<Vec<(String, Record)>, String> {
    crate::measured::records(root, RECORDED, "cargo xtask measure-ffi-cost")
}

/// Every figure that is no claim and every one a record lacks, as
/// sentences.
fn faults(records: &[(String, Record)]) -> Vec<String> {
    let mut found = Vec::new();
    for (name, record) in records {
        found.extend(crate::measured::dirty(name, &record.commit));
        for t in &record.figures {
            if let Some(fault) = t.figure().fault() {
                found.push(format!(
                    "{name}: `{}` in {}, in µs, is {fault}",
                    t.key, t.binding
                ));
            }
        }
        for (binding, _) in BINDINGS {
            for (key, _) in KEYS {
                if record.figure(binding, key).is_none() {
                    found.push(format!(
                        "{name}: no `{key}` figure from {binding}; re-record with \
                         `cargo xtask measure-ffi-cost`"
                    ));
                }
            }
        }
    }
    found
}

/// What a binding pays over Rust for one key: its figure less Rust's,
/// with its share of `cells` where there are more than one.
fn over_rust(record: &Record, binding: &str, key: &str, cells: f64) -> String {
    if binding == "rust" {
        return String::from("—");
    }
    match (record.figure(binding, key), record.figure("rust", key)) {
        (Some(b), Some(r)) => difference(&b.figure(), &r.figure()).render(|d| {
            if cells > 1.0 {
                format!("{} ({} a cell)", us(d), us(d / cells))
            } else {
                us(d)
            }
        }),
        _ => String::from("—"),
    }
}

fn section(name: &str, record: &Record) -> String {
    let mut out = String::new();
    let runtimes: Vec<String> = record
        .runtimes
        .iter()
        .map(|(binding, version)| format!("{binding} {version}"))
        .collect();
    let _ = writeln!(
        out,
        "## `{name}`\n\n{} {}, commit `{}`; {}.\n",
        record.os,
        record.arch,
        record.commit,
        runtimes.join(", ")
    );
    let _ = write!(out, "| binding |");
    for (key, asks) in KEYS {
        let _ = write!(out, " `{key}` ({asks}) |");
    }
    let _ = writeln!(
        out,
        " marshalling: `call-native` over Rust | callback: `call-hosted` over Rust | a grid's \
         callback: `grid-hosted` over Rust |"
    );
    let _ = writeln!(out, "|---|---:|---:|---:|---:|---:|---:|---:|---:|");
    for (binding, _) in BINDINGS {
        let _ = write!(out, "| {binding} |");
        for (key, _) in KEYS {
            let cell = record
                .figure(binding, key)
                .map_or_else(|| String::from("not recorded"), |t| us(t.median_us));
            let _ = write!(out, " {cell} |");
        }
        let _ = writeln!(
            out,
            " {} | {} | {} |",
            over_rust(record, binding, "call-native", 1.0),
            over_rust(record, binding, "call-hosted", 1.0),
            over_rust(record, binding, "grid-hosted", GRID_CELLS)
        );
    }
    let _ = writeln!(
        out,
        "\nMedians in microseconds a call. The spread of every figure:\n"
    );
    let _ = writeln!(
        out,
        "| binding | key | runs | calls a run | fastest | slowest |"
    );
    let _ = writeln!(out, "|---|---|---:|---:|---:|---:|");
    for t in &record.figures {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} |",
            t.binding,
            t.key,
            t.runs,
            t.calls,
            us(t.min_us),
            us(t.max_us)
        );
    }
    out
}

fn page(records: &[(String, Record)]) -> String {
    let mut out = String::from("# What a binding pays to cross\n\n");
    let _ = writeln!(
        out,
        "Status: `generated` by `cargo xtask ffi-cost` from `{RECORDED}/*.json`, gated by \
         `check-ffi-cost`. Do not edit. Each record is written by `cargo xtask measure-ffi-cost`, \
         which builds the library in release and runs each binding's harness in turn \
         (`03-design/ffi-cost.md`).\n"
    );
    let _ = writeln!(
        out,
        "Every binding times the same calls. `positions` is asked for 1 instant × 3 bodies and \
         for 366 instants × 10 bodies, first over the library's own test provider and then over \
         a **hosted** provider, one written in the binding's own language (in Rust, behind a \
         vtable) that answers every cell with a fixed longitude, so its work is the columns and \
         nothing it computes. A call includes reading the answer's grid size, so a lazy decode \
         is paid as a consumer pays it, and `decode` is the decoder alone on the grid's bytes. \
         Rust is the floor: **marshalling** is a binding's `call-native` over Rust's, what its \
         request and its decoding add, and **callback** its `call-hosted` over Rust's, what \
         hosting a provider in it adds; the grid's is shared over its 3,660 cells. A difference \
         is printed only where the two figures' fastest-to-slowest ranges are apart, and \
         `within spread` where they meet. No budget binds these figures yet. A figure from \
         fewer than three runs, or whose median lies outside its own range, fails the gate, and \
         so does a record missing a key for a binding below or measured from a tree with \
         changes.\n"
    );
    for (binding, from) in BINDINGS {
        let _ = writeln!(out, "- **{binding}**: {from}");
    }
    out.push('\n');
    let found = faults(records);
    if !found.is_empty() {
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
    match recorded(root) {
        Ok(records) => Ok((vec![Output::new(PAGE, page(&records))], faults(&records))),
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
/// figure that is no claim or a record that lacks one.
pub(crate) fn check_generated(root: &Path) -> i32 {
    match outputs(root) {
        Ok((outputs, found)) => {
            for line in &found {
                println!("FAIL  {line}");
            }
            let page = check(root, &outputs, "cargo xtask ffi-cost");
            if found.is_empty() { page } else { 1 }
        }
        Err(code) => code,
    }
}

/// The JSON lines a harness printed, each a figure.
fn lines(binding: &str, output: &std::process::Output) -> Result<Vec<Timed>, String> {
    if !output.status.success() {
        return Err(format!(
            "the {binding} harness failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| line.starts_with('{'))
        .map(|line| {
            serde_json::from_str::<Timed>(line).map_err(|e| format!("{binding}: {line}: {e}"))
        })
        .collect()
}

/// Where the Rust floor writes the grid's blob, for the Java harness to
/// decode: that binding hands a consumer no call's raw bytes.
const GRID_BLOB: &str = "target/ffi-cost/grid.tsrb";

/// Runs every harness, one after another, and writes this machine's record
/// and the page.
pub(crate) fn measure(root: &Path) -> i32 {
    match harnesses(root) {
        Ok(record) => match crate::measured::write_record(root, RECORDED, &record) {
            Ok(()) => generate(root),
            Err(message) => {
                println!("FAIL  {message}");
                1
            }
        },
        Err(message) => {
            println!("FAIL  {message}");
            1
        }
    }
}

/// This machine's record: each binding's figures, from its harness, and
/// each runtime's version, in the order [`BINDINGS`] names them.
fn harnesses(root: &Path) -> Result<Record, String> {
    let run = |binding: &str, command: &mut Command| {
        command
            .output()
            .map_err(|e| format!("the {binding} harness did not start: {e}"))
            .and_then(|output| lines(binding, &output))
    };
    let unbuilt = |what: &str| format!("{what} did not build");
    let library = crate::binding::library(root).map_err(|()| unbuilt("the library"))?;
    let grid = root.join(GRID_BLOB);
    std::fs::create_dir_all(grid.parent().unwrap_or(root)).map_err(|e| e.to_string())?;
    let mut figures = run(
        "rust",
        Command::new(cargo())
            .args(["run", "--quiet", "--release", "-p", "teistro-ffi"])
            .args(["--example", "ffi_cost", "--"])
            .arg(&grid)
            .current_dir(root),
    )?;
    let mut runtimes = vec![(String::from("rust"), version("rustc", "--version"))];
    build(root, "teistro-node", "the Node addon").map_err(|()| unbuilt("the Node addon"))?;
    crate::node_binding::install_addon(root).map_err(|()| unbuilt("the Node addon"))?;
    figures.append(&mut run(
        "node",
        Command::new("node")
            .arg("bench/ffi-cost.mjs")
            .current_dir(root.join("bindings/node")),
    )?);
    runtimes.push((String::from("node"), version("node", "--version")));
    let python = crate::binding::python();
    figures.append(&mut run(
        "python",
        crate::binding::python_command(&python)
            .arg("bench/ffi_cost.py")
            .env("PYTHONPATH", root.join("bindings/python"))
            .env("TEISTRO_LIBRARY", &library)
            .current_dir(root.join("bindings/python")),
    )?);
    runtimes.push((String::from("python"), version(&python, "--version")));
    figures.append(&mut run(
        "dart",
        Command::new("dart")
            .args(["run", "bench/ffi_cost.dart"])
            .env("TEISTRO_LIBRARY", &library)
            .current_dir(root.join("bindings/dart")),
    )?);
    runtimes.push((String::from("dart"), version("dart", "--version")));
    crate::java_binding::compile_examples(root, &[root.join(crate::java_binding::BENCH)])
        .map_err(|()| unbuilt("the Java harness"))?;
    figures.append(&mut run(
        "java",
        crate::java_binding::example(root, "FfiCost", &library).arg(&grid),
    )?);
    runtimes.push((String::from("java"), version("java", "--version")));
    Ok(Record {
        os: String::from(std::env::consts::OS),
        arch: String::from(std::env::consts::ARCH),
        commit: crate::measured::commit(root),
        runtimes,
        figures,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn timed(binding: &str, key: &str, median: f64, min: f64, max: f64) -> Timed {
        Timed {
            binding: binding.into(),
            key: key.into(),
            runs: 5,
            calls: 1000,
            median_us: median,
            min_us: min,
            max_us: max,
        }
    }

    fn whole() -> Record {
        let figures = BINDINGS
            .iter()
            .flat_map(|(binding, _)| {
                KEYS.iter()
                    .map(|(key, _)| timed(binding, key, 2.0, 1.0, 3.0))
                    .collect::<Vec<_>>()
            })
            .collect();
        Record {
            os: String::from("o"),
            arch: String::from("a"),
            commit: String::from("c"),
            runtimes: Vec::new(),
            figures,
        }
    }

    #[test]
    fn a_whole_record_is_a_claim_and_a_missing_or_unheld_figure_is_named() {
        assert_eq!(
            faults(&[(String::from("x"), whole())]),
            Vec::<String>::new()
        );
        let mut lacking = whole();
        lacking
            .figures
            .retain(|t| !(t.binding == "node" && t.key == "decode"));
        let found = faults(&[(String::from("x"), lacking)]);
        assert_eq!(found.len(), 1);
        assert!(
            found
                .iter()
                .all(|line| line.contains("`decode` figure from node"))
        );
        let mut unheld = whole();
        if let Some(first) = unheld.figures.first_mut() {
            first.max_us = 1.5;
        }
        let found = faults(&[(String::from("x"), unheld)]);
        assert!(found.iter().any(|line| line.contains("a median needs")));
        let mut dirty = whole();
        dirty.commit = String::from("abc-dirty");
        let found = faults(&[(String::from("x"), dirty)]);
        assert!(
            found
                .iter()
                .any(|line| line.contains("a tree with changes"))
        );
    }

    #[test]
    fn a_binding_over_rust_is_a_number_only_where_the_ranges_are_apart() {
        let mut record = whole();
        for t in &mut record.figures {
            if t.binding == "node" && t.key == "call-hosted" {
                *t = timed("node", "call-hosted", 10.0, 9.0, 11.0);
            }
            if t.binding == "node" && t.key == "grid-hosted" {
                *t = timed("node", "grid-hosted", 3662.0, 3661.0, 3663.0);
            }
        }
        assert_eq!(over_rust(&record, "node", "call-hosted", 1.0), "8.00");
        assert_eq!(
            over_rust(&record, "node", "call-native", 1.0),
            "within spread"
        );
        assert_eq!(
            over_rust(&record, "rust", "call-hosted", 1.0),
            "—",
            "Rust is the floor"
        );
        assert_eq!(
            over_rust(&record, "node", "grid-hosted", GRID_CELLS),
            "3660 (1.00 a cell)"
        );
    }
}
