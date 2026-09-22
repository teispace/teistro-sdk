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

use teistro_astro::DeltaTModel;
use teistro_core::Status;
use teistro_core::error::Error;
use teistro_core::settings::{Resolved, Settings};
use teistro_intl::Intl;
use teistro_port_ephemeris::{EphemerisProvider, ProviderVtable, VtableProvider};

use crate::string::{TsHash, TsStr, TsString};
use crate::support::{
    c_struct, check_size, construct, optional_text, read_in, with_context, write_out, write_plain,
};
use crate::{TS_CONTEXT_TEST_PROVIDER, TS_ERROR_OWNED};

/// Which of the SDK's own ephemerides a context computes with when no
/// provider vtable is given.
///
/// A caller who passes a vtable has already answered the question, and
/// this is ignored. The states are exclusive, which is why they are an
/// enum and not flag bits (ADR-0028).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TsEphemeris {
    /// None. Positions are `CAPABILITY`, and so is anything built on
    /// them. The zero value, and what a caller who passes a vtable
    /// leaves this at.
    #[default]
    None = 0,
    /// The SDK's own built-in analytic ephemeris: no files, no network,
    /// no licence beyond the SDK's own (ADR-0008). `UNSUPPORTED` naming
    /// the feature if this library was built without it.
    Builtin = 1,
    /// The analytic test provider. For tests and examples only — its
    /// positions are **not astronomy**, and a chart cast from them is a
    /// shape rather than a sky.
    Test = 2,
}

impl TsEphemeris {
    /// The value a caller wrote, or `None` if it names nothing.
    ///
    /// An unknown number is not silently the default: a caller who meant
    /// something this library does not have should hear so.
    const fn from_repr(value: u8) -> Option<TsEphemeris> {
        match value {
            0 => Some(TsEphemeris::None),
            1 => Some(TsEphemeris::Builtin),
            2 => Some(TsEphemeris::Test),
            _ => None,
        }
    }
}

/// How a context is built. Every field may be left at its zero value: a
/// null `profile` selects `ts_default_profile`, a null `settings_json`
/// patches nothing, a null `locale` renders in the base locale, and a
/// zero `ephemeris` leaves the context without one unless a vtable is
/// passed.
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
    /// Chart layouts of the consumer's own, to draw in beside the shipped
    /// ones, as a JSON array of layout rows: each the row `ts_chart_layout_row`
    /// answers, with a key of its own. Every row is checked by the rules a
    /// shipped one passes and refused by its place in the array and its own
    /// field, as `options.layouts_json`, the row's index, then the field's
    /// path; a key the SDK ships is
    /// refused, so a row adds a layout and never replaces one. Null for none
    /// (`03-design/chart-geometry.md` §7f).
    /// `api: nullable`
    pub layouts_json: *const c_char,
    /// Dasha systems of the consumer's own, as a JSON array of definitions,
    /// each naming the `kernel` that runs it. `"kernel": "udu"` is the
    /// nakshatra-seeded kind: a key the catalogue does not have, its lords
    /// and their years in order, the reference nakshatra, and optionally
    /// `count`, `span`, `offset`, `repeats`, `scale`, `year_length`, `depth`
    /// and `sources`. `"kernel": "rashi"` is the sign-based kind: a key, and
    /// optionally `start`, `order`, `length`, `namedLord`, `strongerOf`,
    /// `year_length`, `depth` and `sources` (the document schema's
    /// `DashaDefinition`). The kernel is **stated** and never inferred from
    /// which fields are present, so a typo is refused by the field the
    /// caller wrote rather than by one they did not. Every one is checked
    /// by the rules a shipped row passes and refused by its place in the
    /// array and its own field, as `options.dashas_json`, the index, then
    /// the field.
    /// A request asks for one by the id
    /// `ts_key_parse` gives `dasha_system.<KEY>`, `0x8000` and up in
    /// registration order. Null for none (`03-design/dasha-kernels.md`).
    /// `api: nullable`
    pub dashas_json: *const c_char,
    /// Which of the SDK's own ephemerides to use when no provider vtable
    /// is given; ignored when one is (ADR-0028).
    /// `api: enum=TsEphemeris example=0`
    pub ephemeris: u8,
}

