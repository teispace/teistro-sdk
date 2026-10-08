//! The years a varsha record founds, and the sections they fill: the
//! returns, their annual charts and dashas, and the births' own sahams.

use teistro_core::error::{Error, Status};
use teistro_idl::blob::{ColumnData, Writer};
use teistro_serial::Document;

use crate::chart::{
    PeriodColumns, TsAffliction, TsHarshaGrade, TsSaham, TsSahamStrong, TsSahamWeak,
    TsTajikaDrishti, TsTajikaRelation, TsTajikaYoga, TsVarsheshaChosen, TsYearYoga, graha_mask,
    graha_or_absent,
};
use crate::schemas::SignedBy;

/// Every year's annual dashas (`03-design/annual-dashas.md`): a row a
/// year a system, ragged by `annual_charts.dasha_count`, with each one's
/// ring and periods ragged under it.
#[derive(Default)]
struct YearDashaColumns {
    system: Vec<u16>,
    seeded: Vec<u8>,
    seed: Vec<u16>,
    first: Vec<u8>,
    remaining: Vec<f64>,
    from: Vec<f64>,
    to: Vec<f64>,
    share_count: Vec<u8>,
    period_count: Vec<u32>,
    /// The `year_dasha_shares` section, ragged by `share_count`.
    shares: ShareColumns,
    /// The `year_dasha_periods` section, ragged by `period_count`.
    periods: PeriodColumns,
}

/// The lords a year's dasha runs round, each with its weight.
#[derive(Default)]
struct ShareColumns {
    lord: Vec<u16>,
    has_sign: Vec<u8>,
    sign: Vec<u16>,
    weight: Vec<f64>,
}

impl YearDashaColumns {
    fn push(&mut self, dasha: &teistro::AnnualDasha) {
        let ring = &dasha.ring;
        self.system.push(dasha.system.id());
        self.seeded.push(u8::from(dasha.seed.is_some()));
        self.seed
            .push(dasha.seed.map_or(0, teistro_core::catalogue::Nakshatra::id));
        self.first.push(u8::try_from(ring.first).unwrap_or(u8::MAX));
        self.remaining.push(ring.remaining.unwrap_or(f64::NAN));
        self.from.push(dasha.year.from.get());
        self.to.push(dasha.year.to.get());
        self.share_count
            .push(u8::try_from(ring.ring.len()).unwrap_or(u8::MAX));
        self.period_count
            .push(u32::try_from(dasha.periods.len()).unwrap_or(u32::MAX));
        for share in &ring.ring {
            self.shares.lord.push(share.lord.id());
            self.shares.has_sign.push(u8::from(share.sign.is_some()));
            self.shares
                .sign
                .push(share.sign.map_or(0, teistro_core::catalogue::Rashi::id));
            self.shares.weight.push(share.weight);
        }
        for period in &dasha.periods {
            self.periods.push(period);
        }
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "year_dashas",
            self.system.len(),
            &[
                ColumnData::U16(&self.system),
                ColumnData::U8(&self.seeded),
                ColumnData::U16(&self.seed),
                ColumnData::U8(&self.first),
                ColumnData::F64(&self.remaining),
                ColumnData::F64(&self.from),
                ColumnData::F64(&self.to),
                ColumnData::U8(&self.share_count),
                ColumnData::U32(&self.period_count),
            ],
        )?;
        writer.columns(
            "year_dasha_shares",
            self.shares.lord.len(),
            &[
                ColumnData::U16(&self.shares.lord),
                ColumnData::U8(&self.shares.has_sign),
                ColumnData::U16(&self.shares.sign),
                ColumnData::F64(&self.shares.weight),
            ],
        )?;
        self.periods
            .write(writer, "year_dasha_periods", SignedBy::Period)
    }
}

/// Every chart's annual-chart instants, concatenated and ragged.
///
/// The count is per chart and not per batch for a reason the pass
/// measured: the request settles how many returns are *wanted* and the
/// ephemeris settles how many there *are*
/// (`03-design/annual-chart-measured.md`).
pub(crate) struct PraveshaColumns {
    pub(crate) counts: Vec<u32>,
    years: Vec<u16>,
    jds: Vec<f64>,
    muntha_signs: Vec<u16>,
    muntha_lords: Vec<u16>,
    muntha_degs: Vec<f64>,
    /// The `annual_charts` section, one row per return or none at all.
    annual: AnnualColumns,
    /// How many of `natal` each chart holds: `cast.natal_saham_count`.
    pub(crate) natal_counts: Vec<u32>,
    /// The births' own sahams, the `natal_sahams` section.
    natal: SahamColumns,
}

