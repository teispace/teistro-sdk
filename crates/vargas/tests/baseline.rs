//! Every divisional placement the conformance corpus records.
//!
//! This is the strongest test in the project, and it is strong because
//! of what a varga is: a function of one sidereal longitude. The corpus
//! records the longitude and the answer, so nothing here is a comparison
//! within a tolerance — every one of the placements is right or wrong,
//! and the assertion is equality.
//!
//! The rows come from twice as far as usual. The 55 recorded days carry
//! twenty-one charts each, and 38 of the variant fixtures carry them
//! again under another ayanamsha, another node, another centre or the
//! **tropical** zodiac — which moves every longitude by twenty-four
//! degrees and so lands the bodies in different parts of different signs.
//! A rule that survives both is not fitted to one zodiac.
//!
//! `cargo xtask vargas` derives the same tables from the same corpus
//! without using this crate at all, and writes what it found to
//! `03-design/varga-tables-measured.md`. That pass and this test are
//! deliberately two: one falsifies the design before the code exists, and
//! this one holds the code to it afterwards.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    reason = "tests fail by panicking, index JSON by key and print the measurement under --nocapture"
)]

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::Value;
use teistro_core::angle::Nas;
use teistro_core::catalogue::{Rashi, Varga};
use teistro_core::quantity::Degrees;
use teistro_vargas::{Scheme, place, sign};

/// The bodies the engine places in every divisional chart. `LAGNA` is one
/// of them, which is what makes the vargottama finding visible.
const BODIES: [&str; 10] = [
    "SUN", "MOON", "MARS", "MERCURY", "JUPITER", "VENUS", "SATURN", "RAHU", "KETU", "LAGNA",
];

/// Every fixture that carries divisional charts, from both directories.
fn fixtures() -> Vec<(String, Value)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/baseline");
    let mut found = Vec::new();
    for directory in ["charts", "variants"] {
        let dir = root.join(directory);
        let mut paths: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
            .unwrap_or_else(|e| {
                panic!(
                    "{}: {e}. The corpus is a submodule; `git submodule update --init`",
                    dir.display()
                )
            })
            .map(|entry| entry.expect("an entry").path())
            .filter(|path| path.extension().is_some_and(|e| e == "json"))
            .collect();
        paths.sort();
        for path in &paths {
            let value: Value =
                serde_json::from_str(&std::fs::read_to_string(path).expect("a fixture"))
                    .expect("valid JSON");
            if value["vargas"].is_object() {
                found.push((
                    path.file_stem()
                        .map(|stem| stem.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    value,
                ));
            }
        }
    }
    assert!(!found.is_empty(), "the corpus carries divisional charts");
    found
}

/// A recorded longitude as the canonical angle.
fn at(degrees: f64) -> Nas {
    Nas::from_degrees(Degrees::try_new(degrees.rem_euclid(360.0)).expect("a finite longitude"))
}

#[test]
fn every_recorded_placement_is_the_one_the_kernel_gives() {
    let mut checked = 0;
    let mut by_varga: BTreeMap<Varga, usize> = BTreeMap::new();
    let mut fixtures_seen = 0;
    for (name, fixture) in fixtures() {
        fixtures_seen += 1;
        let charts = fixture["vargas"].as_object().expect("a section");
        for (key, chart) in charts {
            let varga = Varga::from_key(key)
                .unwrap_or_else(|| panic!("{name}: the catalogue has no varga {key}"));
            let scheme = Scheme::of(varga);
            // The engine states the chart's own number beside it, and it
            // is the catalogue's.
            assert_eq!(
                chart["division"].as_u64(),
                Some(u64::from(scheme.divisions)),
                "{name} {key}: the divisions"
            );
            for body in BODIES {
                let (Some(longitude), Some(recorded)) = (
                    fixture["positions"]["bodies"][body]["sidereal_longitude_deg"].as_f64(),
                    chart["sign_index"][body].as_u64(),
                ) else {
                    continue;
                };
                let expected = Rashi::from_id(u16::try_from(recorded).expect("a sign index"))
                    .unwrap_or_else(|| panic!("{name} {key} {body}: sign {recorded}"));
                assert_eq!(
                    sign(&scheme, at(longitude)),
                    expected,
                    "{name} {key} {body} at {longitude}°"
                );
                checked += 1;
                *by_varga.entry(varga).or_default() += 1;
            }
        }
    }
    println!("{checked} placements over {fixtures_seen} fixtures");
    assert_eq!(fixtures_seen, 93, "55 charts and 38 variants");
    assert_eq!(
        checked, 19_530,
        "ten bodies in twenty-one charts, ninety-three times"
    );
    assert_eq!(by_varga.len(), Varga::ALL.len(), "every chart is exercised");
    assert!(
        by_varga.values().all(|count| *count == 930),
        "every chart is exercised equally: {by_varga:?}"
    );
}

