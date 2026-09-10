//! The falsification pass over **representing the Moon as a fitted
//! table** rather than as a theory.
//!
//! `lunar-accuracy-measured.md` found that ELP2000-82B holds a quarter of
//! an arcsecond at its own epoch and 19.4 arcseconds over the 1800 to
//! 2400 that `standard` claims, so `standard`'s lunar claim is
//! falsified and the obvious replacement is the one ADR-0021 already
//! names: a Chebyshev refit from a modern kernel.
//!
//! **That replacement has an arithmetic problem, and this pass is here to
//! settle whether it is fatal.** An analytic theory costs the same
//! whatever span it is asked for — ELP is 49 KB for six centuries,
//! because a periodic series does not know how long it will be
//! evaluated. A fitted table costs one block per interval, so its size
//! is proportional to the span, and the Moon is the fast body: it
//! circles in 27 days where Jupiter takes twelve years. ADR-0021 budgets
//! about 1 MB for every body at the `reference` tier. If the Moon alone
//! is many times that, the ladder in ADR-0021 has to move rather than
//! the tier.
//!
//! # What is fitted, and why it may be fitted to the theory
//!
//! The quantity swept is the geocentric rectangular position from
//! ELP2000-82B, not from the engine. That is deliberate and it is sound
//! for a **sizing** study: what a Chebyshev fit costs depends on how
//! smooth the function is, and the Moon of one modern ephemeris and the
//! Moon of a 1980s theory differ by a slowly varying drift of
//! arcseconds, which changes where the curve sits and not how wiggly it
//! is. The error reported is therefore the **representation** error of
//! the fit, which is the thing a table designer chooses, and it is
//! deliberately not the accuracy of the result.
//!
//! `cargo xtask chebyshev <dir>` writes the record the lunar page reads.

use std::f64::consts::PI;
use std::path::{Path, PathBuf};

use teistro_ephemeris_builtin::elp::ingest::{Theory, load, position};

use crate::generated::{Output, write};

/// Where the record goes; the lunar page reads it.
const RECORD: &str = "crates/ephemeris-builtin/data/moon-chebyshev.json";

/// Radians to arcseconds.
const ARCSEC_PER_RAD: f64 = 206_264.806_247_096_36;

/// The span the `standard` tier claims, as Julian days.
const FROM_JD: f64 = 2_378_497.0;
const TO_JD: f64 = 2_597_641.0;

/// Coefficients are `f64`, three per interval per degree.
const BYTES_PER_COEFFICIENT: usize = 8;

/// Fits one component over one interval and returns its coefficients.
///
/// Interpolation at Chebyshev points of the first kind, which is what a
/// kernel's own blocks use and what makes the fit near-optimal without
/// solving a system: the coefficients fall straight out of a cosine
/// transform of the samples.
fn fit(samples: &[f64]) -> Vec<f64> {
    let n = samples.len();
    let mut coefficients = vec![0.0; n];
    for (j, coefficient) in coefficients.iter_mut().enumerate() {
        let mut total = 0.0;
        for (k, sample) in samples.iter().enumerate() {
            #[expect(
                clippy::cast_precision_loss,
                reason = "a degree and a node index are small integers"
            )]
            let angle = PI * (k as f64 + 0.5) * (j as f64) / (n as f64);
            total += sample * angle.cos();
        }
        #[expect(clippy::cast_precision_loss, reason = "a degree is a small integer")]
        let scale = 2.0 / n as f64;
        *coefficient = total * scale;
    }
    if let Some(first) = coefficients.first_mut() {
        *first /= 2.0;
    }
    coefficients
}

/// Evaluates a Chebyshev series at `x` in `[-1, 1]` by Clenshaw's
/// recurrence, which is stable where the naive power form is not.
fn evaluate(coefficients: &[f64], x: f64) -> f64 {
    let mut b1 = 0.0;
    let mut b2 = 0.0;
    for coefficient in coefficients.iter().skip(1).rev() {
        let b0 = 2.0 * x * b1 - b2 + coefficient;
        b2 = b1;
        b1 = b0;
    }
    x * b1 - b2 + coefficients.first().copied().unwrap_or(0.0)
}

