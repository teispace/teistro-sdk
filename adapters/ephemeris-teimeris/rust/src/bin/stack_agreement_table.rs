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
use teistro_ephemeris_builtin::tables::TIER_NAME;
use teistro_ephemeris_teimeris::{
    TeimerisProvider, data_dir_from_env, profile_from_env, profile_key,
};
use teistro_port_ephemeris::{
    Body, Centre, Coordinates, Corrections, Equinox, Frame, PositionColumns, PositionRequest,
    TimeScale, Zodiac,
};

/// The bodies a Vedic chart asks for. Ketu is not among them: it is
/// Rahu's opposite point and the chart layer derives it, so it carries
/// Rahu's error exactly and measuring it twice would say nothing.
const BODIES: [Body; 9] = [
    Body::Sun,
    Body::Moon,
    Body::Mercury,
    Body::Venus,
    Body::Mars,
    Body::Jupiter,
    Body::Saturn,
    Body::MeanNode,
    Body::TrueNode,
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
    at_2000_arcsec: f64,
    at_2400_arcsec: f64,
    worst_speed_arcsec_per_day: f64,
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
    bija: bool,
    from_jd: f64,
    to_jd: f64,
    step_days: f64,
    samples: usize,
    engine_profile: String,
    note: &'static str,
    moon_worst_tithi_seconds: f64,
    by_correction: Vec<LadderRow>,
    rows: Vec<Row>,
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
                    0,
                ),
                sun_arcsec: worst_between(&builtin_side.columns, &engine_side.columns, 0),
                moon_arcsec: worst_between(&builtin_side.columns, &engine_side.columns, 1),
                mars_arcsec: worst_between(&builtin_side.columns, &engine_side.columns, 4),
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

    let mut rows = Vec::new();
    let mut moon_worst = 0.0_f64;
    for (body_index, body) in BODIES.iter().enumerate() {
        let mut worst = 0.0_f64;
        let mut worst_at = 0.0_f64;
        let mut worst_speed = 0.0_f64;
        let mut total = 0.0;
        let mut ends = [0.0_f64; 3];
        for (jd_index, _) in jds.iter().enumerate() {
            let (Some(a), Some(b)) = (
                over_builtin.columns.at(jd_index, body_index),
                over_engine.columns.at(jd_index, body_index),
            ) else {
                continue;
            };
            if !a.is_ok() || !b.is_ok() {
                continue;
            }
            let difference = apart(a.lon, b.lon).abs() * 3_600.0;
            if difference > worst {
                worst = difference;
                worst_at = jds[jd_index];
            }
            worst_speed = worst_speed.max((a.lon_speed - b.lon_speed).abs() * 3_600.0);
            total += difference;
            if jd_index == 0 {
                ends[0] = difference;
            }
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
            at_2000_arcsec: ends[1],
            at_2400_arcsec: ends[2],
            worst_speed_arcsec_per_day: worst_speed,
            worst_at_jd: worst_at,
        });
    }

    let table = Table {
        tier: TIER_NAME,
        frame: "TOPOCENTRIC/OF_DATE/ECLIPTIC/SIDEREAL(LAHIRI)/APPARENT",
        bija: true,
        from_jd: FROM,
        to_jd: TO,
        step_days: STEP,
        samples: jds.len(),
        engine_profile: profile_key(profile_from_env()).to_string(),
        note: "the same completion over each provider, so the difference is the ephemeris",
        by_correction,
        moon_worst_tithi_seconds: moon_worst * TITHI_SECONDS_PER_ARCSEC,
        rows,
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&table).expect("the table serialises")
    );
    ExitCode::SUCCESS
}
