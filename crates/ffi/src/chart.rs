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

use core::ffi::c_char;

use teistro::ChartRequest;
use teistro::render_svg::Theme;
use teistro_aspect::drishti::Strength;
use teistro_chart::bhava::Reading;
use teistro_chart::day::DayPart;
use teistro_chart::foundation::ChartFoundation;
use teistro_core::catalogue::{ChartKind, DashaSystem, Kind, Varga};
use teistro_core::envelope::Provenance;
use teistro_core::error::{Error, Status};
use teistro_core::key::KeyId;
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro_core::time::UtcOffset;
use teistro_houses::classify::Quadrant;
use teistro_idl::blob::{ColumnData, FixedValue, Writer};
use teistro_serial::Document;
use teistro_state::burn::Burning;

use crate::blob::TsBlob;
use crate::context::TsContext;
use crate::string::TsString;
use crate::support::{c_struct, optional_text, read_in, slice, with_context, write_plain};
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

/// How badly the Sun burns a body.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsBurning {
    /// Far enough from the Sun to be itself.
    None = 0,
    /// Combust.
    Combust = 1,
    /// Deeply combust; only a table that gives a deeper orb reaches it.
    Deep = 2,
}

impl From<Burning> for TsBurning {
    fn from(burning: Burning) -> TsBurning {
        match burning {
            Burning::None => TsBurning::None,
            Burning::Combust => TsBurning::Combust,
            Burning::Deep => TsBurning::Deep,
        }
    }
}

/// Which third of the wheel a bhava stands in.
///
/// The houses crate's own `Quadrant`, which is not a catalogue member —
/// it is a classification of a number rather than a thing with a key —
/// so it crosses as this boundary's own enum, as `TsStrength` does.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsQuadrant {
    /// Angular: the 1st, 4th, 7th and 10th.
    Kendra = 0,
    /// Succedent: the 2nd, 5th, 8th and 11th.
    Panapara = 1,
    /// Cadent: the 3rd, 6th, 9th and 12th.
    Apoklima = 2,
}

impl From<Quadrant> for TsQuadrant {
    fn from(quadrant: Quadrant) -> TsQuadrant {
        match quadrant {
            Quadrant::Kendra => TsQuadrant::Kendra,
            Quadrant::Panapara => TsQuadrant::Panapara,
            Quadrant::Apoklima => TsQuadrant::Apoklima,
        }
    }
}

/// How strongly one body looks at another.
///
/// The aspect crate's own `Strength`, which is not a catalogue member —
/// it is a property of a relation rather than a thing with a key — so it
/// crosses as this boundary's own enum, as `TsReading` and `TsDayPart`
/// do.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsStrength {
    /// No aspect at all.
    None = 0,
    /// A quarter aspect: the third and tenth.
    Quarter = 1,
    /// A half aspect: the fifth and ninth.
    Half = 2,
    /// A three-quarter aspect: the fourth and eighth.
    ThreeQuarters = 3,
    /// A full aspect: the seventh, and a special graha's own two houses.
    Full = 4,
}

impl From<Strength> for TsStrength {
    fn from(strength: Strength) -> TsStrength {
        match strength {
            Strength::None => TsStrength::None,
            Strength::Quarter => TsStrength::Quarter,
            Strength::Half => TsStrength::Half,
            Strength::ThreeQuarters => TsStrength::ThreeQuarters,
            Strength::Full => TsStrength::Full,
        }
    }
}

/// How a dasha's balance at birth was measured.
///
/// The settings' own `Balance`, which is a knob and not a catalogue member,
/// so it crosses as this boundary's own enum, as `TsStrength` does.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsBalance {
    /// By the elapsed part of the Moon's window of nakshatras.
    Spatial = 0,
    /// By the elapsed part of the Moon's stay in its nakshatra.
    Temporal = 1,
}

impl From<teistro_core::settings::Balance> for TsBalance {
    fn from(balance: teistro_core::settings::Balance) -> TsBalance {
        match balance {
            teistro_core::settings::Balance::Temporal => TsBalance::Temporal,
            _ => TsBalance::Spatial,
        }
    }
}

/// Where an Ashtakavarga's reductions and pindas were made: the settings'
/// own `Shodhana`, which is a knob and not a catalogue member.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsShodhana {
    /// In each graha's own Ashtakavarga (BPHS chs. 67 to 69).
    EachGraha = 0,
    /// On the sum of the seven, as the conformance corpus's engine makes them.
    Sarva = 1,
}

impl From<teistro_core::settings::Shodhana> for TsShodhana {
    fn from(shodhana: teistro_core::settings::Shodhana) -> TsShodhana {
        match shodhana {
            teistro_core::settings::Shodhana::Sarva => TsShodhana::Sarva,
            _ => TsShodhana::EachGraha,
        }
    }
}

/// How an Ashtakavarga's Ekadhipatya reduction treated a co-ruled sign beside
/// an occupied one: the settings' own `Ekadhipatya`.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsEkadhipatya {
    /// BPHS ch. 68: an empty sign keeps a difference.
    Bphs = 0,
    /// The empty sign always goes to zero.
    EmptyToZero = 1,
}

impl From<teistro_core::settings::Ekadhipatya> for TsEkadhipatya {
    fn from(rule: teistro_core::settings::Ekadhipatya) -> TsEkadhipatya {
        match rule {
            teistro_core::settings::Ekadhipatya::EmptyToZero => TsEkadhipatya::EmptyToZero,
            _ => TsEkadhipatya::Bphs,
        }
    }
}

/// How a Vimshopaka scored a graha in a varga: the settings' own
/// `Vimshopaka`.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsVimshopakaScoring {
    /// BPHS ch. 7: 20 in exaltation or the own sign, else by the compound
    /// relationship with the sign's lord.
    Bphs = 0,
    /// The conformance corpus's engine: the Saptavargaja virupas over 45 by
    /// natural friendship, rounded to hundredths.
    SaptavargajaVirupas = 1,
}

