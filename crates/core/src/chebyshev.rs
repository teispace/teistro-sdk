//! Chebyshev series: their evaluation, the interpolant through a smooth
//! function's values, and a store of fitted segments.
//!
//! A slowly varying quantity that is costly to compute — a series of a
//! thousand terms, each with a sine — is answered here from a polynomial
//! fitted through a few of its values over a short segment of time. The
//! segments are a fixed grid, so an instant's answer depends on the
//! instant alone, never on what was asked before it: the same request
//! gives the same bits whichever segments happen to be held, which is
//! what the SDK's content hashes rest on.

/// A Chebyshev series and its derivative at `x` in `[-1, 1]`, by
/// Clenshaw's recurrence.
///
/// Clenshaw rather than the power form because the power form loses
/// digits catastrophically at high degree, and rather than two passes
/// because the derivative's recurrence rides along with the value's for
/// the cost of two more additions per term.
///
/// ```
/// use teistro_core::chebyshev::evaluate;
///
/// // 1 + 2 T1(x) + 3 T2(x) at x = 0.5: 1 + 1 - 1.5, and its slope 2 + 12x.
/// let (value, slope) = evaluate(&[1.0, 2.0, 3.0], 0.5);
/// assert!((value - 0.5).abs() < 1e-15 && (slope - 8.0).abs() < 1e-15);
/// assert_eq!(evaluate(&[], 0.5), (0.0, 0.0));
/// ```
#[must_use]
pub fn evaluate(coefficients: &[f64], x: f64) -> (f64, f64) {
    let Some((first, rest)) = coefficients.split_first() else {
        return (0.0, 0.0);
    };
    let (mut b1, mut b2) = (0.0, 0.0);
    let (mut d1, mut d2) = (0.0, 0.0);
    for coefficient in rest.iter().rev() {
        let b0 = 2.0f64.mul_add(x * b1, -b2) + coefficient;
        let d0 = 2.0f64.mul_add(x * d1, 2.0 * b1) - d2;
        b2 = b1;
        b1 = b0;
        d2 = d1;
        d1 = d0;
    }
    (x.mul_add(b1, -b2) + first, x.mul_add(d1, b1) - d2)
}

/// The Chebyshev series of degree `N - 1` through a function's values at
/// the `N` Chebyshev–Lobatto points of `[start, start + length]`, one
/// series per component.
///
/// The Lobatto points are the extrema of the last polynomial,
/// `x_j = cos(pi j / (N - 1))`, and they include both ends: neighbouring
/// segments are fitted through the same value at the instant they share,
/// so a quantity fitted segment by segment is continuous across the seam
/// to the last bit of that value. The coefficients are the discrete
/// cosine transform of the values (type I, the ends weighted by a half),
/// which makes the series the interpolant rather than a least-squares
/// fit: it passes through every value it was given.
///
/// The cosines are the SDK's own ([`crate::math`]), so the same values
/// give the same coefficients on every architecture.
///
/// ```
/// use teistro_core::chebyshev::{evaluate, lobatto};
///
/// // t² over [0, 2] is 1.5 T0 + 2 T1 + 0.5 T2 in the segment's own x.
/// let [series] = lobatto::<3, 1>(|t| [t * t], 0.0, 2.0);
/// for (got, want) in series.iter().zip([1.5, 2.0, 0.5]) {
///     assert!((got - want).abs() < 1e-14);
/// }
/// // It passes through every value it was given: t = 1 is x = 0.
/// assert!((evaluate(&series, 0.0).0 - 1.0).abs() < 1e-14);
/// ```
#[must_use]
pub fn lobatto<const N: usize, const C: usize>(
    mut at: impl FnMut(f64) -> [f64; C],
    start: f64,
    length: f64,
) -> [[f64; N]; C] {
    let mut series = [[0.0; N]; C];
    let Some(degree) = N.checked_sub(1).filter(|degree| *degree > 0) else {
        return series;
    };
    // cos(pi m / degree) for m in 0..2 degree: every node, and every
    // product j k the transform reads, reduced by the period.
    #[expect(
        clippy::cast_precision_loss,
        reason = "a degree and its multiples are tens at most"
    )]
    let cosine = |m: usize| crate::math::cos(core::f64::consts::PI * m as f64 / degree as f64);
    let half = length / 2.0;
    let middle = start + half;
    let mut values = [[0.0; C]; N];
    for (j, value) in values.iter_mut().enumerate() {
        *value = at(half.mul_add(cosine(j), middle));
    }
    #[expect(clippy::cast_precision_loss, reason = "a degree is tens at most")]
    let scale = 2.0 / degree as f64;
    for (component, coefficients) in series.iter_mut().enumerate() {
        for (k, coefficient) in coefficients.iter_mut().enumerate() {
            let mut sum = 0.0;
            for (j, value) in values.iter().enumerate() {
                let weight = if j == 0 || j == degree { 0.5 } else { 1.0 };
                let term = value.get(component).copied().unwrap_or(0.0);
                sum += weight * term * cosine((j * k) % (2 * degree));
            }
            let weight = if k == 0 || k == degree { 0.5 } else { 1.0 };
            *coefficient = weight * scale * sum;
        }
    }
    series
}

