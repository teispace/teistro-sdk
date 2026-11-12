//! `INDIA_GUJARAT` against each year's notification of the General
//! Administration Department, 2019 to 2026, at Gandhinagar under the
//! modern sky (`03-design/gujarat-holidays.md`): every reckoned day of
//! each year's Part I and Note (1) meets the print but the named
//! partings, the fixed days stand on their date, and the printed days
//! stand as printed. A test for each year.

#![cfg(feature = "builtin-ephemeris")]
#![allow(
    clippy::unwrap_used,
    missing_docs,
    reason = "a test fails by panicking"
)]

use teistro::catalogue::Calendar;
use teistro::festival::{Observances, SubjectTo};
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{CalendarDate, Context, Ephemeris, FestivalPack, FestivalRequest, UtcOffset};

/// The keys of each year's reckoned rows, in this order: every
/// notification of 2019 to 2026 lists each, in Part I or its Note (1).
const KEYS: [&str; 15] = [
    "SHIVARATRI",
    "HOLI",
    "CHETI_CHAND",
    "RAMA_NAVAMI",
    "MAHAVIR_JAYANTI",
    "GOOD_FRIDAY",
    "PARASHURAMA_JAYANTI",
    "RAKSHABANDHAN",
    "JANMASHTAMI",
    "SAMVATSARI",
    "VIJAYA_DASHAMI",
    "LAKSHMI_PUJA",
    "VIKRAM_SAMVAT_NEW_YEAR",
    "YAMA_DWITIYA",
    "GURU_NANAK_JAYANTI",
];

/// Each notification's days as (month, day), in the order of [`KEYS`].
type Year = (i32, [(u8, u8); 15]);

const PRINTED: [Year; 8] = [
    (
        2019,
        [
            (3, 4),
            (3, 21),
            (4, 6),
            (4, 14),
            (4, 17),
            (4, 19),
            (5, 7),
            (8, 15),
            (8, 24),
            (9, 2),
            (10, 8),
            (10, 27),
            (10, 28),
            (10, 29),
            (11, 12),
        ],
    ),
    (
        2020,
        [
            (2, 21),
            (3, 10),
            (3, 25),
            (4, 2),
            (4, 6),
            (4, 10),
            (4, 25),
            (8, 3),
            (8, 12),
            (8, 22),
            (10, 25),
            (11, 14),
            (11, 16),
            (11, 16),
            (11, 30),
        ],
    ),
    (
        2021,
        [
            (3, 11),
            (3, 29),
            (4, 13),
            (4, 21),
            (4, 25),
            (4, 2),
            (5, 14),
            (8, 22),
            (8, 30),
            (9, 10),
            (10, 15),
            (11, 4),
            (11, 5),
            (11, 6),
            (11, 19),
        ],
    ),
    (
        2022,
        [
            (3, 1),
            (3, 18),
            (4, 2),
            (4, 10),
            (4, 14),
            (4, 15),
            (5, 3),
            (8, 11),
            (8, 19),
            (8, 31),
            (10, 5),
            (10, 24),
            (10, 26),
            (10, 26),
            (11, 8),
        ],
    ),
    (
        2023,
        [
            (2, 18),
            (3, 8),
            (3, 22),
            (3, 30),
            (4, 4),
            (4, 7),
            (4, 22),
            (8, 30),
            (9, 7),
            (9, 19),
            (10, 24),
            (11, 12),
            (11, 14),
            (11, 15),
            (11, 27),
        ],
    ),
    (
        2024,
        [
            (3, 8),
            (3, 25),
            (4, 10),
            (4, 17),
            (4, 21),
            (3, 29),
            (5, 10),
            (8, 19),
            (8, 26),
            (9, 7),
            (10, 12),
            (10, 31),
            (11, 2),
            (11, 3),
            (11, 15),
        ],
    ),
    (
        2025,
        [
            (2, 26),
            (3, 14),
            (3, 30),
            (4, 6),
            (4, 10),
            (4, 18),
            (4, 29),
            (8, 9),
            (8, 16),
            (8, 27),
            (10, 2),
            (10, 20),
            (10, 22),
            (10, 23),
            (11, 5),
        ],
    ),
    (
        2026,
        [
            (2, 15),
            (3, 4),
            (3, 19),
            (3, 26),
            (3, 31),
            (4, 3),
            (4, 19),
            (8, 28),
            (9, 4),
            (9, 15),
            (10, 20),
            (11, 8),
            (11, 10),
            (11, 11),
            (11, 24),
        ],
    ),
];

/// A row that parts: the year, the key and the days the rule gives.
type Part = (i32, &'static str, &'static [(u8, u8)]);

