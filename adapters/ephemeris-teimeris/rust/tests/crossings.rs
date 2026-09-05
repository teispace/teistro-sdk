//! The SDK's crossings and stations kernel over Teimeris's positions against
//! the engine's own searches, and against the baseline's panchanga
//! transitions (`fixtures/baseline/charts/*.json`, `panchanga_day`): the Sun's
//! sign ingresses over a year, Mercury's with its retrograde re-entries, the
//! tithi lattice over a lunation, Mercury's and Mars's stations over two
//! years, and the tithi, nakshatra, yoga and karana ends of the fixtures'
//! days, which the baseline reckoned geocentrically. Run by hand with the engine
//! present; the measured agreement is published in
//! `docs/03-design/astro-events-and-crossings.md`.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    clippy::cast_precision_loss,
    reason = "a by-hand measurement prints what it finds and fails by panicking"
)]

use std::path::Path;

use serde_json::Value;
use teimeris::{Body as EngineBody, CrossingOptions, CrossingQuantity};
use teistro_astro::events::{Direction, Lattice, Quantity, Search, stations};
use teistro_astro::{Completion, DeltaTModel};
use teistro_core::angle::difference_deg;
use teistro_core::catalogue::Ayanamsha;
use teistro_core::quantity::{JulianDay, Ut1};
use teistro_core::settings::OverridePolicy;
use teistro_ephemeris_teimeris::{TeimerisProvider, data_dir_from_env};
use teistro_port_ephemeris::{Body, Frame, Zodiac};

const J2000: f64 = 2_451_545.0;
const SECOND: f64 = 1.0 / 86_400.0;

fn provider() -> TeimerisProvider {
    TeimerisProvider::open(&data_dir_from_env()).unwrap_or_else(|e| panic!("{e}"))
}

fn engine_body(body: Body) -> EngineBody {
    match body {
        Body::Sun => EngineBody::SUN,
        Body::Moon => EngineBody::MOON,
        Body::Mercury => EngineBody::MERCURY,
        Body::Mars => EngineBody::MARS,
        other => panic!("no engine body for {other:?}"),
    }
}

/// The engine's crossings of a lattice for one body, sorted.
fn engine_crossings(
    provider: &TeimerisProvider,
    body: Body,
    lattice: Lattice,
    from: f64,
    to: f64,
) -> Vec<(f64, f64)> {
    provider.with_context(|ctx| {
        let options = CrossingOptions {
            quantity: CrossingQuantity::LONGITUDE,
            target: lattice.origin_deg,
            step: lattice.step_deg,
            jd_end: to,
            ..CrossingOptions::default()
        };
        let mut found: Vec<(f64, f64)> = ctx
            .crossings(from, engine_body(body), &options)
            .map(|c| c.map(|c| (c.jd, c.longitude)))
            .collect::<Result<_, _>>()
            .unwrap();
        found.sort_by(|a, b| a.0.total_cmp(&b.0));
        found
    })
}

#[test]
fn the_kernel_reproduces_the_engines_ingresses_tithis_and_stations() {
    let provider = provider();
    let completion = Completion::new(
        &provider,
        OverridePolicy::PreferNative,
        DeltaTModel::TableThenModel,
    );
    let longitudes = completion.longitudes(Frame::CANONICAL);
    let from = JulianDay::<Ut1>::literal(J2000);
    let year = JulianDay::<Ut1>::literal(J2000 + 365.25);

    // The Sun's twelve ingresses and Mercury's, with the retrograde re-entries.
    for (body, bound_seconds) in [(Body::Sun, 1.0), (Body::Mercury, 1.0)] {
        let ours = Search::new(&longitudes, Quantity::Longitude(body), Lattice::SIGNS)
            .between(from, year)
            .unwrap();
        let theirs = engine_crossings(&provider, body, Lattice::SIGNS, J2000, J2000 + 365.25);
        println!(
            "{body:?}: {} SDK crossings, {} engine crossings",
            ours.len(),
            theirs.len()
        );
        assert_eq!(ours.len(), theirs.len(), "{body:?}");
        let mut worst = 0.0f64;
        for (mine, (jd, lon)) in ours.iter().zip(&theirs) {
            let apart = (mine.instant.get() - jd).abs() / SECOND;
            worst = worst.max(apart);
            assert!(
                difference_deg(mine.boundary_deg, *lon).abs() < 1e-6,
                "{body:?}: {} against {lon}",
                mine.boundary_deg
            );
        }
        println!("  worst {worst:.3} s");
        assert!(worst < bound_seconds, "{body:?}: {worst} s");
        if body == Body::Mercury {
            assert!(
                ours.iter().any(|e| e.direction == Direction::Falling),
                "a retrograde re-entry"
            );
        }
    }

    // The tithi lattice over a lunation.
    let lunation = JulianDay::<Ut1>::literal(J2000 + 29.53);
    let ours = Search::new(&longitudes, Quantity::ELONGATION, Lattice::TITHIS)
        .between(from, lunation)
        .unwrap();
    let theirs: Vec<(f64, f64)> = provider.with_context(|ctx| {
        let options = CrossingOptions {
            quantity: CrossingQuantity::RELATIVE_ANGLE,
            target: 0.0,
            step: 12.0,
            body_b: EngineBody::SUN,
            coeff_a: 1.0,
            coeff_b: -1.0,
            jd_end: J2000 + 29.53,
            ..CrossingOptions::default()
        };
        let mut found: Vec<(f64, f64)> = ctx
            .crossings(J2000, EngineBody::MOON, &options)
            .map(|c| c.map(|c| (c.jd, c.longitude)))
            .collect::<Result<_, _>>()
            .unwrap();
        found.sort_by(|a, b| a.0.total_cmp(&b.0));
        found
    });
    println!("tithis: {} SDK, {} engine", ours.len(), theirs.len());
    assert_eq!(ours.len(), theirs.len());
    let mut worst = 0.0f64;
    for (mine, (jd, _)) in ours.iter().zip(&theirs) {
        worst = worst.max((mine.instant.get() - jd).abs() / SECOND);
    }
    println!("  worst {worst:.3} s");
    assert!(worst < 1.0, "{worst} s");

    // Stations of Mercury and Mars over two years against the engine's
    // speed-zero search.
    let two_years = JulianDay::<Ut1>::literal(J2000 + 730.5);
    for body in [Body::Mercury, Body::Mars] {
        let ours = stations(&longitudes, body, from, two_years, 1e-7).unwrap();
        let theirs: Vec<(f64, f64)> = provider.with_context(|ctx| {
            let options = CrossingOptions {
                quantity: CrossingQuantity::LONGITUDE_SPEED,
                target: 0.0,
                jd_end: J2000 + 730.5,
                ..CrossingOptions::default()
            };
            let mut found: Vec<(f64, f64)> = ctx
                .crossings(J2000, engine_body(body), &options)
                .map(|c| c.map(|c| (c.jd, c.longitude)))
                .collect::<Result<_, _>>()
                .unwrap();
            found.sort_by(|a, b| a.0.total_cmp(&b.0));
            found
        });
        println!(
            "{body:?} stations: {} SDK, {} engine",
            ours.len(),
            theirs.len()
        );
        assert_eq!(ours.len(), theirs.len(), "{body:?}");
        let mut worst = 0.0f64;
        for (mine, (jd, lon)) in ours.iter().zip(&theirs) {
            worst = worst.max((mine.instant.get() - jd).abs() / SECOND);
            assert!(
                difference_deg(mine.longitude_deg, *lon).abs() < 1e-4,
                "{body:?}: {} against {lon}",
                mine.longitude_deg
            );
        }
        println!("  worst {worst:.1} s");
        // A station is where a speed of arcseconds a day crosses zero: the
        // instant is soft by construction, and the engines differ among
        // themselves by minutes here.
        assert!(worst < 600.0, "{body:?}: {worst} s");
    }
}