/// A failure as the library describes it: the status, the provider's
/// code, and the detail, message, field, hint and message key.
///
/// Read from `ts_context_last_error`, the strings are **lent** by the
/// context until its next call and `flags` is zero; an `OK` record has
/// null strings. Written by a call that makes a handle and failed, the
/// strings are **owned** by the record, `flags` carries
/// `TS_ERROR_OWNED`, and `ts_error_free` releases them. `ts_error_free`
/// on a lent record does nothing, so freeing every record is never wrong.
///
/// `api: role=error`
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
    /// `TS_ERROR_OWNED` when the record owns its strings, else zero.
    pub flags: u32,
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
// **Everything above this line is the C header's documentation**, which
// `cargo xtask gen ffi` extracts and `check-ffi` holds — so a note to
// this repository's maintainers goes here, in a comment the generator
// does not read, and not in the sentence a C consumer meets.
//
// The note: the composition is the façade's (`teistro::Context`). The
// settings, the provider, the locale engine and the ΔT model are
// resolved there, so a Rust consumer and a C caller get the same context
// built the same way rather than two compositions that have to be kept
// equal by hand. `03-design/rust-consumer-surface.md` §3 decided it, and
// the measured page's last two properties are its acceptance test.
pub struct TsContext {
    inner: teistro::Context,
    scratch: RefCell<Scratch>,
    /// A reference that keeps a loaded adapter's library in memory for
    /// as long as this context might call into it (ADR-0029).
    ///
    /// **Declared last on purpose**: fields drop in declaration order, so
    /// `provider` — whose vtable is a table of function pointers into
    /// that library — is gone before the library can be unloaded.
    loaded: Option<crate::provider::Keepalive>,
}

impl core::fmt::Debug for TsContext {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TsContext")
            .field("profile", &self.inner.profile())
            .field("provider", &self.inner.ephemeris().is_some())
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

/// An error with its strings made C strings once, from which either kind
/// of [`TsError`] is laid out.
struct StoredError {
    status: Status,
    provider_code: i32,
    texts: Texts<CString>,
}

/// The five strings of an error record, in whichever form a record needs
/// them: owned, borrowed, or as the pointers C reads.
#[derive(Clone, Copy)]
struct Texts<S> {
    detail: Option<S>,
    message: S,
    field: Option<S>,
    hint: Option<S>,
    key: Option<S>,
}

impl<S> Texts<S> {
    fn map<U>(self, mut f: impl FnMut(S) -> U) -> Texts<U> {
        Texts {
            detail: self.detail.map(&mut f),
            message: f(self.message),
            field: self.field.map(&mut f),
            hint: self.hint.map(&mut f),
            key: self.key.map(&mut f),
        }
    }
}

fn c_string(text: &str) -> CString {
    CString::new(text.replace('\0', " ")).unwrap_or_default()
}

impl StoredError {
    /// An error's strings, made C strings.
    fn of(error: &Error, provider_code: i32) -> StoredError {
        StoredError {
            status: error.status,
            provider_code,
            texts: Texts {
                detail: error
                    .detail
                    .as_ref()
                    .and_then(|d| serde_json::to_value(d).ok())
                    .and_then(|v| v.as_str().map(c_string)),
                message: c_string(&error.message),
                field: error.field().map(c_string),
                hint: error.hint().map(c_string),
                key: error.key().map(|k| c_string(&k.key)),
            },
        }
    }

    /// The record lending this error's strings for as long as it lives.
    fn lent(&self) -> TsError {
        let borrowed = Texts {
            detail: self.texts.detail.as_ref(),
            message: &self.texts.message,
            field: self.texts.field.as_ref(),
            hint: self.texts.hint.as_ref(),
            key: self.texts.key.as_ref(),
        };
        record(
            self.status,
            self.provider_code,
            0,
            borrowed.map(|s| s.as_ptr()),
        )
    }

