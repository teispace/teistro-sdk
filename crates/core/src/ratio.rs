//! Exact rationals over `i128` in lowest terms, bounded so that comparing
//! two of them by cross-multiplication can never overflow: dasha spans as
//! fractions of a parent span, materialised to an instant once, at
//! presentation (ADR-0016, `docs/03-design/exact-arithmetic.md`).
//!
//! ```
//! use teistro_core::ratio::Ratio;
//!
//! let a = Ratio::new(1, 3).expect("in range");
//! let b = Ratio::new(2, 6).expect("in range");
//! assert_eq!(a, b);                       // lowest terms
//! let sum = a.checked_add(b).expect("no overflow");
//! assert_eq!(sum, Ratio::new(2, 3).expect("in range"));
//! assert!(a < sum);
//! assert_eq!(sum.to_string(), "2/3");
//! ```

use core::cmp::Ordering;
use core::fmt;

/// A rational number `num / den` with `den > 0`, in lowest terms, both
/// parts below [`Ratio::LIMIT`] in magnitude.
///
/// ```
/// use teistro_core::ratio::Ratio;
///
/// // Sun's Vimshottari dasha: 6 of the cycle's 120 years.
/// let sun = Ratio::new(6, 120)?;
/// assert_eq!(sun.to_string(), "1/20");
/// assert!(sun < Ratio::new(7, 120)?);
/// # Ok::<(), teistro_core::ratio::RatioError>(())
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ratio {
    num: i128,
    den: i128,
}

/// Why a rational could not be made.
///
/// ```
/// use teistro_core::ratio::{Ratio, RatioError};
///
/// assert_eq!(Ratio::new(1, 0), Err(RatioError::ZeroDenominator));
/// assert_eq!(Ratio::new(Ratio::LIMIT, 1), Err(RatioError::Overflow));
/// assert_eq!(RatioError::Overflow.to_string(), "the rational exceeds the exact range");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RatioError {
    /// A zero denominator.
    ZeroDenominator,
    /// A part beyond [`Ratio::LIMIT`] after reduction.
    Overflow,
}

impl fmt::Display for RatioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            RatioError::ZeroDenominator => "a rational needs a non-zero denominator",
            RatioError::Overflow => "the rational exceeds the exact range",
        })
    }
}

impl std::error::Error for RatioError {}

const fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}

impl Ratio {
    /// The magnitude bound on numerator and denominator: 2⁶², so that a
    /// product of two parts fits `i128` with room to spare.
    pub const LIMIT: i128 = 1 << 62;
    /// Zero.
    pub const ZERO: Ratio = Ratio { num: 0, den: 1 };
    /// One.
    pub const ONE: Ratio = Ratio { num: 1, den: 1 };

