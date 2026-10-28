//! The gochar sidecar through the façade (`03-design/gochar-sidecar.md`
//! §9): every transit operation reads an opened sidecar as it reads the
//! whole chart, in bytes, and a context of other settings refuses to open
//! one.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::catalogue::Graha;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{
    ChartRequest, Context, Document, Envelope, Ephemeris, GocharRequest, GocharSidecar, HitRequest,
    SadeSatiRequest, SarvatobhadraRequest, UtcOffset, canonical_json as to_hash_form,
};

fn sdk(profile: &str) -> Context {
    Context::builder()
        .profile(profile)
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
}

fn day(jd: f64) -> JulianDay<Utc> {
    JulianDay::<Utc>::try_new(jd).unwrap()
}

fn kathmandu() -> Place {
    Place::new(
        Latitude::literal(27.7),
        Longitude::literal(85.3),
        Altitude::literal(1400.0),
    )
}

/// A birth with every section a transit could read and some it does not,
/// so an opened sidecar is narrower than the document it was cut from.
fn chart(sdk: &Context) -> Envelope<Document> {
    let asked = ChartRequest::at(kathmandu(), UtcOffset::literal(5, 45, 0))
        .with_panchanga()
        .with_state()
        .with_ashtakavarga()
        .with_houses();
    sdk.chart()
        .reading(day(2_447_995.489_583_333_5), &asked)
        .unwrap()
}

/// The chart's sidecar, stored as text and read back.
fn stored(read: &Envelope<Document>) -> GocharSidecar {
    let text = GocharSidecar::cut(&read.value, &read.provenance)
        .unwrap()
        .to_text();
    GocharSidecar::read(&text).unwrap()
}

/// Four instants a quarter apart from 2025.
fn instants() -> Vec<JulianDay<Utc>> {
    (0..4)
        .map(|n| day(2_460_676.5 + 91.0 * f64::from(n)))
        .collect()
}

#[test]
fn the_transits_read_from_an_opened_sidecar_are_the_charts() {
    let sdk = sdk("conformance-baseline");
    let read = chart(&sdk);
    let sidecar = stored(&read);
    sidecar.check_chart(read.provenance.content_hash).unwrap();
    let natal = sdk.chart().natal(&sidecar).unwrap();
    assert!(natal.sections().len() < read.value.sections().len());

    let gochar = GocharRequest::over(instants()).with_ashtakavarga();
    assert_eq!(
        to_hash_form(&sdk.chart().gochar(&natal, &gochar).unwrap().value),
        to_hash_form(&sdk.chart().gochar(&read.value, &gochar).unwrap().value),
    );

    let now = day(2_461_000.5);
    let sade_sati = SadeSatiRequest::at(now);
    assert_eq!(
        to_hash_form(&sdk.chart().sade_sati(&natal, &sade_sati).unwrap().value),
        to_hash_form(
            &sdk.chart()
                .sade_sati(&read.value, &sade_sati)
                .unwrap()
                .value
        ),
    );

    let quarter = HitRequest::between(now, day(2_461_091.5));
    assert_eq!(
        to_hash_form(&sdk.chart().hits(&natal, &quarter).unwrap().value),
        to_hash_form(&sdk.chart().hits(&read.value, &quarter).unwrap().value),
    );

    let chakra = SarvatobhadraRequest::over(instants());
    assert_eq!(
        to_hash_form(&sdk.chart().sarvatobhadra(&natal, &chakra).unwrap().value),
        to_hash_form(
            &sdk.chart()
                .sarvatobhadra(&read.value, &chakra)
                .unwrap()
                .value
        ),
    );
}

#[test]
fn a_context_of_other_settings_does_not_open_it() {
    let read = chart(&sdk("conformance-baseline"));
    let sidecar = stored(&read);
    let refused = sdk("parashari-classical")
        .chart()
        .natal(&sidecar)
        .unwrap_err();
    assert_eq!(refused.field(), Some("sidecar.settings"));
}

#[test]
fn a_chart_without_its_panchanga_opens_and_its_chakra_is_refused() {
    let sdk = sdk("conformance-baseline");
    let asked = ChartRequest::at(kathmandu(), UtcOffset::literal(5, 45, 0));
    let read = sdk
        .chart()
        .reading(day(2_447_995.489_583_333_5), &asked)
        .unwrap();
    let natal = sdk.chart().natal(&stored(&read)).unwrap();
    assert_eq!(natal.sections(), ["foundation"]);
    // Its transits read; the chakra, which needs the birth's tithi, does not.
    let today = sdk
        .chart()
        .gochar(&natal, &GocharRequest::at(day(2_461_000.5)))
        .unwrap();
    assert_eq!(
        today.value[0].grahas[Graha::Moon as usize].graha,
        Graha::Moon
    );
    let refused = sdk
        .chart()
        .sarvatobhadra(&natal, &SarvatobhadraRequest::at(day(2_461_000.5)))
        .unwrap_err();
    assert_eq!(refused.field(), Some("panchanga"));
}
