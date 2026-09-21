//! The narrative plans reached through the façade
//! (`03-design/plans-at-the-boundary.md`).
//!
//! What this holds is the adaptation: `sdk.interpret()` takes what a chart
//! reading carries and hands it to the composers, and the test is that it
//! adds nothing — a plan composed through the façade and one composed from
//! the kernel's own types are the same items, the same way the rules tests
//! answer a chart by two roads.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

mod common;

use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::rules::{Readings, shipped};
use teistro::{
    ChartRequest, Context, Ephemeris, PlanRequest, RuleInputs, RuleRequest, ShippedRules, UtcOffset,
};

/// The façade composes what the kernel composes. A reading's `present` list
/// is `(rule, result)` pairs and nothing else, so the area's `readings` is a
/// `map`; if it ever became more than that, this test says so.
#[test]
fn a_plan_through_the_facade_is_the_plan_the_composers_write() {
    let set = RuleRequest::shipped([ShippedRules::Nabhasas, ShippedRules::Arishtas])
        .rule_set()
        .expect("the shipped sets");
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
    let request = ChartRequest::at(place, UtcOffset::try_from_seconds(20_700).unwrap())
        .with_rule_inputs(set.rules())
        .with_points();
    let instant = JulianDay::<Utc>::literal(2_447_995.489_583_333_5);
    let read = sdk
        .chart()
        .readings_with_rules(&[instant], &request, &set)
        .expect("a reading answered by rule")
        .value;
    let (document, reading) = &read[0];

    // The other road: the document's own inputs, composed from the kernel.
    let inputs = RuleInputs::of(document).expect("the rules' inputs");
    let evaluator = inputs.evaluator(Readings::TEXTS).with_rules(set.rules());
    let held: Vec<_> = set
        .rules()
        .iter()
        .filter_map(|rule| {
            let result = evaluator.evaluate(rule);
            result.present.then_some((rule, result))
        })
        .collect();
    assert!(
        !held.is_empty(),
        "the chart holds rules to say something of"
    );
    let expected = teistro::interpret::readings(held.iter().map(|(rule, result)| (*rule, result)));
    assert_eq!(sdk.interpret().readings(reading), expected);

    // And the placements, which read the document rather than the rules.
    let placements = sdk
        .interpret()
        .placements(document)
        .expect("the placements");
    assert_eq!(placements, teistro::interpret::placements(&inputs.chart));

    // And the strengths, which this request has without asking: a rule of
    // the set reads strength, so `with_rule_inputs` asked for the Shadbala
    // on its behalf — which is the same rule the boundary follows for a
    // composer's own section.
    let weighed = sdk.interpret().strength(document).expect("the strengths");
    assert_eq!(weighed.len(), 7);
}

