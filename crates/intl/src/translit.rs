//! Transliteration between the scripts a Sanskrit or Nepali term is
//! written in: Devanagari to IAST, and a Latin scheme's spelling (IAST,
//! ISO 15919, Harvard-Kyoto, ITRANS) back to Devanagari, from one table
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
///
/// ```
/// use teistro_intl::translit::{Script, transliterate};
///
/// assert_eq!(transliterate("सूर्य", Script::Devanagari, Script::Iast)?, "sūrya");
/// # Ok::<(), teistro_intl::translit::Unsupported>(())
/// ```
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
    ///
    /// ```
    /// use teistro_intl::translit::Script;
    ///
    /// assert_eq!(Script::from_key("deva"), Some(Script::Devanagari));
    /// assert_eq!(Script::from_key("taml"), None);
    /// ```
    #[must_use]
    pub fn from_key(key: &str) -> Option<Script> {
        match key.to_ascii_lowercase().as_str() {
            "deva" | "devanagari" => Some(Script::Devanagari),
            "iast" | "latn" | "latin" => Some(Script::Iast),
            _ => None,
        }
    }

    /// The key the tag uses.
    ///
    /// ```
    /// use teistro_intl::translit::Script;
    ///
    /// assert_eq!(Script::Iast.key(), "iast");
    /// assert_eq!(Script::from_key(Script::Devanagari.key()), Some(Script::Devanagari));
    /// ```
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
///
/// ```
/// use teistro_intl::translit::{Script, transliterate};
///
/// // `transliterate` writes Devanagari to IAST; a Latin spelling is read
/// // back by `to_devanagari`, under a declared scheme.
/// let refused = transliterate("sūrya", Script::Iast, Script::Devanagari).unwrap_err();
/// assert_eq!((refused.from, refused.to), (Script::Iast, Script::Devanagari));
/// assert_eq!(refused.to_string(), "no transliteration from iast to deva");
/// ```
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

/// The Latin schemes a Sanskrit or Nepali word is spelled in, each read
/// exactly into Devanagari by [`to_devanagari`] (`03-design/namakarana.md`
/// §2.5, C772).
///
/// Every scheme is a column of one table, the table [`devanagari_to_iast`]
/// writes IAST from, so a letter is spelled in one place.
///
/// ```
/// use teistro_intl::translit::{Scheme, to_devanagari};
///
/// for (spelled, scheme) in [
///     ("kṛṣṇa", Scheme::Iast),
///     ("kr̥ṣṇa", Scheme::Iso15919),
///     ("kRSNa", Scheme::HarvardKyoto),
///     ("kRRiShNa", Scheme::Itrans),
/// ] {
///     assert_eq!(to_devanagari(spelled, scheme)?, "कृष्ण");
/// }
/// # Ok::<(), teistro_intl::translit::Unfit>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Scheme {
    /// The International Alphabet of Sanskrit Transliteration, read with
    /// its case folded.
    Iast,
    /// ISO 15919: IAST with ṁ, r̥, l̥, ē and ō, read with its case folded.
    Iso15919,
    /// Harvard-Kyoto, in ASCII; its capitals are letters, so its case is
    /// read as written.
    HarvardKyoto,
    /// ITRANS 5.3, in ASCII; its capitals are letters, so its case is
    /// read as written.
    Itrans,
}

impl Scheme {
    /// The four schemes, in their columns' order.
    ///
    /// ```
    /// use teistro_intl::translit::Scheme;
    ///
    /// assert_eq!(Scheme::ALL[0], Scheme::Iast);
    /// ```
    pub const ALL: [Scheme; 4] = [
        Scheme::Iast,
        Scheme::Iso15919,
        Scheme::HarvardKyoto,
        Scheme::Itrans,
    ];

