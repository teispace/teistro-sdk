//! Statistics over a batch of charts (`03-design/research.md`).
//!
//! Nothing here reads the sky. The façade founds a study's charts and
//! evaluates each predicate on each once, into a [`Matrix`]; this crate
//! permutes the study's labels over that matrix. A test answers, for every
//! predicate, a permutation p-value `(b + 1)/(m + 1)` with its Monte Carlo
//! interval (or, when every labelling is no more than the permutations
//! asked, the exact `b/N` over all of them), the family's adjusted
//! p-values and the effect sizes. It
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
//! // 252 labellings are fewer than the 9 999 permutations asked, so each
//! // is read once and the permutation p is that count itself.
//! assert!(tested.enumerated);
//! assert_eq!(tested.permutations, 252);
//! assert_eq!(row.p.value, 52.0 / 252.0);
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
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
mod matched;
mod recombined;
mod rng;
mod special;
#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests;

pub use bits::{Cell, Matrix};
pub use effect::{Effect, Interval, Pooled};
pub use events::{
    AfterBirth, DEFAULT_MAX_PAIRS, EventTest, MAX_ATTEMPTS, MAX_SUBJECTS, PairMatrix, timed,
};
pub use matched::{MAX_INSTANTS, MatchedMatrix, matched};
pub use recombined::{MAX_REPLICATES, ReplicateTest, replicated};
pub use rng::{ShuffleVersion, SplitMix64, stream_seed};

use effect::Table;
use engine::{Layout, Permuted as _, Walk};

/// The most permutations one test may draw. It bounds a request's cost
/// (charts × predicates × permutations / 64 popcounts), and its
/// resolution, `1/(m + 1)`, is finer than any correction a study of a
/// few thousand predicates needs.
///
/// ```
/// # use teistro_research::{Cell, Contrast, Design, GroupTest, Matrix, compare};
/// #
/// # // Ten charts, the first five cases; the predicate holds on four cases
/// # // and one other. 252 ways of choosing five cases, each walked once.
/// # let cell = |holds: &u8| if *holds == 1 { Cell::Present } else { Cell::Absent };
/// # let mut matrix = Matrix::new(10);
/// # matrix.push("holds", &[1, 1, 1, 1, 0, 1, 0, 0, 0, 0].iter().map(cell).collect::<Vec<_>>())?;
/// # let design = Design::new(vec![1, 1, 1, 1, 1, 0, 0, 0, 0, 0]);
/// # let test = GroupTest::new(8, 9_999, Contrast::CaseVsRest { case: 1 });
/// # let tested = compare(&matrix, &design, &test)?;
/// # let row = &tested.rows[0];
///
/// use teistro_research::MAX_PERMUTATIONS;
///
/// let more = GroupTest { permutations: MAX_PERMUTATIONS + 1, ..test };
/// assert_eq!(compare(&matrix, &design, &more).unwrap_err().field(), Some("permutations"));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
pub const MAX_PERMUTATIONS: u32 = 10_000_000;

/// The most groups a design may have.
///
/// ```
/// # use teistro_research::{Cell, Contrast, Design, GroupTest, Matrix, compare};
/// #
/// # // Ten charts, the first five cases; the predicate holds on four cases
/// # // and one other. 252 ways of choosing five cases, each walked once.
/// # let cell = |holds: &u8| if *holds == 1 { Cell::Present } else { Cell::Absent };
/// # let mut matrix = Matrix::new(10);
/// # matrix.push("holds", &[1, 1, 1, 1, 0, 1, 0, 0, 0, 0].iter().map(cell).collect::<Vec<_>>())?;
/// # let design = Design::new(vec![1, 1, 1, 1, 1, 0, 0, 0, 0, 0]);
/// # let test = GroupTest::new(8, 9_999, Contrast::CaseVsRest { case: 1 });
/// # let tested = compare(&matrix, &design, &test)?;
/// # let row = &tested.rows[0];
///
/// use teistro_research::MAX_GROUPS;
///
/// // A label of 64 makes 65 groups.
/// let mut groups = design.groups.clone();
/// groups[9] = MAX_GROUPS;
/// assert_eq!(compare(&matrix, &Design::new(groups), &test).unwrap_err().field(), Some("groups"));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
pub const MAX_GROUPS: u16 = 64;

