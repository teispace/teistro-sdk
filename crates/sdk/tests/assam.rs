//! `INDIA_ASSAM` against each year's notification of the General
//! Administration Department, 2019 to 2026, at Guwahati under the modern
//! sky (`03-design/assam-holidays.md`): every reckoned day of each year
//! meets the print but the named partings, and the days set by the moon
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

/// The keys of each year's row, in this order.
const KEYS: [&str; 28] = [
    "MAGH_BIHU",
    "MAGH_BIHU_SECOND",
    "GWTHAR_BATHOU_SAN",
    "BIR_CHILARAI_DIVAS",
    "DOLA_YATRA",
    "BOHAG_BIHU",
    "BOHAG_BIHU_SECOND",
    "BOHAG_BIHU_THIRD",
    "SATI_SADHANI_DIVAS",
    "DAMODARDEVA_TITHI",
    "BUDDHA_JAYANTI",
    "MADHABDEVA_JANMOTSAV",
    "MADHABDEVA_TIROBHAV",
    "JANMASHTAMI",
    "SANKARDEVA_TIROBHAV",
    "KARAM_PUJA",
    "SANKARDEVA_JANMOTSAV",
    "PHULPATI",
    "MAHA_ASHTAMI",
    "MAHA_NAVAMI",
    "VIJAYA_DASHAMI",
    "KATI_BIHU",
    "KALI_PUJA",
    "YAMA_DWITIYA",
    "CHHATH",
    "GURU_NANAK_JAYANTI",
    "GOOD_FRIDAY",
    "KOJAGRAT_PURNIMA",
];

/// Each notification's days as (month, day), in the order of [`KEYS`];
/// `None` where the year's notification has no such row (Sati Sadhani
/// Divas, Karam Puja and Bhatri Dwitiya before 2021). Lakshmi Puja is the
/// half holiday.
type Year = (i32, [Option<(u8, u8)>; 28]);

const PRINTED: [Year; 8] = [
    (
        2019,
        [
            Some((1, 15)),
            Some((1, 16)),
            Some((1, 29)),
            Some((2, 19)),
            Some((3, 21)),
            Some((4, 14)),
            Some((4, 15)),
            Some((4, 16)),
            None,
            Some((5, 5)),
            Some((5, 18)),
            Some((5, 19)),
            Some((8, 20)),
            Some((8, 24)),
            Some((9, 1)),
            None,
            Some((10, 8)),
            Some((10, 5)),
            Some((10, 6)),
            Some((10, 7)),
            Some((10, 8)),
            Some((10, 18)),
            Some((10, 27)),
            None,
            Some((11, 2)),
            Some((11, 12)),
            Some((4, 19)),
            Some((10, 13)),
        ],
    ),
    (
        2020,
        [
            Some((1, 15)),
            Some((1, 16)),
            Some((1, 28)),
            Some((2, 9)),
            Some((3, 9)),
            Some((4, 13)),
            Some((4, 14)),
            Some((4, 15)),
            None,
            Some((4, 23)),
            Some((5, 7)),
            Some((6, 6)),
            Some((9, 7)),
            Some((9, 9)),
            Some((8, 20)),
            None,
            Some((9, 26)),
            Some((10, 23)),
            Some((10, 24)),
            Some((10, 25)),
            Some((10, 26)),
            Some((10, 17)),
            Some((11, 14)),
            None,
            Some((11, 20)),
            Some((11, 30)),
            Some((4, 10)),
            Some((10, 30)),
        ],
    ),
    (
        2021,
        [
            Some((1, 14)),
            Some((1, 15)),
            Some((1, 26)),
            Some((2, 27)),
            Some((3, 28)),
            Some((4, 14)),
            Some((4, 15)),
            Some((4, 16)),
            Some((4, 21)),
            Some((5, 12)),
            Some((5, 26)),
            Some((5, 27)),
            Some((8, 27)),
            Some((8, 30)),
            Some((9, 8)),
            Some((8, 29)),
            Some((10, 15)),
            Some((10, 12)),
            Some((10, 13)),
            Some((10, 14)),
            Some((10, 15)),
            Some((10, 18)),
            Some((11, 4)),
            Some((11, 6)),
            Some((11, 10)),
            Some((11, 19)),
            Some((4, 2)),
            Some((10, 20)),
        ],
    ),
    (
        2022,
        [
            Some((1, 14)),
            Some((1, 15)),
            Some((1, 25)),
            Some((2, 16)),
            Some((3, 18)),
            Some((4, 14)),
            Some((4, 15)),
            Some((4, 16)),
            Some((4, 21)),
            Some((5, 1)),
            Some((5, 16)),
            Some((5, 17)),
            Some((9, 14)),
            Some((8, 19)),
            Some((8, 29)),
            Some((9, 6)),
            Some((10, 5)),
            Some((10, 2)),
            Some((10, 3)),
            Some((10, 4)),
            Some((10, 5)),
            Some((10, 18)),
            Some((10, 24)),
            Some((10, 27)),
            Some((10, 30)),
            Some((11, 8)),
            Some((4, 15)),
            Some((10, 9)),
        ],
    ),
    (
        2023,
        [
            Some((1, 15)),
            Some((1, 16)),
            Some((1, 24)),
            Some((2, 5)),
            Some((3, 7)),
            Some((4, 14)),
            Some((4, 15)),
            Some((4, 16)),
            Some((4, 21)),
            Some((4, 20)),
            Some((5, 5)),
            Some((6, 5)),
            Some((9, 4)),
            Some((9, 6)),
            Some((8, 18)),
            Some((8, 27)),
            Some((9, 25)),
            Some((10, 21)),
            Some((10, 22)),
            Some((10, 23)),
            Some((10, 24)),
            Some((10, 18)),
            Some((11, 12)),
            Some((11, 15)),
            Some((11, 19)),
            Some((11, 27)),
            Some((4, 7)),
            Some((10, 28)),
        ],
    ),
    (
        2024,
        [
            Some((1, 15)),
            Some((1, 16)),
            Some((1, 23)),
            Some((2, 24)),
            Some((3, 25)),
            Some((4, 13)),
            Some((4, 14)),
            Some((4, 15)),
            Some((4, 20)),
            Some((5, 8)),
            Some((5, 23)),
            Some((5, 24)),
            Some((8, 23)),
            Some((8, 26)),
            Some((9, 4)),
            Some((9, 14)),
            Some((10, 13)),
            Some((10, 10)),
            Some((10, 11)),
            Some((10, 12)),
            Some((10, 13)),
            Some((10, 17)),
            Some((10, 31)),
            Some((11, 3)),
            Some((11, 7)),
            Some((11, 15)),
            Some((3, 29)),
            Some((10, 16)),
        ],
    ),
    (
        2025,
        [
            Some((1, 14)),
            Some((1, 15)),
            Some((1, 28)),
            Some((2, 12)),
            Some((3, 14)),
            Some((4, 14)),
            Some((4, 15)),
            Some((4, 16)),
            Some((4, 21)),
            Some((4, 28)),
            Some((5, 12)),
            Some((6, 12)),
            Some((9, 12)),
            Some((9, 14)),
            Some((8, 25)),
            Some((9, 3)),
            Some((10, 2)),
            Some((9, 29)),
            Some((9, 30)),
            Some((10, 1)),
            Some((10, 2)),
            Some((10, 18)),
            Some((10, 20)),
            Some((10, 23)),
            Some((10, 28)),
            Some((11, 5)),
            Some((4, 18)),
            Some((10, 6)),
        ],
    ),
    (
        2026,
        [
            Some((1, 14)),
            Some((1, 15)),
            Some((1, 27)),
            Some((2, 1)),
            Some((3, 3)),
            Some((4, 14)),
            Some((4, 15)),
            Some((4, 16)),
            Some((4, 21)),
            Some((4, 18)),
            Some((5, 1)),
            Some((6, 1)),
            Some((9, 1)),
            Some((9, 4)),
            Some((9, 12)),
            Some((9, 22)),
            Some((9, 21)),
            Some((10, 18)),
            Some((10, 19)),
            Some((10, 20)),
            Some((10, 21)),
            Some((10, 18)),
            Some((11, 8)),
            Some((11, 11)),
            Some((11, 15)),
            Some((11, 24)),
            Some((4, 3)),
            Some((10, 25)),
        ],
    ),
];

