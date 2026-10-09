//! Statistics over a batch of charts (`03-design/research.md`).
//!
//! Nothing here reads the sky. The façade founds a study's charts and
//! evaluates each predicate on each once, into a [`Matrix`]; this crate
//! permutes the study's labels over that matrix. A test answers, for every
//! predicate, a permutation p-value `(b + 1)/(m + 1)` with its Monte Carlo
//! interval, the family's adjusted p-values and the effect sizes. It
//! never says "significant": a caller who passes an `alpha` is told which
//! predicates fall under it by each method, and nothing more.
//!
//! ```
//! use teistro_research::{Cell, Contrast, Design, GroupTest, Matrix, compare};
//!
//! // Ten charts, the first five cases; the predicate holds on four cases
//! // and one other.
//! let mut matrix = Matrix::new(10);
//! let cells: Vec<Cell> = [1, 1, 1, 1, 0, 1, 0, 0, 0, 0]
//!     .iter()
//!     .map(|&holds| if holds == 1 { Cell::Present } else { Cell::Absent })
//!     .collect();
//! matrix.push("holds", &cells)?;
//! let design = Design::new(vec![1, 1, 1, 1, 1, 0, 0, 0, 0, 0]);
//! let test = GroupTest::new(8, 9_999, Contrast::CaseVsRest { case: 1 });
//! let tested = compare(&matrix, &design, &test)?;
//! let row = &tested.rows[0];
//! // The hypergeometric puts 52 of the 252 ways of choosing five cases
//! // at least this far from the mean: 0, 1, 4 or 5 of them holding it.
//! let exact = row.exact.expect("two groups, every chart read");
//! assert!((exact - 52.0 / 252.0).abs() < 1e-12);
//! assert!(row.p.low <= exact && exact <= row.p.high);
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

#![doc(html_no_source)]

use std::collections::BTreeMap;
use std::num::NonZeroU16;

use serde::{Deserialize, Serialize};
use teistro_core::error::Error;

mod bits;
mod correct;
mod effect;
mod engine;
mod events;
mod recombined;
mod rng;
mod special;
#[cfg(test)]
mod tests;

pub use bits::{Cell, Matrix};
pub use effect::{Effect, Interval};
pub use events::{AfterBirth, EventTest, MAX_ATTEMPTS, MAX_SUBJECTS, PairMatrix, timed};
pub use recombined::{MAX_REPLICATES, ReplicateTest, replicated};
pub use rng::{ShuffleVersion, SplitMix64, stream_seed};

use engine::Layout;

/// The most permutations one test may draw. It bounds a request's cost
/// (charts × predicates × permutations / 64 popcounts), and its
/// resolution, `1/(m + 1)`, is finer than any correction a study of a
/// few thousand predicates needs.
pub const MAX_PERMUTATIONS: u32 = 10_000_000;

/// The most groups a design may have.
pub const MAX_GROUPS: u16 = 64;

/// What the labels are compared by.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(
    tag = "kind",
    rename_all = "SCREAMING_SNAKE_CASE",
    rename_all_fields = "camelCase"
)]
pub enum Contrast {
    /// One group (the cases) against every other chart. The statistic is
    /// the case count standardised under the hypergeometric that holds
    /// the margins, so predicates of different prevalence rank on one
    /// scale for max-T; ranked by it, a count and a risk difference give
    /// the same test.
    CaseVsRest {
        /// The cases' group.
        case: u16,
    },
    /// Whether the predicate's share differs between any of the groups:
    /// Pearson's chi-square on the groups-by-(present, absent) table,
    /// which is two-sided by construction.
    AnyDifference,
}

/// The direction a case-against-rest test looks in. A directional
/// hypothesis is declared before the data, never chosen after.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Alternative {
    /// The predicate is commoner among the cases.
    Greater,
    /// The predicate is rarer among the cases.
    Less,
    /// Either.
    #[default]
    TwoSided,
}

/// How many threads count the permutations. The answer is the same bits
/// for every choice (`research.md` §2.2); only the time differs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Parallelism {
    /// The calling thread alone.
    #[default]
    One,
    /// Up to this many threads, each a contiguous range of permutations.
    Threads(NonZeroU16),
}

