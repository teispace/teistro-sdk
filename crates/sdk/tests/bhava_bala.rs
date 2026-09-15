//! A chart document's Bhava bala, computed end to end: the corpus's first
//! chart founded with the built-in ephemeris, read under the conformance
//! profile's reading (the engine's) against every house the corpus recorded,
//! and under the default profile's (BPHS ch. 27's), whose special rules only it
//! adds (`docs/03-design/bhava-bala-measured.md`).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index JSON by key"
)]

mod common;

use common::{fixture, reading};
use teistro::strength::BhavaBalaRules;

/// How far the built-in ephemeris and the engine's hundredths may move a
/// house's strength from the recorded one.
const WITHIN: f64 = 0.02;

#[test]
fn a_reading_carries_the_bhava_bala_the_corpus_recorded() {
    let recorded = fixture("bhava-bala/charts/c001-kathmandu-1990-04-14.json");
    let (_, document) = reading("{}", teistro::ChartRequest::with_bhava_bala);
    assert!(document.sections().contains(&"bhava_bala"));
    assert!(
        document.shadbala.is_none(),
        "the Shadbala it reads was not asked for"
    );
    let bhava_bala = document.bhava_bala.expect("the section asked for");
    assert_eq!(bhava_bala.rules, BhavaBalaRules::RECORDING_ENGINE);
    for (ours, house) in bhava_bala
        .bhavas
        .iter()
        .zip(recorded["bhavas"].as_array().unwrap())
    {
        for (key, value) in [
            ("bhavadhipati", ours.adhipati),
            ("dig", ours.dig),
            ("drishti", ours.drishti),
            ("total", ours.virupas),
        ] {
            let expected = house[key].as_f64().unwrap();
            assert!(
                (value - expected).abs() < WITHIN,
                "bhava {} {key}: {value} against {expected}",
                ours.bhava
            );
        }
    }
}

#[test]
fn the_text_s_reading_adds_the_special_rules() {
    let (_, document) = reading(
        r#"{"strength": {"bhava_dig": "BPHS", "bhava_drishti": "QUARTER_OF_DIG", "bhava_special_rules": "BPHS"}}"#,
        |request| request.with_shadbala().with_bhava_bala(),
    );
    let bhava_bala = document.bhava_bala.expect("the section asked for");
    assert_eq!(bhava_bala.bhavas.len(), 12);
    let shadbala = document.shadbala.expect("asked for beside it");
    for bhava in &bhava_bala.bhavas {
        let lord = shadbala
            .grahas
            .iter()
            .find(|g| g.graha == bhava.lord)
            .unwrap();
        assert!(
            (bhava.adhipati - lord.virupas).abs() < 1e-9,
            "the lord's own Shadbala"
        );
        assert!(
            (bhava.virupas - (bhava.adhipati + bhava.dig + bhava.drishti + bhava.special)).abs()
                < 1e-9
        );
    }
    assert!(bhava_bala.bhavas.iter().any(|b| b.special != 0.0));
}
