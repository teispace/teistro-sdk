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
use teistro_ffi::chart::{
    TsChartRequest, TsKakshyaLord, TsSaham, TsSahamStrong, TsSahamWeak, TsSarvaStanding,
    TsVarsheshaChosen, TsYearYoga, ts_chart_found, ts_chart_layout_row,
};
use teistro_ffi::context::{
    TsContext, TsContextOptions, TsEphemeris, TsError, ts_context_free, ts_context_last_error,
    ts_context_new, ts_context_profile, ts_context_settings_hash, ts_context_settings_json,
    ts_error_free,
};
use teistro_ffi::ephemeris::{ts_ephemeris_call, ts_ephemeris_manifest};
use teistro_ffi::intl::{ts_intl_has, ts_intl_locale, ts_intl_render, ts_intl_set_locale};
use teistro_ffi::key::{ts_key_name, ts_key_parse};
use teistro_ffi::panchanga::{
    TS_PANCHANGA_ECLIPSES, TS_PANCHANGA_NEPAL_SAMBAT, TS_PANCHANGA_YEARS, TsPanchangaRequest,
    ts_panchanga_days,
};
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
use teistro_idl::blob::{Reader, ScalarValue};
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

/// A chart request for `instants` at a place and clock, asking for
/// nothing beside the charts: a test sets the one record it reads with
/// `..chart_request(...)`.
fn chart_request(
    instants: &[f64],
    (latitude_deg, longitude_deg): (f64, f64),
    utc_offset_seconds: i32,
) -> TsChartRequest {
    sized(
        TsChartRequest {
            struct_size: 0,
            kind: 0,
            reserved: 0,
            instants: instants.as_ptr(),
            instant_count: instants.len(),
            latitude_deg,
            longitude_deg,
            altitude_m: 0.0,
            utc_offset_seconds,
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
            rules_json: ptr::null(),
            interpret_json: ptr::null(),
            varsha_json: ptr::null(),
            gochar_json: ptr::null(),
            hits_json: ptr::null(),
            sade_sati_json: ptr::null(),
            kp_json: ptr::null(),
            dignities_json: ptr::null(),
            fortitudes_json: ptr::null(),
            lots_json: ptr::null(),
            considerations_json: ptr::null(),
            perfection_json: ptr::null(),
            progressions_json: ptr::null(),
        },
        |r, s| r.struct_size = s,
    )
}

