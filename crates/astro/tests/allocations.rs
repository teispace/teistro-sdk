//! What the hot paths allocate, counted rather than guessed
//! (`05-testing/01-quality-bar.md`, "allocation counts").
//!
//! A budget here is not a performance claim; it is the shape of the
//! code. A completion that allocates once per cell would be a different
//! program from one that allocates once per column, and only a count
//! tells them apart. Each assertion says what the number is made of, so
//! a change that moves it says which of those parts it moved.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::print_stdout,
    reason = "tests fail by panicking, and report what they measured"
)]

use teistro_astro::Completion;
use teistro_core::quantity::{JulianDay, Ut1};
use teistro_core::settings::OverridePolicy;
use teistro_port_ephemeris::{Body, Frame, PositionRequest, TestProvider, TimeScale};
use teistro_test_allocator::{Counting, measure};

#[global_allocator]
static ALLOCATOR: Counting = Counting::system();

/// The instants of a grid, allocated before the measurement so that the
/// count is the call's own.
fn grid(days: u32) -> Vec<f64> {
    (0..days).map(|i| 2_451_545.0 + f64::from(i)).collect()
}

#[test]
fn a_completion_allocates_per_column_and_not_per_cell() {
    let provider = TestProvider::new();
    let completion = Completion::new(
        &provider,
        OverridePolicy::SdkOnly,
        teistro_astro::DeltaTModel::TableThenModel,
    );
    let bodies = [
        Body::Sun,
        Body::Moon,
        Body::Mars,
        Body::Jupiter,
        Body::Saturn,
    ];
    let small = grid(2);
    let large = grid(200);

    let ask = |jds: &[f64]| {
        let request = PositionRequest::new(jds, TimeScale::Ut1, &bodies, Frame::CANONICAL);
        let (completed, counts) = measure(|| completion.positions(&request).unwrap());
        assert!(
            !completed.steps.is_empty(),
            "the completion says what it did"
        );
        counts
    };

    let ten = ask(&small);
    let thousand = ask(&large);
    println!(
        "10 cells: {} allocations, {} bytes; 1000 cells: {} allocations, {} bytes",
        ten.allocations, ten.bytes, thousand.allocations, thousand.bytes
    );
    assert_eq!(
        ten.allocations, thousand.allocations,
        "a grid a hundred times larger allocates the same number of times: \
         {} against {}; the columns are allocated once each, whatever the grid",
        ten.allocations, thousand.allocations
    );
    assert!(
        thousand.allocations <= 16,
        "a completion allocates {} times: the eight columns, the steps and the \
         provider's own vectors; a number far above that is an allocation per cell",
        thousand.allocations
    );
    assert!(
        thousand.bytes > ten.bytes * 50,
        "the bytes follow the grid even though the count does not: {} against {}",
        thousand.bytes,
        ten.bytes
    );
}

#[test]
fn delta_t_and_the_obliquity_allocate_nothing() {
    let at = JulianDay::<Ut1>::literal(2_451_545.0);
    let (_, counts) = measure(|| {
        teistro_astro::delta_t::delta_t(at, teistro_astro::DeltaTModel::TableThenModel).unwrap()
    });
    assert_eq!(counts.allocations, 0, "Delta T reads a table");
    let (_, counts) = measure(|| teistro_astro::iau::obl06(2_451_545.0, 0.0));
    assert_eq!(counts.allocations, 0, "the obliquity is a polynomial");
    let (_, counts) = measure(|| teistro_astro::iau::nut00b(2_451_545.0, 0.0));
    assert_eq!(counts.allocations, 0, "the nutation is a 77-term series");
}
