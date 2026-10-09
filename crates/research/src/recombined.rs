//! One group against its own recombined population (`research.md` §1.3).
//!
//! The façade refounds the sample `R` times, each replicate pairing one
//! subject's date and place with another's clock time (Gauquelin's
//! control), and counts every predicate on each. This crate reads those
//! counts: a predicate's expected share is its mean share over the
//! replicates, its statistic the observed share standardised by the
//! replicates' own spread, and its p-value the replicates' rank. Max-T
//! runs over the same replicates, which keep the predicates' real
//! dependence as permutations do.

use serde::{Deserialize, Serialize};
use teistro_core::error::Error;

use crate::engine::{Family, Permuted, count, step_down_order};
use crate::rng::ShuffleVersion;
use crate::{
    Alternative, Expectation, GroupCount, Parallelism, PredicateRow, Tested, assess,
    check_permutations, default_level, p_value,
};

/// The most replicates one study may refound.
pub const MAX_REPLICATES: u32 = 100_000;

/// How a sample's observed counts are read against its replicates.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ReplicateTest {
    /// Whether the predicate is looked for more often than the
    /// population, less, or either.
    #[serde(default)]
    pub alternative: Alternative,
    /// Every interval's confidence, in (0, 1).
    #[serde(default = "default_level")]
    pub level: f64,
    /// When given, which predicates fall under it by each method.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alpha: Option<f64>,
}

impl Default for ReplicateTest {
    fn default() -> ReplicateTest {
        ReplicateTest {
            alternative: Alternative::default(),
            level: default_level(),
            alpha: None,
        }
    }
}

/// One predicate's share of the charts it was read on: present over
/// present and absent.
fn share(present: u32, read: u32) -> Option<f64> {
    (read > 0).then(|| f64::from(present) / f64::from(read))
}

/// Each predicate's mean and spread over the replicates it was read on.
struct Moments {
    mean: Vec<f64>,
    spread: Vec<f64>,
}

impl Moments {
    fn of(predicates: usize, replicates: &[Vec<(u32, u32)>]) -> Moments {
        let mut mean = vec![0.0; predicates];
        let mut spread = vec![0.0; predicates];
        for j in 0..predicates {
            let shares: Vec<f64> = replicates
                .iter()
                .filter_map(|replicate| replicate.get(j))
                .filter_map(|&(present, read)| share(present, read))
                .collect();
            if shares.is_empty() {
                continue;
            }
            #[allow(clippy::cast_precision_loss)]
            let count = shares.len() as f64;
            let centre = shares.iter().sum::<f64>() / count;
            let variance = shares
                .iter()
                .map(|s| (s - centre) * (s - centre))
                .sum::<f64>()
                / count;
            if let (Some(m), Some(s)) = (mean.get_mut(j), spread.get_mut(j)) {
                *m = centre;
                *s = variance.sqrt();
            }
        }
        Moments { mean, spread }
    }

    /// The standardised share of predicate `j`, read the way `alternative`
    /// looks.
    fn statistic(&self, j: usize, share: Option<f64>, alternative: Alternative) -> f64 {
        let (Some(share), Some(&mean), Some(&spread)) =
            (share, self.mean.get(j), self.spread.get(j))
        else {
            return 0.0;
        };
        let z = if spread > 0.0 {
            (share - mean) / spread
        } else {
            // Every replicate agrees: the observed share either is theirs
            // or lies beyond all of them, as far as a finite number can
            // say (an infinity would not survive JSON).
            #[allow(
                clippy::float_cmp,
                reason = "both are the same division of counts, so an equal share is equal to the bit"
            )]
            let agrees = share == mean;
            if agrees {
                0.0
            } else {
                (share - mean).signum() * f64::MAX
            }
        };
        match alternative {
            Alternative::Greater => z,
            Alternative::Less => -z,
            Alternative::TwoSided => z.abs(),
        }
    }
}

/// The replicates as a family of statistics, replicate `k` standing where
/// a group test's permutation `k` does.
struct Replicas<'a> {
    replicates: &'a [Vec<(u32, u32)>],
    moments: &'a Moments,
    alternative: Alternative,
}

