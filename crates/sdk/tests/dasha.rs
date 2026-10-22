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
    // Every built system the corpus records; Shodashottari, Shattrimsha-sama
    // and Shashtihayani are built from the text alone, and the recording
    // engine has none of them.
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
    assert_eq!(systems.len(), teistro::dasha::ROWS.len() - 3);
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

/// The sign-based rows the corpus does not record: the Sthira dasa (C129),
/// which a chart with no Brahma graha refuses, the Yogardha (C131) and the
/// Sudarshana chakra (C734). Named so a recording of any, or another
/// unrecorded row, fails.
const UNRECORDED: [DashaSystem; 3] = [
    DashaSystem::Sthira,
    DashaSystem::Yogardha,
    DashaSystem::SudarshanaChakra,
];

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
        .filter(|system| {
            let key = system.key().to_ascii_lowercase();
            let is_recorded = recorded["systems"].get(&key).is_some();
            assert_eq!(!is_recorded, UNRECORDED.contains(system), "{key}");
            is_recorded
        })
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

/// The Sudarshana chakra through the façade (BPHS ch. 74 vv. 21–26, C734):
/// a year a house forward from the lagna, and the twelve again after them
/// although the default settings end every other cycle.
#[test]
fn a_reading_carries_the_sudarshana_chakra_and_its_houses_recur() {
    let (sdk, document) = reading("{}", &[DashaSystem::SudarshanaChakra]);
    let dasha = &document.dashas[0];
    let signs: Vec<u16> = dasha
        .periods
        .iter()
        .filter(|period| period.level() == 1)
        .map(|period| period.sign.unwrap().id())
        .collect();
    assert_eq!(signs.len(), 12);
    assert!(
        signs.windows(2).all(|pair| pair[1] == (pair[0] + 1) % 12),
        "{signs:?}"
    );
    let cursor = sdk
        .chart()
        .dasha(&document, DashaSystem::SudarshanaChakra)
        .unwrap();
    let first = cursor.mahadashas().next().unwrap();
    let year = first.interval.days();
    let later = cursor.at(
        JulianDay::literal(first.interval.from.get() + 13.5 * year),
        Depth::try_new(1).unwrap(),
    );
    let second = later.deepest().unwrap();
    assert_eq!(
        second.sign.map(teistro::catalogue::Rashi::id),
        Some(signs[1])
    );
}

/// The Karaka dasha through the façade (BPHS ch. 46 v. 178, C737): the
/// karakas of the settings' scheme from the Atmakaraka `jaimini` names,
/// each for a whole count of signs, and rebuilt from the document with the
/// division its second level shows.
#[test]
fn a_reading_carries_the_karaka_dasha_under_either_scheme_and_rebuilds_it() {
    for (scheme, lords, division) in [("SEVEN", 7, "PROPORTIONAL"), ("EIGHT", 8, "EQUAL")] {
        let patch = format!(
            r#"{{"jaimini": {{"chara_karakas": "{scheme}"}},
                "dasha": {{"karaka_division": "{division}", "depth": {{"KARAKA": 2}}}}}}"#
        );
        let (sdk, document) = reading(&patch, &[DashaSystem::Karaka]);
        let dasha = &document.dashas[0];
        assert!(dasha.seed.is_none() && dasha.balance.is_none(), "{scheme}");
        let mahadashas: Vec<_> = dasha
            .periods
            .iter()
            .filter(|row| row.level() == 1)
            .collect();
        assert_eq!(mahadashas.len(), lords, "{scheme}");
        assert_eq!(
            mahadashas[0].lord,
            sdk.chart()
                .jaimini(&document)
                .unwrap()
                .karakamsha
                .atmakaraka,
            "{scheme}"
        );
        let year = 365.25;
        for row in &mahadashas {
            let years = row.interval.days() / year;
            assert!(
                (years - years.round()).abs() < 1e-9 && (1.0..=12.0).contains(&years),
                "{scheme}: {years}"
            );
        }
        // Rebuilt, a period's sub-periods are the ones the document stores,
        // under whichever division it was written with.
        let rebuilt = sdk.chart().dasha(&document, DashaSystem::Karaka).unwrap();
        assert!(rebuilt.graha().is_some());
        let first = rebuilt.mahadashas().next().unwrap();
        let stored: Vec<_> = dasha
            .periods
            .iter()
            .filter(|row| row.level() == 2)
            .take(lords)
            .collect();
        for (ours, row) in rebuilt.children(&first).zip(&stored) {
            assert_eq!(ours.lord, row.lord, "{scheme}");
            assert!(
                (ours.interval.from.get() - row.interval.from.get()).abs() < 1e-6
                    && (ours.interval.to.get() - row.interval.to.get()).abs() < 1e-6,
                "{scheme} {}",
                row.path
            );
        }
    }
}

