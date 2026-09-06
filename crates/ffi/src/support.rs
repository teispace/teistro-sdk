//! What every entry point shares: the size handshake on every boundary
//! struct, reading and writing through caller pointers, C strings in and
//! out, and the panic guard that turns a caught panic into `INTERNAL` and
//! records every outcome on the context.

#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

use core::ffi::{CStr, c_char};
use std::panic::{AssertUnwindSafe, catch_unwind};

use teistro_core::error::{Error, Status};

use crate::context::TsContext;

/// A boundary struct that begins with `struct_size`.
pub(crate) trait CStruct: Sized {
    /// The size the caller set.
    fn struct_size(&self) -> u32;
    /// Sets the size.
    fn set_struct_size(&mut self, size: u32);
}

/// Implements [`CStruct`] for structs whose first field is `struct_size`.
macro_rules! c_struct {
    ($($ty:ty),* $(,)?) => {
        $(
            impl $crate::support::CStruct for $ty {
                fn struct_size(&self) -> u32 {
                    self.struct_size
                }

                fn set_struct_size(&mut self, size: u32) {
                    self.struct_size = size;
                }
            }
        )*
    };
}
pub(crate) use c_struct;

/// `sizeof(T)` as the handshake field carries it.
#[allow(
    clippy::cast_possible_truncation,
    reason = "guarded by the comparison above it"
)]
pub(crate) const fn size_of_u32<T>() -> u32 {
    let size = core::mem::size_of::<T>();
    if size > u32::MAX as usize {
        u32::MAX
    } else {
        size as u32
    }
}

/// The short name of a type, for messages.
fn short_name<T>() -> &'static str {
    core::any::type_name::<T>()
        .rsplit("::")
        .next()
        .unwrap_or("struct")
}

/// `INVALID_ARG` for a null pointer.
pub(crate) fn null(name: &str) -> Error {
    Error::invalid_arg(format!("`{name}` is null")).with_field(name)
}

/// `SCHEMA_VERSION` when the caller's `struct_size` is not this build's.
pub(crate) fn check_size<T: CStruct>(size: u32) -> Result<(), Error> {
    let expected = size_of_u32::<T>();
    if size == expected {
        Ok(())
    } else {
        Err(Error::new(
            Status::SchemaVersion,
            format!(
                "{} is {size} bytes to the caller and {expected} to this library; set struct_size to sizeof and rebuild against this header",
                short_name::<T>()
            ),
        )
        .with_field("struct_size"))
    }
}

/// A boundary struct the caller passed in, checked for null and size.
///
/// # Safety
///
/// `ptr` must be null or valid for reads of `T` for the returned lifetime.
pub(crate) unsafe fn read_in<'a, T: CStruct>(ptr: *const T, name: &str) -> Result<&'a T, Error> {
    if ptr.is_null() {
        return Err(null(name));
    }
    // SAFETY: non-null; the caller promises a readable `T`.
    let value = unsafe { &*ptr };
    check_size::<T>(value.struct_size())?;
    Ok(value)
}

/// Writes a boundary struct the caller receives: its `struct_size` must
/// already be set, and is checked before anything is written.
///
/// # Safety
///
/// `ptr` must be null or valid for reads and writes of `T`; every field
/// of `T` must be a plain integer, float, pointer or array of them, so
/// that any bytes the caller left there are a valid `T` to read the size
/// from.
pub(crate) unsafe fn write_out<T: CStruct>(
    ptr: *mut T,
    name: &str,
    mut value: T,
) -> Result<(), Error> {
    if ptr.is_null() {
        return Err(null(name));
    }
    // SAFETY: non-null; the caller promises a `T` whose `struct_size` is set.
    let size = unsafe { (*ptr).struct_size() };
    check_size::<T>(size)?;
    value.set_struct_size(size);
    // SAFETY: as above, and writable.
    unsafe { ptr.write(value) };
    Ok(())
}

/// Writes a plain value (a scalar, a fixed-size struct without the
/// handshake) the caller receives.
///
/// # Safety
///
/// `ptr` must be null or valid for writes of `T`.
pub(crate) unsafe fn write_plain<T>(ptr: *mut T, name: &str, value: T) -> Result<(), Error> {
    if ptr.is_null() {
        return Err(null(name));
    }
    // SAFETY: non-null; the caller promises a writable `T`.
    unsafe { ptr.write(value) };
    Ok(())
}

/// A required NUL-terminated UTF-8 string.
///
/// # Safety
///
/// `ptr` must be null or point at a NUL-terminated string that stays
/// valid for the returned lifetime.
pub(crate) unsafe fn text<'a>(ptr: *const c_char, name: &str) -> Result<&'a str, Error> {
    // SAFETY: as documented.
    unsafe { optional_text(ptr, name) }?.ok_or_else(|| null(name))
}

/// An optional NUL-terminated UTF-8 string.
///
/// # Safety
///
/// As [`text`].
pub(crate) unsafe fn optional_text<'a>(
    ptr: *const c_char,
    name: &str,
) -> Result<Option<&'a str>, Error> {
    if ptr.is_null() {
        return Ok(None);
    }
    // SAFETY: non-null; the caller promises a NUL-terminated string.
    let raw = unsafe { CStr::from_ptr(ptr) };
    raw.to_str()
        .map(Some)
        .map_err(|e| Error::invalid_arg(format!("`{name}` is not UTF-8: {e}")).with_field(name))
}

