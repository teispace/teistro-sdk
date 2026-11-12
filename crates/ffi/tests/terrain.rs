//! The sunrise behind the terrain at the C ABI (SR-91,
//! `03-design/terrain-horizon.md`): `almanac.days` with a horizon profile
//! answers each day's visible rises and sets beside the level horizon's,
//! leaves the days as they were, and names a refused sample by its field.

#![allow(
    unsafe_code,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests cross the boundary, read answers by key and fail by panicking"
)]

use std::ffi::CString;

use serde_json::{Value, json};
use teistro_core::Status;
use teistro_ffi::context::TsEphemeris;
use teistro_ffi::record::{TsDetail, ts_record_answer};
use teistro_ffi::string::TsString;

mod common;

use common::{Ctx, owned};

/// `almanac.days` at Kathmandu for a day, with `profile` when given.
fn days(ctx: &Ctx, profile: Option<Value>) -> (Status, Option<Value>) {
    let mut request = json!({
        "first": {"year": 2026, "month": 11, "day": 12},
        "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1400,
        "utcOffsetSeconds": 20_700,
    });
    if let Some(profile) = profile {
        request["horizonProfile"] = profile;
    }
    let request = CString::new(request.to_string()).unwrap();
    let mut out = TsString::empty();
    // SAFETY: a live context, NUL-terminated strings, no mask and a writable slot.
    let status = unsafe {
        ts_record_answer(
            ctx.handle,
            c"almanac.days".as_ptr(),
            request.as_ptr(),
            TsDetail::Full as u32,
            core::ptr::null(),
            &raw mut out,
        )
    };
    let answer = (status == Status::Ok).then(|| serde_json::from_str(&owned(out)).unwrap());
    (status, answer)
}

#[test]
fn a_profile_answers_the_visible_rise_beside_the_level_one() {
    let ctx = Ctx::with_ephemeris(0, TsEphemeris::Builtin, None, None, None).unwrap();
    let (status, level) = days(&ctx, None);
    assert_eq!(status, Status::Ok);
    let level = level.unwrap();
    let (status, hills) = days(&ctx, Some(json!([{"azimuthDeg": 0, "elevationDeg": 2}])));
    assert_eq!(status, Status::Ok);
    let hills = hills.unwrap();
    // The days are the days, whatever the skyline.
    assert_eq!(level["value"]["days"], hills["value"]["days"]);
    assert!(level["value"].get("visible").is_none());
    let sun = &hills["value"]["visible"]["value"][0]["sun"];
    let rise = sun["rise"].as_f64().unwrap();
    let seen = sun["visibleRise"]["at"].as_f64().unwrap();
    // Some eleven minutes behind two degrees of hills.
    let minutes = (seen - rise) * 1_440.0;
    assert!((10.0..13.5).contains(&minutes), "{minutes}");
    assert_eq!(sun["visibleRise"]["terrainDeg"], 2.0);
    assert_eq!(sun["visibleState"], "CROSSES");
}

#[test]
fn a_refused_sample_is_named_by_its_field() {
    let ctx = Ctx::with_ephemeris(0, TsEphemeris::Builtin, None, None, None).unwrap();
    for (profile, field) in [
        (json!([]), "horizonProfile"),
        (
            json!([{"azimuthDeg": 400, "elevationDeg": 0}]),
            "horizonProfile[0].azimuthDeg",
        ),
        (
            json!([{"azimuthDeg": 0, "elevationDeg": 1}, {"azimuthDeg": 0, "elevationDeg": 2}]),
            "horizonProfile[1].azimuthDeg",
        ),
        (
            json!([{"azimuthDeg": 0, "elevationDeg": 90}]),
            "horizonProfile[0].elevationDeg",
        ),
    ] {
        assert_eq!(days(&ctx, Some(profile)).0, Status::InvalidArg, "{field}");
        let (_, _, named, _, _) = ctx.last_error();
        assert_eq!(named.as_deref(), Some(field));
    }
}
