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
use teistro::catalogue::DashaSystem;
use teistro::catalogue::Varga;
use teistro::quantity::Depth;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::rules::{Body, Evaluator, House, Karaka, Readings, Rule, shipped};
use teistro::rules::{Levels, Timing};
use teistro::{
    Context, Ephemeris, RuleInputs, Timeline, UtcOffset, rule_chart, rule_periods, rule_vargas,
};

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
    }

    // The navamsha the bridge computes, and every division ch. 39 of BPHS
    // reads the lagna in, are the vargas the conformance corpus recorded for
    // this very chart, body for body — the SDK's own against the recording
    // engine's.
    let recorded = fixture("charts/c001-kathmandu-1990-04-14.json");
    let divisions = rule_vargas(
        &chart,
        [Varga::D2, Varga::D3, Varga::D9, Varga::D12, Varga::D30],
    )
    .expect("the divisions");
    for body in Body::ALL {
        let recorded_sign =
            |varga: &str| recorded["vargas"][varga]["sign_index"][body.key()].as_u64();
        assert_eq!(
            recorded_sign("D9"),
            Some(u64::from(chart.placement(body).navamsha as u8)),
            "{}",
            body.key()
        );
        for division in &divisions {
            assert_eq!(
                recorded_sign(division.varga.key()),
                Some(u64::from(division.signs[body.index()] as u8)),
                "{} in {}",
                body.key(),
                division.varga.key()
            );
        }
    }

    // The chara karakas are the ones the corpus recorded for the chart: the
    // seven-karaka scheme, computed from the SDK's own longitudes.
    let karakas = fixture("doshas/charts/c001-kathmandu-1990-04-14.json");
    for body in Body::ALL {
        let karaka = chart
            .placement(body)
            .karaka7
            .map(|karaka| serde_json::to_value(Karaka(karaka)).unwrap());
        assert_eq!(
            karaka.as_ref(),
            karakas["inputs"]["chara_karaka_7"].get(body.key()),
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
    // A rule that names a chara karaka is read on this chart, the bridge
    // computing them: the Atmakaraka's own navamsha is the Karakamsha.
    let karakamsha: Rule = serde_json::from_str(
        r#"{"key": "KARAKAMSHA", "category": "jaimini", "source": {"text": "BPHS"},
            "conditions": [{"type": "same-sign", "of": {"navamsha": {"karaka": "AK"}},
                            "as": {"navamsha": "MERCURY"}}]}"#,
    )
    .unwrap();
    assert!(evaluator.evaluate(&karakamsha).present);
}

#[test]
fn a_result_says_which_of_the_sdk_s_own_dasha_periods_deliver_it() {
    let (foundation, states) = founded();
    let chart = rule_chart(&foundation, &states, None, None).expect("a rule chart");
    let rules = shipped::nabhasas();
    let evaluator = Evaluator::new(&chart, Readings::TEXTS).with_rules(rules);
    let (sdk, document) = common::reading("{}", |request| {
        request.with_dashas([DashaSystem::Vimshottari])
    });
    let cursor = sdk
        .chart()
        .dasha(&document, DashaSystem::Vimshottari)
        .expect("the Vimshottari");

    // BPHS ch. 41 v. 16: wealth in the periods of the ninth and fifth lords
    // and of the grahas joining either, read off the chart directly.
    let wealth = rules
        .iter()
        .find(|rule| rule.key == "BPHS_NINTH_AND_FIFTH_LORDS_AND_THEIR_COMPANIONS_GIVE_WEALTH")
        .unwrap();
    let result = evaluator.evaluate(wealth);
    let lord_of = |house: u8| {
        evaluator
            .house_sign(House::try_new(house).unwrap())
            .attributes()
            .lord
    };
    let sign_of = |graha| chart.placement(Body::Graha(graha)).sign;
    let givers = [lord_of(9), lord_of(5)];
    let throughout = rules
        .iter()
        .find(|rule| rule.timing == Timing::Throughout && evaluator.evaluate(rule).present)
        .expect("a Nabhasa yoga present");
    let nabhasa = evaluator.evaluate(throughout);

    let depth = Depth::try_new(2).unwrap();
    let mut delivering = 0;
    for mahadasha in cursor.mahadashas() {
        let middle = f64::midpoint(mahadasha.interval.from.get(), mahadasha.interval.to.get());
        let chain = cursor.at(teistro::quantity::JulianDay::literal(middle), depth);
        assert_eq!(
            chain.iter().next().map(|period| period.lord),
            Some(mahadasha.lord)
        );
        let giver = givers
            .iter()
            .any(|lord| *lord == mahadasha.lord || sign_of(*lord) == sign_of(mahadasha.lord));
        let levels = evaluator.delivery(wealth, &result, rule_periods(&chain));
        assert_eq!(levels.contains(0), giver, "{:?}", mahadasha.lord);
        delivering += usize::from(giver);
        assert_eq!(
            evaluator.delivery(throughout, &nabhasa, rule_periods(&chain)),
            Levels::all(chain.len())
        );
    }
    assert!(delivering >= 2, "the two lords' own mahadashas at least");
}

