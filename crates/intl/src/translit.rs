//! Transliteration between the scripts a Sanskrit or Nepali term is
//! written in: Devanagari to IAST today, and the reverse, from one table
//! rather than from code
//! (`02-architecture/03-localization-architecture.md`, "Axes that are not
//! the language").
//!
//! It is what derives `sa-Latn` from `sa-Deva`, so a Latin-script reader
//! of Sanskrit terms gets every entity without anyone writing them twice
//! (`teistro-intl derive`).
//!
//! Devanagari writes a consonant with an inherent `a` that a vowel sign
//! or a virama replaces, which is the whole of the algorithm: a
//! consonant, then its sign if it has one, its `a` if it has neither, and
//! nothing when a virama follows.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::LazyLock;

/// A script a term is written in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Script {
    /// Devanagari, as Sanskrit, Nepali and Hindi are written.
    Devanagari,
    /// The International Alphabet of Sanskrit Transliteration.
    Iast,
}

impl Script {
    /// The script a tag names (`deva`, `iast`), or `None` for one this
    /// build does not know.
    #[must_use]
    pub fn from_key(key: &str) -> Option<Script> {
        match key.to_ascii_lowercase().as_str() {
            "deva" | "devanagari" => Some(Script::Devanagari),
            "iast" | "latn" | "latin" => Some(Script::Iast),
            _ => None,
        }
    }

    /// The key the tag uses.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Script::Devanagari => "deva",
            Script::Iast => "iast",
        }
    }
}

impl fmt::Display for Script {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.key())
    }
}

/// A pair a script conversion does not know how to make.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unsupported {
    /// The script asked from.
    pub from: Script,
    /// The script asked to.
    pub to: Script,
}

impl fmt::Display for Unsupported {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "no transliteration from {} to {}", self.from, self.to)
    }
}

impl std::error::Error for Unsupported {}

/// The independent vowels: the letter a word may begin with.
const VOWELS: [(char, &str); 16] = [
    ('अ', "a"),
    ('आ', "ā"),
    ('इ', "i"),
    ('ई', "ī"),
    ('उ', "u"),
    ('ऊ', "ū"),
    ('ऋ', "ṛ"),
    ('ॠ', "ṝ"),
    ('ऌ', "ḷ"),
    ('ॡ', "ḹ"),
    ('ए', "e"),
    ('ऐ', "ai"),
    ('ओ', "o"),
    ('औ', "au"),
    ('ऍ', "ê"),
    ('ऑ', "ô"),
];

/// The vowel signs, which replace a consonant's inherent `a`.
const SIGNS: [(char, &str); 16] = [
    ('ा', "ā"),
    ('ि', "i"),
    ('ी', "ī"),
    ('ु', "u"),
    ('ू', "ū"),
    ('ृ', "ṛ"),
    ('ॄ', "ṝ"),
    ('ॢ', "ḷ"),
    ('ॣ', "ḹ"),
    ('े', "e"),
    ('ै', "ai"),
    ('ो', "o"),
    ('ौ', "au"),
    ('ॅ', "ê"),
    ('ॉ', "ô"),
    ('ऺ', "ê"),
];

/// The consonants, without their inherent `a`.
const CONSONANTS: [(char, &str); 48] = [
    ('क', "k"),
    ('ख', "kh"),
    ('ग', "g"),
    ('घ', "gh"),
    ('ङ', "ṅ"),
    ('च', "c"),
    ('छ', "ch"),
    ('ज', "j"),
    ('झ', "jh"),
    ('ञ', "ñ"),
    ('ट', "ṭ"),
    ('ठ', "ṭh"),
    ('ड', "ḍ"),
    ('ढ', "ḍh"),
    ('ण', "ṇ"),
    ('त', "t"),
    ('थ', "th"),
    ('द', "d"),
    ('ध', "dh"),
    ('न', "n"),
    ('\u{929}', "ṉ"),
    ('प', "p"),
    ('फ', "ph"),
    ('ब', "b"),
    ('भ', "bh"),
    ('म', "m"),
    ('य', "y"),
    ('र', "r"),
    ('\u{931}', "ṟ"),
    ('ल', "l"),
    ('ळ', "ḷ"),
    ('\u{934}', "ḻ"),
    ('व', "v"),
    ('श', "ś"),
    ('ष', "ṣ"),
    ('स', "s"),
    ('ह', "h"),
    // The nukta letters, as Nepali and Hindi write borrowed sounds.
    ('\u{958}', "q"),
    ('\u{959}', "k͟h"),
    ('\u{95a}', "ġ"),
    ('\u{95b}', "z"),
    ('\u{95c}', "ṛ"),
    ('\u{95d}', "ṛh"),
    ('\u{95e}', "f"),
    ('\u{95f}', "ẏ"),
    ('ॹ', "z"),
    ('ॺ', "y"),
    ('ॻ', "g"),
];

