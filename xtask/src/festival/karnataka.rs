//! The `INDIA_KARNATAKA` pack against the Government of Karnataka's
//! notifications (`03-design/karnataka-holidays.md`, SR-03), as
//! [`super::assam`] holds Assam's, through the same comparison.
//!
//! Every rule of the pack, at Bengaluru on India's clock, against each
//! year's general holidays (Notification-I's table and the days its Note 1
//! names on a Sunday or a second or fourth Saturday), the Annexure's
//! restricted holidays, and the banks' closing of accounts in the two
//! years Notification-II was read; every parting is named with its cause,
//! which the pass checks both ways. The days the notifications print
//! rather than reckon are held only to stand in their year, the moon's
//! marked, and a year with no notification holds none.

use std::fmt::Write as _;

use teistro::FestivalPack;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};

use super::MonthDay;
use super::india::{Measured, Notices, Sky, printed_table};

/// Each year's notification of the Department of Personnel and
/// Administrative Reforms, and the copy read.
const NOTICES: [(i32, &str); 8] = [
    (
        2019,
        "DPAR 13 HHL 2018, November 2018: rank 3, a news site's list of the general holidays",
    ),
    (
        2020,
        "DPAR 14 HHL 2019, 22 November 2019: rank 1 as copied, the signed English scan",
    ),
    (
        2021,
        "DPAR 12 HHL 2020, 21 November 2020, Karnataka Rajyapatra Part IV-A No. 582: rank 1, the gazette's pages",
    ),
    (
        2022,
        "DPAR 37 HHL 2021, 30 November 2021, with its corrigendum of 3 December 2021: rank 1 as copied, the signed scans",
    ),
    (
        2023,
        "DPAR 16 HHL 2022, 21 November 2022: rank 1 as copied, the signed Kannada scan",
    ),
    (
        2024,
        "DPAR 16 HHL 2023, 28 November 2023: rank 1 as copied, the signed English scan",
    ),
    (
        2025,
        "DPAR 16 HHL 2024, 21 November 2024: rank 1 as copied, the signed Kannada and English scans",
    ),
    (
        2026,
        "DPAR 16 HHL 2025, 17 November 2025: rank 1 as copied, the signed Kannada scan",
    ),
];

