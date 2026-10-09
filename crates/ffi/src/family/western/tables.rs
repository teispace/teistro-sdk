//! The Western tables a chart request asks for, read and written: the
//! aspects, the synastry and its composites, the declinations and
//! parallels, the antiscia, the midpoints, the houses, the harmonic chart
//! and the progressions.

use teistro_core::error::Error;
use teistro_idl::blob::{ColumnData, Writer};

use crate::chart::{Composed, TsMotion, TsWesternAspect, no_code, one_a_chart, point_cells};

impl TsWesternAspect {
    /// The code an aspect crosses as, or none for a member added to the
    /// non-exhaustive `WesternAspect` before it was given one here; the
    /// spelling gate (`tests/keys.rs`) fails on that member until it is.
    #[must_use]
    pub const fn of(aspect: teistro::WesternAspect) -> Option<TsWesternAspect> {
        use teistro::WesternAspect;
        Some(match aspect {
            WesternAspect::Conjunction => TsWesternAspect::Conjunction,
            WesternAspect::SemiSextile => TsWesternAspect::SemiSextile,
            WesternAspect::SemiSquare => TsWesternAspect::SemiSquare,
            WesternAspect::Sextile => TsWesternAspect::Sextile,
            WesternAspect::Square => TsWesternAspect::Square,
            WesternAspect::Trine => TsWesternAspect::Trine,
            WesternAspect::Sesquiquadrate => TsWesternAspect::Sesquiquadrate,
            WesternAspect::Quincunx => TsWesternAspect::Quincunx,
            WesternAspect::Opposition => TsWesternAspect::Opposition,
            _ => return None,
        })
    }
}

/// Every chart's progressions (`western-progressions.md`): a row a chart in
/// `progressions`, the progressed and directed planets a row a graha in
/// `progressed_grahas` and `directed_grahas` when the record named an
/// instant of life, and the contacts in `progressed_contacts`, ragged by
/// that row's count.
#[derive(Default)]
struct ProgressionColumns {
    life: Vec<f64>,
    sky: Vec<f64>,
    armc_deg: Vec<f64>,
    ascendant_deg: Vec<f64>,
    midheaven_deg: Vec<f64>,
    arc_deg: Vec<f64>,
    directed_ascendant_deg: Vec<f64>,
    directed_midheaven_deg: Vec<f64>,
    contact_count: Vec<u32>,
    contacts_asked: Vec<u8>,
    progressed_graha: Vec<u16>,
    progressed_longitude_deg: Vec<f64>,
    progressed_tropical_deg: Vec<f64>,
    progressed_speed: Vec<f64>,
    directed_graha: Vec<u16>,
    directed_longitude_deg: Vec<f64>,
    contact_life: Vec<f64>,
    contact_sky: Vec<f64>,
    contact_graha: Vec<u16>,
    contact_to_lagna: Vec<u8>,
    contact_to_graha: Vec<u16>,
    contact_angle: Vec<u16>,
    contact_motion: Vec<u8>,
}

/// `harmonics`, `harmonic_points` and `harmonic_rows`: each chart's
/// harmonic chart, its points and the points meeting in it
/// (`western-harmonics.md`).
#[derive(Default)]
struct HarmonicColumns {
    number: Vec<u16>,
    point_count: Vec<u32>,
    row_count: Vec<u32>,
    point: HarmonicPointCells,
    longitude_deg: Vec<f64>,
    house: Vec<u8>,
    first: HarmonicPointCells,
    second: HarmonicPointCells,
    apart_deg: Vec<f64>,
    multiple: Vec<u16>,
    orb_deg: Vec<f64>,
}

/// A harmonic chart's point as the boundary carries it: the angle, 0 for
/// a graha, 1 for the ascendant and 2 for the midheaven, and the graha's
/// id, 0 for an angle.
#[derive(Default)]
struct HarmonicPointCells {
    angle: Vec<u8>,
    graha: Vec<u16>,
}

impl HarmonicPointCells {
    fn push(&mut self, point: teistro::HarmonicPoint) {
        let (angle, graha) = match point {
            teistro::HarmonicPoint::Graha { graha } => (0, graha.id()),
            teistro::HarmonicPoint::Ascendant => (1, 0),
            teistro::HarmonicPoint::Midheaven => (2, 0),
        };
        self.angle.push(angle);
        self.graha.push(graha);
    }

    fn columns(&self) -> [ColumnData<'_>; 2] {
        [ColumnData::U8(&self.angle), ColumnData::U16(&self.graha)]
    }
}