    /// The scheme's name, as a refusal names it.
    ///
    /// ```
    /// use teistro_intl::translit::Scheme;
    ///
    /// assert_eq!(Scheme::HarvardKyoto.name(), "Harvard-Kyoto");
    /// ```
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Scheme::Iast => "IAST",
            Scheme::Iso15919 => "ISO 15919",
            Scheme::HarvardKyoto => "Harvard-Kyoto",
            Scheme::Itrans => "ITRANS",
        }
    }

    /// Whether the scheme's case is folded before it is read: IAST's and
    /// ISO 15919's are, the ASCII schemes' capitals being letters.
    const fn folds_case(self) -> bool {
        matches!(self, Scheme::Iast | Scheme::Iso15919)
    }

    /// The scheme's column in [`Spelled`].
    const fn column(self) -> usize {
        match self {
            Scheme::Iast => 0,
            Scheme::Iso15919 => 1,
            Scheme::HarvardKyoto => 2,
            Scheme::Itrans => 3,
        }
    }
}

impl fmt::Display for Scheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// A letter in each scheme's column, in [`Scheme::ALL`]'s order: the
/// spelling written first, then any other the scheme reads as the same
/// letter; an empty column is a letter the scheme does not spell.
type Spelled = [&'static [&'static str]; 4];

/// The vowels: the letter a word may begin with, and the sign that
/// replaces a consonant's inherent `a` (none for `a` itself, and no letter
/// for the one sign written only after a consonant).
const VOWELS: [(Option<char>, Option<char>, Spelled); 17] = [
    (Some('अ'), None, [&["a"], &["a"], &["a"], &["a"]]),
    (Some('आ'), Some('ा'), [&["ā"], &["ā"], &["A"], &["aa", "A"]]),
    (Some('इ'), Some('ि'), [&["i"], &["i"], &["i"], &["i"]]),
    (Some('ई'), Some('ी'), [&["ī"], &["ī"], &["I"], &["ii", "I"]]),
    (Some('उ'), Some('ु'), [&["u"], &["u"], &["u"], &["u"]]),
    (Some('ऊ'), Some('ू'), [&["ū"], &["ū"], &["U"], &["uu", "U"]]),
    (
        Some('ऋ'),
        Some('ृ'),
        [&["ṛ"], &["r\u{325}"], &["R"], &["RRi", "R^i"]],
    ),
    (
        Some('ॠ'),
        Some('ॄ'),
        [&["ṝ"], &["r\u{325}\u{304}"], &["RR"], &["RRI", "R^I"]],
    ),
    (
        Some('ऌ'),
        Some('ॢ'),
        [&["ḷ"], &["l\u{325}"], &["lR"], &["LLi", "L^i"]],
    ),
    (
        Some('ॡ'),
        Some('ॣ'),
        [&["ḹ"], &["l\u{325}\u{304}"], &["lRR"], &["LLI", "L^I"]],
    ),
    (Some('ए'), Some('े'), [&["e"], &["ē"], &["e"], &["e"]]),
    (Some('ऐ'), Some('ै'), [&["ai"], &["ai"], &["ai"], &["ai"]]),
    (Some('ओ'), Some('ो'), [&["o"], &["ō"], &["o"], &["o"]]),
    (Some('औ'), Some('ौ'), [&["au"], &["au"], &["au"], &["au"]]),
    (Some('ऍ'), Some('ॅ'), [&["ê"], &["ê"], &[], &[]]),
    (Some('ऑ'), Some('ॉ'), [&["ô"], &["ô"], &[], &[]]),
    (None, Some('ऺ'), [&["ê"], &[], &[], &[]]),
];

