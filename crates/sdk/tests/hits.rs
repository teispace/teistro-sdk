//! The transit hit list through the façade: every ingress read back through
//! a chart founded either side of it, Ketu as Rahu turned half a circle,
//! stations alternating, and one order however often it is asked
//! (`docs/03-design/transit-hit-list.md`).

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "tests fail by panicking and index what they found"
)]

use teistro::catalogue::{ChartKind, Graha, Rashi};
use teistro::gochar::hits::{AspectPhase, HitEvent, Motion};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{
    ChartRequest, Context, Document, Ephemeris, Hit, HitKind, HitRequest, NatalPoint, UtcOffset,
};

fn context(profile: Option<&str>) -> Context {
    let mut builder = Context::builder().ephemeris([Ephemeris::Builtin]);
    if let Some(profile) = profile {
        builder = builder.profile(profile);
    }
    builder.build().unwrap()
}

fn place() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    )
}

fn natal(sdk: &Context) -> Document {
    sdk.chart()
        .reading(
            JulianDay::<Utc>::literal(2_447_995.489_583_333_5),
            &ChartRequest::at(place(), UtcOffset::literal(5, 45, 0)),
        )
        .unwrap()
        .value
}

fn year() -> HitRequest {
    HitRequest::between(
        JulianDay::<Utc>::literal(2_460_676.5),
        JulianDay::<Utc>::literal(2_461_041.5),
    )
}

/// The acceptance is the consumer, not the search: a chart founded a
/// second before an ingress places the graha in the sign it left, and a
/// second after in the sign it entered — under the default profile and the
/// conformance one, whose nutated ayanamsha a frame's mean reading misses.
#[test]
fn every_sign_ingress_is_where_a_founded_chart_puts_it() {
    for profile in [None, Some("conformance-baseline")] {
        let sdk = context(profile);
        let natal = natal(&sdk);
        let hits = sdk
            .chart()
            .hits(&natal, &year().with_kinds([HitKind::SignIngress]))
            .unwrap()
            .value;
        assert!(hits.len() > 20, "{} ingresses in a year", hits.len());
        let second = 1.0 / 86_400.0;
        let either_side: Vec<JulianDay<Utc>> = hits
            .iter()
            .flat_map(|hit| {
                [
                    JulianDay::<Utc>::literal(hit.instant.get() - second),
                    JulianDay::<Utc>::literal(hit.instant.get() + second),
                ]
            })
            .collect();
        let founded = sdk
            .chart()
            .found_many(&either_side, &place(), UtcOffset::UTC, ChartKind::Natal)
            .unwrap()
            .value;
        for (hit, pair) in hits.iter().zip(founded.chunks(2)) {
            let HitEvent::SignIngress { into, motion } = hit.event else {
                panic!("only sign ingresses were asked for: {hit:?}");
            };
            let sign = |at: usize| {
                let deg = pair[at].graha(hit.graha).unwrap().longitude_deg;
                Rashi::ALL[(deg.rem_euclid(360.0) / 30.0) as usize % 12]
            };
            let step = |by: usize| Rashi::ALL[(into as usize + by) % 12];
            let left = match motion {
                Motion::Direct => step(11),
                Motion::Retrograde => step(1),
            };
            assert_eq!((sign(0), sign(1)), (left, into), "{profile:?} {hit:?}");
        }
    }
}

/// Ketu is Rahu's opposite point, searched as Rahu turned half a circle:
/// every one of its ingresses is Rahu's at the same instant, six signs on.
#[test]
fn ketu_enters_the_sign_opposite_rahu_at_the_same_instant() {
    let sdk = context(None);
    let natal = natal(&sdk);
    let decade = HitRequest::between(
        JulianDay::<Utc>::literal(2_460_676.5),
        JulianDay::<Utc>::literal(2_464_329.0),
    )
    .with_grahas([Graha::Rahu, Graha::Ketu])
    .with_kinds([HitKind::SignIngress]);
    let hits = sdk.chart().hits(&natal, &decade).unwrap().value;
    let (rahu, ketu): (Vec<&Hit>, Vec<_>) = hits.iter().partition(|hit| hit.graha == Graha::Rahu);
    assert!(!rahu.is_empty() && rahu.len() == ketu.len());
    for (r, k) in rahu.iter().zip(&ketu) {
        assert!((r.instant.get() - k.instant.get()).abs() < 1e-6);
        let (HitEvent::SignIngress { into: a, .. }, HitEvent::SignIngress { into: b, .. }) =
            (r.event, k.event)
        else {
            panic!("sign ingresses only");
        };
        assert_eq!(Rashi::ALL[(a as usize + 6) % 12], b);
    }
}

