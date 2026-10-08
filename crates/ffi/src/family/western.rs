//! The Western family at the boundary (`03-design/western-aspects.md` and
//! the pages beside it): a chart request's eight Western records, the
//! tables they ask for, and the sections those fill.
#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

#[cfg(feature = "western")]
mod tables;

#[cfg(feature = "western")]
pub(crate) use tables::{Columns, Tables, progressions_of};

use super::{answer, record};

record!(
    "western",
    "progressions_json",
    "progressions",
    ProgressionsRequest,
    progressions_request_of,
    teistro::ProgressionsRequest
);
record!(
    "western",
    "western_aspects_json",
    "westernAspects",
    AspectRequest,
    aspects_request_of,
    teistro::AspectRequest
);
record!(
    "western",
    "synastry_json",
    "synastry",
    SynastryRequest,
    synastry_request_of,
    teistro::PartnerSynastry
);
record!(
    "western",
    "parallels_json",
    "parallels",
    ParallelRequest,
    parallels_request_of,
    teistro::ParallelRequest
);
record!(
    "western",
    "antiscia_json",
    "antiscia",
    AntisciaRequest,
    antiscia_request_of,
    teistro::AntisciaRequest
);
record!(
    "western",
    "midpoints_json",
    "midpoints",
    MidpointRequest,
    midpoints_request_of,
    teistro::MidpointRequest
);
record!(
    "western",
    "western_houses_json",
    "westernHouses",
    HouseRequest,
    houses_request_of,
    teistro::HouseRequest
);
record!(
    "western",
    "harmonic_json",
    "harmonic",
    HarmonicRequest,
    harmonic_request_of,
    teistro::HarmonicRequest
);

/// The Western records a chart request sends, each read by the façade's
/// own reader and refused, naming its root, by a build without the family.
#[derive(Default)]
#[cfg_attr(
    not(feature = "western"),
    allow(dead_code, reason = "a build without the family holds none to read")
)]
pub(crate) struct Records {
    pub(crate) progressions: Option<ProgressionsRequest>,
    pub(crate) western_aspects: Option<AspectRequest>,
    pub(crate) synastry: Option<SynastryRequest>,
    pub(crate) parallels: Option<ParallelRequest>,
    pub(crate) antiscia: Option<AntisciaRequest>,
    pub(crate) midpoints: Option<MidpointRequest>,
    pub(crate) western_houses: Option<HouseRequest>,
    pub(crate) harmonic: Option<HarmonicRequest>,
}

impl Records {
    /// Every Western record `asked` sends.
    ///
    /// # Safety
    ///
    /// Each of `asked`'s Western fields null or a NUL-terminated string.
    pub(crate) unsafe fn of(
        asked: &crate::chart::TsChartRequest,
    ) -> Result<Records, teistro_core::error::Error> {
        // SAFETY: the caller's contract, for every field read below.
        unsafe {
            Ok(Records {
                progressions: progressions_request_of(asked.progressions_json)?,
                western_aspects: aspects_request_of(asked.western_aspects_json)?,
                synastry: synastry_request_of(asked.synastry_json)?,
                parallels: parallels_request_of(asked.parallels_json)?,
                antiscia: antiscia_request_of(asked.antiscia_json)?,
                midpoints: midpoints_request_of(asked.midpoints_json)?,
                western_houses: houses_request_of(asked.western_houses_json)?,
                harmonic: harmonic_request_of(asked.harmonic_json)?,
            })
        }
    }
}

answer!("western", Progressions, teistro::Progressions);
answer!("western", AspectRows, Vec<teistro::WesternAspectRow>);
answer!("western", PartnerReading, teistro::PartnerReading);
answer!("western", Declinations, teistro::Declinations);
answer!("western", ParallelRows, Vec<teistro::ParallelRow>);
answer!("western", Antiscia, teistro::Antiscia);
answer!("western", MidpointRows, Vec<teistro::MidpointRow>);
answer!("western", WesternHouses, teistro::WesternHouses);
answer!("western", HarmonicChart, teistro::HarmonicChart);

