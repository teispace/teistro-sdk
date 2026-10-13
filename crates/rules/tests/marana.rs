//! BPHS ch. 44's manner and place of death and the worlds before and after,
//! and Phaladeepika ch. 14's diseases, death and births, held to the 93
//! recorded charts: what each answers, and the partitions the verses make of
//! every chart.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they read"
)]

mod common;

use std::collections::BTreeMap;

use common::{chart_at, files_in, vargas};
use teistro_rules::{Evaluator, Readings, Rule, shipped};

#[test]
fn the_verses_partition_every_chart_as_they_say() {
    let rules: Vec<Rule> = shipped::nabhasas()
        .iter()
        .filter(|rule| {
            matches!(rule.category.as_str(), "marana" | "loka")
                && rule.source.text == "BPHS"
                && rule.source.chapter.as_deref() == Some("44")
        })
        .cloned()
        .collect();
    assert_eq!(rules.len(), 34);
    let at = |key: &str| rules.iter().position(|rule| rule.key == key).unwrap();
    let groups = |keys: &[&str]| keys.iter().map(|key| at(key)).collect::<Vec<_>>();
    let place = groups(&[
        "BPHS_BENEFICS_ALONE_IN_THIRD_DEATH_IN_A_HOLY_PLACE",
        "BPHS_MALEFICS_ALONE_IN_THIRD_DEATH_IN_AN_UNHOLY_PLACE",
        "BPHS_BENEFICS_AND_MALEFICS_IN_THIRD_DEATH_IN_A_MIXED_PLACE",
    ]);
    let awareness = groups(&[
        "BPHS_JUPITER_OR_VENUS_IN_THIRD_CONSCIOUS_AT_DEATH",
        "BPHS_OTHERS_IN_THIRD_UNCONSCIOUS_BEFORE_DEATH",
    ]);
    let modality = groups(&[
        "BPHS_MOVABLE_THIRD_PLACE_OF_DEATH",
        "BPHS_FIXED_THIRD_PLACE_OF_DEATH",
        "BPHS_DUAL_THIRD_PLACE_OF_DEATH",
    ]);
    let descent = groups(&[
        "BPHS_DESCENT_FROM_THE_GODS",
        "BPHS_DESCENT_FROM_THE_MANES",
        "BPHS_DESCENT_FROM_YAMA",
        "BPHS_DESCENT_FROM_HELL",
    ]);
    let mut fired: BTreeMap<&str, usize> =
        rules.iter().map(|rule| (rule.key.as_str(), 0)).collect();
    let (mut charts, mut occupied, mut compared) = (0, 0, 0);
    for (path, file) in files_in("doshas") {
        let chart = chart_at(&path, &file["inputs"]);
        let divisions = vargas(&chart);
        let evaluator = Evaluator::new(&chart, Readings::TEXTS).with_vargas(&divisions);
        let present: Vec<bool> = rules
            .iter()
            .map(|rule| evaluator.evaluate(rule).present)
            .collect();
        for (rule, _) in rules.iter().zip(&present).filter(|(_, present)| **present) {
            *fired.get_mut(rule.key.as_str()).unwrap() += 1;
        }
        let holding = |group: &[usize]| group.iter().filter(|at| present[**at]).count();
        let third_occupied = teistro_rules::Body::ALL.into_iter().take(9).any(|body| {
            evaluator.house_sign(teistro_rules::House::try_new(3).unwrap())
                == chart.placement(body).sign
        });
        let expected = usize::from(third_occupied);
        // V. 32: every graha is a benefic or a malefic, so an occupied third is
        // exactly one of holy, unholy and mixed. V. 33: conscious or not.
        assert_eq!(holding(&place), expected, "{}", path.display());
        assert_eq!(holding(&awareness), expected, "{}", path.display());
        // V. 34: every sign is movable, fixed or dual.
        assert_eq!(holding(&modality), 1, "{}", path.display());
        // Vv. 41 to 42: the four lists of decanate lords are the seven grahas,
        // so where strength says which luminary is stronger, exactly one world.
        let strengths = evaluator.strengths();
        let sun = teistro_rules::Body::Graha(teistro_core::catalogue::Graha::Sun);
        let moon = teistro_rules::Body::Graha(teistro_core::catalogue::Graha::Moon);
        let decided = strengths.exceeds(sun, moon) || strengths.exceeds(moon, sun);
        assert_eq!(
            holding(&descent),
            usize::from(decided),
            "{}",
            path.display()
        );
        occupied += expected;
        compared += usize::from(decided);
        charts += 1;
    }
    assert_eq!((charts, occupied, compared), (93, 59, 71));
    let counts: Vec<(&str, usize)> = fired.into_iter().collect();
    assert_eq!(counts.as_slice(), ANSWERS.as_slice());
}

