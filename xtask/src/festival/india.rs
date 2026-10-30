//! The `INDIA_CENTRAL` pack against the Government of India's notices
//! (`03-design/india-central-holidays.md`, SR-03), as [`super::nepal`] holds
//! Nepal's rules to the committee's print.
//!
//! Every rule of the pack, at Delhi on India's clock, against each year's
//! Annexure I; every parting is named with its cause, which the pass checks
//! both ways. The days set by the moon are not reckoned, so the pass holds
//! only that each stands in its year, marked unless the Department declared
//! it, and that a year with no notice holds none.

use std::fmt::Write as _;

use teistro::festival::{Observances, SubjectTo};
use teistro::{CalendarDate, Context, Ephemeris, FestivalPack, FestivalRequest, UtcOffset};

use super::{MonthDay, date, place};

/// The years a notice is in hand for, and where each was read. 2021 and
/// 2022 are scans; 2023 and 2027 have a text layer; 2024 to 2026 were read
/// off the page images. Each is the Department's own memorandum as an
/// office or a staff site republished it, whose Annexure I the row list
/// was checked against.
const NOTICES: [(i32, &str); 7] = [
    (2021, "F.No.12/9/2020-JCA-2, 10 June 2020"),
    (2022, "F.No.12/5/2021-JCA-2, 8 June 2021"),
    (2023, "F.No.12/5/2022-JCA, 16 June 2022"),
    (2024, "F.No.12/2/2023-JCA, 3 July 2023"),
    (2025, "F.No.12/2/2023-JCA, 9 July 2024"),
    (2026, "F.No.12/2/2023-JCA, 3 July 2025"),
    (2027, "F.No.12/2/2023-JCA, 16 July 2026"),
];

/// Each year's list, less the days set by the moon and the fixed national
/// days and Christmas, which [`CIVIL`] holds in every year.
const LISTED: [(i32, &str, MonthDay); 63] = [
    (2021, "HOLI", (3, 29)),
    (2021, "GOOD_FRIDAY", (4, 2)),
    (2021, "RAMA_NAVAMI", (4, 21)),
    (2021, "MAHAVIR_JAYANTI", (4, 25)),
    (2021, "BUDDHA_JAYANTI", (5, 26)),
    (2021, "JANMASHTAMI", (8, 30)),
    (2021, "VIJAYA_DASHAMI", (10, 15)),
    (2021, "LAKSHMI_PUJA", (11, 4)),
    (2021, "GURU_NANAK_JAYANTI", (11, 19)),
    (2022, "SHIVARATRI", (3, 1)),
    (2022, "HOLI", (3, 18)),
    (2022, "MAHAVIR_JAYANTI", (4, 14)),
    (2022, "GOOD_FRIDAY", (4, 15)),
    (2022, "BUDDHA_JAYANTI", (5, 16)),
    (2022, "JANMASHTAMI", (8, 19)),
    (2022, "VIJAYA_DASHAMI", (10, 5)),
    (2022, "LAKSHMI_PUJA", (10, 24)),
    (2022, "GURU_NANAK_JAYANTI", (11, 8)),
    (2023, "HOLI", (3, 8)),
    (2023, "RAMA_NAVAMI", (3, 30)),
    (2023, "MAHAVIR_JAYANTI", (4, 4)),
    (2023, "GOOD_FRIDAY", (4, 7)),
    (2023, "BUDDHA_JAYANTI", (5, 5)),
    (2023, "JANMASHTAMI", (9, 7)),
    (2023, "VIJAYA_DASHAMI", (10, 24)),
    (2023, "LAKSHMI_PUJA", (11, 12)),
    (2023, "GURU_NANAK_JAYANTI", (11, 27)),
    (2024, "HOLI", (3, 25)),
    (2024, "GOOD_FRIDAY", (3, 29)),
    (2024, "RAMA_NAVAMI", (4, 17)),
    (2024, "MAHAVIR_JAYANTI", (4, 21)),
    (2024, "BUDDHA_JAYANTI", (5, 23)),
    (2024, "JANMASHTAMI", (8, 26)),
    (2024, "VIJAYA_DASHAMI", (10, 12)),
    (2024, "LAKSHMI_PUJA", (10, 31)),
    (2024, "GURU_NANAK_JAYANTI", (11, 15)),
    (2025, "SHIVARATRI", (2, 26)),
    (2025, "HOLI", (3, 14)),
    (2025, "MAHAVIR_JAYANTI", (4, 10)),
    (2025, "GOOD_FRIDAY", (4, 18)),
    (2025, "BUDDHA_JAYANTI", (5, 12)),
    (2025, "JANMASHTAMI", (8, 16)),
    (2025, "VIJAYA_DASHAMI", (10, 2)),
    (2025, "LAKSHMI_PUJA", (10, 20)),
    (2025, "GURU_NANAK_JAYANTI", (11, 5)),
    (2026, "HOLI", (3, 4)),
    (2026, "RAMA_NAVAMI", (3, 26)),
    (2026, "MAHAVIR_JAYANTI", (3, 31)),
    (2026, "GOOD_FRIDAY", (4, 3)),
    (2026, "BUDDHA_JAYANTI", (5, 1)),
    (2026, "JANMASHTAMI", (9, 4)),
    (2026, "VIJAYA_DASHAMI", (10, 20)),
    (2026, "LAKSHMI_PUJA", (11, 8)),
    (2026, "GURU_NANAK_JAYANTI", (11, 24)),
    (2027, "HOLI", (3, 23)),
    (2027, "GOOD_FRIDAY", (3, 26)),
    (2027, "RAMA_NAVAMI", (4, 15)),
    (2027, "MAHAVIR_JAYANTI", (4, 19)),
    (2027, "BUDDHA_JAYANTI", (5, 20)),
    (2027, "JANMASHTAMI", (8, 25)),
    (2027, "VIJAYA_DASHAMI", (10, 9)),
    (2027, "LAKSHMI_PUJA", (10, 29)),
    (2027, "GURU_NANAK_JAYANTI", (11, 14)),
];

