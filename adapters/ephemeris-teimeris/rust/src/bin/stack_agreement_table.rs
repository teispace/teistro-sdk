//! **What the built-in ephemeris costs a chart**, measured against the
//! engine through the whole stack.
//!
//! Every measurement so far has isolated one thing: the truncation
//! against the whole theory, the theory against the engine in its own
//! frame, the Moon's budget against a tithi. This measures what a
//! consumer actually receives — a position in the frame a chart is cast
//! in, sidereal and apparent and of date — with the SDK's own completion
//! doing precession, light time, deflection, aberration, nutation and
//! the ayanamsha on both sides.
//!
//! Both sides use the same completion, the same settings and the same
//! instants. The only difference is which provider answered, so what the
//! table reports is the ephemeris and not the astronomy around it.
//!
//! The correction ladder is part of the document rather than something
//! printed beside it: stdout is the data file, so a table written to it
//! would be a table written into the JSON.
//!
//! This is the figure ADR-0027's per-body bounds are claims about, and
//! the one Phase 3's exit is gated on.
//!
//! The file is named for the tier the binary was built with, because the
//! tier is a compile-time feature and one run can only measure one of
//! them. `cargo xtask agreement` renders every tier it finds recorded and
//! names the ones it does not.
//!
//! ```text
//! TEIMERIS_LIB_DIR=... \
//!   cargo run --release --bin teistro-ephemeris-teimeris-stack-agreement \
//!   > ../../../crates/ephemeris-builtin/data/stack-agreement-standard.json
//! ```

#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "a tooling binary writes its document to stdout and stops on a broken engine"
)]

use std::process::ExitCode;

use serde::Serialize;
use teistro_astro::ayanamsha::Basis;
use teistro_astro::{Completion, DeltaTModel};
use teistro_core::catalogue::Ayanamsha;
use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
use teistro_core::settings::OverridePolicy;
use teistro_port_ephemeris::EphemerisProvider;
use teistro_ephemeris_builtin::provider::Builtin;
use teistro_ephemeris_builtin::tables::{MOON_BIJA_FITTED_OVER, TIER_NAME};
use teistro_ephemeris_teimeris::{
    TeimerisProvider, data_dir_from_env, profile_from_env, profile_key,
};
use teistro_port_ephemeris::{
    Body, Centre, Coordinates, Corrections, Equinox, Frame, PositionColumns, PositionRequest,
    TimeScale, Zodiac,
};

/// **Every body the port defines**, which since the osculating apogee
/// was built is every body the built-in computes.
///
/// A Vedic reading uses nine of them and no more, but an accuracy
/// document that covered only those would be silent about exactly the
/// bodies whose accuracy a reader cannot infer: Uranus and Neptune, where
/// VSOP87's own age and not a truncation is the limit; Pluto, which is
/// fitted rather than derived; and the osculating apogee, which is built
/// from a mass rather than from a theory.
///
/// The original note on Ketu: Ketu is not among them: it is
/// Rahu's opposite point and the chart layer derives it, so it carries
/// Rahu's error exactly and measuring it twice would say nothing.
const BODIES: [Body; 14] = [
    Body::Sun,
    Body::Moon,
    Body::Mercury,
    Body::Venus,
    Body::Mars,
    Body::Jupiter,
    Body::Saturn,
    Body::Uranus,
    Body::Neptune,
    Body::Pluto,
    Body::MeanNode,
    Body::TrueNode,
    Body::MeanApogee,
    Body::OsculatingApogee,
];

/// Kathmandu, where the conformance corpus's charts are cast.
fn place() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.3240),
        Altitude::literal(1400.0),
    )
}

/// The frame a chart is cast in: seen from the place, in the equinox of
/// date, sidereal by Lahiri, and apparent.
fn chart_frame() -> Frame {
    Frame {
        centre: Centre::Topocentric,
        equinox: Equinox::OfDate,
        coordinates: Coordinates::Ecliptic,
        zodiac: Zodiac::sidereal(Ayanamsha::Lahiri),
        corrections: Corrections::APPARENT,
    }
}

/// Seconds of tithi boundary per arcsecond of lunar longitude
/// (`03-design/lunar-accuracy-measured.md`).
const TITHI_SECONDS_PER_ARCSEC: f64 = 86_400.0 / (10.670 * 3_600.0);

