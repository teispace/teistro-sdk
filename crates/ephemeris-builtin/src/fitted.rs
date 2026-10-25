//! The Moon and the Earth from fitted series: each theory evaluated at
//! nine instants of a two-day segment, and every instant inside the
//! segment answered from the Chebyshev series through them
//! ([`teistro_core::chebyshev::Segments`]).
//!
//! # Why
//!
//! An almanac asks for the Sun and the Moon hundreds of times a day:
//! every sunrise, tithi end, nakshatra end and sankranti is a root
//! search. Each answer from a theory is hundreds to a thousand terms with
//! a sine each, and almost all of the built-in's time went there (P2-6
//! of the consumer brief: 80% in the sine, the cosine and their argument
//! reduction, in ELP2000-82B). Over two days either body is a smooth arc
//! that nine values pin: a segment costs nine evaluations of the theory
//! and then answers any number of instants for about thirty
//! multiplications each.
//!
//! The Earth is fitted because the Sun is the Earth reversed and every
//! planet is its own place less the Earth's. The other planets are not:
//! a chart asks for each once, where a fit would cost nine.
//!
//! # How close
//!
//! Measured against the theories over the provider's six centuries
//! (`tests/fitted.rs`): the Moon within 1.1e-5 arcseconds and 3.6e-7
//! degrees a day, the Sun within 7e-7 arcseconds and 3e-9 degrees a day.
//! Neither is the fit's own figure: a Julian day near 2.4 million is an
//! `f64` with a resolution of 40 microseconds, in which the Moon moves
//! 2e-5 arcseconds, so the theory itself cannot be asked more finely than
//! that. The tiers' own truncation on the Moon is 18, 0.71 and 0
//! arcseconds, and the theories differ from a modern ephemeris by 20 to
//! 30. The fit's rate is the closer of the two to the theory's own
//! derivative: the series' analytic rate is 1e-8 to 3e-8 from it, the
//! fit's 6e-9.
//!
//! # The rate is the fit's derivative
//!
//! The port asks for a speed beside every position, and this crate
//! differentiates rather than differences. A Chebyshev series has an
//! exact derivative, so the rate is the derivative of the position the
//! same call returns: the two can never disagree, and the nodes need only
//! the theory's positions, which for the Moon is half its work.
//!
//! # Determinism
//!
//! The segments are a fixed grid, two days from J2000, so an instant's
//! answer depends on the instant alone: never on what was asked before
//! it, nor on which segments a thread happens to hold. The segments are a
//! thread's own, so threads share nothing and take no lock, and a segment
//! evicted and fitted again is the same bits.

use std::cell::RefCell;

use teistro_core::chebyshev::Segments;

use crate::elp;
use crate::series::J2000;
use crate::tables;

/// A segment's length, days: a power of two, so an instant's place in
/// its segment is exact.
///
/// ```
/// use teistro_ephemeris_builtin::fitted::SEGMENT_DAYS;
///
/// assert_eq!(SEGMENT_DAYS.log2().fract(), 0.0, "a power of two");
/// ```
pub const SEGMENT_DAYS: f64 = 2.0;

/// The series' degree; a segment is fitted through one more value than
/// this.
///
/// ```
/// use teistro_ephemeris_builtin::fitted::{DEGREE, SEGMENT_DAYS};
///
/// // Nine values to a two-day segment.
/// assert_eq!((DEGREE + 1, SEGMENT_DAYS), (9, 2.0));
/// ```
pub const DEGREE: usize = 8;

/// The values a segment is fitted through.
const NODES: usize = DEGREE + 1;

/// A thread's fitted segments of one body.
type Store = RefCell<Segments<NODES, 3>>;

thread_local! {
    /// The Moon's segments this thread has fitted, from J2000.
    static MOON: Store = const { RefCell::new(Segments::new(J2000, SEGMENT_DAYS)) };
    /// The Earth's segments this thread has fitted, from J2000.
    static EARTH: Store = const { RefCell::new(Segments::new(J2000, SEGMENT_DAYS)) };
}