/// The node positions of a degree-`n` fit over `[-1, 1]`.
fn nodes(count: usize) -> Vec<f64> {
    (0..count)
        .map(|k| {
            #[expect(
                clippy::cast_precision_loss,
                reason = "a node index is a small integer"
            )]
            let angle = PI * (k as f64 + 0.5) / count as f64;
            angle.cos()
        })
        .collect()
}

/// One (interval, degree) pair's result.
#[derive(Debug, serde::Serialize)]
struct Row {
    interval_days: f64,
    coefficients: usize,
    worst_arcsec: f64,
    intervals_over_span: usize,
    bytes: usize,
}

/// The whole record.
#[derive(Debug, serde::Serialize)]
struct Record {
    source: &'static str,
    from_jd: f64,
    to_jd: f64,
    note: &'static str,
    rows: Vec<Row>,
}

/// Fits one interval and reports the worst angular error over it.
fn error_over(theory: &Theory, start: f64, length: f64, count: usize, probes: usize) -> f64 {
    let half = length / 2.0;
    let middle = start + half;
    let node_positions = nodes(count);
    let mut components: [Vec<f64>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    for x in &node_positions {
        let value = position(theory, middle + x * half, 0.0);
        for (column, component) in components.iter_mut().zip(value) {
            column.push(component);
        }
    }
    let fitted: Vec<Vec<f64>> = components.iter().map(|column| fit(column)).collect();

    let mut worst: f64 = 0.0;
    for probe in 0..=probes {
        #[expect(
            clippy::cast_precision_loss,
            reason = "a probe index is a small integer"
        )]
        let x = 2.0 * (probe as f64 / probes as f64) - 1.0;
        let truth = position(theory, middle + x * half, 0.0);
        let mut approximation = [0.0; 3];
        for (slot, coefficients) in approximation.iter_mut().zip(&fitted) {
            *slot = evaluate(coefficients, x);
        }
        // The angle between the two directions, which is what a chart
        // sees; a radial error moves no boundary.
        let cross = [
            truth[1] * approximation[2] - truth[2] * approximation[1],
            truth[2] * approximation[0] - truth[0] * approximation[2],
            truth[0] * approximation[1] - truth[1] * approximation[0],
        ];
        let length_of_cross = cross.iter().map(|c| c * c).sum::<f64>().sqrt();
        let dot: f64 = truth.iter().zip(approximation).map(|(a, b)| a * b).sum();
        worst = worst.max(length_of_cross.atan2(dot) * ARCSEC_PER_RAD);
    }
    worst
}

/// Intervals a kernel might plausibly use, against counts of
/// coefficients that reach from coarse to well past the arcsecond.
const INTERVALS: [f64; 4] = [4.0, 8.0, 16.0, 32.0];
const COUNTS: [usize; 6] = [8, 10, 12, 14, 16, 20];