#[derive(Debug, Serialize)]
struct Row {
    body: String,
    worst_arcsec: f64,
    mean_arcsec: f64,
    at_1800_arcsec: f64,
    at_2100_arcsec: f64,
    at_2400_arcsec: f64,
    worst_speed_arcsec_per_day: f64,
    /// Latitude, which longitude alone hides. It is what sets an orbit's
    /// plane, so a node is far more sensitive to it than to longitude,
    /// and a graha's war is decided by it.
    /// Whether the body is a *direction* — a node or an apogee, which
    /// has a longitude and nothing else. The SDK answers one with no
    /// latitude and no distance by construction, so those two columns
    /// compare a declared zero against whatever the engine supplies and
    /// are a convention rather than an error. The row says so rather than
    /// leaving a reader to know it.
    is_direction: bool,
    worst_lat_arcsec: f64,
    /// The latitude's *rate*, which is the out-of-plane motion and so
    /// what tilts the orbital plane the nodes are cut from. A position
    /// can agree while the plane it is moving in does not.
    worst_lat_speed_arcsec_per_day: f64,
    /// Distance, relative, which sets the parallax the centre step
    /// applies and the disc a phenomenon is computed from.
    worst_dist_relative: f64,
    /// The instant the worst disagreement happened at, which is what
    /// tells a systematic offset from a spike.
    worst_at_jd: f64,
}

/// One rung of the correction ladder.
#[derive(Debug, Serialize)]
struct LadderRow {
    correction: &'static str,
    /// How far the engine's own answer moves from its own geometric one
    /// when only this correction is asked for. A flag the engine does
    /// not honour leaves this at zero, and a comparison against a
    /// correction the engine did not apply says nothing at all — so the
    /// number sits beside the disagreement rather than in a footnote.
    engine_moves_sun_arcsec: f64,
    sun_arcsec: f64,
    moon_arcsec: f64,
    mars_arcsec: f64,
    /// Which implementation answered each step of the completion, so the
    /// row names who did the work it is measuring.
    builtin_steps: Vec<String>,
}

#[derive(Debug, Serialize)]
struct Table {
    tier: &'static str,
    frame: &'static str,
    /// The Moon's worst disagreement with the bija applied and with it
    /// off, so the field is a measurement of what the correction does
    /// rather than an assertion that it was switched on.
    moon_with_bija_arcsec: f64,
    moon_without_bija_arcsec: f64,
    /// The true node under the same pair. The bija corrects a mean
    /// longitude, so if its rate reaches the velocity the node — which is
    /// built from position crossed with velocity — moves with it.
    true_node_with_bija_arcsec: f64,
    true_node_without_bija_arcsec: f64,
    /// The span the bija's coefficients were fitted over, carried from
    /// the tables rather than restated, so a refit moves the page.
    moon_bija_fitted_over: &'static str,
    /// Each provider's true node against **its own** Moon's ascending
    /// crossing, degrees. The node is defined as the place where the
    /// orbit meets the ecliptic, so a provider whose Moon reaches zero
    /// latitude somewhere else is answering a different question — and
    /// this separates a disagreement between two definitions from an
    /// error in either.
    node_against_own_crossing: Vec<SelfCheck>,
    from_jd: f64,
    to_jd: f64,
    step_days: f64,
    samples: usize,
    engine_profile: String,
    note: &'static str,
    moon_worst_tithi_seconds: f64,
    by_correction: Vec<LadderRow>,
    /// From the Earth's centre: the ephemeris, and nothing that needs
    /// Delta T.
    geocentric_rows: Vec<Row>,
    /// From the place the chart is cast for: what a consumer receives,
    /// which carries the two sides' Delta T models as well.
    topocentric_rows: Vec<Row>,
}

/// One provider's true node against its own Moon.
#[derive(Debug, Serialize)]
struct SelfCheck {
    provider: &'static str,
    year: i32,
    crossings: usize,
    worst_degrees: f64,
}

/// The shortest way round the circle, degrees.
fn apart(a: f64, b: f64) -> f64 {
    let mut d = a - b;
    while d > 180.0 {
        d -= 360.0;
    }
    while d < -180.0 {
        d += 360.0;
    }
    d
}