/// A body's position and rate at `jd` from a thread's store, fitted from
/// `theory`; `None` for an instant that is not finite, which falls in no
/// segment.
fn from(
    store: &'static std::thread::LocalKey<Store>,
    jd: f64,
    theory: impl FnMut(f64) -> [f64; 3],
) -> Option<([f64; 3], [f64; 3])> {
    store.with(|segments| segments.borrow_mut().at(jd, theory))
}

/// The Moon's geocentric position and rate at a Julian day in dynamical
/// time: kilometres and kilometres a day, in the frame
/// [`elp::position_and_rate_from`] answers in.
///
/// An instant that is not finite is answered by the series itself.
///
/// ```
/// use teistro_ephemeris_builtin::{elp, fitted, tables};
///
/// // The fit against the series it was fitted from, a quarter-day off a node.
/// let jd = 2_460_000.3;
/// let (fit, fit_rate) = fitted::moon(jd);
/// let (series, series_rate) = elp::position_and_rate_from(&tables::MOON_MAIN, &tables::MOON_PERTURBATIONS, jd);
/// for axis in 0..3 {
///     assert!((fit[axis] - series[axis]).abs() < 1e-4, "under ten centimetres");
///     assert!((fit_rate[axis] - series_rate[axis]).abs() < 1e-2, "of some 90 000 km a day");
/// }
/// // An instant that is not finite is the series' own answer.
/// assert!(fitted::moon(f64::NAN).0[0].is_nan());
/// ```
#[must_use]
pub fn moon(jd: f64) -> ([f64; 3], [f64; 3]) {
    from(&MOON, jd, |at| {
        elp::position_from(&tables::MOON_MAIN, &tables::MOON_PERTURBATIONS, at)
    })
    .unwrap_or_else(|| {
        elp::position_and_rate_from(&tables::MOON_MAIN, &tables::MOON_PERTURBATIONS, jd)
    })
}

/// The Earth's heliocentric position and rate at a Julian day in
/// dynamical time, astronomical units and astronomical units a day,
/// fitted from `theory`: the provider's own VSOP87A sum of the Earth's
/// position.
///
/// `None` for an instant that is not finite, which the caller answers
/// from the theory itself.
///
/// ```
/// use teistro_ephemeris_builtin::fitted;
/// use teistro_ephemeris_builtin::series::{millennia, sum};
/// use teistro_ephemeris_builtin::tables::PLANETS;
///
/// // The Earth's VSOP87A position, as the provider sums it.
/// let theory = |jd: f64| PLANETS[2].1.map(|terms| sum(terms, millennia(jd)));
/// let jd = 2_460_000.3;
/// let (fit, _) = fitted::earth(jd, theory).ok_or("a finite instant")?;
/// for (fit, series) in fit.iter().zip(theory(jd)) {
///     assert!((fit - series).abs() < 1e-11, "under two metres");
/// }
/// assert!(fitted::earth(f64::NAN, theory).is_none());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[must_use]
pub fn earth(jd: f64, theory: impl FnMut(f64) -> [f64; 3]) -> Option<([f64; 3], [f64; 3])> {
    from(&EARTH, jd, theory)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::float_cmp,
        reason = "the cache is checked to give the same bits, so it is compared exactly"
    )]

    use super::moon;

    /// An instant's answer is the same bits on a fresh thread as on one
    /// that has fitted other segments first.
    #[test]
    fn an_answer_does_not_depend_on_the_thread() {
        let jd = 2_461_000.123_456;
        let fresh = std::thread::spawn(move || moon(jd))
            .join()
            .unwrap_or_default();
        let _ = moon(jd - 1.7);
        let _ = moon(jd + 300.0);
        assert_eq!(moon(jd), fresh);
    }
}
