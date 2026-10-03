//! Declinations and parallels (`03-design/western-declinations.md`), on
//! King George V's birth (Leo, *How to Judge a Nativity*, p. 130), against
//! a Moshier recast of it (pyswisseph, rank 3).

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking"
)]

use teistro::catalogue::Graha;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{ChartRequest, Context, Document, Ephemeris, ParallelRequest, UtcOffset};

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

/// The recast's declinations, degrees: each planet as the ephemeris turns
/// it to the equator, and each angle as the Sun's at that degree.
const RECAST: [(Graha, f64); 10] = [
    (Graha::Sun, 22.2997),
    (Graha::Moon, -2.6727),
    (Graha::Mercury, 14.1694),
    (Graha::Venus, 13.2803),
    (Graha::Mars, 20.2751),
    (Graha::Jupiter, -22.9401),
    (Graha::Saturn, -6.8533),
    (Graha::Uranus, 23.6505),
    (Graha::Neptune, 2.6495),
    (Graha::Pluto, 1.1235),
];

#[test]
fn every_declination_agrees_with_the_recast() {
    let (sdk, chart) = george(Some("western-tropical-default"));
    let read = sdk.chart().declinations(&chart).unwrap();
    assert_eq!(read.grahas.len(), RECAST.len(), "the nodes are not read");
    for (graha, recast) in RECAST {
        let ours = read.graha(graha).unwrap();
        assert!(
            (ours - recast).abs() < 0.01,
            "{graha:?}: {ours} against {recast}"
        );
    }
    assert!((read.lagna_deg - 0.8366).abs() < 0.01, "{}", read.lagna_deg);
    assert!(
        (read.midheaven_deg - -23.452).abs() < 0.01,
        "{}",
        read.midheaven_deg
    );
    assert!((read.obliquity_deg - 23.4544).abs() < 0.001);
}

#[test]
fn the_parallels_hold_on_either_side_of_the_equator() {
    let (sdk, chart) = george(Some("western-tropical-default"));
    let rows = sdk
        .chart()
        .parallels(&chart, &ParallelRequest::default())
        .unwrap();
    let found: Vec<(Graha, Graha, bool)> = rows
        .iter()
        .map(|row| (row.first, row.second, row.contrary))
        .collect();
    assert_eq!(
        found,
        [
            (Graha::Moon, Graha::Neptune, true),
            (Graha::Sun, Graha::Jupiter, true),
            (Graha::Jupiter, Graha::Uranus, true),
            (Graha::Mercury, Graha::Venus, false),
        ]
    );
    let wider = sdk
        .chart()
        .parallels(&chart, &ParallelRequest::default().with_orb_deg(1.5))
        .unwrap();
    assert!(wider.len() > rows.len());
    let refused = sdk
        .chart()
        .parallels(&chart, &ParallelRequest::default().with_orb_deg(0.0))
        .unwrap_err();
    assert_eq!(refused.field(), Some("orbDeg"));
}

#[test]
fn a_declination_does_not_depend_on_the_zodiac() {
    let (tropical_sdk, tropical) = george(Some("western-tropical-default"));
    let (sidereal_sdk, sidereal) = george(None);
    assert!(sidereal.foundation.zodiac.is_sidereal());
    let a = tropical_sdk.chart().declinations(&tropical).unwrap();
    let b = sidereal_sdk.chart().declinations(&sidereal).unwrap();
    for (one, other) in a.grahas.iter().zip(&b.grahas) {
        assert_eq!(one.graha, other.graha);
        assert!(
            (one.declination_deg - other.declination_deg).abs() < 1e-6,
            "{one:?} {other:?}"
        );
    }
}
