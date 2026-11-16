//! The horizon convention a rise or set is defined by, and the request
//! for one from a provider's own search (the rise and set override).
//! The SDK's solver in `teistro-astro` reads the same convention.

use core::fmt;

use serde::{Deserialize, Serialize};
use teistro_core::quantity::{JulianDay, Place, Ut1};
/// The point of the disc and the refraction a horizon is reckoned by:
/// the settings' own types, so a convention and its horizon share them.
pub use teistro_core::settings::{DiscPoint, Refraction};
use teistro_core::settings::{Sunrise, SunriseConvention, dip_deg};

use crate::body::Body;

/// Which event at the horizon.
///
/// ```
/// use teistro_port_ephemeris::HorizonEventKind;
///
/// assert_eq!(HorizonEventKind::ALL.map(HorizonEventKind::key), ["RISE", "SET", "TRANSIT", "ANTITRANSIT"]);
/// ```
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
    ///
    /// ```
    /// use teistro_port_ephemeris::HorizonEventKind;
    ///
    /// assert_eq!(HorizonEventKind::Transit.id(), 2);
    /// ```
    #[must_use]
    pub fn id(self) -> u8 {
        HorizonEventKind::ALL
            .iter()
            .position(|k| *k == self)
            .and_then(|i| u8::try_from(i).ok())
            .unwrap_or(u8::MAX)
    }

    /// The kind with an id.
    ///
    /// ```
    /// use teistro_port_ephemeris::HorizonEventKind;
    ///
    /// for kind in HorizonEventKind::ALL {
    ///     assert_eq!(HorizonEventKind::from_id(kind.id()), Some(kind));
    /// }
    /// assert_eq!(HorizonEventKind::from_id(4), None);
    /// ```
    #[must_use]
    pub fn from_id(id: u8) -> Option<HorizonEventKind> {
        HorizonEventKind::ALL.get(usize::from(id)).copied()
    }

    /// The key stamped in provenance.
    ///
    /// ```
    /// use teistro_port_ephemeris::HorizonEventKind;
    ///
    /// assert_eq!(HorizonEventKind::Antitransit.key(), "ANTITRANSIT");
    /// ```
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

    /// The centre of the disc with standard refraction
    /// (`CENTRE_REFRACTION`): the sunrise of the Indian almanacs that take
    /// the disc's middle, and the convention Nepal's daily panchanga's
    /// sunrise is nearest, measured (C39).
    pub const CENTRE_REFRACTION: Horizon = Horizon {
        disc: DiscPoint::Centre,
        refraction: Refraction::Standard,
        altitude_deg: 0.0,
    };

    /// The horizon a named convention is: its point of the disc, with
    /// standard refraction or without, on the ideal horizon.
    const fn named(which: Sunrise) -> Horizon {
        Horizon {
            disc: which.disc(),
            refraction: if which.refracts() {
                Refraction::Standard
            } else {
                Refraction::None
            },
            altitude_deg: 0.0,
        }
    }

    /// The horizon a settings convention names: a named convention as
    /// above, the disc's centre without refraction at a custom altitude,
    /// or a refracted named convention with its air.
    ///
    /// The settings refuse an air given to a convention that does not
    /// refract; one that reaches here anyway keeps its named horizon,
    /// which has no refraction for the air to replace.
    ///
    /// ```
    /// use teistro_core::settings::Sunrise;
    /// use teistro_port_ephemeris::{DiscPoint, Horizon, HorizonEventKind, Refraction};
    ///
    /// let horizon = Horizon::from_convention(Sunrise::CentreRefraction.into());
    /// assert_eq!(horizon, Horizon::CENTRE_REFRACTION);
    /// assert_eq!((horizon.disc, horizon.refraction), (DiscPoint::Centre, Refraction::Standard));
    /// ```
    #[must_use]
    pub fn from_convention(convention: SunriseConvention) -> Horizon {
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
            SunriseConvention::Composed {
                disc,
                refraction,
                height_m,
            } => Horizon {
                disc,
                refraction,
                altitude_deg: -dip_deg(refraction, height_m),
            },
        }
    }

    /// The settings convention this horizon is: a named one, its air, or
    /// a custom altitude of the centre without refraction, where one
    /// names it; else the composed convention whose dip lowers it this
    /// far. `None` only for a horizon raised above the ideal one at a
    /// limb or with refraction, which no convention names.
    ///
    /// ```
    /// use teistro_core::settings::{DiscPoint, Refraction, Sunrise, SunriseConvention};
    /// use teistro_port_ephemeris::Horizon;
    ///
    /// for which in Sunrise::ALL {
    ///     let convention: SunriseConvention = (*which).into();
    ///     assert_eq!(Horizon::from_convention(convention).convention(), Some(convention));
    /// }
    /// // A composed convention with a height reads back with that height.
    /// let deck = SunriseConvention::Composed { disc: DiscPoint::UpperLimb, refraction: Refraction::Standard, height_m: 30.0 };
    /// let Some(SunriseConvention::Composed { height_m, .. }) = Horizon::from_convention(deck).convention() else { panic!() };
    /// assert!((height_m - 30.0).abs() < 1e-9);
    /// // A limb raised above the horizon is a horizon no convention names.
    /// let raised = Horizon { altitude_deg: 2.0, ..Horizon::UPPER_LIMB_REFRACTION };
    /// assert_eq!(raised.convention(), None);
    /// ```
    #[must_use]
    pub fn convention(&self) -> Option<SunriseConvention> {
        let named = Sunrise::of_parts(self.disc, self.refraction.refracts());
        if self.altitude_deg == 0.0 {
            return Some(match self.refraction {
                Refraction::Atmosphere(air) => SunriseConvention::Atmospheric { which: named, air },
                Refraction::None | Refraction::Standard => named.into(),
            });
        }
        if self.disc == DiscPoint::Centre && self.refraction == Refraction::None {
            return Some(SunriseConvention::Custom {
                altitude_deg: self.altitude_deg,
            });
        }
        if self.altitude_deg > 0.0 {
            return None;
        }
        // The height whose dip is this depression: the inverse of
        // `dip_deg`, the refracted dip's square or the tangent's.
        let dip = -self.altitude_deg;
        let height_m = match self.refraction {
            Refraction::None => {
                let radius = teistro_core::settings::EARTH_MEAN_RADIUS_M;
                radius / teistro_core::math::cos(dip.to_radians()) - radius
            }
            Refraction::Standard | Refraction::Atmosphere(_) => {
                let root = dip * 60.0 / teistro_core::settings::DIP_REFRACTED_ARCMIN_PER_ROOT_M;
                root * root
            }
        };
        Some(SunriseConvention::Composed {
            disc: self.disc,
            refraction: self.refraction,
            height_m,
        })
    }

    /// The key stamped in provenance: disc, refraction and altitude, and
    /// an atmosphere's air as it was named (`STANDARD` for a part left to
    /// the standard at the place).
    ///
    /// ```
    /// use teistro_port_ephemeris::{DiscPoint, Horizon, HorizonEventKind, Refraction};
    ///
    /// assert_eq!(Horizon::UPPER_LIMB_REFRACTION.key(), "UPPER_LIMB/STANDARD_REFRACTION/0");
    /// ```
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