/// The rules' reading of every chart in the corpus, computed by the SDK from
/// the birth each records, against what the corpus recorded for the rules:
/// each body's sign, dignity, motion and combustion, and each graha's karakas.
/// What differs is counted and pinned, so a change in the bridge or in the
/// SDK's own chart that moves a rule's input fails here first.
#[test]
fn every_corpus_chart_the_sdk_computes_reads_as_the_corpus_recorded_it() {
    let sdk = Context::builder()
        .profile("conformance-baseline")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("the conformance profile and the built-in ephemeris");
    let rules = shipped::nabhasas();
    let (mut charts, mut points, mut differ) = (0, 0, [0_usize; 5]);
    let mut refused = Vec::new();
    for (name, chart) in common::charts() {
        let document =
            match common::reading_of(&sdk, &chart, |request| request.with_rule_inputs(rules)) {
                Ok(document) => document,
                Err(error) => {
                    // The built-in ephemeris covers 1800 to 2400, and a strength
                    // reads the year before a birth: the corpus's two charts at
                    // its edges are refused by the provider, by name.
                    assert_eq!(error.status, teistro::Status::Provider, "{name}: {error}");
                    refused.push(name);
                    continue;
                }
            };
        let inputs = RuleInputs::of(&document).expect("the rules' inputs");
        let recorded = fixture(&format!("doshas/charts/{name}"));
        let recorded = &recorded["inputs"];
        for body in Body::ALL {
            let ours = inputs.chart.placement(body);
            let theirs = &recorded["bodies"][body.key()];
            let fields = [
                theirs["sign_index"].as_u64() != Some(u64::from(ours.sign as u8)),
                theirs["dignity"].as_str()
                    != Some(teistro::catalogue::Catalogued::key(ours.dignity)),
                theirs["is_retrograde"].as_bool() != Some(ours.retrograde),
                (theirs["combust"].as_str() != Some("none")) != ours.combust,
                recorded["chara_karaka_7"].get(body.key())
                    != ours
                        .karaka7
                        .map(|karaka| serde_json::to_value(Karaka(karaka)).unwrap())
                        .as_ref(),
            ];
            for (count, differs) in differ.iter_mut().zip(fields) {
                *count += usize::from(differs);
            }
        }
        // Every division a shipped rule steps into, and the special lagnas.
        let asked: Vec<Varga> = inputs.vargas.iter().map(|varga| varga.varga).collect();
        for varga in rules.iter().flat_map(Rule::vargas) {
            assert!(asked.contains(&varga), "{name}: {varga:?}");
        }
        points += inputs.points.len();
        charts += 1;
    }
    // Every difference is one chart's: c051 is cast at the Sun's entry into
    // Aries, where the recording engine's Sun stands 0.2″ short of it and the
    // built-in ephemeris's past it. So the Sun's sign and dignity differ, and
    // the Sun, 29.99999° into Pisces for the engine and the least advanced
    // graha for the SDK, reorders all seven karakas. No field differs
    // anywhere else, and no chart the shipped rules read asked for a point.
    assert_eq!((charts, points, differ), (53, 0, [1, 1, 0, 0, 7]));
    assert_eq!(
        refused,
        [
            "c047-london-1800-01-02.json",
            "c048-kathmandu-2399-12-30.json"
        ]
    );
}
