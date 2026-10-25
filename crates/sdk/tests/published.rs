//! Published days laid over the rules' (`03-design/published-overrides.md`):
//! each action, the layering, the edges of the range and every refusal,
//! asked of a real year at Kathmandu. The rules and days are read off the
//! computed answer, so the test holds for any pack's days.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    missing_docs,
    reason = "a test fails by panicking and indexes what it built"
)]

use std::collections::BTreeMap;

use teistro::catalogue::Calendar;
use teistro::festival::{
    Observances, PublishedDay, PublishedEntry, PublishedSet, PublishedStatus, apply_published,
};
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{
    CalendarDate, Context, Ephemeris, FestivalPack, FestivalRequest, FixedDay, UtcOffset,
};
use teistro_calendar::{CalendarSystem as _, Gregorian};

fn kathmandu() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    )
}

fn date(year: i32, month: u8, day: u8) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, year, month, day)
}

fn fixed(day: &CalendarDate) -> FixedDay {
    Gregorian.fixed_of(day).unwrap()
}

fn shifted(day: &CalendarDate, by: i64) -> CalendarDate {
    Gregorian.date_of(fixed(day).plus_days(by)).unwrap()
}

fn asked(
    request: &FestivalRequest,
    from: &CalendarDate,
    to: &CalendarDate,
) -> teistro::Envelope<Observances> {
    Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
        .almanac()
        .festivals(
            from,
            to,
            &kathmandu(),
            UtcOffset::literal(5, 45, 0),
            request,
        )
        .unwrap()
}

fn year() -> (CalendarDate, CalendarDate) {
    (date(2027, 1, 1), date(2027, 12, 31))
}

fn set(id: &str, version: u32, entries: Vec<PublishedEntry>) -> PublishedSet {
    PublishedSet {
        format: String::from(teistro::festival::PUBLISHED_FORMAT),
        publisher: String::from("a patro printed in Nepal"),
        id: String::from(id),
        version,
        entries,
    }
}

fn set_entry(rule: &str, day: &CalendarDate) -> PublishedEntry {
    PublishedEntry::Set {
        rule: rule.to_owned(),
        day: PublishedDay::from(day),
        month: None,
        adhika: None,
        window: None,
        source: String::from("p. 1"),
    }
}

/// The first karmakala observance of the year away from its ends, so a
/// day either side stays in the range.
fn base() -> (FestivalRequest, Observances, usize) {
    let request = FestivalRequest::from(FestivalPack::Dharmasindhu);
    let (from, to) = year();
    let found = asked(&request, &from, &to).value;
    let at = found
        .observances
        .iter()
        .position(|o| o.day.month > 1 && o.day.month < 12)
        .unwrap();
    (request, found, at)
}

#[test]
fn no_set_is_the_answer_it_was() {
    let (request, found, _) = base();
    let text = serde_json::to_string(&found).unwrap();
    for absent in ["\"published\"", "\"added\"", "\"unplaced\"", "\"applied\""] {
        assert!(!text.contains(absent), "{absent}");
    }
    assert_eq!(request.published(), &[]);
}

#[test]
fn a_set_moves_a_day_and_keeps_the_rule_s() {
    let (request, found, at) = base();
    let (rule, day) = (
        found.observances[at].rule.clone(),
        found.observances[at].day.clone(),
    );
    let moved = shifted(&day, 1);
    let (from, to) = year();
    let plain = asked(&request, &from, &to);
    let answer = asked(
        &request
            .clone()
            .with_published(set("np", 1, vec![set_entry(&rule, &moved)])),
        &from,
        &to,
    );
    let kept = answer
        .value
        .observances
        .iter()
        .find(|o| o.rule == rule && o.day == moved)
        .unwrap();
    let published = kept.published.as_ref().unwrap();
    assert_eq!(published.status, PublishedStatus::Published);
    assert_eq!(published.computed.as_ref(), Some(&day));
    assert_eq!((published.set.as_str(), published.version), ("np", 1));
    assert!(
        kept.parana.is_empty(),
        "a moved observance keeps no computed parana"
    );
    assert_eq!(answer.value.applied.len(), 1);
    assert_ne!(answer.provenance.input_hash, plain.provenance.input_hash);
    // The same day published says so and keeps no `computed`.
    let same = asked(
        &request.with_published(set("np", 1, vec![set_entry(&rule, &day)])),
        &from,
        &to,
    );
    let kept = same
        .value
        .observances
        .iter()
        .find(|o| o.rule == rule && o.day == day)
        .unwrap();
    assert_eq!(kept.published.as_ref().unwrap().computed, None);
}