    /// The record owning this error's strings, which `ts_error_free`
    /// takes back.
    fn owned(self) -> TsError {
        let released = self.texts.map(|s| s.into_raw().cast_const());
        record(self.status, self.provider_code, TS_ERROR_OWNED, released)
    }
}

/// One layout for every kind of record, so a lent one and an owned one
/// cannot disagree about which string goes in which field.
fn record(status: Status, provider_code: i32, flags: u32, texts: Texts<*const c_char>) -> TsError {
    let or_null = |s: Option<*const c_char>| s.unwrap_or(ptr::null());
    TsError {
        struct_size: 0,
        status: status.code(),
        provider_code,
        flags,
        detail: or_null(texts.detail),
        message: texts.message,
        field: or_null(texts.field),
        hint: or_null(texts.hint),
        key: or_null(texts.key),
    }
}

/// Writes a refusal whole into a caller's record, which owns its strings.
///
/// Null is allowed and ignored. A `struct_size` this build does not know
/// leaves the record untouched and the call still returns its own status:
/// a `SCHEMA_VERSION` about the diagnostics would replace the refusal the
/// caller needs (`ffi-abi-and-api-description.md` §6.1).
///
/// # Safety
///
/// `out_error` must be null or a `TsError` valid for reads and writes.
pub(crate) unsafe fn refuse_into(out_error: *mut TsError, error: &Error) {
    // SAFETY: the caller's contract.
    let Some(slot) = (unsafe { out_error.as_mut() }) else {
        return;
    };
    if check_size::<TsError>(slot.struct_size).is_err() {
        return;
    }
    *slot = TsError {
        struct_size: slot.struct_size,
        ..StoredError::of(error, 0).owned()
    };
}

/// Releases the strings of a record a failed constructor wrote, and
/// zeroes it but for its size; null, a lent record from
/// `ts_context_last_error`, and a record already freed are all ignored.
///
/// # Safety
///
/// `error` must be null or a `TsError` valid for reads and writes, whose
/// strings, when `flags` carries `TS_ERROR_OWNED`, are the ones this
/// library wrote there and are not used again.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_error_free(error: *mut TsError) {
    // SAFETY: the entry point's contract.
    let Some(slot) = (unsafe { error.as_mut() }) else {
        return;
    };
    if slot.flags & TS_ERROR_OWNED == 0 {
        return;
    }
    for text in [slot.detail, slot.message, slot.field, slot.hint, slot.key] {
        if !text.is_null() {
            // SAFETY: an owned record's strings came from
            // `CString::into_raw` in `StoredError::owned`, and the flag
            // is cleared below, so each is taken back exactly once.
            drop(unsafe { CString::from_raw(text.cast_mut()) });
        }
    }
    *slot = TsError {
        struct_size: slot.struct_size,
        ..TsError::ok()
    };
}

impl TsError {
    /// The record of a call that succeeded: `OK`, and no strings.
    fn ok() -> TsError {
        TsError {
            struct_size: 0,
            status: Status::Ok.code(),
            provider_code: 0,
            flags: 0,
            detail: ptr::null(),
            message: ptr::null(),
            field: ptr::null(),
            hint: ptr::null(),
            key: ptr::null(),
        }
    }
}

