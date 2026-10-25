//! `docs/05-testing/ab-measured.md`: the SDK through Node against the
//! baseline engine (`03-design/ab-baseline.md`), rendered from the records
//! `cargo xtask measure-ab` writes, one file per machine in
//! `docs/05-testing/ab/`, and the gate on them.
//!
//! A record carries the interleaved requests (§2, §3), the throughput on
//! threads and processes (§4), the tail of a sustained run (§5), the cold
//! start (§6) and the memory (§7). Every figure is held to the results
//! schema; the requests, the throughput and the tail carry the claim that
//! the SDK is not slower, and the cold start and the memory are compared
//! without one (§8).
//!
//! The baseline engine is never named here: the runner imports an adapter
//! from the path `TEISTRO_AB_BASELINE` names, and the record keeps the
//! version string the adapter reports.

use std::fmt::Write as _;
use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};

mod accuracy;

use crate::generated::{Output, check, write};
use crate::measured::{Comparison, Figure, ratio, us, version};

const PAGE: &str = "docs/05-testing/ab-measured.md";
const RECORDED: &str = "docs/05-testing/ab";
const CORPUS: &str = "fixtures/baseline/charts";

/// The corpus's tags under which a side may refuse a request, each with
/// words the refusal's reason must hold. At an edge of the data's years an
/// ephemeris's span ends and a day's spans run past it, whatever is asked.
/// At a polar birth a rule that needs a sunrise may refuse a day on which
/// the Sun does not rise, and nothing else.
const EXCUSES: [(&str, &str); 4] = [
    ("range-start", ""),
    ("range-end", ""),
    ("polar-day", SUNLESS),
    ("polar-night", SUNLESS),
];

/// What a refusal for want of a sunrise says.
const SUNLESS: &str = "where the Sun does not rise";

/// A birth of the corpus on which a refusal is excused, with the words its
/// reason must hold.
type Excused = (String, &'static str);

/// The two sides, in the order the page prints them.
const SIDES: [&str; 2] = ["sdk", "baseline"];

/// The fewest rounds a side's figure is taken from: the runner's
/// `ROUNDS`, since a figure from fewer was not taken the way the page says.
const ROUNDS: usize = 15;

/// The requests both sides answer, and what each asks: the runner's
/// `REQUESTS`, in its order.
const REQUESTS: [(&str, &str); 14] = [
    (
        "chart",
        "a birth chart with every varga, its drishti, upagrahas, houses, dignities, arudhas and \
         Vimshottari to depth 3",
    ),
    (
        "day",
        "the almanac day of the birth's date at its place, its limbs, kaalas, choghadiya and horas",
    ),
    (
        "full",
        "the whole natal chart: every section and varga, 22 dashas to depth 3, every shipped rule \
         and the longevity",
    ),
    (
        "vargas",
        "the chart's foundation, every divisional chart with it",
    ),
    ("yogas", "the yogas and doshas of a chart"),
    ("strength", "the Shadbala, Bhava bala and Ashtakavarga"),
    ("dashas", "eighteen dasha systems to depth 3"),
    ("month", "thirty almanac days from the birth's date"),
    ("milan", "the birth matched with the next one of the corpus"),
    (
        "varsha",
        "the first annual chart, its yogas, sahams and Mudda dasha",
    ),
    (
        "kp",
        "the KP cusps, sub lords, significators and ruling planets",
    ),
    (
        "gochar",
        "the grahas thirty years on over the chart, with the Ashtakavarga",
    ),
    ("sadeSati", "Sade Sati and the dhaiyyas over thirty years"),
    ("batch", "ten whole charts an hour apart, at one place"),
];

/// The requests driven under load and for their tail: the runner's
/// `LOAD_REQUESTS`.
const LOAD_REQUESTS: [&str; 3] = ["chart", "day", "full"];

/// The two ways a load runs, and the fewest trials a count's figure is
/// taken from: the runner's `MODES` and `TRIALS`.
const MODES: [&str; 2] = ["threads", "processes"];
const TRIALS: usize = 3;

/// The fewest windows a percentile is taken from, and the percentiles: the
/// runner's `TAIL_WINDOWS` and `PERCENTILES`.
const TAIL_WINDOWS: usize = 10;
const PERCENTILES: [u32; 3] = [50, 95, 99];

/// The cold-start phases, and the fewest starts each is taken from: the
/// runner's `STARTS`.
const PHASES: [(&str, &str); 3] = [
    (
        "open",
        "from the process's start to the side being ready, the import and the engine or context \
         included",
    ),
    ("first", "the first chart"),
    ("process", "the whole process, from spawn to exit"),
];
const STARTS: usize = 9;

/// The memory measures, the fewest processes each is taken from, and the
/// requests each process answers: the runner's `MEMORY_RUNS` and
/// `MEMORY_REQUESTS`.
const MEASURES: [(&str, &str); 3] = [
    ("peak-rss", "the peak resident set over the process's life"),
    (
        "heap",
        "the heap and external bytes, settled, after the workload",
    ),
    ("heap-growth", "their growth over the workload"),
];
const MEMORY_RUNS: usize = 3;
const MEMORY_REQUESTS: usize = 10_000;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Timed {
    side: String,
    version: String,
    request: String,
    rounds: usize,
    inputs: usize,
    median_us: f64,
    min_us: f64,
    max_us: f64,
}

impl Timed {
    fn figure(&self) -> Figure {
        Figure {
            runs: self.rounds,
            median: self.median_us,
            min: self.min_us,
            max: self.max_us,
        }
    }
}

/// A request a side refused for one birth, which leaves that request's
/// timing.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct Refused {
    id: String,
    side: String,
    request: String,
    reason: String,
}

/// One side's throughput at a load count, in requests a second.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct Load {
    side: String,
    request: String,
    mode: String,
    threads: usize,
    trials: usize,
    seconds: f64,
    median_rps: f64,
    min_rps: f64,
    max_rps: f64,
}

impl Load {
    fn figure(&self) -> Figure {
        Figure {
            runs: self.trials,
            median: self.median_rps,
            min: self.min_rps,
            max: self.max_rps,
        }
    }
}