#[test]
fn a_later_set_replaces_an_earlier_one_s_entry() {
    let (request, found, at) = base();
    let (rule, day) = (
        found.observances[at].rule.clone(),
        found.observances[at].day.clone(),
    );
    let (from, to) = year();
    let answer = asked(
        &request
            .with_published(set("nation", 1, vec![set_entry(&rule, &shifted(&day, 1))]))
            .with_published(set("state", 2, vec![set_entry(&rule, &shifted(&day, -1))])),
        &from,
        &to,
    );
    let kept: Vec<_> = answer
        .value
        .observances
        .iter()
        .filter(|o| o.published.is_some())
        .collect();
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].day, shifted(&day, -1));
    let published = kept[0].published.as_ref().unwrap();
    assert_eq!(
        (published.set.as_str(), published.computed.as_ref()),
        ("state", Some(&day))
    );
    let ids: Vec<&str> = answer.value.applied.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(ids, ["nation", "state"]);
    assert_ne!(answer.value.applied[0].hash, answer.value.applied[1].hash);
}

#[test]
fn withdraw_keeps_the_day_and_add_gives_one() {
    let (request, found, at) = base();
    let (rule, day) = (
        found.observances[at].rule.clone(),
        found.observances[at].day.clone(),
    );
    let (from, to) = year();
    let entries = vec![
        PublishedEntry::Withdraw {
            rule: rule.clone(),
            day: PublishedDay::from(&day),
            month: None,
            adhika: None,
            source: String::from("correction 2"),
        },
        PublishedEntry::Add {
            key: String::from("A_NATIONAL_DAY"),
            day: PublishedDay::from(&date(2027, 5, 29)),
            names: [(String::from("en-Latn"), String::from("A national day"))].into(),
            window: None,
            source: String::from("notice 1"),
        },
        // Outside the range: passed over.
        PublishedEntry::Add {
            key: String::from("NEXT_YEAR"),
            day: PublishedDay::from(&date(2028, 5, 29)),
            names: BTreeMap::default(),
            window: None,
            source: String::from("notice 2"),
        },
    ];
    let answer = asked(&request.with_published(set("np", 1, entries)), &from, &to).value;
    let kept = answer
        .observances
        .iter()
        .find(|o| o.rule == rule && o.day == day)
        .unwrap();
    assert_eq!(
        kept.published.as_ref().unwrap().status,
        PublishedStatus::Withdrawn
    );
    assert_eq!(answer.added.len(), 1);
    assert_eq!(answer.added[0].key, "A_NATIONAL_DAY");
    assert_eq!(answer.added[0].names["en-Latn"], "A national day");
}

#[test]
fn a_day_no_occurrence_takes_is_unplaced() {
    let (request, found, at) = base();
    let (rule, day) = (
        found.observances[at].rule.clone(),
        found.observances[at].day.clone(),
    );
    let (from, to) = year();
    // Far from any of the rule's occurrences, but inside the range.
    let far = found
        .observances
        .iter()
        .filter(|o| o.rule == rule)
        .map(|o| o.day.clone())
        .collect::<Vec<_>>();
    let lonely = (1..300)
        .map(|k| shifted(&day, k * if day.month > 6 { -1 } else { 1 }))
        .find(|candidate| far.iter().all(|d| (days_between(d, candidate)).abs() > 7))
        .unwrap();
    let answer = asked(
        &request.with_published(set("np", 1, vec![set_entry(&rule, &lonely)])),
        &from,
        &to,
    )
    .value;
    assert_eq!(answer.unplaced.len(), 1);
    assert_eq!(
        (
            answer.unplaced[0].rule.as_str(),
            &answer.unplaced[0].day,
            answer.unplaced[0].entry
        ),
        (rule.as_str(), &lonely, 0)
    );
    assert!(answer.observances.iter().all(|o| o.published.is_none()));
}

