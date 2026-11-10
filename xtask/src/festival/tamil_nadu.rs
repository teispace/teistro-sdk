//! The `INDIA_TAMIL_NADU` pack against the Government of Tamil Nadu's
//! orders (`03-design/tamil-nadu-holidays.md`, SR-03), as
//! [`super::kerala`] holds Kerala's, through the same comparison, under two
//! skies.
//!
//! Every rule of the pack, at Chennai on India's clock, against each
//! year's public holidays: under the `tamil-vakya` profile, the vakya
//! almanac's tithi and nakshatra, which the pack ships for (C1350), and
//! beside it under the modern sky, so the two readings stay measured. Every
//! parting under either sky is named with its cause, which the pass checks
//! both ways. The days the orders print rather than reckon are held only
//! to stand in their year, marked, and a year with no order holds none.

use std::fmt::Write as _;

use teistro::FestivalPack;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};

use super::MonthDay;
use super::india::{Measured, Notices, Sky, printed_table};

/// Each year's order of the Public (Miscellaneous) Department, and the
/// copy read (C1354).
const ORDERS: [(i32, &str); 8] = [
    (
        2019,
        "G.O.Ms. No. 840, 29 October 2018: rank 1 as copied, a staff site's copy with its number and table",
    ),
    (
        2020,
        "G.O.Ms. No. 762, 22 October 2019: rank 1 as copied, a staff site's copy with its number and table",
    ),
    (
        2021,
        "G.O.Ms. No. 554, 27 October 2020: rank 1 as copied; Thai Poosam added by a later order of January 2021",
    ),
    (
        2022,
        "G.O.(Ms.) No. 935, 2 November 2021, Gazette Extraordinary Part II(1) No. 525: rank 3, a list whose first rows match the gazette's",
    ),
    (
        2023,
        "G.O.(Ms.) No. 682, 11 October 2022, amended by G.O.(Ms.) No. 528, 31 August 2023: rank 1 as copied",
    ),
    (
        2024,
        "G.O.(Ms.) No. 692, 9 November 2023: rank 1 as copied, a staff site's copy with its number and table",
    ),
    (
        2025,
        "G.O.(Ms.) No. 792, 22 November 2024: rank 1, the signed order and its appendix",
    ),
    (
        2026,
        "G.O.(Ms.) No. 708, 11 November 2025: rank 1, the signed order and its appendix",
    ),
];

