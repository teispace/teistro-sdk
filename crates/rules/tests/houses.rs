//! What a house says when everything bearing on it is gathered, over the
//! conformance corpus's recorded charts (`03-design/rules-engine.md`, "What a
//! house says when several grahas share it").

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
use teistro_core::catalogue::Graha;
use teistro_rules::{Body, Evaluator, House, Kind, Readings, Rule, shipped};

/// Everything the SDK writes from a text, which is what a consumer would ask a
/// house of.
fn rules() -> Vec<Rule> {
    shipped::arishtas()
        .iter()
        .chain(shipped::gandantas())
        .chain(shipped::nabhasas())
        .chain(shipped::readings())
        .cloned()
        .collect()
}

/// Reading the twelve houses at once gives what reading them one at a time
/// gives, and evaluates each rule once instead of twelve times.
#[test]
fn reading_the_whole_chart_gives_what_reading_each_house_gives() {
    let rules = rules();
    for (path, file) in files_in("doshas").into_iter().take(12) {
        let chart = chart_at(&path, &file["inputs"]);
        let evaluator = Evaluator::new(&chart, Readings::RECORDING_ENGINE).with_rules(&rules);
        let whole = evaluator.house_readings(&rules);
        for reading in &whole {
            assert_eq!(*reading, evaluator.house_reading(reading.house, &rules));
        }
    }
}

#[test]
fn a_house_gathers_every_rule_whose_grahas_stand_in_it() {
    let rules = rules();
    let mut occupied: BTreeMap<usize, usize> = BTreeMap::new();
    let (mut charts, mut houses, mut held, mut composed) = (0, 0, 0, 0);
    for (path, file) in files_in("doshas") {
        let chart = chart_at(&path, &file["inputs"]);
        let evaluator = Evaluator::new(&chart, Readings::RECORDING_ENGINE).with_rules(&rules);
        charts += 1;
        let readings = evaluator.house_readings(&rules);
        assert_eq!(readings.len(), 12);
        for reading in &readings {
            houses += 1;
            held += reading.held.len();
            composed += reading.composition.len();
            *occupied.entry(reading.occupants.len()).or_default() += 1;
            // Its sign is the one the chart gives that house, and its
            // occupants are exactly the grahas standing in that sign.
            assert_eq!(reading.sign, evaluator.house_sign(reading.house));
            let standing: Vec<Body> = Body::nine()
                .into_iter()
                .filter(|body| chart.placement(*body).sign == reading.sign)
                .collect();
            assert_eq!(reading.occupants, standing);
            for gathered in &reading.held {
                // Nothing is gathered under a house its grahas do not stand
                // in, and nothing gathered is absent.
                assert!(gathered.result.present, "{}", gathered.rule.key);
                assert!(
                    gathered.result.houses.contains(&reading.house),
                    "{} under house {}",
                    gathered.rule.key,
                    reading.house.get()
                );
            }
            // A composition bears only where its own conditions are met.
            for composition in &reading.composition {
                assert!(
                    composition.houses.is_empty()
                        || composition.houses.contains(&reading.house.get())
                );
                if !composition.strong {
                    assert!(reading.occupants.len() >= usize::from(composition.at_least));
                }
                assert_eq!(
                    composition
                        .source()
                        .rank
                        .map(teistro_rules::EvidenceRank::get),
                    Some(1)
                );
            }
        }
        // Every graha stands in exactly one house, so the twelve readings of a
        // chart account for the nine between them.
        let counted: usize = readings.iter().map(|reading| reading.occupants.len()).sum();
        assert_eq!(counted, 9);
    }
    assert_eq!((charts, houses), (93, 93 * 12));
    // How crowded the recorded houses are, and what the corpus therefore
    // exercises: the composition rules bear only where two or more share a
    // house, and the refusal only where three or more share an angle.
    let crowded: Vec<(usize, usize)> = occupied.into_iter().collect();
    assert_eq!(crowded, OCCUPANTS);
    // Forty-eight of the 1116 houses hold three grahas or more — the case a
    // reading of one graha at a time cannot answer, and the case the texts
    // decline to write.
    let crowded_houses: usize = crowded
        .iter()
        .filter(|(occupants, _)| *occupants >= 3)
        .map(|(_, houses)| *houses)
        .sum();
    assert_eq!(crowded_houses, 48);
    // What a consumer receives: 5731 rule results gathered under a house over
    // the 93 charts, and 455 statements of how to read them together.
    assert_eq!((held, composed), (5731, 455));
}

