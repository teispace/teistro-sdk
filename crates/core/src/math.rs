//! The transcendental functions every computation crate calls, one
//! implementation on every target (ADR-0022; `03-design/wasm-binding.md`,
//! "Determinism first").
//!
//! `f64::sin` and its kin are the platform's C library: Apple's on macOS,
//! glibc on Linux, the CRT on Windows, and a WASI libc or a Rust port in
//! wasm32. They are faithful to about an ulp, and **they are not the same
//! ulp**. Measured over two million inputs each, Apple's library differs
//! from the others in every one of the thirteen functions the SDK calls,
//! and the scenario's `astro` and `houses` sections differ in 2 015 of
//! their values between Linux and macOS and in 5 193 between macOS and
//! wasm32 — by a relative 8e-14 at worst, which no reader can see and
//! which still breaks a byte-identical document.
//!
//! These are the `libm` crate's, a pure-Rust port of musl's, compiled the
//! same way on every target, so they give the same bits everywhere; the
//! same measurement found them identical natively and in wasm32 for all
//! thirteen. A lint refuses the `f64` methods in every library crate
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
#[inline]
#[must_use]
pub fn sin(x: f64) -> f64 {
    libm::sin(x)
}

/// The cosine of `x` radians.
#[inline]
#[must_use]
pub fn cos(x: f64) -> f64 {
    libm::cos(x)
}

/// The sine and cosine of `x` radians, as one call.
#[inline]
#[must_use]
pub fn sin_cos(x: f64) -> (f64, f64) {
    libm::sincos(x)
}

/// The tangent of `x` radians.
#[inline]
#[must_use]
pub fn tan(x: f64) -> f64 {
    libm::tan(x)
}

/// The arcsine of `x`, in radians from −π/2 to π/2; NaN outside [−1, 1].
#[inline]
#[must_use]
pub fn asin(x: f64) -> f64 {
    libm::asin(x)
}

/// The arccosine of `x`, in radians from 0 to π; NaN outside [−1, 1].
#[inline]
#[must_use]
pub fn acos(x: f64) -> f64 {
    libm::acos(x)
}

/// The arctangent of `x`, in radians from −π/2 to π/2.
#[inline]
#[must_use]
pub fn atan(x: f64) -> f64 {
    libm::atan(x)
}

/// The angle of the point (`x`, `y`) from the positive x axis, in radians
/// from −π to π; `y` first, as `f64::atan2` takes it.
#[inline]
#[must_use]
pub fn atan2(y: f64, x: f64) -> f64 {
    libm::atan2(y, x)
}

/// e raised to `x`.
#[inline]
#[must_use]
pub fn exp(x: f64) -> f64 {
    libm::exp(x)
}

/// The natural logarithm of `x`.
#[inline]
#[must_use]
pub fn ln(x: f64) -> f64 {
    libm::log(x)
}

/// The base-10 logarithm of `x`.
#[inline]
#[must_use]
pub fn log10(x: f64) -> f64 {
    libm::log10(x)
}

/// `x` raised to the power `y`.
#[inline]
#[must_use]
pub fn powf(x: f64, y: f64) -> f64 {
    libm::pow(x, y)
}

/// The length of the hypotenuse, √(x² + y²), without overflowing on the
/// way.
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

#[cfg(test)]
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
