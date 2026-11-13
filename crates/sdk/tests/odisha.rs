//! `INDIA_ODISHA` against each year's holiday notification of the Revenue
//! and Disaster Management Department, 2019 to 2026, at Bhubaneswar under
//! the modern sky (`03-design/odisha-holidays.md`): the first day the pack
//! reckons for each row meets the print but the named partings, the fixed
//! days stand on their date, and the printed days stand as printed. A test
//! for each year.

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
const KEYS: [&str; 31] = [
    "UTTARAYANA_PUNYAKALA",
    "BASANTA_PANCHAMI",
    "SHIVARATRI",
    "DOLA_YATRA",
    "HOLI",
    "RAMA_NAVAMI",
    "GOOD_FRIDAY",
    "EASTER_SATURDAY",
    "MESHA_SANKRANTI",
    "BUDDHA_JAYANTI",
    "SAVITRI_AMAVASYA",
    "PAHILI_RAJA",
    "MITHUNA_SANKRANTI",
    "JAGANNATH_RATH_JATRA",
    "BAHUDA_JATRA",
    "JHULANA_PURNIMA",
    "JANMASHTAMI",
    "GANESHA_CHATURTHI",
    "NUAKHAI",
    "MAHALAYA_AMAVASYA",
    "PHULPATI",
    "MAHA_ASHTAMI",
    "MAHA_NAVAMI",
    "VIJAYA_DASHAMI",
    "KUMAR_PURNIMA",
    "KALI_PUJA",
    "ANLA_NAVAMI",
    "RAHAS_PURNIMA",
    "PRATHAMASHTAMI",
    "GURU_GOBIND_SINGH_JAYANTI",
    "BRAHMO_SAMAJ_FOUNDATION_DAY",
];

/// Each notification's days as (month, day), in the order of [`KEYS`];
/// `None` where the year's notification has no such row.
type Year = (i32, [Option<(u8, u8)>; 31]);

