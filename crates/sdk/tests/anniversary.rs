//! Anniversaries kept by the tithi (`03-design/shraddha.md`) over a real
//! sky, at Kathmandu under the committee's setup: the Sora Shraddha days
//! the committee announced for 2077 to 2079 BS, read as parvana
//! anniversaries of each tithi of the fortnight; origins at a moment, a
//! date and in an adhika month; the first anniversary's month when it is
//! doubled; and a janma tithi's two-muhurta floor.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::catalogue::{Calendar, Masa, Tithi};
use teistro::festival::{
    Anniversary, AnniversaryDay, AnniversaryKind, FirstYearAdhika, Half, JanmaOptions, MonthClause,
    Origin,
};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
use teistro::{CalendarDate, Context, FestivalPack, FestivalRequest, UtcOffset};

fn kathmandu() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    )
}

const NPT: UtcOffset = UtcOffset::literal(5, 45, 0);

fn date(year: i32, month: u8, day: u8) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, year, month, day)
}

/// A UTC Julian day from a Nepal clock reading.
fn npt(year: i32, month: u8, day: u8, hour: f64) -> JulianDay<teistro::quantity::Utc> {
    // Days from 2000-01-01 00:00 UTC (JD 2451544.5), by the proleptic
    // Gregorian count.
    let days = |y: i32, m: u8, d: u8| -> i64 {
        let (y, m) = if m <= 2 {
            (i64::from(y) - 1, i64::from(m) + 12)
        } else {
            (i64::from(y), i64::from(m))
        };
        365 * y + y / 4 - y / 100 + y / 400 + (153 * (m - 3) + 2) / 5 + i64::from(d)
    };
    #[expect(clippy::cast_precision_loss, reason = "a day count well under 2^52")]
    let from_2000 = (days(year, month, day) - days(2000, 1, 1)) as f64;
    JulianDay::literal(2_451_544.5 + from_2000 + (hour - 5.75) / 24.0)
}

fn anniversary(key: &str, origin: Origin, kind: AnniversaryKind) -> Anniversary {
    Anniversary {
        key: key.to_owned(),
        origin,
        kind,
        window: None,
        first_year_adhika: FirstYearAdhika::Adhika,
        janma: JanmaOptions::default(),
    }
}

fn lunar(month: Masa, tithi: Tithi) -> Origin {
    Origin::Lunar {
        month,
        tithi,
        adhika: false,
        half: Half::First,
        at: None,
    }
}

/// The anniversaries kept over whole Gregorian years, a request a year,
/// since a request covers at most 366 days.
fn kept(asked: Vec<Anniversary>, years: core::ops::RangeInclusive<i32>) -> Vec<AnniversaryDay> {
    let sdk = Context::builder().nepal_official([]).build().unwrap();
    let request = asked.into_iter().fold(
        FestivalRequest::new(Vec::new()),
        FestivalRequest::with_anniversary,
    );
    years
        .flat_map(|year| {
            sdk.almanac()
                .festivals(
                    &date(year, 1, 1),
                    &date(year, 12, 31),
                    &kathmandu(),
                    NPT,
                    &request,
                )
                .unwrap()
                .value
                .anniversaries
        })
        .collect()
}

fn day_of(kept: &AnniversaryDay) -> (i32, u8, u8) {
    let day = &kept.observance.day;
    (day.year, day.month, day.day)
}

/// The fortnight's tithis, Bhadrapada's dark half amanta (C672).
const FORTNIGHT: [Tithi; 15] = [
    Tithi::KrishnaPratipada,
    Tithi::KrishnaDvitiya,
    Tithi::KrishnaTritiya,
    Tithi::KrishnaChaturthi,
    Tithi::KrishnaPanchami,
    Tithi::KrishnaShashthi,
    Tithi::KrishnaSaptami,
    Tithi::KrishnaAshtami,
    Tithi::KrishnaNavami,
    Tithi::KrishnaDashami,
    Tithi::KrishnaEkadashi,
    Tithi::KrishnaDwadashi,
    Tithi::KrishnaTrayodashi,
    Tithi::KrishnaChaturdashi,
    Tithi::Amavasya,
];

