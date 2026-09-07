//! An interval of time, and the equal division every period of a day is.
//!
//! A daily panchanga is sixteen choghadiya, twenty-four horas, thirty
//! muhurtas and three inauspicious eighths, and not one of them divides a
//! clock hour: each is an equal division of the daylight or of the night,
//! so a period's length changes with the season and the latitude
//! (`03-design/panchanga-day-conventions.md` §3, measured over the 55
//! recorded days). One divider serves all of them, and it belongs here
//! rather than in the module that happened to need it first: dashas,
//! muhurta windows and transits are intervals too, and none of them
//! should define its own.
//!
//! ```
//! use teistro_core::interval::Interval;
//! use teistro_core::quantity::{JulianDay, Utc};
//!
//! let sunrise = JulianDay::<Utc>::literal(2_447_995.497_036_06);
//! let sunset = JulianDay::<Utc>::literal(2_448_000.0);
//! let daylight = Interval::new(sunrise, sunset).expect("sunset follows sunrise");
//!
//! // The eight choghadiya of the daylight.
//! let parts: Vec<Interval> = daylight.divided(8).collect();
//! assert_eq!(parts.len(), 8);
//! assert_eq!(parts[0].from, daylight.from);
//! assert_eq!(parts[7].to, daylight.to);
//! // Consecutive parts share an instant, so nothing falls between two.
//! assert_eq!(parts[0].to, parts[1].from);
//! ```

use core::fmt;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Status};
use crate::quantity::{JulianDay, Utc};

/// The most parts a division may have.
///
/// A day is divided into at most sixty karanas, and nothing an almanac
/// reports divides an arc more finely; a caller asking for more has a
/// bug rather than a requirement, and the refusal names the limit.
pub const MOST_PARTS: u16 = 64;

/// A half-open interval of time, `[from, to)`.
///
/// Half-open is what makes a sequence of parts a partition: an instant
/// belongs to exactly one hora, not to two at the boundary. The last
/// part of a divided arc is the exception a reader expects — its `to` is
/// the arc's own end, and [`Interval::contains`] is false there, so use
/// [`Interval::contains_inclusive`] when the closing instant counts.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Interval {
    /// When it begins.
    pub from: JulianDay<Utc>,
    /// When it ends.
    pub to: JulianDay<Utc>,
}

impl Interval {
    /// An interval from one instant to another.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` when `to` precedes `from`. An empty interval — one
    /// whose ends are equal — is accepted, because an arc that a polar
    /// day collapsed is a real answer and the value should be able to
    /// say so; [`Interval::is_empty`] reports it.
    pub fn new(from: JulianDay<Utc>, to: JulianDay<Utc>) -> Result<Interval, Error> {
        if to.get() < from.get() {
            return Err(Error::invalid_arg(format!(
                "an interval ends before it begins: {from} to {to}"
            ))
            .with_field("to"));
        }
        Ok(Interval { from, to })
    }

    /// An interval written as a literal, in days.
    ///
    /// # Panics
    ///
    /// When either end is not finite, or `to` precedes `from`; in a
    /// constant both are compile errors, which is what this is for.
    #[must_use]
    pub const fn literal(from: f64, to: f64) -> Interval {
        assert!(from <= to, "an interval literal ends before it begins");
        Interval {
            from: JulianDay::literal(from),
            to: JulianDay::literal(to),
        }
    }

    /// Its length in days.
    #[must_use]
    pub fn days(self) -> f64 {
        self.to.get() - self.from.get()
    }

    /// Its length in hours, which is how a period is read.
    #[must_use]
    pub fn hours(self) -> f64 {
        self.days() * 24.0
    }

