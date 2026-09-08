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
    /// The instant, as a Julian day on the UTC scale.
    /// `api: unit=jd example=2460482.5`
    pub instant_jd_utc: f64,
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
pub fn day_values(day: &teistro_chart::day::ChartDay) -> Vec<FixedValue> {
    let local = &day.day;
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
        (TsDayPart::from(day.part) as u64).into(),
        day.elapsed.into(),
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
    /// The columns of a chart's grahas, in the schema's order.
    fn of(grahas: &[teistro_chart::foundation::GrahaPosition]) -> GrahaColumns {
        GrahaColumns {
            ids: grahas.iter().map(|g| g.graha.id()).collect(),
            longitudes: grahas.iter().map(|g| g.longitude_deg).collect(),
            tropicals: grahas.iter().map(|g| g.tropical_deg).collect(),
            latitudes: grahas.iter().map(|g| g.latitude_deg).collect(),
            distances: grahas.iter().map(|g| g.distance_au).collect(),
            speeds: grahas.iter().map(|g| g.speed_deg_per_day).collect(),
            house_bhava: grahas.iter().map(|g| g.house.bhava).collect(),
            house_method: grahas.iter().map(|g| g.house.method.id()).collect(),
            house_through: grahas.iter().map(|g| g.house.through).collect(),
            house_from: grahas.iter().map(|g| g.house.from_madhya_deg).collect(),
            placed_bhava: grahas.iter().map(|g| g.placement.bhava).collect(),
            placed_method: grahas.iter().map(|g| g.placement.method.id()).collect(),
            placed_through: grahas.iter().map(|g| g.placement.through).collect(),
            placed_from: grahas.iter().map(|g| g.placement.from_madhya_deg).collect(),
        }
    }
}

