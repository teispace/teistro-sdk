//! **What Pluto costs**, and the fit that will carry it.
//!
//! Pluto has no VSOP87 series and no published analytic theory this SDK
//! can ship (ADR-0008), so it is the one body that must be *fitted*. This
//! measures what the fit costs: a ladder of interval lengths against a
//! ladder of degrees, each one fitted from the reference engine and then
//! asked how far its answer lands from the engine's own.
//!
//! # Heliocentric, and why that is not a detail
//!
//! What is fitted is the **heliocentric** vector. Pluto's motion about
//! the Sun is slow and smooth — one circuit in 248 years — and a
//! Chebyshev series is cheap on a smooth function. Its *geocentric*
//! motion is that same slow curve plus the Earth's whole annual orbit, a
//! wiggle of a full year superimposed on it, and fitting that would pay
//! for the Earth all over again at every one of Pluto's blocks. The Earth
//! already has a theory.
//!
//! The error reported is nevertheless the **geocentric angle**, because
//! that is what a chart shows. Each probe rebuilds the geocentric vector
//! from the fitted heliocentric one and the engine's own Earth, so the
//! number is what a consumer would see and not what the fitter would
//! like to report.
//!
//! # What it is fitted to
//!
//! The reference engine, as the Moon's bija is (ADR-0027). Agreement with
//! a reference proves consistency and not correctness (ADR-0021), the
//! provenance travels with the coefficients, and the refit against JPL
//! Horizons or CSPICE before v1 is the same refit the bija already owes.
//!
//! ```text
//! TEIMERIS_LIB_DIR=... \
//!   cargo run --release --bin teistro-ephemeris-teimeris-pluto-fit \
//!   > ../../../crates/ephemeris-builtin/data/pluto-fit.json
//! ```

#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "a tooling binary writes its document to stdout and stops on a broken engine"
)]

use std::f64::consts::PI;
use std::process::ExitCode;

use serde::Serialize;
use teistro_ephemeris_builtin::chebyshev;
use teistro_ephemeris_teimeris::{
    TeimerisProvider, data_dir_from_env, profile_from_env, profile_key,
};
use teistro_port_ephemeris::{
    Body, Centre, Coordinates, Corrections, EphemerisProvider, Equinox, Frame, PositionRequest,
    TimeScale, Zodiac,
};

/// The span `standard` claims, which is the span a table has to cover.
const FROM: f64 = 2_378_497.0; // 1800
const TO: f64 = 2_597_641.0; // 2400

/// A Julian year, the natural unit for an interval length.
const YEAR: f64 = 365.25;

/// The interval lengths swept, years.
const INTERVALS: [f64; 6] = [40.0, 20.0, 10.0, 5.0, 2.0, 1.0];

/// The terms per component swept, which is the degree plus one.
const TERMS: [usize; 6] = [6, 8, 10, 12, 14, 16];

/// Bytes a coefficient costs in the shipped table.
const BYTES_PER_COEFFICIENT: usize = 8;

/// The frame the fit is stated in: the provider's own native frame, so
/// the table drops straight in.
fn frame() -> Frame {
    Frame {
        centre: Centre::Geocentric,
        equinox: Equinox::J2000,
        coordinates: Coordinates::Ecliptic,
        zodiac: Zodiac::Tropical,
        corrections: Corrections::GEOMETRIC,
    }
}

/// A spherical cell as a rectangular vector.
fn rectangular(lon_deg: f64, lat_deg: f64, dist: f64) -> [f64; 3] {
    let (lon, lat) = (lon_deg.to_radians(), lat_deg.to_radians());
    let (sin_lat, cos_lat) = lat.sin_cos();
    let (sin_lon, cos_lon) = lon.sin_cos();
    [
        dist * cos_lat * cos_lon,
        dist * cos_lat * sin_lon,
        dist * sin_lat,
    ]
}

/// The angle between two vectors, arcseconds.
fn apart_arcsec(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dot: f64 = (0..3).map(|i| a[i] * b[i]).sum();
    let len = |v: [f64; 3]| v.iter().map(|c| c * c).sum::<f64>().sqrt();
    let (la, lb) = (len(a), len(b));
    if la == 0.0 || lb == 0.0 {
        return f64::NAN;
    }
    (dot / (la * lb)).clamp(-1.0, 1.0).acos() * 180.0 / PI * 3_600.0
}

/// The Sun and Pluto as the engine gives them, geocentric and rectangular.
struct Sample {
    sun: [f64; 3],
    pluto: [f64; 3],
}

/// Reads the engine at a grid of instants.
fn sample(engine: &TeimerisProvider, jds: &[f64]) -> Option<Vec<Sample>> {
    let request = PositionRequest::new(jds, TimeScale::Tt, &[Body::Sun, Body::Pluto], frame());
    let columns = engine.positions(&request).ok()?;
    (0..jds.len())
        .map(|index| {
            let sun = columns.at(index, 0)?;
            let pluto = columns.at(index, 1)?;
            (sun.is_ok() && pluto.is_ok()).then(|| Sample {
                sun: rectangular(sun.lon, sun.lat, sun.dist),
                pluto: rectangular(pluto.lon, pluto.lat, pluto.dist),
            })
        })
        .collect()
}

