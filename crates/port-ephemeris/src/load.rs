//! Opening an adapter: a provider that lives in another shared library
//! (ADR-0029), loaded by the three symbols [`crate::plugin`] names.
//!
//! One loader, in the port, for every caller: the C boundary's
//! `ts_provider_load`, a Rust consumer's [`Adapter::load`] and the agent
//! server's `--plugin`. Each grew its own copy of the symbol names, the
//! two version checks and the error vocabulary before, or would have.
//!
//! # Lifetimes
//!
//! An [`Adapter`] is reference counted, and every provider it binds
//! ([`Adapter::provider`]) holds a reference of its own, so one adapter
//! may found several contexts and the handles may be dropped in any
//! order. The library is unloaded when the last reference goes, after
//! the adapter's own `close` has run.
//!
//! # Not on wasm
//!
//! A wasm module cannot open a shared library, so this module is compiled
//! out there (ADR-0029).

#![cfg(not(target_family = "wasm"))]
#![allow(
    unsafe_code,
    reason = "the loader: dlopen and a C entry point, one SAFETY comment per block"
)]

use core::ffi::{c_char, c_void};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use libloading::{Library, Symbol};
use teistro_core::error::{Error, Status};

use crate::ProviderError;
use crate::plugin::{
    ABI_VERSION_SYMBOL, CLOSE_SYMBOL, OPEN_SYMBOL, PLUGIN_ABI_VERSION, PluginAbiVersionFn,
    PluginCloseFn, PluginOpenFn,
};
use crate::vtable::{ProviderVtable, VtableProvider};

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
struct Loaded {
    vtable: ProviderVtable,
    user_data: *mut c_void,
    close: PluginCloseFn,
    /// Never read, and that is its whole purpose: holding the library
    /// means the code behind `vtable`'s function pointers is still
    /// mapped. It is dropped last, which unloads it.
    #[expect(dead_code, reason = "held for its `Drop`, which unloads the library")]
    library: Library,
}

// SAFETY: a provider behind a vtable is required to be callable from any
// thread with its `user_data` (`VtableProvider::bind`'s contract). What
// is shared here is the right to *keep* the library loaded.
unsafe impl Send for Loaded {}
// SAFETY: as above.
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

/// An adapter opened from its platform binary: the provider inside it,
/// bound as often as a caller needs ([`Adapter::provider`]).
///
/// ```no_run
/// use teistro_port_ephemeris::load::Adapter;
///
/// // SAFETY: the adapter is a file this program installed and trusts.
/// let adapter = unsafe { Adapter::load("/opt/adapters/libteimeris_teistro.so", None) }?;
/// let provider = adapter.provider()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone)]
pub struct Adapter {
    loaded: Arc<Loaded>,
    path: Arc<str>,
}

impl core::fmt::Debug for Adapter {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Adapter")
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}

impl Adapter {
    /// Opens the adapter at `path` and the provider inside it.
    ///
    /// `path` is the adapter's platform binary, the file its package
    /// ships; `config_json` is that adapter's own options, or `None`.
    ///
    /// # Errors
    ///
    /// `UNSUPPORTED` when the file is not an adapter, `SCHEMA_VERSION`
    /// when it is one of another plugin ABI, and the adapter's own
    /// refusal (`DATA_MISSING`, `INVALID_ARG`, …) passed through with its
    /// message; each names `path`.
    ///
    /// # Safety
    ///
    /// Loading a library runs its initialisers, and its entry points are
    /// trusted to keep the plugin contract: `path` must be a file the
    /// caller trusts.
    pub unsafe fn load(path: &str, config_json: Option<&str>) -> Result<Adapter, Error> {
        // SAFETY: the caller's undertaking, above.
        let library = unsafe { Library::new(path) }.map_err(|error| {
            Error::new(
                Status::Unsupported,
                format!("cannot load the adapter at `{path}`: {error}"),
            )
            .with_field("path")
        })?;
        // The version before anything else: a library built against
        // another version of this contract may not have the symbols about
        // to be looked for, and asking it for them is how a loader crashes
        // rather than reports.
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
        let open: Symbol<'_, PluginOpenFn> =
            unsafe { library.get(OPEN_SYMBOL.to_bytes_with_nul()) }
                .map_err(|_| missing(path, OPEN_SYMBOL))?;
        // SAFETY: as above.
        let close: Symbol<'_, PluginCloseFn> =
            unsafe { library.get(CLOSE_SYMBOL.to_bytes_with_nul()) }
                .map_err(|_| missing(path, CLOSE_SYMBOL))?;
        let (open, close) = (*open, *close);
        let config = config_json
            .map(|text| {
                std::ffi::CString::new(text).map_err(|_| {
                    Error::invalid_arg("config_json contains a NUL").with_field("config_json")
                })
            })
            .transpose()?;
        // Zeroed rather than plausible: if an adapter returns success
        // without writing, `VtableProvider::bind` refuses it for a zero
        // size and a zero version rather than calling a null pointer.
        let mut vtable = ProviderVtable::EMPTY;
        let mut user_data: *mut c_void = core::ptr::null_mut();
        let mut message = [0 as c_char; ERROR_CAPACITY];
        // SAFETY: the adapter's contract; every pointer is live for the
        // call and the buffer is the length given.
        let code = unsafe {
            open(
                config.as_ref().map_or(core::ptr::null(), |c| c.as_ptr()),
                &raw mut vtable,
                &raw mut user_data,
                message.as_mut_ptr(),
                message.len(),
            )
        };
        if code != 0 {
            // SAFETY: the adapter wrote a NUL-terminated string or
            // nothing, and the buffer's last byte is a terminator either
            // way.
            let detail = unsafe { core::ffi::CStr::from_ptr(message.as_ptr()) }
                .to_string_lossy()
                .into_owned();
            return Err(refused(code, path, &detail));
        }
        Ok(Adapter {
            loaded: Arc::new(Loaded {
                vtable,
                user_data,
                close,
                library,
            }),
            path: Arc::from(path),
        })
    }

    /// The file the adapter was opened from.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// The adapter's provider, bound and holding the library open for as
    /// long as it lives: one per context, all over the one opened
    /// provider.
    ///
    /// # Errors
    ///
    /// A vtable the adapter wrote that does not bind: another vtable
    /// version or size, or a missing required function.
    pub fn provider(&self) -> Result<VtableProvider, ProviderError> {
        // SAFETY: the vtable and its user data came from this library's
        // own `open`, under `load`'s contract, and the reference handed
        // to the provider keeps the library loaded while it lives.
        let bound = unsafe { VtableProvider::bind(self.loaded.vtable, self.loaded.user_data) }?;
        let owner: Arc<dyn core::any::Any + Send + Sync> = self.loaded.clone();
        Ok(bound.keeping(owner))
    }
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

/// An adapter's own refusal, kept as its own rather than flattened: the
/// port already turns a provider's code and message into the SDK's error,
/// and a second table here would be a second opinion about what `-3`
/// means.
fn refused(code: i32, path: &str, detail: &str) -> Error {
    let said = if detail.is_empty() {
        format!("the adapter at `{path}` refused with code {code} and no message")
    } else {
        format!("the adapter at `{path}`: {detail}")
    };
    Error::from(ProviderError::from_code(code, &said)).with_field("path")
}
