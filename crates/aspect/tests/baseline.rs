//! The graha drishti against the corpus's `baseline/aspects`: every graded
//! strength the recording engine gave each of the nine grahas on each of the
//! twelve houses, and the houses each aspects fully, from the whole-sign
//! houses the corpus records (`05-testing/01-golden-vectors.md`, the second
//! export's aspects).
//!
//! The houses are what the engine read, and a drishti counted by sign reads
//! the same relation from them: every recorded house is the whole-sign house
//! from the lagna (`03-design/aspect-drishti-measured.md` §4).

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    reason = "tests fail by panicking, index JSON by key and print what they skip"
)]

use std::path::Path;

#[path = "../../core/tests/support/conformance.rs"]
mod conformance;

use conformance::{Tally, carried};

use serde_json::Value;
use teistro_aspect::Strength;
use teistro_aspect::drishti::quarters_under;
use teistro_core::catalogue::Graha;
use teistro_core::settings::{Profile, SettingsPatch};

const SECTION: &str = "baseline/aspects";

/// The nine grahas the engine places, in its own order.
const GRAHAS: [&str; 9] = [
    "SUN", "MOON", "MARS", "MERCURY", "JUPITER", "VENUS", "SATURN", "RAHU", "KETU",
];

/// Every recorded file of the section, charts then variants, each in name
/// order.
fn files() -> Vec<(String, Value)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/baseline/aspects");
    let mut out = Vec::new();
    for dir in ["charts", "variants"] {
        let mut paths: Vec<_> = std::fs::read_dir(root.join(dir))
            .unwrap_or_else(|err| panic!("{dir}: {err}; the corpus lists {SECTION}"))
            .flatten()
            .map(|entry| entry.path())
            .collect();
        paths.sort();
        for path in paths {
            let file = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
            out.push((
                format!("{dir}/{}", path.file_name().unwrap().to_string_lossy()),
                file,
            ));
        }
    }
    out
}

/// The recorded percentage a strength is.
fn percent(strength: Strength) -> u64 {
    u64::from(strength.quarters()) * 25
}

/// The house `count` houses on from `house`, the first being itself.
fn counted(house: u64, count: u64) -> u64 {
    (house + count - 2) % 12 + 1
}

#[test]
fn every_recorded_drishti_is_the_sdk_s_under_the_engine_s_reading() {
    if let Err(skipped) = carried(SECTION) {
        println!("{skipped}");
        return;
    }
    let settings = Profile::shipped("conformance-baseline")
        .unwrap()
        .resolve(&SettingsPatch::default())
        .unwrap()
        .settings;
    let nodes = settings.aspect.node_aspects;
    let files = files();
    let mut tally = Tally::new(SECTION, "the engine's reading");
    for (name, file) in &files {
        for key in GRAHAS {
            let graha = Graha::from_key(key).unwrap();
            let at = file["inputs"]["graha_house"][key].as_u64().unwrap();
            let recorded = &file["strength"][key];
            let mut full = Vec::new();
            for house in 1..=12_u64 {
                let count = (house + 12 - at) % 12 + 1;
                let ours = quarters_under(graha, u8::try_from(count).unwrap(), nodes);
                if ours == Strength::Full {
                    full.push(house);
                }
                tally.same(
                    percent(ours),
                    recorded[house.to_string()].as_u64().unwrap_or(0),
                    || format!("{name} {key} on house {house}"),
                );
            }
            // The houses it aspects fully, the seventh first and then its
            // specials in the engine's order.
            let mut recorded_full: Vec<u64> = file["aspects"][key]
                .as_array()
                .unwrap()
                .iter()
                .map(|h| h.as_u64().unwrap())
                .collect();
            assert_eq!(recorded_full.first(), Some(&counted(at, 7)), "{name} {key}");
            recorded_full.sort_unstable();
            tally.same(full, recorded_full, || {
                format!("{name} {key}: its full aspects")
            });
        }
    }
    assert_eq!(files.len(), 93);
    tally.record();
}
