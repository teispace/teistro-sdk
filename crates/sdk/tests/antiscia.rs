//! Antiscia (`03-design/western-antiscia.md`, C244), on King George V's
//! birth (Leo, *How to Judge a Nativity*, p. 130), against a Moshier
//! recast of it (pyswisseph, rank 3).

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking"
)]

use teistro::catalogue::Graha;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{AntisciaRequest, ChartRequest, Context, Document, Ephemeris, OrbModel, UtcOffset};

/// "born 1-18 a.m., 3rd June, 1865, London", at Marlborough House.
const GEORGE: (f64, f64, f64) = (2_402_390.554_166_667, 51.5045, -0.1366);

fn george(profile: Option<&str>) -> (Context, Document) {
    let builder = Context::builder().ephemeris([Ephemeris::Builtin]);
    let sdk = match profile {
        Some(profile) => builder.profile(profile),
        None => builder,
    }
    .build()
    .unwrap();
    let (jd, latitude, longitude) = GEORGE;
    let place = Place::new(
        Latitude::literal(latitude),
        Longitude::literal(longitude),
        Altitude::literal(0.0),
    );
    let chart = sdk
        .chart()
        .reading(
            JulianDay::<Utc>::try_new(jd).unwrap(),
            &ChartRequest::at(place, UtcOffset::UTC).with_outer_planets(),
        )
        .unwrap()
        .value;
    (sdk, chart)
}

/// The recast's antiscions, tropical degrees: 180° less each planet's
/// longitude.
const RECAST: [(Graha, f64); 7] = [
    (Graha::Sun, 107.5685),
    (Graha::Moon, 358.9488),
    (Graha::Mercury, 131.5274),
    (Graha::Venus, 140.3642),
    (Graha::Mars, 54.4075),
    (Graha::Jupiter, 274.3301),
    (Graha::Saturn, 335.9445),
];

#[test]
fn the_antiscia_agree_with_the_recast() {
    let (sdk, chart) = george(Some("western-tropical-default"));
    let read = sdk
        .chart()
        .antiscia(&chart, &AntisciaRequest::default())
        .unwrap();
    for (graha, recast) in RECAST {
        let point = read.points.iter().find(|one| one.graha == graha).unwrap();
        assert!(
            (point.antiscion_deg - recast).abs() < 0.01,
            "{graha:?}: {} against {recast}",
            point.antiscion_deg
        );
        let opposite = (point.contrantiscion_deg - point.antiscion_deg).rem_euclid(360.0);
        assert!((opposite - 180.0).abs() < 1e-9, "{graha:?}");
    }
    // Under the moieties the recast holds one pair, in the catalogue's
    // order: Mars's antiscion 5.94° from Mercury, inside their 7¼°.
    assert_eq!(read.pairs.len(), 1, "{:?}", read.pairs);
    let pair = read.pairs[0];
    assert_eq!(
        (pair.first, pair.second, pair.contrary),
        (Graha::Mars, Graha::Mercury, false)
    );
    assert!((pair.apart_deg - 5.935).abs() < 0.02, "{}", pair.apart_deg);
    assert!((pair.orb_deg - 7.25).abs() < 1e-12);
    assert_eq!(read.unpaired, [Graha::Uranus, Graha::Neptune, Graha::Pluto]);
}

#[test]
fn a_sidereal_chart_gives_the_same_antiscia() {
    let (tropical_sdk, tropical) = george(Some("western-tropical-default"));
    let (sidereal_sdk, sidereal) = george(None);
    let request = AntisciaRequest::default().with_orbs(OrbModel::Leo);
    let one = tropical_sdk.chart().antiscia(&tropical, &request).unwrap();
    let other = sidereal_sdk.chart().antiscia(&sidereal, &request).unwrap();
    assert!(
        other.unpaired.is_empty(),
        "Leo's orbs give every planet one"
    );
    assert_eq!(one.points.len(), other.points.len());
    for (a, b) in one.points.iter().zip(&other.points) {
        assert_eq!(a.graha, b.graha);
        assert!(
            (a.antiscion_deg - b.antiscion_deg).abs() < 1e-6,
            "{:?}",
            a.graha
        );
    }
    let pairs = |read: &teistro::Antiscia| -> Vec<_> {
        read.pairs
            .iter()
            .map(|row| (row.first, row.second, row.contrary))
            .collect()
    };
    assert_eq!(pairs(&one), pairs(&other));
}

#[test]
fn a_refusal_names_its_field() {
    let refused =
        AntisciaRequest::from_json(r#"{"orbs": {"model": "MOIETIES", "orbs": []}, "x": 1}"#)
            .unwrap_err();
    assert_eq!(refused.field(), Some("antiscia.x"));
}
