//! The muhurta search at the boundary
//! (`03-design/muhurta-at-the-boundary.md`): a panchanga request's
//! `muhurta_json` record and the `muhurta` section it answers.
#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

use teistro_core::error::Error;

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

/// The `muhurta` section: the search's answer in full, empty when none
/// was asked for.
#[cfg(feature = "muhurta")]
pub(crate) fn json(answered: &mut teistro::AlmanacAnswer) -> Result<String, Error> {
    crate::panchanga::written(answered.muhurta.take(), teistro::muhurta::spelling::in_full)
}

/// The `muhurta` section of a build without the search, which no request
/// can ask for.
#[cfg(not(feature = "muhurta"))]
#[allow(
    clippy::unnecessary_wraps,
    reason = "the signature of the build with the search"
)]
pub(crate) fn json(_answered: &mut teistro::AlmanacAnswer) -> Result<String, Error> {
    Ok(String::new())
}