fn days_between(a: &CalendarDate, b: &CalendarDate) -> i64 {
    fixed(b).get() - fixed(a).get()
}

#[test]
fn a_day_published_across_the_range_s_edge_moves_with_it() {
    let (request, found, at) = base();
    let (rule, day) = (
        found.observances[at].rule.clone(),
        found.observances[at].day.clone(),
    );
    // A range ending on the computed day: published a day later, it
    // leaves; a range starting the day after: it arrives.
    let moved = vec![set_entry(&rule, &shifted(&day, 1))];
    let request = request.with_published(set("np", 1, moved));
    let before = asked(&request, &shifted(&day, -3), &day).value;
    assert!(before.observances.iter().all(|o| o.rule != rule));
    assert!(
        before.unplaced.is_empty(),
        "outside the range, not unplaced"
    );
    let after = asked(&request, &shifted(&day, 1), &shifted(&day, 3)).value;
    assert!(
        after
            .observances
            .iter()
            .any(|o| o.rule == rule && o.published.is_some())
    );
}

#[test]
fn a_moved_fast_keeps_only_the_published_window() {
    let request = FestivalRequest::from(FestivalPack::Dharmasindhu);
    let (from, to) = year();
    let found = asked(&request, &from, &to).value;
    let fast = found
        .ekadashis
        .iter()
        .find(|f| {
            f.parana.as_ref().is_some_and(|p| p.kept.is_some())
                && f.day.month > 1
                && f.day.month < 12
        })
        .unwrap();
    let window = fast.parana.as_ref().unwrap().kept.unwrap().span;
    let entries = vec![PublishedEntry::Set {
        rule: fast.rule.clone(),
        day: PublishedDay::from(&shifted(&fast.day, 1)),
        month: Some(fast.month),
        adhika: Some(fast.adhika),
        window: Some(window),
        source: String::from("p. 3"),
    }];
    let answer = asked(&request.with_published(set("np", 1, entries)), &from, &to).value;
    let moved = answer
        .ekadashis
        .iter()
        .find(|f| f.published.is_some())
        .unwrap();
    assert_eq!(moved.parana, None);
    assert_eq!(moved.published.as_ref().unwrap().window, Some(window));
}

#[test]
fn two_occurrences_that_near_are_refused_without_a_month() {
    let (_, mut found, at) = base();
    let mut twin = found.observances[at].clone();
    let day = twin.day.clone();
    twin.day = shifted(&day, 2);
    found.observances.push(twin);
    let rule = found.observances[at].rule.clone();
    let range = (fixed(&year().0), fixed(&year().1));
    let refused = apply_published(
        &mut found.clone(),
        &[set("np", 1, vec![set_entry(&rule, &shifted(&day, 1))])],
        Calendar::Gregorian,
        range,
    )
    .unwrap_err();
    assert_eq!(refused.field(), Some("published[0].entries[0]"));
    assert!(
        refused.message.contains("2 occurrences"),
        "{}",
        refused.message
    );
    // Its month names one, and the other month none.
    let month = found.observances[at].month;
    let named = PublishedEntry::Set {
        rule: rule.clone(),
        day: PublishedDay::from(&shifted(&day, 1)),
        month: Some(month),
        adhika: Some(!found.observances[at].adhika),
        window: None,
        source: String::from("p. 1"),
    };
    let mut other = found.clone();
    apply_published(
        &mut other,
        &[set("np", 1, vec![named])],
        Calendar::Gregorian,
        range,
    )
    .unwrap();
    assert_eq!(other.unplaced.len(), 1);
}

