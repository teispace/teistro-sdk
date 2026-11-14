//! `INDIA_BIHAR` against each year's notification of the General
//! Administration Department, 2019 to 2026, at Patna under the modern sky
//! (`03-design/bihar-holidays.md`): the days the pack reckons for each key
//! of each year's general, public and restricted lists meet the print but
//! the named partings, the fixed days stand on their date, and the printed
//! days stand as printed, the Islamic ones marked. A test for each year.

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

/// Each year's reckoned days, of every schedule, as the notification
/// prints them, less the days every notification lists on one date. A
/// Pausha that opens and closes one civil year gives 2025 two Guru Gobind
/// Singh's birthdays.
const LISTED: [(i32, &str, (u8, u8)); 236] = [
    (2019, "GURU_GOBIND_SINGH_JAYANTI", (1, 13)),
    (2019, "BASANTA_PANCHAMI", (2, 10)),
    (2019, "RAVIDAS_JAYANTI", (2, 19)),
    (2019, "SHIVARATRI", (3, 4)),
    (2019, "SAMRAT_ASHOK_ASHTAMI", (4, 12)),
    (2019, "MAHAVIR_JAYANTI", (4, 17)),
    (2019, "SITA_NAVAMI", (5, 13)),
    (2019, "BUDDHA_JAYANTI", (5, 18)),
    (2019, "KABIR_JAYANTI", (6, 17)),
    (2019, "PHULPATI", (10, 5)),
    (2019, "YAMA_DWITIYA", (10, 29)),
    (2019, "UTTARAYANA_PUNYAKALA", (1, 15)),
    (2019, "HOLIKA", (3, 20)),
    (2019, "RAKSHABANDHAN", (8, 15)),
    (2019, "ANANTA_CHATURDASHI", (9, 13)),
    (2019, "NAVARATRA_ARAMBHA", (9, 29)),
    (2019, "VIJAYA_DASHAMI_NEXT_DAY", (10, 9)),
    (2019, "CHHATH_KHARNA", (11, 1)),
    (2019, "HOLI", (3, 21)),
    (2019, "HOLI_SECOND_DAY", (3, 22)),
    (2019, "RAMA_NAVAMI", (4, 13)),
    (2019, "GOOD_FRIDAY", (4, 19)),
    (2019, "JANMASHTAMI", (8, 23)),
    (2019, "MAHA_ASHTAMI", (10, 6)),
    (2019, "MAHA_NAVAMI", (10, 7)),
    (2019, "VIJAYA_DASHAMI", (10, 8)),
    (2019, "LAKSHMI_PUJA", (10, 27)),
    (2019, "CHHATH", (11, 2)),
    (2019, "CHHATH_SECOND_DAY", (11, 3)),
    (2020, "GURU_GOBIND_SINGH_JAYANTI", (1, 2)),
    (2020, "BASANTA_PANCHAMI", (1, 30)),
    (2020, "RAVIDAS_JAYANTI", (2, 9)),
    (2020, "SHIVARATRI", (2, 21)),
    (2020, "SAMRAT_ASHOK_ASHTAMI", (4, 1)),
    (2020, "MAHAVIR_JAYANTI", (4, 6)),
    (2020, "SITA_NAVAMI", (5, 2)),
    (2020, "BUDDHA_JAYANTI", (5, 7)),
    (2020, "KABIR_JAYANTI", (6, 5)),
    (2020, "PHULPATI", (10, 23)),
    (2020, "YAMA_DWITIYA", (11, 16)),
    (2020, "UTTARAYANA_PUNYAKALA", (1, 15)),
    (2020, "HOLIKA", (3, 9)),
    (2020, "RAKSHABANDHAN", (8, 3)),
    (2020, "ANANTA_CHATURDASHI", (9, 1)),
    (2020, "NAVARATRA_ARAMBHA", (10, 17)),
    (2020, "VIJAYA_DASHAMI_NEXT_DAY", (10, 27)),
    (2020, "CHHATH_KHARNA", (11, 19)),
    (2020, "HOLI", (3, 10)),
    (2020, "HOLI_SECOND_DAY", (3, 11)),
    (2020, "RAMA_NAVAMI", (4, 2)),
    (2020, "GOOD_FRIDAY", (4, 10)),
    (2020, "JANMASHTAMI", (8, 11)),
    (2020, "MAHA_ASHTAMI", (10, 24)),
    (2020, "MAHA_NAVAMI", (10, 25)),
    (2020, "VIJAYA_DASHAMI", (10, 26)),
    (2020, "LAKSHMI_PUJA", (11, 14)),
    (2020, "CHHATH", (11, 20)),
    (2020, "CHHATH_SECOND_DAY", (11, 21)),
    (2021, "GURU_GOBIND_SINGH_JAYANTI", (1, 20)),
    (2021, "BASANTA_PANCHAMI", (2, 16)),
    (2021, "RAVIDAS_JAYANTI", (2, 27)),
    (2021, "SHIVARATRI", (3, 11)),
    (2021, "SAMRAT_ASHOK_ASHTAMI", (4, 20)),
    (2021, "MAHAVIR_JAYANTI", (4, 25)),
    (2021, "SITA_NAVAMI", (5, 20)),
    (2021, "BUDDHA_JAYANTI", (5, 26)),
    (2021, "KABIR_JAYANTI", (6, 24)),
    (2021, "PHULPATI", (10, 12)),
    (2021, "YAMA_DWITIYA", (11, 6)),
    (2021, "UTTARAYANA_PUNYAKALA", (1, 14)),
    (2021, "HOLIKA", (3, 28)),
    (2021, "RAKSHABANDHAN", (8, 22)),
    (2021, "ANANTA_CHATURDASHI", (9, 19)),
    (2021, "NAVARATRA_ARAMBHA", (10, 7)),
    (2021, "VIJAYA_DASHAMI_NEXT_DAY", (10, 16)),
    (2021, "CHHATH_KHARNA", (11, 9)),
    (2021, "HOLI", (3, 29)),
    (2021, "HOLI_SECOND_DAY", (3, 30)),
    (2021, "RAMA_NAVAMI", (4, 21)),
    (2021, "GOOD_FRIDAY", (4, 2)),
    (2021, "JANMASHTAMI", (8, 30)),
    (2021, "MAHA_ASHTAMI", (10, 13)),
    (2021, "MAHA_NAVAMI", (10, 14)),
    (2021, "VIJAYA_DASHAMI", (10, 15)),
    (2021, "LAKSHMI_PUJA", (11, 4)),
    (2021, "CHHATH", (11, 10)),
    (2021, "CHHATH_SECOND_DAY", (11, 11)),
    (2022, "GURU_GOBIND_SINGH_JAYANTI", (1, 9)),
    (2022, "BASANTA_PANCHAMI", (2, 5)),
    (2022, "RAVIDAS_JAYANTI", (2, 16)),
    (2022, "SHIVARATRI", (3, 1)),
    (2022, "SAMRAT_ASHOK_ASHTAMI", (4, 9)),
    (2022, "MAHAVIR_JAYANTI", (4, 14)),
    (2022, "SITA_NAVAMI", (5, 10)),
    (2022, "BUDDHA_JAYANTI", (5, 16)),
    (2022, "KABIR_JAYANTI", (6, 14)),
    (2022, "PHULPATI", (10, 2)),
    (2022, "YAMA_DWITIYA", (10, 27)),
    (2022, "UTTARAYANA_PUNYAKALA", (1, 15)),
    (2022, "HOLIKA", (3, 17)),
    (2022, "RAKSHABANDHAN", (8, 12)),
    (2022, "ANANTA_CHATURDASHI", (9, 9)),
    (2022, "NAVARATRA_ARAMBHA", (9, 26)),
    (2022, "VIJAYA_DASHAMI_NEXT_DAY", (10, 6)),
    (2022, "CHHATH_KHARNA", (10, 29)),
    (2022, "HOLI", (3, 18)),
    (2022, "HOLI_SECOND_DAY", (3, 19)),
    (2022, "RAMA_NAVAMI", (4, 10)),
    (2022, "GOOD_FRIDAY", (4, 15)),
    (2022, "JANMASHTAMI", (8, 19)),
    (2022, "MAHA_ASHTAMI", (10, 3)),
    (2022, "MAHA_NAVAMI", (10, 4)),
    (2022, "VIJAYA_DASHAMI", (10, 5)),
    (2022, "LAKSHMI_PUJA", (10, 24)),
    (2022, "CHHATH", (10, 30)),
    (2022, "CHHATH_SECOND_DAY", (10, 31)),
    (2023, "GURU_GOBIND_SINGH_JAYANTI", (1, 5)),
    (2023, "BASANTA_PANCHAMI", (1, 26)),
    (2023, "RAVIDAS_JAYANTI", (2, 5)),
    (2023, "SHIVARATRI", (2, 18)),
    (2023, "SAMRAT_ASHOK_ASHTAMI", (3, 29)),
    (2023, "MAHAVIR_JAYANTI", (4, 4)),
    (2023, "SITA_NAVAMI", (4, 29)),
    (2023, "BUDDHA_JAYANTI", (5, 5)),
    (2023, "KABIR_JAYANTI", (6, 4)),
    (2023, "PHULPATI", (10, 21)),
    (2023, "YAMA_DWITIYA", (11, 15)),
    (2023, "UTTARAYANA_PUNYAKALA", (1, 15)),
    (2023, "HOLIKA", (3, 6)),
    (2023, "RAKSHABANDHAN", (8, 30)),
    (2023, "ANANTA_CHATURDASHI", (9, 28)),
    (2023, "NAVARATRA_ARAMBHA", (10, 15)),
    (2023, "VIJAYA_DASHAMI_NEXT_DAY", (10, 25)),
    (2023, "CHHATH_KHARNA", (11, 18)),
    (2023, "HOLI", (3, 8)),
    (2023, "HOLI_SECOND_DAY", (3, 9)),
    (2023, "RAMA_NAVAMI", (3, 30)),
    (2023, "GOOD_FRIDAY", (4, 7)),
    (2023, "JANMASHTAMI", (9, 6)),
    (2023, "MAHA_ASHTAMI", (10, 22)),
    (2023, "MAHA_NAVAMI", (10, 23)),
    (2023, "VIJAYA_DASHAMI", (10, 24)),
    (2023, "LAKSHMI_PUJA", (11, 12)),
    (2023, "CHHATH", (11, 19)),
    (2023, "CHHATH_SECOND_DAY", (11, 20)),
    (2024, "GURU_GOBIND_SINGH_JAYANTI", (1, 17)),
    (2024, "BASANTA_PANCHAMI", (2, 14)),
    (2024, "RAVIDAS_JAYANTI", (2, 24)),
    (2024, "SHIVARATRI", (3, 8)),
    (2024, "SAMRAT_ASHOK_ASHTAMI", (4, 16)),
    (2024, "MAHAVIR_JAYANTI", (4, 21)),
    (2024, "SITA_NAVAMI", (5, 17)),
    (2024, "BUDDHA_JAYANTI", (5, 23)),
    (2024, "KABIR_JAYANTI", (6, 22)),
    (2024, "PHULPATI", (10, 10)),
    (2024, "YAMA_DWITIYA", (11, 3)),
    (2024, "UTTARAYANA_PUNYAKALA", (1, 15)),
    (2024, "HOLIKA", (3, 24)),
    (2024, "RAKSHABANDHAN", (8, 19)),
    (2024, "ANANTA_CHATURDASHI", (9, 17)),
    (2024, "NAVARATRA_ARAMBHA", (10, 3)),
    (2024, "VIJAYA_DASHAMI_NEXT_DAY", (10, 13)),
    (2024, "CHHATH_KHARNA", (11, 6)),
    (2024, "HOLI", (3, 26)),
    (2024, "HOLI_SECOND_DAY", (3, 27)),
    (2024, "RAMA_NAVAMI", (4, 17)),
    (2024, "GOOD_FRIDAY", (3, 29)),
    (2024, "JANMASHTAMI", (8, 26)),
    (2024, "MAHA_ASHTAMI", (10, 11)),
    (2024, "MAHA_NAVAMI", (10, 11)),
    (2024, "VIJAYA_DASHAMI", (10, 12)),
    (2024, "LAKSHMI_PUJA", (10, 31)),
    (2024, "CHHATH", (11, 7)),
    (2024, "CHHATH_SECOND_DAY", (11, 8)),
    (2025, "GURU_GOBIND_SINGH_JAYANTI", (1, 6)),
    (2025, "BASANTA_PANCHAMI", (2, 3)),
    (2025, "RAVIDAS_JAYANTI", (2, 12)),
    (2025, "SHIVARATRI", (2, 26)),
    (2025, "SAMRAT_ASHOK_ASHTAMI", (4, 5)),
    (2025, "MAHAVIR_JAYANTI", (4, 10)),
    (2025, "SITA_NAVAMI", (5, 6)),
    (2025, "BUDDHA_JAYANTI", (5, 12)),
    (2025, "KABIR_JAYANTI", (6, 11)),
    (2025, "PHULPATI", (9, 29)),
    (2025, "YAMA_DWITIYA", (10, 23)),
    (2025, "UTTARAYANA_PUNYAKALA", (1, 14)),
    (2025, "HOLIKA", (3, 13)),
    (2025, "RAKSHABANDHAN", (8, 9)),
    (2025, "HARITALIKA", (8, 26)),
    (2025, "ANANTA_CHATURDASHI", (9, 6)),
    (2025, "JITIYA", (9, 14)),
    (2025, "NAVARATRA_ARAMBHA", (9, 22)),
    (2025, "VIJAYA_DASHAMI_NEXT_DAY", (10, 3)),
    (2025, "CHHATH_KHARNA", (10, 26)),
    (2025, "HOLI", (3, 14)),
    (2025, "HOLI_SECOND_DAY", (3, 15)),
    (2025, "RAMA_NAVAMI", (4, 6)),
    (2025, "GOOD_FRIDAY", (4, 18)),
    (2025, "JANMASHTAMI", (8, 16)),
    (2025, "MAHA_ASHTAMI", (9, 30)),
    (2025, "MAHA_NAVAMI", (10, 1)),
    (2025, "VIJAYA_DASHAMI", (10, 2)),
    (2025, "LAKSHMI_PUJA", (10, 22)),
    (2025, "CHHATH", (10, 27)),
    (2025, "CHHATH_SECOND_DAY", (10, 28)),
    (2025, "GURU_GOBIND_SINGH_JAYANTI", (12, 27)),
    (2026, "BASANTA_PANCHAMI", (1, 23)),
    (2026, "RAVIDAS_JAYANTI", (2, 1)),
    (2026, "SHIVARATRI", (2, 15)),
    (2026, "SAMRAT_ASHOK_ASHTAMI", (3, 26)),
    (2026, "SITA_NAVAMI", (4, 25)),
    (2026, "KABIR_JAYANTI", (6, 29)),
    (2026, "PHULPATI", (10, 17)),
    (2026, "YAMA_DWITIYA", (11, 10)),
    (2026, "UTTARAYANA_PUNYAKALA", (1, 14)),
    (2026, "HOLIKA", (3, 2)),
    (2026, "RAKSHABANDHAN", (8, 28)),
    (2026, "HARITALIKA", (9, 14)),
    (2026, "ANANTA_CHATURDASHI", (9, 25)),
    (2026, "JITIYA", (10, 4)),
    (2026, "NAVARATRA_ARAMBHA", (10, 11)),
    (2026, "VIJAYA_DASHAMI_NEXT_DAY", (10, 21)),
    (2026, "CHHATH_KHARNA", (11, 14)),
    (2026, "HOLI", (3, 3)),
    (2026, "HOLI_SECOND_DAY", (3, 4)),
    (2026, "RAMA_NAVAMI", (3, 27)),
    (2026, "MAHAVIR_JAYANTI", (3, 31)),
    (2026, "GOOD_FRIDAY", (4, 3)),
    (2026, "BUDDHA_JAYANTI", (5, 1)),
    (2026, "JANMASHTAMI", (9, 4)),
    (2026, "MAHA_ASHTAMI", (10, 18)),
    (2026, "MAHA_NAVAMI", (10, 19)),
    (2026, "VIJAYA_DASHAMI", (10, 20)),
    (2026, "LAKSHMI_PUJA", (11, 8)),
    (2026, "CHHATH", (11, 15)),
    (2026, "CHHATH_SECOND_DAY", (11, 16)),
];

