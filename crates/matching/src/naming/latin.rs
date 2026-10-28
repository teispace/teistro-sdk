//! A name in Latin letters read into Devanagari: exactly under a declared
//! scheme, or every way an everyday spelling allows under `CASUAL`
//! (C293, C772).

use serde::{Deserialize, Serialize};
use teistro_core::error::Error;
use teistro_intl::translit::{Scheme, to_devanagari};

use super::cakra::{VIRAMA, consonant_of, is_devanagari};

/// How a name written in Latin letters is read (C293, C772).
///
/// ```
/// use teistro_matching::{LatinName, NameRules, name_syllable};
///
/// assert_eq!(name_syllable("Rāma", NameRules::default()).unwrap_err().field(), Some("name"));
/// let iast = NameRules { latin: LatinName::Iast, ..NameRules::default() };
/// assert_eq!(name_syllable("Rāma", iast)?, name_syllable("राम", NameRules::default())?);
/// let hk = NameRules { latin: LatinName::HarvardKyoto, ..NameRules::default() };
/// assert_eq!(name_syllable("rAma", hk)?, name_syllable("राम", NameRules::default())?);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LatinName {
    /// Refused: an English spelling is not guessed at, since its "ch" is
    /// च where IAST's is छ.
    #[default]
    Refuse,
    /// Read as IAST, case folded and combining marks composed.
    Iast,
    /// Read as ISO 15919 (IAST with ṁ, r̥, ē and ō), case folded and
    /// combining marks composed.
    #[serde(rename = "ISO_15919")]
    Iso15919,
    /// Read as Harvard-Kyoto, whose capitals are letters: case as written.
    HarvardKyoto,
    /// Read as ITRANS 5.3, whose capitals are letters: case as written.
    Itrans,
    /// Read every way [`CASUAL`]'s list allows an everyday spelling to be
    /// read, and answered with every cell those readings reach; never one
    /// chosen among them (C772).
    Casual,
}

impl LatinName {
    /// The declared scheme read exactly, `None` for `REFUSE` and `CASUAL`.
    fn scheme(self) -> Option<Scheme> {
        match self {
            LatinName::Iast => Some(Scheme::Iast),
            LatinName::Iso15919 => Some(Scheme::Iso15919),
            LatinName::HarvardKyoto => Some(Scheme::HarvardKyoto),
            LatinName::Itrans => Some(Scheme::Itrans),
            LatinName::Refuse | LatinName::Casual => None,
        }
    }
}

/// What an everyday Latin spelling may stand for, as IAST (C772,
/// `03-design/namakarana.md` §2.5): each spelling, the longest first where
/// one begins another, and every reading the list allows it, the plainest
/// first. The design's table lists the spellings that stand for more than
/// one letter; the rest are each one letter's own, listed so that nothing
/// is inferred. The design's "ru" reads as रु too, its plain reading,
/// which the design's row leaves unsaid.
///
/// ```
/// use teistro_matching::CASUAL;
///
/// let sh = CASUAL.iter().find(|(spelled, _)| *spelled == "sh").map(|(_, read)| *read);
/// assert_eq!(sh, Some(&["ś", "ṣ"][..]));
/// ```
pub const CASUAL: [(&str, &[&str]); 48] = [
    // The vowels: length doubled, and the diphthongs C293 refuses first.
    ("aa", &["ā"]),
    ("ai", &["ai"]),
    ("au", &["au"]),
    ("a", &["a"]),
    ("ee", &["ī"]),
    ("e", &["e"]),
    ("ii", &["ī"]),
    ("i", &["i"]),
    ("oo", &["ū"]),
    ("o", &["o"]),
    ("uu", &["ū"]),
    ("u", &["u"]),
    // ṛ, spelled as English and Nepali spell it.
    ("ri", &["ri", "ṛ"]),
    ("ru", &["ru", "ṛ"]),
    // The consonants.
    ("ksh", &["kṣ"]),
    ("kh", &["kh", "ṣ"]),
    ("k", &["k"]),
    ("gh", &["gh"]),
    ("gya", &["gya", "jña"]),
    ("gy", &["gy", "jñ"]),
    ("g", &["g"]),
    ("chh", &["ch"]),
    ("ch", &["c", "ch"]),
    ("jh", &["jh"]),
    ("j", &["j"]),
    ("th", &["th", "ṭh", "t", "ṭ"]),
    ("t", &["t", "ṭ"]),
    ("dh", &["dh", "ḍh"]),
    ("d", &["d", "ḍ"]),
    ("n", &["n", "ṇ"]),
    ("ph", &["ph"]),
    ("f", &["f"]),
    ("p", &["p"]),
    ("bh", &["bh"]),
    ("b", &["v", "b"]),
    ("v", &["v", "b"]),
    ("w", &["v", "b"]),
    ("m", &["m"]),
    ("y", &["y"]),
    ("r", &["r"]),
    ("l", &["l"]),
    ("shh", &["ṣ"]),
    ("sh", &["ś", "ṣ"]),
    ("s", &["s", "ś"]),
    ("h", &["h"]),
    ("x", &["kṣ"]),
    ("z", &["z"]),
    ("q", &["q"]),
];

