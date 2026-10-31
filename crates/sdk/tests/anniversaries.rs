//! Anniversaries over a span of years (`03-design/anniversary-span.md`):
//! the one-call answer read back against the festival reckoning a year at
//! a time, the rule named on each day, the event kind, and every refusal.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use std::collections::BTreeSet;

use serde_json::{Value, json};
use teistro::catalogue::{Calendar, Masa, Tithi};
use teistro::festival::{
    Anniversary, AnniversaryDay, AnniversaryKind, DayClause, FirstYearAdhika, Half, InSamsarpa,
    JanmaOptions, Origin, TithiShape, Window,
};
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{AnniversariesRequest, CalendarDate, Context, FestivalRequest, Status, UtcOffset};

const NPT: UtcOffset = UtcOffset::literal(5, 45, 0);

fn kathmandu() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    )
}

fn sdk() -> Context {
    Context::builder().nepal_official([]).build().unwrap()
}

fn date(year: i32, month: u8, day: u8) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, year, month, day)
}

fn lunar(key: &str, month: Masa, tithi: Tithi, kind: AnniversaryKind) -> Anniversary {
    Anniversary {
        key: key.to_owned(),
        origin: Origin::Lunar {
            month,
            tithi,
            adhika: false,
            half: Half::First,
            at: None,
        },
        kind,
        window: None,
        first_year_adhika: FirstYearAdhika::Adhika,
        in_samsarpa: InSamsarpa::Also,
        janma: JanmaOptions::default(),
    }
}

fn request(from_year: i32, years: u16, anniversaries: Vec<Anniversary>) -> AnniversariesRequest {
    AnniversariesRequest {
        from_year,
        years,
        place: kathmandu(),
        offset: NPT,
        calendar: Calendar::Gregorian,
        anniversaries,
    }
}

/// Every month's every tithi, as a family's list might fall: thirty people
/// on thirty tithis of the year's twelve months.
fn family() -> Vec<Anniversary> {
    (0..30)
        .map(|index| {
            let (month, tithi) = (Masa::ALL[index % 12], Tithi::ALL[index]);
            let kind = match index % 3 {
                0 => AnniversaryKind::Shraddha,
                1 => AnniversaryKind::JanmaTithi,
                _ => AnniversaryKind::Event,
            };
            lunar(&format!("PERSON_{index}"), month, tithi, kind)
        })
        .collect()
}

fn sorted(days: Vec<AnniversaryDay>) -> Vec<(String, String, Value)> {
    let mut days: Vec<_> = days
        .into_iter()
        .map(|day| {
            (
                day.key.clone(),
                day.observance.day.to_string(),
                serde_json::to_value(&day).unwrap(),
            )
        })
        .collect();
    days.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
    days
}

/// The acceptance test: one call over six years answers what six festival
/// requests answer, value by value, so the walk it shares changes nothing.
#[test]
fn a_span_answers_what_each_year_answers_alone() {
    let sdk = sdk();
    let asked = family();
    let festival = asked.iter().cloned().fold(
        FestivalRequest::new(Vec::new()),
        FestivalRequest::with_anniversary,
    );
    let years = 2026..=2031;
    let each: Vec<AnniversaryDay> = years
        .clone()
        .flat_map(|year| {
            sdk.almanac()
                .festivals(
                    &date(year, 1, 1),
                    &date(year, 12, 31),
                    &kathmandu(),
                    NPT,
                    &festival,
                )
                .unwrap()
                .value
                .anniversaries
        })
        .collect();
    let span = sdk
        .almanac()
        .anniversaries(&request(2026, 6, asked))
        .unwrap();
    assert!(each.len() > 150, "{} days", each.len());
    assert_eq!(sorted(span.value.anniversaries), sorted(each));
    assert_eq!(span.value.unjudged, []);
}

