//! The transcendental functions every computation crate calls, one
//! implementation on every target (ADR-0022; `03-design/wasm-binding.md`,
//! "Determinism first").
//!
//! `f64::sin` and its kin are the platform's C library: Apple's on macOS,
//! glibc on Linux, the CRT on Windows, and a WASI libc or a Rust port in
//! wasm32. They are faithful to about an ulp, and **they are not the same
//! ulp**. Measured over two million inputs each, Apple's library differs
//! from the others in every one of the thirteen functions the SDK called
//! when it was measured, and the scenario's `astro` and `houses` sections
//! differ in 2 015 of their values between Linux and macOS and in 5 193
//! between macOS and wasm32 — by a relative 8e-14 at worst, which no
//! reader can see and which still breaks a byte-identical document.
//!
//! These are the `libm` crate's, a pure-Rust port of musl's, compiled the
//! same way on every target, so they give the same bits everywhere; the
//! same measurement found them identical natively and in wasm32 for all
//! thirteen. `ln_gamma` and `erfc`, which the research statistics added
//! later, are the same port's and were not in that measurement. A lint
//! refuses the `f64` methods in every library crate
//! (`uses-one-libm`), so a new call cannot quietly reach the platform's.
//!
//! What is not here needs nothing: `sqrt`, `mul_add`, `abs`, `floor`,
//! `rem_euclid` and the arithmetic operators are exact or correctly
//! rounded by IEEE 754, and so already agree everywhere.
//!
//! ```
//! use teistro_core::math;
//!
//! let (sin, cos) = math::sin_cos(core::f64::consts::FRAC_PI_6);
//! assert!((sin - 0.5).abs() < 1e-15);
//! assert!((cos - 3f64.sqrt() / 2.0).abs() < 1e-15);
//! assert_eq!(math::powi(3.0, 4).to_bits(), 81f64.to_bits());
//! ```

/// The sine of `x` radians.
///
/// ```
/// use core::f64::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_6, PI};
/// use teistro_core::math;
///
/// assert!((math::sin(FRAC_PI_6) - 0.5).abs() < 1e-15);
/// assert_eq!(math::sin(0.0), 0.0);
/// ```
#[inline]
#[must_use]
pub fn sin(x: f64) -> f64 {
    libm::sin(x)
}

/// The cosine of `x` radians.
///
/// ```
/// use core::f64::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_6, PI};
/// use teistro_core::math;
///
/// assert!(math::cos(FRAC_PI_2).abs() < 1e-15);
/// assert_eq!(math::cos(0.0), 1.0);
/// ```
#[inline]
#[must_use]
pub fn cos(x: f64) -> f64 {
    libm::cos(x)
}

/// The sine and cosine of `x` radians, as one call.
///
/// ```
/// use core::f64::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_6, PI};
/// use teistro_core::math;
///
/// let (sin, cos) = math::sin_cos(PI / 3.0);
/// // The same bits as the two calls apart.
/// assert_eq!((sin.to_bits(), cos.to_bits()), (math::sin(PI / 3.0).to_bits(), math::cos(PI / 3.0).to_bits()));
/// ```
#[inline]
#[must_use]
pub fn sin_cos(x: f64) -> (f64, f64) {
    libm::sincos(x)
}

/// The tangent of `x` radians.
///
/// ```
/// use core::f64::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_6, PI};
/// use teistro_core::math;
///
/// assert!((math::tan(FRAC_PI_4) - 1.0).abs() < 1e-15);
/// ```
#[inline]
#[must_use]
pub fn tan(x: f64) -> f64 {
    libm::tan(x)
}

/// The arcsine of `x`, in radians from −π/2 to π/2; NaN outside [−1, 1].
///
/// ```
/// use core::f64::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_6, PI};
/// use teistro_core::math;
///
/// assert!((math::asin(0.5) - FRAC_PI_6).abs() < 1e-15);
/// assert!(math::asin(1.5).is_nan());
/// ```
#[inline]
#[must_use]
pub fn asin(x: f64) -> f64 {
    libm::asin(x)
}

/// The arccosine of `x`, in radians from 0 to π; NaN outside [−1, 1].
///
/// ```
/// use core::f64::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_6, PI};
/// use teistro_core::math;
///
/// assert!((math::acos(-1.0) - PI).abs() < 1e-15);
/// assert!(math::acos(-1.5).is_nan());
/// ```
#[inline]
#[must_use]
pub fn acos(x: f64) -> f64 {
    libm::acos(x)
}