impl From<teistro_core::settings::Vimshopaka> for TsVimshopakaScoring {
    fn from(scoring: teistro_core::settings::Vimshopaka) -> TsVimshopakaScoring {
        match scoring {
            teistro_core::settings::Vimshopaka::SaptavargajaVirupas => {
                TsVimshopakaScoring::SaptavargajaVirupas
            }
            _ => TsVimshopakaScoring::Bphs,
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
    /// Which of the document's sections to compute beside the
    /// foundation, as a bit set: 1 the day's almanac, 2 the planetary
    /// states, 4 the aspects, 8 the derived points, 16 the houses
    /// service, 32 the Ashtakavarga, 64 the Vimshopaka, 128 the Shadbala, 256 the Bhava bala. Zero for the foundation alone, which is what every
    /// caller compiled against an earlier header passes by not passing
    /// it at all.
    ///
    /// A bit set here and a named option in every ergonomic layer, which
    /// is the split `ts_frame_pack` already has: nothing but a generated
    /// layer writes bits (`03-design/chart-reading.md` §5).
    /// `api: example=0`
    pub sections: u32,
    /// Reserved; write zero.
    pub reserved_sections: u32,
    /// Which divisional charts to compute, as catalogue ids, in the
    /// order they should be answered in; null with a count of zero for
    /// none, as `instants` takes a grid of none.
    ///
    /// **Not `nullable`**, and that is the description's word rather
    /// than a promise about the pointer: `nullable` makes the generated
    /// field an `Option` of the whole parameter, and an optional *array
    /// of enum members* is a shape no emitter has been shown — it mapped
    /// the option's contents where it meant to map the array's. An empty
    /// array says "none" without needing one, which is what `instants`
    /// already does.
    /// `api: len=varga_count enum=Varga`
    pub vargas: *const u16,
    /// How many divisional charts `vargas` points at.
    pub varga_count: usize,
    /// Which charts to draw, and in which layouts, in the order they should
    /// be answered in: each `layout_id << 16 | varga_id`, a `chart_layout`
    /// catalogue id and a `Varga` id, `D1` for the founded chart. Null with a
    /// count of zero for none.
    ///
    /// Packed, as `sections` is a bit set, so the request carries one array
    /// and one count rather than two arrays that must agree; every ergonomic
    /// layer takes named pairs and writes the bits (`03-design/chart-geometry.md`).
    /// `api: len=drawing_count`
    pub drawings: *const u32,
    /// How many drawings `drawings` points at.
    pub drawing_count: usize,
    /// Which dashas to compute, as catalogue ids, in the order they should be
    /// answered in: each one's balance and its periods to the settings'
    /// `dasha.depth`. Null with a count of zero for none.
    /// `api: len=dasha_count enum=DashaSystem`
    pub dashas: *const u16,
    /// How many dashas `dashas` points at.
    pub dasha_count: usize,
    /// A theme to write every drawing as SVG in, as JSON: an object of
    /// `style` and `content` naming only what it changes, over the light
    /// theme or the shipped one its `extends` names (`{"extends": "dark"}`).
    /// The SVGs come back in the blob's `svgs` section, in the context's
    /// locale. Null for none, which costs nothing
    /// (`03-design/render-svg.md`).
    /// `api: nullable example={"extends":"dark"}`
    pub theme_json: *const c_char,
}

// **The handshake, which this struct carried and nothing read.**
// `struct_size` is documented as "`sizeof(ts_chart_request)` as the caller compiled
// it", and the entry point below dereferenced the pointer raw: a caller
// compiled against an older header passed a shorter struct and the
// library read past it, which is undefined behaviour rather than the
// `SCHEMA_VERSION` refusal the field exists to give. Eleven of the
// thirteen boundary structs with the field were registered here; these
// two were not, and they are the two biggest requests.
// `check-lints`' `handshake-is-checked` holds the class now.
c_struct!(TsChartRequest);

/// Which document sections a `sections` bit set asks for, as the
/// request's own vocabulary.
///
/// **The bits are the boundary's and the names are the façade's**, and
/// the translation is here rather than in `teistro` on purpose: these
/// five values are in `teistro.h` and are therefore an ABI, while the
/// façade's own set is an implementation detail that must stay free to
/// change. A `Reading` is built by naming what is wanted, which is what
/// makes an unknown bit a silent no rather than a wrong section — and
/// what makes the table below the only place the mapping is written
/// (`03-design/chart-reading.md` §5).
type SectionBit = (u32, fn(ChartRequest) -> ChartRequest);

/// `TS_CHART_ASPECTS`, the bit a caller sets for the drishti.
///
/// Declared here beside the table and named in the header, because the
/// bits are the boundary's vocabulary: a consumer of the C ABI writes
/// `TS_CHART_ASPECTS`, and every generated layer writes a named option
/// instead (`03-design/chart-reading.md` §5).
pub const TS_CHART_PANCHANGA: u32 = 1;
/// The planetary states.
pub const TS_CHART_STATE: u32 = 2;
/// The drishti.
pub const TS_CHART_ASPECTS: u32 = 4;
/// The derived points.
pub const TS_CHART_POINTS: u32 = 8;
/// The houses service.
pub const TS_CHART_HOUSES: u32 = 16;
/// The Ashtakavarga.
pub const TS_CHART_ASHTAKAVARGA: u32 = 32;
/// The Vimshopaka.
pub const TS_CHART_VIMSHOPAKA: u32 = 64;
/// The Shadbala.
pub const TS_CHART_SHADBALA: u32 = 128;
/// The Bhava bala.
pub const TS_CHART_BHAVA_BALA: u32 = 256;

const SECTION_BITS: [SectionBit; 9] = [
    (TS_CHART_PANCHANGA, ChartRequest::with_panchanga),
    (TS_CHART_STATE, ChartRequest::with_state),
    (TS_CHART_ASPECTS, ChartRequest::with_aspects),
    (TS_CHART_POINTS, ChartRequest::with_points),
    (TS_CHART_HOUSES, ChartRequest::with_houses),
    (TS_CHART_ASHTAKAVARGA, ChartRequest::with_ashtakavarga),
    (TS_CHART_VIMSHOPAKA, ChartRequest::with_vimshopaka),
    (TS_CHART_SHADBALA, ChartRequest::with_shadbala),
    (TS_CHART_BHAVA_BALA, ChartRequest::with_bhava_bala),
];

/// The reading a bit set asks for, added to a request.
fn sections_of(bits: u32, mut request: ChartRequest) -> ChartRequest {
    for (bit, add) in SECTION_BITS {
        if bits & bit == bit {
            request = add(request);
        }
    }
    request
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
    fn of(charts: &[&ChartFoundation]) -> GrahaColumns {
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
    varga_count: u32,
    dasha_count: u32,
) -> Vec<FixedValue> {
    vec![
        u64::from(kind.id()).into(),
        u64::from(chart_count).into(),
        u64::from(graha_count).into(),
        u64::from(varga_count).into(),
        u64::from(dasha_count).into(),
        place.latitude.get().into(),
        place.longitude.get().into(),
        place.altitude.get().into(),
    ]
}

/// How many grahas every chart in the batch holds, or `INTERNAL` for a
/// batch that mixes sizes.
///
/// The blob's layout is one count for the batch, and only this crate
/// could have built a value that disagrees with itself.
fn one_size(charts: &[&ChartFoundation]) -> Result<usize, Error> {
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
    Ok(graha_count)
}

/// The drishti of a batch, as the section carries them.
///
/// **Charts outermost**, as every per-chart section here is, and a batch
/// whose charts hold different numbers of relations is `INTERNAL` for
/// the reason a batch of differing graha counts is: the layout is one
/// count for the batch. That is not a restriction in practice — the
/// relations are a function of the grahas' signs, and a batch founded
/// from one request over one place has the same nine bodies in every
/// chart — but it is a fact the layout depends on, so it is checked
/// rather than assumed.
struct AspectColumns {
    /// How many relations each chart holds, one entry per chart.
    ///
    /// **Per chart and not one for the batch**, and the check that would
    /// have enforced a batch-wide count is what found out: a chart's
    /// drishti are a function of where the bodies stand rather than of
    /// how many there are, and two charts of the same nine grahas at one
    /// place hold 47 relations and 40. So the rows are concatenated and
    /// a reader prefix-sums these, which is the panchanga blob's own
    /// rule for a ragged list.
    counts: Vec<u32>,
    /// The drishti table every one of them was read under, which is one
    /// to a batch because it is a setting.
    table: String,
    from: Vec<u16>,
    to: Vec<u16>,
    houses: Vec<u8>,
    strength: Vec<u8>,
    from_sign: Vec<f64>,
    from_nakshatra: Vec<f64>,
    from_pada: Vec<f64>,
    to_sign: Vec<f64>,
    to_nakshatra: Vec<f64>,
    to_pada: Vec<f64>,
}

impl AspectColumns {
    fn of(documents: &[Document]) -> AspectColumns {
        let rows: usize = documents
            .iter()
            .map(|d| d.aspects.as_ref().map_or(0, |a| a.all().len()))
            .sum();
        let mut columns = AspectColumns {
            counts: Vec::with_capacity(documents.len()),
            table: documents
                .first()
                .and_then(|d| d.aspects.as_ref())
                .map_or_else(String::new, |a| a.table().to_owned()),
            from: Vec::with_capacity(rows),
            to: Vec::with_capacity(rows),
            houses: Vec::with_capacity(rows),
            strength: Vec::with_capacity(rows),
            from_sign: Vec::with_capacity(rows),
            from_nakshatra: Vec::with_capacity(rows),
            from_pada: Vec::with_capacity(rows),
            to_sign: Vec::with_capacity(rows),
            to_nakshatra: Vec::with_capacity(rows),
            to_pada: Vec::with_capacity(rows),
        };
        for document in documents {
            let Some(aspects) = document.aspects.as_ref() else {
                columns.counts.push(0);
                continue;
            };
            columns
                .counts
                .push(u32::try_from(aspects.all().len()).unwrap_or(u32::MAX));
            for drishti in aspects.all() {
                columns.from.push(drishti.from.id());
                columns.to.push(drishti.to.id());
                columns.houses.push(drishti.houses);
                columns
                    .strength
                    .push(TsStrength::from(drishti.strength) as u8);
                columns.from_sign.push(drishti.from_edge.sign_deg);
                columns.from_nakshatra.push(drishti.from_edge.nakshatra_deg);
                columns.from_pada.push(drishti.from_edge.pada_deg);
                columns.to_sign.push(drishti.to_edge.sign_deg);
                columns.to_nakshatra.push(drishti.to_edge.nakshatra_deg);
                columns.to_pada.push(drishti.to_edge.pada_deg);
            }
        }
        columns
    }

    /// The section and the table it was read under.
    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "aspects",
            self.from.len(),
            &[
                ColumnData::U16(&self.from),
                ColumnData::U16(&self.to),
                ColumnData::U8(&self.houses),
                ColumnData::U8(&self.strength),
                ColumnData::F64(&self.from_sign),
                ColumnData::F64(&self.from_nakshatra),
                ColumnData::F64(&self.from_pada),
                ColumnData::F64(&self.to_sign),
                ColumnData::F64(&self.to_nakshatra),
                ColumnData::F64(&self.to_pada),
            ],
        )?;
        writer.bytes("drishti_table", self.table.as_bytes())
    }
}

/// What each graha is, as the section carries it.
///
/// One row per graha per chart and no count: a state is a reading of a
/// placement, so there is one per placement.
struct StateColumns {
    /// The combustion table every `burning` was judged against, one to a
    /// batch because it is a setting.
    table: String,
    rows: Vec<Vec<FixedValue>>,
}

/// A set of catalogue members as a bit set: bit `n` is the member with
/// id `n`.
///
/// Six members in the only enum this is used for, so a `u32` holds any
/// of the three lists with room to spare — and a set stays a set rather
/// than becoming three ragged sections with three prefix sums.
fn bits_of<M: teistro_core::catalogue::Catalogued>(members: &[M]) -> u64 {
    members
        .iter()
        .fold(0_u64, |set, member| set | (1_u64 << member.id()))
}

impl StateColumns {
    fn of(documents: &[Document]) -> StateColumns {
        let mut columns = StateColumns {
            table: String::new(),
            rows: Vec::new(),
        };
        for document in documents {
            let Some(states) = document.state.as_ref() else {
                continue;
            };
            for state in states {
                columns.rows.push(state_values(state));
            }
        }
        if documents.iter().any(|d| d.state.is_some()) {
            // The table is the settings', and every state in the batch
            // was judged against the same one.
            columns.table = String::from("settings.state.combustion_orbs");
        }
        columns
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.rows("states", &self.rows)?;
        writer.bytes("combustion_orbs", self.table.as_bytes())
    }
}

/// One graha's state, in the order `states` declares its columns.
#[must_use]
fn state_values(state: &teistro_state::GrahaState) -> Vec<FixedValue> {
    let flag = |yes: bool| FixedValue::from(u64::from(yes));
    let friendship = &state.friendship;
    let war = state.war;
    vec![
        u64::from(state.graha.id()).into(),
        u64::from(state.sign.id()).into(),
        u64::from(state.house).into(),
        u64::from(state.dignity.id()).into(),
        u64::from(friendship.natural.id()).into(),
        u64::from(friendship.temporary.id()).into(),
        u64::from(friendship.compound.id()).into(),
        flag(friendship.dispositor.is_some()),
        u64::from(
            friendship
                .dispositor
                .map_or(0, teistro_core::catalogue::Graha::id),
        )
        .into(),
        (TsBurning::from(state.combustion.burning) as u64).into(),
        flag(state.combustion.from_sun_deg.is_some()),
        state.combustion.from_sun_deg.unwrap_or(0.0).into(),
        flag(state.combustion.orbs.is_some()),
        state.combustion.orbs.map_or(0.0, |o| o.orb_deg).into(),
        flag(state.combustion.orbs.is_some_and(|o| o.deep_deg.is_some())),
        state
            .combustion
            .orbs
            .and_then(|o| o.deep_deg)
            .unwrap_or(0.0)
            .into(),
        u64::from(state.age.id()).into(),
        u64::from(state.wakefulness.id()).into(),
        flag(state.deeptadi.is_some()),
        u64::from(
            state
                .deeptadi
                .map_or(0, teistro_core::catalogue::AvasthaDeeptadi::id),
        )
        .into(),
        bits_of(&state.lajjitadi.holding).into(),
        bits_of(&state.lajjitadi.ruled_out).into(),
        bits_of(&state.lajjitadi.undecided).into(),
        flag(war.is_some()),
        u64::from(war.map_or(0, |w| w.opponent.id())).into(),
        flag(war.is_some_and(|w| w.is_winner)),
        war.map_or(0.0, |w| w.apart_deg).into(),
        state.boundaries.sign_deg.into(),
        state.boundaries.nakshatra_deg.into(),
        state.boundaries.pada_deg.into(),
    ]
}

/// The twelve bhavas of each chart, as the houses service reads them.
///
/// **Not ragged**, and this is the case that says why the rule is about
/// the values rather than about the section: a chart that has bhavas has
/// twelve, always, so the count is a constant and an empty section can
/// only mean "not asked for".
struct BhavaColumns {
    sign: Vec<u16>,
    lord: Vec<u16>,
    quadrant: Vec<u8>,
}

impl BhavaColumns {
    fn of(documents: &[Document]) -> BhavaColumns {
        let rows = documents.len() * 12;
        let mut columns = BhavaColumns {
            sign: Vec::with_capacity(rows),
            lord: Vec::with_capacity(rows),
            quadrant: Vec::with_capacity(rows),
        };
        for document in documents {
            let Some(houses) = document.houses.as_ref() else {
                continue;
            };
            for number in 1..=12_u8 {
                let Some(bhava) = houses.bhava(number) else {
                    continue;
                };
                columns.sign.push(bhava.sign.id());
                columns.lord.push(bhava.lord.id());
                columns
                    .quadrant
                    .push(TsQuadrant::from(bhava.quadrant) as u8);
            }
        }
        columns
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "bhavas",
            self.sign.len(),
            &[
                ColumnData::U16(&self.sign),
                ColumnData::U16(&self.lord),
                ColumnData::U8(&self.quadrant),
            ],
        )
    }
}

