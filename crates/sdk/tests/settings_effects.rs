//! The frame knobs' effect notes, measured (SR-110,
//! `03-design/settings-and-profiles.md` §3.5): each note's number is
//! what the built-in ephemeris gives over 1900 to 2100, sampled every
//! 3.1 days (a step that walks through every hour of the day and every
//! phase of the Moon). The measurement takes a minute and a half in a
//! release build, so it is ignored by default and run as
//! `cargo test --release -p teistro --test settings_effects -- --ignored`;
//! what always runs is a coarse sample held under each note's bound.
#![cfg(feature = "builtin-ephemeris")]
#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking"
)]
#![allow(
    clippy::print_stdout,
    reason = "a measurement prints what it measured, for the note it backs"
)]

use teistro::{Context, Ephemeris};
use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
use teistro_port_ephemeris::{Body, Centre, Corrections, Frame, PositionRequest, TimeScale};

/// 1900 to 2100 every `step` days.
fn instants(step: f64) -> Vec<f64> {
    let (first, last) = (2_415_020.5, 2_488_069.5);
    core::iter::successors(Some(first), |jd| {
        Some(jd + step).filter(|next| *next < last)
    })
    .collect()
}

/// The measurement's step.
const FINE: f64 = 3.1;
/// The step of the sample that always runs.
const COARSE: f64 = 61.3;

fn context() -> Context {
    Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("the built-in ephemeris")
}

/// The longitudes of `bodies` at every instant, in `frame`.
fn longitudes(
    sdk: &Context,
    jds: &[f64],
    bodies: &[Body],
    frame: Frame,
    at: Option<Place>,
) -> Vec<f64> {
    let mut request = PositionRequest::new(jds, TimeScale::Ut1, bodies, frame);
    request.observer = at;
    sdk.positions(&request)
        .expect("the built-in sky")
        .value
        .columns
        .lon
}

/// The largest separation in longitude, in degrees, between two grids.
fn largest(a: &[f64], b: &[f64], body: usize, bodies: usize) -> f64 {
    a.iter()
        .zip(b)
        .enumerate()
        .filter(|(i, _)| i % bodies == body)
        .map(|(_, (x, y))| {
            let d = (x - y).rem_euclid(360.0);
            d.min(360.0 - d)
        })
        .fold(0.0, f64::max)
}

/// The true node's distance from the mean one, arcminutes.
fn node(step: f64) -> f64 {
    let sdk = context();
    let jds = instants(step);
    let both = longitudes(
        &sdk,
        &jds,
        &[Body::MeanNode, Body::TrueNode],
        Frame::CANONICAL,
        None,
    );
    let mean: Vec<f64> = both.iter().step_by(2).copied().collect();
    let true_node: Vec<f64> = both.iter().skip(1).step_by(2).copied().collect();
    largest(&mean, &true_node, 0, 1) * 60.0
}

/// `frame.node`: the true node stands up to 1°58′ from the mean one
/// (measured 117.96′).
#[test]
#[ignore = "a measurement: a minute in a release build"]
fn the_true_node_stands_up_to_two_degrees_from_the_mean() {
    let arcmin = node(FINE);
    println!("frame.node: {arcmin:.2}′");
    assert!(
        (117.0..=118.0).contains(&arcmin),
        "{arcmin}′, the note says up to 1°58′"
    );
}

/// `frame.positions`: apparent against true moves the Sun by 21″, the
/// Moon by under 1″ and a planet by up to 61″, Mercury (measured 20.9,
/// 0.8, 60.7, 49.3, 41.6, 33.4 and 28.8″ for the Sun to Saturn).
#[test]
#[ignore = "a measurement: a minute in a release build"]
fn apparent_positions_stand_seconds_from_true_ones() {
    let sdk = context();
    let jds = instants(FINE);
    let bodies = [
        Body::Sun,
        Body::Moon,
        Body::Mercury,
        Body::Venus,
        Body::Mars,
        Body::Jupiter,
        Body::Saturn,
    ];
    let apparent = longitudes(&sdk, &jds, &bodies, Frame::CANONICAL, None);
    let geometric = Frame {
        corrections: Corrections {
            light_time: false,
            aberration: false,
            deflection: false,
            nutation: true,
        },
        ..Frame::CANONICAL
    };
    let true_places = longitudes(&sdk, &jds, &bodies, geometric, None);
    let arcsec: Vec<f64> = (0..bodies.len())
        .map(|body| largest(&apparent, &true_places, body, bodies.len()) * 3600.0)
        .collect();
    println!("frame.positions: {bodies:?} {arcsec:.1?}″");
    assert!((20.5..=21.0).contains(&arcsec[0]), "the Sun {}″", arcsec[0]);
    assert!(arcsec[1] < 1.0, "the Moon {}″", arcsec[1]);
    let worst = arcsec.iter().copied().fold(0.0, f64::max);
    assert!(
        (60.0..=61.0).contains(&worst),
        "the worst {worst}″, the note says up to 61″"
    );
}

/// The Moon from Kathmandu against the Earth's centre, arcminutes.
fn centre(step: f64) -> f64 {
    let sdk = context();
    let jds = instants(step);
    let kathmandu = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );
    let geocentric = longitudes(&sdk, &jds, &[Body::Moon], Frame::CANONICAL, None);
    let topocentric = longitudes(
        &sdk,
        &jds,
        &[Body::Moon],
        Frame::CANONICAL.with_centre(Centre::Topocentric),
        Some(kathmandu),
    );
    largest(&geocentric, &topocentric, 0, 1) * 60.0
}

/// `frame.centre`: from Kathmandu the Moon stands up to 61′ from where
/// the Earth's centre sees it (measured 61.42′).
#[test]
#[ignore = "a measurement: a minute in a release build"]
fn the_place_moves_the_moon_by_up_to_a_degree() {
    let arcmin = centre(FINE);
    println!("frame.centre: {arcmin:.2}′");
    assert!(
        (61.0..=62.0).contains(&arcmin),
        "{arcmin}′, the note says up to 61′"
    );
}

/// What always runs: a coarse sample never passes a note's bound, and
/// comes near it.
#[test]
fn a_coarse_sample_stays_inside_each_note() {
    let node = node(COARSE);
    assert!((90.0..=118.0).contains(&node), "frame.node {node}′");
    let centre = centre(COARSE);
    assert!((40.0..=62.0).contains(&centre), "frame.centre {centre}′");
}