/// A span past one range is founded a range at a time. The occurrence on
/// a seam between two is one occurrence, on a provider other than the
/// committee's too: no person's day is answered twice.
#[test]
fn a_span_across_its_seams_answers_each_day_once() {
    let sdk = Context::builder()
        .ephemeris([teistro::Ephemeris::Builtin])
        .build()
        .unwrap();
    let days = sdk
        .almanac()
        .anniversaries(&request(2026, 3, family()))
        .unwrap()
        .value
        .anniversaries;
    let mut seen = BTreeSet::new();
    for day in &days {
        assert!(
            seen.insert((day.key.clone(), day.observance.day.to_string())),
            "{} on {}",
            day.key,
            day.observance.day
        );
    }
    assert!(days.len() > 70, "{}", days.len());
}

/// Every day says how its tithi stood, read back from the almanac's own
/// sunrises, and the clause the day's case gives; each shape and clause a
/// decade at Kathmandu holds is reached.
#[test]
fn each_day_names_the_rule_applied_to_its_tithi() {
    let sdk = sdk();
    // Every tithi of every month over twelve years.
    let asked: Vec<Anniversary> = Tithi::ALL
        .iter()
        .flat_map(|&tithi| {
            Masa::ALL.iter().flat_map(move |&month| {
                [AnniversaryKind::Ekoddishta, AnniversaryKind::JanmaTithi]
                    .into_iter()
                    .map(move |kind| {
                        lunar(&format!("{month:?}_{tithi:?}_{kind:?}"), month, tithi, kind)
                    })
            })
        })
        .collect();
    let days = sdk
        .almanac()
        .anniversaries(&request(2026, 12, asked))
        .unwrap()
        .value
        .anniversaries;
    let shapes: BTreeSet<String> = days
        .iter()
        .map(|d| format!("{:?}", d.day_rule.tithi))
        .collect();
    assert_eq!(shapes.len(), 3, "{shapes:?}");
    for day in &days {
        let rule = &day.day_rule;
        assert_eq!(
            rule.at,
            match day.kind {
                AnniversaryKind::JanmaTithi => Window::Sunrise,
                _ => Window::DayMuhurta { from: 8, to: 9 },
            }
        );
        assert_ne!(rule.source, "");
        if day.kind == AnniversaryKind::JanmaTithi {
            // At sunrise a tithi's shape is the case itself.
            match rule.tithi {
                TithiShape::Kshaya => assert_eq!(rule.clause, DayClause::Neither),
                TithiShape::Vriddhi => assert_eq!(rule.clause, DayClause::BothWhole),
                _ => assert_eq!(rule.clause, DayClause::OneDay),
            }
        }
    }
}

/// An event keeps the window it states, sunrise by default, and names the
/// convention, in the festival reckoning and the span alike.
#[test]
fn an_event_keeps_its_stated_window_and_names_its_convention() {
    let sdk = sdk();
    let wedding = lunar(
        "WEDDING",
        Masa::Margashirsha,
        Tithi::ShuklaPanchami,
        AnniversaryKind::Event,
    );
    let at_sunrise = sdk
        .almanac()
        .anniversaries(&request(2026, 3, vec![wedding.clone()]))
        .unwrap();
    assert_eq!(at_sunrise.value.anniversaries.len(), 3);
    assert!(
        at_sunrise
            .value
            .anniversaries
            .iter()
            .all(|day| day.kind == AnniversaryKind::Event && day.day_rule.at == Window::Sunrise)
    );
    assert!(
        at_sunrise
            .provenance
            .applied_conventions
            .iter()
            .any(|c| c.knob == "anniversary.event")
    );
    let festival = FestivalRequest::new(Vec::new()).with_anniversary(wedding.clone());
    let reckoned = sdk
        .almanac()
        .festivals(
            &date(2026, 1, 1),
            &date(2026, 12, 31),
            &kathmandu(),
            NPT,
            &festival,
        )
        .unwrap();
    assert!(
        reckoned
            .provenance
            .applied_conventions
            .iter()
            .any(|c| c.knob == "anniversary.event")
    );

    let at_noon = Anniversary {
        window: Some(Window::DayMuhurta { from: 8, to: 9 }),
        ..wedding
    };
    let days = sdk
        .almanac()
        .anniversaries(&request(2026, 3, vec![at_noon]))
        .unwrap()
        .value
        .anniversaries;
    assert!(
        days.iter()
            .all(|day| day.day_rule.at == Window::DayMuhurta { from: 8, to: 9 })
    );
}