/// A load count a side could not be opened or asked at, and why.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct Unloaded {
    side: String,
    request: String,
    mode: String,
    threads: usize,
    reason: String,
}

/// One percentile of a sustained run, over its windows, in microseconds.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct Tail {
    side: String,
    request: String,
    percentile: u32,
    seconds: f64,
    windows: usize,
    requests: usize,
    median_us: f64,
    min_us: f64,
    max_us: f64,
}

impl Tail {
    fn figure(&self) -> Figure {
        Figure {
            runs: self.windows,
            median: self.median_us,
            min: self.min_us,
            max: self.max_us,
        }
    }
}

/// One phase of a cold start, over the starts, in milliseconds.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct Cold {
    side: String,
    phase: String,
    runs: usize,
    median_ms: f64,
    min_ms: f64,
    max_ms: f64,
}

impl Cold {
    fn figure(&self) -> Figure {
        Figure {
            runs: self.runs,
            median: self.median_ms,
            min: self.min_ms,
            max: self.max_ms,
        }
    }
}

/// One memory measure over the processes, in its `unit`.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct Memory {
    side: String,
    request: String,
    measure: String,
    unit: String,
    runs: usize,
    requests: usize,
    median_value: f64,
    min_value: f64,
    max_value: f64,
}

