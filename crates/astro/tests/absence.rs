//! The absence fast path answers what the walk answered.
//!
//! `Solver::event` tries to prove a rise or a set **absent** from two
//! readings before looking for it
//! (`03-design/horizon-absence-measured.md`). The measured page holds the
//! *rule* — that the bound never claims an absence where an event exists,
//! over a sweep run with the fast path off. This holds the *code*: that
//! turning the fast path on changes nothing but the cost.
//!
//! Two different things, and both are needed. The page could be right
//! about the mathematics while the solver applied it to the wrong window,
//! at the wrong latitude, or for a transit — and the page would not
//! notice, because it computes the bound itself rather than asking the
//! solver for it.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::print_stdout,
    reason = "a test fails by panicking and prints what it measured"
)]

use teistro_astro::rise_set::{Solver, declination_chord_margin};
use teistro_astro::{Completion, DeltaTModel};
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Ut1};
use teistro_core::settings::OverridePolicy;
use teistro_port_ephemeris::{Body, CountingProvider, Horizon, HorizonEventKind, TestProvider};

/// The same sweep the measured page walks: the tropics, the temperate
/// belt and both polar circles, where an absence stops being rare.
const LATITUDES: [f64; 13] = [
    -78.0, -69.0, -60.0, -45.0, -27.7, -10.0, 0.0, 10.0, 27.7, 45.0, 60.0, 69.0, 78.0,
];
const DAYS: u32 = 34;
const DAY_STEP: f64 = 11.0;

#[test]
fn the_fast_path_answers_what_the_walk_answered_and_costs_less() {
    let provider = CountingProvider::new(TestProvider::new());
    let completion = Completion::new(
        &provider,
        OverridePolicy::SdkOnly,
        DeltaTModel::TableThenModel,
    );
    let horizon = Horizon::CENTRE_NO_REFRACTION;

    let (mut compared, mut absences, mut proved) = (0usize, 0usize, 0usize);
    let (mut cells_walked, mut cells_fast) = (0u64, 0u64);

    for body in [Body::Sun, Body::Moon, Body::Mars] {
        for latitude in LATITUDES {
            let place = Place::new(
                Latitude::literal(latitude),
                Longitude::literal(0.0),
                Altitude::literal(0.0),
            );
            let walking = Solver::new(
                &completion,
                body,
                place,
                horizon,
                DeltaTModel::TableThenModel,
            )
            .with_absence_check(false);
            let fast = Solver::new(
                &completion,
                body,
                place,
                horizon,
                DeltaTModel::TableThenModel,
            );
            for day in 0..DAYS {
                let from = JulianDay::<Ut1>::literal(2_451_545.0 + f64::from(day) * DAY_STEP);
                for kind in [
                    HorizonEventKind::Rise,
                    HorizonEventKind::Set,
                    // A transit is a crossing of the hour angle and
                    // happens once a rotation whatever the altitude does,
                    // so the fast path must never touch it.
                    HorizonEventKind::Transit,
                    HorizonEventKind::Antitransit,
                ] {
                    provider.reset();
                    let Ok(walked) = walking.event(kind, from, 1.0) else {
                        continue;
                    };
                    cells_walked += provider.calls().cells;

                    provider.reset();
                    let quick = fast.event(kind, from, 1.0).expect("the same search");
                    cells_fast += provider.calls().cells;

                    assert_eq!(
                        walked.is_some(),
                        quick.is_some(),
                        "{body:?} at {latitude}° on day {day}, {kind}: one found an \
                         event and the other did not"
                    );
                    if let (Some(walked), Some(quick)) = (walked, quick) {
                        assert_eq!(
                            walked.instant.get().to_bits(),
                            quick.instant.get().to_bits(),
                            "{body:?} at {latitude}° on day {day}, {kind}: {} against {}",
                            walked.instant,
                            quick.instant
                        );
                        assert_eq!(walked.method, quick.method);
                    } else {
                        absences += 1;
                        if quick.is_none() && provider.calls().cells < 10 {
                            proved += 1;
                        }
                    }
                    compared += 1;
                }
            }
        }
    }

    println!("searches compared: {compared}");
    println!("absences:          {absences}, {proved} proved without walking");
    println!("cells walking:     {cells_walked}");
    println!("cells fast:        {cells_fast}");
    #[allow(clippy::cast_precision_loss, reason = "a ratio of counts")]
    let saved = 100.0 * (cells_walked - cells_fast) as f64 / cells_walked as f64;
    println!("saved:             {saved:.1}%");

    assert!(compared > 1_500, "the whole sweep ran: {compared}");
    assert!(absences > 0, "the sweep reaches absences at all");
    assert!(
        cells_fast < cells_walked,
        "the fast path costs less: {cells_fast} against {cells_walked}"
    );
}

/// A body the sweep has not measured gets no fast path, and the same
/// answers.
#[test]
fn an_unmeasured_body_walks_as_it_always_did() {
    assert!(declination_chord_margin(Body::Sun).is_some());
    assert!(declination_chord_margin(Body::Moon).is_some());
    for body in [Body::Mars, Body::Jupiter, Body::MeanNode] {
        assert!(
            declination_chord_margin(body).is_none(),
            "{body:?}: a bound that might be too small is worse than none"
        );
    }

    // And with no rate, the check cannot fire: the two solvers cost the
    // same, cell for cell.
    let provider = CountingProvider::new(TestProvider::new());
    let completion = Completion::new(
        &provider,
        OverridePolicy::SdkOnly,
        DeltaTModel::TableThenModel,
    );
    let place = Place::new(
        Latitude::literal(69.0),
        Longitude::literal(0.0),
        Altitude::literal(0.0),
    );
    let from = JulianDay::<Ut1>::literal(2_451_545.0);
    let make = |on: bool| {
        Solver::new(
            &completion,
            Body::Mars,
            place,
            Horizon::CENTRE_NO_REFRACTION,
            DeltaTModel::TableThenModel,
        )
        .with_absence_check(on)
    };
    provider.reset();
    let walked = make(false)
        .event(HorizonEventKind::Rise, from, 1.0)
        .unwrap();
    let walking_cells = provider.calls().cells;
    provider.reset();
    let quick = make(true).event(HorizonEventKind::Rise, from, 1.0).unwrap();
    assert_eq!(walked.is_some(), quick.is_some());
    assert_eq!(
        provider.calls().cells,
        walking_cells,
        "no reading saved, none spent"
    );
}