/// A shraddha's adhika policies do not reach an event: the first year's
/// month, doubled in 2026, stays the ordinary one.
#[test]
fn an_event_keeps_the_ordinary_month() {
    let sdk = sdk();
    let origin = sdk
        .almanac()
        .anniversaries(&request(
            2025,
            1,
            vec![lunar(
                "ANY",
                Masa::Jyeshtha,
                Tithi::ShuklaPanchami,
                AnniversaryKind::Event,
            )],
        ))
        .unwrap();
    assert_eq!(origin.value.anniversaries.len(), 1);
    let event = Anniversary {
        origin: Origin::Lunar {
            month: Masa::Jyeshtha,
            tithi: Tithi::ShuklaPanchami,
            adhika: false,
            half: Half::First,
            at: Some(origin.value.anniversaries[0].observance.tithi.from),
        },
        ..lunar(
            "EVENT",
            Masa::Jyeshtha,
            Tithi::ShuklaPanchami,
            AnniversaryKind::Event,
        )
    };
    let shraddha = Anniversary {
        key: "SHRADDHA".to_owned(),
        kind: AnniversaryKind::Ekoddishta,
        ..event.clone()
    };
    let days = sdk
        .almanac()
        .anniversaries(&request(2026, 1, vec![event, shraddha]))
        .unwrap()
        .value
        .anniversaries;
    let on = |key: &str| days.iter().find(|day| day.key == key).unwrap();
    assert!(!on("EVENT").observance.adhika);
    assert!(on("SHRADDHA").observance.adhika);
}

/// The span's years are the calendar's own, and a year's edges are month 1,
/// day 1 and the day before the next.
#[test]
fn the_span_runs_in_the_calendars_own_years() {
    let sdk = sdk();
    let mut asked = request(
        2083,
        2,
        vec![lunar(
            "BARKHI",
            Masa::Kartika,
            Tithi::ShuklaDashami,
            AnniversaryKind::Ekoddishta,
        )],
    );
    asked.calendar = Calendar::BikramSambat;
    let answer = sdk.almanac().anniversaries(&asked).unwrap();
    let days = &answer.value.anniversaries;
    assert_eq!(days.len(), 2);
    assert!(
        days.iter()
            .all(|day| day.observance.day.calendar == Calendar::BikramSambat)
    );
    assert_eq!(
        days.iter()
            .map(|day| day.observance.day.year)
            .collect::<Vec<_>>(),
        [2083, 2084]
    );
    let knob = answer
        .provenance
        .applied_conventions
        .iter()
        .find(|c| c.knob == "anniversary.span")
        .unwrap();
    assert!(knob.value.contains(".."), "{}", knob.value);
}

