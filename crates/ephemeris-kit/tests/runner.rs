//! The shared body of the kit binaries, run as `kit` runs it against the
//! test provider: the kit and the rows written under `--out`, the
//! `sdk-only` identity through the façade, and a corpus that cannot be
//! opened failing the run rather than passing it unmeasured.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index the JSON they expect"
)]

use std::path::Path;
use std::process::ExitCode;

use teistro_ephemeris_kit::runner::{self, Grid};
use teistro_port_ephemeris::{EphemerisProvider, TestProvider};

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|&arg| arg.to_owned()).collect()
}

#[test]
fn the_kit_runs_and_writes_its_results() {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join("kit-runner");
    let _ = std::fs::remove_dir_all(&out);
    let args = args(&["--out", out.to_str().unwrap()]);
    let provider = TestProvider::new();
    let grid = Grid::standard(&TestProvider::BODIES);
    let code = runner::run("test-provider", &provider, runner::out_dir(&args), || {
        runner::standard_rows(&provider, &grid)
    });
    assert_eq!(code, ExitCode::SUCCESS);
    let text = std::fs::read_to_string(out.join("test-provider.json")).unwrap();
    let results: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(results["provider"], "test-provider");
    assert_eq!(results["report"]["passed"], true);
    assert_ne!(results["bench"].as_array().map(Vec::len), Some(0));
}

#[test]
fn the_identity_passes_and_an_unopened_corpus_fails() {
    let open = || -> Box<dyn EphemerisProvider> { Box::new(TestProvider::new()) };
    assert!(runner::charts("test-provider", &open, &[]));
    let missing = args(&["--corpus", "/nowhere/teistro-corpus", "--class", "builtin"]);
    assert_eq!(
        runner::corpus_args(&missing),
        Some((Path::new("/nowhere/teistro-corpus"), "builtin"))
    );
    assert!(!runner::charts("test-provider", &open, &missing));
}