/// Runs the sweep and writes the record.
pub(crate) fn generate(root: &Path, argument: Option<&str>) -> i32 {
    let Some(dir) = argument
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("TEISTRO_ELP_DIR").map(PathBuf::from))
    else {
        eprintln!(
            "no source directory: pass one as `cargo xtask chebyshev <dir>` or set \
             TEISTRO_ELP_DIR. The ELP2000-82B files are CDS catalogue VI/79 and are not \
             in the repository."
        );
        return 1;
    };
    let theory = match load(&dir) {
        Ok(theory) => theory,
        Err(error) => {
            eprintln!("{error}");
            return 1;
        }
    };

    let mut rows = Vec::new();
    for interval in INTERVALS {
        for count in COUNTS {
            // Three intervals across the span, at its ends and middle,
            // because the fit's difficulty follows the Moon's speed and
            // that is not the same everywhere.
            let worst = [FROM_JD, f64::midpoint(FROM_JD, TO_JD), TO_JD - interval]
                .into_iter()
                .map(|start| error_over(&theory, start, interval, count, 64))
                .fold(0.0_f64, f64::max);
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "the span in intervals is a positive count well inside usize"
            )]
            let intervals_over_span = ((TO_JD - FROM_JD) / interval).ceil() as usize;
            rows.push(Row {
                interval_days: interval,
                coefficients: count,
                worst_arcsec: worst,
                intervals_over_span,
                bytes: intervals_over_span * count * 3 * BYTES_PER_COEFFICIENT,
            });
        }
    }

    let record = Record {
        source: "ELP2000-82B, every term, as the function being represented",
        from_jd: FROM_JD,
        to_jd: TO_JD,
        note: "representation error of the fit, not accuracy against an ephemeris: a sizing study",
        rows,
    };
    let text = match serde_json::to_string_pretty(&record) {
        Ok(text) => format!("{text}\n"),
        Err(error) => {
            eprintln!("cannot serialise the record: {error}");
            return 1;
        }
    };
    write(root, &[Output::new(RECORD, text)])
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "a test fails by panicking and indexes its own fixtures"
    )]

    use super::*;

    /// A Chebyshev fit interpolates exactly at its own nodes, which is
    /// the property that makes the cosine transform the right one.
    #[test]
    fn the_fit_reproduces_its_samples() {
        let count = 10;
        let positions = nodes(count);
        // An arbitrary smooth function.
        let samples: Vec<f64> = positions.iter().map(|x| (3.0 * x).sin() + x * x).collect();
        let coefficients = fit(&samples);
        for (x, sample) in positions.iter().zip(&samples) {
            let got = evaluate(&coefficients, *x);
            assert!((got - sample).abs() < 1e-12, "{got} against {sample}");
        }
    }

    /// A polynomial below the fit's degree is represented exactly
    /// everywhere, not only at the nodes.
    #[test]
    fn a_low_polynomial_is_exact_between_the_nodes() {
        let samples: Vec<f64> = nodes(8).iter().map(|x| 2.0 * x * x - 1.0).collect();
        let coefficients = fit(&samples);
        for step in 0..=20 {
            let x = f64::from(step) / 10.0 - 1.0;
            let expected = 2.0 * x * x - 1.0;
            let got = evaluate(&coefficients, x);
            assert!(
                (got - expected).abs() < 1e-12,
                "at {x}: {got} against {expected}"
            );
        }
    }

    /// More coefficients cannot fit a smooth function worse. This is the
    /// property the sweep's whole shape rests on.
    #[test]
    fn more_coefficients_do_not_fit_worse() {
        let f = |x: f64| (4.0 * x).cos();
        let error_at = |count: usize| {
            let samples: Vec<f64> = nodes(count).iter().map(|x| f(*x)).collect();
            let coefficients = fit(&samples);
            (0..=50)
                .map(|step| {
                    let x = f64::from(step) / 25.0 - 1.0;
                    (evaluate(&coefficients, x) - f(x)).abs()
                })
                .fold(0.0_f64, f64::max)
        };
        let short = error_at(8);
        let long = error_at(16);
        assert!(long <= short, "a longer fit is at least as good");
        // The property that matters is not a threshold but the rate: for
        // a smooth function the error falls faster than any power of the
        // degree, which is what lets a modest block reach an arcsecond
        // and what the whole sweep rests on.
        assert!(
            long * 1e5 < short,
            "doubling the coefficients should gain orders, not a factor: {short} to {long}"
        );
    }

    #[test]
    fn the_nodes_are_inside_the_interval_and_ordered() {
        let positions = nodes(12);
        assert_eq!(positions.len(), 12);
        assert!(positions.iter().all(|x| *x > -1.0 && *x < 1.0));
        assert!(
            positions.windows(2).all(|pair| pair[0] > pair[1]),
            "the first-kind nodes descend from near +1"
        );
    }
}
