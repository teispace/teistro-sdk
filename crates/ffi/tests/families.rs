//! The module families at the boundary (`03-design/wasm-profiles.md`):
//! each answers in a build with it, and a build without it refuses a call
//! into it as `CAPABILITY`, naming the record.
#![allow(
    unsafe_code,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests cross the boundary and fail by panicking"
)]

use std::ffi::CString;

use teistro_core::Status;
use teistro_core::catalogue::Calendar;
use teistro_ffi::TS_CONTEXT_TEST_PROVIDER;
use teistro_ffi::blob::{TsBlob, ts_blob_free};
#[cfg(feature = "chart")]
use teistro_ffi::chart::TsChartRequest;
use teistro_ffi::panchanga::{TsPanchangaRequest, ts_panchanga_days};
#[cfg(feature = "chart")]
use teistro_ffi::schemas;
use teistro_ffi::string::{TsString, ts_string_free};
#[cfg(feature = "chart")]
use teistro_idl::blob::Reader;

mod common;

use common::{Ctx, sized};
#[cfg(feature = "chart")]
use common::{chart_blob, chart_request};

/// An entry point that reads one JSON request and answers JSON.
type JsonEntry = unsafe extern "C" fn(
    *const teistro_ffi::context::TsContext,
    *const core::ffi::c_char,
    *mut TsString,
) -> Status;

/// The status `entry` answers `request` with.
fn json_status(ctx: &Ctx, entry: JsonEntry, request: &str) -> Status {
    let request = CString::new(request).unwrap();
    let mut json = TsString::empty();
    // SAFETY: a live context, a NUL-terminated request and a valid slot.
    let status = unsafe { entry(ctx.handle, request.as_ptr(), &raw mut json) };
    // SAFETY: a descriptor the library wrote, or the empty one.
    unsafe { ts_string_free(&raw mut json) };
    status
}

/// The status two days of almanac answer with a muhurta record of `text`.
fn muhurta_status(ctx: &Ctx, text: &str) -> Status {
    let muhurta = CString::new(text).unwrap();
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
            muhurta_json: muhurta.as_ptr(),
            festivals_json: core::ptr::null(),
        },
        |r, s| r.struct_size = s,
    );
    let mut blob = TsBlob::empty();
    // SAFETY: a live context, a valid request and a valid slot.
    let status = unsafe { ts_panchanga_days(ctx.handle, &raw const request, &raw mut blob) };
    // SAFETY: a descriptor the library wrote, or the empty one.
    unsafe { ts_blob_free(&raw mut blob) };
    status
}

/// Every family read off a chart answers in a build with it and is
/// refused as `CAPABILITY`, naming the record, in one without it; and a
/// chart that asks for no family is answered in every build with the
/// chart area, which proves each left-out family's list of empty
/// sections: the blob writer refuses a section missing from it, or one
/// named twice (`wasm-profiles.md`).
///
/// A build with the chart area and no other family runs it with
/// `cargo test -p teistro-ffi --no-default-features --features
/// builtin-compact,chart --test families`.
#[cfg(feature = "chart")]
#[test]
fn every_chart_family_answers_in_its_build_and_is_refused_without_it() {
    let ctx = Ctx::new(TS_CONTEXT_TEST_PROVIDER, None, None, None).expect("a test context");
    let instants = [2_451_545.0];
    let place = (27.7172, 85.324);
    let bare = chart_request(&instants, place, 20_700);
    let blob = chart_blob(&ctx, &bare).expect("a chart asking for no family is answered");
    assert!(Reader::parse(&blob, &schemas::charts()).is_ok());

    let record = CString::new("{}").unwrap();
    let text = record.as_ptr();
    /// A record field of the request, its family and the root it is
    /// refused by.
    macro_rules! field {
        ($family:literal, $root:literal, $field:ident) => {
            (
                $family,
                cfg!(feature = $family),
                $root,
                (|request, text| request.$field = text)
                    as fn(&mut TsChartRequest, *const core::ffi::c_char),
            )
        };
    }
    let fields = [
        field!("kp", "kp", kp_json),
        field!("prashna", "prashna", prashna_json),
        field!("remedies", "remedies", remedies_json),
        field!("svg", "theme", theme_json),
        field!("tajika", "varsha", varsha_json),
        field!("western", "progressions", progressions_json),
        field!("western", "westernAspects", western_aspects_json),
        field!("western", "synastry", synastry_json),
        field!("western", "parallels", parallels_json),
        field!("western", "antiscia", antiscia_json),
        field!("western", "midpoints", midpoints_json),
        field!("western", "westernHouses", western_houses_json),
        field!("western", "harmonic", harmonic_json),
    ];
    for (family, built, root, set) in fields {
        let mut asked = chart_request(&instants, place, 20_700);
        set(&mut asked, text);
        let answered = chart_blob(&ctx, &asked);
        if built {
            assert_ne!(
                answered.err(),
                Some(Status::Capability),
                "{family} is built: `{root}`"
            );
        } else {
            assert_eq!(
                answered.err(),
                Some(Status::Capability),
                "{family} is left out: `{root}`"
            );
            let (_, message, field, hint, _) = ctx.last_error();
            assert_eq!(field.as_deref(), Some(root), "the refusal names the record");
            assert!(message.contains(&format!("`{family}`")), "{message}");
            assert!(
                hint.is_some_and(|hint| hint.contains("full")),
                "the hint names the full build"
            );
        }
    }
}