/// Each year's reckoned days, the restricted ones from 2020 among them,
/// with the restricted fixed days and the banks' closing in the years it
/// was read, less the days the notifications print and the fixed general
/// days, which [`CIVIL`] holds in every year. 2019's list is the general
/// one alone.
const LISTED: [(i32, &str, MonthDay); 263] = [
    (2019, "UTTARAYANA_PUNYAKALA", (1, 15)),
    (2019, "SHIVARATRI", (3, 4)),
    (2019, "SAMVATSARA_ARAMBHA", (4, 6)),
    (2019, "SOURAMANA_UGADI", (4, 14)),
    (2019, "MAHAVIR_JAYANTI", (4, 17)),
    (2019, "GOOD_FRIDAY", (4, 19)),
    (2019, "BASAVA_JAYANTI", (5, 7)),
    (2019, "GANESHA_CHATURTHI", (9, 2)),
    (2019, "AMAVASYA_SHRADDHA", (9, 28)),
    (2019, "MAHA_NAVAMI", (10, 7)),
    (2019, "VIJAYA_DASHAMI", (10, 8)),
    (2019, "VALMIKI_JAYANTI", (10, 13)),
    (2019, "NARAK_CHATURDASHI", (10, 27)),
    (2019, "BALI_PRATIPADA", (10, 29)),
    (2019, "KANAKADASA_JAYANTI", (11, 15)),
    (2020, "NEW_YEAR_S_DAY_AD", (1, 1)),
    (2020, "UTTARAYANA_PUNYAKALA", (1, 15)),
    (2020, "MADHVA_NAVAMI", (2, 3)),
    (2020, "SHIVARATRI", (2, 21)),
    (2020, "HOLIKA", (3, 9)),
    (2020, "SAMVATSARA_ARAMBHA", (3, 25)),
    (2020, "DEVARA_DASIMAYYA_JAYANTI", (3, 29)),
    (2020, "BANK_ACCOUNTS_CLOSING", (4, 1)),
    (2020, "RAMA_NAVAMI", (4, 2)),
    (2020, "MAHAVIR_JAYANTI", (4, 6)),
    (2020, "GOOD_FRIDAY", (4, 10)),
    (2020, "EASTER_SATURDAY", (4, 11)),
    (2020, "SOURAMANA_UGADI", (4, 13)),
    (2020, "BASAVA_JAYANTI", (4, 26)),
    (2020, "RAMANUJACHARYA_JAYANTI", (4, 28)),
    (2020, "SHANKARACHARYA_JAYANTI", (4, 28)),
    (2020, "BUDDHA_JAYANTI", (5, 7)),
    (2020, "JANMASHTAMI", (8, 11)),
    (2020, "SWARNA_GOWRI_VRATA", (8, 21)),
    (2020, "GANESHA_CHATURTHI", (8, 22)),
    (2020, "ANANTA_CHATURDASHI", (8, 31)),
    (2020, "THIRUVONAM", (8, 31)),
    (2020, "UPAKARMA_TAITTIRIYA", (9, 1)),
    (2020, "SREE_NARAYANA_GURU_JAYANTHI", (9, 2)),
    (2020, "KANYA_MARIYAMMA_JAYANTI", (9, 8)),
    (2020, "AMAVASYA_SHRADDHA", (9, 17)),
    (2020, "VISHWAKARMA_PUJA", (9, 17)),
    (2020, "TULA_SANKRANTI", (10, 17)),
    (2020, "MAHA_NAVAMI", (10, 25)),
    (2020, "VIJAYA_DASHAMI", (10, 26)),
    (2020, "VALMIKI_JAYANTI", (10, 31)),
    (2020, "NARAK_CHATURDASHI", (11, 14)),
    (2020, "BALI_PRATIPADA", (11, 16)),
    (2020, "GURU_NANAK_JAYANTI", (11, 30)),
    (2020, "KANAKADASA_JAYANTI", (12, 3)),
    (2020, "CHRISTMAS_EVE", (12, 24)),
    (2021, "NEW_YEAR_S_DAY_AD", (1, 1)),
    (2021, "UTTARAYANA_PUNYAKALA", (1, 14)),
    (2021, "MADHVA_NAVAMI", (2, 21)),
    (2021, "SHIVARATRI", (3, 11)),
    (2021, "HOLIKA", (3, 28)),
    (2021, "GOOD_FRIDAY", (4, 2)),
    (2021, "EASTER_SATURDAY", (4, 3)),
    (2021, "SAMVATSARA_ARAMBHA", (4, 13)),
    (2021, "SOURAMANA_UGADI", (4, 14)),
    (2021, "DEVARA_DASIMAYYA_JAYANTI", (4, 17)),
    (2021, "RAMANUJACHARYA_JAYANTI", (4, 18)),
    (2021, "RAMA_NAVAMI", (4, 21)),
    (2021, "MAHAVIR_JAYANTI", (4, 25)),
    (2021, "BASAVA_JAYANTI", (5, 14)),
    (2021, "SHANKARACHARYA_JAYANTI", (5, 17)),
    (2021, "BUDDHA_JAYANTI", (5, 26)),
    (2021, "THIRUVONAM", (8, 21)),
    (2021, "UPAKARMA_TAITTIRIYA", (8, 22)),
    (2021, "SREE_NARAYANA_GURU_JAYANTHI", (8, 23)),
    (2021, "JANMASHTAMI", (8, 29)),
    (2021, "KANYA_MARIYAMMA_JAYANTI", (9, 8)),
    (2021, "SWARNA_GOWRI_VRATA", (9, 9)),
    (2021, "GANESHA_CHATURTHI", (9, 10)),
    (2021, "VISHWAKARMA_PUJA", (9, 17)),
    (2021, "ANANTA_CHATURDASHI", (9, 19)),
    (2021, "AMAVASYA_SHRADDHA", (10, 6)),
    (2021, "MAHA_NAVAMI", (10, 14)),
    (2021, "VIJAYA_DASHAMI", (10, 15)),
    (2021, "TULA_SANKRANTI", (10, 18)),
    (2021, "VALMIKI_JAYANTI", (10, 20)),
    (2021, "NARAK_CHATURDASHI", (11, 3)),
    (2021, "BALI_PRATIPADA", (11, 5)),
    (2021, "GURU_NANAK_JAYANTI", (11, 19)),
    (2021, "KANAKADASA_JAYANTI", (11, 22)),
    (2021, "CHRISTMAS_EVE", (12, 24)),
    (2022, "NEW_YEAR_S_DAY_AD", (1, 1)),
    (2022, "UTTARAYANA_PUNYAKALA", (1, 15)),
    (2022, "MADHVA_NAVAMI", (2, 10)),
    (2022, "SHIVARATRI", (3, 1)),
    (2022, "HOLIKA", (3, 17)),
    (2022, "SAMVATSARA_ARAMBHA", (4, 2)),
    (2022, "DEVARA_DASIMAYYA_JAYANTI", (4, 6)),
    (2022, "RAMA_NAVAMI", (4, 10)),
    (2022, "MAHAVIR_JAYANTI", (4, 14)),
    (2022, "SOURAMANA_UGADI", (4, 14)),
    (2022, "GOOD_FRIDAY", (4, 15)),
    (2022, "EASTER_SATURDAY", (4, 16)),
    (2022, "BASAVA_JAYANTI", (5, 3)),
    (2022, "RAMANUJACHARYA_JAYANTI", (5, 5)),
    (2022, "SHANKARACHARYA_JAYANTI", (5, 6)),
    (2022, "BUDDHA_JAYANTI", (5, 16)),
    (2022, "UPAKARMA_TAITTIRIYA", (8, 11)),
    (2022, "JANMASHTAMI", (8, 19)),
    (2022, "SWARNA_GOWRI_VRATA", (8, 30)),
    (2022, "GANESHA_CHATURTHI", (8, 31)),
    (2022, "KANYA_MARIYAMMA_JAYANTI", (9, 8)),
    (2022, "THIRUVONAM", (9, 8)),
    (2022, "ANANTA_CHATURDASHI", (9, 9)),
    (2022, "SREE_NARAYANA_GURU_JAYANTHI", (9, 10)),
    (2022, "VISHWAKARMA_PUJA", (9, 17)),
    (2022, "AMAVASYA_SHRADDHA", (9, 25)),
    (2022, "MAHA_NAVAMI", (10, 4)),
    (2022, "VIJAYA_DASHAMI", (10, 5)),
    (2022, "VALMIKI_JAYANTI", (10, 9)),
    (2022, "TULA_SANKRANTI", (10, 18)),
    (2022, "NARAK_CHATURDASHI", (10, 24)),
    (2022, "BALI_PRATIPADA", (10, 26)),
    (2022, "GURU_NANAK_JAYANTI", (11, 8)),
    (2022, "KANAKADASA_JAYANTI", (11, 11)),
    (2022, "CHRISTMAS_EVE", (12, 24)),
    (2023, "NEW_YEAR_S_DAY_AD", (1, 1)),
    (2023, "UTTARAYANA_PUNYAKALA", (1, 15)),
    (2023, "MADHVA_NAVAMI", (1, 30)),
    (2023, "SHIVARATRI", (2, 18)),
    (2023, "HOLIKA", (3, 8)),
    (2023, "SAMVATSARA_ARAMBHA", (3, 22)),
    (2023, "DEVARA_DASIMAYYA_JAYANTI", (3, 26)),
    (2023, "RAMA_NAVAMI", (3, 30)),
    (2023, "MAHAVIR_JAYANTI", (4, 3)),
    (2023, "GOOD_FRIDAY", (4, 7)),
    (2023, "EASTER_SATURDAY", (4, 8)),
    (2023, "SOURAMANA_UGADI", (4, 14)),
    (2023, "BASAVA_JAYANTI", (4, 23)),
    (2023, "RAMANUJACHARYA_JAYANTI", (4, 25)),
    (2023, "SHANKARACHARYA_JAYANTI", (4, 25)),
    (2023, "BUDDHA_JAYANTI", (5, 5)),
    (2023, "THIRUVONAM", (8, 29)),
    (2023, "UPAKARMA_TAITTIRIYA", (8, 30)),
    (2023, "SREE_NARAYANA_GURU_JAYANTHI", (8, 31)),
    (2023, "JANMASHTAMI", (9, 6)),
    (2023, "KANYA_MARIYAMMA_JAYANTI", (9, 8)),
    (2023, "VISHWAKARMA_PUJA", (9, 17)),
    (2023, "GANESHA_CHATURTHI", (9, 18)),
    (2023, "SWARNA_GOWRI_VRATA", (9, 18)),
    (2023, "ANANTA_CHATURDASHI", (9, 28)),
    (2023, "AMAVASYA_SHRADDHA", (10, 14)),
    (2023, "TULA_SANKRANTI", (10, 18)),
    (2023, "MAHA_NAVAMI", (10, 23)),
    (2023, "VIJAYA_DASHAMI", (10, 24)),
    (2023, "VALMIKI_JAYANTI", (10, 28)),
    (2023, "NARAK_CHATURDASHI", (11, 12)),
    (2023, "BALI_PRATIPADA", (11, 14)),
    (2023, "GURU_NANAK_JAYANTI", (11, 27)),
    (2023, "KANAKADASA_JAYANTI", (11, 30)),
    (2023, "CHRISTMAS_EVE", (12, 24)),
    (2024, "NEW_YEAR_S_DAY_AD", (1, 1)),
    (2024, "UTTARAYANA_PUNYAKALA", (1, 15)),
    (2024, "MADHVA_NAVAMI", (2, 18)),
    (2024, "SHIVARATRI", (3, 8)),
    (2024, "HOLIKA", (3, 25)),
    (2024, "GOOD_FRIDAY", (3, 29)),
    (2024, "EASTER_SATURDAY", (3, 30)),
    (2024, "BANK_ACCOUNTS_CLOSING", (4, 1)),
    (2024, "SAMVATSARA_ARAMBHA", (4, 9)),
    (2024, "DEVARA_DASIMAYYA_JAYANTI", (4, 13)),
    (2024, "SOURAMANA_UGADI", (4, 13)),
    (2024, "RAMA_NAVAMI", (4, 17)),
    (2024, "MAHAVIR_JAYANTI", (4, 21)),
    (2024, "BASAVA_JAYANTI", (5, 10)),
    (2024, "RAMANUJACHARYA_JAYANTI", (5, 12)),
    (2024, "SHANKARACHARYA_JAYANTI", (5, 12)),
    (2024, "BUDDHA_JAYANTI", (5, 23)),
    (2024, "UPAKARMA_TAITTIRIYA", (8, 19)),
    (2024, "SREE_NARAYANA_GURU_JAYANTHI", (8, 20)),
    (2024, "JANMASHTAMI", (8, 26)),
    (2024, "SWARNA_GOWRI_VRATA", (9, 6)),
    (2024, "GANESHA_CHATURTHI", (9, 7)),
    (2024, "KANYA_MARIYAMMA_JAYANTI", (9, 8)),
    (2024, "THIRUVONAM", (9, 15)),
    (2024, "ANANTA_CHATURDASHI", (9, 16)),
    (2024, "VISHWAKARMA_PUJA", (9, 17)),
    (2024, "AMAVASYA_SHRADDHA", (10, 2)),
    (2024, "MAHA_NAVAMI", (10, 11)),
    (2024, "VIJAYA_DASHAMI", (10, 12)),
    (2024, "TULA_SANKRANTI", (10, 17)),
    (2024, "VALMIKI_JAYANTI", (10, 17)),
    (2024, "NARAK_CHATURDASHI", (10, 31)),
    (2024, "BALI_PRATIPADA", (11, 2)),
    (2024, "GURU_NANAK_JAYANTI", (11, 15)),
    (2024, "KANAKADASA_JAYANTI", (11, 18)),
    (2024, "CHRISTMAS_EVE", (12, 24)),
    (2025, "NEW_YEAR_S_DAY_AD", (1, 1)),
    (2025, "UTTARAYANA_PUNYAKALA", (1, 14)),
    (2025, "MADHVA_NAVAMI", (2, 6)),
    (2025, "SHIVARATRI", (2, 26)),
    (2025, "HOLIKA", (3, 13)),
    (2025, "SAMVATSARA_ARAMBHA", (3, 30)),
    (2025, "DEVARA_DASIMAYYA_JAYANTI", (4, 2)),
    (2025, "RAMA_NAVAMI", (4, 6)),
    (2025, "MAHAVIR_JAYANTI", (4, 10)),
    (2025, "SOURAMANA_UGADI", (4, 14)),
    (2025, "GOOD_FRIDAY", (4, 18)),
    (2025, "EASTER_SATURDAY", (4, 19)),
    (2025, "BASAVA_JAYANTI", (4, 30)),
    (2025, "RAMANUJACHARYA_JAYANTI", (5, 2)),
    (2025, "SHANKARACHARYA_JAYANTI", (5, 2)),
    (2025, "BUDDHA_JAYANTI", (5, 12)),
    (2025, "UPAKARMA_TAITTIRIYA", (8, 9)),
    (2025, "JANMASHTAMI", (8, 16)),
    (2025, "SWARNA_GOWRI_VRATA", (8, 26)),
    (2025, "GANESHA_CHATURTHI", (8, 27)),
    (2025, "THIRUVONAM", (9, 5)),
    (2025, "ANANTA_CHATURDASHI", (9, 6)),
    (2025, "SREE_NARAYANA_GURU_JAYANTHI", (9, 7)),
    (2025, "KANYA_MARIYAMMA_JAYANTI", (9, 8)),
    (2025, "VISHWAKARMA_PUJA", (9, 17)),
    (2025, "AMAVASYA_SHRADDHA", (9, 21)),
    (2025, "MAHA_NAVAMI", (10, 1)),
    (2025, "VIJAYA_DASHAMI", (10, 1)),
    (2025, "VALMIKI_JAYANTI", (10, 7)),
    (2025, "TULA_SANKRANTI", (10, 18)),
    (2025, "NARAK_CHATURDASHI", (10, 20)),
    (2025, "BALI_PRATIPADA", (10, 22)),
    (2025, "GURU_NANAK_JAYANTI", (11, 5)),
    (2025, "KANAKADASA_JAYANTI", (11, 8)),
    (2025, "CHRISTMAS_EVE", (12, 24)),
    (2026, "NEW_YEAR_S_DAY_AD", (1, 1)),
    (2026, "UTTARAYANA_PUNYAKALA", (1, 15)),
    (2026, "MADHVA_NAVAMI", (1, 27)),
    (2026, "SHIVARATRI", (2, 15)),
    (2026, "HOLIKA", (3, 2)),
    (2026, "SAMVATSARA_ARAMBHA", (3, 19)),
    (2026, "DEVARA_DASIMAYYA_JAYANTI", (3, 23)),
    (2026, "RAMA_NAVAMI", (3, 27)),
    (2026, "MAHAVIR_JAYANTI", (3, 31)),
    (2026, "GOOD_FRIDAY", (4, 3)),
    (2026, "EASTER_SATURDAY", (4, 4)),
    (2026, "SOURAMANA_UGADI", (4, 14)),
    (2026, "BASAVA_JAYANTI", (4, 20)),
    (2026, "SHANKARACHARYA_JAYANTI", (4, 21)),
    (2026, "RAMANUJACHARYA_JAYANTI", (4, 22)),
    (2026, "BUDDHA_JAYANTI", (5, 1)),
    (2026, "THIRUVONAM", (8, 26)),
    (2026, "UPAKARMA_TAITTIRIYA", (8, 27)),
    (2026, "RAKSHABANDHAN", (8, 28)),
    (2026, "SREE_NARAYANA_GURU_JAYANTHI", (8, 28)),
    (2026, "JANMASHTAMI", (9, 4)),
    (2026, "KANYA_MARIYAMMA_JAYANTI", (9, 8)),
    (2026, "GANESHA_CHATURTHI", (9, 14)),
    (2026, "SWARNA_GOWRI_VRATA", (9, 14)),
    (2026, "VISHWAKARMA_PUJA", (9, 17)),
    (2026, "ANANTA_CHATURDASHI", (9, 25)),
    (2026, "AMAVASYA_SHRADDHA", (10, 10)),
    (2026, "TULA_SANKRANTI", (10, 18)),
    (2026, "MAHA_NAVAMI", (10, 20)),
    (2026, "VIJAYA_DASHAMI", (10, 21)),
    (2026, "VALMIKI_JAYANTI", (10, 25)),
    (2026, "NARAK_CHATURDASHI", (11, 8)),
    (2026, "BALI_PRATIPADA", (11, 10)),
    (2026, "GURU_NANAK_JAYANTI", (11, 24)),
    (2026, "KANAKADASA_JAYANTI", (11, 27)),
    (2026, "CHRISTMAS_EVE", (12, 24)),
];