/// The days every notice lists on one date.
const CIVIL: [(&str, MonthDay); 4] = [
    ("INDIA_REPUBLIC_DAY", (1, 26)),
    ("INDIA_INDEPENDENCE_DAY", (8, 15)),
    ("GANDHI_JAYANTI", (10, 2)),
    ("CHRISTMAS", (12, 25)),
];

/// Why a rule parts from a notice, per (year, rule). The pass fails on a
/// parting with no entry, and on an entry whose row agrees.
const PARTS: [(i32, &str, &str); 5] = [
    (
        2022,
        "VIJAYA_DASHAMI",
        "the 10th holds only the earlier aparahna, and Shravana begins that night inside the 10th, so it joins both days and the rule stays on the earlier; the Department prints the later, where Shravana stands in the aparahna (C855)",
    ),
    (
        2023,
        "HOLI",
        "bhadra covers the earlier pradosha and the later has no full moon; the rule takes the earlier, the Department a Holi a day later (C854)",
    ),
    (
        2024,
        "LAKSHMI_PUJA",
        "the new moon lasts past a ghati after the later sunset, so the text takes the later; the Department, printing 16 months ahead, the earlier (C856)",
    ),
    (
        2026,
        "HOLI",
        "bhadra covers the earlier pradosha and the later has no full moon; the rule takes the earlier, the Department a Holi a day later (C854)",
    ),
    (
        2027,
        "HOLI",
        "bhadra covers the earlier pradosha and the later has no full moon; the rule takes the earlier, and Annexure II prints Holika Dahan on the later (C854)",
    ),
];

/// What the pass measured: the rows, and each day set by the moon as its
/// year, key, day and whether it stands marked.
pub(super) type Measured = (Vec<Row>, Vec<(i32, String, MonthDay, bool)>);

/// A year's outcome: one row of the page.
pub(super) struct Row {
    year: i32,
    rule: &'static str,
    listed: MonthDay,
    found: Option<CalendarDate>,
}

impl Row {
    fn agrees(&self) -> bool {
        self.found
            .as_ref()
            .is_some_and(|day| (day.month, day.day) == self.listed)
    }
}

/// The day the answer gives a rule, of whichever kind.
fn day_of(answer: &Observances, rule: &str) -> Option<CalendarDate> {
    answer
        .observances
        .iter()
        .find(|found| found.rule == rule)
        .map(|found| found.day.clone())
        .or_else(|| {
            answer
                .dated
                .iter()
                .find(|found| found.rule == rule)
                .map(|found| found.day.clone())
        })
}