/// The days every notification lists on one date, in any schedule.
const CIVIL: [(&str, (u8, u8)); 16] = [
    ("NEW_YEAR_S_DAY_AD", (1, 1)),
    ("KARPOORI_THAKUR_JAYANTI", (1, 24)),
    ("INDIA_REPUBLIC_DAY", (1, 26)),
    ("BIHAR_DIWAS", (3, 22)),
    ("AMBEDKAR_JAYANTI", (4, 14)),
    ("VEER_KUNWAR_SINGH_JAYANTI", (4, 23)),
    ("INTERNATIONAL_LABOUR_DAY", (5, 1)),
    ("ANUGRAH_NARAYAN_SINHA_JAYANTI", (6, 18)),
    ("INDIA_INDEPENDENCE_DAY", (8, 15)),
    ("VISHWAKARMA_PUJA", (9, 17)),
    ("GANDHI_JAYANTI", (10, 2)),
    ("JAYAPRAKASH_NARAYAN_JAYANTI", (10, 11)),
    ("SHRI_KRISHNA_SINHA_JAYANTI", (10, 21)),
    ("RAJENDRA_PRASAD_JAYANTI", (12, 3)),
    ("CHRISTMAS_EVE", (12, 24)),
    ("CHRISTMAS", (12, 25)),
];