#[test]
fn a_set_is_refused_by_name() {
    let (request, found, at) = base();
    let rule = found.observances[at].rule.clone();
    let day = found.observances[at].day.clone();
    let refusal = |set: PublishedSet| request.clone().with_published(set).check().unwrap_err();
    let unknown = refusal(set("np", 1, vec![set_entry("NO_SUCH_RULE", &day)]));
    assert_eq!(unknown.field(), Some("published[0].entries[0].rule"));
    let reused = refusal(set(
        "np",
        1,
        vec![PublishedEntry::Add {
            key: rule.clone(),
            day: PublishedDay::from(&day),
            names: BTreeMap::default(),
            window: None,
            source: String::from("p. 1"),
        }],
    ));
    assert_eq!(reused.field(), Some("published[0].entries[0].key"));
    let twice = refusal(set(
        "np",
        1,
        vec![set_entry(&rule, &day), set_entry(&rule, &day)],
    ));
    assert_eq!(twice.field(), Some("published[0].entries[1]"));
    let mut other = set("np", 1, Vec::new());
    other.format = String::from("teistro.published/2");
    assert_eq!(refusal(other).field(), Some("published[0].format"));
    let no_day = refusal(set("np", 1, vec![set_entry(&rule, &date(2027, 2, 30))]));
    assert_eq!(no_day.field(), Some("published[0].entries[0].day"));

    // A dated rule and an anniversary under new keys are rules of the
    // request too: a `SET` may name them and an `ADD` may not.
    let mut dated = FestivalPack::NepalPanchanga.dated().remove(0);
    dated.key = String::from("A_CIVIL_DAY");
    let barkhi = teistro::festival::Anniversary {
        key: String::from("GRANDFATHER"),
        origin: teistro::festival::Origin::Lunar {
            month: teistro::catalogue::Masa::Kartika,
            tithi: teistro::catalogue::Tithi::ShuklaDashami,
            adhika: false,
            half: teistro::festival::Half::default(),
            at: None,
        },
        kind: teistro::festival::AnniversaryKind::Shraddha,
        window: None,
        first_year_adhika: teistro::festival::FirstYearAdhika::Adhika,
        in_samsarpa: teistro::festival::InSamsarpa::Also,
        janma: teistro::festival::JanmaOptions::default(),
    };
    let widened = request.clone().with_dated(dated).with_anniversary(barkhi);
    for key in ["A_CIVIL_DAY", "GRANDFATHER"] {
        let carried = widened
            .clone()
            .with_published(set("np", 1, vec![set_entry(key, &day)]));
        carried.check().unwrap();
        let added = widened.clone().with_published(set(
            "np",
            1,
            vec![PublishedEntry::Add {
                key: String::from(key),
                day: PublishedDay::from(&day),
                names: BTreeMap::default(),
                window: None,
                source: String::from("p. 1"),
            }],
        ));
        assert_eq!(
            added.check().unwrap_err().field(),
            Some("published[0].entries[0].key")
        );
    }
}

#[test]
fn a_binding_s_json_reads_the_sets() {
    let (request, found, at) = base();
    let rule = &found.observances[at].rule;
    let day = &found.observances[at].day;
    let text = format!(
        r#"{{"rules": "DHARMASINDHU", "published": [{{"format": "teistro.published/1", "publisher": "p", "id": "np", "version": 1,
            "entries": [{{"action": "SET", "rule": "{rule}", "day": {}, "source": "p. 1"}}]}}]}}"#,
        serde_json::to_string(&PublishedDay::from(day)).unwrap()
    );
    let read = FestivalRequest::from_json(&text).unwrap();
    let mut expected = set("np", 1, vec![set_entry(rule, day)]);
    expected.publisher = String::from("p");
    assert_eq!(read, request.with_published(expected));
    let wrong = FestivalRequest::from_json(r#"{"rules": "DHARMASINDHU", "published": [{"format": "teistro.published/1", "publisher": "p", "id": "np", "version": 1, "entries": [], "extra": 1}]}"#).unwrap_err();
    assert_eq!(wrong.field(), Some("festivals.published[0].extra"));
}