impl Permuted for Replicas<'_> {
    type Scratch = ();

    fn scratch(&self) {}

    fn read(&self, k: u64, (): &mut (), out: &mut [f64]) -> Result<(), Error> {
        let replicate = usize::try_from(k).ok().and_then(|k| self.replicates.get(k));
        for (j, slot) in out.iter_mut().enumerate() {
            let at = replicate
                .and_then(|r| r.get(j))
                .and_then(|&(p, n)| share(p, n));
            *slot = self.moments.statistic(j, at, self.alternative);
        }
        Ok(())
    }
}

/// Reads a sample's counts against its recombined replicates: one name
/// and one observed count per predicate, and per replicate each
/// predicate's `(present, read)`.
///
/// # Errors
///
/// `INVALID_ARG` naming its field: no predicate, or names and counts not
/// one per predicate (`predicates`); no replicate, more than
/// [`MAX_REPLICATES`], or too few to reach `alpha` over the family
/// (`replicates`); a replicate without one count per predicate
/// (`replicates[i]`); `level` or `alpha` outside (0, 1).
pub fn replicated(
    names: &[String],
    observed: &[GroupCount],
    replicates: &[Vec<(u32, u32)>],
    test: &ReplicateTest,
) -> Result<Tested, Error> {
    let predicates = names.len();
    if predicates == 0 || observed.len() != predicates {
        return Err(Error::invalid_arg(format!(
            "{predicates} names for {} observed counts; give one of each per predicate",
            observed.len()
        ))
        .with_field("predicates"));
    }
    let count_of = u32::try_from(replicates.len()).unwrap_or(u32::MAX);
    if count_of > MAX_REPLICATES {
        return Err(Error::invalid_arg(format!(
            "{count_of} replicates is above the most a study may refound, {MAX_REPLICATES}"
        ))
        .with_field("replicates"));
    }
    check_permutations(count_of, test.level, test.alpha, predicates).map_err(
        |error| match error.field() {
            Some("permutations") => error.with_field("replicates"),
            _ => error,
        },
    )?;
    if let Some(short) = replicates.iter().position(|r| r.len() != predicates) {
        return Err(Error::invalid_arg(format!(
            "replicate {short} counts {} predicates of {predicates}",
            replicates.get(short).map_or(0, Vec::len)
        ))
        .with_field(format!("replicates[{short}]")));
    }
    let moments = Moments::of(predicates, replicates);
    let shares: Vec<Option<f64>> = observed
        .iter()
        .map(|c| share(c.present, c.present + c.absent))
        .collect();
    let statistics: Vec<f64> = shares
        .iter()
        .enumerate()
        .map(|(j, &s)| moments.statistic(j, s, test.alternative))
        .collect();
    let order = step_down_order(&statistics);
    let replicas = Replicas {
        replicates,
        moments: &moments,
        alternative: test.alternative,
    };
    let counted = count(
        &replicas,
        &Family::new(&statistics, &order),
        count_of,
        Parallelism::One,
    )?;
    let assessed = assess(&counted, &order, count_of, test.level, test.alpha);
    let rows = names
        .iter()
        .zip(observed)
        .zip(&shares)
        .zip(&statistics)
        .zip(assessed)
        .enumerate()
        .map(|(j, ((((name, &counts), &own), &statistic), assessed))| {
            let expected = moments.mean.get(j).copied().unwrap_or(0.0);
            PredicateRow {
                predicate: name.clone(),
                counts: vec![counts],
                observed: statistic,
                p: assessed.p,
                exact: None,
                adjusted: assessed.adjusted,
                effect: None,
                expected: own.map(|own| Expectation {
                    observed: own,
                    expected,
                    ratio: (expected > 0.0).then(|| own / expected),
                }),
                under_alpha: assessed.under_alpha,
            }
        })
        .collect();
    Ok(Tested {
        rows,
        permutations: count_of,
        resolution: p_value(0, u64::from(count_of)),
        shuffle: ShuffleVersion::V1,
    })
}
