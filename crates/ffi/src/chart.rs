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

use teistro_chart::bhava::Reading;
use teistro_chart::day::DayPart;
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

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        reason = "a test fails by panicking, and the message names the member with no id"
    )]

    use super::{
        TsDayPart, TsDayState, TsGhatiReckoning, TsHoraReckoning, TsPolarDayPolicy, TsPolarKind,
        TsReading, TsSunrise,
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
