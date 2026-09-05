//! The checks every provider passes, each a row of the report with what
//! was measured and the bound it was held to: capabilities are coherent;
//! positions are finite and in range; identical requests give identical
//! bits; a grid equals the same cells asked one at a time; reported speeds
//! agree with a central difference; positions are continuous over a short
//! step; an instant outside coverage and a body outside the declared set
//! are reported, never guessed; every declared override answers and
//! agrees with the SDK's own routine within the published bound (the
//! obliquity and nutation against IAU 2006 and 2000B, Delta T against the
//! IERS table, the ayanamsha against the published values at J2000.0,
//! rise and set against the SDK's solver); and, for a provider that can
//! return equatorial coordinates, the SDK's frame completion reproduces
//! the provider's native ecliptic positions.

use std::fmt::Write;
use std::time::Instant;

use serde::Serialize;
use teistro_astro::events::Search;
use teistro_astro::rise_set::Solver;
use teistro_astro::{Completion, DeltaTModel, delta_t, sky, tt_of};
use teistro_core::angle::difference_deg;
use teistro_core::catalogue::Ayanamsha;
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Ut1};
use teistro_core::settings::OverridePolicy;
use teistro_port_ephemeris::{
    Astronomy, Body, Capabilities, CellStatus, Coordinates, CrossingEvent, CrossingRequest,
    EphemerisProvider, Frame, Horizon, HorizonEventKind, HorizonRequest, Identity, Lattice,
    Obliquity, Overrides, PositionColumns, PositionRequest, ProviderError, Quantity, SpeedModel,
    TimeScale,
};

use crate::bench::Row;

/// The bounds the kit holds a provider to; one set, published in
/// `docs/03-design/ephemeris-port-and-adapters.md`, never per provider.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Bounds {
    /// Reported longitude speed against a central difference, degrees per day.
    pub speed_deg_per_day: f64,
    /// Longitude change over the continuity step against speed times the step, degrees.
    pub continuity_deg: f64,
    /// The continuity step, days.
    pub continuity_step_days: f64,
    /// A native obliquity against the SDK's, arcseconds.
    pub obliquity_arcsec: f64,
    /// A native Delta T against the SDK's table, seconds, inside the table's span.
    pub delta_t_seconds: f64,
    /// A native ayanamsha at J2000.0 against the published value, degrees.
    pub ayanamsha_deg: f64,
    /// A native DUT1 against the definition's bound, seconds.
    pub dut1_seconds: f64,
    /// A native rise or set of the Sun against the SDK's solver under the
    /// geometric convention (the disc's centre on the true horizon, no
    /// refraction), seconds: pure geometry on both sides.
    pub rise_set_geometric_seconds: f64,
    /// The same under the almanac's convention (the upper limb with
    /// standard refraction), seconds: the SDK's standard refraction is the
    /// almanac's 34 arcminutes, an engine's is its standard atmosphere, and
    /// the difference grows where the Sun rises at a grazing angle.
    pub rise_set_refracted_seconds: f64,
    /// A native crossing search against the SDK's kernel over the same
    /// provider, seconds: the same positions on both sides, so the
    /// difference is the two searches' own convergence.
    pub crossings_seconds: f64,
    /// Completion of a provider's equatorial output against its own ecliptic output, arcseconds, when the provider's obliquity is used.
    pub completion_native_arcsec: f64,
    /// The same with the SDK's obliquity and nutation, arcseconds.
    pub completion_sdk_arcsec: f64,
}

impl Bounds {
    /// The published bounds.
    pub const DEFAULT: Bounds = Bounds {
        speed_deg_per_day: 2e-3,
        continuity_deg: 5e-6,
        continuity_step_days: 1e-4,
        obliquity_arcsec: 0.01,
        delta_t_seconds: 1.0,
        ayanamsha_deg: 0.1,
        dut1_seconds: 0.9,
        rise_set_geometric_seconds: 1.0,
        rise_set_refracted_seconds: 10.0,
        crossings_seconds: 1.0,
        completion_native_arcsec: 1e-4,
        completion_sdk_arcsec: 0.05,
    };
}

/// One check's outcome.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Check {
    /// The check's name.
    pub name: &'static str,
    /// Whether it passed.
    pub passed: bool,
    /// What was seen, for a reader.
    pub detail: String,
    /// The measured value, when the check measures one.
    pub measured: Option<f64>,
    /// The bound it was held to, when it measures one.
    pub bound: Option<f64>,
    /// The unit of the measurement.
    pub unit: Option<&'static str>,
}

impl Check {
    fn pass(name: &'static str, detail: impl Into<String>) -> Check {
        Check {
            name,
            passed: true,
            detail: detail.into(),
            measured: None,
            bound: None,
            unit: None,
        }
    }

    fn fail(name: &'static str, detail: impl Into<String>) -> Check {
        Check {
            passed: false,
            ..Check::pass(name, detail)
        }
    }

    fn measured(
        name: &'static str,
        measured: f64,
        bound: f64,
        unit: &'static str,
        detail: impl Into<String>,
    ) -> Check {
        Check {
            name,
            passed: measured.is_finite() && measured <= bound,
            detail: detail.into(),
            measured: Some(measured),
            bound: Some(bound),
            unit: Some(unit),
        }
    }

    fn skipped(name: &'static str, detail: impl Into<String>) -> Check {
        Check::pass(name, format!("skipped: {}", detail.into()))
    }

    /// A measurement published without a bound: a classical text's
    /// definition against modern astronomy, or a rule's speed against
    /// the derivative.
    fn informational(
        name: &'static str,
        measured: f64,
        unit: &'static str,
        detail: impl Into<String>,
    ) -> Check {
        Check {
            measured: Some(measured),
            unit: Some(unit),
            ..Check::pass(name, format!("measured, not gated: {}", detail.into()))
        }
    }

    /// Whether the check ran rather than being skipped.
    #[must_use]
    pub fn ran(&self) -> bool {
        !self.detail.starts_with("skipped:")
    }
}

