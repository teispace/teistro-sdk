//! Every recorded derived point of the conformance corpus, through this
//! crate.
//!
//! `cargo xtask points` measured the same rules against the same corpus
//! without using this crate, and wrote what it found to
//! `03-design/points-measured.md`. That pass falsified the design before
//! the code existed; this test holds the code to it afterwards, at the
//! pass's own bounds — **exact** for the six rules that are exact, and
//! inside the published bracket for the three the clock drives.
//!
//! Nothing here needs an ephemeris. The ascendant is the sidereal time
//! and the latitude, so `astro::houses` answers it from the fixture's
//! own instant, place and recorded ayanamsha.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    reason = "tests fail by panicking, index JSON by key and print the measurement under --nocapture"
)]

use std::path::Path;

use serde_json::Value;
use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::houses::{ChartFrame, houses_at};
use teistro_astro::scale::tt_of;
use teistro_core::catalogue::{HouseSystem, Point, Vara};
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Ut1, Utc};
use teistro_core::settings::PolarPolicy;
use teistro_points::eighth::{self, Ascendant};
use teistro_points::{lagna, solar};

/// The bound the falsification pass publishes for the three lagnas the
/// clock drives: the engine's elapsed time is at most 1.633 minutes from
/// the ishtakaal recorded beside them, which at the fastest of the three
/// rates is this many degrees.
const CLOCK_BOUND_DEG: f64 = 1.633 / 60.0 * 75.0;

/// What "exact" means for a rule the pass found exact: the last bits of
/// a double, and nothing else here is that near.
const EXACT_DEG: f64 = 1e-9;

/// The bound §6 of the pass publishes for the two day-division points:
/// the SDK's ascendant against the engine's at the same instant.
const ASCENDANT_BOUND_DEG: f64 = 0.01;

/// One fixture, in the shapes this test needs.
struct Chart {
    name: String,
    sun: f64,
    moon: f64,
    lagna: f64,
    hours: f64,
    is_day: bool,
    vara: Vara,
    arc: Interval,
    place: Place,
    ayanamsha: f64,
    upagrahas: Vec<(String, f64)>,
    lagnas: Vec<(String, f64)>,
}

impl Ascendant for Chart {
    fn ascendant_deg(&self, at: JulianDay<Utc>) -> Result<f64, Error> {
        let ut1 = JulianDay::<Ut1>::literal(at.get());
        let (tt, _) = tt_of(ut1, DeltaTModel::TableThenModel)?;
        let frame = ChartFrame {
            sidereal_offset_deg: self.ayanamsha,
            sun_declination_deg: None,
        };
        let houses = houses_at(
            HouseSystem::WholeSign,
            ut1,
            tt,
            &self.place,
            &frame,
            PolarPolicy::FallbackWholeSign,
        )?;
        Ok((houses.angles.ascendant_deg - self.ayanamsha).rem_euclid(360.0))
    }
}

impl Chart {
    fn recorded(&self, key: &str) -> Option<f64> {
        self.lagnas
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| *value)
    }

    fn upagraha(&self, key: &str) -> Option<f64> {
        self.upagrahas
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| *value)
    }
}

/// The angle between two longitudes, degrees, the shorter way round.
fn apart(first: f64, second: f64) -> f64 {
    let gap = (first - second).abs().rem_euclid(360.0);
    gap.min(360.0 - gap)
}

fn charts() -> Vec<Chart> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/baseline");
    let mut charts = Vec::new();
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
            if let Some(chart) = read(
                path.file_stem()
                    .map(|stem| stem.to_string_lossy().to_string())
                    .unwrap_or_default(),
                &value,
            ) {
                charts.push(chart);
            }
        }
    }
    assert!(!charts.is_empty(), "the corpus carries derived points");
    charts
}

