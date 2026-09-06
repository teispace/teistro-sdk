//! Writes the engine's house cusps and angles for every catalogued system at
//! a grid of instants, latitudes and longitudes, as the reference table the
//! SDK's houses are measured against (`fixtures/teimeris/houses.json`): the
//! number, the tool and its version, never the code that produced it. The
//! Sunshine system is left out, its Sun being the engine's own.
//!
//! ```text
//! TEIMERIS_LIB_DIR=... cargo run --release --bin teistro-ephemeris-teimeris-houses-table > fixtures/teimeris/houses.json
//! ```

#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::expect_used,
    reason = "a tooling binary prints its table and stops on a broken engine"
)]

use std::process::ExitCode;

use serde::Serialize;
use teimeris::HouseSystem as EngineSystem;
use teistro_core::catalogue::HouseSystem;
use teistro_ephemeris_teimeris::{
    TeimerisProvider, data_dir_from_env, profile_from_env, profile_key,
};
use teistro_port_ephemeris::EphemerisProvider;

/// Latitudes from the southern polar circle to inside the northern one.
const LATITUDES: [f64; 10] = [
    -66.0, -45.0, -23.5, 0.0, 27.7172, 45.0, 55.75, 64.8378, 69.6492, 80.0,
];

/// Two longitudes, so the meridian is not always Greenwich's.
const LONGITUDES: [f64; 2] = [0.0, 85.324];

/// Three UT1 instants across a day and a century.
const INSTANTS: [f64; 3] = [2_451_545.0, 2_460_000.5 + 0.3, 2_440_000.5 + 0.7];

/// The engine's name for a catalogued system.
fn engine_system(system: HouseSystem) -> Option<EngineSystem> {
    Some(match system {
        HouseSystem::WholeSign => EngineSystem::WHOLE_SIGN,
        HouseSystem::Placidus => EngineSystem::PLACIDUS,
        HouseSystem::Koch => EngineSystem::KOCH,
        HouseSystem::Regiomontanus => EngineSystem::REGIOMONTANUS,
        HouseSystem::Campanus => EngineSystem::CAMPANUS,
        HouseSystem::Equal => EngineSystem::EQUAL,
        HouseSystem::Meridian => EngineSystem::MERIDIAN,
        HouseSystem::Alcabitius => EngineSystem::ALCABITIUS,
        HouseSystem::Porphyry => EngineSystem::PORPHYRY,
        HouseSystem::Topocentric => EngineSystem::POLICH_PAGE,
        HouseSystem::Morinus => EngineSystem::MORINUS,
        HouseSystem::Sripati => EngineSystem::SRIPATI,
        HouseSystem::EqualMc => EngineSystem::EQUAL_MC,
        HouseSystem::EqualAries => EngineSystem::EQUAL_ASC_OFFSET,
        HouseSystem::Vehlow => EngineSystem::VEHLOW,
        HouseSystem::Carter => EngineSystem::CARTER_POLI_EQUATORIAL,
        HouseSystem::Horizon => EngineSystem::HORIZONTAL,
        HouseSystem::PullenSd => EngineSystem::PULLEN_SD,
        HouseSystem::PullenSr => EngineSystem::PULLEN_SR,
        HouseSystem::Krusinski => EngineSystem::KRUSINSKI,
        HouseSystem::Apc => EngineSystem::APC,
        // Sunshine needs the Sun and is measured against the baseline instead.
        _ => return None,
    })
}

#[derive(Serialize)]
struct Table {
    schema: &'static str,
    tool: String,
    /// The engine profile the numbers were taken under: `compatible`
    /// reproduces the engine's own upstream, `max` carries the
    /// corrections the findings register asked for.
    profile: &'static str,
    zodiac: &'static str,
    rows: Vec<Row>,
}

#[derive(Serialize)]
struct Row {
    system: &'static str,
    jd_ut1: f64,
    latitude_deg: f64,
    longitude_deg: f64,
    /// The system the engine used: Porphyry stands in inside the polar circle.
    substituted: bool,
    cusps: Vec<f64>,
    ascendant_deg: f64,
    midheaven_deg: f64,
    armc_deg: f64,
    vertex_deg: f64,
    equatorial_ascendant_deg: f64,
    co_ascendant_koch_deg: f64,
    co_ascendant_munkasey_deg: f64,
    polar_ascendant_deg: f64,
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
    for system in HouseSystem::ALL {
        let Some(engine) = engine_system(system) else {
            continue;
        };
        for jd_ut1 in INSTANTS {
            for latitude_deg in LATITUDES {
                for longitude_deg in LONGITUDES {
                    let result = provider.with_context(|ctx| {
                        ctx.houses(jd_ut1, latitude_deg, longitude_deg, engine)
                    });
                    match result {
                        Ok((cusps, angles)) => rows.push(Row {
                            system: system.key(),
                            jd_ut1,
                            latitude_deg,
                            longitude_deg,
                            substituted: angles.system_used != engine,
                            cusps,
                            ascendant_deg: angles.ascendant,
                            midheaven_deg: angles.midheaven,
                            armc_deg: angles.armc,
                            vertex_deg: angles.vertex,
                            equatorial_ascendant_deg: angles.equatorial_ascendant,
                            co_ascendant_koch_deg: angles.co_ascendant_koch,
                            co_ascendant_munkasey_deg: angles.co_ascendant_munkasey,
                            polar_ascendant_deg: angles.polar_ascendant,
                        }),
                        Err(error) => eprintln!(
                            "{} at JD {jd_ut1}, {latitude_deg}, {longitude_deg}: {error}",
                            system.key()
                        ),
                    }
                }
            }
        }
    }
    let table = Table {
        schema: "teistro-conformance/houses-table/1",
        tool: format!(
            "{} {}",
            capabilities.identity.name, capabilities.identity.version
        ),
        profile: profile_key(profile),
        zodiac: "TROPICAL",
        rows,
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&table).expect("a serialisable table")
    );
    ExitCode::SUCCESS
}