/// Who is in which group, and inside which strata the labels may move.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Design {
    /// Each chart's group, `0..groups`, in the matrix's order.
    pub groups: Vec<u16>,
    /// Each chart's stratum, when the study's exchangeability holds only
    /// inside strata (a birth decade, a region, a sex). The labels are
    /// shuffled within each stratum, strata in ascending key order.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strata: Option<Vec<u32>>,
}

impl Design {
    /// A design of `groups`, one per chart, shuffled across the whole batch.
    #[must_use]
    pub fn new(groups: Vec<u16>) -> Design {
        Design {
            groups,
            strata: None,
        }
    }

    /// The same design, its labels shuffled only inside `strata`, one key
    /// per chart.
    #[must_use]
    pub fn with_strata(mut self, strata: Vec<u32>) -> Design {
        self.strata = Some(strata);
        self
    }
}

/// A permutation test of the design's groups.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GroupTest {
    /// The study's seed. Required: a default seed would hide that one
    /// was chosen.
    pub seed: u64,
    /// How many permutations to draw, 1 to [`MAX_PERMUTATIONS`]. Fixed in
    /// the request: there is no adaptive mode, so a study cannot draw
    /// until it likes the answer.
    pub permutations: u32,
    /// What the groups are compared by.
    pub contrast: Contrast,
    /// The direction of a case-against-rest test.
    #[serde(default)]
    pub alternative: Alternative,
    /// The confidence of every interval the answer gives, in (0, 1).
    #[serde(default = "default_level")]
    pub level: f64,
    /// When given, in (0, 1), the answer lists which predicates fall
    /// under it by each method.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alpha: Option<f64>,
    /// How many threads count the permutations.
    #[serde(default)]
    pub parallelism: Parallelism,
    /// The generator and shuffle.
    #[serde(default)]
    pub shuffle: ShuffleVersion,
}

pub(crate) const fn default_level() -> f64 {
    0.95
}

impl GroupTest {
    /// A two-sided test of `contrast` over `permutations` permutations
    /// drawn from `seed`, at 95% intervals, on one thread.
    #[must_use]
    pub fn new(seed: u64, permutations: u32, contrast: Contrast) -> GroupTest {
        GroupTest {
            seed,
            permutations,
            contrast,
            alternative: Alternative::default(),
            level: default_level(),
            alpha: None,
            parallelism: Parallelism::default(),
            shuffle: ShuffleVersion::default(),
        }
    }
}

/// A predicate's charts in one group.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GroupCount {
    /// Read, and the predicate holds.
    pub present: u32,
    /// Read, and the predicate does not hold.
    pub absent: u32,
    /// Not read: left out of the denominator, never counted absent.
    pub unreadable: u32,
    /// Different answers inside the chart's time uncertainty: left out of
    /// the denominator and counted here.
    pub unstable: u32,
}

/// A permutation p-value and its Monte Carlo uncertainty.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PValue {
    /// How many permutations were at least as extreme as the observed
    /// labelling.
    pub exceed: u64,
    /// `(exceed + 1)/(m + 1)` (Phipson and Smyth, 2010): the observed
    /// labelling is a member of its own reference set, so it is never
    /// zero.
    pub value: f64,
    /// The Clopper–Pearson interval on the p-value every permutation
    /// would give, from `exceed` of `m`.
    pub low: f64,
    /// The interval's upper end.
    pub high: f64,
}

/// The family's adjusted p-values: every predicate in one request is one
/// family (`research.md` §1.5).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Adjusted {
    /// Westfall–Young step-down max-T, from the same permutations: the
    /// familywise error rate, using the predicates' real dependence.
    pub max_t: f64,
    /// Holm's step-down, which holds under any dependence.
    pub holm: f64,
    /// Bonferroni, the bound Holm improves on.
    pub bonferroni: f64,
    /// Benjamini–Hochberg, the false discovery rate under positive
    /// dependence.
    pub bh: f64,
    /// Benjamini–Yekutieli, the false discovery rate under any dependence.
    pub by: f64,
}

/// A one-group design's share against what its null expects.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Expectation {
    /// The share of the study's own pairs (or charts) the predicate holds
    /// on, over those it was read on.
    pub observed: f64,
    /// The share the null expects.
    pub expected: f64,
    /// Observed over expected, the ratio Gauquelin's literature quotes;
    /// absent when nothing is expected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ratio: Option<f64>,
}

