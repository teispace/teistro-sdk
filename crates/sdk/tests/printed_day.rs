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
    // 15 June 2025 00:00 IST is JD 2_460_841.270_833...
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

/// The committee's printed places at sunrise (Kathmandu), the first row of
/// each half-month page of VS 2082 and 2083 (page, the UT Julian day of
/// that morning's sunrise, then Mars, Mercury, Jupiter, Venus and Saturn,
/// `sign:degree:minute:second` in its zodiac).
const COMMITTEE_PLANETS: [(&str, f64, [&str; 5]); 49] = [
    (
        "2082-02",
        2_460_779.496_830,
        [
            "3:4:33:34",
            "11:5:59:23",
            "1:24:7:9",
            "11:1:21:24",
            "11:1:47:21",
        ],
    ),
    (
        "2082-03",
        2_460_793.487_558,
        [
            "3:10:38:54",
            "11:17:38:2",
            "1:26:40:16",
            "11:4:37:29",
            "11:3:15:55",
        ],
    ),
    (
        "2082-04",
        2_460_808.479_886,
        [
            "3:17:45:37",
            "0:9:56:37",
            "1:29:44:15",
            "11:13:54:25",
            "11:4:39:35",
        ],
    ),
    (
        "2082-05",
        2_460_823.475_279,
        [
            "3:25:23:13",
            "1:10:12:55",
            "2:2:59:15",
            "11:26:36:47",
            "11:5:50:38",
        ],
    ),
    (
        "2082-06",
        2_460_838.474_072,
        [
            "4:3:26:48",
            "2:11:38:10",
            "2:6:18:42",
            "0:11:11:55",
            "11:6:46:8",
        ],
    ),
    (
        "2082-07",
        2_460_852.475_796,
        [
            "4:11:15:46",
            "3:5:4:43",
            "2:9:26:38",
            "0:25:51:35",
            "11:7:18:24",
        ],
    ),
    (
        "2082-08",
        2_460_867.479_906,
        [
            "4:19:55:11",
            "3:18:15:52",
            "2:12:47:49",
            "1:12:20:42",
            "11:7:30:53",
        ],
    ),
    (
        "2082-09",
        2_460_881.484_883,
        [
            "4:28:14:22",
            "3:18:26:38",
            "2:15:51:52",
            "1:28:12:55",
            "11:7:25:1",
        ],
    ),
    (
        "2082-10",
        2_460_897.490_848,
        [
            "5:8:0:45",
            "3:9:43:11",
            "2:19:18:18",
            "2:16:47:49",
            "11:6:56:38",
        ],
    ),
    (
        "2082-11",
        2_460_911.495_782,
        [
            "5:16:47:8",
            "3:19:21:9",
            "2:22:2:4",
            "3:3:27:29",
            "11:6:12:28",
        ],
    ),
    (
        "2082-12",
        2_460_926.500_674,
        [
            "5:26:24:17",
            "4:16:26:28",
            "2:24:42:16",
            "3:21:30:25",
            "11:5:13:56",
        ],
    ),
    (
        "2082-13",
        2_460_940.505_121,
        [
            "6:5:37:51",
            "5:12:9:42",
            "2:26:57:31",
            "4:8:39:26",
            "11:4:8:33",
        ],
    ),
    (
        "2082-14",
        2_460_956.510_571,
        [
            "6:16:20:59",
            "6:8:34:36",
            "2:28:58:34",
            "4:28:30:16",
            "11:2:54:48",
        ],
    ),
    (
        "2082-15",
        2_460_970.516_111,
        [
            "6:26:12:12",
            "6:27:40:59",
            "3:0:10:31",
            "5:15:58:34",
            "11:1:59:59",
        ],
    ),
    (
        "2082-16",
        2_460_985.523_095,
        [
            "7:6:52:38",
            "7:11:46:40",
            "3:0:52:13",
            "6:4:40:43",
            "11:1:16:12",
        ],
    ),
    (
        "2082-17",
        2_461_000.530_968,
        [
            "7:17:44:57",
            "7:3:3:11",
            "3:0:46:41",
            "6:23:33:47",
            "11:0:50:8",
        ],
    ),
    (
        "2082-18",
        2_461_014.538_371,
        [
            "7:28:3:49",
            "6:28:58:58",
            "3:0:4:7",
            "7:11:7:56",
            "11:0:52:15",
        ],
    ),
    (
        "2082-19",
        2_461_029.545_030,
        [
            "8:9:20:7",
            "7:16:31:42",
            "2:28:40:23",
            "8:0:0:27",
            "11:1:13:2",
        ],
    ),
    (
        "2082-20",
        2_461_044.548_878,
        [
            "8:20:50:25",
            "8:8:44:34",
            "2:26:45:14",
            "8:18:51:21",
            "11:1:59:16",
        ],
    ),
    (
        "2082-21",
        2_461_059.548_898,
        [
            "9:2:28:35",
            "9:3:1:18",
            "2:24:37:43",
            "9:7:39:33",
            "11:3:8:41",
        ],
    ),
    (
        "2082-22",
        2_461_073.545_290,
        [
            "9:13:25:16",
            "9:26:54:45",
            "2:22:53:28",
            "9:25:12:13",
            "11:4:24:19",
        ],
    ),
    (
        "2082-23",
        2_461_089.537_463,
        [
            "9:25:56:9",
            "10:22:43:20",
            "2:21:28:27",
            "10:15:11:25",
            "11:6:5:41",
        ],
    ),
    (
        "2082-24",
        2_461_103.528_233,
        [
            "10:6:56:5",
            "10:26:9:50",
            "2:20:49:30",
            "11:2:38:27",
            "11:7:44:51",
        ],
    ),
    (
        "2082-25",
        2_461_119.516_146,
        [
            "10:19:30:1",
            "10:15:36:32",
            "2:20:55:35",
            "11:22:34:38",
            "11:9:46:24",
        ],
    ),
    (
        "2083-02",
        2_461_144.497_000,
        [
            "11:9:0:57",
            "11:4:17:57",
            "2:22:35:19",
            "0:23:28:53",
            "11:12:51:8",
        ],
    ),
    (
        "2083-03",
        2_461_148.494_184,
        [
            "11:12:6:36",
            "11:9:29:37",
            "2:22:59:21",
            "0:28:25:12",
            "11:13:20:19",
        ],
    ),
    (
        "2083-04",
        2_461_162.485_384,
        [
            "11:22:51:51",
            "0:3:30:45",
            "2:24:50:6",
            "1:15:36:6",
            "11:14:56:7",
        ],
    ),
    (
        "2083-05",
        2_461_177.478_422,
        [
            "0:4:11:25",
            "1:4:45:27",
            "2:27:14:15",
            "2:3:47:7",
            "11:16:27:35",
        ],
    ),
    (
        "2083-06",
        2_461_192.474_656,
        [
            "0:15:20:46",
            "2:5:16:42",
            "2:29:51:20",
            "2:21:49:7",
            "11:17:45:40",
        ],
    ),
    (
        "2083-07",
        2_461_207.474_288,
        [
            "0:26:19:5",
            "2:25:0:42",
            "3:2:47:23",
            "3:9:26:56",
            "11:18:53:18",
        ],
    ),
    (
        "2083-08",
        2_461_221.476_645,
        [
            "1:6:21:4",
            "3:0:34:57",
            "3:5:45:0",
            "3:25:31:5",
            "11:19:39:54",
        ],
    ),
    (
        "2083-09",
        2_461_236.481_175,
        [
            "1:16:51:0",
            "2:25:21:33",
            "3:8:59:35",
            "4:12:15:58",
            "11:20:5:38",
        ],
    ),
    (
        "2083-10",
        2_461_251.486_664,
        [
            "1:27:6:51",
            "2:23:2:16",
            "3:12:17:51",
            "4:28:13:35",
            "11:20:15:2",
        ],
    ),
    (
        "2083-11",
        2_461_265.491_850,
        [
            "2:6:31:52",
            "3:11:42:45",
            "3:15:23:22",
            "5:12:9:35",
            "11:20:0:22",
        ],
    ),
    (
        "2083-12",
        2_461_281.497_368,
        [
            "2:16:59:17",
            "4:12:42:35",
            "3:18:52:11",
            "5:26:23:35",
            "11:19:23:52",
        ],
    ),
    (
        "2083-13",
        2_461_295.501_865,
        [
            "2:26:12:21",
            "5:8:24:8",
            "3:21:51:45",
            "6:6:12:9",
            "11:18:36:30",
        ],
    ),
    (
        "2083-14",
        2_461_310.506_673,
        [
            "3:5:0:5",
            "6:1:49:4",
            "3:24:45:37",
            "6:12:17:27",
            "11:17:32:44",
        ],
    ),
    (
        "2083-15",
        2_461_324.511_594,
        [
            "3:13:8:25",
            "6:18:41:17",
            "3:27:12:29",
            "6:12:18:39",
            "11:16:25:29",
        ],
    ),
    (
        "2083-16",
        2_461_340.518_203,
        [
            "3:21:52:10",
            "6:26:49:38",
            "3:29:43:58",
            "6:5:30:39",
            "11:15:15:39",
        ],
    ),
    (
        "2083-17",
        2_461_354.525_004,
        [
            "3:28:57:4",
            "6:11:11:34",
            "4:1:23:42",
            "5:29:38:40",
            "11:14:26:10",
        ],
    ),
    (
        "2083-18",
        2_461_369.532_989,
        [
            "4:5:46:7",
            "6:19:34:3",
            "4:2:34:53",
            "6:1:59:52",
            "11:13:49:48",
        ],
    ),
    (
        "2083-19",
        2_461_383.540_220,
        [
            "4:10:55:52",
            "7:9:34:26",
            "4:3:7:31",
            "6:10:9:50",
            "11:13:35:24",
        ],
    ),
    (
        "2083-20",
        2_461_399.546_616,
        [
            "4:15:1:35",
            "8:4:31:35",
            "4:2:50:19",
            "6:23:20:8",
            "11:13:45:26",
        ],
    ),
    (
        "2083-21",
        2_461_413.549_266,
        [
            "4:16:26:41",
            "8:27:14:47",
            "4:1:59:27",
            "7:7:9:35",
            "11:14:16:50",
        ],
    ),
    (
        "2083-22",
        2_461_428.548_263,
        [
            "4:15:24:48",
            "9:22:0:58",
            "4:0:23:22",
            "7:23:14:49",
            "11:15:14:34",
        ],
    ),
    (
        "2083-23",
        2_461_443.543_338,
        [
            "4:11:35:38",
            "10:11:17:46",
            "3:28:21:35",
            "8:10:9:10",
            "11:16:29:33",
        ],
    ),
    (
        "2083-24",
        2_461_457.535_796,
        [
            "4:6:11:51",
            "10:2:41:5",
            "3:26:20:1",
            "8:26:25:20",
            "11:17:51:6",
        ],
    ),
    (
        "2083-25",
        2_461_473.524_764,
        [
            "4:0:2:0",
            "9:29:9:30",
            "3:24:24:20",
            "9:15:18:8",
            "11:19:37:5",
        ],
    ),
    (
        "2083-26",
        2_461_487.513_998,
        [
            "3:27:30:13",
            "10:11:1:2",
            "3:23:13:1",
            "10:1:59:22",
            "11:21:21:39",
        ],
    ),
];