/// What each answers over the 93 recorded charts. The corpus's rule inputs
/// carry no points, so the Moon with Gulika answers none here; the SDK reads
/// it with a chart's points.
const ANSWERS: [(&str, usize); 34] = [
    ("BPHS_ASCENT_TO_EARTH", 62),
    ("BPHS_ASCENT_TO_HEAVEN", 26),
    ("BPHS_ASCENT_TO_HELL", 53),
    ("BPHS_ASCENT_TO_THE_MANES", 65),
    ("BPHS_BENEFICS_ALONE_IN_THIRD_DEATH_IN_A_HOLY_PLACE", 9),
    (
        "BPHS_BENEFICS_AND_MALEFICS_IN_THIRD_DEATH_IN_A_MIXED_PLACE",
        14,
    ),
    (
        "BPHS_BENEFIC_ON_EIGHTH_AND_NINTH_LORD_WITH_A_BENEFIC_DEATH_IN_A_SHRINE",
        17,
    ),
    ("BPHS_DESCENT_FROM_HELL", 26),
    ("BPHS_DESCENT_FROM_THE_GODS", 8),
    ("BPHS_DESCENT_FROM_THE_MANES", 20),
    ("BPHS_DESCENT_FROM_YAMA", 17),
    ("BPHS_DUAL_THIRD_PLACE_OF_DEATH", 16),
    ("BPHS_FIXED_THIRD_PLACE_OF_DEATH", 42),
    ("BPHS_JUPITER_IN_EIGHTH_DEATH_BY_DISEASE", 14),
    ("BPHS_JUPITER_ON_THIRD_DEATH_BY_SWELLING", 23),
    ("BPHS_JUPITER_OR_VENUS_IN_THIRD_CONSCIOUS_AT_DEATH", 17),
    ("BPHS_MALEFICS_ALONE_IN_THIRD_DEATH_IN_AN_UNHOLY_PLACE", 36),
    (
        "BPHS_MALEFIC_ON_EIGHTH_AND_NINTH_LORD_WITH_A_MALEFIC_DEATH_ELSEWHERE",
        21,
    ),
    ("BPHS_MANY_ON_THIRD_DEATH_BY_MANY_DISEASES", 29),
    ("BPHS_MARS_IN_EIGHTH_DEATH_BY_WEAPONS", 2),
    ("BPHS_MARS_IN_THIRD_DEATH_BY_WOUNDS", 8),
    ("BPHS_MERCURY_IN_EIGHTH_DEATH_BY_FEVER", 3),
    ("BPHS_MERCURY_ON_THIRD_DEATH_BY_FEVER", 18),
    ("BPHS_MOON_AND_GULIKA_ON_THIRD_DEATH_BY_INSECTS", 0),
    ("BPHS_MOON_IN_EIGHTH_DEATH_BY_WATER", 12),
    ("BPHS_MOON_IN_THIRD_DEATH_BY_CONSUMPTION", 23),
    ("BPHS_MOVABLE_THIRD_PLACE_OF_DEATH", 35),
    ("BPHS_OTHERS_IN_THIRD_UNCONSCIOUS_BEFORE_DEATH", 42),
    ("BPHS_SATURN_AND_RAHU_ON_THIRD_DEATH_BY_POISON", 1),
    ("BPHS_SATURN_IN_EIGHTH_DEATH_BY_THIRST", 5),
    ("BPHS_STRONG_SUN_IN_THIRD_DEATH_BY_THE_STATE", 1),
    ("BPHS_SUN_IN_EIGHTH_DEATH_BY_FIRE", 6),
    ("BPHS_VENUS_IN_EIGHTH_DEATH_BY_HUNGER", 18),
    ("BPHS_VENUS_ON_THIRD_DEATH_BY_URINARY_DISEASE", 13),
];

