//! The context: settings resolved from a profile and a patch, the
//! ephemeris provider, the locale engine, the Delta T model, and the
//! last error with the strings the context lends. One context serves one
//! thread at a time; it may be moved between threads.

#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

use core::cell::{Ref, RefCell, RefMut};
use core::ffi::{c_char, c_void};
use core::ptr;
use std::ffi::CString;
use std::panic::{AssertUnwindSafe, catch_unwind};

use teistro_astro::DeltaTModel;
use teistro_core::Status;
use teistro_core::error::Error;
use teistro_core::settings::{
    DEFAULT_PROFILE, Profile, Resolved, SHIPPED_PROFILES, Settings, SettingsPatch,
};
use teistro_intl::Intl;
use teistro_intl::pack::locales_from_packs;
use teistro_port_ephemeris::{EphemerisProvider, ProviderVtable, TestProvider, VtableProvider};

use crate::TS_CONTEXT_TEST_PROVIDER;
use crate::strings::{TsHash, TsStr, TsString};
use crate::support::{c_struct, optional_text, read_in, with_context, write_out, write_plain};

include!(concat!(env!("OUT_DIR"), "/bundles.rs"));

/// How a context is built. Every field may be left at its zero value: a
/// null `profile` selects `ts_default_profile`, a null `settings_json`
/// patches nothing, a null `locale` renders in the base locale.
#[repr(C)]
#[derive(Debug)]
pub struct TsContextOptions {
    /// `sizeof(ts_context_options)` as the caller compiled it.
    pub struct_size: u32,
    /// `TS_CONTEXT_*` flags, or zero.
    /// `api: example=0`
    pub flags: u32,
    /// The shipped profile's id (`parashari-classical`, `nepali-default`,
    /// `kp-default`, `western-tropical-default`, `conformance-baseline`).
    /// `api: nullable example=parashari-classical`
    pub profile: *const c_char,
    /// A JSON settings patch over the profile: an object whose groups and
    /// knobs are the settings document's, every one optional; an unknown
    /// knob or a value outside its type is `INVALID_ARG` naming it.
    /// `api: nullable`
    pub settings_json: *const c_char,
    /// The locale every render resolves from (`ne-Deva-NP`).
    /// `api: nullable example=en-Latn`
    pub locale: *const c_char,
}

/// The last error of a call on a context: the status, the detail, and the
/// message, field, hint and message key as strings the context lends
/// until its next call; an `OK` record has empty strings.
#[repr(C)]
#[derive(Debug)]
pub struct TsError {
    /// `sizeof(ts_error)` as the caller compiled it.
    pub struct_size: u32,
    /// The status code.
    /// `api: enum=Status`
    pub status: i32,
    /// The provider's own code when the status is `PROVIDER`, else zero.
    pub provider_code: i32,
    /// Reserved, zero.
    pub reserved: u32,
    /// The detail's name (`UNKNOWN_KEY`), or null.
    /// `api: nullable`
    pub detail: *const c_char,
    /// The English message naming the field and the range.
    pub message: *const c_char,
    /// The field involved, or null.
    /// `api: nullable`
    pub field: *const c_char,
    /// A hint (`did you mean ...`), or null.
    /// `api: nullable`
    pub hint: *const c_char,
    /// The localisable message key, or null.
    /// `api: nullable`
    pub key: *const c_char,
}

c_struct!(TsContextOptions, TsError);

/// An opaque context: settings, a provider, the locale engine, the last
/// error. Used by one thread at a time.
pub struct TsContext {
    settings: Resolved,
    provider: Option<Box<dyn EphemerisProvider>>,
    intl: RefCell<Intl>,
    delta_t: DeltaTModel,
    scratch: RefCell<Scratch>,
}

impl core::fmt::Debug for TsContext {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TsContext")
            .field("profile", &self.settings.profile)
            .field("provider", &self.provider.is_some())
            .finish_non_exhaustive()
    }
}

