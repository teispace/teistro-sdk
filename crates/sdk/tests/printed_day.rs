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

use teistro::catalogue::{Calendar, ChartKind, Rashi};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
use teistro::{AlmanacRequest, CalendarDate, Context, Ephemeris, Panchanga, UtcOffset};

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

/// The committee's daily lagna starts, Rashtriya Panchangam 2082 p. 6
/// ("दैनिक लग्न आरम्भ समयतालिका"), at Kathmandu in Nepal time: Asar 1 to
/// 11 (15 to 25 June 2025), each day's twelve signs from Mithuna, an hour
/// past 24 being the next morning. Mithuna rose twice on Asar 1, the
/// second time at 29:07, so each later day's Mithuna is the one before
/// its next sunrise.
const COMMITTEE_ASAR: [[(u8, u8); 12]; 11] = [
    [
        (5, 11),
        (7, 26),
        (9, 45),
        (12, 2),
        (14, 17),
        (16, 36),
        (18, 54),
        (20, 58),
        (22, 41),
        (24, 9),
        (25, 35),
        (27, 11),
    ],
    [
        (29, 3),
        (7, 22),
        (9, 42),
        (11, 58),
        (14, 13),
        (16, 32),
        (18, 50),
        (20, 54),
        (22, 37),
        (24, 5),
        (25, 31),
        (27, 7),
    ],
    [
        (28, 59),
        (7, 18),
        (9, 38),
        (11, 54),
        (14, 9),
        (16, 28),
        (18, 46),
        (20, 50),
        (22, 33),
        (24, 1),
        (25, 27),
        (27, 3),
    ],
    [
        (28, 55),
        (7, 14),
        (9, 34),
        (11, 50),
        (14, 5),
        (16, 24),
        (18, 42),
        (20, 46),
        (22, 29),
        (23, 57),
        (25, 23),
        (26, 59),
    ],
    [
        (28, 52),
        (7, 10),
        (9, 30),
        (11, 46),
        (14, 1),
        (16, 20),
        (18, 38),
        (20, 42),
        (22, 25),
        (23, 53),
        (25, 19),
        (26, 55),
    ],
    [
        (28, 48),
        (7, 6),
        (9, 26),
        (11, 42),
        (13, 57),
        (16, 16),
        (18, 34),
        (20, 38),
        (22, 21),
        (23, 49),
        (25, 15),
        (26, 51),
    ],
    [
        (28, 44),
        (7, 2),
        (9, 22),
        (11, 38),
        (13, 54),
        (16, 12),
        (18, 30),
        (20, 34),
        (22, 17),
        (23, 45),
        (25, 11),
        (26, 47),
    ],
    [
        (28, 40),
        (6, 58),
        (9, 18),
        (11, 34),
        (13, 50),
        (16, 8),
        (18, 26),
        (20, 30),
        (22, 13),
        (23, 41),
        (25, 7),
        (26, 44),
    ],
    [
        (28, 36),
        (6, 54),
        (9, 14),
        (11, 30),
        (13, 46),
        (16, 4),
        (18, 22),
        (20, 26),
        (22, 9),
        (23, 37),
        (25, 3),
        (26, 40),
    ],
    [
        (28, 32),
        (6, 50),
        (9, 10),
        (11, 26),
        (13, 42),
        (16, 0),
        (18, 18),
        (20, 22),
        (22, 5),
        (23, 34),
        (24, 59),
        (26, 36),
    ],
    [
        (28, 28),
        (6, 46),
        (9, 6),
        (11, 23),
        (13, 38),
        (15, 56),
        (18, 14),
        (20, 18),
        (22, 1),
        (23, 30),
        (24, 56),
        (26, 32),
    ],
];

/// The committee's lagna table is the sphere's ascendant in Lahiri's
/// zodiac (C680): under `nepal_official` every printed start of eleven
/// days stands before the SDK's cut by under two minutes, never after it
/// (`03-design/amrit-kalam-and-lagnas.md` §2.2). The text's own rising
/// times, the chart's lagna under the profile, miss by up to four and a
/// half minutes.
#[test]
fn the_committees_lagna_starts_are_the_spheres_under_lahiri() {
    let kathmandu = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1300.0),
    );
    let days = Context::builder()
        .nepal_official([])
        .build()
        .unwrap()
        .almanac()
        .asked(
            &CalendarDate::defined(Calendar::Gregorian, 2025, 6, 15),
            &CalendarDate::defined(Calendar::Gregorian, 2025, 6, 25),
            &kathmandu,
            UtcOffset::literal(5, 45, 0),
            &AlmanacRequest::new().with_lagnas(),
        )
        .unwrap()
        .days
        .value;
    // 15 June 2025 00:00 Nepal time.
    let midnight = 2_460_841.5 - 5.75 / 24.0;
    let mut worst = (f64::INFINITY, f64::NEG_INFINITY);
    for (k, (day, printed)) in days.iter().zip(COMMITTEE_ASAR).enumerate() {
        // Twelve signs, the one rising at sunrise again before the next,
        // and on Asar 1 Mithuna twice over besides.
        let expected = if k == 0 { 14 } else { 13 };
        assert_eq!(day.lagnas.len(), expected, "Asar {}", k + 1);
        for (s, (hour, minute)) in printed.into_iter().enumerate() {
            // Mithuna first, the third sign.
            let sign = Rashi::ALL[(s + 2) % 12];
            #[allow(clippy::cast_precision_loss, reason = "a day index")]
            let at = midnight + k as f64 + (f64::from(hour) * 60.0 + f64::from(minute)) / 1440.0;
            let cut = day
                .lagnas
                .iter()
                .filter(|span| span.member == sign)
                .map(|span| (span.whole.from.get() - at) * 86_400.0)
                .min_by(|a, b| a.abs().total_cmp(&b.abs()))
                .unwrap();
            assert!(
                (0.0..120.0).contains(&cut),
                "Asar {} {sign:?}: printed {hour}:{minute:02}, the cut {cut:.0} s after",
                k + 1
            );
            worst = (worst.0.min(cut), worst.1.max(cut));
        }
    }
    // Measured: 9 to 101 seconds. Each sign's lead grows some four
    // seconds a day through the month, the print stepping its starts by
    // 3 min 53 s a day where the sky steps 3 min 56 s, and falls back a
    // minute each time the printed minute steps (§2.2).
    assert!(worst.0 > 5.0 && worst.1 < 105.0, "{worst:?}");
}

