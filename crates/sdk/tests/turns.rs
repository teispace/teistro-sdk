//! Nepal's day as its daily panchanga prints it
//! (`03-design/nepal-day-measured.md`): the ends the committee's Surya
//! Siddhanta gives, a limb's member naming two days or none — the days
//! the print calls a tithi "दिनरात" (vriddhi) and the days it prints three
//! (kshaya) — and an end read in ghatis from sunrise.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::catalogue::{Calendar, Nakshatra, Tithi, Yoga};
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{CalendarDate, Context, Ephemeris, Panchanga, Sunrises, UtcOffset};
use teistro_port_ephemeris::{EphemerisProvider, TestProvider};

fn day(sdk: &Context, (y, m, d): (i32, u8, u8)) -> Panchanga {
    let place = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );
    let date = CalendarDate::defined(Calendar::Gregorian, y, m, d);
    let mut days = sdk
        .almanac()
        .of(&date, &date, &place, UtcOffset::literal(5, 45, 0))
        .unwrap()
        .value;
    days.remove(0)
}

/// Nepal's national panchanga: the text with the committee's bija, in
/// the text's own zodiac, its day begun at a modern sunrise.
fn nepal() -> Context {
    Context::builder()
        .profile("nepali-committee")
        .ephemeris([Ephemeris::SuryaSiddhanta])
        .build()
        .unwrap()
}

/// An instant by Nepal's clock, as the print writes it: an hour past 24
/// is the next morning.
fn npt((y, m, d): (i32, u8, u8), (hour, minute): (u8, u8)) -> f64 {
    let midnight = teistro_calendar::gregorian::fixed_from_gregorian(y, m, d)
        .jd_at_midnight()
        .unwrap()
        .get();
    midnight + (f64::from(hour) * 60.0 + f64::from(minute) - 345.0) / 1440.0
}

fn minutes_apart(ours: f64, printed: f64) -> f64 {
    (ours - printed).abs() * 1440.0
}

#[test]
fn the_committee_reading_ends_each_limb_where_the_print_does() {
    // 11 November 2025: Shashthi to 06:24, Saptami to 29:05, Pushya to
    // 24:49, Shubha to 16:47. The nakshatra and the yoga are read in the
    // text's own zodiac, which a search over the text reads from the
    // provider rather than from the catalogue member of the same name,
    // 1.6° away; without the bija the Moon parts from the print by up to
    // a quarter of an hour.
    let sdk = nepal();
    let date = (2025, 11, 11);
    let found = day(&sdk, date);
    let end = |spans: Vec<(u16, f64)>, member: u16| {
        spans
            .into_iter()
            .find(|(m, _)| *m == member)
            .map(|(_, to)| to)
            .unwrap()
    };
    let tithis = found
        .limbs
        .tithi
        .iter()
        .map(|s| (s.member as u16, s.whole.to.get()))
        .collect();
    let stars = found
        .limbs
        .nakshatra
        .iter()
        .map(|s| (s.member as u16, s.whole.to.get()))
        .collect();
    let yogas = found
        .limbs
        .yoga
        .iter()
        .map(|s| (s.member as u16, s.whole.to.get()))
        .collect::<Vec<_>>();
    for (ours, printed, what) in [
        (
            end(tithis, Tithi::KrishnaSaptami as u16),
            npt(date, (29, 5)),
            "Saptami",
        ),
        (
            end(stars, Nakshatra::Pushya as u16),
            npt(date, (24, 49)),
            "Pushya",
        ),
        (
            end(yogas, Yoga::Shubha as u16),
            npt(date, (16, 47)),
            "Shubha",
        ),
    ] {
        assert!(
            minutes_apart(ours, printed) < 1.5,
            "{what}: {} min",
            (ours - printed) * 1440.0
        );
    }
}

#[test]
fn a_consumers_own_bija_is_the_named_one_when_the_counts_agree() {
    let own = Context::builder()
        .profile("nepali-committee")
        .settings_json(
            r#"{"frame": {"siddhanta": {"kind": "SURYA", "bija": {"kind": "CUSTOM", "revolutions": {"moon_apsis": -4}}, "sunrise": "MODERN"}}}"#,
        )
        .ephemeris([Ephemeris::SuryaSiddhanta])
        .build()
        .unwrap();
    let date = (2025, 11, 11);
    assert_eq!(day(&own, date).limbs, day(&nepal(), date).limbs);
    // And the stamp names the set the provider was built with.
    let stamp = own
        .ephemeris()
        .unwrap()
        .capabilities()
        .identity
        .data_version;
    assert!(stamp.contains("moon_apsis -4"), "{stamp}");
}

#[test]
fn a_tithi_printed_day_and_night_holds_both_sunrises() {
    let sdk = nepal();
    for (date, tithi) in [
        ((2025, 4, 13), Tithi::KrishnaPratipada),
        ((2025, 8, 1), Tithi::ShuklaAshtami),
        ((2025, 11, 17), Tithi::KrishnaTrayodashi),
        ((2025, 12, 20), Tithi::ShuklaPratipada),
        ((2026, 5, 5), Tithi::KrishnaChaturthi),
        ((2026, 6, 30), Tithi::KrishnaPratipada),
        ((2026, 8, 24), Tithi::ShuklaDwadashi),
    ] {
        let found = day(&sdk, date);
        assert_eq!(found.vriddhi(&found.limbs.tithi), Some(tithi), "{date:?}");
        assert!(found.kshaya(&found.limbs.tithi).is_empty(), "{date:?}");
    }
}

