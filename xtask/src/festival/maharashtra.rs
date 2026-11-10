//! The `INDIA_MAHARASHTRA` pack against the Government of Maharashtra's
//! notifications (`03-design/maharashtra-holidays.md`, SR-03), as
//! [`super::india`] holds the central list to its memoranda.
//!
//! Every rule of the pack, at Mumbai on India's clock, against each year's
//! list; every parting is named with its cause, which the pass checks both
//! ways. The days the notifications print rather than reckon are held only
//! to stand in their year, the moon's marked, and a year with no
//! notification holds none.

use std::fmt::Write as _;

use teistro::FestivalPack;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};

use super::MonthDay;
use super::india::{Measured, Notices, Sky, printed_table};

/// Each year's notification of the General Administration Department,
/// Desk-29, and the rank of the copy read: the gazette page, or a copy
/// giving the notification's number and table (rank 1), or a press list
/// alone (rank 3, C1244).
const NOTICES: [(i32, &str); 7] = [
    (
        2020,
        "PHD. 1119/C.R.69/Desk-29, 7 December 2019: rank 1, a copy giving its number and table",
    ),
    (
        2021,
        "PHD. 1120/C.R.82/Desk-29, 1 December 2020: rank 3, a press list",
    ),
    (
        2022,
        "PHD. 1121/C.R.97/Desk-29, 2 December 2021, Bhaubeej by PHD. 1120/C.R.97/Desk-29 of the same day: rank 1, a copy giving its number and table",
    ),
    (
        2023,
        "PHD-1122/C.R.117/Desk-29, 2 December 2022: rank 1, the gazette page",
    ),
    (
        2024,
        "सार्वसु-1123/प्र.क्र.148/जपुक (कार्या-29), 9 November 2023, Gazette Extraordinary No. 118: rank 1, the Marathi gazette enclosed by the Municipal Corporation of Greater Mumbai",
    ),
    (
        2025,
        "of 4 December 2024, its number not seen: rank 3, a press list",
    ),
    (
        2026,
        "PHD-1125/C.R.199/Japuk (29), December 2025: rank 3, a press list",
    ),
];

