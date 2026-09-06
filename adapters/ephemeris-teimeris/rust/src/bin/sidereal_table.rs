//! Writes the engine's Greenwich apparent sidereal time at a grid of UT1
//! instants, as the reference table the SDK's sidereal time is measured
//! against (`fixtures/teimeris/sidereal.json`): under the engine's default
//! model (the IERS 2010 expression inside its 1850 to 2050 window, a
//! long-term construction at and beyond the bounds) and under its IERS
//! 2010 model at every instant, with its Delta T, obliquity and nutation
//! at each, so the SDK's expression is held to the engine's on the same
//! TT. The number, the tool and its version, never the code that produced
//! it.
//!
//! ```text
//! TEIMERIS_LIB_DIR=... cargo run --release --bin teistro-ephemeris-teimeris-sidereal-table > fixtures/teimeris/sidereal.json
//! ```

#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::expect_used,
    reason = "a tooling binary prints its table and stops on a broken engine"
)]

use std::process::ExitCode;

use serde::Serialize;
use teimeris::{Context, ModelKind, SiderealTimeModel, TimeScale};
use teistro_ephemeris_teimeris::{
    TeimerisProvider, data_dir_from_env, profile_from_env, profile_key,
};
use teistro_port_ephemeris::EphemerisProvider;

/// The engine's window: strictly inside it the IERS 2010 expression, at
/// and beyond either bound its long-term construction.
const WINDOW_START_JD: f64 = 2_396_758.5;
const WINDOW_END_JD: f64 = 2_469_807.5;
const FIVE_YEARS_DAYS: f64 = 1826.25;

/// UT1 instants: 1700, 1750 and 1800; the window's first bound and the day
/// after; every five years through the window; the day before the second
/// bound and the bound; 2100, 2200 and 2300.
fn instants() -> Vec<f64> {
    let mut jds = vec![
        2_341_972.5,
        2_360_234.5,
        2_378_496.5,
        WINDOW_START_JD,
        WINDOW_START_JD + 1.0,
    ];
    jds.extend((1..=39).map(|k| WINDOW_START_JD + f64::from(k) * FIVE_YEARS_DAYS));
    jds.extend([
        WINDOW_END_JD - 1.0,
        WINDOW_END_JD,
        2_488_069.5,
        2_524_593.5,
        2_561_117.5,
    ]);
    jds
}

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
}

#[derive(Serialize)]
struct Row {
    jd_ut1: f64,
    /// `iers_2010` strictly inside the window, `long_term` at and beyond
    /// its bounds: the branch the default model takes.
    branch: &'static str,
    delta_t_seconds: f64,
    /// Under the default model.
    gast_deg: f64,
    /// Under the IERS 2010 model at every instant.
    gast_iers_2010_deg: f64,
    mean_obliquity_deg: f64,
    true_obliquity_deg: f64,
    nutation_lon_deg: f64,
    nutation_obl_deg: f64,
}

fn row(ctx: &Context, jd_ut1: f64) -> Result<Row, teimeris::Error> {
    let delta_t_seconds = ctx.delta_t(jd_ut1)?;
    let gast_deg = ctx.sidereal_time(jd_ut1, 0.0)? * 15.0;
    ctx.set_model(ModelKind::SIDEREAL_TIME, SiderealTimeModel::IERS_2010.raw())?;
    let iers = ctx.sidereal_time(jd_ut1, 0.0);
    ctx.set_model(ModelKind::SIDEREAL_TIME, SiderealTimeModel::LONG_TERM.raw())?;
    let obliquity = ctx.obliquity(jd_ut1 + delta_t_seconds / 86_400.0, TimeScale::TT)?;
    let inside = jd_ut1 > WINDOW_START_JD && jd_ut1 < WINDOW_END_JD;
    Ok(Row {
        jd_ut1,
        branch: if inside { "iers_2010" } else { "long_term" },
        delta_t_seconds,
        gast_deg,
        gast_iers_2010_deg: iers? * 15.0,
        mean_obliquity_deg: obliquity.mean_obliquity,
        true_obliquity_deg: obliquity.true_obliquity,
        nutation_lon_deg: obliquity.nutation_lon,
        nutation_obl_deg: obliquity.nutation_obl,
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
    for jd in instants() {
        match provider.with_context(|ctx| row(ctx, jd)) {
            Ok(row) => rows.push(row),
            Err(error) => eprintln!("JD {jd}: {error}"),
        }
    }
    let table = Table {
        schema: "teistro-conformance/sidereal-table/1",
        tool: format!(
            "{} {}",
            capabilities.identity.name, capabilities.identity.version
        ),
        profile: profile_key(profile),
        scale: "UT1",
        rows,
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&table).expect("a serialisable table")
    );
    ExitCode::SUCCESS
}
