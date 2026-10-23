//! What founding a chart allocates, counted rather than guessed
//! (`05-testing/01-quality-bar.md`, "allocation counts").
//!
//! A foundation asks the sky for its zodiac, its day's sunrises, its two
//! house divisions, its lagnas and the grahas, and each completion
//! allocates its 13 (`crates/astro/tests/allocations.rs`); the rest is
//! the chart's own vectors and its stamp. So the count is the number of
//! times it asks, and what this test holds is its shape: a chart keeps
//! nothing once dropped, and a batch, stamped once, costs less than its
//! charts founded one by one.

#![allow(
    clippy::unwrap_used,
    clippy::print_stdout,
    reason = "tests fail by panicking, and report what they measured"
)]

use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::precession::PrecessionModel;
use teistro_calendar::Gregorian;
use teistro_calendar::solar::drik::DrikSun;
use teistro_chart::foundation::Founder;
use teistro_core::catalogue::{Ayanamsha, ChartKind};
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro_core::settings::{OverridePolicy, Profile, SettingsPatch, Sunrise};
use teistro_core::time::UtcOffset;
use teistro_port_ephemeris::test_provider::TestProvider;
use teistro_test_allocator::{Counting, Counts, measure};

#[global_allocator]
static ALLOCATOR: Counting = Counting::system();

/// A call's allocations, made once first so that what is counted is the
/// call's own, and dropped inside the measurement so that what it keeps
/// is what it leaves behind.
fn counted<T>(call: impl Fn() -> T) -> Counts {
    drop(call());
    measure(|| drop(call())).1
}

#[test]
fn a_chart_keeps_nothing_and_a_batch_is_stamped_once() {
    let provider = TestProvider;
    let resolved = Profile::shipped(teistro_core::settings::DEFAULT_PROFILE)
        .unwrap()
        .resolve(&SettingsPatch::default())
        .unwrap();
    let model = DrikSun::new(
        &provider,
        Ayanamsha::Lahiri,
        Sunrise::CentreNoRefraction.into(),
        OverridePolicy::PreferNative,
        DeltaTModel::TableThenModel,
    );
    let clock = UtcOffset::literal(5, 45, 0);
    let founder = Founder::new(
        &provider,
        &resolved,
        &model,
        &Gregorian,
        &clock,
        PrecessionModel::Vondrak2011,
        DeltaTModel::TableThenModel,
    );
    let place = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );
    // Ten instants, each on a day of its own, so no two share a sunrise.
    let instants: Vec<JulianDay<Utc>> = (0..10)
        .map(|i| JulianDay::<Utc>::literal(2_460_483.2 + 1.37 * f64::from(i)))
        .collect();

    let one = counted(|| {
        founder
            .found_one(
                JulianDay::<Utc>::literal(2_460_483.2),
                &place,
                ChartKind::Natal,
            )
            .unwrap()
    });
    let ten = counted(|| founder.found(&instants, &place, ChartKind::Natal).unwrap());
    println!(
        "a chart: {} allocations, {} bytes; ten in a batch: {}",
        one.allocations, one.bytes, ten.allocations
    );

    for (what, counts) in [("a chart", &one), ("a batch of ten", &ten)] {
        assert_eq!(
            counts.retained(),
            0,
            "{what} keeps {} bytes once dropped: a founder caches nothing between calls",
            counts.retained()
        );
    }
    assert!(
        one.allocations <= 1_100,
        "a chart allocates {} times, measured at 881: its completions of 13 each, \
         its vectors and its stamp; far above that is a search asking the sky more \
         often than it did",
        one.allocations
    );
    assert!(
        ten.allocations < 10 * one.allocations,
        "ten charts in a batch allocate {} times against {} for one: the batch is \
         stamped once, so it costs less than ten charts founded one by one",
        ten.allocations,
        one.allocations
    );
}
