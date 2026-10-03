//! A Western chart's houses (`03-design/western-houses.md`, C249, C250),
//! on Leo's own illustration in *How to Judge a Nativity* (p. 150), "a
//! female born at 2.42 A.M. 13th December, 1835, London", against a
//! Moshier recast (pyswisseph, rank 3) and the cusps Leo printed.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking"
)]

use teistro::catalogue::{Graha, HouseSystem};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{ChartRequest, Context, Document, Ephemeris, HouseRequest, UtcOffset};

/// 2.42 a.m. Greenwich, London's clock to the minute.
const BORN: f64 = 2_391_625.612_5;

/// The cusps Leo prints, tenth to third: ♋29½, ♍4, ♎1, ♎22, ♏18, ♐21.
const PRINTED: [(usize, f64); 6] = [
    (10, 119.5),
    (11, 154.0),
    (12, 181.0),
    (1, 202.0),
    (2, 228.0),
    (3, 261.0),
];

/// The recast's Placidus cusps, first to twelfth, tropical.
const RECAST: [f64; 12] = [
    202.1436, 228.5143, 261.3785, 299.3147, 333.8805, 1.1763, 22.1436, 48.5143, 81.3785, 119.3147,
    153.8805, 181.1763,
];

/// Each planet's house in the recast: the Sun at ♐20½ just short of the
/// third cusp, Mars at ♐22 just past it, Saturn rising.
const HOUSES: [(Graha, u8); 10] = [
    (Graha::Sun, 2),
    (Graha::Moon, 11),
    (Graha::Mercury, 2),
    (Graha::Venus, 3),
    (Graha::Mars, 3),
    (Graha::Jupiter, 9),
    (Graha::Saturn, 1),
    (Graha::Uranus, 4),
    (Graha::Neptune, 4),
    (Graha::Pluto, 6),
];

fn chart(profile: Option<&str>) -> (Context, Document) {
    let builder = Context::builder().ephemeris([Ephemeris::Builtin]);
    let sdk = match profile {
        Some(profile) => builder.profile(profile),
        None => builder,
    }
    .build()
    .unwrap();
    let london = Place::new(
        Latitude::literal(51.5),
        Longitude::literal(-0.1),
        Altitude::literal(0.0),
    );
    let request = ChartRequest::at(london, UtcOffset::UTC).with_outer_planets();
    let instant = JulianDay::<Utc>::try_new(BORN).unwrap();
    let chart = sdk.chart().reading(instant, &request).unwrap().value;
    (sdk, chart)
}

fn apart(a: f64, b: f64) -> f64 {
    ((a - b + 180.0).rem_euclid(360.0) - 180.0).abs()
}

#[test]
fn leos_figure_is_placidus() {
    let (sdk, chart) = chart(Some("western-tropical-default"));
    let houses = sdk
        .chart()
        .western_houses(&chart, &HouseRequest::default())
        .unwrap();
    assert_eq!(houses.system, HouseSystem::Placidus);
    for (cusp, recast) in houses.frame.cusps_deg.iter().zip(RECAST) {
        assert!(apart(*cusp, recast) < 0.01, "{cusp} against {recast}");
    }
    // Every printed cusp within the printing's half degree.
    for (house, printed) in PRINTED {
        let cusp = houses.frame.cusps_deg[house - 1];
        assert!(apart(cusp, printed) < 0.6, "house {house}: {cusp}");
    }
    // Regiomontanus, Lilly's division, misses the third by 3.8°.
    let regiomontanus = sdk
        .chart()
        .western_houses(
            &chart,
            &HouseRequest::default().with_system(HouseSystem::Regiomontanus),
        )
        .unwrap();
    assert_eq!(regiomontanus.system, HouseSystem::Regiomontanus);
    assert!(apart(regiomontanus.frame.cusps_deg[2], 261.0) > 3.5);
}

#[test]
fn each_planet_is_in_the_recasts_house_and_saturn_rises() {
    let (sdk, chart) = chart(Some("western-tropical-default"));
    let houses = sdk
        .chart()
        .western_houses(&chart, &HouseRequest::default())
        .unwrap();
    for (graha, house) in HOUSES {
        let placed = houses.planets.iter().find(|at| at.graha == graha).unwrap();
        assert_eq!(placed.house.get(), house, "{graha:?}");
        assert_eq!(placed.with_ascendant, graha == Graha::Saturn, "{graha:?}");
    }
    // The hour before raised ♎11.6, 10.5° of longitude above the
    // ascendant: Libra rises slowly at London.
    assert!(
        apart(houses.frame.reach_deg, 191.6089) < 0.01,
        "{}",
        houses.frame.reach_deg
    );
}

#[test]
fn a_sidereal_chart_counts_the_same_houses() {
    // The default profile founds the chart sidereal: every cusp and
    // planet moves by the ayanamsha together, so no house changes.
    let (tropical_sdk, tropical) = chart(Some("western-tropical-default"));
    let (sdk, sidereal) = chart(None);
    let asked = HouseRequest::default();
    let theirs = tropical_sdk
        .chart()
        .western_houses(&tropical, &asked)
        .unwrap();
    let ours = sdk.chart().western_houses(&sidereal, &asked).unwrap();
    assert_eq!(ours.system, HouseSystem::Placidus);
    assert_eq!(ours.planets, theirs.planets);
    let shift = apart(theirs.frame.cusps_deg[0], ours.frame.cusps_deg[0]);
    assert!(shift > 20.0 && shift < 24.0, "{shift}");
}
