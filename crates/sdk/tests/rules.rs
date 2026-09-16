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
use teistro::catalogue::Varga;
use teistro::catalogue::{DashaSystem, Graha, Point};
use teistro::quantity::Depth;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::rules::{Body, Evaluator, House, Karaka, Readings, Rule, shipped};
use teistro::rules::{Levels, Timing};
use teistro::{
    Context, Ephemeris, RuleInputs, Timeline, UtcOffset, maraka_windows, rule_chart, rule_periods,
    rule_vargas,
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

/// How many bodies of a rule chart differ from what the corpus recorded, field
/// by field: sign, dignity, motion, combustion and seven-karaka scheme.
fn differences(chart: &teistro::RuleChart, recorded: &serde_json::Value) -> [usize; 5] {
    let mut differ = [0; 5];
    for body in Body::ALL {
        let ours = chart.placement(body);
        let theirs = &recorded["bodies"][body.key()];
        let fields = [
            theirs["sign_index"].as_u64() != Some(u64::from(ours.sign as u8)),
            theirs["dignity"].as_str() != Some(teistro::catalogue::Catalogued::key(ours.dignity)),
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
    differ
}

/// The three pairs over the corpus's 51 charts with a hora lagna, by how the
/// class was decided and what it came to: three independent pairs agree all
/// three one time in nine and differ all three two times in nine, and these
/// are 9 and 14; Saturn among the contributors takes five short lives to
/// Yogarishta.
const THREE_PAIRS: [(&str, usize); 11] = [
    ("Agreement { pairs: 2 }/Long", 4),
    ("Agreement { pairs: 2 }/Medium", 8),
    ("Agreement { pairs: 2 }/Short", 12),
    ("Agreement { pairs: 2 }/Yogarishta", 4),
    ("Agreement { pairs: 3 }/Long", 2),
    ("Agreement { pairs: 3 }/Medium", 4),
    ("Agreement { pairs: 3 }/Short", 3),
    ("LagnaPair/Long", 4),
    ("LagnaPair/Medium", 3),
    ("LagnaPair/Short", 6),
    ("SaturnAndMoon/Yogarishta", 1),
];

/// The rules' reading of every chart in the corpus, computed by the SDK from
/// the birth each records, against what the corpus recorded for the rules:
/// each body's sign, dignity, motion and combustion, and each graha's karakas.
/// What differs is counted and pinned, so a change in the bridge or in the
/// SDK's own chart that moves a rule's input fails here first.
#[test]
fn every_corpus_chart_the_sdk_computes_reads_as_the_corpus_recorded_it() {
    // BPHS ch. 39's figures on the special lagnas, which only a reading's
    // points can answer.
    const SPECIAL: [&str; 2] = [
        "BPHS_DIGNIFIED_GRAHAS_ON_THE_LAGNA_HORA_AND_GHATIKA_LAGNAS",
        "BPHS_EXALTED_ASPECTS_ON_TWO_OF_THE_BHAVA_HORA_AND_GHATIKA_LAGNAS",
    ];
    let sdk = Context::builder()
        .profile("conformance-baseline")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("the conformance profile and the built-in ephemeris");
    let rules = shipped::nabhasas();
    let (mut charts, mut points, mut differ) = (0, 0, [0_usize; 5]);
    let mut special = [0_usize; 2];
    let mut decided = std::collections::BTreeMap::new();
    let mut refused = Vec::new();
    for (name, chart) in common::charts() {
        let document =
            match common::reading_of(&sdk, &chart, |request| request.with_rule_inputs(rules)) {
                Ok(document) => document,
                Err(error) => {
                    refused.push((name, error.status));
                    continue;
                }
            };
        let inputs = RuleInputs::of(&document).expect("the rules' inputs");
        let recorded = fixture(&format!("doshas/charts/{name}"));
        let recorded = &recorded["inputs"];
        for (count, differs) in differ.iter_mut().zip(differences(&inputs.chart, recorded)) {
            *count += differs;
        }
        // Every division a shipped rule steps into.
        let asked: Vec<Varga> = inputs.vargas.iter().map(|varga| varga.varga).collect();
        for varga in rules.iter().flat_map(Rule::vargas) {
            assert!(asked.contains(&varga), "{name}: {varga:?}");
        }
        // BPHS ch. 4 vv. 2 to 5: the bhava lagna moves a sign in five ghatis
        // and the hora lagna in two and a half, both from the Sun, so the one
        // is always half as far round as the other.
        let sun = document.foundation.graha(Graha::Sun).unwrap().longitude_deg;
        let from_sun = |point: Point| {
            let at = document
                .points
                .as_ref()
                .and_then(|found| found.at(point))
                .unwrap();
            (at.longitude_deg - sun).rem_euclid(360.0)
        };
        let (bhava, hora) = (from_sun(Point::BhavaLagna), from_sun(Point::HoraLagna));
        assert!(
            ((2.0 * bhava).rem_euclid(360.0) - hora).abs() < 1e-6,
            "{name}: {bhava} {hora}"
        );
        let evaluator = inputs.evaluator(Readings::TEXTS).with_rules(rules);
        for (count, key) in special.iter_mut().zip(SPECIAL) {
            let rule = rules.iter().find(|rule| rule.key == key).unwrap();
            *count += usize::from(evaluator.evaluate(rule).present);
        }
        // BPHS ch. 43 vv. 33 to 50 on every chart with a hora lagna.
        let pairs = evaluator
            .three_pairs(teistro::rules::longevity::ThreePairsRules::VERSE)
            .expect("a hora lagna");
        *decided
            .entry(format!("{:?}/{:?}", pairs.decided, pairs.class))
            .or_insert(0) += 1;
        points += inputs.points.len();
        charts += 1;
    }
    // Every difference is one chart's: c051 is cast at the Sun's entry into
    // Aries, where the recording engine's Sun stands 0.2″ short of it and the
    // built-in ephemeris's past it. So the Sun's sign and dignity differ, and
    // the Sun, 29.99999° into Pisces for the engine and the least advanced
    // graha for the SDK, reorders all seven karakas. No field differs
    // anywhere else. Twelve points and fourteen with Gulika and Mandi, and
    // neither special-lagna figure on any chart.
    assert_eq!(
        (charts, points, differ, special),
        (51, 51 * 14, [1, 1, 0, 0, 7], [0, 0])
    );
    let decided: Vec<(&str, usize)> = decided
        .iter()
        .map(|(key, count)| (key.as_str(), *count))
        .collect();
    assert_eq!(decided, THREE_PAIRS);
    // Refused, each for what it cannot have: Tromsø's midnight sun and polar
    // night, whose days have no sunrise for a special lagna to count from; and
    // the two charts at the built-in ephemeris's edges, whose strength reads
    // the year before a birth outside 1800 to 2400.
    let refused: Vec<(&str, teistro::Status)> = refused
        .iter()
        .map(|(name, status)| (name.as_str(), *status))
        .collect();
    assert_eq!(
        refused,
        [
            ("c028-troms-1988-06-21.json", teistro::Status::OutOfRange),
            ("c029-troms-1988-12-21.json", teistro::Status::OutOfRange),
            ("c047-london-1800-01-02.json", teistro::Status::Provider),
            ("c048-kathmandu-2399-12-30.json", teistro::Status::Provider),
        ]
    );
}

/// BPHS ch. 43 on the translator's worked example: a man born on
/// 21 May 1944 at 19:01:15 Indian War Time, which was UTC+6:30, at 13° 40′ N,
/// 79° 20′ E. The SDK's chart is the translator's to a tenth of a degree. He
/// finds the lagna and eighth lords both movable, long; Saturn dual and the
/// Moon movable, short; the lagna and the hora lagna both fixed, short — so
/// short life by two pairs, 36 years. He stops there; v. 47 then lowers the
/// class, Saturn being among the contributors and neither dignified nor
/// touched by malefics alone, so the verses give Yogarishta. His rectification
/// is not checked: he rectifies with Mars and Mercury, the lords of a pair
/// that did not decide, and by degrees gone where the verse counts degrees to
/// run (crux C103).
#[test]
fn the_translator_s_worked_example_reads_as_he_reads_it() {
    use teistro::rules::LifeClass;
    use teistro::rules::longevity::{Decided, Giver, Shift, ThreePairsRules};

    let sdk = Context::builder()
        .profile("conformance-baseline")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("the conformance profile and the built-in ephemeris");
    let place = Place::new(
        Latitude::try_new(13.0 + 40.0 / 60.0).unwrap(),
        Longitude::try_new(79.0 + 20.0 / 60.0).unwrap(),
        Altitude::try_new(0.0).unwrap(),
    );
    let rules = shipped::nabhasas();
    let request = teistro::ChartRequest::at(place, UtcOffset::try_from_seconds(23_400).unwrap())
        .with_rule_inputs(rules)
        .with_points();
    let document = sdk
        .chart()
        .reading(JulianDay::<Utc>::literal(2_431_232.021_701_388_7), &request)
        .expect("a reading")
        .value;
    let inputs = RuleInputs::of(&document).expect("the rules' inputs");
    let evaluator = inputs.evaluator(Readings::TEXTS);
    let reading = evaluator
        .three_pairs(ThreePairsRules::VERSE)
        .expect("a hora lagna");
    assert_eq!(
        reading.pairs.map(|pair| pair.class),
        [LifeClass::Long, LifeClass::Short, LifeClass::Short]
    );
    assert_eq!(reading.decided, Decided::Agreement { pairs: 2 });
    assert_eq!(reading.decided_class, LifeClass::Short);
    assert_eq!(reading.shifts, [Some(Shift::Saturn), None]);
    assert_eq!(
        (reading.class, reading.years),
        (LifeClass::Yogarishta, Some(20.0))
    );

    // BPHS ch. 43 vv. 4 to 8 on the same chart: each graha's basic Pindayu
    // years are the translator's, to the precision of his longitudes against
    // the SDK's — a hundredth of a year, and a tenth for the fast Moon.
    let spans = evaluator.ayurdaya(teistro::rules::longevity::AyurdayaRules::default());
    let translator = [17.5642, 24.6247, 8.4036, 6.9968, 14.1200, 19.2327, 12.3979];
    for (given, years) in spans.pindayu.contributions.iter().zip(translator) {
        let tolerance = if matches!(given.giver, Giver::Graha(Graha::Moon)) {
            0.1
        } else {
            0.01
        };
        assert!(
            (given.basic - years).abs() < tolerance,
            "{given:?} against {years}"
        );
    }
    // His two reductions: the Sun in Venus's sign, a natural enemy's, keeps two
    // thirds, which is more than the seventh house takes; the combust Moon
    // keeps half.
    let [sun, moon, ..] = spans.pindayu.contributions;
    assert!((sun.net - 11.7095).abs() < 0.01, "{sun:?}");
    assert!((moon.net - 12.3124).abs() < 0.1, "{moon:?}");
}

/// BPHS ch. 44's maraka windows on a chart the SDK computed, over the ages its
/// own three pairs give its life: every window a sub-period of a maraka's
/// major period, inside those ages, in time order, and presented as a
/// vulnerability and never a date.
#[test]
fn a_chart_s_maraka_windows_lie_in_the_ages_its_class_of_life_runs_to() {
    use teistro::rules::longevity::{Presentation, ThreePairsRules, age_span};

    let (sdk, document) = common::reading("{}", |request| {
        request
            .with_rule_inputs(shipped::nabhasas())
            .with_points()
            .with_dashas([DashaSystem::Vimshottari])
    });
    let inputs = RuleInputs::of(&document).expect("the rules' inputs");
    let evaluator = inputs.evaluator(Readings::TEXTS);
    let class = evaluator
        .three_pairs(ThreePairsRules::VERSE)
        .expect("a hora lagna")
        .class;
    let cursor = sdk
        .chart()
        .dasha(&document, DashaSystem::Vimshottari)
        .expect("the Vimshottari");
    let birth = document.foundation.instant;
    let windows = maraka_windows(&evaluator, &cursor, birth, class).expect("the windows");
    let (from, to) = age_span(class);
    let age = |jd: f64| (jd - birth.get()) / 365.25;
    let marakas = evaluator.marakas();
    let mut fatal = 0;
    for (window, next) in windows.iter().zip(windows.iter().skip(1)) {
        assert!(window.interval.from.get() <= next.interval.from.get());
    }
    for window in &windows {
        assert!(
            age(window.interval.to.get()) > from
                && to.is_none_or(|to| age(window.interval.from.get()) < to)
        );
        assert!(!marakas.of(window.major).is_empty());
        assert_eq!(
            window.vulnerability.presentation,
            Presentation::Vulnerability
        );
        fatal += usize::from(window.vulnerability.malefic_in_malefic);
    }
    // Medium life, 32 to 64 years: of the sub-periods those years hold, 18 fall
    // in a maraka's major period and 7 are a malefic major period in a
    // malefic sub-period, which v. 8 makes the fatal kind.
    assert_eq!(
        (class, windows.len(), fatal),
        (teistro::rules::LifeClass::Medium, 18, 7)
    );
}
