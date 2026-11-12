//! The SDK's own geo suite (`crates/sdk/tests/geo.rs`) run over this
//! provider where it applies: a context built with it resolves a place
//! through `sdk.geo()` and the `geo.resolve` record alike, a chart founded
//! on the place keeps it in its receipt and replays without the provider,
//! the place founds the chart its parts written out found, and its height
//! reaches the sunrise saying where it came from.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking"
)]

mod common;

use std::sync::Arc;

use serde_json::{Value, json};
use teistro::verify::{IssueRequest, ReceiptVerdict};
use teistro::{Context, Ephemeris, PlaceRequest, Status, ZoneSpec};

const KATHMANDU: PlaceRequest = PlaceRequest {
    latitude_deg: 27.7172,
    longitude_deg: 85.324,
};

/// Noon in Kathmandu on 1 June 2026.
const BIRTH: f64 = 2_461_192.760_417;

/// A context with the provider, or `None` when no pack is named.
fn context(test: &str) -> Option<Context> {
    let geo = common::provider(test)?;
    Some(
        Context::builder()
            .ephemeris([Ephemeris::Builtin])
            .geo(Arc::new(geo))
            .build()
            .unwrap(),
    )
}

fn bare() -> Context {
    Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
}

/// The first value at `key` anywhere in `value`.
fn find<'v>(value: &'v Value, key: &str) -> Option<&'v Value> {
    match value {
        Value::Object(fields) => fields
            .get(key)
            .or_else(|| fields.values().find_map(|inner| find(inner, key))),
        Value::Array(items) => items.iter().find_map(|inner| find(inner, key)),
        _ => None,
    }
}

/// `value` with every number as a float, so `1310` and `1310.0` agree.
fn numbers(value: &Value) -> Value {
    match value {
        Value::Number(number) => json!(number.as_f64()),
        Value::Array(items) => Value::Array(items.iter().map(numbers).collect()),
        Value::Object(fields) => Value::Object(
            fields
                .iter()
                .map(|(key, inner)| (key.clone(), numbers(inner)))
                .collect(),
        ),
        _ => value.clone(),
    }
}

#[test]
fn the_sdk_resolves_a_place_through_teigeodata() {
    let Some(sdk) = context("sdk resolve") else {
        return;
    };
    let place = sdk.geo().resolve(&KATHMANDU).unwrap();
    assert_eq!(place.zone.as_deref(), Some("Asia/Kathmandu"));
    assert_eq!(place.source.name, "teigeodata");
    assert!(place.height.is_some());
    // The same through the record, as every binding asks it.
    let answered = teistro::records::record("geo.resolve")
        .unwrap()
        .answer(&sdk, r#"{"latitudeDeg": 27.7172, "longitudeDeg": 85.324}"#)
        .unwrap();
    assert_eq!(answered.value, serde_json::to_value(&place).unwrap());
    // A context without it refuses, as the SDK's suite holds.
    let refused = bare().geo().resolve(&KATHMANDU).unwrap_err();
    assert_eq!(
        (refused.status, refused.field()),
        (Status::Capability, Some("geo"))
    );
}

#[test]
fn a_chart_on_a_teigeodata_place_keeps_it_and_replays_without_it() {
    let Some(sdk) = context("sdk receipt") else {
        return;
    };
    let place = sdk.geo().resolve(&KATHMANDU).unwrap();
    let metres = place.height.as_ref().unwrap().metres;
    let issued = sdk
        .verify()
        .issue(&IssueRequest {
            record: String::from("chart.found"),
            request: json!({"instant": BIRTH, "place": place, "state": true}),
        })
        .unwrap();
    let kept = &issued.receipt.request["place"];
    assert_eq!(kept["zone"], "Asia/Kathmandu");
    assert_eq!(kept["height"]["datum"], "EGM2008");
    assert_eq!(kept["source"]["name"], "teigeodata");
    assert_eq!(
        kept["source"]["dataVersion"].as_str(),
        Some(place.source.data_version.as_str())
    );
    assert_eq!(kept["admin"][0]["code"], "NP");
    assert_eq!(
        find(&issued.answer["value"]["charts"], "clock").map(|clock| &clock["offset"]),
        Some(&json!(20_700))
    );

    // Replayed on a context with no provider at all: held.
    let check = bare().verify().receipt(&issued.receipt);
    assert_eq!(check.verdict, ReceiptVerdict::Held, "{check:?}");
    assert!(check.moved.is_empty(), "{check:?}");

    // The chart its parts written out found.
    let written = teistro::records::record("chart.found")
        .unwrap()
        .answer(
            &bare(),
            &json!({"instant": BIRTH, "latitudeDeg": 27.7172, "longitudeDeg": 85.324,
                "altitudeM": metres, "zone": ZoneSpec::iana("Asia/Kathmandu"), "state": true})
            .to_string(),
        )
        .unwrap();
    assert_eq!(
        numbers(&written.value["charts"]),
        numbers(&issued.answer["value"]["charts"]),
        "a resolved place founds the chart its parts written out found"
    );
}

#[test]
fn a_teigeodata_height_reaches_the_sunrise_and_says_where_it_came_from() {
    let Some(geo) = common::provider("sdk sunrise") else {
        return;
    };
    // The sunrise under the standard air at the observer's height, which
    // reads the height (`geo-port.md` §6a).
    let sdk = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .settings_json(
            r#"{"day": {"sunrise": {"kind": "ATMOSPHERIC", "which": "UPPER_LIMB_REFRACTION", "air": {"pressure_hpa": null, "temperature_c": null}}}}"#,
        )
        .geo(Arc::new(geo))
        .build()
        .unwrap();
    let place = sdk.geo().resolve(&KATHMANDU).unwrap();
    let height = place.height.clone().unwrap();
    let days = teistro::records::record("almanac.days").unwrap();
    let first = json!({"year": 2026, "month": 6, "day": 1});
    let resolved = days
        .answer(&sdk, &json!({"first": first, "place": place}).to_string())
        .unwrap();
    let by_hand = days
        .answer(
            &sdk,
            &json!({"first": first, "latitudeDeg": 27.7172, "longitudeDeg": 85.324,
                "altitudeM": height.metres, "zone": ZoneSpec::iana("Asia/Kathmandu")})
            .to_string(),
        )
        .unwrap();
    assert_eq!(
        numbers(&resolved.value["days"]),
        numbers(&by_hand.value["days"])
    );
    let convention = resolved
        .provenance
        .as_ref()
        .unwrap()
        .applied_conventions
        .iter()
        .find(|convention| convention.knob == "observer.height")
        .unwrap();
    assert!(
        convention.reason.contains("teigeodata") && convention.reason.contains(&height.source),
        "{}",
        convention.reason
    );
}