/// The general days every notification lists on one date.
const CIVIL: [(&str, MonthDay); 7] = [
    ("INDIA_REPUBLIC_DAY", (1, 26)),
    ("AMBEDKAR_JAYANTI", (4, 14)),
    ("INTERNATIONAL_LABOUR_DAY", (5, 1)),
    ("INDIA_INDEPENDENCE_DAY", (8, 15)),
    ("GANDHI_JAYANTI", (10, 2)),
    ("KANNADA_RAJYOTSAVA", (11, 1)),
    ("CHRISTMAS", (12, 25)),
];

/// Why a rule parts from a notification, per (year, rule).
const PARTS: [(i32, &str, &str); 17] = [
    (
        2020,
        "UPAKARMA_TAITTIRIYA",
        "Shravana's full moon runs from 21:29 on 2 August to 21:29 on the 3rd, and *Dharmasindhu*'s rule takes the 3rd; the notification lists 1 September, the full moon of the next lunar month, in solar Simha (C1382)",
    ),
    (
        2020,
        "ANANTA_CHATURDASHI",
        "the bright 14th runs from 08:49 on 31 August to 09:39 on 1 September, and the printed panchanga's rule takes the 1st; the notification lists 31 August, whose afternoon the tithi holds, on one row with Onam and Rig Upakarma (C1383)",
    ),
    (
        2021,
        "JANMASHTAMI",
        "the dark 8th runs from 23:26 on 29 August to 02:00 on the 31st, holding both midnights, and *Dharmasindhu*'s rule takes the 30th; the notification lists the 29th, a Sunday named in the Annexure's note (C1381)",
    ),
    (
        2021,
        "TULA_SANKRANTI",
        "`MALAYALAM_SOLAR` begins Thulam on 17 October, the sankranti falling before that day's three-fifths mark at Thiruvananthapuram; the notification lists the 18th, as in four other years (C1373)",
    ),
    (
        2022,
        "RAMANUJACHARYA_JAYANTI",
        "Ardra in Chittirai gives 6 May; the notification lists the 5th, the day before Shankaracharya Jayanti (C1374)",
    ),
    (
        2022,
        "JANMASHTAMI",
        "the dark 8th runs from 21:22 on 18 August to 23:00 on the 19th, and *Dharmasindhu*'s midnight rule takes the 18th; the notification lists the 19th, whose sunrise the ashtami holds (C1381)",
    ),
    (
        2023,
        "MAHAVIR_JAYANTI",
        "the bright 13th begins at 06:25 on 3 April, after Bengaluru's sunrise, and the central list's rule takes the 4th; the notification lists the 3rd, as West Bengal's does (C1242, C1385)",
    ),
    (
        2023,
        "GANESHA_CHATURTHI",
        "the bright 4th runs from 12:40 on 18 September to 13:44 on the 19th, and the printed panchanga's midday rule takes the 19th; the notification lists the 18th (C1386)",
    ),
    (
        2023,
        "HOLIKA",
        "the full moon runs from 16:18 on 6 March to 18:10 on the 7th, and *Dharmasindhu*'s pradosha rule takes the 6th; the notification lists the 8th, the day after the full moon ends (C1384)",
    ),
    (
        2024,
        "MAHA_NAVAMI",
        "the bright 9th runs from 12:07 on 11 October to 10:59 on the 12th, and the sunrise reading takes the 12th; the notification lists the 11th, whose afternoon the navami holds (C1380)",
    ),
    (
        2024,
        "VIJAYA_DASHAMI",
        "the bright 10th runs from 10:59 on 12 October to 09:09 on the 13th, and the sunrise reading takes the 13th; the notification lists the 12th, the day after the Navami (C1380)",
    ),
    (
        2024,
        "HOLIKA",
        "the full moon runs from 09:56 on 24 March to 12:30 on the 25th, and *Dharmasindhu*'s pradosha rule takes the 24th; the notification lists the 25th, the day the full moon ends (C1384)",
    ),
    (
        2024,
        "ANANTA_CHATURDASHI",
        "the bright 14th runs from 15:11 on 16 September to 11:45 on the 17th, and the printed panchanga's rule takes the 17th; the notification lists the 16th, beside Eid-Milad (C1383)",
    ),
    (
        2025,
        "VIJAYA_DASHAMI",
        "the bright 10th runs from 19:02 on 1 October to 19:11 on the 2nd, and the sunrise reading takes the 2nd, Gandhi Jayanti; the notification prints Vijayadashami on the 1st, on the Navami's row (C1380)",
    ),
    (
        2025,
        "JANMASHTAMI",
        "the dark 8th runs from 23:50 on 15 August to 21:35 on the 16th, and *Dharmasindhu*'s midnight rule takes the 15th; the notification lists the 16th, whose sunrise the ashtami holds (C1381)",
    ),
    (
        2026,
        "VALMIKI_JAYANTI",
        "the full moon runs from 11:57 on 25 October to 09:42 on the 26th, and the sunrise reading takes the 26th; the notification lists the 25th, a Sunday named in Note 1 (C1387)",
    ),
    (
        2026,
        "UPAKARMA_TAITTIRIYA",
        "Shravana's full moon runs from 09:09 on 27 August to 09:49 on the 28th, and *Dharmasindhu*'s rule takes the 28th, the day the notification keeps Raksha Bandhana; it lists Yajur Upakarma on the 27th (C1382)",
    ),
];

