//! The permutation engine (`research.md` §§1.2, 1.5, 2.2, 2.3).
//!
//! Every permutation draws its own stream ([`SplitMix64::for_permutation`]),
//! shuffles the labels inside each stratum, and reads every predicate's
//! statistic off the bitsets. A float statistic is computed per
//! permutation in one fixed order; only the integer exceedance counts
//! cross threads, and an integer sum has no order, so the answer is the
//! same bits at every thread count.

use std::collections::BTreeMap;
use std::ops::Range;

use teistro_core::error::Error;

use crate::bits::{Bits, Column, Matrix};
use crate::rng::{ShuffleVersion, SplitMix64};
use crate::{Alternative, Contrast, Parallelism};

/// How a test walks its reference set: `m` draws from it, or every one
/// of its `N` arrangements once.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Walk {
    /// `m` permutations drawn at random.
    Sampled(u32),
    /// Every arrangement, the observed one among them.
    Enumerated(u32),
}

impl Walk {
    /// Every arrangement when `shuffle` enumerates and the reference set
    /// holds no more than the `permutations` asked, which then cost no
    /// more than drawing them; the draws otherwise.
    pub(crate) fn of(
        shuffle: ShuffleVersion,
        permutations: u32,
        arrangements: Option<u64>,
    ) -> Walk {
        match arrangements.and_then(|n| u32::try_from(n).ok()) {
            Some(n) if shuffle.enumerates() && n <= permutations => Walk::Enumerated(n),
            _ => Walk::Sampled(permutations),
        }
    }

    /// How many statistics the walk reads.
    pub(crate) const fn len(self) -> u32 {
        match self {
            Walk::Sampled(m) | Walk::Enumerated(m) => m,
        }
    }

    /// Whether the walk is the whole reference set.
    pub(crate) const fn enumerated(self) -> bool {
        matches!(self, Walk::Enumerated(_))
    }
}

/// `n!/(c₁!⋯c_k!)` for counts `c` summing to `n`, or `None` past `u64`.
fn multinomial(counts: impl IntoIterator<Item = u64>) -> Option<u64> {
    let (mut ways, mut placed) = (1_u128, 0_u128);
    for count in counts {
        for i in 1..=u128::from(count) {
            placed += 1;
            // ways · C(placed, i) built a factor at a time; each step is
            // a whole number, so the division is exact.
            ways = ways.checked_mul(placed)? / i;
            if ways > u128::from(u64::MAX) {
                return None;
            }
        }
    }
    u64::try_from(ways).ok()
}

