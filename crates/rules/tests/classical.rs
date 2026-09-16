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

use common::{chart_at, files_in};
use teistro_rules::{Evaluator, Readings, Rule, Tables, shipped};

#[test]
fn every_rule_the_sdk_writes_fires_where_it_did() {
    let rules: Vec<&Rule> = shipped::arishtas()
        .iter()
        .chain(shipped::gandantas())
        .collect();
    assert_eq!(rules.len(), 76);
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
        // Only Saravali grades: no other text says how long the child lives.
        assert!(
            rule.outcomes.is_empty() || rule.source.text == "Saravali",
            "{}: an outcome is a span its verse gives",
            rule.key
        );
    }
    // The spans the verses give, in the units they give them in.
    let spans: Vec<f64> = rules
        .iter()
        .filter_map(|rule| rule.life_span())
        .collect();
    let graded: Vec<&str> = rules
        .iter()
        .filter(|rule| !rule.outcomes.is_empty())
        .map(|rule| rule.key.as_str())
        .collect();
    assert_eq!(graded, GRADED);
    assert_eq!(spans.len(), 13);
    assert!(
        spans
            .iter()
            .all(|days| *days >= 16.0 && *days <= 100.0 * 365.25)
    );
    // The antidotes an evil names are shipped beside it, and nothing loops.
    let owned: Vec<Rule> = rules.iter().map(|rule| (*rule).clone()).collect();
    teistro_rules::check_references(&owned).expect("the pack names itself soundly");
    saravali_evils_name_the_antidotes_of_their_own_text(&rules);
    the_sirshodaya_rule_names_the_catalogue_s_sirshodaya_signs(&rules);

    let mut fired: BTreeMap<&str, usize> =
        rules.iter().map(|rule| (rule.key.as_str(), 0)).collect();
    let mut charts = 0;
    // How many firings an antidote of the same text put out, in all and for
    // Saravali's graded evils alone.
    let (mut cancelled, mut saravali_cancelled) = (0, 0);
    for (path, file) in files_in("doshas") {
        let chart = chart_at(&path, &file["inputs"]);
        let evaluator = Evaluator::new(&chart, Readings::RECORDING_ENGINE).with_rules(&owned);
        charts += 1;
        for rule in &rules {
            let result = evaluator.evaluate(rule);
            if result.present {
                *fired.get_mut(rule.key.as_str()).unwrap() += 1;
                if result.is_cancelled() {
                    cancelled += 1;
                    if rule.source.text == "Saravali" {
                        saravali_cancelled += 1;
                    }
                }
            }
        }
    }
    assert_eq!(charts, 93);
    assert_eq!((cancelled, saravali_cancelled), (324, 8));
    let counts: Vec<(&str, usize)> = fired.into_iter().collect();
    assert_eq!(counts.as_slice(), ANSWERED.as_slice(), "a rule's answers moved");

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

/// Saravali ch. 3 v. 24 names the six signs that rise with their head, and the
/// catalogue carries the same property, so the rule that reads ch. 12's "all
/// the planets in Sirshodaya signs" is held to it rather than to a list typed
/// out twice.
fn the_sirshodaya_rule_names_the_catalogue_s_sirshodaya_signs(rules: &[&Rule]) {
    use teistro_core::catalogue::{Rashi, Rising};

    let rule = rules
        .iter()
        .find(|rule| rule.key == "SARAVALI_BHANGA_ALL_PLANETS_DIRECT_IN_SIRSHODAYA_SIGNS")
        .expect("the pack ships it");
    let named: Vec<Rashi> = rule
        .every_condition()
        .filter_map(|condition| match condition {
            teistro_rules::Condition::PlanetInSign { signs, .. } => Some(signs.clone()),
            _ => None,
        })
        .flatten()
        .collect();
    let catalogued: Vec<Rashi> = Rashi::ALL
        .into_iter()
        .filter(|sign| sign.attributes().rising == Rising::Sirshodaya)
        .collect();
    assert_eq!(named, catalogued);
    assert_eq!(named.len(), 6);
}

/// Saravali ch. 12 counters every evil at birth and ch. 11 those "emanating
/// from, or afflicting the Moon" (v. 1), so each evil of ch. 10 names ch. 12's
/// antidotes, and those whose conditions name the Moon as a body name ch. 11's
/// beside them. The criterion is the rule's own references, not a reading of
/// the verse: Venus in a dusthana owned by the Moon (v. 8) asks for a sign and
/// never for her, so it carries ch. 12's alone.
fn saravali_evils_name_the_antidotes_of_their_own_text(rules: &[&Rule]) {
    let of_chapter = |chapter: &str| -> Vec<&str> {
        rules
            .iter()
            .filter(|rule| {
                rule.category == "arishta-bhanga" && rule.source.chapter.as_deref() == Some(chapter)
            })
            .map(|rule| rule.key.as_str())
            .collect()
    };
    let (eleven, twelve) = (of_chapter("11"), of_chapter("12"));
    assert_eq!((eleven.len(), twelve.len()), (5, 6));
    for rule in rules.iter().filter(|rule| {
        rule.source.text == "Saravali" && rule.category == "arishta"
    }) {
        // Whichever way a condition reaches her — as a subject, as an
        // aspecting body, as the sign counted from — she is written `MOON`.
        let written = serde_json::to_string(&rule.conditions).unwrap();
        let names_the_moon = written.contains("\"MOON\"");
        let mut expected = if names_the_moon {
            eleven.clone()
        } else {
            Vec::new()
        };
        expected.extend(twelve.iter().copied());
        let named: Vec<&str> = rule
            .cancellations
            .iter()
            .map(|cancellation| match &cancellation.condition {
                teistro_rules::Condition::RuleHolds { key } => key.as_str(),
                other => panic!("{}: {other:?} is not an antidote by key", rule.key),
            })
            .collect();
        assert_eq!(named, expected, "{}", rule.key);
    }
}

/// The rules whose verse says what follows: Saravali ch. 10's evils, all but
/// two of them with the span they leave, and the one antidote of ch. 12 that
/// counts a life in years rather than calling it illimitable. Saravali is the
/// only text here that says anything beyond presence at all.
const GRADED: [&str; 15] = [
    "SARAVALI_JUPITER_IN_EIGHTH_IN_A_SIGN_OF_MARS",
    "SARAVALI_RETROGRADE_SATURN_IN_A_SIGN_OF_MARS",
    "SARAVALI_SATURN_WITH_BOTH_LUMINARIES",
    "SARAVALI_MARS_SUN_SATURN_IN_TAURUS_AS_EIGHTH",
    "SARAVALI_MALEFIC_IN_A_VENUS_EIGHTH",
    "SARAVALI_VENUS_IN_A_LUMINARY_DUSTHANA",
    "SARAVALI_MERCURY_IN_CANCER_AS_SIXTH_OR_EIGHTH",
    "SARAVALI_SATURN_IN_LAGNA_ASPECTED_BY_MALEFICS",
    "SARAVALI_SATURN_IN_LAGNA_WITH_MALEFICS",
    "SARAVALI_SATURN_ALONE_IN_LAGNA",
    "SARAVALI_BIRTH_STAR_IS_KETU_S",
    "SARAVALI_SUN_IN_A_TENTH_OF_MARS_OR_SATURN",
    "SARAVALI_RAHU_IN_A_KENDRA_ASPECTED_BY_MALEFICS",
    "SARAVALI_THREE_LORDS_COMBUST",
    "SARAVALI_BHANGA_JUPITER_AND_VENUS_IN_KENDRAS",
];

/// The rules no recorded chart answers.
const SILENT: [&str; 33] = [
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
    "SARAVALI_BHANGA_ALL_PLANETS_DIRECT_IN_SIRSHODAYA_SIGNS",
    "SARAVALI_BHANGA_JUPITER_AND_MOON_IN_CANCER_MERCURY_AND_SATURN_IN_LIBRA",
    "SARAVALI_BHANGA_MERCURY_AND_VENUS_TWELFTH_FROM_THE_MOON",
    "SARAVALI_JUPITER_IN_EIGHTH_IN_A_SIGN_OF_MARS",
    "SARAVALI_MARS_SUN_SATURN_IN_TAURUS_AS_EIGHTH",
    "SARAVALI_MERCURY_IN_CANCER_AS_SIXTH_OR_EIGHTH",
    "SARAVALI_RETROGRADE_SATURN_IN_A_SIGN_OF_MARS",
    "SARAVALI_SUN_IN_A_TENTH_OF_MARS_OR_SATURN",
    "SARAVALI_THREE_LORDS_COMBUST",
    "SARAVALI_VENUS_IN_A_LUMINARY_DUSTHANA",
    "TITHI_GANDANTA",
];

