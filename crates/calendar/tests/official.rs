//! The national panchanga committee's own sankranti instants and month
//! starts for BS 2082 and 2083 (`fixtures/official/npns-2082-2083.json`,
//! read from its published panchangas) against the SDK's engine over the
//! text: the instants within two minutes, every month start on the day
//! the committee printed it, including a Makara sankranti at 03:23 and
//! four ordinary ones between 01:46 and 04:19.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "tests fail by panicking and read a small fixture"
)]

use std::path::Path;

use serde_json::Value;
use teistro_calendar::bikram_sambat::{Engine, KATHMANDU};
use teistro_calendar::{BikramSambat, CalendarSystem, FixedDay, MonthStartRule};
use teistro_siddhanta::SuryaSiddhanta;
use teistro_time::zones;

/// Nepal's clock, in days.
const NPT_DAYS: f64 = 5.75 / 24.0;

/// The committee prints its instants to the minute and the SDK's Sun
/// agrees with its printed places within three arcseconds, a minute and a
/// quarter of the Sun's motion.
const INSTANT_TOLERANCE_MINUTES: f64 = 2.0;

fn fixture() -> Value {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/official/npns-2082-2083.json");
    serde_json::from_str(&std::fs::read_to_string(path).expect("the committee fixture"))
        .expect("valid JSON")
}

/// A printed time `H:MM`, hours running past 24, as (whole days, day fraction).
fn printed_time(text: &str) -> (i64, f64) {
    let (h, m) = text.split_once(':').expect("H:MM");
    let hours: i64 = h.parse().expect("hours");
    let minutes: f64 = m.parse().expect("minutes");
    (
        hours / 24,
        (f64::from(u8::try_from(hours % 24).unwrap()) * 60.0 + minutes) / 1440.0,
    )
}

#[test]
fn the_engine_reproduces_the_committees_sankrantis_and_month_starts() {
    let fixture = fixture();
    let text = SuryaSiddhanta::text();
    let table = BikramSambat::shipped();
    let engine = Engine::new(&text, zones::nepal(), KATHMANDU, MonthStartRule::Punyakala);
    let mut worst_minutes = 0.0f64;
    let mut compared = 0;
    for entry in fixture["sankrantis"].as_array().unwrap() {
        let year = entry["bs_year"].as_i64().unwrap() as i32;
        let printed_month = entry["printed_month"].as_u64().unwrap() as u8;
        let gate = entry["printed_gate"].as_u64().unwrap() as u8;
        let sign = entry["sign_index"].as_u64().unwrap() as usize;
        let (extra_days, fraction) = printed_time(entry["printed_time"].as_str().unwrap());
        // The printed day is a date of the official calendar; a time past 24
        // hours is the small hours of the following civil day.
        let gate_day: FixedDay = table
            .to_fixed_ymd(year, printed_month, gate)
            .expect("an official date");
        let printed_day = gate_day.plus_days(extra_days);
        let committee_utc = printed_day.jd_at_midnight().unwrap().get() + fraction - NPT_DAYS;

        let row = engine.year(year).expect("the year");
        let ours_utc = row.sankrantis[sign].get();
        let minutes = (ours_utc - committee_utc) * 1440.0;
        assert!(
            minutes.abs() < INSTANT_TOLERANCE_MINUTES,
            "BS {year} sign {sign}: SDK {ours_utc} against the committee {committee_utc}: {minutes:+.1} min"
        );
        worst_minutes = worst_minutes.max(minutes.abs());

        // The month the sankranti begins (Mesha begins Baisakh) starts on
        // gate 1 of the new month when the committee printed the sankranti
        // there, and on the day after a gate 30 or 31 otherwise: the civil
        // day of a time past midnight, or the following day for a Makara
        // sankranti after sunset (21:10 in 2082).
        let expected = if gate == 1 {
            gate_day
        } else {
            gate_day.plus_days(1)
        };
        let start: FixedDay = row
            .start
            .plus_days(row.months[..sign].iter().map(|m| i64::from(*m)).sum());
        assert_eq!(start, expected, "BS {year} month {}", sign + 1);
        compared += 1;
    }
    assert_eq!(compared, 24);
    assert!(
        worst_minutes < INSTANT_TOLERANCE_MINUTES,
        "worst {worst_minutes:.1} min"
    );

    // The month lengths under the rule are the official rows for both years.
    for year in [2082, 2083] {
        let official = table
            .official_rows()
            .into_iter()
            .find(|(y, _)| *y == year)
            .map(|(_, months)| months)
            .expect("an official row");
        assert_eq!(engine.year(year).unwrap().months, official, "BS {year}");
    }
}
