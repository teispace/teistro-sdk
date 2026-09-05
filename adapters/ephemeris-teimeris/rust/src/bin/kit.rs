//! Runs the conformance kit and the boundary benchmarks against the
//! Teimeris adapter, with the engine's own batch call as the row the port
//! is measured against. `--out DIR` writes `DIR/teimeris.json`.

#![allow(clippy::print_stderr, reason = "a tooling binary")]

use std::process::ExitCode;

use teistro_ephemeris_kit::runner::{self, Grid};
use teistro_ephemeris_teimeris::{TeimerisProvider, data_dir_from_env};
use teistro_port_ephemeris::Body;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let data_dir = data_dir_from_env();
    let provider = match TeimerisProvider::open(&data_dir) {
        Ok(provider) => provider,
        Err(error) => {
            eprintln!("cannot open Teimeris over {}: {error}", data_dir.display());
            return ExitCode::FAILURE;
        }
    };
    let grid = Grid::standard(&Body::PLANETS);
    runner::run("teimeris", &provider, runner::out_dir(&args), || {
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
    })
}
