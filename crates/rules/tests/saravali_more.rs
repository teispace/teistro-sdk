//! Saravali chs. 20, 38, 39, 44 and 46's yogas, held to the 93 recorded
//! charts: what each answers, and that the yogas Saravali states as another
//! shipped text does answer what that text's do.

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

/// The chapters this pack encodes.
const CHAPTERS: [&str; 5] = ["20", "38", "39", "44", "46"];

/// The rules of the given Saravali chapters.
fn chapters(numbers: &[&str]) -> Vec<Rule> {
    shipped::nabhasas()
        .iter()
        .filter(|rule| {
            rule.source.text == "Saravali"
                && rule
                    .source
                    .chapter
                    .as_deref()
                    .is_some_and(|chapter| numbers.contains(&chapter))
        })
        .cloned()
        .collect()
}

/// The yogas Saravali states as another shipped text does, each with that
/// text's rule (ours without its `SARAVALI_` prefix first). The conditions
/// are one; where the outcomes differ, as ch. 46's places of death and
/// ch. 20's orders do, the difference is the outcome's and not the yoga's.
/// Ch. 44's friend and enemy counts are the ones Jataka Parijata ch. 8
/// restates.
const TWINS: [(&str, &str); 59] = [
    ("PRAVRAJYA_SUN_STRONGEST", "BJ_PRAVRAJYA_SUN"),
    ("PRAVRAJYA_MOON_STRONGEST", "BJ_PRAVRAJYA_MOON"),
    ("PRAVRAJYA_MARS_STRONGEST", "BJ_PRAVRAJYA_MARS"),
    ("PRAVRAJYA_MERCURY_STRONGEST", "BJ_PRAVRAJYA_MERCURY"),
    ("PRAVRAJYA_JUPITER_STRONGEST", "BJ_PRAVRAJYA_JUPITER"),
    ("PRAVRAJYA_VENUS_STRONGEST", "BJ_PRAVRAJYA_VENUS"),
    ("PRAVRAJYA_SATURN_STRONGEST", "BJ_PRAVRAJYA_SATURN"),
    (
        "PRAVRAJYA_MOON_STRONGEST_COMBUST",
        "BJ_PRAVRAJYA_MOON_COMBUST",
    ),
    (
        "PRAVRAJYA_MARS_STRONGEST_COMBUST",
        "BJ_PRAVRAJYA_MARS_COMBUST",
    ),
    (
        "PRAVRAJYA_MERCURY_STRONGEST_COMBUST",
        "BJ_PRAVRAJYA_MERCURY_COMBUST",
    ),
    (
        "PRAVRAJYA_JUPITER_STRONGEST_COMBUST",
        "BJ_PRAVRAJYA_JUPITER_COMBUST",
    ),
    (
        "PRAVRAJYA_VENUS_STRONGEST_COMBUST",
        "BJ_PRAVRAJYA_VENUS_COMBUST",
    ),
    (
        "PRAVRAJYA_SATURN_STRONGEST_COMBUST",
        "BJ_PRAVRAJYA_SATURN_COMBUST",
    ),
    (
        "PRAVRAJYA_MOON_IN_SATURNS_DREKKANA_ASPECTED_BY_MARS_AND_SATURN",
        "PHALADEEPIKA_PRAVRAJYA_MOON_IN_SATURNS_DREKKANA_ASPECTED_BY_MARS_AND_SATURN",
    ),
    (
        "PRAVRAJYA_MOON_IN_A_NAVAMSHA_OF_MARS_ASPECTED_BY_SATURN",
        "PHALADEEPIKA_PRAVRAJYA_MOON_IN_A_NAVAMSHA_OF_MARS_ASPECTED_BY_SATURN",
    ),
    (
        "PRAVRAJYA_MOON_SIGN_LORD_ASPECTED_BY_SATURN_ALONE",
        "PHALADEEPIKA_PRAVRAJYA_MOON_SIGN_LORD_ASPECTED_BY_SATURN_ALONE",
    ),
    (
        "BHANGA_THREE_DEBILITATED",
        "PARIJATA_YOGA_THREE_DEBILITATED",
    ),
    (
        "AYUR_CANCER_RISING_WITH_JUPITER_AND_MOON_UNLIMITED",
        "BJ_AYUR_CANCER_RISING_WITH_JUPITER_AND_MOON_BEYOND_RECKONING",
    ),
    (
        "MARANA_SUN_IN_THE_EIGHTH_DEATH_BY_FIRE",
        "BJ_MARANA_SUN_IN_THE_EIGHTH_DEATH_BY_FIRE",
    ),
    (
        "MARANA_MOON_IN_THE_EIGHTH_DEATH_BY_WATER",
        "BJ_MARANA_MOON_IN_THE_EIGHTH_DEATH_BY_WATER",
    ),
    (
        "MARANA_MARS_IN_THE_EIGHTH_DEATH_BY_A_WEAPON",
        "BJ_MARANA_MARS_IN_THE_EIGHTH_DEATH_BY_A_WEAPON",
    ),
    (
        "MARANA_MERCURY_IN_THE_EIGHTH_DEATH_BY_FEVER",
        "BJ_MARANA_MERCURY_IN_THE_EIGHTH_DEATH_BY_FEVER",
    ),
    (
        "MARANA_JUPITER_IN_THE_EIGHTH_DEATH_BY_DISEASE",
        "BJ_MARANA_JUPITER_IN_THE_EIGHTH_DEATH_BY_AN_UNKNOWN_ILLNESS",
    ),
    (
        "MARANA_VENUS_IN_THE_EIGHTH_DEATH_BY_THIRST",
        "BJ_MARANA_VENUS_IN_THE_EIGHTH_DEATH_BY_THIRST",
    ),
    (
        "MARANA_SATURN_IN_THE_EIGHTH_DEATH_BY_HUNGER",
        "BJ_MARANA_SATURN_IN_THE_EIGHTH_DEATH_BY_HUNGER",
    ),
    (
        "MARANA_MOVABLE_EIGHTH_PLACE_OF_DEATH",
        "BJ_MARANA_MOVABLE_EIGHTH_PLACE_OF_DEATH",
    ),
    (
        "MARANA_FIXED_EIGHTH_PLACE_OF_DEATH",
        "BJ_MARANA_FIXED_EIGHTH_PLACE_OF_DEATH",
    ),
    (
        "MARANA_DUAL_EIGHTH_PLACE_OF_DEATH",
        "BJ_MARANA_DUAL_EIGHTH_PLACE_OF_DEATH",
    ),
    (
        "MARANA_SUN_AND_MARS_TENTH_AND_FOURTH_FALL_FROM_A_HILL",
        "BJ_MARANA_SUN_AND_MARS_TENTH_AND_FOURTH_DEATH_BY_A_FALLING_STONE",
    ),
    (
        "MARANA_SATURN_MOON_MARS_FOURTH_SEVENTH_TENTH_IN_A_WELL",
        "BJ_MARANA_SATURN_MOON_MARS_FOURTH_SEVENTH_TENTH_DEATH_IN_A_WELL",
    ),
    (
        "MARANA_LUMINARIES_IN_VIRGO_ASPECTED_BY_A_MALEFIC_BONDS",
        "BJ_MARANA_LUMINARIES_IN_VIRGO_ASPECTED_BY_A_MALEFIC_DEATH_THROUGH_KIN",
    ),
    (
        "MARANA_LUMINARIES_IN_A_DUAL_LAGNA_DROWNING",
        "BJ_MARANA_LUMINARIES_IN_A_DUAL_LAGNA_DEATH_BY_DROWNING",
    ),
    (
        "MARANA_SATURN_IN_CANCER_MOON_IN_CAPRICORN_DROPSY",
        "BJ_MARANA_SATURN_IN_CANCER_MOON_IN_CAPRICORN_DEATH_BY_DROPSY",
    ),
    (
        "MARANA_MOON_IN_VIRGO_BETWEEN_MALEFICS",
        "BJ_MARANA_MOON_IN_VIRGO_BETWEEN_MALEFICS",
    ),
    (
        "MARANA_MOON_IN_A_SIGN_OF_SATURN_BETWEEN_MALEFICS",
        "BJ_MARANA_MOON_IN_A_SIGN_OF_SATURN_BETWEEN_MALEFICS",
    ),
    (
        "MARANA_TWENTY_SECOND_DREKKANA_A_SERPENT_NOOSE_OR_FETTER_CAPTIVITY",
        "BJ_MARANA_TWENTY_SECOND_DREKKANA_A_SERPENT_OR_A_NOOSE_DEATH_IN_CAPTIVITY",
    ),
    (
        "MARANA_PISCES_RISING_SUN_MOON_SEVENTH_WITH_A_MALEFIC_VENUS_IN_ARIES",
        "BJ_MARANA_MOON_IN_VIRGO_SEVENTH_WITH_A_MALEFIC_VENUS_IN_ARIES_SUN_RISING",
    ),
    (
        "MARANA_SUN_OR_MARS_FOURTH_SATURN_TENTH_IMPALED",
        "BJ_MARANA_SUN_OR_MARS_FOURTH_SATURN_TENTH_IMPALED",
    ),
    (
        "MARANA_MALEFICS_WITH_THE_WANING_MOON_IN_TRINES_AND_LAGNA_IMPALED",
        "BJ_MARANA_SUN_MARS_SATURN_IN_THE_FIRST_FIFTH_NINTH_WITH_THE_WANING_MOON_IMPALED",
    ),
    (
        "MARANA_WANING_MOON_EIGHTH_MARS_TENTH_SATURN_FIRST_SUN_FOURTH_CLUBBED",
        "BJ_MARANA_WANING_MOON_EIGHTH_MARS_TENTH_SATURN_FIRST_SUN_FOURTH_CLUBBED",
    ),
    (
        "MARANA_WANING_MOON_TENTH_MARS_NINTH_SATURN_FIRST_SUN_FIFTH_SMOKE_OR_FIRE",
        "BJ_MARANA_WANING_MOON_TENTH_MARS_NINTH_SATURN_FIRST_SUN_FIFTH_SMOKE_OR_FIRE",
    ),
    (
        "MARANA_MARS_FOURTH_SUN_SEVENTH_SATURN_TENTH_WEAPON_FIRE_OR_KING",
        "BJ_MARANA_MARS_FOURTH_SUN_SEVENTH_SATURN_TENTH_WEAPON_FIRE_OR_KING",
    ),
    (
        "MARANA_MARS_TENTH_MOON_FOURTH_SATURN_SECOND_WORMS_IN_A_WOUND",
        "BJ_MARANA_SATURN_SECOND_MOON_FOURTH_MARS_TENTH_WORMS_IN_A_WOUND",
    ),
    (
        "MARANA_WANING_MOON_TENTH_SUN_SEVENTH_MARS_FOURTH_FILTH",
        "BJ_MARANA_WANING_MOON_TENTH_SUN_SEVENTH_MARS_FOURTH_DEATH_IN_FILTH",
    ),
    (
        "MARANA_WANING_MOON_ASPECTED_BY_STRONG_MARS_SATURN_EIGHTH",
        "BJ_MARANA_WANING_MOON_ASPECTED_BY_STRONG_MARS_SATURN_EIGHTH_PILES",
    ),
    (
        "MARANA_SUN_AND_MARS_SEVENTH_SATURN_EIGHTH_MOON_FOURTH_BIRDS",
        "BJ_MARANA_SUN_AND_MARS_SEVENTH_SATURN_EIGHTH_WANING_MOON_FOURTH_BIRDS",
    ),
    (
        "MARANA_SUN_MARS_SATURN_MOON_IN_THE_LAGNA_EIGHTH_AND_TRINES_A_FALL",
        "BJ_MARANA_SUN_FIRST_MARS_FIFTH_SATURN_EIGHTH_MOON_NINTH_A_FALL",
    ),
    (
        "MARANA_UNCONSCIOUS_LAGNA_UNASPECTED_BY_ITS_LORD_OR_A_BENEFIC",
        "BJ_MARANA_UNCONSCIOUS_LAGNA_UNASPECTED_BY_ITS_LORD_OR_A_BENEFIC",
    ),
    (
        "MARANA_UNCONSCIOUS_LAGNA_ASPECTED_BY_ITS_LORD",
        "BJ_MARANA_UNCONSCIOUS_LAGNA_ASPECTED_BY_ITS_LORD",
    ),
    (
        "MARANA_UNCONSCIOUS_LAGNA_ASPECTED_BY_A_BENEFIC",
        "BJ_MARANA_UNCONSCIOUS_LAGNA_ASPECTED_BY_A_BENEFIC",
    ),
    (
        "TWO_IN_FRIENDS_SIGNS",
        "PARIJATA_DHANA_TWO_IN_FRIENDS_SIGNS",
    ),
    (
        "THREE_IN_FRIENDS_SIGNS",
        "PARIJATA_DHANA_THREE_IN_FRIENDS_SIGNS",
    ),
    (
        "FOUR_IN_FRIENDS_SIGNS",
        "PARIJATA_DHANA_FOUR_IN_FRIENDS_SIGNS",
    ),
    (
        "FIVE_IN_FRIENDS_SIGNS",
        "PARIJATA_RAJA_FIVE_IN_FRIENDS_SIGNS",
    ),
    ("SIX_IN_FRIENDS_SIGNS", "PARIJATA_RAJA_SIX_IN_FRIENDS_SIGNS"),
    (
        "SEVEN_IN_FRIENDS_SIGNS",
        "PARIJATA_RAJA_SEVEN_IN_FRIENDS_SIGNS",
    ),
    (
        "FIVE_IN_ENEMIES_SIGNS",
        "PARIJATA_YOGA_FIVE_IN_ENEMIES_SIGNS",
    ),
    (
        "SIX_IN_ENEMIES_SIGNS",
        "PARIJATA_DARIDRA_SIX_IN_ENEMIES_SIGNS",
    ),
    (
        "SEVEN_IN_ENEMIES_SIGNS",
        "PARIJATA_DARIDRA_SEVEN_IN_ENEMIES_SIGNS",
    ),
];