#[test]
fn a_day_printed_in_another_calendar_lands_in_the_range_s() {
    let (request, found, at) = base();
    let (rule, day) = (
        found.observances[at].rule.clone(),
        found.observances[at].day.clone(),
    );
    let moved = shifted(&day, 1);
    let sambat = teistro_calendar::shipped(Calendar::BikramSambat).unwrap();
    let printed = sambat.date_of(fixed(&moved)).unwrap();
    let (from, to) = year();
    let answer = asked(
        &request.with_published(set("np", 1, vec![set_entry(&rule, &printed)])),
        &from,
        &to,
    )
    .value;
    let kept = answer
        .observances
        .iter()
        .find(|o| o.published.is_some())
        .unwrap();
    assert_eq!((kept.rule.as_str(), &kept.day), (rule.as_str(), &moved));
    assert_eq!(kept.day.calendar, Calendar::Gregorian);
}

/// The autumn of 2027, which holds most of a pack's festivals.
fn autumn() -> (CalendarDate, CalendarDate) {
    (date(2027, 9, 1), date(2027, 11, 30))
}

fn context(profile: &str, ephemeris: Ephemeris) -> Context {
    Context::builder()
        .profile(profile)
        .ephemeris([ephemeris])
        .build()
        .unwrap()
}

/// The first observance two answers give on different days, by rule and
/// nearest day: its rule, this answer's day and the other's.
fn first_parting(
    ours: &Observances,
    theirs: &Observances,
) -> Option<(String, CalendarDate, CalendarDate)> {
    ours.observances.iter().find_map(|o| {
        let near = theirs
            .observances
            .iter()
            .filter(|t| t.rule == o.rule)
            .min_by_key(|t| fixed(&o.day).days_until(fixed(&t.day)).abs())?;
        let apart = fixed(&o.day).days_until(fixed(&near.day)).abs();
        (apart > 0 && apart <= 3).then(|| (o.rule.clone(), o.day.clone(), near.day.clone()))
    })
}

fn reasons(
    sky: &Context,
    request: &FestivalRequest,
    places: &[teistro::ReckonedPlace],
    skies: &[&Context],
) -> Vec<teistro::PublishedReason> {
    let (from, to) = autumn();
    sky.almanac()
        .published_reasons(
            (&from, &to),
            (&kathmandu(), UtcOffset::literal(5, 45, 0)),
            request,
            places,
            skies,
        )
        .unwrap()
        .value
        .reasons
}

#[test]
fn a_published_day_is_explained_by_the_first_change_that_gives_it() {
    use teistro::PublishedCause;

    let modern = context("nepali-default", Ephemeris::Builtin);
    let committee = context("nepali-committee", Ephemeris::SuryaSiddhanta);
    let (from, to) = autumn();
    let clock = UtcOffset::literal(5, 45, 0);
    let plain = FestivalRequest::from(FestivalPack::Nepal);
    let ours = modern
        .almanac()
        .festivals(&from, &to, &kathmandu(), clock, &plain)
        .unwrap()
        .value;

    // Nothing moved, nothing to explain.
    assert_eq!(reasons(&modern, &plain, &[], &[&committee]), []);

    // The committee's day, published over the modern sky: the sky.
    let theirs = committee
        .almanac()
        .festivals(&from, &to, &kathmandu(), clock, &plain)
        .unwrap()
        .value;
    let (rule, here, there) =
        first_parting(&ours, &theirs).expect("the skies part somewhere in an autumn");
    let request = plain
        .clone()
        .with_published(set("np", 1, vec![set_entry(&rule, &there)]));
    let [explained] = reasons(&modern, &request, &[], &[&committee])
        .try_into()
        .unwrap();
    assert_eq!(
        (explained.rule.as_str(), &explained.day, &explained.computed),
        (rule.as_str(), &there, &here)
    );
    assert!(
        matches!(explained.reason, PublishedCause::Sky { ref profile, .. } if profile == "nepali-committee")
    );
    assert!(explained.tried[0].reproduces);

    // Dharmasindhu's rule's day, published over Nepal's rules: the rule.
    let dharmasindhu = modern
        .almanac()
        .festivals(
            &from,
            &to,
            &kathmandu(),
            clock,
            &FestivalRequest::from(FestivalPack::Dharmasindhu),
        )
        .unwrap()
        .value;
    let (rule, _, there) =
        first_parting(&ours, &dharmasindhu).expect("the two packs part somewhere in an autumn");
    let request = plain
        .clone()
        .with_published(set("np", 1, vec![set_entry(&rule, &there)]));
    let [explained] = reasons(&modern, &request, &[], &[]).try_into().unwrap();
    assert_eq!(
        explained.reason,
        PublishedCause::Rule {
            pack: "DHARMASINDHU"
        }
    );

    // A day no change gives: the publisher's own, every change shown.
    let (rule, here) = (
        ours.observances[0].rule.clone(),
        ours.observances[0].day.clone(),
    );
    let honolulu = teistro::ReckonedPlace {
        name: String::from("Honolulu"),
        place: Place::new(
            Latitude::literal(21.3069),
            Longitude::literal(-157.8583),
            Altitude::literal(0.0),
        ),
        offset: UtcOffset::literal(-10, 0, 0),
    };
    let request =
        plain
            .clone()
            .with_published(set("np", 1, vec![set_entry(&rule, &shifted(&here, 3))]));
    let [explained] = reasons(&modern, &request, &[honolulu], &[&committee])
        .try_into()
        .unwrap();
    assert_eq!(explained.reason, PublishedCause::Publisher);
    assert!(explained.tried.len() >= 2 && explained.tried.iter().all(|t| !t.reproduces));
    assert_eq!(
        explained.tried[0].change,
        PublishedCause::Place {
            name: String::from("Honolulu")
        }
    );
}