/// The arctangent of `x`, in radians from −π/2 to π/2.
///
/// ```
/// use core::f64::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_6, PI};
/// use teistro_core::math;
///
/// assert!((math::atan(1.0) - FRAC_PI_4).abs() < 1e-15);
/// ```
#[inline]
#[must_use]
pub fn atan(x: f64) -> f64 {
    libm::atan(x)
}

/// The angle of the point (`x`, `y`) from the positive x axis, in radians
/// from −π to π; `y` first, as `f64::atan2` takes it.
///
/// ```
/// use core::f64::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_6, PI};
/// use teistro_core::math;
///
/// // The third quadrant, which `atan(y / x)` cannot tell from the first.
/// assert!((math::atan2(-1.0, -1.0) + 3.0 * FRAC_PI_4).abs() < 1e-15);
/// ```
#[inline]
#[must_use]
pub fn atan2(y: f64, x: f64) -> f64 {
    libm::atan2(y, x)
}

/// e raised to `x`.
///
/// ```
/// use teistro_core::math;
///
/// assert_eq!(math::exp(0.0), 1.0);
/// assert!((math::exp(1.0) - core::f64::consts::E).abs() < 1e-15);
/// ```
#[inline]
#[must_use]
pub fn exp(x: f64) -> f64 {
    libm::exp(x)
}

/// The natural logarithm of `x`.
///
/// ```
/// use teistro_core::math;
///
/// assert_eq!(math::ln(1.0), 0.0);
/// assert!((math::ln(core::f64::consts::E) - 1.0).abs() < 1e-15);
/// ```
#[inline]
#[must_use]
pub fn ln(x: f64) -> f64 {
    libm::log(x)
}

/// The natural logarithm of the gamma function's magnitude, |Γ(x)|; at a
/// whole number `n + 1` it is ln n!. The research statistics' binomial
/// coefficients and beta function are built from it.
///
/// ```
/// use teistro_core::math;
///
/// // ln 5! = ln 120.
/// assert!((math::ln_gamma(6.0) - math::ln(120.0)).abs() < 1e-13);
/// ```
#[inline]
#[must_use]
pub fn ln_gamma(x: f64) -> f64 {
    libm::lgamma(x)
}

/// The complementary error function, 1 − erf(x), accurate in the tail
/// where 1 − erf(x) would round to zero; the normal distribution's tail
/// is ½·erfc(x/√2).
///
/// ```
/// use teistro_core::math;
///
/// assert_eq!(math::erfc(0.0), 1.0);
/// // The normal tail beyond 1.96 sigma, about 2.5 %.
/// let tail = 0.5 * math::erfc(1.96 / 2f64.sqrt());
/// assert!((tail - 0.025).abs() < 1e-4);
/// ```
#[inline]
#[must_use]
pub fn erfc(x: f64) -> f64 {
    libm::erfc(x)
}

/// The base-10 logarithm of `x`.
///
/// ```
/// use teistro_core::math;
///
/// assert!((math::log10(1000.0) - 3.0).abs() < 1e-15);
/// ```
#[inline]
#[must_use]
pub fn log10(x: f64) -> f64 {
    libm::log10(x)
}

/// `x` raised to the power `y`.
///
/// ```
/// use teistro_core::math;
///
/// assert!((math::powf(2.0, 0.5) - 2f64.sqrt()).abs() < 1e-15);
/// assert!((math::powf(10.0, -2.0) - 0.01).abs() < 1e-17);
/// ```
#[inline]
#[must_use]
pub fn powf(x: f64, y: f64) -> f64 {
    libm::pow(x, y)
}

/// The length of the hypotenuse, √(x² + y²), without overflowing on the
/// way.
///
/// ```
/// use teistro_core::math;
///
/// assert_eq!(math::hypot(3.0, 4.0), 5.0);
/// // Squaring 1e300 would overflow; the hypotenuse does not.
/// assert!(math::hypot(1e300, 1e300).is_finite());
/// ```
#[inline]
#[must_use]
pub fn hypot(x: f64, y: f64) -> f64 {
    libm::hypot(x, y)
}