/// The marks a syllable may carry, and the punctuation of the script.
const MARKS: [(char, &str); 9] = [
    ('ं', "ṃ"),
    ('ः', "ḥ"),
    ('ँ', "m̐"),
    ('ऽ', "'"),
    ('॑', ""),
    ('॒', ""),
    ('।', "."),
    ('॥', ".."),
    ('ॐ', "oṃ"),
];

/// The digits, which every script writes with its own figures.
const DIGITS: [(char, char); 10] = [
    ('०', '0'),
    ('१', '1'),
    ('२', '2'),
    ('३', '3'),
    ('४', '4'),
    ('५', '5'),
    ('६', '6'),
    ('७', '7'),
    ('८', '8'),
    ('९', '9'),
];

/// The anusvara, whose sound is the nasal of whatever follows it.
const ANUSVARA: char = 'ं';

/// The nasal an anusvara stands for before a given letter: the nasal of
/// that letter's class, and the plain mark before a semivowel, a
/// sibilant, `h`, or nothing at all.
fn assimilated(next: Option<char>) -> &'static str {
    match next {
        Some('क'..='ङ') => "ṅ",
        Some('च'..='ञ') => "ñ",
        Some('ट'..='ण') => "ṇ",
        Some('त'..='न') => "n",
        Some('प'..='म') => "m",
        _ => "ṃ",
    }
}

/// The virama, which takes a consonant's inherent `a` away.
const VIRAMA: char = '्';

/// The nukta, which the tables above carry pre-composed; a decomposed one
/// is folded onto the letter before it.
const NUKTA: char = '़';

/// Devanagari to IAST, one syllable at a time. Anything the tables do not
/// know is passed through, so a name in two scripts is transliterated in
/// the half that needs it and left alone in the half that does not.
///
/// ```
/// use teistro_intl::translit::devanagari_to_iast;
///
/// assert_eq!(devanagari_to_iast("सूर्य"), "sūrya");
/// assert_eq!(devanagari_to_iast("मङ्गल"), "maṅgala");
/// assert_eq!(devanagari_to_iast("बृहस्पति"), "bṛhaspati");
/// assert_eq!(devanagari_to_iast("ॐ"), "oṃ");
/// ```
#[must_use]
pub fn devanagari_to_iast(text: &str) -> String {
    let table = &*TABLES;
    let letters: Vec<char> = fold_nukta(text);
    let mut out = String::with_capacity(text.len());
    let mut index = 0;
    while let Some(&letter) = letters.get(index) {
        index += 1;
        if let Some(consonant) = table.consonants.get(&letter) {
            out.push_str(consonant);
            // The inherent `a`, unless a sign or a virama says otherwise.
            match letters.get(index) {
                Some(&VIRAMA) => index += 1,
                Some(next) if table.signs.contains_key(next) => {
                    if let Some(sign) = table.signs.get(next) {
                        out.push_str(sign);
                    }
                    index += 1;
                }
                _ => out.push('a'),
            }
            continue;
        }
        if let Some(vowel) = table.vowels.get(&letter) {
            out.push_str(vowel);
            continue;
        }
        if letter == ANUSVARA {
            // An anusvara before a stop is that stop's own nasal, which
            // is how a Sanskrit or Nepali word is read and written in
            // Latin (`maṅgala`, not `maṃgala`); before anything else it
            // stays the plain mark.
            out.push_str(assimilated(letters.get(index).copied()));
            continue;
        }
        if let Some(mark) = table.marks.get(&letter) {
            out.push_str(mark);
            continue;
        }
        if let Some(digit) = table.digits.get(&letter) {
            out.push(*digit);
            continue;
        }
        if letter == VIRAMA {
            // A virama with no consonant before it is nothing to write.
            continue;
        }
        out.push(letter);
    }
    out
}

/// A decomposed nukta folded onto the letter it follows, so `क` and a
/// nukta read as `क़` does.
fn fold_nukta(text: &str) -> Vec<char> {
    let mut letters: Vec<char> = Vec::with_capacity(text.chars().count());
    for letter in text.chars() {
        if letter != NUKTA {
            letters.push(letter);
            continue;
        }
        let Some(previous) = letters.pop() else {
            continue;
        };
        let composed = match previous {
            'क' => '\u{958}',
            'ख' => '\u{959}',
            'ग' => '\u{95a}',
            'ज' => '\u{95b}',
            'ड' => '\u{95c}',
            'ढ' => '\u{95d}',
            'फ' => '\u{95e}',
            'य' => '\u{95f}',
            'न' => '\u{929}',
            'र' => '\u{931}',
            other => other,
        };
        letters.push(composed);
    }
    letters
}

