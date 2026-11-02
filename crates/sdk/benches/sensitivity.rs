//! The cost of a birth-time sensitivity (`docs/03-design/rectification.md`,
//! "Birth-time sensitivity", A2-5) through the record a consumer asks, over
//! the built-in ephemeris at Kathmandu: the default facts an hour either
//! side, then every varga the chart carries and the dasha at a reference
//! instant, and the default facts over a whole day, twelve hours either
//! side. The chart is founded once outside the loop, so each figure is the
//! search alone: every sample founds the sky at its instant, and every
//! change is bisected to a hundredth of a second.

#![allow(
    missing_docs,
    clippy::expect_used,
    reason = "a benchmark binary stops on a bad fixture"
)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use teistro::catalogue::DashaSystem;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{ChartRequest, Context, Ephemeris, RectificationRequest, UtcOffset};

/// 14 April 1990 at Kathmandu, the corpus's first moment.
const MOMENT: f64 = 2_447_995.489_583_333_5;

fn sensitivity(c: &mut Criterion) {
    let sdk = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("the built-in ephemeris");
    let offset = UtcOffset::literal(5, 45, 0);
    let place = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );
    let request = ChartRequest::at(place, offset)
        .with_every_varga()
        .with_dashas([DashaSystem::Vimshottari]);
    let birth = sdk
        .chart()
        .reading(JulianDay::<Utc>::literal(MOMENT), &request)
        .expect("a chart")
        .value;
    let mut group = c.benchmark_group("sensitivity");
    group.sample_size(10);
    for (id, json) in [
        (
            "hour_default",
            r#"{"sensitivity":{"minutes":60,"vargas":["D9","D10"]}}"#,
        ),
        (
            "hour_every_varga_and_dasha",
            r#"{"sensitivity":{"minutes":60,"dasha":{"at":2458953.5}}}"#,
        ),
        (
            "day_default",
            r#"{"sensitivity":{"minutes":720,"vargas":["D9","D10"]}}"#,
        ),
    ] {
        let asked = RectificationRequest::from_json(json).expect("a valid request");
        group.bench_function(id, |b| {
            b.iter(|| {
                sdk.chart()
                    .rectification(black_box(&birth), offset, &asked)
                    .expect("an answer")
            });
        });
    }
    group.finish();
}

criterion_group!(benches, sensitivity);
criterion_main!(benches);
