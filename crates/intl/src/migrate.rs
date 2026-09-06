//! `migrate baseline`: the one-time import of the baseline engine's entity
//! name tables into `sdk.entity` for the four launch languages
//! (`02-architecture/03-localization-architecture.md`, "The CLI"; ADR-0010).
//! The engine's exporter writes one JSON document with every entity type,
//! every entity in index order and its names in `sa`, `ne`, `en` and `hi`
//! (`fixtures/baseline/names.json`); this module maps the engine's types
//! and keys onto the SDK's catalogue, which stays the authority: a type
//! the catalogue has no kind for is reported and skipped, a key the
//! catalogue lacks is reported and skipped, and every record written
//! resolves. Existing records are kept unless overwriting is asked for.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value as Json};
use teistro_core::key::resolve;

use crate::source::{BASE_LOCALE, ENTITY_NAMESPACE, Entity, Entry, META_FILE, Tree};

/// The exporter's document.
#[derive(Clone, Debug, Deserialize)]
pub struct Dump {
    /// `teistro-conformance/baseline-names/1`.
    pub schema: String,
    /// The engine and its version.
    pub tool: String,
    /// The export date.
    #[serde(default)]
    pub exported: String,
    /// The languages the names come in.
    pub languages: Vec<String>,
    /// The entity types, each with its entities in index order.
    pub types: BTreeMap<String, Vec<Row>>,
}

/// One entity of the engine.
#[derive(Clone, Debug, Deserialize)]
pub struct Row {
    /// The engine's key.
    pub key: String,
    /// Its index inside the type.
    pub index: u32,
    /// Its symbol, when it has one.
    #[serde(default)]
    pub symbol: Option<String>,
    /// Its names by language.
    pub names: BTreeMap<String, Name>,
}

/// One language's name of an entity.
#[derive(Clone, Debug, Deserialize)]
pub struct Name {
    /// The display name.
    pub primary: String,
    /// The abbreviation for chart cells.
    #[serde(default)]
    pub abbreviation: Option<String>,
    /// The scholarly transliteration (IAST or ISO 15919).
    #[serde(default)]
    pub transliteration: Option<String>,
    /// Alternative names.
    #[serde(default)]
    pub synonyms: Vec<String>,
}

/// The engine's language codes and the SDK's locale tags.
pub const LOCALES: [(&str, &str); 4] = [
    ("en", BASE_LOCALE),
    ("ne", "ne-Deva-NP"),
    ("hi", "hi-Deva-IN"),
    ("sa", "sa-Deva"),
];

/// The engine's entity types that name a catalogue kind, and the kind.
pub const TYPE_KINDS: [(&str, &str); 20] = [
    ("GRAHA", "graha"),
    ("RASHI", "rashi"),
    ("NAKSHATRA", "nakshatra"),
    ("TATWA", "tatwa"),
    ("VARNA", "varna"),
    ("GANA", "gana"),
    ("NADI", "nadi"),
    ("YONI", "yoni"),
    ("TITHI", "tithi"),
    ("PANCHANGA_YOGA", "yoga"),
    ("KARANA", "karana"),
    ("VARA", "vara"),
    ("DIGNITY", "dignity"),
    ("AVASTHA_BALADI", "avastha_baladi"),
    ("SAMVATSAR", "samvatsara"),
    ("CHARA_KARAKA", "chara_karaka"),
    ("PAKSHA", "paksha"),
    ("AYANA", "ayana"),
    ("DEITY", "deity"),
    ("PANCHADHA_MAITRI", "relationship"),
];

/// The engine's keys whose spelling or kind the catalogue does not share:
/// the engine's type and key, and the catalogue's full key.
pub const KEY_ALIASES: [((&str, &str), &str); 18] = [
    (("GRAHA", "LAGNA"), "point.LAGNA"),
    (("PANCHANGA_YOGA", "VISHKAMBA"), "yoga.VISHKAMBHA"),
    (("CHARA_KARAKA", "AK"), "chara_karaka.ATMAKARAKA"),
    (("CHARA_KARAKA", "AmK"), "chara_karaka.AMATYAKARAKA"),
    (("CHARA_KARAKA", "BK"), "chara_karaka.BHRATRIKARAKA"),
    (("CHARA_KARAKA", "MK"), "chara_karaka.MATRIKARAKA"),
    (("CHARA_KARAKA", "PK"), "chara_karaka.PUTRAKARAKA"),
    (("CHARA_KARAKA", "GK"), "chara_karaka.GNATIKARAKA"),
    (("CHARA_KARAKA", "DK"), "chara_karaka.DARAKARAKA"),
    (("CHARA_KARAKA", "PiK"), "chara_karaka.PITRIKARAKA"),
    (("DEITY", "Ashwini Kumars"), "deity.ASHWINI_KUMARA"),
    (("DEITY", "Nagas"), "deity.NAGA"),
    (("DEITY", "Savitru"), "deity.SAVITR"),
    (("DEITY", "Vishwakarma"), "deity.VISHVAKARMA"),
    (("DEITY", "Indra-Agni"), "deity.INDRAGNI"),
    (("DEITY", "Vishwadevas"), "deity.VISHVADEVA"),
    (("DEITY", "Ashta Vasus"), "deity.VASU"),
    (("DEITY", "Ahir Budhnya"), "deity.AHIRBUDHNYA"),
];

