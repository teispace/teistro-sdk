//! Gochar through the façade: the reference is the natal chart's, each
//! house is the transit counted from it, the batch is the calls one by
//! one, and the settings reach the judgement (`docs/03-design/gochar.md`).

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "tests fail by panicking and index what they found"
)]

use teistro::catalogue::{Graha, Rashi};
use teistro::gochar::{GRAHAS, GocharRules};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::settings::NodeObstruction;
use teistro::{ChartRequest, Context, Document, Ephemeris, GocharFrom, GocharRequest, UtcOffset};

/// A Kathmandu birth, 14 April 1990, the corpus's first.
const BIRTH: f64 = 2_447_995.489_583_333_5;

fn context(patch: &str) -> Context {
    Context::builder()
        .settings_json(patch)
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
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
            JulianDay::<Utc>::literal(BIRTH),
            &ChartRequest::at(place(), UtcOffset::literal(5, 45, 0)),
        )
        .unwrap()
        .value
}

fn sign_of(longitude_deg: f64) -> Rashi {
    Rashi::ALL[(longitude_deg.rem_euclid(360.0) / 30.0) as usize % 12]
}

#[test]
fn every_house_is_the_transit_counted_from_the_natal_reference() {
    let sdk = context("{}");
    let natal = natal(&sdk);
    let moon = sign_of(natal.foundation.graha(Graha::Moon).unwrap().longitude_deg);
    let lagna = Rashi::ALL[usize::from(natal.foundation.lagna_sign_index())];
    let instants: Vec<_> = (0..24)
        .map(|k| JulianDay::<Utc>::literal(2_460_676.5 + 15.25 * f64::from(k)))
        .collect();
    for (from, reference) in [(GocharFrom::Moon, moon), (GocharFrom::Lagna, lagna)] {
        let batch = sdk
            .chart()
            .gochar(
                &natal,
                &GocharRequest::over(instants.clone()).counted_from(from),
            )
            .unwrap()
            .value;
        assert_eq!(batch.len(), instants.len());
        for (reading, at) in batch.iter().zip(&instants) {
            assert_eq!(reading.reference, reference, "{from:?}");
            // The transit chart founded on its own, and each graha counted
            // from the reference by hand.
            let transit = sdk
                .chart()
                .reading(*at, &ChartRequest::at(place(), UtcOffset::UTC))
                .unwrap()
                .value;
            for (read, graha) in reading.grahas.iter().zip(GRAHAS) {
                let sign = sign_of(transit.foundation.graha(graha).unwrap().longitude_deg);
                assert_eq!(read.transit.sign, sign, "{graha:?}");
                let house = (sign as usize + 12 - reference as usize) % 12 + 1;
                assert_eq!(usize::from(read.house), house, "{graha:?}");
            }
            // The batch is the calls one by one.
            let single = sdk
                .chart()
                .gochar(&natal, &GocharRequest::at(*at).counted_from(from))
                .unwrap()
                .value;
            assert_eq!(single, vec![reading.clone()]);
        }
    }
}

#[test]
fn the_settings_reach_the_judgement() {
    let text = context("{}");
    let natal = natal(&text);
    let at = GocharRequest::at(JulianDay::<Utc>::literal(2_460_676.5));
    let reading = &text.chart().gochar(&natal, &at).unwrap().value[0];
    assert_eq!(reading.rules, GocharRules::TEXT);
    let seven = context(r#"{"gochar": {"node_obstruction": "NONE"}}"#);
    let reading = &seven.chart().gochar(&natal, &at).unwrap().value[0];
    assert_eq!(reading.rules.node_obstruction, NodeObstruction::None);
    // Under it no node obstructs anyone.
    for read in &reading.grahas {
        assert!(
            read.obstructed_by
                .iter()
                .all(|by| !matches!(by, Graha::Rahu | Graha::Ketu)),
            "{read:?}"
        );
    }
}