const PRINTED: [Year; 8] = [
    (
        2019,
        [
            None,
            Some((2, 10)),
            Some((3, 4)),
            Some((3, 21)),
            Some((3, 22)),
            Some((4, 13)),
            Some((4, 19)),
            Some((4, 20)),
            Some((4, 14)),
            Some((5, 18)),
            Some((6, 3)),
            Some((6, 14)),
            Some((6, 15)),
            Some((7, 4)),
            Some((7, 12)),
            Some((8, 15)),
            Some((8, 23)),
            Some((9, 2)),
            Some((9, 3)),
            Some((9, 28)),
            Some((10, 5)),
            Some((10, 6)),
            Some((10, 7)),
            Some((10, 8)),
            Some((10, 13)),
            Some((10, 27)),
            Some((11, 5)),
            Some((11, 12)),
            Some((11, 20)),
            Some((1, 13)),
            Some((1, 25)),
        ],
    ),
    (
        2020,
        [
            None,
            Some((1, 30)),
            Some((2, 21)),
            Some((3, 9)),
            Some((3, 10)),
            Some((4, 2)),
            Some((4, 10)),
            Some((4, 11)),
            Some((4, 13)),
            Some((5, 7)),
            Some((5, 22)),
            Some((6, 14)),
            Some((6, 15)),
            Some((6, 23)),
            Some((7, 1)),
            Some((8, 3)),
            Some((8, 11)),
            Some((8, 22)),
            Some((8, 23)),
            Some((9, 17)),
            Some((10, 23)),
            Some((10, 24)),
            Some((10, 25)),
            Some((10, 26)),
            Some((10, 31)),
            Some((11, 14)),
            Some((11, 23)),
            Some((11, 30)),
            Some((12, 8)),
            Some((1, 2)),
            Some((1, 25)),
        ],
    ),
    (
        2021,
        [
            Some((1, 14)),
            Some((2, 16)),
            Some((3, 11)),
            Some((3, 28)),
            Some((3, 29)),
            Some((4, 21)),
            Some((4, 2)),
            Some((4, 3)),
            Some((4, 14)),
            Some((5, 26)),
            Some((6, 10)),
            Some((6, 14)),
            Some((6, 15)),
            Some((7, 12)),
            Some((7, 20)),
            Some((8, 22)),
            Some((8, 30)),
            Some((9, 10)),
            Some((9, 11)),
            Some((10, 6)),
            Some((10, 12)),
            Some((10, 13)),
            Some((10, 14)),
            Some((10, 15)),
            Some((10, 20)),
            Some((11, 4)),
            Some((11, 12)),
            Some((11, 19)),
            Some((11, 27)),
            Some((1, 20)),
            Some((1, 25)),
        ],
    ),
    (
        2022,
        [
            Some((1, 14)),
            Some((2, 5)),
            Some((3, 1)),
            Some((3, 18)),
            Some((3, 19)),
            Some((4, 10)),
            Some((4, 15)),
            Some((4, 16)),
            Some((4, 14)),
            Some((5, 16)),
            Some((5, 30)),
            Some((6, 14)),
            Some((6, 15)),
            Some((7, 1)),
            Some((7, 9)),
            Some((8, 11)),
            Some((8, 18)),
            Some((8, 31)),
            Some((9, 1)),
            Some((9, 25)),
            Some((10, 2)),
            Some((10, 3)),
            Some((10, 4)),
            Some((10, 5)),
            Some((10, 9)),
            Some((10, 24)),
            Some((11, 2)),
            Some((11, 8)),
            Some((11, 16)),
            Some((1, 9)),
            Some((1, 25)),
        ],
    ),
    (
        2023,
        [
            Some((1, 14)),
            Some((1, 26)),
            Some((2, 18)),
            Some((3, 7)),
            Some((3, 8)),
            Some((3, 30)),
            Some((4, 7)),
            Some((4, 8)),
            Some((4, 14)),
            Some((5, 5)),
            Some((5, 19)),
            Some((6, 14)),
            Some((6, 15)),
            Some((6, 20)),
            Some((6, 28)),
            Some((8, 30)),
            Some((9, 6)),
            Some((9, 19)),
            Some((9, 20)),
            Some((10, 14)),
            Some((10, 21)),
            Some((10, 22)),
            Some((10, 23)),
            Some((10, 24)),
            Some((10, 28)),
            Some((11, 12)),
            Some((11, 21)),
            Some((11, 27)),
            Some((12, 5)),
            Some((1, 5)),
            Some((1, 25)),
        ],
    ),
    (
        2024,
        [
            Some((1, 15)),
            Some((2, 14)),
            Some((3, 8)),
            Some((3, 25)),
            Some((3, 26)),
            Some((4, 17)),
            Some((3, 29)),
            Some((3, 30)),
            Some((4, 14)),
            Some((5, 23)),
            Some((6, 6)),
            Some((6, 14)),
            Some((6, 15)),
            Some((7, 7)),
            Some((7, 15)),
            Some((8, 19)),
            Some((8, 26)),
            Some((9, 7)),
            Some((9, 8)),
            Some((10, 2)),
            Some((10, 10)),
            Some((10, 11)),
            Some((10, 12)),
            Some((10, 13)),
            Some((10, 16)),
            Some((10, 31)),
            Some((11, 10)),
            Some((11, 15)),
            Some((11, 23)),
            Some((1, 17)),
            Some((1, 25)),
        ],
    ),
    (
        2025,
        [
            Some((1, 14)),
            Some((2, 2)),
            Some((2, 26)),
            Some((3, 14)),
            Some((3, 15)),
            Some((4, 6)),
            Some((4, 18)),
            Some((4, 19)),
            Some((4, 14)),
            Some((5, 12)),
            Some((5, 27)),
            Some((6, 14)),
            Some((6, 15)),
            Some((6, 27)),
            Some((7, 5)),
            Some((8, 9)),
            Some((8, 15)),
            Some((8, 27)),
            Some((8, 28)),
            Some((9, 21)),
            Some((9, 29)),
            Some((9, 30)),
            Some((10, 1)),
            Some((10, 2)),
            Some((10, 7)),
            Some((10, 21)),
            Some((10, 31)),
            Some((11, 5)),
            Some((11, 12)),
            Some((1, 6)),
            Some((1, 25)),
        ],
    ),
    (
        2026,
        [
            Some((1, 14)),
            Some((1, 23)),
            Some((2, 15)),
            Some((3, 3)),
            Some((3, 4)),
            Some((3, 27)),
            Some((4, 3)),
            Some((4, 4)),
            Some((4, 14)),
            Some((5, 1)),
            Some((5, 16)),
            Some((6, 14)),
            Some((6, 15)),
            Some((7, 16)),
            Some((7, 24)),
            Some((8, 27)),
            Some((9, 4)),
            Some((9, 14)),
            Some((9, 15)),
            Some((10, 10)),
            Some((10, 17)),
            Some((10, 18)),
            Some((10, 19)),
            Some((10, 20)),
            Some((10, 25)),
            Some((11, 8)),
            Some((11, 18)),
            Some((11, 24)),
            Some((12, 1)),
            Some((1, 6)),
            Some((8, 20)),
        ],
    ),
];

/// A row that parts: the year, the key and the days the rule gives.
type Part = (i32, &'static str, &'static [(u8, u8)]);