impl HarmonicColumns {
    fn of(read: &[teistro::HarmonicChart], charts: usize) -> Result<HarmonicColumns, Error> {
        one_a_chart(read.len(), charts, "harmonic")?;
        let mut columns = HarmonicColumns::default();
        for one in read {
            columns.number.push(one.harmonic);
            columns.point_count.push(row_count(one.points.len())?);
            columns.row_count.push(row_count(one.rows.len())?);
            for placed in &one.points {
                columns.point.push(placed.point);
                columns.longitude_deg.push(placed.longitude_deg);
                columns.house.push(placed.house.get());
            }
            for row in &one.rows {
                columns.first.push(row.first);
                columns.second.push(row.second);
                columns.apart_deg.push(row.apart_deg);
                columns.multiple.push(row.multiple);
                columns.orb_deg.push(row.orb_deg);
            }
        }
        Ok(columns)
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "harmonics",
            self.number.len(),
            &[
                ColumnData::U16(&self.number),
                ColumnData::U32(&self.point_count),
                ColumnData::U32(&self.row_count),
            ],
        )?;
        let [angle, graha] = self.point.columns();
        writer.columns(
            "harmonic_points",
            self.longitude_deg.len(),
            &[
                angle,
                graha,
                ColumnData::F64(&self.longitude_deg),
                ColumnData::U8(&self.house),
            ],
        )?;
        let [first_angle, first_graha] = self.first.columns();
        let [second_angle, second_graha] = self.second.columns();
        writer.columns(
            "harmonic_rows",
            self.apart_deg.len(),
            &[
                first_angle,
                first_graha,
                second_angle,
                second_graha,
                ColumnData::F64(&self.apart_deg),
                ColumnData::U16(&self.multiple),
                ColumnData::F64(&self.orb_deg),
            ],
        )
    }
}

/// `western_houses`, `western_house_cusps` and `western_house_planets`:
/// each chart's Western division, its twelve cusps and its planets'
/// houses (`western-houses.md`).
#[derive(Default)]
struct WesternHouseColumns {
    system: Vec<u16>,
    ascendant_deg: Vec<f64>,
    reach_deg: Vec<f64>,
    planet_count: Vec<u32>,
    cusp_deg: Vec<f64>,
    graha: Vec<u16>,
    house: Vec<u8>,
    with_ascendant: Vec<u8>,
}

impl WesternHouseColumns {
    fn of(read: &[teistro::WesternHouses], charts: usize) -> Result<WesternHouseColumns, Error> {
        one_a_chart(read.len(), charts, "westernHouses")?;
        let mut columns = WesternHouseColumns::default();
        for one in read {
            columns.system.push(one.system.id());
            columns.ascendant_deg.push(one.frame.ascendant_deg);
            columns.reach_deg.push(one.frame.reach_deg);
            columns.planet_count.push(row_count(one.planets.len())?);
            columns.cusp_deg.extend(one.frame.cusps_deg);
            for placed in &one.planets {
                columns.graha.push(placed.graha.id());
                columns.house.push(placed.house.get());
                columns.with_ascendant.push(u8::from(placed.with_ascendant));
            }
        }
        Ok(columns)
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "western_houses",
            self.system.len(),
            &[
                ColumnData::U16(&self.system),
                ColumnData::F64(&self.ascendant_deg),
                ColumnData::F64(&self.reach_deg),
                ColumnData::U32(&self.planet_count),
            ],
        )?;
        writer.columns(
            "western_house_cusps",
            self.cusp_deg.len(),
            &[ColumnData::F64(&self.cusp_deg)],
        )?;
        writer.columns(
            "western_house_planets",
            self.graha.len(),
            &[
                ColumnData::U16(&self.graha),
                ColumnData::U8(&self.house),
                ColumnData::U8(&self.with_ascendant),
            ],
        )
    }
}

/// `synastry_davisons`: each chart's Davison birth with the synastry's
/// partner, when asked (`western-composites.md`, C248).
#[derive(Default)]
struct DavisonColumns {
    instant: Vec<f64>,
    latitude_deg: Vec<f64>,
    longitude_deg: Vec<f64>,
    altitude_m: Vec<f64>,
    utc_offset_seconds: Vec<i32>,
}

