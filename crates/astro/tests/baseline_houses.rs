//! The house cusps of every system against the baseline engine's fixtures:
//! the 55 charts carry all twenty-two systems' sidereal cusps (Lahiri) with
//! the ayanamsha the baseline applied, so the tropical cusps are recovered
//! and compared with the SDK's from the chart's instant and place. Rank 2
//! (the baseline over the Swiss Ephemeris) with a tolerance.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    reason = "tests fail by panicking, read fixtures and print the measurement under --nocapture"
)]

mod common;

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::Value;
use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::houses::{ChartFrame, Outcome, houses_at};
use teistro_astro::scale::tt_of;
use teistro_core::angle::difference_deg;
use teistro_core::catalogue::HouseSystem;
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Ut1};
use teistro_core::settings::PolarPolicy;

/// The baseline's cusps against the SDK's, degrees, for charts between 1800
/// and 2200: measured 0.00021° at worst (three quarters of an arcsecond,
/// the two engines' sidereal times), bounded at an arcsecond.
const BOUND_DEG: f64 = 1.0 / 3600.0;

/// Beyond 2200 the engine behind the baseline switches to a long-term
/// sidereal-time model and the meridian differs from the SDK's IAU 2000
/// formula by up to 0.8 s of time (measured 0.0033° on the ascendant of a
/// chart in 2399); the bound there.
const FAR_BOUND_DEG: f64 = 5e-3;

/// The Julian days of 1800-01-01 and 2200-01-01.
const NEAR_SPAN: (f64, f64) = (2_378_497.0, 2_524_593.0);

fn charts() -> Vec<Value> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/baseline/charts");
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
    charts
}

/// The baseline's system names to the catalogue.
fn system_of(name: &str) -> Option<HouseSystem> {
    let key = name.to_uppercase().replace('-', "_");
    HouseSystem::from_key(&key)
}

#[test]
fn every_system_reproduces_the_baselines_cusps_on_all_charts() {
    let charts = charts();
    assert_eq!(charts.len(), 55);
    let mut worst: BTreeMap<String, (f64, String)> = BTreeMap::new();
    let mut worst_far: BTreeMap<String, (f64, String)> = BTreeMap::new();
    let (mut near_compared, mut far_compared) = (0usize, 0usize);
    let mut compared = 0;
    let mut substituted = Vec::new();
    for chart in &charts {
        let id = chart["id"].as_str().unwrap();
        let place = Place::new(
            Latitude::literal(chart["input"]["place"]["latitude"].as_f64().unwrap()),
            Longitude::literal(chart["input"]["place"]["longitude"].as_f64().unwrap()),
            Altitude::literal(
                chart["input"]["place"]["altitude_m"]
                    .as_f64()
                    .unwrap_or(0.0),
            ),
        );
        let ut1 = JulianDay::<Ut1>::literal(chart["input"]["resolved"]["jd_ut"].as_f64().unwrap());
        let (tt, _) = tt_of(ut1, DeltaTModel::TableThenModel).unwrap();
        let ayanamsha = chart["foundation"]["ayanamsha"]["value_deg"]
            .as_f64()
            .unwrap();
        // The baseline's Sun for the Sunshine system: its tropical longitude
        // and latitude give the declination under the SDK's obliquity.
        let sun = &chart["positions"]["bodies"]["SUN"];
        let obliquity = teistro_astro::sky::obliquity(tt).true_deg.to_radians();
        let sun_declination = sun["tropical_longitude_deg"].as_f64().map(|lon| {
            let lat = sun["latitude_deg"].as_f64().unwrap_or(0.0).to_radians();
            let lon = lon.to_radians();
            (lat.sin() * obliquity.cos() + lat.cos() * obliquity.sin() * lon.sin())
                .asin()
                .to_degrees()
        });
        for (name, section) in chart["houses"]["all_systems"].as_object().unwrap() {
            let Some(system) = system_of(name) else {
                panic!("{id}: unknown system {name}");
            };
            let expected: Vec<f64> = section["cusps"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| c.as_f64().unwrap())
                .collect();
            let chart_frame = ChartFrame {
                sidereal_offset_deg: ayanamsha,
                sun_declination_deg: sun_declination,
            };
            let ours = houses_at(
                system,
                ut1,
                tt,
                &place,
                &chart_frame,
                PolarPolicy::FallbackPorphyry,
            )
            .unwrap_or_else(|e| panic!("{id} {name}: {e}"));
            if ours.outcome != Outcome::Defined {
                substituted.push(format!("{id} {name}"));
            }
            let far = !(NEAR_SPAN.0..NEAR_SPAN.1).contains(&ut1.get());
            let table = if far { &mut worst_far } else { &mut worst };
            let counted = if far {
                &mut far_compared
            } else {
                &mut near_compared
            };
            let entry = table.entry(name.clone()).or_insert((0.0, String::new()));
            for (house, (theirs, mine)) in expected.iter().zip(ours.cusps.iter()).enumerate() {
                let tropical_theirs = theirs + ayanamsha;
                let apart = difference_deg(*mine, tropical_theirs).abs();
                if apart > entry.0 {
                    *entry = (apart, format!("{id} house {}", house + 1));
                }
                compared += 1;
                *counted += 1;
            }
        }
    }
    for (name, (apart, where_)) in &worst {
        let far = worst_far.get(name).map_or(0.0, |w| w.0);
        println!("{name:<14} worst {apart:.7}° at {where_}; beyond 2200: {far:.7}°");
    }
    println!("{compared} cusps compared; substituted: {substituted:?}");
    assert!(compared > 55 * 22 * 12 - 12);
    for (table, bound) in [(&worst, BOUND_DEG), (&worst_far, FAR_BOUND_DEG)] {
        for (name, (apart, where_)) in table {
            // Sunshine trisects the Sun's arcs and is ill-conditioned where the
            // Sun barely rises (Fairbanks in June); the baseline's Sun is
            // topocentric, so its declination differs by arcseconds.
            let bound = if name == "sunshine" { 0.1 } else { bound };
            assert!(*apart < bound, "{name}: {apart}° at {where_}");
        }
    }
    record_measurements(&worst, &worst_far, near_compared, far_compared);
}

/// The worst differences for the accuracy document: the systems other than
/// Sunshine over the two spans, and Sunshine on its own bound.
fn record_measurements(
    worst: &BTreeMap<String, (f64, String)>,
    worst_far: &BTreeMap<String, (f64, String)>,
    near_compared: usize,
    far_compared: usize,
) {
    let worst_of = |table: &BTreeMap<String, (f64, String)>, sunshine: bool| {
        table
            .iter()
            .filter(|(name, _)| (name.as_str() == "sunshine") == sunshine)
            .map(|(_, (apart, _))| *apart)
            .fold(0.0, f64::max)
    };
    common::record(
        "houses",
        "cusps against the baseline's charts, 1800 to 2200",
        worst_of(worst, false),
        "°",
        BOUND_DEG,
        near_compared,
    );
    common::record(
        "houses",
        "cusps against the baseline's charts beyond 2200",
        worst_of(worst_far, false),
        "°",
        FAR_BOUND_DEG,
        far_compared,
    );
    common::record(
        "houses",
        "Sunshine cusps against the baseline's topocentric Sun",
        worst_of(worst, true).max(worst_of(worst_far, true)),
        "°",
        0.1,
        near_compared + far_compared,
    );
}
