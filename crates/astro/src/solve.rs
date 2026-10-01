//! The shared boundary solver: when does a quantity that advances around a
//! circle next reach a target? One kernel for every ingress-shaped search
//! (a sankranti, a sign change, a tithi boundary), so the SDK has one set
//! of caps, one tolerance contract and one convergence proof.
//!
//! The kernel is bracket-then-refine: it jumps toward the target by the
//! quantity's mean rate (the mean rate is a fine first guess when the true
//! rate never strays far from it, which holds for the Sun and for the
//! Moon's elongation), finds a bracket in which the signed gap changes
//! sign, and narrows it to the tolerance by the ITP method (interpolate,
//! truncate, project): a regula falsi estimate pulled toward the midpoint
//! and confined to the interval a bisection would have reached, so a
//! smooth curve converges superlinearly (a day-wide bracket to a tenth of
//! a millisecond in six or seven steps) while no curve costs more than a
//! bisection plus one step (a day to a microsecond in 38). It needs no
//! derivative, so the same code serves a tabular classical model and a
//! modern ephemeris. Every loop has a cap; an unmet cap is an error, never
//! a spin.
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
use teistro_core::error::{Error, Status};
use teistro_core::math;

use teistro_core::angle::difference_deg;

/// How many steps a search may take before it is a defect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Caps {
    /// Steps by the mean rate to bracket the crossing.
    pub bracket_steps: u32,
    /// Narrowing steps of the bracket.
    pub refinements: u32,
}

impl Caps {
    /// Sixty-four steps each way: a bracket from half a circle away takes
    /// a handful of jumps, and sixty-four halvings of a day exceed `f64`'s
    /// resolution (the narrowing never needs more than a bisection and
    /// one), so a search that needs more is not converging.
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
        /// The steps done.
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
                write!(f, "bracket still {width} wide after {steps} steps")
            }
        }
    }
}

impl<E: fmt::Debug + fmt::Display> std::error::Error for SolveError<E> {}

impl SolveError<Error> {
    /// The SDK error a failed search becomes. An evaluation's own error
    /// passes through whole, so a provider's refusal keeps its kind and its
    /// field; anything else is `NotConverged`, naming what was sought.
    ///
    /// ```
    /// use teistro_astro::solve::SolveError;
    /// use teistro_core::error::{Error, Status};
    ///
    /// let stuck = SolveError::<Error>::NotConverged { steps: 64, width: 0.5 };
    /// let error = stuck.into_error(|| "the new moon after JD 2451545".into());
    /// assert_eq!(error.status, Status::NotConverged);
    /// assert!(error.message.starts_with("the new moon after JD 2451545 was not found"));
    /// ```
    #[must_use]
    pub fn into_error(self, sought: impl FnOnce() -> String) -> Error {
        match self {
            SolveError::Evaluation(inner) => inner,
            other => Error::new(
                Status::NotConverged,
                format!("{} was not found: {other}", sought()),
            ),
        }
    }
}

/// The signed gap from `target` to `angle`, in (-180, 180].
fn gap(angle: f64, target: f64) -> f64 {
    difference_deg(angle, target)
}

/// A bracket: `f` is negative at `lo` and non-negative at `hi`.
#[derive(Clone, Copy, Debug)]
struct Bracket {
    lo: f64,
    f_lo: f64,
    hi: f64,
    f_hi: f64,
}

/// How far the interpolated estimate is pulled toward the midpoint: this
/// fraction of the bracket's width, times the width's ratio to the first
/// width (the method's `κ₁` with `κ₂ = 2`, made independent of the unit).
const PULL: f64 = 0.1;

/// The steps the narrowing may take beyond a bisection's count (`n₀`).
const SLACK: i32 = 1;

/// How many halvings take `width` down to the tolerance.
fn halvings(width: f64, tolerance: f64) -> i32 {
    let mut count = 0;
    let mut remaining = width;
    while remaining > tolerance {
        remaining *= 0.5;
        count += 1;
    }
    count
}

/// The crossing at the middle of a narrowed bracket.
fn crossing((lo, hi): (f64, f64), evaluations: u32) -> Crossing {
    Crossing {
        instant: lo + (hi - lo) * 0.5,
        width: hi - lo,
        evaluations,
    }
}