/// The consonants, without their inherent `a`.
const CONSONANTS: [(char, Spelled); 48] = [
    ('क', [&["k"], &["k"], &["k"], &["k"]]),
    ('ख', [&["kh"], &["kh"], &["kh"], &["kh"]]),
    ('ग', [&["g"], &["g"], &["g"], &["g"]]),
    ('घ', [&["gh"], &["gh"], &["gh"], &["gh"]]),
    ('ङ', [&["ṅ"], &["ṅ"], &["G"], &["~N", "N^"]]),
    ('च', [&["c"], &["c"], &["c"], &["ch"]]),
    ('छ', [&["ch"], &["ch"], &["ch"], &["Ch", "chh"]]),
    ('ज', [&["j"], &["j"], &["j"], &["j"]]),
    ('झ', [&["jh"], &["jh"], &["jh"], &["jh"]]),
    ('ञ', [&["ñ"], &["ñ"], &["J"], &["~n", "JN"]]),
    ('ट', [&["ṭ"], &["ṭ"], &["T"], &["T"]]),
    ('ठ', [&["ṭh"], &["ṭh"], &["Th"], &["Th"]]),
    ('ड', [&["ḍ"], &["ḍ"], &["D"], &["D"]]),
    ('ढ', [&["ḍh"], &["ḍh"], &["Dh"], &["Dh"]]),
    ('ण', [&["ṇ"], &["ṇ"], &["N"], &["N"]]),
    ('त', [&["t"], &["t"], &["t"], &["t"]]),
    ('थ', [&["th"], &["th"], &["th"], &["th"]]),
    ('द', [&["d"], &["d"], &["d"], &["d"]]),
    ('ध', [&["dh"], &["dh"], &["dh"], &["dh"]]),
    ('न', [&["n"], &["n"], &["n"], &["n"]]),
    ('\u{929}', [&["ṉ"], &["ṉ"], &[], &[]]),
    ('प', [&["p"], &["p"], &["p"], &["p"]]),
    ('फ', [&["ph"], &["ph"], &["ph"], &["ph"]]),
    ('ब', [&["b"], &["b"], &["b"], &["b"]]),
    ('भ', [&["bh"], &["bh"], &["bh"], &["bh"]]),
    ('म', [&["m"], &["m"], &["m"], &["m"]]),
    ('य', [&["y"], &["y"], &["y"], &["y"]]),
    ('र', [&["r"], &["r"], &["r"], &["r"]]),
    ('\u{931}', [&["ṟ"], &["ṟ"], &[], &[]]),
    ('ल', [&["l"], &["l"], &["l"], &["l"]]),
    ('ळ', [&["ḷ"], &["ḷ"], &[], &["L", "ld"]]),
    ('\u{934}', [&["ḻ"], &["ḻ"], &[], &[]]),
    ('व', [&["v"], &["v"], &["v"], &["v", "w"]]),
    ('श', [&["ś"], &["ś"], &["z"], &["sh"]]),
    ('ष', [&["ṣ"], &["ṣ"], &["S"], &["Sh", "shh"]]),
    ('स', [&["s"], &["s"], &["s"], &["s"]]),
    ('ह', [&["h"], &["h"], &["h"], &["h"]]),
    // The nukta letters, as Nepali and Hindi write borrowed sounds.
    ('\u{958}', [&["q"], &["q"], &[], &["q"]]),
    ('\u{959}', [&["k͟h"], &["ḵẖ"], &[], &["K"]]),
    ('\u{95a}', [&["ġ"], &["ġ"], &[], &["G"]]),
    ('\u{95b}', [&["z"], &["z"], &[], &["z"]]),
    ('\u{95c}', [&["ṛ"], &["ṛ"], &[], &[".D"]]),
    ('\u{95d}', [&["ṛh"], &["ṛh"], &[], &[".Dh"]]),
    ('\u{95e}', [&["f"], &["f"], &[], &["f"]]),
    ('\u{95f}', [&["ẏ"], &["ẏ"], &[], &["Y"]]),
    ('ॹ', [&["z"], &[], &[], &[]]),
    ('ॺ', [&["y"], &[], &[], &[]]),
    ('ॻ', [&["g"], &[], &[], &[]]),
];

