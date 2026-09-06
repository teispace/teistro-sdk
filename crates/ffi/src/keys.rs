//! Catalogue keys and ids at the boundary: a key such as `graha.SUN` packs
//! to a 32-bit id (`kind << 16 | member`) that every struct and column
//! carries, and back.

#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

use core::ffi::c_char;

use teistro_core::Status;
use teistro_core::error::{Detail, Error};
use teistro_core::key::{KeyId, resolve};

use crate::context::TsContext;
use crate::strings::TsStr;
use crate::support::{text, with_context, write_plain};

/// Resolves a full key (`graha.SUN`, an alias, or a former key) to its
/// packed id. An unknown key is `UNSUPPORTED` with the nearest known key as
/// the hint in the context's last error.
///
/// # Safety
///
/// `context` must be a live handle; `key` a NUL-terminated string; `out_id`
/// valid for a write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_key_parse(
    context: *const TsContext,
    key: *const c_char,
    out_id: *mut u32,
) -> Status {
    with_context(context, |_| {
        // SAFETY: the entry point's contract.
        let key = unsafe { text(key, "key") }?;
        let id = resolve(key)?;
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_id, "out_id", id.bits()) }
    })
}

/// The full key of a packed id (`graha.SUN`), lent until the next call on
/// the context. An id no catalogued member has is `UNSUPPORTED`.
///
/// # Safety
///
/// `context` must be a live handle; `out_key` valid for a write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_key_name(
    context: *const TsContext,
    id: u32,
    out_key: *mut TsStr,
) -> Status {
    with_context(context, |ctx| {
        let key_id = KeyId::from_bits(id);
        let (Some(kind), Some(key)) = (key_id.kind(), key_id.key()) else {
            return Err(
                Error::unsupported(format!("no catalogued member has id {id:#010x}"))
                    .with_detail(Detail::UnknownKey)
                    .with_field("id"),
            );
        };
        let full = format!("{}.{key}", kind.name());
        let lent = ctx.lend(&full);
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_key, "out_key", lent) }
    })
}
