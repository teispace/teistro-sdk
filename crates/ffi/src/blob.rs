//! The result blob descriptor: bytes the library allocated in the `TSRB`
//! layout ([`teistro_idl::blob`]), owned by the library until
//! `ts_blob_free`.

#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

use core::mem::ManuallyDrop;
use core::ptr;

/// A result blob the library allocated; its layout is the `TSRB` encoding
/// under the schema the entry point names, both in `idl/api.json`.
///
/// `api: role=blob`
#[repr(C)]
#[derive(Debug)]
pub struct TsBlob {
    /// The bytes, 8-aligned; null when empty.
    pub data: *mut u8,
    /// The number of bytes.
    pub len: usize,
    /// The allocation's capacity, needed to free it.
    pub cap: usize,
}

impl TsBlob {
    /// The empty blob, which frees as a no-op.
    #[must_use]
    pub const fn empty() -> TsBlob {
        TsBlob {
            data: ptr::null_mut(),
            len: 0,
            cap: 0,
        }
    }

    /// Hands encoded bytes to the caller. A `Vec<u8>` is at least
    /// 8-aligned on every allocator the SDK builds with; the decoders
    /// re-align a buffer that is not, so a view is never misaligned.
    #[must_use]
    pub(crate) fn from_vec(bytes: Vec<u8>) -> TsBlob {
        let mut bytes = ManuallyDrop::new(bytes);
        TsBlob {
            data: bytes.as_mut_ptr(),
            len: bytes.len(),
            cap: bytes.capacity(),
        }
    }
}

/// Frees a blob the library allocated and zeroes the descriptor; null or
/// an empty blob is ignored.
///
/// # Safety
///
/// `blob` must be null or a descriptor written by this library that is not
/// used again.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_blob_free(blob: *mut TsBlob) {
    if blob.is_null() {
        return;
    }
    // SAFETY: non-null; the caller promises a descriptor this library
    // wrote, replaced here by the empty one so a second free is a no-op.
    let descriptor = unsafe { ptr::replace(blob, TsBlob::empty()) };
    if descriptor.data.is_null() {
        return;
    }
    // SAFETY: the three fields are those of the Vec `from_vec` leaked.
    drop(unsafe { Vec::from_raw_parts(descriptor.data, descriptor.len, descriptor.cap) });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_blob_frees_once_and_null_is_ignored() {
        let mut blob = TsBlob::from_vec(vec![1, 2, 3]);
        assert_eq!(blob.len, 3);
        // SAFETY: a descriptor this module wrote; freed twice on purpose.
        unsafe {
            ts_blob_free(&raw mut blob);
            assert!(blob.data.is_null());
            ts_blob_free(&raw mut blob);
            ts_blob_free(ptr::null_mut());
        }
    }
}
