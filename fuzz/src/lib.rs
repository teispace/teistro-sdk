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
use teistro_ffi::chart::{TsChartRequest, ts_chart_found, ts_chart_layout_row};
use teistro_ffi::context::{
    TsContext, TsContextOptions, TsEphemeris, TsError, ts_context_free, ts_context_new,
    ts_error_free,
};
use teistro_ffi::ephemeris::ts_ephemeris_call;
use teistro_ffi::intl::{
    TsIntlLoaded, ts_intl_entity, ts_intl_has, ts_intl_load_pack, ts_intl_render,
    ts_intl_set_locale, ts_intl_transliterate,
};
use teistro_ffi::key::ts_key_parse;
use teistro_ffi::naam::ts_naam_milan;
use teistro_ffi::numerology::ts_numerology_profile;
use teistro_ffi::pakshi::ts_pakshi;
use teistro_ffi::panchanga::{
    TsPanchangaRequest, ts_almanac_ashauch, ts_almanac_disagreements, ts_almanac_festival_calendar,
    ts_almanac_first_year, ts_almanac_published_reasons, ts_almanac_sankalpa, ts_panchanga_days,
};
use teistro_ffi::published::ts_published_verify;
use teistro_ffi::rashifal::ts_rashifal;
use teistro_ffi::research::ts_research;
use teistro_ffi::string::{TsStr, TsString, ts_string_free};
use teistro_idl::blob::Reader;
use teistro_idl::model::{Api, BlobSchema, SectionKind};
use teistro_mcp::{Engine, Limits, Server};

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

/// The second reckoning a comparison is asked against: the Surya
/// Siddhanta's own astronomy, built once and never freed, as `shared` is.
fn other() -> *const TsContext {
    static CONTEXT: OnceLock<usize> = OnceLock::new();
    let address = *CONTEXT.get_or_init(|| {
        open(TsEphemeris::SuryaSiddhanta, None).map_or(0, |handle| handle as usize)
    });
    assert_ne!(address, 0, "the Surya Siddhanta context opens");
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
    "ts_almanac_sankalpa",
    "ts_almanac_ashauch",
    "ts_almanac_first_year",
    "ts_almanac_festival_calendar",
    "ts_almanac_disagreements",
    "ts_almanac_published_reasons",
    "ts_chart_layout_row key",
    "ts_ephemeris_call",
];

/// Whether a seed of the `json` target is to be answered, or only not
/// answered `INTERNAL`: a fuzzer's finding kept so it is never found again
/// may well be refused.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Seed {
    /// A well-formed section its slot answers.
    Answered,
    /// A finding kept as a regression: any answer but `INTERNAL`.
    Regression,
    /// A well-formed request its slot refuses by design, to reach the
    /// reader before the refusal: any answer but `OK` or `INTERNAL`.
    Refused,
}