/// Narrows a bracket to the tolerance by the ITP method (Oliveira and
/// Takahashi, "An enhancement of the bisection method average performance
/// preserving minmax optimality", ACM Transactions on Mathematical
/// Software 47, 2021): each step interpolates the secant's root, truncates
/// it toward the midpoint by a pull that shrinks with the square of the
/// width, and projects it into the interval a bisection would have reached
/// by that step. A smooth curve converges superlinearly; no curve takes
/// more than [`SLACK`] steps over a bisection's count. Returns the final
/// `(lo, hi)`.
fn narrow<E>(
    mut f: impl FnMut(f64) -> Result<f64, SolveError<E>>,
    bracket: Bracket,
    tolerance: f64,
    caps: Caps,
) -> Result<(f64, f64), SolveError<E>> {
    let Bracket {
        mut lo,
        mut f_lo,
        mut hi,
        mut f_hi,
    } = bracket;
    let first_width = hi - lo;
    // The steps a bisection would still have: the interval it would have
    // reached by now is the tolerance grown back by this many halvings.
    let mut budget = halvings(first_width, tolerance) + SLACK;
    let mut steps = 0u32;
    while hi - lo > tolerance {
        if steps >= caps.refinements {
            return Err(SolveError::NotConverged {
                steps,
                width: hi - lo,
            });
        }
        steps += 1;
        let width = hi - lo;
        let mid = lo + width * 0.5;
        if mid <= lo || mid >= hi {
            break; // `f64` cannot split the bracket further.
        }
        // Interpolate: the secant's root, inside the bracket because the
        // values differ in sign.
        let secant = (f_hi * lo - f_lo * hi) / (f_hi - f_lo);
        // Truncate: toward the midpoint, never past it.
        let toward = if secant <= mid { 1.0 } else { -1.0 };
        let pull = PULL * width * (width / first_width);
        let truncated = if pull <= (mid - secant).abs() {
            secant + toward * pull
        } else {
            mid
        };
        // Project: into the interval a bisection would have reached.
        let radius = (tolerance * 0.5 * math::powi(2f64, budget) - width * 0.5).max(0.0);
        let projected = if (truncated - mid).abs() <= radius {
            truncated
        } else {
            mid - toward * radius
        };
        // Floating point: an estimate that rounds onto an end of the bracket
        // would step without narrowing, so every step lands at least a
        // quarter of the tolerance inside (toward the middle, so the
        // projection's guarantee stands).
        let least = tolerance * 0.25;
        let estimate = projected.max(lo + least).min(hi - least);
        budget = budget.saturating_sub(1);
        let value = f(estimate)?;
        if value < 0.0 {
            lo = estimate;
            f_lo = value;
        } else {
            hi = estimate;
            f_hi = value;
        }
    }
    Ok((lo, hi))
}

/// The first zero of `f` inside `[from, to]`, found by stepping from
/// `from` until the sign of `f` changes in the wanted direction (upward:
/// negative to non-negative; downward: the reverse) and narrowing that
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
/// or a bracket that does not narrow within the cap.
pub fn first_zero<E>(
    f: impl FnMut(f64) -> Result<f64, E>,
    from: f64,
    to: f64,
    step: f64,
    upward: bool,
    tolerance: f64,
    caps: Caps,
) -> Result<Option<Crossing>, SolveError<E>> {
    // One kernel, two front doors: a walk is a grid of one, so this is
    // [`first_zero_gridded`] with the chunk at one rather than a second
    // copy of the same loop.
    let f = core::cell::RefCell::new(f);
    let scan = Scan::new(from, to, step, upward, tolerance)
        .with_caps(caps)
        .with_chunk(1);
    first_zero_gridded(
        |instants, out| {
            out.clear();
            out.reserve(instants.len());
            for at in instants {
                out.push((f.borrow_mut())(*at)?);
            }
            Ok(())
        },
        |at| (f.borrow_mut())(at),
        &scan,
    )
}

/// How many of a bracket scan's instants are asked for in one grid by
/// default.
///
/// Unlike a lattice search, a bracket scan **stops at the first sign
/// change**, so a chunk can be asked for and not used: the waste is at
/// most one chunk less one. Thirty-two is sized from what the scan is
/// for. A rise or a set the iteration could not settle is found within a
/// few steps of where the iteration left it; an *absence* — the case
/// that dominates, because proving a window holds no more events is what
/// the almanac's event loop does twice a day — walks the whole window
/// and finds nothing, and there thirty-two turns a hundred and forty-four
/// round trips into five.
pub const SCAN_CHUNK: usize = 32;

/// Where a bracket scan looks, and how it walks.
///
/// The kernel took seven positional numbers and would have grown to nine
/// when the grid arrived; this is them, named. A caller writes what it
/// means and takes the defaults for the rest.
///
/// ```
/// use teistro_astro::solve::{Caps, Scan};
///
/// // From here to a day on, ten-minute steps, the upward crossing.
/// let scan = Scan::new(0.0, 1.0, 1.0 / 144.0, true, 1e-7).with_chunk(64);
/// assert_eq!(scan.chunk, 64);
/// assert_eq!(scan.caps, Caps::DEFAULT);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scan {
    /// Where the walk starts.
    pub from: f64,
    /// Where it stops.
    pub to: f64,
    /// The step it takes.
    pub step: f64,
    /// Whether the crossing wanted goes upward: negative to non-negative.
    pub upward: bool,
    /// The tolerance the crossing is narrowed to.
    pub tolerance: f64,
    /// The caps on the walk and the narrowing.
    pub caps: Caps,
    /// How many instants to ask for at once; one is a walk.
    pub chunk: usize,
}