/// Each year's reckoned days, less the days the notifications print and
/// the fixed days, which [`CIVIL`] holds in every year. Ram Navami 2025 is
/// not on the press list in hand; Bhaubeej is on the lists of 2022, 2023,
/// 2025 and 2026.
const LISTED: [(i32, &str, MonthDay); 87] = [
    (2020, "SHIVARATRI", (2, 21)),
    (2020, "HOLI", (3, 10)),
    (2020, "SAMVATSARA_ARAMBHA", (3, 25)),
    (2020, "RAMA_NAVAMI", (4, 2)),
    (2020, "MAHAVIR_JAYANTI", (4, 6)),
    (2020, "GOOD_FRIDAY", (4, 10)),
    (2020, "BUDDHA_JAYANTI", (5, 7)),
    (2020, "GANESHA_CHATURTHI", (8, 22)),
    (2020, "VIJAYA_DASHAMI", (10, 25)),
    (2020, "LAKSHMI_PUJA", (11, 14)),
    (2020, "BALI_PRATIPADA", (11, 16)),
    (2020, "GURU_NANAK_JAYANTI", (11, 30)),
    (2021, "SHIVARATRI", (3, 11)),
    (2021, "HOLI", (3, 29)),
    (2021, "GOOD_FRIDAY", (4, 2)),
    (2021, "SAMVATSARA_ARAMBHA", (4, 13)),
    (2021, "RAMA_NAVAMI", (4, 21)),
    (2021, "MAHAVIR_JAYANTI", (4, 25)),
    (2021, "BUDDHA_JAYANTI", (5, 26)),
    (2021, "GANESHA_CHATURTHI", (9, 10)),
    (2021, "VIJAYA_DASHAMI", (10, 15)),
    (2021, "LAKSHMI_PUJA", (11, 4)),
    (2021, "BALI_PRATIPADA", (11, 5)),
    (2021, "GURU_NANAK_JAYANTI", (11, 19)),
    (2022, "SHIVARATRI", (3, 1)),
    (2022, "HOLI", (3, 18)),
    (2022, "SAMVATSARA_ARAMBHA", (4, 2)),
    (2022, "RAMA_NAVAMI", (4, 10)),
    (2022, "MAHAVIR_JAYANTI", (4, 14)),
    (2022, "GOOD_FRIDAY", (4, 15)),
    (2022, "BUDDHA_JAYANTI", (5, 16)),
    (2022, "GANESHA_CHATURTHI", (8, 31)),
    (2022, "VIJAYA_DASHAMI", (10, 5)),
    (2022, "LAKSHMI_PUJA", (10, 24)),
    (2022, "BALI_PRATIPADA", (10, 26)),
    (2022, "YAMA_DWITIYA", (10, 26)),
    (2022, "GURU_NANAK_JAYANTI", (11, 8)),
    (2023, "SHIVARATRI", (2, 18)),
    (2023, "HOLI", (3, 7)),
    (2023, "SAMVATSARA_ARAMBHA", (3, 22)),
    (2023, "RAMA_NAVAMI", (3, 30)),
    (2023, "MAHAVIR_JAYANTI", (4, 4)),
    (2023, "GOOD_FRIDAY", (4, 7)),
    (2023, "BUDDHA_JAYANTI", (5, 5)),
    (2023, "GANESHA_CHATURTHI", (9, 19)),
    (2023, "VIJAYA_DASHAMI", (10, 24)),
    (2023, "LAKSHMI_PUJA", (11, 12)),
    (2023, "BALI_PRATIPADA", (11, 14)),
    (2023, "YAMA_DWITIYA", (11, 15)),
    (2023, "GURU_NANAK_JAYANTI", (11, 27)),
    (2024, "SHIVARATRI", (3, 8)),
    (2024, "HOLI", (3, 25)),
    (2024, "GOOD_FRIDAY", (3, 29)),
    (2024, "SAMVATSARA_ARAMBHA", (4, 9)),
    (2024, "RAMA_NAVAMI", (4, 17)),
    (2024, "MAHAVIR_JAYANTI", (4, 21)),
    (2024, "BUDDHA_JAYANTI", (5, 23)),
    (2024, "GANESHA_CHATURTHI", (9, 7)),
    (2024, "VIJAYA_DASHAMI", (10, 12)),
    (2024, "LAKSHMI_PUJA", (11, 1)),
    (2024, "BALI_PRATIPADA", (11, 2)),
    (2024, "GURU_NANAK_JAYANTI", (11, 15)),
    (2025, "SHIVARATRI", (2, 26)),
    (2025, "HOLI", (3, 14)),
    (2025, "SAMVATSARA_ARAMBHA", (3, 30)),
    (2025, "MAHAVIR_JAYANTI", (4, 10)),
    (2025, "GOOD_FRIDAY", (4, 18)),
    (2025, "BUDDHA_JAYANTI", (5, 12)),
    (2025, "GANESHA_CHATURTHI", (8, 27)),
    (2025, "VIJAYA_DASHAMI", (10, 2)),
    (2025, "LAKSHMI_PUJA", (10, 21)),
    (2025, "BALI_PRATIPADA", (10, 22)),
    (2025, "YAMA_DWITIYA", (10, 23)),
    (2025, "GURU_NANAK_JAYANTI", (11, 5)),
    (2026, "SHIVARATRI", (2, 15)),
    (2026, "HOLI", (3, 3)),
    (2026, "SAMVATSARA_ARAMBHA", (3, 19)),
    (2026, "RAMA_NAVAMI", (3, 26)),
    (2026, "MAHAVIR_JAYANTI", (3, 31)),
    (2026, "GOOD_FRIDAY", (4, 3)),
    (2026, "BUDDHA_JAYANTI", (5, 1)),
    (2026, "GANESHA_CHATURTHI", (9, 14)),
    (2026, "VIJAYA_DASHAMI", (10, 20)),
    (2026, "LAKSHMI_PUJA", (11, 8)),
    (2026, "BALI_PRATIPADA", (11, 10)),
    (2026, "YAMA_DWITIYA", (11, 11)),
    (2026, "GURU_NANAK_JAYANTI", (11, 24)),
];

