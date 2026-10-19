//! Jataka Parijata ch. 3 vv. 61 to 79, the place of birth, the lamp, the
//! lying-in chamber and the marks of the body, held to the 93 recorded
//! charts: what each answers, and that the figures the verses state as Brihat
//! Jataka ch. 5 does answer what Varahamihira's do.

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

/// The pack, read for its keys: the verses share ch. 3 with
/// `classical-parijata-yoga.json`, so the chapter alone does not part them.
const PACK: &str = include_str!("../rules/classical-parijata-late.json");

/// Every rule the pack ships.
fn verses() -> Vec<Rule> {
    let pack: serde_json::Value = serde_json::from_str(PACK).expect("the pack reads");
    let keys: Vec<&str> = pack["rules"]
        .as_array()
        .expect("the pack lists rules")
        .iter()
        .map(|rule| rule["key"].as_str().expect("a rule has a key"))
        .collect();
    shipped::nabhasas()
        .iter()
        .filter(|rule| keys.contains(&rule.key.as_str()))
        .cloned()
        .collect()
}

/// The figures the verses state as Brihat Jataka ch. 5 does, each with
/// Varahamihira's, read from their own texts to the same conditions.
const TWINS: [(&str, &str); 22] = [
    (
        "JANMA_BOAT_FULL_MOON_IN_CANCER_MERCURY_RISING_A_BENEFIC_FOURTH",
        "BJ_JANMA_FULL_MOON_IN_CANCER_MERCURY_RISING_A_BENEFIC_FOURTH_IN_A_BOAT",
    ),
    (
        "JANMA_BOAT_WATERY_SIGN_RISING_MOON_SEVENTH",
        "BJ_JANMA_WATERY_SIGN_RISING_MOON_SEVENTH_IN_A_BOAT",
    ),
    (
        "JANMA_WATERY_SIGN_RISING_BY_THE_WATERS",
        "BJ_JANMA_WATERY_SIGN_RISING_BY_THE_WATERS",
    ),
    (
        "JANMA_MOON_RISING_SATURN_TWELFTH_ASPECTED_BY_A_MALEFIC",
        "BJ_JANMA_MOON_RISING_SATURN_TWELFTH_ASPECTED_BY_A_MALEFIC_IN_PRISON",
    ),
    (
        "JANMA_SCORPIO_OR_CANCER_RISING_WITH_SATURN_ASPECTED_BY_THE_MOON",
        "BJ_JANMA_SCORPIO_OR_CANCER_RISING_WITH_SATURN_ASPECTED_BY_THE_MOON_IN_A_PIT",
    ),
    (
        "JANMA_WATERY_SIGN_RISING_WITH_SATURN_ASPECTED_BY_MERCURY",
        "BJ_JANMA_WATERY_SIGN_RISING_WITH_SATURN_ASPECTED_BY_MERCURY",
    ),
    (
        "JANMA_WATERY_SIGN_RISING_WITH_SATURN_ASPECTED_BY_SUN",
        "BJ_JANMA_WATERY_SIGN_RISING_WITH_SATURN_ASPECTED_BY_SUN",
    ),
    (
        "JANMA_WATERY_SIGN_RISING_WITH_SATURN_ASPECTED_BY_MOON",
        "BJ_JANMA_WATERY_SIGN_RISING_WITH_SATURN_ASPECTED_BY_MOON",
    ),
    (
        "JANMA_HUMAN_SIGN_RISING_WITH_SATURN_ASPECTED_BY_MARS",
        "BJ_JANMA_BIPED_SIGN_RISING_WITH_SATURN_ASPECTED_BY_MARS",
    ),
    (
        "JANMA_HUMAN_SIGN_RISING_WITH_SATURN_ASPECTED_BY_VENUS_AND_THE_MOON",
        "BJ_JANMA_BIPED_SIGN_RISING_WITH_SATURN_ASPECTED_BY_VENUS_AND_THE_MOON",
    ),
    (
        "JANMA_HUMAN_SIGN_RISING_WITH_SATURN_ASPECTED_BY_JUPITER",
        "BJ_JANMA_BIPED_SIGN_RISING_WITH_SATURN_ASPECTED_BY_JUPITER",
    ),
    (
        "JANMA_HUMAN_SIGN_RISING_WITH_SATURN_ASPECTED_BY_SUN",
        "BJ_JANMA_BIPED_SIGN_RISING_WITH_SATURN_ASPECTED_BY_SUN",
    ),
    (
        "JANMA_HUMAN_SIGN_RISING_WITH_SATURN_ASPECTED_BY_MERCURY",
        "BJ_JANMA_BIPED_SIGN_RISING_WITH_SATURN_ASPECTED_BY_MERCURY",
    ),
    (
        "JANMA_VARGOTTAMA_LAGNA",
        "BJ_JANMA_VARGOTTAMA_LAGNA_IN_THE_MOTHERS_OWN_HOUSE",
    ),
    (
        "JANMA_MOON_RISING_ASPECTED_BY_A_MALEFIC_MARS_SEVENTH",
        "BJ_JANMA_MOON_RISING_ASPECTED_BY_A_MALEFIC_MARS_SEVENTH_DESERTED_AND_DIES",
    ),
    (
        "JANMA_MOON_RISING_MARS_AND_SATURN_ELEVENTH",
        "BJ_JANMA_MOON_RISING_MARS_AND_SATURN_ELEVENTH_DESERTED_AND_DIES",
    ),
    (
        "JANMA_DESERTED_MOON_ASPECTED_BY_A_BENEFIC",
        "BJ_JANMA_DESERTED_MOON_ASPECTED_BY_A_BENEFIC_TAKEN_IN",
    ),
    (
        "JANMA_MOON_DARKENED_BY_SATURN",
        "BJ_JANMA_MOON_DARKENED_BY_SATURN_BORN_IN_THE_DARK",
    ),
    (
        "JANMA_THREE_DEBILITATED",
        "BJ_JANMA_THREE_DEBILITATED_BORN_ON_THE_GROUND",
    ),
    (
        "JANMA_SIGN_RISING_HEAD_FIRST",
        "BJ_JANMA_SIGN_RISING_HEAD_FIRST",
    ),
    (
        "JANMA_SIGN_RISING_FEET_FIRST",
        "BJ_JANMA_SIGN_RISING_FEET_FIRST",
    ),
    (
        "JANMA_SIGN_RISING_BOTH_WAYS",
        "BJ_JANMA_SIGN_RISING_BOTH_WAYS",
    ),
];

