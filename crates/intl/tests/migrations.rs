//! The one-time imports from the baseline engine's exporters, fed what an
//! exporter could get wrong: a document of another schema, a key the
//! catalogue lacks, a language the SDK does not ship, a reading with
//! nothing to say. Each is reported and skipped, and what is written
//! is what the dump held.

#![allow(clippy::unwrap_used, reason = "a test fails by panicking")]

use std::path::{Path, PathBuf};

use teistro_intl::migrate::{
    self, Dump, MigrationReport, ReadingsDump, STATE_SCHEMA, StatesDump, catalogue_key,
};

/// A fresh directory of the test's own.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("intl-migrations")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn pair(form: &str, text: &str) -> (String, String) {
    (form.to_string(), text.to_string())
}

fn forms(migration: &migrate::Migration, tag: &str, key: &str) -> Vec<(String, String)> {
    migration
        .records
        .get(tag)
        .and_then(|records| migrate::record(records, key))
        .map(|entity| entity.forms.clone().into_iter().collect())
        .unwrap_or_default()
}

#[test]
fn a_type_key_is_normalised_to_the_catalogue_spelling() {
    assert_eq!(
        catalogue_key("GRAHA", " -sun moon- ").as_deref(),
        Some("graha.SUN_MOON")
    );
    assert_eq!(catalogue_key("COLOUR", "RED"), None);
}

#[test]
fn a_names_dump_maps_what_the_catalogue_knows_and_reports_the_rest() {
    let dump: Dump = serde_json::from_value(serde_json::json!({
        "schema": "teistro-conformance/baseline-names/1",
        "tool": "the baseline engine",
        "languages": ["en", "ne", "xx"],
        "types": {
            "GRAHA": [
                {"key": "SUN", "index": 0, "names": {
                    "en": {"primary": "Sun"},
                    "sa": {"primary": "सूर्यः", "transliteration": "sūryaḥ"},
                    "xx": {"primary": "?"}}},
                {"key": "SUN", "index": 0, "names": {"en": {"primary": "Sun", "abbreviation": "Su"}}},
                {"key": "VULCAN", "index": 10, "names": {"en": {"primary": "Vulcan"}}}
            ],
            "COLOUR": [{"key": "RED", "index": 0, "names": {"en": {"primary": "Red"}}}]
        }
    }))
    .unwrap();
    let skeleton = serde_json::json!({"graha": {"SUN": {"glyph": "☉", "gender": "m"}}});
    let mut migration = migrate::plan(&dump, Some(&skeleton));
    // The row given twice is one record, the second laid over the first,
    // and the Sanskrit's transliteration names it in every language.
    let sun = forms(&migration, "en-Latn", "graha.SUN");
    assert!(sun.contains(&pair("short", "Su")), "{sun:?}");
    assert!(sun.contains(&pair("iast", "sūryaḥ")), "{sun:?}");
    let record = migration
        .records
        .get("en-Latn")
        .and_then(|records| migrate::record(records, "graha.SUN"))
        .unwrap();
    assert_eq!(
        (record.glyph.as_deref(), record.gender.as_deref()),
        (Some("☉"), Some("m")),
        "the skeleton's glyph and gender"
    );
    assert!(!migration.records.keys().any(|tag| tag.starts_with("xx")));
    assert_eq!(migration.report.unmapped.get("COLOUR"), Some(&1));
    assert_eq!(migration.report.unknown_keys, ["GRAHA: VULCAN"]);
    assert!(migrate::every_record_resolves(&migration));

    // Written into an empty root, the base locale's meta is strict and the
    // others fall back to it; written again, every record is kept.
    let root = scratch("names");
    let written = migrate::apply(&mut migration, &root, false).unwrap();
    assert!(
        written
            .iter()
            .any(|path| path.ends_with("sa-Deva/_meta.json"))
    );
    let meta = |tag: &str| -> serde_json::Value {
        serde_json::from_str(&std::fs::read_to_string(root.join(tag).join("_meta.json")).unwrap())
            .unwrap()
    };
    let at = |tag: &str, pointer: &str| meta(tag).pointer(pointer).cloned();
    assert_eq!(at("en-Latn", "/completeness"), Some("strict".into()));
    assert_eq!(at("sa-Deva", "/completeness"), Some("base".into()));
    assert_eq!(at("sa-Deva", "/numberingSystem"), Some("deva".into()));
    assert_eq!(
        at("sa-Deva", "/listPatterns/and/pair"),
        Some("{0} तथा {1}".into())
    );
    assert_eq!(migration.report.written.get("en-Latn"), Some(&1));
    migrate::apply(&mut migration, &root, false).unwrap();
    assert_eq!(migration.report.kept.get("en-Latn"), Some(&1));
    assert_eq!(migration.report.written.get("en-Latn"), Some(&0));
    migrate::apply(&mut migration, &root, true).unwrap();
    assert_eq!(
        migration.report.written.get("en-Latn"),
        Some(&1),
        "overwriting"
    );

    // A group that is not an object is left alone rather than replaced.
    let odd = scratch("odd-group");
    std::fs::create_dir_all(odd.join("en-Latn")).unwrap();
    std::fs::write(
        odd.join("en-Latn").join("sdk.entity.json"),
        r#"{"graha": "x"}"#,
    )
    .unwrap();
    migrate::apply(&mut migration, &odd, true).unwrap();
    assert_eq!(migration.report.written.get("en-Latn"), Some(&0));
}