/// How many segments a [`Segments`] store holds: enough that a year's
/// range, asked in lockstep rounds that sweep every day of it, keeps all
/// of its segments resident at two days each. At 64 a year's rounds
/// evicted each other's segments and refitted every one of them each
/// round, and a year of almanac days took twice as long as with no fit.
///
/// ```
/// use teistro_core::chebyshev::SLOTS;
///
/// // A year of two-day segments fits with room to spare.
/// assert!(SLOTS * 2 >= 366);
/// ```
pub const SLOTS: usize = 256;

/// [`SLOTS`] as a segment number's modulus.
const MODULUS: i64 = 256;

/// A slot that holds no segment yet. No instant numbers its segment so:
/// a segment number is the days from the origin over the length, and an
/// `f64` Julian day cannot reach `i64::MIN` segments away.
const EMPTY: i64 = i64::MIN;

/// Fitted segments of a quantity of `C` components over a fixed grid of
/// equal lengths, each a Chebyshev series through `N` values
/// ([`lobatto`]).
///
/// The store is directly mapped by the segment's number, so a year's
/// range stays resident and a root search that steps back and forth
/// across a seam does not fit either side twice. A segment evicted and
/// fitted again is the same bits. It holds no lock: a caller keeps one a
/// thread (a `thread_local!` of a `RefCell`), so threads share nothing.
/// The slots are allocated on the first instant asked, so a thread that
/// never asks pays nothing.
///
/// ```
/// use teistro_core::chebyshev::Segments;
///
/// // Segments of two days from JD 0, three values each: exact for a quadratic.
/// let mut store: Segments<3, 1> = Segments::new(0.0, 2.0);
/// let mut fits = 0;
/// for _ in 0..3 {
///     store.at(3.0, |t| { fits += 1; [t * t] });
/// }
/// // Fitted once, then answered from the store.
/// assert_eq!(fits, 3);
/// ```
#[derive(Clone, Debug)]
pub struct Segments<const N: usize, const C: usize> {
    /// The first segment's start, a Julian day.
    origin: f64,
    /// Each segment's length, days.
    length: f64,
    /// The segments held, by slot: the segment's number and its series.
    /// Empty until the first instant is asked, then [`SLOTS`] long.
    slots: Vec<(i64, [[f64; N]; C])>,
}

impl<const N: usize, const C: usize> Segments<N, C> {
    /// An empty store of segments of `length` days from `origin`.
    ///
    /// A length that is a power of two keeps an instant's place in its
    /// segment exact.
    ///
    /// ```
    /// use teistro_core::chebyshev::Segments;
    ///
    /// // Segments of two days from JD 0, three values each: exact for a quadratic.
    /// let mut store: Segments<3, 1> = Segments::new(0.0, 2.0);
    /// assert_eq!(store.start_of(0), 0.0);
    /// ```
    #[must_use]
    pub const fn new(origin: f64, length: f64) -> Segments<N, C> {
        Segments {
            origin,
            length,
            slots: Vec::new(),
        }
    }

