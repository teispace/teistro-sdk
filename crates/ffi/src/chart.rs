//! The chart foundation at the C boundary: the enums a founded chart's
//! blob carries that no other entry point needed, and the conversions
//! from the Rust values they mirror.
//!
//! Designed in `03-design/chart-at-the-boundary.md`. Five enums are
//! declared here rather than taken from the catalogue, because they are
//! not catalogue kinds: two belong to the chart layer (`Reading`,
//! `DayPart`) and three are settings knobs a result has to carry, since
//! a ghati count means nothing without the reckoning that produced it
//! and a chalit means nothing without the reading it was taken under.
//!
#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]
//!
//! Two of them mirror a Rust enum through an **exhaustive** match, which
//! is what stops the two drifting: a variant added stops this crate
//! compiling rather than silently mapping to whatever was first.
//! `TsResolution` in `calendar.rs` is the pattern.
//!
//! The three knobs cannot do that. A settings knob is `#[non_exhaustive]`
//! on purpose, so a match on one needs a `_` arm and a member added
//! later would fall into it silently. They convert **fallibly** instead
//! and refuse a member this build does not know, and the guard is a test
//! over the knob's own `ALL`: adding a member fails it by name rather
//! than shipping a wrong id.

use teistro_astro::precession::PrecessionModel;
use teistro_calendar::shipped;
use teistro_calendar::solar::drik::DrikSun;
use teistro_chart::bhava::Reading;
use teistro_chart::day::DayPart;
use teistro_chart::foundation::{ChartFoundation, Founder};
use teistro_core::catalogue::{Ayanamsha, ChartKind};
use teistro_core::envelope::{Provenance, content_hash};
use teistro_core::error::{Error, Status};
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro_core::settings::AyanamshaChoice;
use teistro_core::time::UtcOffset;
use teistro_idl::blob::{ColumnData, FixedValue, Writer};

use crate::blob::TsBlob;
use crate::context::TsContext;
use crate::support::{with_context, write_plain};
use teistro_core::settings::{GhatiReckoning, HoraReckoning, PolarDayPolicy, Sunrise};
use teistro_time::local_day::{DayState, PolarKind};

/// Which bound of a bhava a placement was read against.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsReading {
    /// From one sandhi to the next: the bhava as a span between cusps.
    Sandhi = 0,
    /// From one madhya to the next: the bhava as a span between centres.
    Madhya = 1,
}

impl From<Reading> for TsReading {
    fn from(reading: Reading) -> TsReading {
        match reading {
            Reading::Sandhi => TsReading::Sandhi,
            Reading::Madhya => TsReading::Madhya,
        }
    }
}

/// Which arc of its day an instant falls in.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsDayPart {
    /// Between sunrise and sunset.
    Daylight = 0,
    /// Between sunset and the next sunrise.
    Night = 1,
}

impl From<DayPart> for TsDayPart {
    fn from(part: DayPart) -> TsDayPart {
        match part {
            DayPart::Daylight => TsDayPart::Daylight,
            DayPart::Night => TsDayPart::Night,
        }
    }
}

/// Which sunrise a day was reckoned from.
///
/// The named conventions only. A profile may ask for the centre of the
/// disc at a chosen altitude instead, which is a `Custom` convention;
/// a blob carries that as its altitude beside this, because a variant
/// with a payload cannot be an id (`03-design/chart-at-the-boundary.md`
/// §8).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsSunrise {
    /// The centre of the disc on the geometric horizon.
    CentreNoRefraction = 0,
    /// The upper limb with refraction.
    UpperLimbRefraction = 1,
    /// The lower limb with refraction.
    LowerLimbRefraction = 2,
}

impl TsSunrise {
    /// The id this build gives a member, or `None` for one it does
    /// not know — a member added to the knob since this was written,
    /// which is refused by name rather than defaulted.
    #[must_use]
    pub fn of(sunrise: Sunrise) -> Option<TsSunrise> {
        match sunrise {
            Sunrise::CentreNoRefraction => Some(TsSunrise::CentreNoRefraction),
            Sunrise::UpperLimbRefraction => Some(TsSunrise::UpperLimbRefraction),
            Sunrise::LowerLimbRefraction => Some(TsSunrise::LowerLimbRefraction),
            _ => None,
        }
    }
}

/// How the sixty ghatis of a day are measured.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsGhatiReckoning {
    /// Twenty-four minutes each, from sunrise.
    Civil = 0,
    /// Thirty over the actual daylight and thirty over the actual night.
    Proportional = 1,
}

impl TsGhatiReckoning {
    /// The id this build gives a member, or `None` for one it does
    /// not know — a member added to the knob since this was written,
    /// which is refused by name rather than defaulted.
    #[must_use]
    pub fn of(reckoning: GhatiReckoning) -> Option<TsGhatiReckoning> {
        match reckoning {
            GhatiReckoning::Civil => Some(TsGhatiReckoning::Civil),
            GhatiReckoning::Proportional => Some(TsGhatiReckoning::Proportional),
            _ => None,
        }
    }
}