impl DavisonColumns {
    fn of(read: &[teistro::Partner], charts: usize) -> Result<DavisonColumns, Error> {
        one_a_chart(read.len(), charts, "davisons")?;
        let mut columns = DavisonColumns::default();
        for birth in read {
            columns.instant.push(birth.instant.get());
            columns.latitude_deg.push(birth.place.latitude.get());
            columns.longitude_deg.push(birth.place.longitude.get());
            columns.altitude_m.push(birth.place.altitude.get());
            columns.utc_offset_seconds.push(birth.utc_offset.seconds());
        }
        Ok(columns)
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "synastry_davisons",
            self.instant.len(),
            &[
                ColumnData::F64(&self.instant),
                ColumnData::F64(&self.latitude_deg),
                ColumnData::F64(&self.longitude_deg),
                ColumnData::F64(&self.altitude_m),
                ColumnData::I32(&self.utc_offset_seconds),
            ],
        )
    }
}

/// `midpoints` and `midpoint_rows`: each chart's planets equally distant
/// from two others (`western-midpoints.md`).
#[derive(Default)]
struct MidpointColumns {
    count: Vec<u32>,
    rows: MidpointRowColumns,
}

/// The equal distances, one chart's own or across a synastry, as the row
/// sections cross them; the synastry's carry whose pair it is besides.
#[derive(Default)]
struct MidpointRowColumns {
    first: Vec<u16>,
    second: Vec<u16>,
    middle: Vec<u16>,
    partners_pair: Vec<u8>,
    far: Vec<u8>,
    distance_deg: Vec<f64>,
    from_axis_deg: Vec<f64>,
    orb_deg: Vec<f64>,
}

impl MidpointRowColumns {
    fn push(&mut self, row: &teistro::MidpointRow) {
        self.first.push(row.first.id());
        self.second.push(row.second.id());
        self.middle.push(row.middle.id());
        self.far.push(u8::from(row.far));
        self.distance_deg.push(row.distance_deg);
        self.from_axis_deg.push(row.from_axis_deg);
        self.orb_deg.push(row.orb_deg);
    }

    fn push_across(&mut self, row: &teistro::SynastryMidpointRow) {
        self.first.push(row.first.id());
        self.second.push(row.second.id());
        self.middle.push(row.middle.id());
        self.partners_pair.push(u8::from(row.partners_pair));
        self.far.push(u8::from(row.far));
        self.distance_deg.push(row.distance_deg);
        self.from_axis_deg.push(row.from_axis_deg);
        self.orb_deg.push(row.orb_deg);
    }

    /// The rows as `section`, with `partners_pair` after `middle` when
    /// `across`.
    fn write(
        &self,
        writer: &mut Writer<'_>,
        section: &str,
        across: bool,
    ) -> Result<(), teistro_idl::blob::BlobError> {
        let mut columns = vec![
            ColumnData::U16(&self.first),
            ColumnData::U16(&self.second),
            ColumnData::U16(&self.middle),
        ];
        if across {
            columns.push(ColumnData::U8(&self.partners_pair));
        }
        columns.extend([
            ColumnData::U8(&self.far),
            ColumnData::F64(&self.distance_deg),
            ColumnData::F64(&self.from_axis_deg),
            ColumnData::F64(&self.orb_deg),
        ]);
        writer.columns(section, self.first.len(), &columns)
    }
}

impl MidpointColumns {
    fn of(read: &[Vec<teistro::MidpointRow>], charts: usize) -> Result<MidpointColumns, Error> {
        one_a_chart(read.len(), charts, "midpoints")?;
        let mut columns = MidpointColumns::default();
        for rows in read {
            columns.count.push(row_count(rows.len())?);
            for row in rows {
                columns.rows.push(row);
            }
        }
        Ok(columns)
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "midpoints",
            self.count.len(),
            &[ColumnData::U32(&self.count)],
        )?;
        self.rows.write(writer, "midpoint_rows", false)
    }
}

/// `antiscia`, `antiscion_points` and `antiscion_rows`: each chart's
/// planets reflected about the solstices and the equinoxes, and the pairs
/// standing in one (`western-antiscia.md`).
#[derive(Default)]
struct AntisciaColumns {
    point_count: Vec<u32>,
    pair_count: Vec<u32>,
    graha: Vec<u16>,
    antiscion_deg: Vec<f64>,
    contrantiscion_deg: Vec<f64>,
    paired: Vec<u8>,
    pairs: AntiscionRowColumns,
    cusp_count: Vec<u32>,
    cusp_system: Vec<u16>,
    cusp_graha: Vec<u16>,
    cusp_house: Vec<u8>,
    cusp_contrary: Vec<u8>,
}

/// The pairs in antiscion, one chart's own or across two, as the row
/// sections cross them.
#[derive(Default)]
struct AntiscionRowColumns {
    first: Vec<u16>,
    second: Vec<u16>,
    contrary: Vec<u8>,
    apart_deg: Vec<f64>,
    orb_deg: Vec<f64>,
}

