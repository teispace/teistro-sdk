//! A rule's or a kind's days year by year through the façade
//! (`03-design/rule-years.md`, SR-126): each year read back against the
//! festival reckoning over that year alone, a year the rule falls on twice
//! or on no day said so, the adhika-only fast in its adhika years alone,
//! every refusal by its field, and the cost of one call against a call a
//! year.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::catalogue::Calendar;
use teistro::festival::Observances;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{
    CalendarDate, Context, Ephemeris, FestivalRequest, NextKind, OccurrencesRequest, Status,
    UtcOffset,
};

fn committee() -> Context {
    Context::builder()
        .profile("nepali-committee")
        .ephemeris([Ephemeris::SuryaSiddhanta])
        .build()
        .unwrap()
}

fn kathmandu() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    )
}

/// Kathmandu's clock.
fn npt() -> UtcOffset {
    UtcOffset::literal(5, 45, 0)
}

fn asked(span: &str, target: &str) -> OccurrencesRequest {
    OccurrencesRequest::from_json(&format!(
        r#"{{{span}, "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1400,
            "utcOffsetSeconds": 20700, {target}}}"#
    ))
    .unwrap()
}

fn refused(text: &str) -> teistro::Error {
    OccurrencesRequest::from_json(text).unwrap_err()
}

/// A year's reckoning, cut to `keys` and to the spans of `group`, as one
/// festival call over that year alone answers it.
fn alone(
    sdk: &Context,
    (first, last): (&CalendarDate, &CalendarDate),
    request: &FestivalRequest,
    (keys, group): (&[String], Option<&str>),
) -> Observances {
    let mut found = sdk
        .almanac()
        .festivals(first, last, &kathmandu(), npt(), request)
        .unwrap()
        .value;
    found.keep(|key| keys.iter().any(|k| k == key));
    found.unplaced.retain(|u| keys.contains(&u.rule));
    found
        .groups
        .retain(|span| Some(span.group.as_str()) == group);
    found
}

/// The days and keys of a year, as `month-day:KEY+KEY`.
fn written(year: &teistro::OccurrenceYear) -> Vec<String> {
    year.days
        .iter()
        .map(|d| format!("{}-{}:{}", d.day.month, d.day.day, d.keys.join("+")))
        .collect()
}

