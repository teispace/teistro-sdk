//! Runs the conformance kit and the boundary benchmarks against the
//! Teimeris adapter, with the engine's own batch call as the row the port
//! is measured against. Writes `spikes/03-ephemeris-port/results/teimeris.json`.

#![allow(clippy::print_stderr, reason = "a tooling binary")]

use std::path::Path;
use std::process::ExitCode;

use teistro_spike_adapter_teimeris::{TeimerisProvider, data_dir_from_env};
use teistro_spike_port::runner::{self, Grid};

fn main() -> ExitCode {
    let data_dir = data_dir_from_env();
    let provider = match TeimerisProvider::open(&data_dir) {
        Ok(provider) => provider,
        Err(error) => {
            eprintln!("cannot open Teimeris over {}: {error}", data_dir.display());
            return ExitCode::FAILURE;
        }
    };
    let grid = Grid::standard(&runner::PLANETS);
    runner::run(
        "teimeris",
        &provider,
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../results"),
        || {
            let mut rows = vec![runner::row(
                &format!(
                    "positions grid {} through the Teimeris binding directly",
                    grid.label()
                ),
                || {
                    let _ = provider.direct_columns(&grid.jds, &grid.bodies);
                },
            )];
            rows.extend(runner::standard_rows(&provider, &grid));
            rows
        },
    )
}