/// The derived points of a batch, as the section carries them.
///
/// **Ragged**, as the drishti are: a chart's points depend on what its
/// day allows — Saturn's eighth needs an arc to divide — so the count is
/// a per-chart fact. The drishti taught that lesson by refusing a batch
/// (`03-design/chart-reading.md` §5); this one takes it as read.
/// Every chart's Ashtakavarga, each section empty when it was not asked for.
struct AshtakavargaColumns {
    graha: Vec<u16>,
    shodhana: Vec<u8>,
    ekadhipatya: Vec<u8>,
    rashi_pinda: Vec<u32>,
    graha_pinda: Vec<u32>,
    yoga_pinda: Vec<u32>,
    bindus: Vec<u8>,
    reduced: Vec<u8>,
    sarva: Vec<u16>,
    trikona: Vec<u16>,
    sarva_reduced: Vec<u16>,
}

impl AshtakavargaColumns {
    fn of(documents: &[Document]) -> AshtakavargaColumns {
        let charts = documents
            .iter()
            .filter(|d| d.ashtakavarga.is_some())
            .count();
        let mut columns = AshtakavargaColumns {
            graha: Vec::with_capacity(charts * 7),
            shodhana: Vec::with_capacity(charts * 7),
            ekadhipatya: Vec::with_capacity(charts * 7),
            rashi_pinda: Vec::with_capacity(charts * 7),
            graha_pinda: Vec::with_capacity(charts * 7),
            yoga_pinda: Vec::with_capacity(charts * 7),
            bindus: Vec::with_capacity(charts * 84),
            reduced: Vec::with_capacity(charts * 84),
            sarva: Vec::with_capacity(charts * 12),
            trikona: Vec::with_capacity(charts * 12),
            sarva_reduced: Vec::with_capacity(charts * 12),
        };
        for reading in documents.iter().filter_map(|d| d.ashtakavarga.as_ref()) {
            for graha in &reading.grahas {
                columns.graha.push(graha.graha.id());
                columns
                    .shodhana
                    .push(TsShodhana::from(reading.rules.shodhana) as u8);
                columns
                    .ekadhipatya
                    .push(TsEkadhipatya::from(reading.rules.ekadhipatya) as u8);
                columns.rashi_pinda.push(graha.rashi_pinda);
                columns.graha_pinda.push(graha.graha_pinda);
                columns.yoga_pinda.push(graha.yoga_pinda);
                columns.bindus.extend(graha.bindus);
                columns.reduced.extend(graha.reduced.unwrap_or([0; 12]));
            }
            columns.sarva.extend(reading.sarva);
            columns.trikona.extend(reading.trikona);
            columns.sarva_reduced.extend(reading.reduced);
        }
        columns
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "ashtakavarga",
            self.graha.len(),
            &[
                ColumnData::U16(&self.graha),
                ColumnData::U8(&self.shodhana),
                ColumnData::U8(&self.ekadhipatya),
                ColumnData::U32(&self.rashi_pinda),
                ColumnData::U32(&self.graha_pinda),
                ColumnData::U32(&self.yoga_pinda),
            ],
        )?;
        writer.columns(
            "ashtakavarga_bindus",
            self.bindus.len(),
            &[ColumnData::U8(&self.bindus), ColumnData::U8(&self.reduced)],
        )?;
        writer.columns(
            "sarvashtakavarga",
            self.sarva.len(),
            &[
                ColumnData::U16(&self.sarva),
                ColumnData::U16(&self.trikona),
                ColumnData::U16(&self.sarva_reduced),
            ],
        )
    }
}

