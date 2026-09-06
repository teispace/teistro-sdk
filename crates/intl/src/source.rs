//! The `i18n/` conventions: a root holding one directory per locale, each
//! with `_meta.json` and one JSON file per namespace; nested objects whose
//! leaves are messages, or entity records in the entity namespace; keys as
//! dotted paths. Loading validates the shape (names, kinds, types) and
//! nothing about the content; the content gates are [`crate::validate`].

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// The base locale: complete by definition, the source of keys and
/// parameters for every other locale.
pub const BASE_LOCALE: &str = "en-Latn";
/// The locale metadata file inside every locale directory.
pub const META_FILE: &str = "_meta.json";
/// The namespace whose leaves are entity records rather than messages.
pub const ENTITY_NAMESPACE: &str = "sdk.entity";
/// The entity form every record must carry.
pub const NAME_FORM: &str = "name";

/// Text direction.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    /// Left to right.
    #[default]
    Ltr,
    /// Right to left.
    Rtl,
}

/// What a locale promises about coverage.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Completeness {
    /// Every base key is present; gated. SDK-shipped locales.
    #[default]
    Strict,
    /// Missing keys fall back along the chain. Consumer packs.
    Base,
}

/// The patterns of one list type, CLDR style: `{0}` and `{1}`.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ListPattern {
    /// Two items.
    pub pair: String,
    /// Joining the accumulated start to a middle item.
    pub middle: String,
    /// Joining the accumulated start to the last item.
    pub end: String,
}

/// `_meta.json`.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Meta {
    /// The locale tag; must equal the directory name.
    pub locale: String,
    /// Text direction.
    #[serde(default)]
    pub direction: Direction,
    /// The CLDR numbering system every number renders in.
    #[serde(default = "latn")]
    pub numbering_system: String,
    /// Digit group sizes from the right; the last repeats (`[3, 2]` for
    /// the Indian grouping).
    #[serde(default = "three")]
    pub grouping: Vec<u8>,
    /// The decimal separator.
    #[serde(default = "full_stop")]
    pub decimal: char,
    /// The group separator.
    #[serde(default = "comma")]
    pub group: char,
    /// The fallback chain, nearest first, ending in the base locale for a
    /// non-base locale.
    #[serde(default)]
    pub fallback: Vec<String>,
    /// What the locale promises.
    #[serde(default)]
    pub completeness: Completeness,
    /// The contexts messages may select on, each with its closed value set.
    #[serde(default)]
    pub contexts: BTreeMap<String, Vec<String>>,
    /// The default term style.
    #[serde(default)]
    pub term_style: Option<String>,
    /// List patterns by type (`and`, `or`).
    #[serde(default)]
    pub list_patterns: BTreeMap<String, ListPattern>,
}

fn latn() -> String {
    String::from("latn")
}

fn three() -> Vec<u8> {
    vec![3]
}

const fn full_stop() -> char {
    '.'
}

const fn comma() -> char {
    ','
}

/// An entity record: named forms, a gender from the `gender` context,
/// and an optional glyph.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Entity {
    /// The forms by name: `short`, `name`, `prose`, `iast`, and any the
    /// locale adds.
    pub forms: BTreeMap<String, String>,
    /// The grammatical gender, a value of the `gender` context.
    pub gender: Option<String>,
    /// The glyph.
    pub glyph: Option<String>,
}

impl Entity {
    /// A form by name; `glyph` is a form too.
    #[must_use]
    pub fn form(&self, form: &str) -> Option<&str> {
        if form == "glyph" {
            return self.glyph.as_deref();
        }
        self.forms.get(form).map(String::as_str)
    }

    /// The `name` form, which loading guarantees.
    #[must_use]
    pub fn name(&self) -> &str {
        self.forms.get(NAME_FORM).map_or("", String::as_str)
    }
}

/// A leaf of a namespace.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Entry {
    /// A `MessageFormat 2` source.
    Message(String),
    /// An entity record.
    Entity(Entity),
}