    /// `num / den` reduced to lowest terms with a positive denominator.
    ///
    /// ```
    /// use teistro_core::ratio::Ratio;
    ///
    /// // Lowest terms, the sign on the numerator.
    /// let half = Ratio::new(3, -6)?;
    /// assert_eq!((half.numerator(), half.denominator()), (-1, 2));
    /// # Ok::<(), teistro_core::ratio::RatioError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A zero denominator, or parts beyond the bound after reduction.
    pub const fn new(num: i128, den: i128) -> Result<Ratio, RatioError> {
        if den == 0 {
            return Err(RatioError::ZeroDenominator);
        }
        let (num, den) = if den < 0 { (-num, -den) } else { (num, den) };
        let g = gcd(num.unsigned_abs(), den.unsigned_abs());
        #[allow(
            clippy::cast_possible_wrap,
            reason = "a divisor of two i128 magnitudes fits i128"
        )]
        let g = if g == 0 { 1 } else { g as i128 };
        let (num, den) = (num / g, den / g);
        if num.abs() >= Ratio::LIMIT || den >= Ratio::LIMIT {
            return Err(RatioError::Overflow);
        }
        Ok(Ratio { num, den })
    }

    /// A whole number.
    ///
    /// ```
    /// use teistro_core::ratio::Ratio;
    ///
    /// assert_eq!(Ratio::from_integer(120), Ratio::new(240, 2)?);
    /// # Ok::<(), teistro_core::ratio::RatioError>(())
    /// ```
    #[must_use]
    pub const fn from_integer(n: i64) -> Ratio {
        Ratio {
            num: n as i128,
            den: 1,
        }
    }

    /// The numerator.
    ///
    /// ```
    /// use teistro_core::ratio::Ratio;
    ///
    /// assert_eq!(Ratio::new(10, 4)?.numerator(), 5);
    /// # Ok::<(), teistro_core::ratio::RatioError>(())
    /// ```
    #[must_use]
    pub const fn numerator(self) -> i128 {
        self.num
    }

    /// The denominator, always positive.
    ///
    /// ```
    /// use teistro_core::ratio::Ratio;
    ///
    /// assert_eq!(Ratio::new(10, -4)?.denominator(), 2);
    /// # Ok::<(), teistro_core::ratio::RatioError>(())
    /// ```
    #[must_use]
    pub const fn denominator(self) -> i128 {
        self.den
    }

    /// Whether the value is zero.
    ///
    /// ```
    /// use teistro_core::ratio::Ratio;
    ///
    /// assert!(Ratio::new(0, 7)?.is_zero());
    /// assert_eq!(Ratio::new(0, 7)?, Ratio::ZERO);
    /// # Ok::<(), teistro_core::ratio::RatioError>(())
    /// ```
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.num == 0
    }

    /// `self + other`.
    ///
    /// ```
    /// use teistro_core::ratio::Ratio;
    ///
    /// let third = Ratio::new(1, 3)?;
    /// assert_eq!(third.checked_add(third)?.checked_add(third)?, Ratio::ONE);
    /// # Ok::<(), teistro_core::ratio::RatioError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Beyond the exact range.
    pub const fn checked_add(self, other: Ratio) -> Result<Ratio, RatioError> {
        Ratio::new(
            self.num * other.den + other.num * self.den,
            self.den * other.den,
        )
    }

    /// `self − other`.
    ///
    /// ```
    /// use teistro_core::ratio::Ratio;
    ///
    /// assert_eq!(Ratio::ONE.checked_sub(Ratio::new(1, 20)?)?, Ratio::new(19, 20)?);
    /// # Ok::<(), teistro_core::ratio::RatioError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Beyond the exact range.
    pub const fn checked_sub(self, other: Ratio) -> Result<Ratio, RatioError> {
        Ratio::new(
            self.num * other.den - other.num * self.den,
            self.den * other.den,
        )
    }

    /// `self × other`.
    ///
    /// ```
    /// use teistro_core::ratio::Ratio;
    ///
    /// // The Moon's antardasha inside Sun's mahadasha: 10/120 of 6/120.
    /// let sub = Ratio::new(6, 120)?.checked_mul(Ratio::new(10, 120)?)?;
    /// assert_eq!(sub, Ratio::new(1, 240)?);
    /// # Ok::<(), teistro_core::ratio::RatioError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Beyond the exact range.
    pub const fn checked_mul(self, other: Ratio) -> Result<Ratio, RatioError> {
        Ratio::new(self.num * other.num, self.den * other.den)
    }

    /// `self ÷ other`.
    ///
    /// ```
    /// use teistro_core::ratio::{Ratio, RatioError};
    ///
    /// assert_eq!(Ratio::ONE.checked_div(Ratio::new(1, 4)?)?, Ratio::from_integer(4));
    /// assert_eq!(Ratio::ONE.checked_div(Ratio::ZERO), Err(RatioError::ZeroDenominator));
    /// # Ok::<(), teistro_core::ratio::RatioError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Division by zero, or beyond the exact range.
    pub const fn checked_div(self, other: Ratio) -> Result<Ratio, RatioError> {
        if other.num == 0 {
            return Err(RatioError::ZeroDenominator);
        }
        Ratio::new(self.num * other.den, self.den * other.num)
    }

    /// The value as `f64`, for presentation only.
    ///
    /// ```
    /// use teistro_core::ratio::Ratio;
    ///
    /// assert_eq!(Ratio::new(1, 4)?.to_f64(), 0.25);
    /// # Ok::<(), teistro_core::ratio::RatioError>(())
    /// ```
    #[must_use]
    #[allow(clippy::cast_precision_loss, reason = "a presentation value")]
    pub fn to_f64(self) -> f64 {
        self.num as f64 / self.den as f64
    }

    /// `floor(self × scale)` as an integer: a fraction of a span in some
    /// unit, rounded down, for materialising a period boundary.
    ///
    /// ```
    /// use teistro_core::ratio::Ratio;
    ///
    /// // Seven thirds of ten units, rounded down: towards minus infinity.
    /// assert_eq!(Ratio::new(7, 3)?.scaled_floor(10)?, 23);
    /// assert_eq!(Ratio::new(-7, 3)?.scaled_floor(10)?, -24);
    /// # Ok::<(), teistro_core::ratio::RatioError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Beyond the exact range.
    pub const fn scaled_floor(self, scale: i64) -> Result<i128, RatioError> {
        let product = self.num * (scale as i128);
        if product.abs() >= Ratio::LIMIT * Ratio::LIMIT {
            return Err(RatioError::Overflow);
        }
        Ok(product.div_euclid(self.den))
    }
}

impl Ord for Ratio {
    fn cmp(&self, other: &Ratio) -> Ordering {
        // Both parts are below 2⁶², so the products are below 2¹²⁴.
        (self.num * other.den).cmp(&(other.num * self.den))
    }
}

impl PartialOrd for Ratio {
    fn partial_cmp(&self, other: &Ratio) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Ratio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.den == 1 {
            write!(f, "{}", self.num)
        } else {
            write!(f, "{}/{}", self.num, self.den)
        }
    }
}

impl fmt::Debug for Ratio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ratio({self})")
    }
}

impl core::str::FromStr for Ratio {
    type Err = RatioError;

    fn from_str(s: &str) -> Result<Ratio, RatioError> {
        let (num, den) = s.split_once('/').unwrap_or((s, "1"));
        let num: i128 = num.trim().parse().map_err(|_| RatioError::Overflow)?;
        let den: i128 = den.trim().parse().map_err(|_| RatioError::Overflow)?;
        Ratio::new(num, den)
    }
}