/// `x` raised to the whole power `n`, by squaring and multiplying.
///
/// `f64::powi` is an LLVM intrinsic whose order of multiplications is
/// the target's to choose, and it differs natively and in wasm32. This is
/// the order written here, and every step is one IEEE multiplication, so
/// it is the same everywhere. A negative power is the reciprocal of the
/// positive one.
///
/// ```
/// use teistro_core::math;
///
/// assert_eq!(math::powi(2.0, 10), 1024.0);
/// assert_eq!(math::powi(2.0, -2), 0.25);
/// assert_eq!(math::powi(7.0, 0), 1.0);
/// ```
#[inline]
#[must_use]
pub fn powi(x: f64, n: i32) -> f64 {
    let mut base = x;
    let mut exponent = n.unsigned_abs();
    let mut result = 1.0;
    while exponent > 0 {
        if exponent & 1 == 1 {
            result *= base;
        }
        base *= base;
        exponent >>= 1;
    }
    if n < 0 { 1.0 / result } else { result }
}

/// The remainder of `x` divided by `y`, the sign of `x`: `x % y` (C's
/// `fmod`) to the bit, sooner.
///
/// The remainder is exact, so any routine that computes it exactly gives
/// the same bits. `%` calls the C library's `fmod`, which divides a bit
/// at a time, and the nutation series takes one for each of its 77 terms
/// at every reading of the sky: a fifth of a year of almanac days went
/// there. This takes the quotient's whole part and subtracts that many
/// `y`s in one fused multiply-add, which rounds once, and the exact result
/// is representable, so it is exact too; the quotient can only be one too
/// large (the division rounded up onto a whole number), which leaves the
/// result one `y` below zero and is mended by adding it back, again
/// exactly. A quotient past 2⁵² (where its whole part is no longer the
/// division's), a zero, an infinite or a NaN, and a target that has no
/// fused multiply-add instruction take `%` itself.
///
/// ```
/// use teistro_core::math;
///
/// assert_eq!(math::rem(7.5, 2.0), 1.5);
/// assert_eq!(math::rem(-7.5, 2.0), -1.5);
/// assert_eq!(math::rem(-4.0, 2.0).to_bits(), (-0.0f64).to_bits());
/// let x = 1_717_915_923.217_8 * 0.25;
/// assert_eq!(math::rem(x, 1_296_000.0).to_bits(), (x % 1_296_000.0).to_bits());
/// ```
#[inline]
#[must_use]
pub fn rem(x: f64, y: f64) -> f64 {
    #[cfg(any(target_arch = "aarch64", target_feature = "fma"))]
    {
        /// 2⁵²: below it a double's whole part is exact.
        const EXACT_WHOLE: f64 = 4_503_599_627_370_496.0;
        let (ax, ay) = (x.abs(), y.abs());
        let whole = (ax / ay).trunc();
        // A NaN anywhere fails the comparison and takes `%`.
        if whole < EXACT_WHOLE && ax.is_finite() && ay.is_finite() && ay > 0.0 {
            let mut r = (-whole).mul_add(ay, ax);
            if r < 0.0 {
                r += ay;
            }
            return r.copysign(x);
        }
    }
    x % y
}

