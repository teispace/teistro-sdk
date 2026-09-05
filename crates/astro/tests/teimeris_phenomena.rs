//! The planetary phenomena against Teimeris's recorded values
//! (`fixtures/teimeris/pheno.json`, written by the adapter's `pheno-table`
//! binary): for eleven bodies at sixteen instants the engine's phase angle,
//! illuminated fraction, elongation, apparent diameter, magnitude and
//! horizontal parallax, with the geometry it read them from, so the SDK's
//! arithmetic is held to the engine's on the same positions; and the
//! engine's equation of time with the Sun's apparent right ascension, so
//! the SDK's construction is held with its own sidereal time. Rank 2
//! reference values with a tolerance (`CLEAN_ROOM.md`); the bounds are the
//! measured agreement, published in
//! `docs/03-design/astro-planetary-phenomena.md`.

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
use teistro_astro::DeltaTModel;
use teistro_astro::phenomena::{EclipticPosition, Geometry, HeliocentricLeg, Phenomena};
use teistro_astro::sky::{Apparent, ApparentPositions, equation_of_time_seconds};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Tt, Ut1};
use teistro_port_ephemeris::Body;

/// The same geometry, the same arithmetic: angles to the rounding of the
/// construction.
const ANGLE_BOUND_DEG: f64 = 1e-9;

/// The magnitude models are the same formulae; the Sun's differs by the
/// disc each side takes (the IAU 2015 nominal radius against the older
/// 696 000 km), 0.001 magnitude.
const MAGNITUDE_BOUND: f64 = 2e-3;

/// The apparent diameter: the same radii for every body but the Sun, whose
/// 0.04 % of radius (695 700 km against 696 000) is 0.84″ of disc.
const DIAMETER_BOUND_ARCSEC: f64 = 1.0;

/// The horizontal parallax. The engine reports it for the Moon alone and
/// zero for the rest, where the SDK gives every body its own, so the
/// Moon's rows are compared; and the engine reads it from a distance up to
/// 40 km from the one its disc uses, where the SDK reads both from the
/// apparent distance: 0.35″ of the Moon's 3400″.
const PARALLAX_BOUND_ARCSEC: f64 = 0.5;

/// The equation of time from the same right ascension with the SDK's
/// sidereal time, through 2030.
const EQUATION_BOUND_SECONDS: f64 = 1e-3;

/// From 2050 the engine's sidereal time steps by 1.9″ where its long-term
/// branch takes over (`docs/05-testing/02-engine-findings.md`, F1), so its
/// equation of time is 0.127 s from the one its own Sun implies; the SDK's
/// sidereal time is continuous. Held loosely there.
const LATER_EQUATION_BOUND_SECONDS: f64 = 0.2;
const LAST_CONSISTENT_JD: f64 = 2_462_502.5;

fn fixture() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/teimeris/pheno.json");
    serde_json::from_str(&std::fs::read_to_string(path).expect("the recorded table"))
        .expect("valid JSON")
}

fn position(value: &Value) -> EclipticPosition {
    EclipticPosition::new(
        value["lon_deg"].as_f64().unwrap(),
        value["lat_deg"].as_f64().unwrap(),
        value["dist_au"].as_f64().unwrap(),
    )
}

