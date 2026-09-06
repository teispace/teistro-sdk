//! XLIFF 2.1 export and import, so a translator's own tools can take the
//! sources away and bring them back
//! (`02-architecture/03-localization-architecture.md`, the CLI table).
//!
//! One file per locale: the base locale is the source, the locale being
//! translated is the target, and every message and every entity form is a
//! unit under the namespace it belongs to. A message crosses as its
//! `MessageFormat 2` source, because XLIFF has no notion of it, with a
//! note
//! naming the parameters so a translator knows what `{$graha}` stands for.
//!
//! Import is the inverse and no more: a unit with an empty target is one
//! nobody has translated yet and is left alone, a unit whose id the base
//! locale does not have is reported, and anything the file does not
//! mention keeps whatever the locale already said.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::analysis::signature;
use crate::source::{ENTITY_NAMESPACE, Entity, Entry, LocaleSource, Namespace, Tree};

/// The forms of an entity a translator sees, in the order they are shown.
/// The glyph and the gender are the record's own: a symbol is the same in
/// every language, and a gender is a grammatical fact the locale states
/// rather than a translation.
pub const TRANSLATED_FORMS: [&str; 4] = ["name", "prose", "short", "iast"];

/// What an export produced.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Exported {
    /// The XLIFF document.
    pub text: String,
    /// The units written.
    pub units: usize,
    /// The units whose target is empty, which is the work to do.
    pub untranslated: usize,
}

/// What an import found.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Imported {
    /// The locale the file names as its target.
    pub locale: String,
    /// The entries to write, by namespace.
    pub namespaces: BTreeMap<String, BTreeMap<String, String>>,
    /// The units with a target.
    pub translated: usize,
    /// The units left alone because their target is empty.
    pub empty: usize,
    /// The unit ids the base locale does not have, which are a fault: the
    /// file was made from another version of the sources.
    pub unknown: Vec<String>,
}

/// The sources as one XLIFF 2.1 document: the base locale's text as the
/// source and `locale`'s as the target.
///
/// # Errors
///
/// No base locale, or no such target locale.
pub fn export(tree: &Tree, locale: &str) -> Result<Exported, String> {
    let base = tree.base().ok_or("no base locale")?;
    let target = tree
        .locales
        .get(locale)
        .ok_or_else(|| format!("no locale `{locale}` in the sources"))?;
    let mut out = Exported::default();
    let mut body = String::new();
    for (namespace, entries) in &base.namespaces {
        let _ = writeln!(body, "  <file id=\"{}\">", escape(namespace));
        let theirs = target.namespaces.get(namespace);
        for key in &entries.order {
            let Some(entry) = entries.entries.get(key) else {
                continue;
            };
            match entry {
                Entry::Message(source) => {
                    let note = parameters(base, source);
                    let text = theirs.and_then(|ns| match ns.entries.get(key) {
                        Some(Entry::Message(text)) => Some(text.as_str()),
                        _ => None,
                    });
                    unit(
                        &mut body,
                        &mut out,
                        &format!("{namespace}.{key}"),
                        source,
                        text,
                        &note,
                    );
                }
                Entry::Entity(entity) => {
                    let theirs = theirs.and_then(|ns| match ns.entries.get(key) {
                        Some(Entry::Entity(entity)) => Some(entity),
                        _ => None,
                    });
                    for form in TRANSLATED_FORMS {
                        let Some(source) = entity.forms.get(form) else {
                            continue;
                        };
                        let text = theirs.and_then(|e| e.forms.get(form)).map(String::as_str);
                        unit(
                            &mut body,
                            &mut out,
                            &format!("{namespace}.{key}#{form}"),
                            source,
                            text,
                            &format!("the `{form}` form of {key}"),
                        );
                    }
                }
            }
        }
        let _ = writeln!(body, "  </file>");
    }
    out.text = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<xliff xmlns=\"urn:oasis:names:tc:xliff:document:2.0\" version=\"2.1\" srcLang=\"{}\" trgLang=\"{}\">\n{body}</xliff>\n",
        escape(&base.tag),
        escape(&target.tag)
    );
    Ok(out)
}

/// One unit: the source, the target when the locale has one, and a note.
fn unit(
    body: &mut String,
    out: &mut Exported,
    id: &str,
    source: &str,
    target: Option<&str>,
    note: &str,
) {
    let target = target.unwrap_or_default();
    out.units += 1;
    if target.is_empty() {
        out.untranslated += 1;
    }
    let _ = writeln!(
        body,
        "    <unit id=\"{}\">\n      <notes>\n        <note>{}</note>\n      </notes>\n      <segment>\n        <source>{}</source>\n        <target>{}</target>\n      </segment>\n    </unit>",
        escape(id),
        escape(note),
        escape(source),
        escape(target)
    );
}

