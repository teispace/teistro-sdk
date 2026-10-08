//! The rashifal at the C boundary: periods of civil days at a place, each
//! read for the twelve signs, answered as canonical JSON
//! (`03-design/rashifal.md`).
//!
//! An answer is a period's nine places, each sign's gochar, Saturn's
//! standing and events, nested records rather than columns, so it crosses
//! as the JSON the façade's `RashifalAnswer` serialises to: every binding
//! parses it into its own types.
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

/// Reads periods of civil days at a place for each of the twelve signs and
/// answers with an array of `{period, baseline}` as canonical JSON: the
/// sky at the reference day's sunrise (or a clock time), each sign's
/// gochar from Phaladeepika ch. 26, Saturn's standing, and every ingress
/// and station of the period counted from each sign; `baseline` the
/// baseline engine's score of each sign when the request names a period
/// for it, absent otherwise.
///
/// `request_json` is `{"periods", "baseline"}`: each period `{"calendar",
/// "first", "last", "latitudeDeg", "longitudeDeg", "altitudeM",
/// "utcOffsetSeconds", "snapshot", "events", "spells"}`, the days as
/// `{"year", "month", "day"}` and everything but `first`, the place and
/// the offset optional; `baseline` one of `DAILY`, `WEEKLY`, `MONTHLY`
/// and `YEARLY`. A key it does not read, a last day before the first, a
/// clock off the clock or a graha named twice is `INVALID_ARG`, named
/// under `rashifal`. A context without an ephemeris is `CAPABILITY`, as is
/// a build that leaves the `rashifal` family out.
///
/// # Safety
///
/// `context` must be a live handle; `request_json` NUL-terminated;
/// `out_json` valid for a write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_rashifal(
    context: *const TsContext,
    request_json: *const c_char,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |ctx| {
        in_family!("rashifal", [ctx, request_json, out_json], {
            // SAFETY: the entry point's contract.
            let asked = teistro::RashifalBatch::from_json(unsafe {
                crate::support::text(request_json, "request_json")
            }?)?;
            let answers = ctx.sdk().chart().rashifal_answers(&asked)?;
            let json =
                TsString::from_string(teistro_core::envelope::canonical_json(&answers.value));
            // SAFETY: the entry point's contract.
            unsafe { crate::support::write_plain(out_json, "out_json", json) }
        })
    })
}
