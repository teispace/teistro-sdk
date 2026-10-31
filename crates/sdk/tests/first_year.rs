//! A death's first year at Kathmandu (`03-design/shraddha.md` §3.3), held
//! to the *Dharmasindhu*'s own counts (p. 331): sixteen shraddhas, or
//! seventeen when a month of the year is doubled; sixteen of them done
//! again after an eleventh-day sapindana, fifteen after a twelfth-day one,
//! thirteen after one at the traipakshika, and none at the year's end.
//! The eleventh day is the ashauch's, under every night rule.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index the JSON they read"
)]

use teistro::ashauch::{
    AshauchEvent, AshauchRequest, AshauchTable, Deceased, Nearness, NightRule, Relation,
};
use teistro::festival::FirstYearAdhika;
use teistro::first_year::{
    FirstAbdika, FirstYear, FirstYearRite, FirstYearSchedule, Rite, RiteForm, SapindanaAt, UnaDay,
};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{Context, Ephemeris, FirstYearQuery, UtcOffset};

/// Nepal time, five hours forty-five minutes east.
const NPT: f64 = (5.0 + 45.0 / 60.0) / 24.0;

/// A Nepal-time instant as a UTC Julian day: `jd0` the UTC Julian day of
/// the date's 00:00 UTC.
fn npt(jd0: f64, hour: u8, minute: u8) -> JulianDay<Utc> {
    let time = (f64::from(hour) + f64::from(minute) / 60.0) / 24.0;
    JulianDay::try_new(jd0 + time - NPT).unwrap()
}

/// 12 January 2026, 00:00 UTC.
const JAN_12_2026: f64 = 2_461_052.5;
/// 20 October 2026, 00:00 UTC: a year with no doubled month after it.
const OCT_20_2026: f64 = 2_461_333.5;
/// 10 June 2025, 00:00 UTC: Jyeshtha, doubled the next year.
const JUN_10_2025: f64 = 2_460_836.5;

fn context() -> Context {
    Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .settings_json(
            r#"{"day": {"sunrise": {"kind": "NAMED", "which": "UPPER_LIMB_REFRACTION"}}}"#,
        )
        .build()
        .unwrap()
}

fn kathmandu() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1337.0),
    )
}

const OFFSET: UtcOffset = UtcOffset::literal(5, 45, 0);

fn schedule(asked: &FirstYear) -> FirstYearSchedule {
    context()
        .almanac()
        .first_year(asked, &kathmandu(), OFFSET)
        .unwrap()
        .value
}

/// The run's day count between two dates of one schedule.
fn days_between(from: &FirstYearRite, to: &FirstYearRite) -> f64 {
    let at = |rite: &FirstYearRite| rite.observance.as_ref().map(|kept| kept.tithi.from.get());
    at(to).unwrap() - at(from).unwrap()
}

fn sixteen(year: &FirstYearSchedule) -> Vec<&FirstYearRite> {
    year.rites
        .iter()
        .filter(|rite| rite.place.is_some())
        .collect()
}

fn of(year: &FirstYearSchedule, rite: Rite) -> Vec<&FirstYearRite> {
    year.rites.iter().filter(|kept| kept.rite == rite).collect()
}

fn drawn(year: &FirstYearSchedule) -> usize {
    year.rites
        .iter()
        .filter(|rite| rite.drawn_to.is_some())
        .count()
}

