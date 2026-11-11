//! The `INDIA_ASSAM` pack against the Government of Assam's notifications
//! (`03-design/assam-holidays.md`, SR-03), as [`super::kerala`] holds
//! Kerala's, through the same comparison.
//!
//! Every rule of the pack, at Guwahati on India's clock, against each
//! year's closed days and the two half holidays, Muharram and Lakshmi
//! Puja; every parting is named with its cause, which the pass checks both
//! ways. The days the notifications print rather than reckon are held only
//! to stand in their year, marked, and a year with no notification holds
//! none.

use std::fmt::Write as _;

use teistro::FestivalPack;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};

use super::MonthDay;
use super::india::{Measured, Notices, Sky, printed_table};

/// Each year's notification of the General Administration Department,
/// and the copy read (rank 1).
const NOTICES: [(i32, &str); 8] = [
    (
        2019,
        "No. GAG(A).81/2016/Pt/I/220, 19 December 2018, Assam Gazette Extraordinary No. 629: rank 1, the gazette's scan",
    ),
    (
        2020,
        "No. GAG(A)81/2016/494, 29 November 2019, Assam Gazette Extraordinary No. 486: rank 1, the gazette's scan",
    ),
    (
        2021,
        "No. GAG(A).81/2016/637, 18 December 2020, Assam Gazette Extraordinary No. 566: rank 1, the gazette's scan",
    ),
    (
        2022,
        "No. GAG(A).81/2016/749, 28 October 2021, with its corrigenda of 30 December 2021 and 12 January 2022: rank 1, the signed notification's scan",
    ),
    (
        2023,
        "e-file No. 221561/1, 28 November 2022: rank 1, the signed notification's scan",
    ),
    (
        2024,
        "e-file No. 221561/11, November 2023: rank 1 as copied, a copy of the e-signed notification",
    ),
    (
        2025,
        "No. E-221561/347, 6 December 2024, with its corrigendum of 27 December 2024: rank 1, the e-signed notification",
    ),
    (
        2026,
        "e-file No. 221561/440, 17 November 2025: rank 1, the e-signed notification",
    ),
];