/// The one completion every row of these tables is measured through, so
/// that a difference between two rows is the frame and never the setup.
///
/// **The two sides are not the same pipeline, and cannot be made so.**
/// The engine answers the whole frame in a single native call — its step
/// list is `positions:Native` and nothing else — while the built-in
/// ephemeris is completed by seven of the SDK's own steps. `SDK_ONLY`
/// looks like the fix and is not: the engine's native positions are
/// *apparent*, the SDK can add corrections and never remove them, so
/// asking it for a geometric frame is refused with
/// `Unsupported { step: "corrections" }`. Measured, not assumed.
///
/// So the tables record **both centres**. Geocentric isolates the
/// ephemeris: the instants are TT, so nothing in that path needs Delta T.
/// Topocentric is what a chart actually receives, and it carries the two
/// sides' Delta T models as well — at 2400 they differ by enough Earth
/// rotation to move the Moon about a hundred arcseconds, which is why the
/// Moon reads 3.2 arcseconds geocentric and 116.7 topocentric.
///
/// The true basis is the conformance baseline's: the engine has no basis
/// knob and applies the nutated ayanamsha, so the mean value would leave
/// the whole nutation in longitude — up to 18.5 arcseconds — in every
/// sidereal row at once.
fn completion<P: EphemerisProvider + ?Sized>(provider: &P) -> Completion<'_, P> {
    Completion::new(
        provider,
        OverridePolicy::PreferNative,
        DeltaTModel::TableThenModel,
    )
        .with_ayanamsha_basis(Basis::True)
}

