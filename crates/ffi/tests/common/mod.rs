//! What the boundary's test files share: a context freed on drop, an error
//! record copied out, and a chart request asking for nothing beside the
//! charts.
#![allow(
    dead_code,
    unsafe_code,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_possible_truncation,
    reason = "each test file uses part of it; it crosses the boundary and reads sizes as C does"
)]

use core::ffi::CStr;
use core::ptr;
use std::ffi::CString;

use teistro_core::Status;
#[cfg(feature = "chart")]
use teistro_ffi::blob::{TsBlob, ts_blob_free};
#[cfg(feature = "chart")]
use teistro_ffi::chart::{TsChartRequest, ts_chart_found};
use teistro_ffi::context::{
    TsContext, TsContextOptions, TsEphemeris, TsError, ts_context_free, ts_context_last_error,
    ts_context_new, ts_error_free,
};
use teistro_ffi::string::{TsStr, TsString, ts_string_free};
use teistro_ffi::{TS_CONTEXT_TEST_PROVIDER, TS_ERROR_OWNED};

/// An error record's status and strings, copied out: the status, the
/// message, the field, the hint and the detail.
pub(crate) type Record = (
    Status,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
);

/// An empty record with this build's size, ready for a call to write.
pub(crate) fn blank_error() -> TsError {
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
pub(crate) fn read_record(error: &TsError) -> Record {
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
pub(crate) struct Ctx {
    pub(crate) handle: *mut TsContext,
}

impl Ctx {
    pub(crate) fn new(
        flags: u32,
        profile: Option<&str>,
        settings_json: Option<&str>,
        locale: Option<&str>,
    ) -> Result<Ctx, Record> {
        Ctx::with_ephemeris(flags, TsEphemeris::None, profile, settings_json, locale)
    }

    /// The same, naming one of the SDK's own ephemerides (ADR-0028).
    pub(crate) fn with_ephemeris(
        flags: u32,
        ephemeris: TsEphemeris,
        profile: Option<&str>,
        settings_json: Option<&str>,
        locale: Option<&str>,
    ) -> Result<Ctx, Record> {
        Ctx::open(flags, ephemeris, profile, settings_json, locale, None, None)
    }

    /// A context on the test provider with a consumer's own layouts.
    pub(crate) fn with_layouts(layouts_json: &str) -> Result<Ctx, Record> {
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
    pub(crate) fn with_dashas(dashas_json: &str) -> Result<Ctx, Record> {
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

    pub(crate) fn open(
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

    pub(crate) fn defaults() -> Ctx {
        Ctx::new(0, None, None, None).expect("a context with every default")
    }

    pub(crate) fn last_error(&self) -> Record {
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

pub(crate) fn lent(s: TsStr) -> String {
    // SAFETY: the library lends NUL-terminated strings.
    unsafe { CStr::from_ptr(s.data) }
        .to_str()
        .unwrap()
        .to_string()
}

pub(crate) fn owned(mut s: TsString) -> String {
    // SAFETY: a NUL-terminated string the library allocated.
    let text = unsafe { CStr::from_ptr(s.data.cast()) }
        .to_str()
        .unwrap()
        .to_string();
    // SAFETY: a descriptor the library wrote.
    unsafe { ts_string_free(&raw mut s) };
    text
}

pub(crate) fn sized<T>(mut value: T, set: impl FnOnce(&mut T, u32)) -> T {
    set(&mut value, size_of::<T>() as u32);
    value
}

#[cfg(feature = "chart")]
/// A chart request for `instants` at a place and clock, asking for
/// nothing beside the charts: a test sets the one record it reads with
/// `..chart_request(...)`.
pub(crate) fn chart_request(
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
            western_aspects_json: ptr::null(),
            synastry_json: ptr::null(),
            parallels_json: ptr::null(),
            antiscia_json: ptr::null(),
            midpoints_json: ptr::null(),
            western_houses_json: ptr::null(),
            harmonic_json: ptr::null(),
            matching_json: ptr::null(),
            prashna_json: ptr::null(),
            remedies_json: ptr::null(),
            rectification_json: ptr::null(),
            lalkitab_json: ptr::null(),
        },
        |r, s| r.struct_size = s,
    )
}

/// The blob a chart request answers, or the status it was refused with.
#[cfg(feature = "chart")]
pub(crate) fn chart_blob(ctx: &Ctx, asked: &TsChartRequest) -> Result<Vec<u8>, Status> {
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
