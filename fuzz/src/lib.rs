//! What the fuzz targets share: one context a process, the entry points
//! that read a caller's text, and the rule that makes an answer a finding
//! (`docs/05-testing/03-fuzzing.md` §2). A refusal is the answer most
//! inputs should get; a panic aborts under the fuzzer's hook before the
//! boundary can catch it, and an `INTERNAL` that is not a caught panic
//! is a finding all the same.

#![allow(
    unsafe_code,
    reason = "a fuzz target crosses the C ABI exactly as a binding does"
)]

use core::ffi::c_char;
use core::ptr;
use std::ffi::CString;
use std::sync::OnceLock;

use teistro_core::Status;
use teistro_core::catalogue::Calendar;
use teistro_ffi::blob::{TsBlob, ts_blob_free};
use teistro_ffi::chart::{TsChartRequest, ts_chart_found};
use teistro_ffi::context::{
    TsContext, TsContextOptions, TsEphemeris, TsError, ts_context_free, ts_context_new,
    ts_error_free,
};
use teistro_ffi::intl::{
    TsIntlLoaded, ts_intl_entity, ts_intl_has, ts_intl_load_pack, ts_intl_render,
    ts_intl_set_locale, ts_intl_transliterate,
};
use teistro_ffi::key::ts_key_parse;
use teistro_ffi::naam::ts_naam_milan;
use teistro_ffi::numerology::ts_numerology_profile;
use teistro_ffi::pakshi::ts_pakshi;
use teistro_ffi::panchanga::{TsPanchangaRequest, ts_panchanga_days};
use teistro_ffi::rashifal::ts_rashifal;
use teistro_ffi::research::ts_research;
use teistro_ffi::string::{TsStr, TsString, ts_string_free};
use teistro_mcp::Limits;

/// A size as a C caller writes it into `struct_size`.
fn size<T>() -> u32 {
    u32::try_from(size_of::<T>()).unwrap_or(u32::MAX)
}

/// The text a C caller would pass: the bytes up to the first NUL.
fn c_text(bytes: &[u8]) -> CString {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    CString::new(bytes.get(..end).unwrap_or_default()).unwrap_or_default()
}

/// The finding rule: any status but `INTERNAL` is an answer.
fn answered(status: Status, at: &str) {
    assert_ne!(status, Status::Internal, "{at} answered INTERNAL");
}

/// A context built with `ephemeris` and, optionally, one JSON option;
/// `None` when the context refuses it, which is an answer.
fn open(ephemeris: TsEphemeris, option: Option<(usize, &CString)>) -> Option<*mut TsContext> {
    let text = |slot: usize| {
        option
            .filter(|(at, _)| *at == slot)
            .map_or(ptr::null(), |(_, text)| text.as_ptr())
    };
    let options = TsContextOptions {
        struct_size: size::<TsContextOptions>(),
        flags: 0,
        profile: ptr::null(),
        settings_json: text(0),
        locale: ptr::null(),
        layouts_json: text(1),
        dashas_json: text(2),
        ephemeris: ephemeris as u8,
    };
    let mut handle = ptr::null_mut();
    let mut error = TsError {
        struct_size: size::<TsError>(),
        status: 0,
        provider_code: 0,
        flags: 0,
        detail: ptr::null(),
        message: ptr::null(),
        field: ptr::null(),
        hint: ptr::null(),
        key: ptr::null(),
    };
    // SAFETY: valid pointers for the call.
    let status = unsafe {
        ts_context_new(
            &raw const options,
            ptr::null(),
            ptr::null_mut(),
            &raw mut handle,
            &raw mut error,
        )
    };
    answered(status, "ts_context_new");
    if status == Status::Ok {
        return Some(handle);
    }
    // SAFETY: a record the library wrote, freed once.
    unsafe { ts_error_free(&raw mut error) };
    None
}