/// The product of `factors`, or `None` past `u64`.
pub(crate) fn product(factors: impl IntoIterator<Item = u64>) -> Option<u64> {
    factors.into_iter().try_fold(1_u64, u64::checked_mul)
}

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

    /// Each stratum's labels, ascending, with how many positions carry
    /// each, into `counts`.
    fn stratum_counts(&self, positions: &[usize], counts: &mut Vec<(u16, u64)>) {
        counts.clear();
        for &position in positions {
            if let Some(&label) = self.groups.get(position) {
                match counts.binary_search_by_key(&label, |&(l, _)| l) {
                    Ok(at) => {
                        if let Some(entry) = counts.get_mut(at) {
                            entry.1 += 1;
                        }
                    }
                    Err(at) => counts.insert(at, (label, 1)),
                }
            }
        }
    }

    /// How many distinct labellings the strata allow: each stratum's
    /// multinomial, multiplied, or `None` past `u64`.
    pub(crate) fn arrangements(&self) -> Option<u64> {
        let mut counts = Vec::new();
        product(self.strata.iter().map(|positions| {
            self.stratum_counts(positions, &mut counts);
            multinomial(counts.iter().map(|&(_, c)| c)).unwrap_or(u64::MAX)
        }))
        .filter(|&n| n < u64::MAX)
    }

    /// Writes the labelling of rank `rank` into `labels`: the rank's
    /// mixed-radix digits over the strata, each unranked as a multiset
    /// permutation in lexicographic order. Every rank below
    /// [`arrangements`](Self::arrangements) is one distinct labelling.
    pub(crate) fn arrange(&self, mut rank: u64, labels: &mut [u16], counts: &mut Vec<(u16, u64)>) {
        for positions in &self.strata {
            self.stratum_counts(positions, counts);
            let radix = multinomial(counts.iter().map(|&(_, c)| c)).unwrap_or(1);
            let mut digit = u128::from(rank % radix);
            rank /= radix;
            let mut ways = u128::from(radix);
            let mut left = positions.len() as u128;
            for &position in positions {
                for (label, count) in counts.iter_mut() {
                    if *count == 0 {
                        continue;
                    }
                    // The labellings of the rest with `label` here.
                    let here = ways * u128::from(*count) / left;
                    if digit < here {
                        if let Some(slot) = labels.get_mut(position) {
                            *slot = *label;
                        }
                        *count -= 1;
                        ways = here;
                        break;
                    }
                    digit -= here;
                }
                left -= 1;
            }
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

/// The mean and variance of how many of `drawn` charts carry a predicate
/// that `marked` of `total` carry: the hypergeometric that holds the
/// margins. None under two charts, where nothing can move. The plain
/// statistic, the stratified one and the exact p all read it, so the
/// three round alike.
pub(crate) fn moments(drawn: u32, marked: u32, total: u32) -> Option<(f64, f64)> {
    let total = f64::from(total);
    if total < 2.0 {
        return None;
    }
    let drawn = f64::from(drawn);
    let share = f64::from(marked) / total;
    let mean = drawn * share;
    let variance = drawn * share * (1.0 - share) * (total - drawn) / (total - 1.0);
    Some((mean, variance))
}

/// The standardised case count: the count's departure from its mean
/// under the hypergeometric that holds the margins, in its standard
/// deviations. Zero where the predicate cannot vary (it holds on every
/// read chart or none, or one group has every read chart).
fn standardised(case: Share, column: &Column) -> f64 {
    match moments(case.read, column.present_total, column.readable_total) {
        Some((mean, variance)) if variance > 0.0 => {
            (f64::from(case.present) - mean) / variance.sqrt()
        }
        _ => 0.0,
    }
}

/// One stratum's margins on one predicate: its charts the predicate was
/// read on, and those it holds on. No permutation moves them.
#[derive(Clone, Copy)]
struct Margin {
    read: u32,
    present: u32,
}

/// The strata a case-against-rest statistic sums over, each a contiguous
/// range of the batch as [`Matrix::reordered`] lays it out, with every
/// predicate's margins in each.
pub(crate) struct Strata {
    bounds: Vec<Range<usize>>,
    /// Per predicate, per stratum.
    margins: Vec<Vec<Margin>>,
}

impl Strata {
    /// The strata of `layout` over `matrix`, whose strata are contiguous;
    /// none for a single stratum, whose statistic is the plain one.
    pub(crate) fn of(layout: &Layout<'_>, matrix: &Matrix) -> Option<Strata> {
        if layout.strata.len() < 2 {
            return None;
        }
        let bounds: Vec<Range<usize>> = layout
            .strata
            .iter()
            .map(|positions| {
                let start = positions.first().copied().unwrap_or(0);
                let end = positions.last().map_or(start, |last| last + 1);
                debug_assert_eq!(end - start, positions.len(), "a stratum is contiguous");
                start..end
            })
            .collect();
        let margins = matrix
            .columns
            .iter()
            .map(|column| {
                bounds
                    .iter()
                    .map(|range| Margin {
                        read: column.readable.common_in(&column.readable, range),
                        present: column.present.common_in(&column.present, range),
                    })
                    .collect()
            })
            .collect();
        Some(Strata { bounds, margins })
    }
}

/// The Cochran–Mantel–Haenszel statistic (Mantel and Haenszel, *J. Natl.
/// Cancer Inst.* 22, 1959): each stratum's case count less its mean under
/// its own hypergeometric, summed, over the root of the summed variances.
/// With one stratum it is [`standardised`].
fn stratified(case: &Bits, column: &Column, margins: &[Margin], bounds: &[Range<usize>]) -> f64 {
    let (mut departure, mut variance) = (0.0, 0.0);
    for (range, margin) in bounds.iter().zip(margins) {
        let drawn = case.common_in(&column.readable, range);
        if let Some((mean, spread)) = moments(drawn, margin.present, margin.read) {
            departure += f64::from(case.common_in(&column.present, range)) - mean;
            variance += spread;
        }
    }
    if variance > 0.0 {
        departure / variance.sqrt()
    } else {
        0.0
    }
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
    /// The strata a case-against-rest statistic sums over; none when the
    /// design has one stratum, or for a chi-square.
    pub(crate) strata: Option<Strata>,
}

impl Statistic {
    /// Every predicate's statistic under the labels whose group masks are
    /// `masks`.
    fn read(&self, masks: &[Bits], matrix: &Matrix, out: &mut [f64], shares: &mut Vec<Share>) {
        for (j, (column, slot)) in matrix.columns.iter().zip(out.iter_mut()).enumerate() {
            *slot = match self.contrast {
                Contrast::CaseVsRest { case } => {
                    let z = masks
                        .get(usize::from(case))
                        .map_or(0.0, |mask| match &self.strata {
                            Some(strata) => strata.margins.get(j).map_or(0.0, |margins| {
                                stratified(mask, column, margins, &strata.bounds)
                            }),
                            None => standardised(Share::of(mask, column), column),
                        });
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

    /// How many arrangements the reference set holds, each as likely
    /// under the null; `None` past `u64`, or for a set that is not a
    /// finite set of arrangements (replicates drawn from a population).
    fn arrangements(&self) -> Option<u64> {
        None
    }

    /// Every predicate's statistic under the arrangement of rank `rank`,
    /// below [`arrangements`](Self::arrangements), into `out`. Called
    /// only when that is `Some`.
    fn arrange(&self, rank: u64, scratch: &mut Self::Scratch, out: &mut [f64]) {
        let _ = (rank, scratch);
        out.fill(f64::NAN);
    }
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
    counts: Vec<(u16, u64)>,
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
            counts: Vec::new(),
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

    fn arrangements(&self) -> Option<u64> {
        self.layout.arrangements()
    }

    fn arrange(&self, rank: u64, scratch: &mut LabelScratch, out: &mut [f64]) {
        self.layout
            .arrange(rank, &mut scratch.labels, &mut scratch.counts);
        fill_masks(&scratch.labels, &mut scratch.masks);
        self.statistic
            .read(&scratch.masks, self.matrix, out, &mut scratch.shares);
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
    enumerated: bool,
) -> Result<Counted, Error> {
    let predicates = family.observed.len();
    let mut counted = Counted::zero(predicates);
    let mut scratch = permuted.scratch();
    let mut stats = vec![0.0; predicates];
    for k in range {
        if enumerated {
            permuted.arrange(k, &mut scratch, &mut stats);
        } else {
            permuted.read(k, &mut scratch, &mut stats)?;
        }
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

/// Counts the walk's statistics, permutations or arrangements
/// `0..walk.len()`, split into contiguous ranges
/// over the threads `parallelism` names. A thread the platform cannot
/// start (wasm has none) runs its range on the calling thread instead,
/// which by construction counts the same; and when ranges fail, the
/// lowest one's error is the answer, whatever the thread count.
pub(crate) fn count<P: Permuted>(
    permuted: &P,
    family: &Family<'_>,
    walk: Walk,
    parallelism: Parallelism,
) -> Result<Counted, Error> {
    let m = u64::from(walk.len());
    let enumerated = walk.enumerated();
    let threads = match parallelism {
        Parallelism::One => 1,
        Parallelism::Threads(n) => u64::from(n.get()).min(m.max(1)),
    };
    if threads <= 1 {
        return count_range(permuted, family, 0..m, enumerated);
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
                        move || count_range(permuted, family, range, enumerated)
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
                Err(range) => count_range(permuted, family, range, enumerated),
            })
            .collect()
    });
    let mut total = Counted::zero(family.observed.len());
    for part in parts {
        total.add(&part?);
    }
    Ok(total)
}