impl TsContext {
    /// Builds a context from options, a provider and a locale.
    ///
    /// # Errors
    ///
    /// An unknown profile, a patch that does not parse or contradicts the
    /// profile, a vtable that does not bind, an unknown locale.
    pub fn build(
        texts: &OptionTexts<'_>,
        ephemeris: teistro::Ephemeris,
    ) -> Result<TsContext, Error> {
        // **The façade composes it.** This function used to resolve the
        // profile, parse the patch, load the embedded bundles, start the
        // locale engine, read the ΔT knob and wrap the provider in its
        // cache -- and `teistro::Context` does all of that, because a
        // Rust consumer needed it too and two compositions kept equal by
        // hand is one composition and a hope.
        let mut building = teistro::Context::builder();
        if let Some(id) = texts.profile {
            building = building.profile(id);
        }
        if let Some(json) = texts.settings_json {
            building = building.settings_json(json);
        }
        if let Some(tag) = texts.locale {
            building = building.locale(tag);
        }
        if let Some(json) = texts.layouts_json {
            for layout in layouts_of(json)? {
                building = building.layout(layout);
            }
        }
        if let Some(json) = texts.dashas_json {
            for definition in dashas_of(json)? {
                building = building.dasha_system(definition);
            }
        }
        // One entry, never a chain: a C caller names one ephemeris and
        // gets it or a refusal, which is what `ts_context_new`'s
        // selector means. A chain is the ergonomic layers' shape,
        // assembled above this boundary.
        // The builder names a registered layout by its place among the
        // layouts; here those are the rows of `options.layouts_json`.
        let inner = building.ephemeris([ephemeris]).build().map_err(|error| {
            // The builder names a registered row by its place among the
            // layouts or the dashas; here those are the options' arrays.
            let renamed = error.field().and_then(|field| {
                field
                    .strip_prefix("layouts")
                    .map(|rest| format!("options.layouts_json{rest}"))
                    .or_else(|| {
                        field
                            .strip_prefix("dashas")
                            .map(|rest| format!("options.dashas_json{rest}"))
                    })
            });
            match renamed {
                Some(field) => error.with_field(field),
                None => error,
            }
        })?;
        Ok(TsContext {
            inner,
            scratch: RefCell::new(Scratch::default()),
            loaded: None,
        })
    }

    /// The same context, keeping a loaded adapter's library alive for as
    /// long as it lives (ADR-0029).
    #[must_use]
    pub(crate) fn keeping(mut self, loaded: crate::provider::Keepalive) -> TsContext {
        self.loaded = Some(loaded);
        self
    }

    /// The SDK's own context, for what a binding reaches through this
    /// one.
    #[must_use]
    pub fn sdk(&self) -> &teistro::Context {
        &self.inner
    }

    /// The resolved settings.
    #[must_use]
    pub fn settings(&self) -> &Settings {
        self.inner.settings()
    }

    /// The whole resolution, which the chart layer takes: the settings
    /// with the profile they came from and what was applied to get them.
    #[must_use]
    pub fn resolved(&self) -> &Resolved {
        self.inner.resolved()
    }

    /// The profile the settings came from.
    #[must_use]
    pub fn profile(&self) -> &str {
        self.inner.profile()
    }

    /// The provider, when the context has one.
    #[must_use]
    pub fn provider(&self) -> Option<&dyn EphemerisProvider> {
        self.inner.ephemeris()
    }

    /// The Delta T model the settings chose.
    #[must_use]
    pub fn delta_t(&self) -> DeltaTModel {
        self.inner.delta_t()
    }

    /// The locale engine.
    #[must_use]
    pub fn intl(&self) -> Ref<'_, Intl> {
        self.inner.locale_engine()
    }

    /// The locale engine, to change it.
    #[must_use]
    pub fn intl_mut(&self) -> RefMut<'_, Intl> {
        self.inner.locale_engine_mut()
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
        scratch.error = error.map(|e| StoredError::of(&e, provider_code));
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

/// Which of the SDK's own providers a caller asked for, and the one it
/// gets.
///
/// The selector and `TS_CONTEXT_TEST_PROVIDER` are two spellings of one
/// question, so they are answered in one place: the selector decides
/// whenever it is not `NONE`, and the flag decides when it is. A total
/// rule rather than a conflict to report, and one code path, so the two
/// cannot drift apart (ADR-0028).
fn resolve(ephemeris: u8, flags: u32) -> Result<TsEphemeris, Error> {
    let Some(asked) = TsEphemeris::from_repr(ephemeris) else {
        return Err(Error::invalid_arg(format!(
            "options.ephemeris is {ephemeris}, which names no ephemeris; \
             it is 0 for none, 1 for the built-in, 2 for the test provider"
        ))
        .with_field("options.ephemeris"));
    };
    Ok(match asked {
        TsEphemeris::None if flags & TS_CONTEXT_TEST_PROVIDER != 0 => TsEphemeris::Test,
        other => other,
    })
}

