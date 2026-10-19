//! Jataka Parijata ch. 7's raja yogas and their kin, vv. 1 to 127, held to
//! the 93 recorded charts: what each answers, and that those the chapter
//! states as another text does answer what that text's do.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they read"
)]

mod common;

use std::collections::BTreeMap;

use common::{chart_at, files_in};
use teistro_rules::{Evaluator, Readings, Rule, shipped};

/// Every rule the verses ship: ch. 7 up to v. 127.
fn verses() -> Vec<Rule> {
    shipped::nabhasas()
        .iter()
        .filter(|rule| {
            rule.source.text == "Jataka Parijata"
                && rule.source.chapter.as_deref() == Some("7")
                && first_verse(rule) < 128
        })
        .cloned()
        .collect()
}

/// The first verse a rule cites.
fn first_verse(rule: &Rule) -> u32 {
    let verse = rule.source.verse.as_deref().unwrap_or_default();
    let digits: String = verse.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().expect("a verse number")
}

/// The yogas the chapter states as another text does, each with that text's.
const TWINS: [(&str, &str); 8] = [
    (
        "RAJA_VARGOTTAMA_LAGNA_ASPECTED_BY_FOUR",
        "BJ_RAJA_VARGOTTAMA_LAGNA_ASPECTED_BY_FOUR",
    ),
    (
        "RAJA_VARGOTTAMA_MOON_ASPECTED_BY_FOUR",
        "BJ_RAJA_VARGOTTAMA_MOON_ASPECTED_BY_FOUR",
    ),
    (
        "RAJA_VENUS_IN_ASHWINI_RISING",
        "PHALADEEPIKA_MAHARAJA_VENUS_IN_ASHVINI_RISING",
    ),
    ("RAJA_SIX_EXALTED", "BPHS_SIX_GRAHAS_EXALTED"),
    (
        "RAJA_JUPITER_SECOND_WITH_VENUS",
        "SARAVALI_RAJA_JUPITER_SECOND_WITH_VENUS",
    ),
    ("AMALA", "PHALADEEPIKA_AMALA"),
    ("SHUBHA_KARTARI", "PHALADEEPIKA_SHUBHA_KARTARI"),
    ("PAPA_KARTARI", "PHALADEEPIKA_PAPA_KARTARI"),
];

