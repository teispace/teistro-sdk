//! `INDIA_ANDHRA_PRADESH` against each year's G.O. of the General
//! Administration (Poll.B) Department, 2020 to 2026, at Amaravati under the
//! modern sky (`03-design/andhra-pradesh-holidays.md`): every reckoned day
//! of each year meets the print but the named partings, the fixed days
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

/// The keys of each year's reckoned rows, general and optional, in this
/// order.
const KEYS: [&str; 20] = [
    "BHOGI",
    "UTTARAYANA_PUNYAKALA",
    "KANUMU",
    "SHIVARATRI",
    "HOLI",
    "SAMVATSARA_ARAMBHA",
    "RAMA_NAVAMI",
    "GOOD_FRIDAY",
    "JANMASHTAMI",
    "GANESHA_CHATURTHI",
    "MAHALAYA_AMAVASYA",
    "MAHA_ASHTAMI",
    "VIJAYA_DASHAMI",
    "NARAK_CHATURDASHI",
    "LAKSHMI_PUJA",
    "GURU_NANAK_JAYANTI",
    "JAGANNATH_RATH_JATRA",
    "MAHAVIR_JAYANTI",
    "BASAVA_JAYANTI",
    "BUDDHA_JAYANTI",
];

/// Each G.O.'s days as (month, day), in the order of [`KEYS`]; `None`
/// where the year's G.O. has no such row.
type Year = (i32, [Option<(u8, u8)>; 20]);

const PRINTED: [Year; 7] = [
    (
        2020,
        [
            Some((1, 14)),
            Some((1, 15)),
            Some((1, 16)),
            Some((2, 21)),
            Some((3, 10)),
            Some((3, 25)),
            Some((4, 2)),
            Some((4, 10)),
            Some((8, 11)),
            Some((8, 22)),
            Some((9, 17)),
            Some((10, 24)),
            Some((10, 25)),
            None,
            Some((11, 14)),
            Some((11, 30)),
            Some((6, 23)),
            Some((4, 6)),
            Some((4, 26)),
            Some((5, 7)),
        ],
    ),
    (
        2021,
        [
            Some((1, 13)),
            Some((1, 14)),
            Some((1, 15)),
            Some((3, 11)),
            Some((3, 29)),
            Some((4, 13)),
            Some((4, 21)),
            Some((4, 2)),
            Some((8, 30)),
            Some((9, 10)),
            Some((10, 6)),
            Some((10, 13)),
            Some((10, 15)),
            Some((11, 3)),
            Some((11, 4)),
            Some((11, 19)),
            Some((7, 12)),
            Some((4, 25)),
            Some((5, 14)),
            Some((5, 26)),
        ],
    ),
    (
        2022,
        [
            Some((1, 14)),
            Some((1, 15)),
            Some((1, 16)),
            Some((3, 1)),
            Some((3, 18)),
            Some((4, 2)),
            Some((4, 10)),
            Some((4, 15)),
            Some((8, 19)),
            Some((8, 31)),
            Some((9, 25)),
            Some((10, 3)),
            Some((10, 5)),
            Some((10, 23)),
            Some((10, 24)),
            Some((11, 8)),
            Some((7, 1)),
            Some((4, 14)),
            Some((5, 3)),
            Some((5, 16)),
        ],
    ),
    (
        2023,
        [
            Some((1, 14)),
            Some((1, 15)),
            Some((1, 16)),
            Some((2, 18)),
            Some((3, 8)),
            Some((3, 22)),
            Some((3, 30)),
            Some((4, 7)),
            Some((9, 6)),
            Some((9, 18)),
            Some((10, 14)),
            Some((10, 22)),
            Some((10, 23)),
            None,
            Some((11, 12)),
            Some((11, 27)),
            Some((6, 20)),
            Some((4, 4)),
            Some((4, 23)),
            Some((5, 5)),
        ],
    ),
    (
        2024,
        [
            Some((1, 14)),
            Some((1, 15)),
            Some((1, 16)),
            Some((3, 8)),
            Some((3, 25)),
            Some((4, 9)),
            Some((4, 17)),
            Some((3, 29)),
            Some((8, 26)),
            Some((9, 7)),
            Some((10, 2)),
            Some((10, 11)),
            Some((10, 12)),
            None,
            Some((10, 31)),
            Some((11, 15)),
            Some((7, 7)),
            Some((4, 21)),
            Some((5, 10)),
            Some((5, 23)),
        ],
    ),
    (
        2025,
        [
            Some((1, 13)),
            Some((1, 14)),
            Some((1, 15)),
            Some((2, 26)),
            Some((3, 14)),
            Some((3, 30)),
            Some((4, 6)),
            Some((4, 18)),
            Some((8, 16)),
            Some((8, 27)),
            Some((9, 21)),
            Some((9, 30)),
            Some((10, 2)),
            None,
            Some((10, 20)),
            Some((11, 5)),
            Some((6, 27)),
            Some((4, 10)),
            Some((4, 30)),
            Some((5, 12)),
        ],
    ),
    (
        2026,
        [
            Some((1, 14)),
            Some((1, 15)),
            Some((1, 16)),
            Some((2, 15)),
            Some((3, 3)),
            Some((3, 19)),
            Some((3, 27)),
            Some((4, 3)),
            Some((9, 4)),
            Some((9, 14)),
            Some((10, 10)),
            Some((10, 18)),
            Some((10, 20)),
            None,
            Some((11, 8)),
            Some((11, 24)),
            Some((7, 16)),
            Some((3, 31)),
            Some((4, 20)),
            Some((5, 1)),
        ],
    ),
];

