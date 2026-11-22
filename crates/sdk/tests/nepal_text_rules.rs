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

#[test]
fn a_named_ekadashi_is_the_fortnight_s_ekadashi_as_the_print_keeps_it() {
    // Pierced at sunrise with the 11th alone in excess (S2): the later day,
    // against Dharmasindhu's householder (C2322).
    for (rule, from, to, day) in [
        (
            "AMALAKI_EKADASHI",
            (2020, 3, 1),
            (2020, 3, 10),
            (2020, 3, 6),
        ),
        (
            "UTPATTIKA_EKADASHI",
            (2020, 12, 6),
            (2020, 12, 15),
            (2020, 12, 11),
        ),
        (
            "VIJAYA_EKADASHI",
            (2022, 2, 22),
            (2022, 3, 3),
            (2022, 2, 27),
        ),
        (
            "MOKSHADA_EKADASHI",
            (2023, 12, 18),
            (2023, 12, 27),
            (2023, 12, 23),
        ),
        (
            "SHRAWAN_PUTRADA_EKADASHI",
            (2024, 8, 11),
            (2024, 8, 20),
            (2024, 8, 16),
        ),
    ] {
        assert_eq!(kept(rule, from, to), [day], "{rule}");
    }
    // Pure with both in excess (S5, by Madhava): the later day, where the
    // udaya rule gave the earlier, and every calendar prints the later.
    assert_eq!(
        kept("MOKSHADA_EKADASHI", (2019, 12, 3), (2019, 12, 12)),
        [(2019, 12, 8)]
    );
}

#[test]
fn darsha_shraddha_holds_the_new_moon_at_aparahna() {
    // Dharmasindhu p. 266; each new moon below ends before the later
    // day's aparahna, so the sunrise's rule gave the day after, and
    // Nepal's calendars print the earlier.
    for (from, to, day) in [
        ((2023, 4, 15), (2023, 4, 24), (2023, 4, 19)),
        ((2025, 11, 15), (2025, 11, 24), (2025, 11, 19)),
        ((2026, 3, 14), (2026, 3, 23), (2026, 3, 18)),
    ] {
        assert_eq!(kept("DARSHA_SHRADDHA", from, to), [day]);
    }
}

#[test]
fn the_monthly_ashtami_vrata_is_the_later_bright_8th_and_the_dark_8th_at_pradosha() {
    // Dharmasindhu p. 10: a bright 8th the later day; a dark one, kept for
    // Kalabhairava, the day whose pradosha holds more of it. Each pair
    // parts, so the other half's rule would give the other day.
    for (from, to, day) in [
        ((2023, 9, 19), (2023, 9, 27), (2023, 9, 23)),
        ((2026, 5, 20), (2026, 5, 28), (2026, 5, 24)),
    ] {
        assert_eq!(kept("SHUKLA_ASHTAMI_VRATA", from, to), [day]);
    }
    for (from, to, day) in [
        ((2023, 6, 6), (2023, 6, 14), (2023, 6, 10)),
        ((2025, 10, 9), (2025, 10, 17), (2025, 10, 13)),
        ((2026, 8, 1), (2026, 8, 9), (2026, 8, 5)),
    ] {
        assert_eq!(kept("KRISHNA_ASHTAMI_VRATA", from, to), [day]);
    }
}

#[test]
fn bhai_tika_keeps_the_aparahna_where_the_2nd_barely_holds_the_later_sunrise() {
    // Dharmasindhu p. 79 (C2329): the 2nd holds the later sunrise by 6
    // and 42 minutes, and Nepal's calendars print the earlier day, which
    // holds the aparahna; the sunrise's reading gave the day after.
    assert_eq!(
        kept("YAMA_DWITIYA", (2019, 10, 25), (2019, 11, 2)),
        [(2019, 10, 29)]
    );
    assert_eq!(
        kept("YAMA_DWITIYA", (2020, 11, 12), (2020, 11, 20)),
        [(2020, 11, 16)]
    );
}

#[test]
fn the_tulasi_days_and_vyasa_jayanti_keep_their_festivals_days() {
    // Nepal's calendars print तुलसी रोपण on Harishayani, तुलसीपिए the day
    // after, and व्यास जयन्ती on Guru Purnima (2024).
    assert_eq!(
        kept("TULASI_ROPANA", (2024, 7, 10), (2024, 7, 25)),
        kept("HARISHAYANI_EKADASHI", (2024, 7, 10), (2024, 7, 25))
    );
    assert_eq!(
        kept("TULASI_PIYE", (2024, 7, 10), (2024, 7, 25)),
        [(2024, 7, 18)]
    );
    assert_eq!(
        kept("VYASA_JAYANTI", (2024, 7, 15), (2024, 7, 25)),
        kept("GURU_PURNIMA", (2024, 7, 15), (2024, 7, 25))
    );
}
