//! The `INDIA_KERALA` pack against the Government of Kerala's
//! notifications (`03-design/kerala-holidays.md`, SR-03), as
//! [`super::west_bengal`] holds West Bengal's, through the same comparison.
//!
//! Every rule of the pack, at Thiruvananthapuram on India's clock, against
//! each year's public holidays to the public offices, the occasions it
//! names as falling on a Sunday or a second Saturday, and its restricted
//! holidays; every parting is named with its cause, which the pass checks
//! both ways. The days the notifications print rather than reckon are held
//! only to stand in their year, the moon's marked, and a year with no
//! notification holds none.

use std::fmt::Write as _;

use teistro::FestivalPack;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};

use super::MonthDay;
use super::india::{Measured, Notices, Sky, printed_table};

/// Each year's notification of the General Administration
/// (Co-ordination) Department, and the copy read (rank 1).
const NOTICES: [(i32, &str); 6] = [
    (
        2021,
        "G.O.(P) No. 16/2020/GAD, 22 October 2020: rank 1, a staff site's text of the signed order",
    ),
    (
        2022,
        "G.O.(P) No. 29/2021/GAD, 21 October 2021, Kerala Gazette Extraordinary No. 3060: rank 1, gad.kerala.gov.in",
    ),
    (
        2023,
        "G.O.(P) No. 22/2022/GAD, 26 October 2022, Kerala Gazette Extraordinary No. 3657: rank 1, a copy of the signed gazette",
    ),
    (
        2024,
        "G.O.(P) No. 24/2023/GAD, 4 October 2023, Kerala Gazette Extraordinary No. 3254: rank 1, a copy of the signed gazette",
    ),
    (
        2025,
        "G.O.(P) No. 34/2024/GAD, 14 October 2024: rank 1, a copy of the signed gazette",
    ),
    (
        2026,
        "G.O.(P) No. 14/2025/GAD, 31 October 2025, Kerala Gazette Extraordinary No. 3845: rank 1, gad.kerala.gov.in",
    ),
];

