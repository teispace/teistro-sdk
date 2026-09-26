//! The horizon convention a rise or set is defined by, and the request
//! for one from a provider's own search (the rise and set override).
//! The SDK's solver in `teistro-astro` reads the same convention.

use core::fmt;

use serde::{Deserialize, Serialize};
use teistro_core::quantity::{JulianDay, Place, Ut1};
use teistro_core::settings::{Atmosphere, Sunrise, SunriseConvention};

use crate::body::Body;

/// Which event at the horizon.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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

/// Whether atmospheric refraction lifts the body at the horizon, and by
/// what.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Refraction {
    /// The geometric horizon: what the classical texts and the panchanga
    /// reckon by.
    None,
    /// The standard refraction at the horizon (the almanac's 34 arcminutes
    /// in the SDK's solver; an engine's own standard atmosphere in a
    /// native search).
    Standard,
    /// The refraction through a named air, resolved at the observer's
    /// height, which a native search is given too
    /// (`03-design/horizon-atmosphere.md`).
    Atmosphere(Atmosphere),
}

impl Refraction {
    /// The stable id at the C boundary; an atmosphere's air crosses
    /// beside it.
    #[must_use]
    pub const fn id(self) -> u8 {
        match self {
            Refraction::None => 0,
            Refraction::Standard => 1,
            Refraction::Atmosphere(_) => 2,
        }
    }

    /// The refraction with an id, and the air an atmosphere is given
    /// (read only for id 2).
    #[must_use]
    pub const fn from_id(id: u8, air: Atmosphere) -> Option<Refraction> {
        match id {
            0 => Some(Refraction::None),
            1 => Some(Refraction::Standard),
            2 => Some(Refraction::Atmosphere(air)),
            _ => None,
        }
    }

    /// The key stamped in provenance; an atmosphere's air is stamped by
    /// [`Horizon::key`].
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Refraction::None => "NO_REFRACTION",
            Refraction::Standard => "STANDARD_REFRACTION",
            Refraction::Atmosphere(_) => "ATMOSPHERE",
        }
    }
}

/// The horizon a rise or set is reckoned against: which point of the
/// disc, whether refraction applies, and the altitude the point reaches
/// at the event (zero for the ideal horizon; negative for a twilight or a
/// custom depression).
///
/// ```
/// use teistro_core::settings::{Atmosphere, Sunrise, SunriseConvention};
/// use teistro_port_ephemeris::{DiscPoint, Horizon, Refraction};
///
/// let horizon = Horizon::from_convention(Sunrise::UpperLimbRefraction.into());
/// assert_eq!(horizon.disc, DiscPoint::UpperLimb);
/// assert_eq!(horizon.refraction, Refraction::Standard);
/// let twilight = Horizon::from_convention(SunriseConvention::Custom { altitude_deg: -6.0 });
/// assert_eq!(twilight.key(), "CENTRE/NO_REFRACTION/-6");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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

    /// The horizon a named convention is.
    const fn named(which: Sunrise) -> Horizon {
        match which {
            Sunrise::UpperLimbRefraction => Horizon::UPPER_LIMB_REFRACTION,
            Sunrise::LowerLimbRefraction => Horizon::LOWER_LIMB_REFRACTION,
            // The classical convention, and any named convention core
            // adds before this crate learns it.
            _ => Horizon::CENTRE_NO_REFRACTION,
        }
    }

    /// The horizon a settings convention names: a named convention as
    /// above, the disc's centre without refraction at a custom altitude,
    /// or a refracted named convention with its air.
    ///
    /// The settings refuse an air given to a convention that does not
    /// refract; one that reaches here anyway keeps its named horizon,
    /// which has no refraction for the air to replace.
    #[must_use]
    pub const fn from_convention(convention: SunriseConvention) -> Horizon {
        match convention {
            SunriseConvention::Named { which } => Horizon::named(which),
            SunriseConvention::Custom { altitude_deg } => Horizon {
                altitude_deg,
                ..Horizon::CENTRE_NO_REFRACTION
            },
            SunriseConvention::Atmospheric { which, air } => {
                let named = Horizon::named(which);
                match named.refraction {
                    Refraction::None => named,
                    _ => Horizon {
                        refraction: Refraction::Atmosphere(air),
                        ..named
                    },
                }
            }
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
        } else if let Refraction::Atmosphere(air) = self.refraction {
            let standard = Horizon {
                refraction: Refraction::Standard,
                ..*self
            };
            standard.convention().and_then(|named| match named {
                SunriseConvention::Named { which } => {
                    Some(SunriseConvention::Atmospheric { which, air })
                }
                _ => None,
            })
        } else {
            None
        }
    }

    /// The key stamped in provenance: disc, refraction and altitude, and
    /// an atmosphere's air as it was named (`STANDARD` for a part left to
    /// the standard at the place).
    #[must_use]
    pub fn key(&self) -> String {
        let refraction = match self.refraction {
            Refraction::Atmosphere(air) => {
                let part = |value: Option<f64>, unit: &str| {
                    value.map_or_else(|| String::from("STANDARD"), |v| format!("{v}{unit}"))
                };
                format!(
                    "ATMOSPHERE[{},{}]",
                    part(air.pressure_hpa, "hPa"),
                    part(air.temperature_c, "C")
                )
            }
            other => String::from(other.key()),
        };
        format!("{}/{}/{}", self.disc.key(), refraction, self.altitude_deg)
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
        let air = Atmosphere::given(987.0, 4.5);
        for refraction in [
            Refraction::None,
            Refraction::Standard,
            Refraction::Atmosphere(air),
        ] {
            assert_eq!(Refraction::from_id(refraction.id(), air), Some(refraction));
        }
        assert_eq!(Refraction::from_id(9, air), None);
        for which in [Sunrise::UpperLimbRefraction, Sunrise::LowerLimbRefraction] {
            for air in [Atmosphere::STANDARD, air] {
                let convention = SunriseConvention::Atmospheric { which, air };
                let horizon = Horizon::from_convention(convention);
                assert_eq!(horizon.refraction, Refraction::Atmosphere(air));
                assert_eq!(horizon.convention(), Some(convention));
            }
        }
        // The settings refuse an air for a convention that does not
        // refract; reaching here, it keeps its geometric horizon.
        let refused = SunriseConvention::Atmospheric {
            which: Sunrise::CentreNoRefraction,
            air,
        };
        assert_eq!(
            Horizon::from_convention(refused),
            Horizon::CENTRE_NO_REFRACTION
        );
        assert_eq!(
            Horizon::from_convention(SunriseConvention::Atmospheric {
                which: Sunrise::UpperLimbRefraction,
                air: Atmosphere {
                    pressure_hpa: Some(987.0),
                    temperature_c: None,
                },
            })
            .key(),
            "UPPER_LIMB/ATMOSPHERE[987hPa,STANDARD]/0"
        );
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
