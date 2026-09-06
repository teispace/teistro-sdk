//! The sidereal time against Teimeris's recorded values
//! (`fixtures/teimeris/sidereal.json`, written by the adapter's
//! `sidereal-table` binary): the engine's Greenwich apparent sidereal time
//! under its default model (the IERS 2010 expression strictly inside its
//! 1850 to 2050 window, a long-term construction at and beyond the bounds)
//! and under its IERS 2010 model at every instant, with its Delta T,
//! obliquity and nutation, so the SDK's IAU 2006 expression (`gst06b`) is
//! held to the engine's on the same TT. Rank 2 reference values with a
//! tolerance (`CLEAN_ROOM.md`); the bounds are the measured agreement,
//! published in `docs/03-design/astro-events-and-crossings.md`.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    reason = "tests fail by panicking, read a recorded table and print the measurement under --nocapture"
)]

mod common;

use std::path::Path;

use serde_json::Value;
use teistro_astro::sky::{greenwich_sidereal_time_deg, obliquity};
use teistro_core::quantity::{JulianDay, Tt, Ut1};

/// Inside the engine's window both sides compute the IERS 2010 expression
/// on the same TT; what remains is the two readings of the IAU 2000B
/// nutation (the fundamental arguments, C43; the fixed offsets, F6): 1.3
/// mas at 1850, under 0.4 mas from 1875.
const INSIDE_BOUND_ARCSEC: f64 = 0.002;

/// The engine's IERS 2010 model at every instant, 1700 to 2300: the two
/// 2000B readings drift apart with the fundamental arguments, 6 mas at
/// 1700.
const EXPRESSION_BOUND_ARCSEC: f64 = 0.01;

/// The nutation in longitude and in obliquity, the same series read the
/// two ways.
const NUTATION_BOUND_ARCSEC: f64 = 0.01;

/// Where the engine's long-term branch takes over from the IERS 2010
/// expression, the two must meet: the branch's joining offsets are the
/// engine's own, and they stepped by −1.909″ at 2050 until they were
/// fixed (F1). Measured at 0.001″ either side.
const JOIN_BOUND_ARCSEC: f64 = 0.01;

/// The bounds of the engine's window, where the branches join.
const WINDOW: (f64, f64) = (2_396_758.5, 2_469_807.5);

fn fixture() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/teimeris/sidereal.json");
    serde_json::from_str(&std::fs::read_to_string(path).expect("the recorded table"))
        .expect("valid JSON")
}

/// The signed difference of two angles, arcseconds, folded to a half turn.
fn arcsec_apart(ours_deg: f64, theirs_deg: f64) -> f64 {
    ((ours_deg - theirs_deg + 540.0).rem_euclid(360.0) - 180.0) * 3600.0
}

#[test]
fn the_sidereal_time_reproduces_the_engines_expression_on_the_same_tt() {
    let table = fixture();
    let mut worst_inside = 0.0f64;
    let mut inside = 0;
    let mut worst_expression = 0.0f64;
    let mut worst_nutation = 0.0f64;
    let mut worst_join = 0.0f64;
    let mut joins = 0;
    let mut compared = 0;
    for row in table["rows"].as_array().unwrap() {
        let jd = row["jd_ut1"].as_f64().unwrap();
        let ut1 = JulianDay::<Ut1>::literal(jd);
        let tt = JulianDay::<Tt>::literal(jd + row["delta_t_seconds"].as_f64().unwrap() / 86_400.0);
        let ours = greenwich_sidereal_time_deg(ut1, tt);
        let expression = arcsec_apart(ours, row["gast_iers_2010_deg"].as_f64().unwrap()).abs();
        assert!(
            expression < EXPRESSION_BOUND_ARCSEC,
            "JD {jd}: {expression}″ from the engine's IERS 2010 expression"
        );
        worst_expression = worst_expression.max(expression);
        let nutation = obliquity(tt);
        let lon =
            (nutation.nutation_lon_deg - row["nutation_lon_deg"].as_f64().unwrap()).abs() * 3600.0;
        let obl =
            (nutation.nutation_obl_deg - row["nutation_obl_deg"].as_f64().unwrap()).abs() * 3600.0;
        assert!(
            lon < NUTATION_BOUND_ARCSEC && obl < NUTATION_BOUND_ARCSEC,
            "JD {jd}: nutation {lon}″ in longitude, {obl}″ in obliquity from the engine's"
        );
        worst_nutation = worst_nutation.max(lon).max(obl);
        compared += 1;
        let default = arcsec_apart(ours, row["gast_deg"].as_f64().unwrap());
        if row["branch"] == "iers_2010" {
            assert!(
                default.abs() < INSIDE_BOUND_ARCSEC,
                "JD {jd}: {default}″ from the engine inside its window"
            );
            worst_inside = worst_inside.max(default.abs());
            inside += 1;
        } else if (jd - WINDOW.0).abs() < 0.5 || (jd - WINDOW.1).abs() < 0.5 {
            // At the window's bounds the engine's long-term branch must
            // meet the expression: it stepped by −1.909″ at 2050 and
            // +0.098″ at 1850 until the engine's joining offsets were
            // fixed (`docs/05-testing/02-engine-findings.md`, F1, closed),
            // and this is the regression test for that.
            assert!(
                default.abs() < JOIN_BOUND_ARCSEC,
                "JD {jd}: the branch joins {default}″ from the expression"
            );
            worst_join = worst_join.max(default.abs());
            joins += 1;
        } else {
            // Beyond the window the branch is a different model rather
            // than the IERS 2010 expression; reported, not held.
            println!(
                "JD {jd}: the engine's long-term branch {:+.3}″ from the expression",
                -default
            );
        }
    }
    assert!(inside >= 40 && compared >= 48);
    assert_eq!(joins, 2, "the window's two bounds are in the table");
    println!(
        "{compared} instants; inside the window worst {worst_inside:.5}″ over {inside}; at the window's bounds worst {worst_join:.5}″; against the IERS 2010 expression worst {worst_expression:.5}″; nutation worst {worst_nutation:.5}″"
    );
    common::record(
        "sidereal_time",
        "against the engine's apparent sidereal time inside its 1850 to 2050 window",
        worst_inside,
        "″",
        INSIDE_BOUND_ARCSEC,
        inside,
    );
    common::record(
        "sidereal_time",
        "against the engine's IERS 2010 expression, 1700 to 2300",
        worst_expression,
        "″",
        EXPRESSION_BOUND_ARCSEC,
        compared,
    );
    common::record(
        "nutation",
        "against the engine's IAU 2000B nutation, 1700 to 2300",
        worst_nutation,
        "″",
        NUTATION_BOUND_ARCSEC,
        compared,
    );
}
