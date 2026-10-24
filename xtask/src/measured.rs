//! The results schema every measured page shares (`05-testing/01-quality-
//! bar.md`, "results schema"; `03-design/ffi-cost.md` §3).
//!
//! A timed figure is a claim only with its runs, its median, and its fastest
//! and slowest run, and only when it was taken from enough runs and its
//! median lies inside its own range. A comparison between two figures is a
//! number only where their ranges do not overlap. Where they overlap,
//! neither is measurably the larger, whatever their medians say.

/// The fewest runs a median is taken from.
pub(crate) const MIN_RUNS: usize = 3;

/// One timed figure, in whatever unit its page states.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Figure {
    /// The runs the median was taken from.
    pub(crate) runs: usize,
    /// The median run.
    pub(crate) median: f64,
    /// The fastest run.
    pub(crate) min: f64,
    /// The slowest run.
    pub(crate) max: f64,
}

impl Figure {
    /// Why the figure is no claim, or `None` when it is one: too few runs,
    /// a median outside its own range, or a value that is not finite.
    pub(crate) fn fault(&self) -> Option<String> {
        let finite = [self.median, self.min, self.max]
            .iter()
            .all(|value| value.is_finite());
        let held = finite && self.min <= self.median && self.median <= self.max;
        (self.runs < MIN_RUNS || !held).then(|| {
            format!(
                "{} runs from {} to {} around {}: a median needs {MIN_RUNS} runs and a spread \
                 that holds it",
                self.runs, self.min, self.max, self.median
            )
        })
    }

    /// Whether the two figures' fastest-to-slowest ranges meet.
    pub(crate) fn overlaps(&self, other: &Figure) -> bool {
        self.min <= other.max && other.min <= self.max
    }
}

/// What a comparison of two figures says.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Comparison {
    /// The ranges overlap: neither is measurably the larger.
    WithinSpread,
    /// The ranges are apart, and this is the measured value.
    Measured(f64),
    /// The comparison has no value, such as a ratio over a zero median.
    Undefined,
}

impl Comparison {
    /// The comparison as a page prints it, its value written by `format`.
    pub(crate) fn render(self, format: impl Fn(f64) -> String) -> String {
        match self {
            Comparison::WithinSpread => String::from("within spread"),
            Comparison::Measured(value) => format(value),
            Comparison::Undefined => String::from("—"),
        }
    }
}

/// `left ÷ right` between medians, where the ranges are apart.
pub(crate) fn ratio(left: &Figure, right: &Figure) -> Comparison {
    if left.overlaps(right) {
        Comparison::WithinSpread
    } else if right.median > 0.0 {
        Comparison::Measured(left.median / right.median)
    } else {
        Comparison::Undefined
    }
}

/// `left − right` between medians, where the ranges are apart.
pub(crate) fn difference(left: &Figure, right: &Figure) -> Comparison {
    if left.overlaps(right) {
        Comparison::WithinSpread
    } else {
        Comparison::Measured(left.median - right.median)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn figure(runs: usize, median: f64, min: f64, max: f64) -> Figure {
        Figure {
            runs,
            median,
            min,
            max,
        }
    }

    #[test]
    fn a_figure_needs_its_runs_and_a_spread_that_holds_its_median() {
        assert_eq!(figure(3, 1.0, 0.5, 2.0).fault(), None);
        assert_eq!(
            figure(MIN_RUNS, 1.0, 1.0, 1.0).fault(),
            None,
            "a range of one point holds it"
        );
        assert!(figure(2, 1.0, 0.5, 2.0).fault().is_some(), "too few runs");
        assert!(
            figure(3, 1.0, 1.5, 2.0).fault().is_some(),
            "below its range"
        );
        assert!(
            figure(3, 3.0, 1.5, 2.0).fault().is_some(),
            "above its range"
        );
        assert!(
            figure(3, f64::NAN, 0.0, 1.0).fault().is_some(),
            "not a number"
        );
        assert!(
            figure(3, 1.0, 0.0, f64::INFINITY).fault().is_some(),
            "unbounded"
        );
    }

    #[test]
    fn two_figures_compare_only_where_their_ranges_are_apart() {
        let slow = figure(3, 10.0, 9.0, 11.0);
        let fast = figure(3, 2.0, 1.0, 3.0);
        let near = figure(3, 8.0, 7.0, 9.0);
        assert_eq!(ratio(&slow, &fast), Comparison::Measured(5.0));
        assert_eq!(difference(&slow, &fast), Comparison::Measured(8.0));
        assert_eq!(
            ratio(&slow, &near),
            Comparison::WithinSpread,
            "they meet at 9"
        );
        assert_eq!(difference(&near, &slow), Comparison::WithinSpread);
        let zero = figure(3, 0.0, 0.0, 0.0);
        assert_eq!(ratio(&slow, &zero), Comparison::Undefined);
        assert_eq!(
            Comparison::WithinSpread.render(|v| v.to_string()),
            "within spread"
        );
        assert_eq!(
            Comparison::Measured(1.5).render(|v| format!("{v:.1}×")),
            "1.5×"
        );
        assert_eq!(Comparison::Undefined.render(|v| v.to_string()), "—");
    }
}
