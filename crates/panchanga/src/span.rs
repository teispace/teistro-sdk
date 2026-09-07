//! A limb's span: which member ran, when it really ran, and how much of
//! that fell inside the day.
//!
//! The conformance corpus records only the second of those — a tithi that
//! began at 21:05 yesterday is written as beginning at sunrise, and its
//! real start is not recoverable from the fixture
//! (`03-design/panchanga-day-conventions.md` §2). An almanac prints both
//! facts, so this carries both.

use serde::Serialize;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Utc};

/// One member of a limb, and when it ran.
///
/// ```
/// use teistro_core::interval::Interval;
/// use teistro_core::quantity::{JulianDay, Utc};
/// use teistro_panchanga::span::Span;
///
/// let window = Interval::literal(2_447_995.5, 2_447_996.5);
/// // A tithi that began yesterday evening and ends this afternoon.
/// let whole = Interval::literal(2_447_995.2, 2_447_996.1);
/// let span = Span::new("Krishna Chaturdashi", whole, window).expect("it touches the day");
///
/// assert!(span.began_before(), "so the almanac says 'from yesterday'");
/// assert!(!span.ends_after());
/// assert_eq!(span.inside.from, window.from, "the printed row starts at sunrise");
/// assert_eq!(span.inside.to, whole.to, "and ends where the tithi does");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Span<T> {
    /// Which member ran.
    pub member: T,
    /// When the member itself began and ended, whether or not either
    /// instant lies inside the day.
    pub whole: Interval,
    /// The part of it inside the day: what an almanac row prints.
    pub inside: Interval,
}

impl<T> Span<T> {
    /// A span of a member over a window, or `None` when the member does
    /// not touch the window at all.
    #[must_use]
    pub fn new(member: T, whole: Interval, window: Interval) -> Option<Span<T>> {
        whole.clipped_to(window).map(|inside| Span {
            member,
            whole,
            inside,
        })
    }

    /// Whether the member began before the day did.
    #[must_use]
    pub fn began_before(&self) -> bool {
        self.whole.from.get() < self.inside.from.get()
    }

    /// Whether the member outlasts the day.
    #[must_use]
    pub fn ends_after(&self) -> bool {
        self.whole.to.get() > self.inside.to.get()
    }

    /// How far through the member an instant is: 0 when it began, 1 when
    /// it ends. Measured over the member's own span, not the clipped one,
    /// because that is the fraction every rule that reads one wants.
    #[must_use]
    pub fn fraction_at(&self, instant: JulianDay<Utc>) -> f64 {
        self.whole.fraction_at(instant)
    }

    /// Whether the member was running at an instant, by its own bounds.
    #[must_use]
    pub fn contains(&self, instant: JulianDay<Utc>) -> bool {
        self.whole.contains(instant)
    }

    /// The same span with the member mapped, keeping both intervals.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Span<U> {
        Span {
            member: f(self.member),
            whole: self.whole,
            inside: self.inside,
        }
    }
}

/// The member of a list of spans that was running at an instant.
///
/// The lists a day carries are short — one to four entries — so this is a
/// scan, and it is here rather than written out four times at each call
/// site.
#[must_use]
pub fn at<T>(spans: &[Span<T>], instant: JulianDay<Utc>) -> Option<&Span<T>> {
    spans
        .iter()
        .find(|span| span.contains(instant))
        // The last span's closing instant belongs to it rather than to
        // nothing, which is what a caller asking about the end of the day
        // means.
        .or_else(|| {
            spans
                .last()
                .filter(|span| span.whole.contains_inclusive(instant))
        })
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "tests fail by panicking"
    )]

    use super::{Span, at};
    use teistro_core::interval::Interval;
    use teistro_core::quantity::{JulianDay, Utc};

    fn jd(value: f64) -> JulianDay<Utc> {
        JulianDay::literal(value)
    }

    fn window() -> Interval {
        Interval::literal(2_447_995.5, 2_447_996.5)
    }

    #[test]
    fn a_span_that_misses_the_window_is_not_a_span() {
        assert!(Span::new((), Interval::literal(2_447_990.0, 2_447_991.0), window()).is_none());
        assert!(Span::new((), Interval::literal(2_447_999.0, 2_448_000.0), window()).is_none());
        assert!(Span::new((), Interval::literal(2_447_995.0, 2_447_996.0), window()).is_some());
    }

    #[test]
    fn the_clipped_view_is_what_an_almanac_prints_and_the_whole_is_what_it_loses() {
        let whole = Interval::literal(2_447_995.2, 2_447_996.1);
        let span = Span::new("a tithi", whole, window()).expect("it touches the day");
        assert!(span.began_before());
        assert!(!span.ends_after());
        assert_eq!(span.inside.from, window().from);
        assert_eq!(span.inside.to, whole.to);
        assert_eq!(span.whole, whole, "the member's own bounds are kept");

        let over = Span::new((), Interval::literal(2_447_996.0, 2_447_997.0), window())
            .expect("it touches the day");
        assert!(!over.began_before());
        assert!(over.ends_after());
    }

    #[test]
    fn a_fraction_is_of_the_member_and_not_of_the_clipped_part() {
        let whole = Interval::literal(2_447_995.0, 2_447_996.0);
        let span = Span::new((), whole, window()).expect("it touches the day");
        // Halfway through the tithi is before the day even began.
        assert!((span.fraction_at(jd(2_447_995.5)) - 0.5).abs() < 1e-9);
        assert!(span.fraction_at(jd(2_447_995.0)).abs() < 1e-9);
    }

    #[test]
    fn one_span_of_a_list_holds_an_instant() {
        let spans = [
            Span::new(1, Interval::literal(2_447_995.0, 2_447_996.0), window()).unwrap(),
            Span::new(2, Interval::literal(2_447_996.0, 2_447_997.0), window()).unwrap(),
        ];
        assert_eq!(at(&spans, jd(2_447_995.7)).map(|s| s.member), Some(1));
        assert_eq!(at(&spans, jd(2_447_996.0)).map(|s| s.member), Some(2));
        // The last span's closing instant belongs to it, not to nothing.
        assert_eq!(at(&spans, jd(2_447_997.0)).map(|s| s.member), Some(2));
        assert_eq!(at(&spans, jd(2_447_990.0)).map(|s| s.member), None);
        assert_eq!(at::<u8>(&[], jd(2_447_996.0)).map(|s| s.member), None);
    }

    #[test]
    fn a_member_can_be_mapped_without_moving_its_instants() {
        let whole = Interval::literal(2_447_995.2, 2_447_996.1);
        let span = Span::new(3_u8, whole, window()).expect("it touches the day");
        let mapped = span.map(|member| u32::from(member) * 2);
        assert_eq!(mapped.member, 6);
        assert_eq!(mapped.whole, span.whole);
        assert_eq!(mapped.inside, span.inside);
    }
}
