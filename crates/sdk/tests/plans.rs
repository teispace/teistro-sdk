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
    // The façade asks its own locale engine which readings it carries; a
    // context with no readings pack loaded carries none, which is what
    // `NoReadings` says here, so the two must agree item for item.
    let expected = teistro::interpret::readings(
        held.iter().map(|(rule, result)| (*rule, result)),
        &teistro::interpret::NoReadings,
    );
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

/// The positions are the fifth composer. They say what `placements` rounds
/// away — the degree — and are a composer of their own rather than a line
/// inside it, because `grahaAt` subsumes `grahaInRashi` and a plan saying
/// both would name the sign twice a graha.
#[test]
fn the_positions_say_the_degree_placements_rounds_away() {
    let (sdk, document) = common::reading("{}", ChartRequest::with_state);
    let plan = sdk.interpret().positions(&document).expect("the positions");
    assert_eq!(plan.len(), 9, "the nine grahas, the lagna is a point");

    // Every item carries the chart's own longitude as a number: the words
    // and the rounding are the locale's.
    let inputs = RuleInputs::of(&document).expect("the rules' inputs");
    let said: Vec<teistro::Value> = plan
        .items
        .iter()
        .filter_map(|item| item.params.get("longitude").cloned())
        .collect();
    assert_eq!(said.len(), 9);
    assert_eq!(
        said[0],
        teistro::Value::Num(inputs.chart.placements[0].longitude)
    );

    // A plan holds no rendered angle, so the degree signs are the
    // renderer's and never the composer's.
    let written = serde_json::to_string(&plan).expect("a plan writes");
    for rendered in ["\u{b0}", "\u{2032}", "\u{2033}"] {
        assert!(!written.contains(rendered), "{written}");
    }

    // It needs what `placements` needs and nothing more, so a document
    // without the states is refused by the same name.
    let (without, bare) = common::reading("{}", |request| request);
    let refused = without.interpret().positions(&bare).unwrap_err();
    assert_eq!(refused.field(), Some("state"));
}

/// **A consumer's own composer**, written with nothing but the published
/// surface — which is the extensibility table's promise for composers, "a
/// narrative plan function (Rust)", turned from a claim into a test
/// (`02-architecture/08-extensibility.md`).
///
/// Nothing here is a registry and nothing is registered: a composer is a
/// function returning a `Plan`, and a report concatenates what it wants.
/// That is the whole interface, and this test is what says so.
#[test]
fn a_consumer_writes_its_own_composer_with_the_published_surface_alone() {
    use teistro::catalogue::{Graha, Rashi};
    use teistro::messages::sdk::reason;
    use teistro::rules::{Body, RuleChart};
    use teistro::{Item, Plan, TypedMessage};

    /// A consumer's composer: the grahas that share a sign with the Moon,
    /// which no shipped composer says. It reads the same reading the SDK's
    /// own composers read, and emits a key the packs already carry.
    fn with_the_moon(chart: &RuleChart) -> Plan {
        let moon = chart.placements[Body::Graha(Graha::Moon).index()].sign;
        let mut plan = Plan::default();
        for graha in Graha::ALL.into_iter().take(9) {
            let at = chart.placements[Body::Graha(graha).index()];
            if graha != Graha::Moon && at.sign == moon {
                plan.say(&reason::GrahaInRashi {
                    graha,
                    rashi: at.sign,
                });
            }
        }
        plan
    }

    let (sdk, document) = common::reading("{}", ChartRequest::with_state);
    let inputs = RuleInputs::of(&document).expect("the rules' inputs");

    // It composes, it concatenates with the SDK's own, and the whole plan
    // renders — the consumer's items are not second-class.
    let mut plan = sdk.interpret().placements(&document).expect("placements");
    let mine = with_the_moon(&inputs.chart);
    plan.items.extend(mine.items.clone());
    assert!(plan.len() > mine.len());
    for item in &plan {
        let said = sdk.intl().render(&item.key, &item.params);
        assert!(!said.text.is_empty(), "{} said nothing", item.key);
        assert!(!said.is_fallback, "{} fell back", item.key);
    }

    // And the table's validation column, "keys exist", is a property the
    // consumer can check for itself. **Two different failures, and only one
    // of them is a fallback**: `is_fallback` says a *fallback locale*
    // answered, while a key no locale carries at all leaves `resolved_from`
    // empty. A consumer checking the first would never see the second,
    // which is why this test names both.
    let slots = reason::GrahaInRashi {
        graha: Graha::Moon,
        rashi: Rashi::Aries,
    }
    .params();
    let invented = Item::new("sdk.reason.notAMessage", slots);
    assert!(!sdk.intl().has(&invented.key), "asked before rendering");
    let said = sdk.intl().render(&invented.key, &invented.params);
    assert_eq!(said.resolved_from, None, "no locale answered it");
    assert!(!said.is_fallback, "nothing fell back: nothing answered");

    // The key a composer does emit is carried by the strict locale itself,
    // so it neither falls back nor goes unanswered.
    sdk.intl()
        .set_locale("ne-Deva-NP")
        .expect("a strict locale");
    let carried = Item::of(&reason::GrahaInRashi {
        graha: Graha::Moon,
        rashi: Rashi::Aries,
    });
    let said = sdk.intl().render(&carried.key, &carried.params);
    assert_eq!(said.resolved_from.as_deref(), Some("ne-Deva-NP"));
    assert!(!said.is_fallback, "a strict locale carries it itself");
}

