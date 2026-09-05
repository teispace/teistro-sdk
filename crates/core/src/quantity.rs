//! Validated newtypes for every domain quantity (ADR-0023): parse once at
//! the boundary, trust inside. No bare primitive crosses a public
//! signature above the C ABI, and there is no `From<f64>` anywhere here.
//!
//! ```
//! use teistro_core::quantity::{Latitude, Longitude, Altitude, Place};
//!
//! let kathmandu = Place::new(
//!     Latitude::try_new(27.7172).expect("in range"),
//!     Longitude::try_new(85.324).expect("in range"),
//!     Altitude::try_new(1400.0).expect("in range"),
//! );
//! assert_eq!(kathmandu.to_string(), "27.7172°N 85.324°E 1400 m");
//!
//! let wrong = Latitude::try_new(95.0).unwrap_err();
//! assert_eq!(wrong.to_string(), "latitude 95 is outside -90 to 90 degrees");
//! ```

use core::fmt;
use core::marker::PhantomData;

/// A value refused at construction: the quantity, the value, the accepted
/// range, and the field it came from when the caller says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvalidValue {
    /// The quantity's name (`latitude`).
    pub quantity: &'static str,
    /// The value, rendered.
    pub value: String,
    /// What is accepted (`-90 to 90 degrees`).
    pub accepted: &'static str,
    /// The field the value came from, when known.
    pub field: Option<String>,
}

impl InvalidValue {
    /// Names the field the value came from.
    #[must_use]
    pub fn with_field(mut self, field: &str) -> InvalidValue {
        self.field = Some(field.to_string());
        self
    }
}

impl fmt::Display for InvalidValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} is outside {}",
            self.quantity, self.value, self.accepted
        )?;
        if let Some(field) = &self.field {
            write!(f, " (field `{field}`)")?;
        }
        Ok(())
    }
}

impl std::error::Error for InvalidValue {}

macro_rules! bounded_float {
    ($(#[$doc:meta])* $name:ident, $quantity:literal, $min:expr, $max:expr, $accepted:literal, $unit:literal) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, serde::Serialize)]
        #[serde(transparent)]
        pub struct $name(f64);

        impl $name {
            /// The smallest accepted value.
            pub const MIN: $name = $name($min);
            /// The largest accepted value.
            pub const MAX: $name = $name($max);
            /// What the quantity is called in messages.
            pub const NAME: &'static str = $quantity;

            /// Accepts a finite value inside the range.
            ///
            /// # Errors
            ///
            /// A NaN, infinite or out-of-range value.
            pub fn try_new(value: f64) -> Result<$name, InvalidValue> {
                if value.is_finite() && ($min..=$max).contains(&value) {
                    Ok($name(value))
                } else {
                    Err(InvalidValue {
                        quantity: $quantity,
                        value: value.to_string(),
                        accepted: $accepted,
                        field: None,
                    })
                }
            }

            /// A value written as a literal in a constant.
            ///
            /// # Panics
            ///
            /// When the value is outside the range; in a constant that is a
            /// compile error, which is what the constructor is for. Runtime
            /// values go through `try_new`.
            #[must_use]
            pub const fn literal(value: f64) -> $name {
                assert!(
                    value >= $min && value <= $max,
                    concat!(stringify!($name), " literal outside ", $accepted)
                );
                $name(value)
            }

            /// The value.
            #[must_use]
            pub const fn get(self) -> f64 {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}{}", self.0, $unit)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<$name, D::Error> {
                let value = f64::deserialize(deserializer)?;
                $name::try_new(value).map_err(serde::de::Error::custom)
            }
        }
    };
}

