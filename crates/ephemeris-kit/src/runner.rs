//! What every kit binary does, once: the standard grid, the rows every
//! provider is measured on, and the run that prints the report and the
//! rows, writes the results and turns the verdict into an exit code. An
//! adapter's binary opens its provider, adds its direct-binding row and
//! calls [`run`], then [`charts`] for the two checks made through the
//! façade: `sdk-only` always, and the conformance corpus when
//! `--corpus DIR --class CLASS` names it. `--native-frame-only` founds the
//! corpus's charts over the provider reduced to its native frame, so what
//! is measured is the SDK's completion from that provider's positions
//! rather than the engine's own frames.

use std::path::Path;
use std::process::ExitCode;

use teistro_astro::{Completion, DeltaTModel};
use teistro_core::settings::OverridePolicy;
use teistro_port_ephemeris::{
    Body, Coordinates, EphemerisProvider, Exported, Frame, PositionRequest, TimeScale,
};

use crate::bench::{self, Row};
use crate::corpus::{self, Corpus};
use crate::kit::{self, Bounds, Refusing, Results};
use crate::sdk_only::{self, NativeFrameOnly, Open};

/// Timed calls per round.
pub const ITERATIONS: usize = 200;
/// Untimed calls before each round.
pub const WARMUP: usize = 20;
/// Rounds; the best is kept.
pub const ROUNDS: usize = 3;

/// The instants and bodies a benchmark runs over.
#[derive(Clone, Debug, PartialEq)]
pub struct Grid {
    /// The instants, UT1.
    pub jds: Vec<f64>,
    /// The bodies.
    pub bodies: Vec<Body>,
}

impl Grid {
    /// A hundred instants at 36.525-day steps from J2000 over `bodies`.
    #[must_use]
    pub fn standard(bodies: &[Body]) -> Grid {
        Grid {
            jds: (0..100)
                .map(|i| 2_451_545.0 + f64::from(i) * 36.525)
                .collect(),
            bodies: bodies.to_vec(),
        }
    }

    /// `instants × bodies`, for row names.
    #[must_use]
    pub fn label(&self) -> String {
        format!("{} × {}", self.jds.len(), self.bodies.len())
    }

    /// The request over the grid in `frame`, geocentric, with speeds.
    #[must_use]
    pub fn request(&self, frame: Frame) -> PositionRequest<'_> {
        PositionRequest::new(&self.jds, TimeScale::Ut1, &self.bodies, frame)
    }
}

/// Times `f` with the standard counts.
pub fn row(name: &str, f: impl FnMut()) -> Row {
    bench::bench(name, ITERATIONS, WARMUP, ROUNDS, f)
}

/// The rows every provider is measured on: the grid through the trait,
/// through the C vtable, and completed to the equatorial frame with the
/// native obliquity and with the SDK's.
pub fn standard_rows<P: EphemerisProvider>(provider: &P, grid: &Grid) -> Vec<Row> {
    let label = grid.label();
    let canonical = grid.request(Frame::CANONICAL);
    let equatorial = grid.request(Frame::CANONICAL.with_coordinates(Coordinates::Equatorial));
    let mut rows = vec![row(
        &format!("positions grid {label} through the trait"),
        || {
            let _ = provider.positions(&canonical);
        },
    )];
    let exported = Exported::new(provider);
    if let Ok(bound) = exported.bound() {
        rows.push(row(
            &format!("positions grid {label} through the C vtable"),
            || {
                let _ = bound.positions(&canonical);
            },
        ));
    }
    // The rotation is measured over a provider that refuses the equatorial
    // frame, so the SDK does it whether or not the engine could have.
    let refusing = Refusing::new(provider, Frame::CANONICAL);
    for (policy, what) in [
        (OverridePolicy::PreferNative, "native"),
        (OverridePolicy::SdkOnly, "SDK"),
    ] {
        let completion = Completion::new(&refusing, policy, DeltaTModel::TableThenModel);
        rows.push(row(
            &format!("positions grid {label} completed to equatorial, {what} obliquity"),
            || {
                let _ = completion.positions(&equatorial);
            },
        ));
    }
    rows
}

/// The `--out DIR` argument of a kit binary, when given.
#[must_use]
pub fn out_dir(args: &[String]) -> Option<&Path> {
    args.iter()
        .position(|a| a == "--out")
        .and_then(|i| args.get(i + 1))
        .map(Path::new)
}

