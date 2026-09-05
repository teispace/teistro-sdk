//! The core crate's budgets (`docs/03-design/core-types-and-catalogue.md`,
//! section 7): key resolution, attribute lookup, classification, settings
//! resolution and hashing.

#![allow(
    missing_docs,
    clippy::expect_used,
    reason = "a benchmark binary stops on a bad fixture"
)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use teistro_core::angle::Nas;
use teistro_core::catalogue::{Graha, Nakshatra, Rashi};
use teistro_core::key::resolve;
use teistro_core::quantity::Degrees;
use teistro_core::settings::{Profile, SettingsPatch};

fn keys(c: &mut Criterion) {
    c.bench_function("key: resolve graha.MARS", |b| {
        b.iter(|| resolve(black_box("graha.MARS")));
    });
    c.bench_function("key: Nakshatra::from_key PURVA_PHALGUNI", |b| {
        b.iter(|| Nakshatra::from_key(black_box("PURVA_PHALGUNI")));
    });
    c.bench_function("key: unknown key with suggestion", |b| {
        b.iter(|| resolve(black_box("graha.MARZ")));
    });
    c.bench_function("attributes: Graha::Mars", |b| {
        b.iter(|| black_box(Graha::Mars).attributes().exaltation);
    });
}

fn angles(c: &mut Criterion) {
    let degrees = Degrees::try_new(222.5763).expect("finite");
    c.bench_function("angle: Nas::from_degrees", |b| {
        b.iter(|| Nas::from_degrees(black_box(degrees)));
    });
    let nas = Nas::from_degrees(degrees);
    c.bench_function("angle: sign, nakshatra, pada", |b| {
        b.iter(|| {
            let n = black_box(nas);
            (n.sign(), n.nakshatra(), n.pada())
        });
    });
    c.bench_function("angle: part of nine", |b| b.iter(|| black_box(nas).part(9)));
    c.bench_function("angle: display", |b| b.iter(|| black_box(nas).to_string()));
    assert_eq!(nas.sign(), Rashi::Scorpio);
}

fn settings(c: &mut Criterion) {
    let profile = Profile::shipped("nepali-default").expect("shipped");
    let patch = SettingsPatch::default();
    c.bench_function("settings: resolve nepali-default", |b| {
        b.iter(|| profile.resolve(black_box(&patch)));
    });
    let resolved = profile.resolve(&patch).expect("coherent");
    c.bench_function("settings: canonical json and hash", |b| {
        b.iter(|| black_box(&resolved.settings).hash());
    });
}

criterion_group!(benches, keys, angles, settings);
criterion_main!(benches);
