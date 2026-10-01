//! Festival rules through the façade (`03-design/festival-rules.md` §4.3):
//! a year's observances over days founded once, each rule once, nothing
//! left unjudged, the widening declared, and a bad rule refused by the
//! place it holds in the request.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::catalogue::Calendar;
use teistro::festival::{Edge, FestivalRule, Guard, Predicate, Which};
use teistro::muhurta::ActivityRules;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{
    AlmanacRequest, CalendarDate, Context, Ephemeris, FestivalPack, FestivalRequest,
    MuhurtaRequest, UtcOffset,
};

fn context() -> Context {
    Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
}

fn delhi() -> Place {
    Place::new(
        Latitude::literal(28.6139),
        Longitude::literal(77.209),
        Altitude::literal(216.0),
    )
}

fn year(year: i32) -> (CalendarDate, CalendarDate) {
    (
        CalendarDate::defined(Calendar::Gregorian, year, 1, 1),
        CalendarDate::defined(Calendar::Gregorian, year, 12, 31),
    )
}

#[test]
fn a_year_holds_each_shipped_rule_once_in_its_season() {
    let (from, to) = year(2026);
    let found = context()
        .almanac()
        .festivals(
            &from,
            &to,
            &delhi(),
            UtcOffset::literal(5, 30, 0),
            &FestivalRequest::from(FestivalPack::Dharmasindhu),
        )
        .unwrap();
    assert!(
        found.value.unjudged.is_empty(),
        "{:?}",
        found.value.unjudged
    );
    // Each in the Gregorian months its lunar month reaches.
    let shipped = FestivalRule::dharmasindhu();
    let seasons = [
        ("RAMA_NAVAMI", 3..=4),
        ("JANMASHTAMI", 8..=9),
        ("VIJAYA_DASHAMI", 9..=10),
        ("LAKSHMI_PUJA", 10..=11),
        ("HARITALIKA", 8..=9),
        ("NAVARATRA_ARAMBHA", 9..=10),
        ("YAMA_DWITIYA", 10..=11),
        ("SHIVARATRI", 2..=3),
    ];
    assert_eq!(
        seasons.iter().map(|(rule, _)| *rule).collect::<Vec<_>>(),
        shipped
            .iter()
            .map(|rule| rule.key.as_str())
            .collect::<Vec<_>>(),
        "every shipped rule has its season here"
    );
    for (rule, months) in seasons {
        let days: Vec<_> = found
            .value
            .observances
            .iter()
            .filter(|observance| observance.rule == rule)
            .collect();
        assert_eq!(days.len(), 1, "{rule}: {days:?}");
        assert!(
            months.contains(&days[0].day.month),
            "{rule} on {}",
            days[0].day
        );
    }
    // Two Ekadashis a lunar month, so 24 to 26 a year, the same number
    // under each observer.
    for rule in [
        "EKADASHI_VAISHNAVA",
        "EKADASHI_SMARTA",
        "EKADASHI_SMARTA_RENUNCIANT",
    ] {
        let fasts = found
            .value
            .ekadashis
            .iter()
            .filter(|fast| fast.rule == rule)
            .count();
        assert!((24..=26).contains(&fasts), "{rule}: {fasts}");
    }
    let widened = found
        .provenance
        .applied_conventions
        .iter()
        .find(|convention| convention.knob == "festival.days")
        .unwrap();
    assert_eq!(widened.value, "GREGORIAN 2025-12-30..GREGORIAN 2027-01-02");
}

#[test]
fn a_rule_is_refused_by_its_place_in_the_request() {
    let (from, to) = year(2026);
    let mut rules = FestivalRule::dharmasindhu();
    rules[2].decide.push(Guard::new(
        [Predicate::Lasts {
            day: Which::Later,
            from: Edge::Sunset,
            ghatis: 0,
        }],
        teistro::festival::Choice::Later,
    ));
    let error = context()
        .almanac()
        .festivals(
            &from,
            &to,
            &delhi(),
            UtcOffset::literal(5, 30, 0),
            &FestivalRequest::new(rules),
        )
        .unwrap_err();
    assert_eq!(error.field(), Some("rules[2].decide.when.ghatis"));
}

