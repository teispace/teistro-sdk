//! **A computed longitude against the recorded charts**: the comparison
//! Phase 4's exit waits on.
//!
//! Everything else that reads `fixtures/baseline/charts/*.json` reads the
//! recorded longitudes and measures a rule over them —
//! `chart/tests/baseline_bhavas.rs` checks which bhava a recorded
//! position falls in, which measures the placement and not the position.
//! Nothing computed a position from the instant and the place and
//! compared it, because that needs an ephemeris, and the SDK's own tests
//! run without one.
//!
//! This runs where the engine is. For each of the 55 charts it takes the
//! instant and the place the fixture records, asks the SDK for every
//! graha through the Teimeris adapter in the frame the fixture was
//! recorded in — sidereal Lahiri, apparent, of date, topocentric at the
//! place — and compares longitude, latitude, speed and distance against
//! what was recorded.
//!
//! What is asserted is a **bound**, and what is printed is the measured
//! agreement, so the design page carries the number rather than a claim.
//! Run by hand with the engine present.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::print_stdout,
    clippy::indexing_slicing,
    reason = "a test fails by panicking, reads its fixtures by key and reports its measurement"
)]

use std::collections::BTreeMap;
use std::path::Path;

use teistro_astro::{Completion, DeltaTModel};
use teistro_core::catalogue::Ayanamsha;
use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
use teistro_core::settings::OverridePolicy;
use teistro_ephemeris_teimeris::{TeimerisProvider, data_dir_from_env};
use teistro_port_ephemeris::{
    Body, Centre, Coordinates, Equinox, Frame, PositionRequest, TimeScale, Zodiac,
};

/// What one chart records of one graha.
#[derive(Clone, Copy, Debug)]
struct Recorded {
    sidereal_deg: f64,
    tropical_deg: f64,
    latitude_deg: f64,
    speed_deg_per_day: f64,
    distance_au: f64,
    /// The centre this one was recorded from.
    ///
    /// The chart is topocentric and the `outer` block says
    /// `"frame": "geocentric"` for each of its three, so they are
    /// recorded from a different centre than the grahas beside them.
    /// Comparing them topocentrically measures the parallax and calls it
    /// a disagreement — 0.44″ to 0.56″, which is 8.79″ over the distance
    /// in astronomical units, exactly what a parallax is. The fixture
    /// says which centre; this reads it rather than assuming.
    geocentric: bool,
}

/// One recorded chart, as much of it as a position needs.
struct Chart {
    id: String,
    jd_ut: f64,
    place: Place,
    topocentric: bool,
    /// The node the chart was cast with: the fixtures record `mean` or
    /// `true`, and which one Rahu *is* changes the answer by minutes.
    true_node: bool,
    bodies: BTreeMap<String, Recorded>,
}

/// Every recorded chart that names an ayanamsha this comparison can make.
///
/// A chart cast in another ayanamsha is not skipped quietly: the count
/// read and the count compared are both printed, and the assertion at the
/// end is over the count, so a fixture that stops being comparable shows
/// up as a failure rather than as silence.
fn charts() -> Vec<Chart> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../fixtures/baseline/charts");
    let mut charts = Vec::new();
    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .expect("the fixtures directory")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|e| e == "json"))
        .collect();
    paths.sort();
    for path in paths {
        let value: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).expect("readable")).expect("json");
        let settings = &value["settings"];
        if settings["ayanamsha"].as_str() != Some("lahiri") {
            continue;
        }
        let input = &value["input"];
        let place = &input["place"];
        let mut bodies = BTreeMap::new();
        let empty = serde_json::Map::new();
        let outer = value["positions"]["outer"].as_object().unwrap_or(&empty);
        for (key, body) in value["positions"]["bodies"]
            .as_object()
            .expect("the bodies")
            .iter()
            .chain(outer.iter())
        {
            let Some(sidereal) = body["sidereal_longitude_deg"].as_f64() else {
                continue;
            };
            bodies.insert(
                key.clone(),
                Recorded {
                    sidereal_deg: sidereal,
                    tropical_deg: body["tropical_longitude_deg"].as_f64().unwrap_or(f64::NAN),
                    latitude_deg: body["latitude_deg"].as_f64().unwrap_or(f64::NAN),
                    speed_deg_per_day: body["speed_deg_per_day"].as_f64().unwrap_or(f64::NAN),
                    distance_au: body["distance_au"].as_f64().unwrap_or(f64::NAN),
                    geocentric: body["frame"].as_str() == Some("geocentric"),
                },
            );
        }
        charts.push(Chart {
            id: value["id"].as_str().unwrap_or("?").to_string(),
            jd_ut: input["resolved"]["jd_ut"].as_f64().expect("the instant"),
            place: Place::new(
                Latitude::literal(place["latitude"].as_f64().expect("a latitude")),
                Longitude::literal(place["longitude"].as_f64().expect("a longitude")),
                Altitude::literal(place["altitude_m"].as_f64().unwrap_or(0.0)),
            ),
            topocentric: settings["topocentric"].as_bool().unwrap_or(false),
            true_node: settings["node"].as_str() == Some("true"),
            bodies,
        });
    }
    charts
}