/// A key that parts: the year, the key and the days the rule gives.
type Part = (i32, &'static str, &'static [(u8, u8)]);

/// The keys that part from the print, with the days the rule gives
/// (`festival-measured.md` §22, C1561, C1565 to C1571). Guru Gobind Singh's
/// birthday of 2022 meets on its first day, which the page's row holds, and
/// parts on its second, 29 December, where the 2023 list prints 5 January.
const PARTS: [Part; 23] = [
    (2019, "SAMRAT_ASHOK_ASHTAMI", &[(4, 13)]),
    (2019, "MAHA_NAVAMI", &[(10, 6)]),
    (2020, "MAHA_NAVAMI", &[(10, 24)]),
    (2021, "SITA_NAVAMI", &[(5, 21)]),
    (2022, "UTTARAYANA_PUNYAKALA", &[(1, 14)]),
    (2022, "GURU_GOBIND_SINGH_JAYANTI", &[(1, 9), (12, 29)]),
    (2022, "RAKSHABANDHAN", &[(8, 11)]),
    (2022, "JANMASHTAMI", &[(8, 18)]),
    (2023, "GURU_GOBIND_SINGH_JAYANTI", &[]),
    (2023, "HOLIKA", &[(3, 7)]),
    (2024, "HOLI", &[(3, 25)]),
    (2024, "HOLI_SECOND_DAY", &[(3, 26)]),
    (2024, "VIJAYA_DASHAMI", &[(10, 13)]),
    (2024, "VIJAYA_DASHAMI_NEXT_DAY", &[(10, 14)]),
    (2025, "JANMASHTAMI", &[(8, 15)]),
    (2025, "LAKSHMI_PUJA", &[(10, 20)]),
    (2025, "CHHATH_KHARNA", &[(10, 27)]),
    (2025, "CHHATH", &[(10, 28)]),
    (2025, "CHHATH_SECOND_DAY", &[(10, 29)]),
    (2026, "RAMA_NAVAMI", &[(3, 26)]),
    (2026, "PHULPATI", &[(10, 18)]),
    (2026, "MAHA_ASHTAMI", &[(10, 19)]),
    (2026, "YAMA_DWITIYA", &[(11, 11)]),
];

