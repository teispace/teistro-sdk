//! `INDIA_UTTAR_PRADESH` against each year's notification of the General
//! Administration Section, 2019 to 2026, at Lucknow under the modern sky
//! (`03-design/uttar-pradesh-holidays.md`): every reckoned day of each
//! year's public and restricted lists meets the print but the named
//! parting, the fixed days stand on their date, and the twelve days the
//! moon sets stand as printed, each marked. A test for each year.

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

/// Each year's reckoned days, of both lists, as the notification prints
/// them, less the days every notification lists on one date.
const LISTED: [(i32, &str, (u8, u8)); 232] = [
    (2019, "SHIVARATRI", (3, 4)),
    (2019, "HOLIKA", (3, 20)),
    (2019, "HOLI", (3, 21)),
    (2019, "RAMA_NAVAMI", (4, 13)),
    (2019, "MAHAVIR_JAYANTI", (4, 17)),
    (2019, "GOOD_FRIDAY", (4, 19)),
    (2019, "BUDDHA_JAYANTI", (5, 18)),
    (2019, "RAKSHABANDHAN", (8, 15)),
    (2019, "JANMASHTAMI", (8, 23)),
    (2019, "MAHA_NAVAMI", (10, 7)),
    (2019, "VIJAYA_DASHAMI", (10, 8)),
    (2019, "LAKSHMI_PUJA", (10, 27)),
    (2019, "GOVARDHAN_PUJA", (10, 28)),
    (2019, "YAMA_DWITIYA", (10, 29)),
    (2019, "GURU_NANAK_JAYANTI", (11, 12)),
    (2019, "UTTARAYANA_PUNYAKALA", (1, 15)),
    (2019, "BASANTA_PANCHAMI", (2, 10)),
    (2019, "RAVIDAS_JAYANTI", (2, 19)),
    (2019, "HOLI_SECOND_DAY", (3, 22)),
    (2019, "CHETI_CHAND", (4, 6)),
    (2019, "EASTER_SATURDAY", (4, 20)),
    (2019, "EASTER_MONDAY", (4, 22)),
    (2019, "PARASHURAMA_JAYANTI", (5, 7)),
    (2019, "ANANTA_CHATURDASHI", (9, 12)),
    (2019, "AGRASEN_JAYANTI", (9, 29)),
    (2019, "MAHA_ASHTAMI", (10, 6)),
    (2019, "VALMIKI_JAYANTI", (10, 13)),
    (2019, "NARAK_CHATURDASHI", (10, 26)),
    (2019, "CHHATH", (11, 2)),
    (2020, "SHIVARATRI", (2, 21)),
    (2020, "HOLIKA", (3, 9)),
    (2020, "HOLI", (3, 10)),
    (2020, "RAMA_NAVAMI", (4, 2)),
    (2020, "MAHAVIR_JAYANTI", (4, 6)),
    (2020, "GOOD_FRIDAY", (4, 10)),
    (2020, "BUDDHA_JAYANTI", (5, 7)),
    (2020, "RAKSHABANDHAN", (8, 3)),
    (2020, "JANMASHTAMI", (8, 12)),
    (2020, "MAHA_NAVAMI", (10, 24)),
    (2020, "VIJAYA_DASHAMI", (10, 25)),
    (2020, "LAKSHMI_PUJA", (11, 14)),
    (2020, "GOVARDHAN_PUJA", (11, 15)),
    (2020, "YAMA_DWITIYA", (11, 16)),
    (2020, "GURU_NANAK_JAYANTI", (11, 30)),
    (2020, "UTTARAYANA_PUNYAKALA", (1, 15)),
    (2020, "BASANTA_PANCHAMI", (1, 30)),
    (2020, "RAVIDAS_JAYANTI", (2, 9)),
    (2020, "HOLI_SECOND_DAY", (3, 11)),
    (2020, "CHETI_CHAND", (3, 25)),
    (2020, "EASTER_SATURDAY", (4, 11)),
    (2020, "EASTER_MONDAY", (4, 13)),
    (2020, "PARASHURAMA_JAYANTI", (4, 25)),
    (2020, "ANANTA_CHATURDASHI", (8, 31)),
    (2020, "AGRASEN_JAYANTI", (10, 17)),
    (2020, "MAHA_ASHTAMI", (10, 24)),
    (2020, "VALMIKI_JAYANTI", (10, 31)),
    (2020, "NARAK_CHATURDASHI", (11, 13)),
    (2020, "CHHATH", (11, 20)),
    (2021, "SHIVARATRI", (3, 11)),
    (2021, "HOLIKA", (3, 28)),
    (2021, "HOLI", (3, 29)),
    (2021, "RAMA_NAVAMI", (4, 21)),
    (2021, "MAHAVIR_JAYANTI", (4, 25)),
    (2021, "GOOD_FRIDAY", (4, 2)),
    (2021, "BUDDHA_JAYANTI", (5, 26)),
    (2021, "RAKSHABANDHAN", (8, 22)),
    (2021, "JANMASHTAMI", (8, 30)),
    (2021, "MAHA_NAVAMI", (10, 14)),
    (2021, "VIJAYA_DASHAMI", (10, 15)),
    (2021, "LAKSHMI_PUJA", (11, 4)),
    (2021, "GOVARDHAN_PUJA", (11, 5)),
    (2021, "YAMA_DWITIYA", (11, 6)),
    (2021, "GURU_NANAK_JAYANTI", (11, 19)),
    (2021, "UTTARAYANA_PUNYAKALA", (1, 14)),
    (2021, "BASANTA_PANCHAMI", (2, 16)),
    (2021, "RAVIDAS_JAYANTI", (2, 27)),
    (2021, "HOLI_SECOND_DAY", (3, 30)),
    (2021, "CHETI_CHAND", (4, 13)),
    (2021, "EASTER_SATURDAY", (4, 3)),
    (2021, "EASTER_MONDAY", (4, 5)),
    (2021, "PARASHURAMA_JAYANTI", (5, 14)),
    (2021, "ANANTA_CHATURDASHI", (9, 19)),
    (2021, "AGRASEN_JAYANTI", (10, 7)),
    (2021, "MAHA_ASHTAMI", (10, 13)),
    (2021, "VALMIKI_JAYANTI", (10, 20)),
    (2021, "NARAK_CHATURDASHI", (11, 3)),
    (2021, "CHHATH", (11, 10)),
    (2022, "SHIVARATRI", (3, 1)),
    (2022, "HOLIKA", (3, 17)),
    (2022, "HOLI", (3, 18)),
    (2022, "RAMA_NAVAMI", (4, 10)),
    (2022, "MAHAVIR_JAYANTI", (4, 14)),
    (2022, "GOOD_FRIDAY", (4, 15)),
    (2022, "BUDDHA_JAYANTI", (5, 16)),
    (2022, "RAKSHABANDHAN", (8, 12)),
    (2022, "JANMASHTAMI", (8, 18)),
    (2022, "MAHA_NAVAMI", (10, 4)),
    (2022, "VIJAYA_DASHAMI", (10, 5)),
    (2022, "LAKSHMI_PUJA", (10, 24)),
    (2022, "GOVARDHAN_PUJA", (10, 26)),
    (2022, "YAMA_DWITIYA", (10, 27)),
    (2022, "GURU_NANAK_JAYANTI", (11, 8)),
    (2022, "UTTARAYANA_PUNYAKALA", (1, 14)),
    (2022, "BASANTA_PANCHAMI", (2, 5)),
    (2022, "RAVIDAS_JAYANTI", (2, 16)),
    (2022, "HOLI_SECOND_DAY", (3, 19)),
    (2022, "CHETI_CHAND", (4, 2)),
    (2022, "EASTER_SATURDAY", (4, 16)),
    (2022, "EASTER_MONDAY", (4, 18)),
    (2022, "PARASHURAMA_JAYANTI", (5, 3)),
    (2022, "ANANTA_CHATURDASHI", (9, 9)),
    (2022, "AGRASEN_JAYANTI", (9, 26)),
    (2022, "MAHA_ASHTAMI", (10, 3)),
    (2022, "VALMIKI_JAYANTI", (10, 9)),
    (2022, "NARAK_CHATURDASHI", (10, 23)),
    (2022, "CHHATH", (10, 30)),
    (2023, "SHIVARATRI", (2, 18)),
    (2023, "HOLIKA", (3, 7)),
    (2023, "HOLI", (3, 8)),
    (2023, "RAMA_NAVAMI", (3, 30)),
    (2023, "MAHAVIR_JAYANTI", (4, 4)),
    (2023, "GOOD_FRIDAY", (4, 7)),
    (2023, "BUDDHA_JAYANTI", (5, 5)),
    (2023, "RAKSHABANDHAN", (8, 31)),
    (2023, "JANMASHTAMI", (9, 7)),
    (2023, "MAHA_NAVAMI", (10, 23)),
    (2023, "VIJAYA_DASHAMI", (10, 24)),
    (2023, "LAKSHMI_PUJA", (11, 12)),
    (2023, "GOVARDHAN_PUJA", (11, 13)),
    (2023, "YAMA_DWITIYA", (11, 15)),
    (2023, "GURU_NANAK_JAYANTI", (11, 27)),
    (2023, "UTTARAYANA_PUNYAKALA", (1, 15)),
    (2023, "BASANTA_PANCHAMI", (1, 26)),
    (2023, "RAVIDAS_JAYANTI", (2, 5)),
    (2023, "HOLI_SECOND_DAY", (3, 9)),
    (2023, "CHETI_CHAND", (3, 22)),
    (2023, "EASTER_SATURDAY", (4, 8)),
    (2023, "EASTER_MONDAY", (4, 10)),
    (2023, "PARASHURAMA_JAYANTI", (4, 22)),
    (2023, "ANANTA_CHATURDASHI", (9, 28)),
    (2023, "AGRASEN_JAYANTI", (10, 15)),
    (2023, "MAHA_ASHTAMI", (10, 22)),
    (2023, "VALMIKI_JAYANTI", (10, 28)),
    (2023, "NARAK_CHATURDASHI", (11, 11)),
    (2023, "CHHATH", (11, 19)),
    (2024, "SHIVARATRI", (3, 8)),
    (2024, "HOLIKA", (3, 24)),
    (2024, "HOLI", (3, 25)),
    (2024, "RAMA_NAVAMI", (4, 17)),
    (2024, "MAHAVIR_JAYANTI", (4, 21)),
    (2024, "GOOD_FRIDAY", (3, 29)),
    (2024, "BUDDHA_JAYANTI", (5, 23)),
    (2024, "RAKSHABANDHAN", (8, 19)),
    (2024, "JANMASHTAMI", (8, 26)),
    (2024, "MAHA_NAVAMI", (10, 12)),
    (2024, "VIJAYA_DASHAMI", (10, 12)),
    (2024, "LAKSHMI_PUJA", (10, 31)),
    (2024, "GOVARDHAN_PUJA", (11, 2)),
    (2024, "YAMA_DWITIYA", (11, 3)),
    (2024, "GURU_NANAK_JAYANTI", (11, 15)),
    (2024, "UTTARAYANA_PUNYAKALA", (1, 15)),
    (2024, "BASANTA_PANCHAMI", (2, 14)),
    (2024, "RAVIDAS_JAYANTI", (2, 24)),
    (2024, "HOLI_SECOND_DAY", (3, 26)),
    (2024, "CHETI_CHAND", (4, 9)),
    (2024, "EASTER_SATURDAY", (3, 30)),
    (2024, "EASTER_MONDAY", (4, 1)),
    (2024, "PARASHURAMA_JAYANTI", (5, 10)),
    (2024, "ANANTA_CHATURDASHI", (9, 17)),
    (2024, "AGRASEN_JAYANTI", (10, 3)),
    (2024, "MAHA_ASHTAMI", (10, 11)),
    (2024, "VALMIKI_JAYANTI", (10, 17)),
    (2024, "NARAK_CHATURDASHI", (10, 30)),
    (2024, "CHHATH", (11, 7)),
    (2025, "SHIVARATRI", (2, 26)),
    (2025, "HOLIKA", (3, 13)),
    (2025, "HOLI", (3, 14)),
    (2025, "RAMA_NAVAMI", (4, 6)),
    (2025, "MAHAVIR_JAYANTI", (4, 10)),
    (2025, "GOOD_FRIDAY", (4, 18)),
    (2025, "BUDDHA_JAYANTI", (5, 12)),
    (2025, "RAKSHABANDHAN", (8, 9)),
    (2025, "JANMASHTAMI", (8, 16)),
    (2025, "MAHA_NAVAMI", (10, 1)),
    (2025, "VIJAYA_DASHAMI", (10, 2)),
    (2025, "LAKSHMI_PUJA", (10, 20)),
    (2025, "GOVARDHAN_PUJA", (10, 22)),
    (2025, "YAMA_DWITIYA", (10, 23)),
    (2025, "GURU_NANAK_JAYANTI", (11, 5)),
    (2025, "UTTARAYANA_PUNYAKALA", (1, 14)),
    (2025, "BASANTA_PANCHAMI", (2, 2)),
    (2025, "RAVIDAS_JAYANTI", (2, 12)),
    (2025, "HOLI_SECOND_DAY", (3, 15)),
    (2025, "CHETI_CHAND", (3, 30)),
    (2025, "EASTER_SATURDAY", (4, 19)),
    (2025, "EASTER_MONDAY", (4, 21)),
    (2025, "PARASHURAMA_JAYANTI", (4, 29)),
    (2025, "ANANTA_CHATURDASHI", (9, 6)),
    (2025, "AGRASEN_JAYANTI", (9, 22)),
    (2025, "MAHA_ASHTAMI", (9, 30)),
    (2025, "VALMIKI_JAYANTI", (10, 7)),
    (2025, "NARAK_CHATURDASHI", (10, 20)),
    (2025, "CHHATH", (10, 28)),
    (2026, "SHIVARATRI", (2, 15)),
    (2026, "HOLIKA", (3, 2)),
    (2026, "HOLI", (3, 4)),
    (2026, "RAMA_NAVAMI", (3, 26)),
    (2026, "MAHAVIR_JAYANTI", (3, 31)),
    (2026, "GOOD_FRIDAY", (4, 3)),
    (2026, "BUDDHA_JAYANTI", (5, 1)),
    (2026, "RAKSHABANDHAN", (8, 28)),
    (2026, "JANMASHTAMI", (9, 4)),
    (2026, "MAHA_NAVAMI", (10, 20)),
    (2026, "VIJAYA_DASHAMI", (10, 20)),
    (2026, "LAKSHMI_PUJA", (11, 8)),
    (2026, "GOVARDHAN_PUJA", (11, 9)),
    (2026, "YAMA_DWITIYA", (11, 11)),
    (2026, "GURU_NANAK_JAYANTI", (11, 24)),
    (2026, "UTTARAYANA_PUNYAKALA", (1, 14)),
    (2026, "BASANTA_PANCHAMI", (1, 23)),
    (2026, "RAVIDAS_JAYANTI", (2, 1)),
    (2026, "HOLI_SECOND_DAY", (3, 5)),
    (2026, "CHETI_CHAND", (3, 19)),
    (2026, "EASTER_SATURDAY", (4, 4)),
    (2026, "EASTER_MONDAY", (4, 6)),
    (2026, "PARASHURAMA_JAYANTI", (4, 19)),
    (2026, "ANANTA_CHATURDASHI", (9, 25)),
    (2026, "AGRASEN_JAYANTI", (10, 11)),
    (2026, "MAHA_ASHTAMI", (10, 19)),
    (2026, "VALMIKI_JAYANTI", (10, 26)),
    (2026, "NARAK_CHATURDASHI", (11, 8)),
    (2026, "CHHATH", (11, 15)),
];

