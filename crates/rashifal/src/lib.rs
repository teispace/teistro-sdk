//! A rashifal: one period of the sky read for each of the twelve signs,
//! each taken as a reader's janma rāśi (`03-design/rashifal.md`).
//!
//! Phaladeepika ch. 26 is the doctrine, already built in
//! [`teistro_gochar`]: the houses are counted from the Moon's sign (v. 1),
//! which a rashifal's sign is, so each of the twelve is
//! [`gochar`]`(`[`Reference::moon`]`(sign), …)` over one snapshot. Saturn's
//! standing is read through [`sade_sati`]'s phases and smaller spells, and
//! each event of the period is counted from every sign. The text gives no
//! score; the baseline engine's is [`baseline`], reached only when asked
//! (C361).
//!
//! The crate reads no clock and no ephemeris: the façade founds the
//! snapshot and finds the events.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::house::House;
use teistro_gochar::hits::{Hit, HitEvent};
use teistro_gochar::sade_sati::{self, Phase};
use teistro_gochar::{GocharReading, GocharRules, Reference, Transit, gochar, good_house};

pub mod baseline;
pub use baseline::{BaselineScore, Period};

#[cfg(test)]
mod tests;

/// The readings a rashifal is given under.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RashifalRules {
    /// The gochar's own readings (C136, C137, C140), from the settings'
    /// `gochar` group.
    pub gochar: GocharRules,
    /// The houses from the sign counted as Saturn's smaller spells (C149).
    pub spells: Vec<u8>,
}

impl Default for RashifalRules {
    /// The text read whole ([`GocharRules::TEXT`]), with the 4th and the
    /// 8th as the spells.
    fn default() -> Self {
        RashifalRules {
            gochar: GocharRules::TEXT,
            spells: sade_sati::DEFAULT_SPELLS.to_vec(),
        }
    }
}

/// One event of the period: a hit, and the sign it happens in — the sign
/// entered, or the one a graha stations in.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PeriodEvent {
    /// When, whose and what.
    pub hit: Hit,
    /// The sign it happens in.
    pub sign: Rashi,
}

/// An event counted from one sign.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct EventFrom {
    /// The event.
    pub event: PeriodEvent,
    /// The house its sign is from this one, 1 to 12.
    pub house: u8,
    /// Whether v. 2 makes the graha's transit of that house good. An event
    /// is a fact of the period, so no vedha is read against it.
    pub good_house: bool,
}

/// Saturn's standing from one sign.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SaturnStanding {
    /// Its house from the sign, 1 to 12.
    pub house: u8,
    /// The Sade Sati phase that house is, when it is the 12th, 1st or 2nd.
    pub sade_sati: Option<Phase>,
    /// Whether the house is one of the smaller spells the rules count.
    pub spell: bool,
}

/// One sign's reading of the period.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct RashiReading {
    /// The sign, taken as the reader's janma rāśi.
    pub rashi: Rashi,
    /// Every graha's transit counted from it at the snapshot.
    pub gochar: GocharReading,
    /// Saturn's standing from it at the snapshot.
    pub saturn: SaturnStanding,
    /// Each event of the period, counted from it, in the events' order.
    pub events: Vec<EventFrom>,
}

/// The twelve signs' readings of one period: the nine `transits` at its
/// snapshot, the Sun to Ketu, and its `events`.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_gochar::Transit;
/// use teistro_rashifal::{RashifalRules, rashifal};
///
/// // Saturn in Pisces: Sade Sati's peak for Pisces, its rising for Aries.
/// let mut transits = [Transit::new(Rashi::Aries, 5.0); 9];
/// transits[Graha::Saturn as usize] = Transit::new(Rashi::Pisces, 12.0);
/// transits[Graha::Rahu as usize] = Transit::new(Rashi::Virgo, 3.0);
/// transits[Graha::Ketu as usize] = Transit::new(Rashi::Pisces, 3.0);
/// let read = rashifal(&transits, &[], &RashifalRules::default());
/// assert_eq!(read[11].saturn.house, 1);
/// assert_eq!(read[0].saturn.house, 12);
/// ```
#[must_use]
pub fn rashifal(
    transits: &[Transit; 9],
    events: &[PeriodEvent],
    rules: &RashifalRules,
) -> [RashiReading; 12] {
    Rashi::ALL.map(|rashi| {
        let saturn = House::between(
            rashi,
            transits
                .get(Graha::Saturn as usize)
                .map_or(rashi, |at| at.sign),
        )
        .get();
        RashiReading {
            rashi,
            gochar: gochar(Reference::moon(rashi), transits, rules.gochar),
            saturn: SaturnStanding {
                house: saturn,
                sade_sati: Phase::of(saturn),
                spell: rules.spells.contains(&saturn),
            },
            events: events
                .iter()
                .map(|&event| {
                    let house = House::between(rashi, event.sign).get();
                    EventFrom {
                        event,
                        house,
                        good_house: good_house(event.hit.graha, house),
                    }
                })
                .collect(),
        }
    })
}

/// The day a period of `days` civil days is read at, counted from its
/// first day: the middle one, ⌊(days − 1)/2⌋ (C359). A week's fourth day,
/// a 31-day month's sixteenth, a common year's 183rd.
///
/// ```
/// assert_eq!(teistro_rashifal::reference_day(1), 0);
/// assert_eq!(teistro_rashifal::reference_day(7), 3);
/// assert_eq!(teistro_rashifal::reference_day(31), 15);
/// assert_eq!(teistro_rashifal::reference_day(366), 182);
/// ```
#[must_use]
pub const fn reference_day(days: u32) -> u32 {
    days.saturating_sub(1) / 2
}

/// Whether an event is a sign ingress or a station, the two a period
/// reports.
#[must_use]
pub const fn is_period_event(event: HitEvent) -> bool {
    matches!(
        event,
        HitEvent::SignIngress { .. } | HitEvent::Station { .. }
    )
}
