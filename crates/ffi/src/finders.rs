//! The finders at the C boundary (A2-1): the election search, the
//! observance finder and the people digest, each one JSON record in and
//! `{value, provenance}` out as canonical JSON, the façade's own answer
//! (`03-design/election-search.md`, `observance-finder.md`,
//! `people-digest.md`). Catalogue members are serde's bare keys.
#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

use core::ffi::c_char;

use teistro_core::error::Status;

use crate::context::TsContext;
use crate::family::in_family;
use crate::string::TsString;
use crate::support::{text, with_context, write_plain};

/// The times a muhurta rule set leaves over a range at a place, ranked
/// best first (`03-design/election-search.md`), answered with `{value,
/// provenance}` as canonical JSON.
///
/// `value` is `{windows, closed, daysJudged, daysCut, windowsBlackedOut,
/// ranking, unjudged}`, each window `{rank, at, open, holds, fails,
/// barredBy, score}`: `rank` 1 the best, `at` its UTC Julian days,
/// `holds` the clauses that count for the time over it and `fails` those
/// against it, each with the span it held over, and `barredBy` the rules'
/// bars that close it.
///
/// `request_json` is `{"calendar", "first", "last", "latitudeDeg",
/// "longitudeDeg", "altitudeM", "utcOffsetSeconds", "muhurta"}`: the days
/// as `{"year", "month", "day"}` in `calendar` (Gregorian when left out),
/// `last` the first when left out, and `muhurta` the rule set as
/// `almanac.days` reads it (`rules` an activity's name or rules spelt
/// out, `native`, `ranking`, `daysWithWindows`, `most`, `asta`,
/// `direction`). A key it does not read, a place or offset out of range or
/// a rule set its reader refuses is `INVALID_ARG`, named under `elect`; a
/// context without an ephemeris is `CAPABILITY`, as is a build without the
/// muhurta family.
///
/// ```
/// # #[cfg(all(feature = "builtin-ephemeris", feature = "muhurta"))] {
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::Builtin)?;
/// use teistro_core::Status;
/// use teistro_ffi::finders::ts_almanac_elect;
/// use teistro_ffi::string::{TsString, ts_string_free};
///
/// let request = c"{\"first\": {\"year\": 2026, \"month\": 11, \"day\": 25}, \"last\": {\"year\": 2026, \"month\": 11, \"day\": 26}, \"latitudeDeg\": 27.7172, \"longitudeDeg\": 85.324, \"utcOffsetSeconds\": 20700, \"muhurta\": {\"rules\": \"RAMAN_MARRIAGE\", \"most\": 2}}";
/// let mut out = TsString::empty();
/// // SAFETY: a live context, a NUL-terminated request and a writable slot.
/// let status = unsafe { ts_almanac_elect(&raw const ctx, request.as_ptr(), &raw mut out) };
/// assert_eq!(status, Status::Ok);
/// // SAFETY: the library wrote `len` bytes of UTF-8 at `data`.
/// let bytes = unsafe { core::slice::from_raw_parts(out.data, out.len) };
/// let answer: serde_json::Value = serde_json::from_slice(bytes)?;
/// assert!(answer["value"]["windows"].as_array().is_some_and(|w| w.len() <= 2));
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
pub unsafe extern "C" fn ts_almanac_elect(
    context: *const TsContext,
    request_json: *const c_char,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |ctx| {
        in_family!("muhurta", [ctx, request_json, out_json], {
            // SAFETY: the entry point's contract.
            let asked = teistro::finders::ElectRequest::from_json(unsafe {
                text(request_json, "request_json")
            }?)?;
            let answer = ctx.sdk().almanac().elect(&asked)?;
            let json = TsString::from_string(teistro_core::envelope::canonical_json(&answer));
            // SAFETY: the entry point's contract.
            unsafe { write_plain(out_json, "out_json", json) }
        })
    })
}

