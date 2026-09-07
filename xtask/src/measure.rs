//! What a falsification pass shares: a claim, a verdict, and a page that
//! stays readable however wide the numbers turn out to be.
//!
//! The project's working pattern is to propose a rule and measure it
//! against the conformance corpus **before** the module that would rely
//! on it is written (`docs/STATUS.md`, "How to resume"). Two passes do
//! that so far — [`crate::panchanga`] over the recorded daily panchanga
//! and [`crate::vargas`] over the recorded divisional charts — and each
//! writes a generated page a gate holds to the build. This module is what
//! they have in common, so that there is one claim table and not two.

use std::fmt::Write as _;

/// What the corpus said about a proposed rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Verdict {
    /// The rule reproduces every row the corpus records.
    Holds,
    /// The corpus contradicts it.
    Falsified,
    /// The corpus holds no row that would tell the difference.
    Untested,
}

impl Verdict {
    /// How a page marks it.
    pub(crate) const fn mark(self) -> &'static str {
        match self {
            Verdict::Holds => "**holds**",
            Verdict::Falsified => "falsified",
            Verdict::Untested => "untested",
        }
    }
}

/// A verdict from whether a rule survived.
pub(crate) const fn verdict_of(ok: bool) -> Verdict {
    if ok {
        Verdict::Holds
    } else {
        Verdict::Falsified
    }
}

/// One proposed rule and the measurement that decided it.
pub(crate) struct Claim {
    pub(crate) rule: String,
    pub(crate) verdict: Verdict,
    pub(crate) measured: String,
}

impl Claim {
    /// A claim decided by whether a worst-case error in seconds is inside
    /// a stated bound.
    pub(crate) fn within(rule: impl Into<String>, worst_seconds: f64, bound: f64) -> Claim {
        Claim {
            rule: rule.into(),
            verdict: verdict_of(worst_seconds.is_finite() && worst_seconds <= bound),
            measured: format!("worst {}", seconds(worst_seconds)),
        }
    }

    /// A claim decided by a count of comparisons that contradict it.
    ///
    /// Nothing to compare is `untested` and not a pass: a rule no row
    /// exercises has not been tested, and saying so is the point of the
    /// verdict.
    pub(crate) fn counted(rule: impl Into<String>, wrong: usize, of: usize) -> Claim {
        Claim {
            rule: rule.into(),
            verdict: if of == 0 {
                Verdict::Untested
            } else {
                verdict_of(wrong == 0)
            },
            measured: if of == 0 {
                String::from("nothing tests it")
            } else {
                format!("{wrong} of {of} disagree")
            },
        }
    }

    /// A claim whose measurement is stated rather than counted.
    pub(crate) fn stated(
        rule: impl Into<String>,
        verdict: Verdict,
        measured: impl Into<String>,
    ) -> Claim {
        Claim {
            rule: rule.into(),
            verdict,
            measured: measured.into(),
        }
    }
}

/// The claims as a table.
pub(crate) fn table(claims: &[Claim]) -> String {
    let mut out = String::from("| proposed rule | verdict | measured |\n|---|---|---|\n");
    for claim in claims {
        let _ = writeln!(
            out,
            "| {} | {} | {} |",
            claim.rule,
            claim.verdict.mark(),
            claim.measured
        );
    }
    out
}

/// A duration in seconds, written at the scale it is.
pub(crate) fn seconds(value: f64) -> String {
    if !value.is_finite() {
        String::from("not a number")
    } else if value == 0.0 {
        String::from("0 s, exactly")
    } else if value < 0.001 {
        format!("{:.3} ms", value * 1000.0)
    } else if value < 120.0 {
        format!("{value:.3} s")
    } else {
        format!("{:.2} h", value / 3600.0)
    }
}

/// A count, with a space every three digits from five digits up: the
/// house style for a number a reader has to take in at a glance.
pub(crate) fn count(value: usize) -> String {
    let digits = value.to_string();
    if digits.len() < 5 {
        return digits;
    }
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            out.push(' ');
        }
        out.push(digit);
    }
    out
}

/// The greatest of a set of measurements.
pub(crate) fn worst(values: impl IntoIterator<Item = f64>) -> f64 {
    values.into_iter().fold(0.0_f64, f64::max)
}

/// The width a generated page's prose is filled to.
pub(crate) const FILL: usize = 72;