impl serde::Serialize for Ratio {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> serde::Deserialize<'de> for Ratio {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Ratio, D::Error> {
        let text = String::deserialize(deserializer)?;
        text.parse().map_err(serde::de::Error::custom)
    }
}

hand_schema!(Ratio, "Ratio", {
    "type": "string",
    "description": "An exact rational, `numerator/denominator` or a whole number.",
    "pattern": "^\\s*[+-]?[0-9]+\\s*(/\\s*[+-]?[0-9]+\\s*)?$",
});

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

    fn r(n: i128, d: i128) -> Ratio {
        Ratio::new(n, d).unwrap_or_else(|e| panic!("{e}"))
    }

    #[test]
    fn construction_reduces_and_signs() {
        assert_eq!(r(2, 6), r(1, 3));
        assert_eq!(r(1, -3), r(-1, 3));
        assert_eq!(r(0, 5), Ratio::ZERO);
        assert_eq!(Ratio::new(1, 0), Err(RatioError::ZeroDenominator));
        assert_eq!(Ratio::new(Ratio::LIMIT, 1), Err(RatioError::Overflow));
        assert_eq!(r(120, 1).to_string(), "120");
        assert_eq!("7/20".parse::<Ratio>(), Ok(r(7, 20)));
    }

    #[test]
    fn a_vimshottari_cycle_sums_exactly() {
        let years = [7, 20, 6, 10, 7, 18, 16, 19, 17];
        let mut total = Ratio::ZERO;
        for y in years {
            total = total
                .checked_add(r(y, 120))
                .unwrap_or_else(|e| panic!("{e}"));
        }
        assert_eq!(total, Ratio::ONE);
        let nested = r(7, 120)
            .checked_mul(r(20, 120))
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(nested, r(7, 720));
        assert_eq!(r(1, 3).scaled_floor(1_000_000), Ok(333_333));
    }

    #[test]
    fn parts_read_back_and_refusals_say_why() {
        let x = Ratio::from_integer(-4);
        assert_eq!((x.numerator(), x.denominator()), (-4, 1));
        assert!(!x.is_zero() && Ratio::ZERO.is_zero());
        assert_eq!(r(3, 4).checked_div(r(-1, 2)), Ok(r(-3, 2)));
        assert_eq!(
            r(3, 4).checked_div(Ratio::ZERO),
            Err(RatioError::ZeroDenominator)
        );
        let big = r(Ratio::LIMIT - 1, 1);
        assert_eq!(big.scaled_floor(i64::MAX), Err(RatioError::Overflow));
        assert_eq!(r(-1, 3).scaled_floor(3), Ok(-1));
        assert_eq!(r(-1, 3).scaled_floor(2), Ok(-1));
        assert_eq!(r(1, 3).partial_cmp(&r(1, 2)), Some(Ordering::Less));
        assert_eq!(format!("{:?}", r(-2, 6)), "Ratio(-1/3)");
        assert_eq!(
            RatioError::ZeroDenominator.to_string(),
            "a rational needs a non-zero denominator"
        );
        assert_eq!(
            RatioError::Overflow.to_string(),
            "the rational exceeds the exact range"
        );
        assert_eq!(" 6 / -8 ".parse::<Ratio>(), Ok(r(-3, 4)));
        assert_eq!("x/2".parse::<Ratio>(), Err(RatioError::Overflow));
        assert_eq!("1/y".parse::<Ratio>(), Err(RatioError::Overflow));
        assert_eq!("1/0".parse::<Ratio>(), Err(RatioError::ZeroDenominator));
    }

    #[test]
    fn a_ratio_crosses_serde_as_its_text() {
        let json = serde_json::to_string(&r(7, 20)).unwrap();
        assert_eq!(json, "\"7/20\"");
        assert_eq!(serde_json::from_str::<Ratio>(&json).unwrap(), r(7, 20));
        assert_eq!(
            serde_json::from_str::<Ratio>("\"5\"").unwrap(),
            Ratio::from_integer(5)
        );
        assert!(serde_json::from_str::<Ratio>("\"1/0\"").is_err());
        assert!(serde_json::from_str::<Ratio>("3").is_err());
    }

    proptest! {
        #[test]
        fn ordering_agrees_with_floating_point(a in -1_000_000i128..1_000_000, b in 1i128..1_000_000, c in -1_000_000i128..1_000_000, d in 1i128..1_000_000) {
            let x = Ratio::new(a, b).unwrap_or(Ratio::ZERO);
            let y = Ratio::new(c, d).unwrap_or(Ratio::ZERO);
            let by_float = (x.to_f64()).partial_cmp(&y.to_f64()).unwrap_or(Ordering::Equal);
            if (x.to_f64() - y.to_f64()).abs() > 1e-9 {
                prop_assert_eq!(x.cmp(&y), by_float);
            }
            let sum = x.checked_add(y).unwrap_or(Ratio::ZERO);
            let back = sum.checked_sub(y).unwrap_or(Ratio::ZERO);
            prop_assert_eq!(back, x);
        }
    }
}