/// Which methods put a predicate at or under the caller's alpha.
#[allow(
    clippy::struct_excessive_bools,
    reason = "one answer per method, each independent of the others"
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct UnderAlpha {
    /// The raw p-value, uncorrected.
    pub raw: bool,
    /// Max-T.
    pub max_t: bool,
    /// Holm.
    pub holm: bool,
    /// Bonferroni.
    pub bonferroni: bool,
    /// Benjamini–Hochberg.
    pub bh: bool,
    /// Benjamini–Yekutieli.
    pub by: bool,
}

/// One predicate's answer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PredicateRow {
    /// The predicate, by the name its column was added under.
    pub predicate: String,
    /// Its charts in each group, groups in index order.
    pub counts: Vec<GroupCount>,
    /// The test statistic under the observed labels; none when it is
    /// unbounded, which is a recombined sample whose replicates all agree
    /// and which lies beyond them. The p-value still ranks it beyond every
    /// replicate.
    pub observed: Option<f64>,
    /// The permutation p-value.
    pub p: PValue,
    /// The exact p of the same statistic, from the hypergeometric: given
    /// for a case-against-rest test without strata on a predicate read on
    /// every chart, as the permutation p's cross-check.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exact: Option<f64>,
    /// The family's adjusted p-values.
    pub adjusted: Adjusted,
    /// The cases against the rest, for a case-against-rest test whose
    /// groups both have a chart the predicate was read on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect: Option<Effect>,
    /// The share observed against the share the null expects, for a
    /// one-group design: an event study, or a sample against its own
    /// recombined population.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected: Option<Expectation>,
    /// Which methods put it under the caller's alpha, when one was given.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub under_alpha: Option<UnderAlpha>,
}

/// What a test answers: per predicate, never a single verdict.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Tested {
    /// One row per predicate, in the matrix's order.
    pub rows: Vec<PredicateRow>,
    /// How many permutations were drawn.
    pub permutations: u32,
    /// The smallest p-value they can give, `1/(m + 1)`.
    pub resolution: f64,
    /// The generator and shuffle they were drawn by.
    pub shuffle: ShuffleVersion,
}

fn refuse(field: &str, message: String) -> Error {
    Error::invalid_arg(message).with_field(field)
}

/// Checks the design against the matrix, returning the number of groups.
fn check_design(matrix: &Matrix, design: &Design) -> Result<u16, Error> {
    let charts = matrix.charts();
    if charts < 2 {
        return Err(refuse(
            "groups",
            format!("a study needs at least two charts, and this one has {charts}"),
        ));
    }
    if design.groups.len() != charts {
        return Err(refuse(
            "groups",
            format!(
                "{} group labels for {charts} charts; give one label per chart",
                design.groups.len()
            ),
        ));
    }
    if matrix.predicates() == 0 {
        return Err(refuse("predicates", "the study names no predicate".into()));
    }
    let group_count = design.groups.iter().max().map_or(0, |max| max + 1);
    if group_count > MAX_GROUPS {
        return Err(refuse(
            "groups",
            format!(
                "group {} is above the most a design may have, {MAX_GROUPS}",
                group_count - 1
            ),
        ));
    }
    let mut sizes = vec![0_usize; usize::from(group_count)];
    for &group in &design.groups {
        if let Some(size) = sizes.get_mut(usize::from(group)) {
            *size += 1;
        }
    }
    if let Some(empty) = sizes.iter().position(|&size| size == 0) {
        return Err(refuse(
            "groups",
            format!("group {empty} has no chart; groups are numbered from 0 without gaps"),
        ));
    }
    if group_count < 2 {
        return Err(refuse(
            "groups",
            "every chart carries one label, so there is nothing to compare".into(),
        ));
    }
    if let Some(strata) = &design.strata {
        if strata.len() != charts {
            return Err(refuse(
                "strata",
                format!(
                    "{} strata for {charts} charts; give one stratum per chart",
                    strata.len()
                ),
            ));
        }
        if !movable(&design.groups, strata) {
            return Err(refuse(
                "strata",
                "no stratum holds two charts of different groups, so no label can move".into(),
            ));
        }
    }
    Ok(group_count)
}