impl AntiscionRowColumns {
    fn push(&mut self, row: &teistro::AntiscionRow) {
        self.first.push(row.first.id());
        self.second.push(row.second.id());
        self.contrary.push(u8::from(row.contrary));
        self.apart_deg.push(row.apart_deg);
        self.orb_deg.push(row.orb_deg);
    }

    fn write(
        &self,
        writer: &mut Writer<'_>,
        section: &str,
    ) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            section,
            self.first.len(),
            &[
                ColumnData::U16(&self.first),
                ColumnData::U16(&self.second),
                ColumnData::U8(&self.contrary),
                ColumnData::F64(&self.apart_deg),
                ColumnData::F64(&self.orb_deg),
            ],
        )
    }
}

impl AntisciaColumns {
    fn of(read: &[teistro::Antiscia], charts: usize) -> Result<AntisciaColumns, Error> {
        one_a_chart(read.len(), charts, "antiscia")?;
        let mut columns = AntisciaColumns::default();
        for one in read {
            columns.point_count.push(row_count(one.points.len())?);
            columns.pair_count.push(row_count(one.pairs.len())?);
            for point in &one.points {
                columns.graha.push(point.graha.id());
                columns.antiscion_deg.push(point.antiscion_deg);
                columns.contrantiscion_deg.push(point.contrantiscion_deg);
                columns
                    .paired
                    .push(u8::from(!one.unpaired.contains(&point.graha)));
            }
            for row in &one.pairs {
                columns.pairs.push(row);
            }
            columns.cusp_count.push(row_count(one.on_cusps.len())?);
            columns.cusp_system.push(
                one.cusp_system
                    .map_or(u16::MAX, teistro_core::catalogue::Catalogued::id),
            );
            for row in &one.on_cusps {
                columns.cusp_graha.push(row.graha.id());
                columns.cusp_house.push(row.house.get());
                columns.cusp_contrary.push(u8::from(row.contrary));
            }
        }
        Ok(columns)
    }

    /// `antiscion_cusp_rows`, the reflections upon a cusp's very degree,
    /// written after every section before it.
    fn write_cusps(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "antiscion_cusp_rows",
            self.cusp_graha.len(),
            &[
                ColumnData::U16(&self.cusp_graha),
                ColumnData::U8(&self.cusp_house),
                ColumnData::U8(&self.cusp_contrary),
            ],
        )
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "antiscia",
            self.point_count.len(),
            &[
                ColumnData::U32(&self.point_count),
                ColumnData::U32(&self.pair_count),
                ColumnData::U32(&self.cusp_count),
                ColumnData::U16(&self.cusp_system),
            ],
        )?;
        writer.columns(
            "antiscion_points",
            self.graha.len(),
            &[
                ColumnData::U16(&self.graha),
                ColumnData::F64(&self.antiscion_deg),
                ColumnData::F64(&self.contrantiscion_deg),
                ColumnData::U8(&self.paired),
            ],
        )?;
        self.pairs.write(writer, "antiscion_rows")
    }
}

/// `declinations`, `declination_rows` and `parallel_rows`: each chart's
/// distances from the equator and the parallels among its planets
/// (`western-declinations.md`).
#[derive(Default)]
struct DeclinationColumns {
    obliquity_deg: Vec<f64>,
    lagna_deg: Vec<f64>,
    midheaven_deg: Vec<f64>,
    graha_count: Vec<u32>,
    parallel_count: Vec<u32>,
    graha: Vec<u16>,
    declination_deg: Vec<f64>,
    first: Vec<u16>,
    second: Vec<u16>,
    contrary: Vec<u8>,
    apart_deg: Vec<f64>,
    orb_deg: Vec<f64>,
}

