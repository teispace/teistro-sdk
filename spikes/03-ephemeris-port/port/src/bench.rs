//! One timing helper for every Rust measurement of the spike: the median
//! of the best of `rounds` rounds, in microseconds, with the 90th
//! percentile, the same statistic the JavaScript and Dart harnesses of
//! spike 2 report.

use std::fmt::Write;
use std::time::Instant;

use serde::Serialize;

/// One measurement.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Row {
    /// What was measured.
    pub name: String,
    /// The median of the best round, microseconds.
    pub median_us: f64,
    /// The 90th percentile of the best round, microseconds.
    pub p90_us: f64,
    /// Timed calls per round.
    pub iterations: usize,
    /// Rounds.
    pub rounds: usize,
}

/// Times `f` over `rounds` rounds of `iterations` calls after `warmup`
/// calls each, keeping the round with the lowest median.
pub fn bench<F: FnMut()>(
    name: &str,
    iterations: usize,
    warmup: usize,
    rounds: usize,
    mut f: F,
) -> Row {
    let mut best: Option<Row> = None;
    for _ in 0..rounds.max(1) {
        for _ in 0..warmup {
            f();
        }
        let mut times: Vec<f64> = (0..iterations.max(1))
            .map(|_| {
                let start = Instant::now();
                f();
                start.elapsed().as_secs_f64() * 1e6
            })
            .collect();
        times.sort_by(f64::total_cmp);
        let at = |q: f64| {
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                clippy::cast_precision_loss,
                reason = "a quantile index"
            )]
            let i = ((q * times.len() as f64) as usize).min(times.len().saturating_sub(1));
            times.get(i).copied().unwrap_or_default()
        };
        let row = Row {
            name: name.to_string(),
            median_us: at(0.5),
            p90_us: at(0.9),
            iterations: iterations.max(1),
            rounds: rounds.max(1),
        };
        if best.as_ref().is_none_or(|b| row.median_us < b.median_us) {
            best = Some(row);
        }
    }
    best.unwrap_or(Row {
        name: name.to_string(),
        median_us: 0.0,
        p90_us: 0.0,
        iterations,
        rounds,
    })
}

/// The rows as a Markdown table.
#[must_use]
pub fn markdown(rows: &[Row]) -> String {
    let mut out = String::from("| measurement | median µs | p90 µs |\n|---|---:|---:|\n");
    for row in rows {
        let _ = writeln!(
            out,
            "| {} | {:.2} | {:.2} |",
            row.name, row.median_us, row.p90_us
        );
    }
    out
}