/// A byte buffer the caller passed in; a null pointer with a zero length
/// is the empty buffer.
///
/// # Safety
///
/// `ptr` must be null or valid for `len` reads for the returned lifetime.
pub(crate) unsafe fn bytes<'a>(ptr: *const u8, len: usize, name: &str) -> Result<&'a [u8], Error> {
    if len == 0 {
        return Ok(&[]);
    }
    if ptr.is_null() {
        return Err(null(name));
    }
    // SAFETY: non-null; the caller promises `len` readable bytes.
    Ok(unsafe { core::slice::from_raw_parts(ptr, len) })
}

/// The context behind a handle.
///
/// # Safety
///
/// `ptr` must be null or a live handle from `ts_context_new`.
pub(crate) unsafe fn context<'a>(ptr: *const TsContext) -> Option<&'a TsContext> {
    if ptr.is_null() {
        None
    } else {
        // SAFETY: non-null; the caller promises a live handle.
        Some(unsafe { &*ptr })
    }
}

/// Runs an entry point's body: a panic becomes `INTERNAL`, and the outcome
/// is recorded on the context for `ts_context_last_error`. The context's
/// lent strings are released at the start, so a pointer a previous call
/// handed out is valid exactly until the next call.
pub(crate) fn guarded(
    context: Option<&TsContext>,
    body: impl FnOnce() -> Result<(), Error>,
) -> Status {
    if let Some(ctx) = context {
        ctx.begin_call();
    }
    let outcome = catch_unwind(AssertUnwindSafe(body)).unwrap_or_else(|payload| {
        let detail = payload
            .downcast_ref::<&str>()
            .map(|s| (*s).to_string())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| String::from("a panic without a message"));
        Err(Error::internal(format!(
            "a panic was caught at the boundary: {detail}"
        )))
    });
    let status = outcome
        .as_ref()
        .map_or_else(|error| error.status, |()| Status::Ok);
    if let Some(ctx) = context {
        ctx.record(outcome.err());
    }
    status
}

/// An entry point over a context: null handle, then the guarded body.
pub(crate) fn with_context(
    ptr: *const TsContext,
    body: impl FnOnce(&TsContext) -> Result<(), Error>,
) -> Status {
    // SAFETY: the entry points' contract: null or a live handle.
    let Some(ctx) = (unsafe { context(ptr) }) else {
        return Status::InvalidArg;
    };
    guarded(Some(ctx), || body(ctx))
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        reason = "tests fail by panicking, and one panics on purpose"
    )]

    use super::*;

    #[repr(C)]
    #[derive(Debug)]
    struct Probe {
        struct_size: u32,
        value: u32,
    }
    c_struct!(Probe);

    #[test]
    fn the_handshake_and_the_pointers_are_checked() {
        let mut probe = Probe {
            struct_size: 3,
            value: 0,
        };
        // SAFETY: a valid struct.
        let refused = unsafe { read_in(&raw const probe, "probe") }.unwrap_err();
        assert_eq!(refused.status, Status::SchemaVersion);
        assert!(refused.message.contains("Probe is 3 bytes"));
        // SAFETY: null is allowed.
        assert_eq!(
            unsafe { read_in::<Probe>(core::ptr::null(), "probe") }
                .unwrap_err()
                .status,
            Status::InvalidArg
        );
        probe.struct_size = size_of_u32::<Probe>();
        // SAFETY: a valid, writable struct with its size set.
        unsafe {
            write_out(
                &raw mut probe,
                "probe",
                Probe {
                    struct_size: 0,
                    value: 9,
                },
            )
        }
        .unwrap();
        assert_eq!((probe.struct_size, probe.value), (8, 9));
        let mut slot = 0u64;
        // SAFETY: a valid slot.
        unsafe { write_plain(&raw mut slot, "slot", 5u64) }.unwrap();
        assert_eq!(slot, 5);
        // SAFETY: a NUL-terminated literal, and null.
        unsafe {
            assert_eq!(text(c"namaste".as_ptr(), "s").unwrap(), "namaste");
            assert_eq!(optional_text(core::ptr::null(), "s").unwrap(), None);
            assert_eq!(text(core::ptr::null(), "s").unwrap_err().field(), Some("s"));
            assert_eq!(bytes(core::ptr::null(), 0, "b").unwrap(), &[] as &[u8]);
            assert_eq!(
                bytes(core::ptr::null(), 3, "b").unwrap_err().status,
                Status::InvalidArg
            );
            assert_eq!(bytes([1u8, 2].as_ptr(), 2, "b").unwrap(), &[1, 2]);
        }
        let bad = [0xffu8, 0];
        // SAFETY: a NUL-terminated buffer.
        let error = unsafe { text(bad.as_ptr().cast(), "s") }.unwrap_err();
        assert!(error.message.contains("not UTF-8"));
    }

    #[test]
    fn a_panic_becomes_internal_and_a_result_its_status() {
        assert_eq!(guarded(None, || Ok(())), Status::Ok);
        assert_eq!(
            guarded(None, || Err(Error::new(Status::Limit, "too many"))),
            Status::Limit
        );
        assert_eq!(guarded(None, || panic!("boom")), Status::Internal);
        assert_eq!(
            with_context(core::ptr::null(), |_| Ok(())),
            Status::InvalidArg
        );
    }
}