/// Each year's reckoned days, those falling on a Sunday or a second
/// Saturday and the restricted Avani Avittom among them, less the days the
/// notifications print and the fixed days, which [`CIVIL`] holds in every
/// year. Sivarathri and Deepavali are the central list's rules, Deepavali
/// under its key `LAKSHMI_PUJA`.
const LISTED: [(i32, &str, MonthDay); 102] = [
    (2021, "SHIVARATRI", (3, 11)),
    (2021, "MAUNDY_THURSDAY", (4, 1)),
    (2021, "GOOD_FRIDAY", (4, 2)),
    (2021, "EASTER", (4, 4)),
    (2021, "VISHU", (4, 14)),
    (2021, "KARKIDAKA_VAVU", (8, 8)),
    (2021, "FIRST_ONAM", (8, 20)),
    (2021, "THIRUVONAM", (8, 21)),
    (2021, "AVANI_AVITTOM", (8, 22)),
    (2021, "THIRD_ONAM", (8, 22)),
    (2021, "FOURTH_ONAM", (8, 23)),
    (2021, "SREE_NARAYANA_GURU_JAYANTHI", (8, 23)),
    (2021, "JANMASHTAMI", (8, 30)),
    (2021, "SREE_NARAYANA_GURU_SAMADHI", (9, 21)),
    (2021, "MAHA_NAVAMI", (10, 14)),
    (2021, "VIJAYA_DASHAMI", (10, 15)),
    (2021, "LAKSHMI_PUJA", (11, 4)),
    (2022, "SHIVARATRI", (3, 1)),
    (2022, "MAUNDY_THURSDAY", (4, 14)),
    (2022, "GOOD_FRIDAY", (4, 15)),
    (2022, "VISHU", (4, 15)),
    (2022, "EASTER", (4, 17)),
    (2022, "KARKIDAKA_VAVU", (7, 28)),
    (2022, "AVANI_AVITTOM", (8, 11)),
    (2022, "JANMASHTAMI", (8, 18)),
    (2022, "FIRST_ONAM", (9, 7)),
    (2022, "THIRUVONAM", (9, 8)),
    (2022, "THIRD_ONAM", (9, 9)),
    (2022, "FOURTH_ONAM", (9, 10)),
    (2022, "SREE_NARAYANA_GURU_JAYANTHI", (9, 10)),
    (2022, "SREE_NARAYANA_GURU_SAMADHI", (9, 21)),
    (2022, "MAHA_NAVAMI", (10, 4)),
    (2022, "VIJAYA_DASHAMI", (10, 5)),
    (2022, "LAKSHMI_PUJA", (10, 24)),
    (2023, "SHIVARATRI", (2, 18)),
    (2023, "MAUNDY_THURSDAY", (4, 6)),
    (2023, "GOOD_FRIDAY", (4, 7)),
    (2023, "EASTER", (4, 9)),
    (2023, "VISHU", (4, 15)),
    (2023, "KARKIDAKA_VAVU", (7, 17)),
    (2023, "FIRST_ONAM", (8, 28)),
    (2023, "THIRUVONAM", (8, 29)),
    (2023, "AVANI_AVITTOM", (8, 30)),
    (2023, "THIRD_ONAM", (8, 30)),
    (2023, "FOURTH_ONAM", (8, 31)),
    (2023, "SREE_NARAYANA_GURU_JAYANTHI", (8, 31)),
    (2023, "JANMASHTAMI", (9, 6)),
    (2023, "SREE_NARAYANA_GURU_SAMADHI", (9, 22)),
    (2023, "MAHA_NAVAMI", (10, 23)),
    (2023, "VIJAYA_DASHAMI", (10, 24)),
    (2023, "LAKSHMI_PUJA", (11, 12)),
    (2024, "SHIVARATRI", (3, 8)),
    (2024, "MAUNDY_THURSDAY", (3, 28)),
    (2024, "GOOD_FRIDAY", (3, 29)),
    (2024, "EASTER", (3, 31)),
    (2024, "VISHU", (4, 14)),
    (2024, "KARKIDAKA_VAVU", (8, 3)),
    (2024, "AVANI_AVITTOM", (8, 19)),
    (2024, "SREE_NARAYANA_GURU_JAYANTHI", (8, 20)),
    (2024, "JANMASHTAMI", (8, 26)),
    (2024, "FIRST_ONAM", (9, 14)),
    (2024, "THIRUVONAM", (9, 15)),
    (2024, "THIRD_ONAM", (9, 16)),
    (2024, "FOURTH_ONAM", (9, 17)),
    (2024, "SREE_NARAYANA_GURU_SAMADHI", (9, 21)),
    (2024, "MAHA_NAVAMI", (10, 12)),
    (2024, "VIJAYA_DASHAMI", (10, 13)),
    (2024, "LAKSHMI_PUJA", (10, 31)),
    (2025, "SHIVARATRI", (2, 26)),
    (2025, "VISHU", (4, 14)),
    (2025, "MAUNDY_THURSDAY", (4, 17)),
    (2025, "GOOD_FRIDAY", (4, 18)),
    (2025, "EASTER", (4, 20)),
    (2025, "KARKIDAKA_VAVU", (7, 24)),
    (2025, "AVANI_AVITTOM", (8, 9)),
    (2025, "FIRST_ONAM", (9, 4)),
    (2025, "THIRUVONAM", (9, 5)),
    (2025, "THIRD_ONAM", (9, 6)),
    (2025, "FOURTH_ONAM", (9, 7)),
    (2025, "SREE_NARAYANA_GURU_JAYANTHI", (9, 7)),
    (2025, "JANMASHTAMI", (9, 14)),
    (2025, "SREE_NARAYANA_GURU_SAMADHI", (9, 21)),
    (2025, "MAHA_NAVAMI", (10, 1)),
    (2025, "VIJAYA_DASHAMI", (10, 2)),
    (2025, "LAKSHMI_PUJA", (10, 20)),
    (2026, "SHIVARATRI", (2, 15)),
    (2026, "MAUNDY_THURSDAY", (4, 2)),
    (2026, "GOOD_FRIDAY", (4, 3)),
    (2026, "EASTER", (4, 5)),
    (2026, "VISHU", (4, 15)),
    (2026, "KARKIDAKA_VAVU", (8, 12)),
    (2026, "FIRST_ONAM", (8, 25)),
    (2026, "THIRUVONAM", (8, 26)),
    (2026, "THIRD_ONAM", (8, 27)),
    (2026, "AVANI_AVITTOM", (8, 28)),
    (2026, "FOURTH_ONAM", (8, 28)),
    (2026, "SREE_NARAYANA_GURU_JAYANTHI", (8, 28)),
    (2026, "JANMASHTAMI", (9, 4)),
    (2026, "SREE_NARAYANA_GURU_SAMADHI", (9, 21)),
    (2026, "MAHA_NAVAMI", (10, 20)),
    (2026, "VIJAYA_DASHAMI", (10, 21)),
    (2026, "LAKSHMI_PUJA", (11, 8)),
];