impl Scan {
    /// A scan with the default caps and chunk.
    #[must_use]
    pub const fn new(from: f64, to: f64, step: f64, upward: bool, tolerance: f64) -> Scan {
        Scan {
            from,
            to,
            step,
            upward,
            tolerance,
            caps: Caps::DEFAULT,
            chunk: SCAN_CHUNK,
        }
    }

    /// The caps instead of the default.
    #[must_use]
    pub const fn with_caps(mut self, caps: Caps) -> Scan {
        self.caps = caps;
        self
    }

    /// How many instants to ask for at once. Zero is read as one.
    #[must_use]
    pub const fn with_chunk(mut self, instants: usize) -> Scan {
        self.chunk = if instants == 0 { 1 } else { instants };
        self
    }
}

/// [`first_zero`] over a source that can answer many instants at once.
///
/// The scan walks its window in fixed steps and knows every instant it
/// will visit before it visits the first, so it asks for them a grid at
/// a time; only the narrowing stays serial, because each of its steps is
/// chosen from the answer to the last. The instants are produced by the
/// walk's own recurrence, so the grid asks for exactly what the walk
/// asked for and the values it reads are the values the walk read: the
/// crossing is the same crossing, to the bit.
///
/// `sample` is given a slice of instants and writes their values into
/// the buffer in the order asked; `refine_at` reads one instant, for the
/// narrowing, which cannot be asked for in advance.
///
/// # Errors
///
/// As [`first_zero`], and `Argument` when a source answers a different
/// number of instants from the number it was asked for.
pub fn first_zero_gridded<E>(
    mut sample: impl FnMut(&[f64], &mut Vec<f64>) -> Result<(), E>,
    mut refine_at: impl FnMut(f64) -> Result<f64, E>,
    scan: &Scan,
) -> Result<Option<Crossing>, SolveError<E>> {
    let Scan {
        from,
        to,
        step,
        upward,
        tolerance,
        caps,
        chunk,
    } = *scan;
    for (name, value) in [("step", step), ("tolerance", tolerance)] {
        if !(value.is_finite() && value > 0.0) {
            return Err(SolveError::Argument { name, value });
        }
    }
    let chunk = chunk.max(1);
    let sign = |value: f64| if upward { value } else { -value };
    let mut evaluations = 0u32;
    let mut instants: Vec<f64> = Vec::with_capacity(chunk);
    let mut values: Vec<f64> = Vec::with_capacity(chunk);

    let mut previous: Option<(f64, f64)> = None;
    let mut cursor = from;
    let mut steps = 0u32;
    loop {
        instants.clear();
        if previous.is_none() {
            instants.push(cursor);
        }
        while instants.len() < chunk && cursor < to {
            if steps >= caps.bracket_steps {
                return Err(SolveError::NotBracketed {
                    steps,
                    last: cursor,
                });
            }
            steps += 1;
            cursor = (cursor + step).min(to);
            instants.push(cursor);
        }
        if instants.is_empty() {
            break;
        }
        sample(&instants, &mut values).map_err(SolveError::Evaluation)?;
        if values.len() != instants.len() {
            return Err(SolveError::Argument {
                name: "grid",
                value: f64::from(u32::try_from(values.len()).unwrap_or(u32::MAX)),
            });
        }
        for (at, raw) in instants.iter().copied().zip(values.iter().copied()) {
            evaluations += 1;
            let g_hi = sign(raw);
            let Some((lo, g_lo)) = previous else {
                previous = Some((at, g_hi));
                continue;
            };
            if g_lo < 0.0 && g_hi >= 0.0 {
                let bracket = Bracket {
                    lo,
                    f_lo: g_lo,
                    hi: at,
                    f_hi: g_hi,
                };
                let mut narrowing = |t: f64| -> Result<f64, SolveError<E>> {
                    evaluations += 1;
                    refine_at(t).map(sign).map_err(SolveError::Evaluation)
                };
                let narrowed = narrow(&mut narrowing, bracket, tolerance, caps)?;
                return Ok(Some(crossing(narrowed, evaluations)));
            }
            previous = Some((at, g_hi));
        }
        if cursor >= to {
            break;
        }
    }
    Ok(None)
}

