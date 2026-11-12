//! The `INDIA_UTTAR_PRADESH` pack against the Government of Uttar
//! Pradesh's notifications (`03-design/uttar-pradesh-holidays.md`, SR-03),
//! as [`super::gujarat`] holds Gujarat's, through the same comparison.
//!
//! Every rule of the pack, at Lucknow on India's clock, against each
//! year's lists of public and restricted holidays; every parting is named
//! with its cause, which the pass checks both ways. The days the
//! notifications print rather than reckon are held only to stand in their
//! year, the moon's marked, and a year with no notification holds none.

use teistro::FestivalPack;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};

use super::MonthDay;
use super::india::{Measured, Notices, Section, Sky};

/// Each year's notification of the General Administration Section, and
/// the copy read.
const NOTICES: [(i32, &str); 8] = [
    (
        2019,
        "12/2018/846/तीन-2018-39(2)/2016, 27 November 2018: rank 1 as copied, the electronic issue on the state police's site",
    ),
    (
        2020,
        "680/तीन-2019-39(2)/2016, 4 November 2019: rank 1 as copied, the electronic issue on the state police's site",
    ),
    (
        2021,
        "19/2020/912/तीन-2020-39(2)/2016, 11 December 2020: rank 1 as copied, the electronic issue on the state police's site",
    ),
    (
        2022,
        "10/2021/514/तीन-2021-39(2)/2016, 20 October 2021: rank 1 as copied, the electronic issue as an HR firm republished it",
    ),
    (
        2023,
        "508/तीन-2022-39(2)/2016, 15 October 2022: rank 1 as copied, the signed scan",
    ),
    (
        2024,
        "528/तीन-2023-39(2)/2016, 4 December 2023: rank 1 as copied, the signed scan",
    ),
    (
        2025,
        "870/तीन-2024-39(2)/2016, 17 December 2024: rank 1 as copied, the signed scan",
    ),
    (
        2026,
        "671/तीन-2025-39(2)/2016, 17 November 2025: rank 1 as copied, the Uttar Pradesh Gazette, Extraordinary",
    ),
];

/// Each year's reckoned days, of both lists, less the days every
/// notification lists on one date, which [`CIVIL`] holds in every year.
const LISTED: [(i32, &str, MonthDay); 232] = [
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
const CIVIL: [(&str, MonthDay); 15] = [
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

/// Why a rule parts from a notification, per (year, rule).
const PARTS: [(i32, &str, &str); 13] = [
    (
        2019,
        "JANMASHTAMI",
        "the dark 8th runs from 08:09 on 23 August to 08:32 on the 24th; the central list's rule takes the 24th, whose sunrise it holds, and the notification lists the 23rd, whose midnight it holds (C1529)",
    ),
    (
        2019,
        "VIJAYA_DASHAMI",
        "the bright 10th runs from 12:38 on 7 October to 14:50 on the 8th, holding both afternoons, and Telangana's rule takes the 7th; the notification lists the 8th, whose sunrise it holds (C1529)",
    ),
    (
        2020,
        "ANANTA_CHATURDASHI",
        "the bright 14th runs from 08:49 on 31 August to 09:39 on 1 September; the print's rule takes the 1st, whose sunrise it holds, and the notification lists the 31st, the day it fills (C1529)",
    ),
    (
        2020,
        "MAHA_NAVAMI",
        "the bright 9th runs from 06:59 on 24 October to 07:42 on the 25th; the print's rule takes the 25th, whose sunrise it holds, and the notification lists the 24th, the day it fills (C1525)",
    ),
    (
        2022,
        "JANMASHTAMI",
        "the dark 8th runs from 21:22 on 18 August to 23:00 on the 19th; the central list's rule takes the 19th, whose sunrise it holds, and the notification lists the 18th, whose midnight it holds (C1529)",
    ),
    (
        2023,
        "GOVARDHAN_PUJA",
        "Kartika's bright 1st runs from 14:57 on 13 November to 14:37 on the 14th; Dharmasindhu's Bali pratipada takes the 14th, whose sunrise it holds, and the notification lists the 13th, the day it begins (C1529)",
    ),
    (
        2025,
        "BASANTA_PANCHAMI",
        "the bright 5th runs from 09:15 on 2 February to 06:53 on the 3rd, two minutes past the 3rd's sunrise; the print's rule takes the 3rd, and the notification lists the 2nd, the day it fills (C1529)",
    ),
    (
        2025,
        "NARAK_CHATURDASHI",
        "Deepawali falls on 20 October, and the rule counts the day before it; the notification lists Narak Chaturdashi on Deepawali's own day, whose sunrise the dark 14th holds until 15:45 (C1528)",
    ),
    (
        2025,
        "CHHATH",
        "the bright 6th runs from 06:05 on 27 October to 08:00 on the 28th, holding both sunrises; the print's rule keeps the earlier, and the notification lists the 28th, the dawn's offering (C1529)",
    ),
    (
        2026,
        "HOLI",
        "the full moon runs from 17:56 on 2 March to 17:08 on the 3rd, only the 2nd's evening holding it, so the Holika fire is the 2nd and Holi the 3rd; the notification lists Holi on the 4th, as the central list's notice does (C854, C1445)",
    ),
    (
        2026,
        "HOLI_SECOND_DAY",
        "Holi's second day follows the fire by two days, the 4th; the notification lists the 5th, the day after its Holi (C854)",
    ),
    (
        2026,
        "GOVARDHAN_PUJA",
        "Kartika's bright 1st runs from 12:32 on 9 November to 14:01 on the 10th; Dharmasindhu's Bali pratipada takes the 10th, whose sunrise it holds, and the notification lists the 9th, the day it begins (C1529)",
    ),
    (
        2026,
        "NARAK_CHATURDASHI",
        "Deepawali falls on 8 November, and the rule counts the day before it; the notification lists Narak Chaturdashi on Deepawali's own day, whose sunrise the dark 14th holds until 11:28 (C1528)",
    ),
];

/// The days each year's notification prints rather than reckons: the five
/// public days and the seven restricted ones the moon sets.
const PRINTED: [usize; 8] = [12; 8];

/// The page's section on the list.
const SECTION: Section = Section {
    heading: "20. Uttar Pradesh's public holidays against the notifications",
    against: "`INDIA_UTTAR_PRADESH` at Lucknow on India's clock, against each year's lists of \
         public and restricted holidays of the General Administration Section's notification. \
         The days are reckoned under the modern sky (the built-in ephemeris, the default \
         profile).",
    civil: "civil",
    printed: "The days the notifications print are not reckoned: the five public days and the \
         seven restricted ones the notes leave to the local sighting of the moon (Hazrat Ali's \
         birthday, Id-ul-Fitr, Id-uz-Zuha, Muharram and Id-e-Milad; the Urs of Khwaja \
         Moinuddin Chishti, Shab-e-Barat, Jamat-ul-Vida, the restricted days beside the two Ids \
         and Muharram, and Chehlum), each marked. 2031, with no notification, holds none.",
};

/// Lucknow, the seat the notifications are issued from.
fn lucknow() -> Place {
    Place::new(
        Latitude::literal(26.8467),
        Longitude::literal(80.9462),
        Altitude::literal(0.0),
    )
}

/// The state's list.
fn uttar_pradesh() -> Notices {
    Notices {
        pack: FestivalPack::IndiaUttarPradesh,
        sky: Sky::Drik,
        place: lucknow(),
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
    uttar_pradesh().compare(problems)
}

/// §20 of the page.
pub(super) fn render(out: &mut String, measured: &Measured) {
    uttar_pradesh().render(out, &SECTION, measured);
}