/// The kit's report.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Report {
    /// The provider.
    pub identity: Identity,
    /// Its capabilities as declared.
    pub capabilities: Capabilities,
    /// The bounds applied.
    pub bounds: Bounds,
    /// The checks, in order.
    pub checks: Vec<Check>,
    /// Whether every check passed.
    pub passed: bool,
    /// The kit's run time, milliseconds.
    pub elapsed_ms: f64,
}

impl Report {
    /// The report as a Markdown table.
    #[must_use]
    pub fn markdown(&self) -> String {
        let mut out = format!(
            "provider: {}; {} checks, {}\n\n| check | result | measured | bound | detail |\n|---|---|---:|---:|---|\n",
            self.identity,
            self.checks.len(),
            if self.passed { "all passed" } else { "FAILED" }
        );
        for c in &self.checks {
            let unit = c.unit.unwrap_or("");
            let value = |v: Option<f64>| v.map_or(String::new(), |v| format!("{v:.3e} {unit}"));
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} | {} |",
                c.name,
                if c.passed { "pass" } else { "fail" },
                value(c.measured),
                value(c.bound),
                c.detail
            );
        }
        out
    }

    /// The check with a name.
    #[must_use]
    pub fn check(&self, name: &str) -> Option<&Check> {
        self.checks.iter().find(|c| c.name == name)
    }
}

/// What every runner writes: the kit report and the timing rows of one
/// provider, so a result page reads the same shape for each.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Results {
    /// The provider's short name.
    pub provider: String,
    /// The kit report.
    pub report: Report,
    /// The timing rows.
    pub bench: Vec<Row>,
}

impl Results {
    /// Writes the results as pretty JSON to `<dir>/<provider>.json`.
    ///
    /// # Errors
    ///
    /// When the directory cannot be created or the file written.
    pub fn write(&self, dir: &std::path::Path) -> std::io::Result<std::path::PathBuf> {
        std::fs::create_dir_all(dir)?;
        let path = dir.join(format!("{}.json", self.provider));
        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(&path, format!("{json}\n"))?;
        Ok(path)
    }
}

/// The instants the kit uses: spread over the provider's coverage,
/// clipped to 1900 to 2100 so every provider class is exercised where it
/// is meant to be accurate.
fn instants(capabilities: &Capabilities) -> Vec<f64> {
    let (lo, hi) = capabilities.jd_range;
    let lo = lo.max(2_415_020.5);
    let hi = hi.min(2_488_069.5);
    if lo >= hi {
        return vec![lo];
    }
    (0..5)
        .map(|i| lo + (hi - lo) * f64::from(i) / 4.0)
        .collect()
}

/// The places the rise and set override is checked at: Kathmandu, a
/// northern city inside the polar circle's reach in summer, a southern
/// one; all at sea level, because the SDK's standard refraction is the
/// almanac's sea-level value while an engine's standard atmosphere thins
/// with the observer's height (at Kathmandu's 1400 m the two differ by
/// half a minute of time), and the kit compares conventions both sides
/// implement.
fn places() -> [(&'static str, Place); 3] {
    [
        (
            "Kathmandu",
            Place::new(
                Latitude::literal(27.7172),
                Longitude::literal(85.324),
                Altitude::literal(0.0),
            ),
        ),
        (
            "Reykjavik",
            Place::new(
                Latitude::literal(64.1466),
                Longitude::literal(-21.9426),
                Altitude::literal(0.0),
            ),
        ),
        (
            "Ushuaia",
            Place::new(
                Latitude::literal(-54.8019),
                Longitude::literal(-68.3030),
                Altitude::literal(0.0),
            ),
        ),
    ]
}

fn request<'a>(
    jds: &'a [f64],
    bodies: &'a [Body],
    frame: Frame,
    speeds: bool,
) -> PositionRequest<'a> {
    let request = PositionRequest::new(jds, TimeScale::Ut1, bodies, frame);
    if speeds {
        request
    } else {
        request.without_speeds()
    }
}

/// Runs the kit.
pub fn run<P: EphemerisProvider + ?Sized>(provider: &P, bounds: &Bounds) -> Report {
    let start = Instant::now();
    let capabilities = provider.capabilities();
    let bodies: Vec<Body> = Body::ALL
        .iter()
        .copied()
        .filter(|b| capabilities.has_body(*b))
        .collect();
    let jds = instants(&capabilities);
    let frame = capabilities.native_frame;
    let mut checks = vec![check_capabilities(&capabilities)];

    match provider.positions(&request(&jds, &bodies, frame, true)) {
        Ok(columns) => {
            checks.push(check_finite(&columns, &bodies));
            checks.push(check_determinism(
                provider,
                &jds,
                &bodies,
                frame,
                &columns,
                capabilities.deterministic,
            ));
            checks.push(check_batch_equals_single(
                provider, &jds, &bodies, frame, &columns,
            ));
        }
        Err(error) => checks.push(Check::fail(
            "positions",
            format!("the native-frame request failed: {error}"),
        )),
    }
    checks.push(check_speed(
        provider,
        &jds,
        &bodies,
        frame,
        bounds,
        capabilities.speed_model,
    ));
    checks.push(check_continuity(
        provider,
        &jds,
        &bodies,
        frame,
        bounds,
        capabilities.speed_model,
    ));
    checks.push(check_out_of_range(provider, &capabilities, &bodies, frame));
    checks.push(check_unsupported_body(provider, &capabilities, &jds, frame));
    checks.push(check_obliquity(provider, &capabilities, &jds, bounds));
    checks.push(check_delta_t(provider, &capabilities, bounds));
    checks.push(check_ayanamsha(provider, &capabilities, bounds));
    checks.push(check_dut1(provider, &capabilities, &jds, bounds));
    checks.push(check_rise_set(
        provider,
        &capabilities,
        &jds,
        "override_rise_set_geometric",
        Horizon::CENTRE_NO_REFRACTION,
        bounds.rise_set_geometric_seconds,
    ));
    checks.push(check_rise_set(
        provider,
        &capabilities,
        &jds,
        "override_rise_set_refracted",
        Horizon::UPPER_LIMB_REFRACTION,
        bounds.rise_set_refracted_seconds,
    ));
    checks.push(check_crossings(
        provider,
        &capabilities,
        &jds,
        "override_crossings_longitude",
        Quantity::Longitude(Body::Mercury),
        Lattice::SIGNS,
        120.0,
        bounds.crossings_seconds,
    ));
    checks.push(check_crossings(
        provider,
        &capabilities,
        &jds,
        "override_crossings_composite",
        Quantity::ELONGATION,
        Lattice::TITHIS,
        30.0,
        bounds.crossings_seconds,
    ));
    checks.extend(check_completion(provider, &capabilities, &jds, bounds));

    let passed = checks.iter().all(|c| c.passed);
    Report {
        identity: capabilities.identity.clone(),
        capabilities,
        bounds: bounds.clone(),
        checks,
        passed,
        elapsed_ms: start.elapsed().as_secs_f64() * 1e3,
    }
}

