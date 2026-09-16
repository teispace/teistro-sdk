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
    assert_eq!(rules.len(), 72);
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
            rule.outcome.is_none() || rule.source.text == "Saravali",
            "{}: an outcome is a span its verse gives",
            rule.key
        );
    }
    // The spans the verses give, in the units they give them in.
    let spans: Vec<f64> = rules
        .iter()
        .filter_map(|rule| rule.outcome.as_ref().and_then(teistro_rules::Outcome::days))
        .collect();
    let graded: Vec<&str> = rules
        .iter()
        .filter(|rule| rule.outcome.is_some())
        .map(|rule| rule.key.as_str())
        .collect();
    assert_eq!(graded, GRADED);
    assert_eq!(spans.len(), 11);
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
    for (_, file) in files_in("doshas") {
        let chart = chart(&file["inputs"]);
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
    assert_eq!((cancelled, saravali_cancelled), (307, 3));
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

/// The rules whose verse says how long the native lives: Saravali ch. 10's
/// evils, each with the span it leaves, and the one antidote of ch. 12 that
/// counts a life in years rather than calling it illimitable.
const GRADED: [&str; 11] = [
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
    "SARAVALI_BHANGA_JUPITER_AND_VENUS_IN_KENDRAS",
];

/// The rules no recorded chart answers.
const SILENT: [&str; 31] = [
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
    "SARAVALI_VENUS_IN_A_LUMINARY_DUSTHANA",
    "TITHI_GANDANTA",
];

/// How many of the 93 recorded charts each rule answers.
const ANSWERED: [(&str, usize); 72] = [
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
    ("SARAVALI_JUPITER_IN_EIGHTH_IN_A_SIGN_OF_MARS", 0),
    ("SARAVALI_MALEFIC_IN_A_VENUS_EIGHTH", 3),
    ("SARAVALI_MARS_SUN_SATURN_IN_TAURUS_AS_EIGHTH", 0),
    ("SARAVALI_MERCURY_IN_CANCER_AS_SIXTH_OR_EIGHTH", 0),
    ("SARAVALI_RETROGRADE_SATURN_IN_A_SIGN_OF_MARS", 0),
    ("SARAVALI_SATURN_ALONE_IN_LAGNA", 2),
    ("SARAVALI_SATURN_IN_LAGNA_ASPECTED_BY_MALEFICS", 1),
    ("SARAVALI_SATURN_IN_LAGNA_WITH_MALEFICS", 1),
    ("SARAVALI_SATURN_WITH_BOTH_LUMINARIES", 2),
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
    assert_eq!(rules.len(), 212);
    let mut pairs: Vec<(&str, &str)> = Vec::new();
    let mut moon: Vec<(&str, &str)> = Vec::new();
    // Jataka Parijata's lists, by how many grahas share the sign.
    let mut together: BTreeMap<usize, usize> = BTreeMap::new();
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
            .outcome
            .as_ref()
            .and_then(teistro_rules::Outcome::text)
            .unwrap_or_else(|| panic!("{}: says in words what follows", rule.key));
        assert!(!text.is_empty() && rule.outcome.as_ref().unwrap().days().is_none());
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

    // What the generator builds is a rule like any other: it writes out in the
    // language and reads back the same, outcome and all.
    let written = serde_json::to_value(rules).unwrap();
    let back: Vec<Rule> = serde_json::from_value(written).unwrap();
    assert_eq!(back, rules);

    let mut fired: BTreeMap<&str, usize> =
        rules.iter().map(|rule| (rule.key.as_str(), 0)).collect();
    let mut charts = 0;
    for (_, file) in files_in("doshas") {
        let chart = chart(&file["inputs"]);
        let evaluator = Evaluator::new(&chart, Readings::RECORDING_ENGINE);
        charts += 1;
        for rule in rules {
            let result = evaluator.evaluate(rule);
            if result.present {
                *fired.get_mut(rule.key.as_str()).unwrap() += 1;
                // A present reading hands on what the rule says, unchanged.
                assert_eq!(result.outcome.as_ref(), rule.outcome.as_ref());
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
    // Every pair of grahas happens somewhere in 93 charts. Eighty-four
    // readings stay silent: 35 of the Moon's 72, among them all six of Aries,
    // where she stands in one chart only and nothing aspects her, and the
    // larger assemblies, which want four, five or six grahas in one sign.
    let silent = counts.iter().filter(|(_, count)| *count == 0).count();
    assert_eq!(silent, 84);
}

/// What each reading answers over the 93 recorded charts.
const READINGS: [(&str, usize); 212] = [
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
];

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
