//! The lagna's risings: each instant it enters a navamsa, found by
//! search (`03-design/muhurta.md` §4.3, `03-design/festival-sait.md`).
//!
//! The lagna is sampled along the clock and each change of navamsa is
//! bisected; a change of sign is a change of navamsa. Beyond the polar
//! circle the lagna jumps across the arcs that never rise, and the
//! bisection then closes on the jump: one cut, where several navamsas are
//! skipped at once, which is what the sky does.

use serde::{Deserialize, Serialize};
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Utc};

/// How far apart the lagna is sampled, in days: one minute. The lagna
/// crosses a navamsa (3°20′) in about thirteen minutes on average and in
/// no less than about four where it rises fastest outside the polar
/// circles, so a minute sees every crossing there.
///
/// ```
/// use teistro_panchanga::rising::SAMPLE_DAYS;
///
/// assert_eq!(SAMPLE_DAYS * 1440.0, 1.0);
/// ```
pub const SAMPLE_DAYS: f64 = 1.0 / 1440.0;

/// How close a cut is found, in days: about a millisecond.
///
/// ```
/// use teistro_core::interval::Interval;
/// use teistro_core::quantity::JulianDay;
/// use teistro_panchanga::rising::lagna_cuts;
///
/// // A lagna turning a whole circle a day, at 0° Aries at T.
/// const T: f64 = 2_460_000.5;
/// let lagna_at = |at: JulianDay<_>| Ok(((at.get() - T) * 360.0).rem_euclid(360.0));
/// use teistro_panchanga::rising::TOLERANCE_DAYS;
///
/// // The first navamsa is entered within a tolerance of its exact instant.
/// let cuts = lagna_cuts(Interval::literal(T + 0.001, T + 0.02), lagna_at)?;
/// assert!((cuts[0].at.get() - (T + 1.0 / 108.0)).abs() <= TOLERANCE_DAYS);
/// # Ok::<(), teistro_core::Error>(())
/// ```
pub const TOLERANCE_DAYS: f64 = 1e-8;

/// An instant the lagna enters a navamsa.
///
/// ```
/// use teistro_core::interval::Interval;
/// use teistro_core::quantity::JulianDay;
/// use teistro_panchanga::rising::lagna_cuts;
///
/// // A lagna turning a whole circle a day, at 0° Aries at T.
/// const T: f64 = 2_460_000.5;
/// let lagna_at = |at: JulianDay<_>| Ok(((at.get() - T) * 360.0).rem_euclid(360.0));
/// // Six hours from just after T: 26 navamsas entered, two of them new signs.
/// let cuts = lagna_cuts(Interval::literal(T + 0.001, T + 0.249), lagna_at)?;
/// assert_eq!(cuts.len(), 26);
/// assert_eq!(cuts.iter().filter(|c| c.sign_changed).count(), 2);
/// assert_eq!((cuts[0].navamsa, cuts[8].navamsa), (1, 9));
/// # Ok::<(), teistro_core::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct LagnaCut {
    /// When.
    pub at: JulianDay<Utc>,
    /// The navamsa entered, 0 (Aries' first) to 107 (Pisces' last).
    pub navamsa: u8,
    /// Whether the lagna entered a new sign here.
    pub sign_changed: bool,
}

/// Every instant in a window at which the lagna enters a navamsa.
///
/// `lagna_at` gives the lagna's longitude, degrees, at an instant; it is
/// asked once a minute and then at each bisection step.
///
/// ```
/// use teistro_core::interval::Interval;
/// use teistro_core::quantity::JulianDay;
/// use teistro_panchanga::rising::lagna_cuts;
///
/// // A lagna turning a whole circle a day, at 0° Aries at T.
/// const T: f64 = 2_460_000.5;
/// let lagna_at = |at: JulianDay<_>| Ok(((at.get() - T) * 360.0).rem_euclid(360.0));
/// let cuts = lagna_cuts(Interval::literal(T + 0.001, T + 0.249), lagna_at)?;
/// assert!(cuts.windows(2).all(|w| w[0].at.get() < w[1].at.get()));
/// # Ok::<(), teistro_core::Error>(())
/// ```
///
/// # Errors
///
/// Whatever `lagna_at` returns.
pub fn lagna_cuts<F>(within: Interval, mut lagna_at: F) -> Result<Vec<LagnaCut>, Error>
where
    F: FnMut(JulianDay<Utc>) -> Result<f64, Error>,
{
    let mut cuts = Vec::new();
    let (from, to) = (within.from.get(), within.to.get());
    let mut before = (from, navamsa_of(lagna_at(within.from)?));
    let mut t = from;
    while t < to {
        t = (t + SAMPLE_DAYS).min(to);
        let now = (t, navamsa_of(lagna_at(JulianDay::literal(t))?));
        if now.1 != before.1 {
            refine(before, now, &mut lagna_at, &mut cuts)?;
        }
        before = now;
    }
    Ok(cuts)
}

