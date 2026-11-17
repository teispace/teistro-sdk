//! The geo port at the C ABI (`03-design/geo-port.md`, SR-87): a context
//! has no provider until a table is loaded, a table refused leaves it as
//! it was, and a place is resolved through the table loaded, with each
//! refusal named by its field.

#![allow(
    unsafe_code,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests cross the boundary, read answers by key and fail by panicking"
)]

use std::ffi::CString;

use teistro_core::Status;
use teistro_ffi::context::TsEphemeris;
use teistro_ffi::geo::{
    ts_geo_list, ts_geo_load, ts_geo_names, ts_geo_resolve, ts_geo_reverse, ts_geo_search,
};
use teistro_ffi::string::TsString;

mod common;

use common::{Ctx, owned};

/// The SDK's test table, the one every binding's suite loads.
const TABLE: &str = include_str!("../../sdk/tests/geo-table.json");

fn context() -> Ctx {
    Ctx::with_ephemeris(0, TsEphemeris::None, None, None, None).unwrap()
}

/// Loads `table`, answering the status and, when it is `Ok`, the source.
fn load(ctx: &Ctx, table: &str) -> (Status, Option<serde_json::Value>) {
    let table = CString::new(table).unwrap();
    let mut out = TsString::empty();
    // SAFETY: a live context, a NUL-terminated table and a writable slot.
    let status = unsafe { ts_geo_load(ctx.handle, table.as_ptr(), &raw mut out) };
    let source = (status == Status::Ok).then(|| serde_json::from_str(&owned(out)).unwrap());
    (status, source)
}

/// Resolves `request`, answering the status and, when it is `Ok`, the place.
fn resolve(ctx: &Ctx, request: &str) -> (Status, Option<serde_json::Value>) {
    let request = CString::new(request).unwrap();
    let mut out = TsString::empty();
    // SAFETY: a live context, a NUL-terminated request and a writable slot.
    let status = unsafe { ts_geo_resolve(ctx.handle, request.as_ptr(), &raw mut out) };
    let place = (status == Status::Ok).then(|| serde_json::from_str(&owned(out)).unwrap());
    (status, place)
}

const KATHMANDU: &str = r#"{"latitudeDeg": 27.7, "longitudeDeg": 85.3}"#;

#[test]
fn a_context_has_no_provider_until_a_table_is_loaded() {
    let ctx = context();
    assert_eq!(resolve(&ctx, KATHMANDU).0, Status::Capability);
    let (_, _, field, hint, _) = ctx.last_error();
    assert_eq!(field.as_deref(), Some("geo"));
    assert!(hint.unwrap().contains("geo.load"));
}

