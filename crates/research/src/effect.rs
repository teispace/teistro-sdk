//! Effect sizes and their intervals (`research.md` §1.6): a p-value says
//! nothing about size, so every two-group row carries these.

use serde::{Deserialize, Serialize};
use teistro_core::math;

use crate::special;

/// An estimate with its interval at the test's confidence level.
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
/// let effect = row.effect.ok_or_else(|| teistro_core::error::Error::internal("an effect"))?;
/// let case = effect.risk_case;
/// assert_eq!(case.estimate, 0.8);
/// assert!(case.low < 0.8 && 0.8 < case.high);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "EffectEstimate"))]
pub struct Interval {
    /// The point estimate.
    pub estimate: f64,
    /// The interval's lower end.
    pub low: f64,
    /// The interval's upper end.
    pub high: f64,
}

/// How much commoner a predicate is among the cases than among the rest,
/// over the charts it was read on.
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
/// // Four of five cases against one of five others.
/// let effect = row.effect.ok_or_else(|| teistro_core::error::Error::internal("an effect"))?;
/// assert!((effect.risk_difference.estimate - 0.6).abs() < 1e-12);
/// assert_eq!(effect.risk_ratio.map(|ratio| ratio.estimate), Some(4.0));
/// assert_eq!(effect.odds_ratio, Some(16.0));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Effect {
    /// The share of the cases the predicate holds on, with Wilson's
    /// score interval.
    pub risk_case: Interval,
    /// The share of the rest, with Wilson's score interval.
    pub risk_rest: Interval,
    /// The cases' share less the rest's, with Newcombe's hybrid score
    /// interval (his method 10, *Stat. Med.* 17, 1998), built from the
    /// two Wilson intervals.
    pub risk_difference: Interval,
    /// The cases' share over the rest's, with Katz's log interval
    /// (*Biometrics* 34, 1978). Absent when either group has no chart the
    /// predicate holds on, because the log interval has no width there and
    /// no continuity correction is applied silently.
    pub risk_ratio: Option<Interval>,
    /// The odds ratio, absent when a cell of the two-by-two table is
    /// empty.
    pub odds_ratio: Option<f64>,
    /// Cohen's *h*, 2·asin√p₁ − 2·asin√p₀, the difference on the scale
    /// where a proportion's sampling variance does not depend on it, for
    /// comparing studies (*Statistical Power Analysis*, 2nd ed., 1988,
    /// ch. 6).
    pub cohen_h: f64,
    /// The ratios pooled over the design's strata, for a design that has
    /// them. Beside the crude ratios above, the difference is the
    /// confounding the strata removed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pooled: Option<Pooled>,
}

/// The cases against the rest within strata, pooled (`research.md` §1.6).
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
/// // Pooled only for a design with strata.
/// assert!(row.effect.is_some_and(|effect| effect.pooled.is_none()));
/// let strata = design.clone().with_strata(vec![0, 0, 0, 1, 1, 0, 0, 0, 1, 1]);
/// let pooled = compare(&matrix, &strata, &test)?.rows[0].effect.and_then(|effect| effect.pooled);
/// assert!(pooled.is_some());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Pooled {
    /// Mantel and Haenszel's pooled risk ratio, with Greenland and
    /// Robins's log interval (*Biometrics* 41, 1985). Absent when no
    /// stratum has a chart the predicate holds on in one of the groups.
    pub risk_ratio: Option<Interval>,
    /// Mantel and Haenszel's pooled odds ratio, with Robins, Breslow and
    /// Greenland's log interval (*Biometrics* 42, 1986). Absent when every
    /// stratum's cross product on one side is zero.
    pub odds_ratio: Option<Interval>,
    /// Whether the strata share one odds ratio: Breslow and Day's
    /// statistic with Tarone's correction (*Biometrika* 72, 1985), its p
    /// on K − 1 degrees of freedom over the K strata in which both groups
    /// were read and the predicate both holds and does not. A small p says
    /// the pooled ratio averages effects that differ. Absent under two
    /// such strata, or without a pooled odds ratio.
    pub homogeneity_p: Option<f64>,
}

/// One stratum's two-by-two table: the predicate among the cases and
/// among the rest, over the charts it was read on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Table {
    pub(crate) case_present: u32,
    pub(crate) case_read: u32,
    pub(crate) rest_present: u32,
    pub(crate) rest_read: u32,
}

impl Table {
    /// The charts read, and those the predicate holds on.
    pub(crate) fn margins(self) -> (u32, u32) {
        (
            self.case_read + self.rest_read,
            self.case_present + self.rest_present,
        )
    }
}

/// An estimate and its interval from a log-scale variance.
fn log_interval(estimate: f64, variance: f64, z: f64) -> Interval {
    let spread = z * variance.max(0.0).sqrt();
    let ln_estimate = math::ln(estimate);
    Interval {
        estimate,
        low: math::exp(ln_estimate - spread),
        high: math::exp(ln_estimate + spread),
    }
}

