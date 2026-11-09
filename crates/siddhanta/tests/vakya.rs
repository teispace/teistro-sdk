//! The Manonmani Vilasam vakya almanac's printed tithi and nakshatra
//! ends (`tests/data/manonmani-vakya.txt`) against the *Vakyakarana*'s
//! Sun and Moon with the almanacs' manyadi (`docs/calendars/vakya.md`).
//!
//! The bound is the measured one: over 127 printed ends in 1956, 1998
//! and 2015 the model is within 10 vinadis of the page (root mean square
//! about 3), the slip the page's own karana corrects aside (C1291). The
//! text's Moon without the manyadi misses the same nakshatra ends by 29
//! to 92 vinadis, twelve minutes to more than half an hour.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::cast_precision_loss,
    reason = "tests fail by panicking and read a small data file"
)]

use std::path::Path;

use teistro_calendar::gregorian::fixed_from_gregorian;
use teistro_siddhanta::vakya::{self, ARC_PER_CIRCLE, KALI_DAY_JD, LimbEnd, PARTS_PER_DAY};

/// The bound on a printed end, in vinadis: the largest measured, 10,
/// with two to spare (C1295).
const BOUND_VINADIS: i64 = 12;

/// The Julian day number at noon of the calendar crate's fixed day 0.
const FIXED_ZERO_JD: i64 = 1_721_425;

/// One printed end.
struct Printed {
    date: String,
    day: i64,
    nakshatra: bool,
    number: u8,
    /// Vinadis after sunrise, or after the day's first end when `after_first`.
    vinadis: i64,
    after_first: bool,
}

fn printed() -> Vec<Printed> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/manonmani-vakya.txt");
    std::fs::read_to_string(path)
        .expect("the data file")
        .lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .map(|line| {
            let fields: Vec<&str> = line.split_whitespace().collect();
            let mut ymd = fields[0].split('-');
            let year: i32 = ymd.next().unwrap().parse().unwrap();
            let month: u8 = ymd.next().unwrap().parse().unwrap();
            let day: u8 = ymd.next().unwrap().parse().unwrap();
            let fixed = fixed_from_gregorian(year, month, day).get();
            let end = fields[3];
            let after_first = end.starts_with('+');
            let (nadis, vinadis) = end.trim_start_matches('+').split_once('-').unwrap();
            Printed {
                date: fields[0].to_owned(),
                day: fixed + FIXED_ZERO_JD - KALI_DAY_JD,
                nakshatra: fields[1] == "nakshatra",
                number: fields[2].parse().unwrap(),
                vinadis: nadis.parse::<i64>().unwrap() * 60 + vinadis.parse::<i64>().unwrap(),
                after_first,
            }
        })
        .collect()
}

/// The model's ends for a printed row's day and limb.
fn model(row: &Printed) -> Vec<LimbEnd> {
    if row.nakshatra {
        vakya::nakshatra_ends(row.day)
    } else {
        vakya::tithi_ends(row.day)
    }
}

/// Vinadis after the day's sunrise, quarters kept as a fraction.
fn vinadis(end: &LimbEnd, day: i64) -> f64 {
    (end.at.parts() - (day - 1) * PARTS_PER_DAY) as f64 / 4.0
}

/// Each printed end and the model's, in vinadis as the page counts them.
fn residuals() -> Vec<(String, f64)> {
    printed()
        .iter()
        .map(|row| {
            let ends = model(row);
            let at = ends
                .iter()
                .position(|end| end.number == row.number)
                .unwrap_or_else(|| panic!("{}: no end of {} in {ends:?}", row.date, row.number));
            let mut computed = vinadis(&ends[at], row.day);
            if row.after_first {
                computed -= vinadis(&ends[at - 1], row.day);
            }
            (
                format!("{} {}", row.date, row.number),
                computed - row.vinadis as f64,
            )
        })
        .collect()
}

#[test]
fn every_printed_end_is_the_models_within_the_bound() {
    let all = residuals();
    assert_eq!(all.len(), 127);
    let (slip, rest): (Vec<_>, Vec<_>) = all
        .into_iter()
        .partition(|(what, _)| what == "2015-10-30 18");
    // The slip: 16-24 printed, 16-02 by the row's own karana.
    assert!((slip[0].1 + 22.0).abs() <= BOUND_VINADIS as f64, "{slip:?}");
    let worst = rest.iter().map(|(_, r)| r.abs()).fold(0.0, f64::max);
    let rms = (rest.iter().map(|(_, r)| r * r).sum::<f64>() / rest.len() as f64).sqrt();
    assert!(worst <= BOUND_VINADIS as f64, "worst {worst}: {rest:?}");
    assert!(rms < 4.0, "rms {rms}");
}

#[test]
fn every_printed_day_ends_as_many_limbs_as_the_model() {
    for row in printed() {
        let count = printed()
            .iter()
            .filter(|other| other.date == row.date && other.nakshatra == row.nakshatra)
            .count();
        assert_eq!(model(&row).len(), count, "{} {}", row.date, row.number);
    }
}

#[test]
fn without_the_manyadi_the_moon_misses_by_ten_minutes_and_more() {
    // The nakshatras by the text's Moon alone: every single-end day's
    // end moves by 29 vinadis or more (the least, 1998-07-16, in Mithuna
    // where the manyadi is 7′ to 10′), so the bound is 25, ten minutes.
    let size = ARC_PER_CIRCLE / 27;
    for row in printed()
        .iter()
        .filter(|row| row.nakshatra && !row.after_first)
    {
        let (first, next) = (vakya::true_moon(row.day - 1), vakya::true_moon(row.day));
        let travel = next.since(first).quarter_seconds();
        let boundary = i64::from(row.number) * size;
        let gone = (boundary - first.quarter_seconds()).rem_euclid(ARC_PER_CIRCLE);
        let at = gone as f64 / travel as f64 * 3600.0;
        assert!((at - row.vinadis as f64).abs() > 25.0, "{}: {at}", row.date);
    }
}
