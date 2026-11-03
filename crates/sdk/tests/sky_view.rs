//! The sky view through the façade (`03-design/sky-view.md`): every
//! graha's angles against JPL Horizons' observer tables, which read DE441
//! and share nothing with the SDK; the record and its refusals.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    reason = "tests fail by panicking, index what they asked for and print the measurement under --nocapture"
)]

use std::collections::BTreeMap;

use serde_json::Value;
use teistro::catalogue::Graha;
use teistro::sky_view::EclipticPointKind;
use teistro::{Context, Ephemeris, SkyViewRequest};
use teistro_core::angle::difference_deg;
use teistro_core::error::Status;

/// The worst differences from Horizons, measured here and published on
/// the design page. Horizons' clock is UTC and the SDK reads it as UT1;
/// its planets are DE441's and the built-in's are its own theories.
const ANGLE_BOUND_DEG: f64 = 0.001;
/// The refracted altitude above 2°, where both models are fits to one
/// atmosphere.
const REFRACTED_BOUND_DEG: f64 = 0.001;

fn context() -> Context {
    Context::builder()
        .profile("nepali-drik")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
}

fn request(latitude: f64, longitude: f64, altitude: f64, at: f64) -> SkyViewRequest {
    SkyViewRequest::from_json(&format!(
        r#"{{"latitudeDeg": {latitude}, "longitudeDeg": {longitude}, "altitudeM": {altitude}, "at": {at}, "grahas": ["SUN", "MOON", "MERCURY", "VENUS", "MARS", "JUPITER", "SATURN"], "stars": []}}"#
    ))
    .unwrap()
}

