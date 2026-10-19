//! Jataka Parijata ch. 3, manifold births, ch. 8's counts of dignified
//! grahas and ch. 9's ganda nakshatras and evil limbs of the day, held to the
//! 93 recorded charts: what each answers, and that those the chapters state
//! as BPHS does answer what BPHS's do.

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

/// The pack of ch. 3's later verses, which `parijata_late.rs` holds apart.
const LATE: &str = include_str!("../rules/classical-parijata-late.json");

/// Every rule the three chapters ship in this pack.
fn verses() -> Vec<Rule> {
    shipped::nabhasas()
        .iter()
        .filter(|rule| {
            rule.source.text == "Jataka Parijata"
                && matches!(rule.source.chapter.as_deref(), Some("3" | "8" | "9"))
                && !LATE.contains(&format!("\"{}\"", rule.key))
        })
        .cloned()
        .collect()
}

/// The figures ch. 9 states as BPHS chs. 85 to 87 do, each with BPHS's, and
/// the figures of ch. 3 vv. 40 and 46 and ch. 8 v. 111 that Brihat Jataka
/// ch. 5 states alike, each read from its own text to the same conditions.
const TWINS: [(&str, &str); 11] = [
    (
        "KRISHNA_CHATURDASHI_PART_1",
        "BPHS_KRISHNA_CHATURDASHI_PART_1",
    ),
    (
        "KRISHNA_CHATURDASHI_PART_2",
        "BPHS_KRISHNA_CHATURDASHI_PART_2",
    ),
    (
        "KRISHNA_CHATURDASHI_PART_3",
        "BPHS_KRISHNA_CHATURDASHI_PART_3",
    ),
    (
        "KRISHNA_CHATURDASHI_PART_4",
        "BPHS_KRISHNA_CHATURDASHI_PART_4",
    ),
    (
        "KRISHNA_CHATURDASHI_PART_5",
        "BPHS_KRISHNA_CHATURDASHI_PART_5",
    ),
    (
        "KRISHNA_CHATURDASHI_PART_6",
        "BPHS_KRISHNA_CHATURDASHI_PART_6",
    ),
    ("BIRTH_ON_AMAVASYA", "BPHS_BIRTH_ON_AMAVASYA"),
    ("BIRTH_IN_VISHTI", "BPHS_BIRTH_IN_BHADRA"),
    (
        "JANMA_CORD_ARIES_LEO_OR_TAURUS_RISING_WITH_SATURN_OR_MARS",
        "BJ_JANMA_ARIES_LEO_OR_TAURUS_RISING_WITH_SATURN_OR_MARS_THE_CORD",
    ),
    (
        "JANMA_FATHER_AWAY_SUN_EIGHTH_OR_NINTH_IN_A_MOVABLE_SIGN",
        "BJ_JANMA_FATHER_ABROAD_SUN_EIGHTH_OR_NINTH_IN_A_MOVABLE_SIGN",
    ),
    (
        "YOGA_THREE_DEBILITATED",
        "BJ_JANMA_THREE_DEBILITATED_BORN_ON_THE_GROUND",
    ),
];