/// The years' own charts, parallel to `praveshas` row for row when a place
/// was asked for and empty when none was.
///
/// All or none, never some: every year of a batch is founded at the one
/// place the request named, so a year with no chart is a year whose
/// founding failed, and that is refused rather than written as a gap.
#[derive(Default)]
struct AnnualColumns {
    lagnas: Vec<f64>,
    daylight: Vec<u8>,
    janma_lagna: Vec<u16>,
    varsha_lagna: Vec<u16>,
    tri_rashi: Vec<u16>,
    dina_ratri: Vec<u16>,
    year_lord: Vec<u16>,
    chosen: Vec<u8>,
    year_lord_vishwa: Vec<i32>,
    moon_passed_over: Vec<u8>,
    claim_counts: Vec<u8>,
    yoga_counts: Vec<u8>,
    retrograde: Vec<u8>,
    combust: Vec<u8>,
    matter_counts: Vec<u8>,
    /// The `year_claims` section: every year's claimants concatenated, in
    /// the order each year ranks them, ragged by `claim_count`.
    claims: ClaimColumns,
    /// The `year_yogas` section: every year's yoga-making pairs, ragged by
    /// `yoga_count`.
    yogas: YogaColumns,
    /// The `year_matters` section and the two under it, ragged by
    /// `matter_count`.
    matters: MatterColumns,
    saham_counts: Vec<u8>,
    /// The `year_sahams` section, ragged by `saham_count`, and the seven
    /// rows under each.
    sahams: SahamColumns,
    /// The `year_harsha` section, seven rows a year.
    harsha: HarshaColumns,
    dasha_counts: Vec<u8>,
    /// The `year_dashas` section and the two under it, ragged by
    /// `dasha_count`; written after the births' sahams, whose ids come
    /// first.
    dashas: YearDashaColumns,
}

/// A chart's sahams, flat and ragged, each where it fell, what it fell in
/// and its strength clause by clause, with the seven planets' facts under
/// each (`03-design/tajika-saham-strength.md`).
///
/// One shape for the years' sahams and the births', so each binding
/// decodes a saham in one place.
#[derive(Default)]
struct SahamColumns {
    saham: Vec<u8>,
    longitude_deg: Vec<f64>,
    sign: Vec<u16>,
    lord: Vec<u16>,
    house: Vec<u8>,
    added_sign: Vec<u8>,
    strong: Vec<u16>,
    weak: Vec<u8>,
    lord_vishwa: Vec<i32>,
    lord_harsha: Vec<u8>,
    node_axis: Vec<u8>,
    /// Seven rows under each saham, the catalogue's order.
    seven_graha: Vec<u16>,
    seven_drishti: Vec<u8>,
    seven_relation: Vec<u8>,
    seven_company: Vec<u8>,
}

/// What a saham row's `node_axis` holds when the chart placed no nodes.
const NODE_AXIS_UNREAD: u8 = 2;

impl SahamColumns {
    fn push(&mut self, one: &teistro::SahamStrength) {
        let place = &one.place;
        self.saham.push(TsSaham::from(one.saham) as u8);
        self.longitude_deg.push(place.longitude_deg);
        self.sign.push(place.sign.id());
        self.lord.push(place.lord.id());
        self.house.push(place.house.get());
        self.added_sign.push(u8::from(place.added_sign));
        self.strong
            .push(one.strong().iter().fold(0, |bits, (clause, holds)| {
                if *holds {
                    bits | 1_u16 << TsSahamStrong::from(*clause) as u8
                } else {
                    bits
                }
            }));
        self.weak
            .push(one.weak().iter().fold(0, |bits, (clause, holds)| {
                if *holds {
                    bits | 1_u8 << TsSahamWeak::from(*clause) as u8
                } else {
                    bits
                }
            }));
        self.lord_vishwa.push(sub_sub(one.lord_vishwa));
        self.lord_harsha
            .push(TsHarshaGrade::from(one.lord_harsha) as u8);
        self.node_axis
            .push(one.in_node_axis.map_or(NODE_AXIS_UNREAD, u8::from));
        for (at, graha) in teistro::tajika::SEVEN.iter().enumerate() {
            self.seven_graha.push(graha.id());
            self.seven_drishti.push(
                one.aspects
                    .get(at)
                    .map_or(TsTajikaDrishti::None, |drishti| {
                        TsTajikaDrishti::from(*drishti)
                    }) as u8,
            );
            self.seven_relation.push(
                one.relations
                    .get(at)
                    .map_or(TsTajikaRelation::Neutral, |relation| {
                        TsTajikaRelation::from(*relation)
                    }) as u8,
            );
            self.seven_company
                .push(u8::from(one.company.get(at).copied().unwrap_or(false)));
        }
    }