#[test]
fn a_tithi_between_two_sunrises_names_no_day() {
    let sdk = nepal();
    for (date, tithi) in [
        ((2025, 4, 26), Tithi::KrishnaChaturdashi),
        ((2025, 8, 14), Tithi::KrishnaShashthi),
        // Shashthi ended a minute after the print's sunrise and seventeen
        // before the text's own, which has no equation of time: the modern
        // sunrise the committee's sky begins its day at is what loses
        // Saptami here (C39).
        ((2025, 11, 11), Tithi::KrishnaSaptami),
        ((2026, 1, 31), Tithi::ShuklaChaturdashi),
        ((2026, 4, 22), Tithi::ShuklaShashthi),
        ((2026, 5, 15), Tithi::KrishnaChaturdashi),
        ((2026, 6, 16), Tithi::ShuklaDvitiya),
        ((2026, 8, 10), Tithi::KrishnaTrayodashi),
        ((2026, 9, 1), Tithi::KrishnaPanchami),
    ] {
        let found = day(&sdk, date);
        assert_eq!(found.kshaya(&found.limbs.tithi), vec![tithi], "{date:?}");
        assert_eq!(found.vriddhi(&found.limbs.tithi), None, "{date:?}");
        let turns: Vec<Sunrises> = found
            .limbs
            .tithi
            .iter()
            .map(|span| found.sunrises(span))
            .collect();
        assert_eq!(
            turns,
            [Sunrises::Opening, Sunrises::Neither, Sunrises::Next],
            "{date:?}"
        );
    }
}

#[test]
fn a_span_ends_in_ghatis_from_sunrise() {
    // 25 September 2026: Chaturdashi ended at 22:16 by Nepal's clock,
    // sunrise 05:54: 16 h 22 m, which is 40 ghatis 55 palas civil.
    let sdk = nepal();
    let found = day(&sdk, (2026, 9, 25));
    let ends = found.limbs.tithi[0];
    let ghati = found.ghati_pala(ends.whole.to).unwrap();
    assert_eq!(found.sunrises(&ends), Sunrises::Opening);
    assert!((40..=41).contains(&ghati.ghati), "{ghati}");
    // A member outlasting the day reads as the day's whole count.
    let last = found.limbs.tithi.last().unwrap();
    let whole = found.ghati_pala(last.whole.to).unwrap();
    assert!((59..=60).contains(&whole.ghati), "{whole}");
}

#[test]
fn the_committees_sunrise_comes_from_the_next_modern_entry() {
    let stamp = |sdk: &Context| {
        sdk.ephemeris()
            .unwrap()
            .capabilities()
            .identity
            .data_version
    };
    // Alone in the chain, the text takes the built-in ephemeris's sunrise.
    assert!(
        stamp(&nepal()).contains("sunrise from teistro-builtin"),
        "{}",
        stamp(&nepal())
    );
    // A modern entry after it gives the sunrise instead, and the limbs
    // stay the text's.
    let modern = TestProvider::new().capabilities().identity.name;
    let beside = Context::builder()
        .profile("nepali-committee")
        .ephemeris([
            Ephemeris::SuryaSiddhanta,
            Ephemeris::Provider(Box::new(TestProvider::new())),
        ])
        .build()
        .unwrap();
    assert!(
        stamp(&beside).contains(&format!("sunrise from {modern}")),
        "{}",
        stamp(&beside)
    );
    // Its day is another, so compare a member both days hold whole.
    let date = (2025, 11, 11);
    let saptami = |sdk: &Context| {
        day(sdk, date)
            .limbs
            .tithi
            .into_iter()
            .find(|span| span.member == Tithi::KrishnaSaptami)
            .unwrap()
            .whole
    };
    assert_eq!(saptami(&beside), saptami(&nepal()));
    assert_ne!(
        day(&beside, date).day.sunrise,
        day(&nepal(), date).day.sunrise
    );
    // The text's own sunrise names nothing else.
    let text = Context::builder()
        .profile("nepali-committee")
        .settings_json(r#"{"frame": {"siddhanta": {"kind": "SURYA", "bija": {"kind": "NEPAL_COMMITTEE"}, "sunrise": "TEXT"}}}"#)
        .ephemeris([Ephemeris::SuryaSiddhanta])
        .build()
        .unwrap();
    assert!(!stamp(&text).contains("sunrise from"), "{}", stamp(&text));
}

#[test]
fn the_modern_sunrise_is_the_prints_and_the_texts_is_not() {
    // 11 November 2025 by Nepal's clock: the print's sunrise is 06:23.
    let printed = npt((2025, 11, 11), (6, 23));
    let modern = day(&nepal(), (2025, 11, 11)).day.sunrise.get();
    assert!(
        minutes_apart(modern, printed) < 1.5,
        "{}",
        (modern - printed) * 1440.0
    );
    let text = Context::builder()
        .profile("nepali-committee")
        .settings_json(r#"{"frame": {"siddhanta": {"kind": "SURYA", "bija": {"kind": "NEPAL_COMMITTEE"}, "sunrise": "TEXT"}}, "day": {"sunrise": {"kind": "NAMED", "which": "CENTRE_NO_REFRACTION"}}}"#)
        .ephemeris([Ephemeris::SuryaSiddhanta])
        .build()
        .unwrap();
    let own = day(&text, (2025, 11, 11)).day.sunrise.get();
    assert!(
        minutes_apart(own, printed) > 10.0,
        "{}",
        (own - printed) * 1440.0
    );
}
