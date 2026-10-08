//! Tajika at the boundary (`03-design/annual-chart.md`,
//! `03-design/tajika-sahams.md`): a chart request's `varsha_json` record,
//! the years it founds and the sections they fill.
#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

#[cfg(feature = "tajika")]
mod years;

#[cfg(feature = "tajika")]
pub(crate) use years::{PraveshaColumns, praveshas_of};

super::record!(
    "tajika",
    "varsha_json",
    "varsha",
    Request,
    request_of,
    teistro::VarshaRequest
);

super::answer!("tajika", Varsha, teistro::Varsha);

/// The sections the years fill: written empty by a build without Tajika,
/// as a request that founded no years writes them. A name missing here is
/// a section `Writer::finish` refuses as never written, so a chart call
/// in such a build checks the list.
#[cfg(not(feature = "tajika"))]
const SECTIONS: [&str; 15] = [
    "praveshas",
    "annual_charts",
    "year_sahams",
    "year_saham_seven",
    "year_harsha",
    "year_matters",
    "matter_legs",
    "matter_yogas",
    "year_yogas",
    "year_claims",
    "natal_sahams",
    "natal_saham_seven",
    "year_dashas",
    "year_dasha_shares",
    "year_dasha_periods",
];

/// The years' sections in a build without Tajika: no chart holds a year.
#[cfg(not(feature = "tajika"))]
#[derive(Default)]
pub(crate) struct PraveshaColumns {
    /// How many years each chart holds: none.
    pub(crate) counts: Vec<u32>,
    /// How many natal sahams each chart holds: none.
    pub(crate) natal_counts: Vec<u32>,
}

#[cfg(not(feature = "tajika"))]
impl PraveshaColumns {
    #[allow(
        clippy::unnecessary_wraps,
        reason = "the signature of the build with Tajika"
    )]
    pub(crate) fn of(varsha: &[Varsha]) -> Result<PraveshaColumns, teistro_core::error::Error> {
        match varsha.first() {
            None => Ok(PraveshaColumns::default()),
            Some(never) => match *never {},
        }
    }

    #[allow(clippy::unused_self, reason = "the signature of the build with Tajika")]
    pub(crate) fn write(
        &self,
        writer: &mut teistro_idl::blob::Writer<'_>,
    ) -> Result<(), teistro_idl::blob::BlobError> {
        SECTIONS.iter().try_for_each(|name| writer.empty(name))
    }
}

/// No years in a build without Tajika, which holds no varsha record.
#[cfg(not(feature = "tajika"))]
#[allow(
    clippy::unnecessary_wraps,
    reason = "the signature of the build with Tajika"
)]
pub(crate) fn praveshas_of(
    _sdk: &teistro::Context,
    _documents: &[teistro_serial::Document],
    _birth_clock: teistro::UtcOffset,
    asked: Option<&Request>,
) -> Result<Vec<Varsha>, teistro_core::error::Error> {
    match asked {
        None => Ok(Vec::new()),
        Some(never) => match *never {},
    }
}