impl Pooled {
    /// The pooled ratios over `tables`, one per stratum, at the normal
    /// quantile `z`.
    #[allow(
        clippy::many_single_char_names,
        reason = "the names are the two-by-two table's own, as the papers print them"
    )]
    pub(crate) fn of(tables: &[Table], z: f64) -> Pooled {
        let (mut rr_top, mut rr_bottom, mut rr_spread) = (0.0, 0.0, 0.0);
        let (mut r_sum, mut s_sum, mut pr, mut ps_qr, mut qs) = (0.0, 0.0, 0.0, 0.0, 0.0);
        for table in tables {
            let (read, present) = table.margins();
            if read == 0 {
                continue;
            }
            let n = f64::from(read);
            let (a, n1) = (f64::from(table.case_present), f64::from(table.case_read));
            let (b, n0) = (f64::from(table.rest_present), f64::from(table.rest_read));
            let (c, d) = (n1 - a, n0 - b);
            rr_top += a * n0 / n;
            rr_bottom += b * n1 / n;
            rr_spread += (f64::from(present) * n1 * n0 - a * b * n) / (n * n);
            let (p, q) = ((a + d) / n, (b + c) / n);
            let (r, s) = (a * d / n, b * c / n);
            r_sum += r;
            s_sum += s;
            pr += p * r;
            ps_qr += p * s + q * r;
            qs += q * s;
        }
        let risk_ratio = (rr_top > 0.0 && rr_bottom > 0.0)
            .then(|| log_interval(rr_top / rr_bottom, rr_spread / (rr_top * rr_bottom), z));
        let odds_ratio = (r_sum > 0.0 && s_sum > 0.0).then(|| {
            let variance = pr / (2.0 * r_sum * r_sum)
                + ps_qr / (2.0 * r_sum * s_sum)
                + qs / (2.0 * s_sum * s_sum);
            log_interval(r_sum / s_sum, variance, z)
        });
        Pooled {
            risk_ratio,
            odds_ratio,
            homogeneity_p: odds_ratio.and_then(|ratio| breslow_day(tables, ratio.estimate)),
        }
    }
}

/// Breslow and Day's homogeneity test with Tarone's correction, under the
/// common odds ratio `psi`: each informative stratum's case count against
/// the one its margins give at `psi`, found by bisection to the last bit.
fn breslow_day(tables: &[Table], psi: f64) -> Option<f64> {
    let (mut statistic, mut observed, mut fitted, mut variance) = (0.0, 0.0, 0.0, 0.0);
    let mut informative = 0_u32;
    for table in tables {
        let (read, present) = table.margins();
        if table.case_read == 0 || table.rest_read == 0 || present == 0 || present == read {
            continue;
        }
        informative += 1;
        let (n1, n0, m1) = (
            f64::from(table.case_read),
            f64::from(table.rest_read),
            f64::from(present),
        );
        // The case count `x` whose table has odds ratio `psi`; the ratio
        // rises from 0 to infinity across the count's support.
        let odds = |x: f64| x * (n0 - m1 + x) / ((n1 - x) * (m1 - x));
        let fit = special::bisect((m1 - n0).max(0.0), n1.min(m1), psi, odds);
        let spread =
            1.0 / (1.0 / fit + 1.0 / (n1 - fit) + 1.0 / (m1 - fit) + 1.0 / (n0 - m1 + fit));
        let a = f64::from(table.case_present);
        statistic += (a - fit) * (a - fit) / spread;
        observed += a;
        fitted += fit;
        variance += spread;
    }
    if informative < 2 || variance <= 0.0 {
        return None;
    }
    let tarone = statistic - (observed - fitted) * (observed - fitted) / variance;
    Some(special::chi_square_upper(tarone, informative - 1))
}

/// Wilson's score interval on `present` of `read`, at the normal
/// quantile `z`.
fn wilson(present: u32, read: u32, z: f64) -> Interval {
    let n = f64::from(read);
    let p = f64::from(present) / n;
    let z2 = z * z;
    let scale = 1.0 + z2 / n;
    let centre = (p + z2 / (2.0 * n)) / scale;
    let half = z / scale * (p * (1.0 - p) / n + z2 / (4.0 * n * n)).sqrt();
    Interval {
        estimate: p,
        low: (centre - half).max(0.0),
        high: (centre + half).min(1.0),
    }
}

impl Effect {
    /// The effect of a predicate holding on `case_present` of
    /// `case_read` cases and `rest_present` of `rest_read` others, at the
    /// normal quantile `z`. Absent when either group has no chart the
    /// predicate was read on.
    pub(crate) fn of(
        case_present: u32,
        case_read: u32,
        rest_present: u32,
        rest_read: u32,
        z: f64,
    ) -> Option<Effect> {
        if case_read == 0 || rest_read == 0 {
            return None;
        }
        let case = wilson(case_present, case_read, z);
        let rest = wilson(rest_present, rest_read, z);
        let difference = case.estimate - rest.estimate;
        let risk_difference = Interval {
            estimate: difference,
            low: difference - math::hypot(case.estimate - case.low, rest.high - rest.estimate),
            high: difference + math::hypot(case.high - case.estimate, rest.estimate - rest.low),
        };
        let (a, n1, b, n0) = (
            f64::from(case_present),
            f64::from(case_read),
            f64::from(rest_present),
            f64::from(rest_read),
        );
        let risk_ratio = (case_present > 0 && rest_present > 0).then(|| {
            log_interval(
                case.estimate / rest.estimate,
                1.0 / a - 1.0 / n1 + 1.0 / b - 1.0 / n0,
                z,
            )
        });
        let odds_ratio = (case_present > 0
            && rest_present > 0
            && case_present < case_read
            && rest_present < rest_read)
            .then(|| (a * (n0 - b)) / (b * (n1 - a)));
        let cohen_h =
            2.0 * math::asin(case.estimate.sqrt()) - 2.0 * math::asin(rest.estimate.sqrt());
        Some(Effect {
            risk_case: case,
            risk_rest: rest,
            risk_difference,
            risk_ratio,
            odds_ratio,
            cohen_h,
            pooled: None,
        })
    }
}
