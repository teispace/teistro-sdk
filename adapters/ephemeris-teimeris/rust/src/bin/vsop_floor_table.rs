//! **What VSOP87 itself costs**, measured against the engine.
//!
//! `docs/03-design/builtin-ephemeris-measured.md` swept the truncation
//! of VSOP87 and published the curve, and it declined to pick a
//! threshold, for a stated reason: its last column is the whole theory
//! compared against *itself*, so it reads zero by construction while the
//! theory's own departure from reality does not. Choosing `standard` off
//! that page would be buying precision under a floor the page cannot
//! see.
//!
//! This measures the floor. Where it lands decides the tiers: a
//! threshold whose truncation error is far under the floor is paying
//! bytes for nothing.
//!
//! # Why the frame is the whole difficulty
//!
//! ADR-0021 requires validation to be stage-isolated — one rung of the
//! reduction chain at a time, so a discrepancy names its stage. VSOP87A
//! is *heliocentric, ecliptic, J2000, geometric*: no light time, no
//! aberration, no deflection, no nutation, no precession. The port can
//! ask for exactly that (`Centre::Heliocentric`, `Equinox::J2000`,
//! `Coordinates::Ecliptic`, `Corrections::GEOMETRIC`), so the comparison
//! is the theory against the engine and nothing else.
//!
//! **One rung still differs, and it is the size of the thing being
//! measured.** VSOP87 is referred to the *dynamical* ecliptic and
//! equinox of J2000; a modern engine works in the ICRS. The rotation
//! between them is a fixed frame bias of order a tenth of an arcsecond —
//! the same order as the floor. Booking it as "theory error" would be
//! wrong, and removing it silently would be worse.
//!
//! So this **separates the constant part from the moving part**, and
//! reports two diagnostics that say which is which: the heliocentric
//! **distance**, which a wrong place along a right orbit leaves alone,
//! and the **trend** across the span, which is flat for a frame bias and
//! grows for a fit drifting from its epoch.
//!
//! ```text
//! TEISTRO_VSOP87_DIR=... TEIMERIS_LIB_DIR=... \
//!   cargo run --release --bin teistro-ephemeris-teimeris-vsop-floor \
//!   > ../../../crates/ephemeris-builtin/data/vsop87-floor.json
//! ```
//!
//! The number, the tool and its version, never the code that produced
//! it: `cargo xtask vsop` reads the recorded table into the measured
//! page, so the page carries a measurement rather than a claim and does
//! not need the engine to regenerate.

#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "a tooling binary prints its table and stops on a broken engine"
)]

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::ExitCode;

use serde::Serialize;

use teistro_ephemeris_builtin::ingest::{BodySeries, load};
use teistro_ephemeris_builtin::series::millennia;
use teistro_ephemeris_teimeris::{
    TeimerisProvider, data_dir_from_env, profile_from_env, profile_key,
};
use teistro_port_ephemeris::{
    Body, Centre, Coordinates, Corrections, EphemerisProvider, Equinox, Frame, PositionRequest,
    TimeScale, Zodiac,
};

/// Radians to arcseconds.
const ARCSEC_PER_RAD: f64 = 206_264.806_247_096_36;

/// The bodies the comparison covers, with the name the series file uses.
const PAIRS: [(Body, &str); 7] = [
    (Body::Mercury, "Mercury"),
    (Body::Venus, "Venus"),
    (Body::Mars, "Mars"),
    (Body::Jupiter, "Jupiter"),
    (Body::Saturn, "Saturn"),
    (Body::Uranus, "Uranus"),
    (Body::Neptune, "Neptune"),
];

/// Where the published series are, if this machine has them.
fn series_dir() -> Option<PathBuf> {
    std::env::var_os("TEISTRO_VSOP87_DIR").map(PathBuf::from)
}

/// The heliocentric ecliptic direction the theory gives, as a unit
/// vector, and its distance in astronomical units.
fn theory_position(series: &BodySeries, jd: f64) -> ([f64; 3], f64) {
    let [x, y, z] = series.at(millennia(jd), 0.0);
    let length = (x * x + y * y + z * z).sqrt();
    ([x / length, y / length, z / length], length)
}

/// A spherical ecliptic longitude and latitude as a unit vector.
fn direction_of(longitude_deg: f64, latitude_deg: f64) -> [f64; 3] {
    let longitude = longitude_deg.to_radians();
    let latitude = latitude_deg.to_radians();
    [
        latitude.cos() * longitude.cos(),
        latitude.cos() * longitude.sin(),
        latitude.sin(),
    ]
}

/// The angle between two directions, in arcseconds, by the form that
/// stays accurate as the angle goes to zero.
fn separation_arcsec(a: [f64; 3], b: [f64; 3]) -> f64 {
    let cross = [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ];
    let length = cross.iter().map(|c| c * c).sum::<f64>().sqrt();
    let dot: f64 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    length.atan2(dot) * ARCSEC_PER_RAD
}

