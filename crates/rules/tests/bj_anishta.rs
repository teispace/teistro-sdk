//! Brihat Jataka ch. 23's evil yogas, held to the 93 recorded charts: what
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
            rule.source.text == "Brihat Jataka" && rule.source.chapter.as_deref() == Some("23")
        })
        .cloned()
        .collect()
}

#[test]
fn the_evils_of_the_chapter_answer_where_they_did() {
    let rules = chapter();
    assert_eq!(rules.len(), 42);
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
const ANSWERED: [(&str, usize); 42] = [
    (
        "BJ_ANISHTA_ECLIPSED_MOON_RISING_SATURN_AND_MARS_IN_THE_TRINES",
        0,
    ),
    (
        "BJ_ANISHTA_ECLIPSED_SUN_RISING_SATURN_AND_MARS_IN_THE_TRINES",
        0,
    ),
    ("BJ_ANISHTA_LEPROUS_SIGN_FIFTH_OR_NINTH", 15),
    ("BJ_ANISHTA_LUMINARIES_MARS_SATURN_IN_THE_HIDDEN_HOUSES", 0),
    (
        "BJ_ANISHTA_LUMINARIES_MARS_SATURN_IN_THE_NINTH_ELEVENTH_THIRD_FIFTH",
        1,
    ),
    ("BJ_ANISHTA_LUMINARIES_SEVENTH_ASPECTED_BY_SATURN", 0),
    ("BJ_ANISHTA_LUMINARIES_SIXTH_AND_TWELFTH", 0),
    ("BJ_ANISHTA_LUMINARIES_TOGETHER_IN_CANCER_OR_LEO", 2),
    ("BJ_ANISHTA_MALEFICS_FOURTH_AND_EIGHTH_FROM_VENUS", 10),
    ("BJ_ANISHTA_MALEFICS_TWELFTH_FIFTH_SECOND_NINTH", 2),
    (
        "BJ_ANISHTA_MALEFICS_TWELFTH_SEVENTH_AND_FIRST_WANING_MOON_FIFTH",
        0,
    ),
    (
        "BJ_ANISHTA_MALEFIC_SAGITTARIUS_OR_TAURUS_RISING_ASPECTED_BY_A_MALEFIC",
        29,
    ),
    ("BJ_ANISHTA_MARS_IN_A_TRINE_ASPECTED_BY_A_MALEFIC", 3),
    ("BJ_ANISHTA_MARS_SEVENTH_JUPITER_RISING", 3),
    (
        "BJ_ANISHTA_MOON_BETWEEN_SATURN_AND_MARS_SUN_IN_CAPRICORN",
        0,
    ),
    (
        "BJ_ANISHTA_MOON_IN_A_CANCER_OR_SCORPIO_NAVAMSHA_WITH_A_MALEFIC",
        5,
    ),
    ("BJ_ANISHTA_MOON_IN_A_LEPROUS_NAVAMSHA", 2),
    (
        "BJ_ANISHTA_MOON_RISING_SATURN_AND_MARS_ABOUT_SUN_SEVENTH",
        0,
    ),
    (
        "BJ_ANISHTA_MOON_TENTH_MARS_SEVENTH_SATURN_SECOND_FROM_THE_SUN",
        0,
    ),
    ("BJ_ANISHTA_MOON_TENTH_VENUS_SEVENTH_MALEFIC_FOURTH", 0),
    ("BJ_ANISHTA_MOON_WITH_SATURN_ASPECTED_BY_MARS", 1),
    ("BJ_ANISHTA_NO_SONS", 27),
    ("BJ_ANISHTA_NO_WIFE", 16),
    ("BJ_ANISHTA_SATURN_IN_A_TRINE_ASPECTED_BY_A_MALEFIC", 2),
    ("BJ_ANISHTA_SATURN_RISING_MARS_IN_A_TRINE_OR_THE_SEVENTH", 0),
    ("BJ_ANISHTA_SATURN_RISING_VENUS_SEVENTH_AT_A_JUNCTION", 0),
    ("BJ_ANISHTA_SATURN_SEVENTH_JUPITER_RISING", 3),
    ("BJ_ANISHTA_SUN_IN_A_TRINE_ASPECTED_BY_A_MALEFIC", 1),
    ("BJ_ANISHTA_SUN_IN_THE_MOONS_SIGN_OR_NAVAMSHA", 14),
    ("BJ_ANISHTA_SUN_SATURN_AND_MARS_TENTH_UNASPECTED", 0),
    (
        "BJ_ANISHTA_TAURUS_ARIES_OR_SAGITTARIUS_RISING_ASPECTED_BY_A_MALEFIC",
        9,
    ),
    ("BJ_ANISHTA_VENUS_AND_MARS_SEVENTH_ASPECTED_BY_A_MALEFIC", 0),
    ("BJ_ANISHTA_VENUS_AND_SUN_IN_A_TRINE_OR_THE_SEVENTH", 2),
    ("BJ_ANISHTA_VENUS_HEMMED_BY_MALEFICS", 19),
    ("BJ_ANISHTA_VENUS_MARRED_MOON_SATURN_MARS_SEVENTH", 0),
    ("BJ_ANISHTA_VENUS_SEVENTH_MARRED", 0),
    ("BJ_ANISHTA_VENUS_TWELFTH_IN_A_SATURNIAN_NAVAMSHA", 11),
    ("BJ_ANISHTA_VENUS_UNTENDED_BY_BENEFICS", 31),
    ("BJ_ANISHTA_VENUS_WITH_MOON_SATURN_AND_MARS_SEVENTH", 0),
    ("BJ_ANISHTA_VIRGO_SUN_RISING_MARS_IN_CAPRICORN", 0),
    ("BJ_ANISHTA_VIRGO_SUN_RISING_SATURN_IN_PISCES", 0),
    ("BJ_ANISHTA_WANING_MOON_AND_SATURN_TWELFTH", 0),
];
