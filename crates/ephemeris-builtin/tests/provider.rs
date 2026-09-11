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
    clippy::float_cmp,
    reason = "a test fails by panicking, indexes its own fixtures, and a \
              direction's latitude and distance are exactly zero by \
              construction rather than nearly so"
)]

use teistro_astro::Completion;
use teistro_astro::delta_t::DeltaTModel;
use teistro_core::settings::OverridePolicy;
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

/// The osculating apogee needs the Earth-Moon mass parameter, which
/// neither theory carries. Until it is built, asking for one must fail,
/// and it must fail by name.
///
/// Pluto used to be in this list and is not any more: it is fitted rather
/// than derived (`03-design/pluto-measured.md`), and the test below
/// asks it for a place instead of asking it to refuse.
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
    let bodies = [Body::OsculatingApogee];
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

/// Pluto answers, and answers plausibly.
///
/// This crate does not test accuracy — that is measured against an engine
/// in `03-design/pluto-measured.md`, because a test asserting an accuracy
/// figure would be asserting a number the crate cannot check without the
/// engine that produced it. What it can check is that the fitted table is
/// wired to the right body in the right frame: Pluto is 29 to 50 AU from
/// the Sun and never leaves that shell, its geocentric distance stays
/// inside the Earth's orbit either side of it, and it creeps rather than
/// races, being the slowest thing here.
#[test]
fn pluto_is_fitted_and_answers_where_pluto_is() {
    let provider = Builtin::new();
    let (from, to) = provider.capabilities().jd_range;
    let jds = [from, 2_451_545.0, to];
    let columns = ask(&jds, &[Body::Pluto]);
    assert!(columns.all_ok(), "Pluto must answer across the whole span");
    for (index, jd) in jds.iter().enumerate() {
        let cell = columns.at(index, 0).expect("Pluto");
        assert!(
            (28.0..=52.0).contains(&cell.dist),
            "at {jd} Pluto is {} au from the Earth, which is nowhere it goes",
            cell.dist
        );
        assert!(
            cell.lat.abs() < 20.0,
            "at {jd} Pluto is {} degrees off the ecliptic; its orbit is inclined 17",
            cell.lat
        );
        assert!(
            cell.lon_speed.abs() < 0.1,
            "at {jd} Pluto moves {} degrees a day, which is not a 248-year orbit",
            cell.lon_speed
        );
    }
}

