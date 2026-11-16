//! The four Bikram Sambat month starts the table and other sources
//! dispute (SR-107; `docs/calendars/bikram-sambat.md`, "The four disputed
//! rows"; crux C1850): each pinned to the decided value, the table's,
//! with the engine's other day and the sankranti's margin to the boundary
//! its rule uses, and the data file's own record of the four held to the
//! same values.
//!
//! Sources, for every row: the table's day is an online almanac
//! published in Nepal (rank 3), the reading the best-ranked source
//! gives; the other day is that of the open-source converter tables
//! `bikram-sambat` 1.8.1 (npm, Medic) and `nepali-datetime` (the Python
//! Package Index), rank 4, and of the SDK's engine. No patro of those
//! years, nor a dated document carrying a weekday in those months, has
//! been found.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::cast_possible_truncation,
    reason = "tests fail by panicking and read a small data file"
)]

use std::path::Path;

use serde_json::Value;
use teistro_calendar::bikram_sambat::{Engine, KATHMANDU};
use teistro_calendar::gregorian::fixed_from_gregorian;
use teistro_calendar::solar::civil_day_light;
use teistro_calendar::{BikramSambat, CalendarSystem, FixedDay, MonthStartRule};
use teistro_core::envelope::{CalendarResolution, MonthDay};
use teistro_siddhanta::SuryaSiddhanta;
use teistro_time::zones;

/// Nepal's clock in 1920 to 1941 as the engine reads it (tzdb: +05:30
/// from 1920 to 1986), in days.
const CLOCK_DAYS: f64 = 5.5 / 24.0;

/// One disputed month start.
struct Disputed {
    year: i32,
    month: u8,
    /// The table's 1st of the month, the decided value.
    table: (i32, u8, u8),
    /// The other sources' and the engine's 1st of the month.
    other: (i32, u8, u8),
    /// The sankranti's signed distance in minutes from the boundary the
    /// rule uses (midnight, or sunset for Makara), on Nepal's clock of the
    /// time: negative before it.
    margin_minutes: f64,
}

const ROWS: [Disputed; 4] = [
    // Kartik 1977 (Tula): the almanac 17 Oct 1920; Medic, nepali-datetime
    // and the engine 16 Oct. The text's sankranti 23:59:29, half a minute
    // before midnight (00:25 on the 17th in Kathmandu's apparent time).
    Disputed {
        year: 1977,
        month: 7,
        table: (1920, 10, 17),
        other: (1920, 10, 16),
        margin_minutes: -0.5,
    },
    // Magh 1977 (Makara, the before-sunset rule): the almanac 14 Jan 1921;
    // the others 13 Jan. The sankranti 16:51:12, 7.9 minutes before the
    // text's sunset.
    Disputed {
        year: 1977,
        month: 10,
        table: (1921, 1, 14),
        other: (1921, 1, 13),
        margin_minutes: -7.9,
    },
    // Chaitra 1981 (Meena): the almanac 13 Mar 1925; the others 14 Mar.
    // The sankranti 00:07:54, 7.9 minutes after midnight.
    Disputed {
        year: 1981,
        month: 12,
        table: (1925, 3, 13),
        other: (1925, 3, 14),
        margin_minutes: 7.9,
    },
    // Ashwin 1998 (Kanya): the almanac 17 Sep 1941; the others 16 Sep.
    // The sankranti 23:47:55, 12.1 minutes before midnight (00:04 on the
    // 17th in Kathmandu's apparent time).
    Disputed {
        year: 1998,
        month: 6,
        table: (1941, 9, 17),
        other: (1941, 9, 16),
        margin_minutes: -12.1,
    },
];

fn day((y, m, d): (i32, u8, u8)) -> FixedDay {
    fixed_from_gregorian(y, m, d)
}

/// The 1st of `month` in a row of month lengths starting on `start`.
fn month_start(start: FixedDay, months: &[u8; 12], month: u8) -> FixedDay {
    start.plus_days(
        months[..usize::from(month - 1)]
            .iter()
            .map(|m| i64::from(*m))
            .sum(),
    )
}