impl Memory {
    fn figure(&self) -> Figure {
        Figure {
            runs: self.runs,
            median: self.median_value,
            min: self.min_value,
            max: self.max_value,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    os: String,
    arch: String,
    commit: String,
    node: String,
    /// The births the corpus holds; a figure over fewer is no claim.
    corpus: usize,
    /// The births on which the two sides computed different skies.
    differ: Vec<String>,
    /// The requests a side refused, by birth.
    refused: Vec<Refused>,
    figures: Vec<Timed>,
    /// The threads the machine offered, which names the load's counts; 0
    /// in a record taken before the load was measured.
    #[serde(default)]
    parallelism: usize,
    #[serde(default)]
    load: Vec<Load>,
    #[serde(default)]
    unloaded: Vec<Unloaded>,
    #[serde(default)]
    tail: Vec<Tail>,
    #[serde(default)]
    cold: Vec<Cold>,
    #[serde(default)]
    memory: Vec<Memory>,
    /// The two sides' answers compared, and each held to a reference
    /// (§10, §11).
    #[serde(default)]
    accuracy: accuracy::Accuracy,
}

impl Record {
    fn figure(&self, side: &str, request: &str) -> Option<&Timed> {
        self.figures
            .iter()
            .find(|t| t.side == side && t.request == request)
    }

    fn load(&self, side: &str, request: &str, mode: &str, threads: usize) -> Option<&Load> {
        self.load.iter().find(|l| {
            l.side == side && l.request == request && l.mode == mode && l.threads == threads
        })
    }

    fn unloaded(&self, side: &str, request: &str, mode: &str, threads: usize) -> Option<&Unloaded> {
        self.unloaded.iter().find(|u| {
            u.side == side && u.request == request && u.mode == mode && u.threads == threads
        })
    }

    fn tail(&self, side: &str, request: &str, percentile: u32) -> Option<&Tail> {
        self.tail
            .iter()
            .find(|t| t.side == side && t.request == request && t.percentile == percentile)
    }

    fn cold(&self, side: &str, phase: &str) -> Option<&Cold> {
        self.cold
            .iter()
            .find(|c| c.side == side && c.phase == phase)
    }

    fn memory(&self, side: &str, measure: &str) -> Option<&Memory> {
        self.memory
            .iter()
            .find(|m| m.side == side && m.measure == measure)
    }

    /// The births a request was refused for, on either side, each once.
    fn refused_for(&self, request: &str) -> Vec<&str> {
        let mut ids: Vec<&str> = self
            .refused
            .iter()
            .filter(|r| r.request == request)
            .map(|r| r.id.as_str())
            .collect();
        ids.sort_unstable();
        ids.dedup();
        ids
    }

    /// The load counts the record's machine names: the runner's `counts`.
    fn counts(&self) -> Vec<usize> {
        counts(self.parallelism)
    }
}

/// 1, 2, 4, … up to `most`, and `most` itself; none for a machine that
/// offered none.
fn counts(most: usize) -> Vec<usize> {
    let mut out: Vec<usize> = std::iter::successors(Some(1usize), |n| n.checked_mul(2))
        .take_while(|n| *n <= most)
        .collect();
    if most > 0 && out.last() != Some(&most) {
        out.push(most);
    }
    out
}

/// Why a figure is no claim, as a sentence naming what it is, or `None`.
fn schema(name: &str, what: &str, figure: &Figure, unit: &str) -> Option<String> {
    figure
        .fault()
        .map(|fault| format!("{name}: {what}, in {unit}, is {fault}"))
}

/// A sentence when `figure` was taken from fewer than `least`, or `None`.
fn too_few(name: &str, what: &str, taken: usize, least: usize, of: &str) -> Option<String> {
    (taken < least)
        .then(|| format!("{name}: {what} is {taken} {of}, under the {least} the page says"))
}

/// Why each record is no claim, or where the SDK is measurably slower.
/// `excused` are the corpus's births a side may refuse, and why.
fn faults(records: &[(String, Record)], excused: &[Excused]) -> Vec<String> {
    let mut found = Vec::new();
    for (name, record) in records {
        found.extend(crate::measured::dirty(name, &record.commit));
        request_faults(name, record, excused, &mut found);
        load_faults(name, record, &mut found);
        tail_faults(name, record, &mut found);
        start_and_memory_faults(name, record, &mut found);
        accuracy::faults(name, &record.accuracy, excused, &mut found);
    }
    found
}

/// The interleaved requests' faults (§2, §3).
fn request_faults(name: &str, record: &Record, excused: &[Excused], found: &mut Vec<String>) {
    let forgiven = |r: &Refused| {
        excused
            .iter()
            .any(|(id, words)| *id == r.id && r.reason.contains(words))
    };
    for r in record.refused.iter().filter(|r| !forgiven(r)) {
        found.push(format!(
            "{name}: the {} refused `{}` for {}, which the corpus does not excuse there: {}",
            r.side, r.request, r.id, r.reason
        ));
    }
    if !record.differ.is_empty() {
        found.push(format!(
            "{name}: the two sides computed different skies for {}; a timing over the rest is not \
             the corpus",
            record.differ.join(", ")
        ));
    }
    for t in &record.figures {
        let what = format!("`{}` from the {}", t.request, t.side);
        found.extend(schema(name, &what, &t.figure(), "µs"));
        found.extend(too_few(name, &what, t.rounds, ROUNDS, "rounds"));
        let refused = record.refused_for(&t.request).len();
        if t.inputs + refused < record.corpus {
            found.push(format!(
                "{name}: {what} is over {} of the corpus's {} births, and {refused} were refused",
                t.inputs, record.corpus
            ));
        }
    }
    for (request, _) in REQUESTS {
        match (
            record.figure("sdk", request),
            record.figure("baseline", request),
        ) {
            (Some(sdk), Some(baseline)) => {
                if let Comparison::Measured(r) = ratio(&sdk.figure(), &baseline.figure()) {
                    if r > 1.0 {
                        found.push(format!(
                            "{name}: `{request}` through the SDK is {r:.2}× the baseline's, {} µs \
                             against {} µs, and the ranges are apart",
                            us(sdk.median_us),
                            us(baseline.median_us)
                        ));
                    }
                }
            }
            _ => found.push(format!(
                "{name}: `{request}` lacks a side; re-record with `cargo xtask measure-ab`"
            )),
        }
    }
}

/// The throughput's faults (§4). A side may be unmeasured only in worker
/// threads, and only the baseline: the SDK is held to answering there.
fn load_faults(name: &str, record: &Record, found: &mut Vec<String>) {
    if record.parallelism == 0 {
        found.push(format!(
            "{name}: names no parallelism, so no load was measured; re-record with `cargo xtask \
             measure-ab`"
        ));
    }
    for l in &record.load {
        let what = format!(
            "`{}` on {} {} from the {}",
            l.request, l.threads, l.mode, l.side
        );
        found.extend(schema(name, &what, &l.figure(), "requests a second"));
        found.extend(too_few(name, &what, l.trials, TRIALS, "trials"));
    }
    for u in &record.unloaded {
        if u.side != "baseline" || u.mode != "threads" {
            found.push(format!(
                "{name}: the {} could not be measured on {} {} for `{}`, and only the baseline \
                 may be unmeasured, and only in threads: {}",
                u.side, u.threads, u.mode, u.request, u.reason
            ));
        }
    }
    for request in LOAD_REQUESTS {
        for mode in MODES {
            for threads in record.counts() {
                let [sdk, baseline] = SIDES.map(|side| record.load(side, request, mode, threads));
                for (side, load) in SIDES.iter().zip([sdk, baseline]) {
                    if load.is_none() && record.unloaded(side, request, mode, threads).is_none() {
                        found.push(format!(
                            "{name}: `{request}` on {threads} {mode} lacks the {side}, and the \
                             record does not say why"
                        ));
                    }
                }
                if let (Some(sdk), Some(baseline)) = (sdk, baseline) {
                    if let Comparison::Measured(r) = ratio(&baseline.figure(), &sdk.figure()) {
                        if r > 1.0 {
                            found.push(format!(
                                "{name}: `{request}` on {threads} {mode} answers {} a second \
                                 through the SDK against the baseline's {}, and the ranges are \
                                 apart",
                                us(sdk.median_rps),
                                us(baseline.median_rps)
                            ));
                        }
                    }
                }
            }
        }
    }
}

/// The tail's faults (§5).
fn tail_faults(name: &str, record: &Record, found: &mut Vec<String>) {
    for t in &record.tail {
        let what = format!("p{} of `{}` from the {}", t.percentile, t.request, t.side);
        found.extend(schema(name, &what, &t.figure(), "µs"));
        found.extend(too_few(name, &what, t.windows, TAIL_WINDOWS, "windows"));
    }
    for request in LOAD_REQUESTS {
        for p in PERCENTILES {
            match (
                record.tail("sdk", request, p),
                record.tail("baseline", request, p),
            ) {
                (Some(sdk), Some(baseline)) => {
                    if let Comparison::Measured(r) = ratio(&sdk.figure(), &baseline.figure()) {
                        if r > 1.0 {
                            found.push(format!(
                                "{name}: p{p} of `{request}` through the SDK is {r:.2}× the \
                                 baseline's, {} µs against {} µs, and the ranges are apart",
                                us(sdk.median_us),
                                us(baseline.median_us)
                            ));
                        }
                    }
                }
                _ => found.push(format!("{name}: p{p} of `{request}` lacks a side")),
            }
        }
    }
}

/// The cold start's and the memory's faults (§6, §7): the schema and the
/// counts, and no claim.
fn start_and_memory_faults(name: &str, record: &Record, found: &mut Vec<String>) {
    for side in SIDES {
        for (phase, _) in PHASES {
            let what = format!("the cold start's `{phase}` from the {side}");
            match record.cold(side, phase) {
                Some(c) => {
                    found.extend(schema(name, &what, &c.figure(), "ms"));
                    found.extend(too_few(name, &what, c.runs, STARTS, "starts"));
                }
                None => found.push(format!("{name}: lacks {what}")),
            }
        }
        for (measure, _) in MEASURES {
            let what = format!("the memory's `{measure}` from the {side}");
            match record.memory(side, measure) {
                Some(m) => {
                    found.extend(schema(name, &what, &m.figure(), &m.unit));
                    found.extend(too_few(name, &what, m.runs, MEMORY_RUNS, "processes"));
                    found.extend(too_few(
                        name,
                        &what,
                        m.requests,
                        MEMORY_REQUESTS,
                        "requests",
                    ));
                }
                None => found.push(format!("{name}: lacks {what}")),
            }
        }
    }
}

/// A figure as a page prints it: its median, and its fastest to slowest.
fn cell(figure: Option<Figure>) -> (String, String) {
    figure.map_or_else(
        || (String::from("not measured"), String::new()),
        |f| (us(f.median), format!("{} to {}", us(f.min), us(f.max))),
    )
}

/// `left ÷ right` as a page prints it, or `—` when either is missing.
fn rendered_ratio(left: Option<Figure>, right: Option<Figure>) -> String {
    match (left, right) {
        (Some(l), Some(r)) => ratio(&l, &r).render(|value| format!("{value:.2}×")),
        _ => String::from("—"),
    }
}

/// One table row of a SDK figure against a baseline figure.
fn row(out: &mut String, label: &str, sdk: Option<Figure>, baseline: Option<Figure>) {
    let (sdk_median, sdk_range) = cell(sdk);
    let (baseline_median, baseline_range) = cell(baseline);
    let _ = writeln!(
        out,
        "| {label} | {sdk_median} | {baseline_median} | {} | {sdk_range} | {baseline_range} |",
        rendered_ratio(sdk, baseline)
    );
}

fn header(out: &mut String, first: &str) {
    let _ = writeln!(
        out,
        "| {first} | SDK | baseline | SDK ÷ baseline | SDK fastest to slowest | baseline fastest to slowest |"
    );
    let _ = writeln!(out, "|---|---:|---:|---:|---:|---:|");
}

/// The share of linear throughput a count keeps: its median over the one
/// thread's times the count.
fn efficiency(record: &Record, side: &str, request: &str, mode: &str, threads: usize) -> String {
    match (
        record.load(side, request, mode, threads),
        record.load(side, request, mode, 1),
    ) {
        (Some(at), Some(one)) if one.median_rps > 0.0 => {
            #[expect(
                clippy::cast_precision_loss,
                reason = "a thread count is far below 2^52"
            )]
            let linear = one.median_rps * threads as f64;
            format!("{:.2}", at.median_rps / linear)
        }
        _ => String::from("—"),
    }
}

