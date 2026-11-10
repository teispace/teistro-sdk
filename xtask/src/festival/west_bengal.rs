//! The `INDIA_WEST_BENGAL` pack against the Government of West Bengal's
//! notifications (`03-design/west-bengal-holidays.md`, SR-03), as
//! [`super::maharashtra`] holds Maharashtra's, through the same comparison.
//!
//! Every rule of the pack, at Kolkata on India's clock, against each year's
//! Lists I to III and the note naming the occasions that fall on a Sunday;
//! every parting is named with its cause, which the pass checks both ways.
//! The days the notifications print rather than reckon are held only to
//! stand in their year, the moon's marked, and a year with no notification
//! holds none.

use std::fmt::Write as _;

use teistro::FestivalPack;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};

use super::MonthDay;
use super::india::{Measured, Notices, Sky, printed_table};

/// Each year's notification of the Finance (Audit) Department, and the
/// copy read: a staff site's copy giving each notification's number, its
/// lists and its notes (rank 1; the state's server refused a scripted
/// fetch, C1279).
const NOTICES: [(i32, &str); 8] = [
    (
        2019,
        "Nos. 6812-F(P2) to 6815-F(P2), 5 November 2018: rank 1, a copy giving their numbers and tables",
    ),
    (
        2020,
        "Nos. 5962-F(P2) to 5965-F(P2), 1 November 2019: rank 1, a copy giving their numbers and tables",
    ),
    (
        2021,
        "No. 3100-F(P2), 5 November 2020, Muharram moved by No. 2365-F: rank 1, a copy giving its number and tables",
    ),
    (
        2022,
        "No. 3640-F(P2), 26 November 2021: rank 1, a copy giving its number and tables",
    ),
    (
        2023,
        "No. 4331-F(P2), 21 October 2022, Mahavir Jayanti moved by No. 1430-F(P2) of 25 March 2023: rank 1, a copy giving its number and tables",
    ),
    (
        2024,
        "No. 6112-F(P2), 9 November 2023: rank 1, a copy giving its number and tables",
    ),
    (
        2025,
        "No. 4712-F(P2), 22 November 2024: rank 1, a copy giving its number and tables",
    ),
    (
        2026,
        "No. 4188-F(P2), 27 November 2025: rank 1, a copy giving its number and tables",
    ),
];

