//! Rectification through the façade: the kernel's sky read back through
//! the charts it stands for (`docs/03-design/rectification.md`).

#![allow(
    clippy::unwrap_used,
    reason = "tests fail by panicking on what they asked for"
)]

use teistro::catalogue::{Graha, Point, Rashi};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::rectification::{GulikaAt, PranapadaRule, Purifier, Reference, Rules, Window};
use teistro::{ChartRequest, Context, Document, Ephemeris, UtcOffset};

/// A moment at Kathmandu, 14 April 1990, the corpus's first.
const MOMENT: f64 = 2_447_995.489_583_333_5;

const OFFSET: UtcOffset = UtcOffset::literal(5, 45, 0);

fn sdk() -> Context {
    Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
}

fn kathmandu() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    )
}

fn window(hours: f64) -> Window {
    Window::between(
        JulianDay::literal(MOMENT),
        JulianDay::literal(MOMENT + hours / 24.0),
    )
    .unwrap()
}

fn chart(sdk: &Context, at: f64) -> Document {
    sdk.chart()
        .reading(
            JulianDay::<Utc>::literal(at),
            &ChartRequest::at(kathmandu(), OFFSET).with_points(),
        )
        .unwrap()
        .value
}

fn point(document: &Document, point: Point) -> Rashi {
    document
        .points
        .as_ref()
        .unwrap()
        .all()
        .iter()
        .find(|found| found.point == point)
        .unwrap()
        .sign
}

#[test]
fn every_clause_is_what_the_chart_of_its_instant_says() {
    // The SDK's own points, so each clause has a chart value to meet.
    let rules = Rules {
        gulika_at: GulikaAt::Start,
        pranapada_rule: PranapadaRule::SdkPoint,
        purify_as: teistro::rectification::PurifyAs::Weight,
        ..Rules::default()
    };
    let sdk = sdk();
    let answer = sdk
        .chart()
        .rectify(window(3.0), &kathmandu(), OFFSET, &rules)
        .unwrap()
        .value;
    assert!(
        answer.intervals.len() > 1,
        "a lagna changes sign in three hours"
    );
    for run in &answer.intervals {
        let middle = f64::midpoint(run.from.get(), run.to.get());
        let document = chart(&sdk, middle);
        let lagna = Rashi::of_longitude(document.foundation.lagna_deg);
        let moon = document
            .foundation
            .graha(Graha::Moon)
            .unwrap()
            .longitude_deg;
        for clause in &run.verdict.clauses {
            assert_eq!(clause.lagna, lagna, "at {middle}");
            let expected = match (clause.purifier, clause.reference) {
                (Purifier::Moon, _) => Rashi::of_longitude(moon),
                (Purifier::Gulika, Reference::Itself) => point(&document, Point::Gulika),
                (Purifier::Pranapada, _) => point(&document, Point::PranapadaLagna),
                _ => continue,
            };
            assert_eq!(clause.sign, expected, "{clause:?} at {middle}");
        }
    }
}

#[test]
fn the_runs_tile_the_window_and_a_bar_keeps_only_the_pure() {
    let window = window(2.0);
    let answer = sdk()
        .chart()
        .rectify(window, &kathmandu(), OFFSET, &Rules::default())
        .unwrap();
    let mut runs: Vec<_> = answer
        .value
        .intervals
        .iter()
        .chain(&answer.value.removed)
        .collect();
    runs.sort_by(|a, b| a.from.get().total_cmp(&b.from.get()));
    assert_eq!(runs.first().unwrap().from, window.from);
    assert_eq!(runs.last().unwrap().to, window.to);
    assert!(
        runs.iter()
            .zip(runs.iter().skip(1))
            .all(|(a, b)| a.to == b.from)
    );
    assert!(answer.value.intervals.iter().all(|run| run.verdict.pure));
    assert!(answer.value.removed.iter().all(|run| !run.verdict.pure));
    // The answer is sealed and says what it was asked.
    assert_ne!(answer.provenance.input_hash, answer.provenance.content_hash);
}

#[test]
fn a_refused_rule_names_its_field_before_any_sky_is_read() {
    let refused = Context::builder()
        .build()
        .unwrap()
        .chart()
        .rectify(
            window(1.0),
            &kathmandu(),
            OFFSET,
            &Rules {
                seed_minutes: 0.0,
                ..Rules::default()
            },
        )
        .unwrap_err();
    assert_eq!(refused.field(), Some("rectification.seedMinutes"));
}