/// The days every notification lists on one date, Vishwakarma Day as a
/// restricted holiday.
const CIVIL: [(&str, MonthDay); 9] = [
    ("MANNAM_JAYANTHI", (1, 2)),
    ("INDIA_REPUBLIC_DAY", (1, 26)),
    ("AMBEDKAR_JAYANTI", (4, 14)),
    ("INTERNATIONAL_LABOUR_DAY", (5, 1)),
    ("INDIA_INDEPENDENCE_DAY", (8, 15)),
    ("AYYANKALI_JAYANTHI", (8, 28)),
    ("VISHWAKARMA_PUJA", (9, 17)),
    ("GANDHI_JAYANTI", (10, 2)),
    ("CHRISTMAS", (12, 25)),
];

/// Why a rule parts from a notification, per (year, rule).
const PARTS: [(i32, &str, &str); 2] = [
    (
        2024,
        "KARKIDAKA_VAVU",
        "the new moon runs from 15:51 on 3 August to 16:43 on the 4th, so the sunrise reading takes the 4th; the notice lists the 3rd, the day the new moon holds the sunset (C1322)",
    ),
    (
        2025,
        "JANMASHTAMI",
        "*Dharmasindhu*'s rule keeps the 8th of the lunar month in August, which fell in Karkidakam; the notice lists 14 September, the dark 8th in Chingam (C1320)",
    ),
];

/// The days each year's notification prints rather than reckons: the four
/// set by the moon and Ayya Vaikunda Swami's birthday.
const PRINTED: [usize; 6] = [5; 6];

/// Thiruvananthapuram, the seat the notifications are issued from, as the
/// Malayalam calendar reckons there (`regional-solar.md`).
fn thiruvananthapuram() -> Place {
    Place::new(
        Latitude::literal(8.5241),
        Longitude::literal(76.9366),
        Altitude::literal(0.0),
    )
}

/// The state's list.
fn kerala() -> Notices {
    Notices {
        pack: FestivalPack::IndiaKerala,
        sky: Sky::Drik,
        place: thiruvananthapuram(),
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
    kerala().compare(problems)
}

/// §12 of the page.
pub(super) fn render(out: &mut String, (rows, printed): &Measured) {
    let notices = kerala();
    let parted = rows.iter().filter(|row| !row.agrees()).count();
    let _ = writeln!(
        out,
        "## 12. Kerala's public holidays against the notifications\n\n\
         `INDIA_KERALA` at Thiruvananthapuram on India's clock, against each year's public \
         holidays to the public offices of the General Administration (Co-ordination) \
         Department's notifications, the occasions they name as falling on a Sunday or a \
         second Saturday, and their restricted holidays. {} of {} rows agree.\n",
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
        "\nThe days the notifications print are not reckoned: the four set by the moon, each \
         marked, and Ayya Vaikunda Swami's birthday. 2031, with no notification, holds none.\n\n{}",
        printed_table(printed, "printed")
    );
}
