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
//!
//! # Not on wasm
//!
//! A wasm module cannot open a shared library, so a wasm build has the
//! built-in ephemeris and a host provider and not this (ADR-0029). The
//! whole module is compiled out there, and the description reads that
//! condition off this file: its three functions are marked native-only,
//! the C header guards them, and a wasm binding emits nothing for them.

#![cfg(not(target_family = "wasm"))]
#![allow(unsafe_code, reason = "C entry points over the port's loader")]

use core::ffi::c_char;

use teistro_core::error::{Error, Status};
use teistro_port_ephemeris::load::Adapter;

use crate::support::optional_text;

/// An ephemeris loaded from a shared library. Free with
/// [`ts_provider_free`]; a context built from it keeps its own reference,
/// so the order does not matter.
#[derive(Debug)]
pub struct TsProvider {
    adapter: Adapter,
}

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
    out_error: *mut crate::context::TsError,
) -> Status {
    // SAFETY: the entry point's contract.
    unsafe {
        crate::support::construct(out_provider, "out_provider", out_error, || {
            load(path, config_json)
        })
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
    let adapter = unsafe { Adapter::load(path, config) }?;
    Ok(TsProvider { adapter })
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
    out_error: *mut crate::context::TsError,
) -> Status {
    // SAFETY: the entry point's contract.
    unsafe {
        crate::support::construct(out_context, "out_context", out_error, || {
            if provider.is_null() {
                return Err(crate::support::null("provider"));
            }
            with_provider(options, provider)
        })
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
    let bound = unsafe { &*provider }.adapter.provider()?;
    // SAFETY: the entry point's contract.
    let texts = unsafe { crate::context::read_options(options) }?;
    crate::context::TsContext::build(&texts, teistro::Ephemeris::Provider(Box::new(bound)))
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
