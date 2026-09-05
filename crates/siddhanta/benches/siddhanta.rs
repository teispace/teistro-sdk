//! The siddhanta crate's budgets (`docs/03-design/siddhanta.md`, section
//! 7): the Sun's longitude, all nine grahas, the day's arc.

#![allow(
    missing_docs,
    clippy::expect_used,
    reason = "a benchmark binary stops on a bad fixture"
)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use teistro_core::catalogue::Graha;
use teistro_core::quantity::{JulianDay, Latitude, Longitude, Ut1};
use teistro_siddhanta::{Parameters, SuryaSiddhanta, Trig};

fn model(c: &mut Criterion) {
    let text = SuryaSiddhanta::text();
    let exact = SuryaSiddhanta::new(Parameters::TEXT, Trig::Exact);
    let at = JulianDay::<Ut1>::try_new(2_460_413.5).expect("finite");
    c.bench_function("sun longitude, table", |b| {
        b.iter(|| text.sun_longitude_deg(black_box(2_460_413.5)));
    });
    c.bench_function("sun longitude, exact", |b| {
        b.iter(|| exact.sun_longitude_deg(black_box(2_460_413.5)));
    });
    c.bench_function("moon, table", |b| {
        b.iter(|| text.moon(black_box(at)));
    });
    c.bench_function("saturn (four steps, the motion by rule), table", |b| {
        b.iter(|| text.graha(Graha::Saturn, black_box(at)));
    });
    c.bench_function("all nine grahas, table", |b| {
        b.iter(|| text.all(black_box(at)));
    });
    let kathmandu = Latitude::try_new(27.7172).expect("in range");
    let midnight = JulianDay::<Ut1>::try_new(2_460_482.5 - 85.324 / 360.0).expect("finite");
    c.bench_function("day arc at Kathmandu, table", |b| {
        b.iter(|| text.day_arc(black_box(midnight), kathmandu));
    });
    c.bench_function("lagna at Kathmandu, table", |b| {
        b.iter(|| {
            text.lagna(
                black_box(at),
                Latitude::literal(27.7172),
                Longitude::literal(85.324),
            )
        });
    });
}

criterion_group!(benches, model);
criterion_main!(benches);
