//! Writes the engine's planetary phenomena for the bodies with a disc and
//! the mean node at a grid of instants, with the geometry it read them
//! from, and its equation of time with the Sun's apparent right ascension,
//! as the reference table the SDK's phenomena are measured against
//! (`fixtures/teimeris/pheno.json`): the number, the tool and its version,
//! never the code that produced it. The geometry lets the SDK's arithmetic
//! be held to the engine's on the same positions.
//!
//! ```text
//! TEIMERIS_LIB_DIR=... cargo run --release --bin teistro-ephemeris-teimeris-pheno-table > fixtures/teimeris/pheno.json
//! ```

#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::expect_used,
    reason = "a tooling binary prints its table and stops on a broken engine"
)]

use std::process::ExitCode;

use serde::Serialize;
use teimeris::{Body as EngineBody, Context, Flags, TimeScale};
use teistro_ephemeris_teimeris::{
    TeimerisProvider, data_dir_from_env, profile_from_env, profile_key,
};
use teistro_port_ephemeris::{Body, EphemerisProvider};

/// The light time for one astronomical unit, days.
const LIGHT_TIME_DAYS_PER_AU: f64 = 499.004_783_836_156_4 / 86_400.0;

/// TT instants: a spread over two centuries, the day after the new Moon of
/// January 2000 (a thin crescent), Venus at its inferior conjunction of
/// June 2020 and a week later (the two branches of its fit and the gap
/// beyond it).
const INSTANTS: [f64; 16] = [
    2_415_020.0,
    2_430_000.5,
    2_440_587.5,
    2_444_239.5,
    2_447_892.5,
    2_451_545.0,
    2_451_551.0,
    2_455_197.5,
    2_459_003.5,
    2_459_010.5,
    2_459_580.5,
    2_460_000.5,
    2_462_502.5,
    2_469_807.5,
    2_477_112.5,
    2_488_070.0,
];

const BODIES: [(Body, EngineBody); 11] = [
    (Body::Sun, EngineBody::SUN),
    (Body::Moon, EngineBody::MOON),
    (Body::Mercury, EngineBody::MERCURY),
    (Body::Venus, EngineBody::VENUS),
    (Body::Mars, EngineBody::MARS),
    (Body::Jupiter, EngineBody::JUPITER),
    (Body::Saturn, EngineBody::SATURN),
    (Body::Uranus, EngineBody::URANUS),
    (Body::Neptune, EngineBody::NEPTUNE),
    (Body::Pluto, EngineBody::PLUTO),
    (Body::MeanNode, EngineBody::MEAN_NODE),
];

#[derive(Serialize)]
struct Table {
    schema: &'static str,
    tool: String,
    /// The engine profile the numbers were taken under: `compatible`
    /// reproduces the engine's own upstream, `max` carries the
    /// corrections the findings register asked for.
    profile: &'static str,
    scale: &'static str,
    rows: Vec<Row>,
    solar: Vec<Solar>,
}

#[derive(Serialize)]
struct Position {
    lon_deg: f64,
    lat_deg: f64,
    dist_au: f64,
}

#[derive(Serialize)]
struct Row {
    body: &'static str,
    jd_tt: f64,
    /// The body and the Sun, apparent and geocentric, and the body from the
    /// Sun at the retarded instant, in the ecliptic of date.
    body_position: Position,
    sun_position: Position,
    body_from_sun: Option<Position>,
    phase_angle_deg: f64,
    illuminated_fraction: f64,
    elongation_deg: f64,
    diameter_deg: f64,
    magnitude: Option<f64>,
    horizontal_parallax_deg: f64,
}

#[derive(Serialize)]
struct Solar {
    jd_ut1: f64,
    /// The Sun's apparent right ascension and declination, degrees.
    sun_ra_deg: f64,
    sun_dec_deg: f64,
    equation_of_time_seconds: f64,
}

fn position(
    ctx: &Context,
    jd_tt: f64,
    body: EngineBody,
    flags: Flags,
) -> Result<Position, teimeris::Error> {
    let p = ctx.position(jd_tt, body, flags)?;
    Ok(Position {
        lon_deg: p.lon,
        lat_deg: p.lat,
        dist_au: p.dist,
    })
}

fn row(ctx: &Context, body: Body, engine: EngineBody, jd_tt: f64) -> Result<Row, teimeris::Error> {
    let pheno = ctx.phenomena(jd_tt, engine, Flags::EPH_SWISS)?;
    let body_position = position(ctx, jd_tt, engine, Flags::EPH_SWISS)?;
    let sun_position = position(ctx, jd_tt, EngineBody::SUN, Flags::EPH_SWISS)?;
    let body_from_sun = if body.has_distance() && body != Body::Sun {
        let retarded = jd_tt - body_position.dist_au * LIGHT_TIME_DAYS_PER_AU;
        Some(position(
            ctx,
            retarded,
            engine,
            Flags::EPH_SWISS | Flags::HELIOCENTRIC,
        )?)
    } else {
        None
    };
    Ok(Row {
        body: body.key(),
        jd_tt,
        body_position,
        sun_position,
        body_from_sun,
        phase_angle_deg: pheno.phase_angle,
        illuminated_fraction: pheno.phase,
        elongation_deg: pheno.elongation,
        diameter_deg: pheno.diameter,
        magnitude: pheno.magnitude,
        horizontal_parallax_deg: pheno.horizontal_parallax,
    })
}

fn solar(ctx: &Context, jd_ut1: f64) -> Result<Solar, teimeris::Error> {
    let sun = ctx.position_at(
        jd_ut1,
        EngineBody::SUN,
        Flags::EPH_SWISS | Flags::EQUATORIAL,
        TimeScale::UT1,
        None,
    )?;
    Ok(Solar {
        jd_ut1,
        sun_ra_deg: sun.lon,
        sun_dec_deg: sun.lat,
        equation_of_time_seconds: ctx.equation_of_time(jd_ut1)?,
    })
}

fn main() -> ExitCode {
    let profile = profile_from_env();
    let data_dir = data_dir_from_env();
    let provider = match TeimerisProvider::open(&data_dir) {
        Ok(provider) => provider,
        Err(error) => {
            eprintln!("cannot open Teimeris over {}: {error}", data_dir.display());
            return ExitCode::FAILURE;
        }
    };
    let capabilities = provider.capabilities();
    let mut rows = Vec::new();
    let mut solar_rows = Vec::new();
    for jd in INSTANTS {
        for (body, engine) in BODIES {
            match provider.with_context(|ctx| row(ctx, body, engine, jd)) {
                Ok(row) => rows.push(row),
                Err(error) => eprintln!("{} at JD {jd}: {error}", body.key()),
            }
        }
        match provider.with_context(|ctx| solar(ctx, jd)) {
            Ok(row) => solar_rows.push(row),
            Err(error) => eprintln!("the Sun at JD {jd}: {error}"),
        }
    }
    let table = Table {
        schema: "teistro-conformance/pheno-table/1",
        tool: format!(
            "{} {}",
            capabilities.identity.name, capabilities.identity.version
        ),
        profile: profile_key(profile),
        scale: "TT",
        rows,
        solar: solar_rows,
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&table).expect("a serialisable table")
    );
    ExitCode::SUCCESS
}
