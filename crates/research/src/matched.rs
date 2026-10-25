//! Self-matched referents: the case-crossover design (Maclure, *Am. J.
//! Epidemiol.* 133, 1991; `research.md` §1.4, C367).
//!
//! Each subject is its own control. The façade reads every predicate at
//! the subject's event and at referent instants of the subject's own
//! life (time-stratified, Janes, Sheppard and Lumley, *Epidemiology* 16,
//! 2005), the event first. Under the null the event is as likely at any
//! of them, so a permutation draws, for each subject alone, which of its
//! instants is the event. Everything fixed about the person, the chart
//! included, is matched away.

use teistro_core::error::Error;

use crate::bits::{Bits, Cell};
use crate::effect::{Effect, Pooled, Table};
use crate::engine::{Permuted, Walk, product, reaches, step_down_order};
use crate::events::{AfterBirth, EventTest, MAX_SUBJECTS};
use crate::rng::{Digits, Draw, SplitMix64};
use crate::{Alternative, Expectation, GroupCount, PredicateRow, Tested, special};

/// The most instants one subject may be read at, its event included.
///
/// ```
/// use teistro_research::{MAX_INSTANTS, MatchedMatrix};
///
/// assert_eq!(MatchedMatrix::new(&[MAX_INSTANTS + 1, 2]).unwrap_err().field(), Some("subjects[0]"));
/// assert!(MatchedMatrix::new(&[MAX_INSTANTS, 2]).is_ok());
/// ```
pub const MAX_INSTANTS: u16 = 1_000;

/// The most subjects whose exposure varies across their instants for
/// which the exact p is convolved: its cost is their count squared.
const EXACT_SUBJECTS: usize = 5_000;

/// One predicate over every subject's instants.
#[derive(Clone, Debug)]
struct MatchedColumn {
    name: String,
    present: Bits,
    readable: Bits,
}

/// The study's predicates over each subject's instants, the event first,
/// built once.
///
/// ```
/// # use teistro_research::{Alternative, Cell, EventTest, MatchedMatrix, matched};
/// #
/// # // Four subjects, each read at its event and two referents; the predicate
/// # // holds at every event and no referent. 3⁴ = 81 placements of the events.
/// # let mut matrix = MatchedMatrix::new(&[3, 3, 3, 3])?;
/// # matrix.push("at the event", |_, instant| if instant == 0 { Cell::Present } else { Cell::Absent })?;
/// # let test = EventTest { alternative: Alternative::Greater, ..EventTest::new(7, 9_999) };
///
/// assert_eq!(matrix.subjects(), 4);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug)]
pub struct MatchedMatrix {
    /// How many instants each subject is read at.
    instants: Vec<u16>,
    /// Where each subject's instants start in a column.
    starts: Vec<usize>,
    cells: usize,
    columns: Vec<MatchedColumn>,
}

impl MatchedMatrix {
    /// A matrix over subjects read at `instants[i]` instants each, the
    /// event the first of them.
    ///
    /// ```
    /// use teistro_research::MatchedMatrix;
    ///
    /// // A subject with no instant has no event.
    /// assert_eq!(MatchedMatrix::new(&[3, 0]).unwrap_err().field(), Some("subjects[1]"));
    /// assert_eq!(MatchedMatrix::new(&[3]).unwrap_err().field(), Some("subjects"));
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on `subjects` when there are fewer than two or more
    /// than [`MAX_SUBJECTS`], and on `subjects[i]` when a subject has no
    /// instant or more than [`MAX_INSTANTS`].
    pub fn new(instants: &[u16]) -> Result<MatchedMatrix, Error> {
        if !(2..=MAX_SUBJECTS).contains(&instants.len()) {
            return Err(Error::invalid_arg(format!(
                "a self-matched study has 2 to {MAX_SUBJECTS} subjects, not {}",
                instants.len()
            ))
            .with_field("subjects"));
        }
        let mut starts = Vec::with_capacity(instants.len());
        let mut cells = 0;
        for (index, &count) in instants.iter().enumerate() {
            if !(1..=MAX_INSTANTS).contains(&count) {
                return Err(Error::invalid_arg(format!(
                    "subject {index} is read at {count} instants; a subject has its event and at most {} referents",
                    MAX_INSTANTS - 1
                ))
                .with_field(format!("subjects[{index}]")));
            }
            starts.push(cells);
            cells += usize::from(count);
        }
        Ok(MatchedMatrix {
            instants: instants.to_vec(),
            starts,
            cells,
            columns: Vec::new(),
        })
    }