/// The note a message carries: what its parameters are, so a translator
/// keeps every one of them.
fn parameters(base: &LocaleSource, source: &str) -> String {
    let Ok(message) = crate::mf2::parse(source) else {
        return String::from("a MessageFormat 2 message");
    };
    let signature = signature(&message, &base.meta);
    if signature.params.is_empty() {
        return String::from("a MessageFormat 2 message with no parameters");
    }
    let names: Vec<String> = signature
        .params
        .iter()
        .map(|(name, kind)| format!("${name} ({kind:?})"))
        .collect();
    format!(
        "a MessageFormat 2 message; keep every parameter: {}",
        names.join(", ")
    )
}

/// An XLIFF document read back: what it says the locale's entries are.
///
/// # Errors
///
/// A document that does not parse, or one whose target language the
/// sources do not have.
pub fn import(tree: &Tree, xml: &str) -> Result<Imported, String> {
    let base = tree.base().ok_or("no base locale")?;
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut out = Imported::default();
    let (mut id, mut inside, mut text) = (String::new(), Field::None, String::new());
    let mut target = String::new();
    loop {
        match reader.read_event() {
            Err(error) => return Err(format!("the document does not parse: {error}")),
            Ok(Event::Eof) => break,
            Ok(Event::Start(tag)) => match tag.name().as_ref() {
                b"xliff" => out.locale = attribute(&tag, b"trgLang")?,
                b"unit" => {
                    id = attribute(&tag, b"id")?;
                    target.clear();
                }
                b"target" => {
                    inside = Field::Target;
                    text.clear();
                }
                b"source" | b"note" => inside = Field::Other,
                _ => {}
            },
            Ok(Event::Text(chunk)) if inside == Field::Target => {
                text.push_str(&chunk.decode().map_err(|e| e.to_string())?);
            }
            Ok(Event::End(tag)) => match tag.name().as_ref() {
                b"target" => {
                    target.clone_from(&text);
                    inside = Field::None;
                }
                b"source" | b"note" => inside = Field::None,
                b"unit" => take(&mut out, base, &id, &target),
                _ => {}
            },
            Ok(_) => {}
        }
    }
    if out.locale.is_empty() {
        return Err(String::from("the document names no target language"));
    }
    Ok(out)
}

/// Which element's text is being read.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Field {
    None,
    Target,
    Other,
}

/// One unit's target, taken when the base locale has its key and the
/// target says something.
fn take(out: &mut Imported, base: &LocaleSource, id: &str, target: &str) {
    let (key, form) = match id.split_once('#') {
        Some((key, form)) => (key, Some(form)),
        None => (id, None),
    };
    let Some((namespace, inside)) = base.split(key) else {
        out.unknown.push(id.to_string());
        return;
    };
    let known = base
        .namespaces
        .get(namespace)
        .is_some_and(|ns| ns.entries.contains_key(inside));
    if !known {
        out.unknown.push(id.to_string());
        return;
    }
    if target.is_empty() {
        out.empty += 1;
        return;
    }
    out.translated += 1;
    let entry = match form {
        Some(form) => format!("{inside}#{form}"),
        None => inside.to_string(),
    };
    out.namespaces
        .entry(namespace.to_string())
        .or_default()
        .insert(entry, target.to_string());
}

/// The imported entries applied to a locale's namespaces: a message
/// replaced, an entity's form replaced, everything else left as it was.
#[must_use]
pub fn apply(imported: &Imported, locale: &LocaleSource) -> BTreeMap<String, Namespace> {
    let mut out = locale.namespaces.clone();
    for (namespace, entries) in &imported.namespaces {
        let target = out.entry(namespace.clone()).or_default();
        for (key, text) in entries {
            match key.split_once('#') {
                Some((key, form)) if namespace == ENTITY_NAMESPACE => {
                    let entry = target
                        .entries
                        .entry(key.to_string())
                        .or_insert_with(|| Entry::Entity(Entity::default()));
                    if let Entry::Entity(entity) = entry {
                        entity.forms.insert(form.to_string(), text.clone());
                    }
                    if !target.order.iter().any(|k| k == key) {
                        target.order.push(key.to_string());
                    }
                }
                _ => {
                    target.insert(key.clone(), Entry::Message(text.clone()));
                }
            }
        }
    }
    out
}

/// An attribute's value, or the name of the one that is missing.
fn attribute(tag: &quick_xml::events::BytesStart<'_>, name: &[u8]) -> Result<String, String> {
    for attribute in tag.attributes().flatten() {
        if attribute.key.as_ref() == name {
            // An XLIFF file is XML 1.0, whether or not it says so, and
            // the two spellings normalise an attribute the same way.
            return attribute
                .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                .map(|value| value.to_string())
                .map_err(|e| e.to_string());
        }
    }
    Err(format!(
        "a <{}> without `{}`",
        String::from_utf8_lossy(tag.name().as_ref()),
        String::from_utf8_lossy(name)
    ))
}