/// A Shadbala value column: its name in the `shadbala` section, what it
/// holds, and where a graha's reading keeps it.
pub(crate) type ShadbalaColumn = (
    &'static str,
    &'static str,
    fn(&teistro::strength::GrahaShadbala) -> f64,
);

/// The `shadbala` section's value columns in order, which the section's
/// schema and its writer both read.
pub(crate) const SHADBALA_COLUMNS: [ShadbalaColumn; 23] = [
    (
        "uchcha",
        "Sthana: from the distance to the debilitation point, 0 to 60.",
        |g| g.sthana.uchcha,
    ),
    (
        "saptavargaja",
        "Sthana: from the dignity in the seven vargas.",
        |g| g.sthana.saptavargaja,
    ),
    (
        "ojayugma",
        "Sthana: from the rasi's and navamsha's parity, 0, 15 or 30.",
        |g| g.sthana.ojayugma,
    ),
    ("kendradi", "Sthana: from the house, 60, 30 or 15.", |g| {
        g.sthana.kendradi
    }),
    ("drekkana", "Sthana: from the decanate, 0 or 15.", |g| {
        g.sthana.drekkana
    }),
    (
        "dig",
        "Dig: from the distance to the powerless kendra, 0 to 60.",
        |g| g.dig,
    ),
    ("nathonnatha", "Kaala: from the hour, 0 to 60.", |g| {
        g.kaala.nathonnatha
    }),
    (
        "paksha",
        "Kaala: from the Moon's elongation, the Moon's doubled.",
        |g| g.kaala.paksha,
    ),
    (
        "tribhaga",
        "Kaala: 60 to the lord of the third of the day or night, and to Jupiter.",
        |g| g.kaala.tribhaga,
    ),
    ("abda", "Kaala: 15 to the year's lord.", |g| g.kaala.abda),
    ("masa", "Kaala: 30 to the month's lord.", |g| g.kaala.masa),
    ("vara", "Kaala: 45 to the weekday's lord.", |g| g.kaala.vara),
    ("hora", "Kaala: 60 to the hour's lord.", |g| g.kaala.hora),
    ("ayana", "Kaala: from the declination.", |g| g.kaala.ayana),
    (
        "yuddha",
        "Kaala: gained by the victor and lost by the vanquished of a planetary war.",
        |g| g.kaala.yuddha,
    ),
    ("cheshta", "Cheshta: motional strength.", |g| g.cheshta),
    ("naisargika", "Naisargika: natural strength.", |g| {
        g.naisargika
    }),
    (
        "drik",
        "Drik: aspectual strength, which may be negative.",
        |g| g.drik,
    ),
    ("virupas", "The six together, virupas.", |g| g.virupas),
    ("rupas", "The six together, rupas.", |g| g.rupas),
    (
        "required_rupas",
        "The rupas it must reach to be strong.",
        |g| g.required_rupas,
    ),
    (
        "ishta",
        "How far it tends to good, 0 to 60 (BPHS ch. 28).",
        |g| g.ishta,
    ),
    ("kashta", "How far it tends to harm, 0 to 60.", |g| g.kashta),
];

/// Every chart's Shadbala, a row a graha, empty when it was not asked for.
struct ShadbalaColumns {
    graha: Vec<u16>,
    values: Vec<Vec<f64>>,
    strong: Vec<u8>,
}

impl ShadbalaColumns {
    fn of(documents: &[Document]) -> ShadbalaColumns {
        let readings: Vec<_> = documents
            .iter()
            .filter_map(|d| d.shadbala.as_ref())
            .collect();
        let rows = readings.iter().map(|r| r.grahas.len()).sum();
        let mut columns = ShadbalaColumns {
            graha: Vec::with_capacity(rows),
            values: SHADBALA_COLUMNS
                .iter()
                .map(|_| Vec::with_capacity(rows))
                .collect(),
            strong: Vec::with_capacity(rows),
        };
        for graha in readings.iter().flat_map(|r| &r.grahas) {
            columns.graha.push(graha.graha.id());
            for (column, (_, _, read)) in columns.values.iter_mut().zip(SHADBALA_COLUMNS) {
                column.push(read(graha));
            }
            columns.strong.push(u8::from(graha.strong));
        }
        columns
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        let mut data = Vec::with_capacity(self.values.len() + 2);
        data.push(ColumnData::U16(&self.graha));
        data.extend(self.values.iter().map(|column| ColumnData::F64(column)));
        data.push(ColumnData::U8(&self.strong));
        writer.columns("shadbala", self.graha.len(), &data)
    }
}

/// A Bhava bala value column: its name, what it holds, and where a bhava's
/// reading keeps it.
pub(crate) type BhavaBalaColumn = (
    &'static str,
    &'static str,
    fn(&teistro::strength::BhavaStrength) -> f64,
);

/// The `bhava_bala` section's value columns in order, which the section's
/// schema and its writer both read.
pub(crate) const BHAVA_BALA_COLUMNS: [BhavaBalaColumn; 5] = [
    ("adhipati", "The lord's Shadbala.", |b| b.adhipati),
    ("dig", "From its direction, 0 to 60.", |b| b.dig),
    (
        "drishti",
        "From the drishtis it receives, which may be negative.",
        |b| b.drishti,
    ),
    (
        "special",
        "From its occupants and its sign's rising, under BPHS's special rules.",
        |b| b.special,
    ),
    ("virupas", "The four together.", |b| b.virupas),
];

/// Every chart's Bhava bala, a row a bhava, empty when it was not asked for.
struct BhavaBalaColumns {
    lord: Vec<u16>,
    values: Vec<Vec<f64>>,
}

impl BhavaBalaColumns {
    fn of(documents: &[Document]) -> BhavaBalaColumns {
        let bhavas: Vec<_> = documents
            .iter()
            .filter_map(|d| d.bhava_bala.as_ref())
            .flat_map(|reading| &reading.bhavas)
            .collect();
        let mut columns = BhavaBalaColumns {
            lord: Vec::with_capacity(bhavas.len()),
            values: BHAVA_BALA_COLUMNS
                .iter()
                .map(|_| Vec::with_capacity(bhavas.len()))
                .collect(),
        };
        for bhava in bhavas {
            columns.lord.push(bhava.lord.id());
            for (column, (_, _, read)) in columns.values.iter_mut().zip(BHAVA_BALA_COLUMNS) {
                column.push(read(bhava));
            }
        }
        columns
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        let mut data = Vec::with_capacity(self.values.len() + 1);
        data.push(ColumnData::U16(&self.lord));
        data.extend(self.values.iter().map(|column| ColumnData::F64(column)));
        writer.columns("bhava_bala", self.lord.len(), &data)
    }
}