impl DeclinationColumns {
    fn of(
        declinations: &[teistro::Declinations],
        parallels: &[Vec<teistro::ParallelRow>],
        charts: usize,
    ) -> Result<DeclinationColumns, Error> {
        one_a_chart(declinations.len(), charts, "declinations")?;
        one_a_chart(parallels.len(), declinations.len(), "parallels")?;
        let mut columns = DeclinationColumns::default();
        for (read, rows) in declinations.iter().zip(parallels) {
            columns.obliquity_deg.push(read.obliquity_deg);
            columns.lagna_deg.push(read.lagna_deg);
            columns.midheaven_deg.push(read.midheaven_deg);
            columns.graha_count.push(row_count(read.grahas.len())?);
            columns.parallel_count.push(row_count(rows.len())?);
            for one in &read.grahas {
                columns.graha.push(one.graha.id());
                columns.declination_deg.push(one.declination_deg);
            }
            for row in rows {
                columns.first.push(row.first.id());
                columns.second.push(row.second.id());
                columns.contrary.push(u8::from(row.contrary));
                columns.apart_deg.push(row.apart_deg);
                columns.orb_deg.push(row.orb_deg);
            }
        }
        Ok(columns)
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "declinations",
            self.obliquity_deg.len(),
            &[
                ColumnData::F64(&self.obliquity_deg),
                ColumnData::F64(&self.lagna_deg),
                ColumnData::F64(&self.midheaven_deg),
                ColumnData::U32(&self.graha_count),
                ColumnData::U32(&self.parallel_count),
            ],
        )?;
        writer.columns(
            "declination_rows",
            self.graha.len(),
            &[
                ColumnData::U16(&self.graha),
                ColumnData::F64(&self.declination_deg),
            ],
        )?;
        writer.columns(
            "parallel_rows",
            self.first.len(),
            &[
                ColumnData::U16(&self.first),
                ColumnData::U16(&self.second),
                ColumnData::U8(&self.contrary),
                ColumnData::F64(&self.apart_deg),
                ColumnData::F64(&self.orb_deg),
            ],
        )
    }
}

/// Every chart's Western aspects (`western-aspects.md`): a row a chart in
/// `western_aspects` with its count, and the rows ragged under it in
/// `western_aspect_rows`.
#[derive(Default)]
struct WesternAspectColumns {
    count: Vec<u32>,
    first: Vec<u16>,
    second: Vec<u16>,
    measures: AspectMeasures,
    applying: Vec<u8>,
}

impl WesternAspectColumns {
    fn of(
        read: &[Vec<teistro::WesternAspectRow>],
        charts: usize,
    ) -> Result<WesternAspectColumns, Error> {
        one_a_chart(read.len(), charts, "western aspects")?;
        let mut columns = WesternAspectColumns::default();
        for rows in read {
            columns.count.push(row_count(rows.len())?);
            for row in rows {
                columns.first.push(row.first.id());
                columns.second.push(row.second.id());
                columns.measures.push(
                    row.aspect,
                    row.apart_deg,
                    row.from_exact_deg,
                    row.orb_deg,
                )?;
                columns.applying.push(u8::from(row.applying));
            }
        }
        Ok(columns)
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "western_aspects",
            self.count.len(),
            &[ColumnData::U32(&self.count)],
        )?;
        let [aspect, apart, from_exact, orb] = self.measures.columns();
        writer.columns(
            "western_aspect_rows",
            self.first.len(),
            &[
                ColumnData::U16(&self.first),
                ColumnData::U16(&self.second),
                aspect,
                apart,
                from_exact,
                orb,
                ColumnData::U8(&self.applying),
            ],
        )
    }
}

/// `synastry` and `synastry_rows`: each chart's contacts with the record's
/// partner (`western-synastry.md`); `synastry_parallels` and
/// `synastry_parallel_rows`: the parallels across the two, when asked
/// (`western-declinations.md`); `synastry_antiscia` and
/// `synastry_antiscion_rows`: the antiscia across the two, when asked
/// (`western-antiscia.md`).
#[derive(Default)]
struct SynastryColumns {
    count: Vec<u32>,
    parallel_count: Vec<u32>,
    first: PointCells,
    second: PointCells,
    measures: AspectMeasures,
    parallel_first: PointCells,
    parallel_second: PointCells,
    contrary: Vec<u8>,
    parallel_apart_deg: Vec<f64>,
    parallel_orb_deg: Vec<f64>,
    antiscion_count: Vec<u32>,
    antiscia: AntiscionRowColumns,
    midpoint_count: Vec<u32>,
    midpoints: MidpointRowColumns,
    composite_lagna_deg: Vec<f64>,
    composite_midheaven_deg: Vec<f64>,
    composite_lagna_turned: Vec<u8>,
    composite_count: Vec<u32>,
    composite_graha: Vec<u16>,
    composite_longitude_deg: Vec<f64>,
    composite_speed_deg_per_day: Vec<f64>,
    composite_cusp_count: Vec<u8>,
    composite_cusp_deg: Vec<f64>,
}

