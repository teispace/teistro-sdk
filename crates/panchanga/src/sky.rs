//! What the Moon and the Sun did in the day: rise and set, the sign each
//! stood in, and when either left it.
//!
//! The Moon's rise and set are where the recording engine keeps two
//! windows in one section. Every other field of its `panchanga_day` is
//! bounded by sunrise; the moonrise and moonset are the first at or after
//! the local civil midnight, so 24 of the 108 recorded events fall
//! outside the day the limbs occupy
//! (`03-design/panchanga-day-conventions.md` §9). That is a defect and
//! not a convention — a reader cannot tell which window a row belongs to
//! — so the SDK uses the day's own window and `panchanga.moon_events`
//! carries the engine's reading for the profile that reproduces it.
//!
//! Under the day's window there may be **none, one or two** of each: a
//! moonrise and the next are about 24 h 50 m apart, so a 24-hour window
//! sometimes holds two and sometimes none. The lists say so; a single
//! nullable field would not.

use serde::Serialize;
use teistro_core::catalogue::{Ayana, Rashi};
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Utc};

use crate::span::Span;

/// The first sidereal sign of uttarayana, Capricorn: the Sun's northward
/// half runs from Makara sankranti to the end of Gemini.
const FIRST_UTTARAYANA_SIGN: u16 = 9;
/// The last, Gemini.
const LAST_UTTARAYANA_SIGN: u16 = 2;

/// What the Moon did in the day.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct MoonDay {
    /// Every moonrise inside the window, in order.
    pub rises: Vec<JulianDay<Utc>>,
    /// Every moonset inside the window.
    pub sets: Vec<JulianDay<Utc>>,
    /// The signs the Moon stood in, with when it entered and left each.
    pub signs: Vec<Span<Rashi>>,
    /// Which window the rises and sets were found in, for the stamp.
    pub window: Interval,
}

impl MoonDay {
    /// The sign the Moon was in when the day opened.
    #[must_use]
    pub fn sign(&self) -> Option<Rashi> {
        self.signs.first().map(|span| span.member)
    }

    /// When the Moon changed sign inside the day, if it did.
    #[must_use]
    pub fn changed_sign_at(&self) -> Option<JulianDay<Utc>> {
        self.signs.get(1).map(|span| span.whole.from)
    }
}

/// What the Sun did in the day.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SunDay {
    /// The signs the Sun stood in; two only on a sankranti day.
    pub signs: Vec<Span<Rashi>>,
    /// Which half of the year the day falls in.
    pub ayana: Ayana,
    /// The sankranti inside the day, when the Sun entered a new sign.
    pub sankranti: Option<JulianDay<Utc>>,
}

impl SunDay {
    /// The sign the Sun was in when the day opened.
    #[must_use]
    pub fn sign(&self) -> Option<Rashi> {
        self.signs.first().map(|span| span.member)
    }
}

/// The half of the year a sidereal sign falls in.
///
/// Uttarayana runs while the sidereal Sun is in Capricorn to Gemini —
/// from Makara sankranti — which the corpus bears out on all 55 recorded
/// days (`03-design/panchanga-day-conventions.md` §9).
#[must_use]
pub fn ayana_of(sign: Rashi) -> Ayana {
    let index = sign.id();
    if index >= FIRST_UTTARAYANA_SIGN || index <= LAST_UTTARAYANA_SIGN {
        Ayana::Uttarayana
    } else {
        Ayana::Dakshinayana
    }
}

/// The Sun's day from the signs it stood in.
#[must_use]
pub fn sun_day(signs: Vec<Span<Rashi>>) -> SunDay {
    let ayana = signs
        .first()
        .map_or(Ayana::Uttarayana, |span| ayana_of(span.member));
    let sankranti = signs.get(1).map(|span| span.whole.from);
    SunDay {
        signs,
        ayana,
        sankranti,
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

    use super::{MoonDay, ayana_of, sun_day};
    use teistro_core::catalogue::{Ayana, Rashi};
    use teistro_core::interval::Interval;

    use crate::span::Span;

    fn window() -> Interval {
        Interval::literal(100.0, 101.0)
    }

    fn signs(members: &[(Rashi, f64, f64)]) -> Vec<Span<Rashi>> {
        members
            .iter()
            .filter_map(|(sign, from, to)| {
                Span::new(*sign, Interval::literal(*from, *to), window())
            })
            .collect()
    }

    #[test]
    fn uttarayana_runs_from_capricorn_to_gemini() {
        for sign in [
            Rashi::Capricorn,
            Rashi::Aquarius,
            Rashi::Pisces,
            Rashi::Aries,
            Rashi::Taurus,
            Rashi::Gemini,
        ] {
            assert_eq!(ayana_of(sign), Ayana::Uttarayana, "{sign:?}");
        }
        for sign in [
            Rashi::Cancer,
            Rashi::Leo,
            Rashi::Virgo,
            Rashi::Libra,
            Rashi::Scorpio,
            Rashi::Sagittarius,
        ] {
            assert_eq!(ayana_of(sign), Ayana::Dakshinayana, "{sign:?}");
        }
        // Six each, and every sign in one of them.
        assert_eq!(
            Rashi::ALL
                .iter()
                .filter(|sign| ayana_of(**sign) == Ayana::Uttarayana)
                .count(),
            6
        );
    }

    #[test]
    fn a_sankranti_is_the_second_sign_beginning() {
        let ordinary = sun_day(signs(&[(Rashi::Leo, 99.0, 130.0)]));
        assert_eq!(ordinary.sign(), Some(Rashi::Leo));
        assert_eq!(ordinary.ayana, Ayana::Dakshinayana);
        assert!(ordinary.sankranti.is_none());

        let turning = sun_day(signs(&[
            (Rashi::Sagittarius, 70.0, 100.4),
            (Rashi::Capricorn, 100.4, 130.0),
        ]));
        assert_eq!(turning.sign(), Some(Rashi::Sagittarius));
        assert_eq!(
            turning.ayana,
            Ayana::Dakshinayana,
            "the ayana is the sign the day opened in"
        );
        assert_eq!(
            turning
                .sankranti
                .map(teistro_core::quantity::JulianDay::get),
            Some(100.4)
        );
    }

    #[test]
    fn the_moon_reports_what_it_did_and_no_more() {
        let moon = MoonDay {
            rises: Vec::new(),
            sets: Vec::new(),
            signs: signs(&[(Rashi::Aries, 99.5, 100.3), (Rashi::Taurus, 100.3, 102.0)]),
            window: window(),
        };
        assert_eq!(moon.sign(), Some(Rashi::Aries));
        assert_eq!(
            moon.changed_sign_at()
                .map(teistro_core::quantity::JulianDay::get),
            Some(100.3)
        );
        // A day the Moon does not change sign in has no transition, and
        // one with no events reports none rather than a placeholder.
        let still = MoonDay {
            rises: Vec::new(),
            sets: Vec::new(),
            signs: signs(&[(Rashi::Aries, 99.0, 102.0)]),
            window: window(),
        };
        assert!(still.changed_sign_at().is_none());
        assert!(still.rises.is_empty());
    }
}
