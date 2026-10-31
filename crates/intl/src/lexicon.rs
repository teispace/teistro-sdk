//! Every catalogue member's names in a locale, romanised forms too, and
//! the reverse: the members a piece of text may name
//! (`docs/03-design/lexicon.md`, A2-3).
//!
//! The reverse matches on a **fold** ([`fold`]), one function applied to
//! the names and to the text, so `Ekadashi`, `ekādaśī`, `Ekadasi` and
//! `एकादशी` meet. A span that may name several members answers every one
//! of them: nothing is guessed.
//!
//! ```
//! use teistro_intl::lexicon::LookupRequest;
//! use teistro_intl::source::Tree;
//! use teistro_intl::{Intl, sdk_root};
//!
//! let intl = Intl::from_tree(&Tree::load(&sdk_root())?)?;
//! let asked = LookupRequest::from_json(r#"{"text": "चन्द्रमा रोहिणीमा छ", "kinds": ["nakshatra"]}"#)?;
//! let found = intl.lookup(&asked)?;
//! assert_eq!(found.matches[0].text, "रोहिणी");
//! assert_eq!(found.matches[0].candidates[0].key, "nakshatra.ROHINI");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, PoisonError};

use serde::{Deserialize, Serialize};
use teistro_core::error::Error;
use teistro_core::key::KeyId;

use crate::source::{ENTITY_NAMESPACE, Entry, LocaleSource};
use crate::translit::devanagari_to_iast;
use crate::{BASE_LOCALE, Intl};

/// The record's name where a binding sends a listing, which a refusal is
/// named under.
const LEXICON: &str = "lexicon";
/// The record's name where a binding sends a text to look up.
const LOOKUP: &str = "lookup";

/// The most words one name spans: *Purva Bhadrapada*, *Shukla Ekadashi*.
const LONGEST_NAME: usize = 4;

/// The forms a locale writes, in the order a listing gives them; any other
/// form the locale adds follows, by name.
const FORMS: [&str; 4] = ["name", "prose", "short", "iast"];

/// The derived form: the name in the Latin letters a person types.
pub const ROMANISED: &str = "romanised";

/// The postpositions Nepali and Hindi fuse to a name (`रोहिणीमा`,
/// `एकादशीको`), and their romanised spellings, tried off a word that names
/// nothing whole; longest first, so `लाई` is tried before `ले`.
const SUFFIXES: [&str; 25] = [
    "देखि",
    "सम्म",
    "हरू",
    "लाई",
    "बाट",
    "में",
    "मा",
    "को",
    "का",
    "की",
    "के",
    "ले",
    "से",
    "ने",
    "dekhi",
    "samma",
    "haru",
    "lai",
    "bata",
    "ma",
    "ko",
    "ka",
    "ki",
    "le",
    "se",
];

/// What a caller asks to be listed: a locale's names for every member, or
/// for some kinds.
///
/// ```
/// use teistro_intl::lexicon::LexiconRequest;
///
/// let asked = LexiconRequest::from_json(r#"{"locale": "ne-Deva-NP", "kinds": ["masa"]}"#)?;
/// assert_eq!(asked.kinds.as_deref(), Some(&[String::from("masa")][..]));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "words-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct LexiconRequest {
    /// The locale, a loaded tag or one that means one (`ne`); the engine's
    /// current one when left out.
    #[serde(default)]
    pub locale: Option<String>,
    /// The kinds to list (`tithi`, `nakshatra`); every kind when left out.
    #[serde(default)]
    pub kinds: Option<Vec<String>>,
}

impl LexiconRequest {
    /// The record a binding sends, as JSON.
    ///
    /// ```
    /// use teistro_intl::lexicon::LexiconRequest;
    ///
    /// let typo = LexiconRequest::from_json(r#"{"kind": ["masa"]}"#);
    /// assert_eq!(typo.unwrap_err().field(), Some("lexicon.kind"));
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record or a key it does not
    /// read, named under `lexicon`.
    pub fn from_json(text: &str) -> Result<LexiconRequest, Error> {
        teistro_core::strict::read(text, LEXICON)
    }
}

