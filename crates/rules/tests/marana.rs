//! BPHS ch. 44's manner and place of death and the worlds before and after,
//! held to the 93 recorded charts: what each answers, and the partitions the
//! verses make of every chart.

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