/// How the twenty-four horas of a day are measured.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsHoraReckoning {
    /// Twelve over the daylight and twelve over the night.
    Proportional = 0,
    /// Twenty-four of sixty minutes, from sunrise.
    Equal = 1,
}

impl TsHoraReckoning {
    /// The id this build gives a member, or `None` for one it does
    /// not know — a member added to the knob since this was written,
    /// which is refused by name rather than defaulted.
    #[must_use]
    pub fn of(reckoning: HoraReckoning) -> Option<TsHoraReckoning> {
        match reckoning {
            HoraReckoning::Proportional => Some(TsHoraReckoning::Proportional),
            HoraReckoning::Equal => Some(TsHoraReckoning::Equal),
            _ => None,
        }
    }
}

/// Whether a day had a sunrise, and what was done when it had not.
///
/// The kind half of a tagged enum: a polar day carries which polar
/// state it was and which policy was applied, in `state_polar_kind` and
/// `state_polar_policy` beside it, because a variant with a payload
/// cannot be an id (`03-design/chart-at-the-boundary.md` §8).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsDayState {
    /// Sunrise and sunset occurred; the two fields beside this are zero.
    Normal = 0,
    /// No horizon crossing, and the policy synthesised the bounds.
    Polar = 1,
}

/// Which polar state a day without a sunrise was in.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsPolarKind {
    /// The Sun stayed up.
    Day = 0,
    /// The Sun stayed down.
    Night = 1,
}

impl From<PolarKind> for TsPolarKind {
    fn from(kind: PolarKind) -> TsPolarKind {
        match kind {
            PolarKind::Day => TsPolarKind::Day,
            PolarKind::Night => TsPolarKind::Night,
        }
    }
}

/// What the settings say a day without a sunrise is.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsPolarDayPolicy {
    /// An undefined state: the day has no bounds.
    Undefined = 0,
    /// The nearest rise or set stands in for the missing one.
    NearestEvent = 1,
    /// Civil midnight stands in for it.
    CivilMidnight = 2,
}

impl TsPolarDayPolicy {
    /// The id this build gives a member, or `None` for one it does not
    /// know — a member added to the knob since this was written, which
    /// is refused by name rather than defaulted.
    #[must_use]
    pub fn of(policy: PolarDayPolicy) -> Option<TsPolarDayPolicy> {
        match policy {
            PolarDayPolicy::Undefined => Some(TsPolarDayPolicy::Undefined),
            PolarDayPolicy::NearestEvent => Some(TsPolarDayPolicy::NearestEvent),
            PolarDayPolicy::CivilMidnight => Some(TsPolarDayPolicy::CivilMidnight),
            _ => None,
        }
    }
}

impl TsDayState {
    /// A day's state split into the three scalars a blob carries: the
    /// kind, and the polar kind and policy that only a polar day has.
    #[must_use]
    pub fn split(state: DayState) -> (TsDayState, u8, u8) {
        match state {
            DayState::Normal => (TsDayState::Normal, 0, 0),
            DayState::Polar { kind, policy } => (
                TsDayState::Polar,
                TsPolarKind::from(kind) as u8,
                TsPolarDayPolicy::of(policy).map_or(0, |p| p as u8),
            ),
        }
    }
}

/// What a chart is founded on: when, where, what kind, and the clock its
/// day is reckoned in.
///
/// Everything else is the context's settings, which is what makes two
/// calls under one context comparable and what the settings hash is for.
/// The clock is here because nothing else knows it: a chart's day runs
/// from a local sunrise and its date is a civil date, and a longitude
/// gives local *mean* time rather than a civil offset
/// (`03-design/chart-at-the-boundary.md` §5).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TsChartRequest {
    /// `sizeof(ts_chart_request)` as the caller compiled it.
    pub struct_size: u32,
    /// What kind of chart to found.
    /// `api: enum=ChartKind example=0`
    pub kind: u16,
    /// Reserved; write zero.
    pub reserved: u16,
    /// The instants, as Julian days on the UTC scale: one chart each.
    ///
    /// A grid, not a scalar, because the founder shares the settings and
    /// the solar model across a batch and a rectification pass wants a
    /// hundred charts (`03-design/chart-at-the-boundary.md` §3a). A
    /// caller wanting one passes a grid of one, as `ts_positions` takes
    /// a grid of one instant.
    /// `api: len=instant_count unit=jd`
    pub instants: *const f64,
    /// How many instants `instants` points at.
    pub instant_count: usize,
    /// The place's latitude, degrees north.
    /// `api: unit=deg range=[-90,90] example=27.7172`
    pub latitude_deg: f64,
    /// The place's longitude, degrees east.
    /// `api: unit=deg range=[-180,180] example=85.324`
    pub longitude_deg: f64,
    /// The place's altitude, metres above the ellipsoid.
    /// `api: unit=m range=[-500,9000] example=1400`
    pub altitude_m: f64,
    /// The local clock's offset from UTC in seconds, east positive: the
    /// clock the day's date is read in.
    /// `api: unit=s range=[-64800,64800] example=20700`
    pub utc_offset_seconds: i32,
    /// Reserved; write zero.
    pub reserved_tail: i32,
}

