//! Claims checked at the C boundary: what a text claims of the time,
//! judged against the sankalpa the SDK computes at an instant and a place
//! (`03-design/lexicon.md` §3); and receipts, a record answered with one
//! and one checked by answering it again
//! (`03-design/receipts-and-datasets.md` §1). Each answered as canonical
//! JSON.
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

/// A record answered with its receipt (`03-design/receipts-and-datasets.md`
/// §1.2), answered as canonical JSON: `{answer, receipt}`.
///
/// `answer` is the record's own answer, `{value, provenance}` where it is
/// sealed and the value alone where it is not. `receipt` is `{format,
/// record, request, requestDigest, settings, locale, packs, provider, tzdb,
/// sdkVersion, calculationVersion, catalogueVersion, outputDigest,
/// digest}`: `format` `teistro.receipt/1`, `settings` the snapshot
/// `{profile, settings, hash}` with the whole settings document,
/// `provider` `{name, version, dataVersion}` or null where the answer read
/// no ephemeris, `outputDigest` the SHA-256 of the value's canonical JSON,
/// and `digest` the seal, the SHA-256 of the receipt's canonical JSON
/// without `digest`. An OKF *Attested Computation*'s executor; hand the
/// receipt to `ts_verify_receipt` to attest it.
///
/// `request_json` is `{"record", "request"}`: a record's name as the
/// records table lists it and its request. A key it does not read is
/// `INVALID_ARG`, named under `issue`; a record this build does not
/// carry is `UNSUPPORTED`, named `issue.record`; a request that is not an
/// object is `INVALID_ARG`, named `issue.request`; otherwise whatever the
/// record refuses.
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::None)?;
/// use teistro_core::Status;
/// use teistro_ffi::string::{TsString, ts_string_free};
/// use teistro_ffi::verify::ts_verify_issue;
///
/// let request = c"{\"record\": \"calendar.convert\", \"request\": {\"date\": {\"year\": 2026, \"month\": 10, \"day\": 10}, \"into\": \"calendar.BIKRAM_SAMBAT\"}}";
/// let mut out = TsString::empty();
/// // SAFETY: a live context, a NUL-terminated request and a writable slot.
/// let status = unsafe { ts_verify_issue(&raw const ctx, request.as_ptr(), &raw mut out) };
/// assert_eq!(status, Status::Ok);
/// // SAFETY: the library wrote `len` bytes of UTF-8 at `data`.
/// let bytes = unsafe { core::slice::from_raw_parts(out.data, out.len) };
/// let issued: serde_json::Value = serde_json::from_slice(bytes)?;
/// assert_eq!(issued["answer"]["date"]["year"], 2083);
/// assert_eq!(issued["receipt"]["record"], "calendar.convert");
/// // SAFETY: a descriptor the library wrote, freed once.
/// unsafe { ts_string_free(&raw mut out) };
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `request_json` NUL-terminated;
/// `out_json` valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_verify_issue(
    context: *const TsContext,
    request_json: *const c_char,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |ctx| {
        // SAFETY: the entry point's contract.
        let asked = teistro::verify::IssueRequest::from_json(unsafe {
            text(request_json, "request_json")
        }?)?;
        let issued = ctx.sdk().verify().issue(&asked)?;
        let json = TsString::from_string(teistro_core::envelope::canonical_json(&issued));
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_json, "out_json", json) }
    })
}

/// A receipt checked (`03-design/receipts-and-datasets.md` §1.3): its
/// record answered again on this context under the receipt's own settings,
/// locale and request, and the value's digest compared, answered as
/// canonical JSON: `{verdict, record, moved, broken, issued, now, reason}`.
///
/// `verdict` is `HELD` (the digest is the receipt's), `DIFFERS` (another),
/// `NOT_RERUN` (not answered again; `reason` is `{cause, detail}`, the
/// cause `RECORD`, `PROVIDER`, `SETTINGS`, `LOCALE`, `PACKS`, `OUT_OF_RANGE`
/// or `REFUSED`) or `BROKEN` (a digest the receipt carries is not its own
/// fields'; `broken` names each: `requestDigest`, `settings.hash`,
/// `digest`). `moved` lists each field that moved between the receipt and
/// the check, `{field, issued, now}`: `output` first when the value did,
/// then `sdkVersion`, `calculationVersion`, `catalogueVersion`, `provider`
/// and `tzdb`. `issued` and `now` are the two sides' `{sdkVersion,
/// calculationVersion, catalogueVersion, provider, tzdb}`. An OKF
/// *Attested Computation*'s attester.
///
/// `request_json` is the receipt `ts_verify_issue` answered, whole. A key
/// it does not have is `INVALID_ARG`, named under `receipt`; a format other
/// than `teistro.receipt/1` is `SCHEMA_VERSION`, named `receipt.format`.
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::None)?;
/// use teistro_core::Status;
/// use teistro_ffi::string::{TsString, ts_string_free};
/// use teistro_ffi::verify::ts_verify_receipt;
///
/// let issued = ctx.sdk().verify().issue(&teistro::verify::IssueRequest::from_json(
///     r#"{"record": "calendar.convert", "request": {"date": {"year": 2026, "month": 10, "day": 10}, "into": "calendar.BIKRAM_SAMBAT"}}"#,
/// )?)?;
/// let receipt = std::ffi::CString::new(teistro_core::envelope::canonical_json(&issued.receipt))?;
/// let mut out = TsString::empty();
/// // SAFETY: a live context, a NUL-terminated receipt and a writable slot.
/// let status = unsafe { ts_verify_receipt(&raw const ctx, receipt.as_ptr(), &raw mut out) };
/// assert_eq!(status, Status::Ok);
/// // SAFETY: the library wrote `len` bytes of UTF-8 at `data`.
/// let bytes = unsafe { core::slice::from_raw_parts(out.data, out.len) };
/// let check: serde_json::Value = serde_json::from_slice(bytes)?;
/// assert_eq!(check["verdict"], "HELD");
/// // SAFETY: a descriptor the library wrote, freed once.
/// unsafe { ts_string_free(&raw mut out) };
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `request_json` NUL-terminated;
/// `out_json` valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_verify_receipt(
    context: *const TsContext,
    request_json: *const c_char,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |ctx| {
        // SAFETY: the entry point's contract.
        let receipt =
            teistro::verify::Receipt::from_json(unsafe { text(request_json, "request_json") }?)?;
        let check = ctx.sdk().verify().receipt(&receipt);
        let json = TsString::from_string(teistro_core::envelope::canonical_json(&check));
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_json, "out_json", json) }
    })
}