/// The horizons a day's Sun and Moon rise and set by: `day.sunrise`'s
/// and `day.moonrise`'s, each read with its own body's semidiameter and
/// parallax (SR-102). One horizon is both.
///
/// ```
/// use teistro_core::settings::{Sunrise, root};
/// use teistro_port_ephemeris::{Body, Horizon, Horizons};
///
/// let mut day = root().day;
/// day.sunrise = Sunrise::CentreRefraction.into();
/// let horizons = Horizons::of_day(&day);
/// // Nepal's sunrise does not move the Moon off the almanacs' upper limb.
/// assert_eq!(horizons.of(Body::Sun), Horizon::CENTRE_REFRACTION);
/// assert_eq!(horizons.of(Body::Moon), Horizon::UPPER_LIMB_REFRACTION);
/// assert_eq!(Horizons::from(Horizon::CENTRE_NO_REFRACTION).moon, Horizon::CENTRE_NO_REFRACTION);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Horizons {
    /// The Sun's, and every body's but the Moon's.
    pub sun: Horizon,
    /// The Moon's.
    pub moon: Horizon,
}

impl Horizons {
    /// The day settings' two conventions.
    #[must_use]
    pub fn of_day(day: &teistro_core::settings::Day) -> Horizons {
        Horizons {
            sun: Horizon::from_convention(day.sunrise),
            moon: Horizon::from_convention(day.moonrise),
        }
    }