/// The marks a syllable may carry, and the punctuation of the script.
const MARKS: [(char, Spelled); 9] = [
    ('ं', [&["ṃ"], &["ṁ"], &["M"], &["M", ".n"]]),
    ('ः', [&["ḥ"], &["ḥ"], &["H"], &["H"]]),
    ('ँ', [&["m̐"], &["m̐"], &[], &[".N"]]),
    ('ऽ', [&["'"], &["'"], &["'"], &[".a"]]),
    ('॑', [&[""], &[], &[], &[]]),
    ('॒', [&[""], &[], &[], &[]]),
    ('।', [&["."], &[], &[], &[]]),
    ('॥', [&[".."], &[], &[], &[]]),
    ('ॐ', [&["oṃ"], &[], &[], &[]]),
];

/// What IAST writes and does not read back, because another letter's
/// spelling is the same or begins it: ढ़'s `ṛh` would take the `ṛh` of
/// `bṛhaspati`, and the punctuation and ॐ are not a name's letters. (ड़'s
/// `ṛ` and ळ's `ḷ` need no entry: the vowel spelled the same is the
/// earlier row, and the earlier row is read.)
const WRITTEN_ONLY: [(Scheme, char); 4] = [
    (Scheme::Iast, '\u{95d}'),
    (Scheme::Iast, '।'),
    (Scheme::Iast, '॥'),
    (Scheme::Iast, 'ॐ'),
];

/// The conjuncts a scheme spells as one letter.
const CONJUNCTS: [(Scheme, &str, [char; 2]); 3] = [
    (Scheme::Itrans, "x", ['क', 'ष']),
    (Scheme::Itrans, "GY", ['ज', 'ञ']),
    (Scheme::Itrans, "dny", ['ज', 'ञ']),
];

/// A combining mark composed onto the letter before it, so a name typed
/// decomposed reads as the same name precomposed. A pair with no
/// precomposed letter (ISO 15919's r̥) stays as typed, and the table
/// spells it decomposed.
const COMPOSED: [(char, char, char); 19] = [
    ('a', '\u{304}', 'ā'),
    ('i', '\u{304}', 'ī'),
    ('u', '\u{304}', 'ū'),
    ('e', '\u{304}', 'ē'),
    ('o', '\u{304}', 'ō'),
    ('r', '\u{323}', 'ṛ'),
    ('ṛ', '\u{304}', 'ṝ'),
    ('l', '\u{323}', 'ḷ'),
    ('ḷ', '\u{304}', 'ḹ'),
    ('n', '\u{307}', 'ṅ'),
    ('n', '\u{303}', 'ñ'),
    ('t', '\u{323}', 'ṭ'),
    ('d', '\u{323}', 'ḍ'),
    ('n', '\u{323}', 'ṇ'),
    ('s', '\u{301}', 'ś'),
    ('s', '\u{323}', 'ṣ'),
    ('m', '\u{323}', 'ṃ'),
    ('m', '\u{307}', 'ṁ'),
    ('h', '\u{323}', 'ḥ'),
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
/// ```
/// use teistro_intl::translit::{Script, transliterate};
///
/// // The inherent vowel, the virama and the visarga.
/// assert_eq!(transliterate("राजयोगः", Script::Devanagari, Script::Iast)?, "rājayogaḥ");
/// assert_eq!(transliterate("कृष्ण", Script::Devanagari, Script::Iast)?, "kṛṣṇa");
/// # Ok::<(), teistro_intl::translit::Unsupported>(())
/// ```
///
/// # Errors
///
/// A pair with no table: `transliterate` writes Devanagari to IAST only,
/// and [`to_devanagari`] reads a Latin scheme back.
pub fn transliterate(text: &str, from: Script, to: Script) -> Result<String, Unsupported> {
    match (from, to) {
        (Script::Devanagari, Script::Iast) => Ok(devanagari_to_iast(text)),
        (from, to) if from == to => Ok(text.to_string()),
        (from, to) => Err(Unsupported { from, to }),
    }
}

/// A Latin name a scheme does not spell: the letter it stopped at.
///
/// ```
/// use teistro_intl::translit::{Scheme, to_devanagari};
///
/// // IAST has no w.
/// let refused = to_devanagari("Wasim", Scheme::Iast).unwrap_err();
/// assert_eq!((refused.scheme, refused.letter.as_str()), (Scheme::Iast, "w"));
/// assert_eq!(refused.to_string(), "IAST has no letter w");
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unfit {
    /// The scheme read under.
    pub scheme: Scheme,
    /// The first letter it does not spell, as it was read (case folded
    /// for IAST and ISO 15919).
    pub letter: String,
}

impl fmt::Display for Unfit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} has no letter {}", self.scheme, self.letter)
    }
}

