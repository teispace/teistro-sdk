//! A chart document's dasha, computed end to end: founded with the built-in
//! ephemeris under the corpus's own profile and compared with what the
//! corpus recorded for its first chart, under both balance methods
//! (`docs/03-design/dasha-measured.md`).
//!
//! `crates/dasha/tests/baseline.rs` holds the kernel to the recorded Moon.
//! This holds the whole path a consumer takes — the Moon the SDK computes,
//! the nakshatra span it searches in the chart's own frame, the settings'
//! rules and depth — so the bounds are the corpus's published tolerances
//! for the built-in `standard` tier (`fixtures/tolerances.json`).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use serde_json::Value;
use teistro::catalogue::{DashaSystem, Graha, Nakshatra};
use teistro::quantity::{Altitude, Depth, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{ChartRequest, Context, Document, Ephemeris, Timeline, UtcOffset};

/// `dashas.methods.spatial.remaining_fraction` at `builtin-standard`.
const SPATIAL_FRACTION: f64 = 1e-4;
/// `dashas.methods.temporal.remaining_fraction` at `builtin-standard`.
const TEMPORAL_FRACTION: f64 = 1e-3;
/// `dashas.methods.*.periods[*][2..3]` and the span at `builtin-standard`, days.
const BOUNDARY_DAYS: f64 = 1.0;

fn fixture(name: &str) -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/baseline")
        .join(name);
    serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap()
}

/// The corpus's first chart read under `conformance-baseline`, with a
/// settings patch.
fn reading(patch: &str, dashas: &[DashaSystem]) -> (Context, Document) {
    let sdk = Context::builder()
        .profile("conformance-baseline")
        .settings_json(patch)
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("the conformance profile and the built-in ephemeris");
    let place = Place::new(
        Latitude::try_new(27.7172).unwrap(),
        Longitude::try_new(85.324).unwrap(),
        Altitude::try_new(1400.0).unwrap(),
    );
    let request = ChartRequest::at(place, UtcOffset::try_from_seconds(20_700).unwrap())
        .with_dashas(dashas.iter().copied());
    let document = sdk
        .chart()
        .reading(JulianDay::<Utc>::literal(2_447_995.489_583_333_5), &request)
        .expect("a reading")
        .value;
    (sdk, document)
}

/// The document's dasha against a recorded method: first lord, balance and
/// every period of the recorded tree, and the chains at its instants
/// through the cursor rebuilt from the document.
fn agrees(sdk: &Context, document: &Document, recorded: &Value, fraction: f64) {
    assert_eq!(document.sections().last(), Some(&"dashas"));
    let dasha = &document.dashas[0];
    assert_eq!(
        (dasha.system, dasha.first_lord, dasha.seed),
        (DashaSystem::Vimshottari, Graha::Saturn, Nakshatra::Anuradha)
    );
    let remaining = recorded["remaining_fraction"].as_f64().unwrap();
    assert!(
        (dasha.balance.remaining - remaining).abs() < fraction,
        "remaining {} against {remaining}",
        dasha.balance.remaining
    );
    let days = recorded["balance"]["total_days"].as_f64().unwrap();
    assert!(
        (dasha.balance.days - days).abs() < BOUNDARY_DAYS,
        "balance {} against {days}",
        dasha.balance.days
    );

    let rows = recorded["periods"].as_array().unwrap();
    assert_eq!(dasha.depth.get(), 3, "the settings' default depth");
    assert!(dasha.periods.len() >= rows.len());
    let mut worst = 0.0_f64;
    for row in rows {
        let path = row[0].as_str().unwrap();
        let ours = dasha
            .periods
            .iter()
            .find(|period| period.path == path)
            .unwrap_or_else(|| panic!("no period at {path}"));
        assert_eq!(
            teistro::catalogue::Catalogued::key(ours.lord),
            row[1].as_str().unwrap(),
            "{path}"
        );
        worst = worst
            .max((ours.interval.from.get() - row[2].as_f64().unwrap()).abs())
            .max((ours.interval.to.get() - row[3].as_f64().unwrap()).abs());
    }
    assert!(worst < BOUNDARY_DAYS, "worst boundary {worst} days");

    let cursor = sdk
        .chart()
        .dasha(document, DashaSystem::Vimshottari)
        .unwrap();
    for active in recorded["active_at"].as_array().unwrap() {
        let chain = cursor.at(
            JulianDay::literal(active["jd"].as_f64().unwrap()),
            Depth::try_new(5).unwrap(),
        );
        let lords: Vec<&str> = chain
            .iter()
            .map(|period| teistro::catalogue::Catalogued::key(period.lord))
            .collect();
        let recorded: Vec<&str> = active["chain"]
            .as_array()
            .unwrap()
            .iter()
            .map(|link| link["lord"].as_str().unwrap())
            .collect();
        assert_eq!(lords, recorded);
    }
}

#[test]
fn a_reading_carries_the_corpus_s_spatial_dasha() {
    let (sdk, document) = reading("{}", &[DashaSystem::Vimshottari]);
    let recorded = fixture("charts/c001-kathmandu-1990-04-14.json");
    let spatial = &recorded["dashas"]["methods"]["spatial"];
    agrees(&sdk, &document, spatial, SPATIAL_FRACTION);
    assert_eq!(
        document.dashas[0].moon_span, None,
        "a spatial balance reads no span"
    );
}

