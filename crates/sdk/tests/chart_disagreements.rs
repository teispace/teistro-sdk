//! Two charts compared (`03-design/chart-disagreements.md`): two births
//! under one setting read apart, and every refusal of the request named
//! where it stands.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests fail by panicking"
)]

use teistro::catalogue::Graha;
use teistro::{ChartDisagreementRequest, Context, Ephemeris};

const KATHMANDU: &str =
    r#""latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700"#;

fn context() -> Context {
    Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
}

#[test]
fn two_births_under_one_setting_part_where_the_sky_moved() {
    let asked = ChartDisagreementRequest::from_json(&format!(
        r#"{{"first": {{"instant": 2447000.25, {KATHMANDU}}},
            "second": {{"instant": 2447090.5, {KATHMANDU}}},
            "at": 2460000.5}}"#
    ))
    .unwrap();
    let sky = context();
    let differ = sky.chart().disagreements(&sky, &asked).unwrap().value;
    // A quarter year and six hours apart: the Sun is in another sign and
    // the lagna on another side of the sky.
    assert!(differ.grahas.iter().any(|one| one.graha == Graha::Sun));
    assert!(differ.lagna.is_some());
    assert!(differ.at.is_some());
    let full = sky
        .chart()
        .disagreements_request(&sky, &asked)
        .unwrap()
        .value;
    assert!(full.get("grahas").is_some_and(serde_json::Value::is_array));
}

#[test]
fn every_refusal_names_the_field_it_stands_on() {
    let first = format!(r#""first": {{"instant": 2447000.25, {KATHMANDU}}}"#);
    for (text, field) in [
        ("[]".to_owned(), "disagreements"),
        ("{}".to_owned(), "disagreements.first"),
        (format!(r#"{{{first}, "at": "noon"}}"#), "disagreements.at"),
        (format!(r#"{{{first}, "when": 1}}"#), "disagreements.when"),
        (
            format!(r#"{{"first": {{"instants": [2447000.25, 2447001.25], {KATHMANDU}}}}}"#),
            "disagreements.first.instants",
        ),
    ] {
        let refused = ChartDisagreementRequest::from_json(&text).expect_err(&text);
        assert!(
            refused
                .field()
                .is_some_and(|named| named.starts_with(field)),
            "{text}: {:?}",
            refused.field()
        );
    }
}
