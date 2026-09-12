//! The plugin contract: how a provider that lives in **another shared
//! library** is opened, driven and closed (ADR-0029).
//!
//! [`ProviderVtable`] says how a provider is *driven* once it is in hand.
//! This says how it is got hold of: three exported symbols, one
//! configuration string, and a version to check before any of it is
//! called. An adapter implements them with [`export_provider!`] and
//! writes no `unsafe` of its own; the SDK's loader resolves them by the
//! names below.
//!
//! # Why an adapter is a separate library at all
//!
//! Because a licence says so. The SDK is Apache-2.0 and the engines it
//! adapts are AGPL-3.0, so an engine can never be linked into the SDK's
//! own artefact (ADR-0019, ADR-0029). A consumer installs the adapter
//! deliberately, as its own package, and this is the seam where the two
//! meet.
//!
//! # The contract
//!
//! | symbol | signature | when |
//! |---|---|---|
//! | [`ABI_VERSION_SYMBOL`] | [`PluginAbiVersionFn`] | first, and before anything else is called |
//! | [`OPEN_SYMBOL`] | [`PluginOpenFn`] | to build a provider from a configuration |
//! | [`CLOSE_SYMBOL`] | [`PluginCloseFn`] | once, on what `open` handed back |
//!
//! Two versions are checked and they are **not** the same version. This
//! module's [`PLUGIN_ABI_VERSION`] covers the three entry points above;
//! [`VTABLE_ABI_VERSION`] covers the table of function pointers they hand
//! back, and [`crate::VtableProvider::bind`] checks it. They can change
//! independently — a new override on the vtable is not a change to how a
//! library is opened — and a loader that conflated them would report the
//! wrong one.
//!
//! # Errors do not allocate across the boundary
//!
//! `open` writes its message into a buffer the **caller** owns. An
//! adapter may be built by another compiler against another allocator,
//! and a string allocated on one side and freed on the other is the
//! classic way for a plugin system to crash in the field. Nothing here
//! frees anything the other side allocated.

#![allow(
    unsafe_code,
    reason = "the plugin boundary of the port: three exported symbols and \
              a caller's error buffer, one SAFETY comment per block"
)]

use core::ffi::{CStr, c_char, c_void};

use crate::ProviderError;
use crate::vtable::ProviderVtable;

/// The version of the three entry points below.
///
/// Bumped when their signatures or their contract change, and **not**
/// when the vtable changes. A loader reads it before calling anything
/// else, because a library built against another version may not have
/// the symbols it is about to look for.
pub const PLUGIN_ABI_VERSION: u32 = 1;

/// The symbol a loader resolves to read [`PLUGIN_ABI_VERSION`].
pub const ABI_VERSION_SYMBOL: &CStr = c"teistro_provider_abi_version";

/// The symbol a loader resolves to open a provider.
pub const OPEN_SYMBOL: &CStr = c"teistro_provider_open";

/// The symbol a loader resolves to close one.
pub const CLOSE_SYMBOL: &CStr = c"teistro_provider_close";

/// Reads the adapter's plugin ABI version.
pub type PluginAbiVersionFn = unsafe extern "C" fn() -> u32;

/// Opens a provider from a configuration.
///
/// `config_json` is the adapter's own options as a JSON object, or null
/// for none — an engine needs a data directory where another needs a
/// host and a key, and a fixed struct would have to know both. It is the
/// *adapter* that types it, in the package a consumer installs.
///
/// On success it writes the vtable and the `user_data` to pass with it,
/// and returns `0`. On failure it returns one of [`crate::ProviderCode`]'s
/// codes, writes nothing to the out-parameters, and writes a NUL
/// terminated message into `error` if that is non-null — never more than
/// `error_capacity` bytes including the terminator.
pub type PluginOpenFn = unsafe extern "C" fn(
    config_json: *const c_char,
    out_vtable: *mut ProviderVtable,
    out_user_data: *mut *mut c_void,
    error: *mut c_char,
    error_capacity: usize,
) -> i32;

/// Closes what [`PluginOpenFn`] handed back. Null is ignored, and a
/// pointer is closed once.
pub type PluginCloseFn = unsafe extern "C" fn(user_data: *mut c_void);

