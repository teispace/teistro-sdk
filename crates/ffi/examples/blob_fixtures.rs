//! Writes one blob of every schema into a directory, through the C ABI, so
//! a binding's decoder can be tested against what the library really
//! produces rather than against a hand-built buffer.
//!
//! ```sh
//! cargo run -p teistro-ffi --example blob_fixtures -- target/tsrb
//! ```
//!
//! `cargo xtask check-node` runs it before the Node test.

// An example, not a library: it reports through stdout and stops on a bad
// argument, and it calls the C ABI, so the library's lints are relaxed
// here as they are in the boundary's own tests.
#![allow(
    unsafe_code,
    clippy::print_stdout,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "an example that drives the C boundary"
)]

use std::ffi::CString;
use std::path::Path;
use std::ptr;

use teistro_core::Status;
use teistro_core::catalogue::Calendar;
use teistro_ffi::TS_CONTEXT_TEST_PROVIDER;
use teistro_ffi::blob::{TsBlob, ts_blob_free};
use teistro_ffi::chart::{TsChartRequest, ts_chart_found};
use teistro_ffi::context::{
    TsContext, TsContextOptions, TsEphemeris, ts_context_free, ts_context_new,
};
use teistro_ffi::frame::{TsFrame, ts_frame_canonical, ts_frame_pack};
use teistro_ffi::intl::ts_intl_render;
use teistro_ffi::naam::ts_naam_milan;
use teistro_ffi::panchanga::{TsPanchangaRequest, ts_almanac_digest, ts_panchanga_days};
use teistro_ffi::positions::ts_positions;
use teistro_port_ephemeris::vtable::{ObserverC, PositionRequestC};
use teistro_port_ephemeris::{Body, TimeScale};

/// The bytes of a blob the library filled, copied out before it is freed.
fn take(blob: &mut TsBlob) -> Vec<u8> {
    if blob.data.is_null() {
        return Vec::new();
    }
    // SAFETY: the library wrote `len` bytes at `data`.
    let bytes = unsafe { std::slice::from_raw_parts(blob.data, blob.len) }.to_vec();
    // SAFETY: a descriptor the library wrote, freed once.
    unsafe { ts_blob_free(&raw mut *blob) };
    bytes
}

fn write(dir: &Path, name: &str, bytes: &[u8]) {
    let path = dir.join(name);
    std::fs::write(&path, bytes).expect("the fixture directory is writable");
    println!("{} ({} bytes)", path.display(), bytes.len());
}

/// Every field of a digest as `bindings/c/tests/digest_read.c` prints
/// them, through the library's own reader, so the two can be compared.
fn dump(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    use teistro_idl::blob::{Reader, ScalarValue};
    let schema = teistro::digest_schema();
    let reader = Reader::parse(bytes, &schema).expect("the library's digest parses");
    let column = |section: &str, name: &str| -> Vec<i64> {
        reader
            .column(section, name)
            .expect("the schema's column")
            .iter()
            .map(|value: &ScalarValue| value.as_i64())
            .collect()
    };
    let field = |name: &str| reader.field("summary", name).expect("the field").as_i64();
    let text = reader.bytes("text").expect("the text");
    let (key_at, key_length) = (column("names", "key_at"), column("names", "key_length"));
    let (text_at, text_length) = (column("names", "text_at"), column("names", "text_length"));
    let cut = |at: i64, length: i64| {
        let at = usize::try_from(at).expect("an offset");
        let length = usize::try_from(length).expect("a length");
        String::from_utf8_lossy(text.get(at..at + length).expect("inside the text")).into_owned()
    };
    let name = |row: i64| {
        if row == i64::from(teistro::DIGEST_NO_NAME) {
            return " -".to_owned();
        }
        let row = usize::try_from(row).expect("a row");
        format!(
            " {}={}",
            cut(key_at[row], key_length[row]),
            cut(text_at[row], text_length[row])
        )
    };
    let mut out = format!(
        "days {} base {} offset {} calendar {}\n",
        field("day_count"),
        field("base_unix_seconds"),
        field("utc_offset_seconds"),
        field("calendar"),
    );
    let of = |name: &str| column("days", name);
    let (span_first, span_count) = (of("span_first"), of("span_count"));
    let (observed_first, observed_count) = (of("observance_first"), of("observance_count"));
    let (kind, span_name) = (column("spans", "kind"), column("spans", "name"));
    let (from, to) = (column("spans", "from"), column("spans", "to"));
    let (observed_name, flags) = (
        column("observances", "name"),
        column("observances", "flags"),
    );
    let (year, month, day, vara, state) =
        (of("year"), of("month"), of("day"), of("vara"), of("state"));
    let (fixed, sunrise, sunset) = (of("fixed_day"), of("sunrise"), of("sunset"));
    let clock = column("clock", "utc_offset_seconds");
    let (moonrise, moonset, phase, lunar) = (
        of("moonrise"),
        of("moonset"),
        of("moon_phase"),
        of("lunar_month"),
    );
    for index in 0..year.len() {
        let _ = writeln!(
            out,
            "day {}-{}-{} vara {} state {} fixed {} offset {} sun {} {} moon {} {} phase {} month{}",
            year[index],
            month[index],
            day[index],
            vara[index],
            state[index],
            fixed[index],
            clock[index],
            sunrise[index],
            sunset[index],
            moonrise[index],
            moonset[index],
            phase[index],
            name(lunar[index]),
        );
        for row in span_first[index]..span_first[index] + span_count[index] {
            let row = usize::try_from(row).expect("a row");
            let _ = writeln!(
                out,
                "span {} {} {}{}",
                kind[row],
                from[row],
                to[row],
                name(span_name[row])
            );
        }
        for row in observed_first[index]..observed_first[index] + observed_count[index] {
            let row = usize::try_from(row).expect("a row");
            let _ = writeln!(out, "observance {}{}", flags[row], name(observed_name[row]));
        }
    }
    out
}

