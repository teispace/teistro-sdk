//! The remedies at the boundary (`03-design/remedies.md`): a chart
//! request's `remedies_json` record and the `remedies` section it answers.
#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

super::chart_record!(
    "remedies",
    "remedies_json",
    "remedies",
    teistro::RemedyRequest,
    remedies
);

/// Whether the record reads the remedies at an instant, which reads the
/// Vimśottarī daśā running then, so the charts must carry it.
#[cfg(feature = "remedies")]
pub(crate) fn at_an_instant(asked: Option<&Request>) -> bool {
    asked.is_some_and(|asked| asked.at.is_some())
}

/// No record can be sent to a build without the remedies.
#[cfg(not(feature = "remedies"))]
pub(crate) const fn at_an_instant(asked: Option<&Request>) -> bool {
    match asked {
        None => false,
        Some(never) => match *never {},
    }
}
