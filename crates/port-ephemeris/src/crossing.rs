//! The vocabulary of a crossing search: the quantity searched (a body's
//! longitude, its speed, or a composite angle of two bodies), the lattice
//! of boundaries it is searched against, and the events found. The SDK's
//! kernel in `teistro-astro` (`events`) and a provider's own search (the
//! crossings override, [`crate::EphemerisProvider::crossings`]) speak the
//! same words, so the kit can hold one to the other.

use core::fmt;

use serde::{Deserialize, Serialize};
use teistro_core::quantity::{JulianDay, Place, Ut1};

use crate::body::Body;
use crate::frame::Frame;

/// What is searched for a boundary.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Quantity {
    /// A body's ecliptic longitude, degrees.
    Longitude(Body),
    /// A body's rate of longitude, degrees a day; a target of zero is a
    /// station.
    Speed(Body),
    /// `a × longitude(first) + b × longitude(second)`, degrees, reduced to
    /// a circle: the tithi and the karana (Moon less Sun), the yoga (Moon
    /// plus Sun), an aspect (first less second at one angle).
    Composite {
        /// The first body's coefficient.
        a: f64,
        /// The first body.
        first: Body,
        /// The second body's coefficient.
        b: f64,
        /// The second body.
        second: Body,
    },
}

impl Quantity {
    /// The Moon less the Sun: the tithi and karana lattices.
    pub const ELONGATION: Quantity = Quantity::Composite {
        a: 1.0,
        first: Body::Moon,
        b: -1.0,
        second: Body::Sun,
    };

    /// The Moon plus the Sun: the yoga lattice.
    pub const MOON_PLUS_SUN: Quantity = Quantity::Composite {
        a: 1.0,
        first: Body::Moon,
        b: 1.0,
        second: Body::Sun,
    };

    /// The angle from `second` to `first`: an aspect searched as a single
    /// target.
    #[must_use]
    pub const fn separation(first: Body, second: Body) -> Quantity {
        Quantity::Composite {
            a: 1.0,
            first,
            b: -1.0,
            second,
        }
    }

    /// Whether the quantity is an angle on a circle (a longitude or a
    /// composite) rather than a rate.
    #[must_use]
    pub const fn wraps(self) -> bool {
        !matches!(self, Quantity::Speed(_))
    }

    /// The bodies the quantity reads.
    #[must_use]
    pub const fn bodies(self) -> (Body, Option<Body>) {
        match self {
            Quantity::Longitude(body) | Quantity::Speed(body) => (body, None),
            Quantity::Composite { first, second, .. } => (first, Some(second)),
        }
    }

    /// The kind's stable id at the C boundary: 0 a longitude, 1 a speed,
    /// 2 a composite.
    #[must_use]
    pub const fn kind_id(self) -> u8 {
        match self {
            Quantity::Longitude(_) => 0,
            Quantity::Speed(_) => 1,
            Quantity::Composite { .. } => 2,
        }
    }

    /// The quantity from its C parts.
    #[must_use]
    pub fn from_parts(
        kind: u8,
        first: Body,
        second: Option<Body>,
        a: f64,
        b: f64,
    ) -> Option<Quantity> {
        match (kind, second) {
            (0, _) => Some(Quantity::Longitude(first)),
            (1, _) => Some(Quantity::Speed(first)),
            (2, Some(second)) => Some(Quantity::Composite {
                a,
                first,
                b,
                second,
            }),
            _ => None,
        }
    }

    /// The key stamped in provenance.
    #[must_use]
    pub fn key(self) -> String {
        match self {
            Quantity::Longitude(body) => format!("LONGITUDE({})", body.key()),
            Quantity::Speed(body) => format!("SPEED({})", body.key()),
            Quantity::Composite {
                a,
                first,
                b,
                second,
            } => format!("{a}*{}{b:+}*{}", first.key(), second.key()),
        }
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.key())
    }
}

/// A lattice of boundaries: every `origin + k × step` degrees, or the
/// single boundary at the origin when the step is zero.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Lattice {
    /// The first line, degrees.
    pub origin_deg: f64,
    /// The spacing, degrees; zero for the single target at the origin.
    pub step_deg: f64,
}

impl Lattice {
    /// The twelve signs.
    pub const SIGNS: Lattice = Lattice {
        origin_deg: 0.0,
        step_deg: 30.0,
    };

    /// The twenty-seven nakshatras.
    pub const NAKSHATRAS: Lattice = Lattice {
        origin_deg: 0.0,
        step_deg: 360.0 / 27.0,
    };

    /// The thirty tithis of the elongation.
    pub const TITHIS: Lattice = Lattice {
        origin_deg: 0.0,
        step_deg: 12.0,
    };

    /// The sixty karanas of the elongation.
    pub const KARANAS: Lattice = Lattice {
        origin_deg: 0.0,
        step_deg: 6.0,
    };