/// How many of the 93 recorded charts each rule answers.
const ANSWERED: [(&str, usize); 76] = [
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
    ("SARAVALI_BHANGA_ALL_PLANETS_DIRECT_IN_SIRSHODAYA_SIGNS", 0),
    ("SARAVALI_BHANGA_BENEFIC_IN_SIXTH_SEVENTH_OR_EIGHTH_FROM_MOON", 48),
    ("SARAVALI_BHANGA_JUPITER_AND_MOON_IN_CANCER_MERCURY_AND_SATURN_IN_LIBRA", 0),
    ("SARAVALI_BHANGA_JUPITER_AND_VENUS_IN_KENDRAS", 6),
    ("SARAVALI_BHANGA_MERCURY_AND_VENUS_TWELFTH_FROM_THE_MOON", 0),
    ("SARAVALI_BHANGA_MOON_ASPECTED_BY_HER_DISPOSITOR", 5),
    ("SARAVALI_BHANGA_MOON_IN_A_BENEFIC_SIGN_ASPECTED_BY_THE_LAGNA_LORD", 4),
    ("SARAVALI_BHANGA_MOON_IN_THE_THIRD_FOURTH_SIXTH_TENTH_OR_ELEVENTH", 5),
    ("SARAVALI_BHANGA_RAHU_IN_LAGNA_IN_ARIES_TAURUS_OR_CANCER", 2),
    ("SARAVALI_BHANGA_RAHU_IN_THE_THIRD_SIXTH_OR_ELEVENTH", 11),
    ("SARAVALI_BHANGA_UNCOMBUST_JUPITER_IN_LAGNA", 5),
    ("SARAVALI_BIRTH_STAR_IS_KETU_S", 3),
    ("SARAVALI_JUPITER_IN_EIGHTH_IN_A_SIGN_OF_MARS", 0),
    ("SARAVALI_MALEFIC_IN_A_VENUS_EIGHTH", 3),
    ("SARAVALI_MARS_SUN_SATURN_IN_TAURUS_AS_EIGHTH", 0),
    ("SARAVALI_MERCURY_IN_CANCER_AS_SIXTH_OR_EIGHTH", 0),
    ("SARAVALI_RAHU_IN_A_KENDRA_ASPECTED_BY_MALEFICS", 26),
    ("SARAVALI_RETROGRADE_SATURN_IN_A_SIGN_OF_MARS", 0),
    ("SARAVALI_SATURN_ALONE_IN_LAGNA", 2),
    ("SARAVALI_SATURN_IN_LAGNA_ASPECTED_BY_MALEFICS", 1),
    ("SARAVALI_SATURN_IN_LAGNA_WITH_MALEFICS", 1),
    ("SARAVALI_SATURN_WITH_BOTH_LUMINARIES", 2),
    ("SARAVALI_SUN_IN_A_TENTH_OF_MARS_OR_SATURN", 0),
    ("SARAVALI_THREE_LORDS_COMBUST", 0),
    ("SARAVALI_VENUS_IN_A_LUMINARY_DUSTHANA", 0),
    ("TITHI_GANDANTA", 0),
];

/// The readings of one shape, built by the generator from its table: every
/// pair of the seven grahas in one sign (Brihat Jataka ch. 14) and the Moon in
/// each sign under each of six aspects (Phaladeepika ch. 18). None of them
/// grades anything, so each says in words what its verse says and carries no
/// severity, no cancellation and no span.
#[test]
fn the_generator_makes_one_rule_a_reading() {
    let rules = shipped::readings();
    assert_eq!(rules.len(), 548);
    every_family_is_whole(rules);
    // What the generator builds is a rule like any other: it writes out in the
    // language and reads back the same, outcome and all.
    let written = serde_json::to_value(rules).unwrap();
    let back: Vec<Rule> = serde_json::from_value(written).unwrap();
    assert_eq!(back, rules);

    let mut fired: BTreeMap<&str, usize> =
        rules.iter().map(|rule| (rule.key.as_str(), 0)).collect();
    let mut charts = 0;
    for (path, file) in files_in("doshas") {
        let chart = chart_at(&path, &file["inputs"]);
        let evaluator = Evaluator::new(&chart, Readings::RECORDING_ENGINE);
        charts += 1;
        for rule in rules {
            let result = evaluator.evaluate(rule);
            if result.present {
                *fired.get_mut(rule.key.as_str()).unwrap() += 1;
                // A present reading hands on what the rule says, unchanged.
                assert_eq!(result.outcomes, rule.outcomes);
            }
        }
    }
    assert_eq!(charts, 93);
    let counts: Vec<(&str, usize)> = fired.into_iter().collect();
    assert_eq!(counts.as_slice(), READINGS.as_slice(), "a reading's answers moved");
    // Varahamihira's pair and Jataka Parijata's are the same figure read
    // twice, so the two answer the same charts, reading for reading.
    let answered = |prefix: &str| -> Vec<usize> {
        counts
            .iter()
            .filter(|(key, _)| key.starts_with(prefix))
            .filter(|(key, _)| key.matches('_').count() == prefix.matches('_').count() + 1)
            .map(|(_, count)| *count)
            .collect()
    };
    let (varahamihira, parijata) = (answered("DWIGRAHA_"), answered("PARIJATA_TOGETHER_"));
    assert_eq!(varahamihira.len(), 21);
    assert_eq!(varahamihira, parijata);
    // Every pair of grahas, and every graha in every sign, happens somewhere
    // in 93 charts. A hundred and fifty-seven readings stay silent: most of
    // them navamsas, a ninth of a sign being a narrow thing to rise in; 35 of
    // the Moon's 72,
    // among them all six of Aries, where she stands in one chart only and
    // nothing aspects her; the larger assemblies, which want four grahas or
    // more in one sign; and two houses no graha of the seven reached.
    let silent = counts.iter().filter(|(_, count)| *count == 0).count();
    assert_eq!(silent, 157);
    // A graha stands in exactly one sign in every chart, so each of Saravali's
    // chapters answers 93 times over the 93.
    // One part of one sign rises in every chart, so each of Saravali's two
    // divisions answers 93 times over the 93 — which says the bands tile a
    // sign with no gap and no overlap.
    for division in ["HORA", "DECANATE", "NAVAMSA"] {
        let total: usize = counts
            .iter()
            .filter(|(key, _)| key.starts_with(&format!("SARAVALI_{division}_")))
            .map(|(_, count)| *count)
            .sum();
        assert_eq!(total, 93, "{division}");
    }
    // A graha stands in exactly one sign and in exactly one house in every
    // chart, so each of Saravali's two families answers 93 times a graha over
    // the 93 — an arithmetic that holds the reader against the corpus.
    let mut by_family: BTreeMap<(&str, bool), usize> = BTreeMap::new();
    for (key, count) in &counts {
        if let Some(rest) = key.strip_prefix("SARAVALI_") {
            let Some((graha, place)) = rest.split_once("_IN_") else {
                continue;
            };
            *by_family
                .entry((graha, place.starts_with("BHAVA_")))
                .or_default() += count;
        }
    }
    assert_eq!(by_family.len(), 14);
    assert!(
        by_family.values().all(|total| *total == 93),
        "{by_family:?}"
    );
}

