//! The transit hit list's events (`03-design/transit-hit-list.md`): when a
//! graha enters a sign or a nakshatra, and when it stands still.
//!
//! The searches are the SDK's; this module is the vocabulary the answer is
//! said in, and the one place a crossing becomes the sign it entered — a
//! crossing passed backwards enters the sign **before** its line, which is
//! the retrograde re-entry a hit list exists to catch.
//!
//! ```
//! use teistro_core::catalogue::Rashi;
//! use teistro_gochar::hits::{Motion, entered};
//!
//! // Forward over 120° is Leo; backward over it is Cancer again.
//! assert_eq!(entered(120.0, Motion::Direct, 12), 4);
//! assert_eq!(entered(120.0, Motion::Retrograde, 12), 3);
//! assert_eq!(Rashi::ALL[4], Rashi::Leo);
//! ```

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Nakshatra, Rashi};
use teistro_core::quantity::{JulianDay, Utc};

/// Which way a graha was moving, through a line or out of a station.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Motion {
    /// Forward through the zodiac.
    Direct,
    /// Backward: a retrograde graha, or the nodes' mean motion.
    Retrograde,
}

/// A natal point a transit can aspect, ordered as a tie between hits is
/// broken: the grahas in the catalogue's order, then the lagna.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "point", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NatalPoint {
    /// A natal graha.
    Graha {
        /// Which.
        graha: Graha,
    },
    /// The natal lagna.
    Lagna,
}

impl NatalPoint {
    /// Its place in a tie's order: the grahas by id, then the lagna.
    const fn rank(self) -> u8 {
        match self {
            NatalPoint::Graha { graha } => graha as u8,
            NatalPoint::Lagna => u8::MAX,
        }
    }
}

/// Where in an aspect's window a hit falls (C146).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AspectPhase {
    /// The transit came within the orb.
    Entering,
    /// The aspect is exact.
    Exact,
    /// The transit passed out of the orb.
    Leaving,
}

/// Which line of an aspect a lattice holds: the exact one, or the orb's
/// edge before or past it in the zodiac's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    /// The exact line.
    Exact,
    /// The orb before the line.
    Before,
    /// The orb past the line.
    Past,
}

/// What a hit was.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HitEvent {
    /// The graha entered a sign.
    SignIngress {
        /// The sign entered.
        into: Rashi,
        /// Which way it was moving.
        motion: Motion,
    },
    /// The graha entered a nakshatra.
    NakshatraIngress {
        /// The nakshatra entered.
        into: Nakshatra,
        /// Which way it was moving.
        motion: Motion,
    },
    /// The graha stood still in longitude.
    Station {
        /// The motion it turned to.
        turns: Motion,
    },
    /// The graha aspected a natal point, or came within or left its orb.
    Aspect {
        /// The natal point aspected.
        to: NatalPoint,
        /// The aspect's angle, 0 to 180 degrees; the angle past 180 is the
        /// same aspect from the other side.
        angle: u16,
        /// Where in its window.
        phase: AspectPhase,
        /// Which way the transit was moving.
        motion: Motion,
    },
}

impl HitEvent {
    /// The kind's place in a tie's order: ingresses before stations, signs
    /// before nakshatras.
    const fn rank(self) -> u8 {
        match self {
            HitEvent::SignIngress { .. } => 0,
            HitEvent::NakshatraIngress { .. } => 1,
            HitEvent::Station { .. } => 2,
            HitEvent::Aspect { .. } => 3,
        }
    }

    /// The natal point's place in a tie's order, for an aspect.
    const fn point_rank(self) -> u8 {
        match self {
            HitEvent::Aspect { to, .. } => to.rank(),
            _ => 0,
        }
    }
}

/// One event of a transit window.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Hit {
    /// When, a UTC Julian day.
    pub instant: JulianDay<Utc>,
    /// Whose.
    pub graha: Graha,
    /// What.
    pub event: HitEvent,
}

/// Hits in their one order: by instant, then by graha, then by kind, so
/// two runs over the same window list them identically.
pub fn sort(hits: &mut [Hit]) {
    hits.sort_by(|a, b| {
        a.instant
            .get()
            .total_cmp(&b.instant.get())
            .then((a.graha as u8).cmp(&(b.graha as u8)))
            .then(a.event.rank().cmp(&b.event.rank()))
            .then(a.event.point_rank().cmp(&b.event.point_rank()))
    });
}

