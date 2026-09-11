//! What a body with no analytic theory is carried by: a **fitted table**
//! of Chebyshev series, one block per interval.
//!
//! Every other body here comes from a published series — VSOP87 for the
//! planets, ELP2000-82B for the Moon. Pluto has neither (ADR-0008), so it
//! is fitted from a reference ephemeris by the SDK's own tool and shipped
//! as coefficients. That is the same shape a JPL kernel uses, and for the
//! same reason: a slow body over a long span is cheap to fit and cheap to
//! evaluate, and nothing has to be re-derived at runtime.
//!
//! # Why the rate is analytic
//!
//! The port asks for a speed beside every position, and this crate's
//! contract is that it **differentiates rather than differences**
//! (`tests/provider.rs`). A central difference over a fitted table would
//! return an average rate across the step, which the conformance kit
//! measures and rejects. A Chebyshev series has an exact derivative, so
//! [`Block::at`] returns both from one pass of a single recurrence.
//!
//! # The shape of a table
//!
//! Equal intervals over a contiguous span, each interval holding one
//! series per rectangular component. Equal intervals are what let an
//! instant find its block by arithmetic rather than by search, which
//! matters when a consumer asks for a grid of thousands of them.

/// A fitted table: equal intervals over one contiguous span.
///
/// The coefficients are flattened `[interval][component][term]` so that a
/// table is one `&'static [f64]` in the binary rather than a tree of
/// slices — a generated table of nested slices costs a relocation per
/// interval, and this one is meant for a wasm consumer counting
/// kilobytes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Table {
    /// The first interval's start, Julian day in the dynamical scale.
    pub start_jd: f64,
    /// Each interval's length, days.
    pub interval_days: f64,
    /// Terms per component, which is the degree plus one.
    pub terms: usize,
    /// `[interval][component][term]`, flattened.
    pub coefficients: &'static [f64],
}

/// The number of rectangular components a block carries.
const COMPONENTS: usize = 3;

impl Table {
    /// How many intervals the table holds.
    ///
    /// Derived rather than stored: a stored count and a coefficient slice
    /// can disagree, and this crate has been bitten by a declared number
    /// that nothing checked.
    #[must_use]
    pub const fn intervals(&self) -> usize {
        if self.terms == 0 {
            return 0;
        }
        self.coefficients.len() / (COMPONENTS * self.terms)
    }

    /// The span the table covers, Julian days, as `(first, last)`.
    ///
    /// An empty table covers nothing, and says so as an empty range
    /// rather than as a point.
    #[must_use]
    pub fn span(&self) -> (f64, f64) {
        let intervals = self.intervals();
        #[expect(
            clippy::cast_precision_loss,
            reason = "an interval count is thousands at most"
        )]
        let length = intervals as f64 * self.interval_days;
        (self.start_jd, self.start_jd + length)
    }

    /// Whether an instant falls inside the table.
    #[must_use]
    pub fn covers(&self, jd: f64) -> bool {
        let (first, last) = self.span();
        jd >= first && jd <= last && self.intervals() > 0
    }

    /// The position and its rate at an instant: astronomical units and
    /// astronomical units a day, in whatever frame the table was fitted
    /// in.
    ///
    /// `None` outside the span — a fitted table is worthless beyond its
    /// fit, and a Chebyshev series diverges spectacularly rather than
    /// gracefully, so this refuses rather than extrapolates.
    #[must_use]
    pub fn at(&self, jd: f64) -> Option<([f64; 3], [f64; 3])> {
        if !self.covers(jd) || self.interval_days <= 0.0 {
            return None;
        }
        let intervals = self.intervals();
        let offset = (jd - self.start_jd) / self.interval_days;
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "the offset is non-negative and under the interval count, \
                      both checked by `covers`"
        )]
        // The last instant belongs to the last block rather than to one
        // past the end, which is what `floor` alone would give it.
        let index = (offset.floor() as usize).min(intervals - 1);
        #[expect(
            clippy::cast_precision_loss,
            reason = "an interval index is thousands at most"
        )]
        // Chebyshev's variable runs over [-1, 1] across the interval.
        let x = 2.0f64.mul_add(offset - index as f64, -1.0);
        // Two units of x to one interval, so the chain rule's factor.
        let per_day = 2.0 / self.interval_days;

        let mut position = [0.0; COMPONENTS];
        let mut rate = [0.0; COMPONENTS];
        for component in 0..COMPONENTS {
            let from = (index * COMPONENTS + component) * self.terms;
            let series = self.coefficients.get(from..from + self.terms)?;
            let (value, slope) = evaluate(series, x);
            *position.get_mut(component)? = value;
            *rate.get_mut(component)? = slope * per_day;
        }
        Some((position, rate))
    }
}