/// The baseline's panchanga day: its transitions are geocentric although
/// its chart positions are topocentric (`fixtures/README.md`, convention
/// one), and its zodiac is Lahiri's.
#[test]
fn the_kernel_reproduces_the_baselines_panchanga_transitions() {
    let provider = provider();
    let completion = Completion::new(
        &provider,
        OverridePolicy::PreferNative,
        DeltaTModel::TableThenModel,
    );
    let frame = Frame::CANONICAL.with_zodiac(Zodiac::sidereal(Ayanamsha::Lahiri));
    let longitudes = completion.longitudes(frame);
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../fixtures/baseline/charts");
    let mut charts: Vec<Value> = std::fs::read_dir(dir)
        .expect("the fixtures directory")
        .map(|entry| entry.expect("an entry").path())
        .filter(|path| path.extension().is_some_and(|e| e == "json"))
        .map(|path| {
            serde_json::from_str(&std::fs::read_to_string(path).expect("a fixture"))
                .expect("valid JSON")
        })
        .collect();
    charts.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    let searches: [(&str, Quantity, Lattice); 4] = [
        ("tithi", Quantity::ELONGATION, Lattice::TITHIS),
        (
            "nakshatra",
            Quantity::Longitude(Body::Moon),
            Lattice::NAKSHATRAS,
        ),
        ("yoga", Quantity::MOON_PLUS_SUN, Lattice::YOGAS),
        ("karana", Quantity::ELONGATION, Lattice::KARANAS),
    ];
    let mut worst: [(f64, String); 4] = Default::default();
    let mut compared = 0;
    let mut all: Vec<f64> = Vec::new();
    for chart in &charts {
        let id = chart["id"].as_str().unwrap();
        let day = &chart["panchanga_day"];
        let sunrise = day["sunrise_jd"].as_f64().unwrap();
        let from = JulianDay::<Ut1>::literal(sunrise - 0.05);
        let to = JulianDay::<Ut1>::literal(sunrise + 1.1);
        for (index, (name, quantity, lattice)) in searches.iter().enumerate() {
            let ours = Search::new(&longitudes, *quantity, *lattice)
                .between(from, to)
                .unwrap();
            let entries = day[*name].as_array().unwrap();
            // The last entry ends at the next sunrise, not at a transition.
            for entry in &entries[..entries.len() - 1] {
                let end = entry["end_jd"].as_f64().unwrap();
                let nearest = ours
                    .iter()
                    .map(|e| (e.instant.get() - end).abs())
                    .fold(f64::INFINITY, f64::min);
                let seconds = nearest / SECOND;
                compared += 1;
                all.push(seconds);
                if seconds > worst[index].0 {
                    worst[index] = (seconds, format!("{id} {name} ending {end}"));
                }
            }
        }
    }
    all.sort_by(f64::total_cmp);
    for ((name, _, _), (seconds, where_)) in searches.iter().zip(&worst) {
        println!("{name:<10} worst {seconds:.2} s at {where_}");
    }
    println!(
        "{compared} transitions compared; median {:.2} s",
        all[all.len() / 2]
    );
    assert!(compared > 100);
    for (seconds, where_) in &worst {
        assert!(*seconds < 10.0, "{seconds} s at {where_}");
    }
}
