//! Mundane charts at the C boundary (`03-design/mundane-charts.md`): the
//! moment a mundane chart is cast for, found by the SDK's own searches,
//! answered as canonical JSON. A binding founds the chart itself from the
//! moment, with its own chart reader, as `chart.found` does.
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

/// **The moment of a mundane chart** (`03-design/mundane-charts.md`): an
/// ingress, the lunation before one, an eclipse, a great conjunction or
/// an event on record, and the place it is cast for, as `{value,
/// provenance}` canonical JSON sealed by the chart cast there, with the
/// request as its input.
///
/// `value` is `{kind, instant, latitudeDeg, longitudeDeg, altitudeM,
/// utcOffsetSeconds, ingress, solarEclipse, lunarEclipse, name,
/// source}`: `kind` a `mundane_kind` key, `instant` the moment as a UTC
/// Julian day; `ingress` `{kind, instant}` the ingress a lunation
/// precedes; `solarEclipse` or `lunarEclipse` an eclipse's kind; `name`
/// and `source` an event's, as the record gave them. A field a kind does
/// not carry is absent.
///
/// `request_json` is `{"kind", "year", "from", "ingress", "event",
/// "latitudeDeg", "longitudeDeg", "altitudeM", "utcOffsetSeconds" or "zone"}`:
/// `kind` a `mundane_kind` key; `year` (Gregorian) or `from` (a UTC
/// Julian day) where the search starts, the first ingress, eclipse or
/// conjunction at or after it; `ingress` the ingress a lunation precedes,
/// `MESHA_SANKRANTI` by default (C1151); `event` `{"name", "instant",
/// "source"}` for an `EVENT`, which reads neither `year` nor `from`. The
/// sankrantis are read in the context's own zodiac, the four tropical
/// ingresses in the tropical one. A key the record does not read, a
/// missing or extra `year`, `from`, `ingress` or `event`, or a place out
/// of range is `INVALID_ARG`, named under `mundane`; an eclipse over a
/// classical astronomy is `UNSUPPORTED` naming `kind`; a context without
/// an ephemeris is `CAPABILITY`.
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
/// use teistro_ffi::mundane::ts_chart_mundane;
/// use teistro_ffi::string::TsString;
///
/// let request = c"{\"kind\": \"ARIES_INGRESS\", \"year\": 2024, \"latitudeDeg\": 51.5, \"longitudeDeg\": -0.12}";
/// let mut out = TsString::empty();
/// // SAFETY: a live context, a NUL-terminated request and a writable slot.
/// let status = unsafe { ts_chart_mundane(&raw const ctx, request.as_ptr(), &raw mut out) };
/// assert_eq!(status, Status::Ok);
/// let answer: serde_json::Value = serde_json::from_str(&owned(out))?;
/// // USNO: 20 March 2024, 03:06 UTC.
/// assert!((answer["value"]["instant"].as_f64().unwrap() - 2_460_389.629_2).abs() < 2.0 / 1440.0);
/// # }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `request_json` NUL-terminated;
/// `out_json` valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_chart_mundane(
    context: *const TsContext,
    request_json: *const c_char,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |ctx| {
        in_family!("chart", [ctx, request_json, out_json], {
            // SAFETY: the entry point's contract.
            let text = unsafe { crate::support::text(request_json, "request_json") }?;
            let asked = teistro::MundaneRequest::from_json(text)?;
            let answer = ctx.sdk().chart().mundane_moment(&asked)?;
            let json = TsString::from_string(teistro_core::envelope::canonical_json(&answer));
            // SAFETY: the entry point's contract.
            unsafe { crate::support::write_plain(out_json, "out_json", json) }
        })
    })
}
