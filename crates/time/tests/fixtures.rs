//! The zone resolutions of spike 1's fixtures: every chart's civil time
//! resolves to the baseline engine's instant, offset, source and era
//! under the embedded tzdb, with the deliberate differences named
//! (`docs/05-testing/01-golden-vectors.md`, the baseline conventions).

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests fail by panicking"
)]

use std::path::Path;

use serde::Deserialize;
use teistro_calendar::CalendarDate;
use teistro_core::catalogue::Calendar;
use teistro_core::quantity::Longitude;
use teistro_core::settings::DstOverlap;
use teistro_time::{
    CivilDateTime, CivilTime, EmbeddedTzdb, Policy, Warning, ZoneEra, ZoneSource, ZoneSpec, resolve,
};

#[derive(Deserialize)]
struct Chart {
    id: String,
    input: Input,
}

#[derive(Deserialize)]
struct Input {
    place: PlaceIn,
    local: Local,
    resolved: ResolvedIn,
}

#[derive(Deserialize)]
struct PlaceIn {
    longitude: f64,
}

#[derive(Deserialize)]
struct Local {
    calendar: String,
    date: String,
    time: String,
    is_lmt: bool,
    dst_choice: Option<String>,
}

#[derive(Deserialize)]
struct ResolvedIn {
    jd_ut: f64,
    tz_offset_min: i32,
    iana: String,
    tz_source: String,
    tz_era: String,
    warnings: Vec<String>,
}

/// Charts whose era the baseline engine labelled by comparing with the
/// offset in force when it exported (a northern summer), where the SDK
/// compares with the offsets the zone applies in the database's year
/// (deliberate difference eleven): Sydney and Auckland in their summer
/// time, Berlin and New York in their standard time, and the later
/// occurrence of the New York fold, EST; every one an offset its zone
/// still applies each year.
const ERA_DIFFERENCES: [&str; 5] = ["c018", "c019", "c029", "c035", "c037"];

/// The baseline rounds local mean time to the whole minute (deliberate
/// difference two); the SDK keeps the second.
const LMT_TOLERANCE_DAYS: f64 = 31.0 / 86_400.0;
const TOLERANCE_DAYS: f64 = 1.0 / 86_400.0;

fn charts() -> Vec<Chart> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/baseline/charts");
    let mut charts: Vec<Chart> = std::fs::read_dir(&dir)
        .expect("the fixtures directory")
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|e| e == "json"))
        .map(|entry| {
            let text = std::fs::read_to_string(entry.path()).expect("a readable fixture");
            serde_json::from_str(&text)
                .unwrap_or_else(|e| panic!("{}: {e}", entry.path().display()))
        })
        .collect();
    charts.sort_by(|a, b| a.id.cmp(&b.id));
    charts
}

#[test]
#[allow(
    clippy::too_many_lines,
    reason = "one pass over the corpus, checked field by field"
)]
fn every_fixture_resolves_to_the_baselines_instant_and_metadata() {
    let charts = charts();
    assert_eq!(charts.len(), 55);
    let db = EmbeddedTzdb::shared();
    let mut era_differences = Vec::new();
    for chart in &charts {
        let local = &chart.input.local;
        let expected = &chart.input.resolved;
        assert_eq!(local.calendar, "gregorian", "{}", chart.id);
        let mut parts = local.date.split('-').map(|p| p.parse::<i32>().unwrap());
        let (y, m, d) = (
            parts.next().unwrap(),
            parts.next().unwrap(),
            parts.next().unwrap(),
        );
        let civil = CivilDateTime::at(
            CalendarDate::defined(
                Calendar::Gregorian,
                y,
                u8::try_from(m).unwrap(),
                u8::try_from(d).unwrap(),
            ),
            local.time.parse::<CivilTime>().unwrap(),
        );
        let zone = if local.is_lmt {
            ZoneSpec::local_mean(Longitude::try_new(chart.input.place.longitude).unwrap())
        } else {
            ZoneSpec::iana(expected.iana.clone())
        };
        let policy = Policy {
            overlap: match local.dst_choice.as_deref() {
                Some("later") => DstOverlap::Later,
                _ => DstOverlap::Earlier,
            },
            ..Policy::default()
        };
        let resolved =
            resolve(&civil, &zone, &policy, db).unwrap_or_else(|e| panic!("{}: {e}", chart.id));
        let tolerance = if local.is_lmt {
            LMT_TOLERANCE_DAYS
        } else {
            TOLERANCE_DAYS
        };
        assert!(
            (resolved.instant.get() - expected.jd_ut).abs() < tolerance,
            "{}: instant {} against {} ({:.1} s off)",
            chart.id,
            resolved.instant,
            expected.jd_ut,
            (resolved.instant.get() - expected.jd_ut) * 86_400.0
        );
        let minutes = (f64::from(resolved.zone.offset.seconds()) / 60.0).round();
        assert!(
            (minutes - f64::from(expected.tz_offset_min)).abs() < 0.5,
            "{}: offset {}",
            chart.id,
            resolved.zone.offset
        );
        let source = match expected.tz_source.as_str() {
            "iana" => ZoneSource::Iana,
            "lmt" => ZoneSource::LocalMean,
            other => panic!("{}: source {other}", chart.id),
        };
        assert_eq!(resolved.zone.source, source, "{}", chart.id);
        match expected.tz_era.as_str() {
            "current" => assert_eq!(resolved.zone.era, ZoneEra::Current, "{}", chart.id),
            "historical" => {
                if resolved.zone.era == ZoneEra::Current {
                    era_differences.push(chart.id.clone());
                }
            }
            "lmt" => assert_eq!(resolved.zone.source, ZoneSource::LocalMean, "{}", chart.id),
            other => panic!("{}: era {other}", chart.id),
        }
        let ambiguous = expected.warnings.iter().any(|w| w == "DST_AMBIGUOUS");
        assert_eq!(
            resolved.zone.has(Warning::DstAmbiguous),
            ambiguous,
            "{}",
            chart.id
        );
        if !local.is_lmt {
            let differs = expected
                .warnings
                .iter()
                .any(|w| w == "OFFSET_DIFFERS_FROM_CURRENT_RULES");
            assert_eq!(
                resolved.zone.has(Warning::OffsetDiffersFromCurrentRules),
                differs && !ERA_DIFFERENCES.iter().any(|id| chart.id.starts_with(id)),
                "{}: warnings {:?} against {:?}",
                chart.id,
                resolved.zone.warnings,
                expected.warnings
            );
        }
        assert!(resolved.zone.time_known);
    }
    let expected_differences: Vec<String> = charts
        .iter()
        .filter(|c| ERA_DIFFERENCES.iter().any(|id| c.id.starts_with(id)))
        .map(|c| c.id.clone())
        .collect();
    assert_eq!(era_differences, expected_differences);
}
