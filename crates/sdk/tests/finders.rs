//! The finders (A2-1) on real skies, each answer read back through the
//! record it is built from: an election through `almanac.muhurta`, an
//! observance through each tradition's `almanac.festivals`, a people
//! digest through `chart.gochar`, `chart.hits` and the almanac's day
//! (`03-design/election-search.md`, `observance-finder.md`,
//! `people-digest.md`).
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking"
)]

use serde_json::{Value, json};
use teistro::finders::{ElectRequest, ObservanceRequest, Parting, PeopleRequest, Reckoning};
use teistro::{ChartRequest, Context, Ephemeris, GocharRequest, HitRequest};

const KATHMANDU: &str = r#""latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1400, "utcOffsetSeconds": 20700"#;

fn sky(ephemeris: Ephemeris) -> Context {
    Context::builder()
        .ephemeris([ephemeris])
        .build()
        .expect("a context")
}

#[test]
fn an_election_is_the_muhurta_search_ranked_and_split() {
    let sdk = sky(Ephemeris::Builtin);
    let asked = ElectRequest::from_json(&format!(
        r#"{{"first": {{"year": 2026, "month": 11, "day": 20}}, "last": {{"year": 2026, "month": 12, "day": 15}},
            {KATHMANDU}, "muhurta": {{"rules": "RAMAN_MARRIAGE", "most": 5}}}}"#
    ))
    .unwrap();
    let elected = sdk.almanac().elect(&asked).unwrap().value;
    let searched = sdk
        .almanac()
        .muhurta(
            &asked.first,
            &asked.last,
            &asked.place,
            asked.clock.offset().unwrap(),
            &asked.muhurta,
        )
        .unwrap()
        .value;
    assert!(!elected.windows.is_empty(), "a marriage window in 26 days");
    assert_eq!(elected.windows.len(), searched.windows.len());
    assert_eq!(elected.days_judged, searched.days_judged);
    for (at, (window, judged)) in elected.windows.iter().zip(&searched.windows).enumerate() {
        assert_eq!(window.rank, at + 1);
        assert_eq!(window.at, judged.at);
        assert_eq!(window.open, judged.open());
        assert_eq!(
            window.holds.len() + window.fails.len(),
            judged.clauses.len()
        );
        assert!(window.holds.iter().all(teistro_muhurta::Clause::favourable));
        assert!(!window.fails.iter().any(teistro_muhurta::Clause::favourable));
    }
}

/// Each tradition's days, by rule, as `almanac.festivals` reckons them.
fn festival_days(sdk: &Context, asked: &ObservanceRequest, at: usize) -> Vec<Value> {
    let tradition = &asked.traditions[at];
    let found = sdk
        .almanac()
        .festivals(
            &asked.first,
            &asked.last,
            &asked.place,
            asked.clock.offset().unwrap(),
            &tradition.festivals,
        )
        .unwrap()
        .value;
    let found = serde_json::to_value(&found).unwrap();
    let rule = json!(tradition.rule);
    ["observances", "ekadashis"]
        .iter()
        .flat_map(|list| found[list].as_array().cloned().unwrap_or_default())
        .filter(|one| one["rule"] == rule)
        .map(|one| one["day"].clone())
        .collect()
}

/// The days the finder says a tradition keeps.
fn kept_days(found: &Value, at: usize) -> Vec<Value> {
    found["occurrences"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["kept"][at]["day"].clone())
        .filter(|day| !day.is_null())
        .collect()
}

#[test]
fn two_rules_under_one_sky_part_on_the_rule() {
    let sdk = sky(Ephemeris::Builtin);
    let asked = ObservanceRequest::from_json(&format!(
        r#"{{"year": 2026, {KATHMANDU}, "traditions": [
            {{"name": "smarta", "festivals": {{"rules": "DHARMASINDHU"}}, "rule": "EKADASHI_SMARTA"}},
            {{"name": "vaishnava", "festivals": {{"rules": "DHARMASINDHU"}}, "rule": "EKADASHI_VAISHNAVA"}}]}}"#
    ))
    .unwrap();
    let found = sdk.almanac().observance(&asked, None).unwrap().value;
    // A year holds 24 or so Ekadashis, each paired across the two.
    assert!(
        (23..=27).contains(&found.occurrences.len()),
        "{}",
        found.occurrences.len()
    );
    assert_eq!(
        found.parted,
        found.occurrences.iter().filter(|o| !o.agree).count()
    );
    for parted in found.occurrences.iter().filter(|o| !o.agree) {
        assert!(matches!(
            parted.reason,
            Some(Parting::Rule | Parting::Published)
        ));
    }
    let json = serde_json::to_value(&found).unwrap();
    for at in 0..2 {
        assert_eq!(
            kept_days(&json, at),
            festival_days(&sdk, &asked, at),
            "tradition {at}"
        );
    }
}

