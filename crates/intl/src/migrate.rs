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
    /// Whether these records **add forms** to records another root names,
    /// rather than naming their subjects themselves.
    ///
    /// A locale created for an overlay root declares `base` completeness,
    /// including the base locale: nothing here is complete, and a record
    /// here has no name of its own on purpose
    /// (`03-design/state-readings.md` §3).
    pub overlay: bool,
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
/// Adds a record, laying it over one already planned for the same key
/// rather than replacing it ([`Entity::overlaid`]).
///
/// Two of the engine's own corpora describe the same subject under
/// different names — `nakshatra-phala` and `namakarana-nakshatra` both key
/// onto `nakshatra.ASHWINI` — so a migration that replaced would keep
/// whichever it read last (`03-design/state-readings.md` §3).
fn push_record(records: &mut Vec<(String, Entity)>, full: String, entity: Entity) {
    if let Some(slot) = records.iter_mut().find(|(k, _)| *k == full) {
        slot.1 = slot.1.overlaid(&entity);
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
///
/// **The base locale is the exception, and it took a second corpus to find
/// it.** `i18n/` already had `en-Latn/_meta.json`, so this only ever ran
/// for the others; a root created from scratch gave the base a fallback to
/// itself, which the validator refuses and rightly. A base locale falls
/// back to nothing and is complete by definition.
fn new_locale_meta(tag: &str, overlay: bool) -> Json {
    let base = tag == BASE_LOCALE;
    let complete = base && !overlay;
    let (and, or) = match tag {
        "hi-Deva-IN" => ("{0} और {1}", "{0} या {1}"),
        "sa-Deva" => ("{0} तथा {1}", "{0} वा {1}"),
        _ => ("{0} and {1}", "{0} or {1}"),
    };
    serde_json::json!({
        "locale": tag,
        "direction": "ltr",
        "numberingSystem": if base { "latn" } else { "deva" },
        "grouping": if base { vec![3] } else { vec![3, 2] },
        "decimal": ".",
        "group": ",",
        "fallback": if base { Vec::new() } else { vec![BASE_LOCALE] },
        "completeness": if complete { "strict" } else { "base" },
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
                format!(
                    "{}\n",
                    serde_json::to_string_pretty(&new_locale_meta(tag, migration.overlay))?
                ),
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

// ─── The rule readings ────────────────────────────────────────────────
//
// The same one-time import pointed at a different corpus: the baseline
// engine's reading for each yoga and dosha, in the same four languages
// (`03-design/interpretation-records.md`). A reading carries no parameters,
// so it is an **entity** of the catalogue's `rule` kind and not a message —
// 649 records over three fields would otherwise generate some 1 950 structs
// into four surfaces for text nothing interpolates.
//
// It is planned into a root of its own rather than into `i18n/`, because
// `crates/sdk/build.rs` compiles `i18n/` into every artefact and this corpus
// is 2.54 MB against an `i18n/` of 360 KB. A consumer computing a Julian day
// should not carry every Nepali yoga reading to do it, so the records ship
// as a pack that is loaded rather than one that is embedded.

/// The catalogue kind a rule reading is keyed under, and the catalogue's
/// only **open** kind: its members are a consumer's rule packs and not a
/// table, so a key here is checked for being well formed and the packs
/// decide whether it names a rule.
pub const RULE_KIND: &str = "rule";

/// The readings exporter's document.
#[derive(Clone, Debug, Deserialize)]
pub struct ReadingsDump {
    /// `teistro-conformance/baseline-readings/1`.
    pub schema: String,
    /// The engine and its version.
    pub tool: String,
    /// The export date.
    #[serde(default)]
    pub exported: String,
    /// The engine's language codes, in the document's order.
    pub languages: Vec<String>,
    /// Every effect facet the document uses, sorted. Reported rather than
    /// relied on: a record carries the facets its reading has.
    #[serde(default)]
    pub facets: Vec<String>,
    /// The readings, by rule key.
    pub readings: BTreeMap<String, ReadingRow>,
}

/// One rule's reading in every language the document carries.
#[derive(Clone, Debug, Deserialize)]
pub struct ReadingRow {
    /// `yoga` or `dosha`, which is the engine's own grouping.
    pub group: String,
    /// The classical text the reading cites, where it names one. **Not
    /// migrated**: a rule already carries its own citation, and a second
    /// copy beside the prose would be the one-rule-in-two-places shape.
    #[serde(default)]
    pub source: Option<String>,
    /// The reading per language code.
    #[serde(flatten)]
    pub languages: BTreeMap<String, Reading>,
}

/// One language's reading of one rule.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Reading {
    /// One sentence.
    #[serde(default)]
    pub summary: Option<String>,
    /// The passage.
    #[serde(default)]
    pub full: Option<String>,
    /// The named facets, an open set.
    #[serde(default)]
    pub effects: BTreeMap<String, String>,
}

/// Whether a key is one a rule pack could name.
///
/// The loader and this importer must agree on what a key looks like, so
/// there is one function and this is its name here:
/// [`source::is_member_key`](crate::source::is_member_key), which says why
/// it is looser than upper case.
#[must_use]
pub fn well_formed_rule_key(key: &str) -> bool {
    crate::source::is_member_key(key)
}

/// Plans the migration of a readings dump: every well-formed key becomes an
/// entity record of the `rule` kind for every language the dump carries.
///
/// The forms are `name` (the summary, which a record must have because
/// loading guarantees a non-empty name), `prose` (the full passage) and one
/// per effect facet. Nothing is inferred: a key that resembles another's is
/// not given its reading, because a rule's reading is a claim about that
/// rule (`03-design/interpretation-records.md` §4).
///
/// The report reads: `mapped` counts the records each of the engine's
/// groups gave, and `unknown_keys` lists what was skipped and why.
#[must_use]
pub fn plan_readings(dump: &ReadingsDump) -> Migration {
    let mut migration = Migration::default();
    let tags: BTreeMap<&str, &str> = LOCALES.iter().copied().collect();
    for (key, row) in &dump.readings {
        if !well_formed_rule_key(key) {
            migration
                .report
                .unknown_keys
                .push(format!("{key}: not a rule key"));
            continue;
        }
        let full = format!("{RULE_KIND}.{key}");
        let mut written_any = false;
        for (code, reading) in &row.languages {
            let Some(tag) = tags.get(code.as_str()) else {
                continue;
            };
            let Some(summary) = reading.summary.as_ref().filter(|s| !s.trim().is_empty()) else {
                migration
                    .report
                    .unknown_keys
                    .push(format!("{key} ({code}): no summary to name the record by"));
                continue;
            };
            let mut forms = BTreeMap::new();
            forms.insert(String::from("name"), summary.clone());
            if let Some(full_text) = reading.full.as_ref().filter(|s| !s.trim().is_empty()) {
                forms.insert(String::from("prose"), full_text.clone());
            }
            for (facet, text) in &reading.effects {
                if !text.trim().is_empty() {
                    forms.insert(facet.clone(), text.clone());
                }
            }
            push_record(
                migration.records.entry((*tag).to_string()).or_default(),
                full.clone(),
                Entity {
                    forms,
                    gender: None,
                    glyph: None,
                },
            );
            written_any = true;
        }
        if written_any {
            *migration
                .report
                .mapped
                .entry(row.group.clone())
                .or_default() += 1;
        }
    }
    migration
}

/// Reads a readings dump.
///
/// # Errors
///
/// A file that cannot be read or is not the readings exporter's document.
pub fn read_readings_dump(path: &Path) -> Result<ReadingsDump, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let dump: ReadingsDump =
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    if dump.schema != "teistro-conformance/baseline-readings/1" {
        return Err(format!(
            "{}: schema `{}` is not the readings exporter's",
            path.display(),
            dump.schema
        ));
    }
    Ok(dump)
}

// ---------------------------------------------------------------------------
// The state readings (`03-design/state-readings.md`): a reading for what a
// chart *is* rather than for a rule it triggers. The same document shape as
// the rule readings, pointed at 38 subjects instead of one.
// ---------------------------------------------------------------------------

/// The state readings exporter's schema.
pub const STATE_SCHEMA: &str = "teistro-conformance/baseline-state-readings/1";

/// One of the engine's state categories, and where its readings land.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StateCategory {
    /// The engine's own name for the category.
    pub category: &'static str,
    /// The catalogue kind its keys name.
    pub kind: &'static str,
    /// The form a reading becomes on the record, or `""` where the reading
    /// **is** the record and takes `name`, `prose` and its facets.
    ///
    /// A form rather than a record because 38 of the corpus's keys are
    /// shared between categories: `nakshatra-phala` and
    /// `namakarana-nakshatra` both key onto `nakshatra.ASHWINI` and mean
    /// different things by it (`03-design/state-readings.md` §3).
    pub form: &'static str,
}

const fn state(category: &'static str, kind: &'static str, form: &'static str) -> StateCategory {
    StateCategory {
        category,
        kind,
        form,
    }
}

/// Every state category this migration maps, with the kind and the form.
///
/// Nothing here is inferred from resemblance: a category the SDK has no
/// subject for is reported and skipped rather than guessed at, and the
/// fourteen that are missing from this list are named in
/// `03-design/state-readings.md` §8.
pub const STATE_CATEGORIES: [StateCategory; 24] = [
    state("avastha-baladi", "avastha_baladi", "phala"),
    state("avastha-deeptadi", "avastha_deeptadi", "phala"),
    state("avastha-jagradadi", "avastha_jagradadi", "phala"),
    state("avastha-lajjitadi", "avastha_lajjitadi", "phala"),
    state("dasha-lord-activation", "graha", "dashaActivation"),
    state("dasha-lord-effect", "graha", "dashaPhala"),
    state("dosha-timing", RULE_KIND, "timing"),
    state("gana", "gana", "phala"),
    state("graha-bhava", GRAHA_BHAVA_KIND, ""),
    state("graha-color", "graha", "colour"),
    state("graha-direction", "graha", "direction"),
    state("ishta-devata", "rashi", "ishtaDevata"),
    state("lagna-rashi", "rashi", "lagnaPhala"),
    state("mantra-ritual", "graha", "mantra"),
    state("nadi", "nadi", "phala"),
    state("nakshatra-phala", "nakshatra", "phala"),
    state("namakarana-nakshatra", "nakshatra", "namakarana"),
    state("special-lagna", "point", "phala"),
    state("tatwa", "tatwa", "phala"),
    state("tithi-phala", "tithi", "phala"),
    state("vara-phala", "vara", "phala"),
    state("varna", "varna", "phala"),
    state("yoga-phala", "yoga", "phala"),
    state("yoni", "yoni", "phala"),
];

/// The catalogue's second **open** kind: a graha and a bhava together.
///
/// Open for the same reason `rule` is — the catalogue holds the kind and
/// not the members — but for a different cause: these members are the
/// product of two closed kinds, and 108 rows of generated table would buy
/// a `resolve` the migration's own grammar already performs
/// (`03-design/state-readings.md` §4).
pub const GRAHA_BHAVA_KIND: &str = "graha_bhava";

/// What separates the graha from the house in a `graha-bhava` key.
const IN_BHAVA: &str = "_IN_";

/// The engine's graha abbreviations, written out because four of the nine
/// differ from the catalogue's spelling and a prefix rule cannot tell which.
pub const GRAHA_ABBREVIATIONS: [(&str, &str); 9] = [
    ("JUP", "JUPITER"),
    ("KETU", "KETU"),
    ("MARS", "MARS"),
    ("MER", "MERCURY"),
    ("MOON", "MOON"),
    ("RAHU", "RAHU"),
    ("SAT", "SATURN"),
    ("SUN", "SUN"),
    ("VEN", "VENUS"),
];

/// The engine's twelve lagna keys and the sign each names.
///
/// Written out rather than stripped of a prefix: a rule that took
/// `LAGNA_` off the front would work for all twelve and mis-file the
/// thirteenth silently, which is the inference
/// `03-design/interpretation-records.md` §4 forbids.
pub const LAGNA_RASHIS: [(&str, &str); 12] = [
    ("LAGNA_AQUARIUS", "AQUARIUS"),
    ("LAGNA_ARIES", "ARIES"),
    ("LAGNA_CANCER", "CANCER"),
    ("LAGNA_CAPRICORN", "CAPRICORN"),
    ("LAGNA_GEMINI", "GEMINI"),
    ("LAGNA_LEO", "LEO"),
    ("LAGNA_LIBRA", "LIBRA"),
    ("LAGNA_PISCES", "PISCES"),
    ("LAGNA_SAGITTARIUS", "SAGITTARIUS"),
    ("LAGNA_SCORPIO", "SCORPIO"),
    ("LAGNA_TAURUS", "TAURUS"),
    ("LAGNA_VIRGO", "VIRGO"),
];

/// The state readings exporter's document.
#[derive(Clone, Debug, Deserialize)]
pub struct StatesDump {
    /// `teistro-conformance/baseline-state-readings/1`.
    pub schema: String,
    /// The engine and its version.
    pub tool: String,
    /// The export date.
    #[serde(default)]
    pub exported: String,
    /// The engine's language codes, in the document's order.
    pub languages: Vec<String>,
    /// Every effect facet the document uses, sorted.
    #[serde(default)]
    pub facets: Vec<String>,
    /// The readings, by category and then by the engine's key.
    pub categories: BTreeMap<String, BTreeMap<String, StateRow>>,
}

/// One subject's reading in every language the document carries.
#[derive(Clone, Debug, Deserialize)]
pub struct StateRow {
    /// The classical text the reading cites. **Not migrated**: a citation
    /// beside the prose would be the one-fact-in-two-places shape.
    #[serde(default)]
    pub source: Option<String>,
    /// The reading per language code.
    #[serde(flatten)]
    pub languages: BTreeMap<String, Reading>,
}

/// The catalogue key a category's key names, or why it names none.
///
/// # Errors
///
/// A key no catalogue member and no written alias answers to, a
/// `graha-bhava` key that is not a graha and a house from 1 to 12, or a
/// reading of an open kind whose key no pack could name.
pub fn state_key(category: &str, kind: &str, key: &str) -> Result<String, String> {
    if kind == GRAHA_BHAVA_KIND {
        let (abbreviation, house) = key
            .split_once(IN_BHAVA)
            .ok_or_else(|| format!("`{key}` is not a graha and a house"))?;
        let graha = GRAHA_ABBREVIATIONS
            .iter()
            .find(|(short, _)| *short == abbreviation)
            .ok_or_else(|| format!("`{abbreviation}` is no graha of the engine's nine"))?
            .1;
        let bhava: u8 = house
            .parse()
            .map_err(|_| format!("`{house}` is not a house number"))?;
        if !(1..=12).contains(&bhava) {
            return Err(format!("house {bhava} is not one of twelve"));
        }
        return Ok(format!("{GRAHA_BHAVA_KIND}.{graha}{IN_BHAVA}{bhava}"));
    }
    if category == "lagna-rashi" {
        let sign = LAGNA_RASHIS
            .iter()
            .find(|(lagna, _)| *lagna == key)
            .ok_or_else(|| format!("`{key}` is no lagna of the engine's twelve"))?
            .1;
        return Ok(format!("{kind}.{sign}"));
    }
    let full = format!("{kind}.{key}");
    if crate::source::is_open_kind_key(&full) {
        return Ok(full);
    }
    resolve(&full)
        .map(|id| id.to_string())
        .map_err(|unknown| unknown.to_string())
}

/// The form a reading's summary, passage and facets take on a record.
///
/// An empty prefix means the reading **is** the record — `name`, `prose`
/// and the facet's own name, as a rule reading is. A prefix means it is a
/// form on a record that stands already, and every form it brings carries
/// the prefix so two categories on one subject cannot collide.
fn prefixed_form(prefix: &str, form: &str) -> String {
    if prefix.is_empty() {
        return String::from(form);
    }
    let mut out = String::from(prefix);
    let mut letters = form.chars();
    if let Some(first) = letters.next() {
        out.extend(first.to_uppercase());
        out.push_str(letters.as_str());
    }
    out
}

/// The forms one language's reading becomes under a prefix.
fn state_forms(reading: &Reading, prefix: &str) -> BTreeMap<String, String> {
    let mut forms = BTreeMap::new();
    let summary_form = if prefix.is_empty() {
        String::from("name")
    } else {
        String::from(prefix)
    };
    if let Some(summary) = reading.summary.as_ref().filter(|s| !s.trim().is_empty()) {
        forms.insert(summary_form, summary.clone());
    }
    if let Some(passage) = reading.full.as_ref().filter(|s| !s.trim().is_empty()) {
        forms.insert(prefixed_form(prefix, "prose"), passage.clone());
    }
    for (facet, text) in &reading.effects {
        if !text.trim().is_empty() {
            forms.insert(prefixed_form(prefix, facet), text.clone());
        }
    }
    forms
}

/// Plans the migration of a state readings dump: every category of
/// [`STATE_CATEGORIES`] becomes entity records, and every category that is
/// not in that list is reported unmapped with the records it would have
/// given.
///
/// The report reads: `mapped` counts the records each category gave,
/// `unmapped` names the categories the SDK has no subject for, and
/// `unknown_keys` lists every key that named no member and why.
#[must_use]
pub fn plan_states(dump: &StatesDump) -> Migration {
    let mut migration = Migration {
        overlay: true,
        ..Migration::default()
    };
    let tags: BTreeMap<&str, &str> = LOCALES.iter().copied().collect();
    for (category, rows) in &dump.categories {
        let Some(mapping) = STATE_CATEGORIES
            .iter()
            .find(|state| state.category == category)
        else {
            migration
                .report
                .unmapped
                .insert(category.clone(), rows.len());
            continue;
        };
        for (key, row) in rows {
            let full = match state_key(category, mapping.kind, key) {
                Ok(full) => full,
                Err(why) => {
                    migration
                        .report
                        .unknown_keys
                        .push(format!("{category}: {key}: {why}"));
                    continue;
                }
            };
            let mut written_any = false;
            for (code, reading) in &row.languages {
                let Some(tag) = tags.get(code.as_str()) else {
                    continue;
                };
                let forms = state_forms(reading, mapping.form);
                if forms.is_empty() {
                    migration
                        .report
                        .unknown_keys
                        .push(format!("{category}: {key} ({code}): nothing to say"));
                    continue;
                }
                push_record(
                    migration.records.entry((*tag).to_string()).or_default(),
                    full.clone(),
                    Entity {
                        forms,
                        gender: None,
                        glyph: None,
                    },
                );
                written_any = true;
            }
            if written_any {
                *migration.report.mapped.entry(category.clone()).or_default() += 1;
            }
        }
    }
    migration
}

/// Reads a state readings dump.
///
/// # Errors
///
/// A file that cannot be read or is not the state readings exporter's
/// document.
pub fn read_states_dump(path: &Path) -> Result<StatesDump, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let dump: StatesDump =
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    if dump.schema != STATE_SCHEMA {
        return Err(format!(
            "{}: schema `{}` is not the state readings exporter's",
            path.display(),
            dump.schema
        ));
    }
    Ok(dump)
}