/// Each year's reckoned days, those of the Sunday note among them, less
/// the days the notifications print and the fixed days, which [`CIVIL`]
/// holds in every year. A row is here only in the years its notice lists
/// it: Rathayatra from 2021, Rakhi Bandhan from 2023, Ram Navami in 2025
/// and 2026, Viswakarma Puja in 2026, and Harichand Thakur's birthday from
/// 2023, the 2022 note being garbled (C1274). Mahavir Jayanti 2023 is the
/// day No. 1430-F(P2) declared.
const LISTED: [(i32, &str, MonthDay); 215] = [
    (2019, "BASANTA_PANCHAMI", (2, 10)),
    (2019, "SHIVARATRI", (3, 4)),
    (2019, "DOLA_YATRA", (3, 21)),
    (2019, "HOLI", (3, 22)),
    (2019, "POHELA_BOISHAKH", (4, 15)),
    (2019, "MAHAVIR_JAYANTI", (4, 17)),
    (2019, "GOOD_FRIDAY", (4, 19)),
    (2019, "EASTER_SATURDAY", (4, 20)),
    (2019, "RABINDRA_JAYANTI", (5, 9)),
    (2019, "BUDDHA_JAYANTI", (5, 18)),
    (2019, "JANMASHTAMI", (8, 23)),
    (2019, "AMAVASYA_SHRADDHA", (9, 28)),
    (2019, "DURGA_PANCHAMI", (10, 3)),
    (2019, "DURGA_SHASHTHI", (10, 4)),
    (2019, "PHULPATI", (10, 5)),
    (2019, "MAHA_ASHTAMI", (10, 6)),
    (2019, "MAHA_NAVAMI", (10, 7)),
    (2019, "VIJAYA_DASHAMI", (10, 8)),
    (2019, "KOJAGRAT_PURNIMA", (10, 13)),
    (2019, "KALI_PUJA", (10, 27)),
    (2019, "YAMA_DWITIYA", (10, 29)),
    (2019, "CHHATH", (11, 2)),
    (2019, "GURU_NANAK_JAYANTI", (11, 12)),
    (2020, "BASANTA_PANCHAMI", (1, 30)),
    (2020, "SHIVARATRI", (2, 21)),
    (2020, "DOLA_YATRA", (3, 9)),
    (2020, "HOLI", (3, 10)),
    (2020, "MAHAVIR_JAYANTI", (4, 6)),
    (2020, "GOOD_FRIDAY", (4, 10)),
    (2020, "EASTER_SATURDAY", (4, 11)),
    (2020, "POHELA_BOISHAKH", (4, 14)),
    (2020, "BUDDHA_JAYANTI", (5, 7)),
    (2020, "RABINDRA_JAYANTI", (5, 8)),
    (2020, "JANMASHTAMI", (8, 11)),
    (2020, "AMAVASYA_SHRADDHA", (9, 17)),
    (2020, "DURGA_CHATURTHI", (10, 20)),
    (2020, "DURGA_PANCHAMI", (10, 21)),
    (2020, "DURGA_SHASHTHI", (10, 22)),
    (2020, "PHULPATI", (10, 23)),
    (2020, "MAHA_ASHTAMI", (10, 24)),
    (2020, "MAHA_NAVAMI", (10, 25)),
    (2020, "VIJAYA_DASHAMI", (10, 26)),
    (2020, "KOJAGRAT_PURNIMA", (10, 30)),
    (2020, "KALI_PUJA", (11, 14)),
    (2020, "YAMA_DWITIYA", (11, 16)),
    (2020, "CHHATH", (11, 20)),
    (2020, "GURU_NANAK_JAYANTI", (11, 30)),
    (2021, "BASANTA_PANCHAMI", (2, 16)),
    (2021, "SHIVARATRI", (3, 11)),
    (2021, "DOLA_YATRA", (3, 28)),
    (2021, "HOLI", (3, 29)),
    (2021, "GOOD_FRIDAY", (4, 2)),
    (2021, "EASTER_SATURDAY", (4, 3)),
    (2021, "POHELA_BOISHAKH", (4, 15)),
    (2021, "MAHAVIR_JAYANTI", (4, 25)),
    (2021, "RABINDRA_JAYANTI", (5, 9)),
    (2021, "BUDDHA_JAYANTI", (5, 26)),
    (2021, "JAGANNATH_RATH_JATRA", (7, 12)),
    (2021, "JANMASHTAMI", (8, 30)),
    (2021, "AMAVASYA_SHRADDHA", (10, 6)),
    (2021, "DURGA_SHASHTHI", (10, 11)),
    (2021, "PHULPATI", (10, 12)),
    (2021, "MAHA_ASHTAMI", (10, 13)),
    (2021, "MAHA_NAVAMI", (10, 14)),
    (2021, "VIJAYA_DASHAMI", (10, 15)),
    (2021, "KOJAGRAT_PURNIMA", (10, 20)),
    (2021, "KALI_PUJA", (11, 4)),
    (2021, "YAMA_DWITIYA", (11, 6)),
    (2021, "CHHATH", (11, 10)),
    (2021, "GURU_NANAK_JAYANTI", (11, 19)),
    (2022, "BASANTA_PANCHAMI", (2, 5)),
    (2022, "THAKUR_PANCHANAN_BARMA_JAYANTI", (2, 14)),
    (2022, "SHIVARATRI", (3, 1)),
    (2022, "DOLA_YATRA", (3, 18)),
    (2022, "HOLI", (3, 19)),
    (2022, "MAHAVIR_JAYANTI", (4, 14)),
    (2022, "GOOD_FRIDAY", (4, 15)),
    (2022, "POHELA_BOISHAKH", (4, 15)),
    (2022, "EASTER_SATURDAY", (4, 16)),
    (2022, "RABINDRA_JAYANTI", (5, 9)),
    (2022, "BUDDHA_JAYANTI", (5, 16)),
    (2022, "JAGANNATH_RATH_JATRA", (7, 1)),
    (2022, "JANMASHTAMI", (8, 19)),
    (2022, "AMAVASYA_SHRADDHA", (9, 25)),
    (2022, "DURGA_PANCHAMI", (9, 30)),
    (2022, "DURGA_SHASHTHI", (10, 1)),
    (2022, "PHULPATI", (10, 2)),
    (2022, "MAHA_ASHTAMI", (10, 3)),
    (2022, "MAHA_NAVAMI", (10, 4)),
    (2022, "VIJAYA_DASHAMI", (10, 5)),
    (2022, "KOJAGRAT_PURNIMA", (10, 9)),
    (2022, "KALI_PUJA", (10, 24)),
    (2022, "YAMA_DWITIYA", (10, 27)),
    (2022, "CHHATH", (10, 30)),
    (2022, "GURU_NANAK_JAYANTI", (11, 8)),
    (2022, "BIRSA_MUNDA_JAYANTI", (11, 15)),
    (2023, "BASANTA_PANCHAMI", (1, 26)),
    (2023, "THAKUR_PANCHANAN_BARMA_JAYANTI", (2, 14)),
    (2023, "SHIVARATRI", (2, 18)),
    (2023, "DOLA_YATRA", (3, 7)),
    (2023, "HOLI", (3, 8)),
    (2023, "HARICHAND_THAKUR_JAYANTI", (3, 19)),
    (2023, "MAHAVIR_JAYANTI", (4, 3)),
    (2023, "GOOD_FRIDAY", (4, 7)),
    (2023, "EASTER_SATURDAY", (4, 8)),
    (2023, "POHELA_BOISHAKH", (4, 15)),
    (2023, "BUDDHA_JAYANTI", (5, 5)),
    (2023, "RABINDRA_JAYANTI", (5, 9)),
    (2023, "JAGANNATH_RATH_JATRA", (6, 20)),
    (2023, "RAKSHABANDHAN", (8, 30)),
    (2023, "JANMASHTAMI", (9, 6)),
    (2023, "AMAVASYA_SHRADDHA", (10, 14)),
    (2023, "DURGA_CHATURTHI", (10, 18)),
    (2023, "DURGA_PANCHAMI", (10, 19)),
    (2023, "DURGA_SHASHTHI", (10, 20)),
    (2023, "PHULPATI", (10, 21)),
    (2023, "MAHA_ASHTAMI", (10, 22)),
    (2023, "MAHA_NAVAMI", (10, 23)),
    (2023, "VIJAYA_DASHAMI", (10, 24)),
    (2023, "KOJAGRAT_PURNIMA", (10, 28)),
    (2023, "KALI_PUJA", (11, 12)),
    (2023, "BIRSA_MUNDA_JAYANTI", (11, 15)),
    (2023, "YAMA_DWITIYA", (11, 15)),
    (2023, "CHHATH", (11, 19)),
    (2023, "GURU_NANAK_JAYANTI", (11, 27)),
    (2024, "BASANTA_PANCHAMI", (2, 14)),
    (2024, "THAKUR_PANCHANAN_BARMA_JAYANTI", (2, 14)),
    (2024, "SHIVARATRI", (3, 8)),
    (2024, "DOLA_YATRA", (3, 25)),
    (2024, "HOLI", (3, 26)),
    (2024, "GOOD_FRIDAY", (3, 29)),
    (2024, "EASTER_SATURDAY", (3, 30)),
    (2024, "HARICHAND_THAKUR_JAYANTI", (4, 6)),
    (2024, "POHELA_BOISHAKH", (4, 14)),
    (2024, "MAHAVIR_JAYANTI", (4, 21)),
    (2024, "RABINDRA_JAYANTI", (5, 8)),
    (2024, "BUDDHA_JAYANTI", (5, 23)),
    (2024, "JAGANNATH_RATH_JATRA", (7, 7)),
    (2024, "RAKSHABANDHAN", (8, 19)),
    (2024, "JANMASHTAMI", (8, 26)),
    (2024, "AMAVASYA_SHRADDHA", (10, 2)),
    (2024, "DURGA_CHATURTHI", (10, 7)),
    (2024, "DURGA_PANCHAMI", (10, 8)),
    (2024, "DURGA_SHASHTHI", (10, 9)),
    (2024, "PHULPATI", (10, 10)),
    (2024, "MAHA_ASHTAMI", (10, 11)),
    (2024, "MAHA_NAVAMI", (10, 11)),
    (2024, "VIJAYA_DASHAMI", (10, 12)),
    (2024, "KOJAGRAT_PURNIMA", (10, 16)),
    (2024, "KALI_PUJA", (10, 31)),
    (2024, "YAMA_DWITIYA", (11, 3)),
    (2024, "CHHATH", (11, 7)),
    (2024, "BIRSA_MUNDA_JAYANTI", (11, 15)),
    (2024, "GURU_NANAK_JAYANTI", (11, 15)),
    (2025, "BASANTA_PANCHAMI", (2, 2)),
    (2025, "THAKUR_PANCHANAN_BARMA_JAYANTI", (2, 14)),
    (2025, "SHIVARATRI", (2, 26)),
    (2025, "DOLA_YATRA", (3, 14)),
    (2025, "HOLI", (3, 15)),
    (2025, "HARICHAND_THAKUR_JAYANTI", (3, 27)),
    (2025, "RAMA_NAVAMI", (4, 6)),
    (2025, "MAHAVIR_JAYANTI", (4, 10)),
    (2025, "POHELA_BOISHAKH", (4, 15)),
    (2025, "GOOD_FRIDAY", (4, 18)),
    (2025, "EASTER_SATURDAY", (4, 19)),
    (2025, "RABINDRA_JAYANTI", (5, 9)),
    (2025, "BUDDHA_JAYANTI", (5, 12)),
    (2025, "JAGANNATH_RATH_JATRA", (6, 27)),
    (2025, "RAKSHABANDHAN", (8, 9)),
    (2025, "JANMASHTAMI", (8, 15)),
    (2025, "AMAVASYA_SHRADDHA", (9, 21)),
    (2025, "DURGA_CHATURTHI", (9, 26)),
    (2025, "DURGA_PANCHAMI", (9, 27)),
    (2025, "DURGA_SHASHTHI", (9, 28)),
    (2025, "PHULPATI", (9, 29)),
    (2025, "MAHA_ASHTAMI", (9, 30)),
    (2025, "MAHA_NAVAMI", (10, 1)),
    (2025, "VIJAYA_DASHAMI", (10, 2)),
    (2025, "KOJAGRAT_PURNIMA", (10, 6)),
    (2025, "KALI_PUJA", (10, 20)),
    (2025, "YAMA_DWITIYA", (10, 23)),
    (2025, "CHHATH", (10, 27)),
    (2025, "GURU_NANAK_JAYANTI", (11, 5)),
    (2025, "BIRSA_MUNDA_JAYANTI", (11, 15)),
    (2026, "BASANTA_PANCHAMI", (1, 23)),
    (2026, "THAKUR_PANCHANAN_BARMA_JAYANTI", (2, 14)),
    (2026, "SHIVARATRI", (2, 15)),
    (2026, "DOLA_YATRA", (3, 3)),
    (2026, "HOLI", (3, 4)),
    (2026, "HARICHAND_THAKUR_JAYANTI", (3, 17)),
    (2026, "RAMA_NAVAMI", (3, 26)),
    (2026, "MAHAVIR_JAYANTI", (3, 31)),
    (2026, "GOOD_FRIDAY", (4, 3)),
    (2026, "EASTER_SATURDAY", (4, 4)),
    (2026, "POHELA_BOISHAKH", (4, 15)),
    (2026, "BUDDHA_JAYANTI", (5, 1)),
    (2026, "RABINDRA_JAYANTI", (5, 9)),
    (2026, "JAGANNATH_RATH_JATRA", (7, 16)),
    (2026, "RAKSHABANDHAN", (8, 28)),
    (2026, "JANMASHTAMI", (9, 4)),
    (2026, "VISHWAKARMA_PUJA", (9, 17)),
    (2026, "AMAVASYA_SHRADDHA", (10, 10)),
    (2026, "DURGA_CHATURTHI", (10, 15)),
    (2026, "DURGA_PANCHAMI", (10, 16)),
    (2026, "DURGA_SHASHTHI", (10, 17)),
    (2026, "PHULPATI", (10, 18)),
    (2026, "MAHA_ASHTAMI", (10, 19)),
    (2026, "MAHA_NAVAMI", (10, 20)),
    (2026, "VIJAYA_DASHAMI", (10, 21)),
    (2026, "KOJAGRAT_PURNIMA", (10, 25)),
    (2026, "KALI_PUJA", (11, 8)),
    (2026, "YAMA_DWITIYA", (11, 11)),
    (2026, "BIRSA_MUNDA_JAYANTI", (11, 15)),
    (2026, "CHHATH", (11, 15)),
    (2026, "GURU_NANAK_JAYANTI", (11, 24)),
];