#[test]
fn the_rashi_chart_reproduces_the_engines_own_sign_index() {
    // D1 is the identity, so the corpus's own `sign_index` for a body is
    // both the input and the answer — which makes this a check of the
    // classification itself and not of the varga rule.
    let mut checked = 0;
    for (name, fixture) in fixtures() {
        for body in BODIES {
            let position = &fixture["positions"]["bodies"][body];
            let (Some(longitude), Some(recorded)) = (
                position["sidereal_longitude_deg"].as_f64(),
                position["sign_index"].as_u64(),
            ) else {
                continue;
            };
            let expected =
                Rashi::from_id(u16::try_from(recorded).expect("a sign index")).expect("a sign");
            assert_eq!(
                sign(&Scheme::of(Varga::D1), at(longitude)),
                expected,
                "{name} {body} at {longitude}°"
            );
            checked += 1;
        }
    }
    println!("{checked} sign classifications");
    assert!(checked >= 900);
}

#[test]
fn vargottama_is_the_engines_flag_for_the_grahas_and_more_for_the_lagna() {
    // Entry 20 of the deliberate-difference registry: the engine never
    // marks the lagna vargottama, and on two recorded charts the lagna's
    // navamsha sign *is* its rashi sign. Asserted as a difference rather
    // than skipped.
    let navamsha = Scheme::of(Varga::D9);
    let mut grahas = 0;
    let mut lagnas = 0;
    let mut lagna_qualifies = Vec::new();
    for (name, fixture) in fixtures() {
        for body in BODIES {
            let position = &fixture["positions"]["bodies"][body];
            let (Some(longitude), Some(flagged)) = (
                position["sidereal_longitude_deg"].as_f64(),
                position["is_vargottama"].as_bool(),
            ) else {
                continue;
            };
            let ours = place(&navamsha, at(longitude)).keeps_its_sign();
            if body == "LAGNA" {
                lagnas += 1;
                assert!(!flagged, "{name}: the engine never marks a lagna");
                if ours {
                    lagna_qualifies.push(name.clone());
                }
            } else {
                assert_eq!(ours, flagged, "{name} {body} at {longitude}°");
                grahas += 1;
            }
        }
    }
    println!(
        "{grahas} grahas agree; {lagnas} lagnas, of which {} are vargottama and none is flagged: {}",
        lagna_qualifies.len(),
        lagna_qualifies.join(", ")
    );
    assert_eq!(grahas, 837, "nine grahas on ninety-three fixtures");
    assert_eq!(lagnas, 93);
    assert_eq!(
        lagna_qualifies.len(),
        2,
        "the two the registry records, asserted as a difference"
    );
}

#[test]
fn the_tropical_variants_exercise_the_same_rules() {
    // A tropical fixture's longitudes are twenty-four degrees from the
    // sidereal ones, so its bodies fall in different parts of different
    // signs. If the rules were fitted to one zodiac these would fail.
    let mut checked = 0;
    let mut tropical = 0;
    for (name, fixture) in fixtures() {
        if fixture["settings"]["profile"].as_str() != Some("tropical") {
            continue;
        }
        tropical += 1;
        for (key, chart) in fixture["vargas"].as_object().expect("a section") {
            let scheme = Scheme::of(Varga::from_key(key).expect("a catalogued chart"));
            for body in BODIES {
                let (Some(longitude), Some(recorded)) = (
                    fixture["positions"]["bodies"][body]["sidereal_longitude_deg"].as_f64(),
                    chart["sign_index"][body].as_u64(),
                ) else {
                    continue;
                };
                let expected = Rashi::from_id(u16::try_from(recorded).unwrap()).expect("a sign");
                assert_eq!(
                    sign(&scheme, at(longitude)),
                    expected,
                    "{name} {key} {body}"
                );
                checked += 1;
            }
        }
    }
    println!("{checked} placements over {tropical} tropical fixtures");
    assert_eq!(tropical, 4, "the corpus records four");
    assert_eq!(checked, 4 * 21 * 10);
}
