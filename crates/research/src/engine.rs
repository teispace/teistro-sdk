//! The permutation engine (`research.md` §§1.2, 1.5, 2.2, 2.3).
//!
//! Every permutation draws its own stream ([`SplitMix64::for_permutation`]),
//! shuffles the labels inside each stratum, and reads every predicate's
//! statistic off the bitsets. A float statistic is computed per
//! permutation in one fixed order; only the integer exceedance counts
//! cross threads, and an integer sum has no order, so the answer is the
//! same bits at every thread count.

use std::collections::BTreeMap;

use teistro_core::error::Error;

use crate::bits::{Bits, Column, Matrix};
use crate::rng::SplitMix64;
use crate::{Alternative, Contrast, Parallelism};

/// The relative margin inside which a permuted statistic counts as a
/// tie with the observed one. The statistics are built from integer
/// counts, so two labellings with the same counts give the same bits;
/// the margin is for two different counts whose statistics agree in
/// exact arithmetic and differ by a rounding, which must count as
/// "at least as extreme". Distinct values of a standardised count differ
/// by many orders of magnitude more than this.
const TIE: f64 = 1e-12;

/// Whether `permuted` is at least as extreme as `observed`.
pub(crate) fn reaches(permuted: f64, observed: f64) -> bool {
    permuted >= observed - TIE * observed.abs().max(1.0)
}

/// The labels and how they may move.
pub(crate) struct Layout<'a> {
    pub(crate) groups: &'a [u16],
    pub(crate) group_count: u16,
    /// The positions of each stratum, strata in their key order and the
    /// positions ascending within each.
    pub(crate) strata: Vec<Vec<usize>>,
}

impl<'a> Layout<'a> {
    pub(crate) fn new(groups: &'a [u16], group_count: u16, strata: Option<&[u32]>) -> Layout<'a> {
        let strata = match strata {
            None => vec![(0..groups.len()).collect()],
            Some(keys) => {
                let mut by_key: BTreeMap<u32, Vec<usize>> = BTreeMap::new();
                for (position, key) in keys.iter().enumerate() {
                    by_key.entry(*key).or_default().push(position);
                }
                by_key.into_values().collect()
            }
        };
        Layout {
            groups,
            group_count,
            strata,
        }
    }

    /// Writes permutation `k`'s labels into `labels`.
    pub(crate) fn permute(&self, seed: u64, k: u64, labels: &mut [u16], scratch: &mut Vec<u16>) {
        self.permute_with(&mut SplitMix64::for_permutation(seed, k), labels, scratch);
    }

    /// Writes the next draw of `rng`'s labels into `labels`: a rejected
    /// draw is followed by the same stream's next one.
    pub(crate) fn permute_with(
        &self,
        rng: &mut SplitMix64,
        labels: &mut [u16],
        scratch: &mut Vec<u16>,
    ) {
        if self.strata.len() == 1 {
            labels.copy_from_slice(self.groups);
            rng.shuffle(labels);
            return;
        }
        for positions in &self.strata {
            scratch.clear();
            scratch.extend(positions.iter().filter_map(|&p| self.groups.get(p)));
            rng.shuffle(scratch);
            for (&position, &label) in positions.iter().zip(scratch.iter()) {
                if let Some(slot) = labels.get_mut(position) {
                    *slot = label;
                }
            }
        }
    }
}

/// One group's counts on one predicate: how many of its charts the
/// predicate was read on, and how many it holds on.
#[derive(Clone, Copy)]
struct Share {
    read: u32,
    present: u32,
}

impl Share {
    fn of(mask: &Bits, column: &Column) -> Share {
        Share {
            read: mask.common(&column.readable),
            present: mask.common(&column.present),
        }
    }
}

/// The standardised case count: the count's departure from its mean
/// under the hypergeometric that holds the margins, in its standard
/// deviations. Zero where the predicate cannot vary (it holds on every
/// read chart or none, or one group has every read chart).
fn standardised(case: Share, column: &Column) -> f64 {
    let total = f64::from(column.readable_total);
    let marked = f64::from(column.present_total);
    let drawn = f64::from(case.read);
    if total < 2.0 {
        return 0.0;
    }
    let share = marked / total;
    let mean = drawn * share;
    let variance = drawn * share * (1.0 - share) * (total - drawn) / (total - 1.0);
    if variance <= 0.0 {
        return 0.0;
    }
    (f64::from(case.present) - mean) / variance.sqrt()
}

/// Pearson's chi-square on the groups-by-(present, absent) table, over
/// the charts the predicate was read on.
fn chi_square(shares: &[Share], column: &Column) -> f64 {
    let total = f64::from(column.readable_total);
    if total == 0.0 {
        return 0.0;
    }
    let share = f64::from(column.present_total) / total;
    if share <= 0.0 || share >= 1.0 {
        return 0.0;
    }
    shares
        .iter()
        .filter(|s| s.read > 0)
        .map(|s| {
            let read = f64::from(s.read);
            let departure = f64::from(s.present) - read * share;
            departure * departure / (read * share * (1.0 - share))
        })
        .sum()
}

/// The test statistic each predicate is ranked by, larger the more
/// extreme.
pub(crate) struct Statistic {
    pub(crate) contrast: Contrast,
    pub(crate) alternative: Alternative,
}

impl Statistic {
    /// Every predicate's statistic under the labels whose group masks are
    /// `masks`.
    fn read(&self, masks: &[Bits], matrix: &Matrix, out: &mut [f64], shares: &mut Vec<Share>) {
        for (column, slot) in matrix.columns.iter().zip(out.iter_mut()) {
            *slot = match self.contrast {
                Contrast::CaseVsRest { case } => {
                    let z = masks
                        .get(usize::from(case))
                        .map_or(0.0, |mask| standardised(Share::of(mask, column), column));
                    match self.alternative {
                        Alternative::Greater => z,
                        Alternative::Less => -z,
                        Alternative::TwoSided => z.abs(),
                    }
                }
                Contrast::AnyDifference => {
                    shares.clear();
                    shares.extend(masks.iter().map(|mask| Share::of(mask, column)));
                    chi_square(shares, column)
                }
            };
        }
    }
}

/// The masks a statistic reads: only the case's for a case-against-rest
/// contrast, every group's otherwise.
fn masks_for(contrast: Contrast, group_count: u16, charts: usize) -> Vec<Bits> {
    let count = match contrast {
        Contrast::CaseVsRest { case } => usize::from(case) + 1,
        Contrast::AnyDifference => usize::from(group_count),
    };
    vec![Bits::empty(charts); count]
}

fn fill_masks(labels: &[u16], masks: &mut [Bits]) {
    for mask in masks.iter_mut() {
        mask.clear();
    }
    for (index, &label) in labels.iter().enumerate() {
        if let Some(mask) = masks.get_mut(usize::from(label)) {
            mask.insert(index);
        }
    }
}

/// The observed statistics, under the labels as given.
pub(crate) fn observed(layout: &Layout<'_>, matrix: &Matrix, statistic: &Statistic) -> Vec<f64> {
    let mut masks = masks_for(statistic.contrast, layout.group_count, matrix.charts());
    fill_masks(layout.groups, &mut masks);
    let mut out = vec![0.0; matrix.predicates()];
    statistic.read(&masks, matrix, &mut out, &mut Vec::new());
    out
}

/// A design's family of statistics under each permutation: what the
/// counting loop asks of a group test and an event study alike.
pub(crate) trait Permuted: Sync {
    /// What one thread reuses from permutation to permutation.
    type Scratch;