/// How many houses of the 1116 hold each number of grahas.
const OCCUPANTS: [(usize, usize); 6] = [(0, 547), (1, 365), (2, 156), (3, 35), (4, 10), (5, 3)];

/// The texts say four things about reading a crowded house together and refuse
/// a fifth; each is a verse, and the refusal is as much a result as the rest.
#[test]
fn the_compositions_are_what_the_texts_say_and_one_of_them_is_a_refusal() {
    let all = teistro_rules::COMPOSITIONS;
    assert_eq!(all.len(), 6);
    let kinds: Vec<Kind> = all.iter().map(|composition| composition.kind).collect();
    assert_eq!(
        kinds,
        [
            Kind::Compose,
            Kind::Arbitrate,
            Kind::Arbitrate,
            Kind::Modulate,
            Kind::Arbitrate,
            Kind::Refuse
        ]
    );
    for composition in &all {
        let source = composition.source();
        assert_eq!(source.rank.map(teistro_rules::EvidenceRank::get), Some(1));
        assert!(source.chapter.is_some() && source.verse.is_some());
        assert!(!composition.says.is_empty());
        assert!(
            composition
                .houses
                .iter()
                .all(|house| (1..=12).contains(house))
        );
    }
    // The one refusal is Saravali's, and it bears only on the four angles it
    // declined to write.
    let refusal = all
        .iter()
        .find(|composition| composition.kind == Kind::Refuse)
        .unwrap();
    assert_eq!(refusal.houses, &[1, 4, 7, 10]);
    assert_eq!(refusal.at_least, 3);
    assert!(!refusal.bears_on(House::try_new(4).unwrap(), 2, 0));
    assert!(refusal.bears_on(House::try_new(4).unwrap(), 3, 0));
    assert!(!refusal.bears_on(House::try_new(5).unwrap(), 5, 0));
    // The ascetic order's is the only one that asks for strength, so a house
    // of four grahas none of them strong does not reach it.
    let ascetic = all
        .iter()
        .find(|composition| composition.key == "ARBITRATE_THE_ASCETIC_ORDER_BY_THE_STRONGEST")
        .unwrap();
    assert!(!ascetic.bears_on(House::try_new(1).unwrap(), 4, 3));
    assert!(ascetic.bears_on(House::try_new(1).unwrap(), 4, 4));
}

/// Saravali ch. 34 reads named sets of grahas in named houses — three of them
/// in the second, two in the seventh — which is the shape a reading of one
/// graha at a time cannot say (crux C96). Where such a rule holds, it is
/// gathered under exactly the house it names.
#[test]
fn a_named_set_in_a_named_house_is_gathered_under_that_house() {
    let rules: Vec<Rule> = shipped::nabhasas()
        .iter()
        .filter(|rule| {
            rule.source.text == "Saravali" && rule.source.chapter.as_deref() == Some("34")
        })
        .cloned()
        .collect();
    // Fourteen here and the three counts shipped earlier — the whole of what
    // ch. 34 gives that a one-graha-at-a-time reading cannot.
    assert_eq!(rules.len(), 17);
    let mut fired: BTreeMap<&str, usize> =
        rules.iter().map(|rule| (rule.key.as_str(), 0)).collect();
    let mut gathered = 0;
    for (path, file) in files_in("doshas") {
        let chart = chart_at(&path, &file["inputs"]);
        let evaluator = Evaluator::new(&chart, Readings::RECORDING_ENGINE).with_rules(&rules);
        for rule in &rules {
            if evaluator.evaluate(rule).present {
                *fired.get_mut(rule.key.as_str()).unwrap() += 1;
            }
        }
        // A rule reaches a house through the grahas that stand there, so one
        // gathered under a house has at least one participant standing in it.
        // Not *every* participant: a rule whose grahas stand in two houses is
        // gathered under both, which is why the second from the Moon —
        // aspected by a benefic standing elsewhere — appears under the
        // benefic's house and not the Moon's second.
        for reading in evaluator.house_readings(&rules) {
            for held in &reading.held {
                gathered += 1;
                assert!(
                    held.result
                        .participants
                        .iter()
                        .any(|body| reading.occupants.contains(&body) || body == Body::Lagna),
                    "{} under house {}",
                    held.rule.key,
                    reading.house.get()
                );
            }
        }
    }
    let counts: Vec<(&str, usize)> = fired.into_iter().collect();
    assert_eq!(counts.as_slice(), NAMED_SETS.as_slice());
    assert_eq!(gathered, 117);
    // The named triple in the second never happens in these 93 births, and
    // the named pair in the seventh happens seven times: that is how rare the
    // shape is, which is why the corpus carries so little of it.
    let named_in_a_house: usize = counts
        .iter()
        .filter(|(key, _)| key.contains("_IN_THE_SECOND") || key.contains("_IN_THE_SEVENTH"))
        .map(|(_, count)| *count)
        .sum();
    assert_eq!(named_in_a_house, 13);
}