/// `shraddha.md` §5.1 rows 1 to 12: each day the committee named, and the
/// two days it said hold no shraddha, from the aparahna rule alone.
#[test]
fn the_fortnight_falls_where_the_committee_announced_it() {
    let asked = FORTNIGHT
        .iter()
        .map(|&tithi| {
            anniversary(
                &format!("{tithi:?}"),
                lunar(Masa::Bhadrapada, tithi),
                AnniversaryKind::Parvana,
            )
        })
        .collect();
    let kept = kept(asked, 2020..=2022);
    let on = |tithi: Tithi| -> Vec<(i32, u8, u8)> {
        kept.iter()
            .filter(|kept| kept.origin.tithi == tithi)
            .map(day_of)
            .collect()
    };
    let announced = [
        (
            Tithi::KrishnaPratipada,
            [(2020, 9, 2), (2021, 9, 21), (2022, 9, 11)],
        ),
        (
            Tithi::KrishnaDvitiya,
            [(2020, 9, 3), (2021, 9, 22), (2022, 9, 11)],
        ),
    ];
    for (tithi, days) in announced {
        assert_eq!(on(tithi), days, "{tithi:?}");
    }
    for (tithi, day) in [
        (Tithi::KrishnaTritiya, (2020, 9, 4)),
        (Tithi::KrishnaTritiya, (2021, 9, 23)),
        (Tithi::KrishnaChaturthi, (2021, 9, 24)),
        (Tithi::KrishnaPanchami, (2021, 9, 25)),
        (Tithi::KrishnaSaptami, (2022, 9, 16)),
        (Tithi::KrishnaAshtami, (2022, 9, 18)),
    ] {
        assert!(
            on(tithi).contains(&day),
            "{tithi:?} on {day:?}: {:?}",
            on(tithi)
        );
    }
    for none in [(2020, 9, 5), (2022, 9, 17)] {
        assert!(
            kept.iter().all(|kept| day_of(kept) != none),
            "{none:?} holds no shraddha"
        );
    }
    // Every one is the parvana it was asked as, in the ordinary month.
    assert!(kept.iter().all(|kept| kept.kind == AnniversaryKind::Parvana
        && kept.month_by == MonthClause::Nija
        && kept.count.is_none()));
    assert_eq!(kept.len(), 15 * 3);
}

/// §5.2: a death at 02:00 before sunrise takes the tithi of that hour, the
/// dark 14th, though the civil day's sunrise holds the new moon; a date
/// alone takes that sunrise tithi, marked, with both beside it; and a death
/// in the fortnight is kept as a parvana (C661, C665).
#[test]
fn an_origin_is_the_tithi_of_its_moment() {
    let sdk = Context::builder().nepal_official([]).build().unwrap();
    let found = |origin: Origin| {
        let request = FestivalRequest::new(Vec::new()).with_anniversary(anniversary(
            "ONE",
            origin,
            AnniversaryKind::Shraddha,
        ));
        sdk.almanac()
            .festivals(
                &date(2023, 9, 1),
                &date(2023, 10, 31),
                &kathmandu(),
                NPT,
                &request,
            )
            .unwrap()
            .value
            .anniversaries
            .remove(0)
    };
    let night = found(Origin::Instant {
        at: npt(2022, 9, 25, 2.0),
    });
    assert_eq!(night.origin.tithi, Tithi::KrishnaChaturdashi);
    assert_eq!(night.origin.month, Masa::Bhadrapada);
    assert!(!night.origin.assumed_sunrise && night.origin.candidates.is_empty());
    assert_eq!(night.kind, AnniversaryKind::Parvana);
    assert_eq!(night.count, Some(1));

    let dated = found(Origin::Date {
        date: date(2022, 9, 25),
    });
    assert_eq!(dated.origin.tithi, Tithi::Amavasya);
    assert!(dated.origin.assumed_sunrise);
    assert_eq!(
        dated.origin.candidates,
        [Tithi::KrishnaChaturdashi, Tithi::Amavasya]
    );

    let evening = found(Origin::Instant {
        at: npt(2022, 9, 12, 20.0),
    });
    assert_eq!(evening.origin.tithi, Tithi::KrishnaTritiya);
}

