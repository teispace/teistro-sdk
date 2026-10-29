//! The Jaimini slice against the corpus's `baseline/jaimini`: from each
//! fixture's recorded longitudes, signs, whole-sign houses and navamsha
//! signs, the chara karakas under seven and eight, the karakamsha under
//! each, the argala on every house, the twelve arudha padas and every
//! graha's rashi drishti, as the recording engine read them
//! (`05-testing/01-golden-vectors.md`, the second export's Jaimini slice).
//!
//! The SDK computes the slice in three crates: the karakas and the argala
//! here in the rules, the karakamsha in `dasha::jaimini`, the padas in
//! `points::arudha` and the rashi drishti in `aspect::rashi`; each part is
//! scored under its own reading.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::print_stdout,
    clippy::too_many_lines,
    reason = "tests fail by panicking, index what they read, print what they skip and walk a file's every comparison in one test"
)]

mod common;
#[path = "../../core/tests/support/conformance.rs"]
mod conformance;

use std::collections::BTreeSet;
use std::path::Path;

use conformance::{Tally, carried};

use serde::Deserialize as _;
use serde_json::Value;
use teistro_aspect::rashi::aspected;
use teistro_core::catalogue::{CharaKaraka, Graha, Rashi};
use teistro_core::house::House;
use teistro_dasha::jaimini::karakamsha;
use teistro_points::arudha::arudha;
use teistro_rules::SignRef;
use teistro_rules::{
    ArgalaPlace, Body, Condition, EightKarakas, Evaluator, Karaka, Participants, Readings,
};

const SECTION: &str = "baseline/jaimini";

/// An intervention the SDK holds by verse 4's second test, one intervening
/// graha stronger than every obstructing one, where the engine reads the
/// count alone.
const STRONGER: &str = "an intervention one stronger graha holds, which the engine's count alone obstructs (rules-engine.md, argala)";

/// The SDK's four interventions, each with the house it comes from.
const PLACES: [(ArgalaPlace, u64); 4] = [
    (ArgalaPlace::Second, 2),
    (ArgalaPlace::Fourth, 4),
    (ArgalaPlace::Fifth, 5),
    (ArgalaPlace::Eleventh, 11),
];

fn sign(value: &Value) -> Rashi {
    Rashi::from_id(u16::try_from(value.as_u64().unwrap()).unwrap()).unwrap()
}

/// The karaka a recorded abbreviation names, or none.
fn karaka(value: &Value) -> Option<CharaKaraka> {
    (!value.is_null()).then(|| Karaka::deserialize(value).unwrap().0)
}

fn keys(value: &Value) -> BTreeSet<String> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_owned())
        .collect()
}

/// The house `count` houses on from `house`, the first being itself.
fn counted(house: u64, count: u64) -> u64 {
    (house + count - 2) % 12 + 1
}

