//! Nothing the C ABI hands out outlives its free (`05-testing/01-quality-
//! bar.md`, "leak checks in Rust"). Each family of entry points runs its
//! create-use-free cycle once to warm what is built once (the lazily built
//! tables, a context's memo for the same inputs), then [`CYCLES`] times
//! measured, and must give back every byte it took: a leak of one byte a
//! cycle shows as [`CYCLES`] bytes, so the count is exact rather than a
//! curve read for flatness. The answers and the refusals both run, a
//! refused call's record included.
#![cfg(feature = "full")]
#![allow(
    unsafe_code,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests cross the boundary and fail by panicking"
)]

use core::ptr;
use std::ffi::{CString, c_char};

use teistro_core::Status;
use teistro_core::catalogue::Calendar;
use teistro_ffi::TS_CONTEXT_TEST_PROVIDER;
use teistro_ffi::blob::{TsBlob, ts_blob_free};
use teistro_ffi::context::{TsContext, TsEphemeris, ts_context_settings_json};
use teistro_ffi::intl::{ts_intl_entity, ts_intl_render, ts_intl_transliterate};
use teistro_ffi::panchanga::{TsPanchangaRequest, ts_panchanga_days};
use teistro_ffi::positions::ts_positions;
use teistro_ffi::string::{TsStr, TsString, ts_string_free};
use teistro_port_ephemeris::vtable::{ObserverC, PositionRequestC};
use teistro_port_ephemeris::{Body, Frame, TimeScale};
use teistro_test_allocator::{Counting, measure};

mod common;

use common::{Ctx, chart_blob, chart_request, sized};

#[global_allocator]
static ALLOCATOR: Counting = Counting::system();

/// The measured cycles of each family.
const CYCLES: usize = 100;

/// Runs `cycle` once unmeasured and [`CYCLES`] times measured, and fails
/// when the measured runs kept any byte.
fn holds_nothing(family: &str, mut cycle: impl FnMut()) {
    cycle();
    let ((), counts) = measure(|| (0..CYCLES).for_each(|_| cycle()));
    assert_ne!(counts.allocations, 0, "{family} allocated nothing to free");
    assert_eq!(counts.retained(), 0, "{family} kept bytes: {counts:?}");
}

/// A context on the built-in ephemeris, which every family can answer on.
fn builtin() -> Ctx {
    Ctx::with_ephemeris(0, TsEphemeris::Builtin, None, None, None).unwrap()
}

/// A JSON entry point's answer, freed.
type JsonEntry = unsafe extern "C" fn(*const TsContext, *const c_char, *mut TsString) -> Status;

fn json(ctx: &Ctx, entry: JsonEntry, request: &str) -> Status {
    let request = CString::new(request).unwrap();
    let mut answer = TsString::empty();
    // SAFETY: a live context, a NUL-terminated request and a valid slot.
    let status = unsafe { entry(ctx.handle, request.as_ptr(), &raw mut answer) };
    // SAFETY: a descriptor the library wrote, or the empty one.
    unsafe { ts_string_free(&raw mut answer) };
    status
}

#[test]
fn a_context_and_a_refused_constructor_give_back_everything() {
    holds_nothing("a context", || drop(builtin()));
    holds_nothing("a test-provider context", || {
        drop(Ctx::new(TS_CONTEXT_TEST_PROVIDER, None, None, None).unwrap());
    });
    holds_nothing("a refused constructor's owned record", || {
        assert!(Ctx::with_layouts("not json").is_err());
    });
    let ctx = builtin();
    holds_nothing("the settings", || {
        let mut settings = TsString::empty();
        // SAFETY: a live context and a valid slot.
        assert_eq!(
            unsafe { ts_context_settings_json(ctx.handle, &raw mut settings) },
            Status::Ok
        );
        // SAFETY: a descriptor the library wrote.
        unsafe { ts_string_free(&raw mut settings) };
    });
}

#[test]
fn positions_and_charts_give_back_their_blobs() {
    let ctx = builtin();
    let jds = [2_451_545.0, 2_451_546.0];
    let bodies = [Body::Sun.id(), Body::Moon.id()];
    let request = sized(
        PositionRequestC {
            struct_size: 0,
            scale: TimeScale::Tt.id(),
            frame_bits: Frame::CANONICAL.to_bits(),
            speeds: 1,
            has_observer: 0,
            reserved: [0; 2],
            observer: ObserverC::default(),
            jds: jds.as_ptr(),
            jd_count: jds.len(),
            bodies: bodies.as_ptr(),
            body_count: bodies.len(),
        },
        |r, s| r.struct_size = s,
    );
    holds_nothing("positions", || {
        let mut blob = TsBlob::empty();
        // SAFETY: a live context, a valid request and a valid slot.
        assert_eq!(
            unsafe { ts_positions(ctx.handle, &raw const request, &raw mut blob) },
            Status::Ok
        );
        // SAFETY: a descriptor the library wrote.
        unsafe { ts_blob_free(&raw mut blob) };
    });
    let instants = [2_446_431.274_305_6];
    let asked = chart_request(&instants, (27.7172, 85.324), 20_700);
    holds_nothing("a chart", || {
        chart_blob(&ctx, &asked).unwrap();
    });
    let refused = chart_request(&instants, (127.0, 85.324), 20_700);
    holds_nothing("a refused chart and its record", || {
        assert_eq!(chart_blob(&ctx, &refused), Err(Status::InvalidArg));
        drop(ctx.last_error());
    });
}

