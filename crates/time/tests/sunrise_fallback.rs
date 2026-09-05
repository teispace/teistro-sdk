//! The `SUNRISE` unknown-time fallback: a birth without a time resolves to
//! the place's sunrise on that date, with the resolution saying so
//! (`docs/03-design/time-and-timezone.md`, §4).

#![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

use teistro_calendar::{CalendarDate, Gregorian};
use teistro_core::catalogue::Calendar;
use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
use teistro_core::settings::{PolarDayPolicy, UnknownTime};
use teistro_siddhanta::SuryaSiddhanta;
use teistro_time::{
    CivilDateTime, DayContext, EmbeddedTzdb, Policy, Warning, ZoneSpec, local_day, resolve,
    resolve_at_place, zones,
};

const KATHMANDU: Place = Place::new(
    Latitude::literal(27.7172),
    Longitude::literal(85.324),
    Altitude::literal(1400.0),
);

#[test]
fn a_date_without_a_time_resolves_to_the_sunrise_of_the_local_day() {
    let text = SuryaSiddhanta::text();
    let date = CalendarDate::defined(Calendar::Gregorian, 1990, 4, 14);
    let civil = CivilDateTime::date_only(date.clone());
    let zone = ZoneSpec::iana("Asia/Kathmandu");
    let policy = Policy {
        unknown_time: UnknownTime::Sunrise,
        ..Policy::default()
    };
    let day = DayContext {
        model: &text,
        place: KATHMANDU,
        polar_day_policy: PolarDayPolicy::Undefined,
    };
    let resolved = resolve_at_place(&civil, &zone, &policy, EmbeddedTzdb::shared(), day).unwrap();
    let expected = local_day(
        &text,
        &Gregorian,
        zones::nepal(),
        &KATHMANDU,
        &date,
        PolarDayPolicy::Undefined,
    )
    .unwrap();
    assert_eq!(resolved.instant, expected.sunrise);
    assert!(!resolved.zone.time_known);
    assert!(resolved.zone.has(Warning::TimeUnknownFallback));
    assert_eq!(resolved.zone.offset.to_string(), "+05:45");
    let time = resolved.civil.time.unwrap();
    let at = &resolved.civil.date;
    assert_eq!((at.year, at.month, at.day), (1990, 4, 14));
    assert!((5..=6).contains(&time.hour()), "{time}");
    // With a time given, the fallback is not consulted.
    let with_time = CivilDateTime::at(
        date.clone(),
        teistro_time::CivilTime::new(5, 30, 0).unwrap(),
    );
    let plain = resolve_at_place(&with_time, &zone, &policy, EmbeddedTzdb::shared(), day).unwrap();
    assert!(plain.zone.time_known);
    assert_eq!(
        plain,
        resolve(&with_time, &zone, &policy, EmbeddedTzdb::shared()).unwrap()
    );
    // The plain resolver still refuses a missing time under SUNRISE, and
    // names the way through.
    let refused = resolve(&civil, &zone, &policy, EmbeddedTzdb::shared()).unwrap_err();
    assert!(refused.hint().unwrap().contains("resolve_at_place"));
    // An unknown zone is refused before any day is computed.
    let unknown = resolve_at_place(
        &civil,
        &ZoneSpec::iana("Asia/Kathmandoo"),
        &policy,
        EmbeddedTzdb::shared(),
        day,
    )
    .unwrap_err();
    assert!(
        unknown.hint().is_some_and(|h| h.contains("Asia/Kathmandu")),
        "{unknown}"
    );
    // Local mean time and a fixed offset work as clocks too.
    for spec in [
        ZoneSpec::local_mean(Longitude::literal(85.324)),
        ZoneSpec::fixed(teistro_time::UtcOffset::literal(5, 45, 0)),
    ] {
        let at = resolve_at_place(&civil, &spec, &policy, EmbeddedTzdb::shared(), day).unwrap();
        assert_eq!(at.instant, expected.sunrise, "{spec}");
        assert!(!at.zone.time_known);
    }
}
