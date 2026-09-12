//! Loading an ephemeris that lives in another shared library (ADR-0029).
//!
//! The SDK is Apache-2.0 and the engines worth using are AGPL-3.0, so an
//! engine cannot be linked into this library and has to be *loaded* into
//! it. A consumer installs an adapter package, the package ships a
//! platform binary, and this opens it.
//!
//! # Why the loader is here and not in each binding
//!
//! Node, Python and Dart can each open a shared library themselves, and
//! if each did, each would grow its own copy of the symbol names, the two
//! version checks, the error vocabulary and the ordering rules — which is
//! exactly how the three bindings once grew three copies of the provider
//! error policy before `validate` moved to this side. One implementation,
//! one place the `unsafe` lives, and **no table of function pointers
//! crosses a host language**.
//!
//! # Lifetimes, and why nothing here has an ordering footgun
//!
//! A loaded provider is reference counted. `ts_provider_load` hands back
//! a handle holding one reference; a context built from it takes another;
//! `ts_provider_free` and `ts_context_free` each drop their own. So a
//! caller may free them in either order, and one provider may found
//! several contexts — a consumer casting under two profiles opens the
//! engine once.
//!
//! The library is unloaded only when the last reference goes, **after**
//! the adapter's own `close` has run. Dropping it sooner would leave a
//! vtable of function pointers into an address space that no longer has
//! the code.

#![allow(unsafe_code, reason = "the loader: dlopen and a C entry point")]

use core::ffi::{c_char, c_void};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use libloading::{Library, Symbol};
use teistro_core::error::{Error, Status};
use teistro_port_ephemeris::ProviderError;
use teistro_port_ephemeris::plugin::{
    ABI_VERSION_SYMBOL, CLOSE_SYMBOL, OPEN_SYMBOL, PLUGIN_ABI_VERSION, PluginAbiVersionFn,
    PluginCloseFn, PluginOpenFn,
};
use teistro_port_ephemeris::vtable::ProviderVtable;

use crate::strings::TsString;
use crate::support::optional_text;

/// How much room an adapter is given for its refusal.
///
/// Generous, because the useful part of such a message is the path or the
/// option that was wrong and both can be long, and cheap, because it is
/// one stack buffer per load rather than per call.
const ERROR_CAPACITY: usize = 1024;

/// A library, the provider it opened, and the way to close it.
///
/// The field order is the drop order and it is deliberate: `close` runs
/// against the still-loaded library, and only then does `library` fall
/// out of scope and unload it.
pub(crate) struct Loaded {
    vtable: ProviderVtable,
    user_data: *mut c_void,
    close: PluginCloseFn,
    /// Never read, and that is its whole purpose: holding the library
    /// means the code behind `vtable`'s function pointers is still
    /// mapped. It is dropped last, which unloads it.
    #[expect(dead_code, reason = "held for its `Drop`, which unloads the library")]
    library: Library,
}

// SAFETY: the port requires a provider behind a vtable to be usable from
// the thread that owns the context, and a context is used by one thread
// at a time (`TsContext`'s own contract). What is shared here is the
// right to *keep* the library loaded, not to call into it concurrently.
unsafe impl Send for Loaded {}
unsafe impl Sync for Loaded {}

impl Drop for Loaded {
    fn drop(&mut self) {
        // An adapter's close may panic; that must not unwind out of a
        // drop into the caller's language.
        let close = self.close;
        let user_data = self.user_data;
        let _ = catch_unwind(AssertUnwindSafe(|| {
            // SAFETY: `user_data` is what this library's own `open` wrote,
            // closed exactly once, here.
            unsafe { close(user_data) };
        }));
    }
}

/// An ephemeris loaded from a shared library. Free with
/// [`ts_provider_free`]; a context built from it keeps its own reference,
/// so the order does not matter.
pub struct TsProvider {
    loaded: Arc<Loaded>,
}

impl core::fmt::Debug for TsProvider {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TsProvider").finish_non_exhaustive()
    }
}

impl TsProvider {
    /// The vtable and `user_data` a context binds, and the reference that
    /// keeps the library loaded while it does.
    pub(crate) fn parts(&self) -> (ProviderVtable, *mut c_void, Arc<Loaded>) {
        (
            self.loaded.vtable,
            self.loaded.user_data,
            Arc::clone(&self.loaded),
        )
    }
}

/// A reference that keeps a loaded library alive, held beside the
/// provider it came from.
///
/// The context stores one of these next to its boxed provider rather than
/// wrapping the provider in a delegating type: the trait has ten methods
/// and a wrapper would forward all of them to say one thing, which is
/// that the library outlives the calls.
pub(crate) type Keepalive = Arc<Loaded>;

