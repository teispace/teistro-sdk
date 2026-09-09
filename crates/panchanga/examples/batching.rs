//! What a batch actually asks the ephemeris for, written as JSON for the
//! pass that measures it (`cargo xtask batching`).
//!
//! The port's shape says a caller asks for a **grid** — instants times
//! bodies in one call — because an ephemeris call is the expensive thing
//! in this library, and the architecture's own performance note says a
//! batch of a thousand charts should be "linear, parallelisable across
//! contexts" (`01-research/platform/07-performance.md`). Whether the
//! SDK's own layers keep to that is not an opinion: wrap the provider in
//! [`CountingProvider`], found the batch, read the tally.
//!
//! Three things are measured, each at several sizes, because what matters
//! is not the count but how it **scales**: a batch that asks for one call
//! whatever its size is a batch, and one whose calls grow with it is a
//! loop wearing a batch's signature.
//!
//! The analytic test provider answers instantly, so nothing here is a
//! timing. The counts are the same for any provider, and it is the
//! counts that decide the design: an ephemeris costing a millisecond a
//! call is fine at one call a chart and unusable at eight hundred.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::print_stdout,
    reason = "an example fails by panicking and reports what it measured"
)]

use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::precession::PrecessionModel;
use teistro_calendar::solar::drik::DrikSun;
use teistro_calendar::{CalendarDate, CalendarSystem, Gregorian};
use teistro_chart::foundation::Founder;
use teistro_core::catalogue::{Ayanamsha, Calendar, ChartKind};
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro_core::settings::{
    DEFAULT_PROFILE, OverridePolicy, Profile, Resolved, SettingsPatch, Sunrise,
};
use teistro_core::time::UtcOffset;
use teistro_panchanga::Almanac;
use teistro_port_ephemeris::caching::DEFAULT_CAPACITY;
use teistro_port_ephemeris::test_provider::TestProvider;
use teistro_port_ephemeris::{
    Body, CacheStats, CachingProvider, CountingProvider, EphemerisProvider, Frame, PositionRequest,
    ProviderCalls, TimeScale,
};

/// Kathmandu, where the corpus's own charts are.
fn place() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.3240),
        Altitude::literal(1400.0),
    )
}

/// The operations the port names: `positions` and the seven overrides.
/// Everything an engine does beyond them is unreachable through the SDK
/// today, which is the other half of what this measures.
const PORT_OPERATIONS: usize = 8;

/// The batch sizes measured. One, so the per-item cost is visible; then
/// sizes far enough apart that a constant, a linear and a quadratic term
/// are told apart by the ratios rather than by a fit.
const SIZES: [usize; 4] = [1, 2, 10, 50];

fn resolved() -> Resolved {
    Profile::shipped(DEFAULT_PROFILE)
        .expect("the default profile")
        .resolve(&SettingsPatch::default())
        .expect("it resolves")
}

/// The provider every measurement is made through: a counter, so what
/// reached the ephemeris is visible, inside a cache, so the memo can be
/// switched on and off without changing a type. A capacity of nothing is
/// a cache that does nothing, which is how the uncached arm is measured
/// — the same code path, one number apart.
type Measured = CachingProvider<CountingProvider<TestProvider>>;

fn counted() -> Measured {
    cached(false)
}

fn cached(memo: bool) -> Measured {
    let counter = CountingProvider::new(TestProvider).watching_repeats();
    let capacity = if memo { DEFAULT_CAPACITY } else { 0 };
    CachingProvider::with_capacity(counter, capacity)
}

/// What reached the ephemeris under a measured provider.
fn reached(provider: &Measured) -> ProviderCalls {
    provider.inner().calls()
}

fn tally(calls: ProviderCalls) -> serde_json::Value {
    serde_json::json!({
        "positions": calls.positions,
        "cells": calls.cells,
        "widest": calls.widest,
        "distinct_cells": calls.distinct_cells,
        "repeat_share": calls.repeat_share(),
        "mean_width": calls.mean_width(),
        "obliquity": calls.obliquity,
        "delta_t": calls.delta_t,
        "ayanamsha": calls.ayanamsha,
        "horizon": calls.horizon,
        "total": calls.total(),
    })
}

/// The solar model the chart and almanac layers are founded on. The same
/// three settings every time: it is the provider underneath that is
/// being measured, not the model.
fn sun_model(provider: &Measured) -> DrikSun<'_, Measured> {
    DrikSun::new(
        provider,
        Ayanamsha::Lahiri,
        Sunrise::CentreNoRefraction.into(),
        OverridePolicy::PreferNative,
        DeltaTModel::TableThenModel,
    )
}

/// The clock the corpus's own charts are cast against.
fn clock() -> UtcOffset {
    UtcOffset::literal(5, 45, 0)
}

/// A grid of positions: the port's own shape, and the control. Whatever
/// the batch, one call; everything below is compared with this.
fn positions_grid(measurements: &mut Vec<serde_json::Value>) {
    for size in SIZES {
        let provider = counted();
        let jds: Vec<f64> = (0..size)
            .map(|day| 2_451_545.0 + f64::from(u16::try_from(day).unwrap_or(0)))
            .collect();
        let bodies = [Body::Sun, Body::Moon, Body::Mars];
        let request = PositionRequest::new(&jds, TimeScale::Ut1, &bodies, Frame::CANONICAL);
        provider.positions(&request).expect("the canonical frame");
        measurements.push(serde_json::json!({
            "operation": "positions",
            "size": size,
            "unit": "instant",
            "calls": tally(reached(&provider)),
        }));
    }
}

