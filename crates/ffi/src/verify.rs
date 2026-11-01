//! Claims checked at the C boundary: what a text claims of the time,
//! judged against the sankalpa the SDK computes at an instant and a place
//! (`03-design/lexicon.md` §3), answered as canonical JSON.
#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

use core::ffi::c_char;

use teistro_core::error::Status;

use crate::context::TsContext;
use crate::string::TsString;
use crate::support::{text, with_context, write_plain};

/// Every claim a text makes of the time, each judged against what
/// `almanac.sankalpa` computes at the instant and the place
/// (`03-design/lexicon.md` §3), answered with `{value, provenance}` as
/// canonical JSON.
///
/// `value` is `{claims}`, each claim `{start, end, text, kind, graha,
/// claimed, verdict, computed, reason}` in the text's order: the span
/// naming the member (counted in Unicode scalar values), the kind claimed,
/// the graha whose sign or nakshatra it is, every key the span may name,
/// `HELD`, `CONTRADICTED` or `UNREAD`, the key computed on a claim judged,
/// and on one unread why: `QUESTION`, `NEGATED`, `OTHER_TIME`,
/// `NOT_COMPUTED` or `AMBIGUOUS`.
///
/// `request_json` is `{"text", "at", "latitudeDeg", "longitudeDeg",
/// "altitudeM", "utcOffsetSeconds", "ayana", "locale"}`: `at` a UTC Julian
/// day, `altitudeM` 0 and `ayana` `SIDEREAL` when left out, and `locale`
/// whose names are searched, every loaded locale's when left out. A key it
/// does not read or an instant, place or offset out of range is
/// `INVALID_ARG`, named under `claims`; a locale no loaded one answers is
/// `UNSUPPORTED`, named `claims.locale`; a context without an ephemeris is
/// `CAPABILITY`.
///
/// ```
/// # #[cfg(feature = "builtin-ephemeris")] {
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts { profile: Some("nepali-drik"), ..OptionTexts::default() }, teistro::Ephemeris::Builtin)?;
/// use teistro_core::Status;
/// use teistro_ffi::string::{TsString, ts_string_free};
/// use teistro_ffi::verify::ts_verify_claims;
///
/// let request = c"{\"text\": \"The Sun is in Capricorn.\", \"at\": 2461055.59375, \"latitudeDeg\": 27.7172, \"longitudeDeg\": 85.324, \"utcOffsetSeconds\": 20700}";
/// let mut out = TsString::empty();
/// // SAFETY: a live context, a NUL-terminated request and a writable slot.
/// let status = unsafe { ts_verify_claims(&raw const ctx, request.as_ptr(), &raw mut out) };
/// assert_eq!(status, Status::Ok);
/// // SAFETY: the library wrote `len` bytes of UTF-8 at `data`.
/// let bytes = unsafe { core::slice::from_raw_parts(out.data, out.len) };
/// let answer: serde_json::Value = serde_json::from_slice(bytes)?;
/// let claim = &answer["value"]["claims"][0];
/// assert_eq!(claim["graha"], "graha.SUN");
/// assert_eq!(claim["verdict"], "HELD");
/// // SAFETY: a descriptor the library wrote, freed once.
/// unsafe { ts_string_free(&raw mut out) };
/// # }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `request_json` NUL-terminated;
/// `out_json` valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_verify_claims(
    context: *const TsContext,
    request_json: *const c_char,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |ctx| {
        // SAFETY: the entry point's contract.
        let asked = teistro::verify::ClaimsRequest::from_json(unsafe {
            text(request_json, "request_json")
        }?)?;
        let answer = ctx.sdk().verify().claims(&asked)?;
        let json = TsString::from_string(teistro_core::envelope::canonical_json(&answer));
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_json, "out_json", json) }
    })
}
