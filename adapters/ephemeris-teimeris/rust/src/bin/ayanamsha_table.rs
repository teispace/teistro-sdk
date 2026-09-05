//! Writes the engine's mean ayanamsha for every catalogued member it offers
//! at a grid of instants, as the reference table the SDK's catalogue is
//! measured against (`fixtures/teimeris/ayanamsha.json`): the number, the
//! tool and its version, never the code that produced it.
//!
//! ```text
//! TEIMERIS_LIB_DIR=... cargo run --release --bin teistro-ephemeris-teimeris-ayanamsha-table > fixtures/teimeris/ayanamsha.json
//! ```

#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::expect_used,
    reason = "a tooling binary prints its table and stops on a broken engine"
)]

use std::process::ExitCode;

use serde::Serialize;
use teistro_core::catalogue::Ayanamsha;
use teistro_ephemeris_teimeris::{TeimerisProvider, data_dir_from_env};
use teistro_port_ephemeris::{EphemerisProvider, TimeScale};

/// The instants: Julian epochs 1500 to 2500 by fifty years, and the
/// ancient anchors the historical definitions are stated at.
const EPOCH_YEARS: [f64; 24] = [
    -700.0, -500.0, -300.0, -100.0, 100.0, 300.0, 500.0, 700.0, 900.0, 1100.0, 1300.0, 1500.0,
    1600.0, 1700.0, 1800.0, 1850.0, 1900.0, 1950.0, 2000.0, 2025.0, 2050.0, 2100.0, 2300.0, 2500.0,
];

#[derive(Serialize)]
struct Table {
    schema: &'static str,
    tool: String,
    scale: &'static str,
    basis: &'static str,
    rows: Vec<Row>,
}

#[derive(Serialize)]
struct Row {
    ayanamsha: &'static str,
    jd_tt: f64,
    mean_deg: f64,
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
    let mut rows = Vec::new();
    for id in Ayanamsha::ALL {
        if !capabilities.has_ayanamsha(id) {
            continue;
        }
        for year in EPOCH_YEARS {
            let jd_tt = 2_451_545.0 + (year - 2000.0) * 365.25;
            match provider.ayanamsha_deg(jd_tt, TimeScale::Tt, id) {
                Ok(mean_deg) => rows.push(Row {
                    ayanamsha: id.key(),
                    jd_tt,
                    mean_deg,
                }),
                Err(error) => eprintln!("{} at JD {jd_tt}: {error}", id.key()),
            }
        }
    }
    let table = Table {
        schema: "teistro-conformance/ayanamsha-table/1",
        tool: format!(
            "{} {}",
            capabilities.identity.name, capabilities.identity.version
        ),
        scale: "TT",
        basis: "MEAN",
        rows,
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&table).expect("a serialisable table")
    );
    ExitCode::SUCCESS
}