/// Vimshottari's lords and years, from Ketu.
const VIMSHOTTARI_LORDS: [(Graha, f64); 9] = [
    (Graha::Ketu, 7.0),
    (Graha::Venus, 20.0),
    (Graha::Sun, 6.0),
    (Graha::Moon, 10.0),
    (Graha::Mars, 7.0),
    (Graha::Rahu, 18.0),
    (Graha::Jupiter, 16.0),
    (Graha::Saturn, 19.0),
    (Graha::Mercury, 17.0),
];

/// The Tara dasha through the façade (BPHS ch. 46 vv. 207 to 209, C735):
/// over a year of births, Vimshottari from the strongest of the seven in a
/// kendra by the chart's shadbala under either measure, or refused where
/// none stands in one; the same whether the request asked for the shadbala
/// or not, and rebuilt from the document. Births where two or more stand in
/// kendras are counted, so the choice of the strongest is tested at all.
#[test]
fn a_reading_carries_the_tara_dasha_from_the_strongest_graha_in_a_kendra() {
    let mut contested = 0;
    for measure in ["SHADBALA_TOTAL", "SHADBALA_RATIO"] {
        let sdk = Context::builder()
            .profile("conformance-baseline")
            .settings_json(format!(
                r#"{{"dasha": {{"strongest_graha": "{measure}", "depth": {{"TARA": 2}}}}}}"#
            ))
            .ephemeris([teistro::Ephemeris::Builtin])
            .build()
            .unwrap();
        for step in 0..12 {
            let birth = JulianDay::literal(2_447_995.49 + f64::from(step) * 31.3);
            contested += usize::from(tara_at(&sdk, measure, birth).unwrap_or(false));
        }
    }
    assert!(contested >= 4, "{contested} births with a contest");
}