/// The sections the Western tables fill: written empty by a build without
/// the family, as a request that asked for none of them writes them. A
/// name missing here is a section `Writer::finish` refuses as never
/// written, so a chart call in such a build checks the list.
#[cfg(not(feature = "western"))]
const SECTIONS: [&str; 33] = [
    "progressions",
    "progressed_grahas",
    "directed_grahas",
    "progressed_contacts",
    "western_aspects",
    "western_aspect_rows",
    "synastry",
    "synastry_rows",
    "declinations",
    "declination_rows",
    "parallel_rows",
    "synastry_parallels",
    "synastry_parallel_rows",
    "antiscia",
    "antiscion_points",
    "antiscion_rows",
    "antiscion_cusp_rows",
    "synastry_antiscia",
    "synastry_antiscion_rows",
    "midpoints",
    "midpoint_rows",
    "synastry_composites",
    "synastry_composite_rows",
    "synastry_davisons",
    "synastry_midpoints",
    "synastry_midpoint_rows",
    "western_houses",
    "western_house_cusps",
    "western_house_planets",
    "synastry_composite_cusps",
    "harmonics",
    "harmonic_points",
    "harmonic_rows",
];

/// The Western tables of a build without the family: none, since no
/// request can hold a Western record.
#[cfg(not(feature = "western"))]
#[derive(Default)]
pub(crate) struct Tables {
    pub(crate) aspects: Vec<AspectRows>,
    pub(crate) synastry: Vec<PartnerReading>,
    pub(crate) declinations: Vec<Declinations>,
    pub(crate) parallels: Vec<ParallelRows>,
    pub(crate) antiscia: Vec<Antiscia>,
    pub(crate) midpoints: Vec<MidpointRows>,
    pub(crate) davisons: Vec<teistro::Partner>,
    pub(crate) houses: Vec<WesternHouses>,
    pub(crate) harmonics: Vec<HarmonicChart>,
}

#[cfg(not(feature = "western"))]
impl Tables {
    #[allow(
        clippy::unnecessary_wraps,
        reason = "the signature of the build with the family"
    )]
    pub(crate) fn of(
        _sdk: &teistro::Context,
        _documents: &[teistro_serial::Document],
        _records: &Records,
        _clock: teistro_core::time::UtcOffset,
    ) -> Result<Tables, teistro_core::error::Error> {
        Ok(Tables::default())
    }
}

/// No progressions in a build without the family.
#[cfg(not(feature = "western"))]
#[allow(
    clippy::unnecessary_wraps,
    reason = "the signature of the build with the family"
)]
pub(crate) fn progressions_of(
    _sdk: &teistro::Context,
    _documents: &[teistro_serial::Document],
    asked: Option<&ProgressionsRequest>,
    _request: &teistro::ChartRequest,
) -> Result<Vec<Progressions>, teistro_core::error::Error> {
    match asked {
        None => Ok(Vec::new()),
        Some(never) => match *never {},
    }
}

/// The Western sections of a build without the family, every one empty.
#[cfg(not(feature = "western"))]
pub(crate) struct Columns;

#[cfg(not(feature = "western"))]
impl Columns {
    #[allow(
        clippy::unnecessary_wraps,
        reason = "the signature of the build with the family"
    )]
    pub(crate) fn of(
        _composed: &crate::chart::Composed<'_>,
        _charts: usize,
    ) -> Result<Columns, teistro_core::error::Error> {
        Ok(Columns)
    }

    #[allow(
        clippy::unused_self,
        reason = "the signature of the build with the family"
    )]
    pub(crate) fn write(
        &self,
        writer: &mut teistro_idl::blob::Writer<'_>,
    ) -> Result<(), teistro_idl::blob::BlobError> {
        SECTIONS.iter().try_for_each(|name| writer.empty(name))
    }
}