/// The days each notification prints rather than reckons, as (marked,
/// unmarked): the ten Islamic days the note leaves to the moon, the last
/// Monday of Shravana, and Guru Nanak's birthday in 2019 and 2020.
const PRINTED_DAYS: [(i32, usize, usize); 8] = [
    (2019, 10, 2),
    (2020, 10, 2),
    (2021, 10, 1),
    (2022, 10, 1),
    (2023, 10, 1),
    (2024, 10, 1),
    (2025, 10, 1),
    (2026, 10, 1),
];

/// Patna: the Old Secretariat.
fn patna() -> Place {
    Place::new(
        Latitude::literal(25.5941),
        Longitude::literal(85.1376),
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

/// Holds `year`'s notification to the pack: the days of each listed key
/// meet the print or are a named parting giving the days named, the fixed
/// days stand on their date, and the printed days stand, the Islamic ones
/// marked.
fn meets(year: i32) {
    let answer = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
        .almanac()
        .festivals(
            &date(year, 1, 1),
            &date(year, 12, 31),
            &patna(),
            UtcOffset::literal(5, 30, 0),
            &FestivalRequest::from(FestivalPack::IndiaBihar),
        )
        .unwrap()
        .value;
    let mut keys: Vec<&str> = LISTED
        .iter()
        .filter(|row| row.0 == year)
        .map(|row| row.1)
        .collect();
    keys.sort_unstable();
    keys.dedup();
    let mut parted: Vec<(&str, Vec<(u8, u8)>)> = keys
        .into_iter()
        .filter_map(|key| {
            let listed: Vec<(u8, u8)> = LISTED
                .iter()
                .filter(|row| row.0 == year && row.1 == key)
                .map(|row| row.2)
                .collect();
            let found = days_of(&answer, key);
            (found != listed).then_some((key, found))
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
    let &(_, marked, unmarked) = PRINTED_DAYS.iter().find(|row| row.0 == year).unwrap();
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
