//! The kernel against the conformance corpus's `baseline/yogas`: every one of
//! the recording engine's yoga rules, read strictly from its own
//! `rules.json`, evaluated over every recorded chart under the engine's
//! reading, and held to what the engine recorded — each decision, each present
//! rule's participants and houses, and each cancellation that held
//! (`03-design/yogas-measured.md`).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they read"
)]

mod common;

use common::{chart, files, recorded_chart, rules, strings};
use teistro_rules::{Body, Evaluator, Readings, Rule, RuleResult};

#[test]
fn the_kernel_reproduces_every_recorded_yoga() {
    let rules = rules();
    assert_eq!(rules.len(), 605);
    let evaluable = rules.iter().filter(|r| r.is_evaluable()).count();
    assert_eq!(
        evaluable, 597,
        "the Neecha Bhanga family is outside the language"
    );

    let (mut charts, mut decisions, mut presences) = (0, 0, 0);
    for (path, file) in files() {
        let chart = chart(&file["inputs"]);
        // The navamsha the reader computes is the chart's recorded D9, body for
        // body. Not the recorded `is_vargottama`: the engine never sets it for
        // the lagna, though two lagnas here stand in their own navamsha.
        let recorded = recorded_chart(&path);
        for body in Body::ALL {
            assert_eq!(
                recorded["vargas"]["D9"]["sign_index"][body.key()].as_u64(),
                Some(chart.placement(body).navamsha as u64),
                "{} {}: the navamsha",
                path.display(),
                body.key()
            );
        }
        let evaluator = Evaluator::new(&chart, Readings::RECORDING_ENGINE);
        let present = file["present"].as_object().unwrap();
        charts += 1;
        for rule in rules.iter().filter(|r| r.is_evaluable()) {
            decisions += 1;
            let result = evaluator.evaluate(rule);
            explained(&evaluator, rule, &result);
            let at = format!("{} {}", path.display(), rule.key);
            let recorded = present.get(&rule.key);
            assert_eq!(result.present, recorded.is_some(), "{at}: presence");
            let Some(recorded) = recorded else {
                continue;
            };
            presences += 1;
            let participants: Vec<String> = result
                .participants
                .iter()
                .map(|b| b.key().to_owned())
                .collect();
            assert_eq!(
                participants,
                strings(&recorded["planets"]),
                "{at}: participants"
            );
            let houses: Vec<u64> = result.houses.iter().map(|h| u64::from(h.get())).collect();
            let recorded_houses: Vec<u64> = recorded["houses"]
                .as_array()
                .unwrap()
                .iter()
                .map(|h| h.as_u64().unwrap())
                .collect();
            assert_eq!(houses, recorded_houses, "{at}: houses");
            let fired: Vec<String> = result
                .cancellations
                .iter()
                .map(|i| rule.cancellations[*i].condition.kind().to_owned())
                .collect();
            assert_eq!(
                fired,
                strings(&recorded["cancellations"]),
                "{at}: cancellations"
            );
            assert_eq!(
                result.is_cancelled(),
                recorded["cancelled"].as_bool().unwrap()
            );
        }
    }
    assert_eq!((charts, decisions, presences), (93, 55_521, 5350));
}

/// An explanation answers what the evaluation answers, its conditions stop at
/// the first that failed, and under the engine's gathering the bodies its
/// steps added are the participants.
fn explained(evaluator: &Evaluator<'_>, rule: &Rule, result: &RuleResult) {
    let explanation = evaluator.explain(rule);
    assert_eq!(&explanation.result, result, "{}", rule.key);
    let held: Vec<bool> = explanation.conditions.iter().map(|s| s.held).collect();
    let checked = held.iter().take_while(|h| **h).count();
    assert_eq!(
        result.present,
        checked == rule.conditions.len(),
        "{}",
        rule.key
    );
    assert!(
        held.len() == checked || held.len() == checked + 1,
        "{}",
        rule.key
    );
    if result.present {
        let mut added: Vec<Body> = Vec::new();
        for body in explanation.conditions.iter().flat_map(|s| s.added.iter()) {
            if !added.contains(body) {
                added.push(*body);
            }
        }
        assert_eq!(
            added,
            result.participants.iter().collect::<Vec<_>>(),
            "{}",
            rule.key
        );
        assert_eq!(
            explanation.cancellations.len(),
            rule.cancellations.len(),
            "{}",
            rule.key
        );
    } else {
        assert!(explanation.cancellations.is_empty(), "{}", rule.key);
    }
}

