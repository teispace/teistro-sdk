//! A frame: the answer to "what do these numbers mean". Five facts, each
//! a closed enumeration, packed to 32 bits for the C boundary
//! (`docs/03-design/ephemeris-port-and-adapters.md`, §3).

use core::fmt;

use serde::Serialize;
use teistro_core::catalogue::Ayanamsha;

use crate::error::ProviderError;

/// Where a position is seen from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Centre {
    /// The centre of the Earth.
    Geocentric,
    /// An observer on the Earth; the request carries the place.
    Topocentric,
    /// The centre of the Sun.
    Heliocentric,
    /// The solar-system barycentre.
    Barycentric,
}

impl Centre {
    /// The key stamped in provenance.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Centre::Geocentric => "GEOCENTRIC",
            Centre::Topocentric => "TOPOCENTRIC",
            Centre::Heliocentric => "HELIOCENTRIC",
            Centre::Barycentric => "BARYCENTRIC",
        }
    }
}

/// The equinox and equator the coordinates refer to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Equinox {
    /// The equinox of date.
    OfDate,
    /// The J2000.0 equinox.
    J2000,
}

impl Equinox {
    /// The key stamped in provenance.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Equinox::OfDate => "OF_DATE",
            Equinox::J2000 => "J2000",
        }
    }
}

/// The coordinate system of a position.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Coordinates {
    /// Ecliptic longitude and latitude.
    Ecliptic,
    /// Right ascension and declination.
    Equatorial,
}

impl Coordinates {
    /// The key stamped in provenance.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Coordinates::Ecliptic => "ECLIPTIC",
            Coordinates::Equatorial => "EQUATORIAL",
        }
    }
}

/// Which zodiac longitudes are measured in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Zodiac {
    /// From the equinox.
    Tropical,
    /// From the equinox less a catalogued ayanamsha (the mean value,
    /// without the nutation in longitude).
    Sidereal {
        /// The ayanamsha.
        ayanamsha: Ayanamsha,
    },
}

impl Zodiac {
    /// A sidereal zodiac by a catalogued ayanamsha.
    #[must_use]
    pub const fn sidereal(ayanamsha: Ayanamsha) -> Zodiac {
        Zodiac::Sidereal { ayanamsha }
    }

    /// The ayanamsha, when sidereal.
    #[must_use]
    pub const fn ayanamsha(self) -> Option<Ayanamsha> {
        match self {
            Zodiac::Tropical => None,
            Zodiac::Sidereal { ayanamsha } => Some(ayanamsha),
        }
    }
}

impl fmt::Display for Zodiac {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Zodiac::Tropical => f.write_str("TROPICAL"),
            Zodiac::Sidereal { ayanamsha } => write!(f, "SIDEREAL({})", ayanamsha.key()),
        }
    }
}

/// Which corrections a position includes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "four independent switches of a frame"
)]
pub struct Corrections {
    /// Light-time correction applied.
    pub light_time: bool,
    /// Annual aberration applied.
    pub aberration: bool,
    /// Gravitational deflection applied.
    pub deflection: bool,
    /// Nutation applied (the true equinox rather than the mean).
    pub nutation: bool,
}

impl Corrections {
    /// Every correction: an apparent position.
    pub const APPARENT: Corrections = Corrections {
        light_time: true,
        aberration: true,
        deflection: true,
        nutation: true,
    };
    /// No correction: a geometric position.
    pub const GEOMETRIC: Corrections = Corrections {
        light_time: false,
        aberration: false,
        deflection: false,
        nutation: false,
    };

    /// The names of the corrections applied, in bit order.
    #[must_use]
    pub fn names(self) -> Vec<&'static str> {
        [
            (self.light_time, "light_time"),
            (self.aberration, "aberration"),
            (self.deflection, "deflection"),
            (self.nutation, "nutation"),
        ]
        .into_iter()
        .filter(|(on, _)| *on)
        .map(|(_, name)| name)
        .collect()
    }

    /// The key stamped in provenance: `APPARENT`, `GEOMETRIC`, or the
    /// corrections applied.
    #[must_use]
    pub fn key(self) -> String {
        if self == Corrections::APPARENT {
            String::from("APPARENT")
        } else if self == Corrections::GEOMETRIC {
            String::from("GEOMETRIC")
        } else {
            self.names().join("+")
        }
    }
}

/// A position frame.
///
/// ```
/// use teistro_core::catalogue::Ayanamsha;
/// use teistro_port_ephemeris::{Coordinates, Frame, Zodiac};
///
/// let frame = Frame::CANONICAL
///     .with_coordinates(Coordinates::Equatorial)
///     .with_zodiac(Zodiac::sidereal(Ayanamsha::Lahiri));
/// assert_eq!(Frame::try_from_bits(frame.to_bits()), Ok(frame));
/// assert_eq!(frame.key(), "GEOCENTRIC/OF_DATE/EQUATORIAL/SIDEREAL(LAHIRI)/APPARENT");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct Frame {
    /// Where the position is seen from.
    pub centre: Centre,
    /// Which equinox.
    pub equinox: Equinox,
    /// Which coordinates.
    pub coordinates: Coordinates,
    /// Which zodiac.
    pub zodiac: Zodiac,
    /// Which corrections.
    pub corrections: Corrections,
}

impl Frame {
    /// The SDK's canonical frame: apparent geocentric ecliptic of date,
    /// tropical; what both licensed engines return by default and what
    /// every chart module consumes.
    pub const CANONICAL: Frame = Frame {
        centre: Centre::Geocentric,
        equinox: Equinox::OfDate,
        coordinates: Coordinates::Ecliptic,
        zodiac: Zodiac::Tropical,
        corrections: Corrections::APPARENT,
    };

