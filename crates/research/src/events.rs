//! Events in lives against the shuffled-event null (`research.md` §1.4).
//!
//! Each subject is a birth with one dated event. The façade evaluates each
//! predicate on every subject *at every subject's event* (person `i` at
//! the instant event `j` would have under the study's shuffle), into one
//! `N × N` bitset per predicate. The observed statistic reads the
//! diagonal, `i` at its own event; a permutation `π` reads `i` at
//! `π(i)`'s. Nothing here knows which shuffle built the matrix: that is
//! the façade's, and the answer names it.

use serde::{Deserialize, Serialize};
use teistro_core::error::Error;

use crate::bits::Bits;
use crate::engine::{Layout, Permuted, step_down_order};
use crate::rng::{ShuffleVersion, SplitMix64};
use crate::{Alternative, Expectation, GroupCount, Parallelism, PredicateRow, Tested, p_value};

/// The most subjects an event study may have: its matrices are `N²`
/// bits a predicate.
pub const MAX_SUBJECTS: usize = 20_000;

/// The most attempts one permutation may make to draw a pairing that puts
/// no event before its birth, under [`AfterBirth::RestrictPairings`].
pub const MAX_ATTEMPTS: u32 = 10_000;

/// What a pair a permutation may not make is: an event placed before the
/// birth it is paired with.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum AfterBirth {
    /// Refuse a study in which any pairing would put an event before a
    /// birth: every permutation is then possible.
    #[default]
    Refuse,
    /// Permute only among pairings that keep every event after its birth,
    /// each drawn uniformly by rejection. This changes the reference set,
    /// which the answer says.
    RestrictPairings,
}

/// One predicate over every subject at every event.
#[derive(Clone, Debug)]
pub(crate) struct PairColumn {
    name: String,
    /// Row `i`, column `j` at `i * n + j`.
    present: Bits,
    readable: Bits,
}

/// The study's predicates over subjects by events, built once.
#[derive(Clone, Debug)]
pub struct PairMatrix {
    subjects: usize,
    /// The pairs a permutation may make; every pair unless the façade
    /// marks one (an event before a birth).
    allowed: Bits,
    columns: Vec<PairColumn>,
}

use crate::bits::Cell;

impl PairMatrix {
    /// A matrix over `subjects` subjects, every pairing allowed.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on `subjects` when there are fewer than two or more
    /// than [`MAX_SUBJECTS`].
    pub fn new(subjects: usize) -> Result<PairMatrix, Error> {
        if !(2..=MAX_SUBJECTS).contains(&subjects) {
            return Err(Error::invalid_arg(format!(
                "an event study has 2 to {MAX_SUBJECTS} subjects, not {subjects}"
            ))
            .with_field("subjects"));
        }
        let mut allowed = Bits::empty(subjects * subjects);
        for index in 0..subjects * subjects {
            allowed.insert(index);
        }
        Ok(PairMatrix {
            subjects,
            allowed,
            columns: Vec::new(),
        })
    }

    /// The study's size.
    #[must_use]
    pub const fn subjects(&self) -> usize {
        self.subjects
    }

    /// Forbids pairing subject `subject` with event `event`, because the
    /// event would come before the birth.
    pub fn forbid(&mut self, subject: usize, event: usize) {
        if subject < self.subjects && event < self.subjects {
            self.allowed.remove(subject * self.subjects + event);
        }
    }

    /// Whether any pairing is forbidden.
    #[must_use]
    pub fn restricted(&self) -> bool {
        self.allowed.count() as usize != self.subjects * self.subjects
    }

    /// Adds a predicate under `name`: `cell(i, j)` says what it is for
    /// subject `i` at event `j`.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on `predicates[i]` when the name is an earlier
    /// predicate's.
    pub fn push(
        &mut self,
        name: impl Into<String>,
        mut cell: impl FnMut(usize, usize) -> Cell,
    ) -> Result<(), Error> {
        let name = name.into();
        if self.columns.iter().any(|column| column.name == name) {
            return Err(Error::invalid_arg(format!(
                "predicate `{name}` is named twice; a family names each predicate once"
            ))
            .with_field(format!("predicates[{}]", self.columns.len())));
        }
        let n = self.subjects;
        let mut present = Bits::empty(n * n);
        let mut readable = Bits::empty(n * n);
        for i in 0..n {
            for j in 0..n {
                match cell(i, j) {
                    Cell::Present => {
                        present.insert(i * n + j);
                        readable.insert(i * n + j);
                    }
                    Cell::Absent => readable.insert(i * n + j),
                    Cell::Unreadable | Cell::Unstable => {}
                }
            }
        }
        self.columns.push(PairColumn {
            name,
            present,
            readable,
        });
        Ok(())
    }
}