fn check_capabilities(c: &Capabilities) -> Check {
    let mut problems = Vec::new();
    if c.identity.name.is_empty() || c.identity.version.is_empty() {
        problems.push("identity incomplete");
    }
    if c.jd_range.0 >= c.jd_range.1 {
        problems.push("jd range empty");
    }
    if c.bodies.is_empty() {
        problems.push("no bodies");
    }
    if c.has(Overrides::AYANAMSHA) && c.ayanamshas.is_empty() {
        problems.push("ayanamsha override without an ayanamsha list");
    }
    if !c.has(Overrides::AYANAMSHA) && !c.ayanamshas.is_empty() {
        problems.push("an ayanamsha list without the ayanamsha override");
    }
    if problems.is_empty() {
        Check::pass("capabilities", c.describe())
    } else {
        Check::fail("capabilities", problems.join("; "))
    }
}

fn check_finite(columns: &PositionColumns, bodies: &[Body]) -> Check {
    let mut bad = Vec::new();
    for (index, cell) in columns.cells().enumerate() {
        let body = bodies.get(index % bodies.len().max(1)).copied();
        let finite = [
            cell.lon,
            cell.lat,
            cell.dist,
            cell.lon_speed,
            cell.lat_speed,
            cell.dist_speed,
        ]
        .iter()
        .all(|v| v.is_finite());
        let in_range = (0.0..360.0).contains(&cell.lon)
            && (-90.0..=90.0).contains(&cell.lat)
            && (cell.dist > 0.0 || body.is_some_and(|b| !b.has_distance()));
        if !cell.is_ok() || !finite || !in_range {
            bad.push(format!(
                "cell {index} ({:?}): status {:?}",
                body.map(Body::key),
                cell.status
            ));
        }
    }
    if bad.is_empty() {
        Check::pass(
            "positions_finite_in_range",
            format!("{} cells", columns.len()),
        )
    } else {
        Check::fail("positions_finite_in_range", bad.join("; "))
    }
}

fn check_determinism<P: EphemerisProvider + ?Sized>(
    provider: &P,
    jds: &[f64],
    bodies: &[Body],
    frame: Frame,
    first: &PositionColumns,
    declared: bool,
) -> Check {
    match provider.positions(&request(jds, bodies, frame, true)) {
        Ok(second) if first.bit_identical(&second) => {
            Check::pass("determinism", "identical bits on a repeated request")
        }
        Ok(_) if !declared => Check::pass(
            "determinism",
            "differs on repetition, and the provider declares itself non-deterministic",
        ),
        Ok(_) => Check::fail(
            "determinism",
            "declared deterministic, but a repeated request differs",
        ),
        Err(error) => Check::fail(
            "determinism",
            format!("the repeated request failed: {error}"),
        ),
    }
}

fn check_batch_equals_single<P: EphemerisProvider + ?Sized>(
    provider: &P,
    jds: &[f64],
    bodies: &[Body],
    frame: Frame,
    grid: &PositionColumns,
) -> Check {
    for (jd_index, jd) in jds.iter().enumerate() {
        for (body_index, body) in bodies.iter().enumerate() {
            let one = [*jd];
            let single = match provider.positions(&request(
                &one,
                core::slice::from_ref(body),
                frame,
                true,
            )) {
                Ok(columns) => columns,
                Err(error) => return Check::fail("batch_equals_single", format!("{error}")),
            };
            let (Some(a), Some(b)) = (grid.at(jd_index, body_index), single.cell(0)) else {
                return Check::fail("batch_equals_single", "a cell is missing");
            };
            let same = [
                (a.lon, b.lon),
                (a.lat, b.lat),
                (a.dist, b.dist),
                (a.lon_speed, b.lon_speed),
            ]
            .iter()
            .all(|(x, y)| x.to_bits() == y.to_bits());
            if !same || a.status != b.status {
                return Check::fail(
                    "batch_equals_single",
                    format!("{} at JD {jd}: grid {} single {}", body.key(), a.lon, b.lon),
                );
            }
        }
    }
    Check::pass(
        "batch_equals_single",
        format!("{} cells identical", grid.len()),
    )
}