/// The days each year's notification prints rather than reckons: the four
/// general days set by the moon, and from 2020 the three restricted ones
/// and Varamahalakshmi Vrata, Rig Upakarma and Huttari.
const PRINTED: [usize; 8] = [4, 10, 10, 10, 10, 10, 10, 10];

/// Bengaluru, the seat the notifications are issued from.
fn bengaluru() -> Place {
    Place::new(
        Latitude::literal(12.9716),
        Longitude::literal(77.5946),
        Altitude::literal(0.0),
    )
}

/// The state's list.
fn karnataka() -> Notices {
    Notices {
        pack: FestivalPack::IndiaKarnataka,
        sky: Sky::Drik,
        place: bengaluru(),
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
    karnataka().compare(problems)
}

/// §15 of the page.
pub(super) fn render(out: &mut String, (rows, printed): &Measured) {
    let notices = karnataka();
    let parted = rows.iter().filter(|row| !row.agrees()).count();
    let _ = writeln!(
        out,
        "## 15. Karnataka's public holidays against the notifications\n\n\
         `INDIA_KARNATAKA` at Bengaluru on India's clock, against each year's general \
         holidays of the Department of Personnel and Administrative Reforms' Notification-I \
         (its table and the days its Note 1 names on a Sunday or a second or fourth \
         Saturday), the Annexure's restricted holidays from 2020, and the banks' closing of \
         accounts in 2020 and 2024, the years Notification-II was read. The days are \
         reckoned under the modern sky (the built-in ephemeris, the default profile). 2019's \
         list is a news site's (rank 3), its general days alone. {} of {} rows agree.\n",
        rows.len() - parted,
        rows.len()
    );
    for (year, notice) in NOTICES {
        let _ = writeln!(out, "- {year}: {notice}");
    }
    let _ = writeln!(out, "\n{}", notices.rows_table(rows));
    let _ = writeln!(
        out,
        "The {} general civil days stand on their date in all {} years.\n",
        CIVIL.len(),
        NOTICES.len()
    );
    out.push_str(&notices.parts_list());
    let _ = writeln!(
        out,
        "\nThe days the notifications print are not reckoned: the four general days set by the \
         moon and, from 2020, the restricted Shab-e-Barat, Shab-e-Qadar and Jumat-ul-Vida, each \
         marked; and Varamahalakshmi Vrata, Rig Upakarma and Huttari, as printed. 2031, with no \
         notification, holds none.\n\n{}",
        printed_table(printed, "printed")
    );
}
