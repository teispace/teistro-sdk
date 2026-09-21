//! The boundary exercised as a C caller would: contexts with defaults,
//! refusals with their messages, positions through the test provider
//! decoded from the blob, calendars, time, keys and the locale engine,
//! and the handshake and panic guards that keep a wrong caller from
//! corrupting anything.

#![allow(
    unsafe_code,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::too_many_lines,
    clippy::print_stdout,
    reason = "tests cross the boundary, fail by panicking, read sizes as C does, \
              walk one scenario each, and say when one is skipped for want of \
              an adapter a checkout does not have"
)]

use core::ffi::CStr;
use core::ptr;
use std::ffi::CString;

use teistro_core::Status;
use teistro_core::catalogue::{Calendar, DashaSystem, Era, Graha, Varga};
use teistro_ffi::blob::{TsBlob, ts_blob_free};
use teistro_ffi::calendar::{
    TsCalendarDate, TsResolution, ts_calendar_convert, ts_calendar_fixed_of_jd,
    ts_calendar_from_fixed, ts_calendar_is_leap, ts_calendar_jd_of_fixed, ts_calendar_month_length,
    ts_calendar_to_fixed, ts_calendar_weekday,
};
use teistro_ffi::chart::{TsChartRequest, ts_chart_found, ts_chart_layout_row};
use teistro_ffi::context::{
    TsContext, TsContextOptions, TsEphemeris, TsError, ts_context_free, ts_context_last_error,
    ts_context_new, ts_context_profile, ts_context_settings_hash, ts_context_settings_json,
    ts_error_free,
};
use teistro_ffi::ephemeris::{ts_ephemeris_call, ts_ephemeris_manifest};
use teistro_ffi::intl::{ts_intl_has, ts_intl_locale, ts_intl_render, ts_intl_set_locale};
use teistro_ffi::key::{ts_key_name, ts_key_parse};
use teistro_ffi::positions::ts_positions;
use teistro_ffi::provider::{
    TsProvider, ts_context_new_with_provider, ts_provider_free, ts_provider_load,
};
use teistro_ffi::schemas;
use teistro_ffi::string::{TsHash, TsStr, TsString, ts_string_free};
use teistro_ffi::time::{
    TsCivilDateTime, TsCivilTime, TsDeltaT, TsDeltaTSource, TsScale, TsTimeConversion, TsZoneEra,
    TsZoneResolution, TsZoneSource, TsZoneSpec, ts_time_civil, ts_time_convert, ts_time_delta_t,
    ts_time_resolve,
};
use teistro_ffi::{TS_CONTEXT_TEST_PROVIDER, TS_ERROR_OWNED, ts_abi_version};
use teistro_idl::blob::Reader;
use teistro_port_ephemeris::{Body, Coordinates, Frame, PositionRequestC, TimeScale};

/// An error record's status and strings, copied out: the status, the
/// message, the field, the hint and the detail.
type Record = (
    Status,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
);

/// An empty record with this build's size, ready for a call to write.
fn blank_error() -> TsError {
    TsError {
        struct_size: size_of::<TsError>() as u32,
        status: 99,
        provider_code: 0,
        flags: 0,
        detail: ptr::null(),
        message: ptr::null(),
        field: ptr::null(),
        hint: ptr::null(),
        key: ptr::null(),
    }
}

/// A record's strings copied out, whoever owns them.
fn read_record(error: &TsError) -> Record {
    let text = |p: *const core::ffi::c_char| {
        if p.is_null() {
            None
        } else {
            // SAFETY: the library writes NUL-terminated strings.
            Some(unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned())
        }
    };
    (
        Status::from_code(error.status).unwrap(),
        text(error.message).unwrap_or_default(),
        text(error.field),
        text(error.hint),
        text(error.detail),
    )
}

/// A context with its options, freed on drop.
#[derive(Debug)]
struct Ctx {
    handle: *mut TsContext,
}

impl Ctx {
    fn new(
        flags: u32,
        profile: Option<&str>,
        settings_json: Option<&str>,
        locale: Option<&str>,
    ) -> Result<Ctx, Record> {
        Ctx::with_ephemeris(flags, TsEphemeris::None, profile, settings_json, locale)
    }

    /// The same, naming one of the SDK's own ephemerides (ADR-0028).
    fn with_ephemeris(
        flags: u32,
        ephemeris: TsEphemeris,
        profile: Option<&str>,
        settings_json: Option<&str>,
        locale: Option<&str>,
    ) -> Result<Ctx, Record> {
        Ctx::open(flags, ephemeris, profile, settings_json, locale, None, None)
    }

    /// A context on the test provider with a consumer's own layouts.
    fn with_layouts(layouts_json: &str) -> Result<Ctx, Record> {
        Ctx::open(
            TS_CONTEXT_TEST_PROVIDER,
            TsEphemeris::None,
            None,
            None,
            None,
            Some(layouts_json),
            None,
        )
    }

    /// A context on the test provider with a consumer's own dasha systems.
    fn with_dashas(dashas_json: &str) -> Result<Ctx, Record> {
        Ctx::open(
            TS_CONTEXT_TEST_PROVIDER,
            TsEphemeris::None,
            None,
            None,
            None,
            None,
            Some(dashas_json),
        )
    }

    fn open(
        flags: u32,
        ephemeris: TsEphemeris,
        profile: Option<&str>,
        settings_json: Option<&str>,
        locale: Option<&str>,
        layouts_json: Option<&str>,
        dashas_json: Option<&str>,
    ) -> Result<Ctx, Record> {
        let layouts = layouts_json.map(|p| CString::new(p).unwrap());
        let dashas = dashas_json.map(|p| CString::new(p).unwrap());
        let profile = profile.map(|p| CString::new(p).unwrap());
        let settings = settings_json.map(|p| CString::new(p).unwrap());
        let locale = locale.map(|p| CString::new(p).unwrap());
        let options = TsContextOptions {
            struct_size: size_of::<TsContextOptions>() as u32,
            flags,
            profile: profile.as_ref().map_or(ptr::null(), |p| p.as_ptr()),
            settings_json: settings.as_ref().map_or(ptr::null(), |p| p.as_ptr()),
            locale: locale.as_ref().map_or(ptr::null(), |p| p.as_ptr()),
            layouts_json: layouts.as_ref().map_or(ptr::null(), |p| p.as_ptr()),
            dashas_json: dashas.as_ref().map_or(ptr::null(), |p| p.as_ptr()),
            ephemeris: ephemeris as u8,
        };
        let mut handle = ptr::null_mut();
        let mut error = blank_error();
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
        if status == Status::Ok {
            return Ok(Ctx { handle });
        }
        assert_eq!(
            error.flags, TS_ERROR_OWNED,
            "a constructor's record owns its strings"
        );
        let record = read_record(&error);
        assert_eq!(record.0, status, "the record is the call's own refusal");
        // SAFETY: a record the library wrote, freed once.
        unsafe { ts_error_free(&raw mut error) };
        Err(record)
    }

    fn defaults() -> Ctx {
        Ctx::new(0, None, None, None).expect("a context with every default")
    }

    fn last_error(&self) -> Record {
        let mut error = blank_error();
        // SAFETY: a live handle and a valid struct with its size set.
        assert_eq!(
            unsafe { ts_context_last_error(self.handle, &raw mut error) },
            Status::Ok
        );
        assert_eq!(error.flags, 0, "a context's record lends its strings");
        read_record(&error)
    }
}

impl Drop for Ctx {
    fn drop(&mut self) {
        // SAFETY: the handle came from `ts_context_new` and is dropped once.
        unsafe { ts_context_free(self.handle) };
    }
}

fn lent(s: TsStr) -> String {
    // SAFETY: the library lends NUL-terminated strings.
    unsafe { CStr::from_ptr(s.data) }
        .to_str()
        .unwrap()
        .to_string()
}

fn owned(mut s: TsString) -> String {
    // SAFETY: a NUL-terminated string the library allocated.
    let text = unsafe { CStr::from_ptr(s.data.cast()) }
        .to_str()
        .unwrap()
        .to_string();
    // SAFETY: a descriptor the library wrote.
    unsafe { ts_string_free(&raw mut s) };
    text
}

fn sized<T>(mut value: T, set: impl FnOnce(&mut T, u32)) -> T {
    set(&mut value, size_of::<T>() as u32);
    value
}

fn date(calendar: Calendar, year: i32, month: u8, day: u8) -> TsCalendarDate {
    sized(
        TsCalendarDate {
            struct_size: 0,
            calendar: calendar.id(),
            era: 0xFFFF,
            year,
            era_year: 0,
            month,
            day,
            resolution: 0,
            computed_month: 0,
            computed_day: 0,
            reserved: [0; 3],
        },
        |d, s| d.struct_size = s,
    )
}

fn blank_date() -> TsCalendarDate {
    date(Calendar::Gregorian, 0, 0, 0)
}

