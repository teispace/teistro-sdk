#![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

//! The printed month beginnings each calendar is held to: individual days
//! of the *Rashtriya Panchang*'s regional tables (Positional Astronomy
//! Centre, Kolkata; the edition is named with each), chosen where the rule
//! decides, with the sankranti's printed IST beside them. The misses the
//! page records (`docs/calendars/regional-solar.md`) are asserted as the
//! SDK's day, with the printed day in the comment.

use teistro_core::catalogue::{Calendar, Era};
use teistro_core::envelope::CalendarResolution;
use teistro_core::error::Status;

use super::*;
use crate::CalendarSystem;
use crate::gregorian::{fixed_from_gregorian, gregorian_from_fixed};

fn calendar(id: Calendar) -> &'static RegionalSolar {
    RegionalSolar::shipped(id).unwrap()
}

/// The Gregorian day a month of the year that opens in a Gregorian year
/// begins on: the year's month counted from 1 at its first sign.
fn month_start(id: Calendar, gregorian_year: i32, month: u8) -> (i32, u8, u8) {
    let system = calendar(id);
    let year = gregorian_year - system.region().year_start.gregorian_offset;
    gregorian_from_fixed(system.to_fixed_ymd(year, month, 1).unwrap())
}

/// Asserts each `(sign, opening Gregorian year, printed start)`: the
/// sign's month in the year that opened in that Gregorian year.
fn assert_starts(id: Calendar, cases: &[(u8, i32, (i32, u8, u8))]) {
    let region = calendar(id).region();
    for &(sign, year, printed) in cases {
        let month = (1..=12)
            .find(|month| region.sign_of_month(*month) == Some(sign))
            .unwrap();
        assert_eq!(
            month_start(id, year, month),
            printed,
            "{}: sign {sign} of the year opening in {year}",
            region.name
        );
    }
}

#[test]
fn tamil_months_begin_before_sunset_at_chennai() {
    assert_starts(
        Calendar::TamilSolar,
        &[
            // Saka 1914: Chittirai, 13 April 1992.
            (0, 1992, (1992, 4, 13)),
            // Saka 1940: Tula at 18:44 and Vrishchika at 18:32, after
            // sunset, begin the next day.
            (6, 2018, (2018, 10, 18)),
            (7, 2018, (2018, 11, 17)),
            // Saka 1942: Simha at 19:11, the next day.
            (4, 2020, (2020, 8, 17)),
            // Saka 1945: Mithuna at 18:16, before sunset, the same day.
            (2, 2023, (2023, 6, 15)),
            // Saka 1946: Vrisha at 17:54, the same day.
            (1, 2024, (2024, 5, 14)),
            // Saka 1946: Meena at 18:51 on 14 March 2025, the next day.
            (11, 2024, (2025, 3, 15)),
            // Saka 1947: Karka at 17:32, the same day.
            (3, 2025, (2025, 7, 16)),
            // Saka 1942: Meena at 18:04 on 14 March 2021, the same day.
            (11, 2020, (2021, 3, 14)),
            // Saka 1948: Mesha, 14 April 2026.
            (0, 2026, (2026, 4, 14)),
        ],
    );
}

#[test]
fn bengal_months_begin_the_civil_day_after() {
    assert_starts(
        Calendar::BengalSolar,
        &[
            // Saka 1914: Vaisakha, 14 April 1992.
            (0, 1992, (1992, 4, 14)),
            // Saka 1940: Vrisha at 05:03 on 15 May 2018, after midnight,
            // begins the day after next by the Hindu day: 16 May.
            (1, 2018, (2018, 5, 16)),
            // Saka 1942: Mithuna at 23:54 on 14 June 2020, before
            // midnight: 15 June (Assam printed 16 June, C765).
            (2, 2020, (2020, 6, 15)),
            // Saka 1943: Vrisha at 23:25 on 14 May 2021: 15 May by the
            // rule; the column prints 16 May, most likely Assam's day.
            (1, 2021, (2021, 5, 15)),
            // Saka 1943: Meena at 00:16 on 15 March 2022: 16 March.
            (11, 2021, (2022, 3, 16)),
            // Saka 1946: Mithuna at 00:27 on 15 June 2024: 16 June.
            (2, 2024, (2024, 6, 16)),
            // Saka 1948: Karka at 23:39 on 16 July 2026: 17 July.
            (3, 2026, (2026, 7, 17)),
            // Saka 1946: Bengali San 1431 opened on 14 April 2024.
            (0, 2024, (2024, 4, 14)),
        ],
    );
}

