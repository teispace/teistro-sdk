//! The chart's zodiac against the recorded charts: one ayanamsha value
//! per chart, and every graha and cusp measured from it.
//!
//! Two things are checked, and neither needs an ephemeris.
//!
//! The first is the SDK's own ayanamsha against the value the recording
//! engine applied — the same comparison `crates/astro` makes against
//! Teimeris, here against the second rank-2 source and over the charts a
//! chart module will actually be run on.
//!
//! The second is the *relation*: that a chart's sidereal longitude is its
//! tropical one less that single value. The corpus holds both readings of
//! all 550 bodies, so the relation can be checked exactly, and it is what
//! justifies the design — the chart asks a provider for tropical
//! positions and shifts them itself, rather than letting the provider
//! apply an ayanamsha of its own to the grahas while the SDK applies one
//! to the cusps.

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
use teistro_astro::precession::PrecessionModel;
use teistro_astro::scale::tt_of;
use teistro_chart::zodiac::ChartZodiac;
use teistro_core::angle::difference_deg;
use teistro_core::quantity::{JulianDay, Ut1};
use teistro_core::settings::{Profile, SettingsPatch};

/// The SDK's Lahiri against the engine's, degrees. `crates/astro` holds
/// the same ayanamsha to 1e-7 arcseconds of Teimeris; the engine behind
/// the corpus is a second implementation, and the spread measured over
/// these 55 charts is 0.0086 arcseconds under the basis it applies. The
/// bound is a hundredth of an arcsecond, which is that with a little
/// room and nothing like enough room to hide the nutation, which is what
/// this comparison found the first time it ran (entry 16).
const AYANAMSHA_BOUND_DEG: f64 = 0.01 / 3600.0;

/// The relation between the two recorded readings is arithmetic, so it
/// holds to the last bit the JSON carries.
const RELATION_BOUND_DEG: f64 = 1e-9;

fn charts() -> Vec<Value> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/baseline/charts");
    let mut charts: Vec<Value> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| {
            panic!(
                "{}: {e}. The corpus is a submodule; `git submodule update --init`",
                dir.display()
            )
        })
        .map(|entry| entry.expect("an entry").path())
        .filter(|path| path.extension().is_some_and(|e| e == "json"))
        .map(|path| {
            serde_json::from_str(&std::fs::read_to_string(path).expect("a fixture"))
                .expect("valid JSON")
        })
        .collect();
    charts.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    charts
}

/// The profile whose job is to reproduce these charts. Not the SDK's own
/// default, which is the texts as read (ADR-0024) and is deliberately not
/// this engine: comparing against it here would be measuring a decision
/// rather than an implementation.
fn settings() -> teistro_core::settings::Settings {
    Profile::shipped("conformance-baseline")
        .unwrap_or_else(|| panic!("the conformance profile"))
        .resolve(&SettingsPatch::default())
        .unwrap_or_else(|e| panic!("{e}"))
        .settings
}

