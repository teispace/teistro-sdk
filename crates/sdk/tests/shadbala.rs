//! A chart document's Shadbala, computed end to end: the corpus's first chart
//! founded with the built-in ephemeris, read under the conformance profile's
//! reading (the engine's) against every component the corpus recorded, and
//! under the default profile's (BPHS ch. 27's)
//! (`docs/03-design/shadbala-measured.md`).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index JSON by key"
)]

mod common;

use common::{fixture, reading};
use teistro::strength::ShadbalaRules;
use teistro::strength::shadbala::VIRUPAS_PER_RUPA;

/// How far the built-in ephemeris may move a continuous strength from the
/// recorded one: its positions stand within arcseconds of the recording
/// engine's, and a strength is at most a third of a degree's arc.
const WITHIN: f64 = 0.01;

#[test]
fn a_reading_carries_the_shadbala_the_corpus_recorded() {
    let recorded = fixture("shadbala/charts/c001-kathmandu-1990-04-14.json");
    let (_, document) = reading("{}", teistro::ChartRequest::with_shadbala);
    assert!(document.sections().contains(&"shadbala"));
    let shadbala = document.shadbala.expect("the section asked for");
    assert_eq!(
        shadbala.rules,
        ShadbalaRules::RECORDING_ENGINE,
        "the conformance profile takes the engine's reading"
    );
    for graha in &shadbala.grahas {
        let at = &recorded["shadbala"][graha.graha.key()];
        let (sthana, kaala) = (&graha.sthana, &graha.kaala);
        for (path, ours) in [
            ("sthana.uccha", sthana.uchcha),
            ("sthana.saptavargiya", sthana.saptavargaja),
            ("sthana.ojayugma", sthana.ojayugma),
            ("sthana.kendra", sthana.kendradi),
            ("sthana.drekkana", sthana.drekkana),
            ("dig", graha.dig),
            ("kaala.nathonnatha", kaala.nathonnatha),
            ("kaala.paksha", kaala.paksha),
            ("kaala.tribhaga", kaala.tribhaga),
            ("kaala.abda", kaala.abda),
            ("kaala.masa", kaala.masa),
            ("kaala.vara", kaala.vara),
            ("kaala.hora", kaala.hora),
            ("kaala.ayana", kaala.ayana),
            ("cheshta", graha.cheshta),
            ("naisargika", graha.naisargika),
            ("drik", graha.drik),
            ("total_shashtiamshas", graha.virupas),
        ] {
            let expected = path.split('.').fold(at, |v, key| &v[key]).as_f64().unwrap();
            assert!(
                (ours - expected).abs() < WITHIN,
                "{:?} {path}: {ours} against {expected}",
                graha.graha
            );
        }
        assert_eq!(graha.strong, at["is_sufficient"].as_bool().unwrap());
    }
}

#[test]
fn the_text_s_reading_counts_the_chapter_s_strengths() {
    let (_, document) = reading(
        r#"{"strength": {"saptavargaja": "COMPOUND", "drekkana": "MALE_FEMALE_NEUTER", "nathonnatha": "MIDNIGHT", "pre_dawn_night": "PREVIOUS_EVENING", "benefics": "CONDITIONAL", "sun_ayana": "DOUBLED", "luminary_cheshta": "AYANA_AND_PAKSHA", "kranti": "TRUE", "kaala_lords": "AHARGANA", "dig": "ANGLES", "yuddha": "SRIPATI", "cheshta": "SRIPATI", "drik": "QUARTER_WITH_JUPITER_MERCURY", "naisargika": "EXACT", "required_rupas": "BPHS"}}"#,
        teistro::ChartRequest::with_shadbala,
    );
    let shadbala = document.shadbala.expect("the section asked for");
    assert_eq!(shadbala.rules, ShadbalaRules::BPHS);
    let natural: f64 = shadbala.grahas.iter().map(|g| g.naisargika).sum();
    assert!((natural - 4.0 * VIRUPAS_PER_RUPA).abs() < 1e-9);
    let (sun, moon) = (shadbala.grahas[0], shadbala.grahas[1]);
    assert!((sun.kaala.ayana - 2.0 * sun.cheshta).abs() < 1e-12);
    assert!((moon.cheshta - moon.kaala.paksha).abs() < 1e-12);
    // One of the lords of the year, the month, the day and the hour each.
    let lords = |f: fn(&teistro::strength::GrahaShadbala) -> f64, value: f64| {
        shadbala
            .grahas
            .iter()
            .filter(|g| (f(g) - value).abs() < 1e-12)
            .count()
    };
    assert_eq!(lords(|g| g.kaala.abda, 15.0), 1);
    assert_eq!(lords(|g| g.kaala.masa, 30.0), 1);
    assert_eq!(lords(|g| g.kaala.vara, 45.0), 1);
    assert_eq!(lords(|g| g.kaala.hora, 60.0), 1);
    for graha in &shadbala.grahas {
        assert!(graha.virupas.is_finite());
        assert_eq!(graha.strong, graha.rupas >= graha.required_rupas);
    }
}

#[test]
fn an_unbuilt_scheme_is_refused_by_name() {
    let sdk = teistro::Context::builder()
        .profile("conformance-baseline")
        .settings_json(r#"{"strength": {"bala_scheme": "PARASHARA_EXTENDED"}}"#)
        .ephemeris([teistro::Ephemeris::Builtin])
        .build()
        .expect("the settings are coherent");
    let place = teistro::quantity::Place::new(
        teistro::quantity::Latitude::try_new(27.7172).unwrap(),
        teistro::quantity::Longitude::try_new(85.324).unwrap(),
        teistro::quantity::Altitude::try_new(1400.0).unwrap(),
    );
    let request =
        teistro::ChartRequest::at(place, teistro::UtcOffset::try_from_seconds(20_700).unwrap())
            .with_shadbala();
    let refused = sdk
        .chart()
        .reading(
            teistro::quantity::JulianDay::<teistro::quantity::Utc>::literal(
                2_447_995.489_583_333_5,
            ),
            &request,
        )
        .unwrap_err();
    assert_eq!(refused.field(), Some("strength.bala_scheme"));
}