/// The drishtis are the sixth composer and **the first whose messages were
/// written for it**: no locale carried a word for an aspect, so `sdk.aspect`
/// was added in English and Nepali from the tradition's own vocabulary. This
/// says the plan is composed; that both locales can say it is the measured
/// page's business, and it holds every item of all 93 recorded charts.
#[test]
fn the_drishtis_are_said_with_the_messages_written_for_them() {
    let (sdk, document) = common::reading("{}", ChartRequest::with_aspects);
    let plan = sdk.interpret().aspects(&document).expect("the aspects");
    assert!(!plan.is_empty(), "every chart holds a drishti");

    let read = document.aspects.as_ref().expect("the aspects asked for");
    let casts = plan
        .items
        .iter()
        .filter(|item| item.key == "sdk.aspect.cast")
        .count();
    assert_eq!(casts, read.all().len(), "one cast a relation");
    let pairs = plan
        .items
        .iter()
        .filter(|item| item.key == "sdk.aspect.mutual")
        .count();
    assert_eq!(pairs, read.mutual().count(), "one item a mutual pair");

    // Every item renders from each strict locale's own message: the point
    // of writing the pack in both rather than one.
    for locale in ["en-Latn", "ne-Deva-NP"] {
        sdk.intl().set_locale(locale).expect("a strict locale");
        for item in &plan {
            let said = sdk.intl().render(&item.key, &item.params);
            assert_eq!(said.resolved_from.as_deref(), Some(locale), "{}", item.key);
            assert!(!said.is_fallback, "{} fell back in {locale}", item.key);
            assert!(said.warnings.is_empty(), "{:?}", said.warnings);
        }
    }

    // The edges say how near a boundary the reading stands, which is a fact
    // about the ayanamsha rather than about the native; no locale carries it
    // and the plan claims none of it.
    let written = serde_json::to_string(&plan).expect("a plan writes");
    for claim in ["edge", "sign_deg", "nakshatra_deg", "houses"] {
        assert!(!written.contains(claim), "`{claim}` is in {written}");
    }

    // A document without the section is refused by the knob's name.
    let (without, bare) = common::reading("{}", |request| request);
    let refused = without.interpret().aspects(&bare).unwrap_err();
    assert_eq!(refused.field(), Some("aspects"));
}