    /// A fresh scratch.
    fn scratch(&self) -> Self::Scratch;

    /// Every predicate's statistic under permutation `k`, into `out`.
    ///
    /// # Errors
    ///
    /// Whatever stops permutation `k` being drawn (a restricted pairing
    /// that rejection cannot meet).
    fn read(&self, k: u64, scratch: &mut Self::Scratch, out: &mut [f64]) -> Result<(), Error>;
}

/// The labels of a group test, permuted inside their strata, and the
/// statistic read off them.
pub(crate) struct Labelled<'a> {
    pub(crate) layout: &'a Layout<'a>,
    pub(crate) matrix: &'a Matrix,
    pub(crate) statistic: &'a Statistic,
    pub(crate) seed: u64,
}

/// A group test's scratch: the permuted labels, the stratum buffer, the
/// group masks and the shares.
pub(crate) struct LabelScratch {
    labels: Vec<u16>,
    stratum: Vec<u16>,
    masks: Vec<Bits>,
    shares: Vec<Share>,
}

impl Permuted for Labelled<'_> {
    type Scratch = LabelScratch;

    fn scratch(&self) -> LabelScratch {
        let charts = self.matrix.charts();
        LabelScratch {
            labels: vec![0; charts],
            stratum: Vec::new(),
            masks: masks_for(self.statistic.contrast, self.layout.group_count, charts),
            shares: Vec::new(),
        }
    }

    fn read(&self, k: u64, scratch: &mut LabelScratch, out: &mut [f64]) -> Result<(), Error> {
        self.layout
            .permute(self.seed, k, &mut scratch.labels, &mut scratch.stratum);
        fill_masks(&scratch.labels, &mut scratch.masks);
        self.statistic
            .read(&scratch.masks, self.matrix, out, &mut scratch.shares);
        Ok(())
    }
}

