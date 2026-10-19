//! Brihat Jataka ch. 21's grahas in the vargas, held to the 93 recorded
//! charts: what each answers, and that the counts the chapter states as
//! Jataka Parijata ch. 8 does answer what that text's do.

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
            rule.source.text == "Brihat Jataka" && rule.source.chapter.as_deref() == Some("21")
        })
        .cloned()
        .collect()
}

/// The counts the chapter states as another text does, each with that
/// text's: v. 1's grahas in friends' signs are Jataka Parijata ch. 8 v. 109's,
/// count for count.
const TWINS: [(&str, &str); 7] = [
    (
        "ONE_IN_A_FRIENDS_SIGN",
        "PARIJATA_DARIDRA_ONE_IN_FRIENDS_SIGNS",
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
];

#[test]
fn the_vargas_of_the_chapter_answer_and_their_twins_agree() {
    let rules = chapter();
    assert_eq!(rules.len(), 73);
    for rule in &rules {
        assert!(rule.is_evaluable(), "{} is evaluable", rule.key);
        assert!(rule.source.verse.is_some(), "{}: cites a verse", rule.key);
    }
    teistro_rules::check_references(&rules).expect("the chapter names itself soundly");
    let find = |key: &str| {
        shipped::nabhasas()
            .iter()
            .find(|rule| rule.key == key)
            .cloned()
            .expect("the twin ships")
    };
    let twins: Vec<(Rule, Rule)> = TWINS
        .iter()
        .map(|(ours, theirs)| (find(&format!("BJ_VARGA_{ours}")), find(theirs)))
        .collect();
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

/// What each answers over the 93 recorded charts; the figures that hold on
/// none of them are measured on the computed sky and its fixtures instead
/// (`rule-cofiring-measured.md`).
const ANSWERED: [(&str, usize); 73] = [
    ("BJ_VARGA_AQUARIUS_DWADASHAMSHA_RISING", 9),
    ("BJ_VARGA_AQUARIUS_NAVAMSHA_RISING", 13),
    ("BJ_VARGA_AQUARIUS_NAVAMSHA_RISING_VARGOTTAMA", 0),
    ("BJ_VARGA_AQUARIUS_RISING", 10),
    ("BJ_VARGA_ARIES_NAVAMSHA_RISING", 4),
    ("BJ_VARGA_ARIES_NAVAMSHA_RISING_VARGOTTAMA", 0),
    ("BJ_VARGA_BENEFICS_IN_LUNAR_HORAS_NOT_ALL_IN_EVEN_SIGNS", 11),
    ("BJ_VARGA_BENEFICS_IN_LUNAR_HORAS_OF_EVEN_SIGNS", 5),
    ("BJ_VARGA_CANCER_NAVAMSHA_RISING", 2),
    ("BJ_VARGA_CANCER_NAVAMSHA_RISING_VARGOTTAMA", 0),
    ("BJ_VARGA_CAPRICORN_NAVAMSHA_RISING", 7),
    ("BJ_VARGA_CAPRICORN_NAVAMSHA_RISING_VARGOTTAMA", 0),
    ("BJ_VARGA_FIVE_IN_ENEMIES_SIGNS_OR_DEBILITATED", 0),
    ("BJ_VARGA_FIVE_IN_FRIENDS_SIGNS", 1),
    ("BJ_VARGA_FIVE_IN_OWN_HOUSES", 0),
    ("BJ_VARGA_FOUR_IN_ENEMIES_SIGNS_OR_DEBILITATED", 5),
    ("BJ_VARGA_FOUR_IN_FRIENDS_SIGNS", 14),
    ("BJ_VARGA_FOUR_IN_OWN_HOUSES", 0),
    ("BJ_VARGA_GEMINI_NAVAMSHA_RISING", 8),
    ("BJ_VARGA_GEMINI_NAVAMSHA_RISING_VARGOTTAMA", 0),
    ("BJ_VARGA_JUPITER_IN_HIS_OWN_TRIMSHAMSHA", 25),
    ("BJ_VARGA_LEO_NAVAMSHA_RISING", 7),
    ("BJ_VARGA_LEO_NAVAMSHA_RISING_VARGOTTAMA", 1),
    ("BJ_VARGA_LIBRA_NAVAMSHA_RISING", 7),
    ("BJ_VARGA_LIBRA_NAVAMSHA_RISING_VARGOTTAMA", 0),
    ("BJ_VARGA_MALEFICS_IN_SOLAR_HORAS_NOT_ALL_IN_ODD_SIGNS", 2),
    ("BJ_VARGA_MALEFICS_IN_SOLAR_HORAS_OF_ODD_SIGNS", 0),
    ("BJ_VARGA_MARS_IN_HIS_OWN_TRIMSHAMSHA", 27),
    ("BJ_VARGA_MERCURY_IN_HIS_OWN_TRIMSHAMSHA", 34),
    ("BJ_VARGA_MOON_IN_A_BIRD_DREKKANA", 3),
    ("BJ_VARGA_MOON_IN_A_QUADRUPED_DREKKANA", 12),
    ("BJ_VARGA_MOON_IN_A_SERPENT_DREKKANA", 9),
    ("BJ_VARGA_MOON_IN_A_TRIMSHAMSHA_OF_JUPITER", 32),
    ("BJ_VARGA_MOON_IN_A_TRIMSHAMSHA_OF_MARS", 4),
    ("BJ_VARGA_MOON_IN_A_TRIMSHAMSHA_OF_MERCURY", 32),
    ("BJ_VARGA_MOON_IN_A_TRIMSHAMSHA_OF_SATURN", 14),
    ("BJ_VARGA_MOON_IN_A_TRIMSHAMSHA_OF_VENUS", 11),
    ("BJ_VARGA_MOON_IN_A_WEAPON_DREKKANA", 5),
    ("BJ_VARGA_MOON_IN_HER_OWN_OR_A_FRIENDS_DREKKANA", 40),
    ("BJ_VARGA_ONE_EXALTED_ASPECTED_BY_A_FRIEND", 8),
    ("BJ_VARGA_ONE_EXALTED_WITH_A_FRIEND", 16),
    ("BJ_VARGA_ONE_IN_AN_ENEMYS_SIGN_OR_DEBILITATED", 33),
    ("BJ_VARGA_ONE_IN_A_FRIENDS_SIGN", 25),
    ("BJ_VARGA_ONE_IN_OWN_HOUSE", 42),
    ("BJ_VARGA_PISCES_NAVAMSHA_RISING", 4),
    ("BJ_VARGA_PISCES_NAVAMSHA_RISING_VARGOTTAMA", 0),
    ("BJ_VARGA_SAGITTARIUS_NAVAMSHA_RISING", 12),
    ("BJ_VARGA_SAGITTARIUS_NAVAMSHA_RISING_VARGOTTAMA", 0),
    ("BJ_VARGA_SATURN_IN_HIS_OWN_TRIMSHAMSHA", 6),
    ("BJ_VARGA_SCORPIO_NAVAMSHA_RISING", 8),
    ("BJ_VARGA_SCORPIO_NAVAMSHA_RISING_VARGOTTAMA", 0),
    ("BJ_VARGA_SEVEN_IN_ENEMIES_SIGNS_OR_DEBILITATED", 0),
    ("BJ_VARGA_SEVEN_IN_FRIENDS_SIGNS", 1),
    ("BJ_VARGA_SEVEN_IN_OWN_HOUSES", 0),
    ("BJ_VARGA_SIX_IN_ENEMIES_SIGNS_OR_DEBILITATED", 0),
    ("BJ_VARGA_SIX_IN_FRIENDS_SIGNS", 0),
    ("BJ_VARGA_SIX_IN_OWN_HOUSES", 0),
    ("BJ_VARGA_SUN_IN_A_TRIMSHAMSHA_OF_JUPITER", 24),
    ("BJ_VARGA_SUN_IN_A_TRIMSHAMSHA_OF_MARS", 25),
    ("BJ_VARGA_SUN_IN_A_TRIMSHAMSHA_OF_MERCURY", 11),
    ("BJ_VARGA_SUN_IN_A_TRIMSHAMSHA_OF_SATURN", 13),
    ("BJ_VARGA_SUN_IN_A_TRIMSHAMSHA_OF_VENUS", 20),
    ("BJ_VARGA_TAURUS_NAVAMSHA_RISING", 5),
    ("BJ_VARGA_TAURUS_NAVAMSHA_RISING_VARGOTTAMA", 1),
    ("BJ_VARGA_THREE_IN_ENEMIES_SIGNS_OR_DEBILITATED", 14),
    ("BJ_VARGA_THREE_IN_FRIENDS_SIGNS", 17),
    ("BJ_VARGA_THREE_IN_OWN_HOUSES", 3),
    ("BJ_VARGA_TWO_IN_ENEMIES_SIGNS_OR_DEBILITATED", 34),
    ("BJ_VARGA_TWO_IN_FRIENDS_SIGNS", 29),
    ("BJ_VARGA_TWO_IN_OWN_HOUSES", 28),
    ("BJ_VARGA_VENUS_IN_HIS_OWN_TRIMSHAMSHA", 9),
    ("BJ_VARGA_VIRGO_NAVAMSHA_RISING", 14),
    ("BJ_VARGA_VIRGO_NAVAMSHA_RISING_VARGOTTAMA", 0),
];
