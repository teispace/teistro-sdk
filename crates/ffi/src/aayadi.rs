//! Aayadi at the C boundary: a building's two sides read by remainders
//! under one system, answered as canonical JSON (`03-design/aayadi.md`).
//!
//! The reading reads no sky and is one small record, so it crosses as the
//! JSON the kernel serialises to, as the navatara chakra does: every
//! binding parses it into its own types, and the agent server answers the
//! same bytes.
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

/// A building's measure read under one system: `{"system", "quantities",
/// "vyaya", "tara", "clauses", "unjudged"}`, each quantity `{"quantity",
/// "measure", "value", "multiplier", "divisor", "quotient", "remainder",
/// "count", "yoni", "nakshatra", "vara", "tithi", "yoga", "source"}` and
/// each clause `{"clause", "holds", "source"}` (C1190 to C1199).
///
/// `request_json` is `{"system", "length", "breadth", "owner"?,
/// "measure"?}`: `MAYAMATA`, `MANASARA`, `SAMARANGANA` or `CHINTAMANI`,
/// the two sides in whole units (1 to 1 000 000), the owner's nakshatra
/// key, and for `MANASARA` only the measure. A key the record does not
/// read, a side out of range or a measure asked of another system is
/// `INVALID_ARG`, named under `aayadi`.
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
/// use teistro_ffi::aayadi::ts_aayadi;
/// use teistro_ffi::string::TsString;
///
/// let request = c"{\"system\": \"CHINTAMANI\", \"length\": 29, \"breadth\": 7}";
/// let mut out = TsString::empty();
/// // SAFETY: a live context, a NUL-terminated request and a writable slot.
/// let status = unsafe { ts_aayadi(&raw const ctx, request.as_ptr(), &raw mut out) };
/// assert_eq!(status, Status::Ok);
/// let answer: serde_json::Value = serde_json::from_str(&owned(out))?;
/// // The area 203: the lion and Rohini.
/// assert_eq!((&answer["quantities"][0]["yoni"], &answer["quantities"][5]["nakshatra"]), (&serde_json::json!("SIMHA"), &serde_json::json!("ROHINI")));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `request_json` NUL-terminated;
/// `out_json` valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_aayadi(
    context: *const TsContext,
    request_json: *const c_char,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |_| {
        in_family!("muhurta", [request_json, out_json], {
            // SAFETY: the entry point's contract.
            let text = unsafe { crate::support::text(request_json, "request_json") }?;
            let reading = teistro::AayadiRequest::from_json(text)?.answer()?;
            let json = TsString::from_string(teistro_core::envelope::canonical_json(&reading));
            // SAFETY: the entry point's contract.
            unsafe { crate::support::write_plain(out_json, "out_json", json) }
        })
    })
}
