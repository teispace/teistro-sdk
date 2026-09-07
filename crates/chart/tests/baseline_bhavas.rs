//! The bhavas against the recorded charts: the madhya, the sandhi and
//! every graha's placement, over all 55 charts of the conformance corpus.
//!
//! This is the first thing that reads the corpus's `houses.bhava_chalit`
//! section. It was recorded in spike 1 and has had nothing to compare
//! against until now.
//!
//! The recording engine's chalit is Vehlow whatever its label says, which
//! `cargo xtask chalit` measured over the same 55 charts and the
//! deliberate-difference registry records as entry 14. So the comparison
//! here is against **Vehlow**, and the test asserts the registry's claim
//! rather than avoiding it: Sripati is computed too, and is expected to
//! differ.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    reason = "tests fail by panicking, read fixtures and print the measurement under --nocapture"
)]

use std::path::Path;

use serde_json::Value;
use teistro_chart::bhava::{Bhavas, Chalit};
use teistro_core::angle::difference_deg;
use teistro_core::catalogue::HouseSystem;

/// The recording engine's cusps and the SDK's agree within an arcsecond
/// over these charts (`crates/astro/tests/baseline_houses.rs`); the
/// bhavas are read from the recorded cusps here, so what is compared is
/// the reading and not the astronomy, and the band is for the JSON's own
/// rounding.
const BOUND_DEG: f64 = 1e-9;

/// The grahas the engine places, in its own order.
const GRAHAS: [&str; 9] = [
    "SUN", "MOON", "MARS", "MERCURY", "JUPITER", "VENUS", "SATURN", "RAHU", "KETU",
];

fn charts() -> Vec<Value> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/baseline/charts");
    let mut charts: Vec<Value> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| {
            panic!(
                "{}: {e}. The corpus is a submodule; `git submodule update --init`",
                dir.display()
            )
        })
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

/// One system's recorded cusps, sidereal.
fn cusps(chart: &Value, system: &str) -> [f64; 12] {
    let recorded = chart["houses"]["all_systems"][system]["cusps"]
        .as_array()
        .unwrap_or_else(|| panic!("{}: no {system} cusps", chart["id"]));
    let mut out = [0.0_f64; 12];
    for (slot, value) in out.iter_mut().zip(recorded) {
        *slot = value.as_f64().expect("a number");
    }
    out
}

/// The bhavas of a chalit, from the recorded cusps of the system it reads.
fn bhavas(chart: &Value, method: HouseSystem) -> Bhavas {
    let chalit = Chalit::of(method);
    let source = match chalit.source {
        HouseSystem::Porphyry => "porphyry",
        HouseSystem::Vehlow => "vehlow",
        HouseSystem::Placidus => "placidus",
        HouseSystem::WholeSign => "whole-sign",
        other => panic!("no recorded cusps for {other:?}"),
    };
    Bhavas::of(chalit, &cusps(chart, source))
}

#[test]
fn the_madhya_and_sandhi_are_the_ones_the_engine_recorded() {
    let charts = charts();
    assert_eq!(charts.len(), 55);
    let mut worst = (0.0_f64, String::new());
    let mut compared = 0;
    for chart in &charts {
        let id = chart["id"].as_str().unwrap();
        let recorded = &chart["houses"]["bhava_chalit"];
        let ours = bhavas(chart, HouseSystem::Vehlow);
        for (name, mine) in [
            ("bhava_madhya", &ours.madhya),
            ("bhava_sandhi", &ours.sandhi),
        ] {
            let theirs = recorded[name].as_array().unwrap_or_else(|| {
                panic!("{id}: no {name}");
            });
            assert_eq!(theirs.len(), 12, "{id} {name}");
            for (index, (value, mine)) in theirs.iter().zip(mine.iter()).enumerate() {
                let apart = difference_deg(*mine, value.as_f64().expect("a number")).abs();
                compared += 1;
                if apart > worst.0 {
                    worst = (apart, format!("{id} {name}[{index}]"));
                }
            }
        }
    }
    println!(
        "bhava boundaries and middles: {compared} compared, worst {:.3e}° at {}",
        worst.0, worst.1
    );
    assert!(
        worst.0 < BOUND_DEG,
        "worst {:.3e}° at {} exceeds {BOUND_DEG:.0e}°",
        worst.0,
        worst.1
    );
}

