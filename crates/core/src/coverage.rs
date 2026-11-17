//! The instants an ephemeris answers, and the mark a span carries when
//! they cut it (SR-136, `03-design/coverage-edges.md`).
//!
//! A search that reads around a date reads past the coverage near its
//! ends: the tithi running at the first sunrise began before the first
//! instant any engine was asked for, and a year began months before. The
//! SDK clips every such search to the coverage once, where the search
//! is made, and a span it cut says so with [`Clipped`] rather than being
//! refused or given an instant nobody computed (C2242).
//!
//! ```
//! use teistro_core::coverage::{Clipped, Coverage};
//!
//! // 1800-01-01 to 2400-01-01, midnight to midnight.
//! let coverage = Coverage::new(2_378_496.5, 2_597_641.5)?;
//! // A tithi read from three days before the first midnight is cut there.
//! let (from, to, clipped) = coverage.clip(2_378_493.5, 2_378_497.2).ok_or("it meets the coverage")?;
//! assert_eq!((from, to, clipped), (2_378_496.5, 2_378_497.2, Some(Clipped::Start)));
//! assert_eq!(coverage.clip(2_400_000.0, 2_400_001.0).map(|cut| cut.2), Some(None));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use serde::{Deserialize, Serialize};

use crate::error::Error;

/// Which ends of a span the coverage cut: the member began before the
/// first instant the ephemeris answers (`START`), outlasts the last
/// (`END`), or both. The cut end is the coverage's edge and **not** the
/// member's own; the member itself is the one the covered part names.
///
/// ```
/// use teistro_core::coverage::Clipped;
///
/// assert_eq!(Clipped::of(true, false), Some(Clipped::Start));
/// assert_eq!(Clipped::of(false, false), None);
/// assert_eq!(Clipped::Start.with(Some(Clipped::End)), Clipped::Both);
/// assert!(Clipped::Both.start() && Clipped::Both.end());
/// assert_eq!(serde_json::to_string(&Clipped::End)?, r#""END""#);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Clipped {
    /// It began before the coverage: its start is the coverage's first
    /// instant.
    Start,
    /// It outlasts the coverage: its end is the coverage's last instant.
    End,
    /// Both.
    Both,
}

impl Clipped {
    /// The mark for a span cut at its start, its end, both or neither.
    #[must_use]
    pub const fn of(start: bool, end: bool) -> Option<Clipped> {
        match (start, end) {
            (true, true) => Some(Clipped::Both),
            (true, false) => Some(Clipped::Start),
            (false, true) => Some(Clipped::End),
            (false, false) => None,
        }
    }

    /// Whether the start was cut.
    #[must_use]
    pub const fn start(self) -> bool {
        matches!(self, Clipped::Start | Clipped::Both)
    }

    /// Whether the end was cut.
    #[must_use]
    pub const fn end(self) -> bool {
        matches!(self, Clipped::End | Clipped::Both)
    }

    /// This mark joined with another: a span cut at either end by either.
    #[must_use]
    pub const fn with(self, other: Option<Clipped>) -> Clipped {
        match other {
            None => self,
            Some(other) => {
                match Clipped::of(self.start() || other.start(), self.end() || other.end()) {
                    Some(joined) => joined,
                    None => self,
                }
            }
        }
    }

    /// Two optional marks joined.
    ///
    /// ```
    /// use teistro_core::coverage::Clipped;
    ///
    /// assert_eq!(Clipped::join(None, Some(Clipped::End)), Some(Clipped::End));
    /// assert_eq!(Clipped::join(None, None), None);
    /// ```
    #[must_use]
    pub const fn join(a: Option<Clipped>, b: Option<Clipped>) -> Option<Clipped> {
        match a {
            Some(a) => Some(a.with(b)),
            None => b,
        }
    }
}

/// The instants an ephemeris answers, Julian days on UT1 (the scale every
/// search of the SDK runs on), inclusive at both ends.
///
/// ```
/// use teistro_core::coverage::Coverage;
///
/// assert!(Coverage::UNBOUNDED.contains(1.0e9) && !Coverage::UNBOUNDED.is_bounded());
/// let coverage = Coverage::new(10.0, 20.0)?;
/// assert!(coverage.contains(10.0) && coverage.contains(20.0) && !coverage.contains(20.5));
/// assert!(Coverage::new(20.0, 10.0).is_err());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coverage {
    /// The first instant answered.
    pub from: f64,
    /// The last instant answered.
    pub to: f64,
}

impl Coverage {
    /// Every instant: what a source that states no coverage answers, so a
    /// search over it is never cut.
    pub const UNBOUNDED: Coverage = Coverage {
        from: f64::NEG_INFINITY,
        to: f64::INFINITY,
    };

    /// A coverage from its two ends.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on `coverage` for an end that is not a number or a
    /// span that runs backwards.
    pub fn new(from: f64, to: f64) -> Result<Coverage, Error> {
        if from.is_nan() || to.is_nan() || from > to {
            return Err(Error::invalid_arg(format!(
                "a coverage runs forward, not from {from} to {to}"
            ))
            .with_field("coverage"));
        }
        Ok(Coverage { from, to })
    }

    /// Whether an instant is answered.
    #[must_use]
    pub fn contains(self, jd: f64) -> bool {
        jd >= self.from && jd <= self.to
    }

    /// Whether either end is finite.
    #[must_use]
    pub fn is_bounded(self) -> bool {
        self.from.is_finite() || self.to.is_finite()
    }

    /// The instants both answer.
    ///
    /// ```
    /// use teistro_core::coverage::Coverage;
    ///
    /// let both = Coverage::new(0.0, 10.0)?.intersect(Coverage::new(5.0, 20.0)?);
    /// assert_eq!((both.from, both.to), (5.0, 10.0));
    /// assert_eq!(Coverage::UNBOUNDED.intersect(both), both);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn intersect(self, other: Coverage) -> Coverage {
        let from = self.from.max(other.from);
        Coverage {
            from,
            to: self.to.min(other.to).max(from),
        }
    }

    /// A window cut to the coverage, with the mark saying which ends were
    /// cut; `None` when the window and the coverage do not meet.
    ///
    /// ```
    /// use teistro_core::coverage::{Clipped, Coverage};
    ///
    /// let coverage = Coverage::new(10.0, 20.0)?;
    /// assert_eq!(coverage.clip(12.0, 15.0), Some((12.0, 15.0, None)));
    /// assert_eq!(coverage.clip(5.0, 25.0), Some((10.0, 20.0, Some(Clipped::Both))));
    /// assert_eq!(coverage.clip(21.0, 25.0), None);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn clip(self, from: f64, to: f64) -> Option<(f64, f64, Option<Clipped>)> {
        let (start, end) = (from.max(self.from), to.min(self.to));
        (start <= end).then(|| (start, end, Clipped::of(from < self.from, to > self.to)))
    }
}
