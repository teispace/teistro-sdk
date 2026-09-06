//! The engine's runtime API (`02-architecture/03-localization-architecture.md`,
//! "Runtime API"): a pack or bundle loaded after construction adds a
//! locale or a namespace or replaces entries, an in-memory override
//! patches one message without a rebuild, and the report says what is
//! loaded, overridden and covered. Every render still says which locale
//! answered and whether an override did ([`crate::Rendered`]).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use serde::Serialize;

use crate::mf2::parse;
use crate::pack::{BUNDLE_MAGIC, Bundle, Pack, PackError};
use crate::render::{Intl, IntlError, plurals_for};
use crate::source::{BASE_LOCALE, Entry, LocaleSource, Meta, Namespace};

/// A pack or bundle loaded at runtime: what the provenance envelope
/// records (ADR-0020).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Loaded {
    /// The locale.
    pub locale: String,
    /// The namespaces the file carried.
    pub namespaces: Vec<String>,
    /// The entries it carried.
    pub entries: usize,
    /// The entries that replaced ones already loaded.
    pub replaced: usize,
    /// The file's SHA-256, lower-case hex.
    pub sha256: String,
}

/// One locale in the runtime report.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LocaleReport {
    /// The tag.
    pub tag: String,
    /// The namespaces loaded.
    pub namespaces: Vec<String>,
    /// The entries loaded.
    pub entries: usize,
    /// The overrides in force.
    pub overrides: usize,
    /// Base keys present (through an entry or an override) against base
    /// keys, when the base locale is loaded.
    pub coverage: Option<(usize, usize)>,
}

/// What the engine holds: the current locale, every locale with its
/// coverage, the files loaded at runtime and the overrides in force.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RuntimeReport {
    /// The current locale.
    pub current: String,
    /// Every locale, sorted.
    pub locales: Vec<LocaleReport>,
    /// The files loaded at runtime, in order.
    pub loaded: Vec<Loaded>,
    /// The overrides in force, every locale's.
    pub overrides: usize,
}

impl RuntimeReport {
    /// The report as Markdown.
    #[must_use]
    pub fn markdown(&self) -> String {
        let mut out = format!(
            "current locale {}; {} loaded at runtime; {} overrides\n\n| locale | namespaces | entries | overrides | coverage |\n|---|---|---:|---:|---:|\n",
            self.current,
            self.loaded.len(),
            self.overrides
        );
        for locale in &self.locales {
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} | {} |",
                locale.tag,
                locale.namespaces.join(", "),
                locale.entries,
                locale.overrides,
                locale
                    .coverage
                    .map_or_else(|| String::from("—"), |(p, t)| format!("{p}/{t}"))
            );
        }
        if !self.loaded.is_empty() {
            out.push_str("\n| loaded | namespaces | entries | replaced | sha256 |\n|---|---|---:|---:|---|\n");
            for file in &self.loaded {
                let _ = writeln!(
                    out,
                    "| {} | {} | {} | {} | {} |",
                    file.locale,
                    file.namespaces.join(", "),
                    file.entries,
                    file.replaced,
                    file.sha256
                );
            }
        }
        out
    }
}

/// The content of one file: its locale, its metadata when it carries any,
/// its namespaces and its hash.
struct FileContent {
    tag: String,
    meta: Option<Meta>,
    namespaces: Vec<(String, Namespace)>,
    sha256: String,
}

fn pack_error(error: PackError) -> IntlError {
    IntlError(error.0)
}

fn read_file(bytes: &[u8]) -> Result<FileContent, IntlError> {
    if bytes.starts_with(&BUNDLE_MAGIC) {
        let bundle = Bundle::parse(bytes).map_err(pack_error)?;
        return Ok(FileContent {
            tag: bundle.locale().to_string(),
            meta: Some(bundle.meta().map_err(pack_error)?),
            namespaces: bundle
                .packs()
                .iter()
                .map(|pack| (pack.namespace().to_string(), pack.to_namespace()))
                .collect(),
            sha256: bundle.content_sha256(),
        });
    }
    let pack = Pack::parse(bytes).map_err(pack_error)?;
    let meta = if pack.has_meta() {
        Some(pack.meta().map_err(pack_error)?)
    } else {
        None
    };
    Ok(FileContent {
        tag: pack.locale().to_string(),
        meta,
        namespaces: vec![(pack.namespace().to_string(), pack.to_namespace())],
        sha256: pack.content_sha256(),
    })
}