#[test]
fn a_context_with_every_default_reports_its_settings() {
    assert_eq!(ts_abi_version(), 1);
    let ctx = Ctx::defaults();
    let mut profile = TsStr {
        data: ptr::null(),
        len: 0,
    };
    // SAFETY: a live handle and a valid slot.
    assert_eq!(
        unsafe { ts_context_profile(ctx.handle, &raw mut profile) },
        Status::Ok
    );
    assert_eq!(lent(profile), "parashari-classical");
    let mut json = TsString::empty();
    // SAFETY: a live handle and a valid slot.
    assert_eq!(
        unsafe { ts_context_settings_json(ctx.handle, &raw mut json) },
        Status::Ok
    );
    let json = owned(json);
    assert!(json.starts_with("{\"aspect\":"), "{}", &json[..30]);
    let mut hash = TsHash { bytes: [0; 32] };
    // SAFETY: a live handle and a valid slot.
    assert_eq!(
        unsafe { ts_context_settings_hash(ctx.handle, &raw mut hash) },
        Status::Ok
    );
    assert_eq!(
        hash,
        teistro_core::envelope::Hash::of(json.as_bytes()).into()
    );
    let (status, message, field, hint, detail) = ctx.last_error();
    assert_eq!(
        (status, message.as_str(), field, hint, detail),
        (Status::Ok, "", None, None, None)
    );
}

#[test]
fn a_refused_construction_owns_its_record_and_frees_it_once() {
    let profile = CString::new("nepali-defualt").unwrap();
    let options = TsContextOptions {
        struct_size: size_of::<TsContextOptions>() as u32,
        flags: 0,
        profile: profile.as_ptr(),
        settings_json: ptr::null(),
        locale: ptr::null(),
        layouts_json: ptr::null(),
        dashas_json: ptr::null(),
        ephemeris: 0,
    };
    let new = |out: *mut *mut TsContext, error: *mut TsError| {
        // SAFETY: valid options; the slots are the test's to pass.
        unsafe { ts_context_new(&raw const options, ptr::null(), ptr::null_mut(), out, error) }
    };
    let mut handle = ptr::null_mut();

    // No record asked for: the status alone.
    assert_eq!(new(&raw mut handle, ptr::null_mut()), Status::Unsupported);

    // A record of a size this build does not know is left alone, and the
    // call still says what it refused rather than `SCHEMA_VERSION`.
    let mut stale = blank_error();
    stale.struct_size = 8;
    assert_eq!(new(&raw mut handle, &raw mut stale), Status::Unsupported);
    assert_eq!(
        (stale.status, stale.flags, stale.message),
        (99, 0, ptr::null())
    );

    // The whole refusal, owned.
    let mut error = blank_error();
    assert_eq!(new(&raw mut handle, &raw mut error), Status::Unsupported);
    assert!(handle.is_null(), "no handle is written on failure");
    assert_eq!(error.flags, TS_ERROR_OWNED);
    let (_, _, field, hint, _) = read_record(&error);
    assert_eq!(field.as_deref(), Some("profile"));
    assert!(hint.is_some_and(|h| h.contains("nepali-default")));

    // Freed, zeroed but for the size, and a second free is a no-op.
    // SAFETY: the record the call wrote.
    unsafe { ts_error_free(&raw mut error) };
    assert_eq!(
        (error.struct_size, error.status, error.flags, error.message),
        (size_of::<TsError>() as u32, 0, 0, ptr::null())
    );
    // SAFETY: as above; the flag is clear now.
    unsafe { ts_error_free(&raw mut error) };
    // SAFETY: null is ignored.
    unsafe { ts_error_free(ptr::null_mut()) };

    // A null slot for the handle is refused, and the record names it.
    let mut error = blank_error();
    assert_eq!(new(ptr::null_mut(), &raw mut error), Status::InvalidArg);
    assert_eq!(read_record(&error).2.as_deref(), Some("out_context"));
    // SAFETY: the record the call wrote.
    unsafe { ts_error_free(&raw mut error) };

    // A lent record is not the caller's to free: freeing it does nothing,
    // and its strings are still the context's.
    let ctx = Ctx::defaults();
    let mut id = 0u32;
    let unknown = CString::new("graha.SUNN").unwrap();
    // SAFETY: a live handle and valid slots.
    let refused = unsafe { ts_key_parse(ctx.handle, unknown.as_ptr(), &raw mut id) };
    assert_eq!(refused, Status::Unsupported);
    let mut lent = blank_error();
    // SAFETY: a live handle and a valid struct with its size set.
    unsafe { ts_context_last_error(ctx.handle, &raw mut lent) };
    let before = read_record(&lent);
    // SAFETY: a lent record; the call must leave it alone.
    unsafe { ts_error_free(&raw mut lent) };
    assert_eq!(read_record(&lent), before);
}

