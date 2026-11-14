//! The `INDIA_BIHAR` pack against the Government of Bihar's notifications
//! (`03-design/bihar-holidays.md`, SR-03), as [`super::uttar_pradesh`]
//! holds Uttar Pradesh's, through the same comparison.
//!
//! Every rule of the pack, at Patna on India's clock, against each year's
//! general, public and restricted holidays; every parting is named with
//! its cause, which the pass checks both ways. The days the notifications
//! print rather than reckon are held only to stand in their year, the
//! moon's marked, and a year with no notification holds none.

use teistro::FestivalPack;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};

use super::MonthDay;
use super::india::{Measured, Notices, Section, Sky};

/// Each year's notification of the General Administration Department, and
/// the copy read.
const NOTICES: [(i32, &str); 8] = [
    (
        2019,
        "3/एच0-02/2018-सा0प्र0 15543, 29 November 2018: rank 1 as copied, the signed scan a compliance site republished",
    ),
    (
        2020,
        "3/एच0-03/2019-13501, 30 September 2019: rank 1 as copied, the signed scan a compliance site republished",
    ),
    (
        2021,
        "3/एच0-02/2020-12008, 16 December 2020: rank 1 as copied, the signed scan a compliance site republished",
    ),
    (
        2022,
        "3/एच0-01/2021-11265, 24 September 2021: rank 1 as copied, the signed scan a compliance site republished",
    ),
    (
        2023,
        "3/एच0-01/2022-15471, 31 August 2022: rank 1 as copied, the Bihar Gazette, Extraordinary, No. 653, as an HR firm republished it",
    ),
    (
        2024,
        "3/एच0-02/2023-16953, 5 September 2023: rank 1 as copied, the Bihar Gazette, Extraordinary, No. 736, as an HR firm republished it",
    ),
    (
        2025,
        "3/एच0-01/2024-15764, 1 October 2024: rank 1 as copied, the Bihar Gazette, Extraordinary, No. 956, as an HR firm republished it",
    ),
    (
        2026,
        "3/एच0-01/2025-15401, 19 August 2025: rank 1 as copied, the signed scan an HR firm republished",
    ),
];