impl std::error::Error for Unfit {}

/// What a Latin spelling reads as: Devanagari's own units, which
/// [`to_devanagari`] joins into syllables.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Unit {
    /// A vowel: its letter, and its sign after a consonant (none for `a`).
    Vowel(char, Option<char>),
    /// A consonant, or a conjunct's consonants.
    Consonants(char, Option<char>),
    /// A mark.
    Mark(char),
}

/// A Latin text read exactly under a scheme into Devanagari: each letter
/// the longest spelling the scheme's column holds there (the earlier row
/// when two rows spell it alike), a consonant taking the vowel after it
/// as its sign and a virama when none follows. What is not a letter
/// (space, hyphen, digit) passes through, and ends a word.
///
/// ```
/// use teistro_intl::translit::{Scheme, to_devanagari};
///
/// assert_eq!(to_devanagari("Rāma", Scheme::Iast)?, "राम");
/// // A final consonant is written with its virama: the scheme says so.
/// assert_eq!(to_devanagari("rām", Scheme::Iast)?, "राम्");
/// // Decomposed marks read as their precomposed letters.
/// assert_eq!(to_devanagari("Ra\u{304}ma", Scheme::Iast)?, "राम");
/// assert_eq!(to_devanagari("rAmaH", Scheme::HarvardKyoto)?, "रामः");
/// assert_eq!(to_devanagari("j~nAna", Scheme::Itrans)?, "ज्ञान");
/// # Ok::<(), teistro_intl::translit::Unfit>(())
/// ```
///
/// # Errors
///
/// A letter, or a combining mark, the scheme does not spell.
pub fn to_devanagari(text: &str, scheme: Scheme) -> Result<String, Unfit> {
    let text = if scheme.folds_case() {
        composed(&text.to_lowercase())
    } else {
        text.to_owned()
    };
    let readings = READINGS.get(scheme.column()).map_or(&[][..], Vec::as_slice);
    let mut out = String::with_capacity(text.len() * 3);
    // Whether the last unit written was a consonant still waiting for its
    // vowel.
    let mut open = false;
    let mut rest = text.as_str();
    while let Some(next) = rest.chars().next() {
        let Some((spelled, unit)) = readings
            .iter()
            .filter(|(spelled, _)| rest.starts_with(spelled.as_str()))
            .fold(None::<&(String, Unit)>, |best, row| match best {
                Some(best) if best.0.len() >= row.0.len() => Some(best),
                _ => Some(row),
            })
            .map(|(spelled, unit)| (spelled.len(), *unit))
        else {
            if next.is_alphabetic() || is_combining(next) {
                return Err(Unfit {
                    scheme,
                    letter: next.to_string(),
                });
            }
            if open {
                out.push(VIRAMA);
            }
            open = false;
            out.push(next);
            rest = &rest[next.len_utf8()..];
            continue;
        };
        rest = &rest[spelled..];
        match unit {
            Unit::Consonants(first, second) => {
                if open {
                    out.push(VIRAMA);
                }
                out.push(first);
                if let Some(second) = second {
                    out.push(VIRAMA);
                    out.push(second);
                }
                open = true;
            }
            Unit::Vowel(letter, sign) => {
                match (open, sign) {
                    (true, Some(sign)) => out.push(sign),
                    (true, None) => {}
                    (false, _) => out.push(letter),
                }
                open = false;
            }
            Unit::Mark(mark) => {
                if open {
                    out.push(VIRAMA);
                }
                out.push(mark);
                open = false;
            }
        }
    }
    if open {
        out.push(VIRAMA);
    }
    Ok(out)
}

