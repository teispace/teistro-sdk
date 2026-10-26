//! The muhurta search through the façade (`03-design/muhurta-at-the-boundary.md`
//! §3): the answer alone and beside its days agree, the provenance names
//! what the search applied and hashes what it was asked, and the asta
//! criterion a request names is the one the season closes days by.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::catalogue::Calendar;
use teistro::muhurta::season::BlackoutKind;
use teistro::muhurta::{ActivityRules, Answer};
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{CalendarDate, Context, Criterion, Ephemeris, MuhurtaRequest, UtcOffset};

fn context() -> Context {
    Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
}

fn place() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    )
}

fn clock() -> UtcOffset {
    UtcOffset::literal(5, 45, 0)
}

fn date(month: u8, day: u8) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, 2026, month, day)
}

#[test]
fn the_answer_alone_and_beside_its_days_agree_and_say_what_they_applied() {
    let sdk = context();
    let (from, to) = (date(11, 25), date(12, 5));
    let asked = MuhurtaRequest::new(ActivityRules::raman_marriage()).with_windows_on(2);
    let alone = sdk
        .almanac()
        .muhurta(&from, &to, &place(), clock(), &asked)
        .unwrap();
    let beside = sdk
        .almanac()
        .muhurta_with_days(&from, &to, &place(), clock(), &asked)
        .unwrap();
    assert_eq!(beside.answer, alone);
    assert!(
        !alone.value.windows.is_empty(),
        "a fortnight after Devuthani holds windows"
    );

    // The days are the almanac's own, sealed as `of_each` seals them.
    let (days, each) = sdk
        .almanac()
        .of_each(&from, &to, &place(), clock())
        .unwrap();
    assert_eq!(beside.days, days);
    assert_eq!(beside.day_hashes, each);

    // What the search applied beside the settings, by name.
    let applied: Vec<(&str, &str)> = alone
        .provenance
        .applied_conventions
        .iter()
        .map(|c| (c.knob.as_str(), c.value.as_str()))
        .collect();
    let asta = teistro::canonical_json(&Criterion::SURYA_SIDDHANTA);
    assert!(
        applied.contains(&("muhurta.asta", asta.as_str())),
        "{applied:?}"
    );
    let zodiac_at: f64 = applied
        .iter()
        .find(|(knob, _)| *knob == "muhurta.zodiacAt")
        .unwrap()
        .1
        .parse()
        .unwrap();
    // The middle of eleven civil days on a +5:45 clock: 2026-11-30 12:00
    // local, 06:15 UTC.
    assert!(
        (zodiac_at - 2_461_374.760_416_7).abs() < 1e-6,
        "{zodiac_at}"
    );

    // The input hash covers the request: another criterion is another input.
    let other = sdk
        .almanac()
        .muhurta(
            &from,
            &to,
            &place(),
            clock(),
            &asked.clone().with_asta(Criterion::PTOLEMY),
        )
        .unwrap();
    assert_ne!(other.provenance.input_hash, alone.provenance.input_hash);
}

/// The dates a search's season closed by Shukra's asta, as (month, day):
/// the calendar's own dates carry the era, which a date written without
/// one does not.
fn closed_by_shukra(answer: &Answer) -> Vec<(u8, u8)> {
    answer
        .closed
        .iter()
        .filter(|day| day.by.contains(&BlackoutKind::ShukraAsta))
        .map(|day| (day.date.month, day.date.day))
        .collect()
}

#[test]
fn the_asta_a_request_names_is_the_one_the_season_closes_by() {
    let sdk = context();
    // Rules that heed Shukra's asta alone, over the days its BS 2083
    // window opens in: the Surya Siddhanta's opens on 2026-10-10 and the
    // combustion orbs' on 10-18 (`muhurta-measured.md` §2), each on the
    // evening the Venus was last seen, so the day after is the first closed
    // whole.
    let mut rules = ActivityRules::raman_marriage();
    rules.heeds = vec![BlackoutKind::ShukraAsta];
    let asked = MuhurtaRequest::new(rules).with_windows_on(1).at_most(1);
    let (from, to) = (date(10, 10), date(10, 20));
    let under = |criterion| {
        let answer = sdk
            .almanac()
            .muhurta(
                &from,
                &to,
                &place(),
                clock(),
                &asked.clone().with_asta(criterion),
            )
            .unwrap();
        closed_by_shukra(&answer.value)
    };
    let (texts, orbs) = (
        under(Criterion::SURYA_SIDDHANTA),
        under(Criterion::COMBUSTION_ORB),
    );
    assert_eq!(texts, (11..=20).map(|day| (10, day)).collect::<Vec<_>>());
    assert!(!orbs.is_empty(), "`orbs` is empty");
    assert!(
        orbs.iter().all(|&(_, day)| day >= 18),
        "the orbs open it on the 18th: {orbs:?}"
    );
}

