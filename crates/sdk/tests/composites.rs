//! Composite and Davison charts (`03-design/western-composites.md`, C247,
//! C248), on the married pair whose births Leo gives in *How to Judge a
//! Nativity* (p. 130), King George V and Queen Mary, against a Moshier
//! recast of both (pyswisseph, rank 3).

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking"
)]

use teistro::catalogue::Graha;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{ChartRequest, Context, Document, Ephemeris, Partner, SynastryZodiac, UtcOffset};

/// "born 1-18 a.m., 3rd June, 1865, London", at Marlborough House.
const GEORGE: (f64, f64, f64) = (2_402_390.554_166_667, 51.5045, -0.1366);
/// "born 11-59 p.m., 26th May, 1867, London", at Kensington Palace.
const MARY: (f64, f64, f64) = (2_403_113.499_305_556, 51.5058, -0.1878);

/// The recast's composite: each planet at the near midpoint of its two
/// tropical places, and the mean of its two speeds.
const COMPOSITE: [(Graha, f64, f64); 10] = [
    (Graha::Sun, 68.8193, 0.958_16),
    (Graha::Moon, 259.7289, 12.213_17),
    (Graha::Mars, 130.5171, 0.559_74),
    (Graha::Mercury, 53.9101, 1.644_69),
    (Graha::Jupiter, 300.8516, -0.013_31),
    (Graha::Venus, 36.5246, 0.681_51),
    (Graha::Saturn, 216.881, -0.054_14),
    (Graha::Uranus, 92.561, 0.055_73),
    (Graha::Neptune, 12.2451, 0.023_82),
    (Graha::Pluto, 44.2555, 0.020_83),
];

/// The recast's composite Placidus cusps, first to twelfth, tropical.
const CUSPS: [f64; 12] = [
    334.007, 27.5378, 57.8601, 78.1797, 96.1916, 117.375, 154.007, 207.5378, 237.8601, 258.1797,
    276.1916, 297.375,
];

/// The recast cast at the Davison birth, tropical. Mars is where the two
/// methods part most: the composite's 130.5° is the midpoint of two
/// places, the Davison's 20.1° is where Mars stood.
const DAVISON: [(Graha, f64); 10] = [
    (Graha::Sun, 68.8173),
    (Graha::Moon, 259.5589),
    (Graha::Mars, 20.1267),
    (Graha::Mercury, 50.6662),
    (Graha::Jupiter, 302.1054),
    (Graha::Venus, 92.5841),
    (Graha::Saturn, 216.8687),
    (Graha::Uranus, 92.5368),
    (Graha::Neptune, 12.2514),
    (Graha::Pluto, 44.2566),
];

fn context(profile: Option<&str>) -> Context {
    let builder = Context::builder().ephemeris([Ephemeris::Builtin]);
    match profile {
        Some(profile) => builder.profile(profile),
        None => builder,
    }
    .build()
    .unwrap()
}

fn partner((jd, latitude, longitude): (f64, f64, f64)) -> Partner {
    Partner {
        instant: JulianDay::<Utc>::try_new(jd).unwrap(),
        place: Place::new(
            Latitude::literal(latitude),
            Longitude::literal(longitude),
            Altitude::literal(0.0),
        ),
        utc_offset: UtcOffset::UTC,
    }
}

fn born(sdk: &Context, birth: &Partner, outer: bool) -> Document {
    let request = ChartRequest::at(birth.place, birth.utc_offset);
    let request = if outer {
        request.with_outer_planets()
    } else {
        request
    };
    sdk.chart().reading(birth.instant, &request).unwrap().value
}

fn near(a: f64, b: f64) -> bool {
    ((a - b + 180.0).rem_euclid(360.0) - 180.0).abs() < 0.01
}

#[test]
fn the_composite_holds_the_recasts_midpoints() {
    // Founded sidereal and read tropical, as a Western reader reads it.
    let sdk = context(None);
    let george = born(&sdk, &partner(GEORGE), true);
    let mary = born(&sdk, &partner(MARY), true);
    let both = sdk
        .chart()
        .composite(&george, &mary, SynastryZodiac::Tropical)
        .unwrap();
    assert_eq!(both.planets.len(), COMPOSITE.len());
    for (graha, longitude, speed) in COMPOSITE {
        let at = both.planets.iter().find(|at| at.graha == graha).unwrap();
        assert!(near(at.longitude_deg, longitude), "{graha:?}: {at:?}");
        assert!(
            (at.speed_deg_per_day - speed).abs() < 1e-3,
            "{graha:?}: {at:?}"
        );
    }
    // His midheaven 270.80°, hers 245.56°; his lagna 2.10°, hers 305.91°,
    // whose near midpoint stands 75.8° after the midheaven: not turned.
    assert!(near(both.midheaven_deg, 258.1797), "{}", both.midheaven_deg);
    assert!(near(both.lagna_deg, 334.007), "{}", both.lagna_deg);
    // Each Placidus cusp at the near midpoint of the two, turned as the
    // lagna is; the first and tenth are the lagna and the midheaven.
    let cusps = both.cusps_deg.unwrap();
    for (cusp, recast) in cusps.iter().zip(CUSPS) {
        assert!(near(*cusp, recast), "{cusp} against {recast}");
    }
    assert_eq!((cusps[0], cusps[9]), (both.lagna_deg, both.midheaven_deg));
    // The composite Sun at 68.8° stands in the third house.
    assert_eq!(both.house_of(Graha::Sun).unwrap().get(), 3);
    assert!(!both.lagna_turned);
}