/// Text from one script into another, or the pair the tables do not know.
///
/// # Errors
///
/// A pair with no table (only Devanagari to IAST is built).
pub fn transliterate(text: &str, from: Script, to: Script) -> Result<String, Unsupported> {
    match (from, to) {
        (Script::Devanagari, Script::Iast) => Ok(devanagari_to_iast(text)),
        (from, to) if from == to => Ok(text.to_string()),
        (from, to) => Err(Unsupported { from, to }),
    }
}

/// The tables as maps, built once.
struct Tables {
    vowels: BTreeMap<char, String>,
    signs: BTreeMap<char, String>,
    consonants: BTreeMap<char, String>,
    marks: BTreeMap<char, String>,
    digits: BTreeMap<char, char>,
}

static TABLES: LazyLock<Tables> = LazyLock::new(|| Tables {
    vowels: VOWELS.iter().map(|(k, v)| (*k, (*v).to_string())).collect(),
    signs: SIGNS.iter().map(|(k, v)| (*k, (*v).to_string())).collect(),
    consonants: CONSONANTS
        .iter()
        .map(|(k, v)| (*k, (*v).to_string()))
        .collect(),
    marks: MARKS.iter().map(|(k, v)| (*k, (*v).to_string())).collect(),
    digits: DIGITS.iter().copied().collect(),
});

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    #[test]
    fn a_syllable_carries_its_inherent_vowel_unless_something_takes_it() {
        assert_eq!(devanagari_to_iast("क"), "ka");
        assert_eq!(devanagari_to_iast("का"), "kā");
        assert_eq!(devanagari_to_iast("क्"), "k");
        assert_eq!(devanagari_to_iast("कि"), "ki");
        assert_eq!(devanagari_to_iast("क्ष"), "kṣa", "a conjunct is a virama");
        assert_eq!(devanagari_to_iast("ज्ञ"), "jña");
    }

    #[test]
    fn the_grahas_read_as_the_texts_write_them() {
        for (deva, iast) in [
            ("सूर्य", "sūrya"),
            ("चन्द्र", "candra"),
            ("मङ्गल", "maṅgala"),
            ("बुध", "budha"),
            ("गुरु", "guru"),
            ("शुक्र", "śukra"),
            ("शनि", "śani"),
            ("राहु", "rāhu"),
            ("केतु", "ketu"),
            ("बृहस्पति", "bṛhaspati"),
        ] {
            assert_eq!(devanagari_to_iast(deva), iast, "{deva}");
        }
    }

    #[test]
    fn an_anusvara_takes_the_sound_of_what_follows_it() {
        assert_eq!(devanagari_to_iast("मंगल"), "maṅgala");
        assert_eq!(devanagari_to_iast("पिंगल"), "piṅgala");
        assert_eq!(devanagari_to_iast("मंगलवार"), "maṅgalavāra");
        assert_eq!(devanagari_to_iast("चंचल"), "cañcala");
        assert_eq!(devanagari_to_iast("कंठ"), "kaṇṭha");
        assert_eq!(devanagari_to_iast("संत"), "santa");
        assert_eq!(devanagari_to_iast("कंप"), "kampa");
        assert_eq!(
            devanagari_to_iast("सिंह"),
            "siṃha",
            "before an `h` the mark stands"
        );
        assert_eq!(devanagari_to_iast("सं"), "saṃ", "and at the end of a word");
    }

    #[test]
    fn the_marks_and_the_digits_carry_over() {
        assert_eq!(devanagari_to_iast("संस्कृत"), "saṃskṛta");
        assert_eq!(devanagari_to_iast("दुःख"), "duḥkha");
        assert_eq!(devanagari_to_iast("ॐ"), "oṃ");
        assert_eq!(devanagari_to_iast("२०७२"), "2072");
        assert_eq!(devanagari_to_iast("अश्विनी"), "aśvinī");
    }

    #[test]
    fn what_is_not_devanagari_passes_through() {
        assert_eq!(devanagari_to_iast("Sun (सूर्य)"), "Sun (sūrya)");
        assert_eq!(devanagari_to_iast(""), "");
        assert_eq!(devanagari_to_iast("Jupiter"), "Jupiter");
    }

    #[test]
    fn a_pair_with_no_table_is_refused_by_name() {
        assert_eq!(
            transliterate("sūrya", Script::Iast, Script::Devanagari)
                .unwrap_err()
                .to_string(),
            "no transliteration from iast to deva"
        );
        assert_eq!(
            transliterate("x", Script::Iast, Script::Iast).ok(),
            Some(String::from("x"))
        );
        assert_eq!(Script::from_key("DEVA"), Some(Script::Devanagari));
        assert_eq!(Script::from_key("tamil"), None);
    }
}