#[test]
fn the_table_keeps_the_four_disputed_month_starts() {
    let bs = BikramSambat::shipped();
    for row in &ROWS {
        let first = bs.to_fixed_ymd(row.year, row.month, 1).unwrap();
        assert_eq!(first, day(row.table), "BS {} month {}", row.year, row.month);
        // The month before ends a day later or earlier than the other
        // reading has it, and the year's total is the same in both.
        let other = day(row.other);
        assert_eq!(first.days_until(other).abs(), 1, "BS {}", row.year);
    }
}

#[test]
fn the_engine_gives_the_other_day_within_minutes_of_its_boundary() {
    let text = SuryaSiddhanta::text();
    let clock = zones::nepal();
    let engine = Engine::new(&text, clock, KATHMANDU, MonthStartRule::Punyakala);
    let bs = BikramSambat::shipped();
    for row in &ROWS {
        let year = engine.year(row.year).unwrap();
        // The engine and the table agree on 1 Baisakh (every official New
        // Year is reproduced), and differ on this month by a day.
        assert_eq!(year.start, bs.year_start(row.year).unwrap());
        let computed = month_start(year.start, &year.months, row.month);
        assert_eq!(
            computed,
            day(row.other),
            "BS {} month {}",
            row.year,
            row.month
        );

        let instant = year.sankrantis[usize::from(row.month - 1)].get();
        let boundary = if row.month == 10 {
            civil_day_light(&text, clock, &KATHMANDU, day(row.other))
                .unwrap()
                .arc()
                .unwrap()
                .sunset
                .get()
        } else {
            // Midnight on Nepal's clock nearest the sankranti.
            ((instant + CLOCK_DAYS - 0.5).round() + 0.5) - CLOCK_DAYS
        };
        let margin = (instant - boundary) * 1440.0;
        assert!(
            (margin - row.margin_minutes).abs() < 0.1,
            "BS {} month {}: {margin:+.2} min, recorded {:+.1}",
            row.year,
            row.month,
            row.margin_minutes
        );
    }
}

#[test]
fn a_date_in_a_disputed_month_reports_both_labels() {
    let bs = BikramSambat::shipped();
    for row in &ROWS {
        let date = bs.date_of(day(row.table)).unwrap();
        assert_eq!((date.year, date.month, date.day), (row.year, row.month, 1));
        let CalendarResolution::Divergent {
            tabular,
            computed,
            model,
        } = &date.resolution
        else {
            panic!("BS {} month {}: {:?}", row.year, row.month, date.resolution);
        };
        assert_eq!(
            *tabular,
            MonthDay {
                month: row.month,
                day: 1
            }
        );
        // The engine's label for the same day: the 2nd when its month began
        // the day before, the previous month's last day when it begins the
        // day after.
        let expected = if day(row.other) < day(row.table) {
            MonthDay {
                month: row.month,
                day: 2,
            }
        } else {
            let previous = row.month - 1;
            let length = bs.month_length(row.year, previous).unwrap() + 1;
            MonthDay {
                month: previous,
                day: length,
            }
        };
        assert_eq!(*computed, expected, "BS {} month {}", row.year, row.month);
        assert_eq!(model, bs.frame());
    }
}

#[test]
fn the_data_file_records_the_same_four_rows() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/bikram-sambat.json");
    let data: Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("the table")).expect("JSON");
    let recorded = data["disputed_rows"].as_array().expect("disputed_rows");
    assert_eq!(recorded.len(), ROWS.len());
    let iso = |(y, m, d): (i32, u8, u8)| format!("{y:04}-{m:02}-{d:02}");
    for (entry, row) in recorded.iter().zip(&ROWS) {
        assert_eq!(entry["year"].as_i64(), Some(i64::from(row.year)));
        assert_eq!(entry["month"].as_u64(), Some(u64::from(row.month)));
        assert_eq!(entry["table_start"].as_str(), Some(iso(row.table).as_str()));
        assert_eq!(entry["other_start"].as_str(), Some(iso(row.other).as_str()));
        assert!(entry["decision"].as_str().unwrap().contains("C1850"));
    }
}
