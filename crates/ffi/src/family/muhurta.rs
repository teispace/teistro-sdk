//! The muhurta search at the boundary
//! (`03-design/muhurta-at-the-boundary.md`): a panchanga request's
//! `muhurta_json` record, refused by a build without the search. The
//! `muhurta` section it answers is written by the façade
//! (`AlmanacAnswer::sections`).
#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

super::record!(
    "muhurta",
    "muhurta_json",
    "muhurta",
    Request,
    request_of,
    teistro::MuhurtaRequest
);

/// `beside` with the search the record asks for, when one was sent.
#[cfg(feature = "muhurta")]
pub(crate) fn asking(
    beside: teistro::AlmanacRequest,
    asked: Option<Request>,
) -> teistro::AlmanacRequest {
    match asked {
        Some(asked) => beside.with_muhurta(asked),
        None => beside,
    }
}

/// `beside` unchanged: a build without the search holds no record.
#[cfg(not(feature = "muhurta"))]
pub(crate) fn asking(
    beside: teistro::AlmanacRequest,
    asked: Option<Request>,
) -> teistro::AlmanacRequest {
    match asked {
        None => beside,
        Some(never) => match never {},
    }
}
