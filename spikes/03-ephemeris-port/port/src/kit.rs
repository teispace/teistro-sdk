//! The provider conformance kit: the checks every adapter and every
//! built-in tier must pass, with a machine-readable report. It runs
//! against anything that implements the port, native or through a vtable.
//!
//! Checks, each a row of the report with what was measured and the bound:
//! capabilities are coherent; positions are finite and in range; reported
//! speeds agree with a central difference; positions are continuous over
//! a short step; identical requests give identical bits; a grid equals
//! the same cells asked one at a time; an instant outside coverage and a
//! body outside the declared set are reported, never guessed; every
//! declared override answers and agrees with the SDK's own routine within
//! a published bound; and, for a provider that can return equatorial
//! coordinates, the SDK's frame completion reproduces the provider's
//! native ecliptic positions.

use std::fmt::Write;
use std::time::Instant;

use serde::Serialize;

use crate::astro;
use crate::completion::Completion;
use crate::model::{
    AyanamshaId, Body, Capabilities, CellStatus, Coordinates, Frame, Identity, OverridePolicy,
    Overrides, PositionColumns, PositionRequest, TimeScale,
};
use crate::provider::EphemerisProvider;

/// The bounds the kit holds a provider to; one set, published, never per
/// provider.
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
    /// A native Delta T against the SDK's model, seconds.
    pub delta_t_seconds: f64,
    /// Completion of a provider's equatorial output against its own ecliptic output, arcseconds, when the provider's obliquity is used.
    pub completion_native_arcsec: f64,
    /// The same with the SDK's obliquity and nutation, arcseconds.
    pub completion_sdk_arcsec: f64,
}

