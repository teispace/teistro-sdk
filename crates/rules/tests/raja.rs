//! BPHS ch. 39's raja yogas and ch. 40's yogas for royal association, held to
//! the 93 recorded charts: what each answers, and what the chapters' own
//! arithmetic says must hold between them.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they read"
)]

mod common;

use std::collections::BTreeMap;

use common::{chart_at, files_in, vargas};
use teistro_rules::{Evaluator, Readings, Rule, shipped};

/// Every rule the two chapters ship.
fn chapters() -> Vec<Rule> {
    shipped::nabhasas()
        .iter()
        .filter(|rule| {
            rule.category == "raja"
                && rule.source.text == "BPHS"
                && matches!(rule.source.chapter.as_deref(), Some("39" | "40"))
        })
        .cloned()
        .collect()
}

#[test]
fn the_royal_combinations_answer_where_they_did() {
    let rules = chapters();
    assert_eq!(rules.len(), 58);
    let at = |key: &str| rules.iter().position(|rule| rule.key == key).unwrap();
    let exalted = [
        at("BPHS_ONE_TO_THREE_GRAHAS_EXALTED"),
        at("BPHS_SIX_GRAHAS_EXALTED"),
    ];
    let same = [
        at("BPHS_KING_BY_MALEFICS_THIRD_AND_SIXTH_FROM_ATMAKARAKA"),
        at("BPHS_ARMY_CHIEF_BY_MALEFICS_THIRD_AND_SIXTH_FROM_ATMAKARAKA"),
    ];
    let mut fired: BTreeMap<&str, usize> =
        rules.iter().map(|rule| (rule.key.as_str(), 0)).collect();
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
        // Vv. 44 and 46 count exalted grahas in bands that do not meet, so no
        // chart answers both.
        assert!(!exalted.iter().all(|at| present[*at]), "{}", path.display());
        // Ch. 39 v. 10 and ch. 40 v. 7 give one figure from the Atmakaraka two
        // effects, a king and an army chief: two claims, so two rules, which
        // must answer on exactly the same charts.
        assert_eq!(present[same[0]], present[same[1]], "{}", path.display());
    }
    let counts: Vec<(&str, usize)> = fired.into_iter().collect();
    assert_eq!(counts.as_slice(), ROYAL.as_slice());
}