#[test]
fn the_yogas_answer_and_their_twins_agree() {
    let rules = chapters(&CHAPTERS);
    assert_eq!(rules.len(), 162);
    for rule in &rules {
        assert!(rule.is_evaluable(), "{} is evaluable", rule.key);
        assert!(rule.source.verse.is_some(), "{}: cites a verse", rule.key);
        assert!(rule.key.starts_with("SARAVALI_"), "{}", rule.key);
    }
    // Ch. 20 v. 37 asks for a raja yoga, naming ch. 35's by key, so the set
    // they are asked in holds that chapter too.
    let mut set = rules.clone();
    set.extend(chapters(&["35"]));
    teistro_rules::check_references(&set).expect("the chapters name ch. 35 soundly");
    let find = |key: &str| {
        shipped::nabhasas()
            .iter()
            .find(|rule| rule.key == key)
            .cloned()
            .expect("the twin ships")
    };
    let twins: Vec<(Rule, Rule)> = TWINS
        .iter()
        .map(|(ours, theirs)| (find(&format!("SARAVALI_{ours}")), find(theirs)))
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
            .with_rules(&set);
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

/// What each answers over the 93 recorded charts. A placeholder: the
/// integrator fills it from the evaluator's own run.
const ANSWERED: [(&str, usize); 0] = [];
