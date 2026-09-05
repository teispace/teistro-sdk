//! The astronomy layer's budgets (`docs/03-design/ephemeris-port-and-adapters.md`,
//! §8; `astro-events-and-crossings.md`, §7): the obliquity and nutation,
//! sidereal time, Delta T, a grid completed to the equatorial frame, a
//! sunrise.

#![allow(
    missing_docs,
    clippy::expect_used,
    reason = "a benchmark binary stops on a bad fixture"
)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use teistro_astro::ayanamsha;
use teistro_astro::houses::{self, Input};
use teistro_astro::precession::{self, PrecessionModel};
use teistro_astro::rise_set::Solver;
use teistro_astro::{Completion, DeltaTModel, delta_t, sky};
use teistro_core::catalogue::{Ayanamsha, HouseSystem};
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Tt, Ut1};
use teistro_core::settings::{OverridePolicy, PolarPolicy};
use teistro_port_ephemeris::{
    Body, Coordinates, Frame, Horizon, PositionRequest, TestProvider, TimeScale,
};

fn benches(c: &mut Criterion) {
    let tt = JulianDay::<Tt>::literal(2_460_000.5);
    let ut1 = JulianDay::<Ut1>::literal(2_460_000.5);
    c.bench_function("obliquity and nutation (IAU 2006, 2000B)", |b| {
        b.iter(|| sky::obliquity(black_box(tt)));
    });
    c.bench_function("sidereal time at a longitude", |b| {
        b.iter(|| sky::sidereal_time_deg(black_box(ut1), tt, Longitude::literal(85.324)));
    });
    c.bench_function("delta_t: table", |b| {
        b.iter(|| delta_t(black_box(ut1), DeltaTModel::TableThenModel));
    });
    c.bench_function("precession matrix (Vondrak 2011)", |b| {
        b.iter(|| precession::matrix(PrecessionModel::Vondrak2011, black_box(tt)));
    });
    c.bench_function("precession matrix (IAU 2006)", |b| {
        b.iter(|| precession::matrix(PrecessionModel::Iau2006, black_box(tt)));
    });
    c.bench_function("ayanamsha: Lahiri, mean (Vondrak 2011)", |b| {
        b.iter(|| {
            ayanamsha::mean_deg(
                &Ayanamsha::Lahiri.into(),
                black_box(tt),
                PrecessionModel::Vondrak2011,
                DeltaTModel::TableThenModel,
            )
        });
    });
    let input = Input {
        armc_deg: 258.67,
        latitude_deg: 27.7172,
        obliquity_deg: 23.44,
        sun_declination_deg: None,
        sidereal_offset_deg: 0.0,
    };
    for (name, system) in [
        ("houses: Placidus with the angles", HouseSystem::Placidus),
        (
            "houses: Regiomontanus with the angles",
            HouseSystem::Regiomontanus,
        ),
        ("houses: whole sign with the angles", HouseSystem::WholeSign),
    ] {
        c.bench_function(name, |b| {
            b.iter(|| houses::houses(system, black_box(&input), PolarPolicy::Error));
        });
    }
    c.bench_function(
        "ayanamsha: Fagan/Bradley, mean (fitted with Newcomb)",
        |b| {
            b.iter(|| {
                ayanamsha::mean_deg(
                    &Ayanamsha::FaganBradley.into(),
                    black_box(tt),
                    PrecessionModel::Vondrak2011,
                    DeltaTModel::TableThenModel,
                )
            });
        },
    );
    let provider = TestProvider::new();
    let completion = Completion::new(
        &provider,
        OverridePolicy::SdkOnly,
        DeltaTModel::TableThenModel,
    );
    let jds: Vec<f64> = (0..100)
        .map(|i| 2_451_545.0 + f64::from(i) * 36.525)
        .collect();
    let bodies = TestProvider::BODIES;
    let equatorial = PositionRequest::new(
        &jds,
        TimeScale::Ut1,
        &bodies,
        Frame::CANONICAL.with_coordinates(Coordinates::Equatorial),
    );
    c.bench_function(
        "completion: 100 x 8 grid to equatorial, SDK obliquity",
        |b| {
            b.iter(|| completion.positions(black_box(&equatorial)));
        },
    );
    let kathmandu = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );
    let solver = Solver::new(
        &completion,
        Body::Sun,
        kathmandu,
        Horizon::CENTRE_NO_REFRACTION,
        DeltaTModel::TableThenModel,
    );
    let midnight = JulianDay::<Ut1>::literal(2_460_482.5 - 85.324 / 360.0);
    c.bench_function(
        "rise and set: a day at Kathmandu over the test provider",
        |b| {
            b.iter(|| solver.day(black_box(midnight)));
        },
    );
}

criterion_group!(astro, benches);
criterion_main!(astro);