/// One namespace of one locale: keys are dotted paths inside the
/// namespace (`graha.SUN`, `strength.score`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Namespace {
    /// The entries by key.
    pub entries: BTreeMap<String, Entry>,
    /// The keys in source order, which the generators follow so an enum
    /// lists the Sun before the Moon; a pack does not carry it.
    pub order: Vec<String>,
}

impl Namespace {
    /// Adds an entry, keeping source order.
    pub fn insert(&mut self, key: String, entry: Entry) {
        if !self.entries.contains_key(&key) {
            self.order.push(key.clone());
        }
        self.entries.insert(key, entry);
    }

    /// The entries in source order, then any without a recorded order.
    pub fn in_source_order(&self) -> impl Iterator<Item = (&String, &Entry)> {
        let ordered = self
            .order
            .iter()
            .filter_map(|key| self.entries.get_key_value(key));
        let rest = self
            .entries
            .iter()
            .filter(|(key, _)| !self.order.contains(key));
        ordered.chain(rest)
    }
}

/// One locale directory.
#[derive(Clone, Debug, PartialEq)]
pub struct LocaleSource {
    /// The tag, the directory name.
    pub tag: String,
    /// `_meta.json`.
    pub meta: Meta,
    /// The namespaces by name.
    pub namespaces: BTreeMap<String, Namespace>,
}

impl LocaleSource {
    /// Splits a full key into its namespace and the key inside it, taking
    /// the longest namespace the locale has.
    #[must_use]
    pub fn split<'k>(&self, full_key: &'k str) -> Option<(&str, &'k str)> {
        self.namespaces
            .keys()
            .filter_map(|ns| {
                full_key
                    .strip_prefix(ns.as_str())
                    .and_then(|rest| rest.strip_prefix('.'))
                    .map(|rest| (ns.as_str(), rest))
            })
            .max_by_key(|(ns, _)| ns.len())
    }

    /// The entry at a full key.
    #[must_use]
    pub fn entry(&self, full_key: &str) -> Option<&Entry> {
        let (ns, key) = self.split(full_key)?;
        self.namespaces.get(ns)?.entries.get(key)
    }

    /// The entity at a catalogue key (`graha.SUN`).
    #[must_use]
    pub fn entity(&self, catalogue_key: &str) -> Option<&Entity> {
        match self
            .namespaces
            .get(ENTITY_NAMESPACE)?
            .entries
            .get(catalogue_key)?
        {
            Entry::Entity(entity) => Some(entity),
            Entry::Message(_) => None,
        }
    }

    /// Every full key, sorted.
    pub fn keys(&self) -> impl Iterator<Item = String> + '_ {
        self.namespaces
            .iter()
            .flat_map(|(ns, namespace)| namespace.entries.keys().map(move |k| format!("{ns}.{k}")))
    }
}

/// A loaded `i18n/` root.
#[derive(Clone, Debug, PartialEq)]
pub struct Tree {
    /// Where it was read from.
    pub root: PathBuf,
    /// The locales by tag.
    pub locales: BTreeMap<String, LocaleSource>,
}

impl Tree {
    /// The base locale, when present.
    #[must_use]
    pub fn base(&self) -> Option<&LocaleSource> {
        self.locales.get(BASE_LOCALE)
    }

    /// Loads every locale directory under `root`.
    ///
    /// # Errors
    ///
    /// The first file that cannot be read, parsed, or that breaks the
    /// shape rules, with its path.
    pub fn load(root: &Path) -> Result<Tree, SourceError> {
        let mut locales = BTreeMap::new();
        let mut dirs: Vec<PathBuf> = std::fs::read_dir(root)
            .map_err(|e| SourceError::new(root, e.to_string()))?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_dir())
            .collect();
        dirs.sort();
        for dir in dirs {
            let locale = load_locale(&dir)?;
            locales.insert(locale.tag.clone(), locale);
        }
        if locales.is_empty() {
            return Err(SourceError::new(root, "no locale directories"));
        }
        Ok(Tree {
            root: root.to_path_buf(),
            locales,
        })
    }
}

/// A file that could not be loaded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceError {
    /// The file or directory.
    pub path: PathBuf,
    /// What is wrong.
    pub detail: String,
}

