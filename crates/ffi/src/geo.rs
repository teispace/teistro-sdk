//! The geo port at the C boundary (`03-design/geo-port.md`, SR-87): a
//! place resolved through the context's geo provider, and a table of the
//! caller's own installed as that provider.
#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

use core::ffi::c_char;

use teistro_core::error::Status;

use crate::context::TsContext;
use crate::string::TsString;
use crate::support::{text, with_context, write_plain};

/// A place resolved through the context's geo provider, answered as
/// canonical JSON `{latitudeDeg, longitudeDeg, zone, height, admin,
/// source}`: the IANA zone (or null), the ground height
/// `{metres, source, resolutionM, datum}` (or null), the admin units
/// holding the point coarse to fine, each `{level, code, name}` with its
/// ISO 3166-1 (level 0) or ISO 3166-2 code, and the provider's
/// `{name, version, dataVersion, tzdb}`. A chart request's `place` takes
/// it as it is, so the request a receipt keeps replays without the
/// provider.
///
/// `request_json` is `{"latitudeDeg", "longitudeDeg"}`. A key it does not
/// read is `INVALID_ARG` under `geo`, a point out of range `INVALID_ARG` on
/// `latitudeDeg` or `longitudeDeg`; a context with no provider is
/// `CAPABILITY` on `geo`; an answer of the provider's that is not well
/// formed is `PROVIDER`, named by its field (`admin[1].code`).
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::None)?;
/// use teistro_core::Status;
/// use teistro_ffi::geo::ts_geo_resolve;
/// use teistro_ffi::string::TsString;
///
/// // A context is given no provider unless it is asked for one.
/// let mut out = TsString::empty();
/// // SAFETY: a live context, a NUL-terminated request and a writable slot.
/// let status = unsafe {
///     ts_geo_resolve(&raw const ctx, c"{\"latitudeDeg\": 27.7, \"longitudeDeg\": 85.3}".as_ptr(), &raw mut out)
/// };
/// assert_eq!(status, Status::Capability);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `request_json` NUL-terminated;
/// `out_json` valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_geo_resolve(
    context: *const TsContext,
    request_json: *const c_char,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |ctx| {
        // SAFETY: the entry point's contract.
        let asked =
            teistro::PlaceRequest::from_json(unsafe { text(request_json, "request_json") }?)?;
        let place = ctx.sdk().geo().resolve(&asked)?;
        let json = TsString::from_string(teistro_core::envelope::canonical_json(&place));
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_json, "out_json", json) }
    })
}

/// Installs a geo table as the context's geo provider, in place of any it
/// had, answering the table's source as canonical JSON
/// `{name, version, dataVersion, tzdb}`.
///
/// `table_json` is `{"source", "areas"}`: `source` as answered, and
/// `areas` a list of boxes `{south, west, north, east}` in degrees (`west`
/// east of `east` for a box across the antimeridian), each with any of
/// `zone`, `height` and `admin` as `ts_geo_resolve` answers them. Each
/// question is answered by the last box holding the point that answers
/// it, so a table is written coarse to fine. A key it does not read, an
/// edge out of range or a malformed answer is `INVALID_ARG` named under
/// `geo` (`geo.areas[2].admin[0].code`), and leaves the context as it was.
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
/// use teistro_ffi::geo::{ts_geo_load, ts_geo_resolve};
/// use teistro_ffi::string::TsString;
///
/// let table = c"{\"source\": {\"name\": \"places\", \"version\": \"1\", \"dataVersion\": \"2026.11.0\", \"tzdb\": null},
///     \"areas\": [{\"south\": 26.3, \"west\": 80.0, \"north\": 30.5, \"east\": 88.2, \"zone\": \"Asia/Kathmandu\"}]}";
/// let mut out = TsString::empty();
/// // SAFETY: a live context, a NUL-terminated table and a writable slot.
/// assert_eq!(unsafe { ts_geo_load(&raw const ctx, table.as_ptr(), &raw mut out) }, Status::Ok);
/// let source: serde_json::Value = serde_json::from_str(&owned(out))?;
/// assert_eq!(source["dataVersion"], "2026.11.0");
///
/// let mut out = TsString::empty();
/// // SAFETY: as above, with a NUL-terminated request.
/// let status = unsafe {
///     ts_geo_resolve(&raw const ctx, c"{\"latitudeDeg\": 27.7, \"longitudeDeg\": 85.3}".as_ptr(), &raw mut out)
/// };
/// assert_eq!(status, Status::Ok);
/// let place: serde_json::Value = serde_json::from_str(&owned(out))?;
/// assert_eq!(place["zone"], "Asia/Kathmandu");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `table_json` NUL-terminated;
/// `out_json` valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_geo_load(
    context: *const TsContext,
    table_json: *const c_char,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |ctx| {
        // SAFETY: the entry point's contract.
        let table = unsafe { text(table_json, "table_json") }?;
        let source = ctx.sdk().geo().load(table)?;
        let json = TsString::from_string(teistro_core::envelope::canonical_json(&source));
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_json, "out_json", json) }
    })
}
