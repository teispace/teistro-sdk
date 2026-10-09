//! The SVG renderer at the boundary (`03-design/render-svg.md`): a chart
//! request's `theme_json` record, whose `svgs` section the façade answers.
#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

super::record!(
    "svg",
    "theme_json",
    "theme",
    Request,
    request_of,
    teistro::render_svg::Theme
);
