//! The horizon convention a rise or set is defined by, and the request
//! for one from a provider's own search (the rise and set override).
//! The SDK's solver in `teistro-astro` reads the same convention.

use core::fmt;

use serde::{Deserialize, Serialize};
use teistro_core::quantity::{JulianDay, Place, Ut1};
use teistro_core::settings::{Sunrise, SunriseConvention};

use crate::body::Body;

/// Which event at the horizon.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HorizonEventKind {
    /// The body reaches the horizon going up.
    Rise,
    /// The body reaches the horizon going down.
    Set,
    /// The body crosses the meridian above the pole.
    Transit,
    /// The body crosses the meridian below the pole.
    Antitransit,
}

impl HorizonEventKind {
    /// Every kind, in id order.
    pub const ALL: [HorizonEventKind; 4] = [
        HorizonEventKind::Rise,
        HorizonEventKind::Set,
        HorizonEventKind::Transit,
        HorizonEventKind::Antitransit,
    ];

    /// The stable id at the C boundary.
    #[must_use]
    pub fn id(self) -> u8 {
        HorizonEventKind::ALL
            .iter()
            .position(|k| *k == self)
            .and_then(|i| u8::try_from(i).ok())
            .unwrap_or(u8::MAX)
    }

    /// The kind with an id.
    #[must_use]
    pub fn from_id(id: u8) -> Option<HorizonEventKind> {
        HorizonEventKind::ALL.get(usize::from(id)).copied()
    }

    /// The key stamped in provenance.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            HorizonEventKind::Rise => "RISE",
            HorizonEventKind::Set => "SET",
            HorizonEventKind::Transit => "TRANSIT",
            HorizonEventKind::Antitransit => "ANTITRANSIT",
        }
    }
}

impl fmt::Display for HorizonEventKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.key())
    }
}

/// Which point of the disc the event is reckoned for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DiscPoint {
    /// The centre of the disc.
    Centre,
    /// The upper limb: the first point to appear and the last to vanish.
    UpperLimb,
    /// The lower limb.
    LowerLimb,
}

impl DiscPoint {
    /// The stable id at the C boundary.
    #[must_use]
    pub const fn id(self) -> u8 {
        match self {
            DiscPoint::Centre => 0,
            DiscPoint::UpperLimb => 1,
            DiscPoint::LowerLimb => 2,
        }
    }

    /// The point with an id.
    #[must_use]
    pub const fn from_id(id: u8) -> Option<DiscPoint> {
        match id {
            0 => Some(DiscPoint::Centre),
            1 => Some(DiscPoint::UpperLimb),
            2 => Some(DiscPoint::LowerLimb),
            _ => None,
        }
    }

    /// The key stamped in provenance.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            DiscPoint::Centre => "CENTRE",
            DiscPoint::UpperLimb => "UPPER_LIMB",
            DiscPoint::LowerLimb => "LOWER_LIMB",
        }
    }
}

/// Whether atmospheric refraction lifts the body at the horizon.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Refraction {
    /// The geometric horizon: what the classical texts and the panchanga
    /// reckon by.
    None,
    /// The standard refraction at the horizon (the almanac's 34 arcminutes
    /// in the SDK's solver; an engine's own standard atmosphere in a
    /// native search).
    Standard,
}

impl Refraction {
    /// The stable id at the C boundary.
    #[must_use]
    pub const fn id(self) -> u8 {
        match self {
            Refraction::None => 0,
            Refraction::Standard => 1,
        }
    }

    /// The refraction with an id.
    #[must_use]
    pub const fn from_id(id: u8) -> Option<Refraction> {
        match id {
            0 => Some(Refraction::None),
            1 => Some(Refraction::Standard),
            _ => None,
        }
    }

    /// The key stamped in provenance.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Refraction::None => "NO_REFRACTION",
            Refraction::Standard => "STANDARD_REFRACTION",
        }
    }
}

/// The horizon a rise or set is reckoned against: which point of the
/// disc, whether refraction applies, and the altitude the point reaches
/// at the event (zero for the ideal horizon; negative for a twilight or a
/// custom depression).
///
/// ```
/// use teistro_core::settings::{Sunrise, SunriseConvention};
/// use teistro_port_ephemeris::{DiscPoint, Horizon, Refraction};
///
/// let horizon = Horizon::from_convention(Sunrise::UpperLimbRefraction.into());
/// assert_eq!(horizon.disc, DiscPoint::UpperLimb);
/// assert_eq!(horizon.refraction, Refraction::Standard);
/// let twilight = Horizon::from_convention(SunriseConvention::Custom { altitude_deg: -6.0 });
/// assert_eq!(twilight.key(), "CENTRE/NO_REFRACTION/-6");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Horizon {
    /// Which point of the disc.
    pub disc: DiscPoint,
    /// Whether refraction applies.
    pub refraction: Refraction,
    /// The altitude of the point at the event, degrees.
    pub altitude_deg: f64,
}