/// The day's seventeen-and-three values, in the order `day_section`
/// declares them.
///
/// Declared once in `schemas::day_section` and filled once here, so the
/// panchanga blob writes the same day the same way rather than a second
/// copy of the same arithmetic
/// (`03-design/chart-at-the-boundary.md` §8).
#[must_use]
pub fn day_values(local: &teistro_time::local_day::LocalDay) -> Vec<FixedValue> {
    let (state, polar_kind, polar_policy) = TsDayState::split(local.state);
    let (convention, convention_value) = match local.convention {
        teistro_core::settings::SunriseConvention::Named { which } => (
            TsSunrise::of(which).map_or(u64::from(u8::MAX), |s| s as u64),
            0.0,
        ),
        // No id names a custom convention, so the sentinel says "read the
        // altitude beside this" rather than naming a convention it is not.
        teistro_core::settings::SunriseConvention::Custom { altitude_deg } => {
            (u64::from(u8::MAX), altitude_deg)
        }
    };
    let era = local.date.era;
    vec![
        local.sunrise.get().into(),
        local.sunset.get().into(),
        local.next_sunrise.get().into(),
        u64::from(local.vara.id()).into(),
        u64::from(local.date.calendar.id()).into(),
        era.map_or(u64::from(u16::MAX), |e| u64::from(e.era.id()))
            .into(),
        i64::from(local.date.year).into(),
        i64::from(era.map_or(0, |e| e.year)).into(),
        u64::from(local.date.month).into(),
        u64::from(local.date.day).into(),
        (resolution_id(&local.date.resolution)).into(),
        u64::from(computed(&local.date.resolution).0).into(),
        u64::from(computed(&local.date.resolution).1).into(),
        (state as u64).into(),
        u64::from(polar_kind).into(),
        u64::from(polar_policy).into(),
        convention.into(),
        convention_value.into(),
    ]
}

/// A resolution's id, as `TsResolution` numbers them.
fn resolution_id(resolution: &teistro_core::envelope::CalendarResolution) -> u64 {
    use teistro_core::envelope::CalendarResolution as R;
    match resolution {
        R::Defined => 0,
        R::Tabular { .. } => 1,
        R::Computed { .. } => 2,
        R::Divergent { .. } => 3,
    }
}

/// The engine's month and day where a divergent resolution reports them,
/// and nought otherwise.
fn computed(resolution: &teistro_core::envelope::CalendarResolution) -> (u8, u8) {
    match resolution {
        teistro_core::envelope::CalendarResolution::Divergent { computed, .. } => {
            (computed.month, computed.day)
        }
        _ => (0, 0),
    }
}

/// One vector per graha column: the writer takes slices, and a column
/// is the unit the format stores.
struct GrahaColumns {
    ids: Vec<u16>,
    longitudes: Vec<f64>,
    tropicals: Vec<f64>,
    latitudes: Vec<f64>,
    distances: Vec<f64>,
    speeds: Vec<f64>,
    house_bhava: Vec<u8>,
    house_method: Vec<u16>,
    house_through: Vec<f64>,
    house_from: Vec<f64>,
    placed_bhava: Vec<u8>,
    placed_method: Vec<u16>,
    placed_through: Vec<f64>,
    placed_from: Vec<f64>,
}

impl GrahaColumns {
    /// The columns of every chart's grahas, charts outermost: row
    /// `chart * graha_count + g` is graha `g` of chart `chart`, the same
    /// order the positions blob puts its cells in.
    fn of(charts: &[ChartFoundation]) -> GrahaColumns {
        let rows: Vec<&teistro_chart::foundation::GrahaPosition> =
            charts.iter().flat_map(|c| c.grahas.iter()).collect();
        GrahaColumns {
            ids: rows.iter().map(|g| g.graha.id()).collect(),
            longitudes: rows.iter().map(|g| g.longitude_deg).collect(),
            tropicals: rows.iter().map(|g| g.tropical_deg).collect(),
            latitudes: rows.iter().map(|g| g.latitude_deg).collect(),
            distances: rows.iter().map(|g| g.distance_au).collect(),
            speeds: rows.iter().map(|g| g.speed_deg_per_day).collect(),
            house_bhava: rows.iter().map(|g| g.house.bhava).collect(),
            house_method: rows.iter().map(|g| g.house.method.id()).collect(),
            house_through: rows.iter().map(|g| g.house.through).collect(),
            house_from: rows.iter().map(|g| g.house.from_madhya_deg).collect(),
            placed_bhava: rows.iter().map(|g| g.placement.bhava).collect(),
            placed_method: rows.iter().map(|g| g.placement.method.id()).collect(),
            placed_through: rows.iter().map(|g| g.placement.through).collect(),
            placed_from: rows.iter().map(|g| g.placement.from_madhya_deg).collect(),
        }
    }
}

