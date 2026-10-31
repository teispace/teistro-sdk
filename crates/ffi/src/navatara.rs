//! The navatara chakra at the C boundary: a birth star's 27 stars, each
//! with its count, tara, round and special star, answered as canonical
//! JSON (`03-design/navatara-chakra.md`).
//!
//! The chakra reads no sky and is one short list, so it crosses as the
//! JSON the façade's rows serialise to, as numerology's profile does:
//! every binding parses it into its own types, and the agent server
//! answers the same bytes.
#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

use core::ffi::c_char;

use teistro_core::error::Status;

use crate::context::TsContext;
use crate::family::in_family;
use crate::string::TsString;
use crate::support::with_context;

/// The navatara chakra of a birth star: the 27 stars in the order the
/// count from the birth star reaches them, each `{"nakshatra", "count",
/// "tara", "cycle", "special"}`, `cycle` the round of nine (1 janma, 2
/// anujanma, 3 trijanma) and `special` the star *Muhurta Chintamani*
/// names at that count (`JANMA`, `KARMA`, `SANGHATIKA`, `SAMUDAYA`,
/// `VAINASHIKA`, `MANASA`) or null (C1053).
///
/// `request_json` is `{"birth"}`, a nakshatra key. A key the record does
/// not read, or a star that is not a nakshatra key, is `INVALID_ARG`,
/// named under `navatara`.
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::None)?;
/// # fn owned(mut out: teistro_ffi::string::TsString) -> String {
/// #     // SAFETY: the library wrote `len` bytes of UTF-8 at `data`.
/// #     let text = String::from_utf8_lossy(unsafe { core::slice::from_raw_parts(out.data, out.len) }).into_owned();
/// #     // SAFETY: a descriptor the library wrote, freed once.
/// #     unsafe { teistro_ffi::string::ts_string_free(&raw mut out) };
/// #     text
/// # }
/// use teistro_core::Status;
/// use teistro_ffi::navatara::ts_navatara;
/// use teistro_ffi::string::TsString;
///
/// let request = c"{\"birth\": \"ASHWINI\"}";
/// let mut out = TsString::empty();
/// // SAFETY: a live context, a NUL-terminated request and a writable slot.
/// let status = unsafe { ts_navatara(&raw const ctx, request.as_ptr(), &raw mut out) };
/// assert_eq!(status, Status::Ok);
/// let answer: serde_json::Value = serde_json::from_str(&owned(out))?;
/// // Born in Ashwini, Magha is the 10th: karma.
/// assert_eq!((&answer[9]["nakshatra"], &answer[9]["special"]), (&serde_json::json!("MAGHA"), &serde_json::json!("KARMA")));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `request_json` NUL-terminated;
/// `out_json` valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_navatara(
    context: *const TsContext,
    request_json: *const c_char,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |_| {
        in_family!("muhurta", [request_json, out_json], {
            // SAFETY: the entry point's contract.
            let text = unsafe { crate::support::text(request_json, "request_json") }?;
            let chakra = teistro::NavataraRequest::from_json(text)?.answer();
            let json = TsString::from_string(teistro_core::envelope::canonical_json(&chakra));
            // SAFETY: the entry point's contract.
            unsafe { crate::support::write_plain(out_json, "out_json", json) }
        })
    })
}