/// The body a recorded key names, given which node the chart was cast
/// with. `KETU` is not here: no body is Ketu, it is Rahu's opposite
/// point, which the chart layer derives.
fn body_of(key: &str, true_node: bool) -> Option<Body> {
    if key == "RAHU" {
        return Some(if true_node {
            Body::TrueNode
        } else {
            Body::MeanNode
        });
    }
    Body::ALL
        .iter()
        .copied()
        .find(|body| body.graha().is_some_and(|graha| graha.key() == key))
        .filter(|body| !matches!(body, Body::MeanNode | Body::TrueNode))
}

/// The worst difference seen for one quantity, where, and **how many
/// times it looked**.
///
/// The count is not decoration. A quantity the fixtures do not record
/// comes back as `NaN`, which is skipped — and a worst of nought that
/// looked at nothing prints exactly like perfect agreement. The first
/// version of this harness reported `0.000000″` for the tropical
/// longitude and the distance and both were simply never compared. Every
/// count is printed and asserted.
#[derive(Default)]
struct Worst {
    value: f64,
    where_: String,
    seen: usize,
}

impl Worst {
    fn see(&mut self, difference: f64, chart: &str, key: &str) {
        if !difference.is_finite() {
            return;
        }
        self.seen += 1;
        if difference > self.value {
            self.value = difference;
            self.where_ = format!("{chart} {key}");
        }
    }

    /// The worst, in arcseconds, and where — or that it looked nowhere.
    fn report(&self, unit: &str) -> String {
        if self.seen == 0 {
            return String::from("nothing compared");
        }
        format!(
            "{:.6}{unit} at {} ({} compared)",
            self.value * 3600.0,
            if self.where_.is_empty() {
                "every one exact"
            } else {
                &self.where_
            },
            self.seen
        )
    }
}

/// The shortest way round between two longitudes, degrees.
fn apart(a: f64, b: f64) -> f64 {
    let mut d = (a - b) % 360.0;
    if d > 180.0 {
        d -= 360.0;
    } else if d < -180.0 {
        d += 360.0;
    }
    d.abs()
}

/// Every worst the comparison keeps.
#[derive(Default)]
struct Worsts {
    sidereal: Worst,
    tropical: Worst,
    latitude: Worst,
    speed: Worst,
    distance: Worst,
    per_body: BTreeMap<String, Worst>,
    compared: usize,
}