/// The process's context on the built-in ephemeris, built once and never
/// freed: every target reads it, and none changes what it answers.
fn shared() -> *const TsContext {
    static CONTEXT: OnceLock<usize> = OnceLock::new();
    let address = *CONTEXT
        .get_or_init(|| open(TsEphemeris::Builtin, None).map_or(0, |handle| handle as usize));
    assert_ne!(address, 0, "the default context opens");
    address as *const TsContext
}

/// Frees a context this crate opened.
fn close(handle: *mut TsContext) {
    // SAFETY: the handle came from `ts_context_new` and is freed once.
    unsafe { ts_context_free(handle) };
}

macro_rules! chart_slots {
    ($($field:ident),* $(,)?) => {
        /// Every JSON section of a chart request, in the order the
        /// selector byte counts them.
        pub const CHART_SLOTS: &[&str] = &[$(stringify!($field)),*];

        /// Points the slot `at` of `request` at `text`.
        fn set_slot(request: &mut TsChartRequest, at: usize, text: *const c_char) {
            let mut slot = 0;
            $(
                if slot == at {
                    request.$field = text;
                }
                slot += 1;
            )*
            let _ = slot;
        }
    };
}

chart_slots!(
    theme_json,
    rules_json,
    interpret_json,
    varsha_json,
    gochar_json,
    hits_json,
    sade_sati_json,
    kp_json,
    dignities_json,
    fortitudes_json,
    lots_json,
    considerations_json,
    perfection_json,
    progressions_json,
    western_aspects_json,
    synastry_json,
    parallels_json,
    antiscia_json,
    midpoints_json,
    western_houses_json,
    harmonic_json,
    matching_json,
    prashna_json,
    remedies_json,
    rectification_json,
    lalkitab_json,
    sarvatobhadra_json,
    tithi_pravesha_json,
    dossier_json,
    masa_pravesha_json,
);

/// The entry points beside the chart request that read a caller's JSON,
/// in the selector's order after the chart's slots.
pub const OTHER_SLOTS: &[&str] = &[
    "ts_research",
    "ts_pakshi",
    "ts_rashifal",
    "ts_numerology_profile",
    "ts_naam_milan",
    "muhurta_json",
    "festivals_json",
    "ts_intl_render params",
    "settings_json",
    "layouts_json",
    "dashas_json",
];

/// A chart request for one instant at Kathmandu, every section absent.
fn chart_request(instant: &f64) -> TsChartRequest {
    // SAFETY: every field is an integer, a float or a pointer, so all
    // zeros is a request asking for nothing, as a C caller's `= {0}` is.
    let mut request: TsChartRequest = unsafe { core::mem::zeroed() };
    request.struct_size = size::<TsChartRequest>();
    request.instants = instant;
    request.instant_count = 1;
    request.latitude_deg = 27.7172;
    request.longitude_deg = 85.324;
    request.altitude_m = 1400.0;
    request.utc_offset_seconds = 20_700;
    request
}

/// An entry point that answers a string, its answer freed.
fn string_call(
    call: unsafe extern "C" fn(*const TsContext, *const c_char, *mut TsString) -> Status,
    text: &CString,
    at: &str,
) {
    let mut out = TsString::empty();
    // SAFETY: a live context, a NUL-terminated string and a valid slot.
    let status = unsafe { call(shared(), text.as_ptr(), &raw mut out) };
    answered(status, at);
    // SAFETY: a descriptor the library wrote, or the empty one.
    unsafe { ts_string_free(&raw mut out) };
}

/// Whether `text` is a record the shipped server refuses for its size.
fn beyond_bounds(text: &[u8]) -> bool {
    serde_json::from_slice::<serde_json::Value>(text)
        .is_ok_and(|record| Limits::default().check(&record).is_err())
}

