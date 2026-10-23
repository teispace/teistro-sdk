//! What rule evaluation allocates, counted rather than guessed
//! (`05-testing/01-quality-bar.md`, "allocation counts").
//!
//! The evaluator borrows the chart and builds nothing; a condition reads
//! the chart in place and gathers its bodies into a fixed array. So the
//! only memory a rule asks for is its answer's: one block for each of
//! the houses, the places it was found from and the cancellations that
//! held, when the answer has any. An absent rule allocates nothing, and
//! a bank of 597 rules over a chart costs what its present rules report.

#![allow(
    clippy::indexing_slicing,
    clippy::print_stdout,
    reason = "a test indexes the fixture it reads and reports what it measured"
)]

mod common;

use teistro_rules::{Evaluator, Readings, Rule, RuleResult};
use teistro_test_allocator::{Counting, measure};

#[global_allocator]
static ALLOCATOR: Counting = Counting::system();

/// The blocks an answer holds: one for each of its vectors that is not
/// empty.
fn blocks(result: &RuleResult) -> u64 {
    [
        result.houses.is_empty(),
        result.found_from.is_empty(),
        result.cancellations.is_empty(),
    ]
    .into_iter()
    .map(|empty| u64::from(!empty))
    .sum()
}

#[test]
fn a_rule_allocates_its_answer_and_nothing_else() {
    let rules: Vec<Rule> = common::rules()
        .into_iter()
        .filter(Rule::is_evaluable)
        .collect();
    let (_, file) = common::files().swap_remove(0);
    let chart = common::chart(&file["inputs"]);

    let (evaluator, built) = measure(|| Evaluator::new(&chart, Readings::RECORDING_ENGINE));
    assert_eq!(
        built.allocations, 0,
        "building an evaluator allocates {} times: it borrows the chart",
        built.allocations
    );

    let (mut present, mut total) = (0, 0);
    for rule in &rules {
        let (result, counts) = measure(|| evaluator.evaluate(rule));
        assert_eq!(
            counts.allocations,
            blocks(&result),
            "{} allocates {} times for an answer of {} blocks: present {}, {} houses, \
             found from {}, {} cancellations",
            rule.key,
            counts.allocations,
            blocks(&result),
            result.present,
            result.houses.len(),
            result.found_from.len(),
            result.cancellations.len()
        );
        present += u64::from(result.present);
        total += counts.allocations;
    }
    println!(
        "{} rules, {present} present: {total} allocations",
        rules.len()
    );
    assert!(
        present > 0 && present < rules.len() as u64,
        "the chart holds some rules and not others, so both halves are checked"
    );
}
