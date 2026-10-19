//! Phaladeepika ch. 16, the effects of the twelve houses, held to the 93
//! recorded charts: what each answers, and that the figures the chapter
//! narrows hold only where the wider ones do.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they read"
)]

mod common;

use std::collections::BTreeMap;

use common::{chart_at, files_in, vargas};
use teistro_rules::{Evaluator, Readings, Rule, shipped};

/// Every rule ch. 16 ships.
fn verses() -> Vec<Rule> {
    shipped::nabhasas()
        .iter()
        .filter(|rule| {
            rule.source.text == "Phaladeepika" && rule.source.chapter.as_deref() == Some("16")
        })
        .cloned()
        .collect()
}

/// Each pair is a figure and the same figure narrowed, so the second holds
/// only where the first does.
const WITHIN: [(&str, &str); 3] = [
    ("BHAVA_SUN_OR_MARS_IN_THE_TENTH", "BHAVA_SUN_OR_MARS_IN_THE_TENTH_ITS_LORD_WELL_PLACED"),
    (
        "BHAVA_LUMINARIES_IN_THE_TRINES_OF_SATURN_AND_MARS",
        "BHAVA_LUMINARIES_IN_THE_TRINES_OF_SATURN_AND_MARS_ASPECTED_BY_JUPITER",
    ),
    (
        "BHAVA_LAGNA_LORD_WELL_PLACED",
        "BHAVA_LAGNA_LORD_EXALTED_OR_OWN_IN_A_KENDRA_OR_TRIKONA_EIGHTH_LORD_AWAY_BENEFIC_RISING",
    ),
];

#[test]
fn the_twelve_houses_answer_and_their_narrowings_hold_within() {
    let rules = verses();
    assert_eq!(rules.len(), 59);
    for rule in &rules {
        assert!(rule.is_evaluable(), "{} is evaluable", rule.key);
        assert!(rule.source.verse.is_some(), "{}: cites a verse", rule.key);
    }
    teistro_rules::check_references(&rules).expect("the chapter names itself soundly");
    let at = |key: &str| {
        rules
            .iter()
            .position(|rule| rule.key == format!("PHALADEEPIKA_{key}"))
            .unwrap()
    };
    let within: Vec<(usize, usize)> = WITHIN
        .iter()
        .map(|(wide, narrow)| (at(wide), at(narrow)))
        .collect();
    let mut fired: BTreeMap<&str, usize> =
        rules.iter().map(|rule| (rule.key.as_str(), 0)).collect();
    for (path, file) in files_in("doshas") {
        let chart = chart_at(&path, &file["inputs"]);
        let divisions = vargas(&chart);
        let evaluator = Evaluator::new(&chart, Readings::TEXTS)
            .with_vargas(&divisions)
            .with_rules(&rules);
        let present: Vec<bool> = rules
            .iter()
            .map(|rule| evaluator.evaluate(rule).present)
            .collect();
        for (rule, held) in rules.iter().zip(&present) {
            *fired.get_mut(rule.key.as_str()).unwrap() += usize::from(*held);
        }
        for (wide, narrow) in &within {
            assert!(
                present[*wide] || !present[*narrow],
                "{} holds without {} in {path:?}",
                rules[*narrow].key,
                rules[*wide].key
            );
        }
    }
    let counts: Vec<(&str, usize)> = fired.into_iter().collect();
    assert_eq!(counts.as_slice(), ANSWERED.as_slice());
}

/// What each answers over the 93 recorded charts: to be filled from a run.
const ANSWERED: [(&str, usize); 0] = [];
