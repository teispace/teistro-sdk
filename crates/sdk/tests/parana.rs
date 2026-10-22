//! Ekadashi parana (`03-design/festival-rules.md` §8.5) held to a
//! published almanac: every Ekadashi of 2025 and 2026 that Drik Panchang
//! (rank 3) prints for New Delhi and Kathmandu, the Smarta fast and the
//! second day it gives sannyasis and Vaishnavas, each window within the
//! printed minute under *Dharmasindhu*'s rule. The rows were read off the
//! almanac's Ekadashi pages on 2026-10-22; its adhika-month pair of 2026
//! (Padmini and Parama) is not on them.
//!
//! Every row falls on the printed day, in the printed window, cut by the
//! printed clause. An edge the window sets (a sunrise, a fifth of the
//! daylight) lies within the printed minute. An edge a tithi sets (the
//! 12th's end, the hari vasara's) lies 0.5 to 2 minutes after it: the
//! almanac's tithi ends stand about 1.2 minutes before Swiss Ephemeris's,
//! which the SDK's agree with to 0.1 minute (C698).

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::catalogue::Calendar;
use teistro::festival::{EkadashiFast, ParanaEdge};
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{CalendarDate, Context, Ephemeris, FestivalPack, FestivalRequest, UtcOffset};

/// The fast's day, whether it is the Smarta's, the parana's day, and the
/// window's printed start and end, local time.
type Row = (&'static str, bool, &'static str, &'static str, &'static str);

const DELHI: [Row; 55] = [
    ("2025-01-25", true, "2025-01-26", "07:12", "09:21"),
    ("2025-02-08", true, "2025-02-09", "07:04", "09:17"),
    ("2025-02-24", true, "2025-02-25", "06:50", "09:08"),
    ("2025-03-10", true, "2025-03-11", "06:35", "08:13"),
    ("2025-03-25", true, "2025-03-26", "13:41", "16:08"),
    ("2025-03-26", false, "2025-03-27", "06:17", "08:45"),
    ("2025-04-08", true, "2025-04-09", "06:02", "08:34"),
    ("2025-04-24", true, "2025-04-25", "05:46", "08:23"),
    ("2025-06-06", true, "2025-06-07", "13:44", "16:31"),
    ("2025-06-07", false, "2025-06-08", "05:23", "07:17"),
    ("2025-10-17", true, "2025-10-18", "06:24", "08:41"),
    ("2025-01-10", true, "2025-01-11", "07:15", "08:21"),
    ("2025-05-08", true, "2025-05-09", "05:34", "08:16"),
    ("2025-05-23", true, "2025-05-24", "05:26", "08:11"),
    ("2025-06-21", true, "2025-06-22", "13:47", "16:35"),
    ("2025-06-22", false, "2025-06-23", "05:24", "08:12"),
    ("2025-07-06", true, "2025-07-07", "05:29", "08:16"),
    ("2025-07-21", true, "2025-07-22", "05:37", "07:05"),
    ("2025-08-05", true, "2025-08-06", "05:45", "08:26"),
    ("2025-08-19", true, "2025-08-20", "05:53", "08:29"),
    ("2025-09-03", true, "2025-09-04", "13:36", "16:07"),
    ("2025-09-17", true, "2025-09-18", "06:07", "08:34"),
    ("2025-10-03", true, "2025-10-04", "06:16", "08:37"),
    ("2025-11-01", true, "2025-11-02", "13:11", "15:23"),
    ("2025-11-02", false, "2025-11-03", "06:34", "08:46"),
    ("2025-11-15", true, "2025-11-16", "13:10", "15:18"),
    ("2025-12-01", true, "2025-12-02", "06:57", "09:03"),
    ("2025-12-15", true, "2025-12-16", "07:07", "09:11"),
    ("2025-12-30", true, "2025-12-31", "13:26", "15:31"),
    ("2025-12-31", false, "2026-01-01", "07:14", "09:18"),
    ("2026-01-14", true, "2026-01-15", "07:15", "09:21"),
    ("2026-01-29", true, "2026-01-30", "07:10", "09:20"),
    ("2026-02-13", true, "2026-02-14", "07:00", "09:14"),
    ("2026-02-27", true, "2026-02-28", "06:47", "09:06"),
    ("2026-03-15", true, "2026-03-16", "06:30", "08:54"),
    ("2026-03-29", true, "2026-03-30", "06:14", "07:09"),
    ("2026-04-13", true, "2026-04-14", "06:54", "08:31"),
    ("2026-04-27", true, "2026-04-28", "05:43", "08:21"),
    ("2026-05-13", true, "2026-05-14", "05:31", "08:14"),
    ("2026-06-25", true, "2026-06-26", "05:25", "08:13"),
    ("2026-07-10", true, "2026-07-11", "13:50", "16:36"),
    ("2026-07-11", false, "2026-07-12", "05:32", "08:18"),
    ("2026-07-25", true, "2026-07-26", "05:39", "08:22"),
    ("2026-08-09", true, "2026-08-10", "05:47", "08:00"),
    ("2026-08-23", true, "2026-08-24", "13:41", "16:16"),
    ("2026-08-24", false, "2026-08-25", "05:55", "06:20"),
    ("2026-09-07", true, "2026-09-08", "06:02", "08:33"),
    ("2026-09-22", true, "2026-09-23", "06:10", "08:35"),
    ("2026-10-06", true, "2026-10-07", "06:17", "08:38"),
    ("2026-10-22", true, "2026-10-23", "06:27", "08:42"),
    ("2026-11-05", true, "2026-11-06", "06:37", "08:48"),
    ("2026-11-20", true, "2026-11-21", "13:11", "15:18"),
    ("2026-11-21", false, "2026-11-22", "06:49", "08:56"),
    ("2026-12-04", true, "2026-12-05", "06:59", "09:04"),
    ("2026-12-20", true, "2026-12-21", "07:10", "09:13"),
];