#[test]
fn every_graha_agrees_with_jpl_horizons() {
    let table: Value =
        serde_json::from_str(include_str!("horizons-grahas.json")).expect("the recorded table");
    let sdk = context();
    let mut worst: BTreeMap<&str, f64> = BTreeMap::new();
    let mut note = |name, value: f64| {
        let held = worst.entry(name).or_insert(0.0);
        *held = held.max(value.abs());
    };
    let rows = table["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 224);
    for row in rows {
        let site = &table["sites"][row["site"].as_str().unwrap()];
        let field = |name: &str| row[name].as_f64().unwrap();
        let asked = request(
            site[0].as_f64().unwrap(),
            site[1].as_f64().unwrap(),
            site[2].as_f64().unwrap(),
            field("jd"),
        );
        let view = sdk.almanac().sky_view(&asked).unwrap().value;
        let graha: Graha = row["graha"].as_str().unwrap().parse().unwrap();
        let seen = view.graha(graha).unwrap();
        note("altitude", seen.altitude_deg - field("altitude"));
        // An azimuth runs fast near the zenith, and is held in the sky's
        // own arc.
        note(
            "azimuth",
            difference_deg(seen.azimuth_deg, field("azimuth"))
                * seen.altitude_deg.to_radians().cos(),
        );
        note(
            "right ascension",
            difference_deg(seen.ra_deg, field("ra")) * seen.dec_deg.to_radians().cos(),
        );
        note("declination", seen.dec_deg - field("dec"));
        if field("refracted") > 2.0 {
            note(
                "refracted",
                seen.refracted_altitude_deg - field("refracted"),
            );
        }
    }
    println!("worst against Horizons: {worst:?}");
    for (name, bound) in [
        ("altitude", ANGLE_BOUND_DEG),
        ("azimuth", ANGLE_BOUND_DEG),
        ("right ascension", ANGLE_BOUND_DEG),
        ("declination", ANGLE_BOUND_DEG),
        ("refracted", REFRACTED_BOUND_DEG),
    ] {
        assert!(worst[name] < bound, "{name}: {}", worst[name]);
    }
}

#[test]
fn the_record_answers_the_area_and_draws_its_lists() {
    let sdk = context();
    let text =
        r#"{"latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1400, "at": 2461345.25}"#;
    let asked = SkyViewRequest::from_json(text).unwrap();
    let view = sdk.almanac().sky_view(&asked).unwrap();
    let record = teistro::records::record("almanac.skyView").unwrap();
    let answered = record.answer(&sdk, text).unwrap();
    assert_eq!(answered.value["grahas"].as_array().map(Vec::len), Some(9));
    assert_eq!(answered.value["stars"][13]["star"], "star.SPICA");
    assert_eq!(answered.value["stars"][13]["nakshatra"], "nakshatra.CHITRA");
    let written = sdk.almanac().sky_view_query(&asked).unwrap();
    assert_eq!(answered.value, written.value);
    assert_eq!(
        answered
            .provenance
            .as_ref()
            .map(|p| p.content_hash.to_string()),
        Some(written.provenance.content_hash.to_string())
    );
    assert_eq!(written.provenance.input_hash, view.provenance.input_hash);
    let value = &view.value;
    // 72 longitudes and the two crossings; 72 azimuths.
    assert_eq!(value.ecliptic.len(), 74);
    assert_eq!(value.horizon.len(), 72);
    let kinds = |kind| {
        value
            .ecliptic
            .iter()
            .filter(|point| point.kind == kind)
            .count()
    };
    assert_eq!(
        (
            kinds(EclipticPointKind::Ascendant),
            kinds(EclipticPointKind::Descendant)
        ),
        (1, 1)
    );
    let rahu = value.graha(Graha::Rahu).unwrap();
    let ketu = value.graha(Graha::Ketu).unwrap();
    assert!((rahu.altitude_deg + ketu.altitude_deg).abs() < 1e-9);
    assert!(difference_deg(rahu.azimuth_deg, ketu.azimuth_deg + 180.0).abs() < 1e-9);
    assert!(
        view.provenance
            .applied_conventions
            .iter()
            .any(|convention| convention.knob == "sky.refraction")
    );
}

#[test]
fn a_context_without_a_modern_sky_is_refused() {
    let asked = request(27.7172, 85.324, 1400.0, 2_461_345.25);
    let none = Context::builder()
        .ephemeris([Ephemeris::None])
        .build()
        .unwrap();
    assert_eq!(
        none.almanac().sky_view(&asked).unwrap_err().status,
        Status::Capability
    );
    let text = Context::builder()
        .profile("nepali-committee")
        .ephemeris([Ephemeris::SuryaSiddhanta])
        .build()
        .unwrap();
    let refused = text.almanac().sky_view(&asked).unwrap_err();
    assert_eq!(refused.status, Status::Unsupported);
    assert!(refused.message.contains("C1185"), "{refused:?}");
}

/// `SkyView::MEMBERS` held to serde: every member the value names is
/// written in full, and nothing else changes.
#[test]
fn every_member_crosses_in_full_and_nothing_else_does() {
    let sdk = context();
    let asked = SkyViewRequest::from_json(
        r#"{"latitudeDeg": -33.8688, "longitudeDeg": 151.2093, "altitudeM": 58, "at": 2460689.75}"#,
    )
    .unwrap();
    let view = sdk.almanac().sky_view(&asked).unwrap().value;
    let bare = serde_json::to_value(&view).unwrap();
    let full = view.in_full().unwrap();
    let mut members = 0;
    for (list, field, kind) in [
        ("grahas", "graha", "graha"),
        ("stars", "star", "star"),
        ("stars", "nakshatra", "nakshatra"),
    ] {
        let rows = bare[list].as_array().unwrap();
        for (row, written) in rows.iter().zip(full[list].as_array().unwrap()) {
            let (Value::String(key), Value::String(in_full)) = (&row[field], &written[field])
            else {
                assert!(
                    row[field].is_null() && written[field].is_null(),
                    "{list}.{field}"
                );
                continue;
            };
            assert_eq!(in_full, &format!("{kind}.{key}"));
            members += 1;
        }
    }
    assert_eq!(members, 9 + 27 + 27);
    let unwritten = full
        .to_string()
        .replace("\"graha.", "\"")
        .replace("\"star.", "\"")
        .replace("\"nakshatra.", "\"");
    assert_eq!(unwritten, bare.to_string());
}
