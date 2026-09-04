//! Runs the conformance kit and the boundary benchmarks against the test
//! provider: the zero-setup baseline every adapter's numbers are read
//! against. Writes `spikes/03-ephemeris-port/results/test-provider.json`.

use std::path::Path;
use std::process::ExitCode;

use teistro_spike_port::runner::{self, Grid};
use teistro_spike_port::test_provider::SliceTestProvider;

fn main() -> ExitCode {
    let provider = SliceTestProvider::new();
    let grid = Grid::standard(&SliceTestProvider::BODIES);
    runner::run(
        "test-provider",
        &provider,
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../results"),
        || runner::standard_rows(&provider, &grid),
    )
}
