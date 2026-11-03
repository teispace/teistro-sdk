//! The finders' cost (A2-1): an election over 26 days, an observance of
//! two traditions over a year, and a people digest of 1 and 10 births
//! (`03-design/election-search.md` §4, `observance-finder.md` §7,
//! `people-digest.md` §5).

#![allow(
    missing_docs,
    clippy::expect_used,
    reason = "a benchmark binary stops on a bad fixture"
)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use teistro::finders::{ElectRequest, ObservanceRequest, PeopleRequest};
use teistro::{Context, Ephemeris};

const KATHMANDU: &str = r#""latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1400, "utcOffsetSeconds": 20700"#;

fn finders(c: &mut Criterion) {
    let sdk = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("a context");
    let elect = ElectRequest::from_json(&format!(
        r#"{{"first": {{"year": 2026, "month": 11, "day": 20}}, "last": {{"year": 2026, "month": 12, "day": 15}},
            {KATHMANDU}, "muhurta": {{"rules": "RAMAN_MARRIAGE", "most": 3}}}}"#
    ))
    .expect("an election");
    c.bench_function("election search, 26 days", |b| {
        b.iter(|| sdk.almanac().elect(black_box(&elect)).expect("answered"));
    });
    let observance = ObservanceRequest::from_json(&format!(
        r#"{{"year": 2026, {KATHMANDU}, "traditions": [
            {{"name": "smarta", "festivals": {{"rules": "DHARMASINDHU"}}, "rule": "EKADASHI_SMARTA"}},
            {{"name": "vaishnava", "festivals": {{"rules": "DHARMASINDHU"}}, "rule": "EKADASHI_VAISHNAVA"}}]}}"#
    ))
    .expect("an observance");
    c.bench_function("observance finder, two traditions, a year", |b| {
        b.iter(|| {
            sdk.almanac()
                .observance(black_box(&observance), None)
                .expect("answered")
        });
    });
    for count in [1_usize, 10] {
        let people: Vec<String> = (0..count)
            .map(|at| {
                let instant = 2_447_000.25 + 400.0 * f64::from(u32::try_from(at).expect("few"));
                format!(r#"{{"name": "p{at}", "instant": {instant}, {KATHMANDU}}}"#)
            })
            .collect();
        let digest = PeopleRequest::from_json(&format!(
            r#"{{"day": {{"year": 2026, "month": 11, "day": 1}}, {KATHMANDU}, "people": [{}]}}"#,
            people.join(", ")
        ))
        .expect("a digest");
        c.bench_function(&format!("people digest, {count} people"), |b| {
            b.iter(|| sdk.almanac().people(black_box(&digest)).expect("answered"));
        });
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = finders
}
criterion_main!(benches);
