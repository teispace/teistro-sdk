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
            let asked = life().counted_from(from).reckoned(reckoning);
            let report = sdk.chart().sade_sati(&natal, &asked).unwrap().value;
            assert_eq!((report.reference.from, report.reckoning), (from, reckoning));
            assert!(
                report.sade_sati.len() >= 2,
                "two or three Sade Satis in eighty years: {report:?}"
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
/// at one instant inside a phase, the period comes back as the lifetime's
/// search found it — which is what the widening search promises.
#[test]
fn a_period_is_whole_however_the_window_is_drawn() {
    let sdk = context(None);
    let natal = natal(&sdk);
    for reckoning in Reckoning::ALL {
        let asked = life().reckoned(*reckoning);
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