impl SourceError {
    fn new(path: &Path, detail: impl Into<String>) -> SourceError {
        SourceError {
            path: path.to_path_buf(),
            detail: detail.into(),
        }
    }
}

impl fmt::Display for SourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path.display(), self.detail)
    }
}

impl std::error::Error for SourceError {}

/// Whether `s` is a locale tag with a script: `language-Script[-REGION]`.
#[must_use]
pub fn is_locale_tag(s: &str) -> bool {
    let mut parts = s.split('-');
    let language = parts.next().unwrap_or_default();
    let script = parts.next().unwrap_or_default();
    let region = parts.next();
    let language_ok =
        (2..=3).contains(&language.len()) && language.bytes().all(|b| b.is_ascii_lowercase());
    let script_ok = script.len() == 4
        && script
            .bytes()
            .next()
            .is_some_and(|b| b.is_ascii_uppercase())
        && script.bytes().skip(1).all(|b| b.is_ascii_lowercase());
    let region_ok = region.is_none_or(|r| {
        (r.len() == 2 && r.bytes().all(|b| b.is_ascii_uppercase()))
            || (r.len() == 3 && r.bytes().all(|b| b.is_ascii_digit()))
    });
    language_ok && script_ok && region_ok && parts.next().is_none()
}

/// Whether `s` is a namespace name: lowercase words joined by dots.
#[must_use]
pub fn is_namespace_name(s: &str) -> bool {
    !s.is_empty()
        && s.split('.').all(|word| {
            word.bytes().next().is_some_and(|b| b.is_ascii_lowercase())
                && word
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        })
}

/// Whether `s` is a key segment: a `camelCase` identifier, or a catalogue
/// key (`UPPER_SNAKE`) for entities and rule keys.
#[must_use]
pub fn is_key_segment(s: &str) -> bool {
    let Some(first) = s.bytes().next() else {
        return false;
    };
    // The catalogue's kind names are the groups of the entity namespace,
    // and a kind of two words is written with an underscore
    // (`avastha_baladi`).
    if teistro_core::catalogue::Kind::from_name(s).is_some() {
        return true;
    }
    if first.is_ascii_lowercase() {
        s.bytes().all(|b| b.is_ascii_alphanumeric())
    } else if first.is_ascii_uppercase() {
        s.bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
    } else {
        false
    }
}

fn load_locale(dir: &Path) -> Result<LocaleSource, SourceError> {
    let tag = dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    if !is_locale_tag(&tag) {
        return Err(SourceError::new(
            dir,
            "a locale directory is named by a tag with a script, like `ne-Deva-NP`",
        ));
    }
    let meta_path = dir.join(META_FILE);
    let meta_text = std::fs::read_to_string(&meta_path)
        .map_err(|e| SourceError::new(&meta_path, e.to_string()))?;
    let meta: Meta = serde_json::from_str(&meta_text)
        .map_err(|e| SourceError::new(&meta_path, e.to_string()))?;
    if meta.locale != tag {
        return Err(SourceError::new(
            &meta_path,
            format!("`locale` is {} but the directory is {tag}", meta.locale),
        ));
    }
    let mut namespaces = BTreeMap::new();
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| SourceError::new(dir, e.to_string()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|e| e == "json"))
        .filter(|path| path.file_name().is_none_or(|n| n != META_FILE))
        .collect();
    files.sort();
    for file in files {
        let name = file
            .file_stem()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if !is_namespace_name(&name) {
            return Err(SourceError::new(
                &file,
                "a namespace file is named `<namespace>.json` with lowercase dotted words",
            ));
        }
        let text =
            std::fs::read_to_string(&file).map_err(|e| SourceError::new(&file, e.to_string()))?;
        let value: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| SourceError::new(&file, e.to_string()))?;
        let mut namespace = Namespace::default();
        flatten(&file, &name, "", &value, &mut namespace)?;
        namespaces.insert(name, namespace);
    }
    Ok(LocaleSource {
        tag,
        meta,
        namespaces,
    })
}