impl SynastryColumns {
    fn of(read: &[teistro::PartnerReading], charts: usize) -> Result<SynastryColumns, Error> {
        one_a_chart(read.len(), charts, "synastries")?;
        let mut columns = SynastryColumns::default();
        for one in read {
            columns.count.push(row_count(one.aspects.len())?);
            if let Some(parallels) = &one.parallels {
                columns.parallel_count.push(row_count(parallels.len())?);
            }
            for row in one.parallels.iter().flatten() {
                columns.parallel_first.push(row.first);
                columns.parallel_second.push(row.second);
                columns.contrary.push(u8::from(row.contrary));
                columns.parallel_apart_deg.push(row.apart_deg);
                columns.parallel_orb_deg.push(row.orb_deg);
            }
            if let Some(antiscia) = &one.antiscia {
                columns.antiscion_count.push(row_count(antiscia.len())?);
            }
            for row in one.antiscia.iter().flatten() {
                columns.antiscia.push(row);
            }
            if let Some(midpoints) = &one.midpoints {
                columns.midpoint_count.push(row_count(midpoints.len())?);
            }
            for row in one.midpoints.iter().flatten() {
                columns.midpoints.push_across(row);
            }
            if let Some(composite) = &one.composite {
                columns.composite_lagna_deg.push(composite.lagna_deg);
                columns
                    .composite_midheaven_deg
                    .push(composite.midheaven_deg);
                columns
                    .composite_lagna_turned
                    .push(u8::from(composite.lagna_turned));
                columns
                    .composite_count
                    .push(row_count(composite.planets.len())?);
                let cusps = composite
                    .cusps_deg
                    .as_ref()
                    .map_or(&[][..], |cusps| &cusps[..]);
                columns
                    .composite_cusp_count
                    .push(u8::from(composite.cusps_deg.is_some()) * 12);
                columns.composite_cusp_deg.extend_from_slice(cusps);
                for at in &composite.planets {
                    columns.composite_graha.push(at.graha.id());
                    columns.composite_longitude_deg.push(at.longitude_deg);
                    columns
                        .composite_speed_deg_per_day
                        .push(at.speed_deg_per_day);
                }
            }
            for row in &one.aspects {
                columns.first.push(row.first);
                columns.second.push(row.second);
                columns.measures.push(
                    row.aspect,
                    row.apart_deg,
                    row.from_exact_deg,
                    row.orb_deg,
                )?;
            }
        }
        Ok(columns)
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "synastry",
            self.count.len(),
            &[ColumnData::U32(&self.count)],
        )?;
        let [first_lagna, first_graha] = self.first.columns();
        let [second_lagna, second_graha] = self.second.columns();
        let [aspect, apart, from_exact, orb] = self.measures.columns();
        writer.columns(
            "synastry_rows",
            self.measures.aspect.len(),
            &[
                first_lagna,
                first_graha,
                second_lagna,
                second_graha,
                aspect,
                apart,
                from_exact,
                orb,
            ],
        )
    }

    /// `synastry_parallels` and `synastry_parallel_rows`, after the
    /// declinations'.
    fn write_parallels(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "synastry_parallels",
            self.parallel_count.len(),
            &[ColumnData::U32(&self.parallel_count)],
        )?;
        let [first_lagna, first_graha] = self.parallel_first.columns();
        let [second_lagna, second_graha] = self.parallel_second.columns();
        writer.columns(
            "synastry_parallel_rows",
            self.contrary.len(),
            &[
                first_lagna,
                first_graha,
                second_lagna,
                second_graha,
                ColumnData::U8(&self.contrary),
                ColumnData::F64(&self.parallel_apart_deg),
                ColumnData::F64(&self.parallel_orb_deg),
            ],
        )
    }

    /// `synastry_antiscia` and `synastry_antiscion_rows`, after the
    /// antiscia's.
    fn write_antiscia(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "synastry_antiscia",
            self.antiscion_count.len(),
            &[ColumnData::U32(&self.antiscion_count)],
        )?;
        self.antiscia.write(writer, "synastry_antiscion_rows")
    }

    /// `synastry_midpoints` and `synastry_midpoint_rows`, after the
    /// composites and the Davison births.
    fn write_midpoints(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "synastry_midpoints",
            self.midpoint_count.len(),
            &[ColumnData::U32(&self.midpoint_count)],
        )?;
        self.midpoints.write(writer, "synastry_midpoint_rows", true)
    }

    /// `synastry_composites` and `synastry_composite_rows`, after the
    /// equal distances.
    fn write_composites(
        &self,
        writer: &mut Writer<'_>,
    ) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "synastry_composites",
            self.composite_count.len(),
            &[
                ColumnData::F64(&self.composite_lagna_deg),
                ColumnData::F64(&self.composite_midheaven_deg),
                ColumnData::U8(&self.composite_lagna_turned),
                ColumnData::U32(&self.composite_count),
                ColumnData::U8(&self.composite_cusp_count),
            ],
        )?;
        writer.columns(
            "synastry_composite_rows",
            self.composite_graha.len(),
            &[
                ColumnData::U16(&self.composite_graha),
                ColumnData::F64(&self.composite_longitude_deg),
                ColumnData::F64(&self.composite_speed_deg_per_day),
            ],
        )
    }

    /// `synastry_composite_cusps`, each composite's twelve cusps when it
    /// has them, written after the Western houses.
    fn write_composite_cusps(
        &self,
        writer: &mut Writer<'_>,
    ) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "synastry_composite_cusps",
            self.composite_cusp_deg.len(),
            &[ColumnData::F64(&self.composite_cusp_deg)],
        )
    }
}