/// What the batch decided once, in the order `summary` declares it.
///
/// Every value here comes from the request rather than from a chart, so
/// a batch that founded nothing still says where and what it founded
/// nothing of — as the positions blob writes its grid's frame whether or
/// not the grid has a cell.
#[must_use]
fn summary_values(
    place: &Place,
    kind: ChartKind,
    chart_count: u32,
    graha_count: u32,
) -> Vec<FixedValue> {
    vec![
        u64::from(kind.id()).into(),
        u64::from(chart_count).into(),
        u64::from(graha_count).into(),
        place.latitude.get().into(),
        place.longitude.get().into(),
        place.altitude.get().into(),
    ]
}

/// One row per chart, in the order `cast` declares its columns.
#[must_use]
fn chart_rows(charts: &[ChartFoundation]) -> Vec<Vec<FixedValue>> {
    charts
        .iter()
        .map(|chart| {
            vec![
                chart.instant.get().into(),
                chart.lagna_deg.into(),
                chart.day_lagna_deg.into(),
                chart.zodiac.offset_deg.into(),
                (TsDayPart::from(chart.day.part) as u64).into(),
                chart.day.elapsed.into(),
            ]
        })
        .collect()
}

/// Twelve bhavas per chart, charts outermost, as `houses` and `chalit`
/// both want them.
#[must_use]
fn bhava_columns(
    charts: &[ChartFoundation],
    of: fn(&ChartFoundation) -> &teistro_chart::bhava::Bhavas,
) -> (Vec<f64>, Vec<f64>) {
    let madhya = charts.iter().flat_map(|c| of(c).madhya).collect();
    let sandhi = charts.iter().flat_map(|c| of(c).sandhi).collect();
    (madhya, sandhi)
}

/// The birth timing's values, in the order `timing` declares them.
#[must_use]
fn timing_values(timing: &teistro_chart::foundation::BirthTiming) -> Vec<FixedValue> {
    vec![
        u64::from(timing.ishtakaal.ghati).into(),
        u64::from(timing.ishtakaal.pala).into(),
        u64::from(timing.ishtakaal.vipala).into(),
        TsGhatiReckoning::of(timing.ghati_reckoning)
            .map_or(u64::from(u8::MAX), |g| g as u64)
            .into(),
        u64::from(timing.hora.number).into(),
        u64::from(timing.hora.lord.id()).into(),
        timing.hora.start.get().into(),
        timing.hora.end.get().into(),
        TsHoraReckoning::of(timing.hora_reckoning)
            .map_or(u64::from(u8::MAX), |h| h as u64)
            .into(),
    ]
}

/// What only founding can tell about a batch, gathered from its first
/// chart.
///
/// The place, the kind and the counts come from the request, so they are
/// not here; these four are decided while a chart is founded — the house
/// systems that actually built the bhavas, the frame the positions were
/// asked for, the solar model's own description, the completion steps —
/// and are the same for every chart of a batch, which is why the first
/// answers for all of them.
///
/// A batch of none has no first chart. The sections are still declared,
/// so they are written as zeroes and empty text rather than left out,
/// and `summary.chart_count` is what says they mean nothing; the
/// provenance envelope still carries the settings hash that would have
/// produced them.
struct BatchOnce {
    readings: Vec<FixedValue>,
    frame_bits: u32,
    ayanamsha_kind: u64,
    ayanamsha: u64,
    model: String,
    steps: String,
}

impl BatchOnce {
    fn of(first: Option<&ChartFoundation>) -> BatchOnce {
        let Some(chart) = first else {
            return BatchOnce {
                readings: vec![FixedValue::Uint(0); 6],
                frame_bits: 0,
                ayanamsha_kind: 0,
                ayanamsha: 0,
                model: String::new(),
                steps: String::from("[]"),
            };
        };
        let (ayanamsha_kind, ayanamsha) = match chart.zodiac.ayanamsha {
            None => (0_u64, 0_u64),
            Some(teistro_core::settings::AyanamshaChoice::Catalogued { id }) => {
                (1, u64::from(id.id()))
            }
            // A custom ayanamsha's coefficients are settings, and the
            // settings hash pins them: a result carries what it applied.
            Some(teistro_core::settings::AyanamshaChoice::Custom { .. }) => (2, 0),
        };
        BatchOnce {
            readings: vec![
                u64::from(chart.houses.chalit.method.id()).into(),
                u64::from(chart.houses.chalit.source.id()).into(),
                (TsReading::from(chart.houses.chalit.reading) as u64).into(),
                u64::from(chart.chalit.chalit.method.id()).into(),
                u64::from(chart.chalit.chalit.source.id()).into(),
                (TsReading::from(chart.chalit.chalit.reading) as u64).into(),
            ],
            frame_bits: chart.zodiac.request.to_bits(),
            ayanamsha_kind,
            ayanamsha,
            model: chart.day.day.model.clone(),
            steps: serde_json::to_string(&chart.steps).unwrap_or_else(|_| String::from("[]")),
        }
    }
}

