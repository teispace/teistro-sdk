//! The SDK's ayanamsha catalogue against Teimeris's recorded values
//! (`fixtures/teimeris/ayanamsha.json`, written by the adapter's
//! `ayanamsha-table` binary): every epoch-defined and frame member the
//! engine offers, over Julian epochs −700 to 2500. Rank 2 reference
//! values with a tolerance (`CLEAN_ROOM.md`); the bounds are the measured
//! agreement, published in `docs/03-design/astro-ayanamsha-catalogue.md`.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    reason = "tests fail by panicking, read a recorded table and print the measurement under --nocapture"
)]

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::Value;
use teistro_astro::ayanamsha::{Definition, EpochScale, definition, mean_deg};
use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::precession::PrecessionModel;
use teistro_core::angle::difference_deg;
use teistro_core::catalogue::Ayanamsha;
use teistro_core::quantity::{JulianDay, Tt};

/// A definition stated in TT agrees with the engine to the rounding of the
/// construction: a hundredth of a milliarcsecond.
const TT_EPOCH_BOUND_ARCSEC: f64 = 1e-5;

/// A definition stated in Universal Time takes the SDK's Delta T at its
/// epoch, which differs from the engine's by a few seconds in antiquity:
/// measured 2.1e-4″ of precession at worst (the Surya Siddhanta epoch of
/// 499 CE and the Sassanian one), bounded at half a milliarcsecond.
const UT_EPOCH_BOUND_ARCSEC: f64 = 5e-4;

fn fixture() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/teimeris/ayanamsha.json");
    serde_json::from_str(&std::fs::read_to_string(path).expect("the recorded table"))
        .expect("valid JSON")
}

#[test]
fn the_catalogue_reproduces_the_engines_mean_values() {
    let table = fixture();
    assert_eq!(table["basis"], "MEAN");
    assert_eq!(table["scale"], "TT");
    let mut worst: BTreeMap<&str, (f64, f64)> = BTreeMap::new();
    let mut compared = 0;
    for row in table["rows"].as_array().unwrap() {
        let key = row["ayanamsha"].as_str().unwrap();
        let id = Ayanamsha::from_key(key).expect("a catalogued key");
        let (Definition::Epoch(epoch) | Definition::Frame(epoch)) = definition(id) else {
            continue;
        };
        let jd = row["jd_tt"].as_f64().unwrap();
        let theirs = row["mean_deg"].as_f64().unwrap();
        let ours = mean_deg(
            &id.into(),
            JulianDay::<Tt>::literal(jd),
            PrecessionModel::Vondrak2011,
            DeltaTModel::TableThenModel,
        )
        .expect("an epoch-defined ayanamsha");
        let arcsec = difference_deg(ours, theirs) * 3600.0;
        let bound = match epoch.scale {
            EpochScale::Tt => TT_EPOCH_BOUND_ARCSEC,
            EpochScale::Ut => UT_EPOCH_BOUND_ARCSEC,
        };
        let entry = worst.entry(key).or_insert((0.0, jd));
        if arcsec.abs() > entry.0.abs() {
            *entry = (arcsec, jd);
        }
        assert!(
            arcsec.abs() <= bound,
            "{key} at JD {jd}: SDK {ours} against Teimeris {theirs}: {arcsec:+.6}\" (bound {bound}\")"
        );
        compared += 1;
    }
    assert!(compared >= 30 * 20, "{compared} rows compared");
    for (key, (arcsec, jd)) in &worst {
        println!("{key:<22} worst {arcsec:+.7}\" at JD {jd}");
    }
}