/// What a context keeps between calls: the last error and the strings it
/// lent during the last call.
#[derive(Default)]
struct Scratch {
    error: Option<StoredError>,
    provider_code: i32,
    lent: Vec<CString>,
}

/// An error as `ts_context_last_error` reads it.
struct StoredError {
    status: Status,
    provider_code: i32,
    detail: Option<CString>,
    message: CString,
    field: Option<CString>,
    hint: Option<CString>,
    key: Option<CString>,
}

fn c_string(text: &str) -> CString {
    CString::new(text.replace('\0', " ")).unwrap_or_default()
}

impl TsContext {
    /// Builds a context from options, a provider and a locale.
    ///
    /// # Errors
    ///
    /// An unknown profile, a patch that does not parse or contradicts the
    /// profile, a vtable that does not bind, an unknown locale.
    pub fn build(
        profile: Option<&str>,
        settings_json: Option<&str>,
        provider: Option<Box<dyn EphemerisProvider>>,
        locale: Option<&str>,
    ) -> Result<TsContext, Error> {
        let id = profile.unwrap_or(DEFAULT_PROFILE);
        let profile = Profile::shipped(id).ok_or_else(|| {
            Error::unsupported(format!("no shipped profile `{id}`"))
                .with_field("profile")
                .with_hint(format!(
                    "the shipped profiles are {}",
                    SHIPPED_PROFILES.join(", ")
                ))
        })?;
        let patch: SettingsPatch = match settings_json {
            Some(json) => serde_json::from_str(json).map_err(|e| {
                Error::invalid_arg(format!("the settings patch does not parse: {e}"))
                    .with_field("settings_json")
            })?,
            None => SettingsPatch::default(),
        };
        let settings = profile.resolve(&patch)?;
        let bundles: Vec<&[u8]> = BUNDLES.iter().map(|(_, bytes)| *bytes).collect();
        let locales = locales_from_packs(&bundles).map_err(|e| {
            Error::new(
                Status::Pack,
                format!("the embedded bundles do not load: {e}"),
            )
        })?;
        let mut intl = Intl::new(locales)
            .map_err(|e| Error::internal(format!("the locale engine did not start: {e}")))?;
        if let Some(tag) = locale {
            intl.set_locale(tag)
                .map_err(|e| unknown_locale(&intl, tag, &e.to_string()))?;
        }
        let delta_t = DeltaTModel::from_knob(settings.settings.time.delta_t).unwrap_or_default();
        Ok(TsContext {
            settings,
            provider,
            intl: RefCell::new(intl),
            delta_t,
            scratch: RefCell::new(Scratch::default()),
        })
    }

    /// The resolved settings.
    #[must_use]
    pub fn settings(&self) -> &Settings {
        &self.settings.settings
    }

    /// The profile the settings came from.
    #[must_use]
    pub fn profile(&self) -> &str {
        self.settings.profile.as_str()
    }

    /// The provider, when the context has one.
    #[must_use]
    pub fn provider(&self) -> Option<&dyn EphemerisProvider> {
        self.provider.as_deref()
    }

    /// The Delta T model the settings chose.
    #[must_use]
    pub const fn delta_t(&self) -> DeltaTModel {
        self.delta_t
    }

