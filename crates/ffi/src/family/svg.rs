//! The SVG renderer at the boundary (`03-design/render-svg.md`): a chart
//! request's `theme_json` record and the `svgs` section it answers.
#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

use teistro_core::error::Error;
use teistro_serial::Document;

super::record!(
    "svg",
    "theme_json",
    "theme",
    Request,
    request_of,
    teistro::render_svg::Theme
);

/// Every chart's drawings written as SVG in one theme, as the canonical
/// JSON the `svgs` section carries: one array of strings per chart, in the
/// order the drawings were asked for, or nothing when no theme was sent.
#[cfg(feature = "svg")]
pub(crate) fn json(
    sdk: &teistro::Context,
    documents: &[Document],
    theme: Option<&Request>,
) -> Result<String, Error> {
    let Some(theme) = theme else {
        return Ok(String::new());
    };
    let mut per_chart = Vec::with_capacity(documents.len());
    for document in documents {
        let mut svgs = Vec::with_capacity(document.drawings.len());
        for index in 0..document.drawings.len() {
            svgs.push(sdk.chart().svg(document, index, theme)?);
        }
        per_chart.push(svgs);
    }
    Ok(teistro_core::envelope::canonical_json(&per_chart))
}

/// The `svgs` section of a build without the renderer, which no request
/// can ask for.
#[cfg(not(feature = "svg"))]
#[allow(
    clippy::unnecessary_wraps,
    reason = "the signature of the build with the renderer"
)]
pub(crate) fn json(
    _sdk: &teistro::Context,
    _documents: &[Document],
    theme: Option<&Request>,
) -> Result<String, Error> {
    Ok(super::unasked(theme))
}