/// The rules the SDK writes for the eight the engine computes in code
/// (`03-design/yogas-measured.md`, "The eight, written as rules").
#[test]
fn the_sdk_s_rules_for_the_neecha_bhanga_family_say_present_where_the_engine_s_code_did() {
    let shipped = teistro_rules::shipped::computed_yogas();
    assert_eq!(shipped.len(), 8);
    let computed: Vec<String> = rules()
        .iter()
        .filter(|r| !r.is_evaluable())
        .map(|r| r.key.clone())
        .collect();
    assert_eq!(computed.len(), 8);
    for rule in shipped {
        assert!(computed.contains(&rule.key), "{} is one of them", rule.key);
        assert!(rule.is_evaluable(), "{} is evaluable", rule.key);
    }

    let (mut decisions, mut presences, mut exact) = (0, 0, 0);
    for (path, file) in files() {
        let chart = chart(&file["inputs"]);
        let evaluator = Evaluator::new(&chart, Readings::RECORDING_ENGINE);
        let present = file["present"].as_object().unwrap();
        for rule in shipped {
            decisions += 1;
            let result = evaluator.evaluate(rule);
            let at = format!("{} {}", path.display(), rule.key);
            let recorded = present.get(&rule.key);
            assert_eq!(result.present, recorded.is_some(), "{at}: presence");
            let Some(recorded) = recorded else {
                continue;
            };
            presences += 1;
            let mut ours: Vec<String> = result
                .participants
                .iter()
                .map(|b| b.key().to_owned())
                .collect();
            let mut theirs = strings(&recorded["planets"]);
            // The same debilitated grahas; the aggregate lists them as the
            // chart does and the engine in the order its conditions hit.
            if ours == theirs {
                exact += 1;
            }
            ours.sort();
            theirs.sort();
            assert_eq!(ours, theirs, "{at}: the grahas");
            let houses: Vec<u64> = result.houses.iter().map(|h| u64::from(h.get())).collect();
            let recorded_houses: Vec<u64> = recorded["houses"]
                .as_array()
                .unwrap()
                .iter()
                .map(|h| h.as_u64().unwrap())
                .collect();
            assert_eq!(houses, recorded_houses, "{at}: houses");
        }
    }
    assert_eq!((decisions, presences, exact), (93 * 8, 202, 199));
}

#[test]
fn every_rule_round_trips_through_the_language() {
    let rules = rules();
    let back: Vec<Rule> = serde_json::from_value(serde_json::to_value(&rules).unwrap()).unwrap();
    assert_eq!(back, rules);
}