impl Bounds {
    /// The published bounds of this spike.
    pub const DEFAULT: Bounds = Bounds {
        speed_deg_per_day: 2e-3,
        continuity_deg: 5e-6,
        continuity_step_days: 1e-4,
        obliquity_arcsec: 0.01,
        delta_t_seconds: 5.0,
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
            "provider: {} {} ({}); {} checks, {}\n\n| check | result | measured | bound | detail |\n|---|---|---:|---:|---|\n",
            self.identity.name,
            self.identity.version,
            self.identity.data_version,
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

fn request<'a>(
    jds: &'a [f64],
    bodies: &'a [Body],
    frame: Frame,
    speeds: bool,
) -> PositionRequest<'a> {
    PositionRequest {
        jds,
        scale: TimeScale::Ut1,
        bodies,
        frame,
        observer: None,
        speeds,
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
    checks.push(check_speed(provider, &jds, &bodies, frame, bounds));
    checks.push(check_continuity(provider, &jds, &bodies, frame, bounds));
    checks.push(check_out_of_range(provider, &capabilities, &bodies, frame));
    checks.push(check_unsupported_body(provider, &capabilities, &jds, frame));
    checks.push(check_obliquity(provider, &capabilities, &jds, bounds));
    checks.push(check_delta_t(provider, &capabilities, bounds));
    checks.push(check_ayanamsha(provider, &capabilities, &jds));
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
    if c.overrides.contains(Overrides::AYANAMSHA) && c.ayanamshas.is_empty() {
        problems.push("ayanamsha override without an ayanamsha list");
    }
    if problems.is_empty() {
        Check::pass(
            "capabilities",
            format!(
                "{} bodies, JD {:.1} to {:.1}, overrides [{}]",
                c.bodies.len(),
                c.jd_range.0,
                c.jd_range.1,
                c.overrides
            ),
        )
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
        if cell.status != CellStatus::Ok || !finite || !in_range {
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
            if at.status != CellStatus::Ok {
                continue;
            }
            let difference = astro::angle_difference_deg(after.lon, before.lon) / (2.0 * h);
            let error = (difference - at.lon_speed).abs();
            if error > worst {
                worst = error;
                where_ = format!("{} at JD {jd}", body.key());
            }
        }
    }
    Check::measured(
        "speed_consistency",
        worst,
        bounds.speed_deg_per_day,
        "deg/day",
        format!("worst {where_}"),
    )
}

fn check_continuity<P: EphemerisProvider + ?Sized>(
    provider: &P,
    jds: &[f64],
    bodies: &[Body],
    frame: Frame,
    bounds: &Bounds,
) -> Check {
    let h = bounds.continuity_step_days;
    let mut worst = 0.0f64;
    let mut where_ = String::new();
    for jd in jds {
        let two = [*jd, jd + h];
        let columns = match provider.positions(&request(&two, bodies, frame, true)) {
            Ok(c) => c,
            Err(error) => return Check::fail("continuity", format!("{error}")),
        };
        for (body_index, body) in bodies.iter().enumerate() {
            let (Some(a), Some(b)) = (columns.at(0, body_index), columns.at(1, body_index)) else {
                continue;
            };
            if a.status != CellStatus::Ok || b.status != CellStatus::Ok {
                continue;
            }
            let predicted = a.lon_speed * h;
            let actual = astro::angle_difference_deg(b.lon, a.lon);
            let error = (actual - predicted).abs();
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

fn check_obliquity<P: EphemerisProvider + ?Sized>(
    provider: &P,
    capabilities: &Capabilities,
    jds: &[f64],
    bounds: &Bounds,
) -> Check {
    if !capabilities.overrides.contains(Overrides::OBLIQUITY) {
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
        let delta_t = provider
            .delta_t_seconds(*jd)
            .unwrap_or_else(|_| astro::delta_t_seconds_approx(*jd));
        let sdk = astro::obliquity(astro::tt_from_ut1(*jd, delta_t));
        let error = ((native.true_deg - sdk.true_deg).abs())
            .max((native.mean_deg - sdk.mean_deg).abs())
            * 3600.0;
        worst = worst.max(error);
    }
    Check::measured(
        "override_obliquity",
        worst,
        bounds.obliquity_arcsec,
        "arcsec",
        "native against IAU 2006 and 2000B",
    )
}

/// The era in which the SDK's Delta T fit is built from measurements:
/// 1900-01-01 to 2005-01-01. After it the fit extrapolates, and by 2025 it
/// is 5 s above the measured value the engines carry; beyond 2050 two good
/// models differ by minutes within a century. The kit therefore holds a
/// native Delta T to the fit only here, and the SDK's Phase 1 Delta T is a
/// table plus a model rather than a fit.
pub const DELTA_T_MEASURED_ERA: (f64, f64) = (2_415_020.5, 2_453_371.5);

fn check_delta_t<P: EphemerisProvider + ?Sized>(
    provider: &P,
    capabilities: &Capabilities,
    bounds: &Bounds,
) -> Check {
    if !capabilities.overrides.contains(Overrides::DELTA_T) {
        return Check::skipped("override_delta_t", "not declared");
    }
    let (lo, hi) = DELTA_T_MEASURED_ERA;
    let (lo, hi) = (
        lo.max(capabilities.jd_range.0),
        hi.min(capabilities.jd_range.1),
    );
    if lo > hi {
        return Check::skipped("override_delta_t", "the coverage misses the measured era");
    }
    let mut worst = (0.0f64, lo);
    for i in 0..=10 {
        let jd = lo + (hi - lo) * f64::from(i) / 10.0;
        match provider.delta_t_seconds(jd) {
            Ok(native) => {
                let gap = (native - astro::delta_t_seconds_approx(jd)).abs();
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
            "native against the Espenak and Meeus fit in its measured era (1900 to 2005); worst at JD {}",
            worst.1
        ),
    )
}

fn check_ayanamsha<P: EphemerisProvider + ?Sized>(
    provider: &P,
    capabilities: &Capabilities,
    jds: &[f64],
) -> Check {
    if !capabilities.overrides.contains(Overrides::AYANAMSHA) {
        return Check::skipped("override_ayanamsha", "not declared");
    }
    // Published values at J2000.0 to a tenth of a degree, as capability honesty.
    let expected = [
        (AyanamshaId::LAHIRI, 23.85),
        (AyanamshaId::RAMAN, 22.40),
        (AyanamshaId::KRISHNAMURTI, 23.76),
    ];
    let mut details = Vec::new();
    for (id, value) in expected {
        if !capabilities.ayanamshas.contains(&id) {
            continue;
        }
        match provider.ayanamsha_deg(astro::DJ00, TimeScale::Tt, id) {
            Ok(native) if (native - value).abs() < 0.1 => {
                details.push(format!("{}: {native:.4}", id.0));
            }
            Ok(native) => {
                return Check::fail(
                    "override_ayanamsha",
                    format!("id {} gave {native} at J2000, expected about {value}", id.0),
                );
            }
            Err(error) => {
                return Check::fail(
                    "override_ayanamsha",
                    format!("declared, but failed: {error}"),
                );
            }
        }
    }
    let _ = jds;
    Check::pass("override_ayanamsha", details.join(", "))
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
        // The provider seen as equatorial-native, so completion has to rotate
        // back to the ecliptic frame the reference columns are in.
        let proxy = EquatorialProxy {
            inner: provider,
            frame: equatorial,
        };
        let completion = Completion::new(&proxy, policy);
        match completion.positions(&request(jds, &bodies, native, true)) {
            Ok(done) => {
                let mut worst = 0.0f64;
                for (index, cell) in done.columns.cells().enumerate() {
                    let Some(expected) = reference.cell(index) else {
                        continue;
                    };
                    let dlon = astro::angle_difference_deg(cell.lon, expected.lon).abs() * 3600.0;
                    let dlat = (cell.lat - expected.lat).abs() * 3600.0;
                    worst = worst.max(dlon).max(dlat);
                }
                let steps: Vec<String> = done
                    .steps
                    .iter()
                    .map(|s| format!("{}:{:?}", s.name, s.implementation))
                    .collect();
                out.push(Check::measured(
                    name,
                    worst,
                    bound,
                    "arcsec",
                    format!("steps {}", steps.join(" ")),
                ));
            }
            Err(error) => out.push(Check::fail(name, format!("{error}"))),
        }
    }
    out
}

/// A provider seen as equatorial-native: the same provider, its native
/// frame declared with equatorial coordinates, so completion has to
/// rotate. Only the kit uses it.
struct EquatorialProxy<'a, P: EphemerisProvider + ?Sized> {
    inner: &'a P,
    frame: Frame,
}

impl<P: EphemerisProvider + ?Sized> EphemerisProvider for EquatorialProxy<'_, P> {
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            native_frame: self.frame,
            ..self.inner.capabilities()
        }
    }