fn main() {
    let dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| String::from("target/tsrb"));
    let dir = Path::new(&dir);
    std::fs::create_dir_all(dir).expect("the fixture directory is creatable");

    let profile = CString::new("nepali-drik").expect("a profile id");
    let locale = CString::new("ne-Deva-NP").expect("a locale tag");
    let options = TsContextOptions {
        struct_size: u32::try_from(size_of::<TsContextOptions>()).unwrap_or(0),
        flags: TS_CONTEXT_TEST_PROVIDER,
        profile: profile.as_ptr(),
        settings_json: ptr::null(),
        locale: locale.as_ptr(),
        layouts_json: ptr::null(),
        dashas_json: ptr::null(),
        ephemeris: TsEphemeris::None as u8,
    };
    let mut context: *mut TsContext = ptr::null_mut();
    // SAFETY: valid pointers for the call.
    let status = unsafe {
        ts_context_new(
            &raw const options,
            ptr::null(),
            ptr::null_mut(),
            &raw mut context,
            ptr::null_mut(),
        )
    };
    assert!(status == Status::Ok, "the context builds");

    let mut frame = TsFrame {
        struct_size: u32::try_from(size_of::<TsFrame>()).unwrap_or(0),
        ayanamsha: 0,
        centre: 0,
        equinox: 0,
        coordinates: 0,
        sidereal: 0,
        light_time: 0,
        aberration: 0,
        deflection: 0,
        nutation: 0,
    };
    let mut bits = 0u32;
    // SAFETY: valid pointers for the calls.
    unsafe {
        assert!(
            ts_frame_canonical(&raw mut frame) == Status::Ok,
            "the canonical frame"
        );
        assert!(
            ts_frame_pack(&raw const frame, &raw mut bits) == Status::Ok,
            "the frame packs"
        );
    }

    let jds = [2_451_545.0_f64, 2_451_546.0];
    let bodies = [Body::Sun.id(), Body::Moon.id(), Body::Mars.id()];
    let request = PositionRequestC {
        struct_size: u32::try_from(size_of::<PositionRequestC>()).unwrap_or(0),
        scale: TimeScale::Ut1.id(),
        frame_bits: bits,
        speeds: 1,
        has_observer: 0,
        reserved: [0; 2],
        observer: ObserverC::default(),
        jds: jds.as_ptr(),
        jd_count: jds.len(),
        bodies: bodies.as_ptr(),
        body_count: bodies.len(),
    };
    let mut blob = TsBlob::empty();
    // SAFETY: a live handle, a valid request and a valid slot.
    let status = unsafe { ts_positions(context, &raw const request, &raw mut blob) };
    assert!(status == Status::Ok, "positions compute");
    write(dir, "positions.tsrb", &take(&mut blob));

    let key = CString::new("sdk.reason.grahaInBhava").expect("a key");
    let params = CString::new(r#"{"graha": {"$entity": "graha.JUPITER"}, "bhava": 7}"#)
        .expect("the parameters");
    // SAFETY: a live handle and valid pointers.
    let status = unsafe { ts_intl_render(context, key.as_ptr(), params.as_ptr(), &raw mut blob) };
    assert!(status == Status::Ok, "the message renders");
    write(dir, "intl_render.tsrb", &take(&mut blob));

    charts_panchanga_and_names(context, dir);

    // A week across New York's clock change, so the C reader's dump meets
    // two offsets.
    let week = CString::new(
        r#"{"first": {"year": 2025, "month": 10, "day": 30}, "days": 7, "latitudeDeg": 40.7128, "longitudeDeg": -74.006, "altitudeM": 10, "zone": {"kind": "IANA", "zone": "America/New_York"}, "festivals": {"rules": "NEPAL"}}"#,
    )
    .expect("a request");
    // SAFETY: a live handle, a NUL-terminated request and a valid slot.
    let status = unsafe { ts_almanac_digest(context, week.as_ptr(), &raw mut blob) };
    assert!(status == Status::Ok, "the digest writes");
    let digest = take(&mut blob);
    write(dir, "digest.tsrb", &digest);
    write(dir, "digest.dump", dump(&digest).as_bytes());

    write(dir, PACK, &overlay_pack());

    // SAFETY: the handle came from `ts_context_new` and is not used again.
    unsafe { ts_context_free(context) };
}