/// The yogas the SDK reads from BPHS chs. 35, 37 and 38, against the answers
/// the recording engine recorded for the same figures over the same 93 charts.
#[test]
fn the_sdk_s_yogas_answer_what_the_engine_answered() {
    // The engine's key, or keys, for each figure the SDK writes; a figure it
    // does not carry is left out of the comparison.
    const SAME: [(&str, &[&str]); 38] = [
        ("LUNAR_SUNAPHA", &["SUNAPHA"]),
        ("LUNAR_ANAPHA", &["ANAPHA"]),
        ("LUNAR_DURADHARA", &["DURUDHARA"]),
        ("LUNAR_KEMADRUMA", &["KEMADRUMA"]),
        ("LUNAR_ADHI_YOGA", &["ADHI_YOGA"]),
        ("SOLAR_VESI", &["VESI_BROAD"]),
        ("SOLAR_VOSI", &["VOSI_BROAD"]),
        ("SOLAR_UBHAYACHARI", &["UBHAYACHARI_BROAD"]),
        ("NABHASA_RAJJU", &["RAJJU"]),
        ("NABHASA_MUSALA", &["MUSALA"]),
        ("NABHASA_NALA", &["NALA"]),
        ("NABHASA_MAALA", &["MAALA"]),
        ("NABHASA_SARPA", &["SARPA"]),
        ("NABHASA_GADA", &["GADA", "GADA_4_7"]),
        ("NABHASA_SAKATA", &["SAKATA"]),
        ("NABHASA_VIHAGA", &["VIHAGA"]),
        ("NABHASA_SRINGATAKA", &["SHRINGATAKA"]),
        (
            "NABHASA_HALA",
            &["HALA_AKRITI", "HALA_AKRITI_3_7_11", "HALA_AKRITI_4_8_12"],
        ),
        ("NABHASA_KAMALA", &["PADMA"]),
        ("NABHASA_VAPI", &["VAPI"]),
        ("NABHASA_YUPA", &["YUPA"]),
        ("NABHASA_SARA", &["ISHU"]),
        ("NABHASA_SAKTHI", &["SAKTI"]),
        ("NABHASA_DANDA", &["DANDA_AKRITI"]),
        ("NABHASA_NAUKA", &["NAUKHA"]),
        ("NABHASA_KOOTA", &["KUTA"]),
        ("NABHASA_CHATRA", &["CHATRA"]),
        ("NABHASA_CHAPA", &["CHAPA"]),
        ("NABHASA_ARDHA_CHANDRA", &["ARDHA_CHANDRA"]),
        ("NABHASA_CHAKRA", &["CHAKRA"]),
        ("NABHASA_SAMUDRA", &["SAMUDRA"]),
        ("NABHASA_GOLA", &["GOLA"]),
        ("NABHASA_YUGA", &["YUGA"]),
        ("NABHASA_SOOLA", &["SHOOLA"]),
        ("NABHASA_KEDARA", &["KEDARA"]),
        ("NABHASA_PASA", &["PASHA"]),
        ("NABHASA_DAMA", &["DAMA"]),
        ("NABHASA_VEENA", &["VEENA"]),
    ];
    let ours = teistro_rules::shipped::nabhasas();
    let owned: Vec<Rule> = ours.to_vec();
    let mut tally: std::collections::BTreeMap<&str, (usize, usize, usize)> =
        SAME.iter().map(|(key, _)| (*key, (0, 0, 0))).collect();
    for (_, file) in files() {
        let chart = chart(&file["inputs"]);
        let evaluator = Evaluator::new(&chart, Readings::RECORDING_ENGINE).with_rules(&owned);
        let present = file["present"].as_object().unwrap();
        for (key, theirs) in SAME {
            let rule = ours.iter().find(|rule| rule.key == key).unwrap();
            let we = evaluator.evaluate(rule);
            let they = theirs.iter().any(|key| present.contains_key(*key));
            let counted = tally.get_mut(key).unwrap();
            match (we.present, they) {
                (true, true) => counted.0 += 1,
                (true, false) => counted.1 += 1,
                (false, true) => counted.2 += 1,
                (false, false) => {}
            }
        }
    }
    let counted: Vec<(&str, usize, usize, usize)> = tally
        .into_iter()
        .map(|(key, (both, ours, theirs))| (key, both, ours, theirs))
        .collect();
    assert_eq!(counted.as_slice(), AGREEMENT.as_slice());
    // Thirty-four of the thirty-eight figures answer exactly alike. The four
    // that do not are reading differences, recorded in cruxes C93 and C94:
    //
    // - Ardha Chandra: the SDK takes Saravali's "seven continuous houses from
    //   a house that is not an angle", all eight starts; the engine takes the
    //   one window from the second. The SDK's answers contain the engine's.
    // - Koota: the engine asks that all seven houses be occupied, where its
    //   own Nauka, Chatra and Chapa ask only that the seven be confined to
    //   them. The SDK reads the whole family as confinement.
    // - Sarpa: the SDK reads "three angles occupied by malefics" as three
    //   angles, the lagna among them, each holding some malefic; the engine
    //   asks that Mars, Saturn and the Sun each stand in the fourth, seventh
    //   or tenth. The two never agree on these 93 charts.
    // - Kemadruma: the SDK reads the whole of vv. 11 to 13 — no graha but the
    //   Sun with the Moon, in the second or twelfth from her, or in an angle
    //   from the ascendant — where the engine reads the second and twelfth
    //   alone and excepts the nodes besides. One chart of the 93 answers the
    //   verse; fifty-seven answer the engine.
    let (agreeing, diverging): (Vec<&(&str, usize, usize, usize)>, Vec<_>) = counted
        .iter()
        .partition(|(_, _, ours, theirs)| *ours == 0 && *theirs == 0);
    assert_eq!(agreeing.len(), 34);
    assert_eq!(
        diverging.iter().map(|(key, ..)| *key).collect::<Vec<_>>(),
        [
            "LUNAR_KEMADRUMA",
            "NABHASA_ARDHA_CHANDRA",
            "NABHASA_KOOTA",
            "NABHASA_SARPA"
        ]
    );

    sankhya_yogas_stand_down_where_another_nabhasa_holds(ours, &owned);
}