/// A batch of charts founded at once.
fn chart_batches(measurements: &mut Vec<serde_json::Value>) {
    for size in SIZES {
        let provider = counted();
        let resolved = resolved();
        let model = sun_model(&provider);
        let clock = clock();
        let founder = Founder::new(
            &provider,
            &resolved,
            &model,
            &Gregorian,
            &clock,
            PrecessionModel::default(),
            DeltaTModel::TableThenModel,
        );
        let instants: Vec<JulianDay<Utc>> = (0..size)
            .map(|day| {
                JulianDay::<Utc>::literal(2_460_000.5 + f64::from(u16::try_from(day).unwrap_or(0)))
            })
            .collect();
        founder
            .found(&instants, &place(), ChartKind::Natal)
            .expect("a chart at every instant");
        measurements.push(serde_json::json!({
            "operation": "charts",
            "size": size,
            "unit": "chart",
            "calls": tally(reached(&provider)),
        }));
    }
}

/// A range of almanac days asked for as one range.
fn almanac_ranges(measurements: &mut Vec<serde_json::Value>) {
    for size in SIZES {
        let provider = counted();
        let resolved = resolved();
        let model = sun_model(&provider);
        let clock = clock();
        let almanac = Almanac::new(
            &provider,
            &resolved,
            &model,
            &Gregorian,
            &clock,
            PrecessionModel::default(),
            DeltaTModel::TableThenModel,
        );
        let from = CalendarDate::defined(Calendar::Gregorian, 2024, 4, 1);
        let last = Gregorian
            .fixed_of(&from)
            .expect("a date the calendar has")
            .plus_days(i64::try_from(size).unwrap_or(1) - 1);
        let to = Gregorian.date_of(last).expect("a date the calendar has");
        almanac
            .between(&from, &to, &place())
            .expect("an almanac for every day");
        measurements.push(serde_json::json!({
            "operation": "almanac",
            "size": size,
            "unit": "day",
            "calls": tally(reached(&provider)),
        }));
    }
}

/// One sunrise, on its own. Named separately because the batches above
/// are made of these, and a number that dominates a chart should be
/// attributed rather than inferred from a subtraction.
fn one_sunrise(measurements: &mut Vec<serde_json::Value>) {
    let provider = counted();
    let model = sun_model(&provider);
    let (day, _) =
        teistro_calendar::fixed::FixedDay::from_jd(JulianDay::<Utc>::literal(2_460_000.5));
    let light = teistro_calendar::solar::SolarModel::day_light(&model, day, &place());
    let found = light.is_ok();
    measurements.push(serde_json::json!({
        "operation": "sunrise",
        "size": 1,
        "unit": "day",
        "found": found,
        "calls": tally(reached(&provider)),
    }));
}

/// What a memo saves: the same batches, the cache off and on.
///
/// The cache is sound only over a provider that answers identical
/// requests with identical bits, which the port has always asked every
/// provider to declare and nothing read until now. Both arms run the
/// same code through the same types; only the capacity differs, so what
/// separates the numbers is the memo and nothing else.
fn memo(measurements: &mut Vec<serde_json::Value>) {
    for size in SIZES {
        for memo in [false, true] {
            let provider = cached(memo);
            let stats = run_almanac(&provider, size);
            measurements.push(serde_json::json!({
                "operation": "almanac",
                "size": size,
                "unit": "day",
                "memo": memo,
                "caching": provider.caching(),
                "hit_share": stats.hit_share(),
                "calls": tally(reached(&provider)),
            }));
        }
    }
}

/// One range of almanac days through a provider, and what its cache did.
fn run_almanac(provider: &Measured, size: usize) -> CacheStats {
    let resolved = resolved();
    let model = sun_model(provider);
    let clock = clock();
    let almanac = Almanac::new(
        provider,
        &resolved,
        &model,
        &Gregorian,
        &clock,
        PrecessionModel::default(),
        DeltaTModel::TableThenModel,
    );
    let from = CalendarDate::defined(Calendar::Gregorian, 2024, 4, 1);
    let last = Gregorian
        .fixed_of(&from)
        .expect("a date the calendar has")
        .plus_days(i64::try_from(size).unwrap_or(1) - 1);
    let to = Gregorian.date_of(last).expect("a date the calendar has");
    almanac
        .between(&from, &to, &place())
        .expect("an almanac for every day");
    provider.stats()
}

/// What the port reaches of a provider: the other half of the same
/// question, because a batch that asks well is still limited to what it
/// may ask for. The port names eight operations; an engine names many
/// more, and what it names beyond them is unreachable through the SDK.
fn reach() -> serde_json::Value {
    let provider = TestProvider;
    // What the engine names beyond the port, reached through the port
    // rather than around it: the manifest the provider ships, read
    // through `Native`. The SDK holds no list of these — it counts what
    // the engine declares, so this number moves when the engine does and
    // not when the SDK does.
    let native = provider
        .native()
        .and_then(|native| native.manifest().ok())
        .map_or(0, |manifest| manifest.len());
    serde_json::json!({
        "port_operations": PORT_OPERATIONS,
        "declared_overrides": provider
            .capabilities()
            .overrides
            .names()
            .len(),
        "native_operations": native,
    })
}

fn main() {
    let mut measurements = Vec::new();
    positions_grid(&mut measurements);
    chart_batches(&mut measurements);
    almanac_ranges(&mut measurements);
    one_sunrise(&mut measurements);
    let mut memos = Vec::new();
    memo(&mut memos);

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "measurements": measurements,
            "memo": memos,
            "reach": reach(),
        }))
        .expect("the measurements serialise")
    );
}
