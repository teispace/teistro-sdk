//! The `GAUDIYA` pack's festivals (`03-design/festival-rules.md` §8.10)
//! held to a published Gaudiya calendar (rank 3, the output of `GCal`): Drik
//! Panchang's ISKCON lists for New Delhi and New York, read 2026-11-01,
//! Janmashtami's for 2020 to 2030 and the other four for 2023 to 2026.
//!
//! The two places part where the clock moves a tithi's start across a
//! sunrise: Janmashtami 2023 begins before New York's sunrise of 6
//! September and after Delhi's, so the lists keep the 6th there and the
//! 7th here, as the rule does. The lists' sunrise is the refracted upper
//! limb's, as in `mahadvadashi.rs`, and the coordinates are the lists'.

#![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

use std::collections::BTreeMap;

use teistro::catalogue::Calendar;
use teistro::festival::{FastEnd, Observances};
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{CalendarDate, Context, Ephemeris, FestivalPack, FestivalRequest, UtcOffset};

/// A published day: the rule's key and (year, month, day).
type Row = (&'static str, (i32, u8, u8));

/// The five keys, in the order the rows below give them each year.
const KEYS: [&str; 5] = [
    "GAURA_PURNIMA",
    "RAMA_NAVAMI_VAISHNAVA",
    "NRISIMHA_CHATURDASHI",
    "JANMASHTAMI_VAISHNAVA",
    "RADHASHTAMI_VAISHNAVA",
];

/// Each year's five days at a place, as (month, day) in [`KEYS`]' order.
type Year = (i32, [(u8, u8); 5]);

/// New Delhi, 2023 to 2026.
const DELHI: [Year; 4] = [
    (2023, [(3, 7), (3, 30), (5, 4), (9, 7), (9, 23)]),
    (2024, [(3, 25), (4, 17), (5, 22), (8, 26), (9, 11)]),
    (2025, [(3, 14), (4, 6), (5, 11), (8, 16), (8, 31)]),
    (2026, [(3, 3), (3, 27), (4, 30), (9, 4), (9, 19)]),
];

/// New York, 2023 to 2026.
const NEW_YORK: [Year; 4] = [
    (2023, [(3, 6), (3, 30), (5, 4), (9, 6), (9, 22)]),
    (2024, [(3, 24), (4, 16), (5, 22), (8, 26), (9, 11)]),
    (2025, [(3, 13), (4, 6), (5, 11), (8, 16), (8, 31)]),
    (2026, [(3, 3), (3, 26), (4, 30), (9, 4), (9, 18)]),
];

/// A year's Janmashtami: the year, then (month, day) at Delhi and at New
/// York.
type Janmashtami = (i32, (u8, u8), (u8, u8));

/// Janmashtami's lists, 2020 to 2030.
const JANMASHTAMI: [Janmashtami; 11] = [
    (2020, (8, 12), (8, 11)),
    (2021, (8, 30), (8, 30)),
    (2022, (8, 19), (8, 19)),
    (2023, (9, 7), (9, 6)),
    (2024, (8, 26), (8, 26)),
    (2025, (8, 16), (8, 16)),
    (2026, (9, 4), (9, 4)),
    (2027, (8, 25), (8, 25)),
    (2028, (8, 13), (8, 13)),
    (2029, (9, 1), (9, 1)),
    (2030, (8, 21), (8, 21)),
];

/// The lists' sunrise is the upper limb's, refracted.
fn context() -> Context {
    Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .settings_json(
            r#"{"day": {"sunrise": {"kind": "NAMED", "which": "UPPER_LIMB_REFRACTION"}}}"#,
        )
        .build()
        .unwrap()
}

fn place(latitude: f64, longitude: f64) -> Place {
    Place::new(
        Latitude::literal(latitude),
        Longitude::literal(longitude),
        Altitude::literal(0.0),
    )
}

/// The lists' New Delhi, on India's clock.
fn delhi() -> (Place, UtcOffset) {
    (place(28.6356, 77.2244), UtcOffset::literal(5, 30, 0))
}

/// The lists' New York, on its daylight clock: a sunrise falls on the same
/// civil day on either of its clocks.
fn new_york() -> (Place, UtcOffset) {
    (place(40.7142, -74.0058), UtcOffset::literal(-4, 0, 0))
}

/// The pack's festivals from February to October of `year`.
fn festivals(sdk: &Context, (place, offset): &(Place, UtcOffset), year: i32) -> Observances {
    let date = |month, day| CalendarDate::defined(Calendar::Gregorian, year, month, day);
    let found = sdk
        .almanac()
        .festivals(
            &date(2, 1),
            &date(10, 1),
            place,
            *offset,
            &FestivalRequest::new(FestivalPack::Gaudiya.rules()),
        )
        .unwrap()
        .value;
    assert!(found.unjudged.is_empty(), "{year}: {:?}", found.unjudged);
    found
}

/// Each row the pack does not keep on its day, with the day it keeps.
fn missed(found: &BTreeMap<i32, Observances>, rows: &[Row]) -> Vec<String> {
    rows.iter()
        .filter_map(|&(key, (year, month, day))| {
            let kept: Vec<_> = found[&year]
                .observances
                .iter()
                .filter(|observance| observance.rule == key)
                .map(|observance| (observance.day.month, observance.day.day))
                .collect();
            (kept != [(month, day)])
                .then(|| format!("{year} {key}: listed {month}/{day}, kept {kept:?}"))
        })
        .collect()
}

/// The rows of a place's years, with Janmashtami's further years.
fn rows(years: &[Year], janmashtami: impl Fn(&Janmashtami) -> (u8, u8)) -> Vec<Row> {
    let mut rows: Vec<Row> = years
        .iter()
        .flat_map(|&(year, days)| {
            KEYS.into_iter()
                .zip(days)
                .map(move |(key, (month, day))| (key, (year, month, day)))
        })
        .collect();
    rows.extend(
        JANMASHTAMI
            .iter()
            .filter(|row| !years.iter().any(|&(year, _)| year == row.0))
            .map(|row| {
                let (month, day) = janmashtami(row);
                ("JANMASHTAMI_VAISHNAVA", (row.0, month, day))
            }),
    );
    rows
}

fn held_at(at: &(Place, UtcOffset), rows: &[Row]) -> Vec<String> {
    let sdk = context();
    let found: BTreeMap<i32, Observances> = rows
        .iter()
        .map(|row| row.1.0)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .map(|year| (year, festivals(&sdk, at, year)))
        .collect();
    missed(&found, rows)
}

#[test]
fn the_gaudiya_days_are_the_published_calendar_s_at_delhi() {
    let missed = held_at(&delhi(), &rows(&DELHI, |row| row.1));
    assert!(missed.is_empty(), "{missed:#?}");
}

#[test]
fn the_gaudiya_days_are_the_published_calendar_s_at_new_york() {
    let missed = held_at(&new_york(), &rows(&NEW_YORK, |row| row.2));
    assert!(missed.is_empty(), "{missed:#?}");
}

/// The Smarta's Janmashtami stays *Dharmasindhu*'s: in 2025 at Delhi the
/// 8th holds the 15th's nishitha alone, so the Smarta fasts then, and the
/// Vaishnava on the 16th, whose sunrise the 8th holds.
#[test]
fn the_smarta_and_the_vaishnava_janmashtami_part_where_the_lists_do() {
    let sdk = context();
    let (place, offset) = delhi();
    let date = |day| CalendarDate::defined(Calendar::Gregorian, 2025, 8, day);
    let request =
        FestivalRequest::from(FestivalPack::Dharmasindhu).with_pack(FestivalPack::Gaudiya);
    let found = sdk
        .almanac()
        .festivals(&date(1), &date(31), &place, offset, &request)
        .unwrap()
        .value;
    let day = |key: &str| {
        found
            .observances
            .iter()
            .find(|observance| observance.rule == key)
            .map(|observance| observance.day.day)
    };
    assert_eq!(day("JANMASHTAMI"), Some(15));
    assert_eq!(day("JANMASHTAMI_VAISHNAVA"), Some(16));
    let vaishnava = found
        .observances
        .iter()
        .find(|observance| observance.rule == "JANMASHTAMI_VAISHNAVA")
        .unwrap();
    // HBV 15.385: the parana the day after, at the 8th's and Rohini's end.
    assert_ne!(vaishnava.parana.len(), 0);
}

/// Each festival's fast, as the texts and the calendar give it: Gaura
/// Purnima's broken at moonrise, so its answer carries the moonrise.
#[test]
fn each_gaudiya_festival_carries_its_fast() {
    let listed = FestivalPack::Gaudiya.listing();
    let row = |key: &str| listed.rules.iter().find(|row| row.key == key).unwrap();
    for key in KEYS.into_iter().chain(["EKADASHI_GAUDIYA"]) {
        assert!(!row(key).meta.fast.is_empty(), "{key}");
    }
    assert_eq!(
        row("GAURA_PURNIMA")
            .meta
            .fast
            .first()
            .and_then(|fast| fast.end),
        Some(FastEnd::Moonrise)
    );
    assert_eq!(row("EKADASHI_GAUDIYA").meta.fast.len(), 7);
    let found = festivals(&context(), &delhi(), 2025);
    let gaura = found
        .observances
        .iter()
        .find(|observance| observance.rule == "GAURA_PURNIMA")
        .unwrap();
    assert!(gaura.moonrise.is_some());
}