/// The least non-negative remainder of `x` by `y`: `f64::rem_euclid` to
/// the bit, by [`rem`], which is how the standard library writes it over
/// `%`.
///
/// ```
/// use teistro_core::math;
///
/// assert_eq!(math::rem_euclid(-30.0, 360.0), 330.0);
/// assert_eq!(math::rem_euclid(-30.0, 360.0).to_bits(), (-30.0f64).rem_euclid(360.0).to_bits());
/// ```
#[inline]
#[must_use]
pub fn rem_euclid(x: f64, y: f64) -> f64 {
    let r = rem(x, y);
    if r < 0.0 { r + y.abs() } else { r }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    /// A whole power is exact where the product is representable, and a
    /// negative one is its reciprocal.
    #[test]
    fn a_whole_power_squares_and_multiplies() {
        let bits = |x: f64| x.to_bits();
        assert_eq!(bits(powi(2.0, 10)), bits(1024.0));
        assert_eq!(bits(powi(-3.0, 3)), bits(-27.0));
        assert_eq!(bits(powi(7.5, 0)), bits(1.0));
        assert_eq!(bits(powi(2.0, -2)), bits(0.25));
        assert_eq!(bits(powi(0.1, 2)), bits(0.1 * 0.1));
    }

    /// `sin_cos` is `sin` and `cos` to the bit wherever a series reads
    /// it, so a caller that needs both may take one reduction for two.
    #[test]
    fn sin_cos_is_sin_and_cos_to_the_bit() {
        for i in -200_000..=200_000 {
            let x = f64::from(i) * 0.000_731 + f64::from(i % 7) * 1e-9;
            let (s, c) = sin_cos(x);
            assert_eq!(
                (s.to_bits(), c.to_bits()),
                (sin(x).to_bits(), cos(x).to_bits()),
                "{x:e}"
            );
        }
        for x in [0.0, -0.0, 1e-300, -1e-20, 1e5, -3e7, 1e300] {
            let (s, c) = sin_cos(x);
            assert_eq!(
                (s.to_bits(), c.to_bits()),
                (sin(x).to_bits(), cos(x).to_bits()),
                "{x:e}"
            );
        }
    }

    /// The quick remainder is `%` to the bit: over the arguments the
    /// nutation and the sidereal time reduce, both signs, values a hair
    /// either side of a whole multiple, and the special values.
    #[test]
    fn the_remainder_is_the_operators_to_the_bit() {
        let same = |x: f64, y: f64| {
            assert_eq!(rem(x, y).to_bits(), (x % y).to_bits(), "{x:e} % {y:e}");
            assert_eq!(
                rem_euclid(x, y).to_bits(),
                x.rem_euclid(y).to_bits(),
                "{x:e} rem_euclid {y:e}"
            );
        };
        let divisors = [
            core::f64::consts::TAU,
            1_296_000.0,
            1.0,
            360.0,
            -360.0,
            0.1,
            3.0,
            f64::MIN_POSITIVE,
            1e300,
        ];
        // A fixed walk of a 64-bit generator, so the inputs are the same
        // on every run.
        let mut state = 0x9E37_79B9_7F4A_7C15_u64;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        for &y in &divisors {
            for _ in 0..20_000 {
                let bits = next();
                // Magnitudes from a millionth to ten million of the divisor.
                #[allow(clippy::cast_precision_loss, reason = "a generator's draw scaled")]
                let scale = (bits >> 11) as f64 / (1u64 << 53) as f64;
                let x = y * libm::exp(scale * 30.0 - 14.0);
                same(x, y);
                same(-x, y);
                for k in [1.0, 2.0, 7.0, 1e6] {
                    let multiple = k * y;
                    same(multiple, y);
                    same(-multiple, y);
                    same(f64::from_bits(multiple.to_bits() + 1), y);
                    same(f64::from_bits(multiple.to_bits() - 1), y);
                }
            }
            for x in [
                0.0,
                -0.0,
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::NAN,
                1e308,
                -1e-310,
            ] {
                same(x, y);
            }
        }
        for x in [1.0, -2.5, 0.0] {
            same(x, 0.0);
            same(x, f64::INFINITY);
            same(x, f64::NAN);
        }
    }

    /// Each function is the one its name says, to within an ulp of the
    /// platform's — the point being the bits, not a new approximation.
    #[test]
    fn each_function_agrees_with_the_platform_to_an_ulp() {
        let close = |a: f64, b: f64| (a - b).abs() <= 2.0 * f64::EPSILON * a.abs().max(1.0);
        for i in -40..=40 {
            let x = f64::from(i) * 0.173;
            let u = f64::from(i) / 41.0;
            assert!(close(sin(x), x.sin()), "sin {x}");
            assert!(close(cos(x), x.cos()), "cos {x}");
            assert_eq!(sin_cos(x), (sin(x), cos(x)), "sin_cos {x}");
            assert!(close(tan(x), x.tan()), "tan {x}");
            assert!(close(asin(u), u.asin()), "asin {u}");
            assert!(close(acos(u), u.acos()), "acos {u}");
            assert!(close(atan(x), x.atan()), "atan {x}");
            assert!(close(atan2(x, 1.3), x.atan2(1.3)), "atan2 {x}");
            assert!(close(exp(u), u.exp()), "exp {u}");
            assert!(close(hypot(x, 2.0), x.hypot(2.0)), "hypot {x}");
            if x > 0.0 {
                assert!(close(ln(x), x.ln()), "ln {x}");
                assert!(close(log10(x), x.log10()), "log10 {x}");
                assert!(close(powf(x, 1.7), x.powf(1.7)), "powf {x}");
            }
            assert!(close(powi(x, 5), x.powi(5)), "powi {x}");
        }
    }
}
