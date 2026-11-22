//! NEPAL_PANCHANGA's rules that rc.8 moved from a fitted rule to their
//! text's (C2320), each on a year where the old rule and the new part.
//! The expected day is the text's decision over the committee's sky at
//! Kathmandu; the published calendars of Nepal keep it too (C2320's
//! back-test, rows not kept).

#![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

use teistro::catalogue::Calendar;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{CalendarDate, Context, Ephemeris, FestivalPack, FestivalRequest, UtcOffset};

/// The Gregorian days a rule is kept on between two dates, both in.
fn kept(rule: &str, from: (i32, u8, u8), to: (i32, u8, u8)) -> Vec<(i32, u8, u8)> {
    let sdk = Context::builder()
        .profile("nepali-committee")
        .ephemeris([Ephemeris::SuryaSiddhanta])
        .build()
        .unwrap();
    let on = |(year, month, day): (i32, u8, u8)| {
        CalendarDate::defined(Calendar::Gregorian, year, month, day)
    };
    let kathmandu = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );
    sdk.almanac()
        .festivals(
            &on(from),
            &on(to),
            &kathmandu,
            UtcOffset::literal(5, 45, 0),
            &FestivalRequest::from(FestivalPack::NepalPanchanga),
        )
        .unwrap()
        .value
        .observances
        .into_iter()
        .filter(|kept| kept.rule == rule)
        .map(|kept| (kept.day.year, kept.day.month, kept.day.day))
        .collect()
}

#[test]
fn kalki_jayanti_is_the_later_day_only_when_it_alone_holds_the_sunset() {
    // Dharmasindhu's sunset rule; udaya gave each a day later.
    assert_eq!(
        kept("KALKI_JAYANTI", (2019, 8, 1), (2019, 8, 10)),
        [(2019, 8, 5)]
    );
    assert_eq!(
        kept("KALKI_JAYANTI", (2020, 7, 20), (2020, 7, 30)),
        [(2020, 7, 25)]
    );
    assert_eq!(
        kept("KALKI_JAYANTI", (2021, 8, 9), (2021, 8, 18)),
        [(2021, 8, 13)]
    );
}

#[test]
fn tilakunda_chaturthi_is_the_earlier_day_when_it_alone_holds_the_pradosha() {
    // Magha's bright 4th of 2025; the share of madhyahna gave 2 February.
    assert_eq!(
        kept("TILAKUNDA_CHATURTHI", (2025, 1, 28), (2025, 2, 6)),
        [(2025, 2, 1)]
    );
}

#[test]
fn vamana_dwadashi_is_the_later_day_only_when_it_alone_holds_madhyahna() {
    // Bhadrapada's bright 12th of 2020; the share of the first fifth gave
    // 30 August.
    assert_eq!(
        kept("VAMANA_DWADASHI", (2020, 8, 25), (2020, 9, 3)),
        [(2020, 8, 29)]
    );
}

#[test]
fn dvapara_yugadi_holds_the_new_moon_at_aparahna() {
    // Magha's new moon of 2023 (Dharmasindhu p. 94); the share of
    // madhyahna gave 20 February.
    assert_eq!(
        kept("DVAPARA_YUGADI", (2023, 2, 15), (2023, 2, 24)),
        [(2023, 2, 19)]
    );
}

#[test]
fn navami_shraddha_holds_the_aparahna_as_the_fortnight_s_shraddhas_do() {
    // 2019: the dark 9th holds more of the earlier afternoon; udaya gave
    // 23 September.
    assert_eq!(
        kept("NAVAMI_SHRADDHA", (2019, 9, 18), (2019, 9, 27)),
        [(2019, 9, 22)]
    );
}

#[test]
fn gathamuga_charhe_is_the_earlier_day_whenever_the_14th_touches_its_pradosha() {
    // 2023: the 14th begins inside the earlier pradosha; the share gave
    // 16 July. Three calendars of Nepal print the 15th.
    assert_eq!(
        kept("GATHAMUGA_CHARHE", (2023, 7, 11), (2023, 7, 20)),
        [(2023, 7, 15)]
    );
}

#[test]
fn matsyanarayana_mela_is_kept_on_matsya_jayanti_s_day() {
    // Chaitra's bright 3rd at aparahna: in 2022 and 2025 it begins after
    // the earlier sunrise, and udaya gave the day after.
    for (from, to, day) in [
        ((2022, 3, 30), (2022, 4, 8), (2022, 4, 3)),
        ((2025, 3, 27), (2025, 4, 5), (2025, 3, 31)),
    ] {
        assert_eq!(kept("MATSYANARAYANA_MELA", from, to), [day]);
        assert_eq!(kept("MATSYA_JAYANTI", from, to), [day]);
    }
}

#[test]
fn hile_jatra_is_shravana_s_dark_9th() {
    // The committee prints it on 17 August 2025 (VS 2082 p. 10) and
    // 5 September 2026; Bhadrapada's bright 9th prints none.
    assert_eq!(
        kept("HILE_JATRA", (2025, 8, 10), (2025, 9, 10)),
        [(2025, 8, 17)]
    );
    assert_eq!(
        kept("HILE_JATRA", (2026, 8, 30), (2026, 9, 30)),
        [(2026, 9, 5)]
    );
}