/// When an observance falls in a year or a window under each named
/// tradition (`03-design/observance-finder.md`), the occurrences paired
/// across them, answered with `{value, provenance}` as canonical JSON,
/// sealed by the first tradition's reckoning.
///
/// `value` is `{traditions, occurrences, parted, unjudged}`: each
/// tradition `{name, rule, reckoning, profile, settingsHash, provider}`;
/// each occurrence `{month, adhika, kept, agree, differs, reason}`, `kept` a
/// tradition's `{tradition, day, observance}` or `{tradition, day, fast}`
/// in the request's order (no `day` where it keeps none), `differs` among
/// `ABSENT`, `DAY` and `MONTH`, and `reason` `RULE`, `SKY`,
/// `RULE_AND_SKY` or `PUBLISHED` where the days or months part.
///
/// `request_json` is `{"calendar", "year", "first", "last",
/// "latitudeDeg", "longitudeDeg", "altitudeM", "utcOffsetSeconds",
/// "traditions"}`: a `year` of `calendar` (Gregorian when left out) or
/// the days `first` to `last`; `traditions` 1 to 8, each `{"name",
/// "festivals", "rule", "reckoning"}`, `festivals` a festival request,
/// `rule` a key of it and `reckoning` `THIS` (the default) or `OTHER`,
/// which reckons it under `other`. `other` is null, or `context` itself,
/// when no tradition is reckoned `OTHER`. A key it does not read, a place
/// or offset out of range, a rule `festivals` does not carry, a name given
/// twice or an `OTHER` tradition with no other context is `INVALID_ARG`;
/// more than 8 traditions `LIMIT`; a dated rule `UNSUPPORTED`; each named
/// under `observance`.
///
/// ```
/// # #[cfg(feature = "builtin-ephemeris")] {
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::Builtin)?;
/// use teistro_core::Status;
/// use teistro_ffi::finders::ts_almanac_observance;
/// use teistro_ffi::string::{TsString, ts_string_free};
///
/// let request = c"{\"first\": {\"year\": 2026, \"month\": 1, \"day\": 1}, \"last\": {\"year\": 2026, \"month\": 1, \"day\": 31}, \"latitudeDeg\": 27.7172, \"longitudeDeg\": 85.324, \"utcOffsetSeconds\": 20700, \"traditions\": [{\"name\": \"smarta\", \"festivals\": {\"rules\": \"DHARMASINDHU\"}, \"rule\": \"EKADASHI_SMARTA\"}]}";
/// let mut out = TsString::empty();
/// // SAFETY: a live context, no other, a NUL-terminated request and a writable slot.
/// let status = unsafe { ts_almanac_observance(&raw const ctx, core::ptr::null(), request.as_ptr(), &raw mut out) };
/// assert_eq!(status, Status::Ok);
/// // SAFETY: the library wrote `len` bytes of UTF-8 at `data`.
/// let bytes = unsafe { core::slice::from_raw_parts(out.data, out.len) };
/// let answer: serde_json::Value = serde_json::from_slice(bytes)?;
/// // January holds two Ekadashis, and one tradition agrees with itself.
/// assert_eq!(answer["value"]["occurrences"].as_array().map(Vec::len), Some(2));
/// assert_eq!(answer["value"]["parted"], 0);
/// // SAFETY: a descriptor the library wrote, freed once.
/// unsafe { ts_string_free(&raw mut out) };
/// # }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `other` null or a live handle, which
/// may be `context`; `request_json` NUL-terminated; `out_json` valid for
/// a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_almanac_observance(
    context: *const TsContext,
    other: *const TsContext,
    request_json: *const c_char,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |ctx| {
        // This context itself is no other reckoning.
        let other = if core::ptr::eq(other, context) {
            None
        } else {
            // SAFETY: the caller promises null or a live handle.
            unsafe { crate::support::context(other) }
        };
        // SAFETY: the entry point's contract.
        let asked = teistro::finders::ObservanceRequest::from_json(unsafe {
            text(request_json, "request_json")
        }?)?;
        let answer = ctx
            .sdk()
            .almanac()
            .observance(&asked, other.map(TsContext::sdk))?;
        let json = TsString::from_string(teistro_core::envelope::canonical_json(&answer));
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_json, "out_json", json) }
    })
}