#[test]
fn the_charts_own_zodiac_moves_every_point_alike() {
    let sdk = context(None);
    let george = born(&sdk, &partner(GEORGE), true);
    let mary = born(&sdk, &partner(MARY), true);
    let tropical = sdk
        .chart()
        .composite(&george, &mary, SynastryZodiac::Tropical)
        .unwrap();
    let own = sdk
        .chart()
        .composite(&george, &mary, SynastryZodiac::Charts)
        .unwrap();
    // Two years apart, the two ayanamshas differ by under a minute, so
    // every point moves by their mean.
    let shift = (tropical.midheaven_deg - own.midheaven_deg).rem_euclid(360.0);
    assert!((20.0..23.0).contains(&shift), "{shift}");
    for (a, b) in tropical.planets.iter().zip(&own.planets) {
        let moved = (a.longitude_deg - b.longitude_deg).rem_euclid(360.0);
        assert!((moved - shift).abs() < 1e-3, "{a:?} {b:?}");
    }
    assert!(((tropical.lagna_deg - own.lagna_deg).rem_euclid(360.0) - shift).abs() < 1e-3);
}

#[test]
fn a_composite_refuses_what_does_not_pair() {
    let sdk = context(None);
    let george = born(&sdk, &partner(GEORGE), true);
    let mary = born(&sdk, &partner(MARY), false);
    let why = sdk
        .chart()
        .composite(&george, &mary, SynastryZodiac::Tropical)
        .unwrap_err();
    assert_eq!(why.field(), Some("first.planets[7]"), "{why}");
    let western = context(Some("western-tropical-default"));
    let tropical = born(&western, &partner(MARY), true);
    let why = sdk
        .chart()
        .composite(&george, &tropical, SynastryZodiac::Charts)
        .unwrap_err();
    assert_eq!(why.field(), Some("zodiac"), "{why}");
}

#[test]
fn the_davison_chart_is_the_recast_cast_at_the_midpoint_birth() {
    let sdk = context(Some("western-tropical-default"));
    let between = partner(GEORGE).davison(&partner(MARY)).unwrap();
    assert!((between.instant.get() - 2_402_752.026_736_111).abs() < 1e-8);
    assert!((between.place.latitude.get() - 51.505_15).abs() < 1e-9);
    assert!((between.place.longitude.get() + 0.1622).abs() < 1e-9);
    let chart = born(&sdk, &between, true);
    for (graha, longitude) in DAVISON {
        let at = chart.foundation.graha(graha).unwrap().longitude_deg;
        assert!(near(at, longitude), "{graha:?}: {at}");
    }
    assert!(
        near(chart.foundation.lagna_deg, 171.0178),
        "{}",
        chart.foundation.lagna_deg
    );
}

#[test]
fn a_davison_birth_takes_the_shorter_way_round_and_the_mean_clock() {
    let east = Partner {
        utc_offset: UtcOffset::try_from_seconds(12 * 3600).unwrap(),
        place: Place::try_from_degrees(-36.85, 174.76, 10.0).unwrap(),
        ..partner(GEORGE)
    };
    let west = Partner {
        utc_offset: UtcOffset::try_from_seconds(-10 * 3600).unwrap(),
        place: Place::try_from_degrees(21.31, -157.86, 20.0).unwrap(),
        ..partner(MARY)
    };
    let between = east.davison(&west).unwrap();
    // 174.76° east and 157.86° west are 27.38° apart across the date line.
    assert!((between.place.longitude.get() + 171.55).abs() < 1e-9);
    assert!((between.place.latitude.get() + 7.77).abs() < 1e-9);
    assert!((between.place.altitude.get() - 15.0).abs() < 1e-9);
    assert_eq!(between.utc_offset.seconds(), 3600);
    // The same birth, either way round.
    assert_eq!(west.davison(&east).unwrap(), between);
}