/// What a caller asks to be found: the members a text may name.
///
/// ```
/// use teistro_intl::lexicon::LookupRequest;
///
/// let asked = LookupRequest::from_json(r#"{"text": "Shukla Ekadashi"}"#)?;
/// assert!(asked.locale.is_none() && asked.kinds.is_none());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "words-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct LookupRequest {
    /// The text.
    pub text: String,
    /// The locale whose chain's names are searched; every loaded locale's
    /// when left out, so a Hindi word in a Nepali sentence is found.
    #[serde(default)]
    pub locale: Option<String>,
    /// The kinds a match may be of; every kind when left out.
    #[serde(default)]
    pub kinds: Option<Vec<String>>,
}

impl LookupRequest {
    /// The record a binding sends, as JSON.
    ///
    /// ```
    /// use teistro_intl::lexicon::LookupRequest;
    ///
    /// let empty = LookupRequest::from_json("{}");
    /// // A record with no text is named by its root.
    /// assert_eq!(empty.unwrap_err().field(), Some("lookup"));
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record or a key it does not
    /// read, named under `lookup`.
    pub fn from_json(text: &str) -> Result<LookupRequest, Error> {
        teistro_core::strict::read(text, LOOKUP)
    }
}

/// One form of a name: what it is, its text, and the locale that gave it.
///
/// ```
/// use teistro_intl::lexicon::LexiconForm;
///
/// let form = LexiconForm { form: "name".into(), text: "रोहिणी".into(), locale: "ne-Deva-NP".into() };
/// assert_eq!(serde_json::to_value(&form)?["locale"], "ne-Deva-NP");
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "words-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct LexiconForm {
    /// The form: `name`, `prose`, `short`, `iast`, any the locale adds, or
    /// `romanised`, derived.
    pub form: String,
    /// Its text.
    pub text: String,
    /// The locale that gave it: the one asked for, or a fallback.
    pub locale: String,
}

/// A member and its forms.
///
/// ```
/// use teistro_intl::lexicon::LexiconEntry;
///
/// let entry = LexiconEntry { key: "graha.SUN".into(), forms: Vec::new() };
/// assert_eq!(entry.kind(), "graha");
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "words-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct LexiconEntry {
    /// The catalogue key, `kind.MEMBER`.
    pub key: String,
    /// Its forms, each text once, the locale's own first.
    pub forms: Vec<LexiconForm>,
}

impl LexiconEntry {
    /// The kind, the key before its dot.
    ///
    /// ```
    /// use teistro_intl::lexicon::LexiconEntry;
    ///
    /// let entry = LexiconEntry { key: "tithi.PURNIMA".into(), forms: Vec::new() };
    /// assert_eq!(entry.kind(), "tithi");
    /// ```
    #[must_use]
    pub fn kind(&self) -> &str {
        kind_of(&self.key)
    }
}

/// A locale's names for every member asked for.
///
/// ```
/// use teistro_intl::lexicon::LexiconRequest;
/// use teistro_intl::source::Tree;
/// use teistro_intl::{Intl, sdk_root};
///
/// let intl = Intl::from_tree(&Tree::load(&sdk_root())?)?;
/// let months = intl.lexicon(&LexiconRequest::from_json(r#"{"locale": "ne", "kinds": ["masa"]}"#)?)?;
/// assert_eq!(months.locale, "ne-Deva-NP");
/// assert_eq!(months.entries.len(), 12);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "words-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Lexicon {
    /// The loaded locale the request's tag meant.
    pub locale: String,
    /// Every member a locale in the chain names, in the catalogue's order.
    pub entries: Vec<LexiconEntry>,
}

/// A member a span may name, with the forms the span met.
///
/// ```
/// use teistro_intl::lexicon::LookupCandidate;
///
/// let candidate = LookupCandidate { key: "masa.SHRAVANA".into(), via: Vec::new() };
/// assert_eq!(serde_json::to_value(&candidate)?["key"], "masa.SHRAVANA");
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "words-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct LookupCandidate {
    /// The catalogue key.
    pub key: String,
    /// The forms whose fold the span's is.
    pub via: Vec<LexiconForm>,
}