/// The `--corpus DIR --class CLASS` arguments of a kit binary, when both
/// are given: the corpus's root and the provider class to hold it to.
#[must_use]
pub fn corpus_args(args: &[String]) -> Option<(&Path, &str)> {
    let value = |flag: &str| {
        args.iter()
            .position(|a| a == flag)
            .and_then(|i| args.get(i + 1))
    };
    Some((Path::new(value("--corpus")?), value("--class")?.as_str()))
}

/// Runs the checks made through the façade: the `sdk-only` identity, and
/// the corpus when [`corpus_args`] names one, writing
/// `<out>/<name>-corpus.json` in the corpus's report format when a
/// directory is given. Answers whether both passed.
#[allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "the shared body of the kit binaries"
)]
pub fn charts(name: &str, open: Open<'_>, args: &[String]) -> bool {
    let identity = sdk_only::check(open);
    println!(
        "{}: {}; {}\n",
        identity.name,
        if identity.passed { "pass" } else { "FAIL" },
        identity.detail
    );
    let Some((root, class)) = corpus_args(args) else {
        return identity.passed;
    };
    let reduced = || -> Box<dyn EphemerisProvider> { Box::new(NativeFrameOnly::new(open())) };
    let (open, name): (Open<'_>, String) = if args.iter().any(|a| a == "--native-frame-only") {
        (&reduced, format!("{name}-native-frame"))
    } else {
        (open, name.to_owned())
    };
    let report = match Corpus::open(root).and_then(|corpus| corpus::run(&corpus, open, class)) {
        Ok(report) => report,
        Err(why) => {
            eprintln!("the corpus check did not run: {why}");
            return false;
        }
    };
    println!("{}", report.markdown());
    let judged = report.against(&corpus::KNOWN);
    println!(
        "{} misses explained by the known divergences; unexplained: {:?}; idle: {:?}",
        judged.explained,
        judged.unexplained,
        judged
            .idle
            .iter()
            .map(|divergence| divergence.name)
            .collect::<Vec<_>>()
    );
    if let Some(dir) = out_dir(args) {
        match report.write(dir, &format!("{name}-corpus")) {
            Ok(path) => println!("written {}", path.display()),
            Err(error) => {
                eprintln!("cannot write the corpus report: {error}");
                return false;
            }
        }
    }
    // Reduced to its native frame the run measures the SDK's completion,
    // which the known divergences do not describe: it is reported, and
    // only the provider's own run is judged.
    let reduced_run = args.iter().any(|a| a == "--native-frame-only");
    identity.passed && (reduced_run || judged.holds())
}

/// Runs the kit against `provider`, prints the report, runs and prints
/// the rows `bench` produces, writes `<out>/<name>.json` when a directory
/// is given, and returns success when the kit passed.
#[allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "the shared body of the kit binaries"
)]
pub fn run<P: EphemerisProvider>(
    name: &str,
    provider: &P,
    out: Option<&Path>,
    bench: impl FnOnce() -> Vec<Row>,
) -> ExitCode {
    let report = kit::run(provider, &Bounds::DEFAULT);
    println!("{}", report.markdown());
    let rows = bench();
    println!("{}", bench::markdown(&rows));
    let results = Results {
        provider: name.to_string(),
        report,
        bench: rows,
    };
    if let Some(dir) = out {
        match results.write(dir) {
            Ok(path) => println!("written {}", path.display()),
            Err(error) => {
                eprintln!("cannot write the results: {error}");
                return ExitCode::FAILURE;
            }
        }
    }
    if results.report.passed {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_grid_and_the_out_argument_are_read() {
        let grid = Grid::standard(&[Body::Sun, Body::Moon]);
        assert_eq!(grid.jds.len(), 100);
        assert_eq!(grid.label(), "100 × 2");
        assert_eq!(grid.request(Frame::CANONICAL).cell_count(), 200);
        let args = [String::from("--out"), String::from("target/kit")];
        assert_eq!(out_dir(&args), Some(Path::new("target/kit")));
        assert_eq!(out_dir(&[]), None);
        let corpus = ["--corpus", "fixtures", "--class", "builtin-full"].map(String::from);
        assert_eq!(
            corpus_args(&corpus),
            Some((Path::new("fixtures"), "builtin-full"))
        );
        assert_eq!(corpus_args(&corpus[..2]), None);
    }
}
