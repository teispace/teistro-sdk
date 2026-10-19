//! Uttara Kalamrita's yogas, Kanda I section 4 and Kanda II vv. 79 to 105,
//! held to the 93 recorded charts: what each answers, and that those the
//! book states as another text does answer what that text's do.

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

/// Every rule the book ships.
fn verses() -> Vec<Rule> {
    shipped::nabhasas()
        .iter()
        .filter(|rule| rule.source.text == "Uttara Kalamrita")
        .cloned()
        .collect()
}

/// The yogas the book states as another text does, each with that text's.
const TWINS: [(&str, &str); 2] = [
    (
        "UTTARA_RAJA_NINTH_AND_TENTH_LORDS_EXCHANGED",
        "PHALADEEPIKA_MAHARAJA_NINTH_AND_TENTH_LORDS_EXCHANGED",
    ),
    ("UTTARA_CHANDRADHI", "PARIJATA_CHANDRADHI"),
];

#[test]
fn the_yogas_answer_and_their_twins_agree() {
    let rules = verses();
    assert_eq!(rules.len(), 123);
    for rule in &rules {
        assert!(rule.is_evaluable(), "{} is evaluable", rule.key);
        assert_eq!(
            rule.source.rank.map(teistro_rules::EvidenceRank::get),
            Some(1),
            "{}: read from the text",
            rule.key
        );
        assert!(rule.source.verse.is_some(), "{}: cites a verse", rule.key);
    }
    teistro_rules::check_references(&rules).expect("the verses name themselves soundly");
    let find = |key: &str| {
        shipped::nabhasas()
            .iter()
            .find(|rule| rule.key == key)
            .cloned()
            .expect("the twin ships")
    };
    let twins: Vec<(Rule, Rule)> = TWINS
        .iter()
        .map(|&(ours, theirs)| (find(ours), find(theirs)))
        .collect();
    for (ours, theirs) in &twins {
        assert_eq!(ours.conditions, theirs.conditions, "{}", ours.key);
    }
    let mut fired: BTreeMap<&str, usize> =
        rules.iter().map(|rule| (rule.key.as_str(), 0)).collect();
    for (path, file) in files_in("doshas") {
        let chart = chart_at(&path, &file["inputs"]);
        let divisions = vargas(&chart);
        let evaluator = Evaluator::new(&chart, Readings::TEXTS)
            .with_vargas(&divisions)
            .with_rules(&rules);
        for rule in &rules {
            *fired.get_mut(rule.key.as_str()).unwrap() +=
                usize::from(evaluator.evaluate(rule).present);
        }
        for (ours, theirs) in &twins {
            assert_eq!(
                evaluator.evaluate(ours).present,
                evaluator.evaluate(theirs).present,
                "{} in {path:?}",
                ours.key
            );
        }
    }
    let counts: Vec<(&str, usize)> = fired.into_iter().collect();
    assert_eq!(counts.as_slice(), ANSWERED.as_slice());
}

/// What each answers over the 93 recorded charts. The Indu lagna's three
/// answer none here, the recorded charts carrying no points.
const ANSWERED: [(&str, usize); 0] = [];