/// What each answers over the 93 recorded charts.
const NAMED_SETS: [(&str, usize); 17] = [
    ("SARAVALI_A_LEO_ARIES_OR_SCORPIO_NAVAMSA_RISING", 20),
    ("SARAVALI_BENEFICS_IN_THE_SECOND", 5),
    ("SARAVALI_ENEMIES_BY_THE_SIXTH", 54),
    ("SARAVALI_JUPITER_IN_THE_SECOND_ASPECTED_BY_MERCURY", 0),
    ("SARAVALI_LAGNADHI_FROM_THE_SIXTH", 0),
    ("SARAVALI_MARS_AND_SUN_IN_THE_SECOND", 0),
    ("SARAVALI_MARS_SATURN_AND_SUN_IN_THE_SECOND", 0),
    (
        "SARAVALI_MARS_SATURN_AND_SUN_IN_THE_SECOND_UNDER_A_WEAK_MOON",
        0,
    ),
    ("SARAVALI_MERCURY_AND_JUPITER_IN_THE_SEVENTH", 0),
    ("SARAVALI_MERCURY_AND_VENUS_IN_THE_SEVENTH", 7),
    ("SARAVALI_MERCURY_IN_THE_SECOND_ASPECTED_BY_THE_MOON", 0),
    ("SARAVALI_SATURN_ALONE_IN_THE_SECOND_ASPECTED_BY_MERCURY", 0),
    ("SARAVALI_SUN_IN_THE_SECOND_ASPECTED_BY_SATURN_ALONE", 1),
    (
        "SARAVALI_THE_SECOND_FROM_THE_MOON_ASPECTED_BY_A_BENEFIC",
        24,
    ),
    ("SARAVALI_THREE_GRAHAS_IN_THE_LAGNA", 1),
    ("SARAVALI_THREE_MALEFICS_IN_THE_LAGNA", 0),
    ("SARAVALI_WEAK_MOON_IN_THE_SECOND_ASPECTED_BY_MERCURY", 0),
];

/// The fate of the corpse is read from the twenty-second decanate, which is
/// the eighth house's sign at the third of it the lagna stands in. Exactly one
/// of the three lord-kinds answers on any chart, because a sign has one lord
/// and every graha is a benefic, a malefic or one of the mixed pair.
#[test]
fn exactly_one_fate_of_the_corpse_answers_a_chart() {
    let rules: Vec<Rule> = shipped::nabhasas()
        .iter()
        .filter(|rule| rule.category == "ayur")
        .cloned()
        .collect();
    assert_eq!(rules.len(), 4);
    let mut by_lord = 0;
    let mut serpents = 0;
    for (path, file) in files_in("doshas") {
        let chart = chart_at(&path, &file["inputs"]);
        let evaluator = Evaluator::new(&chart, Readings::TEXTS).with_rules(&rules);
        let held: Vec<&str> = rules
            .iter()
            .filter(|rule| evaluator.evaluate(rule).present)
            .map(|rule| rule.key.as_str())
            .collect();
        let kinds = held
            .iter()
            .filter(|key| !key.ends_with("IS_A_SERPENT"))
            .count();
        assert_eq!(kinds, 1, "{}: {held:?}", path.display());
        by_lord += kinds;
        serpents += held.len() - kinds;
    }
    // The serpent reading is a second answer where it holds, the verse giving
    // it beside the lord's and not instead of it.
    assert_eq!((by_lord, serpents), (93, 7));
}

