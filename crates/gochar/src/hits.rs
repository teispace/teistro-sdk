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
}

impl HitEvent {
    /// The kind's place in a tie's order: ingresses before stations, signs
    /// before nakshatras.
    const fn rank(self) -> u8 {
        match self {
            HitEvent::SignIngress { .. } => 0,
            HitEvent::NakshatraIngress { .. } => 1,
            HitEvent::Station { .. } => 2,
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

#[cfg(test)]
mod tests {
    #![allow(
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
