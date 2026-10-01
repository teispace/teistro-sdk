//! The essential dignities through the façade: the sect read from the
//! Sun's altitude in every zodiac and at the poles, the chart's daylight as
//! the named alternative, and every knob reported back
//! (`docs/03-design/essential-dignities.md`, `sect-measured.md`).

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they found"
)]

use teistro::catalogue::Graha;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{
    ChartRequest, Context, DignityRequest, DignityRules, Document, Ephemeris, Scores, Sect,
    SectRule, Terms, UtcOffset,
};

fn context(patch: Option<&str>) -> Context {
    let builder = Context::builder()
        .profile("conformance-baseline")
        .ephemeris([Ephemeris::Builtin]);
    match patch {
        Some(json) => builder.settings_json(json),
        None => builder,
    }
    .build()
    .unwrap()
}

fn chart(sdk: &Context, latitude: f64, longitude: f64, jd: f64) -> Document {
    let place = Place::new(
        Latitude::literal(latitude),
        Longitude::literal(longitude),
        Altitude::literal(0.0),
    );
    sdk.chart()
        .reading(
            JulianDay::<Utc>::literal(jd),
            &ChartRequest::at(place, UtcOffset::UTC),
        )
        .unwrap()
        .value
}

/// Tromsø, the corpus's polar fixtures (c028, c029).
const TROMSO: (f64, f64) = (69.6492, 18.9553);

#[test]
fn a_polar_night_noon_is_a_night_chart_and_a_midnight_sun_noon_a_day_chart() {
    let sdk = context(None);
    // 21 December 1988, 11:00 UTC: the Sun culminates three degrees under
    // the horizon. Valens's degrees from the Ascendant read it as a day
    // chart there, which is why the horizon is the altitude.
    let winter = chart(&sdk, TROMSO.0, TROMSO.1, 2_447_516.958_333_333_5);
    let read = sdk
        .chart()
        .dignities(&winter, &DignityRequest::default())
        .unwrap();
    assert_eq!(read.sect, Sect::Night);
    // 21 June 1988, 10:00 UTC: the Sun has not set for weeks.
    let summer = chart(&sdk, TROMSO.0, TROMSO.1, 2_447_333.916_666_666_5);
    let read = sdk
        .chart()
        .dignities(&summer, &DignityRequest::default())
        .unwrap();
    assert_eq!(read.sect, Sect::Day);
}

#[test]
fn the_sect_does_not_depend_on_the_zodiac_and_the_signs_do() {
    let sidereal = context(None);
    let tropical = context(Some(r#"{"frame": {"zodiac": "TROPICAL"}}"#));
    // Kathmandu, through a day at three-hour steps.
    for step in 0..8 {
        let jd = 2_460_676.5 + f64::from(step) * 0.125;
        let one = chart(&sidereal, 27.7172, 85.324, jd);
        let other = chart(&tropical, 27.7172, 85.324, jd);
        let asked = DignityRequest::default();
        let (one, other) = (
            sidereal.chart().dignities(&one, &asked).unwrap(),
            tropical.chart().dignities(&other, &asked).unwrap(),
        );
        assert_eq!(one.sect, other.sect, "step {step}");
        // The same planets, a whole ayanamsha apart.
        let shift = other.planets[3].longitude_deg - one.planets[3].longitude_deg;
        assert!(
            (shift.rem_euclid(360.0) - 24.2).abs() < 0.5,
            "step {step}: {shift}"
        );
    }
}

#[test]
fn the_daylight_rule_reads_the_charts_own_day_part() {
    let sdk = context(None);
    let lit = DignityRequest::default().with_sect_rule(SectRule::Daylight);
    for step in 0..8 {
        let document = chart(&sdk, 27.7172, 85.324, 2_460_676.5 + f64::from(step) * 0.125);
        let read = sdk.chart().dignities(&document, &lit).unwrap();
        let daylight = document.foundation.day.part.is_daylight();
        assert_eq!(read.sect == Sect::Day, daylight, "step {step}");
        assert_eq!(read.sect_rule, SectRule::Daylight);
    }
}

#[test]
fn every_knob_comes_back_and_moves_what_it_names() {
    let sdk = context(None);
    let document = chart(&sdk, 27.7172, 85.324, 2_460_676.75);
    let scores = Scores::new(1, 1, 1, 1, 1, -1, -1, 0);
    let rules = DignityRules::LILLY.with_terms(Terms::Egyptian);
    let asked = DignityRequest::default()
        .with_sect_rule(SectRule::Night)
        .with_rules(rules)
        .with_scores(scores);
    let read = sdk.chart().dignities(&document, &asked).unwrap();
    assert_eq!(
        (read.sect, read.sect_rule, read.rules, read.scores),
        (Sect::Night, SectRule::Night, rules, scores)
    );
    let order: Vec<_> = read.planets.iter().map(|at| at.planet).collect();
    assert_eq!(order, teistro::hellenistic::CHALDEAN_ORDER);
    for at in read.planets {
        let placed = document.foundation.graha(at.planet).unwrap();
        assert_eq!(at.longitude_deg, placed.longitude_deg, "{:?}", at.planet);
        assert_eq!(at.score, at.dignity.score(&scores), "{:?}", at.planet);
    }
    // The Sun and the Moon take no term under any table.
    for luminary in [Graha::Sun, Graha::Moon] {
        let at = read
            .planets
            .iter()
            .find(|at| at.planet == luminary)
            .unwrap();
        assert!(!at.dignity.term, "{luminary:?}");
    }
}
