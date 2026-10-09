//! Rectification at the boundary (`03-design/rectification.md`, step 8):
//! a chart request's `rectification_json` record and the `rectification`
//! section it answers, every chart read as the birth on record, on the
//! request's own clock.
#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

use teistro_core::time::UtcOffset;

super::record!(
    "rectification",
    "rectification_json",
    "rectification",
    Request,
    request_of,
    teistro::RectificationRequest
);

/// Every chart's rectification, as the canonical JSON its section carries:
/// the record's readings around each chart's instant, on `clock`.
#[cfg(feature = "rectification")]
pub(crate) fn json(
    sdk: &teistro::Context,
    documents: &[teistro_serial::Document],
    asked: Option<&Request>,
    clock: UtcOffset,
) -> Result<String, teistro_core::error::Error> {
    super::each_json(documents, asked, |document, asked| {
        sdk.chart().rectification(document, clock, asked)
    })
}

/// The `rectification` section of a build without `rectification`, which
/// no request can ask for.
#[cfg(not(feature = "rectification"))]
#[allow(
    clippy::unnecessary_wraps,
    reason = "the signature of the build with the family"
)]
pub(crate) fn json(
    _sdk: &teistro::Context,
    _documents: &[teistro_serial::Document],
    asked: Option<&Request>,
    _clock: UtcOffset,
) -> Result<String, teistro_core::error::Error> {
    Ok(super::unasked(asked))
}
