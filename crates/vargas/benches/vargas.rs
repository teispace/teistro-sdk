//! The vargas' budgets (`02-architecture/09-performance-architecture.md`):
//! one varga chart in 5 µs and every one in 100 µs, over one founded
//! moment. `cargo xtask measure-micro` runs both and holds them to it
//! (`03-design/micro-budgets.md`).

#![allow(
    missing_docs,
    clippy::panic,
    reason = "a benchmark binary stops on a fixture it cannot found"
)]

#[path = "../tests/common/mod.rs"]
mod common;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use teistro_core::catalogue::Varga;
use teistro_vargas::{Axis, chart, every_chart};

fn bench(c: &mut Criterion) {
    let foundation = common::founded();
    let navamsha = Axis::of(Varga::D9);
    c.bench_function("varga: D9 chart", |b| {
        b.iter(|| chart(black_box(&foundation), black_box(navamsha)));
    });
    c.bench_function("varga: every catalogued chart", |b| {
        b.iter(|| every_chart(black_box(&foundation)));
    });
}

criterion_group!(vargas, bench);
criterion_main!(vargas);