/// The days every notification lists on one date, in either list.
const CIVIL: [(&str, (u8, u8)); 15] = [
    ("NEW_YEAR_S_DAY_AD", (1, 1)),
    ("KARPOORI_THAKUR_JAYANTI", (1, 24)),
    ("INDIA_REPUBLIC_DAY", (1, 26)),
    ("KASHYAP_NISHADRAJ_JAYANTI", (4, 5)),
    ("AMBEDKAR_JAYANTI", (4, 14)),
    ("CHANDRASHEKHAR_JAYANTI", (4, 17)),
    ("MAHARANA_PRATAP_JAYANTI", (5, 9)),
    ("INDIA_INDEPENDENCE_DAY", (8, 15)),
    ("VISHWAKARMA_PUJA", (9, 17)),
    ("GANDHI_JAYANTI", (10, 2)),
    ("SARDAR_PATEL_JAYANTI", (10, 31)),
    ("UDA_DEVI_SHAHEED_DIWAS", (11, 16)),
    ("CHARAN_SINGH_JAYANTI", (12, 23)),
    ("CHRISTMAS_EVE", (12, 24)),
    ("CHRISTMAS", (12, 25)),
];

/// A row that parts: the year, the key and the days the rule gives.
type Part = (i32, &'static str, &'static [(u8, u8)]);