/// Opens an adapter and the provider inside it.
///
/// `path` is the adapter's platform binary — the file its package ships.
/// `config_json` is that adapter's own options, or null; what they mean
/// is the adapter's to say and its package's to type.
///
/// `UNSUPPORTED` when the file is not an adapter of this version,
/// `DATA_MISSING` when it is and its data is not there, `INVALID_ARG`
/// when its configuration is wrong — each of them the adapter's own
/// judgement, passed through with its message rather than replaced.
///
/// # Safety
///
/// Every pointer must be null or valid for the access its documentation
/// describes, for the duration of the call. Loading a library runs its
/// initialisers, so `path` must be a file the caller trusts.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_provider_load(
    path: *const c_char,
    config_json: *const c_char,
    out_provider: *mut *mut TsProvider,
    out_error: *mut TsString,
) -> Status {
    if out_provider.is_null() {
        return Status::InvalidArg;
    }
    let opened = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: the entry point's contract.
        unsafe { load(path, config_json) }
    }))
    .unwrap_or_else(|_| {
        Err(Error::internal(
            "a panic was caught while loading a provider",
        ))
    });
    match opened {
        Ok(provider) => {
            // SAFETY: non-null; the caller promises a writable slot.
            unsafe { out_provider.write(Box::into_raw(Box::new(provider))) };
            Status::Ok
        }
        Err(error) => {
            if !out_error.is_null() {
                // SAFETY: non-null; the caller promises a writable slot.
                unsafe { out_error.write(TsString::from_string(error.to_string())) };
            }
            error.status
        }
    }
}

