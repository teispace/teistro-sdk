//! The punya kala of each sankranti through the façade
//! (`03-design/panchanga-day.md` §21.5): one day a sign, the window in
//! that day's daylight, and the rules of the night read on a real sky.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::Panchanga;
use teistro::catalogue::{Calendar, Rashi};
use teistro::punya::PunyaKala;
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

/// The days of 2026 at Kathmandu on the Nepal clock.
fn year(sdk: &Context) -> Vec<Panchanga> {
    let place = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );
    let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    sdk.almanac()
        .of(
            &date(1, 1),
            &date(12, 31),
            &place,
            UtcOffset::literal(5, 45, 0),
        )
        .unwrap()
        .value
}

/// Each day that holds a punya kala, with it.
fn kalas(days: &[Panchanga]) -> Vec<(&Panchanga, &PunyaKala)> {
    days.iter()
        .filter_map(|day| day.punya_kala.as_ref().map(|kala| (day, kala)))
        .collect()
}

/// A civil ghati, in days.
const GHATI: f64 = 1.0 / 60.0;

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn each_sign_has_one_window_in_one_daylight() {
    let days = year(&context(None));
    let kalas = kalas(&days);
    let signs: Vec<Rashi> = kalas.iter().map(|(_, kala)| kala.span.member).collect();
    // Makara on 14 January, then the year round to Dhanu.
    assert_eq!(signs.len(), 12);
    assert_eq!(signs[0], Rashi::Capricorn);
    for pair in signs.windows(2) {
        assert_eq!((pair[0].id() + 1) % 12, pair[1].id());
    }
    for (day, kala) in &kalas {
        let inside = kala.span.inside;
        assert!(
            day.day.sunrise.get() <= inside.from.get(),
            "{:?}",
            day.day.date
        );
        assert!(
            inside.to.get() <= day.day.sunset.get(),
            "{:?}",
            day.day.date
        );
        assert!(inside.days() > 0.0);
        // Within a day of its sankranti either way.
        assert!((kala.sankranti.get() - inside.from.get()).abs() < 1.0);
    }
}

#[test]
fn the_texts_rules_read_on_the_sky() {
    let days = year(&context(None));
    let kalas = kalas(&days);
    let of = |sign: Rashi| {
        kalas
            .iter()
            .find(|(_, kala)| kala.span.member == sign)
            .copied()
            .unwrap()
    };
    // Makara in the afternoon of 14 January: its forty ghatis after it
    // run past sunset, so the window closes there.
    let (day, kala) = of(Rashi::Capricorn);
    assert_eq!(day.day.date.day, 14);
    assert_eq!(kala.span.inside.from, kala.sankranti);
    assert_eq!(kala.span.inside.to, day.day.sunset);
    assert!(near(kala.span.whole.days(), 40.0 * GHATI));
    // Mesha by day: fifteen ghatis either side.
    let (_, kala) = of(Rashi::Aries);
    assert!(near(
        kala.sankranti.get() - kala.span.whole.from.get(),
        15.0 * GHATI
    ));
    assert!(near(
        kala.span.whole.to.get() - kala.sankranti.get(),
        15.0 * GHATI
    ));
    // Karka near midnight of 16 July: the day before, by the thirty
    // ghatis before it.
    let (day, kala) = of(Rashi::Cancer);
    assert_eq!((day.day.date.month, day.day.date.day), (7, 16));
    assert!(kala.sankranti.get() > day.day.sunset.get());
    assert_eq!(kala.span.whole.to, kala.sankranti);
    assert_eq!(kala.span.inside.to, day.day.sunset);
    // Tula on the evening of 17 October, before midnight: the afternoon.
    let (day, kala) = of(Rashi::Libra);
    assert_eq!((day.day.date.month, day.day.date.day), (10, 17));
    let midday = f64::midpoint(day.day.sunrise.get(), day.day.sunset.get());
    assert!(near(kala.span.inside.from.get(), midday));
    assert_eq!(kala.span.inside.to, day.day.sunset);
    // Kumbha before the dawn of 13 February, after midnight: the morning.
    let (day, kala) = of(Rashi::Aquarius);
    assert_eq!((day.day.date.month, day.day.date.day), (2, 13));
    assert!(kala.sankranti.get() < day.day.sunrise.get());
    assert_eq!(kala.span.inside.from, day.day.sunrise);
    let midday = f64::midpoint(day.day.sunrise.get(), day.day.sunset.get());
    assert!(near(kala.span.inside.to.get(), midday));
}

#[test]
fn the_tables_and_the_day_boundary_are_settings() {
    let shipped = year(&context(None));
    let hemadri = year(&context(Some(
        r#"{"panchanga": {"punya_kala": "HEMADRI"}}"#,
    )));
    let mesha = |days: &[Panchanga]| {
        kalas(days)
            .into_iter()
            .find(|(_, kala)| kala.span.member == Rashi::Aries)
            .map(|(_, kala)| kala.span.whole.days())
            .unwrap()
    };
    assert!(near(mesha(&shipped), 30.0 * GHATI));
    assert!(near(mesha(&hemadri), 20.0 * GHATI));
    // A day counted from midnight finds the same sankrantis, and the
    // same windows on the same dates.
    let midnight = year(&context(Some(r#"{"day": {"day_boundary": "MIDNIGHT"}}"#)));
    let windows = |days: &[Panchanga]| {
        kalas(days)
            .into_iter()
            .map(|(day, kala)| (day.day.date.month, day.day.date.day, kala.span.inside))
            .collect::<Vec<_>>()
    };
    assert_eq!(windows(&shipped), windows(&midnight));
}