    /// The locale engine.
    #[must_use]
    pub fn intl(&self) -> Ref<'_, Intl> {
        self.intl.borrow()
    }

    /// The locale engine, to change it.
    #[must_use]
    pub fn intl_mut(&self) -> RefMut<'_, Intl> {
        self.intl.borrow_mut()
    }

    /// Starts a call: the strings lent by the previous call are released
    /// and the last error cleared.
    pub(crate) fn begin_call(&self) {
        let mut scratch = self.scratch.borrow_mut();
        scratch.lent.clear();
        scratch.error = None;
        scratch.provider_code = 0;
    }

    /// Records the provider's own code for the error being reported.
    pub(crate) fn set_provider_code(&self, code: i32) {
        self.scratch.borrow_mut().provider_code = code;
    }

    /// Records a call's outcome.
    pub(crate) fn record(&self, error: Option<Error>) {
        let mut scratch = self.scratch.borrow_mut();
        let provider_code = scratch.provider_code;
        scratch.error = error.map(|e| StoredError {
            status: e.status,
            provider_code,
            detail: e
                .detail
                .and_then(|d| serde_json::to_value(d).ok())
                .and_then(|v| v.as_str().map(c_string)),
            message: c_string(&e.message),
            field: e.field().map(c_string),
            hint: e.hint().map(c_string),
            key: e.key().map(|k| c_string(&k.key)),
        });
    }

    /// Lends a string to the caller until the next call on this context.
    pub(crate) fn lend(&self, text: &str) -> TsStr {
        let owned = c_string(text);
        let view = TsStr {
            data: owned.as_ptr(),
            len: owned.as_bytes().len(),
        };
        self.scratch.borrow_mut().lent.push(owned);
        view
    }

    /// Lends a string's pointer, or null for none.
    pub(crate) fn lend_ptr(&self, text: Option<&str>) -> *const c_char {
        text.map_or(ptr::null(), |t| self.lend(t).data)
    }
}

/// `UNSUPPORTED` for a locale the engine does not hold, naming the ones
/// it does.
pub(crate) fn unknown_locale(intl: &Intl, tag: &str, detail: &str) -> Error {
    let known: Vec<&str> = intl.locales().map(|l| l.tag.as_str()).collect();
    Error::unsupported(detail.to_string())
        .with_field("locale")
        .with_hint(format!(
            "`{tag}` is not loaded; the locales are {}",
            known.join(", ")
        ))
}

/// Creates a context. `options` may be null for every default; `provider`
/// may be null, in which case the `TS_CONTEXT_TEST_PROVIDER` flag selects
/// the analytic test provider and no flag leaves the context without an
/// ephemeris (positions are then `CAPABILITY`); `provider_user_data` is
/// passed back to the vtable's functions untouched and must stay valid
/// until `ts_context_free`. On success `*out_context` owns the context;
/// on failure, when `out_error` is not null, it receives the error's
/// message as a string to free with `ts_string_free`.
///
/// # Safety
///
/// Every pointer must be null or valid for the access its documentation
/// describes, for the duration of the call; a vtable's functions must be
/// callable with `provider_user_data` until the context is freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_context_new(
    options: *const TsContextOptions,
    provider: *const ProviderVtable,
    provider_user_data: *mut c_void,
    out_context: *mut *mut TsContext,
    out_error: *mut TsString,
) -> Status {
    if out_context.is_null() {
        return Status::InvalidArg;
    }
    let built = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: the entry point's contract.
        unsafe { build(options, provider, provider_user_data) }
    }))
    .unwrap_or_else(|_| {
        Err(Error::internal(
            "a panic was caught while building the context",
        ))
    });
    match built {
        Ok(context) => {
            // SAFETY: non-null; the caller promises a writable slot.
            unsafe { out_context.write(Box::into_raw(Box::new(context))) };
            Status::Ok
        }
        Err(error) => {
            if !out_error.is_null() {
                // SAFETY: non-null; the caller promises a writable descriptor.
                unsafe { out_error.write(TsString::from_string(error.to_string())) };
            }
            error.status
        }
    }
}

/// # Safety
///
/// As [`ts_context_new`].
unsafe fn build(
    options: *const TsContextOptions,
    provider: *const ProviderVtable,
    provider_user_data: *mut c_void,
) -> Result<TsContext, Error> {
    // SAFETY: the entry point's contract; a null pointer means defaults.
    let options = if options.is_null() {
        None
    } else {
        Some(unsafe { read_in(options, "options") }?)
    };
    let flags = options.map_or(0, |o| o.flags);
    // SAFETY: the entry point's contract.
    let (profile, settings_json, locale) = unsafe {
        (
            optional_text(
                options.map_or(ptr::null(), |o| o.profile),
                "options.profile",
            )?,
            optional_text(
                options.map_or(ptr::null(), |o| o.settings_json),
                "options.settings_json",
            )?,
            optional_text(options.map_or(ptr::null(), |o| o.locale), "options.locale")?,
        )
    };
    let provider: Option<Box<dyn EphemerisProvider>> = if provider.is_null() {
        (flags & TS_CONTEXT_TEST_PROVIDER != 0)
            .then(|| Box::new(TestProvider::new()) as Box<dyn EphemerisProvider>)
    } else {
        // SAFETY: non-null; the caller promises a readable vtable whose
        // functions stay valid with `provider_user_data`.
        let bound = unsafe { VtableProvider::bind(ptr::read(provider), provider_user_data) }?;
        Some(Box::new(bound))
    };
    TsContext::build(profile, settings_json, provider, locale)
}

