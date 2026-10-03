//! The outer planets beside the nine (`03-design/western-outer-planets.md`):
//! asked for, placed in the same request and the chart's own frame, and a
//! chart that does not ask them is the chart it always was.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking"
)]

use teistro::catalogue::Graha;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{ChartRequest, Context, Document, Ephemeris, UtcOffset};

/// 1 January 2000, 12.00 UT at Kathmandu: a date the corpus brackets.
const INSTANT: f64 = 2_451_545.0;

fn context(profile: &str) -> Context {
    Context::builder()
        .profile(profile)
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
}

fn kathmandu() -> ChartRequest {
    ChartRequest::at(
        Place::new(
            Latitude::literal(27.7172),
            Longitude::literal(85.324),
            Altitude::literal(1400.0),
        ),
        UtcOffset::try_from_seconds(20_700).unwrap(),
    )
}

fn read(sdk: &Context, request: &ChartRequest) -> teistro::Envelope<Document> {
    sdk.chart()
        .reading(JulianDay::<Utc>::try_new(INSTANT).unwrap(), request)
        .unwrap()
}

#[test]
fn a_chart_that_does_not_ask_is_unchanged() {
    let sdk = context("conformance-baseline");
    let chart = read(&sdk, &kathmandu()).value;
    assert_eq!(chart.foundation.outer, Vec::new());
    let json = serde_json::to_value(&chart).unwrap();
    assert!(
        json["foundation"].get("outer").is_none(),
        "an unasked list is left out of the document"
    );
    assert!(chart.foundation.graha(Graha::Uranus).is_none());
}

#[test]
fn asking_adds_the_three_and_moves_none_of_the_nine() {
    for profile in ["conformance-baseline", "western-tropical-default"] {
        let sdk = context(profile);
        let bare = read(&sdk, &kathmandu());
        let asked = read(&sdk, &kathmandu().with_outer_planets());
        assert_eq!(
            asked.value.foundation.grahas, bare.value.foundation.grahas,
            "{profile}: the nine are the same to the bit"
        );
        let outer: Vec<Graha> = asked
            .value
            .foundation
            .outer
            .iter()
            .map(|at| at.graha)
            .collect();
        assert_eq!(outer, [Graha::Uranus, Graha::Neptune, Graha::Pluto]);
        for at in &asked.value.foundation.outer {
            // The chart's own zodiac and frame, as the nine.
            let tropical = asked.value.foundation.zodiac.of_tropical(at.tropical_deg);
            assert!((tropical - at.longitude_deg).abs() < 1e-9, "{profile}");
            assert!(at.distance_au > 15.0, "{profile}: {:?}", at.graha);
            assert!((1..=12).contains(&at.house.bhava), "{profile}");
        }
        assert_ne!(
            asked.provenance.input_hash, bare.provenance.input_hash,
            "{profile}: asking for more is a different question"
        );
    }
}
