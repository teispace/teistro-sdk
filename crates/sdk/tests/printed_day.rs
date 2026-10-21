//! A published panchanga's day page, row for row: the windows it prints
//! beside the limbs, held to the minute it prints over the SDK's own sky
//! (`03-design/amrit-kalam-and-lagnas.md`).
//!
//! The rows are Drik Panchang's for Delhi, 15 to 21 June 2025, read off
//! its day pages: a rank-3 almanac, under Lahiri like the default
//! context, so a row that parts names a reading, not a sky.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index fixed lists"
)]

use teistro::catalogue::Calendar;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{CalendarDate, Context, Ephemeris, Panchanga, UtcOffset};

/// A printed window: the June day it is printed on, and its start and end
/// as hours and minutes of Indian time, the end past midnight when it
/// is earlier than the start.
type Row = (u8, (u8, u8), (u8, u8));

fn week() -> Vec<Panchanga> {
    let delhi = Place::new(
        Latitude::literal(28.6139),
        Longitude::literal(77.209),
        Altitude::literal(216.0),
    );
    Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
        .almanac()
        .of(
            &CalendarDate::defined(Calendar::Gregorian, 2025, 6, 15),
            &CalendarDate::defined(Calendar::Gregorian, 2025, 6, 21),
            &delhi,
            UtcOffset::literal(5, 30, 0),
        )
        .unwrap()
        .value
}

/// An instant as the minute of Indian time the almanac prints, counted
/// from midnight beginning 15 June 2025: it rounds to the nearest.
fn printed_minute(jd: f64) -> i64 {
    // 15 June 2025 00:00 IST is JD 2460841.270833...
    let midnight = 2_460_841.5 - 5.5 / 24.0;
    #[allow(clippy::cast_possible_truncation, reason = "a week of minutes")]
    let minutes = ((jd - midnight) * 1440.0).round() as i64;
    minutes
}

fn minute_of(day: u8, (hour, minute): (u8, u8)) -> i64 {
    i64::from(day - 15) * 1440 + i64::from(hour) * 60 + i64::from(minute)
}

fn rows(windows: impl Iterator<Item = (f64, f64)>) -> Vec<(i64, i64)> {
    windows
        .map(|(from, to)| (printed_minute(from), printed_minute(to)))
        .collect()
}

/// Whether each window found is the printed one within the print's own
/// minute: the almanac's sky and the SDK's part by seconds, which carry
/// an edge across a minute either way.
fn within_the_minute(found: &[(i64, i64)], rows: &[Row]) {
    let printed = printed(rows);
    assert_eq!(found.len(), printed.len(), "{found:?} against {printed:?}");
    for (found, printed) in found.iter().zip(&printed) {
        assert!(
            (found.0 - printed.0).abs() <= 1 && (found.1 - printed.1).abs() <= 1,
            "{found:?} against {printed:?}"
        );
    }
}

fn printed(rows: &[Row]) -> Vec<(i64, i64)> {
    rows.iter()
        .map(|&(day, from, to)| {
            let start = minute_of(day, from);
            let end = minute_of(day, to);
            (start, if end < start { end + 1440 } else { end })
        })
        .collect()
}

/// Every amrita kalam the week's day pages print, from the reading the
/// published panchangas print (C677), each edge within the printed
/// minute: seven of seven.
#[test]
fn the_amrit_kalam_is_the_printed_one_to_the_minute() {
    let found: Vec<(i64, i64)> = rows(week().iter().flat_map(|day| {
        day.omens
            .amrit_kalam
            .iter()
            .filter(|span| span.whole.from.get() < day.day.next_sunrise.get())
            .filter(|span| span.whole.from.get() >= day.day.sunrise.get())
            .map(|span| (span.whole.from.get(), span.whole.to.get()))
    }));
    within_the_minute(
        &found,
        &[
            (15, (14, 19), (15, 58)),
            (16, (14, 43), (16, 20)),
            (17, (17, 53), (19, 28)),
            (18, (16, 36), (18, 9)),
            (19, (18, 42), (20, 13)),
            (20, (19, 30), (21, 0)),
            (21, (13, 12), (14, 41)),
        ],
    );
}

/// The varjyam the same pages print, from Muhurta Chintamani's table, the
/// default: seven of seven, Uttara Bhadrapada's 24 where Raman's table has
/// 30, and Bharani's before the sunrise of 22 June, which the 21st's page
/// prints unlabelled beside its day's.
#[test]
fn the_varjyam_is_the_printed_one_to_the_minute() {
    let found: Vec<(i64, i64)> = rows(week().iter().flat_map(|day| {
        day.omens
            .varjyam
            .iter()
            .filter(|span| span.whole.from.get() < day.day.next_sunrise.get())
            .filter(|span| span.whole.from.get() >= day.day.sunrise.get())
            .map(|span| (span.whole.from.get(), span.whole.to.get()))
    }));
    within_the_minute(
        &found,
        &[
            (16, (5, 2), (6, 39)),
            (17, (8, 22), (9, 57)),
            (18, (7, 15), (8, 48)),
            (19, (9, 32), (11, 4)),
            (20, (10, 31), (12, 0)),
            (21, (16, 9), (17, 37)),
            (22, (4, 33), (6, 0)),
        ],
    );
}