/// The division a crossing of the line at `boundary_deg` entered, of
/// `divisions` equal parts of the circle: the one past the line when the
/// graha was moving forward, the one before it when it was moving back.
#[must_use]
pub fn entered(boundary_deg: f64, motion: Motion, divisions: u16) -> usize {
    let width = 360.0 / f64::from(divisions);
    // The line itself, rounded to the nearest whole division: a boundary
    // the search reports a hair either side of its line is still that line.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a longitude over a division's width is a small whole count"
    )]
    let line = (boundary_deg.rem_euclid(360.0) / width).round() as usize % usize::from(divisions);
    match motion {
        Motion::Direct => line,
        Motion::Retrograde => (line + usize::from(divisions) - 1) % usize::from(divisions),
    }
}

/// A sign ingress at a crossing of the sign lattice.
#[must_use]
pub fn sign_ingress(boundary_deg: f64, motion: Motion) -> HitEvent {
    HitEvent::SignIngress {
        into: Rashi::ALL
            .get(entered(boundary_deg, motion, 12))
            .copied()
            .unwrap_or(Rashi::Aries),
        motion,
    }
}

/// A nakshatra ingress at a crossing of the nakshatra lattice.
#[must_use]
pub fn nakshatra_ingress(boundary_deg: f64, motion: Motion) -> HitEvent {
    HitEvent::NakshatraIngress {
        into: Nakshatra::ALL
            .get(entered(boundary_deg, motion, 27))
            .copied()
            .unwrap_or(Nakshatra::Ashwini),
        motion,
    }
}

/// The step of the lattice about a natal point that holds every line of the
/// `angles` asked for, from either side, and as few others as it can: the
/// greatest common divisor of 360 and each line, degrees.
///
/// A lattice at 30° holds every aspect at once, and a search over it
/// refines every crossing of it; asked for the conjunction and the
/// opposition alone, ten of each twelve refinements were thrown away by
/// [`aspect_hit`]. At this step the lines the list keeps are all but every
/// line (the one exception being a set like 90° alone, whose lattice at 90°
/// also holds 0° and 180°, which the filter still drops).
///
/// ```
/// use teistro_gochar::hits::aspect_step_deg;
///
/// assert_eq!(aspect_step_deg(&[0, 180]), 180.0);
/// assert_eq!(aspect_step_deg(&[0]), 360.0);
/// assert_eq!(aspect_step_deg(&[0, 60, 90, 120, 180]), 30.0);
/// // Leo's table, with the semi-square and the sesquiquadrate.
/// assert_eq!(aspect_step_deg(&[0, 30, 45, 60, 90, 120, 135, 150, 180]), 15.0);
/// ```
#[must_use]
pub fn aspect_step_deg(angles: &[u16]) -> f64 {
    const fn gcd(a: u16, b: u16) -> u16 {
        if b == 0 { a } else { gcd(b, a % b) }
    }
    f64::from(
        angles
            .iter()
            .flat_map(|angle| [*angle % 360, (360 - *angle % 360) % 360])
            .fold(360, gcd),
    )
}