/// One span of the text that may name a member.
///
/// ```
/// use teistro_intl::lexicon::LookupMatch;
///
/// let found = LookupMatch { start: 0, end: 7, text: "Rohini".into(), candidates: Vec::new() };
/// assert_eq!(found.end - found.start, 7);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "words-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct LookupMatch {
    /// Where the span begins, counted in Unicode scalar values.
    pub start: usize,
    /// Where it ends, exclusive, in the same count.
    pub end: usize,
    /// The span's text.
    pub text: String,
    /// Every member it may name, in key order.
    pub candidates: Vec<LookupCandidate>,
}

/// The spans of a text that may name a member.
///
/// ```
/// use teistro_intl::lexicon::LookupRequest;
/// use teistro_intl::source::Tree;
/// use teistro_intl::{Intl, sdk_root};
///
/// let intl = Intl::from_tree(&Tree::load(&sdk_root())?)?;
/// let asked = LookupRequest::from_json(r#"{"text": "Shravana", "kinds": ["masa", "nakshatra"]}"#)?;
/// let found = intl.lookup(&asked)?;
/// let keys: Vec<&str> = found.matches[0].candidates.iter().map(|c| c.key.as_str()).collect();
/// // A month and a star: both, not a guess.
/// assert_eq!(keys, ["masa.SHRAVANA", "nakshatra.SHRAVANA"]);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "words-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Lookup {
    /// The spans, in the text's order.
    pub matches: Vec<LookupMatch>,
}

/// A locale's lexicon with its reverse: built once and kept on the engine.
#[derive(Debug, Default)]
pub(crate) struct Index {
    /// The members and their forms.
    pub(crate) entries: Vec<LexiconEntry>,
    /// A fold to the forms it is: (entry, form) by position.
    folded: BTreeMap<String, Vec<(usize, usize)>>,
}

impl Index {
    /// The entries whose forms fold to `folded`, each with the forms met.
    pub(crate) fn candidates(&self, folded: &str, kinds: &Kinds) -> Vec<LookupCandidate> {
        let Some(found) = self.folded.get(folded) else {
            return Vec::new();
        };
        let mut by_key: BTreeMap<&str, Vec<LexiconForm>> = BTreeMap::new();
        for &(entry, form) in found {
            let Some(entry) = self.entries.get(entry) else {
                continue;
            };
            if !kinds.admits(entry.kind()) {
                continue;
            }
            if let Some(form) = entry.forms.get(form) {
                by_key.entry(&entry.key).or_default().push(form.clone());
            }
        }
        by_key
            .into_iter()
            .map(|(key, via)| LookupCandidate {
                key: key.to_owned(),
                via,
            })
            .collect()
    }

    fn insert(&mut self, entry: LexiconEntry) {
        let at = self.entries.len();
        for (form_at, form) in entry.forms.iter().enumerate() {
            if form.form == "short" {
                continue;
            }
            let folded = fold(&form.text);
            if folded.len() < 2 {
                continue;
            }
            let slot = self.folded.entry(folded).or_default();
            if !slot.iter().any(|&(entry, _)| entry == at) {
                slot.push((at, form_at));
            }
        }
        self.entries.push(entry);
    }
}

/// The kinds a request admits: every one, or those named.
#[derive(Debug)]
pub(crate) struct Kinds(Option<BTreeSet<String>>);

impl Kinds {
    /// Every kind.
    pub(crate) const fn every() -> Kinds {
        Kinds(None)
    }

    /// The kinds named.
    pub(crate) fn only<I: IntoIterator<Item = S>, S: Into<String>>(kinds: I) -> Kinds {
        Kinds(Some(kinds.into_iter().map(Into::into).collect()))
    }

    pub(crate) fn admits(&self, kind: &str) -> bool {
        self.0.as_ref().is_none_or(|kinds| kinds.contains(kind))
    }
}

/// The kind of a catalogue key, the part before its dot.
pub(crate) fn kind_of(key: &str) -> &str {
    key.split_once('.').map_or(key, |(kind, _)| kind)
}

