//! The locales an engine holds, each decoded the first time it is read.
//!
//! An engine over the SDK's embedded bundles once decoded all of them
//! before its first render, every message and entity copied into owned
//! text, though an application reads one locale and its fallbacks. A
//! bundle is now verified when the locales are made, so a bad one is
//! refused there and then, and decoded the first time an engine reads
//! it: the engine's choice of locale reads tags only, and a render
//! decodes its locale's chain. Engines built over clones of one
//! [`Locales`] share each decoded locale and its plural rules.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, OnceLock};

use crate::pack::{Bundle, PackError};
use crate::render::{Plurals, plurals_for};
use crate::source::{LocaleSource, Meta};

/// The locales an engine is built over ([`crate::Intl::over`]): sources
/// decoded already, or bundles decoded the first time an engine reads
/// them.
///
/// Engines built over clones of one `Locales` share it: a locale any of
/// them reads is decoded once for all of them, and its plural rules are
/// built once; a locale a runtime pack changes is copied for that engine
/// alone. Listing the locales or choosing one reads tags and decodes
/// nothing.
///
/// ```
/// use teistro_intl::locales::Locales;
/// use teistro_intl::source::Tree;
/// use teistro_intl::{Intl, Params, pack, sdk_root};
///
/// let tree = Tree::load(&sdk_root())?;
/// // Bundles a program embeds, as `include_bytes!` gives them.
/// let bundles: Vec<&'static [u8]> = tree
///     .locales
///     .values()
///     .map(|locale| &*pack::build_bundle(locale).expect("a bundle").leak())
///     .collect();
/// let locales = Locales::from_bundles(&bundles)?;
/// let mut intl = Intl::over(locales.clone())?;
/// intl.set_locale("ne-Deva-NP")?;
/// assert_eq!(locales.decoded().count(), 0, "choosing a locale decodes nothing");
/// intl.render("sdk.reason.appName", &Params::new());
/// // The locale rendered from and its fallback, and nothing else.
/// assert_eq!(locales.decoded().collect::<Vec<_>>(), ["en-Latn", "ne-Deva-NP"]);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Default)]
pub struct Locales {
    held: BTreeMap<String, Held>,
}

/// One locale and its plural rules, each made the first time it is read.
/// A clone shares both.
#[derive(Clone)]
struct Held {
    source: Source,
    /// `None` when ICU4X built no rules for the tag, which the engine
    /// reads as a locale without plural categories.
    plurals: Arc<OnceLock<Option<Plurals>>>,
}

#[derive(Clone)]
enum Source {
    Decoded(Arc<LocaleSource>),
    Embedded(Arc<Embedded>),
}

/// A verified bundle, decoded the first time it is read.
struct Embedded {
    bundle: Bundle<'static>,
    meta: Meta,
    decoded: OnceLock<Arc<LocaleSource>>,
}

impl Embedded {
    fn decoded(&self) -> &Arc<LocaleSource> {
        self.decoded.get_or_init(|| {
            Arc::new(LocaleSource {
                tag: self.bundle.locale().to_string(),
                meta: self.meta.clone(),
                namespaces: self.bundle.namespaces().collect(),
            })
        })
    }
}

impl Held {
    fn of(source: Source, plurals: OnceLock<Option<Plurals>>) -> Held {
        Held {
            source,
            plurals: Arc::new(plurals),
        }
    }

    fn shared(&self) -> &Arc<LocaleSource> {
        match &self.source {
            Source::Decoded(source) => source,
            Source::Embedded(embedded) => embedded.decoded(),
        }
    }

    fn is_decoded(&self) -> bool {
        match &self.source {
            Source::Decoded(_) => true,
            Source::Embedded(embedded) => embedded.decoded.get().is_some(),
        }
    }

    /// The locale to change, copied for this holder alone when another
    /// shares it, and decoded first when it has not been: the holder
    /// keeps the decoded copy as its own, so the bundle's stands for the
    /// others. Always some; an option only because the borrow checker
    /// cannot see that the holder was made decoded just above.
    fn make_mut(&mut self) -> Option<&mut LocaleSource> {
        if let Source::Embedded(embedded) = &self.source {
            self.source = Source::Decoded(Arc::clone(embedded.decoded()));
        }
        match &mut self.source {
            Source::Decoded(source) => Some(Arc::make_mut(source)),
            Source::Embedded(_) => None,
        }
    }
}

