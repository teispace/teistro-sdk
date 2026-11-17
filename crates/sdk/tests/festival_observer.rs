//! `festivals.observer` (`03-design/festival-traditions.md`, SR-116): the
//! tradition a family follows chooses, of each festival a request reads
//! two or more ways, that tradition's reading, by the setting alone.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::catalogue::Calendar;
use teistro::festival::EkadashiRule;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{
    CalendarDate, Context, Ephemeris, FestivalPack, FestivalRequest, NextRequest, UtcOffset,
};

fn context(observer: Option<&str>) -> Context {
    let builder = Context::builder().ephemeris([Ephemeris::Builtin]);
    match observer {
        Some(observer) => builder
            .settings_json(&format!(r#"{{"festivals": {{"observer": "{observer}"}}}}"#))
            .build()
            .unwrap(),
        None => builder.build().unwrap(),
    }
}

fn delhi() -> Place {
    Place::new(
        Latitude::literal(28.6139),
        Longitude::literal(77.209),
        Altitude::literal(216.0),
    )
}

/// The Ekadashi fasts over a range at Delhi, as (rule, day of month).
fn fasts(
    observer: Option<&str>,
    request: &FestivalRequest,
    (from, to): (CalendarDate, CalendarDate),
) -> Vec<(String, u8)> {
    context(observer)
        .almanac()
        .festivals(&from, &to, &delhi(), UtcOffset::literal(5, 30, 0), request)
        .unwrap()
        .value
        .ekadashis
        .into_iter()
        .map(|fast| (fast.rule, fast.day.day))
        .collect()
}

fn september_2023() -> (CalendarDate, CalendarDate) {
    (
        CalendarDate::defined(Calendar::Gregorian, 2023, 9, 20),
        CalendarDate::defined(Calendar::Gregorian, 2023, 9, 30),
    )
}

/// *Dharmasindhu*'s S8 row on a sky that happened (`festival-rules.md`
/// §8.1): Bhadrapada's bright 11th of 2023 at Delhi began after the 25th's
/// sunrise and ended before the 26th's. The published almanac gives the
/// Smarta fast the 25th and the renunciant's and the Vaishnava's the 26th,
/// and each observer keeps its own by the setting alone.
#[test]
fn each_observer_keeps_its_own_ekadashi_of_the_text_s_s8_row() {
    let asked = FestivalRequest::from(FestivalPack::Dharmasindhu);
    let one = |rule: &str, day: u8| vec![(rule.to_owned(), day)];
    assert_eq!(
        fasts(None, &asked, september_2023()),
        [
            ("EKADASHI_VAISHNAVA".to_owned(), 26),
            ("EKADASHI_SMARTA".to_owned(), 25),
            ("EKADASHI_SMARTA_RENUNCIANT".to_owned(), 26),
        ]
    );
    assert_eq!(
        fasts(Some("EVERY"), &asked, september_2023()),
        fasts(None, &asked, september_2023())
    );
    assert_eq!(
        fasts(Some("SMARTA"), &asked, september_2023()),
        one("EKADASHI_SMARTA", 25)
    );
    assert_eq!(
        fasts(Some("VAISHNAVA"), &asked, september_2023()),
        one("EKADASHI_VAISHNAVA", 26)
    );
    assert_eq!(
        fasts(Some("SMARTA_RENUNCIANT"), &asked, september_2023()),
        one("EKADASHI_SMARTA_RENUNCIANT", 26)
    );
    // The pack has no Gaudiya reading, so the Gaudiya keeps the
    // Vaishnava's (C2142).
    assert_eq!(
        fasts(Some("GAUDIYA"), &asked, september_2023()),
        one("EKADASHI_VAISHNAVA", 26)
    );
}

/// Over a year, each observer's fasts are the default answer's fasts of
/// its rule, day for day: the setting drops readings and moves none.
#[test]
fn an_observer_keeps_its_rules_days_of_the_default_answer_and_moves_none() {
    let asked = FestivalRequest::from(FestivalPack::Nepal);
    let year = (
        CalendarDate::defined(Calendar::Gregorian, 2026, 1, 1),
        CalendarDate::defined(Calendar::Gregorian, 2026, 12, 31),
    );
    let every = fasts(None, &asked, year.clone());
    for (observer, rule) in [
        ("SMARTA", "EKADASHI_SMARTA"),
        ("SMARTA_RENUNCIANT", "EKADASHI_SMARTA_RENUNCIANT"),
        ("VAISHNAVA", "EKADASHI_VAISHNAVA"),
    ] {
        let kept: Vec<(String, u8)> = every.iter().filter(|f| f.0 == rule).cloned().collect();
        // Each rule keeps its 24 fasts of 2026 under its own observer.
        assert_eq!(kept.len(), 24, "{rule}");
        assert_eq!(
            fasts(Some(observer), &asked, year.clone()),
            kept,
            "{observer}"
        );
    }
}

/// A Smarta pack beside the Gaudiya one reads Janmashtami, Rama Navami and
/// the Ekadashi both ways; the observer keeps one of each, and a festival
/// read one way (Radhashtami, Gaura Purnima) stands under every observer
/// (C2143).
#[test]
fn a_smarta_pack_beside_the_gaudiya_one_keeps_one_reading_of_each() {
    let asked = FestivalRequest::from_json(r#"{"rules": ["NEPAL", "GAUDIYA"]}"#).unwrap();
    let year = (
        CalendarDate::defined(Calendar::Gregorian, 2026, 1, 1),
        CalendarDate::defined(Calendar::Gregorian, 2026, 12, 31),
    );
    let keys = |observer: Option<&str>| {
        let found = context(observer)
            .almanac()
            .festivals(
                &year.0,
                &year.1,
                &delhi(),
                UtcOffset::literal(5, 30, 0),
                &asked,
            )
            .unwrap()
            .value;
        let mut keys: Vec<String> = found
            .observances
            .iter()
            .map(|o| o.rule.clone())
            .chain(found.ekadashis.iter().map(|f| f.rule.clone()))
            .collect();
        keys.sort();
        keys.dedup();
        keys
    };
    let every = keys(None);
    for key in [
        "JANMASHTAMI",
        "JANMASHTAMI_VAISHNAVA",
        "EKADASHI_GAUDIYA",
        "EKADASHI_SMARTA",
    ] {
        assert!(every.iter().any(|k| k == key), "{key}");
    }
    let smarta = keys(Some("SMARTA"));
    let gaudiya = keys(Some("GAUDIYA"));
    for (kept, left) in [
        ("JANMASHTAMI", "JANMASHTAMI_VAISHNAVA"),
        ("RAMA_NAVAMI", "RAMA_NAVAMI_VAISHNAVA"),
        ("EKADASHI_SMARTA", "EKADASHI_GAUDIYA"),
    ] {
        assert!(smarta.iter().any(|k| k == kept) && !smarta.iter().any(|k| k == left));
        assert!(gaudiya.iter().any(|k| k == left) && !gaudiya.iter().any(|k| k == kept));
    }
    // The Gaudiya's own Ekadashi is carried, so the Vaishnava's goes too.
    assert!(!gaudiya.iter().any(|k| k == "EKADASHI_VAISHNAVA"));
    for one_way in ["RADHASHTAMI_VAISHNAVA", "GAURA_PURNIMA", "VIJAYA_DASHAMI"] {
        assert!(smarta.iter().any(|k| k == one_way), "{one_way}");
        assert!(gaudiya.iter().any(|k| k == one_way), "{one_way}");
    }
}

/// `almanac.next`'s `EKADASHI` kind answers the observer's fast rather than
/// the earliest of three: from 20 September 2023 at Delhi, the Smarta's
/// 25th and the Vaishnava's 26th.
#[test]
fn the_next_ekadashi_is_the_observers() {
    // Midnight of 20 September 2023 at Delhi, in UTC.
    let asked = NextRequest::from_json(
        r#"{"after": 2460207.2708333335, "kind": "EKADASHI", "latitudeDeg": 28.6139,
            "longitudeDeg": 77.209, "altitudeM": 216, "utcOffsetSeconds": 19800}"#,
    )
    .unwrap();
    let next = |observer: Option<&str>| {
        let found = context(observer).almanac().next(&asked).unwrap().value;
        (found.keys, found.day.day)
    };
    let one = |key: &str, day: u8| (vec![key.to_owned()], day);
    assert_eq!(next(None), one("EKADASHI_SMARTA", 25));
    assert_eq!(next(Some("SMARTA")), one("EKADASHI_SMARTA", 25));
    assert_eq!(next(Some("VAISHNAVA")), one("EKADASHI_VAISHNAVA", 26));
    assert_eq!(
        next(Some("SMARTA_RENUNCIANT")),
        one("EKADASHI_SMARTA_RENUNCIANT", 26)
    );
}

/// The knob is a closed set, refused by its field; a rule naming `EVERY`
/// as its reading is refused by its place.
#[test]
fn a_bad_observer_is_refused_by_its_field() {
    let refused = Context::builder()
        .settings_json(r#"{"festivals": {"observer": "SHAIVA"}}"#)
        .build()
        .unwrap_err();
    assert_eq!(refused.field(), Some("settings.festivals.observer"));
    let mut rule = serde_json::to_value(EkadashiRule::dharmasindhu().remove(0)).unwrap();
    rule["key"] = "MINE".into();
    rule["meta"] = serde_json::json!({"variant": {"festival": "EKADASHI", "observer": "EVERY"}});
    let refused = FestivalRequest::from_json(&format!(r#"{{"rules": [{rule}]}}"#)).unwrap_err();
    assert_eq!(
        refused.field(),
        Some("festivals.ekadashis[0].meta.variant.observer")
    );
}
