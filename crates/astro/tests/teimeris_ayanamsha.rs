//! The SDK's ayanamsha catalogue against Teimeris's recorded values
//! (`fixtures/teimeris/ayanamsha.json`, written by the adapter's
//! `ayanamsha-table` binary): every member the engine offers, epoch-defined,
//! frame or anchored, over Julian epochs −700 to 2500. Rank 2 reference
//! values with a tolerance (`CLEAN_ROOM.md`); the bounds are the measured
//! agreement, published in `docs/03-design/astro-ayanamsha-catalogue.md`.
//! An anchored member's bound is the astrometry the two tables carry: the
//! same Hipparcos row agrees to the construction's rounding, a Gaia DR3 row
//! differs from the engine's Hipparcos one by the catalogues' proper
//! motions times the years from J2000.0, and the galactic centre by the
//! proper-motion convention (`docs/03-design/astro-star-table.md`, §8).

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
use teistro_astro::ayanamsha::{Definition, EpochScale, definition, mean_deg};
use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::precession::PrecessionModel;
use teistro_core::angle::difference_deg;
use teistro_core::catalogue::{Ayanamsha, Star};
use teistro_core::quantity::{JulianDay, Tt};

/// A definition stated in TT agrees with the engine to the rounding of the
/// construction: a hundredth of a milliarcsecond.
const TT_EPOCH_BOUND_ARCSEC: f64 = 1e-5;

/// A definition stated in Universal Time takes the SDK's Delta T at its
/// epoch, which differs from the engine's by a few seconds in antiquity:
/// measured 2.1e-4″ of precession at worst (the Surya Siddhanta epoch of
/// 499 CE and the Sassanian one), bounded at half a milliarcsecond.
const UT_EPOCH_BOUND_ARCSEC: f64 = 5e-4;

/// The bound for an anchored member at a Julian epoch, arcseconds, from
/// the measured differences (`docs/03-design/astro-star-table.md`, §8):
/// the anchors both tables take from Hipparcos (Spica, λ Scorpii) and the
/// true galactic pole agree to 0.003″; the galactic centre differs by the
/// engine's FK5 record alone, 0.025″ over ±1300 years; the two Gaia DR3
/// rows (ζ Piscium A, δ Cancri) differ from the engine's Hipparcos rows
/// by about 1.1 mas/yr, 1.5″ at 700 CE.
///
/// Two of these were wider until the engine's star table was corrected
/// (`docs/05-testing/02-engine-findings.md`, F5, closed): the galactic
/// centre carried a drift term for an east proper motion 0.4 mas/yr short
/// of Reid and Brunthaler's (0.68″ at 700 CE, now 0.05″ flat), and the
/// IAU 1958 pole carried 0.3″ for a built-in row that was the B1950
/// definition (now 0.0002″, inside the general bound).
fn anchored_bound_arcsec(anchor: Star, jd_tt: f64) -> f64 {
    let years = ((jd_tt - 2_451_545.0) / 365.25).abs();
    match anchor {
        Star::SgrAStar => 0.05,
        Star::Revati | Star::AsellusAustralis => 0.05 + 1.5e-3 * years,
        _ => 5e-3,
    }
}

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
    // The worst difference, the bound and the count by kind of definition,
    // for the accuracy document.
    let mut kinds: BTreeMap<&str, (f64, f64, usize)> = BTreeMap::new();
    let mut compared = 0;
    for row in table["rows"].as_array().unwrap() {
        let key = row["ayanamsha"].as_str().unwrap();
        let id = Ayanamsha::from_key(key).expect("a catalogued key");
        let jd = row["jd_tt"].as_f64().unwrap();
        let (kind, bound) = match definition(id) {
            Definition::Epoch(epoch) | Definition::Frame(epoch) => match epoch.scale {
                EpochScale::Tt => ("epoch-defined members stated in TT", TT_EPOCH_BOUND_ARCSEC),
                EpochScale::Ut => (
                    "epoch-defined members stated in Universal Time",
                    UT_EPOCH_BOUND_ARCSEC,
                ),
            },
            Definition::Object { anchor, .. } => (
                match anchor {
                    Star::Spica | Star::Shaula | Star::GalacticPole => {
                        "anchored members, the same Hipparcos rows and pole"
                    }
                    _ => "anchored members, the two tables' astrometry apart",
                },
                anchored_bound_arcsec(anchor, jd),
            ),
            Definition::Unsourced => continue,
        };
        let theirs = row["mean_deg"].as_f64().unwrap();
        let ours = mean_deg(
            &id.into(),
            JulianDay::<Tt>::literal(jd),
            PrecessionModel::Vondrak2011,
            DeltaTModel::TableThenModel,
        )
        .expect("a defined ayanamsha");
        let arcsec = difference_deg(ours, theirs) * 3600.0;
        let entry = worst.entry(key).or_insert((0.0, jd));
        if arcsec.abs() > entry.0.abs() {
            *entry = (arcsec, jd);
        }
        let tally = kinds.entry(kind).or_insert((0.0, 0.0, 0));
        tally.0 = tally.0.max(arcsec.abs());
        tally.1 = tally.1.max(bound);
        tally.2 += 1;
        assert!(
            arcsec.abs() <= bound,
            "{key} at JD {jd}: SDK {ours} against Teimeris {theirs}: {arcsec:+.6}\" (bound {bound}\")"
        );
        compared += 1;
    }
    assert!(compared >= 42 * 20, "{compared} rows compared");
    for (key, (arcsec, jd)) in &worst {
        println!("{key:<22} worst {arcsec:+.7}\" at JD {jd}");
    }
    for (kind, (value, bound, count)) in &kinds {
        common::record("ayanamsha", kind, *value, "″", *bound, *count);
    }
}
