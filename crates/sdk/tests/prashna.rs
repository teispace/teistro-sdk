//! Prashna through the façade: the chart of the question's moment read as
//! the kernel reads it, its Shadbala asked for, and the record a binding
//! sends (`docs/03-design/prashna.md`).

#![allow(
    clippy::unwrap_used,
    reason = "tests fail by panicking on what they asked for"
)]

use teistro::catalogue::Graha;
use teistro::prashna::{GRAHAS, MookRule, PrashnaRules, Question, ScoreRule};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{ChartRequest, Context, Document, Ephemeris, PrashnaRequest, UtcOffset};

/// A moment at Kathmandu, 14 April 1990, the corpus's first.
const MOMENT: f64 = 2_447_995.489_583_333_5;

fn sdk() -> Context {
    Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
}

fn chart(sdk: &Context, shadbala: bool) -> Document {
    let place = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );
    let request = ChartRequest::at(place, UtcOffset::literal(5, 45, 0));
    let request = if shadbala {
        request.with_shadbala()
    } else {
        request
    };
    sdk.chart()
        .reading(JulianDay::<Utc>::literal(MOMENT), &request)
        .unwrap()
        .value
}

#[test]
fn the_sky_is_the_charts_own() {
    let sdk = sdk();
    let document = chart(&sdk, true);
    let sky = sdk.chart().prashna_sky(&document).unwrap();
    assert_eq!(sky.lagna_deg, document.foundation.lagna_deg);
    for (placed, graha) in sky.grahas.iter().zip(GRAHAS) {
        let at = document.foundation.graha(graha).unwrap();
        assert_eq!(placed.longitude_deg, at.longitude_deg, "{graha:?}");
    }
    // The nodes move backwards, always.
    assert!(sky.grahas[7].retrograde && sky.grahas[8].retrograde);
    let shadbala = document.shadbala.as_ref().unwrap();
    let sun = shadbala
        .grahas
        .iter()
        .find(|one| one.graha == Graha::Sun)
        .unwrap();
    assert_eq!(sky.strength[0], sun.rupas);
}

#[test]
fn a_prashna_reads_the_chart_it_is_given() {
    let sdk = sdk();
    let document = chart(&sdk, true);
    let asked = PrashnaRequest::new(Question::about(7), PrashnaRules::default());
    let read = sdk.chart().prashna(&document, &asked).unwrap();
    assert!(read.links.is_some());
    assert_eq!(read.score, None);
    let baseline = PrashnaRequest::new(Question::about(7), PrashnaRules::baseline());
    let read = sdk.chart().prashna(&document, &baseline).unwrap();
    assert!(read.score.is_some());
    assert_eq!(read.mook.rule, MookRule::Baseline);
    assert_eq!(read.rules.score, ScoreRule::Baseline);
}

#[test]
fn a_refusal_names_what_to_ask_for() {
    let sdk = sdk();
    let bare = chart(&sdk, false);
    let asked = PrashnaRequest::default();
    let error = sdk.chart().prashna(&bare, &asked).unwrap_err();
    assert_eq!(error.field(), Some("shadbala"));
    let document = chart(&sdk, true);
    let thirteenth = PrashnaRequest::from_json(r#"{"question": {"house": 13}}"#).unwrap();
    let error = sdk.chart().prashna(&document, &thirteenth).unwrap_err();
    assert_eq!(error.field(), Some("prashna.question.house"));
    let error = PrashnaRequest::from_json(r#"{"question": {"number": 0}}"#)
        .and_then(|asked| sdk.chart().prashna(&document, &asked))
        .unwrap_err();
    assert_eq!(error.field(), Some("prashna.question.number"));
}
