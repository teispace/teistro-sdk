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

mod common;

use common::fixture;
use serde_json::Value;
use teistro::catalogue::{DashaSystem, Graha, Nakshatra};
use teistro::quantity::{Depth, JulianDay};
use teistro::{ChartRequest, Context, Document, Timeline, UtcOffset};

/// `dashas.methods.spatial.remaining_fraction` at `builtin-standard`.
const SPATIAL_FRACTION: f64 = 1e-4;
/// `dashas.methods.temporal.remaining_fraction` at `builtin-standard`.
const TEMPORAL_FRACTION: f64 = 1e-3;
/// `dashas.methods.*.periods[*][2..3]` and the span at `builtin-standard`, days.
const BOUNDARY_DAYS: f64 = 1.0;

/// The corpus's first chart with its dashas.
fn reading(patch: &str, dashas: &[DashaSystem]) -> (Context, Document) {
    common::reading(patch, |request| request.with_dashas(dashas.iter().copied()))
}

/// The document's dasha against a recorded method: first lord, balance and
/// every period of the recorded tree, and the chains at its instants
/// through the cursor rebuilt from the document.
fn agrees(sdk: &Context, document: &Document, recorded: &Value, fraction: f64) {
    assert_eq!(document.sections().last(), Some(&"dashas"));
    let dasha = &document.dashas[0];
    assert_eq!(
        (dasha.system.clone(), dasha.first_lord, dasha.seed),
        (
            DashaSystem::Vimshottari.into(),
            Graha::Saturn,
            Some(Nakshatra::Anuradha)
        )
    );
    let balance = dasha.balance.expect("a nakshatra-seeded dasha's balance");
    let remaining = recorded["remaining_fraction"].as_f64().unwrap();
    assert!(
        (balance.remaining - remaining).abs() < fraction,
        "remaining {} against {remaining}",
        balance.remaining
    );
    let days = recorded["balance"]["total_days"].as_f64().unwrap();
    assert!(
        (balance.days - days).abs() < BOUNDARY_DAYS,
        "balance {} against {days}",
        balance.days
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
    // Every built system the corpus records; Shashtihayani is built from
    // the text alone, and the recording engine has none.
    let systems: Vec<DashaSystem> = teistro::dasha::ROWS
        .iter()
        .filter_map(|row| row.system.catalogued())
        .filter(|system| {
            *system == DashaSystem::Vimshottari
                || recorded["systems"]
                    .get(system.key().to_ascii_lowercase())
                    .is_some()
        })
        .collect();
    assert_eq!(systems.len(), teistro::dasha::ROWS.len() - 1);
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
            let key = dasha.system.key().to_ascii_lowercase();
            let answer = &recorded["systems"][&key]["methods"][method];
            let at = format!("{key} {method}");
            assert_eq!(
                teistro::catalogue::Catalogued::key(dasha.first_lord),
                answer["first_lord"].as_str().unwrap(),
                "{at}"
            );
            let balance = dasha.balance.expect("a nakshatra-seeded dasha's balance");
            let years = balance.days / balance.remaining;
            let days = answer["balance"]["total_days"].as_f64().unwrap();
            assert!(
                (balance.days - days).abs() < fraction * years + BOUNDARY_DAYS,
                "{at}: balance {} against {days}",
                balance.days
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

/// Every sign-based system on the corpus's first chart, founded here with the
/// built-in ephemeris: the arudha and navamsa lagnas and the dignities the
/// façade computes must be the ones the corpus recorded for the mahadashas'
/// signs, lords and years to agree, and each antardasha with them
/// (`docs/03-design/rashi-dashas-measured.md`).
#[test]
fn a_reading_carries_every_sign_based_system_and_each_agrees_with_the_corpus() {
    let recorded = fixture("rashi-dashas/charts/c001-kathmandu-1990-04-14.json");
    let systems: Vec<DashaSystem> = teistro::dasha::RASHI_ROWS
        .iter()
        .filter_map(|row| row.system.catalogued())
        .collect();
    let (sdk, document) = reading("{}", &systems);
    for dasha in &document.dashas {
        let key = dasha.system.key().to_ascii_lowercase();
        let rows = recorded["systems"][&key]["periods"].as_array().unwrap();
        assert!(dasha.seed.is_none() && dasha.balance.is_none(), "{key}");
        for row in rows {
            let path = row[0].as_str().unwrap();
            let ours = dasha
                .periods
                .iter()
                .find(|period| period.path == path)
                .unwrap_or_else(|| panic!("{key}: no period at {path}"));
            assert_eq!(
                ours.sign.map(|sign| u64::from(sign.id())),
                row[1].as_u64(),
                "{key} {path}: sign"
            );
            assert_eq!(
                teistro::catalogue::Catalogued::key(ours.lord),
                row[2].as_str().unwrap(),
                "{key} {path}: lord"
            );
            assert!(
                (ours.interval.from.get() - row[3].as_f64().unwrap()).abs() < BOUNDARY_DAYS
                    && (ours.interval.to.get() - row[4].as_f64().unwrap()).abs() < BOUNDARY_DAYS,
                "{key} {path}: bounds"
            );
        }
        // The cursor rebuilt from the document gives the document's periods.
        let cursor = sdk.chart().dasha(&document, &dasha.system).unwrap();
        let first = cursor.mahadashas().next().unwrap();
        assert_eq!(first.sign, dasha.periods[0].sign, "{key}");
        assert!(cursor.rashi().is_some() && cursor.nakshatra().is_none());
    }
}

/// The Kalachakra on the corpus's first chart, founded here with the built-in
/// ephemeris, under both balance methods: seeded and signed, its balance and
/// every recorded period within the corpus's tolerances, carried to the
/// antardashas it stops at, and rebuilt from the document
/// (`docs/03-design/kalachakra-measured.md`).
#[test]
fn a_reading_carries_the_kalachakra_and_it_agrees_with_the_corpus() {
    let recorded = fixture("kalachakra/charts/c001-kathmandu-1990-04-14.json");
    for (patch, method, fraction) in [
        ("{}", "spatial", SPATIAL_FRACTION),
        (
            r#"{"dasha": {"balance": "TEMPORAL"}}"#,
            "temporal",
            TEMPORAL_FRACTION,
        ),
    ] {
        // The corpus's tolerance on a nakshatra's remaining fraction, in a
        // pada a quarter of the span, of a sign of at most 21 years.
        let bound = fraction * 4.0 * 21.0 * 365.25;
        let (sdk, document) = reading(patch, &[DashaSystem::Kalachakra]);
        let dasha = &document.dashas[0];
        let answer = &recorded["methods"][method];
        assert!(
            dasha.kalachakra.is_some() && dasha.seed.is_some(),
            "{method}"
        );
        assert_eq!(dasha.depth.get(), 2, "{method}: carried to the antardashas");
        let balance = dasha.balance.expect("a balance");
        assert!(
            (balance.days - answer["balance"]["total_days"].as_f64().unwrap()).abs() < bound,
            "{method}: balance {}",
            balance.days
        );
        let rows = answer["periods"].as_array().unwrap();
        assert_eq!(dasha.periods.len(), rows.len(), "{method}");
        for (ours, row) in dasha.periods.iter().zip(rows) {
            assert_eq!(ours.path, row[0].as_str().unwrap(), "{method}");
            assert_eq!(
                ours.sign.map(|sign| u64::from(sign.id())),
                row[1].as_u64(),
                "{method} {}",
                ours.path
            );
            assert!(
                (ours.interval.to.get() - row[4].as_f64().unwrap()).abs() < bound,
                "{method} {}",
                ours.path
            );
        }
        let cursor = sdk
            .chart()
            .dasha(&document, DashaSystem::Kalachakra)
            .unwrap();
        assert!(cursor.kalachakra().is_some());
        assert_eq!(
            cursor.mahadashas().count(),
            dasha
                .periods
                .iter()
                .filter(|p| !p.path.contains('/'))
                .count()
        );
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
        .with_dashas([DashaSystem::Vimshottari, DashaSystem::SudarshanaChakra]);
    let error = sdk
        .chart()
        .reading(document.foundation.instant, &request)
        .expect_err("Sudarshana Chakra is not built");
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

/// A sign-based dasha's readings come from the settings and travel with the
/// document: `conformance-baseline` keeps the recording engine's, a patch
/// selects BPHS ch. 46's, and the cursor rebuilt from each document gives that
/// document's own mahadashas back (cruxes C51, C53).
#[test]
fn a_rashi_dasha_records_its_readings_and_rebuilds_under_them() {
    use teistro::dasha::RashiRules;
    let systems = [DashaSystem::Chara, DashaSystem::Mandooka];
    for (patch, expected) in [
        ("{}", RashiRules::RECORDING_ENGINE),
        (
            r#"{"dasha": {"dual_lord": "BPHS", "rashi_start": "STRONGER"}}"#,
            RashiRules::BPHS,
        ),
    ] {
        let (sdk, document) = reading(patch, &systems);
        for dasha in &document.dashas {
            assert_eq!(dasha.rashi, Some(expected), "{patch} {}", dasha.system);
            let cursor = sdk.chart().dasha(&document, &dasha.system).unwrap();
            let rebuilt: Vec<_> = cursor
                .mahadashas()
                .map(|p| (p.sign, p.lord, p.interval))
                .collect();
            let carried: Vec<_> = dasha
                .periods
                .iter()
                .filter(|row| !row.path.contains('/'))
                .map(|row| (row.sign, row.lord, row.interval))
                .collect();
            assert_eq!(rebuilt, carried, "{patch} {}", dasha.system);
        }
    }
}

/// A system BPHS counts over the twenty-eight nakshatras with Abhijit reads
/// the Moon across its own segment of that wheel, and a document keeps the
/// grouping it was read under (`docs/03-design/dasha-kernels.md`, "The
/// 28-nakshatra wheel"; cruxes C1, C5).
#[test]
fn a_system_counted_with_abhijit_reads_the_moon_across_its_own_segment() {
    use teistro::quantity::{Altitude, Latitude, Longitude, Place, Utc};
    use teistro::{Ephemeris, catalogue::ChartKind};

    let sdk = Context::builder()
        .profile("conformance-baseline")
        .settings_json(
            r#"{"dasha": {"balance": "TEMPORAL", "ashtottari_grouping": "FOUR_AND_THREE"}}"#,
        )
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap();
    let place = Place::new(
        Latitude::try_new(27.7172).unwrap(),
        Longitude::try_new(85.324).unwrap(),
        Altitude::try_new(1400.0).unwrap(),
    );
    let offset = UtcOffset::try_from_seconds(20_700).unwrap();
    let moon_at = |jd: f64| {
        sdk.chart()
            .found(
                JulianDay::<Utc>::literal(jd),
                &place,
                offset,
                ChartKind::Natal,
            )
            .unwrap()
            .value
            .graha(Graha::Moon)
            .unwrap()
            .longitude_deg
    };
    let (abhijit_from, abhijit_to) = (276.0 + 40.0 / 60.0, 280.0 + 53.0 / 60.0 + 20.0 / 3600.0);
    // A birth whose Moon stands well inside Abhijit's four degrees.
    let birth = (0..400)
        .map(|step| 2_447_995.5 + f64::from(step) * 0.1)
        .find(|jd| (abhijit_from + 0.5..abhijit_to - 0.5).contains(&moon_at(*jd)))
        .expect("the Moon passes Abhijit within forty days");

    let systems = [
        DashaSystem::Vimshottari,
        DashaSystem::Ashtottari,
        DashaSystem::Shashtihayani,
    ];
    let document = sdk
        .chart()
        .reading(
            JulianDay::<Utc>::literal(birth),
            &ChartRequest::at(place, offset).with_dashas(systems),
        )
        .unwrap()
        .value;
    let [vimshottari, ashtottari, shashtihayani] = &document.dashas[..] else {
        panic!("three dashas");
    };
    // Both texts' systems give Abhijit to Saturn; the seed is still the
    // catalogue's Uttarashadha, since Abhijit is a segment and no member.
    assert_eq!(ashtottari.first_lord, Graha::Saturn);
    assert_eq!(shashtihayani.first_lord, Graha::Saturn);
    assert_eq!(shashtihayani.seed, Some(Nakshatra::UttaraAshadha));
    // The twenty-eight's systems read the Moon across Abhijit itself, from
    // where it entered at 276°40′ to where it leaves at 280°53′20″;
    // Vimshottari reads it across Uttarashadha.
    for dasha in [ashtottari, shashtihayani] {
        let span = dasha.moon_span.expect("a temporal balance's span");
        assert!(span.contains_inclusive(JulianDay::<Utc>::literal(birth)));
        for (at, bound) in [(span.from.get(), abhijit_from), (span.to.get(), abhijit_to)] {
            let off = (moon_at(at) - bound + 180.0).rem_euclid(360.0) - 180.0;
            let off = off.abs();
            assert!(off < 1e-5, "{}: {off}° from {bound}", dasha.system);
        }
    }
    let nakshatra = vimshottari.moon_span.expect("Vimshottari's span");
    let abhijit = ashtottari.moon_span.unwrap();
    assert!(nakshatra.from.get() < abhijit.from.get() && nakshatra.to.get() < abhijit.to.get());
    // The grouping travels with the document, and the cursor rebuilt from a
    // stored copy gives the text's Ashtottari back.
    assert_eq!(
        ashtottari.rules.ashtottari_grouping,
        teistro::settings::AshtottariGrouping::FourAndThree
    );
    let stored: Document =
        serde_json::from_str(&serde_json::to_string(&document).unwrap()).unwrap();
    let cursor = sdk.chart().dasha(&stored, DashaSystem::Ashtottari).unwrap();
    assert_eq!(
        cursor.mahadashas().next().unwrap().interval,
        ashtottari.periods[0].interval
    );
}