/// A graha's stations alternate, and every ingress runs the way the
/// stations either side of it say the graha was moving.
#[test]
fn stations_alternate_and_bound_the_backward_ingresses() {
    let sdk = context(None);
    let natal = natal(&sdk);
    let hits = sdk
        .chart()
        .hits(
            &natal,
            &year().with_grahas([Graha::Mercury, Graha::Mars, Graha::Saturn]),
        )
        .unwrap()
        .value;
    for graha in [Graha::Mercury, Graha::Mars, Graha::Saturn] {
        let mine: Vec<_> = hits.iter().filter(|hit| hit.graha == graha).collect();
        // The first station says which way it was moving before it: a
        // graha already retrograde when the window opens turns direct first.
        let first = mine.iter().find_map(|hit| match hit.event {
            HitEvent::Station { turns } => Some(turns),
            _ => None,
        });
        let Some(first) = first else {
            panic!("{graha:?} stood still at least once in a year");
        };
        let mut backward = first == Motion::Direct;
        for hit in mine {
            match hit.event {
                HitEvent::Station { turns } => {
                    assert_eq!(
                        turns == Motion::Retrograde,
                        !backward,
                        "{graha:?} at {}",
                        hit.instant.get()
                    );
                    backward = turns == Motion::Retrograde;
                }
                HitEvent::SignIngress { motion, .. }
                | HitEvent::NakshatraIngress { motion, .. }
                | HitEvent::Aspect { motion, .. } => {
                    assert_eq!(motion == Motion::Retrograde, backward, "{graha:?} {hit:?}");
                }
            }
        }
    }
}

#[test]
fn one_order_however_often_it_is_asked_and_a_bad_request_is_named() {
    let sdk = context(None);
    let natal = natal(&sdk);
    let first = sdk.chart().hits(&natal, &year()).unwrap().value;
    assert_eq!(first, sdk.chart().hits(&natal, &year()).unwrap().value);
    assert!(
        first
            .windows(2)
            .all(|pair| pair[0].instant.get() <= pair[1].instant.get())
    );
    let backwards = HitRequest::between(year().to(), year().from());
    assert_eq!(
        sdk.chart().hits(&natal, &backwards).unwrap_err().field(),
        Some("to")
    );
    let none = year().with_grahas([]);
    assert_eq!(
        sdk.chart().hits(&natal, &none).unwrap_err().field(),
        Some("grahas")
    );
}

/// The separation a founded chart gives at a hit's instant, between the
/// transiting graha and the natal point, degrees forward from the point.
fn separation(sdk: &Context, natal: &Document, hit: &Hit, to: NatalPoint) -> f64 {
    let chart = sdk
        .chart()
        .found_many(&[hit.instant], &place(), UtcOffset::UTC, ChartKind::Natal)
        .unwrap()
        .value
        .remove(0);
    let transit = chart.graha(hit.graha).unwrap().longitude_deg;
    let natal_deg = match to {
        NatalPoint::Graha { graha } => natal.foundation.graha(graha).unwrap().longitude_deg,
        NatalPoint::Lagna => natal.foundation.lagna_deg,
    };
    (transit - natal_deg).rem_euclid(360.0)
}

/// Whether a separation stands `off` degrees from an aspect's angle, from
/// either side of the natal point.
fn at_angle(separation: f64, angle: u16, off: f64) -> bool {
    let near = |target: f64| {
        let apart = (separation - target).rem_euclid(360.0);
        apart.min(360.0 - apart) < 1e-3
    };
    let angle = f64::from(angle);
    [
        angle + off,
        angle - off,
        360.0 - angle + off,
        360.0 - angle - off,
    ]
    .into_iter()
    .any(near)
}

