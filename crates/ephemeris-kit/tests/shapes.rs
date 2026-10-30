//! The kit over providers whose declarations take a check down a path the
//! test provider never does: a provider that refuses every request, one
//! that offers no body or every body, one whose native frame is
//! equatorial, one that declares rise and set or crossings without the
//! bodies they need, one whose coverage misses the Delta T table, one
//! whose Moon runs out and one that flags a cell asked for alone. Each
//! check says what it saw.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "a test fails by panicking"
)]

use teistro_ephemeris_kit::kit::{Bounds, Report, run};
use teistro_port_ephemeris::{
    Body, Capabilities, CellStatus, Coordinates, EphemerisProvider, Frame, Overrides,
    PositionColumns, PositionRequest, ProviderError, SpeedModel, TestProvider,
};

/// The kit's fourth instant over the test provider's coverage: its five
/// spread over 1900 to 2100.
const FOURTH_INSTANT: f64 = 2_469_807.25;

/// How a provider answers a position request.
#[derive(Clone, Copy, PartialEq)]
enum Answer {
    /// As the test provider does.
    Truly,
    /// Never.
    Refuses,
    /// With a longitude that is not a number in the first cell.
    Poisoned,
    /// A degree off whenever it is asked for one cell alone.
    Drifting,
    /// The test provider's numbers, but a cell asked for alone flagged
    /// with the provider's own code.
    Flagged,
    /// No Moon from half a continuity step before the kit's fourth
    /// instant: its data runs out there.
    MoonEnds,
}

/// The test provider under other declarations, answering as `answer` says.
struct Declared {
    capabilities: Capabilities,
    answer: Answer,
}

impl Declared {
    fn new(change: impl FnOnce(&mut Capabilities), refuses: bool) -> Declared {
        Declared::answering(
            change,
            if refuses {
                Answer::Refuses
            } else {
                Answer::Truly
            },
        )
    }

    fn answering(change: impl FnOnce(&mut Capabilities), answer: Answer) -> Declared {
        let mut capabilities = TestProvider::new().capabilities();
        change(&mut capabilities);
        Declared {
            capabilities,
            answer,
        }
    }
}

impl EphemerisProvider for Declared {
    fn capabilities(&self) -> Capabilities {
        self.capabilities.clone()
    }

