//! Sade Sati through the façade: every bound read back through a chart
//! founded either side of it, the same period whole however the window is
//! drawn around it, and a batch answering as each chart alone
//! (`docs/03-design/sade-sati.md`).

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "tests fail by panicking and index what they found"
)]

use teistro::ChartFoundation;
use teistro::catalogue::{ChartKind, Graha};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::sade_sati::{Phase, Reckoning, Report, SadeSati, Spell};
use teistro::{ChartRequest, Context, Document, Ephemeris, GocharFrom, SadeSatiRequest, UtcOffset};

fn context(profile: Option<&str>) -> Context {
    let mut builder = Context::builder().ephemeris([Ephemeris::Builtin]);
    if let Some(profile) = profile {
        builder = builder.profile(profile);
    }
    builder.build().unwrap()
}

fn place() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    )
}

fn natal_at(sdk: &Context, jd: f64) -> Document {
    sdk.chart()
        .reading(
            JulianDay::<Utc>::literal(jd),
            &ChartRequest::at(place(), UtcOffset::literal(5, 45, 0)),
        )
        .unwrap()
        .value
}

fn natal(sdk: &Context) -> Document {
    natal_at(sdk, 2_447_995.489_583_333_5)
}

/// Eighty years from the birth in 1990.
fn life() -> SadeSatiRequest {
    SadeSatiRequest::between(
        JulianDay::<Utc>::literal(2_447_995.5),
        JulianDay::<Utc>::literal(2_477_215.5),
    )
}

/// Thirty years from 2000, a circuit of Saturn's: every reckoning and
/// reference finds one Sade Sati in it and each smaller spell, at a
/// third of a lifetime's price.
fn circuit() -> SadeSatiRequest {
    SadeSatiRequest::between(
        JulianDay::<Utc>::literal(2_451_545.0),
        JulianDay::<Utc>::literal(2_462_502.5),
    )
}

fn spells_of(report: &Report) -> Vec<&Spell> {
    report
        .sade_sati
        .iter()
        .flat_map(|one| one.phases.iter())
        .chain(&report.spells)
        .collect()
}

/// The house Saturn stands in from the reference in a founded chart, as the
/// reckoning counts it.
fn house_in(chart: &ChartFoundation, reference_deg: f64, reckoning: Reckoning) -> u8 {
    let saturn = chart.graha(Graha::Saturn).unwrap().longitude_deg;
    let origin = reckoning.origin_deg(reference_deg);
    ((saturn - origin).rem_euclid(360.0) / 30.0) as u8 % 12 + 1
}

/// The acceptance is the consumer: a chart founded a second inside each
/// visit's bounds has Saturn in the visit's house, and one a second
/// outside has it in a neighbour — for each reckoning and reference, under
/// the default profile and the topocentric conformance one.
#[test]
fn every_bound_is_where_a_founded_chart_moves_saturn() {
    for profile in [None, Some("conformance-baseline")] {
        let sdk = context(profile);
        let natal = natal(&sdk);
        for (from, reckoning) in [
            (GocharFrom::Moon, Reckoning::Sign),
            (GocharFrom::Moon, Reckoning::Degree),
            (GocharFrom::Lagna, Reckoning::Sign),
        ] {
            let asked = circuit().counted_from(from).reckoned(reckoning);
            let report = sdk.chart().sade_sati(&natal, &asked).unwrap().value;
            assert_eq!((report.reference.from, report.reckoning), (from, reckoning));
            assert!(
                !report.sade_sati.is_empty() && report.spells.len() >= 2,
                "a Sade Sati and the 4th and 8th in a circuit: {report:?}"
            );
            let reference_deg = match from {
                GocharFrom::Lagna => natal.foundation.lagna_deg,
                _ => natal.foundation.graha(Graha::Moon).unwrap().longitude_deg,
            };
            let second = 1.0 / 86_400.0;
            let mut probes = Vec::new();
            for spell in spells_of(&report) {
                for visit in &spell.visits {
                    for (bound, inside) in [(visit.from, second), (visit.to, -second)] {
                        let Some(bound) = bound else {
                            panic!("the builtin ephemeris covers the century");
                        };
                        probes.push((spell.house, bound.get() + inside, true));
                        probes.push((spell.house, bound.get() - inside, false));
                    }
                }
            }
            let instants: Vec<_> = probes
                .iter()
                .map(|(_, jd, _)| JulianDay::<Utc>::literal(*jd))
                .collect();
            let founded = sdk
                .chart()
                .found_many(&instants, &place(), UtcOffset::UTC, ChartKind::Natal)
                .unwrap()
                .value;
            for ((house, jd, inside), chart) in probes.iter().zip(&founded) {
                let stands = house_in(chart, reference_deg, reckoning);
                assert_eq!(
                    stands == *house,
                    *inside,
                    "{profile:?} {from:?} {reckoning:?}: house {house} at {jd}, Saturn in {stands}"
                );
            }
        }
    }
}

