//! The star table against Teimeris's recorded places
//! (`fixtures/teimeris/stars.json`, written by the adapter's `stars-table`
//! binary): for every catalogued star the engine knows by name, its place
//! at four instants under three sets of corrections, and the engine's own
//! catalogue record. Two comparisons, kept apart: the SDK's pipeline over
//! the ENGINE's record must reproduce the engine's places (the same
//! astrometry, so any difference is the computation); the SDK's own
//! astrometry against the engine's record is a difference of catalogues
//! (Gaia DR3 against Hipparcos for some rows) and is reported, not held.
//! Rank 2 reference values with a tolerance (`CLEAN_ROOM.md`); the bounds
//! are the measured agreement, published in
//! `docs/03-design/astro-star-table.md`.

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
use teistro_astro::iau::{DAU, DAYSEC, DJC, RAD2DEG, nut00b};
use teistro_astro::precession::{PrecessionModel, mean_obliquity_rad};
use teistro_astro::stars::{Astrometry, Corrections, Options, place};
use teistro_core::angle::difference_deg;
use teistro_core::catalogue::Star;
use teistro_core::quantity::{JulianDay, Tt};

/// The pipeline over the same astrometry: the construction's rounding and
/// the nutation models (the engine's against IAU 2000B) leave under a
/// milliarcsecond.
const PIPELINE_BOUND_ARCSEC: f64 = 2e-3;

/// The catalogues against each other: a Hipparcos row against the same
/// Hipparcos row agrees to the milliarcsecond; a Gaia row against the
/// engine's older row differs by up to fifty milliarcseconds a year in
/// proper motion (Errai, 5.0″ at 2100) or by a couple of arcseconds in the
/// position itself (Heze, Alnasl, Aljanah); anything more is a wrong unit
/// or a wrong star.
const DATA_BOUND_ARCSEC: f64 = 10.0;

/// Two rows further apart than this at J2000.0 name different stars (a
/// traditional name the catalogues give to different stars, as Sadalbari
/// to λ and μ Pegasi): reported and left out of the data comparison.
const DIFFERENT_STAR_ARCSEC: f64 = 30.0;

/// Two rows whose proper motions differ by more than this, milliarcseconds
/// a year, are different catalogue rows for the same star (the engine's
/// Rigil Kentaurus is 200 mas/yr from Hipparcos's): reported and left out,
/// since no bound on the places would be a measurement of the SDK.
const DIFFERENT_ROW_MAS_YR: f64 = 50.0;

/// The engine's true-position flag drops the aberration and the deflection
/// but keeps the annual parallax; the SDK's `GEOMETRIC` drops all three,
/// so the engine's geometric rows are compared with the parallax kept.
const ENGINE_GEOMETRIC: Options = Options {
    corrections: Corrections {
        nutation: false,
        aberration: false,
        deflection: false,
        parallax: true,
    },
    ..Options::APPARENT
};

/// The engine's record, in the SDK's units: arcseconds a century to
/// milliarcseconds a year, astronomical units a century to km/s.
fn engine_astrometry(record: &Value) -> Astrometry {
    let au_per_century_to_km_s = DAU / 1e3 / (DJC * DAYSEC);
    Astrometry::icrs(
        record["ra_deg"].as_f64().unwrap(),
        record["dec_deg"].as_f64().unwrap(),
    )
    .with_proper_motion(
        record["pm_ra_arcsec_century"].as_f64().unwrap() * 10.0,
        record["pm_dec_arcsec_century"].as_f64().unwrap() * 10.0,
    )
    .with_parallax(record["parallax_arcsec"].as_f64().unwrap() * 1e3)
    .with_radial_velocity(
        record["radial_velocity_au_century"].as_f64().unwrap() * au_per_century_to_km_s,
    )
}

/// The great-circle separation of two places, arcseconds.
fn separation_arcsec(lon_a: f64, lat_a: f64, lon_b: f64, lat_b: f64) -> f64 {
    let dlon = difference_deg(lon_a, lon_b) * lat_b.to_radians().cos();
    let dlat = lat_a - lat_b;
    (dlon * dlon + dlat * dlat).sqrt() * 3600.0
}

fn fixture() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/teimeris/stars.json");
    serde_json::from_str(&std::fs::read_to_string(path).expect("the recorded table"))
        .expect("valid JSON")
}

