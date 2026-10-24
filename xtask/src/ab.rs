//! `docs/05-testing/ab-measured.md`: the SDK through Node against the
//! baseline engine (`03-design/ab-baseline.md`), rendered from the records
//! `cargo xtask measure-ab` writes, one file per machine in
//! `docs/05-testing/ab/`, and the gate on them.
//!
//! The baseline engine is never named here: the runner imports an adapter
//! from the path `TEISTRO_AB_BASELINE` names, and the record keeps the
//! version string the adapter reports.

use std::fmt::Write as _;
use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::generated::{Output, check, write};
use crate::measured::{Comparison, Figure, ratio, us, version};

const PAGE: &str = "docs/05-testing/ab-measured.md";
const RECORDED: &str = "docs/05-testing/ab";
const CORPUS: &str = "fixtures/baseline/charts";

/// The corpus's tags for a birth at an edge of the data's years, the only
/// births a side may refuse: there an ephemeris's span ends, and a day's
/// spans run past it.
const EDGES: [&str; 2] = ["range-start", "range-end"];

/// The fewest rounds a side's figure is taken from: the runner's
/// `ROUNDS`, since a figure from fewer was not taken the way the page says.
const ROUNDS: usize = 15;

/// The requests both sides answer, and what each asks.
const REQUESTS: [(&str, &str); 2] = [
    ("chart", "a birth chart with its Vimshottari to depth 3"),
    ("day", "the almanac day of the birth's date at its place"),
];

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
}