/// Fits one component over one interval from samples at Chebyshev nodes.
///
/// Interpolation at the points of the first kind, so the coefficients
/// fall out of a cosine transform rather than out of a solved system.
fn fit(samples: &[f64]) -> Vec<f64> {
    let n = samples.len();
    #[expect(clippy::cast_precision_loss, reason = "a degree is a small integer")]
    let scale = 2.0 / n as f64;
    let mut coefficients: Vec<f64> = (0..n)
        .map(|j| {
            let total: f64 = samples
                .iter()
                .enumerate()
                .map(|(k, sample)| {
                    #[expect(
                        clippy::cast_precision_loss,
                        reason = "a node index is a small integer"
                    )]
                    let angle = PI * (k as f64 + 0.5) * (j as f64) / (n as f64);
                    sample * angle.cos()
                })
                .sum();
            total * scale
        })
        .collect();
    if let Some(first) = coefficients.first_mut() {
        *first /= 2.0;
    }
    coefficients
}

/// The Chebyshev nodes of a degree-`n` fit, mapped onto `[from, from + length]`.
fn node_instants(from: f64, length: f64, count: usize) -> Vec<f64> {
    (0..count)
        .map(|k| {
            #[expect(
                clippy::cast_precision_loss,
                reason = "a node index is a small integer"
            )]
            let angle = PI * (k as f64 + 0.5) / count as f64;
            let x = angle.cos();
            from + (x + 1.0) / 2.0 * length
        })
        .collect()
}

/// What each tier asks of Pluto.
///
/// `compact` and `standard` carry the bounds their own manifest states —
/// an arcminute and an arcsecond. `full` cannot say "the theory as
/// published", because for a fitted body there is no published theory, so
/// its bound is **the floor the ladder itself reaches**: the cheapest
/// rung whose error is within half again of the best any rung achieves.
/// Spending more than that buys nothing, and the page says what the floor
/// was and that its cause is not established here.
const TIER_BOUNDS: [(&str, f64); 2] = [("compact", 60.0), ("standard", 1.0)];

/// How close to the best rung `full` must come.
const FULL_MARGIN: f64 = 1.5;

/// One rung of the ladder.
#[derive(Debug, Serialize)]
struct Rung {
    interval_years: f64,
    terms: usize,
    intervals: usize,
    coefficients: usize,
    bytes: usize,
    worst_arcsec: f64,
    mean_arcsec: f64,
}

/// The rung a tier takes, with the coefficients it takes.
#[derive(Debug, Serialize)]
struct Chosen {
    tier: String,
    bound_arcsec: f64,
    interval_years: f64,
    terms: usize,
    intervals: usize,
    start_jd: f64,
    interval_days: f64,
    bytes: usize,
    worst_arcsec: f64,
    coefficients: Vec<f64>,
}

#[derive(Debug, Serialize)]
struct Table {
    body: &'static str,
    frame: &'static str,
    fitted_to: &'static str,
    note: &'static str,
    from_jd: f64,
    to_jd: f64,
    engine_profile: String,
    probes_per_interval: usize,
    /// The floor the ladder reaches, which is what `full` is judged
    /// against. Its cause is not established here.
    floor_arcsec: f64,
    rungs: Vec<Rung>,
    chosen: Vec<Chosen>,
}