/// The `json` target: a selector byte names a slot and the rest is its
/// text.
///
/// A record the shipped server would refuse for its size is skipped: the
/// SDK computes whatever it is asked, so two centuries of daily readings
/// are slow by request, not a hang, and bounding a caller is the host's
/// (`teistro_mcp::Limits`). Text that is not JSON still goes in, to reach
/// the readers' refusals.
pub fn json(data: &[u8]) {
    let Some((selector, rest)) = data.split_first() else {
        return;
    };
    if beyond_bounds(rest) {
        return;
    }
    let text = c_text(rest);
    let at = usize::from(*selector) % (CHART_SLOTS.len() + OTHER_SLOTS.len());
    if let Some(name) = CHART_SLOTS.get(at) {
        let instant = 2_460_000.5;
        let mut request = chart_request(&instant);
        set_slot(&mut request, at, text.as_ptr());
        let mut blob = TsBlob::empty();
        // SAFETY: a live context, a valid request and a valid slot.
        let status = unsafe { ts_chart_found(shared(), &raw const request, &raw mut blob) };
        answered(status, name);
        // SAFETY: a descriptor the library wrote, or the empty one.
        unsafe { ts_blob_free(&raw mut blob) };
        return;
    }
    match at - CHART_SLOTS.len() {
        0 => string_call(ts_research, &text, "ts_research"),
        1 => string_call(ts_pakshi, &text, "ts_pakshi"),
        2 => string_call(ts_rashifal, &text, "ts_rashifal"),
        3 => string_call(ts_numerology_profile, &text, "ts_numerology_profile"),
        4 => {
            let mut blob = TsBlob::empty();
            // SAFETY: a live context, a NUL-terminated string and a valid slot.
            let status = unsafe { ts_naam_milan(shared(), text.as_ptr(), &raw mut blob) };
            answered(status, "ts_naam_milan");
            // SAFETY: as above.
            unsafe { ts_blob_free(&raw mut blob) };
        }
        5 | 6 => panchanga(&text, at - CHART_SLOTS.len() == 6),
        7 => {
            let key = c_text(b"sdk.graha.name");
            let mut blob = TsBlob::empty();
            // SAFETY: a live context, NUL-terminated strings and a valid slot.
            let status =
                unsafe { ts_intl_render(shared(), key.as_ptr(), text.as_ptr(), &raw mut blob) };
            answered(status, "ts_intl_render");
            // SAFETY: as above.
            unsafe { ts_blob_free(&raw mut blob) };
        }
        slot => {
            if let Some(handle) = open(TsEphemeris::None, Some((slot - 8, &text))) {
                close(handle);
            }
        }
    }
}

/// A panchanga request for two days at Kathmandu with one JSON section.
fn panchanga(text: &CString, festivals: bool) {
    let request = TsPanchangaRequest {
        struct_size: size::<TsPanchangaRequest>(),
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
        muhurta_json: if festivals {
            ptr::null()
        } else {
            text.as_ptr()
        },
        festivals_json: if festivals {
            text.as_ptr()
        } else {
            ptr::null()
        },
    };
    let mut blob = TsBlob::empty();
    // SAFETY: a live context, a valid request and a valid slot.
    let status = unsafe { ts_panchanga_days(shared(), &raw const request, &raw mut blob) };
    answered(
        status,
        if festivals {
            "festivals_json"
        } else {
            "muhurta_json"
        },
    );
    // SAFETY: a descriptor the library wrote, or the empty one.
    unsafe { ts_blob_free(&raw mut blob) };
}

/// The `pack` target: the bytes as a caller's pack, loaded into a context
/// of its own, since a load changes what the context says.
pub fn pack(data: &[u8]) {
    let Some(handle) = open(TsEphemeris::None, None) else {
        return;
    };
    let mut loaded = TsIntlLoaded {
        struct_size: size::<TsIntlLoaded>(),
        entries: 0,
        replaced: 0,
        merged: 0,
        locale: ptr::null(),
        sha256: ptr::null(),
    };
    // SAFETY: a live context, a readable buffer of that length and a
    // valid slot.
    let status = unsafe { ts_intl_load_pack(handle, data.as_ptr(), data.len(), &raw mut loaded) };
    answered(status, "ts_intl_load_pack");
    close(handle);
}