/// Every Devanagari spelling a name may be read as under `latin`, in the
/// order its readings are listed: one, unless `CASUAL` reads it more ways.
/// A name in Devanagari is its own one spelling.
pub(crate) fn spellings(name: &str, latin: LatinName) -> Result<Vec<String>, Error> {
    let is_latin = name
        .chars()
        .next()
        .is_some_and(|first| first.is_alphabetic() && !is_devanagari(first));
    if !is_latin {
        return Ok(vec![name.to_owned()]);
    }
    if let Some(scheme) = latin.scheme() {
        return to_devanagari(name, scheme)
            .map(|spelled| vec![spelled])
            .map_err(|unfit| {
                Error::invalid_arg(format!("a name in {unfit}"))
                    .with_field("name")
                    .with_hint(format!(
                        "spell the name in {} throughout, or pass it in Devanagari",
                        unfit.scheme
                    ))
            });
    }
    match latin {
        LatinName::Casual => casual(name),
        _ => Err(Error::invalid_arg(
            "a name in Latin letters is read only under a declared scheme, never guessed from English",
        )
        .with_field("name")
        .with_hint(
            "pass the name in Devanagari, or set latin to the scheme it is spelled in \
             (IAST, ISO_15919, HARVARD_KYOTO, ITRANS) or to CASUAL",
        )),
    }
}

/// One spelling of a name read by [`CASUAL`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Token {
    /// Letters, with every reading the list allows them.
    Letters(&'static [&'static str]),
    /// What is not a letter (a space or a hyphen between words), passed
    /// through.
    Other(char),
}

impl Token {
    /// Its readings: one, for what is not a letter.
    fn readings(self) -> Vec<String> {
        match self {
            Token::Letters(readings) => readings.iter().map(|&r| r.to_owned()).collect(),
            Token::Other(other) => vec![other.to_string()],
        }
    }

    /// Whether it ends the first akshara: a vowel, or no letter at all.
    fn ends_akshara(self) -> bool {
        match self {
            Token::Letters(readings) => readings
                .iter()
                .any(|reading| reading.chars().any(|c| "aeiouāīūṛ".contains(c))),
            Token::Other(_) => true,
        }
    }
}

/// A name's spellings as [`CASUAL`] lists them, case folded.
fn tokens(name: &str) -> Result<Vec<Token>, Error> {
    let lower = name.to_lowercase();
    let mut rest = lower.as_str();
    let mut tokens = Vec::new();
    while let Some(next) = rest.chars().next() {
        if let Some((spelled, readings)) = CASUAL.iter().find(|(s, _)| rest.starts_with(s)) {
            tokens.push(Token::Letters(readings));
            rest = &rest[spelled.len()..];
            continue;
        }
        if next.is_alphabetic() {
            return Err(
                Error::invalid_arg(format!("a name read as CASUAL has no letter {next}"))
                    .with_field("name")
                    .with_hint("pass the name in Devanagari, or spell it in a declared scheme"),
            );
        }
        tokens.push(Token::Other(next));
        rest = &rest[next.len_utf8()..];
    }
    Ok(tokens)
}

/// Every spelling of a name's first akshara the list allows, each with
/// the rest of the name in its plainest reading. Only the first akshara
/// decides a cell, so only it is read every way: its consonants and its
/// vowel, and nothing past them.
fn casual(name: &str) -> Result<Vec<String>, Error> {
    let tokens = tokens(name)?;
    let first = tokens
        .iter()
        .position(|token| token.ends_akshara())
        .map_or(tokens.len(), |at| at + 1);
    let (head, tail) = tokens.split_at(first);
    let rest: String = tail
        .iter()
        .filter_map(|token| token.readings().into_iter().next())
        .collect();
    let heads = head.iter().fold(vec![String::new()], |heads, token| {
        heads
            .iter()
            .flat_map(|so_far| {
                token
                    .readings()
                    .into_iter()
                    .map(move |reading| format!("{so_far}{reading}"))
            })
            .collect()
    });
    Ok(heads
        .into_iter()
        .filter_map(|head| to_devanagari(&format!("{head}{rest}"), Scheme::Iast).ok())
        .map(|spelled| unclosed(&spelled))
        .collect())
}

/// A word's last consonant left open, as Nepali and Hindi write a name
/// whose final `a` their speech drops ("Ram" is रम, not रम्).
fn unclosed(spelled: &str) -> String {
    let letters: Vec<char> = spelled.chars().collect();
    letters
        .iter()
        .enumerate()
        .filter(|&(at, &letter)| {
            letter != VIRAMA
                || letters
                    .get(at + 1)
                    .is_some_and(|&next| consonant_of(next).is_some())
        })
        .map(|(_, &letter)| letter)
        .collect()
}
