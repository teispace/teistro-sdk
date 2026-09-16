//! BPHS ch. 43's combinations for the class of life, held to the 93 recorded
//! charts: what each answers, and what the verses' own arithmetic says must
//! hold between them.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they read"
)]

mod common;

use std::collections::BTreeMap;

use common::{chart_at, files_in};
use teistro_rules::{Evaluator, LifeClass, Readings, Rule, shipped};

#[test]
fn the_classes_of_life_answer_where_they_did() {
    let rules: Vec<Rule> = shipped::nabhasas()
        .iter()
        .filter(|rule| rule.source.text == "BPHS" && rule.source.chapter.as_deref() == Some("43"))
        .cloned()
        .collect();
    assert_eq!(rules.len(), 21);
    // Every one of them gives a class of life and nothing else.
    for rule in &rules {
        assert!(
            rule.life_class().is_some() && rule.effect().is_none(),
            "{}",
            rule.key
        );
    }
    let at = |key: &str| rules.iter().position(|rule| rule.key == key).unwrap();
    let placed = [
        at("BPHS_STRONGER_OF_LAGNA_AND_EIGHTH_LORDS_IN_ANGLE"),
        at("BPHS_STRONGER_OF_LAGNA_AND_EIGHTH_LORDS_IN_PANAPHARA"),
        at("BPHS_STRONGER_OF_LAGNA_AND_EIGHTH_LORDS_IN_APOKLIMA"),
    ];
    let helped = [
        at("BPHS_LAGNA_AND_EIGHTH_LORDS_WEAK_AND_UNDIGNIFIED_SHORT"),
        at("BPHS_LAGNA_AND_EIGHTH_LORDS_WEAK_AND_UNDIGNIFIED_BUT_HELPED_MEDIUM"),
    ];
    let mut fired: BTreeMap<&str, usize> =
        rules.iter().map(|rule| (rule.key.as_str(), 0)).collect();
    let mut by_class: BTreeMap<LifeClass, usize> = BTreeMap::new();
    let (mut charts, mut with_strength) = (0, 0);
    for (path, file) in files_in("doshas") {
        let chart = chart_at(&path, &file["inputs"]);
        let evaluator = Evaluator::new(&chart, Readings::TEXTS).with_rules(&rules);
        let present: Vec<bool> = rules
            .iter()
            .map(|rule| evaluator.evaluate(rule).present)
            .collect();
        for (rule, _) in rules.iter().zip(&present).filter(|(_, present)| **present) {
            *fired.get_mut(rule.key.as_str()).unwrap() += 1;
            *by_class.entry(rule.life_class().unwrap()).or_default() += 1;
        }
        // Vv. 71 to 73: the angles, panapharas and apoklimas are the twelve
        // houses between them, so where strength can be compared the stronger
        // lord stands in exactly one; without it, in at most one.
        let placements = placed.iter().filter(|at| present[**at]).count();
        if chart.strengths.is_some() {
            assert_eq!(placements, 1, "{}", path.display());
            with_strength += 1;
        } else {
            assert!(placements <= 1, "{}", path.display());
        }
        // V. 78: helped and unhelped are one question answered two ways.
        assert!(!helped.iter().all(|at| present[*at]), "{}", path.display());
        charts += 1;
    }
    assert_eq!((charts, with_strength), (93, 71));
    // Seventy-one charts compare strength and three more have one graha
    // lording both houses, so vv. 71 to 73 place 74 between them.
    assert_eq!(
        placed
            .iter()
            .map(|at| fired[rules[*at].key.as_str()])
            .sum::<usize>(),
        74
    );
    let classes: Vec<(LifeClass, usize)> = by_class.into_iter().collect();
    assert_eq!(
        classes,
        [
            (LifeClass::Short, 101),
            (LifeClass::Medium, 18),
            (LifeClass::Long, 97)
        ]
    );
    let counts: Vec<(&str, usize)> = fired.into_iter().collect();
    assert_eq!(counts.as_slice(), CLASSES.as_slice());
}

/// What each answers over the 93 recorded charts. The widest, v. 74, is the
/// verse's own arithmetic: some malefic joins or aspects a given graha about
/// seven charts in ten, both of a pair about half, and either pair about three
/// in four.
const CLASSES: [(&str, usize); 21] = [
    (
        "BPHS_BENEFICS_ANGULAR_OR_TRINE_MALEFICS_UPACHAYA_BENEFIC_EIGHTH_DIVINE",
        0,
    ),
    (
        "BPHS_BENEFICS_SIXTH_SEVENTH_EIGHTH_MALEFICS_THIRD_ELEVENTH_LONG",
        0,
    ),
    (
        "BPHS_BENEFIC_ANGULAR_AND_LAGNA_LORD_WITH_A_BENEFIC_LONG",
        18,
    ),
    ("BPHS_CANCER_RISING_WITH_JUPITER_AND_MOON_LIMITLESS", 0),
    (
        "BPHS_DUAL_LAGNA_TWO_MALEFICS_ANGULAR_FROM_STRONG_LAGNA_LORD_LONG",
        13,
    ),
    ("BPHS_DUAL_LAGNA_WITH_ITS_LORD_WELL_PLACED_LONG", 24),
    (
        "BPHS_LAGNA_AND_EIGHTH_LORDS_WEAK_AND_UNDIGNIFIED_BUT_HELPED_MEDIUM",
        1,
    ),
    ("BPHS_LAGNA_AND_EIGHTH_LORDS_WEAK_AND_UNDIGNIFIED_SHORT", 6),
    ("BPHS_LAGNA_LORD_ANGULAR_WITH_JUPITER_AND_VENUS_LONG", 0),
    (
        "BPHS_LAGNA_LORD_WITH_A_MALEFIC_IN_A_DUSTHANA_UNHELPED_SHORT",
        5,
    ),
    ("BPHS_MALEFICS_TWELFTH_AND_SECOND_UNHELPED_SHORT", 4),
    ("BPHS_MALEFICS_UPACHAYA_BENEFICS_ANGULAR_LONG", 0),
    ("BPHS_MALEFIC_EIGHTH_AND_TENTH_LORD_EXALTED_LONG", 3),
    (
        "BPHS_MARS_AND_THIRD_LORD_OR_EIGHTH_LORD_AND_SATURN_AFFLICTED_SHORT",
        53,
    ),
    ("BPHS_SATURN_OR_LAGNA_LORD_WITH_AN_EXALTED_GRAHA_LONG", 8),
    ("BPHS_STRONGER_OF_LAGNA_AND_EIGHTH_LORDS_IN_ANGLE", 31),
    ("BPHS_STRONGER_OF_LAGNA_AND_EIGHTH_LORDS_IN_APOKLIMA", 26),
    ("BPHS_STRONGER_OF_LAGNA_AND_EIGHTH_LORDS_IN_PANAPHARA", 17),
    (
        "BPHS_THREE_DIGNIFIED_IN_EIGHTH_AND_LAGNA_LORD_STRONG_LONG",
        0,
    ),
    ("BPHS_THREE_EXALTED_WITH_LAGNA_AND_EIGHTH_LORDS_LONG", 0),
    ("BPHS_UNHELPED_MALEFIC_ANGULAR_AND_LAGNA_LORD_WEAK_SHORT", 7),
];
