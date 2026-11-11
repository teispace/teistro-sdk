//! The geo port through the façade (`03-design/geo-port.md`, SR-87): the
//! test provider answers each question, a context without one refuses, and
//! a chart resolved through it records the zone, the height and the data
//! version in its receipt and replays the same with the provider removed.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    reason = "tests fail by panicking, and compare the numbers a table states"
)]

use std::sync::Arc;

use serde_json::{Value, json};
use teistro::quantity::JulianDay;
use teistro::verify::{IssueRequest, ReceiptVerdict};
use teistro::{
    ChartRequest, Context, Ephemeris, GeoTable, PlaceRequest, ResolvedPlace, Status, ZoneSpec,
};

const TABLE: &str = include_str!("geo-table.json");

/// Kathmandu, Pokhara, the open Bay of Bengal, and the middle of the
/// Atlantic.
const KATHMANDU: PlaceRequest = PlaceRequest {
    latitude_deg: 27.7172,
    longitude_deg: 85.324,
};
const POKHARA: PlaceRequest = PlaceRequest {
    latitude_deg: 28.2096,
    longitude_deg: 83.9856,
};
const BAY: PlaceRequest = PlaceRequest {
    latitude_deg: 15.0,
    longitude_deg: 88.0,
};
const ATLANTIC: PlaceRequest = PlaceRequest {
    latitude_deg: 0.0,
    longitude_deg: -30.0,
};

/// Noon in Kathmandu on 1 June 2026.
const BIRTH: f64 = 2_461_192.760_417;

fn context(geo: bool) -> Context {
    let builder = Context::builder().ephemeris([Ephemeris::Builtin]);
    let builder = if geo {
        builder.geo(Arc::new(GeoTable::from_json(TABLE).unwrap()))
    } else {
        builder
    };
    builder.build().unwrap()
}