impl Locales {
    /// Locales over bundles, as a program embeds them: each verified now
    /// (magic, version, checksums, key order and metadata, as
    /// [`Bundle::parse`] and [`Bundle::meta`] check them) and decoded the
    /// first time an engine reads it.
    ///
    /// ```
    /// use teistro_intl::locales::Locales;
    /// use teistro_intl::source::Tree;
    /// use teistro_intl::{pack, sdk_root};
    ///
    /// let tree = Tree::load(&sdk_root())?;
    /// let nepali: &'static [u8] = pack::build_bundle(&tree.locales["ne-Deva-NP"])?.leak();
    /// let locales = Locales::from_bundles(&[nepali])?;
    /// assert_eq!(locales.tags().collect::<Vec<_>>(), ["ne-Deva-NP"]);
    /// // A bundle is checked now, not when it is first read.
    /// assert!(Locales::from_bundles(&[b"not a bundle".as_slice()]).is_err());
    /// // A locale is one bundle.
    /// assert!(Locales::from_bundles(&[nepali, nepali]).is_err());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Bytes that are not a well-formed bundle, metadata that does not
    /// parse, or two bundles of one locale.
    pub fn from_bundles(bundles: &[&'static [u8]]) -> Result<Locales, PackError> {
        let mut held = BTreeMap::new();
        for bytes in bundles {
            let bundle = Bundle::parse(bytes)?;
            let meta = bundle.meta()?;
            let tag = bundle.locale().to_string();
            let embedded = Embedded {
                bundle,
                meta,
                decoded: OnceLock::new(),
            };
            let embedded = Held::of(Source::Embedded(Arc::new(embedded)), OnceLock::new());
            if held.insert(tag.clone(), embedded).is_some() {
                return Err(PackError(format!("two bundles of the locale {tag}")));
            }
        }
        Ok(Locales { held })
    }

    /// The locales' tags, sorted. Reads no locale.
    ///
    /// ```
    /// use teistro_intl::locales::Locales;
    /// use teistro_intl::source::Tree;
    /// use teistro_intl::sdk_root;
    ///
    /// let locales = Locales::from(Tree::load(&sdk_root())?.locales);
    /// assert_eq!(locales.tags().count(), 5);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn tags(&self) -> impl Iterator<Item = &str> {
        self.held.keys().map(String::as_str)
    }

    /// The locales decoded so far, sorted: every one given as a source,
    /// and every bundle an engine has read.
    ///
    /// ```
    /// use teistro_intl::locales::Locales;
    /// use teistro_intl::source::Tree;
    /// use teistro_intl::sdk_root;
    ///
    /// // Sources are decoded already.
    /// let locales = Locales::from(Tree::load(&sdk_root())?.locales);
    /// assert_eq!(locales.decoded().count(), 5);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn decoded(&self) -> impl Iterator<Item = &str> {
        self.held
            .iter()
            .filter(|(_, held)| held.is_decoded())
            .map(|(tag, _)| tag.as_str())
    }

    pub(crate) fn contains(&self, tag: &str) -> bool {
        self.held.contains_key(tag)
    }

    /// A locale, decoded now if it has not been.
    pub(crate) fn source(&self, tag: &str) -> Option<&LocaleSource> {
        self.shared(tag).map(Arc::as_ref)
    }

    /// A locale as it is shared, decoded now if it has not been.
    pub(crate) fn shared(&self, tag: &str) -> Option<&Arc<LocaleSource>> {
        self.held.get(tag).map(Held::shared)
    }

    /// Every locale, sorted, each decoded now if it has not been.
    pub(crate) fn sources(&self) -> impl Iterator<Item = &LocaleSource> {
        self.held.values().map(|held| held.shared().as_ref())
    }

    /// A locale to change, copied for this engine alone (see
    /// [`Held::make_mut`]).
    pub(crate) fn source_mut(&mut self, tag: &str) -> Option<&mut LocaleSource> {
        self.held.get_mut(tag).and_then(Held::make_mut)
    }

    /// Adds a locale with the plural rules already built for it.
    pub(crate) fn add(&mut self, source: LocaleSource, plurals: Plurals) {
        let tag = source.tag.clone();
        let held = Held::of(
            Source::Decoded(Arc::new(source)),
            OnceLock::from(Some(plurals)),
        );
        self.held.insert(tag, held);
    }

    /// A locale's plural rules, built now if they have not been.
    pub(crate) fn plurals(&self, tag: &str) -> Option<&Plurals> {
        self.held
            .get(tag)?
            .plurals
            .get_or_init(|| plurals_for(tag).ok())
            .as_ref()
    }
}

