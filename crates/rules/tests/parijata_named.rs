//! Jataka Parijata ch. 7's named yogas held to the 93 recorded charts: what
//! each answers, and that the four the chapter states as BPHS ch. 36 does
//! answer what BPHS's do.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they read"
)]

mod common;

use std::collections::BTreeMap;

use common::{chart_at, files_in};
use teistro_rules::{Evaluator, Readings, Rule, shipped};

/// Every rule the verses ship: ch. 7 from v. 128 on.
fn verses() -> Vec<Rule> {
    shipped::nabhasas()
        .iter()
        .filter(|rule| {
            rule.source.text == "Jataka Parijata"
                && rule.source.chapter.as_deref() == Some("7")
                && first_verse(rule) >= 128
        })
        .cloned()
        .collect()
}

/// The first verse a rule cites.
fn first_verse(rule: &Rule) -> u32 {
    let verse = rule.source.verse.as_deref().unwrap_or_default();
    let digits: String = verse.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().expect("a verse number")
}

/// The yogas both texts state alike.
const TWINS: [&str; 4] = ["CHAMARA", "SANKHA", "SRINATHA", "KHADGA"];

#[test]
fn the_named_yogas_answer_and_their_twins_agree() {
    let rules = verses();
    assert_eq!(rules.len(), 31);
    teistro_rules::check_references(&rules).expect("the verses name themselves soundly");
    let twins: Vec<Rule> = TWINS
        .iter()
        .map(|name| {
            let key = format!("PARASHARA_{name}");
            shipped::nabhasas()
                .iter()
                .find(|rule| rule.key == key)
                .cloned()
                .expect("BPHS ships the twin")
        })
        .collect();
    let mut fired: BTreeMap<&str, usize> =
        rules.iter().map(|rule| (rule.key.as_str(), 0)).collect();
    for (path, file) in files_in("doshas") {
        let chart = chart_at(&path, &file["inputs"]);
        let evaluator = Evaluator::new(&chart, Readings::TEXTS).with_rules(&rules);
        for rule in &rules {
            *fired.get_mut(rule.key.as_str()).unwrap() +=
                usize::from(evaluator.evaluate(rule).present);
        }
        for (name, twin) in TWINS.iter().zip(&twins) {
            let ours = rules
                .iter()
                .find(|rule| rule.key == format!("PARIJATA_{name}"))
                .unwrap();
            assert_eq!(
                evaluator.evaluate(ours).present,
                evaluator.evaluate(twin).present,
                "{name} in {path:?}"
            );
        }
    }
    let counts: Vec<(&str, usize)> = fired.into_iter().collect();
    assert_eq!(counts.as_slice(), ANSWERED.as_slice());
}

/// What each answers over the 93 recorded charts.
const ANSWERED: [(&str, usize); 31] = [
    ("PARIJATA_AMSAVATARA", 0),
    ("PARIJATA_BHERI", 9),
    ("PARIJATA_CHAMARA", 17),
    ("PARIJATA_HARIHARABRAHMA_FROM_THE_LAGNA_LORD", 0),
    ("PARIJATA_HARIHARABRAHMA_FROM_THE_SECOND_LORD", 0),
    ("PARIJATA_HARIHARABRAHMA_FROM_THE_SEVENTH_LORD", 0),
    ("PARIJATA_KAHALA", 26),
    ("PARIJATA_KALANIDHI", 2),
    ("PARIJATA_KHADGA", 0),
    ("PARIJATA_KURMA", 0),
    ("PARIJATA_KUSUMA", 0),
    ("PARIJATA_LAKSHMI", 0),
    ("PARIJATA_MALIKA_FROM_THE_EIGHTH", 0),
    ("PARIJATA_MALIKA_FROM_THE_ELEVENTH", 0),
    ("PARIJATA_MALIKA_FROM_THE_FIFTH", 0),
    ("PARIJATA_MALIKA_FROM_THE_FIRST", 0),
    ("PARIJATA_MALIKA_FROM_THE_FOURTH", 0),
    ("PARIJATA_MALIKA_FROM_THE_NINTH", 0),
    ("PARIJATA_MALIKA_FROM_THE_SECOND", 0),
    ("PARIJATA_MALIKA_FROM_THE_SEVENTH", 0),
    ("PARIJATA_MALIKA_FROM_THE_SIXTH", 0),
    ("PARIJATA_MALIKA_FROM_THE_TENTH", 0),
    ("PARIJATA_MALIKA_FROM_THE_THIRD", 0),
    ("PARIJATA_MALIKA_FROM_THE_TWELFTH", 0),
    ("PARIJATA_MATSYA", 0),
    ("PARIJATA_MRIDANGA", 5),
    ("PARIJATA_PARIJATA", 58),
    ("PARIJATA_PARVATA", 25),
    ("PARIJATA_SANKHA", 22),
    ("PARIJATA_SARADA", 0),
    ("PARIJATA_SRINATHA", 0),
];