/// The `key` target: the text as a key, an entity, a locale and a text to
/// transliterate.
pub fn key(data: &[u8]) {
    let text = c_text(data);
    let context = shared();
    let mut id = 0_u32;
    // SAFETY: a live context, a NUL-terminated string and a valid slot.
    answered(
        unsafe { ts_key_parse(context, text.as_ptr(), &raw mut id) },
        "ts_key_parse",
    );
    let mut has = 0_u8;
    // SAFETY: as above.
    answered(
        unsafe { ts_intl_has(context, text.as_ptr(), &raw mut has) },
        "ts_intl_has",
    );
    let mut lent = TsStr {
        data: ptr::null(),
        len: 0,
    };
    // SAFETY: as above; the answer is lent and not freed.
    answered(
        unsafe { ts_intl_entity(context, text.as_ptr(), &raw mut lent) },
        "ts_intl_entity",
    );
    let mut blob = TsBlob::empty();
    // SAFETY: as above, with no parameters.
    answered(
        unsafe { ts_intl_render(context, text.as_ptr(), ptr::null(), &raw mut blob) },
        "ts_intl_render",
    );
    // SAFETY: a descriptor the library wrote, or the empty one.
    unsafe { ts_blob_free(&raw mut blob) };
    let (from, to) = (c_text(b"Deva"), c_text(b"Latn"));
    // SAFETY: a live context, NUL-terminated strings and a valid slot.
    answered(
        unsafe {
            ts_intl_transliterate(
                context,
                text.as_ptr(),
                from.as_ptr(),
                to.as_ptr(),
                &raw mut lent,
            )
        },
        "ts_intl_transliterate",
    );
    // A locale changes what the context says, so it is tried on one of
    // its own.
    if let Some(handle) = open(TsEphemeris::None, None) {
        // SAFETY: a live context and a NUL-terminated string.
        answered(
            unsafe { ts_intl_set_locale(handle, text.as_ptr()) },
            "ts_intl_set_locale",
        );
        close(handle);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The slots are a list and the request's fields are the truth: every
    /// `*_json` field of `ts_chart_request` is a slot, and every slot a
    /// field, read from the source both ways.
    #[test]
    fn every_json_section_of_a_chart_request_is_a_slot() {
        let source = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../crates/ffi/src/chart_request.rs"
        ))
        .unwrap_or_default();
        let fields: Vec<&str> = source
            .lines()
            .filter_map(|line| line.trim().strip_prefix("pub "))
            .filter_map(|line| line.strip_suffix(": *const c_char,"))
            .filter(|name| name.ends_with("_json"))
            .collect();
        assert_eq!(fields, CHART_SLOTS);
    }

    /// Every slot answers a seed without a finding.
    #[test]
    fn every_slot_answers_its_seed() {
        for selector in 0..=u8::MAX {
            let mut input = vec![selector];
            input.extend_from_slice(b"{}");
            json(&input);
        }
        pack(b"not a pack");
        key(b"sdk.graha.name");
    }

    /// The fuzzer's first timeout, two centuries of weekly readings, is
    /// skipped as the server refuses it, and a year of them is not.
    #[test]
    fn a_record_past_the_servers_bounds_is_skipped() {
        let period = |first: u16, last: u16| {
            format!(
                r#"{{"periods": [{{"first": {{"year": {first}, "month": 10, "day": 4}}, "last": {{"year": {last}, "month": 10, "day": 3}}, "latitudeDeg": 27.7152, "longitudeDeg": 85.324}}], "baseline": "WEEKLY"}}"#
            )
        };
        assert!(beyond_bounds(period(1837, 2037).as_bytes()));
        assert!(!beyond_bounds(period(2026, 2027).as_bytes()));
        assert!(!beyond_bounds(b"{\"periods\": ["));
    }
}
