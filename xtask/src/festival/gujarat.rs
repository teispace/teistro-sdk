//! The `INDIA_GUJARAT` pack against the Government of Gujarat's
//! notifications (`03-design/gujarat-holidays.md`, SR-03), as
//! [`super::telangana`] holds Telangana's, through the same comparison.
//!
//! Every rule of the pack, at Gandhinagar on India's clock, against each
//! year's Part I public holidays and its Note (1) of the days on a Sunday
//! or a second or fourth Saturday; every parting is named with its cause,
//! which the pass checks both ways. The days the notifications print
//! rather than reckon are held only to stand in their year, the moon's
//! marked, and a year with no notification holds none.

use teistro::FestivalPack;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};

use super::MonthDay;
use super::india::{Measured, Notices, Section, Sky};

/// Each year's notification of the General Administration Department,
/// and the copy read.
const NOTICES: [(i32, &str); 8] = [
    (
        2019,
        "GS-32/2018-PHD/102018/1411/GH, 3 November 2018: rank 1, the Government Central Press print",
    ),
    (
        2020,
        "GS/30/2019/PHD/102019/1111/GH, 19 October 2019: rank 1 as copied, a staff site's copy of its text",
    ),
    (
        2021,
        "GS/33/2020/PHD/102020/500/GH, 28 October 2020: rank 1 as copied, the signed scan",
    ),
    (
        2022,
        "GS/20/2021/PHD/102021/431/GH, 24 November 2021: rank 1 as copied, the signed scan",
    ),
    (
        2023,
        "GS/16/2022/PHD/102022/603/GH, 5 November 2022: rank 1 as copied, the signed scan",
    ),
    (
        2024,
        "GS/27/2023/JSR/102023/930/GH, 20 November 2023: rank 1 as copied, the signed scan",
    ),
    (
        2025,
        "GS/26/2024/JSR/2024/605/GH, 14 November 2024: rank 1 as copied, the signed scan",
    ),
    (
        2026,
        "GS/22/2025/JSR/2025/509/GH, 27 November 2025: rank 1, the Gujarat Government Gazette, Extraordinary No. 160",
    ),
];

