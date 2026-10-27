//! The whole chara karaka order of every recorded birth, under seven
//! karakas and under eight, against the order the corpus records
//! (`houses.chara_karakas`), the chart founded by the SDK over its own sky
//! under the profile the corpus was recorded with (`conformance-baseline`).
//!
//! `docs/03-design/jaimini-measured.md` §1 holds the Atmakaraka alone; this
//! holds every rank below it. The order is compared as a ranking — which
//! graha stands at each rank — and the recording's names are read into
//! ranks in the recording's own naming of the eight, the Pitrikaraka last
//! (crux C101). Which name the eighth rank carries is a table, held on its
//! own by `crates/rules/tests/baseline.rs`; the ranking is what a sky
//! decides, and is what the Karaka dasha runs in.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    reason = "tests fail by panicking, index the corpus's JSON by key and print the measurement under --nocapture"
)]

mod common;

use serde_json::Value;
use teistro::catalogue::Graha;
use teistro::settings::CharaKarakas;
use teistro::{ChartRequest, Context, Document, Ephemeris, RuleInputs};

/// The recording's karaka names from the most advanced graha down, under
/// seven karakas and under eight (the Pitrikaraka last, crux C101).
const SCHEMES: [(&str, bool, &[&str]); 2] = [
    (
        "system7",
        false,
        &["AK", "AmK", "BK", "MK", "PK", "GK", "DK"],
    ),
    (
        "system8",
        true,
        &["AK", "AmK", "BK", "MK", "PK", "GK", "DK", "PiK"],
    ),
];

/// A disagreement the corpus's own numbers explain: the birth, the scheme,
/// and why. Each is checked below to be what it says: a graha standing
/// within [`EDGE_ARCSEC`] of a sign's edge, which the two ephemerides put on
/// either side of it, so that it ranks near the top in one and near the
/// bottom in the other, and every other graha keeping its rank order.
const KNOWN: &[(&str, &str, &str)] = &[
    (
        "c051",
        "system7",
        "searched to the second after the Sun's sidereal Aries ingress of 2020: the recording's \
         Sun is 0.02″ short of Aries and ranks first, the SDK's 0.03″ into it and ranks last; \
         the other six keep their order",
    ),
    (
        "c051",
        "system8",
        "the same Sun at the same edge, first in the recording and last in the SDK's; the \
         other seven keep their order",
    ),
];

/// How near a sign's edge a graha must stand for its rank to be the
/// ephemerides' rather than the ranking's: the built-in standard tier's
/// worst Sun is under an arc-second (`jaimini-measured.md` §1 uses the same
/// figure).
const EDGE_ARCSEC: f64 = 1.0;

/// Arc-seconds from a longitude to the nearest sign's edge.
fn to_edge(longitude_deg: f64) -> f64 {
    let into = longitude_deg.rem_euclid(30.0);
    into.min(30.0 - into) * 3600.0
}

/// The recorded ranking of a scheme: the graha at each rank.
fn recorded_ranking(chart: &Value, scheme: &str, names: &[&str]) -> Vec<Graha> {
    let karakas = chart["houses"]["chara_karakas"][scheme]
        .as_object()
        .unwrap();
    names
        .iter()
        .map(|name| {
            let key = karakas
                .iter()
                .find(|(_, karaka)| karaka.as_str() == Some(name))
                .map(|(graha, _)| graha.as_str())
                .unwrap();
            Graha::from_key(key).unwrap()
        })
        .collect()
}

/// The grahas within [`EDGE_ARCSEC`] of a sign's edge in the recording or
/// in the SDK's founding.
fn on_an_edge(chart: &Value, document: &Document) -> Vec<Graha> {
    Graha::ALL
        .into_iter()
        .take(8)
        .filter(|&graha| {
            let recorded = chart["positions"]["bodies"][graha.key()]["sidereal_longitude_deg"]
                .as_f64()
                .unwrap();
            let ours = document.foundation.graha(graha).unwrap().longitude_deg;
            to_edge(recorded).min(to_edge(ours)) <= EDGE_ARCSEC
        })
        .collect()
}

/// A ranking without some grahas.
fn without(ranking: &[Graha], left_out: &[Graha]) -> Vec<Graha> {
    ranking
        .iter()
        .copied()
        .filter(|graha| !left_out.contains(graha))
        .collect()
}

#[test]
fn the_whole_chara_karaka_order_is_the_recorded_one() {
    let sdk = Context::builder()
        .profile("conformance-baseline")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap();
    // The corpus was recorded under eight (every profile of its manifest
    // says `chara_karaka: 8`), and the profile says so too.
    assert_eq!(sdk.settings().jaimini.chara_karakas, CharaKarakas::Eight);
    let manifest = common::fixture("manifest.json");
    for (name, profile) in manifest["profiles"].as_object().unwrap() {
        assert_eq!(
            profile["settings"]["chara_karaka"].as_u64(),
            Some(8),
            "the manifest's {name} profile"
        );
    }

    let mut agree = [0_usize; 2];
    let mut differ: Vec<(String, &str)> = Vec::new();
    let charts = common::charts();
    for (file, chart) in &charts {
        let document = common::reading_of(&sdk, chart, ChartRequest::with_state).unwrap();
        let ruled = RuleInputs::of(&document).unwrap().chart;
        for (at, (scheme, eight, names)) in SCHEMES.into_iter().enumerate() {
            let ours = ruled.chara_karaka_ranking(eight);
            let recorded = recorded_ranking(chart, scheme, names);
            if ours == recorded {
                agree[at] += 1;
                continue;
            }
            println!("{file} {scheme}: recorded {recorded:?}, ours {ours:?}");
            // What explains it: a graha on a sign's edge, and the rest in
            // the same order once it is set aside.
            let edge = on_an_edge(chart, &document);
            assert!(
                !edge.is_empty() && without(&ours, &edge) == without(&recorded, &edge),
                "{file} {scheme}: no graha on a sign's edge accounts for recorded \
                 {recorded:?} against ours {ours:?}"
            );
            differ.push((chart["id"].as_str().unwrap().to_owned(), scheme));
        }
    }
    println!(
        "{} births: {} agree under seven, {} under eight; {} differ",
        charts.len(),
        agree[0],
        agree[1],
        differ.len()
    );
    assert_eq!(charts.len(), 55);
    // Every disagreement is named in `KNOWN`, and every one named is met.
    let met: Vec<(&str, &str)> = differ
        .iter()
        .map(|(id, scheme)| (id.as_str(), *scheme))
        .collect();
    let known: Vec<(&str, &str)> = KNOWN.iter().map(|(id, scheme, _)| (*id, *scheme)).collect();
    assert_eq!(met, known, "the disagreements are the explained ones");
}