    /// The sahams under `name`, and their seven rows under `seven`.
    fn write(
        &self,
        writer: &mut Writer<'_>,
        name: &str,
        seven: &str,
    ) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            name,
            self.saham.len(),
            &[
                ColumnData::U8(&self.saham),
                ColumnData::F64(&self.longitude_deg),
                ColumnData::U16(&self.sign),
                ColumnData::U16(&self.lord),
                ColumnData::U8(&self.house),
                ColumnData::U8(&self.added_sign),
                ColumnData::U16(&self.strong),
                ColumnData::U8(&self.weak),
                ColumnData::I32(&self.lord_vishwa),
                ColumnData::U8(&self.lord_harsha),
                ColumnData::U8(&self.node_axis),
            ],
        )?;
        writer.columns(
            seven,
            self.seven_graha.len(),
            &[
                ColumnData::U16(&self.seven_graha),
                ColumnData::U8(&self.seven_drishti),
                ColumnData::U8(&self.seven_relation),
                ColumnData::U8(&self.seven_company),
            ],
        )
    }
}

/// Every founded year's Harsha bala: seven rows each, the catalogue's
/// order (`03-design/tajika-harsha.md`).
#[derive(Default)]
struct HarshaColumns {
    graha: Vec<u16>,
    house: Vec<u8>,
    sthana: Vec<u8>,
    uchcha_swakshetra: Vec<u8>,
    stri_purusha: Vec<u8>,
    dina_ratri: Vec<u8>,
    total: Vec<u8>,
    grade: Vec<u8>,
}

impl HarshaColumns {
    fn push(&mut self, seven: &[teistro::Harsha; 7]) {
        for one in seven {
            self.graha.push(one.graha.id());
            self.house.push(one.house.get());
            self.sthana.push(u8::from(one.sthana));
            self.uchcha_swakshetra.push(u8::from(one.uchcha_swakshetra));
            self.stri_purusha.push(u8::from(one.stri_purusha));
            self.dina_ratri.push(u8::from(one.dina_ratri));
            self.total
                .push(u8::try_from(one.total.units()).unwrap_or(u8::MAX));
            self.grade.push(TsHarshaGrade::from(one.grade) as u8);
        }
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "year_harsha",
            self.graha.len(),
            &[
                ColumnData::U16(&self.graha),
                ColumnData::U8(&self.house),
                ColumnData::U8(&self.sthana),
                ColumnData::U8(&self.uchcha_swakshetra),
                ColumnData::U8(&self.stri_purusha),
                ColumnData::U8(&self.dina_ratri),
                ColumnData::U8(&self.total),
                ColumnData::U8(&self.grade),
            ],
        )
    }
}

/// How two planets stand, as the matter sections carry it: `year_yogas`'
/// own columns and a presence flag on the yoga, because a pair the matter
/// sections carry may make none.
///
/// One shape for the matter's own pair and for every leg, so each binding
/// decodes a pair in one place.
#[derive(Default)]
struct PairColumns {
    faster: Vec<u16>,
    slower: Vec<u16>,
    drishti: Vec<u8>,
    yoga: Vec<u8>,
    yoga_present: Vec<u8>,
    orb_deg: Vec<f64>,
    apart_deg: Vec<f64>,
}

impl PairColumns {
    fn push(&mut self, pair: Option<&teistro::Between>) {
        let Some(pair) = pair else {
            // Absent: a row of noughts, read only when the section's own
            // flag says the pair is there.
            self.faster.push(0);
            self.slower.push(0);
            self.drishti.push(0);
            self.yoga.push(0);
            self.yoga_present.push(0);
            self.orb_deg.push(0.0);
            self.apart_deg.push(0.0);
            return;
        };
        self.faster.push(pair.faster.id());
        self.slower.push(pair.slower.id());
        self.drishti.push(TsTajikaDrishti::from(pair.drishti) as u8);
        self.yoga
            .push(pair.yoga.map_or(0, |yoga| TsTajikaYoga::from(yoga) as u8));
        self.yoga_present.push(u8::from(pair.yoga.is_some()));
        self.orb_deg.push(pair.orb_deg);
        self.apart_deg.push(pair.apart_deg);
    }

    /// The seven columns, in the order every pair-carrying section
    /// declares them.
    fn data(&self) -> [ColumnData<'_>; 7] {
        [
            ColumnData::U16(&self.faster),
            ColumnData::U16(&self.slower),
            ColumnData::U8(&self.drishti),
            ColumnData::U8(&self.yoga),
            ColumnData::U8(&self.yoga_present),
            ColumnData::F64(&self.orb_deg),
            ColumnData::F64(&self.apart_deg),
        ]
    }
}

/// Every year's matters, flat and ragged, with the yogas each holds and
/// the legs each of those stands on.
#[derive(Default)]
struct MatterColumns {
    house: Vec<u8>,
    sign: Vec<u16>,
    lagnesha: Vec<u16>,
    karyesha: Vec<u16>,
    same_lord: Vec<u8>,
    /// The lords' own relation, present unless `same_lord`.
    pair: PairColumns,
    unanswered: Vec<u16>,
    held_count: Vec<u8>,
    held: HeldColumns,
}

