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
    (
        "BHAVA_SUN_OR_MARS_IN_THE_TENTH",
        "BHAVA_SUN_OR_MARS_IN_THE_TENTH_ITS_LORD_WELL_PLACED",
    ),
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
const ANSWERED: [(&str, usize); 59] = [
    ("PHALADEEPIKA_BHAVA_BENEFICS_IN_THE_TENTH", 0),
    ("PHALADEEPIKA_BHAVA_BENEFIC_IN_THE_TENTH", 7),
    (
        "PHALADEEPIKA_BHAVA_EIGHTH_LORD_AWAY_FROM_THE_KENDRAS_WEAKER_THAN_THE_LAGNA_LORD",
        37,
    ),
    ("PHALADEEPIKA_BHAVA_FIFTH_OF_MERCURY", 41),
    (
        "PHALADEEPIKA_BHAVA_FOURTH_LORD_AND_MOON_IN_DUSTHANAS_AFFLICTED",
        0,
    ),
    (
        "PHALADEEPIKA_BHAVA_FOURTH_LORD_AND_MOON_STRONG_WITH_BENEFICS_BENEFIC_FOURTH",
        7,
    ),
    (
        "PHALADEEPIKA_BHAVA_FOURTH_LORD_AND_VENUS_IN_THE_LAGNA_OR_FOURTH",
        11,
    ),
    (
        "PHALADEEPIKA_BHAVA_FOURTH_LORD_IN_THE_FOURTH_WITH_RAHU_OR_SATURN",
        0,
    ),
    (
        "PHALADEEPIKA_BHAVA_FOURTH_LORD_WITH_MARS_AND_THE_SUN_IN_A_DUSTHANA_OR_THEY_IN_THE_FOURTH",
        1,
    ),
    ("PHALADEEPIKA_BHAVA_FOURTH_OR_ITS_LORD_HELD_BY_AN_ENEMY", 27),
    ("PHALADEEPIKA_BHAVA_LAGNA_LORD_BRIGHT", 85),
    ("PHALADEEPIKA_BHAVA_LAGNA_LORD_CONNECTED_TO_A_BENEFIC", 70),
    ("PHALADEEPIKA_BHAVA_LAGNA_LORD_EXALTED", 3),
    (
        "PHALADEEPIKA_BHAVA_LAGNA_LORD_EXALTED_OR_OWN_IN_A_KENDRA_OR_TRIKONA_EIGHTH_LORD_AWAY_BENEFIC_RISING",
        2,
    ),
    ("PHALADEEPIKA_BHAVA_LAGNA_LORD_IN_A_DUAL_SIGN", 32),
    ("PHALADEEPIKA_BHAVA_LAGNA_LORD_IN_A_DUSTHANA", 20),
    ("PHALADEEPIKA_BHAVA_LAGNA_LORD_IN_A_FIXED_SIGN", 32),
    (
        "PHALADEEPIKA_BHAVA_LAGNA_LORD_IN_A_MALEFICS_ENEMYS_OR_DEBILITATION_SIGN",
        62,
    ),
    ("PHALADEEPIKA_BHAVA_LAGNA_LORD_IN_A_MOVABLE_SIGN", 29),
    ("PHALADEEPIKA_BHAVA_LAGNA_LORD_IN_OWN_SIGN", 14),
    ("PHALADEEPIKA_BHAVA_LAGNA_LORD_IN_THE_SIXTH", 4),
    ("PHALADEEPIKA_BHAVA_LAGNA_LORD_IN_THE_TENTH", 10),
    (
        "PHALADEEPIKA_BHAVA_LAGNA_LORD_OVER_THE_SIXTH_LORD_IN_A_BENEFIC_NAVAMSHA_FOURTH_LORD_STRONG",
        1,
    ),
    ("PHALADEEPIKA_BHAVA_LAGNA_LORD_WELL_PLACED", 73),
    ("PHALADEEPIKA_BHAVA_LAGNA_LORD_WITH_A_STRONG_GRAHA", 21),
    (
        "PHALADEEPIKA_BHAVA_LAGNA_LORD_WITH_OR_ASPECTED_BY_A_STRONG_SIXTH_LORD",
        12,
    ),
    (
        "PHALADEEPIKA_BHAVA_LORDS_OF_ONE_AND_FOUR_EXCHANGING_HOUSES_THE_MOON_WITH_ONE",
        8,
    ),
    (
        "PHALADEEPIKA_BHAVA_LORDS_OF_ONE_AND_FOUR_HOSTILE_AND_UNCONNECTED",
        4,
    ),
    (
        "PHALADEEPIKA_BHAVA_LORDS_OF_ONE_AND_THREE_EXCHANGED_AND_STRONG",
        0,
    ),
    (
        "PHALADEEPIKA_BHAVA_LUMINARIES_IN_THE_TRINES_OF_SATURN_AND_MARS",
        0,
    ),
    (
        "PHALADEEPIKA_BHAVA_LUMINARIES_IN_THE_TRINES_OF_SATURN_AND_MARS_ASPECTED_BY_JUPITER",
        0,
    ),
    ("PHALADEEPIKA_BHAVA_MALEFICS_IN_THE_SECOND", 16),
    ("PHALADEEPIKA_BHAVA_MALEFICS_IN_THE_SIXTH", 20),
    (
        "PHALADEEPIKA_BHAVA_MARS_OR_SUN_NINTH_ITS_LORD_IN_A_DUSTHANA_OR_HEMMED",
        1,
    ),
    (
        "PHALADEEPIKA_BHAVA_NINTH_OR_ITS_LORD_MOVABLE_WITH_SATURN_TWELFTH_LORD_STRONG",
        9,
    ),
    ("PHALADEEPIKA_BHAVA_RAHU_SATURN_OR_KETU_IN_THE_TENTH", 19),
    (
        "PHALADEEPIKA_BHAVA_SATURN_NINTH_LORD_IN_A_MOVABLE_SIGN_UNASPECTED_BY_BENEFICS",
        6,
    ),
    ("PHALADEEPIKA_BHAVA_SECOND_LORD_CONNECTED_TO_JUPITER", 47),
    ("PHALADEEPIKA_BHAVA_SECOND_LORD_CONNECTED_TO_KETU", 49),
    ("PHALADEEPIKA_BHAVA_SECOND_LORD_CONNECTED_TO_MARS", 27),
    ("PHALADEEPIKA_BHAVA_SECOND_LORD_CONNECTED_TO_MERCURY", 13),
    ("PHALADEEPIKA_BHAVA_SECOND_LORD_CONNECTED_TO_RAHU", 39),
    ("PHALADEEPIKA_BHAVA_SECOND_LORD_CONNECTED_TO_SATURN", 43),
    ("PHALADEEPIKA_BHAVA_SECOND_LORD_CONNECTED_TO_THE_MOON", 39),
    ("PHALADEEPIKA_BHAVA_SECOND_LORD_CONNECTED_TO_THE_SUN", 34),
    ("PHALADEEPIKA_BHAVA_SECOND_LORD_CONNECTED_TO_VENUS", 48),
    (
        "PHALADEEPIKA_BHAVA_SECOND_LORD_RISING_BENEFIC_IN_THE_SECOND",
        1,
    ),
    (
        "PHALADEEPIKA_BHAVA_SEVENTH_WITH_A_BENEFIC_ITS_LORD_STRONG",
        18,
    ),
    (
        "PHALADEEPIKA_BHAVA_SIXTH_LORD_FALLEN_IN_A_DUSTHANA_LAGNA_LORD_STRONGER_SUN_NINTH",
        0,
    ),
    ("PHALADEEPIKA_BHAVA_SIXTH_LORD_IN_A_KENDRA", 34),
    (
        "PHALADEEPIKA_BHAVA_SIXTH_LORD_WITH_OR_ASPECTED_BY_A_MALEFIC",
        65,
    ),
    (
        "PHALADEEPIKA_BHAVA_SUN_BY_DAY_SATURN_BY_NIGHT_WELL_PLACED_ASPECTED_BY_A_BENEFIC_NINTH_LORD_STRONG",
        12,
    ),
    ("PHALADEEPIKA_BHAVA_SUN_IN_A_DUSTHANA", 27),
    ("PHALADEEPIKA_BHAVA_SUN_OR_MARS_IN_THE_TENTH", 25),
    (
        "PHALADEEPIKA_BHAVA_SUN_OR_MARS_IN_THE_TENTH_ITS_LORD_WELL_PLACED",
        22,
    ),
    (
        "PHALADEEPIKA_BHAVA_TENTH_LORD_STRONG_EXALTED_OR_OWN_IN_A_KENDRA_OR_TRIKONA",
        14,
    ),
    (
        "PHALADEEPIKA_BHAVA_THIRD_LORD_AND_MARS_ODD_ASPECTED_BY_JUPITER_SUN_AND_MARS",
        0,
    ),
    (
        "PHALADEEPIKA_BHAVA_THIRD_LORD_STRONG_WITH_A_BENEFIC_MARS_STRONG_AND_WELL_PLACED",
        6,
    ),
    ("PHALADEEPIKA_BHAVA_THIRD_LORD_WEAK_IN_A_DUSTHANA", 3),
];