fn section(name: &str, record: &Record) -> String {
    let mut out = String::new();
    let versions: Vec<String> = SIDES
        .iter()
        .filter_map(|side| {
            record
                .figures
                .iter()
                .find(|t| t.side == *side)
                .map(|t| format!("the {side} `{}`", t.version))
        })
        .collect();
    let _ = writeln!(
        out,
        "## `{name}`\n\n{} {}, commit `{}`, Node {}, {} threads offered; {}.\n",
        record.os,
        record.arch,
        record.commit,
        record.node,
        record.parallelism,
        versions.join(", ")
    );
    requests_table(&mut out, record);
    throughput_table(&mut out, record);
    tail_table(&mut out, record);
    start_and_memory_tables(&mut out, record);
    accuracy::tables(&mut out, &record.accuracy);
    out
}

/// The interleaved requests (§2, §3), and what was refused.
fn requests_table(out: &mut String, record: &Record) {
    let _ = writeln!(out, "### Requests, per request in µs\n");
    header(out, "request");
    for (request, asks) in REQUESTS {
        let [sdk, baseline] = SIDES.map(|side| record.figure(side, request).map(Timed::figure));
        row(out, &format!("{request} ({asks})"), sdk, baseline);
    }
    if !record.refused.is_empty() {
        let _ = writeln!(out, "\nRefused, and kept out of that request's timing:\n");
        for r in &record.refused {
            let _ = writeln!(
                out,
                "- {}, `{}`, by the {}: {}",
                r.id, r.request, r.side, r.reason
            );
        }
    }
}

/// The throughput on threads and processes (§4), and what went unmeasured.
fn throughput_table(out: &mut String, record: &Record) {
    let _ = writeln!(
        out,
        "\n### Throughput, in requests a second\n\nAll threads or processes together, each \
         with its side of its own; the ratio is the SDK's throughput over the baseline's, and \
         efficiency the share of linear throughput the count keeps.\n"
    );
    let _ = writeln!(
        out,
        "| request | mode | at once | SDK | baseline | SDK ÷ baseline | SDK fastest to slowest | baseline fastest to slowest | SDK efficiency | baseline efficiency |"
    );
    let _ = writeln!(out, "|---|---|---:|---:|---:|---:|---:|---:|---:|---:|");
    for request in LOAD_REQUESTS {
        for mode in MODES {
            for threads in record.counts() {
                let [sdk, baseline] =
                    SIDES.map(|side| record.load(side, request, mode, threads).map(Load::figure));
                let (sdk_median, sdk_range) = cell(sdk);
                let (baseline_median, baseline_range) = cell(baseline);
                let _ = writeln!(
                    out,
                    "| {request} | {mode} | {threads} | {sdk_median} | {baseline_median} | {} | \
                     {sdk_range} | {baseline_range} | {} | {} |",
                    rendered_ratio(sdk, baseline),
                    efficiency(record, "sdk", request, mode, threads),
                    efficiency(record, "baseline", request, mode, threads)
                );
            }
        }
    }
    if !record.unloaded.is_empty() {
        let _ = writeln!(out, "\nNot measured, and why:\n");
        for u in &record.unloaded {
            let _ = writeln!(
                out,
                "- the {}, `{}` on {} {}: {}",
                u.side, u.request, u.threads, u.mode, u.reason
            );
        }
    }
}

/// The tail of a sustained run (§5).
fn tail_table(out: &mut String, record: &Record) {
    let requests: Vec<String> = LOAD_REQUESTS
        .iter()
        .filter_map(|request| {
            let [sdk, baseline] = SIDES.map(|side| {
                record
                    .tail(side, request, PERCENTILES[0])
                    .map(|t| t.requests)
            });
            Some(format!(
                "`{request}` {} through the SDK and {} through the baseline",
                sdk?, baseline?
            ))
        })
        .collect();
    let _ = writeln!(
        out,
        "\n### The tail of a sustained run, per answer in µs\n\nOne thread, each side in a \
         process of its own; a percentile is the median of the windows' with the fastest and \
         slowest. Answers: {}.\n",
        if requests.is_empty() {
            String::from("none recorded")
        } else {
            requests.join("; ")
        }
    );
    header(out, "percentile");
    for request in LOAD_REQUESTS {
        for p in PERCENTILES {
            let [sdk, baseline] = SIDES.map(|side| record.tail(side, request, p).map(Tail::figure));
            row(out, &format!("{request}, p{p}"), sdk, baseline);
        }
    }
}

