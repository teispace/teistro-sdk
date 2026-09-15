//! A chart document's Vimshopaka, computed end to end: the corpus's first
//! chart founded with the built-in ephemeris, scored under the conformance
//! profile's reading (the engine's virupas) against what the corpus
//! recorded, and under the default profile's (BPHS's points)
//! (`docs/03-design/vimshopaka-measured.md`).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index JSON by key"
)]

mod common;

use common::{fixture, reading};
use teistro::settings::Vimshopaka;

#[test]
fn a_reading_carries_the_vimshopaka_the_corpus_recorded() {
    let recorded = fixture("vimshopaka/charts/c001-kathmandu-1990-04-14.json");
    let (_, document) = reading("{}", teistro::ChartRequest::with_vimshopaka);
    assert!(document.sections().contains(&"vimshopaka"));
    let vimshopaka = document.vimshopaka.expect("the section asked for");
    assert_eq!(
        vimshopaka.scoring,
        Vimshopaka::SaptavargajaVirupas,
        "the conformance profile takes the engine's scoring"
    );
    assert_eq!(vimshopaka.grahas.len(), 7);
    for graha in &vimshopaka.grahas {
        let scores = &recorded["scores"][graha.graha.key()];
        let ours = [
            ("shadvarga", graha.shadvarga),
            ("saptavarga", graha.saptavarga),
            ("dashavarga", graha.dashavarga),
            ("shodashavarga", graha.shodashavarga),
        ];
        for (key, value) in ours {
            assert!(
                (value - scores[key].as_f64().unwrap()).abs() < 1e-9,
                "{:?} {key}: {value} against {}",
                graha.graha,
                scores[key]
            );
        }
    }
}

#[test]
fn the_text_s_points_score_every_scheme_within_20() {
    let (_, document) = reading(
        r#"{"strength": {"vimshopaka": "BPHS"}}"#,
        teistro::ChartRequest::with_vimshopaka,
    );
    let vimshopaka = document.vimshopaka.expect("the section asked for");
    assert_eq!(vimshopaka.scoring, Vimshopaka::Bphs);
    for graha in &vimshopaka.grahas {
        for score in [
            graha.shadvarga,
            graha.saptavarga,
            graha.dashavarga,
            graha.shodashavarga,
        ] {
            // The text's least is 5 points of 20 in every varga; its most 20.
            assert!((5.0..=20.0).contains(&score), "{:?}: {score}", graha.graha);
        }
    }
}