#[test]
fn a_loaded_table_answers_its_source_and_resolves_coarse_to_fine() {
    let ctx = context();
    let (status, source) = load(&ctx, TABLE);
    assert_eq!(status, Status::Ok);
    let source = source.unwrap();
    assert_eq!(source["name"], "teistro-test-places");
    assert_eq!(source["dataVersion"], "2026.11.0");

    let place = resolve(&ctx, KATHMANDU).1.unwrap();
    // The zone from the country's box, the height and the admin units from
    // the valley's, which comes after it.
    assert_eq!(place["zone"], "Asia/Kathmandu");
    assert_eq!(place["height"]["metres"], 1337.0);
    assert_eq!(place["admin"][1]["code"], "NP-P3");
    assert_eq!(place["source"]["tzdb"], "2026c");

    // A point no box holds answers nothing, which is not a refusal.
    let nowhere = resolve(&ctx, r#"{"latitudeDeg": -40.0, "longitudeDeg": 0.0}"#)
        .1
        .unwrap();
    assert!(nowhere["zone"].is_null());
    assert!(nowhere["height"].is_null());
}

#[test]
fn a_refused_table_leaves_the_context_as_it_was() {
    let ctx = context();
    assert_eq!(load(&ctx, TABLE).0, Status::Ok);
    for table in [
        r#"{"source": {}}"#.to_owned(),
        TABLE.replacen("\"south\": 26.3", "\"south\": 96.3", 1),
        TABLE.replacen("\"code\": \"NP-P3\"", "\"code\": 7", 1),
        "not json".to_owned(),
    ] {
        assert_eq!(load(&ctx, &table).0, Status::InvalidArg, "{table}");
        let (_, _, field, _, _) = ctx.last_error();
        assert!(field.unwrap().starts_with("geo"), "{table}");
    }
    // The table loaded first still answers.
    assert_eq!(
        resolve(&ctx, KATHMANDU).1.unwrap()["zone"],
        "Asia/Kathmandu"
    );
}

#[test]
fn a_request_is_read_strictly_and_a_point_held_to_its_range() {
    let ctx = context();
    assert_eq!(load(&ctx, TABLE).0, Status::Ok);
    for (request, named) in [
        (
            r#"{"latitudeDeg": 91.0, "longitudeDeg": 85.3}"#,
            "latitudeDeg",
        ),
        (
            r#"{"latitudeDeg": 27.7, "longitudeDeg": 181.0}"#,
            "longitudeDeg",
        ),
        (
            r#"{"latitudeDeg": 27.7, "longitudeDeg": 85.3, "zone": "UTC"}"#,
            "zone",
        ),
    ] {
        assert_eq!(resolve(&ctx, request).0, Status::InvalidArg, "{request}");
        let (_, _, field, _, _) = ctx.last_error();
        assert!(field.unwrap().contains(named), "{request}");
    }
    // A point with a coordinate left out is no point.
    assert_eq!(
        resolve(&ctx, r#"{"latitudeDeg": 27.7}"#).0,
        Status::InvalidArg
    );
}

/// An entry point of the place catalogue.
type Catalogue = unsafe extern "C" fn(
    *const teistro_ffi::context::TsContext,
    *const core::ffi::c_char,
    *mut TsString,
) -> Status;

/// Asks `entry` with `request`, answering the status and, when it is `Ok`,
/// the answer.
fn ask(ctx: &Ctx, entry: Catalogue, request: &str) -> (Status, Option<serde_json::Value>) {
    let request = CString::new(request).unwrap();
    let mut out = TsString::empty();
    // SAFETY: a live context, a NUL-terminated request and a writable slot.
    let status = unsafe { entry(ctx.handle, request.as_ptr(), &raw mut out) };
    let answer = (status == Status::Ok).then(|| serde_json::from_str(&owned(out)).unwrap());
    (status, answer)
}

#[test]
fn the_place_catalogue_answers_from_the_loaded_table() {
    let ctx = context();
    // No provider, no catalogue.
    assert_eq!(
        ask(&ctx, ts_geo_search, r#"{"query": "x"}"#).0,
        Status::Capability
    );
    assert_eq!(load(&ctx, TABLE).0, Status::Ok);

    let found = ask(&ctx, ts_geo_search, r#"{"query": "काठमाडौं", "lang": "en"}"#)
        .1
        .unwrap();
    assert_eq!(found[0]["place"]["id"], "np-ktm");
    assert_eq!(found[0]["place"]["name"], "Kathmandu");
    assert_eq!(found[0]["place"]["label"], "Kathmandu, Bagmati, Nepal");
    assert_eq!(found[0]["place"]["kind"], "SETTLEMENT");

    let nearby = ask(
        &ctx,
        ts_geo_reverse,
        r#"{"latitudeDeg": 27.7172, "longitudeDeg": 85.324, "limit": 1, "lang": "ne"}"#,
    )
    .1
    .unwrap();
    assert_eq!(nearby["near"][0]["place"]["name"], "काठमाडौं");
    assert_eq!(nearby["admin"][1]["code"], "NP-P3");

    let names = ask(
        &ctx,
        ts_geo_names,
        r#"{"id": "np-ktm", "langs": ["ne", "sa"]}"#,
    )
    .1
    .unwrap();
    assert_eq!(names["best"][0]["name"]["text"], "काठमाडौं");
    // Sanskrit has no name tagged; the table offers its Devanagari one.
    assert_eq!(names["best"][1]["asked"], "sa");

    let listed = ask(
        &ctx,
        ts_geo_list,
        r#"{"within": "NP", "depth": null, "kinds": ["SETTLEMENT"]}"#,
    )
    .1
    .unwrap();
    let ids: Vec<_> = listed
        .as_array()
        .unwrap()
        .iter()
        .map(|place| place["id"].clone())
        .collect();
    assert_eq!(ids, ["np-ktm", "np-ltp", "np-pkr"]);
    assert_eq!(listed[1]["population"]["kind"], "MODELLED");

    // Each refusal is named by its field.
    for (entry, request, field) in [
        (ts_geo_search as Catalogue, r#"{"query": ""}"#, "query"),
        (ts_geo_search, r#"{"query": "x", "limit": 0}"#, "limit"),
        (ts_geo_search, r#"{"query": "x", "lang": "?"}"#, "lang"),
        (
            ts_geo_reverse,
            r#"{"latitudeDeg": 0.0, "longitudeDeg": 200.0}"#,
            "longitudeDeg",
        ),
        (ts_geo_names, r#"{"id": "np", "langs": ["1"]}"#, "langs[0]"),
        (ts_geo_list, r#"{"within": "NP", "limit": 5000}"#, "limit"),
        (ts_geo_list, r#"{"within": "NP", "parent": "x"}"#, "geo"),
    ] {
        assert_eq!(ask(&ctx, entry, request).0, Status::InvalidArg, "{request}");
        let (_, _, refused_field, _, _) = ctx.last_error();
        assert!(refused_field.unwrap().starts_with(field), "{request}");
    }
}
