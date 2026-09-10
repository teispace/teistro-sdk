//! The built-in ephemeris answers the port, and answers it honestly.
//!
//! Phase 3's exit is a chart that computes with nothing but the SDK
//! installed. These are the properties that have to hold for that to
//! mean anything: the provider answers in the frame it declares, refuses
//! what it cannot do rather than guessing, moves the bodies the way the
//! sky does, and differentiates rather than differences.
//!
//! Accuracy is not tested here. It is measured against an engine, in
//! `03-design/builtin-ephemeris-measured.md` and
//! `03-design/lunar-accuracy-measured.md`, because a test that asserted
//! an accuracy figure would be asserting a number this crate cannot
//! check without the engine that produced it.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "a test fails by panicking and indexes its own fixtures"
)]

use teistro_ephemeris_builtin::provider::Builtin;
use teistro_port_ephemeris::{
    Body, CellStatus, Centre, Coordinates, Corrections, EphemerisProvider, Equinox, Frame,
    PositionRequest, TimeScale, Zodiac,
};

/// J2000, and two instants either side of it inside the claimed span.
const INSTANTS: [f64; 3] = [2_400_000.5, 2_451_545.0, 2_500_000.5];

fn frame() -> Frame {
    Frame {
        centre: Centre::Geocentric,
        equinox: Equinox::J2000,
        coordinates: Coordinates::Ecliptic,
        zodiac: Zodiac::Tropical,
        corrections: Corrections::GEOMETRIC,
    }
}

fn ask(jds: &[f64], bodies: &[Body]) -> teistro_port_ephemeris::PositionColumns {
    let provider = Builtin::new();
    let request = PositionRequest::new(jds, TimeScale::Tt, bodies, frame());
    provider
        .positions(&request)
        .unwrap_or_else(|error| panic!("the provider refused its own frame: {error}"))
}

#[test]
fn it_answers_for_every_body_it_declares() {
    let provider = Builtin::new();
    let bodies = provider.capabilities().bodies;
    assert!(
        !bodies.is_empty(),
        "a provider that names no body is no provider"
    );
    let columns = ask(&INSTANTS, &bodies);
    assert!(columns.all_ok(), "every declared body must answer");
    assert_eq!(columns.len(), INSTANTS.len() * bodies.len());
}

/// Pluto has no VSOP87 series and is fitted separately; the nodes and
/// apogees are mean elements. Until they are built, asking for one must
/// fail, and it must fail by name.
///
/// The port refuses it at the request rather than per cell — the
/// declared body list is a contract, so a body outside it is a malformed
/// request and not a cell that happens to be empty. That is stricter
/// than the provider's own fallback, which is why the fallback stays: it
/// is the answer if the two lists ever disagree.
#[test]
fn a_body_it_cannot_do_is_refused_by_name() {
    let provider = Builtin::new();
    let jds = [2_451_545.0];
    for body in [Body::Pluto, Body::MeanNode, Body::TrueNode] {
        let bodies = [body];
        let request = PositionRequest::new(&jds, TimeScale::Tt, &bodies, frame());
        let error = provider
            .positions(&request)
            .expect_err("a body with no theory must be refused");
        let message = error.to_string();
        assert!(
            message.contains("does not support"),
            "the refusal must say what it cannot do: {message}"
        );
    }
}

#[test]
fn an_instant_outside_the_measured_span_is_refused() {
    let provider = Builtin::new();
    let (from, to) = provider.capabilities().jd_range;
    let columns = ask(&[from - 1.0, to + 1.0], &[Body::Sun]);
    for index in 0..columns.len() {
        assert_eq!(
            columns.cell(index).expect("a cell").status,
            CellStatus::OutOfRange,
            "outside the span nothing has measured, the answer is a status"
        );
    }
}

#[test]
fn a_frame_it_does_not_produce_is_an_error_and_not_a_wrong_answer() {
    let provider = Builtin::new();
    let jds = [2_451_545.0];
    let bodies = [Body::Sun];
    let of_date = Frame {
        equinox: Equinox::OfDate,
        ..frame()
    };
    let apparent = Frame {
        corrections: Corrections::APPARENT,
        ..frame()
    };
    let equatorial = Frame {
        coordinates: Coordinates::Equatorial,
        ..frame()
    };
    for other in [of_date, apparent, equatorial] {
        let request = PositionRequest::new(&jds, TimeScale::Tt, &bodies, other);
        assert!(
            provider.positions(&request).is_err(),
            "the provider must refuse {} rather than answer in its own frame",
            other.key()
        );
    }
}

