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
const ANSWERED: [(&str, usize); 123] = [
    ("UTTARA_AQUARIUS_LAGNA_MERCURY_WITH_MARS", 0),
    ("UTTARA_ARIES_LAGNA_JUPITER_WITH_SATURN", 0),
    ("UTTARA_ARIES_LAGNA_JUPITER_WITH_SUN", 0),
    ("UTTARA_ARUDHA_LAGNA_A_BENEFICS_SIGN_HELD_BY_A_BENEFIC", 13),
    ("UTTARA_ARUDHA_LAGNA_EXALTED_GRAHA_FOURTH", 2),
    (
        "UTTARA_ARUDHA_LAGNA_FOURTH_HELD_OR_ASPECTED_BY_THE_MOON_AND_VENUS",
        3,
    ),
    ("UTTARA_ATMAKARAKA_BENEFIC_THIRD", 26),
    (
        "UTTARA_ATMAKARAKA_FOURTH_HELD_OR_ASPECTED_BY_THE_MOON_AND_VENUS",
        0,
    ),
    ("UTTARA_ATMAKARAKA_JOINED_BY_A_NODE", 13),
    ("UTTARA_ATMAKARAKA_JOINED_BY_JUPITER", 9),
    ("UTTARA_ATMAKARAKA_JOINED_BY_MARS", 11),
    ("UTTARA_ATMAKARAKA_JOINED_BY_MERCURY", 11),
    ("UTTARA_ATMAKARAKA_JOINED_BY_SATURN", 10),
    ("UTTARA_ATMAKARAKA_JOINED_BY_SUN", 13),
    ("UTTARA_ATMAKARAKA_JOINED_BY_VENUS", 15),
    ("UTTARA_ATMAKARAKA_MALEFIC_THIRD", 33),
    ("UTTARA_ATMAKARAKA_MOON_FOURTH_ASPECTED_BY_VENUS", 0),
    (
        "UTTARA_ATMAKARAKA_SECOND_HELD_OR_ASPECTED_BY_THE_SUN_AND_VENUS",
        1,
    ),
    (
        "UTTARA_BHAVA_EIGHTH_DESTROYED_ITS_LORD_EIGHTH_FROM_IT_COMBUST_FALLEN_OR_HOSTILE",
        28,
    ),
    (
        "UTTARA_BHAVA_EIGHTH_LOST_ITS_LORD_AFFLICTED_AND_UNHELPED",
        36,
    ),
    (
        "UTTARA_BHAVA_EIGHTH_PROMOTED_ITS_LORDS_DISPOSITORS_STRONG",
        46,
    ),
    (
        "UTTARA_BHAVA_EIGHTH_PROSPERS_ITS_LORD_WELL_PLACED_HEMMED_BY_STRONG_BENEFICS",
        0,
    ),
    (
        "UTTARA_BHAVA_ELEVENTH_DESTROYED_ITS_LORD_EIGHTH_FROM_IT_COMBUST_FALLEN_OR_HOSTILE",
        20,
    ),
    (
        "UTTARA_BHAVA_ELEVENTH_LOST_ITS_LORD_AFFLICTED_AND_UNHELPED",
        27,
    ),
    (
        "UTTARA_BHAVA_ELEVENTH_PROMOTED_ITS_LORDS_DISPOSITORS_STRONG",
        43,
    ),
    (
        "UTTARA_BHAVA_ELEVENTH_PROSPERS_ITS_LORD_WELL_PLACED_HEMMED_BY_STRONG_BENEFICS",
        0,
    ),
    (
        "UTTARA_BHAVA_FIFTH_DESTROYED_ITS_LORD_EIGHTH_FROM_IT_COMBUST_FALLEN_OR_HOSTILE",
        34,
    ),
    (
        "UTTARA_BHAVA_FIFTH_LOST_ITS_LORD_AFFLICTED_AND_UNHELPED",
        41,
    ),
    (
        "UTTARA_BHAVA_FIFTH_PROMOTED_ITS_LORDS_DISPOSITORS_STRONG",
        42,
    ),
    (
        "UTTARA_BHAVA_FIFTH_PROSPERS_ITS_LORD_WELL_PLACED_HEMMED_BY_STRONG_BENEFICS",
        0,
    ),
    (
        "UTTARA_BHAVA_FIRST_DESTROYED_ITS_LORD_EIGHTH_FROM_IT_COMBUST_FALLEN_OR_HOSTILE",
        28,
    ),
    (
        "UTTARA_BHAVA_FIRST_LOST_ITS_LORD_AFFLICTED_AND_UNHELPED",
        44,
    ),
    (
        "UTTARA_BHAVA_FIRST_PROMOTED_ITS_LORDS_DISPOSITORS_STRONG",
        45,
    ),
    (
        "UTTARA_BHAVA_FIRST_PROSPERS_ITS_LORD_WELL_PLACED_HEMMED_BY_STRONG_BENEFICS",
        0,
    ),
    (
        "UTTARA_BHAVA_FOURTH_DESTROYED_ITS_LORD_EIGHTH_FROM_IT_COMBUST_FALLEN_OR_HOSTILE",
        8,
    ),
    (
        "UTTARA_BHAVA_FOURTH_LOST_ITS_LORD_AFFLICTED_AND_UNHELPED",
        19,
    ),
    (
        "UTTARA_BHAVA_FOURTH_PROMOTED_ITS_LORDS_DISPOSITORS_STRONG",
        37,
    ),
    (
        "UTTARA_BHAVA_FOURTH_PROSPERS_ITS_LORD_WELL_PLACED_HEMMED_BY_STRONG_BENEFICS",
        0,
    ),
    (
        "UTTARA_BHAVA_NINTH_DESTROYED_ITS_LORD_EIGHTH_FROM_IT_COMBUST_FALLEN_OR_HOSTILE",
        21,
    ),
    (
        "UTTARA_BHAVA_NINTH_LOST_ITS_LORD_AFFLICTED_AND_UNHELPED",
        32,
    ),
    (
        "UTTARA_BHAVA_NINTH_PROMOTED_ITS_LORDS_DISPOSITORS_STRONG",
        41,
    ),
    (
        "UTTARA_BHAVA_NINTH_PROSPERS_ITS_LORD_WELL_PLACED_HEMMED_BY_STRONG_BENEFICS",
        0,
    ),
    (
        "UTTARA_BHAVA_SECOND_DESTROYED_ITS_LORD_EIGHTH_FROM_IT_COMBUST_FALLEN_OR_HOSTILE",
        11,
    ),
    (
        "UTTARA_BHAVA_SECOND_LOST_ITS_LORD_AFFLICTED_AND_UNHELPED",
        33,
    ),
    (
        "UTTARA_BHAVA_SECOND_PROMOTED_ITS_LORDS_DISPOSITORS_STRONG",
        40,
    ),
    (
        "UTTARA_BHAVA_SECOND_PROSPERS_ITS_LORD_WELL_PLACED_HEMMED_BY_STRONG_BENEFICS",
        0,
    ),
    (
        "UTTARA_BHAVA_SEVENTH_DESTROYED_ITS_LORD_EIGHTH_FROM_IT_COMBUST_FALLEN_OR_HOSTILE",
        17,
    ),
    (
        "UTTARA_BHAVA_SEVENTH_LOST_ITS_LORD_AFFLICTED_AND_UNHELPED",
        32,
    ),
    (
        "UTTARA_BHAVA_SEVENTH_PROMOTED_ITS_LORDS_DISPOSITORS_STRONG",
        40,
    ),
    (
        "UTTARA_BHAVA_SEVENTH_PROSPERS_ITS_LORD_WELL_PLACED_HEMMED_BY_STRONG_BENEFICS",
        0,
    ),
    (
        "UTTARA_BHAVA_SIXTH_DESTROYED_ITS_LORD_EIGHTH_FROM_IT_COMBUST_FALLEN_OR_HOSTILE",
        9,
    ),
    (
        "UTTARA_BHAVA_SIXTH_LOST_ITS_LORD_AFFLICTED_AND_UNHELPED",
        19,
    ),
    (
        "UTTARA_BHAVA_SIXTH_PROMOTED_ITS_LORDS_DISPOSITORS_STRONG",
        37,
    ),
    (
        "UTTARA_BHAVA_SIXTH_PROSPERS_ITS_LORD_WELL_PLACED_HEMMED_BY_STRONG_BENEFICS",
        0,
    ),
    (
        "UTTARA_BHAVA_TENTH_DESTROYED_ITS_LORD_EIGHTH_FROM_IT_COMBUST_FALLEN_OR_HOSTILE",
        17,
    ),
    (
        "UTTARA_BHAVA_TENTH_LOST_ITS_LORD_AFFLICTED_AND_UNHELPED",
        29,
    ),
    (
        "UTTARA_BHAVA_TENTH_PROMOTED_ITS_LORDS_DISPOSITORS_STRONG",
        33,
    ),
    (
        "UTTARA_BHAVA_TENTH_PROSPERS_ITS_LORD_WELL_PLACED_HEMMED_BY_STRONG_BENEFICS",
        0,
    ),
    (
        "UTTARA_BHAVA_THIRD_DESTROYED_ITS_LORD_EIGHTH_FROM_IT_COMBUST_FALLEN_OR_HOSTILE",
        9,
    ),
    (
        "UTTARA_BHAVA_THIRD_LOST_ITS_LORD_AFFLICTED_AND_UNHELPED",
        18,
    ),
    (
        "UTTARA_BHAVA_THIRD_PROMOTED_ITS_LORDS_DISPOSITORS_STRONG",
        35,
    ),
    (
        "UTTARA_BHAVA_THIRD_PROSPERS_ITS_LORD_WELL_PLACED_HEMMED_BY_STRONG_BENEFICS",
        0,
    ),
    (
        "UTTARA_BHAVA_TWELFTH_DESTROYED_ITS_LORD_EIGHTH_FROM_IT_COMBUST_FALLEN_OR_HOSTILE",
        25,
    ),
    (
        "UTTARA_BHAVA_TWELFTH_LOST_ITS_LORD_AFFLICTED_AND_UNHELPED",
        36,
    ),
    (
        "UTTARA_BHAVA_TWELFTH_PROMOTED_ITS_LORDS_DISPOSITORS_STRONG",
        35,
    ),
    (
        "UTTARA_BHAVA_TWELFTH_PROSPERS_ITS_LORD_WELL_PLACED_HEMMED_BY_STRONG_BENEFICS",
        0,
    ),
    ("UTTARA_CHANDRADHI", 6),
    (
        "UTTARA_DHANA_BENEFIC_ON_THE_INDU_LAGNA_WITHOUT_A_MALEFIC",
        0,
    ),
    ("UTTARA_DHANA_EXALTED_MALEFIC_ALONE_ON_THE_INDU_LAGNA", 0),
    (
        "UTTARA_DHANA_LOST_TWO_LORDS_OF_THE_SECOND_FIFTH_ELEVENTH_NINTH_RELATED_ONE_A_DUSTHANA_LORD",
        14,
    ),
    ("UTTARA_DHANA_MALEFIC_ALONE_ON_THE_INDU_LAGNA", 0),
    (
        "UTTARA_DHANA_NINTH_AND_TENTH_LORDS_IN_THE_FIRST_FOURTH_FIFTH_OR_SEVENTH",
        8,
    ),
    (
        "UTTARA_DHANA_NINTH_AND_TENTH_LORDS_JOINED_BY_A_LORD_OF_THE_FIRST_FOURTH_FIFTH_OR_SEVENTH",
        3,
    ),
    (
        "UTTARA_DHANA_SATURN_RISING_IN_PISCES_LIBRA_OR_SAGITTARIUS",
        1,
    ),
    (
        "UTTARA_DHANA_SECOND_AND_ELEVENTH_LORDS_TOGETHER_EXCHANGED_OR_DIGNIFIED_IN_MUTUAL_ASPECT",
        1,
    ),
    (
        "UTTARA_DHANA_TWO_LORDS_OF_THE_SECOND_FIFTH_ELEVENTH_NINTH_RELATED_STRONG",
        23,
    ),
    ("UTTARA_EIGHTH_AND_TWELFTH_LORDS_TOGETHER_OR_EXCHANGED", 13),
    (
        "UTTARA_JANMA_JARAJA_SUNDAY_TUESDAY_OR_SATURDAY_ON_A_BHADRA_TITHI",
        8,
    ),
    ("UTTARA_KALATRA_BENEFIC_ON_THE_SEVENTHS_UPAPADA", 16),
    (
        "UTTARA_KALATRA_DEBILITATED_MALEFIC_ON_THE_SEVENTHS_UPAPADA",
        2,
    ),
    (
        "UTTARA_KALATRA_EXALTED_MALEFIC_ON_THE_SEVENTHS_UPAPADA_WITHOUT_A_BENEFIC",
        0,
    ),
    (
        "UTTARA_KALATRA_EXALTED_MALEFIC_ON_THE_SEVENTHS_UPAPADA_WITH_A_BENEFIC",
        0,
    ),
    ("UTTARA_KALATRA_VAHNINIPATA", 1),
    ("UTTARA_LEO_LAGNA_JUPITER_WITH_VENUS", 0),
    ("UTTARA_LIBRA_LAGNA_MOON_WITH_MERCURY", 2),
    (
        "UTTARA_LORDS_OF_THE_THIRD_FOURTH_FIFTH_SEVENTH_TOGETHER_EXCHANGED_OR_DIGNIFIED_IN_MUTUAL_ASPECT",
        47,
    ),
    (
        "UTTARA_MOON_WITH_VENUS_THEIR_LORD_IN_A_KENDRA_OR_TRIKONA",
        5,
    ),
    ("UTTARA_NODE_IN_A_KENDRA_OR_TRIKONA_WITH_ITS_LORD", 1),
    ("UTTARA_NODE_WITH_A_TRIKONA_LORD", 14),
    ("UTTARA_PISCES_LAGNA_JUPITER_WITH_MARS", 0),
    ("UTTARA_PUTRA_ADOPTED_SATURN_ON_THE_TENTH_AND_ITS_LORD", 5),
    ("UTTARA_PUTRA_SATURN_AND_JUPITER_TOGETHER", 24),
    ("UTTARA_RAJA_ALL_IN_KENDRAS_AND_PANAPHARAS", 3),
    (
        "UTTARA_RAJA_ALL_IN_KENDRAS_AND_PANAPHARAS_BENEFICS_AND_FULL_MOON_STRONG_UNAFFLICTED",
        1,
    ),
    (
        "UTTARA_RAJA_BENEFIC_IN_A_KENDRA_OR_TRIKONA_FROM_THE_ARUDHA_LAGNA",
        80,
    ),
    ("UTTARA_RAJA_EIGHTH_LORD_IN_THE_TWELFTH_OR_SIXTH", 38),
    (
        "UTTARA_RAJA_FULL_STRONG_MOON_DIGNIFIED_IN_THE_NINTH_FOURTH_TENTH_OR_SEVENTH_WITH_JUPITER_OR_VENUS",
        2,
    ),
    ("UTTARA_RAJA_LAGNA_AND_MOON_SIGN_LORDS_EXCHANGED", 0),
    ("UTTARA_RAJA_LAGNA_LORD_RISING_IN_THE_FIRST_DREKKANA", 3),
    ("UTTARA_RAJA_LAGNA_LORD_RISING_IN_THE_SECOND_DREKKANA", 0),
    ("UTTARA_RAJA_LAGNA_LORD_RISING_IN_THE_THIRD_DREKKANA", 2),
    (
        "UTTARA_RAJA_LAGNA_LORD_RISING_WITH_A_BENEFICS_HELP_OR_EXALTED",
        4,
    ),
    (
        "UTTARA_RAJA_LORDS_OF_THE_SIXTH_EIGHTH_AND_TWELFTH_RELATED_AMONG_THEMSELVES_ALONE",
        1,
    ),
    (
        "UTTARA_RAJA_MALEFIC_DEBILITATED_STRONG_BENEFIC_ANGULAR_OR_TRINE_LORDS_OF_TEN_NINE_FOUR_TOGETHER",
        0,
    ),
    ("UTTARA_RAJA_NINTH_AND_TENTH_LORDS_EACH_AT_HOME", 1),
    ("UTTARA_RAJA_NINTH_AND_TENTH_LORDS_EXCHANGED", 0),
    (
        "UTTARA_RAJA_NINTH_AND_TENTH_LORDS_FROM_THE_NINTH_RELATED_STRONG_OR_IN_MUTUAL_KENDRAS",
        31,
    ),
    (
        "UTTARA_RAJA_NINTH_AND_TENTH_LORDS_FROM_THE_TENTH_RELATED_STRONG_OR_IN_MUTUAL_KENDRAS",
        37,
    ),
    ("UTTARA_RAJA_NINTH_AND_TENTH_LORDS_IN_MUTUAL_ASPECT", 14),
    (
        "UTTARA_RAJA_NINTH_AND_TENTH_LORDS_RELATED_STRONG_OR_IN_MUTUAL_KENDRAS",
        42,
    ),
    (
        "UTTARA_RAJA_NINTH_AND_TENTH_LORDS_TOGETHER_EXCHANGED_OR_DIGNIFIED_IN_MUTUAL_ASPECT",
        12,
    ),
    (
        "UTTARA_RAJA_NINTH_AND_TENTH_LORDS_TOGETHER_IN_THE_NINTH_OR_TENTH",
        2,
    ),
    (
        "UTTARA_RAJA_NODE_AND_A_TRIKONA_LORD_IN_THE_NINTH_OR_TENTH",
        5,
    ),
    (
        "UTTARA_RAJA_SATURN_RISING_IN_PISCES_LIBRA_OR_SAGITTARIUS_SEVENTH_FREE_OF_JUPITER_AND_MERCURY",
        1,
    ),
    ("UTTARA_RAJA_SIXTH_LORD_IN_THE_EIGHTH_OR_TWELFTH", 14),
    (
        "UTTARA_RAJA_STRONG_MARS_AND_SATURN_SECOND_AND_TENTH_ASPECTED_BY_BENEFICS",
        0,
    ),
    ("UTTARA_RAJA_TWELFTH_LORD_IN_THE_EIGHTH_OR_SIXTH", 14),
    ("UTTARA_SAGITTARIUS_LAGNA_SUN_WITH_MERCURY", 2),
    (
        "UTTARA_SATURN_EIGHTH_FROM_THE_FOURTH_LORD_IN_AN_ODD_NAVAMSHA",
        0,
    ),
    (
        "UTTARA_SATURN_EIGHTH_FROM_THE_NINTH_LORD_IN_AN_ODD_NAVAMSHA",
        0,
    ),
    ("UTTARA_SCORPIO_LAGNA_SUN_WITH_MOON", 0),
    ("UTTARA_STRONG_BENEFIC_IN_A_FRIENDLY_SIGN_AND_NAVAMSHA", 22),
    ("UTTARA_VIRGO_LAGNA_VENUS_WITH_MOON", 1),
];
