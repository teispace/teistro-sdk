//! A locale derived from another by transliteration: `sa-Latn` from
//! `sa-Deva`, so a Latin-script reader of Sanskrit terms gets every
//! entity, and the calendar's weekdays, months and dates, without anyone
//! writing them twice
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

use crate::source::{ENTITY_NAMESPACE, Entity, Entry, LocaleSource, META_FILE, Namespace, Tree};
use crate::translit::{Script, transliterate};

/// The file a derived locale keeps its hand corrections in.
///
/// ```
/// use teistro_intl::derive::OVERRIDES_FILE;
/// use teistro_intl::sdk_root;
///
/// // `sa-Latn` keeps its hand corrections beside its sources.
/// assert!(sdk_root().join("sa-Latn").join(OVERRIDES_FILE).is_file());
/// ```
pub const OVERRIDES_FILE: &str = "_overrides.json";

/// What deriving a locale produced: the files and what they say.
///
/// ```
/// # use teistro_intl::sdk_root;
/// # use teistro_intl::source::Tree;
/// #
/// # let tree = Tree::load(&sdk_root())?;
///
/// use teistro_intl::derive::{Casing, derive, overrides_of};
///
/// let derived = derive(&tree, "sa-Deva", "sa-Latn", &overrides_of(&sdk_root(), "sa-Latn")?, Casing::Names)?;
/// // What is checked in is what the derivation writes.
/// for (path, text) in &derived.files {
///     assert_eq!(&std::fs::read_to_string(sdk_root().join(path))?, text);
/// }
/// assert!(derived.stale.is_empty());
/// assert!(derived.agreeing <= derived.entities);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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

/// How a derived locale capitalises what it transliterates.
///
/// The source script has no case, so the target's is the derivation's to
/// choose — and the right choice depends on what the root holds, which is
/// why it is **declared** and never inferred from the text. A name table
/// and a corpus of passages both reach this function, and a rule that
/// guessed between them by counting words would be exactly the inference
/// this project refuses elsewhere.
///
/// ```
/// use teistro_intl::derive::Casing;
///
/// // A name table is title-cased; a corpus of passages, by sentence.
/// assert_ne!(Casing::Names, Casing::Sentences);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Casing {
    /// Every word's first letter, which is how a Latin script writes a
    /// **name** and how the sources' own `iast` forms are written:
    /// `Aśvinī Kumāra`, not `Aśvinī kumāra`. This is what `i18n/` wants,
    /// where every form is a name.
    #[default]
    Names,
    /// The first letter of each sentence, which is how a Latin script
    /// writes **prose**. A corpus of readings is passages, and title-casing
    /// one gives `Gururlagne Rājayogakārakaḥ. Prajñāvān, Dhārmikaḥ` where
    /// the text says a sentence (`03-design/state-readings.md`).
    Sentences,
}

