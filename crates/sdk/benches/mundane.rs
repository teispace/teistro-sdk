//! What a mundane chart, a relocation and a map's lines cost on the
//! built-in ephemeris (`03-design/mundane-charts.md`,
//! `03-design/astrocartography.md`): each search a kind runs, the chart
//! cast after it, and the lines' founding plus their geometry.

#![allow(
    missing_docs,
    clippy::expect_used,
    reason = "a benchmark binary stops on a bad fixture"
)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use teistro::{AstrocartographyRequest, Context, Ephemeris, MundaneRequest, RelocationRequest};

fn benches(c: &mut Criterion) {
    let sdk = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("a built-in context");
    for (name, body) in [
        (
            "mundane: Mesha sankranti",
            r#""kind": "MESHA_SANKRANTI", "year": 2026"#,
        ),
        (
            "mundane: Aries ingress",
            r#""kind": "ARIES_INGRESS", "year": 2026"#,
        ),
        (
            "mundane: new moon before Mesha",
            r#""kind": "NEW_MOON_BEFORE_INGRESS", "year": 2026"#,
        ),
        (
            "mundane: solar eclipse",
            r#""kind": "SOLAR_ECLIPSE", "year": 2026"#,
        ),
        (
            "mundane: great conjunction",
            r#""kind": "GREAT_CONJUNCTION", "year": 2020"#,
        ),
        (
            "mundane: event",
            r#""kind": "EVENT", "event": {"name": "x", "instant": 2461000.5, "source": "y"}"#,
        ),
    ] {
        let asked = MundaneRequest::from_json(&format!(
            r#"{{{body}, "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700}}"#
        ))
        .expect("a mundane request");
        c.bench_function(name, |b| {
            b.iter(|| sdk.chart().mundane(black_box(&asked)).expect("cast"));
        });
    }
    let relocated = RelocationRequest::from_json(
        r#"{"instant": 2461000.5, "places": [{"latitudeDeg": 51.5, "longitudeDeg": -0.12}]}"#,
    )
    .expect("a relocation");
    c.bench_function("relocation: one place", |b| {
        b.iter(|| {
            sdk.chart()
                .relocated(black_box(&relocated))
                .expect("recast")
        });
    });
    let map = AstrocartographyRequest::new(2_461_000.5);
    c.bench_function("astrocartography: the seven at 1°", |b| {
        b.iter(|| {
            sdk.chart()
                .astrocartography(black_box(&map))
                .expect("drawn")
        });
    });
}

criterion_group! {
    name = mundane;
    config = Criterion::default().sample_size(10);
    targets = benches
}
criterion_main!(mundane);