/// Each year's reckoned days, less the days the orders print and the
/// fixed days, which [`CIVIL`] holds in every year. Vinayakar Chathurthi
/// 2023 is the day G.O.(Ms.) No. 528 declared, 18 September, where the
/// order of 2022 printed the 17th. Thai Poosam is listed from 2021.
const LISTED: [(i32, &str, MonthDay); 102] = [
    (2019, "THAI_PONGAL", (1, 15)),
    (2019, "THIRUVALLUVAR_DAY", (1, 16)),
    (2019, "UZHAVAR_THIRUNAL", (1, 17)),
    (2019, "SAMVATSARA_ARAMBHA", (4, 6)),
    (2019, "PUTHANDU", (4, 14)),
    (2019, "MAHAVIR_JAYANTI", (4, 17)),
    (2019, "GOOD_FRIDAY", (4, 19)),
    (2019, "JANMASHTAMI", (8, 23)),
    (2019, "GANESHA_CHATURTHI", (9, 2)),
    (2019, "AYUDHA_PUJA", (10, 7)),
    (2019, "VIJAYA_DASHAMI", (10, 8)),
    (2019, "NARAK_CHATURDASHI", (10, 27)),
    (2020, "THAI_PONGAL", (1, 15)),
    (2020, "THIRUVALLUVAR_DAY", (1, 16)),
    (2020, "UZHAVAR_THIRUNAL", (1, 17)),
    (2020, "SAMVATSARA_ARAMBHA", (3, 25)),
    (2020, "MAHAVIR_JAYANTI", (4, 6)),
    (2020, "GOOD_FRIDAY", (4, 10)),
    (2020, "PUTHANDU", (4, 14)),
    (2020, "JANMASHTAMI", (8, 11)),
    (2020, "GANESHA_CHATURTHI", (8, 22)),
    (2020, "AYUDHA_PUJA", (10, 25)),
    (2020, "VIJAYA_DASHAMI", (10, 26)),
    (2020, "NARAK_CHATURDASHI", (11, 14)),
    (2021, "THAI_PONGAL", (1, 14)),
    (2021, "THIRUVALLUVAR_DAY", (1, 15)),
    (2021, "UZHAVAR_THIRUNAL", (1, 16)),
    (2021, "THAI_POOSAM", (1, 28)),
    (2021, "GOOD_FRIDAY", (4, 2)),
    (2021, "SAMVATSARA_ARAMBHA", (4, 13)),
    (2021, "PUTHANDU", (4, 14)),
    (2021, "MAHAVIR_JAYANTI", (4, 25)),
    (2021, "JANMASHTAMI", (8, 30)),
    (2021, "GANESHA_CHATURTHI", (9, 10)),
    (2021, "AYUDHA_PUJA", (10, 14)),
    (2021, "VIJAYA_DASHAMI", (10, 15)),
    (2021, "NARAK_CHATURDASHI", (11, 4)),
    (2022, "THAI_PONGAL", (1, 14)),
    (2022, "THIRUVALLUVAR_DAY", (1, 15)),
    (2022, "UZHAVAR_THIRUNAL", (1, 16)),
    (2022, "THAI_POOSAM", (1, 18)),
    (2022, "SAMVATSARA_ARAMBHA", (4, 2)),
    (2022, "MAHAVIR_JAYANTI", (4, 14)),
    (2022, "PUTHANDU", (4, 14)),
    (2022, "GOOD_FRIDAY", (4, 15)),
    (2022, "JANMASHTAMI", (8, 19)),
    (2022, "GANESHA_CHATURTHI", (8, 31)),
    (2022, "AYUDHA_PUJA", (10, 4)),
    (2022, "VIJAYA_DASHAMI", (10, 5)),
    (2022, "NARAK_CHATURDASHI", (10, 24)),
    (2023, "THAI_PONGAL", (1, 15)),
    (2023, "THIRUVALLUVAR_DAY", (1, 16)),
    (2023, "UZHAVAR_THIRUNAL", (1, 17)),
    (2023, "THAI_POOSAM", (2, 5)),
    (2023, "SAMVATSARA_ARAMBHA", (3, 22)),
    (2023, "MAHAVIR_JAYANTI", (4, 4)),
    (2023, "GOOD_FRIDAY", (4, 7)),
    (2023, "PUTHANDU", (4, 14)),
    (2023, "JANMASHTAMI", (9, 6)),
    (2023, "GANESHA_CHATURTHI", (9, 18)),
    (2023, "AYUDHA_PUJA", (10, 23)),
    (2023, "VIJAYA_DASHAMI", (10, 24)),
    (2023, "NARAK_CHATURDASHI", (11, 12)),
    (2024, "THAI_PONGAL", (1, 15)),
    (2024, "THIRUVALLUVAR_DAY", (1, 16)),
    (2024, "UZHAVAR_THIRUNAL", (1, 17)),
    (2024, "THAI_POOSAM", (1, 25)),
    (2024, "GOOD_FRIDAY", (3, 29)),
    (2024, "SAMVATSARA_ARAMBHA", (4, 9)),
    (2024, "PUTHANDU", (4, 14)),
    (2024, "MAHAVIR_JAYANTI", (4, 21)),
    (2024, "JANMASHTAMI", (8, 26)),
    (2024, "GANESHA_CHATURTHI", (9, 7)),
    (2024, "AYUDHA_PUJA", (10, 11)),
    (2024, "VIJAYA_DASHAMI", (10, 12)),
    (2024, "NARAK_CHATURDASHI", (10, 31)),
    (2025, "THAI_PONGAL", (1, 14)),
    (2025, "THIRUVALLUVAR_DAY", (1, 15)),
    (2025, "UZHAVAR_THIRUNAL", (1, 16)),
    (2025, "THAI_POOSAM", (2, 11)),
    (2025, "SAMVATSARA_ARAMBHA", (3, 30)),
    (2025, "MAHAVIR_JAYANTI", (4, 10)),
    (2025, "PUTHANDU", (4, 14)),
    (2025, "GOOD_FRIDAY", (4, 18)),
    (2025, "JANMASHTAMI", (8, 16)),
    (2025, "GANESHA_CHATURTHI", (8, 27)),
    (2025, "AYUDHA_PUJA", (10, 1)),
    (2025, "VIJAYA_DASHAMI", (10, 2)),
    (2025, "NARAK_CHATURDASHI", (10, 20)),
    (2026, "THAI_PONGAL", (1, 15)),
    (2026, "THIRUVALLUVAR_DAY", (1, 16)),
    (2026, "UZHAVAR_THIRUNAL", (1, 17)),
    (2026, "THAI_POOSAM", (2, 1)),
    (2026, "SAMVATSARA_ARAMBHA", (3, 19)),
    (2026, "MAHAVIR_JAYANTI", (3, 31)),
    (2026, "GOOD_FRIDAY", (4, 3)),
    (2026, "PUTHANDU", (4, 14)),
    (2026, "JANMASHTAMI", (9, 4)),
    (2026, "GANESHA_CHATURTHI", (9, 14)),
    (2026, "AYUDHA_PUJA", (10, 19)),
    (2026, "VIJAYA_DASHAMI", (10, 20)),
    (2026, "NARAK_CHATURDASHI", (11, 8)),
];