#[test]
fn the_test_provider_answers_each_question() {
    let sdk = context(true);
    let kathmandu = sdk.geo().resolve(&KATHMANDU).unwrap();
    // The zone, inherited from the coarse box.
    assert_eq!(kathmandu.zone.as_deref(), Some("Asia/Kathmandu"));
    // The height, with its source, resolution and datum.
    let height = kathmandu.height.as_ref().unwrap();
    assert_eq!(height.metres, 1337.0);
    assert_eq!(height.source, "the test table");
    assert_eq!(height.resolution_m, 30.0);
    assert_eq!(height.datum, "EGM2008");
    // The admin units, coarse to fine, with their ISO 3166 codes.
    let codes: Vec<Option<&str>> = kathmandu
        .admin
        .iter()
        .map(|unit| unit.code.as_deref())
        .collect();
    assert_eq!(codes, [Some("NP"), Some("NP-P3"), None]);
    assert_eq!(kathmandu.admin[2].name.as_deref(), Some("Kathmandu"));
    // And who answered, from which data.
    assert_eq!(kathmandu.source.name, "teistro-test-places");
    assert_eq!(kathmandu.source.data_version, "2026.11.0");
    assert_eq!(kathmandu.source.tzdb.as_deref(), Some("2026c"));
    assert_eq!(
        (kathmandu.latitude_deg, kathmandu.longitude_deg),
        (27.7172, 85.324)
    );

    // Pokhara: the coarse box alone, so a zone and a country, no height.
    let pokhara = sdk.geo().resolve(&POKHARA).unwrap();
    assert_eq!(pokhara.zone.as_deref(), Some("Asia/Kathmandu"));
    assert!(pokhara.height.is_none());
    assert_eq!(pokhara.admin.len(), 1);
    // At sea: a zone and nothing else.
    let bay = sdk.geo().resolve(&BAY).unwrap();
    assert_eq!(bay.zone.as_deref(), Some("Asia/Kolkata"));
    assert!(bay.height.is_none() && bay.admin.is_empty());
    // Outside every box: no answer is an answer.
    let atlantic = sdk.geo().resolve(&ATLANTIC).unwrap();
    assert!(atlantic.zone.is_none() && atlantic.height.is_none() && atlantic.admin.is_empty());

    // The same through the record, as every binding asks it.
    let record = teistro::records::record("geo.resolve").unwrap();
    let answered = record
        .answer(&sdk, r#"{"latitudeDeg": 27.7172, "longitudeDeg": 85.324}"#)
        .unwrap();
    assert_eq!(
        answered.value,
        serde_json::to_value(&kathmandu).unwrap(),
        "the record answers the area's place"
    );
    assert_eq!(answered.value["height"]["resolutionM"], 30.0);
}

#[test]
fn a_context_resolves_nothing_unless_given_a_provider() {
    let sdk = context(false);
    let refused = sdk.geo().resolve(&KATHMANDU).unwrap_err();
    assert_eq!(
        (refused.status, refused.field()),
        (Status::Capability, Some("geo"))
    );
    // A point out of range is the caller's, whatever the provider.
    let far = PlaceRequest {
        latitude_deg: 91.0,
        longitude_deg: 0.0,
    };
    assert_eq!(
        sdk.geo().resolve(&far).unwrap_err().field(),
        Some("latitudeDeg")
    );
    // A table loaded later is the provider from then on, forks and derived
    // contexts included; a refused one leaves the context as it was.
    let refused = sdk
        .geo()
        .load(r#"{"source": {}, "areas": []}"#)
        .unwrap_err();
    assert!(
        refused
            .field()
            .is_some_and(|field| field.starts_with("geo.source"))
    );
    assert!(sdk.geo().resolve(&KATHMANDU).is_err());
    let source = sdk.geo().load(TABLE).unwrap();
    assert_eq!(source.data_version, "2026.11.0");
    assert!(sdk.geo().resolve(&KATHMANDU).is_ok());
    assert!(sdk.fork().geo().resolve(&KATHMANDU).is_ok());
    let snapshot = teistro::settings::SettingsSnapshot::of(sdk.resolved());
    let derived = sdk.with_settings(&snapshot, None).unwrap();
    assert!(derived.geo().resolve(&KATHMANDU).is_ok());
    // A request the reader does not take.
    let record = teistro::records::record("geo.resolve").unwrap();
    let typo = record
        .answer(&sdk, r#"{"latitudeDeg": 27.7, "longitude": 85.3}"#)
        .unwrap_err();
    assert!(typo.field().is_some_and(|field| field.starts_with("geo")));
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

/// `value` with every number as a float, so `1337` and `1337.0` agree.
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
fn a_chart_on_a_resolved_place_keeps_it_in_its_receipt_and_replays_without_the_provider() {
    let sdk = context(true);
    let place = sdk.geo().resolve(&KATHMANDU).unwrap();
    let request = json!({"instant": BIRTH, "place": place, "state": true});
    let issued = sdk
        .verify()
        .issue(&IssueRequest {
            record: String::from("chart.found"),
            request: request.clone(),
        })
        .unwrap();
    // The receipt records the zone, the height and the data version.
    let kept = &issued.receipt.request["place"];
    assert_eq!(kept["zone"], "Asia/Kathmandu");
    assert_eq!(kept["height"]["metres"], 1337.0);
    assert_eq!(kept["height"]["datum"], "EGM2008");
    assert_eq!(kept["source"]["dataVersion"], "2026.11.0");
    assert_eq!(kept["admin"][1]["code"], "NP-P3");
    // The chart stands at the place's height, on the zone's clock.
    assert_eq!(
        find(&issued.answer["value"]["charts"], "altitude"),
        Some(&json!(1337))
    );
    assert_eq!(
        find(&issued.answer["value"]["charts"], "clock").map(|clock| &clock["offset"]),
        Some(&json!(20_700))
    );

    // Replayed on a context with no provider at all: held.
    let bare = context(false);
    assert!(bare.geo().resolve(&KATHMANDU).is_err());
    let check = bare.verify().receipt(&issued.receipt);
    assert_eq!(check.verdict, ReceiptVerdict::Held, "{check:?}");
    assert!(check.moved.is_empty(), "{check:?}");

    // The same chart as the Rust façade founds it, and as the coordinates
    // written out found it.
    let resolved = ChartRequest::resolved(&place).unwrap();
    let from_place = bare
        .chart()
        .readings(&[JulianDay::try_new(BIRTH).unwrap()], &resolved)
        .unwrap();
    let written = teistro::records::record("chart.found")
        .unwrap()
        .answer(
            &bare,
            &json!({"instant": BIRTH, "latitudeDeg": 27.7172, "longitudeDeg": 85.324,
                "altitudeM": 1337, "zone": ZoneSpec::iana("Asia/Kathmandu"), "state": true})
            .to_string(),
        )
        .unwrap();
    // The receipt's answer is canonical JSON (`1337`), the record's a
    // value (`1337.0`): compared as the numbers they are.
    assert_eq!(
        numbers(&written.value["charts"]),
        numbers(&issued.answer["value"]["charts"]),
        "a resolved place founds the chart its parts written out found"
    );
    let foundation = &from_place.value[0].foundation;
    assert_eq!(foundation.place.altitude.get(), 1337.0);
    assert_eq!(foundation.clock.as_ref().unwrap().offset.seconds(), 20_700);
}

#[test]
fn a_resolved_place_is_refused_beside_what_it_already_says() {
    let sdk = context(true);
    let place = serde_json::to_value(sdk.geo().resolve(&KATHMANDU).unwrap()).unwrap();
    let record = teistro::records::record("chart.found").unwrap();
    let field = |request: Value| {
        record
            .answer(&sdk, &request.to_string())
            .unwrap_err()
            .field()
            .unwrap()
            .to_owned()
    };
    for (key, value) in [
        ("latitudeDeg", json!(27.7)),
        ("longitudeDeg", json!(85.3)),
        ("zone", json!({"kind": "IANA", "zone": "Asia/Kathmandu"})),
    ] {
        let mut request = json!({"instant": BIRTH, "place": place});
        request[key] = value;
        assert_eq!(field(request), key);
    }
    // A place with a zone takes no offset; one without needs one.
    assert_eq!(
        field(json!({"instant": BIRTH, "place": place, "utcOffsetSeconds": 20_700})),
        "utcOffsetSeconds"
    );
    let at_sea = serde_json::to_value(sdk.geo().resolve(&ATLANTIC).unwrap()).unwrap();
    assert_eq!(
        field(json!({"instant": BIRTH, "place": at_sea})),
        "place.zone"
    );
    assert!(
        record
            .answer(
                &sdk,
                &json!({"instant": BIRTH, "place": at_sea, "utcOffsetSeconds": -7200}).to_string()
            )
            .is_ok()
    );
    // Neither the coordinates nor a place.
    assert_eq!(
        field(json!({"instant": BIRTH, "utcOffsetSeconds": 20_700})),
        "latitudeDeg"
    );
    assert_eq!(
        field(json!({"instant": BIRTH, "latitudeDeg": 27.7, "utcOffsetSeconds": 20_700})),
        "longitudeDeg"
    );
    // A place edited out of shape is refused by its field under `place`.
    let mut edited = place.clone();
    edited["admin"][1]["code"] = json!("IN-MH");
    assert_eq!(
        field(json!({"instant": BIRTH, "place": edited})),
        "place.admin[1].code"
    );
    let mut far: ResolvedPlace = serde_json::from_value(place).unwrap();
    far.latitude_deg = 95.0;
    assert_eq!(
        ChartRequest::resolved(&far).unwrap_err().field(),
        Some("place.latitudeDeg")
    );
}

/// The convention a resolved place's height is recorded under, from an
/// answer's provenance.
fn height_of(answered: &teistro::records::Answered) -> (String, String) {
    let convention = answered
        .provenance
        .as_ref()
        .unwrap()
        .applied_conventions
        .iter()
        .find(|convention| convention.knob == "observer.height")
        .unwrap();
    (convention.value.clone(), convention.reason.clone())
}

#[test]
fn a_resolved_height_reaches_the_sunrise_and_says_where_it_came_from() {
    // SR-90: a Kathmandu sunrise on the provider's height is the sunrise
    // on the same height given by hand, and the provenance names the
    // provider, its data and the height's own source. The height moves a
    // sunrise reckoned under the air (`horizon-atmosphere.md`): the
    // standard air at the observer's height, the engines' own convention.
    // The almanac's fixed 34' of the shipped profiles reads no height.
    let sdk = airy();
    let place = sdk.geo().resolve(&KATHMANDU).unwrap();
    let days = teistro::records::record("almanac.days").unwrap();
    let first = json!({"year": 2026, "month": 6, "day": 1});
    let resolved = days
        .answer(&sdk, &json!({"first": first, "place": place}).to_string())
        .unwrap();
    let by_hand = days
        .answer(
            &sdk,
            &json!({"first": first, "latitudeDeg": 27.7172, "longitudeDeg": 85.324,
                "altitudeM": 1337, "zone": ZoneSpec::iana("Asia/Kathmandu")})
            .to_string(),
        )
        .unwrap();
    assert_eq!(
        numbers(&resolved.value["days"]),
        numbers(&by_hand.value["days"])
    );
    let (value, reason) = height_of(&resolved);
    assert_eq!(value, "1337 m");
    assert!(
        reason.contains("teistro-test-places") && reason.contains("2026.11.0"),
        "{reason}"
    );
    assert!(
        reason.contains("the test table") && reason.contains("EGM2008"),
        "{reason}"
    );
    // A place written out records nothing: its height is the request's.
    assert!(
        by_hand
            .provenance
            .as_ref()
            .unwrap()
            .applied_conventions
            .iter()
            .all(|convention| convention.knob != "observer.height")
    );
    // The height moves the sunrise: sea level is another answer.
    let sea = days
        .answer(
            &sdk,
            &json!({"first": first, "latitudeDeg": 27.7172, "longitudeDeg": 85.324,
                "zone": ZoneSpec::iana("Asia/Kathmandu")})
            .to_string(),
        )
        .unwrap();
    let sunrise = |answered: &teistro::records::Answered| {
        find(&answered.value["days"][0], "sunrise")
            .cloned()
            .unwrap()
    };
    assert_ne!(sunrise(&resolved), sunrise(&sea));

    // The caller's own altitude wins over the provider's, and says so.
    let stated = days
        .answer(
            &sdk,
            &json!({"first": first, "place": place, "altitudeM": 0}).to_string(),
        )
        .unwrap();
    assert_eq!(numbers(&stated.value["days"]), numbers(&sea.value["days"]));
    let (value, reason) = height_of(&stated);
    assert_eq!(value, "0 m");
    assert!(reason.starts_with("stated by the caller"), "{reason}");
    assert!(reason.contains("1337 m"), "{reason}");

    // A place the provider knows no height for stands at sea level, and
    // says that too.
    let pokhara = sdk.geo().resolve(&POKHARA).unwrap();
    let low = days
        .answer(&sdk, &json!({"first": first, "place": pokhara}).to_string())
        .unwrap();
    let (value, reason) = height_of(&low);
    assert_eq!(value, "0 m");
    assert!(reason.contains("knows no height here"), "{reason}");
}

/// A context with the test table whose sunrise is reckoned under the
/// standard air at the observer's height, the engines' own convention.
fn airy() -> Context {
    Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .settings_json(
            r#"{"day": {"sunrise": {"kind": "ATMOSPHERIC", "which": "UPPER_LIMB_REFRACTION", "air": {"pressure_hpa": null, "temperature_c": null}}}}"#,
        )
        .geo(Arc::new(GeoTable::from_json(TABLE).unwrap()))
        .build()
        .unwrap()
}

#[test]
fn a_resolved_height_is_recorded_in_a_chart_and_a_receipt() {
    let sdk = airy();
    let place = sdk.geo().resolve(&KATHMANDU).unwrap();
    let first = json!({"year": 2026, "month": 6, "day": 1});
    // A chart says the same in its own provenance, and a receipt keeps it
    // and replays it without the provider.
    let found = teistro::records::record("chart.found").unwrap();
    let chart = found
        .answer(&sdk, &json!({"instant": BIRTH, "place": place}).to_string())
        .unwrap();
    assert_eq!(height_of(&chart).0, "1337 m");
    let issued = sdk
        .verify()
        .issue(&IssueRequest {
            record: String::from("almanac.days"),
            request: json!({"first": first, "place": place}),
        })
        .unwrap();
    let conventions = &issued.answer["provenance"]["applied_conventions"];
    assert!(
        conventions
            .as_array()
            .unwrap()
            .iter()
            .any(|convention| convention["knob"] == "observer.height"
                && convention["reason"]
                    .as_str()
                    .is_some_and(|reason| reason.contains("the test table"))),
        "{conventions}"
    );
    let check = context(false).verify().receipt(&issued.receipt);
    assert_eq!(check.verdict, ReceiptVerdict::Held, "{check:?}");
}