/// Two people on one date of different hours are still two origins, and an
/// origin the almanac refuses is named by its place in the list.
#[test]
fn an_instant_and_a_date_are_found_and_a_refusal_names_the_entry() {
    let sdk = sdk();
    let birth = Anniversary {
        origin: serde_json::from_value(json!({"by": "INSTANT", "at": 2_451_545.0})).unwrap(),
        ..lunar(
            "BIRTH",
            Masa::Chaitra,
            Tithi::Purnima,
            AnniversaryKind::JanmaTithi,
        )
    };
    let date_origin = Anniversary {
        origin: serde_json::from_value(json!({"by": "DATE", "date": {
            "calendar": "GREGORIAN", "year": 2001, "month": 2, "day": 3}}))
        .unwrap(),
        ..lunar(
            "DEATH",
            Masa::Chaitra,
            Tithi::Purnima,
            AnniversaryKind::Shraddha,
        )
    };
    let days = sdk
        .almanac()
        .anniversaries(&request(2026, 2, vec![birth.clone(), date_origin]))
        .unwrap()
        .value
        .anniversaries;
    assert!(
        days.iter()
            .any(|d| d.key == "BIRTH" && d.count.is_some_and(|age| (26..=28).contains(&age)))
    );
    assert!(
        days.iter()
            .any(|d| d.key == "DEATH" && d.origin.assumed_sunrise)
    );
    let bad = Anniversary {
        origin: serde_json::from_value(json!({"by": "DATE", "date": {
            "calendar": "GREGORIAN", "year": 2001, "month": 2, "day": 30}}))
        .unwrap(),
        ..birth
    };
    let refused = sdk
        .almanac()
        .anniversaries(&request(2026, 1, vec![bad]))
        .unwrap_err();
    assert_eq!(refused.field(), Some("anniversaries[0].origin.day"));
}

fn asking(extra: &Value) -> Value {
    let mut base = json!({
        "fromYear": 2026, "years": 10, "latitudeDeg": 27.7172, "longitudeDeg": 85.324,
        "utcOffsetSeconds": 20700, "anniversaries": [{
            "key": "A", "origin": {"by": "LUNAR", "month": "KARTIKA", "tithi": "SHUKLA_DASHAMI"}}]
    });
    for (key, value) in extra.as_object().unwrap() {
        base[key] = value.clone();
    }
    base
}

/// A request built by hand meets the limits one read from text meets.
#[test]
fn a_request_built_by_hand_is_held_to_the_limits() {
    let sdk = sdk();
    let one = family().swap_remove(0);
    let too_many: Vec<Anniversary> = (0..=teistro::MOST_ANNIVERSARIES)
        .map(|at| Anniversary {
            key: format!("K{at}"),
            ..one.clone()
        })
        .collect();
    for (asked, status, field) in [
        (
            request(2026, 0, vec![one.clone()]),
            Status::OutOfRange,
            "anniversaries.years",
        ),
        (
            request(2026, 1, too_many),
            Status::Limit,
            "anniversaries.anniversaries",
        ),
    ] {
        let refused = sdk.almanac().anniversaries(&asked).unwrap_err();
        assert_eq!((refused.status, refused.field()), (status, Some(field)));
    }
}

/// `anniversary-span.md` §2.1 builds nothing for exactly equal shares
/// because a decade of every tithi of every month holds none, while the
/// tithi's two shapes it does report are both there.
#[test]
fn a_decade_of_shraddhas_holds_no_equal_shares() {
    let asked: Vec<Anniversary> = Masa::ALL
        .iter()
        .flat_map(|&month| Tithi::ALL.iter().map(move |&tithi| (month, tithi)))
        .flat_map(|(month, tithi)| {
            [AnniversaryKind::Ekoddishta, AnniversaryKind::Parvana]
                .map(|kind| lunar(&format!("{month:?}_{tithi:?}_{kind:?}"), month, tithi, kind))
        })
        .collect();
    let days = sdk()
        .almanac()
        .anniversaries(&request(2026, 10, asked))
        .unwrap()
        .value
        .anniversaries;
    let count = |shape: TithiShape| {
        days.iter()
            .filter(|day| day.day_rule.tithi == shape)
            .count()
    };
    assert!(count(TithiShape::Kshaya) > 0 && count(TithiShape::Vriddhi) > 0);
    assert!(
        days.iter()
            .all(|day| day.day_rule.clause != DayClause::EqualParts),
        "§2.1 says the scheme decides nothing here"
    );
}