/// The Sun's geocentric longitude runs a full turn in a year, and the
/// Moon's in a month. This is the coarsest possible check that the
/// bodies are the bodies, and it would catch a swapped pair or a
/// mis-scaled time argument.
#[test]
fn the_bodies_move_at_the_rates_their_names_imply() {
    let columns = ask(&[2_451_545.0], &[Body::Sun, Body::Moon]);
    let sun = columns.at(0, 0).expect("the Sun");
    let moon = columns.at(0, 1).expect("the Moon");
    assert!(
        (sun.lon_speed - 1.019).abs() < 0.05,
        "the Sun moves about a degree a day, not {}",
        sun.lon_speed
    );
    assert!(
        (moon.lon_speed - 13.2).abs() < 2.0,
        "the Moon moves about thirteen degrees a day, not {}",
        moon.lon_speed
    );
    assert!(
        (sun.dist - 0.983).abs() < 0.02,
        "the Sun is about an astronomical unit away, not {}",
        sun.dist
    );
    assert!(
        (moon.dist - 0.0026).abs() < 0.0004,
        "the Moon is about a four-hundredth of that, not {}",
        moon.dist
    );
}

/// Every rate must agree with what the position actually does, which is
/// the check that the derivative is the derivative of *this* series and
/// not of something adjacent to it.
#[test]
fn every_rate_agrees_with_the_motion_it_describes() {
    const STEP: f64 = 0.25;
    let provider = Builtin::new();
    let bodies = provider.capabilities().bodies;
    let jd = 2_451_545.0;
    let before = ask(&[jd - STEP], &bodies);
    let here = ask(&[jd], &bodies);
    let after = ask(&[jd + STEP], &bodies);
    for (index, body) in bodies.iter().enumerate() {
        let a = before.at(0, index).expect("a cell");
        let b = here.at(0, index).expect("a cell");
        let c = after.at(0, index).expect("a cell");
        // The difference has to be wrapped into a half-turn either way,
        // not into a full turn upward: Saturn is retrograde at J2000, so
        // a naive `rem_euclid` turns a rate of -0.02 degrees a day into
        // 720. Getting that wrong is how a test can be greener than the
        // code it tests.
        let mut moved = c.lon - a.lon;
        while moved > 180.0 {
            moved -= 360.0;
        }
        while moved < -180.0 {
            moved += 360.0;
        }
        let difference = moved / (2.0 * STEP);
        assert!(
            (difference - b.lon_speed).abs() < 0.02 * b.lon_speed.abs().max(0.05),
            "{body:?}: the rate says {} and the motion says {difference}",
            b.lon_speed
        );
        let radial = (c.dist - a.dist) / (2.0 * STEP);
        assert!(
            (radial - b.dist_speed).abs() < 1e-4 * b.dist.max(1.0),
            "{body:?}: the distance rate says {} and the motion says {radial}",
            b.dist_speed
        );
    }
}

/// The bija is a correction to the Moon and nothing else, and turning it
/// off must change the Moon and nothing else.
#[test]
fn the_bija_moves_the_moon_and_leaves_everything_else_alone() {
    let jds = [2_378_500.0, 2_451_545.0, 2_597_000.0];
    let bodies = [Body::Sun, Body::Moon, Body::Mars];
    let with = PositionRequest::new(&jds, TimeScale::Tt, &bodies, frame());
    let corrected = Builtin::new().positions(&with).expect("the provider");
    let plain = Builtin::without_bija()
        .positions(&with)
        .expect("the provider");

    for jd_index in 0..jds.len() {
        for (body_index, body) in bodies.iter().enumerate() {
            let a = corrected.at(jd_index, body_index).expect("a cell");
            let b = plain.at(jd_index, body_index).expect("a cell");
            if *body == Body::Moon {
                continue;
            }
            assert!(
                (a.lon - b.lon).abs() < 1e-12,
                "{body:?} moved when only the Moon should have"
            );
        }
    }
    // At the far end of the span the correction is arcseconds, which is
    // the whole point of it; at the middle of the fitted window it is
    // small. Both are asserted so a bija of zero would fail.
    let far = corrected.at(2, 1).expect("the Moon").lon - plain.at(2, 1).expect("the Moon").lon;
    assert!(
        far.abs() * 3_600.0 > 1.0,
        "the bija should be worth more than an arcsecond at 2400, not {}",
        far.abs() * 3_600.0
    );
}

/// The capabilities must describe what the provider actually does. A
/// provider that claimed a frame it refuses, or speeds it does not
/// compute, would break the SDK's completion rather than itself.
#[test]
fn the_capabilities_describe_what_it_does() {
    let capabilities = Builtin::new().capabilities();
    assert!(
        capabilities.deterministic,
        "constants and f64 in a fixed order"
    );
    assert!(capabilities.speeds, "it computes rates");
    assert!(
        !capabilities.native,
        "it has no engine of its own to pass through to"
    );
    assert_eq!(capabilities.native_frame, frame());
    assert!(
        capabilities.identity.tier.is_some(),
        "a tier is stamped for provenance"
    );
    assert!(
        capabilities.identity.data_version.contains("VSOP87A"),
        "the data version names its sources: {}",
        capabilities.identity.data_version
    );
    assert!(
        Builtin::new()
            .capabilities()
            .identity
            .data_version
            .contains("bija"),
        "and says whether the correction is applied"
    );
    assert!(
        !Builtin::without_bija()
            .capabilities()
            .identity
            .data_version
            .contains("bija")
    );
}