    fn positions(&self, request: &PositionRequest<'_>) -> Result<PositionColumns, ProviderError> {
        let mut columns = match self.answer {
            Answer::Refuses => return Err(ProviderError::invalid("nothing is answered here")),
            _ => TestProvider::new().positions(request)?,
        };
        let alone = request.jds.len() == 1 && request.bodies.len() == 1;
        if let Some(lon) = columns.lon.first_mut() {
            match self.answer {
                Answer::Poisoned => *lon = f64::NAN,
                Answer::Drifting if alone => *lon = (*lon + 1.0) % 360.0,
                _ => {}
            }
        }
        match self.answer {
            Answer::Flagged if alone && columns.status.first() == Some(&CellStatus::Ok) => {
                columns.status[0] = CellStatus::Provider { code: 7 };
            }
            Answer::MoonEnds => {
                let ends = FOURTH_INSTANT - Bounds::DEFAULT.continuity_step_days / 2.0;
                for (row, jd) in request.jds.iter().enumerate() {
                    for (column, body) in request.bodies.iter().enumerate() {
                        if *body == Body::Moon && *jd >= ends {
                            let at = columns.index(row, column).unwrap();
                            columns.status[at] = CellStatus::DataMissing;
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(columns)
    }
}

fn detail<'r>(report: &'r Report, name: &str) -> (&'r str, bool) {
    let check = report.check(name).unwrap();
    (check.detail.as_str(), check.passed)
}

#[test]
fn a_provider_that_answers_nothing_fails_the_positions_check() {
    let report = run(&Declared::new(|_| {}, true), &Bounds::DEFAULT);
    let (said, passed) = detail(&report, "positions");
    assert!(!passed);
    assert!(
        said.starts_with("the native-frame request failed: ")
            && said.contains("nothing is answered here"),
        "{said}"
    );
    assert!(!report.passed);
}

#[test]
fn a_provider_offering_no_body_says_so_in_its_capabilities() {
    let report = run(
        &Declared::new(|c| c.bodies.clear(), false),
        &Bounds::DEFAULT,
    );
    let (said, passed) = detail(&report, "capabilities");
    assert!(!passed);
    assert!(said.contains("no bodies"), "{said}");
}

#[test]
fn an_equatorial_native_frame_skips_the_native_completion() {
    let report = run(
        &Declared::new(
            |c| c.native_frame = Frame::CANONICAL.with_coordinates(Coordinates::Equatorial),
            false,
        ),
        &Bounds::DEFAULT,
    );
    let (said, passed) = detail(&report, "completion_native");
    assert!(passed);
    assert_eq!(said, "skipped: the native frame is not ecliptic");
}

#[test]
fn rise_and_set_declared_without_the_sun_are_skipped() {
    let report = run(
        &Declared::new(
            |c| {
                c.overrides = Overrides::RISE_SET;
                c.bodies.retain(|body| *body != Body::Sun);
            },
            false,
        ),
        &Bounds::DEFAULT,
    );
    for name in ["override_rise_set_geometric", "override_rise_set_refracted"] {
        let (said, passed) = detail(&report, name);
        assert!(passed, "{name}");
        assert_eq!(said, "skipped: the Sun is not offered", "{name}");
    }
}

#[test]
fn a_longitude_that_is_not_a_number_fails_the_finite_check() {
    let report = run(
        &Declared::answering(|_| {}, Answer::Poisoned),
        &Bounds::DEFAULT,
    );
    let (said, passed) = detail(&report, "positions_finite_in_range");
    assert!(!passed);
    assert!(said.starts_with("cell 0 (Some("), "{said}");
    assert!(!report.passed);
}

#[test]
fn a_cell_asked_for_alone_must_be_the_cell_of_the_grid() {
    let report = run(
        &Declared::answering(|_| {}, Answer::Drifting),
        &Bounds::DEFAULT,
    );
    let (said, passed) = detail(&report, "batch_equals_single");
    assert!(!passed);
    assert!(
        said.contains(" at JD ") && said.contains(": grid "),
        "{said}"
    );
}

#[test]
fn speeds_a_rule_gives_are_measured_and_not_gated() {
    let report = run(
        &Declared::new(|c| c.speed_model = SpeedModel::Rule, false),
        &Bounds::DEFAULT,
    );
    let (said, passed) = detail(&report, "speed_consistency");
    assert!(passed);
    assert!(
        said.starts_with(
            "measured, not gated: the provider's speeds follow its rule, not the derivative; worst "
        ),
        "{said}"
    );
}

#[test]
fn a_cell_asked_for_alone_must_carry_the_grids_status() {
    let report = run(
        &Declared::answering(|_| {}, Answer::Flagged),
        &Bounds::DEFAULT,
    );
    let (said, passed) = detail(&report, "batch_equals_single");
    assert!(!passed);
    assert!(
        said.contains(" at JD ") && said.contains(": grid "),
        "{said}"
    );
}

#[test]
fn a_moon_that_runs_out_fails_the_grid_and_is_left_out_of_the_derivatives() {
    let report = run(
        &Declared::answering(|_| {}, Answer::MoonEnds),
        &Bounds::DEFAULT,
    );
    let (said, passed) = detail(&report, "positions_finite_in_range");
    assert!(!passed);
    assert!(
        said.contains("(Some(\"MOON\")): status DataMissing"),
        "{said}"
    );
    // The speed and the continuity are judged on the cells there are.
    for name in ["speed_consistency", "continuity"] {
        let (said, passed) = detail(&report, name);
        assert!(passed, "{name}: {said}");
    }
}

#[test]
fn a_coverage_past_the_delta_t_table_skips_its_check() {
    let report = run(
        &Declared::new(
            |c| {
                c.overrides = Overrides::DELTA_T;
                c.jd_range = (2_470_000.0, 2_480_000.0);
            },
            false,
        ),
        &Bounds::DEFAULT,
    );
    let (said, passed) = detail(&report, "override_delta_t");
    assert!(passed);
    assert_eq!(said, "skipped: the coverage misses the table's span");
}

#[test]
fn a_provider_offering_every_body_has_none_to_refuse() {
    let report = run(
        &Declared::new(|c| c.bodies = Body::ALL.to_vec(), false),
        &Bounds::DEFAULT,
    );
    let (said, passed) = detail(&report, "unsupported_body_reported");
    assert!(passed);
    assert_eq!(said, "skipped: every body is offered");
}

#[test]
fn crossings_declared_without_their_bodies_are_skipped() {
    let report = run(
        &Declared::new(
            |c| {
                c.overrides = Overrides::CROSSINGS;
                c.bodies
                    .retain(|body| !matches!(body, Body::Sun | Body::Mercury));
            },
            false,
        ),
        &Bounds::DEFAULT,
    );
    // The longitude's one body, and the elongation's second.
    for name in [
        "override_crossings_longitude",
        "override_crossings_composite",
    ] {
        let (said, passed) = detail(&report, name);
        assert!(passed, "{name}");
        assert!(
            said.starts_with("skipped: ") && said.ends_with(" is not offered"),
            "{name}: {said}"
        );
    }
}