const KATHMANDU: [Row; 38] = [
    ("2025-01-10", true, "2025-01-11", "06:56", "08:36"),
    ("2025-01-25", true, "2025-01-26", "06:53", "09:02"),
    ("2025-02-08", true, "2025-02-09", "06:46", "08:59"),
    ("2025-02-24", true, "2025-02-25", "06:32", "08:50"),
    ("2025-03-10", true, "2025-03-11", "06:18", "08:28"),
    ("2025-03-25", true, "2025-03-26", "13:23", "15:51"),
    ("2025-03-26", false, "2025-03-27", "06:00", "08:28"),
    ("2025-04-08", true, "2025-04-09", "05:45", "08:17"),
    ("2025-04-24", true, "2025-04-25", "05:29", "08:06"),
    ("2025-05-08", true, "2025-05-09", "05:18", "07:59"),
    ("2025-05-23", true, "2025-05-24", "05:11", "07:55"),
    ("2025-06-06", true, "2025-06-07", "13:26", "16:12"),
    ("2025-06-07", false, "2025-06-08", "05:07", "07:32"),
    ("2025-06-21", true, "2025-06-22", "13:29", "16:16"),
    ("2025-06-22", false, "2025-06-23", "05:09", "07:56"),
    ("2025-07-06", true, "2025-07-07", "05:14", "08:00"),
    ("2025-07-21", true, "2025-07-22", "05:21", "07:20"),
    ("2025-08-05", true, "2025-08-06", "05:29", "08:09"),
    ("2025-08-19", true, "2025-08-20", "05:37", "08:13"),
    ("2025-09-03", true, "2025-09-04", "13:18", "15:50"),
    ("2025-09-17", true, "2025-09-18", "05:50", "08:17"),
    ("2025-10-03", true, "2025-10-04", "05:58", "08:20"),
    ("2025-10-17", true, "2025-10-18", "06:05", "08:23"),
    ("2025-11-01", true, "2025-11-02", "13:10", "15:06"),
    ("2025-11-02", false, "2025-11-03", "06:16", "08:28"),
    ("2025-11-15", true, "2025-11-16", "12:53", "15:02"),
    ("2025-12-01", true, "2025-12-02", "06:38", "08:44"),
    ("2025-12-15", true, "2025-12-16", "06:47", "08:52"),
    ("2025-12-30", true, "2025-12-31", "13:09", "15:14"),
    ("2025-12-31", false, "2026-01-01", "06:55", "09:00"),
    ("2026-01-14", true, "2026-01-15", "06:56", "09:03"),
    ("2026-01-29", true, "2026-01-30", "06:52", "09:02"),
    ("2026-02-13", true, "2026-02-14", "06:42", "08:56"),
    ("2026-02-27", true, "2026-02-28", "06:29", "08:48"),
    ("2026-03-15", true, "2026-03-16", "06:12", "08:36"),
    ("2026-03-29", true, "2026-03-30", "05:57", "07:24"),
    ("2026-04-13", true, "2026-04-14", "07:09", "08:14"),
    ("2026-04-27", true, "2026-04-28", "05:27", "08:05"),
];

