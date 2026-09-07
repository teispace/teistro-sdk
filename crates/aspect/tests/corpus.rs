//! The relation counts over the conformance corpus's own placements.
//!
//! The corpus records no aspect, so it cannot say whether one is right.
//! What it can say is **how many** there are in the positions it does
//! record, and that is worth holding: a change that moved the number
//! would be caught here, and the numbers themselves come from
//! `cargo xtask aspect`, which computes them without importing this
//! crate (`03-design/aspect-drishti-measured.md` §§4 and 5).
//!
//! Two independent derivations of the same tables, held to each other.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::print_stdout,
    clippy::indexing_slicing,
    reason = "tests fail by panicking, index JSON by key and print the measurement under --nocapture"
)]

use std::path::Path;

use serde_json::Value;
use teistro_aspect::drishti::{between, house_count, mutually_full};
use teistro_aspect::rashi;
use teistro_core::angle::Nas;
use teistro_core::boundary::Boundaries;
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::quantity::Degrees;

/// The nine grahas the engine gives a position to, in its own order.
const GRAHAS: [&str; 9] = [
    "SUN", "MOON", "MARS", "MERCURY", "JUPITER", "VENUS", "SATURN", "RAHU", "KETU",
];

/// One body as the corpus records it.
struct Body {
    graha: Graha,
    longitude: f64,
    near_sign: bool,
}

impl Body {
    fn sign(&self) -> Rashi {
        at(self.longitude).sign()
    }
}

fn at(degrees: f64) -> Nas {
    Nas::from_degrees(Degrees::try_new(degrees.rem_euclid(360.0)).expect("a finite longitude"))
}

fn charts() -> Vec<Vec<Body>> {
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
            let Some(bodies) = value["positions"]["bodies"].as_object() else {
                continue;
            };
            let mut chart = Vec::new();
            for key in GRAHAS {
                let Some(recorded) = bodies.get(key) else {
                    continue;
                };
                chart.push(Body {
                    graha: Graha::from_key(key).expect("a catalogued graha"),
                    longitude: recorded["sidereal_longitude_deg"]
                        .as_f64()
                        .expect("a longitude"),
                    near_sign: recorded["near_sign_boundary"].as_bool().unwrap_or(false),
                });
            }
            if !chart.is_empty() {
                charts.push(chart);
            }
        }
    }
    assert!(!charts.is_empty(), "the corpus carries positions");
    charts
}

/// Every ordered pair of distinct bodies in a chart.
fn pairs(chart: &[Body]) -> impl Iterator<Item = (&Body, &Body)> {
    chart.iter().flat_map(move |from| {
        chart
            .iter()
            .filter(move |to| to.graha != from.graha)
            .map(move |to| (from, to))
    })
}

#[test]
fn the_relation_counts_are_the_ones_the_pass_published() {
    let charts = charts();
    let (mut all, mut graha, mut full, mut sign, mut both, mut neither) = (0, 0, 0, 0, 0, 0);
    for chart in &charts {
        for (from, to) in pairs(chart) {
            all += 1;
            let strength = between(from.graha, from.sign(), to.sign());
            let rashi = rashi::aspects(from.sign(), to.sign());
            graha += usize::from(strength.is_any());
            full += usize::from(strength.is_full());
            sign += usize::from(rashi);
            both += usize::from(strength.is_any() && rashi);
            neither += usize::from(!strength.is_any() && !rashi);
        }
    }
    println!(
        "{all} pairs: {graha} graha ({full} full), {sign} rashi, {both} both, {neither} neither"
    );
    assert_eq!(charts.len(), 93, "the fixtures that carry positions");
    assert_eq!(
        all, 6696,
        "nine grahas against eight others, ninety-three times"
    );
    assert_eq!(graha, 3679);
    assert_eq!(full, 893);
    assert_eq!(sign, 1718);
    assert_eq!(both, 1377);
    assert_eq!(neither, 2676);
    assert_eq!(
        both + neither,
        4053,
        "the two systems agree on four thousand and fifty-three"
    );
}

