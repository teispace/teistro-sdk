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
