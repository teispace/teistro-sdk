//! From a verdict that is constant between edges to the runs between them
//! (X15): sample the window on a seed grid, bisect every pair of samples
//! whose verdicts differ until the change is pinned, judge each piece at
//! its middle, and merge neighbours that agree.
//!
//! An edge the grid cannot see, a clause that changes and changes back
//! between two samples, is missed; the seed step is chosen shorter than the
//! briefest run a clause holds, and the instants a stage knows change it
//! (a sunrise, a sunset) are passed in as edges of their own.

use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};

use crate::{EDGE_TOLERANCE_DAYS, Run, Verdict, Window};

/// The runs a window splits into, the edges between them, and how many
/// cells the seed grid had.
type Split = (Vec<Run>, Vec<JulianDay<Utc>>, u32);

/// The window's runs, the edges between them, and how many cells the seed
/// grid had.
pub(crate) fn runs<F>(
    window: Window,
    step_days: f64,
    known: &[f64],
    verdict: F,
) -> Result<Split, Error>
where
    F: Fn(JulianDay<Utc>) -> Result<Verdict, Error>,
{
    let judge = |at: f64| verdict(JulianDay::literal(at));
    let (from, to) = (window.from.get(), window.to.get());
    let cells = cells(window.days(), step_days);
    let mut cuts: Vec<f64> = known
        .iter()
        .copied()
        .filter(|at| from < *at && *at < to)
        .collect();
    let mut before = (from, judge(from)?);
    for cell in 1..=cells {
        let at = if cell == cells {
            to
        } else {
            from + step_days * f64::from(cell)
        };
        let after = (at, judge(at)?);
        bisect(&judge, before.clone(), after.clone(), &mut cuts)?;
        before = after;
    }
    cuts.sort_by(f64::total_cmp);
    cuts.dedup_by(|a, b| *a - *b < EDGE_TOLERANCE_DAYS);

    let mut runs: Vec<Run> = Vec::with_capacity(cuts.len() + 1);
    let mut start = from;
    for end in cuts.iter().copied().chain(std::iter::once(to)) {
        if end - start < EDGE_TOLERANCE_DAYS {
            continue;
        }
        let verdict = judge(f64::midpoint(start, end))?;
        match runs.last_mut() {
            Some(last) if last.verdict == verdict => last.to = JulianDay::literal(end),
            _ => runs.push(Run {
                from: JulianDay::literal(start),
                to: JulianDay::literal(end),
                verdict,
            }),
        }
        start = end;
    }
    let edges = runs.iter().skip(1).map(|run| run.from).collect();
    Ok((runs, edges, cells))
}

/// How many seed cells a window of so many days takes: at least one.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a window of at most a day and a half over a step of at least a second is \
              at most 129 600 cells, positive"
)]
fn cells(days: f64, step_days: f64) -> u32 {
    ((days / step_days).ceil() as u32).max(1)
}

/// Pins every change between two samples whose verdicts differ.
fn bisect<J>(
    judge: &J,
    before: (f64, Verdict),
    after: (f64, Verdict),
    cuts: &mut Vec<f64>,
) -> Result<(), Error>
where
    J: Fn(f64) -> Result<Verdict, Error>,
{
    if before.1 == after.1 {
        return Ok(());
    }
    if after.0 - before.0 <= EDGE_TOLERANCE_DAYS {
        cuts.push(f64::midpoint(before.0, after.0));
        return Ok(());
    }
    let middle = f64::midpoint(before.0, after.0);
    let at = (middle, judge(middle)?);
    bisect(judge, before, at.clone(), cuts)?;
    bisect(judge, at, after, cuts)
}
