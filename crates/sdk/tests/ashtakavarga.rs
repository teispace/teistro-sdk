//! A chart document's Ashtakavarga, computed end to end: the corpus's first
//! chart founded with the built-in ephemeris, read under the conformance
//! profile's reading (the engine's) against what the corpus recorded, and
//! under the default profile's (BPHS's), whose bindus are the same and whose
//! reductions are each graha's own (`docs/03-design/ashtakavarga-measured.md`).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

mod common;

use common::{fixture, reading};
use serde_json::Value;
use teistro::settings::{Ekadhipatya, Shodhana};

const GRAHAS: [&str; 7] = [
    "SUN", "MOON", "MARS", "MERCURY", "JUPITER", "VENUS", "SATURN",
];

fn numbers(value: &Value) -> Vec<u64> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap())
        .collect()
}

#[test]
fn a_reading_carries_the_ashtakavarga_the_corpus_recorded() {
    let recorded = fixture("ashtakavarga/charts/c001-kathmandu-1990-04-14.json");
    let (_, document) = reading("{}", teistro::ChartRequest::with_ashtakavarga);
    assert!(document.sections().contains(&"ashtakavarga"));
    let ashtakavarga = document.ashtakavarga.expect("the section asked for");
    assert_eq!(
        (ashtakavarga.rules.shodhana, ashtakavarga.rules.ekadhipatya),
        (Shodhana::Sarva, Ekadhipatya::EmptyToZero),
        "the conformance profile takes the engine's reading"
    );
    let classical = &recorded["ekadhipatya"]["classical"];
    for (graha, key) in ashtakavarga.grahas.iter().zip(GRAHAS) {
        assert_eq!(
            graha.bindus.map(u64::from).to_vec(),
            numbers(&recorded["bav"][key]),
            "{key}"
        );
        assert_eq!(
            u64::from(graha.yoga_pinda),
            classical["yoga_pinda"][key].as_u64().unwrap(),
            "{key}"
        );
    }
    assert_eq!(
        ashtakavarga.reduced.map(u64::from).to_vec(),
        numbers(&classical["sav_reduced"])
    );
}

#[test]
fn the_text_s_reading_reduces_each_graha_s_own_ashtakavarga() {
    let (_, document) = reading(
        r#"{"strength": {"shodhana": "EACH_GRAHA", "ekadhipatya": "BPHS"}}"#,
        teistro::ChartRequest::with_ashtakavarga,
    );
    let ashtakavarga = document.ashtakavarga.expect("the section asked for");
    assert!(
        ashtakavarga
            .grahas
            .iter()
            .all(|graha| graha.reduced.is_some())
    );
    assert_eq!(
        ashtakavarga
            .sarva
            .iter()
            .map(|b| u32::from(*b))
            .sum::<u32>(),
        337
    );
    // The sum of the grahas' own reductions is the reduced sum.
    for sign in 0..12 {
        let own: u32 = ashtakavarga
            .grahas
            .iter()
            .map(|graha| u32::from(graha.reduced.unwrap()[sign]))
            .sum();
        assert_eq!(own, u32::from(ashtakavarga.reduced[sign]));
    }
}