fn check_speed<P: EphemerisProvider + ?Sized>(
    provider: &P,
    jds: &[f64],
    bodies: &[Body],
    frame: Frame,
    bounds: &Bounds,
    speed_model: SpeedModel,
) -> Check {
    let sample: Vec<Body> = bodies
        .iter()
        .copied()
        .filter(|b| matches!(b, Body::Sun | Body::Moon | Body::Mars | Body::MeanNode))
        .collect();
    if sample.is_empty() {
        return Check::skipped("speed_consistency", "no sampled body offered");
    }
    let h = 0.05;
    let mut worst = 0.0f64;
    let mut where_ = String::new();
    for jd in jds {
        let three = [jd - h, *jd, jd + h];
        let columns = match provider.positions(&request(&three, &sample, frame, true)) {
            Ok(c) => c,
            Err(error) => return Check::fail("speed_consistency", format!("{error}")),
        };
        for (body_index, body) in sample.iter().enumerate() {
            let (Some(before), Some(at), Some(after)) = (
                columns.at(0, body_index),
                columns.at(1, body_index),
                columns.at(2, body_index),
            ) else {
                continue;
            };
            if !at.is_ok() {
                continue;
            }
            let difference = difference_deg(after.lon, before.lon) / (2.0 * h);
            let error = (difference - at.lon_speed).abs();
            if error > worst {
                worst = error;
                where_ = format!("{} at JD {jd}", body.key());
            }
        }
    }
    if speed_model == SpeedModel::Rule {
        // A text's rule for the daily motion is the speed its tradition
        // uses; its distance from the derivative of the text's places is
        // published.
        return Check::informational(
            "speed_consistency",
            worst,
            "deg/day",
            format!("the provider's speeds follow its rule, not the derivative; worst {where_}"),
        );
    }
    Check::measured(
        "speed_consistency",
        worst,
        bounds.speed_deg_per_day,
        "deg/day",
        format!("worst {where_}"),
    )
}

fn check_dut1<P: EphemerisProvider + ?Sized>(
    provider: &P,
    capabilities: &Capabilities,
    jds: &[f64],
    bounds: &Bounds,
) -> Check {
    if !capabilities.has(Overrides::DUT1) {
        return Check::skipped("override_dut1", "not declared");
    }
    let mut worst = 0.0f64;
    for jd in jds {
        match provider.dut1_seconds(*jd) {
            Ok(value) if value.is_finite() => worst = worst.max(value.abs()),
            Ok(value) => {
                return Check::fail("override_dut1", format!("declared, but answered {value}"));
            }
            Err(error) => {
                return Check::fail("override_dut1", format!("declared, but failed: {error}"));
            }
        }
    }
    Check::measured(
        "override_dut1",
        worst,
        bounds.dut1_seconds,
        "s",
        "the largest native DUT1 on the kit's instants against the definition's bound",
    )
}

/// Positions are continuous over the step: the change over it agrees
/// with the reported speed times the step or, for a provider whose
/// speeds follow a rule, with the change over the step before (the
/// second difference vanishes), which catches a jump without appeal to
/// the speed.
fn check_continuity<P: EphemerisProvider + ?Sized>(
    provider: &P,
    jds: &[f64],
    bodies: &[Body],
    frame: Frame,
    bounds: &Bounds,
    speed_model: SpeedModel,
) -> Check {
    let h = bounds.continuity_step_days;
    let mut worst = 0.0f64;
    let mut where_ = String::new();
    for jd in jds {
        let three = [jd - h, *jd, jd + h];
        let columns = match provider.positions(&request(&three, bodies, frame, true)) {
            Ok(c) => c,
            Err(error) => return Check::fail("continuity", format!("{error}")),
        };
        for (body_index, body) in bodies.iter().enumerate() {
            let (Some(before), Some(at), Some(after)) = (
                columns.at(0, body_index),
                columns.at(1, body_index),
                columns.at(2, body_index),
            ) else {
                continue;
            };
            if !before.is_ok() || !at.is_ok() || !after.is_ok() {
                continue;
            }
            let forward = difference_deg(after.lon, at.lon);
            let error = match speed_model {
                SpeedModel::Derivative => (forward - at.lon_speed * h).abs(),
                SpeedModel::Rule => (forward - difference_deg(at.lon, before.lon)).abs(),
            };
            if error > worst {
                worst = error;
                where_ = format!("{} at JD {jd}", body.key());
            }
        }
    }
    Check::measured(
        "continuity",
        worst,
        bounds.continuity_deg,
        "deg",
        format!("worst {where_}"),
    )
}

fn check_out_of_range<P: EphemerisProvider + ?Sized>(
    provider: &P,
    capabilities: &Capabilities,
    bodies: &[Body],
    frame: Frame,
) -> Check {
    let outside = [capabilities.jd_range.0 - 100_000.0];
    let sample: Vec<Body> = bodies.iter().copied().take(1).collect();
    match provider.positions(&request(&outside, &sample, frame, false)) {
        Err(error) => Check::pass("out_of_range_reported", format!("refused: {error}")),
        Ok(columns)
            if columns
                .status
                .iter()
                .all(|s| matches!(s, CellStatus::OutOfRange | CellStatus::DataMissing)) =>
        {
            Check::pass("out_of_range_reported", "reported per cell")
        }
        Ok(columns) => Check::fail(
            "out_of_range_reported",
            format!(
                "an instant outside coverage came back as {:?}",
                columns.status.first()
            ),
        ),
    }
}

fn check_unsupported_body<P: EphemerisProvider + ?Sized>(
    provider: &P,
    capabilities: &Capabilities,
    jds: &[f64],
    frame: Frame,
) -> Check {
    let Some(missing) = Body::ALL
        .iter()
        .copied()
        .find(|b| !capabilities.has_body(*b))
    else {
        return Check::skipped("unsupported_body_reported", "every body is offered");
    };
    let one = [jds.first().copied().unwrap_or(2_451_545.0)];
    match provider.positions(&request(&one, &[missing], frame, false)) {
        Err(error) => Check::pass(
            "unsupported_body_reported",
            format!("{}: refused: {error}", missing.key()),
        ),
        Ok(columns)
            if columns
                .status
                .iter()
                .all(|s| *s == CellStatus::UnsupportedBody) =>
        {
            Check::pass(
                "unsupported_body_reported",
                format!("{}: reported per cell", missing.key()),
            )
        }
        Ok(columns) => Check::fail(
            "unsupported_body_reported",
            format!(
                "{} came back as {:?}",
                missing.key(),
                columns.status.first()
            ),
        ),
    }
}

