//! What every kit binary does, once: the standard grid, the rows every
//! provider is measured on, and the run that prints the report and the
//! rows, writes the results and turns the verdict into an exit code. An
//! adapter's binary opens its provider, adds its direct-binding row and
//! calls [`run`].

use std::path::Path;
use std::process::ExitCode;

use crate::bench::{self, Row};
use crate::completion::Completion;
use crate::kit::{self, Bounds, Results};
use crate::model::{Body, Coordinates, Frame, OverridePolicy, PositionRequest, TimeScale};
use crate::provider::EphemerisProvider;
use crate::vtable::Exported;

/// Timed calls per round.
pub const ITERATIONS: usize = 200;
/// Untimed calls before each round.
pub const WARMUP: usize = 20;
/// Rounds; the best is kept.
pub const ROUNDS: usize = 3;

/// The ten classical bodies, the grid the engines are measured on.
pub const PLANETS: [Body; 10] = [
    Body::Sun,
    Body::Moon,
    Body::Mercury,
    Body::Venus,
    Body::Mars,
    Body::Jupiter,
    Body::Saturn,
    Body::Uranus,
    Body::Neptune,
    Body::Pluto,
];

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
        PositionRequest {
            jds: &self.jds,
            scale: TimeScale::Ut1,
            bodies: &self.bodies,
            frame,
            observer: None,
            speeds: true,
        }
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
    for (policy, what) in [
        (OverridePolicy::PreferNative, "native"),
        (OverridePolicy::SdkOnly, "SDK"),
    ] {
        let completion = Completion::new(provider, policy);
        rows.push(row(
            &format!("positions grid {label} completed to equatorial, {what} obliquity"),
            || {
                let _ = completion.positions(&equatorial);
            },
        ));
    }
    rows
}

/// Runs the kit against `provider`, prints the report, runs and prints
/// the rows `bench` produces, writes `<results_dir>/<name>.json`, and
/// returns success when the kit passed.
#[allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "the shared body of the kit binaries"
)]
pub fn run<P: EphemerisProvider>(
    name: &str,
    provider: &P,
    results_dir: &Path,
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
    match results.write(results_dir) {
        Ok(path) => println!("written {}", path.display()),
        Err(error) => {
            eprintln!("cannot write the results: {error}");
            return ExitCode::FAILURE;
        }
    }
    if results.report.passed {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
