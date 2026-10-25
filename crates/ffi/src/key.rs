//! Catalogue keys and ids at the boundary: a key such as `graha.SUN` packs
//! to a 32-bit id (`kind << 16 | member`) that every struct and column
//! carries, and back.

#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

use core::ffi::c_char;

use teistro_core::Status;
use teistro_core::key::KeyId;

use crate::context::TsContext;
use crate::string::TsStr;
use crate::support::{text, with_context, write_plain};

/// Resolves a full key (`graha.SUN`, an alias, a former key, or a member the
/// context registered, `chart_layout.ACME_KERALA`) to its packed id. An
/// unknown key is `UNSUPPORTED` with the nearest known key as the hint in
/// the context's last error.
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::Test)?;
/// use teistro_core::Status;
/// use teistro_core::catalogue::{Catalogued, Graha};
/// use teistro_ffi::key::ts_key_parse;
///
/// let mut id = 0_u32;
/// // SAFETY: a live context, a NUL-terminated key and a writable slot.
/// assert_eq!(unsafe { ts_key_parse(&raw const ctx, c"graha.SUN".as_ptr(), &raw mut id) }, Status::Ok);
/// assert_eq!(id, Graha::Sun.key_id().bits());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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
    with_context(context, |ctx| {
        // SAFETY: the entry point's contract.
        let key = unsafe { text(key, "key") }?;
        // The façade's lookup, which knows what this context registered.
        let id = ctx.sdk().keys().id(key)?;
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_id, "out_id", id.bits()) }
    })
}

/// The full key of a packed id (`graha.SUN`), lent until the next call on
/// the context. An id no catalogued member has is `UNSUPPORTED`.
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::Test)?;
/// # fn lent(out: teistro_ffi::string::TsStr) -> String {
/// #     // SAFETY: the library lent `len` bytes of UTF-8 at `data`.
/// #     String::from_utf8_lossy(unsafe { core::slice::from_raw_parts(out.data.cast::<u8>(), out.len) }).into_owned()
/// # }
/// # let mut out = teistro_ffi::string::TsStr { data: core::ptr::null(), len: 0 };
/// use teistro_core::Status;
/// use teistro_core::catalogue::{Catalogued, Graha};
/// use teistro_ffi::key::ts_key_name;
///
/// // SAFETY: a live context and a writable slot.
/// assert_eq!(unsafe { ts_key_name(&raw const ctx, Graha::Moon.key_id().bits(), &raw mut out) }, Status::Ok);
/// assert_eq!(lent(out), "graha.MOON");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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
        let full = ctx.sdk().keys().name(KeyId::from_bits(id))?;
        let lent = ctx.lend(&full);
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_key, "out_key", lent) }
    })
}