/// Each year's reckoned days, Lakshmi Puja's half holiday among them,
/// less the days the notifications print and the fixed days, which
/// [`CIVIL`] holds in every year. Sati Sadhani Divas, Karam Puja and
/// Bhatri Dwitiya are listed from 2021.
const LISTED: [(i32, &str, MonthDay); 218] = [
    (2019, "MAGH_BIHU", (1, 15)),
    (2019, "MAGH_BIHU_SECOND", (1, 16)),
    (2019, "GWTHAR_BATHOU_SAN", (1, 29)),
    (2019, "BIR_CHILARAI_DIVAS", (2, 19)),
    (2019, "DOLA_YATRA", (3, 21)),
    (2019, "BOHAG_BIHU", (4, 14)),
    (2019, "BOHAG_BIHU_SECOND", (4, 15)),
    (2019, "BOHAG_BIHU_THIRD", (4, 16)),
    (2019, "GOOD_FRIDAY", (4, 19)),
    (2019, "DAMODARDEVA_TITHI", (5, 5)),
    (2019, "BUDDHA_JAYANTI", (5, 18)),
    (2019, "MADHABDEVA_JANMOTSAV", (5, 19)),
    (2019, "MADHABDEVA_TIROBHAV", (8, 20)),
    (2019, "JANMASHTAMI", (8, 24)),
    (2019, "SANKARDEVA_TIROBHAV", (9, 1)),
    (2019, "PHULPATI", (10, 5)),
    (2019, "MAHA_ASHTAMI", (10, 6)),
    (2019, "MAHA_NAVAMI", (10, 7)),
    (2019, "SANKARDEVA_JANMOTSAV", (10, 8)),
    (2019, "VIJAYA_DASHAMI", (10, 8)),
    (2019, "KOJAGRAT_PURNIMA", (10, 13)),
    (2019, "KATI_BIHU", (10, 18)),
    (2019, "KALI_PUJA", (10, 27)),
    (2019, "CHHATH", (11, 2)),
    (2019, "GURU_NANAK_JAYANTI", (11, 12)),
    (2020, "MAGH_BIHU", (1, 15)),
    (2020, "MAGH_BIHU_SECOND", (1, 16)),
    (2020, "GWTHAR_BATHOU_SAN", (1, 28)),
    (2020, "BIR_CHILARAI_DIVAS", (2, 9)),
    (2020, "DOLA_YATRA", (3, 9)),
    (2020, "GOOD_FRIDAY", (4, 10)),
    (2020, "BOHAG_BIHU", (4, 13)),
    (2020, "BOHAG_BIHU_SECOND", (4, 14)),
    (2020, "BOHAG_BIHU_THIRD", (4, 15)),
    (2020, "DAMODARDEVA_TITHI", (4, 23)),
    (2020, "BUDDHA_JAYANTI", (5, 7)),
    (2020, "MADHABDEVA_JANMOTSAV", (6, 6)),
    (2020, "SANKARDEVA_TIROBHAV", (8, 20)),
    (2020, "MADHABDEVA_TIROBHAV", (9, 7)),
    (2020, "JANMASHTAMI", (9, 9)),
    (2020, "SANKARDEVA_JANMOTSAV", (9, 26)),
    (2020, "KATI_BIHU", (10, 17)),
    (2020, "PHULPATI", (10, 23)),
    (2020, "MAHA_ASHTAMI", (10, 24)),
    (2020, "MAHA_NAVAMI", (10, 25)),
    (2020, "VIJAYA_DASHAMI", (10, 26)),
    (2020, "KOJAGRAT_PURNIMA", (10, 30)),
    (2020, "KALI_PUJA", (11, 14)),
    (2020, "CHHATH", (11, 20)),
    (2020, "GURU_NANAK_JAYANTI", (11, 30)),
    (2021, "MAGH_BIHU", (1, 14)),
    (2021, "MAGH_BIHU_SECOND", (1, 15)),
    (2021, "GWTHAR_BATHOU_SAN", (1, 26)),
    (2021, "BIR_CHILARAI_DIVAS", (2, 27)),
    (2021, "DOLA_YATRA", (3, 28)),
    (2021, "GOOD_FRIDAY", (4, 2)),
    (2021, "BOHAG_BIHU", (4, 14)),
    (2021, "BOHAG_BIHU_SECOND", (4, 15)),
    (2021, "BOHAG_BIHU_THIRD", (4, 16)),
    (2021, "SATI_SADHANI_DIVAS", (4, 21)),
    (2021, "DAMODARDEVA_TITHI", (5, 12)),
    (2021, "BUDDHA_JAYANTI", (5, 26)),
    (2021, "MADHABDEVA_JANMOTSAV", (5, 27)),
    (2021, "MADHABDEVA_TIROBHAV", (8, 27)),
    (2021, "KARAM_PUJA", (8, 29)),
    (2021, "JANMASHTAMI", (8, 30)),
    (2021, "SANKARDEVA_TIROBHAV", (9, 8)),
    (2021, "PHULPATI", (10, 12)),
    (2021, "MAHA_ASHTAMI", (10, 13)),
    (2021, "MAHA_NAVAMI", (10, 14)),
    (2021, "SANKARDEVA_JANMOTSAV", (10, 15)),
    (2021, "VIJAYA_DASHAMI", (10, 15)),
    (2021, "KATI_BIHU", (10, 18)),
    (2021, "KOJAGRAT_PURNIMA", (10, 20)),
    (2021, "KALI_PUJA", (11, 4)),
    (2021, "YAMA_DWITIYA", (11, 6)),
    (2021, "CHHATH", (11, 10)),
    (2021, "GURU_NANAK_JAYANTI", (11, 19)),
    (2022, "MAGH_BIHU", (1, 14)),
    (2022, "MAGH_BIHU_SECOND", (1, 15)),
    (2022, "GWTHAR_BATHOU_SAN", (1, 25)),
    (2022, "BIR_CHILARAI_DIVAS", (2, 16)),
    (2022, "DOLA_YATRA", (3, 18)),
    (2022, "BOHAG_BIHU", (4, 14)),
    (2022, "BOHAG_BIHU_SECOND", (4, 15)),
    (2022, "GOOD_FRIDAY", (4, 15)),
    (2022, "BOHAG_BIHU_THIRD", (4, 16)),
    (2022, "SATI_SADHANI_DIVAS", (4, 21)),
    (2022, "DAMODARDEVA_TITHI", (5, 1)),
    (2022, "BUDDHA_JAYANTI", (5, 16)),
    (2022, "MADHABDEVA_JANMOTSAV", (5, 17)),
    (2022, "JANMASHTAMI", (8, 19)),
    (2022, "SANKARDEVA_TIROBHAV", (8, 29)),
    (2022, "KARAM_PUJA", (9, 6)),
    (2022, "MADHABDEVA_TIROBHAV", (9, 14)),
    (2022, "PHULPATI", (10, 2)),
    (2022, "MAHA_ASHTAMI", (10, 3)),
    (2022, "MAHA_NAVAMI", (10, 4)),
    (2022, "SANKARDEVA_JANMOTSAV", (10, 5)),
    (2022, "VIJAYA_DASHAMI", (10, 5)),
    (2022, "KOJAGRAT_PURNIMA", (10, 9)),
    (2022, "KATI_BIHU", (10, 18)),
    (2022, "KALI_PUJA", (10, 24)),
    (2022, "YAMA_DWITIYA", (10, 27)),
    (2022, "CHHATH", (10, 30)),
    (2022, "GURU_NANAK_JAYANTI", (11, 8)),
    (2023, "MAGH_BIHU", (1, 15)),
    (2023, "MAGH_BIHU_SECOND", (1, 16)),
    (2023, "GWTHAR_BATHOU_SAN", (1, 24)),
    (2023, "BIR_CHILARAI_DIVAS", (2, 5)),
    (2023, "DOLA_YATRA", (3, 7)),
    (2023, "GOOD_FRIDAY", (4, 7)),
    (2023, "BOHAG_BIHU", (4, 14)),
    (2023, "BOHAG_BIHU_SECOND", (4, 15)),
    (2023, "BOHAG_BIHU_THIRD", (4, 16)),
    (2023, "DAMODARDEVA_TITHI", (4, 20)),
    (2023, "SATI_SADHANI_DIVAS", (4, 21)),
    (2023, "BUDDHA_JAYANTI", (5, 5)),
    (2023, "MADHABDEVA_JANMOTSAV", (6, 5)),
    (2023, "SANKARDEVA_TIROBHAV", (8, 18)),
    (2023, "KARAM_PUJA", (8, 27)),
    (2023, "MADHABDEVA_TIROBHAV", (9, 4)),
    (2023, "JANMASHTAMI", (9, 6)),
    (2023, "SANKARDEVA_JANMOTSAV", (9, 25)),
    (2023, "KATI_BIHU", (10, 18)),
    (2023, "PHULPATI", (10, 21)),
    (2023, "MAHA_ASHTAMI", (10, 22)),
    (2023, "MAHA_NAVAMI", (10, 23)),
    (2023, "VIJAYA_DASHAMI", (10, 24)),
    (2023, "KOJAGRAT_PURNIMA", (10, 28)),
    (2023, "KALI_PUJA", (11, 12)),
    (2023, "YAMA_DWITIYA", (11, 15)),
    (2023, "CHHATH", (11, 19)),
    (2023, "GURU_NANAK_JAYANTI", (11, 27)),
    (2024, "MAGH_BIHU", (1, 15)),
    (2024, "MAGH_BIHU_SECOND", (1, 16)),
    (2024, "GWTHAR_BATHOU_SAN", (1, 23)),
    (2024, "BIR_CHILARAI_DIVAS", (2, 24)),
    (2024, "DOLA_YATRA", (3, 25)),
    (2024, "GOOD_FRIDAY", (3, 29)),
    (2024, "BOHAG_BIHU", (4, 13)),
    (2024, "BOHAG_BIHU_SECOND", (4, 14)),
    (2024, "BOHAG_BIHU_THIRD", (4, 15)),
    (2024, "SATI_SADHANI_DIVAS", (4, 20)),
    (2024, "DAMODARDEVA_TITHI", (5, 8)),
    (2024, "BUDDHA_JAYANTI", (5, 23)),
    (2024, "MADHABDEVA_JANMOTSAV", (5, 24)),
    (2024, "MADHABDEVA_TIROBHAV", (8, 23)),
    (2024, "JANMASHTAMI", (8, 26)),
    (2024, "SANKARDEVA_TIROBHAV", (9, 4)),
    (2024, "KARAM_PUJA", (9, 14)),
    (2024, "PHULPATI", (10, 10)),
    (2024, "MAHA_ASHTAMI", (10, 11)),
    (2024, "MAHA_NAVAMI", (10, 12)),
    (2024, "SANKARDEVA_JANMOTSAV", (10, 13)),
    (2024, "VIJAYA_DASHAMI", (10, 13)),
    (2024, "KOJAGRAT_PURNIMA", (10, 16)),
    (2024, "KATI_BIHU", (10, 17)),
    (2024, "KALI_PUJA", (10, 31)),
    (2024, "YAMA_DWITIYA", (11, 3)),
    (2024, "CHHATH", (11, 7)),
    (2024, "GURU_NANAK_JAYANTI", (11, 15)),
    (2025, "MAGH_BIHU", (1, 14)),
    (2025, "MAGH_BIHU_SECOND", (1, 15)),
    (2025, "GWTHAR_BATHOU_SAN", (1, 28)),
    (2025, "BIR_CHILARAI_DIVAS", (2, 12)),
    (2025, "DOLA_YATRA", (3, 14)),
    (2025, "BOHAG_BIHU", (4, 14)),
    (2025, "BOHAG_BIHU_SECOND", (4, 15)),
    (2025, "BOHAG_BIHU_THIRD", (4, 16)),
    (2025, "GOOD_FRIDAY", (4, 18)),
    (2025, "SATI_SADHANI_DIVAS", (4, 21)),
    (2025, "DAMODARDEVA_TITHI", (4, 28)),
    (2025, "BUDDHA_JAYANTI", (5, 12)),
    (2025, "MADHABDEVA_JANMOTSAV", (6, 12)),
    (2025, "SANKARDEVA_TIROBHAV", (8, 25)),
    (2025, "KARAM_PUJA", (9, 3)),
    (2025, "MADHABDEVA_TIROBHAV", (9, 12)),
    (2025, "JANMASHTAMI", (9, 14)),
    (2025, "PHULPATI", (9, 29)),
    (2025, "MAHA_ASHTAMI", (9, 30)),
    (2025, "MAHA_NAVAMI", (10, 1)),
    (2025, "SANKARDEVA_JANMOTSAV", (10, 2)),
    (2025, "VIJAYA_DASHAMI", (10, 2)),
    (2025, "KOJAGRAT_PURNIMA", (10, 6)),
    (2025, "KATI_BIHU", (10, 18)),
    (2025, "KALI_PUJA", (10, 20)),
    (2025, "YAMA_DWITIYA", (10, 23)),
    (2025, "CHHATH", (10, 28)),
    (2025, "GURU_NANAK_JAYANTI", (11, 5)),
    (2026, "MAGH_BIHU", (1, 14)),
    (2026, "MAGH_BIHU_SECOND", (1, 15)),
    (2026, "GWTHAR_BATHOU_SAN", (1, 27)),
    (2026, "BIR_CHILARAI_DIVAS", (2, 1)),
    (2026, "DOLA_YATRA", (3, 3)),
    (2026, "GOOD_FRIDAY", (4, 3)),
    (2026, "BOHAG_BIHU", (4, 14)),
    (2026, "BOHAG_BIHU_SECOND", (4, 15)),
    (2026, "BOHAG_BIHU_THIRD", (4, 16)),
    (2026, "DAMODARDEVA_TITHI", (4, 18)),
    (2026, "SATI_SADHANI_DIVAS", (4, 21)),
    (2026, "BUDDHA_JAYANTI", (5, 1)),
    (2026, "MADHABDEVA_JANMOTSAV", (6, 1)),
    (2026, "MADHABDEVA_TIROBHAV", (9, 1)),
    (2026, "JANMASHTAMI", (9, 4)),
    (2026, "SANKARDEVA_TIROBHAV", (9, 12)),
    (2026, "SANKARDEVA_JANMOTSAV", (9, 21)),
    (2026, "KARAM_PUJA", (9, 22)),
    (2026, "KATI_BIHU", (10, 18)),
    (2026, "PHULPATI", (10, 18)),
    (2026, "MAHA_ASHTAMI", (10, 19)),
    (2026, "MAHA_NAVAMI", (10, 20)),
    (2026, "VIJAYA_DASHAMI", (10, 21)),
    (2026, "KOJAGRAT_PURNIMA", (10, 25)),
    (2026, "KALI_PUJA", (11, 8)),
    (2026, "YAMA_DWITIYA", (11, 11)),
    (2026, "CHHATH", (11, 15)),
    (2026, "GURU_NANAK_JAYANTI", (11, 24)),
];

