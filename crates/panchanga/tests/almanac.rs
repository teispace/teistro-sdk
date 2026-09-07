//! A day assembled end to end, over the analytic test provider.
//!
//! The provider's numbers are a plausible sky and not the sky, so nothing
//! here compares an instant with an almanac; `baseline.rs` does that
//! against the corpus for everything that does not need positions. What
//! is checked here is what only the assembled value can show: that the
//! parts agree with each other, and that they agree with the design's own
//! claims.
//!
//! Those claims are: the periods divide the arcs of the very day they are
//! reported with; the window is the day and moves with `day.day_boundary`;
//! the same date asked for twice, or reached from an instant inside it, is
//! the same value; a range is its days; and a day's own accessors answer
//! about the day.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    reason = "tests fail by panicking, index fixed lists and print measurements under --nocapture"
)]

use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::precession::PrecessionModel;
use teistro_calendar::solar::drik::DrikSun;
use teistro_calendar::{CalendarDate, Gregorian};
use teistro_core::catalogue::{Ayanamsha, Calendar};
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro_core::settings::{
    DayBoundary, MoonEvents, OverridePolicy, Profile, Resolved, SettingsPatch, Sunrise,
};
use teistro_core::time::UtcOffset;
use teistro_panchanga::{Almanac, Panchanga};
use teistro_port_ephemeris::test_provider::TestProvider;

/// Kathmandu, where the corpus's own charts are.
fn place() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.3240),
        Altitude::literal(1400.0),
    )
}

fn date(year: i32, month: u8, day: u8) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, year, month, day)
}

fn resolved(patch: &SettingsPatch) -> Resolved {
    Profile::shipped(teistro_core::settings::DEFAULT_PROFILE)
        .unwrap_or_else(|| panic!("the default profile"))
        .resolve(patch)
        .unwrap_or_else(|e| panic!("{e}"))
}