/// Every matter's held yogas, flat and ragged by `held_count`.
#[derive(Default)]
struct HeldColumns {
    yoga: Vec<u8>,
    by_pair: Vec<u8>,
    through: Vec<u16>,
    through_present: Vec<u8>,
    entering: Vec<u16>,
    entering_present: Vec<u8>,
    afflictions_present: Vec<u8>,
    lagnesha_afflictions: Vec<u8>,
    karyesha_afflictions: Vec<u8>,
    leg_count: Vec<u8>,
    /// The `matter_legs` section, ragged by `leg_count`.
    legs: PairColumns,
}

/// Planets as a bit set in a byte: only the seven cross here, ids 0 to 6.
fn graha_bits(grahas: &[teistro::catalogue::Graha]) -> u8 {
    u8::try_from(graha_mask(grahas) & 0x7F).unwrap_or(0)
}

impl MatterColumns {
    fn push(&mut self, matter: &teistro::YearYogas) -> Result<(), Error> {
        self.house.push(matter.house.get());
        self.sign.push(matter.sign.id());
        self.lagnesha.push(matter.lagnesha.id());
        self.karyesha.push(matter.karyesha.id());
        self.same_lord.push(u8::from(matter.same_lord));
        self.pair.push(matter.between.as_ref());
        self.unanswered
            .push(matter.unanswered.iter().fold(0, |bits, yoga| {
                bits | 1_u16 << TsYearYoga::from(*yoga) as u8
            }));
        self.held_count
            .push(u8::try_from(matter.held.len()).unwrap_or(u8::MAX));
        for held in &matter.held {
            // A yoga's `between` is the matter's own pair or nothing, which
            // is why it crosses as a flag; were that ever not so, the flag
            // would lie, and the batch is refused instead.
            if held.between.is_some() && held.between != matter.between {
                return Err(Error::new(
                    Status::Internal,
                    format!(
                        "{:?} for house {} carries a pair other than the matter's own",
                        held.yoga,
                        matter.house.get()
                    ),
                ));
            }
            let held_rows = &mut self.held;
            held_rows.yoga.push(TsYearYoga::from(held.yoga) as u8);
            held_rows.by_pair.push(u8::from(held.between.is_some()));
            let (through, present) = graha_or_absent(held.through);
            held_rows.through.push(through);
            held_rows.through_present.push(present);
            let (entering, present) = graha_or_absent(held.entering);
            held_rows.entering.push(entering);
            held_rows.entering_present.push(present);
            let [lagnesha, karyesha] = held
                .afflictions
                .map_or([0, 0], |both| both.map(TsAffliction::bits));
            held_rows
                .afflictions_present
                .push(u8::from(held.afflictions.is_some()));
            held_rows.lagnesha_afflictions.push(lagnesha);
            held_rows.karyesha_afflictions.push(karyesha);
            let legs = held.legs.as_ref().map_or(&[][..], |legs| &legs[..]);
            held_rows
                .leg_count
                .push(u8::try_from(legs.len()).unwrap_or(u8::MAX));
            for leg in legs {
                held_rows.legs.push(Some(leg));
            }
        }
        Ok(())
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        let mut columns = vec![
            ColumnData::U8(&self.house),
            ColumnData::U16(&self.sign),
            ColumnData::U16(&self.lagnesha),
            ColumnData::U16(&self.karyesha),
            ColumnData::U8(&self.same_lord),
        ];
        columns.extend(self.pair.data());
        columns.extend([
            ColumnData::U16(&self.unanswered),
            ColumnData::U8(&self.held_count),
        ]);
        writer.columns("year_matters", self.house.len(), &columns)?;
        let held = &self.held;
        writer.columns(
            "matter_yogas",
            held.yoga.len(),
            &[
                ColumnData::U8(&held.yoga),
                ColumnData::U8(&held.by_pair),
                ColumnData::U16(&held.through),
                ColumnData::U8(&held.through_present),
                ColumnData::U16(&held.entering),
                ColumnData::U8(&held.entering_present),
                ColumnData::U8(&held.afflictions_present),
                ColumnData::U8(&held.lagnesha_afflictions),
                ColumnData::U8(&held.karyesha_afflictions),
                ColumnData::U8(&held.leg_count),
            ],
        )?;
        writer.columns("matter_legs", held.legs.faster.len(), &held.legs.data())
    }
}

/// Every year's yoga-making pairs, flat and ragged.
#[derive(Default)]
struct YogaColumns {
    faster: Vec<u16>,
    slower: Vec<u16>,
    drishti: Vec<u8>,
    yoga: Vec<u8>,
    orb_deg: Vec<f64>,
    apart_deg: Vec<f64>,
}

impl YogaColumns {
    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "year_yogas",
            self.faster.len(),
            &[
                ColumnData::U16(&self.faster),
                ColumnData::U16(&self.slower),
                ColumnData::U8(&self.drishti),
                ColumnData::U8(&self.yoga),
                ColumnData::F64(&self.orb_deg),
                ColumnData::F64(&self.apart_deg),
            ],
        )
    }
}

