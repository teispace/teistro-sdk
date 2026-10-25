//! The day a chart and an almanac day are both reckoned in, at the C
//! boundary: how its sunrise was taken, whether it had one, and the values
//! the blob's day section carries (`03-design/chart-at-the-boundary.md`
//! §8). Shared by the chart and the panchanga, so it is compiled into every
//! build.

use teistro_chart::day::DayPart;
use teistro_core::settings::{GhatiReckoning, HoraReckoning, PolarDayPolicy, Sunrise};
use teistro_idl::blob::FixedValue;
use teistro_time::local_day::{DayState, PolarKind};

/// Which arc of its day an instant falls in.
///
/// ```
/// use teistro_ffi::day::TsDayPart;
///
/// // A member crosses the boundary as its number: `Daylight` is 0, `Night` 1.
/// assert_eq!((TsDayPart::Daylight as u8, TsDayPart::Night as u8), (0, 1));
/// ```
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
///
/// ```
/// use teistro_ffi::day::TsSunrise;
///
/// // A member crosses the boundary as its number: `CentreNoRefraction` is 0, `CentreRefraction` 3.
/// assert_eq!((TsSunrise::CentreNoRefraction as u8, TsSunrise::CentreRefraction as u8), (0, 3));
/// ```
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsSunrise {
    /// The centre of the disc on the geometric horizon.
    CentreNoRefraction = 0,
    /// The upper limb with refraction.
    UpperLimbRefraction = 1,
    /// The lower limb with refraction.
    LowerLimbRefraction = 2,
    /// The centre of the disc with standard refraction.
    CentreRefraction = 3,
}

impl TsSunrise {
    /// The id this build gives a member, or `None` for one it does
    /// not know — a member added to the knob since this was written,
    /// which is refused by name rather than defaulted.
    ///
    /// ```
    /// use teistro_core::settings::Sunrise;
    /// use teistro_ffi::day::TsSunrise;
    ///
    /// assert_eq!(TsSunrise::of(Sunrise::LowerLimbRefraction), Some(TsSunrise::LowerLimbRefraction));
    /// ```
    #[must_use]
    pub fn of(sunrise: Sunrise) -> Option<TsSunrise> {
        match sunrise {
            Sunrise::CentreNoRefraction => Some(TsSunrise::CentreNoRefraction),
            Sunrise::UpperLimbRefraction => Some(TsSunrise::UpperLimbRefraction),
            Sunrise::LowerLimbRefraction => Some(TsSunrise::LowerLimbRefraction),
            Sunrise::CentreRefraction => Some(TsSunrise::CentreRefraction),
            _ => None,
        }
    }
}

/// How the sixty ghatis of a day are measured.
///
/// ```
/// use teistro_ffi::day::TsGhatiReckoning;
///
/// // A member crosses the boundary as its number: `Civil` is 0, `Proportional` 1.
/// assert_eq!((TsGhatiReckoning::Civil as u8, TsGhatiReckoning::Proportional as u8), (0, 1));
/// ```
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
    ///
    /// ```
    /// use teistro_core::settings::GhatiReckoning;
    /// use teistro_ffi::day::TsGhatiReckoning;
    ///
    /// assert_eq!(TsGhatiReckoning::of(GhatiReckoning::Proportional), Some(TsGhatiReckoning::Proportional));
    /// ```
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
///
/// ```
/// use teistro_ffi::day::TsHoraReckoning;
///
/// // A member crosses the boundary as its number: `Proportional` is 0, `Equal` 1.
/// assert_eq!((TsHoraReckoning::Proportional as u8, TsHoraReckoning::Equal as u8), (0, 1));
/// ```
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
    ///
    /// ```
    /// use teistro_core::settings::HoraReckoning;
    /// use teistro_ffi::day::TsHoraReckoning;
    ///
    /// assert_eq!(TsHoraReckoning::of(HoraReckoning::Equal), Some(TsHoraReckoning::Equal));
    /// ```
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
///
/// ```
/// use teistro_ffi::day::TsDayState;
///
/// // A member crosses the boundary as its number: `Normal` is 0, `Polar` 1.
/// assert_eq!((TsDayState::Normal as u8, TsDayState::Polar as u8), (0, 1));
/// ```
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsDayState {
    /// Sunrise and sunset occurred; the two fields beside this are zero.
    Normal = 0,
    /// No horizon crossing, and the policy synthesised the bounds.
    Polar = 1,
}

