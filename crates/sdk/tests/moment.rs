//! What a chart's instant falls in, and the two rules that read it.
//!
//! A sky of 6000 charts found `LAGNA_GANDANTA` and
//! `BJ_ECLIPSED_MOON_IN_LAGNA_MARS_EIGHTH` never holding
//! (`03-design/rule-cofiring-measured.md`): the bridge gave the rules no
//! lagna span and no eclipse. These hold that a chart now carries both, and
//! that a rule reading them holds where it should and not where it should
//! not.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests fail by panicking"
)]

use teistro::catalogue::Rashi;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::rules::{Readings, Rule};
use teistro::{
    BirthEclipse, ChartRequest, Context, Ephemeris, Moment, RuleInputs, RuleRequest, ShippedRules,
    UtcOffset,
};

fn sdk() -> Context {
    Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("the built-in ephemeris")
}

fn place(latitude: f64, longitude: f64) -> Place {
    Place::new(
        Latitude::try_new(latitude).unwrap(),
        Longitude::try_new(longitude).unwrap(),
        Altitude::try_new(0.0).unwrap(),
    )
}

fn moment(sdk: &Context, jd: f64, at: Place) -> Moment {
    let request = ChartRequest::at(at, UtcOffset::UTC).with_moment();
    sdk.chart()
        .reading(JulianDay::<Utc>::literal(jd), &request)
        .expect("a chart")
        .value
        .moment
        .expect("asked for")
}

/// The total lunar eclipse of 2025 March 14, greatest at 06:58:43 UT, from
/// New York with the Moon up; and the total solar eclipse of 2024 April 8
/// from Dallas, in totality at about 18:42 UT. Neither is an eclipse from
/// where it cannot be seen.
#[test]
fn a_chart_says_the_eclipse_its_place_sees() {
    let sdk = sdk();
    let new_york = place(40.71, -74.0);
    let dallas = place(32.78, -96.80);
    let kathmandu = place(27.7172, 85.324);
    let lunar = 2_460_748.790_8;
    let solar = 2_460_409.279_2;
    assert!(matches!(
        moment(&sdk, lunar, new_york).eclipse,
        BirthEclipse::Lunar { .. }
    ));
    assert!(matches!(
        moment(&sdk, solar, dallas).eclipse,
        BirthEclipse::Solar { .. }
    ));
    // The Moon is below Kathmandu's horizon at the lunar eclipse, and the
    // Sun below it at the solar one.
    assert_eq!(moment(&sdk, lunar, kathmandu).eclipse, BirthEclipse::None);
    assert_eq!(moment(&sdk, solar, kathmandu).eclipse, BirthEclipse::None);
    // A day later nothing is eclipsed anywhere.
    assert_eq!(
        moment(&sdk, lunar + 1.0, new_york).eclipse,
        BirthEclipse::None
    );
}

/// A rule that reads the eclipse holds at the lunar eclipse and not a day
/// later, read from the document as a consumer reads it.
#[test]
fn a_rule_reads_the_eclipse_from_the_document() {
    let sdk = sdk();
    let rule: Rule = serde_json::from_value(serde_json::json!({
        "key": "BORN_IN_A_LUNAR_ECLIPSE",
        "category": "test",
        "source": {"text": "test"},
        "conditions": [{"type": "birth-during-eclipse", "kind": "lunar"}],
        "outcomes": [],
    }))
    .unwrap();
    let set = RuleRequest::default()
        .with_rules([rule.clone()])
        .rule_set()
        .unwrap();
    let request = ChartRequest::at(place(40.71, -74.0), UtcOffset::UTC);
    let held = |jd: f64| {
        let read = sdk
            .chart()
            .readings_with_rules(&[JulianDay::<Utc>::literal(jd)], &request, &set)
            .unwrap()
            .value;
        let (document, reading) = read.first().unwrap();
        // Both roads: the reading's own answer and the document's inputs.
        let inputs = RuleInputs::of(document).unwrap();
        let direct = inputs.evaluator(Readings::TEXTS).evaluate(&rule).present;
        assert_eq!(direct, !reading.present.is_empty());
        direct
    };
    assert!(held(2_460_748.790_8));
    assert!(!held(2_460_749.790_8));
}

/// The rising sign's edges bound the instant, and BPHS ch. 92 v. 4's half
/// ghatika at the start of a fire sign holds three minutes after it rises
/// and not half an hour after.
#[test]
fn the_lagna_gandanta_holds_at_a_fire_signs_first_minutes() {
    let sdk = sdk();
    let kathmandu = place(27.7172, 85.324);
    // Walk the signs from an instant until a fire sign rises.
    let mut at = 2_460_600.0;
    let fire = loop {
        let found = moment(&sdk, at, kathmandu);
        let (rose, sets) = (found.rising.rose.unwrap(), found.rising.sets.unwrap());
        assert!(rose.get() <= at && at < sets.get());
        if matches!(
            found.rising.sign,
            Rashi::Aries | Rashi::Leo | Rashi::Sagittarius
        ) {
            break rose.get();
        }
        at = sets.get() + 1.0 / 1440.0;
    };
    let set = RuleRequest::shipped([ShippedRules::Gandantas])
        .rule_set()
        .unwrap();
    let request = ChartRequest::at(kathmandu, UtcOffset::UTC);
    let gandanta = |jd: f64| {
        let read = sdk
            .chart()
            .readings_with_rules(&[JulianDay::<Utc>::literal(jd)], &request, &set)
            .unwrap()
            .value;
        read.first()
            .unwrap()
            .1
            .present
            .iter()
            .any(|held| held.rule.key == "LAGNA_GANDANTA")
    };
    let minute = 1.0 / 1440.0;
    assert!(gandanta(fire + 3.0 * minute));
    assert!(!gandanta(fire + 30.0 * minute));
}