/// What each reading answers over the 93 recorded charts.
const READINGS: [(&str, usize); 548] = [
    ("CHANDRA_IN_AQUARIUS_ASPECTED_BY_JUPITER", 0),
    ("CHANDRA_IN_AQUARIUS_ASPECTED_BY_MARS", 2),
    ("CHANDRA_IN_AQUARIUS_ASPECTED_BY_MERCURY", 1),
    ("CHANDRA_IN_AQUARIUS_ASPECTED_BY_SATURN", 2),
    ("CHANDRA_IN_AQUARIUS_ASPECTED_BY_SUN", 0),
    ("CHANDRA_IN_AQUARIUS_ASPECTED_BY_VENUS", 0),
    ("CHANDRA_IN_ARIES_ASPECTED_BY_JUPITER", 0),
    ("CHANDRA_IN_ARIES_ASPECTED_BY_MARS", 0),
    ("CHANDRA_IN_ARIES_ASPECTED_BY_MERCURY", 0),
    ("CHANDRA_IN_ARIES_ASPECTED_BY_SATURN", 0),
    ("CHANDRA_IN_ARIES_ASPECTED_BY_SUN", 0),
    ("CHANDRA_IN_ARIES_ASPECTED_BY_VENUS", 0),
    ("CHANDRA_IN_CANCER_ASPECTED_BY_JUPITER", 0),
    ("CHANDRA_IN_CANCER_ASPECTED_BY_MARS", 1),
    ("CHANDRA_IN_CANCER_ASPECTED_BY_MERCURY", 0),
    ("CHANDRA_IN_CANCER_ASPECTED_BY_SATURN", 1),
    ("CHANDRA_IN_CANCER_ASPECTED_BY_SUN", 1),
    ("CHANDRA_IN_CANCER_ASPECTED_BY_VENUS", 2),
    ("CHANDRA_IN_CAPRICORN_ASPECTED_BY_JUPITER", 0),
    ("CHANDRA_IN_CAPRICORN_ASPECTED_BY_MARS", 0),
    ("CHANDRA_IN_CAPRICORN_ASPECTED_BY_MERCURY", 0),
    ("CHANDRA_IN_CAPRICORN_ASPECTED_BY_SATURN", 1),
    ("CHANDRA_IN_CAPRICORN_ASPECTED_BY_SUN", 0),
    ("CHANDRA_IN_CAPRICORN_ASPECTED_BY_VENUS", 0),
    ("CHANDRA_IN_GEMINI_ASPECTED_BY_JUPITER", 0),
    ("CHANDRA_IN_GEMINI_ASPECTED_BY_MARS", 2),
    ("CHANDRA_IN_GEMINI_ASPECTED_BY_MERCURY", 0),
    ("CHANDRA_IN_GEMINI_ASPECTED_BY_SATURN", 4),
    ("CHANDRA_IN_GEMINI_ASPECTED_BY_SUN", 0),
    ("CHANDRA_IN_GEMINI_ASPECTED_BY_VENUS", 1),
    ("CHANDRA_IN_LEO_ASPECTED_BY_JUPITER", 2),
    ("CHANDRA_IN_LEO_ASPECTED_BY_MARS", 3),
    ("CHANDRA_IN_LEO_ASPECTED_BY_MERCURY", 1),
    ("CHANDRA_IN_LEO_ASPECTED_BY_SATURN", 2),
    ("CHANDRA_IN_LEO_ASPECTED_BY_SUN", 0),
    ("CHANDRA_IN_LEO_ASPECTED_BY_VENUS", 0),
    ("CHANDRA_IN_LIBRA_ASPECTED_BY_JUPITER", 1),
    ("CHANDRA_IN_LIBRA_ASPECTED_BY_MARS", 0),
    ("CHANDRA_IN_LIBRA_ASPECTED_BY_MERCURY", 0),
    ("CHANDRA_IN_LIBRA_ASPECTED_BY_SATURN", 2),
    ("CHANDRA_IN_LIBRA_ASPECTED_BY_SUN", 0),
    ("CHANDRA_IN_LIBRA_ASPECTED_BY_VENUS", 0),
    ("CHANDRA_IN_PISCES_ASPECTED_BY_JUPITER", 0),
    ("CHANDRA_IN_PISCES_ASPECTED_BY_MARS", 7),
    ("CHANDRA_IN_PISCES_ASPECTED_BY_MERCURY", 7),
    ("CHANDRA_IN_PISCES_ASPECTED_BY_SATURN", 2),
    ("CHANDRA_IN_PISCES_ASPECTED_BY_SUN", 1),
    ("CHANDRA_IN_PISCES_ASPECTED_BY_VENUS", 7),
    ("CHANDRA_IN_SAGITTARIUS_ASPECTED_BY_JUPITER", 3),
    ("CHANDRA_IN_SAGITTARIUS_ASPECTED_BY_MARS", 2),
    ("CHANDRA_IN_SAGITTARIUS_ASPECTED_BY_MERCURY", 0),
    ("CHANDRA_IN_SAGITTARIUS_ASPECTED_BY_SATURN", 1),
    ("CHANDRA_IN_SAGITTARIUS_ASPECTED_BY_SUN", 2),
    ("CHANDRA_IN_SAGITTARIUS_ASPECTED_BY_VENUS", 0),
    ("CHANDRA_IN_SCORPIO_ASPECTED_BY_JUPITER", 2),
    ("CHANDRA_IN_SCORPIO_ASPECTED_BY_MARS", 2),
    ("CHANDRA_IN_SCORPIO_ASPECTED_BY_MERCURY", 2),
    ("CHANDRA_IN_SCORPIO_ASPECTED_BY_SATURN", 0),
    ("CHANDRA_IN_SCORPIO_ASPECTED_BY_SUN", 0),
    ("CHANDRA_IN_SCORPIO_ASPECTED_BY_VENUS", 0),
    ("CHANDRA_IN_TAURUS_ASPECTED_BY_JUPITER", 1),
    ("CHANDRA_IN_TAURUS_ASPECTED_BY_MARS", 2),
    ("CHANDRA_IN_TAURUS_ASPECTED_BY_MERCURY", 1),
    ("CHANDRA_IN_TAURUS_ASPECTED_BY_SATURN", 1),
    ("CHANDRA_IN_TAURUS_ASPECTED_BY_SUN", 0),
    ("CHANDRA_IN_TAURUS_ASPECTED_BY_VENUS", 1),
    ("CHANDRA_IN_VIRGO_ASPECTED_BY_JUPITER", 3),
    ("CHANDRA_IN_VIRGO_ASPECTED_BY_MARS", 2),
    ("CHANDRA_IN_VIRGO_ASPECTED_BY_MERCURY", 0),
    ("CHANDRA_IN_VIRGO_ASPECTED_BY_SATURN", 1),
    ("CHANDRA_IN_VIRGO_ASPECTED_BY_SUN", 0),
    ("CHANDRA_IN_VIRGO_ASPECTED_BY_VENUS", 0),
    ("DWIGRAHA_JUPITER_SATURN", 24),
    ("DWIGRAHA_JUPITER_VENUS", 6),
    ("DWIGRAHA_MARS_JUPITER", 7),
    ("DWIGRAHA_MARS_MERCURY", 6),
    ("DWIGRAHA_MARS_SATURN", 14),
    ("DWIGRAHA_MARS_VENUS", 23),
    ("DWIGRAHA_MERCURY_JUPITER", 10),
    ("DWIGRAHA_MERCURY_SATURN", 7),
    ("DWIGRAHA_MERCURY_VENUS", 19),
    ("DWIGRAHA_MOON_JUPITER", 12),
    ("DWIGRAHA_MOON_MARS", 6),
    ("DWIGRAHA_MOON_MERCURY", 6),
    ("DWIGRAHA_MOON_SATURN", 3),
    ("DWIGRAHA_MOON_VENUS", 7),
    ("DWIGRAHA_SUN_JUPITER", 6),
    ("DWIGRAHA_SUN_MARS", 13),
    ("DWIGRAHA_SUN_MERCURY", 45),
    ("DWIGRAHA_SUN_MOON", 8),
    ("DWIGRAHA_SUN_SATURN", 8),
    ("DWIGRAHA_SUN_VENUS", 10),
    ("DWIGRAHA_VENUS_SATURN", 8),
    ("PARIJATA_TOGETHER_JUPITER_SATURN", 24),
    ("PARIJATA_TOGETHER_JUPITER_VENUS", 6),
    ("PARIJATA_TOGETHER_JUPITER_VENUS_SATURN", 1),
    ("PARIJATA_TOGETHER_MARS_JUPITER", 7),
    ("PARIJATA_TOGETHER_MARS_JUPITER_SATURN", 4),
    ("PARIJATA_TOGETHER_MARS_JUPITER_VENUS", 0),
    ("PARIJATA_TOGETHER_MARS_JUPITER_VENUS_SATURN", 0),
    ("PARIJATA_TOGETHER_MARS_MERCURY", 6),
    ("PARIJATA_TOGETHER_MARS_MERCURY_JUPITER", 1),
    ("PARIJATA_TOGETHER_MARS_MERCURY_JUPITER_SATURN", 0),
    ("PARIJATA_TOGETHER_MARS_MERCURY_JUPITER_VENUS", 0),
    ("PARIJATA_TOGETHER_MARS_MERCURY_JUPITER_VENUS_SATURN", 0),
    ("PARIJATA_TOGETHER_MARS_MERCURY_SATURN", 1),
    ("PARIJATA_TOGETHER_MARS_MERCURY_VENUS", 3),
    ("PARIJATA_TOGETHER_MARS_MERCURY_VENUS_SATURN", 1),
    ("PARIJATA_TOGETHER_MARS_SATURN", 14),
    ("PARIJATA_TOGETHER_MARS_VENUS", 23),
    ("PARIJATA_TOGETHER_MARS_VENUS_SATURN", 4),
    ("PARIJATA_TOGETHER_MERCURY_JUPITER", 10),
    ("PARIJATA_TOGETHER_MERCURY_JUPITER_SATURN", 2),
    ("PARIJATA_TOGETHER_MERCURY_JUPITER_VENUS", 2),
    ("PARIJATA_TOGETHER_MERCURY_JUPITER_VENUS_SATURN", 0),
    ("PARIJATA_TOGETHER_MERCURY_SATURN", 7),
    ("PARIJATA_TOGETHER_MERCURY_VENUS", 19),
    ("PARIJATA_TOGETHER_MERCURY_VENUS_SATURN", 2),
    ("PARIJATA_TOGETHER_MOON_JUPITER", 12),
    ("PARIJATA_TOGETHER_MOON_JUPITER_SATURN", 0),
    ("PARIJATA_TOGETHER_MOON_JUPITER_VENUS", 1),
    ("PARIJATA_TOGETHER_MOON_JUPITER_VENUS_SATURN", 0),
    ("PARIJATA_TOGETHER_MOON_MARS", 6),
    ("PARIJATA_TOGETHER_MOON_MARS_JUPITER", 1),
    ("PARIJATA_TOGETHER_MOON_MARS_JUPITER_SATURN", 0),
    ("PARIJATA_TOGETHER_MOON_MARS_JUPITER_VENUS", 0),
    ("PARIJATA_TOGETHER_MOON_MARS_JUPITER_VENUS_SATURN", 0),
    ("PARIJATA_TOGETHER_MOON_MARS_MERCURY", 2),
    ("PARIJATA_TOGETHER_MOON_MARS_MERCURY_JUPITER", 1),
    ("PARIJATA_TOGETHER_MOON_MARS_MERCURY_JUPITER_SATURN", 0),
    ("PARIJATA_TOGETHER_MOON_MARS_MERCURY_JUPITER_VENUS", 0),
    ("PARIJATA_TOGETHER_MOON_MARS_MERCURY_JUPITER_VENUS_SATURN", 0),
    ("PARIJATA_TOGETHER_MOON_MARS_MERCURY_SATURN", 0),
    ("PARIJATA_TOGETHER_MOON_MARS_MERCURY_VENUS", 1),
    ("PARIJATA_TOGETHER_MOON_MARS_MERCURY_VENUS_SATURN", 0),
    ("PARIJATA_TOGETHER_MOON_MARS_SATURN", 1),
    ("PARIJATA_TOGETHER_MOON_MARS_VENUS", 2),
    ("PARIJATA_TOGETHER_MOON_MARS_VENUS_SATURN", 0),
    ("PARIJATA_TOGETHER_MOON_MERCURY", 6),
    ("PARIJATA_TOGETHER_MOON_MERCURY_JUPITER", 1),
    ("PARIJATA_TOGETHER_MOON_MERCURY_JUPITER_SATURN", 0),
    ("PARIJATA_TOGETHER_MOON_MERCURY_JUPITER_VENUS", 0),
    ("PARIJATA_TOGETHER_MOON_MERCURY_JUPITER_VENUS_SATURN", 0),
    ("PARIJATA_TOGETHER_MOON_MERCURY_SATURN", 1),
    ("PARIJATA_TOGETHER_MOON_MERCURY_VENUS", 2),
    ("PARIJATA_TOGETHER_MOON_MERCURY_VENUS_SATURN", 1),
    ("PARIJATA_TOGETHER_MOON_SATURN", 3),
    ("PARIJATA_TOGETHER_MOON_VENUS", 7),
    ("PARIJATA_TOGETHER_MOON_VENUS_SATURN", 1),
    ("PARIJATA_TOGETHER_SUN_JUPITER", 6),
    ("PARIJATA_TOGETHER_SUN_JUPITER_SATURN", 3),
    ("PARIJATA_TOGETHER_SUN_JUPITER_VENUS", 1),
    ("PARIJATA_TOGETHER_SUN_JUPITER_VENUS_SATURN", 1),
    ("PARIJATA_TOGETHER_SUN_MARS", 13),
    ("PARIJATA_TOGETHER_SUN_MARS_JUPITER", 1),
    ("PARIJATA_TOGETHER_SUN_MARS_JUPITER_SATURN", 0),
    ("PARIJATA_TOGETHER_SUN_MARS_JUPITER_VENUS", 0),
    ("PARIJATA_TOGETHER_SUN_MARS_JUPITER_VENUS_SATURN", 0),
    ("PARIJATA_TOGETHER_SUN_MARS_MERCURY", 3),
    ("PARIJATA_TOGETHER_SUN_MARS_MERCURY_JUPITER", 1),
    ("PARIJATA_TOGETHER_SUN_MARS_MERCURY_JUPITER_SATURN", 0),
    ("PARIJATA_TOGETHER_SUN_MARS_MERCURY_JUPITER_VENUS", 0),
    ("PARIJATA_TOGETHER_SUN_MARS_MERCURY_JUPITER_VENUS_SATURN", 0),
    ("PARIJATA_TOGETHER_SUN_MARS_MERCURY_SATURN", 0),
    ("PARIJATA_TOGETHER_SUN_MARS_MERCURY_VENUS", 0),
    ("PARIJATA_TOGETHER_SUN_MARS_MERCURY_VENUS_SATURN", 0),
    ("PARIJATA_TOGETHER_SUN_MARS_SATURN", 1),
    ("PARIJATA_TOGETHER_SUN_MARS_VENUS", 0),
    ("PARIJATA_TOGETHER_SUN_MARS_VENUS_SATURN", 0),
    ("PARIJATA_TOGETHER_SUN_MERCURY", 45),
    ("PARIJATA_TOGETHER_SUN_MERCURY_JUPITER", 4),
    ("PARIJATA_TOGETHER_SUN_MERCURY_JUPITER_SATURN", 2),
    ("PARIJATA_TOGETHER_SUN_MERCURY_JUPITER_VENUS", 0),
    ("PARIJATA_TOGETHER_SUN_MERCURY_JUPITER_VENUS_SATURN", 0),
    ("PARIJATA_TOGETHER_SUN_MERCURY_SATURN", 4),
    ("PARIJATA_TOGETHER_SUN_MERCURY_VENUS", 4),
    ("PARIJATA_TOGETHER_SUN_MERCURY_VENUS_SATURN", 1),
    ("PARIJATA_TOGETHER_SUN_MOON", 8),
    ("PARIJATA_TOGETHER_SUN_MOON_JUPITER", 1),
    ("PARIJATA_TOGETHER_SUN_MOON_JUPITER_SATURN", 0),
    ("PARIJATA_TOGETHER_SUN_MOON_JUPITER_VENUS", 0),
    ("PARIJATA_TOGETHER_SUN_MOON_JUPITER_VENUS_SATURN", 0),
    ("PARIJATA_TOGETHER_SUN_MOON_MARS", 2),
    ("PARIJATA_TOGETHER_SUN_MOON_MARS_JUPITER", 1),
    ("PARIJATA_TOGETHER_SUN_MOON_MARS_JUPITER_SATURN", 0),
    ("PARIJATA_TOGETHER_SUN_MOON_MARS_JUPITER_VENUS", 0),
    ("PARIJATA_TOGETHER_SUN_MOON_MARS_JUPITER_VENUS_SATURN", 0),
    ("PARIJATA_TOGETHER_SUN_MOON_MARS_MERCURY", 1),
    ("PARIJATA_TOGETHER_SUN_MOON_MARS_MERCURY_JUPITER", 1),
    ("PARIJATA_TOGETHER_SUN_MOON_MARS_MERCURY_JUPITER_SATURN", 0),
    ("PARIJATA_TOGETHER_SUN_MOON_MARS_MERCURY_JUPITER_VENUS", 0),
    ("PARIJATA_TOGETHER_SUN_MOON_MARS_MERCURY_SATURN", 0),
    ("PARIJATA_TOGETHER_SUN_MOON_MARS_MERCURY_VENUS", 0),
    ("PARIJATA_TOGETHER_SUN_MOON_MARS_MERCURY_VENUS_SATURN", 0),
    ("PARIJATA_TOGETHER_SUN_MOON_MARS_SATURN", 1),
    ("PARIJATA_TOGETHER_SUN_MOON_MARS_VENUS", 0),
    ("PARIJATA_TOGETHER_SUN_MOON_MARS_VENUS_SATURN", 0),
    ("PARIJATA_TOGETHER_SUN_MOON_MERCURY", 3),
    ("PARIJATA_TOGETHER_SUN_MOON_MERCURY_JUPITER", 1),
    ("PARIJATA_TOGETHER_SUN_MOON_MERCURY_JUPITER_SATURN", 0),
    ("PARIJATA_TOGETHER_SUN_MOON_MERCURY_JUPITER_VENUS", 0),
    ("PARIJATA_TOGETHER_SUN_MOON_MERCURY_JUPITER_VENUS_SATURN", 0),
    ("PARIJATA_TOGETHER_SUN_MOON_MERCURY_SATURN", 1),
    ("PARIJATA_TOGETHER_SUN_MOON_MERCURY_VENUS", 1),
    ("PARIJATA_TOGETHER_SUN_MOON_MERCURY_VENUS_SATURN", 1),
    ("PARIJATA_TOGETHER_SUN_MOON_SATURN", 2),
    ("PARIJATA_TOGETHER_SUN_MOON_VENUS", 3),
    ("PARIJATA_TOGETHER_SUN_MOON_VENUS_SATURN", 1),
    ("PARIJATA_TOGETHER_SUN_SATURN", 8),
    ("PARIJATA_TOGETHER_SUN_VENUS", 10),
    ("PARIJATA_TOGETHER_SUN_VENUS_SATURN", 2),
    ("PARIJATA_TOGETHER_VENUS_SATURN", 8),
    ("SARAVALI_DECANATE_1_OF_AQUARIUS", 8),
    ("SARAVALI_DECANATE_1_OF_ARIES", 1),
    ("SARAVALI_DECANATE_1_OF_CANCER", 2),
    ("SARAVALI_DECANATE_1_OF_CAPRICORN", 1),
    ("SARAVALI_DECANATE_1_OF_GEMINI", 3),
    ("SARAVALI_DECANATE_1_OF_LEO", 1),
    ("SARAVALI_DECANATE_1_OF_LIBRA", 2),
    ("SARAVALI_DECANATE_1_OF_PISCES", 2),
    ("SARAVALI_DECANATE_1_OF_SAGITTARIUS", 2),
    ("SARAVALI_DECANATE_1_OF_SCORPIO", 8),
    ("SARAVALI_DECANATE_1_OF_TAURUS", 3),
    ("SARAVALI_DECANATE_1_OF_VIRGO", 2),
    ("SARAVALI_DECANATE_2_OF_AQUARIUS", 2),
    ("SARAVALI_DECANATE_2_OF_ARIES", 0),
    ("SARAVALI_DECANATE_2_OF_CANCER", 0),
    ("SARAVALI_DECANATE_2_OF_CAPRICORN", 1),
    ("SARAVALI_DECANATE_2_OF_GEMINI", 1),
    ("SARAVALI_DECANATE_2_OF_LEO", 5),
    ("SARAVALI_DECANATE_2_OF_LIBRA", 1),
    ("SARAVALI_DECANATE_2_OF_PISCES", 9),
    ("SARAVALI_DECANATE_2_OF_SAGITTARIUS", 5),
    ("SARAVALI_DECANATE_2_OF_SCORPIO", 2),
    ("SARAVALI_DECANATE_2_OF_TAURUS", 1),
    ("SARAVALI_DECANATE_2_OF_VIRGO", 6),
    ("SARAVALI_DECANATE_3_OF_AQUARIUS", 0),
    ("SARAVALI_DECANATE_3_OF_ARIES", 2),
    ("SARAVALI_DECANATE_3_OF_CANCER", 1),
    ("SARAVALI_DECANATE_3_OF_CAPRICORN", 0),
    ("SARAVALI_DECANATE_3_OF_GEMINI", 1),
    ("SARAVALI_DECANATE_3_OF_LEO", 1),
    ("SARAVALI_DECANATE_3_OF_LIBRA", 5),
    ("SARAVALI_DECANATE_3_OF_PISCES", 11),
    ("SARAVALI_DECANATE_3_OF_SAGITTARIUS", 0),
    ("SARAVALI_DECANATE_3_OF_SCORPIO", 2),
    ("SARAVALI_DECANATE_3_OF_TAURUS", 2),
    ("SARAVALI_DECANATE_3_OF_VIRGO", 0),
    ("SARAVALI_HORA_1_OF_AQUARIUS", 10),
    ("SARAVALI_HORA_1_OF_ARIES", 1),
    ("SARAVALI_HORA_1_OF_CANCER", 2),
    ("SARAVALI_HORA_1_OF_CAPRICORN", 2),
    ("SARAVALI_HORA_1_OF_GEMINI", 4),
    ("SARAVALI_HORA_1_OF_LEO", 3),
    ("SARAVALI_HORA_1_OF_LIBRA", 2),
    ("SARAVALI_HORA_1_OF_PISCES", 5),
    ("SARAVALI_HORA_1_OF_SAGITTARIUS", 4),
    ("SARAVALI_HORA_1_OF_SCORPIO", 10),
    ("SARAVALI_HORA_1_OF_TAURUS", 4),
    ("SARAVALI_HORA_1_OF_VIRGO", 7),
    ("SARAVALI_HORA_2_OF_AQUARIUS", 0),
    ("SARAVALI_HORA_2_OF_ARIES", 2),
    ("SARAVALI_HORA_2_OF_CANCER", 1),
    ("SARAVALI_HORA_2_OF_CAPRICORN", 0),
    ("SARAVALI_HORA_2_OF_GEMINI", 1),
    ("SARAVALI_HORA_2_OF_LEO", 4),
    ("SARAVALI_HORA_2_OF_LIBRA", 6),
    ("SARAVALI_HORA_2_OF_PISCES", 17),
    ("SARAVALI_HORA_2_OF_SAGITTARIUS", 3),
    ("SARAVALI_HORA_2_OF_SCORPIO", 2),
    ("SARAVALI_HORA_2_OF_TAURUS", 2),
    ("SARAVALI_HORA_2_OF_VIRGO", 1),
    ("SARAVALI_JUPITER_IN_AQUARIUS", 3),
    ("SARAVALI_JUPITER_IN_ARIES", 8),
    ("SARAVALI_JUPITER_IN_BHAVA_1", 5),
    ("SARAVALI_JUPITER_IN_BHAVA_10", 3),
    ("SARAVALI_JUPITER_IN_BHAVA_11", 0),
    ("SARAVALI_JUPITER_IN_BHAVA_12", 3),
    ("SARAVALI_JUPITER_IN_BHAVA_2", 1),
    ("SARAVALI_JUPITER_IN_BHAVA_3", 12),
    ("SARAVALI_JUPITER_IN_BHAVA_4", 23),
    ("SARAVALI_JUPITER_IN_BHAVA_5", 16),
    ("SARAVALI_JUPITER_IN_BHAVA_6", 5),
    ("SARAVALI_JUPITER_IN_BHAVA_7", 4),
    ("SARAVALI_JUPITER_IN_BHAVA_8", 14),
    ("SARAVALI_JUPITER_IN_BHAVA_9", 7),
    ("SARAVALI_JUPITER_IN_CANCER", 4),
    ("SARAVALI_JUPITER_IN_CAPRICORN", 12),
    ("SARAVALI_JUPITER_IN_GEMINI", 15),
    ("SARAVALI_JUPITER_IN_LEO", 1),
    ("SARAVALI_JUPITER_IN_LIBRA", 3),
    ("SARAVALI_JUPITER_IN_PISCES", 17),
    ("SARAVALI_JUPITER_IN_SAGITTARIUS", 5),
    ("SARAVALI_JUPITER_IN_SCORPIO", 2),
    ("SARAVALI_JUPITER_IN_TAURUS", 12),
    ("SARAVALI_JUPITER_IN_VIRGO", 11),
    ("SARAVALI_MARS_IN_AQUARIUS", 16),
    ("SARAVALI_MARS_IN_ARIES", 4),
    ("SARAVALI_MARS_IN_BHAVA_1", 3),
    ("SARAVALI_MARS_IN_BHAVA_10", 18),
    ("SARAVALI_MARS_IN_BHAVA_11", 5),
    ("SARAVALI_MARS_IN_BHAVA_12", 14),
    ("SARAVALI_MARS_IN_BHAVA_2", 10),
    ("SARAVALI_MARS_IN_BHAVA_3", 8),
    ("SARAVALI_MARS_IN_BHAVA_4", 9),
    ("SARAVALI_MARS_IN_BHAVA_5", 3),
    ("SARAVALI_MARS_IN_BHAVA_6", 12),
    ("SARAVALI_MARS_IN_BHAVA_7", 5),
    ("SARAVALI_MARS_IN_BHAVA_8", 2),
    ("SARAVALI_MARS_IN_BHAVA_9", 4),
    ("SARAVALI_MARS_IN_CANCER", 1),
    ("SARAVALI_MARS_IN_CAPRICORN", 7),
    ("SARAVALI_MARS_IN_GEMINI", 6),
    ("SARAVALI_MARS_IN_LEO", 18),
    ("SARAVALI_MARS_IN_LIBRA", 5),
    ("SARAVALI_MARS_IN_PISCES", 9),
    ("SARAVALI_MARS_IN_SAGITTARIUS", 8),
    ("SARAVALI_MARS_IN_SCORPIO", 13),
    ("SARAVALI_MARS_IN_TAURUS", 4),
    ("SARAVALI_MARS_IN_VIRGO", 2),
    ("SARAVALI_MERCURY_IN_AQUARIUS", 7),
    ("SARAVALI_MERCURY_IN_ARIES", 21),
    ("SARAVALI_MERCURY_IN_BHAVA_1", 5),
    ("SARAVALI_MERCURY_IN_BHAVA_10", 7),
    ("SARAVALI_MERCURY_IN_BHAVA_11", 3),
    ("SARAVALI_MERCURY_IN_BHAVA_12", 6),
    ("SARAVALI_MERCURY_IN_BHAVA_2", 10),
    ("SARAVALI_MERCURY_IN_BHAVA_3", 10),
    ("SARAVALI_MERCURY_IN_BHAVA_4", 5),
    ("SARAVALI_MERCURY_IN_BHAVA_5", 6),
    ("SARAVALI_MERCURY_IN_BHAVA_6", 22),
    ("SARAVALI_MERCURY_IN_BHAVA_7", 8),
    ("SARAVALI_MERCURY_IN_BHAVA_8", 3),
    ("SARAVALI_MERCURY_IN_BHAVA_9", 8),
    ("SARAVALI_MERCURY_IN_CANCER", 14),
    ("SARAVALI_MERCURY_IN_CAPRICORN", 6),
    ("SARAVALI_MERCURY_IN_GEMINI", 1),
    ("SARAVALI_MERCURY_IN_LEO", 3),
    ("SARAVALI_MERCURY_IN_LIBRA", 2),
    ("SARAVALI_MERCURY_IN_PISCES", 7),
    ("SARAVALI_MERCURY_IN_SAGITTARIUS", 9),
    ("SARAVALI_MERCURY_IN_SCORPIO", 6),
    ("SARAVALI_MERCURY_IN_TAURUS", 7),
    ("SARAVALI_MERCURY_IN_VIRGO", 10),
    ("SARAVALI_MOON_IN_AQUARIUS", 4),
    ("SARAVALI_MOON_IN_ARIES", 1),
    ("SARAVALI_MOON_IN_BHAVA_1", 9),
    ("SARAVALI_MOON_IN_BHAVA_10", 0),
    ("SARAVALI_MOON_IN_BHAVA_11", 6),
    ("SARAVALI_MOON_IN_BHAVA_12", 5),
    ("SARAVALI_MOON_IN_BHAVA_2", 5),
    ("SARAVALI_MOON_IN_BHAVA_3", 23),
    ("SARAVALI_MOON_IN_BHAVA_4", 9),
    ("SARAVALI_MOON_IN_BHAVA_5", 4),
    ("SARAVALI_MOON_IN_BHAVA_6", 3),
    ("SARAVALI_MOON_IN_BHAVA_7", 5),
    ("SARAVALI_MOON_IN_BHAVA_8", 12),
    ("SARAVALI_MOON_IN_BHAVA_9", 12),
    ("SARAVALI_MOON_IN_CANCER", 5),
    ("SARAVALI_MOON_IN_CAPRICORN", 8),
    ("SARAVALI_MOON_IN_GEMINI", 6),
    ("SARAVALI_MOON_IN_LEO", 10),
    ("SARAVALI_MOON_IN_LIBRA", 3),
    ("SARAVALI_MOON_IN_PISCES", 13),
    ("SARAVALI_MOON_IN_SAGITTARIUS", 12),
    ("SARAVALI_MOON_IN_SCORPIO", 16),
    ("SARAVALI_MOON_IN_TAURUS", 4),
    ("SARAVALI_MOON_IN_VIRGO", 11),
    ("SARAVALI_NAVAMSA_1_OF_AQUARIUS", 0),
    ("SARAVALI_NAVAMSA_1_OF_ARIES", 0),
    ("SARAVALI_NAVAMSA_1_OF_CANCER", 0),
    ("SARAVALI_NAVAMSA_1_OF_CAPRICORN", 0),
    ("SARAVALI_NAVAMSA_1_OF_GEMINI", 3),
    ("SARAVALI_NAVAMSA_1_OF_LEO", 0),
    ("SARAVALI_NAVAMSA_1_OF_LIBRA", 0),
    ("SARAVALI_NAVAMSA_1_OF_PISCES", 0),
    ("SARAVALI_NAVAMSA_1_OF_SAGITTARIUS", 1),
    ("SARAVALI_NAVAMSA_1_OF_SCORPIO", 0),
    ("SARAVALI_NAVAMSA_1_OF_TAURUS", 1),
    ("SARAVALI_NAVAMSA_1_OF_VIRGO", 1),
    ("SARAVALI_NAVAMSA_2_OF_AQUARIUS", 0),
    ("SARAVALI_NAVAMSA_2_OF_ARIES", 1),
    ("SARAVALI_NAVAMSA_2_OF_CANCER", 1),
    ("SARAVALI_NAVAMSA_2_OF_CAPRICORN", 1),
    ("SARAVALI_NAVAMSA_2_OF_GEMINI", 0),
    ("SARAVALI_NAVAMSA_2_OF_LEO", 0),
    ("SARAVALI_NAVAMSA_2_OF_LIBRA", 1),
    ("SARAVALI_NAVAMSA_2_OF_PISCES", 2),
    ("SARAVALI_NAVAMSA_2_OF_SAGITTARIUS", 0),
    ("SARAVALI_NAVAMSA_2_OF_SCORPIO", 2),
    ("SARAVALI_NAVAMSA_2_OF_TAURUS", 1),
    ("SARAVALI_NAVAMSA_2_OF_VIRGO", 0),
    ("SARAVALI_NAVAMSA_3_OF_AQUARIUS", 8),
    ("SARAVALI_NAVAMSA_3_OF_ARIES", 0),
    ("SARAVALI_NAVAMSA_3_OF_CANCER", 1),
    ("SARAVALI_NAVAMSA_3_OF_CAPRICORN", 0),
    ("SARAVALI_NAVAMSA_3_OF_GEMINI", 0),
    ("SARAVALI_NAVAMSA_3_OF_LEO", 1),
    ("SARAVALI_NAVAMSA_3_OF_LIBRA", 1),
    ("SARAVALI_NAVAMSA_3_OF_PISCES", 0),
    ("SARAVALI_NAVAMSA_3_OF_SAGITTARIUS", 1),
    ("SARAVALI_NAVAMSA_3_OF_SCORPIO", 6),
    ("SARAVALI_NAVAMSA_3_OF_TAURUS", 1),
    ("SARAVALI_NAVAMSA_3_OF_VIRGO", 1),
    ("SARAVALI_NAVAMSA_4_OF_AQUARIUS", 2),
    ("SARAVALI_NAVAMSA_4_OF_ARIES", 0),
    ("SARAVALI_NAVAMSA_4_OF_CANCER", 0),
    ("SARAVALI_NAVAMSA_4_OF_CAPRICORN", 1),
    ("SARAVALI_NAVAMSA_4_OF_GEMINI", 1),
    ("SARAVALI_NAVAMSA_4_OF_LEO", 2),
    ("SARAVALI_NAVAMSA_4_OF_LIBRA", 0),
    ("SARAVALI_NAVAMSA_4_OF_PISCES", 1),
    ("SARAVALI_NAVAMSA_4_OF_SAGITTARIUS", 0),
    ("SARAVALI_NAVAMSA_4_OF_SCORPIO", 2),
    ("SARAVALI_NAVAMSA_4_OF_TAURUS", 0),
    ("SARAVALI_NAVAMSA_4_OF_VIRGO", 1),
    ("SARAVALI_NAVAMSA_5_OF_AQUARIUS", 0),
    ("SARAVALI_NAVAMSA_5_OF_ARIES", 0),
    ("SARAVALI_NAVAMSA_5_OF_CANCER", 0),
    ("SARAVALI_NAVAMSA_5_OF_CAPRICORN", 0),
    ("SARAVALI_NAVAMSA_5_OF_GEMINI", 0),
    ("SARAVALI_NAVAMSA_5_OF_LEO", 1),
    ("SARAVALI_NAVAMSA_5_OF_LIBRA", 0),
    ("SARAVALI_NAVAMSA_5_OF_PISCES", 7),
    ("SARAVALI_NAVAMSA_5_OF_SAGITTARIUS", 2),
    ("SARAVALI_NAVAMSA_5_OF_SCORPIO", 0),
    ("SARAVALI_NAVAMSA_5_OF_TAURUS", 1),
    ("SARAVALI_NAVAMSA_5_OF_VIRGO", 4),
    ("SARAVALI_NAVAMSA_6_OF_AQUARIUS", 0),
    ("SARAVALI_NAVAMSA_6_OF_ARIES", 0),
    ("SARAVALI_NAVAMSA_6_OF_CANCER", 0),
    ("SARAVALI_NAVAMSA_6_OF_CAPRICORN", 0),
    ("SARAVALI_NAVAMSA_6_OF_GEMINI", 0),
    ("SARAVALI_NAVAMSA_6_OF_LEO", 2),
    ("SARAVALI_NAVAMSA_6_OF_LIBRA", 1),
    ("SARAVALI_NAVAMSA_6_OF_PISCES", 1),
    ("SARAVALI_NAVAMSA_6_OF_SAGITTARIUS", 3),
    ("SARAVALI_NAVAMSA_6_OF_SCORPIO", 0),
    ("SARAVALI_NAVAMSA_6_OF_TAURUS", 0),
    ("SARAVALI_NAVAMSA_6_OF_VIRGO", 1),
    ("SARAVALI_NAVAMSA_7_OF_AQUARIUS", 0),
    ("SARAVALI_NAVAMSA_7_OF_ARIES", 0),
    ("SARAVALI_NAVAMSA_7_OF_CANCER", 1),
    ("SARAVALI_NAVAMSA_7_OF_CAPRICORN", 0),
    ("SARAVALI_NAVAMSA_7_OF_GEMINI", 1),
    ("SARAVALI_NAVAMSA_7_OF_LEO", 1),
    ("SARAVALI_NAVAMSA_7_OF_LIBRA", 0),
    ("SARAVALI_NAVAMSA_7_OF_PISCES", 0),
    ("SARAVALI_NAVAMSA_7_OF_SAGITTARIUS", 0),
    ("SARAVALI_NAVAMSA_7_OF_SCORPIO", 1),
    ("SARAVALI_NAVAMSA_7_OF_TAURUS", 0),
    ("SARAVALI_NAVAMSA_7_OF_VIRGO", 0),
    ("SARAVALI_NAVAMSA_8_OF_AQUARIUS", 0),
    ("SARAVALI_NAVAMSA_8_OF_ARIES", 0),
    ("SARAVALI_NAVAMSA_8_OF_CANCER", 0),
    ("SARAVALI_NAVAMSA_8_OF_CAPRICORN", 0),
    ("SARAVALI_NAVAMSA_8_OF_GEMINI", 0),
    ("SARAVALI_NAVAMSA_8_OF_LEO", 0),
    ("SARAVALI_NAVAMSA_8_OF_LIBRA", 0),
    ("SARAVALI_NAVAMSA_8_OF_PISCES", 11),
    ("SARAVALI_NAVAMSA_8_OF_SAGITTARIUS", 0),
    ("SARAVALI_NAVAMSA_8_OF_SCORPIO", 0),
    ("SARAVALI_NAVAMSA_8_OF_TAURUS", 0),
    ("SARAVALI_NAVAMSA_8_OF_VIRGO", 0),
    ("SARAVALI_NAVAMSA_9_OF_AQUARIUS", 0),
    ("SARAVALI_NAVAMSA_9_OF_ARIES", 2),
    ("SARAVALI_NAVAMSA_9_OF_CANCER", 0),
    ("SARAVALI_NAVAMSA_9_OF_CAPRICORN", 0),
    ("SARAVALI_NAVAMSA_9_OF_GEMINI", 0),
    ("SARAVALI_NAVAMSA_9_OF_LEO", 0),
    ("SARAVALI_NAVAMSA_9_OF_LIBRA", 5),
    ("SARAVALI_NAVAMSA_9_OF_PISCES", 0),
    ("SARAVALI_NAVAMSA_9_OF_SAGITTARIUS", 0),
    ("SARAVALI_NAVAMSA_9_OF_SCORPIO", 1),
    ("SARAVALI_NAVAMSA_9_OF_TAURUS", 2),
    ("SARAVALI_NAVAMSA_9_OF_VIRGO", 0),
    ("SARAVALI_SATURN_IN_AQUARIUS", 5),
    ("SARAVALI_SATURN_IN_ARIES", 14),
    ("SARAVALI_SATURN_IN_BHAVA_1", 3),
    ("SARAVALI_SATURN_IN_BHAVA_10", 1),
    ("SARAVALI_SATURN_IN_BHAVA_11", 21),
    ("SARAVALI_SATURN_IN_BHAVA_12", 4),
    ("SARAVALI_SATURN_IN_BHAVA_2", 7),
    ("SARAVALI_SATURN_IN_BHAVA_3", 22),
    ("SARAVALI_SATURN_IN_BHAVA_4", 9),
    ("SARAVALI_SATURN_IN_BHAVA_5", 4),
    ("SARAVALI_SATURN_IN_BHAVA_6", 6),
    ("SARAVALI_SATURN_IN_BHAVA_7", 6),
    ("SARAVALI_SATURN_IN_BHAVA_8", 5),
    ("SARAVALI_SATURN_IN_BHAVA_9", 5),
    ("SARAVALI_SATURN_IN_CANCER", 5),
    ("SARAVALI_SATURN_IN_CAPRICORN", 20),
    ("SARAVALI_SATURN_IN_GEMINI", 3),
    ("SARAVALI_SATURN_IN_LEO", 2),
    ("SARAVALI_SATURN_IN_LIBRA", 1),
    ("SARAVALI_SATURN_IN_PISCES", 4),
    ("SARAVALI_SATURN_IN_SAGITTARIUS", 9),
    ("SARAVALI_SATURN_IN_SCORPIO", 10),
    ("SARAVALI_SATURN_IN_TAURUS", 9),
    ("SARAVALI_SATURN_IN_VIRGO", 11),
    ("SARAVALI_SUN_IN_AQUARIUS", 7),
    ("SARAVALI_SUN_IN_ARIES", 15),
    ("SARAVALI_SUN_IN_BHAVA_1", 4),
    ("SARAVALI_SUN_IN_BHAVA_10", 7),
    ("SARAVALI_SUN_IN_BHAVA_11", 6),
    ("SARAVALI_SUN_IN_BHAVA_12", 4),
    ("SARAVALI_SUN_IN_BHAVA_2", 12),
    ("SARAVALI_SUN_IN_BHAVA_3", 5),
    ("SARAVALI_SUN_IN_BHAVA_4", 13),
    ("SARAVALI_SUN_IN_BHAVA_5", 1),
    ("SARAVALI_SUN_IN_BHAVA_6", 17),
    ("SARAVALI_SUN_IN_BHAVA_7", 14),
    ("SARAVALI_SUN_IN_BHAVA_8", 6),
    ("SARAVALI_SUN_IN_BHAVA_9", 4),
    ("SARAVALI_SUN_IN_CANCER", 11),
    ("SARAVALI_SUN_IN_CAPRICORN", 6),
    ("SARAVALI_SUN_IN_GEMINI", 11),
    ("SARAVALI_SUN_IN_LEO", 8),
    ("SARAVALI_SUN_IN_LIBRA", 3),
    ("SARAVALI_SUN_IN_PISCES", 6),
    ("SARAVALI_SUN_IN_SAGITTARIUS", 13),
    ("SARAVALI_SUN_IN_SCORPIO", 2),
    ("SARAVALI_SUN_IN_TAURUS", 9),
    ("SARAVALI_SUN_IN_VIRGO", 2),
    ("SARAVALI_VENUS_IN_AQUARIUS", 13),
    ("SARAVALI_VENUS_IN_ARIES", 2),
    ("SARAVALI_VENUS_IN_BHAVA_1", 3),
    ("SARAVALI_VENUS_IN_BHAVA_10", 2),
    ("SARAVALI_VENUS_IN_BHAVA_11", 4),
    ("SARAVALI_VENUS_IN_BHAVA_12", 16),
    ("SARAVALI_VENUS_IN_BHAVA_2", 3),
    ("SARAVALI_VENUS_IN_BHAVA_3", 6),
    ("SARAVALI_VENUS_IN_BHAVA_4", 14),
    ("SARAVALI_VENUS_IN_BHAVA_5", 7),
    ("SARAVALI_VENUS_IN_BHAVA_6", 2),
    ("SARAVALI_VENUS_IN_BHAVA_7", 11),
    ("SARAVALI_VENUS_IN_BHAVA_8", 18),
    ("SARAVALI_VENUS_IN_BHAVA_9", 7),
    ("SARAVALI_VENUS_IN_CANCER", 7),
    ("SARAVALI_VENUS_IN_CAPRICORN", 12),
    ("SARAVALI_VENUS_IN_GEMINI", 8),
    ("SARAVALI_VENUS_IN_LEO", 5),
    ("SARAVALI_VENUS_IN_LIBRA", 1),
    ("SARAVALI_VENUS_IN_PISCES", 5),
    ("SARAVALI_VENUS_IN_SAGITTARIUS", 10),
    ("SARAVALI_VENUS_IN_SCORPIO", 4),
    ("SARAVALI_VENUS_IN_TAURUS", 18),
    ("SARAVALI_VENUS_IN_VIRGO", 8),
];