/// One more spelling the catalogue does not share.
const PUSHA: ((&str, &str), &str) = (("DEITY", "Pusha"), "deity.PUSHAN");

/// The catalogue's full key for an engine entity: an alias, else the kind
/// and the key uppercased with every run of other characters as `_`.
#[must_use]
pub fn catalogue_key(entity_type: &str, key: &str) -> Option<String> {
    if let Some((_, full)) = KEY_ALIASES
        .iter()
        .chain(std::iter::once(&PUSHA))
        .find(|((t, k), _)| *t == entity_type && *k == key)
    {
        return Some((*full).to_string());
    }
    let (_, kind) = TYPE_KINDS.iter().find(|(t, _)| *t == entity_type)?;
    let mut normalised = String::with_capacity(key.len());
    let mut underscore = false;
    for c in key.chars() {
        if c.is_ascii_alphanumeric() {
            normalised.push(c.to_ascii_uppercase());
            underscore = false;
        } else if !underscore && !normalised.is_empty() {
            normalised.push('_');
            underscore = true;
        }
    }
    Some(format!("{kind}.{}", normalised.trim_end_matches('_')))
}

/// What the migration would write, and what it could not.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct MigrationReport {
    /// The engine's types mapped, with the records each gave.
    pub mapped: BTreeMap<String, usize>,
    /// The engine's types the catalogue has no kind for, with their sizes.
    pub unmapped: BTreeMap<String, usize>,
    /// Keys of mapped types the catalogue does not have, as `type: key`.
    pub unknown_keys: Vec<String>,
    /// Records written per locale.
    pub written: BTreeMap<String, usize>,
    /// Records kept as they were per locale (present, not overwritten).
    pub kept: BTreeMap<String, usize>,
}

impl MigrationReport {
    /// The report as Markdown.
    #[must_use]
    pub fn markdown(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(
            out,
            "mapped {} types ({} records), {} types without a catalogue kind, {} unknown keys\n",
            self.mapped.len(),
            self.mapped.values().sum::<usize>(),
            self.unmapped.len(),
            self.unknown_keys.len()
        );
        out.push_str("| locale | written | kept |\n|---|---:|---:|\n");
        for (locale, written) in &self.written {
            let _ = writeln!(
                out,
                "| {locale} | {written} | {} |",
                self.kept.get(locale).copied().unwrap_or(0)
            );
        }
        if !self.unmapped.is_empty() {
            out.push_str("\n| type without a kind | entities |\n|---|---:|\n");
            for (kind, count) in &self.unmapped {
                let _ = writeln!(out, "| {kind} | {count} |");
            }
        }
        if !self.unknown_keys.is_empty() {
            out.push_str("\n| unknown key |\n|---|\n");
            for key in &self.unknown_keys {
                let _ = writeln!(out, "| {key} |");
            }
        }
        out
    }
}

/// The migration planned: records per locale in the engine's order, and
/// the report.
#[derive(Clone, Debug, Default)]
pub struct Migration {
    /// Records per SDK locale, each as the catalogue key (`graha.SUN`) and
    /// the record, in the engine's order (type by type, index by index),
    /// which the sources keep and the generators follow.
    pub records: BTreeMap<String, Vec<(String, Entity)>>,
    /// What was mapped and what was not.
    pub report: MigrationReport,
}

/// The record at a catalogue key, when the migration has one.
#[must_use]
pub fn record<'r>(records: &'r [(String, Entity)], key: &str) -> Option<&'r Entity> {
    records
        .iter()
        .find(|(full, _)| full == key)
        .map(|(_, entity)| entity)
}