/// Each year's reckoned days, Part I's and Note (1)'s, less the days every
/// notification lists on one date, which [`CIVIL`] holds in every year.
const LISTED: [(i32, &str, MonthDay); 120] = [
    (2019, "SHIVARATRI", (3, 4)),
    (2019, "HOLI", (3, 21)),
    (2019, "CHETI_CHAND", (4, 6)),
    (2019, "RAMA_NAVAMI", (4, 14)),
    (2019, "MAHAVIR_JAYANTI", (4, 17)),
    (2019, "GOOD_FRIDAY", (4, 19)),
    (2019, "PARASHURAMA_JAYANTI", (5, 7)),
    (2019, "RAKSHABANDHAN", (8, 15)),
    (2019, "JANMASHTAMI", (8, 24)),
    (2019, "SAMVATSARI", (9, 2)),
    (2019, "VIJAYA_DASHAMI", (10, 8)),
    (2019, "LAKSHMI_PUJA", (10, 27)),
    (2019, "VIKRAM_SAMVAT_NEW_YEAR", (10, 28)),
    (2019, "YAMA_DWITIYA", (10, 29)),
    (2019, "GURU_NANAK_JAYANTI", (11, 12)),
    (2020, "SHIVARATRI", (2, 21)),
    (2020, "HOLI", (3, 10)),
    (2020, "CHETI_CHAND", (3, 25)),
    (2020, "RAMA_NAVAMI", (4, 2)),
    (2020, "MAHAVIR_JAYANTI", (4, 6)),
    (2020, "GOOD_FRIDAY", (4, 10)),
    (2020, "PARASHURAMA_JAYANTI", (4, 25)),
    (2020, "RAKSHABANDHAN", (8, 3)),
    (2020, "JANMASHTAMI", (8, 12)),
    (2020, "SAMVATSARI", (8, 22)),
    (2020, "VIJAYA_DASHAMI", (10, 25)),
    (2020, "LAKSHMI_PUJA", (11, 14)),
    (2020, "VIKRAM_SAMVAT_NEW_YEAR", (11, 16)),
    (2020, "YAMA_DWITIYA", (11, 16)),
    (2020, "GURU_NANAK_JAYANTI", (11, 30)),
    (2021, "SHIVARATRI", (3, 11)),
    (2021, "HOLI", (3, 29)),
    (2021, "CHETI_CHAND", (4, 13)),
    (2021, "RAMA_NAVAMI", (4, 21)),
    (2021, "MAHAVIR_JAYANTI", (4, 25)),
    (2021, "GOOD_FRIDAY", (4, 2)),
    (2021, "PARASHURAMA_JAYANTI", (5, 14)),
    (2021, "RAKSHABANDHAN", (8, 22)),
    (2021, "JANMASHTAMI", (8, 30)),
    (2021, "SAMVATSARI", (9, 10)),
    (2021, "VIJAYA_DASHAMI", (10, 15)),
    (2021, "LAKSHMI_PUJA", (11, 4)),
    (2021, "VIKRAM_SAMVAT_NEW_YEAR", (11, 5)),
    (2021, "YAMA_DWITIYA", (11, 6)),
    (2021, "GURU_NANAK_JAYANTI", (11, 19)),
    (2022, "SHIVARATRI", (3, 1)),
    (2022, "HOLI", (3, 18)),
    (2022, "CHETI_CHAND", (4, 2)),
    (2022, "RAMA_NAVAMI", (4, 10)),
    (2022, "MAHAVIR_JAYANTI", (4, 14)),
    (2022, "GOOD_FRIDAY", (4, 15)),
    (2022, "PARASHURAMA_JAYANTI", (5, 3)),
    (2022, "RAKSHABANDHAN", (8, 11)),
    (2022, "JANMASHTAMI", (8, 19)),
    (2022, "SAMVATSARI", (8, 31)),
    (2022, "VIJAYA_DASHAMI", (10, 5)),
    (2022, "LAKSHMI_PUJA", (10, 24)),
    (2022, "VIKRAM_SAMVAT_NEW_YEAR", (10, 26)),
    (2022, "YAMA_DWITIYA", (10, 26)),
    (2022, "GURU_NANAK_JAYANTI", (11, 8)),
    (2023, "SHIVARATRI", (2, 18)),
    (2023, "HOLI", (3, 8)),
    (2023, "CHETI_CHAND", (3, 22)),
    (2023, "RAMA_NAVAMI", (3, 30)),
    (2023, "MAHAVIR_JAYANTI", (4, 4)),
    (2023, "GOOD_FRIDAY", (4, 7)),
    (2023, "PARASHURAMA_JAYANTI", (4, 22)),
    (2023, "RAKSHABANDHAN", (8, 30)),
    (2023, "JANMASHTAMI", (9, 7)),
    (2023, "SAMVATSARI", (9, 19)),
    (2023, "VIJAYA_DASHAMI", (10, 24)),
    (2023, "LAKSHMI_PUJA", (11, 12)),
    (2023, "VIKRAM_SAMVAT_NEW_YEAR", (11, 14)),
    (2023, "YAMA_DWITIYA", (11, 15)),
    (2023, "GURU_NANAK_JAYANTI", (11, 27)),
    (2024, "SHIVARATRI", (3, 8)),
    (2024, "HOLI", (3, 25)),
    (2024, "CHETI_CHAND", (4, 10)),
    (2024, "RAMA_NAVAMI", (4, 17)),
    (2024, "MAHAVIR_JAYANTI", (4, 21)),
    (2024, "GOOD_FRIDAY", (3, 29)),
    (2024, "PARASHURAMA_JAYANTI", (5, 10)),
    (2024, "RAKSHABANDHAN", (8, 19)),
    (2024, "JANMASHTAMI", (8, 26)),
    (2024, "SAMVATSARI", (9, 7)),
    (2024, "VIJAYA_DASHAMI", (10, 12)),
    (2024, "LAKSHMI_PUJA", (10, 31)),
    (2024, "VIKRAM_SAMVAT_NEW_YEAR", (11, 2)),
    (2024, "YAMA_DWITIYA", (11, 3)),
    (2024, "GURU_NANAK_JAYANTI", (11, 15)),
    (2025, "SHIVARATRI", (2, 26)),
    (2025, "HOLI", (3, 14)),
    (2025, "CHETI_CHAND", (3, 30)),
    (2025, "RAMA_NAVAMI", (4, 6)),
    (2025, "MAHAVIR_JAYANTI", (4, 10)),
    (2025, "GOOD_FRIDAY", (4, 18)),
    (2025, "PARASHURAMA_JAYANTI", (4, 29)),
    (2025, "RAKSHABANDHAN", (8, 9)),
    (2025, "JANMASHTAMI", (8, 16)),
    (2025, "SAMVATSARI", (8, 27)),
    (2025, "VIJAYA_DASHAMI", (10, 2)),
    (2025, "LAKSHMI_PUJA", (10, 20)),
    (2025, "VIKRAM_SAMVAT_NEW_YEAR", (10, 22)),
    (2025, "YAMA_DWITIYA", (10, 23)),
    (2025, "GURU_NANAK_JAYANTI", (11, 5)),
    (2026, "SHIVARATRI", (2, 15)),
    (2026, "HOLI", (3, 4)),
    (2026, "CHETI_CHAND", (3, 19)),
    (2026, "RAMA_NAVAMI", (3, 26)),
    (2026, "MAHAVIR_JAYANTI", (3, 31)),
    (2026, "GOOD_FRIDAY", (4, 3)),
    (2026, "PARASHURAMA_JAYANTI", (4, 19)),
    (2026, "RAKSHABANDHAN", (8, 28)),
    (2026, "JANMASHTAMI", (9, 4)),
    (2026, "SAMVATSARI", (9, 15)),
    (2026, "VIJAYA_DASHAMI", (10, 20)),
    (2026, "LAKSHMI_PUJA", (11, 8)),
    (2026, "VIKRAM_SAMVAT_NEW_YEAR", (11, 10)),
    (2026, "YAMA_DWITIYA", (11, 11)),
    (2026, "GURU_NANAK_JAYANTI", (11, 24)),
];