/// One birth's Tara dasha checked as the test above says: whether two or
/// more of the seven stood in kendras, or nothing where it was refused.
fn tara_at(sdk: &Context, measure: &str, birth: JulianDay<teistro::quantity::Utc>) -> Option<bool> {
    use teistro::catalogue::Rashi;
    use teistro::quantity::{Altitude, Latitude, Longitude, Place};
    use teistro::strength::shadbala::GrahaShadbala;
    use teistro::{House, Status};

    let place = Place::new(
        Latitude::try_new(27.7172).unwrap(),
        Longitude::try_new(85.324).unwrap(),
        Altitude::try_new(1400.0).unwrap(),
    );
    let offset = UtcOffset::try_from_seconds(20_700).unwrap();
    let ask = |shadbala: bool| {
        let request = ChartRequest::at(place, offset).with_dashas([DashaSystem::Tara]);
        let request = if shadbala {
            request.with_shadbala()
        } else {
            request
        };
        sdk.chart()
            .reading(birth, &request)
            .map(|reading| reading.value)
    };
    let document = match ask(true) {
        Ok(document) => document,
        Err(error) => {
            // A chart with none of the seven in a kendra, by name.
            assert_eq!(error.status, Status::Unsupported, "{error}");
            assert!(error.message.contains("kendra"), "{error}");
            return None;
        }
    };
    let strength = |entry: &GrahaShadbala| {
        if measure == "SHADBALA_RATIO" {
            entry.rupas / entry.required_rupas
        } else {
            entry.virupas
        }
    };
    let lagna = Rashi::of_longitude(sdk.chart().angles(&document).unwrap().ascendant_deg);
    let kendra: Vec<&GrahaShadbala> = document
        .shadbala
        .as_ref()
        .unwrap()
        .grahas
        .iter()
        .filter(|entry| {
            let at = document
                .foundation
                .graha(entry.graha)
                .unwrap()
                .longitude_deg;
            House::between(lagna, Rashi::of_longitude(at)).is_kendra()
        })
        .collect();
    let strongest = kendra
        .iter()
        .max_by(|a, b| strength(a).total_cmp(&strength(b)))
        .unwrap()
        .graha;
    let dasha = &document.dashas[0];
    let lords: Vec<(Graha, f64)> = dasha
        .periods
        .iter()
        .filter(|row| row.level() == 1)
        .map(|row| (row.lord, (row.interval.days() / 365.25).round()))
        .collect();
    let from = VIMSHOTTARI_LORDS
        .iter()
        .position(|(graha, _)| *graha == strongest)
        .unwrap();
    let expected: Vec<(Graha, f64)> = (0..9)
        .map(|at| VIMSHOTTARI_LORDS[(from + at) % 9])
        .collect();
    assert_eq!(lords, expected, "{measure} at {birth:?}");
    // Unasked, the shadbala is computed for the dasha alone.
    let bare = ask(false).unwrap();
    assert!(bare.shadbala.is_none());
    assert_eq!(bare.dashas[0].periods, dasha.periods, "{measure}");
    // Rebuilt, the antardashas are Vimshottari's proportions.
    let rebuilt = sdk.chart().dasha(&document, DashaSystem::Tara).unwrap();
    let first = rebuilt.mahadashas().next().unwrap();
    let stored: Vec<_> = dasha
        .periods
        .iter()
        .filter(|row| row.level() == 2)
        .take(9)
        .collect();
    for (ours, row) in rebuilt.children(&first).zip(&stored) {
        assert_eq!(ours.lord, row.lord, "{measure}");
        assert!(
            (ours.interval.to.get() - row.interval.to.get()).abs() < 1e-6,
            "{measure}"
        );
    }
    Some(kendra.len() > 1)
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
        .with_dashas([DashaSystem::Vimshottari, DashaSystem::Panchaswara]);
    let error = sdk
        .chart()
        .reading(document.foundation.instant, &request)
        .expect_err("Panchaswara is not built");
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

/// The systems whose sources stop above five levels, and where (C58), in
/// the order `systems()` lists them:
/// asked for more, each stops there and its reading says so.
const SHALLOW: [(DashaSystem, u8); 3] = [
    (DashaSystem::Kalachakra, 2),
    (DashaSystem::Profection, 1),
    (DashaSystem::Firdaria, 2),
];

/// Every system a chart always gives, asked for five levels: the depth a
/// reading carries is its deepest period's, five for every system but the
/// ones [`SHALLOW`] names, and never more than was asked
/// (`03-design/settings-and-profiles.md`, the validation table).
#[test]
fn a_reading_says_the_depth_it_carries_for_every_system() {
    let mut shallow = Vec::new();
    for system in teistro::dasha::systems_every_chart_gives() {
        let patch = format!(r#"{{"dasha": {{"depth": {{"{}": 5}}}}}}"#, system.key());
        let (_, document) = reading(&patch, &[system]);
        let dasha = &document.dashas[0];
        let deepest = dasha
            .periods
            .iter()
            .map(|period| period.path.split('/').count())
            .max()
            .unwrap();
        assert_eq!(usize::from(dasha.depth.get()), deepest, "{system:?}");
        if deepest < 5 {
            shallow.push((system, dasha.depth.get()));
        }
    }
    assert_eq!(shallow, SHALLOW, "the systems that stop above five levels");
}

/// The ayurdaya dasha through the façade (BPHS ch. 46 vv. 201 to 203,
/// ch. 43 vv. 30 to 32, ch. 51 vv. 13 to 16), read back against the
/// chart's own strengths and spans over a year of births: it begins from
/// the strongest of the lagna, the Sun and the Moon by the document's
/// shadbala and Bhava bala in rupas, runs that one's span (the lagna's
/// Amsayu, the Sun's Pindayu, the Moon's Nisargayu) giver by giver, runs
/// the rest by kendras, panapharas and apoklimas from it by strength, and
/// is the same under its two names. It is rebuilt from the document, and a
/// chosen span gives that span's years. Births begun from two or more of
/// the three are required, so the choice is tested at all.
#[test]
fn a_reading_carries_the_ayurdaya_dasha_from_the_strongest_of_the_three() {
    let sdk = Context::builder()
        .profile("conformance-baseline")
        .ephemeris([teistro::Ephemeris::Builtin])
        .build()
        .unwrap();
    let nisargayu = Context::builder()
        .profile("conformance-baseline")
        .settings_json(r#"{"dasha": {"ayurdaya_span": "NISARGAYU"}}"#)
        .ephemeris([teistro::Ephemeris::Builtin])
        .build()
        .unwrap();
    let mut firsts = std::collections::BTreeSet::new();
    for step in 0..12 {
        let birth = JulianDay::literal(2_447_995.49 + f64::from(step) * 31.3);
        firsts.insert(ayurdaya_at(&sdk, &nisargayu, birth));
    }
    assert!(firsts.len() >= 2, "begun only from {firsts:?}");
}

/// One birth's ayurdaya dasha checked as the test above says, naming which
/// of the three it began from.
fn ayurdaya_at(
    sdk: &Context,
    nisargayu: &Context,
    birth: JulianDay<teistro::quantity::Utc>,
) -> &'static str {
    use teistro::catalogue::Rashi;
    use teistro::quantity::{Altitude, Latitude, Longitude, Place};
    use teistro::rules::longevity::{AyurdayaRules, Giver, Span};
    use teistro::rules::{Body, Readings};
    use teistro::{House, RuleInputs};

    let place = Place::new(
        Latitude::try_new(27.7172).unwrap(),
        Longitude::try_new(85.324).unwrap(),
        Altitude::try_new(1400.0).unwrap(),
    );
    let offset = UtcOffset::try_from_seconds(20_700).unwrap();
    let request = ChartRequest::at(place, offset)
        .with_dashas([DashaSystem::Aayu, DashaSystem::Ashtakavarga])
        .with_state()
        .with_shadbala()
        .with_bhava_bala();
    let document = sdk.chart().reading(birth, &request).unwrap().value;
    let [aayu, ashtakavarga] = [DashaSystem::Aayu, DashaSystem::Ashtakavarga].map(|system| {
        document
            .dashas
            .iter()
            .find(|reading| reading.system == teistro::dasha::DashaName::Catalogued(system))
            .unwrap()
    });
    assert_eq!(aayu.periods, ashtakavarga.periods, "one dasha, two names");

    // The other road: the chart's strengths and spans, read directly.
    let inputs = RuleInputs::of(&document).unwrap();
    let evaluator = inputs.evaluator(Readings::default());
    let spans = evaluator.ayurdaya(AyurdayaRules::default());
    let strengths = evaluator.strengths();
    let lagna = document.bhava_bala.as_ref().unwrap().bhavas[0].virupas / 60.0;
    let strength = |giver: Giver| match giver {
        Giver::Graha(graha) => strengths.of[Body::Graha(graha).index()].unwrap(),
        Giver::Lagna => lagna,
    };
    let body = |giver: Giver| match giver {
        Giver::Graha(graha) => Body::Graha(graha),
        Giver::Lagna => Body::Lagna,
    };
    let sign = |giver: Giver| inputs.chart.placement(body(giver)).sign;
    let (first, span, name): (Giver, &Span, &str) = [
        (Giver::Lagna, &spans.amsayu, "lagna"),
        (Giver::Graha(Graha::Sun), &spans.pindayu, "Sun"),
        (Giver::Graha(Graha::Moon), &spans.nisargayu, "Moon"),
    ]
    .into_iter()
    .max_by(|a, b| strength(a.0).total_cmp(&strength(b.0)))
    .unwrap();
    // A row is a giver's: the lagna's carries its sign under its lord.
    let giver_of = |row: &teistro::dasha::PeriodRow| match row.sign {
        Some(_) => Giver::Lagna,
        None => Giver::Graha(row.lord),
    };
    let year_days = aayu.rules.year_length.days();
    let mahadashas: Vec<&teistro::dasha::PeriodRow> =
        aayu.periods.iter().filter(|row| row.level() == 1).collect();
    assert_eq!(giver_of(mahadashas[0]), first);
    for row in &mahadashas {
        let giver = giver_of(row);
        let years = span
            .contributions
            .iter()
            .find(|contribution| contribution.giver == giver)
            .unwrap()
            .net;
        assert!(
            (row.interval.days() / year_days - years).abs() < 1e-6,
            "{giver:?}"
        );
        if giver == Giver::Lagna {
            assert_eq!(row.sign, Some(sign(Giver::Lagna)));
            assert_eq!(row.lord, Rashi::attributes(sign(Giver::Lagna)).lord);
        }
    }
    let total: f64 = mahadashas.iter().map(|row| row.interval.days()).sum();
    assert!((total / year_days - span.years).abs() < 1e-6);
    // After the first, kendras, panapharas, apoklimas from it, each by
    // strength.
    let group = |giver: Giver| {
        let house = House::between(sign(first), sign(giver));
        if house.is_kendra() {
            0
        } else if house.is_panaphara() {
            1
        } else {
            2
        }
    };
    for pair in mahadashas[1..].windows(2) {
        let (a, b) = (giver_of(pair[0]), giver_of(pair[1]));
        assert!(
            group(a) < group(b) || (group(a) == group(b) && strength(a) >= strength(b)),
            "{a:?} before {b:?}"
        );
    }

    rebuilds_as_stored(sdk, &document, aayu);

    // A chosen span runs its own years from the same first.
    let (years, opening) = first_and_years(nisargayu, birth, &request);
    assert!((years - spans.nisargayu.years).abs() < 1e-6);
    assert_eq!(giver_of(&opening), first);
    name
}

/// A stored ayurdaya dasha rebuilt from its document gives its two stored
/// levels back and reads a level deeper.
fn rebuilds_as_stored(sdk: &Context, document: &Document, aayu: &teistro::dasha::DashaReading) {
    let mahadashas: Vec<&teistro::dasha::PeriodRow> =
        aayu.periods.iter().filter(|row| row.level() == 1).collect();
    let cursor = sdk.chart().dasha(document, DashaSystem::Aayu).unwrap();
    let rebuilt: Vec<teistro::dasha::Period> = cursor.mahadashas().collect();
    assert_eq!(rebuilt.len(), mahadashas.len());
    for (period, row) in rebuilt.iter().zip(&mahadashas) {
        assert_eq!((period.lord, period.sign), (row.lord, row.sign));
        assert!((period.interval.days() - row.interval.days()).abs() < 1e-6);
        let children: Vec<teistro::dasha::Period> = cursor.children(period).collect();
        let stored: Vec<&teistro::dasha::PeriodRow> = aayu
            .periods
            .iter()
            .filter(|child| child.level() == 2 && child.path.starts_with(&format!("{}/", row.path)))
            .collect();
        assert_eq!(children.len(), stored.len());
        for (child, kept) in children.iter().zip(stored) {
            assert_eq!((child.lord, child.sign), (kept.lord, kept.sign));
            assert!((child.interval.days() - kept.interval.days()).abs() < 1e-6);
        }
        assert!(cursor.children(&children[0]).next().is_some());
    }
}

/// The years an ayurdaya dasha's mahadashas run in all under `sdk`'s
/// settings, and its first mahadasha.
fn first_and_years(
    sdk: &Context,
    birth: JulianDay<teistro::quantity::Utc>,
    request: &ChartRequest,
) -> (f64, teistro::dasha::PeriodRow) {
    let document = sdk.chart().reading(birth, request).unwrap().value;
    let reading = &document.dashas[0];
    let year_days = reading.rules.year_length.days();
    let years = reading
        .periods
        .iter()
        .filter(|row| row.level() == 1)
        .map(|row| row.interval.days() / year_days)
        .sum();
    (years, reading.periods[0].clone())
}

/// The Naisargika dasha through the façade (BPHS ch. 43 vv. 16 and 17, ch.
/// 51 vv. 13 to 16, C743): the seven in their natural order for their
/// natural years of 360 days from birth. Each mahadasha's antardashas are
/// the verse's shares, worked here from the document's own signs and
/// shadbala: the lord whole, then of the bodies with it, in a trikona, in
/// the fourth or eighth and in the seventh, the strongest of each place.
/// Under `PROPORTIONAL` they are the natural years' proportions. Both
/// rebuild from the document.
#[test]
fn a_reading_carries_the_naisargika_dasha_in_the_natural_order() {
    use teistro::RuleInputs;
    use teistro::quantity::{Altitude, Latitude, Longitude, Place};
    use teistro::rules::Body;

    let natural = [
        (Graha::Moon, 1.0),
        (Graha::Mars, 2.0),
        (Graha::Mercury, 9.0),
        (Graha::Venus, 20.0),
        (Graha::Jupiter, 18.0),
        (Graha::Sun, 20.0),
        (Graha::Saturn, 50.0),
    ];
    let place = Place::new(
        Latitude::try_new(27.7172).unwrap(),
        Longitude::try_new(85.324).unwrap(),
        Altitude::try_new(1400.0).unwrap(),
    );
    let request = ChartRequest::at(place, UtcOffset::try_from_seconds(20_700).unwrap())
        .with_dashas([DashaSystem::Naisargika])
        .with_state()
        .with_shadbala();
    let context = |division: &str| {
        Context::builder()
            .profile("conformance-baseline")
            .settings_json(format!(
                r#"{{"dasha": {{"naisargika_division": "{division}"}}}}"#
            ))
            .ephemeris([teistro::Ephemeris::Builtin])
            .build()
            .unwrap()
    };
    let (shares, proportional) = (context("SHARES"), context("PROPORTIONAL"));
    let mut related = 0;
    for step in 0..6 {
        let birth = JulianDay::literal(2_447_995.49 + f64::from(step) * 47.1);
        let document = shares.chart().reading(birth, &request).unwrap().value;
        let reading = &document.dashas[0];
        let maha: Vec<&teistro::dasha::PeriodRow> = reading
            .periods
            .iter()
            .filter(|row| row.level() == 1)
            .collect();
        assert_eq!(maha.len(), 7);
        let mut from = birth.get();
        for (row, (lord, years)) in maha.iter().zip(natural) {
            assert_eq!(row.lord, lord);
            assert!((row.interval.from.get() - from).abs() < 1e-6);
            assert!((row.interval.days() - years * 360.0).abs() < 1e-6);
            from = row.interval.to.get();
        }
        // The other road: the verse's shares from the document's signs and
        // strengths.
        let inputs = RuleInputs::of(&document).unwrap();
        let sign = |graha: Graha| inputs.chart.placement(Body::Graha(graha)).sign;
        let rupas =
            |graha: Graha| inputs.chart.strengths.unwrap().of[Body::Graha(graha).index()].unwrap();
        for (at, row) in maha.iter().enumerate() {
            let lord = row.lord;
            let expected = verse_shares(&natural, at, &sign, &rupas);
            related += expected.len() - 1;
            let sum: f64 = expected.iter().map(|(_, share)| share).sum();
            let children: Vec<&teistro::dasha::PeriodRow> = reading
                .periods
                .iter()
                .filter(|child| {
                    child.level() == 2 && child.path.starts_with(&format!("{}/", row.path))
                })
                .collect();
            assert_eq!(
                children.iter().map(|child| child.lord).collect::<Vec<_>>(),
                expected.iter().map(|(lord, _)| *lord).collect::<Vec<_>>(),
                "{lord:?}"
            );
            for (child, (_, share)) in children.iter().zip(&expected) {
                assert!((child.interval.days() - row.interval.days() * share / sum).abs() < 1e-6);
            }
        }
        let cursor = shares
            .chart()
            .dasha(&document, DashaSystem::Naisargika)
            .unwrap();
        let venus = cursor.mahadashas().nth(3).unwrap();
        let deeper = cursor.children(&venus).next().unwrap();
        assert_eq!(deeper.lord, Graha::Venus);
        assert!(cursor.children(&deeper).next().is_some());

        let document = proportional.chart().reading(birth, &request).unwrap().value;
        let periods = &document.dashas[0].periods;
        let venus = periods.iter().find(|row| row.path == "3").unwrap();
        let children: Vec<&teistro::dasha::PeriodRow> = periods
            .iter()
            .filter(|child| child.level() == 2 && child.path.starts_with("3/"))
            .collect();
        assert_eq!(children.len(), 7);
        for (child, (lord, years)) in children.iter().zip(natural.iter().cycle().skip(3)) {
            assert_eq!(child.lord, *lord);
            assert!((child.interval.days() - venus.interval.days() * years / 120.0).abs() < 1e-6);
        }
    }
    // The charts give the shares something to choose among.
    assert!(related >= 12, "{related} related bodies over six charts");
}

/// Ch. 51 vv. 13 and 14's antardashas of the `at`th of `natural`'s lords,
/// worked from the verse: the lord whole, then the strongest by `rupas` of
/// those with it, in a trikona, in the fourth or eighth and in the seventh
/// by `sign`, in the order the lords run from it, each with its share.
fn verse_shares(
    natural: &[(Graha, f64); 7],
    at: usize,
    sign: &dyn Fn(Graha) -> teistro::catalogue::Rashi,
    rupas: &dyn Fn(Graha) -> f64,
) -> Vec<(Graha, f64)> {
    let lord = natural[at].0;
    let mut places: Vec<(u8, Graha, f64)> = Vec::new();
    for step in 1..7 {
        let other = natural[(at + step) % 7].0;
        let (place, share) = match teistro::House::between(sign(lord), sign(other)).get() {
            1 => (1, 0.5),
            5 | 9 => (5, 1.0 / 3.0),
            4 | 8 => (4, 0.25),
            7 => (7, 1.0 / 7.0),
            _ => continue,
        };
        match places.iter_mut().find(|held| held.0 == place) {
            Some(held) if rupas(other) > rupas(held.1) => *held = (place, other, share),
            Some(_) => {}
            None => places.push((place, other, share)),
        }
    }
    let order =
        |graha: Graha| (natural.iter().position(|(lord, _)| *lord == graha).unwrap() + 7 - at) % 7;
    places.sort_by_key(|held| order(held.1));
    std::iter::once((lord, 1.0))
        .chain(places.iter().map(|held| (held.1, held.2)))
        .collect()
}