#[test]
fn the_birth_count_and_ganda_verses_answer_and_their_twins_agree() {
    let rules = verses();
    assert_eq!(rules.len(), 169);
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

/// What each answers over the 93 recorded charts: to be filled from a run.
const ANSWERED: [(&str, usize); 169] = [
    ("PARIJATA_ABHUKTA_MULA", 0),
    ("PARIJATA_ASHLESHA_PADA_2", 1),
    ("PARIJATA_ASHLESHA_PADA_3", 0),
    ("PARIJATA_ASHLESHA_PADA_4", 0),
    (
        "PARIJATA_BIRDS_PAKSHI_DREKKANA_OR_MOVABLE_DWADASHAMSHA_RISING",
        1,
    ),
    ("PARIJATA_BIRTH_IN_GANDA", 0),
    ("PARIJATA_BIRTH_IN_PARIGHA", 2),
    ("PARIJATA_BIRTH_IN_SHOOLA", 1),
    ("PARIJATA_BIRTH_IN_VAIDHRITI", 1),
    ("PARIJATA_BIRTH_IN_VISHKAMBHA", 5),
    ("PARIJATA_BIRTH_IN_VISHTI", 9),
    ("PARIJATA_BIRTH_IN_VYATIPATA", 9),
    ("PARIJATA_BIRTH_ON_AMAVASYA", 2),
    ("PARIJATA_BIRTH_STAR_ASSAILED_BY_A_MALEFIC", 13),
    ("PARIJATA_DARIDRA_ONE_IN_FRIENDS_SIGNS", 25),
    ("PARIJATA_DARIDRA_SEVEN_IN_ENEMIES_SIGNS", 0),
    ("PARIJATA_DARIDRA_SIX_IN_ENEMIES_SIGNS", 0),
    ("PARIJATA_DARIDRA_THREE_COMBUST", 3),
    ("PARIJATA_DHANA_FOUR_IN_FRIENDS_SIGNS", 14),
    ("PARIJATA_DHANA_FOUR_IN_OWN_SIGNS", 0),
    ("PARIJATA_DHANA_ONE_EXALTED_AND_STRONG", 14),
    ("PARIJATA_DHANA_THREE_IN_FRIENDS_SIGNS", 17),
    ("PARIJATA_DHANA_THREE_IN_OWN_SIGNS", 2),
    ("PARIJATA_DHANA_TWO_IN_FRIENDS_SIGNS", 29),
    ("PARIJATA_GANDANTA_A_YAMA_AT_THE_THREE_JUNCTIONS", 0),
    ("PARIJATA_GANDA_BY_DAY", 0),
    ("PARIJATA_GANDA_BY_NIGHT", 0),
    ("PARIJATA_GARBHA_BENEFIC_IN_OR_ASPECTING_THE_LAGNA_SON", 43),
    (
        "PARIJATA_GARBHA_EUNUCH_MARS_ASPECTS_THE_SUN_IN_AN_EVEN_SIGN",
        0,
    ),
    ("PARIJATA_GARBHA_EUNUCH_MERCURY_AND_SATURN_FACING", 4),
    (
        "PARIJATA_GARBHA_EUNUCH_MOON_AND_LAGNA_ODD_ASPECTED_BY_MARS",
        0,
    ),
    (
        "PARIJATA_GARBHA_EUNUCH_MOON_EVEN_MERCURY_ODD_ASPECTED_BY_MARS",
        0,
    ),
    ("PARIJATA_GARBHA_EUNUCH_MOON_FACING_THE_SUN_OR_SATURN", 7),
    ("PARIJATA_GARBHA_EUNUCH_SUN_AND_MOON_FACING", 3),
    (
        "PARIJATA_GARBHA_EUNUCH_VENUS_MOON_LAGNA_IN_ODD_NAVAMSHAS",
        23,
    ),
    (
        "PARIJATA_GARBHA_FEMALE_HERMAPHRODITE_MERCURY_SIXTH_SIXTH_LORD_RISING",
        0,
    ),
    ("PARIJATA_GARBHA_FEMALE_LAGNA_SUN_JUPITER_MOON_EVEN", 1),
    ("PARIJATA_GARBHA_FEMALE_MOON_VENUS_MARS_EVEN", 19),
    (
        "PARIJATA_GARBHA_MALE_HERMAPHRODITE_SATURN_WITH_MERCURY_SIXTH",
        0,
    ),
    ("PARIJATA_GARBHA_MALE_JUPITER_AND_SUN_ODD", 25),
    ("PARIJATA_GARBHA_MALE_LAGNA_SUN_JUPITER_MOON_ODD", 0),
    ("PARIJATA_GARBHA_MALE_SATURN_IN_AN_ODD_HOUSE", 58),
    ("PARIJATA_GARBHA_MANY_LAGNA_IN_SAGITTARIUS_LAST_NAVAMSHA", 0),
    ("PARIJATA_GARBHA_SUN_THIRD_OR_IN_A_TRIKONA_SON", 14),
    ("PARIJATA_GARBHA_TRIPLETS_THREE_BOYS", 0),
    ("PARIJATA_GARBHA_TRIPLETS_THREE_GIRLS", 0),
    ("PARIJATA_GARBHA_TRIPLETS_TWO_BOYS_AND_A_GIRL", 0),
    ("PARIJATA_GARBHA_TRIPLETS_TWO_GIRLS_AND_A_BOY", 0),
    (
        "PARIJATA_GARBHA_TWINS_LAGNA_AND_MOON_EVEN_ASPECTED_BY_A_MALE_GRAHA",
        10,
    ),
    ("PARIJATA_GARBHA_TWINS_LAGNA_LORD_THIRD_OR_EXALTED", 24),
    ("PARIJATA_GARBHA_TWINS_LORDS_OF_ONE_AND_THREE_RISING", 0),
    ("PARIJATA_GARBHA_TWINS_LORDS_OF_ONE_AND_THREE_TOGETHER", 8),
    (
        "PARIJATA_GARBHA_TWINS_MERCURY_MARS_JUPITER_LAGNA_STRONG_IN_EVEN_SIGNS",
        3,
    ),
    (
        "PARIJATA_GARBHA_TWINS_MOON_VENUS_EVEN_MARS_MERCURY_JUPITER_LAGNA_ODD",
        2,
    ),
    (
        "PARIJATA_GARBHA_TWIN_BOYS_FIVE_IN_GEMINI_OR_SAGITTARIUS_NAVAMSHAS",
        0,
    ),
    ("PARIJATA_GARBHA_TWIN_BOYS_SUN_AND_JUPITER_IN_DUAL_SIGNS", 5),
    (
        "PARIJATA_GARBHA_TWIN_BOY_AND_GIRL_FIVE_IN_DUAL_NAVAMSHAS_OF_BOTH_KINDS",
        0,
    ),
    (
        "PARIJATA_GARBHA_TWIN_GIRLS_FIVE_IN_VIRGO_OR_PISCES_NAVAMSHAS",
        0,
    ),
    (
        "PARIJATA_GARBHA_TWIN_GIRLS_MOON_VENUS_MARS_IN_DUAL_SIGNS",
        1,
    ),
    ("PARIJATA_GIRL_IN_JYESHTHA_ON_A_TUESDAY", 0),
    ("PARIJATA_GIRL_IN_MULA_ON_A_SUNDAY", 1),
    (
        "PARIJATA_JANMA_AURASA_LAGNA_OR_MOON_BENEFIC_SIGN_JUPITER_NAVAMSHA",
        14,
    ),
    (
        "PARIJATA_JANMA_CORD_ARIES_LEO_OR_TAURUS_RISING_WITH_SATURN_OR_MARS",
        3,
    ),
    ("PARIJATA_JANMA_CORD_EIGHTH_LORD_WITH_A_MALEFIC_RISING", 1),
    ("PARIJATA_JANMA_CORD_EIGHTH_LORD_WITH_RAHU_RISING", 1),
    (
        "PARIJATA_JANMA_CORD_LAGNA_HEMMED_BY_MALEFICS_RAHU_RISING",
        1,
    ),
    (
        "PARIJATA_JANMA_CORD_MALEFIC_LAGNA_ASPECTED_BY_MALEFICS_OR_WITH_A_NODE",
        18,
    ),
    ("PARIJATA_JANMA_CORD_MARS_RISING_ASPECTED_BY_THE_SUN", 0),
    ("PARIJATA_JANMA_CORD_SATURN_RISING_ASPECTED_BY_MARS", 0),
    (
        "PARIJATA_JANMA_DATTA_MANDI_ASPECTED_BY_THE_MOON_WITH_SATURN",
        0,
    ),
    ("PARIJATA_JANMA_FATHER_AWAY_MARS_SEVENTH", 5),
    (
        "PARIJATA_JANMA_FATHER_AWAY_MOON_BETWEEN_MERCURY_AND_VENUS",
        3,
    ),
    ("PARIJATA_JANMA_FATHER_AWAY_SATURN_RISING", 3),
    (
        "PARIJATA_JANMA_FATHER_AWAY_SUN_EIGHTH_OR_NINTH_IN_A_DUAL_SIGN",
        4,
    ),
    (
        "PARIJATA_JANMA_FATHER_AWAY_SUN_EIGHTH_OR_NINTH_IN_A_FIXED_SIGN",
        1,
    ),
    (
        "PARIJATA_JANMA_FATHER_AWAY_SUN_EIGHTH_OR_NINTH_IN_A_MOVABLE_SIGN",
        4,
    ),
    (
        "PARIJATA_JANMA_FATHER_CONFINED_SATURN_AND_MARS_FROM_THE_SUN_IN_A_DUAL_SIGN",
        0,
    ),
    (
        "PARIJATA_JANMA_FATHER_CONFINED_SATURN_AND_MARS_FROM_THE_SUN_IN_A_FIXED_SIGN",
        0,
    ),
    (
        "PARIJATA_JANMA_FATHER_CONFINED_SATURN_AND_MARS_FROM_THE_SUN_IN_A_MOVABLE_SIGN",
        0,
    ),
    ("PARIJATA_JANMA_FEET_FIRST_RAHU_RISING_LAGNA_LORD_TENTH", 1),
    ("PARIJATA_JANMA_FEET_FIRST_THIRD_LORD_AND_RAHU_RISING", 0),
    (
        "PARIJATA_JANMA_JARAJA_BHADRA_TITHI_TRIPADA_STAR_ON_SATURDAY_SUNDAY_OR_TUESDAY",
        1,
    ),
    (
        "PARIJATA_JANMA_JARAJA_JUPITER_NEITHER_SEES_NOR_JOINS_LAGNA_OR_MOON",
        31,
    ),
    (
        "PARIJATA_JANMA_JARAJA_JUPITER_SEES_NEITHER_LAGNA_NOR_MOON_APART_FROM_THE_SUN",
        28,
    ),
    ("PARIJATA_JANMA_JARAJA_LAGNA_OR_FOURTH_LORD_WITH_A_NODE", 17),
    ("PARIJATA_JANMA_JARAJA_LORDS_OF_ONE_AND_FOUR_EXCHANGED", 0),
    ("PARIJATA_JANMA_JARAJA_MOON_WITH_THE_SUN_AND_A_MALEFIC", 1),
    ("PARIJATA_JANMA_JARAJA_TITHI_VARA_AND_NAKSHATRA", 0),
    (
        "PARIJATA_JANMA_KANINA_LUMINARIES_IN_EACH_OTHERS_NAVAMSHA",
        0,
    ),
    ("PARIJATA_JANMA_KANINA_LUMINARIES_TOGETHER", 8),
    ("PARIJATA_JANMA_KANINA_SUN_ASPECTS_THE_TWELFTH", 17),
    (
        "PARIJATA_JANMA_KRITRIMA_SATURN_AND_MARS_FIFTH_OR_SEVENTH_UNASPECTED",
        1,
    ),
    (
        "PARIJATA_JANMA_KSHETRAJA_JUPITER_UNASPECTED_BY_MARS_MOON_WITH_THE_SUN",
        1,
    ),
    (
        "PARIJATA_JANMA_NOT_OF_ANOTHER_MOON_IN_JUPITERS_SIGN_OR_WITH_HIM",
        44,
    ),
    (
        "PARIJATA_JANMA_NO_SEEMANTHA_MALEFIC_IN_A_MALEFICS_ELEVENTH",
        19,
    ),
    (
        "PARIJATA_JANMA_NO_SEEMANTHA_NINTH_AND_TENTH_LORDS_IN_DUSTHANAS",
        1,
    ),
    (
        "PARIJATA_JANMA_PAUNARBHAVA_MOON_IN_SATURNS_NAVAMSHA_SATURN_FIFTH",
        0,
    ),
    (
        "PARIJATA_JANMA_SERPENT_MOON_IN_A_MALEFIC_LAGNA_OR_MARS_DREKKANA_BENEFICS_SECOND_ELEVENTH",
        0,
    ),
    (
        "PARIJATA_JANMA_SERPENT_RAHU_IN_A_KENDRA_LAGNA_IN_A_MALEFICS_DREKKANA",
        0,
    ),
    (
        "PARIJATA_JANMA_SERPENT_SARPA_PAKSHI_OR_NIGADA_DREKKANA_RISING_WITH_ITS_LORD",
        0,
    ),
    ("PARIJATA_JANMA_TWINS_IN_ONE_SAC_SUN_IN_A_QUADRUPED_SIGN", 0),
    ("PARIJATA_JYESHTHA_PADA_1", 1),
    ("PARIJATA_JYESHTHA_PADA_2", 0),
    ("PARIJATA_JYESHTHA_PADA_3", 2),
    ("PARIJATA_JYESHTHA_PADA_4", 0),
    ("PARIJATA_JYESHTHA_TENTH_1", 0),
    ("PARIJATA_JYESHTHA_TENTH_10", 0),
    ("PARIJATA_JYESHTHA_TENTH_2", 0),
    ("PARIJATA_JYESHTHA_TENTH_3", 0),
    ("PARIJATA_JYESHTHA_TENTH_4", 0),
    ("PARIJATA_JYESHTHA_TENTH_5", 0),
    ("PARIJATA_JYESHTHA_TENTH_6", 0),
    ("PARIJATA_JYESHTHA_TENTH_7", 0),
    ("PARIJATA_JYESHTHA_TENTH_8", 0),
    ("PARIJATA_JYESHTHA_TENTH_9", 0),
    ("PARIJATA_KRISHNA_CHATURDASHI_PART_1", 0),
    ("PARIJATA_KRISHNA_CHATURDASHI_PART_2", 0),
    ("PARIJATA_KRISHNA_CHATURDASHI_PART_3", 0),
    ("PARIJATA_KRISHNA_CHATURDASHI_PART_4", 0),
    ("PARIJATA_KRISHNA_CHATURDASHI_PART_5", 0),
    ("PARIJATA_KRISHNA_CHATURDASHI_PART_6", 0),
    ("PARIJATA_MULA_FIFTEENTH_1", 0),
    ("PARIJATA_MULA_FIFTEENTH_10", 0),
    ("PARIJATA_MULA_FIFTEENTH_11", 0),
    ("PARIJATA_MULA_FIFTEENTH_12", 0),
    ("PARIJATA_MULA_FIFTEENTH_13", 0),
    ("PARIJATA_MULA_FIFTEENTH_14", 0),
    ("PARIJATA_MULA_FIFTEENTH_15", 0),
    ("PARIJATA_MULA_FIFTEENTH_2", 0),
    ("PARIJATA_MULA_FIFTEENTH_3", 0),
    ("PARIJATA_MULA_FIFTEENTH_4", 0),
    ("PARIJATA_MULA_FIFTEENTH_5", 0),
    ("PARIJATA_MULA_FIFTEENTH_6", 0),
    ("PARIJATA_MULA_FIFTEENTH_7", 0),
    ("PARIJATA_MULA_FIFTEENTH_8", 0),
    ("PARIJATA_MULA_FIFTEENTH_9", 0),
    ("PARIJATA_MULA_MAGHA_ASHWINI_FIRST_PADA", 1),
    ("PARIJATA_MULA_PADA_1", 0),
    ("PARIJATA_MULA_PADA_2", 0),
    ("PARIJATA_MULA_PADA_3", 4),
    ("PARIJATA_MULA_PADA_4", 1),
    ("PARIJATA_PARENT_BY_SEX_PADAS", 5),
    ("PARIJATA_PURVA_ASHADHA_MOON_SAGITTARIUS_RISING", 0),
    ("PARIJATA_PUSHYA_MOON_CANCER_RISING", 0),
    ("PARIJATA_RAJA_FIVE_EXALTED_AND_STRONG", 0),
    ("PARIJATA_RAJA_FIVE_IN_FRIENDS_SIGNS", 1),
    ("PARIJATA_RAJA_FIVE_IN_OWN_SIGNS", 0),
    ("PARIJATA_RAJA_FOUR_EXALTED_AND_STRONG_IN_KENDRAS", 0),
    ("PARIJATA_RAJA_ONE_EXALTED_ASPECTED_BY_A_FRIEND", 8),
    ("PARIJATA_RAJA_SEVEN_IN_FRIENDS_SIGNS", 1),
    ("PARIJATA_RAJA_SIX_IN_FRIENDS_SIGNS", 0),
    ("PARIJATA_RAJA_SIX_IN_OWN_SIGNS", 0),
    ("PARIJATA_RAJA_THREE_EXALTED", 0),
    ("PARIJATA_RAJA_THREE_EXALTED_AND_STRONG", 0),
    ("PARIJATA_RAJA_TWO_EXALTED_AND_STRONG", 4),
    ("PARIJATA_REVATI_JYESHTHA_ASHLESHA_LAST_PADA", 1),
    ("PARIJATA_RIKTA_TITHI", 14),
    ("PARIJATA_VIYONI", 0),
    ("PARIJATA_VIYONI_LAGNA_ASPECTED_BY_VENUS_COW", 0),
    (
        "PARIJATA_VIYONI_LAGNA_WITH_OR_ASPECTED_BY_SATURN_BUFFALO",
        0,
    ),
    ("PARIJATA_VIYONI_LORDS_OF_ONE_AND_FOUR_EXCHANGED", 0),
    ("PARIJATA_VIYONI_LORDS_OF_ONE_AND_FOUR_WITH_A_NODE", 5),
    ("PARIJATA_VIYONI_MALEFICS_RISING_OTHER_KIND", 0),
    ("PARIJATA_VIYONI_NODE_RISING_SHEEP_OR_GOAT", 0),
    ("PARIJATA_YOGA_FIVE_IN_ENEMIES_SIGNS", 0),
    ("PARIJATA_YOGA_ONE_IN_OWN_SIGNS", 41),
    ("PARIJATA_YOGA_THREE_DEBILITATED", 2),
    ("PARIJATA_YOGA_THREE_IN_OWN_SIGNS_MINISTER", 2),
    ("PARIJATA_YOGA_TWO_IN_OWN_SIGNS", 30),
];