/// The rows that part from the print, with the days the rule gives
/// (`festival-measured.md` §20, C1525, C1528, C1529).
const PARTS: [Part; 13] = [
    (2019, "JANMASHTAMI", &[(8, 24)]),
    (2019, "VIJAYA_DASHAMI", &[(10, 7)]),
    (2020, "ANANTA_CHATURDASHI", &[(9, 1)]),
    (2020, "MAHA_NAVAMI", &[(10, 25)]),
    (2022, "JANMASHTAMI", &[(8, 19)]),
    (2023, "GOVARDHAN_PUJA", &[(11, 14)]),
    (2025, "BASANTA_PANCHAMI", &[(2, 3)]),
    (2025, "NARAK_CHATURDASHI", &[(10, 19)]),
    (2025, "CHHATH", &[(10, 27)]),
    (2026, "HOLI", &[(3, 3)]),
    (2026, "HOLI_SECOND_DAY", &[(3, 4)]),
    (2026, "GOVARDHAN_PUJA", &[(11, 10)]),
    (2026, "NARAK_CHATURDASHI", &[(11, 7)]),
];

/// The days each notification prints rather than reckons, all set by the
/// moon: five public and seven restricted.
const PRINTED_DAYS: usize = 12;

fn lucknow() -> Place {
    Place::new(
        Latitude::literal(26.8467),
        Longitude::literal(80.9462),
        Altitude::literal(0.0),
    )
}