/// Which polar state a day without a sunrise was in.
///
/// ```
/// use teistro_ffi::day::TsPolarKind;
///
/// // A member crosses the boundary as its number: `Day` is 0, `Night` 1.
/// assert_eq!((TsPolarKind::Day as u8, TsPolarKind::Night as u8), (0, 1));
/// ```
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
///
/// ```
/// use teistro_ffi::day::TsPolarDayPolicy;
///
/// // A member crosses the boundary as its number: `Undefined` is 0, `CivilMidnight` 2.
/// assert_eq!((TsPolarDayPolicy::Undefined as u8, TsPolarDayPolicy::CivilMidnight as u8), (0, 2));
/// ```
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
    ///
    /// ```
    /// use teistro_core::settings::PolarDayPolicy;
    /// use teistro_ffi::day::TsPolarDayPolicy;
    ///
    /// assert_eq!(TsPolarDayPolicy::of(PolarDayPolicy::NearestEvent), Some(TsPolarDayPolicy::NearestEvent));
    /// ```
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
    ///
    /// ```
    /// use teistro_core::settings::PolarDayPolicy;
    /// use teistro_ffi::day::{TsDayState, TsPolarDayPolicy, TsPolarKind};
    /// use teistro_time::local_day::{DayState, PolarKind};
    ///
    /// assert_eq!(TsDayState::split(DayState::Normal), (TsDayState::Normal, 0, 0));
    /// let polar = DayState::Polar { kind: PolarKind::Night, policy: PolarDayPolicy::CivilMidnight };
    /// assert_eq!(TsDayState::split(polar), (TsDayState::Polar, TsPolarKind::Night as u8, TsPolarDayPolicy::CivilMidnight as u8));
    /// ```
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

/// The day's seventeen-and-five values, in the order `day_section`
/// declares them.
///
/// Declared once in `schemas::day_section` and filled once here, so the
/// panchanga blob writes the same day the same way rather than a second
/// copy of the same arithmetic
/// (`03-design/chart-at-the-boundary.md` §8).
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_core::catalogue::{Calendar, Vara};
/// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
/// # use teistro_core::settings::{Sunrise, SunriseConvention};
/// # use teistro_time::local_day::{DayState, LocalDay};
/// #
/// # let day = LocalDay {
/// #     place: Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0)),
/// #     date: CalendarDate::defined(Calendar::Gregorian, 2024, 1, 1),
/// #     vara: Vara::Somavara,
/// #     sunrise: JulianDay::literal(2_460_310.553),
/// #     sunset: JulianDay::literal(2_460_310.973),
/// #     next_sunrise: JulianDay::literal(2_460_311.553),
/// #     state: DayState::Normal,
/// #     convention: SunriseConvention::Named { which: Sunrise::UpperLimbRefraction },
/// #     model: String::from("an example"),
/// # };
/// use teistro_idl::blob::FixedValue;
/// use teistro_ffi::day::{TsSunrise, day_values};
///
/// // Monday 1 January 2024 at Kathmandu: sunrise first, the convention named.
/// let values = day_values(&day);
/// assert_eq!(values.len(), 20);
/// assert_eq!(values[0], FixedValue::Float(2_460_310.553));
/// assert_eq!(values[16], FixedValue::Uint(TsSunrise::UpperLimbRefraction as u64));
/// ```
#[must_use]
pub fn day_values(local: &teistro_time::local_day::LocalDay) -> Vec<FixedValue> {
    let (state, polar_kind, polar_policy) = TsDayState::split(local.state);
    let named = |which| TsSunrise::of(which).map_or(u64::from(u8::MAX), |s| s as u64);
    let (convention, convention_value) = match local.convention {
        // The convention an air was given to is named as the convention;
        // the air crosses beside it, as it was applied (`LocalDay::air`).
        teistro_core::settings::SunriseConvention::Named { which }
        | teistro_core::settings::SunriseConvention::Atmospheric { which, .. } => {
            (named(which), 0.0)
        }
        // No id names a custom convention, so the sentinel says "read the
        // altitude beside this" rather than naming a convention it is not.
        teistro_core::settings::SunriseConvention::Custom { altitude_deg } => {
            (u64::from(u8::MAX), altitude_deg)
        }
    };
    let air = local.air();
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
        air.map_or(0.0, |air| air.pressure_hpa).into(),
        air.map_or(0.0, |air| air.temperature_c).into(),
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