/// What the permutations counted: per predicate, how many reached its
/// observed statistic, and per rank of the step-down order, how many
/// reached that rank's observed statistic with the running maximum over
/// it and every rank below.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Counted {
    pub(crate) exceed: Vec<u64>,
    pub(crate) step_down: Vec<u64>,
}

impl Counted {
    fn zero(predicates: usize) -> Counted {
        Counted {
            exceed: vec![0; predicates],
            step_down: vec![0; predicates],
        }
    }

    fn add(&mut self, other: &Counted) {
        for (a, b) in self.exceed.iter_mut().zip(&other.exceed) {
            *a += b;
        }
        for (a, b) in self.step_down.iter_mut().zip(&other.step_down) {
            *a += b;
        }
    }
}

/// The predicates in the step-down order: observed statistic descending,
/// ties in the request's order.
pub(crate) fn step_down_order(observed: &[f64]) -> Vec<usize> {
    let mut order: Vec<(usize, f64)> = observed.iter().copied().enumerate().collect();
    order.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    order.into_iter().map(|(index, _)| index).collect()
}

/// The observed family a count compares against.
pub(crate) struct Family<'a> {
    pub(crate) observed: &'a [f64],
    pub(crate) order: &'a [usize],
    /// The observed statistics in the step-down order.
    pub(crate) ordered: Vec<f64>,
}

impl<'a> Family<'a> {
    pub(crate) fn new(observed: &'a [f64], order: &'a [usize]) -> Family<'a> {
        Family {
            observed,
            order,
            ordered: order
                .iter()
                .filter_map(|&j| observed.get(j).copied())
                .collect(),
        }
    }
}

/// Counts permutations `range` (Westfall and Young, *Resampling-Based
/// Multiple Testing*, 1993, algorithm 4.1 for max-T).
fn count_range<P: Permuted>(
    permuted: &P,
    family: &Family<'_>,
    range: std::ops::Range<u64>,
) -> Result<Counted, Error> {
    let predicates = family.observed.len();
    let mut counted = Counted::zero(predicates);
    let mut scratch = permuted.scratch();
    let mut stats = vec![0.0; predicates];
    for k in range {
        permuted.read(k, &mut scratch, &mut stats)?;
        for ((slot, &stat), &obs) in counted.exceed.iter_mut().zip(&stats).zip(family.observed) {
            *slot += u64::from(reaches(stat, obs));
        }
        let mut running = f64::NEG_INFINITY;
        for ((slot, &j), &obs) in counted
            .step_down
            .iter_mut()
            .zip(family.order)
            .zip(&family.ordered)
            .rev()
        {
            running = running.max(stats.get(j).copied().unwrap_or(f64::NEG_INFINITY));
            *slot += u64::from(reaches(running, obs));
        }
    }
    Ok(counted)
}

/// Counts permutations `0..permutations`, split into contiguous ranges
/// over the threads `parallelism` names. A thread the platform cannot
/// start (wasm has none) runs its range on the calling thread instead,
/// which by construction counts the same; and when ranges fail, the
/// lowest one's error is the answer, whatever the thread count.
pub(crate) fn count<P: Permuted>(
    permuted: &P,
    family: &Family<'_>,
    permutations: u32,
    parallelism: Parallelism,
) -> Result<Counted, Error> {
    let m = u64::from(permutations);
    let threads = match parallelism {
        Parallelism::One => 1,
        Parallelism::Threads(n) => u64::from(n.get()).min(m.max(1)),
    };
    if threads <= 1 {
        return count_range(permuted, family, 0..m);
    }
    let ranges: Vec<std::ops::Range<u64>> = (0..threads)
        .map(|t| (m * t / threads)..(m * (t + 1) / threads))
        .collect();
    let parts: Vec<Result<Counted, Error>> = std::thread::scope(|scope| {
        let handles: Vec<_> = ranges
            .iter()
            .map(|range| {
                let range = range.clone();
                std::thread::Builder::new()
                    .spawn_scoped(scope, {
                        let range = range.clone();
                        move || count_range(permuted, family, range)
                    })
                    .map_err(|_| range)
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| match handle {
                Ok(handle) => match handle.join() {
                    Ok(part) => part,
                    Err(panic) => std::panic::resume_unwind(panic),
                },
                Err(range) => count_range(permuted, family, range),
            })
            .collect()
    });
    let mut total = Counted::zero(family.observed.len());
    for part in parts {
        total.add(&part?);
    }
    Ok(total)
}