/// A request that asks for a reading without rules is refused by name, and
/// a composer that does not exist is refused beside the ones that do — the
/// two dead ends `plans-at-the-boundary.md` §5 names.
#[test]
fn a_reading_without_rules_is_refused_and_so_is_a_composer_that_is_not_one() {
    let request = PlanRequest::from_json(r#"{"placements": true, "readings": true}"#).unwrap();
    assert!(request.check(true).is_ok());
    let refused = request.check(false).unwrap_err();
    assert_eq!(refused.field(), Some("readings"));

    let wrong = PlanRequest::from_json(r#"{"readigns": true}"#).unwrap_err();
    let said = wrong.to_string();
    assert!(
        said.contains("placements") && said.contains("readings"),
        "{said}"
    );

    // Nothing asked for is not an error; it is nothing to do.
    assert!(!PlanRequest::from_json("{}").unwrap().asks_for_something());
}

/// The strengths are the third kind of composer: over a section rather than
/// over the chart's placements or a rule's answer. It says what each graha
/// weighs, strongest first, and says nothing of whether it is strong enough,
/// because no locale carries a message for that.
#[test]
fn the_strengths_are_said_strongest_first_and_claim_nothing_more() {
    let (sdk, document) = common::reading("{}", |request| {
        request
            .with_rule_inputs(shipped::nabhasas())
            .with_shadbala()
    });
    let plan = sdk.interpret().strength(&document).expect("the strengths");
    assert_eq!(plan.len(), 7, "the seven grahas, Sun to Saturn");

    let shadbala = document.shadbala.as_ref().expect("the Shadbala asked for");
    let mut weights: Vec<f64> = shadbala.grahas.iter().map(|graha| graha.rupas).collect();
    weights.sort_by(|a, b| b.total_cmp(a));
    let said: Vec<f64> = plan
        .items
        .iter()
        .filter_map(|item| match item.params.get("score") {
            Some(teistro::Value::Num(score)) => Some(*score),
            _ => None,
        })
        .collect();
    assert_eq!(said, weights, "strongest first");

    // The document carries `strong` and `required_rupas`; the plan does not.
    let written = serde_json::to_string(&plan).expect("a plan writes");
    assert!(!written.contains("strong"), "{written}");
    assert!(!written.contains("required"), "{written}");

    // A document without the section is refused by the knob's name, not
    // answered with an empty plan.
    let (without, unweighed) = common::reading("{}", |request| request);
    let refused = without.interpret().strength(&unweighed).unwrap_err();
    assert_eq!(refused.field(), Some("shadbala"));
}

/// The houses are the fourth composer and the second over a section. It
/// says who rules each bhava — the relation the rest of the tradition is
/// read through, and the one no other composer says — and nothing else,
/// because nothing else a bhava knows has a message in any locale.
#[test]
fn the_houses_are_said_by_their_lords_and_claim_nothing_more() {
    let (sdk, document) = common::reading("{}", ChartRequest::with_houses);
    let plan = sdk.interpret().houses(&document).expect("the houses");
    assert_eq!(plan.len(), 12, "the twelve bhavas, the first house first");

    // Every item is the lord the document's own bhava carries, in the
    // houses' own order: the façade adapts and does not re-derive.
    let read = document.houses.as_ref().expect("the houses asked for");
    let lords: Vec<teistro::Value> = read
        .all()
        .iter()
        .map(|bhava| teistro::Value::catalogued(bhava.lord))
        .collect();
    let said: Vec<teistro::Value> = plan
        .items
        .iter()
        .filter_map(|item| item.params.get("graha").cloned())
        .collect();
    assert_eq!(said, lords);
    let numbers: Vec<teistro::Value> = plan
        .items
        .iter()
        .filter_map(|item| item.params.get("bhava").cloned())
        .collect();
    assert_eq!(numbers[0], teistro::Value::Int(1));
    assert_eq!(numbers[11], teistro::Value::Int(12));

    // The bhava carries its sign, its cusps and its quadrant; the plan
    // claims none of them, because no locale can say them.
    let written = serde_json::to_string(&plan).expect("a plan writes");
    for claim in ["rashi", "madhya", "sandhi", "quadrant"] {
        assert!(!written.contains(claim), "`{claim}` is in {written}");
    }

    // A document without the section is refused by the knob's name, not
    // answered with an empty plan.
    let (without, undivided) = common::reading("{}", |request| request);
    let refused = without.interpret().houses(&undivided).unwrap_err();
    assert_eq!(refused.field(), Some("houses"));
}

/// A plan crosses as its own JSON, and what comes back is what went out:
/// the shape a golden file holds and the boundary's `plans` section carries
/// are one shape, which is what lets a fixture move between them. A plan is
/// its items and nothing else, so it writes as the array it is.
#[test]
fn a_plan_reads_back_from_its_own_json() {
    let (_, document) = common::reading("{}", |request| {
        request.with_rule_inputs(shipped::nabhasas())
    });
    let inputs = RuleInputs::of(&document).expect("the rules' inputs");
    let plan = teistro::interpret::placements(&inputs.chart);
    let json = serde_json::to_string(&plan).expect("a plan writes");
    assert!(json.starts_with(r#"[{"key":"#), "{json}");
    let read: teistro::Plan = serde_json::from_str(&json).expect("a plan reads");
    assert_eq!(read, plan);
}
