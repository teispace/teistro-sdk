//! Writes the engine's places for every star of the SDK's star table it
//! knows by name, with the engine's own catalogue record for each, as the
//! reference table the SDK's places are measured against
//! (`fixtures/teimeris/stars.json`): the number, the tool and its version,
//! never the code that produced it. The record lets the SDK's pipeline be
//! held to the engine's on the same astrometry, and the SDK's own
//! astrometry be compared with the engine's as data.
//!
//! ```text
//! TEIMERIS_LIB_DIR=... cargo run --release --bin teistro-ephemeris-teimeris-stars-table > fixtures/teimeris/stars.json
//! ```

#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::expect_used,
    reason = "a tooling binary prints its table and stops on a broken engine"
)]

use std::process::ExitCode;

use serde::Serialize;
use teimeris::{Context, Flags, TimeScale};
use teistro_core::catalogue::Star;
use teistro_ephemeris_teimeris::{TeimerisProvider, data_dir_from_env};
use teistro_port_ephemeris::EphemerisProvider;

/// Four TT instants: 1900, J2000.0, 2023 and 2100.
const INSTANTS: [f64; 4] = [2_415_020.0, 2_451_545.0, 2_460_000.5, 2_488_070.0];

/// The engine's name for a catalogued member: the traditional name the
/// catalogue's doc string begins with, or a nomenclature name for the
/// objects that have none.
fn engine_name(star: Star) -> String {
    match star {
        Star::SgrAStar => ",SgrA*".to_owned(),
        Star::GalacticPole => ",GPol".to_owned(),
        Star::GalacticPoleIau1958 => ",GP1958".to_owned(),
        other => other
            .doc()
            .split(',')
            .next()
            .unwrap_or_default()
            .trim()
            .to_owned(),
    }
}

#[derive(Serialize)]
struct Table {
    schema: &'static str,
    tool: String,
    scale: &'static str,
    /// The engine's obliquity and nutation at each instant, degrees.
    frames: Vec<Frame>,
    rows: Vec<Row>,
}

#[derive(Serialize)]
struct Frame {
    jd_tt: f64,
    mean_obliquity_deg: f64,
    true_obliquity_deg: f64,
    nutation_lon_deg: f64,
    nutation_obl_deg: f64,
}

#[derive(Serialize)]
struct Record {
    /// The engine's traditional name and nomenclature designation for the
    /// record it resolved the name to, so a name shared by two stars is
    /// caught.
    name: String,
    designation: String,
    /// 0 for ICRS, else 1950 or 2000.
    epoch: f64,
    ra_deg: f64,
    dec_deg: f64,
    /// Arcseconds a century, the great-circle rate.
    pm_ra_arcsec_century: f64,
    pm_dec_arcsec_century: f64,
    /// Astronomical units a century.
    radial_velocity_au_century: f64,
    parallax_arcsec: f64,
    magnitude: f64,
}

#[derive(Serialize)]
struct Place {
    jd_tt: f64,
    /// The true equator and ecliptic of date with every correction.
    apparent_lon_deg: f64,
    apparent_lat_deg: f64,
    apparent_ra_deg: f64,
    apparent_dec_deg: f64,
    /// The mean equator and ecliptic of date with aberration and deflection.
    mean_lon_deg: f64,
    mean_lat_deg: f64,
    /// The mean equator and ecliptic of date without aberration or deflection.
    geometric_lon_deg: f64,
    geometric_lat_deg: f64,
}

#[derive(Serialize)]
struct Row {
    star: &'static str,
    engine_name: String,
    record: Record,
    places: Vec<Place>,
}

fn places(ctx: &Context, name: &str) -> Result<Vec<Place>, teimeris::Error> {
    let mut places = Vec::new();
    for jd_tt in INSTANTS {
        let apparent = ctx.star_position(jd_tt, name, Flags::EPH_SWISS)?;
        let equatorial = ctx.star_position(jd_tt, name, Flags::EPH_SWISS | Flags::EQUATORIAL)?;
        let mean = ctx.star_position(jd_tt, name, Flags::EPH_SWISS | Flags::NO_NUTATION)?;
        let geometric = ctx.star_position(
            jd_tt,
            name,
            Flags::EPH_SWISS | Flags::NO_NUTATION | Flags::NO_ABERRATION | Flags::NO_DEFLECTION,
        )?;
        places.push(Place {
            jd_tt,
            apparent_lon_deg: apparent.lon,
            apparent_lat_deg: apparent.lat,
            apparent_ra_deg: equatorial.lon,
            apparent_dec_deg: equatorial.lat,
            mean_lon_deg: mean.lon,
            mean_lat_deg: mean.lat,
            geometric_lon_deg: geometric.lon,
            geometric_lat_deg: geometric.lat,
        });
    }
    Ok(places)
}

fn main() -> ExitCode {
    let data_dir = data_dir_from_env();
    let provider = match TeimerisProvider::open(&data_dir) {
        Ok(provider) => provider,
        Err(error) => {
            eprintln!("cannot open Teimeris over {}: {error}", data_dir.display());
            return ExitCode::FAILURE;
        }
    };
    let capabilities = provider.capabilities();
    let frames = provider.with_context(|ctx| {
        INSTANTS
            .iter()
            .map(|&jd_tt| {
                let o = ctx.obliquity(jd_tt, TimeScale::TT)?;
                Ok(Frame {
                    jd_tt,
                    mean_obliquity_deg: o.mean_obliquity,
                    true_obliquity_deg: o.true_obliquity,
                    nutation_lon_deg: o.nutation_lon,
                    nutation_obl_deg: o.nutation_obl,
                })
            })
            .collect::<Result<Vec<_>, teimeris::Error>>()
    });
    let frames = match frames {
        Ok(frames) => frames,
        Err(error) => {
            eprintln!("the engine's obliquity: {error}");
            return ExitCode::FAILURE;
        }
    };
    let mut rows = Vec::new();
    for star in Star::ALL {
        let name = engine_name(star);
        let result = provider.with_context(|ctx| {
            let records = ctx.stars_named(&name)?;
            let Some(record) = records.first() else {
                return Ok(None);
            };
            let places = places(ctx, &name)?;
            Ok::<_, teimeris::Error>(Some((
                Record {
                    name: record.name.clone(),
                    designation: record.designation.clone(),
                    epoch: record.epoch,
                    ra_deg: record.ra,
                    dec_deg: record.dec,
                    pm_ra_arcsec_century: record.pm_ra,
                    pm_dec_arcsec_century: record.pm_dec,
                    radial_velocity_au_century: record.radial_velocity,
                    parallax_arcsec: record.parallax,
                    magnitude: record.magnitude,
                },
                places,
            )))
        });
        match result {
            Ok(Some((record, places))) => rows.push(Row {
                star: star.key(),
                engine_name: name,
                record,
                places,
            }),
            Ok(None) => eprintln!("{}: the engine has no star named {name}", star.key()),
            Err(error) => eprintln!("{}: {error}", star.key()),
        }
    }
    let table = Table {
        schema: "teistro-conformance/stars-table/1",
        tool: format!(
            "{} {}",
            capabilities.identity.name, capabilities.identity.version
        ),
        scale: "TT",
        frames,
        rows,
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&table).expect("a serialisable table")
    );
    ExitCode::SUCCESS
}