/// The conditions are the seventh composer. They close the six facts of a
/// placement that `placements` and `positions` leave unsaid — four of them
/// here and the two chara karakas in `karakas` — and they read the same
/// graha states, so a document that can be placed can be conditioned.
#[test]
fn the_conditions_say_what_a_graha_is_where_it_stands() {
    let (sdk, document) = common::reading("{}", ChartRequest::with_state);
    let plan = sdk
        .interpret()
        .conditions(&document)
        .expect("the conditions");
    let inputs = RuleInputs::of(&document).expect("the rules' inputs");
    let placed = &inputs.chart.placements[..9];

    // A dignity and a navamsha for each of the nine, and the three
    // conditions that are absences only where they hold.
    let of = |key: &str| plan.items.iter().filter(|item| item.key == key).count();
    assert_eq!(of("sdk.condition.dignity"), 9, "sama is a dignity too");
    assert_eq!(of("sdk.condition.navamsha"), 9);
    assert_eq!(
        of("sdk.condition.vargottama"),
        placed.iter().filter(|at| at.navamsha == at.sign).count()
    );
    assert_eq!(
        of("sdk.condition.retrograde"),
        placed.iter().filter(|at| at.retrograde).count()
    );
    assert_eq!(
        of("sdk.condition.combust"),
        placed.iter().filter(|at| at.combust).count()
    );
    let named = of("sdk.condition.vargottama")
        + of("sdk.condition.retrograde")
        + of("sdk.condition.combust");
    assert_eq!(plan.len(), 18 + named, "nothing said that was not counted");

    // A dignity crosses as the catalogue's own key, not as a rendered word,
    // which is what lets a locale carrying nothing but `sdk.entity` say it.
    let said = plan.items[0].params.get("dignity").cloned();
    assert_eq!(said, Some(teistro::Value::catalogued(placed[0].dignity)));

    for locale in ["en-Latn", "ne-Deva-NP"] {
        sdk.intl().set_locale(locale).expect("a strict locale");
        for item in &plan {
            let said = sdk.intl().render(&item.key, &item.params);
            assert_eq!(said.resolved_from.as_deref(), Some(locale), "{}", item.key);
            assert!(!said.is_fallback, "{} fell back in {locale}", item.key);
            assert!(said.warnings.is_empty(), "{:?}", said.warnings);
        }
    }

    // It needs what `placements` needs and nothing more.
    let (without, bare) = common::reading("{}", |request| request);
    let refused = without.interpret().conditions(&bare).unwrap_err();
    assert_eq!(refused.field(), Some("state"));
}

/// The karakas are the eighth composer, and the only one that says the same
/// fact twice on purpose: the two chara karaka schemes disagree about half
/// the time, so emitting one would choose for the consumer. The key says
/// which scheme, so filtering by key gives one whole.
#[test]
fn the_karakas_say_both_schemes_because_they_disagree() {
    let (sdk, document) = common::reading("{}", ChartRequest::with_state);
    let plan = sdk.interpret().karakas(&document).expect("the karakas");
    let inputs = RuleInputs::of(&document).expect("the rules' inputs");
    let placed = &inputs.chart.placements[..9];

    let of = |key: &str| plan.items.iter().filter(|item| item.key == key).count();
    assert_eq!(
        of("sdk.karaka.ofSeven"),
        placed.iter().filter(|at| at.karaka7.is_some()).count()
    );
    assert_eq!(
        of("sdk.karaka.ofEight"),
        placed.iter().filter(|at| at.karaka8.is_some()).count()
    );
    assert_eq!(of("sdk.karaka.ofSeven"), 7, "the Sun to Saturn");
    assert_eq!(of("sdk.karaka.ofEight"), 8, "and Rahu besides");

    for locale in ["en-Latn", "ne-Deva-NP"] {
        sdk.intl().set_locale(locale).expect("a strict locale");
        for item in &plan {
            let said = sdk.intl().render(&item.key, &item.params);
            assert_eq!(said.resolved_from.as_deref(), Some(locale), "{}", item.key);
            assert!(!said.is_fallback, "{} fell back in {locale}", item.key);
            assert!(said.warnings.is_empty(), "{:?}", said.warnings);
        }
    }

    let (without, bare) = common::reading("{}", |request| request);
    let refused = without.interpret().karakas(&bare).unwrap_err();
    assert_eq!(refused.field(), Some("state"));
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