#[test]
fn a_temporal_balance_searches_the_moon_s_nakshatra_in_the_chart_s_frame() {
    let (sdk, document) = reading(
        r#"{"dasha": {"balance": "TEMPORAL"}}"#,
        &[DashaSystem::Vimshottari],
    );
    let recorded = fixture("variants/c001-kathmandu-1990-04-14--temporal.json");
    let temporal = &recorded["dashas"]["methods"]["temporal"];
    agrees(&sdk, &document, temporal, TEMPORAL_FRACTION);
    let span = document.dashas[0]
        .moon_span
        .expect("the span the balance read");
    let entry = temporal["nakshatra_span"]["entry_jd"].as_f64().unwrap();
    let exit = temporal["nakshatra_span"]["exit_jd"].as_f64().unwrap();
    assert!(
        (span.from.get() - entry).abs() < 1e-3,
        "entry {} against {entry}",
        span.from.get()
    );
    assert!(
        (span.to.get() - exit).abs() < 1e-3,
        "exit {} against {exit}",
        span.to.get()
    );
}

/// Every other built system on the corpus's first chart, founded here with
/// the built-in ephemeris, against what the corpus recorded from its own
/// Moon: the first lord, the balance and every period of the recorded tree,
/// under both balance methods (`docs/03-design/dasha-systems-measured.md`).
#[test]
fn a_reading_carries_every_built_system_and_each_agrees_with_the_corpus() {
    let recorded = fixture("dasha-systems/charts/c001-kathmandu-1990-04-14.json");
    let systems: Vec<DashaSystem> = teistro::dasha::ROWS.iter().map(|row| row.system).collect();
    for (patch, method, fraction) in [
        ("{}", "spatial", SPATIAL_FRACTION),
        (
            r#"{"dasha": {"balance": "TEMPORAL"}}"#,
            "temporal",
            TEMPORAL_FRACTION,
        ),
    ] {
        let (_, document) = reading(patch, &systems);
        assert_eq!(document.dashas.len(), systems.len());
        for dasha in document.dashas.iter().skip(1) {
            let key = teistro::catalogue::Catalogued::key(dasha.system).to_ascii_lowercase();
            let answer = &recorded["systems"][&key]["methods"][method];
            let at = format!("{key} {method}");
            assert_eq!(
                teistro::catalogue::Catalogued::key(dasha.first_lord),
                answer["first_lord"].as_str().unwrap(),
                "{at}"
            );
            let years = dasha.balance.days / dasha.balance.remaining;
            let days = answer["balance"]["total_days"].as_f64().unwrap();
            assert!(
                (dasha.balance.days - days).abs() < fraction * years + BOUNDARY_DAYS,
                "{at}: balance {} against {days}",
                dasha.balance.days
            );
            let rows = answer["periods"].as_array().unwrap();
            assert!(dasha.periods.len() >= rows.len(), "{at}");
            for row in rows {
                let path = row[0].as_str().unwrap();
                let ours = dasha
                    .periods
                    .iter()
                    .find(|period| period.path == path)
                    .unwrap_or_else(|| panic!("{at}: no period at {path}"));
                assert_eq!(
                    teistro::catalogue::Catalogued::key(ours.lord),
                    row[1].as_str().unwrap(),
                    "{at} {path}"
                );
                assert!(
                    (ours.interval.from.get() - row[2].as_f64().unwrap()).abs() < BOUNDARY_DAYS
                        && (ours.interval.to.get() - row[3].as_f64().unwrap()).abs()
                            < BOUNDARY_DAYS,
                    "{at} {path}"
                );
            }
        }
    }
}

#[test]
fn a_dasha_not_built_yet_is_refused_by_its_place_and_a_stored_document_reads_back() {
    let (sdk, document) = reading("{}", &[DashaSystem::Vimshottari]);
    let json = serde_json::to_string(&document).unwrap();
    let stored: Document = serde_json::from_str(&json).unwrap();
    assert_eq!(stored, document, "the section round-trips");
    // The cursor rebuilt from the stored document gives the same periods.
    let cursor = sdk
        .chart()
        .dasha(&stored, DashaSystem::Vimshottari)
        .unwrap();
    let first = cursor.mahadashas().next().unwrap();
    assert_eq!(first.interval, stored.dashas[0].periods[0].interval);

    let place = document.foundation.place;
    let request = ChartRequest::at(place, UtcOffset::try_from_seconds(20_700).unwrap())
        .with_dashas([DashaSystem::Vimshottari, DashaSystem::Kalachakra]);
    let error = sdk
        .chart()
        .reading(document.foundation.instant, &request)
        .expect_err("Kalachakra is not built");
    assert_eq!(error.field(), Some("dashas[1]"));
    assert!(
        error
            .hint()
            .is_some_and(|hint| hint.contains("VIMSHOTTARI")),
        "{error:?}"
    );

    let missing = sdk
        .chart()
        .dasha(&document, DashaSystem::Yogini)
        .unwrap_err();
    assert_eq!(missing.field(), Some("system"));
}