macro_rules! bounded_int {
    ($(#[$doc:meta])* $name:ident, $inner:ty, $quantity:literal, $min:expr, $max:expr, $accepted:literal) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
        #[serde(transparent)]
        pub struct $name($inner);

        impl $name {
            /// The smallest accepted value.
            pub const MIN: $name = $name($min);
            /// The largest accepted value.
            pub const MAX: $name = $name($max);
            /// What the quantity is called in messages.
            pub const NAME: &'static str = $quantity;

            /// Accepts a value inside the range.
            ///
            /// # Errors
            ///
            /// An out-of-range value.
            pub fn try_new(value: $inner) -> Result<$name, InvalidValue> {
                if ($min..=$max).contains(&value) {
                    Ok($name(value))
                } else {
                    Err(InvalidValue {
                        quantity: $quantity,
                        value: value.to_string(),
                        accepted: $accepted,
                        field: None,
                    })
                }
            }

            /// For generated tables and classification, where the range is
            /// guaranteed by construction.
            #[must_use]
            #[allow(dead_code, reason = "used by the kinds classification produces")]
            pub(crate) const fn new_unchecked(value: $inner) -> $name {
                $name(value)
            }

            /// The value.
            #[must_use]
            pub const fn get(self) -> $inner {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<$name, D::Error> {
                let value = <$inner>::deserialize(deserializer)?;
                $name::try_new(value).map_err(serde::de::Error::custom)
            }
        }
    };
}

bounded_float!(
    /// A geographic latitude in degrees, north positive.
    Latitude, "latitude", -90.0, 90.0, "-90 to 90 degrees", "°"
);
bounded_float!(
    /// A geographic longitude in degrees, east positive.
    Longitude, "longitude", -180.0, 180.0, "-180 to 180 degrees", "°"
);
bounded_float!(
    /// An altitude above sea level in metres.
    Altitude, "altitude", -500.0, 12_000.0, "-500 to 12000 metres", " m"
);
bounded_float!(
    /// A finite angle in degrees, of any range; normalisation is the
    /// canonical angle's job.
    Degrees, "angle", f64::NEG_INFINITY, f64::INFINITY, "a finite number of degrees", "°"
);

bounded_int!(
    /// A sign index, 0 for Aries to 11 for Pisces.
    SignIndex, u8, "sign index", 0, 11, "0 to 11"
);
bounded_int!(
    /// A house number, 1 to 12.
    HouseNumber, u8, "house number", 1, 12, "1 to 12"
);
bounded_int!(
    /// A nakshatra index, 0 for Ashwini to 26 for Revati.
    NakshatraIndex, u8, "nakshatra index", 0, 26, "0 to 26"
);
bounded_int!(
    /// A pada index inside a nakshatra, 0 to 3.
    PadaIndex, u8, "pada index", 0, 3, "0 to 3"
);
bounded_int!(
    /// A tithi index, 0 for Shukla Pratipada to 29 for Amavasya.
    TithiIndex, u8, "tithi index", 0, 29, "0 to 29"
);
bounded_int!(
    /// The divisions of a varga, 1 to 300.
    VargaDivisions, u16, "varga divisions", 1, 300, "1 to 300"
);
bounded_int!(
    /// A dasha depth, 1 to 6 levels.
    Depth, u8, "depth", 1, 6, "1 to 6"
);

impl HouseNumber {
    /// The house as a 0-based offset from the first.
    #[must_use]
    pub const fn offset(self) -> u8 {
        self.0 - 1
    }
}

/// A place on Earth.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Place {
    /// North positive.
    pub latitude: Latitude,
    /// East positive.
    pub longitude: Longitude,
    /// Above sea level.
    pub altitude: Altitude,
}

impl Place {
    /// A place from validated parts; the types make a swapped latitude and
    /// longitude a compile error.
    #[must_use]
    pub const fn new(latitude: Latitude, longitude: Longitude, altitude: Altitude) -> Place {
        Place {
            latitude,
            longitude,
            altitude,
        }
    }

    /// A place from bare degrees and metres, each named in its error.
    ///
    /// # Errors
    ///
    /// The first part outside its range, naming the field.
    pub fn try_from_degrees(lat_deg: f64, lon_deg: f64, alt_m: f64) -> Result<Place, InvalidValue> {
        Ok(Place {
            latitude: Latitude::try_new(lat_deg).map_err(|e| e.with_field("lat_deg"))?,
            longitude: Longitude::try_new(lon_deg).map_err(|e| e.with_field("lon_deg"))?,
            altitude: Altitude::try_new(alt_m).map_err(|e| e.with_field("alt_m"))?,
        })
    }
}

impl fmt::Display for Place {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lat = self.latitude.get();
        let lon = self.longitude.get();
        write!(
            f,
            "{}°{} {}°{} {} m",
            lat.abs(),
            if lat < 0.0 { "S" } else { "N" },
            lon.abs(),
            if lon < 0.0 { "W" } else { "E" },
            self.altitude.get()
        )
    }
}

mod sealed {
    pub trait Sealed {}
}

/// A time scale, as a type so a UT1 instant cannot be passed as TT.
pub trait Scale: sealed::Sealed + Copy + fmt::Debug + Default + 'static {
    /// The scale's name in stamps and field names.
    const NAME: &'static str;
}

/// Universal Time, the scale of civil time and of rise and set.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Ut1;
/// Terrestrial Time, the scale of the ephemerides.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Tt;
/// Coordinated Universal Time, the scale of clocks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Utc;

impl sealed::Sealed for Ut1 {}
impl sealed::Sealed for Tt {}
impl sealed::Sealed for Utc {}

impl Scale for Ut1 {
    const NAME: &'static str = "UT1";
}
impl Scale for Tt {
    const NAME: &'static str = "TT";
}
impl Scale for Utc {
    const NAME: &'static str = "UTC";
}

