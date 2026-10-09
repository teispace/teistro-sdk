//! Rectification at the boundary (`03-design/rectification.md`, step 8):
//! a chart request's `rectification_json` record and the `rectification`
//! section the façade answers for it, every chart read as the birth on
//! record, on the request's own clock.
#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

super::record!(
    "rectification",
    "rectification_json",
    "rectification",
    Request,
    request_of,
    teistro::RectificationRequest
);