/// A day for many people at once (`03-design/people-digest.md`),
/// answered with `{value, provenance}` as canonical JSON, sealed by the
/// day's reckoning.
///
/// `value` is `{day, sunrise, nextSunrise, nakshatra, moonSign, dasha,
/// people}`: the day's star and the Moon's sign at sunrise, and each
/// person `{name, birthStar, moonSign, lagna, dasha, dashaOpens, transits,
/// hits, tara, chandraHouse, chandrashtama, sadeSati}` in the request's
/// order: the dasha running at sunrise a period a level, the periods
/// opening before the next sunrise, every graha's gochar from their Moon,
/// the day's transit hits on their chart, the day's star counted from
/// their birth star, and the Moon's house from their Moon.
///
/// `request_json` is `{"calendar", "day", "latitudeDeg", "longitudeDeg",
/// "altitudeM", "utcOffsetSeconds", "people", "dasha", "depth"}`: `day`
/// as `{"year", "month", "day"}` in `calendar` (Gregorian when left out)
/// at the place given; `people` 1 to 50, each `{"name", "instant",
/// "latitudeDeg", "longitudeDeg", "altitudeM", "utcOffsetSeconds"}`,
/// `instant` the birth's UTC Julian day; `dasha` `VIMSHOTTARI` and
/// `depth` 3 when left out. A key it does not read, a place, offset or
/// instant out of range, a name given twice or a depth past 1 to 6 is
/// `INVALID_ARG`, more than 50 people `LIMIT`, each named under `people`;
/// a context without an ephemeris is `CAPABILITY`, as is a build without
/// the chart or the muhurta family.
///
/// ```
/// # #[cfg(all(feature = "builtin-ephemeris", feature = "chart", feature = "muhurta"))] {
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::Builtin)?;
/// use teistro_core::Status;
/// use teistro_ffi::finders::ts_almanac_people;
/// use teistro_ffi::string::{TsString, ts_string_free};
///
/// let request = c"{\"day\": {\"year\": 2026, \"month\": 11, \"day\": 1}, \"latitudeDeg\": 27.7172, \"longitudeDeg\": 85.324, \"utcOffsetSeconds\": 20700, \"people\": [{\"name\": \"Asha\", \"instant\": 2447000.25, \"latitudeDeg\": 27.7172, \"longitudeDeg\": 85.324, \"utcOffsetSeconds\": 20700}]}";
/// let mut out = TsString::empty();
/// // SAFETY: a live context, a NUL-terminated request and a writable slot.
/// let status = unsafe { ts_almanac_people(&raw const ctx, request.as_ptr(), &raw mut out) };
/// assert_eq!(status, Status::Ok);
/// // SAFETY: the library wrote `len` bytes of UTF-8 at `data`.
/// let bytes = unsafe { core::slice::from_raw_parts(out.data, out.len) };
/// let answer: serde_json::Value = serde_json::from_slice(bytes)?;
/// let asha = &answer["value"]["people"][0];
/// assert_eq!(asha["dasha"].as_array().map(Vec::len), Some(3));
/// assert_eq!(asha["transits"].as_array().map(Vec::len), Some(9));
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
pub unsafe extern "C" fn ts_almanac_people(
    context: *const TsContext,
    request_json: *const c_char,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |ctx| {
        in_family!("chart", [ctx, request_json, out_json], {
            in_family!("muhurta", [ctx, request_json, out_json], {
                // SAFETY: the entry point's contract.
                let asked = teistro::finders::PeopleRequest::from_json(unsafe {
                    text(request_json, "request_json")
                }?)?;
                let answer = ctx.sdk().almanac().people(&asked)?;
                let json = TsString::from_string(teistro_core::envelope::canonical_json(&answer));
                // SAFETY: the entry point's contract.
                unsafe { write_plain(out_json, "out_json", json) }
            })
        })
    })
}