/// The `json` target's seeds, each by the name of the slot it fills. The
/// selector byte is counted from the name when the corpus is written, so
/// a slot added anywhere moves no seed: they were kept by position once,
/// and five chart sections added before the other entry points sent every
/// seed after them to the wrong slot. `fuzz/corpus/json` is this table
/// written out (`the_json_corpus_is_the_seed_table`).
pub const JSON_SEEDS: &[(&str, Seed, &str)] = &[
    ("theme_json", Seed::Answered, r#"{"extends":"DARK"}"#),
    ("rules_json", Seed::Answered, r#"{"shipped":["NABHASAS"]}"#),
    ("interpret_json", Seed::Answered, r#"{"placements":true}"#),
    (
        "varsha_json",
        Seed::Answered,
        r#"{"reading":"SIDEREAL","through":12}"#,
    ),
    (
        "gochar_json",
        Seed::Answered,
        r#"{"instants":[2460676.5],"from":"MOON"}"#,
    ),
    (
        "hits_json",
        Seed::Answered,
        r#"{"from":2460676.5,"to":2461041.5,"grahas":["SATURN"]}"#,
    ),
    (
        "hits_json",
        Seed::Regression,
        r#"{"from":2460676.5,"to":6021641.5,"grahas":["SATURN"]}"#,
    ),
    (
        "sade_sati_json",
        Seed::Answered,
        r#"{"from":2460676.5,"to":2464329.0,"reckoning":"SIGN"}"#,
    ),
    (
        "sade_sati_json",
        Seed::Regression,
        r#"{"from":2460676.5,"to":246434429.0,"reckoning":"SIGN"}"#,
    ),
    (
        "kp_json",
        Seed::Answered,
        r#"{"number":74,"anyAyanamsha":true}"#,
    ),
    (
        "dignities_json",
        Seed::Answered,
        r#"{"sectRule":"HORIZON","rules":{"terms":"EGYPTIAN"}}"#,
    ),
    (
        "fortitudes_json",
        Seed::Answered,
        r#"{"rules":{"partile":{"WITHIN":{"orbDeg":1}}},"scores":{"regulus":5}}"#,
    ),
    (
        "lots_json",
        Seed::Answered,
        r#"{"fortune":"REVERSED_WHILE_MOON_UP"}"#,
    ),
    (
        "considerations_json",
        Seed::Answered,
        r#"{"moonLateFromDeg":25}"#,
    ),
    ("perfection_json", Seed::Answered, r#"{"house":7}"#),
    ("progressions_json", Seed::Answered, r#"{"at":2460676.5}"#),
    (
        "progressions_json",
        Seed::Regression,
        r#"{"at":12444444444444444444444444444444301}"#,
    ),
    (
        "western_aspects_json",
        Seed::Answered,
        r#"{"aspects":["TRINE","SQUARE"]}"#,
    ),
    (
        "synastry_json",
        Seed::Answered,
        r#"{"partner":{"instant":2403113.4993,"place":{"latitude":51.5058,"longitude":-0.1878,"altitude":0}}}"#,
    ),
    ("parallels_json", Seed::Answered, r#"{"orbDeg":1}"#),
    (
        "antiscia_json",
        Seed::Answered,
        r#"{"orbs":{"model":"LEO"}}"#,
    ),
    ("midpoints_json", Seed::Answered, r#"{"orbDeg":1}"#),
    (
        "western_houses_json",
        Seed::Answered,
        r#"{"system":"KOCH"}"#,
    ),
    ("harmonic_json", Seed::Answered, r#"{"number":9}"#),
    (
        "matching_json",
        Seed::Answered,
        r#"{"partner":{"instant":2447892.5,"place":{"latitude":27.7172,"longitude":85.324,"altitude":1400}},"partnerRole":"BRIDE"}"#,
    ),
    (
        "prashna_json",
        Seed::Answered,
        r#"{"question":{"house":7},"rules":{"mook":"MOON_HOUSE"}}"#,
    ),
    (
        "remedies_json",
        Seed::Answered,
        r#"{"at":2460676.5,"rules":{"shanti":{"rik":"YAJNAVALKYA"}}}"#,
    ),
    (
        "rectification_json",
        Seed::Answered,
        r#"{"purify":{"minutes":30},"circumstance":{"facts":{"fatherPresent":false}}}"#,
    ),
    (
        "lalkitab_json",
        Seed::Answered,
        r#"{"cycle":{"planet":"VENUS","year":17},"year":30}"#,
    ),
    (
        "sarvatobhadra_json",
        Seed::Answered,
        r#"{"instants":[2460001.5]}"#,
    ),
    ("tithi_pravesha_json", Seed::Answered, r#"{"through":3}"#),
    (
        "dossier_json",
        Seed::Answered,
        r#"{"sections":["CHART","GRAHAS","SHADBALA"]}"#,
    ),
    ("masa_pravesha_json", Seed::Answered, r#"{"year":3}"#),
    ("western_configurations_json", Seed::Answered, r#"{}"#),
    (
        "ts_research",
        Seed::Answered,
        r#"{"study": "COMPARE", "rules": {"shipped": ["YOGAS"]}, "births": [{"instant": 2447995.4895833335, "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700}, {"instant": 2447995.4895833335, "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700}, {"instant": 2447995.4895833335, "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700}, {"instant": 2447995.4895833335, "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700}], "design": {"groups": [0, 1, 0, 1]}, "test": {"seed": 5, "permutations": 99, "contrast": {"kind": "CASE_VS_REST", "case": 1}}}"#,
    ),
    (
        "ts_pakshi",
        Seed::Answered,
        r#"{"first": {"year": 1984, "month": 10, "day": 31}, "latitudeDeg": 13, "longitudeDeg": 80, "utcOffsetSeconds": 19800, "native": {"nakshatra": "BHARANI", "paksha": "SHUKLA"}}"#,
    ),
    (
        "ts_rashifal",
        Seed::Answered,
        r#"{"periods": [{"first": {"year": 2026, "month": 10, "day": 4}, "last": {"year": 2026, "month": 10, "day": 10}, "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1400, "utcOffsetSeconds": 20700}], "baseline": "WEEKLY"}"#,
    ),
    (
        "ts_rashifal",
        Seed::Regression,
        r#"{"periods": [{"first": {"year": 1837, "month": 10, "day": 4}, "last": {"year": 2037, "month": 10, "day": 10}, "latitudeDeg": 27.7152, "longitudeDeg": 85.324, "altitudeM": 1400, "utcOffsetSeconds": 20700}], "baseline": "WEEKLY"}"#,
    ),
    (
        "ts_numerology_profile",
        Seed::Answered,
        r#"{"name": "Henry Elder", "date": {"year": 1872, "month": 1, "day": 17}}"#,
    ),
    (
        "ts_naam_milan",
        Seed::Answered,
        r#"{"bride": "सीता", "groom": "राम"}"#,
    ),
    (
        "muhurta_json",
        Seed::Answered,
        r#"{"rules":"RAMAN_MARRIAGE"}"#,
    ),
    (
        "festivals_json",
        Seed::Answered,
        r#"{"rules":"DHARMASINDHU"}"#,
    ),
    (
        "ts_intl_render params",
        Seed::Answered,
        r#"{"graha": "graha.SUN"}"#,
    ),
    (
        "settings_json",
        Seed::Answered,
        r#"{"frame": {"zodiac": "SIDEREAL"}}"#,
    ),
    ("layouts_json", Seed::Answered, r#"[]"#),
    (
        "dashas_json",
        Seed::Answered,
        r#"[{"kernel":"UDU","key":"ACME_SAPTAKA","lords":[{"graha":"SUN","years":10},{"graha":"MOON","years":10},{"graha":"MARS","years":10},{"graha":"MERCURY","years":10},{"graha":"JUPITER","years":10},{"graha":"VENUS","years":10},{"graha":"SATURN","years":10}],"reference":"KRITTIKA"}]"#,
    ),
    (
        "ts_almanac_sankalpa",
        Seed::Answered,
        r#"{"at":2461055.59375,"latitudeDeg":27.7172,"longitudeDeg":85.324,"altitudeM":1400,"utcOffsetSeconds":20700,"ayana":"TROPICAL"}"#,
    ),
    (
        "ts_almanac_ashauch",
        Seed::Answered,
        r#"{"latitudeDeg":27.7172,"longitudeDeg":85.324,"altitudeM":1337,"utcOffsetSeconds":20700,"request":{"key":"grandfather","relation":"SAPINDA","event":{"event":"DEATH","at":2461051.677083}}}"#,
    ),
    (
        "ts_almanac_first_year",
        Seed::Answered,
        r#"{"latitudeDeg":27.7172,"longitudeDeg":85.324,"altitudeM":1337,"utcOffsetSeconds":20700,"request":{"key":"grandfather","at":2461051.677083,"firstAbdika":"ELEVENTH_DAY","firstYearAdhika":"NIJA"}}"#,
    ),
    (
        "ts_almanac_festival_calendar",
        Seed::Answered,
        r#"{"first":{"year":2025,"month":10,"day":1},"last":{"year":2025,"month":10,"day":3},"latitudeDeg":27.7172,"longitudeDeg":85.324,"altitudeM":1400,"utcOffsetSeconds":20700,"festivals":{"rules":"NEPAL"},"stampUnixSeconds":1767225600,"name":"Nepal","refreshMinutes":0}"#,
    ),
    (
        "ts_almanac_disagreements",
        Seed::Answered,
        r#"{"first":{"year":2025,"month":11,"day":1},"last":{"year":2025,"month":11,"day":2},"latitudeDeg":27.7172,"longitudeDeg":85.324,"altitudeM":1400,"utcOffsetSeconds":20700,"festivals":{"rules":"NEPAL"}}"#,
    ),
    (
        "ts_almanac_published_reasons",
        Seed::Answered,
        r#"{"first":{"year":2026,"month":10,"day":10},"last":{"year":2026,"month":10,"day":12},"latitudeDeg":28.2096,"longitudeDeg":83.9856,"altitudeM":827,"utcOffsetSeconds":20700,"festivals":{"rules":"NEPAL","published":[{"format":"teistro.published/1","publisher":"a patro printed in Nepal","id":"np-2083","version":3,"entries":[{"action":"SET","rule":"NAVARATRA_ARAMBHA","day":{"calendar":"GREGORIAN","year":2026,"month":10,"day":11},"source":"p. 41"}]}]},"places":[{"name":"Kathmandu","latitudeDeg":27.7172,"longitudeDeg":85.324,"altitudeM":1400,"utcOffsetSeconds":20700}]}"#,
    ),
    (
        "ts_chart_layout_row key",
        Seed::Answered,
        "chart_layout.SOUTH_INDIAN",
    ),
    ("ts_chart_layout_row key", Seed::Answered, "NORTH_INDIAN"),
    // An engine's own operations are an adapter's, so the built-in
    // ephemeris refuses them once it has read the call.
    (
        "ts_ephemeris_call",
        Seed::Refused,
        "tp_sum\n{\"values\":[1,2.5]}",
    ),
];

/// The corpus file of one seed: its selector byte, counted from the slot's
/// name, then its text; `None` for a name that is no slot.
#[cfg(test)]
fn seed_file(slot: &str, text: &str) -> Option<Vec<u8>> {
    let at = CHART_SLOTS
        .iter()
        .chain(OTHER_SLOTS)
        .position(|name| *name == slot)?;
    let mut bytes = vec![u8::try_from(at).ok()?];
    bytes.extend_from_slice(text.as_bytes());
    Some(bytes)
}

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
) -> Status {
    let mut out = TsString::empty();
    // SAFETY: a live context, a NUL-terminated string and a valid slot.
    let status = unsafe { call(shared(), text.as_ptr(), &raw mut out) };
    // SAFETY: a descriptor the library wrote, or the empty one.
    unsafe { ts_string_free(&raw mut out) };
    status
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
    answered(answer(at, &text), slot_name(at));
}

/// The name of the slot the selector counts to `at`.
fn slot_name(at: usize) -> &'static str {
    CHART_SLOTS
        .iter()
        .chain(OTHER_SLOTS)
        .nth(at)
        .copied()
        .unwrap_or("a slot past the end")
}

/// What the entry point behind slot `at` answers `text`.
fn answer(at: usize, text: &CString) -> Status {
    if CHART_SLOTS.get(at).is_some() {
        let instant = 2_460_000.5;
        let mut request = chart_request(&instant);
        set_slot(&mut request, at, text.as_ptr());
        let mut blob = TsBlob::empty();
        // SAFETY: a live context, a valid request and a valid slot.
        let status = unsafe { ts_chart_found(shared(), &raw const request, &raw mut blob) };
        // SAFETY: a descriptor the library wrote, or the empty one.
        unsafe { ts_blob_free(&raw mut blob) };
        return status;
    }
    match slot_name(at) {
        "ts_research" => string_call(ts_research, text),
        "ts_pakshi" => string_call(ts_pakshi, text),
        "ts_rashifal" => string_call(ts_rashifal, text),
        "ts_numerology_profile" => string_call(ts_numerology_profile, text),
        "ts_naam_milan" => {
            let mut blob = TsBlob::empty();
            // SAFETY: a live context, a NUL-terminated string and a valid slot.
            let status = unsafe { ts_naam_milan(shared(), text.as_ptr(), &raw mut blob) };
            // SAFETY: as above.
            unsafe { ts_blob_free(&raw mut blob) };
            status
        }
        "muhurta_json" => panchanga(text, false),
        "festivals_json" => panchanga(text, true),
        "ts_intl_render params" => {
            let key = c_text(b"sdk.graha.name");
            let mut blob = TsBlob::empty();
            // SAFETY: a live context, NUL-terminated strings and a valid slot.
            let status =
                unsafe { ts_intl_render(shared(), key.as_ptr(), text.as_ptr(), &raw mut blob) };
            // SAFETY: as above.
            unsafe { ts_blob_free(&raw mut blob) };
            status
        }
        "settings_json" => option(0, text),
        "layouts_json" => option(1, text),
        "dashas_json" => option(2, text),
        "ts_almanac_sankalpa" => string_call(ts_almanac_sankalpa, text),
        "ts_almanac_ashauch" => string_call(ts_almanac_ashauch, text),
        "ts_almanac_first_year" => string_call(ts_almanac_first_year, text),
        "ts_almanac_festival_calendar" => string_call(ts_almanac_festival_calendar, text),
        "ts_almanac_disagreements" => paired_call(ts_almanac_disagreements, text),
        "ts_almanac_published_reasons" => paired_call(ts_almanac_published_reasons, text),
        "ts_chart_layout_row key" => string_call(ts_chart_layout_row, text),
        "ts_ephemeris_call" => ephemeris_call(text),
        _ => Status::Ok,
    }
}

/// A context built with one JSON option, `Ok` when it opens.
fn option(slot: usize, text: &CString) -> Status {
    open(TsEphemeris::None, Some((slot, text))).map_or(Status::InvalidArg, |handle| {
        close(handle);
        Status::Ok
    })
}

/// An entry point comparing the shared context with `other`'s reckoning.
fn paired_call(
    call: unsafe extern "C" fn(
        *const TsContext,
        *const TsContext,
        *const c_char,
        *mut TsString,
    ) -> Status,
    text: &CString,
) -> Status {
    let mut out = TsString::empty();
    // SAFETY: two live contexts, a NUL-terminated string and a valid slot.
    let status = unsafe { call(shared(), other(), text.as_ptr(), &raw mut out) };
    // SAFETY: a descriptor the library wrote, or the empty one.
    unsafe { ts_string_free(&raw mut out) };
    status
}

/// An engine's own function: the text's first line names it and the rest
/// are its arguments, as a caller passes the two.
fn ephemeris_call(text: &CString) -> Status {
    let mut lines = text.as_bytes().splitn(2, |byte| *byte == b'\n');
    let function = c_text(lines.next().unwrap_or_default());
    let arguments = c_text(lines.next().unwrap_or_default());
    let mut out = TsString::empty();
    // SAFETY: a live context, NUL-terminated strings and a valid slot.
    let status = unsafe {
        ts_ephemeris_call(
            shared(),
            function.as_ptr(),
            arguments.as_ptr(),
            &raw mut out,
        )
    };
    // SAFETY: a descriptor the library wrote, or the empty one.
    unsafe { ts_string_free(&raw mut out) };
    status
}

/// A panchanga request for two days at Kathmandu with one JSON section.
fn panchanga(text: &CString, festivals: bool) -> Status {
    let request = if festivals {
        panchanga_request(ptr::null(), text.as_ptr())
    } else {
        panchanga_request(text.as_ptr(), ptr::null())
    };
    let mut blob = TsBlob::empty();
    // SAFETY: a live context, a valid request and a valid slot.
    let status = unsafe { ts_panchanga_days(shared(), &raw const request, &raw mut blob) };
    // SAFETY: a descriptor the library wrote, or the empty one.
    unsafe { ts_blob_free(&raw mut blob) };
    status
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
    let data = &checksummed(data);
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

/// `data` with its pack checksum made right, where it has a header to hold
/// one. A mutation almost never keeps a CRC-32 whole, so without this the
/// target would test the header and nothing behind it; a checksum is no
/// authentication, and every reader behind it must hold against a pack a
/// hostile writer checksummed. The layout is `Pack::parse`'s (format 3):
/// magic, version and flags (8 bytes), locale and namespace (each a `u16`
/// length and its bytes), the entry count and arena length, then the CRC
/// of every other byte of the file.
fn checksummed(data: &[u8]) -> Vec<u8> {
    let mut bytes = data.to_vec();
    let u16_at = |at: usize| -> Option<usize> {
        Some(usize::from(u16::from_le_bytes(
            data.get(at..at + 2)?.try_into().ok()?,
        )))
    };
    let crc_at = (|| {
        let after_locale = 10 + u16_at(8)?;
        let after_namespace = after_locale + 2 + u16_at(after_locale)?;
        Some(after_namespace + 8)
    })();
    if let Some(at) = crc_at
        && let (Some(before), Some(after)) = (data.get(..at), data.get(at + 4..))
        && let Some(slot) = bytes.get_mut(at..at + 4)
    {
        let mut crc = crc32fast::Hasher::new();
        crc.update(before);
        crc.update(after);
        slot.copy_from_slice(&crc.finalize().to_le_bytes());
    }
    bytes
}

/// The `published` target: the first byte is the newest version the
/// caller has applied, and the rest a publisher's pack, signed here so
/// the fuzzer reaches the reader behind the signature. The key is made
/// from a fixed seed, as the SDK's own tests make theirs; a signature
/// is the publisher's proof, not the reader's defence, so a hostile
/// publisher's bytes are what this fuzzes.
pub fn published(data: &[u8]) {
    if let Some(status) = signed_and_verified(data) {
        answered(status, "ts_published_verify");
    }
}

/// What `ts_published_verify` answers the `published` target's input,
/// `None` for no input.
fn signed_and_verified(data: &[u8]) -> Option<Status> {
    use ed25519_dalek::Signer as _;
    let (at_least, pack) = data.split_first()?;
    let publisher = ed25519_dalek::SigningKey::from_bytes(&[7; 32]);
    let signature = publisher.sign(pack).to_bytes();
    let key = publisher.verifying_key().to_bytes();
    let mut out = TsString::empty();
    // SAFETY: a live context, byte ranges valid for their lengths and a
    // valid slot.
    let status = unsafe {
        ts_published_verify(
            shared(),
            pack.as_ptr(),
            pack.len(),
            signature.as_ptr(),
            signature.len(),
            key.as_ptr(),
            key.len(),
            u32::from(*at_least),
            &raw mut out,
        )
    };
    // SAFETY: a descriptor the library wrote, or the empty one.
    unsafe { ts_string_free(&raw mut out) };
    Some(status)
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

/// The `mcp` target: the bytes as one message a transport read, handed to
/// a fresh server over the built-in ephemeris, so no message leans on
/// what an earlier input left (an `initialize`, a cancellation). It runs
/// no task in the background, so a call computes while the fuzzer
/// watches. The server bounds a request itself (`Limits`), so nothing is
/// skipped.
pub fn mcp(data: &[u8]) {
    let mut server = Server::new(Engine::Builtin).with_task_patience(None);
    if let Some(reply) = server.handle_bytes(data) {
        held(&reply);
    }
}

/// The finding rule for a reply: it is one JSON-RPC response, a result
/// or an error, and neither says the server failed (`-32603`, or a tool
/// refusal whose status is `INTERNAL`).
fn held(reply: &str) {
    let reply: serde_json::Value = serde_json::from_str(reply)
        .unwrap_or_else(|error| panic!("a reply that is no JSON ({error}): {reply}"));
    assert_eq!(reply["jsonrpc"], "2.0", "{reply}");
    let (result, error) = (reply.get("result"), reply.get("error"));
    assert!(result.is_some() != error.is_some(), "{reply}");
    assert_ne!(reply["error"]["code"], -32_603, "{reply}");
    assert_ne!(
        reply["result"]["structuredContent"]["status"], "INTERNAL",
        "{reply}"
    );
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
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

    /// Every seed is answered in the slot it names, or, for a kept
    /// finding, at least not answered `INTERNAL`: a seed counted to the
    /// wrong slot is refused there and starts the fuzzer nowhere.
    #[test]
    fn every_json_seed_is_answered_in_its_slot() {
        let _turn = MESSAGES
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut wrong = Vec::new();
        for &(slot, seed, text) in JSON_SEEDS {
            let at = usize::from(seed_file(slot, text).unwrap()[0]);
            // What the target passes over, as `json` does.
            if beyond_bounds(CHART_SLOTS.get(at).copied(), text.as_bytes()) {
                continue;
            }
            let status = answer(at, &c_text(text.as_bytes()));
            let fine = match seed {
                Seed::Answered => status == Status::Ok,
                Seed::Regression => status != Status::Internal,
                Seed::Refused => !matches!(status, Status::Ok | Status::Internal),
            };
            if !fine {
                wrong.push(format!("{slot} answered {status:?} to {text}"));
            }
        }
        assert!(wrong.is_empty(), "{wrong:#?}");
    }

    /// `fuzz/corpus/<target>` is `wanted` written out: set
    /// `TEISTRO_FUZZ_WRITE_SEEDS=1` to write it after changing a table.
    fn corpus_is(target: &str, wanted: &BTreeMap<String, Vec<u8>>) {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("corpus")
            .join(target);
        if std::env::var_os("TEISTRO_FUZZ_WRITE_SEEDS").is_some() {
            std::fs::create_dir_all(&dir).unwrap();
            for entry in std::fs::read_dir(&dir).unwrap() {
                std::fs::remove_file(entry.unwrap().path()).unwrap();
            }
            for (name, bytes) in wanted {
                std::fs::write(dir.join(name), bytes).unwrap();
            }
        }
        let found: BTreeMap<String, Vec<u8>> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|entry| {
                let path = entry.unwrap().path();
                (
                    path.file_name().unwrap().to_string_lossy().into_owned(),
                    std::fs::read(&path).unwrap(),
                )
            })
            .collect();
        assert!(
            &found == wanted,
            "fuzz/corpus/{target} is not its seed table written out; run the test with TEISTRO_FUZZ_WRITE_SEEDS=1"
        );
    }

    /// The committed json corpus is `JSON_SEEDS` written out.
    #[test]
    fn the_json_corpus_is_the_seed_table() {
        let wanted = JSON_SEEDS
            .iter()
            .enumerate()
            .map(|(n, &(slot, _, text))| {
                (
                    format!("{slot}-{n:02}").replace(' ', "-"),
                    seed_file(slot, text).unwrap(),
                )
            })
            .collect();
        corpus_is("json", &wanted);
    }

    /// The `published` target's seeds: the newest version applied, then a
    /// publisher's pack, each answered as listed.
    const PUBLISHED_SEEDS: &[(&str, u8, bool, &str)] = &[
        (
            "one-entry",
            0,
            true,
            r#"{"format":"teistro.published/1","publisher":"a patro printed in Nepal","id":"np-2083","version":3,"entries":[{"action":"SET","rule":"NAVARATRA_ARAMBHA","day":{"calendar":"GREGORIAN","year":2026,"month":10,"day":11},"source":"p. 41"}]}"#,
        ),
        (
            "no-entries",
            0,
            true,
            r#"{"format":"teistro.published/1","publisher":"a patro printed in Nepal","id":"np-2083","version":1,"entries":[]}"#,
        ),
        (
            "rolled-back",
            4,
            false,
            r#"{"format":"teistro.published/1","publisher":"a patro printed in Nepal","id":"np-2083","version":3,"entries":[]}"#,
        ),
    ];

    fn published_seeds() -> BTreeMap<String, Vec<u8>> {
        PUBLISHED_SEEDS
            .iter()
            .map(|&(name, at_least, _, pack)| {
                let mut bytes = vec![at_least];
                bytes.extend_from_slice(pack.as_bytes());
                (name.to_owned(), bytes)
            })
            .collect()
    }

    /// Each seed reads as listed: a set, or a refusal by its field.
    #[test]
    fn every_published_seed_is_answered_as_listed() {
        let _turn = MESSAGES
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for (&(name, _, answered, _), (_, data)) in PUBLISHED_SEEDS.iter().zip(published_seeds()) {
            let status = signed_and_verified(&data).unwrap();
            assert_ne!(status, Status::Internal, "{name}");
            assert_eq!(status == Status::Ok, answered, "{name}: {status:?}");
        }
    }

    /// The committed published corpus is the seed table written out.
    #[test]
    fn the_published_corpus_is_the_seed_table() {
        corpus_is("published", &published_seeds());
    }

    /// The harness seals a pack where the library's writer does: every
    /// committed pack seed, a real file, is its own checksummed form, so a
    /// format that moves its checksum fails here rather than leaving the
    /// target to test the header alone.
    #[test]
    fn a_pack_the_library_wrote_is_its_own_checksummed_form() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("corpus/pack");
        let mut packs = 0;
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            let bytes = std::fs::read(&path).unwrap();
            if bytes.starts_with(b"TPK1") {
                packs += 1;
                assert!(checksummed(&bytes) == bytes, "{}", path.display());
            }
        }
        assert!(packs > 0, "no pack seed");
    }

    /// The weekly long run's matrix names every target the manifest
    /// builds, so a target added is run for an hour too.
    #[test]
    fn the_long_run_fuzzes_every_target() {
        let manifest = include_str!("../Cargo.toml");
        let mut built: Vec<&str> = manifest
            .split("[[bin]]")
            .skip(1)
            .filter_map(|bin| bin.lines().find_map(|line| line.strip_prefix("name = ")))
            .map(|name| name.trim_matches('"'))
            .collect();
        let workflow = include_str!("../../.github/workflows/fuzz.yml");
        let mut run: Vec<&str> = workflow
            .lines()
            .find_map(|line| line.trim().strip_prefix("target: ["))
            .and_then(|list| list.strip_suffix(']'))
            .unwrap()
            .split(", ")
            .collect();
        built.sort_unstable();
        run.sort_unstable();
        assert_eq!(run, built);
    }

    /// The protocol's own messages the `mcp` target starts from, each
    /// under the stateless revision unless it opens the legacy one, and
    /// whether it is answered with a result (`true`) or refused.
    const MCP_SEEDS: &[(&str, bool, &str)] = &[
        ("ping", true, r#"{"method":"ping","params":{}}"#),
        (
            "discover",
            true,
            r#"{"method":"server/discover","params":{}}"#,
        ),
        ("tools-list", true, r#"{"method":"tools/list","params":{}}"#),
        (
            "tools-list-cursor",
            false,
            r#"{"method":"tools/list","params":{"cursor":"2"}}"#,
        ),
        (
            "resources-list",
            true,
            r#"{"method":"resources/list","params":{}}"#,
        ),
        (
            "resource-templates",
            true,
            r#"{"method":"resources/templates/list","params":{}}"#,
        ),
        (
            "resource-read",
            true,
            r#"{"method":"resources/read","params":{"uri":"teistro://catalogue/graha"}}"#,
        ),
        (
            "resource-unknown",
            false,
            r#"{"method":"resources/read","params":{"uri":"teistro://nothing"}}"#,
        ),
        (
            "prompts-list",
            true,
            r#"{"method":"prompts/list","params":{}}"#,
        ),
        (
            "prompt-get",
            true,
            r#"{"method":"prompts/get","params":{"name":"day-panchanga","arguments":{"date":"2026-01-01","last":"2026-01-03","zone":"+05:45","latitude":"27.7","longitude":"85.3"}}}"#,
        ),
        (
            "complete-prompt",
            true,
            r#"{"method":"completion/complete","params":{"ref":{"type":"ref/prompt","name":"day-panchanga"},"argument":{"name":"festivals","value":""}}}"#,
        ),
        (
            "complete-resource",
            true,
            r#"{"method":"completion/complete","params":{"ref":{"type":"ref/resource","uri":"teistro://catalogue/{kind}"},"argument":{"name":"kind","value":"gr"}}}"#,
        ),
        (
            "task-unknown",
            false,
            r#"{"method":"tasks/get","params":{"taskId":"none"}}"#,
        ),
        (
            "method-unknown",
            false,
            r#"{"method":"teistro/nothing","params":{}}"#,
        ),
        (
            "legacy-initialize",
            true,
            r#"{"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"fuzz","version":"0"}}}"#,
        ),
    ];

    /// The stateless revision's `_meta`.
    fn meta() -> serde_json::Value {
        serde_json::json!({ "io.modelcontextprotocol/protocolVersion": "2026-07-28",
                            "io.modelcontextprotocol/clientCapabilities": {} })
    }

    /// Every `mcp` seed by its file name, whether it is answered, and its
    /// bytes: the protocol's messages, then one `tools/call` for each
    /// example request the SDK's own tests send (`crates/sdk/tests/requests.json`),
    /// so a tool gains a seed with its example and none is written by hand.
    fn mcp_seeds() -> Vec<(String, bool, Vec<u8>)> {
        let message = |id: usize, mut message: serde_json::Value| {
            message["jsonrpc"] = "2.0".into();
            message["id"] = id.into();
            if message["method"] != "initialize" {
                message["params"]["_meta"] = meta();
            }
            message.to_string().into_bytes()
        };
        let protocol = MCP_SEEDS
            .iter()
            .enumerate()
            .map(|(id, &(name, answered, text))| {
                (
                    name.to_owned(),
                    answered,
                    message(id, serde_json::from_str(text).unwrap()),
                )
            });
        let examples: Vec<serde_json::Value> =
            serde_json::from_str(include_str!("../../crates/sdk/tests/requests.json")).unwrap();
        let tools = examples.into_iter().enumerate().map(|(id, example)| {
            let tool = example["record"].as_str().unwrap().to_owned();
            let call = serde_json::json!({ "method": "tools/call", "params": {
                "name": tool, "arguments": { "request": example["request"] } } });
            // An engine's own operations are an adapter's, so the
            // built-in ephemeris refuses them.
            let answered = !tool.starts_with("engine.");
            (format!("tool-{tool}"), answered, message(100 + id, call))
        });
        protocol.chain(tools).collect()
    }

    /// Every seed is held to the finding rule and answered as the table
    /// says: a protocol message with a result or an error, a tool call
    /// with a result that is not a refusal.
    #[test]
    fn every_mcp_seed_is_answered_as_listed() {
        let mut wrong = Vec::new();
        for (name, answered, bytes) in mcp_seeds() {
            let mut server = Server::new(Engine::Builtin).with_task_patience(None);
            let Some(reply) = server.handle_bytes(&bytes) else {
                wrong.push(format!("{name} had no reply"));
                continue;
            };
            held(&reply);
            let reply: serde_json::Value = serde_json::from_str(&reply).unwrap();
            let result = reply.get("result").is_some() && reply["result"]["isError"] != true;
            if result != answered {
                wrong.push(format!("{name}: {reply}"));
            }
        }
        assert!(wrong.is_empty(), "{wrong:#?}");
    }

    /// The committed mcp corpus is the seed table written out.
    #[test]
    fn the_mcp_corpus_is_the_seed_table() {
        let wanted = mcp_seeds()
            .into_iter()
            .map(|(name, _, bytes)| (name, bytes))
            .collect();
        corpus_is("mcp", &wanted);
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
