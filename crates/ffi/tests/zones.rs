//! A zone pack and a range's change points at the C ABI
//! (`03-design/zone-packs.md`, `03-design/change-points.md`): a pack that
//! does not verify is refused by name and leaves the database as it was,
//! one that does replaces it, and a request is read strictly.

#![cfg(feature = "signed-packs")]
#![allow(
    unsafe_code,
    clippy::unwrap_used,
    reason = "tests cross the boundary and fail by panicking"
)]

use core::ffi::CStr;

use teistro_core::Status;
use teistro_ffi::context::TsEphemeris;
use teistro_ffi::panchanga::ts_almanac_changes;
use teistro_ffi::string::TsString;
use teistro_ffi::time::{ts_time_load_zones, ts_time_zone_database};

mod common;

use common::{Ctx, owned};

/// A one-zone pack of an imagined release, as the parity runners pin it.
const PACK: &str = r#"{"format":"teistro.zones/1","version":"2099a","zones":{"Asia/Kathmandu":"VFppZjIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQAAAAEAAAAAAAAAVFppZjIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACAAAAAwAAABD/////ofJ9hAAAAAAeGDCoAQIAAE/8AAAAAE1YAAQAAFDcAApMTVQAKzA1MzAAKzA1NDUACjwrMDU0NT4tNTo0NQo="},"links":{"Asia/Katmandu":"Asia/Kathmandu"}}"#;
const SIGNATURE: &str = "73a4a729d2d30fdf174da7411bf20d8ff297231d816d0004b7435b6e7b70449d2e128616f7fb7cdcb2d6d0aecab2e1a287f806da782520c99bfe770dd0b9b406";
const KEY: &str = "ea4a6c63e29c520abef5507b132ec5f9954776aebebe7b92421eea691446d22c";

fn unhex(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(text.get(at..at + 2).unwrap(), 16).unwrap())
        .collect()
}

fn database(ctx: &Ctx) -> String {
    let mut out = TsString::empty();
    // SAFETY: a live context and a writable slot.
    let status = unsafe { ts_time_zone_database(ctx.handle, &raw mut out) };
    assert_eq!(status, Status::Ok);
    owned(out)
}

fn load(ctx: &Ctx, pack: &[u8], signature: &[u8], at_least: &CStr) -> (Status, String) {
    let key = unhex(KEY);
    let mut out = TsString::empty();
    // SAFETY: a live context, readable ranges, a NUL-terminated floor and
    // a writable slot.
    let status = unsafe {
        ts_time_load_zones(
            ctx.handle,
            pack.as_ptr(),
            pack.len(),
            signature.as_ptr(),
            signature.len(),
            key.as_ptr(),
            key.len(),
            at_least.as_ptr(),
            &raw mut out,
        )
    };
    (
        status,
        if status == Status::Ok {
            owned(out)
        } else {
            String::new()
        },
    )
}

#[test]
fn a_signed_pack_replaces_the_database_and_a_refused_one_leaves_it() {
    let ctx = Ctx::with_ephemeris(0, TsEphemeris::None, None, None, None).unwrap();
    let before = database(&ctx);
    assert!(before.contains("EMBEDDED"), "{before}");

    let signature = unhex(SIGNATURE);
    let (status, _) = load(&ctx, b"{}", &signature, c"");
    assert_eq!(status, Status::InvalidArg);
    assert_eq!(ctx.last_error().2.as_deref(), Some("signature"));
    let (status, _) = load(&ctx, PACK.as_bytes(), &signature, c"2099b");
    assert_eq!(status, Status::InvalidArg);
    assert_eq!(ctx.last_error().2.as_deref(), Some("version"));
    assert_eq!(database(&ctx), before);

    let (status, loaded) = load(&ctx, PACK.as_bytes(), &signature, c"2099a");
    assert_eq!(status, Status::Ok);
    assert!(loaded.contains("PACK"), "{loaded}");
    assert!(loaded.contains("2099a"), "{loaded}");
    assert_eq!(database(&ctx), loaded);
}

#[test]
fn a_changes_request_is_read_strictly_and_names_one_clock() {
    let ctx = Ctx::with_ephemeris(0, TsEphemeris::None, None, None, None).unwrap();
    let asked = |request: &CStr| {
        let mut out = TsString::empty();
        // SAFETY: a live context, a NUL-terminated request and a writable slot.
        let status = unsafe { ts_almanac_changes(ctx.handle, request.as_ptr(), &raw mut out) };
        if status == Status::Ok {
            drop(owned(out));
        }
        status
    };
    assert_eq!(asked(c"{\"first\": 1}"), Status::InvalidArg);
    assert_eq!(
        asked(c"{\"first\": {\"year\": 2026, \"month\": 11, \"day\": 1}, \"latitudeDeg\": 27.7, \"longitudeDeg\": 85.3}"),
        Status::InvalidArg
    );
    assert_eq!(
        ctx.last_error().2.as_deref(),
        Some("changes.utcOffsetSeconds")
    );
}
