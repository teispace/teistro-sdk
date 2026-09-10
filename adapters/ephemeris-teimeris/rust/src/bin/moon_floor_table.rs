//! **What ELP2000-82B costs**, measured against the engine.
//!
//! `docs/03-design/lunar-accuracy-measured.md` established the budget: to
//! hold every panchanga boundary to a second of clock time the Moon's
//! longitude must be right to 0.445 arcseconds, and the tithi is the
//! binding limb. It also established that ELP/MPP02, the theory ADR-0013
//! chose, cannot be obtained, and that the theory which can says of
//! itself "Constants fitted to JPL's ephemerides DE200/LE200" — the same
//! 1981-generation fit whose age the planetary floor found drifting
//! VSOP87 by arcseconds.
//!
//! This is the measurement that settles whether the Moon inherits it.
//!
//! # The frame
//!
//! ELP is geocentric, ecliptic, and referred to the mean dynamical
//! ecliptic and inertial equinox of J2000, with no light time, no
//! aberration, no nutation and no precession applied. The port asks the
//! engine for exactly that, so the comparison is the theory against the
//! engine and no rung of the reduction chain besides — the same
//! stage-isolation ADR-0021 requires and the planets had.
//!
//! What it reports is the same three-way split, for the same reason: the
//! **mean** carries the fixed rotation between the dynamical frame and
//! the ICRS, the **scatter** is what a better theory would reduce, and
//! the **trend** tells a frame offset (flat) from a fit drifting away
//! from its epoch (growing).
//!
//! ```text
//! TEISTRO_ELP_DIR=... TEIMERIS_LIB_DIR=... \
//!   cargo run --release --bin teistro-ephemeris-teimeris-moon-floor \
//!   > ../../../crates/ephemeris-builtin/data/elp82b-floor.json
//! ```

#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "a tooling binary prints its table and stops on a broken engine"
)]

use std::path::PathBuf;
use std::process::ExitCode;

use serde::Serialize;
use teistro_ephemeris_builtin::elp::ingest::{Theory, load, position};
use teistro_ephemeris_teimeris::{
    TeimerisProvider, data_dir_from_env, profile_from_env, profile_key,
};
use teistro_port_ephemeris::{
    Body, Centre, Coordinates, Corrections, EphemerisProvider, Equinox, Frame, PositionRequest,
    TimeScale, Zodiac,
};

/// Radians to arcseconds.
const ARCSEC_PER_RAD: f64 = 206_264.806_247_096_36;

/// Seconds of tithi boundary per arcsecond of lunar longitude, from the
/// budget page: the elongation's slowest rate is 10.670 degrees a day.
const TITHI_SECONDS_PER_ARCSEC: f64 = 86_400.0 / (10.670 * 3_600.0);

/// The truncation ladder, in the coordinate's own unit — arcseconds for
/// longitude and latitude, kilometres for distance, which is the knob the
/// published reader itself takes.
const THRESHOLDS: [f64; 7] = [1.0, 0.3, 0.1, 0.03, 0.01, 0.001, 0.0];

fn direction_of(longitude_deg: f64, latitude_deg: f64) -> [f64; 3] {
    let longitude = longitude_deg.to_radians();
    let latitude = latitude_deg.to_radians();
    [
        latitude.cos() * longitude.cos(),
        latitude.cos() * longitude.sin(),
        latitude.sin(),
    ]
}

fn separation_arcsec(a: [f64; 3], b: [f64; 3]) -> f64 {
    let cross = [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ];
    let length = cross.iter().map(|c| c * c).sum::<f64>().sqrt();
    let dot: f64 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    length.atan2(dot) * ARCSEC_PER_RAD
}

fn unit(v: [f64; 3]) -> ([f64; 3], f64) {
    let length = v.iter().map(|c| c * c).sum::<f64>().sqrt();
    ([v[0] / length, v[1] / length, v[2] / length], length)
}

/// One threshold's row.
#[derive(Debug, Serialize)]
struct Row {
    threshold: f64,
    terms: usize,
    bytes: usize,
    mean_arcsec: f64,
    scatter_arcsec: f64,
    worst_arcsec: f64,
    worst_tithi_seconds: f64,
    radius_relative: f64,
    at_1800_arcsec: f64,
    at_2100_arcsec: f64,
    at_2400_arcsec: f64,
}

/// One span, measured with every term kept.
///
/// The span sweep is the finding: truncation converges long before the
/// theory does, so what decides the Moon is not how many terms are kept
/// but how far from its own epoch it is asked to work.
#[derive(Debug, Serialize)]
struct Span {
    label: &'static str,
    from_jd: f64,
    to_jd: f64,
    samples: usize,
    worst_arcsec: f64,
    mean_arcsec: f64,
    worst_tithi_seconds: f64,
    radius_relative: f64,
}

#[derive(Debug, Serialize)]
struct Table {
    source: &'static str,
    frame: &'static str,
    from_jd: f64,
    to_jd: f64,
    step_days: f64,
    samples: usize,
    engine_profile: String,
    whole_theory_terms: usize,
    tithi_seconds_per_arcsec: f64,
    rows: Vec<Row>,
    spans: Vec<Span>,
}

/// How many terms survive a threshold, and what they cost in a table.
fn kept(theory: &Theory, threshold: f64) -> (usize, usize) {
    let main = theory
        .main
        .iter()
        .filter(|(_, term)| term.coefficients[0].abs() >= threshold)
        .count();
    let perturbations = theory
        .perturbations
        .iter()
        .filter(|(_, term)| term.amplitude >= threshold)
        .count();
    // A main term keeps four multipliers and seven coefficients; a
    // perturbation keeps eleven multipliers, a phase and an amplitude.
    (main + perturbations, main * 60 + perturbations * 27)
}

