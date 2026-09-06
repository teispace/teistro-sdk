//! The C ABI of the Teistro SDK: the one audited boundary every binding is
//! generated against (`docs/03-design/ffi-abi-and-api-description.md`,
//! ADR-0001, ADR-0007, ADR-0023).
//!
//! The conventions (`docs/02-architecture/06-api-conventions.md`): no
//! global state, a context handle first in every call that has one; every
//! boundary struct begins with `struct_size`, set by the caller, and a
//! size this build does not know is refused as `SCHEMA_VERSION`; a status
//! code on every call with the error's message, field and hint kept on the
//! context for `ts_context_last_error`; no panic escapes, a caught one is
//! `INTERNAL`; tree- and grid-shaped results cross as a result blob
//! ([`teistro_idl::blob`]) the library owns until `ts_blob_free`; strings
//! the library allocates are freed by `ts_string_free`, strings it lends
//! are valid until the next call on the same context.
//!
//! - [`context`]: the handle, its options, the last error;
//! - [`keys`]: catalogue keys and ids;
//! - [`calendar`]: dates in every shipped calendar, the fixed day;
//! - [`time`]: civil times to instants with the zone metadata, the scales;
//! - [`intl`]: the locale engine over the embedded bundles and loaded packs;
//! - [`positions`]: positions over the ephemeris port, completed;
//! - [`strings`] and [`blob`]: the owned and borrowed result carriers;
//! - [`schemas`]: the result blob schemas, read by the generators.
//!
//! The API description is extracted from this crate's source, the port's
//! vtable and the core's status by `cargo xtask gen ffi`, which writes
//! `idl/api.json` and `bindings/c/include/teistro.h`.
//!
//! ```
//! use core::ptr;
//! use teistro_core::Status;
//! use teistro_ffi::context::{TsContext, ts_context_free, ts_context_new};
//!
//! let mut context: *mut TsContext = ptr::null_mut();
//! // SAFETY: a null options pointer selects every default; the slot is valid.
//! let status = unsafe { ts_context_new(ptr::null(), ptr::null(), ptr::null_mut(), &raw mut context, ptr::null_mut()) };
//! assert_eq!(status, Status::Ok);
//! // SAFETY: the handle came from `ts_context_new` and is not used again.
//! unsafe { ts_context_free(context) };
//! ```

pub mod blob;
pub mod calendar;
pub mod context;
pub mod intl;
pub mod keys;
pub mod positions;
pub mod schemas;
pub mod strings;
mod support;
pub mod time;

use core::ffi::c_char;

use teistro_core::Status;

/// The ABI version `ts_abi_version` returns; a binding refuses to load a
/// library whose version it was not generated for.
///
/// `api: constant`
pub const TS_ABI_VERSION: u32 = 1;

/// The SDK version this crate was built as, the one `ts_sdk_version`
/// returns and the description records.
pub const SDK_VERSION: &str = env!("CARGO_PKG_VERSION");

/// A context flag: use the SDK's analytic test provider when no provider
/// vtable is given. For tests and examples only; its positions are not
/// astronomy.
///
/// `api: constant`
pub const TS_CONTEXT_TEST_PROVIDER: u32 = 1 << 0;

/// The ABI version this library implements.
#[unsafe(no_mangle)]
#[allow(
    unsafe_code,
    reason = "the exported-symbol attribute is unsafe in edition 2024"
)]
pub extern "C" fn ts_abi_version() -> u32 {
    TS_ABI_VERSION
}

/// The SDK version as a static NUL-terminated string (`0.0.0`).
#[unsafe(no_mangle)]
#[allow(
    unsafe_code,
    reason = "the exported-symbol attribute is unsafe in edition 2024"
)]
pub extern "C" fn ts_sdk_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr().cast()
}

/// The catalogue schema version every result's provenance stamps.
#[unsafe(no_mangle)]
#[allow(
    unsafe_code,
    reason = "the exported-symbol attribute is unsafe in edition 2024"
)]
pub extern "C" fn ts_catalogue_version() -> u32 {
    teistro_core::catalogue::SCHEMA_VERSION
}

/// The profile a context uses when its options name none, as a static
/// NUL-terminated string.
#[unsafe(no_mangle)]
#[allow(
    unsafe_code,
    reason = "the exported-symbol attribute is unsafe in edition 2024"
)]
pub extern "C" fn ts_default_profile() -> *const c_char {
    concat!("parashari-classical", "\0").as_ptr().cast()
}

/// A static NUL-terminated English phrase for a status code; `unknown
/// status` for a code this build does not know.
#[unsafe(no_mangle)]
#[allow(
    unsafe_code,
    reason = "the exported-symbol attribute is unsafe in edition 2024"
)]
pub extern "C" fn ts_status_message(status: i32) -> *const c_char {
    let phrase: &'static str = match Status::from_code(status) {
        Some(Status::Ok) => "ok\0",
        Some(Status::InvalidArg) => "invalid argument\0",
        Some(Status::OutOfRange) => "out of range\0",
        Some(Status::Capability) => "capability missing\0",
        Some(Status::Provider) => "provider failed\0",
        Some(Status::NotConverged) => "did not converge\0",
        Some(Status::Unsupported) => "unsupported\0",
        Some(Status::Pack) => "pack rejected\0",
        Some(Status::Limit) => "limit exceeded\0",
        Some(Status::SchemaVersion) => "schema version mismatch\0",
        Some(Status::Internal) => "internal error\0",
        _ => "unknown status\0",
    };
    phrase.as_ptr().cast()
}

#[cfg(test)]
mod tests {
    #![allow(unsafe_code, clippy::unwrap_used, reason = "tests cross the boundary")]

    use core::ffi::CStr;

    use super::*;

    #[test]
    fn the_static_answers_read_back() {
        assert_eq!(ts_abi_version(), 1);
        // SAFETY: static NUL-terminated strings.
        let (version, profile, message, unknown) = unsafe {
            (
                CStr::from_ptr(ts_sdk_version()),
                CStr::from_ptr(ts_default_profile()),
                CStr::from_ptr(ts_status_message(Status::Provider.code())),
                CStr::from_ptr(ts_status_message(7)),
            )
        };
        assert_eq!(version.to_str().unwrap(), env!("CARGO_PKG_VERSION"));
        assert_eq!(
            profile.to_str().unwrap(),
            teistro_core::settings::DEFAULT_PROFILE
        );
        assert_eq!(message.to_str().unwrap(), Status::Provider.phrase());
        assert_eq!(unknown.to_str().unwrap(), "unknown status");
        assert_eq!(ts_catalogue_version(), 1);
        for status in [Status::Ok, Status::InvalidArg, Status::Internal] {
            // SAFETY: a static string.
            let phrase = unsafe { CStr::from_ptr(ts_status_message(status.code())) };
            assert_eq!(phrase.to_str().unwrap(), status.phrase());
        }
    }
}
