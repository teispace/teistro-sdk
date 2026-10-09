//! Effect sizes and their intervals (`research.md` §1.6): a p-value says
//! nothing about size, so every two-group row carries these.

use serde::{Deserialize, Serialize};
use teistro_core::math;

/// An estimate with its interval at the test's confidence level.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
            let ratio = case.estimate / rest.estimate;
            let spread = z * (1.0 / a - 1.0 / n1 + 1.0 / b - 1.0 / n0).sqrt();
            let ln_ratio = math::ln(ratio);
            Interval {
                estimate: ratio,
                low: math::exp(ln_ratio - spread),
                high: math::exp(ln_ratio + spread),
            }
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
        })
    }
}