/// A blob of each of the schemas a chart, a run of days and two names
/// answer with.
fn charts_panchanga_and_names(context: *mut TsContext, dir: &Path) {
    let mut blob = TsBlob::empty();
    // A chart at one instant with the day's almanac and the houses
    // service beside the foundation, so the optional sections a decoder
    // reads are in the blob rather than absent.
    let instants = [2_451_545.0_f64];
    // SAFETY: every field of the request is a number or a pointer, for
    // which zero is the value an unset field takes.
    let mut chart: TsChartRequest = unsafe { std::mem::zeroed() };
    chart.struct_size = u32::try_from(size_of::<TsChartRequest>()).unwrap_or(0);
    chart.instants = instants.as_ptr();
    chart.instant_count = instants.len();
    chart.latitude_deg = 27.7172;
    chart.longitude_deg = 85.324;
    chart.altitude_m = 1400.0;
    chart.utc_offset_seconds = 20_700;
    chart.sections = 1 | 16;
    // SAFETY: a live handle, a valid request and a valid slot.
    let status = unsafe { ts_chart_found(context, &raw const chart, &raw mut blob) };
    assert!(status == Status::Ok, "the chart is founded");
    write(dir, "charts.tsrb", &take(&mut blob));

    // SAFETY: as for the chart request.
    let mut days: TsPanchangaRequest = unsafe { std::mem::zeroed() };
    days.struct_size = u32::try_from(size_of::<TsPanchangaRequest>()).unwrap_or(0);
    days.calendar = Calendar::Gregorian.id();
    days.from_year = 2026;
    days.from_month = 11;
    days.from_day = 20;
    days.to_year = 2026;
    days.to_month = 11;
    days.to_day = 21;
    days.latitude_deg = 27.7172;
    days.longitude_deg = 85.324;
    days.altitude_m = 1400.0;
    days.utc_offset_seconds = 20_700;
    // SAFETY: a live handle, a valid request and a valid slot.
    let status = unsafe { ts_panchanga_days(context, &raw const days, &raw mut blob) };
    assert!(status == Status::Ok, "the days compute");
    write(dir, "panchanga.tsrb", &take(&mut blob));

    let names = CString::new(r#"{"bride": "सीता", "groom": "राम"}"#).expect("the request");
    // SAFETY: a live handle, a NUL-terminated request and a valid slot.
    let status = unsafe { ts_naam_milan(context, names.as_ptr(), &raw mut blob) };
    assert!(status == Status::Ok, "the names match");
    write(dir, "naam.tsrb", &take(&mut blob));
}

/// The fixture pack's name, which every binding's test reads.
const PACK: &str = "overlay.tpack";

/// A one-record pack that **overlays** a record the engine already has:
/// `graha.SUN` with a form it does not carry.
///
/// Every binding generates `loadPack` and, until this, no binding had ever
/// called it — the path existed in four languages and ran in none, which
/// is the configuration nothing exercises. One record is enough to prove
/// all of it: the bytes cross, the record lands, and the shipped `name`
/// survives beside the new form, which is the merge that a pack carrying
/// one form depends on.
fn overlay_pack() -> Vec<u8> {
    use std::collections::BTreeMap;

    use teistro_intl::source::{ENTITY_NAMESPACE, Entity, Entry, LocaleSource, Namespace};

    let mut forms = BTreeMap::new();
    forms.insert(String::from("phala"), String::from("a reading of the Sun"));
    let mut namespace = Namespace::default();
    namespace.insert(
        String::from("graha.SUN"),
        Entry::Entity(Entity {
            forms,
            gender: None,
            glyph: None,
        }),
    );
    let locale = LocaleSource {
        tag: String::from("en-Latn"),
        meta: serde_json::from_value(serde_json::json!({ "locale": "en-Latn" }))
            .expect("a tag is a locale"),
        namespaces: [(String::from(ENTITY_NAMESPACE), namespace)]
            .into_iter()
            .collect(),
    };
    teistro_intl::pack::build(&locale, ENTITY_NAMESPACE).expect("the fixture pack builds")
}