/// A fast, a following observance and a dated one are explained as an
/// observance is: each found again under every change by its rule, set
/// and day, and each pack asked with its own rule of that kind.
#[test]
fn a_fast_a_following_and_a_dated_day_are_explained_too() {
    let modern = context("nepali-default", Ephemeris::Builtin);
    let (from, to) = autumn();
    let plain = FestivalRequest::from(FestivalPack::NepalPanchanga);
    let ours = modern
        .almanac()
        .festivals(
            &from,
            &to,
            &kathmandu(),
            UtcOffset::literal(5, 45, 0),
            &plain,
        )
        .unwrap()
        .value;
    let fast = ours.ekadashis.first().expect("an autumn holds an Ekadashi");
    let dated = ours
        .dated
        .first()
        .expect("an autumn holds a dated observance");
    let following = FestivalPack::NepalPanchanga
        .following()
        .into_iter()
        .find_map(|rule| ours.observances.iter().find(|o| o.rule == rule.key))
        .expect("an autumn holds a following observance");
    let moved = [
        (&fast.rule, &fast.day),
        (&dated.rule, &dated.day),
        (&following.rule, &following.day),
    ];
    let request = plain.clone().with_published(set(
        "np",
        1,
        moved
            .iter()
            .map(|(rule, day)| set_entry(rule, &shifted(day, 1)))
            .collect(),
    ));
    let explained = reasons(&modern, &request, &[], &[]);
    let mut asked: Vec<&str> = moved.iter().map(|(rule, _)| rule.as_str()).collect();
    let mut answered: Vec<&str> = explained.iter().map(|r| r.rule.as_str()).collect();
    asked.sort_unstable();
    answered.sort_unstable();
    assert_eq!(answered, asked);
    for reason in &explained {
        let (_, day) = moved
            .iter()
            .find(|(rule, _)| **rule == reason.rule)
            .unwrap();
        assert_eq!((&reason.day, &reason.computed), (&shifted(day, 1), *day));
    }
}