#[test]
fn odia_months_begin_on_the_sunrise_day_at_bhubaneswar() {
    assert_starts(
        Calendar::OdiaSolar,
        &[
            // Saka 1914: Vaisakha, 13 April 1992.
            (0, 1992, (1992, 4, 13)),
            // Saka 1940: Vrisha at 05:03 on 15 May 2018, before sunrise:
            // the day before.
            (1, 2018, (2018, 5, 14)),
            // Saka 1945: Karka at 05:07 on 17 July 2023: 16 July.
            (3, 2023, (2023, 7, 16)),
            // Saka 1944: Vrisha at 05:30 on 15 May 2022, after sunrise:
            // the same day.
            (1, 2022, (2022, 5, 15)),
            // Saka 1943: Mithuna at 06:01 on 15 June 2021: the same day.
            (2, 2021, (2021, 6, 15)),
            // Saka 1942: Mithuna at 23:54 on 14 June 2020: the same day.
            (2, 2020, (2020, 6, 14)),
            // Saka 1947: Kumbha at 04:09 on 13 February 2026: the 12th.
            (10, 2025, (2026, 2, 12)),
            // Saka 1941 printed Vrishchika (00:51 on 17 November 2019) on
            // the 17th, against every other edition's reckoning; the SDK
            // keeps the sunrise day, the 16th.
            (7, 2019, (2019, 11, 16)),
        ],
    );
}

#[test]
fn malayalam_months_begin_before_three_fifths_of_the_daylight() {
    assert_starts(
        Calendar::MalayalamSolar,
        &[
            // Kollam 1168: Chingam, 17 August 1992.
            (4, 1992, (1992, 8, 17)),
            // Kollam 1195: Simha at 13:02 and Kanya at 13:03, before the
            // mark: the same day.
            (4, 2019, (2019, 8, 17)),
            (5, 2019, (2019, 9, 17)),
            // Kollam 1197: Tula at 13:12, the same day.
            (6, 2021, (2021, 10, 17)),
            // Kollam 1199: Kanya at 13:31, after the mark: the next day.
            (5, 2023, (2023, 9, 18)),
            // Kollam 1201: Tula at 13:46, the next day.
            (6, 2025, (2025, 10, 18)),
            // Kollam 1201: Chingam, 17 August 2025.
            (4, 2025, (2025, 8, 17)),
            // Kollam 1199: Simha at 13:33 on 17 August 2023, a minute and
            // a half from the mark: the SDK's sankranti falls before it
            // (the 17th), as the Kerala Gazette prints it; the column
            // prints the 18th, its slip.
            (4, 2023, (2023, 8, 17)),
            // Kollam 1167: Mesha 1992 by the Malabar rule, as printed.
            (0, 1991, (1992, 4, 14)),
            // Kollam 1194: Mesha at 14:09 on 14 April 2019, after the
            // mark: 15 April by the Malabar rule; the 2018 and later
            // editions print the 14th, the sunrise-to-sunrise day, where
            // the Kerala Gazette prints the Malabar rule's (C764).
            (0, 2018, (2019, 4, 15)),
        ],
    );
}