impl Horizon {
    /// The centre of the disc on the geometric horizon: the classical
    /// sunrise (`CENTRE_NO_REFRACTION`).
    pub const CENTRE_NO_REFRACTION: Horizon = Horizon {
        disc: DiscPoint::Centre,
        refraction: Refraction::None,
        altitude_deg: 0.0,
    };

    /// The upper limb with standard refraction: the almanac's sunrise
    /// (`UPPER_LIMB_REFRACTION`).
    pub const UPPER_LIMB_REFRACTION: Horizon = Horizon {
        disc: DiscPoint::UpperLimb,
        refraction: Refraction::Standard,
        altitude_deg: 0.0,
    };

    /// The lower limb with standard refraction (`LOWER_LIMB_REFRACTION`).
    pub const LOWER_LIMB_REFRACTION: Horizon = Horizon {
        disc: DiscPoint::LowerLimb,
        refraction: Refraction::Standard,
        altitude_deg: 0.0,
    };

    /// The horizon a settings convention names: a named convention as
    /// above, or the disc's centre without refraction at a custom altitude.
    #[must_use]
    pub const fn from_convention(convention: SunriseConvention) -> Horizon {
        match convention {
            SunriseConvention::Named { which } => match which {
                Sunrise::UpperLimbRefraction => Horizon::UPPER_LIMB_REFRACTION,
                Sunrise::LowerLimbRefraction => Horizon::LOWER_LIMB_REFRACTION,
                // The classical convention, and any named convention core
                // adds before this crate learns it.
                _ => Horizon::CENTRE_NO_REFRACTION,
            },
            SunriseConvention::Custom { altitude_deg } => Horizon {
                altitude_deg,
                ..Horizon::CENTRE_NO_REFRACTION
            },
        }
    }

    /// The settings convention this horizon is, when it is a named one or
    /// a custom altitude of the centre without refraction; `None` for a
    /// combination the settings cannot name.
    #[must_use]
    pub fn convention(&self) -> Option<SunriseConvention> {
        if *self == Horizon::CENTRE_NO_REFRACTION {
            Some(Sunrise::CentreNoRefraction.into())
        } else if *self == Horizon::UPPER_LIMB_REFRACTION {
            Some(Sunrise::UpperLimbRefraction.into())
        } else if *self == Horizon::LOWER_LIMB_REFRACTION {
            Some(Sunrise::LowerLimbRefraction.into())
        } else if self.disc == DiscPoint::Centre && self.refraction == Refraction::None {
            Some(SunriseConvention::Custom {
                altitude_deg: self.altitude_deg,
            })
        } else {
            None
        }
    }

    /// The key stamped in provenance: disc, refraction and altitude.
    #[must_use]
    pub fn key(&self) -> String {
        format!(
            "{}/{}/{}",
            self.disc.key(),
            self.refraction.key(),
            self.altitude_deg
        )
    }
}

impl fmt::Display for Horizon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.key())
    }
}

/// A request for the next horizon event of a body at a place from a
/// provider's own search.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct HorizonRequest {
    /// The body.
    pub body: Body,
    /// Which event.
    pub kind: HorizonEventKind,
    /// The place.
    pub place: Place,
    /// The search begins here.
    pub from: JulianDay<Ut1>,
    /// The search ends this many days later; an event beyond it is
    /// reported as absent.
    pub window_days: f64,
    /// The horizon convention.
    pub horizon: Horizon,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    #[test]
    fn conventions_round_trip_and_ids_hold() {
        for kind in HorizonEventKind::ALL {
            assert_eq!(HorizonEventKind::from_id(kind.id()), Some(kind));
            assert_eq!(kind.to_string(), kind.key());
        }
        for disc in [
            DiscPoint::Centre,
            DiscPoint::UpperLimb,
            DiscPoint::LowerLimb,
        ] {
            assert_eq!(DiscPoint::from_id(disc.id()), Some(disc));
        }
        assert_eq!(Refraction::from_id(1), Some(Refraction::Standard));
        assert_eq!(Refraction::from_id(9), None);
        for named in [
            Sunrise::CentreNoRefraction,
            Sunrise::UpperLimbRefraction,
            Sunrise::LowerLimbRefraction,
        ] {
            let convention: SunriseConvention = named.into();
            assert_eq!(
                Horizon::from_convention(convention).convention(),
                Some(convention)
            );
        }
        let custom = SunriseConvention::Custom {
            altitude_deg: -12.0,
        };
        assert_eq!(Horizon::from_convention(custom).convention(), Some(custom));
        let unnamed = Horizon {
            disc: DiscPoint::UpperLimb,
            refraction: Refraction::None,
            altitude_deg: 0.0,
        };
        assert_eq!(unnamed.convention(), None);
        assert_eq!(
            Horizon::UPPER_LIMB_REFRACTION.to_string(),
            "UPPER_LIMB/STANDARD_REFRACTION/0"
        );
    }
}
