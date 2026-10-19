//! Phaladeepika ch. 10's wife, ch. 12's children and ch. 27's ascetics, held
//! to the 93 recorded charts: what each answers, what the chapters' own
//! grades say must hold between them, and that the yoga ch. 10 states as
//! Brihat Jataka ch. 23 does answer what Varahamihira's does.

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

/// The rules of some Phaladeepika chapters.
fn chapters(numbers: &[&str]) -> Vec<Rule> {
    shipped::nabhasas()
        .iter()
        .filter(|rule| {
            rule.source.text == "Phaladeepika"
                && rule
                    .source
                    .chapter
                    .as_deref()
                    .is_some_and(|chapter| numbers.contains(&chapter))
        })
        .cloned()
        .collect()
}

/// The yogas the chapters state as another text does, each with that text's.
const TWINS: [(&str, &str); 1] = [(
    "PHALADEEPIKA_KALATRA_WANING_MOON_FIFTH_MALEFICS_TWELFTH_SEVENTH_FIRST",
    "BJ_ANISHTA_MALEFICS_TWELFTH_SEVENTH_AND_FIRST_WANING_MOON_FIFTH",
)];

/// Each pair is a figure and the same figure narrowed, so the second holds
/// only where the first does.
const WITHIN: [(&str, &str); 4] = [
    (
        "PUTRA_MALEFIC_FIFTH_LORD_AT_HOME",
        "PUTRA_MALEFIC_FIFTH_LORD_AT_HOME_WITH_ANOTHER_MALEFIC",
    ),
    (
        "PRAVRAJYA_LAGNA_LORD_ASPECTS_THE_WANING_MOON",
        "PRAVRAJYA_LAGNA_LORD_ASPECTS_THE_WANING_MOON_IN_A_RAJA_YOGA",
    ),
    (
        "PRAVRAJYA_TENTH_LORD_AMONG_FOUR_IN_ONE_SIGN",
        "PRAVRAJYA_TENTH_LORD_WITH_FOUR_IN_A_KENDRA_OR_TRIKONA",
    ),
    (
        "KALATRA_SEVENTH_OR_ITS_LORD_AFFLICTED",
        "KALATRA_AFFLICTED_SEVENTH_LORD_AND_SEVENTH",
    ),
];

