//! Numerology at the C boundary: a name and a birth date read under
//! Balliett's letter cycle and Cheiro's Chaldean table, answered as
//! canonical JSON (`03-design/numerology.md`, C320 to C328).
//!
//! The answer is one document per call, of nested records and lists of
//! words, so it crosses as the JSON the façade's `Profile` serialises to
//! rather than as a blob: every binding parses it into its own types.
#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

use core::ffi::c_char;

use teistro_core::error::Status;

use crate::context::TsContext;
use crate::string::TsString;
use crate::support::{text, with_context, write_plain};

/// Reads a name and a birth date under numerology's two systems and
/// answers with the profile as canonical JSON: the name under each
/// system, word by word with every reduction step, Balliett's birth
/// number, Cheiro's day and year, and the baseline engine's own numbers
/// under the baseline rules only (`03-design/numerology.md`).
///
/// `request_json` is `{"name", "date", "rules"}`: the name in the 26
/// Latin letters, the date as `{"year", "month", "day"}` in the Gregorian
/// calendar, and the `NumerologyRules` with every field optional. A
/// character outside A to Z while `rules.nonLatin` refuses, a name with
/// no letter, or a date the calendar does not have is `INVALID_ARG`,
/// named under `numerology`.
///
/// # Safety
///
/// `context` must be a live handle; `request_json` NUL-terminated;
/// `out_json` valid for a write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_numerology_profile(
    context: *const TsContext,
    request_json: *const c_char,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |_| {
        // SAFETY: the entry point's contract.
        let asked =
            teistro::NumerologyRequest::from_json(unsafe { text(request_json, "request_json") }?)?;
        let json = TsString::from_string(teistro_core::envelope::canonical_json(&asked.answer()?));
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_json, "out_json", json) }
    })
}