/// What the labels are compared by.
///
/// ```
/// # use teistro_research::{Cell, Contrast, Design, GroupTest, Matrix, compare};
/// #
/// # // Ten charts, the first five cases; the predicate holds on four cases
/// # // and one other. 252 ways of choosing five cases, each walked once.
/// # let cell = |holds: &u8| if *holds == 1 { Cell::Present } else { Cell::Absent };
/// # let mut matrix = Matrix::new(10);
/// # matrix.push("holds", &[1, 1, 1, 1, 0, 1, 0, 0, 0, 0].iter().map(cell).collect::<Vec<_>>())?;
/// # let design = Design::new(vec![1, 1, 1, 1, 1, 0, 0, 0, 0, 0]);
/// # let test = GroupTest::new(8, 9_999, Contrast::CaseVsRest { case: 1 });
/// # let tested = compare(&matrix, &design, &test)?;
/// # let row = &tested.rows[0];
///
/// // With two groups the chi-square is the case count's square, so the
/// // two contrasts rank every labelling alike.
/// let any = GroupTest::new(8, 9_999, Contrast::AnyDifference);
/// assert_eq!(compare(&matrix, &design, &any)?.rows[0].p, row.p);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
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
///
/// ```
/// # use teistro_research::{Cell, Contrast, Design, GroupTest, Matrix, compare};
/// #
/// # // Ten charts, the first five cases; the predicate holds on four cases
/// # // and one other. 252 ways of choosing five cases, each walked once.
/// # let cell = |holds: &u8| if *holds == 1 { Cell::Present } else { Cell::Absent };
/// # let mut matrix = Matrix::new(10);
/// # matrix.push("holds", &[1, 1, 1, 1, 0, 1, 0, 0, 0, 0].iter().map(cell).collect::<Vec<_>>())?;
/// # let design = Design::new(vec![1, 1, 1, 1, 1, 0, 0, 0, 0, 0]);
/// # let test = GroupTest::new(8, 9_999, Contrast::CaseVsRest { case: 1 });
/// # let tested = compare(&matrix, &design, &test)?;
/// # let row = &tested.rows[0];
///
/// use teistro_research::Alternative;
///
/// // Four or five of the cases: 25 + 1 of the 252 choices.
/// let greater = GroupTest { alternative: Alternative::Greater, ..test };
/// assert_eq!(compare(&matrix, &design, &greater)?.rows[0].p.value, 26.0 / 252.0);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
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
///
/// ```
/// # use teistro_research::{Cell, Contrast, Design, GroupTest, Matrix, compare};
/// #
/// # // Ten charts, the first five cases; the predicate holds on four cases
/// # // and one other. 252 ways of choosing five cases, each walked once.
/// # let cell = |holds: &u8| if *holds == 1 { Cell::Present } else { Cell::Absent };
/// # let mut matrix = Matrix::new(10);
/// # matrix.push("holds", &[1, 1, 1, 1, 0, 1, 0, 0, 0, 0].iter().map(cell).collect::<Vec<_>>())?;
/// # let design = Design::new(vec![1, 1, 1, 1, 1, 0, 0, 0, 0, 0]);
/// # let test = GroupTest::new(8, 9_999, Contrast::CaseVsRest { case: 1 });
/// # let tested = compare(&matrix, &design, &test)?;
/// # let row = &tested.rows[0];
///
/// use std::num::NonZeroU16;
/// use teistro_research::Parallelism;
///
/// // The same bits on any number of threads.
/// let threads = NonZeroU16::new(2).ok_or_else(|| teistro_core::error::Error::internal("two"))?;
/// let sampled = GroupTest::new(8, 999, Contrast::CaseVsRest { case: 1 });
/// let one = compare(&matrix, &design, &sampled)?;
/// let two = compare(&matrix, &design, &GroupTest { parallelism: Parallelism::Threads(threads), ..sampled })?;
/// assert_eq!(one, two);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
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
///
/// ```
/// use teistro_research::Design;
///
/// let design: Design = serde_json::from_str(r#"{"groups": [1, 0, 1, 0], "strata": [0, 0, 1, 1]}"#)?;
/// assert_eq!(design, Design::new(vec![1, 0, 1, 0]).with_strata(vec![0, 0, 1, 1]));
/// # Ok::<(), serde_json::Error>(())
/// ```
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
    ///
    /// ```
    /// use teistro_research::Design;
    ///
    /// let design = Design::new(vec![1, 1, 0, 0]);
    /// assert!(design.strata.is_none());
    /// ```
    #[must_use]
    pub fn new(groups: Vec<u16>) -> Design {
        Design {
            groups,
            strata: None,
        }
    }

    /// The same design, its labels shuffled only inside `strata`, one key
    /// per chart.
    ///
    /// ```
    /// # use teistro_research::{Cell, Contrast, Design, GroupTest, Matrix, compare};
    /// #
    /// # // Ten charts, the first five cases; the predicate holds on four cases
    /// # // and one other. 252 ways of choosing five cases, each walked once.
    /// # let cell = |holds: &u8| if *holds == 1 { Cell::Present } else { Cell::Absent };
    /// # let mut matrix = Matrix::new(10);
    /// # matrix.push("holds", &[1, 1, 1, 1, 0, 1, 0, 0, 0, 0].iter().map(cell).collect::<Vec<_>>())?;
    /// # let design = Design::new(vec![1, 1, 1, 1, 1, 0, 0, 0, 0, 0]);
    /// # let test = GroupTest::new(8, 9_999, Contrast::CaseVsRest { case: 1 });
    /// # let tested = compare(&matrix, &design, &test)?;
    /// # let row = &tested.rows[0];
    ///
    /// // One stratum holding every chart is no stratum at all.
    /// let one = compare(&matrix, &design.clone().with_strata(vec![0; 10]), &test)?;
    /// assert_eq!(one.rows[0].p, row.p);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn with_strata(mut self, strata: Vec<u32>) -> Design {
        self.strata = Some(strata);
        self
    }
}