impl Intl {
    /// A locale's names for every member, or for the kinds asked: the
    /// forms the locale's chain gives, and the name in the Latin letters a
    /// person types (`docs/03-design/lexicon.md` §1).
    ///
    /// ```
    /// use teistro_intl::lexicon::LexiconRequest;
    /// use teistro_intl::source::Tree;
    /// use teistro_intl::{Intl, sdk_root};
    ///
    /// let intl = Intl::from_tree(&Tree::load(&sdk_root())?)?;
    /// let asked = LexiconRequest::from_json(r#"{"locale": "ne-Deva-NP", "kinds": ["masa"]}"#)?;
    /// let months = intl.lexicon(&asked)?;
    /// let mangsir = months.entries.iter().find(|e| e.key == "masa.MARGASHIRSHA").ok_or("the month")?;
    /// let texts: Vec<&str> = mangsir.forms.iter().map(|f| f.text.as_str()).collect();
    /// // The name, its IAST and the romanised spelling first, then any
    /// // spelling a vernacular locale adds.
    /// assert_eq!(&texts[..3], ["मार्गशीर्ष", "Mārgaśīrṣa", "Margashirsha"]);
    /// let bad = LexiconRequest::from_json(r#"{"kinds": ["planet"]}"#)?;
    /// assert_eq!(intl.lexicon(&bad).unwrap_err().field(), Some("lexicon.kinds"));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Named under `lexicon`: `UNSUPPORTED` for a locale no loaded one
    /// answers; `INVALID_ARG` for a kind the catalogue's names do not have.
    pub fn lexicon(&self, request: &LexiconRequest) -> Result<Lexicon, Error> {
        self.lexicon_unnamed(request).map_err(|e| e.under(LEXICON))
    }

    fn lexicon_unnamed(&self, request: &LexiconRequest) -> Result<Lexicon, Error> {
        let kinds = self.kinds(request.kinds.as_deref())?;
        let tag = self.meant(request.locale.as_deref())?;
        let index = self.index(Some(&tag));
        Ok(Lexicon {
            locale: tag,
            entries: index
                .entries
                .iter()
                .filter(|entry| kinds.admits(entry.kind()))
                .cloned()
                .collect(),
        })
    }

    /// Every span of a text that may name a member, each with every member
    /// it may name (`docs/03-design/lexicon.md` §1.3).
    ///
    /// ```
    /// use teistro_intl::lexicon::LookupRequest;
    /// use teistro_intl::source::Tree;
    /// use teistro_intl::{Intl, sdk_root};
    ///
    /// let intl = Intl::from_tree(&Tree::load(&sdk_root())?)?;
    /// let asked = LookupRequest::from_json(r#"{"text": "aaja ekadasi ho", "kinds": ["tithi"]}"#)?;
    /// let found = intl.lookup(&asked)?;
    /// let keys: Vec<&str> = found.matches[0].candidates.iter().map(|c| c.key.as_str()).collect();
    /// assert_eq!(keys, ["tithi.KRISHNA_EKADASHI", "tithi.SHUKLA_EKADASHI"]);
    /// assert_eq!((found.matches[0].start, found.matches[0].end), (5, 12));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Named under `lookup`: `UNSUPPORTED` for a locale no loaded one
    /// answers; `INVALID_ARG` for a kind the catalogue's names do not have.
    pub fn lookup(&self, request: &LookupRequest) -> Result<Lookup, Error> {
        self.lookup_unnamed(request).map_err(|e| e.under(LOOKUP))
    }

    fn lookup_unnamed(&self, request: &LookupRequest) -> Result<Lookup, Error> {
        let kinds = self.kinds(request.kinds.as_deref())?;
        let tag = match request.locale.as_deref() {
            Some(tag) => Some(self.meant(Some(tag))?),
            None => None,
        };
        let index = self.index(tag.as_deref());
        Ok(Lookup {
            matches: matches(&index, &request.text, &kinds),
        })
    }

    /// The loaded locale a tag means, or the current one.
    pub(crate) fn meant(&self, tag: Option<&str>) -> Result<String, Error> {
        let Some(tag) = tag else {
            return Ok(self.locale().to_owned());
        };
        self.lookup_locale(tag).map(str::to_owned).map_err(|why| {
            let known: Vec<&str> = self.locales.keys().map(String::as_str).collect();
            Error::unsupported(why.to_string())
                .with_field("locale")
                .with_hint(format!("the locales are {}", known.join(", ")))
        })
    }

