//! A locale derived from another by transliteration: `sa-Latn` from
//! `sa-Deva`, so a Latin-script reader of Sanskrit terms gets every
//! entity without anyone writing them twice
//! (`02-architecture/03-localization-architecture.md`, "Axes that are not
//! the language").
//!
//! The derived files are generated, and a hand override belongs in
//! `_overrides.json` beside them rather than in the file itself, so
//! regenerating never loses a correction and an override that no longer
//! applies is reported rather than left to rot.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value as Json, json};

use crate::source::{ENTITY_NAMESPACE, Entity, Entry, META_FILE, Tree};
use crate::translit::{Script, transliterate};

/// The file a derived locale keeps its hand corrections in.
pub const OVERRIDES_FILE: &str = "_overrides.json";

/// What deriving a locale produced: the files and what they say.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Derived {
    /// The files, relative to the `i18n/` root, and their text.
    pub files: Vec<(PathBuf, String)>,
    /// The entities written.
    pub entities: usize,
    /// The forms an override replaced.
    pub overridden: usize,
    /// The overrides that matched nothing, which are a fault: the source
    /// they corrected has moved or gone.
    pub stale: Vec<String>,
    /// The derived names that are letter for letter the source's own
    /// `iast` form, which is the measure of the transliteration: the rest
    /// are the source's variants (an `iast` form that adds the category
    /// word, a differently spelled Devanagari name), not the table's
    /// mistakes.
    pub agreeing: usize,
}

/// A locale derived from another by transliteration.
///
/// `overrides` is the target's `_overrides.json` as it stands: a map of
/// `<kind>.<KEY>` to the forms that replace the mechanical result
/// (`{"graha.SUN": {"name": "Sūrya"}}`).
///
/// # Errors
///
/// No such source locale, or a script pair with no table.
pub fn derive(
    tree: &Tree,
    from: &str,
    to: &str,
    overrides: &BTreeMap<String, BTreeMap<String, String>>,
) -> Result<Derived, String> {
    let source = tree
        .locales
        .get(from)
        .ok_or_else(|| format!("no locale `{from}` in the sources"))?;
    let script = Script::from_key(script_of(to))
        .ok_or_else(|| format!("`{to}` names no script this build transliterates into"))?;
    let latin = |text: &str| -> Result<String, String> {
        transliterate(text, Script::Devanagari, script).map_err(|e| e.to_string())
    };
    let mut out = Derived::default();
    let mut used: BTreeMap<&String, bool> = overrides.keys().map(|key| (key, false)).collect();

    // The metadata: the source's, in the target's script.
    let mut meta: Json =
        serde_json::to_value(&source.meta).map_err(|e| format!("{from}'s metadata: {e}"))?;
    if let Some(object) = meta.as_object_mut() {
        object.insert(String::from("locale"), json!(to));
        object.insert(String::from("numberingSystem"), json!("latn"));
        object.insert(String::from("grouping"), json!([3]));
        if let Some(Json::Object(patterns)) = object.get_mut("listPatterns") {
            for pattern in patterns.values_mut() {
                if let Json::Object(parts) = pattern {
                    for part in parts.values_mut() {
                        if let Json::String(text) = part {
                            *part = json!(latin(text)?);
                        }
                    }
                }
            }
        }
    }
    out.files.push((
        PathBuf::from(to).join(META_FILE),
        format!("{}\n", pretty(&meta)?),
    ));

    // The entities: every form transliterated, `iast` kept because it is
    // already in the target's script, and an override last.
    let Some(namespace) = source.namespaces.get(ENTITY_NAMESPACE) else {
        return Ok(out);
    };
    let mut document = Map::new();
    for key in &namespace.order {
        let Some(Entry::Entity(entity)) = namespace.entries.get(key) else {
            continue;
        };
        let Some((kind, member)) = key.split_once('.') else {
            continue;
        };
        let mut derived = Entity {
            forms: BTreeMap::new(),
            gender: entity.gender.clone(),
            glyph: entity.glyph.clone(),
        };
        for (form, text) in &entity.forms {
            // `iast` is already in the target's script; the rest are
            // transliterated and given the capital a Latin-script name
            // carries, which is how the sources' own `iast` forms are
            // written.
            let value = if form == "iast" {
                text.clone()
            } else {
                capitalised(&latin(text)?)
            };
            derived.forms.insert(form.clone(), value);
        }
        if derived.forms.get("name") == entity.forms.get("iast") {
            out.agreeing += 1;
        }
        if let Some(corrections) = overrides.get(key) {
            used.insert(overrides.get_key_value(key).map_or(key, |(k, _)| k), true);
            for (form, text) in corrections {
                derived.forms.insert(form.clone(), text.clone());
                out.overridden += 1;
            }
        }
        let group = document
            .entry(kind.to_string())
            .or_insert_with(|| Json::Object(Map::new()));
        if let Json::Object(group) = group {
            group.insert(member.to_string(), entity_json(&derived));
        }
        out.entities += 1;
    }
    out.stale = used
        .into_iter()
        .filter(|(_, used)| !used)
        .map(|(key, _)| key.clone())
        .collect();
    out.files.push((
        PathBuf::from(to).join(format!("{ENTITY_NAMESPACE}.json")),
        format!("{}\n", pretty(&Json::Object(document))?),
    ));
    Ok(out)
}

