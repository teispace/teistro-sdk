//! The rules the SDK writes from the texts, over the conformance corpus's
//! recorded charts: every one reads strictly, is evaluable, and fires on a
//! pinned number of the 93 charts, so a rule that quietly stops answering — or
//! starts answering everywhere — fails the build.
//!
//! The corpus records no engine answer for these: they are the SDK's own
//! readings of BPHS chs. 9, 10 and 92 (`03-design/rules-engine.md`). What can
//! be held is that each is evaluable, that what it needs is what the chart
//! carries, and that its answers do not move.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they read"
)]

mod common;

use std::collections::BTreeMap;

use common::{chart, files_in};
use teistro_rules::{Evaluator, Readings, Rule, Tables, shipped};

#[test]
fn every_rule_the_sdk_writes_fires_where_it_did() {
    let rules: Vec<&Rule> = shipped::arishtas()
        .iter()
        .chain(shipped::gandantas())
        .collect();
    assert_eq!(rules.len(), 51);
    for rule in &rules {
        assert!(rule.is_evaluable(), "{} is evaluable", rule.key);
        assert_eq!(
            rule.source.rank.map(teistro_rules::EvidenceRank::get),
            Some(1),
            "{}: read from a text",
            rule.key
        );
        assert!(rule.source.verse.is_some(), "{}: cites a verse", rule.key);
        Tables::EMPTY.check(rule).expect("these name no table");
    }
    // The antidotes an evil names are shipped beside it, and nothing loops.
    let owned: Vec<Rule> = rules.iter().map(|rule| (*rule).clone()).collect();
    teistro_rules::check_references(&owned).expect("the pack names itself soundly");

    let mut fired: BTreeMap<&str, usize> =
        rules.iter().map(|rule| (rule.key.as_str(), 0)).collect();
    let mut charts = 0;
    for (_, file) in files_in("doshas") {
        let chart = chart(&file["inputs"]);
        let evaluator = Evaluator::new(&chart, Readings::RECORDING_ENGINE).with_rules(&owned);
        charts += 1;
        for rule in &rules {
            if evaluator.evaluate(rule).present {
                *fired.get_mut(rule.key.as_str()).unwrap() += 1;
            }
        }
    }
    assert_eq!(charts, 93);
    let counts: Vec<(&str, usize)> = fired.into_iter().collect();
    assert_eq!(counts, ANSWERED, "a rule's answers moved");

    // A rule no chart answers has no positive case here, which is worth
    // saying: the corpus holds 93 births, the gandantas need ghatikas it does
    // not record, and the rest are coincidences of three, four or five grahas.
    let silent: Vec<&str> = counts
        .iter()
        .filter(|(_, count)| *count == 0)
        .map(|(key, _)| *key)
        .collect();
    assert_eq!(silent, SILENT);
}

/// The rules no recorded chart answers.
const SILENT: [&str; 23] = [
    "ABHUKTA_MOOLA",
    "ARISHTA_FIVE_IN_THE_SECOND",
    "ARISHTA_JUPITER_LAGNA_FOUR_IN_SECOND",
    "ARISHTA_JUPITER_SATURN_RAHU_IN_ORDER",
    "ARISHTA_MARS_IN_TENTH_IN_ENEMY_SIGN",
    "ARISHTA_MARS_SATURN_IN_KENDRA_FROM_MOON",
    "ARISHTA_MERCURY_SECOND_MALEFICS_AROUND",
    "ARISHTA_MOON_SIXTH_SATURN_LAGNA_MARS_SEVENTH",
    "ARISHTA_NODES_WITH_LUMINARIES_LAGNA_AFFLICTED",
    "ARISHTA_RAHU_WITH_JUPITER_IN_LAGNA_OR_FOURTH",
    "ARISHTA_SATURN_LAGNA_MOON_EIGHTH_JUPITER_THIRD",
    "ARISHTA_SATURN_MARS_SUN_IN_FIFTH",
    "ARISHTA_SATURN_TENTH_MOON_SIXTH_MARS_SEVENTH",
    "ARISHTA_SATURN_TWELFTH_SUN_NINTH_MARS_EIGHTH",
    "ARISHTA_SUN_NINTH_MARS_SEVENTH_JUPITER_VENUS_ELEVENTH",
    "ARISHTA_SUN_SEVENTH_MARS_TENTH_RAHU_TWELFTH",
    "BJ_ECLIPSED_MOON_IN_LAGNA_MARS_EIGHTH",
    "BJ_LUMINARY_IN_LAGNA_MALEFICS_IN_FIVE_EIGHT_NINE",
    "BJ_SATURN_SUN_MOON_MARS_IN_ORDER",
    "BJ_WANING_MOON_IN_TWELFTH",
    "LAGNA_GANDANTA",
    "NAKSHATRA_GANDANTA",
    "TITHI_GANDANTA",
];

