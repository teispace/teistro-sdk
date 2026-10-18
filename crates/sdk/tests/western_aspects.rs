//! The Western aspects a chart holds (`03-design/western-aspects.md`),
//! held to Leo's reading of King Edward VII's nativity in *How to Judge a
//! Nativity* (pp. 295–296).

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking"
)]

use teistro::catalogue::Graha;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{
    AspectRequest, ChartRequest, Context, Document, Ephemeris, UtcOffset, WesternAspect,
};

/// 9 November 1841, 10.48 a.m. GMT (p. 295).
const BIRTH: f64 = 2_393_783.95;

fn context() -> Context {
    Context::builder()
        .profile("western-tropical-default")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
}

/// Buckingham Palace.
fn request() -> ChartRequest {
    ChartRequest::at(
        Place::new(
            Latitude::literal(51.501),
            Longitude::literal(-0.142),
            Altitude::literal(0.0),
        ),
        UtcOffset::UTC,
    )
    .with_outer_planets()
}

fn birth(sdk: &Context) -> Document {
    sdk.chart()
        .reading(JulianDay::<Utc>::try_new(BIRTH).unwrap(), &request())
        .unwrap()
        .value
}

/// Whether the table holds this aspect between the two, either way round.
fn holds(rows: &[teistro::WesternAspectRow], a: Graha, aspect: WesternAspect, b: Graha) -> bool {
    rows.iter().any(|row| {
        row.aspect == aspect
            && ((row.first, row.second) == (a, b) || (row.first, row.second) == (b, a))
    })
}

#[test]
fn leos_four_aspects_of_king_edward_hold_under_his_orbs() {
    let sdk = context();
    let rows = sdk
        .chart()
        .western_aspects(&birth(&sdk), &AspectRequest::default())
        .unwrap();
    // "The Sun ... in trine to Uranus", "the sextile of Mars", "the square
    // of Neptune", and the Moon and Saturn "in square to one another".
    for (a, aspect, b) in [
        (Graha::Sun, WesternAspect::Trine, Graha::Uranus),
        (Graha::Sun, WesternAspect::Sextile, Graha::Mars),
        (Graha::Sun, WesternAspect::Square, Graha::Neptune),
        (Graha::Moon, WesternAspect::Square, Graha::Saturn),
    ] {
        assert!(
            holds(&rows, a, aspect, b),
            "{a:?} {aspect:?} {b:?} in {rows:#?}"
        );
    }
    // Closest first, every row inside its orb, no node read.
    assert!(
        rows.windows(2)
            .all(|pair| pair[0].from_exact_deg <= pair[1].from_exact_deg)
    );
    assert!(rows.iter().all(|row| row.from_exact_deg <= row.orb_deg));
    assert!(rows.iter().all(|row| {
        ![row.first, row.second]
            .iter()
            .any(|graha| matches!(graha, Graha::Rahu | Graha::Ketu))
    }));
}

#[test]
fn lillys_moieties_read_the_seven_and_name_a_planet_they_lack() {
    let sdk = context();
    let refused = sdk
        .chart()
        .western_aspects(&birth(&sdk), &AspectRequest::lilly())
        .unwrap_err();
    assert!(refused.to_string().contains("URANUS"), "{refused}");

    // Without the outer three, the Ptolemaic five under his moieties: the
    // Moon (12½) and Saturn (10) square within 11¼.
    let seven = sdk
        .chart()
        .reading(
            JulianDay::<Utc>::try_new(BIRTH).unwrap(),
            &ChartRequest::at(
                Place::new(
                    Latitude::literal(51.501),
                    Longitude::literal(-0.142),
                    Altitude::literal(0.0),
                ),
                UtcOffset::UTC,
            ),
        )
        .unwrap()
        .value;
    let rows = sdk
        .chart()
        .western_aspects(&seven, &AspectRequest::lilly())
        .unwrap();
    let square = rows
        .iter()
        .find(|row| {
            row.aspect == WesternAspect::Square
                && row.first == Graha::Moon
                && row.second == Graha::Saturn
        })
        .unwrap();
    assert!((square.orb_deg - 11.25).abs() < 1e-12);
    assert!(
        rows.iter()
            .all(|row| WesternAspect::PTOLEMAIC.contains(&row.aspect))
    );
}

#[test]
fn king_edwards_shapes_hold_on_his_own_aspects() {
    let sdk = context();
    let chart = birth(&sdk);
    let rows = sdk
        .chart()
        .western_aspects(&chart, &AspectRequest::default())
        .unwrap();
    let shapes = sdk
        .chart()
        .western_configurations(
            &chart,
            &AspectRequest::default(),
            teistro::ConfigurationRules::default(),
        )
        .unwrap();
    // Two T-squares, and each read back from the aspect table itself:
    // the apex square to both ends of an opposition.
    let found: Vec<(teistro::ConfigurationKind, Vec<Graha>, Option<Graha>)> = shapes
        .iter()
        .map(|shape| (shape.kind, shape.bodies.clone(), shape.apex))
        .collect();
    assert_eq!(
        found,
        [
            (
                teistro::ConfigurationKind::TSquare,
                vec![Graha::Moon, Graha::Jupiter, Graha::Uranus],
                Some(Graha::Jupiter),
            ),
            (
                teistro::ConfigurationKind::TSquare,
                vec![Graha::Mars, Graha::Venus, Graha::Pluto],
                Some(Graha::Mars),
            ),
        ]
    );
    for shape in &shapes {
        let apex = shape.apex.unwrap();
        let ends: Vec<Graha> = shape
            .bodies
            .iter()
            .copied()
            .filter(|g| *g != apex)
            .collect();
        assert!(holds(&rows, ends[0], WesternAspect::Opposition, ends[1]));
        for end in &ends {
            assert!(holds(&rows, apex, WesternAspect::Square, *end));
        }
        let widest = shape.widest_from_exact_deg.unwrap();
        assert!(rows.iter().all(|row| row.from_exact_deg <= row.orb_deg) && widest <= 10.0);
    }
}

#[test]
fn a_composed_batch_carries_the_shapes_the_facade_answers() {
    let sdk = context();
    let mut records = teistro::ChartRecords::default();
    records
        .read("westernConfigurations", r#"{"stelliumLeast": 3}"#)
        .unwrap();
    let instant = JulianDay::<Utc>::try_new(BIRTH).unwrap();
    let composed = sdk
        .chart()
        .compose(&[instant], &request(), &records)
        .unwrap();
    assert_eq!(
        composed.western.configurations,
        [sdk.chart()
            .western_configurations(
                &birth(&sdk),
                &AspectRequest::default(),
                teistro::ConfigurationRules::default(),
            )
            .unwrap()]
    );
    // A refusal is named under the record's root.
    let mut two = teistro::ChartRecords::default();
    let refused = two
        .read("westernConfigurations", r#"{"stelliumLeast": 2}"#)
        .unwrap_err();
    assert_eq!(refused.field(), Some("westernConfigurations.stelliumLeast"));
}