    /// The same frame seen from elsewhere.
    #[must_use]
    pub const fn with_centre(self, centre: Centre) -> Frame {
        Frame { centre, ..self }
    }

    /// The same frame with other coordinates.
    #[must_use]
    pub const fn with_coordinates(self, coordinates: Coordinates) -> Frame {
        Frame {
            coordinates,
            ..self
        }
    }

    /// The same frame in another zodiac.
    #[must_use]
    pub const fn with_zodiac(self, zodiac: Zodiac) -> Frame {
        Frame { zodiac, ..self }
    }

    /// The frame packed for the C boundary: bits 0 and 1 the centre, bit 2
    /// the equinox, bit 3 the coordinates, bits 4 to 7 the corrections,
    /// bit 8 sidereal, bits 16 to 31 the ayanamsha's catalogue id.
    #[must_use]
    pub fn to_bits(self) -> u32 {
        let centre = match self.centre {
            Centre::Geocentric => 0,
            Centre::Topocentric => 1,
            Centre::Heliocentric => 2,
            Centre::Barycentric => 3,
        };
        let mut bits = centre;
        if self.equinox == Equinox::J2000 {
            bits |= 1 << 2;
        }
        if self.coordinates == Coordinates::Equatorial {
            bits |= 1 << 3;
        }
        let c = self.corrections;
        bits |= u32::from(c.light_time) << 4;
        bits |= u32::from(c.aberration) << 5;
        bits |= u32::from(c.deflection) << 6;
        bits |= u32::from(c.nutation) << 7;
        if let Zodiac::Sidereal { ayanamsha } = self.zodiac {
            bits |= 1 << 8;
            bits |= u32::from(ayanamsha.id()) << 16;
        }
        bits
    }

    /// The frame from its packed form.
    ///
    /// # Errors
    ///
    /// A sidereal bit with an ayanamsha id the catalogue does not have, or
    /// a reserved bit set.
    pub fn try_from_bits(bits: u32) -> Result<Frame, ProviderError> {
        const RESERVED: u32 = 0b1111_1110_0000_0000;
        if bits & RESERVED != 0 {
            return Err(ProviderError::invalid(format!(
                "frame bits {bits:#x} set a reserved bit"
            )));
        }
        let centre = match bits & 0b11 {
            1 => Centre::Topocentric,
            2 => Centre::Heliocentric,
            3 => Centre::Barycentric,
            _ => Centre::Geocentric,
        };
        let flag = |bit: u32| bits & (1 << bit) != 0;
        let zodiac = if flag(8) {
            let id = u16::try_from(bits >> 16).unwrap_or(u16::MAX);
            let ayanamsha = Ayanamsha::from_id(id).ok_or_else(|| {
                ProviderError::invalid(format!(
                    "frame bits name ayanamsha id {id}, which the catalogue does not have"
                ))
            })?;
            Zodiac::Sidereal { ayanamsha }
        } else {
            Zodiac::Tropical
        };
        Ok(Frame {
            centre,
            equinox: if flag(2) {
                Equinox::J2000
            } else {
                Equinox::OfDate
            },
            coordinates: if flag(3) {
                Coordinates::Equatorial
            } else {
                Coordinates::Ecliptic
            },
            zodiac,
            corrections: Corrections {
                light_time: flag(4),
                aberration: flag(5),
                deflection: flag(6),
                nutation: flag(7),
            },
        })
    }

    /// The key stamped in provenance: centre, equinox, coordinates,
    /// zodiac and corrections, slash-separated.
    #[must_use]
    pub fn key(&self) -> String {
        format!(
            "{}/{}/{}/{}/{}",
            self.centre.key(),
            self.equinox.key(),
            self.coordinates.key(),
            self.zodiac,
            self.corrections.key()
        )
    }
}

impl fmt::Display for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.key())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    #[test]
    fn frames_round_trip_through_bits() {
        let frames = [
            Frame::CANONICAL,
            Frame::CANONICAL.with_coordinates(Coordinates::Equatorial),
            Frame::CANONICAL.with_zodiac(Zodiac::sidereal(Ayanamsha::TrueChitra)),
            Frame {
                centre: Centre::Topocentric,
                equinox: Equinox::J2000,
                coordinates: Coordinates::Equatorial,
                zodiac: Zodiac::Tropical,
                corrections: Corrections::GEOMETRIC,
            },
            Frame::CANONICAL.with_centre(Centre::Barycentric),
        ];
        for frame in frames {
            assert_eq!(Frame::try_from_bits(frame.to_bits()), Ok(frame));
        }
        assert!(Frame::try_from_bits(1 << 9).is_err());
        assert!(Frame::try_from_bits((1 << 8) | (60_000 << 16)).is_err());
        assert_eq!(Frame::CANONICAL.corrections.key(), "APPARENT");
        assert_eq!(
            Corrections {
                nutation: false,
                ..Corrections::APPARENT
            }
            .key(),
            "light_time+aberration+deflection"
        );
        assert_eq!(
            Frame::CANONICAL.to_string(),
            "GEOCENTRIC/OF_DATE/ECLIPTIC/TROPICAL/APPARENT"
        );
        assert_eq!(
            Zodiac::sidereal(Ayanamsha::Lahiri).ayanamsha(),
            Some(Ayanamsha::Lahiri)
        );
        let json = serde_json::to_string(
            &Frame::CANONICAL.with_zodiac(Zodiac::sidereal(Ayanamsha::Raman)),
        )
        .unwrap();
        assert!(json.contains("\"ayanamsha\":\"RAMAN\""), "{json}");
    }
}