/// Each year's reckoned days, of every schedule, less the days every
/// notification lists on one date, which [`CIVIL`] holds in every year.
/// A row is a key's first day of the year: 2025's second Guru Gobind
/// Singh's birthday, 27 December, is held by `crates/sdk/tests/bihar.rs`.
const LISTED: [(i32, &str, MonthDay); 235] = [
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
const CIVIL: [(&str, MonthDay); 16] = [
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

/// Why a rule parts from a notification, per (year, rule).
const PARTS: [(i32, &str, &str); 22] = [
    (
        2019,
        "SAMRAT_ASHOK_ASHTAMI",
        "Chaitra's bright 8th runs from 13:24 on 12 April to 11:42 on the 13th; the print's rule takes the 13th, whose sunrise it holds, and the notification lists the 12th, the day it fills (C1571)",
    ),
    (
        2019,
        "MAHA_NAVAMI",
        "the bright 9th runs from 10:55 on 6 October to 12:38 on the 7th, holding both middays; Telangana's rule takes the 6th, and the notification lists the 7th, whose sunrise it holds (C1567)",
    ),
    (
        2020,
        "MAHA_NAVAMI",
        "the bright 9th runs from 06:59 on 24 October to 07:42 on the 25th; Telangana's rule takes the 24th, whose midday it holds, and the notification lists the 25th, whose sunrise it holds (C1567)",
    ),
    (
        2021,
        "SITA_NAVAMI",
        "Vaishakha's bright 9th runs from 12:23 on 20 May to 11:11 on the 21st; the rule takes the 21st, whose sunrise it holds, and the notification lists the 20th, whose midday it holds (C1561)",
    ),
    (
        2022,
        "UTTARAYANA_PUNYAKALA",
        "the Sun enters Makara at 14:35 on 14 January, before sunset, and the rule takes the 14th; the notification lists the 15th, where 2026's, with the entry at 15:05, lists the day of the entry (C1566)",
    ),
    (
        2022,
        "RAKSHABANDHAN",
        "the full moon runs from 10:39 on 11 August to 07:06 on the 12th; Dharmasindhu's rule takes the 11th, whose afternoon it holds, and the notification lists the 12th, whose sunrise it holds (C1571)",
    ),
    (
        2022,
        "JANMASHTAMI",
        "the dark 8th runs from 21:22 on 18 August to 23:00 on the 19th; Dharmasindhu's rule takes the 18th, whose midnight it holds, and the notification lists the 19th, whose sunrise it holds (C1571)",
    ),
    (
        2023,
        "GURU_GOBIND_SINGH_JAYANTI",
        "no Pausha bright 7th falls in 2023, the one before on 29 December 2022 and the one after on 17 January 2024, so the rule finds none; the notification lists 5 January, Pausha's bright 14th (C1565)",
    ),
    (
        2023,
        "HOLIKA",
        "the full moon runs from 16:18 on 6 March to 18:10 on the 7th; Dharmasindhu's rule takes the 7th, whose evening it holds after sunset, and the notification lists the 6th (C1570)",
    ),
    (
        2024,
        "HOLI",
        "Chaitra's dark 1st runs from 12:30 on 25 March to 14:56 on the 26th; the rule counts the day after the Holika fire, the 25th, and the notification lists the 26th, whose sunrise it holds (C1570)",
    ),
    (
        2024,
        "HOLI_SECOND_DAY",
        "Holi's second day follows the fire by two days, the 26th; the notification lists the 27th, the day after its Holi (C1570)",
    ),
    (
        2024,
        "VIJAYA_DASHAMI",
        "the bright 10th runs from 10:59 on 12 October to 09:09 on the 13th; Dharmasindhu's visarjana takes the 13th, whose sunrise it holds, and the notification lists the 12th, whose afternoon it holds (C1567)",
    ),
    (
        2024,
        "VIJAYA_DASHAMI_NEXT_DAY",
        "the day after Vijaya Dashami follows the rule's 13th, the 14th; the notification lists the 13th, the day after its own (C1567)",
    ),
    (
        2025,
        "JANMASHTAMI",
        "the dark 8th runs from 23:50 on 15 August to 21:35 on the 16th; Dharmasindhu's rule takes the 15th, whose midnight it holds, and the notification lists the 16th, whose sunrise it holds (C1571)",
    ),
    (
        2025,
        "LAKSHMI_PUJA",
        "the new moon runs from 15:45 on 20 October to 17:55 on the 21st; Telangana's rule takes the 20th, whose evening it holds, and the notification lists the 22nd; a later order of the department moved the holiday to the 20th (C1569)",
    ),
    (
        2025,
        "CHHATH_KHARNA",
        "Kharna follows Chhath's parting: the rule counts the day before the 28th, the 27th, and the notification lists the 26th (C1568)",
    ),
    (
        2025,
        "CHHATH",
        "the bright 6th runs from 06:05 on 27 October, ten minutes after sunrise, to 08:00 on the 28th; the print's rule takes the 28th, whose sunrise it holds, and the notification lists the 27th, the evening's offering (C1568)",
    ),
    (
        2025,
        "CHHATH_SECOND_DAY",
        "the morning arghya follows Chhath's parting: the rule counts the 29th, and the notification lists the 28th (C1568)",
    ),
    (
        2026,
        "RAMA_NAVAMI",
        "Chaitra's bright 9th runs from 11:49 on 26 March to 10:07 on the 27th; the central list's rule takes the 26th, whose midday it holds, and the notification lists the 27th, whose sunrise it holds (C1571)",
    ),
    (
        2026,
        "PHULPATI",
        "the bright 7th begins at 05:55 on 17 October, six minutes after sunrise; the print's rule takes the 18th, whose sunrise it holds, and the notification lists the 17th (C1567)",
    ),
    (
        2026,
        "MAHA_ASHTAMI",
        "the bright 8th begins at 08:29 on 18 October; the print's rule takes the 19th, whose sunrise it holds, and the notification lists the 18th, the day it fills (C1567)",
    ),
    (
        2026,
        "YAMA_DWITIYA",
        "Kartika's bright 2nd runs from 14:01 on 10 November to 15:54 on the 11th; Dharmasindhu's rule takes the 11th, and the notification lists the 10th; the department's amendment of 6 November 2026 moved the holiday to the 11th (C1569)",
    ),
];

/// The days each year's notification prints rather than reckons: the ten
/// Islamic days the note leaves to the moon, the last Monday of Shravana,
/// and Guru Nanak's birthday in 2019 and 2020.
const PRINTED: [usize; 8] = [12, 12, 11, 11, 11, 11, 11, 11];

/// The page's section on the list.
const SECTION: Section = Section {
    heading: "22. Bihar's public holidays against the notifications",
    against: "`INDIA_BIHAR` at Patna on India's clock, against each year's general holidays \
         (Schedule-1), public holidays under the Negotiable Instruments Act (Schedule-3) and \
         restricted holidays (Schedule-2) of the General Administration Department's \
         notification. The days are reckoned under the modern sky (the built-in ephemeris, \
         the default profile).",
    civil: "civil",
    printed: "The days the notifications print are not reckoned: the six public and four \
         restricted Islamic days the note leaves to the sighting of the moon (Shab-e-Barat, \
         Id-ul-Fitr, Id-uz-Zoha, Muharram, Chehlum and the Prophet's birthday; Jamat-ul-Vida \
         and the days after the two Ids and Muharram), each marked; the last Monday of \
         Shravana, unmarked; and Guru Nanak's birthday, listed in 2019 and 2020, unmarked. \
         2031, with no notification, holds none.",
};

/// Patna, the seat the notifications are issued from: the Old Secretariat.
fn patna() -> Place {
    Place::new(
        Latitude::literal(25.5941),
        Longitude::literal(85.1376),
        Altitude::literal(0.0),
    )
}

/// The state's list.
fn bihar() -> Notices {
    Notices {
        pack: FestivalPack::IndiaBihar,
        sky: Sky::Drik,
        place: patna(),
        years: &NOTICES,
        listed: &LISTED,
        civil: &CIVIL,
        parts: &PARTS,
        printed: &PRINTED,
        unnoticed: 2031,
    }
}

/// Every year's rows, and a problem for every parting no entry names.
pub(super) fn compare(problems: &mut Vec<String>) -> Result<Measured, String> {
    bihar().compare(problems)
}

/// §22 of the page.
pub(super) fn render(out: &mut String, measured: &Measured) {
    bihar().render(out, &SECTION, measured);
}
