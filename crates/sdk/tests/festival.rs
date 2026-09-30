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
use teistro::festival::{FestivalRule, Guard, Predicate, Which};
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{CalendarDate, Context, Ephemeris, UtcOffset};

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
            &FestivalRule::dharmasindhu(),
        )
        .unwrap();
    assert!(
        found.value.unjudged.is_empty(),
        "{:?}",
        found.value.unjudged
    );
    // Rama Navami in March or April, Janmashtami in August or September,
    // Vijaya Dashami in September or October, Lakshmi puja in October or
    // November: the lunar months' Gregorian reach.
    for (rule, months) in [
        ("RAMA_NAVAMI", 3..=4),
        ("JANMASHTAMI", 8..=9),
        ("VIJAYA_DASHAMI", 9..=10),
        ("LAKSHMI_PUJA", 10..=11),
    ] {
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
    let widened = found
        .provenance
        .applied_conventions
        .iter()
        .find(|convention| convention.knob == "festival.days")
        .unwrap();
    assert_eq!(widened.value, "GREGORIAN 2025-12-31..GREGORIAN 2027-01-02");
}

#[test]
fn a_rule_is_refused_by_its_place_in_the_request() {
    let (from, to) = year(2026);
    let mut rules = FestivalRule::dharmasindhu();
    rules[2].decide.push(Guard::new(
        [Predicate::Lasts {
            day: Which::Later,
            muhurtas: 0,
        }],
        teistro::festival::Choice::Later,
    ));
    let error = context()
        .almanac()
        .festivals(&from, &to, &delhi(), UtcOffset::literal(5, 30, 0), &rules)
        .unwrap_err();
    assert_eq!(error.field(), Some("rules[2].decide.when.muhurtas"));
}