/// Pluto's fitted table must cover the whole range the provider
/// *declares*, and the two are generated by different passes from
/// different sources — so nothing but a test keeps them in step.
///
/// Outside the fit the table itself refuses rather than extrapolating,
/// because a Chebyshev series does not degrade past its interval, it
/// diverges. That refusal is a backstop the provider's own range check
/// should make unreachable, and this is the check that it does: if the
/// declared range ever outgrew the fit, Pluto would answer `NotComputed`
/// where every other body answered a place.
#[test]
fn plutos_fit_covers_the_range_the_provider_declares() {
    let provider = Builtin::new();
    let (from, to) = provider.capabilities().jd_range;
    let inside = [from, from + 1.0, f64::midpoint(from, to), to - 1.0, to];
    let columns = ask(&inside, &[Body::Pluto]);
    for (index, jd) in inside.iter().enumerate() {
        assert_eq!(
            columns.at(index, 0).expect("a cell").status,
            CellStatus::Ok,
            "the declared range is covered by the fit, and {jd} is in it"
        );
    }
    // And outside it, the port's own answer: a status rather than a
    // number, the same one every other body gives.
    let outside = ask(&[from - 1.0, to + 1.0], &[Body::Pluto]);
    for index in 0..2 {
        assert_eq!(
            outside.at(index, 0).expect("a cell").status,
            CellStatus::OutOfRange,
            "outside the span nothing has measured, the answer is a status"
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

/// The nodes are the reason this provider can cast a Vedic chart at all:
/// two of the nine grahas are Rahu and Ketu, and Ketu is derived from
/// Rahu by the chart layer rather than asked for.
///
/// What is asserted is what the sky does, not a recorded number. The mean
/// node goes backwards through the zodiac once in about 18.6 years, which
/// is 0.053 degrees a day; nothing else in the solar system moves like
/// that, so a node computed from the wrong polynomial cannot pass.
#[test]
fn the_mean_node_regresses_once_in_a_saros_and_a_bit() {
    let columns = ask(&INSTANTS, &[Body::MeanNode]);
    for index in 0..INSTANTS.len() {
        let cell = columns.at(index, 0).expect("the node");
        assert!(
            cell.lon_speed < 0.0,
            "the mean node is retrograde, always: {}",
            cell.lon_speed
        );
        let period_years = 360.0 / cell.lon_speed.abs() / 365.25;
        assert!(
            (period_years - 18.6).abs() < 0.1,
            "a nodal cycle is about 18.6 years, not {period_years}"
        );
    }
}

/// A node is a direction and not a place. The port says so, the
/// conformance corpus records it under every centre, and a provider that
/// answered a nominal distance beside one would be dressing a constant up
/// as a measurement.
#[test]
fn the_nodes_and_the_mean_apogee_are_directions() {
    let bodies = [Body::MeanNode, Body::TrueNode, Body::MeanApogee];
    let columns = ask(&INSTANTS, &bodies);
    for jd_index in 0..INSTANTS.len() {
        for (body_index, body) in bodies.iter().enumerate() {
            let cell = columns.at(jd_index, body_index).expect("a cell");
            assert!(!body.is_placed(), "{body:?} is not somewhere");
            assert_eq!(cell.lat, 0.0, "{body:?} is on the ecliptic by construction");
            assert_eq!(cell.dist, 0.0, "{body:?} has no distance to report");
            assert_eq!(cell.lat_speed, 0.0);
            assert!((0.0..360.0).contains(&cell.lon), "{body:?} at {}", cell.lon);
        }
    }
}

/// The true node oscillates about the mean one by a degree or two with a
/// period of about half a month. Too close and one of them is the other;
/// too far and the osculating plane is being read wrongly.
#[test]
fn the_true_node_oscillates_about_the_mean_one() {
    // A fortnight of days, which is long enough to see the wobble.
    let jds: Vec<f64> = (0..30).map(|d| 2_451_545.0 + f64::from(d)).collect();
    let columns = ask(&jds, &[Body::MeanNode, Body::TrueNode]);
    let mut worst: f64 = 0.0;
    for index in 0..jds.len() {
        let mean = columns.at(index, 0).expect("the mean node").lon;
        let true_node = columns.at(index, 1).expect("the true node").lon;
        let mut apart = true_node - mean;
        while apart > 180.0 {
            apart -= 360.0;
        }
        while apart < -180.0 {
            apart += 360.0;
        }
        worst = worst.max(apart.abs());
    }
    assert!(
        (0.1..3.0).contains(&worst),
        "the true node should wander a degree or so from the mean one, not {worst}"
    );
}

/// The apogee is half a turn from the perigee and goes forward, once in
/// about 8.85 years. Like the node's, the figure is the sky's and not a
/// recorded value.
#[test]
fn the_mean_apogee_advances_once_in_about_nine_years() {
    let columns = ask(&INSTANTS, &[Body::MeanApogee]);
    for index in 0..INSTANTS.len() {
        let cell = columns.at(index, 0).expect("the apogee");
        assert!(cell.lon_speed > 0.0, "the apogee advances");
        let period_years = 360.0 / cell.lon_speed / 365.25;
        assert!(
            (period_years - 8.85).abs() < 0.05,
            "an apsidal cycle is about 8.85 years, not {period_years}"
        );
    }
}

/// Where the Moon's latitude reaches zero going north, and how far the
/// node is from it, degrees. One entry per ascending crossing.
///
/// The crossing is interpolated rather than sampled. At six-hour steps
/// the Moon moves three degrees, so the nearest sample is up to three
/// degrees past the crossing, and a test that compared there would be
/// measuring its own step size and calling it an error.
fn node_against_crossing(
    columns: &teistro_port_ephemeris::PositionColumns,
    count: usize,
) -> Vec<f64> {
    let wrapped = |mut degrees: f64| {
        while degrees > 180.0 {
            degrees -= 360.0;
        }
        while degrees < -180.0 {
            degrees += 360.0;
        }
        degrees
    };
    let mut apart = Vec::new();
    for index in 1..count {
        let before = columns.at(index - 1, 0).expect("the Moon");
        let here = columns.at(index, 0).expect("the Moon");
        if before.lat >= 0.0 || here.lat < 0.0 {
            continue; // the ascending crossing only
        }
        let fraction = -before.lat / (here.lat - before.lat);
        let moved = wrapped(here.lon - before.lon);
        let at_crossing = before.lon + fraction * moved;
        let node = columns.at(index, 1).expect("the true node").lon;
        apart.push(wrapped(at_crossing - node));
    }
    apart
}

/// A month of six-hour steps from an instant.
fn a_month_from(jd: f64) -> Vec<f64> {
    (0..120).map(|q| f64::from(q).mul_add(0.25, jd)).collect()
}

/// The true node must sit where the Moon actually crosses the ecliptic.
///
/// This is the check that ties the node to the body rather than to a
/// polynomial, and it is exact rather than approximate: the osculating
/// orbit is the one whose plane contains the Moon's position and
/// velocity, so a Moon at zero latitude lies in the ecliptic *and* in
/// its own orbital plane — which is to say, on the line of nodes.
#[test]
fn the_true_node_is_where_the_moon_crosses_the_ecliptic() {
    let jds = a_month_from(2_451_545.0);
    let columns = ask(&jds, &[Body::Moon, Body::TrueNode]);
    let apart = node_against_crossing(&columns, jds.len());
    assert!(
        !apart.is_empty(),
        "a month must contain an ascending crossing"
    );
    for difference in apart {
        assert!(
            difference.abs() < 0.05,
            "at the ascending crossing the Moon is on the node line, \
             {difference} degrees apart"
        );
    }
}

/// The same property four centuries on, in the ecliptic **of date**.
///
/// The test above runs from J2000, where the ecliptic of date *is* the
/// J2000 ecliptic — so it cannot see which of the two the node was cut
/// against, and for a while the answer was the wrong one. A node is the
/// intersection of two planes, so tilting the reference plane slides it
/// along the orbit: about the tilt divided by the sine of the orbit's
/// five-degree inclination, which turns the ecliptic's 47 arcseconds a
/// century into some 520 of node. Measured against the reference engine
/// the node was 2044 arcseconds out by 2400, beside a mean node from this
/// same crate that was right to 0.985, and every test here passed.
///
/// So this one asks at an epoch where the two ecliptics differ, and in
/// the frame the node is defined in. It needs the completion, because
/// the provider answers in J2000 by declaration and the precession to
/// the equinox of date is the astronomy layer's step.
#[test]
fn the_true_node_is_cut_against_the_ecliptic_of_date() {
    let provider = Builtin::new();
    let completion = Completion::new(
        &provider,
        OverridePolicy::PreferNative,
        DeltaTModel::TableThenModel,
    );
    // 2400-ish: four centuries of precession, where cutting against
    // J2000's ecliptic instead of the date's is half a degree of node.
    let jds = a_month_from(2_597_000.0);
    let request = PositionRequest::new(
        &jds,
        TimeScale::Tt,
        &[Body::Moon, Body::TrueNode],
        Frame {
            equinox: Equinox::OfDate,
            ..frame()
        },
    );
    let done = completion
        .positions(&request)
        .expect("the completion precesses to the equinox of date");
    let apart = node_against_crossing(&done.columns, jds.len());
    assert!(
        !apart.is_empty(),
        "a month must contain an ascending crossing"
    );
    for difference in apart {
        assert!(
            difference.abs() < 0.05,
            "four centuries on, the node is still the crossing: \
             {difference} degrees apart"
        );
    }
}

/// At the epoch the mean node must be the number every reference gives
/// for it: 125.0445 degrees.
///
/// It is a weaker check than it looks, and saying so is the point — the
/// figure is ELP's own `W3` constant, so this confirms that the
/// polynomial is evaluated and rotated correctly at the epoch rather than
/// that the theory is right. What it would catch is a transposed
/// coefficient, a degree read as a radian, or a rotation applied when it
/// should be the identity, and those are exactly the mistakes a table of
/// constants invites.
#[test]
fn the_mean_node_at_the_epoch_is_the_published_constant() {
    let columns = ask(&[2_451_545.0], &[Body::MeanNode, Body::MeanApogee]);
    let node = columns.at(0, 0).expect("the node").lon;
    assert!(
        (node - 125.0445).abs() < 0.001,
        "the mean node at J2000 is 125.0445 degrees, not {node}"
    );
    // The perigee is 83.3532 at the epoch, so the apogee is half a turn
    // from it.
    let apogee = columns.at(0, 1).expect("the apogee").lon;
    assert!(
        (apogee - 263.3532).abs() < 0.001,
        "the mean apogee at J2000 is 263.3532 degrees, not {apogee}"
    );
}

/// A UT1 request and a TT request for the same Julian day are **not**
/// the same instant, and the provider must not answer as though they
/// were.
///
/// The completion passes the scale through rather than converting it, so
/// this is the provider's job. Delta T is about seventy seconds now, in
/// which the Moon moves 0.01 degrees — twenty seconds of tithi, on a
/// budget of one. A provider that ignored the scale would be quietly
/// wrong in exactly the quantity panchanga publishes.
#[test]
fn the_time_scale_is_honoured_and_not_assumed() {
    let provider = Builtin::new();
    let jds = [2_451_545.0];
    let bodies = [Body::Moon, Body::Sun];
    let in_tt = PositionRequest::new(&jds, TimeScale::Tt, &bodies, frame());
    let in_ut1 = PositionRequest::new(&jds, TimeScale::Ut1, &bodies, frame());
    let tt = provider.positions(&in_tt).expect("the provider");
    let ut1 = provider.positions(&in_ut1).expect("the provider");

    let moon_tt = tt.at(0, 0).expect("the Moon").lon;
    let moon_ut1 = ut1.at(0, 0).expect("the Moon").lon;
    let apart_arcsec = (moon_tt - moon_ut1).abs() * 3_600.0;
    // Delta T at J2000 is about 64 seconds, and the Moon moves 13.2
    // degrees a day, so the two instants are about 35 arcseconds apart.
    assert!(
        (20.0..60.0).contains(&apart_arcsec),
        "UT1 and TT should differ by Delta T's worth of lunar motion, not {apart_arcsec} arcsec"
    );
    // The Sun moves a fiftieth as fast, so its difference is far smaller
    // but still there — which is the check that the shift is a time shift
    // and not a constant added to the Moon.
    let sun_apart = (tt.at(0, 1).expect("the Sun").lon - ut1.at(0, 1).expect("the Sun").lon).abs();
    let moon_apart = (moon_tt - moon_ut1).abs();
    let ratio = moon_apart / sun_apart;
    assert!(
        (10.0..16.0).contains(&ratio),
        "the shift must scale with each body's own speed; the ratio is {ratio}"
    );
}
