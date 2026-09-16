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

use common::{chart, chart_at, files, recorded_chart, rules, strings};
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

/// The engine's key, or keys, for each figure the SDK writes; a figure it does
/// not carry is left out of the comparison.
const SAME: [(&str, &[&str]); 62] = [
    ("PRAVRAJYA_SUN", &["PRAVRAJYA_SUN_TAPASVI"]),
    ("PRAVRAJYA_MOON", &["PRAVRAJYA_MOON_VRIDHHA"]),
    ("PRAVRAJYA_MARS", &["PRAVRAJYA_MARS_SHAKYA"]),
    ("PRAVRAJYA_MERCURY", &["PRAVRAJYA_MERCURY_BHIKSHU"]),
    ("PRAVRAJYA_JUPITER", &["PRAVRAJYA_JUPITER_VYRTAKA"]),
    ("PRAVRAJYA_VENUS", &["PRAVRAJYA_VENUS_CHARAKA"]),
    ("PRAVRAJYA_SATURN", &["PRAVRAJYA_SATURN_NIRGRANTHA"]),
    ("MAHAPURUSHA_RUCHAKA", &["RUCHAKA"]),
    ("MAHAPURUSHA_BHADRA", &["BHADRA"]),
    ("MAHAPURUSHA_HAMSA", &["HAMSA"]),
    ("MAHAPURUSHA_MALAVYA", &["MALAVYA"]),
    ("MAHAPURUSHA_SASA", &["SHASHA"]),
    ("PARASHARA_GAJA_KESARI", &["GAJA_KESARI"]),
    (
        "PARASHARA_AMALA",
        &["AMALA", "AMALA_KIRTI_BENEFIC_10_FROM_MOON"],
    ),
    ("PARASHARA_CHAMARA", &["CHAMARA"]),
    ("PARASHARA_MATSYA", &["MATSYA"]),
    ("PARASHARA_KALANIDHI", &["KALANIDHI"]),
    ("PARASHARA_LAGNADHI", &["LAGNADHI"]),
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
    (
        "BPHS_FIFTH_AND_NINTH_LORDS_RELATED",
        &["RAJA_LORD_CONJ_5_9"],
    ),
    ("BPHS_LORDS_OF_10_AND_5_JOINED", &["RAJA_LORD_CONJ_5_10"]),
    ("BPHS_LORDS_OF_10_AND_9_JOINED", &["RAJA_LORD_CONJ_9_10"]),
    ("BPHS_LORDS_OF_4_AND_5_JOINED", &["RAJA_LORD_CONJ_4_5"]),
    (
        "BPHS_MAHA_RAJA_LAGNA_AND_FIFTH_LORDS_EXCHANGED",
        &["RAJA_1_5"],
    ),
    (
        "BPHS_TENTH_AND_LAGNA_LORDS_EXCHANGED",
        &["MAHA_PARIVARTANA_1_10"],
    ),
    ("NABHASA_DAMA", &["DAMA"]),
    ("NABHASA_VEENA", &["VEENA"]),
];

