//! The falsification pass over what a batch asks the ephemeris for,
//! which the SDK's batching, caching and parallelism are designed from.
//!
//! The architecture's own performance note says a batch of a thousand
//! charts should be "linear, parallelisable across contexts"
//! (`01-research/platform/07-performance.md`), and the port's shape says
//! a caller asks for a **grid** rather than a cell at a time
//! (`ephemeris-port-and-adapters.md` §3). Neither is a claim about the
//! layers built on top, and nothing had measured them.
//!
//! An ephemeris call is the expensive thing in this library — a file
//! read, a series evaluation, a lock — so the subject is **calls**, not
//! seconds. Counts are the same on every machine and for every provider,
//! which is what lets this page be gated at all; a timing would be noise
//! and could not be.
//!
//! `crates/panchanga/examples/batching` wraps the analytic provider in
//! [`CountingProvider`](teistro_port_ephemeris::CountingProvider) and
//! founds the three things the SDK offers — a grid of positions, a batch
//! of charts, a range of almanac days — at four sizes each. This reads
//! that and decides three questions: whether a batch is a batch, how much
//! of its work it asks for twice, and how wide the calls it makes are.
//!
//! `cargo xtask batching` writes the page; `check-batching` regenerates
//! it in memory and fails on any difference.

use std::fmt::Write as _;
use std::path::Path;
use std::process::Command;

use serde_json::Value;

use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict, count, fill, spelled, table, verdict_of};

const PAGE: &str = "docs/03-design/batch-and-parallelism-measured.md";

/// What the port names: `positions` and the seven declared overrides.
const PORT_OPERATIONS: u64 = 8;

/// What Teimeris names in its public header, counted by hand from
/// `core/include/teimeris/*.h` on 2026-09-09. The engine is not in this
/// workspace, so this is a recorded number rather than a computed one,
/// and the page says so.
const TEIMERIS_OPERATIONS: u64 = 177;

/// One measured operation at one size.
#[derive(Clone, Debug)]
struct Measured {
    operation: String,
    size: u64,
    calls: u64,
    cells: u64,
    distinct: u64,
    widest: u64,
    mean_width: f64,
    repeat_share: f64,
}

impl Measured {
    /// Calls per item of the batch: the number that says whether a batch
    /// is a batch.
    fn per_item(&self) -> f64 {
        if self.size == 0 {
            return 0.0;
        }
        #[allow(
            clippy::cast_precision_loss,
            reason = "a ratio of counts, for reporting"
        )]
        {
            self.calls as f64 / self.size as f64
        }
    }
}

pub(crate) fn generate(root: &Path) -> i32 {
    match page(root) {
        Ok(text) => write(root, &[Output::new(PAGE, text)]),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    match page(root) {
        Ok(text) => i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask batching") != 0),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

/// Runs the example and reads what it measured.
/// One almanac range measured with the memo off and on.
#[derive(Clone, Copy, Debug)]
struct Memo {
    size: u64,
    calls: u64,
    cells: u64,
    cached_calls: u64,
    cached_cells: u64,
    hit_share: f64,
}

fn measurements(root: &Path) -> Result<(Vec<Measured>, Vec<Memo>, Value), String> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| String::from("cargo"));
    let output = Command::new(cargo)
        .args([
            "run",
            "--quiet",
            "-p",
            "teistro-panchanga",
            "--example",
            "batching",
        ])
        .current_dir(root)
        .output()
        .map_err(|err| format!("cannot run the batching example: {err}"))?;
    if !output.status.success() {
        return Err(format!(
            "the batching example failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let value: Value = serde_json::from_str(&text)
        .map_err(|err| format!("the batching example did not print JSON: {err}"))?;
    let rows = value["measurements"]
        .as_array()
        .ok_or("the example prints an array of measurements")?
        .iter()
        .map(|row| {
            let calls = &row["calls"];
            Measured {
                operation: row["operation"].as_str().unwrap_or("?").to_string(),
                size: row["size"].as_u64().unwrap_or(0),
                calls: calls["positions"].as_u64().unwrap_or(0),
                cells: calls["cells"].as_u64().unwrap_or(0),
                distinct: calls["distinct_cells"].as_u64().unwrap_or(0),
                widest: calls["widest"].as_u64().unwrap_or(0),
                mean_width: calls["mean_width"].as_f64().unwrap_or(0.0),
                repeat_share: calls["repeat_share"].as_f64().unwrap_or(0.0),
            }
        })
        .collect();
    let mut memos: Vec<Memo> = Vec::new();
    for row in value["memo"].as_array().map_or(&[][..], Vec::as_slice) {
        let size = row["size"].as_u64().unwrap_or(0);
        let made = row["calls"]["positions"].as_u64().unwrap_or(0);
        let asked = row["calls"]["cells"].as_u64().unwrap_or(0);
        if row["memo"].as_bool().unwrap_or(false) {
            if let Some(memo) = memos.iter_mut().find(|memo| memo.size == size) {
                memo.cached_calls = made;
                memo.cached_cells = asked;
                memo.hit_share = row["hit_share"].as_f64().unwrap_or(0.0);
            }
        } else {
            memos.push(Memo {
                size,
                calls: made,
                cells: asked,
                cached_calls: 0,
                cached_cells: 0,
                hit_share: 0.0,
            });
        }
    }
    Ok((rows, memos, value["reach"].clone()))
}

/// The rows of one operation, in size order.
fn rows_of<'m>(rows: &'m [Measured], operation: &str) -> Vec<&'m Measured> {
    rows.iter()
        .filter(|row| row.operation == operation)
        .collect()
}

/// The largest size measured of an operation.
fn largest<'m>(rows: &'m [Measured], operation: &str) -> Option<&'m Measured> {
    rows_of(rows, operation).into_iter().last()
}