#[test]
fn an_ordinary_year_keeps_the_sixteen_in_the_texts_order() {
    let year = schedule(&FirstYear::dharmasindhu(
        "grandfather",
        npt(OCT_20_2026, 10, 0),
    ));
    let places: Vec<u8> = sixteen(&year)
        .iter()
        .map(|rite| rite.place.unwrap())
        .collect();
    assert_eq!(places, (1..=16).collect::<Vec<_>>(), "{year:#?}");
    let monthly = of(&year, Rite::Masika);
    assert_eq!(
        monthly
            .iter()
            .map(|rite| rite.month.unwrap())
            .collect::<Vec<_>>(),
        (2..=12).collect::<Vec<_>>()
    );
    for pair in monthly.windows(2) {
        let lunation = days_between(pair[0], pair[1]);
        assert!((29.0..30.1).contains(&lunation), "{lunation}");
    }
    assert!(monthly.iter().all(|rite| !rite.adhika));
    // The first monthly on the eleventh day, the sapindana the next.
    let adya = of(&year, Rite::AdyaMasika)[0];
    assert_eq!(adya.day, year.eleventh_day);
    let sapindana = of(&year, Rite::Sapindana)[0];
    assert_eq!(sapindana.day, year.sapindana);
    assert_eq!(
        (sapindana.day.month, sapindana.day.day),
        (adya.day.month, adya.day.day + 1)
    );
    // Madhava's ūna rites on the day before the day each is short of.
    let before = |rite: Rite, of_rite: &FirstYearRite| {
        let una = of(&year, rite)[0];
        let at = year.rites.iter().position(|kept| kept == una).unwrap();
        let next = year.rites.iter().position(|kept| kept == of_rite).unwrap();
        assert!(at < next, "{rite:?}");
        una.day.clone()
    };
    let abdika = of(&year, Rite::Abdika)[0];
    assert_eq!(abdika.year, Some(1));
    for (una, short_of) in [
        (Rite::UnaMasika, monthly[0]),
        (Rite::UnaShanmasika, monthly[5]),
        (Rite::UnaAbdika, abdika),
    ] {
        let day = before(una, short_of);
        assert_ne!(day, short_of.day, "{una:?}");
    }
    // The traipakshika three fortnights on, in the other paksha.
    let traipakshika = of(&year, Rite::Traipakshika)[0];
    let since =
        traipakshika.observance.as_ref().unwrap().tithi.from.get() - npt(OCT_20_2026, 10, 0).get();
    assert!((40.0..47.0).contains(&since), "{since}");
    // A year on.
    let year_on =
        abdika.observance.as_ref().unwrap().tithi.from.get() - npt(OCT_20_2026, 10, 0).get();
    assert!((340.0..390.0).contains(&year_on), "{year_on}");
}

#[test]
fn the_sapindanas_day_sets_how_many_are_done_again() {
    let at = npt(OCT_20_2026, 10, 0);
    let under = |sapindana| {
        schedule(&FirstYear {
            sapindana,
            ..FirstYear::dharmasindhu("grandfather", at)
        })
    };
    // A rite done again in its own time is judged at aparahna, so its
    // extents are not the ekoddishta's at kutapa to rauhina.
    let (twelfth, year_end) = (under(SapindanaAt::TwelfthDay), under(SapindanaAt::YearEnd));
    let extents = |year: &FirstYearSchedule, place: u8| {
        year.rites
            .iter()
            .find(|rite| rite.place == Some(place))
            .and_then(|rite| rite.observance.as_ref())
            .map(|kept| kept.extents.clone())
    };
    assert!((3..=15).any(|place| extents(&twelfth, place) != extents(&year_end, place)));
    for (sapindana, again) in [
        (SapindanaAt::EleventhDay, 16),
        (SapindanaAt::TwelfthDay, 15),
        (SapindanaAt::Traipakshika, 13),
        (SapindanaAt::YearEnd, 0),
    ] {
        let year = under(sapindana);
        assert_eq!(drawn(&year), again, "{sapindana:?}");
        for rite in &year.rites {
            if rite.place.is_some() {
                let expected = if rite.drawn_to.is_some() {
                    assert_eq!(rite.drawn_to.as_ref(), Some(&year.sapindana));
                    RiteForm::Parvana
                } else {
                    RiteForm::Ekoddishta
                };
                assert_eq!(rite.form, Some(expected), "{sapindana:?} {rite:?}");
            }
        }
    }
}