#[test]
fn a_report_prints_every_table_it_has_rows_for() {
    let mut report = MigrationReport::default();
    assert!(!report.markdown().contains("unknown key |"));
    report.unmapped.insert("COLOUR".into(), 2);
    report.unknown_keys.push("GRAHA: VULCAN".into());
    report
        .refused
        .push("inauspicious-kaal: varjyam: no key".into());
    report.written.insert("en-Latn".into(), 3);
    let text = report.markdown();
    assert!(
        text.contains("refused inauspicious-kaal: varjyam: no key\n"),
        "{text}"
    );
    assert!(text.contains("| en-Latn | 3 | 0 |\n"), "{text}");
    assert!(text.contains("| COLOUR | 2 |\n"), "{text}");
    assert!(text.contains("| GRAHA: VULCAN |\n"), "{text}");
}

#[test]
fn a_readings_dump_writes_each_rule_by_its_summary() {
    let dump: ReadingsDump = serde_json::from_value(serde_json::json!({
        "schema": "teistro-conformance/baseline-readings/1",
        "tool": "the baseline engine",
        "languages": ["en", "ne"],
        "readings": {
            "GAJAKESARI": {"group": "yoga",
                "en": {"summary": "Fame", "full": "Jupiter in a kendra from the Moon.",
                       "effects": {"wealth": "Riches", "health": "  "}},
                "ne": {"summary": " "},
                "xx": {"summary": "?"}},
            "SILENT": {"group": "yoga", "en": {"full": "No summary."}},
            "not-a-rule": {"group": "yoga", "en": {"summary": "x"}}
        }
    }))
    .unwrap();
    let migration = migrate::plan_readings(&dump);
    let fame = forms(&migration, "en-Latn", "rule.GAJAKESARI");
    assert_eq!(
        fame,
        [
            pair("name", "Fame"),
            pair("prose", "Jupiter in a kendra from the Moon."),
            pair("wealth", "Riches"),
        ]
    );
    assert_eq!(forms(&migration, "ne-Deva-NP", "rule.GAJAKESARI"), []);
    assert_eq!(
        migration.report.mapped.get("yoga"),
        Some(&1),
        "SILENT wrote nothing"
    );
    for reported in [
        "not-a-rule: not a rule key",
        "GAJAKESARI (ne): no summary to name the record by",
        "SILENT (en): no summary to name the record by",
    ] {
        assert!(
            migration
                .report
                .unknown_keys
                .iter()
                .any(|key| key == reported),
            "{reported}: {:?}",
            migration.report.unknown_keys
        );
    }
}