/// Text as XML carries it.
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            c => out.push(c),
        }
    }
    out
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
    fn a_locale_exports_every_message_and_form_with_its_source() {
        let tree = tree();
        let exported = export(&tree, "ne-Deva-NP").unwrap_or_else(|e| panic!("{e}"));
        assert!(exported.text.starts_with("<?xml version=\"1.0\""));
        assert!(
            exported
                .text
                .contains("srcLang=\"en-Latn\" trgLang=\"ne-Deva-NP\"")
        );
        assert!(exported.text.contains("<file id=\"sdk.reason\">"));
        assert!(
            exported
                .text
                .contains("<unit id=\"sdk.entity.graha.SUN#name\">")
        );
        assert!(exported.text.contains("<source>the Sun</source>"));
        assert!(exported.text.contains("<target>सूर्य</target>"));
        assert!(
            exported.text.contains("keep every parameter: $bhava"),
            "the note names the parameters"
        );
        assert!(exported.units > 300);
        assert_eq!(
            exported.untranslated, 8,
            "the eight `short` forms the Nepali locale does not abbreviate"
        );

        // A locale whose messages are untranslated says so.
        let sanskrit = export(&tree, "sa-Deva").unwrap_or_else(|e| panic!("{e}"));
        assert!(sanskrit.untranslated > 0);
    }

    #[test]
    fn what_was_exported_imports_back_unchanged() {
        let tree = tree();
        let exported = export(&tree, "ne-Deva-NP").unwrap_or_else(|e| panic!("{e}"));
        let imported = import(&tree, &exported.text).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(imported.locale, "ne-Deva-NP");
        assert_eq!(imported.translated, exported.units - exported.untranslated);
        assert_eq!(imported.empty, exported.untranslated);
        assert!(imported.unknown.is_empty());

        let locale = tree
            .locales
            .get("ne-Deva-NP")
            .unwrap_or_else(|| panic!("locale"));
        let applied = apply(&imported, locale);
        for (name, namespace) in &locale.namespaces {
            let after = applied.get(name).unwrap_or_else(|| panic!("{name}"));
            assert_eq!(
                &after.entries, &namespace.entries,
                "{name} came back as it went"
            );
        }
    }

    #[test]
    fn a_translation_replaces_what_it_names_and_nothing_else() {
        let tree = tree();
        let xml = "<?xml version=\"1.0\"?>\n<xliff xmlns=\"urn:oasis:names:tc:xliff:document:2.0\" version=\"2.1\" srcLang=\"en-Latn\" trgLang=\"sa-Deva\">\n  <file id=\"sdk.reason\">\n    <unit id=\"sdk.reason.welcome\"><segment><source>Welcome</source><target>स्वागतम्</target></segment></unit>\n    <unit id=\"sdk.reason.appName\"><segment><source>Teistro</source><target></target></segment></unit>\n    <unit id=\"sdk.reason.nowhere\"><segment><source>x</source><target>y</target></segment></unit>\n  </file>\n  <file id=\"sdk.entity\">\n    <unit id=\"sdk.entity.graha.SUN#name\"><segment><source>the Sun</source><target>आदित्य</target></segment></unit>\n  </file>\n</xliff>";
        let imported = import(&tree, xml).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(imported.locale, "sa-Deva");
        assert_eq!(imported.translated, 2);
        assert_eq!(imported.empty, 1, "an untranslated unit is left alone");
        assert_eq!(imported.unknown, vec![String::from("sdk.reason.nowhere")]);

        let locale = tree
            .locales
            .get("sa-Deva")
            .unwrap_or_else(|| panic!("locale"));
        let applied = apply(&imported, locale);
        let reason = applied
            .get("sdk.reason")
            .unwrap_or_else(|| panic!("reason"));
        assert_eq!(
            reason.entries.get("welcome"),
            Some(&Entry::Message(String::from("स्वागतम्")))
        );
        let entities = applied
            .get("sdk.entity")
            .unwrap_or_else(|| panic!("entities"));
        let Some(Entry::Entity(sun)) = entities.entries.get("graha.SUN") else {
            panic!("the Sun");
        };
        assert_eq!(sun.form("name"), Some("आदित्य"));
        assert_eq!(
            sun.form("iast"),
            Some("Sūrya"),
            "a form nobody sent is kept"
        );
        assert_eq!(sun.glyph.as_deref(), Some("☉"), "and so is the glyph");
    }

    #[test]
    fn a_document_that_is_not_one_is_refused_by_name() {
        let tree = tree();
        assert!(
            import(&tree, "not xml at all")
                .unwrap_err()
                .contains("names no target")
        );
        assert!(
            import(&tree, "<xliff srcLang=\"en-Latn\"></xliff>")
                .unwrap_err()
                .contains("without `trgLang`")
        );
        assert!(export(&tree, "xx-Latn").unwrap_err().contains("no locale"));
    }
}
