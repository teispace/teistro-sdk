//! The canonical angle: an `i64` count of nanoarcseconds in `[0, CIRCLE)`,
//! and the exact classification of a longitude into signs, nakshatras,
//! padas and varga parts by integer arithmetic (ADR-0016,
//! `docs/03-design/exact-arithmetic.md`). Floating point enters once, in
//! [`Nas::from_degrees`], and leaves for presentation only.
//!
//! ```
//! use teistro_core::angle::Nas;
//! use teistro_core::catalogue::{Nakshatra, Rashi};
//! use teistro_core::quantity::Degrees;
//!
//! let exactly_thirty = Nas::from_degrees(Degrees::try_new(30.0).expect("finite"));
//! assert_eq!(exactly_thirty.sign(), Rashi::Taurus);          // lower-inclusive boundaries
//! assert_eq!(exactly_thirty.nakshatra(), Nakshatra::Krittika);
//! assert_eq!(exactly_thirty.pada().get(), 1);
//! assert_eq!(exactly_thirty.to_string(), "30°00′00.000000000″");
//! ```

use core::fmt;
use core::ops::{Add, Sub};

use crate::catalogue::{Nakshatra, Rashi};
use crate::quantity::{Degrees, InvalidValue, NakshatraIndex, PadaIndex, SignIndex};

/// Degrees into `[0, 360)`: the floating-point companion of [`Nas::new`]
/// for values that stay floating (a provider's longitude, a solver's
/// target) before they become canonical.
///
/// ```
/// use teistro_core::angle::normalise_deg;
///
/// assert_eq!(normalise_deg(370.0), 10.0);
/// assert_eq!(normalise_deg(-90.0), 270.0);
/// assert_eq!(normalise_deg(360.0), 0.0);
/// ```
#[must_use]
pub fn normalise_deg(deg: f64) -> f64 {
    let wrapped = deg.rem_euclid(360.0);
    if wrapped >= 360.0 { 0.0 } else { wrapped }
}

/// The smaller signed difference `a - b` of two angles in degrees, in
/// `(-180, 180]`: the floating-point companion of
/// [`Nas::signed_difference`].
///
/// ```
/// use teistro_core::angle::difference_deg;
///
/// assert_eq!(difference_deg(10.0, 350.0), 20.0);
/// assert_eq!(difference_deg(350.0, 10.0), -20.0);
/// assert_eq!(difference_deg(180.0, 0.0), 180.0);
/// ```
#[must_use]
pub fn difference_deg(a: f64, b: f64) -> f64 {
    let d = (a - b).rem_euclid(360.0);
    if d > 180.0 { d - 360.0 } else { d }
}

/// A canonical angle in nanoarcseconds, `0 ..< CIRCLE`. Exact, ordered,
/// hashable; the value every classification reads.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Nas(i64);

impl Nas {
    /// One circle: 360 × 3600 × 10⁹.
    pub const CIRCLE: i64 = 1_296_000_000_000_000;
    /// One degree.
    pub const PER_DEGREE: i64 = 3_600_000_000_000;
    /// One arcminute.
    pub const PER_ARCMINUTE: i64 = 60_000_000_000;
    /// One arcsecond.
    pub const PER_ARCSECOND: i64 = 1_000_000_000;
    /// One sign, thirty degrees.
    pub const PER_SIGN: i64 = Nas::CIRCLE / 12;
    /// One nakshatra, 13°20′.
    pub const PER_NAKSHATRA: i64 = Nas::CIRCLE / 27;
    /// Zero.
    pub const ZERO: Nas = Nas(0);

    /// From a raw count, normalised into the circle.
    #[must_use]
    pub const fn new(nas: i64) -> Nas {
        Nas(nas.rem_euclid(Nas::CIRCLE))
    }

    /// The raw count.
    #[must_use]
    pub const fn get(self) -> i64 {
        self.0
    }