    /// The study's size.
    ///
    /// ```
    /// use teistro_research::MatchedMatrix;
    ///
    /// assert_eq!(MatchedMatrix::new(&[2, 5, 3])?.subjects(), 3);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn subjects(&self) -> usize {
        self.instants.len()
    }

    /// Adds a predicate under `name`: `cell(i, k)` says what it is for
    /// subject `i` at its instant `k`, the event at `k = 0`.
    ///
    /// ```
    /// use teistro_research::{Cell, MatchedMatrix};
    ///
    /// let mut matrix = MatchedMatrix::new(&[2, 2])?;
    /// matrix.push("eclipse", |subject, instant| if subject == instant { Cell::Present } else { Cell::Absent })?;
    /// assert!(matrix.push("eclipse", |_, _| Cell::Absent).is_err());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
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
        let mut present = Bits::empty(self.cells);
        let mut readable = Bits::empty(self.cells);
        for (i, (&count, &start)) in self.instants.iter().zip(&self.starts).enumerate() {
            for k in 0..usize::from(count) {
                match cell(i, k) {
                    Cell::Present => {
                        present.insert(start + k);
                        readable.insert(start + k);
                    }
                    Cell::Absent => readable.insert(start + k),
                    Cell::Unreadable | Cell::Unstable => {}
                }
            }
        }
        self.columns.push(MatchedColumn {
            name,
            present,
            readable,
        });
        Ok(())
    }
}

/// One subject's exposure on one predicate: whether it holds at the
/// event, and at how many of the subject's instants. A subject is read
/// only when every one of its instants was.
#[derive(Clone, Copy)]
struct Exposure {
    at_event: bool,
    present: u16,
    instants: u16,
}

impl Exposure {
    /// The chance the drawn instant holds, under the null.
    fn share(self) -> f64 {
        f64::from(self.present) / f64::from(self.instants)
    }
}

/// Each predicate's subjects, the null's mean and variance of the count
/// at the drawn instants, and what a permutation reads.
struct Matched<'a> {
    matrix: &'a MatchedMatrix,
    seed: u64,
    alternative: Alternative,
    /// Per predicate, each subject's exposure; `None` where unread.
    exposures: Vec<Vec<Option<Exposure>>>,
    moments: Vec<(f64, f64)>,
}

impl Matched<'_> {
    /// Every predicate's statistic with each subject's event drawn among
    /// its instants by `draw`.
    fn draw(&self, draw: &mut impl Draw, labels: &mut [u16], out: &mut [f64]) {
        for (label, &count) in labels.iter_mut().zip(&self.matrix.instants) {
            *label = u16::try_from(draw.below(u64::from(count))).unwrap_or(0);
        }
        self.statistics(labels, out);
    }

    /// Every predicate's standardised count with subject `i` at instant
    /// `labels[i]`.
    fn statistics(&self, labels: &[u16], out: &mut [f64]) {
        for (((column, exposures), &(mean, variance)), slot) in self
            .matrix
            .columns
            .iter()
            .zip(&self.exposures)
            .zip(&self.moments)
            .zip(out)
        {
            let mut count = 0_u32;
            for ((exposure, &label), &start) in
                exposures.iter().zip(labels).zip(&self.matrix.starts)
            {
                if exposure.is_some() {
                    count += u32::from(column.present.contains(start + usize::from(label)));
                }
            }
            *slot = directed(standardised(count, mean, variance), self.alternative);
        }
    }
}

fn standardised(count: u32, mean: f64, variance: f64) -> f64 {
    if variance > 0.0 {
        (f64::from(count) - mean) / variance.sqrt()
    } else {
        0.0
    }
}

const fn directed(z: f64, alternative: Alternative) -> f64 {
    match alternative {
        Alternative::Greater => z,
        Alternative::Less => -z,
        Alternative::TwoSided => z.abs(),
    }
}