/// Each family holds its own shape: twenty-one distinct pairs, twelve signs of
/// six aspects each, every combination of the seven from two to six once, and
/// each of the seven in each of the twelve signs.
fn every_family_is_whole(rules: &[Rule]) {
    let mut pairs: Vec<(&str, &str)> = Vec::new();
    let mut moon: Vec<(&str, &str)> = Vec::new();
    // Jataka Parijata's lists, by how many grahas share the sign.
    let mut together: BTreeMap<usize, usize> = BTreeMap::new();
    // Saravali's chapters, by the graha each is about.
    let mut in_rasi: BTreeMap<&str, usize> = BTreeMap::new();
    let mut in_bhava: BTreeMap<&str, usize> = BTreeMap::new();
    // Saravali's rising halves and thirds, by the division each cuts.
    let mut rising: BTreeMap<&str, usize> = BTreeMap::new();
    for rule in rules {
        assert!(rule.is_evaluable(), "{} is evaluable", rule.key);
        assert_eq!(
            rule.source.rank.map(teistro_rules::EvidenceRank::get),
            Some(1),
            "{}: read from a text",
            rule.key
        );
        assert!(rule.source.verse.is_some(), "{}: cites a verse", rule.key);
        assert!(rule.source.note.is_some(), "{}: says how it was read", rule.key);
        assert!(
            rule.severity.is_none() && rule.cancellations.is_empty(),
            "{}: a reading grades nothing and nothing cancels it",
            rule.key
        );
        let text = rule
            .effect()
            .unwrap_or_else(|| panic!("{}: says in words what follows", rule.key));
        assert!(!text.is_empty() && rule.life_span().is_none());
        match rule.category.as_str() {
            "dwigraha" => pairs.push(named(&rule.key, "DWIGRAHA_")),
            "chandra-drishti" => moon.push(named(&rule.key, "CHANDRA_IN_")),
            "grahas-together" => {
                let teistro_rules::Condition::PlanetConjunct { planets, .. } = &rule.conditions[0]
                else {
                    panic!("{}: grahas sharing a sign", rule.key)
                };
                *together.entry(planets.len()).or_default() += 1;
            }
            "graha-in-rasi" => *in_rasi.entry(named(&rule.key, "SARAVALI_").0).or_default() += 1,
            "rising-part" => {
                let rest = rule.key.strip_prefix("SARAVALI_").unwrap();
                *rising.entry(rest.split('_').next().unwrap()).or_default() += 1;
            }
            "graha-in-bhava" => {
                let (graha, _) = rule.key.strip_prefix("SARAVALI_").unwrap().split_once("_IN_BHAVA_").unwrap();
                *in_bhava.entry(graha).or_default() += 1;
            }
            other => panic!("{}: {other} is not a family here", rule.key),
        }
    }
    // Every unordered pair of the seven, once each, and every sign under every
    // one of the six aspects.
    assert_eq!(pairs.len(), 21);
    let mut seen = pairs.clone();
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(seen.len(), 21);
    assert_eq!(moon.len(), 72);
    let signs: BTreeMap<&str, usize> = moon.iter().fold(BTreeMap::new(), |mut counted, (sign, _)| {
        *counted.entry(*sign).or_default() += 1;
        counted
    });
    assert_eq!(signs.len(), 12);
    assert!(signs.values().all(|count| *count == 6));
    // Every combination of the seven from two to six, once each: 21, 35, 35,
    // 21 and 7.
    assert_eq!(
        together.into_iter().collect::<Vec<_>>(),
        [(2, 21), (3, 35), (4, 35), (5, 21), (6, 7)]
    );
    // Each of the seven in each of the twelve signs, once.
    assert_eq!(in_rasi.len(), 7);
    assert!(in_rasi.values().all(|count| *count == 12));
    assert_eq!(in_bhava.len(), 7);
    assert!(in_bhava.values().all(|count| *count == 12));
    // Two halves and three thirds of each of the twelve signs.
    assert_eq!(
        rising.into_iter().collect::<Vec<_>>(),
        [("DECANATE", 36), ("HORA", 24), ("NAVAMSA", 108)]
    );

}

