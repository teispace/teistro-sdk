//! The kernel against the conformance corpus's `baseline/doshas`: every one of
//! the recording engine's natal dosha rules, read strictly from its own
//! `rules.json`, and every one the language can evaluate held, over every
//! recorded chart and panchanga under the engine's dosha reading, to what the
//! engine recorded: each decision, where it was found from, its participants
//! and houses, its severity, the cancellations that held, and its net status.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they read"
)]

mod common;

use common::{chart, files_in, rules_in, strings};
use teistro_rules::{
    Body, Cancellation, Condition, Evaluator, Found, NetStatus, Readings, Rule, Tables,
};

/// How the engine names a graha in a label: its key in title case.
fn display(body: Body) -> String {
    let key = body.key();
    let mut chars = key.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_string() + &chars.as_str().to_ascii_lowercase()
    })
}

/// The label the engine's result names a cancellation by: its own, or the one
/// the engine makes from the condition.
fn label(cancellation: &Cancellation) -> String {
    if let Some(label) = &cancellation.label {
        return label.clone();
    }
    let name_of = |r: &teistro_rules::BodyRef| display(r.named().expect("a named body"));
    match &cancellation.condition {
        Condition::PlanetDignity { planet, dignities } => {
            let list: Vec<String> = dignities
                .iter()
                .map(|d| match d.key() {
                    "OWN_SIGN" => String::from("own"),
                    other => other.to_ascii_lowercase(),
                })
                .collect();
            format!("{} in {} sign", name_of(planet), list.join("/"))
        }
        Condition::PlanetConjunct { planets, .. } => {
            let names: Vec<String> = planets.iter().map(name_of).collect();
            let (last, head) = names.split_last().expect("bodies");
            format!("{} conjunct {last}", head.join(", "))
        }
        Condition::PlanetInHouse { planet, houses } => {
            let teistro_rules::Subject::Ref(reference) = planet else {
                panic!("a named body")
            };
            let houses: Vec<String> = houses.iter().map(|h| h.get().to_string()).collect();
            format!(
                "{} in {}th house",
                display(reference.named().expect("a named body")),
                houses.join("/")
            )
        }
        Condition::LordOfHouseStrong { house_ruled } => {
            format!("Lord of {}th in own/exalted sign", house_ruled.get())
        }
        other => other.kind().to_owned(),
    }
}

fn status(recorded: &str) -> NetStatus {
    match recorded {
        "active" => NetStatus::Active,
        "partially-cancelled" => NetStatus::PartiallyCancelled,
        "fully-cancelled" => NetStatus::FullyCancelled,
        other => panic!("status {other}"),
    }
}

#[test]
fn the_kernel_reproduces_every_recorded_dosha_the_language_can_say() {
    let rules = rules_in("doshas");
    assert_eq!(rules.len(), 52);
    let evaluable: Vec<&Rule> = rules.iter().filter(|r| r.is_evaluable()).collect();
    let computed: Vec<&str> = rules.iter().filter_map(|r| r.computed.as_deref()).collect();
    assert_eq!(
        (evaluable.len(), computed.len()),
        (35, 17),
        "Kalsarpa and its forms, Kala Amrita, Mrityu Bhaga, Dagdha Rashi and Badhaka are computed in code"
    );
    for rule in &rules {
        Tables::EMPTY
            .check(rule)
            .expect("the engine's rules name no table");
    }

    let (mut charts, mut decisions, mut presences, mut with_panchanga) = (0, 0, 0, 0);
    for (path, file) in files_in("doshas") {
        let chart = chart(&file["inputs"]);
        with_panchanga += usize::from(chart.panchanga.is_some());
        let evaluator = Evaluator::new(&chart, Readings::RECORDING_ENGINE_DOSHAS);
        let present = file["present"].as_object().unwrap();
        charts += 1;
        for rule in &evaluable {
            decisions += 1;
            let result = evaluator.evaluate(rule);
            let at = format!("{} {}", path.display(), rule.key);
            let recorded = present.get(&rule.key);
            assert_eq!(result.present, recorded.is_some(), "{at}: presence");
            let Some(recorded) = recorded else {
                continue;
            };
            presences += 1;
            let found: Vec<String> = result
                .found_from
                .iter()
                .map(|found| match found {
                    Found::Conditions => String::from("Lagna"),
                    Found::Group(i) => rule.groups[*i].label.clone(),
                })
                .collect();
            assert_eq!(
                found,
                strings(&recorded["present_from"]),
                "{at}: found from"
            );
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
            assert_eq!(
                result.severity.map(u64::from),
                recorded["severity"].as_u64(),
                "{at}: severity"
            );
            let fired: Vec<String> = result
                .cancellations
                .iter()
                .map(|i| label(&rule.cancellations[*i]))
                .collect();
            assert_eq!(
                fired,
                strings(&recorded["cancellations"]),
                "{at}: cancellations"
            );
            assert_eq!(
                result.status,
                Some(status(recorded["net_status"].as_str().unwrap())),
                "{at}: net status"
            );
        }
    }
    assert_eq!((charts, with_panchanga), (93, 77));
    assert_eq!((decisions, presences), (93 * 35, 885));
}

#[test]
fn every_dosha_rule_round_trips_through_the_language() {
    let rules = rules_in("doshas");
    let back: Vec<Rule> = serde_json::from_value(serde_json::to_value(&rules).unwrap()).unwrap();
    assert_eq!(back, rules);
}