/// The yogas the SDK reads from BPHS chs. 35 to 39 and 79, against the answers
/// the recording engine recorded for the same figures over the same 93 charts.
#[test]
fn the_sdk_s_yogas_answer_what_the_engine_answered() {
    let ours = teistro_rules::shipped::nabhasas();
    let owned: Vec<Rule> = ours.to_vec();
    let mut tally: std::collections::BTreeMap<&str, (usize, usize, usize)> =
        SAME.iter().map(|(key, _)| (*key, (0, 0, 0))).collect();
    for (path, file) in files() {
        // The SDK's rules ask questions of strength; the engine's never do, so
        // only this pass needs a chart that can answer them.
        let chart = chart_at(&path, &file["inputs"]);
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
    divergences_are_the_ones_read_up_in_the_cruxes(&counted);

    sankhya_yogas_stand_down_where_another_nabhasa_holds(ours, &owned);
}

/// Which figures the SDK and the engine disagree on, and why each is a
/// reading rather than a defect.
fn divergences_are_the_ones_read_up_in_the_cruxes(counted: &[(&str, usize, usize, usize)]) {
    // Forty-six of the sixty-two figures answer exactly alike — among them all
    // five Pancha Mahapurusha yogas, on 49 answers, and BPHS ch. 39 v. 37's
    // angular lord joining a trinal lord, on 42. The two exchanges of ch. 39
    // v. 6 and ch. 40 v. 13 agree only in that no chart here holds either. The
    // sixteen that do not are reading differences, recorded in cruxes C93 to
    // C95, C97 and C100:
    //
    // - The fifth and ninth lords: BPHS ch. 39 vv. 33 and 34 relate them by
    //   mutual aspect or by standing in the seventh from each other as well as
    //   by sharing a sign; the engine keeps the sign. The SDK's answers contain
    //   the engine's, five charts wider.
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
    // - Amala: the verse asks for *exclusively* a benefic in the tenth, and
    //   the engine drops the "no malefic there" half of it.
    // - Gaja Kesari: the verse wants Jupiter in an angle from the ascendant or
    //   the Moon, joined or aspected by another benefic, and neither
    //   debilitated, combust nor in an enemy's sign; the engine keeps only the
    //   angle from the Moon.
    // - Kalanidhi: the verse has Mercury and Venus *aspecting* Jupiter in the
    //   second or fifth; the engine has Mercury sharing his sign.
    // - Chamara: the engine's figure is not the verse's at all — it asks the
    //   lagna lord in an angle with Jupiter in the first, fifth, seventh or
    //   ninth, where the verse asks the lagna lord exalted in an angle under
    //   Jupiter's aspect, or two benefics in one of four houses.
    let (agreeing, diverging): (Vec<&(&str, usize, usize, usize)>, Vec<_>) = counted
        .iter()
        .partition(|(_, _, ours, theirs)| *ours == 0 && *theirs == 0);
    assert_eq!(agreeing.len(), 46);
    assert_eq!(
        diverging.iter().map(|(key, ..)| *key).collect::<Vec<_>>(),
        [
            "BPHS_FIFTH_AND_NINTH_LORDS_RELATED",
            "LUNAR_KEMADRUMA",
            "NABHASA_ARDHA_CHANDRA",
            "NABHASA_KOOTA",
            "NABHASA_SARPA",
            "PARASHARA_AMALA",
            "PARASHARA_CHAMARA",
            "PARASHARA_GAJA_KESARI",
            "PARASHARA_KALANIDHI",
            "PRAVRAJYA_JUPITER",
            "PRAVRAJYA_MARS",
            "PRAVRAJYA_MERCURY",
            "PRAVRAJYA_MOON",
            "PRAVRAJYA_SATURN",
            "PRAVRAJYA_SUN",
            "PRAVRAJYA_VENUS"
        ]
    );
    // The ascetic yogas are the sharpest of the divergences (crux C97). The
    // engine anchors each graha to a house of four and asks nothing else, so
    // it answers 34 chart-rules over the 93 and on one chart says the native
    // enters four holy orders at once. BPHS ch. 79 vv. 2 to 3 ask that all
    // four be strong and give the order of the strongest alone: one chart of
    // the 93, one order, Saturn's.
    let theirs: usize = counted
        .iter()
        .filter(|(key, ..)| key.starts_with("PRAVRAJYA_"))
        .map(|(_, both, _, theirs)| both + theirs)
        .sum();
    let ours: usize = counted
        .iter()
        .filter(|(key, ..)| key.starts_with("PRAVRAJYA_"))
        .map(|(_, both, ours, _)| both + ours)
        .sum();
    assert_eq!((ours, theirs), (1, 34));
    // Every Pancha Mahapurusha yoga reproduces the engine exactly, on 49
    // answers over the 93 charts.
    let mahapurusha: usize = counted
        .iter()
        .filter(|(key, ..)| key.starts_with("MAHAPURUSHA_"))
        .map(|(_, both, ours, theirs)| {
            assert_eq!((*ours, *theirs), (0, 0));
            *both
        })
        .sum();
    assert_eq!(mahapurusha, 49);
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
    for (path, file) in files() {
        let chart = chart_at(&path, &file["inputs"]);
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
const AGREEMENT: [(&str, usize, usize, usize); 62] = [
    ("BPHS_FIFTH_AND_NINTH_LORDS_RELATED", 6, 5, 0),
    ("BPHS_LORDS_OF_10_AND_5_JOINED", 8, 0, 0),
    ("BPHS_LORDS_OF_10_AND_9_JOINED", 16, 0, 0),
    ("BPHS_LORDS_OF_4_AND_5_JOINED", 18, 0, 0),
    ("BPHS_MAHA_RAJA_LAGNA_AND_FIFTH_LORDS_EXCHANGED", 0, 0, 0),
    ("BPHS_TENTH_AND_LAGNA_LORDS_EXCHANGED", 0, 0, 0),
    ("LUNAR_ADHI_YOGA", 0, 0, 0),
    ("LUNAR_ANAPHA", 19, 0, 0),
    ("LUNAR_DURADHARA", 7, 0, 0),
    ("LUNAR_KEMADRUMA", 1, 0, 56),
    ("LUNAR_SUNAPHA", 24, 0, 0),
    ("MAHAPURUSHA_BHADRA", 10, 0, 0),
    ("MAHAPURUSHA_HAMSA", 9, 0, 0),
    ("MAHAPURUSHA_MALAVYA", 11, 0, 0),
    ("MAHAPURUSHA_RUCHAKA", 14, 0, 0),
    ("MAHAPURUSHA_SASA", 5, 0, 0),
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
    ("PARASHARA_AMALA", 10, 0, 4),
    ("PARASHARA_CHAMARA", 2, 15, 12),
    ("PARASHARA_GAJA_KESARI", 12, 2, 13),
    ("PARASHARA_KALANIDHI", 0, 0, 3),
    ("PARASHARA_LAGNADHI", 0, 0, 0),
    ("PARASHARA_MATSYA", 0, 0, 0),
    ("PRAVRAJYA_JUPITER", 0, 0, 4),
    ("PRAVRAJYA_MARS", 0, 0, 4),
    ("PRAVRAJYA_MERCURY", 0, 0, 6),
    ("PRAVRAJYA_MOON", 0, 0, 4),
    ("PRAVRAJYA_SATURN", 1, 0, 5),
    ("PRAVRAJYA_SUN", 0, 0, 6),
    ("PRAVRAJYA_VENUS", 0, 0, 4),
    ("SOLAR_UBHAYACHARI", 33, 0, 0),
    ("SOLAR_VESI", 50, 0, 0),
    ("SOLAR_VOSI", 51, 0, 0),
];

/// The nakshatra the kernel reads from a body's own longitude is the one the
/// corpus recorded for the Moon, chart by chart and pada by pada. The two come
/// by different roads — `planet-in-nakshatra` divides a longitude, and
/// `panchanga-nakshatra` reads what the chart was given — so their agreeing on
/// every one of the 93 charts is what says the division is right.
#[test]
fn the_nakshatra_read_from_a_longitude_is_the_one_the_corpus_recorded() {
    use teistro_core::catalogue::Nakshatra;
    use teistro_rules::{BodyRef, Condition, Pada};

    let moon = BodyRef::Body(Body::Graha(teistro_core::catalogue::Graha::Moon));
    let (mut charts, mut recorded_stars, mut matched) = (0, 0, 0);
    for (_, file) in common::files_in("doshas") {
        let chart = chart(&file["inputs"]);
        charts += 1;
        // Only a chart given a panchanga records a nakshatra to compare
        // against; the rest exercise `panchanga-nakshatra` answering false.
        let evaluator = Evaluator::new(&chart, Readings::RECORDING_ENGINE);
        if chart.panchanga.is_none() {
            continue;
        }
        recorded_stars += 1;
        for nakshatra in Nakshatra::ALL {
            for pada in 1..=4 {
                let padas = vec![Pada::try_new(pada).unwrap()];
                let recorded = Condition::PanchangaNakshatra {
                    nakshatras: vec![nakshatra],
                    padas: padas.clone(),
                };
                let read = Condition::PlanetInNakshatra {
                    planet: moon.clone(),
                    nakshatras: vec![nakshatra],
                    padas,
                };
                let mut into = teistro_rules::Participants::default();
                let one = evaluator.holds(&recorded, &mut into);
                let other = evaluator.holds(&read, &mut into);
                assert_eq!(one, other, "{nakshatra:?} pada {pada}");
                matched += usize::from(one);
            }
        }
    }
    // Exactly one nakshatra and pada holds on each chart that records one.
    assert_eq!((charts, recorded_stars, matched), (93, 77, 77));
}
