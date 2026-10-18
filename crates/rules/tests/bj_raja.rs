//! Brihat Jataka ch. 11's raja yogas, held to the 93 recorded charts: what
//! each answers, and that the chapter's counts do not meet.

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

/// Every rule the chapter ships.
fn chapter() -> Vec<Rule> {
    shipped::nabhasas()
        .iter()
        .filter(|rule| {
            rule.source.text == "Brihat Jataka" && rule.source.chapter.as_deref() == Some("11")
        })
        .cloned()
        .collect()
}

#[test]
fn the_kings_of_the_chapter_answer_where_they_did() {
    let rules = chapter();
    assert_eq!(rules.len(), 30);
    for rule in &rules {
        assert!(rule.is_evaluable(), "{} is evaluable", rule.key);
        assert!(rule.source.verse.is_some(), "{}: cites a verse", rule.key);
    }
    teistro_rules::check_references(&rules).expect("the chapter names itself soundly");
    let at = |key: &str| {
        rules
            .iter()
            .position(|rule| rule.key == format!("BJ_RAJA_{key}"))
            .unwrap()
    };
    // Each pair counts the same figure in ranges that do not meet.
    let apart = [
        (
            at("THREE_OF_FOUR_EXALTED_ONE_RISING"),
            at("ONE_OR_TWO_OF_FOUR_EXALTED_ONE_RISING_MOON_IN_CANCER"),
        ),
        (
            at("THREE_STRONG_AND_DIGNIFIED"),
            at("FIVE_STRONG_AND_DIGNIFIED"),
        ),
    ];
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
        for (rule, _) in rules.iter().zip(&present).filter(|(_, present)| **present) {
            *fired.get_mut(rule.key.as_str()).unwrap() += 1;
        }
        for (one, other) in apart {
            assert!(!(present[one] && present[other]), "{}", path.display());
        }
    }
    let counts: Vec<(&str, usize)> = fired.into_iter().collect();
    assert_eq!(counts.as_slice(), ANSWERED.as_slice());
}

/// What each answers over the 93 recorded charts; the figures of named signs
/// hold on none of them and are measured on the computed sky and its fixtures
/// instead (`rule-cofiring-measured.md`).
const ANSWERED: [(&str, usize); 30] = [
    ("BJ_RAJA_AQUARIUS_ARIES_TAURUS_FIGURE", 0),
    ("BJ_RAJA_ARIES_SUN_AND_MOON_RISING_FIGURE", 0),
    ("BJ_RAJA_ARIES_SUN_RISING_FIGURE", 0),
    ("BJ_RAJA_BENEFICS_SIGN_RISING_SATURN_SEVENTH_SUN_TENTH", 0),
    (
        "BJ_RAJA_BENEFIC_SIGNS_IN_THE_KENDRAS_MALEFICS_IN_MALEFIC_SIGNS",
        8,
    ),
    ("BJ_RAJA_CANCER_JUPITER_RISING_FIGURE", 0),
    ("BJ_RAJA_CAPRICORN_MARS_AND_MOON_FIGURE", 0),
    ("BJ_RAJA_CAPRICORN_MARS_AND_SATURN_FIGURE", 0),
    ("BJ_RAJA_CAPRICORN_SATURN_RISING_FIGURE", 0),
    ("BJ_RAJA_CAPRICORN_SATURN_RISING_LORDS_AT_HOME_FIGURE", 0),
    ("BJ_RAJA_FIVE_STRONG_AND_DIGNIFIED", 0),
    ("BJ_RAJA_JUPITER_RISING_MOON_TENTH", 0),
    ("BJ_RAJA_LIBRA_TAURUS_FIGURE", 0),
    ("BJ_RAJA_MARS_AND_SATURN_RISING", 0),
    ("BJ_RAJA_MARS_IN_ARIES_JUPITER_IN_CANCER", 0),
    (
        "BJ_RAJA_ONE_OR_TWO_OF_FOUR_EXALTED_ONE_RISING_MOON_IN_CANCER",
        0,
    ),
    ("BJ_RAJA_PISCES_MOON_RISING_FIGURE", 0),
    ("BJ_RAJA_SAGITTARIUS_MOON_AND_JUPITER_FIGURE", 0),
    ("BJ_RAJA_SATURN_RISING_LUMINARIES_TENTH", 0),
    ("BJ_RAJA_STRONG_MERCURY_RISING", 0),
    ("BJ_RAJA_TAURUS_MOON_RISING_FIGURE", 0),
    ("BJ_RAJA_TAURUS_MOON_RISING_OTHERS_ELEVENTH", 0),
    ("BJ_RAJA_THREE_MALEFICS_EXALTED", 0),
    ("BJ_RAJA_THREE_OF_FOUR_EXALTED_ONE_RISING", 0),
    ("BJ_RAJA_THREE_STRONG_AND_DIGNIFIED", 7),
    ("BJ_RAJA_VARGOTTAMA_LAGNA_ASPECTED_BY_FOUR", 0),
    ("BJ_RAJA_VARGOTTAMA_MOON_ASPECTED_BY_FOUR", 0),
    ("BJ_RAJA_VENUS_AT_HOME_IN_THE_FOURTH", 0),
    ("BJ_RAJA_VIRGO_MERCURY_RISING_FIFTH_AND_FOURTH_FIGURE", 0),
    ("BJ_RAJA_VIRGO_MERCURY_RISING_PISCES_FIGURE", 0),
];
