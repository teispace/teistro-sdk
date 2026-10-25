//! Studies over a batch of births at the C boundary, answered as
//! canonical JSON (`03-design/research.md`).
//!
//! A study's answer is a table of rows per predicate, each with its
//! counts by group, its p-values and its effect sizes: nested records
//! rather than columns, so it crosses as the JSON the façade's envelope
//! serialises to, provenance and all, because the input hash is the
//! study's pre-registration.
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

/// Runs a study over a batch of births and answers with `{value,
/// provenance}` as canonical JSON. `value` is `{rows}` for a `COUNTS`
/// study, each row `{predicate, counts}` with every group's `{present,
/// absent, unreadable, unstable}`; for any other study it is `{rows,
/// permutations, resolution, shuffle}`, each row adding `observed`, `p`
/// (`{exceed, value, low, high}`), `exact`, `adjusted` (`{maxT, holm,
/// bonferroni, bh, by}`), `effect`, `expected` and `underAlpha`, the last
/// four only where they apply. `provenance.input_hash` seals the study
/// and is what a study publishes before its data are collected.
///
/// `request_json` is `{"study", "rules", "holds", ...}`: `study` one of
/// `COUNTS`, `COMPARE`, `EXPECTED` and `TIMED`, `rules` the record a chart
/// request's rules are, and what the study reads: `births` and `design`
/// (and a `test` for `COMPARE`); `births`, `control` and an optional
/// `test` for `EXPECTED`; `subjects`, `dasha`, `shuffle`, `test` and
/// optionally `depth` and `strata` for `TIMED`. A birth is `{instant,
/// latitudeDeg, longitudeDeg, altitudeM, utcOffsetSeconds,
/// uncertaintyMinutes}`, its instant a Julian day in UTC. A seed is a
/// number or a decimal string. A key it does not read, a field the study
/// does not read or misses, or a value out of range is `INVALID_ARG`,
/// named under `research`, as `research.test.seed`; what the study
/// refuses once it runs is named as the façade names it. A context
/// without an ephemeris is `CAPABILITY`, as is a build that leaves the
/// `research` family out.
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
/// use teistro_ffi::research::ts_research;
///
/// let request = c"{\"study\": \"COMPARE\", \"rules\": {\"shipped\": [\"YOGAS\"]}, \"births\": [{\"instant\": 2447000.25, \"latitudeDeg\": 27.7172, \"longitudeDeg\": 85.324, \"utcOffsetSeconds\": 20700}, {\"instant\": 2447977.55, \"latitudeDeg\": 27.7172, \"longitudeDeg\": 85.324, \"utcOffsetSeconds\": 20700}, {\"instant\": 2448954.85, \"latitudeDeg\": 27.7172, \"longitudeDeg\": 85.324, \"utcOffsetSeconds\": 20700}, {\"instant\": 2449932.15, \"latitudeDeg\": 27.7172, \"longitudeDeg\": 85.324, \"utcOffsetSeconds\": 20700}, {\"instant\": 2450909.45, \"latitudeDeg\": 27.7172, \"longitudeDeg\": 85.324, \"utcOffsetSeconds\": 20700}, {\"instant\": 2451886.75, \"latitudeDeg\": 27.7172, \"longitudeDeg\": 85.324, \"utcOffsetSeconds\": 20700}, {\"instant\": 2452864.05, \"latitudeDeg\": 27.7172, \"longitudeDeg\": 85.324, \"utcOffsetSeconds\": 20700}, {\"instant\": 2453841.35, \"latitudeDeg\": 27.7172, \"longitudeDeg\": 85.324, \"utcOffsetSeconds\": 20700}], \"design\": {\"groups\": [0, 1, 0, 1, 0, 1, 0, 1]}, \"test\": {\"seed\": 5, \"permutations\": 99, \"contrast\": {\"kind\": \"CASE_VS_REST\", \"case\": 1}}}";
/// let mut out = TsString::empty();
/// // SAFETY: a live context, a NUL-terminated request and a writable slot.
/// let status = unsafe { ts_research(&raw const ctx, request.as_ptr(), &raw mut out) };
/// assert_eq!(status, Status::Ok);
/// let answer: serde_json::Value = serde_json::from_str(&owned(out))?;
/// // The study's envelope whole: what produced it can be quoted.
/// assert!(answer["provenance"]["input_hash"].is_string());
/// assert!(answer["value"].is_object());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `request_json` NUL-terminated;
/// `out_json` valid for a write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_research(
    context: *const TsContext,
    request_json: *const c_char,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |ctx| {
        in_family!("research", [ctx, request_json, out_json], {
            // SAFETY: the entry point's contract.
            let asked = teistro::ResearchRequest::from_json(unsafe {
                crate::support::text(request_json, "request_json")
            }?)?;
            let answer = ctx.sdk().research().request(&asked)?;
            let json = TsString::from_string(teistro_core::envelope::canonical_json(&answer));
            // SAFETY: the entry point's contract.
            unsafe { crate::support::write_plain(out_json, "out_json", json) }
        })
    })
}