    /// The kinds a request names, each one the catalogue's names have.
    pub(crate) fn kinds(&self, asked: Option<&[String]>) -> Result<Kinds, Error> {
        let Some(asked) = asked else {
            return Ok(Kinds::every());
        };
        let known: BTreeSet<&str> = self
            .locales
            .get(BASE_LOCALE)
            .and_then(|base| base.namespaces.get(ENTITY_NAMESPACE))
            .map(|entities| entities.entries.keys().map(|key| kind_of(key)).collect())
            .unwrap_or_default();
        for (at, kind) in asked.iter().enumerate() {
            if !known.contains(kind.as_str()) {
                return Err(Error::invalid_arg(format!("no kind `{kind}` is named"))
                    .with_field(format!("kinds[{at}]"))
                    .with_hint(format!(
                        "the kinds are {}",
                        known.iter().copied().collect::<Vec<_>>().join(", ")
                    ))
                    .with_field("kinds"));
            }
        }
        Ok(Kinds::only(asked.iter().cloned()))
    }

    /// The index for a locale's chain, or for every locale: built the
    /// first time it is asked for, then kept until a pack or an override
    /// changes what the engine holds.
    pub(crate) fn index(&self, tag: Option<&str>) -> Arc<Index> {
        let slot = tag.unwrap_or("*").to_owned();
        if let Some(index) = self
            .lexicons
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&slot)
        {
            return Arc::clone(index);
        }
        let built = Arc::new(self.build(tag));
        self.lexicons
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .entry(slot)
            .or_insert(built)
            .clone()
    }

    /// The catalogue's keys, in the base locale's order: every member a
    /// name can exist for.
    /// Every key the base locale names, in the catalogue's own order
    /// (kind, then member): what a source tree and a pack agree on, where
    /// a pack keeps no source order. A key the catalogue does not resolve
    /// follows, in the base locale's order.
    fn catalogue_keys(&self) -> Vec<&str> {
        let mut keys: Vec<&str> = self
            .locales
            .get(BASE_LOCALE)
            .and_then(|base| base.namespaces.get(ENTITY_NAMESPACE))
            .map(|entities| entities.order.iter().map(String::as_str).collect())
            .unwrap_or_default();
        keys.sort_by_key(|key| teistro_core::key::resolve(key).map_or(u32::MAX, KeyId::bits));
        keys
    }

    fn build(&self, tag: Option<&str>) -> Index {
        let sources: Vec<&LocaleSource> = match tag {
            Some(tag) => self.chain_from(tag),
            None => self.locales().collect(),
        };
        let mut index = Index::default();
        for key in self.catalogue_keys() {
            let mut forms: Vec<LexiconForm> = Vec::new();
            // One locale's forms for a chain (the first that names the
            // member); every locale's for the whole engine.
            for source in &sources {
                let Some(Entry::Entity(entity)) = source
                    .namespaces
                    .get(ENTITY_NAMESPACE)
                    .and_then(|entities| entities.entries.get(key))
                else {
                    continue;
                };
                let vernacular = source.tag.contains("-Deva")
                    && source.meta.term_style.as_deref() == Some("vernacular");
                let named = |form: &str, text: &str| LexiconForm {
                    form: form.to_owned(),
                    text: text.to_owned(),
                    locale: source.tag.clone(),
                };
                let ordered = FORMS.iter().copied().chain(
                    entity
                        .forms
                        .keys()
                        .map(String::as_str)
                        .filter(|form| !FORMS.contains(form)),
                );
                for form in ordered {
                    if let Some(text) = entity.forms.get(form) {
                        push(&mut forms, named(form, text));
                    }
                }
                let iast = entity.forms.get("iast").cloned().or_else(|| {
                    let name = entity.forms.get("name")?;
                    is_devanagari(name).then(|| devanagari_to_iast(name))
                });
                if let Some(iast) = iast {
                    push(&mut forms, named(ROMANISED, &romanised(&iast)));
                    if vernacular {
                        push(
                            &mut forms,
                            named(ROMANISED, &romanised(&schwa_dropped(&iast))),
                        );
                    }
                }
                if tag.is_some() {
                    break;
                }
            }
            if !forms.is_empty() {
                index.insert(LexiconEntry {
                    key: key.to_owned(),
                    forms,
                });
            }
        }
        index
    }
}

/// Adds a form unless its text is already there.
fn push(forms: &mut Vec<LexiconForm>, form: LexiconForm) {
    if !form.text.is_empty() && !forms.iter().any(|known| known.text == form.text) {
        forms.push(form);
    }
}