/// Every chart's Vimshopaka, a row a graha, empty when it was not asked for.
struct VimshopakaColumns {
    graha: Vec<u16>,
    scoring: Vec<u8>,
    shadvarga: Vec<f64>,
    saptavarga: Vec<f64>,
    dashavarga: Vec<f64>,
    shodashavarga: Vec<f64>,
}

impl VimshopakaColumns {
    fn of(documents: &[Document]) -> VimshopakaColumns {
        let rows = documents
            .iter()
            .filter_map(|d| d.vimshopaka.as_ref())
            .map(|reading| reading.grahas.len())
            .sum();
        let mut columns = VimshopakaColumns {
            graha: Vec::with_capacity(rows),
            scoring: Vec::with_capacity(rows),
            shadvarga: Vec::with_capacity(rows),
            saptavarga: Vec::with_capacity(rows),
            dashavarga: Vec::with_capacity(rows),
            shodashavarga: Vec::with_capacity(rows),
        };
        for reading in documents.iter().filter_map(|d| d.vimshopaka.as_ref()) {
            for graha in &reading.grahas {
                columns.graha.push(graha.graha.id());
                columns
                    .scoring
                    .push(TsVimshopakaScoring::from(reading.scoring) as u8);
                columns.shadvarga.push(graha.shadvarga);
                columns.saptavarga.push(graha.saptavarga);
                columns.dashavarga.push(graha.dashavarga);
                columns.shodashavarga.push(graha.shodashavarga);
            }
        }
        columns
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "vimshopaka",
            self.graha.len(),
            &[
                ColumnData::U16(&self.graha),
                ColumnData::U8(&self.scoring),
                ColumnData::F64(&self.shadvarga),
                ColumnData::F64(&self.saptavarga),
                ColumnData::F64(&self.dashavarga),
                ColumnData::F64(&self.shodashavarga),
            ],
        )
    }
}

struct PointColumns {
    counts: Vec<u32>,
    point: Vec<u16>,
    longitude: Vec<f64>,
    sign: Vec<u16>,
    sign_deg: Vec<f64>,
    nakshatra_deg: Vec<f64>,
    pada_deg: Vec<f64>,
}

impl PointColumns {
    fn of(documents: &[Document]) -> PointColumns {
        let rows: usize = documents
            .iter()
            .map(|d| d.points.as_ref().map_or(0, |p| p.all().len()))
            .sum();
        let mut columns = PointColumns {
            counts: Vec::with_capacity(documents.len()),
            point: Vec::with_capacity(rows),
            longitude: Vec::with_capacity(rows),
            sign: Vec::with_capacity(rows),
            sign_deg: Vec::with_capacity(rows),
            nakshatra_deg: Vec::with_capacity(rows),
            pada_deg: Vec::with_capacity(rows),
        };
        for document in documents {
            let Some(points) = document.points.as_ref() else {
                columns.counts.push(0);
                continue;
            };
            columns
                .counts
                .push(u32::try_from(points.all().len()).unwrap_or(u32::MAX));
            for found in points.all() {
                columns.point.push(found.point.id());
                columns.longitude.push(found.longitude_deg);
                columns.sign.push(found.sign.id());
                columns.sign_deg.push(found.boundaries.sign_deg);
                columns.nakshatra_deg.push(found.boundaries.nakshatra_deg);
                columns.pada_deg.push(found.boundaries.pada_deg);
            }
        }
        columns
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "points",
            self.point.len(),
            &[
                ColumnData::U16(&self.point),
                ColumnData::F64(&self.longitude),
                ColumnData::U16(&self.sign),
                ColumnData::F64(&self.sign_deg),
                ColumnData::F64(&self.nakshatra_deg),
                ColumnData::F64(&self.pada_deg),
            ],
        )
    }
}

/// The divisional charts of a batch, as the two sections carry them.
///
/// **Charts outermost, then charts asked for, then grahas** — the same
/// ordering rule every per-chart section in this blob follows, so a
/// decoder slices by arithmetic rather than by searching.
///
/// A batch whose documents hold different numbers of divisional charts
/// is `INTERNAL` for the reason a batch of differing graha counts is:
/// the layout is one count for the batch, and only this crate could have
/// built such a value.
struct VargaColumns {
    /// How many divisional charts each document holds.
    count: u32,
    ids: Vec<u16>,
    lagna_rashi: Vec<u16>,
    lagna_part: Vec<u16>,
    lagna_sign: Vec<u16>,
    rashi: Vec<u16>,
    part: Vec<u16>,
    sign: Vec<u16>,
}

impl VargaColumns {
    fn of(documents: &[Document], graha_count: usize) -> Result<VargaColumns, Error> {
        let count = documents.first().map_or(0, |d| d.vargas.len());
        if let Some(odd) = documents.iter().find(|d| d.vargas.len() != count) {
            return Err(Error::new(
                Status::Internal,
                format!(
                    "the batch mixes divisional chart counts: {count} and {}, though every document was read from one request",
                    odd.vargas.len()
                ),
            ));
        }
        let charts = documents.len() * count;
        let mut columns = VargaColumns {
            count: u32::try_from(count).unwrap_or(u32::MAX),
            ids: Vec::with_capacity(charts),
            lagna_rashi: Vec::with_capacity(charts),
            lagna_part: Vec::with_capacity(charts),
            lagna_sign: Vec::with_capacity(charts),
            rashi: Vec::with_capacity(charts * graha_count),
            part: Vec::with_capacity(charts * graha_count),
            sign: Vec::with_capacity(charts * graha_count),
        };
        for document in documents {
            for varga in &document.vargas {
                // The axis's own chart, which for every axis this
                // boundary can ask for is one catalogued member: the
                // request takes `Varga` ids, so a mixed axis or an
                // arbitrary D-N is reachable in Rust and not here
                // (`03-design/chart-reading.md` §8).
                columns.ids.push(
                    varga
                        .axis
                        .grahas
                        .varga
                        .map_or(u16::MAX, teistro_core::catalogue::Catalogued::id),
                );
                columns.lagna_rashi.push(varga.lagna.rashi.id());
                columns.lagna_part.push(varga.lagna.part);
                columns.lagna_sign.push(varga.lagna.sign.id());
                if varga.grahas.len() != graha_count {
                    return Err(Error::new(
                        Status::Internal,
                        format!(
                            "a divisional chart holds {} grahas where the foundation holds {graha_count}",
                            varga.grahas.len()
                        ),
                    ));
                }
                for placed in &varga.grahas {
                    columns.rashi.push(placed.at.rashi.id());
                    columns.part.push(placed.at.part);
                    columns.sign.push(placed.at.sign.id());
                }
            }
        }
        Ok(columns)
    }

    /// Both sections, written where the schema declares them.
    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "vargas",
            self.ids.len(),
            &[
                ColumnData::U16(&self.ids),
                ColumnData::U16(&self.lagna_rashi),
                ColumnData::U16(&self.lagna_part),
                ColumnData::U16(&self.lagna_sign),
            ],
        )?;
        writer.columns(
            "varga_grahas",
            self.rashi.len(),
            &[
                ColumnData::U16(&self.rashi),
                ColumnData::U16(&self.part),
                ColumnData::U16(&self.sign),
            ],
        )
    }
}