/// The difference between the theory and the engine, resolved into the
/// two components that have to be told apart: a fixed rotation of the
/// frame, and the theory's own error.
struct Difference {
    /// The mean separation over the grid, in arcseconds. A frame bias
    /// shows up here.
    mean_arcsec: f64,
    /// The worst departure from that mean, in arcseconds. This is the
    /// part a better theory would reduce, and it is the floor.
    scatter_arcsec: f64,
    /// The worst raw separation, for the record.
    worst_arcsec: f64,
    /// The worst relative disagreement in heliocentric distance.
    ///
    /// This is the diagnostic that says *what kind* of difference this
    /// is. A theory whose orbit is right but whose body sits at the
    /// wrong place along it — a mean-longitude or epoch difference,
    /// which is what an older fit produces — disagrees in direction and
    /// agrees in distance. A theory with the wrong orbit disagrees in
    /// both.
    radius_relative: f64,
    /// The separation at the first, middle and last instant, which
    /// separates a fixed offset from a drift. A frame bias is flat; an
    /// error in mean longitude grows.
    trend_arcsec: [f64; 3],
}

fn compare(
    provider: &TeimerisProvider,
    series: &BTreeMap<&'static str, BodySeries>,
    body: Body,
    name: &str,
    jds: &[f64],
) -> Difference {
    let frame = Frame {
        centre: Centre::Heliocentric,
        equinox: Equinox::J2000,
        coordinates: Coordinates::Ecliptic,
        zodiac: Zodiac::Tropical,
        corrections: Corrections::GEOMETRIC,
    };
    let bodies = [body];
    let request = PositionRequest::new(jds, TimeScale::Tt, &bodies, frame);
    let answered = provider
        .positions(&request)
        .unwrap_or_else(|error| panic!("{name}: the engine refused the isolated frame: {error}"));

    let mut separations = Vec::with_capacity(jds.len());
    let mut radius_relative: f64 = 0.0;
    for (index, jd) in jds.iter().enumerate() {
        let cell = answered
            .at(index, 0)
            .unwrap_or_else(|| panic!("{name}: no cell at {jd}"));
        let engine = direction_of(cell.lon, cell.lat);
        let (theory, distance) = theory_position(&series[name], *jd);
        separations.push(separation_arcsec(theory, engine));
        radius_relative = radius_relative.max(((distance - cell.dist) / distance).abs());
    }

    let samples = separations.len();
    let mean = separations.iter().sum::<f64>() / samples as f64;
    Difference {
        mean_arcsec: mean,
        scatter_arcsec: separations
            .iter()
            .map(|s| (s - mean).abs())
            .fold(0.0_f64, f64::max),
        worst_arcsec: separations.iter().copied().fold(0.0_f64, f64::max),
        radius_relative,
        trend_arcsec: [
            separations[0],
            separations[samples / 2],
            separations[samples - 1],
        ],
    }
}

/// One body's row in the recorded table.
#[derive(Debug, Serialize)]
struct Row {
    body: String,
    mean_arcsec: f64,
    scatter_arcsec: f64,
    worst_arcsec: f64,
    radius_relative: f64,
    at_1800_arcsec: f64,
    at_2100_arcsec: f64,
    at_2400_arcsec: f64,
}

/// The whole record, with what produced it.
#[derive(Debug, Serialize)]
struct Table {
    source: &'static str,
    frame: &'static str,
    from_jd: f64,
    to_jd: f64,
    step_days: f64,
    samples: usize,
    engine_profile: String,
    rows: Vec<Row>,
}

fn main() -> ExitCode {
    let Some(dir) = series_dir() else {
        eprintln!("set TEISTRO_VSOP87_DIR to the published VSOP87A files");
        return ExitCode::FAILURE;
    };
    let series = load(&dir).expect("the published series");
    let provider =
        TeimerisProvider::open(&data_dir_from_env()).expect("the engine, for this measurement");

    // Every 40 days over 1800 to 2400, which is `standard`'s own span.
    let jds: Vec<f64> = (0..)
        .map(|step| 2_378_497.0 + f64::from(step) * 40.0)
        .take_while(|jd| *jd <= 2_597_641.0)
        .collect();

    let mut rows = Vec::new();
    for (body, name) in PAIRS {
        let difference = compare(&provider, &series, body, name, &jds);
        rows.push(Row {
            body: name.to_string(),
            mean_arcsec: difference.mean_arcsec,
            scatter_arcsec: difference.scatter_arcsec,
            worst_arcsec: difference.worst_arcsec,
            radius_relative: difference.radius_relative,
            at_1800_arcsec: difference.trend_arcsec[0],
            at_2100_arcsec: difference.trend_arcsec[1],
            at_2400_arcsec: difference.trend_arcsec[2],
        });
    }

    let table = Table {
        source: "VSOP87A, CDS catalogue VI/81 (Bretagnon and Francou 1988), every term",
        frame: "HELIOCENTRIC/J2000/ECLIPTIC/TROPICAL/GEOMETRIC",
        from_jd: 2_378_497.0,
        to_jd: 2_597_641.0,
        step_days: 40.0,
        samples: jds.len(),
        engine_profile: profile_key(profile_from_env()).to_string(),
        rows,
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&table).expect("the table serialises")
    );
    ExitCode::SUCCESS
}