/// Narrows a bracket to the tolerance by the shared method: `f` must be
/// negative at `lo` and non-negative at `hi`. For a caller that has found
/// its own bracket, so that every search in the SDK converges the same way.
///
/// ```
/// use teistro_astro::solve::{refine, Caps};
///
/// // A sine through zero at 0.3 of a day, from a day-wide bracket to a
/// // tenth of a millisecond in a handful of evaluations.
/// let curve = |t: f64| -> Result<f64, ()> { Ok(((t - 0.3) * 1.5).sin()) };
/// let crossing = refine(curve, 0.0, 1.0, 1e-9, Caps::DEFAULT).expect("bracketed");
/// assert!((crossing.instant - 0.3).abs() < 1e-9);
/// assert!(crossing.evaluations <= 9);
/// ```
///
/// # Errors
///
/// A non-positive or non-finite tolerance, a bracket without the sign
/// change, an evaluation error, or a bracket that does not narrow within
/// the cap.
pub fn refine<E>(
    mut f: impl FnMut(f64) -> Result<f64, E>,
    lo: f64,
    hi: f64,
    tolerance: f64,
    caps: Caps,
) -> Result<Crossing, SolveError<E>> {
    if !(tolerance.is_finite() && tolerance > 0.0) {
        return Err(SolveError::Argument {
            name: "tolerance",
            value: tolerance,
        });
    }
    let f_lo = f(lo).map_err(SolveError::Evaluation)?;
    let f_hi = f(hi).map_err(SolveError::Evaluation)?;
    let narrowed = refine_known(f, (lo, f_lo), (hi, f_hi), tolerance, caps)?;
    Ok(Crossing {
        evaluations: narrowed.evaluations + 2,
        ..narrowed
    })
}

/// [`refine`] for a caller that already holds `f` at both ends of its
/// bracket, as a scan does at the two samples that straddle a line: the
/// narrowing is the same step for step, and the two evaluations of the
/// ends are not paid again. `evaluations` counts the narrowing's alone.
///
/// ```
/// use teistro_astro::solve::{refine, refine_known, Caps};
///
/// let curve = |t: f64| -> Result<f64, ()> { Ok(((t - 0.3) * 1.5).sin()) };
/// let ends = ((0.0, curve(0.0).unwrap()), (1.0, curve(1.0).unwrap()));
/// let known = refine_known(curve, ends.0, ends.1, 1e-9, Caps::DEFAULT).expect("bracketed");
/// let asked = refine(curve, 0.0, 1.0, 1e-9, Caps::DEFAULT).expect("bracketed");
/// assert_eq!(known.instant.to_bits(), asked.instant.to_bits());
/// assert_eq!(known.evaluations + 2, asked.evaluations);
/// ```
///
/// # Errors
///
/// As [`refine`].
pub fn refine_known<E>(
    mut f: impl FnMut(f64) -> Result<f64, E>,
    (lo, f_lo): (f64, f64),
    (hi, f_hi): (f64, f64),
    tolerance: f64,
    caps: Caps,
) -> Result<Crossing, SolveError<E>> {
    if !(tolerance.is_finite() && tolerance > 0.0) {
        return Err(SolveError::Argument {
            name: "tolerance",
            value: tolerance,
        });
    }
    if !(f_lo < 0.0 && f_hi >= 0.0) {
        return Err(SolveError::NotBracketed { steps: 0, last: lo });
    }
    let mut evaluations = 0u32;
    let mut evaluate = |t: f64| -> Result<f64, SolveError<E>> {
        evaluations += 1;
        f(t).map_err(SolveError::Evaluation)
    };
    let bracket = Bracket { lo, f_lo, hi, f_hi };
    let narrowed = narrow(&mut evaluate, bracket, tolerance, caps)?;
    Ok(crossing(narrowed, evaluations))
}

/// One end of a bracket with the quantity's rate there: the instant, the
/// signed value and its derivative in the same unit per unit of time.
pub type Sloped = (f64, f64, f64);

/// How many polynomial steps the seed may take: a cubic on the unit
/// interval halves at worst, and fifty-two halvings exhaust `f64`.
const SEED_STEPS: u32 = 60;

/// The root of the cubic that has the bracket's two values and two rates
/// at its ends (the cubic Hermite interpolant), as a fraction of the way
/// from `lo` to `hi`: where the curve would cross if it bent no more than
/// its ends say. Found on the polynomial alone, with no evaluation of the
/// quantity, by Newton's method kept inside a bracket that bisects when a
/// step would leave it. The secant's root when a rate is not finite.
fn hermite_root((lo, f_lo, d_lo): Sloped, (hi, f_hi, d_hi): Sloped) -> f64 {
    let secant = f_lo / (f_lo - f_hi);
    let width = hi - lo;
    let (m_lo, m_hi) = (d_lo * width, d_hi * width);
    if !(m_lo.is_finite() && m_hi.is_finite()) {
        return secant;
    }
    // p(s) = f_lo + m_lo·s + c2·s² + c3·s³ with p(1) = f_hi, p'(1) = m_hi.
    let c2 = 3.0 * (f_hi - f_lo) - 2.0 * m_lo - m_hi;
    let c3 = 2.0 * (f_lo - f_hi) + m_lo + m_hi;
    let value = |s: f64| ((c3 * s + c2) * s + m_lo) * s + f_lo;
    let slope = |s: f64| (3.0 * c3 * s + 2.0 * c2) * s + m_lo;
    let (mut below, mut above) = (0.0f64, 1.0f64);
    let mut fraction = secant;
    for _ in 0..SEED_STEPS {
        if value(fraction) < 0.0 {
            below = fraction;
        } else {
            above = fraction;
        }
        let newton = fraction - value(fraction) / slope(fraction);
        let next = if newton > below && newton < above {
            newton
        } else {
            below + (above - below) * 0.5
        };
        if (next - fraction).abs() <= f64::EPSILON || above - below <= f64::EPSILON {
            return next;
        }
        fraction = next;
    }
    fraction
}