#[test]
fn the_place_lamp_and_chamber_verses_answer_and_their_twins_agree() {
    let rules = verses();
    assert_eq!(rules.len(), 46);
    for rule in &rules {
        assert!(rule.is_evaluable(), "{} is evaluable", rule.key);
        assert_eq!(rule.source.chapter.as_deref(), Some("3"), "{}", rule.key);
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
        .map(|(ours, theirs)| (find(&format!("PARIJATA_{ours}")), find(theirs)))
        .collect();
    for (ours, theirs) in &twins {
        assert_eq!(ours.conditions, theirs.conditions, "{}", ours.key);
        assert_eq!(ours.cancellations, theirs.cancellations, "{}", ours.key);
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

/// What each answers over the 93 recorded charts: to be filled from a run.
const ANSWERED: [(&str, usize); 46] = [
    ("PARIJATA_JANMA_A_BENEFIC_DEBILITATED", 25),
    (
        "PARIJATA_JANMA_BOAT_FULL_MOON_IN_CANCER_MERCURY_RISING_A_BENEFIC_FOURTH",
        0,
    ),
    ("PARIJATA_JANMA_BOAT_WATERY_SIGN_RISING_MOON_SEVENTH", 1),
    ("PARIJATA_JANMA_CHAMBER_IN_THE_EAST", 18),
    ("PARIJATA_JANMA_CHAMBER_IN_THE_NORTH", 35),
    ("PARIJATA_JANMA_CHAMBER_IN_THE_SOUTH", 11),
    ("PARIJATA_JANMA_CHAMBER_IN_THE_WEST", 9),
    ("PARIJATA_JANMA_CHAMBER_JUPITER_THE_STRONGEST", 10),
    ("PARIJATA_JANMA_CHAMBER_MARS_THE_STRONGEST", 3),
    ("PARIJATA_JANMA_CHAMBER_MERCURY_THE_STRONGEST", 17),
    ("PARIJATA_JANMA_CHAMBER_SATURN_THE_STRONGEST", 5),
    ("PARIJATA_JANMA_CHAMBER_THE_MOON_THE_STRONGEST", 21),
    ("PARIJATA_JANMA_CHAMBER_THE_SUN_THE_STRONGEST", 14),
    ("PARIJATA_JANMA_CHAMBER_VENUS_THE_STRONGEST", 1),
    (
        "PARIJATA_JANMA_DESERTED_MOON_ASPECTED_BY_ANOTHER_MALEFIC",
        0,
    ),
    ("PARIJATA_JANMA_DESERTED_MOON_ASPECTED_BY_A_BENEFIC", 0),
    ("PARIJATA_JANMA_FOUR_WITH_MERCURY_IN_ONE_DREKKANA", 0),
    (
        "PARIJATA_JANMA_HUMAN_SIGN_RISING_WITH_SATURN_ASPECTED_BY_JUPITER",
        0,
    ),
    (
        "PARIJATA_JANMA_HUMAN_SIGN_RISING_WITH_SATURN_ASPECTED_BY_MARS",
        0,
    ),
    (
        "PARIJATA_JANMA_HUMAN_SIGN_RISING_WITH_SATURN_ASPECTED_BY_MERCURY",
        0,
    ),
    (
        "PARIJATA_JANMA_HUMAN_SIGN_RISING_WITH_SATURN_ASPECTED_BY_SUN",
        0,
    ),
    (
        "PARIJATA_JANMA_HUMAN_SIGN_RISING_WITH_SATURN_ASPECTED_BY_VENUS_AND_THE_MOON",
        0,
    ),
    ("PARIJATA_JANMA_LAGNA_AND_ITS_NAVAMSHA_DUAL", 7),
    ("PARIJATA_JANMA_LAGNA_AND_ITS_NAVAMSHA_FIXED", 3),
    ("PARIJATA_JANMA_LAGNA_AND_ITS_NAVAMSHA_MOVABLE", 2),
    ("PARIJATA_JANMA_LAMP_SUN_IN_A_DUAL_SIGN", 32),
    ("PARIJATA_JANMA_LAMP_SUN_IN_A_FIXED_SIGN", 26),
    ("PARIJATA_JANMA_LAMP_SUN_IN_A_MOVABLE_SIGN", 35),
    (
        "PARIJATA_JANMA_MALEFICS_WITH_THE_MOON_OR_FOURTH_OR_SEVENTH_FROM_HER",
        40,
    ),
    ("PARIJATA_JANMA_MOON_DARKENED_BY_SATURN", 36),
    (
        "PARIJATA_JANMA_MOON_RISING_ASPECTED_BY_A_MALEFIC_MARS_SEVENTH",
        0,
    ),
    ("PARIJATA_JANMA_MOON_RISING_MARS_AND_SATURN_ELEVENTH", 0),
    (
        "PARIJATA_JANMA_MOON_RISING_SATURN_TWELFTH_ASPECTED_BY_A_MALEFIC",
        0,
    ),
    (
        "PARIJATA_JANMA_MOON_RISING_UNASPECTED_BY_BENEFICS_TOGETHER",
        2,
    ),
    (
        "PARIJATA_JANMA_MOON_SEVENTH_IN_THE_TRINES_OF_MARS_AND_SATURN",
        1,
    ),
    (
        "PARIJATA_JANMA_MOON_SEVENTH_IN_THE_TRINES_OF_MARS_AND_SATURN_ASPECTED_BY_JUPITER",
        1,
    ),
    (
        "PARIJATA_JANMA_SCORPIO_OR_CANCER_RISING_WITH_SATURN_ASPECTED_BY_THE_MOON",
        0,
    ),
    ("PARIJATA_JANMA_SIGN_RISING_BOTH_WAYS", 22),
    ("PARIJATA_JANMA_SIGN_RISING_FEET_FIRST", 21),
    ("PARIJATA_JANMA_SIGN_RISING_HEAD_FIRST", 50),
    ("PARIJATA_JANMA_THREE_DEBILITATED", 2),
    ("PARIJATA_JANMA_VARGOTTAMA_LAGNA", 2),
    ("PARIJATA_JANMA_WATERY_SIGN_RISING_BY_THE_WATERS", 9),
    (
        "PARIJATA_JANMA_WATERY_SIGN_RISING_WITH_SATURN_ASPECTED_BY_MERCURY",
        0,
    ),
    (
        "PARIJATA_JANMA_WATERY_SIGN_RISING_WITH_SATURN_ASPECTED_BY_MOON",
        0,
    ),
    (
        "PARIJATA_JANMA_WATERY_SIGN_RISING_WITH_SATURN_ASPECTED_BY_SUN",
        0,
    ),
];
