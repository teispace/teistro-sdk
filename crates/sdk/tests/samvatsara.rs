//! The sixty-year cycle through the façade
//! (`03-design/samvatsara-measured.md`): the years Nepal named, the one
//! it expunged, and the southern count behind its knob.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::catalogue::{Calendar, Samvatsara};
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::settings::SamvatsaraCount;
use teistro::{AlmanacRequest, CalendarDate, Context, Ephemeris, LunarYear, UtcOffset};

fn context(patch: Option<&str>) -> Context {
    let builder = Context::builder()
        .profile("nepali-default")
        .ephemeris([Ephemeris::Builtin]);
    match patch {
        Some(json) => builder.settings_json(json),
        None => builder,
    }
    .build()
    .unwrap()
}

fn kathmandu() -> (Place, UtcOffset) {
    (
        Place::new(
            Latitude::literal(27.7172),
            Longitude::literal(85.324),
            Altitude::literal(1400.0),
        ),
        UtcOffset::literal(5, 45, 0),
    )
}

fn day(y: i32, m: u8, d: u8) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, y, m, d)
}

/// The one year holding a day.
fn year_of(sdk: &Context, y: i32, m: u8, d: u8) -> LunarYear {
    let (place, clock) = kathmandu();
    let years = sdk
        .almanac()
        .years(&day(y, m, d), &day(y, m, d), &place, clock)
        .unwrap();
    assert_eq!(years.value.len(), 1, "a day falls in one year");
    years.value[0].clone()
}

#[test]
fn the_years_nepal_named_are_named() {
    let sdk = context(None);
    // Mid-year days, so the year is the one the press named whichever
    // anchor it read.
    for (vikrama, name) in [
        (2076, Samvatsara::Paridhaavi),
        (2077, Samvatsara::Pramadicha),
        (2078, Samvatsara::Rakshasa),
        (2079, Samvatsara::Nala),
        (2081, Samvatsara::Kalayukta),
        (2082, Samvatsara::Siddharthi),
    ] {
        let year = year_of(&sdk, vikrama - 57, 8, 1);
        assert_eq!(year.vikrama, vikrama);
        assert_eq!(year.samvatsara, name, "VS {vikrama}");
        assert_eq!(year.count, SamvatsaraCount::Barhaspatya);
    }
}

#[test]
fn ananda_began_and_ended_inside_2077_and_names_no_year() {
    let sdk = context(None);
    let year = year_of(&sdk, 2020, 8, 1);
    assert_eq!(year.samvatsara, Samvatsara::Pramadicha);
    assert_eq!(year.lupta, Some(Samvatsara::Ananda));
    let running: Vec<Samvatsara> = year.jovian.iter().map(|jovian| jovian.member).collect();
    assert_eq!(
        running,
        [
            Samvatsara::Pramadicha,
            Samvatsara::Ananda,
            Samvatsara::Rakshasa
        ]
    );
    // The committee: Rakshasa began before the next pratipada.
    assert!(year.jovian[2].from.get() < year.ended.get());
}

#[test]
fn a_name_names_one_year_so_2080_is_pingala() {
    // Pingala began two days after VS 2080's pratipada, so Nala met it,
    // and Nala had named 2079. The committee named 2080 Pingala: a name
    // names one year (C184).
    let year = year_of(&context(None), 2023, 8, 1);
    assert_eq!(year.samvatsara, Samvatsara::Pingala);
    assert_eq!(year.jovian[0].member, Samvatsara::Nala);
    let late = year.jovian[1].from.get() - year.began.get();
    assert!((1.0..3.0).contains(&late), "{late} days");
    assert_eq!(year.lupta, None);
}

#[test]
fn the_running_reading_repeats_nala_and_expunges_pingala() {
    let sdk = context(Some(
        r#"{"calendars": {"samvatsara": "BARHASPATYA_RUNNING"}}"#,
    ));
    let names: Vec<(Samvatsara, Option<Samvatsara>)> = [2022, 2023, 2024]
        .into_iter()
        .map(|y| {
            let year = year_of(&sdk, y, 8, 1);
            (year.samvatsara, year.lupta)
        })
        .collect();
    assert_eq!(
        names,
        [
            (Samvatsara::Nala, None),
            (Samvatsara::Nala, Some(Samvatsara::Pingala)),
            (Samvatsara::Kalayukta, None),
        ]
    );
}

#[test]
fn a_range_lists_every_year_it_touches_and_they_abut() {
    let sdk = context(None);
    let (place, clock) = kathmandu();
    let years = sdk
        .almanac()
        .years(&day(2025, 3, 1), &day(2026, 2, 28), &place, clock)
        .unwrap()
        .value;
    let vikrama: Vec<i32> = years.iter().map(|year| year.vikrama).collect();
    assert_eq!(vikrama, [2081, 2082]);
    for pair in years.windows(2) {
        assert_eq!(pair[0].ended, pair[1].began);
        assert!(pair[0].opened.get() < pair[0].began.get());
    }
}

#[test]
fn the_southern_count_names_every_year_in_order() {
    let sdk = context(Some(r#"{"calendars": {"samvatsara": "CHANDRAMANA"}}"#));
    let year = year_of(&sdk, 2025, 8, 1);
    assert_eq!(year.count, SamvatsaraCount::Chandramana);
    assert_eq!(year.shaka, 1947);
    assert_eq!(year.samvatsara, Samvatsara::Vishvavasu);
    let earlier = year_of(&sdk, 2020, 8, 1);
    assert_eq!(earlier.samvatsara, Samvatsara::Sharvari);
    assert_eq!(earlier.lupta, None);
}

#[test]
fn the_years_asked_beside_the_days_are_the_years() {
    let sdk = context(None);
    let (place, clock) = kathmandu();
    let (from, to) = (day(2025, 3, 25), day(2025, 4, 5));
    let answer = sdk
        .almanac()
        .asked(
            &from,
            &to,
            &place,
            clock,
            &AlmanacRequest::new().with_years(),
        )
        .unwrap();
    let alone = sdk.almanac().years(&from, &to, &place, clock).unwrap();
    assert_eq!(answer.years.as_ref().unwrap().value, alone.value);
    // Every day falls in the year holding its sunrise.
    let years = &alone.value;
    for panchanga in &answer.days.value {
        let sunrise = panchanga.day.sunrise;
        assert_eq!(
            years.iter().filter(|year| year.contains(sunrise)).count(),
            1
        );
    }
    assert!(
        sdk.almanac()
            .asked(&from, &to, &place, clock, &AlmanacRequest::new())
            .unwrap()
            .years
            .is_none()
    );
}
