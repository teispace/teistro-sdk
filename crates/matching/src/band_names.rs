//! That `sdk.matching.band` names every [`GunaBand`] by the spelling serde
//! gives it, in every locale that carries it, and names nothing else
//! (SR-39).

#![allow(clippy::expect_used, clippy::panic, reason = "tests fail by panicking")]

use std::collections::BTreeSet;
use std::path::Path;

use crate::band::GunaBand;

/// The locales that carry `sdk.matching`.
const LOCALES: [&str; 4] = ["en-Latn", "ne-Deva-NP", "hi-Deva-IN", "sa-Deva"];

/// Each band's spelling, listed by a `match` with no wildcard so a band
/// added fails this build until it is listed, and so until it is named.
fn bands() -> BTreeSet<String> {
    let listed = |band: GunaBand| match band {
        GunaBand::Unfit
        | GunaBand::Fit
        | GunaBand::Middling
        | GunaBand::Good
        | GunaBand::Excellent => band,
    };
    GunaBand::ALL
        .map(listed)
        .iter()
        .map(|band| {
            serde_json::to_value(band)
                .expect("a band serialises")
                .as_str()
                .expect("a band spells as a string")
                .to_owned()
        })
        .collect()
}

/// The keys a one-selector `.match` names, its catch-all left out.
fn variant_keys(message: &str) -> BTreeSet<String> {
    message
        .split("}}")
        .filter_map(|piece| piece.split_once("{{"))
        .filter_map(|(head, _)| head.split_whitespace().last())
        .filter(|key| *key != "*")
        .map(str::to_owned)
        .collect()
}

#[test]
fn every_band_is_named_by_its_spelling_and_nothing_else() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../intl/i18n");
    for locale in LOCALES {
        let path = root.join(locale).join("sdk.matching.json");
        let text = std::fs::read_to_string(&path).expect("the pack is read");
        let messages: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(&text).expect("the pack is JSON");
        let message = messages
            .get("band")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| panic!("{locale} has no sdk.matching.band"));
        assert_eq!(
            variant_keys(message),
            bands(),
            "{locale}'s sdk.matching.band names the bands by their serde spelling"
        );
    }
}

#[test]
fn every_band_is_listed_once_lowest_first() {
    let mut sorted = GunaBand::ALL;
    sorted.sort();
    assert_eq!(sorted, GunaBand::ALL);
    assert_eq!(bands().len(), GunaBand::ALL.len());
}