/// Runs a closure with an almanac over the test provider.
fn with_almanac<T>(patch: &SettingsPatch, run: impl FnOnce(&Almanac<'_, TestProvider>) -> T) -> T {
    let provider = TestProvider;
    let resolved = resolved(patch);
    let model = DrikSun::new(
        &provider,
        Ayanamsha::Lahiri,
        Sunrise::CentreNoRefraction.into(),
        OverridePolicy::PreferNative,
        DeltaTModel::TableThenModel,
    );
    let clock = UtcOffset::literal(5, 45, 0);
    let almanac = Almanac::new(
        &provider,
        &resolved,
        &model,
        &Gregorian,
        &clock,
        PrecessionModel::Vondrak2011,
        DeltaTModel::TableThenModel,
    );
    run(&almanac)
}

/// One day of the plausible sky at Kathmandu.
fn day() -> Panchanga {
    with_almanac(&SettingsPatch::default(), |almanac| {
        almanac
            .day(&date(2024, 6, 21), &place())
            .unwrap_or_else(|e| panic!("{e}"))
            .value
    })
}

#[test]
fn a_day_is_assembled_whole() {
    let day = day();
    println!(
        "{}: {} tithis, {} nakshatras, {} yogas, {} karanas, {} choghadiya, {} horas",
        day.day.date,
        day.limbs.tithi.len(),
        day.limbs.nakshatra.len(),
        day.limbs.yoga.len(),
        day.limbs.karana.len(),
        day.choghadiya.len(),
        day.horas.len()
    );
    assert_eq!(day.horas.len(), 24);
    assert_eq!(day.choghadiya.len(), 16);
    assert_eq!(day.kaalas.len(), 3);
    assert_eq!(day.muhurtas.daylight.len(), 15);
    assert_eq!(day.muhurtas.night.len(), 15);
    assert!(day.muhurtas.abhijit.is_some());
    assert!(!day.limbs.tithi.is_empty());
    assert!(!day.limbs.nakshatra.is_empty());
    assert!(!day.limbs.yoga.is_empty());
    assert!(!day.limbs.karana.is_empty());
    assert!(!day.sun.signs.is_empty());
    assert!(!day.moon.signs.is_empty());
}

#[test]
fn every_period_divides_the_arc_of_the_day_it_is_reported_with() {
    let day = day();
    let sunrise = day.day.sunrise.get();
    let sunset = day.day.sunset.get();
    let next = day.day.next_sunrise.get();

    // The eighths and the first eight choghadiya are of the daylight.
    for kaala in &day.kaalas {
        assert!(
            kaala.at.from.get() >= sunrise && kaala.at.to.get() <= sunset,
            "{:?} is not inside the daylight",
            kaala.kaala
        );
        assert!(
            (kaala.at.days() - (sunset - sunrise) / 8.0).abs() < 1e-9,
            "an eighth of the daylight"
        );
    }
    let daytime: Vec<_> = day
        .choghadiya
        .iter()
        .filter(|part| part.is_daytime)
        .collect();
    assert_eq!(daytime.len(), 8);
    assert!((daytime[0].at.from.get() - sunrise).abs() < 1e-9);
    assert!((daytime[7].at.to.get() - sunset).abs() < 1e-9);
    let night: Vec<_> = day
        .choghadiya
        .iter()
        .filter(|part| !part.is_daytime)
        .collect();
    assert!((night[0].at.from.get() - sunset).abs() < 1e-9);
    assert!((night[7].at.to.get() - next).abs() < 1e-9);

    // The horas run sunrise to sunrise, twelve and twelve.
    assert!((day.horas[0].start.get() - sunrise).abs() < 1e-9);
    assert!((day.horas[11].end.get() - sunset).abs() < 1e-9);
    assert!((day.horas[23].end.get() - next).abs() < 1e-9);

    // And Brahma muhurta is before this sunrise, not after it.
    let brahma = day.muhurtas.brahma.expect("yesterday's arc is known");
    assert!(brahma.to.get() <= sunrise, "it ends before sunrise");
    assert!(brahma.from.get() > sunrise - 0.5, "and inside that night");
}

#[test]
fn the_window_is_the_day_and_the_limbs_fill_it() {
    let day = day();
    assert_eq!(day.window.from, day.day.sunrise);
    assert_eq!(day.window.to, day.day.next_sunrise);
    let tithis = &day.limbs.tithi;
    assert_eq!(tithis[0].inside.from, day.window.from);
    assert_eq!(tithis[tithis.len() - 1].inside.to, day.window.to);
    // The Moon's events are searched in that same window by default.
    assert_eq!(day.moon.window, day.window);
}

#[test]
fn a_midnight_boundary_moves_the_window_and_not_the_periods() {
    let mut patch = SettingsPatch::default();
    patch.day.day_boundary = Some(DayBoundary::Midnight);
    let midnight = with_almanac(&patch, |almanac| {
        almanac
            .day(&date(2024, 6, 21), &place())
            .unwrap_or_else(|e| panic!("{e}"))
            .value
    });
    let sunrise = day();

    // The window moved.
    assert_ne!(midnight.window.from, sunrise.window.from);
    assert!(
        (midnight.window.days() - 1.0).abs() < 1e-9,
        "a civil day is a day long"
    );
    // The arcs, and so every period, did not: a choghadiya divides the
    // daylight whatever the window is.
    assert_eq!(midnight.day.sunrise, sunrise.day.sunrise);
    assert_eq!(midnight.choghadiya.len(), sunrise.choghadiya.len());
    assert_eq!(
        midnight.choghadiya[0].at.from,
        sunrise.choghadiya[0].at.from
    );
    assert_eq!(midnight.horas[0].start, sunrise.horas[0].start);
}

#[test]
fn the_moon_events_knob_moves_the_window_they_are_found_in() {
    let mut patch = SettingsPatch::default();
    patch.panchanga.moon_events = Some(MoonEvents::CivilDay);
    let civil = with_almanac(&patch, |almanac| {
        almanac
            .day(&date(2024, 6, 21), &place())
            .unwrap_or_else(|e| panic!("{e}"))
            .value
    });
    let day = day();
    assert_ne!(civil.moon.window, day.moon.window);
    assert!(
        (civil.moon.window.days() - 1.0).abs() < 1e-9,
        "the civil day is a day"
    );
    // Everything else is untouched: the knob is about two fields.
    assert_eq!(civil.limbs, day.limbs);
    assert_eq!(civil.choghadiya, day.choghadiya);
}

#[test]
fn a_day_reached_from_an_instant_inside_it_is_the_same_day() {
    with_almanac(&SettingsPatch::default(), |almanac| {
        let by_date = almanac
            .day(&date(2024, 6, 21), &place())
            .unwrap_or_else(|e| panic!("{e}"))
            .value;
        // Noon of that day, which is inside its arc.
        let noon = JulianDay::<Utc>::literal(by_date.day.sunrise.get() + 0.25);
        let by_instant = almanac
            .at(noon, &place())
            .unwrap_or_else(|e| panic!("{e}"))
            .value;
        assert_eq!(by_instant, by_date, "the same day, reached two ways");

        // And an instant before sunrise belongs to the day before, which
        // is the crux `crates/chart`'s day module exists for.
        let small_hours = JulianDay::<Utc>::literal(by_date.day.sunrise.get() - 0.05);
        let earlier = almanac
            .at(small_hours, &place())
            .unwrap_or_else(|e| panic!("{e}"))
            .value;
        assert_ne!(earlier.day.date, by_date.day.date);
        assert!(
            earlier.day.next_sunrise.get() <= by_date.day.sunrise.get() + 1e-9,
            "yesterday's day ends where today's begins"
        );
    });
}

#[test]
fn a_range_is_its_days_and_is_bounded() {
    with_almanac(&SettingsPatch::default(), |almanac| {
        let range = almanac
            .between(&date(2024, 6, 21), &date(2024, 6, 25), &place())
            .unwrap_or_else(|e| panic!("{e}"))
            .value;
        assert_eq!(range.len(), 5, "both ends included");
        for (offset, value) in range.iter().enumerate() {
            let day = u8::try_from(21 + offset).expect("a day of the month");
            let alone = almanac
                .day(&date(2024, 6, day), &place())
                .unwrap_or_else(|e| panic!("{e}"))
                .value;
            assert_eq!(*value, alone, "day {day} in a range and alone");
        }
        // Consecutive windows share an instant, which is what makes a
        // range cheaper than its days.
        for pair in range.windows(2) {
            assert_eq!(pair[0].window.to, pair[1].window.from);
        }
        // A range that ends before it begins, and one too long, are
        // refused by name.
        assert!(
            almanac
                .between(&date(2024, 6, 25), &date(2024, 6, 21), &place())
                .is_err()
        );
        let long = almanac.between(&date(2020, 1, 1), &date(2024, 1, 1), &place());
        assert!(long.is_err(), "four years is more than the limit");
    });
}

#[test]
fn a_day_answers_about_itself() {
    let day = day();
    let noon = JulianDay::<Utc>::literal(day.day.sunrise.get() + 0.25);
    assert!(day.tithi_at(noon).is_some());
    assert!(day.nakshatra_at(noon).is_some());
    let hora = day.hora_at(noon).expect("a hora holds noon");
    assert!((1..=24).contains(&hora.number));
    let choghadiya = day.choghadiya_at(noon).expect("a choghadiya holds noon");
    assert!(choghadiya.is_daytime, "noon is in the daylight");
    // The first hora's lord is the day's own vara's.
    assert_eq!(day.horas[0].lord, day.vara().attributes().lord);
    // An instant outside the day belongs to no part of it.
    let far = JulianDay::<Utc>::literal(day.window.to.get() + 10.0);
    assert!(day.hora_at(far).is_none());
    assert!(day.choghadiya_at(far).is_none());
    assert!(!day.is_inauspicious(far));
}

#[test]
fn the_stamp_carries_the_profile_and_the_settings_hash() {
    with_almanac(&SettingsPatch::default(), |almanac| {
        let envelope = almanac
            .day(&date(2024, 6, 21), &place())
            .unwrap_or_else(|e| panic!("{e}"));
        let stamp = &envelope.provenance;
        assert_eq!(stamp.profile, teistro_core::settings::DEFAULT_PROFILE);
        assert_ne!(stamp.settings_hash.bytes(), &[0_u8; 32]);
        assert_ne!(stamp.input_hash.bytes(), &[0_u8; 32]);
        // Two days of the same date hash the same input; two dates do not.
        let again = almanac
            .day(&date(2024, 6, 21), &place())
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(again.provenance.input_hash, stamp.input_hash);
        let other = almanac
            .day(&date(2024, 6, 22), &place())
            .unwrap_or_else(|e| panic!("{e}"));
        assert_ne!(other.provenance.input_hash, stamp.input_hash);
    });
}
