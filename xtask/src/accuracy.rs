//! The accuracy document: `accuracy` runs the astronomy layer's
//! measurement tests with `TEISTRO_ACCURACY_DIR` set, so each records its
//! worst difference against its recorded reference, and renders
//! `docs/05-testing/ACCURACY.md` from those measurements and the rows of
//! `docs/05-testing/accuracy-rows.yaml` (the areas, their targets, the
//! evidence and the by-hand measurements); `check-accuracy` renders in
//! memory and compares, so the checked-in document can never drift from
//! what the tests measure.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;
use std::process::Command;

use serde::Deserialize;

use crate::generated::{Output, check, write};

const ROWS: &str = "docs/05-testing/accuracy-rows.yaml";
const DOCUMENT: &str = "docs/05-testing/ACCURACY.md";
const MEASUREMENTS: &str = "measurements.jsonl";
const DIR_ENV: &str = "TEISTRO_ACCURACY_DIR";

#[derive(Deserialize)]
struct Rows {
    rows: Vec<Row>,
}

#[derive(Deserialize)]
struct Row {
    id: String,
    area: String,
    target: String,
    page: String,
    evidence: String,
    #[serde(default)]
    by_hand: String,
}

#[derive(Deserialize)]
struct Measurement {
    row: String,
    quantity: String,
    value: f64,
    unit: String,
    bound: f64,
    count: usize,
}

/// Runs the measurement tests and collects what they recorded.
fn measure(root: &Path) -> Vec<Measurement> {
    let dir = root.join("target").join("accuracy");
    std::fs::create_dir_all(&dir).expect("the accuracy directory");
    let file = dir.join(MEASUREMENTS);
    let _ = std::fs::remove_file(&file);
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let status = Command::new(cargo)
        .args(["test", "-p", "teistro-astro", "--tests", "--quiet"])
        .env(DIR_ENV, &dir)
        .current_dir(root)
        .status()
        .expect("cargo runs");
    assert!(status.success(), "the astronomy layer's tests failed");
    let text = std::fs::read_to_string(&file).unwrap_or_default();
    let mut measurements: Vec<Measurement> = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("a measurement line"))
        .collect();
    measurements.sort_by(|a, b| (&a.row, &a.quantity).cmp(&(&b.row, &b.quantity)));
    measurements
}

/// A difference with its unit, to two or three significant digits,
/// deterministic so the gate can compare texts.
fn quantity(value: f64, unit: &str) -> String {
    if value == 0.0 {
        return format!("0{unit}");
    }
    if (0.01..1000.0).contains(&value.abs()) {
        let text = format!("{value:.3}");
        let trimmed = text.trim_end_matches('0').trim_end_matches('.');
        return format!("{trimmed}{unit}");
    }
    format!("{value:.1e}{unit}")
}

fn render(rows: &Rows, measurements: &[Measurement]) -> String {
    let mut by_row: BTreeMap<&str, Vec<&Measurement>> = BTreeMap::new();
    for m in measurements {
        by_row.entry(m.row.as_str()).or_default().push(m);
    }
    let mut s = String::new();
    s.push_str("# Accuracy\n\n");
    s.push_str(
        "Status: `generated`, by `cargo xtask accuracy` from the measurement tests and\n\
         `accuracy-rows.yaml`, held by `cargo xtask check-accuracy`; do not edit.\n\n",
    );
    s.push_str(
        "Every area of the astronomy layer \
         (`01-research/platform/13-astronomy-layer.md`) with its conformance\n\
         target, what CI measures on every run (the tests under `crates/astro/tests`\n\
         against the recorded tables in `fixtures/teimeris/` and `fixtures/baseline/`,\n\
         the worst difference over every value compared, and the bound the test holds\n\
         it to), the measurements that need the reference engine present and are run\n\
         by hand with their date, and the evidence a reader can rerun. A difference\n\
         the SDK traced to the engine is in `02-engine-findings.md`; a convention\n\
         either side chose is in the cruxes register.\n\n",
    );
    s.push_str("| area | target | measured in CI | by hand | evidence | page |\n");
    s.push_str("|---|---|---|---|---|---|\n");
    for row in &rows.rows {
        let measured = by_row.get(row.id.as_str()).map_or_else(
            || "—".to_owned(),
            |list| {
                list.iter()
                    .map(|m| {
                        format!(
                            "{}: {} over {} values (bound {})",
                            m.quantity,
                            quantity(m.value, &m.unit),
                            m.count,
                            quantity(m.bound, &m.unit)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("; ")
            },
        );
        let by_hand = if row.by_hand.is_empty() {
            "—".to_owned()
        } else {
            row.by_hand.clone()
        };
        let _ = writeln!(
            s,
            "| {} | {} | {} | {} | {} | `{}` |",
            row.area, row.target, measured, by_hand, row.evidence, row.page
        );
    }
    s.push_str(
        "\nA measured value is the worst difference over every value the test compares; a\n\
                bound is what the test asserts, set from the measurement with its reason in\n\
                the test's source. `—` in the CI column: the area's evidence is a reference\n\
                test without a recorded engine table, a by-hand measurement, or the area is\n\
                not built yet.\n",
    );
    s
}

fn outputs(root: &Path) -> Vec<Output> {
    let rows: Rows = serde_yaml_ng::from_str(
        &std::fs::read_to_string(root.join(ROWS)).expect("the accuracy rows"),
    )
    .expect("valid accuracy rows");
    for row in &rows.rows {
        assert!(
            root.join("docs").join(&row.page).exists(),
            "{}: page {} does not exist",
            row.id,
            row.page
        );
    }
    let measurements = measure(root);
    for m in &measurements {
        assert!(
            rows.rows.iter().any(|row| row.id == m.row),
            "a test recorded a measurement for the unknown row {}",
            m.row
        );
    }
    vec![Output {
        path: DOCUMENT,
        text: render(&rows, &measurements),
    }]
}

pub(crate) fn generate(root: &Path) -> i32 {
    write(root, &outputs(root))
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    let failures = check(root, &outputs(root), "cargo xtask accuracy");
    i32::from(failures != 0)
}