/// The days every notification lists on one date.
const CIVIL: [(&str, MonthDay); 12] = [
    ("NEW_YEAR_S_DAY_AD", (1, 1)),
    ("VIVEKANANDA_JAYANTI", (1, 12)),
    ("NETAJI_JAYANTI", (1, 23)),
    ("INDIA_REPUBLIC_DAY", (1, 26)),
    ("BANK_ACCOUNTS_CLOSING", (4, 1)),
    ("AMBEDKAR_JAYANTI", (4, 14)),
    ("INTERNATIONAL_LABOUR_DAY", (5, 1)),
    ("HUL_DIWAS", (6, 30)),
    ("BHANU_JAYANTI", (7, 13)),
    ("INDIA_INDEPENDENCE_DAY", (8, 15)),
    ("GANDHI_JAYANTI", (10, 2)),
    ("CHRISTMAS", (12, 25)),
];

/// Why a rule parts from a notification, per (year, rule).
const PARTS: [(i32, &str, &str); 9] = [
    (
        2021,
        "KOJAGRAT_PURNIMA",
        "the full moon holds the earlier nishitha alone, so *Dharmasindhu* p. 74's rule takes the 19th; the notice lists the 20th, and no one window meets both 2020 and 2021 (C1271)",
    ),
    (
        2022,
        "JANMASHTAMI",
        "the rule takes the 18th by the 8th at nishitha; the notice lists the 19th (C1275)",
    ),
    (
        2023,
        "MAHAVIR_JAYANTI",
        "the rule takes the 4th at Kolkata, the day the notice first listed; No. 1430-F(P2) declared the 3rd, the day the 13th began (C1277, C1242)",
    ),
    (
        2024,
        "HARICHAND_THAKUR_JAYANTI",
        "the 13th runs from 10:19 on 6 April to 06:54 on the 7th, past a sunrise of 05:23, so the sunrise reading takes the 7th; the notice lists the 6th (C1274)",
    ),
    (
        2024,
        "MAHA_NAVAMI",
        "the 9th runs from 12:07 on 11 October to 10:59 on the 12th, so the sunrise reading takes the 12th; the notice lists Maha Astami and Maha Nabami together on the 11th (C1270)",
    ),
    (
        2024,
        "VIJAYA_DASHAMI",
        "counted from the Navami's 12th, the rule takes the 13th; the 10th runs from 10:59 on 12 October to 09:09 on the 13th, and the notice lists the 12th (C1270)",
    ),
    (
        2025,
        "BASANTA_PANCHAMI",
        "the 5th runs from 09:14 on 2 February to 06:52 on the 3rd, so the sunrise reading takes the 3rd; the note lists Sree Panchami on Sunday the 2nd (C1272)",
    ),
    (
        2025,
        "JANMASHTAMI",
        "the 8th begins at 23:50 on 15 August, after Kolkata's nishitha, so the rule takes the 16th; the notice lists the 15th (C1275)",
    ),
    (
        2025,
        "CHHATH",
        "the 6th begins at 06:05 on 27 October, after sunrise, and ends at 08:00 on the 28th, so the sunrise reading takes the 28th; the notice lists the 27th, the evening arghya's day (C1273)",
    ),
];

