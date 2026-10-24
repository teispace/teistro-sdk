//! `check-properties`: the quality bar's property invariants, each named
//! with the test that holds it (`03-design/property-coverage.md` §3).
//!
//! [`HELD`] is the one list. The quality bar's "property tests" row is its
//! prose, and the gate fails an invariant the row names that no entry
//! holds, an entry whose test is gone, and an entry said to reach the
//! boundaries whose file never calls the generator it names.

use std::path::Path;

/// The quality bar, whose "property tests" row names the invariants.
const QUALITY_BAR: &str = "docs/05-testing/01-quality-bar.md";

/// How a test holds an invariant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum How {
    /// Over the whole space, which is enumerable.
    Sweep,
    /// Over values `proptest` draws.
    Sample,
    /// At a few chosen values: a test, and no hold on the invariant.
    Example,
}

/// One test holding one invariant.
#[derive(Clone, Copy, Debug)]
struct Held {
    /// The invariant, as the quality bar's row words it.
    invariant: &'static str,
    file: &'static str,
    test: &'static str,
    how: How,
    /// The generator the test reaches the boundaries with, when it does:
    /// a function the file calls.
    boundary: Option<&'static str>,
}

const fn held(invariant: &'static str, file: &'static str, test: &'static str, how: How) -> Held {
    Held {
        invariant,
        file,
        test,
        how,
        boundary: None,
    }
}

const fn at_edges(
    invariant: &'static str,
    file: &'static str,
    test: &'static str,
    how: How,
    generator: &'static str,
) -> Held {
    Held {
        invariant,
        file,
        test,
        how,
        boundary: Some(generator),
    }
}

const PARTITION: &str = "exact partition of the circle for every divisor and varga";
const SUMS: &str = "sums";
const TREE: &str = "tree consistency";
const FINDS: &str = "`dasha_at` finds every instant";
const MONOTONIC: &str = "monotonic transitions";
const FIXED: &str = "serialise-deserialise fixed points";
const CALENDAR: &str = "calendar round trips";

/// The invariants the row names, in its order.
const INVARIANTS: [&str; 7] = [PARTITION, SUMS, TREE, FINDS, MONOTONIC, FIXED, CALENDAR];

/// What holds each invariant.
const HELD: [Held; 20] = [
    held(
        PARTITION,
        "crates/core/src/angle.rs",
        "every_divisor_partitions_the_circle",
        How::Sample,
    ),
    at_edges(
        PARTITION,
        "crates/core/src/angle.rs",
        "every_classification_turns_exactly_at_its_edges",
        How::Sweep,
        "edges",
    ),
    held(
        PARTITION,
        "crates/vargas/tests/exhaustive.rs",
        "the_spans_fill_a_sign_and_the_groups_match_the_classifier",
        How::Sweep,
    ),
    at_edges(
        PARTITION,
        "crates/vargas/tests/exhaustive.rs",
        "a_part_boundary_is_decided_and_not_a_coin_toss",
        How::Sweep,
        "first_nas_of",
    ),
    held(
        SUMS,
        "crates/strength/src/ashtakavarga.rs",
        "every_chart_holds_the_classical_totals_under_either_reading",
        How::Sweep,
    ),
    held(
        SUMS,
        "crates/strength/src/shadbala/tests.rs",
        "every_graha_s_total_is_its_six_strengths_under_every_reading",
        How::Sweep,
    ),
    held(
        SUMS,
        "crates/strength/src/bhava_bala.rs",
        "every_bhava_s_total_is_its_components",
        How::Sweep,
    ),
    at_edges(
        TREE,
        "crates/dasha/tests/timeline.rs",
        "every_nakshatra_dasha_keeps_the_contract",
        How::Sweep,
        "below",
    ),
    at_edges(
        TREE,
        "crates/dasha/tests/timeline.rs",
        "every_sign_dasha_keeps_the_contract",
        How::Sweep,
        "below",
    ),
    at_edges(
        FINDS,
        "crates/dasha/tests/timeline.rs",
        "the_kalachakra_keeps_the_contract",
        How::Sweep,
        "below",
    ),
    at_edges(
        FINDS,
        "crates/dasha/tests/timeline.rs",
        "every_graha_dasha_keeps_the_contract",
        How::Sweep,
        "below",
    ),
    held(
        MONOTONIC,
        "crates/astro/src/events.rs",
        "the_suns_ingresses_and_the_tithis_are_found_in_order_at_the_boundaries",
        How::Sweep,
    ),
    held(
        MONOTONIC,
        "crates/calendar/src/mixed.rs",
        "every_day_round_trips_and_is_monotonic",
        How::Sweep,
    ),
    held(
        MONOTONIC,
        "crates/muhurta/tests/season.rs",
        "every_blackout_lies_inside_the_range_in_order",
        How::Sweep,
    ),
    held(
        FIXED,
        "crates/rules/src/rule.rs",
        "a_rule_reads_the_engine_s_shape_and_writes_it_back",
        How::Example,
    ),
    held(
        FIXED,
        "crates/core/src/envelope.rs",
        "hashes_versions_and_the_envelope_round_trip",
        How::Example,
    ),
    held(
        FIXED,
        "crates/sdk/src/varsha.rs",
        "a_request_reads_back_what_it_writes",
        How::Example,
    ),
    held(
        FIXED,
        "crates/core/src/settings/mod.rs",
        "every_shipped_profile_resolves_and_hashes_stably",
        How::Sweep,
    ),
    held(
        FIXED,
        "crates/sdk/tests/fixed_points.rs",
        "every_corpus_chart_reads_back_as_the_document_it_was",
        How::Sweep,
    ),
    held(
        CALENDAR,
        "crates/calendar/tests/range.rs",
        "every_shipped_calendar_reads_back_the_days_it_names",
        How::Sweep,
    ),
];

