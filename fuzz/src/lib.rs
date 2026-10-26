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
use teistro_idl::blob::Reader;
use teistro_idl::model::{Api, BlobSchema, SectionKind};
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
    western_configurations_json,
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

/// Whether `text` is a record the shipped server refuses for its size,
/// read where the server reads it: a chart section under its own name in
/// the chart's record (`hits_json` is the record's `hits`, `sade_sati_json`
/// its `sadeSati`), where the server's bounds look for a window, and any
/// other slot's text as the whole record.
fn beyond_bounds(section: Option<&str>, text: &[u8]) -> bool {
    serde_json::from_slice::<serde_json::Value>(text).is_ok_and(|value| {
        let record = match section.and_then(|slot| slot.strip_suffix("_json")) {
            Some(name) => serde_json::json!({ camel(name): value }),
            None => value,
        };
        Limits::default().check(&record).is_err()
    })
}

/// A slot's name as the chart's record spells it: `sade_sati` is
/// `sadeSati`.
fn camel(name: &str) -> String {
    let mut words = name.split('_');
    let mut spelt = String::from(words.next().unwrap_or_default());
    for word in words {
        let mut letters = word.chars();
        spelt.extend(letters.next().map(|first| first.to_ascii_uppercase()));
        spelt.push_str(letters.as_str());
    }
    spelt
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
    let at = usize::from(*selector) % (CHART_SLOTS.len() + OTHER_SLOTS.len());
    // Bounded as the library reads it, up to the first NUL: the whole
    // input with bytes after a NUL is no JSON and would pass unbounded.
    let text = c_text(rest);
    if beyond_bounds(CHART_SLOTS.get(at).copied(), text.as_bytes()) {
        return;
    }
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
    let request = if festivals {
        panchanga_request(ptr::null(), text.as_ptr())
    } else {
        panchanga_request(text.as_ptr(), ptr::null())
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

/// Two days at Kathmandu with these sections, either of them null.
fn panchanga_request(
    muhurta_json: *const c_char,
    festivals_json: *const c_char,
) -> TsPanchangaRequest {
    TsPanchangaRequest {
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
        muhurta_json,
        festivals_json,
    }
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

/// The key the `mf2` target writes its message under, which no shipped
/// locale has, so the fuzzer's text is the only source it can render.
const MF2_KEY: &str = "sdk.fuzz.message";

/// What the `mf2` target renders every message with: one value of each
/// type the engine formats, read by the boundary's own parameter reader.
const MF2_PARAMS: &str = r#"{"n": 7, "x": -1.5, "s": "text", "l": [1, "a", {"$entity": "rashi.MESHA"}], "graha": {"$entity": "graha.SUN"}, "d": {"$date": {"calendar": "BIKRAM_SAMBAT", "year": 2083, "month": 5, "day": 21}}, "t": {"$time": {"hour": 6, "minute": 15, "second": 0}}, "dt": {"$datetime": {"date": {"calendar": "GREGORIAN", "year": 2026, "month": 9, "day": 6}, "time": {"hour": 23, "minute": 59, "second": 59}}}, "gh": {"$ghati": {"ghati": 12, "pala": 30, "vipala": 0}}, "at": {"$instant": 2460000.25}}"#;

/// The context the `mf2` target writes its message into, one a locale,
/// built once and never freed: the base locale, and Nepali with its own
/// digits and plural rules. They are not [`shared`], which every other
/// target reads and none changes.
fn messages(nepali: bool) -> *const TsContext {
    static CONTEXTS: [OnceLock<usize>; 2] = [OnceLock::new(), OnceLock::new()];
    let address = *CONTEXTS[usize::from(nepali)].get_or_init(|| {
        let handle = open(TsEphemeris::None, None).unwrap_or_else(|| panic!("a context opens"));
        if nepali {
            let tag = c_text(b"ne-Deva-NP");
            // SAFETY: a live context and a NUL-terminated string.
            let status = unsafe { ts_intl_set_locale(handle, tag.as_ptr()) };
            assert_eq!(status, Status::Ok, "the Nepali locale is shipped");
        }
        handle as usize
    });
    address as *const TsContext
}

/// The `mf2` target: the text as a message, set as an override in the
/// context's locale and rendered through `ts_intl_render`, so it reaches
/// the parser and the formatter as a pack's message would. A selector
/// byte picks the base locale or Nepali.
pub fn mf2(data: &[u8]) {
    let Some((selector, rest)) = data.split_first() else {
        return;
    };
    // A message reaches the engine as text: a pack holding bytes that are
    // not UTF-8 is refused before any message is read.
    let Ok(source) = core::str::from_utf8(rest) else {
        return;
    };
    let context = messages(selector % 2 == 1);
    // SAFETY: a context this crate opened and never frees.
    let held = unsafe { &*context };
    let tag = held.intl().locale().to_string();
    if held.intl_mut().set_override(&tag, MF2_KEY, source).is_err() {
        return;
    }
    let (key, params) = (c_text(MF2_KEY.as_bytes()), c_text(MF2_PARAMS.as_bytes()));
    let mut blob = TsBlob::empty();
    // SAFETY: a live context, NUL-terminated strings and a valid slot.
    let status = unsafe { ts_intl_render(context, key.as_ptr(), params.as_ptr(), &raw mut blob) };
    answered(status, "ts_intl_render");
    // SAFETY: a descriptor the library wrote, or the empty one.
    unsafe { ts_blob_free(&raw mut blob) };
    held.intl_mut().clear_override(&tag, MF2_KEY);
}

/// The blob schemas, read once from the description every binding's
/// decoder is generated from.
fn schemas() -> &'static [BlobSchema] {
    static API: OnceLock<Api> = OnceLock::new();
    &API.get_or_init(|| {
        serde_json::from_str(include_str!("../../idl/api.json"))
            .unwrap_or_else(|error| panic!("idl/api.json reads: {error}"))
    })
    .blobs
}

/// The `blob` target: the bytes as a result blob, read whole by the
/// reference reader under the schema its header names, or the first when
/// it names none: every section by its kind and every field of each.
pub fn blob(data: &[u8]) {
    let schemas = schemas();
    let named = data
        .get(16..20)
        .and_then(|word| word.try_into().ok())
        .map(u32::from_le_bytes);
    let Some(schema) = schemas
        .iter()
        .find(|schema| Some(schema.id) == named)
        .or_else(|| schemas.first())
    else {
        return;
    };
    let Ok(reader) = Reader::parse(data, schema) else {
        return;
    };
    for section in &schema.sections {
        let name = section.name.as_str();
        let _ = reader.count(name);
        match section.kind {
            SectionKind::Fixed => {
                let _ = reader.fixed(name);
                for field in &section.fields {
                    let _ = reader.field(name, &field.name);
                }
            }
            SectionKind::Columns => {
                for field in &section.fields {
                    let _ = reader.column(name, &field.name);
                }
            }
            SectionKind::Bytes => {
                let _ = reader.bytes(name);
                let _ = reader.text(name);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    /// The tests run on threads and a context is one caller's, so the two
    /// that write into [`messages`] take turns, as the fuzzer's one thread
    /// does.
    static MESSAGES: Mutex<()> = Mutex::new(());

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
        let _turn = MESSAGES
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for selector in 0..=u8::MAX {
            let mut input = vec![selector];
            input.extend_from_slice(b"{}");
            json(&input);
        }
        pack(b"not a pack");
        key(b"sdk.graha.name");
        for selector in [0, 1] {
            mf2(&[&[selector][..], b"{$graha} at {$n :number} on {$d :date}"].concat());
            mf2(&[
                &[selector][..],
                b".match {$n :number} one {{one}} * {{many}}",
            ]
            .concat());
            mf2(&[selector, b'{']);
        }
        blob(b"");
        blob(b"TSRB");
    }

    /// The bytes of a blob the library wrote, copied out, the blob freed.
    fn written(blob: &mut TsBlob) -> Vec<u8> {
        let bytes = if blob.data.is_null() {
            Vec::new()
        } else {
            // SAFETY: a blob the library wrote holds `len` bytes at `data`.
            unsafe { core::slice::from_raw_parts(blob.data, blob.len) }.to_vec()
        };
        // SAFETY: a descriptor the library wrote, freed once.
        unsafe { ts_blob_free(blob) };
        bytes
    }

    /// The reference reader reads every schema of blob the library
    /// writes, whole: every section the blob carries, by its kind. With
    /// `TEISTRO_FUZZ_SEEDS` naming a directory, the blobs are also written
    /// there, which is how `corpus/blob/` was seeded
    /// (`docs/05-testing/03-fuzzing.md` §3).
    #[test]
    fn every_blob_the_library_writes_reads_whole() {
        let _turn = MESSAGES
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut blobs: Vec<(&str, Vec<u8>)> = Vec::new();
        let mut out = TsBlob::empty();
        let instant = 2_460_000.5;
        let request = chart_request(&instant);
        // SAFETY: a live context, a valid request and a valid slot.
        let status = unsafe { ts_chart_found(shared(), &raw const request, &raw mut out) };
        assert_eq!(status, Status::Ok, "ts_chart_found");
        blobs.push(("charts", written(&mut out)));
        let request = panchanga_request(ptr::null(), ptr::null());
        // SAFETY: as above.
        let status = unsafe { ts_panchanga_days(shared(), &raw const request, &raw mut out) };
        assert_eq!(status, Status::Ok, "ts_panchanga_days");
        blobs.push(("panchanga", written(&mut out)));
        let names = c_text("{\"bride\": \"सीता\", \"groom\": \"राम\"}".as_bytes());
        // SAFETY: a live context, a NUL-terminated string and a valid slot.
        let status = unsafe { ts_naam_milan(shared(), names.as_ptr(), &raw mut out) };
        assert_eq!(status, Status::Ok, "ts_naam_milan");
        blobs.push(("naam", written(&mut out)));
        let (key, params) = (c_text(MF2_KEY.as_bytes()), c_text(MF2_PARAMS.as_bytes()));
        // SAFETY: a context this crate opened and never frees.
        let held = unsafe { &*messages(true) };
        let tag = held.intl().locale().to_string();
        let set = held
            .intl_mut()
            .set_override(&tag, MF2_KEY, "{$graha} {$n :number} {$d :date}");
        assert!(set.is_ok(), "{set:?}");
        // SAFETY: a live context, NUL-terminated strings and a valid slot.
        let status =
            unsafe { ts_intl_render(messages(true), key.as_ptr(), params.as_ptr(), &raw mut out) };
        assert_eq!(status, Status::Ok, "ts_intl_render");
        blobs.push(("intl_render", written(&mut out)));
        held.intl_mut().clear_override(&tag, MF2_KEY);

        let seeds = std::env::var_os("TEISTRO_FUZZ_SEEDS");
        for (name, bytes) in &blobs {
            let schema = schemas()
                .iter()
                .find(|schema| schema.name == *name)
                .unwrap_or_else(|| panic!("a `{name}` schema"));
            let reader =
                Reader::parse(bytes, schema).unwrap_or_else(|error| panic!("{name}: {error}"));
            let mut carried = 0;
            for section in &schema.sections {
                let section_name = section.name.as_str();
                if reader.entry(section_name).is_none() {
                    continue;
                }
                carried += 1;
                let read = match section.kind {
                    SectionKind::Fixed => reader.fixed(section_name).map(drop),
                    SectionKind::Columns => section
                        .fields
                        .iter()
                        .try_for_each(|field| reader.column(section_name, &field.name).map(drop)),
                    SectionKind::Bytes => reader.bytes(section_name).map(drop),
                };
                assert!(read.is_ok(), "{name}.{section_name}: {read:?}");
            }
            assert!(carried > 0, "{name} carries no section");
            blob(bytes);
            if let Some(dir) = &seeds {
                let path = std::path::Path::new(dir).join(format!("{name}.tsrb"));
                std::fs::write(&path, bytes).unwrap_or_else(|error| panic!("{error}"));
            }
        }
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
        assert!(beyond_bounds(None, period(1837, 2037).as_bytes()));
        assert!(!beyond_bounds(None, period(2026, 2027).as_bytes()));
        assert!(!beyond_bounds(None, b"{\"periods\": ["));
    }

    /// The verify run's timeout on 2026-10-26: Saturn's spells over
    /// 670,000 years in `sade_sati_json`, followed by a NUL and more
    /// bytes. The whole input is no JSON, so bounding it skipped nothing,
    /// while the library read the window up to the NUL; bounded as the
    /// library reads it, it is skipped.
    #[test]
    fn a_window_before_a_nul_is_bounded_as_the_library_reads_it() {
        let input = b"{\"from\":2460676.5,\"to\":246434429.0,\"reckoning\":\"SIGN\"}\0\0\0<";
        let slot = Some("sade_sati_json");
        assert!(!beyond_bounds(slot, input));
        assert!(beyond_bounds(slot, c_text(input).as_bytes()));
    }

    /// The nightly minute's first timeout (2026-10-23): a transit window
    /// of 9,750 years in `hits_json`, which the server refuses as the
    /// chart record's `hits` and the harness read as a bare record. A
    /// year's window is not skipped, and the skip is the server's own:
    /// the same window under another section's name is not a window.
    #[test]
    fn a_transit_window_past_the_servers_bounds_is_skipped() {
        let window =
            |to: f64| format!(r#"{{"from": 2460676.5, "to": {to}, "grahas": ["SATURN"]}}"#);
        assert!(beyond_bounds(
            Some("hits_json"),
            window(6_021_641.5).as_bytes()
        ));
        assert!(!beyond_bounds(
            Some("hits_json"),
            window(2_461_041.5).as_bytes()
        ));
        assert!(!beyond_bounds(
            Some("gochar_json"),
            window(6_021_641.5).as_bytes()
        ));
    }

    /// Verify's smoke run on `8557e728` timed out on 670,000 years of
    /// Saturn's spells in `sade_sati_json`: the search runs on to the
    /// ephemeris's edge, six centuries of Saturn, which the server now
    /// bounds in years as the chart record's `sadeSati`.
    #[test]
    fn a_window_of_spells_past_the_servers_bounds_is_skipped() {
        let window = |to: f64| format!(r#"{{"from": 2460676.5, "to": {to}, "reckoning": "SIGN"}}"#);
        assert_eq!(camel("sade_sati"), "sadeSati");
        assert_eq!(camel("hits"), "hits");
        assert!(beyond_bounds(
            Some("sade_sati_json"),
            window(246_434_429.0).as_bytes()
        ));
        assert!(!beyond_bounds(
            Some("sade_sati_json"),
            window(2_490_000.5).as_bytes()
        ));
    }
}