impl Intl {
    /// Loads a `.tpack` or a `.tbundle` after construction: a new locale
    /// is added (its metadata from the file, its plural rules from ICU4X),
    /// a known locale gains the namespaces and every entry the file
    /// carries, replacing what was loaded before under the same key. The
    /// file is verified as on any read; its hash goes to the record
    /// returned and to [`Intl::report`].
    ///
    /// # Errors
    ///
    /// A file that does not parse or verify; a pack without metadata for a
    /// locale not yet loaded; a locale ICU4X has no plural rules for.
    pub fn load_pack(&mut self, bytes: &[u8]) -> Result<Loaded, IntlError> {
        let file = read_file(bytes)?;
        if !self.locales.contains_key(&file.tag) {
            let meta = file.meta.ok_or_else(|| {
                IntlError(format!(
                    "{}: the pack carries no metadata and the locale is not loaded",
                    file.tag
                ))
            })?;
            self.plurals
                .insert(file.tag.clone(), plurals_for(&file.tag)?);
            self.locales.insert(
                file.tag.clone(),
                LocaleSource {
                    tag: file.tag.clone(),
                    meta,
                    namespaces: BTreeMap::new(),
                },
            );
        }
        let locale = self
            .locales
            .get_mut(&file.tag)
            .ok_or_else(|| IntlError(format!("no locale {}", file.tag)))?;
        let mut entries = 0;
        let mut replaced = 0;
        let mut names = Vec::with_capacity(file.namespaces.len());
        for (name, namespace) in file.namespaces {
            let target = locale.namespaces.entry(name.clone()).or_default();
            for (key, entry) in namespace.entries {
                entries += 1;
                if target.entries.contains_key(&key) {
                    replaced += 1;
                }
                target.insert(key, entry);
            }
            names.push(name);
        }
        self.forget_parsed(&file.tag, None);
        let loaded = Loaded {
            locale: file.tag,
            namespaces: names,
            entries,
            replaced,
            sha256: file.sha256,
        };
        self.loaded.push(loaded.clone());
        Ok(loaded)
    }

    /// Overrides one message of a locale in memory: the source is checked
    /// now and stands before the locale's own entry, and before any
    /// fallback, until cleared. A key the locale lacks becomes available.
    ///
    /// # Errors
    ///
    /// An unknown locale, or a source that does not parse (with its
    /// offset).
    pub fn set_override(&mut self, tag: &str, key: &str, source: &str) -> Result<(), IntlError> {
        if !self.locales.contains_key(tag) {
            return Err(IntlError(format!("unknown locale {tag}")));
        }
        parse(source)
            .map_err(|e| IntlError(format!("override `{key}` in {tag} does not parse: {e}")))?;
        self.overrides
            .entry(tag.to_string())
            .or_default()
            .insert(key.to_string(), Entry::Message(source.to_string()));
        self.forget_parsed(tag, Some(key));
        Ok(())
    }

