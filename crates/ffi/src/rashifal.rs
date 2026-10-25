//! The rashifal at the C boundary: periods of civil days at a place, each
//! read for the twelve signs, answered as canonical JSON
//! (`03-design/rashifal.md`).
//!
//! An answer is a period's nine places, each sign's gochar, Saturn's
//! standing and events, nested records rather than columns, so it crosses
//! as the JSON the façade's envelope of `RashifalAnswer`s serialises to,
//! provenance and all: every binding parses it into its own types.
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
/// answers with `{value, provenance}` as canonical JSON, `value` an array
/// of `{period, baseline}`, one per period: the
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
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::Builtin)?;
/// # fn owned(mut out: teistro_ffi::string::TsString) -> String {
/// #     // SAFETY: the library wrote `len` bytes of UTF-8 at `data`.
/// #     let text = String::from_utf8_lossy(unsafe { core::slice::from_raw_parts(out.data, out.len) }).into_owned();
/// #     // SAFETY: a descriptor the library wrote, freed once.
/// #     unsafe { teistro_ffi::string::ts_string_free(&raw mut out) };
/// #     text
/// # }
/// use teistro_core::Status;
/// use teistro_ffi::string::TsString;
/// use teistro_ffi::rashifal::ts_rashifal;
///
/// let request = c"{\"periods\": [{\"first\": {\"year\": 2026, \"month\": 10, \"day\": 4}, \"last\": {\"year\": 2026, \"month\": 10, \"day\": 10}, \"latitudeDeg\": 27.7172, \"longitudeDeg\": 85.324, \"altitudeM\": 1400, \"utcOffsetSeconds\": 20700}]}";
/// let mut out = TsString::empty();
/// // SAFETY: a live context, a NUL-terminated request and a writable slot.
/// let status = unsafe { ts_rashifal(&raw const ctx, request.as_ptr(), &raw mut out) };
/// assert_eq!(status, Status::Ok);
/// let answer: serde_json::Value = serde_json::from_str(&owned(out))?;
/// // One period asked, one answered, sealed by its provenance.
/// assert!(answer["provenance"]["input_hash"].is_string());
/// assert_eq!(answer["value"].as_array().map(Vec::len), Some(1));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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
            let json = TsString::from_string(teistro_core::envelope::canonical_json(&answers));
            // SAFETY: the entry point's contract.
            unsafe { crate::support::write_plain(out_json, "out_json", json) }
        })
    })
}