/// A count that came from a ratio, written as the house style writes a
/// number: rounded to the nearest, spaced from five digits.
fn rounded(value: f64) -> String {
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a count of calls, clamped into range before the cast"
    )]
    let whole = value.round().clamp(0.0, f64::from(u32::MAX)) as usize;
    count(whole)
}

/// A share as a percentage, to one place.
fn share(value: f64) -> String {
    format!("{:.1}%", value * 100.0)
}

fn page(root: &Path) -> Result<String, String> {
    let (rows, memos, reach) = measurements(root)?;
    let positions = largest(&rows, "positions").ok_or("the example measures a positions grid")?;
    let charts = largest(&rows, "charts").ok_or("the example measures a batch of charts")?;
    let almanac = largest(&rows, "almanac").ok_or("the example measures a range of days")?;
    let sunrise = largest(&rows, "sunrise").ok_or("the example measures one sunrise")?;

    let mut claims = Vec::new();
    claims.push(Claim::stated(
        "a batch of positions is one call whatever its size",
        verdict_of(positions.calls == 1),
        format!(
            "{} call for {} instants, {} cells wide",
            count(usize::try_from(positions.calls).unwrap_or_default()),
            count(usize::try_from(positions.size).unwrap_or_default()),
            count(usize::try_from(positions.widest).unwrap_or_default())
        ),
    ));
    for row in [charts, almanac] {
        claims.push(Claim::stated(
            format!("a batch of {} is one call whatever its size", row.operation),
            verdict_of(row.calls <= 1),
            format!(
                "{} calls for {}, {:.0} per item",
                count(usize::try_from(row.calls).unwrap_or_default()),
                count(usize::try_from(row.size).unwrap_or_default()),
                row.per_item()
            ),
        ));
    }
    claims.push(Claim::stated(
        "the calls a batch makes are grids rather than cells",
        verdict_of(charts.mean_width > 4.0 && almanac.mean_width > 4.0),
        format!(
            "a chart's calls are {:.2} cells wide on average and an almanac day's {:.2}; \
             the widest either makes is {} and {}",
            charts.mean_width,
            almanac.mean_width,
            count(usize::try_from(charts.widest).unwrap_or_default()),
            count(usize::try_from(almanac.widest).unwrap_or_default())
        ),
    ));
    claims.push(Claim::stated(
        "a batch asks for each cell once",
        verdict_of(charts.repeat_share < 0.01 && almanac.repeat_share < 0.01),
        format!(
            "{} of a batch of {} charts and {} of {} almanac days are cells already fetched",
            share(charts.repeat_share),
            count(usize::try_from(charts.size).unwrap_or_default()),
            share(almanac.repeat_share),
            count(usize::try_from(almanac.size).unwrap_or_default())
        ),
    ));
    if let Some(largest) = memos.last() {
        claims.push(Claim::stated(
            "a memo answers a repeated cell without touching the engine",
            verdict_of(largest.hit_share > 0.5 && largest.cached_cells < largest.cells),
            format!(
                "{} of a range of {} days is answered from memory: {} cells instead of {}",
                share(largest.hit_share),
                count(usize::try_from(largest.size).unwrap_or_default()),
                count(usize::try_from(largest.cached_cells).unwrap_or_default()),
                count(usize::try_from(largest.cells).unwrap_or_default())
            ),
        ));
    }
    let native = reach["native_operations"].as_u64().unwrap_or(0);
    claims.push(Claim::stated(
        "a consumer can reach what their engine offers beyond the port",
        verdict_of(native > 0),
        format!(
            "the port names {PORT_OPERATIONS} and an engine's own manifest is read \
             through it; \
             the provider measured here declares {} beyond them",
            count(usize::try_from(native).unwrap_or_default())
        ),
    ));

    Ok(fill(&text(
        &rows, &memos, &claims, positions, charts, almanac, sunrise,
    )))
}

