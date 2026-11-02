//! From a value that is constant between edges to the runs between them
//! (X15): sample the window on a seed grid, bisect every pair of samples
//! whose values differ until the change is pinned, judge each piece at
//! its middle, and merge neighbours that agree.
//!
//! The purifier's verdict is one such value; the birth-time sensitivity's
//! set of facts is another (`sensitivity.rs`).
//!
//! An edge the grid cannot see, a value that changes and changes back
//! between two samples, is missed; the seed step is chosen shorter than the
//! briefest run a value holds, and the instants a stage knows change it
//! (a sunrise, a sunset) are passed in as edges of their own.

use teistro_core::error::Error;

use crate::{EDGE_TOLERANCE_DAYS, Window};

/// One run of a window: from, to (UTC Julian days) and the value every
/// instant of it shares.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Piece<V> {
    pub(crate) from: f64,
    pub(crate) to: f64,
    pub(crate) value: V,
}

/// The window's runs in order, and how many cells the seed grid had.
pub(crate) fn runs<V, F>(
    window: Window,
    step_days: f64,
    known: &[f64],
    value: F,
) -> Result<(Vec<Piece<V>>, u32), Error>
where
    V: Clone + PartialEq,
    F: Fn(f64) -> Result<V, Error>,
{
    let judge = value;
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

    let mut runs: Vec<Piece<V>> = Vec::with_capacity(cuts.len() + 1);
    let mut start = from;
    for end in cuts.iter().copied().chain(std::iter::once(to)) {
        if end - start < EDGE_TOLERANCE_DAYS {
            continue;
        }
        let value = judge(f64::midpoint(start, end))?;
        match runs.last_mut() {
            Some(last) if last.value == value => last.to = end,
            _ => runs.push(Piece {
                from: start,
                to: end,
                value,
            }),
        }
        start = end;
    }
    Ok((runs, cells))
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

/// Pins every change between two samples whose values differ.
pub(crate) fn bisect<V, J>(
    judge: &J,
    before: (f64, V),
    after: (f64, V),
    cuts: &mut Vec<f64>,
) -> Result<(), Error>
where
    V: Clone + PartialEq,
    J: Fn(f64) -> Result<V, Error>,
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