/// Creates a context. `options` may be null for every default; `provider`
/// may be null, in which case the `TS_CONTEXT_TEST_PROVIDER` flag selects
/// the analytic test provider and no flag leaves the context without an
/// ephemeris (positions are then `CAPABILITY`); `provider_user_data` is
/// passed back to the vtable's functions untouched and must stay valid
/// until `ts_context_free`. On success `*out_context` owns the context;
/// on failure, when `out_error` is not null, it receives the whole
/// refusal as a record that owns its strings, released by
/// `ts_error_free`.
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
    out_error: *mut TsError,
) -> Status {
    // SAFETY: the entry point's contract.
    unsafe {
        construct(out_context, "out_context", out_error, || {
            build(options, provider, provider_user_data)
        })
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
    let texts = unsafe { texts_of(options) }?;
    let ephemeris = options.map_or(0, |o| o.ephemeris);
    let chosen: teistro::Ephemeris = if provider.is_null() {
        own_ephemeris(ephemeris, flags)?
    } else {
        // SAFETY: non-null; the caller promises a readable vtable whose
        // functions stay valid with `provider_user_data`.
        let bound = unsafe { VtableProvider::bind(ptr::read(provider), provider_user_data) }?;
        teistro::Ephemeris::Provider(Box::new(bound))
    };
    TsContext::build(&texts, chosen)
}

/// The ephemeris a caller asked for by name, or the refusal that says
/// why this build has none.
fn own_ephemeris(ephemeris: u8, flags: u32) -> Result<teistro::Ephemeris, Error> {
    Ok(match resolve(ephemeris, flags)? {
        TsEphemeris::None => teistro::Ephemeris::None,
        TsEphemeris::Test => teistro::Ephemeris::Test,
        TsEphemeris::Builtin => builtin()?,
    })
}

/// The built-in ephemeris, when this library was built with it.
///
/// A build without it refuses **by name**: not a silent fall back to no
/// ephemeris, and not a silent fall back to the test provider, because
/// either would answer a chart the caller did not ask for (ADR-0028).
///
/// The façade gates the *variant* on the feature, so a Rust consumer
/// without it cannot name `Builtin` at all -- a compile error rather
/// than this refusal, which is the better answer where a binding can
/// have it. A C caller passes a number, so the refusal stays here.
#[cfg(feature = "builtin-ephemeris")]
#[expect(
    clippy::unnecessary_wraps,
    reason = "the other half of this cfg pair is all error; one signature               keeps the caller from having two"
)]
fn builtin() -> Result<teistro::Ephemeris, Error> {
    Ok(teistro::Ephemeris::Builtin)
}

#[cfg(not(feature = "builtin-ephemeris"))]
fn builtin() -> Result<teistro::Ephemeris, Error> {
    Err(Error::new(
        teistro_core::error::Status::Unsupported,
        "this build of the library has no built-in ephemeris: rebuild with \
         the `builtin-ephemeris` feature, or pass a provider vtable to \
         ts_context_new",
    )
    .with_field("options.ephemeris"))
}

/// The strings an options record carries, each optional, by name: a tuple
/// of them let two constructors transpose one without a compiler noticing.
#[derive(Clone, Copy, Debug, Default)]
pub struct OptionTexts<'a> {
    /// The shipped profile's id.
    pub profile: Option<&'a str>,
    /// A JSON settings patch over the profile.
    pub settings_json: Option<&'a str>,
    /// The locale every render resolves from.
    pub locale: Option<&'a str>,
    /// A JSON array of the consumer's own layout rows.
    pub layouts_json: Option<&'a str>,
    /// A JSON array of the consumer's own dasha system definitions.
    pub dashas_json: Option<&'a str>,
}

