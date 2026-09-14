//! What reading a dasha allocates, counted rather than guessed
//! (`05-testing/01-quality-bar.md`, "allocation counts";
//! `03-design/dasha-kernels.md`, "The cursor": `at(t, 5)` with zero
//! allocations).
//!
//! A dasha is read far more often than it is made: a timeline asks for
//! the chain at every instant it draws. So making one may allocate its two
//! small tables, and reading one allocates nothing at any depth.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests fail by panicking"
)]

use teistro_core::angle::Nas;
use teistro_core::quantity::{Degrees, Depth, JulianDay};
use teistro_core::settings::{AfterCycle, Balance, BirthPeriod, SeedOverflow, YearLength};
use teistro_dasha::{Birth, Dasha, Rules, VIMSHOTTARI};
use teistro_test_allocator::{Counting, measure};

#[global_allocator]
static ALLOCATOR: Counting = Counting::system();

fn dasha(after_cycle: AfterCycle) -> Dasha {
    let birth = Birth {
        instant: JulianDay::literal(2_447_995.489_583_333_5),
        moon: Nas::from_degrees(Degrees::try_new(221.786_980_828_370_36).unwrap()),
        moon_span: None,
    };
    let rules = Rules {
        balance: Balance::Spatial,
        year_length: YearLength::Julian36525,
        birth_period: BirthPeriod::Compressed,
        after_cycle,
        seed_overflow: SeedOverflow::WrapToStart,
    };
    Dasha::new(&VIMSHOTTARI, &birth, rules).unwrap()
}

#[test]
fn making_a_dasha_allocates_its_two_tables_and_reading_one_allocates_nothing() {
    let (made, counts) = measure(|| dasha(AfterCycle::End));
    assert_eq!(
        counts.allocations, 2,
        "where the birth cycle's periods begin, and a whole cycle's"
    );

    let deepest = Depth::try_new(6).unwrap();
    for years in [0.5, 7.0, 40.0, 100.0] {
        let instant = JulianDay::literal(2_447_995.489_583_333_5 + years * 365.25);
        let (chain, counts) = measure(|| made.at(instant, deepest));
        assert_eq!(chain.len(), 6, "{years} years on");
        assert_eq!(counts.allocations, 0, "the chain {years} years on");
    }

    let repeating = dasha(AfterCycle::Repeat);
    let far = JulianDay::literal(2_447_995.5 + 300.0 * 365.25);
    let (chain, counts) = measure(|| repeating.at(far, deepest));
    assert_eq!(chain.deepest().unwrap().path.cycle(), 2);
    assert_eq!(counts.allocations, 0, "a later cycle's chain");

    let maha = made.mahadasha(0, 4).unwrap();
    let (count, counts) = measure(|| made.children(&maha).count());
    assert_eq!((count, counts.allocations), (9, 0), "a period's children");
}