/// A permutation test of the design's groups.
///
/// ```
/// use teistro_research::{Contrast, GroupTest};
///
/// let asked: GroupTest = serde_json::from_str(
///     r#"{"seed": 8, "permutations": 9999, "contrast": {"kind": "CASE_VS_REST", "case": 1}}"#,
/// )?;
/// assert_eq!(asked, GroupTest::new(8, 9_999, Contrast::CaseVsRest { case: 1 }));
/// # Ok::<(), serde_json::Error>(())
/// ```
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
    ///
    /// ```
    /// use teistro_research::{Alternative, Contrast, GroupTest};
    ///
    /// let test = GroupTest::new(8, 9_999, Contrast::AnyDifference);
    /// assert_eq!((test.alternative, test.level, test.alpha), (Alternative::TwoSided, 0.95, None));
    /// ```
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
///
/// ```
/// # use teistro_research::{Cell, Contrast, Design, GroupTest, Matrix, compare};
/// #
/// # // Ten charts, the first five cases; the predicate holds on four cases
/// # // and one other. 252 ways of choosing five cases, each walked once.
/// # let cell = |holds: &u8| if *holds == 1 { Cell::Present } else { Cell::Absent };
/// # let mut matrix = Matrix::new(10);
/// # matrix.push("holds", &[1, 1, 1, 1, 0, 1, 0, 0, 0, 0].iter().map(cell).collect::<Vec<_>>())?;
/// # let design = Design::new(vec![1, 1, 1, 1, 1, 0, 0, 0, 0, 0]);
/// # let test = GroupTest::new(8, 9_999, Contrast::CaseVsRest { case: 1 });
/// # let tested = compare(&matrix, &design, &test)?;
/// # let row = &tested.rows[0];
///
/// use teistro_research::GroupCount;
///
/// // Groups in index order: the rest, then the cases.
/// assert_eq!(row.counts[1], GroupCount { present: 4, absent: 1, unreadable: 0, unstable: 0 });
/// assert_eq!(row.counts[0].present, 1);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
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
///
/// ```
/// # use teistro_research::{Cell, Contrast, Design, GroupTest, Matrix, compare};
/// #
/// # // Ten charts, the first five cases; the predicate holds on four cases
/// # // and one other. 252 ways of choosing five cases, each walked once.
/// # let cell = |holds: &u8| if *holds == 1 { Cell::Present } else { Cell::Absent };
/// # let mut matrix = Matrix::new(10);
/// # matrix.push("holds", &[1, 1, 1, 1, 0, 1, 0, 0, 0, 0].iter().map(cell).collect::<Vec<_>>())?;
/// # let design = Design::new(vec![1, 1, 1, 1, 1, 0, 0, 0, 0, 0]);
/// # let test = GroupTest::new(8, 9_999, Contrast::CaseVsRest { case: 1 });
/// # let tested = compare(&matrix, &design, &test)?;
/// # let row = &tested.rows[0];
///
/// // Walked whole, so exact: 52 of the 252 at least as far from the mean.
/// assert_eq!(row.p.exceed, 52);
/// assert!(row.p.low <= row.p.value && row.p.value <= row.p.high);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
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
///
/// ```
/// # use teistro_research::{Cell, Contrast, Design, GroupTest, Matrix, compare};
/// #
/// # // Ten charts, the first five cases; the predicate holds on four cases
/// # // and one other. 252 ways of choosing five cases, each walked once.
/// # let cell = |holds: &u8| if *holds == 1 { Cell::Present } else { Cell::Absent };
/// # let mut matrix = Matrix::new(10);
/// # matrix.push("holds", &[1, 1, 1, 1, 0, 1, 0, 0, 0, 0].iter().map(cell).collect::<Vec<_>>())?;
/// # let design = Design::new(vec![1, 1, 1, 1, 1, 0, 0, 0, 0, 0]);
/// # let test = GroupTest::new(8, 9_999, Contrast::CaseVsRest { case: 1 });
/// # let tested = compare(&matrix, &design, &test)?;
/// # let row = &tested.rows[0];
///
/// // A family of one: every correction leaves the p where it is.
/// assert_eq!((row.adjusted.holm, row.adjusted.bonferroni, row.adjusted.bh), (row.p.value, row.p.value, row.p.value));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
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
///
/// ```
/// # use teistro_research::{Alternative, Cell, EventTest, PairMatrix, timed};
/// #
/// # // Four subjects; the predicate holds for each at its own event and never
/// # // at another's. Four events pair four subjects 24 ways.
/// # let mut matrix = PairMatrix::new(4)?;
/// # matrix.push("at own event", |subject, event| if subject == event { Cell::Present } else { Cell::Absent })?;
/// # let test = EventTest { alternative: Alternative::Greater, ..EventTest::new(7, 9_999) };
///
/// // Every subject at its own event, against a quarter of all pairings.
/// let expected = timed(&matrix, None, &test)?.rows[0].expected.ok_or_else(|| teistro_core::error::Error::internal("expected"))?;
/// assert_eq!((expected.observed, expected.expected, expected.ratio), (1.0, 0.25, Some(4.0)));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
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
///
/// ```
/// # use teistro_research::{Cell, Contrast, Design, GroupTest, Matrix, compare};
/// #
/// # // Ten charts, the first five cases; the predicate holds on four cases
/// # // and one other. 252 ways of choosing five cases, each walked once.
/// # let cell = |holds: &u8| if *holds == 1 { Cell::Present } else { Cell::Absent };
/// # let mut matrix = Matrix::new(10);
/// # matrix.push("holds", &[1, 1, 1, 1, 0, 1, 0, 0, 0, 0].iter().map(cell).collect::<Vec<_>>())?;
/// # let design = Design::new(vec![1, 1, 1, 1, 1, 0, 0, 0, 0, 0]);
/// # let test = GroupTest::new(8, 9_999, Contrast::CaseVsRest { case: 1 });
/// # let tested = compare(&matrix, &design, &test)?;
/// # let row = &tested.rows[0];
///
/// // Under an alpha of 0.25 the raw p of 52/252 falls, and so does every
/// // correction of a family of one.
/// let under = compare(&matrix, &design, &GroupTest { alpha: Some(0.25), ..test })?.rows[0].under_alpha;
/// assert!(under.is_some_and(|under| under.raw && under.holm && under.max_t));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
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
///
/// ```
/// # use teistro_research::{Cell, Contrast, Design, GroupTest, Matrix, compare};
/// #
/// # // Ten charts, the first five cases; the predicate holds on four cases
/// # // and one other. 252 ways of choosing five cases, each walked once.
/// # let cell = |holds: &u8| if *holds == 1 { Cell::Present } else { Cell::Absent };
/// # let mut matrix = Matrix::new(10);
/// # matrix.push("holds", &[1, 1, 1, 1, 0, 1, 0, 0, 0, 0].iter().map(cell).collect::<Vec<_>>())?;
/// # let design = Design::new(vec![1, 1, 1, 1, 1, 0, 0, 0, 0, 0]);
/// # let test = GroupTest::new(8, 9_999, Contrast::CaseVsRest { case: 1 });
/// # let tested = compare(&matrix, &design, &test)?;
/// # let row = &tested.rows[0];
///
/// assert_eq!(row.predicate, "holds");
/// // A case-against-rest row read on every chart carries its exact p and effect.
/// assert!(row.exact.is_some() && row.effect.is_some() && row.expected.is_none());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
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
    /// The exact p of the same statistic, given for a case-against-rest
    /// test on a predicate read on every chart, as the permutation p's
    /// cross-check: the hypergeometric of the case count, or under strata
    /// the convolution of each stratum's (the exact conditional test,
    /// Zelen, *Biometrika* 58, 1971).
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
///
/// ```
/// # use teistro_research::{Cell, Contrast, Design, GroupTest, Matrix, compare};
/// #
/// # // Ten charts, the first five cases; the predicate holds on four cases
/// # // and one other. 252 ways of choosing five cases, each walked once.
/// # let cell = |holds: &u8| if *holds == 1 { Cell::Present } else { Cell::Absent };
/// # let mut matrix = Matrix::new(10);
/// # matrix.push("holds", &[1, 1, 1, 1, 0, 1, 0, 0, 0, 0].iter().map(cell).collect::<Vec<_>>())?;
/// # let design = Design::new(vec![1, 1, 1, 1, 1, 0, 0, 0, 0, 0]);
/// # let test = GroupTest::new(8, 9_999, Contrast::CaseVsRest { case: 1 });
/// # let tested = compare(&matrix, &design, &test)?;
/// # let row = &tested.rows[0];
///
/// assert!(tested.enumerated);
/// assert_eq!((tested.permutations, tested.resolution), (252, 1.0 / 252.0));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Tested {
    /// One row per predicate, in the matrix's order.
    pub rows: Vec<PredicateRow>,
    /// How many permutations were drawn, or, when `enumerated`, how
    /// many arrangements the reference set holds.
    pub permutations: u32,
    /// The smallest p-value they can give: `1/(m + 1)` drawn, `1/N`
    /// enumerated.
    pub resolution: f64,
    /// Whether the reference set was walked whole, every arrangement
    /// once, because it held no more than the permutations asked. Every
    /// p-value is then exact, `b/N` with no Monte Carlo interval, and
    /// more permutations could not refine it.
    #[serde(default)]
    pub enumerated: bool,
    /// The generator and shuffle they were drawn by.
    pub shuffle: ShuffleVersion,
}