#[test]
fn the_almanac_and_the_json_families_give_back_their_answers() {
    let ctx = builtin();
    let request = sized(
        TsPanchangaRequest {
            struct_size: 0,
            calendar: Calendar::Gregorian.id(),
            reserved: 0,
            from_year: 2026,
            from_month: 11,
            from_day: 20,
            to_month: 11,
            to_day: 21,
            to_year: 2026,
            latitude_deg: 27.7172,
            longitude_deg: 85.324,
            altitude_m: 1400.0,
            utc_offset_seconds: 20_700,
            sections: 0,
            muhurta_json: ptr::null(),
            festivals_json: ptr::null(),
        },
        |r, s| r.struct_size = s,
    );
    holds_nothing("two days of almanac", || {
        let mut blob = TsBlob::empty();
        // SAFETY: a live context, a valid request and a valid slot.
        assert_eq!(
            unsafe { ts_panchanga_days(ctx.handle, &raw const request, &raw mut blob) },
            Status::Ok
        );
        // SAFETY: a descriptor the library wrote.
        unsafe { ts_blob_free(&raw mut blob) };
    });
    let numerology = r#"{"name": "Henry Elder", "date": {"year": 1872, "month": 1, "day": 17}}"#;
    holds_nothing("a numerology profile", || {
        let entry = teistro_ffi::numerology::ts_numerology_profile;
        assert_eq!(json(&ctx, entry, numerology), Status::Ok);
    });
    let pakshi = r#"{"first": {"year": 1984, "month": 10, "day": 30},
        "latitudeDeg": 13.0827, "longitudeDeg": 80.2707, "altitudeM": 6,
        "utcOffsetSeconds": 19800, "native": {"bird": "COCK"}}"#;
    holds_nothing("a pakshi day", || {
        assert_eq!(
            json(&ctx, teistro_ffi::pakshi::ts_pakshi, pakshi),
            Status::Ok
        );
    });
    let entries: [JsonEntry; 4] = [
        teistro_ffi::numerology::ts_numerology_profile,
        teistro_ffi::pakshi::ts_pakshi,
        teistro_ffi::rashifal::ts_rashifal,
        teistro_ffi::research::ts_research,
    ];
    holds_nothing("refused JSON requests and their records", || {
        for entry in entries {
            assert_ne!(json(&ctx, entry, "{\"unknown\": 1}"), Status::Ok);
            drop(ctx.last_error());
        }
    });
}

