//! KP at the boundary (`03-design/kp.md`): a chart request's `kp_json`
//! record, whose `kp` section the façade answers.
#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

super::record!(
    "kp",
    "kp_json",
    "kp",
    Request,
    request_of,
    teistro::KpRequest
);