/// A consumer's layout rows, each read strictly and checked by the rules a
/// shipped row passes, refused by its place in the array
/// (`03-design/chart-geometry.md` §7f).
fn layouts_of(json: &str) -> Result<Vec<teistro::Layout>, Error> {
    const ROOT: &str = "options.layouts_json";
    let rows: Vec<serde_json::Value> = teistro_core::strict::read(json, ROOT)?;
    rows.into_iter()
        .enumerate()
        .map(|(index, row)| {
            let at = format!("{ROOT}[{index}]");
            let layout: teistro::Layout = teistro_core::strict::read_value(&row, &at)?;
            layout.validate().map_err(|error| error.under(&at))?;
            Ok(layout)
        })
        .collect()
}

/// A consumer's dasha system definitions, each read strictly; the context's
/// registry checks each by the rules a shipped row passes and names it by
/// its place.
fn dashas_of(json: &str) -> Result<Vec<teistro::dasha::DashaDefinition>, Error> {
    const ROOT: &str = "options.dashas_json";
    let rows: Vec<serde_json::Value> = teistro_core::strict::read(json, ROOT)?;
    rows.into_iter()
        .enumerate()
        .map(|(index, row)| teistro_core::strict::read_value(&row, &format!("{ROOT}[{index}]")))
        .collect()
}

/// The strings an options record carries, checked and borrowed.
///
/// Shared by both ways of making a context, so a field added here reaches
/// each of them and neither can forget one.
///
/// # Safety
///
/// `options` must be null or a readable record whose strings stay valid
/// for the returned lifetime.
pub(crate) unsafe fn read_options<'a>(
    options: *const TsContextOptions,
) -> Result<OptionTexts<'a>, Error> {
    let options = if options.is_null() {
        None
    } else {
        // SAFETY: the caller's contract.
        Some(unsafe { read_in(options, "options") }?)
    };
    // SAFETY: the caller's contract.
    unsafe { texts_of(options) }
}