/// A batch of founded charts as the blob its schema describes.
///
/// Every per-chart section runs charts outermost, so a batch of one is
/// the same blob a one-chart entry point would have written, with the
/// counts saying so.
///
/// The sections that describe the batch rather than a chart — the place,
/// the kind, the frame, the house systems, the solar model, the
/// completion steps — are written from the request where the request
/// knows them and from the first chart where only founding can tell.
/// A batch of none therefore carries zeroes in the latter, and its
/// provenance envelope still carries the settings hash that would have
/// produced them.
///
/// # Errors
///
/// Whatever the writer refuses: a section the schema does not have, or a
/// column of the wrong length. Neither can happen for a value this crate
/// built, so a failure here is a schema that has drifted from this
/// function rather than a caller's mistake. Charts of differing graha
/// counts are `INTERNAL`, since the blob's layout is one count for the
/// batch.
pub fn encode(
    charts: &[ChartFoundation],
    place: &Place,
    kind: ChartKind,
    provenance: &Provenance,
) -> Result<Vec<u8>, Error> {
    let schema = crate::schemas::charts();
    let mut writer = Writer::new(&schema);
    let chart_count = u32::try_from(charts.len()).unwrap_or(u32::MAX);
    let graha_count = charts.first().map_or(0, |c| c.grahas.len());
    if let Some(odd) = charts.iter().find(|c| c.grahas.len() != graha_count) {
        return Err(Error::new(
            Status::Internal,
            format!(
                "the batch mixes chart sizes: {graha_count} grahas and {}, though every chart is the same kind",
                odd.grahas.len()
            ),
        ));
    }
    let columns = GrahaColumns::of(charts);
    let (house_madhya, house_sandhi) = bhava_columns(charts, |c| &c.houses);
    let (chalit_madhya, chalit_sandhi) = bhava_columns(charts, |c| &c.chalit);
    let day_rows: Vec<Vec<FixedValue>> = charts.iter().map(|c| day_values(&c.day.day)).collect();
    let timing_rows: Vec<Vec<FixedValue>> =
        charts.iter().map(|c| timing_values(&c.timing)).collect();
    let once = BatchOnce::of(charts.first());

    let write = || -> Result<Vec<u8>, teistro_idl::blob::BlobError> {
        writer.fixed(
            "summary",
            &summary_values(
                place,
                kind,
                chart_count,
                u32::try_from(graha_count).unwrap_or(u32::MAX),
            ),
        )?;
        writer.rows("cast", &chart_rows(charts))?;
        writer.columns(
            "grahas",
            charts.len() * graha_count,
            &[
                ColumnData::U16(&columns.ids),
                ColumnData::F64(&columns.longitudes),
                ColumnData::F64(&columns.tropicals),
                ColumnData::F64(&columns.latitudes),
                ColumnData::F64(&columns.distances),
                ColumnData::F64(&columns.speeds),
                ColumnData::U8(&columns.house_bhava),
                ColumnData::U16(&columns.house_method),
                ColumnData::F64(&columns.house_through),
                ColumnData::F64(&columns.house_from),
                ColumnData::U8(&columns.placed_bhava),
                ColumnData::U16(&columns.placed_method),
                ColumnData::F64(&columns.placed_through),
                ColumnData::F64(&columns.placed_from),
            ],
        )?;
        writer.fixed("readings", &once.readings)?;
        writer.columns(
            "houses",
            charts.len() * 12,
            &[
                ColumnData::F64(&house_madhya),
                ColumnData::F64(&house_sandhi),
            ],
        )?;
        writer.columns(
            "chalit",
            charts.len() * 12,
            &[
                ColumnData::F64(&chalit_madhya),
                ColumnData::F64(&chalit_sandhi),
            ],
        )?;
        writer.fixed(
            "zodiac",
            &[
                u64::from(once.frame_bits).into(),
                once.ayanamsha_kind.into(),
                once.ayanamsha.into(),
            ],
        )?;
        writer.rows("day", &day_rows)?;
        writer.rows("timing", &timing_rows)?;
        writer.bytes("model", once.model.as_bytes())?;
        writer.bytes("steps", once.steps.as_bytes())?;
        writer.bytes(
            "provenance",
            teistro_core::envelope::canonical_json(provenance).as_bytes(),
        )?;
        writer.finish()
    };
    write().map_err(|error| {
        Error::new(
            Status::Internal,
            format!("the chart blob could not be written: {error}"),
        )
    })
}