/// The Kerala Gazette's printed pairs, each a Christian Era date and the
/// Malayalam Era date beside it (Kollam year, month from Chingam, day):
/// the Government of Kerala's "Public Holidays" notifications for 2021 to
/// 2026 (G.O.(P) No. 16/2020, 29/2021, 22/2022, 24/2023, 34/2024 and
/// 14/2025/GAD), each holiday printed with its ME date, and three
/// mastheads. Every month a pair falls in has its first day pinned by it:
/// 61 months, 2020-12 to 2026-12.
const KERALA_GAZETTE: [((i32, u8, u8), (i32, u8, u8)); 73] = [
    ((2021, 1, 2), (1196, 5, 18)),
    ((2021, 1, 26), (1196, 6, 13)),
    ((2021, 3, 11), (1196, 7, 27)),
    ((2021, 4, 1), (1196, 8, 18)),
    ((2021, 4, 14), (1196, 9, 1)),
    ((2021, 7, 20), (1196, 12, 4)),
    ((2021, 8, 19), (1197, 1, 3)),
    ((2021, 9, 21), (1197, 2, 5)),
    ((2021, 10, 19), (1197, 3, 3)),
    ((2021, 12, 25), (1197, 5, 10)),
    ((2022, 1, 2), (1197, 5, 18)),
    ((2022, 1, 26), (1197, 6, 12)),
    ((2022, 3, 1), (1197, 7, 17)),
    ((2022, 4, 14), (1197, 9, 1)),
    ((2022, 4, 15), (1197, 9, 2)),
    ((2022, 7, 9), (1197, 11, 25)),
    ((2022, 7, 28), (1197, 12, 12)),
    ((2022, 8, 18), (1198, 1, 2)),
    ((2022, 9, 21), (1198, 2, 5)),
    ((2022, 10, 24), (1198, 3, 7)),
    ((2022, 10, 27), (1198, 3, 10)),
    ((2022, 12, 25), (1198, 5, 10)),
    ((2023, 1, 2), (1198, 5, 18)),
    ((2023, 1, 26), (1198, 6, 12)),
    ((2023, 2, 18), (1198, 7, 6)),
    ((2023, 4, 6), (1198, 8, 23)),
    ((2023, 4, 14), (1198, 8, 31)),
    ((2023, 4, 15), (1198, 9, 1)),
    ((2023, 6, 28), (1198, 11, 13)),
    ((2023, 7, 17), (1198, 12, 1)),
    ((2023, 8, 28), (1199, 1, 12)),
    ((2023, 9, 22), (1199, 2, 5)),
    ((2023, 10, 5), (1199, 2, 18)),
    ((2023, 10, 24), (1199, 3, 7)),
    ((2023, 12, 25), (1199, 5, 9)),
    ((2024, 1, 2), (1199, 5, 17)),
    ((2024, 1, 26), (1199, 6, 12)),
    ((2024, 3, 8), (1199, 7, 24)),
    ((2024, 3, 28), (1199, 8, 15)),
    ((2024, 4, 14), (1199, 9, 1)),
    ((2024, 6, 17), (1199, 11, 3)),
    ((2024, 7, 16), (1199, 12, 1)),
    ((2024, 8, 20), (1200, 1, 4)),
    ((2024, 9, 17), (1200, 2, 1)),
    ((2024, 10, 31), (1200, 3, 15)),
    ((2024, 12, 25), (1200, 5, 10)),
    ((2025, 1, 26), (1200, 6, 13)),
    ((2025, 2, 26), (1200, 7, 14)),
    ((2025, 3, 31), (1200, 8, 17)),
    ((2025, 4, 14), (1200, 9, 1)),
    ((2025, 6, 6), (1200, 10, 23)),
    ((2025, 7, 6), (1200, 11, 22)),
    ((2025, 7, 24), (1200, 12, 8)),
    ((2025, 8, 15), (1200, 12, 30)),
    ((2025, 8, 28), (1201, 1, 12)),
    ((2025, 9, 21), (1201, 2, 5)),
    ((2025, 10, 1), (1201, 2, 15)),
    ((2025, 10, 20), (1201, 3, 3)),
    ((2025, 10, 31), (1201, 3, 14)),
    ((2025, 12, 25), (1201, 5, 10)),
    ((2026, 1, 2), (1201, 5, 18)),
    ((2026, 1, 26), (1201, 6, 12)),
    ((2026, 2, 15), (1201, 7, 3)),
    ((2026, 3, 20), (1201, 8, 6)),
    ((2026, 4, 14), (1201, 9, 1)),
    ((2026, 4, 15), (1201, 9, 2)),
    ((2026, 5, 27), (1201, 10, 13)),
    ((2026, 6, 25), (1201, 11, 11)),
    ((2026, 8, 12), (1201, 12, 27)),
    ((2026, 8, 25), (1202, 1, 9)),
    ((2026, 9, 21), (1202, 2, 5)),
    ((2026, 10, 20), (1202, 3, 3)),
    ((2026, 12, 25), (1202, 5, 10)),
];

/// The first day of each month a printed pair pins, by (year, month).
fn printed_starts(
    printed: &[((i32, u8, u8), (i32, u8, u8))],
) -> std::collections::BTreeMap<(i32, u8), FixedDay> {
    let mut starts = std::collections::BTreeMap::new();
    for &((y, m, d), (year, month, day)) in printed {
        let start = fixed_from_gregorian(y, m, d).plus_days(1 - i64::from(day));
        let earlier = starts.insert((year, month), start);
        assert!(
            earlier.is_none_or(|earlier| earlier == start),
            "{year}-{month}: the print's own pairs disagree"
        );
    }
    starts
}

/// Asserts every printed pair is the calendar's date, and returns the
/// months pinned.
fn assert_pairs(id: Calendar, printed: &[((i32, u8, u8), (i32, u8, u8))]) -> usize {
    let system = calendar(id);
    for &((y, m, d), wanted) in printed {
        let date = system.date_of(fixed_from_gregorian(y, m, d)).unwrap();
        assert_eq!(
            (date.year, date.month, date.day),
            wanted,
            "{}: {y}-{m}-{d}",
            system.region().name
        );
    }
    printed_starts(printed).len()
}

#[test]
fn malayalam_months_match_the_kerala_gazette() {
    // Every pair printed, so every one of the 61 months begins on the
    // gazette's day, Medam and the minute-scale boundaries included.
    assert_eq!(assert_pairs(Calendar::MalayalamSolar, &KERALA_GAZETTE), 61);
}

#[test]
fn kerala_medam_follows_the_malabar_rule() {
    // Medam 1 (the Mesha sankranti's month) in the six gazette years: by
    // the Malabar rule for every sign. The Rashtriya Panchang's Kerala
    // column prints the day before in 2021, 2023, 2024 and 2025, the
    // sankranti's sunrise-to-sunrise day (C764, closed for the gazette).
    for (year, printed) in [
        (1196, (2021, 4, 14)),
        (1197, (2022, 4, 14)),
        (1198, (2023, 4, 15)),
        (1199, (2024, 4, 14)),
        (1200, (2025, 4, 14)),
        (1201, (2026, 4, 14)),
    ] {
        let system = calendar(Calendar::MalayalamSolar);
        assert_eq!(
            gregorian_from_fixed(system.to_fixed_ymd(year, 9, 1).unwrap()),
            printed,
            "Medam {year}"
        );
    }
    // The era: 1 Chingam 1200 is 17 August 2024 and 30 Karkidakam 1200,
    // the year's last day, 16 August 2025.
    let system = calendar(Calendar::MalayalamSolar);
    assert_eq!(
        gregorian_from_fixed(system.to_fixed_ymd(1200, 1, 1).unwrap()),
        (2024, 8, 17)
    );
    let last = system.month_length(1200, 12).unwrap();
    assert_eq!(
        gregorian_from_fixed(system.to_fixed_ymd(1200, 12, last).unwrap()),
        (2025, 8, 16)
    );
}