/// The rows whose first reckoned day parts from the print, each with the
/// days the rule gives (`festival-measured.md` §21).
const PARTS: [Part; 22] = [
    (2019, "BRAHMO_SAMAJ_FOUNDATION_DAY", &[(8, 20)]),
    (2020, "PAHILI_RAJA", &[(6, 13)]),
    (2020, "MITHUNA_SANKRANTI", &[(6, 14)]),
    (2020, "BRAHMO_SAMAJ_FOUNDATION_DAY", &[(8, 20)]),
    (2021, "BRAHMO_SAMAJ_FOUNDATION_DAY", &[(8, 20)]),
    (2022, "BRAHMO_SAMAJ_FOUNDATION_DAY", &[(8, 20)]),
    (2023, "GURU_GOBIND_SINGH_JAYANTI", &[]),
    (2023, "BRAHMO_SAMAJ_FOUNDATION_DAY", &[(8, 20)]),
    (2024, "MESHA_SANKRANTI", &[(4, 13)]),
    (2024, "BRAHMO_SAMAJ_FOUNDATION_DAY", &[(8, 20)]),
    (2025, "BASANTA_PANCHAMI", &[(2, 3)]),
    (2025, "KUMAR_PURNIMA", &[(10, 6)]),
    (2025, "KALI_PUJA", &[(10, 20)]),
    (2025, "BRAHMO_SAMAJ_FOUNDATION_DAY", &[(8, 20)]),
    (2026, "RAMA_NAVAMI", &[(3, 26)]),
    (2026, "JHULANA_PURNIMA", &[(8, 28)]),
    (2026, "PHULPATI", &[(10, 18)]),
    (2026, "MAHA_ASHTAMI", &[(10, 19)]),
    (2026, "MAHA_NAVAMI", &[(10, 20)]),
    (2026, "VIJAYA_DASHAMI", &[(10, 21)]),
    (2026, "ANLA_NAVAMI", &[(11, 19)]),
    (2026, "GURU_GOBIND_SINGH_JAYANTI", &[]),
];

/// The days each year's notification prints rather than reckons, as
/// (marked, unmarked): no notification marks a day as the moon's, so the
/// four Islamic days stand unmarked, with Shab-e-Qader where listed,
/// Panchayati Raj Divas to 2025, Mahaveer Jayanti to 2021 and 2026's day
/// after Nuakhai.
const PRINTED_DAYS: [(usize, usize); 8] = [
    (0, 7),
    (0, 7),
    (0, 7),
    (0, 6),
    (0, 5),
    (0, 6),
    (0, 6),
    (0, 6),
];

/// The days every notification lists on one date.
const CIVIL: [(&str, (u8, u8)); 9] = [
    ("NETAJI_JAYANTI", (1, 23)),
    ("INDIA_REPUBLIC_DAY", (1, 26)),
    ("UTKAL_DIVAS", (4, 1)),
    ("AMBEDKAR_JAYANTI", (4, 14)),
    ("RAM_MOHAN_ROY_JAYANTI", (5, 22)),
    ("INDIA_INDEPENDENCE_DAY", (8, 15)),
    ("GANDHI_JAYANTI", (10, 2)),
    ("CHRISTMAS_EVE", (12, 24)),
    ("CHRISTMAS", (12, 25)),
];

/// Bhubaneswar: the Lok Seva Bhavan.
fn bhubaneswar() -> Place {
    Place::new(
        Latitude::literal(20.2961),
        Longitude::literal(85.8245),
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

/// Holds `year`'s notification to the pack: each reckoned row's first day
/// meets the print or is a named parting giving the days named (a Pausha
/// that opens and closes one civil year gives Guru Gobind Singh two days),
/// the fixed days stand on their date, and the printed days stand.
fn meets(year: i32) {
    let index = PRINTED.iter().position(|row| row.0 == year).unwrap();
    let (_, printed) = PRINTED.get(index).unwrap();
    let (marked, unmarked) = *PRINTED_DAYS.get(index).unwrap();
    let answer = modern_sky()
        .almanac()
        .festivals(
            &date(year, 1, 1),
            &date(year, 12, 31),
            &bhubaneswar(),
            UtcOffset::literal(5, 30, 0),
            &FestivalRequest::from(FestivalPack::IndiaOdisha),
        )
        .unwrap()
        .value;
    let mut parted: Vec<(&str, Vec<(u8, u8)>)> = Vec::new();
    for (key, day) in KEYS.iter().zip(*printed) {
        let Some(day) = day else { continue };
        let found = days_of(&answer, key);
        if found.first() != Some(&day) {
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
