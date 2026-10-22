//! A vrata's parana (`03-design/festival-rules.md` §8.7) held to a
//! published almanac: Janmashtami 2023 to 2027 and Shivaratri 2023 to
//! 2028 as Drik Panchang (rank 3) prints them for New Delhi, read off its
//! festival pages on 2026-10-22, under *Dharmasindhu*'s readings.
//!
//! Every row falls on the printed day. An edge the day sets (a sunrise,
//! three yamas) or a nakshatra's end lies within the printed minute; an
//! edge a tithi sets lies 0.5 to 2 minutes after it, the almanac's tithi
//! ends standing about 1.2 minutes before Swiss Ephemeris's (C698). Its
//! nakshatra ends do not: Rohini's on 27 August 2024 is 0.1 minute from
//! the SDK's.
//!
//! The 2027 Janmashtami page read was the 25 August fast, the Vaishnava
//! day of the government's list, where *Dharmasindhu*'s Smarta day is the
//! 24th (`festival-measured.md` §1, C175); it is not a row here.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::catalogue::Calendar;
use teistro::festival::VrataEdge;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{CalendarDate, Context, Ephemeris, FestivalPack, FestivalRequest, UtcOffset};

/// The rule, the fast's day, which of its readings, the parana's day,
/// and the printed start and end (`-` for none), local time.
type Row = (
    &'static str,
    &'static str,
    usize,
    &'static str,
    &'static str,
    &'static str,
);

const DELHI: [Row; 13] = [
    // "Parana as per Dharma Shastra", then the alternate at sunrise.
    ("JANMASHTAMI", "2023-09-06", 0, "2023-09-07", "16:14", "-"),
    ("JANMASHTAMI", "2023-09-06", 1, "2023-09-07", "06:02", "-"),
    ("JANMASHTAMI", "2024-08-26", 0, "2024-08-27", "15:38", "-"),
    ("JANMASHTAMI", "2024-08-26", 1, "2024-08-27", "05:57", "-"),
    ("JANMASHTAMI", "2025-08-15", 0, "2025-08-16", "21:34", "-"),
    ("JANMASHTAMI", "2025-08-15", 1, "2025-08-16", "05:51", "-"),
    ("JANMASHTAMI", "2026-09-04", 0, "2026-09-05", "06:01", "-"),
    (
        "SHIVARATRI",
        "2023-02-18",
        0,
        "2023-02-19",
        "06:56",
        "15:24",
    ),
    (
        "SHIVARATRI",
        "2024-03-08",
        0,
        "2024-03-09",
        "06:37",
        "15:29",
    ),
    (
        "SHIVARATRI",
        "2025-02-26",
        0,
        "2025-02-27",
        "06:48",
        "08:54",
    ),
    (
        "SHIVARATRI",
        "2026-02-15",
        0,
        "2026-02-16",
        "06:59",
        "15:24",
    ),
    (
        "SHIVARATRI",
        "2027-03-06",
        0,
        "2027-03-07",
        "06:40",
        "13:46",
    ),
    (
        "SHIVARATRI",
        "2028-02-23",
        0,
        "2028-02-24",
        "06:52",
        "13:26",
    ),
];

/// The almanac's sunrise is the upper limb's, refracted (§8.6).
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

/// Minutes past local midnight of an instant.
fn local(jd: f64, offset_hours: f64) -> f64 {
    let day = jd + 0.5 + offset_hours / 24.0;
    (day - day.floor()) * 1440.0
}

/// Whether an edge lies where C698 puts it.
fn within(miss: f64, by: VrataEdge) -> bool {
    match by {
        VrataEdge::TithiEnd => (0.5..2.0).contains(&miss),
        _ => miss.abs() < 1.0,
    }
}

#[test]
fn every_printed_vrata_parana_at_delhi_is_the_reading_s() {
    let delhi = Place::new(
        Latitude::literal(28.6356),
        Longitude::literal(77.2244),
        Altitude::literal(0.0),
    );
    let sdk = context();
    let observances: Vec<_> = (2023..=2028)
        .flat_map(|year| {
            sdk.almanac()
                .festivals(
                    &CalendarDate::defined(Calendar::Gregorian, year, 1, 1),
                    &CalendarDate::defined(Calendar::Gregorian, year, 12, 31),
                    &delhi,
                    UtcOffset::literal(5, 30, 0),
                    &FestivalRequest::from(FestivalPack::Dharmasindhu),
                )
                .unwrap()
                .value
                .observances
        })
        .collect();
    let mut missed = Vec::new();
    for &(rule, fast, reading, day, from, to) in &DELHI {
        let Some(kept) = observances
            .iter()
            .find(|o| o.rule == rule && (o.day.year, o.day.month, o.day.day) == date(fast))
        else {
            missed.push(format!("{rule} {fast}: not kept that day"));
            continue;
        };
        let time = &kept.parana[reading];
        let on = (time.day.year, time.day.month, time.day.day);
        let a = local(time.from.get(), 5.5) - printed(from);
        let b = match (time.to, time.to_by, to) {
            (None, None, "-") => true,
            (Some(end), Some(by), to) if to != "-" => {
                within(local(end.get(), 5.5) - printed(to), by)
            }
            _ => false,
        };
        if on != date(day) || !within(a, time.from_by) || !b {
            missed.push(format!(
                "{rule} {fast} [{reading}] on {on:?}: {a:+.2} min ({:?}), to {:?} {:?}",
                time.from_by,
                time.to.map(|end| local(end.get(), 5.5) - printed(to)),
                time.to_by
            ));
        }
    }
    assert!(missed.is_empty(), "{}", missed.join("\n"));
}