    /// Whether it holds no time at all, which a collapsed polar arc does.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.days() <= 0.0
    }

    /// Whether an instant lies in `[from, to)`.
    #[must_use]
    pub fn contains(self, instant: JulianDay<Utc>) -> bool {
        instant.get() >= self.from.get() && instant.get() < self.to.get()
    }

    /// Whether an instant lies in `[from, to]`, the closing instant
    /// included: what the last part of a divided arc needs.
    #[must_use]
    pub fn contains_inclusive(self, instant: JulianDay<Utc>) -> bool {
        instant.get() >= self.from.get() && instant.get() <= self.to.get()
    }

    /// How far through it an instant is: 0 at the start, 1 at the end.
    ///
    /// An instant outside the interval gives a value outside `[0, 1]`,
    /// which is the useful answer rather than a clamp — how far past the
    /// end something is is a question a caller asks. An empty interval
    /// gives 0, there being no "through" to be.
    #[must_use]
    pub fn fraction_at(self, instant: JulianDay<Utc>) -> f64 {
        let length = self.days();
        if length <= 0.0 {
            0.0
        } else {
            (instant.get() - self.from.get()) / length
        }
    }

    /// The instant a fraction of the way through it.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` for a fraction that is not finite.
    pub fn at_fraction(self, fraction: f64) -> Result<JulianDay<Utc>, Error> {
        if !fraction.is_finite() {
            return Err(Error::invalid_arg(format!(
                "a fraction of an interval must be finite, not {fraction}"
            ))
            .with_field("fraction"));
        }
        Ok(JulianDay::literal(
            self.days().mul_add(fraction, self.from.get()),
        ))
    }

    /// The part of it that lies inside another interval, or `None` when
    /// they do not overlap.
    ///
    /// This is what clips a limb's own span to the day's window: a tithi
    /// that began yesterday evening is reported as beginning at sunrise,
    /// and its real bounds are kept beside the clipped ones.
    #[must_use]
    pub fn clipped_to(self, window: Interval) -> Option<Interval> {
        let from = self.from.get().max(window.from.get());
        let to = self.to.get().min(window.to.get());
        (from <= to).then(|| Interval {
            from: JulianDay::literal(from),
            to: JulianDay::literal(to),
        })
    }

    /// Whether it overlaps another interval at all.
    #[must_use]
    pub fn overlaps(self, other: Interval) -> bool {
        self.from.get() < other.to.get() && other.from.get() < self.to.get()
    }

    /// The interval divided into `parts` equal parts, in order.
    ///
    /// Every period of a panchanga day is one of these: the three
    /// inauspicious eighths and the eight choghadiya divide the daylight
    /// eight ways, the horas twelve, the muhurtas fifteen. Consecutive
    /// parts share an instant exactly, because each is computed from the
    /// whole rather than by adding a step, so no instant falls between
    /// two parts and the last part ends exactly where the arc does.
    ///
    /// `parts` of zero yields nothing, and more than [`MOST_PARTS`] is
    /// truncated to it; [`Interval::part`] is the checked single-part
    /// form when a caller needs the refusal.
    pub fn divided(self, parts: u16) -> impl Iterator<Item = Interval> {
        let parts = parts.min(MOST_PARTS);
        (0..parts).filter_map(move |index| self.part(index, parts).ok())
    }

    /// One part of an equal division, counted from zero.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` for zero parts, more than [`MOST_PARTS`], or an
    /// index that is not one of them; each names the limit and the value.
    pub fn part(self, index: u16, of: u16) -> Result<Interval, Error> {
        if of == 0 || of > MOST_PARTS {
            return Err(Error::invalid_arg(format!(
                "an interval divides into 1 to {MOST_PARTS} parts, not {of}"
            ))
            .with_field("of"));
        }
        if index >= of {
            return Err(Error::new(
                Status::OutOfRange,
                format!("part {index} of {of}: the parts are 0 to {}", of - 1),
            )
            .with_field("index"));
        }
        let whole = self.days();
        let of = f64::from(of);
        Ok(Interval {
            from: JulianDay::literal(whole.mul_add(f64::from(index) / of, self.from.get())),
            to: JulianDay::literal(whole.mul_add(f64::from(index + 1) / of, self.from.get())),
        })
    }

    /// Which part of an equal division an instant falls in, counted from
    /// zero, or `None` when it is outside or the interval is empty.
    ///
    /// The closing instant belongs to the last part, so an instant at the
    /// very end of the day is in the last hora rather than in none.
    #[must_use]
    pub fn part_at(self, instant: JulianDay<Utc>, of: u16) -> Option<u16> {
        if of == 0 || of > MOST_PARTS || self.is_empty() || !self.contains_inclusive(instant) {
            return None;
        }
        let through = self.fraction_at(instant) * f64::from(of);
        // A fraction below one is an index below `of`, and the closing
        // instant lands exactly on `of`, which is the last part.
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a fraction of a bounded division is 0 to `of`, which is at most 64"
        )]
        let index = through as u16;
        Some(index.min(of - 1))
    }
}