/// Every window a search offers lies in the part of its day the rules
/// keep: the tonsure's forenoon, the ear boring's daylight, and the whole
/// day when the same rules keep any hour.
#[test]
fn a_rite_held_in_part_of_the_day_is_offered_only_that_part() {
    use teistro::muhurta::Hours;

    let sdk = context();
    // Past Venus's combustion, which shuts the tonsure until late February.
    let (from, to) = (date(2, 25), date(3, 10));
    let windows = |rules: ActivityRules| {
        let asked = MuhurtaRequest::new(rules)
            .with_windows_on(14)
            .at_most(100_000);
        let found = sdk
            .almanac()
            .muhurta_with_days(&from, &to, &place(), clock(), &asked)
            .unwrap();
        let days = found.days.value;
        found
            .answer
            .value
            .windows
            .iter()
            .map(|w| {
                let day = days
                    .iter()
                    .find(|d| d.window.contains(w.at.from))
                    .unwrap()
                    .day
                    .clone();
                (w.at, day.sunrise.get(), day.sunset.get())
            })
            .collect::<Vec<_>>()
    };
    let tonsure = windows(ActivityRules::raman_chudakarana());
    assert!(
        !tonsure.is_empty(),
        "a fortnight past Venus's combustion holds a forenoon"
    );
    for (at, sunrise, sunset) in &tonsure {
        let noon = f64::midpoint(*sunrise, *sunset);
        assert!(at.from.get() >= *sunrise && at.to.get() <= noon, "{at:?}");
    }
    let boring = windows(ActivityRules::raman_karnavedha());
    assert!(!boring.is_empty(), "the fortnight holds a daylight window");
    for (at, sunrise, sunset) in &boring {
        assert!(
            at.from.get() >= *sunrise && at.to.get() <= *sunset,
            "{at:?}"
        );
    }
    let mut any_hour = ActivityRules::raman_karnavedha();
    any_hour.hours = Hours::Any;
    let all = windows(any_hour);
    assert!(
        all.iter().any(|(at, _, sunset)| at.from.get() >= *sunset),
        "the same rules at any hour reach the night"
    );
}

/// A journey towards the day's disha shool opens at its release, counted
/// in the day's own ghatis; towards any other direction, or none, the day
/// is open from sunrise.
#[test]
fn a_journey_towards_the_days_shool_waits_for_its_release() {
    use teistro::catalogue::{Direction, Vara};
    use teistro::muhurta::Hours;

    let sdk = context();
    let (from, to) = (date(1, 4), date(1, 24));
    let starts = |asked: MuhurtaRequest| {
        let found = sdk
            .almanac()
            .muhurta_with_days(
                &from,
                &to,
                &place(),
                clock(),
                &asked.with_windows_on(21).at_most(100_000),
            )
            .unwrap();
        let days = found.days.value;
        found
            .answer
            .value
            .windows
            .iter()
            .map(|w| {
                let day = days.iter().find(|d| d.window.contains(w.at.from)).unwrap();
                (day.vara(), day.ghati_pala(w.at.from).unwrap().ghati)
            })
            .collect::<Vec<_>>()
    };
    let journey = MuhurtaRequest::new(ActivityRules::raman_yatra());
    // Thursday's shool is the south, released past 22 ghatis.
    let south = starts(journey.clone().towards(Direction::South));
    let thursdays: Vec<u8> = south
        .iter()
        .filter(|(vara, _)| *vara == Vara::Guruvara)
        .map(|(_, ghati)| *ghati)
        .collect();
    assert!(
        !thursdays.is_empty(),
        "three Thursdays hold a journey south"
    );
    assert!(thursdays.iter().all(|g| *g >= 22), "{thursdays:?}");
    // Unasked, or asked east, a Thursday opens before its 22nd ghati.
    for asked in [journey.clone(), journey.towards(Direction::East)] {
        assert!(
            starts(asked)
                .iter()
                .any(|(vara, ghati)| *vara == Vara::Guruvara && *ghati < 22),
            "a Thursday open from sunrise"
        );
    }
    // Held to the forenoon, Thursday's release at 22 ghatis falls past
    // noon, so a journey south keeps none of that day.
    let mut forenoon = ActivityRules::raman_yatra();
    forenoon.hours = Hours::Forenoon;
    let south = starts(MuhurtaRequest::new(forenoon).towards(Direction::South));
    assert!(!south.is_empty(), "the other days keep their forenoons");
    assert!(
        south.iter().all(|(vara, _)| *vara != Vara::Guruvara),
        "{south:?}"
    );
}
