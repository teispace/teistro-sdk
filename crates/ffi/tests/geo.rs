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
use teistro_ffi::geo::{ts_geo_load, ts_geo_resolve};
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