/// Refills a generated page's prose paragraphs.
///
/// Every measurement on such a page is substituted into a sentence, and a
/// number that is two digits wide today may be three tomorrow, so prose
/// wrapped in the source drifts ragged as the corpus grows. Filling the
/// finished page instead keeps it tidy whatever the numbers turn out to
/// be. A block that is a heading, a table or a list is left exactly as it
/// was written.
pub(crate) fn fill(page: &str) -> String {
    page.split("\n\n")
        .map(|block| {
            let prose = block.lines().all(|line| {
                let line = line.trim_start();
                !line.starts_with('|')
                    && !line.starts_with('#')
                    && !line.starts_with("- ")
                    && !line.starts_with(|c: char| c.is_ascii_digit())
            });
            if prose {
                wrapped(
                    &block
                        .split_whitespace()
                        .map(str::to_string)
                        .collect::<Vec<_>>(),
                    FILL,
                    " ",
                )
            } else {
                block.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// A list of items, wrapped so that generated prose stays readable.
pub(crate) fn wrapped(items: &[String], width: usize, separator: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();
    for item in items {
        if !line.is_empty() && line.len() + separator.len() + item.len() > width {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push_str(separator);
        }
        line.push_str(item);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::{Claim, Verdict, count, fill, seconds, table, worst, wrapped};

    #[test]
    fn a_verdict_says_what_it_is() {
        assert_eq!(Verdict::Holds.mark(), "**holds**");
        assert_eq!(Verdict::Falsified.mark(), "falsified");
        assert_eq!(Verdict::Untested.mark(), "untested");
    }

    #[test]
    fn nothing_to_compare_is_untested_and_not_a_pass() {
        let none = Claim::counted("a rule no row exercises", 0, 0);
        assert_eq!(none.verdict, Verdict::Untested);
        assert_eq!(none.measured, "nothing tests it");
        assert_eq!(Claim::counted("a rule", 0, 5).verdict, Verdict::Holds);
        assert_eq!(Claim::counted("a rule", 1, 5).verdict, Verdict::Falsified);
    }

    #[test]
    fn a_bound_decides_a_measured_claim() {
        assert_eq!(Claim::within("a", 0.5, 1.0).verdict, Verdict::Holds);
        assert_eq!(Claim::within("a", 1.5, 1.0).verdict, Verdict::Falsified);
        assert_eq!(
            Claim::within("a", f64::NAN, 1.0).verdict,
            Verdict::Falsified
        );
    }

    #[test]
    fn a_duration_is_written_at_the_scale_it_is() {
        assert_eq!(seconds(0.0), "0 s, exactly");
        assert_eq!(seconds(0.000_5), "0.500 ms");
        assert_eq!(seconds(12.5), "12.500 s");
        assert_eq!(seconds(7200.0), "2.00 h");
        assert_eq!(seconds(f64::INFINITY), "not a number");
    }

    #[test]
    fn a_table_has_a_row_for_every_claim() {
        let rendered = table(&[
            Claim::counted("one", 0, 3),
            Claim::stated("two", Verdict::Untested, "nothing"),
        ]);
        assert_eq!(rendered.lines().count(), 4, "a head, a rule and two rows");
        assert!(rendered.contains("| one | **holds** | 0 of 3 disagree |"));
        assert!(rendered.contains("| two | untested | nothing |"));
    }

    #[test]
    fn a_large_count_is_spaced_and_a_small_one_is_not() {
        assert_eq!(count(0), "0");
        assert_eq!(count(930), "930");
        assert_eq!(count(9999), "9999", "four digits stay together");
        assert_eq!(count(19_530), "19 530");
        assert_eq!(count(1_234_567), "1 234 567");
    }

    #[test]
    fn the_greatest_of_nothing_is_nothing() {
        assert!(worst([]).abs() < f64::EPSILON);
        assert!((worst([1.0, 3.0, 2.0]) - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn wrapping_breaks_before_the_width_and_not_after() {
        let items = ["alpha", "beta", "gamma"].map(str::to_string);
        assert_eq!(wrapped(&items, 40, ", "), "alpha, beta, gamma");
        assert_eq!(wrapped(&items, 12, ", "), "alpha, beta\ngamma");
        assert_eq!(wrapped(&[], 12, ", "), "");
    }

    #[test]
    fn filling_leaves_tables_and_lists_alone() {
        let page = "# A heading\n\nsome prose that is\nwrapped oddly\n\n\
                    | a | table |\n|---|---|\n| and | a row |\n\n\
                    1. a list item\n   and its continuation\n";
        let filled = fill(page);
        assert!(
            filled.contains("some prose that is wrapped oddly"),
            "{filled}"
        );
        assert!(filled.contains("| a | table |\n|---|---|"), "{filled}");
        assert!(
            filled.contains("1. a list item\n   and its continuation"),
            "{filled}"
        );
    }
}