    /// The twenty-seven yogas of the Moon plus the Sun.
    pub const YOGAS: Lattice = Lattice::NAKSHATRAS;

    /// One boundary.
    #[must_use]
    pub const fn single(target_deg: f64) -> Lattice {
        Lattice {
            origin_deg: target_deg,
            step_deg: 0.0,
        }
    }

    /// Whether the lattice is one boundary.
    #[must_use]
    pub fn is_single(&self) -> bool {
        self.step_deg == 0.0
    }

    /// The `k`th line, degrees, unwrapped.
    #[must_use]
    pub fn line(&self, k: i64) -> f64 {
        // A line index is a small integer; the conversion is exact far
        // beyond any lattice a search visits.
        #[allow(
            clippy::cast_precision_loss,
            reason = "lattice indices are small integers"
        )]
        let k = k as f64;
        self.origin_deg + k * self.step_deg
    }
}

/// Which way a boundary was passed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Direction {
    /// The quantity was increasing through the boundary.
    Rising,
    /// The quantity was decreasing through it (a retrograde re-entry).
    Falling,
}

impl Direction {
    /// The stable id at the C boundary.
    #[must_use]
    pub const fn id(self) -> u8 {
        match self {
            Direction::Rising => 0,
            Direction::Falling => 1,
        }
    }

    /// The direction with an id.
    #[must_use]
    pub const fn from_id(id: u8) -> Option<Direction> {
        match id {
            0 => Some(Direction::Rising),
            1 => Some(Direction::Falling),
            _ => None,
        }
    }

    /// The key stamped in provenance.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Direction::Rising => "RISING",
            Direction::Falling => "FALLING",
        }
    }
}

/// A crossing found.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Event {
    /// The instant, UT1.
    pub instant: JulianDay<Ut1>,
    /// The boundary reached, degrees in `[0, 360)` for an angle.
    pub boundary_deg: f64,
    /// Which way it was passed.
    pub direction: Direction,
    /// How many times the source was asked to place this event beyond the
    /// sampling that bracketed it; zero for a provider's own search.
    pub evaluations: u32,
}

/// A request for every crossing of a quantity over a lattice inside a
/// window, from a provider's own search.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CrossingRequest {
    /// What is searched.
    pub quantity: Quantity,
    /// The boundaries.
    pub lattice: Lattice,
    /// The window's start, UT1.
    pub from: JulianDay<Ut1>,
    /// The window's end, UT1, after the start.
    pub to: JulianDay<Ut1>,
    /// How closely each instant is placed, days.
    pub tolerance_days: f64,
    /// The frame the quantity is read in (the zodiac, the centre, the
    /// corrections).
    pub frame: Frame,
    /// The observer, for a topocentric frame.
    pub observer: Option<Place>,
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::float_cmp,
        reason = "tests fail by panicking"
    )]

    use super::*;

    #[test]
    fn the_vocabulary_round_trips_and_names_itself() {
        for quantity in [
            Quantity::Longitude(Body::Sun),
            Quantity::Speed(Body::Mars),
            Quantity::ELONGATION,
            Quantity::MOON_PLUS_SUN,
            Quantity::separation(Body::Mars, Body::Saturn),
        ] {
            let (first, second) = quantity.bodies();
            let (a, b) = match quantity {
                Quantity::Composite { a, b, .. } => (a, b),
                _ => (0.0, 0.0),
            };
            assert_eq!(
                Quantity::from_parts(quantity.kind_id(), first, second, a, b),
                Some(quantity)
            );
        }
        assert!(Quantity::from_parts(2, Body::Sun, None, 1.0, 1.0).is_none());
        assert!(Quantity::from_parts(9, Body::Sun, None, 1.0, 1.0).is_none());
        assert!(Quantity::ELONGATION.wraps() && !Quantity::Speed(Body::Sun).wraps());
        assert_eq!(Quantity::ELONGATION.to_string(), "1*MOON-1*SUN");
        assert_eq!(Quantity::Longitude(Body::Sun).key(), "LONGITUDE(SUN)");
        for direction in [Direction::Rising, Direction::Falling] {
            assert_eq!(Direction::from_id(direction.id()), Some(direction));
        }
        assert!(Direction::from_id(2).is_none());
        assert_eq!(Lattice::SIGNS.line(4), 120.0);
        assert_eq!(Lattice::single(100.0).line(7), 100.0);
        assert!(Lattice::single(100.0).is_single() && !Lattice::TITHIS.is_single());
        assert_eq!(Lattice::YOGAS.step_deg, Lattice::NAKSHATRAS.step_deg);
        let json = serde_json::to_string(&Quantity::ELONGATION).unwrap();
        assert!(json.contains("COMPOSITE"));
    }
}