/// BPHS ch. 41's combinations for wealth, whose figures are so particular that
/// each holds for only one or two ascendants. The pass holds what each answers
/// and the arithmetic the chapter's own formula implies: a great-affluence
/// yoga wants the lord of the fifth in the fifth, so no chart can answer two
/// of the seven at once.
#[test]
fn the_wealth_combinations_answer_where_they_did() {
    let rules: Vec<Rule> = shipped::nabhasas()
        .iter()
        .filter(|rule| rule.source.text == "BPHS" && rule.source.chapter.as_deref() == Some("41"))
        .cloned()
        .collect();
    assert_eq!(rules.len(), 14);
    let mut fired: BTreeMap<&str, usize> =
        rules.iter().map(|rule| (rule.key.as_str(), 0)).collect();
    for (path, file) in files_in("doshas") {
        let chart = chart_at(&path, &file["inputs"]);
        let evaluator = Evaluator::new(&chart, Readings::TEXTS).with_rules(&rules);
        let mut great = 0;
        for rule in &rules {
            if evaluator.evaluate(rule).present {
                *fired.get_mut(rule.key.as_str()).unwrap() += 1;
                if rule
                    .source
                    .verse
                    .as_deref()
                    .is_some_and(|verse| matches!(verse, "2" | "3" | "4" | "5" | "6" | "7" | "8"))
                {
                    great += 1;
                }
            }
        }
        // Each of vv. 2 to 8 puts a different graha in the fifth in its own
        // sign, and one sign is the fifth, so at most one can hold.
        assert!(great <= 1, "{}: {great}", path.display());
    }
    let counts: Vec<(&str, usize)> = fired.into_iter().collect();
    assert_eq!(counts.as_slice(), WEALTH.as_slice());
}

/// What each answers over the 93 recorded charts.
const WEALTH: [(&str, usize); 14] = [
    (
        "BPHS_JUPITER_IN_A_FIFTH_OF_HIS_OWN_WITH_MERCURY_ELEVENTH",
        0,
    ),
    ("BPHS_JUPITER_RISING_IN_HIS_OWN_SIGN", 0),
    ("BPHS_MARS_IN_A_FIFTH_OF_HIS_OWN_WITH_VENUS_ELEVENTH", 0),
    ("BPHS_MARS_RISING_IN_HIS_OWN_SIGN", 0),
    ("BPHS_MERCURY_IN_A_FIFTH_OF_HIS_OWN_WITH_THREE_ELEVENTH", 0),
    ("BPHS_MERCURY_RISING_IN_HIS_OWN_SIGN", 1),
    (
        "BPHS_SATURN_IN_A_FIFTH_OF_HIS_OWN_WITH_THE_LUMINARIES_ELEVENTH",
        0,
    ),
    ("BPHS_SATURN_RISING_IN_HIS_OWN_SIGN", 0),
    (
        "BPHS_THE_MOON_IN_CANCER_AS_THE_FIFTH_WITH_SATURN_ELEVENTH",
        0,
    ),
    ("BPHS_THE_MOON_RISING_IN_CANCER", 0),
    ("BPHS_THE_SUN_IN_LEO_AS_THE_FIFTH_WITH_THREE_ELEVENTH", 0),
    ("BPHS_THE_SUN_RISING_IN_LEO", 0),
    ("BPHS_VENUS_IN_A_FIFTH_OF_HIS_OWN_WITH_MARS_ELEVENTH", 0),
    ("BPHS_VENUS_RISING_IN_HIS_OWN_SIGN", 0),
];

