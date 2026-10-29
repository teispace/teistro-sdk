//! The kit fails a provider for each fault it claims to catch, and names
//! the check that caught it.
//!
//! The kit's tests pass an honest provider and fail one whose obliquity is
//! a degree off; neither shows that a check fails on the fault it exists
//! for. Each case here breaks the test provider in one way and asserts
//! that the check named for that fault fails, so a check whose failure
//! branch had stopped being reachable would fail this suite.

#![allow(clippy::unwrap_used, clippy::panic, reason = "tests fail by panicking")]

use std::sync::atomic::{AtomicU32, Ordering};

use teistro_core::catalogue::Ayanamsha;
use teistro_ephemeris_kit::kit::{Bounds, run};
use teistro_port_ephemeris::{
    Body, Capabilities, CellStatus, EphemerisProvider, Overrides, PositionColumns, PositionRequest,
    ProviderError, TestProvider,
};

/// One way to break the test provider.
#[derive(Clone, Copy, Debug)]
enum Fault {
    /// One cell's longitude is not a number.
    NotANumber,
    /// Each call moves every longitude a little further.
    Drifting,
    /// A longitude depends on how many instants the request holds.
    BatchSized,
    /// Every speed is twice the derivative.
    DoubledSpeed,
    /// A longitude jumps by a hundredth of a degree every ten-thousandth
    /// of a day.
    Stepped,
    /// An instant outside the coverage is answered from its edge.
    AnswersOutsideCoverage,
    /// A body left out of the capabilities is still answered.
    AnswersAnUnofferedBody,
    /// DUT1 is declared, and every call fails.
    FailingDut1,
    /// Delta T is declared, and answers a day for every instant.
    WrongDeltaT,
    /// The identity has no name and the coverage is empty.
    BadCapabilities,
    /// The identity has no version, and ayanamshas are listed with no
    /// ayanamsha override to answer them.
    UnversionedAyanamshas,
    /// DUT1 is declared, and answers a number that is not one.
    NotANumberDut1,
    /// One cell's latitude is past the pole.
    PastThePole,
    /// Every distance is nought, for bodies that have one.
    NoDistance,
    /// Each call moves every longitude, and the provider says it may.
    DeclaredDrifting,
}

struct Sabotaged {
    fault: Fault,
    calls: AtomicU32,
}

impl Sabotaged {
    fn new(fault: Fault) -> Sabotaged {
        Sabotaged {
            fault,
            calls: AtomicU32::new(0),
        }
    }
}

impl EphemerisProvider for Sabotaged {
    fn capabilities(&self) -> Capabilities {
        let honest = TestProvider::new().capabilities();
        match self.fault {
            Fault::AnswersAnUnofferedBody => Capabilities {
                bodies: honest
                    .bodies
                    .iter()
                    .copied()
                    .filter(|body| *body != Body::Pluto)
                    .collect(),
                ..honest
            },
            Fault::FailingDut1 | Fault::NotANumberDut1 => Capabilities {
                overrides: Overrides::DUT1,
                ..honest
            },
            Fault::WrongDeltaT => Capabilities {
                overrides: Overrides::DELTA_T,
                ..honest
            },
            Fault::BadCapabilities => {
                let mut broken = honest;
                broken.identity.name.clear();
                broken.jd_range = (broken.jd_range.1, broken.jd_range.0);
                broken
            }
            Fault::UnversionedAyanamshas => {
                let mut broken = honest;
                broken.identity.version.clear();
                broken.ayanamshas = vec![Ayanamsha::Lahiri];
                broken
            }
            Fault::DeclaredDrifting => Capabilities {
                deterministic: false,
                ..honest
            },
            _ => honest,
        }
    }

    fn positions(&self, request: &PositionRequest<'_>) -> Result<PositionColumns, ProviderError> {
        let inner = TestProvider::new();
        let (low, high) = TestProvider::JD_RANGE;
        let clamped: Vec<f64> = request.jds.iter().map(|jd| jd.clamp(low, high)).collect();
        let mut columns = match self.fault {
            Fault::AnswersOutsideCoverage => inner.positions(&PositionRequest {
                jds: &clamped,
                ..*request
            })?,
            _ => inner.positions(request)?,
        };
        let call = self.calls.fetch_add(1, Ordering::Relaxed);
        let size = f64::from(u32::try_from(request.jds.len()).unwrap());
        for (index, lon) in columns.lon.iter_mut().enumerate() {
            let jd = request
                .jds
                .get(index / request.bodies.len().max(1))
                .copied()
                .unwrap_or(0.0);
            match self.fault {
                Fault::NotANumber if index == 0 => *lon = f64::NAN,
                Fault::Drifting | Fault::DeclaredDrifting => *lon += 1e-6 * f64::from(call),
                Fault::BatchSized => *lon += 1e-6 * size,
                Fault::Stepped => *lon += 0.01 * (jd * 1e4).floor().rem_euclid(2.0),
                _ => {}
            }
        }
        match self.fault {
            Fault::PastThePole => {
                if let Some(lat) = columns.lat.first_mut() {
                    *lat = 90.5;
                }
            }
            Fault::NoDistance => columns.dist.fill(0.0),
            _ => {}
        }
        if matches!(self.fault, Fault::DoubledSpeed) {
            for speed in &mut columns.lon_speed {
                *speed *= 2.0;
            }
        }
        Ok(columns)
    }

