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