/// C667, Dhakal's case worked in amanta names: a death on 5 June 2026 in
/// the adhika Jyeshtha's dark 5th recurs in the ordinary Jyeshtha's dark
/// 5th, 24 June 2027, and is kept as an ekoddishta.
#[test]
fn an_adhika_origin_recurs_in_the_ordinary_month() {
    let kept = kept(
        vec![anniversary(
            "ADHIKA",
            Origin::Instant {
                at: npt(2026, 6, 5, 12.0),
            },
            AnniversaryKind::Shraddha,
        )],
        2027..=2027,
    );
    assert_eq!(kept.len(), 1);
    let one = &kept[0];
    assert_eq!(
        (one.origin.month, one.origin.tithi, one.origin.kind.key()),
        (Masa::Jyeshtha, Tithi::KrishnaPanchami, "ADHIKA")
    );
    assert_eq!(day_of(one), (2027, 6, 24));
    assert_eq!(
        (one.kind, one.month_by, one.count),
        (AnniversaryKind::Ekoddishta, MonthClause::Nija, Some(1))
    );
}

/// C666: an ordinary origin a year before a doubled month keeps its first
/// anniversary in the adhika by the texts, in the ordinary month by
/// Dhakal's note for Nepal, and its second in the ordinary month either
/// way.
#[test]
fn the_first_anniversary_takes_the_adhika_month_unless_asked_not_to() {
    let origin = Origin::Instant {
        at: npt(2025, 6, 15, 12.0),
    };
    let with = |policy| {
        let mut asked = anniversary("FIRST", origin.clone(), AnniversaryKind::Shraddha);
        asked.first_year_adhika = policy;
        kept(vec![asked], 2026..=2027)
    };
    let texts = with(FirstYearAdhika::Adhika);
    let nepal = with(FirstYearAdhika::Nija);
    assert_eq!(texts.len(), 2);
    assert_eq!(nepal.len(), 2);
    assert_eq!(texts[0].origin.month, Masa::Jyeshtha);
    assert_eq!(
        (
            texts[0].count,
            texts[0].month_by,
            texts[0].observance.adhika
        ),
        (Some(1), MonthClause::FirstYearAdhika, true)
    );
    assert_eq!(
        (
            nepal[0].count,
            nepal[0].month_by,
            nepal[0].observance.adhika
        ),
        (Some(1), MonthClause::Nija, false)
    );
    let apart = nepal[0].observance.tithi.from.get() - texts[0].observance.tithi.from.get();
    assert!((29.0..30.5).contains(&apart), "{apart}");
    assert_eq!(texts[1], nepal[1]);
    assert_eq!(
        (texts[1].count, texts[1].month_by),
        (Some(2), MonthClause::Nija)
    );
}

/// C670: a janma tithi is the sunrise tithi; with *Dharmasindhu*'s floor
/// a sunrise tithi lasting under two muhurtas gives the day before. The
/// dark 5th of Ashadha held 8 July 2023's sunrise for 19 minutes.
#[test]
fn a_janma_tithi_under_two_muhurtas_moves_back_with_the_floor() {
    let born = Origin::Lunar {
        month: Masa::Ashadha,
        tithi: Tithi::KrishnaPanchami,
        adhika: false,
        half: Half::First,
        at: Some(npt(1990, 7, 12, 9.0)),
    };
    let with = |floor| {
        let mut asked = anniversary("BIRTH", born.clone(), AnniversaryKind::JanmaTithi);
        asked.janma.two_muhurta_floor = floor;
        kept(vec![asked], 2023..=2023)
    };
    let plain = with(false);
    let floored = with(true);
    assert_eq!(day_of(&plain[0]), (2023, 7, 8));
    assert_eq!(day_of(&floored[0]), (2023, 7, 7));
    assert_eq!(plain[0].count, Some(33));
    assert_eq!(plain[0].kind, AnniversaryKind::JanmaTithi);
}

/// A binding writes an anniversary as an item of `rules` with an `origin`,
/// and a refusal names its field.
#[test]
fn an_anniversary_is_read_from_a_bindings_request() {
    let request = FestivalRequest::from_json(
        r#"{"rules": [{"key": "GRANDFATHER", "origin": {"by": "LUNAR", "month": "masa.KARTIKA", "tithi": "SHUKLA_DASHAMI"}, "kind": "PARVANA"}]}"#,
    )
    .unwrap();
    assert_eq!(request.anniversaries()[0].kind, AnniversaryKind::Parvana);
    let wrong = FestivalRequest::from_json(
        r#"{"rules": [{"key": "BIRTH", "origin": {"by": "LUNAR", "month": "KARTIKA", "tithi": "SHUKLA_DASHAMI"}, "kind": "JANMA_TITHI", "window": {"window": "SUNSET"}}]}"#,
    )
    .unwrap_err();
    assert_eq!(wrong.field(), Some("festivals.rules[0].window"));
}