/// The days every notification lists on one date.
const CIVIL: [(&str, MonthDay); 8] = [
    ("INDIA_REPUBLIC_DAY", (1, 26)),
    ("CHHATRAPATI_SHIVAJI_JAYANTI", (2, 19)),
    ("BANK_ACCOUNTS_CLOSING", (4, 1)),
    ("AMBEDKAR_JAYANTI", (4, 14)),
    ("MAHARASHTRA_DAY", (5, 1)),
    ("INDIA_INDEPENDENCE_DAY", (8, 15)),
    ("GANDHI_JAYANTI", (10, 2)),
    ("CHRISTMAS", (12, 25)),
];

/// Why a rule parts from a notification, per (year, rule).
const PARTS: [(i32, &str, &str); 5] = [
    (
        2020,
        "BALI_PRATIPADA",
        "the 1st runs from 10:37 on the 15th to 07:06 on the 16th at Mumbai, past a sunrise of 06:46; the rule takes the 15th, the notification the 16th, the day whose sunrise holds the 1st (C1241)",
    ),
    (
        2022,
        "VIJAYA_DASHAMI",
        "the 10th holds only the earlier aparahna and Shravana joins it on both days, so the rule stays on the earlier; the state, as the Department, prints the later, where Shravana stands in the aparahna (C855)",
    ),
    (
        2023,
        "MAHAVIR_JAYANTI",
        "the 13th begins at 06:24 on the 3rd, against a sunrise at Mumbai near 06:30, so both sunrises hold it and the rule takes the earlier; the state, as the Department, prints the 4th (C1242)",
    ),
    (
        2024,
        "LAKSHMI_PUJA",
        "the new moon ends at 18:17 on 1 November, 12 minutes past sunset at Mumbai, short of the ghati p. 77 asks, so the rule takes the earlier; the state the later, the day the rule gives at Delhi (C1240, C856)",
    ),
    (
        2025,
        "LAKSHMI_PUJA",
        "the new moon ends at 17:55 on 21 October, before sunset at Mumbai, so the rule takes the earlier; the state the later (C1240)",
    ),
];

/// Mumbai, the seat the notifications are issued from.
fn mumbai() -> Place {
    Place::new(
        Latitude::literal(18.9402),
        Longitude::literal(72.8347),
        Altitude::literal(14.0),
    )
}

/// The state's list.
fn maharashtra() -> Notices {
    Notices {
        pack: FestivalPack::IndiaMaharashtra,
        sky: Sky::Drik,
        place: mumbai(),
        years: &NOTICES,
        listed: &LISTED,
        civil: &CIVIL,
        parts: &PARTS,
        printed: &[5; 7],
        unnoticed: 2031,
    }
}

/// Every year's rows, and a problem for every parting no entry names.
pub(super) fn compare(problems: &mut Vec<String>) -> Result<Measured, String> {
    maharashtra().compare(problems)
}

/// §10 of the page.
pub(super) fn render(out: &mut String, (rows, printed): &Measured) {
    let notices = maharashtra();
    let parted = rows.iter().filter(|row| !row.agrees()).count();
    let _ = writeln!(
        out,
        "## 10. Maharashtra's public holidays against the notifications\n\n\
         `INDIA_MAHARASHTRA` at Mumbai on India's clock, against each year's list of the \
         General Administration Department's notification under s. 25 of the Negotiable \
         Instruments Act, with Bhaubeej from its own notification. {} of {} rows agree.\n",
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
        "\nThe state's list sides with the shipped rules against the central list twice: on \
         Holi in 2023 and 2026, where the Department prints a day later (C854), and on Diwali \
         in 2024, where the Department prints the earlier day (C856).\n\n\
         The days the notifications print are not reckoned: the four set by the moon, each \
         marked, and the Parsi New Year (Shahenshahi), whose calendar the catalogue lacks \
         (C1243); 2031, with no notification, holds none.\n\n{}",
        printed_table(printed, "printed")
    );
}