    fn dut1_seconds(&self, _jd_utc: f64) -> Result<f64, ProviderError> {
        match self.fault {
            Fault::NotANumberDut1 => Ok(f64::NAN),
            _ => Err(ProviderError::invalid("no DUT1 here")),
        }
    }

    fn delta_t_seconds(&self, _jd_ut1: f64) -> Result<f64, ProviderError> {
        Ok(86_400.0)
    }
}

/// Each fault fails its own check, and the report as a whole.
#[test]
fn every_fault_fails_the_check_named_for_it() {
    for (fault, check) in [
        (Fault::NotANumber, "positions_finite_in_range"),
        (Fault::Drifting, "determinism"),
        (Fault::BatchSized, "batch_equals_single"),
        (Fault::DoubledSpeed, "speed_consistency"),
        (Fault::Stepped, "continuity"),
        (Fault::AnswersOutsideCoverage, "out_of_range_reported"),
        (Fault::AnswersAnUnofferedBody, "unsupported_body_reported"),
        (Fault::FailingDut1, "override_dut1"),
        (Fault::WrongDeltaT, "override_delta_t"),
        (Fault::BadCapabilities, "capabilities"),
        (Fault::UnversionedAyanamshas, "capabilities"),
        (Fault::NotANumberDut1, "override_dut1"),
        (Fault::PastThePole, "positions_finite_in_range"),
        (Fault::NoDistance, "positions_finite_in_range"),
    ] {
        let report = run(&Sabotaged::new(fault), &Bounds::DEFAULT);
        let found = report
            .check(check)
            .unwrap_or_else(|| panic!("{fault:?}: no `{check}` check"));
        assert!(
            !found.passed,
            "{fault:?} passed `{check}`: {}\n{}",
            found.detail,
            report.markdown()
        );
        assert!(!report.passed, "{fault:?}: the report passed");
        assert!(report.markdown().contains("FAILED"), "{fault:?}");
    }
}

/// An unoffered body answered per cell as unsupported is what the kit
/// asks: the same capabilities with an honest answer pass.
#[test]
fn an_unoffered_body_reported_per_cell_passes() {
    struct Honest;
    impl EphemerisProvider for Honest {
        fn capabilities(&self) -> Capabilities {
            Sabotaged::new(Fault::AnswersAnUnofferedBody).capabilities()
        }
        fn positions(
            &self,
            request: &PositionRequest<'_>,
        ) -> Result<PositionColumns, ProviderError> {
            let mut columns = TestProvider::new().positions(request)?;
            for (index, status) in columns.status.iter_mut().enumerate() {
                if request.bodies.get(index % request.bodies.len().max(1)) == Some(&Body::Pluto) {
                    *status = CellStatus::UnsupportedBody;
                }
            }
            Ok(columns)
        }
    }
    let report = run(&Honest, &Bounds::DEFAULT);
    let check = report.check("unsupported_body_reported").unwrap();
    assert!(check.ran() && check.passed, "{}", check.detail);
}

/// A failure says what was wrong, each problem named: a check that
/// failed without saying why would leave the adapter's author guessing.
#[test]
fn a_failure_names_each_problem_it_found() {
    let report = run(
        &Sabotaged::new(Fault::UnversionedAyanamshas),
        &Bounds::DEFAULT,
    );
    let detail = &report.check("capabilities").unwrap().detail;
    assert!(detail.contains("identity incomplete"), "{detail}");
    assert!(
        detail.contains("an ayanamsha list without the ayanamsha override"),
        "{detail}"
    );
    let report = run(&Sabotaged::new(Fault::NotANumberDut1), &Bounds::DEFAULT);
    let detail = &report.check("override_dut1").unwrap().detail;
    assert!(detail.contains("answered NaN"), "{detail}");
}

/// A provider that declares itself non-deterministic is not failed for
/// differing on a repeated request: the check passes and says why.
#[test]
fn a_declared_drift_passes_determinism_and_says_so() {
    let report = run(&Sabotaged::new(Fault::DeclaredDrifting), &Bounds::DEFAULT);
    let check = report.check("determinism").unwrap();
    assert!(check.passed, "{}", check.detail);
    assert!(
        check.detail.contains("non-deterministic"),
        "{}",
        check.detail
    );
}