/// A Julian day on one time scale.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, serde::Serialize)]
#[serde(transparent)]
pub struct JulianDay<S: Scale> {
    jd: f64,
    #[serde(skip)]
    scale: PhantomData<S>,
}

impl<S: Scale> JulianDay<S> {
    /// J2000.0, 2000 January 1.5.
    pub const J2000: JulianDay<S> = JulianDay {
        jd: 2_451_545.0,
        scale: PhantomData,
    };

    /// Accepts a finite Julian day.
    ///
    /// # Errors
    ///
    /// A NaN or infinite value.
    pub fn try_new(jd: f64) -> Result<JulianDay<S>, InvalidValue> {
        if jd.is_finite() {
            Ok(JulianDay {
                jd,
                scale: PhantomData,
            })
        } else {
            Err(InvalidValue {
                quantity: "julian day",
                value: jd.to_string(),
                accepted: "a finite number of days",
                field: None,
            })
        }
    }

    /// A Julian day written as a literal in a constant.
    ///
    /// # Panics
    ///
    /// When the value is not finite; in a constant that is a compile
    /// error, which is what the constructor is for. Runtime values go
    /// through [`JulianDay::try_new`].
    #[must_use]
    pub const fn literal(jd: f64) -> JulianDay<S> {
        assert!(jd.is_finite(), "a Julian day literal must be finite");
        JulianDay {
            jd,
            scale: PhantomData,
        }
    }

    /// The value in days.
    #[must_use]
    pub const fn get(self) -> f64 {
        self.jd
    }

    /// The same instant shifted by `days`; a non-finite result is refused.
    ///
    /// # Errors
    ///
    /// A non-finite result.
    pub fn plus_days(self, days: f64) -> Result<JulianDay<S>, InvalidValue> {
        JulianDay::try_new(self.jd + days)
    }

    /// The whole day and the fraction since its start at noon, for
    /// sub-millisecond arithmetic.
    #[must_use]
    pub fn split(self) -> (f64, f64) {
        let day = self.jd.floor();
        (day, self.jd - day)
    }

    /// Reinterprets the value on another scale after a conversion has been
    /// applied by the caller; the conversion functions live in `time`.
    #[must_use]
    pub const fn relabel<T: Scale>(self) -> JulianDay<T> {
        JulianDay {
            jd: self.jd,
            scale: PhantomData,
        }
    }
}

impl<S: Scale> fmt::Display for JulianDay<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JD {} {}", self.jd, S::NAME)
    }
}

impl<'de, S: Scale> serde::Deserialize<'de> for JulianDay<S> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<JulianDay<S>, D::Error> {
        let jd = f64::deserialize(deserializer)?;
        JulianDay::try_new(jd).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use super::*;

    #[test]
    fn ranges_are_enforced_with_named_fields() {
        assert!(Latitude::try_new(90.0).is_ok());
        assert!(Latitude::try_new(90.000_001).is_err());
        assert!(Latitude::try_new(f64::NAN).is_err());
        assert!(Longitude::try_new(-180.0).is_ok());
        assert!(Altitude::try_new(-501.0).is_err());
        assert!(Degrees::try_new(1e300).is_ok());
        assert!(Degrees::try_new(f64::INFINITY).is_err());
        let error = Place::try_from_degrees(27.7, 190.0, 0.0).unwrap_err();
        assert_eq!(
            error.to_string(),
            "longitude 190 is outside -180 to 180 degrees (field `lon_deg`)"
        );
        assert_eq!(
            HouseNumber::try_new(0).unwrap_err().to_string(),
            "house number 0 is outside 1 to 12"
        );
        assert_eq!(HouseNumber::try_new(7).map(HouseNumber::offset), Ok(6));
        assert_eq!(VargaDivisions::MAX.get(), 300);
        assert_eq!(Depth::MIN.get(), 1);
    }

    #[test]
    fn julian_days_carry_their_scale() {
        let tt = JulianDay::<Tt>::J2000;
        assert_eq!(tt.to_string(), "JD 2451545 TT");
        let later = tt.plus_days(0.5).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(later.split(), (2_451_545.0, 0.5));
        let ut1: JulianDay<Ut1> = later.relabel();
        assert_eq!(ut1.get().to_bits(), 2_451_545.5f64.to_bits());
        assert!(JulianDay::<Utc>::try_new(f64::NAN).is_err());
        let json = serde_json::to_string(&tt).unwrap_or_default();
        assert_eq!(json, "2451545.0");
        let place = Place::try_from_degrees(-33.9, 151.2, 5.0).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(place.to_string(), "33.9°S 151.2°E 5 m");
        let back: Place = serde_json::from_str(&serde_json::to_string(&place).unwrap_or_default())
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(back, place);
        assert!(serde_json::from_str::<Latitude>("91").is_err());
    }
}