fn read(name: String, fixture: &Value) -> Option<Chart> {
    let foundation = fixture.get("foundation")?;
    let bodies = fixture["positions"]["bodies"].as_object()?;
    let lagnas: Vec<(String, f64)> = fixture["houses"]["special_lagnas"]
        .as_object()?
        .iter()
        .filter_map(|(key, value)| value.as_f64().map(|value| (key.clone(), value)))
        .collect();
    let upagrahas: Vec<(String, f64)> = foundation["upagrahas"]
        .as_array()?
        .iter()
        .filter_map(|entry| {
            Some((
                entry["key"].as_str()?.to_string(),
                entry["sidereal_longitude_deg"].as_f64()?,
            ))
        })
        .collect();
    let day_sunrise = foundation["lagna_sunrise_jd"].as_f64()?;
    let jd = foundation["jd_ut"].as_f64()?;
    // The engine numbers the weekday from Monday; the catalogue from
    // Sunday. The field is the vara of the day the chart belongs to.
    let weekday = (u8::try_from(fixture["panchanga"]["weekday_swe"].as_u64()?).ok()? + 1) % 7;
    let vara = Vara::ALL
        .into_iter()
        .find(|vara| vara.attributes().weekday == weekday)?;
    let is_day = foundation["is_day_birth"].as_bool().unwrap_or(false);
    let place = fixture["input"]["place"].as_object()?;
    Some(Chart {
        name,
        sun: bodies["SUN"]["sidereal_longitude_deg"].as_f64()?,
        moon: bodies["MOON"]["sidereal_longitude_deg"].as_f64()?,
        lagna: foundation["lagna"]["sidereal_longitude_deg"].as_f64()?,
        hours: (jd - day_sunrise) * 24.0,
        is_day,
        vara,
        arc: arc_of(foundation, day_sunrise, jd)?,
        place: Place::new(
            Latitude::try_new(place["latitude"].as_f64()?).ok()?,
            Longitude::try_new(place["longitude"].as_f64()?).ok()?,
            Altitude::try_new(place["altitude_m"].as_f64().unwrap_or(0.0)).ok()?,
        ),
        ayanamsha: foundation["ayanamsha"]["value_deg"].as_f64()?,
        upagrahas,
        lagnas,
    })
}

/// The arc the birth falls in: the daylight of the day it belongs to, or
/// the night that follows that daylight.
fn arc_of(foundation: &Value, day_sunrise: f64, jd: f64) -> Option<Interval> {
    let opened_yesterday =
        (foundation["previous_day"]["sunrise_jd"].as_f64()? - day_sunrise).abs() < f64::EPSILON;
    let (sunset, next_sunrise) = if opened_yesterday {
        (
            foundation["previous_day"]["sunset_jd"].as_f64()?,
            foundation["sunrise"]["sunrise_jd"].as_f64()?,
        )
    } else {
        (
            foundation["sunrise"]["sunset_jd"].as_f64()?,
            foundation["next_day"]["sunrise_jd"].as_f64()?,
        )
    };
    Some(if jd < sunset {
        Interval::literal(day_sunrise, sunset)
    } else {
        Interval::literal(sunset, next_sunrise)
    })
}

#[test]
fn every_recorded_solar_upagraha_is_where_the_chain_puts_it() {
    let mut checked = 0;
    let mut worst = 0.0_f64;
    for chart in charts() {
        let found = solar::chain(chart.sun).expect("a finite Sun");
        for derived in found {
            // The engine spells Parivesha without its last letter.
            let key = if derived.point == Point::Parivesha {
                "PARIVESH"
            } else {
                derived.point.key()
            };
            let Some(recorded) = chart.upagraha(key) else {
                continue;
            };
            let gap = apart(derived.longitude_deg, recorded);
            worst = worst.max(gap);
            assert!(
                gap <= EXACT_DEG,
                "{} {key}: {} against {recorded}",
                chart.name,
                derived.longitude_deg
            );
            checked += 1;
        }
    }
    println!("{checked} solar upagrahas, worst {worst:.9}°");
    assert_eq!(checked, 71 * 5, "five on every fixture that records them");
}