/// [`refine_known`] for a quantity whose **rate** comes with its value,
/// as an ephemeris gives a longitude's speed with the longitude: `f`
/// answers `(value, rate)`, and each end carries its rate as well.
///
/// The first estimate is the root of the cubic through both ends' values
/// and rates, which for a body's longitude over a scan's step is already
/// within a few tenths of a second of the crossing; each estimate after it
/// is a Newton step from the last, confined to the bracket, and a step
/// that would leave the bracket or fail to halve the one before it is a
/// bisection instead (the safeguard of Numerical Recipes' `rtsafe`). The
/// search ends as [`refine_known`]'s does, on a bracket at most the
/// tolerance wide whose ends were **evaluated** on either side of the
/// line, so a rate that is wrong, or absent (not finite), costs steps and
/// never the answer: once a Newton step is shorter than half the
/// tolerance, the next evaluation is placed a quarter of the tolerance
/// past it, where it closes the bracket if the step was right. The
/// instant reported is the Newton step from the nearer end, inside that
/// bracket.
///
/// ```
/// use teistro_astro::solve::{refine_known, refine_with_rates, Caps};
///
/// let curve = |t: f64| -> Result<(f64, f64), ()> { Ok((((t - 0.3) * 1.5).sin(), 1.5 * ((t - 0.3) * 1.5).cos())) };
/// let (lo, hi) = (curve(0.0).unwrap(), curve(1.0).unwrap());
/// let seeded = refine_with_rates(curve, (0.0, lo.0, lo.1), (1.0, hi.0, hi.1), 1e-9, Caps::DEFAULT).expect("bracketed");
/// let known = refine_known(|t| curve(t).map(|(v, _)| v), (0.0, lo.0), (1.0, hi.0), 1e-9, Caps::DEFAULT).expect("bracketed");
/// assert!((seeded.instant - 0.3).abs() < 1e-9 && seeded.width <= 1e-9);
/// assert!(seeded.evaluations < known.evaluations);
/// ```
///
/// # Errors
///
/// As [`refine`].
pub fn refine_with_rates<E>(
    mut f: impl FnMut(f64) -> Result<(f64, f64), E>,
    lo: Sloped,
    hi: Sloped,
    tolerance: f64,
    caps: Caps,
) -> Result<Crossing, SolveError<E>> {
    if !(tolerance.is_finite() && tolerance > 0.0) {
        return Err(SolveError::Argument {
            name: "tolerance",
            value: tolerance,
        });
    }
    if !(lo.1 < 0.0 && hi.1 >= 0.0) {
        return Err(SolveError::NotBracketed {
            steps: 0,
            last: lo.0,
        });
    }
    let (mut lo, mut hi) = (lo, hi);
    let least = tolerance * 0.25;
    let mut estimate = lo.0 + (hi.0 - lo.0) * hermite_root(lo, hi);
    // The step before last, for the safeguard: a Newton step must at
    // least halve it or give way to a bisection.
    let mut before = hi.0 - lo.0;
    // Whether the last evaluation was a probe placed to close the
    // bracket: one that failed to means the rate misled the step, so the
    // next is a bisection, and a wrong rate costs at most every other
    // step's halving rather than creeping by a quarter tolerance at a
    // time.
    let mut probed = false;
    let mut evaluations = 0u32;
    while hi.0 - lo.0 > tolerance {
        if evaluations >= caps.refinements {
            return Err(SolveError::NotConverged {
                steps: evaluations,
                width: hi.0 - lo.0,
            });
        }
        let at = estimate.max(lo.0 + least).min(hi.0 - least);
        if at <= lo.0 || at >= hi.0 {
            break; // `f64` cannot split the bracket further.
        }
        evaluations += 1;
        let (value, rate) = f(at).map_err(SolveError::Evaluation)?;
        let end = (at, value, rate);
        if value < 0.0 {
            lo = end;
        } else {
            hi = end;
        }
        let step = -value / rate;
        let newton = at + step;
        // Converged by the step: the next evaluation closes the bracket a
        // quarter of the tolerance past the Newton point, on the far side
        // of the line from this one. Asked before the Newton point is
        // checked against the bracket, because a step shorter than the
        // instant's last bit lands on the end it started from.
        let converged = step.abs() < tolerance * 0.5;
        let trusted = !probed && step.abs() <= before * 0.5;
        probed = trusted && converged;
        estimate = if probed {
            newton + if value < 0.0 { least } else { -least }
        } else if trusted && newton > lo.0 && newton < hi.0 {
            before = step.abs();
            newton
        } else {
            before = (hi.0 - lo.0) * 0.5;
            lo.0 + before
        };
    }
    Ok(Crossing {
        instant: nearer_newton(lo, hi),
        width: hi.0 - lo.0,
        evaluations,
    })
}

