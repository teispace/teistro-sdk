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
/// ```
/// # #[cfg(feature = "builtin-ephemeris")] {
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
/// use teistro_ffi::pakshi::ts_pakshi;
///
/// let request = c"{\"first\": {\"year\": 1984, \"month\": 10, \"day\": 30}, \"last\": {\"year\": 1984, \"month\": 11, \"day\": 1}, \"latitudeDeg\": 13.0827, \"longitudeDeg\": 80.2707, \"altitudeM\": 6, \"utcOffsetSeconds\": 19800, \"native\": {\"nakshatra\": \"nakshatra.UTTARA_ASHADHA\", \"paksha\": \"paksha.SHUKLA\"}}";
/// let mut out = TsString::empty();
/// // SAFETY: a live context, a NUL-terminated request and a writable slot.
/// let status = unsafe { ts_pakshi(&raw const ctx, request.as_ptr(), &raw mut out) };
/// assert_eq!(status, Status::Ok);
/// let answer: serde_json::Value = serde_json::from_str(&owned(out))?;
/// // Three days of the native's bird, one entry each.
/// assert_eq!(answer["value"].as_array().map(Vec::len), Some(3));
/// # }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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