impl fmt::Display for Interval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} to {}", self.from, self.to)
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index their own fixtures"
    )]

    use super::{Interval, MOST_PARTS};
    use crate::quantity::{JulianDay, Utc};

    fn jd(value: f64) -> JulianDay<Utc> {
        JulianDay::literal(value)
    }

    fn day() -> Interval {
        Interval::literal(2_447_995.5, 2_447_996.5)
    }

    #[test]
    fn an_interval_cannot_end_before_it_begins() {
        assert!(Interval::new(jd(2.0), jd(1.0)).is_err());
        let empty = Interval::new(jd(2.0), jd(2.0)).expect("an empty interval is a real answer");
        assert!(empty.is_empty(), "a polar arc collapses to nothing");
        assert!(day().days() > 0.0);
    }

    #[test]
    fn the_bounds_are_half_open_so_a_sequence_partitions() {
        let parts: Vec<Interval> = day().divided(8).collect();
        assert_eq!(parts.len(), 8);
        for pair in parts.windows(2) {
            assert_eq!(pair[0].to, pair[1].from, "no instant falls between parts");
            assert!(pair[0].contains(pair[0].from));
            assert!(!pair[0].contains(pair[0].to), "the end is the next's start");
        }
        assert_eq!(parts[0].from, day().from);
        assert_eq!(parts[7].to, day().to, "the last part closes the arc");
    }

    #[test]
    fn the_parts_are_computed_from_the_whole_and_not_by_adding_a_step() {
        // Thirty parts of a day whose length is not a round number: the
        // last part has to end exactly at the arc's end, which repeated
        // addition would miss.
        let arc = Interval::literal(2_447_995.497_036_061, 2_447_996.029_757_501_6);
        let parts: Vec<Interval> = arc.divided(30).collect();
        assert_eq!(parts.len(), 30);
        assert!((parts[29].to.get() - arc.to.get()).abs() < 1e-12);
        assert!((parts[0].from.get() - arc.from.get()).abs() < 1e-12);
    }

    #[test]
    fn a_division_is_bounded_and_says_so() {
        assert!(day().part(0, 0).is_err(), "zero parts is not a division");
        assert!(day().part(0, MOST_PARTS + 1).is_err());
        assert!(day().part(8, 8).is_err(), "the parts are 0 to 7");
        assert!(day().part(7, 8).is_ok());
        assert_eq!(day().divided(0).count(), 0);
        assert_eq!(
            day().divided(MOST_PARTS + 10).count(),
            usize::from(MOST_PARTS)
        );
    }

    #[test]
    fn an_instant_falls_in_exactly_one_part() {
        let arc = day();
        assert_eq!(arc.part_at(arc.from, 8), Some(0));
        assert_eq!(arc.part_at(jd(2_447_995.5 + 1.0 / 8.0), 8), Some(1));
        assert_eq!(arc.part_at(jd(2_447_995.5 + 7.9 / 8.0), 8), Some(7));
        // The closing instant belongs to the last part rather than to none.
        assert_eq!(arc.part_at(arc.to, 8), Some(7));
        assert_eq!(arc.part_at(jd(2_447_990.0), 8), None, "before the arc");
        assert_eq!(arc.part_at(jd(2_448_000.0), 8), None, "after it");
        assert_eq!(Interval::literal(1.0, 1.0).part_at(jd(1.0), 8), None);
    }

    #[test]
    fn a_fraction_and_an_instant_are_inverse() {
        let arc = day();
        // A Julian day of the modern era is about 2.4 million, where a
        // double's last bit is half a nanosecond; the round trip is that
        // and not tighter, which is a hundred-thousandth of a second.
        for fraction in [0.0, 0.25, 0.5, 0.999] {
            let at = arc.at_fraction(fraction).expect("a finite fraction");
            assert!((arc.fraction_at(at) - fraction).abs() < 1e-9);
        }
        assert!(arc.at_fraction(f64::NAN).is_err());
        // Outside the interval the answer is the useful one, not a clamp.
        assert!(arc.fraction_at(jd(2_447_994.5)) < 0.0);
        assert!(Interval::literal(1.0, 1.0).fraction_at(jd(1.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn clipping_is_what_the_corpus_records_and_the_whole_is_what_it_loses() {
        let window = day();
        // A tithi that began yesterday evening and ends this afternoon.
        let tithi = Interval::literal(2_447_995.0, 2_447_996.1);
        let inside = tithi.clipped_to(window).expect("it overlaps");
        assert_eq!(inside.from, window.from, "clipped to the window's start");
        assert_eq!(inside.to, tithi.to, "and to its own end");
        assert!(tithi.overlaps(window));
        // One that ended before the window began is not in the day.
        let earlier = Interval::literal(2_447_990.0, 2_447_991.0);
        assert!(earlier.clipped_to(window).is_none());
        assert!(!earlier.overlaps(window));
    }

    #[test]
    fn an_interval_says_what_it_is() {
        assert_eq!(
            Interval::literal(2_447_995.5, 2_447_996.5).to_string(),
            "JD 2447995.5 UTC to JD 2447996.5 UTC"
        );
    }
}
