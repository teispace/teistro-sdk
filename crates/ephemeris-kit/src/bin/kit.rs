//! Runs the conformance kit and the boundary benchmarks against the test
//! provider: the zero-setup baseline every adapter's numbers are read
//! against, and the run CI makes on every change. The test provider's
//! positions are not astronomy, so it has no corpus class; `sdk-only` is
//! checked all the same.

use std::process::ExitCode;

use teistro_ephemeris_kit::runner::{self, Grid};
use teistro_port_ephemeris::{EphemerisProvider, TestProvider};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let provider = TestProvider::new();
    let grid = Grid::standard(&TestProvider::BODIES);
    let kit = runner::run("test-provider", &provider, runner::out_dir(&args), || {
        runner::standard_rows(&provider, &grid)
    });
    let open = || -> Box<dyn EphemerisProvider> { Box::new(TestProvider::new()) };
    if runner::charts("test-provider", &open, &args) {
        kit
    } else {
        ExitCode::FAILURE
    }
}