#[test]
fn each_year_of_a_packs_rule_is_that_years_own_reckoning() {
    let sdk = committee();
    let span = asked(
        r#""fromYear": 2080, "years": 4, "calendar": "BIKRAM_SAMBAT""#,
        r#""rule": "VIJAYA_DASHAMI", "festivals": {"rules": "NEPAL_PANCHANGA"}"#,
    );
    let answer = sdk.almanac().occurrences(&span).unwrap().value;
    assert_eq!(answer.keys, ["VIJAYA_DASHAMI"]);
    assert_eq!(answer.years.len(), 4);
    let pack = FestivalRequest::from_json(r#"{"rules": "NEPAL_PANCHANGA"}"#).unwrap();
    for (k, year) in answer.years.iter().enumerate() {
        assert_eq!(year.year, 2080 + i32::try_from(k).unwrap());
        assert_eq!((year.first.month, year.first.day), (1, 1));
        // Once a year, in Asoj or Kartik.
        assert_eq!(year.days.len(), 1, "{}", year.year);
        assert!((6..=7).contains(&year.days[0].day.month), "{:?}", year.days);
        let own = alone(&sdk, (&year.first, &year.last), &pack, (&answer.keys, None));
        assert_eq!(year.festivals, own, "{}", year.year);
    }
}

#[test]
fn a_kinds_years_are_each_years_own_reckoning_every_rule_named() {
    let sdk = committee();
    let span = asked(r#""fromYear": 2025, "years": 2"#, r#""kind": "EKADASHI""#);
    let answer = sdk.almanac().occurrences(&span).unwrap().value;
    let read = FestivalRequest::of_kind(NextKind::Ekadashi.rules());
    let keys: Vec<String> = NextKind::Ekadashi
        .rules()
        .ekadashis
        .iter()
        .map(|r| r.key.clone())
        .collect();
    assert_eq!(answer.keys, keys);
    for year in &answer.years {
        let own = alone(&sdk, (&year.first, &year.last), &read, (&answer.keys, None));
        assert_eq!(year.festivals, own, "{}", year.year);
        // Twenty-four or so a year, the two traditions' days listed once
        // each, the keys on a day in the order they were looked for.
        assert!(
            (23..=27).contains(&year.days.len()),
            "{}",
            written(year).join(" ")
        );
        assert!(year.days.iter().all(|d| !d.keys.is_empty()));
        assert!(year.days.windows(2).all(|pair| {
            (pair[0].day.month, pair[0].day.day) < (pair[1].day.month, pair[1].day.day)
        }));
    }
}

/// A group's years list its members' days, and each year carries the
/// group's spans that reach it whole, as a range over the year answers
/// them (`03-design/festival-groups.md`, SR-125).
#[test]
fn each_year_of_a_group_is_that_years_own_reckoning_its_spans_whole() {
    let sdk = committee();
    let span = asked(
        r#""fromYear": 2025, "years": 2"#,
        r#""group": "TIHAR", "festivals": {"rules": "NEPAL_PANCHANGA"}"#,
    );
    let answer = sdk.almanac().occurrences(&span).unwrap().value;
    let pack = FestivalRequest::from_json(r#"{"rules": "NEPAL_PANCHANGA"}"#).unwrap();
    let members: Vec<String> = pack
        .groups()
        .iter()
        .find(|group| group.key == "TIHAR")
        .unwrap()
        .members
        .clone();
    assert_eq!(answer.keys, members);
    for year in &answer.years {
        let own = alone(
            &sdk,
            (&year.first, &year.last),
            &pack,
            (&answer.keys, Some("TIHAR")),
        );
        assert_eq!(year.festivals, own, "{}", year.year);
        // One Tihar a year, every member's day in it listed.
        assert_eq!(year.festivals.groups.len(), 1, "{}", year.year);
        let tihar = &year.festivals.groups[0];
        assert!((10..=11).contains(&tihar.first.month), "{tihar:?}");
        let listed: usize = year.days.iter().map(|d| d.keys.len()).sum();
        let placed = tihar.members.iter().filter(|m| m.day.is_some()).count();
        assert_eq!(listed, placed, "{}", written(year).join(" "));
    }
}

#[test]
fn a_lunar_rule_meets_a_solar_year_twice_and_the_next_year_not_at_all() {
    // Pausha's bright Ekadashi falls 10 January and 30 December 2025, so
    // 2026 holds none: each year lists what its dates hold (C2120).
    let sdk = committee();
    let answer = sdk
        .almanac()
        .occurrences(&asked(
            r#""fromYear": 2024, "years": 4"#,
            r#""rule": "PAUSHA_PUTRADA_EKADASHI", "festivals": {"rules": "NEPAL_PANCHANGA"}"#,
        ))
        .unwrap()
        .value;
    let by_year: Vec<(i32, Vec<String>)> = answer
        .years
        .iter()
        .map(|year| (year.year, written(year)))
        .collect();
    let day = |text: &str| String::from(text) + ":PAUSHA_PUTRADA_EKADASHI";
    assert_eq!(
        by_year,
        [
            (2024, vec![day("1-21")]),
            (2025, vec![day("1-10"), day("12-30")]),
            (2026, vec![]),
            (2027, vec![day("1-18")]),
        ]
    );
    // An empty year is no refusal and no unjudged occurrence: the rule's
    // month began in the year before and its next in the year after.
    assert_eq!(answer.years[2].festivals.observances.len(), 0);
    assert_eq!(answer.years[2].festivals.ekadashis.len(), 0);
    assert_eq!(answer.years[2].festivals.unjudged.len(), 0);
}

#[test]
fn a_fast_kept_in_the_adhika_month_alone_falls_in_its_adhika_years_alone() {
    // Padmini Ekadashi is the adhika month's bright 11th (VS 2083 p. 5): the
    // adhika Shrawan of VS 2080 and the adhika Jestha of VS 2083 hold it,
    // and no year between them does.
    let sdk = committee();
    let answer = sdk
        .almanac()
        .occurrences(&asked(
            r#""fromYear": 2078, "years": 7, "calendar": "BIKRAM_SAMBAT""#,
            r#""rule": "PADMINI_EKADASHI", "festivals": {"rules": "NEPAL_PANCHANGA"}"#,
        ))
        .unwrap()
        .value;
    let held: Vec<(i32, Vec<String>)> = answer
        .years
        .iter()
        .map(|year| (year.year, written(year)))
        .collect();
    let none = Vec::<String>::new;
    assert_eq!(
        held,
        [
            (2078, none()),
            (2079, none()),
            (2080, vec![String::from("4-13:PADMINI_EKADASHI")]),
            (2081, none()),
            (2082, none()),
            (2083, vec![String::from("2-13:PADMINI_EKADASHI")]),
            (2084, none()),
        ]
    );
    assert_eq!(answer.years[0].first.calendar, Calendar::BikramSambat);
}

/// A day a published set withdraws leaves its year with no day, the
/// withdrawn entry kept in the year's festivals to say why (C2122); a day
/// it adds is listed under its own key.
#[test]
fn a_withdrawn_day_empties_its_year_and_stays_in_its_festivals() {
    let sdk = committee();
    let with = |key: &str, body: &str| {
        asked(
            r#""fromYear": 2025, "years": 2"#,
            &format!(
                r#""rule": "{key}", "festivals": {{"rules": "NEPAL_PANCHANGA",
                    "published": [{{"format": "teistro.published/1", "publisher": "p", "id": "np",
                    "version": 1, "entries": [{{{body}, "source": "p. 1"}}]}}]}}"#
            ),
        )
    };
    let withdrawn = sdk
        .almanac()
        .occurrences(&with(
            "VIJAYA_DASHAMI",
            r#""action": "WITHDRAW", "rule": "VIJAYA_DASHAMI",
               "day": {"calendar": "GREGORIAN", "year": 2025, "month": 10, "day": 2}"#,
        ))
        .unwrap()
        .value;
    assert_eq!(withdrawn.years[0].days.len(), 0);
    assert_eq!(withdrawn.years[0].festivals.observances.len(), 1);
    assert!(
        withdrawn.years[0].festivals.observances[0]
            .published
            .is_some()
    );
    assert_eq!(withdrawn.years[1].days.len(), 1);
    let added = sdk
        .almanac()
        .occurrences(&with(
            "A_PRINTED_DAY",
            r#""action": "ADD", "key": "A_PRINTED_DAY",
               "day": {"calendar": "GREGORIAN", "year": 2026, "month": 2, "day": 10}"#,
        ))
        .unwrap()
        .value;
    assert_eq!(added.years[0].days.len(), 0);
    assert_eq!(written(&added.years[1]), ["2-10:A_PRINTED_DAY"]);
    assert_eq!(added.years[1].festivals.added.len(), 1);
}

#[test]
fn every_refusal_names_its_field() {
    let place = r#""latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700"#;
    for (body, field, status) in [
        (
            r#""fromYear": 2026, "years": 0, "kind": "PURNIMA""#,
            "occurrences.years",
            Status::OutOfRange,
        ),
        (
            r#""fromYear": 2026, "years": 51, "kind": "PURNIMA""#,
            "occurrences.years",
            Status::OutOfRange,
        ),
        (
            r#""fromYear": 2026, "years": 2, "kind": "PURNIMA", "rule": "HOLI""#,
            "occurrences.rule",
            Status::InvalidArg,
        ),
        (
            r#""fromYear": 2026, "years": 2, "rule": "HOLI""#,
            "occurrences.festivals",
            Status::InvalidArg,
        ),
        (
            r#""fromYear": 2026, "years": 2"#,
            "occurrences.kind",
            Status::InvalidArg,
        ),
        (
            r#""fromYear": 2026, "years": 2, "kind": "PURNIMA", "festivals": {"rules": "NEPAL"}"#,
            "occurrences.festivals",
            Status::InvalidArg,
        ),
        (
            r#""fromYear": 2026, "years": 2, "kind": "PURNIMA", "group": "TIHAR""#,
            "occurrences.group",
            Status::InvalidArg,
        ),
        (
            r#""fromYear": 2026, "years": 2, "group": "TIHAR""#,
            "occurrences.festivals",
            Status::InvalidArg,
        ),
    ] {
        let why = refused(&format!("{{{body}, {place}}}"));
        assert_eq!((why.field(), why.status), (Some(field), status), "{body}");
    }
    let past = refused(&format!(
        r#"{{"fromYear": 2026, "years": 51, "kind": "PURNIMA", {place}}}"#
    ));
    assert!(past.hint().is_some_and(|hint| hint.contains("50")));
    // A rule the festivals lack is refused when it is reckoned.
    let sdk = committee();
    let missing = sdk
        .almanac()
        .occurrences(&asked(
            r#""fromYear": 2026, "years": 1"#,
            r#""rule": "NO_SUCH_RULE", "festivals": {"rules": "NEPAL_PANCHANGA"}"#,
        ))
        .unwrap_err();
    assert_eq!(missing.field(), Some("rule"));
    let no_group = sdk
        .almanac()
        .occurrences(&asked(
            r#""fromYear": 2026, "years": 1"#,
            r#""group": "NO_SUCH_GROUP", "festivals": {"rules": "NEPAL_PANCHANGA"}"#,
        ))
        .unwrap_err();
    assert_eq!(no_group.field(), Some("group"));
}

#[test]
fn the_answer_is_sealed_over_the_request_and_says_what_was_founded() {
    let sdk = committee();
    let span = asked(r#""fromYear": 2026, "years": 1"#, r#""kind": "SANKRANTI""#);
    let answer = sdk.almanac().occurrences(&span).unwrap();
    let again = sdk.almanac().occurrences(&span).unwrap();
    assert_eq!(answer.provenance.input_hash, again.provenance.input_hash);
    let founded = answer
        .provenance
        .applied_conventions
        .iter()
        .find(|c| c.knob == "festival.years")
        .unwrap();
    assert!(founded.value.contains(".."));
    // Twelve sankrantis, each the first of a Bikram Sambat month.
    assert_eq!(answer.value.years[0].days.len(), 12);
    let full = sdk.almanac().occurrences_in_full(&span).unwrap().value;
    assert_eq!(full["years"][0]["first"]["calendar"], "calendar.GREGORIAN");
    assert_eq!(
        full["years"][0]["days"][0]["day"]["calendar"],
        "calendar.GREGORIAN"
    );
    assert!(full["years"][0]["festivals"]["dated"].is_array());
}

/// What one call costs against a call a year, for the design page's §5:
/// timed, not gated, since a test's clock is no gate.
/// `cargo test --release -p teistro --test occurrences price -- --ignored --nocapture`.
#[test]
#[ignore = "a measurement for the design page, in release"]
fn price() {
    const RUNS: usize = 21;
    type Sky = (&'static str, fn() -> Context);
    let skies: [Sky; 2] = [
        ("committee, Surya Siddhanta", committee),
        ("drik, built-in", || {
            Context::builder()
                .ephemeris([Ephemeris::Builtin])
                .build()
                .unwrap()
        }),
    ];
    let median = |mut runs: Vec<f64>| {
        runs.sort_by(f64::total_cmp);
        runs[runs.len() / 2] * 1000.0
    };
    let pack = FestivalRequest::from_json(r#"{"rules": "NEPAL_PANCHANGA"}"#).unwrap();
    let narrowed = pack.narrowed_to("VIJAYA_DASHAMI").unwrap();
    let kind = FestivalRequest::of_kind(NextKind::Ekadashi.rules());
    let years = |sdk: &Context| -> Vec<(CalendarDate, CalendarDate)> {
        let span = asked(
            r#""fromYear": 2080, "years": 8, "calendar": "BIKRAM_SAMBAT""#,
            r#""kind": "SANKRANTI""#,
        );
        let answer = sdk.almanac().occurrences(&span).unwrap().value;
        answer
            .years
            .iter()
            .map(|y| (y.first.clone(), y.last.clone()))
            .collect()
    };
    for (name, build) in skies {
        let bounds = years(&build());
        let time = |work: &dyn Fn(&Context)| {
            median(
                (0..RUNS)
                    .map(|_| {
                        let sdk = build();
                        let start = std::time::Instant::now();
                        work(&sdk);
                        start.elapsed().as_secs_f64()
                    })
                    .collect(),
            )
        };
        let one = |target: &'static str| {
            move |sdk: &Context| {
                sdk.almanac()
                    .occurrences(&asked(
                        r#""fromYear": 2080, "years": 8, "calendar": "BIKRAM_SAMBAT""#,
                        target,
                    ))
                    .unwrap();
            }
        };
        let eight = |request: &FestivalRequest| {
            let bounds = bounds.clone();
            let request = request.clone();
            move |sdk: &Context| {
                for (first, last) in &bounds {
                    sdk.almanac()
                        .festivals(first, last, &kathmandu(), npt(), &request)
                        .unwrap();
                }
            }
        };
        let rows: [(&str, f64); 5] = [
            (
                "Vijaya Dashami, 8 years, one call",
                time(&one(
                    r#""rule": "VIJAYA_DASHAMI", "festivals": {"rules": "NEPAL_PANCHANGA"}"#,
                )),
            ),
            ("Vijaya Dashami, 8 narrowed calls", time(&eight(&narrowed))),
            (
                "EKADASHI, 8 years, one call",
                time(&one(r#""kind": "EKADASHI""#)),
            ),
            ("EKADASHI, 8 calls", time(&eight(&kind))),
            ("the whole pack, 8 calls (for scale)", time(&eight(&pack))),
        ];
        for (row, ms) in rows {
            println!("{name} | {row} | {ms:.1} ms");
        }
    }
}
