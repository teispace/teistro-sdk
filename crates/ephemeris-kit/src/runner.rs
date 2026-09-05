//! What every kit binary does, once: the standard grid, the rows every
//! provider is measured on, and the run that prints the report and the
//! rows, writes the results and turns the verdict into an exit code. An
//! adapter's binary opens its provider, adds its direct-binding row and
//! calls [`run`].

use std::path::Path;
use std::process::ExitCode;

use teistro_astro::{Completion, DeltaTModel};
use teistro_core::settings::OverridePolicy;
use teistro_port_ephemeris::{
    Body, Coordinates, EphemerisProvider, Exported, Frame, PositionRequest, TimeScale,
};

use crate::bench::{self, Row};
use crate::kit::{self, Bounds, Refusing, Results};

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
    }
}