#[test]
fn one_rule_under_two_skies_parts_on_the_sky() {
    let modern = sky(Ephemeris::Builtin);
    let official = sky(Ephemeris::SuryaSiddhanta);
    let asked = ObservanceRequest::from_json(&format!(
        r#"{{"first": {{"year": 2025, "month": 9, "day": 1}}, "last": {{"year": 2025, "month": 12, "day": 31}},
            {KATHMANDU}, "traditions": [
            {{"name": "modern", "festivals": {{"rules": "DHARMASINDHU"}}, "rule": "EKADASHI_SMARTA"}},
            {{"name": "official", "festivals": {{"rules": "DHARMASINDHU"}}, "rule": "EKADASHI_SMARTA", "reckoning": "OTHER"}}]}}"#
    ))
    .unwrap();
    assert_eq!(asked.traditions[1].reckoning, Reckoning::Other);
    let found = modern
        .almanac()
        .observance(&asked, Some(&official))
        .unwrap()
        .value;
    // One settings hash, two ephemerides: two skies, told apart by the provider.
    assert_ne!(found.traditions[0].provider, found.traditions[1].provider);
    for parted in found.occurrences.iter().filter(|o| !o.agree) {
        assert!(matches!(
            parted.reason,
            Some(Parting::Sky | Parting::Published)
        ));
    }
    let json = serde_json::to_value(&found).unwrap();
    assert_eq!(kept_days(&json, 0), festival_days(&modern, &asked, 0));
    assert_eq!(kept_days(&json, 1), festival_days(&official, &asked, 1));

    let refused = modern.almanac().observance(&asked, None).unwrap_err();
    assert_eq!(refused.field(), Some("observance.traditions[1].reckoning"));
}

#[test]
fn a_people_digest_is_each_persons_gochar_and_hits_over_the_day() {
    let sdk = sky(Ephemeris::Builtin);
    let asked = PeopleRequest::from_json(&format!(
        r#"{{"day": {{"year": 2026, "month": 11, "day": 1}}, {KATHMANDU}, "people": [
            {{"name": "Asha", "instant": 2447000.25, {KATHMANDU}}},
            {{"name": "Ravi", "instant": 2451545.0, "latitudeDeg": 19.076, "longitudeDeg": 72.8777, "utcOffsetSeconds": 19800}}]}}"#
    ))
    .unwrap();
    let digest = sdk.almanac().people(&asked).unwrap().value;
    let day = sdk
        .almanac()
        .of(
            &asked.day,
            &asked.day,
            &asked.place,
            asked.clock.offset().unwrap(),
        )
        .unwrap()
        .value;
    assert_eq!(digest.sunrise, day[0].day.sunrise);
    assert_eq!(digest.next_sunrise, day[0].day.next_sunrise);
    assert_eq!(digest.people.len(), 2);
    for (person, read) in asked.people.iter().zip(&digest.people) {
        assert_eq!(read.name, person.name);
        let natal = sdk
            .chart()
            .reading(
                person.instant,
                &ChartRequest::at(person.place, person.clock.offset().unwrap())
                    .with_dashas([asked.dasha]),
            )
            .unwrap()
            .value;
        let gochar = sdk
            .chart()
            .gochar(&natal, &GocharRequest::at(digest.sunrise))
            .unwrap()
            .value;
        assert_eq!(
            serde_json::to_value(&read.transits).unwrap(),
            serde_json::to_value(&gochar[0].grahas).unwrap()
        );
        let hits = sdk
            .chart()
            .hits(
                &natal,
                &HitRequest::between(digest.sunrise, digest.next_sunrise),
            )
            .unwrap()
            .value;
        assert_eq!(
            serde_json::to_value(&read.hits).unwrap(),
            serde_json::to_value(&hits).unwrap()
        );
        assert_eq!(read.dasha.len(), usize::from(asked.depth));
        for period in &read.dasha {
            assert!(
                period.from.get() <= digest.sunrise.get() && digest.sunrise.get() < period.to.get()
            );
        }
        assert_eq!(read.chandrashtama, read.chandra_house == 8);
    }
}
