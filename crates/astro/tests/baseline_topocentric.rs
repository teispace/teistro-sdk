//! The completion's topocentric step against the baseline engine's own
//! before and after.
//!
//! The conformance corpus records six charts **twice** — once from the
//! centre of the Earth and once from the place they were cast for, with
//! every other setting equal — so the step has a recorded input and a
//! recorded answer rather than only an answer. This feeds the recorded
//! geocentric cells to [`Station::seen`] and compares what comes out with
//! the recorded topocentric ones. The derivation and every reading it
//! falsified are in `docs/03-design/topocentric-measured.md`; this is the
//! shipped code held to the same rows.
//!
//! Rank 2 (the baseline over the Swiss Ephemeris) with a bound.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    reason = "tests fail by panicking, read fixtures and print the measurement under --nocapture"
)]

mod common;

use std::path::{Path, PathBuf};

use serde_json::Value;
use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::scale::tt_of;
use teistro_astro::sky;
use teistro_astro::topocentric::Station;
use teistro_core::angle::difference_deg;
use teistro_core::quantity::{JulianDay, Place, Ut1};
use teistro_port_ephemeris::{Body, Cell, CellStatus, Corrections, Source};

/// Every body but the Moon comes back inside this: measured 0.0004″.
const BOUND_ARCSEC: f64 = 1e-3;

/// The Moon keeps this much, which is three parts in a hundred thousand
/// of a step of forty arcminutes; the measured page's §7 says what the
/// candidates are and why this corpus cannot tell them apart.
const MOON_BOUND_ARCSEC: f64 = 0.1;

/// The longitude speed, degrees a day. The corpus records a longitude
/// speed and neither a latitude nor a distance speed, and both enter the
/// answer, so the two that are missing are fed as nought and what is
/// left is their size rather than the step's: measured 0.012°/day on the
/// Moon, whose speed the step changes by 5.5°/day.
const SPEED_BOUND_DEG: f64 = 0.02;

/// The keys the corpus records, and the body each is.
const BODIES: [(&str, Body); 9] = [
    ("SUN", Body::Sun),
    ("MOON", Body::Moon),
    ("MERCURY", Body::Mercury),
    ("VENUS", Body::Venus),
    ("MARS", Body::Mars),
    ("JUPITER", Body::Jupiter),
    ("SATURN", Body::Saturn),
    ("RAHU", Body::MeanNode),
    ("KETU", Body::MeanNode),
];

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/baseline")
}

fn read(path: &Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(path).expect("a fixture")).expect("valid JSON")
}

/// The charts the corpus records from both centres, as (name, from the
/// centre of the Earth, from the place).
fn pairs() -> Vec<(String, Value, Value)> {
    let base = fixtures();
    let mut names: Vec<String> = std::fs::read_dir(base.join("variants"))
        .expect("the variants directory")
        .map(|entry| entry.expect("an entry").file_name())
        .filter_map(|name| name.to_str().map(str::to_string))
        .filter_map(|name| name.strip_suffix("--geocentric.json").map(str::to_string))
        .collect();
    names.sort();
    assert!(
        !names.is_empty(),
        "the corpus records a chart from both centres"
    );
    names
        .into_iter()
        .map(|name| {
            let topocentric = read(&base.join("charts").join(format!("{name}.json")));
            let geocentric = read(
                &base
                    .join("variants")
                    .join(format!("{name}--geocentric.json")),
            );
            (name, geocentric, topocentric)
        })
        .collect()
}

/// One recorded body as a cell in the tropical ecliptic of date, with the
/// two rates the corpus does not record left at nought.
fn cell_of(body: &Value) -> Option<Cell> {
    Some(Cell {
        lon: body["tropical_longitude_deg"].as_f64()?,
        lat: body["latitude_deg"].as_f64()?,
        dist: body["distance_au"].as_f64()?,
        lon_speed: body["speed_deg_per_day"].as_f64()?,
        lat_speed: 0.0,
        dist_speed: 0.0,
        status: CellStatus::Ok,
        source: Source::UNKNOWN,
    })
}