/// The days every order lists on one date, the banks' closing of accounts
/// among them.
const CIVIL: [(&str, MonthDay); 8] = [
    ("NEW_YEAR_S_DAY_AD", (1, 1)),
    ("INDIA_REPUBLIC_DAY", (1, 26)),
    ("BANK_ACCOUNTS_CLOSING", (4, 1)),
    ("AMBEDKAR_JAYANTI", (4, 14)),
    ("INTERNATIONAL_LABOUR_DAY", (5, 1)),
    ("INDIA_INDEPENDENCE_DAY", (8, 15)),
    ("GANDHI_JAYANTI", (10, 2)),
    ("CHRISTMAS", (12, 25)),
];

/// Why a rule parts from an order under the vakya sky, per (year, rule).
const PARTS: [(i32, &str, &str); 4] = [
    (
        2021,
        "NARAK_CHATURDASHI",
        "the vakya chaturdashi runs from 07:26 on 3 November to 05:41 on the 4th, holding neither sunrise, so the sunrise reading takes the 3rd; the order lists the 4th, whose arunodaya the chaturdashi holds (C1356)",
    ),
    (
        2024,
        "THAI_POOSAM",
        "Pushya runs from 09:09 on 25 January to 11:06 on the 26th, so its first sunrise is the 26th's; the order lists the 25th, the full moon's day, whose daylight Pushya holds the most of (C1352)",
    ),
    (
        2026,
        "AYUDHA_PUJA",
        "counted from Vijaya Dasami, which the sunrise reading puts on the 21st (C1357); the order lists the 19th, the day before its Vijaya Dasami (C1353)",
    ),
    (
        2026,
        "VIJAYA_DASHAMI",
        "the vakya dashami runs from 10:37 on 20 October to 11:47 on the 21st, so the sunrise reading takes the 21st; the order lists the 20th, whose aparahna the dashami holds (C1357)",
    ),
];