#[test]
fn the_arithmetic_reproduces_the_engines_phenomena_on_its_own_geometry() {
    let table = fixture();
    assert_eq!(table["scale"], "TT");
    let mut compared = 0;
    let mut worst_magnitude = (0.0f64, String::new());
    let mut worst_angle = 0.0f64;
    for row in table["rows"].as_array().unwrap() {
        let body = Body::from_key(row["body"].as_str().unwrap()).expect("a catalogued body");
        let tt = JulianDay::<Tt>::literal(row["jd_tt"].as_f64().unwrap());
        let geometry = Geometry {
            body: position(&row["body_position"]),
            sun: position(&row["sun_position"]),
            body_from_sun: row["body_from_sun"]
                .as_object()
                .map(|_| position(&row["body_from_sun"])),
        };
        let ours = Phenomena::from_geometry(body, &geometry, tt).unwrap();
        let where_ = format!("{} at JD {}", body.key(), tt.get());
        if geometry.body_from_sun.is_some() {
            assert_eq!(ours.heliocentric_leg, HeliocentricLeg::Provider);
        }
        let phase_angle = ours.phase.map_or(0.0, |p| p.angle_deg);
        let fraction = ours.phase.map_or(0.0, |p| p.illuminated_fraction);
        worst_angle = worst_angle
            .max((phase_angle - row["phase_angle_deg"].as_f64().unwrap()).abs())
            .max((ours.elongation_deg - row["elongation_deg"].as_f64().unwrap()).abs());
        assert!(
            (phase_angle - row["phase_angle_deg"].as_f64().unwrap()).abs() < ANGLE_BOUND_DEG,
            "{where_}: phase angle {phase_angle} against {}",
            row["phase_angle_deg"]
        );
        assert!(
            (fraction - row["illuminated_fraction"].as_f64().unwrap()).abs() < 1e-12,
            "{where_}: fraction {fraction} against {}",
            row["illuminated_fraction"]
        );
        assert!(
            (ours.elongation_deg - row["elongation_deg"].as_f64().unwrap()).abs() < ANGLE_BOUND_DEG,
            "{where_}: elongation {} against {}",
            ours.elongation_deg,
            row["elongation_deg"]
        );
        let diameter_apart =
            (ours.apparent_diameter_deg() - row["diameter_deg"].as_f64().unwrap()).abs() * 3600.0;
        assert!(
            diameter_apart < DIAMETER_BOUND_ARCSEC,
            "{where_}: diameter {diameter_apart}\" apart"
        );
        if body == Body::Moon {
            let parallax_apart =
                (ours.disc.parallax_deg - row["horizontal_parallax_deg"].as_f64().unwrap()).abs()
                    * 3600.0;
            assert!(
                parallax_apart < PARALLAX_BOUND_ARCSEC,
                "{where_}: parallax {parallax_apart}\" apart"
            );
        }
        match (ours.magnitude, row["magnitude"].as_f64()) {
            (Some(a), Some(b)) => {
                let apart = (a - b).abs();
                if apart > worst_magnitude.0 {
                    worst_magnitude = (apart, where_.clone());
                }
                assert!(
                    apart < MAGNITUDE_BOUND,
                    "{where_}: magnitude {a} against {b}"
                );
            }
            (None, None) => {}
            // The engine writes zero for a body without a magnitude.
            (None, Some(0.0)) if !body.has_distance() => {}
            (a, b) => panic!("{where_}: magnitude {a:?} against {b:?}"),
        }
        compared += 1;
    }
    assert!(compared >= 11 * 16, "{compared} rows compared");
    println!(
        "{compared} rows compared; magnitudes worst {:.6} at {}",
        worst_magnitude.0, worst_magnitude.1
    );
    common::record(
        "phenomena",
        "phase angles and elongations over the engine's geometry",
        worst_angle,
        "°",
        ANGLE_BOUND_DEG,
        compared,
    );
    common::record(
        "phenomena",
        "magnitudes",
        worst_magnitude.0,
        " mag",
        MAGNITUDE_BOUND,
        compared,
    );
}

/// The Sun as the engine placed it, for the equation of time.
struct RecordedSun {
    ra_deg: f64,
    dec_deg: f64,
}

impl ApparentPositions for RecordedSun {
    fn apparent(&self, body: Body, _ut1: JulianDay<Ut1>) -> Result<Apparent, Error> {
        assert_eq!(body, Body::Sun);
        Ok(Apparent {
            ra_deg: self.ra_deg,
            dec_deg: self.dec_deg,
            distance_au: 1.0,
        })
    }

    fn describe(&self) -> String {
        "the recorded Sun".to_owned()
    }
}

#[test]
fn the_equation_of_time_reproduces_the_engines_from_the_same_sun() {
    let table = fixture();
    let mut worst = 0.0f64;
    let mut compared = 0;
    for row in table["solar"].as_array().unwrap() {
        let ut1 = JulianDay::<Ut1>::literal(row["jd_ut1"].as_f64().unwrap());
        let sun = RecordedSun {
            ra_deg: row["sun_ra_deg"].as_f64().unwrap(),
            dec_deg: row["sun_dec_deg"].as_f64().unwrap(),
        };
        let ours = equation_of_time_seconds(&sun, ut1, DeltaTModel::TableThenModel).unwrap();
        let theirs = row["equation_of_time_seconds"].as_f64().unwrap();
        let bound = if ut1.get() <= LAST_CONSISTENT_JD {
            worst = worst.max((ours - theirs).abs());
            EQUATION_BOUND_SECONDS
        } else {
            LATER_EQUATION_BOUND_SECONDS
        };
        assert!(
            (ours - theirs).abs() < bound,
            "JD {}: {ours} s against {theirs} s (bound {bound} s)",
            ut1.get()
        );
        compared += 1;
    }
    assert!(compared >= 16);
    println!("{compared} instants; equation of time worst {worst:.6} s through 2030");
    common::record(
        "equation_of_time",
        "from the engine's Sun with the SDK's sidereal time, through 2030",
        worst,
        " s",
        EQUATION_BOUND_SECONDS,
        compared,
    );
}