/// Writes a message into a caller's buffer, NUL terminated and truncated
/// to fit, on a character boundary.
///
/// Truncating a UTF-8 message at an arbitrary byte would hand back
/// something that is not a string, so the cut is made where the text
/// allows. A buffer too small for even one character gets the
/// terminator alone.
///
/// # Safety
///
/// `error` must be null, or valid for writes of `capacity` bytes.
pub unsafe fn write_error(message: &str, error: *mut c_char, capacity: usize) {
    if error.is_null() || capacity == 0 {
        return;
    }
    let room = capacity - 1;
    let mut end = message.len().min(room);
    while end > 0 && !message.is_char_boundary(end) {
        end -= 1;
    }
    let Some(bytes) = message.as_bytes().get(..end) else {
        return;
    };
    // SAFETY: the caller promises `capacity` writable bytes, and `end` is
    // at most `capacity - 1`, so the terminator fits.
    unsafe {
        core::ptr::copy_nonoverlapping(bytes.as_ptr().cast::<c_char>(), error, end);
        error.add(end).write(0);
    }
}

/// The configuration a caller passed, as a string.
///
/// Null is an empty configuration rather than an error: an adapter that
/// needs nothing should not require `{}`.
///
/// # Errors
///
/// `INVALID` when the bytes are not UTF-8. A configuration is written by
/// the caller and read by the adapter, so neither end should be guessing
/// at an encoding.
///
/// # Safety
///
/// `config_json` must be null or a NUL-terminated string that stays
/// valid for the call.
pub unsafe fn read_config<'a>(config_json: *const c_char) -> Result<&'a str, ProviderError> {
    if config_json.is_null() {
        return Ok("");
    }
    // SAFETY: the caller promises a NUL-terminated string.
    unsafe { CStr::from_ptr(config_json) }
        .to_str()
        .map_err(|_| ProviderError::invalid("config_json is not UTF-8"))
}

