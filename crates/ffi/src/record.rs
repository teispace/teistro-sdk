//! Any record entry point by name, answered as canonical JSON at the
//! detail a call asks for or by a caller's field mask
//! (`03-design/result-detail.md` D5, D6).
//!
//! The typed entry points (`ts_almanac_digest`, `ts_chart_found`, …)
//! answer a record whole, as a blob or as JSON a binding reads into its
//! own classes. This one answers any record as the envelope an agent
//! reads, so an application that hands an answer to a model it drives
//! itself asks for the brief or the fields it wants, and every binding
//! reaches every record through one function.
#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

use core::ffi::c_char;

use teistro::records::{Answered, Detail, Record};
use teistro_core::error::{Error, Status};

use crate::context::TsContext;
use crate::string::TsString;
use crate::support::{optional_text, text, with_context, write_plain};

/// How much of its answer a record call asks for.
///
/// ```
/// use teistro_ffi::record::TsDetail;
///
/// // A member crosses the boundary as its number: the whole answer is 0,
/// // so a zeroed request asks for it.
/// assert_eq!((TsDetail::Full as u32, TsDetail::Brief as u32), (0, 1));
/// ```
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsDetail {
    /// The whole answer, as serde writes it.
    Full = 0,
    /// The fields a reader needs, as the record declares them, with the
    /// provenance's answer key and none of its hashes.
    Brief = 1,
}

impl TsDetail {
    fn from_id(id: u32) -> Result<Detail, Error> {
        match id {
            0 => Ok(Detail::Full),
            1 => Ok(Detail::Brief),
            _ => Err(Error::invalid_arg(format!(
                "`detail` is {id}; the details are FULL=0, BRIEF=1"
            ))
            .with_field("detail")),
        }
    }
}