#[test]
fn the_kerala_gazette_reckons_the_modern_sun() {
    // The same rule at the same place over the Surya Siddhanta's Sun
    // misses 22 of the gazette's 61 months: the text's sankrantis fall
    // hours from the modern Sun's, across the three-fifths mark (Simha
    // and Tula among them). The shipped (drik) calendar misses none.
    let text = teistro_siddhanta::SuryaSiddhanta::text();
    let engine = MALAYALAM.engine(&text);
    let mut missed = Vec::new();
    for ((year, month), printed) in printed_starts(&KERALA_GAZETTE) {
        let row = engine.year(year).unwrap();
        let before: i64 = row
            .months
            .iter()
            .take(usize::from(month - 1))
            .map(|d| i64::from(*d))
            .sum();
        if row.start.plus_days(before) != printed {
            missed.push((year, month));
        }
    }
    for named in [(1197, 3), (1199, 1), (1199, 3), (1200, 3)] {
        assert!(missed.contains(&named), "{named:?} in {missed:?}");
    }
    assert_eq!(missed.len(), 22, "{missed:?}");
}

/// The Government of Assam's printed pairs, each a Gregorian date and the
/// Assamese (Bhaskarabda) date beside it (year, month from Bohag, day):
/// the General Administration Department's "List of Holidays" for 2023
/// (No. GAD-14021/18/2022-ESTT-GAD of 28 November 2022) and for 2025 (No.
/// E-221561/347 of 6 December 2024, with its corrigendum of 27 December
/// 2024, whose dated masthead is the first 2025 pair). 27 months.
const ASSAM_GAZETTE: [((i32, u8, u8), (i32, u8, u8)); 34] = [
    ((2022, 11, 28), (1429, 8, 11)),
    ((2023, 1, 1), (1429, 9, 16)),
    ((2023, 1, 15), (1429, 9, 30)),
    ((2023, 1, 16), (1429, 10, 1)),
    ((2023, 3, 7), (1429, 11, 22)),
    ((2023, 4, 7), (1429, 12, 23)),
    ((2023, 4, 14), (1429, 12, 30)),
    ((2023, 4, 15), (1430, 1, 1)),
    ((2023, 6, 5), (1430, 2, 21)),
    ((2023, 6, 29), (1430, 3, 13)),
    ((2023, 8, 15), (1430, 4, 29)),
    ((2023, 8, 18), (1430, 4, 32)),
    ((2023, 8, 27), (1430, 5, 9)),
    ((2023, 9, 25), (1430, 6, 7)),
    ((2023, 10, 18), (1430, 6, 30)),
    ((2023, 10, 21), (1430, 7, 3)),
    ((2023, 11, 19), (1430, 8, 2)),
    ((2023, 12, 25), (1430, 9, 8)),
    ((2024, 12, 6), (1431, 8, 20)),
    ((2025, 1, 14), (1431, 9, 29)),
    ((2025, 1, 15), (1431, 10, 1)),
    ((2025, 2, 12), (1431, 10, 29)),
    ((2025, 3, 14), (1431, 11, 29)),
    ((2025, 3, 31), (1431, 12, 17)),
    ((2025, 4, 14), (1431, 12, 31)),
    ((2025, 4, 15), (1432, 1, 1)),
    ((2025, 5, 24), (1432, 2, 9)),
    ((2025, 8, 15), (1432, 4, 29)),
    ((2025, 8, 25), (1432, 5, 8)),
    ((2025, 9, 29), (1432, 6, 12)),
    ((2025, 10, 18), (1432, 6, 31)),
    ((2025, 10, 20), (1432, 7, 2)),
    ((2025, 11, 19), (1432, 8, 2)),
    ((2025, 12, 25), (1432, 9, 9)),
];

#[test]
fn assamese_months_match_the_assam_gazette() {
    // Every pair printed: 27 months, Saon 1430's 32 days among them.
    assert_eq!(assert_pairs(Calendar::AssamSolar, &ASSAM_GAZETTE), 27);
    let system = calendar(Calendar::AssamSolar);
    assert_eq!(system.month_length(1430, 4).unwrap(), 32);
    assert_eq!(
        system.year_start(1432),
        Some(fixed_from_gregorian(2025, 4, 15))
    );
}