    /// The one conversion from floating point in the workspace: normalise
    /// to `[0, 360)`, scale, round half to even.
    #[must_use]
    pub fn from_degrees(degrees: Degrees) -> Nas {
        let normalised = degrees.get().rem_euclid(360.0);
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_precision_loss,
            reason = "the product is below 1.3e15 and rounded to an integer"
        )]
        let scaled = (normalised * Nas::PER_DEGREE as f64).round_ties_even() as i64;
        Nas::new(scaled)
    }

    /// From a bare `f64`, refusing a non-finite value.
    ///
    /// # Errors
    ///
    /// A NaN or infinite input.
    pub fn try_from_degrees(degrees: f64) -> Result<Nas, InvalidValue> {
        Degrees::try_new(degrees).map(Nas::from_degrees)
    }

    /// Decimal degrees, for presentation and for feeding a value back into
    /// floating-point astronomy, never for classification.
    #[must_use]
    #[allow(clippy::cast_precision_loss, reason = "a presentation value")]
    pub fn to_degrees(self) -> f64 {
        self.0 as f64 / Nas::PER_DEGREE as f64
    }

    /// The index of the division this angle falls in when the circle is
    /// cut into `divisions` equal parts. Exact for any divisor: the product
    /// is taken in `i128` and boundaries are lower-inclusive.
    #[must_use]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "the quotient is below `divisions`"
    )]
    pub const fn division_index(self, divisions: u32) -> u32 {
        ((self.0 as i128 * divisions as i128) / Nas::CIRCLE as i128) as u32
    }

    /// The sign.
    #[must_use]
    pub fn sign(self) -> Rashi {
        Rashi::from_id(self.sign_index().get().into()).unwrap_or(Rashi::Aries)
    }

    /// The sign as an index, 0 for Aries.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, reason = "below twelve")]
    pub const fn sign_index(self) -> SignIndex {
        SignIndex::new_unchecked(self.division_index(12) as u8)
    }

    /// The nakshatra in the 27-scheme.
    #[must_use]
    pub fn nakshatra(self) -> Nakshatra {
        Nakshatra::from_id(self.nakshatra_index().get().into()).unwrap_or(Nakshatra::Ashwini)
    }

    /// The nakshatra as an index, 0 for Ashwini.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, reason = "below twenty-seven")]
    pub const fn nakshatra_index(self) -> NakshatraIndex {
        NakshatraIndex::new_unchecked(self.division_index(27) as u8)
    }

    /// The pada inside the nakshatra, 0 to 3.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, reason = "below four")]
    pub const fn pada(self) -> PadaIndex {
        PadaIndex::new_unchecked((self.division_index(108) % 4) as u8)
    }

    /// The pada counted around the whole circle, 0 to 107.
    #[must_use]
    pub const fn pada_global(self) -> u32 {
        self.division_index(108)
    }

    /// The angle inside its sign.
    #[must_use]
    pub const fn in_sign(self) -> Nas {
        Nas(self.0 % Nas::PER_SIGN)
    }

    /// The angle inside its nakshatra.
    #[must_use]
    pub const fn in_nakshatra(self) -> Nas {
        Nas(self.0 % Nas::PER_NAKSHATRA)
    }

    /// The part index inside the sign when the sign is cut into `n` equal
    /// parts, for a varga of equal spans.
    #[must_use]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "the value is non-negative and the quotient is below `n`"
    )]
    pub const fn part(self, n: u32) -> u32 {
        ((self.in_sign().0 as i128 * n as i128) / Nas::PER_SIGN as i128) as u32
    }

    /// The forward arc from `self` to `other`, in `[0, CIRCLE)`.
    #[must_use]
    pub const fn arc_to(self, other: Nas) -> Nas {
        Nas::new(other.0 - self.0)
    }

    /// The signed shortest difference `other − self`, in
    /// `(−CIRCLE/2, CIRCLE/2]` nanoarcseconds.
    #[must_use]
    pub const fn signed_difference(self, other: Nas) -> i64 {
        let forward = self.arc_to(other).0;
        if forward > Nas::CIRCLE / 2 {
            forward - Nas::CIRCLE
        } else {
            forward
        }
    }

    /// Degrees, minutes, seconds and nanoseconds of arc, exactly.
    #[must_use]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "each part is bounded by its divisor"
    )]
    pub const fn dms(self) -> Dms {
        Dms {
            degrees: (self.0 / Nas::PER_DEGREE) as u16,
            minutes: ((self.0 % Nas::PER_DEGREE) / Nas::PER_ARCMINUTE) as u8,
            seconds: ((self.0 % Nas::PER_ARCMINUTE) / Nas::PER_ARCSECOND) as u8,
            nanos: (self.0 % Nas::PER_ARCSECOND) as u32,
        }
    }
}

/// An angle split into its sexagesimal parts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Dms {
    /// Whole degrees, 0 to 359.
    pub degrees: u16,
    /// Arcminutes.
    pub minutes: u8,
    /// Whole arcseconds.
    pub seconds: u8,
    /// Nanoarcseconds inside the second.
    pub nanos: u32,
}

impl fmt::Display for Nas {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let d = self.dms();
        write!(
            f,
            "{}°{:02}′{:02}.{:09}″",
            d.degrees, d.minutes, d.seconds, d.nanos
        )
    }
}

impl fmt::Debug for Nas {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Nas({} = {self})", self.0)
    }
}

impl Add for Nas {
    type Output = Nas;

    fn add(self, other: Nas) -> Nas {
        Nas::new(self.0 + other.0)
    }
}

impl Sub for Nas {
    type Output = Nas;

    fn sub(self, other: Nas) -> Nas {
        Nas::new(self.0 - other.0)
    }
}

impl serde::Serialize for Nas {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_i64(self.0)
    }
}

impl<'de> serde::Deserialize<'de> for Nas {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Nas, D::Error> {
        let raw = i64::deserialize(deserializer)?;
        if (0..Nas::CIRCLE).contains(&raw) {
            Ok(Nas(raw))
        } else {
            Err(serde::de::Error::custom(format!(
                "an angle in nanoarcseconds is 0 to {}, not {raw}",
                Nas::CIRCLE - 1
            )))
        }
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