/// Every exact aspect is exact in a chart founded at its instant, and every
/// orb's edge stands the orb from it; no window opens twice before it closes.
#[test]
fn every_aspect_is_at_its_angle_in_a_founded_chart() {
    let sdk = context(None);
    let natal = natal(&sdk);
    let asked = year()
        .with_kinds([HitKind::Aspect])
        .with_grahas([Graha::Sun, Graha::Mars, Graha::Jupiter, Graha::Saturn])
        .with_aspects([0, 90, 180])
        .with_orb(3.0);
    let hits = sdk.chart().hits(&natal, &asked).unwrap().value;
    let mut phases = [0_usize; 3];
    let mut open = std::collections::BTreeSet::new();
    for hit in &hits {
        let HitEvent::Aspect {
            to, angle, phase, ..
        } = hit.event
        else {
            panic!("only aspects were asked for: {hit:?}");
        };
        let off = if phase == AspectPhase::Exact {
            0.0
        } else {
            3.0
        };
        let apart = separation(&sdk, &natal, hit, to);
        assert!(at_angle(apart, angle, off), "{hit:?}: {apart}");
        phases[phase as usize] += 1;
        let point = match to {
            NatalPoint::Graha { graha } => graha as u8,
            NatalPoint::Lagna => u8::MAX,
        };
        let key = (hit.graha as u8, point, angle);
        match phase {
            AspectPhase::Entering => assert!(open.insert(key), "entered twice: {hit:?}"),
            // A window already open when the year began closes unentered.
            AspectPhase::Leaving => {
                open.remove(&key);
            }
            AspectPhase::Exact => {}
        }
    }
    assert!(phases.iter().all(|count| *count > 0), "{phases:?}");
}

#[test]
fn an_aspect_or_an_orb_that_cannot_be_is_named() {
    let sdk = context(None);
    let natal = natal(&sdk);
    let field = |asked: HitRequest| {
        sdk.chart()
            .hits(&natal, &asked)
            .unwrap_err()
            .field()
            .map(String::from)
    };
    // Any whole degree is an aspect, but the orb must keep the windows of
    // its lattice apart: lines 10° apart allow less than 5°.
    assert_eq!(
        field(year().with_aspects([0, 10]).with_orb(6.0)).as_deref(),
        Some("orb_deg")
    );
    assert_eq!(
        field(year().with_aspects([210])).as_deref(),
        Some("aspects")
    );
    assert_eq!(field(year().with_orb(20.0)).as_deref(), Some("orb_deg"));
    assert_eq!(field(year().with_points([])).as_deref(), Some("points"));
}

/// A batch searches the sky once and answers each chart what it would have
/// been answered alone, to the bit: its ingresses and stations are the
/// sky's, handed to every chart, and its aspects its own.
#[test]
fn many_charts_are_answered_what_each_is_answered_alone() {
    let sdk = context(None);
    let first = natal(&sdk);
    let second = sdk
        .chart()
        .reading(
            JulianDay::<Utc>::literal(2_451_545.0),
            &ChartRequest::at(place(), UtcOffset::UTC),
        )
        .unwrap()
        .value;
    let asked = year().with_aspects([0, 90, 180]).with_orb(2.0);
    let many = sdk
        .chart()
        .hits_many([&first, &second], &asked)
        .unwrap()
        .value;
    assert_eq!(many.len(), 2);
    for (natal, batch) in [&first, &second].into_iter().zip(&many) {
        let alone = sdk.chart().hits(natal, &asked).unwrap().value;
        assert!(!alone.is_empty(), "`alone` is empty");
        assert_eq!(batch, &alone);
    }
    assert_ne!(many[0], many[1], "two charts, two lists of aspects");
    let none: [&Document; 0] = [];
    assert_eq!(
        sdk.chart().hits_many(none, &asked).unwrap_err().field(),
        Some("natals")
    );
}
