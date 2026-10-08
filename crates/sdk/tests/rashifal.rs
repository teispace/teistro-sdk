//! The rashifal through the façade (`03-design/rashifal.md` step 2): read
//! at the reference day's sunrise, its places the founder's own, and its
//! events found to the period's last minute.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking, saying why, and index the nine and the twelve"
)]

use teistro::catalogue::{Calendar, ChartKind, Graha, Rashi};
use teistro::gochar::hits::HitEvent;
use teistro::gochar::{GocharRules, Reference, Transit, gochar};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::rashifal::baseline::Period;
use teistro::{
    CalendarDate, Context, Ephemeris, RashifalPeriod, RashifalRequest, Snapshot, UtcOffset,
};

fn context() -> Context {
    Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
}

fn kathmandu() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    )
}

const NEPAL: UtcOffset = UtcOffset::literal(5, 45, 0);

fn date(year: i32, month: u8, day: u8) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, year, month, day)
}

/// A date's year, month and day, whatever era it carries.
fn ymd(date: &CalendarDate) -> (i32, u8, u8) {
    (date.year, date.month, date.day)
}

fn period(sdk: &Context, request: &RashifalRequest) -> RashifalPeriod {
    sdk.chart().rashifal(request).unwrap().value
}

/// The civil day at Nepal's offset an instant falls in.
fn civil_day(sdk: &Context, instant: JulianDay<Utc>) -> CalendarDate {
    let local = instant.get() + f64::from(NEPAL.seconds()) / 86_400.0;
    sdk.calendar()
        .date_of(Calendar::Gregorian, teistro::fixed_of_jd(local).0)
        .unwrap()
}

#[test]
fn the_sky_is_read_at_the_middle_days_sunrise() {
    let sdk = context();
    let week = RashifalRequest::between(date(2026, 10, 4), date(2026, 10, 10), kathmandu(), NEPAL);
    let read = period(&sdk, &week);
    assert_eq!(ymd(&read.reference), (2026, 10, 7));
    let day = sdk
        .almanac()
        .day(&read.reference, &kathmandu(), NEPAL)
        .unwrap()
        .value;
    assert_eq!(read.instant, day.day.sunrise);
    // The places are the founder's at that instant.
    let chart = sdk
        .chart()
        .found(read.instant, &kathmandu(), NEPAL, ChartKind::Natal)
        .unwrap()
        .value;
    for one in &chart.grahas {
        assert_eq!(
            read.transits[one.graha as usize],
            Transit::at_longitude(one.longitude_deg),
            "{:?}",
            one.graha
        );
    }
    // Each sign's gochar is the kernel's from it.
    let rules = GocharRules::of(sdk.settings());
    for reading in &read.readings {
        assert_eq!(
            reading.gochar,
            gochar(Reference::moon(reading.rashi), &read.transits, rules)
        );
    }
    // The nodes move backwards and the luminaries never do.
    assert!(read.retrograde[Graha::Rahu as usize] || read.retrograde[Graha::Ketu as usize]);
    assert!(!read.retrograde[Graha::Sun as usize] && !read.retrograde[Graha::Moon as usize]);
}

#[test]
fn a_clock_snapshot_is_that_minute_of_the_reference_day() {
    let sdk = context();
    let six = RashifalRequest::day(date(2026, 10, 7), kathmandu(), NEPAL)
        .at(Snapshot::Clock { hour: 6, minute: 0 });
    let read = period(&sdk, &six);
    // 06:00 at +05:45 is 00:15 UTC.
    let expected =
        teistro::jd_of_fixed(sdk.calendar().fixed_of(&date(2026, 10, 7)).unwrap()) + 15.0 / 1440.0;
    assert!((read.instant.get() - expected).abs() < 1e-9);
    assert_eq!(
        sdk.chart()
            .rashifal(&six)
            .unwrap()
            .provenance
            .applied_conventions
            .last()
            .unwrap()
            .value,
        "06:00"
    );
}

#[test]
fn every_nodes_ingress_is_found_and_counted_from_each_sign() {
    let sdk = context();
    // The nodes leave Aquarius and Leo in the winter of 2026.
    let year = RashifalRequest::between(date(2026, 1, 1), date(2026, 12, 31), kathmandu(), NEPAL);
    let read = period(&sdk, &year);
    let aries = read.of(Rashi::Aries);
    for node in [Graha::Rahu, Graha::Ketu] {
        let ingress = aries
            .events
            .iter()
            .find(|one| {
                one.event.hit.graha == node
                    && matches!(one.event.hit.event, HitEvent::SignIngress { .. })
            })
            .expect("each node enters a sign in 2026");
        let into = ingress.event.sign;
        for reading in &read.readings {
            let same = reading
                .events
                .iter()
                .find(|one| one.event == ingress.event)
                .unwrap();
            assert_eq!(
                same.house,
                teistro::House::between(reading.rashi, into).get()
            );
        }
    }
}

