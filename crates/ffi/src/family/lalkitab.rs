//! Lal Kitab at the boundary (`03-design/lalkitab.md`): a chart request's
//! `lalkitab_json` record and the `lalkitab` section the façade answers for it.
#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

super::chart_record!(
    "lalkitab",
    "lalkitab_json",
    "lalkitab",
    teistro::LalKitabRequest
);
