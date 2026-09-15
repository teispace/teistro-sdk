//! The engine's readings against every recorded one: the Ashtakavarga's
//! bindus, sums and, under the engine's classical Ekadhipatya, the reduced
//! sum and every pinda (`docs/03-design/ashtakavarga-measured.md`); and the
//! Vimshopaka's four scores (`docs/03-design/vimshopaka-measured.md`).

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index JSON by key"
)]

use std::path::Path;

use serde_json::Value;
use teistro_core::catalogue::Rashi;
use teistro_core::settings::{Ekadhipatya, Shodhana, Vimshopaka};
use teistro_strength::ashtakavarga::{AshtakavargaChart, AshtakavargaReading, AshtakavargaRules};
use teistro_strength::vimshopaka::{VARGAS, VimshopakaChart, VimshopakaReading};

const GRAHAS: [&str; 7] = [
    "SUN", "MOON", "MARS", "MERCURY", "JUPITER", "VENUS", "SATURN",
];

fn sign(value: &Value) -> Rashi {
    Rashi::from_id(u16::try_from(value.as_u64().unwrap()).unwrap()).unwrap()
}

fn numbers(value: &Value) -> Vec<u64> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap())
        .collect()
}

/// Every recorded file under a baseline directory, by name.
fn files(system: &str) -> Vec<(std::ffi::OsString, Value)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/baseline")
        .join(system);
    let mut out = Vec::new();
    for dir in ["charts", "variants"] {
        for entry in std::fs::read_dir(root.join(dir))
            .unwrap_or_else(|err| panic!("{dir}: {err}; is the fixtures submodule checked out?"))
            .flatten()
        {
            let file =
                serde_json::from_str(&std::fs::read_to_string(entry.path()).unwrap()).unwrap();
            out.push((entry.file_name(), file));
        }
    }
    out
}

#[test]
fn every_recorded_ashtakavarga_is_reproduced_under_the_engine_s_reading() {
    let rules = AshtakavargaRules {
        shodhana: Shodhana::Sarva,
        ekadhipatya: Ekadhipatya::EmptyToZero,
    };
    let files = files("ashtakavarga");
    for (name, file) in &files {
        let chart = AshtakavargaChart {
            lagna: sign(&file["inputs"]["lagna_sign_index"]),
            signs: GRAHAS.map(|g| sign(&file["inputs"]["graha_sign_index"][g])),
        };
        let reading = AshtakavargaReading::of(&chart, rules);
        let recorded = &file["ekadhipatya"]["classical"];
        for (graha, key) in reading.grahas.iter().zip(GRAHAS) {
            assert_eq!(
                graha.bindus.map(u64::from).to_vec(),
                numbers(&file["bav"][key]),
                "{name:?} {key}: bindus"
            );
            assert_eq!(
                u64::from(graha.graha_pinda),
                recorded["graha_pinda"][key].as_u64().unwrap(),
                "{name:?} {key}"
            );
            assert_eq!(
                u64::from(graha.yoga_pinda),
                recorded["yoga_pinda"][key].as_u64().unwrap(),
                "{name:?} {key}"
            );
        }
        assert_eq!(
            reading.sarva.map(u64::from).to_vec(),
            numbers(&file["sav"]),
            "{name:?}: sum"
        );
        assert_eq!(
            reading.trikona.map(u64::from).to_vec(),
            numbers(&file["sav_trikona"]),
            "{name:?}: trine"
        );
        assert_eq!(
            reading.reduced.map(u64::from).to_vec(),
            numbers(&recorded["sav_reduced"]),
            "{name:?}: reduced"
        );
    }
    assert_eq!(files.len(), 77);
}

#[test]
fn every_recorded_vimshopaka_is_reproduced_under_the_engine_s_reading() {
    let files = files("vimshopaka");
    for (name, file) in &files {
        let chart = VimshopakaChart::from_fn(|varga, graha| {
            sign(&file["inputs"]["sign_index"][varga.key()][graha.key()])
        });
        assert_eq!(
            file["inputs"]["sign_index"].as_object().unwrap().len(),
            VARGAS.len()
        );
        let reading = VimshopakaReading::of(&chart, Vimshopaka::SaptavargajaVirupas);
        for graha in &reading.grahas {
            let recorded = &file["scores"][graha.graha.key()];
            let ours = [
                graha.shadvarga,
                graha.saptavarga,
                graha.dashavarga,
                graha.shodashavarga,
            ];
            for (key, value) in ["shadvarga", "saptavarga", "dashavarga", "shodashavarga"]
                .iter()
                .zip(ours)
            {
                assert!(
                    (value - recorded[*key].as_f64().unwrap()).abs() < 1e-9,
                    "{name:?} {:?} {key}: {value} against {}",
                    graha.graha,
                    recorded[*key]
                );
            }
        }
    }
    assert_eq!(files.len(), 93);
}