/// What each answers over the 93 recorded charts. The widest is the verse's
/// own arithmetic: seven signs of twelve are a benefic's, so an Atmakaraka in
/// a benefic's sign or navamsha is about five charts in six.
const ROYAL: [(&str, usize); 58] = [
    ("BPHS_AMATYAKARAKA_IN_LAGNA_FIFTH_OR_NINTH", 11),
    ("BPHS_AMATYAKARAKA_STRONG_WITH_A_BENEFIC_OR_DIGNIFIED", 22),
    ("BPHS_AMATYAKARAKA_WITH_ATMAKARAKAS_DISPOSITOR", 13),
    (
        "BPHS_ARMY_CHIEF_BY_MALEFICS_THIRD_AND_SIXTH_FROM_ARUDHA_LAGNA",
        8,
    ),
    (
        "BPHS_ARMY_CHIEF_BY_MALEFICS_THIRD_AND_SIXTH_FROM_ATMAKARAKA",
        8,
    ),
    ("BPHS_ARMY_CHIEF_BY_MALEFICS_THIRD_AND_SIXTH_FROM_LAGNA", 25),
    ("BPHS_ARUDHA_LAGNA_AND_DARAPADA_WELL_RELATED", 45),
    (
        "BPHS_ARUDHA_LAGNA_HELD_BY_AN_EXALTED_GRAHA_OR_JUPITER_OR_VENUS_UNOBSTRUCTED",
        1,
    ),
    ("BPHS_ATMAKARAKA_IN_A_BENEFICS_SIGN_OR_NAVAMSHA", 83),
    (
        "BPHS_ATMAKARAKA_IN_FIFTH_SEVENTH_NINTH_OR_TENTH_WITH_A_BENEFIC",
        6,
    ),
    ("BPHS_ATMAKARAKA_WELL_PLACED_ASPECTED_BY_NINTH_LORD", 4),
    ("BPHS_A_BENEFIC_EXALTED_AND_A_BENEFIC_ANGULAR", 14),
    ("BPHS_A_BENEFIC_EXALTED_IN_THE_SECOND", 0),
    ("BPHS_BENEFICS_LAGNA_SECOND_FOURTH_AND_A_MALEFIC_THIRD", 0),
    ("BPHS_DEBILITATED_DUSTHANA_LORD_ASPECTS_LAGNA", 9),
    (
        "BPHS_DEBILITATED_GRAHAS_SIXTH_EIGHTH_THIRD_AND_LAGNA_LORD_DIGNIFIED",
        0,
    ),
    ("BPHS_DEBILITATED_GRAHA_IN_SIXTH_OR_EIGHTH_ASPECTS_LAGNA", 0),
    (
        "BPHS_DEBILITATED_GRAHA_IN_THIRD_OR_ELEVENTH_ASPECTS_LAGNA",
        1,
    ),
    (
        "BPHS_DIGNIFIED_GRAHAS_ON_THE_LAGNA_HORA_AND_GHATIKA_LAGNAS",
        0,
    ),
    ("BPHS_DUSTHANA_LORDS_AFFLICTED_AND_LAGNA_LORD_DIGNIFIED", 0),
    (
        "BPHS_ELEVENTH_LORD_IN_ELEVENTH_UNASPECTED_AND_ATMAKARAKA_WITH_A_BENEFIC",
        1,
    ),
    (
        "BPHS_EVERY_BENEFIC_ANGULAR_AND_EVERY_MALEFIC_IN_UPACHAYAS",
        0,
    ),
    (
        "BPHS_EXALTED_ASPECTS_ON_TWO_OF_THE_BHAVA_HORA_AND_GHATIKA_LAGNAS",
        0,
    ),
    (
        "BPHS_EXALTED_GRAHAS_ON_THE_LAGNA_DREKKANA_AND_NAVAMSHA_LAGNAS",
        0,
    ),
    ("BPHS_FIFTH_AND_NINTH_LORDS_RELATED", 11),
    ("BPHS_FIFTH_LORD_ANGULAR_WITH_NINTH_OR_LAGNA_LORD", 3),
    (
        "BPHS_FOURTH_AND_TENTH_LORDS_EXCHANGED_ASPECTED_BY_TRINE_LORDS",
        0,
    ),
    ("BPHS_FOUR_OR_FIVE_GRAHAS_EXALTED_OR_MOOLATRIKONA", 0),
    ("BPHS_JUPITER_OWN_NINTH_WITH_VENUS_OR_FIFTH_LORD", 0),
    (
        "BPHS_KING_BY_BENEFICS_SECOND_FOURTH_AND_FIFTH_FROM_ATMAKARAKA",
        0,
    ),
    (
        "BPHS_KING_BY_BENEFICS_SECOND_FOURTH_AND_FIFTH_FROM_LAGNA_LORD",
        0,
    ),
    ("BPHS_KING_BY_MALEFICS_THIRD_AND_SIXTH_FROM_ATMAKARAKA", 8),
    ("BPHS_KING_BY_MALEFICS_THIRD_AND_SIXTH_FROM_LAGNA_LORD", 2),
    ("BPHS_LAGNA_FOURTH_FIFTH_TENTH_LORDS_IN_NINTH", 0),
    (
        "BPHS_LAGNA_LORD_AND_ATMAKARAKA_IN_LAGNA_FIFTH_OR_SEVENTH_WITH_BENEFICS",
        4,
    ),
    (
        "BPHS_LAGNA_LORD_OR_ATMAKARAKA_WITH_FIFTH_LORD_ANGULAR_OR_TRINE",
        8,
    ),
    ("BPHS_LORDS_OF_10_AND_5_JOINED", 8),
    ("BPHS_LORDS_OF_10_AND_9_JOINED", 16),
    ("BPHS_LORDS_OF_4_AND_5_JOINED", 18),
    ("BPHS_LORDS_OF_4_AND_9_JOINED", 21),
    ("BPHS_MAHA_RAJA_ATMAKARAKA_AND_PUTRAKARAKA_WELL_PLACED", 0),
    ("BPHS_MAHA_RAJA_LAGNA_AND_FIFTH_LORDS_EXCHANGED", 0),
    (
        "BPHS_MOONS_DISPOSITOR_AS_ATMAKARAKA_IN_LAGNA_WITH_A_BENEFIC",
        0,
    ),
    (
        "BPHS_MOON_AND_A_BENEFIC_IN_ARUDHA_LAGNA_AND_JUPITER_SECOND",
        0,
    ),
    ("BPHS_MOON_AND_VENUS_THIRD_ELEVENTH_OR_IN_MUTUAL_ASPECT", 19),
    ("BPHS_MOON_IN_A_BENEFIC_ARUDHA_LAGNA_AND_JUPITER_SECOND", 0),
    ("BPHS_NINTHS_ARUDHA_ON_LAGNA_OR_ATMAKARAKA_NINTH", 17),
    ("BPHS_ONE_GRAHA_ON_THE_LAGNA_OF_SIX_VARGAS", 5),
    ("BPHS_ONE_TO_THREE_GRAHAS_EXALTED", 37),
    ("BPHS_SIX_GRAHAS_EXALTED", 0),
    ("BPHS_STRONG_VARGOTTAMA_MOON_ASPECTED_BY_FOUR", 0),
    (
        "BPHS_TENTH_AND_ELEVENTH_FREE_OF_MALEFICS_AND_ELEVENTH_LORD_ASPECTS_IT",
        0,
    ),
    ("BPHS_TENTH_AND_LAGNA_LORDS_EXCHANGED", 0),
    ("BPHS_TENTH_LORD_DIGNIFIED_ASPECTS_LAGNA", 9),
    ("BPHS_TENTH_LORD_WITH_AMATYAKARAKA_OR_ITS_DISPOSITOR", 26),
    ("BPHS_VENUS_AND_MOON_FOURTH_FROM_KARAKAMSHA", 0),
    ("BPHS_VENUS_IN_A_ROYAL_PLACE_WITH_JUPITER_OR_THE_MOON", 15),
    (
        "BPHS_WEALTH_LORD_ASPECTS_LAGNA_AND_VENUS_ON_ARUDHA_ELEVENTH",
        3,
    ),
];