#[test]
fn a_request_is_read_or_refused_by_name() {
    let read = |extra: Value| AnniversariesRequest::from_json(&asking(&extra).to_string());
    let asked = read(json!({"calendar": "BIKRAM_SAMBAT", "altitudeM": 1400.0})).unwrap();
    assert_eq!(asked.calendar, Calendar::BikramSambat);
    let origin = json!({"by": "LUNAR", "month": "KARTIKA", "tithi": "PURNIMA"});
    let many: Vec<Value> = (0..=teistro::MOST_ANNIVERSARIES)
        .map(|at| json!({"key": format!("K{at}"), "origin": origin}))
        .collect();
    let twice = json!([{"key": "A", "origin": origin}, {"key": "A", "origin": origin}]);
    let empty = json!([{"key": " ", "origin": origin}]);
    for (extra, status, field) in [
        (
            json!({"years": 0}),
            Status::OutOfRange,
            "anniversaries.years",
        ),
        (
            json!({"years": 101}),
            Status::OutOfRange,
            "anniversaries.years",
        ),
        (
            json!({"anniversaries": many}),
            Status::Limit,
            "anniversaries.anniversaries",
        ),
        (
            json!({"anniversaries": twice}),
            Status::InvalidArg,
            "anniversaries.anniversaries[1].key",
        ),
        (
            json!({"anniversaries": empty}),
            Status::InvalidArg,
            "anniversaries.anniversaries[0].key",
        ),
        (
            json!({"latitudeDeg": 91.0}),
            Status::InvalidArg,
            "anniversaries.latitudeDeg",
        ),
        (
            json!({"utcOffsetSeconds": 90_000}),
            Status::InvalidArg,
            "anniversaries.utcOffsetSeconds",
        ),
    ] {
        let refused = read(extra.clone()).unwrap_err();
        assert_eq!(
            (refused.status, refused.field()),
            (status, Some(field)),
            "{extra}"
        );
    }
    assert_eq!(
        read(json!({"nonsense": 1})).unwrap_err().status,
        Status::InvalidArg
    );
}

/// A polar winter gives occurrences no day can judge; only those inside
/// the span are told, and the span's in-full answer is sealed and keeps
/// its catalogue keys in full.
#[test]
fn an_unjudged_occurrence_is_told_and_the_answer_is_written_in_full() {
    let sdk = sdk();
    let mut north = request(2026, 1, family());
    north.place = Place::new(
        Latitude::literal(78.2),
        Longitude::literal(15.6),
        Altitude::literal(0.0),
    );
    north.offset = UtcOffset::literal(1, 0, 0);
    let found = sdk.almanac().anniversaries(&north).unwrap().value;
    assert!(found.unjudged.len() > 3, "{}", found.unjudged.len());
    // The year, and past it by the polar night its last day waits through.
    assert!(
        found
            .unjudged
            .iter()
            .all(|u| u.tithi.to.get() > 2_461_041.0 && u.tithi.from.get() < 2_461_500.0)
    );
    let full = sdk
        .almanac()
        .anniversaries_in_full(&request(
            2026,
            1,
            vec![lunar(
                "A",
                Masa::Kartika,
                Tithi::ShuklaDashami,
                AnniversaryKind::Shraddha,
            )],
        ))
        .unwrap();
    let day = &full.value["anniversaries"][0];
    assert_eq!(day["origin"]["tithi"], "tithi.SHUKLA_DASHAMI");
    assert_eq!(day["dayRule"]["tithi"], "ORDINARY");
    assert_eq!(day["dayRule"]["at"]["window"], "DAY_MUHURTA");
}

/// A request past `MOST_RULES` is refused before any reckoning, naming
/// `rules`.
#[test]
fn a_festival_request_is_held_to_its_size() {
    let family = (0..=FestivalRequest::MOST_RULES).map(|at| {
        lunar(
            &format!("K{at}"),
            Masa::Kartika,
            Tithi::Purnima,
            AnniversaryKind::Event,
        )
    });
    let refused = family
        .fold(
            FestivalRequest::new(Vec::new()),
            FestivalRequest::with_anniversary,
        )
        .check()
        .unwrap_err();
    assert_eq!(
        (refused.status, refused.field()),
        (Status::Limit, Some("rules"))
    );
}