    /// Several overrides of one locale at once.
    ///
    /// # Errors
    ///
    /// As [`Intl::set_override`]; the overrides before the failing one
    /// stand.
    pub fn set_overrides<'k>(
        &mut self,
        tag: &str,
        overrides: impl IntoIterator<Item = (&'k str, &'k str)>,
    ) -> Result<(), IntlError> {
        for (key, source) in overrides {
            self.set_override(tag, key, source)?;
        }
        Ok(())
    }

    /// Clears one override; whether there was one.
    pub fn clear_override(&mut self, tag: &str, key: &str) -> bool {
        let cleared = self
            .overrides
            .get_mut(tag)
            .is_some_and(|overrides| overrides.remove(key).is_some());
        if cleared {
            self.forget_parsed(tag, Some(key));
        }
        cleared
    }

    /// Clears every override of a locale, or of every locale.
    pub fn clear_overrides(&mut self, tag: Option<&str>) {
        if let Some(tag) = tag {
            self.overrides.remove(tag);
            self.forget_parsed(tag, None);
        } else {
            let tags: Vec<String> = self.overrides.keys().cloned().collect();
            self.overrides.clear();
            for tag in tags {
                self.forget_parsed(&tag, None);
            }
        }
    }

    /// The overrides in force, every locale's.
    #[must_use]
    pub fn override_count(&self) -> usize {
        self.overrides.values().map(BTreeMap::len).sum()
    }

    /// The files loaded at runtime, in order.
    #[must_use]
    pub fn loaded(&self) -> &[Loaded] {
        &self.loaded
    }

    /// What the engine holds: every locale with its coverage of the base
    /// locale's keys, the files loaded at runtime, the overrides in force.
    #[must_use]
    pub fn report(&self) -> RuntimeReport {
        let base_keys: Option<BTreeSet<String>> = self
            .locales
            .get(BASE_LOCALE)
            .map(|base| base.keys().collect());
        let locales = self
            .locales
            .values()
            .map(|locale| {
                let overrides = self.overrides.get(&locale.tag);
                let coverage = base_keys.as_ref().map(|keys| {
                    let present = keys
                        .iter()
                        .filter(|key| {
                            locale.entry(key).is_some()
                                || overrides.is_some_and(|o| o.contains_key(key.as_str()))
                        })
                        .count();
                    (present, keys.len())
                });
                LocaleReport {
                    tag: locale.tag.clone(),
                    namespaces: locale.namespaces.keys().cloned().collect(),
                    entries: locale.namespaces.values().map(|ns| ns.entries.len()).sum(),
                    overrides: overrides.map_or(0, BTreeMap::len),
                    coverage,
                }
            })
            .collect();
        RuntimeReport {
            current: self.current.clone(),
            locales,
            loaded: self.loaded.clone(),
            overrides: self.override_count(),
        }
    }
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
    use crate::pack;
    use crate::render::{Value, params};
    use crate::source::{Tree, sdk_root};

    fn engine(locale: &str) -> Intl {
        let tree = Tree::load(&sdk_root()).unwrap_or_else(|e| panic!("{e}"));
        let mut intl = Intl::from_tree(&tree).unwrap_or_else(|e| panic!("{e}"));
        intl.set_locale(locale).unwrap_or_else(|e| panic!("{e}"));
        intl
    }

    fn namespace(pairs: &[(&str, &str)]) -> Namespace {
        let mut namespace = Namespace::default();
        for (key, source) in pairs {
            namespace.insert(key.to_string(), Entry::Message(source.to_string()));
        }
        namespace
    }

    #[test]
    fn an_override_stands_before_the_locales_own_entry_and_is_reported() {
        let mut intl = engine("ne-Deva-NP");
        let before = intl.render("sdk.reason.appName", &params([]));
        assert!(!before.is_override);
        intl.set_override("ne-Deva-NP", "sdk.reason.appName", "टेइस्ट्रो प्रो")
            .unwrap();
        let after = intl.render("sdk.reason.appName", &params([]));
        assert_eq!(after.text, "टेइस्ट्रो प्रो");
        assert!(after.is_override && !after.is_fallback);
        assert_eq!(after.resolved_from.as_deref(), Some("ne-Deva-NP"));
        // A key the locale never had becomes available, with parameters.
        assert!(!intl.has("sdk.ui.title"));
        intl.set_override("ne-Deva-NP", "sdk.ui.title", "{$name} को कुण्डली")
            .unwrap();
        assert!(intl.has("sdk.ui.title"));
        let title = intl.render("sdk.ui.title", &params([("name", Value::from("सीता"))]));
        assert_eq!(title.text, "सीता को कुण्डली");
        // An override in the fallback locale answers a key the current one
        // lacks, and says it fell back.
        intl.set_override("en-Latn", "sdk.ui.footer", "Computed by Teistro")
            .unwrap();
        let footer = intl.render("sdk.ui.footer", &params([]));
        assert!(footer.is_override && footer.is_fallback);
        // A broken source and an unknown locale are refused at the call.
        assert!(
            intl.set_override("ne-Deva-NP", "sdk.ui.x", "a } b")
                .is_err_and(|e| e.0.contains("does not parse"))
        );
        assert!(intl.set_override("xx-Latn", "sdk.ui.x", "a").is_err());
        assert_eq!(intl.override_count(), 3);
        // Clearing restores the locale's own entry.
        assert!(intl.clear_override("ne-Deva-NP", "sdk.reason.appName"));
        assert!(!intl.clear_override("ne-Deva-NP", "sdk.reason.appName"));
        assert_eq!(intl.render("sdk.reason.appName", &params([])), before);
        intl.clear_overrides(None);
        assert_eq!(intl.override_count(), 0);
        assert!(!intl.has("sdk.ui.title"));
    }

    #[test]
    fn a_pack_loaded_at_runtime_adds_a_namespace_and_replaces_entries() {
        let mut intl = engine("ne-Deva-NP");
        let nepali = intl.locales.get("ne-Deva-NP").unwrap().clone();
        // A new namespace for a known locale, in a pack with metadata.
        let mut ui = nepali.clone();
        ui.namespaces = [(
            String::from("sdk.ui"),
            namespace(&[("title", "शीर्षक"), ("save", "बचत")]),
        )]
        .into_iter()
        .collect();
        let bytes = pack::build(&ui, "sdk.ui").unwrap();
        let loaded = intl.load_pack(&bytes).unwrap();
        assert_eq!(loaded.locale, "ne-Deva-NP");
        assert_eq!(loaded.namespaces, vec![String::from("sdk.ui")]);
        assert_eq!((loaded.entries, loaded.replaced), (2, 0));
        assert_eq!(loaded.sha256.len(), 64);
        assert_eq!(intl.render("sdk.ui.title", &params([])).text, "शीर्षक");
        // A replacement changes what renders: the parse cache forgot it.
        let first = intl.render("sdk.reason.appName", &params([])).text;
        let mut patched = nepali.clone();
        patched.namespaces = [(
            String::from("sdk.reason"),
            namespace(&[("appName", "नयाँ नाम")]),
        )]
        .into_iter()
        .collect();
        let replaced = intl
            .load_pack(&pack::build(&patched, "sdk.reason").unwrap())
            .unwrap();
        assert_eq!((replaced.entries, replaced.replaced), (1, 1));
        let second = intl.render("sdk.reason.appName", &params([])).text;
        assert_ne!(first, second);
        assert_eq!(second, "नयाँ नाम");
        assert_eq!(intl.loaded().len(), 2);
        // A bundle brings a new locale, selectable, falling back to the base.
        let mut marathi = nepali.clone();
        marathi.tag = String::from("mr-Deva-IN");
        marathi.meta.locale = marathi.tag.clone();
        marathi.meta.fallback = vec![BASE_LOCALE.to_string()];
        marathi.namespaces = [(
            String::from("sdk.reason"),
            namespace(&[("appName", "टेइस्ट्रो")]),
        )]
        .into_iter()
        .collect();
        let bundle = pack::build_bundle(&marathi).unwrap();
        let loaded = intl.load_pack(&bundle).unwrap();
        assert_eq!(loaded.locale, "mr-Deva-IN");
        intl.set_locale("mr-Deva-IN").unwrap();
        assert_eq!(
            intl.render("sdk.reason.appName", &params([])).text,
            "टेइस्ट्रो"
        );
        let fallen = intl.render("sdk.reason.welcome", &params([]));
        assert!(fallen.is_fallback);
        assert_eq!(fallen.resolved_from.as_deref(), Some("en-Latn"));
        // A file that is not a pack is refused before anything changes.
        assert!(intl.load_pack(b"TPK1junk").is_err());
        assert_eq!(intl.loaded().len(), 3);
        let report = intl.report();
        assert_eq!(report.current, "mr-Deva-IN");
        assert_eq!(report.loaded.len(), 3);
        let marathi_row = report
            .locales
            .iter()
            .find(|l| l.tag == "mr-Deva-IN")
            .unwrap();
        assert_eq!(marathi_row.entries, 1);
        let base_keys = intl.locales.get(BASE_LOCALE).unwrap().keys().count();
        assert_eq!(
            marathi_row.coverage.map(|(_, total)| total),
            Some(base_keys)
        );
        assert!(report.markdown().contains("| mr-Deva-IN |"));
    }
}