/// The cold start and the memory (§6, §7), compared and not claimed.
fn start_and_memory_tables(out: &mut String, record: &Record) {
    let _ = writeln!(out, "\n### Cold start, in ms\n");
    header(out, "phase");
    for (phase, what) in PHASES {
        let [sdk, baseline] = SIDES.map(|side| record.cold(side, phase).map(Cold::figure));
        row(out, &format!("{phase} ({what})"), sdk, baseline);
    }
    let _ = writeln!(
        out,
        "\n### Memory over {MEMORY_REQUESTS} `chart` requests, after 200 to warm\n"
    );
    header(out, "measure");
    for (measure, what) in MEASURES {
        let [sdk, baseline] = SIDES.map(|side| record.memory(side, measure));
        let unit = sdk.or(baseline).map_or("", |m| m.unit.as_str());
        row(
            out,
            &format!("{measure} ({what}), {unit}"),
            sdk.map(Memory::figure),
            baseline.map(Memory::figure),
        );
    }
}

fn page(records: &[(String, Record)], excused: &[Excused]) -> String {
    let mut out = String::from("# The SDK against the baseline engine\n\n");
    let _ = writeln!(
        out,
        "Status: `generated` by `cargo xtask ab` from `{RECORDED}/*.json`, gated by `check-ab`. \
         Do not edit. Each record is written by `cargo xtask measure-ab`, which runs \
         `bindings/node/bench/ab.mjs` with the baseline engine's adapter \
         (`03-design/ab-baseline.md`).\n"
    );
    let _ = writeln!(
        out,
        "Both sides answer every birth of the golden corpus in one Node process, the SDK as \
         `parashari-classical` over Teimeris (`kp-default` for `kp`), each request asking the SDK \
         for at least what the baseline's answer holds. A birth on which the two Suns stand more \
         than an arcminute apart is reported and kept out, and a record that kept any out is no \
         claim. A request a side refuses is kept out of that request alone, and only at the \
         corpus's range-edge births, where the data's years end and a day's spans run past them, \
         or at its polar births for a rule that needs a sunrise on a day without one. \
         A round times every birth on one side and then the other, the next round the other way \
         round, and a figure is the median of at least {ROUNDS} rounds, per birth in \
         microseconds.\n"
    );
    let _ = writeln!(
        out,
        "Then each side, in processes of its own, answers `chart`, `day` and `full` under load on \
         1, 2, 4 and every thread the machine offers, in worker threads and in processes, at \
         least {TRIALS} trials a count; for 30 seconds on one thread, cut into {TAIL_WINDOWS} \
         windows for the tail; from a fresh process {STARTS} times for the cold start; and \
         {MEMORY_REQUESTS} charts in each of {MEMORY_RUNS} processes for the memory, the heap read \
         after collecting until two reads agree.\n"
    );
    let _ = writeln!(
        out,
        "**The claim** is that the SDK is not slower: the gate fails a request or a percentile \
         where its median is the larger, and a throughput where it is the smaller, when the two \
         fastest-to-slowest ranges are apart. Where they meet the ratio reads `within spread`. \
         The cold start and the memory are compared and not claimed (§8). Only the baseline may \
         go unmeasured, and only in worker threads, with the reason recorded.\n"
    );
    let _ = writeln!(
        out,
        "**Agreement and accuracy:** over the same inputs each answer is read into facts and the \
         two sides' compared field by field, under the SDK's timed profiles and under the ones \
         that cast as the baseline casts, and each side is held to a reference neither side \
         computed where one exists (§10, §11). The gate holds every agreement and reference \
         share to the floor the record measured, and each request's named cause to what the \
         record shows.\n"
    );
    let found = faults(records, excused);
    if found.is_empty() {
        let _ = writeln!(
            out,
            "No request, throughput or percentile is measurably slower through the SDK.\n"
        );
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

/// The corpus's births a side may refuse (`EXCUSES`), by id.
fn excused(root: &Path) -> Result<Vec<Excused>, String> {
    let dir = root.join(CORPUS);
    let entries = std::fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut ids = Vec::new();
    for entry in entries {
        let path = entry.map_err(|e| format!("{}: {e}", dir.display()))?.path();
        if path.extension().is_none_or(|ext| ext != "json") {
            continue;
        }
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let chart: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        let tags = chart.get("tags").and_then(serde_json::Value::as_array);
        let words = EXCUSES.iter().find_map(|(tag, words)| {
            tags.is_some_and(|tags| tags.iter().any(|t| t.as_str() == Some(tag)))
                .then_some(*words)
        });
        if let (Some(words), Some(id)) =
            (words, chart.get("id").and_then(serde_json::Value::as_str))
        {
            ids.push((id.to_string(), words));
        }
    }
    ids.sort();
    Ok(ids)
}

fn outputs(root: &Path) -> Result<(Vec<Output>, Vec<String>), i32> {
    let read = crate::measured::records::<Record>(root, RECORDED, "cargo xtask measure-ab")
        .and_then(|records| Ok((records, excused(root)?)));
    match read {
        Ok((records, excused)) => Ok((
            vec![Output::new(PAGE, page(&records, &excused))],
            faults(&records, &excused),
        )),
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
/// record that is no claim or a request measurably slower through the SDK.
pub(crate) fn check_generated(root: &Path) -> i32 {
    match outputs(root) {
        Ok((outputs, found)) => {
            for line in &found {
                println!("FAIL  {line}");
            }
            let page = check(root, &outputs, "cargo xtask ab");
            if found.is_empty() { page } else { 1 }
        }
        Err(code) => code,
    }
}

/// What the runner printed: its figures, and the births it kept out and
/// why.
#[derive(Default)]
struct Lines {
    figures: Vec<Timed>,
    differ: Vec<String>,
    refused: Vec<Refused>,
    corpus: usize,
    parallelism: usize,
    load: Vec<Load>,
    unloaded: Vec<Unloaded>,
    tail: Vec<Tail>,
    cold: Vec<Cold>,
    memory: Vec<Memory>,
    accuracy: accuracy::Accuracy,
}

/// One line, read as the type its kind names.
fn parsed<T: serde::de::DeserializeOwned>(
    line: &str,
    value: serde_json::Value,
) -> Result<T, String> {
    serde_json::from_value(value).map_err(|e| format!("{line}: {e}"))
}

fn read_lines(stdout: &str) -> Result<Lines, String> {
    let mut lines = Lines::default();
    for line in stdout.lines().filter(|line| line.starts_with('{')) {
        let mut value: serde_json::Value =
            serde_json::from_str(line).map_err(|e| format!("{line}: {e}"))?;
        if let Some(id) = value.get("differs").and_then(serde_json::Value::as_str) {
            lines.differ.push(id.to_string());
            continue;
        }
        let kind = value
            .get("kind")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned);
        match kind.as_deref() {
            Some("machine") => {
                lines.parallelism = value
                    .get("parallelism")
                    .and_then(serde_json::Value::as_u64)
                    .and_then(|n| usize::try_from(n).ok())
                    .unwrap_or(0);
            }
            Some("load") => lines.load.push(parsed(line, value)?),
            Some("unloaded") => lines.unloaded.push(parsed(line, value)?),
            Some("tail") => lines.tail.push(parsed(line, value)?),
            Some("cold") => lines.cold.push(parsed(line, value)?),
            Some("memory") => lines.memory.push(parsed(line, value)?),
            Some("agree") => lines.accuracy.agree.push(parsed(line, value)?),
            Some("reference") => lines.accuracy.reference.push(parsed(line, value)?),
            Some("unanswered") => lines.accuracy.unanswered.push(parsed(line, value)?),
            Some(other) => return Err(format!("{line}: no kind `{other}`")),
            None => {
                if let Some(object) = value.as_object_mut() {
                    if let Some(id) = object.remove("refused") {
                        object.insert(String::from("id"), id);
                        lines.refused.push(parsed(line, value)?);
                        continue;
                    }
                }
                lines.corpus = value
                    .get("corpus")
                    .and_then(serde_json::Value::as_u64)
                    .and_then(|n| usize::try_from(n).ok())
                    .unwrap_or(lines.corpus);
                lines.figures.push(parsed(line, value)?);
            }
        }
    }
    Ok(lines)
}

/// Runs the A/B when the adapter is named, and writes this machine's record
/// and the page.
pub(crate) fn measure(root: &Path) -> i32 {
    let (Ok(_), Ok(_), Ok(_), Ok(_)) = (
        std::env::var("TEISTRO_AB_BASELINE"),
        std::env::var("TEISTRO_TEIMERIS_ADAPTER"),
        std::env::var("TEISTRO_AB_PYTHON"),
        std::env::var("SE_EPHE_PATH"),
    ) else {
        println!(
            "FAIL  set TEISTRO_AB_BASELINE to the baseline engine's adapter, \
             TEISTRO_TEIMERIS_ADAPTER to the Teimeris library (`03-design/ab-baseline.md` §1), \
             TEISTRO_AB_PYTHON to a Python with pyswisseph and SE_EPHE_PATH to its files (§11)"
        );
        return 1;
    };
    if crate::binding::build(root, "teistro-node", "the Node addon").is_err()
        || crate::node_binding::install_addon(root).is_err()
    {
        return 1;
    }
    let output = Command::new("node")
        .arg("bench/ab.mjs")
        .current_dir(root.join("bindings/node"))
        .output();
    let parsed = match output {
        Ok(o) if o.status.success() => read_lines(&String::from_utf8_lossy(&o.stdout)),
        Ok(o) => Err(format!(
            "the A/B runner failed:\n{}",
            String::from_utf8_lossy(&o.stderr)
        )),
        Err(e) => Err(format!("the A/B runner did not start: {e}")),
    };
    let lines = match parsed {
        Ok(read) => read,
        Err(message) => {
            println!("FAIL  {message}");
            return 1;
        }
    };
    let record = Record {
        os: String::from(std::env::consts::OS),
        arch: String::from(std::env::consts::ARCH),
        commit: crate::measured::commit(root),
        node: version("node", "--version"),
        corpus: lines.corpus,
        differ: lines.differ,
        refused: lines.refused,
        figures: lines.figures,
        parallelism: lines.parallelism,
        load: lines.load,
        unloaded: lines.unloaded,
        tail: lines.tail,
        cold: lines.cold,
        memory: lines.memory,
        accuracy: lines.accuracy,
    };
    match crate::measured::write_record(root, RECORDED, &record) {
        Ok(()) => generate(root),
        Err(message) => {
            println!("FAIL  {message}");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn timed(side: &str, request: &str, median: f64, min: f64, max: f64) -> Timed {
        Timed {
            side: side.into(),
            version: format!("{side} 1"),
            request: request.into(),
            rounds: ROUNDS,
            inputs: 55,
            median_us: median,
            min_us: min,
            max_us: max,
        }
    }

    fn load(side: &str, request: &str, mode: &str, threads: usize, rps: f64) -> Load {
        Load {
            side: side.into(),
            request: request.into(),
            mode: mode.into(),
            threads,
            trials: TRIALS,
            seconds: 3.0,
            median_rps: rps,
            min_rps: rps * 0.95,
            max_rps: rps * 1.05,
        }
    }

    fn tail(side: &str, request: &str, percentile: u32, at: f64) -> Tail {
        Tail {
            side: side.into(),
            request: request.into(),
            percentile,
            seconds: 30.0,
            windows: TAIL_WINDOWS,
            requests: 1000,
            median_us: at,
            min_us: at * 0.9,
            max_us: at * 1.1,
        }
    }

    /// A record of two threads offered with every figure there, the SDK
    /// three times as fast as the baseline everywhere, and the baseline
    /// unmeasured in threads past one.
    fn record(figures: Vec<Timed>) -> Record {
        let mut whole = Record {
            os: "macos".into(),
            arch: "aarch64".into(),
            commit: "abc".into(),
            node: "v24".into(),
            corpus: 55,
            differ: Vec::new(),
            refused: Vec::new(),
            figures,
            parallelism: 2,
            load: Vec::new(),
            unloaded: Vec::new(),
            tail: Vec::new(),
            cold: Vec::new(),
            memory: Vec::new(),
            accuracy: accuracy::whole(),
        };
        for request in LOAD_REQUESTS {
            for mode in MODES {
                for threads in [1, 2] {
                    #[expect(clippy::cast_precision_loss, reason = "a small count")]
                    let n = threads as f64;
                    whole
                        .load
                        .push(load("sdk", request, mode, threads, 3000.0 * n));
                    if mode == "threads" && threads > 1 {
                        whole.unloaded.push(Unloaded {
                            side: "baseline".into(),
                            request: request.into(),
                            mode: mode.into(),
                            threads,
                            reason: "died of SIGSEGV".into(),
                        });
                    } else {
                        whole
                            .load
                            .push(load("baseline", request, mode, threads, 1000.0 * n));
                    }
                }
            }
            for p in PERCENTILES {
                whole.tail.push(tail("sdk", request, p, 100.0));
                whole.tail.push(tail("baseline", request, p, 300.0));
            }
        }
        for side in SIDES {
            for (phase, _) in PHASES {
                whole.cold.push(Cold {
                    side: side.into(),
                    phase: phase.into(),
                    runs: STARTS,
                    median_ms: 50.0,
                    min_ms: 40.0,
                    max_ms: 60.0,
                });
            }
            for (measure, _) in MEASURES {
                whole.memory.push(Memory {
                    side: side.into(),
                    request: "chart".into(),
                    measure: measure.into(),
                    unit: "MB".into(),
                    runs: MEMORY_RUNS,
                    requests: MEMORY_REQUESTS,
                    median_value: -1.0,
                    min_value: -2.0,
                    max_value: 3.0,
                });
            }
        }
        whole
    }

    fn refused(id: &str, request: &str) -> Refused {
        Refused {
            id: id.into(),
            side: "sdk".into(),
            request: request.into(),
            reason: "MOON at JD 1: OutOfRange".into(),
        }
    }

    /// Every request, the SDK three times as fast but for `day`, whose
    /// ranges meet.
    fn whole() -> Vec<Timed> {
        let mut figures = Vec::new();
        for (request, _) in REQUESTS {
            if request == "day" {
                figures.push(timed("sdk", request, 1500.0, 1400.0, 1600.0));
                figures.push(timed("baseline", request, 1550.0, 1450.0, 1700.0));
            } else {
                figures.push(timed("sdk", request, 1000.0, 900.0, 1100.0));
                figures.push(timed("baseline", request, 3000.0, 2800.0, 3300.0));
            }
        }
        figures
    }

    #[test]
    fn a_faster_or_overlapping_sdk_is_a_claim() {
        assert_eq!(
            faults(&[("m".into(), record(whole()))], &[]),
            Vec::<String>::new()
        );
        let page = page(&[("m".into(), record(whole()))], &[]);
        assert!(page.contains("| 0.33× |"), "{page}");
        assert!(page.contains("within spread"), "{page}");
        assert!(
            page.contains("| chart | threads | 2 | 6000 | not measured | — |"),
            "{page}"
        );
        assert!(page.contains("| 3.00× |"), "the throughput's ratio: {page}");
        assert!(page.contains("died of SIGSEGV"), "{page}");
    }

    #[test]
    fn a_measurably_slower_sdk_fails_and_so_does_a_partial_record() {
        let mut slow = whole();
        slow[0] = timed("sdk", "chart", 4000.0, 3800.0, 4200.0);
        let found = faults(&[("m".into(), record(slow))], &[]);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].contains("1.33×"), "{found:?}");

        let mut partial = record(whole());
        partial.differ = vec!["c007".into()];
        partial.figures[1].inputs = 54;
        partial.figures[2].rounds = 3;
        partial
            .figures
            .retain(|t| !(t.request == "day" && t.side == "baseline"));
        let found = faults(&[("m".into(), partial)], &[]);
        for needle in [
            "c007",
            "54 of the corpus's 55 births, and 0 were refused",
            "3 rounds",
            "`day` lacks a side",
        ] {
            assert!(
                found.iter().any(|f| f.contains(needle)),
                "{needle}: {found:?}"
            );
        }
    }

    #[test]
    fn a_refusal_counts_only_where_the_corpus_excuses_it() {
        let edges = vec![
            (String::from("c028"), SUNLESS),
            (String::from("c047"), ""),
            (String::from("c048"), ""),
        ];
        let day = |record: &mut Record, inputs: usize| {
            for t in record.figures.iter_mut().filter(|t| t.request == "day") {
                t.inputs = inputs;
            }
        };
        let mut at_edges = record(whole());
        at_edges.refused = vec![refused("c047", "day"), refused("c048", "day")];
        day(&mut at_edges, 53);
        assert_eq!(
            faults(&[("m".into(), at_edges)], &edges),
            Vec::<String>::new()
        );

        let mut inside = record(whole());
        inside.refused = vec![refused("c010", "day")];
        day(&mut inside, 54);
        let found = faults(&[("m".into(), inside)], &edges);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].contains("c010"), "{found:?}");
        assert!(found[0].contains("does not excuse"), "{found:?}");

        let sunless = |reason: &str| {
            let mut polar = record(whole());
            polar.refused = vec![Refused {
                reason: reason.into(),
                ..refused("c028", "day")
            }];
            day(&mut polar, 54);
            faults(&[("m".into(), polar)], &edges)
        };
        assert_eq!(
            sunless("the month-start rule needs a sunrise on RD 1, where the Sun does not rise"),
            Vec::<String>::new()
        );
        let found = sunless("MOON at JD 1: OutOfRange");
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].contains("c028"), "{found:?}");
    }

    #[test]
    fn a_throughput_or_a_tail_measurably_worse_through_the_sdk_fails() {
        let mut slow = record(whole());
        for l in slow.load.iter_mut().filter(|l| {
            l.side == "sdk" && l.request == "full" && l.mode == "processes" && l.threads == 2
        }) {
            *l = load("sdk", "full", "processes", 2, 1000.0);
        }
        for t in slow
            .tail
            .iter_mut()
            .filter(|t| t.side == "sdk" && t.request == "day" && t.percentile == 99)
        {
            *t = tail("sdk", "day", 99, 900.0);
        }
        let found = faults(&[("m".into(), slow)], &[]);
        assert_eq!(found.len(), 2, "{found:?}");
        assert!(
            found[0].contains("`full` on 2 processes answers 1000 a second"),
            "{found:?}"
        );
        assert!(
            found[1].contains("p99 of `day` through the SDK is 3.00×"),
            "{found:?}"
        );
    }

    #[test]
    fn only_the_baseline_may_go_unmeasured_and_only_in_threads() {
        let mut unexplained = record(whole());
        unexplained.unloaded.clear();
        let found = faults(&[("m".into(), unexplained)], &[]);
        assert_eq!(found.len(), LOAD_REQUESTS.len(), "{found:?}");
        assert!(found[0].contains("lacks the baseline, and the record does not say why"));

        let mut sdk_away = record(whole());
        sdk_away
            .load
            .retain(|l| !(l.side == "sdk" && l.request == "chart" && l.mode == "processes"));
        for threads in [1, 2] {
            sdk_away.unloaded.push(Unloaded {
                side: "sdk".into(),
                request: "chart".into(),
                mode: "processes".into(),
                threads,
                reason: "r".into(),
            });
        }
        let found = faults(&[("m".into(), sdk_away)], &[]);
        assert_eq!(found.len(), 2, "{found:?}");
        assert!(
            found
                .iter()
                .all(|f| f.contains("only the baseline may be unmeasured"))
        );
    }

    #[test]
    fn every_kind_of_figure_is_held_to_the_schema_and_its_counts() {
        let mut thin = record(whole());
        thin.parallelism = 0;
        thin.load.clear();
        thin.unloaded.clear();
        thin.tail[0].windows = 4;
        thin.cold[0].runs = 2;
        thin.cold[1].min_ms = 70.0;
        thin.memory[0].requests = 100;
        thin.memory.pop();
        let found = faults(&[("m".into(), thin)], &[]);
        for needle in [
            "names no parallelism",
            "p50 of `chart` from the sdk is 4 windows",
            "the cold start's `open` from the sdk is 2 starts",
            "the cold start's `first` from the sdk, in ms, is",
            "the memory's `peak-rss` from the sdk is 100 requests",
            "lacks the memory's `heap-growth` from the baseline",
        ] {
            assert!(
                found.iter().any(|f| f.contains(needle)),
                "{needle}: {found:?}"
            );
        }
    }

    #[test]
    fn the_load_counts_double_up_to_the_machine() {
        assert_eq!(counts(0), Vec::<usize>::new());
        assert_eq!(counts(1), [1]);
        assert_eq!(counts(4), [1, 2, 4]);
        assert_eq!(counts(12), [1, 2, 4, 8, 12]);
    }

    #[test]
    fn the_runner_s_lines_are_read_with_the_births_it_kept_out() {
        let stdout = "{\"differs\":\"c009\",\"sdk\":1,\"baseline\":2}\n\
            {\"refused\":\"c047\",\"side\":\"sdk\",\"request\":\"day\",\"reason\":\"r\"}\n\
            {\"side\":\"sdk\",\"version\":\"v\",\"request\":\"chart\",\"rounds\":15,\"inputs\":54,\
            \"median_us\":2,\"min_us\":1,\"max_us\":3,\"corpus\":55}\n\
            {\"kind\":\"machine\",\"parallelism\":12}\n\
            {\"kind\":\"load\",\"side\":\"sdk\",\"request\":\"chart\",\"mode\":\"threads\",\
            \"threads\":2,\"trials\":3,\"seconds\":3,\"median_rps\":2,\"min_rps\":1,\"max_rps\":3}\n\
            {\"kind\":\"unloaded\",\"side\":\"baseline\",\"request\":\"chart\",\"mode\":\"threads\",\
            \"threads\":2,\"reason\":\"died of SIGSEGV\"}\n\
            {\"kind\":\"tail\",\"side\":\"sdk\",\"request\":\"chart\",\"percentile\":99,\
            \"seconds\":30,\"windows\":10,\"requests\":9,\"median_us\":2,\"min_us\":1,\"max_us\":3}\n\
            {\"kind\":\"cold\",\"side\":\"sdk\",\"phase\":\"open\",\"runs\":9,\"median_ms\":2,\
            \"min_ms\":1,\"max_ms\":3}\n\
            {\"kind\":\"memory\",\"side\":\"sdk\",\"request\":\"chart\",\"measure\":\"heap\",\
            \"unit\":\"MB\",\"runs\":3,\"requests\":10000,\"median_value\":2,\"min_value\":1,\
            \"max_value\":3}\n";
        let lines = read_lines(stdout).expect("read");
        assert_eq!(
            (lines.figures.len(), lines.differ, lines.corpus),
            (1, vec![String::from("c009")], 55)
        );
        assert_eq!(lines.refused.len(), 1);
        assert_eq!(
            (
                lines.refused[0].id.as_str(),
                lines.refused[0].request.as_str()
            ),
            ("c047", "day")
        );
        assert_eq!(
            (
                lines.parallelism,
                lines.load.len(),
                lines.unloaded.len(),
                lines.tail.len(),
                lines.cold.len(),
                lines.memory.len()
            ),
            (12, 1, 1, 1, 1, 1)
        );
        assert!(read_lines("{\"kind\":\"weather\"}").is_err());
    }

    #[test]
    fn the_corpus_names_its_excused_births() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        assert_eq!(
            excused(&root).expect("corpus"),
            [
                (String::from("c028"), SUNLESS),
                (String::from("c029"), SUNLESS),
                (String::from("c047"), ""),
                (String::from("c048"), ""),
            ]
        );
    }
}