/// How many rows a chart holds in a ragged section, as its count column
/// says.
fn row_count(rows: usize) -> Result<u32, Error> {
    u32::try_from(rows).map_err(|_| Error::internal("more rows than a section can count"))
}

/// The four cells a Western aspect row measures, whichever two points
/// stand at it, in the order the schema's `western_aspect_measures` lists.
#[derive(Default)]
struct AspectMeasures {
    aspect: Vec<u8>,
    apart_deg: Vec<f64>,
    from_exact_deg: Vec<f64>,
    orb_deg: Vec<f64>,
}

impl AspectMeasures {
    fn push(
        &mut self,
        aspect: teistro::WesternAspect,
        apart_deg: f64,
        from_exact_deg: f64,
        orb_deg: f64,
    ) -> Result<(), Error> {
        let code = TsWesternAspect::of(aspect)
            .ok_or_else(|| no_code(&format!("the aspect {}", aspect.key())))?;
        self.aspect.push(code as u8);
        self.apart_deg.push(apart_deg);
        self.from_exact_deg.push(from_exact_deg);
        self.orb_deg.push(orb_deg);
        Ok(())
    }

    fn columns(&self) -> [ColumnData<'_>; 4] {
        [
            ColumnData::U8(&self.aspect),
            ColumnData::F64(&self.apart_deg),
            ColumnData::F64(&self.from_exact_deg),
            ColumnData::F64(&self.orb_deg),
        ]
    }
}

/// A natal point as two cells a row, as the schema's `point_columns`
/// names them.
#[derive(Default)]
struct PointCells {
    lagna: Vec<u8>,
    graha: Vec<u16>,
}

impl PointCells {
    fn push(&mut self, point: teistro::NatalPoint) {
        let (lagna, graha) = point_cells(Some(point));
        self.lagna.push(lagna);
        self.graha.push(graha);
    }

    fn columns(&self) -> [ColumnData<'_>; 2] {
        [ColumnData::U8(&self.lagna), ColumnData::U16(&self.graha)]
    }
}