/// [`lagna_cuts`] for a lagna that only moves forward, as it does
/// everywhere outside the polar circles (the ascendant's rate,
/// `cos ε + tan φ sin ε sin θ` over a positive denominator, is positive
/// for every sidereal time when `tan φ tan ε < 1`): the same cuts, bit
/// for bit, from about a quarter fewer evaluations. The refinement of each
/// cut, some sixteen halvings from a minute to a millisecond, is the
/// same; most of the minute samples between cuts are not taken.
///
/// The minute samples are the same ones, but a run of them is skipped
/// whole when its two ends lie in one navamsa. A lagna moving forward
/// by less than a turn cannot leave a navamsa and come back to it, and a
/// run of sixteen minutes is far less than a turn. The runs that do
/// change are halved over the same samples until two neighbours differ,
/// and each such pair is refined exactly as [`lagna_cuts`] refines it.
///
/// ```
/// use teistro_core::interval::Interval;
/// use teistro_core::quantity::JulianDay;
/// use teistro_panchanga::rising::lagna_cuts;
///
/// // A lagna turning a whole circle a day, at 0° Aries at T.
/// const T: f64 = 2_460_000.5;
/// let lagna_at = |at: JulianDay<_>| Ok(((at.get() - T) * 360.0).rem_euclid(360.0));
/// use teistro_panchanga::rising::lagna_cuts_forward;
///
/// // A lagna that only moves forward gives the same cuts for fewer samples.
/// let within = Interval::literal(T + 0.001, T + 0.249);
/// assert_eq!(lagna_cuts_forward(within, lagna_at)?, lagna_cuts(within, lagna_at)?);
/// # Ok::<(), teistro_core::Error>(())
/// ```
///
/// # Errors
///
/// Whatever `lagna_at` returns.
pub fn lagna_cuts_forward<F>(within: Interval, mut lagna_at: F) -> Result<Vec<LagnaCut>, Error>
where
    F: FnMut(JulianDay<Utc>) -> Result<f64, Error>,
{
    // The sample instants, built by the same sum as `lagna_cuts` so each
    // is the same value to the bit.
    let (from, to) = (within.from.get(), within.to.get());
    let mut grid = vec![from];
    let mut t = from;
    while t < to {
        t = (t + SAMPLE_DAYS).min(to);
        grid.push(t);
    }
    let mut cuts = Vec::new();
    let last = grid.len() - 1;
    let mut start = sampled(&grid, 0, &mut lagna_at)?;
    while start.0 < last {
        let end = sampled(&grid, (start.0 + STRIDE).min(last), &mut lagna_at)?;
        halve(&grid, start, end, &mut lagna_at, &mut cuts)?;
        start = end;
    }
    Ok(cuts)
}

/// The sample at an index of the grid, with the navamsa the lagna is in.
fn sampled<F>(grid: &[f64], i: usize, lagna_at: &mut F) -> Result<(usize, (f64, u8)), Error>
where
    F: FnMut(JulianDay<Utc>) -> Result<f64, Error>,
{
    let at = grid.get(i).copied().unwrap_or(f64::NAN);
    Ok((i, (at, navamsa_of(lagna_at(JulianDay::literal(at))?))))
}

/// How many minute samples [`lagna_cuts_forward`] skips at once: sixteen
/// minutes, in which a lagna outside the polar circles moves well under
/// a turn (a turn takes a day).
const STRIDE: usize = 16;

/// Halves a run of samples whose ends differ until two neighbours do, and
/// refines each such pair.
fn halve<F>(
    grid: &[f64],
    a: (usize, (f64, u8)),
    b: (usize, (f64, u8)),
    lagna_at: &mut F,
    cuts: &mut Vec<LagnaCut>,
) -> Result<(), Error>
where
    F: FnMut(JulianDay<Utc>) -> Result<f64, Error>,
{
    if a.1.1 == b.1.1 {
        return Ok(());
    }
    if b.0 - a.0 == 1 {
        return refine(a.1, b.1, lagna_at, cuts);
    }
    let m = sampled(grid, usize::midpoint(a.0, b.0), lagna_at)?;
    halve(grid, a, m, lagna_at, cuts)?;
    halve(grid, m, b, lagna_at, cuts)
}

/// Finds every change of navamsa between two samples that differ.
fn refine<F>(
    a: (f64, u8),
    b: (f64, u8),
    lagna_at: &mut F,
    cuts: &mut Vec<LagnaCut>,
) -> Result<(), Error>
where
    F: FnMut(JulianDay<Utc>) -> Result<f64, Error>,
{
    let mid = f64::midpoint(a.0, b.0);
    // Converged, or below the last bit at this magnitude (a midpoint that
    // rounds onto an end cannot split the interval further).
    if b.0 - a.0 <= TOLERANCE_DAYS || mid <= a.0 || mid >= b.0 {
        cuts.push(LagnaCut {
            at: JulianDay::literal(b.0),
            navamsa: b.1,
            sign_changed: b.1 / 9 != a.1 / 9,
        });
        return Ok(());
    }
    let m = (mid, navamsa_of(lagna_at(JulianDay::literal(mid))?));
    if m.1 != a.1 {
        refine(a, m, lagna_at, cuts)?;
    }
    if m.1 != b.1 {
        refine(m, b, lagna_at, cuts)?;
    }
    Ok(())
}

/// The navamsa a longitude falls in, 0 to 107: the one floor every
/// sign and navamsa is read through.
///
/// ```
/// use teistro_panchanga::rising::navamsa_of;
///
/// // 3°20′ a navamsa: 10° is Aries' fourth, 359.9° Pisces' last.
/// assert_eq!(navamsa_of(10.0), 3);
/// assert_eq!(navamsa_of(359.9), 107);
/// assert_eq!(navamsa_of(-0.1), 107);
/// ```
#[must_use]
pub fn navamsa_of(deg: f64) -> u8 {
    let index = (deg.rem_euclid(360.0) * 0.3).floor();
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a floor in 0..108"
    )]
    let id = index as u8;
    id.min(107)
}
