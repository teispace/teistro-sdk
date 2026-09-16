//! The rules kernel reached from a chart the SDK computed: the corpus's first
//! chart founded with the built-in ephemeris, joined into a `RuleChart`, and
//! read.
//!
//! What this holds is the join itself. The rules crate's own tests read charts
//! the conformance corpus recorded; this one reads a chart the SDK worked out,
//! which is the path a consumer actually takes.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

mod common;

use common::fixture;
use teistro::catalogue::ChartKind;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::rules::{Body, Evaluator, House, Readings, Rule, shipped};
use teistro::{Context, Ephemeris, UtcOffset, rule_chart};

/// The corpus's first chart, founded and stated.
fn founded() -> (teistro::ChartFoundation, Vec<teistro::GrahaState>) {
    let sdk = Context::builder()
        .profile("conformance-baseline")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("the conformance profile and the built-in ephemeris");
    let place = Place::new(
        Latitude::try_new(27.7172).unwrap(),
        Longitude::try_new(85.324).unwrap(),
        Altitude::try_new(1400.0).unwrap(),
    );
    let foundation = sdk
        .chart()
        .found(
            JulianDay::<Utc>::literal(2_447_995.489_583_333_5),
            &place,
            UtcOffset::try_from_seconds(20_700).unwrap(),
            ChartKind::Natal,
        )
        .expect("a foundation")
        .value;
    let states = teistro_state::state(&foundation, sdk.settings()).expect("the graha states");
    (foundation, states)
}

#[test]
fn a_chart_the_sdk_computed_reads_as_a_rule_chart() {
    let (foundation, states) = founded();
    let chart = rule_chart(&foundation, &states, None, None).expect("a rule chart");

    // The lagna is the first house by construction, and its sign is the
    // foundation's own.
    let lagna = chart.placement(Body::Lagna);
    assert_eq!(lagna.house.get(), 1);
    assert_eq!(lagna.sign as u8, foundation.lagna_sign_index());
    assert!(!lagna.retrograde && !lagna.combust);

    // Every graha stands in the house whole signs from the lagna, which is
    // what the rules count by.
    for body in Body::ALL {
        let placement = chart.placement(body);
        assert_eq!(
            placement.house,
            House::between(lagna.sign, placement.sign),
            "{}",
            body.key()
        );
        // The SDK computes no chara karakas, and the bridge leaves what it
        // cannot fill empty rather than defaulting it.
        assert!(placement.karaka7.is_none() && placement.karaka8.is_none());
    }

    // The navamsha the bridge computes is the D9 the conformance corpus
    // recorded for this very chart, body for body — the SDK's own varga
    // against the recording engine's.
    let recorded = fixture("charts/c001-kathmandu-1990-04-14.json");
    for body in Body::ALL {
        assert_eq!(
            recorded["vargas"]["D9"]["sign_index"][body.key()].as_u64(),
            Some(u64::from(chart.placement(body).navamsha as u8)),
            "{}",
            body.key()
        );
    }
}

#[test]
fn a_consumer_can_evaluate_the_shipped_rules_and_read_a_house() {
    let (foundation, states) = founded();
    let chart = rule_chart(&foundation, &states, None, None).expect("a rule chart");
    let rules: Vec<Rule> = shipped::readings()
        .iter()
        .chain(shipped::nabhasas())
        .cloned()
        .collect();
    let evaluator = Evaluator::new(&chart, Readings::TEXTS).with_rules(&rules);

    // Something holds, and everything that holds says what it says in words.
    let held: Vec<&Rule> = rules
        .iter()
        .filter(|rule| evaluator.evaluate(rule).present)
        .collect();
    assert!(!held.is_empty());

    // A house gathers only what stands in it, and the twelve account for the
    // nine grahas between them.
    let readings = evaluator.house_readings(&rules);
    assert_eq!(readings.len(), 12);
    let occupants: usize = readings.iter().map(|reading| reading.occupants.len()).sum();
    assert_eq!(occupants, 9);
    for reading in &readings {
        for gathered in &reading.held {
            assert!(gathered.result.houses.contains(&reading.house));
        }
    }
    // A rule that names a chara karaka cannot answer on this chart, the SDK
    // computing none; that is a stated gap, not a silent false.
    let karaka = shipped::nabhasas()
        .iter()
        .chain(shipped::readings())
        .find(|rule| {
            serde_json::to_string(&rule.conditions)
                .unwrap()
                .contains("karaka")
        });
    if let Some(rule) = karaka {
        assert!(!evaluator.evaluate(rule).present, "{}", rule.key);
    }
}