/// A row that parts: the year, the key and the days the rule gives.
type Part = (i32, &'static str, &'static [(u8, u8)]);

/// The rows that part from the print, each with the days the rule gives
/// (`festival-measured.md` §14).
const PARTS: [Part; 7] = [
    (2019, "SANKARDEVA_TIROBHAV", &[(8, 31)]),
    (2020, "JANMASHTAMI", &[(9, 10)]),
    (2021, "KARAM_PUJA", &[(9, 17)]),
    (2021, "KOJAGRAT_PURNIMA", &[(10, 19)]),
    (2023, "JANMASHTAMI", &[(9, 7)]),
    (2023, "SANKARDEVA_TIROBHAV", &[(9, 16)]),
    (2023, "KARAM_PUJA", &[(9, 25)]),
];

/// The days each year's notification prints rather than reckons: the two
/// Ids, and Muharram's half holiday to 2024.
const PRINTED_DAYS: [usize; 8] = [3, 3, 3, 3, 3, 3, 2, 2];

fn guwahati() -> Place {
    Place::new(
        Latitude::literal(26.1445),
        Longitude::literal(91.7362),
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
/// stand on their date, and the printed days stand marked.
fn meets(year: i32) {
    let index = PRINTED.iter().position(|row| row.0 == year).unwrap();
    let (_, printed) = PRINTED.get(index).unwrap();
    let marked = *PRINTED_DAYS.get(index).unwrap();
    let answer = modern_sky()
        .almanac()
        .festivals(
            &date(year, 1, 1),
            &date(year, 12, 31),
            &guwahati(),
            UtcOffset::literal(5, 30, 0),
            &FestivalRequest::from(FestivalPack::IndiaAssam),
        )
        .unwrap()
        .value;
    let mut parted: Vec<(&str, Vec<(u8, u8)>)> = Vec::new();
    for (key, day) in KEYS.iter().zip(*printed) {
        let Some(day) = day else { continue };
        let found = days_of(&answer, key);
        if found != [day] {
            parted.push((key, found));
        }
    }
    let named: Vec<(&str, Vec<(u8, u8)>)> = PARTS
        .iter()
        .filter(|part| part.0 == year)
        .map(|&(_, key, days)| (key, days.to_vec()))
        .collect();
    assert_eq!(parted, named, "{year}");
    for (key, day) in [
        ("NETAJI_JAYANTI", (1, 23)),
        ("ME_DAM_ME_PHI", (1, 31)),
        ("BANK_ACCOUNTS_CLOSING", (4, 1)),
        ("LACHIT_DIVAS", (11, 24)),
        ("ASOM_DIVAS", (12, 2)),
    ] {
        assert_eq!(days_of(&answer, key), [day], "{year} {key}");
    }
    assert_eq!(answer.added.len(), marked, "{year}");
    assert!(
        answer
            .added
            .iter()
            .all(|added| added.published.subject_to == Some(SubjectTo::MoonSighting)),
        "{year}"
    );
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