/// `sign:degree:minute:second` to degrees.
fn printed_degrees(text: &str) -> f64 {
    let parts: Vec<f64> = text.split(':').map(|p| p.parse().unwrap()).collect();
    parts[0] * 30.0 + parts[1] + parts[2] / 60.0 + parts[3] / 3600.0
}

/// Each star planet's root mean square miss of the print, in arcminutes,
/// over a context's charts at the printed mornings.
fn misses(context: &Context) -> [f64; 5] {
    use teistro::catalogue::Graha;
    let kathmandu = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1300.0),
    );
    let grahas = [
        Graha::Mars,
        Graha::Mercury,
        Graha::Jupiter,
        Graha::Venus,
        Graha::Saturn,
    ];
    let mut squares = [0.0; 5];
    for (_, jd, printed) in COMMITTEE_PLANETS {
        let chart = context
            .chart()
            .found(
                JulianDay::literal(jd),
                &kathmandu,
                UtcOffset::literal(5, 45, 0),
                ChartKind::Natal,
            )
            .unwrap()
            .value;
        for (j, graha) in grahas.into_iter().enumerate() {
            let ours = chart
                .grahas
                .iter()
                .find(|g| g.graha == graha)
                .unwrap()
                .longitude_deg;
            let apart =
                ((printed_degrees(printed[j]) - ours + 540.0).rem_euclid(360.0) - 180.0) * 60.0;
            squares[j] += apart * apart;
        }
    }
    #[allow(clippy::cast_precision_loss, reason = "a row count")]
    let n = COMMITTEE_PLANETS.len() as f64;
    squares.map(|s| (s / n).sqrt())
}

/// The committee's star planets are not the text's (C38): over 49 printed
/// mornings of VS 2082 and 2083, a modern sky in Lahiri's zodiac misses
/// them by a root mean square of 9 to 33 arcminutes, the Surya
/// Siddhanta's own planets under `nepal_official` by 94 to 460, and no
/// whole bija on the text's counts comes nearer than 20
/// (`03-design/committee-star-planets.md`).
#[test]
fn the_committees_star_planets_are_nearer_a_modern_sky_than_the_text() {
    let modern = misses(
        &Context::builder()
            .ephemeris([Ephemeris::Builtin])
            .build()
            .unwrap(),
    );
    let text = misses(&Context::builder().nepal_official([]).build().unwrap());
    for j in 0..5 {
        assert!(modern[j] < 35.0, "{j}: {modern:?}");
        assert!(
            text[j] > 3.0 * modern[j],
            "{j}: {text:?} against {modern:?}"
        );
    }
}