#[test]
fn every_graha_lands_in_the_bhava_the_engine_recorded() {
    let charts = charts();
    let mut compared = 0;
    let mut differing = Vec::new();
    for chart in &charts {
        let id = chart["id"].as_str().unwrap();
        let ours = bhavas(chart, HouseSystem::Vehlow);
        let recorded = &chart["houses"]["bhava_chalit"]["planet_houses"];
        for graha in GRAHAS {
            let Some(theirs) = recorded[graha].as_u64() else {
                continue;
            };
            let Some(longitude) =
                chart["positions"]["bodies"][graha]["sidereal_longitude_deg"].as_f64()
            else {
                continue;
            };
            let mine = ours.place(longitude);
            compared += 1;
            if u64::from(mine.bhava) != theirs {
                differing.push(format!(
                    "{id} {graha}: {} against {theirs}, {:.4}° through bhava {}",
                    mine.bhava, longitude, mine.bhava
                ));
            }
            assert_eq!(
                mine.method,
                HouseSystem::Vehlow,
                "a placement names its chalit"
            );
            assert!(
                (0.0..1.0).contains(&mine.through),
                "{id} {graha}: through {} is outside its bhava",
                mine.through
            );
        }
    }
    println!("{compared} placements compared");
    assert!(
        differing.is_empty(),
        "{} placement(s) differ:\n  {}",
        differing.len(),
        differing.join("\n  ")
    );
    assert_eq!(compared, 495, "nine grahas on fifty-five charts");
}

#[test]
fn the_placement_agrees_with_the_engines_own_shifted_list() {
    // The engine records which grahas the chalit moves out of their
    // whole-sign house. That is a second, independent statement of the
    // same fact, so it is checked against the same placements.
    let charts = charts();
    let mut checked = 0;
    for chart in &charts {
        let id = chart["id"].as_str().unwrap();
        let chalit = bhavas(chart, HouseSystem::Vehlow);
        let whole_sign = bhavas(chart, HouseSystem::WholeSign);
        let Some(shifted) = chart["houses"]["bhava_chalit"]["shifted"].as_array() else {
            continue;
        };
        for entry in shifted {
            let graha = entry["planet"].as_str().expect("a name");
            let longitude = chart["positions"]["bodies"][graha]["sidereal_longitude_deg"]
                .as_f64()
                .expect("a longitude");
            assert_eq!(
                u64::from(chalit.place(longitude).bhava),
                entry["chalit_house"].as_u64().expect("a house"),
                "{id} {graha}: the chalit house"
            );
            assert_eq!(
                u64::from(whole_sign.place(longitude).bhava),
                entry["whole_sign_house"].as_u64().expect("a house"),
                "{id} {graha}: the whole-sign house"
            );
            checked += 1;
        }
    }
    println!("{checked} shifted grahas checked both ways");
    assert!(checked > 0, "the corpus records shifted grahas");
}

#[test]
fn sripati_is_a_different_chalit_and_the_registry_says_so() {
    // Entry 14 of the deliberate-difference registry: the engine computes
    // Vehlow while its documentation says Sripati. A test that avoided
    // the subject would let the claim rot; this one asserts it.
    let charts = charts();
    let mut differing = 0_u32;
    let mut compared = 0_u32;
    for chart in &charts {
        let vehlow = bhavas(chart, HouseSystem::Vehlow);
        let sripati = bhavas(chart, HouseSystem::Sripati);
        for graha in GRAHAS {
            let Some(longitude) =
                chart["positions"]["bodies"][graha]["sidereal_longitude_deg"].as_f64()
            else {
                continue;
            };
            compared += 1;
            if vehlow.place(longitude).bhava != sripati.place(longitude).bhava {
                differing += 1;
            }
        }
    }
    let percent = f64::from(differing) / f64::from(compared) * 100.0;
    println!("Sripati against Vehlow: {differing} of {compared} placements, {percent:.1}%");
    // `cargo xtask chalit` measures 21.8% over the same charts from the
    // SDK's own cusps; the recorded cusps give the same answer, and the
    // band is wide enough that a rounding change does not fail the build
    // while a wrong reading does.
    assert!(
        (20.0..24.0).contains(&percent),
        "the two chalits disagree on {percent:.1}% of placements, which is not the measured 21.8%"
    );

    // And Sripati's own madhya are Porphyry's cusps, which is the whole
    // difference between the two readings.
    for chart in charts.iter().take(3) {
        let sripati = bhavas(chart, HouseSystem::Sripati);
        for (mine, theirs) in sripati.madhya.iter().zip(cusps(chart, "porphyry")) {
            assert!(
                difference_deg(*mine, theirs).abs() < BOUND_DEG,
                "{}: Sripati's madhya are Porphyry's cusps",
                chart["id"]
            );
        }
        // And its sandhi are what `astro` returns as the `SRIPATI` system.
        for (mine, theirs) in sripati.sandhi.iter().zip(cusps(chart, "sripati")) {
            assert!(
                difference_deg(*mine, theirs).abs() < BOUND_DEG,
                "{}: Sripati's sandhi are the engine's SRIPATI cusps",
                chart["id"]
            );
        }
    }
}