#[test]
fn the_wife_the_children_and_the_ascetics_answer_and_their_twin_agrees() {
    let rules = chapters(&["10", "12", "27"]);
    assert_eq!(rules.len(), 75);
    for rule in &rules {
        assert!(rule.is_evaluable(), "{} is evaluable", rule.key);
        assert!(rule.source.verse.is_some(), "{}: cites a verse", rule.key);
    }
    // Ch. 27 v. 7's raja yoga names ch. 7's by key, so the set they are
    // asked in holds both.
    let mut set = rules.clone();
    set.extend(chapters(&["7"]));
    teistro_rules::check_references(&set).expect("the chapters name ch. 7 soundly");
    let find = |key: &str| {
        shipped::nabhasas()
            .iter()
            .find(|rule| rule.key == key)
            .cloned()
            .expect("the twin ships")
    };
    let twins: Vec<(Rule, Rule)> = TWINS
        .iter()
        .map(|(ours, theirs)| (find(ours), find(theirs)))
        .collect();
    for (ours, theirs) in &twins {
        assert_eq!(ours.conditions, theirs.conditions, "{}", ours.key);
    }
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
            .with_rules(&set);
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

/// What each answers over the 93 recorded charts, filled from the first run.
const ANSWERED: [(&str, usize); 75] = [
    (
        "PHALADEEPIKA_KALATRA_AFFLICTED_SEVENTH_LORD_AND_SEVENTH",
        39,
    ),
    (
        "PHALADEEPIKA_KALATRA_BENEFICS_SECOND_SEVENTH_ELEVENTH_FROM_THE_SEVENTH_LORD",
        0,
    ),
    (
        "PHALADEEPIKA_KALATRA_BENEFIC_SEVENTH_NOT_A_DUSTHANA_LORD",
        12,
    ),
    ("PHALADEEPIKA_KALATRA_EUNUCH_SEVENTH_TWO_IN_THE_ELEVENTH", 0),
    ("PHALADEEPIKA_KALATRA_EVEN_SEVENTH_STRONG_BRIGHT_LORDS", 7),
    ("PHALADEEPIKA_KALATRA_FIFTH_BEARS_FRUIT", 71),
    ("PHALADEEPIKA_KALATRA_FIFTH_OR_EIGHTH_LORD_SEVENTH", 13),
    (
        "PHALADEEPIKA_KALATRA_HOSTILE_MALEFIC_SEVENTH_EIGHTH_OR_SECOND",
        18,
    ),
    ("PHALADEEPIKA_KALATRA_JUPITER_SEVENTH_DEBILITATED", 0),
    (
        "PHALADEEPIKA_KALATRA_LORDS_OF_SECOND_SEVENTH_TWELFTH_ASPECTED_BY_JUPITER_IN_KENDRAS_OR_TRIKONAS",
        1,
    ),
    (
        "PHALADEEPIKA_KALATRA_MALEFICS_TWELFTH_FOURTH_EIGHTH_FROM_VENUS",
        8,
    ),
    ("PHALADEEPIKA_KALATRA_MALEFIC_SEVENTH_LORD_AT_HOME", 1),
    ("PHALADEEPIKA_KALATRA_MERCURY_SEVENTH_IN_TAURUS", 0),
    ("PHALADEEPIKA_KALATRA_MOON_AND_SATURN_SEVENTH", 0),
    ("PHALADEEPIKA_KALATRA_SATURN_AND_MARS_SEVENTH_IN_CANCER", 0),
    ("PHALADEEPIKA_KALATRA_SATURN_OR_MARS_SEVENTH_IN_PISCES", 0),
    ("PHALADEEPIKA_KALATRA_SECOND_AND_SEVENTH_AFFLICTED", 43),
    ("PHALADEEPIKA_KALATRA_SECOND_AND_SEVENTH_BLESSED", 6),
    ("PHALADEEPIKA_KALATRA_SEVENTH_AND_EIGHTH_AFFLICTED", 42),
    ("PHALADEEPIKA_KALATRA_SEVENTH_AND_EIGHTH_BLESSED", 6),
    ("PHALADEEPIKA_KALATRA_SEVENTH_BEARS_FRUIT", 87),
    (
        "PHALADEEPIKA_KALATRA_SEVENTH_LORD_AND_VENUS_IN_DUAL_SIGNS_OR_NAVAMSHAS",
        33,
    ),
    ("PHALADEEPIKA_KALATRA_SEVENTH_LORD_FIFTH", 9),
    ("PHALADEEPIKA_KALATRA_SEVENTH_OR_ITS_LORD_AFFLICTED", 90),
    ("PHALADEEPIKA_KALATRA_STRONG_BENEFIC_SEVENTH_LORD", 20),
    ("PHALADEEPIKA_KALATRA_SUN_AND_RAHU_SEVENTH", 0),
    (
        "PHALADEEPIKA_KALATRA_VENUS_AND_MOON_SEVENTH_FROM_MARS_AND_SATURN",
        0,
    ),
    ("PHALADEEPIKA_KALATRA_VENUS_HEMMED_BY_MALEFICS", 19),
    (
        "PHALADEEPIKA_KALATRA_VENUS_IN_A_VARGA_OF_MARS_OR_SATURN_OR_ASPECTED_BY_THEM",
        61,
    ),
    (
        "PHALADEEPIKA_KALATRA_VENUS_JOINED_OR_ASPECTED_BY_A_MALEFIC",
        70,
    ),
    ("PHALADEEPIKA_KALATRA_VENUS_SEVENTH_IN_SCORPIO", 0),
    (
        "PHALADEEPIKA_KALATRA_VENUS_WITH_A_MALEFIC_SEVENTH_FIFTH_OR_NINTH",
        11,
    ),
    (
        "PHALADEEPIKA_KALATRA_WANING_MOON_FIFTH_MALEFICS_TWELFTH_SEVENTH_FIRST",
        0,
    ),
    (
        "PHALADEEPIKA_PRAVRAJYA_FOUR_IN_ONE_SIGN_JUPITER_STRONGEST",
        2,
    ),
    ("PHALADEEPIKA_PRAVRAJYA_FOUR_IN_ONE_SIGN_MARS_STRONGEST", 0),
    (
        "PHALADEEPIKA_PRAVRAJYA_FOUR_IN_ONE_SIGN_MERCURY_STRONGEST",
        1,
    ),
    ("PHALADEEPIKA_PRAVRAJYA_FOUR_IN_ONE_SIGN_MOON_STRONGEST", 2),
    (
        "PHALADEEPIKA_PRAVRAJYA_FOUR_IN_ONE_SIGN_SATURN_STRONGEST",
        3,
    ),
    ("PHALADEEPIKA_PRAVRAJYA_FOUR_IN_ONE_SIGN_SUN_STRONGEST", 0),
    ("PHALADEEPIKA_PRAVRAJYA_FOUR_IN_ONE_SIGN_VENUS_STRONGEST", 0),
    (
        "PHALADEEPIKA_PRAVRAJYA_LAGNA_AT_THE_END_OF_A_BENEFICS_SIGN_JUPITER_IN_A_KENDRA_OR_TRIKONA",
        4,
    ),
    (
        "PHALADEEPIKA_PRAVRAJYA_LAGNA_LORD_ASPECTS_THE_WANING_MOON",
        1,
    ),
    (
        "PHALADEEPIKA_PRAVRAJYA_LAGNA_LORD_ASPECTS_THE_WANING_MOON_IN_A_RAJA_YOGA",
        1,
    ),
    (
        "PHALADEEPIKA_PRAVRAJYA_MOON_IN_A_NAVAMSHA_OF_MARS_ASPECTED_BY_SATURN",
        2,
    ),
    (
        "PHALADEEPIKA_PRAVRAJYA_MOON_IN_SATURNS_DREKKANA_ASPECTED_BY_MARS_AND_SATURN",
        1,
    ),
    (
        "PHALADEEPIKA_PRAVRAJYA_MOON_SIGN_LORD_ASPECTED_BY_SATURN_ALONE",
        12,
    ),
    (
        "PHALADEEPIKA_PRAVRAJYA_TENTH_LORD_AMONG_FOUR_IN_ONE_SIGN",
        2,
    ),
    (
        "PHALADEEPIKA_PRAVRAJYA_TENTH_LORD_WITH_FOUR_IN_A_KENDRA_OR_TRIKONA",
        0,
    ),
    ("PHALADEEPIKA_PRAVRAJYA_THREE_STRONG_IN_ONE_GOOD_HOUSE", 12),
    (
        "PHALADEEPIKA_PUTRA_BENEFIC_FIFTH_IN_OWN_SIGN_OR_EXALTED",
        13,
    ),
    ("PHALADEEPIKA_PUTRA_FIFTHS_AFFLICTED_AND_UNRELIEVED", 16),
    ("PHALADEEPIKA_PUTRA_FIFTHS_HEMMED_LORDS_IN_DUSTHANAS", 0),
    ("PHALADEEPIKA_PUTRA_FIFTH_A_SIGN_OF_FEW_CHILDREN", 14),
    (
        "PHALADEEPIKA_PUTRA_FIFTH_LORD_AT_HOME_UNASPECTED_BY_BENEFICS",
        12,
    ),
    (
        "PHALADEEPIKA_PUTRA_FIFTH_LORD_HOSTILE_COMBUST_OR_WITH_A_DUSTHANA_LORD",
        50,
    ),
    (
        "PHALADEEPIKA_PUTRA_FIFTH_LORD_JUPITER_MARS_AND_SUN_IN_MALE_NAVAMSHAS",
        11,
    ),
    (
        "PHALADEEPIKA_PUTRA_FIFTH_OF_SATURN_OR_MERCURY_WITH_MANDI_OR_SATURN",
        18,
    ),
    ("PHALADEEPIKA_PUTRA_JUPITER_ALONE_IN_A_CANCER_FIFTH", 0),
    ("PHALADEEPIKA_PUTRA_LAGNA_AND_FIFTH_LORDS_CONNECTED", 13),
    (
        "PHALADEEPIKA_PUTRA_LORDS_OF_THE_FIFTH_AND_JUPITER_WELL_PLACED",
        46,
    ),
    ("PHALADEEPIKA_PUTRA_MALEFICS_TWELFTH_EIGHTH_FIFTH_FIRST", 0),
    ("PHALADEEPIKA_PUTRA_MALEFIC_FIFTH_LORD_AT_HOME", 3),
    (
        "PHALADEEPIKA_PUTRA_MALEFIC_FIFTH_LORD_AT_HOME_WITH_ANOTHER_MALEFIC",
        1,
    ),
    (
        "PHALADEEPIKA_PUTRA_MALEFIC_FOURTH_VENUS_SEVENTH_MOON_TENTH",
        0,
    ),
    (
        "PHALADEEPIKA_PUTRA_MALEFIC_RISING_LAGNA_LORD_FIFTH_FIFTH_LORD_THIRD_MOON_FOURTH",
        0,
    ),
    (
        "PHALADEEPIKA_PUTRA_MOON_ELEVENTH_MALEFIC_FIFTH_FROM_JUPITER_MANY_RISING",
        0,
    ),
    (
        "PHALADEEPIKA_PUTRA_MOON_FIFTH_MALEFICS_EIGHTH_TWELFTH_FIRST",
        0,
    ),
    ("PHALADEEPIKA_PUTRA_MOON_FIFTH_ODD_ASPECTED_BY_THE_SUN", 0),
    (
        "PHALADEEPIKA_PUTRA_MOON_OR_MERCURY_ALONE_IN_A_CANCER_FIFTH",
        1,
    ),
    ("PHALADEEPIKA_PUTRA_SATURN_ALONE_IN_A_CANCER_FIFTH", 0),
    (
        "PHALADEEPIKA_PUTRA_SATURN_RISING_JUPITER_EIGHTH_MARS_TWELFTH_FIFTH_OF_FEW_CHILDREN",
        0,
    ),
    (
        "PHALADEEPIKA_PUTRA_SUN_FIFTH_IN_A_SIGN_OF_FEW_CHILDREN_SATURN_EIGHTH_MARS_RISING",
        0,
    ),
    (
        "PHALADEEPIKA_PUTRA_SUN_MARS_OR_VENUS_ALONE_IN_A_CANCER_FIFTH",
        0,
    ),
    (
        "PHALADEEPIKA_PUTRA_VENUS_AND_MERCURY_SEVENTH_JUPITER_FIFTH_MALEFIC_FOURTH",
        0,
    ),
    (
        "PHALADEEPIKA_PUTRA_WEAK_FIFTH_LORD_UNBOUND_TO_THE_FIRST_AND_SEVENTH_LORDS",
        7,
    ),
];