#[test]
fn a_binding_s_request_is_explained_in_full() {
    let modern = context("nepali-default", Ephemeris::Builtin);
    let committee = context("nepali-committee", Ephemeris::SuryaSiddhanta);
    let asked = teistro::PublishedReasonsRequest::from_json(
        r#"{"first": {"year": 2027, "month": 9, "day": 1}, "last": {"year": 2027, "month": 11, "day": 30},
            "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1400, "utcOffsetSeconds": 20700,
            "festivals": {"rules": "NEPAL", "published": [{"format": "teistro.published/1",
                "publisher": "a patro printed in Nepal", "id": "np", "version": 1, "entries": [
                {"action": "SET", "rule": "LAKSHMI_PUJA", "source": "p. 1",
                 "day": {"calendar": "GREGORIAN", "year": 2027, "month": 11, "day": 1}}]}]},
            "places": [{"name": "Delhi", "latitudeDeg": 28.6139, "longitudeDeg": 77.209, "utcOffsetSeconds": 19800}]}"#,
    )
    .unwrap();
    let answer = modern
        .almanac()
        .published_reasons_request(&[&committee], &asked)
        .unwrap()
        .value;
    for reason in answer["reasons"].as_array().unwrap() {
        assert!(
            reason["day"]["calendar"]
                .as_str()
                .unwrap()
                .starts_with("calendar.")
        );
        let tried = reason["tried"].as_array().unwrap();
        assert_eq!(
            tried[0]["change"],
            serde_json::json!({"by": "PLACE", "name": "Delhi"})
        );
        assert_eq!(tried[1]["change"]["by"], "SKY");
    }
    let stray = teistro::PublishedReasonsRequest::from_json(
        r#"{"first": {"year": 2027, "month": 9, "day": 1}, "latitudeDeg": 27.7, "longitudeDeg": 85.3,
            "utcOffsetSeconds": 20700, "festivals": {"rules": "NEPAL"},
            "places": [{"name": "Nowhere", "latitudeDeg": 91, "longitudeDeg": 0, "utcOffsetSeconds": 0}]}"#,
    )
    .unwrap_err();
    assert_eq!(
        stray.field(),
        Some("publishedReasons.places[0].latitudeDeg")
    );
}

#[test]
fn a_feed_says_who_published_a_day_and_drops_a_withdrawn_one() {
    let (request, found, _) = base();
    // Two days away from the range's ends with no parana, which a moved
    // or withdrawn day would take out of the feed with it.
    let mut quiet = found
        .observances
        .iter()
        .filter(|o| o.parana.is_empty() && o.day.month > 1 && o.day.month < 12);
    let moved = quiet.next().unwrap();
    let gone = quiet.find(|o| o.rule != moved.rule).unwrap();
    let (from, to) = year();
    let entries = vec![
        set_entry(&moved.rule, &shifted(&moved.day, 1)),
        PublishedEntry::Withdraw {
            rule: gone.rule.clone(),
            day: PublishedDay::from(&gone.day),
            month: None,
            adhika: None,
            source: String::from("p. 2"),
        },
        PublishedEntry::Add {
            key: String::from("A_NATIONAL_DAY"),
            day: PublishedDay::from(&date(2027, 5, 29)),
            names: [(String::from("en-Latn"), String::from("A national day"))].into(),
            window: None,
            source: String::from("p. 3"),
        },
    ];
    let feed = |request: &FestivalRequest| {
        let text = Context::builder()
            .ephemeris([Ephemeris::Builtin])
            .build()
            .unwrap()
            .almanac()
            .festival_calendar(
                &from,
                &to,
                &kathmandu(),
                UtcOffset::literal(5, 45, 0),
                request,
                &teistro::FeedOptions::new(1_798_761_600),
            )
            .unwrap()
            .value;
        // Unfolded (RFC 5545 §3.1), its text escapes undone.
        text.replace("\r\n ", "")
            .replace("\\,", ",")
            .replace("\\n", "\n")
    };
    let plain = feed(&request);
    let published = feed(&request.with_published(set("np", 1, entries)));
    let events = |text: &str| text.matches("BEGIN:VEVENT").count();
    // One withdrawn and one added.
    assert_eq!(events(&published), events(&plain));
    let rules_day = format!(
        "The rule's own day is {:04}-{:02}-{:02}.",
        moved.day.year, moved.day.month, moved.day.day
    );
    assert!(published.contains(&format!(
        "Published by a patro printed in Nepal in np version 1: p. 1. {rules_day}"
    )));
    assert!(published.contains("SUMMARY:A national day"));
    assert!(published.contains("Published by a patro printed in Nepal in np version 1: p. 3."));
    assert!(!plain.contains("Published by"));
    assert_eq!(
        published.matches("Answered by").count(),
        2,
        "the moved day and the added one"
    );
}