#[allow(
    clippy::too_many_lines,
    reason = "one generated page, written in the order it reads"
)]
fn text(
    rows: &[Measured],
    memos: &[Memo],
    claims: &[Claim],
    positions: &Measured,
    charts: &Measured,
    almanac: &Measured,
    sunrise: &Measured,
) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "# What a batch asks the ephemeris for, measured\n\n\
         Status: `generated` by `cargo xtask batching` over the analytic\n\
         test provider. Do not edit: `check-batching` regenerates this page\n\
         and fails on any difference. The plan written from it is\n\
         [`../07-roadmap/02-plan-performance-and-passthrough.md`](../07-roadmap/02-plan-performance-and-passthrough.md).\n"
    );

    let _ = writeln!(
        out,
        "## 1. Why calls and not seconds\n\n\
         An ephemeris call is the expensive thing in this library: a file\n\
         read, a series evaluation, a lock. How long one takes belongs to\n\
         the engine and the machine — Teimeris answers in microseconds and\n\
         a JPL kernel over a network file system in milliseconds — but **how\n\
         many the SDK makes** belongs to the SDK, is the same everywhere,\n\
         and is therefore the only part of the performance a gate can hold.\n\
         Every number below is a count. Nothing here is a timing and\n\
         nothing here would change on another machine.\n"
    );
    let _ = writeln!(
        out,
        "The provider underneath is the analytic test one, which answers\n\
         instantly. That is deliberate: it makes the counts visible without\n\
         making them slow, and the counts are what a real engine multiplies\n\
         by its own cost.\n"
    );

    let _ = writeln!(out, "## 2. What each operation asks for\n");
    let mut measured = String::from(
        "| operation | size | calls | cells | distinct cells | widest call | mean width | calls per item |\n|---|---:|---:|---:|---:|---:|---:|---:|\n",
    );
    for row in rows {
        let _ = writeln!(
            measured,
            "| {} | {} | {} | {} | {} | {} | {:.2} | {:.1} |",
            row.operation,
            count(usize::try_from(row.size).unwrap_or_default()),
            count(usize::try_from(row.calls).unwrap_or_default()),
            count(usize::try_from(row.cells).unwrap_or_default()),
            count(usize::try_from(row.distinct).unwrap_or_default()),
            count(usize::try_from(row.widest).unwrap_or_default()),
            row.mean_width,
            row.per_item()
        );
    }
    let _ = writeln!(out, "{measured}");

    let _ = writeln!(
        out,
        "Read the first row and the last of each operation together. A\n\
         grid of positions is **one call** whether it holds one instant or\n\
         {}: that is what the port was shaped for, and it is the control\n\
         everything else is compared with. A batch of {} charts is {} calls\n\
         — {:.0} per chart, the same as one chart costs on its own — and a\n\
         range of {} almanac days is {}, or {:.0} a day. Neither is a batch\n\
         in any sense the ephemeris can see. They are loops that share a\n\
         provenance stamp.\n",
        count(usize::try_from(positions.size).unwrap_or_default()),
        count(usize::try_from(charts.size).unwrap_or_default()),
        count(usize::try_from(charts.calls).unwrap_or_default()),
        charts.per_item(),
        count(usize::try_from(almanac.size).unwrap_or_default()),
        count(usize::try_from(almanac.calls).unwrap_or_default()),
        almanac.per_item()
    );

    let _ = writeln!(
        out,
        "## 3. The calls are cells, not grids\n\n\
         The widest call a batch of charts makes is {} cells — the grahas of\n\
         one chart, asked for once — and the widest an almanac makes is {}.\n\
         Everything else is one instant and one or two bodies. The mean\n\
         width is {:.2} for charts and {:.2} for an almanac: the SDK asks its\n\
         ephemeris for **one cell at a time**, tens of thousands of times,\n\
         through a port whose one required operation takes a grid.\n",
        count(usize::try_from(charts.widest).unwrap_or_default()),
        count(usize::try_from(almanac.widest).unwrap_or_default()),
        charts.mean_width,
        almanac.mean_width
    );
    let _ = writeln!(
        out,
        "One sunrise is worth naming on its own: {} calls, every one of\n\
         them a single cell, none of them a repeat. Meeus's iteration\n\
         answers it, and it is serial by construction — each instant is\n\
         computed from the sample before it, so there is no grid to ask\n\
         for. A day's {} calls are **not** made of searches like it: an\n\
         attribution of every one of them to its caller (recorded in\n\
         [`../07-roadmap/02-plan-performance-and-passthrough.md`](../07-roadmap/02-plan-performance-and-passthrough.md),\n\
         A1) found 44% of them in the Moon's sign search, which reaches\n\
         forty days either side of the day because the constant that\n\
         sizes it was chosen for the Sun. The dominant cost is a window,\n\
         not a round trip.\n",
        count(usize::try_from(sunrise.calls).unwrap_or_default()),
        rounded(almanac.per_item())
    );

    let _ = writeln!(
        out,
        "## 4. How much of the work is asked for twice\n\n\
         The distinct column counts cells rather than calls: one instant,\n\
         one body, one frame, however many times it was asked for. The gap\n\
         between it and the cells column is what a memo would have\n\
         answered without touching the engine.\n"
    );
    let mut repeats = String::from("| operation | 1 item | largest batch |\n|---|---:|---:|\n");
    for operation in ["charts", "almanac"] {
        let rows = rows_of(rows, operation);
        let (Some(first), Some(last)) = (rows.first(), rows.last()) else {
            continue;
        };
        let _ = writeln!(
            repeats,
            "| {operation} | {} | {} of {} items |",
            share(first.repeat_share),
            share(last.repeat_share),
            count(usize::try_from(last.size).unwrap_or_default())
        );
    }
    let _ = writeln!(out, "{repeats}");
    let _ = writeln!(
        out,
        "The share **rises with the batch**, which is the finding. Within\n\
         one chart {} of the cells are asked for more than once; across {}\n\
         charts it is {}. That growth can only come from sharing *between*\n\
         the items — consecutive instants at one place scan overlapping\n\
         windows for the same sunrise, and read the same grahas at the same\n\
         instants — and it is exactly the sharing a batch exists to\n\
         exploit and this one does not.\n",
        share(
            rows_of(rows, "charts")
                .first()
                .map_or(0.0, |row| row.repeat_share)
        ),
        count(usize::try_from(charts.size).unwrap_or_default()),
        share(charts.repeat_share)
    );
    let _ = writeln!(
        out,
        "A memo is only sound over a provider that answers identical\n\
         requests with identical bits, and the port already asks every\n\
         provider to say whether it does: `Capabilities::deterministic`,\n\
         declared by both shipped adapters and read by nothing. It has a\n\
         reader now, and it is the right one — the cache is correct exactly\n\
         when that flag is true, and refuses to exist when it is not.\n"
    );

    let _ = writeln!(
        out,
        "## 5. What the memo actually saves\n\n\
         [`CachingProvider`](../../crates/port-ephemeris/src/caching.rs)\n\
         over the same batches, the cache off and on. Both arms run the\n\
         same code through the same types — a cache of nothing is a cache\n\
         that does nothing — so what separates the numbers is the memo and\n\
         nothing else, and the answers are identical cell for cell.\n"
    );
    let mut saved = String::from(
        "| days | calls | cells | calls, cached | cells, cached | answered from memory |\n\
         |---:|---:|---:|---:|---:|---:|\n",
    );
    for memo in memos {
        let _ = writeln!(
            saved,
            "| {} | {} | {} | {} | {} | {} |",
            count(usize::try_from(memo.size).unwrap_or_default()),
            count(usize::try_from(memo.calls).unwrap_or_default()),
            count(usize::try_from(memo.cells).unwrap_or_default()),
            count(usize::try_from(memo.cached_calls).unwrap_or_default()),
            count(usize::try_from(memo.cached_cells).unwrap_or_default()),
            share(memo.hit_share)
        );
    }
    let _ = writeln!(out, "{saved}");
    if let (Some(first), Some(last)) = (memos.first(), memos.last()) {
        let _ = writeln!(
            out,
            "The share answered from memory **rises with the batch** — {}\n\
             for a single day, {} across {} — which is the same finding as\n\
             §4 read from the other side, and the reason the memo is worth\n\
             more than a cache of one call's own repeats. A range of {}\n\
             days asks the ephemeris for {} cells instead of {}, in {}\n\
             calls instead of {}.\n",
            share(first.hit_share),
            share(last.hit_share),
            count(usize::try_from(last.size).unwrap_or_default()),
            count(usize::try_from(last.size).unwrap_or_default()),
            count(usize::try_from(last.cached_cells).unwrap_or_default()),
            count(usize::try_from(last.cells).unwrap_or_default()),
            count(usize::try_from(last.cached_calls).unwrap_or_default()),
            count(usize::try_from(last.calls).unwrap_or_default())
        );
    }

    let _ = writeln!(
        out,
        "## 6. What is reachable at all\n\n\
         The other half of the same question. A batch that asks well is\n\
         still limited to what it may ask for, and the port names \
         {PORT_OPERATIONS} operations: `positions` and seven declared\n\
         overrides. Teimeris's public header names\n\
         {TEIMERIS_OPERATIONS}. The difference — its eclipses, its stars\n\
         and orbits, its own calendar grids and chart blobs, its scans —\n\
         was unreachable through this SDK, and a consumer who wanted any\n\
         of it had to open a second handle to the same engine: the largest\n\
         dead end in the project.\n\n\
         **The route is built** — the port's `native_manifest` and\n\
         `native_call`, and `port_ephemeris::Native` over them. The SDK\n\
         holds no list of an engine's operations. It reads the manifest\n\
         the engine ships and relays a call by the name that manifest\n\
         gives, so an operation the engine gains *after* the SDK ships is\n\
         callable the day it appears. The count below is what the measured\n\
         provider declares, which is why it moves when an engine does and\n\
         not when the SDK does; the number for Teimeris arrives with its\n\
         adapter's own manifest.\n"
    );
    let _ = writeln!(
        out,
        "The count of Teimeris's operations is recorded rather than\n\
         computed: the engine is not in this workspace, so this pass cannot\n\
         count it on every run. It was counted from\n\
         `core/include/teimeris/*.h` on 2026-09-09 and is a floor, since\n\
         the same is true of every other engine an adapter might wrap.\n"
    );

    let _ = writeln!(out, "## 7. What this pass decides\n");
    let _ = writeln!(out, "{}", table(claims));
    let _ = writeln!(
        out,
        "{} of the {} claims are falsified, and they are falsified in an\n\
         order that matters. Threads would multiply the work rather than\n\
         reduce it while more than half of a range's cells are asked for\n\
         twice; the design fixes the arithmetic first and spends hardware\n\
         last, and the numbers on this page move as each step of it lands\n\
         ([`../07-roadmap/02-plan-performance-and-passthrough.md`](../07-roadmap/02-plan-performance-and-passthrough.md)).\n",
        spelled(
            claims
                .iter()
                .filter(|claim| claim.verdict == Verdict::Falsified)
                .count()
        ),
        spelled(claims.len())
    );
    out
}
