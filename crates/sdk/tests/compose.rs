//! A chart request composed whole (`docs/03-design/mcp-server.md`, step
//! 2): every section `compose` answers is the call it composes, chart by
//! chart, and the records refuse what they cannot both answer.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they found"
)]

use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{
    ChartRecords, ChartRequest, ConsiderationRules, Context, DignityRequest, Ephemeris,
    FortitudeRequest, FortuneRule, KpRequest, Lot, LotRequest, PerfectionRequest, PrashnaRequest,
    UtcOffset,
};

fn context() -> Context {
    Context::builder()
        .profile("conformance-baseline")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
}

fn kathmandu() -> ChartRequest {
    let place = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );
    ChartRequest::at(place, UtcOffset::try_from_seconds(20_700).unwrap())
}

/// Two births a year apart, so a section read per chart is two rows.
fn instants() -> [JulianDay<Utc>; 2] {
    [
        JulianDay::<Utc>::literal(2_447_000.25),
        JulianDay::<Utc>::literal(2_447_365.75),
    ]
}

#[test]
fn every_section_is_the_call_it_composes() {
    let sdk = context();
    let request = kathmandu();
    let records = ChartRecords {
        fortitudes: Some(FortitudeRequest::default()),
        lots: Some(LotRequest::default()),
        considerations: Some(ConsiderationRules::default()),
        perfection: Some(
            PerfectionRequest::from_json(r#"{"querent": "VENUS", "quesited": "MARS"}"#).unwrap(),
        ),
        kp: Some(KpRequest::new().under_any_ayanamsha()),
        prashna: Some(PrashnaRequest::from_json("{}").unwrap()),
        ..ChartRecords::default()
    }
    .checked()
    .unwrap();
    let composed = sdk
        .chart()
        .compose(&instants(), &request, &records)
        .unwrap();
    let documents = &composed.founded.value;
    assert_eq!(documents.len(), 2);
    assert_eq!(composed.hashes.len(), 2);
    for (at, document) in documents.iter().enumerate() {
        let fortitudes = sdk
            .chart()
            .fortitudes(document, &FortitudeRequest::default())
            .unwrap();
        assert_eq!(composed.fortitudes[at], fortitudes);
        assert_eq!(
            composed.lots[at],
            sdk.chart()
                .lots_with_request(document, &Lot::ALL, LotRequest::default())
                .unwrap()
        );
        assert_eq!(
            composed.considerations[at],
            sdk.chart()
                .considerations(
                    document,
                    &FortitudeRequest::default(),
                    ConsiderationRules::default()
                )
                .unwrap()
        );
        let quesited = records.perfection.as_ref().unwrap();
        assert_eq!(
            composed.perfections[at].0,
            sdk.chart()
                .perfection_in(document, &fortitudes, quesited)
                .unwrap()
        );
        // A KP record naming no clock reads the request's own.
        let kp = records.kp.unwrap().on_clock(request.offset());
        assert_eq!(
            composed.kp[at],
            sdk.chart().kp_reading(document, &kp).unwrap()
        );
        assert_eq!(
            composed.prashna[at],
            sdk.chart()
                .prashna(document, records.prashna.as_ref().unwrap())
                .unwrap()
        );
        // A prashna weighs the seven by their Shadbala, so the charts carry it.
        assert!(
            document.shadbala.is_some(),
            "chart {at} carries its Shadbala"
        );
    }
    for section in [
        composed.gochar.len(),
        composed.hits.len(),
        composed.dignities.len(),
    ] {
        assert_eq!(section, 0, "a record not sent is a section not read");
    }
    assert!(composed.readings.is_empty() && composed.plans.is_empty());
}

#[test]
fn the_dignities_asked_twice_are_refused_by_the_record_that_repeats_them() {
    let refused = ChartRecords {
        dignities: Some(DignityRequest::default()),
        fortitudes: Some(FortitudeRequest::default()),
        ..ChartRecords::default()
    }
    .checked()
    .unwrap_err();
    assert_eq!(refused.field(), Some("dignities"));
    assert!(refused.hint().unwrap().contains("fortitudes.dignities"));
}

#[test]
fn the_records_widen_the_request_by_what_they_read_off_the_charts() {
    let request = kathmandu();
    assert_eq!(ChartRecords::default().widen(request.clone()), request);
    let lots = LotRequest::default().with_fortune(FortuneRule::DayAndNight);
    let widened = ChartRecords {
        lots: Some(lots),
        ..ChartRecords::default()
    }
    .widen(request.clone());
    assert_eq!(
        widened,
        request.clone().with_lot_rules(lots),
        "the time lords release from the lots asked for"
    );
    let weighed = ChartRecords {
        prashna: Some(PrashnaRequest::from_json("{}").unwrap()),
        ..ChartRecords::default()
    }
    .widen(request.clone());
    assert_eq!(
        weighed,
        request.with_shadbala(),
        "a prashna weighs the seven by their Shadbala"
    );
}
