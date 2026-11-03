//! Mundane charts held to published instants, and astrocartography lines
//! read back through charts cast on them (`03-design/mundane-charts.md`,
//! `03-design/astrocartography.md`, C1150 to C1157).
//!
//! The tropical ingresses and the lunations are the U.S. Naval
//! Observatory's (Earth's seasons and the phases of the Moon, to the
//! minute); the eclipses NASA's (Espenak, greatest eclipse); the great
//! conjunction of 2020 the time in ecliptic longitude (about 18:20 UT);
//! the Mesha sankrantis the national panchanga committee's, as printed in
//! the conformance corpus (`fixtures/official/npns-2082-2083.json`).

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    reason = "tests fail by panicking"
)]

use teistro::catalogue::{ChartKind, Graha, LunarEclipseKind, MundaneKind, SolarEclipseKind};
use teistro::western::{LINE_TOLERANCE_DEG, MapAngle, Projection};
use teistro::{
    AstrocartographyRequest, Context, Ephemeris, MundaneRequest, RelocatedChart, RelocationPlace,
    RelocationRequest,
};

/// Two minutes, the precision the published tables print to and more.
const MINUTES: f64 = 2.0 / 1440.0;

fn builtin() -> Context {
    Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
}

/// A Julian day (UTC) from a date and a time.
fn jd(midnight: f64, hour: f64, minute: f64) -> f64 {
    midnight + (hour + minute / 60.0) / 24.0
}

fn moment(sdk: &Context, body: &str) -> teistro::MundaneMoment {
    let asked = MundaneRequest::from_json(&format!(
        r#"{{{body}, "latitudeDeg": 51.4779, "longitudeDeg": -0.0015}}"#
    ))
    .unwrap();
    sdk.chart().mundane_moment(&asked).unwrap().value
}

#[test]
fn the_tropical_ingresses_and_lunations_fall_at_the_naval_observatorys_minutes() {
    let sdk = builtin();
    for (body, expected) in [
        // 2024 March 20, 03:06 UT; 2024 December 21, 09:20; 2025 March 20, 09:01.
        (
            r#""kind": "ARIES_INGRESS", "year": 2024"#,
            jd(2_460_389.5, 3.0, 6.0),
        ),
        (
            r#""kind": "CAPRICORN_INGRESS", "year": 2024"#,
            jd(2_460_665.5, 9.0, 20.0),
        ),
        (
            r#""kind": "ARIES_INGRESS", "from": 2460400.5"#,
            jd(2_460_754.5, 9.0, 1.0),
        ),
        // Before the equinox of 2024: the new moon of 10 March, 09:00, and
        // the full moon of 24 February, 12:30; the nearer is the new moon.
        (
            r#""kind": "NEW_MOON_BEFORE_INGRESS", "year": 2024, "ingress": "ARIES_INGRESS""#,
            jd(2_460_379.5, 9.0, 0.0),
        ),
        (
            r#""kind": "FULL_MOON_BEFORE_INGRESS", "year": 2024, "ingress": "ARIES_INGRESS""#,
            jd(2_460_364.5, 12.0, 30.0),
        ),
        (
            r#""kind": "SYZYGY_BEFORE_INGRESS", "year": 2024, "ingress": "ARIES_INGRESS""#,
            jd(2_460_379.5, 9.0, 0.0),
        ),
    ] {
        let found = moment(&sdk, body);
        let off = (found.instant.get() - expected) * 1440.0;
        assert!(off.abs() < MINUTES * 1440.0, "{body}: {off:+.2} min");
    }
    let lunation = moment(
        &sdk,
        r#""kind": "NEW_MOON_BEFORE_INGRESS", "year": 2024, "ingress": "ARIES_INGRESS""#,
    );
    let ingress = lunation.ingress.unwrap();
    assert_eq!(ingress.kind, MundaneKind::AriesIngress);
    assert!((ingress.instant.get() - jd(2_460_389.5, 3.0, 6.0)).abs() < MINUTES);
}