/// Adds a record, replacing one already planned under the same key.
fn push_record(records: &mut Vec<(String, Entity)>, full: String, entity: Entity) {
    if let Some(slot) = records.iter_mut().find(|(k, _)| *k == full) {
        slot.1 = entity;
    } else {
        records.push((full, entity));
    }
}

/// Plans the migration of a dump: every mapped type's entities become
/// records for every language the dump carries. The forms are `name` (the
/// primary name), `prose` (the same), `short` (the abbreviation when the
/// engine has one) and `iast` (the language's transliteration, else the
/// Sanskrit one, since a transliteration names the Sanskrit word whatever
/// the script); the glyph is the engine's symbol or the catalogue's
/// skeleton's; the gender the skeleton's.
#[must_use]
pub fn plan(dump: &Dump, skeleton: Option<&Json>) -> Migration {
    let mut migration = Migration::default();
    let mut unknown: BTreeSet<String> = BTreeSet::new();
    for (entity_type, rows) in &dump.types {
        if !TYPE_KINDS.iter().any(|(t, _)| t == entity_type) {
            migration
                .report
                .unmapped
                .insert(entity_type.clone(), rows.len());
            continue;
        }
        let mut mapped = 0;
        for row in rows {
            let Some(full) = catalogue_key(entity_type, &row.key) else {
                continue;
            };
            if resolve(&full).is_err() {
                unknown.insert(format!("{entity_type}: {}", row.key));
                continue;
            }
            mapped += 1;
            let (kind, key) = full.split_once('.').unwrap_or((&full, ""));
            let from_skeleton = skeleton.and_then(|s| s.get(kind)).and_then(|k| k.get(key));
            let glyph = row.symbol.clone().or_else(|| {
                from_skeleton
                    .and_then(|s| s.get("glyph"))
                    .and_then(Json::as_str)
                    .map(str::to_string)
            });
            let gender = from_skeleton
                .and_then(|s| s.get("gender"))
                .and_then(Json::as_str)
                .map(str::to_string);
            let sanskrit_iast = row.names.get("sa").and_then(|n| n.transliteration.clone());
            for (code, tag) in LOCALES {
                let Some(name) = row.names.get(code) else {
                    continue;
                };
                let mut forms = BTreeMap::new();
                forms.insert(String::from("name"), name.primary.clone());
                forms.insert(String::from("prose"), name.primary.clone());
                if let Some(short) = &name.abbreviation {
                    forms.insert(String::from("short"), short.clone());
                }
                if let Some(iast) = name
                    .transliteration
                    .clone()
                    .or_else(|| sanskrit_iast.clone())
                {
                    forms.insert(String::from("iast"), iast);
                }
                push_record(
                    migration.records.entry(tag.to_string()).or_default(),
                    full.clone(),
                    Entity {
                        forms,
                        gender: gender.clone(),
                        glyph: glyph.clone(),
                    },
                );
            }
        }
        migration.report.mapped.insert(entity_type.clone(), mapped);
    }
    migration.report.unknown_keys = unknown.into_iter().collect();
    migration
}

/// The metadata of a locale the migration creates: Devanagari digits with
/// the Indian grouping, a fallback to the base, `base` completeness until
/// its messages are translated, and the list patterns of the language.
fn new_locale_meta(tag: &str) -> Json {
    let (and, or) = match tag {
        "hi-Deva-IN" => ("{0} और {1}", "{0} या {1}"),
        "sa-Deva" => ("{0} तथा {1}", "{0} वा {1}"),
        _ => ("{0} and {1}", "{0} or {1}"),
    };
    serde_json::json!({
        "locale": tag,
        "direction": "ltr",
        "numberingSystem": "deva",
        "grouping": [3, 2],
        "decimal": ".",
        "group": ",",
        "fallback": [BASE_LOCALE],
        "completeness": "base",
        "contexts": { "gender": ["m", "f", "n"] },
        "termStyle": "vernacular",
        "listPatterns": {
            "and": { "pair": and, "middle": "{0}, {1}", "end": and },
            "or": { "pair": or, "middle": "{0}, {1}", "end": or }
        }
    })
}

fn entity_json(entity: &Entity) -> Json {
    let mut object = Map::new();
    for form in ["short", "name", "prose", "iast"] {
        if let Some(value) = entity.forms.get(form) {
            object.insert(form.to_string(), Json::String(value.clone()));
        }
    }
    for (form, value) in &entity.forms {
        if !object.contains_key(form) {
            object.insert(form.clone(), Json::String(value.clone()));
        }
    }
    if let Some(glyph) = &entity.glyph {
        object.insert(String::from("glyph"), Json::String(glyph.clone()));
    }
    if let Some(gender) = &entity.gender {
        object.insert(String::from("gender"), Json::String(gender.clone()));
    }
    Json::Object(object)
}