#[test]
fn the_raja_yogas_answer_and_their_twins_agree() {
    let rules = verses();
    assert_eq!(rules.len(), 106);
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
    let mut fired: BTreeMap<&str, usize> =
        rules.iter().map(|rule| (rule.key.as_str(), 0)).collect();
    for (path, file) in files_in("doshas") {
        let chart = chart_at(&path, &file["inputs"]);
        let evaluator = Evaluator::new(&chart, Readings::TEXTS).with_rules(&rules);
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

/// What each answers over the 93 recorded charts.
const ANSWERED: [(&str, usize); 106] = [
    ("PARIJATA_AMALA", 14),
    ("PARIJATA_ASHUBHA", 25),
    ("PARIJATA_BHASKARA", 1),
    ("PARIJATA_BUDHA", 0),
    ("PARIJATA_CHANDRADHI", 6),
    (
        "PARIJATA_DARIDRA_MOON_DEBILITATED_OR_HOSTILE_IN_A_GOOD_HOUSE_JUPITER_HIDDEN_FROM_HER",
        11,
    ),
    (
        "PARIJATA_DARIDRA_MOON_IN_LIBRA_IN_AN_ENEMY_SIGN_ASPECTED_BY_A_DEBILITATED_GRAHA",
        0,
    ),
    (
        "PARIJATA_DARIDRA_MOON_MOVABLE_IN_A_MALEFIC_NAVAMSHA_UNASPECTED_BY_JUPITER",
        9,
    ),
    (
        "PARIJATA_DARIDRA_MOON_WITH_A_NODE_ASPECTED_BY_A_MALEFIC",
        24,
    ),
    (
        "PARIJATA_DARIDRA_MOON_WITH_THE_SUN_IN_A_MALEFIC_NAVAMSHA_ASPECTED_BY_A_DEBILITATED_GRAHA",
        0,
    ),
    (
        "PARIJATA_DARIDRA_NIGHT_AFFLICTED_WEAK_MOON_ASPECTED_BY_THE_TENTH_LORD",
        3,
    ),
    (
        "PARIJATA_DARIDRA_NIGHT_MALEFIC_ON_THE_EIGHTH_FROM_THE_WANING_MOON",
        26,
    ),
    ("PARIJATA_DARIDRA_NIGHT_WANING_MOON_DEBILITATED", 8),
    (
        "PARIJATA_DARIDRA_SATURN_AND_VENUS_FALLEN_TOGETHER_OR_FACING",
        14,
    ),
    ("PARIJATA_GAJAKESARI", 25),
    ("PARIJATA_INDRA", 0),
    ("PARIJATA_KEMADRUMA", 14),
    (
        "PARIJATA_KEMADRUMA_MOON_IN_THE_FIRST_OR_SEVENTH_UNASPECTED_BY_JUPITER",
        12,
    ),
    ("PARIJATA_LAGNADHI", 0),
    ("PARIJATA_MARUT", 0),
    (
        "PARIJATA_MOON_IN_OWN_OR_A_GREAT_FRIENDS_NAVAMSHA_ASPECTED_BY_JUPITER_BY_DAY_VENUS_BY_NIGHT",
        0,
    ),
    ("PARIJATA_PAPA_KARTARI", 17),
    (
        "PARIJATA_RAJA_BENEFICS_IN_THE_TWELFTH_ELEVENTH_FIRST_SECOND_THIRD_TENTH",
        3,
    ),
    (
        "PARIJATA_RAJA_BENEFICS_IN_UPACHAYAS_MALEFICS_FIRST_OR_TENTH",
        0,
    ),
    (
        "PARIJATA_RAJA_BENEFICS_NINTH_AND_ELEVENTH_MALEFICS_SIXTH_AND_TENTH",
        0,
    ),
    ("PARIJATA_RAJA_COMBUST_MERCURY_AT_HOME", 3),
    (
        "PARIJATA_RAJA_DEBILITATED_GRAHAS_NAVAMSHA_LORD_WELL_PLACED_MOVABLE_LAGNA",
        11,
    ),
    (
        "PARIJATA_RAJA_DEBILITATED_GRAHAS_SIGN_OR_EXALTATION_LORD_IN_A_KENDRA",
        48,
    ),
    ("PARIJATA_RAJA_DIGNIFIED_LAGNA_LORD_ASPECTS_THE_MOON", 3),
    (
        "PARIJATA_RAJA_DIGNIFIED_SUN_THIRD_VENUS_SIXTH_MERCURY_EIGHTH_FROM_THE_MOON",
        0,
    ),
    (
        "PARIJATA_RAJA_EXALTED_MARS_ASPECTED_BY_SUN_MOON_AND_JUPITER",
        0,
    ),
    (
        "PARIJATA_RAJA_EXALTED_STRONG_LAGNA_LORD_ASPECTED_BY_THE_MOON",
        0,
    ),
    ("PARIJATA_RAJA_FIVE_EXALTED_JUPITER_RISING", 0),
    ("PARIJATA_RAJA_FIVE_IN_OWN_NAVAMSHAS_ONE_RISING", 0),
    (
        "PARIJATA_RAJA_FULL_MOON_IN_A_KENDRA_ASPECTED_BY_VENUS_AND_JUPITER",
        0,
    ),
    (
        "PARIJATA_RAJA_FULL_MOON_JOINED_ASPECTED_BY_VENUS_MERCURY_JUPITER",
        0,
    ),
    ("PARIJATA_RAJA_FULL_MOON_NINTH_SATURN_TENTH_MARS_SECOND", 0),
    (
        "PARIJATA_RAJA_FULL_MOON_RISING_IN_PISCES_MARS_EXALTED_SATURN_AQUARIUS",
        0,
    ),
    ("PARIJATA_RAJA_FULL_MOON_STRONG", 5),
    (
        "PARIJATA_RAJA_FULL_MOON_TENTH_WITH_OR_ASPECTED_BY_A_BENEFIC",
        0,
    ),
    (
        "PARIJATA_RAJA_JUPITER_AND_MOON_IN_TAURUS_LAGNA_LORD_IN_A_TRIKONA",
        1,
    ),
    ("PARIJATA_RAJA_JUPITER_FIFTH_MOON_THIRD_SUN_NINTH", 0),
    (
        "PARIJATA_RAJA_JUPITER_LAGNA_LORD_TWELFTH_SATURN_OR_SUN_BEYOND",
        0,
    ),
    (
        "PARIJATA_RAJA_JUPITER_RISING_MERCURY_IN_A_KENDRA_ASPECTED_BY_THE_NINTH_LORD",
        1,
    ),
    ("PARIJATA_RAJA_JUPITER_SECOND_WITH_VENUS", 0),
    (
        "PARIJATA_RAJA_JUPITER_VENUS_MARS_VARGOTTAMA_NO_MALEFIC_IN_A_KENDRA",
        0,
    ),
    ("PARIJATA_RAJA_JUPITER_WITH_MARS_AND_THE_MOON_VARGOTTAMA", 0),
    ("PARIJATA_RAJA_LAGNA_LORD_EXALTED_IN_A_KENDRA_ALONE", 0),
    (
        "PARIJATA_RAJA_LAGNA_LORD_IN_AN_UPACHAYA_FROM_THE_MOON_BENEFICS_IN_KENDRAS_MALEFICS_WEAK",
        0,
    ),
    (
        "PARIJATA_RAJA_LAGNA_LORD_IN_AN_UPACHAYA_MOON_NINTH_STRONG_BENEFIC_IN_A_KENDRA",
        1,
    ),
    (
        "PARIJATA_RAJA_LAGNA_LORD_IN_A_KENDRA_TENTH_LORD_FOURTH_NINTH_LORD_ELEVENTH",
        2,
    ),
    ("PARIJATA_RAJA_LAGNA_LORD_IN_A_KENDRA_UNAFFLICTED_ALONE", 23),
    ("PARIJATA_RAJA_LAGNA_LORD_NINTH_OR_TENTH_MOON_RISING", 0),
    ("PARIJATA_RAJA_LIBRA_FIGURE", 0),
    (
        "PARIJATA_RAJA_LUMINARIES_EARLY_SAGITTARIUS_STRONG_SATURN_RISING_MARS_EXALTED",
        0,
    ),
    (
        "PARIJATA_RAJA_MALEFICS_THIRD_SIXTH_ELEVENTH_LAGNA_LORD_ASPECTED_BY_A_BENEFIC",
        0,
    ),
    ("PARIJATA_RAJA_MARS_CAPRICORN_SUN_ARIES_JUPITER_AQUARIUS", 0),
    ("PARIJATA_RAJA_MARS_RISING_IN_CAPRICORN_MOON_IN_CANCER", 0),
    ("PARIJATA_RAJA_MARS_RISING_IN_FIRE_ASPECTED_BY_A_FRIEND", 0),
    ("PARIJATA_RAJA_MERCURY_JOINED_OR_ASPECTED_BY_JUPITER", 29),
    ("PARIJATA_RAJA_MERCURY_RISING_IN_ARIES_JUPITER_EXALTED", 0),
    (
        "PARIJATA_RAJA_MERCURY_RISING_JUPITER_SEVENTH_BRIGHT_MOON_FOURTH_IN_CANCER_VENUS_TENTH",
        0,
    ),
    (
        "PARIJATA_RAJA_MERCURY_VIRGO_MOON_PISCES_MARS_GEMINI_JUPITER_SAGITTARIUS_SATURN_CAPRICORN",
        0,
    ),
    ("PARIJATA_RAJA_MOON_RISING_IN_CANCER_SUN_IN_PISCES", 0),
    (
        "PARIJATA_RAJA_MOON_RISING_JUPITER_FOURTH_VENUS_TENTH_SATURN_DIGNIFIED",
        0,
    ),
    ("PARIJATA_RAJA_MOON_WITH_SATURN_ELEVENTH_FOURTH_OR_TENTH", 1),
    (
        "PARIJATA_RAJA_NIGHT_MOON_IN_A_GREAT_FRIENDS_NAVAMSHA_ASPECTED_BY_VENUS_ALONE",
        0,
    ),
    (
        "PARIJATA_RAJA_NINTH_LORDS_NAVAMSHA_LORD_FIFTH_OR_FOURTH",
        16,
    ),
    (
        "PARIJATA_RAJA_ONE_EACH_IN_THE_FIRST_SECOND_THIRD_FOURTH_FIFTH_SEVENTH_NINTH",
        0,
    ),
    (
        "PARIJATA_RAJA_ONE_EXALTED_THE_REST_AT_HOME_OR_WITH_FRIENDS",
        0,
    ),
    (
        "PARIJATA_RAJA_ONE_LORD_OF_GAIN_IN_A_KENDRA_FROM_THE_MOON_JUPITER_ELEVENTH_LORD",
        11,
    ),
    (
        "PARIJATA_RAJA_RAHU_TENTH_SATURN_ELEVENTH_ASPECTED_BY_THE_NINTH_LORD",
        0,
    ),
    (
        "PARIJATA_RAJA_SATURN_AND_MARS_TENTH_FIFTH_OR_FIRST_FULL_MOON_IN_JUPITERS_SIGN",
        0,
    ),
    ("PARIJATA_RAJA_SATURN_RISING_IN_AQUARIUS_FOUR_EXALTED", 0),
    (
        "PARIJATA_RAJA_SATURN_RISING_IN_A_SIGN_OF_JUPITER_VENUS_MARS_OR_HIS_OWN",
        1,
    ),
    ("PARIJATA_RAJA_SEVEN_IN_LIBRA_ARIES_TAURUS_PISCES", 0),
    (
        "PARIJATA_RAJA_SEVEN_IN_TAURUS_SAGITTARIUS_PISCES_OR_THE_KENDRAS",
        5,
    ),
    ("PARIJATA_RAJA_SEVEN_IN_THE_MOONS_HORA", 0),
    ("PARIJATA_RAJA_SEVEN_IN_VIRGO_ARIES_LIBRA_LEO_AQUARIUS", 0),
    (
        "PARIJATA_RAJA_SEVEN_IN_VIRGO_PISCES_GEMINI_TAURUS_LEO_SAGITTARIUS_AQUARIUS",
        9,
    ),
    ("PARIJATA_RAJA_SHIRSHODAYA_GRAHAS_MOON_RISING_IN_CANCER", 0),
    ("PARIJATA_RAJA_SIX_EXALTED", 0),
    (
        "PARIJATA_RAJA_STRONG_BENEFIC_IN_A_KENDRA_MARS_AND_SATURN_NINTH_AND_ELEVENTH",
        0,
    ),
    (
        "PARIJATA_RAJA_STRONG_DIGNIFIED_SATURN_WELL_PLACED_ASPECTED_BY_THE_ELEVENTH_LORD",
        0,
    ),
    (
        "PARIJATA_RAJA_STRONG_LAGNA_LORD_IN_A_KENDRA_ASPECTED_BY_A_FRIEND",
        4,
    ),
    (
        "PARIJATA_RAJA_STRONG_MARS_RISING_IN_CAPRICORN_SATURN_NINTH_OR_TWELFTH_LUMINARIES_SEVENTH",
        0,
    ),
    (
        "PARIJATA_RAJA_SUN_AND_MERCURY_FOURTH_SATURN_AND_MOON_TENTH_MARS_RISING",
        0,
    ),
    (
        "PARIJATA_RAJA_SUN_IN_A_TRIKONA_IN_PISCES_MOON_AND_JUPITER_IN_CANCER",
        0,
    ),
    (
        "PARIJATA_RAJA_SUN_MOON_MERCURY_VENUS_TENTH_IN_FRIENDS_NAVAMSHAS",
        0,
    ),
    ("PARIJATA_RAJA_SUN_RISING_IN_LEO_MERCURY_IN_VIRGO", 0),
    (
        "PARIJATA_RAJA_SUN_RISING_IN_SAGITTARIUS_MARS_AND_MOON_TENTH_VENUS_ELEVENTH_OR_TWELFTH",
        0,
    ),
    ("PARIJATA_RAJA_SUN_THIRD_MOON_NINTH_JUPITER_FIFTH", 0),
    ("PARIJATA_RAJA_TAURUS_MOON_RISING_ASPECTED_BY_ALL", 0),
    ("PARIJATA_RAJA_TENTH_LORD_EIGHTH_IN_A_GOOD_NAVAMSHA", 0),
    ("PARIJATA_RAJA_THREE_IN_OWN_NAVAMSHAS_ONE_RISING", 2),
    ("PARIJATA_RAJA_TWO_OR_THREE_EXALTED_MOON_IN_CANCER", 1),
    (
        "PARIJATA_RAJA_TWO_TO_FOUR_DEBILITATED_IN_EXALTATION_NAVAMSHAS",
        0,
    ),
    ("PARIJATA_RAJA_VARGOTTAMA_LAGNA_ASPECTED_BY_FOUR", 0),
    ("PARIJATA_RAJA_VARGOTTAMA_MOON_ASPECTED_BY_FOUR", 0),
    ("PARIJATA_RAJA_VENUS_IN_ASHWINI_RISING", 0),
    (
        "PARIJATA_RAJA_VENUS_RISING_IN_PISCES_IN_A_PISCES_NAVAMSHA",
        0,
    ),
    ("PARIJATA_RAJA_VENUS_SECOND_STRONG_LAGNA_LORD", 1),
    ("PARIJATA_RAJA_VENUS_STRONG_ELEVENTH_OR_TWELFTH", 11),
    ("PARIJATA_SAKATA", 3),
    ("PARIJATA_SHUBHA", 10),
    ("PARIJATA_SHUBHA_KARTARI", 2),
];