/// A row that parts: the year, the key and the days the rule gives.
type Part = (i32, &'static str, &'static [(u8, u8)]);

/// The rows that part from the print, each with the days the rule gives
/// (`festival-measured.md` §18).
const PARTS: [Part; 8] = [
    (2020, "MAHA_ASHTAMI", &[(10, 23)]),
    (2021, "BASAVA_JAYANTI", &[(5, 15)]),
    (2022, "JANMASHTAMI", &[(8, 18)]),
    (2022, "NARAK_CHATURDASHI", &[(10, 24)]),
    (2023, "HOLI", &[(3, 7)]),
    (2023, "VIJAYA_DASHAMI", &[(10, 24)]),
    (2024, "MAHA_ASHTAMI", &[(10, 10)]),
    (2025, "JANMASHTAMI", &[(8, 15)]),
];

/// The days each year's G.O. prints rather than reckons, as (marked,
/// unmarked): the four general days set by the moon, with 2021's
/// Shab-e-Barath; and Annexure-II's Islamic days, 2023's Shab-e-Meraj,
/// Varalakshmi Vratham, the Parsi New Year and 2023's second Vijayadasami.
const PRINTED_DAYS: [(usize, usize); 7] = [
    (4, 13),
    (5, 12),
    (4, 13),
    (4, 14),
    (4, 13),
    (4, 13),
    (4, 13),
];

/// The days every G.O. lists on one date.
const CIVIL: [(&str, (u8, u8)); 9] = [
    ("NEW_YEAR_S_DAY_AD", (1, 1)),
    ("INDIA_REPUBLIC_DAY", (1, 26)),
    ("JAGJIVAN_RAM_JAYANTI", (4, 5)),
    ("AMBEDKAR_JAYANTI", (4, 14)),
    ("INDIA_INDEPENDENCE_DAY", (8, 15)),
    ("GANDHI_JAYANTI", (10, 2)),
    ("CHRISTMAS_EVE", (12, 24)),
    ("CHRISTMAS", (12, 25)),
    ("BOXING_DAY", (12, 26)),
];

/// Amaravati: the Secretariat at Velagapudi.
fn amaravati() -> Place {
    Place::new(
        Latitude::literal(16.5193),
        Longitude::literal(80.515),
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

/// Holds `year`'s G.O. to the pack: each reckoned row meets the print or
/// is a named parting giving the days named, the fixed days stand on their
/// date, and the printed days stand, the moon's marked.
fn meets(year: i32) {
    let index = PRINTED.iter().position(|row| row.0 == year).unwrap();
    let (_, printed) = PRINTED.get(index).unwrap();
    let (marked, unmarked) = *PRINTED_DAYS.get(index).unwrap();
    let answer = modern_sky()
        .almanac()
        .festivals(
            &date(year, 1, 1),
            &date(year, 12, 31),
            &amaravati(),
            UtcOffset::literal(5, 30, 0),
            &FestivalRequest::from(FestivalPack::IndiaAndhraPradesh),
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
fn the_go_for_2020_meets_the_pack() {
    meets(2020);
}

#[test]
fn the_go_for_2021_meets_the_pack() {
    meets(2021);
}

#[test]
fn the_go_for_2022_meets_the_pack() {
    meets(2022);
}

#[test]
fn the_go_for_2023_meets_the_pack() {
    meets(2023);
}

#[test]
fn the_go_for_2024_meets_the_pack() {
    meets(2024);
}

#[test]
fn the_go_for_2025_meets_the_pack() {
    meets(2025);
}

#[test]
fn the_go_for_2026_meets_the_pack() {
    meets(2026);
}
