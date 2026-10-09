//! The remedies at the boundary (`03-design/remedies.md`): a chart
//! request's `remedies_json` record and the `remedies` section the façade answers for it.
#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

super::chart_record!(
    "remedies",
    "remedies_json",
    "remedies",
    teistro::RemedyRequest
);