#[test]
fn a_doubled_month_in_the_year_makes_seventeen() {
    let year = schedule(&FirstYear::dharmasindhu(
        "grandfather",
        npt(JAN_12_2026, 1, 0),
    ));
    let places = sixteen(&year);
    assert_eq!(places.len(), 17, "{year:#?}");
    let monthly = of(&year, Rite::Masika);
    let doubled: Vec<_> = monthly.iter().filter(|rite| rite.adhika).collect();
    assert_eq!(doubled.len(), 1);
    let month = doubled[0].month;
    assert_eq!(monthly.iter().filter(|rite| rite.month == month).count(), 2);
    // Fifteen done again, and the doubled month's second too.
    assert_eq!(drawn(&year), 16);
}

#[test]
fn the_eleventh_day_is_the_ashauchs_under_every_night_rule() {
    // K1: a death at 01:00 on 12 January 2026, in the night's last third.
    let at = npt(JAN_12_2026, 1, 0);
    for night in [
        NightRule::Thirds,
        NightRule::HalfNight,
        NightRule::CivilMidnight,
        NightRule::Sunrise,
    ] {
        let year = schedule(&FirstYear {
            night,
            ..FirstYear::dharmasindhu("k1", at)
        });
        let table = AshauchTable {
            night,
            ..AshauchTable::dharmasindhu()
        };
        let asked = AshauchRequest {
            key: "k1".to_owned(),
            event: AshauchEvent::Death {
                at,
                cremated_at: None,
                ahitagni: false,
                deceased: Deceased::default(),
            },
            relation: Relation::Sapinda,
            heard_at: None,
            abroad: false,
            nearness: Nearness::SameVillage,
            full_days: None,
            during: Vec::new(),
            fire_giver: false,
        };
        let ashauch = context()
            .almanac()
            .ashauch(&table, &asked, &kathmandu(), OFFSET)
            .unwrap()
            .value;
        assert_eq!(year.day_one, ashauch.day_one, "{night:?}");
        // The ten days' end is the eleventh day's sunrise.
        let last = ashauch.last_day.unwrap();
        assert_eq!(
            (year.eleventh_day.month, year.eleventh_day.day),
            (last.month, last.day + 1),
            "{night:?}"
        );
    }
}

#[test]
fn nepal_keeps_the_first_abdika_on_the_eleventh_day_and_the_barkhi_in_the_ordinary_month() {
    // A death in Jyeshtha 2025, the month doubled in 2026.
    let at = npt(JUN_10_2025, 10, 0);
    let texts = schedule(&FirstYear::dharmasindhu("father", at));
    let nepal = schedule(&FirstYear::nepal("father", at));
    let anniversary = |year: &FirstYearSchedule| {
        of(year, Rite::Abdika)
            .into_iter()
            .find(|rite| rite.observance.is_some())
            .unwrap()
            .clone()
    };
    let (adhika, nija) = (anniversary(&texts), anniversary(&nepal));
    assert!(adhika.adhika && !nija.adhika);
    assert_eq!((adhika.year, nija.year), (Some(1), Some(2)));
    let month = days_between(&adhika, &nija);
    assert!((29.0..30.1).contains(&month), "{month}");
    let first = of(&nepal, Rite::Abdika)[0];
    assert_eq!((first.year, &first.day), (Some(1), &nepal.eleventh_day));
    assert_eq!(first.form, Some(RiteForm::Ekoddishta));
    // The texts' ūnābdika is short of the adhika anniversary; Nepal's of
    // the barkhi.
    let una = |year: &FirstYearSchedule| of(year, Rite::UnaAbdika)[0].day.clone();
    assert_ne!(una(&texts), una(&nepal));
    assert_eq!(
        FirstYear {
            first_abdika: FirstAbdika::EleventhDay,
            first_year_adhika: FirstYearAdhika::Nija,
            ..FirstYear::dharmasindhu("father", at)
        },
        FirstYear::nepal("father", at)
    );
}

