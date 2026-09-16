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
    assert_eq!(rules.len(), 19);
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

    let mut fired: BTreeMap<&str, usize> =
        rules.iter().map(|rule| (rule.key.as_str(), 0)).collect();
    let mut charts = 0;
    for (_, file) in files_in("doshas") {
        let chart = chart(&file["inputs"]);
        let evaluator = Evaluator::new(&chart, Readings::RECORDING_ENGINE);
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
    // saying: the corpus holds 93 births and these figures are rare.
    let silent: Vec<&str> = counts
        .iter()
        .filter(|(_, count)| *count == 0)
        .map(|(key, _)| *key)
        .collect();
    assert_eq!(
        silent,
        [
            "ABHUKTA_MOOLA",
            "ARISHTA_NODES_WITH_LUMINARIES_LAGNA_AFFLICTED",
            "ARISHTA_SATURN_LAGNA_MOON_EIGHTH_JUPITER_THIRD",
            "ARISHTA_SATURN_MARS_SUN_IN_FIFTH",
            "ARISHTA_SATURN_TENTH_MOON_SIXTH_MARS_SEVENTH",
            "ARISHTA_SATURN_TWELFTH_SUN_NINTH_MARS_EIGHTH",
            "ARISHTA_SUN_NINTH_MARS_SEVENTH_JUPITER_VENUS_ELEVENTH",
            "LAGNA_GANDANTA",
            "NAKSHATRA_GANDANTA",
            "TITHI_GANDANTA",
        ],
        "the four gandantas are silent because the corpus records no limb's ghatikas, and the rest are coincidences of three or four grahas"
    );
}

/// How many of the 93 recorded charts each rule answers.
const ANSWERED: [(&str, usize); 19] = [
    ("ABHUKTA_MOOLA", 0),
    ("ARISHTA_BHANGA_BENEFICS_IN_KENDRAS_AND_TRIKONAS", 37),
    ("ARISHTA_BHANGA_BENEFIC_IN_KENDRA", 70),
    ("ARISHTA_BHANGA_LAGNA_ASPECTED_BY_PAKSHA", 12),
    ("ARISHTA_BHANGA_MARS_WITH_JUPITER", 31),
    ("ARISHTA_LUMINARY_VENUS_OR_RAHU_IN_TWELFTH", 22),
    ("ARISHTA_MARS_IN_LAGNA_OR_EIGHTH", 1),
    ("ARISHTA_MOON_IN_DUSTHANA_MALEFIC_ASPECT", 15),
    ("ARISHTA_NODES_WITH_LUMINARIES_LAGNA_AFFLICTED", 0),
    ("ARISHTA_RETROGRADE_BENEFIC_IN_DUSTHANA", 5),
    ("ARISHTA_SATURN_LAGNA_MOON_EIGHTH_JUPITER_THIRD", 0),
    ("ARISHTA_SATURN_MARS_SUN_IN_FIFTH", 0),
    ("ARISHTA_SATURN_TENTH_MOON_SIXTH_MARS_SEVENTH", 0),
    ("ARISHTA_SATURN_TWELFTH_SUN_NINTH_MARS_EIGHTH", 0),
    ("ARISHTA_SUN_NINTH_MARS_SEVENTH_JUPITER_VENUS_ELEVENTH", 0),
    ("ARISHTA_WANING_MOON_IN_LAGNA_MALEFIC_SEVENTH", 1),
    ("LAGNA_GANDANTA", 0),
    ("NAKSHATRA_GANDANTA", 0),
    ("TITHI_GANDANTA", 0),
];