/// Checks the test against the design's `group_count` groups and the
/// matrix's family.
fn check_test(matrix: &Matrix, test: &GroupTest, group_count: u16) -> Result<(), Error> {
    match test.contrast {
        Contrast::CaseVsRest { case } if case >= group_count => {
            return Err(refuse(
                "contrast.case",
                format!(
                    "group {case} is not in the design, whose groups are 0 to {}",
                    group_count - 1
                ),
            ));
        }
        Contrast::AnyDifference if test.alternative != Alternative::TwoSided => {
            return Err(refuse(
                "alternative",
                "a chi-square over every group has no direction; ask TWO_SIDED, or a CASE_VS_REST contrast".into(),
            ));
        }
        _ => {}
    }
    check_permutations(
        test.permutations,
        test.level,
        test.alpha,
        matrix.predicates(),
    )
}

/// Checks a test's permutations, interval level and alpha against a
/// family of `family` predicates.
pub(crate) fn check_permutations(
    permutations: u32,
    level: f64,
    alpha: Option<f64>,
    family: usize,
) -> Result<(), Error> {
    if permutations == 0 || permutations > MAX_PERMUTATIONS {
        return Err(refuse(
            "permutations",
            format!("{permutations} permutations is outside 1 to {MAX_PERMUTATIONS}"),
        ));
    }
    if !(level > 0.0 && level < 1.0) {
        return Err(refuse(
            "level",
            format!("an interval's confidence is inside (0, 1), not {level}"),
        ));
    }
    if let Some(alpha) = alpha {
        if !(alpha > 0.0 && alpha < 1.0) {
            return Err(refuse(
                "alpha",
                format!("alpha is inside (0, 1), not {alpha}"),
            ));
        }
        if !reaches_alpha(u64::from(permutations), family, alpha) {
            #[allow(
                clippy::cast_precision_loss,
                reason = "a family is a request's rules, far inside f64's exact integers"
            )]
            let bonferroni = alpha / family as f64;
            return Err(refuse(
                "permutations",
                format!(
                    "{permutations} permutations cannot reach a p-value under alpha / {family} = {bonferroni}, the strictest threshold the answer reports"
                ),
            )
            .with_hint(format!(
                "at least {} permutations",
                least_reaching(family, alpha)
            )));
        }
    }
    Ok(())
}

/// Whether some stratum holds two charts of different groups, so that a
/// shuffle inside strata can move a label.
fn movable(groups: &[u16], strata: &[u32]) -> bool {
    let mut first: BTreeMap<u32, u16> = BTreeMap::new();
    groups
        .iter()
        .zip(strata)
        .any(|(&group, &stratum)| *first.entry(stratum).or_insert(group) != group)
}

/// The exact p of the case-against-rest statistic from the hypergeometric
/// of the case count, when every chart was read and nothing is stratified.
fn exact(
    column: &bits::Column,
    case_size: u32,
    statistic: &engine::Statistic,
    observed: f64,
) -> f64 {
    let total = column.readable_total;
    let marked = column.present_total;
    let mean = f64::from(case_size) * f64::from(marked) / f64::from(total);
    let variance = f64::from(case_size)
        * (f64::from(marked) / f64::from(total))
        * (1.0 - f64::from(marked) / f64::from(total))
        * f64::from(total - case_size)
        / (f64::from(total) - 1.0);
    let sd = if variance > 0.0 { variance.sqrt() } else { 0.0 };
    special::hypergeometric(total, marked, case_size)
        .into_iter()
        .filter(|&(k, _)| {
            let z = if sd > 0.0 {
                (f64::from(k) - mean) / sd
            } else {
                0.0
            };
            let stat = match statistic.alternative {
                Alternative::Greater => z,
                Alternative::Less => -z,
                Alternative::TwoSided => z.abs(),
            };
            engine::reaches(stat, observed)
        })
        .map(|(_, p)| p)
        .sum::<f64>()
        .min(1.0)
}