/// Every year's claimants on the lordship, flat and ragged.
#[derive(Default)]
struct ClaimColumns {
    graha: Vec<u16>,
    vishwa: Vec<i32>,
    portfolios: Vec<u8>,
    aspects_lagna: Vec<u8>,
}

impl ClaimColumns {
    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "year_claims",
            self.graha.len(),
            &[
                ColumnData::U16(&self.graha),
                ColumnData::I32(&self.vishwa),
                ColumnData::U8(&self.portfolios),
                ColumnData::U8(&self.aspects_lagna),
            ],
        )
    }
}

impl AnnualColumns {
    fn push(&mut self, year: &teistro::AnnualChart) -> Result<(), Error> {
        let bearers = &year.bearers;
        self.lagnas.push(year.lagna_deg);
        self.daylight.push(u8::from(bearers.by_day));
        self.janma_lagna.push(bearers.janma_lagna.id());
        self.varsha_lagna.push(bearers.varsha_lagna.id());
        self.tri_rashi.push(bearers.tri_rashi.id());
        self.dina_ratri.push(bearers.dina_ratri.id());
        let lord = &year.year_lord;
        self.year_lord.push(lord.graha.id());
        self.chosen.push(TsVarsheshaChosen::from(lord.chosen) as u8);
        self.year_lord_vishwa.push(sub_sub(lord.vishwa));
        self.moon_passed_over.push(u8::from(lord.moon_passed_over));
        self.claim_counts
            .push(u8::try_from(lord.claims.len()).unwrap_or(u8::MAX));
        self.yoga_counts
            .push(u8::try_from(year.yogas.len()).unwrap_or(u8::MAX));
        for pair in &year.yogas {
            self.yogas.faster.push(pair.faster.id());
            self.yogas.slower.push(pair.slower.id());
            self.yogas
                .drishti
                .push(TsTajikaDrishti::from(pair.drishti) as u8);
            // Only pairs that make one are here, so the fallback is dead.
            self.yogas.yoga.push(
                pair.yoga
                    .map_or(u8::MAX, |yoga| TsTajikaYoga::from(yoga) as u8),
            );
            self.yogas.orb_deg.push(pair.orb_deg);
            self.yogas.apart_deg.push(pair.apart_deg);
        }
        for claim in &lord.claims {
            self.claims.graha.push(claim.graha.id());
            self.claims.vishwa.push(sub_sub(claim.vishwa));
            self.claims.portfolios.push(claim.portfolios);
            self.claims
                .aspects_lagna
                .push(u8::from(claim.aspects_lagna));
        }
        self.retrograde.push(graha_bits(&year.states.retrograde));
        self.combust.push(graha_bits(&year.states.combust));
        self.matter_counts
            .push(u8::try_from(year.matters.len()).unwrap_or(u8::MAX));
        self.saham_counts
            .push(u8::try_from(year.sahams.len()).unwrap_or(u8::MAX));
        for one in &year.sahams {
            self.sahams.push(one);
        }
        self.harsha.push(&year.harsha);
        self.dasha_counts
            .push(u8::try_from(year.dashas.len()).unwrap_or(u8::MAX));
        for dasha in &year.dashas {
            self.dashas.push(dasha);
        }
        year.matters
            .iter()
            .try_for_each(|matter| self.matters.push(matter))
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "annual_charts",
            self.lagnas.len(),
            &[
                ColumnData::F64(&self.lagnas),
                ColumnData::U8(&self.daylight),
                ColumnData::U16(&self.janma_lagna),
                ColumnData::U16(&self.varsha_lagna),
                ColumnData::U16(&self.tri_rashi),
                ColumnData::U16(&self.dina_ratri),
                ColumnData::U16(&self.year_lord),
                ColumnData::U8(&self.chosen),
                ColumnData::I32(&self.year_lord_vishwa),
                ColumnData::U8(&self.moon_passed_over),
                ColumnData::U8(&self.claim_counts),
                ColumnData::U8(&self.yoga_counts),
                ColumnData::U8(&self.retrograde),
                ColumnData::U8(&self.combust),
                ColumnData::U8(&self.matter_counts),
                ColumnData::U8(&self.saham_counts),
                ColumnData::U8(&self.dasha_counts),
            ],
        )?;
        self.claims.write(writer)?;
        self.yogas.write(writer)?;
        self.matters.write(writer)?;
        self.sahams
            .write(writer, "year_sahams", "year_saham_seven")?;
        self.harsha.write(writer)
    }
}

/// A strength as the boundary carries it: exact, in sub-sub units, of
/// which a unit holds 3600.
///
/// An integer and not a float, because the source works to sub-sub units
/// and two office-bearers a sub-sub unit apart decide a year between them.
fn sub_sub(bala: teistro::Bala) -> i32 {
    i32::try_from(bala.as_sub_sub()).unwrap_or(i32::MAX)
}