fn main() -> ExitCode {
    let Some(dir) = std::env::var_os("TEISTRO_ELP_DIR").map(PathBuf::from) else {
        eprintln!("set TEISTRO_ELP_DIR to the published ELP2000-82B files (CDS VI/79)");
        return ExitCode::FAILURE;
    };
    let theory = load(&dir).expect("the published series");
    let provider =
        TeimerisProvider::open(&data_dir_from_env()).expect("the engine, for this measurement");

    let frame = Frame {
        centre: Centre::Geocentric,
        equinox: Equinox::J2000,
        coordinates: Coordinates::Ecliptic,
        zodiac: Zodiac::Tropical,
        corrections: Corrections::GEOMETRIC,
    };
    // The span is settable so the same measurement can be pointed at the
    // theory's own epoch, which is where a port has to be right before
    // anything it says about the edges can be believed.
    let number = |name: &str, fallback: f64| -> f64 {
        std::env::var(name)
            .ok()
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(fallback)
    };
    let from = number("TEISTRO_FROM_JD", 2_378_497.0);
    let to = number("TEISTRO_TO_JD", 2_597_641.0);
    let step = number("TEISTRO_STEP_DAYS", 7.0);
    let jds: Vec<f64> = (0..)
        .map(|index| step.mul_add(f64::from(index), from))
        .take_while(|jd| *jd <= to)
        .collect();
    let bodies = [Body::Moon];
    let request = PositionRequest::new(&jds, TimeScale::Tt, &bodies, frame);
    let answered = provider
        .positions(&request)
        .expect("the engine in the isolated frame");

    let mut rows = Vec::new();
    for threshold in THRESHOLDS {
        let (terms, bytes) = kept(&theory, threshold);
        let mut separations = Vec::with_capacity(jds.len());
        let mut radius_relative: f64 = 0.0;
        for (index, jd) in jds.iter().enumerate() {
            let cell = answered.at(index, 0).expect("a cell");
            let (theory_direction, distance_km) = unit(position(&theory, *jd, threshold));
            let engine = direction_of(cell.lon, cell.lat);
            separations.push(separation_arcsec(theory_direction, engine));
            // The port reports distance in astronomical units.
            let engine_km = cell.dist * 149_597_870.7;
            radius_relative =
                radius_relative.max(((distance_km - engine_km) / engine_km).abs());
        }
        let samples = separations.len();
        let mean = separations.iter().sum::<f64>() / samples as f64;
        let worst = separations.iter().copied().fold(0.0_f64, f64::max);
        rows.push(Row {
            threshold,
            terms,
            bytes,
            mean_arcsec: mean,
            scatter_arcsec: separations
                .iter()
                .map(|s| (s - mean).abs())
                .fold(0.0_f64, f64::max),
            worst_arcsec: worst,
            worst_tithi_seconds: worst * TITHI_SECONDS_PER_ARCSEC,
            radius_relative,
            at_1800_arcsec: separations[0],
            at_2100_arcsec: separations[samples / 2],
            at_2400_arcsec: separations[samples - 1],
        });
    }

    // The spans, each with every term kept, which is what shows that the
    // theory and not the table is the limit.
    const SPANS: [(&str, f64, f64); 5] = [
        ("1980 to 2020", 2_444_240.0, 2_458_850.0),
        ("1900 to 2100", 2_415_021.0, 2_488_070.0),
        ("1850 to 2150", 2_396_759.0, 2_506_332.0),
        ("1800 to 2400", 2_378_497.0, 2_597_641.0),
        ("1700 to 2500", 2_342_338.0, 2_634_166.0),
    ];
    let mut spans = Vec::new();
    for (label, from, to) in SPANS {
        let span_jds: Vec<f64> = (0..)
            .map(|index| 3.0_f64.mul_add(f64::from(index), from))
            .take_while(|jd| *jd <= to)
            .collect();
        let span_request = PositionRequest::new(&span_jds, TimeScale::Tt, &bodies, frame);
        let span_answered = provider
            .positions(&span_request)
            .expect("the engine in the isolated frame");
        let mut separations = Vec::with_capacity(span_jds.len());
        let mut radius_relative: f64 = 0.0;
        for (index, jd) in span_jds.iter().enumerate() {
            let cell = span_answered.at(index, 0).expect("a cell");
            let (theory_direction, distance_km) = unit(position(&theory, *jd, 0.0));
            separations.push(separation_arcsec(
                theory_direction,
                direction_of(cell.lon, cell.lat),
            ));
            let engine_km = cell.dist * 149_597_870.7;
            radius_relative = radius_relative.max(((distance_km - engine_km) / engine_km).abs());
        }
        let samples = separations.len();
        let worst = separations.iter().copied().fold(0.0_f64, f64::max);
        spans.push(Span {
            label,
            from_jd: from,
            to_jd: to,
            samples,
            worst_arcsec: worst,
            mean_arcsec: separations.iter().sum::<f64>() / samples as f64,
            worst_tithi_seconds: worst * TITHI_SECONDS_PER_ARCSEC,
            radius_relative,
        });
    }

    let table = Table {
        source: "ELP2000-82B, CDS catalogue VI/79 (Chapront-Touze and Chapront 1988), every file",
        frame: "GEOCENTRIC/J2000/ECLIPTIC/TROPICAL/GEOMETRIC",
        from_jd: from,
        to_jd: to,
        step_days: step,
        samples: jds.len(),
        engine_profile: profile_key(profile_from_env()).to_string(),
        whole_theory_terms: theory.terms(),
        tithi_seconds_per_arcsec: TITHI_SECONDS_PER_ARCSEC,
        rows,
        spans,
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&table).expect("the table serialises")
    );
    ExitCode::SUCCESS
}