/// A test of events against the shuffled-event null.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EventTest {
    /// The study's seed; required.
    pub seed: u64,
    /// How many permutations, 1 to [`MAX_PERMUTATIONS`].
    pub permutations: u32,
    /// Whether the predicate is looked for more often than chance, less,
    /// or either.
    #[serde(default)]
    pub alternative: Alternative,
    /// What to do with a pairing that puts an event before a birth.
    #[serde(default)]
    pub after_birth: AfterBirth,
    /// Every interval's confidence, in (0, 1).
    #[serde(default = "crate::default_level")]
    pub level: f64,
    /// When given, which predicates fall under it by each method.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alpha: Option<f64>,
    /// How many threads.
    #[serde(default)]
    pub parallelism: Parallelism,
    /// The generator and shuffle.
    #[serde(default)]
    pub shuffle: ShuffleVersion,
}

impl EventTest {
    /// A two-sided test over `permutations` permutations from `seed`.
    #[must_use]
    pub fn new(seed: u64, permutations: u32) -> EventTest {
        EventTest {
            seed,
            permutations,
            alternative: Alternative::default(),
            after_birth: AfterBirth::default(),
            level: crate::default_level(),
            alpha: None,
            parallelism: Parallelism::default(),
            shuffle: ShuffleVersion::default(),
        }
    }
}

/// An event study's pairings, permuted inside strata, and the
/// standardised count each predicate reads off them.
struct Paired<'a> {
    matrix: &'a PairMatrix,
    layout: &'a Layout<'a>,
    seed: u64,
    restricted: bool,
    alternative: Alternative,
    /// Each predicate's share over every allowed pair it was read on:
    /// what a random pairing expects.
    null: Vec<f64>,
}

/// One thread's pairing and stratum buffer.
struct PairScratch {
    pairing: Vec<u16>,
    stratum: Vec<u16>,
}

impl Paired<'_> {
    /// Every predicate's standardised count under `pairing`.
    fn statistics(&self, pairing: &[u16], out: &mut [f64]) {
        let n = self.matrix.subjects;
        for ((column, &share), slot) in self.matrix.columns.iter().zip(&self.null).zip(out) {
            let (mut present, mut read) = (0_u32, 0_u32);
            for (i, &j) in pairing.iter().enumerate() {
                let cell = i * n + usize::from(j);
                present += u32::from(column.present.contains(cell));
                read += u32::from(column.readable.contains(cell));
            }
            let z = standardised(present, read, share);
            *slot = match self.alternative {
                Alternative::Greater => z,
                Alternative::Less => -z,
                Alternative::TwoSided => z.abs(),
            };
        }
    }

    fn allowed(&self, pairing: &[u16]) -> bool {
        let n = self.matrix.subjects;
        pairing
            .iter()
            .enumerate()
            .all(|(i, &j)| self.matrix.allowed.contains(i * n + usize::from(j)))
    }
}

/// How far `present` of `read` pairs departs from the share a random
/// pairing expects, in standard deviations of a binomial at that share.
fn standardised(present: u32, read: u32, share: f64) -> f64 {
    let read = f64::from(read);
    let variance = read * share * (1.0 - share);
    if variance <= 0.0 {
        return 0.0;
    }
    (f64::from(present) - read * share) / variance.sqrt()
}

impl Permuted for Paired<'_> {
    type Scratch = PairScratch;

    fn scratch(&self) -> PairScratch {
        PairScratch {
            pairing: vec![0; self.matrix.subjects],
            stratum: Vec::new(),
        }
    }

    fn read(&self, k: u64, scratch: &mut PairScratch, out: &mut [f64]) -> Result<(), Error> {
        let mut rng = SplitMix64::for_permutation(self.seed, k);
        let mut attempts = 0;
        loop {
            self.layout
                .permute_with(&mut rng, &mut scratch.pairing, &mut scratch.stratum);
            if !self.restricted || self.allowed(&scratch.pairing) {
                break;
            }
            attempts += 1;
            if attempts >= MAX_ATTEMPTS {
                return Err(Error::invalid_arg(format!(
                    "permutation {k} found no pairing that keeps every event after its birth in {MAX_ATTEMPTS} draws"
                ))
                .with_field("afterBirth")
                .with_hint("stratify by birth cohort, so that a shuffle stays among people born before each other's events"));
            }
        }
        self.statistics(&scratch.pairing, out);
        Ok(())
    }
}