/// Applies a planned migration to an `i18n/` root: every locale's
/// `sdk.entity.json` gains the records (an existing record kept unless
/// `overwrite`), a locale the root lacks is created with its metadata.
/// Returns the files written; the report's `written` and `kept` are
/// filled.
///
/// # Errors
///
/// A file that cannot be read, parsed or written.
pub fn apply(
    migration: &mut Migration,
    root: &Path,
    overwrite: bool,
) -> std::io::Result<Vec<PathBuf>> {
    let mut written_files = Vec::new();
    for (tag, records) in &migration.records {
        let dir = root.join(tag);
        std::fs::create_dir_all(&dir)?;
        let meta_path = dir.join(META_FILE);
        if !meta_path.exists() {
            std::fs::write(
                &meta_path,
                format!("{}\n", serde_json::to_string_pretty(&new_locale_meta(tag))?),
            )?;
            written_files.push(meta_path);
        }
        let path = dir.join(format!("{ENTITY_NAMESPACE}.json"));
        let mut document: Map<String, Json> = if path.exists() {
            serde_json::from_str(&std::fs::read_to_string(&path)?)?
        } else {
            Map::new()
        };
        let mut written = 0;
        let mut kept = 0;
        for (full, entity) in records {
            let Some((kind, key)) = full.split_once('.') else {
                continue;
            };
            let group = document
                .entry(kind.to_string())
                .or_insert_with(|| Json::Object(Map::new()));
            let Json::Object(group) = group else {
                continue;
            };
            if group.contains_key(key) && !overwrite {
                kept += 1;
                continue;
            }
            group.insert(key.to_string(), entity_json(entity));
            written += 1;
        }
        std::fs::write(
            &path,
            format!(
                "{}\n",
                serde_json::to_string_pretty(&Json::Object(document))?
            ),
        )?;
        written_files.push(path);
        migration.report.written.insert(tag.clone(), written);
        migration.report.kept.insert(tag.clone(), kept);
    }
    Ok(written_files)
}

/// Reads a dump.
///
/// # Errors
///
/// A file that cannot be read or is not the exporter's document.
pub fn read_dump(path: &Path) -> Result<Dump, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let dump: Dump = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    if dump.schema != "teistro-conformance/baseline-names/1" {
        return Err(format!(
            "{}: schema `{}` is not the names exporter's",
            path.display(),
            dump.schema
        ));
    }
    Ok(dump)
}

/// The catalogue's entity skeleton (`catalogue/entity-skeleton.json`),
/// when the file is there: glyphs and genders per kind and key.
#[must_use]
pub fn read_skeleton(path: &Path) -> Option<Json> {
    serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()
}

/// Loads the tree after a migration, to validate what was written.
///
/// # Errors
///
/// As [`Tree::load`].
pub fn reload(root: &Path) -> Result<Tree, crate::source::SourceError> {
    Tree::load(root)
}

/// A migration's own consistency check: every written key resolves in the
/// catalogue, and the entry it came from is an entity.
#[must_use]
pub fn every_record_resolves(migration: &Migration) -> bool {
    migration
        .records
        .values()
        .all(|records| records.iter().all(|(full, _)| resolve(full).is_ok()))
}