/// # Safety
///
/// The record's strings must stay valid for the returned lifetime.
unsafe fn texts_of<'a>(options: Option<&TsContextOptions>) -> Result<OptionTexts<'a>, Error> {
    let field = |pick: fn(&TsContextOptions) -> *const c_char, name: &str| {
        // SAFETY: the caller's contract: null, or a NUL-terminated string
        // that stays valid for the returned lifetime.
        unsafe { optional_text(options.map_or(ptr::null(), pick), name) }
    };
    Ok(OptionTexts {
        profile: field(|o| o.profile, "options.profile")?,
        settings_json: field(|o| o.settings_json, "options.settings_json")?,
        locale: field(|o| o.locale, "options.locale")?,
        layouts_json: field(|o| o.layouts_json, "options.layouts_json")?,
        dashas_json: field(|o| o.dashas_json, "options.dashas_json")?,
    })
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
    let record = scratch
        .error
        .as_ref()
        .map_or_else(TsError::ok, StoredError::lent);

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

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "tests fail by panicking"
    )]

    use super::{
        OptionTexts, TS_CONTEXT_TEST_PROVIDER, TsContext, TsEphemeris, own_ephemeris, resolve,
    };
    use teistro_core::settings::DEFAULT_CACHE_CELLS;
    use teistro_port_ephemeris::EphemerisProvider;

    /// The knob's default is the port's default, in one place.
    #[test]
    fn the_shipped_profile_remembers_what_a_range_needs() {
        let context = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::None)
            .expect("the default profile");
        assert_eq!(
            context.settings().provider.cache_cells,
            DEFAULT_CACHE_CELLS,
            "the root profile takes the measured capacity"
        );
    }

    /// A caller can turn the memo off, and size it, through the settings a
    /// binding already passes as JSON.
    #[test]
    fn the_knob_is_reachable_from_a_settings_patch() {
        for cells in [0u32, 1, 4096] {
            let patch = format!(r#"{{"provider":{{"cache_cells":{cells}}}}}"#);
            let context = TsContext::build(
                &OptionTexts {
                    settings_json: Some(&patch),
                    ..OptionTexts::default()
                },
                teistro::Ephemeris::None,
            )
            .expect("the patch applies");
            assert_eq!(context.settings().provider.cache_cells, cells);
        }
    }

    /// The selector and the flag are two spellings of one question, and
    /// the rule between them is total: the selector decides whenever it
    /// is not `NONE`, the flag decides when it is. Nothing here is a
    /// conflict to report, so nothing here can be reported inconsistently.
    #[test]
    fn the_selector_and_the_flag_answer_one_question() {
        let none = 0;
        let builtin = TsEphemeris::Builtin as u8;
        let test = TsEphemeris::Test as u8;
        assert_eq!(resolve(none, 0).unwrap(), TsEphemeris::None);
        assert_eq!(
            resolve(none, TS_CONTEXT_TEST_PROVIDER).unwrap(),
            TsEphemeris::Test,
            "the flag still says what it always said"
        );
        assert_eq!(resolve(test, 0).unwrap(), TsEphemeris::Test);
        assert_eq!(resolve(builtin, 0).unwrap(), TsEphemeris::Builtin);
        assert_eq!(
            resolve(builtin, TS_CONTEXT_TEST_PROVIDER).unwrap(),
            TsEphemeris::Builtin,
            "a caller who named one is not overruled by a flag they left set"
        );
    }

    /// A number that names no ephemeris is refused by name rather than
    /// rounded down to the default, because a caller who meant something
    /// this library does not have should hear so.
    #[test]
    fn an_unknown_ephemeris_is_refused_by_name() {
        let error = resolve(7, 0).expect_err("7 names nothing");
        assert_eq!(error.status, teistro_core::error::Status::InvalidArg);
        assert_eq!(error.field(), Some("options.ephemeris"));
        assert!(
            error.to_string().contains('7'),
            "the refusal must quote what was asked: {error}"
        );
    }

    /// The built-in answers where the test provider only pretends to.
    ///
    /// This is Phase 3's promise reduced to one assertion: a context
    /// built with nothing but a selector computes a real position. What
    /// the position *is* is measured elsewhere — in
    /// `03-design/completion-measured.md`, against an engine — because a
    /// test that asserted an accuracy figure would be asserting a number
    /// this crate cannot check.
    #[cfg(feature = "builtin-ephemeris")]
    #[test]
    fn the_built_in_selector_yields_an_ephemeris_that_computes() {
        // Opened here rather than asserted as a variant: what this pins
        // is that a selector alone yields something that computes.
        let provider = own_ephemeris(TsEphemeris::Builtin as u8, 0)
            .expect("the built-in is compiled in")
            .open()
            .expect("it opens")
            .expect("and is a provider");
        let capabilities = provider.capabilities();
        assert_eq!(capabilities.identity.name, "teistro-builtin");
        assert!(
            capabilities.identity.tier.is_some(),
            "the built-in stamps the tier it was built at"
        );
        assert!(
            capabilities.bodies.len() >= 9,
            "a Vedic chart needs nine grahas and it declares {}",
            capabilities.bodies.len()
        );
    }

    /// A build without the built-in refuses **by name**, and does not
    /// quietly hand back the test provider or no provider at all —
    /// either would answer a chart the caller did not ask for.
    #[cfg(not(feature = "builtin-ephemeris"))]
    #[test]
    fn without_the_feature_the_built_in_is_refused_by_name() {
        // A `match` rather than `expect_err`, which would want `Debug` on
        // a boxed trait object that has no reason to carry it.
        let Err(error) = own_ephemeris(TsEphemeris::Builtin as u8, 0) else {
            panic!("a build without the feature must refuse the built-in");
        };
        assert_eq!(error.status, teistro_core::error::Status::Unsupported);
        assert!(
            error.to_string().contains("builtin-ephemeris"),
            "the refusal must name the feature: {error}"
        );
    }
}