/// Why a rule parts from an order under the modern sky, per (year, rule).
const PARTS_DRIK: [(i32, &str, &str); 8] = [
    (
        2022,
        "JANMASHTAMI",
        "the drik ashtami begins at 21:21 on 18 August and holds its midnight; the vakya ashtami begins at 01:40 on the 19th, the order's day (C1355)",
    ),
    (
        2023,
        "GANESHA_CHATURTHI",
        "the drik chaturthi runs from 12:39 on 18 September to 13:43 on the 19th and holds the 19th's madhyahna; G.O.(Ms.) No. 528 declared the 18th, the vakya day (C1296)",
    ),
    (
        2024,
        "AYUDHA_PUJA",
        "counted from the drik Vijaya Dasami, 13 October; the order lists the 11th (C1353)",
    ),
    (
        2024,
        "THAI_POOSAM",
        "Pushya runs from 08:16 on 25 January to 10:28 on the 26th, so its first sunrise is the 26th's, under both skies (C1352)",
    ),
    (
        2024,
        "VIJAYA_DASHAMI",
        "the drik dashami runs from 10:59 on 12 October to 09:09 on the 13th, so the sunrise reading takes the 13th; the vakya dashami holds the 12th's sunrise (C1350)",
    ),
    (
        2025,
        "JANMASHTAMI",
        "the drik ashtami begins at 23:50 on 15 August and holds its midnight; the vakya ashtami begins at 01:28 on the 16th, the order's day (C1355)",
    ),
    (
        2026,
        "AYUDHA_PUJA",
        "counted from Vijaya Dasami, the 21st under both skies (C1357)",
    ),
    (
        2026,
        "VIJAYA_DASHAMI",
        "the drik dashami runs from 12:50 on 20 October to 14:12 on the 21st, so the sunrise reading takes the 21st, as under the vakya sky (C1357)",
    ),
];

/// The days each year's order prints rather than reckons: the four set by
/// the moon.
const PRINTED: [usize; 8] = [4; 8];

/// Chennai, the seat the orders are issued from and the vakya almanacs'
/// press (`calendars/vakya.md`, C1294).
fn chennai() -> Place {
    Place::new(
        Latitude::literal(13.0827),
        Longitude::literal(80.2707),
        Altitude::literal(0.0),
    )
}

/// The state's list under `sky`, with the partings named for it.
fn tamil_nadu(sky: Sky, parts: &'static [(i32, &'static str, &'static str)]) -> Notices {
    Notices {
        pack: FestivalPack::IndiaTamilNadu,
        sky,
        place: chennai(),
        years: &ORDERS,
        listed: &LISTED,
        civil: &CIVIL,
        parts,
        printed: &PRINTED,
        unnoticed: 2031,
    }
}

/// Every year's rows under the vakya sky and under the modern, and a
/// problem for every parting no entry names.
pub(super) fn compare(problems: &mut Vec<String>) -> Result<(Measured, Measured), String> {
    let vakya = tamil_nadu(Sky::Vakya, &PARTS).compare(problems)?;
    let drik = tamil_nadu(Sky::Drik, &PARTS_DRIK).compare(problems)?;
    Ok((vakya, drik))
}

/// §13 of the page.
pub(super) fn render(out: &mut String, ((rows, printed), (drik, _)): &(Measured, Measured)) {
    let vakya = tamil_nadu(Sky::Vakya, &PARTS);
    let modern = tamil_nadu(Sky::Drik, &PARTS_DRIK);
    let agree = |rows: &[super::india::Row]| rows.iter().filter(|row| row.agrees()).count();
    let _ = writeln!(
        out,
        "## 13. Tamil Nadu's public holidays against the orders\n\n\
         `INDIA_TAMIL_NADU` at Chennai on India's clock, against each year's public holidays \
         of the Public (Miscellaneous) Department's orders. The pack ships for the \
         `tamil-vakya` profile, the vakya almanac's tithi and nakshatra (C1350): {} of {} \
         rows agree there. Under the modern sky (the built-in ephemeris, the default \
         profile) {} of {} agree. The Tamil months are the vakya school's under both \
         (`TAMIL_VAKYA`, C1351).\n",
        agree(rows),
        rows.len(),
        agree(drik),
        drik.len()
    );
    for (year, order) in ORDERS {
        let _ = writeln!(out, "- {year}: {order}");
    }
    let _ = writeln!(
        out,
        "\n{}",
        vakya.rows_beside(rows, drik, ["vakya", "modern"])
    );
    let _ = writeln!(
        out,
        "The {} civil days stand on their date in all {} years, under both skies.\n\n\
         Under the vakya sky:\n",
        CIVIL.len(),
        ORDERS.len()
    );
    out.push_str(&vakya.parts_list());
    out.push_str("\nUnder the modern sky:\n\n");
    out.push_str(&modern.parts_list());
    let _ = writeln!(
        out,
        "\nThe days the orders print are not reckoned: the four set by the moon, each \
         marked. 2031, with no order, holds none.\n\n{}",
        printed_table(printed, "printed")
    );
}