#[test]
fn the_assam_gazette_reckons_the_surya_siddhantas_sun() {
    // The same rule over the modern Sun is the Bengal calendar's table
    // (the civil day after, Bengali San = Bhaskarabda): it begins 11 of
    // the 27 months a day before the gazette.
    let bengal = calendar(Calendar::BengalSolar);
    let starts = printed_starts(&ASSAM_GAZETTE);
    let early = starts
        .iter()
        .filter(|((year, month), printed)| {
            bengal.to_fixed_ymd(*year, *month, 1).unwrap().plus_days(1) == **printed
        })
        .count();
    let held = starts
        .iter()
        .filter(|((year, month), printed)| {
            bengal.to_fixed_ymd(*year, *month, 1).unwrap() == **printed
        })
        .count();
    assert_eq!((held, early), (16, 11));
}

/// The minutes after the IST midnight that begins a sankranti's civil day.
fn ist_minutes(
    instant: teistro_core::quantity::JulianDay<teistro_core::quantity::Utc>,
) -> (FixedDay, f64) {
    let (day, fraction) = FixedDay::from_local_jd(instant.get() + 5.5 / 24.0);
    (day, fraction * 1440.0)
}

#[test]
fn the_near_midnight_rows_fall_after_midnight_on_the_text() {
    // The three gazette months whose Surya Siddhanta sankranti a rough
    // model put within an hour and a half after midnight: Puh 1430
    // (Dhanu, 17 December 2023), Phagun 1431 (Kumbha, 13 February 2025)
    // and Aghun 1432 (Vrishchika, 17 November 2025). On the SDK's own
    // text each falls after IST midnight on that day (00:52, 01:27 and
    // 01:30), so the month begins the day after, as printed.
    let text = teistro_siddhanta::SuryaSiddhanta::text();
    let engine = ASSAM.engine(&text);
    for (year, month, day, after) in [
        (1430, 9_u8, (2023, 12, 17), 52.0),
        (1431, 11, (2025, 2, 13), 86.0),
        (1432, 8, (2025, 11, 17), 90.0),
    ] {
        let row = engine.year(year).unwrap();
        let (civil, minutes) = ist_minutes(row.sankrantis[usize::from(month - 1)]);
        assert_eq!(gregorian_from_fixed(civil), day, "{year}-{month}");
        assert!(
            minutes > after,
            "{year}-{month}: {minutes:.1} minutes after midnight"
        );
    }
    // No printed month's sankranti on the text falls within 52 minutes
    // of IST midnight (Puh 1430 is the nearest, at 00:52), so no row
    // tells IST midnight from Guwahati's or from a tithi test (C1302).
    for ((year, month), _) in printed_starts(&ASSAM_GAZETTE) {
        let row = engine.year(year).unwrap();
        let (_, minutes) = ist_minutes(row.sankrantis[usize::from(month - 1)]);
        assert!(minutes.min(1440.0 - minutes) > 52.0, "{year}-{month}: {minutes:.1}");
    }
}

#[test]
fn bihu_falls_on_the_assamese_month_ends() {
    // Kati Bihu is the last day of Ahin, the Tula sankranti's day: the
    // gazette prints 17 October 2020, 18 October 2023 and 2025 (30 and 31
    // Ahin); press copies of the lists 18 October 2021, 2022 and 2026
    // and 17 October 2024. Each is also 1 Kartik in the Bikram Sambat table,
    // another rule (punya kala) on the same sky.
    let assam = calendar(Calendar::AssamSolar);
    let nepal = crate::BikramSambat::shipped();
    for (year, kati_bihu) in [
        (1427, (2020, 10, 17)),
        (1428, (2021, 10, 18)),
        (1429, (2022, 10, 18)),
        (1430, (2023, 10, 18)),
        (1431, (2024, 10, 17)),
        (1432, (2025, 10, 18)),
        (1433, (2026, 10, 18)),
    ] {
        let day = assam.to_fixed_ymd(year, 7, 1).unwrap().plus_days(-1);
        assert_eq!(gregorian_from_fixed(day), kati_bihu, "Kati Bihu {year}");
        let kartik = nepal.date_of(day).unwrap();
        assert_eq!((kartik.month, kartik.day), (7, 1), "{kati_bihu:?}");
    }
    // Magh Bihu is the last day of Puh: 15 January 2023 and 14 January 2025.
    for (year, magh_bihu) in [(1429, (2023, 1, 15)), (1431, (2025, 1, 14))] {
        let day = assam.to_fixed_ymd(year, 10, 1).unwrap().plus_days(-1);
        assert_eq!(gregorian_from_fixed(day), magh_bihu, "Magh Bihu {year}");
    }
}

