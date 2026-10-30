//! Published sets at the C boundary: a signed pack verified against the
//! publisher's key and answered as the set's canonical JSON, which a
//! binding hands back in a festival request's `published`
//! (`03-design/published-overrides.md` §5).
#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

use teistro_core::error::Status;

use crate::context::TsContext;
use crate::family::in_family;
use crate::string::TsString;
use crate::support::with_context;

/// Verifies a signed pack and answers with its published set as canonical
/// JSON. `pack` is the set's exact bytes, `signature` the publisher's
/// detached Ed25519 signature over them (64 bytes) and `key` the
/// publisher's public key (32 bytes), which the caller fixes at its own
/// build: the library embeds none. A set whose `version` is under
/// `at_least`, the newest the caller has applied, is refused, so a replayed
/// pack cannot roll a correction back; 0 takes any.
///
/// A key that is no curve point is `INVALID_ARG` on `key`; a signature
/// that is not 64 bytes or does not verify, `INVALID_ARG` on `signature`;
/// bytes that are not a set, `INVALID_ARG` on the field that is wrong; an
/// older version, `INVALID_ARG` on `version`. A build without
/// `signed-packs` answers `CAPABILITY`.
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::None)?;
/// use teistro_core::Status;
/// use teistro_ffi::published::ts_published_verify;
/// use teistro_ffi::string::TsString;
///
/// // A key with no curve point is refused, and nothing is read; a build
/// // without `signed-packs` refuses the call itself.
/// let (pack, signature) = (b"{}", [0_u8; 64]);
/// let mut key = [0_u8; 32];
/// key[0] = 2;
/// let mut out = TsString::empty();
/// // SAFETY: a live context, readable bytes and a writable slot.
/// let status = unsafe {
///     ts_published_verify(
///         &raw const ctx,
///         pack.as_ptr(), pack.len(),
///         signature.as_ptr(), signature.len(),
///         key.as_ptr(), key.len(),
///         0,
///         &raw mut out,
///     )
/// };
/// assert!(matches!(status, Status::InvalidArg | Status::Capability));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `pack`, `signature` and `key` valid
/// for their lengths' reads; `out_json` valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
#[allow(
    clippy::too_many_arguments,
    reason = "three byte ranges cross the C boundary as pointer and length each"
)]
pub unsafe extern "C" fn ts_published_verify(
    context: *const TsContext,
    pack: *const u8,
    pack_len: usize,
    signature: *const u8,
    signature_len: usize,
    key: *const u8,
    key_len: usize,
    at_least: u32,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |_| {
        in_family!(
            "signed-packs",
            [
                pack,
                pack_len,
                signature,
                signature_len,
                key,
                key_len,
                at_least,
                out_json
            ],
            {
                // SAFETY: the entry point's contract.
                let pack = unsafe { crate::support::bytes(pack, pack_len, "pack") }?;
                // SAFETY: the entry point's contract.
                let signature =
                    unsafe { crate::support::bytes(signature, signature_len, "signature") }?;
                // SAFETY: the entry point's contract.
                let key = unsafe { crate::support::bytes(key, key_len, "key") }?;
                let set =
                    teistro::festival::PublishedSet::from_signed(pack, signature, key, at_least)?;
                let json = TsString::from_string(teistro_core::envelope::canonical_json(&set));
                // SAFETY: the entry point's contract.
                unsafe { crate::support::write_plain(out_json, "out_json", json) }
            }
        )
    })
}