#[test]
fn intl_gives_back_what_it_renders() {
    let ctx = builtin();
    let key = CString::new("sdk.condition.vargottama").unwrap();
    let params = CString::new(r#"{"graha": {"$entity": "graha.SUN"}}"#).unwrap();
    holds_nothing("a rendered message", || {
        let mut blob = TsBlob::empty();
        // SAFETY: a live context, NUL-terminated strings and a valid slot.
        let status =
            unsafe { ts_intl_render(ctx.handle, key.as_ptr(), params.as_ptr(), &raw mut blob) };
        assert_eq!(status, Status::Ok, "{:?}", ctx.last_error());
        // SAFETY: a descriptor the library wrote.
        unsafe { ts_blob_free(&raw mut blob) };
    });
    let entity = CString::new("graha.SUN").unwrap();
    let (text, from, to) = (
        CString::new("सूर्य").unwrap(),
        CString::new("deva").unwrap(),
        CString::new("iast").unwrap(),
    );
    holds_nothing("an entity and a transliteration, lent", || {
        let mut lent = TsStr {
            data: ptr::null(),
            len: 0,
        };
        // SAFETY: a live context, NUL-terminated strings and a valid slot.
        let status = unsafe { ts_intl_entity(ctx.handle, entity.as_ptr(), &raw mut lent) };
        assert_eq!(status, Status::Ok, "{:?}", ctx.last_error());
        // SAFETY: the same.
        let status = unsafe {
            ts_intl_transliterate(
                ctx.handle,
                text.as_ptr(),
                from.as_ptr(),
                to.as_ptr(),
                &raw mut lent,
            )
        };
        assert_eq!(status, Status::Ok, "{:?}", ctx.last_error());
    });
}

/// An entry point comparing a context's reckoning with another's.
type PairedEntry = unsafe extern "C" fn(
    *const TsContext,
    *const TsContext,
    *const c_char,
    *mut TsString,
) -> Status;

fn paired(ctx: &Ctx, other: &Ctx, entry: PairedEntry, request: &str) -> Status {
    let request = CString::new(request).unwrap();
    let mut answer = TsString::empty();
    // SAFETY: two live contexts, a NUL-terminated request and a valid slot.
    let status = unsafe { entry(ctx.handle, other.handle, request.as_ptr(), &raw mut answer) };
    // SAFETY: a descriptor the library wrote, or the empty one.
    unsafe { ts_string_free(&raw mut answer) };
    status
}

/// The almanac's rites, its festival calendar and its two comparisons:
/// each answer and each refusal's record given back.
#[test]
fn the_rites_and_the_comparisons_give_back_their_answers() {
    let ctx = builtin();
    let place = r#""latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1337, "utcOffsetSeconds": 20700"#;
    let rites: [(&str, JsonEntry, String); 4] = [
        (
            "a sankalpa",
            teistro_ffi::panchanga::ts_almanac_sankalpa,
            format!(r#"{{"at": 2461055.59375, {place}, "ayana": "TROPICAL"}}"#),
        ),
        (
            "an ashauch",
            teistro_ffi::panchanga::ts_almanac_ashauch,
            format!(
                r#"{{{place}, "request": {{"key": "grandfather", "relation": "SAPINDA", "event": {{"event": "DEATH", "at": 2461051.677083}}}}}}"#
            ),
        ),
        (
            "a first year",
            teistro_ffi::panchanga::ts_almanac_first_year,
            format!(
                r#"{{{place}, "request": {{"key": "grandfather", "at": 2461051.677083, "firstAbdika": "ELEVENTH_DAY", "firstYearAdhika": "NIJA"}}}}"#
            ),
        ),
        (
            "a festival calendar",
            teistro_ffi::panchanga::ts_almanac_festival_calendar,
            format!(
                r#"{{"first": {{"year": 2025, "month": 10, "day": 1}}, "last": {{"year": 2025, "month": 10, "day": 2}}, {place}, "festivals": {{"rules": "NEPAL"}}, "stampUnixSeconds": 1767225600, "name": "Nepal", "refreshMinutes": 0}}"#
            ),
        ),
    ];
    for (family, entry, request) in &rites {
        holds_nothing(family, || {
            assert_eq!(json(&ctx, *entry, request), Status::Ok);
        });
        holds_nothing(&format!("{family} refused, and its record"), || {
            assert_ne!(json(&ctx, *entry, "{\"unknown\": 1}"), Status::Ok);
            drop(ctx.last_error());
        });
    }
    let other = Ctx::with_ephemeris(0, TsEphemeris::SuryaSiddhanta, None, None, None).unwrap();
    let days = format!(
        r#"{{"first": {{"year": 2025, "month": 11, "day": 1}}, "last": {{"year": 2025, "month": 11, "day": 2}}, {place}, "festivals": {{"rules": "NEPAL"}}}}"#
    );
    let comparisons: [(&str, PairedEntry); 2] = [
        (
            "two reckonings compared",
            teistro_ffi::panchanga::ts_almanac_disagreements,
        ),
        (
            "a published day's reasons",
            teistro_ffi::panchanga::ts_almanac_published_reasons,
        ),
    ];
    for (family, entry) in comparisons {
        holds_nothing(family, || {
            assert_eq!(paired(&ctx, &other, entry, &days), Status::Ok);
        });
    }
}

/// A signed pack, two names matched, the engine's own calls (which the
/// built-in refuses), a layout's row and a loaded pack: every answer and
/// every refusal's record given back.
#[test]
fn the_remaining_families_give_back_their_answers() {
    let ctx = builtin();
    // The vector the Rust suite pins (`crates/sdk/tests/signed_pack.rs`).
    let pack = br#"{"format":"teistro.published/1","publisher":"a patro printed in Nepal","id":"np-2083","version":3,"entries":[{"action":"SET","rule":"GHATASTHAPANA","day":{"calendar":"GREGORIAN","year":2026,"month":10,"day":11},"source":"p. 41"}]}"#;
    let unhex = |text: &str| -> Vec<u8> {
        (0..text.len())
            .step_by(2)
            .map(|at| u8::from_str_radix(text.get(at..at + 2).unwrap(), 16).unwrap())
            .collect()
    };
    let signature = unhex(
        "cae87f1a99d4869d5ba41086cd6ceba8cdb035fd11225f53e09882e98423e21332fb40c9ad564380fb9abcdf4c21cb2726f80fb692511b9593f0f9e60fbe320e",
    );
    let key = unhex("ea4a6c63e29c520abef5507b132ec5f9954776aebebe7b92421eea691446d22c");
    let verify = |at_least: u32| {
        let mut answer = TsString::empty();
        // SAFETY: a live context, byte ranges valid for their lengths and
        // a valid slot.
        let status = unsafe {
            teistro_ffi::published::ts_published_verify(
                ctx.handle,
                pack.as_ptr(),
                pack.len(),
                signature.as_ptr(),
                signature.len(),
                key.as_ptr(),
                key.len(),
                at_least,
                &raw mut answer,
            )
        };
        // SAFETY: a descriptor the library wrote, or the empty one.
        unsafe { ts_string_free(&raw mut answer) };
        status
    };
    holds_nothing("a signed pack", || assert_eq!(verify(3), Status::Ok));
    holds_nothing("a replayed signed pack and its record", || {
        assert_eq!(verify(4), Status::InvalidArg);
        drop(ctx.last_error());
    });
    let names = CString::new(
        r#"{"bride": "सीता", "groom": "जोशी", "rules": {"name": {"abhijit": "SHRAVANA"}}}"#,
    )
    .unwrap();
    holds_nothing("two names matched", || {
        let mut blob = TsBlob::empty();
        // SAFETY: a live context, a NUL-terminated request and a valid slot.
        let status =
            unsafe { teistro_ffi::naam::ts_naam_milan(ctx.handle, names.as_ptr(), &raw mut blob) };
        assert_eq!(status, Status::Ok, "{:?}", ctx.last_error());
        // SAFETY: a descriptor the library wrote.
        unsafe { ts_blob_free(&raw mut blob) };
    });
    let (function, arguments) = (
        CString::new("tp_sum").unwrap(),
        CString::new(r#"{"values": [1, 2.5]}"#).unwrap(),
    );
    holds_nothing("an engine's own calls refused, and their records", || {
        let mut answer = TsString::empty();
        // SAFETY: a live context, NUL-terminated strings and a valid slot.
        let status = unsafe {
            teistro_ffi::ephemeris::ts_ephemeris_call(
                ctx.handle,
                function.as_ptr(),
                arguments.as_ptr(),
                &raw mut answer,
            )
        };
        assert_eq!(status, Status::Unsupported);
        drop(ctx.last_error());
        // SAFETY: the same.
        let status =
            unsafe { teistro_ffi::ephemeris::ts_ephemeris_manifest(ctx.handle, &raw mut answer) };
        assert_eq!(status, Status::Unsupported);
        drop(ctx.last_error());
        // SAFETY: a descriptor the library wrote, or the empty one.
        unsafe { ts_string_free(&raw mut answer) };
    });
    let layout = CString::new("chart_layout.SOUTH_INDIAN").unwrap();
    holds_nothing("a layout's row", || {
        let mut answer = TsString::empty();
        // SAFETY: a live context, a NUL-terminated key and a valid slot.
        let status = unsafe {
            teistro_ffi::chart::ts_chart_layout_row(ctx.handle, layout.as_ptr(), &raw mut answer)
        };
        assert_eq!(status, Status::Ok, "{:?}", ctx.last_error());
        // SAFETY: a descriptor the library wrote.
        unsafe { ts_string_free(&raw mut answer) };
    });
    // A shipped pack, as the pack fuzzer's corpus keeps it.
}

#[test]
fn a_loaded_pack_gives_back_its_bytes() {
    let bytes = &common::aspect_pack();
    let load = |bytes: &[u8]| {
        let fresh = builtin();
        let mut loaded = sized(
            teistro_ffi::intl::TsIntlLoaded {
                struct_size: 0,
                entries: 0,
                replaced: 0,
                merged: 0,
                locale: ptr::null(),
                sha256: ptr::null(),
            },
            |r, s| r.struct_size = s,
        );
        // SAFETY: a live context, readable bytes and a sized record.
        let status = unsafe {
            teistro_ffi::intl::ts_intl_load_pack(
                fresh.handle,
                bytes.as_ptr(),
                bytes.len(),
                &raw mut loaded,
            )
        };
        if status != Status::Ok {
            drop(fresh.last_error());
        }
        status
    };
    holds_nothing("a context that loaded a pack", || {
        assert_eq!(load(bytes), Status::Ok);
    });
    holds_nothing("a context that refused a pack, and its record", || {
        assert_ne!(load(b"not a pack"), Status::Ok);
    });
}
