//! The recorded houses through this crate.
//!
//! `cargo xtask houses` measured the parts of `houses.*` nothing else
//! reads, without using this crate, and wrote what it found to
//! `03-design/houses-measured.md`. This holds the code to it: the shift
//! counted **both ways**, and the three degeneracy disagreements
//! asserted *as* disagreements rather than skipped.

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
use teistro_astro::houses::{ChartFrame, Outcome, houses_at};
use teistro_astro::scale::tt_of;
use teistro_core::catalogue::{Graha, HouseSystem, Rashi};
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Ut1};
use teistro_core::settings::PolarPolicy;
use teistro_houses::classify::{self, HOUSES};

/// The nine grahas the engine places.
const GRAHAS: [&str; 9] = [
    "SUN", "MOON", "MARS", "MERCURY", "JUPITER", "VENUS", "SATURN", "RAHU", "KETU",
];

/// The charts the pass names, where the engine's `is_degenerate` and the
/// SDK's outcome disagree — registry entry 26. Each is asserted as a
/// difference rather than skipped.
const REGISTERED: [(&str, bool); 3] = [
    // The engine flags these; the SDK computes Placidus at 64°.
    ("c027-reykjavik-1975-06-21--placidus", true),
    ("c043-fairbanks-2015-06-21--placidus", true),
    // And leaves this clear at 69.6°, where the SDK cannot.
    ("c028-troms-1988-06-21--placidus", false),
];

struct Chart {
    name: String,
    system: HouseSystem,
    ascendant: f64,
    degenerate: bool,
    place: Place,
    jd: f64,
    ayanamsha: Option<f64>,
    sandhi: Vec<f64>,
    shifted: Vec<String>,
    longitudes: Vec<(String, f64)>,
}

fn system_of(name: &str) -> Option<HouseSystem> {
    HouseSystem::ALL
        .into_iter()
        .find(|system| system.key().to_lowercase().replace('_', "-") == name)
}

fn numbers(value: &Value) -> Vec<f64> {
    value
        .as_array()
        .map(|list| list.iter().filter_map(Value::as_f64).collect())
        .unwrap_or_default()
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
            let Some(selected) = value["houses"]["selected"].as_object() else {
                continue;
            };
            let place = &value["input"]["place"];
            let chalit = &value["houses"]["bhava_chalit"];
            let Some(system) = selected
                .get("system")
                .and_then(Value::as_str)
                .and_then(system_of)
            else {
                continue;
            };
            charts.push(Chart {
                name: path
                    .file_stem()
                    .map(|stem| stem.to_string_lossy().to_string())
                    .unwrap_or_default(),
                system,
                ascendant: selected["ascendant"].as_f64().expect("an ascendant"),
                degenerate: selected
                    .get("is_degenerate")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                place: Place::new(
                    Latitude::try_new(place["latitude"].as_f64().expect("a latitude"))
                        .expect("in range"),
                    Longitude::try_new(place["longitude"].as_f64().expect("a longitude"))
                        .expect("in range"),
                    Altitude::try_new(place["altitude_m"].as_f64().unwrap_or(0.0))
                        .expect("in range"),
                ),
                jd: value["foundation"]["jd_ut"]
                    .as_f64()
                    .or_else(|| value["input"]["resolved"]["jd_ut"].as_f64())
                    .expect("an instant"),
                ayanamsha: value["foundation"]["ayanamsha"]["value_deg"].as_f64(),
                sandhi: numbers(&chalit["bhava_sandhi"]),
                shifted: chalit["shifted"]
                    .as_array()
                    .map(|list| {
                        list.iter()
                            .filter_map(|entry| entry["planet"].as_str().map(ToString::to_string))
                            .collect()
                    })
                    .unwrap_or_default(),
                longitudes: value["positions"]["bodies"]
                    .as_object()
                    .map(|bodies| {
                        GRAHAS
                            .into_iter()
                            .filter_map(|body| {
                                Some((
                                    body.to_string(),
                                    bodies.get(body)?["sidereal_longitude_deg"].as_f64()?,
                                ))
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
            });
        }
    }
    assert!(!charts.is_empty(), "the corpus carries houses");
    charts
}

/// Which sign a longitude falls in, 0 for Aries.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a normalised longitude over thirty is 0 to 11"
)]
fn sign_index(longitude_deg: f64) -> u8 {
    ((longitude_deg.rem_euclid(360.0) / 30.0) as u8).min(11)
}