    use proptest::prelude::*;

    use super::*;

    fn deg(value: f64) -> Nas {
        Nas::try_from_degrees(value).unwrap_or_else(|e| panic!("{e}"))
    }

    /// `ceil(a / b)` for non-negative `a` and positive `b`.
    fn ceil_div(a: i128, b: i128) -> i128 {
        (a + b - 1).div_euclid(b)
    }

    #[test]
    fn boundaries_are_lower_inclusive() {
        assert_eq!(deg(30.0).sign(), Rashi::Taurus);
        assert_eq!(Nas::new(Nas::PER_SIGN - 1).sign(), Rashi::Aries);
        assert_eq!(deg(13.0 + 20.0 / 60.0).nakshatra(), Nakshatra::Bharani);
        assert_eq!(
            Nas::new(Nas::PER_NAKSHATRA - 1).nakshatra(),
            Nakshatra::Ashwini
        );
        assert_eq!(deg(359.999_999_999_9).sign(), Rashi::Pisces);
        assert_eq!(deg(360.0), Nas::ZERO);
        assert_eq!(deg(-30.0), deg(330.0));
    }

    #[test]
    fn rounding_is_half_to_even_and_lossless_at_f64_resolution() {
        assert_eq!(
            Nas::from_degrees(Degrees::try_new(0.0).unwrap_or_default()),
            Nas::ZERO
        );
        let one_nas = Nas::new(1).to_degrees();
        assert_eq!(deg(one_nas), Nas::new(1));
        let big = Nas::new(Nas::CIRCLE - 1);
        assert_eq!(deg(big.to_degrees()), big);
    }

    #[test]
    fn parts_and_arcs() {
        let v = deg(222.5763);
        assert_eq!(v.sign(), Rashi::Scorpio);
        assert!((v.in_sign().to_degrees() - 12.5763).abs() < 1e-9);
        assert_eq!(v.part(9), 3);
        assert_eq!(v.pada_global(), 66);
        assert_eq!(deg(10.0).arc_to(deg(350.0)), deg(340.0));
        assert_eq!(
            deg(10.0).signed_difference(deg(350.0)),
            -20 * Nas::PER_DEGREE
        );
        assert_eq!(
            deg(350.0).signed_difference(deg(10.0)),
            20 * Nas::PER_DEGREE
        );
        assert_eq!(deg(0.0).signed_difference(deg(180.0)), Nas::CIRCLE / 2);
        assert_eq!(deg(350.0) + deg(20.0), deg(10.0));
        assert_eq!(deg(10.0) - deg(20.0), deg(350.0));
    }

    #[test]
    fn display_and_serde() {
        assert_eq!(deg(222.5763).to_string(), "222°34′34.680000000″");
        let json = serde_json::to_string(&deg(30.0)).unwrap_or_default();
        assert_eq!(json, "108000000000000");
        let back: Nas = serde_json::from_str(&json).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(back, deg(30.0));
        assert!(serde_json::from_str::<Nas>("-1").is_err());
        assert!(Nas::try_from_degrees(f64::NAN).is_err());
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2000))]

        #[test]
        fn every_divisor_partitions_the_circle(divisions in 1u32..=360, raw in 0i64..Nas::CIRCLE) {
            let nas = Nas::new(raw);
            let index = nas.division_index(divisions);
            prop_assert!(index < divisions);
            // The division's own lower boundary maps to the same index, and the
            // last value before the next boundary too.
            let width = i128::from(Nas::CIRCLE);
            let lower = ceil_div(i128::from(index) * width, i128::from(divisions));
            let upper = ceil_div((i128::from(index) + 1) * width, i128::from(divisions)) - 1;
            prop_assert!(i128::from(raw) >= lower && i128::from(raw) <= upper);
            prop_assert_eq!(Nas::new(i64::try_from(lower).unwrap_or(0)).division_index(divisions), index);
            prop_assert_eq!(Nas::new(i64::try_from(upper).unwrap_or(0)).division_index(divisions), index);
        }

        #[test]
        fn degrees_round_trip_within_one_nanoarcsecond(value in -720.0f64..720.0) {
            let nas = Nas::try_from_degrees(value).unwrap_or_default();
            let again = Nas::try_from_degrees(nas.to_degrees()).unwrap_or_default();
            prop_assert!((again.get() - nas.get()).abs() <= 1);
        }

        #[test]
        fn a_value_just_below_a_boundary_stays_below(
            (divisions, index) in (2u32..=360).prop_flat_map(|d| (Just(d), 1u32..d))
        ) {
            let width = i128::from(Nas::CIRCLE);
            let boundary = ceil_div(i128::from(index) * width, i128::from(divisions));
            let below = Nas::new(i64::try_from(boundary - 1).unwrap_or(0));
            prop_assert_eq!(below.division_index(divisions), index - 1);
            let at = Nas::new(i64::try_from(boundary).unwrap_or(0));
            prop_assert_eq!(at.division_index(divisions), index);
        }
    }
}