/// The days each year's notification prints rather than reckons: the five
/// set by the moon, and the days added around a festival and the sectional
/// days of a year or a few.
const PRINTED: [usize; 8] = [15, 14, 12, 16, 16, 17, 19, 19];

/// Kolkata, the seat the notifications are issued for.
fn kolkata() -> Place {
    Place::new(
        Latitude::literal(22.5726),
        Longitude::literal(88.3639),
        Altitude::literal(9.0),
    )
}

/// The state's list.
fn west_bengal() -> Notices {
    Notices {
        pack: FestivalPack::IndiaWestBengal,
        sky: Sky::Drik,
        place: kolkata(),
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
    west_bengal().compare(problems)
}

/// §11 of the page.
pub(super) fn render(out: &mut String, (rows, printed): &Measured) {
    let notices = west_bengal();
    let parted = rows.iter().filter(|row| !row.agrees()).count();
    let _ = writeln!(
        out,
        "## 11. West Bengal's public holidays against the notifications\n\n\
         `INDIA_WEST_BENGAL` at Kolkata on India's clock, against each year's Lists I to III of \
         the Finance (Audit) Department's notifications and the note naming the occasions that \
         fall on a Sunday. {} of {} rows agree.\n",
        rows.len() - parted,
        rows.len()
    );
    for (year, notice) in NOTICES {
        let _ = writeln!(out, "- {year}: {notice}");
    }
    let _ = writeln!(out, "\n{}", notices.rows_table(rows));
    let _ = writeln!(
        out,
        "The {} civil days stand on their date in all {} years.\n",
        CIVIL.len(),
        NOTICES.len()
    );
    out.push_str(&notices.parts_list());
    let _ = writeln!(
        out,
        "\nThe Durga Puja's Chaturthi, Panchami and Shashthi are counted back from the Saptami, \
         and stand in every year listed, where the tithis would part in 2026 (the 6th holds both \
         sunrises). Dol, Kali Puja and Holi as the day after Dol stand in all eight years.\n\n\
         The days the notifications print are not reckoned: the five set by the moon, each \
         marked but Muharram 2021, which an order moved; the days added around a festival; and \
         the sectional days of a year or a few. 2031, with no notification, holds none.\n\n{}",
        printed_table(printed, "printed")
    );
}
