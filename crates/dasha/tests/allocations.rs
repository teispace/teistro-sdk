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
use teistro_core::catalogue::{Dignity, Rashi};
use teistro_core::quantity::{Degrees, Depth, JulianDay};
use teistro_core::settings::{AfterCycle, Balance, BirthPeriod, SeedOverflow, YearLength};
use teistro_dasha::{
    Birth, Dasha, RASHI_ROWS, ROWS, RashiChart, RashiDasha, Rules, Timeline, UduRow, VIMSHOTTARI,
};
use teistro_test_allocator::{Counting, measure};

#[global_allocator]
static ALLOCATOR: Counting = Counting::system();

fn dasha(after_cycle: AfterCycle) -> Dasha {
    dasha_of(&VIMSHOTTARI, after_cycle)
}

fn dasha_of(row: &'static UduRow, after_cycle: AfterCycle) -> Dasha {
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
    Dasha::new(row, &birth, rules).unwrap()
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

    // Every row reads the same way: a scaled one's second round and a
    // windowed one's chain allocate nothing either.
    for row in ROWS {
        let (made, counts) = measure(|| dasha_of(row, AfterCycle::End));
        assert_eq!(counts.allocations, 2, "{:?}: its two tables", row.system);
        let instant = JulianDay::literal(2_447_995.489_583_333_5 + 20.0 * 365.25);
        let (chain, counts) = measure(|| made.at(instant, deepest));
        assert!(!chain.is_empty(), "{:?}", row.system);
        assert_eq!(counts.allocations, 0, "{:?}: the chain", row.system);
    }
}

#[test]
fn a_sign_based_dasha_allocates_nothing_to_make_or_to_read() {
    let chart = RashiChart {
        lagna: Rashi::Pisces,
        arudha_lagna: Rashi::Gemini,
        navamsa_lagna: Rashi::Aquarius,
        signs: [
            Rashi::Aries,
            Rashi::Scorpio,
            Rashi::Aquarius,
            Rashi::Aries,
            Rashi::Gemini,
            Rashi::Aquarius,
            Rashi::Capricorn,
            Rashi::Capricorn,
            Rashi::Cancer,
        ],
        dignities: [Dignity::Neutral; 9],
    };
    let birth = JulianDay::literal(2_447_995.489_583_333_5);
    let deepest = Depth::try_new(6).unwrap();
    for row in RASHI_ROWS {
        let (made, counts) = measure(|| {
            RashiDasha::new(
                row,
                &chart,
                birth,
                YearLength::Julian36525,
                AfterCycle::Repeat,
            )
            .unwrap()
        });
        assert_eq!(
            counts.allocations, 0,
            "{:?}: its tables are arrays",
            row.system
        );
        for years in [0.5, 30.0, 250.0] {
            let instant = JulianDay::literal(birth.get() + years * 365.25);
            let (chain, counts) = measure(|| made.at(instant, deepest));
            assert_eq!(chain.len(), 6, "{:?} {years} years on", row.system);
            assert_eq!(
                counts.allocations, 0,
                "{:?}: the chain {years} years on",
                row.system
            );
        }
    }
}

#[test]
fn the_kalachakra_allocates_nothing_to_make_or_to_read() {
    use teistro_core::settings::{KalachakraAfterNinth, KalachakraBalance, KalachakraMembership};
    use teistro_dasha::{KalachakraDasha, KalachakraRules};
    let birth = Birth {
        instant: JulianDay::literal(2_447_995.489_583_333_5),
        moon: Nas::from_degrees(Degrees::try_new(221.786_980_828_370_36).unwrap()),
        moon_span: None,
    };
    let rules = KalachakraRules {
        balance: Balance::Spatial,
        year_length: YearLength::Julian36525,
        after_cycle: AfterCycle::Repeat,
        membership: KalachakraMembership::Listed,
        balance_of: KalachakraBalance::WholePada,
        after_ninth: KalachakraAfterNinth::Reverse,
    };
    let (made, counts) = measure(|| KalachakraDasha::new(&birth, rules).unwrap());
    assert_eq!(counts.allocations, 0, "its tables are arrays");
    for years in [0.5, 60.0, 600.0] {
        let instant = JulianDay::literal(birth.instant.get() + years * 365.25);
        let (chain, counts) = measure(|| made.at(instant, Depth::try_new(6).unwrap()));
        assert_eq!(chain.len(), 2, "{years} years on: to the antardashas");
        assert_eq!(counts.allocations, 0, "the chain {years} years on");
    }
}