/// Each Sade Sati is its three phases in order, its visits in time order
/// and apart, and the next one a circuit of Saturn's later.
#[test]
fn a_sade_sati_is_three_phases_and_the_next_is_a_circuit_away() {
    let sdk = context(None);
    let report = sdk.chart().sade_sati(&natal(&sdk), &life()).unwrap().value;
    for one in &report.sade_sati {
        let houses: Vec<u8> = one.phases.iter().map(|spell| spell.house).collect();
        assert_eq!(houses, [12, 1, 2]);
        // In time order the visits abut or leave a gap, never overlap: a
        // retrograde loop across the 1st and 2nd interleaves the phases.
        let mut visits: Vec<_> = one.phases.iter().flat_map(|spell| &spell.visits).collect();
        visits.sort_by(|a, b| a.from.unwrap().get().total_cmp(&b.from.unwrap().get()));
        for pair in visits.windows(2) {
            assert!(pair[0].to.unwrap().get() <= pair[1].from.unwrap().get());
        }
        let years = (one.ends().unwrap().get() - one.begins().unwrap().get()) / 365.25;
        assert!((6.5..9.5).contains(&years), "{years} years");
    }
    for pair in report.sade_sati.windows(2) {
        let apart = (pair[1].begins().unwrap().get() - pair[0].begins().unwrap().get()) / 365.25;
        assert!((27.0..32.0).contains(&apart), "{apart} years apart");
    }
    // The 4th and the 8th, alternating, each within its circuit.
    let houses: Vec<u8> = report.spells.iter().map(|spell| spell.house).collect();
    assert!(
        houses.windows(2).all(|pair| pair[0] != pair[1]),
        "{houses:?}"
    );
}

/// The window chooses which periods are reported, never their bounds: asked
/// at one instant inside a phase, the period comes back as the circuit's
/// search found it — which is what the widening search promises.
#[test]
fn a_period_is_whole_however_the_window_is_drawn() {
    let sdk = context(None);
    let natal = natal(&sdk);
    for reckoning in Reckoning::ALL {
        let asked = circuit().reckoned(*reckoning);
        let whole = sdk.chart().sade_sati(&natal, &asked).unwrap().value;
        for one in &whole.sade_sati {
            for phase in Phase::ALL {
                let spell = one.phase(phase).unwrap();
                let (from, to) = (spell.begins().unwrap(), spell.ends().unwrap());
                let mid = JulianDay::<Utc>::literal(f64::midpoint(from.get(), to.get()));
                let now = SadeSatiRequest::at(mid).reckoned(*reckoning);
                let found = sdk.chart().sade_sati(&natal, &now).unwrap().value;
                let at_now: Vec<&SadeSati> = found.sade_sati.iter().collect();
                assert_eq!(at_now, [one], "{reckoning:?} {phase:?}");
                assert_eq!(
                    found.phase_at(mid),
                    one.phase_at(mid),
                    "{reckoning:?} {phase:?}"
                );
            }
        }
    }
}

#[test]
fn a_batch_answers_as_each_chart_alone() {
    let sdk = context(None);
    let charts = [
        natal(&sdk),
        natal_at(&sdk, 2_451_545.2),
        natal_at(&sdk, 2_444_000.7),
    ];
    for reckoning in Reckoning::ALL {
        let asked = SadeSatiRequest::between(
            JulianDay::<Utc>::literal(2_460_676.5),
            JulianDay::<Utc>::literal(2_464_329.0),
        )
        .reckoned(*reckoning)
        .with_spells([4, 7, 8]);
        let batch = sdk.chart().sade_sati_many(&charts, &asked).unwrap().value;
        assert_eq!(batch.len(), charts.len());
        for (chart, together) in charts.iter().zip(&batch) {
            let alone = sdk.chart().sade_sati(chart, &asked).unwrap().value;
            assert_eq!(&alone, together, "{reckoning:?}");
        }
    }
}

#[test]
fn a_bad_request_is_refused_by_the_field_it_names() {
    let sdk = context(None);
    let natal = natal(&sdk);
    let backwards = SadeSatiRequest::between(life().end(), life().start());
    assert_eq!(
        sdk.chart()
            .sade_sati(&natal, &backwards)
            .unwrap_err()
            .field(),
        Some("to")
    );
    for spells in [vec![1], vec![12], vec![4, 4]] {
        let refused = life().with_spells(spells.clone());
        assert_eq!(
            sdk.chart().sade_sati(&natal, &refused).unwrap_err().field(),
            Some("spells"),
            "{spells:?}"
        );
    }
    let none: [&Document; 0] = [];
    assert_eq!(
        sdk.chart()
            .sade_sati_many(none, &life())
            .unwrap_err()
            .field(),
        Some("natals")
    );
}

