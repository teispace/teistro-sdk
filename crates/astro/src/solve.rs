//! The shared boundary solver: when does a quantity that advances around a
//! circle next reach a target? One kernel for every ingress-shaped search
//! (a sankranti, a sign change, a tithi boundary), so the SDK has one set
//! of caps, one tolerance contract and one convergence proof.
//!
//! The kernel is bracket-then-refine: it jumps toward the target by the
//! quantity's mean rate (the mean rate is a fine first guess when the true
//! rate never strays far from it, which holds for the Sun and for the
//! Moon's elongation), narrows a bracket in which the signed gap changes
//! sign, and bisects it to the tolerance. Bisection halves a day-wide
//! bracket to a microsecond in 37 steps and needs no derivative, so the
//! same code serves a tabular classical model and a modern ephemeris. Every
//! loop has a cap; an unmet cap is an error, never a spin.
//!
//! ```
//! use teistro_astro::solve::{next_crossing, Caps};
//!
//! // A body moving at exactly one degree a day from 350° reaches 0° ten
//! // days later.
//! let angle = |t: f64| -> Result<f64, ()> { Ok((350.0 + t).rem_euclid(360.0)) };
//! let crossing = next_crossing(angle, 0.0, 0.0, 1.0, 1e-9, Caps::DEFAULT).expect("bracketed");
//! assert!((crossing.instant - 10.0).abs() < 1e-8);
//! ```

use core::fmt;

use teistro_core::angle::difference_deg;

/// How many steps a search may take before it is a defect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Caps {
    /// Steps by the mean rate to bracket the crossing.
    pub bracket_steps: u32,
    /// Bisections of the bracket.
    pub refinements: u32,
}

impl Caps {
    /// Sixty-four steps each way: a bracket from half a circle away takes
    /// a handful of jumps, and sixty-four bisections of a day exceed `f64`'s
    /// resolution, so a search that needs more is not converging.
    pub const DEFAULT: Caps = Caps {
        bracket_steps: 64,
        refinements: 64,
    };
}

impl Default for Caps {
    fn default() -> Caps {
        Caps::DEFAULT
    }
}

/// A found crossing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Crossing {
    /// The instant, in the unit the search was given (days).
    pub instant: f64,
    /// The final bracket width, at most the tolerance.
    pub width: f64,
    /// How many times the quantity was evaluated.
    pub evaluations: u32,
}

/// Why a search failed.
#[derive(Clone, Debug, PartialEq)]
pub enum SolveError<E> {
    /// The quantity could not be evaluated.
    Evaluation(E),
    /// The rate or the tolerance was not a positive finite number.
    Argument {
        /// Which argument.
        name: &'static str,
        /// Its value.
        value: f64,
    },
    /// The crossing was not bracketed within the cap.
    NotBracketed {
        /// The steps taken.
        steps: u32,
        /// The last instant tried.
        last: f64,
    },
    /// The bracket did not narrow to the tolerance within the cap.
    NotConverged {
        /// The bisections done.
        steps: u32,
        /// The bracket width reached.
        width: f64,
    },
}

impl<E: fmt::Display> fmt::Display for SolveError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SolveError::Evaluation(e) => write!(f, "the quantity could not be evaluated: {e}"),
            SolveError::Argument { name, value } => {
                write!(f, "{name} must be a positive finite number, not {value}")
            }
            SolveError::NotBracketed { steps, last } => {
                write!(
                    f,
                    "no crossing bracketed in {steps} steps (last instant {last})"
                )
            }
            SolveError::NotConverged { steps, width } => {
                write!(f, "bracket still {width} wide after {steps} bisections")
            }
        }
    }
}

impl<E: fmt::Debug + fmt::Display> std::error::Error for SolveError<E> {}

/// The signed gap from `target` to `angle`, in (-180, 180].
fn gap(angle: f64, target: f64) -> f64 {
    difference_deg(angle, target)
}