/// The `PITRI_PAKSHA` pack (`shraddha.md` §3.2) keeps each mahalaya where
/// the committee announced its Sora Shraddha (§5.1 rows 1 to 12) and where
/// the same tithi's parvana anniversary falls, so the two readings of the
/// fortnight are one; the special days share their tithi's day.
#[test]
fn the_pitri_paksha_pack_keeps_the_announced_days() {
    let sdk = Context::builder().nepal_official([]).build().unwrap();
    let observed: Vec<(String, (i32, u8, u8))> = (2020..=2022)
        .flat_map(|year| {
            sdk.almanac()
                .festivals(
                    &date(year, 8, 1),
                    &date(year, 10, 31),
                    &kathmandu(),
                    NPT,
                    &FestivalRequest::from(FestivalPack::PitriPaksha),
                )
                .unwrap()
                .value
                .observances
        })
        .map(|kept| {
            let day = &kept.day;
            (kept.rule.clone(), (day.year, day.month, day.day))
        })
        .collect();
    let on = |key: &str| -> Vec<(i32, u8, u8)> {
        observed
            .iter()
            .filter(|(rule, _)| rule == key)
            .map(|&(_, day)| day)
            .collect()
    };
    assert_eq!(
        on("PRATIPADA_SHRADDHA"),
        [(2020, 9, 2), (2021, 9, 21), (2022, 9, 11)]
    );
    assert_eq!(
        on("DWITIYA_SHRADDHA"),
        [(2020, 9, 3), (2021, 9, 22), (2022, 9, 11)]
    );
    for (key, day) in [
        ("TRITIYA_SHRADDHA", (2020, 9, 4)),
        ("TRITIYA_SHRADDHA", (2021, 9, 23)),
        ("CHATURTHI_SHRADDHA", (2021, 9, 24)),
        ("PANCHAMI_SHRADDHA", (2021, 9, 25)),
        ("SAPTAMI_SHRADDHA", (2022, 9, 16)),
        ("ASHTAMI_SHRADDHA", (2022, 9, 18)),
    ] {
        assert!(on(key).contains(&day), "{key} on {day:?}: {:?}", on(key));
    }
    // The fortnight's mahalayas, one a tithi; the 14th is the killed by a
    // weapon's alone.
    let mahalaya = |rule: &str| {
        rule.ends_with("_SHRADDHA")
            && rule != "CHATURDASHI_SHRADDHA"
            && rule != "MAGHA_SHRADDHA"
            && rule != "BHARANI_SHRADDHA"
    };
    let mahalayas = |day| {
        observed
            .iter()
            .any(|(rule, on)| *on == day && mahalaya(rule))
    };
    assert!(!mahalayas((2020, 9, 5)) && !mahalayas((2022, 9, 17)));
    for (special, tithi) in [
        ("AVIDHAVA_NAVAMI", "NAVAMI_SHRADDHA"),
        ("SANYASI_MAHALAYA", "DWADASHI_SHRADDHA"),
        ("MAGHA_SHRADDHA", "TRAYODASHI_SHRADDHA"),
    ] {
        assert_eq!(on(special), on(tithi), "{special}");
        assert_eq!(on(special).len(), 3, "{special}");
    }
    assert_eq!(on("CHATURDASHI_SHRADDHA").len(), 3);
    assert_eq!(on("BHARANI_SHRADDHA").len(), 3);
    // Read back through the anniversaries: each tithi's parvana falls on
    // its mahalaya's day.
    let asked = FORTNIGHT
        .iter()
        .filter(|&&tithi| tithi != Tithi::KrishnaChaturdashi)
        .map(|&tithi| {
            anniversary(
                &format!("{tithi:?}"),
                lunar(Masa::Bhadrapada, tithi),
                AnniversaryKind::Parvana,
            )
        })
        .collect();
    let mut by_anniversary: Vec<(i32, u8, u8)> =
        kept(asked, 2020..=2022).iter().map(day_of).collect();
    let mut by_pack: Vec<(i32, u8, u8)> = observed
        .iter()
        .filter(|(rule, _)| mahalaya(rule))
        .map(|&(_, day)| day)
        .collect();
    by_anniversary.sort_unstable();
    by_pack.sort_unstable();
    assert_eq!(by_pack, by_anniversary);
}