/// How many of the 93 recorded charts each rule answers.
const ANSWERED: [(&str, usize); 51] = [
    ("ABHUKTA_MOOLA", 0),
    ("ARISHTA_BHANGA_BENEFICS_IN_KENDRAS_AND_TRIKONAS", 37),
    ("ARISHTA_BHANGA_BENEFIC_IN_KENDRA", 70),
    ("ARISHTA_BHANGA_LAGNA_ASPECTED_BY_PAKSHA", 12),
    ("ARISHTA_BHANGA_MARS_WITH_JUPITER", 31),
    ("ARISHTA_FIVE_IN_THE_SECOND", 0),
    ("ARISHTA_JUPITER_LAGNA_FOUR_IN_SECOND", 0),
    ("ARISHTA_JUPITER_SATURN_RAHU_IN_ORDER", 0),
    ("ARISHTA_LUMINARY_VENUS_OR_RAHU_IN_TWELFTH", 22),
    ("ARISHTA_MALEFICS_FROM_THE_MOON", 73),
    ("ARISHTA_MALEFICS_FROM_THE_SUN", 69),
    ("ARISHTA_MALEFICS_IN_FOURTH_TENTH_AND_TWELFTH", 2),
    ("ARISHTA_MALEFICS_IN_SIXTH_AND_TWELFTH", 13),
    ("ARISHTA_MALEFIC_IN_FOURTH_FROM_MOON", 2),
    ("ARISHTA_MALEFIC_IN_TRIKONA_FROM_WANING_MOON", 32),
    ("ARISHTA_MARS_IN_LAGNA_OR_EIGHTH", 1),
    ("ARISHTA_MARS_IN_TENTH_IN_ENEMY_SIGN", 0),
    ("ARISHTA_MARS_SATURN_IN_KENDRA_FROM_MOON", 0),
    ("ARISHTA_MERCURY_SECOND_MALEFICS_AROUND", 0),
    ("ARISHTA_MOON_ASPECTED_BY_THREE_MALEFICS", 1),
    ("ARISHTA_MOON_HEMMED_OR_ASPECTED_BY_MALEFICS", 55),
    ("ARISHTA_MOON_IN_DUSTHANA_MALEFIC_ASPECT", 15),
    ("ARISHTA_MOON_SIXTH_SATURN_LAGNA_MARS_SEVENTH", 0),
    ("ARISHTA_NODES_WITH_LUMINARIES_LAGNA_AFFLICTED", 0),
    ("ARISHTA_RAHU_WITH_JUPITER_IN_LAGNA_OR_FOURTH", 0),
    ("ARISHTA_RETROGRADE_BENEFIC_IN_DUSTHANA", 5),
    ("ARISHTA_SATURN_LAGNA_MOON_EIGHTH_JUPITER_THIRD", 0),
    ("ARISHTA_SATURN_MARS_SUN_IN_FIFTH", 0),
    ("ARISHTA_SATURN_TENTH_MOON_SIXTH_MARS_SEVENTH", 0),
    ("ARISHTA_SATURN_TWELFTH_SUN_NINTH_MARS_EIGHTH", 0),
    ("ARISHTA_SUN_AFFLICTED_WITH_MALEFIC_IN_SEVENTH_FROM_HIM", 13),
    ("ARISHTA_SUN_ASPECTED_BY_SATURN_IN_MARS_NAVAMSHA", 4),
    ("ARISHTA_SUN_EXALTED_OR_DEBILITATED_IN_SEVENTH", 3),
    ("ARISHTA_SUN_HEMMED_OR_ASPECTED_BY_MALEFICS", 40),
    ("ARISHTA_SUN_NINTH_MARS_SEVENTH_JUPITER_VENUS_ELEVENTH", 0),
    ("ARISHTA_SUN_SEVENTH_MARS_TENTH_RAHU_TWELFTH", 0),
    ("ARISHTA_WANING_MOON_IN_LAGNA_MALEFIC_SEVENTH", 1),
    ("BJ_ECLIPSED_MOON_IN_LAGNA_MARS_EIGHTH", 0),
    ("BJ_LUMINARY_IN_LAGNA_MALEFICS_IN_FIVE_EIGHT_NINE", 0),
    ("BJ_MALEFICS_IN_FIFTH_AND_NINTH", 12),
    ("BJ_MALEFICS_IN_SIXTH_AND_EIGHTH", 14),
    ("BJ_MALEFICS_IN_TWELFTH_AND_SECOND", 17),
    ("BJ_MALEFICS_ON_LAGNA_SEVENTH_AND_THE_MOON", 4),
    ("BJ_MOON_IN_LAGNA_MALEFIC_IN_SEVENTH", 2),
    ("BJ_MOON_IN_THE_LAST_NAVAMSHA", 3),
    ("BJ_SATURN_SUN_MOON_MARS_IN_ORDER", 0),
    ("BJ_WANING_MOON_IN_TWELFTH", 0),
    ("BJ_WANING_MOON_WITH_A_MALEFIC", 22),
    ("LAGNA_GANDANTA", 0),
    ("NAKSHATRA_GANDANTA", 0),
    ("TITHI_GANDANTA", 0),
];