/// What a loaded corpus says of the periods, through the façade: nothing
/// before the states pack is loaded, then each house the report holds
/// once, rendering the pack's own words with the house beside them; and
/// nothing for a report counted from the lagna, whose houses are not the
/// corpus's subject.
#[test]
fn a_report_is_said_in_the_corpus_words_once_a_pack_is_loaded() {
    let sdk = context(None);
    let natal = natal(&sdk);
    let report = sdk.chart().sade_sati(&natal, &life()).unwrap().value;
    assert!(
        sdk.interpret().sade_sati(&report).is_empty(),
        "{:?}",
        sdk.interpret().sade_sati(&report)
    );

    let states = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packs/states");
    let tree = teistro::Tree::load(&states).unwrap();
    let bytes = teistro::pack::build(&tree.locales["en-Latn"], "sdk.entity").unwrap();
    sdk.intl().load_pack(&bytes).unwrap();

    let plan = sdk.interpret().sade_sati(&report);
    let mut houses: Vec<u8> = Vec::new();
    for spell in spells_of(&report) {
        if !houses.contains(&spell.house) {
            houses.push(spell.house);
        }
    }
    assert_eq!(plan.len(), houses.len(), "each house once: {houses:?}");
    for item in &plan.items {
        let rendered = sdk.intl().render(&item.key, &item.params);
        assert!(
            !rendered.is_fallback && rendered.warnings.is_empty() && !rendered.text.is_empty(),
            "{item:?} rendered {rendered:?}"
        );
    }
    let first: Vec<i64> = plan
        .items
        .iter()
        .map(|item| match item.params.get("house") {
            Some(teistro::Value::Int(house)) => *house,
            other => panic!("a house slot, not {other:?}"),
        })
        .collect();
    // The houses in the order Saturn first reached them over the life.
    let mut by_entry: Vec<&Spell> = spells_of(&report);
    by_entry.sort_by(|a, b| {
        let at = |spell: &Spell| spell.begins().map_or(f64::NEG_INFINITY, JulianDay::get);
        at(a).total_cmp(&at(b))
    });
    let mut expected: Vec<i64> = Vec::new();
    for spell in by_entry {
        if !expected.contains(&i64::from(spell.house)) {
            expected.push(i64::from(spell.house));
        }
    }
    assert_eq!(first, expected);
    assert!(
        [12, 1, 2, 4, 8].iter().all(|house| first.contains(house)),
        "eighty years reach every phase and both smaller spells: {first:?}"
    );

    let from_lagna = sdk
        .chart()
        .sade_sati(&natal, &circuit().counted_from(GocharFrom::Lagna))
        .unwrap()
        .value;
    assert!(
        sdk.interpret().sade_sati(&from_lagna).is_empty(),
        "{:?}",
        sdk.interpret().sade_sati(&from_lagna)
    );
}

/// The plan asked for beside the charts: `interpreted` searches once for
/// the batch, gives each chart the report the search alone gives, and says
/// it as the composer does — the plans are what `InterpretArea::plans`
/// composes from the same answers, rules and all; it is refused without a
/// window, and a batch of none is an empty answer rather than the
/// search's refusal of no charts.
#[test]
fn a_reading_asked_to_say_its_periods_searches_once_and_says_them() {
    use teistro::{Answers, PlanInputs, PlanRequest, RuleRequest};

    let sdk = context(None);
    let states = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packs/states");
    let tree = teistro::Tree::load(&states).unwrap();
    let bytes = teistro::pack::build(&tree.locales["en-Latn"], "sdk.entity").unwrap();
    sdk.intl().load_pack(&bytes).unwrap();

    let window = circuit();
    let births = [2_447_995.489_583_333_5, 2_451_545.0].map(JulianDay::<Utc>::literal);
    let request = ChartRequest::at(place(), UtcOffset::literal(5, 45, 0));
    let rules = RuleRequest::shipped([]).rule_set().unwrap();
    let asked = PlanRequest::default().with_sade_sati().with_readings();
    let read = sdk
        .chart()
        .interpreted(
            &births,
            &request,
            PlanInputs::none()
                .with_rules(&rules)
                .with_sade_sati(&window),
            asked,
        )
        .unwrap()
        .value;
    assert_eq!(read.len(), births.len());
    for chart in &read {
        let (Some(report), Some(plan), Some(reading)) = (
            chart.sade_sati.as_ref(),
            chart.plans.sade_sati.as_ref(),
            chart.reading.as_ref(),
        ) else {
            panic!("a window, a plan and rules were asked for");
        };
        let alone = sdk
            .chart()
            .sade_sati(&chart.document, &window)
            .unwrap()
            .value;
        assert_eq!(report, &alone, "the batch's report is the chart's own");
        assert_eq!(plan, &sdk.interpret().sade_sati(report));
        assert!(!plan.is_empty(), "thirty years hold a period");
        let answers = Answers::none().with_reading(reading).with_sade_sati(report);
        assert_eq!(
            sdk.interpret()
                .plans(&chart.document, answers, asked)
                .unwrap(),
            chart.plans
        );
    }

    let refused = sdk
        .chart()
        .interpreted(&births, &request, &rules, asked)
        .unwrap_err();
    assert_eq!(refused.field(), Some("interpret.sadeSati"));

    let none = sdk
        .chart()
        .interpreted(
            &[],
            &request,
            PlanInputs::from(&rules).with_sade_sati(&window),
            asked,
        )
        .unwrap();
    assert!(none.value.is_empty(), "{:?}", none.value);
}