/// A day's table tiles its arc: the spans run on without a gap, the first
/// begins at or before sunrise and the last ends at or after the next,
/// and each sign is the one rising at its span's middle.
#[test]
fn a_days_lagna_table_tiles_its_arc() {
    let delhi = Place::new(
        Latitude::literal(28.6139),
        Longitude::literal(77.209),
        Altitude::literal(216.0),
    );
    let sdk = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap();
    let date = CalendarDate::defined(Calendar::Gregorian, 2025, 6, 15);
    let offset = UtcOffset::literal(5, 30, 0);
    let answer = sdk
        .almanac()
        .asked(
            &date,
            &date,
            &delhi,
            offset,
            &AlmanacRequest::new().with_lagnas(),
        )
        .unwrap();
    let day = &answer.days.value[0];
    let table = &day.lagnas;
    assert!(table.len() >= 12, "{}", table.len());
    assert_eq!(table[0].inside.from, day.day.sunrise);
    assert_eq!(table[table.len() - 1].inside.to, day.day.next_sunrise);
    for pair in table.windows(2) {
        assert_eq!(pair[0].whole.to, pair[1].whole.from, "no gap");
        assert_eq!(pair[0].inside.to, pair[1].inside.from);
    }
    for span in table {
        let middle = f64::midpoint(span.whole.from.get(), span.whole.to.get());
        let chart = sdk
            .chart()
            .found(JulianDay::literal(middle), &delhi, offset, ChartKind::Natal)
            .unwrap();
        assert_eq!(
            Rashi::ALL[usize::from(chart.value.lagna_sign_index())],
            span.member
        );
    }
    // Not asked, the day carries none, and its document is as before.
    let plain = sdk.almanac().day(&date, &delhi, offset).unwrap();
    assert_eq!(plain.value.lagnas.len(), 0, "not asked, none");
    assert!(
        !serde_json::to_string(&plain.value)
            .unwrap()
            .contains("lagnas")
    );
}

/// The default context's table is Swiss Ephemeris's: pyswisseph's
/// sidereal Lahiri ascendant at Delhi on 15 June 2025, each sign's entry
/// bisected to a millisecond (sign index, Aries 0, and the Julian day,
/// UTC), against the SDK's own starts, within two seconds.
#[test]
fn the_default_table_cuts_where_swiss_ephemeris_does() {
    const SWISS: [(usize, f64); 13] = [
        (2, 2_460_841.498_227),
        (3, 2_460_841.591_719),
        (4, 2_460_841.689_155),
        (5, 2_460_841.784_478),
        (6, 2_460_841.879_115),
        (7, 2_460_841.975_975),
        (8, 2_460_842.072_189),
        (9, 2_460_842.158_285),
        (10, 2_460_842.229_324),
        (11, 2_460_842.290_064),
        (0, 2_460_842.349_076),
        (1, 2_460_842.415_229),
        (2, 2_460_842.495_496),
    ];
    let delhi = Place::new(
        Latitude::literal(28.6139),
        Longitude::literal(77.209),
        Altitude::literal(216.0),
    );
    let date = CalendarDate::defined(Calendar::Gregorian, 2025, 6, 15);
    let day = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
        .almanac()
        .asked(
            &date,
            &date,
            &delhi,
            UtcOffset::literal(5, 30, 0),
            &AlmanacRequest::new().with_lagnas(),
        )
        .unwrap()
        .days
        .value
        .remove(0);
    let starts: Vec<(Rashi, f64)> = day
        .lagnas
        .iter()
        .skip(1)
        .map(|span| (span.member, span.whole.from.get()))
        .collect();
    assert_eq!(starts.len(), SWISS.len());
    for ((sign, at), (index, swiss)) in starts.into_iter().zip(SWISS) {
        assert_eq!(sign, Rashi::ALL[index]);
        assert!(
            (at - swiss).abs() * 86_400.0 < 2.0,
            "{sign:?}: {at} against {swiss}"
        );
    }
}