/// The SDK's obliquity at a UT1 instant, through the provider's Delta T
/// when it has one (so the comparison isolates the obliquity) and the
/// SDK's table otherwise.
fn sdk_obliquity<P: EphemerisProvider + ?Sized>(provider: &P, jd_ut1: f64) -> Option<Obliquity> {
    let ut1 = JulianDay::<Ut1>::try_new(jd_ut1).ok()?;
    let tt = match provider.delta_t_seconds(jd_ut1) {
        Ok(seconds) => JulianDay::try_new(jd_ut1 + seconds / 86_400.0).ok()?,
        Err(_) => tt_of(ut1, DeltaTModel::TableThenModel).ok()?.0,
    };
    Some(sky::obliquity(tt))
}

fn check_obliquity<P: EphemerisProvider + ?Sized>(
    provider: &P,
    capabilities: &Capabilities,
    jds: &[f64],
    bounds: &Bounds,
) -> Check {
    if !capabilities.has(Overrides::OBLIQUITY) {
        return Check::skipped("override_obliquity", "not declared");
    }
    let mut worst = 0.0f64;
    for jd in jds {
        let native = match provider.obliquity(*jd, TimeScale::Ut1) {
            Ok(o) => o,
            Err(error) => {
                return Check::fail(
                    "override_obliquity",
                    format!("declared, but failed: {error}"),
                );
            }
        };
        let Some(sdk) = sdk_obliquity(provider, *jd) else {
            return Check::fail("override_obliquity", format!("JD {jd} is not an instant"));
        };
        let error = ((native.true_deg - sdk.true_deg).abs())
            .max((native.mean_deg - sdk.mean_deg).abs())
            * 3600.0;
        worst = worst.max(error);
    }
    if capabilities.astronomy == Astronomy::Classical {
        return Check::informational(
            "override_obliquity",
            worst,
            "arcsec",
            "the text's obliquity against IAU 2006 and IAU 2000B",
        );
    }
    Check::measured(
        "override_obliquity",
        worst,
        bounds.obliquity_arcsec,
        "arcsec",
        "native against IAU 2006 and IAU 2000B",
    )
}

fn check_delta_t<P: EphemerisProvider + ?Sized>(
    provider: &P,
    capabilities: &Capabilities,
    bounds: &Bounds,
) -> Check {
    if !capabilities.has(Overrides::DELTA_T) {
        return Check::skipped("override_delta_t", "not declared");
    }
    let Some((table_lo, table_hi)) = teistro_astro::delta_t::table_span() else {
        return Check::skipped("override_delta_t", "the SDK has no table");
    };
    let (lo, hi) = (
        table_lo.max(capabilities.jd_range.0),
        table_hi.min(capabilities.jd_range.1),
    );
    if lo >= hi {
        return Check::skipped("override_delta_t", "the coverage misses the table's span");
    }
    let mut worst = (0.0f64, lo);
    for i in 0..=10 {
        let jd = lo + (hi - lo) * f64::from(i) / 10.0;
        let sdk = match JulianDay::<Ut1>::try_new(jd)
            .map_err(teistro_core::Error::from)
            .and_then(|ut1| delta_t(ut1, DeltaTModel::TableThenModel))
        {
            Ok(value) => value.seconds,
            Err(error) => return Check::fail("override_delta_t", format!("{error}")),
        };
        match provider.delta_t_seconds(jd) {
            Ok(native) => {
                let gap = (native - sdk).abs();
                if gap > worst.0 {
                    worst = (gap, jd);
                }
            }
            Err(error) => {
                return Check::fail("override_delta_t", format!("declared, but failed: {error}"));
            }
        }
    }
    Check::measured(
        "override_delta_t",
        worst.0,
        bounds.delta_t_seconds,
        "s",
        format!(
            "native against the IERS table over JD {lo:.1} to {hi:.1}; worst at JD {}",
            worst.1
        ),
    )
}

fn check_ayanamsha<P: EphemerisProvider + ?Sized>(
    provider: &P,
    capabilities: &Capabilities,
    bounds: &Bounds,
) -> Check {
    const BURGESS_1860: f64 = 2_400_410.714;
    if !capabilities.has(Overrides::AYANAMSHA) {
        return Check::skipped("override_ayanamsha", "not declared");
    }
    // The published values (Lahiri at J2000.0 from the Indian Astronomical
    // Ephemeris; Raman and Krishnamurti at J2000.0 from their authors'
    // tables), as capability honesty rather than accuracy. The
    // SURYASIDDHANTA member means two things: for a classical astronomy it
    // is the text's own reckoning, 20°24′39″ at Burgess's instant (midnight
    // of 1 January 1860 at Washington, his worked example under III.9 to
    // 12); for a modern one it is the catalogued epoch definition (0° at
    // the text's zero of 499 CE, carried by precession), which the SDK
    // computes and which stands about 18.94° at the same instant.
    let suryasiddhanta_expected = if capabilities.astronomy == Astronomy::Classical {
        Some(20.0 + 24.0 / 60.0 + 39.0 / 3600.0)
    } else {
        teistro_astro::ayanamsha::mean_deg(
            &Ayanamsha::Suryasiddhanta.into(),
            JulianDay::literal(BURGESS_1860),
            teistro_astro::precession::PrecessionModel::Vondrak2011,
            DeltaTModel::TableThenModel,
        )
        .ok()
    };
    let expected = [
        (
            Ayanamsha::Lahiri,
            teistro_astro::iau::DJ00,
            Some(23.85),
            "J2000.0",
        ),
        (
            Ayanamsha::Raman,
            teistro_astro::iau::DJ00,
            Some(22.40),
            "J2000.0",
        ),
        (
            Ayanamsha::Krishnamurti,
            teistro_astro::iau::DJ00,
            Some(23.76),
            "J2000.0",
        ),
        (
            Ayanamsha::Suryasiddhanta,
            BURGESS_1860,
            suryasiddhanta_expected,
            "Burgess's 1860 instant",
        ),
    ];
    let mut worst = 0.0f64;
    let mut details = Vec::new();
    for (ayanamsha, jd, value, at) in expected {
        let (true, Some(value)) = (capabilities.has_ayanamsha(ayanamsha), value) else {
            continue;
        };
        match provider.ayanamsha_deg(jd, TimeScale::Tt, ayanamsha) {
            Ok(native) => {
                worst = worst.max((native - value).abs());
                details.push(format!("{} at {at}: {native:.4}", ayanamsha.key()));
            }
            Err(error) => {
                return Check::fail(
                    "override_ayanamsha",
                    format!("declared, but failed: {error}"),
                );
            }
        }
    }
    if details.is_empty() {
        return Check::skipped(
            "override_ayanamsha",
            "none of the published ayanamshas is offered",
        );
    }
    Check::measured(
        "override_ayanamsha",
        worst,
        bounds.ayanamsha_deg,
        "deg",
        details.join(", "),
    )
}