/// Whether a text holds a Devanagari letter.
fn is_devanagari(text: &str) -> bool {
    text.chars().any(|c| ('\u{900}'..='\u{97f}').contains(&c))
}

/// Whether a character belongs to a word: a letter, a Devanagari sign, or
/// a combining mark typed after a Latin letter.
fn in_word(c: char) -> bool {
    c.is_alphabetic()
        || ('\u{900}'..='\u{963}').contains(&c)
        || ('\u{971}'..='\u{97f}').contains(&c)
        || ('\u{300}'..='\u{36f}').contains(&c)
        // The joiners Nepali writes inside a word (`र्‍`, a half form).
        || matches!(c, '\u{200c}' | '\u{200d}')
}

/// One word of a text: where it begins and ends, in scalar values, and its
/// text.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Word<'t> {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) text: &'t str,
}

/// The words of a text, in order.
pub(crate) fn words(text: &str) -> Vec<Word<'_>> {
    let mut out = Vec::new();
    let mut open: Option<(usize, usize)> = None;
    let mut count = 0;
    for (byte, c) in text.char_indices() {
        match (in_word(c), open) {
            (true, None) => open = Some((count, byte)),
            (false, Some((start, from))) => {
                out.push(Word {
                    start,
                    end: count,
                    text: text.get(from..byte).unwrap_or_default(),
                });
                open = None;
            }
            _ => {}
        }
        count += 1;
    }
    if let Some((start, from)) = open {
        out.push(Word {
            start,
            end: count,
            text: text.get(from..).unwrap_or_default(),
        });
    }
    out
}

/// Every span of `text` the index names, the longest at each place.
pub(crate) fn matches(index: &Index, text: &str, kinds: &Kinds) -> Vec<LookupMatch> {
    let words = words(text);
    let scalars: Vec<char> = text.chars().collect();
    let span = |start: usize, end: usize| -> String {
        scalars
            .get(start..end)
            .map(|chars| chars.iter().collect())
            .unwrap_or_default()
    };
    let mut out = Vec::new();
    let mut at = 0;
    while let Some(first) = words.get(at) {
        let mut found: Option<(usize, usize, Vec<LookupCandidate>)> = None;
        for count in (1..=LONGEST_NAME.min(words.len() - at)).rev() {
            let Some(run) = words.get(at..at + count) else {
                continue;
            };
            let joined: String = run.iter().map(|word| word.text).collect();
            let candidates = index.candidates(&fold(&joined), kinds);
            if !candidates.is_empty() {
                let end = run.last().map_or(first.end, |word| word.end);
                found = Some((count, end, candidates));
                break;
            }
        }
        if found.is_none() {
            found = SUFFIXES.iter().find_map(|suffix| {
                let stem = first.text.strip_suffix(suffix)?;
                let candidates = index.candidates(&fold(stem), kinds);
                (!stem.is_empty() && !candidates.is_empty())
                    .then(|| (1, first.start + stem.chars().count(), candidates))
            });
        }
        match found {
            Some((count, end, candidates)) => {
                out.push(LookupMatch {
                    start: first.start,
                    end,
                    text: span(first.start, end),
                    candidates,
                });
                at += count;
            }
            None => at += 1,
        }
    }
    out
}

/// IAST in the Latin letters a person types: ā ī ū as a i u, ś and ṣ as
/// *sh*, c as *ch*, ṛ as *ri*, every nasal as *n*, ṭ ḍ as t d, ḥ as h; a
/// capital stays a capital (`docs/03-design/lexicon.md` §1.2).
///
/// ```
/// use teistro_intl::lexicon::romanised;
///
/// assert_eq!(romanised("Ekādaśī"), "Ekadashi");
/// assert_eq!(romanised("Kṛṣṇa Caturdaśī"), "Krishna Chaturdashi");
/// assert_eq!(romanised("maṃsira"), "mansira");
/// ```
#[must_use]
pub fn romanised(iast: &str) -> String {
    let mut out = String::with_capacity(iast.len());
    for c in iast.chars() {
        let lower: String = c.to_lowercase().collect();
        let plain = match lower.as_str() {
            "ā" => "a",
            "ī" => "i",
            "ū" => "u",
            "ē" => "e",
            "ō" => "o",
            "ṛ" | "ṝ" => "ri",
            "ḷ" | "ḹ" => "li",
            "ṅ" | "ñ" | "ṇ" | "ṃ" | "ṁ" => "n",
            "ś" | "ṣ" => "sh",
            "c" => "ch",
            "ṭ" => "t",
            "ḍ" => "d",
            "ḥ" => "h",
            "\u{310}" | "\u{325}" | "\u{304}" => "",
            other => other,
        };
        if c.is_uppercase() {
            let mut letters = plain.chars();
            if let Some(head) = letters.next() {
                out.extend(head.to_uppercase());
                out.push_str(letters.as_str());
            }
        } else {
            out.push_str(plain);
        }
    }
    out
}

