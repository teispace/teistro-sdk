//! The season through the façade (`03-design/ritu-measured.md`): the days
//! Nepal's daily panchanga changed season on, and the two rival readings
//! behind the knob.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::catalogue::{Calendar, Ritu};
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{CalendarDate, Context, Ephemeris, UtcOffset};

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

/// The season of a Gregorian day at Kathmandu.
fn ritu_on(sdk: &Context, (y, m, d): (i32, u8, u8)) -> Ritu {
    let place = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );
    let date = CalendarDate::defined(Calendar::Gregorian, y, m, d);
    let days = sdk
        .almanac()
        .of(&date, &date, &place, UtcOffset::literal(5, 45, 0))
        .unwrap();
    days.value[0].sun.ritu
}

#[test]
fn the_seasons_change_where_nepal_changed_them() {
    let sdk = context(None);
    // Nepal's daily panchanga: each season from a Bikram Sambat month's
    // first day. 15 May 2026's sankranti fell after sunrise, and 16 July
    // 2026's modern one before the next dawn, where the Surya Siddhanta's
    // fell a day later.
    for (day, ritu) in [
        ((2025, 11, 16), Ritu::Sharad),
        ((2025, 11, 17), Ritu::Hemanta),
        ((2026, 3, 11), Ritu::Shishira),
        ((2026, 3, 22), Ritu::Vasanta),
        ((2026, 5, 14), Ritu::Vasanta),
        ((2026, 5, 15), Ritu::Grishma),
        ((2026, 7, 16), Ritu::Grishma),
        ((2026, 7, 17), Ritu::Varsha),
        ((2026, 9, 16), Ritu::Varsha),
        ((2026, 9, 17), Ritu::Sharad),
    ] {
        assert_eq!(ritu_on(&sdk, day), ritu, "{day:?}");
    }
}

/// Under the root profile the months are placed by the punya-kala over
/// the modern Sun: the sankranti after sunrise on 15 May 2026 still
/// begins Grishma that day, and the Karka sankranti before the dawn of 17
/// July 2026 begins Varsha on the 16th.
#[test]
fn the_punyakala_places_the_modern_sankrantis() {
    let sdk = context(Some(r#"{"panchanga": {"solar_month_start": "PUNYAKALA"}}"#));
    assert_eq!(ritu_on(&sdk, (2026, 5, 14)), Ritu::Vasanta);
    assert_eq!(ritu_on(&sdk, (2026, 5, 15)), Ritu::Grishma);
    assert_eq!(ritu_on(&sdk, (2026, 7, 15)), Ritu::Grishma);
    assert_eq!(ritu_on(&sdk, (2026, 7, 16)), Ritu::Varsha);
}

#[test]
fn the_tropical_sun_runs_its_seasons_a_sign_ahead() {
    // 25 April 2026: the sidereal Sun is in Aries and the tropical in
    // Taurus.
    let day = (2026, 4, 25);
    assert_eq!(ritu_on(&context(None), day), Ritu::Vasanta);
    let tropical = context(Some(r#"{"panchanga": {"ritu": "TROPICAL"}}"#));
    assert_eq!(ritu_on(&tropical, day), Ritu::Grishma);
}

#[test]
fn the_lunar_month_names_its_own_season() {
    // 20 September 2026: the Sun is in Virgo, Sharad by the solar months,
    // and the lunar month is still Bhadrapada, Varsha, since the adhika
    // Jyeshtha put the lunar months a month behind.
    let day = (2026, 9, 20);
    assert_eq!(ritu_on(&context(None), day), Ritu::Sharad);
    let lunar = context(Some(r#"{"panchanga": {"ritu": "LUNAR"}}"#));
    assert_eq!(ritu_on(&lunar, day), Ritu::Varsha);
}