#[test]
fn the_answer_alone_and_beside_its_days_agree_and_the_days_are_the_almanacs() {
    let sdk = context();
    let (from, to) = (
        CalendarDate::defined(Calendar::Gregorian, 2026, 10, 15),
        CalendarDate::defined(Calendar::Gregorian, 2026, 11, 10),
    );
    let clock = UtcOffset::literal(5, 30, 0);
    let asked = FestivalRequest::from(FestivalPack::Dharmasindhu);
    let alone = sdk
        .almanac()
        .festivals(&from, &to, &delhi(), clock, &asked)
        .unwrap();
    let beside = sdk
        .almanac()
        .festivals_with_days(&from, &to, &delhi(), clock, &asked)
        .unwrap();
    assert_eq!(beside.answer, alone);
    let rules: Vec<&str> = alone
        .value
        .observances
        .iter()
        .map(|o| o.rule.as_str())
        .collect();
    assert_eq!(rules, ["VIJAYA_DASHAMI", "LAKSHMI_PUJA"]);
    let (days, each) = sdk.almanac().of_each(&from, &to, &delhi(), clock).unwrap();
    assert_eq!(beside.days, days);
    assert_eq!(beside.day_hashes, each);

    // The input hash covers the rules: another rule is another input.
    let mut earlier = FestivalRule::dharmasindhu().remove(3);
    earlier.decide.clear();
    let other = sdk
        .almanac()
        .festivals(&from, &to, &delhi(), clock, &asked.with_rule(earlier))
        .unwrap();
    assert_ne!(other.provenance.input_hash, alone.provenance.input_hash);
}

#[test]
fn muhurta_and_festivals_asked_together_answer_as_each_alone_over_one_run_of_days() {
    let sdk = context();
    let (from, to) = (
        CalendarDate::defined(Calendar::Gregorian, 2026, 10, 15),
        CalendarDate::defined(Calendar::Gregorian, 2026, 11, 10),
    );
    let clock = UtcOffset::literal(5, 30, 0);
    let muhurta = MuhurtaRequest::new(ActivityRules::raman_marriage()).with_windows_on(2);
    let festivals = FestivalRequest::from(FestivalPack::Dharmasindhu);
    let asked = AlmanacRequest::new()
        .with_muhurta(muhurta.clone())
        .with_festivals(festivals.clone());
    let both = sdk
        .almanac()
        .asked(&from, &to, &delhi(), clock, &asked)
        .unwrap();
    let (days, each) = sdk.almanac().of_each(&from, &to, &delhi(), clock).unwrap();
    assert_eq!(both.days, days);
    assert_eq!(both.day_hashes, each);
    assert_eq!(
        both.muhurta,
        Some(
            sdk.almanac()
                .muhurta(&from, &to, &delhi(), clock, &muhurta)
                .unwrap()
        )
    );
    assert_eq!(
        both.festivals,
        Some(
            sdk.almanac()
                .festivals(&from, &to, &delhi(), clock, &festivals)
                .unwrap()
        )
    );

    // Asked for neither, it is the days alone.
    let neither = sdk
        .almanac()
        .asked(&from, &to, &delhi(), clock, &AlmanacRequest::new())
        .unwrap();
    assert_eq!(
        (neither.days, neither.muhurta, neither.festivals),
        (days, None, None)
    );
}

#[test]
fn a_kshaya_eleventh_pierced_at_sunrise_splits_the_three_observers() {
    // 2023's bright 11th of Bhadrapada at Delhi began after the 25th's
    // sunrise and ended before the 26th's: pierced at sunrise, holding no
    // sunrise of its own. The published almanac gives the Smarta fast the
    // 25th and the renunciant's and the Vaishnava's the 26th, as the text's
    // S8 does.
    let (from, to) = (
        CalendarDate::defined(Calendar::Gregorian, 2023, 9, 20),
        CalendarDate::defined(Calendar::Gregorian, 2023, 9, 30),
    );
    let found = context()
        .almanac()
        .festivals(
            &from,
            &to,
            &delhi(),
            UtcOffset::literal(5, 30, 0),
            &FestivalRequest::from(FestivalPack::Dharmasindhu),
        )
        .unwrap()
        .value;
    let days: Vec<(&str, u8)> = found
        .ekadashis
        .iter()
        .map(|fast| (fast.rule.as_str(), fast.day.day))
        .collect();
    assert_eq!(
        days,
        [
            ("EKADASHI_VAISHNAVA", 26),
            ("EKADASHI_SMARTA", 25),
            ("EKADASHI_SMARTA_RENUNCIANT", 26),
        ]
    );
    let fast = &found.ekadashis[0];
    assert_eq!(fast.pierced_at, Some(teistro::festival::Vedha::Sunrise));
    assert_eq!(fast.days.each_ref().map(|day| day.day), [25, 26]);
}