/// The native rise and set of the Sun against the SDK's solver over the
/// same provider, under one horizon convention, at three places, on the
/// kit's instants.
/// A native crossing search against the SDK's kernel over the same
/// provider: the same events, in order, at the same boundaries and
/// directions, within the bound in seconds.
#[allow(
    clippy::too_many_arguments,
    reason = "a check is named, quantified and bounded by its caller, as the rise and set checks are"
)]
fn check_crossings<P: EphemerisProvider + ?Sized>(
    provider: &P,
    capabilities: &Capabilities,
    jds: &[f64],
    name: &'static str,
    quantity: Quantity,
    lattice: Lattice,
    window_days: f64,
    bound_seconds: f64,
) -> Check {
    if !capabilities.has(Overrides::CROSSINGS) {
        return Check::skipped(name, "not declared");
    }
    let (first, second) = quantity.bodies();
    if !capabilities.has_body(first) || second.is_some_and(|body| !capabilities.has_body(body)) {
        return Check::skipped(name, format!("{quantity} is not offered"));
    }
    let Some(from) = jds
        .first()
        .and_then(|jd| JulianDay::<Ut1>::try_new(*jd).ok())
    else {
        return Check::skipped(name, "no instant to start from");
    };
    let Ok(to) = from.plus_days(window_days) else {
        return Check::skipped(name, "the window runs off the calendar");
    };
    let request = CrossingRequest {
        quantity,
        lattice,
        from,
        to,
        tolerance_days: 1e-7,
        frame: Frame::CANONICAL,
        observer: None,
    };
    let native = match provider.crossings(&request) {
        Ok(events) => events,
        Err(ProviderError::Unsupported { what }) => {
            return Check::skipped(name, format!("the provider does not offer {what}"));
        }
        Err(error) => return Check::fail(name, format!("declared, but failed: {error}")),
    };
    let completion = Completion::new(
        provider,
        OverridePolicy::SdkOnly,
        DeltaTModel::TableThenModel,
    );
    let longitudes = completion.longitudes(Frame::CANONICAL);
    let sdk = match Search::new(&longitudes, quantity, lattice).between(from, to) {
        Ok(events) => events,
        Err(error) => return Check::fail(name, format!("the SDK's kernel failed: {error}")),
    };
    if native.len() != sdk.len() {
        return Check::fail(
            name,
            format!(
                "the provider found {} events of {quantity} over {window_days} days where the SDK's kernel found {}",
                native.len(),
                sdk.len()
            ),
        );
    }
    let mut worst = 0.0f64;
    for (theirs, ours) in native.iter().zip(&sdk) {
        if theirs.direction != ours.direction
            || difference_deg(theirs.boundary_deg, ours.boundary_deg).abs() > 1e-6
        {
            return Check::fail(
                name,
                format!(
                    "at JD {}: the provider crossed {}° {:?} where the SDK's kernel crossed {}° {:?}",
                    theirs.instant.get(),
                    theirs.boundary_deg,
                    theirs.direction,
                    ours.boundary_deg,
                    ours.direction
                ),
            );
        }
        worst = worst.max((theirs.instant.get() - ours.instant.get()).abs() * 86_400.0);
    }
    if native.is_empty() {
        return Check::pass(
            name,
            format!("no crossing of {quantity} in {window_days} days, on either side"),
        );
    }
    Check::measured(
        name,
        worst,
        bound_seconds,
        "s",
        format!(
            "{} events of {quantity} over {window_days} days; worst {worst:.4} s apart",
            native.len()
        ),
    )
}

fn check_rise_set<P: EphemerisProvider + ?Sized>(
    provider: &P,
    capabilities: &Capabilities,
    jds: &[f64],
    name: &'static str,
    horizon: Horizon,
    bound: f64,
) -> Check {
    if !capabilities.has(Overrides::RISE_SET) {
        return Check::skipped(name, "not declared");
    }
    if !capabilities.has_body(Body::Sun) {
        return Check::skipped(name, "the Sun is not offered");
    }
    let completion = Completion::new(
        provider,
        OverridePolicy::PreferNative,
        DeltaTModel::TableThenModel,
    );
    let mut worst = (0.0f64, String::new());
    let mut compared = 0u32;
    for (place_name, place) in places() {
        let solver = Solver::new(
            &completion,
            Body::Sun,
            place,
            horizon,
            DeltaTModel::TableThenModel,
        );
        for jd in jds {
            let Ok(from) = JulianDay::<Ut1>::try_new(*jd) else {
                continue;
            };
            for kind in [HorizonEventKind::Rise, HorizonEventKind::Set] {
                let native = match provider.horizon_event(&HorizonRequest {
                    body: Body::Sun,
                    kind,
                    place,
                    from,
                    window_days: 1.0,
                    horizon,
                }) {
                    Ok(found) => found,
                    Err(ProviderError::Unsupported { what }) => {
                        return Check::skipped(name, format!("the provider does not offer {what}"));
                    }
                    Err(error) => {
                        return Check::fail(
                            name,
                            format!("declared, but failed at {place_name}: {error}"),
                        );
                    }
                };
                let sdk = match solver.event(kind, from, 1.0) {
                    Ok(found) => found.map(|e| e.instant),
                    Err(error) => {
                        return Check::fail(
                            name,
                            format!("the SDK's solver failed at {place_name}: {error}"),
                        );
                    }
                };
                match (native, sdk) {
                    (Some(a), Some(b)) => {
                        compared += 1;
                        let gap = (a.get() - b.get()).abs() * 86_400.0;
                        if gap > worst.0 {
                            worst = (gap, format!("{kind} at {place_name} from JD {jd}"));
                        }
                    }
                    (None, None) => compared += 1,
                    (Some(a), None) | (None, Some(a)) => {
                        return Check::fail(
                            name,
                            format!(
                                "{kind} at {place_name} from JD {jd}: one side found {a} and the other nothing"
                            ),
                        );
                    }
                }
            }
        }
    }
    if capabilities.astronomy == Astronomy::Classical {
        return Check::informational(
            name,
            worst.0,
            "s",
            format!(
                "{horizon}: the text's sunrise against hour-angle geometry over its own places; {compared} events compared; worst {}",
                worst.1
            ),
        );
    }
    Check::measured(
        name,
        worst.0,
        bound,
        "s",
        format!(
            "{horizon}: {compared} events compared at sea level; worst {}",
            worst.1
        ),
    )
}

