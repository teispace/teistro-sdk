//! KP at the boundary (`03-design/kp.md`): a chart request's `kp_json`
//! record and the `kp` section it answers.
#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

use core::ffi::c_char;

use teistro_core::error::Error;
use teistro_core::time::UtcOffset;
use teistro_serial::Document;

/// The record `kp_json` carries.
#[cfg(feature = "kp")]
pub(crate) type Request = teistro::KpRequest;
/// The record `kp_json` carries, which this build cannot hold.
#[cfg(not(feature = "kp"))]
pub(crate) type Request = super::Absent;

/// The KP reading a request's `kp_json` asks for, none for null; the
/// façade reads the record ([`teistro::KpRequest::from_json`]), naming a
/// refusal from its root, `kp.number`. A record naming no clock takes the
/// chart request's own, which is the clock the charts were asked on.
///
/// # Safety
///
/// `kp_json` null or a NUL-terminated string.
#[cfg(feature = "kp")]
pub(crate) unsafe fn request_of(
    kp_json: *const c_char,
    clock: UtcOffset,
) -> Result<Option<Request>, Error> {
    // SAFETY: the caller's contract.
    let Some(text) = unsafe { crate::support::optional_text(kp_json, "kp_json") }? else {
        return Ok(None);
    };
    let request = teistro::KpRequest::from_json(text)?;
    Ok(Some(match request.clock() {
        Some(_) => request,
        None => request.on_clock(clock),
    }))
}

/// A `kp_json` record, refused in a build without KP.
///
/// # Safety
///
/// None needed: the pointer is only compared with null.
#[cfg(not(feature = "kp"))]
pub(crate) unsafe fn request_of(
    kp_json: *const c_char,
    _clock: UtcOffset,
) -> Result<Option<Request>, Error> {
    super::refused_if_sent(kp_json, "kp", "kp")
}

/// Every chart's KP reading as the canonical JSON the `kp` section carries:
/// an array with one reading a chart, or nothing at all when none was asked
/// for, as `drawings` is.
#[cfg(feature = "kp")]
pub(crate) fn json(
    sdk: &teistro::Context,
    documents: &[Document],
    asked: Option<&Request>,
) -> Result<String, Error> {
    crate::family::each_json(documents, asked, |document, asked| {
        sdk.chart().kp_reading(document, asked)
    })
}

/// The `kp` section of a build without KP, which no request can ask for.
#[cfg(not(feature = "kp"))]
#[allow(
    clippy::unnecessary_wraps,
    reason = "the signature of the build with KP"
)]
pub(crate) fn json(
    _sdk: &teistro::Context,
    _documents: &[Document],
    asked: Option<&Request>,
) -> Result<String, Error> {
    Ok(super::unasked(asked))
}