/// # Safety
///
/// As [`ts_provider_load`].
unsafe fn load(path: *const c_char, config_json: *const c_char) -> Result<TsProvider, Error> {
    // SAFETY: the entry point's contract.
    let path = unsafe { optional_text(path, "path") }?
        .ok_or_else(|| Error::invalid_arg("path is required").with_field("path"))?;
    // SAFETY: the entry point's contract.
    let config = unsafe { optional_text(config_json, "config_json") }?;

    // SAFETY: loading a library runs its initialisers; the entry point's
    // documentation makes trusting the file the caller's undertaking.
    let library = unsafe { Library::new(path) }.map_err(|error| {
        Error::new(
            Status::Unsupported,
            format!("cannot load the adapter at `{path}`: {error}"),
        )
        .with_field("path")
    })?;

    // The version before anything else: a library built against another
    // version of this contract may not have the symbols about to be
    // looked for, and asking it for them is how a loader crashes rather
    // than reports.
    // SAFETY: the symbol is called only if it resolves, and its type is
    // this contract's.
    let version: Symbol<'_, PluginAbiVersionFn> =
        unsafe { library.get(ABI_VERSION_SYMBOL.to_bytes_with_nul()) }.map_err(|_| {
            Error::new(
                Status::Unsupported,
                format!(
                    "`{path}` is a shared library but not a Teistro ephemeris adapter: \
                     it exports no `{}`",
                    ABI_VERSION_SYMBOL.to_string_lossy()
                ),
            )
            .with_field("path")
        })?;
    // SAFETY: resolved just above, with this contract's signature.
    let found = unsafe { version() };
    if found != PLUGIN_ABI_VERSION {
        return Err(Error::new(
            Status::SchemaVersion,
            format!(
                "the adapter at `{path}` implements plugin ABI {found}; \
                 this library speaks {PLUGIN_ABI_VERSION}"
            ),
        )
        .with_field("path"));
    }

    // SAFETY: as above.
    let open: Symbol<'_, PluginOpenFn> = unsafe { library.get(OPEN_SYMBOL.to_bytes_with_nul()) }
        .map_err(|_| missing(path, OPEN_SYMBOL))?;
    // SAFETY: as above.
    let close: Symbol<'_, PluginCloseFn> = unsafe { library.get(CLOSE_SYMBOL.to_bytes_with_nul()) }
        .map_err(|_| missing(path, CLOSE_SYMBOL))?;
    let (open, close) = (*open, *close);

    let config_c = config
        .map(|text| {
            std::ffi::CString::new(text).map_err(|_| {
                Error::invalid_arg("config_json contains a NUL").with_field("config_json")
            })
        })
        .transpose()?;
    // Zeroed rather than plausible: if an adapter returns success without
    // writing, `VtableProvider::bind` refuses it for a zero size and a
    // zero version rather than calling a null pointer.
    let mut vtable = ProviderVtable {
        struct_size: 0,
        abi_version: 0,
        capabilities: None,
        positions: None,
        obliquity: None,
        delta_t: None,
        ayanamsha: None,
        dut1: None,
        horizon_event: None,
        crossings: None,
        native_manifest: None,
        native_call: None,
    };
    let mut user_data: *mut c_void = core::ptr::null_mut();
    let mut message = [0 as c_char; ERROR_CAPACITY];
    // SAFETY: the adapter's contract; every pointer is live for the call
    // and the buffer is the length given.
    let code = unsafe {
        open(
            config_c.as_ref().map_or(core::ptr::null(), |c| c.as_ptr()),
            &raw mut vtable,
            &raw mut user_data,
            message.as_mut_ptr(),
            message.len(),
        )
    };
    if code != 0 {
        // SAFETY: the adapter wrote a NUL-terminated string or nothing,
        // and the buffer's last byte is a terminator either way.
        let detail = unsafe { core::ffi::CStr::from_ptr(message.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        return Err(from_provider_code(code, path, &detail));
    }

    Ok(TsProvider {
        loaded: Arc::new(Loaded {
            vtable,
            user_data,
            close,
            library,
        }),
    })
}

fn missing(path: &str, symbol: &core::ffi::CStr) -> Error {
    Error::new(
        Status::Unsupported,
        format!(
            "the adapter at `{path}` exports `{}` but not `{}`; it is not a complete adapter",
            ABI_VERSION_SYMBOL.to_string_lossy(),
            symbol.to_string_lossy()
        ),
    )
    .with_field("path")
}

/// An adapter's own refusal, kept as its own rather than flattened.
///
/// The port already turns a provider's code and message into the SDK's
/// error, hint and all; a second table here would be a second opinion
/// about what `-3` means, and the two would drift.
fn from_provider_code(code: i32, path: &str, detail: &str) -> Error {
    let said = if detail.is_empty() {
        format!("the adapter at `{path}` refused with code {code} and no message")
    } else {
        format!("the adapter at `{path}`: {detail}")
    };
    Error::from(ProviderError::from_code(code, &said)).with_field("path")
}

/// Creates a context that computes with a **loaded** provider.
///
/// The same as `ts_context_new` in every other respect — `options` may be
/// null for the defaults, and `options.ephemeris` is ignored because this
/// call has already answered the question it asks.
///
/// The context takes its own reference to the adapter, so this handle may
/// be freed immediately afterwards or kept to found another context; the
/// library is unloaded when the last of them goes.
///
/// # Safety
///
/// `provider` must be a live handle from [`ts_provider_load`]; every
/// other pointer must be null or valid for the access its documentation
/// describes, for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_context_new_with_provider(
    options: *const crate::context::TsContextOptions,
    provider: *const TsProvider,
    out_context: *mut *mut crate::context::TsContext,
    out_error: *mut TsString,
) -> Status {
    if out_context.is_null() || provider.is_null() {
        return Status::InvalidArg;
    }
    let built = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: the entry point's contract.
        unsafe { with_provider(options, provider) }
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
                // SAFETY: non-null; the caller promises a writable slot.
                unsafe { out_error.write(TsString::from_string(error.to_string())) };
            }
            error.status
        }
    }
}

/// # Safety
///
/// As [`ts_context_new_with_provider`].
unsafe fn with_provider(
    options: *const crate::context::TsContextOptions,
    provider: *const TsProvider,
) -> Result<crate::context::TsContext, Error> {
    // SAFETY: the entry point's contract.
    let (vtable, user_data, keepalive) = unsafe { &*provider }.parts();
    // SAFETY: the vtable and its user data came from an adapter this
    // library loaded, and `keepalive` holds that library open for at
    // least as long as the context built below.
    let bound = unsafe { teistro_port_ephemeris::VtableProvider::bind(vtable, user_data) }?;
    // SAFETY: the entry point's contract.
    let (profile, settings_json, locale) = unsafe { crate::context::read_options(options) }?;
    crate::context::TsContext::build(profile, settings_json, Some(Box::new(bound)), locale)
        .map(|context| context.keeping(keepalive))
}

/// Frees a loaded provider; null is ignored.
///
/// The library is unloaded when the last reference goes, so a context
/// still using it keeps it alive and the order of these calls does not
/// matter.
///
/// # Safety
///
/// `provider` must be null or a handle from [`ts_provider_load`] that is
/// not used again.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_provider_free(provider: *mut TsProvider) {
    if provider.is_null() {
        return;
    }
    // SAFETY: the caller promises a handle from `ts_provider_load`,
    // freed once.
    drop(unsafe { Box::from_raw(provider) });
}