    /// The horizon a body rises by: the Moon's for the Moon, the Sun's
    /// for every other.
    #[must_use]
    pub const fn of(&self, body: Body) -> Horizon {
        match body {
            Body::Moon => self.moon,
            _ => self.sun,
        }
    }
}

impl From<Horizon> for Horizons {
    fn from(both: Horizon) -> Horizons {
        Horizons {
            sun: both,
            moon: both,
        }
    }
}

/// A request for the next horizon event of a body at a place from a
/// provider's own search.
///
/// ```
/// use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
/// use teistro_core::quantity::JulianDay;
/// use teistro_port_ephemeris::{Body, Horizon, HorizonEventKind, HorizonRequest};
///
/// let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
/// // The next sunrise at Kathmandu within a day of midnight UT, 1 January 2024.
/// let request = HorizonRequest {
///     body: Body::Sun,
///     kind: HorizonEventKind::Rise,
///     place: kathmandu,
///     from: JulianDay::literal(2_460_310.5),
///     window_days: 1.0,
///     horizon: Horizon::UPPER_LIMB_REFRACTION,
/// };
/// assert_eq!(request.kind.key(), "RISE");
/// ```
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
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;
    use teistro_core::settings::Atmosphere;

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
        // Every named convention is a horizon of its own and reads back as
        // itself, so a member core adds without a horizon here fails.
        let horizons: Vec<Horizon> = Sunrise::ALL
            .iter()
            .map(|which| Horizon::from_convention((*which).into()))
            .collect();
        for (which, horizon) in Sunrise::ALL.iter().zip(&horizons) {
            assert_eq!(horizon.convention(), Some((*which).into()), "{which}");
            let same = horizons.iter().filter(|other| *other == horizon).count();
            assert_eq!(same, 1, "{which} shares its horizon");
        }
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
        let custom = SunriseConvention::Custom {
            altitude_deg: -12.0,
        };
        assert_eq!(Horizon::from_convention(custom).convention(), Some(custom));
        let raised = Horizon {
            disc: DiscPoint::LowerLimb,
            refraction: Refraction::None,
            altitude_deg: 1.0,
        };
        assert_eq!(raised.convention(), None);
        // A composed convention is its parts, its height a dip of the
        // horizon by the air's rule or the tangent's.
        for refraction in [
            Refraction::None,
            Refraction::Standard,
            Refraction::Atmosphere(air),
        ] {
            for disc in DiscPoint::ALL {
                let composed = SunriseConvention::Composed {
                    disc: *disc,
                    refraction,
                    height_m: 100.0,
                };
                let horizon = Horizon::from_convention(composed);
                assert_eq!((horizon.disc, horizon.refraction), (*disc, refraction));
                assert_eq!(horizon.altitude_deg, -dip_deg(refraction, 100.0));
                if *disc != DiscPoint::Centre || refraction != Refraction::None {
                    let Some(SunriseConvention::Composed { height_m, .. }) = horizon.convention()
                    else {
                        panic!("{composed:?}")
                    };
                    assert!((height_m - 100.0).abs() < 1e-6, "{height_m}");
                }
            }
        }
        assert_eq!(
            Horizon::UPPER_LIMB_REFRACTION.to_string(),
            "UPPER_LIMB/STANDARD_REFRACTION/0"
        );
    }
}