/// A locale derived from another by transliteration.
///
/// `overrides` is the target's `_overrides.json` as it stands: a map of
/// `<kind>.<KEY>` to the forms that replace the mechanical result
/// (`{"graha.SUN": {"name": "Sūrya"}}`).
///
/// `casing` says what the root holds — names or prose — because the source
/// script carries no case and the target's is not derivable from the text
/// ([`Casing`]).
///
/// ```
/// # use teistro_intl::sdk_root;
/// # use teistro_intl::source::Tree;
/// #
/// # let tree = Tree::load(&sdk_root())?;
///
/// use teistro_intl::derive::{Casing, derive, overrides_of};
///
/// let derived = derive(&tree, "sa-Deva", "sa-Latn", &overrides_of(&sdk_root(), "sa-Latn")?, Casing::Names)?;
/// assert!(derived.entities > 0);
/// // A source the tree lacks is refused by name.
/// let missing = derive(&tree, "pi-Deva", "pi-Latn", &Default::default(), Casing::Names).unwrap_err();
/// assert!(missing.contains("pi-Deva"));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// No such source locale, or a script pair with no table.
pub fn derive(
    tree: &Tree,
    from: &str,
    to: &str,
    overrides: &BTreeMap<String, BTreeMap<String, String>>,
    casing: Casing,
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
    let meta = derived_meta(source, from, to, &latin)?;
    out.files.push((
        PathBuf::from(to).join(META_FILE),
        format!("{}\n", pretty(&meta)?),
    ));

    // The messages: each source's Devanagari in the target's script and
    // its syntax as it was.
    for name in DERIVED_MESSAGES {
        if let Some(namespace) = source.namespaces.get(name) {
            let sentences = SENTENCE_NAMESPACES.contains(&name);
            out.files.push((
                PathBuf::from(to).join(format!("{name}.json")),
                format!(
                    "{}\n",
                    pretty(&derived_messages(namespace, sentences, &latin)?)?
                ),
            ));
        }
    }

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
                cased(&latin(text)?, casing)
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
            group.insert(member.to_string(), derived.source_json());
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

/// The message namespaces a derived locale carries beside its entities,
/// so a Latin-script reader of Sanskrit gets its weekdays, months and
/// dates, the composed text (SR-38) and the currency words an amount in
/// words is closed with (`sdk.amount`, `03-design/amounts-in-words.md` §5),
/// a match's bands (`sdk.matching`, SR-39) and the sankalpa
/// (`sdk.sankalpa`, SR-29) in Sanskrit rather than in the fallback's
/// English. A namespace listed here and absent from the source is
/// skipped, so the list may name what the source has yet to write.
///
/// ```
/// use teistro_intl::derive::DERIVED_MESSAGES;
///
/// assert_eq!(DERIVED_MESSAGES[0], "sdk.calendar");
/// assert!(DERIVED_MESSAGES.contains(&"sdk.reason"));
/// assert!(DERIVED_MESSAGES.contains(&"sdk.amount"));
/// ```
pub const DERIVED_MESSAGES: [&str; 12] = [
    "sdk.calendar",
    "sdk.reason",
    "sdk.condition",
    "sdk.phala",
    "sdk.reading",
    "sdk.karaka",
    "sdk.aspect",
    "sdk.naming",
    "sdk.day",
    "sdk.amount",
    "sdk.matching",
    "sdk.sankalpa",
];

/// The namespaces whose messages are sentences, which a Latin script
/// opens with a capital, as the base's English ones are (`The Sun rises
/// at …`); the rest are phrases a reader sets inside their own text, left
/// as the transliteration writes them. Listed, for the reason
/// [`NAMED_MESSAGES`] is.
const SENTENCE_NAMESPACES: [&str; 1] = ["sdk.day"];

/// The keys whose messages are names (a weekday, a month), which a Latin
/// script capitalises; the rest are patterns and phrases, left as the
/// transliteration writes them. Listed, because the source script has no
/// case to read it from.
const NAMED_MESSAGES: [&str; 5] = [
    "weekdayName",
    "weekdayShort",
    "monthName",
    "monthShort",
    "nepalSambatMonth",
];

/// A namespace of messages, each transliterated, nested by its dotted
/// keys as a source file writes it.
fn derived_messages(
    namespace: &Namespace,
    sentences: bool,
    latin: &dyn Fn(&str) -> Result<String, String>,
) -> Result<Json, String> {
    let mut document = Map::new();
    for key in &namespace.order {
        let Some(Entry::Message(text)) = namespace.entries.get(key) else {
            continue;
        };
        let named = sentences
            || key
                .rsplit('.')
                .next()
                .is_some_and(|last| NAMED_MESSAGES.contains(&last));
        let mut path: Vec<&str> = key.split('.').collect();
        let Some(leaf) = path.pop() else {
            continue;
        };
        let mut at = &mut document;
        for part in path {
            let next = at
                .entry(part.to_string())
                .or_insert_with(|| Json::Object(Map::new()));
            let Json::Object(next) = next else {
                return Err(format!("`{key}` is both a message and a group"));
            };
            at = next;
        }
        at.insert(
            leaf.to_string(),
            json!(transliterated_message(text, named, latin)?),
        );
    }
    Ok(Json::Object(document))
}

/// A `MessageFormat 2` source with each run of Devanagari transliterated
/// and everything else, the syntax and its placeholders, kept. In a name's
/// message, or a sentence's, a run that opens a variant, or the message,
/// is capitalised.
fn transliterated_message(
    text: &str,
    named: bool,
    latin: &dyn Fn(&str) -> Result<String, String>,
) -> Result<String, String> {
    let devanagari = |c: &char| matches!(c, '\u{0900}'..='\u{097F}' | '\u{A8E0}'..='\u{A8FF}' | '\u{1CD0}'..='\u{1CFF}');
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(&c) = chars.peek() {
        if devanagari(&c) {
            // A run opens the message or a variant when nothing but its
            // `{` comes before it.
            let opens = out.chars().next_back().is_none_or(|before| before == '{');
            let mut run = String::new();
            while let Some(c) = chars.next_if(devanagari) {
                run.push(c);
            }
            let written = latin(&run)?;
            out.push_str(&if named && opens {
                capitalised(&written)
            } else {
                written
            });
        } else {
            out.push(c);
            chars.next();
        }
    }
    Ok(out)
}

/// A derived locale's metadata: the source's, its tag, digits and
/// grouping the target's, its list patterns transliterated, and only the
/// drafts of the namespaces the derivation carries.
fn derived_meta(
    source: &LocaleSource,
    from: &str,
    to: &str,
    latin: &dyn Fn(&str) -> Result<String, String>,
) -> Result<Json, String> {
    let mut meta: Json =
        serde_json::to_value(&source.meta).map_err(|e| format!("{from}'s metadata: {e}"))?;
    if let Some(object) = meta.as_object_mut() {
        object.insert(String::from("locale"), json!(to));
        object.insert(String::from("numberingSystem"), json!("latn"));
        object.insert(String::from("grouping"), json!([3]));
        // The source's drafts that the derivation carries stay drafts:
        // transliterating a draft does not review it.
        if let Some(Json::Array(drafts)) = object.get_mut("draft") {
            drafts.retain(|name| {
                name.as_str().is_some_and(|name| {
                    DERIVED_MESSAGES.contains(&name) && source.namespaces.contains_key(name)
                })
            });
            if drafts.is_empty() {
                object.remove("draft");
            }
        }
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
    Ok(meta)
}

/// The overrides beside a derived locale, or none when it has no file.
///
/// ```
/// use teistro_intl::derive::overrides_of;
/// use teistro_intl::sdk_root;
///
/// assert!(!overrides_of(&sdk_root(), "sa-Latn")?.is_empty());
/// // A locale with no file has none.
/// assert!(overrides_of(&sdk_root(), "ne-Deva-NP")?.is_empty());
/// # Ok::<(), String>(())
/// ```
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

/// What ends a sentence, in either script's punctuation: the Latin stops
/// and the danda, which a Devanagari source writes and the transliteration
/// carries through.
const SENTENCE_ENDS: [char; 5] = ['.', '!', '?', '।', '॥'];

/// Transliterated text, cased as the root it came from is written.
fn cased(text: &str, casing: Casing) -> String {
    match casing {
        Casing::Names => capitalised(text),
        Casing::Sentences => sentence_cased(text),
    }
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

/// Prose as a Latin script writes it: the first letter of the text and of
/// every sentence after it a capital, and nothing else touched.
///
/// A letter is capitalised only where a sentence has ended and the words
/// between have not started one — so `8 adhyāye` keeps its digit and
/// capitalises the letter that follows it, and an abbreviation's stop is
/// treated as an end like any other, which is the cost of not parsing the
/// language.
fn sentence_cased(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut starting = true;
    for letter in text.chars() {
        if starting && letter.is_alphabetic() {
            out.extend(letter.to_uppercase());
            starting = false;
        } else {
            out.push(letter);
            if SENTENCE_ENDS.contains(&letter) {
                starting = true;
            }
        }
    }
    out
}

/// The script half of a locale tag (`sa-Latn` is `Latn`), lowercased.
fn script_of(tag: &str) -> &str {
    tag.split('-').nth(1).unwrap_or("")
}

fn pretty(value: &Json) -> Result<String, String> {
    serde_json::to_string_pretty(value).map_err(|e| e.to_string())
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
    use crate::source::sdk_root;

    fn tree() -> Tree {
        Tree::load(&sdk_root()).unwrap_or_else(|e| panic!("{e}"))
    }

    /// A name table is cased by word and a corpus of prose by sentence,
    /// and the derivation is told which: the source script carries no case
    /// and no rule over the text could tell a two-word passage from a
    /// two-word name.
    #[test]
    fn prose_is_cased_by_sentence_and_a_name_by_word() {
        let passage = "gururlagne rājayogakārakaḥ. prajñāvān, dhārmikaḥ. 8 adhyāye uktam";
        assert_eq!(
            super::sentence_cased(passage),
            "Gururlagne rājayogakārakaḥ. Prajñāvān, dhārmikaḥ. 8 Adhyāye uktam",
            "each sentence opens with a capital and nothing else moves"
        );
        assert_eq!(
            super::capitalised(passage),
            "Gururlagne Rājayogakārakaḥ. Prajñāvān, Dhārmikaḥ. 8 Adhyāye Uktam",
            "a name table capitalises every word, which is why prose needs its own"
        );
        // The danda ends a sentence where a transliteration keeps it.
        assert_eq!(super::sentence_cased("ekaḥ। dvau"), "Ekaḥ। Dvau");
        // And text that starts with punctuation still capitalises its
        // first letter rather than nothing.
        assert_eq!(super::sentence_cased("— ekaḥ"), "— Ekaḥ");
        assert_eq!(super::sentence_cased(""), "");
    }

    /// A final source derives a final locale: no draft list at all.
    #[test]
    fn a_final_source_derives_no_draft_list() {
        let derived = derive(
            &tree(),
            "sa-Deva",
            "sa-Latn",
            &BTreeMap::new(),
            Casing::Names,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        let meta = file(&derived, "_meta.json");
        assert!(!meta.contains("\"draft\""), "{meta}");
    }

    /// A derived file's text by its name.
    fn file<'a>(derived: &'a super::Derived, name: &str) -> &'a str {
        derived
            .files
            .iter()
            .find(|(path, _)| path.ends_with(name))
            .map_or_else(|| panic!("no {name} derived"), |(_, text)| text.as_str())
    }

    #[test]
    fn a_latin_locale_is_derived_from_the_devanagari_one() {
        // Marked draft, as a third party's unreviewed source would be, so
        // the test sees the mark carried across.
        let mut source = tree();
        if let Some(sanskrit) = source.locales.get_mut("sa-Deva") {
            sanskrit.meta.draft = DERIVED_MESSAGES
                .iter()
                .map(|name| (*name).to_owned())
                .collect();
        }
        let derived = derive(
            &source,
            "sa-Deva",
            "sa-Latn",
            &BTreeMap::new(),
            Casing::Names,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        // The metadata, the entities, and a file for each namespace listed:
        // the calendar's, the currency words (SR-49) and the composed text
        // (SR-38).
        assert_eq!(derived.files.len(), 2 + DERIVED_MESSAGES.len());
        assert_eq!(
            derived.entities, 1428,
            "408 until the 18 dasha systems were named, 426 until the six ritus, \
             432 until the year's six offices, two texts and four rules, 444 until \
             every listed member was named (SR-12), 792 until the round and petalled \
             charts and the navamsha's short form (SR-36), 795 until the 429 \
             festivals were named in Sanskrit, 1224 until the four chart frames, \
             1228 until Tamil Nadu's six, 1234 until Assam's eighteen, 1252 until \
             Karnataka's twelve, 1264 until Telangana's seven, 1271 until Gujarat's three, \
             1274 until Andhra Pradesh's one, \
             1275 until SR-01's nine, 1284 until Uttar Pradesh's ten, \
             1294 until Odisha's thirteen, \
             1307 until SR-01's twenty-eight civil days, \
             1335 until Yosin utthan, \
             1336 until its fifty-six UN and organisers' days, \
             1392 until SR-01's twenty-nine Nepal and organisers' days, \
             1421 until the six printed charhe and Ghantakarna"
        );
        assert!(derived.stale.is_empty(), "{:?}", derived.stale);

        let entities = file(&derived, "sdk.entity.json");
        assert!(entities.contains("\"name\": \"Sūrya\""), "the Sun's name");
        assert!(
            entities.contains("\"iast\": \"Sūrya\""),
            "the iast form stands"
        );
        assert!(entities.contains("\"glyph\": \"☉\""));
        let calendar = file(&derived, "sdk.calendar.json");
        assert!(
            calendar.contains("{{Ravivāsaraḥ}}"),
            "a name's message is capitalised"
        );
        assert!(calendar.contains("{$day :integer}"), "the syntax is kept");
        // The composed text: a phrase is left as written, a sentence opens
        // with a capital, and a stem's case ending follows its slot.
        let reason = file(&derived, "sdk.reason.json");
        assert!(
            reason.contains("1 {{{$graha}ḥ prathame bhāve}}"),
            "a phrase keeps the transliteration's case"
        );
        let day = file(&derived, "sdk.day.json");
        assert!(
            day.contains("\"Sūryaḥ sakaladinaṃ"),
            "a sentence opens with a capital"
        );
        let meta = file(&derived, "_meta.json");
        assert!(meta.contains("\"locale\": \"sa-Latn\""));
        for name in DERIVED_MESSAGES {
            assert!(
                meta.contains(&format!("\"{name}\"")),
                "the derived draft {name} stays a draft"
            );
        }
        assert!(meta.contains("\"numberingSystem\": \"latn\""));
        assert!(meta.contains("tathā"), "the list pattern is transliterated");
        // The currency words come too.
        let amount = file(&derived, "sdk.amount.json");
        assert!(amount.contains("{{rūpyakāṇi}}"), "{amount}");
        assert!(
            amount.contains(".match $count 1"),
            "the syntax passes through"
        );
        // 261 before the vetted name tables' import, and 48 of the 114
        // it added: every record whose Devanagari is exactly the source's
        // Sanskrit (the upagrahas, months, directions and states, and
        // three ayanamshas), so the transliterator matched each vetted
        // IAST form it was given; 326 with 17 of the 18 dasha systems.
        // The eighteenth is Vimshottari, whose source writes the anusvara
        // `ṁ` where IAST writes `ṃ`. 331 with five of the six ritus: the
        // source writes शरद without the virama the Sanskrit stem śarad
        // takes, which the shipped overrides restore. 343 with the year's
        // rulers, each named in the texts' own Sanskrit. 475 once every
        // listed member was named (SR-12), each Sanskrit term with its IAST.
        // 518 with the festivals in Sanskrit: 43 of the 151 that carry an
        // IAST form are the text's closed compound it spells; the rest
        // differ by a space or by taking the texts' name over the print's.
        // 519 with SR-01's महालक्ष्मीव्रतारम्भ, a closed compound too; 520
        // with the six charhe and Ghantakarna, one of whose IAST forms the
        // transliterator writes letter for letter (the charhe's `Carhe` is
        // the spoken name, as Dila's and Lai's are).
        assert_eq!(
            derived.agreeing, 520,
            "the derived names that are letter for letter the sources' own iast form"
        );
    }

    #[test]
    fn the_shipped_overrides_all_apply_and_restore_the_western_names() {
        // Transliterating a Western name that was itself transliterated
        // into Devanagari writes nonsense (`Plāsiḍasa`), so the shipped
        // overrides give those names the source's English
        // (`docs/03-design/entity-names.md` §3). Every one must still
        // match an entity, and each replaces a name and its prose.
        let overrides = overrides_of(&sdk_root(), "sa-Latn").unwrap_or_else(|e| panic!("{e}"));
        let derived = derive(&tree(), "sa-Deva", "sa-Latn", &overrides, Casing::Names)
            .unwrap_or_else(|e| panic!("{e}"));
        assert!(derived.stale.is_empty(), "{:?}", derived.stale);
        assert_eq!(derived.overridden, 2 * overrides.len());
        let entities = file(&derived, "sdk.entity.json");
        assert!(entities.contains("\"name\": \"Placidus\""));
        assert!(entities.contains("\"name\": \"Fagan-Bradley\""));
        assert!(
            !entities.contains("Plāsiḍasa"),
            "the mechanical name is gone"
        );
        // And a descriptive Sanskrit name keeps its derived form.
        assert!(entities.contains("\"name\": \"Sampūrṇa Rāśi (rāśi-bhāva)\""));
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
        let derived = derive(&tree(), "sa-Deva", "sa-Latn", &overrides, Casing::Names)
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(derived.overridden, 1);
        assert_eq!(derived.stale, vec![String::from("graha.VULCAN")]);
        assert!(file(&derived, "sdk.entity.json").contains("\"name\": \"Sūrya\""));
    }

    #[test]
    fn a_locale_or_a_script_the_build_does_not_know_is_refused_by_name() {
        assert!(
            derive(
                &tree(),
                "xx-Deva",
                "xx-Latn",
                &BTreeMap::new(),
                Casing::Names
            )
            .unwrap_err()
            .contains("no locale `xx-Deva`")
        );
        assert!(
            derive(
                &tree(),
                "sa-Deva",
                "sa-Taml",
                &BTreeMap::new(),
                Casing::Names
            )
            .unwrap_err()
            .contains("names no script")
        );
    }
}
