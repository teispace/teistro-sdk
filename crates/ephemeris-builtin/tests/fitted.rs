//! The fitted Moon and Earth against their theories summed at every
//! instant (`Builtin::with_series`), over the provider's six centuries:
//! the bound `fitted.rs` states, at every tier this crate is built with.
//!
//! The bound is the instant's own resolution, not the fit's: a Julian day
//! near 2.4 million is an `f64` good to 40 microseconds, in which the
//! Moon moves 2e-5 arcseconds and turns its rate by 3e-8 of itself.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::print_stdout,
    reason = "a test fails by panicking, indexes its own columns, and prints \
              the worst figures it measured for the design page"
)]

use teistro_ephemeris_builtin::provider::Builtin;
use teistro_port_ephemeris::{
    Body, Centre, Coordinates, Corrections, EphemerisProvider, Equinox, Frame, PositionColumns,
    PositionRequest, TimeScale, Zodiac,
};

fn frame() -> Frame {
    Frame {
        centre: Centre::Geocentric,
        equinox: Equinox::J2000,
        coordinates: Coordinates::Ecliptic,
        zodiac: Zodiac::Tropical,
        corrections: Corrections::GEOMETRIC,
    }
}

fn ask(provider: &Builtin, jds: &[f64], bodies: &[Body]) -> PositionColumns {
    provider
        .positions(&PositionRequest::new(jds, TimeScale::Tt, bodies, frame()))
        .unwrap_or_else(|error| panic!("the provider refused: {error}"))
}

/// Instants over 1800 to 2400 at an awkward step, and each segment's
/// seams: the start of a two-day segment is a whole Julian day from
/// J2000's noon, and the instant a hair either side of it.
fn instants() -> Vec<f64> {
    let mut jds = Vec::new();
    for i in 0..3_000 {
        let jd = 2_378_500.0 + f64::from(i) * 73.137_11;
        let seam = 2_451_545.0 + 2.0 * ((jd - 2_451_545.0) / 2.0).floor();
        jds.extend([jd, seam, seam - 1e-6, seam + 1e-6]);
    }
    jds
}

/// The Sun's longitude and the Moon's, their latitudes, distances and
/// every rate, from the fit against the theories.
#[test]
fn the_fit_is_the_theory_to_the_resolution_of_the_instant() {
    let bodies = [Body::Sun, Body::Moon, Body::Mars, Body::TrueNode];
    let jds = instants();
    let (fitted, series) = (
        ask(&Builtin::new(), &jds, &bodies),
        ask(&Builtin::new().with_series(), &jds, &bodies),
    );
    // (body, position in arcseconds, distance in parts, rate in degrees
    // a day). A planet's rate is a difference that passes through zero at
    // a station, so the rates are bounded absolutely. The true node is
    // the plane of the Moon's position and velocity, so it shows the
    // rate's difference: the series' analytic rate is 1e-8 to 3e-8 from
    // the derivative of its own positions (a twice-extrapolated central
    // difference), the fit's derivative 6e-9, its floor, and the plane
    // tilts by hundredths of an arcsecond between the two.
    let bounds = [
        (Body::Sun, 2e-6, 1e-12, 1e-8),
        (Body::Moon, 3e-5, 1e-11, 1e-6),
        (Body::Mars, 2e-6, 1e-11, 1e-8),
        (Body::TrueNode, 0.1, 0.0, 1e-4),
    ];
    for (b, (body, angle, distance, rate)) in bounds.into_iter().enumerate() {
        let (mut worst_angle, mut worst_distance, mut worst_rate) = (0.0_f64, 0.0_f64, 0.0_f64);
        for (j, jd) in jds.iter().enumerate() {
            let (f, s) = (fitted.at(j, b).unwrap(), series.at(j, b).unwrap());
            let mut along = (f.lon - s.lon).abs();
            if along > 180.0 {
                along = 360.0 - along;
            }
            let off = along.hypot(f.lat - s.lat) * 3_600.0;
            assert!(off <= angle, "{body:?} at {jd}: {off}″");
            worst_angle = worst_angle.max(off);
            if s.dist > 0.0 {
                worst_distance = worst_distance.max((f.dist - s.dist).abs() / s.dist);
            }
            worst_rate = worst_rate
                .max((f.lon_speed - s.lon_speed).abs())
                .max((f.lat_speed - s.lat_speed).abs());
        }
        assert!(
            worst_distance <= distance,
            "{body:?} distance {worst_distance}"
        );
        assert!(worst_rate <= rate, "{body:?} rate {worst_rate}");
        println!(
            "{body:?}: {worst_angle:.2e}″, distance {worst_distance:.1e}, rate {worst_rate:.1e}°/d"
        );
    }
}

/// The fit is a fixed grid, so the same instant gives the same bits
/// asked alone, in a batch, or after a thread has fitted other years.
#[test]
fn an_instant_answers_the_same_bits_however_it_is_asked() {
    let provider = Builtin::new();
    let bodies = [Body::Sun, Body::Moon];
    let jd = 2_461_000.271_828;
    let alone = std::thread::spawn(move || ask(&Builtin::new(), &[jd], &[Body::Sun, Body::Moon]))
        .join()
        .unwrap();
    let _ = ask(&provider, &instants(), &bodies);
    let batched = ask(&provider, &[jd - 0.5, jd, jd + 0.5], &bodies);
    for (b, body) in bodies.iter().enumerate() {
        assert_eq!(
            format!("{:?}", alone.at(0, b).unwrap()),
            format!("{:?}", batched.at(1, b).unwrap()),
            "{body:?}"
        );
    }
}

/// The provider says it fitted, so a cached answer and a chart's
/// provenance tell the two apart.
#[test]
fn the_identity_says_whether_it_fitted() {
    let data = |provider: Builtin| provider.capabilities().identity.data_version;
    assert!(
        data(Builtin::new()).ends_with(" + fitted"),
        "{}",
        data(Builtin::new())
    );
    assert!(!data(Builtin::new().with_series()).contains("fitted"));
    assert_eq!(data(Builtin::default()), data(Builtin::new()));
}
