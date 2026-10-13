//! Prashna through the façade: the chart of the question's moment read as
//! the kernel reads it, its Shadbala asked for, and the record a binding
//! sends (`docs/03-design/prashna.md`).

#![allow(
    clippy::unwrap_used,
    reason = "tests fail by panicking on what they asked for"
)]

use teistro::catalogue::{Graha, Rashi};
use teistro::prashna::{
    ClauseKind, GRAHAS, MookRule, PrashnaRules, Question, RisingVarga, ScoreRule,
};
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
    // The lagna's six vargas: the rasi is its sign, the hora a luminary's,
    // and the navāṁśa the ninth part counted round the zodiac.
    let [rasi, hora, _, navamsha, _, _] = sky.lagna_vargas;
    assert_eq!(rasi, sky.lagna());
    assert!(matches!(hora, Rashi::Leo | Rashi::Cancer), "{hora:?}");
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a ninth part of the zodiac, 0 to 107"
    )]
    let ninth = (sky.lagna_deg * 9.0 / 30.0).floor() as u16 % 12;
    assert_eq!(navamsha, Rashi::from_id(ninth).unwrap());
    assert_eq!(sky.lagna_navamsha(), navamsha);
}

#[test]
fn the_rising_varga_is_read_as_the_request_names_it() {
    let sdk = sdk();
    let document = chart(&sdk, true);
    let kinds = |json: &str| {
        let asked = PrashnaRequest::from_json(json).unwrap();
        let read = sdk.chart().prashna(&document, &asked).unwrap();
        read.verdict
            .clauses
            .iter()
            .map(|clause| clause.kind)
            .filter(|kind| {
                matches!(
                    kind,
                    ClauseKind::RisingNavamsha | ClauseKind::RisingShadvarga
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(kinds("{}"), [ClauseKind::RisingNavamsha]);
    assert_eq!(
        kinds(r#"{"rules": {"risingVarga": "SHADVARGA"}}"#),
        [ClauseKind::RisingShadvarga]
    );
    let error =
        PrashnaRequest::from_json(r#"{"rules": {"risingVarga": "SAPTAVARGA"}}"#).unwrap_err();
    assert_eq!(error.field(), Some("prashna.rules.risingVarga"));
    assert_eq!(
        PrashnaRequest::from_json(r#"{"rules": {"risingVarga": "SHADVARGA"}}"#)
            .unwrap()
            .rules
            .rising_varga,
        RisingVarga::Shadvarga
    );
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
