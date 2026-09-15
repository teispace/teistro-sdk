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
use teistro_rules::{Body, Evaluator, Readings, Rule};

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
                .map(|i| rule.cancellations[*i].kind().to_owned())
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

#[test]
fn every_rule_round_trips_through_the_language() {
    let rules = rules();
    let back: Vec<Rule> = serde_json::from_value(serde_json::to_value(&rules).unwrap()).unwrap();
    assert_eq!(back, rules);
}