impl Record {
    fn figure(&self, side: &str, request: &str) -> Option<&Timed> {
        self.figures
            .iter()
            .find(|t| t.side == side && t.request == request)
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
}

/// Why each record is no claim, or where the SDK is measurably slower.
/// `edges` are the corpus's births at the data's edges, the only ones a
/// side may refuse.
fn faults(records: &[(String, Record)], edges: &[String]) -> Vec<String> {
    let mut found = Vec::new();
    for (name, record) in records {
        found.extend(crate::measured::dirty(name, &record.commit));
        for r in record.refused.iter().filter(|r| !edges.contains(&r.id)) {
            found.push(format!(
                "{name}: the {} refused `{}` for {}, which is not at an edge of the data's \
                 years: {}",
                r.side, r.request, r.id, r.reason
            ));
        }
        if !record.differ.is_empty() {
            found.push(format!(
                "{name}: the two sides computed different skies for {}; a timing over the rest \
                 is not the corpus",
                record.differ.join(", ")
            ));
        }
        for t in &record.figures {
            if let Some(fault) = t.figure().fault() {
                found.push(format!(
                    "{name}: `{}` from the {}, in µs, is {fault}",
                    t.request, t.side
                ));
            }
            if t.rounds < ROUNDS {
                found.push(format!(
                    "{name}: `{}` from the {} is {} rounds, under the {ROUNDS} the page says",
                    t.request, t.side, t.rounds
                ));
            }
            let refused = record.refused_for(&t.request).len();
            if t.inputs + refused < record.corpus {
                found.push(format!(
                    "{name}: `{}` from the {} is over {} of the corpus's {} births, and {refused} \
                     were refused",
                    t.request, t.side, t.inputs, record.corpus
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
                                "{name}: `{request}` through the SDK is {r:.2}× the baseline's, \
                                 {} µs against {} µs, and the ranges are apart",
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
    found
}

fn section(name: &str, record: &Record) -> String {
    let mut out = String::new();
    let versions: Vec<String> = ["sdk", "baseline"]
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
        "## `{name}`\n\n{} {}, commit `{}`, Node {}; {}.\n",
        record.os,
        record.arch,
        record.commit,
        record.node,
        versions.join(", ")
    );
    let _ = writeln!(
        out,
        "| request | SDK | baseline | SDK ÷ baseline | SDK fastest to slowest | baseline fastest to slowest |"
    );
    let _ = writeln!(out, "|---|---:|---:|---:|---:|---:|");
    for (request, asks) in REQUESTS {
        let (Some(sdk), Some(baseline)) = (
            record.figure("sdk", request),
            record.figure("baseline", request),
        ) else {
            let _ = writeln!(out, "| {request} ({asks}) | not recorded | | | | |");
            continue;
        };
        let _ = writeln!(
            out,
            "| {request} ({asks}) | {} | {} | {} | {} to {} | {} to {} |",
            us(sdk.median_us),
            us(baseline.median_us),
            ratio(&sdk.figure(), &baseline.figure()).render(|r| format!("{r:.2}×")),
            us(sdk.min_us),
            us(sdk.max_us),
            us(baseline.min_us),
            us(baseline.max_us)
        );
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
    out
}

fn page(records: &[(String, Record)], edges: &[String]) -> String {
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
         `parashari-classical` over Teimeris. A birth on which the two Suns stand more than an \
         arcminute apart is reported and kept out, and a record that kept any out is no claim. A \
         request a side refuses is kept out of that request alone, and only at the corpus's \
         range-edge births, where the data's years end and a day's spans run past them. A \
         round times every birth on one side and then the other, the next round the other way \
         round, and a figure is the median of at least {ROUNDS} rounds, per birth in \
         microseconds. **The claim** is that the SDK is not slower: the gate fails a request \
         where its median is the larger and the two fastest-to-slowest ranges are apart. Where \
         they meet the ratio reads `within spread`.\n"
    );
    let found = faults(records, edges);
    if found.is_empty() {
        let _ = writeln!(out, "No request is measurably slower through the SDK.\n");
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

/// The corpus's births tagged at an edge of the data's years, by id.
fn edges(root: &Path) -> Result<Vec<String>, String> {
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
        let at_edge = chart
            .get("tags")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|tags| {
                tags.iter()
                    .any(|t| t.as_str().is_some_and(|t| EDGES.contains(&t)))
            });
        if let (true, Some(id)) = (at_edge, chart.get("id").and_then(serde_json::Value::as_str)) {
            ids.push(id.to_string());
        }
    }
    ids.sort();
    Ok(ids)
}

fn outputs(root: &Path) -> Result<(Vec<Output>, Vec<String>), i32> {
    let read = crate::measured::records::<Record>(root, RECORDED, "cargo xtask measure-ab")
        .and_then(|records| Ok((records, edges(root)?)));
    match read {
        Ok((records, edges)) => Ok((
            vec![Output::new(PAGE, page(&records, &edges))],
            faults(&records, &edges),
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
struct Lines {
    figures: Vec<Timed>,
    differ: Vec<String>,
    refused: Vec<Refused>,
    corpus: usize,
}

fn read_lines(stdout: &str) -> Result<Lines, String> {
    let mut figures = Vec::new();
    let mut differ = Vec::new();
    let mut refused = Vec::new();
    let mut corpus = 0;
    for line in stdout.lines().filter(|line| line.starts_with('{')) {
        let mut value: serde_json::Value =
            serde_json::from_str(line).map_err(|e| format!("{line}: {e}"))?;
        if let Some(id) = value.get("differs").and_then(serde_json::Value::as_str) {
            differ.push(id.to_string());
            continue;
        }
        if let Some(object) = value.as_object_mut() {
            if let Some(id) = object.remove("refused") {
                object.insert(String::from("id"), id);
                refused.push(serde_json::from_value(value).map_err(|e| format!("{line}: {e}"))?);
                continue;
            }
        }
        corpus = value
            .get("corpus")
            .and_then(serde_json::Value::as_u64)
            .and_then(|n| usize::try_from(n).ok())
            .unwrap_or(corpus);
        figures.push(serde_json::from_value(value).map_err(|e| format!("{line}: {e}"))?);
    }
    Ok(Lines {
        figures,
        differ,
        refused,
        corpus,
    })
}

/// Runs the A/B when the adapter is named, and writes this machine's record
/// and the page.
pub(crate) fn measure(root: &Path) -> i32 {
    let (Ok(_), Ok(_)) = (
        std::env::var("TEISTRO_AB_BASELINE"),
        std::env::var("TEISTRO_TEIMERIS_ADAPTER"),
    ) else {
        println!(
            "FAIL  set TEISTRO_AB_BASELINE to the baseline engine's adapter and \
             TEISTRO_TEIMERIS_ADAPTER to the Teimeris library (`03-design/ab-baseline.md` §1)"
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

    fn record(figures: Vec<Timed>) -> Record {
        Record {
            os: "macos".into(),
            arch: "aarch64".into(),
            commit: "abc".into(),
            node: "v24".into(),
            corpus: 55,
            differ: Vec::new(),
            refused: Vec::new(),
            figures,
        }
    }

    fn refused(id: &str, request: &str) -> Refused {
        Refused {
            id: id.into(),
            side: "sdk".into(),
            request: request.into(),
            reason: "MOON at JD 1: OutOfRange".into(),
        }
    }

    fn whole() -> Vec<Timed> {
        vec![
            timed("sdk", "chart", 1000.0, 900.0, 1100.0),
            timed("baseline", "chart", 3000.0, 2800.0, 3300.0),
            timed("sdk", "day", 1500.0, 1400.0, 1600.0),
            timed("baseline", "day", 1550.0, 1450.0, 1700.0),
        ]
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
        partial.figures.pop();
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
    fn a_refusal_counts_only_at_an_edge_of_the_data() {
        let edges = vec![String::from("c047"), String::from("c048")];
        let mut at_edges = record(whole());
        at_edges.refused = vec![refused("c047", "day"), refused("c048", "day")];
        at_edges.figures[2].inputs = 53;
        at_edges.figures[3].inputs = 53;
        assert_eq!(
            faults(&[("m".into(), at_edges)], &edges),
            Vec::<String>::new()
        );

        let mut inside = record(whole());
        inside.refused = vec![refused("c010", "day")];
        inside.figures[2].inputs = 54;
        inside.figures[3].inputs = 54;
        let found = faults(&[("m".into(), inside)], &edges);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].contains("c010"), "{found:?}");
        assert!(found[0].contains("not at an edge"), "{found:?}");
    }

    #[test]
    fn the_runner_s_lines_are_read_with_the_births_it_kept_out() {
        let stdout = "{\"differs\":\"c009\",\"sdk\":1,\"baseline\":2}\n\
            {\"refused\":\"c047\",\"side\":\"sdk\",\"request\":\"day\",\"reason\":\"r\"}\n\
            {\"side\":\"sdk\",\"version\":\"v\",\"request\":\"chart\",\"rounds\":15,\"inputs\":54,\
            \"median_us\":2,\"min_us\":1,\"max_us\":3,\"corpus\":55}\n";
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
    }

    #[test]
    fn the_corpus_names_its_range_edge_births() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        assert_eq!(edges(&root).expect("corpus"), ["c047", "c048"]);
    }
}
