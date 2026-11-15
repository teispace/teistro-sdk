//! The polar day under `NEAREST_EVENT` on a modern sky (SR-103, C1900):
//! Tromsø's midnight sun and polar night of 2025, each date its own day.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    missing_docs,
    reason = "a test fails by panicking"
)]

use teistro::catalogue::Calendar;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{CalendarDate, Context, Ephemeris, UtcOffset};

/// Tromsø's days from `first` on, under `nepali-drik` (whose policy is
/// `NEAREST_EVENT`): each 23 to 25 hours, touching the next, holding at
/// most three tithis, and each date's own.
fn assert_own_days(first: (i32, u8, u8), count: u8, clock: UtcOffset) {
    let sdk = Context::builder()
        .profile("nepali-drik")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("a context");
    let tromso = Place::new(
        Latitude::literal(69.6492),
        Longitude::literal(18.9553),
        Altitude::literal(0.0),
    );
    let days: Vec<_> = (0..count)
        .map(|n| {
            let date = CalendarDate::defined(Calendar::Gregorian, first.0, first.1, first.2 + n);
            sdk.almanac().day(&date, &tromso, clock).unwrap().value
        })
        .collect();
    for pair in days.windows(2) {
        let [this, next] = pair else { continue };
        assert_eq!(this.day.next_sunrise, next.day.sunrise, "{}", this.day.date);
    }
    for day in &days {
        let hours = (day.day.next_sunrise.get() - day.day.sunrise.get()) * 24.0;
        assert!(
            (23.0..=25.0).contains(&hours),
            "{}: {hours} h",
            day.day.date
        );
        assert!((1..=3).contains(&day.limbs.tithi.len()), "{}", day.day.date);
    }
}

/// The consumer's case: 18 to 26 May 2025 answered one 68-day window
/// holding 72 tithis; each date is now a day of its own.
#[test]
fn tromso_s_midnight_sun_begins_day_by_day() {
    assert_own_days((2025, 5, 18), 9, UtcOffset::literal(2, 0, 0));
    assert_own_days((2025, 7, 22), 7, UtcOffset::literal(2, 0, 0));
}

/// The twin: the polar night's first days, and its last.
#[test]
fn tromso_s_polar_night_begins_and_ends_day_by_day() {
    assert_own_days((2025, 11, 24), 7, UtcOffset::literal(1, 0, 0));
    assert_own_days((2026, 1, 12), 7, UtcOffset::literal(1, 0, 0));
}
