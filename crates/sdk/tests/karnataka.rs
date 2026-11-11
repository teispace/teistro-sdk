//! `INDIA_KARNATAKA` against each year's notification of the Department
//! of Personnel and Administrative Reforms, 2019 to 2026, at Bengaluru
//! under the modern sky (`03-design/karnataka-holidays.md`): every reckoned
//! day of each year meets the print but the named partings, the fixed days
//! stand on their date, and the printed days stand as printed. A test for
//! each year.

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

/// The keys of each year's rows, the general holidays first, then the
/// restricted ones and the banks' closing of accounts, in this order.
const KEYS: [&str; 37] = [
    "UTTARAYANA_PUNYAKALA",
    "SHIVARATRI",
    "SAMVATSARA_ARAMBHA",
    "MAHAVIR_JAYANTI",
    "GOOD_FRIDAY",
    "BASAVA_JAYANTI",
    "GANESHA_CHATURTHI",
    "AMAVASYA_SHRADDHA",
    "MAHA_NAVAMI",
    "VIJAYA_DASHAMI",
    "VALMIKI_JAYANTI",
    "NARAK_CHATURDASHI",
    "BALI_PRATIPADA",
    "KANAKADASA_JAYANTI",
    "NEW_YEAR_S_DAY_AD",
    "MADHVA_NAVAMI",
    "HOLIKA",
    "DEVARA_DASIMAYYA_JAYANTI",
    "RAMA_NAVAMI",
    "EASTER_SATURDAY",
    "SOURAMANA_UGADI",
    "SHANKARACHARYA_JAYANTI",
    "RAMANUJACHARYA_JAYANTI",
    "BUDDHA_JAYANTI",
    "UPAKARMA_TAITTIRIYA",
    "JANMASHTAMI",
    "SWARNA_GOWRI_VRATA",
    "THIRUVONAM",
    "SREE_NARAYANA_GURU_JAYANTHI",
    "KANYA_MARIYAMMA_JAYANTI",
    "ANANTA_CHATURDASHI",
    "VISHWAKARMA_PUJA",
    "TULA_SANKRANTI",
    "GURU_NANAK_JAYANTI",
    "CHRISTMAS_EVE",
    "RAKSHABANDHAN",
    "BANK_ACCOUNTS_CLOSING",
];

/// Each notification's days as (month, day), in the order of [`KEYS`];
/// `None` where the year's notification has no such row: 2019's list,
/// read from a news site, has the general holidays alone.
type Year = (i32, [Option<(u8, u8)>; 37]);