    /// The number of the segment an instant falls in: the one that
    /// starts at or before it.
    ///
    /// ```
    /// use teistro_core::chebyshev::Segments;
    ///
    /// // Segments of two days from JD 0, three values each: exact for a quadratic.
    /// let mut store: Segments<3, 1> = Segments::new(0.0, 2.0);
    /// assert_eq!((store.segment_of(3.0), store.segment_of(4.0), store.segment_of(-0.5)), (1, 2, -1));
    /// ```
    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a finite Julian day over a length of days is far inside i64"
    )]
    pub fn segment_of(&self, jd: f64) -> i64 {
        ((jd - self.origin) / self.length).floor() as i64
    }

    /// A segment's first instant.
    ///
    /// ```
    /// use teistro_core::chebyshev::Segments;
    ///
    /// // Segments of two days from JD 0, three values each: exact for a quadratic.
    /// let mut store: Segments<3, 1> = Segments::new(0.0, 2.0);
    /// assert_eq!(store.start_of(1), 2.0);
    /// assert_eq!(store.start_of(-1), -2.0);
    /// ```
    #[must_use]
    #[expect(
        clippy::cast_precision_loss,
        reason = "a segment number is far below 2^53"
    )]
    pub fn start_of(&self, segment: i64) -> f64 {
        self.length.mul_add(segment as f64, self.origin)
    }

    /// The quantity and its rate a day at `jd`, from the segment the
    /// instant falls in, fitting that segment from `at` first when it is
    /// not held.
    ///
    /// `None` for an instant that is not finite, which falls in no
    /// segment; the caller answers it from `at` itself.
    ///
    /// ```
    /// use teistro_core::chebyshev::Segments;
    ///
    /// // Segments of two days from JD 0, three values each: exact for a quadratic.
    /// let mut store: Segments<3, 1> = Segments::new(0.0, 2.0);
    /// // t² and its rate 2t at t = 3, from the segment [2, 4].
    /// let ([value], [rate]) = store.at(3.0, |t| [t * t]).unwrap();
    /// assert!((value - 9.0).abs() < 1e-12 && (rate - 6.0).abs() < 1e-12);
    /// assert_eq!(store.at(f64::NAN, |t| [t * t]), None);
    /// ```
    pub fn at(&mut self, jd: f64, at: impl FnMut(f64) -> [f64; C]) -> Option<([f64; C], [f64; C])> {
        if !jd.is_finite() || self.length <= 0.0 {
            return None;
        }
        let segment = self.segment_of(jd);
        let start = self.start_of(segment);
        let length = self.length;
        let slot = usize::try_from(segment.rem_euclid(MODULUS)).ok()?;
        if self.slots.is_empty() {
            self.slots = vec![(EMPTY, [[0.0; N]; C]); SLOTS];
        }
        let held = self.slots.get_mut(slot)?;
        if held.0 != segment {
            *held = (segment, lobatto::<N, C>(at, start, length));
        }
        let x = (jd - start).mul_add(2.0 / length, -1.0);
        let per_day = 2.0 / length;
        let mut value = [0.0; C];
        let mut rate = [0.0; C];
        for ((value, rate), series) in value.iter_mut().zip(&mut rate).zip(&held.1) {
            let (at, slope) = evaluate(series, x);
            *value = at;
            *rate = slope * per_day;
        }
        Some((value, rate))
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::float_cmp,
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "a test fails by panicking, indexes its own fixtures, and checks a \
                  segment gives the same bits by comparing exactly"
    )]

    use super::{SLOTS, Segments, evaluate, lobatto};

    /// `T_0 = 1`, `T_1 = x`, `T_2 = 2x^2 - 1`: the series a reader can
    /// check by hand, with the derivatives it must produce.
    #[test]
    fn the_recurrence_is_the_polynomials_it_claims() {
        for x in [-1.0, -0.5, 0.0, 0.25, 1.0_f64] {
            assert!((evaluate(&[1.0], x).0 - 1.0).abs() < 1e-15, "T0 at {x}");
            assert!(evaluate(&[1.0], x).1.abs() < 1e-15, "T0' at {x}");
            assert!((evaluate(&[0.0, 1.0], x).0 - x).abs() < 1e-15, "T1 at {x}");
            assert!(
                (evaluate(&[0.0, 1.0], x).1 - 1.0).abs() < 1e-15,
                "T1' at {x}"
            );
            let t2 = 2.0f64.mul_add(x * x, -1.0);
            assert!(
                (evaluate(&[0.0, 0.0, 1.0], x).0 - t2).abs() < 1e-15,
                "T2 at {x}"
            );
            assert!(
                (evaluate(&[0.0, 0.0, 1.0], x).1 - 4.0 * x).abs() < 1e-14,
                "T2' at {x}"
            );
        }
        assert_eq!(evaluate(&[], 0.5), (0.0, 0.0));
    }

    /// A polynomial of the fit's degree is reproduced exactly, so the
    /// transform's weights and the nodes' placement are the right ones;
    /// and the interpolant passes through the values at both ends.
    #[test]
    fn the_interpolant_reproduces_a_polynomial_of_its_degree() {
        let cubic = |t: f64| [t.powi(3) - 2.0 * t, 5.0, 0.5 * t * t];
        let series = lobatto::<4, 3>(cubic, 1.0, 2.0);
        for t in [1.0, 1.3, 2.0, 2.71, 3.0_f64] {
            let x = t - 2.0;
            for (component, coefficients) in series.iter().enumerate() {
                let (value, slope) = evaluate(coefficients, x);
                let expected = cubic(t)[component];
                assert!((value - expected).abs() < 1e-12, "{component} at {t}");
                let derivative = [3.0 * t * t - 2.0, 0.0, t][component];
                assert!((slope - derivative).abs() < 1e-12, "{component}' at {t}");
            }
        }
    }

    /// A sine over a short segment, the shape the Moon's coordinates
    /// have, is fitted to the last digits by nine values.
    #[test]
    fn a_smooth_arc_is_fitted_far_below_its_size() {
        let mut segments = Segments::<9, 1>::new(0.0, 2.0);
        let arc = |t: f64| [(0.23 * t).sin()];
        for i in 0..200 {
            let t = f64::from(i) * 0.37;
            let (value, rate) = segments.at(t, arc).unwrap();
            assert!((value[0] - arc(t)[0]).abs() < 1e-13, "at {t}");
            assert!(
                (rate[0] - 0.23 * (0.23 * t).cos()).abs() < 1e-11,
                "rate at {t}"
            );
        }
    }

    /// The answer is the same bits fresh, after neighbours, and after the
    /// segment was evicted by one that maps to the same slot.
    #[test]
    fn an_answer_does_not_depend_on_what_was_held() {
        let quantity = |t: f64| [(0.1 * t).cos(), t.sqrt()];
        let jd = 2_461_000.123_456;
        let fresh = Segments::<9, 2>::new(2_451_545.0, 2.0)
            .at(jd, quantity)
            .unwrap();
        let mut segments = Segments::<9, 2>::new(2_451_545.0, 2.0);
        let _ = segments.at(jd - 1.7, quantity);
        assert_eq!(segments.at(jd, quantity).unwrap(), fresh);
        #[expect(clippy::cast_precision_loss, reason = "a slot count")]
        let evicting = jd + SLOTS as f64 * 2.0;
        let _ = segments.at(evicting, quantity);
        assert_eq!(segments.at(jd, quantity).unwrap(), fresh);
    }

    /// An instant belongs to the segment that starts at or before it,
    /// and an instant that is not finite to none.
    #[test]
    fn the_segments_are_a_fixed_grid() {
        let mut segments = Segments::<3, 1>::new(2_451_545.0, 2.0);
        assert_eq!(segments.segment_of(2_451_545.0), 0);
        assert_eq!(segments.segment_of(2_451_546.999), 0);
        assert_eq!(segments.segment_of(2_451_547.0), 1);
        assert_eq!(segments.segment_of(2_451_544.999), -1);
        assert_eq!(segments.start_of(-1), 2_451_543.0);
        assert!(segments.at(f64::NAN, |t| [t]).is_none());
        assert!(segments.at(f64::INFINITY, |t| [t]).is_none());
    }
}
