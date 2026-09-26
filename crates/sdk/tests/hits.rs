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
use teistro::gochar::hits::{HitEvent, Motion};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{ChartRequest, Context, Document, Ephemeris, HitKind, HitRequest, UtcOffset};

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
    let (rahu, ketu): (Vec<&teistro::gochar::hits::Hit>, Vec<_>) =
        hits.iter().partition(|hit| hit.graha == Graha::Rahu);
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
                | HitEvent::NakshatraIngress { motion, .. } => {
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