/// The two names a generated key holds, after its family's prefix: the pair,
/// or the sign and the graha aspecting.
fn named<'k>(key: &'k str, prefix: &str) -> (&'k str, &'k str) {
    let rest = key.strip_prefix(prefix).unwrap();
    let (one, two) = rest.split_once("_ASPECTED_BY_").unwrap_or_else(|| {
        let at = rest.rfind('_').unwrap();
        (&rest[..at], &rest[at + 1..])
    });
    (one, two)
}

/// The rules that read the two things the kernel learned for Jaimini: the
/// aspect a sign lends (BPHS ch. 26) and the intervention of ch. 31. Neither
/// the recording engine nor the corpus records an answer for any of them, so
/// what is held is that each is evaluable, reads what the chart carries, and
/// answers a pinned number of the 93.
#[test]
fn the_rules_that_read_a_sign_s_aspect_and_an_intervention_answer_where_they_did() {
    let rules: Vec<&Rule> = shipped::nabhasas()
        .iter()
        .filter(|rule| matches!(rule.category.as_str(), "dhana" | "jaimini"))
        .collect();
    assert_eq!(rules.len(), 7);
    // The three that name an intervention read a strength, an intervention
    // being settled by the numbers of grahas or by which is stronger; the two
    // associations and the two gains that only count grahas do not.
    assert_eq!(
        rules.iter().filter(|rule| rule.reads_strength()).count(),
        3
    );
    for rule in &rules {
        assert!(rule.is_evaluable(), "{} is evaluable", rule.key);
        assert_eq!(
            rule.source.rank.map(teistro_rules::EvidenceRank::get),
            Some(1),
            "{}: read from a text",
            rule.key
        );
        assert!(rule.effect().is_some(), "{}: says what follows", rule.key);
    }
    let mut fired: BTreeMap<&str, usize> =
        rules.iter().map(|rule| (rule.key.as_str(), 0)).collect();
    let mut charts = 0;
    for (path, file) in files_in("doshas") {
        let chart = chart_at(&path, &file["inputs"]);
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
    assert_eq!(counts.as_slice(), JAIMINI.as_slice());
    // The verse grades the gains, and each grade asks for more than the one
    // before it, so each answers fewer charts: an intervention, then a
    // benefic's, then an exalted benefic's.
    let at = |key: &str| counts.iter().find(|(k, _)| *k == key).unwrap().1;
    assert!(
        at("ARUDHA_GAINS_WITH_ARGALA")
            >= at("ARUDHA_GAINS_WITH_BENEFIC_ARGALA")
            && at("ARUDHA_GAINS_WITH_BENEFIC_ARGALA")
                >= at("ARUDHA_GAINS_WITH_EXALTED_BENEFIC_ARGALA")
    );
}

/// What each answers over the 93 recorded charts.
const JAIMINI: [(&str, usize); 7] = [
    ("ARUDHA_GAINS", 18),
    ("ARUDHA_GAINS_BENEFIC_ASPECT_FROM_LAGNA_OR_NINTH", 0),
    ("ARUDHA_GAINS_WITH_ARGALA", 16),
    ("ARUDHA_GAINS_WITH_BENEFIC_ARGALA", 14),
    ("ARUDHA_GAINS_WITH_EXALTED_BENEFIC_ARGALA", 1),
    ("JAIMINI_AK_PK_ASSOCIATED", 20),
    ("JAIMINI_LAGNA_AND_FIFTH_LORDS_ASSOCIATED", 26),
];

/// The yogas BPHS ch. 36 grants only to a strong graha, which the kernel could
/// not read until a chart could carry `Strengths`. The corpus records the
/// recording engine's Shadbala for 71 of the 93 charts, so the pass reads both
/// a chart that can answer a question of strength and one that cannot.
#[test]
fn the_yogas_that_ask_for_a_strong_graha_answer_where_they_did() {
    let rules: Vec<&Rule> = shipped::nabhasas()
        .iter()
        .filter(|rule| rule.source.chapter.as_deref() == Some("36") && rule.reads_strength())
        .collect();
    assert_eq!(rules.len(), 6);
    let mut fired: BTreeMap<&str, usize> =
        rules.iter().map(|rule| (rule.key.as_str(), 0)).collect();
    let (mut charts, mut measured, mut silent) = (0, 0, 0);
    for (path, file) in files_in("doshas") {
        let chart = chart_at(&path, &file["inputs"]);
        charts += 1;
        if chart.strengths.is_some() {
            measured += 1;
        }
        let evaluator = Evaluator::new(&chart, Readings::RECORDING_ENGINE);
        for rule in &rules {
            if evaluator.evaluate(rule).present {
                *fired.get_mut(rule.key.as_str()).unwrap() += 1;
            }
        }
        // Stripped of its strengths the same chart can still answer two of
        // these yogas, because Kahala and Sarada each give a second figure
        // that asks for none; nothing else can.
        let mute = teistro_rules::RuleChart {
            strengths: None,
            ..chart
        };
        let mute = Evaluator::new(&mute, Readings::RECORDING_ENGINE);
        let answered: Vec<&str> = rules
            .iter()
            .filter(|rule| mute.evaluate(rule).present)
            .map(|rule| rule.key.as_str())
            .collect();
        assert!(
            answered
                .iter()
                .all(|key| ["PARASHARA_KAHALA", "PARASHARA_SARADA"].contains(key)),
            "{answered:?}"
        );
        silent += usize::from(answered.is_empty());
    }
    assert_eq!((charts, measured, silent), (93, 71, 91));
    let counts: Vec<(&str, usize)> = fired.into_iter().collect();
    assert_eq!(counts.as_slice(), STRENGTH_BOUND.as_slice());
    // Mridanga asks all seven grahas to stand well at once and answers none of
    // the 93, which is what a yoga of that shape should do.
    assert_eq!(counts.iter().filter(|(_, count)| *count == 0).count(), 1);
}

/// What each answers over the 93 recorded charts.
const STRENGTH_BOUND: [(&str, usize); 6] = [
    ("PARASHARA_BHERI", 5),
    ("PARASHARA_KAHALA", 27),
    ("PARASHARA_LAKSHMI", 2),
    ("PARASHARA_MRIDANGA", 0),
    ("PARASHARA_SANKHA", 22),
    ("PARASHARA_SARADA", 2),
];