/// One chart's positions, computed and compared.
fn compare(chart: &Chart, completion: &Completion<'_, TeimerisProvider>, worsts: &mut Worsts) {
    // The grahas were recorded from the chart's own centre and the outer
    // planets from the geocentre, so they are two requests and not one.
    // Which centre each belongs to is read from the fixture, never
    // assumed.
    for from_geocentre in [false, true] {
        let wanted: Vec<(String, Body)> = chart
            .bodies
            .iter()
            .filter(|(_, recorded)| recorded.geocentric == from_geocentre)
            .filter_map(|(key, _)| body_of(key, chart.true_node).map(|body| (key.clone(), body)))
            .collect();
        if wanted.is_empty() {
            continue;
        }
        let bodies: Vec<Body> = wanted.iter().map(|(_, body)| *body).collect();
        let jds = [chart.jd_ut];
        let topocentric = chart.topocentric && !from_geocentre;

        let frame = Frame {
            centre: if topocentric {
                Centre::Topocentric
            } else {
                Centre::Geocentric
            },
            equinox: Equinox::OfDate,
            coordinates: Coordinates::Ecliptic,
            zodiac: Zodiac::Sidereal {
                ayanamsha: Ayanamsha::Lahiri,
            },
            ..Frame::CANONICAL
        };
        let mut request = PositionRequest::new(&jds, TimeScale::Ut1, &bodies, frame);
        if topocentric {
            request.observer = Some(chart.place);
        }
        let done = completion
            .positions(&request)
            .unwrap_or_else(|e| panic!("{}: {e}", chart.id));

        let mut tropical_request = request;
        tropical_request.frame = frame.with_zodiac(Zodiac::Tropical);
        let done_tropical = completion
            .positions(&tropical_request)
            .unwrap_or_else(|e| panic!("{}: {e}", chart.id));

        for (index, (key, body)) in wanted.iter().enumerate() {
            let recorded = chart.bodies[key];
            let cell = done.columns.at(0, index).expect("a cell");
            let cell_tropical = done_tropical.columns.at(0, index).expect("a cell");
            assert!(cell.is_ok(), "{} {key}: {:?}", chart.id, cell.status);

            let d = apart(cell.lon, recorded.sidereal_deg);
            worsts.sidereal.see(d, &chart.id, key);
            worsts
                .per_body
                .entry(key.clone())
                .or_default()
                .see(d, &chart.id, key);
            worsts.tropical.see(
                apart(cell_tropical.lon, recorded.tropical_deg),
                &chart.id,
                key,
            );
            worsts
                .latitude
                .see((cell.lat - recorded.latitude_deg).abs(), &chart.id, key);
            worsts.speed.see(
                (cell.lon_speed - recorded.speed_deg_per_day).abs(),
                &chart.id,
                key,
            );
            // A node is a direction, not a place: neither the engine nor
            // the baseline reports a distance for one.
            if body.has_distance() {
                worsts
                    .distance
                    .see((cell.dist - recorded.distance_au).abs(), &chart.id, key);
            }
            worsts.compared += 1;
        }
    }
}

#[test]
fn every_recorded_chart_reproduces_its_positions() {
    let provider = TeimerisProvider::open(&data_dir_from_env()).unwrap_or_else(|e| panic!("{e}"));
    let completion = Completion::new(
        &provider,
        OverridePolicy::PreferNative,
        DeltaTModel::TableThenModel,
    );
    let charts = charts();
    assert_eq!(charts.len(), 55, "every recorded chart is in Lahiri");

    let mut worsts = Worsts::default();
    for chart in &charts {
        compare(chart, &completion, &mut worsts);
    }
    let Worsts {
        sidereal,
        tropical,
        latitude,
        speed,
        distance,
        per_body,
        compared,
    } = worsts;

    println!("charts compared:    {}", charts.len());
    println!("positions compared: {compared}");
    println!("sidereal longitude: {}", sidereal.report("\u{2033}"));
    println!("tropical longitude: {}", tropical.report("\u{2033}"));
    println!("latitude:           {}", latitude.report("\u{2033}"));
    println!("speed:              {}", speed.report("\u{2033}/day"));
    println!("distance:           {}", distance.report(" (au x 3600)"));
    println!();
    println!("{:<10} {:>16}  where", "graha", "worst (arcsec)");
    for (key, worst) in &per_body {
        println!(
            "{key:<10} {:>16.6}  {}",
            worst.value * 3600.0,
            worst.report("")
        );
    }

    assert_eq!(compared, 605, "every graha of every chart");
    // A quantity that compared nothing is a harness that proves nothing,
    // and reads exactly like perfect agreement. Each count is asserted.
    assert_eq!(sidereal.seen, compared, "every sidereal longitude");
    assert_eq!(
        tropical.seen, 440,
        "the eight in the bodies block record a tropical longitude; the \
         three outer planets record only a sidereal one"
    );
    assert_eq!(latitude.seen, compared, "every latitude");
    assert_eq!(speed.seen, compared, "every speed");
    assert_eq!(
        distance.seen, 385,
        "the seven that are places: the outer block records no distance \
         and a node is a direction"
    );

    // The bounds are the measured agreement, published in the design
    // page; a change that moves them is a finding and not a nuisance.
    assert!(
        sidereal.value * 3600.0 < 0.001,
        "sidereal longitude {}",
        sidereal.report("\u{2033}")
    );
    assert!(
        tropical.value * 3600.0 < 0.001,
        "tropical longitude {}",
        tropical.report("\u{2033}")
    );
    assert!(
        latitude.value * 3600.0 < 0.001,
        "latitude {}",
        latitude.report("\u{2033}")
    );
    assert!(
        speed.value * 3600.0 < 0.001,
        "speed {}",
        speed.report("\u{2033}/day")
    );
    assert!(distance.value < 1e-9, "distance {}", distance.report(" au"));
}