fn flatten(
    file: &Path,
    namespace: &str,
    prefix: &str,
    value: &serde_json::Value,
    out: &mut Namespace,
) -> Result<(), SourceError> {
    let serde_json::Value::Object(object) = value else {
        return Err(SourceError::new(
            file,
            format!("`{prefix}` must be an object of keys"),
        ));
    };
    if namespace == ENTITY_NAMESPACE
        && object
            .get(NAME_FORM)
            .is_some_and(serde_json::Value::is_string)
    {
        out.insert(
            prefix.to_string(),
            Entry::Entity(entity(file, prefix, object)?),
        );
        return Ok(());
    }
    for (segment, child) in object {
        if !is_key_segment(segment) {
            return Err(SourceError::new(
                file,
                format!(
                    "key segment `{segment}` under `{prefix}` is neither camelCase nor a catalogue key"
                ),
            ));
        }
        let key = if prefix.is_empty() {
            segment.clone()
        } else {
            format!("{prefix}.{segment}")
        };
        match child {
            serde_json::Value::String(text) => {
                if namespace == ENTITY_NAMESPACE {
                    return Err(SourceError::new(
                        file,
                        format!("`{key}`: the entity namespace holds records, not messages"),
                    ));
                }
                out.insert(key, Entry::Message(text.clone()));
            }
            serde_json::Value::Object(_) => flatten(file, namespace, &key, child, out)?,
            _ => {
                return Err(SourceError::new(
                    file,
                    format!("`{key}` must be a string message or an object"),
                ));
            }
        }
    }
    Ok(())
}

fn entity(
    file: &Path,
    key: &str,
    object: &serde_json::Map<String, serde_json::Value>,
) -> Result<Entity, SourceError> {
    let mut entity = Entity::default();
    for (field, value) in object {
        let serde_json::Value::String(text) = value else {
            return Err(SourceError::new(
                file,
                format!("`{key}.{field}` must be a string"),
            ));
        };
        match field.as_str() {
            "gender" => entity.gender = Some(text.clone()),
            "glyph" => entity.glyph = Some(text.clone()),
            _ => {
                if !field.bytes().all(|b| b.is_ascii_lowercase()) {
                    return Err(SourceError::new(
                        file,
                        format!("`{key}.{field}`: a form name is lowercase letters"),
                    ));
                }
                entity.forms.insert(field.clone(), text.clone());
            }
        }
    }
    Ok(entity)
}

/// The SDK's own sources, `i18n/` at the repository root: what the tests,
/// the gate and the command line read by default.
#[must_use]
pub fn sdk_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../i18n")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, reason = "tests fail by panicking")]

    use super::*;

    #[test]
    fn the_sdk_sources_load() {
        let tree = Tree::load(&sdk_root()).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(tree.locales.len(), 4);
        let base = tree.base().unwrap_or_else(|| panic!("base"));
        assert_eq!(base.meta.numbering_system, "latn");
        let sun = base.entity("graha.SUN").unwrap_or_else(|| panic!("sun"));
        assert_eq!(sun.form("prose"), Some("the Sun"));
        assert_eq!(sun.gender.as_deref(), Some("m"));
        assert!(matches!(
            base.entry("sdk.reason.strength.score"),
            Some(Entry::Message(_))
        ));
        assert_eq!(
            base.split("sdk.reason.strength.score"),
            Some(("sdk.reason", "strength.score"))
        );
        let ne = tree
            .locales
            .get("ne-Deva-NP")
            .unwrap_or_else(|| panic!("ne"));
        assert_eq!(ne.meta.fallback, ["en-Latn"]);
        assert_eq!(ne.meta.grouping, [3, 2]);
    }

    #[test]
    fn the_grammars_hold() {
        assert!(is_locale_tag("en-Latn"));
        assert!(is_locale_tag("ne-Deva-NP"));
        assert!(is_locale_tag("es-Latn-419"));
        assert!(!is_locale_tag("en"));
        assert!(!is_locale_tag("en-latn"));
        assert!(is_namespace_name("sdk.entity"));
        assert!(!is_namespace_name("Sdk.entity"));
        assert!(is_key_segment("grahaInBhava"));
        assert!(is_key_segment("PURVA_PHALGUNI"));
        assert!(!is_key_segment("graha_in"));
        assert!(!is_key_segment("1st"));
    }
}