/// Answers the record entry point `name` (`almanac.days`, `chart.found`)
/// for `request_json`, its request as the record's own reader takes it,
/// as canonical JSON: the envelope `{value, provenance}`, or the value
/// alone for a record that seals no provenance.
///
/// `detail` is `FULL` for the whole answer, and `BRIEF` for the fields a
/// reader needs, the provenance written with its answer key and none of
/// its hashes, and `"detail": "brief"` beside them. `fields_json`, when
/// not null, is a caller's own mask, a JSON list of field paths
/// (`["date.year", "weekday"]`) of the full answer to keep, each checked
/// against the record's answer schema; the envelope then carries
/// `"fields"` beside the brief provenance. A mask selects from the full
/// answer, so it is refused beside `BRIEF` (crux C1260).
///
/// `INVALID_ARG` on `name` for a record this build does not carry, on
/// `detail` for another number, on `fields_json` for a mask that is no
/// list of strings or is named beside `BRIEF`, and on `fields[i]` for a
/// path the answer does not have; `CAPABILITY` on `fields_json` for a
/// build without the answer schemas a mask is checked against (the
/// `fields` feature, which a wasm module leaves out); and whatever the
/// record's reader and its area refuse.
///
/// `api: detail: enum=TsDetail example=1`
/// `api: fields_json: nullable example=["weekday"]`
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
/// use teistro_ffi::record::{TsDetail, ts_record_answer};
/// use teistro_ffi::string::TsString;
///
/// let request = c"{\"date\": {\"year\": 2026, \"month\": 10, \"day\": 10}, \"into\": \"calendar.BIKRAM_SAMBAT\"}";
/// let mut out = TsString::empty();
/// // SAFETY: a live context, NUL-terminated strings, no mask and a writable slot.
/// let status = unsafe {
///     ts_record_answer(&raw const ctx, c"calendar.convert".as_ptr(), request.as_ptr(), TsDetail::Full as u32, core::ptr::null(), &raw mut out)
/// };
/// assert_eq!(status, Status::Ok);
/// let answer: serde_json::Value = serde_json::from_str(&owned(out))?;
/// assert_eq!(answer["weekday"], "SATURDAY");
///
/// # #[cfg(feature = "fields")] {
/// // A caller's mask keeps the fields it names.
/// let mut out = TsString::empty();
/// // SAFETY: as above, with a NUL-terminated mask.
/// let status = unsafe {
///     ts_record_answer(&raw const ctx, c"calendar.convert".as_ptr(), request.as_ptr(), TsDetail::Full as u32, c"[\"weekday\"]".as_ptr(), &raw mut out)
/// };
/// assert_eq!(status, Status::Ok);
/// assert_eq!(owned(out), r#"{"weekday":"SATURDAY"}"#);
/// # }
///
/// // A record the build does not carry is refused by name.
/// let mut out = TsString::empty();
/// // SAFETY: as above.
/// let status = unsafe {
///     ts_record_answer(&raw const ctx, c"calendar.nothing".as_ptr(), request.as_ptr(), TsDetail::Brief as u32, core::ptr::null(), &raw mut out)
/// };
/// assert_eq!(status, Status::InvalidArg);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `name` and `request_json`
/// NUL-terminated; `fields_json` null or NUL-terminated; `out_json` valid
/// for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_record_answer(
    context: *const TsContext,
    name: *const c_char,
    request_json: *const c_char,
    detail: u32,
    fields_json: *const c_char,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |ctx| {
        // SAFETY: the entry point's contract.
        let (name, request, fields) = unsafe {
            (
                text(name, "name")?,
                text(request_json, "request_json")?,
                optional_text(fields_json, "fields_json")?,
            )
        };
        let detail = TsDetail::from_id(detail)?;
        let record = teistro::records::record(name).ok_or_else(|| {
            Error::invalid_arg(format!("this build carries no record `{name}`"))
                .with_field("name")
                .with_hint("a record is named `area.operation`, as `calendar.convert`")
        })?;
        let full = record.answer(ctx.sdk(), request)?;
        let answered = match fields {
            None => match detail {
                Detail::Full => full,
                Detail::Brief => record.brief_of(full),
            },
            Some(_) if detail == Detail::Brief => {
                return Err(Error::invalid_arg(
                    "a field mask selects from the full answer, so it is not asked at `BRIEF`",
                )
                .with_field("fields_json")
                .with_hint("ask at `FULL` beside a mask"));
            }
            Some(fields) => masked(record, full, fields)?,
        };
        let json = TsString::from_string(answered.json());
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_json, "out_json", json) }
    })
}

/// `full` selected by the mask `fields`, a JSON list of field paths.
#[cfg(feature = "fields")]
fn masked(record: Record, full: Answered, fields: &str) -> Result<Answered, Error> {
    let paths: Vec<String> = serde_json::from_str(fields).map_err(|_| {
        Error::invalid_arg("`fields_json` is a JSON list of field paths")
            .with_field("fields_json")
            .with_hint("as `[\"date.year\", \"weekday\"]`")
    })?;
    let mask = record.fields(paths)?;
    record.fields_of(full, &mask)
}

/// A build without the answer schemas has no mask to check against.
#[cfg(not(feature = "fields"))]
fn masked(_: Record, _: Answered, _: &str) -> Result<Answered, Error> {
    Err(Error::new(
        Status::Capability,
        "this build carries no answer schemas to check a field mask against",
    )
    .with_field("fields_json")
    .with_hint("a library built with the `fields` feature answers a mask"))
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::panic,
        clippy::expect_used,
        clippy::unwrap_used,
        reason = "a test fails by panicking"
    )]

    use super::TsDetail;
    use teistro::records::Detail;

    #[test]
    fn a_detail_crosses_as_its_number_and_another_is_refused() {
        assert_eq!(TsDetail::from_id(0).unwrap(), Detail::Full);
        assert_eq!(TsDetail::from_id(1).unwrap(), Detail::Brief);
        let refused = TsDetail::from_id(2).unwrap_err();
        assert_eq!(refused.field(), Some("detail"));
    }
}