/// Verse 17 holds that no sankhya yoga stands where another Nabhasa yoga is
/// derivable, which the pack writes as cancellations; the kernel reports that
/// as a cancelled result, not as an absent one.
fn sankhya_yogas_stand_down_where_another_nabhasa_holds(ours: &[Rule], owned: &[Rule]) {
    let sankhya = [
        "NABHASA_GOLA",
        "NABHASA_YUGA",
        "NABHASA_SOOLA",
        "NABHASA_KEDARA",
        "NABHASA_PASA",
        "NABHASA_DAMA",
        "NABHASA_VEENA",
    ];
    let (mut present, mut cancelled) = (0, 0);
    for (_, file) in files() {
        let chart = chart(&file["inputs"]);
        let evaluator = Evaluator::new(&chart, Readings::RECORDING_ENGINE).with_rules(owned);
        for key in sankhya {
            let rule = ours.iter().find(|rule| rule.key == key).unwrap();
            let result = evaluator.evaluate(rule);
            if result.present {
                present += 1;
                if result.is_cancelled() {
                    cancelled += 1;
                }
            }
        }
    }
    assert_eq!((present, cancelled), (93, 46));
}

/// How the SDK's reading of each Nabhasa figure stands to the engine's, over
/// the 93 recorded charts: answered by both, by the SDK alone, by the engine
/// alone.
const AGREEMENT: [(&str, usize, usize, usize); 38] = [
    ("LUNAR_ADHI_YOGA", 0, 0, 0),
    ("LUNAR_ANAPHA", 19, 0, 0),
    ("LUNAR_DURADHARA", 7, 0, 0),
    ("LUNAR_KEMADRUMA", 1, 0, 56),
    ("LUNAR_SUNAPHA", 24, 0, 0),
    ("NABHASA_ARDHA_CHANDRA", 17, 13, 0),
    ("NABHASA_CHAKRA", 1, 0, 0),
    ("NABHASA_CHAPA", 2, 0, 0),
    ("NABHASA_CHATRA", 3, 0, 0),
    ("NABHASA_DAMA", 8, 0, 0),
    ("NABHASA_DANDA", 0, 0, 0),
    ("NABHASA_GADA", 0, 0, 0),
    ("NABHASA_GOLA", 0, 0, 0),
    ("NABHASA_HALA", 0, 0, 0),
    ("NABHASA_KAMALA", 0, 0, 0),
    ("NABHASA_KEDARA", 24, 0, 0),
    ("NABHASA_KOOTA", 0, 3, 0),
    ("NABHASA_MAALA", 0, 0, 0),
    ("NABHASA_MUSALA", 0, 0, 0),
    ("NABHASA_NALA", 0, 0, 0),
    ("NABHASA_NAUKA", 18, 0, 0),
    ("NABHASA_PASA", 46, 0, 0),
    ("NABHASA_RAJJU", 0, 0, 0),
    ("NABHASA_SAKATA", 0, 0, 0),
    ("NABHASA_SAKTHI", 0, 0, 0),
    ("NABHASA_SAMUDRA", 1, 0, 0),
    ("NABHASA_SARA", 1, 0, 0),
    ("NABHASA_SARPA", 0, 15, 7),
    ("NABHASA_SOOLA", 6, 0, 0),
    ("NABHASA_SRINGATAKA", 0, 0, 0),
    ("NABHASA_VAPI", 0, 0, 0),
    ("NABHASA_VEENA", 9, 0, 0),
    ("NABHASA_VIHAGA", 0, 0, 0),
    ("NABHASA_YUGA", 0, 0, 0),
    ("NABHASA_YUPA", 0, 0, 0),
    ("SOLAR_UBHAYACHARI", 33, 0, 0),
    ("SOLAR_VESI", 50, 0, 0),
    ("SOLAR_VOSI", 51, 0, 0),
];
