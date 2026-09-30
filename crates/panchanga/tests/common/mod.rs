//! The synthetic days every festival test builds on: day `k` rises at
//! Julian day `k`, sets at `k + 0.5`, and a ghati is a sixtieth of a day,
//! so the book's ghatis after sunrise are read as written.

#![allow(
    dead_code,
    clippy::unwrap_used,
    reason = "each test file uses part of this, and a test fails by panicking"
)]

use teistro_calendar::CalendarDate;
use teistro_calendar::lunisolar::MonthKind;
use teistro_core::catalogue::{Calendar, Masa, Nakshatra, Tithi};
use teistro_core::interval::Interval;
use teistro_core::quantity::JulianDay;
use teistro_core::settings::LunarMonth as Convention;
use teistro_panchanga::festival::FestivalDay;
use teistro_panchanga::month;
use teistro_panchanga::span::Span;

pub(crate) fn ghati(day: usize, ghatis: f64) -> f64 {
    #[allow(clippy::cast_precision_loss, reason = "a handful of days")]
    let day = day as f64;
    day + ghatis / 60.0
}

/// Five synthetic days over a run of tithis, each `(tithi, ends)` ending
/// at a Julian day, in a month named `amanta`, of `kind`.
pub(crate) fn days_of(
    tithis: &[(Tithi, f64)],
    nakshatras: &[(Nakshatra, f64)],
    amanta: Masa,
    kind: MonthKind,
) -> Vec<FestivalDay> {
    days_over(5, tithis, nakshatras, amanta, kind)
}

/// As [`days_of`], over `count` days.
pub(crate) fn days_over(
    count: usize,
    tithis: &[(Tithi, f64)],
    nakshatras: &[(Nakshatra, f64)],
    amanta: Masa,
    kind: MonthKind,
) -> Vec<FestivalDay> {
    fn spans<T: Copy>(runs: &[(T, f64)], window: Interval) -> Vec<Span<T>> {
        let mut from = 0.0;
        let mut out = Vec::new();
        for &(member, to) in runs {
            if let Some(span) = Span::new(member, Interval::literal(from, to), window) {
                out.push(span);
            }
            from = to;
        }
        out
    }
    (0..count)
        .map(|k| {
            let rise = ghati(k, 0.0);
            let window = Interval::literal(rise, rise + 1.0);
            let tithi = spans(tithis, window);
            let at_sunrise = tithi
                .first()
                .map_or(Tithi::ShuklaPratipada, |span| span.member);
            FestivalDay {
                date: CalendarDate::defined(
                    Calendar::Gregorian,
                    2024,
                    1,
                    u8::try_from(k + 1).unwrap(),
                ),
                sunrise: JulianDay::literal(rise),
                sunset: JulianDay::literal(rise + 0.5),
                next_sunrise: JulianDay::literal(rise + 1.0),
                normal: true,
                month: month::of(amanta, at_sunrise, Convention::Amanta, kind),
                tithi,
                nakshatra: spans(nakshatras, window),
            }
        })
        .collect()
}