/// The overrides beside a derived locale, or none when it has no file.
///
/// # Errors
///
/// A file that cannot be read or does not parse.
pub fn overrides_of(
    root: &Path,
    locale: &str,
) -> Result<BTreeMap<String, BTreeMap<String, String>>, String> {
    let path = root.join(locale).join(OVERRIDES_FILE);
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// A name as a Latin script writes one: every word's first letter a
/// capital, which is how the sources' own `iast` forms are written
/// (`Aśvinī Kumāra`, not `Aśvinī kumāra`).
fn capitalised(text: &str) -> String {
    text.split(' ')
        .map(|word| {
            let mut letters = word.chars();
            letters.next().map_or_else(String::new, |first| {
                first.to_uppercase().chain(letters).collect::<String>()
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The script half of a locale tag (`sa-Latn` is `Latn`), lowercased.
fn script_of(tag: &str) -> &str {
    tag.split('-').nth(1).unwrap_or("")
}

/// The entity as its source file writes it: the named forms first, in the
/// order a reader expects, then the glyph and the gender.
fn entity_json(entity: &Entity) -> Json {
    let mut object = Map::new();
    for form in ["short", "name", "prose", "iast"] {
        if let Some(value) = entity.forms.get(form) {
            object.insert(form.to_string(), Json::String(value.clone()));
        }
    }
    for (form, value) in &entity.forms {
        object
            .entry(form.clone())
            .or_insert_with(|| Json::String(value.clone()));
    }
    if let Some(glyph) = &entity.glyph {
        object.insert(String::from("glyph"), Json::String(glyph.clone()));
    }
    if let Some(gender) = &entity.gender {
        object.insert(String::from("gender"), Json::String(gender.clone()));
    }
    Json::Object(object)
}

fn pretty(value: &Json) -> Result<String, String> {
    serde_json::to_string_pretty(value).map_err(|e| e.to_string())
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

    fn tree() -> Tree {
        Tree::load(&sdk_root()).unwrap_or_else(|e| panic!("{e}"))
    }

    #[test]
    fn a_latin_locale_is_derived_from_the_devanagari_one() {
        let derived = derive(&tree(), "sa-Deva", "sa-Latn", &BTreeMap::new())
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(derived.files.len(), 2);
        assert_eq!(derived.entities, 274);
        assert!(derived.stale.is_empty());

        let entities = &derived.files[1].1;
        assert!(entities.contains("\"name\": \"Sūrya\""), "the Sun's name");
        assert!(
            entities.contains("\"iast\": \"Sūrya\""),
            "the iast form stands"
        );
        assert!(entities.contains("\"glyph\": \"☉\""));
        let meta = &derived.files[0].1;
        assert!(meta.contains("\"locale\": \"sa-Latn\""));
        assert!(meta.contains("\"numberingSystem\": \"latn\""));
        assert!(meta.contains("tathā"), "the list pattern is transliterated");
        assert_eq!(
            derived.agreeing, 241,
            "the derived names that are letter for letter the sources' own iast form"
        );
    }

    #[test]
    fn an_override_replaces_the_mechanical_form_and_a_stale_one_is_named() {
        let mut overrides = BTreeMap::new();
        overrides.insert(
            String::from("graha.SUN"),
            BTreeMap::from([(String::from("name"), String::from("Sūrya"))]),
        );
        overrides.insert(
            String::from("graha.VULCAN"),
            BTreeMap::from([(String::from("name"), String::from("Vulcan"))]),
        );
        let derived =
            derive(&tree(), "sa-Deva", "sa-Latn", &overrides).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(derived.overridden, 1);
        assert_eq!(derived.stale, vec![String::from("graha.VULCAN")]);
        assert!(derived.files[1].1.contains("\"name\": \"Sūrya\""));
    }

    #[test]
    fn a_locale_or_a_script_the_build_does_not_know_is_refused_by_name() {
        assert!(
            derive(&tree(), "xx-Deva", "xx-Latn", &BTreeMap::new())
                .unwrap_err()
                .contains("no locale `xx-Deva`")
        );
        assert!(
            derive(&tree(), "sa-Deva", "sa-Taml", &BTreeMap::new())
                .unwrap_err()
                .contains("names no script")
        );
    }
}
