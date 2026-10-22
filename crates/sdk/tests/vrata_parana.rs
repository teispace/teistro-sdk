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
//! Navaratri 2021 to 2027 is the almanac's "Ashwina Navratri Parana" page
//! for New Delhi, which keeps *Nirnayasindhu*'s reading (the first
//! daylight of the 10th), the second `MAHA_NAVAMI` lists; 2023's 9th ends
//! a minute after the almanac's sunset, so the meal is the next morning's.
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

const DELHI: [Row; 20] = [
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
    ("MAHA_NAVAMI", "2021-10-14", 1, "2021-10-15", "06:22", "-"),
    ("MAHA_NAVAMI", "2022-10-04", 1, "2022-10-04", "14:20", "-"),
    ("MAHA_NAVAMI", "2023-10-23", 1, "2023-10-24", "06:27", "-"),
    ("MAHA_NAVAMI", "2024-10-11", 1, "2024-10-12", "10:58", "-"),
    ("MAHA_NAVAMI", "2025-10-01", 1, "2025-10-02", "06:15", "-"),
    ("MAHA_NAVAMI", "2026-10-19", 1, "2026-10-20", "12:50", "-"),
    ("MAHA_NAVAMI", "2027-10-08", 1, "2027-10-09", "09:01", "-"),
];

/// *Dharmasindhu*'s own Navaratri parana (pp. 70–71), the first reading,
/// as recomputed with pyswisseph (Moshier, Lahiri, the refracted upper
/// limb) in the research for it; no almanac in hand prints it. 2022 waits
/// for the visarjana on the later 10th; 2024 and 2027 keep the meal
/// inside the 9th left at the next sunrise, the visarjana a day later;
/// 2026's visarjana is on the earlier 10th, Shravana's last quarter
/// joining it, from the 10th's start.
const BY_THE_TEXT: [Row; 5] = [
    ("MAHA_NAVAMI", "2021-10-14", 0, "2021-10-15", "06:22", "-"),
    ("MAHA_NAVAMI", "2022-10-04", 0, "2022-10-05", "06:16", "-"),
    (
        "MAHA_NAVAMI",
        "2024-10-11",
        0,
        "2024-10-12",
        "06:20",
        "10:59",
    ),
    ("MAHA_NAVAMI", "2026-10-19", 0, "2026-10-20", "12:51", "-"),
    (
        "MAHA_NAVAMI",
        "2027-10-08",
        0,
        "2027-10-09",
        "06:18",
        "09:02",
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
    let missed = missed(&DELHI, within);
    assert!(missed.is_empty(), "{}", missed.join("\n"));
}

/// The text's own Navaratri parana falls on the day the research put it,
/// each edge within a minute of pyswisseph's (a tithi's edge being the
/// SDK's own, not an almanac's early one).
#[test]
fn the_navaratri_parana_by_the_text_waits_for_the_visarjana() {
    let missed = missed(&BY_THE_TEXT, |miss, _| miss.abs() < 1.0);
    assert!(missed.is_empty(), "{}", missed.join("\n"));
}

/// Each row that does not fall where it says, under `close` for an edge.
fn missed(rows: &[Row], close: fn(f64, VrataEdge) -> bool) -> Vec<String> {
    let delhi = Place::new(
        Latitude::literal(28.6356),
        Longitude::literal(77.2244),
        Altitude::literal(0.0),
    );
    let sdk = context();
    let observances: Vec<_> = (2021..=2028)
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
    for &(rule, fast, reading, day, from, to) in rows {
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
                close(local(end.get(), 5.5) - printed(to), by)
            }
            _ => false,
        };
        if on != date(day) || !close(a, time.from_by) || !b {
            missed.push(format!(
                "{rule} {fast} [{reading}] on {on:?}: {a:+.2} min ({:?}), to {:?} {:?}",
                time.from_by,
                time.to.map(|end| local(end.get(), 5.5) - printed(to)),
                time.to_by
            ));
        }
    }
    missed
}