#[test]
fn an_ingress_on_the_last_evening_belongs_to_the_period() {
    let sdk = context();
    // Saturn leaves Pisces in 2027; find the day, then end a week on it.
    let year = RashifalRequest::between(date(2027, 1, 1), date(2027, 12, 31), kathmandu(), NEPAL)
        .with_events([Graha::Saturn]);
    let read = period(&sdk, &year);
    let ingress = read
        .of(Rashi::Aries)
        .events
        .iter()
        .find(|one| matches!(one.event.hit.event, HitEvent::SignIngress { .. }))
        .unwrap()
        .event;
    let day = civil_day(&sdk, ingress.hit.instant);
    let fixed = sdk.calendar().fixed_of(&day).unwrap();
    let week_before = sdk
        .calendar()
        .date_of(Calendar::Gregorian, fixed.plus_days(-6))
        .unwrap();
    let ending =
        RashifalRequest::between(week_before, day, kathmandu(), NEPAL).with_events([Graha::Saturn]);
    let found = period(&sdk, &ending);
    assert!(
        found
            .of(Rashi::Aries)
            .events
            .iter()
            .any(|one| one.event == ingress)
    );
    // The day after holds it no longer.
    let next = sdk
        .calendar()
        .date_of(Calendar::Gregorian, fixed.plus_days(1))
        .unwrap();
    let after = RashifalRequest::day(next, kathmandu(), NEPAL).with_events([Graha::Saturn]);
    assert_eq!(period(&sdk, &after).of(Rashi::Aries).events, []);
}

#[test]
fn the_baseline_score_reads_the_period_it_was_given() {
    let sdk = context();
    let month = RashifalRequest::between(date(2026, 10, 1), date(2026, 10, 31), kathmandu(), NEPAL);
    let read = period(&sdk, &month);
    assert_eq!(ymd(&read.reference), (2026, 10, 16));
    for rashi in Rashi::ALL {
        let score = read.baseline_score(rashi, Period::Monthly);
        assert!(score.overall <= 100, "{rashi:?}");
        assert!(score.key_influences.len() <= 5);
    }
}

#[test]
fn a_bad_request_is_refused_by_the_field_it_names() {
    let sdk = context();
    let field = |request: &RashifalRequest| {
        sdk.chart()
            .rashifal(request)
            .unwrap_err()
            .field()
            .map(String::from)
    };
    let day = || RashifalRequest::day(date(2026, 10, 7), kathmandu(), NEPAL);
    assert_eq!(
        field(&RashifalRequest::between(
            date(2026, 10, 7),
            date(2026, 10, 6),
            kathmandu(),
            NEPAL
        ))
        .as_deref(),
        Some("last")
    );
    assert_eq!(
        field(&day().at(Snapshot::Clock {
            hour: 24,
            minute: 0
        }))
        .as_deref(),
        Some("snapshot")
    );
    for spells in [vec![1], vec![13], vec![4, 4]] {
        assert_eq!(
            field(&day().with_spells(spells.clone())).as_deref(),
            Some("spells"),
            "{spells:?}"
        );
    }
    assert_eq!(
        field(&day().with_events([Graha::Mars, Graha::Mars])).as_deref(),
        Some("events")
    );
}

#[test]
fn a_batch_reads_each_period_as_it_reads_alone() {
    let sdk = context();
    let periods = [
        RashifalRequest::day(date(2026, 10, 7), kathmandu(), NEPAL),
        RashifalRequest::between(date(2026, 10, 4), date(2026, 10, 10), kathmandu(), NEPAL)
            .at(Snapshot::Clock { hour: 6, minute: 0 }),
    ];
    let batch = sdk.chart().rashifal_many(&periods).unwrap();
    for (one, request) in batch.value.iter().zip(&periods) {
        assert_eq!(*one, period(&sdk, request));
    }
    let stamped: Vec<&str> = batch
        .provenance
        .applied_conventions
        .iter()
        .filter(|one| one.knob == "rashifal.snapshot")
        .map(|one| one.value.as_str())
        .collect();
    assert_eq!(stamped, ["SUNRISE", "06:00"]);
    assert_eq!(
        sdk.chart().rashifal_many(&[]).unwrap_err().field(),
        Some("requests")
    );
}