/// Which bhava a longitude falls in, given the twelve sandhi.
fn house_of(longitude: f64, sandhi: &[f64]) -> u8 {
    for index in 0..HOUSES {
        let from = sandhi[usize::from(index)];
        let to = sandhi[usize::from((index + 1) % HOUSES)];
        let span = (to - from).rem_euclid(360.0);
        if (longitude - from).rem_euclid(360.0) < span {
            return index + 1;
        }
    }
    1
}

#[test]
fn the_shift_is_the_same_set_counted_either_way() {
    let mut listed = 0;
    let mut found = 0;
    let mut seen = 0;
    for chart in charts() {
        if chart.sandhi.len() != usize::from(HOUSES) || chart.longitudes.is_empty() {
            continue;
        }
        seen += 1;
        listed += chart.shifted.len();
        for (body, longitude) in &chart.longitudes {
            let chalit = house_of(*longitude, &chart.sandhi);
            let whole = (sign_index(*longitude) + 12 - sign_index(chart.ascendant)) % 12 + 1;
            let shifts = chalit != whole;
            let engine_says = chart.shifted.iter().any(|who| who == body);
            assert_eq!(
                shifts, engine_says,
                "{} {body}: chalit {chalit}, whole sign {whole}",
                chart.name
            );
            found += usize::from(shifts);
        }
    }
    println!("{seen} charts, {listed} listed shifts, {found} found");
    assert_eq!(seen, 75, "the fixtures with a chalit and positions");
    assert_eq!(listed, 135, "the pass counted a hundred and thirty-five");
    assert_eq!(found, listed, "the same set, counted either way");
}

#[test]
fn the_degeneracy_disagreements_are_the_three_the_registry_names() {
    let mut disagreeing = Vec::new();
    let mut seen = 0;
    for chart in charts() {
        seen += 1;
        let ut1 = JulianDay::<Ut1>::literal(chart.jd);
        let (tt, _) = tt_of(ut1, DeltaTModel::TableThenModel).expect("a scale");
        let frame = ChartFrame {
            sidereal_offset_deg: chart.ayanamsha.unwrap_or(0.0),
            sun_declination_deg: None,
        };
        let houses = houses_at(
            chart.system,
            ut1,
            tt,
            &chart.place,
            &frame,
            PolarPolicy::FallbackWholeSign,
        )
        .expect("houses");
        let ours = houses.outcome != Outcome::Defined;
        if ours != chart.degenerate {
            disagreeing.push((chart.name.clone(), chart.degenerate));
        }
    }
    println!("{seen} charts, {} disagreements", disagreeing.len());
    assert_eq!(seen, 83, "every fixture that names a selected system");
    for (name, flagged) in &REGISTERED {
        assert!(
            disagreeing
                .iter()
                .any(|(seen, engine)| seen == name && engine == flagged),
            "{name} is a registered disagreement and did not appear"
        );
    }
    assert_eq!(
        disagreeing.len(),
        REGISTERED.len(),
        "and no others: {disagreeing:?}"
    );
}

#[test]
fn a_house_takes_the_lord_of_the_sign_its_middle_falls_in() {
    // Not a comparison — the corpus records no house lords — but a check
    // that the rule is total and agrees with the catalogue over every
    // chart's own division.
    let mut checked = 0;
    for chart in charts() {
        if chart.sandhi.len() != usize::from(HOUSES) {
            continue;
        }
        for index in 0..HOUSES {
            let from = chart.sandhi[usize::from(index)];
            let to = chart.sandhi[usize::from((index + 1) % HOUSES)];
            let madhya = (from + (to - from).rem_euclid(360.0) / 2.0).rem_euclid(360.0);
            let sign = Rashi::from_id(u16::from(sign_index(madhya))).expect("a sign");
            let lord = classify::lord_of(sign);
            assert!(
                Graha::ALL.contains(&lord),
                "{} bhava {}: {lord:?}",
                chart.name,
                index + 1
            );
            // And the middle lies inside the house it belongs to.
            assert_eq!(house_of(madhya, &chart.sandhi), index + 1, "{}", chart.name);
            checked += 1;
        }
    }
    println!("{checked} bhavas, each with a lord and a middle inside it");
    assert_eq!(checked, 83 * usize::from(HOUSES));
}