impl Permuted for Matched<'_> {
    type Scratch = Vec<u16>;

    fn scratch(&self) -> Vec<u16> {
        vec![0; self.matrix.subjects()]
    }

    fn read(&self, k: u64, labels: &mut Vec<u16>, out: &mut [f64]) -> Result<(), Error> {
        self.draw(&mut SplitMix64::for_permutation(self.seed, k), labels, out);
        Ok(())
    }

    fn arrangements(&self) -> Option<u64> {
        product(self.matrix.instants.iter().map(|&count| u64::from(count)))
    }

    fn arrange(&self, rank: u64, labels: &mut Vec<u16>, out: &mut [f64]) {
        self.draw(&mut Digits(rank), labels, out);
    }
}

/// Each subject's exposure on `column`.
fn exposures(matrix: &MatchedMatrix, column: &MatchedColumn) -> Vec<Option<Exposure>> {
    matrix
        .instants
        .iter()
        .zip(&matrix.starts)
        .map(|(&count, &start)| {
            let cells = start..start + usize::from(count);
            let read = cells.clone().all(|cell| column.readable.contains(cell));
            read.then(|| Exposure {
                at_event: column.present.contains(start),
                present: u16::try_from(cells.filter(|&cell| column.present.contains(cell)).count())
                    .unwrap_or(u16::MAX),
                instants: count,
            })
        })
        .collect()
}

/// The exact p of the observed statistic: the count at the drawn
/// instants is a sum of independent Bernoulli draws, one per subject, so
/// its distribution is their convolution. Given when at most
/// [`EXACT_SUBJECTS`] subjects' exposure varies across their instants.
fn exact(
    exposures: &[Option<Exposure>],
    mean: f64,
    variance: f64,
    alternative: Alternative,
    observed: f64,
) -> Option<f64> {
    let (mut fixed, mut varying) = (0_usize, Vec::new());
    for exposure in exposures.iter().flatten() {
        if exposure.present == exposure.instants {
            fixed += 1;
        } else if exposure.present > 0 {
            varying.push(exposure.share());
        }
    }
    if varying.len() > EXACT_SUBJECTS {
        return None;
    }
    let mut distribution = vec![1.0];
    for share in varying {
        let mut next = vec![0.0; distribution.len() + 1];
        for (at, &p) in distribution.iter().enumerate() {
            if let Some(slot) = next.get_mut(at) {
                *slot += p * (1.0 - share);
            }
            if let Some(slot) = next.get_mut(at + 1) {
                *slot += p * share;
            }
        }
        distribution = next;
    }
    let fixed = u32::try_from(fixed).unwrap_or(u32::MAX);
    Some(
        (0_u32..)
            .zip(&distribution)
            .filter(|&(count, _)| {
                reaches(
                    directed(standardised(fixed + count, mean, variance), alternative),
                    observed,
                )
            })
            .map(|(_, &p)| p)
            .sum::<f64>()
            .min(1.0),
    )
}

/// One predicate's row: its count at the subjects' events against the
/// share their own instants expect, the effect with each subject a
/// stratum, and the exact p.
fn row(
    column: &MatchedColumn,
    exposures: &[Option<Exposure>],
    (mean, variance): (f64, f64),
    test: &EventTest,
    observed: f64,
    assessed: crate::Assessed,
) -> PredicateRow {
    let mut count = GroupCount::default();
    let mut tables = Vec::new();
    for exposure in exposures {
        match exposure {
            Some(exposure) => {
                let at_event = u32::from(exposure.at_event);
                count.present += at_event;
                count.absent += 1 - at_event;
                tables.push(Table {
                    case_present: at_event,
                    case_read: 1,
                    rest_present: u32::from(exposure.present) - at_event,
                    rest_read: u32::from(exposure.instants) - 1,
                });
            }
            None => count.unreadable += 1,
        }
    }
    let read = count.present + count.absent;
    let z = special::normal_quantile(1.0 - (1.0 - test.level) / 2.0);
    let (rest_present, rest_read) = tables
        .iter()
        .fold((0, 0), |(p, r), t| (p + t.rest_present, r + t.rest_read));
    // The strata are single subjects, where Breslow and Day's large-strata
    // approximation does not hold, so no homogeneity p is offered.
    let effect = Effect::of(count.present, read, rest_present, rest_read, z).map(|effect| Effect {
        pooled: Some(Pooled {
            homogeneity_p: None,
            ..Pooled::of(&tables, z)
        }),
        ..effect
    });
    let own = if read == 0 {
        0.0
    } else {
        f64::from(count.present) / f64::from(read)
    };
    let expected = if read == 0 {
        0.0
    } else {
        mean / f64::from(read)
    };
    PredicateRow {
        predicate: column.name.clone(),
        counts: vec![count],
        observed: Some(observed),
        p: assessed.p,
        exact: exact(exposures, mean, variance, test.alternative, observed),
        adjusted: assessed.adjusted,
        effect,
        expected: Some(Expectation {
            observed: own,
            expected,
            ratio: (expected > 0.0).then(|| own / expected),
        }),
        under_alpha: assessed.under_alpha,
    }
}