#[test]
fn every_date_names_its_era_and_frame() {
    for (id, era) in [
        (Calendar::TamilSolar, Era::Shaka),
        (Calendar::BengalSolar, Era::Bengali),
        (Calendar::OdiaSolar, Era::Shaka),
        (Calendar::MalayalamSolar, Era::Kollam),
    ] {
        let system = calendar(id);
        assert_eq!(system.id(), id);
        let date = system.date_of(fixed_from_gregorian(2025, 1, 1)).unwrap();
        assert_eq!(date.era.map(|e| (e.era, e.year)), Some((era, date.year)));
        let CalendarResolution::Computed { model } = &date.resolution else {
            panic!("{id:?}: {:?}", date.resolution);
        };
        assert_eq!(model, system.table().frame);
        assert!(model.contains("ayanamsha LAHIRI (true basis)"), "{model}");
        assert!(model.contains(&system.region().rule.key()), "{model}");
        assert!(!model.contains(env!("CARGO_PKG_VERSION")), "{model}");
        let capabilities = system.capabilities();
        assert!(!capabilities.needs_ephemeris && !capabilities.needs_place);
        assert_eq!(capabilities.eras, [era]);
    }
    let assam = calendar(Calendar::AssamSolar);
    let date = assam.date_of(fixed_from_gregorian(2025, 4, 15)).unwrap();
    assert_eq!(
        (date.era.map(|e| (e.era, e.year)), date.month, date.day),
        (Some((Era::Bhaskarabda, 1432)), 1, 1)
    );
    let CalendarResolution::Computed { model } = &date.resolution else {
        panic!("{:?}", date.resolution);
    };
    assert_eq!(model, assam.table().frame);
    assert!(
        model.starts_with("Surya Siddhanta (the text, the sine table); +05:30;")
            && model.ends_with("rule FOLLOWING_DAY"),
        "{model}"
    );
    assert_eq!(assam.capabilities().eras, [Era::Bhaskarabda]);
}

#[test]
fn every_day_reads_back_and_years_abut() {
    let ids = REGIONS.iter().map(|region| region.calendar);
    for id in ids.chain([Calendar::TamilVakya]) {
        let system = calendar(id);
        let region = system.region();
        let (first, last) = system.capabilities().range;
        let (first_year, last_year) = system.years();
        assert_eq!(
            usize::try_from(last_year - first_year + 1).unwrap(),
            system.table().month_lengths.len()
        );
        assert_eq!(
            gregorian_from_fixed(first).0,
            GREGORIAN_SPAN.0,
            "{}",
            region.name
        );
        let mut previous: Option<CalendarDate> = None;
        for day in first.get()..=last.get() {
            let fixed = FixedDay::new(day);
            let date = system.date_of(fixed).unwrap();
            assert_eq!(system.fixed_of(&date).unwrap(), fixed, "{date:?}");
            if let Some(before) = previous {
                let next_day = before.day + 1;
                let follows = (date.year, date.month, date.day)
                    == (before.year, before.month, next_day)
                    || (date.day == 1
                        && ((date.year, date.month) == (before.year, before.month + 1)
                            || (date.year, date.month, before.month) == (before.year + 1, 1, 12)));
                assert!(follows, "{before:?} then {date:?}");
            }
            previous = Some(date);
        }
        for year in first_year..=last_year {
            let days: u32 = (1..=12)
                .map(|month| u32::from(system.month_length(year, month).unwrap()))
                .sum();
            assert!(days == 365 || days == 366, "{} {year}: {days}", region.name);
            assert_eq!(system.is_leap(year), days == 366);
            assert!(
                (1..=12)
                    .all(|month| (29..=32).contains(&system.month_length(year, month).unwrap())),
                "{} {year}",
                region.name
            );
        }
    }
}

#[test]
fn a_date_outside_the_table_or_the_month_is_refused() {
    let tamil = calendar(Calendar::TamilSolar);
    let (first, last) = tamil.years();
    for year in [first - 1, last + 1] {
        let refused = tamil.to_fixed_ymd(year, 1, 1).unwrap_err();
        assert_eq!(refused.status, Status::OutOfRange, "{refused}");
        assert!(refused.message.contains("Tamil"), "{refused}");
        assert!(tamil.year_start(year).is_none());
    }
    assert!(tamil.to_fixed_ymd(1946, 13, 1).is_err());
    assert!(tamil.to_fixed_ymd(1946, 0, 1).is_err());
    let length = tamil.month_length(1946, 1).unwrap();
    assert!(tamil.to_fixed_ymd(1946, 1, length + 1).is_err());
    let marked =
        CalendarDate::defined(Calendar::TamilSolar, 1946, 1, 1).with_leap(Some(crate::Leap::Day));
    assert!(tamil.fixed_of(&marked).is_err());
    let (range_first, range_last) = tamil.capabilities().range;
    assert!(tamil.date_of(range_first.plus_days(-1)).is_err());
    assert!(tamil.date_of(range_last.plus_days(1)).is_err());
    assert!(RegionalSolar::shipped(Calendar::Gregorian).is_none());
    assert_eq!(MALAYALAM.sign_of_month(0), None);
}

#[test]
fn the_engine_reproduces_the_shipped_rows_under_the_tables_rule() {
    // The engine run over the Surya Siddhanta's Sun is a different
    // astronomy, but the year start and the rule are the calendar's own:
    // a Malayalam year opens at Simha and runs twelve months.
    let text = teistro_siddhanta::SuryaSiddhanta::text();
    let row = MALAYALAM.engine(&text).year(1201).unwrap();
    let (year, month, _) = gregorian_from_fixed(row.start);
    assert_eq!((year, month), (2025, 8));
    assert!(row.days() == 365 || row.days() == 366);
    assert_eq!(MALAYALAM.engine(&text).year_start(), MALAYALAM.year_start);
}

