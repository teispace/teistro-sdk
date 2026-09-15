//! The engine's readings against every recorded one: the Ashtakavarga's
//! bindus, sums and, under the engine's classical Ekadhipatya, the reduced
//! sum and every pinda (`docs/03-design/ashtakavarga-measured.md`); and the
//! Vimshopaka's four scores (`docs/03-design/vimshopaka-measured.md`); and
//! the Shadbala's every component (`docs/03-design/shadbala-measured.md`).

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index JSON by key"
)]

use std::path::Path;

use serde_json::Value;
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::settings::{Ekadhipatya, Shodhana, Vimshopaka};
use teistro_strength::ashtakavarga::{AshtakavargaChart, AshtakavargaReading, AshtakavargaRules};
use teistro_strength::shadbala::{
    SAPTAVARGAJA_VARGAS, ShadbalaChart, ShadbalaGraha, ShadbalaReading, ShadbalaRules,
};
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

fn fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/baseline")
        .join(name);
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// A recorded Shadbala's inputs as the kernel reads them: the day the
/// engine's own sunrise, sunset and next sunrise put the birth in, the
/// previous day's for a birth before the recorded sunrise.
fn shadbala_chart(file: &Value) -> ShadbalaChart {
    const WEEKDAY_LORDS: [Graha; 7] = [
        Graha::Moon,
        Graha::Mars,
        Graha::Mercury,
        Graha::Jupiter,
        Graha::Venus,
        Graha::Saturn,
        Graha::Sun,
    ];
    let inputs = &file["inputs"];
    let number = |key: &str| inputs[key].as_f64().unwrap();
    let recorded = fixture(file["fixture"].as_str().unwrap());
    let ayanamsha = number("ayanamsha_deg");
    let instant = number("jd_ut");
    let after_midnight = instant < number("sunrise_jd");
    let previous = &recorded["foundation"]["previous_day"];
    let (sunrise, sunset, next_sunrise) = if after_midnight {
        (
            previous["sunrise_jd"].as_f64().unwrap(),
            previous["sunset_jd"].as_f64().unwrap(),
            number("sunrise_jd"),
        )
    } else {
        (
            number("sunrise_jd"),
            number("sunset_jd"),
            number("next_sunrise_jd"),
        )
    };
    let lord = |key: &str| inputs[key].as_str().and_then(Graha::from_key);
    ShadbalaChart {
        grahas: GRAHAS.map(|g| {
            let at = &inputs["grahas"][g];
            let longitude = at["longitude_deg"].as_f64().unwrap();
            ShadbalaGraha {
                longitude,
                tropical: longitude + ayanamsha,
                latitude: 0.0,
                house: u8::try_from(at["house"].as_u64().unwrap()).unwrap(),
            }
        }),
        rahu: None,
        vargas: SAPTAVARGAJA_VARGAS
            .map(|v| GRAHAS.map(|g| sign(&inputs["varga_sign_index"][v.key()][g]))),
        instant,
        sunrise,
        sunset,
        next_sunrise,
        after_midnight,
        civil_day: 0,
        weekday_lord: WEEKDAY_LORDS
            [usize::try_from(inputs["weekday_swe"].as_u64().unwrap()).unwrap()],
        hora_lord: lord("hora_lord").unwrap(),
        sankranti_lord: lord("abda_lord"),
        ascendant: number("lagna_longitude_deg"),
        midheaven: recorded["houses"]["selected"]["mc"].as_f64().unwrap(),
        ayanamsha,
        obliquity: 23.44,
    }
}

#[test]
fn every_recorded_shadbala_is_reproduced_under_the_engine_s_reading() {
    let files = files("shadbala");
    for (name, file) in &files {
        let reading = ShadbalaReading::of(&shadbala_chart(file), ShadbalaRules::RECORDING_ENGINE);
        for graha in &reading.grahas {
            let recorded = &file["shadbala"][graha.graha.key()];
            let (sthana, kaala) = (&graha.sthana, &graha.kaala);
            let ours = [
                ("sthana.uccha", sthana.uchcha),
                ("sthana.saptavargiya", sthana.saptavargaja),
                ("sthana.ojayugma", sthana.ojayugma),
                ("sthana.kendra", sthana.kendradi),
                ("sthana.drekkana", sthana.drekkana),
                ("sthana.total", sthana.total()),
                ("dig", graha.dig),
                ("kaala.nathonnatha", kaala.nathonnatha),
                ("kaala.paksha", kaala.paksha),
                ("kaala.tribhaga", kaala.tribhaga),
                ("kaala.abda", kaala.abda),
                ("kaala.masa", kaala.masa),
                ("kaala.vara", kaala.vara),
                ("kaala.hora", kaala.hora),
                ("kaala.ayana", kaala.ayana),
                ("kaala.total", kaala.total()),
                ("cheshta", graha.cheshta),
                ("naisargika", graha.naisargika),
                ("drik", graha.drik),
                ("total_shashtiamshas", graha.virupas),
                ("total_rupas", graha.rupas),
                ("minimum_rupas", graha.required_rupas),
            ];
            for (path, value) in ours {
                let expected = path
                    .split('.')
                    .fold(recorded, |v, key| &v[key])
                    .as_f64()
                    .unwrap();
                assert!(
                    (value - expected).abs() < 1e-9,
                    "{name:?} {:?} {path}: {value} against {expected}",
                    graha.graha
                );
            }
            assert_eq!(
                graha.strong,
                recorded["is_sufficient"].as_bool().unwrap(),
                "{name:?} {:?}",
                graha.graha
            );
        }
    }
    assert_eq!(files.len(), 71);
}