/// Bisects a bracket in which `f` is negative at `lo` and non-negative at
/// `hi` until it is narrower than the tolerance; `f` is assumed to change
/// sign once inside it.
fn bisect<E>(
    mut f: impl FnMut(f64) -> Result<f64, SolveError<E>>,
    mut lo: f64,
    mut hi: f64,
    tolerance: f64,
    caps: Caps,
) -> Result<(f64, f64), SolveError<E>> {
    let mut refinements = 0u32;
    while hi - lo > tolerance {
        if refinements >= caps.refinements {
            return Err(SolveError::NotConverged {
                steps: refinements,
                width: hi - lo,
            });
        }
        refinements += 1;
        let mid = lo + (hi - lo) * 0.5;
        if mid <= lo || mid >= hi {
            break; // `f64` cannot split the bracket further.
        }
        if f(mid)? < 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Ok((lo, hi))
}

/// The first zero of `f` inside `[from, to]`, found by stepping from
/// `from` until the sign of `f` changes in the wanted direction (upward:
/// negative to non-negative; downward: the reverse) and bisecting that
/// step to the tolerance. `None` when no such change occurs inside the
/// window: the search reports absence rather than guessing, which is what
/// a polar day or a circumpolar body needs.
///
/// ```
/// use teistro_astro::solve::{first_zero, Caps};
///
/// // A quantity that climbs through zero a third of the way in.
/// let f = |t: f64| -> Result<f64, ()> { Ok(t - 1.0 / 3.0) };
/// let zero = first_zero(f, 0.0, 1.0, 0.1, true, 1e-9, Caps::DEFAULT).expect("evaluated").expect("crossed");
/// assert!((zero.instant - 1.0 / 3.0).abs() < 1e-8);
/// ```
///
/// # Errors
///
/// A non-positive or non-finite step or tolerance, an evaluation error,
/// or a bisection that does not converge.
pub fn first_zero<E>(
    mut f: impl FnMut(f64) -> Result<f64, E>,
    from: f64,
    to: f64,
    step: f64,
    upward: bool,
    tolerance: f64,
    caps: Caps,
) -> Result<Option<Crossing>, SolveError<E>> {
    for (name, value) in [("step", step), ("tolerance", tolerance)] {
        if !(value.is_finite() && value > 0.0) {
            return Err(SolveError::Argument { name, value });
        }
    }
    let mut evaluations = 0u32;
    // The signed quantity, negated for a downward crossing so the bracket
    // is always negative at its start.
    let mut evaluate = |t: f64| -> Result<f64, SolveError<E>> {
        evaluations += 1;
        f(t).map(|v| if upward { v } else { -v })
            .map_err(SolveError::Evaluation)
    };
    let mut lo = from;
    let mut g_lo = evaluate(lo)?;
    let mut steps = 0u32;
    while lo < to {
        if steps >= caps.bracket_steps {
            return Err(SolveError::NotBracketed { steps, last: lo });
        }
        steps += 1;
        let hi = (lo + step).min(to);
        let g_hi = evaluate(hi)?;
        if g_lo < 0.0 && g_hi >= 0.0 {
            let (a, b) = bisect(&mut evaluate, lo, hi, tolerance, caps)?;
            return Ok(Some(Crossing {
                instant: a + (b - a) * 0.5,
                width: b - a,
                evaluations,
            }));
        }
        lo = hi;
        g_lo = g_hi;
    }
    Ok(None)
}

/// The first instant at or after `from` at which `angle` (degrees, wrapping
/// at 360, advancing on average at `rate_deg_per_day`) reaches `target`
/// going forward, to within `tolerance_days`.
///
/// The angle at `from` should be behind the target; if it is already at or
/// past it, the search moves to just before the next crossing a circle
/// ahead, assuming the true rate stays within a tenth of the mean.
///
/// # Errors
///
/// A non-positive or non-finite rate or tolerance, an evaluation error,
/// or a cap reached.
pub fn next_crossing<E>(
    mut angle: impl FnMut(f64) -> Result<f64, E>,
    target_deg: f64,
    from: f64,
    rate_deg_per_day: f64,
    tolerance_days: f64,
    caps: Caps,
) -> Result<Crossing, SolveError<E>> {
    for (name, value) in [("rate", rate_deg_per_day), ("tolerance", tolerance_days)] {
        if !(value.is_finite() && value > 0.0) {
            return Err(SolveError::Argument { name, value });
        }
    }
    let mut evaluations = 0u32;
    let mut evaluate = |t: f64| -> Result<f64, SolveError<E>> {
        evaluations += 1;
        angle(t)
            .map(|a| gap(a, target_deg))
            .map_err(SolveError::Evaluation)
    };

    let mut lo = from;
    let mut g_lo = evaluate(lo)?;
    if g_lo >= 0.0 {
        // Already past: the next crossing is a circle ahead. Land short of
        // it so the bracketing below approaches from behind.
        lo += (360.0 - g_lo) / rate_deg_per_day * 0.9;
        g_lo = evaluate(lo)?;
        if g_lo >= 0.0 {
            return Err(SolveError::NotBracketed { steps: 0, last: lo });
        }
    }

    // Bracket: jump by the mean rate while the gap is negative; once the
    // jump would be under a day, step a whole day so the bracket is never
    // narrower than the model's own noise.
    let mut hi;
    let mut steps = 0u32;
    loop {
        if steps >= caps.bracket_steps {
            return Err(SolveError::NotBracketed { steps, last: lo });
        }
        steps += 1;
        let jump = (-g_lo / rate_deg_per_day).max(1.0);
        hi = lo + jump;
        let g_hi = evaluate(hi)?;
        if g_hi >= 0.0 {
            break;
        }
        lo = hi;
        g_lo = g_hi;
    }

    // Refine by bisection: the gap is negative at `lo` and non-negative at
    // `hi`, and monotone between them because the bracket spans less than
    // a circle of motion.
    let (lo, hi) = bisect(&mut evaluate, lo, hi, tolerance_days, caps)?;
    Ok(Crossing {
        instant: lo + (hi - lo) * 0.5,
        width: hi - lo,
        evaluations,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use proptest::prelude::*;

    use super::*;

    fn linear(rate: f64, start: f64) -> impl FnMut(f64) -> Result<f64, ()> {
        move |t| Ok((start + rate * t).rem_euclid(360.0))
    }

    #[test]
    fn a_linear_motion_is_found_to_the_tolerance() {
        let c =
            next_crossing(linear(0.9856, 10.0), 30.0, 0.0, 0.9856, 1e-9, Caps::DEFAULT).unwrap();
        assert!((c.instant - 20.0 / 0.9856).abs() < 1e-8, "{c:?}");
        assert!(c.width <= 1e-9);
        assert!(c.evaluations < 50, "{}", c.evaluations);
    }

    #[test]
    fn a_target_half_a_circle_away_needs_few_jumps() {
        let c = next_crossing(linear(1.0, 0.0), 180.0, 0.0, 1.0, 1e-6, Caps::DEFAULT).unwrap();
        assert!((c.instant - 180.0).abs() < 1e-5);
        assert!(c.evaluations < 40);
    }

    #[test]
    fn already_past_moves_a_circle_ahead() {
        let c = next_crossing(linear(1.0, 5.0), 0.0, 0.0, 1.0, 1e-6, Caps::DEFAULT).unwrap();
        assert!((c.instant - 355.0).abs() < 1e-5, "{c:?}");
    }

    #[test]
    fn bad_arguments_and_caps_are_errors() {
        assert!(matches!(
            next_crossing(linear(1.0, 0.0), 0.0, 0.0, 0.0, 1e-6, Caps::DEFAULT),
            Err(SolveError::Argument { name: "rate", .. })
        ));
        assert!(matches!(
            next_crossing(linear(1.0, 0.0), 0.0, 0.0, 1.0, f64::NAN, Caps::DEFAULT),
            Err(SolveError::Argument {
                name: "tolerance",
                ..
            })
        ));
        // A quantity that never advances is never bracketed.
        let stuck = |_: f64| -> Result<f64, ()> { Ok(10.0) };
        let err = next_crossing(stuck, 20.0, 0.0, 1.0, 1e-6, Caps::DEFAULT).unwrap_err();
        assert!(
            matches!(err, SolveError::NotBracketed { steps: 64, .. }),
            "{err:?}"
        );
        // A cap of one bisection cannot reach a microsecond.
        let tight = Caps {
            bracket_steps: 64,
            refinements: 1,
        };
        let err = next_crossing(linear(1.0, 0.0), 10.0, 0.0, 1.0, 1e-9, tight).unwrap_err();
        assert!(
            matches!(err, SolveError::NotConverged { steps: 1, .. }),
            "{err:?}"
        );
        let failing = |_: f64| -> Result<f64, &'static str> { Err("no data") };
        let err = next_crossing(failing, 10.0, 0.0, 1.0, 1e-9, Caps::DEFAULT).unwrap_err();
        assert_eq!(
            err.to_string(),
            "the quantity could not be evaluated: no data"
        );
    }

    proptest! {
        /// A perturbed motion (the mean rate plus a slow oscillation of
        /// up to a twentieth of it) is bracketed and refined so that the
        /// gap changes sign inside the tolerance around the answer.
        #[test]
        fn perturbed_motions_converge(
            rate in 0.5f64..15.0,
            start in 0.0f64..360.0,
            target in 0.0f64..360.0,
            amplitude in 0.0f64..0.05,
            phase in 0.0f64..core::f64::consts::TAU,
        ) {
            let motion = move |t: f64| -> Result<f64, ()> {
                Ok((start + rate * t + amplitude * 360.0 / core::f64::consts::TAU * (t * rate / 57.3 + phase).sin()).rem_euclid(360.0))
            };
            let c = next_crossing(motion, target, 0.0, rate, 1e-7, Caps::DEFAULT).unwrap();
            let before = gap(motion(c.instant - 2e-7).unwrap(), target);
            let after = gap(motion(c.instant + 2e-7).unwrap(), target);
            prop_assert!(before < 0.0, "before {before}");
            prop_assert!(after >= 0.0, "after {after}");
            prop_assert!(c.instant >= 0.0);
        }
    }
}
