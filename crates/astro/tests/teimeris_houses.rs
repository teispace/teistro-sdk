//! The SDK's house systems against Teimeris's recorded cusps and angles
//! (`fixtures/teimeris/houses.json`, written by the adapter's `houses-table`
//! binary): twenty-one systems at ten latitudes from the southern polar
//! circle to 80° north, two longitudes and three instants, the polar rows
//! included. Rank 2 reference values with a tolerance; the bounds are the
//! measured agreement, published in `docs/03-design/astro-house-systems.md`.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    reason = "tests fail by panicking, read a recorded table and print the measurement under --nocapture"
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

/// The cusps and angles against the engine's, degrees: measured 4e-6° at
/// worst (Koch's sixth cusp at 64.8°, where the construction is steep) and
/// 2.2e-6° on the vertex in the tropics, the two sidereal times' and
/// obliquities' milliarcseconds amplified by the geometry; bounded at a
/// hundredth of an arcsecond.
const BOUND_DEG: f64 = 1e-5;

fn fixture() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/teimeris/houses.json");
    serde_json::from_str(&std::fs::read_to_string(path).expect("the recorded table"))
        .expect("valid JSON")
}

#[test]
fn every_system_reproduces_the_engines_cusps_and_angles_at_ten_latitudes() {
    let table = fixture();
    assert_eq!(table["zodiac"], "TROPICAL");
    let mut worst: BTreeMap<&str, (f64, String)> = BTreeMap::new();
    let mut compared = 0;
    let mut substituted = 0;
    for row in table["rows"].as_array().unwrap() {
        let key = row["system"].as_str().unwrap();
        let system = HouseSystem::from_key(key).expect("a catalogued key");
        let ut1 = JulianDay::<Ut1>::literal(row["jd_ut1"].as_f64().unwrap());
        let (tt, _) = tt_of(ut1, DeltaTModel::TableThenModel).unwrap();
        let latitude = row["latitude_deg"].as_f64().unwrap();
        let place = Place::new(
            Latitude::literal(latitude),
            Longitude::literal(row["longitude_deg"].as_f64().unwrap()),
            Altitude::literal(0.0),
        );
        let ours = houses_at(
            system,
            ut1,
            tt,
            &place,
            &ChartFrame::default(),
            PolarPolicy::FallbackPorphyry,
        )
        .unwrap_or_else(|e| panic!("{key} at {latitude}: {e}"));
        // The engine substitutes Porphyry where a system is undefined; so does
        // the SDK under this policy, and both must agree on when.
        let engine_substituted = row["substituted"].as_bool().unwrap();
        assert_eq!(
            ours.outcome != Outcome::Defined,
            engine_substituted,
            "{key} at {latitude}: SDK {:?}, engine substituted {engine_substituted}",
            ours.outcome
        );
        if engine_substituted {
            substituted += 1;
        }
        let entry = worst.entry(key).or_insert((0.0, String::new()));
        let mut note = |mine: f64, theirs: f64, what: &str| {
            let apart = difference_deg(mine, theirs).abs();
            if apart > entry.0 {
                *entry = (
                    apart,
                    format!("{what} at latitude {latitude}, JD {}", ut1.get()),
                );
            }
            compared += 1;
        };
        for (house, theirs) in row["cusps"].as_array().unwrap().iter().enumerate() {
            note(
                ours.cusps[house],
                theirs.as_f64().unwrap(),
                &format!("house {}", house + 1),
            );
        }
        let angles = &ours.angles;
        for (mine, field) in [
            (angles.ascendant_deg, "ascendant_deg"),
            (angles.midheaven_deg, "midheaven_deg"),
            (angles.armc_deg, "armc_deg"),
            (angles.vertex_deg, "vertex_deg"),
            (angles.equatorial_ascendant_deg, "equatorial_ascendant_deg"),
            (angles.co_ascendant_koch_deg, "co_ascendant_koch_deg"),
            (
                angles.co_ascendant_munkasey_deg,
                "co_ascendant_munkasey_deg",
            ),
            (angles.polar_ascendant_deg, "polar_ascendant_deg"),
        ] {
            // The horizon system's transform of the latitude leaves the
            // engine's restored value a hair below zero at the equator, so
            // its Munkasey co-ascendant there takes the southern branch, a
            // point whose pole height is 90° and which is degenerate anyway.
            if system == HouseSystem::Horizon
                && latitude == 0.0
                && field == "co_ascendant_munkasey_deg"
            {
                continue;
            }
            note(mine, row[field].as_f64().unwrap(), field);
        }
    }
    for (key, (apart, where_)) in &worst {
        println!("{key:<14} worst {apart:.9}° {where_}");
    }
    println!("{compared} values compared, {substituted} rows substituted by both");
    // Twenty-one systems, sixty rows each, twenty values a row, less the six
    // equatorial Horizon rows' Munkasey co-ascendant.
    assert_eq!(compared, 21 * 60 * 20 - 6);
    for (key, (apart, where_)) in &worst {
        assert!(*apart < BOUND_DEG, "{key}: {apart}° {where_}");
    }
    common::record(
        "houses",
        "cusps and angles against Teimeris",
        worst.values().map(|(apart, _)| *apart).fold(0.0, f64::max),
        "°",
        BOUND_DEG,
        compared,
    );
}