/// The days every notification lists on one date, the annual closing of
/// bank accounts among them.
const CIVIL: [(&str, MonthDay); 10] = [
    ("NETAJI_JAYANTI", (1, 23)),
    ("INDIA_REPUBLIC_DAY", (1, 26)),
    ("ME_DAM_ME_PHI", (1, 31)),
    ("BANK_ACCOUNTS_CLOSING", (4, 1)),
    ("INTERNATIONAL_LABOUR_DAY", (5, 1)),
    ("INDIA_INDEPENDENCE_DAY", (8, 15)),
    ("GANDHI_JAYANTI", (10, 2)),
    ("LACHIT_DIVAS", (11, 24)),
    ("ASOM_DIVAS", (12, 2)),
    ("CHRISTMAS", (12, 25)),
];

/// Why a rule parts from a notification, per (year, rule).
const PARTS: [(i32, &str, &str); 7] = [
    (
        2019,
        "SANKARDEVA_TIROBHAV",
        "the bright 2nd runs from 12:13 on 31 August to 08:26 on 1 September, 5 h 31 min of the 31st's daylight and 3 h 24 min of the 1st's, so the daylight reading takes the 31st; the notification lists the 1st, whose sunrise the tithi holds (C1364)",
    ),
    (
        2020,
        "JANMASHTAMI",
        "the dark 8th begins at 02:06 on 10 September, so its sunrise is the 10th's; the notification lists the 9th, which the ashtami holds at neither sunrise nor midnight (C1363)",
    ),
    (
        2021,
        "KARAM_PUJA",
        "Bhadrapada's bright 11th holds the sunrise of 17 September; the notification lists 29 August, amanta Shravana's dark 7th (C1365)",
    ),
    (
        2021,
        "KOJAGRAT_PURNIMA",
        "the full moon runs from 19:03 on 19 October to 20:26 on the 20th, and the rule keeps the night of the 19th; the notification lists the 20th, whose sunrise the full moon holds (C1366)",
    ),
    (
        2023,
        "JANMASHTAMI",
        "the dark 8th begins at 15:38 on 6 September, so its sunrise is the 7th's; the notification lists the 6th, whose midnight the ashtami holds (C1363)",
    ),
    (
        2023,
        "KARAM_PUJA",
        "Bhadrapada's bright 11th holds the sunrise of 25 September; the notification lists 27 August, the bright 11th of nija Shravana and of Bhada (C1365)",
    ),
    (
        2023,
        "SANKARDEVA_TIROBHAV",
        "the notification lists 18 August, Saon 32, whose daylight the bright 2nd holds before Bhada begins; the rule, held to Bhada, takes Bhada's bright 2nd on 16 September (C1364)",
    ),
];

