//! Pancha Pakshi through the façade: a day's birds read over the
//! almanac's own sunrise, sunset and tithi, and a native's bird from a
//! chart's Moon (`docs/03-design/pakshi.md`).

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking on what they asked for"
)]

use teistro::catalogue::{Calendar, Paksha, Vara};
use teistro::pakshi::{Activity, Bird, BirthBird, Half, Rules, birth_bird};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{
    CalendarDate, ChartRequest, Context, Ephemeris, PakshiNative, PakshiRequest, UtcOffset,
};

fn sdk() -> Context {
    Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
}

/// A context that synthesises a polar day's bounds, so the almanac answers
/// the day and Pancha Pakshi is what finds it has no yamas.
fn polar() -> Context {
    Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .settings_json(r#"{"day": {"polar_day_policy": "NEAREST_EVENT"}}"#)
        .build()
        .unwrap()
}

fn tromso() -> Place {
    Place::new(
        Latitude::literal(69.6492),
        Longitude::literal(18.9553),
        Altitude::literal(0.0),
    )
}

fn madras() -> Place {
    Place::new(
        Latitude::literal(13.0827),
        Longitude::literal(80.2707),
        Altitude::literal(6.0),
    )
}

#[test]
fn the_books_bright_wednesday_reads_over_the_almanacs_day() {
    // PUL p. vii: 31 October 1984 is a bright Wednesday, on which the cock
    // sleeps in the day's second yama and dies from the third.
    let sdk = sdk();
    let date = CalendarDate::defined(Calendar::Gregorian, 1984, 10, 31);
    let offset = UtcOffset::literal(5, 30, 0);
    let read = sdk
        .almanac()
        .pakshi(&date, &madras(), offset, Bird::Cock, &Rules::default())
        .unwrap()
        .value;
    assert_eq!(
        (read.day.vara, read.day.paksha),
        (Vara::Budhavara, Paksha::Shukla)
    );
    let day = sdk.almanac().day(&date, &madras(), offset).unwrap().value;
    assert_eq!(
        read.day.sunrise,
        day.day.sunrise.get(),
        "the almanac's own sunrise"
    );
    assert_eq!(read.day.sunset, day.day.sunset.get(), "and its sunset");
    let activities: Vec<(Half, u8, Activity)> = read
        .yamas
        .iter()
        .take(3)
        .map(|yama| (yama.half, yama.yama, yama.activity))
        .collect();
    assert_eq!(activities[1], (Half::Day, 2, Activity::Sleeping));
    assert_eq!(activities[2], (Half::Day, 3, Activity::Dying));
    assert_eq!(read.yamas.len(), 10);
    assert_eq!(read.yamas[9].span.to, day.day.next_sunrise.get());
}

#[test]
fn a_day_without_a_sunrise_has_no_yamas() {
    let midsummer = CalendarDate::defined(Calendar::Gregorian, 2024, 6, 21);
    let offset = UtcOffset::literal(2, 0, 0);
    let refused = polar()
        .almanac()
        .pakshi(&midsummer, &tromso(), offset, Bird::Owl, &Rules::default())
        .unwrap_err();
    assert_eq!(refused.status, teistro::Status::Unsupported);
    assert!(refused.message.contains("no yamas"), "{}", refused.message);
    // Under the default policy the almanac refuses the day itself.
    let almanac = sdk()
        .almanac()
        .pakshi(&midsummer, &tromso(), offset, Bird::Owl, &Rules::default())
        .unwrap_err();
    assert_eq!(almanac.field(), Some("day.polar_day_policy"));
}

#[test]
fn a_natives_bird_is_the_almanacs_star_and_paksha_at_birth() {
    // Kathmandu, 14 April 1990, the corpus's first chart: the bird is read
    // back through the almanac's own nakshatra and tithi at the instant.
    let sdk = sdk();
    let kathmandu = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );
    let birth = JulianDay::<Utc>::literal(2_447_995.489_583_333_5);
    let request = ChartRequest::at(kathmandu, UtcOffset::literal(5, 45, 0));
    let document = sdk.chart().reading(birth, &request).unwrap().value;
    let date = CalendarDate::defined(Calendar::Gregorian, 1990, 4, 14);
    let day = sdk
        .almanac()
        .day(&date, &kathmandu, UtcOffset::literal(5, 45, 0))
        .unwrap()
        .value;
    let star = day.nakshatra_at(birth).unwrap().member;
    let paksha = day.tithi_at(birth).unwrap().member.attributes().paksha;
    for rule in [BirthBird::ByPaksha, BirthBird::Single] {
        assert_eq!(
            sdk.chart().pakshi_native(&document, rule).unwrap().bird(),
            birth_bird(star, paksha, rule),
            "{rule:?}"
        );
    }
}

#[test]
fn a_range_of_days_chains_sunrise_to_sunrise_and_a_polar_day_is_none() {
    let sdk = sdk();
    let asked = PakshiRequest::from_json(
        r#"{"first": {"year": 1984, "month": 10, "day": 30},
            "last": {"year": 1984, "month": 11, "day": 1},
            "latitudeDeg": 13.0827, "longitudeDeg": 80.2707, "altitudeM": 6,
            "utcOffsetSeconds": 19800, "native": {"bird": "COCK"}}"#,
    )
    .unwrap();
    let days = sdk.almanac().pakshi_request(&asked).unwrap().value;
    assert_eq!(days.len(), 3);
    let read: Vec<_> = days
        .iter()
        .map(|day| day.reading.as_ref().unwrap())
        .collect();
    for pair in read.windows(2) {
        assert_eq!(pair[0].day.next_sunrise, pair[1].day.sunrise);
    }
    assert_eq!(read[1].day.vara, Vara::Budhavara);
    let one = sdk
        .almanac()
        .pakshi(
            &days[1].date,
            &madras(),
            UtcOffset::literal(5, 30, 0),
            Bird::Cock,
            &Rules::default(),
        )
        .unwrap()
        .value;
    assert_eq!(&one, read[1], "one day is the range's day");

    let midsummer = polar()
        .almanac()
        .pakshi_days(
            &CalendarDate::defined(Calendar::Gregorian, 2024, 6, 20),
            &CalendarDate::defined(Calendar::Gregorian, 2024, 6, 22),
            &tromso(),
            UtcOffset::literal(2, 0, 0),
            PakshiNative::Bird(Bird::Owl),
            &Rules::default(),
        )
        .unwrap()
        .value;
    assert_eq!(midsummer.len(), 3);
    assert!(midsummer.iter().all(|day| day.reading.is_none()));
}