/// Exports a Rust provider as a loadable plugin: the three symbols of
/// this module, with no `unsafe` in the adapter.
///
/// Takes the provider's type and a `fn(&str) -> Result<P, ProviderError>`
/// that builds one from the configuration. The error is a
/// [`ProviderError`] rather than a string so that an adapter picks the
/// code as well as the message — a missing data directory is
/// `DataMissing` and a bad option is `Invalid`, and a loader can act on
/// the difference.
///
/// ```ignore
/// fn open(config: &str) -> Result<MyProvider, ProviderError> { … }
/// teistro_port_ephemeris::export_provider!(MyProvider, open);
/// ```
///
/// The generated `open` catches a panic and turns it into a refusal.
/// Unwinding out of an `extern "C"` function is undefined behaviour, and
/// this is the one boundary in the port whose two sides may have been
/// built by different compilers on different days.
#[macro_export]
macro_rules! export_provider {
    ($provider:ty, $open:path) => {
        /// The plugin ABI this adapter was built against.
        #[unsafe(no_mangle)]
        pub extern "C" fn teistro_provider_abi_version() -> u32 {
            $crate::plugin::PLUGIN_ABI_VERSION
        }

        /// Opens the provider. See [`teistro_port_ephemeris::plugin::PluginOpenFn`].
        ///
        /// # Safety
        ///
        /// As `PluginOpenFn`: every pointer null or valid for the call,
        /// and the out-parameters writable.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn teistro_provider_open(
            config_json: *const ::core::ffi::c_char,
            out_vtable: *mut $crate::vtable::ProviderVtable,
            out_user_data: *mut *mut ::core::ffi::c_void,
            error: *mut ::core::ffi::c_char,
            error_capacity: usize,
        ) -> i32 {
            let opened = ::std::panic::catch_unwind(|| {
                if out_vtable.is_null() || out_user_data.is_null() {
                    return Err($crate::ProviderError::invalid(
                        "out_vtable and out_user_data must not be null",
                    ));
                }
                // SAFETY: the caller's contract.
                let config = unsafe { $crate::plugin::read_config(config_json) }?;
                let provider: $provider = $open(config)?;
                let exported = $crate::vtable::Exported::new(provider);
                let user_data = exported.user_data();
                // The box outlives this call and is reclaimed by `close`.
                ::core::mem::forget(exported);
                Ok((<$crate::vtable::Exported<$provider>>::vtable(), user_data))
            });
            let result = match opened {
                Ok(result) => result,
                Err(_) => Err($crate::ProviderError::Refused {
                    detail: ::std::string::String::from("the adapter panicked while opening"),
                }),
            };
            match result {
                Ok((vtable, user_data)) => {
                    // SAFETY: checked non-null above; the caller promises
                    // they are writable.
                    unsafe {
                        out_vtable.write(vtable);
                        out_user_data.write(user_data);
                    }
                    0
                }
                Err(failure) => {
                    // SAFETY: the caller's contract for the buffer.
                    unsafe {
                        $crate::plugin::write_error(&failure.to_string(), error, error_capacity);
                    }
                    failure.code()
                }
            }
        }

        /// Closes the provider. See [`teistro_port_ephemeris::plugin::PluginCloseFn`].
        ///
        /// # Safety
        ///
        /// `user_data` must be null, or exactly what `teistro_provider_open`
        /// wrote, closed once.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn teistro_provider_close(user_data: *mut ::core::ffi::c_void) {
            if user_data.is_null() {
                return;
            }
            // A panic in a destructor must not cross the boundary either.
            let _ = ::std::panic::catch_unwind(|| {
                // SAFETY: the caller promises the pointer came from
                // `open` and is closed once.
                drop(unsafe {
                    ::std::boxed::Box::from_raw(
                        user_data.cast::<$crate::vtable::Exported<$provider>>(),
                    )
                });
            });
        }
    };
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::cast_possible_wrap,
        reason = "a test fails by panicking, indexes its own fixtures, and \
                  writes ASCII as `c_char`, which is signed here and \
                  unsigned elsewhere and the same either way in that range"
    )]

    use super::{read_config, write_error};

    /// A message is written, terminated, and never past the buffer.
    #[test]
    fn an_error_is_written_and_terminated() {
        let mut buffer = [1_i8; 8];
        // SAFETY: a live buffer of the stated length.
        unsafe { write_error("abc", buffer.as_mut_ptr(), buffer.len()) };
        assert_eq!(&buffer[..4], &[b'a' as i8, b'b' as i8, b'c' as i8, 0]);
        assert_eq!(buffer[4], 1, "nothing past the terminator was touched");
    }

    /// Too long a message is cut to fit, and still terminated.
    #[test]
    fn a_long_message_is_truncated_rather_than_overrun() {
        let mut buffer = [1_i8; 4];
        // SAFETY: a live buffer of the stated length.
        unsafe { write_error("abcdefgh", buffer.as_mut_ptr(), buffer.len()) };
        assert_eq!(&buffer, &[b'a' as i8, b'b' as i8, b'c' as i8, 0]);
    }

    /// The cut is made on a character boundary, because a message
    /// truncated mid-character is not a string and a caller reading it
    /// back as UTF-8 would fail on the adapter's behalf.
    #[test]
    fn truncation_lands_on_a_character_boundary() {
        // "aéb" is four bytes: 'a', then two for 'é', then 'b'.
        // With room for three, the whole of 'é' fits and only 'b' goes.
        let mut fits = [1_i8; 4];
        // SAFETY: a live buffer of the stated length.
        unsafe { write_error("aéb", fits.as_mut_ptr(), fits.len()) };
        assert_eq!(fits, [b'a' as i8, -61, -87, 0], "'é' fits whole");
        // With room for two, half of 'é' would fit and none of it may.
        let mut split = [1_i8; 3];
        // SAFETY: a live buffer of the stated length.
        unsafe { write_error("aéb", split.as_mut_ptr(), split.len()) };
        assert_eq!(
            split,
            [b'a' as i8, 0, 1],
            "the two-byte character was dropped rather than halved"
        );
    }

    /// A null buffer, or no room at all, is ignored rather than a crash:
    /// a caller who does not want the message says so by passing null.
    #[test]
    fn no_buffer_is_not_a_crash() {
        // SAFETY: null is explicitly allowed.
        unsafe { write_error("abc", core::ptr::null_mut(), 0) };
        let mut buffer = [7_i8; 1];
        // SAFETY: a live buffer with room for the terminator alone.
        unsafe { write_error("abc", buffer.as_mut_ptr(), 1) };
        assert_eq!(buffer[0], 0, "room for the terminator and nothing else");
    }

    /// Null configuration is empty configuration, not a refusal: an
    /// adapter that needs nothing should not demand `{}`.
    #[test]
    fn null_configuration_is_empty_configuration() {
        // SAFETY: null is explicitly allowed.
        assert_eq!(unsafe { read_config(core::ptr::null()) }.unwrap(), "");
        let text = c"{\"dataDir\":\"/x\"}";
        // SAFETY: a live NUL-terminated string.
        let read = unsafe { read_config(text.as_ptr()) }.unwrap();
        assert_eq!(read, "{\"dataDir\":\"/x\"}");
    }
}