/// Frees a context; null is ignored.
///
/// # Safety
///
/// `context` must be null or a handle from `ts_context_new` that is not
/// used again.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_context_free(context: *mut TsContext) {
    if context.is_null() {
        return;
    }
    // SAFETY: the pointer came from `Box::into_raw` in `ts_context_new` and
    // the caller promises not to use it again.
    drop(unsafe { Box::from_raw(context) });
}

/// The outcome of the last call on the context: its status, the provider's
/// code, and the message, field, hint and key as strings lent until the
/// next call other than this one. After a successful call the record is
/// `OK` with a null message.
///
/// # Safety
///
/// `context` must be a live handle; `out_error` valid for a read of its
/// `struct_size` and a write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_context_last_error(
    context: *const TsContext,
    out_error: *mut TsError,
) -> Status {
    // SAFETY: the entry point's contract.
    let Some(ctx) = (unsafe { crate::support::context(context) }) else {
        return Status::InvalidArg;
    };
    let scratch = ctx.scratch.borrow();
    let record = scratch.error.as_ref().map_or_else(
        || TsError {
            struct_size: 0,
            status: Status::Ok.code(),
            provider_code: 0,
            reserved: 0,
            detail: ptr::null(),
            message: ptr::null(),
            field: ptr::null(),
            hint: ptr::null(),
            key: ptr::null(),
        },
        |e| TsError {
            struct_size: 0,
            status: e.status.code(),
            provider_code: e.provider_code,
            reserved: 0,
            detail: e.detail.as_ref().map_or(ptr::null(), |s| s.as_ptr()),
            message: e.message.as_ptr(),
            field: e.field.as_ref().map_or(ptr::null(), |s| s.as_ptr()),
            hint: e.hint.as_ref().map_or(ptr::null(), |s| s.as_ptr()),
            key: e.key.as_ref().map_or(ptr::null(), |s| s.as_ptr()),
        },
    );
    // SAFETY: the entry point's contract.
    match unsafe { write_out(out_error, "out_error", record) } {
        Ok(()) => Status::Ok,
        Err(error) => error.status,
    }
}

/// The id of the profile the context's settings came from, lent until the
/// next call on the context.
///
/// # Safety
///
/// `context` must be a live handle; `out_profile` valid for a write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_context_profile(
    context: *const TsContext,
    out_profile: *mut TsStr,
) -> Status {
    with_context(context, |ctx| {
        let lent = ctx.lend(ctx.profile());
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_profile, "out_profile", lent) }
    })
}

/// The resolved settings as their canonical JSON document, allocated for
/// the caller; free it with `ts_string_free`. Two contexts with the same
/// document compute the same numbers.
///
/// # Safety
///
/// `context` must be a live handle; `out_json` valid for a write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_context_settings_json(
    context: *const TsContext,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |ctx| {
        let json = TsString::from_string(ctx.settings().canonical_json());
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_json, "out_json", json) }
    })
}

/// The SHA-256 of the canonical settings document: the `settings_hash`
/// every result's provenance carries.
///
/// # Safety
///
/// `context` must be a live handle; `out_hash` valid for a write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_context_settings_hash(
    context: *const TsContext,
    out_hash: *mut TsHash,
) -> Status {
    with_context(context, |ctx| {
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_hash, "out_hash", ctx.settings().hash().into()) }
    })
}