/// Fits the whole span at one rung and measures it.
fn measure(
    engine: &TeimerisProvider,
    interval_years: f64,
    terms: usize,
) -> Option<(Rung, Vec<f64>)> {
    let length = interval_years * YEAR;
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "the span over an interval is thousands at most"
    )]
    let intervals = ((TO - FROM) / length).ceil() as usize;

    // Every fit's nodes at once: one call into the engine rather than one
    // per interval, which is the difference between seconds and minutes.
    let mut node_jds = Vec::with_capacity(intervals * terms);
    for index in 0..intervals {
        #[expect(clippy::cast_precision_loss, reason = "an interval index is small")]
        let from = length.mul_add(index as f64, FROM);
        node_jds.extend(node_instants(from, length, terms));
    }
    let nodes = sample(engine, &node_jds)?;

    let mut coefficients = Vec::with_capacity(intervals * 3 * terms);
    for index in 0..intervals {
        for component in 0..3 {
            let samples: Vec<f64> = (0..terms)
                .map(|k| {
                    let at = &nodes[index * terms + k];
                    at.pluto[component] - at.sun[component]
                })
                .collect();
            coefficients.extend(fit(&samples));
        }
    }

    // Probes off the nodes, because a fit is exact *at* its nodes and
    // measuring there would report zero however bad the fit is.
    const PROBES: usize = 9;
    let mut probe_jds = Vec::with_capacity(intervals * PROBES);
    for index in 0..intervals {
        #[expect(clippy::cast_precision_loss, reason = "an interval index is small")]
        let from = length.mul_add(index as f64, FROM);
        for p in 0..PROBES {
            #[expect(clippy::cast_precision_loss, reason = "a probe index is small")]
            let fraction = (p as f64 + 0.5) / PROBES as f64;
            probe_jds.push(length.mul_add(fraction, from));
        }
    }
    let probes = sample(engine, &probe_jds)?;

    let (mut worst, mut total, mut counted) = (0.0_f64, 0.0, 0usize);
    for (index, jd) in probe_jds.iter().enumerate() {
        let block = ((jd - FROM) / length).floor();
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a block index is inside the span by construction"
        )]
        let block = (block as usize).min(intervals - 1);
        #[expect(clippy::cast_precision_loss, reason = "a block index is small")]
        let x = 2.0f64.mul_add((jd - FROM) / length - block as f64, -1.0);
        let mut fitted = [0.0; 3];
        for (component, slot) in fitted.iter_mut().enumerate() {
            let from = (block * 3 + component) * terms;
            *slot = chebyshev::evaluate(&coefficients[from..from + terms], x).0;
        }
        // Back to the geocentric direction a chart would show.
        let at = &probes[index];
        let rebuilt = [
            fitted[0] + at.sun[0],
            fitted[1] + at.sun[1],
            fitted[2] + at.sun[2],
        ];
        let difference = apart_arcsec(rebuilt, at.pluto);
        if difference.is_finite() {
            worst = worst.max(difference);
            total += difference;
            counted += 1;
        }
    }

    #[expect(clippy::cast_precision_loss, reason = "a probe count is thousands")]
    let mean = if counted == 0 {
        f64::NAN
    } else {
        total / counted as f64
    };
    Some((
        Rung {
            interval_years,
            terms,
            intervals,
            coefficients: coefficients.len(),
            bytes: coefficients.len() * BYTES_PER_COEFFICIENT,
            worst_arcsec: worst,
            mean_arcsec: mean,
        },
        coefficients,
    ))
}

/// The cheapest rung meeting a bound, if any does.
///
/// Cheapest and not best: the ladder is on the page for a reader who
/// wants the trade, and a tier that quietly took a finer rung than it
/// promised would be spending a consumer's bytes on the strength of
/// nothing written down.
fn cheapest<'a>(rungs: &'a [(Rung, Vec<f64>)], bound: f64) -> Option<&'a (Rung, Vec<f64>)> {
    rungs
        .iter()
        .filter(|(rung, _)| rung.worst_arcsec <= bound)
        .min_by_key(|(rung, _)| rung.bytes)
}

fn main() -> ExitCode {
    let engine = match TeimerisProvider::open(&data_dir_from_env()) {
        Ok(provider) => provider,
        Err(error) => {
            eprintln!("the engine is needed for this measurement: {error}");
            return ExitCode::FAILURE;
        }
    };
    if sample(&engine, &[2_451_545.0]).is_none() {
        eprintln!("the engine does not answer for Pluto in this frame");
        return ExitCode::FAILURE;
    }

    let mut fitted = Vec::new();
    for interval_years in INTERVALS {
        for terms in TERMS {
            if let Some(rung) = measure(&engine, interval_years, terms) {
                fitted.push(rung);
            }
        }
    }
    let floor = fitted
        .iter()
        .map(|(rung, _)| rung.worst_arcsec)
        .fold(f64::INFINITY, f64::min);

    let mut chosen = Vec::new();
    let bounds = TIER_BOUNDS
        .iter()
        .map(|(tier, bound)| ((*tier).to_string(), *bound))
        .chain(std::iter::once(("full".to_string(), floor * FULL_MARGIN)));
    for (tier, bound) in bounds {
        let Some((rung, coefficients)) = cheapest(&fitted, bound) else {
            eprintln!("no rung on the ladder meets {bound} arcseconds for {tier}");
            return ExitCode::FAILURE;
        };
        chosen.push(Chosen {
            tier,
            bound_arcsec: bound,
            interval_years: rung.interval_years,
            terms: rung.terms,
            intervals: rung.intervals,
            start_jd: FROM,
            interval_days: rung.interval_years * YEAR,
            bytes: rung.bytes,
            worst_arcsec: rung.worst_arcsec,
            coefficients: coefficients.clone(),
        });
    }
    let rungs: Vec<Rung> = fitted.into_iter().map(|(rung, _)| rung).collect();

    let table = Table {
        body: "PLUTO",
        frame: "HELIOCENTRIC/J2000/ECLIPTIC/RECTANGULAR, fitted; the error is the geocentric angle",
        fitted_to: "the reference engine",
        note: "a fit is exact at its own nodes, so every probe is off them; \
               the error is the angle a chart would show, rebuilt from the \
               fitted heliocentric vector and the engine's own Earth",
        from_jd: FROM,
        to_jd: TO,
        engine_profile: profile_key(profile_from_env()).to_string(),
        probes_per_interval: 9,
        floor_arcsec: floor,
        rungs,
        chosen,
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&table).expect("the table serialises")
    );
    ExitCode::SUCCESS
}