/// The days every notification lists on one date, in Part I or Note (1).
const CIVIL: [(&str, MonthDay); 7] = [
    ("UTTARAYANA_PUNYAKALA", (1, 14)),
    ("INDIA_REPUBLIC_DAY", (1, 26)),
    ("AMBEDKAR_JAYANTI", (4, 14)),
    ("INDIA_INDEPENDENCE_DAY", (8, 15)),
    ("GANDHI_JAYANTI", (10, 2)),
    ("SARDAR_PATEL_JAYANTI", (10, 31)),
    ("CHRISTMAS", (12, 25)),
];

/// Why a rule parts from a notification, per (year, rule).
const PARTS: [(i32, &str, &str); 7] = [
    (
        2019,
        "RAMA_NAVAMI",
        "the bright 9th runs from 11:41 on 13 April to 09:35 on the 14th, holding the 13th's midday, and the central list's rule takes the 13th; the notification lists the 14th, whose sunrise the navami holds, a Sunday named in Note (1) (C1443)",
    ),
    (
        2019,
        "VIJAYA_DASHAMI",
        "the bright 10th runs from 12:38 on 7 October to 14:50 on the 8th, holding both afternoons, and the central list's rule keeps the earlier, the 7th; the notification lists the 8th, whose sunrise the dashami holds (C1444)",
    ),
    (
        2022,
        "VIJAYA_DASHAMI",
        "the bright 10th runs from 14:21 on 4 October to 12:00 on the 5th, so only the 4th's afternoon holds it and the central list's rule takes the 4th; the notification lists the 5th, whose sunrise the dashami holds (C1444)",
    ),
    (
        2023,
        "HOLI",
        "the full moon runs from 16:17 on 6 March to 18:10 on the 7th, ending before the 7th's sunset (18:45), so the Holika fire is the 6th's evening and Holi the 7th; the notification lists Dhuleti on the 8th and Part II's Holi on the 7th, a day later, as the central list's notices do (C1445, C854)",
    ),
    (
        2023,
        "MAHAVIR_JAYANTI",
        "the bright 13th runs from 06:24 on 3 April to 08:05 on the 4th, holding both sunrises (06:30 and 06:29), and the print's rule keeps the earlier, the 3rd; the notification lists the 4th, as Maharashtra's does (C1447, C1242)",
    ),
    (
        2024,
        "CHETI_CHAND",
        "Chaitra's bright 1st runs from 23:50 on 8 April to 20:31 on the 9th, and the rule takes the 9th, whose sunrise it holds; the notification lists the 10th, whose sunrise the bright 2nd holds (C1446)",
    ),
    (
        2026,
        "HOLI",
        "the full moon runs from 17:56 on 2 March to 17:07 on the 3rd, ending before the 3rd's sunset, so the Holika fire is the 2nd's evening and Holi the 3rd; the notification lists Dhuleti on the 4th and Part II's Holi on the 3rd, a day later, as the central list's notices do (C1445, C854)",
    ),
];

/// The days each year's notification prints rather than reckons: the four
/// set by the moon and the Parsi New Year (Shahenshahi).
const PRINTED: [usize; 8] = [5; 8];

/// The page's section on the list.
const SECTION: Section = Section {
    heading: "17. Gujarat's public holidays against the notifications",
    against: "`INDIA_GUJARAT` at Gandhinagar on India's clock, against each year's Part I \
         public holidays of the General Administration Department's notification and its Note \
         (1) of the days on a Sunday or a second or fourth Saturday. The days are reckoned under \
         the modern sky (the built-in ephemeris, the default profile).",
    civil: "civil",
    printed: "The days the notifications print are not reckoned: the four set by the moon \
         (Ramjan-Eid, Bakri-Eid, Muharram and Eid-e-Meeladunnabi), each marked, and the Parsi New \
         Year (Shahenshahi), as printed. 2031, with no notification, holds none.",
};

/// Gandhinagar, the seat the notifications are issued from.
fn gandhinagar() -> Place {
    Place::new(
        Latitude::literal(23.2156),
        Longitude::literal(72.6369),
        Altitude::literal(0.0),
    )
}

/// The state's list.
fn gujarat() -> Notices {
    Notices {
        pack: FestivalPack::IndiaGujarat,
        sky: Sky::Drik,
        place: gandhinagar(),
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
    gujarat().compare(problems)
}

/// §17 of the page.
pub(super) fn render(out: &mut String, measured: &Measured) {
    gujarat().render(out, &SECTION, measured);
}
