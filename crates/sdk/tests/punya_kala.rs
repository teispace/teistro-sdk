//! The punya kala of each sankranti through the façade
//! (`03-design/panchanga-day.md` §21.5): one day a sign, the window in
//! that day's daylight, and the rules of the night read on a real sky.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    reason = "tests fail by panicking, index what they asked for, and the sweep prints what it measured"
)]

use teistro::Panchanga;
use teistro::catalogue::{Calendar, Rashi};
use teistro::punya::PunyaKala;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{CalendarDate, Context, Ephemeris, UtcOffset};

fn context(patch: Option<&str>) -> Context {
    let builder = Context::builder()
        .profile("nepali-drik")
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

/// What the sweep found: the entries whose window fell on the day the
/// month-start rule gives, and the two kinds that may not.
#[derive(Debug, Default, PartialEq)]
struct Swept {
    entries: usize,
    agree: usize,
    /// Makara in the three ghatis after sunset or Karka in the three
    /// before sunrise: the window keeps the twilight's day, the month the
    /// night's (C2421).
    twilight: usize,
    /// An entry between the clock's midnight and the middle of the night,
    /// either side: the window turns on the night's middle, the month on
    /// the clock's.
    midnight: usize,
}

/// SR-160: every sankranti at Kathmandu from 1800 to 2399, its window and
/// its maha punya kala read on the day the almanac gives it, held to the
/// day `MonthStartRule::Punyakala` begins the solar month on under the
/// same sky. A disagreement is one of the two kinds `Swept` names, and
/// each is counted.
#[test]
fn every_window_from_1800_to_2399_falls_on_the_month_rules_day() {
    use teistro_calendar::MonthStartRule;
    use teistro_calendar::gregorian::{fixed_from_gregorian, gregorian_from_fixed};
    use teistro_calendar::solar::DayArc;
    use teistro_calendar::solar::rule::local_day;

    let sdk = context(None);
    let place = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );
    let clock = UtcOffset::literal(5, 45, 0);
    let days_around = |from: teistro::FixedDay, to: teistro::FixedDay| {
        let date = |fixed| {
            let (year, month, day) = gregorian_from_fixed(fixed);
            CalendarDate::defined(Calendar::Gregorian, year, month, day)
        };
        sdk.almanac()
            .of(&date(from), &date(to), &place, clock)
            .unwrap()
            .value
    };
    let first = fixed_from_gregorian(1800, 1, 1);
    let last = fixed_from_gregorian(2399, 12, 31);
    let mut next = days_around(first, first)[0]
        .sun
        .signs
        .last()
        .unwrap()
        .whole
        .to;
    let mut swept = Swept::default();
    let started = std::time::Instant::now();
    loop {
        let (civil, fraction) = local_day(&clock, next);
        if civil.plus_days(1) > last {
            break;
        }
        let days = days_around(civil.plus_days(-1), civil.plus_days(1));
        let held: Vec<(&Panchanga, &PunyaKala)> = kalas(&days)
            .into_iter()
            .filter(|(_, kala)| near(kala.sankranti.get(), next.get()))
            .collect();
        assert_eq!(held.len(), 1, "one window for the entry at {next}");
        let (day, kala) = held[0];
        let sign = kala.span.member;
        // The maha punya kala lies inside the window, five ghatis or all of it.
        let (inside, maha) = (kala.span.inside, kala.maha);
        assert!(inside.from.get() <= maha.from.get() && maha.to.get() <= inside.to.get());
        assert!(
            near(maha.days(), (5.0 * GHATI).min(inside.days())),
            "{sign:?} at {next}"
        );
        let arc_of = |fixed: teistro::FixedDay| {
            let (year, month, on) = gregorian_from_fixed(fixed);
            let record = days
                .iter()
                .find(|d| (d.day.date.year, d.day.date.month, d.day.date.day) == (year, month, on))
                .unwrap();
            Ok::<_, teistro::Error>(DayArc {
                sunrise: record.day.sunrise,
                sunset: record.day.sunset,
            })
        };
        let ruled = MonthStartRule::Punyakala
            .month_start_by(u8::try_from(sign.id()).unwrap(), next, &clock, &arc_of)
            .unwrap();
        let (year, month, on) = gregorian_from_fixed(ruled);
        swept.entries += 1;
        if (day.day.date.year, day.day.date.month, day.day.date.day) == (year, month, on) {
            swept.agree += 1;
            next = days[2].sun.signs.last().unwrap().whole.to;
            continue;
        }
        let ghatis = |a: f64, b: f64| (b - a) / GHATI;
        let night_of = days
            .iter()
            .find(|d| d.day.sunset.get() <= next.get() && next.get() < d.day.next_sunrise.get());
        let twilight = night_of.is_some_and(|d| match sign {
            Rashi::Capricorn => ghatis(d.day.sunset.get(), next.get()) < 3.0,
            Rashi::Cancer => ghatis(next.get(), d.day.next_sunrise.get()) < 3.0,
            _ => false,
        });
        let midnight = night_of.is_some_and(|d| {
            let middle = f64::midpoint(d.day.sunset.get(), d.day.next_sunrise.get());
            let clock_midnight = next.get() - fraction;
            let (lo, hi) = (middle.min(clock_midnight), middle.max(clock_midnight));
            lo <= next.get() && next.get() <= hi
        });
        assert!(
            twilight || midnight,
            "{sign:?} at {next}: window on {:?}, the month on {year}-{month}-{on}",
            day.day.date
        );
        if twilight {
            swept.twilight += 1;
        } else {
            swept.midnight += 1;
        }
        next = days[2].sun.signs.last().unwrap().whole.to;
    }
    println!("swept {swept:?} in {:?}", started.elapsed());
    assert_eq!(swept.entries, 600 * 12, "{swept:?}");
    assert_eq!(
        swept,
        Swept {
            entries: 600 * 12,
            agree: 0,
            twilight: 0,
            midnight: 0
        },
        "pinned once measured"
    );
}