impl From<BTreeMap<String, Arc<LocaleSource>>> for Locales {
    /// Locales decoded already, shared as they are.
    ///
    /// ```
    /// use std::sync::Arc;
    /// use teistro_intl::locales::Locales;
    /// use teistro_intl::source::Tree;
    /// use teistro_intl::sdk_root;
    ///
    /// let shared = Tree::load(&sdk_root())?.locales.into_iter().map(|(tag, locale)| (tag, Arc::new(locale)));
    /// assert_eq!(Locales::from(shared.collect::<std::collections::BTreeMap<_, _>>()).tags().count(), 5);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn from(sources: BTreeMap<String, Arc<LocaleSource>>) -> Locales {
        let held = sources
            .into_iter()
            .map(|(tag, source)| (tag, Held::of(Source::Decoded(source), OnceLock::new())))
            .collect();
        Locales { held }
    }
}

impl From<BTreeMap<String, LocaleSource>> for Locales {
    /// Locales decoded already, from a source tree or packs.
    ///
    /// ```
    /// use teistro_intl::locales::Locales;
    /// use teistro_intl::source::Tree;
    /// use teistro_intl::sdk_root;
    ///
    /// let locales = Locales::from(Tree::load(&sdk_root())?.locales);
    /// assert!(locales.tags().any(|tag| tag == "sa-Latn"));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn from(sources: BTreeMap<String, LocaleSource>) -> Locales {
        sources
            .into_iter()
            .map(|(tag, source)| (tag, Arc::new(source)))
            .collect::<BTreeMap<_, _>>()
            .into()
    }
}

impl fmt::Debug for Locales {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Locales")
            .field("tags", &self.tags().collect::<Vec<_>>())
            .field("decoded", &self.decoded().collect::<Vec<_>>())
            .finish()
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use super::*;
    use crate::pack::{self, locales_from_packs};
    use crate::render::{Intl, Params, Value, params};
    use crate::source::{Tree, sdk_root};

    /// The SDK's locales as a program embeds them, and the sources.
    fn embedded() -> (Vec<&'static [u8]>, Tree) {
        let tree = Tree::load(&sdk_root()).unwrap_or_else(|e| panic!("{e}"));
        let bundles = tree
            .locales
            .values()
            .map(|locale| &*pack::build_bundle(locale).unwrap().leak())
            .collect();
        (bundles, tree)
    }

    fn decoded(locales: &Locales) -> Vec<&str> {
        locales.decoded().collect()
    }

    fn plurals_built(locales: &Locales) -> Vec<&str> {
        locales
            .held
            .iter()
            .filter(|(_, held)| held.plurals.get().is_some())
            .map(|(tag, _)| tag.as_str())
            .collect()
    }

    #[test]
    fn an_engine_decodes_the_locales_it_reads_and_no_others() {
        let (bundles, _) = embedded();
        let locales = Locales::from_bundles(&bundles).unwrap();
        let mut intl = Intl::over(locales.clone()).unwrap();
        let tags: Vec<&str> = intl.locale_tags().collect();
        assert_eq!(tags.len(), 5);
        assert_eq!(intl.lookup_locale("ne").unwrap(), "ne-Deva-NP");
        intl.set_locale("ne-Deva-NP").unwrap();
        assert_eq!(
            decoded(&locales),
            Vec::<&str>::new(),
            "listing and choosing read tags"
        );
        let said = intl.render("sdk.reason.appName", &Params::new());
        assert_eq!(said.resolved_from.as_deref(), Some("ne-Deva-NP"));
        assert_eq!(decoded(&locales), ["en-Latn", "ne-Deva-NP"]);
        assert_eq!(
            plurals_built(&locales),
            Vec::<&str>::new(),
            "no plural was selected"
        );

        // Another locale is decoded when it is first read.
        intl.set_locale("sa-Deva").unwrap();
        assert_eq!(decoded(&locales), ["en-Latn", "ne-Deva-NP"]);
        assert_ne!(intl.render("sdk.reason.appName", &Params::new()).text, "");
        assert_eq!(decoded(&locales), ["en-Latn", "ne-Deva-NP", "sa-Deva"]);

        // A plural builds the rules of its locale alone.
        intl.set_locale("en-Latn").unwrap();
        let ordinal = intl.render_source(
            ".input {$n :integer select=ordinal} .match $n one {{{$n}st}} * {{{$n}th}}",
            &params([("n", Value::Int(21))]),
        );
        assert_eq!(ordinal.unwrap().text, "21st");
        assert_eq!(plurals_built(&locales), ["en-Latn"]);
    }