/// The month beginnings the vakya almanacs print, as days of the month
/// from Chittirai to Panguni, by the Gregorian year the Tamil year opens
/// in: the Manonmani Vilasam suddha vakya almanac (archive.org
/// `bqfm-manonmani-vilasa-suddha-vakya-panchangam-sarvajitt` for 1947 to
/// 1962, `calw-manonmani-vilasa-suddha-vakkiya-panchangam-isvara-` for
/// 1997 to 2021) and the Koviloor vakya almanac for 1978 (Tamil Digital
/// Library `KMM1082`). A zero is a month not read.
const VAKYA_PRINTED: [(i32, [u8; 12]); 42] = [
    (1947, [14, 15, 15, 17, 17, 17, 18, 16, 16, 14, 13, 14]),
    (1948, [13, 14, 14, 16, 16, 16, 17, 16, 15, 14, 12, 14]),
    (1949, [13, 14, 14, 16, 17, 17, 17, 16, 16, 14, 12, 14]),
    (1950, [13, 14, 15, 16, 17, 17, 17, 16, 16, 14, 13, 14]),
    (1951, [14, 15, 15, 17, 17, 17, 18, 17, 16, 14, 13, 14]),
    (1952, [13, 14, 14, 16, 16, 16, 17, 16, 15, 14, 12, 14]),
    (1953, [13, 14, 15, 16, 17, 17, 17, 16, 16, 14, 12, 14]),
    (1954, [13, 14, 15, 16, 17, 17, 17, 16, 16, 14, 13, 14]),
    (1955, [14, 15, 15, 17, 17, 17, 18, 17, 16, 14, 13, 14]),
    (1956, [13, 14, 14, 16, 16, 16, 17, 16, 15, 14, 12, 14]),
    (1957, [13, 14, 15, 16, 17, 17, 17, 16, 16, 14, 12, 14]),
    (1958, [14, 14, 15, 16, 17, 17, 17, 16, 16, 14, 13, 14]),
    (1959, [14, 15, 15, 17, 17, 17, 18, 17, 16, 14, 13, 14]),
    (1960, [13, 14, 14, 16, 16, 16, 17, 16, 15, 14, 12, 14]),
    (1961, [13, 14, 15, 16, 17, 17, 17, 16, 16, 14, 12, 14]),
    (1962, [14, 14, 15, 16, 0, 0, 0, 0, 0, 0, 0, 0]),
    (1978, [14, 15, 15, 17, 17, 17, 18, 17, 16, 14, 13, 15]),
    (1997, [14, 15, 15, 17, 17, 17, 18, 16, 16, 14, 13, 15]),
    (1998, [14, 15, 15, 17, 17, 17, 18, 17, 16, 15, 13, 15]),
    (1999, [14, 15, 15, 17, 18, 18, 18, 17, 16, 15, 13, 14]),
    (2000, [13, 14, 15, 16, 17, 17, 17, 16, 16, 14, 13, 14]),
    (2001, [14, 15, 15, 17, 17, 17, 18, 16, 16, 14, 13, 15]),
    (2002, [14, 15, 15, 17, 17, 17, 18, 17, 16, 15, 13, 15]),
    (2003, [14, 15, 15, 17, 18, 18, 18, 17, 16, 15, 13, 14]),
    (2004, [13, 14, 15, 16, 17, 17, 17, 16, 16, 14, 13, 14]),
    (2005, [14, 15, 15, 17, 17, 17, 18, 16, 16, 14, 13, 15]),
    (2006, [14, 15, 15, 17, 17, 17, 18, 17, 16, 15, 13, 15]),
    (2007, [14, 15, 15, 17, 18, 18, 18, 17, 17, 15, 13, 14]),
    (2008, [13, 14, 15, 16, 17, 17, 17, 16, 16, 14, 13, 14]),
    (2009, [14, 15, 15, 17, 17, 17, 18, 17, 16, 14, 13, 15]),
    (2010, [14, 15, 15, 17, 17, 17, 18, 17, 16, 15, 13, 15]),
    (2011, [14, 15, 16, 17, 18, 18, 18, 17, 17, 15, 13, 14]),
    (2012, [13, 14, 15, 16, 17, 17, 17, 16, 16, 14, 13, 14]),
    (2013, [14, 15, 15, 17, 17, 17, 18, 17, 16, 14, 13, 15]),
    (2014, [14, 15, 15, 17, 17, 17, 18, 17, 16, 15, 13, 15]),
    (2015, [14, 15, 16, 17, 18, 18, 18, 17, 17, 15, 13, 14]),
    (2016, [14, 14, 15, 16, 17, 17, 17, 16, 16, 14, 13, 14]),
    (2017, [14, 15, 15, 17, 17, 17, 18, 17, 16, 14, 13, 15]),
    (2018, [14, 15, 15, 17, 17, 17, 18, 17, 16, 15, 13, 15]),
    (2019, [14, 15, 16, 17, 18, 18, 18, 17, 17, 15, 13, 14]),
    (2020, [14, 14, 15, 16, 17, 17, 17, 16, 16, 14, 13, 14]),
    (2021, [14, 15, 15, 17, 17, 17, 18, 17, 16, 14, 13, 15]),
];