impl Tested {
    /// The answer of `rows` walked by `walk` under `shuffle`.
    pub(crate) fn walked(rows: Vec<PredicateRow>, walk: Walk, shuffle: ShuffleVersion) -> Tested {
        Tested {
            rows,
            permutations: walk.len(),
            resolution: walk.resolution(),
            enumerated: walk.enumerated(),
            shuffle,
        }
    }
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

/// The exact p of the case-against-rest statistic when every chart was
/// read: the distribution of the case count summed over the strata, each
/// stratum's hypergeometric convolved in, and the statistic of each total
/// against the observed one. With one stratum it is that stratum's
/// hypergeometric.
fn exact(tables: &[Table], statistic: &engine::Statistic, observed: f64) -> f64 {
    let (mut mean, mut variance) = (0.0, 0.0);
    let mut distribution = vec![1.0];
    for table in tables {
        let (read, present) = table.margins();
        if let Some((m, v)) = engine::moments(table.case_read, present, read) {
            mean += m;
            variance += v;
        }
        distribution = convolve(
            &distribution,
            &special::hypergeometric(read, present, table.case_read),
        );
    }
    let sd = if variance > 0.0 { variance.sqrt() } else { 0.0 };
    (0_u32..)
        .zip(&distribution)
        .filter(|&(total, _)| {
            let z = if sd > 0.0 {
                (f64::from(total) - mean) / sd
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
        .map(|(_, &p)| p)
        .sum::<f64>()
        .min(1.0)
}

/// The distribution of a sum: `left` over the totals so far, `right` one
/// stratum's counts with their probabilities.
fn convolve(left: &[f64], right: &[(u32, f64)]) -> Vec<f64> {
    let width = right.last().map_or(0, |&(k, _)| k as usize);
    let mut out = vec![0.0; left.len() + width];
    for (at, &p) in left.iter().enumerate() {
        for &(k, q) in right {
            if let Some(slot) = out.get_mut(at + k as usize) {
                *slot += p * q;
            }
        }
    }
    out
}

/// A predicate's two-by-two table in each stratum, strata in key order;
/// the whole batch's alone without strata.
fn tables(column: &bits::Column, groups: &[u16], strata: Option<&[u32]>, case: u16) -> Vec<Table> {
    let mut by_key: BTreeMap<u32, Table> = BTreeMap::new();
    for (index, &group) in groups.iter().enumerate() {
        let key = strata
            .and_then(|keys| keys.get(index))
            .copied()
            .unwrap_or(0);
        let table = by_key.entry(key).or_default();
        if !column.readable.contains(index) {
            continue;
        }
        let present = u32::from(column.present.contains(index));
        if group == case {
            table.case_read += 1;
            table.case_present += present;
        } else {
            table.rest_read += 1;
            table.rest_present += present;
        }
    }
    by_key.into_values().collect()
}

/// The batch with each stratum's charts contiguous: strata in key order,
/// charts in their own order within each, which is the order a stratified
/// shuffle already walks, so every permutation gives each chart the label
/// it gave before. A stratum's counts then cost its own words.
struct Contiguous {
    matrix: Matrix,
    groups: Vec<u16>,
    keys: Vec<u32>,
}

impl Contiguous {
    fn new(matrix: &Matrix, groups: &[u16], keys: &[u32]) -> Contiguous {
        let mut order: Vec<usize> = (0..keys.len()).collect();
        order.sort_by_key(|&at| keys.get(at).copied());
        Contiguous {
            matrix: matrix.reordered(&order),
            groups: order
                .iter()
                .filter_map(|&at| groups.get(at).copied())
                .collect(),
            keys: order
                .iter()
                .filter_map(|&at| keys.get(at).copied())
                .collect(),
        }
    }
}

/// Tests whether the design's groups differ on each predicate, by
/// permuting the labels (within strata when the design has them).
///
/// ```
/// # use teistro_research::{Cell, Contrast, Design, GroupTest, Matrix, compare};
/// #
/// # // Ten charts, the first five cases; the predicate holds on four cases
/// # // and one other. 252 ways of choosing five cases, each walked once.
/// # let cell = |holds: &u8| if *holds == 1 { Cell::Present } else { Cell::Absent };
/// # let mut matrix = Matrix::new(10);
/// # matrix.push("holds", &[1, 1, 1, 1, 0, 1, 0, 0, 0, 0].iter().map(cell).collect::<Vec<_>>())?;
/// # let design = Design::new(vec![1, 1, 1, 1, 1, 0, 0, 0, 0, 0]);
/// # let test = GroupTest::new(8, 9_999, Contrast::CaseVsRest { case: 1 });
/// # let tested = compare(&matrix, &design, &test)?;
/// # let row = &tested.rows[0];
///
/// // One label per chart, or the design is refused.
/// let short = Design::new(vec![1, 0]);
/// assert_eq!(compare(&matrix, &short, &test).unwrap_err().field(), Some("groups"));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
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
    let contiguous = design
        .strata
        .as_deref()
        .map(|keys| Contiguous::new(matrix, &design.groups, keys));
    let (laid, groups, keys) = match &contiguous {
        Some(laid) => (
            &laid.matrix,
            laid.groups.as_slice(),
            Some(laid.keys.as_slice()),
        ),
        None => (matrix, design.groups.as_slice(), None),
    };
    let layout = Layout::new(groups, group_count, keys);
    let statistic = engine::Statistic {
        contrast: test.contrast,
        alternative: test.alternative,
        strata: match test.contrast {
            Contrast::CaseVsRest { .. } => engine::Strata::of(&layout, laid),
            Contrast::AnyDifference => None,
        },
    };
    let observed = engine::observed(&layout, laid, &statistic);
    let order = engine::step_down_order(&observed);
    let labelled = engine::Labelled {
        layout: &layout,
        matrix: laid,
        statistic: &statistic,
        seed: test.seed,
    };
    let walk = Walk::of(test.shuffle, test.permutations, labelled.arrangements());
    let counted = engine::count(
        &labelled,
        &engine::Family::new(&observed, &order),
        walk,
        test.parallelism,
    )?;
    let assessed = assess(&counted, &order, walk, test.level, test.alpha);
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
    Ok(Tested::walked(rows, walk, test.shuffle))
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

impl Walk {
    /// The p-value of `b` statistics reaching the observed one:
    /// `(b + 1)/(m + 1)` over `m` draws, the observed labelling added to
    /// its own reference set; `b/N` over all `N`, which already hold it.
    #[allow(
        clippy::cast_precision_loss,
        reason = "both are at most MAX_PERMUTATIONS + 1, far inside f64's exact integers"
    )]
    pub(crate) fn p(self, b: u64) -> f64 {
        match self {
            Walk::Sampled(m) => p_value(b, u64::from(m)),
            Walk::Enumerated(n) => b as f64 / f64::from(n),
        }
    }

    /// The smallest p-value the walk can give: no draw reaching the
    /// observed statistic, or, walked whole, the observed arrangement
    /// alone, which always reaches itself.
    pub(crate) fn resolution(self) -> f64 {
        match self {
            Walk::Sampled(_) => self.p(0),
            Walk::Enumerated(_) => self.p(1),
        }
    }
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
    walk: Walk,
    level: f64,
    alpha: Option<f64>,
) -> Vec<Assessed> {
    let raw: Vec<f64> = counted.exceed.iter().map(|&b| walk.p(b)).collect();
    let mut running: f64 = 0.0;
    let max_t = correct::in_request_order(
        order
            .iter()
            .zip(&counted.step_down)
            .map(|(&j, &b)| {
                running = running.max(walk.p(b));
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
                let (low, high) = match walk {
                    Walk::Sampled(m) => special::clopper_pearson(exceed, u64::from(m), level),
                    Walk::Enumerated(_) => (value, value),
                };
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
                let tables = tables(
                    column,
                    &self.design.groups,
                    self.design.strata.as_deref(),
                    case,
                );
                let exact = every_read.then(|| exact(&tables, self.statistic, observed));
                let effect = Effect::of(
                    case_present,
                    case_read,
                    column.present_total - case_present,
                    column.readable_total - case_read,
                    self.z,
                )
                .map(|effect| Effect {
                    pooled: (tables.len() > 1).then(|| Pooled::of(&tables, self.z)),
                    ..effect
                });
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
///
/// ```
/// # use teistro_research::{Cell, Contrast, Design, GroupTest, Matrix, compare};
/// #
/// # // Ten charts, the first five cases; the predicate holds on four cases
/// # // and one other. 252 ways of choosing five cases, each walked once.
/// # let cell = |holds: &u8| if *holds == 1 { Cell::Present } else { Cell::Absent };
/// # let mut matrix = Matrix::new(10);
/// # matrix.push("holds", &[1, 1, 1, 1, 0, 1, 0, 0, 0, 0].iter().map(cell).collect::<Vec<_>>())?;
/// # let design = Design::new(vec![1, 1, 1, 1, 1, 0, 0, 0, 0, 0]);
/// # let test = GroupTest::new(8, 9_999, Contrast::CaseVsRest { case: 1 });
/// # let tested = compare(&matrix, &design, &test)?;
/// # let row = &tested.rows[0];
///
/// use teistro_research::counts;
///
/// let table = counts(&matrix, &design)?;
/// assert_eq!((table.rows[0].predicate.as_str(), table.rows[0].counts.len()), ("holds", 2));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
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
///
/// ```
/// # use teistro_research::{Cell, Contrast, Design, GroupTest, Matrix, compare};
/// #
/// # // Ten charts, the first five cases; the predicate holds on four cases
/// # // and one other. 252 ways of choosing five cases, each walked once.
/// # let cell = |holds: &u8| if *holds == 1 { Cell::Present } else { Cell::Absent };
/// # let mut matrix = Matrix::new(10);
/// # matrix.push("holds", &[1, 1, 1, 1, 0, 1, 0, 0, 0, 0].iter().map(cell).collect::<Vec<_>>())?;
/// # let design = Design::new(vec![1, 1, 1, 1, 1, 0, 0, 0, 0, 0]);
/// # let test = GroupTest::new(8, 9_999, Contrast::CaseVsRest { case: 1 });
/// # let tested = compare(&matrix, &design, &test)?;
/// # let row = &tested.rows[0];
///
/// use teistro_research::counts;
///
/// // The same counts a test's rows carry, with no shuffle.
/// assert_eq!(counts(&matrix, &design)?.rows[0].counts, row.counts);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
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
/// ```
/// # use teistro_research::{Cell, Contrast, Design, GroupTest, Matrix, compare};
/// #
/// # // Ten charts, the first five cases; the predicate holds on four cases
/// # // and one other. 252 ways of choosing five cases, each walked once.
/// # let cell = |holds: &u8| if *holds == 1 { Cell::Present } else { Cell::Absent };
/// # let mut matrix = Matrix::new(10);
/// # matrix.push("holds", &[1, 1, 1, 1, 0, 1, 0, 0, 0, 0].iter().map(cell).collect::<Vec<_>>())?;
/// # let design = Design::new(vec![1, 1, 1, 1, 1, 0, 0, 0, 0, 0]);
/// # let test = GroupTest::new(8, 9_999, Contrast::CaseVsRest { case: 1 });
/// # let tested = compare(&matrix, &design, &test)?;
/// # let row = &tested.rows[0];
///
/// use teistro_research::counts;
///
/// assert_eq!(counts(&matrix, &design)?.rows[0].counts[1].present, 4);
/// assert!(counts(&Matrix::new(10), &design).is_err());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
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
    ///
    /// ```
    /// # use teistro_research::{Cell, Contrast, Design, GroupTest, Matrix, compare};
    /// #
    /// # // Ten charts, the first five cases; the predicate holds on four cases
    /// # // and one other. 252 ways of choosing five cases, each walked once.
    /// # let cell = |holds: &u8| if *holds == 1 { Cell::Present } else { Cell::Absent };
    /// # let mut matrix = Matrix::new(10);
    /// # matrix.push("holds", &[1, 1, 1, 1, 0, 1, 0, 0, 0, 0].iter().map(cell).collect::<Vec<_>>())?;
    /// # let design = Design::new(vec![1, 1, 1, 1, 1, 0, 0, 0, 0, 0]);
    /// # let test = GroupTest::new(8, 9_999, Contrast::CaseVsRest { case: 1 });
    /// # let tested = compare(&matrix, &design, &test)?;
    /// # let row = &tested.rows[0];
    ///
    /// // Over the whole batch, as one group.
    /// assert_eq!((matrix.tallies()[0].present, matrix.tallies()[0].absent), (5, 5));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
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