impl PraveshaColumns {
    pub(crate) fn of(varsha: &[teistro::Varsha]) -> Result<PraveshaColumns, Error> {
        let mut counts = Vec::with_capacity(varsha.len());
        let mut years = Vec::new();
        let mut jds = Vec::new();
        let mut muntha_signs = Vec::new();
        let mut muntha_lords = Vec::new();
        let mut muntha_degs = Vec::new();
        let mut annual = AnnualColumns::default();
        let mut natal_counts = Vec::with_capacity(varsha.len());
        let mut natal = SahamColumns::default();
        for chart in varsha {
            natal_counts.push(u32::try_from(chart.natal_sahams.len()).unwrap_or(u32::MAX));
            for one in &chart.natal_sahams {
                natal.push(one);
            }
            let found = &chart.years;
            counts.push(u32::try_from(found.len()).unwrap_or(u32::MAX));
            for one in found {
                years.push(one.pravesha.year);
                jds.push(one.pravesha.at.get());
                muntha_signs.push(one.muntha.sign.id());
                muntha_lords.push(one.muntha.lord.id());
                muntha_degs.push(one.muntha.longitude_deg);
                if let Some(year) = &one.annual {
                    annual.push(year)?;
                }
            }
        }
        Ok(PraveshaColumns {
            counts,
            years,
            jds,
            muntha_signs,
            muntha_lords,
            muntha_degs,
            annual,
            natal_counts,
            natal,
        })
    }

    pub(crate) fn write(
        &self,
        writer: &mut Writer<'_>,
    ) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "praveshas",
            self.years.len(),
            &[
                ColumnData::U16(&self.years),
                ColumnData::F64(&self.jds),
                ColumnData::U16(&self.muntha_signs),
                ColumnData::U16(&self.muntha_lords),
                ColumnData::F64(&self.muntha_degs),
            ],
        )?;
        self.annual.write(writer)?;
        self.natal
            .write(writer, "natal_sahams", "natal_saham_seven")?;
        self.annual.dashas.write(writer)
    }
}

/// Every chart's annual charts, empty when none were asked for.
///
/// Composed by the façade ([`teistro::ChartArea::varsha`]), one birth at a
/// time; a refusal says which chart of the batch it was refused for.
pub(crate) fn praveshas_of(
    sdk: &teistro::Context,
    documents: &[Document],
    birth_clock: teistro::UtcOffset,
    asked: Option<&super::Request>,
) -> Result<Vec<teistro::Varsha>, Error> {
    let Some(asked) = asked else {
        return Ok(Vec::new());
    };
    documents
        .iter()
        .enumerate()
        .map(|(at, document)| {
            sdk.chart()
                .varsha(document, birth_clock, asked)
                .map_err(|error| error.with_hint(format!("chart {at}")))
        })
        .collect()
}

impl From<teistro::TajikaDrishti> for TsTajikaDrishti {
    fn from(drishti: teistro::TajikaDrishti) -> TsTajikaDrishti {
        match drishti {
            teistro::TajikaDrishti::Friendly => TsTajikaDrishti::Friendly,
            teistro::TajikaDrishti::SecretlyFriendly => TsTajikaDrishti::SecretlyFriendly,
            teistro::TajikaDrishti::Inimical => TsTajikaDrishti::Inimical,
            teistro::TajikaDrishti::SecretlyInimical => TsTajikaDrishti::SecretlyInimical,
            teistro::TajikaDrishti::None => TsTajikaDrishti::None,
        }
    }
}

impl From<teistro::TajikaYoga> for TsTajikaYoga {
    fn from(yoga: teistro::TajikaYoga) -> TsTajikaYoga {
        match yoga {
            teistro::TajikaYoga::IthasalaVartamana => TsTajikaYoga::IthasalaVartamana,
            teistro::TajikaYoga::IthasalaPoorna => TsTajikaYoga::IthasalaPoorna,
            teistro::TajikaYoga::IthasalaBhavishyat => TsTajikaYoga::IthasalaBhavishyat,
            teistro::TajikaYoga::Ishrafa => TsTajikaYoga::Ishrafa,
        }
    }
}