/// Tests whether the design's groups differ on each predicate, by
/// permuting the labels (within strata when the design has them).
///
/// # Errors
///
/// `INVALID_ARG` naming the field: fewer than two charts, labels or
/// strata not one per chart, a group with no chart or one label for every
/// chart (`groups`); strata in which no label can move (`strata`); a case
/// group not in the design (`contrast.case`); a direction asked of a
/// chi-square (`alternative`); permutations outside 1 to
/// [`MAX_PERMUTATIONS`], or too few to reach `alpha` over the family
/// (`permutations`, with the hint); an interval level or alpha outside
/// (0, 1) (`level`, `alpha`); no predicate (`predicates`).
pub fn compare(matrix: &Matrix, design: &Design, test: &GroupTest) -> Result<Tested, Error> {
    let group_count = check_design(matrix, design)?;
    check_test(matrix, test, group_count)?;
    let layout = Layout::new(&design.groups, group_count, design.strata.as_deref());
    let statistic = engine::Statistic {
        contrast: test.contrast,
        alternative: test.alternative,
    };
    let observed = engine::observed(&layout, matrix, &statistic);
    let order = engine::step_down_order(&observed);
    let labelled = engine::Labelled {
        layout: &layout,
        matrix,
        statistic: &statistic,
        seed: test.seed,
    };
    let counted = engine::count(
        &labelled,
        &engine::Family::new(&observed, &order),
        test.permutations,
        test.parallelism,
    )?;
    let assessed = assess(&counted, &order, test.permutations, test.level, test.alpha);
    let context = RowContext {
        design,
        group_count,
        test,
        statistic: &statistic,
        charts: matrix.charts(),
        z: special::normal_quantile(1.0 - (1.0 - test.level) / 2.0),
    };
    let rows = matrix
        .columns
        .iter()
        .zip(&observed)
        .zip(assessed)
        .map(|((column, &observed), assessed)| context.row(column, observed, assessed))
        .collect();
    Ok(Tested {
        rows,
        permutations: test.permutations,
        resolution: p_value(0, u64::from(test.permutations)),
        shuffle: test.shuffle,
    })
}

/// Whether `permutations` can put the smallest p-value they give,
/// `1/(m + 1)`, at or under `alpha` once Bonferroni adjusts it over
/// `family`: the same arithmetic the answer's `underAlpha.bonferroni`
/// reads, so a count this accepts always can, and one it refuses never
/// can.
#[allow(
    clippy::cast_precision_loss,
    reason = "a family is a request's rules, far inside f64's exact integers"
)]
pub(crate) fn reaches_alpha(permutations: u64, family: usize, alpha: f64) -> bool {
    correct::bonferroni_one(p_value(0, permutations), family as f64) <= alpha
}

/// The fewest permutations [`reaches_alpha`] accepts, found from
/// `⌈M/alpha⌉ − 1` and stepped to the boundary the rounding puts it at,
/// since the estimate can sit one either side.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the estimate is a positive integer-valued float, clamped before the cast"
)]
pub(crate) fn least_reaching(family: usize, alpha: f64) -> u64 {
    let estimate = ((family as f64 / alpha).ceil() - 1.0).clamp(1.0, 1e15);
    let mut least = estimate as u64;
    while least > 1 && reaches_alpha(least - 1, family, alpha) {
        least -= 1;
    }
    while !reaches_alpha(least, family, alpha) {
        least += 1;
    }
    least
}

/// `(b + 1)/(m + 1)`.
#[allow(
    clippy::cast_precision_loss,
    reason = "both are at most MAX_PERMUTATIONS + 1, far inside f64's exact integers"
)]
fn p_value(b: u64, m: u64) -> f64 {
    (b as f64 + 1.0) / (m as f64 + 1.0)
}

/// One predicate's p-value, adjusted p-values and verdicts by alpha.
#[derive(Clone, Copy)]
pub(crate) struct Assessed {
    pub(crate) p: PValue,
    pub(crate) adjusted: Adjusted,
    pub(crate) under_alpha: Option<UnderAlpha>,
}

/// Every predicate's p-value with its Monte Carlo interval, and the
/// family's corrections, from what the permutations counted.
pub(crate) fn assess(
    counted: &engine::Counted,
    order: &[usize],
    permutations: u32,
    level: f64,
    alpha: Option<f64>,
) -> Vec<Assessed> {
    let m = u64::from(permutations);
    let raw: Vec<f64> = counted.exceed.iter().map(|&b| p_value(b, m)).collect();
    let mut running: f64 = 0.0;
    let max_t = correct::in_request_order(
        order
            .iter()
            .zip(&counted.step_down)
            .map(|(&j, &b)| {
                running = running.max(p_value(b, m));
                (j, running)
            })
            .collect(),
    );
    max_t
        .into_iter()
        .zip(correct::holm(&raw))
        .zip(correct::bonferroni(&raw))
        .zip(correct::benjamini_hochberg(&raw))
        .zip(correct::benjamini_yekutieli(&raw))
        .zip(raw.iter().zip(&counted.exceed))
        .map(
            |(((((max_t, holm), bonferroni), bh), by), (&value, &exceed))| {
                let (low, high) = special::clopper_pearson(exceed, m, level);
                let adjusted = Adjusted {
                    max_t,
                    holm,
                    bonferroni,
                    bh,
                    by,
                };
                Assessed {
                    p: PValue {
                        exceed,
                        value,
                        low,
                        high,
                    },
                    adjusted,
                    under_alpha: alpha.map(|alpha| UnderAlpha {
                        raw: value <= alpha,
                        max_t: max_t <= alpha,
                        holm: holm <= alpha,
                        bonferroni: bonferroni <= alpha,
                        bh: bh <= alpha,
                        by: by <= alpha,
                    }),
                }
            },
        )
        .collect()
}

