//! A dossier written in a locale's names (`03-design/dossier.md`,
//! "Names", A2-7, C1166): the dossier the serial crate writes in keys,
//! each key that names one member of the locale's lexicon replaced by its
//! name. The legend and the banners stay as written, so a reader parses a
//! named dossier as it parses a keyed one.

use std::collections::BTreeMap;

use teistro_core::error::Error;
use teistro_intl::lexicon::LexiconRequest;

use crate::context::Context;

/// The legend's sentence on members, as the serial crate writes it.
const KEYED: &str = "Members are catalogue keys (JUPITER, ARIES).";

/// Each bare key (`JUPITER`) the lexicon gives one name, with that name,
/// and the locale the lexicon answered for.
fn names(
    context: &Context,
    locale: Option<&str>,
) -> Result<(String, BTreeMap<String, String>), Error> {
    let lexicon = context.locale_engine().lexicon(&LexiconRequest {
        locale: locale.map(str::to_owned),
        kinds: None,
    })?;
    let pairs = lexicon.entries.iter().filter_map(|entry| {
        let (_, bare) = entry.key.split_once('.')?;
        let form = entry
            .forms
            .iter()
            .find(|form| form.form == "name")
            .or_else(|| entry.forms.first())?;
        Some((bare.to_owned(), form.text.clone()))
    });
    Ok((lexicon.locale, unambiguous(pairs)))
}

/// The keys that name one text: a key two kinds name differently
/// (`MAGHA`, a nakshatra and a month, in a locale that spells them apart)
/// is left out and stays a key. A name holding the column separator would
/// split its row, so `|` becomes `/`.
fn unambiguous(pairs: impl Iterator<Item = (String, String)>) -> BTreeMap<String, String> {
    let mut named: BTreeMap<String, Option<String>> = BTreeMap::new();
    for (bare, name) in pairs {
        let name = name.replace('|', "/");
        named
            .entry(bare)
            .and_modify(|held| {
                if held.as_deref() != Some(name.as_str()) {
                    *held = None;
                }
            })
            .or_insert(Some(name));
    }
    named
        .into_iter()
        .filter_map(|(bare, name)| Some((bare, name?)))
        .collect()
}

/// Whether `c` may stand in a key.
fn in_key(c: char) -> bool {
    c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'
}

/// One row with every whole key the map names replaced.
fn row(line: &str, named: &BTreeMap<String, String>) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(start) = rest.find(|c: char| c.is_ascii_uppercase()) {
        let (before, from) = rest.split_at(start);
        // A key starts where no word runs on into it.
        let joined = before
            .chars()
            .next_back()
            .is_some_and(|c| c.is_alphanumeric() || c == '_');
        let end = from.find(|c: char| !in_key(c)).unwrap_or(from.len());
        let (word, after) = from.split_at(end);
        out.push_str(before);
        match named.get(word) {
            Some(name) if !joined && !after.starts_with(|c: char| c.is_alphanumeric()) => {
                out.push_str(name);
            }
            _ => out.push_str(word),
        }
        rest = after;
    }
    out.push_str(rest);
    out
}

/// `dossier` (written in keys) in `locale`'s names, the context's locale
/// when it names none: every row's keys the lexicon names unambiguously
/// replaced, the legend saying so.
pub(crate) fn named(
    context: &Context,
    dossier: &str,
    locale: Option<&str>,
) -> Result<String, Error> {
    let (tag, named) = names(context, locale)?;
    let legend =
        format!("Members are named in {tag}; a key stands where a name could mean two members.");
    let mut out = String::with_capacity(dossier.len() + dossier.len() / 4);
    for line in dossier.split_inclusive('\n') {
        if line.starts_with('#') {
            out.push_str(&line.replace(KEYED, &legend));
        } else if line.starts_with("== ") {
            out.push_str(line);
        } else {
            out.push_str(&row(line, &named));
        }
    }
    Ok(out)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    fn map(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect()
    }

    #[test]
    fn a_row_names_whole_keys_only() {
        let named = map(&[
            ("JUPITER", "Jupiter"),
            ("ARIES", "Aries"),
            ("D9", "Navamsha"),
        ]);
        assert_eq!(
            row("JUPITER|ARIES|12.5|-\n", &named),
            "Jupiter|Aries|12.5|-\n"
        );
        assert_eq!(
            row("JUPITER>ARIES D9\n", &named),
            "Jupiter>Aries Navamsha\n"
        );
        // Part of a longer key, or of a word, is not a key.
        assert_eq!(
            row("JUPITER_RX|xARIES|ARIESx|UNKNOWN\n", &named),
            "JUPITER_RX|xARIES|ARIESx|UNKNOWN\n"
        );
        assert_eq!(row("", &named), "");
    }

    #[test]
    fn a_key_two_kinds_name_apart_stays_a_key() {
        let pairs = [
            ("MAGHA", "Magha"),
            ("MAGHA", "Magh"),
            ("ROHINI", "Rohini"),
            ("ROHINI", "Rohini"),
            ("BAR", "a|b"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v.to_owned()));
        assert_eq!(
            unambiguous(pairs),
            map(&[("ROHINI", "Rohini"), ("BAR", "a/b")])
        );
    }

    #[test]
    fn a_dossier_keeps_its_legend_and_banners() {
        let sdk = Context::builder()
            .ephemeris([crate::Ephemeris::None])
            .build()
            .unwrap();
        let keyed = format!(
            "# teistro dossier 2\n== LEGEND ==\n# x. {KEYED}\n== GRAHAS ==\n# graha|sign\nJUPITER|ARIES\nMAGHA|-\n"
        );
        let text = named(&sdk, &keyed, Some("en-Latn")).unwrap();
        assert!(text.contains("== GRAHAS ==\n# graha|sign\n"), "{text}");
        assert!(text.contains("Members are named in en-Latn"), "{text}");
        assert!(text.contains("\nJupiter|Aries\n"), "{text}");
        let nepali = named(&sdk, &keyed, Some("ne")).unwrap();
        assert!(
            nepali.contains("बृहस्पति") || nepali.contains("गुरु"),
            "{nepali}"
        );
        assert!(named(&sdk, &keyed, Some("xx-Nowhere")).is_err());
    }
}