impl From<teistro::YearYoga> for TsYearYoga {
    fn from(yoga: teistro::YearYoga) -> TsYearYoga {
        match yoga {
            teistro::YearYoga::Ikabala => TsYearYoga::Ikabala,
            teistro::YearYoga::Induvara => TsYearYoga::Induvara,
            teistro::YearYoga::Ithasala => TsYearYoga::Ithasala,
            teistro::YearYoga::Ishrafa => TsYearYoga::Ishrafa,
            teistro::YearYoga::Nakta => TsYearYoga::Nakta,
            teistro::YearYoga::Yamaya => TsYearYoga::Yamaya,
            teistro::YearYoga::Manau => TsYearYoga::Manau,
            teistro::YearYoga::Kamboola => TsYearYoga::Kamboola,
            teistro::YearYoga::GairiKamboola => TsYearYoga::GairiKamboola,
            teistro::YearYoga::Khallasara => TsYearYoga::Khallasara,
            teistro::YearYoga::Rudda => TsYearYoga::Rudda,
            teistro::YearYoga::DuhphaliKuttha => TsYearYoga::DuhphaliKuttha,
            teistro::YearYoga::DutthotthaDavira => TsYearYoga::DutthotthaDavira,
            teistro::YearYoga::Tambira => TsYearYoga::Tambira,
            teistro::YearYoga::Kuttha => TsYearYoga::Kuttha,
            teistro::YearYoga::Durapha => TsYearYoga::Durapha,
        }
    }
}

impl From<teistro::Saham> for TsSaham {
    fn from(saham: teistro::Saham) -> TsSaham {
        match saham {
            teistro::Saham::Punya => TsSaham::Punya,
            teistro::Saham::Guru => TsSaham::Guru,
            teistro::Saham::Vidya => TsSaham::Vidya,
            teistro::Saham::Yasha => TsSaham::Yasha,
            teistro::Saham::Mitra => TsSaham::Mitra,
            teistro::Saham::Mahatmya => TsSaham::Mahatmya,
            teistro::Saham::Asha => TsSaham::Asha,
            teistro::Saham::Samarthya => TsSaham::Samarthya,
            teistro::Saham::Bhratri => TsSaham::Bhratri,
            teistro::Saham::Gaurava => TsSaham::Gaurava,
            teistro::Saham::Pitri => TsSaham::Pitri,
            teistro::Saham::Raja => TsSaham::Raja,
            teistro::Saham::Matri => TsSaham::Matri,
            teistro::Saham::Putra => TsSaham::Putra,
            teistro::Saham::Jeeva => TsSaham::Jeeva,
            teistro::Saham::Roga => TsSaham::Roga,
            teistro::Saham::Karma => TsSaham::Karma,
            teistro::Saham::Manmatha => TsSaham::Manmatha,
            teistro::Saham::Kali => TsSaham::Kali,
            teistro::Saham::Kshama => TsSaham::Kshama,
            teistro::Saham::Shastra => TsSaham::Shastra,
            teistro::Saham::Bandhu => TsSaham::Bandhu,
            teistro::Saham::Mrityu => TsSaham::Mrityu,
            teistro::Saham::Deshantara => TsSaham::Deshantara,
            teistro::Saham::Artha => TsSaham::Artha,
            teistro::Saham::Paradara => TsSaham::Paradara,
            teistro::Saham::AnyaKarma => TsSaham::AnyaKarma,
            teistro::Saham::Vanika => TsSaham::Vanika,
            teistro::Saham::KaryaSiddhi => TsSaham::KaryaSiddhi,
            teistro::Saham::Vivaha => TsSaham::Vivaha,
            teistro::Saham::Prasava => TsSaham::Prasava,
            teistro::Saham::Santaapa => TsSaham::Santaapa,
            teistro::Saham::Shraddha => TsSaham::Shraddha,
            teistro::Saham::Preeti => TsSaham::Preeti,
            teistro::Saham::Jadya => TsSaham::Jadya,
            teistro::Saham::Vyapara => TsSaham::Vyapara,
            teistro::Saham::PaneeyaPaata => TsSaham::PaneeyaPaata,
            teistro::Saham::Shatru => TsSaham::Shatru,
            teistro::Saham::Jalapatha => TsSaham::Jalapatha,
            teistro::Saham::Bandhana => TsSaham::Bandhana,
            teistro::Saham::Labha => TsSaham::Labha,
        }
    }
}

impl From<teistro::TajikaRelation> for TsTajikaRelation {
    fn from(relation: teistro::TajikaRelation) -> TsTajikaRelation {
        match relation {
            teistro::TajikaRelation::Own => TsTajikaRelation::Own,
            teistro::TajikaRelation::Friend => TsTajikaRelation::Friend,
            teistro::TajikaRelation::Neutral => TsTajikaRelation::Neutral,
            teistro::TajikaRelation::Enemy => TsTajikaRelation::Enemy,
        }
    }
}

impl From<teistro::HarshaGrade> for TsHarshaGrade {
    fn from(grade: teistro::HarshaGrade) -> TsHarshaGrade {
        match grade {
            teistro::HarshaGrade::Nirbala => TsHarshaGrade::Nirbala,
            teistro::HarshaGrade::Alpabali => TsHarshaGrade::Alpabali,
            teistro::HarshaGrade::MadhyaBali => TsHarshaGrade::MadhyaBali,
            teistro::HarshaGrade::PoornaBali => TsHarshaGrade::PoornaBali,
            teistro::HarshaGrade::Extraordinary => TsHarshaGrade::Extraordinary,
        }
    }
}

