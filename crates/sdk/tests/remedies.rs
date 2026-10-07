//! Remedies through the façade: every step read off one chart, the running
//! Vimśottarī daśā at an instant the caller names
//! (`docs/03-design/remedies.md`).

#![allow(
    clippy::unwrap_used,
    reason = "tests fail by panicking on what they asked for"
)]

use teistro::catalogue::DashaSystem;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{ChartRequest, Context, Document, Ephemeris, RemedyRequest, UtcOffset};

/// Kathmandu, 14 April 1990, the corpus's first chart.
const BIRTH: f64 = 2_447_995.489_583_333_5;

fn chart(sdk: &Context, dashas: bool) -> Document {
    let place = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );
    let request = ChartRequest::at(place, UtcOffset::literal(5, 45, 0));
    let request = if dashas {
        request.with_dashas([DashaSystem::Vimshottari])
    } else {
        request
    };
    sdk.chart()
        .reading(JulianDay::<Utc>::literal(BIRTH), &request)
        .unwrap()
        .value
}

#[test]
fn every_step_is_read_off_one_chart() {
    let sdk = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap();
    let document = chart(&sdk, true);
    let plain = sdk
        .chart()
        .remedies(&document, &RemedyRequest::default())
        .unwrap();
    assert_eq!(plain.subjects.antardasha, None, "no instant, no daśā");
    assert_eq!(
        plain.functional.rows.len(),
        7,
        "the seven own signs; the nodes none"
    );
    // Each subject has its śānti, in the subjects' order.
    let named: Vec<_> = plain
        .subjects
        .subjects
        .iter()
        .map(|one| one.graha)
        .collect();
    let prescribed: Vec<_> = plain.shantis.iter().map(|one| one.graha).collect();
    assert_eq!(prescribed, named);
    assert_eq!(
        plain.ishta_devata,
        sdk.chart()
            .ishta_devata(&document, plain.rules.devata)
            .unwrap()
    );

    // Twenty years on, an antardaśā runs and its printed śānti comes with it.
    let asked = RemedyRequest {
        at: Some(BIRTH + 20.0 * 365.25),
        ..RemedyRequest::default()
    };
    let read = sdk.chart().remedies(&document, &asked).unwrap();
    let running = read.subjects.antardasha.unwrap();
    assert_eq!(running.holds.len(), running.shanti.conditions.len());
    assert!(
        read.subjects
            .subjects
            .iter()
            .any(|one| one.graha == running.shanti.antardasha),
        "the antardaśā lord is a subject"
    );

    // Without the daśā the instant cannot be read, and the refusal says why.
    let refused = sdk
        .chart()
        .remedies(&chart(&sdk, false), &asked)
        .unwrap_err();
    assert_eq!(refused.field(), Some("dashas"));
    let nan = RemedyRequest {
        at: Some(f64::NAN),
        ..RemedyRequest::default()
    };
    assert_eq!(
        sdk.chart().remedies(&document, &nan).unwrap_err().field(),
        Some("remedies.at")
    );
}
