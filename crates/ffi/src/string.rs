//! Strings across the boundary: [`TsString`], which the library allocates
//! and the caller frees with `ts_string_free`; [`TsStr`], which the
//! library lends until the next call on the context; and [`TsHash`], a
//! SHA-256 as bytes.

#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

use core::ffi::c_char;
use core::mem::ManuallyDrop;
use core::ptr;

use teistro_core::envelope::Hash;

/// A string the library allocated: UTF-8, NUL-terminated at `data[len]`,
/// `len` bytes before the NUL, freed by `ts_string_free`.
///
/// `api: role=owned_string`
#[repr(C)]
#[derive(Debug)]
pub struct TsString {
    /// The bytes; null when empty.
    pub data: *mut u8,
    /// The number of bytes before the NUL.
    pub len: usize,
    /// The allocation's capacity, needed to free it.
    pub cap: usize,
}

impl TsString {
    /// The empty string, which frees as a no-op.
    #[must_use]
    pub const fn empty() -> TsString {
        TsString {
            data: ptr::null_mut(),
            len: 0,
            cap: 0,
        }
    }

    /// Hands a Rust string to the caller.
    #[must_use]
    pub(crate) fn from_string(text: String) -> TsString {
        let mut bytes = text.into_bytes();
        bytes.push(0);
        let mut bytes = ManuallyDrop::new(bytes);
        TsString {
            data: bytes.as_mut_ptr(),
            len: bytes.len() - 1,
            cap: bytes.capacity(),
        }
    }
}

/// A string the library lends: UTF-8, NUL-terminated, `len` bytes before
/// the NUL, valid until the next call on the same context.
///
/// `api: role=borrowed_string`
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TsStr {
    /// The bytes; never null once written.
    pub data: *const c_char,
    /// The number of bytes before the NUL.
    pub len: usize,
}

/// A SHA-256 as thirty-two bytes; every hash the SDK reports in
/// provenance is one, rendered as sixty-four hex digits in JSON.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TsHash {
    /// The digest.
    pub bytes: [u8; 32],
}

impl From<Hash> for TsHash {
    fn from(hash: Hash) -> TsHash {
        TsHash {
            bytes: *hash.bytes(),
        }
    }
}

/// Frees a string the library allocated and zeroes the descriptor; null
/// or an empty string is ignored.
///
/// # Safety
///
/// `string` must be null or a descriptor written by this library that is
/// not used again.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_string_free(string: *mut TsString) {
    if string.is_null() {
        return;
    }
    // SAFETY: non-null; the caller promises a descriptor this library
    // wrote, replaced here by the empty one so a second free is a no-op.
    let descriptor = unsafe { ptr::replace(string, TsString::empty()) };
    if descriptor.data.is_null() {
        return;
    }
    // SAFETY: the three fields are those of the Vec `from_string` leaked,
    // whose length counted the NUL.
    drop(unsafe { Vec::from_raw_parts(descriptor.data, descriptor.len + 1, descriptor.cap) });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_owned_string_is_nul_terminated_and_frees_once() {
        let mut string = TsString::from_string(String::from("नमस्ते"));
        assert_eq!(string.len, "नमस्ते".len());
        // SAFETY: `len + 1` bytes were allocated.
        let terminated = unsafe { core::slice::from_raw_parts(string.data, string.len + 1) };
        assert_eq!(terminated.last(), Some(&0));
        // SAFETY: a descriptor this module wrote; freed twice on purpose.
        unsafe {
            ts_string_free(&raw mut string);
            assert!(string.data.is_null());
            ts_string_free(&raw mut string);
            ts_string_free(ptr::null_mut());
        }
        let hash: TsHash = Hash::of(b"x").into();
        assert_eq!(hash.bytes.len(), 32);
    }
}