/// The days each year's notification prints rather than reckons: the two
/// Ids, and Muharram's half holiday to 2024.
const PRINTED: [usize; 8] = [3, 3, 3, 3, 3, 3, 2, 2];

/// Guwahati, the seat the notifications are issued from and the place
/// the Assamese month is reckoned at (`regional-solar.md`).
fn guwahati() -> Place {
    Place::new(
        Latitude::literal(26.1445),
        Longitude::literal(91.7362),
        Altitude::literal(0.0),
    )
}

/// The state's list.
fn assam() -> Notices {
    Notices {
        pack: FestivalPack::IndiaAssam,
        sky: Sky::Drik,
        place: guwahati(),
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
    assam().compare(problems)
}

/// §14 of the page.
pub(super) fn render(out: &mut String, (rows, printed): &Measured) {
    let notices = assam();
    let parted = rows.iter().filter(|row| !row.agrees()).count();
    let _ = writeln!(
        out,
        "## 14. Assam's public holidays against the notifications\n\n\
         `INDIA_ASSAM` at Guwahati on India's clock, against each year's closed days and the \
         half holiday of Lakshmi Puja of the General Administration Department's \
         notifications. The lunar days are reckoned under the modern sky (the built-in \
         ephemeris, the default profile) and the Assamese months under the Surya \
         Siddhanta's Sun (`ASSAM_SOLAR`, C1300). {} of {} rows agree.\n",
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
        "\nThe days the notifications print are not reckoned: Id-ul-Fitr, Id-ul-Zuha and \
         Muharram's half holiday, each marked. 2031, with no notification, holds none.\n\n{}",
        printed_table(printed, "printed")
    );
}