/// The Newton step from whichever end of a narrowed bracket is nearer the
/// line by value, kept inside the bracket; its middle when that end's rate
/// is of no use.
fn nearer_newton(lo: Sloped, hi: Sloped) -> f64 {
    let (t, value, rate) = if -lo.1 < hi.1 { lo } else { hi };
    let newton = t - value / rate;
    if newton.is_finite() && newton >= lo.0 && newton <= hi.0 {
        newton
    } else {
        lo.0 + (hi.0 - lo.0) * 0.5
    }
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
    let mut steps = 0u32;
    let bracket = loop {
        if steps >= caps.bracket_steps {
            return Err(SolveError::NotBracketed { steps, last: lo });
        }
        steps += 1;
        let jump = (-g_lo / rate_deg_per_day).max(1.0);
        let hi = lo + jump;
        let g_hi = evaluate(hi)?;
        if g_hi >= 0.0 {
            break Bracket {
                lo,
                f_lo: g_lo,
                hi,
                f_hi: g_hi,
            };
        }
        lo = hi;
        g_lo = g_hi;
    };

    // Narrow: the gap is negative at `lo` and non-negative at `hi`, and
    // monotone between them because the bracket spans less than a circle
    // of motion.
    let narrowed = narrow(&mut evaluate, bracket, tolerance_days, caps)?;
    Ok(crossing(narrowed, evaluations))
}

/// A found minimum.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Minimum {
    /// Where the quantity is least, in the unit the search was given.
    pub instant: f64,
    /// The quantity there.
    pub value: f64,
    /// The final bracket width, at most the tolerance.
    pub width: f64,
    /// How many times the quantity was evaluated.
    pub evaluations: u32,
}

/// The golden section's interior point, `(√5 − 1) / 2`.
const GOLDEN: f64 = 0.618_033_988_749_894_9;