/// Every year's rows, and a problem for every parting no entry names.
pub(super) fn compare(problems: &mut Vec<String>) -> Result<Measured, String> {
    let context = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .map_err(|e| e.to_string())?;
    let request = FestivalRequest::from(FestivalPack::IndiaCentral);
    let offset = UtcOffset::literal(5, 30, 0);
    let mut rows = Vec::new();
    let mut sighted = Vec::new();
    for (year, _) in NOTICES {
        let answer = context
            .almanac()
            .festivals(
                &date(year, (1, 1)),
                &date(year, (12, 31)),
                &place(),
                offset,
                &request,
            )
            .map_err(|e| e.to_string())?
            .value;
        let civil = CIVIL.iter().map(|&(rule, day)| (year, rule, day));
        let lists = LISTED.iter().copied().filter(|row| row.0 == year);
        for (year, rule, listed) in lists.chain(civil) {
            let row = Row {
                year,
                rule,
                listed,
                found: day_of(&answer, rule),
            };
            let excused = PARTS.iter().any(|part| part.0 == year && part.1 == rule);
            if !row.agrees() && !excused {
                problems.push(format!(
                    "{year} {rule}: the notice lists {listed:?}, the rule gives {:?}",
                    row.found.as_ref().map(|day| (day.month, day.day))
                ));
            }
            if row.agrees() && excused {
                problems.push(format!("{year} {rule} agrees and is excused in PARTS"));
            }
            rows.push(row);
        }
        if answer.added.len() != 4 {
            problems.push(format!(
                "{year}: {} days set by the moon, not 4",
                answer.added.len()
            ));
        }
        for added in &answer.added {
            let marked = added.published.subject_to == Some(SubjectTo::MoonSighting);
            sighted.push((
                year,
                added.key.clone(),
                (added.day.month, added.day.day),
                marked,
            ));
        }
    }
    let past = context
        .almanac()
        .festivals(
            &date(2031, (1, 1)),
            &date(2031, (12, 31)),
            &place(),
            offset,
            &request,
        )
        .map_err(|e| e.to_string())?
        .value;
    if !past.added.is_empty() {
        problems.push(String::from(
            "2031 has no notice and holds a day set by the moon",
        ));
    }
    Ok((rows, sighted))
}

/// A Markdown table.
fn table(heads: &[&str], rows: &[Vec<String>]) -> String {
    let line = |cells: &[String]| format!("| {} |\n", cells.join(" | "));
    let heads: Vec<String> = heads.iter().map(|&head| head.to_owned()).collect();
    let rule = vec![String::from("---"); heads.len()];
    let mut out = line(&heads) + &line(&rule);
    for row in rows {
        out.push_str(&line(row));
    }
    out
}

/// §9 of the page.
pub(super) fn render(out: &mut String, (rows, sighted): &Measured) {
    let parted = rows.iter().filter(|row| !row.agrees()).count();
    let _ = writeln!(
        out,
        "## 9. India's central holidays against the notices\n\n\
         `INDIA_CENTRAL` at Delhi on India's clock, against each year's Annexure I of the \
         Department of Personnel and Training's office memorandum. {} of {} rows agree.\n",
        rows.len() - parted,
        rows.len()
    );
    for (year, memorandum) in NOTICES {
        let _ = writeln!(out, "- {year}: {memorandum}");
    }
    let body: Vec<Vec<String>> = rows
        .iter()
        .filter(|row| !CIVIL.iter().any(|civil| civil.0 == row.rule))
        .map(|row| {
            let found = row.found.as_ref().map_or_else(
                || String::from("none"),
                |day| format!("{}/{}", day.month, day.day),
            );
            vec![
                row.year.to_string(),
                row.rule.to_owned(),
                format!("{}/{}", row.listed.0, row.listed.1),
                if row.agrees() {
                    found
                } else {
                    format!("{found} ✗")
                },
            ]
        })
        .collect();
    let _ = writeln!(
        out,
        "\n{}",
        table(&["year", "rule", "listed", "found"], &body)
    );
    let _ = writeln!(
        out,
        "The four civil days stand on their date in all {} years.\n",
        NOTICES.len()
    );
    for &(year, rule, why) in &PARTS {
        let _ = writeln!(out, "- {year} {rule}: {why}");
    }
    let body: Vec<Vec<String>> = sighted
        .iter()
        .map(|(year, key, day, marked)| {
            vec![
                year.to_string(),
                key.clone(),
                format!("{}/{}", day.0, day.1),
                String::from(if *marked {
                    "subject to sighting"
                } else {
                    "declared"
                }),
            ]
        })
        .collect();
    let _ = writeln!(
        out,
        "\nThe days set by the moon are not reckoned. Para 5.1 leaves each to the sighting, so \
         the pack carries the notice's day, marked; two the Department moved stand on the \
         day it declared; 2031, with no notice, holds none.\n\n{}",
        table(&["year", "key", "day", "stands as"], &body)
    );
}
