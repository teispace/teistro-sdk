//! `NEPAL_PANCHANGA`'s rules that rc.8 moved from a fitted rule to their
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
fn dvapara_yugadi_keeps_the_print_s_midday_over_the_text_s_aparahna() {
    // Dharmasindhu p. 94's aparahna gives 7 March 2027, which no calendar
    // of Nepal prints: the committee's VS 2083 book and the published
    // calendars keep the 8th, whose midday holds the new moon. In 2023
    // the calendars split between the 19th and this 20th.
    assert_eq!(
        kept("DVAPARA_YUGADI", (2027, 3, 3), (2027, 3, 12)),
        [(2027, 3, 8)]
    );
    assert_eq!(
        kept("DVAPARA_YUGADI", (2023, 2, 15), (2023, 2, 24)),
        [(2023, 2, 20)]
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
fn ramanujacharya_jayanti_is_ardra_in_vaishakha_s_bright_half() {
    // C2330: in 2021 Tamil Chittirai's Ardra fell on 18 April and Nepal's
    // calendars print Vaishakha's, 16 May.
    assert_eq!(
        kept("RAMANUJACHARYA_JAYANTI", (2021, 4, 10), (2021, 5, 31)),
        [(2021, 5, 16)]
    );
    assert_eq!(
        kept("RAMANUJACHARYA_JAYANTI", (2022, 4, 10), (2022, 5, 31)),
        [(2022, 5, 5)]
    );
    // The national panchanga, VS 2083 p. 3: Baishakh 9.
    assert_eq!(
        kept("RAMANUJACHARYA_JAYANTI", (2026, 4, 10), (2026, 5, 31)),
        [(2026, 4, 22)]
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

#[test]
fn gorakhkali_puja_is_the_bright_8th_s_later_day_and_the_dark_8th_s_at_madhyahna() {
    // The national panchanga, VS 2083: p. 5 the bright 8th's later day;
    // p. 4 the dark 8th's earlier day, the 8th ending before the later
    // madhyahna; p. 10 its later day, a day after the ashtami vrata.
    assert_eq!(
        kept("SHUKLA_GORAKHKALI_PUJA", (2026, 5, 20), (2026, 5, 28)),
        [(2026, 5, 24)]
    );
    assert_eq!(
        kept("KRISHNA_GORAKHKALI_PUJA", (2026, 5, 5), (2026, 5, 13)),
        [(2026, 5, 9)]
    );
    assert_eq!(
        kept("KRISHNA_GORAKHKALI_PUJA", (2026, 8, 1), (2026, 8, 9)),
        [(2026, 8, 6)]
    );
}

#[test]
fn chhaya_darshan_keeps_a_kshaya_14th_on_the_day_it_runs_in() {
    // Magha's bright 14th of VS 2082 runs from 07:33 on 31 January 2026 to
    // 05:40 the next morning, held by no sunrise; the national panchanga,
    // VS 2082 p. 21, prints श्रीपशुपतिनाथको छाँयादर्शन on the 31st, the day
    // it runs in, beside सौभाग्य शूर्पदान, the 13th holding that sunrise
    // (C2347). Udaya's fallback is the earlier day, the 14th's own.
    assert_eq!(
        kept("PASHUPATINATH_CHHAYA_DARSHAN", (2026, 1, 25), (2026, 2, 5)),
        [(2026, 1, 31)]
    );
    assert_eq!(
        kept("SAUBHAGYA_SHURPADAN", (2026, 1, 25), (2026, 2, 5)),
        [(2026, 1, 31)]
    );
    // Republica reports the darshan on 11 February 2025; VS 2083 p. 23
    // prints 20 February 2027, the 14th's sunrise day.
    assert_eq!(
        kept("PASHUPATINATH_CHHAYA_DARSHAN", (2025, 2, 5), (2025, 2, 15)),
        [(2025, 2, 11)]
    );
    assert_eq!(
        kept("PASHUPATINATH_CHHAYA_DARSHAN", (2027, 2, 15), (2027, 2, 25)),
        [(2027, 2, 20)]
    );
}

#[test]
fn saraswati_is_invoked_on_the_day_whose_sunrise_mula_holds() {
    // Dharmasindhu p. 68's Mula in Ashvina's bright half, at sunrise
    // (C2350): 22 October 2020, which Mula holds 82 minutes past sunrise;
    // VS 2082 p. 13's 29 September 2025; VS 2083 p. 15's 17 October 2026.
    assert_eq!(
        kept("SARASWATI_AVAHAN", (2020, 10, 15), (2020, 10, 30)),
        [(2020, 10, 22)]
    );
    assert_eq!(
        kept("SARASWATI_AVAHAN", (2025, 9, 22), (2025, 10, 6)),
        [(2025, 9, 29)]
    );
    assert_eq!(
        kept("SARASWATI_AVAHAN", (2026, 10, 10), (2026, 10, 24)),
        [(2026, 10, 17)]
    );
}

#[test]
fn a_manvadi_keeps_the_sunrise_where_the_text_s_forenoon_parts() {
    // Dharmasindhu p. 32 keeps a bright fortnight's manvadi in the
    // forenoon, which gives 1 April and 13 September 2026; the national
    // panchanga prints मन्वादि on the day whose sunrise holds the tithi
    // (VS 2082 p. 25, VS 2083 p. 13; C2351).
    assert_eq!(
        kept("MANVADI_CHAITRA_PURNIMA", (2026, 3, 28), (2026, 4, 6)),
        [(2026, 4, 2)]
    );
    assert_eq!(
        kept(
            "MANVADI_BHADRAPADA_SHUKLA_TRITIYA",
            (2026, 9, 10),
            (2026, 9, 18)
        ),
        [(2026, 9, 14)]
    );
}

#[test]
fn the_phalguna_dark_1st_s_three_days_keep_its_sunrise() {
    // The national panchanga, VS 2082 p. 24 and VS 2083 p. 26: the mango
    // blossom, Gangaur's beginning and Nala's bath on the dark 1st's
    // sunrise day, 4 March 2026 and 23 March 2027 (C2352, C2353).
    for rule in [
        "AANPKO_MUJURA_KHANE_DIN",
        "GANGAUR_PUJA_ARAMBHA",
        "NALA_MACHHINDRANATH_SNAN",
    ] {
        assert_eq!(kept(rule, (2026, 3, 1), (2026, 3, 8)), [(2026, 3, 4)]);
        assert_eq!(
            kept(rule, (2027, 3, 19), (2027, 3, 26)),
            [(2027, 3, 23)],
            "{rule}"
        );
    }
}

#[test]
fn the_days_printed_on_a_festival_s_day_keep_it() {
    // Gurhi on Nag panchami, which Kantipur reports for 29 July 2025;
    // Banepa's chariot on Buddha jayanti and Nimbarka's jayanti on
    // Kartika's full moon, in those rows of the national panchanga
    // (C2348, C2349, C2352).
    assert_eq!(
        kept("GURHI_PARVA", (2025, 7, 20), (2025, 8, 5)),
        [(2025, 7, 29)]
    );
    for (rule, base, from, to) in [
        ("GURHI_PARVA", "NAG_PANCHAMI", (2025, 7, 20), (2025, 8, 5)),
        (
            "BANEPA_CHANDESHWARI_JATRA",
            "BUDDHA_JAYANTI",
            (2025, 5, 5),
            (2025, 5, 20),
        ),
        (
            "NIMBARKACHARYA_JAYANTI",
            "KARTIK_SNAN_ENDS",
            (2025, 11, 1),
            (2025, 11, 10),
        ),
    ] {
        let days = kept(rule, from, to);
        assert_eq!(days.len(), 1, "{rule}");
        assert_eq!(days, kept(base, from, to), "{rule}");
    }
}