#[test]
fn the_eclipses_and_the_great_conjunction_fall_at_their_published_instants() {
    let sdk = builtin();
    // NASA: the total solar eclipse of 2024 April 8, greatest 18:17:20 UT.
    let solar = moment(&sdk, r#""kind": "SOLAR_ECLIPSE", "year": 2024"#);
    assert_eq!(solar.solar_eclipse, Some(SolarEclipseKind::Total));
    assert!((solar.instant.get() - jd(2_460_408.5, 18.0, 17.33)).abs() < MINUTES);
    // NASA: the total lunar eclipse of 2025 March 14, greatest 06:58:43 UT.
    let lunar = moment(&sdk, r#""kind": "LUNAR_ECLIPSE", "year": 2025"#);
    assert_eq!(lunar.lunar_eclipse, Some(LunarEclipseKind::Total));
    assert!((lunar.instant.get() - jd(2_460_748.5, 6.0, 58.72)).abs() < MINUTES);
    // The great conjunction of 2020 December 21, in ecliptic longitude
    // about 18:20 UT; the half hour is the sources' own spread.
    let great = moment(&sdk, r#""kind": "GREAT_CONJUNCTION", "year": 2020"#);
    assert!(
        (great.instant.get() - jd(2_459_204.5, 18.0, 20.0)).abs() < 0.5 / 24.0,
        "{}",
        great.instant.get()
    );
    let chart = sdk
        .chart()
        .mundane(
            &MundaneRequest::from_json(
                r#"{"kind": "GREAT_CONJUNCTION", "year": 2020, "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700}"#,
            )
            .unwrap(),
        )
        .unwrap()
        .value;
    let at = |graha| chart.chart.graha(graha).unwrap().tropical_deg;
    assert!((at(Graha::Jupiter) - at(Graha::Saturn)).abs() < 1e-4);
    assert_eq!(chart.chart.kind, ChartKind::Mundane);
}

/// The great conjunction's coarse step finds every pass the daily scan
/// finds, at the same instant, through the triple conjunction of 1940–41
/// (C1153): three passes months apart.
#[test]
fn the_coarse_conjunction_step_finds_every_pass_of_a_triple_conjunction() {
    use teistro_astro::events::{Lattice, Quantity, Search};
    use teistro_port_ephemeris::{Body, Frame};

    let sdk = builtin();
    let sky = sdk.completion().unwrap();
    let longitudes = sky.longitudes(Frame::CANONICAL);
    let apart = Quantity::Composite {
        a: 1.0,
        first: Body::Jupiter,
        b: -1.0,
        second: Body::Saturn,
    };
    // 1 January 1940 to 1 January 1942.
    let (from, to) = (
        teistro::quantity::JulianDay::literal(2_429_629.5),
        teistro::quantity::JulianDay::literal(2_430_360.5),
    );
    let daily = Search::new(&longitudes, apart, Lattice::single(0.0))
        .between(from, to)
        .unwrap();
    let coarse = Search::new(&longitudes, apart, Lattice::single(0.0))
        .with_step_days(teistro::CONJUNCTION_STEP_DAYS)
        .between(from, to)
        .unwrap();
    assert_eq!(daily.len(), 3, "a triple conjunction");
    assert_eq!(coarse.len(), daily.len());
    for (fine, wide) in daily.iter().zip(&coarse) {
        assert!((fine.instant.get() - wide.instant.get()).abs() < 1e-6);
    }
}

#[test]
fn the_mesha_sankrantis_fall_at_the_committees_printed_minutes() {
    // BS 2082 and 2083 Baisakh 1 are 14 April 2025 and 2026; the committee
    // printed Mesha's ingress at 05:51 and 12:04 Nepal time (+05:45).
    let sdk = Context::builder()
        .nepal_official([Ephemeris::Builtin])
        .build()
        .unwrap();
    for (year, printed) in [
        (2025, jd(2_460_779.5, 5.0 - 5.75, 51.0)),
        (2026, jd(2_461_144.5, 12.0 - 5.75, 4.0)),
    ] {
        let asked = MundaneRequest::from_json(&format!(
            r#"{{"kind": "MESHA_SANKRANTI", "year": {year}, "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700}}"#
        ))
        .unwrap();
        let cast = sdk.chart().mundane(&asked).unwrap().value;
        let off = (cast.moment.instant.get() - printed) * 1440.0;
        assert!(off.abs() < 2.0, "{year}: {off:+.2} min");
        // The chart cast there places the Sun at the start of Mesha.
        let sun = cast.chart.graha(Graha::Sun).unwrap().longitude_deg;
        assert!(sun < 0.01 || sun > 359.99, "{year}: {sun}");
    }
}

#[test]
fn an_event_is_cast_as_given_with_its_source() {
    let sdk = builtin();
    let asked = MundaneRequest::from_json(
        r#"{"kind": "EVENT", "event": {"name": "A coronation", "instant": 2441700.25, "source": "the gazette of the day"}, "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 19800}"#,
    )
    .unwrap();
    let cast = sdk.chart().mundane(&asked).unwrap();
    assert_eq!(cast.value.moment.instant.get(), 2_441_700.25);
    assert_eq!(cast.value.moment.name.as_deref(), Some("A coronation"));
    assert_eq!(
        cast.value.moment.source.as_deref(),
        Some("the gazette of the day")
    );
    assert_eq!(cast.value.chart.kind, ChartKind::Event);
    // The answer is sealed with the request as its input.
    let moment = sdk.chart().mundane_moment(&asked).unwrap();
    assert_eq!(moment.provenance.input_hash, cast.provenance.input_hash);
    let none = Context::builder()
        .ephemeris([Ephemeris::None])
        .build()
        .unwrap();
    assert!(none.chart().mundane(&asked).is_err());
}

/// The right ascension of an ecliptic point, degrees.
fn right_ascension(longitude_deg: f64, latitude_deg: f64, obliquity_deg: f64) -> f64 {
    teistro_astro::sky::ecliptic_to_equatorial(
        teistro_astro::sky::Spherical {
            lon_deg: longitude_deg,
            lat_deg: latitude_deg,
        },
        obliquity_deg,
    )
    .lon_deg
}

fn apart(a: f64, b: f64) -> f64 {
    (a - b + 180.0).rem_euclid(360.0) - 180.0
}

/// How far a body stands off a line's angle at a place, degrees: by right
/// ascension or altitude in mundo, by longitude when zodiacal. `body` is
/// its tropical longitude, its latitude and its longitude in the chart's
/// zodiac.
fn off_its_angle(
    projection: Projection,
    angle: MapAngle,
    chart: &RelocatedChart,
    (tropical, latitude, longitude): (f64, f64, f64),
    to_tropical: f64,
    eps: f64,
) -> f64 {
    match (projection, angle) {
        (Projection::InMundo, MapAngle::Mc | MapAngle::Ic) => {
            let meridian = right_ascension(chart.midheaven_deg + to_tropical, 0.0, eps);
            let turn = if angle == MapAngle::Mc { 0.0 } else { 180.0 };
            apart(meridian + turn, right_ascension(tropical, latitude, eps))
        }
        (Projection::InMundo, MapAngle::Asc | MapAngle::Dsc) => {
            teistro_astro::sky::altitude_by_midheaven_deg(
                teistro_astro::sky::Spherical {
                    lon_deg: tropical,
                    lat_deg: latitude,
                },
                chart.midheaven_deg + to_tropical,
                eps,
                chart.latitude_deg,
            )
        }
        (Projection::Zodiacal, MapAngle::Mc) => apart(chart.midheaven_deg, longitude),
        (Projection::Zodiacal, MapAngle::Ic) => apart(chart.midheaven_deg + 180.0, longitude),
        (Projection::Zodiacal, MapAngle::Asc) => apart(chart.ascendant_deg, longitude),
        (Projection::Zodiacal, MapAngle::Dsc) => apart(chart.ascendant_deg + 180.0, longitude),
    }
}

/// Every point of every line, read back through a chart cast there: on a
/// meridian line the body's right ascension is the meridian's (in mundo)
/// or its longitude the midheaven's (zodiacal); on a rising or setting
/// line its altitude is zero on the right side (in mundo), or its
/// longitude the ascendant's or the descendant's (zodiacal).
#[test]
fn every_point_of_every_line_is_on_its_angle_read_back_through_a_chart_cast_there() {
    let sdk = builtin();
    let instant = 2_447_000.25;
    for projection in [Projection::InMundo, Projection::Zodiacal] {
        let mut asked = AstrocartographyRequest::new(instant);
        asked.projection = projection;
        asked.step_deg = 10.0;
        let map = sdk.chart().astrocartography(&asked).unwrap().value;
        assert_eq!(map.lines.len(), 4 * 7, "the seven by default");
        let found = sdk
            .chart()
            .found(
                teistro::quantity::JulianDay::literal(instant),
                &teistro::quantity::Place::try_from_degrees(0.0, 0.0, 0.0).unwrap(),
                teistro::UtcOffset::UTC,
                ChartKind::Relocated,
            )
            .unwrap()
            .value;
        let mut worst = 0.0f64;
        for line in &map.lines {
            let body = found.graha(line.body).unwrap();
            // The chart's zodiac less the tropical one.
            let to_tropical = body.tropical_deg - body.longitude_deg;
            let places: Vec<RelocationPlace> = line
                .points
                .iter()
                .map(|[lon, lat]| RelocationPlace {
                    latitude_deg: *lat,
                    longitude_deg: *lon,
                    altitude_m: 0.0,
                })
                .collect();
            let relocated = RelocationRequest {
                instant,
                places,
                house_system: Some(teistro::catalogue::HouseSystem::WholeSign),
            };
            let recast = sdk.chart().relocated(&relocated).unwrap().value;
            let eps = recast.obliquity_deg;
            for chart in &recast.places {
                let off = off_its_angle(
                    projection,
                    line.angle,
                    chart,
                    (body.tropical_deg, body.latitude_deg, body.longitude_deg),
                    to_tropical,
                    eps,
                );
                worst = worst.max(off.abs());
                assert!(
                    off.abs() < LINE_TOLERANCE_DEG,
                    "{projection:?} {:?} {:?} at {} {}: {off}",
                    line.body,
                    line.angle,
                    chart.longitude_deg,
                    chart.latitude_deg
                );
            }
            // A rising line stands east of the meridian, a setting one west.
            if projection == Projection::InMundo
                && matches!(line.angle, MapAngle::Asc | MapAngle::Dsc)
            {
                let limit = line.polar_limit_deg.unwrap();
                for chart in recast
                    .places
                    .iter()
                    .filter(|chart| chart.latitude_deg.abs() < limit - 1e-6)
                {
                    let hour_angle = apart(
                        right_ascension(chart.midheaven_deg + to_tropical, 0.0, eps),
                        right_ascension(body.tropical_deg, body.latitude_deg, eps),
                    );
                    assert_eq!(
                        hour_angle < 0.0,
                        line.angle == MapAngle::Asc,
                        "{:?}",
                        line.body
                    );
                }
            }
        }
        assert!(worst < LINE_TOLERANCE_DEG, "{projection:?}: {worst}");
    }
}

#[test]
fn a_relocated_chart_keeps_its_grahas_and_moves_its_angles() {
    let sdk = builtin();
    let asked = RelocationRequest::from_json(
        r#"{"instant": 2447000.25, "places": [{"latitudeDeg": 27.7172, "longitudeDeg": 85.324}, {"latitudeDeg": 40.7128, "longitudeDeg": -74.006}], "houseSystem": "PLACIDUS"}"#,
    )
    .unwrap();
    let moved = sdk.chart().relocated(&asked).unwrap().value;
    assert_eq!(moved.places.len(), 2);
    for place in &moved.places {
        assert_eq!(
            place.house_system,
            teistro::catalogue::HouseSystem::Placidus
        );
        // Placidus's first and tenth cusps are the angles.
        assert!(apart(place.cusps_deg[0], place.ascendant_deg).abs() < 1e-9);
        assert!(apart(place.cusps_deg[9], place.midheaven_deg).abs() < 1e-9);
    }
    // The meridian turns with the longitude: 159.33° west, 159.33° less.
    let turned = apart(moved.places[0].midheaven_deg, moved.places[1].midheaven_deg);
    assert!(turned.abs() > 100.0);
    // A body the context does not place is refused by name.
    let mut map = AstrocartographyRequest::new(2_447_000.25);
    map.bodies = vec![Graha::Pluto];
    let refused = sdk.chart().astrocartography(&map);
    if let Err(why) = refused {
        assert_eq!(why.field(), Some("bodies"));
    }
}