const PRINTED: [Year; 8] = [
    (
        2019,
        [
            Some((1, 15)),
            Some((3, 4)),
            Some((4, 6)),
            Some((4, 17)),
            Some((4, 19)),
            Some((5, 7)),
            Some((9, 2)),
            Some((9, 28)),
            Some((10, 7)),
            Some((10, 8)),
            Some((10, 13)),
            Some((10, 27)),
            Some((10, 29)),
            Some((11, 15)),
            None,
            None,
            None,
            None,
            None,
            None,
            Some((4, 14)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
    ),
    (
        2020,
        [
            Some((1, 15)),
            Some((2, 21)),
            Some((3, 25)),
            Some((4, 6)),
            Some((4, 10)),
            Some((4, 26)),
            Some((8, 22)),
            Some((9, 17)),
            Some((10, 25)),
            Some((10, 26)),
            Some((10, 31)),
            Some((11, 14)),
            Some((11, 16)),
            Some((12, 3)),
            Some((1, 1)),
            Some((2, 3)),
            Some((3, 9)),
            Some((3, 29)),
            Some((4, 2)),
            Some((4, 11)),
            Some((4, 13)),
            Some((4, 28)),
            Some((4, 28)),
            Some((5, 7)),
            Some((9, 1)),
            Some((8, 11)),
            Some((8, 21)),
            Some((8, 31)),
            Some((9, 2)),
            Some((9, 8)),
            Some((8, 31)),
            Some((9, 17)),
            Some((10, 17)),
            Some((11, 30)),
            Some((12, 24)),
            None,
            Some((4, 1)),
        ],
    ),
    (
        2021,
        [
            Some((1, 14)),
            Some((3, 11)),
            Some((4, 13)),
            Some((4, 25)),
            Some((4, 2)),
            Some((5, 14)),
            Some((9, 10)),
            Some((10, 6)),
            Some((10, 14)),
            Some((10, 15)),
            Some((10, 20)),
            Some((11, 3)),
            Some((11, 5)),
            Some((11, 22)),
            Some((1, 1)),
            Some((2, 21)),
            Some((3, 28)),
            Some((4, 17)),
            Some((4, 21)),
            Some((4, 3)),
            Some((4, 14)),
            Some((5, 17)),
            Some((4, 18)),
            Some((5, 26)),
            Some((8, 22)),
            Some((8, 29)),
            Some((9, 9)),
            Some((8, 21)),
            Some((8, 23)),
            Some((9, 8)),
            Some((9, 19)),
            Some((9, 17)),
            Some((10, 18)),
            Some((11, 19)),
            Some((12, 24)),
            None,
            None,
        ],
    ),
    (
        2022,
        [
            Some((1, 15)),
            Some((3, 1)),
            Some((4, 2)),
            Some((4, 14)),
            Some((4, 15)),
            Some((5, 3)),
            Some((8, 31)),
            Some((9, 25)),
            Some((10, 4)),
            Some((10, 5)),
            Some((10, 9)),
            Some((10, 24)),
            Some((10, 26)),
            Some((11, 11)),
            Some((1, 1)),
            Some((2, 10)),
            Some((3, 17)),
            Some((4, 6)),
            Some((4, 10)),
            Some((4, 16)),
            Some((4, 14)),
            Some((5, 6)),
            Some((5, 5)),
            Some((5, 16)),
            Some((8, 11)),
            Some((8, 19)),
            Some((8, 30)),
            Some((9, 8)),
            Some((9, 10)),
            Some((9, 8)),
            Some((9, 9)),
            Some((9, 17)),
            Some((10, 18)),
            Some((11, 8)),
            Some((12, 24)),
            None,
            None,
        ],
    ),
    (
        2023,
        [
            Some((1, 15)),
            Some((2, 18)),
            Some((3, 22)),
            Some((4, 3)),
            Some((4, 7)),
            Some((4, 23)),
            Some((9, 18)),
            Some((10, 14)),
            Some((10, 23)),
            Some((10, 24)),
            Some((10, 28)),
            Some((11, 12)),
            Some((11, 14)),
            Some((11, 30)),
            Some((1, 1)),
            Some((1, 30)),
            Some((3, 8)),
            Some((3, 26)),
            Some((3, 30)),
            Some((4, 8)),
            Some((4, 14)),
            Some((4, 25)),
            Some((4, 25)),
            Some((5, 5)),
            Some((8, 30)),
            Some((9, 6)),
            Some((9, 18)),
            Some((8, 29)),
            Some((8, 31)),
            Some((9, 8)),
            Some((9, 28)),
            Some((9, 17)),
            Some((10, 18)),
            Some((11, 27)),
            Some((12, 24)),
            None,
            None,
        ],
    ),
    (
        2024,
        [
            Some((1, 15)),
            Some((3, 8)),
            Some((4, 9)),
            Some((4, 21)),
            Some((3, 29)),
            Some((5, 10)),
            Some((9, 7)),
            Some((10, 2)),
            Some((10, 11)),
            Some((10, 12)),
            Some((10, 17)),
            Some((10, 31)),
            Some((11, 2)),
            Some((11, 18)),
            Some((1, 1)),
            Some((2, 18)),
            Some((3, 25)),
            Some((4, 13)),
            Some((4, 17)),
            Some((3, 30)),
            Some((4, 13)),
            Some((5, 12)),
            Some((5, 12)),
            Some((5, 23)),
            Some((8, 19)),
            Some((8, 26)),
            Some((9, 6)),
            Some((9, 15)),
            Some((8, 20)),
            Some((9, 8)),
            Some((9, 16)),
            Some((9, 17)),
            Some((10, 17)),
            Some((11, 15)),
            Some((12, 24)),
            None,
            Some((4, 1)),
        ],
    ),
    (
        2025,
        [
            Some((1, 14)),
            Some((2, 26)),
            Some((3, 30)),
            Some((4, 10)),
            Some((4, 18)),
            Some((4, 30)),
            Some((8, 27)),
            Some((9, 21)),
            Some((10, 1)),
            Some((10, 1)),
            Some((10, 7)),
            Some((10, 20)),
            Some((10, 22)),
            Some((11, 8)),
            Some((1, 1)),
            Some((2, 6)),
            Some((3, 13)),
            Some((4, 2)),
            Some((4, 6)),
            Some((4, 19)),
            Some((4, 14)),
            Some((5, 2)),
            Some((5, 2)),
            Some((5, 12)),
            Some((8, 9)),
            Some((8, 16)),
            Some((8, 26)),
            Some((9, 5)),
            Some((9, 7)),
            Some((9, 8)),
            Some((9, 6)),
            Some((9, 17)),
            Some((10, 18)),
            Some((11, 5)),
            Some((12, 24)),
            None,
            None,
        ],
    ),
    (
        2026,
        [
            Some((1, 15)),
            Some((2, 15)),
            Some((3, 19)),
            Some((3, 31)),
            Some((4, 3)),
            Some((4, 20)),
            Some((9, 14)),
            Some((10, 10)),
            Some((10, 20)),
            Some((10, 21)),
            Some((10, 25)),
            Some((11, 8)),
            Some((11, 10)),
            Some((11, 27)),
            Some((1, 1)),
            Some((1, 27)),
            Some((3, 2)),
            Some((3, 23)),
            Some((3, 27)),
            Some((4, 4)),
            Some((4, 14)),
            Some((4, 21)),
            Some((4, 22)),
            Some((5, 1)),
            Some((8, 27)),
            Some((9, 4)),
            Some((9, 14)),
            Some((8, 26)),
            Some((8, 28)),
            Some((9, 8)),
            Some((9, 25)),
            Some((9, 17)),
            Some((10, 18)),
            Some((11, 24)),
            Some((12, 24)),
            Some((8, 28)),
            None,
        ],
    ),
];

/// A row that parts: the year, the key and the days the rule gives.
type Part = (i32, &'static str, &'static [(u8, u8)]);

/// The rows that part from the print, each with the days the rule gives
/// (`festival-measured.md` §15).
const PARTS: [Part; 17] = [
    (2020, "UPAKARMA_TAITTIRIYA", &[(8, 3)]),
    (2020, "ANANTA_CHATURDASHI", &[(9, 1)]),
    (2021, "JANMASHTAMI", &[(8, 30)]),
    (2021, "TULA_SANKRANTI", &[(10, 17)]),
    (2022, "RAMANUJACHARYA_JAYANTI", &[(5, 6)]),
    (2022, "JANMASHTAMI", &[(8, 18)]),
    (2023, "MAHAVIR_JAYANTI", &[(4, 4)]),
    (2023, "GANESHA_CHATURTHI", &[(9, 19)]),
    (2023, "HOLIKA", &[(3, 6)]),
    (2024, "MAHA_NAVAMI", &[(10, 12)]),
    (2024, "VIJAYA_DASHAMI", &[(10, 13)]),
    (2024, "HOLIKA", &[(3, 24)]),
    (2024, "ANANTA_CHATURDASHI", &[(9, 17)]),
    (2025, "VIJAYA_DASHAMI", &[(10, 2)]),
    (2025, "JANMASHTAMI", &[(8, 15)]),
    (2026, "VALMIKI_JAYANTI", &[(10, 26)]),
    (2026, "UPAKARMA_TAITTIRIYA", &[(8, 28)]),
];

/// The days each year's notification prints rather than reckons: the four
/// general days set by the moon, and from 2020 the three restricted ones
/// set by it, and Varamahalakshmi Vrata, Rig Upakarma and Huttari.
const PRINTED_DAYS: [(usize, usize); 8] = [
    (4, 0),
    (7, 3),
    (7, 3),
    (7, 3),
    (7, 3),
    (7, 3),
    (7, 3),
    (7, 3),
];

fn bengaluru() -> Place {
    Place::new(
        Latitude::literal(12.9716),
        Longitude::literal(77.5946),
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
    let index = PRINTED.iter().position(|row| row.0 == year).unwrap();
    let (_, printed) = PRINTED.get(index).unwrap();
    let (marked, unmarked) = *PRINTED_DAYS.get(index).unwrap();
    let answer = modern_sky()
        .almanac()
        .festivals(
            &date(year, 1, 1),
            &date(year, 12, 31),
            &bengaluru(),
            UtcOffset::literal(5, 30, 0),
            &FestivalRequest::from(FestivalPack::IndiaKarnataka),
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
        ("INDIA_REPUBLIC_DAY", (1, 26)),
        ("AMBEDKAR_JAYANTI", (4, 14)),
        ("INTERNATIONAL_LABOUR_DAY", (5, 1)),
        ("INDIA_INDEPENDENCE_DAY", (8, 15)),
        ("GANDHI_JAYANTI", (10, 2)),
        ("KANNADA_RAJYOTSAVA", (11, 1)),
        ("CHRISTMAS", (12, 25)),
    ] {
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
