//! A chart asked on a zone's clock (`docs/03-design/time-and-timezone.md`
//! §3.4, SR-10): each birth read at the offset the zone had then, the
//! foundation carrying it, the provenance stating the first birth's, and
//! every reader of the civil clock reading the chart's own. The births are
//! real Kathmandu ones: Nepal kept +05:30 until 1986 and +05:45 since.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they found"
)]

use teistro::dossier::DossierOptions;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{
    ChartRecords, ChartRequest, Context, DayClock, Ephemeris, FoundRequest, UtcOffset,
    VarshaRequest, ZoneEra, ZoneSource, ZoneSpec,
};

fn context() -> Context {
    Context::builder()
        .profile("nepali-drik")
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

fn nepal() -> ZoneSpec {
    ZoneSpec::iana("Asia/Kathmandu")
}

/// Noon in Kathmandu on 1 June 1985 (+05:30), 06:30 UTC.
const NOON_1985: f64 = 2_446_217.770_833;
/// Noon in Kathmandu on 1 June 2026 (+05:45), 06:15 UTC.
const NOON_2026: f64 = 2_461_192.760_417;
/// 23:50 in Kathmandu on 1 June 1985 (+05:30), 18:20 UTC: on today's
/// +05:45 it would be five past midnight on the 2nd.
const LATE_1985: f64 = 2_446_218.263_889;

fn at(jd: f64) -> JulianDay<Utc> {
    JulianDay::<Utc>::literal(jd)
}

fn offset(seconds: i32) -> UtcOffset {
    UtcOffset::try_from_seconds(seconds).unwrap()
}

/// A batch across Nepal's change gives each chart its own offset, each
/// founded on the zone carrying it, and the provenance states the first
/// birth's.
#[test]
fn a_zone_reads_each_birth_at_its_own_offset() {
    let sdk = context();
    let request = ChartRequest::in_zone(kathmandu(), nepal());
    let charts = sdk
        .chart()
        .readings(&[at(NOON_1985), at(NOON_2026)], &request)
        .unwrap();
    let clocks: Vec<_> = charts
        .value
        .iter()
        .map(|chart| chart.foundation.clock.clone().unwrap())
        .collect();
    assert_eq!(clocks[0].offset.seconds(), 19_800);
    assert_eq!(clocks[1].offset.seconds(), 20_700);
    assert_eq!(clocks[0].source, ZoneSource::Iana);
    assert_eq!(clocks[0].era, ZoneEra::Historical);
    assert_eq!(clocks[1].era, ZoneEra::Current);
    let stated = charts.provenance.time.clock.as_ref().unwrap();
    assert_eq!(stated, &clocks[0]);
    assert_eq!(charts.provenance.time.tzdb_version, stated.tzdb_version);
}

/// A chart asked on an offset leaves its foundation's clock out, so its
/// document and content hash are what they were; the provenance says the
/// offset was stated. A fixed zone is the same stated offset.
#[test]
fn a_stated_offset_keeps_its_document_and_says_manual() {
    let sdk = context();
    let stated = sdk
        .chart()
        .reading(
            at(NOON_2026),
            &ChartRequest::at(kathmandu(), offset(20_700)),
        )
        .unwrap();
    assert_eq!(stated.value.foundation.clock, None);
    let json = serde_json::to_value(&stated.value).unwrap();
    assert!(json["foundation"].get("clock").is_none());
    let clock = stated.provenance.time.clock.as_ref().unwrap();
    assert_eq!(
        (clock.offset.seconds(), clock.source),
        (20_700, ZoneSource::Manual)
    );
    let fixed = sdk
        .chart()
        .reading(
            at(NOON_2026),
            &ChartRequest::in_zone(kathmandu(), ZoneSpec::fixed(offset(20_700))),
        )
        .unwrap();
    assert_eq!(fixed.value, stated.value);
    assert_eq!(
        fixed.provenance.content_hash,
        stated.provenance.content_hash
    );
}

/// A chart on a zone is the chart on the offset the zone had at the birth,
/// with that clock beside it: every section the civil clock reaches (the
/// day, the Shadbala's civil day, the almanac of the day) reads it.
#[test]
fn a_zone_chart_is_the_chart_on_its_birth_offset() {
    let sdk = context();
    for (birth, seconds) in [(LATE_1985, 19_800), (NOON_2026, 20_700)] {
        let read = |request: ChartRequest| {
            sdk.chart()
                .reading(at(birth), &request.with_shadbala().with_panchanga())
                .unwrap()
                .value
        };
        let mut zoned = read(ChartRequest::in_zone(kathmandu(), nepal()));
        let stated = read(ChartRequest::at(kathmandu(), offset(seconds)));
        assert_eq!(
            zoned.foundation.clock.take().unwrap().offset.seconds(),
            seconds
        );
        // The almanac of a zone's day says its clock too.
        let day = zoned.panchanga.as_mut().unwrap();
        assert_eq!(day.clock.take().unwrap().source, ZoneSource::Iana);
        assert_eq!(zoned, stated, "the chart at {birth}");
    }
}

/// The dossier prints the birth on the chart's own clock: 23:50 on +05:30
/// in 1985, where today's +05:45 would print the next day.
#[test]
fn the_tables_read_each_birth_on_its_own_clock() {
    let sdk = context();
    let records = ChartRecords {
        dossier: Some(DossierOptions {
            sections: Some(vec!["CHART".into()]),
            ..DossierOptions::default()
        }),
        ..ChartRecords::default()
    };
    let dossier = |request: &ChartRequest| {
        sdk.chart()
            .compose(&[at(LATE_1985)], request, &records)
            .unwrap()
            .dossier
            .remove(0)
    };
    let zoned = dossier(&ChartRequest::in_zone(kathmandu(), nepal()));
    assert_eq!(
        zoned,
        dossier(&ChartRequest::at(kathmandu(), offset(19_800)))
    );
    assert_ne!(
        zoned,
        dossier(&ChartRequest::at(kathmandu(), offset(20_700)))
    );
    assert!(zoned.contains("+05:30"), "{zoned}");
}

/// The annual charts are cast on the clock the birth was: a zone's is read
/// at each return, so every return since 1986 is on +05:45.
#[test]
fn the_annual_charts_take_the_zone() {
    let sdk = context();
    let birth = sdk
        .chart()
        .reading(at(NOON_1985), &ChartRequest::in_zone(kathmandu(), nepal()))
        .unwrap()
        .value;
    let asked = VarshaRequest::through(3).at(teistro::AnnualPlace::Birth);
    let zoned = sdk
        .chart()
        .varsha(&birth, DayClock::Zone(nepal()), &asked)
        .unwrap();
    let today = sdk.chart().varsha(&birth, offset(20_700), &asked).unwrap();
    assert_eq!(format!("{zoned:?}"), format!("{today:?}"));
}

/// The record spells the zone as `almanac.days` does, in place of the
/// offset; both together are refused naming `zone`.
#[test]
fn a_json_chart_request_takes_a_zone() {
    let base = r#""instant": 2446217.770833, "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1400"#;
    let found = FoundRequest::from_json(&format!(
        r#"{{{base}, "zone": {{"kind": "IANA", "zone": "Asia/Kathmandu"}}}}"#
    ))
    .unwrap();
    assert_eq!(found.request, ChartRequest::in_zone(kathmandu(), nepal()));
    let refused = |extra: &str| {
        FoundRequest::from_json(&format!("{{{base}{extra}}}"))
            .unwrap_err()
            .field()
            .map(str::to_owned)
    };
    assert_eq!(
        refused(
            r#", "utcOffsetSeconds": 19800, "zone": {"kind": "IANA", "zone": "Asia/Kathmandu"}"#
        ),
        Some("zone".to_owned())
    );
    assert_eq!(refused(""), Some("utcOffsetSeconds".to_owned()));
    // A zone the database does not have is refused when the charts are read.
    let unknown = FoundRequest::from_json(&format!(
        r#"{{{base}, "zone": {{"kind": "IANA", "zone": "Asia/Kathmandoo"}}}}"#
    ))
    .unwrap();
    let refusal = context()
        .chart()
        .readings(&unknown.instants, &unknown.request)
        .unwrap_err();
    assert_eq!(refusal.field(), Some("zone"));
}
