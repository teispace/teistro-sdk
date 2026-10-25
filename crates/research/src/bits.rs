//! The chart-by-predicate matrix, one bitset per predicate's column
//! (`research.md` §2.3).
//!
//! The charts are founded and every predicate evaluated once; a
//! permutation then costs one case mask and a popcount per predicate,
//! so every predicate's statistic for one permutation comes together,
//! which is what max-T needs.

use std::ops::Range;

use serde::{Deserialize, Serialize};
use teistro_core::error::Error;

/// A set of chart indices, `0..len`, packed 64 to a word.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Bits {
    words: Vec<u64>,
}

impl Bits {
    /// The empty set over `len` charts.
    pub(crate) fn empty(len: usize) -> Bits {
        Bits {
            words: vec![0; len.div_ceil(64)],
        }
    }

    /// Adds `index`, which is below the set's length.
    pub(crate) fn insert(&mut self, index: usize) {
        if let Some(word) = self.words.get_mut(index / 64) {
            *word |= 1 << (index % 64);
        }
    }

    /// Takes `index` out.
    pub(crate) fn remove(&mut self, index: usize) {
        if let Some(word) = self.words.get_mut(index / 64) {
            *word &= !(1 << (index % 64));
        }
    }

    pub(crate) fn contains(&self, index: usize) -> bool {
        self.words
            .get(index / 64)
            .is_some_and(|word| word >> (index % 64) & 1 == 1)
    }

    pub(crate) fn clear(&mut self) {
        self.words.fill(0);
    }

    /// Keeps only the members `other` also has.
    pub(crate) fn intersect(&mut self, other: &Bits) {
        for (a, b) in self.words.iter_mut().zip(&other.words) {
            *a &= b;
        }
    }

    /// How many members the set has.
    pub(crate) fn count(&self) -> u32 {
        self.words.iter().map(|word| word.count_ones()).sum()
    }

    /// How many members this set shares with `other`.
    pub(crate) fn common(&self, other: &Bits) -> u32 {
        self.words
            .iter()
            .zip(&other.words)
            .map(|(a, b)| (a & b).count_ones())
            .sum()
    }

    /// How many members this set shares with `other` at indices inside
    /// `range`: a stratum laid out contiguously costs its own words, not
    /// the whole batch's.
    pub(crate) fn common_in(&self, other: &Bits, range: &Range<usize>) -> u32 {
        if range.is_empty() {
            return 0;
        }
        let (first, last) = (range.start / 64, (range.end - 1) / 64);
        let mut total = 0;
        for at in first..=last {
            let (Some(a), Some(b)) = (self.words.get(at), other.words.get(at)) else {
                break;
            };
            let mut word = a & b;
            if at == first {
                word &= u64::MAX << (range.start % 64);
            }
            if at == last {
                let kept = range.end - last * 64;
                if kept < 64 {
                    word &= (1 << kept) - 1;
                }
            }
            total += word.count_ones();
        }
        total
    }

    /// The set with its indices renumbered: index `i` of the answer is
    /// index `order[i]` of this one.
    fn reordered(&self, order: &[usize]) -> Bits {
        let mut out = Bits::empty(order.len());
        for (to, &from) in order.iter().enumerate() {
            if self.contains(from) {
                out.insert(to);
            }
        }
        out
    }
}

/// What one predicate said of one chart.
///
/// ```
/// use teistro_research::{Cell, Matrix};
///
/// // An unreadable chart leaves the denominator; it is not absent.
/// let mut matrix = Matrix::new(3);
/// matrix.push("lagna", &[Cell::Present, Cell::Unreadable, Cell::Unstable])?;
/// let tally = matrix.tallies()[0];
/// assert_eq!((tally.present, tally.absent, tally.unreadable, tally.unstable), (1, 0, 1, 1));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Cell {
    /// The predicate holds.
    Present,
    /// The predicate was read and does not hold.
    Absent,
    /// The predicate cannot be read on this chart (a special lagna on a
    /// polar day). The chart leaves the predicate's denominator and is
    /// never counted as absent (`research.md` §1.7).
    Unreadable,
    /// The predicate gives different answers inside the chart's recorded
    /// time uncertainty. Counted apart and, like an unreadable chart,
    /// left out of the denominator rather than decided.
    Unstable,
}

/// One predicate's cells over the batch.
#[derive(Clone, Debug)]
pub(crate) struct Column {
    /// The predicate's name, which its answer row carries.
    pub(crate) name: String,
    pub(crate) present: Bits,
    pub(crate) readable: Bits,
    pub(crate) unstable: Bits,
    /// How many charts the predicate holds on, and how many it was read
    /// on: the margins every permutation keeps.
    pub(crate) present_total: u32,
    pub(crate) readable_total: u32,
}