/// The Gregorian month each Tamil month begins in, Chittirai first.
const TAMIL_GREGORIAN_MONTHS: [u8; 12] = [4, 5, 6, 7, 8, 9, 10, 11, 12, 1, 2, 3];

#[test]
fn vakya_months_begin_on_the_almanacs_days() {
    let mut read = 0;
    let mut parting = 0;
    for (year, days) in VAKYA_PRINTED {
        for ((month, day), gregorian_month) in (1..=12).zip(days).zip(TAMIL_GREGORIAN_MONTHS) {
            if day == 0 {
                continue;
            }
            let gregorian_year = if month <= 9 { year } else { year + 1 };
            let printed = (gregorian_year, gregorian_month, day);
            assert_eq!(
                month_start(Calendar::TamilVakya, year, month),
                printed,
                "month {month} of {year}"
            );
            read += 1;
            parting += usize::from(month_start(Calendar::TamilSolar, year, month) != printed);
        }
    }
    // Every month read, and the drik calendar misses a fifth of them.
    assert_eq!((read, parting), (496, 89));
}

#[test]
fn vakya_months_turn_on_the_almanacs_daytime() {
    // Kali 5101: the Kumbha sankranti at 29 nadis 41 vinadis after the
    // sunrise of 12 February 2001, past that day's 29 nadis 7 vinadis of
    // daylight: Masi begins on the 13th, as printed, where a thirty-nadi
    // cutoff would give the 12th.
    let kumbha = teistro_siddhanta::vakya::sankranti(5101, 10).unwrap();
    assert_eq!(kumbha.since_sunrise().0, 29);
    assert_eq!(month_start(Calendar::TamilVakya, 2000, 11), (2001, 2, 13));
    // Saka 1869's Mesha sankranti at night: the year began on Monday.
    assert_eq!(
        vakya::month_start(1869, 0),
        Some(fixed_from_gregorian(1947, 4, 14))
    );
    assert_eq!(vakya::month_start(1869, 12), None);
}

#[test]
fn the_vakya_calendar_names_its_frame_and_keeps_the_tamil_year() {
    let system = calendar(Calendar::TamilVakya);
    assert_eq!(system.id(), Calendar::TamilVakya);
    assert_eq!(system.region().calendar, Calendar::TamilSolar);
    assert_eq!(system.years(), calendar(Calendar::TamilSolar).years());
    let date = system.date_of(fixed_from_gregorian(2025, 1, 1)).unwrap();
    assert_eq!(date.calendar, Calendar::TamilVakya);
    assert_eq!(
        date.era.map(|e| (e.era, e.year)),
        Some((Era::Shaka, date.year))
    );
    let CalendarResolution::Computed { model } = &date.resolution else {
        panic!("{:?}", date.resolution);
    };
    assert_eq!(model, vakya::FRAME);
    assert!(
        model.contains("Vakyakarana") && model.contains(&TAMIL.rule.key()),
        "{model}"
    );
    let capabilities = system.capabilities();
    assert!(!capabilities.needs_ephemeris && !capabilities.needs_place);
    assert_eq!(capabilities.eras, [Era::Shaka]);
    let (_, last) = system.years();
    let refused = system.to_fixed_ymd(last + 1, 1, 1).unwrap_err();
    assert!(refused.message.contains("Tamil vakya"), "{refused}");
    let early = system
        .date_of(system.capabilities().range.0.plus_days(-1))
        .unwrap_err();
    assert!(early.message.contains("Tamil vakya"), "{early}");
    let marked =
        CalendarDate::defined(Calendar::TamilVakya, 1946, 1, 1).with_leap(Some(crate::Leap::Day));
    assert!(system.fixed_of(&marked).is_err());
    assert!(system.to_fixed_ymd(1946, 13, 1).is_err());
}

#[test]
fn the_schools_part_in_a_fifth_of_the_months() {
    // Tamil years opening 1900 to 2100: the vakya month begins a day
    // after the drik one in 450 months and a day before it in 74.
    let (mut later, mut earlier) = (0, 0);
    for year in 1900..=2100 {
        for month in 1..=12 {
            let vakya = fixed_from_gregorian_tuple(month_start(Calendar::TamilVakya, year, month));
            let drik = fixed_from_gregorian_tuple(month_start(Calendar::TamilSolar, year, month));
            match drik.days_until(vakya) {
                1 => later += 1,
                -1 => earlier += 1,
                0 => {}
                other => panic!("{year} month {month}: {other} days apart"),
            }
        }
    }
    assert_eq!((later, earlier), (450, 74));
}

fn fixed_from_gregorian_tuple((year, month, day): (i32, u8, u8)) -> FixedDay {
    fixed_from_gregorian(year, month, day)
}