/// The almanac's sunrise is the upper limb's, refracted, where the
/// SDK's default is the disc's centre unrefracted.
fn context() -> Context {
    Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .settings_json(
            r#"{"day": {"sunrise": {"kind": "NAMED", "which": "UPPER_LIMB_REFRACTION"}}}"#,
        )
        .build()
        .unwrap()
}

fn date(text: &str) -> (i32, u8, u8) {
    let mut parts = text.split('-').map(|part| part.parse::<i32>().unwrap());
    let (year, month, day) = (
        parts.next().unwrap(),
        parts.next().unwrap(),
        parts.next().unwrap(),
    );
    (
        year,
        u8::try_from(month).unwrap(),
        u8::try_from(day).unwrap(),
    )
}

/// Minutes past local midnight of a printed `HH:MM`.
fn printed(text: &str) -> f64 {
    let (hours, minutes) = text.split_once(':').unwrap();
    f64::from(hours.parse::<u32>().unwrap() * 60 + minutes.parse::<u32>().unwrap())
}

/// Minutes past local midnight of an instant, on the parana's own day.
fn local(jd: f64, offset_hours: f64) -> f64 {
    let day = jd + 0.5 + offset_hours / 24.0;
    (day - day.floor()) * 1440.0
}

/// Whether an edge lies where C698 puts it: a window's within the
/// printed minute, a tithi's 0.5 to 2 minutes after it.
fn within(miss: f64, by: ParanaEdge) -> bool {
    match by {
        ParanaEdge::Window => miss.abs() < 1.0,
        _ => (0.5..2.0).contains(&miss),
    }
}

/// Every printed window's misses, in minutes, start and end, under the
/// fasts the shipped rules give over the two years.
fn misses(place: &Place, clock: UtcOffset, offset_hours: f64, rows: &[Row]) -> Vec<String> {
    let sdk = context();
    let fasts: Vec<EkadashiFast> = [2025, 2026]
        .into_iter()
        .flat_map(|year| {
            sdk.almanac()
                .festivals(
                    &CalendarDate::defined(Calendar::Gregorian, year, 1, 1),
                    &CalendarDate::defined(Calendar::Gregorian, year, 12, 31),
                    place,
                    clock,
                    &FestivalRequest::from(FestivalPack::Dharmasindhu),
                )
                .unwrap()
                .value
                .ekadashis
        })
        .collect();
    let mut out = Vec::new();
    for &(fast, smarta, parana_day, from, to) in rows {
        let on = date(fast);
        let kept: Vec<&EkadashiFast> = fasts
            .iter()
            .filter(|f| (f.day.year, f.day.month, f.day.day) == on)
            .filter(|f| (f.rule == "EKADASHI_SMARTA") == smarta)
            .collect();
        if kept.is_empty() {
            out.push(format!(
                "{fast}: no {} fast that day",
                if smarta { "Smarta" } else { "second-day" }
            ));
            continue;
        }
        for f in kept {
            let parana = f.parana.as_ref().unwrap();
            let day = (parana.day.year, parana.day.month, parana.day.day);
            let Some(kept) = parana.kept else {
                out.push(format!("{fast} {}: no window", f.rule));
                continue;
            };
            let (a, b) = (
                local(kept.span.from.get(), offset_hours) - printed(from),
                local(kept.span.to.get(), offset_hours) - printed(to),
            );
            if day != date(parana_day) || !within(a, kept.from_by) || !within(b, kept.to_by) {
                out.push(format!(
                    "{fast} {} on {:?}: {a:+.2} {b:+.2} min ({:?} {:?} {:?})",
                    f.rule, day, kept.at, kept.from_by, kept.to_by
                ));
            }
        }
    }
    out
}

#[test]
fn every_printed_parana_at_delhi_is_the_rule_s_window() {
    let delhi = Place::new(
        Latitude::literal(28.6356),
        Longitude::literal(77.2244),
        Altitude::literal(0.0),
    );
    let missed = misses(&delhi, UtcOffset::literal(5, 30, 0), 5.5, &DELHI);
    assert!(missed.is_empty(), "{}", missed.join("\n"));
}

#[test]
fn every_printed_parana_at_kathmandu_is_the_rule_s_window() {
    let kathmandu = Place::new(
        Latitude::literal(27.7017),
        Longitude::literal(85.3206),
        Altitude::literal(0.0),
    );
    let missed = misses(&kathmandu, UtcOffset::literal(5, 45, 0), 5.75, &KATHMANDU);
    assert!(missed.is_empty(), "{}", missed.join("\n"));
}