fn date(year: i32, month: u8, day: u8) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, year, month, day)
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

/// Holds `year`'s notification to the pack: each listed row meets the
/// print or is a named parting giving the days named, the fixed days
/// stand on their date, and the printed days stand, each marked.
fn meets(year: i32) {
    let answer = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
        .almanac()
        .festivals(
            &date(year, 1, 1),
            &date(year, 12, 31),
            &lucknow(),
            UtcOffset::literal(5, 30, 0),
            &FestivalRequest::from(FestivalPack::IndiaUttarPradesh),
        )
        .unwrap()
        .value;
    let mut parted: Vec<(&str, Vec<(u8, u8)>)> = LISTED
        .iter()
        .filter(|row| row.0 == year)
        .filter_map(|&(_, key, day)| {
            let found = days_of(&answer, key);
            (found != [day]).then_some((key, found))
        })
        .collect();
    let mut named: Vec<(&str, Vec<(u8, u8)>)> = PARTS
        .iter()
        .filter(|part| part.0 == year)
        .map(|&(_, key, days)| (key, days.to_vec()))
        .collect();
    parted.sort();
    named.sort();
    assert_eq!(parted, named, "{year}");
    for (key, day) in CIVIL {
        assert_eq!(days_of(&answer, key), [day], "{year} {key}");
    }
    assert_eq!(answer.added.len(), PRINTED_DAYS, "{year}");
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
