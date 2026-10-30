//! A stored baseline settings snapshot at the C boundary: the recording
//! engine's own fields translated into the profile and settings patch a
//! context opens with (`03-design/compat-shim.md` §4).
//!
//! The answer is one document of nested records, so it crosses as the
//! JSON `settings::recorded::Recorded` serialises to rather than as a
//! blob: a binding hands its `natal.settings` straight to a context's
//! settings and its `natal.profile` to the profile option.
#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

use core::ffi::c_char;

use teistro_core::error::{Error, Status};
use teistro_core::settings::recorded::{Snapshot, recorded};

use crate::context::TsContext;
use crate::string::TsString;
use crate::support::with_context;

/// Translates a stored baseline settings snapshot and answers with the
/// translation as canonical JSON: the natal context's profile and patch,
/// the KP context's, the muhurta context's (the natal one from the Earth's
/// centre with the true node), the ephemeris the natal chain needs at its front
/// (`SURYA_SIDDHANTA` or `null`), the locale and every field read and set
/// aside as a note (`03-design/compat-shim.md` §4.1). The context records
/// a refusal; the translation reads none of its settings.
///
/// `snapshot_json` is one object of the baseline engine's own field names,
/// `{"ayanamsaId": 1, "houseSystem": "whole-sign", ...}`, a nested record
/// under its field. A snapshot that is not an object is `INVALID_ARG` on
/// `snapshot`; a field or value the table does not know, `INVALID_ARG` on
/// `snapshot.<field>`; a value the SDK decided not to carry, such as the
/// Surya bija, `UNSUPPORTED` on `snapshot.<field>` with the crux as its
/// hint.
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
/// use teistro_ffi::recorded::ts_recorded_settings;
/// use teistro_ffi::string::TsString;
///
/// let snapshot = c"{\"ayanamsaId\": 3, \"houseSystem\": \"placidus\"}";
/// let mut out = TsString::empty();
/// // SAFETY: a live context, a NUL-terminated snapshot and a writable slot.
/// let status = unsafe { ts_recorded_settings(&raw const ctx, snapshot.as_ptr(), &raw mut out) };
/// assert_eq!(status, Status::Ok);
/// let answer: serde_json::Value = serde_json::from_str(&owned(out))?;
/// assert_eq!(answer["natal"]["profile"], "conformance-baseline");
/// assert_eq!(answer["natal"]["settings"]["houses"]["placement_system"], "PLACIDUS");
/// assert_eq!(answer["muhurta"]["settings"]["frame"]["centre"], "GEOCENTRIC");
///
/// // The Surya bija is refused, naming the field.
/// let bija = c"{\"suryaBija\": true}";
/// let mut out = TsString::empty();
/// // SAFETY: as above.
/// let status = unsafe { ts_recorded_settings(&raw const ctx, bija.as_ptr(), &raw mut out) };
/// assert_eq!(status, Status::Unsupported);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `snapshot_json` NUL-terminated;
/// `out_json` valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_recorded_settings(
    context: *const TsContext,
    snapshot_json: *const c_char,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |_| {
        // SAFETY: the entry point's contract.
        let text = unsafe { crate::support::text(snapshot_json, "snapshot_json") }?;
        let translated = recorded(&Snapshot::from_json(text)?).map_err(Error::from)?;
        let json = TsString::from_string(teistro_core::envelope::canonical_json(&translated));
        // SAFETY: the entry point's contract.
        unsafe { crate::support::write_plain(out_json, "out_json", json) }
    })
}
