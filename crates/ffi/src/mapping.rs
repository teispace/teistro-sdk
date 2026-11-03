//! Relocation and astrocartography at the C boundary
//! (`03-design/astrocartography.md`): a chart's angles and houses recast
//! at other places, and each body's lines on the Earth, answered as
//! canonical JSON.
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

/// **A chart recast at other places** (`03-design/astrocartography.md`):
/// each place's ascendant, midheaven and twelve cusps at one instant, in
/// the context's zodiac, as `{value, provenance}` canonical JSON sealed by
/// the first place's chart with the request as its input.
///
/// `value` is `{instant, obliquityDeg, places}`, each place
/// `{latitudeDeg, longitudeDeg, altitudeM, ascendantDeg, midheavenDeg,
/// cuspsDeg, houseSystem}` in the order asked, `cuspsDeg` twelve and
/// `houseSystem` a `house_system` key.
///
/// `request_json` is `{"instant", "places", "houseSystem"}`: `instant` a
/// UTC Julian day; `places` one to 256 `{"latitudeDeg", "longitudeDeg",
/// "altitudeM"}`; `houseSystem` a `house_system` key, the profile's
/// Western override else Placidus when left out (C1157). A key the record
/// does not read, no place or too many, or a place out of range is
/// `INVALID_ARG`, named under `relocated` (`relocated.places[1].latitudeDeg`);
/// a context without an ephemeris is `CAPABILITY`.
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
/// use teistro_ffi::mapping::ts_chart_relocated;
/// use teistro_ffi::string::TsString;
///
/// let request = c"{\"instant\": 2447000.25, \"places\": [{\"latitudeDeg\": 27.7172, \"longitudeDeg\": 85.324}]}";
/// let mut out = TsString::empty();
/// // SAFETY: a live context, a NUL-terminated request and a writable slot.
/// let status = unsafe { ts_chart_relocated(&raw const ctx, request.as_ptr(), &raw mut out) };
/// assert_eq!(status, Status::Ok);
/// let answer: serde_json::Value = serde_json::from_str(&owned(out))?;
/// assert_eq!(answer["value"]["places"][0]["cuspsDeg"].as_array().map(Vec::len), Some(12));
/// # }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `request_json` NUL-terminated;
/// `out_json` valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_chart_relocated(
    context: *const TsContext,
    request_json: *const c_char,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |ctx| {
        in_family!("western", [ctx, request_json, out_json], {
            // SAFETY: the entry point's contract.
            let text = unsafe { crate::support::text(request_json, "request_json") }?;
            let asked = teistro::RelocationRequest::from_json(text)?;
            let answer = ctx.sdk().chart().relocated(&asked)?;
            let json = TsString::from_string(teistro_core::envelope::canonical_json(&answer));
            // SAFETY: the entry point's contract.
            unsafe { crate::support::write_plain(out_json, "out_json", json) }
        })
    })
}

/// **The astrocartography lines** at an instant
/// (`03-design/astrocartography.md`): each body's MC, IC, ASC and DSC
/// lines on the Earth, as `{value, provenance}` canonical JSON sealed by
/// the chart the bodies were read from, with the request as its input.
///
/// `value` is `{instant, projection, stepDeg, latitudeLimitDeg,
/// toleranceDeg, greenwichRamcDeg, obliquityDeg, lines}`, each line
/// `{body, angle, points, polarLimitDeg}`: `body` a `graha` key, `angle`
/// `MC`, `IC`, `ASC` or `DSC`, `points` `[longitudeDeg, latitudeDeg]`
/// pairs from south to north, longitude east in `[-180, 180)`, and
/// `polarLimitDeg` for a rising or setting line the latitude past which
/// it does not run (C1156). Every point is on its angle, read back
/// through a chart cast there, to `toleranceDeg`.
///
/// `request_json` is `{"instant", "bodies", "projection", "stepDeg",
/// "latitudeLimitDeg"}`: `instant` a UTC Julian day; `bodies` `graha`
/// keys, the seven and the outer planets the profile places when left
/// out; `projection` `IN_MUNDO` (the default) or `ZODIACAL` (C1155);
/// `stepDeg` above 0 and at most 10, 1 by default; `latitudeLimitDeg`
/// above 0 and below 90, 80 by default. A key the record does not read,
/// a value out of range or a body the chart does not place is
/// `INVALID_ARG`, named under `astrocartography`; a context without an
/// ephemeris is `CAPABILITY`.
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
/// use teistro_ffi::mapping::ts_chart_astrocartography;
/// use teistro_ffi::string::TsString;
///
/// let request = c"{\"instant\": 2447000.25, \"bodies\": [\"SUN\"], \"stepDeg\": 5}";
/// let mut out = TsString::empty();
/// // SAFETY: a live context, a NUL-terminated request and a writable slot.
/// let status = unsafe { ts_chart_astrocartography(&raw const ctx, request.as_ptr(), &raw mut out) };
/// assert_eq!(status, Status::Ok);
/// let answer: serde_json::Value = serde_json::from_str(&owned(out))?;
/// assert_eq!(answer["value"]["lines"][2]["angle"], "ASC");
/// # }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `request_json` NUL-terminated;
/// `out_json` valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_chart_astrocartography(
    context: *const TsContext,
    request_json: *const c_char,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |ctx| {
        in_family!("western", [ctx, request_json, out_json], {
            // SAFETY: the entry point's contract.
            let text = unsafe { crate::support::text(request_json, "request_json") }?;
            let asked = teistro::AstrocartographyRequest::from_json(text)?;
            let answer = ctx.sdk().chart().astrocartography(&asked)?;
            let json = TsString::from_string(teistro_core::envelope::canonical_json(&answer));
            // SAFETY: the entry point's contract.
            unsafe { crate::support::write_plain(out_json, "out_json", json) }
        })
    })
}