/// The least value of a quantity with **one** minimum between `lo` and
/// `hi`, narrowed by golden sections to `tolerance`: the companion of
/// [`refine`] for the searches that ask when something is closest (an
/// eclipse's greatest moment) rather than when it crosses a line. Each
/// step keeps one of its two interior points, so a narrowing costs one
/// evaluation a step, and needs no derivative.
///
/// A quantity with two minima in the bracket gets one of them; a caller
/// keeps the bracket inside the stretch where its quantity is unimodal.
///
/// ```
/// use teistro_astro::solve::{minimum, Caps};
///
/// // No constant added: a bowl lifted off zero is flat to the last bit
/// // within 1e-8 of its floor, and no search can see further than that.
/// let bowl = |t: f64| -> Result<f64, ()> { Ok((t - 0.3).powi(2)) };
/// let least = minimum(bowl, 0.0, 1.0, 1e-9, Caps::DEFAULT).expect("narrowed");
/// assert!((least.instant - 0.3).abs() < 1e-8);
/// assert!(least.value < 1e-15);
/// assert!(least.evaluations <= 46);
/// ```
///
/// # Errors
///
/// A non-positive or non-finite tolerance, a bracket that does not run
/// forward, an evaluation error, or a bracket that does not narrow within
/// the cap.
pub fn minimum<E>(
    mut quantity: impl FnMut(f64) -> Result<f64, E>,
    lo: f64,
    hi: f64,
    tolerance: f64,
    caps: Caps,
) -> Result<Minimum, SolveError<E>> {
    if !(tolerance.is_finite() && tolerance > 0.0) {
        return Err(SolveError::Argument {
            name: "tolerance",
            value: tolerance,
        });
    }
    if !(lo.is_finite() && hi.is_finite() && hi > lo) {
        return Err(SolveError::Argument {
            name: "bracket",
            value: hi - lo,
        });
    }
    let (mut low, mut high) = (lo, hi);
    let mut left = high - GOLDEN * (high - low);
    let mut right = low + GOLDEN * (high - low);
    let mut at_left = quantity(left).map_err(SolveError::Evaluation)?;
    let mut at_right = quantity(right).map_err(SolveError::Evaluation)?;
    let mut evaluations = 2;
    let mut steps = 0;
    while high - low > tolerance {
        if steps == caps.refinements {
            return Err(SolveError::NotConverged {
                steps,
                width: high - low,
            });
        }
        steps += 1;
        evaluations += 1;
        if at_left < at_right {
            high = right;
            right = left;
            at_right = at_left;
            left = high - GOLDEN * (high - low);
            at_left = quantity(left).map_err(SolveError::Evaluation)?;
        } else {
            low = left;
            left = right;
            at_left = at_right;
            right = low + GOLDEN * (high - low);
            at_right = quantity(right).map_err(SolveError::Evaluation)?;
        }
    }
    let (instant, value) = if at_left < at_right {
        (left, at_left)
    } else {
        (right, at_right)
    };
    Ok(Minimum {
        instant,
        value,
        width: high - low,
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
    fn a_smooth_curve_narrows_in_a_handful_of_steps() {
        // A sine from a day-wide bracket to a tenth of a millisecond: two
        // evaluations for the ends and a few steps, where a bisection would
        // take thirty.
        let curve = |t: f64| -> Result<f64, ()> { Ok(math::sin((t - 0.3) * 1.5)) };
        let c = refine(curve, 0.0, 1.0, 1e-9, Caps::DEFAULT).unwrap();
        assert!((c.instant - 0.3).abs() < 1e-9, "{c:?}");
        assert!(c.evaluations <= 9, "{}", c.evaluations);
    }

    #[test]
    fn a_step_costs_no_more_than_a_bisection_and_one() {
        // A discontinuity gives the interpolation nothing to work with:
        // the projection keeps the cost at the bisection's thirty halvings
        // plus the slack, after the two ends.
        let step = |t: f64| -> Result<f64, ()> { Ok(if t < 0.7 { -1.0 } else { 1.0 }) };
        let c = refine(step, 0.0, 1.0, 1e-9, Caps::DEFAULT).unwrap();
        assert!((c.instant - 0.7).abs() <= 1e-9, "{c:?}");
        assert_eq!(halvings(1.0, 1e-9), 30);
        assert!(c.evaluations <= 2 + 30 + 1, "{}", c.evaluations);
        // A bracket without the sign change is refused.
        let flat = |_: f64| -> Result<f64, ()> { Ok(1.0) };
        assert!(matches!(
            refine(flat, 0.0, 1.0, 1e-9, Caps::DEFAULT),
            Err(SolveError::NotBracketed { steps: 0, .. })
        ));
    }

    #[test]
    fn a_minimum_is_narrowed_by_golden_sections() {
        // A minimum is only as sharp as the quantity's rounding allows: a
        // parabola resolves to the tolerance, where a cosine about its
        // trough is flat to the last bit 1e-8 either side.
        let bowl = |t: f64| -> Result<f64, ()> { Ok((t - 3.0) * (t - 3.0)) };
        let least = minimum(bowl, 2.0, 4.0, 1e-9, Caps::DEFAULT).unwrap();
        assert!((least.instant - 3.0).abs() < 1e-8, "{least:?}");
        assert!(least.width <= 1e-9);
        // One evaluation a step: 2 / 0.618^n <= 1e-9 at n = 45.
        assert!(least.evaluations <= 2 + 45, "{}", least.evaluations);
        // A minimum at an end is approached, never passed.
        let rising = |t: f64| -> Result<f64, ()> { Ok(t) };
        let edge = minimum(rising, 0.0, 1.0, 1e-9, Caps::DEFAULT).unwrap();
        assert!(edge.instant < 1e-8, "{edge:?}");
        // A bracket the caps cannot narrow is an error, as is a reversed one.
        let tight = Caps {
            bracket_steps: 0,
            refinements: 3,
        };
        assert!(matches!(
            minimum(bowl, 2.0, 4.0, 1e-9, tight),
            Err(SolveError::NotConverged { steps: 3, .. })
        ));
        assert!(matches!(
            minimum(bowl, 4.0, 2.0, 1e-9, Caps::DEFAULT),
            Err(SolveError::Argument {
                name: "bracket",
                ..
            })
        ));
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

    #[test]
    fn the_seed_is_exact_on_a_cubic() {
        // A cubic is its own Hermite interpolant: the seed lands on its
        // root before the quantity is asked anything.
        let cubic = |t: f64| (t - 0.3) * (1.0 + t * t);
        let rate = |t: f64| 1.0 + t * t + (t - 0.3) * 2.0 * t;
        let seed = hermite_root((0.0, cubic(0.0), rate(0.0)), (1.0, cubic(1.0), rate(1.0)));
        assert!((seed - 0.3).abs() < 1e-15, "{seed}");
        // With no rate the seed is the secant's root.
        let secant = hermite_root((0.0, -1.0, f64::NAN), (1.0, 3.0, 1.0));
        assert!((secant - 0.25).abs() < 1e-15, "{secant}");
    }

    #[test]
    fn a_rate_ends_the_search_on_a_bracket_evaluated_on_both_sides() {
        // The Moon over a scan's step at a Julian day's magnitude, where
        // a converged Newton step is shorter than the instant's last bit.
        let origin = 2_461_041.5;
        let moon = move |t: f64| -> Result<(f64, f64), ()> {
            let u = t - origin;
            Ok((
                13.2 * (u - 0.2371) + 0.4 * math::sin(u * 2.0),
                13.2 + 0.8 * math::cos(u * 2.0),
            ))
        };
        let end = |t: f64| {
            let (value, rate) = moon(t).unwrap();
            (t, value, rate)
        };
        let (lo, hi) = (end(origin), end(origin + 0.5));
        let seeded = refine_with_rates(moon, lo, hi, 1e-7, Caps::DEFAULT).unwrap();
        let known = refine_known(
            |t| moon(t).map(|(value, _)| value),
            (lo.0, lo.1),
            (hi.0, hi.1),
            1e-7,
            Caps::DEFAULT,
        )
        .unwrap();
        assert!(seeded.width <= 1e-7, "{seeded:?}");
        assert!(seeded.evaluations <= 3, "{seeded:?}");
        assert!(
            seeded.evaluations * 2 <= known.evaluations,
            "{seeded:?} {known:?}"
        );
        // The Newton point from the nearer end sits on the line, where a
        // bracket's middle may be half the tolerance from it.
        assert!(
            moon(seeded.instant).unwrap().0.abs() < 1e-9 * 13.2,
            "{seeded:?}"
        );
        // A bracket without the sign change, and a bad tolerance, are
        // refused as `refine_known` refuses them.
        assert!(matches!(
            refine_with_rates(moon, hi, lo, 1e-7, Caps::DEFAULT),
            Err(SolveError::NotBracketed { steps: 0, .. })
        ));
        assert!(matches!(
            refine_with_rates(moon, lo, hi, 0.0, Caps::DEFAULT),
            Err(SolveError::Argument {
                name: "tolerance",
                ..
            })
        ));
    }

    proptest! {
        /// A rate that is wrong, of the wrong sign, zero, absent or
        /// absurd costs steps and never the answer: the search still ends
        /// on a bracket at most the tolerance wide with the line inside,
        /// within the cap, at a Julian day's magnitude.
        #[test]
        fn a_misleading_rate_costs_steps_not_the_answer(
            rate in 0.05f64..15.0,
            root in 0.01f64..0.99,
            amplitude in 0.0f64..0.9,
            frequency in 0.1f64..6.0,
            phase in 0.0f64..core::f64::consts::TAU,
            factor in prop::sample::select(vec![1.0, 0.5, 3.0, -1.0, 0.0, f64::NAN, f64::INFINITY, 1e6, 1e-6]),
        ) {
            let origin = 2_451_545.0;
            // Monotonic: the oscillation's rate stays under the drift's.
            let wobble = amplitude * rate / frequency;
            let curve = move |t: f64| -> Result<(f64, f64), ()> {
                let u = t - origin;
                let value = rate * (u - root)
                    + wobble * (math::sin(u * frequency + phase) - math::sin(root * frequency + phase));
                let slope = rate + wobble * frequency * math::cos(u * frequency + phase);
                Ok((value, slope * factor))
            };
            let end = |t: f64| {
                let (value, rate) = curve(t).unwrap();
                (t, value, rate)
            };
            let c = refine_with_rates(curve, end(origin), end(origin + 1.0), 1e-7, Caps::DEFAULT).unwrap();
            prop_assert!(c.width <= 1e-7, "{c:?}");
            prop_assert!(curve(c.instant - 1e-7).unwrap().0 < 0.0, "{c:?}");
            prop_assert!(curve(c.instant + 1e-7).unwrap().0 >= 0.0, "{c:?}");
            // A misled step is followed by a bisection, so the worst is
            // two evaluations a halving and the closing probe.
            let halvings = u32::try_from(halvings(1.0, 1e-7)).unwrap();
            prop_assert!(c.evaluations <= 2 * halvings + 2, "{c:?}");
            // A true rate never costs more than narrowing without one while
            // the curve's slope stays within half of its drift either way.
            // Nearer a stall a Newton step can be misled where narrowing
            // without a rate is not, and it was: measured over a grid of
            // 46 080 of these curves, a true rate cost more in 56, by three
            // evaluations at most, every one with the wobble at 0.7 of the
            // drift or more (none to 0.58), and 3.88 evaluations on average
            // against 6.71. The bound above holds for all of them.
            if factor.to_bits() == 1f64.to_bits() && amplitude <= 0.5 {
                let known = refine_known(
                    |t| curve(t).map(|(value, _)| value),
                    (origin, curve(origin).unwrap().0),
                    (origin + 1.0, curve(origin + 1.0).unwrap().0),
                    1e-7,
                    Caps::DEFAULT,
                )
                .unwrap();
                prop_assert!(c.evaluations <= known.evaluations, "{c:?} {known:?}");
            }
        }

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
                Ok((start + rate * t + amplitude * 360.0 / core::f64::consts::TAU * math::sin(t * rate / 57.3 + phase)).rem_euclid(360.0))
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
