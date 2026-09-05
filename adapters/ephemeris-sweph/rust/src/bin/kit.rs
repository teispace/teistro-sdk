//! Runs the conformance kit and the boundary benchmarks against the Swiss
//! Ephemeris adapter, with the library's own per-cell call as the row the
//! port is measured against. `--out DIR` writes `DIR/sweph.json`.

#![allow(clippy::print_stderr, reason = "a tooling binary")]

use std::process::ExitCode;

use teistro_ephemeris_kit::runner::{self, Grid};
use teistro_ephemeris_sweph::{SwephProvider, data_dir_from_env};
use teistro_port_ephemeris::Body;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let data_dir = data_dir_from_env();
    let provider = match SwephProvider::open(&data_dir) {
        Ok(provider) => provider,
        Err(error) => {
            eprintln!(
                "cannot open the Swiss Ephemeris over {}: {error}",
                data_dir.display()
            );
            return ExitCode::FAILURE;
        }
    };
    let grid = Grid::standard(&Body::PLANETS);
    runner::run("sweph", &provider, runner::out_dir(&args), || {
        let mut rows = vec![runner::row(
            &format!(
                "positions grid {} through the C library directly",
                grid.label()
            ),
            || {
                let _ = provider.direct_grid(&grid.jds, &grid.bodies);
            },
        )];
        rows.extend(runner::standard_rows(&provider, &grid));
        rows
    })
}