/// The aspect a crossing names, if it is one asked for: the transit reached
/// the line `boundary_deg` of the lattice of `edge` about a natal point at
/// `natal_deg`, `orb_deg` either side of the exact line.
///
/// A line of the lattice is a multiple of [`aspect_step_deg`] from the
/// natal point; only the `angles` asked for, from either side, are aspects.
/// Moving forward, the transit enters an orb at the edge before the line
/// and leaves at the edge past it; moving back, the reverse.
#[must_use]
pub fn aspect_hit(
    (to, natal_deg): (NatalPoint, f64),
    (boundary_deg, edge, orb_deg): (f64, Edge, f64),
    motion: Motion,
    angles: &[u16],
) -> Option<HitEvent> {
    let shift = match edge {
        Edge::Exact => 0.0,
        Edge::Before => -orb_deg,
        Edge::Past => orb_deg,
    };
    let from_natal = (boundary_deg - shift - natal_deg).rem_euclid(360.0);
    // Every line is a whole degree from the natal point, so the crossing's
    // separation rounds to it.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a separation in [0, 360) rounds to a whole degree 0 to 360"
    )]
    let line = from_natal.round() as u16 % 360;
    let angle = if line > 180 { 360 - line } else { line };
    if !angles.contains(&angle) {
        return None;
    }
    let phase = match (edge, motion) {
        (Edge::Exact, _) => AspectPhase::Exact,
        (Edge::Before, Motion::Direct) | (Edge::Past, Motion::Retrograde) => AspectPhase::Entering,
        (Edge::Before | Edge::Past, _) => AspectPhase::Leaving,
    };
    Some(HitEvent::Aspect {
        to,
        angle,
        phase,
        motion,
    })
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index what they built"
    )]

    use super::*;

    #[test]
    fn a_crossing_enters_the_sign_past_its_line_or_before_it() {
        for (sign, line) in (0..12_u16).map(|k| (usize::from(k), f64::from(k) * 30.0)) {
            for nudge in [-1e-7, 0.0, 1e-7] {
                assert_eq!(entered(line + nudge, Motion::Direct, 12), sign, "{line}");
                assert_eq!(
                    entered(line + nudge, Motion::Retrograde, 12),
                    (sign + 11) % 12,
                    "{line}"
                );
            }
        }
        // 360° is Aries's line, from either side.
        assert_eq!(entered(360.0, Motion::Direct, 12), 0);
        assert_eq!(entered(359.999_999_9, Motion::Retrograde, 12), 11);
    }

    #[test]
    fn a_nakshatra_is_a_twenty_seventh() {
        let width = 360.0 / 27.0;
        assert_eq!(
            nakshatra_ingress(width, Motion::Direct),
            HitEvent::NakshatraIngress {
                into: Nakshatra::ALL[1],
                motion: Motion::Direct
            }
        );
        assert_eq!(
            nakshatra_ingress(0.0, Motion::Retrograde),
            HitEvent::NakshatraIngress {
                into: Nakshatra::ALL[26],
                motion: Motion::Retrograde
            }
        );
    }

    #[test]
    fn a_crossing_is_the_aspect_asked_for_from_either_side() {
        let sun = NatalPoint::Graha { graha: Graha::Sun };
        let conj_opp = [0, 180];
        let trine = [120];
        let at = |boundary, edge, motion, angles: &[u16]| {
            aspect_hit((sun, 100.0), (boundary, edge, 5.0), motion, angles)
        };
        for (boundary, angle) in [(100.0, 0), (280.0, 180)] {
            assert_eq!(
                at(boundary, Edge::Exact, Motion::Direct, &conj_opp),
                Some(HitEvent::Aspect {
                    to: sun,
                    angle,
                    phase: AspectPhase::Exact,
                    motion: Motion::Direct
                })
            );
        }
        // The trines either side of the natal point.
        for boundary in [220.0, 340.0] {
            assert!(at(boundary, Edge::Exact, Motion::Direct, &trine).is_some());
            assert!(at(boundary, Edge::Exact, Motion::Direct, &conj_opp).is_none());
        }
        // A square is no aspect unless asked for.
        assert!(at(190.0, Edge::Exact, Motion::Direct, &conj_opp).is_none());
        // A sesquiquadrate is its own line, not the trine or the quincunx
        // nearest it, from either side and a hair off the line.
        let leo = [0, 30, 45, 60, 90, 120, 135, 150, 180];
        for boundary in [235.0 + 1e-7, 325.0 - 1e-7] {
            match at(boundary, Edge::Exact, Motion::Direct, &leo) {
                Some(HitEvent::Aspect { angle, .. }) => assert_eq!(angle, 135),
                other => panic!("{other:?}"),
            }
        }
        // The orb's edges: 95° is before the conjunction's line, 105° past it.
        let phase = |boundary, edge, motion| match at(boundary, edge, motion, &conj_opp) {
            Some(HitEvent::Aspect { phase, .. }) => phase,
            other => panic!("{other:?}"),
        };
        assert_eq!(
            phase(95.0, Edge::Before, Motion::Direct),
            AspectPhase::Entering
        );
        assert_eq!(
            phase(105.0, Edge::Past, Motion::Direct),
            AspectPhase::Leaving
        );
        assert_eq!(
            phase(105.0, Edge::Past, Motion::Retrograde),
            AspectPhase::Entering
        );
        assert_eq!(
            phase(95.0, Edge::Before, Motion::Retrograde),
            AspectPhase::Leaving
        );
    }

    #[test]
    fn the_order_is_total() {
        let at = |jd: f64, graha, event| Hit {
            instant: JulianDay::<Utc>::literal(jd),
            graha,
            event,
        };
        let station = HitEvent::Station {
            turns: Motion::Retrograde,
        };
        let ingress = sign_ingress(30.0, Motion::Direct);
        let mut hits = vec![
            at(2.0, Graha::Sun, ingress),
            at(1.0, Graha::Mars, station),
            at(1.0, Graha::Mars, ingress),
            at(1.0, Graha::Sun, station),
        ];
        sort(&mut hits);
        assert_eq!(
            hits,
            vec![
                at(1.0, Graha::Sun, station),
                at(1.0, Graha::Mars, ingress),
                at(1.0, Graha::Mars, station),
                at(2.0, Graha::Sun, ingress),
            ]
        );
    }
}