#[test]
fn an_una_rite_kept_tithis_short_falls_on_that_tithi() {
    let at = npt(OCT_20_2026, 10, 0);
    let by_day = schedule(&FirstYear::dharmasindhu("grandfather", at));
    let short_of = |year: &FirstYearSchedule, rite: Rite| {
        let found = match rite {
            Rite::UnaMasika => of(year, Rite::Masika)[0],
            Rite::UnaShanmasika => of(year, Rite::Masika)[5],
            _ => of(year, Rite::Abdika)[0],
        };
        found.observance.as_ref().unwrap().tithi.from.get()
    };
    for (tithis, una) in [
        (1.0, UnaDay::OneTithi),
        (2.0, UnaDay::TwoTithis),
        (3.0, UnaDay::ThreeTithis),
    ] {
        let year = schedule(&FirstYear {
            una,
            ..FirstYear::dharmasindhu("grandfather", at)
        });
        for rite in [Rite::UnaMasika, Rite::UnaShanmasika, Rite::UnaAbdika] {
            let kept = of(&year, rite)[0];
            let observance = kept.observance.as_ref().unwrap();
            assert_eq!(observance.day, kept.day);
            // A tithi is 0.9 to 1.1 days.
            let ahead = short_of(&year, rite) - observance.tithi.from.get();
            assert!(
                (0.85 * tithis..1.15 * tithis).contains(&ahead),
                "{una:?} {rite:?} {ahead}"
            );
            // Never later than Madhava's day before.
            let madhava = &of(&by_day, rite)[0].day;
            assert!(
                (kept.day.year, kept.day.month, kept.day.day)
                    <= (madhava.year, madhava.month, madhava.day),
                "{una:?} {rite:?}"
            );
        }
    }
}

#[test]
fn a_binding_query_is_the_same_answer_and_names_a_refusal_under_its_record() {
    let asked = FirstYearQuery::from_json(
        r#"{"latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1337,
            "utcOffsetSeconds": 20700,
            "request": {"key": "grandfather", "at": 2461333.677083}}"#,
    )
    .unwrap();
    let sdk = context();
    let found = sdk.almanac().first_year_query(&asked).unwrap();
    let kernel = schedule(&FirstYear::dharmasindhu(
        "grandfather",
        JulianDay::literal(2_461_333.677_083),
    ));
    assert_eq!(found.value, kernel.in_full().unwrap());
    assert_eq!(found.value["sapindana"]["calendar"], "calendar.GREGORIAN");
    // Every member written in full is the serde spelling under its kind.
    let plain = serde_json::to_value(&kernel).unwrap();
    let mut checked = 0;
    for (path, _) in FirstYearSchedule::MEMBERS {
        let (head, tail) = path.split_once('.').unwrap();
        let pairs: Vec<(&serde_json::Value, &serde_json::Value)> =
            match (found.value[head].as_array(), plain[head].as_array()) {
                (Some(full), Some(bare)) => full.iter().zip(bare).collect(),
                _ => vec![(&found.value[head], &plain[head])],
            };
        for (full, bare) in pairs {
            let at = format!("/{}", tail.replace('.', "/"));
            if let (Some(full), Some(bare)) = (full.pointer(&at), bare.pointer(&at))
                && let (Some(full), Some(bare)) = (full.as_str(), bare.as_str())
            {
                assert_eq!(full.split_once('.').unwrap().1, bare, "{path}");
                checked += 1;
            }
        }
    }
    assert!(checked > 40, "{checked}");
    let refused = FirstYearQuery::from_json(
        r#"{"latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700,
            "request": {"key": " ", "at": 2461333.677083}}"#,
    )
    .unwrap_err();
    assert_eq!(refused.field(), Some("firstYear.request.key"), "{refused}");
}