/// One row per chart, in the order `cast` declares its columns.
#[must_use]
fn chart_rows(
    charts: &[&ChartFoundation],
    point_counts: &[u32],
    aspect_counts: &[u32],
) -> Vec<Vec<FixedValue>> {
    charts
        .iter()
        .enumerate()
        .map(|(at, chart)| {
            vec![
                chart.instant.get().into(),
                chart.lagna_deg.into(),
                chart.day_lagna_deg.into(),
                chart.zodiac.offset_deg.into(),
                (TsDayPart::from(chart.day.part) as u64).into(),
                chart.day.elapsed.into(),
                u64::from(point_counts.get(at).copied().unwrap_or(0)).into(),
                u64::from(aspect_counts.get(at).copied().unwrap_or(0)).into(),
            ]
        })
        .collect()
}

/// Twelve bhavas per chart, charts outermost, as `houses` and `chalit`
/// both want them.
#[must_use]
fn bhava_columns(
    charts: &[&ChartFoundation],
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
/// The dashas of a batch, as the two sections carry them: one row a chart
/// a system, and every period of each, concatenated in the same order and
/// **ragged** by `period_count`, since a dasha's depth is the settings' and
/// an elapsed birth period has fewer children than a compressed one.
struct DashaColumns {
    /// How many systems each chart holds: one for the batch, since every
    /// chart answers the same request.
    count: u32,
    system: Vec<u16>,
    seeded: Vec<u8>,
    signed: Vec<u8>,
    seed: Vec<u16>,
    first_lord: Vec<u16>,
    overflow: Vec<u8>,
    balance: Vec<u8>,
    remaining: Vec<f64>,
    days: Vec<f64>,
    years: Vec<u32>,
    months: Vec<u8>,
    whole_days: Vec<u8>,
    hours: Vec<u8>,
    minutes: Vec<u8>,
    span_from: Vec<f64>,
    span_to: Vec<f64>,
    depth: Vec<u8>,
    period_count: Vec<u32>,
    level: Vec<u8>,
    index: Vec<u8>,
    sign: Vec<u16>,
    lord: Vec<u16>,
    from: Vec<f64>,
    to: Vec<f64>,
}

impl DashaColumns {
    fn of(documents: &[Document]) -> Result<DashaColumns, Error> {
        let count = documents.first().map_or(0, |d| d.dashas.len());
        if documents.iter().any(|d| d.dashas.len() != count) {
            return Err(Error::internal(
                "the batch's charts hold different numbers of dashas, though one request asked for them",
            ));
        }
        let rows = documents.len() * count;
        let periods: usize = documents
            .iter()
            .flat_map(|d| &d.dashas)
            .map(|reading| reading.periods.len())
            .sum();
        let mut columns = DashaColumns {
            count: u32::try_from(count).unwrap_or(u32::MAX),
            system: Vec::with_capacity(rows),
            seeded: Vec::with_capacity(rows),
            signed: Vec::with_capacity(rows),
            seed: Vec::with_capacity(rows),
            first_lord: Vec::with_capacity(rows),
            overflow: Vec::with_capacity(rows),
            balance: Vec::with_capacity(rows),
            remaining: Vec::with_capacity(rows),
            days: Vec::with_capacity(rows),
            years: Vec::with_capacity(rows),
            months: Vec::with_capacity(rows),
            whole_days: Vec::with_capacity(rows),
            hours: Vec::with_capacity(rows),
            minutes: Vec::with_capacity(rows),
            span_from: Vec::with_capacity(rows),
            span_to: Vec::with_capacity(rows),
            depth: Vec::with_capacity(rows),
            period_count: Vec::with_capacity(rows),
            level: Vec::with_capacity(periods),
            index: Vec::with_capacity(periods),
            sign: Vec::with_capacity(periods),
            lord: Vec::with_capacity(periods),
            from: Vec::with_capacity(periods),
            to: Vec::with_capacity(periods),
        };
        for reading in documents.iter().flat_map(|d| &d.dashas) {
            columns.push(reading);
        }
        Ok(columns)
    }

    /// One dasha's row and its periods.
    fn push(&mut self, reading: &teistro::DashaReading) {
        let columns = self;
        columns.system.push(reading.system.id());
        columns.seeded.push(u8::from(reading.seed.is_some()));
        let signed = reading
            .periods
            .first()
            .is_some_and(|period| period.sign.is_some());
        columns.signed.push(u8::from(signed));
        columns.seed.push(
            reading
                .seed
                .map_or(0, teistro_core::catalogue::Nakshatra::id),
        );
        columns.first_lord.push(reading.first_lord.id());
        columns.overflow.push(u8::from(reading.overflow));
        let balance = reading.balance;
        columns
            .balance
            .push(balance.map_or(0, |balance| TsBalance::from(balance.method) as u8));
        columns
            .remaining
            .push(balance.map_or(0.0, |balance| balance.remaining));
        columns
            .days
            .push(balance.map_or(0.0, |balance| balance.days));
        let written = balance.map(|balance| balance.written);
        columns
            .years
            .push(written.map_or(0, |written| written.years));
        columns
            .months
            .push(written.map_or(0, |written| written.months));
        columns
            .whole_days
            .push(written.map_or(0, |written| written.days));
        columns
            .hours
            .push(written.map_or(0, |written| written.hours));
        columns
            .minutes
            .push(written.map_or(0, |written| written.minutes));
        columns
            .span_from
            .push(reading.moon_span.map_or(f64::NAN, |span| span.from.get()));
        columns
            .span_to
            .push(reading.moon_span.map_or(f64::NAN, |span| span.to.get()));
        columns.depth.push(reading.depth.get());
        columns
            .period_count
            .push(u32::try_from(reading.periods.len()).unwrap_or(u32::MAX));
        for period in &reading.periods {
            let places: Vec<u8> = period
                .path
                .split('/')
                .map(|step| step.parse().unwrap_or(u8::MAX))
                .collect();
            columns
                .level
                .push(u8::try_from(places.len()).unwrap_or(u8::MAX));
            columns.index.push(places.last().copied().unwrap_or(0));
            columns
                .sign
                .push(period.sign.map_or(0, teistro_core::catalogue::Rashi::id));
            columns.lord.push(period.lord.id());
            columns.from.push(period.interval.from.get());
            columns.to.push(period.interval.to.get());
        }
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "dashas",
            self.system.len(),
            &[
                ColumnData::U16(&self.system),
                ColumnData::U8(&self.seeded),
                ColumnData::U8(&self.signed),
                ColumnData::U16(&self.seed),
                ColumnData::U16(&self.first_lord),
                ColumnData::U8(&self.overflow),
                ColumnData::U8(&self.balance),
                ColumnData::F64(&self.remaining),
                ColumnData::F64(&self.days),
                ColumnData::U32(&self.years),
                ColumnData::U8(&self.months),
                ColumnData::U8(&self.whole_days),
                ColumnData::U8(&self.hours),
                ColumnData::U8(&self.minutes),
                ColumnData::F64(&self.span_from),
                ColumnData::F64(&self.span_to),
                ColumnData::U8(&self.depth),
                ColumnData::U32(&self.period_count),
            ],
        )?;
        writer.columns(
            "dasha_periods",
            self.lord.len(),
            &[
                ColumnData::U8(&self.level),
                ColumnData::U8(&self.index),
                ColumnData::U16(&self.sign),
                ColumnData::U16(&self.lord),
                ColumnData::F64(&self.from),
                ColumnData::F64(&self.to),
            ],
        )
    }
}

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
    documents: &[Document],
    place: &Place,
    kind: ChartKind,
    provenance: &Provenance,
    svgs: &str,
) -> Result<Vec<u8>, Error> {
    let charts: Vec<&ChartFoundation> = documents.iter().map(|d| &d.foundation).collect();
    let charts = charts.as_slice();
    let schema = crate::schemas::charts();
    let mut writer = Writer::new(&schema);
    let chart_count = u32::try_from(charts.len()).unwrap_or(u32::MAX);
    let graha_count = one_size(charts)?;
    let columns = GrahaColumns::of(charts);
    let day_rows: Vec<Vec<FixedValue>> = charts.iter().map(|c| day_values(&c.day.day)).collect();
    let timing_rows: Vec<Vec<FixedValue>> =
        charts.iter().map(|c| timing_values(&c.timing)).collect();
    let once = BatchOnce::of(charts.first().copied());
    let vargas = VargaColumns::of(documents, graha_count)?;
    let aspects = AspectColumns::of(documents);
    let points = PointColumns::of(documents);
    let bhavas = BhavaColumns::of(documents);
    let states = StateColumns::of(documents);
    let dashas = DashaColumns::of(documents)?;
    let ashtakavarga = AshtakavargaColumns::of(documents);
    let vimshopaka = VimshopakaColumns::of(documents);
    let shadbala = ShadbalaColumns::of(documents);
    let bhava_bala = BhavaBalaColumns::of(documents);

    let write = || -> Result<Vec<u8>, teistro_idl::blob::BlobError> {
        writer.fixed(
            "summary",
            &summary_values(
                place,
                kind,
                chart_count,
                u32::try_from(graha_count).unwrap_or(u32::MAX),
                vargas.count,
                dashas.count,
            ),
        )?;
        writer.rows("cast", &chart_rows(charts, &points.counts, &aspects.counts))?;
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
        write_bhavas(&mut writer, "houses", charts, |c| &c.houses)?;
        write_bhavas(&mut writer, "chalit", charts, |c| &c.chalit)?;
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
        vargas.write(&mut writer)?;
        aspects.write(&mut writer)?;
        points.write(&mut writer)?;
        bhavas.write(&mut writer)?;
        states.write(&mut writer)?;
        writer.bytes("drawings", drawings_json(documents).as_bytes())?;
        writer.bytes("svgs", svgs.as_bytes())?;
        dashas.write(&mut writer)?;
        ashtakavarga.write(&mut writer)?;
        vimshopaka.write(&mut writer)?;
        shadbala.write(&mut writer)?;
        bhava_bala.write(&mut writer)?;
        writer.finish()
    };
    write().map_err(|error| {
        Error::new(
            Status::Internal,
            format!("the chart blob could not be written: {error}"),
        )
    })
}