/// The `sdk.entity` entries of a locale as the migration would see them.
#[must_use]
pub fn entity_entries(tree: &Tree, tag: &str) -> usize {
    tree.locales
        .get(tag)
        .and_then(|l| l.namespaces.get(ENTITY_NAMESPACE))
        .map_or(0, |ns| {
            ns.entries
                .values()
                .filter(|e| matches!(e, Entry::Entity(_)))
                .count()
        })
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use super::*;
    use crate::source::sdk_root;
    use crate::validate::validate;

    fn dump() -> Dump {
        let path = sdk_root().join("../fixtures/baseline/names.json");
        read_dump(&path).unwrap_or_else(|e| panic!("{e}"))
    }

    #[test]
    fn the_engines_keys_map_onto_the_catalogue() {
        assert_eq!(catalogue_key("GRAHA", "SUN").as_deref(), Some("graha.SUN"));
        assert_eq!(
            catalogue_key("GRAHA", "LAGNA").as_deref(),
            Some("point.LAGNA")
        );
        assert_eq!(
            catalogue_key("PANCHANGA_YOGA", "VISHKAMBA").as_deref(),
            Some("yoga.VISHKAMBHA")
        );
        assert_eq!(
            catalogue_key("DEITY", "Ashwini Kumars").as_deref(),
            Some("deity.ASHWINI_KUMARA")
        );
        assert_eq!(
            catalogue_key("DEITY", "Aja Ekapada").as_deref(),
            Some("deity.AJA_EKAPADA")
        );
        assert_eq!(
            catalogue_key("PAKSHA", "shukla").as_deref(),
            Some("paksha.SHUKLA")
        );
        assert_eq!(
            catalogue_key("CHARA_KARAKA", "AmK").as_deref(),
            Some("chara_karaka.AMATYAKARAKA")
        );
        assert_eq!(catalogue_key("VASHYA", "manav"), None);
    }

    #[test]
    fn the_dump_plans_into_resolving_records_for_four_locales() {
        let dump = dump();
        let skeleton = read_skeleton(&sdk_root().join("../catalogue/entity-skeleton.json"));
        assert!(skeleton.is_some());
        let migration = plan(&dump, skeleton.as_ref());
        assert!(every_record_resolves(&migration));
        assert!(
            migration.report.unknown_keys.is_empty(),
            "{:?}",
            migration.report.unknown_keys
        );
        assert_eq!(migration.records.len(), 4);
        for (_, tag) in LOCALES {
            let records = &migration.records[tag];
            // Twenty mapped types, 274 entities each in every language.
            assert!(records.len() >= 270, "{tag}: {}", records.len());
            let sun = record(records, "graha.SUN").unwrap();
            assert!(!sun.name().is_empty());
            assert_eq!(sun.gender.as_deref(), Some("m"));
            assert!(sun.glyph.is_some());
            for key in [
                "point.LAGNA",
                "tithi.AMAVASYA",
                "samvatsara.PRABHAVA",
                "deity.PUSHAN",
            ] {
                assert!(record(records, key).is_some(), "{tag}: {key}");
            }
            // The engine's order is kept: the first tithi before the last.
            let position = |key: &str| records.iter().position(|(k, _)| k == key).unwrap();
            assert!(position("tithi.SHUKLA_PRATIPADA") < position("tithi.AMAVASYA"));
            assert!(position("graha.SUN") < position("graha.MOON"));
        }
        let nepali = &migration.records["ne-Deva-NP"];
        assert_eq!(record(nepali, "graha.SUN").unwrap().name(), "सूर्य");
        // A Nepali record without its own transliteration carries the Sanskrit one.
        assert!(
            record(nepali, "samvatsara.PRABHAVA")
                .unwrap()
                .form("iast")
                .is_some()
        );
        assert!(migration.report.mapped.len() >= 19);
        assert!(migration.report.unmapped.contains_key("AVASTHA_DEEPTADI"));
        assert!(migration.report.unmapped.contains_key("VASHYA"));
        assert!(migration.report.markdown().contains("| AVASTHA_DEEPTADI |"));
    }

    #[test]
    fn applying_into_a_fresh_root_gives_a_tree_that_validates() {
        let dump = dump();
        let mut migration = plan(&dump, None);
        let root =
            std::env::temp_dir().join(format!("teistro-intl-migrate-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        // The base locale first, from the SDK's own metadata, so the tree has a base.
        std::fs::create_dir_all(root.join(BASE_LOCALE)).unwrap();
        std::fs::copy(
            sdk_root().join(BASE_LOCALE).join(META_FILE),
            root.join(BASE_LOCALE).join(META_FILE),
        )
        .unwrap();
        let files = apply(&mut migration, &root, false).unwrap();
        assert!(files.iter().any(|f| f.ends_with("hi-Deva-IN/_meta.json")));
        assert!(files.iter().any(|f| f.ends_with("sa-Deva/sdk.entity.json")));
        let tree = reload(&root).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(tree.locales.len(), 4);
        assert_eq!(
            entity_entries(&tree, BASE_LOCALE),
            migration.records[BASE_LOCALE].len()
        );
        let report = validate(&tree);
        assert!(report.passed(), "{}", report.markdown());
        // A second run keeps every record and writes none.
        let mut again = plan(&dump, None);
        apply(&mut again, &root, false).unwrap();
        assert_eq!(again.report.written.values().sum::<usize>(), 0);
        assert_eq!(
            again.report.kept[BASE_LOCALE],
            migration.records[BASE_LOCALE].len()
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}