/// IAST with each word's final inherent `a` dropped where Nepali and Hindi
/// do not say it: after one consonant (an aspirate counted as one) that
/// follows a vowel. `maṃsira` is *maṃsir*; `sūrya`, after two, keeps it.
fn schwa_dropped(iast: &str) -> String {
    iast.split(' ')
        .map(|word| {
            let letters: Vec<char> = word.chars().collect();
            let Some((&'a', before)) = letters.split_last() else {
                return word.to_owned();
            };
            let consonant = |c: &char| c.is_alphabetic() && !"aāiīuūṛṝeēoōḷ".contains(*c);
            let cluster = match before {
                [.., k, 'h'] if "kgcjṭḍtdpb".contains(*k) => 2,
                [.., c] if consonant(c) => 1,
                _ => return word.to_owned(),
            };
            let vowel_before = before
                .len()
                .checked_sub(cluster + 1)
                .and_then(|at| before.get(at))
                .is_some_and(|c| !consonant(c));
            if vowel_before {
                before.iter().collect()
            } else {
                word.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The fold a name and a text are compared by: Devanagari read as IAST,
/// diacritics dropped, and the spellings a typist varies made one
/// (`docs/03-design/lexicon.md` §1.3).
///
/// ```
/// use teistro_intl::lexicon::fold;
///
/// for spelt in ["Ekadashi", "ekādaśī", "Ekadasi", "एकादशी", "EKAADASHI"] {
///     assert_eq!(fold(spelt), "ekadasi", "{spelt}");
/// }
/// for spelt in ["Baishakh", "Baisakh", "Vaishakha", "बैशाख", "वैशाख"] {
///     assert_eq!(fold(spelt), "besak", "{spelt}");
/// }
/// assert_eq!(fold("Jyeshtha"), fold("जेष्ठ"));
/// assert_eq!(fold("Falgun"), fold("Phalgun"));
/// ```
#[must_use]
pub fn fold(text: &str) -> String {
    let latin = if is_devanagari(text) {
        // The chandrabindu's nasal is not written in Latin by a typist.
        devanagari_to_iast(&text.replace('\u{901}', ""))
    } else {
        text.to_owned()
    };
    // Letters without their diacritics.
    let mut plain: Vec<u8> = Vec::with_capacity(latin.len());
    for c in latin.chars().flat_map(char::to_lowercase) {
        let letters: &[u8] = match c {
            'a'..='z' => {
                let mut one = [0_u8; 4];
                plain.extend_from_slice(c.encode_utf8(&mut one).as_bytes());
                continue;
            }
            'ā' | 'á' | 'à' | 'â' | 'ä' | 'ã' => b"a",
            'ī' | 'í' | 'ì' | 'î' | 'ï' => b"i",
            'ū' | 'ú' | 'ù' | 'û' | 'ü' => b"u",
            'ē' | 'é' | 'è' | 'ê' | 'ë' => b"e",
            'ō' | 'ó' | 'ò' | 'ô' | 'ö' => b"o",
            'ṛ' | 'ṝ' => b"ri",
            'ḷ' | 'ḹ' | 'ḻ' => b"l",
            'ṅ' | 'ñ' | 'ṇ' | 'ṃ' | 'ṁ' | 'ṉ' => b"n",
            'ś' | 'ṣ' | 'ş' => b"s",
            'ṭ' => b"t",
            'ḍ' => b"d",
            'ṟ' => b"r",
            'ẏ' => b"y",
            'ġ' => b"g",
            // ISO 15919's r̥: the ring below makes the r a vowel.
            '\u{325}' if plain.last() == Some(&b'r') => b"i",
            _ => b"",
        };
        plain.extend_from_slice(letters);
    }
    // The letters a typist swaps.
    let mut swapped: Vec<u8> = Vec::with_capacity(plain.len());
    for letter in plain {
        match letter {
            b'w' | b'v' => swapped.push(b'b'),
            b'f' => swapped.push(b'p'),
            b'z' => swapped.push(b'j'),
            b'q' => swapped.push(b'k'),
            b'x' => swapped.extend_from_slice(b"ks"),
            other => swapped.push(other),
        }
    }
    let vowel = |letter: u8| matches!(letter, b'a' | b'e' | b'i' | b'o' | b'u');
    let consonant = |letter: Option<&u8>| letter.is_some_and(|&l| !vowel(l));
    let mut out: Vec<u8> = Vec::with_capacity(swapped.len());
    let mut at = 0;
    while let Some(&letter) = swapped.get(at) {
        let next = swapped.get(at + 1).copied();
        at += 1;
        match (letter, next) {
            // An aspirate, or a y glide after a consonant: the typist's
            // choice, dropped.
            (b'h' | b'y', _) if consonant(out.last()) => {}
            (b'n' | b'm', Some(n)) if !vowel(n) && n != b'y' => out.push(b'n'),
            (b'a', Some(b'i')) | (b'e', Some(b'e')) => {
                out.push(if letter == b'a' { b'e' } else { b'i' });
                at += 1;
            }
            (b'a' | b'o', Some(b'u')) | (b'o', Some(b'o')) => {
                out.push(if letter == b'o' && next == Some(b'o') {
                    b'u'
                } else {
                    b'o'
                });
                at += 1;
            }
            _ => out.push(letter),
        }
    }
    out.dedup();
    if out.len() > 2 && out.last() == Some(&b'a') && consonant(out.iter().rev().nth(1)) {
        out.pop();
    }
    String::from_utf8(out).unwrap_or_default()
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::{fold, romanised, schwa_dropped, words};

    #[test]
    fn a_name_folds_the_same_however_it_is_spelt() {
        for (spellings, folded) in [
            (&["Rohini", "rohiṇī", "रोहिणी", "Rohinee"][..], "rohini"),
            (&["Shrawan", "Shravan", "श्रावण", "Srawan"][..], "sraban"),
            (&["Saun", "साउन"][..], "son"),
            (&["Krishna", "kṛṣṇa", "कृष्ण", "kr̥ṣṇa"][..], "krisn"),
            (&["Poush", "पौष"][..], "pos"),
            (&["Asar", "असार"][..], "asar"),
        ] {
            for spelt in spellings {
                assert_eq!(fold(spelt), folded, "{spelt}");
            }
        }
        // Kartik and Kattik are two words, not two spellings: the fold
        // does not make every word one.
        assert_ne!(fold("Kartik"), fold("Kattik"));
        assert_eq!(fold("Kattik"), fold("कात्तिक"));
    }

    #[test]
    fn a_short_word_keeps_its_final_a() {
        assert_eq!(fold("ma"), "ma");
        assert_eq!(fold(""), "");
        assert_eq!(fold("–"), "");
    }

    #[test]
    fn the_romanised_form_is_the_spelling_a_person_types() {
        assert_eq!(romanised("Śrāvaṇa"), "Shravana");
        assert_eq!(romanised("Ṛṣi"), "Rishi");
        assert_eq!(schwa_dropped("maṃsira"), "maṃsir");
        assert_eq!(schwa_dropped("sūrya"), "sūrya");
        assert_eq!(schwa_dropped("bhādrapada"), "bhādrapad");
        assert_eq!(schwa_dropped("māgha"), "māgh");
        assert_eq!(schwa_dropped("pūrva phālguna"), "pūrva phālgun");
        assert_eq!(schwa_dropped("a"), "a");
    }

    #[test]
    fn words_are_counted_in_scalar_values() {
        let found = words("आज, Ekadashi!");
        let spans: Vec<(usize, usize, &str)> =
            found.iter().map(|w| (w.start, w.end, w.text)).collect();
        assert_eq!(spans, [(0, 2, "आज"), (4, 12, "Ekadashi")]);
    }
}