#[test]
fn the_pipeline_reproduces_the_engines_places_on_its_own_astrometry() {
    let table = fixture();
    assert_eq!(table["scale"], "TT");
    let mut worst = (0.0f64, String::new());
    let mut compared = 0;
    for row in table["rows"].as_array().unwrap() {
        let key = row["star"].as_str().unwrap();
        assert!(Star::from_key(key).is_some(), "{key}");
        let record = &row["record"];
        // A record stated in FK5 or FK4 goes through the engine's frame
        // conversions, which the SDK does not model; the table is ICRS.
        if record["epoch"].as_f64().unwrap() != 0.0 {
            continue;
        }
        let astrometry = engine_astrometry(record);
        for entry in row["places"].as_array().unwrap() {
            let tt = JulianDay::<Tt>::literal(entry["jd_tt"].as_f64().unwrap());
            let cases = [
                (
                    "apparent",
                    Options::APPARENT,
                    "apparent_lon_deg",
                    "apparent_lat_deg",
                ),
                ("mean", Options::MEAN, "mean_lon_deg", "mean_lat_deg"),
                (
                    "geometric",
                    ENGINE_GEOMETRIC,
                    "geometric_lon_deg",
                    "geometric_lat_deg",
                ),
            ];
            for (name, options, lon_key, lat_key) in cases {
                let ours = place(&astrometry, tt, &options).unwrap();
                let apart = separation_arcsec(
                    ours.lon_deg,
                    ours.lat_deg,
                    entry[lon_key].as_f64().unwrap(),
                    entry[lat_key].as_f64().unwrap(),
                );
                if apart > worst.0 {
                    worst = (apart, format!("{key} {name} at JD {}", tt.get()));
                }
                assert!(
                    apart <= PIPELINE_BOUND_ARCSEC,
                    "{key} {name} at JD {}: {apart:.6}\" apart (bound {PIPELINE_BOUND_ARCSEC}\")",
                    tt.get()
                );
                compared += 1;
            }
            // The equatorial reading of the apparent place.
            let ours = place(&astrometry, tt, &Options::APPARENT).unwrap();
            let apart = separation_arcsec(
                ours.ra_deg,
                ours.dec_deg,
                entry["apparent_ra_deg"].as_f64().unwrap(),
                entry["apparent_dec_deg"].as_f64().unwrap(),
            );
            assert!(
                apart <= PIPELINE_BOUND_ARCSEC,
                "{key} equatorial at JD {}: {apart:.6}\"",
                tt.get()
            );
        }
    }
    assert!(compared >= 100 * 12, "{compared} places compared");
    println!(
        "{compared} places compared; worst {:.6}\" at {}",
        worst.0, worst.1
    );
    common::record(
        "star_table",
        "places over the engine's own astrometry",
        worst.0,
        "″",
        PIPELINE_BOUND_ARCSEC,
        compared,
    );
}