/// The invariants held only at examples, each with why and what closes
/// it. One here that a sweep or a sample comes to hold fails until it is
/// taken off, so the list says what is open and nothing else.
const OPEN: [(&str, &str); 0] = [];

/// Why the register and the tree disagree, as sentences.
fn faults(row: Option<&str>, read: &dyn Fn(&str) -> Option<String>) -> Vec<String> {
    let mut found = Vec::new();
    match row {
        None => found.push(format!("{QUALITY_BAR} has no \"property tests\" row")),
        Some(row) => {
            for invariant in INVARIANTS {
                if !row.contains(invariant) {
                    found.push(format!(
                        "the quality bar's property row no longer names `{invariant}`, which {} \
                         holds",
                        file!()
                    ));
                }
            }
        }
    }
    for invariant in INVARIANTS {
        if !HELD.iter().any(|h| h.invariant == invariant) {
            found.push(format!(
                "`{invariant}` is held by no test {} names",
                file!()
            ));
        }
    }
    for h in &HELD {
        let Some(text) = read(h.file) else {
            found.push(format!(
                "{}: no such file, which holds `{}`",
                h.file, h.invariant
            ));
            continue;
        };
        if !text.contains(&format!("fn {}(", h.test)) {
            found.push(format!(
                "{}: no test `{}`, which holds `{}`; rename it here or restore it",
                h.file, h.test, h.invariant
            ));
        }
        if h.how == How::Sample && !text.contains("proptest!") {
            found.push(format!(
                "{}: `{}` is said to sample, and the file has no `proptest!`",
                h.file, h.test
            ));
        }
        if let Some(generator) = h.boundary {
            if !text.contains(&format!("{generator}(")) {
                found.push(format!(
                    "{}: `{}` is said to reach the boundaries through `{generator}`, which the \
                     file never calls",
                    h.file, h.test
                ));
            }
        }
    }
    found
}

/// The quality bar's "property tests" row.
fn row(quality_bar: &str) -> Option<&str> {
    quality_bar
        .lines()
        .find(|line| line.starts_with("| property tests |"))
}

/// Holds the register to the quality bar's row and to the tree.
pub(crate) fn check(root: &Path) -> i32 {
    let quality_bar = std::fs::read_to_string(root.join(QUALITY_BAR)).unwrap_or_default();
    let read = |file: &str| std::fs::read_to_string(root.join(file)).ok();
    let found = faults(row(&quality_bar), &read);
    for line in &found {
        println!("FAIL  {line}");
    }
    if found.is_empty() {
        let boundary = HELD.iter().filter(|h| h.boundary.is_some()).count();
        println!(
            "ok    {} invariants held by {} tests, {boundary} of them at their boundaries",
            INVARIANTS.len() - OPEN.len(),
            HELD.iter().filter(|h| h.how != How::Example).count()
        );
        for (invariant, why) in OPEN {
            println!("open  `{invariant}`: {why}");
        }
        0
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_register_holds_the_tree_as_it_stands() {
        let root = crate::repo_root();
        let quality_bar = std::fs::read_to_string(root.join(QUALITY_BAR)).expect("the quality bar");
        let read = |file: &str| std::fs::read_to_string(root.join(file)).ok();
        assert_eq!(faults(row(&quality_bar), &read), Vec::<String>::new());
    }

    #[test]
    fn a_renamed_test_a_lost_generator_and_a_dropped_invariant_are_each_found() {
        let row = INVARIANTS.join(", ");
        // A file with every test but no generator and no proptest.
        let without = |_: &str| {
            Some(HELD.iter().fold(String::new(), |mut file, h| {
                file.push_str("fn ");
                file.push_str(h.test);
                file.push_str("() {}\n");
                file
            }))
        };
        let found = faults(Some(&row), &without);
        assert!(found.iter().any(|f| f.contains("never calls")), "{found:?}");
        assert!(
            found.iter().any(|f| f.contains("no `proptest!`")),
            "{found:?}"
        );
        assert!(!found.iter().any(|f| f.contains("no test")), "{found:?}");

        let empty = |_: &str| Some(String::new());
        let found = faults(Some(&row), &empty);
        assert_eq!(
            found.iter().filter(|f| f.contains("no test")).count(),
            HELD.len()
        );

        let short = INVARIANTS[1..].join(", ");
        let found = faults(Some(&short), &without);
        assert!(found.iter().any(|f| f.contains(INVARIANTS[0])), "{found:?}");
        assert!(
            faults(None, &without)
                .iter()
                .any(|f| f.contains("no \"property tests\" row"))
        );
    }

    #[test]
    fn an_invariant_at_examples_alone_is_open_and_says_so() {
        let open: Vec<&str> = OPEN.iter().map(|(name, _)| *name).collect();
        for invariant in INVARIANTS {
            let holds = HELD
                .iter()
                .any(|h| h.invariant == invariant && h.how != How::Example);
            assert_eq!(!holds, open.contains(&invariant), "{invariant}");
        }
    }
}