#[test]
fn phaladeepika_s_ways_of_death_and_birth_partition_every_chart() {
    let rules: Vec<Rule> = shipped::nabhasas()
        .iter()
        .filter(|rule| {
            rule.source.text == "Phaladeepika" && rule.source.chapter.as_deref() == Some("14")
        })
        .cloned()
        .collect();
    assert_eq!(rules.len(), 85);
    let named = |pattern: &dyn Fn(&str) -> bool| -> Vec<usize> {
        rules
            .iter()
            .enumerate()
            .filter(|(_, rule)| pattern(&rule.key))
            .map(|(at, _)| at)
            .collect()
    };
    let by_graha = named(&|key| key.ends_with("_IN_OR_ASPECTING_THE_EIGHTH_CAUSE_OF_DEATH"));
    let by_sign = named(&|key| key.starts_with("PHALADEEPIKA_EMPTY_EIGHTH_IN_"));
    let by_navamsha = named(&|key| key.starts_with("PHALADEEPIKA_EIGHTH_LORD_IN_A_"));
    let death = named(&|key| key.starts_with("PHALADEEPIKA_TWELFTH_") && key.ends_with("_DEATH"));
    let after = named(&|key| key.starts_with("PHALADEEPIKA_TWELFTH_LORD_"));
    assert_eq!(
        (
            by_graha.len(),
            by_sign.len(),
            by_navamsha.len(),
            death.len(),
            after.len()
        ),
        (9, 12, 12, 2, 2)
    );
    let births: Vec<[Vec<usize>; 3]> = ["PAST", "NEXT"]
        .iter()
        .map(|when| {
            let of = |end: &'static str| {
                let start = format!("PHALADEEPIKA_{when}_BIRTH_LORD_");
                named(&move |key: &str| {
                    key.starts_with(&start) && end.split('|').any(|e| key.ends_with(e))
                })
            };
            [
                of("_GODS|_ISLAND|_BHARATAVARSHA"),
                of("_LAND"),
                of("_HUMAN|_BEAST|_BIRD"),
            ]
        })
        .collect();
    for groups in &births {
        assert_eq!(groups.each_ref().map(Vec::len), [3, 6, 3]);
    }
    let mut fired: BTreeMap<&str, usize> =
        rules.iter().map(|rule| (rule.key.as_str(), 0)).collect();
    let mut charts = 0;
    for (path, file) in files_in("doshas") {
        let chart = chart_at(&path, &file["inputs"]);
        let divisions = vargas(&chart);
        let evaluator = Evaluator::new(&chart, Readings::TEXTS).with_vargas(&divisions);
        let present: Vec<bool> = rules
            .iter()
            .map(|rule| evaluator.evaluate(rule).present)
            .collect();
        for (rule, _) in rules.iter().zip(&present).filter(|(_, present)| **present) {
            *fired.get_mut(rule.key.as_str()).unwrap() += 1;
        }
        let holding = |group: &[usize]| group.iter().filter(|at| present[**at]).count();
        let name = path.display();
        // V. 13: a graha in or aspecting the eighth names the death, and
        // only an eighth free of them falls to its sign, one of twelve.
        let empty = usize::from(holding(&by_graha) == 0);
        assert_eq!(holding(&by_sign), empty, "{name}");
        // V. 16: the eighth lord's navamsha is one sign.
        assert_eq!(holding(&by_navamsha), 1, "{name}");
        // Vv. 21 and 22: each reading and its "otherwise".
        assert_eq!(holding(&death), 1, "{name}");
        assert_eq!(holding(&after), 1, "{name}");
        // Vv. 25, 26 and 28: each lord's dignity, its own name and its
        // sign's regard for the lagna lord are each one of the verse's cases.
        for groups in &births {
            assert_eq!(
                groups.each_ref().map(|group| holding(group)),
                [1, 1, 1],
                "{name}"
            );
        }
        charts += 1;
    }
    assert_eq!(charts, 93);
    let counts: Vec<(&str, usize)> = fired.into_iter().collect();
    assert_eq!(counts.as_slice(), PHALADEEPIKA_ANSWERS.as_slice());
}

