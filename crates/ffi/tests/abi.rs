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
    reason = "tests cross the boundary, fail by panicking, read sizes as C does and walk one scenario each"
)]

use core::ffi::CStr;
use core::ptr;
use std::ffi::CString;

use teistro_core::Status;
use teistro_core::catalogue::{Calendar, Era, Graha};
use teistro_ffi::blob::{TsBlob, ts_blob_free};
use teistro_ffi::calendar::{
    TsCalendarDate, TsResolution, ts_calendar_convert, ts_calendar_fixed_of_jd,
    ts_calendar_from_fixed, ts_calendar_is_leap, ts_calendar_jd_of_fixed, ts_calendar_month_length,
    ts_calendar_to_fixed, ts_calendar_weekday,
};
use teistro_ffi::context::{
    TsContext, TsContextOptions, TsError, ts_context_free, ts_context_last_error, ts_context_new,
    ts_context_profile, ts_context_settings_hash, ts_context_settings_json,
};
use teistro_ffi::intl::{ts_intl_has, ts_intl_locale, ts_intl_render, ts_intl_set_locale};
use teistro_ffi::keys::{ts_key_name, ts_key_parse};
use teistro_ffi::positions::ts_positions;
use teistro_ffi::schemas;
use teistro_ffi::strings::{TsHash, TsStr, TsString, ts_string_free};
use teistro_ffi::time::{
    TsCivilDateTime, TsCivilTime, TsDeltaT, TsDeltaTSource, TsScale, TsTimeConversion, TsZoneEra,
    TsZoneResolution, TsZoneSource, TsZoneSpec, ts_time_civil, ts_time_convert, ts_time_delta_t,
    ts_time_resolve,
};
use teistro_ffi::{TS_CONTEXT_TEST_PROVIDER, ts_abi_version};
use teistro_idl::blob::Reader;
use teistro_port_ephemeris::{Body, Coordinates, Frame, PositionRequestC, TimeScale};

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
    ) -> Result<Ctx, (Status, String)> {
        let profile = profile.map(|p| CString::new(p).unwrap());
        let settings = settings_json.map(|p| CString::new(p).unwrap());
        let locale = locale.map(|p| CString::new(p).unwrap());
        let options = TsContextOptions {
            struct_size: size_of::<TsContextOptions>() as u32,
            flags,
            profile: profile.as_ref().map_or(ptr::null(), |p| p.as_ptr()),
            settings_json: settings.as_ref().map_or(ptr::null(), |p| p.as_ptr()),
            locale: locale.as_ref().map_or(ptr::null(), |p| p.as_ptr()),
        };
        let mut handle = ptr::null_mut();
        let mut error = TsString::empty();
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
        // SAFETY: the library wrote a NUL-terminated string.
        let message = unsafe { CStr::from_ptr(error.data.cast()) }
            .to_string_lossy()
            .into_owned();
        // SAFETY: a descriptor the library wrote.
        unsafe { ts_string_free(&raw mut error) };
        Err((status, message))
    }

    fn defaults() -> Ctx {
        Ctx::new(0, None, None, None).expect("a context with every default")
    }

    fn last_error(
        &self,
    ) -> (
        Status,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
    ) {
        let mut error = TsError {
            struct_size: size_of::<TsError>() as u32,
            status: 99,
            provider_code: 0,
            reserved: 0,
            detail: ptr::null(),
            message: ptr::null(),
            field: ptr::null(),
            hint: ptr::null(),
            key: ptr::null(),
        };
        // SAFETY: a live handle and a valid struct with its size set.
        assert_eq!(
            unsafe { ts_context_last_error(self.handle, &raw mut error) },
            Status::Ok
        );
        let text = |p: *const core::ffi::c_char| {
            if p.is_null() {
                None
            } else {
                // SAFETY: the library lends NUL-terminated strings.
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
fn a_context_refuses_what_it_cannot_build_and_says_why() {
    let (status, message) = Ctx::new(0, Some("vedic-classic"), None, None).unwrap_err();
    assert_eq!(status, Status::Unsupported);
    assert!(
        message.contains("no shipped profile `vedic-classic`")
            && message.contains("parashari-classical"),
        "{message}"
    );
    let (status, message) =
        Ctx::new(0, None, Some(r#"{"frame": {"zodiacs": "TROPICAL"}}"#), None).unwrap_err();
    assert_eq!(status, Status::InvalidArg);
    assert!(
        message.contains("unknown field `zodiacs`") && message.contains("settings_json"),
        "{message}"
    );
    let (status, message) = Ctx::new(0, None, None, Some("xx-Latn")).unwrap_err();
    assert_eq!(status, Status::Unsupported);
    assert!(message.contains("ne-Deva-NP"), "{message}");
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
    let (status, message, field, _, _) = bare.last_error();
    assert_eq!(
        (status, field.as_deref()),
        (Status::Capability, Some("provider"))
    );
    assert!(message.contains("TS_CONTEXT_TEST_PROVIDER"));
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