/// Completion of the provider's equatorial output back to its native
/// ecliptic frame, once with the provider's obliquity and once with the
/// SDK's, compared with the provider's own ecliptic output for the Sun and
/// the Moon.
fn check_completion<P: EphemerisProvider + ?Sized>(
    provider: &P,
    capabilities: &Capabilities,
    jds: &[f64],
    bounds: &Bounds,
) -> Vec<Check> {
    let native = capabilities.native_frame;
    if native.coordinates != Coordinates::Ecliptic {
        return vec![Check::skipped(
            "completion_native",
            "the native frame is not ecliptic",
        )];
    }
    let bodies: Vec<Body> = [Body::Sun, Body::Moon]
        .into_iter()
        .filter(|b| capabilities.has_body(*b))
        .collect();
    let equatorial = native.with_coordinates(Coordinates::Equatorial);
    let reference = match provider.positions(&request(jds, &bodies, native, true)) {
        Ok(c) => c,
        Err(error) => return vec![Check::fail("completion_native", format!("{error}"))],
    };
    if provider
        .positions(&request(jds, &bodies, equatorial, true))
        .is_err()
    {
        return vec![Check::skipped(
            "completion_native",
            "the provider cannot return equatorial coordinates",
        )];
    }
    let mut out = Vec::new();
    for (name, policy, bound) in [
        (
            "completion_native",
            OverridePolicy::PreferNative,
            bounds.completion_native_arcsec,
        ),
        (
            "completion_sdk",
            OverridePolicy::SdkOnly,
            bounds.completion_sdk_arcsec,
        ),
    ] {
        // The provider seen as equatorial-native and refusing every other
        // frame, so completion has to rotate back to the ecliptic frame the
        // reference columns are in.
        let proxy = Refusing::new(provider, equatorial);
        let completion = Completion::new(&proxy, policy, DeltaTModel::TableThenModel);
        match completion.positions(&request(jds, &bodies, native, true)) {
            Ok(done) => {
                let mut worst = 0.0f64;
                for (index, cell) in done.columns.cells().enumerate() {
                    let Some(expected) = reference.cell(index) else {
                        continue;
                    };
                    let dlon = difference_deg(cell.lon, expected.lon).abs() * 3600.0;
                    let dlat = (cell.lat - expected.lat).abs() * 3600.0;
                    worst = worst.max(dlon).max(dlat);
                }
                out.push(Check::measured(
                    name,
                    worst,
                    bound,
                    "arcsec",
                    format!("steps {}", done.step_keys().join(" ")),
                ));
            }
            Err(error) => out.push(Check::fail(name, format!("{error}"))),
        }
    }
    out
}

/// A provider seen through one native frame and refusing every other:
/// the same provider, its native frame declared as given, every request
/// for another frame answered `Unsupported`. What the kit and the runner
/// put in front of a provider so the SDK's completion has to do the work
/// even when the provider could have answered the frame itself.
#[derive(Debug)]
pub struct Refusing<'a, P: EphemerisProvider + ?Sized> {
    inner: &'a P,
    frame: Frame,
}