/// Founds a chart at an instant and a place and answers with its blob:
/// where every graha stands, in which bhava under both readings, in
/// which zodiac, on which day, at what time of that day.
///
/// Everything but the request is the context's settings, so two calls
/// under one context are comparable and the settings hash says why. The
/// civil calendar the day's date is read in comes from
/// `calendars.civil_calendar`, which is what that knob was waiting for.
///
/// A context without an ephemeris is `CAPABILITY`; a provider failure is
/// `PROVIDER` with the provider's own code in the last error.
///
/// `api: blob=charts`
///
/// # Safety
///
/// `context` must be a live handle; `request` valid for a read; `out_blob`
/// valid for a write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_chart_found(
    context: *const TsContext,
    request: *const TsChartRequest,
    out_blob: *mut TsBlob,
) -> Status {
    with_context(context, |ctx| {
        if request.is_null() {
            return Err(crate::support::null("request"));
        }
        // SAFETY: non-null; the caller promises a readable request.
        let asked = unsafe { *request };
        let provider = ctx.provider().ok_or_else(crate::support::no_ephemeris)?;
        let resolved = ctx.resolved();
        let settings = &resolved.settings;
        let place = Place::new(
            Latitude::try_new(asked.latitude_deg)
                .map_err(|e| Error::from(e).with_field("latitude_deg"))?,
            Longitude::try_new(asked.longitude_deg)
                .map_err(|e| Error::from(e).with_field("longitude_deg"))?,
            Altitude::try_new(asked.altitude_m)
                .map_err(|e| Error::from(e).with_field("altitude_m"))?,
        );
        let kind = ChartKind::from_id(asked.kind).ok_or_else(|| {
            Error::new(
                Status::InvalidArg,
                format!("no chart kind with id {}", asked.kind),
            )
            .with_field("kind")
        })?;
        let clock = UtcOffset::try_from_seconds(asked.utc_offset_seconds)
            .map_err(|e| Error::from(e).with_field("utc_offset_seconds"))?;
        // The knob's own deferral said it "gains a reader when `serial`
        // or a binding builds a chart from a settings document alone".
        // This is that reader.
        let calendar = shipped(settings.calendars.civil_calendar).ok_or_else(|| {
            Error::new(
                Status::Unsupported,
                format!(
                    "the SDK does not ship the `{}` calendar",
                    settings.calendars.civil_calendar
                ),
            )
            .with_field("calendars.civil_calendar")
        })?;
        let ayanamsha = match settings.frame.ayanamsha {
            AyanamshaChoice::Catalogued { id } => id,
            AyanamshaChoice::Custom { .. } => Ayanamsha::Lahiri,
        };
        let model = DrikSun::new(
            provider,
            ayanamsha,
            settings.day.sunrise,
            settings.provider.overrides,
            ctx.delta_t(),
        );
        if asked.instants.is_null() && asked.instant_count != 0 {
            return Err(crate::support::null("instants"));
        }
        // SAFETY: the entry point's contract — the caller promises
        // `instant_count` readable doubles at `instants`.
        let instants: Vec<JulianDay<Utc>> =
            unsafe { core::slice::from_raw_parts(asked.instants, asked.instant_count) }
                .iter()
                .map(|jd| JulianDay::<Utc>::literal(*jd))
                .collect();
        let founded = Founder::new(
            provider,
            resolved,
            &model,
            calendar,
            &clock,
            PrecessionModel::default(),
            ctx.delta_t(),
        )
        .found(&instants, &place, kind)?;
        let mut provenance = founded.provenance;
        // The founder leaves the placeholder; the boundary fills it, as
        // `ts_positions` does. Whether the producers should seal instead
        // is `serial-and-the-envelope.md` §8's open question, and this is
        // the first place it shows: a Rust caller's envelope still
        // carries the placeholder where a binding's blob carries a hash.
        provenance.content_hash = content_hash(&founded.value);
        let encoded = encode(&founded.value, &place, kind, &provenance)?;
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_blob, "out_blob", TsBlob::from_vec(encoded)) }
    })
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::float_cmp,
        clippy::cast_possible_wrap,
        reason = "a test fails by panicking, indexes its own blob and compares the numbers it wrote"
    )]

    use super::{
        Altitude, Ayanamsha, ChartKind, DrikSun, Founder, JulianDay, Latitude, Longitude, Place,
        PrecessionModel, Provenance, TsDayPart, TsDayState, TsGhatiReckoning, TsHoraReckoning,
        TsPolarDayPolicy, TsPolarKind, TsReading, TsSunrise, Utc, UtcOffset, shipped,
    };
    use teistro_chart::bhava::Reading;
    use teistro_chart::day::DayPart;
    use teistro_core::settings::{GhatiReckoning, HoraReckoning, PolarDayPolicy, Sunrise};
    use teistro_time::local_day::{DayState, PolarKind};

    /// Every member of every knob crosses, and crosses to an id of its
    /// own.
    ///
    /// This is the guard the three knobs cannot get from a match. A
    /// settings knob is `#[non_exhaustive]`, so a member added later
    /// falls into a wildcard rather than breaking the build; iterating
    /// the knob's own `ALL` fails here instead, naming the member that
    /// has no id.
    #[test]
    fn every_knob_member_crosses_to_an_id_of_its_own() {
        let sunrises: Vec<u8> = Sunrise::ALL
            .iter()
            .map(|member| {
                TsSunrise::of(*member)
                    .unwrap_or_else(|| panic!("`{member}` has no id at the boundary"))
                    as u8
            })
            .collect();
        assert_eq!(sunrises, vec![0, 1, 2], "one id each, in order");

        let ghatis: Vec<u8> = GhatiReckoning::ALL
            .iter()
            .map(|member| {
                TsGhatiReckoning::of(*member)
                    .unwrap_or_else(|| panic!("`{member}` has no id at the boundary"))
                    as u8
            })
            .collect();
        assert_eq!(ghatis, vec![0, 1]);

        let horas: Vec<u8> = HoraReckoning::ALL
            .iter()
            .map(|member| {
                TsHoraReckoning::of(*member)
                    .unwrap_or_else(|| panic!("`{member}` has no id at the boundary"))
                    as u8
            })
            .collect();
        assert_eq!(horas, vec![0, 1]);
    }

    /// Two charts founded over the analytic test provider, with the
    /// provenance the boundary would seal.
    ///
    /// Two rather than one: a batch of one cannot tell a charts-outermost
    /// layout from a scalar one, so every per-chart section here holds
    /// rows that differ.
    fn founded() -> (
        Vec<teistro_chart::foundation::ChartFoundation>,
        Place,
        Provenance,
    ) {
        use teistro_core::envelope::content_hash;
        use teistro_core::settings::{DEFAULT_PROFILE, Profile, SettingsPatch};
        use teistro_port_ephemeris::test_provider::TestProvider;

        let provider = TestProvider;
        let resolved = Profile::shipped(DEFAULT_PROFILE)
            .expect("the default profile")
            .resolve(&SettingsPatch::default())
            .expect("it resolves");
        let model = DrikSun::new(
            &provider,
            Ayanamsha::Lahiri,
            resolved.settings.day.sunrise,
            resolved.settings.provider.overrides,
            teistro_astro::delta_t::DeltaTModel::TableThenModel,
        );
        let clock = UtcOffset::try_from_seconds(5 * 3600 + 45 * 60).expect("in range");
        let calendar = shipped(resolved.settings.calendars.civil_calendar)
            .expect("the profile's calendar ships");
        let place = Place::new(
            Latitude::literal(27.7172),
            Longitude::literal(85.3240),
            Altitude::literal(1400.0),
        );
        let founded = Founder::new(
            &provider,
            &resolved,
            &model,
            calendar,
            &clock,
            PrecessionModel::default(),
            teistro_astro::delta_t::DeltaTModel::TableThenModel,
        )
        .found(
            &[
                JulianDay::<Utc>::literal(2_460_482.5),
                JulianDay::<Utc>::literal(2_460_600.25),
            ],
            &place,
            ChartKind::Natal,
        )
        .expect("two founded charts");
        let mut provenance = founded.provenance;
        provenance.content_hash = content_hash(&founded.value);
        (founded.value, place, provenance)
    }

    /// A batch of founded charts crosses, and reads back as what was
    /// founded.
    ///
    /// The whole point of the module: charts computed in Rust, written
    /// to their blob and read out of it again, compared field by field
    /// against the values. Nothing else checks that the encoder and the
    /// schema agree — a column written in the wrong order would still
    /// decode, into wrong numbers — nor that the second chart's rows sit
    /// where the layout says they do.
    #[test]
    fn a_batch_of_founded_charts_round_trips_through_its_blob() {
        use teistro_idl::blob::Reader;

        let (charts, place, provenance) = founded();
        let bytes =
            super::encode(&charts, &place, ChartKind::Natal, &provenance).expect("it encodes");
        let schema = crate::schemas::charts();
        let reader = Reader::parse(&bytes, &schema).expect("a well-formed blob");
        let graha_count = charts[0].grahas.len();

        let summary = reader.fixed("summary").expect("the summary");
        assert_eq!(summary[0].as_i64(), i64::from(ChartKind::Natal.id()));
        assert_eq!(summary[1].as_i64(), charts.len() as i64);
        assert_eq!(summary[2].as_i64(), graha_count as i64);
        assert_eq!(summary[3].as_f64(), place.latitude.get());

        let instants = reader.column("cast", "instant").expect("the instants");
        let lagnas = reader.column("cast", "lagna_deg").expect("the lagnas");
        assert_eq!(instants.len(), charts.len());
        for (row, chart) in charts.iter().enumerate() {
            assert_eq!(instants[row].as_f64(), chart.instant.get());
            assert_eq!(lagnas[row].as_f64(), chart.lagna_deg);
        }
        assert_ne!(
            lagnas[0].as_f64(),
            lagnas[1].as_f64(),
            "two instants, two lagnas: a constant column would prove nothing"
        );

        // Charts outermost: row `i * graha_count + j` is chart `i`, graha
        // `j`. The second chart's grahas are what a scalar layout would
        // put in the first chart's rows.
        let longitudes = reader
            .column("grahas", "longitude_deg")
            .expect("the grahas");
        assert_eq!(longitudes.len(), charts.len() * graha_count);
        for (i, chart) in charts.iter().enumerate() {
            for (j, graha) in chart.grahas.iter().enumerate() {
                assert_eq!(
                    longitudes[i * graha_count + j].as_f64(),
                    graha.longitude_deg,
                    "chart {i}, graha {j}"
                );
            }
        }

        let madhya = reader.column("houses", "madhya_deg").expect("the houses");
        assert_eq!(madhya.len(), charts.len() * 12);
        for (i, chart) in charts.iter().enumerate() {
            for (j, bhava) in chart.houses.madhya.iter().enumerate() {
                assert_eq!(madhya[i * 12 + j].as_f64(), *bhava, "chart {i}, bhava {j}");
            }
        }

        let sunrise = reader.column("day", "sunrise").expect("the sunrises");
        let vara = reader.column("day", "vara").expect("the varas");
        let ghati = reader.column("timing", "ghati").expect("the ghatis");
        for (row, chart) in charts.iter().enumerate() {
            assert_eq!(sunrise[row].as_f64(), chart.day.day.sunrise.get());
            assert_eq!(vara[row].as_i64(), i64::from(chart.day.day.vara.id()));
            assert_eq!(ghati[row].as_i64(), i64::from(chart.timing.ishtakaal.ghati));
        }

        let model = reader.bytes("model").expect("the model");
        assert_eq!(model, charts[0].day.day.model.as_bytes());
    }

    /// A batch of none is a blob, not an error.
    ///
    /// A caller that filtered a list to nothing gets an empty answer
    /// rather than a refusal, which is what lets a binding pass a list
    /// straight through. What the request knows is still written; what
    /// only founding could tell is zero, and `chart_count` says so.
    #[test]
    fn a_batch_of_no_charts_is_still_a_well_formed_blob() {
        use teistro_idl::blob::Reader;

        let (_, place, provenance) = founded();
        let bytes = super::encode(&[], &place, ChartKind::Natal, &provenance).expect("it encodes");
        let schema = crate::schemas::charts();
        let reader = Reader::parse(&bytes, &schema).expect("a well-formed blob");

        let summary = reader.fixed("summary").expect("the summary");
        assert_eq!(summary[0].as_i64(), i64::from(ChartKind::Natal.id()));
        assert_eq!(summary[1].as_i64(), 0, "no charts");
        assert_eq!(summary[2].as_i64(), 0, "and so no grahas each");
        assert_eq!(summary[3].as_f64(), place.latitude.get(), "but a place");
        assert!(reader.column("cast", "instant").expect("empty").is_empty());
        assert!(reader.column("day", "sunrise").expect("empty").is_empty());
        assert_eq!(reader.bytes("model").expect("the model"), b"");
        assert!(
            !reader.bytes("provenance").expect("the envelope").is_empty(),
            "the settings that founded nothing are still stamped"
        );
    }

    /// A day's state splits into the three scalars a blob carries.
    ///
    /// `DayState::Polar` carries two payload fields, which is what made
    /// the first version of §8's rule — a kind and one value — too
    /// narrow. A normal day leaves both at nought, so a reader that
    /// checks the kind first never looks at them.
    #[test]
    fn a_days_state_splits_into_a_kind_and_its_payload() {
        assert_eq!(
            TsDayState::split(DayState::Normal),
            (TsDayState::Normal, 0, 0)
        );
        for kind in [PolarKind::Day, PolarKind::Night] {
            for policy in PolarDayPolicy::ALL {
                let (state, polar, applied) = TsDayState::split(DayState::Polar {
                    kind,
                    policy: *policy,
                });
                assert_eq!(state, TsDayState::Polar);
                assert_eq!(polar, TsPolarKind::from(kind) as u8);
                assert_eq!(
                    applied,
                    TsPolarDayPolicy::of(*policy)
                        .unwrap_or_else(|| panic!("`{policy}` has no id at the boundary"))
                        as u8
                );
            }
        }
    }

    /// The two chart-layer enums cross exhaustively, so a variant added
    /// to either breaks the build rather than this test; what this holds
    /// is that no two share an id.
    #[test]
    fn the_chart_enums_cross_to_ids_of_their_own() {
        assert_eq!(TsReading::from(Reading::Sandhi) as u8, 0);
        assert_eq!(TsReading::from(Reading::Madhya) as u8, 1);
        assert_eq!(TsDayPart::from(DayPart::Daylight) as u8, 0);
        assert_eq!(TsDayPart::from(DayPart::Night) as u8, 1);
    }
}
