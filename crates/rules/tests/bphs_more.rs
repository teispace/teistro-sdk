//! BPHS chs. 12 to 23, the effects of the twelve houses, held to the 93
//! recorded charts: what each figure answers, and that those the chapters
//! state as another text does (Jataka Parijata chs. 3 and 7, Brihat Jataka
//! ch. 5, Phaladeepika ch. 6, Saravali ch. 34 and BPHS ch. 44), or state twice
//! themselves, answer alike.

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

/// The chapters the pack ships, one to a house.
const CHAPTERS: [&str; 12] = [
    "12", "13", "14", "15", "16", "17", "18", "19", "20", "21", "22", "23",
];

/// Every rule the chapters ship.
fn chapters() -> Vec<Rule> {
    shipped::nabhasas()
        .iter()
        .filter(|rule| {
            rule.source.text == "BPHS"
                && rule
                    .source
                    .chapter
                    .as_deref()
                    .is_some_and(|chapter| CHAPTERS.contains(&chapter))
        })
        .cloned()
        .collect()
}

/// The figures the chapters state as another text does, or twice over, each
/// with the other rule: their conditions are copied, so the two must answer
/// alike.
const TWINS: [(&str, &str); 10] = [
    ("BPHS_TANU_BENEFIC_IN_THE_LAGNA", "PARIJATA_SHUBHA"),
    ("BPHS_TANU_MALEFIC_IN_THE_LAGNA", "PARIJATA_ASHUBHA"),
    (
        "BPHS_TANU_LAGNA_JOINED_OR_ASPECTED_BY_A_BENEFIC",
        "PARIJATA_GARBHA_BENEFIC_IN_OR_ASPECTING_THE_LAGNA_SON",
    ),
    (
        "BPHS_TANU_ARIES_TAURUS_OR_LEO_RISING_WITH_SATURN_OR_MARS_THE_CORD",
        "BJ_JANMA_ARIES_LEO_OR_TAURUS_RISING_WITH_SATURN_OR_MARS_THE_CORD",
    ),
    (
        "BPHS_TANU_SUN_IN_A_QUADRUPED_SIGN_THE_REST_STRONG_IN_DUAL_SIGNS_TWINS",
        "BJ_JANMA_SUN_IN_A_QUADRUPED_SIGN_THE_REST_STRONG_IN_DUAL_SIGNS_TWINS",
    ),
    ("BPHS_DHANA_SECOND_LORD_IN_A_DUSTHANA", "PHALADEEPIKA_NISSVA"),
    ("BPHS_DHANA_BENEFIC_IN_THE_SECOND", "SARAVALI_BENEFICS_IN_THE_SECOND"),
    ("BPHS_SAHAJA_MARS_IN_THE_THIRD", "BPHS_MARS_IN_THIRD_DEATH_BY_WOUNDS"),
    ("BPHS_PUTRA_FIFTH_LORD_IN_A_DUSTHANA", "PHALADEEPIKA_PAMARA"),
    (
        "BPHS_KALATRA_SECOND_AND_ELEVENTH_LORDS_EXCHANGED",
        "BPHS_LABHA_SECOND_AND_ELEVENTH_LORDS_EXCHANGED",
    ),
];

#[test]
fn the_houses_answer_and_their_twins_agree() {
    let rules = chapters();
    assert_eq!(rules.len(), 234);
    for rule in &rules {
        assert!(rule.is_evaluable(), "{} is evaluable", rule.key);
        assert!(rule.source.verse.is_some(), "{}: cites a verse", rule.key);
        assert!(rule.key.starts_with("BPHS_"), "{}", rule.key);
    }
    teistro_rules::check_references(&rules).expect("the chapters name themselves soundly");
    let find = |key: &str| {
        shipped::nabhasas()
            .iter()
            .find(|rule| rule.key == key)
            .cloned()
            .expect("the twin ships")
    };
    let twins: Vec<(Rule, Rule)> = TWINS
        .iter()
        .map(|(ours, theirs)| (find(ours), find(theirs)))
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

/// What each answers over the 93 recorded charts: left for the first run to
/// fill.
const ANSWERED: [(&str, usize); 0] = [];