impl ProgressionColumns {
    fn of(read: &[teistro::Progressions], charts: usize) -> Result<ProgressionColumns, Error> {
        one_a_chart(read.len(), charts, "progressions")?;
        let mut columns = ProgressionColumns::default();
        for one in read {
            let progressed = one.progressed.as_ref();
            let directed = one.directed.as_ref();
            columns
                .life
                .push(progressed.map_or(f64::NAN, |p| p.life.get()));
            columns
                .sky
                .push(progressed.map_or(f64::NAN, |p| p.sky.get()));
            columns
                .armc_deg
                .push(progressed.map_or(f64::NAN, |p| p.armc_deg));
            columns
                .ascendant_deg
                .push(progressed.map_or(f64::NAN, |p| p.angles.ascendant_deg));
            columns
                .midheaven_deg
                .push(progressed.map_or(f64::NAN, |p| p.angles.midheaven_deg));
            columns
                .arc_deg
                .push(directed.map_or(f64::NAN, |d| d.arc_deg));
            columns
                .directed_ascendant_deg
                .push(directed.map_or(f64::NAN, |d| d.ascendant_deg));
            columns
                .directed_midheaven_deg
                .push(directed.map_or(f64::NAN, |d| d.midheaven_deg));
            columns
                .contacts_asked
                .push(u8::from(one.contacts.is_some()));
            let contacts = one.contacts.as_deref().unwrap_or_default();
            columns.contact_count.push(
                u32::try_from(contacts.len())
                    .map_err(|_| Error::internal("more contacts than a section can count"))?,
            );
            let placed = progressed.map(|p| &p.chart.value.foundation);
            for at in placed
                .into_iter()
                .flat_map(|f| f.grahas.iter().chain(&f.outer))
            {
                columns.progressed_graha.push(at.graha.id());
                columns.progressed_longitude_deg.push(at.longitude_deg);
                columns.progressed_tropical_deg.push(at.tropical_deg);
                columns.progressed_speed.push(at.speed_deg_per_day);
            }
            for at in directed.map_or(&[][..], |d| d.planets.as_slice()) {
                columns.directed_graha.push(at.graha.id());
                columns.directed_longitude_deg.push(at.longitude_deg);
            }
            for contact in contacts {
                columns.contact_life.push(contact.life.get());
                columns.contact_sky.push(contact.sky.get());
                columns.contact_graha.push(contact.graha.id());
                let (to_lagna, to_graha) = point_cells(Some(contact.to));
                columns.contact_to_lagna.push(to_lagna);
                columns.contact_to_graha.push(to_graha);
                columns.contact_angle.push(contact.angle);
                columns
                    .contact_motion
                    .push(TsMotion::from(contact.motion) as u8);
            }
        }
        Ok(columns)
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "progressions",
            self.life.len(),
            &[
                ColumnData::F64(&self.life),
                ColumnData::F64(&self.sky),
                ColumnData::F64(&self.armc_deg),
                ColumnData::F64(&self.ascendant_deg),
                ColumnData::F64(&self.midheaven_deg),
                ColumnData::F64(&self.arc_deg),
                ColumnData::F64(&self.directed_ascendant_deg),
                ColumnData::F64(&self.directed_midheaven_deg),
                ColumnData::U32(&self.contact_count),
                ColumnData::U8(&self.contacts_asked),
            ],
        )?;
        writer.columns(
            "progressed_grahas",
            self.progressed_graha.len(),
            &[
                ColumnData::U16(&self.progressed_graha),
                ColumnData::F64(&self.progressed_longitude_deg),
                ColumnData::F64(&self.progressed_tropical_deg),
                ColumnData::F64(&self.progressed_speed),
            ],
        )?;
        writer.columns(
            "directed_grahas",
            self.directed_graha.len(),
            &[
                ColumnData::U16(&self.directed_graha),
                ColumnData::F64(&self.directed_longitude_deg),
            ],
        )?;
        writer.columns(
            "progressed_contacts",
            self.contact_life.len(),
            &[
                ColumnData::F64(&self.contact_life),
                ColumnData::F64(&self.contact_sky),
                ColumnData::U16(&self.contact_graha),
                ColumnData::U8(&self.contact_to_lagna),
                ColumnData::U16(&self.contact_to_graha),
                ColumnData::U16(&self.contact_angle),
                ColumnData::U8(&self.contact_motion),
            ],
        )
    }
}

/// Every Western section a batch writes: the progressions, and the tables
/// written together since their sections stand together, each chart's own
/// beside each chart's with the record's partner.
pub(crate) struct Columns {
    progressions: ProgressionColumns,
    own: WesternAspectColumns,
    across: SynastryColumns,
    declined: DeclinationColumns,
    reflected: AntisciaColumns,
    between: MidpointColumns,
    davisons: DavisonColumns,
    houses: WesternHouseColumns,
    harmonics: HarmonicColumns,
}

impl Columns {
    pub(crate) fn of(composed: &Composed<'_>, charts: usize) -> Result<Columns, Error> {
        Ok(Columns {
            progressions: ProgressionColumns::of(composed.progressions, charts)?,
            own: WesternAspectColumns::of(composed.western_aspects, charts)?,
            across: SynastryColumns::of(composed.synastry, charts)?,
            declined: DeclinationColumns::of(composed.declinations, composed.parallels, charts)?,
            reflected: AntisciaColumns::of(composed.antiscia, charts)?,
            between: MidpointColumns::of(composed.midpoints, charts)?,
            davisons: DavisonColumns::of(composed.davisons, charts)?,
            houses: WesternHouseColumns::of(composed.western_houses, charts)?,
            harmonics: HarmonicColumns::of(composed.harmonics, charts)?,
        })
    }

    pub(crate) fn write(
        &self,
        writer: &mut Writer<'_>,
    ) -> Result<(), teistro_idl::blob::BlobError> {
        self.progressions.write(writer)?;
        self.own.write(writer)?;
        self.across.write(writer)?;
        self.declined.write(writer)?;
        self.across.write_parallels(writer)?;
        self.reflected.write(writer)?;
        self.across.write_antiscia(writer)?;
        self.between.write(writer)?;
        self.across.write_composites(writer)?;
        self.davisons.write(writer)?;
        self.across.write_midpoints(writer)?;
        self.houses.write(writer)?;
        self.across.write_composite_cusps(writer)?;
        self.reflected.write_cusps(writer)?;
        self.harmonics.write(writer)
    }
}