/// The families with entry points of their own answer in a build with
/// them and are refused as `CAPABILITY` in one without.
#[test]
fn every_entry_point_family_answers_in_its_build_and_is_refused_without_it() {
    let ctx = Ctx::new(TS_CONTEXT_TEST_PROVIDER, None, None, None).expect("a test context");
    let entries: [(&str, bool, JsonEntry); 3] = [
        (
            "numerology",
            cfg!(feature = "numerology"),
            teistro_ffi::numerology::ts_numerology_profile,
        ),
        (
            "pakshi",
            cfg!(feature = "pakshi"),
            teistro_ffi::pakshi::ts_pakshi,
        ),
        (
            "rashifal",
            cfg!(feature = "rashifal"),
            teistro_ffi::rashifal::ts_rashifal,
        ),
    ];
    for (family, built, entry) in entries {
        let status = json_status(&ctx, entry, "{}");
        assert_eq!(status == Status::Capability, !built, "{family}: {status:?}");
    }
    let muhurta = muhurta_status(&ctx, "{}");
    assert_eq!(
        muhurta == Status::Capability,
        !cfg!(feature = "muhurta"),
        "muhurta: {muhurta:?}"
    );
}

/// A build without the chart area refuses each of its entry points, and a
/// consumer's own dasha system, as `CAPABILITY` naming `chart`, which is
/// the `panchanga` profile (`wasm-profiles.md`).
///
/// Run with `cargo test -p teistro-ffi --no-default-features --features
/// builtin-compact --test families`.
#[cfg(not(feature = "chart"))]
#[test]
fn the_chart_area_is_refused_without_it() {
    let ctx = Ctx::new(TS_CONTEXT_TEST_PROVIDER, None, None, None).expect("a test context");
    let mut blob = TsBlob::empty();
    let mut json = TsString::empty();
    let key = CString::new("NORTH_INDIAN").unwrap();
    // SAFETY: a live context; the refusing build reads nothing else.
    let statuses = unsafe {
        [
            (
                "ts_chart_found",
                teistro_ffi::chart::ts_chart_found(ctx.handle, core::ptr::null(), &raw mut blob),
            ),
            (
                "ts_chart_layout_row",
                teistro_ffi::chart::ts_chart_layout_row(ctx.handle, key.as_ptr(), &raw mut json),
            ),
            (
                "ts_naam_milan",
                teistro_ffi::naam::ts_naam_milan(ctx.handle, key.as_ptr(), &raw mut blob),
            ),
        ]
    };
    for (entry, status) in statuses {
        assert_eq!(status, Status::Capability, "{entry}");
        let (_, message, _, hint, _) = ctx.last_error();
        assert!(message.contains("`chart`"), "{entry}: {message}");
        assert!(hint.is_some_and(|hint| hint.contains("full")), "{entry}");
    }
    let refused = Ctx::with_dashas("[]").expect_err("a dasha system needs the chart area");
    assert_eq!(refused.0, Status::Capability);
    assert_eq!(refused.2.as_deref(), Some("options.dashas_json"));
}
