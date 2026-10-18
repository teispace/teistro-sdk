//! Saravali ch. 35's raja yogas, vv. 7 to 102, held to the 93 recorded charts: what
//! each answers.

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
            rule.source.text == "Saravali" && rule.source.chapter.as_deref() == Some("35")
        })
        .cloned()
        .collect()
}

#[test]
fn the_kings_of_the_chapter_answer_where_they_did() {
    let rules = chapter();
    assert_eq!(rules.len(), 43);
    for rule in &rules {
        assert!(rule.is_evaluable(), "{} is evaluable", rule.key);
        assert!(rule.source.verse.is_some(), "{}: cites a verse", rule.key);
    }
    teistro_rules::check_references(&rules).expect("the chapter names itself soundly");
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
    }
    let counts: Vec<(&str, usize)> = fired.into_iter().collect();
    assert_eq!(counts.as_slice(), ANSWERED.as_slice());
}

/// What each answers over the 93 recorded charts; the figures that hold on
/// none of them are measured on the computed sky and its fixtures instead
/// (`rule-cofiring-measured.md`).
const ANSWERED: [(&str, usize); 43] = [
    ("SARAVALI_RAJA_ALL_IN_THE_THIRD_FOURTH_FIFTH_AND_TENTH", 0),
    ("SARAVALI_RAJA_ALL_VARGOTTAMA_OR_AT_HOME", 0),
    (
        "SARAVALI_RAJA_ARIES_RISING_JUPITER_AND_SUN_IN_LEO_FIGURE",
        0,
    ),
    ("SARAVALI_RAJA_ARIES_RISING_WITH_MOON_MARS_AND_JUPITER", 0),
    (
        "SARAVALI_RAJA_BENEFICS_IN_THE_FIRST_FOURTH_SEVENTH_MALEFICS_THIRD_NINTH_ELEVENTH",
        0,
    ),
    ("SARAVALI_RAJA_CANCER_MOON_RISING_FIGURE", 0),
    (
        "SARAVALI_RAJA_EXALTED_MARS_ASPECTED_BY_SUN_MOON_AND_JUPITER",
        0,
    ),
    ("SARAVALI_RAJA_EXALTED_MOON_RISING_FIGURE", 0),
    ("SARAVALI_RAJA_EXALTED_SUN_AND_MOON_RISING", 0),
    (
        "SARAVALI_RAJA_EXALTED_SUN_TENTH_WITH_JUPITER_VENUS_NINTH",
        0,
    ),
    ("SARAVALI_RAJA_FULL_MOON_AND_MARS_RISING", 0),
    (
        "SARAVALI_RAJA_FULL_MOON_IN_AN_ARIES_NAVAMSHA_ASPECTED_BY_JUPITER_ALONE",
        0,
    ),
    (
        "SARAVALI_RAJA_FULL_MOON_IN_AN_UPACHAYA_ASPECTED_BY_JUPITER_ALONE",
        0,
    ),
    (
        "SARAVALI_RAJA_FULL_MOON_IN_A_TAURUS_NAVAMSHA_ASPECTED_BY_ALL",
        0,
    ),
    (
        "SARAVALI_RAJA_JUPITER_AND_MOON_FIFTH_THREE_IN_OWN_NAVAMSHAS",
        0,
    ),
    ("SARAVALI_RAJA_JUPITER_EXALTED_IN_A_KENDRA_VENUS_TENTH", 0),
    ("SARAVALI_RAJA_JUPITER_EXALTED_MARS_RISING_IN_ARIES", 0),
    ("SARAVALI_RAJA_JUPITER_FOURTH_AT_HOME_FULL_MOON_NINTH", 0),
    (
        "SARAVALI_RAJA_JUPITER_IN_A_NAVAMSHA_OF_MARS_SUN_TENTH_IN_ARIES",
        0,
    ),
    ("SARAVALI_RAJA_JUPITER_IN_TAURUS_ASPECTS_THE_MOON", 0),
    ("SARAVALI_RAJA_KALASHA", 0),
    (
        "SARAVALI_RAJA_LAGNA_AND_MOON_SIGN_LORDS_IN_KENDRAS_MERCURY_IN_A_TRIKONA",
        3,
    ),
    (
        "SARAVALI_RAJA_LAGNA_LORD_IN_A_KENDRA_ASPECTED_BY_A_FRIEND_BENEFIC_RISING",
        1,
    ),
    ("SARAVALI_RAJA_LEO_SUN_RISING_FIGURE", 0),
    (
        "SARAVALI_RAJA_LUMINARIES_EARLY_SAGITTARIUS_SATURN_RISING",
        0,
    ),
    (
        "SARAVALI_RAJA_MERCURY_RISING_JUPITER_SEVENTH_MOON_FOURTH_IN_CANCER",
        0,
    ),
    (
        "SARAVALI_RAJA_MOON_AND_JUPITER_FIFTH_ASPECTED_BY_ITS_LORD_VENUS_IN_PISCES",
        0,
    ),
    ("SARAVALI_RAJA_MOON_ASPECTED_BY_ALL", 0),
    (
        "SARAVALI_RAJA_MOON_AT_HIGHEST_EXALTATION_ASPECTED_BY_VENUS",
        0,
    ),
    (
        "SARAVALI_RAJA_MOON_SIGN_LORD_IN_AN_UPACHAYA_FROM_THE_MOON",
        0,
    ),
    ("SARAVALI_RAJA_MOON_SIXTH_IN_A_WATERY_NAVAMSHA", 0),
    (
        "SARAVALI_RAJA_ONE_EXALTED_VARGOTTAMA_ASPECTED_BY_A_FRIEND",
        1,
    ),
    ("SARAVALI_RAJA_SATURN_LATE_CAPRICORN_FIGURE", 0),
    (
        "SARAVALI_RAJA_STRONG_MOON_IN_THE_LAST_DECANATE_OF_CANCER",
        1,
    ),
    ("SARAVALI_RAJA_SUN_AT_HIGHEST_EXALTATION_RISING", 0),
    ("SARAVALI_RAJA_TAURUS_RISING_JUPITER_AND_MOON", 0),
    ("SARAVALI_RAJA_THREE_VARGOTTAMA_IN_KENDRAS", 0),
    ("SARAVALI_RAJA_VENUS_AQUARIUS_MARS_ARIES_JUPITER_EXALTED", 0),
    (
        "SARAVALI_RAJA_VENUS_MERCURY_MARS_RISING_MOON_AND_JUPITER_SEVENTH",
        0,
    ),
    (
        "SARAVALI_RAJA_VENUS_MOON_SUN_THIRD_MARS_SEVENTH_JUPITER_NINTH",
        0,
    ),
    (
        "SARAVALI_RAJA_VENUS_RISING_IN_A_SIGN_OF_JUPITER_MERCURY_EXALTED",
        0,
    ),
    (
        "SARAVALI_RAJA_VENUS_SUN_AND_MOON_TOGETHER_ASPECTED_BY_JUPITER_ALONE",
        0,
    ),
    ("SARAVALI_RAJA_VIRGO_MERCURY_RISING_MOON_ELEVENTH", 0),
];