impl<'a, P: EphemerisProvider + ?Sized> Refusing<'a, P> {
    /// The provider behind a native frame it answers alone.
    pub const fn new(inner: &'a P, frame: Frame) -> Refusing<'a, P> {
        Refusing { inner, frame }
    }
}

impl<P: EphemerisProvider + ?Sized> EphemerisProvider for Refusing<'_, P> {
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            native_frame: self.frame,
            ..self.inner.capabilities()
        }
    }

    fn positions(&self, request: &PositionRequest<'_>) -> Result<PositionColumns, ProviderError> {
        if request.frame != self.frame {
            return Err(ProviderError::unsupported(format!(
                "a frame other than {}",
                self.frame
            )));
        }
        self.inner.positions(request)
    }

    fn obliquity(&self, jd: f64, scale: TimeScale) -> Result<Obliquity, ProviderError> {
        self.inner.obliquity(jd, scale)
    }

    fn delta_t_seconds(&self, jd_ut1: f64) -> Result<f64, ProviderError> {
        self.inner.delta_t_seconds(jd_ut1)
    }

    fn ayanamsha_deg(
        &self,
        jd: f64,
        scale: TimeScale,
        ayanamsha: Ayanamsha,
    ) -> Result<f64, ProviderError> {
        self.inner.ayanamsha_deg(jd, scale, ayanamsha)
    }

    fn horizon_event(
        &self,
        request: &HorizonRequest,
    ) -> Result<Option<JulianDay<Ut1>>, ProviderError> {
        self.inner.horizon_event(request)
    }

    fn crossings(&self, request: &CrossingRequest) -> Result<Vec<CrossingEvent>, ProviderError> {
        self.inner.crossings(request)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use teistro_port_ephemeris::TestProvider;

    use super::*;

    #[test]
    fn the_test_provider_passes_the_kit() {
        let report = run(&TestProvider::new(), &Bounds::DEFAULT);
        let failed: Vec<&Check> = report.checks.iter().filter(|c| !c.passed).collect();
        assert!(failed.is_empty(), "{}", report.markdown());
        assert!(!report.check("completion_native").unwrap().ran());
        assert!(!report.check("override_rise_set_geometric").unwrap().ran());
        assert!(!report.check("override_crossings_longitude").unwrap().ran());
        assert!(!report.check("override_dut1").unwrap().ran());
        assert!(report.check("speed_consistency").unwrap().ran());
        assert_eq!(report.checks.len(), 17);
        assert!(report.markdown().contains("all passed"));
        let results = Results {
            provider: String::from("test-provider"),
            report,
            bench: Vec::new(),
        };
        let dir = std::env::temp_dir().join("teistro-ephemeris-kit-results");
        let path = results.write(&dir).unwrap();
        assert!(
            std::fs::read_to_string(path)
                .unwrap()
                .contains("\"passed\": true")
        );
    }

    /// A provider that declares every override and answers each one with
    /// the SDK's own routine, so every check runs and every bound is met.
    struct Honest;

    impl EphemerisProvider for Honest {
        fn capabilities(&self) -> Capabilities {
            Capabilities {
                overrides: Overrides::OBLIQUITY
                    .with(Overrides::DELTA_T)
                    .with(Overrides::AYANAMSHA)
                    .with(Overrides::RISE_SET)
                    .with(Overrides::CROSSINGS)
                    .with(Overrides::DUT1),
                ayanamshas: vec![Ayanamsha::Lahiri],
                ..TestProvider::new().capabilities()
            }
        }

        fn positions(
            &self,
            request: &PositionRequest<'_>,
        ) -> Result<PositionColumns, ProviderError> {
            TestProvider::new().positions(request)
        }

        fn obliquity(&self, jd: f64, scale: TimeScale) -> Result<Obliquity, ProviderError> {
            let tt = match scale {
                TimeScale::Tt => jd,
                TimeScale::Ut1 => jd + self.delta_t_seconds(jd)? / 86_400.0,
            };
            Ok(sky::obliquity(JulianDay::literal(tt)))
        }

        fn dut1_seconds(&self, _jd_utc: f64) -> Result<f64, ProviderError> {
            Ok(0.0)
        }

        fn delta_t_seconds(&self, jd_ut1: f64) -> Result<f64, ProviderError> {
            delta_t(JulianDay::literal(jd_ut1), DeltaTModel::TableThenModel)
                .map(|d| d.seconds)
                .map_err(|e| ProviderError::invalid(e.to_string()))
        }

        fn ayanamsha_deg(
            &self,
            _jd: f64,
            _scale: TimeScale,
            _a: Ayanamsha,
        ) -> Result<f64, ProviderError> {
            Ok(23.85)
        }

        fn horizon_event(
            &self,
            request: &HorizonRequest,
        ) -> Result<Option<JulianDay<Ut1>>, ProviderError> {
            let completion =
                Completion::new(self, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
            let solver = Solver::new(
                &completion,
                request.body,
                request.place,
                request.horizon,
                DeltaTModel::TableThenModel,
            );
            solver
                .event(request.kind, request.from, request.window_days)
                .map(|found| found.map(|e| e.instant))
                .map_err(|e| ProviderError::invalid(e.to_string()))
        }

        fn crossings(
            &self,
            request: &CrossingRequest,
        ) -> Result<Vec<CrossingEvent>, ProviderError> {
            let completion =
                Completion::new(self, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
            let longitudes = completion.longitudes(request.frame);
            Search::new(&longitudes, request.quantity, request.lattice)
                .with_tolerance_days(request.tolerance_days)
                .between(request.from, request.to)
                .map_err(|e| ProviderError::invalid(e.to_string()))
        }
    }

    #[test]
    fn an_honest_provider_runs_every_override_check() {
        let report = run(&Honest, &Bounds::DEFAULT);
        assert!(report.passed, "{}", report.markdown());
        for name in [
            "override_obliquity",
            "override_delta_t",
            "override_ayanamsha",
            "override_dut1",
            "override_rise_set_geometric",
            "override_rise_set_refracted",
            "override_crossings_longitude",
            "override_crossings_composite",
        ] {
            let check = report.check(name).unwrap();
            assert!(check.ran(), "{name}");
            assert_eq!(check.measured, Some(0.0), "{name}: {}", check.detail);
        }
    }

    /// A provider whose declared obliquity is wrong by a degree.
    struct Dishonest;

    impl EphemerisProvider for Dishonest {
        fn capabilities(&self) -> Capabilities {
            Capabilities {
                overrides: Overrides::OBLIQUITY.with(Overrides::AYANAMSHA),
                ayanamshas: Vec::new(),
                ..TestProvider::new().capabilities()
            }
        }

        fn positions(
            &self,
            request: &PositionRequest<'_>,
        ) -> Result<PositionColumns, ProviderError> {
            TestProvider::new().positions(request)
        }

        fn obliquity(&self, _jd: f64, _scale: TimeScale) -> Result<Obliquity, ProviderError> {
            Ok(Obliquity {
                mean_deg: 24.4,
                true_deg: 24.4,
                nutation_lon_deg: 0.0,
                nutation_obl_deg: 0.0,
            })
        }
    }

    #[test]
    fn a_dishonest_provider_fails_by_name() {
        let report = run(&Dishonest, &Bounds::DEFAULT);
        assert!(!report.passed);
        assert!(!report.check("capabilities").unwrap().passed);
        assert!(!report.check("override_obliquity").unwrap().passed);
        assert!(report.markdown().contains("FAILED"));
    }
}