/// Refuses strata that are not one per subject, or in which no event can
/// move.
fn check_strata(strata: &[u32], n: usize) -> Result<(), Error> {
    if strata.len() != n {
        return Err(Error::invalid_arg(format!(
            "{} strata for {n} subjects; give one stratum per subject",
            strata.len()
        ))
        .with_field("strata"));
    }
    let mut sizes: std::collections::BTreeMap<u32, usize> = std::collections::BTreeMap::new();
    for &key in strata {
        *sizes.entry(key).or_default() += 1;
    }
    if sizes.values().all(|&size| size < 2) {
        return Err(
            Error::invalid_arg("every stratum holds one subject, so no event can move")
                .with_field("strata"),
        );
    }
    Ok(())
}

/// One predicate's row: its own count on the diagonal, each subject at
/// its own event, against what a random pairing expects.
fn diagonal_row(
    column: &PairColumn,
    n: usize,
    share: f64,
    observed: f64,
    assessed: crate::Assessed,
) -> PredicateRow {
    let mut count = GroupCount::default();
    for i in 0..n {
        let cell = i * n + i;
        if column.present.contains(cell) {
            count.present += 1;
        } else if column.readable.contains(cell) {
            count.absent += 1;
        } else {
            count.unreadable += 1;
        }
    }
    let read = count.present + count.absent;
    let own = if read == 0 {
        0.0
    } else {
        f64::from(count.present) / f64::from(read)
    };
    PredicateRow {
        predicate: column.name.clone(),
        counts: vec![count],
        observed: Some(observed),
        p: assessed.p,
        exact: None,
        adjusted: assessed.adjusted,
        effect: None,
        expected: Some(Expectation {
            observed: own,
            expected: share,
            ratio: (share > 0.0).then(|| own / share),
        }),
        under_alpha: assessed.under_alpha,
    }
}

/// Tests whether each predicate holds at the subjects' own events more
/// (or less) often than at events shuffled among them, the shuffle inside
/// `strata` when they are given (`research.md` §1.4).
///
/// # Errors
///
/// `INVALID_ARG` naming its field: strata not one per subject, or none
/// that holds two subjects (`strata`); a forbidden pairing under
/// [`AfterBirth::Refuse`], or a restricted pairing rejection cannot meet
/// (`afterBirth`, with a hint); permutations outside 1 to
/// [`MAX_PERMUTATIONS`], or too few to reach `alpha` over the family
/// (`permutations`); `level` or `alpha` outside (0, 1); no predicate
/// (`predicates`).
pub fn timed(
    matrix: &PairMatrix,
    strata: Option<&[u32]>,
    test: &EventTest,
) -> Result<Tested, Error> {
    let n = matrix.subjects;
    if matrix.columns.is_empty() {
        return Err(Error::invalid_arg("the study names no predicate").with_field("predicates"));
    }
    crate::check_permutations(
        test.permutations,
        test.level,
        test.alpha,
        matrix.columns.len(),
    )?;
    if let Some(strata) = strata {
        check_strata(strata, n)?;
    }
    let restricted = matrix.restricted();
    if restricted && test.after_birth == AfterBirth::Refuse {
        return Err(Error::invalid_arg(
            "a shuffled pairing would put an event before a birth",
        )
        .with_field("afterBirth")
        .with_hint("RESTRICT_PAIRINGS permutes only among pairings that keep every event after its birth"));
    }
    let identity: Vec<u16> = (0..n)
        .map(|i| u16::try_from(i).unwrap_or(u16::MAX))
        .collect();
    let layout = Layout::new(&identity, 0, strata);
    let null: Vec<f64> = matrix
        .columns
        .iter()
        .map(|column| {
            let mut present = column.present.clone();
            present.intersect(&matrix.allowed);
            let read = column.readable.common(&matrix.allowed);
            if read == 0 {
                0.0
            } else {
                f64::from(present.count()) / f64::from(read)
            }
        })
        .collect();
    let paired = Paired {
        matrix,
        layout: &layout,
        seed: test.seed,
        restricted,
        alternative: test.alternative,
        null,
    };
    let mut observed = vec![0.0; matrix.columns.len()];
    paired.statistics(&identity, &mut observed);
    let order = step_down_order(&observed);
    let counted = crate::engine::count(
        &paired,
        &crate::engine::Family::new(&observed, &order),
        test.permutations,
        test.parallelism,
    )?;
    let assessed = crate::assess(&counted, &order, test.permutations, test.level, test.alpha);
    let rows = matrix
        .columns
        .iter()
        .zip(&paired.null)
        .zip(&observed)
        .zip(assessed)
        .map(|(((column, &share), &observed), assessed)| {
            diagonal_row(column, n, share, observed, assessed)
        })
        .collect();
    Ok(Tested {
        rows,
        permutations: test.permutations,
        resolution: p_value(0, u64::from(test.permutations)),
        shuffle: test.shuffle,
    })
}