/// A Chebyshev series and its derivative at `x` in `[-1, 1]`, by
/// Clenshaw's recurrence.
///
/// Clenshaw rather than the power form because the power form loses
/// digits catastrophically at high degree, and rather than two passes
/// because the derivative's recurrence rides along with the value's for
/// the cost of two more additions per term.
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

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::indexing_slicing,
        clippy::float_cmp,
        clippy::expect_used,
        reason = "a test fails by panicking, indexes its own fixtures, and \
                  compares a span it constructed exactly"
    )]

    use super::{Table, evaluate};

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

    /// The table's rate is the derivative of its own position, which is
    /// the property the port's speed contract rests on. A central
    /// difference is the check and **not** the implementation: it is
    /// accurate enough to catch a wrong chain-rule factor, and not
    /// accurate enough to ship.
    #[test]
    fn the_rate_is_the_derivative_of_the_position() {
        // x(t) = T2 over each interval, y and z linear, over two days.
        static COEFFICIENTS: [f64; 9] = [
            0.0, 0.0, 1.0, // x
            0.0, 1.0, 0.0, // y
            3.0, 0.0, 0.0, // z
        ];
        let table = Table {
            start_jd: 2_451_545.0,
            interval_days: 2.0,
            terms: 3,
            coefficients: &COEFFICIENTS,
        };
        assert_eq!(table.intervals(), 1);
        assert_eq!(table.span(), (2_451_545.0, 2_451_547.0));
        // A tenth of a day rather than something smaller. Inside a block
        // the series is a polynomial in the instant, so a central
        // difference is exact whatever the step, and the only error is
        // the cancellation in `jd + h` minus `jd - h` — which near
        // Julian day 2.45 million is what decides the answer. At 1e-4
        // this check disagreed with itself by 5e-6; at a tenth of a day
        // it is a thousand times better.
        let step = 0.1;
        for offset in [0.3, 0.9, 1.0, 1.5_f64] {
            let jd = table.start_jd + offset;
            let (_, rate) = table.at(jd).expect("inside the span");
            let (ahead, _) = table.at(jd + step).expect("inside the span");
            let (behind, _) = table.at(jd - step).expect("inside the span");
            for axis in 0..3 {
                let difference = (ahead[axis] - behind[axis]) / (2.0 * step);
                assert!(
                    (rate[axis] - difference).abs() < 1e-8,
                    "axis {axis} at {offset}: analytic {} against {difference}",
                    rate[axis]
                );
            }
        }
    }

    /// Outside the fit it refuses. A Chebyshev series does not degrade
    /// beyond its interval, it diverges, so an extrapolated answer would
    /// be worse than no answer and would look like one.
    #[test]
    fn outside_the_span_it_refuses_rather_than_extrapolates() {
        static COEFFICIENTS: [f64; 6] = [1.0, 0.0, 2.0, 0.0, 3.0, 0.0];
        let table = Table {
            start_jd: 2_451_545.0,
            interval_days: 10.0,
            terms: 2,
            coefficients: &COEFFICIENTS,
        };
        assert_eq!(table.intervals(), 1);
        assert!(table.at(2_451_544.9).is_none());
        assert!(table.at(2_451_555.1).is_none());
        assert!(table.at(2_451_545.0).is_some());
        // The last instant belongs to the last block rather than to one
        // past the end.
        assert!(table.at(2_451_555.0).is_some());
    }

    /// An empty table covers nothing and answers nothing, rather than
    /// dividing by zero on the way to finding out.
    #[test]
    fn an_empty_table_is_empty_rather_than_a_panic() {
        let empty = Table {
            start_jd: 2_451_545.0,
            interval_days: 10.0,
            terms: 0,
            coefficients: &[],
        };
        assert_eq!(empty.intervals(), 0);
        assert!(!empty.covers(2_451_545.0));
        assert!(empty.at(2_451_545.0).is_none());
    }
}
