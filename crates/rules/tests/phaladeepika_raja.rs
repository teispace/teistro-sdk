//! Phaladeepika ch. 7's Maharaja yogas, held to the 93 recorded charts: what
//! each answers, and what the chapter's own grades say must hold between them.

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
            rule.source.text == "Phaladeepika" && rule.source.chapter.as_deref() == Some("7")
        })
        .cloned()
        .collect()
}

#[test]
fn the_kings_of_the_chapter_answer_where_they_did() {
    let rules = chapter();
    assert_eq!(rules.len(), 46);
    for rule in &rules {
        assert!(rule.is_evaluable(), "{} is evaluable", rule.key);
        assert!(rule.source.verse.is_some(), "{}: cites a verse", rule.key);
    }
    teistro_rules::check_references(&rules).expect("the chapter names itself soundly");
    let at = |key: &str| {
        rules
            .iter()
            .position(|rule| rule.key == format!("PHALADEEPIKA_MAHARAJA_{key}"))
            .unwrap()
    };
    // v. 28's grade: the second holds only where the first does.
    let within = [(
        at("NEECHA_BHANGA_ASPECTED_BY_ITS_LORD"),
        at("NEECHA_BHANGA_ASPECTED_BY_ITS_LORD_IN_A_GOOD_HOUSE"),
    )];
    // Each pair counts the same figure in ranges that do not meet.
    let apart = [
        (
            at("THREE_DIGNIFIED_IN_KENDRAS"),
            at("FIVE_DIGNIFIED_IN_KENDRAS"),
        ),
        (
            at("ONE_DEBILITATED_RETROGRADE"),
            at("TWO_DEBILITATED_RETROGRADE"),
        ),
        (at("TWO_WITH_DIG_BALA"), at("FOUR_WITH_DIG_BALA")),
        (
            at("WATERY_MOON_RISING"),
            at("WATERY_MOON_AWAY_FROM_THE_KENDRAS"),
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
        for (wide, narrow) in within {
            assert!(!present[narrow] || present[wide], "{}", path.display());
        }
        for (one, other) in apart {
            assert!(!(present[one] && present[other]), "{}", path.display());
        }
    }
    let counts: Vec<(&str, usize)> = fired.into_iter().collect();
    assert_eq!(counts.as_slice(), ANSWERED.as_slice());
}

/// What each answers over the 93 recorded charts. Most of the figures of
/// named signs and many-graha counts hold on none of them, and are measured
/// on the computed sky and its fixtures instead (`rule-cofiring-measured.md`).
const ANSWERED: [(&str, usize); 46] = [
    ("PHALADEEPIKA_MAHARAJA_BRIGHT_BENEFIC_IN_THE_NINTH", 12),
    ("PHALADEEPIKA_MAHARAJA_FIVE_DIGNIFIED_IN_KENDRAS", 0),
    ("PHALADEEPIKA_MAHARAJA_FOUR_WITH_DIG_BALA", 0),
    ("PHALADEEPIKA_MAHARAJA_FULL_MOON_EXALTED", 1),
    ("PHALADEEPIKA_MAHARAJA_FULL_MOON_IN_A_KENDRA", 2),
    ("PHALADEEPIKA_MAHARAJA_FULL_MOON_IN_A_SOLAR_NAVAMSHA", 0),
    ("PHALADEEPIKA_MAHARAJA_FULL_MOON_IN_PISCES", 5),
    ("PHALADEEPIKA_MAHARAJA_FULL_MOON_THREE_IN_OWN_NAVAMSHAS", 0),
    ("PHALADEEPIKA_MAHARAJA_FULL_MOON_VARGOTTAMA", 3),
    ("PHALADEEPIKA_MAHARAJA_JUPITER_AND_MOON_IN_KENDRAS", 1),
    ("PHALADEEPIKA_MAHARAJA_JUPITER_RISING", 5),
    ("PHALADEEPIKA_MAHARAJA_LAGNA_AND_NINTH_LORDS_VARGOTTAMA", 0),
    (
        "PHALADEEPIKA_MAHARAJA_MALEFICS_IN_UPACHAYAS_FROM_THE_LORDS",
        0,
    ),
    ("PHALADEEPIKA_MAHARAJA_MARS_AND_MERCURY_SECOND", 0),
    ("PHALADEEPIKA_MAHARAJA_MARS_RISING_IN_FIRE", 0),
    (
        "PHALADEEPIKA_MAHARAJA_MARS_SATURN_JUPITER_TENTH_ELEVENTH_FIRST",
        0,
    ),
    (
        "PHALADEEPIKA_MAHARAJA_MOON_IN_A_GREAT_FRIENDS_NAVAMSHA_ASPECTED_BY_JUPITER",
        4,
    ),
    (
        "PHALADEEPIKA_MAHARAJA_MOON_IN_A_GREAT_FRIENDS_NAVAMSHA_ASPECTED_BY_VENUS",
        0,
    ),
    (
        "PHALADEEPIKA_MAHARAJA_NEECHA_BHANGA_ASPECTED_BY_ITS_LORD",
        4,
    ),
    (
        "PHALADEEPIKA_MAHARAJA_NEECHA_BHANGA_ASPECTED_BY_ITS_LORD_IN_A_GOOD_HOUSE",
        4,
    ),
    (
        "PHALADEEPIKA_MAHARAJA_NEECHA_BHANGA_DEBILITATED_IN_A_KENDRA",
        33,
    ),
    (
        "PHALADEEPIKA_MAHARAJA_NEECHA_BHANGA_EXALTED_ONE_IN_A_KENDRA",
        27,
    ),
    (
        "PHALADEEPIKA_MAHARAJA_NEECHA_BHANGA_LORDS_IN_MUTUAL_KENDRAS",
        23,
    ),
    ("PHALADEEPIKA_MAHARAJA_NEECHA_BHANGA_LORD_IN_A_KENDRA", 48),
    ("PHALADEEPIKA_MAHARAJA_NIGHT_MOON_IN_CANCER_TENTH", 0),
    ("PHALADEEPIKA_MAHARAJA_NINTH_AND_TENTH_LORDS_EXCHANGED", 0),
    ("PHALADEEPIKA_MAHARAJA_ONE_AT_HIGHEST_EXALTATION", 0),
    ("PHALADEEPIKA_MAHARAJA_ONE_DEBILITATED_RETROGRADE", 2),
    (
        "PHALADEEPIKA_MAHARAJA_ONE_LORD_OF_GAIN_IN_A_KENDRA_FROM_THE_MOON",
        5,
    ),
    ("PHALADEEPIKA_MAHARAJA_PISCES_FIGURE", 0),
    ("PHALADEEPIKA_MAHARAJA_SAGITTARIUS_FIGURE", 0),
    ("PHALADEEPIKA_MAHARAJA_STRONG_LAGNA_LORD_IN_A_KENDRA", 25),
    (
        "PHALADEEPIKA_MAHARAJA_STRONG_LAGNA_LORD_SECOND_WITH_VENUS",
        0,
    ),
    (
        "PHALADEEPIKA_MAHARAJA_STRONG_MERCURY_IN_A_KENDRA_ASPECTED_BY_JUPITER",
        10,
    ),
    ("PHALADEEPIKA_MAHARAJA_SUN_AND_VENUS_FOURTH", 5),
    (
        "PHALADEEPIKA_MAHARAJA_SUN_IN_OWN_NAVAMSHA_MOON_IN_CANCER",
        0,
    ),
    ("PHALADEEPIKA_MAHARAJA_THREE_DIGNIFIED_IN_KENDRAS", 3),
    ("PHALADEEPIKA_MAHARAJA_TWO_DEBILITATED_RETROGRADE", 0),
    ("PHALADEEPIKA_MAHARAJA_TWO_WITH_DIG_BALA", 16),
    ("PHALADEEPIKA_MAHARAJA_VARGOTTAMA_LAGNA_ASPECTED_BY_FOUR", 0),
    (
        "PHALADEEPIKA_MAHARAJA_VARGOTTAMA_MOON_ASPECTED_BY_A_STRONG_GRAHA",
        3,
    ),
    ("PHALADEEPIKA_MAHARAJA_VENUS_ASPECTED_BY_JUPITER", 43),
    ("PHALADEEPIKA_MAHARAJA_VENUS_IN_ASHVINI_RISING", 0),
    ("PHALADEEPIKA_MAHARAJA_WATERY_MOON_AWAY_FROM_THE_KENDRAS", 7),
    ("PHALADEEPIKA_MAHARAJA_WATERY_MOON_RISING", 9),
    (
        "PHALADEEPIKA_MAHARAJA_WHITE_MOON_ASPECTED_BY_A_DIGNIFIED_GRAHA",
        7,
    ),
];