#[test]
fn a_states_dump_writes_each_subject_under_its_category_s_form() {
    let dump: StatesDump = serde_json::from_value(serde_json::json!({
        "schema": STATE_SCHEMA,
        "tool": "the baseline engine",
        "languages": ["en"],
        "categories": {
            "graha-color": {
                "SUN": {"en": {"summary": "Copper", "full": "The Sun's red.",
                               "effects": {"tone": "warm", "blank": ""}},
                        "xx": {"summary": "?"}},
                "MOON": {"en": {"summary": " ", "full": ""}}
            },
            "graha-bhava": {
                "SUN_IN_1": {"en": {"summary": "Bright"}},
                "SUN_IN_13": {"en": {"summary": "Nowhere"}},
                "PLUTO_IN_1": {"en": {"summary": "Far"}},
                "SUN": {"en": {"summary": "No house"}},
                "SUN_IN_X": {"en": {"summary": "No number"}}
            },
            "inauspicious-kaal": {
                "rahu-kaal": {"en": {"summary": "Rahu's hour"}},
                "varjyam": {"en": {"summary": "Avoided"}}
            },
            "weather": {"RAIN": {"en": {"summary": "Wet"}}}
        }
    }))
    .unwrap();
    let migration = migrate::plan_states(&dump);
    assert!(migration.overlay, "a state reading adds forms to a record");
    assert_eq!(
        forms(&migration, "en-Latn", "graha.SUN"),
        [
            pair("colour", "Copper"),
            pair("colourProse", "The Sun's red."),
            pair("colourTone", "warm"),
        ]
    );
    assert_eq!(
        forms(&migration, "en-Latn", "graha_bhava.SUN_IN_1"),
        [pair("name", "Bright")],
        "a category without a form names the record"
    );
    assert!(
        !forms(&migration, "en-Latn", "kaala.RAHU_KAALA").is_empty(),
        "an alias"
    );
    assert_eq!(migration.report.unmapped.get("weather"), Some(&1));
    assert_eq!(migration.report.refused.len(), 1);
    assert!(
        migration
            .report
            .refused
            .first()
            .is_some_and(|refusal| refusal.starts_with("inauspicious-kaal: varjyam: "))
    );
    for reported in [
        "graha-color: MOON (en): nothing to say",
        "graha-bhava: SUN_IN_13: house 13 is not one of twelve",
        "graha-bhava: PLUTO_IN_1: `PLUTO` is no graha of the engine's nine",
        "graha-bhava: SUN: `SUN` is not a graha and a house",
        "graha-bhava: SUN_IN_X: `X` is not a house number",
    ] {
        assert!(
            migration
                .report
                .unknown_keys
                .iter()
                .any(|key| key == reported),
            "{reported}: {:?}",
            migration.report.unknown_keys
        );
    }
}

#[test]
fn each_reader_refuses_a_missing_malformed_or_foreign_document() {
    let dir = scratch("readers");
    let missing = dir.join("missing.json");
    let malformed = dir.join("malformed.json");
    std::fs::write(&malformed, "{").unwrap();
    let foreign = dir.join("foreign.json");
    std::fs::write(
        &foreign,
        r#"{"schema": "other/1", "tool": "t", "languages": [], "types": {}, "readings": {}, "categories": {}}"#,
    )
    .unwrap();
    for path in [&missing, &malformed] {
        assert!(migrate::read_dump(path).is_err());
        assert!(migrate::read_readings_dump(path).is_err());
        assert!(migrate::read_states_dump(path).is_err());
    }
    assert!(
        migrate::read_dump(&foreign)
            .unwrap_err()
            .ends_with("is not the names exporter's")
    );
    assert!(
        migrate::read_readings_dump(&foreign)
            .unwrap_err()
            .ends_with("is not the readings exporter's")
    );
    assert!(
        migrate::read_states_dump(&foreign)
            .unwrap_err()
            .ends_with("is not the state readings exporter's")
    );
    assert!(migrate::read_skeleton(&missing).is_none());
    assert!(migrate::read_skeleton(&malformed).is_none());

    let readings = dir.join("readings.json");
    std::fs::write(
        &readings,
        r#"{"schema": "teistro-conformance/baseline-readings/1", "tool": "t", "languages": ["en"], "readings": {}}"#,
    )
    .unwrap();
    assert_eq!(
        migrate::read_readings_dump(&readings)
            .unwrap()
            .readings
            .len(),
        0
    );
    let states = dir.join("states.json");
    std::fs::write(
        &states,
        format!(r#"{{"schema": "{STATE_SCHEMA}", "tool": "t", "languages": ["en"], "categories": {{}}}}"#),
    )
    .unwrap();
    assert_eq!(
        migrate::read_states_dump(&states).unwrap().categories.len(),
        0
    );
}