impl From<teistro::StrongClause> for TsSahamStrong {
    fn from(clause: teistro::StrongClause) -> TsSahamStrong {
        match clause {
            teistro::StrongClause::LordExalted => TsSahamStrong::LordExalted,
            teistro::StrongClause::LordOwnSign => TsSahamStrong::LordOwnSign,
            teistro::StrongClause::LordOwnHudda => TsSahamStrong::LordOwnHudda,
            teistro::StrongClause::LordOwnDrekkana => TsSahamStrong::LordOwnDrekkana,
            teistro::StrongClause::LordOwnNavamsha => TsSahamStrong::LordOwnNavamsha,
            teistro::StrongClause::LordInFriendsSign => TsSahamStrong::LordInFriendsSign,
            teistro::StrongClause::WithFriend => TsSahamStrong::WithFriend,
            teistro::StrongClause::WithBenefic => TsSahamStrong::WithBenefic,
            teistro::StrongClause::WithYearLord => TsSahamStrong::WithYearLord,
            teistro::StrongClause::LordConjoins => TsSahamStrong::LordConjoins,
            teistro::StrongClause::LordAspectsSaham => TsSahamStrong::LordAspectsSaham,
            teistro::StrongClause::LordAspectsLagna => TsSahamStrong::LordAspectsLagna,
        }
    }
}

impl From<teistro::WeakClause> for TsSahamWeak {
    fn from(clause: teistro::WeakClause) -> TsSahamWeak {
        match clause {
            teistro::WeakClause::LordWeakVishwa => TsSahamWeak::LordWeakVishwa,
            teistro::WeakClause::LordLacksHarsha => TsSahamWeak::LordLacksHarsha,
            teistro::WeakClause::LordApart => TsSahamWeak::LordApart,
            teistro::WeakClause::WithEnemy => TsSahamWeak::WithEnemy,
            teistro::WeakClause::WithMalefic => TsSahamWeak::WithMalefic,
        }
    }
}

impl TsAffliction {
    /// An affliction's clauses as a bit set: bit `n` is the clause with
    /// id `n`. The struct is destructured whole, so a clause added to
    /// `teistro::Affliction` stops this compiling rather than crossing
    /// unset.
    pub(crate) fn bits(affliction: teistro::Affliction) -> u8 {
        let teistro::Affliction {
            graha: _,
            retrograde,
            combust,
            debilitated,
            trika,
            under_malefic,
        } = affliction;
        [
            (TsAffliction::Retrograde, retrograde),
            (TsAffliction::Combust, combust),
            (TsAffliction::Debilitated, debilitated),
            (TsAffliction::Trika, trika),
            (TsAffliction::UnderMalefic, under_malefic),
        ]
        .into_iter()
        .filter(|(_, holds)| *holds)
        .fold(0, |bits, (clause, _)| bits | 1 << clause as u8)
    }
}

impl From<teistro::Chosen> for TsVarsheshaChosen {
    fn from(chosen: teistro::Chosen) -> TsVarsheshaChosen {
        match chosen {
            teistro::Chosen::Strongest => TsVarsheshaChosen::Strongest,
            teistro::Chosen::MostPortfolios => TsVarsheshaChosen::MostPortfolios,
            teistro::Chosen::MunthaLordUnaspected => TsVarsheshaChosen::MunthaLordUnaspected,
            teistro::Chosen::MunthaLordAllWeak => TsVarsheshaChosen::MunthaLordAllWeak,
            teistro::Chosen::MunthaLordTied => TsVarsheshaChosen::MunthaLordTied,
            teistro::Chosen::DinaRatriTied => TsVarsheshaChosen::DinaRatriTied,
            teistro::Chosen::AnnualLagnaLordUnaspected => {
                TsVarsheshaChosen::AnnualLagnaLordUnaspected
            }
            teistro::Chosen::StrongestUnaspected => TsVarsheshaChosen::StrongestUnaspected,
            teistro::Chosen::MoonsIthasala => TsVarsheshaChosen::MoonsIthasala,
            teistro::Chosen::MoonsSignLord => TsVarsheshaChosen::MoonsSignLord,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{graha_bits, graha_mask};
    use teistro::catalogue::Graha;

    /// The Brahma graha's two masks read back as the grahas they hold, the
    /// nodes too, where the planets' byte holds only the seven.
    #[test]
    fn the_nine_grahas_mask_into_sixteen_bits() {
        let nine = &Graha::ALL[..9];
        assert_eq!(graha_mask(nine), 0x01FF);
        assert_eq!(graha_mask(&[Graha::Ketu, Graha::Sun]), 0x0101);
        assert_eq!(graha_bits(nine), 0x7F);
    }
}
