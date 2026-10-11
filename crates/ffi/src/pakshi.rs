//! Pancha Pakshi at the C boundary: days of a native's bird at a place,
//! answered as canonical JSON (`03-design/pakshi.md`).
//!
//! A day's answer is its ten yamas, each with its sub-periods, nested
//! records rather than columns, so it crosses as the JSON the façade's
//! envelope of `PakshiDay`s serialises to, provenance and all: every
//! binding parses it into its own types.
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

/// Reads a native's bird over each civil day of a range at a place and
/// answers with `{value, provenance}` as canonical JSON, `value` an array
/// of `{date, reading}`, one per day: `reading` the day's ten yamas from the almanac's sunrise, sunset
/// and next sunrise, each `{half, yama, span, activity, quality, subs}`
/// with every sub-period's activity, owner, span and how the native
/// regards its owner, beside the day's `{sunrise, sunset, nextSunrise,
/// vara, paksha}`, the bird, its death bird and the first eaters; null on
/// a day the Sun does not both rise and set.
///
/// `request_json` is `{"calendar", "first", "last", "latitudeDeg",
/// "longitudeDeg", "altitudeM", "utcOffsetSeconds", "native", "rules"}`:
/// the days as `{"year", "month", "day"}`, `native` either `{"bird"}` or
/// `{"nakshatra", "paksha", "rule"}`, and everything but `first`, the
/// place, the offset and `native` optional. A key it does not read, a
/// place or offset out of range or a native that is neither is
/// `INVALID_ARG`, named under `pakshi`, as `pakshi.native.bird`. A context
/// without an ephemeris is `CAPABILITY`, as is a build that leaves the
/// `pakshi` family out.
///
/// # Safety
///
/// `context` must be a live handle; `request_json` NUL-terminated;
/// `out_json` valid for a write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_pakshi(
    context: *const TsContext,
    request_json: *const c_char,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |ctx| {
        in_family!("pakshi", [ctx, request_json, out_json], {
            // SAFETY: the entry point's contract.
            let asked = teistro::PakshiRequest::from_json(unsafe {
                crate::support::text(request_json, "request_json")
            }?)?;
            let days = ctx.sdk().almanac().pakshi_request(&asked)?;
            let json = TsString::from_string(teistro_core::envelope::canonical_json(&days));
            // SAFETY: the entry point's contract.
            unsafe { crate::support::write_plain(out_json, "out_json", json) }
        })
    })
}