/// BPHS ch. 42's combinations for penury, which read the marakas as a class.
/// The pass holds what each answers, and the one partition the chapter writes:
/// v. 17 reads the Sun in the second both ways, so its two halves together
/// answer exactly where the Sun stands in the second.
#[test]
fn the_penury_combinations_answer_where_they_did() {
    let rules: Vec<Rule> = shipped::nabhasas()
        .iter()
        .filter(|rule| rule.source.text == "BPHS" && rule.source.chapter.as_deref() == Some("42"))
        .cloned()
        .collect();
    assert_eq!(rules.len(), 16);
    let mut fired: BTreeMap<&str, usize> =
        rules.iter().map(|rule| (rule.key.as_str(), 0)).collect();
    let (mut sun_in_second, mut cancelled) = (0, 0);
    for (path, file) in files_in("doshas") {
        let chart = chart_at(&path, &file["inputs"]);
        let evaluator = Evaluator::new(&chart, Readings::TEXTS).with_rules(&rules);
        for rule in &rules {
            let result = evaluator.evaluate(rule);
            if result.present {
                *fired.get_mut(rule.key.as_str()).unwrap() += 1;
                if !result.cancellations.is_empty() {
                    cancelled += 1;
                }
            }
        }
        let sun = evaluator.chart().placement(Body::Graha(Graha::Sun));
        if evaluator.house_sign(House::try_new(2).unwrap()) == sun.sign {
            sun_in_second += 1;
        }
    }
    let halves = fired["BPHS_SUN_IN_SECOND_ASPECTED_BY_SATURN"]
        + fired["BPHS_SUN_IN_SECOND_UNASPECTED_BY_SATURN"];
    assert_eq!(halves, sun_in_second);
    // Mercury never aspects Mars and Saturn together in the second here, so
    // v. 16's cancellation is held by the evaluator's own tests, not this one.
    assert_eq!((sun_in_second, cancelled), (12, 0));
    let counts: Vec<(&str, usize)> = fired.into_iter().collect();
    assert_eq!(counts.as_slice(), PENURY.as_slice());
}

/// What each answers over the 93 recorded charts. The two that answer on
/// two charts in five are the verses' own arithmetic, measured: a class of
/// three or four marakas is easy to join, and the corpus's lagna lords
/// cluster where they cast a special aspect on the twelfth (Jupiter in the
/// fourth twelve times, Saturn in the third eight).
const PENURY: [(&str, usize); 16] = [
    (
        "BPHS_DISPOSITORS_OF_DUSTHANA_LORDS_AFFLICTED_IN_DUSTHANAS",
        1,
    ),
    (
        "BPHS_EIGHTH_OR_TWELFTH_ASPECTED_BY_KARAKAMSHA_LORD_AND_LAGNA_LORD",
        18,
    ),
    (
        "BPHS_FIFTH_LORD_IN_SIXTH_AND_NINTH_LORD_IN_TWELFTH_ASPECTED_BY_MARAKAS",
        0,
    ),
    ("BPHS_LAGNA_AND_NAVAMSHA_LAGNA_LORDS_WITH_MARAKAS", 21),
    ("BPHS_LAGNA_AND_SIXTH_LORDS_EXCHANGED_WITH_A_MARAKA", 0),
    ("BPHS_LAGNA_AND_TWELFTH_LORDS_EXCHANGED_WITH_A_MARAKA", 0),
    (
        "BPHS_LAGNA_LORD_WITH_A_DUSTHANA_LORD_OR_SATURN_UNASPECTED_BY_BENEFICS",
        25,
    ),
    (
        "BPHS_LAGNA_LORD_WITH_A_MALEFIC_IN_A_DUSTHANA_AND_SECOND_LORD_INIMICAL",
        0,
    ),
    ("BPHS_LAGNA_OR_MOON_WITH_KETU_AND_LAGNA_LORD_IN_EIGHTH", 5),
    ("BPHS_MALEFIC_IN_LAGNA_WITH_A_MARAKA", 19),
    ("BPHS_MARS_AND_SATURN_IN_SECOND", 0),
    (
        "BPHS_MOON_NAVAMSHA_LORD_WITH_A_MARAKA_OR_IN_A_MARAKA_HOUSE",
        40,
    ),
    ("BPHS_SATURN_IN_SECOND_ASPECTED_BY_SUN", 0),
    ("BPHS_SUN_IN_SECOND_ASPECTED_BY_SATURN", 2),
    ("BPHS_SUN_IN_SECOND_UNASPECTED_BY_SATURN", 10),
    (
        "BPHS_TWELFTH_FROM_ATMAKARAKA_OR_LAGNA_ASPECTED_BY_ITS_LORD",
        37,
    ),
];