#[test]
fn a_context_refuses_what_it_cannot_build_and_says_why() {
    let (status, message, field, hint, _) =
        Ctx::new(0, Some("vedic-classic"), None, None).unwrap_err();
    assert_eq!(
        (status, message.as_str(), field.as_deref()),
        (
            Status::Unsupported,
            "no shipped profile `vedic-classic`",
            Some("profile")
        )
    );
    assert!(hint.is_some_and(|h| h.contains("parashari-classical")));
    let (status, message, field, ..) =
        Ctx::new(0, None, Some(r#"{"frame": {"zodiacs": "TROPICAL"}}"#), None).unwrap_err();
    assert_eq!(
        (status, field.as_deref()),
        (Status::InvalidArg, Some("settings_json"))
    );
    assert!(message.contains("unknown field `zodiacs`"), "{message}");
    let (status, _, field, hint, _) = Ctx::new(0, None, None, Some("xx-Latn")).unwrap_err();
    assert_eq!(
        (status, field.as_deref()),
        (Status::Unsupported, Some("locale"))
    );
    assert!(hint.is_some_and(|h| h.contains("ne-Deva-NP")));
    let patched = Ctx::new(
        0,
        Some("nepali-default"),
        Some(r#"{"frame": {"zodiac": "TROPICAL"}}"#),
        Some("ne-Deva-NP"),
    )
    .unwrap();
    let mut json = TsString::empty();
    // SAFETY: a live handle and a valid slot.
    assert_eq!(
        unsafe { ts_context_settings_json(patched.handle, &raw mut json) },
        Status::Ok
    );
    assert!(owned(json).contains("\"zodiac\":\"TROPICAL\""));
    let mut locale = TsStr {
        data: ptr::null(),
        len: 0,
    };
    // SAFETY: a live handle and a valid slot.
    assert_eq!(
        unsafe { ts_intl_locale(patched.handle, &raw mut locale) },
        Status::Ok
    );
    assert_eq!(lent(locale), "ne-Deva-NP");

    // A wrong struct_size is a schema mismatch, and a null slot is refused.
    let options = TsContextOptions {
        struct_size: 12,
        flags: 0,
        profile: ptr::null(),
        settings_json: ptr::null(),
        locale: ptr::null(),
        layouts_json: ptr::null(),
        dashas_json: ptr::null(),
        ephemeris: TsEphemeris::None as u8,
    };
    let mut handle = ptr::null_mut();
    // SAFETY: valid pointers.
    assert_eq!(
        unsafe {
            ts_context_new(
                &raw const options,
                ptr::null(),
                ptr::null_mut(),
                &raw mut handle,
                ptr::null_mut(),
            )
        },
        Status::SchemaVersion
    );
    // SAFETY: a null slot.
    assert_eq!(
        unsafe {
            ts_context_new(
                ptr::null(),
                ptr::null(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
            )
        },
        Status::InvalidArg
    );
    // SAFETY: null is ignored.
    unsafe { ts_context_free(ptr::null_mut()) };
}

/// **An engine, loaded rather than linked** (ADR-0029).
///
/// This is the 98% path at the boundary a consumer crosses: no vtable
/// written by hand, no engine compiled into this library — which its
/// licence forbids — but an adapter opened from a file and a real sky
/// computed through it.
///
/// It runs only where the adapter has been built and its data is present,
/// because neither can be assumed of a checkout; where they are, it is
/// the test that the whole seam holds. `TEISTRO_TEIMERIS_ADAPTER` names
/// the library.
#[test]
fn an_engine_is_loaded_from_a_shared_library_and_computes() {
    let Some(path) = std::env::var_os("TEISTRO_TEIMERIS_ADAPTER") else {
        // A checkout has neither the adapter built nor the engine's data,
        // and a test that failed for that would fail for everyone.
        println!(
            "skipped: the adapter is built separately; set TEISTRO_TEIMERIS_ADAPTER to its library"
        );
        return;
    };
    let path = CString::new(path.to_string_lossy().as_ref()).unwrap();
    let mut provider: *mut TsProvider = ptr::null_mut();
    let mut error = blank_error();
    // SAFETY: a live path and writable slots.
    let status = unsafe {
        ts_provider_load(
            path.as_ptr(),
            ptr::null(),
            &raw mut provider,
            &raw mut error,
        )
    };
    assert_eq!(
        status,
        Status::Ok,
        "loading the adapter: {:?}",
        read_record(&error)
    );

    let mut context: *mut TsContext = ptr::null_mut();
    // SAFETY: a live handle and writable slots.
    let status = unsafe {
        ts_context_new_with_provider(ptr::null(), provider, &raw mut context, ptr::null_mut())
    };
    assert_eq!(status, Status::Ok, "a context over the loaded engine");

    // Freed **first**, on purpose: the context holds its own reference,
    // so the library must still be there for the call below. An ordering
    // that mattered would be a footgun in every binding.
    // SAFETY: a handle from `ts_provider_load`, freed once.
    unsafe { ts_provider_free(provider) };

    let jds = [2_451_545.0];
    let bodies = [Body::Sun.id()];
    let frame = Frame::CANONICAL;
    let request = sized(
        PositionRequestC {
            struct_size: 0,
            scale: TimeScale::Tt.id(),
            frame_bits: frame.to_bits(),
            speeds: 1,
            has_observer: 0,
            reserved: [0; 2],
            observer: teistro_port_ephemeris::vtable::ObserverC::default(),
            jds: jds.as_ptr(),
            jd_count: jds.len(),
            bodies: bodies.as_ptr(),
            body_count: bodies.len(),
        },
        |r, s| r.struct_size = s,
    );
    let mut blob = TsBlob::empty();
    // SAFETY: a live context, a valid request and a valid slot.
    assert_eq!(
        unsafe { ts_positions(context, &raw const request, &raw mut blob) },
        Status::Ok,
        "the loaded engine answered"
    );
    // SAFETY: the library wrote `len` bytes.
    let bytes = unsafe { core::slice::from_raw_parts(blob.data, blob.len) }.to_vec();
    // SAFETY: a descriptor the library wrote.
    unsafe { ts_blob_free(&raw mut blob) };
    let schema = schemas::positions();
    let reader = Reader::parse(&bytes, &schema).unwrap();
    let sun = reader.column("cells", "lon").unwrap()[0].as_f64();
    assert!(
        (279.0..282.0).contains(&sun),
        "the engine put the Sun at {sun} degrees at J2000"
    );
    // **The whole chain**: a consumer's binding, this library, an adapter
    // loaded from a file, and the engine's own function reached by name
    // (ADR-0030). Nothing of the engine is compiled into this library,
    // and none of the names below appear in it.
    let mut json = TsString::empty();
    // SAFETY: a live context and a writable slot.
    assert_eq!(
        unsafe { ts_ephemeris_manifest(context, &raw mut json) },
        Status::Ok,
        "a loaded engine describes itself"
    );
    // SAFETY: the library wrote `len` bytes.
    let manifest = unsafe { core::slice::from_raw_parts(json.data, json.len) }.to_vec();
    // SAFETY: a descriptor the library wrote.
    unsafe { ts_string_free(&raw mut json) };
    let manifest = String::from_utf8(manifest).expect("the manifest is text");
    assert!(
        manifest.contains("tm_delta_t"),
        "the manifest lists what can be called"
    );
    assert!(
        !manifest.contains("tm_context_close"),
        "and not what the adapter owns"
    );

    let name = CString::new("tm_delta_t").unwrap();
    let arguments = CString::new(r#"{"jd_ut1": 2451545.0}"#).unwrap();
    let mut answer = TsString::empty();
    // SAFETY: a live context, live strings and a writable slot.
    assert_eq!(
        unsafe { ts_ephemeris_call(context, name.as_ptr(), arguments.as_ptr(), &raw mut answer,) },
        Status::Ok,
        "and answers when called by name"
    );
    // SAFETY: the library wrote `len` bytes.
    let said = unsafe { core::slice::from_raw_parts(answer.data, answer.len) }.to_vec();
    // SAFETY: a descriptor the library wrote.
    unsafe { ts_string_free(&raw mut answer) };
    let said = String::from_utf8(said).expect("the answer is text");
    assert!(
        said.contains("out_seconds"),
        "the engine's out-parameter comes back under its own name: {said}"
    );

    // SAFETY: a live context, freed once.
    unsafe { ts_context_free(context) };
}

/// **Phase 3's promise at the boundary a consumer actually crosses.**
///
/// Every other test here computes with the test provider, whose positions
/// are not astronomy. This one names the SDK's own ephemeris and gets a
/// real sky: no vtable, no files, no network, no second library. It is
/// the whole of what "a chart computes with nothing but the SDK
/// installed" means for a caller outside Rust (ADR-0008, ADR-0028).
///
/// It asserts places and not accuracy — accuracy is measured against an
/// engine in `03-design/completion-measured.md` — so what it checks is
/// what a boundary test can: the Sun is where the Sun is at J2000, it
/// moves about a degree a day, and the provenance names the built-in
/// rather than whatever was bound last.
#[cfg(feature = "builtin-ephemeris")]
#[test]
fn the_built_in_ephemeris_computes_a_real_sky_across_the_boundary() {
    let ctx = Ctx::with_ephemeris(0, TsEphemeris::Builtin, None, None, None).unwrap();
    let jds = [2_451_545.0, 2_451_546.0];
    let bodies = [Body::Sun.id()];
    let frame = Frame::CANONICAL;
    let request = sized(
        PositionRequestC {
            struct_size: 0,
            scale: TimeScale::Tt.id(),
            frame_bits: frame.to_bits(),
            speeds: 1,
            has_observer: 0,
            reserved: [0; 2],
            observer: teistro_port_ephemeris::vtable::ObserverC::default(),
            jds: jds.as_ptr(),
            jd_count: jds.len(),
            bodies: bodies.as_ptr(),
            body_count: bodies.len(),
        },
        |r, s| r.struct_size = s,
    );
    let mut blob = TsBlob::empty();
    // SAFETY: a live handle, a valid request and a valid slot.
    assert_eq!(
        unsafe { ts_positions(ctx.handle, &raw const request, &raw mut blob) },
        Status::Ok,
        "the built-in answers where no provider was bound"
    );
    // SAFETY: the library wrote `len` bytes.
    let bytes = unsafe { core::slice::from_raw_parts(blob.data, blob.len) }.to_vec();
    // SAFETY: a descriptor the library wrote.
    unsafe { ts_blob_free(&raw mut blob) };
    let schema = schemas::positions();
    let reader = Reader::parse(&bytes, &schema).unwrap();
    let lon = reader.column("cells", "lon").unwrap();
    let speed = reader.column("cells", "lon_speed").unwrap();
    assert_eq!(reader.count("cells"), Some(2));
    assert!(
        reader
            .column("cells", "status")
            .unwrap()
            .iter()
            .all(|s| s.as_i64() == 0),
        "every cell computed"
    );
    // The Sun's apparent tropical longitude at J2000.0 is 280.4 degrees,
    // a figure any almanac carries; a degree of room is far tighter than
    // anything that could pass by accident and far looser than the
    // arcseconds the accuracy document measures.
    let sun = lon[0].as_f64();
    assert!(
        (279.0..282.0).contains(&sun),
        "the Sun is at {sun} degrees at J2000, which is not where the Sun is"
    );
    // And it moves the way the Sun moves.
    let daily = speed[0].as_f64();
    assert!(
        (0.9..1.1).contains(&daily),
        "the Sun moves {daily} degrees a day, which is not a year"
    );
    let moved = lon[1].as_f64() - sun;
    assert!(
        (0.9..1.1).contains(&moved),
        "a day later it has moved {moved} degrees"
    );
}

#[test]
fn positions_come_back_as_a_blob_with_steps_and_provenance() {
    let ctx = Ctx::new(TS_CONTEXT_TEST_PROVIDER, None, None, None).unwrap();
    let jds = [2_451_545.0, 2_451_546.0];
    let bodies = [Body::Sun.id(), Body::Moon.id()];
    let frame = Frame::CANONICAL.with_coordinates(Coordinates::Equatorial);
    let request = sized(
        PositionRequestC {
            struct_size: 0,
            scale: TimeScale::Ut1.id(),
            frame_bits: frame.to_bits(),
            speeds: 1,
            has_observer: 0,
            reserved: [0; 2],
            observer: teistro_port_ephemeris::vtable::ObserverC::default(),
            jds: jds.as_ptr(),
            jd_count: jds.len(),
            bodies: bodies.as_ptr(),
            body_count: bodies.len(),
        },
        |r, s| r.struct_size = s,
    );
    let mut blob = TsBlob::empty();
    // SAFETY: a live handle, a valid request and a valid slot.
    assert_eq!(
        unsafe { ts_positions(ctx.handle, &raw const request, &raw mut blob) },
        Status::Ok
    );
    // SAFETY: the library wrote `len` bytes.
    let bytes = unsafe { core::slice::from_raw_parts(blob.data, blob.len) }.to_vec();
    // SAFETY: a descriptor the library wrote.
    unsafe { ts_blob_free(&raw mut blob) };
    let schema = schemas::positions();
    let reader = Reader::parse(&bytes, &schema).unwrap();
    let summary = reader.fixed("summary").unwrap();
    assert_eq!(summary[0].as_i64() as u32, frame.to_bits());
    assert_eq!(
        (
            summary[1].as_i64(),
            summary[2].as_i64(),
            summary[3].as_i64()
        ),
        (2, 2, 0)
    );
    assert_eq!(reader.count("cells"), Some(4));
    let lon = reader.column("cells", "lon").unwrap();
    assert!(lon.iter().all(|v| (0.0..360.0).contains(&v.as_f64())));
    assert!(
        reader
            .column("cells", "status")
            .unwrap()
            .iter()
            .all(|s| s.as_i64() == 0)
    );
    assert_eq!(
        reader
            .column("bodies", "body")
            .unwrap()
            .iter()
            .map(|b| b.as_i64())
            .collect::<Vec<_>>(),
        [0, 1]
    );
    assert_eq!(
        reader.column("instants", "jd").unwrap()[1].as_f64(),
        2_451_546.0
    );
    let steps = reader.text("steps").unwrap();
    assert!(
        steps.contains("\"name\":\"obliquity\"") && steps.contains("\"SDK\""),
        "{steps}"
    );
    let provenance: serde_json::Value =
        serde_json::from_str(reader.text("provenance").unwrap()).unwrap();
    assert_eq!(provenance["calculation_version"], 1);
    assert_eq!(provenance["profile"], "parashari-classical");
    assert_eq!(provenance["settings_hash"].as_str().unwrap().len(), 64);
    assert_eq!(provenance["provider"]["frame"], frame.key());
    assert_eq!(provenance["time"]["delta_t_model"], "TABLE_THEN_MODEL");

    // The same request again gives the same bytes: the blob is deterministic.
    let mut again = TsBlob::empty();
    // SAFETY: as above.
    assert_eq!(
        unsafe { ts_positions(ctx.handle, &raw const request, &raw mut again) },
        Status::Ok
    );
    // SAFETY: the library wrote `len` bytes.
    assert_eq!(
        unsafe { core::slice::from_raw_parts(again.data, again.len) },
        &bytes[..]
    );
    // SAFETY: a descriptor the library wrote.
    unsafe { ts_blob_free(&raw mut again) };

    // Without an ephemeris the call is a missing capability naming the field.
    let bare = Ctx::defaults();
    // SAFETY: as above.
    assert_eq!(
        unsafe { ts_positions(bare.handle, &raw const request, &raw mut blob) },
        Status::Capability
    );
    let (status, message, field, hint, _) = bare.last_error();
    assert_eq!(
        (status, field.as_deref()),
        (Status::Capability, Some("ephemeris"))
    );
    // The field and the hint name the option a consumer sets, in whatever
    // language they are in -- not `ts_context_new` and not the flag,
    // which is what this message used to say to a Node caller who has
    // neither.
    assert!(message.contains("has no ephemeris"), "{message}");
    let hint = hint.expect("the refusal hints at what to pass");
    assert!(
        hint.contains("`builtin`") && hint.contains("descriptor"),
        "{hint}"
    );
    // A request with a wrong size, and a null request.
    let mut wrong = request;
    wrong.struct_size = 4;
    // SAFETY: as above.
    assert_eq!(
        unsafe { ts_positions(ctx.handle, &raw const wrong, &raw mut blob) },
        Status::InvalidArg
    );
    // SAFETY: as above.
    assert_eq!(
        unsafe { ts_positions(ctx.handle, ptr::null(), &raw mut blob) },
        Status::InvalidArg
    );
}

#[test]
fn calendars_convert_through_the_fixed_day_with_eras_and_resolutions() {
    let ctx = Ctx::defaults();
    let gregorian = date(Calendar::Gregorian, 2015, 4, 14);
    let mut fixed = 0i64;
    // SAFETY: a live handle and valid pointers.
    assert_eq!(
        unsafe { ts_calendar_to_fixed(ctx.handle, &raw const gregorian, &raw mut fixed) },
        Status::Ok
    );
    let mut bs = blank_date();
    // SAFETY: as above.
    assert_eq!(
        unsafe {
            ts_calendar_convert(
                ctx.handle,
                &raw const gregorian,
                Calendar::BikramSambat.id(),
                &raw mut bs,
            )
        },
        Status::Ok
    );
    assert_eq!((bs.year, bs.month, bs.day), (2072, 1, 1));
    assert_eq!(bs.era, Era::Vikrama.id());
    assert_eq!(bs.era_year, 2072);
    assert_eq!(bs.resolution, TsResolution::Tabular as u8);
    let mut back = blank_date();
    // SAFETY: as above.
    assert_eq!(
        unsafe {
            ts_calendar_from_fixed(ctx.handle, Calendar::Gregorian.id(), fixed, &raw mut back)
        },
        Status::Ok
    );
    assert_eq!(
        (back.year, back.month, back.day, back.resolution),
        (2015, 4, 14, TsResolution::Defined as u8)
    );
    assert_eq!(back.era, Era::CommonEra.id());
    let mut weekday = 0u8;
    // SAFETY: as above.
    assert_eq!(
        unsafe { ts_calendar_weekday(ctx.handle, &raw const gregorian, &raw mut weekday) },
        Status::Ok
    );
    assert_eq!(weekday, 2, "a Tuesday");
    let (mut length, mut leap) = (0u8, 0u8);
    // SAFETY: as above.
    unsafe {
        assert_eq!(
            ts_calendar_month_length(
                ctx.handle,
                Calendar::Gregorian.id(),
                2024,
                2,
                &raw mut length
            ),
            Status::Ok
        );
        assert_eq!(
            ts_calendar_is_leap(ctx.handle, Calendar::Gregorian.id(), 2024, &raw mut leap),
            Status::Ok
        );
    }
    assert_eq!((length, leap), (29, 1));
    assert_eq!(ts_calendar_jd_of_fixed(fixed), 2_457_126.5);
    let mut fraction = 0.0;
    // SAFETY: a valid slot.
    assert_eq!(
        unsafe { ts_calendar_fixed_of_jd(2_457_126.75, &raw mut fraction) },
        fixed
    );
    assert_eq!(fraction, 0.25);
    // SAFETY: null is allowed.
    assert_eq!(
        unsafe { ts_calendar_fixed_of_jd(2_457_126.75, ptr::null_mut()) },
        fixed
    );

    let nonexistent = date(Calendar::Gregorian, 2023, 2, 29);
    // SAFETY: as above.
    assert_eq!(
        unsafe { ts_calendar_to_fixed(ctx.handle, &raw const nonexistent, &raw mut fixed) },
        Status::InvalidArg
    );
    let (_, message, _, _, detail) = ctx.last_error();
    assert_eq!(detail.as_deref(), Some("NONEXISTENT_DATE"), "{message}");
    // SAFETY: as above.
    assert_eq!(
        unsafe { ts_calendar_from_fixed(ctx.handle, 999, fixed, &raw mut back) },
        Status::Unsupported
    );
    let (_, _, field, hint, detail) = ctx.last_error();
    assert_eq!(
        (field.as_deref(), detail.as_deref()),
        (Some("calendar"), Some("UNKNOWN_KEY"))
    );
    assert!(hint.unwrap().contains("GREGORIAN=0"));
    // SAFETY: as above.
    assert_eq!(
        unsafe {
            ts_calendar_from_fixed(
                ctx.handle,
                Calendar::IndianLunisolar.id(),
                fixed,
                &raw mut back,
            )
        },
        Status::Unsupported
    );
    let mut small = gregorian;
    small.struct_size = 8;
    // SAFETY: as above.
    assert_eq!(
        unsafe { ts_calendar_to_fixed(ctx.handle, &raw const small, &raw mut fixed) },
        Status::SchemaVersion
    );
}

fn kathmandu() -> (TsZoneSpec, CString) {
    let name = CString::new("Asia/Kathmandu").unwrap();
    let spec = sized(
        TsZoneSpec {
            zone: name.as_ptr(),
            ..TsZoneSpec::default()
        },
        |z, s| z.struct_size = s,
    );
    (spec, name)
}

fn civil(year: i32, month: u8, day: u8, hour: u8, minute: u8) -> TsCivilDateTime {
    sized(
        TsCivilDateTime {
            struct_size: 0,
            reserved: 0,
            date: date(Calendar::Gregorian, year, month, day),
            time: sized(
                TsCivilTime {
                    struct_size: 0,
                    hour,
                    minute,
                    second: 0,
                    has_time: 1,
                    nanos: 0,
                },
                |t, s| t.struct_size = s,
            ),
        },
        |c, s| c.struct_size = s,
    )
}

fn blank_resolution() -> TsZoneResolution {
    sized(
        TsZoneResolution {
            struct_size: 0,
            offset_seconds: 0,
            dst_shift_seconds: 0,
            warnings: 0,
            instant_jd_utc: 0.0,
            tzdb_version: ptr::null(),
            abbreviation: ptr::null(),
            source: 0,
            era: 0,
            dst: 0,
            chosen: 0,
            time_known: 0,
            reserved: [0; 3],
        },
        |r, s| r.struct_size = s,
    )
}

#[test]
fn a_nepali_birth_time_resolves_with_replay_metadata_and_converts_between_scales() {
    let ctx = Ctx::defaults();
    let (zone, _name) = kathmandu();
    let birth = civil(1986, 1, 1, 0, 20);
    let mut resolution = blank_resolution();
    // SAFETY: a live handle and valid pointers.
    assert_eq!(
        unsafe {
            ts_time_resolve(
                ctx.handle,
                &raw const birth,
                &raw const zone,
                &raw mut resolution,
            )
        },
        Status::Ok
    );
    assert!((resolution.instant_jd_utc - 2_446_431.274_305_6).abs() < 1e-6);
    assert_eq!(resolution.offset_seconds, 20_700);
    assert_eq!(
        (
            resolution.source,
            resolution.era,
            resolution.dst,
            resolution.time_known
        ),
        (TsZoneSource::Iana as u8, TsZoneEra::Current as u8, 0, 1)
    );
    assert_eq!(resolution.warnings, 0);
    // SAFETY: lent strings.
    let version = unsafe { CStr::from_ptr(resolution.tzdb_version) }
        .to_str()
        .unwrap();
    assert!(version.starts_with("20"), "{version}");
    assert!(!resolution.abbreviation.is_null());

    let mut civil_back = civil(0, 1, 1, 0, 0);
    let mut again = blank_resolution();
    // SAFETY: as above.
    assert_eq!(
        unsafe {
            ts_time_civil(
                ctx.handle,
                resolution.instant_jd_utc,
                &raw const zone,
                Calendar::Gregorian.id(),
                &raw mut civil_back,
                &raw mut again,
            )
        },
        Status::Ok
    );
    assert_eq!(
        (
            civil_back.date.year,
            civil_back.date.month,
            civil_back.date.day
        ),
        (1986, 1, 1)
    );
    assert_eq!(
        (
            civil_back.time.hour,
            civil_back.time.minute,
            civil_back.time.has_time
        ),
        (0, 20, 1)
    );
    assert_eq!(again.offset_seconds, 20_700);

    // `Asia/Katmandu` is a link the database keeps; a misspelling beyond it
    // is refused with the nearest names as the hint.
    let unknown = CString::new("Asia/Kathmandou").unwrap();
    let bad = sized(
        TsZoneSpec {
            zone: unknown.as_ptr(),
            ..TsZoneSpec::default()
        },
        |z, s| z.struct_size = s,
    );
    // SAFETY: as above.
    let status = unsafe {
        ts_time_resolve(
            ctx.handle,
            &raw const birth,
            &raw const bad,
            &raw mut resolution,
        )
    };
    assert_ne!(status, Status::Ok);
    let (_, message, _, hint, _) = ctx.last_error();
    assert!(
        hint.unwrap_or_default().contains("Asia/Kathmandu") || message.contains("Asia/Kathmandu"),
        "{message}"
    );
    let nameless = sized(TsZoneSpec::default(), |z, s| z.struct_size = s);
    // SAFETY: as above.
    assert_eq!(
        unsafe {
            ts_time_resolve(
                ctx.handle,
                &raw const birth,
                &raw const nameless,
                &raw mut resolution,
            )
        },
        Status::InvalidArg
    );
    assert_eq!(ctx.last_error().2.as_deref(), Some("zone.zone"));

    let mut conversion = sized(
        TsTimeConversion {
            struct_size: 0,
            from: 0,
            to: 0,
            delta_t_source: 0,
            proleptic_utc: 0,
            has_uncertainty: 0,
            reserved: 0,
            jd: 0.0,
            delta_t_seconds: 0.0,
            uncertainty_seconds: 0.0,
            dut1_seconds: 0.0,
            delta_t_model: ptr::null(),
        },
        |c, s| c.struct_size = s,
    );
    // SAFETY: as above.
    assert_eq!(
        unsafe {
            ts_time_convert(
                ctx.handle,
                2_451_544.5,
                TsScale::Utc as u32,
                TsScale::Tt as u32,
                &raw mut conversion,
            )
        },
        Status::Ok
    );
    assert!((conversion.delta_t_seconds - 64.184).abs() < 1e-9);
    assert!((conversion.jd - (2_451_544.5 + 64.184 / 86_400.0)).abs() < 1e-12);
    assert_eq!(conversion.delta_t_source, TsDeltaTSource::LeapSeconds as u8);
    // SAFETY: lent string.
    assert_eq!(
        unsafe { CStr::from_ptr(conversion.delta_t_model) }
            .to_str()
            .unwrap(),
        "TABLE_THEN_MODEL"
    );
    // SAFETY: as above.
    assert_eq!(
        unsafe {
            ts_time_convert(
                ctx.handle,
                conversion.jd,
                TsScale::Tt as u32,
                TsScale::Utc as u32,
                &raw mut conversion,
            )
        },
        Status::Ok
    );
    assert!((conversion.jd - 2_451_544.5).abs() < 1e-9);
    // SAFETY: as above.
    assert_eq!(
        unsafe { ts_time_convert(ctx.handle, 2_451_544.5, 7, 1, &raw mut conversion) },
        Status::InvalidArg
    );
    assert_eq!(ctx.last_error().2.as_deref(), Some("from"));

    let mut delta = sized(
        TsDeltaT {
            struct_size: 0,
            source: 0,
            has_uncertainty: 0,
            reserved: [0; 2],
            seconds: 0.0,
            uncertainty_seconds: 0.0,
            model: ptr::null(),
        },
        |d, s| d.struct_size = s,
    );
    // SAFETY: as above.
    assert_eq!(
        unsafe { ts_time_delta_t(ctx.handle, 2_451_544.5, &raw mut delta) },
        Status::Ok
    );
    assert!((delta.seconds - 63.83).abs() < 0.02);
    assert_eq!(delta.source, TsDeltaTSource::Table as u8);
    // SAFETY: as above.
    assert_eq!(
        unsafe { ts_time_delta_t(ctx.handle, f64::NAN, &raw mut delta) },
        Status::InvalidArg
    );
}

fn render(ctx: &Ctx, key: &str, params: Option<&str>) -> (String, String, Vec<String>, bool) {
    let key = CString::new(key).unwrap();
    let params = params.map(|p| CString::new(p).unwrap());
    let mut blob = TsBlob::empty();
    // SAFETY: a live handle and valid pointers.
    let status = unsafe {
        ts_intl_render(
            ctx.handle,
            key.as_ptr(),
            params.as_ref().map_or(ptr::null(), |p| p.as_ptr()),
            &raw mut blob,
        )
    };
    assert_eq!(status, Status::Ok, "{:?}", ctx.last_error());
    // SAFETY: the library wrote `len` bytes.
    let bytes = unsafe { core::slice::from_raw_parts(blob.data, blob.len) }.to_vec();
    // SAFETY: a descriptor the library wrote.
    unsafe { ts_blob_free(&raw mut blob) };
    let schema = schemas::intl_render();
    let reader = Reader::parse(&bytes, &schema).unwrap();
    let flags = reader.fixed("flags").unwrap();
    let warnings: Vec<String> = serde_json::from_str(reader.text("warnings").unwrap()).unwrap();
    assert_eq!(flags[2].as_i64() as usize, warnings.len());
    (
        reader.text("text").unwrap().to_string(),
        reader.text("resolved_from").unwrap().to_string(),
        warnings,
        flags[0].as_i64() == 1,
    )
}

#[test]
fn the_locale_engine_renders_typed_parameters_in_nepali() {
    let ctx = Ctx::new(0, None, None, Some("ne-Deva-NP")).unwrap();
    let (text, from, warnings, fallback) = render(
        &ctx,
        "sdk.reason.grahaInBhava",
        Some(r#"{"graha": {"$entity": "graha.JUPITER"}, "bhava": 7}"#),
    );
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(from, "ne-Deva-NP");
    assert!(text.contains('७'), "{text}");
    assert!(!fallback);
    let (_, _, warnings, _) = render(&ctx, "sdk.reason.grahaInBhava", None);
    assert!(!warnings.is_empty());
    let (text, from, warnings, _) = render(&ctx, "sdk.nope.missing", None);
    assert!(from.is_empty() && !warnings.is_empty(), "{text}");

    let mut has = 9u8;
    let key = CString::new("sdk.reason.grahaInBhava").unwrap();
    // SAFETY: as above.
    assert_eq!(
        unsafe { ts_intl_has(ctx.handle, key.as_ptr(), &raw mut has) },
        Status::Ok
    );
    assert_eq!(has, 1);
    let en = CString::new("en-Latn").unwrap();
    // SAFETY: as above.
    assert_eq!(
        unsafe { ts_intl_set_locale(ctx.handle, en.as_ptr()) },
        Status::Ok
    );
    let (text, from, _, _) = render(
        &ctx,
        "sdk.reason.grahaInBhava",
        Some(r#"{"graha": {"$entity": "graha.JUPITER"}, "bhava": 7}"#),
    );
    assert_eq!(from, "en-Latn");
    assert!(text.contains("Jupiter"), "{text}");
    let bad = CString::new("fr-Latn").unwrap();
    // SAFETY: as above.
    assert_eq!(
        unsafe { ts_intl_set_locale(ctx.handle, bad.as_ptr()) },
        Status::Unsupported
    );
    assert!(ctx.last_error().3.unwrap().contains("sa-Deva"));
    let key = CString::new("sdk.reason.grahaInBhava").unwrap();
    let params = CString::new("[1]").unwrap();
    let mut blob = TsBlob::empty();
    // SAFETY: as above.
    assert_eq!(
        unsafe { ts_intl_render(ctx.handle, key.as_ptr(), params.as_ptr(), &raw mut blob) },
        Status::InvalidArg
    );
    assert_eq!(ctx.last_error().2.as_deref(), Some("params_json"));
}

#[test]
fn keys_parse_to_packed_ids_and_back_with_suggestions() {
    let ctx = Ctx::defaults();
    let key = CString::new("graha.SUN").unwrap();
    let mut id = 0u32;
    // SAFETY: a live handle and valid pointers.
    assert_eq!(
        unsafe { ts_key_parse(ctx.handle, key.as_ptr(), &raw mut id) },
        Status::Ok
    );
    assert_eq!(id, Graha::Sun.key_id().bits());
    let mut name = TsStr {
        data: ptr::null(),
        len: 0,
    };
    // SAFETY: as above.
    assert_eq!(
        unsafe { ts_key_name(ctx.handle, id, &raw mut name) },
        Status::Ok
    );
    assert_eq!(lent(name), "graha.SUN");
    assert_eq!(name.len, "graha.SUN".len());
    let wrong = CString::new("graha.SUNN").unwrap();
    // SAFETY: as above.
    assert_eq!(
        unsafe { ts_key_parse(ctx.handle, wrong.as_ptr(), &raw mut id) },
        Status::Unsupported
    );
    let (_, _, _, hint, detail) = ctx.last_error();
    assert_eq!(detail.as_deref(), Some("UNKNOWN_KEY"));
    assert!(hint.unwrap().contains("SUN"));
    // SAFETY: as above.
    assert_eq!(
        unsafe { ts_key_name(ctx.handle, 0xFFFF_FFFF, &raw mut name) },
        Status::Unsupported
    );
    // SAFETY: a null key.
    assert_eq!(
        unsafe { ts_key_parse(ctx.handle, ptr::null(), &raw mut id) },
        Status::InvalidArg
    );
    assert_eq!(ctx.last_error().2.as_deref(), Some("key"));
    // SAFETY: a null handle.
    assert_eq!(
        unsafe { ts_key_parse(ptr::null(), key.as_ptr(), &raw mut id) },
        Status::InvalidArg
    );
}

/// A shipped layout's row, as the boundary answers it.
fn layout_row(ctx: &Ctx, key: &str) -> Result<String, Record> {
    let key = CString::new(key).unwrap();
    let mut json = TsString::empty();
    // SAFETY: a live handle, a NUL-terminated key and a valid slot.
    match unsafe { ts_chart_layout_row(ctx.handle, key.as_ptr(), &raw mut json) } {
        Status::Ok => Ok(owned(json)),
        _ => Err(ctx.last_error()),
    }
}

/// A consumer's own layout, registered from a binding: read a shipped row,
/// rename it, register it, find its key and draw in it
/// (`03-design/chart-geometry.md` §7f).
#[test]
fn a_consumer_s_layout_is_registered_from_json_found_by_key_and_drawn() {
    let base = Ctx::new(TS_CONTEXT_TEST_PROVIDER, None, None, None).unwrap();
    let row = layout_row(&base, "chart_layout.SOUTH_INDIAN").unwrap();
    assert_eq!(
        layout_row(&base, "SOUTH_INDIAN").unwrap(),
        row,
        "bare or full"
    );
    let unknown = layout_row(&base, "ACME_KERALA").unwrap_err();
    assert_eq!(unknown.2.as_deref(), Some("key"));
    assert!(
        unknown
            .3
            .as_deref()
            .unwrap_or_default()
            .contains("NORTH_INDIAN")
    );

    let kerala = row.replacen("\"SOUTH_INDIAN\"", "\"ACME_KERALA\"", 1);
    let ctx = Ctx::with_layouts(&format!("[{kerala}]")).expect("a renamed row registers");
    assert_eq!(layout_row(&ctx, "ACME_KERALA").unwrap(), kerala);

    // The key resolves to a registered id, and the id back to the key.
    let full = CString::new("chart_layout.ACME_KERALA").unwrap();
    let mut id = 0u32;
    // SAFETY: a live handle and valid slots.
    assert_eq!(
        unsafe { ts_key_parse(ctx.handle, full.as_ptr(), &raw mut id) },
        Status::Ok
    );
    assert!(id & 0xFFFF >= 0x8000, "a registered id: {id:#x}");
    let mut name = TsStr {
        data: ptr::null(),
        len: 0,
    };
    // SAFETY: a live handle and a valid slot.
    assert_eq!(
        unsafe { ts_key_name(ctx.handle, id, &raw mut name) },
        Status::Ok
    );
    assert_eq!(lent(name), "chart_layout.ACME_KERALA");

    // Drawn by that id, the chart comes back placed in the consumer's row.
    let instants = [2_451_545.0];
    let drawings = [((id & 0xFFFF) << 16) | u32::from(Varga::D1.id())];
    // And a dasha beside the drawing, so the ragged sections cross too.
    let dashas = [DashaSystem::Vimshottari.id()];
    let request = sized(
        TsChartRequest {
            struct_size: 0,
            kind: 0,
            reserved: 0,
            instants: instants.as_ptr(),
            instant_count: instants.len(),
            latitude_deg: 27.7172,
            longitude_deg: 85.324,
            altitude_m: 1400.0,
            utc_offset_seconds: 20_700,
            reserved_tail: 0,
            sections: 0,
            reserved_sections: 0,
            vargas: ptr::null(),
            varga_count: 0,
            drawings: drawings.as_ptr(),
            drawing_count: drawings.len(),
            dashas: dashas.as_ptr(),
            dasha_count: dashas.len(),
            theme_json: ptr::null(),
            rules_json: ptr::null(),
            interpret_json: ptr::null(),
        },
        |r, s| r.struct_size = s,
    );
    let mut blob = TsBlob::empty();
    // SAFETY: a live context, a valid request and a valid slot.
    assert_eq!(
        unsafe { ts_chart_found(ctx.handle, &raw const request, &raw mut blob) },
        Status::Ok,
        "{:?}",
        ctx.last_error()
    );
    // SAFETY: the library wrote `len` bytes.
    let bytes = unsafe { core::slice::from_raw_parts(blob.data, blob.len) }.to_vec();
    // SAFETY: a descriptor the library wrote.
    unsafe { ts_blob_free(&raw mut blob) };
    let schema = schemas::charts();
    let reader = Reader::parse(&bytes, &schema).unwrap();
    let drawn = reader.text("drawings").unwrap();
    assert!(drawn.contains("\"layout\":\"ACME_KERALA\""), "{drawn}");

    // One dasha row, whose periods are the next `period_count` rows: nine
    // mahadashas, 81 antardashas and 729 below them at the default depth.
    assert_eq!(
        reader.fixed("summary").unwrap()[4].as_i64(),
        1,
        "dasha_count"
    );
    assert_eq!(reader.count("dashas"), Some(1));
    let period_count = reader.column("dashas", "period_count").unwrap()[0].as_i64();
    assert_eq!(period_count, 9 + 81 + 729);
    assert_eq!(reader.count("dasha_periods"), Some(819));
    let levels = reader.column("dasha_periods", "level").unwrap();
    assert_eq!(
        (levels[0].as_i64(), levels[1].as_i64(), levels[2].as_i64()),
        (1, 2, 3)
    );
    let from = reader.column("dasha_periods", "from_jd").unwrap();
    assert_eq!(
        from[0].as_f64(),
        2_451_545.0,
        "the first mahadasha runs from birth"
    );

    // A row is refused by its place and field: a shipped key, a misspelt
    // field, and a row the checks refuse.
    let refused = |rows: String| match Ctx::with_layouts(&rows) {
        Ok(_) => panic!("{rows} registered"),
        Err(record) => record,
    };
    let shipped = refused(format!("[{kerala}, {row}]"));
    assert_eq!(shipped.2.as_deref(), Some("options.layouts_json[1].key"));
    // A misspelt extra field is named by its path; a misspelt required one
    // is refused as the field it is missing.
    let extra = kerala.replacen(
        "\"direction\"",
        "\"heading\":\"clockwise\",\"direction\"",
        1,
    );
    let typo = refused(format!("[{extra}]"));
    assert_eq!(
        typo.2.as_deref(),
        Some("options.layouts_json[0].shape.heading"),
        "{typo:?}"
    );
    let missing = refused(format!(
        "[{}]",
        kerala.replacen("\"direction\"", "\"heading\"", 1)
    ));
    assert!(missing.1.contains("direction"), "{missing:?}");
    // A row the checks refuse: two cells holding Pisces.
    let twice = kerala.replacen("\"value\":\"ARIES\"", "\"value\":\"PISCES\"", 1);
    let invalid = refused(format!("[{twice}]"));
    assert!(
        invalid
            .2
            .as_deref()
            .is_some_and(|field| field.starts_with("options.layouts_json[0].shape.cells[")),
        "{invalid:?}"
    );
    let not_rows = refused(String::from("{}"));
    assert_eq!(not_rows.2.as_deref(), Some("options.layouts_json"));
}

/// A consumer's dasha system crosses whole: registered through
/// `options.dashas_json`, named by `ts_key_parse`, asked for by that id, and
/// answered in the `dashas` section under the same id with its own periods; a
/// definition the row's checks refuse is named by its place and field.
#[test]
fn a_consumer_dasha_system_registers_and_crosses_by_its_id() {
    let saptaka = r#"{"key":"ACME_SAPTAKA","lords":[
        {"graha":"SUN","years":10},{"graha":"MOON","years":10},{"graha":"MARS","years":10},
        {"graha":"MERCURY","years":10},{"graha":"JUPITER","years":10},{"graha":"VENUS","years":10},
        {"graha":"SATURN","years":10}],"reference":"KRITTIKA"}"#;
    let ctx = Ctx::with_dashas(&format!("[{saptaka}]")).expect("a consumer's system registers");
    let full = CString::new("dasha_system.ACME_SAPTAKA").unwrap();
    let mut id = 0u32;
    // SAFETY: a live handle and valid slots.
    assert_eq!(
        unsafe { ts_key_parse(ctx.handle, full.as_ptr(), &raw mut id) },
        Status::Ok,
        "{:?}",
        ctx.last_error()
    );
    assert_eq!(id & 0xFFFF, 0x8000, "the first registered id");

    let instants = [2_451_545.0];
    let dashas = [
        u16::try_from(id & 0xFFFF).unwrap(),
        DashaSystem::Vimshottari.id(),
    ];
    let request = sized(
        TsChartRequest {
            struct_size: 0,
            kind: 0,
            reserved: 0,
            instants: instants.as_ptr(),
            instant_count: instants.len(),
            latitude_deg: 27.7172,
            longitude_deg: 85.324,
            altitude_m: 1400.0,
            utc_offset_seconds: 20_700,
            reserved_tail: 0,
            sections: 0,
            reserved_sections: 0,
            vargas: ptr::null(),
            varga_count: 0,
            drawings: ptr::null(),
            drawing_count: 0,
            dashas: dashas.as_ptr(),
            dasha_count: dashas.len(),
            theme_json: ptr::null(),
            rules_json: ptr::null(),
            interpret_json: ptr::null(),
        },
        |r, s| r.struct_size = s,
    );
    let mut blob = TsBlob::empty();
    // SAFETY: a live context, a valid request and a valid slot.
    assert_eq!(
        unsafe { ts_chart_found(ctx.handle, &raw const request, &raw mut blob) },
        Status::Ok,
        "{:?}",
        ctx.last_error()
    );
    // SAFETY: the library wrote `len` bytes.
    let bytes = unsafe { core::slice::from_raw_parts(blob.data, blob.len) }.to_vec();
    // SAFETY: a descriptor the library wrote.
    unsafe { ts_blob_free(&raw mut blob) };
    let schema = schemas::charts();
    let reader = Reader::parse(&bytes, &schema).unwrap();
    let systems = reader.column("dashas", "system").unwrap();
    assert_eq!(
        (systems[0].as_i64(), systems[1].as_i64()),
        (0x8000, i64::from(DashaSystem::Vimshottari.id()))
    );
    // Seven lords of ten years: seven mahadashas, 49 below them and 343 below
    // those at the default depth of three.
    let counts = reader.column("dashas", "period_count").unwrap();
    assert_eq!(counts[0].as_i64(), 7 + 49 + 343);

    // An id nothing registered is refused by its place in the request.
    let stray = [0x8001_u16];
    let asked = TsChartRequest {
        dashas: stray.as_ptr(),
        dasha_count: stray.len(),
        ..request
    };
    let mut none = TsBlob::empty();
    // SAFETY: as above.
    let status = unsafe { ts_chart_found(ctx.handle, &raw const asked, &raw mut none) };
    assert_eq!(status, Status::InvalidArg);
    let record = ctx.last_error();
    assert_eq!(record.2.as_deref(), Some("dashas[0]"), "{record:?}");
    assert!(
        record
            .3
            .as_deref()
            .unwrap_or_default()
            .contains("ACME_SAPTAKA"),
        "{record:?}"
    );

    // A definition the row's checks refuse, and one taking a catalogued key.
    let refused = |rows: String| match Ctx::with_dashas(&rows) {
        Ok(_) => panic!("{rows} registered"),
        Err(record) => record,
    };
    let narrow = refused(format!(
        "[{}]",
        saptaka.replacen("\"KRITTIKA\"", "\"KRITTIKA\",\"span\":0", 1)
    ));
    assert_eq!(
        narrow.2.as_deref(),
        Some("options.dashas_json[0].span"),
        "{narrow:?}"
    );
    let taken = refused(format!(
        "[{}]",
        saptaka.replacen("ACME_SAPTAKA", "VIMSHOTTARI", 1)
    ));
    assert_eq!(
        taken.2.as_deref(),
        Some("options.dashas_json[0].key"),
        "{taken:?}"
    );
    let typo = refused(format!(
        "[{}]",
        saptaka.replacen("\"reference\"", "\"refrence\"", 1)
    ));
    assert!(
        typo.1.contains("reference") || typo.1.contains("refrence"),
        "{typo:?}"
    );
}

/// A chart request's `rules_json` answers rules over every chart in the same
/// crossing: section `rules` carries each chart's present rules by key, and
/// the longevity readings when asked; a rule that does not read is refused by
/// its place from the request's root (`03-design/rules-at-the-boundary.md`).
#[test]
fn a_chart_request_answers_rules_in_the_same_crossing() {
    let ctx = Ctx::with_ephemeris(
        0,
        TsEphemeris::Builtin,
        Some("conformance-baseline"),
        None,
        None,
    )
    .unwrap();
    let instants = [2_447_995.489_583_333_5, 2_451_545.0];
    let rules = CString::new(r#"{"shipped": ["nabhasas"], "longevity": true}"#).unwrap();
    let request = sized(
        TsChartRequest {
            struct_size: 0,
            kind: 0,
            reserved: 0,
            instants: instants.as_ptr(),
            instant_count: instants.len(),
            latitude_deg: 27.7172,
            longitude_deg: 85.324,
            altitude_m: 1400.0,
            utc_offset_seconds: 20_700,
            reserved_tail: 0,
            sections: 0,
            reserved_sections: 0,
            vargas: ptr::null(),
            varga_count: 0,
            drawings: ptr::null(),
            drawing_count: 0,
            dashas: ptr::null(),
            dasha_count: 0,
            theme_json: ptr::null(),
            rules_json: rules.as_ptr(),
            interpret_json: ptr::null(),
        },
        |r, s| r.struct_size = s,
    );
    let mut blob = TsBlob::empty();
    // SAFETY: a live context, a valid request and a valid slot.
    assert_eq!(
        unsafe { ts_chart_found(ctx.handle, &raw const request, &raw mut blob) },
        Status::Ok,
        "{:?}",
        ctx.last_error()
    );
    // SAFETY: the library wrote `len` bytes.
    let bytes = unsafe { core::slice::from_raw_parts(blob.data, blob.len) }.to_vec();
    // SAFETY: a descriptor the library wrote.
    unsafe { ts_blob_free(&raw mut blob) };
    let schema = schemas::charts();
    let reader = Reader::parse(&bytes, &schema).unwrap();
    let rules_json: serde_json::Value =
        serde_json::from_slice(reader.bytes("rules").unwrap()).unwrap();
    let per_chart = rules_json.as_array().unwrap();
    assert_eq!(per_chart.len(), 2, "one entry a chart");
    for chart in per_chart {
        let present = chart["present"].as_array().unwrap();
        assert!(!present.is_empty());
        for held in present {
            // A rule by its key, never the whole rule again.
            assert!(held["rule"].is_string(), "{held}");
            assert_eq!(held["result"]["present"], serde_json::Value::Bool(true));
        }
        assert!(chart["longevity"]["ayurdaya"]["pindayu"]["years"].is_number());
        assert!(chart.get("houses").is_none(), "houses were not asked for");
    }
    // The same request without rules carries an empty section.
    let plain = TsChartRequest {
        rules_json: ptr::null(),
        interpret_json: ptr::null(),
        ..request
    };
    let mut none = TsBlob::empty();
    // SAFETY: as above.
    assert_eq!(
        unsafe { ts_chart_found(ctx.handle, &raw const plain, &raw mut none) },
        Status::Ok
    );
    // SAFETY: as above.
    let plain_bytes = unsafe { core::slice::from_raw_parts(none.data, none.len) }.to_vec();
    // SAFETY: as above.
    unsafe { ts_blob_free(&raw mut none) };
    let plain_reader = Reader::parse(&plain_bytes, &schema).unwrap();
    assert!(plain_reader.bytes("rules").unwrap().is_empty());

    // A rule that does not read is refused from the request's root.
    let broken = CString::new(r#"{"rules": [{"key": "X", "category": "raja"}]}"#).unwrap();
    let refused = TsChartRequest {
        rules_json: broken.as_ptr(),
        interpret_json: ptr::null(),
        ..request
    };
    let mut nothing = TsBlob::empty();
    // SAFETY: as above.
    let status = unsafe { ts_chart_found(ctx.handle, &raw const refused, &raw mut nothing) };
    assert_eq!(status, Status::InvalidArg);
    let record = ctx.last_error();
    assert_eq!(
        record.2.as_deref(),
        Some("rules_json.rules[0]"),
        "{record:?}"
    );
}

/// A chart request composes narrative plans in the same crossing: the
/// placements and the readings of every chart, each holding no words, each
/// item's params the very JSON `ts_intl_render` takes — so the test says one
/// by handing an item straight back to the renderer, which is the property
/// the crossing exists for. A reading without rules is refused by name, and
/// a composer that is not one is refused beside the composers there are
/// (`03-design/plans-at-the-boundary.md`).
#[test]
fn a_chart_request_composes_plans_in_the_same_crossing_and_renders_them() {
    let ctx = Ctx::with_ephemeris(
        0,
        TsEphemeris::Builtin,
        Some("conformance-baseline"),
        None,
        None,
    )
    .unwrap();
    let instants = [2_447_995.489_583_333_5, 2_451_545.0];
    let rules = CString::new(r#"{"shipped": ["nabhasas"]}"#).unwrap();
    let plans = CString::new(
        r#"{"placements": true, "readings": true, "strength": true, "houses": true,
            "positions": true, "aspects": true, "conditions": true, "karakas": true}"#,
    )
    .unwrap();
    let request = sized(
        TsChartRequest {
            struct_size: 0,
            kind: 0,
            reserved: 0,
            instants: instants.as_ptr(),
            instant_count: instants.len(),
            latitude_deg: 27.7172,
            longitude_deg: 85.324,
            altitude_m: 1400.0,
            utc_offset_seconds: 20_700,
            reserved_tail: 0,
            sections: 0,
            reserved_sections: 0,
            vargas: ptr::null(),
            varga_count: 0,
            drawings: ptr::null(),
            drawing_count: 0,
            dashas: ptr::null(),
            dasha_count: 0,
            theme_json: ptr::null(),
            rules_json: rules.as_ptr(),
            interpret_json: plans.as_ptr(),
        },
        |r, s| r.struct_size = s,
    );
    let mut blob = TsBlob::empty();
    // SAFETY: a live context, a valid request and a valid slot.
    assert_eq!(
        unsafe { ts_chart_found(ctx.handle, &raw const request, &raw mut blob) },
        Status::Ok,
        "{:?}",
        ctx.last_error()
    );
    // SAFETY: the library wrote `len` bytes.
    let bytes = unsafe { core::slice::from_raw_parts(blob.data, blob.len) }.to_vec();
    // SAFETY: a descriptor the library wrote.
    unsafe { ts_blob_free(&raw mut blob) };
    let schema = schemas::charts();
    let reader = Reader::parse(&bytes, &schema).unwrap();
    let composed: serde_json::Value =
        serde_json::from_slice(reader.bytes("plans").unwrap()).unwrap();
    let per_chart = composed.as_array().unwrap();
    assert_eq!(per_chart.len(), 2, "one entry a chart");

    let mut said = 0;
    for chart in per_chart {
        let placements = chart["placements"].as_array().unwrap();
        assert!(!placements.is_empty(), "every chart places its grahas");
        assert!(chart["readings"].is_array(), "asked for, so present");
        // The strengths read the Shadbala, and `sections` never asked for
        // it: a composer's own section is computed for it, as a rule's is.
        let weighed = chart["strength"].as_array().unwrap();
        assert_eq!(
            weighed.len(),
            7 * 2,
            "the seven grahas, a score and a sufficiency each"
        );
        // And the houses read the bhavas, which `sections` never asked for
        // either.
        let lords = chart["houses"].as_array().unwrap();
        assert_eq!(lords.len(), 12 * 2, "a sign and a lord each");
        // And the positions read the same states the placements do, so one
        // request computing them serves both composers.
        let degrees = chart["positions"].as_array().unwrap();
        assert_eq!(degrees.len(), 10, "the lagna, then the nine grahas");
        // And the drishtis read the aspects section, which `sections` never
        // asked for either.
        let looks = chart["aspects"].as_array().unwrap();
        assert!(!looks.is_empty(), "every chart holds a drishti");
        // The conditions and the karakas read the graha states through
        // `RuleInputs`, as the placements do, so the same section serves
        // four composers and `sections` asked for none of it.
        let conditions = chart["conditions"].as_array().unwrap();
        assert!(
            conditions.len() >= 9 * 2,
            "a dignity and a navamsha for each of the nine"
        );
        let karakas = chart["karakas"].as_array().unwrap();
        assert!(!karakas.is_empty(), "every chart ranks its chara karakas");
        for item in placements
            .iter()
            .chain(chart["readings"].as_array().unwrap())
            .chain(weighed)
            .chain(lords)
            .chain(degrees)
            .chain(looks)
            .chain(conditions)
            .chain(karakas)
        {
            // A plan holds keys and slots, never a rendered word.
            let key = item["key"].as_str().unwrap();
            assert!(key.starts_with("sdk."), "{item}");
            // And its params are the renderer's own: handed straight back,
            // with nothing in between, they say the item.
            let key = CString::new(key).unwrap();
            let params = CString::new(serde_json::to_string(&item["params"]).unwrap()).unwrap();
            let mut rendered = TsBlob::empty();
            // SAFETY: a live context, two NUL-terminated strings, a valid slot.
            assert_eq!(
                unsafe {
                    ts_intl_render(ctx.handle, key.as_ptr(), params.as_ptr(), &raw mut rendered)
                },
                Status::Ok,
                "{:?}",
                ctx.last_error()
            );
            // SAFETY: the library wrote `len` bytes.
            let said_bytes =
                unsafe { core::slice::from_raw_parts(rendered.data, rendered.len) }.to_vec();
            // SAFETY: a descriptor the library wrote.
            unsafe { ts_blob_free(&raw mut rendered) };
            let render_schema = schemas::intl_render();
            let render_reader = Reader::parse(&said_bytes, &render_schema).unwrap();
            assert!(
                !render_reader.text("text").unwrap().is_empty(),
                "{key:?} said nothing"
            );
            // A fallback would mean the locale does not carry the key, and a
            // warning that a slot did not fit it.
            let flags = render_reader.fixed("flags").unwrap();
            assert_eq!(flags[0].as_i64(), 0, "{key:?} fell back");
            assert_eq!(flags[2].as_i64(), 0, "{key:?} warned");
            said += 1;
        }
    }
    assert!(said > 20, "only {said} items said");

    // The same request without a composer carries an empty section.
    let plain = TsChartRequest {
        interpret_json: ptr::null(),
        ..request
    };
    let mut none = TsBlob::empty();
    // SAFETY: as above.
    assert_eq!(
        unsafe { ts_chart_found(ctx.handle, &raw const plain, &raw mut none) },
        Status::Ok
    );
    // SAFETY: as above.
    let plain_bytes = unsafe { core::slice::from_raw_parts(none.data, none.len) }.to_vec();
    // SAFETY: as above.
    unsafe { ts_blob_free(&raw mut none) };
    assert!(
        Reader::parse(&plain_bytes, &schema)
            .unwrap()
            .bytes("plans")
            .unwrap()
            .is_empty()
    );

    // A reading needs rules to say what they answered.
    let alone = CString::new(r#"{"readings": true}"#).unwrap();
    let refused = TsChartRequest {
        rules_json: ptr::null(),
        interpret_json: alone.as_ptr(),
        ..request
    };
    let mut nothing = TsBlob::empty();
    // SAFETY: as above.
    let status = unsafe { ts_chart_found(ctx.handle, &raw const refused, &raw mut nothing) };
    assert_eq!(status, Status::InvalidArg);
    let record = ctx.last_error();
    assert_eq!(
        record.2.as_deref(),
        Some("interpret_json.readings"),
        "{record:?}"
    );

    // And a composer that is not one is refused beside the ones that are.
    let typo = CString::new(r#"{"readigns": true}"#).unwrap();
    let wrong = TsChartRequest {
        interpret_json: typo.as_ptr(),
        ..request
    };
    let mut never = TsBlob::empty();
    // SAFETY: as above.
    let status = unsafe { ts_chart_found(ctx.handle, &raw const wrong, &raw mut never) };
    assert_eq!(status, Status::InvalidArg);
    let record = ctx.last_error();
    assert_eq!(record.2.as_deref(), Some("interpret_json"), "{record:?}");
    assert!(
        record.1.contains("placements")
            && record.1.contains("readings")
            && record.1.contains("strength")
            && record.1.contains("houses")
            && record.1.contains("positions")
            && record.1.contains("aspects")
            && record.1.contains("conditions")
            && record.1.contains("karakas"),
        "{record:?}"
    );
}
