//! The distributions the intervals and the exact test read: the normal
//! quantile, the regularised incomplete beta and its quantile, and the
//! hypergeometric. Every transcendental is `teistro_core::math`'s, and
//! every quantile is found by bisection to the last bit, so the answer
//! is the same on every platform.

use teistro_core::math;

/// `ln C(n, k)`, the log of a binomial coefficient.
fn ln_choose(n: u32, k: u32) -> f64 {
    let (n, k) = (f64::from(n), f64::from(k));
    math::ln_gamma(n + 1.0) - math::ln_gamma(k + 1.0) - math::ln_gamma(n - k + 1.0)
}

/// The `x` in `[low, high]` at which the increasing `f` reaches `target`,
/// halving until the interval cannot shrink further.
fn bisect(mut low: f64, mut high: f64, target: f64, f: impl Fn(f64) -> f64) -> f64 {
    loop {
        let middle = low + (high - low) / 2.0;
        if middle <= low || middle >= high {
            return middle;
        }
        if f(middle) < target {
            low = middle;
        } else {
            high = middle;
        }
    }
}

/// The standard normal distribution function, Φ(x) = ½·erfc(−x/√2).
pub(crate) fn normal_cdf(x: f64) -> f64 {
    0.5 * math::erfc(-x / core::f64::consts::SQRT_2)
}

/// Φ⁻¹(p), for `p` in (0, 1).
pub(crate) fn normal_quantile(p: f64) -> f64 {
    bisect(-40.0, 40.0, p, normal_cdf)
}

/// The continued fraction of the incomplete beta (Lentz's method, as in
/// *Numerical Recipes*, 3rd ed., §6.4).
#[allow(
    clippy::many_single_char_names,
    reason = "the names are the recurrence's own, as Numerical Recipes prints it"
)]
fn beta_fraction(a: f64, b: f64, x: f64) -> f64 {
    const TINY: f64 = 1e-300;
    let (qab, qap, qam) = (a + b, a + 1.0, a - 1.0);
    let mut c = 1.0;
    let mut d = 1.0 - qab * x / qap;
    if d.abs() < TINY {
        d = TINY;
    }
    d = 1.0 / d;
    let mut h = d;
    for m in 1..=100_000_u32 {
        let m = f64::from(m);
        let m2 = 2.0 * m;
        let aa = m * (b - m) * x / ((qam + m2) * (a + m2));
        d = 1.0 + aa * d;
        if d.abs() < TINY {
            d = TINY;
        }
        c = 1.0 + aa / c;
        if c.abs() < TINY {
            c = TINY;
        }
        d = 1.0 / d;
        h *= d * c;
        let aa = -(a + m) * (qab + m) * x / ((a + m2) * (qap + m2));
        d = 1.0 + aa * d;
        if d.abs() < TINY {
            d = TINY;
        }
        c = 1.0 + aa / c;
        if c.abs() < TINY {
            c = TINY;
        }
        d = 1.0 / d;
        let step = d * c;
        h *= step;
        if (step - 1.0).abs() <= f64::EPSILON {
            break;
        }
    }
    h
}

/// The regularised incomplete beta function `I_x(a, b)`, for `a, b > 0`.
pub(crate) fn beta_cdf(x: f64, a: f64, b: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    let front = math::exp(
        math::ln_gamma(a + b) - math::ln_gamma(a) - math::ln_gamma(b)
            + a * math::ln(x)
            + b * math::ln(1.0 - x),
    );
    if x < (a + 1.0) / (a + b + 2.0) {
        front * beta_fraction(a, b, x) / a
    } else {
        1.0 - front * beta_fraction(b, a, 1.0 - x) / b
    }
}

/// The `p` quantile of Beta(a, b).
pub(crate) fn beta_quantile(p: f64, a: f64, b: f64) -> f64 {
    bisect(0.0, 1.0, p, |x| beta_cdf(x, a, b))
}

/// The Clopper–Pearson interval on a binomial proportion, `successes` of
/// `trials`, at confidence `level`: the exact interval, whose coverage is
/// never below `level`.
pub(crate) fn clopper_pearson(successes: u64, trials: u64, level: f64) -> (f64, f64) {
    let tail = (1.0 - level) / 2.0;
    #[allow(clippy::cast_precision_loss)]
    let (k, n) = (successes as f64, trials as f64);
    let low = if successes == 0 {
        0.0
    } else {
        beta_quantile(tail, k, n - k + 1.0)
    };
    let high = if successes >= trials {
        1.0
    } else {
        beta_quantile(1.0 - tail, k + 1.0, n - k)
    };
    (low, high)
}

/// The hypergeometric distribution of how many of `drawn` charts carry a
/// predicate that `marked` of `total` carry: each count in its support
/// with its probability.
pub(crate) fn hypergeometric(total: u32, marked: u32, drawn: u32) -> Vec<(u32, f64)> {
    let first = drawn.saturating_sub(total - marked);
    let last = drawn.min(marked);
    let whole = ln_choose(total, drawn);
    (first..=last)
        .map(|k| {
            let ln_p = ln_choose(marked, k) + ln_choose(total - marked, drawn - k) - whole;
            (k, math::exp(ln_p))
        })
        .collect()
}