/// The rows that part from the print, each with the days the rule gives
/// (`festival-measured.md` §17).
const PARTS: [Part; 7] = [
    (2019, "RAMA_NAVAMI", &[(4, 13)]),
    (2019, "VIJAYA_DASHAMI", &[(10, 7)]),
    (2022, "VIJAYA_DASHAMI", &[(10, 4)]),
    (2023, "HOLI", &[(3, 7)]),
    (2023, "MAHAVIR_JAYANTI", &[(4, 3)]),
    (2024, "CHETI_CHAND", &[(4, 9)]),
    (2026, "HOLI", &[(3, 3)]),
];

/// The days each year's notification prints rather than reckons, as
/// (marked, unmarked): the four set by the moon, and the Parsi New Year.
const PRINTED_DAYS: (usize, usize) = (4, 1);

/// The days every notification lists on one date.
const CIVIL: [(&str, (u8, u8)); 7] = [
    ("UTTARAYANA_PUNYAKALA", (1, 14)),
    ("INDIA_REPUBLIC_DAY", (1, 26)),
    ("AMBEDKAR_JAYANTI", (4, 14)),
    ("INDIA_INDEPENDENCE_DAY", (8, 15)),
    ("GANDHI_JAYANTI", (10, 2)),
    ("SARDAR_PATEL_JAYANTI", (10, 31)),
    ("CHRISTMAS", (12, 25)),
];

fn gandhinagar() -> Place {
    Place::new(
        Latitude::literal(23.2156),
        Longitude::literal(72.6369),
        Altitude::literal(0.0),
    )
}

fn date(year: i32, month: u8, day: u8) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, year, month, day)
}

/// The modern sky: the built-in ephemeris under the default profile.
fn modern_sky() -> Context {
    Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
}

/// The days the answer gives `key`, of whichever kind.
fn days_of(answer: &Observances, key: &str) -> Vec<(u8, u8)> {
    let lunar = answer
        .observances
        .iter()
        .filter(|found| found.rule == key)
        .map(|found| (found.day.month, found.day.day));
    let dated = answer
        .dated
        .iter()
        .filter(|found| found.rule == key)
        .map(|found| (found.day.month, found.day.day));
    lunar.chain(dated).collect()
}

/// Holds `year`'s notification to the pack: each reckoned row meets the
/// print or is a named parting giving the days named, the fixed days
/// stand on their date, and the printed days stand, the moon's marked.
fn meets(year: i32) {
    let (_, printed) = PRINTED.iter().find(|row| row.0 == year).unwrap();
    let (marked, unmarked) = PRINTED_DAYS;
    let answer = modern_sky()
        .almanac()
        .festivals(
            &date(year, 1, 1),
            &date(year, 12, 31),
            &gandhinagar(),
            UtcOffset::literal(5, 30, 0),
            &FestivalRequest::from(FestivalPack::IndiaGujarat),
        )
        .unwrap()
        .value;
    let parted: Vec<(&str, Vec<(u8, u8)>)> = KEYS
        .iter()
        .zip(printed)
        .filter_map(|(key, day)| {
            let found = days_of(&answer, key);
            (found != [*day]).then_some((*key, found))
        })
        .collect();
    let named: Vec<(&str, Vec<(u8, u8)>)> = PARTS
        .iter()
        .filter(|part| part.0 == year)
        .map(|&(_, key, days)| (key, days.to_vec()))
        .collect();
    assert_eq!(parted, named, "{year}");
    for (key, day) in CIVIL {
        assert_eq!(days_of(&answer, key), [day], "{year} {key}");
    }
    let moon = answer
        .added
        .iter()
        .filter(|added| added.published.subject_to == Some(SubjectTo::MoonSighting))
        .count();
    assert_eq!(moon, marked, "{year}");
    assert_eq!(answer.added.len(), marked + unmarked, "{year}");
}

#[test]
fn the_notification_for_2019_meets_the_pack() {
    meets(2019);
}

#[test]
fn the_notification_for_2020_meets_the_pack() {
    meets(2020);
}

#[test]
fn the_notification_for_2021_meets_the_pack() {
    meets(2021);
}

#[test]
fn the_notification_for_2022_meets_the_pack() {
    meets(2022);
}

#[test]
fn the_notification_for_2023_meets_the_pack() {
    meets(2023);
}

#[test]
fn the_notification_for_2024_meets_the_pack() {
    meets(2024);
}

#[test]
fn the_notification_for_2025_meets_the_pack() {
    meets(2025);
}

#[test]
fn the_notification_for_2026_meets_the_pack() {
    meets(2026);
}
