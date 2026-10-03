//! Equal distances (`03-design/western-midpoints.md`, C245, C246), on
//! King George V's birth (Leo, *How to Judge a Nativity*, p. 130), against
//! a Moshier recast of it (pyswisseph, rank 3).

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking"
)]

use teistro::catalogue::Graha;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{ChartRequest, Context, Document, Ephemeris, MidpointRequest, UtcOffset};

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

/// The recast's equal distances within 1.5° of the axis, closest first:
/// the pair in the catalogue's order, the planet between, whether on the
/// far point, its distance from the axis and from each of the two.
const RECAST: [(Graha, Graha, Graha, bool, f64, f64); 8] = [
    (
        Graha::Moon,
        Graha::Jupiter,
        Graha::Pluto,
        true,
        0.052,
        137.69,
    ),
    (
        Graha::Mercury,
        Graha::Venus,
        Graha::Pluto,
        false,
        0.642,
        4.42,
    ),
    (
        Graha::Mercury,
        Graha::Saturn,
        Graha::Mars,
        false,
        0.672,
        77.79,
    ),
    (
        Graha::Venus,
        Graha::Neptune,
        Graha::Saturn,
        true,
        0.836,
        165.26,
    ),
    (
        Graha::Uranus,
        Graha::Neptune,
        Graha::Mercury,
        false,
        0.886,
        39.21,
    ),
    (Graha::Sun, Graha::Moon, Graha::Mars, false, 1.149, 54.31),
    (
        Graha::Mars,
        Graha::Pluto,
        Graha::Jupiter,
        true,
        1.167,
        138.91,
    ),
    (
        Graha::Mars,
        Graha::Mercury,
        Graha::Jupiter,
        true,
        1.363,
        141.44,
    ),
];

#[test]
fn the_equal_distances_agree_with_the_recast() {
    let (sdk, chart) = george(Some("western-tropical-default"));
    let rows = sdk
        .chart()
        .midpoints(&chart, &MidpointRequest::default().with_orb_deg(1.5))
        .unwrap();
    assert_eq!(rows.len(), RECAST.len(), "{rows:?}");
    for (row, (first, second, middle, far, from_axis, distance)) in rows.iter().zip(RECAST) {
        assert_eq!(
            (row.first, row.second, row.middle, row.far),
            (first, second, middle, far)
        );
        assert!(
            (row.from_axis_deg - from_axis).abs() < 0.01,
            "{middle:?}: {} against {from_axis}",
            row.from_axis_deg
        );
        assert!((row.distance_deg - distance).abs() < 0.01, "{middle:?}");
        assert_eq!(row.orb_deg, 1.5);
    }
}

#[test]
fn the_default_holds_one() {
    // Under 0.5° only Pluto, on the far point of the Moon and Jupiter.
    let (sdk, chart) = george(Some("western-tropical-default"));
    let rows = sdk
        .chart()
        .midpoints(&chart, &MidpointRequest::default())
        .unwrap();
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(
        (rows[0].first, rows[0].second, rows[0].middle, rows[0].far),
        (Graha::Moon, Graha::Jupiter, Graha::Pluto, true)
    );
    assert_eq!(rows[0].orb_deg, 0.5);
}

#[test]
fn a_sidereal_chart_gives_the_same_rows() {
    let (tropical_sdk, tropical) = george(Some("western-tropical-default"));
    let (sidereal_sdk, sidereal) = george(None);
    let asked = MidpointRequest::default().with_orb_deg(1.5);
    let a = tropical_sdk.chart().midpoints(&tropical, &asked).unwrap();
    let b = sidereal_sdk.chart().midpoints(&sidereal, &asked).unwrap();
    assert_eq!(a.len(), b.len());
    for (a, b) in a.iter().zip(&b) {
        assert_eq!(
            (a.first, a.second, a.middle, a.far),
            (b.first, b.second, b.middle, b.far)
        );
        assert!((a.from_axis_deg - b.from_axis_deg).abs() < 1e-6);
        assert!((a.distance_deg - b.distance_deg).abs() < 1e-6);
    }
}

#[test]
fn a_refusal_names_its_field() {
    let (sdk, chart) = george(None);
    let why = sdk
        .chart()
        .midpoints(&chart, &MidpointRequest::default().with_orb_deg(11.0))
        .unwrap_err();
    assert_eq!(why.field(), Some("orbDeg"));
}