/// Tests whether each predicate holds at the subjects' own events more
/// (or less) often than at their own referent instants, the event's
/// place among each subject's instants drawn alone (C367). With each
/// subject a stratum of one event, the statistic is the
/// Cochran–Mantel–Haenszel statistic over subjects, which conditional
/// logistic regression's score test reduces to.
///
/// ```
/// # use teistro_research::{Alternative, Cell, EventTest, MatchedMatrix, matched};
/// #
/// # // Four subjects, each read at its event and two referents; the predicate
/// # // holds at every event and no referent. 3⁴ = 81 placements of the events.
/// # let mut matrix = MatchedMatrix::new(&[3, 3, 3, 3])?;
/// # matrix.push("at the event", |_, instant| if instant == 0 { Cell::Present } else { Cell::Absent })?;
/// # let test = EventTest { alternative: Alternative::Greater, ..EventTest::new(7, 9_999) };
///
/// // Only the observed placement puts all four events where it holds.
/// let tested = matched(&matrix, &test)?;
/// assert!(tested.enumerated);
/// assert_eq!((tested.permutations, tested.rows[0].p.value), (81, 1.0 / 81.0));
/// let expected = tested.rows[0].expected.ok_or_else(|| teistro_core::error::Error::internal("expected"))?;
/// assert!((expected.expected - 1.0 / 3.0).abs() < 1e-12);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// `INVALID_ARG` naming its field: a restricted pairing asked of a study
/// that pairs no one with another's event (`afterBirth`); permutations
/// outside 1 to [`crate::MAX_PERMUTATIONS`], or too few to reach `alpha`
/// over the family (`permutations`); `level` or `alpha` outside (0, 1);
/// no predicate (`predicates`).
pub fn matched(matrix: &MatchedMatrix, test: &EventTest) -> Result<Tested, Error> {
    if matrix.columns.is_empty() {
        return Err(Error::invalid_arg("the study names no predicate").with_field("predicates"));
    }
    if test.after_birth != AfterBirth::Refuse {
        return Err(Error::invalid_arg(
            "a self-matched study reads each subject at instants of its own life, so it pairs no one with another's event",
        )
        .with_field("afterBirth"));
    }
    crate::check_permutations(
        test.permutations,
        test.level,
        test.alpha,
        matrix.columns.len(),
    )?;
    let exposures: Vec<Vec<Option<Exposure>>> = matrix
        .columns
        .iter()
        .map(|column| exposures(matrix, column))
        .collect();
    let moments: Vec<(f64, f64)> = exposures
        .iter()
        .map(|each| {
            each.iter()
                .flatten()
                .fold((0.0, 0.0), |(mean, variance), exposure| {
                    let share = exposure.share();
                    (mean + share, variance + share * (1.0 - share))
                })
        })
        .collect();
    let paired = Matched {
        matrix,
        seed: test.seed,
        alternative: test.alternative,
        exposures,
        moments,
    };
    let mut observed = vec![0.0; matrix.columns.len()];
    paired.statistics(&vec![0; matrix.subjects()], &mut observed);
    let order = step_down_order(&observed);
    let walk = Walk::of(test.shuffle, test.permutations, paired.arrangements());
    let counted = crate::engine::count(
        &paired,
        &crate::engine::Family::new(&observed, &order),
        walk,
        test.parallelism,
    )?;
    let assessed = crate::assess(&counted, &order, walk, test.level, test.alpha);
    let rows = matrix
        .columns
        .iter()
        .zip(&paired.exposures)
        .zip(&paired.moments)
        .zip(&observed)
        .zip(assessed)
        .map(|((((column, exposures), &moments), &observed), assessed)| {
            row(column, exposures, moments, test, observed, assessed)
        })
        .collect();
    Ok(Tested::walked(rows, walk, test.shuffle))
}