/// What each of Phaladeepika ch. 14's rules answers over the 93 recorded
/// charts.
const PHALADEEPIKA_ANSWERS: [(&str, usize); 85] = [
    (
        "PHALADEEPIKA_EIGHTH_LORD_IN_A_AQUARIUS_NAVAMSHA_CAUSE_OF_DEATH",
        17,
    ),
    (
        "PHALADEEPIKA_EIGHTH_LORD_IN_A_ARIES_NAVAMSHA_CAUSE_OF_DEATH",
        1,
    ),
    (
        "PHALADEEPIKA_EIGHTH_LORD_IN_A_CANCER_NAVAMSHA_CAUSE_OF_DEATH",
        14,
    ),
    (
        "PHALADEEPIKA_EIGHTH_LORD_IN_A_CAPRICORN_NAVAMSHA_CAUSE_OF_DEATH",
        5,
    ),
    (
        "PHALADEEPIKA_EIGHTH_LORD_IN_A_GEMINI_NAVAMSHA_CAUSE_OF_DEATH",
        4,
    ),
    (
        "PHALADEEPIKA_EIGHTH_LORD_IN_A_LEO_NAVAMSHA_CAUSE_OF_DEATH",
        11,
    ),
    (
        "PHALADEEPIKA_EIGHTH_LORD_IN_A_LIBRA_NAVAMSHA_CAUSE_OF_DEATH",
        9,
    ),
    (
        "PHALADEEPIKA_EIGHTH_LORD_IN_A_PISCES_NAVAMSHA_CAUSE_OF_DEATH",
        5,
    ),
    (
        "PHALADEEPIKA_EIGHTH_LORD_IN_A_SAGITTARIUS_NAVAMSHA_CAUSE_OF_DEATH",
        1,
    ),
    (
        "PHALADEEPIKA_EIGHTH_LORD_IN_A_SCORPIO_NAVAMSHA_CAUSE_OF_DEATH",
        9,
    ),
    (
        "PHALADEEPIKA_EIGHTH_LORD_IN_A_TAURUS_NAVAMSHA_CAUSE_OF_DEATH",
        8,
    ),
    (
        "PHALADEEPIKA_EIGHTH_LORD_IN_A_VIRGO_NAVAMSHA_CAUSE_OF_DEATH",
        9,
    ),
    ("PHALADEEPIKA_EMPTY_EIGHTH_IN_AQUARIUS_CAUSE_OF_DEATH", 0),
    ("PHALADEEPIKA_EMPTY_EIGHTH_IN_ARIES_CAUSE_OF_DEATH", 2),
    ("PHALADEEPIKA_EMPTY_EIGHTH_IN_CANCER_CAUSE_OF_DEATH", 2),
    ("PHALADEEPIKA_EMPTY_EIGHTH_IN_CAPRICORN_CAUSE_OF_DEATH", 0),
    ("PHALADEEPIKA_EMPTY_EIGHTH_IN_GEMINI_CAUSE_OF_DEATH", 0),
    ("PHALADEEPIKA_EMPTY_EIGHTH_IN_LEO_CAUSE_OF_DEATH", 1),
    ("PHALADEEPIKA_EMPTY_EIGHTH_IN_LIBRA_CAUSE_OF_DEATH", 8),
    ("PHALADEEPIKA_EMPTY_EIGHTH_IN_PISCES_CAUSE_OF_DEATH", 1),
    ("PHALADEEPIKA_EMPTY_EIGHTH_IN_SAGITTARIUS_CAUSE_OF_DEATH", 0),
    ("PHALADEEPIKA_EMPTY_EIGHTH_IN_SCORPIO_CAUSE_OF_DEATH", 0),
    ("PHALADEEPIKA_EMPTY_EIGHTH_IN_TAURUS_CAUSE_OF_DEATH", 1),
    ("PHALADEEPIKA_EMPTY_EIGHTH_IN_VIRGO_CAUSE_OF_DEATH", 0),
    (
        "PHALADEEPIKA_FIFTH_AND_NINTH_LORDS_TOGETHER_BORN_AT_HOME",
        6,
    ),
    (
        "PHALADEEPIKA_JUPITER_IN_OR_ASPECTING_THE_EIGHTH_CAUSE_OF_DEATH",
        41,
    ),
    ("PHALADEEPIKA_JUPITER_IN_SIXTH_OR_EIGHTH_CONSUMPTION", 19),
    ("PHALADEEPIKA_JUPITER_OF_THE_TWELFTH_GOES_TO_BRAHMALOKA", 16),
    (
        "PHALADEEPIKA_KETU_IN_OR_ASPECTING_THE_EIGHTH_CAUSE_OF_DEATH",
        27,
    ),
    ("PHALADEEPIKA_KETU_OF_THE_TWELFTH_GOES_TO_HELL", 22),
    (
        "PHALADEEPIKA_LUMINARY_IN_THE_SECOND_OR_TWELFTH_AFFLICTED_EYE_DISEASE",
        11,
    ),
    (
        "PHALADEEPIKA_MALEFIC_OWNED_EIGHTH_HOLDING_A_MALEFIC_VIOLENT_DEATH",
        20,
    ),
    (
        "PHALADEEPIKA_MALEFIC_WITH_SIXTH_OR_EIGHTH_LORD_IN_FIFTH_BELLY_DISEASE",
        3,
    ),
    (
        "PHALADEEPIKA_MALEFIC_WITH_SIXTH_OR_EIGHTH_LORD_IN_SEVENTH_OR_EIGHTH_RECTAL_DISEASE",
        12,
    ),
    (
        "PHALADEEPIKA_MARS_IN_OR_ASPECTING_THE_EIGHTH_CAUSE_OF_DEATH",
        18,
    ),
    ("PHALADEEPIKA_MARS_OF_THE_TWELFTH_GOES_TO_THE_EARTH", 22),
    ("PHALADEEPIKA_MARS_OR_KETU_IN_SIXTH_OR_EIGHTH_WOUNDS", 35),
    (
        "PHALADEEPIKA_MERCURY_IN_OR_ASPECTING_THE_EIGHTH_CAUSE_OF_DEATH",
        13,
    ),
    ("PHALADEEPIKA_MERCURY_OF_THE_TWELFTH_GOES_TO_VAIKUNTHA", 15),
    (
        "PHALADEEPIKA_MOON_IN_OR_ASPECTING_THE_EIGHTH_CAUSE_OF_DEATH",
        17,
    ),
    ("PHALADEEPIKA_NEXT_BIRTH_LORD_AT_HOME_BHARATAVARSHA", 62),
    (
        "PHALADEEPIKA_NEXT_BIRTH_LORD_DEBILITATED_OR_HOSTILE_FOREIGN_ISLAND",
        31,
    ),
    ("PHALADEEPIKA_NEXT_BIRTH_LORD_EXALTED_LAND_OF_THE_GODS", 0),
    (
        "PHALADEEPIKA_NEXT_BIRTH_LORD_IN_A_SIGN_HOSTILE_TO_THE_LAGNA_LORD_BIRD",
        15,
    ),
    (
        "PHALADEEPIKA_NEXT_BIRTH_LORD_IN_A_SIGN_KIND_TO_THE_LAGNA_LORD_HUMAN",
        57,
    ),
    (
        "PHALADEEPIKA_NEXT_BIRTH_LORD_IN_A_SIGN_NEUTRAL_TO_THE_LAGNA_LORD_BEAST",
        21,
    ),
    ("PHALADEEPIKA_NEXT_BIRTH_LORD_JUPITER_LAND", 19),
    ("PHALADEEPIKA_NEXT_BIRTH_LORD_MARS_LAND", 10),
    ("PHALADEEPIKA_NEXT_BIRTH_LORD_MERCURY_LAND", 16),
    ("PHALADEEPIKA_NEXT_BIRTH_LORD_SATURN_LAND", 16),
    ("PHALADEEPIKA_NEXT_BIRTH_LORD_SUN_LAND", 3),
    ("PHALADEEPIKA_NEXT_BIRTH_LORD_VENUS_OR_MOON_LAND", 29),
    ("PHALADEEPIKA_PAST_BIRTH_LORD_AT_HOME_BHARATAVARSHA", 72),
    (
        "PHALADEEPIKA_PAST_BIRTH_LORD_DEBILITATED_OR_HOSTILE_FOREIGN_ISLAND",
        17,
    ),
    ("PHALADEEPIKA_PAST_BIRTH_LORD_EXALTED_LAND_OF_THE_GODS", 4),
    (
        "PHALADEEPIKA_PAST_BIRTH_LORD_IN_A_SIGN_HOSTILE_TO_THE_LAGNA_LORD_BIRD",
        18,
    ),
    (
        "PHALADEEPIKA_PAST_BIRTH_LORD_IN_A_SIGN_KIND_TO_THE_LAGNA_LORD_HUMAN",
        47,
    ),
    (
        "PHALADEEPIKA_PAST_BIRTH_LORD_IN_A_SIGN_NEUTRAL_TO_THE_LAGNA_LORD_BEAST",
        28,
    ),
    ("PHALADEEPIKA_PAST_BIRTH_LORD_JUPITER_LAND", 6),
    ("PHALADEEPIKA_PAST_BIRTH_LORD_MARS_LAND", 29),
    ("PHALADEEPIKA_PAST_BIRTH_LORD_MERCURY_LAND", 10),
    ("PHALADEEPIKA_PAST_BIRTH_LORD_SATURN_LAND", 11),
    ("PHALADEEPIKA_PAST_BIRTH_LORD_SUN_LAND", 7),
    ("PHALADEEPIKA_PAST_BIRTH_LORD_VENUS_OR_MOON_LAND", 30),
    (
        "PHALADEEPIKA_RAHU_IN_OR_ASPECTING_THE_EIGHTH_CAUSE_OF_DEATH",
        27,
    ),
    (
        "PHALADEEPIKA_RAHU_IN_SIXTH_OR_EIGHTH_ASPECTED_BY_MARS_BOILS",
        6,
    ),
    ("PHALADEEPIKA_RAHU_OF_THE_TWELFTH_GOES_TO_OTHER_ISLANDS", 15),
    (
        "PHALADEEPIKA_SATURN_IN_OR_ASPECTING_THE_EIGHTH_CAUSE_OF_DEATH",
        39,
    ),
    ("PHALADEEPIKA_SATURN_IN_SIXTH_OR_EIGHTH_WIND_DISEASE", 11),
    ("PHALADEEPIKA_SATURN_OF_THE_TWELFTH_GOES_TO_YAMAS_CITY", 12),
    (
        "PHALADEEPIKA_SATURN_WITH_THE_MOON_IN_SIXTH_OR_EIGHTH_ABDOMINAL_SWELLING",
        0,
    ),
    (
        "PHALADEEPIKA_SUN_IN_OR_ASPECTING_THE_EIGHTH_CAUSE_OF_DEATH",
        18,
    ),
    ("PHALADEEPIKA_SUN_IN_SIXTH_OR_EIGHTH_FEVER", 23),
    (
        "PHALADEEPIKA_SUN_OR_MOON_OF_THE_TWELFTH_GOES_TO_KAILASA",
        22,
    ),
    (
        "PHALADEEPIKA_THIRD_ELEVENTH_AND_JUPITER_AFFLICTED_EAR_DISEASE",
        9,
    ),
    (
        "PHALADEEPIKA_TWELFTH_AND_ITS_LORD_WITHOUT_BENEFICS_PAINFUL_DEATH",
        9,
    ),
    (
        "PHALADEEPIKA_TWELFTH_LORD_EXALTED_BEFRIENDED_OR_JOINED_BY_A_BENEFIC_RISES",
        44,
    ),
    ("PHALADEEPIKA_TWELFTH_LORD_OTHERWISE_DESCENDS", 49),
    (
        "PHALADEEPIKA_TWELFTH_OR_ITS_LORD_WITH_BENEFICS_PEACEFUL_DEATH",
        84,
    ),
    (
        "PHALADEEPIKA_TWO_MALEFICS_IN_KENDRAS_ASPECTING_EACH_OTHER_DEATH_BY_THE_KING",
        13,
    ),
    (
        "PHALADEEPIKA_VENUS_IN_OR_ASPECTING_THE_EIGHTH_CAUSE_OF_DEATH",
        21,
    ),
    ("PHALADEEPIKA_VENUS_IN_SIXTH_OR_EIGHTH_GENITAL_DISEASE", 20),
    ("PHALADEEPIKA_VENUS_OF_THE_TWELFTH_GOES_TO_SVARGA", 23),
    (
        "PHALADEEPIKA_VENUS_WITH_SIXTH_OR_EIGHTH_LORD_IN_SEVENTH_OR_EIGHTH_GENITAL_DISEASE",
        5,
    ),
    (
        "PHALADEEPIKA_WANING_MOON_WITH_A_MALEFIC_IN_A_WATERY_SIXTH_OR_EIGHTH_WATER_DISEASE",
        0,
    ),
];
