//! Prashna at the boundary (`03-design/prashna.md`): a chart request's
//! `prashna_json` record and the `prashna` section the façade answers for it.
#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

super::chart_record!(
    "prashna",
    "prashna_json",
    "prashna",
    teistro::PrashnaRequest
);