    #[test]
    fn engines_over_one_set_share_what_any_of_them_decoded() {
        let (bundles, _) = embedded();
        let locales = Locales::from_bundles(&bundles).unwrap();
        let mut first = Intl::over(locales.clone()).unwrap();
        first.set_locale("hi-Deva-IN").unwrap();
        let mut fork = first.clone();
        assert_ne!(fork.render("sdk.reason.appName", &Params::new()).text, "");
        assert_eq!(decoded(&locales), ["en-Latn", "hi-Deva-IN"]);
        let shared = Arc::as_ptr(locales.shared("hi-Deva-IN").unwrap());
        let again = Intl::over(locales.clone()).unwrap();
        for engine in [&first, &fork, &again] {
            assert_eq!(
                Arc::as_ptr(engine.locales.shared("hi-Deva-IN").unwrap()),
                shared,
                "decoded once, for every engine over the set"
            );
        }
        fork.set_locale("sa-Latn").unwrap();
        assert_ne!(fork.render("sdk.reason.appName", &Params::new()).text, "");
        assert_eq!(decoded(&first.locales), decoded(&locales));
        assert_eq!(decoded(&locales), ["en-Latn", "hi-Deva-IN", "sa-Latn"]);
    }

    #[test]
    fn every_bundle_decodes_as_the_packs_reader_decodes_it() {
        let (bundles, tree) = embedded();
        let locales = Locales::from_bundles(&bundles).unwrap();
        let read = locales_from_packs(&bundles).unwrap();
        let sources: Vec<&LocaleSource> = locales.sources().collect();
        assert_eq!(sources.len(), tree.locales.len());
        for source in sources {
            assert_eq!(source, &read[&source.tag], "{}", source.tag);
        }
        assert_eq!(decoded(&locales).len(), tree.locales.len());
    }

    #[test]
    fn a_bad_bundle_is_refused_when_the_locales_are_made() {
        let (bundles, _) = embedded();
        let mut broken = bundles[2].to_vec();
        let last = broken.len() - 1;
        broken[last] ^= 1;
        let broken: &'static [u8] = broken.leak();
        let refused = Locales::from_bundles(&[bundles[0], broken]).unwrap_err();
        assert!(refused.0.contains("checksum"), "{refused}");
        let twice = Locales::from_bundles(&[bundles[1], bundles[1]]).unwrap_err();
        assert!(twice.0.contains("two bundles"), "{twice}");
    }

    #[test]
    fn a_pack_loaded_over_an_embedded_locale_copies_it_for_that_engine() {
        let (bundles, tree) = embedded();
        let locales = Locales::from_bundles(&bundles).unwrap();
        let open = || {
            let mut intl = Intl::over(locales.clone()).unwrap();
            intl.set_locale("ne-Deva-NP").unwrap();
            intl
        };
        let (mut changed, other) = (open(), open());
        let said = |intl: &Intl| intl.render("sdk.reason.appName", &Params::new()).text;
        let mut patched = tree.locales["ne-Deva-NP"].clone();
        patched.namespaces.retain(|name, _| name == "sdk.reason");
        let reason = patched.namespaces.get_mut("sdk.reason").unwrap();
        reason.insert(
            String::from("appName"),
            crate::source::Entry::Message(String::from("मेरो")),
        );
        changed
            .load_pack(&pack::build(&patched, "sdk.reason").unwrap())
            .unwrap();
        assert_eq!(said(&changed), "मेरो");
        assert_ne!(
            said(&other),
            "मेरो",
            "another engine keeps the build's entry"
        );
        assert_ne!(said(&open()), "मेरो", "and so does one opened after");
        assert!(
            Arc::ptr_eq(
                other.locales.shared("ne-Deva-NP").unwrap(),
                locales.shared("ne-Deva-NP").unwrap()
            ),
            "the engines that changed nothing still share the decoding"
        );
        // The changed engine keeps every other entry of the locale.
        let entity = changed.entity_from("ne-Deva-NP", "graha.SUN").unwrap();
        assert_eq!(entity.name(), "सूर्य");
    }

    #[test]
    fn every_shipped_locale_has_plural_rules() {
        let (_, tree) = embedded();
        for tag in tree.locales.keys() {
            assert!(plurals_for(tag).is_ok(), "{tag}");
        }
    }
}
