//! The Rust newtypes held to the shared validator corpus
//! (`03-design/validator-corpus.md`): every value `idl/validators.json`
//! lists as valid is accepted by the constructor of the quantity it
//! names, and every invalid one refused. The bindings' constructors are
//! generated from the description's ranges and read the same file in
//! their own suites; this is what holds the description to Rust, whose
//! newtypes keep their own constants.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "a test fails by panicking"
)]

use teistro_core::quantity::{Altitude, Latitude, Longitude};

/// Whether the quantity's own constructor accepts `value`.
fn accepts(brand: &str, value: f64) -> bool {
    match brand {
        "latitude" => Latitude::try_new(value).is_ok(),
        "longitude" => Longitude::try_new(value).is_ok(),
        "altitude" => Altitude::try_new(value).is_ok(),
        other => panic!("`{other}` is branded in the description and has no Rust constructor here"),
    }
}

#[test]
fn every_rust_constructor_agrees_with_the_corpus() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../idl/validators.json");
    let corpus: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(corpus["schema"], "teistro-validators/1");
    let brands = corpus["brands"].as_array().unwrap();
    assert!(!brands.is_empty(), "the corpus names no brand");
    for entry in brands {
        let brand = entry["brand"].as_str().unwrap();
        for (list, wanted) in [("valid", true), ("invalid", false)] {
            for text in entry[list].as_array().unwrap() {
                let text = text.as_str().unwrap();
                let value: f64 = text.parse().unwrap();
                assert_eq!(
                    accepts(brand, value),
                    wanted,
                    "{brand}({text}) should be {}",
                    if wanted { "accepted" } else { "refused" }
                );
            }
        }
    }
}