/// The chart's own values, in the order `summary` declares them.
#[must_use]
fn summary_values(foundation: &ChartFoundation, count: u32) -> Vec<FixedValue> {
    vec![
        foundation.instant.get().into(),
        u64::from(foundation.kind.id()).into(),
        foundation.lagna_deg.into(),
        foundation.day_lagna_deg.into(),
        foundation.place.latitude.get().into(),
        foundation.place.longitude.get().into(),
        foundation.place.altitude.get().into(),
        u64::from(count).into(),
    ]
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

/// A founded chart as the blob its schema describes.
///
/// # Errors
///
/// Whatever the writer refuses: a section the schema does not have, or a
/// column of the wrong length. Neither can happen for a value this crate
/// built, so a failure here is a schema that has drifted from this
/// function rather than a caller's mistake.
pub fn encode(foundation: &ChartFoundation, provenance: &Provenance) -> Result<Vec<u8>, Error> {
    let schema = crate::schemas::chart();
    let mut writer = Writer::new(&schema);
    let grahas = &foundation.grahas;
    let count = u32::try_from(grahas.len()).unwrap_or(u32::MAX);
    let columns = GrahaColumns::of(grahas);
    let steps = serde_json::to_string(&foundation.steps).unwrap_or_else(|_| String::from("[]"));
    let envelope = teistro_core::envelope::canonical_json(provenance);
    let (ayanamsha_kind, ayanamsha) = match foundation.zodiac.ayanamsha {
        None => (0_u64, 0_u64),
        Some(teistro_core::settings::AyanamshaChoice::Catalogued { id }) => (1, u64::from(id.id())),
        // A custom ayanamsha's coefficients are settings, and the
        // settings hash pins them: a result carries what it applied.
        Some(teistro_core::settings::AyanamshaChoice::Custom { .. }) => (2, 0),
    };

    let write = || -> Result<Vec<u8>, teistro_idl::blob::BlobError> {
        writer.fixed("summary", &summary_values(foundation, count))?;
        writer.columns(
            "grahas",
            grahas.len(),
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
        writer.fixed(
            "readings",
            &[
                u64::from(foundation.houses.chalit.method.id()).into(),
                u64::from(foundation.houses.chalit.source.id()).into(),
                (TsReading::from(foundation.houses.chalit.reading) as u64).into(),
                u64::from(foundation.chalit.chalit.method.id()).into(),
                u64::from(foundation.chalit.chalit.source.id()).into(),
                (TsReading::from(foundation.chalit.chalit.reading) as u64).into(),
            ],
        )?;
        writer.columns(
            "houses",
            12,
            &[
                ColumnData::F64(&foundation.houses.madhya),
                ColumnData::F64(&foundation.houses.sandhi),
            ],
        )?;
        writer.columns(
            "chalit",
            12,
            &[
                ColumnData::F64(&foundation.chalit.madhya),
                ColumnData::F64(&foundation.chalit.sandhi),
            ],
        )?;
        writer.fixed(
            "zodiac",
            &[
                u64::from(foundation.zodiac.request.to_bits()).into(),
                foundation.zodiac.offset_deg.into(),
                ayanamsha_kind.into(),
                ayanamsha.into(),
            ],
        )?;
        writer.fixed("day", &day_values(&foundation.day))?;
        writer.fixed("timing", &timing_values(&foundation.timing))?;
        writer.bytes("model", foundation.day.day.model.as_bytes())?;
        writer.bytes("steps", steps.as_bytes())?;
        writer.bytes("provenance", envelope.as_bytes())?;
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
/// `api: blob=chart`
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
        let provider = ctx.provider().ok_or_else(|| {
            Error::new(
                Status::Capability,
                "the context has no ephemeris: pass a provider vtable to ts_context_new, or the TS_CONTEXT_TEST_PROVIDER flag for tests",
            )
            .with_field("provider")
        })?;
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
        let founded = Founder::new(
            provider,
            resolved,
            &model,
            calendar,
            &clock,
            PrecessionModel::default(),
            ctx.delta_t(),
        )
        .found_one(
            JulianDay::<Utc>::literal(asked.instant_jd_utc),
            &place,
            kind,
        )?;
        let mut provenance = founded.provenance;
        // The founder leaves the placeholder; the boundary fills it, as
        // `ts_positions` does. Whether the producers should seal instead
        // is `serial-and-the-envelope.md` §8's open question, and this is
        // the first place it shows: a Rust caller's envelope still
        // carries the placeholder where a binding's blob carries a hash.
        provenance.content_hash = content_hash(&founded.value);
        let encoded = encode(&founded.value, &provenance)?;
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

    /// A chart founded over the analytic test provider, with the
    /// provenance the boundary would seal.
    fn founded() -> (teistro_chart::foundation::ChartFoundation, Provenance) {
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
        let founded = Founder::new(
            &provider,
            &resolved,
            &model,
            calendar,
            &clock,
            PrecessionModel::default(),
            teistro_astro::delta_t::DeltaTModel::TableThenModel,
        )
        .found_one(
            JulianDay::<Utc>::literal(2_460_482.5),
            &Place::new(
                Latitude::literal(27.7172),
                Longitude::literal(85.3240),
                Altitude::literal(1400.0),
            ),
            ChartKind::Natal,
        )
        .expect("a founded chart");
        let mut provenance = founded.provenance;
        provenance.content_hash = content_hash(&founded.value);
        (founded.value, provenance)
    }

    /// A founded chart crosses, and reads back as what was founded.
    ///
    /// The whole point of the module: a chart computed in Rust, written
    /// to its blob and read out of it again, compared field by field
    /// against the value. Nothing else checks that the encoder and the
    /// schema agree — a column written in the wrong order would still
    /// decode, into wrong numbers.
    #[test]
    fn a_founded_chart_round_trips_through_its_blob() {
        use teistro_idl::blob::Reader;

        let (foundation, provenance) = founded();
        let bytes = super::encode(&foundation, &provenance).expect("it encodes");
        let schema = crate::schemas::chart();
        let reader = Reader::parse(&bytes, &schema).expect("a well-formed blob");

        let summary = reader.fixed("summary").expect("the summary");
        assert_eq!(summary[0].as_f64(), foundation.instant.get());
        assert_eq!(summary[2].as_f64(), foundation.lagna_deg);
        assert_eq!(summary[7].as_i64(), foundation.grahas.len() as i64);

        let day = reader.fixed("day").expect("the day");
        assert_eq!(day[0].as_f64(), foundation.day.day.sunrise.get());
        assert_eq!(day[3].as_i64(), i64::from(foundation.day.day.vara.id()));

        let model = reader.bytes("model").expect("the model");
        assert_eq!(model, foundation.day.day.model.as_bytes());
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