#[test]
fn the_yogi_points_and_the_sree_lagna_are_exact() {
    let mut checked = 0;
    for chart in charts() {
        for (key, ours) in [
            (
                "yogi_point_deg",
                lagna::yogi(chart.sun, chart.moon).expect("finite"),
            ),
            (
                "avayogi_point_deg",
                lagna::avayogi(chart.sun, chart.moon).expect("finite"),
            ),
            (
                "sree_lagna_deg",
                lagna::sree(chart.lagna, chart.moon).expect("finite"),
            ),
        ] {
            let Some(recorded) = chart.recorded(key) else {
                continue;
            };
            assert!(
                apart(ours.longitude_deg, recorded) <= EXACT_DEG,
                "{} {key}: {} against {recorded}",
                chart.name,
                ours.longitude_deg
            );
            checked += 1;
        }
        // The nakshatra the engine records beside the Yogi is the point's
        // own, which is what makes it a rendering and not a reading.
        if let (Some(recorded), Some(index)) = (
            chart.recorded("yogi_point_deg"),
            chart.recorded("yogi_nakshatra_index"),
        ) {
            let point = lagna::yogi(chart.sun, chart.moon).expect("finite");
            assert!((point.longitude_deg - recorded).abs() <= EXACT_DEG);
            let ours = lagna::nakshatra_of(&point).expect("a nakshatra");
            assert!(
                (f64::from(ours as u16) - index).abs() < f64::EPSILON,
                "{}: {ours:?} against {index}",
                chart.name
            );
        }
    }
    println!("{checked} yogi points and sree lagnas, all exact");
    assert_eq!(checked, 71 * 3);
}

#[test]
fn the_three_the_clock_drives_are_inside_the_bracket_the_pass_published() {
    let mut exact = 0;
    let mut checked = 0;
    let mut worst = 0.0_f64;
    for chart in charts() {
        for (key, ours) in [
            (
                "hora_lagna_deg",
                lagna::hora(chart.sun, chart.hours).expect("an elapsed time"),
            ),
            (
                "ghati_lagna_deg",
                lagna::ghati(chart.sun, chart.hours).expect("an elapsed time"),
            ),
            (
                "pranapada_lagna_deg",
                lagna::pranapada(chart.sun, chart.hours).expect("an elapsed time"),
            ),
        ] {
            let Some(recorded) = chart.recorded(key) else {
                continue;
            };
            let gap = apart(ours.longitude_deg, recorded);
            worst = worst.max(gap);
            exact += usize::from(gap <= EXACT_DEG);
            checked += 1;
            assert!(
                gap <= CLOCK_BOUND_DEG,
                "{} {key}: {} against {recorded}, {gap}° apart",
                chart.name,
                ours.longitude_deg
            );
        }
    }
    println!(
        "{checked} clock-driven lagnas, {exact} exact, worst {worst:.4}° \
         against a bound of {CLOCK_BOUND_DEG:.4}°"
    );
    assert_eq!(checked, 71 * 3);
    assert_eq!(exact, 138, "the pass counted forty-six of each");
}

#[test]
fn gulika_begins_saturns_eighth_and_mandi_ends_it() {
    let mut checked = 0;
    let mut worst = 0.0_f64;
    for chart in charts() {
        let found = eighth::day_division(chart.arc, chart.vara, chart.is_day, &chart)
            .expect("an arc with eighths");
        for derived in found {
            let Some(recorded) = chart.upagraha(derived.point.key()) else {
                continue;
            };
            let gap = apart(derived.longitude_deg, recorded);
            worst = worst.max(gap);
            assert!(
                gap <= ASCENDANT_BOUND_DEG,
                "{} {:?}: {} against {recorded}, {gap}° apart",
                chart.name,
                derived.point,
                derived.longitude_deg
            );
            checked += 1;
        }
    }
    println!("{checked} day-division upagrahas, worst {worst:.6}°");
    assert_eq!(checked, 71 * 2, "Gulika and Mandi on every fixture");
    assert!(worst > 0.0, "they are compared, not trivially equal");
}

#[test]
fn the_varnada_is_recorded_and_the_sdk_does_not_compute_it() {
    // Registry: the engine records a Varnada and no reading of the
    // received rule reproduces it (crux C22), so the SDK ships none.
    // This asserts the *absence* as a deliberate one.
    let mut recorded = 0;
    for chart in charts() {
        let Some(value) = chart.recorded("varnada_lagna_deg") else {
            continue;
        };
        recorded += 1;
        // Every recorded value is a whole sign, which is the one thing
        // about the field the pass did settle.
        assert!(
            (value / 30.0).fract().abs() < f64::EPSILON,
            "{}: {value} is not a whole sign",
            chart.name
        );
    }
    println!("{recorded} Varnada values recorded, none computed");
    assert_eq!(recorded, 71);
    assert!(
        Point::from_key("VARNADA_LAGNA").is_some(),
        "the catalogue keeps the key for whoever cites a school"
    );
}