/// The batch's charts by its predicates: the expensive half of a study,
/// done once before any permutation.
///
/// ```
/// use teistro_research::{Cell, Matrix};
///
/// let mut matrix = Matrix::new(3);
/// matrix.push("yoga.gaja-kesari", &[Cell::Present, Cell::Absent, Cell::Unreadable])?;
/// assert_eq!(matrix.predicates(), 1);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug)]
pub struct Matrix {
    charts: usize,
    pub(crate) columns: Vec<Column>,
}

impl Matrix {
    /// The same matrix with its charts renumbered: chart `i` of the answer
    /// is chart `order[i]` of this one. Counts are unchanged; a stratified
    /// test lays each stratum out contiguously this way.
    pub(crate) fn reordered(&self, order: &[usize]) -> Matrix {
        Matrix {
            charts: order.len(),
            columns: self
                .columns
                .iter()
                .map(|column| Column {
                    name: column.name.clone(),
                    present: column.present.reordered(order),
                    readable: column.readable.reordered(order),
                    unstable: column.unstable.reordered(order),
                    present_total: column.present_total,
                    readable_total: column.readable_total,
                })
                .collect(),
        }
    }

    /// A matrix over `charts` charts with no predicate yet.
    ///
    /// ```
    /// use teistro_research::Matrix;
    ///
    /// let matrix = Matrix::new(12);
    /// assert_eq!((matrix.charts(), matrix.predicates()), (12, 0));
    /// ```
    #[must_use]
    pub fn new(charts: usize) -> Matrix {
        Matrix {
            charts,
            columns: Vec::new(),
        }
    }

    /// The batch's size.
    ///
    /// ```
    /// use teistro_research::Matrix;
    ///
    /// assert_eq!(Matrix::new(3).charts(), 3);
    /// ```
    #[must_use]
    pub const fn charts(&self) -> usize {
        self.charts
    }

    /// How many predicates have been added.
    ///
    /// ```
    /// use teistro_research::{Cell, Matrix};
    ///
    /// let mut matrix = Matrix::new(2);
    /// matrix.push("one", &[Cell::Present, Cell::Absent])?;
    /// matrix.push("two", &[Cell::Absent, Cell::Absent])?;
    /// assert_eq!(matrix.predicates(), 2);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn predicates(&self) -> usize {
        self.columns.len()
    }

    /// Adds one predicate's column under `name`, a cell per chart in the
    /// batch's order.
    ///
    /// ```
    /// use teistro_research::{Cell, Matrix};
    ///
    /// let mut matrix = Matrix::new(2);
    /// matrix.push("one", &[Cell::Present, Cell::Absent])?;
    /// // A name given twice, and a column not one cell per chart, are refused.
    /// assert_eq!(matrix.push("one", &[Cell::Absent, Cell::Absent]).unwrap_err().field(), Some("predicates[1]"));
    /// assert!(matrix.push("two", &[Cell::Absent]).is_err());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on `predicates[i]` when the column is not one cell
    /// per chart, and when its name is one an earlier predicate has.
    pub fn push(&mut self, name: impl Into<String>, cells: &[Cell]) -> Result<(), Error> {
        let name = name.into();
        if self.columns.iter().any(|column| column.name == name) {
            return Err(Error::invalid_arg(format!(
                "predicate `{name}` is named twice; a family names each predicate once"
            ))
            .with_field(format!("predicates[{}]", self.columns.len())));
        }
        if cells.len() != self.charts {
            return Err(Error::invalid_arg(format!(
                "predicate {} has {} cells for {} charts; a column carries one cell per chart",
                self.columns.len(),
                cells.len(),
                self.charts
            ))
            .with_field(format!("predicates[{}]", self.columns.len())));
        }
        let mut present = Bits::empty(self.charts);
        let mut readable = Bits::empty(self.charts);
        let mut unstable = Bits::empty(self.charts);
        for (index, cell) in cells.iter().enumerate() {
            match cell {
                Cell::Present => {
                    present.insert(index);
                    readable.insert(index);
                }
                Cell::Absent => readable.insert(index),
                Cell::Unreadable => {}
                Cell::Unstable => unstable.insert(index),
            }
        }
        self.columns.push(Column {
            name,
            present_total: present.count(),
            readable_total: readable.count(),
            present,
            readable,
            unstable,
        });
        Ok(())
    }
}