#[test]
fn the_catalogues_astrometry_stays_within_the_catalogues_of_the_engines() {
    let table = fixture();
    let mut worst = (0.0f64, String::new());
    let mut ranked: Vec<(f64, String)> = Vec::new();
    let mut compared = 0;
    let mut left_out = 0;
    for row in table["rows"].as_array().unwrap() {
        let key = row["star"].as_str().unwrap();
        let star = Star::from_key(key).expect("a catalogued key");
        let record = &row["record"];
        if record["epoch"].as_f64().unwrap() != 0.0 {
            continue;
        }
        let ours = Astrometry::of(star);
        let theirs = engine_astrometry(record);
        let at_epoch = separation_arcsec(ours.ra_deg, ours.dec_deg, theirs.ra_deg, theirs.dec_deg);
        if at_epoch > DIFFERENT_STAR_ARCSEC {
            left_out += 1;
            println!(
                "{key}: the engine's row named {} is {} ({at_epoch:.0}\" away), not this star",
                record["name"].as_str().unwrap_or_default(),
                record["designation"].as_str().unwrap_or_default()
            );
            continue;
        }
        let pm_apart = ((ours.pm_ra_mas_yr - theirs.pm_ra_mas_yr).powi(2)
            + (ours.pm_dec_mas_yr - theirs.pm_dec_mas_yr).powi(2))
        .sqrt();
        if pm_apart > DIFFERENT_ROW_MAS_YR {
            left_out += 1;
            println!(
                "{key}: the engine's row moves {pm_apart:.0} mas/yr differently ({}, {} against {}, {}): another catalogue's row",
                theirs.pm_ra_mas_yr, theirs.pm_dec_mas_yr, ours.pm_ra_mas_yr, ours.pm_dec_mas_yr
            );
            continue;
        }
        // The two rows carried to the same instants without any correction
        // that would hide the data: the places part by the catalogues'
        // disagreement in position and in proper motion times the years.
        for entry in row["places"].as_array().unwrap() {
            let tt = JulianDay::<Tt>::literal(entry["jd_tt"].as_f64().unwrap());
            let options = Options {
                corrections: Corrections::GEOMETRIC,
                ..Options::APPARENT
            };
            let a = place(&ours, tt, &options).unwrap();
            let b = place(&theirs, tt, &options).unwrap();
            let apart = separation_arcsec(a.lon_deg, a.lat_deg, b.lon_deg, b.lat_deg);
            if apart > worst.0 {
                worst = (apart, format!("{key} at JD {}", tt.get()));
            }
            ranked.push((apart, format!("{key} at JD {}", tt.get())));
            compared += 1;
        }
    }
    ranked.sort_by(|a, b| b.0.total_cmp(&a.0));
    for (apart, where_) in ranked.iter().take(8) {
        println!("  {apart:.4}\" {where_}");
    }
    assert!(compared >= 100 * 4, "{compared} places compared");
    println!(
        "{compared} places compared, {left_out} row(s) left out; the catalogues differ most at {}: {:.4}\"",
        worst.1, worst.0
    );
    // Five rows were left out until the engine's table was corrected
    // (`docs/05-testing/02-engine-findings.md`, F5, closed): two named
    // another star, one carried another catalogue's proper motion, one had
    // a rate with the declination factor applied twice, and one was a
    // different definition of the galactic pole. Every row now compares.
    assert_eq!(left_out, 0, "{left_out} row(s) left out of the comparison");
    assert!(
        worst.0 <= DATA_BOUND_ARCSEC,
        "{}: the SDK's and the engine's astrometry place it {:.4}\" apart (bound {DATA_BOUND_ARCSEC}\")",
        worst.1,
        worst.0
    );
    common::record(
        "star_table",
        "the two catalogues' astrometry apart",
        worst.0,
        "″",
        DATA_BOUND_ARCSEC,
        compared,
    );
}

/// The frame the engine reads its ecliptic coordinates in against the
/// SDK's: the mean obliquity of the long-term model at each instant and the
/// IAU 2000B nutation against the engine's.
#[test]
fn the_obliquity_and_nutation_agree_with_the_engines() {
    let table = fixture();
    let mut compared = 0;
    let mut worst_obliquity = 0.0f64;
    let mut worst_nutation = 0.0f64;
    for frame in table["frames"].as_array().unwrap() {
        let tt = JulianDay::<Tt>::literal(frame["jd_tt"].as_f64().unwrap());
        let (date1, date2) = tt.split();
        let ours = mean_obliquity_rad(PrecessionModel::Vondrak2011, tt) * RAD2DEG;
        let theirs = frame["mean_obliquity_deg"].as_f64().unwrap();
        let apart = (ours - theirs) * 3600.0;
        println!("JD {}: mean obliquity {apart:+.6}\" apart", tt.get());
        assert!(
            apart.abs() < 1e-3,
            "mean obliquity at JD {}: {apart:+.6}\"",
            tt.get()
        );
        worst_obliquity = worst_obliquity.max(apart.abs());
        let nutation = nut00b(date1, date2);
        let dpsi = (nutation.dpsi * RAD2DEG - frame["nutation_lon_deg"].as_f64().unwrap()) * 3600.0;
        let deps = (nutation.deps * RAD2DEG - frame["nutation_obl_deg"].as_f64().unwrap()) * 3600.0;
        println!("JD {}: nutation {dpsi:+.6}\" {deps:+.6}\" apart", tt.get());
        // IAU 2000B against the engine's model: a milliarcsecond.
        assert!(
            dpsi.abs() < 2e-3 && deps.abs() < 2e-3,
            "nutation at JD {}: {dpsi} {deps}",
            tt.get()
        );
        worst_nutation = worst_nutation.max(dpsi.abs()).max(deps.abs());
        compared += 1;
    }
    assert!(compared >= 4);
    common::record(
        "precession",
        "the Vondrák mean obliquity against the engine's",
        worst_obliquity,
        "″",
        1e-3,
        compared,
    );
    common::record(
        "nutation",
        "IAU 2000B against the engine's nutation",
        worst_nutation,
        "″",
        2e-3,
        compared,
    );
}