    fn positions(
        &self,
        request: &PositionRequest<'_>,
    ) -> Result<PositionColumns, crate::model::ProviderError> {
        self.inner.positions(request)
    }

    fn obliquity(
        &self,
        jd: f64,
        scale: TimeScale,
    ) -> Result<crate::model::Obliquity, crate::model::ProviderError> {
        self.inner.obliquity(jd, scale)
    }

    fn delta_t_seconds(&self, jd_ut1: f64) -> Result<f64, crate::model::ProviderError> {
        self.inner.delta_t_seconds(jd_ut1)
    }

    fn ayanamsha_deg(
        &self,
        jd: f64,
        scale: TimeScale,
        id: AyanamshaId,
    ) -> Result<f64, crate::model::ProviderError> {
        self.inner.ayanamsha_deg(jd, scale, id)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, reason = "a test fails by panicking")]

    use super::*;
    use crate::test_provider::SliceTestProvider;

    #[test]
    fn the_test_provider_passes_the_kit() {
        let report = run(&SliceTestProvider::new(), &Bounds::DEFAULT);
        let failed: Vec<&Check> = report.checks.iter().filter(|c| !c.passed).collect();
        assert!(failed.is_empty(), "{}", report.markdown());
        assert!(
            report
                .checks
                .iter()
                .any(|c| c.name == "completion_native" && c.detail.starts_with("skipped"))
        );
    }
}

/// What every runner writes: the kit report and the timing rows of one
/// provider, so the result page reads the same shape for each.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Results {
    /// The provider's short name.
    pub provider: String,
    /// The kit report.
    pub report: Report,
    /// The timing rows.
    pub bench: Vec<crate::bench::Row>,
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