/// One of the two twelve-bhava sections, charts outermost.
fn write_bhavas(
    writer: &mut Writer<'_>,
    name: &str,
    charts: &[&ChartFoundation],
    pick: fn(&ChartFoundation) -> &teistro_chart::bhava::Bhavas,
) -> Result<(), teistro_idl::blob::BlobError> {
    let (madhya, sandhi) = bhava_columns(charts, pick);
    writer.columns(
        name,
        charts.len() * 12,
        &[ColumnData::F64(&madhya), ColumnData::F64(&sandhi)],
    )
}

/// Every chart's drawings as the canonical JSON the `drawings` section
/// carries: one array per chart, or nothing at all when none were asked for,
/// so a caller that drew nothing pays for no text.
fn drawings_json(documents: &[Document]) -> String {
    if documents
        .iter()
        .all(|document| document.drawings.is_empty())
    {
        return String::new();
    }
    let per_chart: Vec<&Vec<teistro_geometry::Drawing>> = documents
        .iter()
        .map(|document| &document.drawings)
        .collect();
    teistro_core::envelope::canonical_json(&per_chart)
}

/// Every chart's drawings written as SVG in one theme, as the canonical JSON
/// the `svgs` section carries: one array of strings per chart, in the order
/// the drawings were asked for.
fn svgs_json(
    sdk: &teistro::Context,
    documents: &[Document],
    theme: &Theme,
) -> Result<String, Error> {
    let mut per_chart = Vec::with_capacity(documents.len());
    for document in documents {
        let mut svgs = Vec::with_capacity(document.drawings.len());
        for index in 0..document.drawings.len() {
            svgs.push(sdk.chart().svg(document, index, theme)?);
        }
        per_chart.push(svgs);
    }
    Ok(teistro_core::envelope::canonical_json(&per_chart))
}