fn main() -> ExitCode {
    let engine = match TeimerisProvider::open(&data_dir_from_env()) {
        Ok(provider) => provider,
        Err(error) => {
            eprintln!("the engine is needed for this measurement: {error}");
            return ExitCode::FAILURE;
        }
    };
    let builtin = Builtin::new();
    // The same ephemeris with the bija off, so what the correction is
    // worth can be read off rather than assumed (ADR-0027 keeps it a
    // declared, inspectable knob for exactly this reason).
    let uncorrected = Builtin::without_bija();

    const FROM: f64 = 2_378_497.0;
    const TO: f64 = 2_597_641.0;
    const STEP: f64 = 40.0;
    let jds: Vec<f64> = (0..)
        .map(|index| STEP.mul_add(f64::from(index), FROM))
        .take_while(|jd| *jd <= TO)
        .collect();

    // One correction at a time, so a disagreement names the correction
    // that introduced it rather than the four together.
    let geometric = Frame {
        centre: Centre::Geocentric,
        equinox: Equinox::OfDate,
        coordinates: Coordinates::Ecliptic,
        zodiac: Zodiac::Tropical,
        corrections: Corrections::GEOMETRIC,
    };
    let with = |f: fn(&mut Corrections)| {
        let mut c = Corrections::GEOMETRIC;
        f(&mut c);
        Frame { corrections: c, ..geometric }
    };
    let ladder: [(&str, Frame); 7] = [
        ("geometric (of date)", geometric),
        ("light time only", with(|c| c.light_time = true)),
        ("deflection only", with(|c| c.deflection = true)),
        ("aberration only", with(|c| c.aberration = true)),
        ("nutation only", with(|c| c.nutation = true)),
        ("apparent, tropical", Frame { corrections: Corrections::APPARENT, ..geometric }),
        ("apparent, sidereal", Frame {
            corrections: Corrections::APPARENT,
            zodiac: Zodiac::sidereal(Ayanamsha::Lahiri),
            ..geometric }),
    ];
    // Does the engine itself distinguish these frames? If a partial
    // correction gives the engine the same answer as geometric, the
    // engine is not honouring the flag and the comparison against it
    // says nothing.
    let base = PositionRequest::new(&jds, TimeScale::Tt, &BODIES, geometric);
    let engine_geometric = completion(&engine)
        .positions(&base)
        .expect("the engine, geometric");
    let worst_between = |a: &PositionColumns, b: &PositionColumns, index: usize| -> f64 {
        (0..jds.len())
            .filter_map(|j| {
                let (x, y) = (a.at(j, index)?, b.at(j, index)?);
                (x.is_ok() && y.is_ok()).then(|| apart(x.lon, y.lon).abs() * 3_600.0)
            })
            .fold(0.0_f64, f64::max)
    };
    // By body rather than by index: the ladder names three of them, and a
    // body added to `BODIES` must not silently re-point a column at its
    // neighbour.
    let column = |body: Body| {
        BODIES
            .iter()
            .position(|candidate| *candidate == body)
            .expect("the ladder's bodies are among the measured ones")
    };
    let (sun, moon, mars) = (column(Body::Sun), column(Body::Moon), column(Body::Mars));
    let by_correction: Vec<LadderRow> = ladder
        .into_iter()
        .map(|(correction, frame)| {
            let probe = PositionRequest::new(&jds, TimeScale::Tt, &BODIES, frame);
            let engine_side = completion(&engine).positions(&probe).expect("the engine");
            let builtin_side = completion(&builtin)
                .positions(&probe)
                .expect("the built-in ephemeris");
            LadderRow {
                correction,
                engine_moves_sun_arcsec: worst_between(
                    &engine_geometric.columns,
                    &engine_side.columns,
                    sun,
                ),
                sun_arcsec: worst_between(&builtin_side.columns, &engine_side.columns, sun),
                moon_arcsec: worst_between(&builtin_side.columns, &engine_side.columns, moon),
                mars_arcsec: worst_between(&builtin_side.columns, &engine_side.columns, mars),
                builtin_steps: builtin_side
                    .step_keys()
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            }
        })
        .collect();

    let frame = chart_frame();
    let mut request = PositionRequest::new(&jds, TimeScale::Tt, &BODIES, frame);
    request.observer = Some(place());

    // The same completion over each provider: what differs is the
    // ephemeris, and nothing else.
    let over_builtin = completion(&builtin)
        .positions(&request)
        .expect("the SDK over its own ephemeris");
    let over_engine = completion(&engine)
        .positions(&request)
        .expect("the SDK over the engine");
    // The same request from the Earth's centre. The instants are TT, so
    // nothing in this path needs Delta T: what is left is the ephemeris.
    let mut geocentric = PositionRequest::new(&jds, TimeScale::Tt, &BODIES, frame);
    geocentric.frame.centre = Centre::Geocentric;
    let geocentric_builtin = completion(&builtin)
        .positions(&geocentric)
        .expect("the SDK over its own ephemeris, from the centre");
    let geocentric_engine = completion(&engine)
        .positions(&geocentric)
        .expect("the SDK over the engine, from the centre");
    // From the centre, like every other figure the bija is judged by:
    // measuring it topocentrically would charge the correction for the
    // Delta T models as well.
    let over_uncorrected = completion(&uncorrected)
        .positions(&geocentric)
        .expect("the SDK over its own ephemeris, bija off");
    let moon_index = BODIES
        .iter()
        .position(|body| *body == Body::Moon)
        .expect("the Moon is among the bodies");
    let moon_without_bija = worst_between(
        &over_uncorrected.columns,
        &geocentric_engine.columns,
        moon_index,
    );
    let moon_with_bija = worst_between(
        &geocentric_builtin.columns,
        &geocentric_engine.columns,
        moon_index,
    );
    let node_index = BODIES
        .iter()
        .position(|body| *body == Body::TrueNode)
        .expect("the true node is among the bodies");
    let node_without_bija = worst_between(
        &over_uncorrected.columns,
        &geocentric_engine.columns,
        node_index,
    );
    let node_with_bija = worst_between(
        &geocentric_builtin.columns,
        &geocentric_engine.columns,
        node_index,
    );

    // One body's row, so the two centres are built by one piece of code
    // and a difference between the tables is the centre and never the
    // arithmetic.
    let rows_for = |a: &PositionColumns, b: &PositionColumns| -> (Vec<Row>, f64) {
        let mut rows = Vec::new();
        let mut moon_worst = 0.0_f64;
        for (body_index, body) in BODIES.iter().enumerate() {
            let mut worst = 0.0_f64;
            let mut worst_at = 0.0_f64;
            let mut worst_speed = 0.0_f64;
            let mut worst_lat = 0.0_f64;
            let mut worst_lat_speed = 0.0_f64;
            let mut worst_dist = 0.0_f64;
            let mut total = 0.0;
            let mut ends = [0.0_f64; 3];
            for (jd_index, jd) in jds.iter().enumerate() {
                let (Some(x), Some(y)) = (a.at(jd_index, body_index), b.at(jd_index, body_index))
                else {
                    continue;
                };
                if !x.is_ok() || !y.is_ok() {
                    continue;
                }
                let difference = apart(x.lon, y.lon).abs() * 3_600.0;
                if difference > worst {
                    worst = difference;
                    worst_at = *jd;
                }
                worst_speed = worst_speed.max((x.lon_speed - y.lon_speed).abs() * 3_600.0);
                worst_lat = worst_lat.max((x.lat - y.lat).abs() * 3_600.0);
                worst_lat_speed =
                    worst_lat_speed.max((x.lat_speed - y.lat_speed).abs() * 3_600.0);
                // A direction carries no distance, so there is nothing to
                // compare and a relative difference would be 0/0.
                if y.dist > 0.0 {
                    worst_dist = worst_dist.max((x.dist - y.dist).abs() / y.dist);
                }
                total += difference;
                if jd_index == 0 {
                    ends[0] = difference;
                }
                // The midpoint of 1800 to 2400, which is 2100 and not
                // 2000. `vsop.rs` names the same column the same way; a
                // field called `at_2000` holding the midpoint would be a
                // name asserting something nothing checks.
                if jd_index == jds.len() / 2 {
                    ends[1] = difference;
                }
                if jd_index == jds.len() - 1 {
                    ends[2] = difference;
                }
            }
            if *body == Body::Moon {
                moon_worst = worst;
            }
            #[expect(
                clippy::cast_precision_loss,
                reason = "a sample count is thousands"
            )]
            let mean = total / jds.len() as f64;
            rows.push(Row {
                body: body.key().to_string(),
                worst_arcsec: worst,
                mean_arcsec: mean,
                at_1800_arcsec: ends[0],
                at_2100_arcsec: ends[1],
                at_2400_arcsec: ends[2],
                worst_speed_arcsec_per_day: worst_speed,
                is_direction: !body.is_placed(),
                worst_lat_arcsec: worst_lat,
                worst_lat_speed_arcsec_per_day: worst_lat_speed,
                worst_dist_relative: worst_dist,
                worst_at_jd: worst_at,
            });
        }
        (rows, moon_worst)
    };

    // Each provider against itself: a month of six-hour steps, dense
    // enough to interpolate the Moon's ascending crossing, at three
    // epochs across the span.
    let self_check = |name: &'static str, provider: &dyn EphemerisProvider, year: i32, from: f64| {
        let dense: Vec<f64> = (0..120).map(|q| f64::from(q).mul_add(0.25, from)).collect();
        let request = PositionRequest::new(
            &dense,
            TimeScale::Tt,
            &[Body::Moon, Body::TrueNode],
            Frame {
                centre: Centre::Geocentric,
                zodiac: Zodiac::Tropical,
                ..chart_frame()
            },
        );
        let columns = completion(provider)
            .positions(&request)
            .expect("a month of the Moon and its node")
            .columns;
        let (mut worst, mut crossings) = (0.0_f64, 0usize);
        for index in 1..dense.len() {
            let (Some(before), Some(here)) = (columns.at(index - 1, 0), columns.at(index, 0)) else {
                continue;
            };
            if before.lat >= 0.0 || here.lat < 0.0 {
                continue; // the ascending crossing only
            }
            let Some(node) = columns.at(index, 1) else {
                continue;
            };
            crossings += 1;
            let fraction = -before.lat / (here.lat - before.lat);
            let at_crossing = before.lon + fraction * apart(here.lon, before.lon);
            worst = worst.max(apart(at_crossing, node.lon).abs());
        }
        SelfCheck {
            provider: name,
            year,
            crossings,
            worst_degrees: worst,
        }
    };
    let mut node_against_own_crossing = Vec::new();
    for (year, from) in [(1850, 2_396_759.0), (2000, 2_451_545.0), (2350, 2_579_305.0)] {
        node_against_own_crossing.push(self_check("built-in", &builtin, year, from));
        node_against_own_crossing.push(self_check("engine", &engine, year, from));
    }

    let (topocentric_rows, moon_worst) = rows_for(&over_builtin.columns, &over_engine.columns);
    let (geocentric_rows, _) = rows_for(&geocentric_builtin.columns, &geocentric_engine.columns);

    let table = Table {
        tier: TIER_NAME,
        frame: "OF_DATE/ECLIPTIC/SIDEREAL(LAHIRI)/APPARENT, from both centres",
        moon_with_bija_arcsec: moon_with_bija,
        moon_without_bija_arcsec: moon_without_bija,
        true_node_with_bija_arcsec: node_with_bija,
        true_node_without_bija_arcsec: node_without_bija,
        moon_bija_fitted_over: MOON_BIJA_FITTED_OVER,
        node_against_own_crossing,
        from_jd: FROM,
        to_jd: TO,
        step_days: STEP,
        samples: jds.len(),
        engine_profile: profile_key(profile_from_env()).to_string(),
        note: "the engine answers the whole frame natively (`positions:Native` alone) \
               and the built-in ephemeris is completed by the SDK's own steps, so these \
               are two pipelines and not one; geocentric isolates the ephemeris because \
               the instants are TT and nothing in that path needs Delta T, while \
               topocentric additionally carries the two sides' Delta T models",
        by_correction,
        moon_worst_tithi_seconds: moon_worst * TITHI_SECONDS_PER_ARCSEC,
        geocentric_rows,
        topocentric_rows,
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&table).expect("the table serialises")
    );
    ExitCode::SUCCESS
}