/// The blob a chart request answers, or the status it was refused with.
fn chart_blob(ctx: &Ctx, asked: &TsChartRequest) -> Result<Vec<u8>, Status> {
    let mut blob = TsBlob::empty();
    // SAFETY: a live context, a valid request and a valid slot.
    let status = unsafe { ts_chart_found(ctx.handle, asked, &raw mut blob) };
    let bytes = (status == Status::Ok).then(|| {
        // SAFETY: the library wrote `len` bytes.
        unsafe { core::slice::from_raw_parts(blob.data, blob.len) }.to_vec()
    });
    // SAFETY: a descriptor the library wrote, or the empty one.
    unsafe { ts_blob_free(&raw mut blob) };
    bytes.ok_or(status)
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
        (Status::InvalidArg, Some("settings.frame.zodiacs"))
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
        serde_json::from_str(reader.text("provenance_json").unwrap()).unwrap();
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
        hint.contains("`BUILTIN`") && hint.contains("descriptor"),
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

/// What one crossing of `ts_intl_render` carried back.
struct Render {
    text: String,
    from: String,
    warnings: Vec<String>,
    fallback: bool,
    /// The `parts` section as it crossed: `[]` when the message has no
    /// markup, which is the boundary's own rule.
    parts: String,
}

fn render(ctx: &Ctx, key: &str, params: Option<&str>) -> Render {
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
    Render {
        text: reader.text("text").unwrap().to_string(),
        from: reader.text("resolved_from").unwrap().to_string(),
        warnings,
        fallback: flags[0].as_i64() == 1,
        parts: reader.text("parts").unwrap().to_string(),
    }
}

/// A message's **markup** crosses, or a rich renderer cannot exist.
///
/// `sdk.reason.lordship` is the corpus's most-said reason and one of the
/// two shipped messages that carry `{#b}`. Every binding builds its
/// renderer on these bytes, so the shape is asserted whole rather than
/// probed: a reader in another language has nothing else to go on.
#[test]
fn a_rich_message_carries_its_parts_across_and_a_plain_one_carries_none() {
    let ctx = Ctx::new(0, None, None, Some("en-Latn")).unwrap();
    let rich = render(
        &ctx,
        "sdk.reason.lordship",
        Some(r#"{"graha": {"$entity": "graha.JUPITER"}, "bhava": 5}"#),
    );
    assert!(rich.warnings.is_empty(), "{:?}", rich.warnings);
    assert_eq!(rich.text, "Jupiter rules house 5");
    assert_eq!(
        rich.parts,
        r#"[{"type":"markup","kind":"open","name":"b","options":{}},{"type":"text","value":"Jupiter"},{"type":"markup","kind":"close","name":"b","options":{}},{"type":"text","value":" rules house 5"}]"#
    );
    // The text parts joined are the text, so a renderer that knows no
    // tag can drop every markup part and lose nothing.
    let parts: Vec<serde_json::Value> = serde_json::from_str(&rich.parts).unwrap();
    let joined: String = parts
        .iter()
        .filter(|part| part["type"] == "text")
        .map(|part| part["value"].as_str().unwrap())
        .collect();
    assert_eq!(joined, rich.text);

    // A message with no markup sends none: the parts would be the text
    // written a second time, and every binding makes the one text part
    // for itself.
    let plain = render(
        &ctx,
        "sdk.reason.grahaInBhava",
        Some(r#"{"graha": {"$entity": "graha.JUPITER"}, "bhava": 7}"#),
    );
    assert_eq!(plain.parts, "[]");
    assert!(!plain.text.is_empty(), "`plain.text` is empty");
}

#[test]
fn the_locale_engine_renders_typed_parameters_in_nepali() {
    let ctx = Ctx::new(0, None, None, Some("ne-Deva-NP")).unwrap();
    let said = render(
        &ctx,
        "sdk.reason.grahaInBhava",
        Some(r#"{"graha": {"$entity": "graha.JUPITER"}, "bhava": 7}"#),
    );
    assert!(said.warnings.is_empty(), "{:?}", said.warnings);
    assert_eq!(said.from, "ne-Deva-NP");
    assert!(said.text.contains('७'), "{}", said.text);
    assert!(!said.fallback);
    assert!(
        !render(&ctx, "sdk.reason.grahaInBhava", None)
            .warnings
            .is_empty(),
        "`render(&ctx, \"sdk.reason.grahaInBhava\", None).warnings` is empty"
    );
    let said = render(&ctx, "sdk.nope.missing", None);
    assert!(
        said.from.is_empty() && !said.warnings.is_empty(),
        "{}",
        said.text
    );

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
    let said = render(
        &ctx,
        "sdk.reason.grahaInBhava",
        Some(r#"{"graha": {"$entity": "graha.JUPITER"}, "bhava": 7}"#),
    );
    assert_eq!(said.from, "en-Latn");
    assert!(said.text.contains("Jupiter"), "{}", said.text);
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
            varsha_json: ptr::null(),
            gochar_json: ptr::null(),
            hits_json: ptr::null(),
            sade_sati_json: ptr::null(),
            kp_json: ptr::null(),
            dignities_json: ptr::null(),
            fortitudes_json: ptr::null(),
            lots_json: ptr::null(),
            considerations_json: ptr::null(),
            perfection_json: ptr::null(),
            progressions_json: ptr::null(),
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
        "\"heading\":\"CLOCKWISE\",\"direction\"",
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
    let saptaka = r#"{"kernel":"UDU","key":"ACME_SAPTAKA","lords":[
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
            varsha_json: ptr::null(),
            gochar_json: ptr::null(),
            hits_json: ptr::null(),
            sade_sati_json: ptr::null(),
            kp_json: ptr::null(),
            dignities_json: ptr::null(),
            fortitudes_json: ptr::null(),
            lots_json: ptr::null(),
            considerations_json: ptr::null(),
            perfection_json: ptr::null(),
            progressions_json: ptr::null(),
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

    // The kernel is stated, so a row that names none is refused by that
    // field rather than by one the caller never wrote.
    let unstated = refused(format!(
        "[{}]",
        saptaka.replacen("\"kernel\":\"UDU\",", "", 1)
    ));
    assert!(unstated.1.contains("kernel"), "{unstated:?}");
}

/// The transits cross: a request's `gochar_json` answers every chart's
/// readings in the `gochar` section, a row a chart an instant, and their
/// grahas in `gochar_grahas`, each the façade's own reading of that chart
/// (`03-design/gochar.md`).
#[test]
fn a_chart_request_answers_the_transits() {
    use teistro::gochar::{Fruition, Verdict};
    let ctx = Ctx::with_ephemeris(
        0,
        TsEphemeris::Builtin,
        Some("conformance-baseline"),
        None,
        None,
    )
    .unwrap();
    let births = [2_447_995.489_583_333_5, 2_451_545.0];
    let text = r#"{"instants":[2460676.5,2460706.5,2460736.5],"from":"LAGNA","ashtakavarga":true}"#;
    let gochar = CString::new(text).unwrap();
    let request = sized(
        TsChartRequest {
            struct_size: 0,
            kind: 0,
            reserved: 0,
            instants: births.as_ptr(),
            instant_count: births.len(),
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
            rules_json: ptr::null(),
            interpret_json: ptr::null(),
            varsha_json: ptr::null(),
            gochar_json: gochar.as_ptr(),
            hits_json: ptr::null(),
            sade_sati_json: ptr::null(),
            kp_json: ptr::null(),
            dignities_json: ptr::null(),
            fortitudes_json: ptr::null(),
            lots_json: ptr::null(),
            considerations_json: ptr::null(),
            perfection_json: ptr::null(),
            progressions_json: ptr::null(),
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
    let column = |section: &str, name: &str| reader.column(section, name).unwrap();

    // The façade's own readings of the same births, to hold each cell to.
    let sdk = teistro::Context::builder()
        .profile("conformance-baseline")
        .ephemeris([teistro::Ephemeris::Builtin])
        .build()
        .unwrap();
    let place = teistro::quantity::Place::try_from_degrees(27.7172, 85.324, 1400.0).unwrap();
    let natal = sdk
        .chart()
        .readings(
            &births.map(teistro::quantity::JulianDay::<teistro::quantity::Utc>::literal),
            &teistro::ChartRequest::at(
                place,
                teistro::UtcOffset::try_from_seconds(20_700).unwrap(),
            ),
        )
        .unwrap()
        .value;
    let asked = teistro::GocharRequest::from_json(text).unwrap();
    let expected: Vec<_> = natal
        .iter()
        .flat_map(|document| sdk.chart().gochar(document, &asked).unwrap().value)
        .collect();

    // Fixed, not ragged: two charts of three instants, nine grahas each.
    let instant = column("gochar", "instant");
    assert_eq!(instant.len(), 6);
    assert_eq!(column("gochar_grahas", "graha").len(), 54);
    let (reference, from) = (
        column("gochar", "reference"),
        column("gochar", "counted_from"),
    );
    let grahas = [
        "graha",
        "sign",
        "degrees",
        "house",
        "good_house",
        "vedha_house",
        "obstructed_by",
        "verdict",
        "fruition",
        "fruitful_now",
    ]
    .map(|name| column("gochar_grahas", name));
    for (row, reading) in expected.iter().enumerate() {
        assert_eq!(instant[row].as_f64(), asked.instants()[row % 3].get());
        assert_eq!(
            reference[row].as_i64(),
            i64::from(reading.reference.sign.id())
        );
        assert_eq!(from[row].as_i64(), 1, "LAGNA");
        for (g, read) in reading.grahas.iter().enumerate() {
            let at = row * 9 + g;
            let cell = |c: usize| grahas[c][at].as_i64();
            assert_eq!(cell(0), i64::from(read.graha.id()));
            assert_eq!(cell(1), i64::from(read.transit.sign.id()));
            assert_eq!(grahas[2][at].as_f64(), read.transit.degrees);
            assert_eq!(cell(3), i64::from(read.house));
            assert_eq!(cell(4), i64::from(read.good_house));
            assert_eq!(cell(5), i64::from(read.vedha_house.unwrap_or(0)));
            let mask = read
                .obstructed_by
                .iter()
                .fold(0, |bits, graha| bits | 1 << graha.id());
            assert_eq!(cell(6), mask);
            let verdict = [Verdict::Good, Verdict::Obstructed, Verdict::NotGood];
            assert_eq!(verdict[usize::try_from(cell(7)).unwrap()], read.verdict);
            let fruition = [
                Fruition::First,
                Fruition::Middle,
                Fruition::Last,
                Fruition::Throughout,
            ];
            assert_eq!(fruition[usize::try_from(cell(8)).unwrap()], read.fruition);
            assert_eq!(cell(9), i64::from(read.fruitful_now));
        }
    }

    // The seven under each row, judged by the natal Ashtakavarga.
    let av = [
        "graha",
        "bindus",
        "good",
        "kakshya",
        "kakshya_lord",
        "kakshya_bindu",
        "sarva",
        "sarva_standing",
    ]
    .map(|name| column("gochar_ashtakavarga", name));
    assert_eq!(av[0].len(), 6 * 7);
    let good_from = column("gochar", "ashtakavarga_good_from");
    for (row, reading) in expected.iter().enumerate() {
        assert_eq!(good_from[row].as_i64(), 0, "FIVE");
        for (g, read) in reading.ashtakavarga.unwrap().iter().enumerate() {
            let at = row * 7 + g;
            let cell = |c: usize| av[c][at].as_i64();
            assert_eq!(cell(0), i64::from(read.graha.id()));
            assert_eq!(cell(1), i64::from(read.bindus));
            assert_eq!(cell(2), i64::from(read.good));
            assert_eq!(cell(3), i64::from(read.kakshya.index));
            assert_eq!(
                cell(4),
                i64::from(TsKakshyaLord::from(read.kakshya.lord) as u8)
            );
            assert_eq!(cell(5), i64::from(read.kakshya_bindu));
            assert_eq!(cell(6), i64::from(read.sarva));
            assert_eq!(
                cell(7),
                i64::from(TsSarvaStanding::from(read.sarva_standing) as u8)
            );
        }
    }

    // A request that names no instant is refused from the field the caller
    // wrote, and so is a reference nobody counts from.
    let refused = |json: &str| {
        let text = CString::new(json).unwrap();
        let asked = TsChartRequest {
            gochar_json: text.as_ptr(),
            hits_json: ptr::null(),
            ..request
        };
        let mut out = TsBlob::empty();
        // SAFETY: as above.
        let status = unsafe { ts_chart_found(ctx.handle, &raw const asked, &raw mut out) };
        assert_eq!(status, Status::InvalidArg, "{json}");
        ctx.last_error()
    };
    let empty = refused(r#"{"instants":[]}"#);
    assert_eq!(empty.2.as_deref(), Some("gochar.instants"), "{empty:?}");
    let unknown = refused(r#"{"instants":[2460676.5],"from":"SUN"}"#);
    assert!(
        unknown
            .2
            .as_deref()
            .is_some_and(|f| f.starts_with("gochar")),
        "{unknown:?}"
    );
}

/// The transit hit list crosses: a request's `hits_json` answers every
/// chart's list in the `hits` section, ragged by `cast.hit_count`, each
/// row the façade's own hit for that chart, and a refusal names the field
/// the caller wrote (`03-design/transit-hit-list.md`).
#[test]
fn a_chart_request_answers_the_hit_list() {
    use teistro::gochar::hits::HitEvent;
    use teistro_ffi::chart::{TsAspectPhase, TsHitKind, TsMotion};
    let ctx = Ctx::with_ephemeris(
        0,
        TsEphemeris::Builtin,
        Some("conformance-baseline"),
        None,
        None,
    )
    .unwrap();
    let births = [2_447_995.489_583_333_5, 2_451_545.0];
    let text = r#"{"from":2460676.5,"to":2460866.5,"grahas":["SUN","MERCURY","SATURN","KETU"],"aspects":[0,90,180],"orbDeg":2}"#;
    let hits = CString::new(text).unwrap();
    let request = sized(
        TsChartRequest {
            struct_size: 0,
            kind: 0,
            reserved: 0,
            instants: births.as_ptr(),
            instant_count: births.len(),
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
            rules_json: ptr::null(),
            interpret_json: ptr::null(),
            varsha_json: ptr::null(),
            gochar_json: ptr::null(),
            hits_json: hits.as_ptr(),
            sade_sati_json: ptr::null(),
            kp_json: ptr::null(),
            dignities_json: ptr::null(),
            fortitudes_json: ptr::null(),
            lots_json: ptr::null(),
            considerations_json: ptr::null(),
            perfection_json: ptr::null(),
            progressions_json: ptr::null(),
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
    let column = |name: &str| reader.column("hits", name).unwrap();

    let sdk = teistro::Context::builder()
        .profile("conformance-baseline")
        .ephemeris([teistro::Ephemeris::Builtin])
        .build()
        .unwrap();
    let place = teistro::quantity::Place::try_from_degrees(27.7172, 85.324, 1400.0).unwrap();
    let natal = sdk
        .chart()
        .readings(
            &births.map(teistro::quantity::JulianDay::<teistro::quantity::Utc>::literal),
            &teistro::ChartRequest::at(
                place,
                teistro::UtcOffset::try_from_seconds(20_700).unwrap(),
            ),
        )
        .unwrap()
        .value;
    let asked = teistro::HitRequest::from_json(text).unwrap();
    let lists: Vec<Vec<teistro::Hit>> = natal
        .iter()
        .map(|document| sdk.chart().hits(document, &asked).unwrap().value)
        .collect();

    // Ragged: each chart's count is its own list's.
    let counts = reader.column("cast", "hit_count").unwrap();
    for (count, list) in counts.iter().zip(&lists) {
        assert_eq!(count.as_i64(), i64::try_from(list.len()).unwrap());
    }
    assert_ne!(lists[0].len(), lists[1].len(), "two charts, two lists");
    let columns = [
        "instant", "graha", "kind", "into", "motion", "to_lagna", "to_graha", "angle", "phase",
    ]
    .map(column);
    let expected: Vec<&teistro::Hit> = lists.iter().flatten().collect();
    assert_eq!(columns[0].len(), expected.len());
    let mut kinds = [0_usize; 4];
    for (at, hit) in expected.iter().enumerate() {
        let cell = |c: usize| columns[c][at].as_i64();
        assert_eq!(columns[0][at].as_f64(), hit.instant.get());
        assert_eq!(cell(1), i64::from(hit.graha.id()));
        let kind = TsHitKind::from(&hit.event);
        assert_eq!(cell(2), i64::from(kind as u8));
        kinds[kind as usize] += 1;
        match hit.event {
            HitEvent::SignIngress { into, motion } => {
                assert_eq!(cell(3), i64::from(into.id()));
                assert_eq!(cell(4), i64::from(TsMotion::from(motion) as u8));
            }
            HitEvent::NakshatraIngress { into, motion } => {
                assert_eq!(cell(3), i64::from(into.id()));
                assert_eq!(cell(4), i64::from(TsMotion::from(motion) as u8));
            }
            HitEvent::Station { turns } => {
                assert_eq!(cell(4), i64::from(TsMotion::from(turns) as u8));
            }
            HitEvent::Aspect {
                to,
                angle,
                phase,
                motion,
            } => {
                assert_eq!(cell(4), i64::from(TsMotion::from(motion) as u8));
                match to {
                    teistro::NatalPoint::Lagna => assert_eq!(cell(5), 1),
                    teistro::NatalPoint::Graha { graha } => {
                        assert_eq!((cell(5), cell(6)), (0, i64::from(graha.id())));
                    }
                }
                assert_eq!(cell(7), i64::from(angle));
                assert_eq!(cell(8), i64::from(TsAspectPhase::from(phase) as u8));
            }
        }
    }
    assert!(kinds.iter().all(|count| *count > 0), "{kinds:?}");

    // A refusal names the field the caller wrote, under `hits`.
    let refused = |json: &str| {
        let text = CString::new(json).unwrap();
        let asked = TsChartRequest {
            hits_json: text.as_ptr(),
            ..request
        };
        let mut out = TsBlob::empty();
        // SAFETY: as above.
        let status = unsafe { ts_chart_found(ctx.handle, &raw const asked, &raw mut out) };
        assert_eq!(status, Status::InvalidArg, "{json}");
        ctx.last_error().2
    };
    assert_eq!(
        refused(r#"{"from":2460676.5,"to":2460600.5}"#).as_deref(),
        Some("hits.to")
    );
    assert_eq!(
        refused(r#"{"from":2460676.5,"to":2460866.5,"orbDeg":20}"#).as_deref(),
        Some("hits.orbDeg")
    );
    assert_eq!(
        refused(r#"{"from":2460676.5,"to":2460866.5,"grahas":["PLUTO"]}"#).as_deref(),
        Some("hits.grahas")
    );
}

/// Sade Sati crosses: a request's `sade_sati_json` answers every chart's
/// report in the `sade_sati` section, a row a chart, and its visits in
/// `sade_sati_visits`, ragged by `cast.sade_sati_visit_count`, each row the
/// façade's own visit in the order the section declares; and a refusal
/// names the field the caller wrote (`03-design/sade-sati.md`).
#[test]
fn a_chart_request_answers_sade_sati() {
    use teistro_ffi::chart::{TsGocharFrom, TsReckoning};
    let ctx = Ctx::with_ephemeris(0, TsEphemeris::Builtin, None, None, None).unwrap();
    let births = [2_447_995.489_583_333_5, 2_451_545.0];
    let text = r#"{"from":2460676.5,"to":2464329.0,"reckoning":"DEGREE","spells":[4,7,8]}"#;
    let json = CString::new(text).unwrap();
    let request = sized(
        TsChartRequest {
            struct_size: 0,
            kind: 0,
            reserved: 0,
            instants: births.as_ptr(),
            instant_count: births.len(),
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
            rules_json: ptr::null(),
            interpret_json: ptr::null(),
            varsha_json: ptr::null(),
            gochar_json: ptr::null(),
            hits_json: ptr::null(),
            sade_sati_json: json.as_ptr(),
            kp_json: ptr::null(),
            dignities_json: ptr::null(),
            fortitudes_json: ptr::null(),
            lots_json: ptr::null(),
            considerations_json: ptr::null(),
            perfection_json: ptr::null(),
            progressions_json: ptr::null(),
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

    let sdk = teistro::Context::builder()
        .ephemeris([teistro::Ephemeris::Builtin])
        .build()
        .unwrap();
    let place = teistro::quantity::Place::try_from_degrees(27.7172, 85.324, 1400.0).unwrap();
    let natal = sdk
        .chart()
        .readings(
            &births.map(teistro::quantity::JulianDay::<teistro::quantity::Utc>::literal),
            &teistro::ChartRequest::at(
                place,
                teistro::UtcOffset::try_from_seconds(20_700).unwrap(),
            ),
        )
        .unwrap()
        .value;
    let asked = teistro::SadeSatiRequest::from_json(text).unwrap();
    let reports: Vec<teistro::sade_sati::Report> = natal
        .iter()
        .map(|document| sdk.chart().sade_sati(document, &asked).unwrap().value)
        .collect();

    // A row a chart: what its houses were counted from and in.
    let charted = |name: &str| reader.column("sade_sati", name).unwrap();
    let (reference, from, reckoning) = (
        charted("reference"),
        charted("counted_from"),
        charted("reckoning"),
    );
    assert_eq!(reference.len(), reports.len());
    for (at, report) in reports.iter().enumerate() {
        assert_eq!(
            reference[at].as_i64(),
            i64::from(report.reference.sign.id())
        );
        assert_eq!(from[at].as_i64(), i64::from(TsGocharFrom::Moon as u8));
        assert_eq!(reckoning[at].as_i64(), i64::from(TsReckoning::Degree as u8));
    }

    // A row a visit, ragged, in the section's order: Sade Satis, then the
    // smaller spells, each period's houses in turn and their visits in time.
    let mut expected = Vec::new();
    for report in &reports {
        let periods = report
            .sade_sati
            .iter()
            .map(|one| one.phases.as_slice())
            .chain(report.spells.iter().map(std::slice::from_ref));
        let mut rows = Vec::new();
        for (period, spells) in periods.enumerate() {
            for spell in spells {
                for visit in &spell.visits {
                    rows.push((period, spell.house, visit.from.unwrap(), visit.to.unwrap()));
                }
            }
        }
        expected.push(rows);
    }
    let counts = reader.column("cast", "sade_sati_visit_count").unwrap();
    for (count, rows) in counts.iter().zip(&expected) {
        assert_eq!(count.as_i64(), i64::try_from(rows.len()).unwrap());
    }
    assert!(
        reports
            .iter()
            .all(|r| !r.sade_sati.is_empty() || !r.spells.is_empty()),
        "ten years hold a period for each chart"
    );
    let visit = |name: &str| reader.column("sade_sati_visits", name).unwrap();
    let columns = ["period", "house", "from", "to"].map(visit);
    let expected: Vec<_> = expected.into_iter().flatten().collect();
    assert_eq!(columns[0].len(), expected.len());
    for (at, (period, house, from, to)) in expected.iter().enumerate() {
        assert_eq!(columns[0][at].as_i64(), i64::try_from(*period).unwrap());
        assert_eq!(columns[1][at].as_i64(), i64::from(*house));
        assert_eq!(columns[2][at].as_f64().to_bits(), from.get().to_bits());
        assert_eq!(columns[3][at].as_f64().to_bits(), to.get().to_bits());
    }

    // A refusal names the field the caller wrote, under `sadeSati`.
    let refused = |json: &str| {
        let text = CString::new(json).unwrap();
        let asked = TsChartRequest {
            sade_sati_json: text.as_ptr(),
            kp_json: ptr::null(),
            dignities_json: ptr::null(),
            fortitudes_json: ptr::null(),
            lots_json: ptr::null(),
            considerations_json: ptr::null(),
            perfection_json: ptr::null(),
            progressions_json: ptr::null(),
            ..request
        };
        let mut out = TsBlob::empty();
        // SAFETY: as above.
        let status = unsafe { ts_chart_found(ctx.handle, &raw const asked, &raw mut out) };
        assert_eq!(status, Status::InvalidArg, "{json}");
        ctx.last_error().2
    };
    assert_eq!(
        refused(r#"{"from":2460676.5,"to":2460600.5}"#).as_deref(),
        Some("sadeSati.to")
    );
    assert_eq!(
        refused(r#"{"from":2460676.5,"spells":[12]}"#).as_deref(),
        Some("sadeSati.spells")
    );
    assert_eq!(
        refused(r#"{"from":2460676.5,"reckoning":"ARC"}"#).as_deref(),
        Some("sadeSati.reckoning")
    );

    // The plan that says the periods crosses beside them, a chart's entry
    // each, from the same search: the reports are the same rows, and with
    // no pack loaded each plan is present and empty, which is an answer.
    let interpret = CString::new(r#"{"sadeSati":true}"#).unwrap();
    let said = TsChartRequest {
        interpret_json: interpret.as_ptr(),
        ..request
    };
    let mut out = TsBlob::empty();
    // SAFETY: as above.
    assert_eq!(
        unsafe { ts_chart_found(ctx.handle, &raw const said, &raw mut out) },
        Status::Ok,
        "{:?}",
        ctx.last_error()
    );
    // SAFETY: the library wrote `len` bytes.
    let with_plans = unsafe { core::slice::from_raw_parts(out.data, out.len) }.to_vec();
    // SAFETY: a descriptor the library wrote.
    unsafe { ts_blob_free(&raw mut out) };
    let reader = Reader::parse(&with_plans, &schema).unwrap();
    let plans: serde_json::Value = serde_json::from_slice(reader.bytes("plans").unwrap()).unwrap();
    assert_eq!(
        plans,
        serde_json::json!([{ "sadeSati": [] }, { "sadeSati": [] }])
    );
    assert_eq!(
        reader.column("sade_sati_visits", "house").unwrap().len(),
        columns[1].len(),
        "one search answers the report and the plan"
    );

    // And without a window to search it is refused by the member asked.
    let unsearched = TsChartRequest {
        sade_sati_json: ptr::null(),
        kp_json: ptr::null(),
        dignities_json: ptr::null(),
        fortitudes_json: ptr::null(),
        lots_json: ptr::null(),
        considerations_json: ptr::null(),
        perfection_json: ptr::null(),
        progressions_json: ptr::null(),
        ..said
    };
    // SAFETY: as above.
    let status = unsafe { ts_chart_found(ctx.handle, &raw const unsearched, &raw mut out) };
    assert_eq!(status, Status::InvalidArg);
    assert_eq!(ctx.last_error().2.as_deref(), Some("interpret.sadeSati"));
}

/// Whether a planet holds one dignity, as `a_chart_request_answers_the_dignities`
/// reads each flag column against the façade's.
type HoldsDignity = fn(&teistro::EssentialDignity) -> bool;

/// The essential dignities cross: a request's `dignities_json` answers every
/// chart's sect and the rules applied in `dignities`, a row a chart, and the
/// seven planets in `dignity_planets`, seven rows a chart in the Chaldean
/// order, each cell the façade's own to the bit; none asked is two empty
/// sections, and a refusal names the field the caller wrote
/// (`03-design/essential-dignities.md`).
#[test]
fn a_chart_request_answers_the_dignities() {
    use teistro_ffi::chart::{TsSect, TsSectRule, TsTerms, TsTriplicities};

    let ctx = Ctx::with_ephemeris(
        0,
        TsEphemeris::Builtin,
        Some("conformance-baseline"),
        None,
        None,
    )
    .unwrap();
    let instants = [2_460_676.5, 2_460_676.75];
    let text = r#"{"sectRule":"DAYLIGHT","rules":{"terms":"EGYPTIAN","triplicities":"PTOLEMY"},"scores":{"peregrine":0}}"#;
    let request = |dignities_json: *const core::ffi::c_char| {
        sized(
            TsChartRequest {
                struct_size: 0,
                kind: 0,
                reserved: 0,
                instants: instants.as_ptr(),
                instant_count: instants.len(),
                latitude_deg: 27.7172,
                longitude_deg: 85.324,
                altitude_m: 0.0,
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
                rules_json: ptr::null(),
                interpret_json: ptr::null(),
                varsha_json: ptr::null(),
                gochar_json: ptr::null(),
                hits_json: ptr::null(),
                sade_sati_json: ptr::null(),
                kp_json: ptr::null(),
                dignities_json,
                fortitudes_json: ptr::null(),
                lots_json: ptr::null(),
                considerations_json: ptr::null(),
                perfection_json: ptr::null(),
                progressions_json: ptr::null(),
            },
            |r, s| r.struct_size = s,
        )
    };
    let found = |asked: &TsChartRequest| {
        let mut blob = TsBlob::empty();
        // SAFETY: a live context, a valid request and a valid slot.
        let status = unsafe { ts_chart_found(ctx.handle, asked, &raw mut blob) };
        (status, blob)
    };
    let json = CString::new(text).unwrap();
    let (status, mut blob) = found(&request(json.as_ptr()));
    assert_eq!(status, Status::Ok, "{:?}", ctx.last_error());
    // SAFETY: the library wrote `len` bytes.
    let bytes = unsafe { core::slice::from_raw_parts(blob.data, blob.len) }.to_vec();
    // SAFETY: a descriptor the library wrote.
    unsafe { ts_blob_free(&raw mut blob) };
    let schema = schemas::charts();
    let reader = Reader::parse(&bytes, &schema).unwrap();

    // The façade's own reading of the same charts.
    let sdk = teistro::Context::builder()
        .ephemeris([teistro::Ephemeris::Builtin])
        .profile("conformance-baseline")
        .build()
        .unwrap();
    let place = teistro::quantity::Place::try_from_degrees(27.7172, 85.324, 0.0).unwrap();
    let clock = teistro::UtcOffset::try_from_seconds(20_700).unwrap();
    let natal = sdk
        .chart()
        .readings(
            &instants.map(teistro::quantity::JulianDay::<teistro::quantity::Utc>::literal),
            &teistro::ChartRequest::at(place, clock),
        )
        .unwrap()
        .value;
    let asked = teistro::DignityRequest::from_json(text).unwrap();
    let expected: Vec<teistro::Dignities> = natal
        .iter()
        .map(|document| sdk.chart().dignities(document, &asked).unwrap())
        .collect();

    let charted = |name: &str| -> Vec<i64> {
        reader
            .column("dignities", name)
            .unwrap()
            .into_iter()
            .map(ScalarValue::as_i64)
            .collect()
    };
    let code = |value: u8| i64::from(value);
    assert_eq!(
        charted("sect"),
        expected
            .iter()
            .map(|one| code(TsSect::from(one.sect) as u8))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        charted("sect_rule"),
        vec![code(TsSectRule::Daylight as u8); 2]
    );
    assert_eq!(charted("terms"), vec![code(TsTerms::Egyptian as u8); 2]);
    assert_eq!(
        charted("triplicities"),
        vec![code(TsTriplicities::Ptolemy as u8); 2]
    );
    let s = asked.scores();
    for (column, worth) in [
        ("score_house", s.house),
        ("score_exaltation", s.exaltation),
        ("score_triplicity", s.triplicity),
        ("score_term", s.term),
        ("score_face", s.face),
        ("score_detriment", s.detriment),
        ("score_fall", s.fall),
        ("score_peregrine", s.peregrine),
    ] {
        assert_eq!(charted(column), vec![i64::from(worth); 2], "{column}");
    }
    assert_eq!(charted("score_peregrine"), [0, 0], "asked for");
    assert_eq!(charted("score_house"), [5, 5], "Lilly's, left out");

    let planets: Vec<&teistro::PlanetDignity> =
        expected.iter().flat_map(|one| one.planets.iter()).collect();
    let row = |name: &str| reader.column("dignity_planets", name).unwrap();
    let ids: Vec<i64> = row("planet").into_iter().map(ScalarValue::as_i64).collect();
    assert_eq!(
        ids,
        planets
            .iter()
            .map(|at| i64::from(at.planet.id()))
            .collect::<Vec<_>>()
    );
    for (cell, at) in row("longitude").into_iter().zip(&planets) {
        assert_eq!(
            cell.as_f64().to_bits(),
            at.longitude_deg.to_bits(),
            "{:?}",
            at.planet
        );
    }
    let flags: [(&str, HoldsDignity); 7] = [
        ("house", |d| d.house),
        ("exaltation", |d| d.exaltation),
        ("triplicity", |d| d.triplicity),
        ("term", |d| d.term),
        ("face", |d| d.face),
        ("detriment", |d| d.detriment),
        ("fall", |d| d.fall),
    ];
    for (name, flag) in flags {
        let cells: Vec<i64> = row(name).into_iter().map(ScalarValue::as_i64).collect();
        let wanted: Vec<i64> = planets
            .iter()
            .map(|at| i64::from(flag(&at.dignity)))
            .collect();
        assert_eq!(cells, wanted, "{name}");
    }
    let scores: Vec<i64> = row("score").into_iter().map(ScalarValue::as_i64).collect();
    assert_eq!(
        scores,
        planets
            .iter()
            .map(|at| i64::from(at.score))
            .collect::<Vec<_>>()
    );
    let reception: Vec<i64> = row("reception")
        .into_iter()
        .map(ScalarValue::as_i64)
        .collect();
    assert_eq!(
        reception,
        planets
            .iter()
            .map(|at| i64::from(at.reception))
            .collect::<Vec<_>>()
    );

    // The receptions, ragged by each chart's count.
    let pairs: Vec<&teistro::Reception> = expected
        .iter()
        .flat_map(|one| one.receptions.iter())
        .collect();
    assert!(!pairs.is_empty(), "two charts with no reception at all");
    assert_eq!(
        charted("reception_count"),
        expected
            .iter()
            .map(|one| i64::try_from(one.receptions.len()).unwrap())
            .collect::<Vec<_>>()
    );
    let pair = |name: &str| -> Vec<i64> {
        reader
            .column("dignity_receptions", name)
            .unwrap()
            .into_iter()
            .map(ScalarValue::as_i64)
            .collect()
    };
    for (k, side) in [("first", 0), ("second", 1)] {
        assert_eq!(
            pair(k),
            pairs
                .iter()
                .map(|one| i64::from(one.planets[side].id()))
                .collect::<Vec<_>>(),
            "{k}"
        );
    }
    for (name, flag) in flags {
        for (side, first) in [("first_in", true), ("second_in", false)] {
            assert_eq!(
                pair(&format!("{side}_{name}")),
                pairs
                    .iter()
                    .map(|one| i64::from(flag(if first { &one.first_in } else { &one.second_in })))
                    .collect::<Vec<_>>(),
                "{side}_{name}"
            );
        }
    }

    // None asked: every section empty.
    let (status, mut blob) = found(&request(ptr::null()));
    assert_eq!(status, Status::Ok);
    // SAFETY: the library wrote `len` bytes.
    let bytes = unsafe { core::slice::from_raw_parts(blob.data, blob.len) }.to_vec();
    // SAFETY: a descriptor the library wrote.
    unsafe { ts_blob_free(&raw mut blob) };
    let reader = Reader::parse(&bytes, &schema).unwrap();
    assert_eq!(reader.column("dignities", "sect").unwrap().len(), 0);
    assert_eq!(reader.column("dignity_planets", "planet").unwrap().len(), 0);
    assert_eq!(
        reader.column("dignity_receptions", "first").unwrap().len(),
        0
    );

    // A refusal names the field the caller wrote.
    let typo = CString::new(r#"{"scores":{"peregrin":0}}"#).unwrap();
    let (status, _) = found(&request(typo.as_ptr()));
    assert_eq!(status, Status::InvalidArg);
    assert_eq!(
        ctx.last_error().2.as_deref(),
        Some("dignities.scores.peregrin")
    );
}

/// A value of a chart's fortitudes the `fortitudes` row carries, as
/// `a_chart_request_answers_the_fortitudes` reads each column.
type ReadsSky = fn(&teistro::Fortitudes) -> f64;

/// A value of a planet's accidents the `fortitude_planets` row carries.
type ReadsPlanet = fn(&teistro::PlanetAccidents) -> i64;

/// The accidental fortitudes cross: a request's `fortitudes_json` answers
/// the essential half in the dignity sections and the accidental half in
/// `fortitudes`, `fortitude_houses`, `fortitude_planets` and
/// `fortitude_accidents`, each cell the façade's own to the bit; asking
/// for the dignities twice is refused, and so is a misspelt rule
/// (`03-design/essential-dignities.md` §Accidental fortitudes).
#[test]
#[allow(clippy::too_many_lines, reason = "one request, every section it fills")]
fn a_chart_request_answers_the_fortitudes() {
    use teistro_ffi::chart::{
        TsAccident, TsFortuneRule, TsPartile, TsPlaceReading, TsSectRule, TsSiege,
    };

    let ctx = Ctx::with_ephemeris(
        0,
        TsEphemeris::Builtin,
        Some("conformance-baseline"),
        None,
        None,
    )
    .unwrap();
    let instants = [2_460_676.5, 2_460_676.75];
    let text = r#"{"dignities":{"sectRule":"DAYLIGHT"},"rules":{"beamsDeg":15,"partile":{"WITHIN":{"orbDeg":1}},"siege":{"WITHIN":{"spanDeg":30}}},"scores":{"regulus":5,"houses":[5,3,1,4,3,-2,4,-2,2,5,4,-5]},"almuten":{"place":"SIGN","fortune":"REVERSED_BY_NIGHT"}}"#;
    let base = sized(
        TsChartRequest {
            struct_size: 0,
            kind: 0,
            reserved: 0,
            instants: instants.as_ptr(),
            instant_count: instants.len(),
            latitude_deg: 27.7172,
            longitude_deg: 85.324,
            altitude_m: 0.0,
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
            rules_json: ptr::null(),
            interpret_json: ptr::null(),
            varsha_json: ptr::null(),
            gochar_json: ptr::null(),
            hits_json: ptr::null(),
            sade_sati_json: ptr::null(),
            kp_json: ptr::null(),
            dignities_json: ptr::null(),
            fortitudes_json: ptr::null(),
            lots_json: ptr::null(),
            considerations_json: ptr::null(),
            perfection_json: ptr::null(),
            progressions_json: ptr::null(),
        },
        |r, s| r.struct_size = s,
    );
    let found = |asked: &TsChartRequest| {
        let mut blob = TsBlob::empty();
        // SAFETY: a live context, a valid request and a valid slot.
        let status = unsafe { ts_chart_found(ctx.handle, asked, &raw mut blob) };
        let bytes = (status == Status::Ok).then(|| {
            // SAFETY: the library wrote `len` bytes.
            unsafe { core::slice::from_raw_parts(blob.data, blob.len) }.to_vec()
        });
        // SAFETY: a descriptor the library wrote, or the empty one.
        unsafe { ts_blob_free(&raw mut blob) };
        (status, bytes)
    };
    let json = CString::new(text).unwrap();
    let (status, bytes) = found(&TsChartRequest {
        fortitudes_json: json.as_ptr(),
        ..base
    });
    assert_eq!(status, Status::Ok, "{:?}", ctx.last_error());
    let bytes = bytes.unwrap();
    let schema = schemas::charts();
    let reader = Reader::parse(&bytes, &schema).unwrap();

    // The façade's own reading of the same charts.
    let sdk = teistro::Context::builder()
        .ephemeris([teistro::Ephemeris::Builtin])
        .profile("conformance-baseline")
        .build()
        .unwrap();
    let place = teistro::quantity::Place::try_from_degrees(27.7172, 85.324, 0.0).unwrap();
    let clock = teistro::UtcOffset::try_from_seconds(20_700).unwrap();
    let natal = sdk
        .chart()
        .readings(
            &instants.map(teistro::quantity::JulianDay::<teistro::quantity::Utc>::literal),
            &teistro::ChartRequest::at(place, clock),
        )
        .unwrap()
        .value;
    let asked = teistro::FortitudeRequest::from_json(text).unwrap();
    let expected: Vec<teistro::Fortitudes> = natal
        .iter()
        .map(|document| sdk.chart().fortitudes(document, &asked).unwrap())
        .collect();
    let ints = |section: &str, name: &str| -> Vec<i64> {
        reader
            .column(section, name)
            .unwrap()
            .into_iter()
            .map(ScalarValue::as_i64)
            .collect()
    };
    let bits = |section: &str, name: &str| -> Vec<u64> {
        reader
            .column(section, name)
            .unwrap()
            .into_iter()
            .map(|cell| cell.as_f64().to_bits())
            .collect()
    };
    let code = |value: u8| i64::from(value);

    // The essential half, in the dignity sections, from the fortitudes'
    // own request.
    assert_eq!(
        ints("dignities", "sect_rule"),
        vec![code(TsSectRule::Daylight as u8); 2]
    );
    let essential: Vec<&teistro::PlanetDignity> = expected
        .iter()
        .flat_map(|one| one.dignities.planets.iter())
        .collect();
    assert_eq!(
        ints("dignity_planets", "score"),
        essential
            .iter()
            .map(|at| i64::from(at.score))
            .collect::<Vec<_>>()
    );

    // A row a chart: the sky as read and the rules as applied.
    assert_eq!(
        ints("fortitudes", "houses"),
        expected
            .iter()
            .map(|one| i64::from(one.sky.houses.id()))
            .collect::<Vec<_>>()
    );
    let sky: [(&str, ReadsSky); 12] = [
        ("ascendant", |one| one.sky.ascendant_deg),
        ("midheaven", |one| one.sky.midheaven_deg),
        ("fortune", |one| one.almutens.fortune_deg),
        ("north_node", |one| one.sky.north_node_deg),
        ("regulus", |one| one.sky.regulus_deg),
        ("spica", |one| one.sky.spica_deg),
        ("algol", |one| one.sky.algol_deg),
        ("combustion_orb", |one| one.rules.combustion_deg),
        ("beams_orb", |one| one.rules.beams_deg),
        ("cazimi_orb", |one| one.rules.cazimi_deg),
        ("cusp_orb", |one| one.rules.cusp_orb_deg),
        ("star_orb", |one| one.rules.star_orb_deg),
    ];
    for (name, of) in sky {
        assert_eq!(
            bits("fortitudes", name),
            expected
                .iter()
                .map(|one| of(one).to_bits())
                .collect::<Vec<_>>(),
            "{name}"
        );
    }
    assert_eq!(bits("fortitudes", "beams_orb"), vec![15.0_f64.to_bits(); 2]);
    assert_eq!(ints("fortitudes", "combustion_in_sign"), [1, 1]);
    assert_eq!(
        ints("fortitudes", "partile"),
        vec![code(TsPartile::Within as u8); 2]
    );
    assert_eq!(
        bits("fortitudes", "partile_orb"),
        vec![1.0_f64.to_bits(); 2]
    );
    assert_eq!(
        ints("fortitudes", "siege"),
        vec![code(TsSiege::Within as u8); 2]
    );
    assert_eq!(
        bits("fortitudes", "siege_span"),
        vec![30.0_f64.to_bits(); 2]
    );
    assert_eq!(
        ints("fortitudes", "almuten_place"),
        vec![code(TsPlaceReading::Sign as u8); 2]
    );
    assert_eq!(
        ints("fortitudes", "almuten_fortune"),
        vec![code(TsFortuneRule::ReversedByNight as u8); 2]
    );
    let lines = asked.scores();
    for (name, worth) in [
        ("score_direct", lines.direct),
        ("score_superior_oriental", lines.superior_oriental),
        ("score_inferior_oriental", lines.inferior_oriental),
        ("score_under_beams", lines.under_beams),
        ("score_square_malefic", lines.square_malefic),
        ("score_regulus", 5),
        ("score_algol", lines.algol),
    ] {
        assert_eq!(
            ints("fortitudes", name),
            vec![i64::from(worth); 2],
            "{name}"
        );
    }
    assert_eq!(ints("fortitudes", "score_regulus"), [5, 5], "asked for");
    assert_eq!(
        ints("fortitudes", "score_spica"),
        [5, 5],
        "Lilly's, left out"
    );

    // Twelve houses a chart.
    assert_eq!(
        bits("fortitude_houses", "cusp"),
        expected
            .iter()
            .flat_map(|one| one.sky.cusps_deg.map(f64::to_bits))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        ints("fortitude_houses", "score"),
        expected
            .iter()
            .flat_map(|one| one.scores.houses.map(i64::from))
            .collect::<Vec<_>>()
    );
    assert_eq!(&ints("fortitude_houses", "score")[..3], [5, 3, 1]);
    for (k, name) in [
        "saturn", "jupiter", "mars", "sun", "venus", "mercury", "moon",
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(
            ints("fortitude_houses", &format!("almuten_{name}")),
            expected
                .iter()
                .flat_map(|one| one.almutens.houses.map(|house| i64::from(house.totals[k])))
                .collect::<Vec<_>>(),
            "{name}"
        );
    }

    // Seven planets a chart, and their accidents, ragged.
    let planets: Vec<(&teistro::Fortitudes, usize, &teistro::PlanetAccidents)> = expected
        .iter()
        .flat_map(|one| {
            one.planets
                .iter()
                .enumerate()
                .map(move |(k, at)| (one, k, at))
        })
        .collect();
    let planet = |name: &str| ints("fortitude_planets", name);
    assert_eq!(
        planet("planet"),
        planets
            .iter()
            .map(|(_, _, at)| i64::from(at.planet.id()))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        bits("fortitude_planets", "speed"),
        planets
            .iter()
            .map(|(one, k, _)| one.sky.speeds_deg_per_day[*k].to_bits())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        bits("fortitude_planets", "mean_motion"),
        planets
            .iter()
            .map(|(one, k, _)| one.rules.mean_motion_deg[*k].to_bits())
            .collect::<Vec<_>>()
    );
    let per: [(&str, ReadsPlanet); 4] = [
        ("house", |at| i64::from(at.house.get())),
        ("fortitude", |at| i64::from(at.fortitude)),
        ("debility", |at| i64::from(at.debility)),
        ("accident_count", |at| {
            i64::try_from(at.accidents.len()).unwrap()
        }),
    ];
    for (name, of) in per {
        assert_eq!(
            planet(name),
            planets.iter().map(|(_, _, at)| of(at)).collect::<Vec<_>>(),
            "{name}"
        );
    }
    assert_eq!(
        planet("places"),
        planets
            .iter()
            .map(|(one, k, _)| i64::from(one.almutens.places.totals[*k]))
            .collect::<Vec<_>>()
    );
    let accidents: Vec<(
        &teistro::Fortitudes,
        &teistro::PlanetAccidents,
        teistro::Accident,
    )> = planets
        .iter()
        .flat_map(|&(one, _, at)| at.accidents.iter().map(move |&line| (one, at, line)))
        .collect();
    assert!(!accidents.is_empty(), "two charts with no accident at all");
    assert_eq!(
        ints("fortitude_accidents", "accident"),
        accidents
            .iter()
            .map(|(_, _, line)| code(TsAccident::of(*line).unwrap() as u8))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        ints("fortitude_accidents", "points"),
        accidents
            .iter()
            .map(|(one, at, line)| i64::from(one.scores.points(at.planet, *line)))
            .collect::<Vec<_>>()
    );

    // None asked: every section empty.
    let (status, bytes) = found(&base);
    assert_eq!(status, Status::Ok);
    let bytes = bytes.unwrap();
    let reader = Reader::parse(&bytes, &schema).unwrap();
    for (section, column) in [
        ("fortitudes", "houses"),
        ("fortitude_houses", "cusp"),
        ("fortitude_planets", "planet"),
        ("fortitude_accidents", "accident"),
        ("dignity_planets", "planet"),
    ] {
        assert_eq!(
            reader.column(section, column).unwrap().len(),
            0,
            "{section}"
        );
    }

    // The essential dignities asked for twice are refused, by the record
    // that should move.
    let dignities = CString::new("{}").unwrap();
    let (status, _) = found(&TsChartRequest {
        dignities_json: dignities.as_ptr(),
        fortitudes_json: json.as_ptr(),
        ..base
    });
    assert_eq!(status, Status::InvalidArg);
    assert_eq!(ctx.last_error().2.as_deref(), Some("dignities"));

    // A refusal names the field the caller wrote, a misspelt one or one
    // out of range.
    for (text, field) in [
        (r#"{"rules":{"beamDeg":15}}"#, "fortitudes.rules.beamDeg"),
        (r#"{"rules":{"beamsDeg":-1}}"#, "fortitudes.rules.beamsDeg"),
    ] {
        let refused = CString::new(text).unwrap();
        let (status, _) = found(&TsChartRequest {
            fortitudes_json: refused.as_ptr(),
            ..base
        });
        assert_eq!(status, Status::InvalidArg, "{text}");
        assert_eq!(ctx.last_error().2.as_deref(), Some(field));
    }
}

/// The lots cross: a request's `lots_json` answers every chart's sect and
/// rules in `lots` and its fourteen lots in `lot_places`, each cell the
/// façade's own to the bit; none asked is an empty section, and a
/// misspelt rule is refused by its field (`03-design/hellenistic-lots.md`).
#[test]
fn a_chart_request_answers_the_lots() {
    use teistro_ffi::chart::{TsFortuneRule, TsLot, TsSect};

    let ctx = Ctx::with_ephemeris(
        0,
        TsEphemeris::Builtin,
        Some("conformance-baseline"),
        None,
        None,
    )
    .unwrap();
    // Kathmandu, a day and a night in late December 2024.
    let instants = [2_460_676.5, 2_460_676.75];
    let base = chart_request(&instants, (27.7172, 85.324), 20_700);
    let text = r#"{"fortune":"REVERSED_WHILE_MOON_UP"}"#;
    let json = CString::new(text).unwrap();
    let bytes = chart_blob(
        &ctx,
        &TsChartRequest {
            lots_json: json.as_ptr(),
            ..base
        },
    )
    .unwrap_or_else(|status| panic!("{status:?}: {:?}", ctx.last_error()));
    let schema = schemas::charts();
    let reader = Reader::parse(&bytes, &schema).unwrap();

    let sdk = teistro::Context::builder()
        .ephemeris([teistro::Ephemeris::Builtin])
        .profile("conformance-baseline")
        .build()
        .unwrap();
    let place = teistro::quantity::Place::try_from_degrees(27.7172, 85.324, 0.0).unwrap();
    let clock = teistro::UtcOffset::try_from_seconds(20_700).unwrap();
    let natal = sdk
        .chart()
        .readings(
            &instants.map(teistro::quantity::JulianDay::<teistro::quantity::Utc>::literal),
            &teistro::ChartRequest::at(place, clock),
        )
        .unwrap()
        .value;
    let asked = teistro::LotRequest::from_json(text).unwrap();
    let expected: Vec<teistro::LotReading> = natal
        .iter()
        .map(|document| {
            sdk.chart()
                .lots_with_request(document, &teistro::Lot::ALL, asked)
                .unwrap()
        })
        .collect();
    let ints = |section: &str, name: &str| -> Vec<i64> {
        reader
            .column(section, name)
            .unwrap()
            .into_iter()
            .map(ScalarValue::as_i64)
            .collect()
    };
    let code = |value: u8| i64::from(value);

    // A row a chart: the sect, the rules and whether Fortune reversed.
    assert_eq!(
        ints("lots", "sect"),
        expected
            .iter()
            .map(|one| code(TsSect::from(one.sect) as u8))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        ints("lots", "fortune"),
        vec![code(TsFortuneRule::ReversedWhileMoonUp as u8); 2]
    );
    assert_eq!(
        ints("lots", "fortune_reversed"),
        expected
            .iter()
            .map(|one| i64::from(one.fortune_reversed))
            .collect::<Vec<_>>()
    );
    // Fourteen rows a chart, each the façade's to the bit.
    let placed: Vec<&teistro::PlacedLot> = expected.iter().flat_map(|one| &one.lots).collect();
    assert_eq!(placed.len(), 28);
    assert_eq!(
        ints("lot_places", "lot"),
        placed
            .iter()
            .map(|one| code(TsLot::of(one.lot).unwrap() as u8))
            .collect::<Vec<_>>()
    );
    let longitudes: Vec<u64> = reader
        .column("lot_places", "longitude_deg")
        .unwrap()
        .into_iter()
        .map(|cell| cell.as_f64().to_bits())
        .collect();
    assert_eq!(
        longitudes,
        placed
            .iter()
            .map(|one| one.place.longitude_deg.to_bits())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        ints("lot_places", "sign"),
        placed
            .iter()
            .map(|one| i64::from(one.place.sign.id()))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        ints("lot_places", "lord"),
        placed
            .iter()
            .map(|one| i64::from(one.place.lord.id()))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        ints("lot_places", "house"),
        placed
            .iter()
            .map(|one| i64::from(one.place.house.get()))
            .collect::<Vec<_>>()
    );

    // None asked is empty sections.
    let bytes = chart_blob(&ctx, &base).unwrap();
    let reader = Reader::parse(&bytes, &schema).unwrap();
    assert_eq!(reader.column("lots", "sect").unwrap().len(), 0);
    assert_eq!(reader.column("lot_places", "lot").unwrap().len(), 0);

    // A misspelt rule is refused by the field the caller wrote.
    let refused = CString::new(r#"{"fortuna":"DAY_AND_NIGHT"}"#).unwrap();
    let status = chart_blob(
        &ctx,
        &TsChartRequest {
            lots_json: refused.as_ptr(),
            ..base
        },
    )
    .unwrap_err();
    assert_eq!(status, Status::InvalidArg);
    assert_eq!(ctx.last_error().2.as_deref(), Some("lots.fortuna"));
}

/// The considerations cross: a request's `considerations_json` answers
/// every chart's clauses in `considerations`, the Moon's two perfections
/// in `consideration_perfections` and the orbs in `consideration_orbs`,
/// each cell the façade's own to the bit, read from the request's own
/// fortitudes; none asked is an empty section, and a rule out of range is
/// refused by its field (`03-design/hellenistic-considerations.md`).
#[test]
fn a_chart_request_answers_the_considerations() {
    use teistro_ffi::chart::TsPtolemaicAspect;

    let ctx = Ctx::with_ephemeris(
        0,
        TsEphemeris::Builtin,
        Some("conformance-baseline"),
        None,
        None,
    )
    .unwrap();
    // London, every three hours over two days of January 2000: the Moon
    // both void and applying.
    let instants: Vec<f64> = (0..16).map(|k| 2_451_545.0 + f64::from(k) / 8.0).collect();
    let base = chart_request(&instants, (51.5, -0.12), 0);
    let rules_text = r#"{"moonLateFromDeg":25}"#;
    let fortitudes_text = r#"{"rules":{"combustionDeg":9}}"#;
    let rules_json = CString::new(rules_text).unwrap();
    let fortitudes_json = CString::new(fortitudes_text).unwrap();
    let bytes = chart_blob(
        &ctx,
        &TsChartRequest {
            considerations_json: rules_json.as_ptr(),
            fortitudes_json: fortitudes_json.as_ptr(),
            ..base
        },
    )
    .unwrap_or_else(|status| panic!("{status:?}: {:?}", ctx.last_error()));
    let schema = schemas::charts();
    let reader = Reader::parse(&bytes, &schema).unwrap();

    let sdk = teistro::Context::builder()
        .ephemeris([teistro::Ephemeris::Builtin])
        .profile("conformance-baseline")
        .build()
        .unwrap();
    let place = teistro::quantity::Place::try_from_degrees(51.5, -0.12, 0.0).unwrap();
    let clock = teistro::UtcOffset::try_from_seconds(0).unwrap();
    let natal = sdk
        .chart()
        .readings(
            &instants
                .iter()
                .map(|&jd| teistro::quantity::JulianDay::<teistro::quantity::Utc>::literal(jd))
                .collect::<Vec<_>>(),
            &teistro::ChartRequest::at(place, clock),
        )
        .unwrap()
        .value;
    let rules = teistro::ConsiderationRules::from_json(rules_text).unwrap();
    let fortitudes = teistro::FortitudeRequest::from_json(fortitudes_text).unwrap();
    let expected: Vec<teistro::Considerations> = natal
        .iter()
        .map(|document| {
            sdk.chart()
                .considerations(document, &fortitudes, rules)
                .unwrap()
        })
        .collect();
    let ints = |section: &str, name: &str| -> Vec<i64> {
        reader
            .column(section, name)
            .unwrap()
            .into_iter()
            .map(ScalarValue::as_i64)
            .collect()
    };
    let bits = |section: &str, name: &str| -> Vec<u64> {
        reader
            .column(section, name)
            .unwrap()
            .into_iter()
            .map(|cell| cell.as_f64().to_bits())
            .collect()
    };
    let each = |read: &dyn Fn(&teistro::Considerations) -> i64| -> Vec<i64> {
        expected.iter().map(read).collect()
    };
    let each_f = |read: &dyn Fn(&teistro::Considerations) -> f64| -> Vec<u64> {
        expected.iter().map(|one| read(one).to_bits()).collect()
    };
    let flag = |value: bool| i64::from(value);

    // A row a chart.
    assert_eq!(
        ints("considerations", "hour_lord"),
        each(&|one| i64::from(one.radicality.hour_lord.id()))
    );
    assert_eq!(
        ints("considerations", "ascendant_lord"),
        each(&|one| i64::from(one.radicality.ascendant_lord.id()))
    );
    assert_eq!(
        ints("considerations", "radical_grounds"),
        each(&|one| {
            one.radicality
                .grounds
                .iter()
                .map(|ground| {
                    1_i64 << (teistro_ffi::chart::TsRadicalGround::of(*ground).unwrap() as u8)
                })
                .sum()
        })
    );
    assert_eq!(
        ints("considerations", "ascendant_sign"),
        each(&|one| i64::from(one.ascendant.sign.id()))
    );
    assert_eq!(
        bits("considerations", "ascendant_degree"),
        each_f(&|one| one.ascendant.degree)
    );
    assert_eq!(
        ints("considerations", "ascendant_early"),
        each(&|one| flag(one.ascendant.early))
    );
    assert_eq!(
        ints("considerations", "ascendant_late"),
        each(&|one| flag(one.ascendant.late))
    );
    assert_eq!(
        ints("considerations", "short_ascension"),
        each(&|one| flag(one.ascendant.short_ascension))
    );
    assert_eq!(
        ints("considerations", "moon_sign"),
        each(&|one| i64::from(one.moon.sign.id()))
    );
    assert_eq!(
        bits("considerations", "moon_degree"),
        each_f(&|one| one.moon.degree)
    );
    assert_eq!(
        ints("considerations", "moon_late"),
        each(&|one| flag(one.moon.late))
    );
    assert_eq!(
        ints("considerations", "moon_late_sign"),
        each(&|one| flag(one.moon.late_sign))
    );
    assert_eq!(
        ints("considerations", "via_combusta"),
        each(&|one| flag(one.moon.via_combusta))
    );
    assert_eq!(
        bits("considerations", "days_in_sign"),
        each_f(&|one| one.moon.course.days_in_sign)
    );
    assert_eq!(
        ints("considerations", "eased"),
        each(&|one| flag(one.moon.course.eased))
    );
    assert_eq!(
        bits("considerations", "seventh_cusp_deg"),
        each_f(&|one| one.seventh.cusp_deg)
    );
    assert_eq!(
        ints("considerations", "seventh_lord"),
        each(&|one| i64::from(one.seventh.lord.id()))
    );
    assert_eq!(
        ints("considerations", "seventh_infortunes"),
        each(&|one| {
            one.seventh
                .infortunes_in_house
                .iter()
                .map(|graha| 1_i64 << graha.id())
                .sum()
        })
    );
    assert_eq!(
        ints("considerations", "seventh_lord_retrograde"),
        each(&|one| flag(one.seventh.lord_retrograde))
    );
    assert_eq!(
        ints("considerations", "seventh_lord_combust"),
        each(&|one| flag(one.seventh.lord_combust))
    );
    assert_eq!(
        ints("considerations", "seventh_lord_in_fall"),
        each(&|one| flag(one.seventh.lord_in_fall))
    );
    assert_eq!(
        ints("considerations", "seventh_lord_in_infortune_term"),
        each(&|one| flag(one.seventh.lord_in_infortune_term))
    );
    assert_eq!(
        ints("considerations", "seventh_lord_net"),
        each(&|one| i64::from(one.seventh.lord_net))
    );
    assert_eq!(
        ints("considerations", "saturn_house"),
        each(&|one| i64::from(one.saturn_house.get()))
    );
    assert_eq!(
        ints("considerations", "saturn_retrograde"),
        each(&|one| flag(one.saturn_retrograde))
    );
    assert_eq!(
        ints("considerations", "ascendant_lord_combust"),
        each(&|one| flag(one.ascendant_lord_combust))
    );
    assert_eq!(
        bits("considerations", "moon_late_from_deg"),
        vec![25.0_f64.to_bits(); instants.len()]
    );

    // Two perfections a chart, by the sign's end then within the moieties.
    let perfections: Vec<Option<teistro::Perfection>> = expected
        .iter()
        .flat_map(|one| [one.moon.course.next, one.moon.course.within_orb])
        .collect();
    assert!(perfections.iter().any(Option::is_some));
    assert!(
        perfections.iter().any(Option::is_none),
        "no void Moon in the batch"
    );
    assert_eq!(
        ints("consideration_perfections", "present"),
        perfections
            .iter()
            .map(|one| i64::from(one.is_some()))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        ints("consideration_perfections", "planet"),
        perfections
            .iter()
            .map(|one| one.map_or(0, |found| i64::from(found.planet.id())))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        ints("consideration_perfections", "aspect"),
        perfections
            .iter()
            .map(|one| one.map_or(0, |found| i64::from(
                TsPtolemaicAspect::of(found.aspect).unwrap() as u8
            )))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        bits("consideration_perfections", "days"),
        perfections
            .iter()
            .map(|one| one.map_or(f64::NAN, |found| found.days).to_bits())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        bits("consideration_perfections", "gap_deg"),
        perfections
            .iter()
            .map(|one| one.map_or(f64::NAN, |found| found.gap_deg).to_bits())
            .collect::<Vec<_>>()
    );
    // Seven orbs a chart, Lilly's.
    assert_eq!(
        bits("consideration_orbs", "orb_deg"),
        expected
            .iter()
            .flat_map(|one| one.rules.orbs_deg.map(f64::to_bits))
            .collect::<Vec<_>>()
    );

    // None asked is empty sections.
    let bytes = chart_blob(&ctx, &base).unwrap();
    let reader = Reader::parse(&bytes, &schema).unwrap();
    for (section, column) in [
        ("considerations", "hour_lord"),
        ("consideration_perfections", "present"),
        ("consideration_orbs", "orb_deg"),
    ] {
        assert_eq!(
            reader.column(section, column).unwrap().len(),
            0,
            "{section}"
        );
    }

    // A rule out of range is refused by the field the caller wrote.
    let refused = CString::new(r#"{"moonLateFromDeg":31}"#).unwrap();
    let status = chart_blob(
        &ctx,
        &TsChartRequest {
            considerations_json: refused.as_ptr(),
            ..base
        },
    )
    .unwrap_err();
    assert_eq!(status, Status::InvalidArg);
    assert_eq!(
        ctx.last_error().2.as_deref(),
        Some("considerations.moonLateFromDeg")
    );
}

/// The perfection crosses: a request's `perfection_json` answers every
/// chart's significators, application, separation and ways in
/// `perfection`, its impediments, translations and collections ragged by
/// that row's counts, and the orbs in `perfection_orbs`, each cell the
/// façade's own to the bit, weighed on Lilly's fortitudes when none were
/// asked; none asked is an empty section, and a request naming no
/// quesited is refused by its field (`03-design/hellenistic-perfection.md`).
#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one assertion per column of five sections"
)]
fn a_chart_request_answers_the_perfection() {
    let ctx = Ctx::with_ephemeris(
        0,
        TsEphemeris::Builtin,
        Some("conformance-baseline"),
        None,
        None,
    )
    .unwrap();
    // London, every eleven days over a year: the seventh's lord changes
    // with the rising sign, and the slow planets turn.
    let instants: Vec<f64> = (0..34)
        .map(|k| 2_451_545.0 + 11.0 * f64::from(k) + f64::from(k % 5) / 7.0)
        .collect();
    let base = chart_request(&instants, (51.5, -0.12), 0);
    let asked_text = r#"{"house":7}"#;
    let asked_json = CString::new(asked_text).unwrap();
    let bytes = chart_blob(
        &ctx,
        &TsChartRequest {
            perfection_json: asked_json.as_ptr(),
            ..base
        },
    )
    .unwrap_or_else(|status| panic!("{status:?}: {:?}", ctx.last_error()));
    let schema = schemas::charts();
    let reader = Reader::parse(&bytes, &schema).unwrap();

    let sdk = teistro::Context::builder()
        .ephemeris([teistro::Ephemeris::Builtin])
        .profile("conformance-baseline")
        .build()
        .unwrap();
    let place = teistro::quantity::Place::try_from_degrees(51.5, -0.12, 0.0).unwrap();
    let clock = teistro::UtcOffset::try_from_seconds(0).unwrap();
    let natal = sdk
        .chart()
        .readings(
            &instants
                .iter()
                .map(|&jd| teistro::quantity::JulianDay::<teistro::quantity::Utc>::literal(jd))
                .collect::<Vec<_>>(),
            &teistro::ChartRequest::at(place, clock),
        )
        .unwrap()
        .value;
    let asked = teistro::PerfectionRequest::from_json(asked_text).unwrap();
    let lilly = teistro::FortitudeRequest::default();
    let expected: Vec<teistro::Matter> = natal
        .iter()
        .map(|document| sdk.chart().perfection(document, &lilly, &asked).unwrap())
        .collect();
    let ints = |section: &str, name: &str| -> Vec<i64> {
        reader
            .column(section, name)
            .unwrap()
            .into_iter()
            .map(ScalarValue::as_i64)
            .collect()
    };
    let bits = |section: &str, name: &str| -> Vec<u64> {
        reader
            .column(section, name)
            .unwrap()
            .into_iter()
            .map(|cell| cell.as_f64().to_bits())
            .collect()
    };
    let dignity = |d: teistro::EssentialDignity| -> i64 {
        [
            d.house,
            d.exaltation,
            d.triplicity,
            d.term,
            d.face,
            d.detriment,
            d.fall,
        ]
        .into_iter()
        .enumerate()
        .map(|(n, flag)| i64::from(flag) << n)
        .sum()
    };
    let graha = |g: Graha| i64::from(g.id());
    let aspect = |a: teistro::PtolemaicAspect| {
        i64::from(teistro_ffi::chart::TsPtolemaicAspect::of(a).unwrap() as u8)
    };
    let flag = |value: bool| i64::from(value);
    let each = |read: &dyn Fn(&teistro::Matter) -> i64| -> Vec<i64> {
        expected.iter().map(read).collect()
    };
    let each_f = |read: &dyn Fn(&teistro::Matter) -> f64| -> Vec<u64> {
        expected.iter().map(|one| read(one).to_bits()).collect()
    };
    let nan = f64::NAN;

    // A row a chart.
    assert_eq!(
        ints("perfection", "querent"),
        each(&|one| graha(one.querent))
    );
    assert_eq!(
        ints("perfection", "quesited"),
        each(&|one| graha(one.quesited))
    );
    assert_eq!(
        bits("perfection", "horizon_days"),
        each_f(&|one| one.horizon_days)
    );
    assert_eq!(bits("perfection", "horizon_rule_days"), each_f(&|_| nan));
    assert_eq!(ints("perfection", "within_sign_rule"), each(&|_| 1));
    assert_eq!(
        ints("perfection", "application_present"),
        each(&|one| flag(one.application.is_some()))
    );
    assert_eq!(
        ints("perfection", "application_aspect"),
        each(&|one| one.application.map_or(0, |a| aspect(a.aspect)))
    );
    assert_eq!(
        bits("perfection", "application_days"),
        each_f(&|one| one.application.map_or(nan, |a| a.days))
    );
    assert_eq!(
        ints("perfection", "applying"),
        each(&|one| one.application.map_or(0, |a| graha(a.applying)))
    );
    assert_eq!(
        ints("perfection", "application_kind"),
        each(&|one| {
            one.application.map_or(0, |a| {
                i64::from(teistro_ffi::chart::TsApplicationKind::of(a.kind).unwrap() as u8)
            })
        })
    );
    assert_eq!(
        bits("perfection", "gap_deg"),
        each_f(&|one| one.application.map_or(nan, |a| a.gap_deg))
    );
    assert_eq!(
        ints("perfection", "within_moieties"),
        each(&|one| one.application.map_or(0, |a| flag(a.within_moieties)))
    );
    assert_eq!(
        ints("perfection", "separation_present"),
        each(&|one| flag(one.separation.is_some()))
    );
    assert_eq!(
        ints("perfection", "separation_aspect"),
        each(&|one| one.separation.map_or(0, |a| aspect(a.aspect)))
    );
    assert_eq!(
        bits("perfection", "separation_past_deg"),
        each_f(&|one| one.separation.map_or(nan, |a| a.past_deg))
    );
    assert_eq!(
        ints("perfection", "querent_house"),
        each(&|one| i64::from(one.ways.querent.house.get()))
    );
    assert_eq!(
        ints("perfection", "querent_dignity"),
        each(&|one| dignity(one.ways.querent.dignity))
    );
    assert_eq!(
        ints("perfection", "quesited_house"),
        each(&|one| i64::from(one.ways.quesited.house.get()))
    );
    assert_eq!(
        ints("perfection", "quesited_dignity"),
        each(&|one| dignity(one.ways.quesited.dignity))
    );
    assert_eq!(
        ints("perfection", "mutual_by_house"),
        each(&|one| flag(one.ways.mutual_by_house))
    );
    assert_eq!(
        ints("perfection", "infortunes_between"),
        each(&|one| {
            one.ways
                .infortunes_between
                .iter()
                .map(|g| 1 << g.id())
                .sum()
        })
    );
    assert_eq!(
        ints("perfection", "moon_relays"),
        each(&|one| flag(one.ways.moon_relays))
    );
    assert_eq!(
        ints("perfection", "quesited_in_ascendant"),
        each(&|one| flag(one.ways.quesited_in_ascendant))
    );
    assert_eq!(
        ints("perfection", "ways_held"),
        each(&|one| {
            one.ways
                .held
                .iter()
                .map(|way| 1 << (teistro_ffi::chart::TsWay::of(*way).unwrap() as u8))
                .sum()
        })
    );
    for (column, read) in [
        (
            "impediment_count",
            (|one: &teistro::Matter| one.impediments.len()) as fn(&teistro::Matter) -> usize,
        ),
        ("translation_count", |one| one.translations.len()),
        ("collection_count", |one| one.collections.len()),
    ] {
        assert_eq!(
            ints("perfection", column),
            each(&|one| i64::try_from(read(one)).unwrap()),
            "{column}"
        );
    }

    // Ragged under the row: every chart's in turn.
    let impediments: Vec<&teistro::Impediment> =
        expected.iter().flat_map(|one| &one.impediments).collect();
    let translations: Vec<&teistro::Translation> =
        expected.iter().flat_map(|one| &one.translations).collect();
    let collections: Vec<&teistro::Collection> =
        expected.iter().flat_map(|one| &one.collections).collect();
    assert!(
        !impediments.is_empty() && !translations.is_empty() && !collections.is_empty(),
        "the sweep reaches impediments, translations and collections"
    );
    assert_eq!(
        ints("perfection_impediments", "kind"),
        impediments
            .iter()
            .map(|i| i64::from(teistro_ffi::chart::TsImpedimentKind::of(i.kind).unwrap() as u8))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        ints("perfection_impediments", "significator"),
        impediments
            .iter()
            .map(|i| graha(i.significator))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        ints("perfection_impediments", "third_present"),
        impediments
            .iter()
            .map(|i| flag(i.third.is_some()))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        ints("perfection_impediments", "third"),
        impediments
            .iter()
            .map(|i| i.third.map_or(0, graha))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        ints("perfection_impediments", "aspect"),
        impediments
            .iter()
            .map(|i| aspect(i.aspect))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        bits("perfection_impediments", "days"),
        impediments
            .iter()
            .map(|i| i.days.to_bits())
            .collect::<Vec<_>>()
    );
    for (column, read) in [
        (
            "translator",
            &(|t: &teistro::Translation| graha(t.translator))
                as &dyn Fn(&teistro::Translation) -> i64,
        ),
        ("from", &|t| graha(t.from)),
        ("to", &|t| graha(t.to)),
        ("separating_aspect", &|t| aspect(t.separating.aspect)),
        ("aspect", &|t| aspect(t.aspect)),
        ("received", &|t| dignity(t.received)),
    ] {
        assert_eq!(
            ints("perfection_translations", column),
            translations.iter().map(|t| read(t)).collect::<Vec<_>>(),
            "{column}"
        );
    }
    assert_eq!(
        bits("perfection_translations", "separating_past_deg"),
        translations
            .iter()
            .map(|t| t.separating.past_deg.to_bits())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        bits("perfection_translations", "days"),
        translations
            .iter()
            .map(|t| t.days.to_bits())
            .collect::<Vec<_>>()
    );
    for (column, read) in [
        (
            "collector",
            &(|c: &teistro::Collection| graha(c.collector)) as &dyn Fn(&teistro::Collection) -> i64,
        ),
        ("from_querent_aspect", &|c| aspect(c.from_querent.aspect)),
        ("from_quesited_aspect", &|c| aspect(c.from_quesited.aspect)),
        ("collector_in_querent", &|c| dignity(c.collector_in_querent)),
        ("collector_in_quesited", &|c| {
            dignity(c.collector_in_quesited)
        }),
        ("querent_in_collector", &|c| dignity(c.querent_in_collector)),
        ("quesited_in_collector", &|c| {
            dignity(c.quesited_in_collector)
        }),
    ] {
        assert_eq!(
            ints("perfection_collections", column),
            collections.iter().map(|c| read(c)).collect::<Vec<_>>(),
            "{column}"
        );
    }
    for (column, read) in [
        (
            "from_querent_days",
            (|c: &teistro::Collection| c.from_querent.days) as fn(&teistro::Collection) -> f64,
        ),
        ("from_quesited_days", |c| c.from_quesited.days),
    ] {
        assert_eq!(
            bits("perfection_collections", column),
            collections
                .iter()
                .map(|c| read(c).to_bits())
                .collect::<Vec<_>>(),
            "{column}"
        );
    }

    // Seven orbs a chart, Lilly's.
    assert_eq!(
        bits("perfection_orbs", "orb_deg"),
        expected
            .iter()
            .flat_map(|_| asked.rules.orbs_deg)
            .map(f64::to_bits)
            .collect::<Vec<_>>()
    );

    // Every contact inside the horizon, as asked, is the flag cleared
    // and no fewer impediments.
    let every = CString::new(r#"{"house":7,"rules":{"withinSign":false}}"#).unwrap();
    let bytes = chart_blob(
        &ctx,
        &TsChartRequest {
            perfection_json: every.as_ptr(),
            ..base
        },
    )
    .unwrap();
    let wider = Reader::parse(&bytes, &schema).unwrap();
    let column = |reader: &Reader<'_, '_>, name: &str| -> Vec<i64> {
        reader
            .column("perfection", name)
            .unwrap()
            .into_iter()
            .map(ScalarValue::as_i64)
            .collect()
    };
    assert!(
        column(&wider, "within_sign_rule")
            .iter()
            .all(|&flag| flag == 0)
    );
    let counted = |reader: &Reader<'_, '_>| column(reader, "impediment_count").iter().sum::<i64>();
    assert!(counted(&wider) > counted(&reader), "{}", counted(&reader));

    // None asked is every section empty.
    let bytes = chart_blob(&ctx, &base).unwrap();
    let reader = Reader::parse(&bytes, &schema).unwrap();
    for (section, column) in [
        ("perfection", "querent"),
        ("perfection_impediments", "kind"),
        ("perfection_translations", "translator"),
        ("perfection_collections", "collector"),
        ("perfection_orbs", "orb_deg"),
    ] {
        assert_eq!(
            reader.column(section, column).unwrap().len(),
            0,
            "{section}"
        );
    }

    // A request naming no quesited is refused by the field it lacks.
    let refused = CString::new("{}").unwrap();
    let status = chart_blob(
        &ctx,
        &TsChartRequest {
            perfection_json: refused.as_ptr(),
            ..base
        },
    )
    .unwrap_err();
    assert_eq!(status, Status::InvalidArg);
    assert_eq!(ctx.last_error().2.as_deref(), Some("perfection.quesited"));
}

/// The lots record also sets the rules releasing reads its lots under: a
/// night birth whose Fortune moves sign under Lilly's rule releases from
/// where Lilly puts it, as the façade's `with_lot_rules` does
/// (`03-design/hellenistic-time-lords.md`).
#[test]
fn a_lots_record_sets_the_lots_releasing_starts_from() {
    let ctx = Ctx::with_ephemeris(
        0,
        TsEphemeris::Builtin,
        Some("conformance-baseline"),
        None,
        None,
    )
    .unwrap();
    // 1 January 2000, 18:00 UTC: night in London.
    let instants = [2_451_545.25];
    let dashas = [DashaSystem::ReleasingFortune.id()];
    let base = TsChartRequest {
        dashas: dashas.as_ptr(),
        dasha_count: dashas.len(),
        ..chart_request(&instants, (51.5, -0.12), 0)
    };
    let schema = schemas::charts();
    let first_sign = |lots_json: *const core::ffi::c_char| {
        let bytes = chart_blob(&ctx, &TsChartRequest { lots_json, ..base })
            .unwrap_or_else(|status| panic!("{status:?}: {:?}", ctx.last_error()));
        let reader = Reader::parse(&bytes, &schema).unwrap();
        reader.column("dasha_periods", "sign").unwrap()[0].as_i64()
    };

    let sdk = teistro::Context::builder()
        .ephemeris([teistro::Ephemeris::Builtin])
        .profile("conformance-baseline")
        .build()
        .unwrap();
    let place = teistro::quantity::Place::try_from_degrees(51.5, -0.12, 0.0).unwrap();
    let facade = |rules: teistro::LotRequest| {
        let request = teistro::ChartRequest::at(place, teistro::UtcOffset::UTC)
            .with_dashas([DashaSystem::ReleasingFortune])
            .with_lot_rules(rules);
        let document = sdk
            .chart()
            .reading(
                teistro::quantity::JulianDay::<teistro::quantity::Utc>::literal(instants[0]),
                &request,
            )
            .unwrap()
            .value;
        i64::from(document.dashas[0].start_sign().unwrap().id())
    };
    let lilly = teistro::LotRequest::VALENS.with_fortune(teistro::FortuneRule::DayAndNight);
    let text = CString::new(r#"{"fortune":"DAY_AND_NIGHT"}"#).unwrap();
    assert_ne!(
        facade(lilly),
        facade(teistro::LotRequest::VALENS),
        "the test must bite"
    );
    assert_eq!(first_sign(text.as_ptr()), facade(lilly));
    assert_eq!(first_sign(ptr::null()), facade(teistro::LotRequest::VALENS));
}

/// KP crosses: a request's `kp_json` answers every chart's reading as
/// canonical JSON in the `kp` section, the façade's own reading on the
/// chart request's clock unless the record names one, spelled as the
/// section's description says; none asked is an empty section, and a
/// refusal names the field the caller wrote (`03-design/kp.md`).
#[test]
fn a_chart_request_answers_kp() {
    let ctx = Ctx::with_ephemeris(0, TsEphemeris::Builtin, Some("kp-default"), None, None).unwrap();
    let births = [2_447_995.489_583_333_5, 2_451_545.0];
    let text = r#"{"number":74}"#;
    let json = CString::new(text).unwrap();
    let request = sized(
        TsChartRequest {
            struct_size: 0,
            kind: 0,
            reserved: 0,
            instants: births.as_ptr(),
            instant_count: births.len(),
            latitude_deg: 13.08,
            longitude_deg: 80.27,
            altitude_m: 6.0,
            utc_offset_seconds: 19_800,
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
            rules_json: ptr::null(),
            interpret_json: ptr::null(),
            varsha_json: ptr::null(),
            gochar_json: ptr::null(),
            hits_json: ptr::null(),
            sade_sati_json: ptr::null(),
            kp_json: json.as_ptr(),
            dignities_json: ptr::null(),
            fortitudes_json: ptr::null(),
            lots_json: ptr::null(),
            considerations_json: ptr::null(),
            perfection_json: ptr::null(),
            progressions_json: ptr::null(),
        },
        |r, s| r.struct_size = s,
    );
    let section = |asked: &TsChartRequest| {
        let mut blob = TsBlob::empty();
        // SAFETY: a live context, a valid request and a valid slot.
        let status = unsafe { ts_chart_found(ctx.handle, asked, &raw mut blob) };
        assert_eq!(status, Status::Ok, "{:?}", ctx.last_error());
        // SAFETY: the library wrote `len` bytes.
        let bytes = unsafe { core::slice::from_raw_parts(blob.data, blob.len) }.to_vec();
        // SAFETY: a descriptor the library wrote.
        unsafe { ts_blob_free(&raw mut blob) };
        let schema = schemas::charts();
        let reader = Reader::parse(&bytes, &schema).unwrap();
        String::from_utf8(reader.bytes("kp").unwrap().to_vec()).unwrap()
    };
    let crossed: serde_json::Value = serde_json::from_str(&section(&request)).unwrap();

    // The façade's own readings, on the chart request's clock.
    let sdk = teistro::Context::builder()
        .ephemeris([teistro::Ephemeris::Builtin])
        .profile("kp-default")
        .build()
        .unwrap();
    let place = teistro::quantity::Place::try_from_degrees(13.08, 80.27, 6.0).unwrap();
    let clock = teistro::UtcOffset::try_from_seconds(19_800).unwrap();
    let natal = sdk
        .chart()
        .readings(
            &births.map(teistro::quantity::JulianDay::<teistro::quantity::Utc>::literal),
            &teistro::ChartRequest::at(place, clock),
        )
        .unwrap()
        .value;
    let asked = teistro::KpRequest::from_json(text).unwrap().on_clock(clock);
    let expected: Vec<teistro::KpReading> = natal
        .iter()
        .map(|document| sdk.chart().kp_reading(document, &asked).unwrap())
        .collect();
    assert_eq!(crossed, serde_json::to_value(&expected).unwrap());

    // Spelled as the section says, member for member.
    let keys = |value: &serde_json::Value| {
        let mut keys: Vec<String> = value.as_object().unwrap().keys().cloned().collect();
        keys.sort();
        keys
    };
    let one = &crossed[0];
    assert_eq!(keys(one), ["chart", "ruling", "significators"]);
    assert_eq!(keys(&one["chart"]), ["cusps", "planets", "system"]);
    assert_eq!(
        keys(&one["chart"]["cusps"][0]),
        ["house", "longitude", "lords"]
    );
    assert_eq!(
        keys(&one["chart"]["planets"][0]),
        ["graha", "house", "longitude", "lords", "retrograde"]
    );
    let lords = &one["chart"]["cusps"][0]["lords"];
    assert_eq!(keys(lords), ["sign", "star", "sub", "subSub"]);
    assert_eq!(keys(&lords["sub"]), ["lord", "span"]);
    assert_eq!(keys(&lords["sub"]["span"]), ["end", "start"]);
    assert_eq!(keys(&one["significators"]), ["houses", "nodes"]);
    assert_eq!(
        keys(&one["significators"]["houses"][0]),
        [
            "aspected",
            "conjoined",
            "house",
            "inLordsStar",
            "inOccupantsStars",
            "intercepted",
            "lord",
            "occupants"
        ]
    );
    assert_eq!(
        keys(&one["significators"]["nodes"][0]),
        ["aspecting", "conjoined", "node", "signLord", "starLord"]
    );
    assert_eq!(keys(&one["ruling"]), ["rulers", "rules"]);
    assert_eq!(
        keys(&one["ruling"]["rulers"][0]),
        [
            "graha",
            "reasons",
            "rejectedBy",
            "rejectedBySub",
            "retrograde"
        ]
    );
    assert_eq!(keys(&one["ruling"]["rulers"][0]["reasons"][0]), ["kind"]);
    // The horary number's lagna, exact.
    assert_eq!(
        one["chart"]["cusps"][0]["longitude"].as_i64(),
        Some(teistro::KpNumber::new(74).unwrap().start().get())
    );

    // None asked is an empty section.
    let none = TsChartRequest {
        kp_json: ptr::null(),
        dignities_json: ptr::null(),
        fortitudes_json: ptr::null(),
        lots_json: ptr::null(),
        considerations_json: ptr::null(),
        perfection_json: ptr::null(),
        progressions_json: ptr::null(),
        ..request
    };
    assert_eq!(section(&none), "");

    // A refusal names the field the caller wrote, under `kp`.
    let refused = |json: &str| {
        let text = CString::new(json).unwrap();
        let asked = TsChartRequest {
            kp_json: text.as_ptr(),
            dignities_json: ptr::null(),
            fortitudes_json: ptr::null(),
            lots_json: ptr::null(),
            considerations_json: ptr::null(),
            perfection_json: ptr::null(),
            progressions_json: ptr::null(),
            ..request
        };
        let mut out = TsBlob::empty();
        // SAFETY: as above.
        let status = unsafe { ts_chart_found(ctx.handle, &raw const asked, &raw mut out) };
        assert_eq!(status, Status::InvalidArg, "{json}");
        ctx.last_error().2
    };
    assert_eq!(refused(r#"{"number":250}"#).as_deref(), Some("kp.number"));
    assert_eq!(refused(r#"{"clock":90000}"#).as_deref(), Some("kp.clock"));
}

/// A batch of none asking for the window searches is an empty blob, as a
/// batch of none is everywhere else: the façade refuses no charts by
/// `natals`, a name no caller of the boundary wrote, so the boundary asks
/// it nothing. The record is still read, so a bad one is still refused.
#[test]
fn a_batch_of_none_asking_for_the_searches_is_empty() {
    let ctx = Ctx::with_ephemeris(0, TsEphemeris::Builtin, None, None, None).unwrap();
    let hits = CString::new(r#"{"from":2460676.5,"to":2460866.5}"#).unwrap();
    let sade_sati = CString::new(r#"{"from":2460676.5}"#).unwrap();
    let request = sized(
        TsChartRequest {
            struct_size: 0,
            kind: 0,
            reserved: 0,
            instants: ptr::null(),
            instant_count: 0,
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
            rules_json: ptr::null(),
            interpret_json: ptr::null(),
            varsha_json: ptr::null(),
            gochar_json: ptr::null(),
            hits_json: hits.as_ptr(),
            sade_sati_json: sade_sati.as_ptr(),
            kp_json: ptr::null(),
            dignities_json: ptr::null(),
            fortitudes_json: ptr::null(),
            lots_json: ptr::null(),
            considerations_json: ptr::null(),
            perfection_json: ptr::null(),
            progressions_json: ptr::null(),
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
    for (section, column) in [
        ("hits", "instant"),
        ("sade_sati", "reference"),
        ("sade_sati_visits", "house"),
    ] {
        assert!(
            reader.column(section, column).unwrap().is_empty(),
            "{section}"
        );
    }

    let bad = CString::new(r#"{"from":2460676.5,"spells":[1]}"#).unwrap();
    let asked = TsChartRequest {
        sade_sati_json: bad.as_ptr(),
        kp_json: ptr::null(),
        dignities_json: ptr::null(),
        fortitudes_json: ptr::null(),
        lots_json: ptr::null(),
        considerations_json: ptr::null(),
        perfection_json: ptr::null(),
        progressions_json: ptr::null(),
        ..request
    };
    let mut out = TsBlob::empty();
    // SAFETY: as above.
    let status = unsafe { ts_chart_found(ctx.handle, &raw const asked, &raw mut out) };
    assert_eq!(status, Status::InvalidArg);
    assert_eq!(ctx.last_error().2.as_deref(), Some("sadeSati.spells"));
}

/// The annual charts cross: a request's `varsha_json` answers every chart's
/// returns in the `praveshas` section, ragged by `cast.pravesha_count`, and
/// the Sun at a return stands where it stood at birth
/// (`03-design/annual-chart.md`).
#[test]
fn a_chart_request_answers_the_annual_charts_instants() {
    let ctx = Ctx::with_ephemeris(
        0,
        TsEphemeris::Builtin,
        Some("conformance-baseline"),
        None,
        None,
    )
    .unwrap();
    let instants = [2_447_995.489_583_333_5, 2_451_545.0];
    let varsha = CString::new(r#"{"reading":"SIDEREAL","through":12}"#).unwrap();
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
            rules_json: ptr::null(),
            interpret_json: ptr::null(),
            varsha_json: varsha.as_ptr(),
            gochar_json: ptr::null(),
            hits_json: ptr::null(),
            sade_sati_json: ptr::null(),
            kp_json: ptr::null(),
            dignities_json: ptr::null(),
            fortitudes_json: ptr::null(),
            lots_json: ptr::null(),
            considerations_json: ptr::null(),
            perfection_json: ptr::null(),
            progressions_json: ptr::null(),
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

    // Ragged by the per-chart count, which is how a reader walks it.
    let counts = reader.column("cast", "pravesha_count").unwrap();
    assert_eq!(counts.len(), 2);
    assert!(counts.iter().all(|count| count.as_i64() == 12));
    let years = reader.column("praveshas", "year").unwrap();
    let jds = reader.column("praveshas", "jd").unwrap();
    assert_eq!(years.len(), 24, "two charts of twelve years");
    assert_eq!(jds.len(), years.len());
    // Each chart's years run 1 to 12 in order, and its returns run forward.
    for (chart, birth) in instants.iter().enumerate() {
        let at = chart * 12;
        for (year, row) in years[at..at + 12].iter().enumerate() {
            assert_eq!(row.as_i64(), i64::try_from(year).unwrap() + 1);
        }
        let first = jds[at].as_f64();
        let last = jds[at + 11].as_f64();
        assert!(first > *birth, "a return is after its birth");
        assert!(last > first, "the years run forward");
        // Eleven sidereal years between the first and the twelfth, to a
        // day: a tropical reading would be a quarter of a day short.
        let span = last - first;
        assert!((span - 11.0 * 365.2564).abs() < 1.0, "{span} days");
    }

    // Nothing asked for is nothing answered, not zeroes.
    let none = TsChartRequest {
        varsha_json: ptr::null(),
        ..request
    };
    let mut bare = TsBlob::empty();
    // SAFETY: as above.
    assert_eq!(
        unsafe { ts_chart_found(ctx.handle, &raw const none, &raw mut bare) },
        Status::Ok
    );
    // SAFETY: the library wrote `len` bytes.
    let empty = unsafe { core::slice::from_raw_parts(bare.data, bare.len) }.to_vec();
    // SAFETY: a descriptor the library wrote.
    unsafe { ts_blob_free(&raw mut bare) };
    let reader = Reader::parse(&empty, &schema).unwrap();
    assert!(
        reader.column("praveshas", "year").unwrap().is_empty(),
        "{:?}",
        reader.column("praveshas", "year").unwrap()
    );
    assert!(
        reader
            .column("cast", "pravesha_count")
            .unwrap()
            .iter()
            .all(|count| count.as_i64() == 0)
    );

    // A year outside the cap is refused from the field the caller wrote.
    let refused = |json: &str| {
        let text = CString::new(json).unwrap();
        let asked = TsChartRequest {
            varsha_json: text.as_ptr(),
            ..request
        };
        let mut out = TsBlob::empty();
        // SAFETY: as above.
        let status = unsafe { ts_chart_found(ctx.handle, &raw const asked, &raw mut out) };
        assert_eq!(status, Status::InvalidArg, "{json}");
        ctx.last_error()
    };
    let wide = refused(r#"{"reading":"SIDEREAL","through":0}"#);
    assert_eq!(wide.2.as_deref(), Some("varsha.through"), "{wide:?}");
    let typo = refused(r#"{"readng":"SIDEREAL","through":4}"#);
    assert!(
        typo.1.contains("readng") || typo.1.contains("reading"),
        "{typo:?}"
    );
}

/// A chart request for two Kathmandu births — 1990-04-11 and J2000 — with
/// the annual charts `varsha` asks for, answered as the blob's bytes or
/// the context's refusal. The annual-chart tests share it, so each says
/// only what it asks.
fn annual_blob(ctx: &Ctx, varsha: &str) -> Result<Vec<u8>, Record> {
    let instants = [2_447_995.489_583_333_5, 2_451_545.0];
    let text = CString::new(varsha).unwrap();
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
            rules_json: ptr::null(),
            interpret_json: ptr::null(),
            varsha_json: text.as_ptr(),
            gochar_json: ptr::null(),
            hits_json: ptr::null(),
            sade_sati_json: ptr::null(),
            kp_json: ptr::null(),
            dignities_json: ptr::null(),
            fortitudes_json: ptr::null(),
            lots_json: ptr::null(),
            considerations_json: ptr::null(),
            perfection_json: ptr::null(),
            progressions_json: ptr::null(),
        },
        |r, s| r.struct_size = s,
    );
    let mut blob = TsBlob::empty();
    // SAFETY: a live context, a valid request and a valid slot.
    let status = unsafe { ts_chart_found(ctx.handle, &raw const request, &raw mut blob) };
    if status != Status::Ok {
        return Err(ctx.last_error());
    }
    // SAFETY: the library wrote `len` bytes.
    let bytes = unsafe { core::slice::from_raw_parts(blob.data, blob.len) }.to_vec();
    // SAFETY: a descriptor the library wrote.
    unsafe { ts_blob_free(&raw mut blob) };
    Ok(bytes)
}

/// A year's own chart is founded only when `varsha_json.place` asks, at the
/// place it names, and its office-bearers come back row for row beside the
/// returns — all of them or none (`03-design/muntha.md`).
#[test]
fn a_chart_request_founds_each_years_chart_where_it_is_told() {
    let ctx = Ctx::with_ephemeris(0, TsEphemeris::Builtin, None, None, None).unwrap();
    let schema = schemas::charts();
    let ask = |json: &str| annual_blob(&ctx, json);
    let column = |bytes: &[u8], name: &str| {
        Reader::parse(bytes, &schema)
            .unwrap()
            .column("annual_charts", name)
            .unwrap()
            .iter()
            .map(|cell| cell.as_f64())
            .collect::<Vec<f64>>()
    };

    // Not asked for, none founded: the instants and nothing else.
    let unasked = ask(r#"{"through":6}"#).unwrap();
    assert!(
        column(&unasked, "lagna_deg").is_empty(),
        "{:?}",
        column(&unasked, "lagna_deg")
    );

    // At the birthplace: one row per return, every one a real chart.
    let birth = ask(r#"{"through":6,"place":"birth"}"#).unwrap();
    let lagnas = column(&birth, "lagna_deg");
    assert_eq!(lagnas.len(), 12, "two charts of six years, row for row");
    assert!(lagnas.iter().all(|deg| (0.0..360.0).contains(deg)));
    assert!(
        column(&birth, "daylight")
            .iter()
            .all(|flag| *flag == 0.0 || *flag == 1.0)
    );
    // The birth lagna's lord is the one office-bearer every year shares.
    let janma = column(&birth, "janma_lagna_lord");
    assert!(janma[..6].iter().all(|lord| *lord == janma[0]));
    assert!(janma[6..].iter().all(|lord| *lord == janma[6]));

    // At a residence the years' lagnas move and the birth's lord does not.
    let delhi = ask(
        r#"{"through":6,"place":{"latitudeDeg":28.6139,"longitudeDeg":77.209,"altitudeM":216,"utcOffsetSeconds":19800}}"#,
    )
    .unwrap();
    let moved = column(&delhi, "lagna_deg");
    assert_eq!(moved.len(), lagnas.len());
    assert!(
        moved
            .iter()
            .zip(&lagnas)
            .all(|(there, here)| (there - here).abs() > 0.1)
    );
    assert_eq!(column(&delhi, "janma_lagna_lord"), janma);

    // Each refusal names the field and says what would have been read.
    let refused = |json: &str| ask(json).expect_err(json);
    let word = refused(r#"{"through":6,"place":"home"}"#);
    assert!(
        word.1.contains("\"birth\"") && word.1.contains("home"),
        "{word:?}"
    );
    let far = refused(
        r#"{"through":6,"place":{"latitudeDeg":95,"longitudeDeg":0,"utcOffsetSeconds":0}}"#,
    );
    assert!(far.1.contains("latitudeDeg"), "{far:?}");
    let extra = refused(
        r#"{"through":6,"place":{"latitudeDeg":1,"longitudeDeg":2,"utcOffsetSeconds":0,"zone":"x"}}"#,
    );
    assert!(extra.1.contains("zone"), "{extra:?}");
    // Each is named where it was written: the place, or the key in it.
    for (error, field) in [
        (&word, "varsha.place"),
        (&far, "varsha.place"),
        (&extra, "varsha.place.zone"),
    ] {
        assert_eq!(error.2.as_deref(), Some(field), "{error:?}");
    }
}

/// A year's chart carries the **lord of that year** and the reckoning it
/// came out of: every claimant, ragged by `claim_count`, with the strength
/// and the aspect each was judged on (`03-design/varshesha.md`).
#[test]
fn a_years_chart_carries_the_lord_of_that_year() {
    let ctx = Ctx::with_ephemeris(0, TsEphemeris::Builtin, None, None, None).unwrap();
    let instants = [2_447_995.489_583_333_5, 2_451_545.0];
    let varsha = CString::new(r#"{"through":4,"place":"birth"}"#).unwrap();
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
            rules_json: ptr::null(),
            interpret_json: ptr::null(),
            varsha_json: varsha.as_ptr(),
            gochar_json: ptr::null(),
            hits_json: ptr::null(),
            sade_sati_json: ptr::null(),
            kp_json: ptr::null(),
            dignities_json: ptr::null(),
            fortitudes_json: ptr::null(),
            lots_json: ptr::null(),
            considerations_json: ptr::null(),
            perfection_json: ptr::null(),
            progressions_json: ptr::null(),
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
    let ints = |section: &str, name: &str| {
        reader
            .column(section, name)
            .unwrap()
            .iter()
            .map(|cell| cell.as_i64())
            .collect::<Vec<i64>>()
    };

    let lords = ints("annual_charts", "year_lord");
    let counts = ints("annual_charts", "claim_count");
    assert_eq!(lords.len(), 8, "two charts of four years");
    assert_eq!(counts.len(), lords.len());
    // Every year names a lord and ranks between one and five claimants.
    assert!(counts.iter().all(|count| (1..=5).contains(count)));
    // The chain's step is one this ABI knows.
    let chosen = ints("annual_charts", "year_lord_chosen");
    let last = TsVarsheshaChosen::MoonsSignLord as i64;
    assert!(chosen.iter().all(|step| (0..=last).contains(step)));
    let succeeds_the_moon = |step: i64| {
        step == TsVarsheshaChosen::MoonsIthasala as i64
            || step == TsVarsheshaChosen::MoonsSignLord as i64
    };
    // A strength is exact, in sub-sub units, and inside its own bound of
    // twenty units.
    let vishwa = ints("annual_charts", "year_lord_vishwa");
    assert!(vishwa.iter().all(|bala| (0..=20 * 3600).contains(bala)));

    // The claims section is ragged by the count, exactly.
    let claim_graha = ints("year_claims", "graha");
    let total: i64 = counts.iter().sum();
    assert_eq!(i64::try_from(claim_graha.len()).unwrap(), total);
    let claim_vishwa = ints("year_claims", "vishwa");
    let aspects = ints("year_claims", "aspects_lagna");
    let portfolios = ints("year_claims", "portfolios");
    assert!(portfolios.iter().all(|held| (1..=5).contains(held)));
    assert!(aspects.iter().all(|flag| *flag == 0 || *flag == 1));

    // Each year's claimants are ranked strongest first, and the lord is
    // one of them — the strongest that aspects, or a named fallback — unless
    // it succeeds the Moon, whose Ithasala may be with any planet.
    let mut from = 0usize;
    for (year, count) in counts.iter().enumerate() {
        let take = usize::try_from(*count).unwrap();
        let block = &claim_vishwa[from..from + take];
        assert!(
            block.windows(2).all(|pair| pair[0] >= pair[1]),
            "year {year} is not ranked"
        );
        assert!(
            succeeds_the_moon(chosen[year])
                || claim_graha[from..from + take].contains(&lords[year]),
            "the lord of year {year} is not among its claimants"
        );
        from += take;
    }

    // Not asked for is empty, not zeroes.
    let none = TsChartRequest {
        varsha_json: ptr::null(),
        ..request
    };
    let mut bare = TsBlob::empty();
    // SAFETY: as above.
    assert_eq!(
        unsafe { ts_chart_found(ctx.handle, &raw const none, &raw mut bare) },
        Status::Ok
    );
    // SAFETY: the library wrote `len` bytes.
    let empty = unsafe { core::slice::from_raw_parts(bare.data, bare.len) }.to_vec();
    // SAFETY: a descriptor the library wrote.
    unsafe { ts_blob_free(&raw mut bare) };
    let reader = Reader::parse(&empty, &schema).unwrap();
    assert!(
        reader.column("year_claims", "graha").unwrap().is_empty(),
        "{:?}",
        reader.column("year_claims", "graha").unwrap()
    );
}

/// A year's chart answers the sixteen Tajika yogas for the **matters** the
/// request names, in its order, each with the question it asked and every
/// yoga that held, ragged three deep (`03-design/tajika-yogas.md`,
/// "Crossing the boundary").
#[test]
fn a_years_chart_answers_the_matters_it_was_asked_about() {
    let ctx = Ctx::with_ephemeris(0, TsEphemeris::Builtin, None, None, None).unwrap();
    let schema = schemas::charts();
    let ask = |json: &str| annual_blob(&ctx, json);
    let ints = |bytes: &[u8], section: &str, name: &str| {
        Reader::parse(bytes, &schema)
            .unwrap()
            .column(section, name)
            .unwrap()
            .iter()
            .map(|cell| usize::try_from(cell.as_i64()).unwrap())
            .collect::<Vec<usize>>()
    };

    // Three matters, in the caller's order, for each of two births' four
    // years.
    let asked = ask(r#"{"through":4,"place":"birth","matters":[10,1,7]}"#).unwrap();
    let counts = ints(&asked, "annual_charts", "matter_count");
    assert_eq!(counts, vec![3; 8]);
    let houses = ints(&asked, "year_matters", "house");
    assert_eq!(houses, [10, 1, 7].repeat(8));
    let same = ints(&asked, "year_matters", "same_lord");
    let held_counts = ints(&asked, "year_matters", "held_count");
    let unanswered = ints(&asked, "year_matters", "unanswered");
    let yogas = ints(&asked, "matter_yogas", "yoga");
    let by_pair = ints(&asked, "matter_yogas", "by_pair");
    let leg_counts = ints(&asked, "matter_yogas", "leg_count");

    // Ragged three deep, each count exactly the rows under it.
    assert_eq!(held_counts.iter().sum::<usize>(), yogas.len());
    assert!(leg_counts.iter().all(|legs| *legs == 0 || *legs == 2));
    assert_eq!(
        leg_counts.iter().sum::<usize>(),
        ints(&asked, "matter_legs", "faster").len()
    );
    assert!(yogas.iter().all(|yoga| *yoga < 16));
    // The façade reads the states and the build computes all sixteen, so
    // no matter carries an unanswered bit.
    assert!(unanswered.iter().all(|bits| *bits == 0));

    // The first house has no pair: its lord is the lagnesha. Only the two
    // chart facts can hold there, and nothing is made by a pair it lacks.
    let mut from = 0;
    for (row, count) in held_counts.iter().enumerate() {
        let block = from..from + count;
        if houses[row] == 1 {
            assert_eq!(same[row], 1, "row {row}");
        }
        if same[row] == 1 {
            assert!(yogas[block.clone()].iter().all(|yoga| *yoga <= 1));
            assert!(by_pair[block.clone()].iter().all(|flag| *flag == 0));
        }
        from += count;
    }
    // Ikabala and Induvara are facts about the chart, so each year's three
    // matters agree on them.
    let facts = |row: usize| {
        let start: usize = held_counts[..row].iter().sum();
        yogas[start..start + held_counts[row]]
            .iter()
            .filter(|yoga| **yoga <= 1)
            .copied()
            .collect::<Vec<usize>>()
    };
    for year in 0..8 {
        assert_eq!(facts(3 * year), facts(3 * year + 1));
        assert_eq!(facts(3 * year), facts(3 * year + 2));
    }
    // What the matters were judged on crosses beside them: the seven's
    // bits and no others.
    for name in ["retrograde", "combust"] {
        assert!(
            ints(&asked, "annual_charts", name)
                .iter()
                .all(|bits| *bits < 1 << 7)
        );
    }

    // `"all"` is the twelve, first to twelfth, every year.
    let all = ask(r#"{"through":2,"place":"birth","matters":"all"}"#).unwrap();
    assert_eq!(ints(&all, "annual_charts", "matter_count"), vec![12; 4]);
    assert_eq!(
        ints(&all, "year_matters", "house"),
        (1..=12).collect::<Vec<usize>>().repeat(4)
    );
    // Tambira's knob is read: letting either lord move can only add.
    let tambira = |bytes: &[u8]| {
        ints(bytes, "matter_yogas", "yoga")
            .iter()
            .filter(|yoga| **yoga == TsYearYoga::Tambira as usize)
            .count()
    };
    let either =
        ask(r#"{"through":2,"place":"birth","matters":"all","yogas":{"tambira":"EITHER_LORD"}}"#)
            .unwrap();
    assert!(tambira(&either) >= tambira(&all));
    // And the Moon's: read only as waxing, it can only take a Kuttha away.
    let kuttha = |bytes: &[u8]| {
        ints(bytes, "matter_yogas", "yoga")
            .iter()
            .filter(|yoga| **yoga == TsYearYoga::Kuttha as usize)
            .count()
    };
    let waxing =
        ask(r#"{"through":2,"place":"birth","matters":"all","yogas":{"moonBenefic":"WAXING"}}"#)
            .unwrap();
    assert!(kuttha(&waxing) <= kuttha(&all));

    // Not asked, nothing answered: no rows, and every count nought.
    let unasked = ask(r#"{"through":2,"place":"birth"}"#).unwrap();
    assert!(
        ints(&unasked, "year_matters", "house").is_empty(),
        "{:?}",
        ints(&unasked, "year_matters", "house")
    );
    assert!(
        ints(&unasked, "annual_charts", "matter_count")
            .iter()
            .all(|n| *n == 0)
    );
    // An empty list asks for nothing and is answered with nothing.
    let empty = ask(r#"{"through":2,"place":"birth","matters":[]}"#).unwrap();
    assert!(
        ints(&empty, "year_matters", "house").is_empty(),
        "{:?}",
        ints(&empty, "year_matters", "house")
    );

    // Both rule records read the boundary's one casing.
    ask(r#"{"through":2,"place":"birth","varshesha":{"noneAspects":"ANNUAL_LAGNA_LORD"}}"#)
        .unwrap();
    ask(r#"{"through":2,"place":"birth","matters":[7],"yogas":{"weakBelow":14400,"strongFrom":43200}}"#)
        .unwrap();

    // Each refusal names the field the caller wrote.
    let refused = |json: &str, field: &str, says: &str| {
        let error = ask(json).expect_err(json);
        assert_eq!(error.2.as_deref(), Some(field), "{error:?}");
        assert!(error.1.contains(says), "{error:?}");
    };
    refused(r#"{"through":2,"matters":[7]}"#, "varsha.matters", "place");
    refused(
        r#"{"through":2,"place":"birth","matters":[7,7]}"#,
        "varsha.matters",
        "twice",
    );
    refused(
        r#"{"through":2,"place":"birth","matters":[13]}"#,
        "varsha.matters",
        "13",
    );
    refused(
        r#"{"through":2,"place":"birth","matters":"some"}"#,
        "varsha.matters",
        "\"all\"",
    );
    refused(
        r#"{"through":2,"place":"birth","matters":[7],"yogas":{"weakBelow":43200,"strongFrom":14400}}"#,
        "varsha.yogas.strongFrom",
        "both",
    );
    refused(
        r#"{"through":2,"place":"birth","matters":[7],"yogas":{"moonBenefic":"full"}}"#,
        "varsha.yogas.moonBenefic",
        "full",
    );
    refused(
        r#"{"through":2,"place":"birth","varshesha":{"none_aspects":"ANNUAL_LAGNA_LORD"}}"#,
        "varsha.varshesha.none_aspects",
        "none_aspects",
    );
}

/// The lord of each sign, Aries first, by `graha` id.
const SIGN_LORDS: [usize; 12] = [2, 5, 3, 1, 0, 3, 5, 2, 4, 6, 6, 4];

/// A year's chart answers the **sahams** the request names, in its order,
/// each where it fell and what it fell in, ragged by `saham_count`
/// (`03-design/tajika-sahams.md`).
#[test]
fn a_years_chart_answers_the_sahams_it_was_asked_for() {
    let ctx = Ctx::with_ephemeris(0, TsEphemeris::Builtin, None, None, None).unwrap();
    let schema = schemas::charts();
    let ask = |json: &str| annual_blob(&ctx, json);
    let cells = |bytes: &[u8], section: &str, name: &str| {
        Reader::parse(bytes, &schema)
            .unwrap()
            .column(section, name)
            .unwrap()
            .iter()
            .map(|cell| cell.as_f64())
            .collect::<Vec<f64>>()
    };
    let ints = |bytes: &[u8], section: &str, name: &str| {
        cells(bytes, section, name)
            .iter()
            .map(|cell| *cell as usize)
            .collect::<Vec<usize>>()
    };

    // A request names a saham by the key every binding reads it back as
    // (`tests/keys.rs` holds the two to one spelling), so a caller can
    // hand back what it was given. Three sahams, in the caller's order, for each of two births' three
    // years.
    let order = [TsSaham::Raja, TsSaham::KaryaSiddhi, TsSaham::Mrityu].map(|one| one as usize);
    let asked =
        ask(r#"{"through":3,"place":"birth","sahams":["RAJA","KARYA_SIDDHI","MRITYU"]}"#).unwrap();
    assert_eq!(ints(&asked, "annual_charts", "saham_count"), vec![3; 6]);
    assert_eq!(ints(&asked, "year_sahams", "saham"), order.repeat(6));
    let longitudes = cells(&asked, "year_sahams", "longitude_deg");
    let signs = ints(&asked, "year_sahams", "sign");
    let lords = ints(&asked, "year_sahams", "lord");
    let houses = ints(&asked, "year_sahams", "house");
    let lagnas = cells(&asked, "annual_charts", "lagna_deg");
    for (row, deg) in longitudes.iter().enumerate() {
        assert!((0.0..360.0).contains(deg), "row {row}: {deg}");
        // The sign is the longitude's, its lord that sign's, and the house
        // counted by whole signs from the year's own lagna.
        assert_eq!(signs[row], (deg / 30.0) as usize, "row {row}");
        assert_eq!(lords[row], SIGN_LORDS[signs[row]], "row {row}");
        let lagna = (lagnas[row / 3] / 30.0) as usize;
        assert_eq!(houses[row], (signs[row] + 12 - lagna) % 12 + 1, "row {row}");
    }

    // `"all"` is the forty-one in the source's order, every year, and the
    // three pairs that share a formula land together.
    let all = ask(r#"{"through":2,"place":"birth","sahams":"all"}"#).unwrap();
    assert_eq!(ints(&all, "annual_charts", "saham_count"), vec![41; 4]);
    assert_eq!(
        ints(&all, "year_sahams", "saham"),
        (0..41).collect::<Vec<usize>>().repeat(4)
    );
    let every = cells(&all, "year_sahams", "longitude_deg");
    for year in 0..4 {
        for (one, other) in [
            (TsSaham::Guru, TsSaham::Vidya),
            (TsSaham::Pitri, TsSaham::Raja),
            (TsSaham::Kali, TsSaham::Kshama),
        ] {
            let at = |saham: TsSaham| every[41 * year + saham as usize];
            assert_eq!(
                at(one).to_bits(),
                at(other).to_bits(),
                "{one:?} and {other:?}"
            );
        }
    }

    // The rules are read: never adding a sign adds none.
    let never =
        ask(r#"{"through":2,"place":"birth","sahams":"all","sahamRules":{"addSign":"NEVER"}}"#)
            .unwrap();
    assert!(
        ints(&never, "year_sahams", "added_sign")
            .iter()
            .all(|flag| *flag == 0)
    );
    ask(r#"{"through":2,"place":"birth","sahams":["MRITYU"],"sahamRules":{"houses":"EQUAL","roga":"SATURN"}}"#)
        .unwrap();

    // Not asked, nothing answered; an empty list asks for nothing.
    for json in [
        r#"{"through":2,"place":"birth"}"#,
        r#"{"through":2,"place":"birth","sahams":[]}"#,
    ] {
        let bytes = ask(json).unwrap();
        assert!(ints(&bytes, "year_sahams", "saham").is_empty(), "{json}");
        assert!(
            ints(&bytes, "annual_charts", "saham_count")
                .iter()
                .all(|n| *n == 0)
        );
    }

    // Each refusal names the field the caller wrote.
    let refused = |json: &str, field: &str, says: &str| {
        let error = ask(json).expect_err(json);
        assert_eq!(error.2.as_deref(), Some(field), "{error:?}");
        assert!(error.1.contains(says), "{error:?}");
    };
    refused(
        r#"{"through":2,"place":"birth","sahams":["PUNYA","PUNYA"]}"#,
        "varsha.sahams",
        "twice",
    );
    refused(
        r#"{"through":2,"place":"birth","sahams":["pnya"]}"#,
        "varsha.sahams[0]",
        "pnya",
    );
    refused(
        r#"{"through":2,"place":"birth","sahams":"some"}"#,
        "varsha.sahams",
        "\"all\"",
    );
    refused(
        r#"{"through":2,"place":"birth","sahams":["PUNYA"],"sahamRules":{"add_sign":"NEVER"}}"#,
        "varsha.sahamRules.add_sign",
        "add_sign",
    );
    refused(
        r#"{"through":2,"place":"birth","sahams":["PUNYA"],"sahamRules":{"houses":"placidus"}}"#,
        "varsha.sahamRules.houses",
        "placidus",
    );

    // Each saham carries its strength, clause by clause, over the enums
    // the generator reads, and seven rows under it, one per planet.
    let asked =
        ask(r#"{"through":3,"place":"birth","sahams":["RAJA","KARYA_SIDDHI","MRITYU"]}"#).unwrap();
    let lords = ints(&asked, "year_sahams", "lord");
    let strong = ints(&asked, "year_sahams", "strong");
    let weak = ints(&asked, "year_sahams", "weak");
    let seven = ints(&asked, "year_saham_seven", "graha");
    assert_eq!(seven.len(), 7 * strong.len());
    assert_eq!(seven[..7], [0, 1, 2, 3, 4, 5, 6]);
    let company = ints(&asked, "year_saham_seven", "company");
    let apart = 1 << TsSahamWeak::LordApart as usize;
    let near = (1 << TsSahamStrong::LordConjoins as usize)
        | (1 << TsSahamStrong::LordAspectsSaham as usize);
    for (row, (strong, weak)) in strong.iter().zip(&weak).enumerate() {
        assert!(*strong < 1 << 12 && *weak < 1 << 5, "row {row}");
        // The two (c) clauses negate each other, so exactly one holds.
        assert_ne!(strong & near != 0, weak & apart != 0, "row {row}");
        // The lord's own row of the seven is its company exactly when it
        // conjoins the saham.
        let conjoins = strong & (1 << TsSahamStrong::LordConjoins as usize) != 0;
        assert_eq!(company[7 * row + lords[row]] == 1, conjoins, "row {row}");
    }
    assert!(
        ints(&asked, "year_sahams", "node_axis")
            .iter()
            .all(|axis| *axis <= 1),
        "a founded chart places the nodes"
    );

    // The Harsha bala comes with every founded year: seven rows each, a
    // total of five units a part held.
    let totals = ints(&asked, "year_harsha", "total");
    assert_eq!(totals.len(), 7 * 6);
    let parts: usize = ["sthana", "uchcha_swakshetra", "stri_purusha", "dina_ratri"]
        .iter()
        .map(|part| ints(&asked, "year_harsha", part).iter().sum::<usize>())
        .sum();
    assert_eq!(totals.iter().sum::<usize>(), 5 * parts);

    // The births' own sahams come beside the years', and need no place:
    // without one they are all that is answered.
    assert_eq!(ints(&asked, "cast", "natal_saham_count"), vec![3, 3]);
    assert_eq!(ints(&asked, "natal_sahams", "saham"), order.repeat(2));
    let births = ask(r#"{"through":2,"sahams":["PUNYA"]}"#).unwrap();
    assert_eq!(ints(&births, "cast", "natal_saham_count"), vec![1, 1]);
    assert_eq!(ints(&births, "natal_saham_seven", "graha").len(), 14);
    assert!(
        ints(&births, "year_sahams", "saham").is_empty(),
        "{:?}",
        ints(&births, "year_sahams", "saham")
    );
    // A birth has no year lord.
    let with_year_lord = 1 << TsSahamStrong::WithYearLord as usize;
    assert!(
        ints(&births, "natal_sahams", "strong")
            .iter()
            .all(|bits| bits & with_year_lord == 0)
    );
    // Their readings are named and read.
    ask(r#"{"through":1,"place":"birth","sahams":"all","sahamStrength":{"natures":"PARASHARI","friendship":"NATURAL","weakBelow":14400},"harshaRules":{"venus":"TWELFTH"}}"#)
        .unwrap();
    refused(
        r#"{"through":1,"sahams":["PUNYA"],"sahamStrength":{"natures":"vedic"}}"#,
        "varsha.sahamStrength.natures",
        "vedic",
    );
    refused(
        r#"{"through":1,"place":"birth","harshaRules":{"venus":"sixth"}}"#,
        "varsha.harshaRules.venus",
        "sixth",
    );
}

/// A year's chart answers the **annual dashas** the request names, in its
/// order: each opens on its return and closes on the next, its ring and
/// its periods ragged under it, and its mahadashas and antardashas run end
/// to end (`03-design/annual-dashas.md`).
#[test]
fn a_years_chart_answers_the_annual_dashas_it_was_asked_for() {
    const PATYAYINI: usize = 36;
    const MUDDA: usize = 37;
    const VARSHA_YOGINI: usize = 39;
    let ctx = Ctx::with_ephemeris(0, TsEphemeris::Builtin, None, None, None).unwrap();
    let schema = schemas::charts();
    let ask = |json: &str| annual_blob(&ctx, json);
    let cells = |bytes: &[u8], section: &str, name: &str| {
        Reader::parse(bytes, &schema)
            .unwrap()
            .column(section, name)
            .unwrap()
            .iter()
            .map(|cell| cell.as_f64())
            .collect::<Vec<f64>>()
    };
    let ints = |bytes: &[u8], section: &str, name: &str| {
        cells(bytes, section, name)
            .iter()
            .map(|cell| *cell as usize)
            .collect::<Vec<usize>>()
    };
    for (system, id) in [
        (teistro::catalogue::DashaSystem::Mudda, MUDDA),
        (teistro::catalogue::DashaSystem::Patyayini, PATYAYINI),
        (teistro::catalogue::DashaSystem::VarshaYogini, VARSHA_YOGINI),
    ] {
        assert_eq!(usize::from(system as u16), id, "{system:?}");
    }

    // Two systems in the caller's order, for each of two births' three
    // years.
    // A system is named by its key, full as every binding reads it back
    // or bare as Rust spells it.
    let asked = ask(r#"{"through":3,"place":"birth","dashas":["dasha_system.MUDDA","PATYAYINI"]}"#)
        .unwrap();
    assert_eq!(ints(&asked, "annual_charts", "dasha_count"), vec![2; 6]);
    assert_eq!(
        ints(&asked, "year_dashas", "system"),
        [MUDDA, PATYAYINI].repeat(6)
    );
    let seeded = ints(&asked, "year_dashas", "seeded");
    let seeds = ints(&asked, "year_dashas", "seed");
    let firsts = ints(&asked, "year_dashas", "first");
    let remaining = cells(&asked, "year_dashas", "remaining");
    let opens = cells(&asked, "year_dashas", "from_jd");
    let closes = cells(&asked, "year_dashas", "to_jd");
    let share_counts = ints(&asked, "year_dashas", "share_count");
    let period_counts = ints(&asked, "year_dashas", "period_count");
    let returns = cells(&asked, "praveshas", "jd");
    let lords = ints(&asked, "year_dasha_shares", "lord");
    let has_sign = ints(&asked, "year_dasha_shares", "has_sign");
    let weights = cells(&asked, "year_dasha_shares", "weight");
    let levels = ints(&asked, "year_dasha_periods", "level");
    let period_signed = ints(&asked, "year_dasha_periods", "has_sign");
    let from = cells(&asked, "year_dasha_periods", "from_jd");
    let to = cells(&asked, "year_dasha_periods", "to_jd");
    assert_eq!(lords.len(), share_counts.iter().sum::<usize>());
    assert_eq!(levels.len(), period_counts.iter().sum::<usize>());
    // The Sun's clock closes a year on the next return, each found to the
    // search's tolerance: twice 1e-7 days apart at most.
    let bound = 2.0 * 1e-7;
    let (mut share, mut period) = (0, 0);
    for row in 0..12 {
        let year = row / 2;
        let mudda = row % 2 == 0;
        assert_eq!(opens[row].to_bits(), returns[year].to_bits(), "row {row}");
        if year % 3 != 2 {
            assert!((closes[row] - returns[year + 1]).abs() < bound, "row {row}");
        }
        // The Mudda is seeded by the birth Moon, the same every year, and
        // its ring advances one lord a year; the Patyayini is read from
        // the year's own chart, and its lagna is the one sign's share.
        let ring = share..share + share_counts[row];
        if mudda {
            assert_eq!((seeded[row], share_counts[row]), (1, 9), "row {row}");
            assert!(remaining[row] > 0.0 && remaining[row] <= 1.0, "row {row}");
            assert!(has_sign[ring.clone()].iter().all(|flag| *flag == 0));
            if year % 3 != 0 {
                assert_eq!(seeds[row], seeds[row - 2], "row {row}");
                assert_eq!(firsts[row], (firsts[row - 2] + 1) % 9, "row {row}");
            }
        } else {
            assert_eq!((seeded[row], seeds[row], share_counts[row]), (0, 0, 8));
            assert!(remaining[row].is_nan(), "row {row}");
            assert_eq!(has_sign[ring.clone()].iter().sum::<usize>(), 1, "row {row}");
        }
        assert!(weights[ring].iter().all(|weight| *weight >= 0.0));
        share += share_counts[row];
        // The mahadashas run end to end from the return to the close, and
        // each one's antardashas end to end across it.
        let periods = period..period + period_counts[row];
        let mut edge = opens[row];
        let mut parent: Option<(f64, f64)> = None;
        let mut inner = 0.0_f64;
        for k in periods.clone() {
            assert!(from[k] < to[k], "row {row} period {k}");
            if levels[k] == 1 {
                if let Some((_, end)) = parent {
                    assert_eq!(inner.to_bits(), end.to_bits(), "row {row} period {k}");
                }
                assert_eq!(from[k].to_bits(), edge.to_bits(), "row {row} period {k}");
                edge = to[k];
                parent = Some((from[k], to[k]));
                inner = from[k];
            } else {
                assert_eq!(levels[k], 2, "row {row} period {k}");
                assert_eq!(from[k].to_bits(), inner.to_bits(), "row {row} period {k}");
                inner = to[k];
            }
        }
        assert_eq!(edge.to_bits(), closes[row].to_bits(), "row {row}");
        let signs = period_signed[periods.clone()]
            .iter()
            .filter(|flag| **flag == 1);
        assert_eq!(signs.count() > 0, !mudda, "row {row}");
        period = periods.end;
    }

    // `"all"` is the three in the catalogue's order.
    let all = ask(r#"{"through":1,"place":"birth","dashas":"all"}"#).unwrap();
    assert_eq!(
        ints(&all, "year_dashas", "system"),
        [PATYAYINI, MUDDA, VARSHA_YOGINI].repeat(2)
    );

    // The rules are read: a year of 360 days is exactly that long, and one
    // level lists the mahadashas alone.
    let days = ask(
        r#"{"through":1,"place":"birth","dashas":["MUDDA"],"dashaRules":{"clock":{"DAYS":360},"depth":1}}"#,
    )
    .unwrap();
    let (opens, closes) = (
        cells(&days, "year_dashas", "from_jd"),
        cells(&days, "year_dashas", "to_jd"),
    );
    for (open, close) in opens.iter().zip(&closes) {
        assert_eq!((close - open).to_bits(), 360.0_f64.to_bits());
    }
    assert!(
        ints(&days, "year_dasha_periods", "level")
            .iter()
            .all(|level| *level == 1)
    );
    let whole =
        ask(r#"{"through":1,"place":"birth","dashas":["MUDDA"],"dashaRules":{"balance":"WHOLE"}}"#)
            .unwrap();
    assert!(
        cells(&whole, "year_dashas", "remaining")
            .iter()
            .all(|r| r.is_nan())
    );
    // Every reading the rules name is accepted as a binding spells it.
    for rules in [
        r#"{"clock":"EVEN","balance":"ENTRY_MOON","measure":"SPATIAL","birthPeriod":"ELAPSED","depth":3}"#,
        r#"{"clock":"SUN_DEGREES","balance":"NATAL_MOON","measure":"TEMPORAL","birthPeriod":"COMPRESSED"}"#,
    ] {
        let json =
            format!(r#"{{"through":1,"place":"birth","dashas":"all","dashaRules":{rules}}}"#);
        ask(&json).expect(&json);
    }

    // Not asked, nothing answered; an empty list asks for nothing.
    for json in [
        r#"{"through":2,"place":"birth"}"#,
        r#"{"through":2,"place":"birth","dashas":[]}"#,
    ] {
        let bytes = ask(json).unwrap();
        assert!(ints(&bytes, "year_dashas", "system").is_empty(), "{json}");
        assert!(
            ints(&bytes, "year_dasha_periods", "level").is_empty(),
            "{json}"
        );
        assert!(
            ints(&bytes, "annual_charts", "dasha_count")
                .iter()
                .all(|n| *n == 0)
        );
    }

    // Each refusal names the field the caller wrote.
    let refused = |json: &str, field: &str, says: &str| {
        let error = ask(json).expect_err(json);
        assert_eq!(error.2.as_deref(), Some(field), "{error:?}");
        assert!(error.1.contains(says), "{error:?}");
    };
    refused(
        r#"{"through":2,"dashas":["MUDDA"]}"#,
        "varsha.dashas",
        "place",
    );
    refused(
        r#"{"through":2,"place":"birth","dashas":["MUDDA","dasha_system.MUDDA"]}"#,
        "varsha.dashas",
        "twice",
    );
    refused(
        r#"{"through":2,"place":"birth","dashas":["VIMSHOTTARI"]}"#,
        "varsha.dashas",
        "not an annual dasha",
    );
    refused(
        r#"{"through":2,"place":"birth","dashas":["MUDA"]}"#,
        "varsha.dashas",
        "did you mean `MUDDA`",
    );
    refused(
        r#"{"through":2,"place":"birth","dashas":["graha.MUDDA"]}"#,
        "varsha.dashas",
        "graha.MUDDA",
    );
    refused(
        r#"{"through":2,"place":"birth","dashas":"some"}"#,
        "varsha.dashas",
        "\"all\"",
    );
    refused(
        r#"{"through":2,"place":"birth","dashas":["MUDDA"],"dashaRules":{"clock":{"DAYS":0}}}"#,
        "varsha.dashaRules.clock",
        "more than none",
    );
    refused(
        r#"{"through":2,"place":"birth","dashas":["MUDDA"],"dashaRules":{"clok":"EVEN"}}"#,
        "varsha.dashaRules.clok",
        "clok",
    );
}

/// A consumer's **sign-based** system crosses the same way: registered
/// through `options.dashas_json` under `"kernel":"RASHI"`, asked for by the
/// id `ts_key_parse` gives, and answered in the `dashas` section with the
/// periods the catalogued row it copies would give
/// (`03-design/dasha-coverage-measured.md`).
#[test]
fn a_consumer_sign_based_system_registers_and_crosses_by_its_id() {
    let chara = r#"{"kernel":"RASHI","key":"ACME_CHARA","start":"LAGNA","order":"CONSECUTIVE","length":"COUNT_TO_LORD","named_lord":"STRONGER"}"#;
    let ctx = Ctx::with_dashas(&format!("[{chara}]")).expect("a sign-based system registers");
    let full = CString::new("dasha_system.ACME_CHARA").unwrap();
    let mut id = 0u32;
    // SAFETY: a live handle and valid slots.
    assert_eq!(
        unsafe { ts_key_parse(ctx.handle, full.as_ptr(), &raw mut id) },
        Status::Ok,
        "{:?}",
        ctx.last_error()
    );

    let instants = [2_451_545.0];
    let dashas = [u16::try_from(id & 0xFFFF).unwrap(), DashaSystem::Chara.id()];
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
            varsha_json: ptr::null(),
            gochar_json: ptr::null(),
            hits_json: ptr::null(),
            sade_sati_json: ptr::null(),
            kp_json: ptr::null(),
            dignities_json: ptr::null(),
            fortitudes_json: ptr::null(),
            lots_json: ptr::null(),
            considerations_json: ptr::null(),
            perfection_json: ptr::null(),
            progressions_json: ptr::null(),
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
        (0x8000, i64::from(DashaSystem::Chara.id()))
    );
    // The consumer's row is Chara's, so the two agree period for period.
    let counts = reader.column("dashas", "period_count").unwrap();
    assert_eq!(counts[0].as_i64(), counts[1].as_i64());

    // A sign-based row is refused by its own field, as a seeded one is.
    let refused = |rows: String| match Ctx::with_dashas(&rows) {
        Ok(_) => panic!("{rows} registered"),
        Err(record) => record,
    };
    let thirteenth = refused(format!(
        "[{}]",
        chara.replacen(
            "\"named_lord\":\"STRONGER\"",
            "\"named_lord\":\"STRONGER\",\"stronger_of\":[1,13]",
            1
        )
    ));
    assert_eq!(
        thirteenth.2.as_deref(),
        Some("options.dashas_json[0].stronger_of[1]"),
        "{thirteenth:?}"
    );
    // A field is spelt as the document spells it, and a misspelt one is
    // refused by name rather than read as its default.
    let camel = refused(format!(
        "[{}]",
        chara.replacen("\"named_lord\"", "\"namedLord\"", 1)
    ));
    assert_eq!(
        camel.2.as_deref(),
        Some("options.dashas_json[0].namedLord"),
        "{camel:?}"
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
    let rules = CString::new(r#"{"shipped": ["NABHASAS"], "longevity": true}"#).unwrap();
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
            varsha_json: ptr::null(),
            gochar_json: ptr::null(),
            hits_json: ptr::null(),
            sade_sati_json: ptr::null(),
            kp_json: ptr::null(),
            dignities_json: ptr::null(),
            fortitudes_json: ptr::null(),
            lots_json: ptr::null(),
            considerations_json: ptr::null(),
            perfection_json: ptr::null(),
            progressions_json: ptr::null(),
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
        assert!(!present.is_empty(), "`present` is empty");
        for held in present {
            // A rule by its key, never the whole rule again.
            assert!(held["rule"].is_string(), "{held}");
            assert_eq!(held["result"]["present"], serde_json::Value::Bool(true));
        }
        assert!(chart["longevity"]["ayurdaya"]["pindayu"]["years"].is_number());
        assert!(chart.get("houses").is_none(), "houses were not asked for");
    }

    // Each chart's own content hash rides beside the batch's, which hashes
    // the list: what a binding's `found(one)` stamps its chart with. They
    // are the hashes the façade seals over each chart, computed here
    // independently of the boundary.
    let hashes = reader.text("content_hashes").unwrap();
    assert_eq!(hashes.len(), 64 * instants.len());
    let provenance: serde_json::Value =
        serde_json::from_str(reader.text("provenance_json").unwrap()).unwrap();
    let sdk = teistro::Context::builder()
        .profile("conformance-baseline")
        .ephemeris([teistro::Ephemeris::Builtin])
        .build()
        .unwrap();
    let set = teistro::RuleRequest::from_json(rules.to_str().unwrap())
        .unwrap()
        .rule_set()
        .unwrap();
    let place = teistro::quantity::Place::new(
        teistro::quantity::Latitude::try_new(27.7172).unwrap(),
        teistro::quantity::Longitude::try_new(85.324).unwrap(),
        teistro::quantity::Altitude::try_new(1400.0).unwrap(),
    );
    let read = sdk
        .chart()
        .interpreted(
            &instants.map(teistro::quantity::JulianDay::literal),
            &teistro::ChartRequest::at(
                place,
                teistro::UtcOffset::try_from_seconds(20_700).unwrap(),
            ),
            Some(&set),
            teistro::PlanRequest::default(),
        )
        .unwrap();
    assert_eq!(
        provenance["content_hash"].as_str(),
        Some(read.provenance.content_hash.to_string().as_str())
    );
    for (at, chart) in read.value.iter().enumerate() {
        let own = &hashes[64 * at..64 * at + 64];
        assert_eq!(own, chart.content_hash.to_string(), "chart {at}");
        assert_ne!(Some(own), provenance["content_hash"].as_str());
    }

    // The same request without rules carries an empty section.
    let plain = TsChartRequest {
        rules_json: ptr::null(),
        interpret_json: ptr::null(),
        varsha_json: ptr::null(),
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
    assert!(
        plain_reader.bytes("rules").unwrap().is_empty(),
        "{:?}",
        plain_reader.bytes("rules").unwrap()
    );

    // A rule that does not read is refused from the request's root.
    let broken = CString::new(r#"{"rules": [{"key": "X", "category": "raja"}]}"#).unwrap();
    let refused = TsChartRequest {
        rules_json: broken.as_ptr(),
        interpret_json: ptr::null(),
        varsha_json: ptr::null(),
        ..request
    };
    let mut nothing = TsBlob::empty();
    // SAFETY: as above.
    let status = unsafe { ts_chart_found(ctx.handle, &raw const refused, &raw mut nothing) };
    assert_eq!(status, Status::InvalidArg);
    let record = ctx.last_error();
    assert_eq!(record.2.as_deref(), Some("rules.rules[0]"), "{record:?}");
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
    let rules = CString::new(r#"{"shipped": ["NABHASAS"]}"#).unwrap();
    let plans = CString::new(
        r#"{"placements": true, "readings": true, "strength": true, "houses": true,
            "positions": true, "aspects": true, "conditions": true, "karakas": true,
            "chalit": true}"#,
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
            varsha_json: ptr::null(),
            gochar_json: ptr::null(),
            hits_json: ptr::null(),
            sade_sati_json: ptr::null(),
            kp_json: ptr::null(),
            dignities_json: ptr::null(),
            fortitudes_json: ptr::null(),
            lots_json: ptr::null(),
            considerations_json: ptr::null(),
            perfection_json: ptr::null(),
            progressions_json: ptr::null(),
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
        // `RuleInputs`, as the placements do, so one section serves every
        // composer that reads what a graha *is* — and `sections` asked for
        // none of it.
        let conditions = chart["conditions"].as_array().unwrap();
        assert!(
            conditions.len() >= 9 * 2,
            "a dignity and a navamsha for each of the nine"
        );
        let karakas = chart["karakas"].as_array().unwrap();
        assert!(!karakas.is_empty(), "every chart ranks its chara karakas");
        // The chalit reads the chart's own grahas, so `sections` asked for
        // nothing on its behalf either; it says only the disagreements.
        let shifts = chart["chalit"].as_array().unwrap();
        assert!(shifts.len() <= 9, "at most one a graha");
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
        varsha_json: ptr::null(),
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
            .is_empty(),
        "{:?}",
        Reader::parse(&plain_bytes, &schema)
            .unwrap()
            .bytes("plans")
            .unwrap()
    );

    // A reading needs rules to say what they answered.
    let alone = CString::new(r#"{"readings": true}"#).unwrap();
    let refused = TsChartRequest {
        rules_json: ptr::null(),
        interpret_json: alone.as_ptr(),
        varsha_json: ptr::null(),
        ..request
    };
    let mut nothing = TsBlob::empty();
    // SAFETY: as above.
    let status = unsafe { ts_chart_found(ctx.handle, &raw const refused, &raw mut nothing) };
    assert_eq!(status, Status::InvalidArg);
    let record = ctx.last_error();
    assert_eq!(
        record.2.as_deref(),
        Some("interpret.readings"),
        "{record:?}"
    );

    // And a composer that is not one is refused beside the ones that are.
    let typo = CString::new(r#"{"readigns": true}"#).unwrap();
    let wrong = TsChartRequest {
        interpret_json: typo.as_ptr(),
        varsha_json: ptr::null(),
        ..request
    };
    let mut never = TsBlob::empty();
    // SAFETY: as above.
    let status = unsafe { ts_chart_found(ctx.handle, &raw const wrong, &raw mut never) };
    assert_eq!(status, Status::InvalidArg);
    let record = ctx.last_error();
    assert_eq!(record.2.as_deref(), Some("interpret"), "{record:?}");
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

/// Every composer asked for **alone** answers, or says here why it cannot.
///
/// This is the maintainer's "no dead ends" rule as a check. A member of
/// `PlanRequest` is a promise that asking for it gets you something, and
/// the section it reads is computed for it: `sections_for` is the one
/// place that mapping lives and nothing held it.
///
/// **Alone, and one at a time**, because asking for several together
/// hides a missing section behind a sibling's — `strength` and `houses`
/// both bring the states that four other composers read, so a member
/// whose own section was forgotten still answers in company.
///
/// What it catches is a member that **refuses** or that **says nothing**.
/// It would not have caught the almanac bug of 2026-09-22 in its own
/// shape, and it is worth being exact about that: there the section was
/// asked for and the *document's* was never turned into the birth's
/// limbs, so `phala` returned an empty plan that this test excuses for a
/// context with no pack loaded. The pass that catches that one is
/// `cargo xtask interpret`'s "every key, emitted at least once", which
/// founds a chart with both corpora and requires every key. The two are
/// different questions — *can it be asked for* and *does it ever say
/// anything* — and both are needed.
///
/// The ones that are legitimately empty carry their reason, and the list
/// fails both ways: a member that says nothing and is not here fails, and
/// one here that says something fails too.
#[test]
fn every_composer_asked_for_alone_answers_or_says_why_not() {
    /// A composer that answers nothing for this chart, and why.
    const SILENT: [(&str, &str); 2] = [
        (
            "phala",
            "it says what a loaded corpus carries and this context has loaded none, which is the \
             composer working rather than failing: a chart composes to the same plan it did \
             before until a consumer asks for the words",
        ),
        (
            "sadeSati",
            "it says what a loaded corpus carries of Saturn's periods, and this context has \
             loaded none, as `phala`",
        ),
    ];

    let ctx = Ctx::with_ephemeris(
        0,
        TsEphemeris::Builtin,
        Some("conformance-baseline"),
        None,
        None,
    )
    .unwrap();
    let instants = [2_447_995.489_583_333_5];
    let rules = CString::new(r#"{"shipped": ["NABHASAS"]}"#).unwrap();
    // Beside the rules `readings` says, the window `sadeSati` says: one
    // instant, since what is under test is that it can be asked for.
    let window = CString::new(r#"{"from": 2460676.5}"#).unwrap();
    let silent: std::collections::BTreeMap<&str, &str> = SILENT.iter().copied().collect();

    for member in teistro::PlanRequest::MEMBERS {
        let plans = CString::new(format!(r#"{{"{member}": true}}"#)).unwrap();
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
                // Nothing asked for: a composer's own section is the
                // library's job, which is the whole point of the check.
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
                varsha_json: ptr::null(),
                gochar_json: ptr::null(),
                hits_json: ptr::null(),
                sade_sati_json: window.as_ptr(),
                kp_json: ptr::null(),
                dignities_json: ptr::null(),
                fortitudes_json: ptr::null(),
                lots_json: ptr::null(),
                considerations_json: ptr::null(),
                perfection_json: ptr::null(),
                progressions_json: ptr::null(),
            },
            |r, s| r.struct_size = s,
        );
        let mut blob = TsBlob::empty();
        // SAFETY: a live context, a valid request and a valid slot.
        assert_eq!(
            unsafe { ts_chart_found(ctx.handle, &raw const request, &raw mut blob) },
            Status::Ok,
            "`{member}` alone: {:?}",
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
        let chart = &composed.as_array().unwrap()[0];
        let items = chart[member]
            .as_array()
            .unwrap_or_else(|| panic!("`{member}` was asked for, so it is present"));

        match (items.is_empty(), silent.get(member)) {
            (true, None) => panic!(
                "`{member}` asked for alone says nothing, and SILENT does not say why — a member \
                 is a promise that asking for it gets you something"
            ),
            (false, Some(why)) => {
                panic!("`{member}` says something now, and SILENT still says: {why}")
            }
            _ => {}
        }
    }
}

/// Asks `ts_panchanga_days` for Kathmandu's days from 2026-11-25 to
/// 12-03, with `muhurta_json` beside them when given, and answers the
/// blob's bytes or the context's record.
fn panchanga_days(ctx: &Ctx, muhurta: Option<&str>) -> Result<Vec<u8>, Record> {
    panchanga_between(ctx, ((11, 25), (12, 3)), muhurta, None)
}

/// Asks `ts_panchanga_days` for Kathmandu's days of 2026 between two
/// (month, day) dates, with `muhurta_json` and `festivals_json` beside
/// them when given.
fn panchanga_between(
    ctx: &Ctx,
    range: ((u8, u8), (u8, u8)),
    muhurta: Option<&str>,
    festivals: Option<&str>,
) -> Result<Vec<u8>, Record> {
    panchanga_asked(ctx, range, (muhurta, festivals), 0)
}

/// Asks `ts_panchanga_days` for Kathmandu's days of 2026 between two
/// (month, day) dates, with `muhurta_json` and `festivals_json` beside
/// them when given and the `sections` bits set.
fn panchanga_asked(
    ctx: &Ctx,
    range: ((u8, u8), (u8, u8)),
    beside: (Option<&str>, Option<&str>),
    sections: u32,
) -> Result<Vec<u8>, Record> {
    panchanga_asked_in(ctx, 2026, range, beside, sections)
}

/// [`panchanga_asked`] over a range in another year.
fn panchanga_asked_in(
    ctx: &Ctx,
    year: i32,
    ((from_month, from_day), (to_month, to_day)): ((u8, u8), (u8, u8)),
    (muhurta, festivals): (Option<&str>, Option<&str>),
    sections: u32,
) -> Result<Vec<u8>, Record> {
    let muhurta = muhurta.map(|text| CString::new(text).unwrap());
    let festivals = festivals.map(|text| CString::new(text).unwrap());
    let request = sized(
        TsPanchangaRequest {
            struct_size: 0,
            calendar: Calendar::Gregorian.id(),
            reserved: 0,
            from_year: year,
            from_month,
            from_day,
            to_month,
            to_day,
            to_year: year,
            latitude_deg: 27.7172,
            longitude_deg: 85.324,
            altitude_m: 1400.0,
            utc_offset_seconds: 20_700,
            sections,
            muhurta_json: muhurta.as_ref().map_or(ptr::null(), |text| text.as_ptr()),
            festivals_json: festivals.as_ref().map_or(ptr::null(), |text| text.as_ptr()),
        },
        |r, s| r.struct_size = s,
    );
    let mut blob = TsBlob::empty();
    // SAFETY: a live context, a valid request and a valid slot.
    let status = unsafe { ts_panchanga_days(ctx.handle, &raw const request, &raw mut blob) };
    if status != Status::Ok {
        return Err(ctx.last_error());
    }
    // SAFETY: the library wrote `len` bytes.
    let bytes = unsafe { core::slice::from_raw_parts(blob.data, blob.len) }.to_vec();
    // SAFETY: a descriptor the library wrote.
    unsafe { ts_blob_free(&raw mut blob) };
    Ok(bytes)
}

/// The clause kinds the search below never reaches, each with why:
/// refused both ways, so a kind it starts reaching, or one it stops
/// reaching, is seen rather than assumed (`muhurta-at-the-boundary.md`
/// §4).
const UNREACHED: [(&str, &str); 5] = [
    (
        "MONTH",
        "every day is in Margashirsha, which Raman grades ordinary, and a middling grade is not reported",
    ),
    (
        "SOLAR_MONTH",
        "Raman keys a marriage's month lunar (C161); the Sun's sign is the baseline's rules",
    ),
    (
        "KARTARI",
        "the malefics stand in Scorpio, Leo, Pisces and Aquarius, no two of them two signs apart, so no lagna has one either side; Rahu enters Capricorn late on 12-03, after the range",
    ),
    (
        "KENDRA_BENEFICS",
        "the Sun in Scorpio, Mars in Leo and Saturn in Pisces fit no lagna's 3rd, 6th and 11th (C167)",
    ),
    (
        "UNWANTED_PLACEMENT",
        "Raman's marriage lists no graha out of a house; the thread ceremony's search below reaches it",
    ),
];

#[test]
fn a_panchanga_request_answers_a_muhurta_over_its_own_days() {
    use teistro::muhurta::Answer;
    use teistro::muhurta::clause::ClauseKey;

    let ctx = Ctx::with_ephemeris(0, TsEphemeris::Builtin, None, None, None).unwrap();
    let text = r#"{"rules":"RAMAN_MARRIAGE","native":{"star":"ROHINI","moonSign":"rashi.TAURUS","lagna":"LEO"},"daysWithWindows":11,"most":1000}"#;
    let with = panchanga_days(&ctx, Some(text)).unwrap();
    let without = panchanga_days(&ctx, None).unwrap();
    let schema = schemas::panchanga();
    let (with, without) = (
        Reader::parse(&with, &schema).unwrap(),
        Reader::parse(&without, &schema).unwrap(),
    );

    // The days are the ones asked without a search: founded once, and
    // the same.
    assert_eq!(
        with.text("content_hashes").unwrap(),
        without.text("content_hashes").unwrap()
    );
    assert_eq!(without.text("muhurta").unwrap(), "");

    // The section is the façade's answer, its members written in full,
    // and the envelope is sealed over what it holds.
    let envelope: serde_json::Value = serde_json::from_str(with.text("muhurta").unwrap()).unwrap();
    let provenance: teistro::Provenance =
        serde_json::from_value(envelope["provenance"].clone()).unwrap();
    assert_eq!(
        provenance.content_hash,
        teistro::content_hash(&envelope["value"])
    );
    let answer: Answer = serde_json::from_value(envelope["value"].clone()).unwrap();
    let sdk = teistro::Context::builder()
        .ephemeris([teistro::Ephemeris::Builtin])
        .build()
        .unwrap();
    let date = |month, day| teistro::CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    let expected = sdk
        .almanac()
        .muhurta(
            &date(11, 25),
            &date(12, 3),
            &teistro::quantity::Place::try_from_degrees(27.7172, 85.324, 1400.0).unwrap(),
            teistro::UtcOffset::try_from_seconds(20_700).unwrap(),
            &teistro::MuhurtaRequest::from_json(text).unwrap(),
        )
        .unwrap();
    assert_eq!(answer, expected.value);
    assert_eq!(provenance.input_hash, expected.provenance.input_hash);
    let star = envelope["value"]["windows"][0]["clauses"]
        .as_array()
        .unwrap()
        .iter()
        .find(|clause| clause["clause"] == "NAKSHATRA")
        .expect("every window names its day's star");
    assert!(
        star["nakshatra"]
            .as_str()
            .unwrap()
            .starts_with("nakshatra."),
        "{star}"
    );

    // Which kinds the search reached, against every kind there is.
    let reached: std::collections::BTreeSet<String> = answer
        .windows
        .iter()
        .flat_map(|window| &window.clauses)
        .map(|clause| {
            serde_json::to_value(clause.kind.key())
                .unwrap()
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect();
    let unreached: Vec<String> = ClauseKey::ALL
        .iter()
        .map(|key| {
            serde_json::to_value(key)
                .unwrap()
                .as_str()
                .unwrap()
                .to_owned()
        })
        .filter(|key| !reached.contains(key))
        .collect();
    let declared: Vec<&str> = UNREACHED.iter().map(|(key, _)| *key).collect();
    assert_eq!(unreached, declared, "reached: {reached:?}");
}

/// A rite beyond marriage crosses by name, and its unwanted placements
/// come back as the façade's, their grahas written in full.
#[test]
fn a_panchanga_request_answers_a_thread_ceremony_with_its_unwanted_placements() {
    use teistro::muhurta::Answer;
    use teistro::muhurta::clause::ClauseKind;

    let ctx = Ctx::with_ephemeris(0, TsEphemeris::Builtin, None, None, None).unwrap();
    let text = r#"{"rules":"RAMAN_UPANAYANA","daysWithWindows":11,"most":1000}"#;
    let blob = panchanga_days(&ctx, Some(text)).unwrap();
    let schema = schemas::panchanga();
    let reader = Reader::parse(&blob, &schema).unwrap();
    let envelope: serde_json::Value =
        serde_json::from_str(reader.text("muhurta").unwrap()).unwrap();
    let answer: Answer = serde_json::from_value(envelope["value"].clone()).unwrap();
    let sdk = teistro::Context::builder()
        .ephemeris([teistro::Ephemeris::Builtin])
        .build()
        .unwrap();
    let date = |month, day| teistro::CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    let expected = sdk
        .almanac()
        .muhurta(
            &date(11, 25),
            &date(12, 3),
            &teistro::quantity::Place::try_from_degrees(27.7172, 85.324, 1400.0).unwrap(),
            teistro::UtcOffset::try_from_seconds(20_700).unwrap(),
            &teistro::MuhurtaRequest::from_json(text).unwrap(),
        )
        .unwrap();
    assert_eq!(answer, expected.value);
    let placed: Vec<&serde_json::Value> = envelope["value"]["windows"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|window| window["clauses"].as_array().unwrap())
        .filter(|clause| clause["clause"] == "UNWANTED_PLACEMENT")
        .collect();
    assert!(
        !placed.is_empty(),
        "a week of lagnas puts a graha somewhere unwanted"
    );
    for clause in &placed {
        let house = clause["house"].as_u64().unwrap();
        assert!((1..=12).contains(&house), "{clause}");
        assert!(
            clause["by"]
                .as_array()
                .unwrap()
                .iter()
                .all(|g| g.as_str().unwrap().starts_with("graha.")),
            "{clause}"
        );
    }
    // The 8th the rite says must be empty bars the window as its own
    // clause; the houses it only says should be empty weigh, never bar.
    let eighth = |kind: &ClauseKind| matches!(kind, ClauseKind::UnwantedPlacement { house: 8, .. });
    for window in &answer.windows {
        for bar in &window.barred_by {
            if let teistro::muhurta::Bar::Clause(kind @ ClauseKind::UnwantedPlacement { .. }) = bar
            {
                assert!(eighth(kind), "{kind:?}");
            }
        }
        if let Some(clause) = window.clauses.iter().find(|c| eighth(&c.kind)) {
            assert!(
                window
                    .barred_by
                    .contains(&teistro::muhurta::Bar::Clause(clause.kind.clone())),
                "an occupied 8th bars"
            );
        }
    }
}

/// Festivals beside a panchanga request's days (`festival-rules.md` §7):
/// the days are the ones asked without them, the section is the façade's
/// answer with its dates' calendar written in full and sealed over what
/// it holds, and a muhurta search asked beside it finds the same days.
#[test]
fn a_panchanga_request_answers_festivals_over_its_own_days() {
    use teistro::festival::Observances;

    let ctx = Ctx::with_ephemeris(0, TsEphemeris::Builtin, None, None, None).unwrap();
    let range = ((10, 15), (11, 10));
    let text = r#"{"rules":"DHARMASINDHU"}"#;
    let with = panchanga_between(&ctx, range, None, Some(text)).unwrap();
    let without = panchanga_between(&ctx, range, None, None).unwrap();
    let both = panchanga_between(
        &ctx,
        range,
        Some(r#"{"rules":"RAMAN_MARRIAGE"}"#),
        Some(text),
    )
    .unwrap();
    let schema = schemas::panchanga();
    let (with, without, both) = (
        Reader::parse(&with, &schema).unwrap(),
        Reader::parse(&without, &schema).unwrap(),
        Reader::parse(&both, &schema).unwrap(),
    );
    for asked in [&with, &both] {
        assert_eq!(
            asked.text("content_hashes").unwrap(),
            without.text("content_hashes").unwrap()
        );
    }
    assert_eq!(without.text("festivals").unwrap(), "");
    assert_eq!(
        both.text("festivals").unwrap(),
        with.text("festivals").unwrap()
    );
    assert_ne!(both.text("muhurta").unwrap(), "");

    let envelope: serde_json::Value =
        serde_json::from_str(with.text("festivals").unwrap()).unwrap();
    let provenance: teistro::Provenance =
        serde_json::from_value(envelope["provenance"].clone()).unwrap();
    assert_eq!(
        provenance.content_hash,
        teistro::content_hash(&envelope["value"])
    );
    assert_eq!(
        envelope["value"]["observances"][0]["day"]["calendar"],
        "calendar.GREGORIAN"
    );
    let answer: Observances = serde_json::from_value(envelope["value"].clone()).unwrap();
    let sdk = teistro::Context::builder()
        .ephemeris([teistro::Ephemeris::Builtin])
        .build()
        .unwrap();
    let date = |month, day| teistro::CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    let expected = sdk
        .almanac()
        .festivals(
            &date(10, 15),
            &date(11, 10),
            &teistro::quantity::Place::try_from_degrees(27.7172, 85.324, 1400.0).unwrap(),
            teistro::UtcOffset::try_from_seconds(20_700).unwrap(),
            &teistro::FestivalRequest::from_json(text).unwrap(),
        )
        .unwrap();
    assert_eq!(answer, expected.value);
    assert_eq!(provenance.input_hash, expected.provenance.input_hash);
    let rules: Vec<&str> = answer.observances.iter().map(|o| o.rule.as_str()).collect();
    assert_eq!(rules, ["VIJAYA_DASHAMI", "LAKSHMI_PUJA", "BALI_PRATIPADA"]);
}

/// The lunar years beside a panchanga request's days
/// (`calendar-indian-lunisolar.md` §10): asked by a bit, the days the
/// ones asked without it, the section the façade's years with their
/// members in full and sealed over what it holds; an unknown bit asks
/// for nothing.
#[test]
fn a_panchanga_request_answers_the_years_its_days_fall_in() {
    let ctx = Ctx::with_ephemeris(0, TsEphemeris::Builtin, None, None, None).unwrap();
    // Across Chaitra Shukla Pratipada of VS 2083, 19 March 2026.
    let range = ((3, 10), (4, 10));
    let asked = |sections| panchanga_asked(&ctx, range, (None, None), sections).unwrap();
    let (with, without, unknown) = (asked(TS_PANCHANGA_YEARS), asked(0), asked(1 << 31));
    let schema = schemas::panchanga();
    let (with, without, unknown) = (
        Reader::parse(&with, &schema).unwrap(),
        Reader::parse(&without, &schema).unwrap(),
        Reader::parse(&unknown, &schema).unwrap(),
    );
    assert_eq!(
        with.text("content_hashes").unwrap(),
        without.text("content_hashes").unwrap()
    );
    assert_eq!(without.text("years").unwrap(), "");
    assert_eq!(unknown.text("years").unwrap(), "");

    let envelope: serde_json::Value = serde_json::from_str(with.text("years").unwrap()).unwrap();
    let provenance: teistro::Provenance =
        serde_json::from_value(envelope["provenance"].clone()).unwrap();
    assert_eq!(
        provenance.content_hash,
        teistro::content_hash(&envelope["value"])
    );
    let named: Vec<&str> = envelope["value"]
        .as_array()
        .unwrap()
        .iter()
        .map(|year| year["samvatsara"].as_str().unwrap())
        .collect();
    assert_eq!(named, ["samvatsara.SIDDHARTHI", "samvatsara.RAUDRA"]);

    let years: Vec<teistro::LunarYear> = serde_json::from_value(envelope["value"].clone()).unwrap();
    let sdk = teistro::Context::builder()
        .ephemeris([teistro::Ephemeris::Builtin])
        .build()
        .unwrap();
    let date = |month, day| teistro::CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    let expected = sdk
        .almanac()
        .years(
            &date(3, 10),
            &date(4, 10),
            &teistro::quantity::Place::try_from_degrees(27.7172, 85.324, 1400.0).unwrap(),
            teistro::UtcOffset::try_from_seconds(20_700).unwrap(),
        )
        .unwrap();
    assert_eq!(years, expected.value);
    assert_eq!(provenance.input_hash, expected.provenance.input_hash);
    let vikrama: Vec<i32> = years.iter().map(|year| year.vikrama).collect();
    assert_eq!(vikrama, [2082, 2083]);
}

/// The eclipses beside a panchanga request's days (`eclipses.md` §5):
/// asked by a bit beside the years', the days the ones asked without it,
/// the section the façade's envelope in camelCase, its kinds written in
/// full, and sealed over what it holds.
#[test]
fn a_panchanga_request_answers_the_eclipses_its_days_hold() {
    let ctx = Ctx::with_ephemeris(0, TsEphemeris::Builtin, None, None, None).unwrap();
    // The annular solar eclipse of 17 February 2026 and the total lunar
    // eclipse of 3 March.
    let range = ((2, 15), (3, 5));
    let asked = |sections| panchanga_asked(&ctx, range, (None, None), sections).unwrap();
    let schema = schemas::panchanga();
    let (with, without, both) = (
        asked(TS_PANCHANGA_ECLIPSES),
        asked(0),
        asked(TS_PANCHANGA_ECLIPSES | TS_PANCHANGA_YEARS),
    );
    let (with, without, both) = (
        Reader::parse(&with, &schema).unwrap(),
        Reader::parse(&without, &schema).unwrap(),
        Reader::parse(&both, &schema).unwrap(),
    );
    assert_eq!(
        with.text("content_hashes").unwrap(),
        without.text("content_hashes").unwrap()
    );
    assert_eq!(without.text("eclipses").unwrap(), "");
    assert_eq!(with.text("years").unwrap(), "");
    assert_eq!(
        both.text("eclipses").unwrap(),
        with.text("eclipses").unwrap()
    );
    assert_ne!(both.text("years").unwrap(), "");

    let envelope: serde_json::Value = serde_json::from_str(with.text("eclipses").unwrap()).unwrap();
    let provenance: teistro::Provenance =
        serde_json::from_value(envelope["provenance"].clone()).unwrap();
    assert_eq!(
        provenance.content_hash,
        teistro::content_hash(&envelope["value"])
    );
    assert_eq!(
        envelope["value"]["lunar"][0]["eclipse"]["kind"],
        "lunar_eclipse_kind.TOTAL"
    );
    assert_eq!(
        envelope["value"]["solar"][0]["eclipse"]["kind"],
        "solar_eclipse_kind.ANNULAR"
    );
    assert!(envelope["value"]["lunar"][0]["eclipse"]["umbralMagnitude"].is_number());
    assert!(envelope["value"]["lunar"][0]["here"]["p1"]["altitudeDeg"].is_number());

    let sdk = teistro::Context::builder()
        .ephemeris([teistro::Ephemeris::Builtin])
        .build()
        .unwrap();
    let date = |month, day| teistro::CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    let expected = sdk
        .almanac()
        .eclipses(
            &date(2, 15),
            &date(3, 5),
            &teistro::quantity::Place::try_from_degrees(27.7172, 85.324, 1400.0).unwrap(),
            teistro::UtcOffset::try_from_seconds(20_700).unwrap(),
        )
        .unwrap();
    assert_eq!(envelope["value"], expected.value.in_full().unwrap());
    assert_eq!(provenance.input_hash, expected.provenance.input_hash);
}

/// Each day's Nepal Sambat date beside a panchanga request's days
/// (`calendar-indian-lunisolar.md` §11): asked by its own bit, one per
/// day in the days' order, its paksha written in full, sealed over what
/// it holds, and the year turning at Kachhala's first day.
#[test]
fn a_panchanga_request_answers_each_days_nepal_sambat_date() {
    let ctx = Ctx::with_ephemeris(0, TsEphemeris::Builtin, None, None, None).unwrap();
    // Kartika's new moon of 2025 and the 1st after it, which opened 1146.
    let range = ((10, 20), (10, 23));
    let asked = |sections| panchanga_asked_in(&ctx, 2025, range, (None, None), sections).unwrap();
    let schema = schemas::panchanga();
    let (with, without) = (asked(TS_PANCHANGA_NEPAL_SAMBAT), asked(0));
    let (with, without) = (
        Reader::parse(&with, &schema).unwrap(),
        Reader::parse(&without, &schema).unwrap(),
    );
    assert_eq!(
        with.text("content_hashes").unwrap(),
        without.text("content_hashes").unwrap()
    );
    assert_eq!(without.text("nepal_sambat").unwrap(), "");
    assert_eq!(with.text("years").unwrap(), "");

    let envelope: serde_json::Value =
        serde_json::from_str(with.text("nepal_sambat").unwrap()).unwrap();
    let provenance: teistro::Provenance =
        serde_json::from_value(envelope["provenance"].clone()).unwrap();
    assert_eq!(
        provenance.content_hash,
        teistro::content_hash(&envelope["value"])
    );
    let read: Vec<(i64, i64, &str)> = envelope["value"]
        .as_array()
        .unwrap()
        .iter()
        .map(|date| {
            (
                date["year"].as_i64().unwrap(),
                date["month"].as_i64().unwrap(),
                date["paksha"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        read,
        [
            (1145, 12, "paksha.KRISHNA"),
            (1145, 12, "paksha.KRISHNA"),
            (1146, 1, "paksha.SHUKLA"),
            (1146, 1, "paksha.SHUKLA"),
        ]
    );

    let sdk = teistro::Context::builder()
        .ephemeris([teistro::Ephemeris::Builtin])
        .build()
        .unwrap();
    let date = |day| teistro::CalendarDate::defined(Calendar::Gregorian, 2025, 10, day);
    let (days, _) = sdk
        .almanac()
        .of_each(
            &date(20),
            &date(23),
            &teistro::quantity::Place::try_from_degrees(27.7172, 85.324, 1400.0).unwrap(),
            teistro::UtcOffset::try_from_seconds(20_700).unwrap(),
        )
        .unwrap();
    let expected: Vec<teistro::NepalSambatDate> = days
        .value
        .iter()
        .map(teistro::Panchanga::nepal_sambat)
        .collect();
    assert_eq!(
        envelope["value"],
        teistro::NepalSambatDate::in_full(&expected).unwrap()
    );
}

#[test]
fn a_festivals_record_is_refused_by_the_key_that_is_wrong() {
    let ctx = Ctx::with_ephemeris(0, TsEphemeris::Builtin, None, None, None).unwrap();
    for (text, field) in [
        (r#"{"rules":"DHARMA"}"#, "festivals.rules"),
        (
            r#"{"rules":["DHARMASINDHU","DHARMA"]}"#,
            "festivals.rules[1]",
        ),
        (r#"{"rules":["DHARMASINDHU"],"most":1}"#, "festivals.most"),
        ("[]", "festivals"),
    ] {
        let (status, message, named, ..) =
            panchanga_between(&ctx, ((10, 15), (10, 16)), None, Some(text)).unwrap_err();
        assert_eq!(status, Status::InvalidArg, "{text}: {message}");
        assert_eq!(named.as_deref(), Some(field), "{text}: {message}");
    }
}

#[test]
fn a_muhurta_record_is_refused_by_the_key_that_is_wrong() {
    let ctx = Ctx::with_ephemeris(0, TsEphemeris::Builtin, None, None, None).unwrap();
    for (text, field) in [
        (r#"{"rules":"RAMAN"}"#, "muhurta.rules"),
        (r#"{"rules":"RAMAN_MARRIAGE","most":0}"#, "muhurta.most"),
        (
            r#"{"rules":"RAMAN_MARRIAGE","ranking":"BASELINE"}"#,
            "muhurta.rules.baseline",
        ),
        (
            r#"{"rules":"RAMAN_MARRIAGE","asta":"SURYA"}"#,
            "muhurta.asta",
        ),
        ("[]", "muhurta"),
    ] {
        let (status, message, named, ..) = panchanga_days(&ctx, Some(text)).unwrap_err();
        assert_eq!(status, Status::InvalidArg, "{text}: {message}");
        assert_eq!(named.as_deref(), Some(field), "{text}: {message}");
    }
}

/// Progressions cross: a request's `progressions_json` answers every
/// chart's progressed chart and direction in `progressions`, the planets in
/// `progressed_grahas` and `directed_grahas`, and the contacts in
/// `progressed_contacts` ragged by the row's count, each cell the façade's
/// own to the bit; none asked is empty sections, and a refusal is named by
/// the field the caller wrote (`03-design/western-progressions.md`).
#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one assertion per column of four sections"
)]
fn a_chart_request_answers_the_progressions() {
    let ctx = Ctx::with_ephemeris(
        0,
        TsEphemeris::Builtin,
        Some("western-tropical-default"),
        None,
        None,
    )
    .unwrap();
    // Leo's birth and three more at London, read at one instant of life.
    let instants = [
        2_400_629.742_361_111,
        2_405_000.25,
        2_410_321.9,
        2_415_020.5,
    ];
    let base = chart_request(&instants, (51.5, -0.12), 0);
    let asked_text = r#"{"at": 2430000.5, "year": "NOON_SIDEREAL_TIME", "angles": "SOLAR_ARC_LONGITUDE",
        "contacts": {"from": 2425000.5, "to": 2428000.5, "grahas": ["MOON", "SUN"], "points": ["LAGNA", "MARS", "VENUS"]}}"#;
    let asked_json = CString::new(asked_text).unwrap();
    let bytes = chart_blob(
        &ctx,
        // Every section the batch can ask for, so the façade's bare
        // request below holds that a progressed chart is founded on the
        // batch's foundation alone: the sections cost, and change nothing.
        &TsChartRequest {
            progressions_json: asked_json.as_ptr(),
            sections: u32::MAX,
            ..base
        },
    )
    .unwrap_or_else(|status| panic!("{status:?}: {:?}", ctx.last_error()));
    let schema = schemas::charts();
    let reader = Reader::parse(&bytes, &schema).unwrap();

    let sdk = teistro::Context::builder()
        .ephemeris([teistro::Ephemeris::Builtin])
        .profile("western-tropical-default")
        .build()
        .unwrap();
    let place = teistro::quantity::Place::try_from_degrees(51.5, -0.12, 0.0).unwrap();
    let request =
        teistro::ChartRequest::at(place, teistro::UtcOffset::try_from_seconds(0).unwrap());
    let natal = sdk
        .chart()
        .readings(
            &instants
                .iter()
                .map(|&jd| teistro::quantity::JulianDay::<teistro::quantity::Utc>::literal(jd))
                .collect::<Vec<_>>(),
            &request,
        )
        .unwrap()
        .value;
    let asked = teistro::ProgressionsRequest::from_json(asked_text).unwrap();
    let expected: Vec<teistro::Progressions> = natal
        .iter()
        .map(|document| {
            sdk.chart()
                .progressions(document, &asked, &request)
                .unwrap()
        })
        .collect();
    let ints = |section: &str, name: &str| -> Vec<i64> {
        reader
            .column(section, name)
            .unwrap()
            .into_iter()
            .map(ScalarValue::as_i64)
            .collect()
    };
    let bits = |section: &str, name: &str| -> Vec<u64> {
        reader
            .column(section, name)
            .unwrap()
            .into_iter()
            .map(|cell| cell.as_f64().to_bits())
            .collect()
    };
    let progressed = |read: &dyn Fn(&teistro::Progressed) -> f64| -> Vec<u64> {
        expected
            .iter()
            .map(|one| read(one.progressed.as_ref().unwrap()).to_bits())
            .collect()
    };
    let directed = |read: &dyn Fn(&teistro::Directed) -> f64| -> Vec<u64> {
        expected
            .iter()
            .map(|one| read(one.directed.as_ref().unwrap()).to_bits())
            .collect()
    };

    // A row a chart.
    assert_eq!(bits("progressions", "life"), progressed(&|p| p.life.get()));
    assert_eq!(bits("progressions", "sky"), progressed(&|p| p.sky.get()));
    assert_eq!(
        bits("progressions", "armc_deg"),
        progressed(&|p| p.armc_deg)
    );
    assert_eq!(
        bits("progressions", "ascendant_deg"),
        progressed(&|p| p.angles.ascendant_deg)
    );
    assert_eq!(
        bits("progressions", "midheaven_deg"),
        progressed(&|p| p.angles.midheaven_deg)
    );
    assert_eq!(bits("progressions", "arc_deg"), directed(&|d| d.arc_deg));
    assert_eq!(
        bits("progressions", "directed_ascendant_deg"),
        directed(&|d| d.ascendant_deg)
    );
    assert_eq!(
        bits("progressions", "directed_midheaven_deg"),
        directed(&|d| d.midheaven_deg)
    );
    let contacts: Vec<&teistro::ProgressedContact> = expected
        .iter()
        .flat_map(|one| one.contacts.as_deref().unwrap())
        .collect();
    assert!(!contacts.is_empty(), "the window holds contacts");
    assert_eq!(
        ints("progressions", "contact_count"),
        expected
            .iter()
            .map(|one| i64::try_from(one.contacts.as_ref().unwrap().len()).unwrap())
            .collect::<Vec<_>>()
    );

    assert_eq!(
        ints("progressions", "contacts_asked"),
        vec![1; instants.len()]
    );

    // The planets, graha-count rows a chart.
    let grahas: Vec<&teistro_chart::foundation::GrahaPosition> = expected
        .iter()
        .flat_map(|one| {
            &one.progressed
                .as_ref()
                .unwrap()
                .chart
                .value
                .foundation
                .grahas
        })
        .collect();
    assert_eq!(
        ints("progressed_grahas", "graha"),
        grahas
            .iter()
            .map(|at| i64::from(at.graha.id()))
            .collect::<Vec<_>>()
    );
    for (column, read) in [
        (
            "longitude_deg",
            (|at: &teistro_chart::foundation::GrahaPosition| at.longitude_deg)
                as fn(&teistro_chart::foundation::GrahaPosition) -> f64,
        ),
        ("tropical_deg", |at| at.tropical_deg),
        ("speed_deg_per_day", |at| at.speed_deg_per_day),
    ] {
        assert_eq!(
            bits("progressed_grahas", column),
            grahas
                .iter()
                .map(|at| read(at).to_bits())
                .collect::<Vec<_>>(),
            "{column}"
        );
    }
    let moved: Vec<&teistro::DirectedPlanet> = expected
        .iter()
        .flat_map(|one| &one.directed.as_ref().unwrap().planets)
        .collect();
    assert_eq!(
        ints("directed_grahas", "graha"),
        moved
            .iter()
            .map(|at| i64::from(at.graha.id()))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        bits("directed_grahas", "longitude_deg"),
        moved
            .iter()
            .map(|at| at.longitude_deg.to_bits())
            .collect::<Vec<_>>()
    );

    // The contacts, ragged.
    assert_eq!(
        bits("progressed_contacts", "life"),
        contacts
            .iter()
            .map(|c| c.life.get().to_bits())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        bits("progressed_contacts", "sky"),
        contacts
            .iter()
            .map(|c| c.sky.get().to_bits())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        ints("progressed_contacts", "graha"),
        contacts
            .iter()
            .map(|c| i64::from(c.graha.id()))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        ints("progressed_contacts", "to_lagna"),
        contacts
            .iter()
            .map(|c| i64::from(c.to == teistro::NatalPoint::Lagna))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        ints("progressed_contacts", "to_graha"),
        contacts
            .iter()
            .map(|c| match c.to {
                teistro::NatalPoint::Graha { graha } => i64::from(graha.id()),
                teistro::NatalPoint::Lagna => 0,
            })
            .collect::<Vec<_>>()
    );
    assert_eq!(
        ints("progressed_contacts", "angle"),
        contacts
            .iter()
            .map(|c| i64::from(c.angle))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        ints("progressed_contacts", "motion"),
        contacts
            .iter()
            .map(|c| i64::from(teistro_ffi::chart::TsMotion::from(c.motion) as u8))
            .collect::<Vec<_>>()
    );

    // None asked is empty sections.
    let bytes = chart_blob(&ctx, &base).unwrap();
    let reader = Reader::parse(&bytes, &schema).unwrap();
    for (section, column) in [
        ("progressions", "life"),
        ("progressed_grahas", "graha"),
        ("directed_grahas", "graha"),
        ("progressed_contacts", "life"),
    ] {
        assert_eq!(
            reader.column(section, column).unwrap().len(),
            0,
            "{section}"
        );
    }

    // A refusal is named by the field the caller wrote.
    let refused = CString::new(r#"{"at": 2430000.5, "direction": {"PER_YEAR": 0}}"#).unwrap();
    let status = chart_blob(
        &ctx,
        &TsChartRequest {
            progressions_json: refused.as_ptr(),
            ..base
        },
    )
    .unwrap_err();
    assert_eq!(status, Status::InvalidArg);
    assert_eq!(
        ctx.last_error().2.as_deref(),
        Some("progressions.direction")
    );
}
