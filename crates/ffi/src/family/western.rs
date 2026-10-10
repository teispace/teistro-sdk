//! The Western family at the boundary (`03-design/western-aspects.md` and
//! the pages beside it): the tables a chart request's Western records
//! ask for, and the sections those fill.
#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

#[cfg(feature = "western")]
mod tables;

#[cfg(feature = "western")]
pub(crate) use tables::Columns;

use super::answer;

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

/// The Western tables of a build without the family, in the shape the
/// façade's [`teistro::WesternTables`] has: none, since no request can hold
/// a Western record.
#[cfg(not(feature = "western"))]
#[derive(Default)]
pub(crate) struct Tables {
    pub(crate) progressions: Vec<Progressions>,
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