/// A chart layout this context can draw in, shipped or registered, as its
/// JSON row: the record `options.layouts_json` takes. Read a shipped row,
/// give it a key of its own, change what differs and register it
/// (`03-design/chart-geometry.md` §7f). `key` is the layout's key, bare
/// (`NORTH_INDIAN`) or full (`chart_layout.NORTH_INDIAN`); an unknown one is
/// `INVALID_ARG` with the keys the context knows as the hint.
///
/// # Safety
///
/// `context` must be a live handle; `key` a NUL-terminated string;
/// `out_json` valid for a write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_chart_layout_row(
    context: *const TsContext,
    key: *const c_char,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |ctx| {
        // SAFETY: the entry point's contract.
        let asked = unsafe { crate::support::text(key, "key") }?;
        let row = ctx.sdk().chart().layout(asked)?;
        let json = TsString::from_string(teistro_core::envelope::canonical_json(&row));
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_json, "out_json", json) }
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
        // SAFETY: the caller promises a readable request; `read_in`
        // checks the handshake before anything else reads a field.
        let asked = *unsafe { read_in(request, "request") }?;
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
        // SAFETY: the entry point's contract — the caller promises
        // `instant_count` readable doubles at `instants`, or null and zero.
        let instants: Vec<JulianDay<Utc>> =
            unsafe { slice(asked.instants, asked.instant_count, "instants") }?
                .iter()
                .map(|jd| JulianDay::<Utc>::literal(*jd))
                .collect();
        // SAFETY: as above, for `varga_count` readable `u16`s.
        let asked_vargas = unsafe { slice(asked.vargas, asked.varga_count, "vargas") }?;
        let mut vargas = Vec::with_capacity(asked_vargas.len());
        for id in asked_vargas {
            vargas.push(Varga::from_id(*id).ok_or_else(|| {
                Error::new(
                    Status::InvalidArg,
                    format!("no divisional chart with id {id}"),
                )
                .with_field("vargas")
            })?);
        }
        // SAFETY: as above, for `dasha_count` readable `u16`s.
        let asked_dashas = unsafe { slice(asked.dashas, asked.dasha_count, "dashas") }?;
        let mut dashas = Vec::with_capacity(asked_dashas.len());
        for (index, id) in asked_dashas.iter().enumerate() {
            dashas.push(DashaSystem::from_id(*id).ok_or_else(|| {
                Error::invalid_arg(format!("no dasha system with id {id}"))
                    .with_field(format!("dashas[{index}]"))
            })?);
        }
        // SAFETY: as above, for `drawing_count` readable `u32`s.
        let asked_drawings = unsafe { slice(asked.drawings, asked.drawing_count, "drawings") }?;
        let mut drawings = Vec::with_capacity(asked_drawings.len());
        for (index, packed) in asked_drawings.iter().enumerate() {
            let (layout, varga) = (packed >> 16, packed & 0xFFFF);
            let varga = u16::try_from(varga)
                .ok()
                .and_then(Varga::from_id)
                .ok_or_else(|| {
                    Error::invalid_arg(format!(
                        "drawing {index} names divisional chart id {varga}, which is none"
                    ))
                    .with_field(format!("drawings[{index}]"))
                })?;
            let layout = KeyId::new(Kind::ChartLayout, u16::try_from(layout).unwrap_or(u16::MAX));
            drawings.push((layout, varga));
        }
        let request = sections_of(
            asked.sections,
            ChartRequest::at(place, clock).with_kind(kind),
        )
        .with_vargas(vargas)
        .with_drawings(drawings)
        .with_dashas(dashas);
        // **The façade reads it**, which is what the dependency inversion
        // was for: `rust-consumer-surface.md` moved the SDK's
        // composition into `teistro` and had this crate depend on it, and
        // `TsContext::build` became a call into the builder — but this
        // entry point went on resolving the calendar, substituting the
        // ayanamsha and building the solar model itself. That was the
        // second copy of the chart composition, kept equal to the first
        // by hand and by nothing else.
        //
        // It also seals, so there is nothing left for the boundary to do
        // but encode what it was given.
        // SAFETY: the entry point's contract — null, or a NUL-terminated
        // string.
        let theme = unsafe { optional_text(asked.theme_json, "theme_json") }?
            .map(Theme::from_json)
            .transpose()
            .map_err(|error| {
                // The theme names its fields from its own root; the request
                // calls that root `theme_json`.
                let field = error.field().map_or_else(
                    || String::from("theme_json"),
                    |inner| format!("theme_json{}", inner.strip_prefix("theme").unwrap_or(inner)),
                );
                error.with_field(field)
            })?;
        let founded = ctx.sdk().chart().readings(&instants, &request)?;
        let svgs = match &theme {
            Some(theme) => svgs_json(ctx.sdk(), &founded.value, theme)?,
            None => String::new(),
        };
        let encoded = encode(&founded.value, &place, kind, &founded.provenance, &svgs)?;
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
        Altitude, ChartKind, JulianDay, Latitude, Longitude, Place, Provenance, TsDayPart,
        TsDayState, TsGhatiReckoning, TsHoraReckoning, TsPolarDayPolicy, TsPolarKind, TsReading,
        TsSunrise, Utc, UtcOffset,
    };
    // The founder and the solar model, which this module builds by hand:
    // the entry point above founds through the façade now, and a test of
    // the *encoder* wants a founding it can control rather than a
    // context.
    use teistro_astro::precession::PrecessionModel;
    use teistro_calendar::shipped;
    use teistro_calendar::solar::drik::DrikSun;
    use teistro_chart::bhava::Reading;
    use teistro_chart::day::DayPart;
    use teistro_chart::foundation::Founder;
    use teistro_core::catalogue::{Ayanamsha, Varga};
    use teistro_core::settings::{GhatiReckoning, HoraReckoning, PolarDayPolicy, Sunrise};
    use teistro_serial::Document;
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
        (founded.value, place, founded.provenance)
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
        let resolved =
            teistro_core::settings::Profile::shipped(teistro_core::settings::DEFAULT_PROFILE)
                .expect("the default profile")
                .resolve(&teistro_core::settings::SettingsPatch::default())
                .expect("it resolves");
        // The **navamsha asked for**, because a section that is only
        // ever empty is a section nothing tests: the blob's layout for a
        // divisional chart is charts outermost then charts asked for,
        // and one varga over two charts is the smallest grid that can
        // come out transposed.
        let documents: Vec<Document> = charts
            .iter()
            .map(|chart| {
                Document::of(chart.clone())
                    .with_varga(
                        teistro_vargas::chart::chart(
                            chart,
                            teistro_vargas::chart::Axis::of(Varga::D9),
                        )
                        .expect("a navamsha"),
                    )
                    .with_aspects(
                        teistro_aspect::Aspects::of(chart, &resolved.settings)
                            .expect("the drishti"),
                    )
            })
            .collect();
        let bytes = super::encode(&documents, &place, ChartKind::Natal, &provenance, "")
            .expect("it encodes");
        let schema = crate::schemas::charts();
        let reader = Reader::parse(&bytes, &schema).expect("a well-formed blob");
        let graha_count = charts[0].grahas.len();

        // **By name, not by position.** The summary has grown a count
        // twice in one session, and a positional read of it reported a
        // latitude of 1.0 both times — a plausible latitude, which is
        // the worst kind of wrong.
        let field = |name: &str| reader.field("summary", name).expect(name);
        assert_eq!(field("kind").as_i64(), i64::from(ChartKind::Natal.id()));
        assert_eq!(field("chart_count").as_i64(), charts.len() as i64);
        assert_eq!(field("graha_count").as_i64(), graha_count as i64);
        assert_eq!(field("varga_count").as_i64(), 1, "the one asked for");
        assert_eq!(field("latitude_deg").as_f64(), place.latitude.get());

        // The navamsha, read back: one row per chart and the graha rows
        // charts-outermost then vargas-outermost, which is the ordering
        // a decoder slices by.
        let asked = reader.column("vargas", "varga").expect("the vargas");
        assert_eq!(asked.len(), charts.len());
        assert!(
            asked
                .iter()
                .all(|id| id.as_i64() == i64::from(Varga::D9.id()))
        );
        let signs = reader.column("varga_grahas", "sign").expect("the signs");
        assert_eq!(signs.len(), charts.len() * graha_count);
        for (index, document) in documents.iter().enumerate() {
            let navamsha = &document.vargas[0];
            for (at, placed) in navamsha.grahas.iter().enumerate() {
                assert_eq!(
                    signs[index * graha_count + at].as_i64(),
                    i64::from(placed.at.sign.id()),
                    "chart {index}, graha {at}"
                );
            }
        }

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

    /// **The drishti, and the ragged layout that carries them.**
    ///
    /// A section of its own because what it proves is its own: two
    /// charts of the same nine grahas hold *different* numbers of
    /// relations — 47 and 40 — which the check that would have enforced
    /// one count for the batch is what found out. A chart's drishti are
    /// a function of where the bodies stand rather than of how many
    /// there are, so the rows are concatenated and a reader prefix-sums
    /// `cast.aspect_count`, which is the panchanga blob's own rule for a
    /// ragged list.
    #[test]
    fn the_drishti_are_ragged_and_the_counts_say_where_each_chart_begins() {
        use teistro_idl::blob::Reader;

        let (charts, place, provenance) = founded();
        let resolved =
            teistro_core::settings::Profile::shipped(teistro_core::settings::DEFAULT_PROFILE)
                .expect("the default profile")
                .resolve(&teistro_core::settings::SettingsPatch::default())
                .expect("it resolves");
        let documents: Vec<Document> = charts
            .iter()
            .map(|chart| {
                Document::of(chart.clone()).with_aspects(
                    teistro_aspect::Aspects::of(chart, &resolved.settings).expect("the drishti"),
                )
            })
            .collect();
        let bytes = super::encode(&documents, &place, ChartKind::Natal, &provenance, "")
            .expect("it encodes");
        let schema = crate::schemas::charts();
        let reader = Reader::parse(&bytes, &schema).expect("a well-formed blob");

        // **The drishti, and the ragged layout that carries them.** Two
        // charts of the same nine grahas hold different numbers of
        // relations — the check that would have enforced one count for
        // the batch is what found that out — so the rows are
        // concatenated and a reader prefix-sums `cast.aspect_count`.
        let counts = reader.column("cast", "aspect_count").expect("the counts");
        let from = reader.column("aspects", "from").expect("the drishti");
        assert_eq!(counts.len(), charts.len());
        assert!(counts.iter().all(|n| n.as_i64() > 0), "a chart has drishti");
        assert_ne!(
            counts[0].as_i64(),
            counts[1].as_i64(),
            "these two charts differ, which is why the section is ragged"
        );
        let mut at = 0_usize;
        for (index, document) in documents.iter().enumerate() {
            let relations = document.aspects.as_ref().expect("asked for").all();
            assert_eq!(counts[index].as_i64(), relations.len() as i64);
            for (k, drishti) in relations.iter().enumerate() {
                assert_eq!(
                    from[at + k].as_i64(),
                    i64::from(drishti.from.id()),
                    "chart {index}, drishti {k}"
                );
            }
            at += relations.len();
        }
        assert_eq!(from.len(), at, "the rows are exactly the counts");
        assert!(
            !reader.bytes("drishti_table").expect("the table").is_empty(),
            "the table every relation was read under"
        );
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
        let bytes =
            super::encode(&[], &place, ChartKind::Natal, &provenance, "").expect("it encodes");
        let schema = crate::schemas::charts();
        let reader = Reader::parse(&bytes, &schema).expect("a well-formed blob");

        let field = |name: &str| reader.field("summary", name).expect(name);
        assert_eq!(field("kind").as_i64(), i64::from(ChartKind::Natal.id()));
        assert_eq!(field("chart_count").as_i64(), 0, "no charts");
        assert_eq!(field("graha_count").as_i64(), 0, "and so no grahas each");
        assert_eq!(field("varga_count").as_i64(), 0, "and no divisional charts");
        assert_eq!(
            field("latitude_deg").as_f64(),
            place.latitude.get(),
            "but a place"
        );
        assert!(
            reader.column("vargas", "varga").expect("empty").is_empty(),
            "a section nobody asked for is written and empty"
        );
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