/// What every row of one test reads besides its own column.
struct RowContext<'a> {
    design: &'a Design,
    group_count: u16,
    test: &'a GroupTest,
    statistic: &'a engine::Statistic,
    charts: usize,
    /// The normal quantile of the test's interval level.
    z: f64,
}

impl RowContext<'_> {
    fn row(&self, column: &bits::Column, observed: f64, assessed: Assessed) -> PredicateRow {
        let counts = group_counts(column, &self.design.groups, self.group_count);
        let (exact, effect) = match self.test.contrast {
            Contrast::CaseVsRest { case } => {
                let case_count = counts.get(usize::from(case)).copied().unwrap_or_default();
                let case_present = case_count.present;
                let case_read = case_count.present + case_count.absent;
                let every_read = column.readable_total as usize == self.charts;
                let exact = (self.design.strata.is_none() && every_read)
                    .then(|| exact(column, case_read, self.statistic, observed));
                let effect = Effect::of(
                    case_present,
                    case_read,
                    column.present_total - case_present,
                    column.readable_total - case_read,
                    self.z,
                );
                (exact, effect)
            }
            Contrast::AnyDifference => (None, None),
        };
        PredicateRow {
            predicate: column.name.clone(),
            counts,
            observed: Some(observed),
            p: assessed.p,
            exact,
            adjusted: assessed.adjusted,
            effect,
            expected: None,
            under_alpha: assessed.under_alpha,
        }
    }
}

/// How often one predicate holds in each group.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CountRow {
    /// The predicate, by its column's name.
    pub predicate: String,
    /// Its charts in each group, groups in index order.
    pub counts: Vec<GroupCount>,
}

/// How often every predicate holds in each group: a study's first table,
/// with no null and no shuffle.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Counts {
    /// One row per predicate, in the matrix's order.
    pub rows: Vec<CountRow>,
}

/// How often each predicate holds in each of the design's groups, with
/// the charts it could not be read on and those it is unstable on
/// counted apart.
///
/// # Errors
///
/// As [`compare`] refuses a design: fewer than two charts, labels or
/// strata not one per chart, a group with no chart or one label for every
/// chart (`groups`), strata in which no label can move (`strata`), no
/// predicate (`predicates`).
pub fn counts(matrix: &Matrix, design: &Design) -> Result<Counts, Error> {
    let group_count = check_design(matrix, design)?;
    Ok(Counts {
        rows: matrix
            .columns
            .iter()
            .map(|column| CountRow {
                predicate: column.name.clone(),
                counts: group_counts(column, &design.groups, group_count),
            })
            .collect(),
    })
}

impl Matrix {
    /// Every predicate's cells over the whole batch, as one group: what a
    /// one-group design counts.
    #[must_use]
    pub fn tallies(&self) -> Vec<GroupCount> {
        let one = vec![0; self.charts()];
        self.columns
            .iter()
            .map(|column| {
                group_counts(column, &one, 1)
                    .first()
                    .copied()
                    .unwrap_or_default()
            })
            .collect()
    }
}

/// A predicate's cells in each group, under the observed labels.
fn group_counts(column: &bits::Column, groups: &[u16], group_count: u16) -> Vec<GroupCount> {
    let mut counts = vec![GroupCount::default(); usize::from(group_count)];
    for (index, &group) in groups.iter().enumerate() {
        let Some(count) = counts.get_mut(usize::from(group)) else {
            continue;
        };
        if column.present.contains(index) {
            count.present += 1;
        } else if column.readable.contains(index) {
            count.absent += 1;
        } else if column.unstable.contains(index) {
            count.unstable += 1;
        } else {
            count.unreadable += 1;
        }
    }
    counts
}
