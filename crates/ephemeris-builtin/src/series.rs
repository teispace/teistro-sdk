//! A periodic series and its sum.
//!
//! Every analytic theory the SDK ships — VSOP87 for the planets,
//! ELP/MPP02 for the Moon — is a sum of terms of the same shape, so one
//! type and one evaluator serve both, and a tier is a prefix of the
//! terms rather than a different kind of table.

/// One term of a coordinate's series: `amplitude · cos(phase +
/// frequency · t)`, scaled by `t` raised to [`Term::power`].
///
/// The units are the theory's own — astronomical units for a rectangular
/// VSOP87 coordinate, radians for a spherical one — and `t` is always in
/// Julian millennia from J2000, which is what the published files use.
///
/// ```
/// use teistro_ephemeris_builtin::series::Term;
///
/// // A constant term: no frequency, no power, so it is its amplitude.
/// let term = Term::new(2.5, 0.0, 0.0, 0);
/// assert!((term.at(0.0) - 2.5).abs() < 1e-15);
/// assert!((term.at(7.0) - 2.5).abs() < 1e-15, "and it does not move");
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Term {
    /// The term's amplitude, in the theory's own unit.
    pub amplitude: f64,
    /// The phase at J2000, in radians.
    pub phase: f64,
    /// The angular frequency, in radians per Julian millennium.
    pub frequency: f64,
    /// The power of `t` the term is scaled by; VSOP87 uses zero to five.
    pub power: u8,
}

impl Term {
    /// One term.
    #[must_use]
    pub const fn new(amplitude: f64, phase: f64, frequency: f64, power: u8) -> Term {
        Term {
            amplitude,
            phase,
            frequency,
            power,
        }
    }

    /// The term's contribution at `t`, in Julian millennia from J2000.
    #[must_use]
    pub fn at(self, t: f64) -> f64 {
        // `powi` rather than `powf`: the power is a small integer and
        // `powi` is exact for one, which the great majority of terms are.
        self.amplitude * f64::cos(self.phase + self.frequency * t) * t.powi(i32::from(self.power))
    }
}

/// The sum of a coordinate's terms at `t`, in Julian millennia from
/// J2000.
///
/// The terms carry their own power of `t`, so a coordinate is one flat
/// slice rather than a group per power. That is what lets a tier be a
/// prefix: sort by descending amplitude once, and every truncation is a
/// length.
///
/// ```
/// use teistro_ephemeris_builtin::series::{Term, sum};
///
/// let terms = [Term::new(1.0, 0.0, 0.0, 0), Term::new(0.5, 0.0, 0.0, 1)];
/// assert!((sum(&terms, 0.0) - 1.0).abs() < 1e-15);
/// assert!((sum(&terms, 2.0) - 2.0).abs() < 1e-15, "1 + 0.5·2");
/// ```
#[must_use]
pub fn sum(terms: &[Term], t: f64) -> f64 {
    terms.iter().map(|term| term.at(t)).sum()
}

/// Julian days in the millennium the time argument counts.
pub const DAYS_PER_MILLENNIUM: f64 = 365_250.0;

/// The epoch the time argument is measured from: J2000.0, as a Julian
/// day in Barycentric Dynamical Time.
pub const J2000: f64 = 2_451_545.0;

/// A Julian day as the time argument the series take.
///
/// ```
/// use teistro_ephemeris_builtin::series::{J2000, millennia};
///
/// assert!(millennia(J2000).abs() < 1e-15);
/// assert!((millennia(J2000 + 365_250.0) - 1.0).abs() < 1e-15);
/// ```
#[must_use]
pub fn millennia(jd: f64) -> f64 {
    (jd - J2000) / DAYS_PER_MILLENNIUM
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "a test fails by panicking"
    )]

    use super::*;

    #[test]
    fn a_term_is_its_amplitude_at_the_phase_it_names() {
        let term = Term::new(3.0, std::f64::consts::FRAC_PI_2, 0.0, 0);
        assert!(term.at(0.0).abs() < 1e-15, "cos(pi/2) is nothing");
    }

    #[test]
    fn a_powered_term_vanishes_at_the_epoch() {
        for power in 1..=5 {
            let term = Term::new(1.0, 0.0, 0.0, power);
            assert!(term.at(0.0).abs() < 1e-15, "t^{power} is nothing at t = 0");
        }
    }

    /// The published files carry each term twice: as `A·cos(B + C·t)`
    /// and as `S·sin(phi) + K·cos(phi)`. The evaluator uses the first,
    /// so it has to satisfy the algebra that relates them.
    ///
    /// Expanding `A·cos(B + x)` gives `(A cos B)·cos x − (A sin B)·sin
    /// x`, so the pair is `K = A cos B` and `S = −A sin B` — which makes
    /// `A = hypot(S, K)` and **`B = atan2(−S, K)`**, with the minus sign
    /// that is easy to drop and impossible to notice afterwards.
    ///
    /// This is the identity and not a check of the files: a file's `B`
    /// is the constant of `phi = Σ a(i)·λ(i)`, which carries the mean
    /// longitudes' own constants and cannot be rebuilt from `S` and `K`.
    /// What can be checked against a file is the amplitude alone, and
    /// the ingester does that.
    #[test]
    fn the_amplitude_form_is_the_sine_and_cosine_form() {
        for (sine, cosine) in [
            (-0.000_015_222_62_f64, 0.999_829_288_33_f64),
            (0.008_140_547_03, -0.001_870_018_68),
            (1.0, 0.0),
            (0.0, -1.0),
            (-0.3, -0.7),
        ] {
            let amplitude = sine.hypot(cosine);
            let phase = (-sine).atan2(cosine);
            let frequency = 6_283.075_849_991_40;
            let term = Term::new(amplitude, phase, frequency, 0);
            for t in [-1.5, -0.2, 0.0, 0.13, 2.4] {
                let angle = frequency * t;
                let expected = sine * angle.sin() + cosine * angle.cos();
                assert!(
                    (term.at(t) - expected).abs() < 1e-12,
                    "S={sine} K={cosine} t={t}: {} against {expected}",
                    term.at(t)
                );
            }
        }
    }

    #[test]
    fn a_sum_of_nothing_is_nothing() {
        assert!(sum(&[], 1.0).abs() < 1e-15);
    }
}