/// Whether a character is a combining mark of the Latin schemes' range,
/// which never stands alone.
fn is_combining(letter: char) -> bool {
    matches!(letter, '\u{300}'..='\u{36F}')
}

/// The text with each [`COMPOSED`] pair made its precomposed letter.
fn composed(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for letter in text.chars() {
        let made = out.chars().next_back().and_then(|before| {
            COMPOSED
                .iter()
                .find(|(base, mark, _)| *base == before && *mark == letter)
                .map(|(_, _, made)| *made)
        });
        match made {
            Some(made) => {
                out.pop();
                out.push(made);
            }
            None => out.push(letter),
        }
    }
    out
}

/// The tables as maps, built once.
struct Tables {
    vowels: BTreeMap<char, &'static str>,
    signs: BTreeMap<char, &'static str>,
    consonants: BTreeMap<char, &'static str>,
    marks: BTreeMap<char, &'static str>,
    digits: BTreeMap<char, char>,
}

/// A row's spelling in a scheme's column, the one it writes.
fn written(spelled: &Spelled, scheme: Scheme) -> Option<&'static str> {
    spelled
        .get(scheme.column())
        .and_then(|column| column.first())
        .copied()
}

static TABLES: LazyLock<Tables> = LazyLock::new(|| {
    let iast = |spelled: &Spelled| written(spelled, Scheme::Iast);
    Tables {
        vowels: VOWELS
            .iter()
            .filter_map(|(letter, _, spelled)| Some(((*letter)?, iast(spelled)?)))
            .collect(),
        signs: VOWELS
            .iter()
            .filter_map(|(_, sign, spelled)| Some(((*sign)?, iast(spelled)?)))
            .collect(),
        consonants: CONSONANTS
            .iter()
            .filter_map(|(letter, spelled)| Some((*letter, iast(spelled)?)))
            .collect(),
        marks: MARKS
            .iter()
            .filter_map(|(mark, spelled)| Some((*mark, iast(spelled)?)))
            .collect(),
        digits: DIGITS.iter().copied().collect(),
    }
});

