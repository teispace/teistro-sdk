//! BPHS ch. 83's curses in a former birth that deny a son, held to the 93
//! recorded charts: what each figure answers, and that every one says whose
//! curse it reads and the remedy its chapter gives.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they read"
)]

mod common;

use std::collections::BTreeMap;

use common::{chart_at, files_in, vargas};
use teistro_rules::{Evaluator, Readings, Rule, shipped};

#[test]
fn each_curse_answers_what_it_answers_over_the_corpus() {
    let rules: Vec<Rule> = shipped::nabhasas()
        .iter()
        .filter(|rule| rule.category == "shapa")
        .cloned()
        .collect();
    assert_eq!(rules.len(), 76);
    let mut whose: BTreeMap<&str, usize> = BTreeMap::new();
    for rule in &rules {
        assert_eq!(rule.source.chapter.as_deref(), Some("83"), "{}", rule.key);
        let note = rule.source.note.as_deref().unwrap();
        if let Some(curse) = rule
            .key
            .strip_prefix("BPHS_")
            .and_then(|key| key.split_once("_CURSE_"))
        {
            *whose.entry(curse.0).or_default() += 1;
            // A figure carries its chapter's remedy, which the verse after
            // the figures gives once for all of them.
            assert!(note.contains("The remedy it gives, vv. "), "{}", rule.key);
        }
    }
    assert_eq!(
        whose.into_iter().collect::<Vec<_>>(),
        [
            ("BRAHMIN", 7),
            ("BROTHER", 10),
            ("FATHER", 11),
            ("MATERNAL_UNCLE", 4),
            ("MOTHER", 13),
            ("SERPENT", 8),
            ("SPIRIT", 10),
            ("WIFE", 11),
        ]
    );

    let mut fired: BTreeMap<&str, usize> =
        rules.iter().map(|rule| (rule.key.as_str(), 0)).collect();
    let mut charts = 0;
    for (path, file) in files_in("doshas") {
        let chart = chart_at(&path, &file["inputs"]);
        let divisions = vargas(&chart);
        let evaluator = Evaluator::new(&chart, Readings::TEXTS).with_vargas(&divisions);
        for rule in &rules {
            if evaluator.evaluate(rule).present {
                *fired.get_mut(rule.key.as_str()).unwrap() += 1;
            }
        }
        charts += 1;
    }
    assert_eq!(charts, 93);
    // So many stacked placings rarely meet: the rest hold on none of the 93,
    // and the cofiring pass finds each a sky of its own (`rule-cofiring-measured.md`).
    let (held, silent): (Vec<_>, Vec<_>) = fired.into_iter().partition(|(_, n)| *n > 0);
    assert_eq!(held.as_slice(), ANSWERS.as_slice());
    assert_eq!(silent.len(), 73);
}

/// The figures that hold on at least one of the 93 charts, and on how many.
const ANSWERS: [(&str, usize); 3] = [
    ("BPHS_MOTHER_CURSE_3", 8),
    ("BPHS_NO_SON_SIGNIFICATORS_WITHOUT_STRENGTH", 3),
    ("BPHS_WIFE_CURSE_4", 1),
];
