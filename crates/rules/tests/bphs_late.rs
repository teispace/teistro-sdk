//! BPHS chs. 24, 30, 33 and 80, the lords of the houses, the upapada, the
//! Karakamsha and the woman's chart, held to the 93 recorded charts: what
//! each figure answers, and that those the chapters state as another text or
//! chapter does (Brihat Jataka ch. 24, Phaladeepika ch. 10, Jataka Parijata
//! ch. 3 and BPHS ch. 44) answer alike.

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

/// The chapters the pack ships.
const CHAPTERS: [&str; 4] = ["24", "30", "33", "80"];

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

/// The figures the chapters state as another text or chapter does, each with
/// the other rule: their conditions are one, so the two must answer alike.
const TWINS: [(&str, &str); 61] = [
    ("BPHS_BHAVESHA_SEVENTH_LORD_IN_THE_FIFTH", "PHALADEEPIKA_KALATRA_SEVENTH_LORD_FIFTH"),
    ("BPHS_STRI_LAGNA_AND_MOON_IN_EVEN_SIGNS", "BJ_STRI_LAGNA_AND_MOON_IN_EVEN_SIGNS"),
    (
        "BPHS_STRI_LAGNA_AND_MOON_IN_EVEN_SIGNS_ASPECTED_BY_BENEFICS",
        "BJ_STRI_LAGNA_AND_MOON_IN_EVEN_SIGNS_ASPECTED_BY_BENEFICS",
    ),
    ("BPHS_STRI_LAGNA_AND_MOON_IN_ODD_SIGNS", "BJ_STRI_LAGNA_AND_MOON_IN_ODD_SIGNS"),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_MARS",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_MARS",
    ),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_VENUS",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_VENUS",
    ),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_MERCURY",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_MERCURY",
    ),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_JUPITER",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_JUPITER",
    ),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_SATURN",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MARS_AND_A_TRIMSHAMSHA_OF_SATURN",
    ),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_MARS",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_MARS",
    ),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_VENUS",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_VENUS",
    ),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_MERCURY",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_MERCURY",
    ),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_SATURN",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_SATURN",
    ),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_JUPITER",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_MERCURY_AND_A_TRIMSHAMSHA_OF_JUPITER",
    ),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_MARS",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_MARS",
    ),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_VENUS",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_VENUS",
    ),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_MERCURY",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_MERCURY",
    ),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_JUPITER",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_JUPITER",
    ),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_SATURN",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_VENUS_AND_A_TRIMSHAMSHA_OF_SATURN",
    ),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_MARS",
        "BJ_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_MARS",
    ),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_VENUS",
        "BJ_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_VENUS",
    ),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_MERCURY",
        "BJ_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_MERCURY",
    ),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_JUPITER",
        "BJ_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_JUPITER",
    ),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_SATURN",
        "BJ_STRI_LAGNA_OR_MOON_IN_CANCER_AND_A_TRIMSHAMSHA_OF_SATURN",
    ),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_MARS",
        "BJ_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_MARS",
    ),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_VENUS",
        "BJ_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_VENUS",
    ),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_MERCURY",
        "BJ_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_MERCURY",
    ),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_JUPITER",
        "BJ_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_JUPITER",
    ),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_SATURN",
        "BJ_STRI_LAGNA_OR_MOON_IN_LEO_AND_A_TRIMSHAMSHA_OF_SATURN",
    ),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_MARS",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_MARS",
    ),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_VENUS",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_VENUS",
    ),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_MERCURY",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_MERCURY",
    ),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_JUPITER",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_JUPITER",
    ),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_SATURN",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_JUPITER_AND_A_TRIMSHAMSHA_OF_SATURN",
    ),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_MARS",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_MARS",
    ),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_VENUS",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_VENUS",
    ),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_MERCURY",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_MERCURY",
    ),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_JUPITER",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_JUPITER",
    ),
    (
        "BPHS_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_SATURN",
        "BJ_STRI_LAGNA_OR_MOON_IN_A_SIGN_OF_SATURN_AND_A_TRIMSHAMSHA_OF_SATURN",
    ),
    ("BPHS_STRI_MARS_SEVENTH", "PARIJATA_JANMA_FATHER_AWAY_MARS_SEVENTH"),
    (
        "BPHS_STRI_VENUS_AND_MARS_IN_EACH_OTHERS_NAVAMSHA",
        "BJ_STRI_VENUS_AND_MARS_IN_EACH_OTHERS_NAVAMSHA_ADULTERY",
    ),
    ("BPHS_STRI_SEVENTH_SIGN_OR_NAVAMSHA_OF_MARS", "BJ_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_MARS"),
    (
        "BPHS_STRI_SEVENTH_SIGN_OR_NAVAMSHA_OF_MERCURY",
        "BJ_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_MERCURY",
    ),
    (
        "BPHS_STRI_SEVENTH_SIGN_OR_NAVAMSHA_OF_JUPITER",
        "BJ_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_JUPITER",
    ),
    ("BPHS_STRI_SEVENTH_SIGN_OR_NAVAMSHA_OF_VENUS", "BJ_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_VENUS"),
    ("BPHS_STRI_SEVENTH_SIGN_OR_NAVAMSHA_OF_SATURN", "BJ_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_SATURN"),
    (
        "BPHS_STRI_SEVENTH_SIGN_OR_NAVAMSHA_OF_THE_SUN",
        "BJ_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_THE_SUN",
    ),
    (
        "BPHS_STRI_SEVENTH_SIGN_OR_NAVAMSHA_OF_THE_MOON",
        "BJ_STRI_SETTING_SIGN_OR_NAVAMSHA_OF_THE_MOON",
    ),
    ("BPHS_STRI_SUN_EIGHTH", "BPHS_SUN_IN_EIGHTH_DEATH_BY_FIRE"),
    ("BPHS_STRI_MOON_EIGHTH", "BPHS_MOON_IN_EIGHTH_DEATH_BY_WATER"),
    ("BPHS_STRI_MARS_EIGHTH", "BPHS_MARS_IN_EIGHTH_DEATH_BY_WEAPONS"),
    ("BPHS_STRI_MERCURY_EIGHTH", "BPHS_MERCURY_IN_EIGHTH_DEATH_BY_FEVER"),
    ("BPHS_STRI_JUPITER_EIGHTH", "BPHS_JUPITER_IN_EIGHTH_DEATH_BY_DISEASE"),
    ("BPHS_STRI_VENUS_EIGHTH", "BPHS_VENUS_IN_EIGHTH_DEATH_BY_HUNGER"),
    ("BPHS_STRI_SATURN_EIGHTH", "BPHS_SATURN_IN_EIGHTH_DEATH_BY_THIRST"),
    (
        "BPHS_STRI_SEVENTH_NAVAMSHA_OF_MARS_ASPECTED_BY_SATURN",
        "BJ_STRI_SETTING_NAVAMSHA_OF_MARS_ASPECTED_BY_SATURN",
    ),
    ("BPHS_STRI_SEVENTH_NAVAMSHA_OF_A_BENEFIC", "BJ_STRI_SETTING_NAVAMSHA_OF_A_BENEFIC"),
    (
        "BPHS_STRI_MOON_AND_VENUS_RISING_IN_A_SIGN_OF_SATURN_OR_MARS_ASPECTED_BY_A_MALEFIC",
        "BJ_STRI_MOON_AND_VENUS_RISING_IN_A_SIGN_OF_SATURN_OR_MARS_ASPECTED_BY_A_MALEFIC",
    ),
    (
        "BPHS_STRI_SATURN_AND_VENUS_IN_EACH_OTHERS_NAVAMSHA_ASPECTING_EACH_OTHER",
        "BJ_STRI_SATURN_AND_VENUS_IN_EACH_OTHERS_NAVAMSHA_ASPECTING_EACH_OTHER",
    ),
    (
        "BPHS_STRI_SIGN_OF_VENUS_RISING_IN_AN_AQUARIUS_NAVAMSHA",
        "BJ_STRI_TAURUS_OR_LIBRA_RISING_IN_AN_AQUARIUS_NAVAMSHA",
    ),
    (
        "BPHS_STRI_MARS_MERCURY_JUPITER_AND_VENUS_STRONG_EVEN_LAGNA",
        "BJ_STRI_JUPITER_MARS_VENUS_MERCURY_STRONG_EVEN_LAGNA",
    ),
];

#[test]
fn the_late_chapters_answer_and_their_twins_agree() {
    let rules = chapters();
    assert_eq!(rules.len(), 419);
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