/// Each scheme's spellings and what each reads as, in the table's order,
/// built once: the vowels, the consonants, the marks, then the conjuncts,
/// leaving out what a scheme writes only and the empty spellings of the
/// accents.
static READINGS: LazyLock<[Vec<(String, Unit)>; 4]> = LazyLock::new(|| {
    Scheme::ALL.map(|scheme| {
        let rows = VOWELS
            .iter()
            .filter_map(|(letter, sign, spelled)| Some((spelled, Unit::Vowel((*letter)?, *sign))))
            .chain(
                CONSONANTS
                    .iter()
                    .map(|(letter, spelled)| (spelled, Unit::Consonants(*letter, None))),
            )
            .chain(
                MARKS
                    .iter()
                    .map(|(mark, spelled)| (spelled, Unit::Mark(*mark))),
            )
            .filter(|(_, unit)| {
                let letter = match unit {
                    Unit::Vowel(letter, _) | Unit::Consonants(letter, _) | Unit::Mark(letter) => {
                        *letter
                    }
                };
                !WRITTEN_ONLY.contains(&(scheme, letter))
            })
            .flat_map(|(spelled, unit)| {
                spelled
                    .get(scheme.column())
                    .into_iter()
                    .flat_map(|column| column.iter())
                    .map(move |one| ((*one).to_owned(), unit))
            });
        let conjuncts = CONJUNCTS.iter().filter(|(of, _, _)| *of == scheme).map(
            |(_, spelled, [first, second])| {
                (
                    (*spelled).to_owned(),
                    Unit::Consonants(*first, Some(*second)),
                )
            },
        );
        rows.chain(conjuncts)
            .filter(|(spelled, _)| !spelled.is_empty())
            .collect()
    })
});

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
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
    fn a_decomposed_nukta_is_folded_onto_its_letter() {
        for (decomposed, iast) in [
            ("क\u{93c}", "qa"),
            ("ख\u{93c}", "k͟ha"),
            ("ग\u{93c}", "ġa"),
            ("ज\u{93c}", "za"),
            ("ड\u{93c}", "ṛa"),
            ("ढ\u{93c}", "ṛha"),
            ("फ\u{93c}", "fa"),
            ("य\u{93c}", "ẏa"),
            ("न\u{93c}", "ṉa"),
            ("र\u{93c}", "ṟa"),
            ("स\u{93c}", "sa"),
            ("\u{93c}", ""),
        ] {
            assert_eq!(devanagari_to_iast(decomposed), iast, "{decomposed}");
        }
        // A virama with nothing before it writes nothing.
        assert_eq!(devanagari_to_iast("्"), "");
    }

    #[test]
    fn what_is_not_devanagari_passes_through() {
        assert_eq!(devanagari_to_iast("Sun (सूर्य)"), "Sun (sūrya)");
        assert_eq!(devanagari_to_iast(""), "");
        assert_eq!(devanagari_to_iast("Jupiter"), "Jupiter");
    }

    #[test]
    fn every_scheme_reads_back_what_devanagari_writes() {
        // Every word the IAST tests above write, read back under each
        // scheme's own spelling.
        for (deva, iast, iso, hk, itrans) in [
            ("सूर्य", "sūrya", "sūrya", "sUrya", "suurya"),
            ("कृष्ण", "kṛṣṇa", "kr̥ṣṇa", "kRSNa", "kR^iShNa"),
            (
                "बृहस्पति",
                "bṛhaspati",
                "br̥haspati",
                "bRhaspati",
                "bRRihaspati",
            ),
            (
                "राजयोगः",
                "rājayogaḥ",
                "rājayōgaḥ",
                "rAjayogaH",
                "rAjayogaH",
            ),
            ("मङ्गल", "maṅgala", "maṅgala", "maGgala", "ma~Ngala"),
            ("चञ्चल", "cañcala", "cañcala", "caJcala", "cha~nchala"),
            ("शिवः", "śivaḥ", "śivaḥ", "zivaH", "shivaH"),
            ("षष्ठ", "ṣaṣṭha", "ṣaṣṭha", "SaSTha", "ShaShTha"),
            ("ऐश्वर्य", "aiśvarya", "aiśvarya", "aizvarya", "aishvarya"),
            ("औषधि", "auṣadhi", "auṣadhi", "auSadhi", "aushhadhi"),
            ("संस्कृत", "saṃskṛta", "saṁskr̥ta", "saMskRta", "saMskRRita"),
            ("ॠ", "ṝ", "r̥̄", "RR", "RRI"),
            ("ऌ", "ḷ", "l̥", "lR", "LLi"),
        ] {
            for (spelled, scheme) in [
                (iast, Scheme::Iast),
                (iso, Scheme::Iso15919),
                (hk, Scheme::HarvardKyoto),
                (itrans, Scheme::Itrans),
            ] {
                assert_eq!(
                    to_devanagari(spelled, scheme).unwrap(),
                    deva,
                    "{spelled} in {scheme}"
                );
            }
            assert_eq!(devanagari_to_iast(deva), iast);
        }
    }

    #[test]
    fn a_scheme_reads_its_other_spellings_and_conjuncts() {
        for (spelled, deva) in [
            ("kShamA", "क्षमा"),
            ("xamA", "क्षमा"),
            ("GYAna", "ज्ञान"),
            ("dnyAna", "ज्ञान"),
            ("chhAyA", "छाया"),
            ("ChAyA", "छाया"),
            ("wiShNu", "विष्णु"),
            ("N^ga", "ङ्ग"),
            ("JNa", "ञ"),
            ("ka.n", "कं"),
            (".Dha", "\u{95d}"),
            ("so.a", "सोऽ"),
            ("ka.N", "कँ"),
            ("ld", "ळ्"),
        ] {
            assert_eq!(
                to_devanagari(spelled, Scheme::Itrans).unwrap(),
                deva,
                "{spelled}"
            );
        }
        // The nukta letters ISO 15919 spells, and IAST's.
        assert_eq!(
            to_devanagari("ḵẖān", Scheme::Iso15919).unwrap(),
            "\u{959}ान्"
        );
        assert_eq!(to_devanagari("ṛh", Scheme::Iso15919).unwrap(), "\u{95d}्");
        assert_eq!(to_devanagari("zoyā", Scheme::Iast).unwrap(), "\u{95b}ोया");
        assert_eq!(
            to_devanagari("fātimā", Scheme::Iast).unwrap(),
            "\u{95e}ातिमा"
        );
    }

    #[test]
    fn a_decomposed_name_reads_as_its_precomposed_spelling() {
        for (decomposed, precomposed) in [
            ("Ra\u{304}ma", "Rāma"),
            ("Kr\u{323}s\u{323}n\u{323}a", "Kṛṣṇa"),
            ("S\u{301}iva", "Śiva"),
            ("san\u{303}jaya", "sañjaya"),
            ("man\u{307}gala", "maṅgala"),
            ("r\u{323}\u{304}", "ṝ"),
            ("l\u{323}\u{304}", "ḹ"),
            ("t\u{323}hd\u{323}", "ṭhḍ"),
            ("i\u{304}u\u{304}", "īū"),
            ("m\u{323}h\u{323}", "ṃḥ"),
        ] {
            assert_eq!(
                to_devanagari(decomposed, Scheme::Iast).unwrap(),
                to_devanagari(precomposed, Scheme::Iast).unwrap(),
                "{precomposed}"
            );
        }
        for (decomposed, precomposed) in [
            ("De\u{304}va", "Dēva"),
            ("So\u{304}ma", "Sōma"),
            ("sam\u{307}", "saṁ"),
        ] {
            assert_eq!(
                to_devanagari(decomposed, Scheme::Iso15919).unwrap(),
                to_devanagari(precomposed, Scheme::Iso15919).unwrap(),
                "{precomposed}"
            );
        }
        // A mark with nothing to compose onto stays, and IAST refuses it.
        assert_eq!(
            to_devanagari("\u{304}a", Scheme::Iast).unwrap_err().letter,
            "\u{304}"
        );
    }

    #[test]
    fn a_letter_a_scheme_does_not_spell_is_refused_by_name() {
        for (spelled, scheme, letter) in [
            ("Wasim", Scheme::Iast, "w"),
            ("Deva", Scheme::Iso15919, "e"),
            ("Rama", Scheme::Itrans, "R"),
            ("xena", Scheme::HarvardKyoto, "x"),
        ] {
            let refused = to_devanagari(spelled, scheme).unwrap_err();
            assert_eq!((refused.scheme, refused.letter.as_str()), (scheme, letter));
        }
        assert_eq!(
            to_devanagari("Wasim", Scheme::Iast)
                .unwrap_err()
                .to_string(),
            "IAST has no letter w"
        );
        assert_eq!(Scheme::Iso15919.to_string(), "ISO 15919");
        assert_eq!(Scheme::Itrans.name(), "ITRANS");
    }

    #[test]
    fn what_is_not_a_letter_passes_through_and_ends_a_word() {
        assert_eq!(
            to_devanagari("rām-prasād 2", Scheme::Iast).unwrap(),
            "राम्-प्रसाद् 2"
        );
        assert_eq!(to_devanagari("", Scheme::Iast).unwrap(), "");
        // A mark after a consonant closes it with a virama.
        assert_eq!(to_devanagari("kḥ", Scheme::Iast).unwrap(), "क्ः");
        // IAST writes ॐ and ढ़ and does not read them back.
        assert_eq!(to_devanagari("oṃ", Scheme::Iast).unwrap(), "ओं");
        assert_eq!(to_devanagari("bṛhat", Scheme::Iast).unwrap(), "बृहत्");
        assert_eq!(devanagari_to_iast("\u{95d}"), "ṛha");
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