fn place_of(fixture: &Value) -> Place {
    let place = &fixture["input"]["place"];
    Place::try_from_degrees(
        place["latitude"].as_f64().expect("a latitude"),
        place["longitude"].as_f64().expect("a longitude"),
        place["altitude_m"].as_f64().unwrap_or(0.0),
    )
    .expect("a place in range")
}

#[test]
fn the_step_reproduces_the_baseline_at_the_same_instant_and_place() {
    let pairs = pairs();
    let mut worst = 0.0_f64;
    let mut worst_moon = 0.0_f64;
    let mut worst_speed = 0.0_f64;
    let mut compared = 0_usize;
    for (name, geocentric, topocentric) in &pairs {
        // The two must differ in the centre and nothing else, or what is
        // measured here is not the centre's doing.
        assert_eq!(geocentric["settings"]["topocentric"], Value::Bool(false));
        assert_eq!(topocentric["settings"]["topocentric"], Value::Bool(true));
        assert_eq!(
            geocentric["settings"]["node"], topocentric["settings"]["node"],
            "{name}: the pair must agree on the node"
        );
        let place = place_of(topocentric);
        let ut1 = JulianDay::<Ut1>::try_new(
            topocentric["input"]["resolved"]["jd_ut"]
                .as_f64()
                .expect("an instant"),
        )
        .expect("an instant in range");
        let (tt, _) = tt_of(ut1, DeltaTModel::TableThenModel).expect("Delta T");
        let station = Station::at(place, ut1, tt).in_ecliptic(sky::obliquity(tt).true_deg);
        for (key, body) in BODIES {
            let (Some(before), Some(after)) = (
                cell_of(&geocentric["positions"]["bodies"][key]),
                cell_of(&topocentric["positions"]["bodies"][key]),
            ) else {
                continue;
            };
            let seen = station.seen(before, body, Corrections::APPARENT, true);
            if !body.is_placed() {
                // A direction takes no part of the step, and the corpus
                // records it under both centres to the last bit.
                assert_eq!(seen, before, "{name}/{key}: a direction was displaced");
                assert_eq!(
                    after.lon.to_bits(),
                    before.lon.to_bits(),
                    "{name}/{key}: the corpus moved a direction"
                );
                compared += 1;
                continue;
            }
            let off = difference_deg(seen.lon, after.lon).abs() * 3600.0;
            let off_lat = (seen.lat - after.lat).abs() * 3600.0;
            let apart = off.max(off_lat);
            if body == Body::Moon {
                worst_moon = worst_moon.max(apart);
            } else {
                worst = worst.max(apart);
            }
            worst_speed = worst_speed.max((seen.lon_speed - after.lon_speed).abs());
            // The distance is recorded too, and the step changes it by
            // about an Earth radius.
            assert!(
                (seen.dist - after.dist).abs() < 1e-8,
                "{name}/{key}: distance {} against {}",
                seen.dist,
                after.dist
            );
            compared += 1;
        }
    }
    println!(
        "topocentric: {worst:.6}″ over every body but the Moon, {worst_moon:.6}″ on the Moon, \
         {worst_speed:.6}°/day of speed, over {compared} comparisons in {} pairs",
        pairs.len()
    );
    common::record(
        "topocentric",
        "the recorded geocentric charts recentred, every body but the Moon",
        worst,
        "″",
        BOUND_ARCSEC,
        compared,
    );
    common::record(
        "topocentric",
        "the same, the Moon",
        worst_moon,
        "″",
        MOON_BOUND_ARCSEC,
        pairs.len(),
    );
    assert!(
        worst < BOUND_ARCSEC,
        "{worst}″ over the bound {BOUND_ARCSEC}″"
    );
    assert!(
        worst_moon < MOON_BOUND_ARCSEC,
        "{worst_moon}″ over the bound {MOON_BOUND_ARCSEC}″"
    );
    assert!(
        worst_speed < SPEED_BOUND_DEG,
        "{worst_speed}°/day over the bound {SPEED_BOUND_DEG}°/day"
    );
    // The guard the chart round-trip test earned: a sample that proves
    // nothing is a sample that passes.
    assert!(
        worst_moon > 0.0 && compared > 40,
        "the pairs must actually exercise the step"
    );
}