#[test]
fn every_recorded_jaimini_reading_is_the_sdk_s_under_the_engine_s_reading() {
    if let Err(skipped) = carried(SECTION) {
        println!("{skipped}");
        return;
    }
    let files = common::files_in("jaimini");
    let mut karakas = Tally::new(SECTION, "the chara karakas, the Pitrikaraka last");
    let mut amsha = Tally::new(SECTION, "the karakamsha, its houses in the rasi chart");
    let mut argala = Tally::new(SECTION, "the argala from the 2nd, 4th, 5th and 11th");
    let mut padas = Tally::new(SECTION, "the arudha padas");
    let mut drishti = Tally::new(SECTION, "the rashi drishti");
    let mut argala_misses = Vec::new();
    for (path, file) in &files {
        let name = format!(
            "{}/{}",
            path.parent()
                .and_then(Path::file_name)
                .unwrap()
                .to_string_lossy(),
            path.file_name().unwrap().to_string_lossy()
        );
        let inputs = &file["inputs"];
        // The yogas file of the same fixture repeats the whole recorded
        // chart, dignities and all, which the rule chart reads.
        let yogas = common::baseline("yogas")
            .join(path.parent().and_then(Path::file_name).unwrap())
            .join(path.file_name().unwrap());
        let chart = common::chart_at(&yogas, &common::read(&yogas)["inputs"])
            .with_chara_karakas(EightKarakas::PitrikarakaLast);
        let graha_sign = |g: Graha| sign(&inputs["graha_sign_index"][g.key()]);
        for body in Body::ALL.into_iter().take(9) {
            let placed = chart.placement(body);
            assert_eq!(
                placed.longitude,
                inputs["graha_longitude_deg"][body.key()].as_f64().unwrap(),
                "{name} {}: the yogas file and the jaimini file read one chart",
                body.key()
            );
            karakas.same(
                placed.karaka7,
                karaka(&file["chara_karaka_7"][body.key()]),
                || format!("{name} {}: seven", body.key()),
            );
            karakas.same(
                placed.karaka8,
                karaka(&file["chara_karaka_8"][body.key()]),
                || format!("{name} {}: eight", body.key()),
            );
        }

        // The karakamsha under each scheme, from the Atmakaraka the SDK
        // ranks, every graha's house counted in the rasi chart.
        let signs: [Rashi; 9] = std::array::from_fn(|i| graha_sign(Graha::ALL[i]));
        let navamshas: [Rashi; 9] =
            std::array::from_fn(|i| sign(&inputs["d9_sign_index"][Graha::ALL[i].key()]));
        for (scheme, eight) in [("system7", false), ("system8", true)] {
            let atmakaraka = chart.chara_karaka_ranking(eight)[0];
            let ours = karakamsha(atmakaraka, &signs, &navamshas);
            let recorded = &file["karakamsha"][scheme];
            amsha.same(
                ours.atmakaraka.key(),
                recorded["atmakaraka"].as_str().unwrap(),
                || format!("{name} {scheme}: the Atmakaraka"),
            );
            amsha.same(ours.sign, sign(&recorded["karakamsha_sign_index"]), || {
                format!("{name} {scheme}: the karakamsha")
            });
            for (graha, house) in Graha::ALL.into_iter().zip(ours.in_rasi) {
                amsha.same(
                    u64::from(house),
                    recorded["house_from_karakamsha"][graha.key()]
                        .as_u64()
                        .unwrap(),
                    || format!("{name} {scheme} {}: its house", graha.key()),
                );
            }
        }

        // The argala on every house, from the four places the SDK reads.
        let evaluator = Evaluator::new(&chart, Readings::RECORDING_ENGINE);
        for entry in file["argala"].as_array().unwrap() {
            let house = entry["house"].as_u64().unwrap();
            for (place, from) in PLACES {
                let source = counted(house, from);
                let recorded = entry["argalas"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|a| a["from_house"].as_u64() == Some(source));
                let condition = Condition::Argala {
                    on: SignRef::House(House::try_new(u8::try_from(house).unwrap()).unwrap()),
                    place,
                };
                let mut into = Participants::default();
                let held = evaluator.holds(&condition, &mut into);
                let theirs = recorded.is_some_and(|a| !a["obstructed"].as_bool().unwrap());
                if held == theirs {
                    argala.agreed(1);
                } else if held
                    && recorded.is_some_and(|a| {
                        a["planets"].as_array().unwrap().len()
                            <= a["virodha_planets"].as_array().unwrap().len()
                    })
                {
                    // Not outnumbered, so the SDK held it by strength.
                    argala.explained(STRONGER, 1);
                } else {
                    argala_misses.push(format!(
                        "{name} house {house} from {from}: ours {held}, recorded {recorded:?}"
                    ));
                }
                if held && theirs {
                    let ours: BTreeSet<String> = into.iter().map(|b| b.key().to_owned()).collect();
                    argala.same(ours, keys(&recorded.unwrap()["planets"]), || {
                        format!("{name} house {house} from {from}: who intervenes")
                    });
                }
            }
        }

        // The twelve padas, counted to the catalogue's lords.
        let lagna = sign(&inputs["lagna_sign_index"]);
        for recorded in file["arudha_padas"].as_array().unwrap() {
            let house = u8::try_from(recorded["house"].as_u64().unwrap()).unwrap();
            let ours = arudha(lagna, house, graha_sign);
            padas.same(
                (ours.sign, ours.counted, ours.moved),
                (
                    sign(&recorded["sign_index"]),
                    sign(&recorded["original_sign_index"]),
                    recorded["exception_applied"].as_bool().unwrap(),
                ),
                || format!("{name} pada {house}"),
            );
        }

        // Each graha's rashi drishti, as the houses of the signs its own
        // sign aspects.
        for graha in Graha::ALL.into_iter().take(9) {
            let ours: BTreeSet<u64> = aspected(graha_sign(graha))
                .into_iter()
                .map(|to| u64::from(House::between(lagna, to).get()))
                .collect();
            let theirs: BTreeSet<u64> = file["rashi_drishti"][graha.key()]
                .as_array()
                .unwrap()
                .iter()
                .map(|h| h.as_u64().unwrap())
                .collect();
            drishti.same(ours, theirs, || format!("{name} {}", graha.key()));
        }
    }
    for miss in &argala_misses {
        println!("unexplained argala: {miss}");
    }
    assert!(
        argala_misses.is_empty(),
        "{} argala misses no entry explains",
        argala_misses.len()
    );
    assert_eq!(files.len(), 93);
    for tally in [karakas, amsha, argala, padas, drishti] {
        tally.record();
    }
}