#[test]
fn mars_and_saturn_meet_where_the_pass_says_they_do() {
    let mut found = 0;
    let mut charts_with = 0;
    for chart in &charts() {
        let here = pairs(chart)
            .filter(|(from, to)| {
                mutually_full(from.graha, from.sign(), to.graha, to.sign())
                    && house_count(from.sign(), to.sign()) != 7
            })
            .count();
        found += here;
        charts_with += usize::from(here > 0);
    }
    println!("{found} mutual full aspects beyond the seventh on {charts_with} charts");
    assert_eq!(found, 14, "the pass counted fourteen");
    assert_eq!(charts_with, 7, "on seven of the ninety-three");
}

#[test]
fn a_body_near_a_sign_edge_puts_its_relations_in_question() {
    let charts = charts();
    let (mut placements, mut flagged, mut flips, mut all) = (0, 0, 0, 0);
    let mut nearest = f64::INFINITY;
    for chart in &charts {
        for body in chart {
            placements += 1;
            flagged += usize::from(body.near_sign);
            nearest = nearest.min(Boundaries::of(at(body.longitude)).sign_deg);
        }
        for (from, to) in pairs(chart) {
            all += 1;
            if !from.near_sign && !to.near_sign {
                continue;
            }
            let now = between(from.graha, from.sign(), to.sign());
            let moved_from = if from.near_sign {
                neighbour(from.longitude)
            } else {
                from.sign()
            };
            let moved_to = if to.near_sign {
                neighbour(to.longitude)
            } else {
                to.sign()
            };
            flips += usize::from(between(from.graha, moved_from, moved_to) != now);
        }
    }
    println!(
        "{flagged} of {placements} placements flagged, {flips} of {all} relations would change; \
         the closest body stands {:.4}\" from an edge",
        nearest * 3600.0
    );
    assert_eq!(placements, 837);
    assert_eq!(flagged, 4, "the engine's own flags");
    assert_eq!(flips, 52, "the pass counted fifty-two");
    assert!(nearest * 3600.0 < 0.02, "one body is on an edge: {nearest}");
}

/// The sign a body would stand in if it crossed the edge it is nearest.
fn neighbour(longitude: f64) -> Rashi {
    let inside = longitude.rem_euclid(30.0);
    let step = if inside < 30.0 - inside { -1.0 } else { 1.0 };
    at(longitude + step * 30.0).sign()
}

#[test]
fn every_recorded_placement_is_the_whole_sign_house_the_corpus_gives() {
    // Why it matters: over this corpus a drishti counted from the sign
    // and one counted from the recorded house are the *same* relation,
    // so a harness cannot tell them apart here. They part on any chart
    // whose houses come from cusps.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/baseline");
    let mut checked = 0;
    for directory in ["charts", "variants"] {
        let dir = root.join(directory);
        let mut paths: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
            .expect("the corpus")
            .map(|entry| entry.expect("an entry").path())
            .filter(|path| path.extension().is_some_and(|e| e == "json"))
            .collect();
        paths.sort();
        for path in &paths {
            let value: Value =
                serde_json::from_str(&std::fs::read_to_string(path).expect("a fixture"))
                    .expect("valid JSON");
            let Some(bodies) = value["positions"]["bodies"].as_object() else {
                continue;
            };
            let Some(lagna) = bodies
                .get("LAGNA")
                .and_then(|body| body["sidereal_longitude_deg"].as_f64())
            else {
                continue;
            };
            for key in GRAHAS {
                let Some(recorded) = bodies.get(key) else {
                    continue;
                };
                let Some(house) = recorded["house"].as_u64() else {
                    continue;
                };
                let sign = at(recorded["sidereal_longitude_deg"]
                    .as_f64()
                    .expect("a longitude"))
                .sign();
                assert_eq!(
                    u64::from(house_count(at(lagna).sign(), sign)),
                    house,
                    "{} {key}",
                    path.display()
                );
                checked += 1;
            }
        }
    }
    println!("{checked} placements, every one whole-sign");
    assert_eq!(checked, 837);
}