fn zodiac_of(chart: &Value) -> ChartZodiac {
    let ut1 = JulianDay::<Ut1>::literal(chart["foundation"]["jd_ut"].as_f64().expect("an instant"));
    let (tt, _) = tt_of(ut1, DeltaTModel::TableThenModel).unwrap_or_else(|e| panic!("{e}"));
    ChartZodiac::of(
        &settings(),
        tt,
        PrecessionModel::Iau2006,
        DeltaTModel::TableThenModel,
    )
    .unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn the_ayanamsha_is_the_one_the_engine_applied() {
    let charts = charts();
    assert_eq!(charts.len(), 55);
    let mut worst = (0.0_f64, String::new());
    let mut compared = 0;
    for chart in &charts {
        let id = chart["id"].as_str().unwrap();
        // Only the charts under the engine's own Lahiri default; the
        // variants under another ayanamsha are a different comparison.
        if chart["settings"]["frame"]["ayanamsha"]
            .as_str()
            .is_some_and(|a| !a.eq_ignore_ascii_case("lahiri"))
        {
            continue;
        }
        let theirs = chart["foundation"]["ayanamsha"]["value_deg"]
            .as_f64()
            .unwrap_or_else(|| panic!("{id}: no ayanamsha"));
        let apart = (zodiac_of(chart).offset_deg - theirs).abs();
        if apart > worst.0 {
            worst = (apart, id.to_string());
        }
        compared += 1;
    }
    println!(
        "{compared} ayanamsha values, worst {:.4} arcseconds at {}",
        worst.0 * 3600.0,
        worst.1
    );
    assert!(
        worst.0 < AYANAMSHA_BOUND_DEG,
        "worst {:.4}\" at {} exceeds an arcsecond",
        worst.0 * 3600.0,
        worst.1
    );
}

#[test]
fn a_charts_sidereal_longitude_is_its_tropical_one_less_one_value() {
    // The design decision this test exists for: the chart holds one
    // ayanamsha and shifts every tropical longitude by it. If the engine
    // that recorded these charts had used a different value per body —
    // or a different one for the cusps than for the grahas — the relation
    // would not close, and asking a provider for sidereal positions while
    // computing cusps here would be defensible. It closes exactly.
    let charts = charts();
    let mut worst = (0.0_f64, String::new());
    let mut compared = 0;
    for chart in &charts {
        let id = chart["id"].as_str().unwrap();
        let value = chart["foundation"]["ayanamsha"]["value_deg"]
            .as_f64()
            .unwrap_or_else(|| panic!("{id}: no ayanamsha"));
        // The chart's own recorded value, so this measures the relation
        // and not the ayanamsha, which the test above measures.
        let zodiac = ChartZodiac {
            request: teistro_port_ephemeris::Frame::CANONICAL,
            offset_deg: value,
            ayanamsha: None,
        };
        let Some(bodies) = chart["positions"]["bodies"].as_object() else {
            continue;
        };
        for (name, body) in bodies {
            let (Some(sidereal), Some(tropical)) = (
                body["sidereal_longitude_deg"].as_f64(),
                body["tropical_longitude_deg"].as_f64(),
            ) else {
                continue;
            };
            let apart = difference_deg(zodiac.of_tropical(tropical), sidereal).abs();
            if apart > worst.0 {
                worst = (apart, format!("{id} {name}"));
            }
            // And back again, which is the property a chart layer relies
            // on when it hands a longitude to a module that wants the
            // other reading.
            assert!(
                difference_deg(zodiac.to_tropical(sidereal), tropical).abs() < RELATION_BOUND_DEG,
                "{id} {name}: the shift does not round-trip"
            );
            compared += 1;
        }
    }
    println!(
        "{compared} bodies, worst {:.3e}° between the two readings at {}",
        worst.0, worst.1
    );
    assert!(
        worst.0 < RELATION_BOUND_DEG,
        "worst {:.3e}° at {}: the two readings are not one value apart",
        worst.0,
        worst.1
    );
    assert_eq!(compared, 550, "ten bodies on fifty-five charts");
}

#[test]
fn the_lagna_stands_in_the_same_zodiac_as_the_grahas() {
    // The reason the chart holds one value rather than two: a placement
    // compares a graha's longitude with a cusp's, and the two have to be
    // in one zodiac for the comparison to mean anything. The corpus
    // records the lagna sidereally and the cusps sidereally, so the check
    // is that the recorded lagna is the recorded ascendant.
    let charts = charts();
    let mut compared = 0;
    for chart in &charts {
        let id = chart["id"].as_str().unwrap();
        let (Some(lagna), Some(ascendant)) = (
            chart["foundation"]["lagna"]["sidereal_longitude_deg"].as_f64(),
            chart["houses"]["selected"]["ascendant"].as_f64(),
        ) else {
            continue;
        };
        assert!(
            difference_deg(lagna, ascendant).abs() < RELATION_BOUND_DEG,
            "{id}: the lagna {lagna} is not the ascendant {ascendant}"
        );
        // And its sign follows from the longitude, thirty degrees apiece.
        let sign = chart["foundation"]["lagna"]["sign_index"]
            .as_u64()
            .unwrap_or_else(|| panic!("{id}: no sign"));
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a longitude divided by thirty is 0 to 11"
        )]
        let expected = (lagna.rem_euclid(360.0) / 30.0) as u64;
        assert_eq!(sign, expected, "{id}: the lagna's sign");
        compared += 1;
    }
    println!("{compared} lagnas checked against their ascendant");
    assert_eq!(compared, 55);
}
