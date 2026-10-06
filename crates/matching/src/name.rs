//! A name's star and varga from its first syllable, and naam milan: the
//! match of two names (`03-design/matching.md`, C291 to C296).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Nakshatra;
use teistro_core::error::Error;

use crate::{
    AshtaKoota, KootaRules, MatchRole, Native, Porutham, PoruthamRules, ashta_koota, porutham,
};

/// The five vowels the cakra reads, length folded (C291).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Svara {
    A,
    I,
    U,
    E,
    O,
}

const SVARAS: [Svara; 5] = [Svara::A, Svara::I, Svara::U, Svara::E, Svara::O];

/// The heads of the cakra's twenty rows, `None` for the vowels' own row
/// (*Svarodaya*, śatapada cakra vv. 3–4).
const ROW_HEADS: [Option<char>; 20] = [
    None,
    Some('व'),
    Some('क'),
    Some('ह'),
    Some('ड'),
    Some('म'),
    Some('ट'),
    Some('प'),
    Some('र'),
    Some('त'),
    Some('न'),
    Some('य'),
    Some('भ'),
    Some('ज'),
    Some('ख'),
    Some('ग'),
    Some('स'),
    Some('द'),
    Some('च'),
    Some('ल'),
];

/// The three letters set after ku, pu, bhu and du, each read whatever its
/// vowel (vv. 4–6).
const PILLARS: [(char, [char; 3]); 4] = [
    ('क', ['घ', 'ङ', 'छ']),
    ('प', ['ष', 'ण', 'ठ']),
    ('भ', ['ध', 'फ', 'ढ']),
    ('द', ['थ', 'झ', 'ञ']),
];

/// One cell of the cakra.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cell {
    /// A row's head with one vowel.
    Syllable(Option<char>, Svara),
    /// A pillar's letter, with any vowel.
    Letter(char),
}

/// The 112 cells from Krittika's first, in order (v. 8).
fn cells() -> impl Iterator<Item = Cell> {
    ROW_HEADS.into_iter().flat_map(|head| {
        SVARAS.into_iter().flat_map(move |svara| {
            let pillar = PILLARS
                .iter()
                .find(|(at, _)| svara == Svara::U && head == Some(*at))
                .map_or(&[][..], |(_, letters)| &letters[..]);
            std::iter::once(Cell::Syllable(head, svara))
                .chain(pillar.iter().map(|&letter| Cell::Letter(letter)))
        })
    })
}

/// The letters the cakra lacks, read as the letter *Svarodaya*'s
/// sarvatobhadra v. 22 pairs them with (C294).
const PAIRED: [(char, char); 2] = [('ब', 'व'), ('श', 'स')];

/// Which of the eight letter groups a name begins in, by the animal VI.35
/// gives it (C295).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NameVarga {
    /// The vowels.
    Garuda,
    /// ka kha ga gha ṅa.
    Cat,
    /// ca cha ja jha ña.
    Lion,
    /// ṭa ṭha ḍa ḍha ṇa.
    Dog,
    /// ta tha da dha na.
    Serpent,
    /// pa pha ba bha ma.
    Rat,
    /// ya ra la va.
    Deer,
    /// śa ṣa sa ha.
    Sheep,
}

const VARGAS: [NameVarga; 8] = [
    NameVarga::Garuda,
    NameVarga::Cat,
    NameVarga::Lion,
    NameVarga::Dog,
    NameVarga::Serpent,
    NameVarga::Rat,
    NameVarga::Deer,
    NameVarga::Sheep,
];

impl NameVarga {
    /// The varga a name's first letter stands in, `None` for a vowel. The
    /// letter is one [`consonant_of`] passed, so the last arm is ś, ṣ, s
    /// and h.
    fn of(first: Option<char>) -> NameVarga {
        match first {
            None => NameVarga::Garuda,
            Some('क'..='ङ') => NameVarga::Cat,
            Some('च'..='ञ') => NameVarga::Lion,
            Some('ट'..='ण') => NameVarga::Dog,
            Some('त'..='न') => NameVarga::Serpent,
            Some('प'..='म') => NameVarga::Rat,
            Some('य' | 'र' | 'ल' | 'व') => NameVarga::Deer,
            Some(_) => NameVarga::Sheep,
        }
    }

    /// The varga's place in VI.35's order, 0 for Garuda.
    fn place(self) -> usize {
        VARGAS.iter().position(|&v| v == self).unwrap_or(0)
    }
}

/// How two vargas stand (VI.35).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VargaRelation {
    /// One varga: good.
    Same,
    /// Each the 5th from the other, eater and eaten: bad.
    Enemy,
    /// Neither: good.
    Neutral,
}

/// The varga koota of two names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct VargaKoota {
    /// The bride's varga.
    pub bride: NameVarga,
    /// The groom's.
    pub groom: NameVarga,
    /// How they stand.
    pub relation: VargaRelation,
}

/// The varga koota of two vargas.
#[must_use]
pub fn varga_koota(bride: NameVarga, groom: NameVarga) -> VargaKoota {
    let apart = (bride.place() + 8 - groom.place()) % 8;
    let relation = match apart {
        0 => VargaRelation::Same,
        4 => VargaRelation::Enemy,
        _ => VargaRelation::Neutral,
    };
    VargaKoota {
        bride,
        groom,
        relation,
    }
}

/// How a name written in Latin letters is read (C293).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LatinName {
    /// Refused: an English spelling is not guessed at, since its "ch" is
    /// च where IAST's is छ.
    #[default]
    Refuse,
    /// Read as IAST.
    Iast,
}

/// Where a name in Abhijit's row stands when a koota needs one of the 108
/// padas (C292).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AbhijitPada {
    /// Refused, naming this knob: no source read places it.
    #[default]
    Refuse,
    /// Uttarashadha's 4th pada, which holds most of Abhijit's span.
    UttaraAshadha,
    /// Shravana's 1st pada.
    Shravana,
}

/// The readings a name is taken under; each default is the one no source
/// read contradicts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NameRules {
    /// How a Latin spelling is read.
    pub latin: LatinName,
    /// Where Abhijit's syllables stand.
    pub abhijit: AbhijitPada,
}

/// A name's first syllable as the cakra reads it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct NameSyllable {
    /// Its place among the cakra's 112 cells, 0 for a, Krittika's first.
    pub cell: u8,
    /// Its star, or `None` for Abhijit, which is none of the 27.
    pub nakshatra: Option<Nakshatra>,
    /// Which of the star's four syllables it is, 1 to 4: the pada, for
    /// one of the 27.
    pub quarter: u8,
    /// The varga of the name's first letter, as written.
    pub varga: NameVarga,
}

impl NameSyllable {
    /// The native whose Moon stands in this syllable's pada, for every
    /// koota a birth star is matched by.
    ///
    /// # Errors
    ///
    /// A syllable of Abhijit's row while `abhijit` refuses, naming the
    /// knob.
    pub fn native(self, abhijit: AbhijitPada) -> Result<Native, Error> {
        let (nakshatra, pada) = match (self.nakshatra, abhijit) {
            (Some(nakshatra), _) => (nakshatra, self.quarter),
            (None, AbhijitPada::UttaraAshadha) => (Nakshatra::UttaraAshadha, 4),
            (None, AbhijitPada::Shravana) => (Nakshatra::Shravana, 1),
            (None, AbhijitPada::Refuse) => {
                return Err(Error::invalid_arg(
                    "the name's syllable is Abhijit's, which is none of the 108 padas a koota reads",
                )
                .with_field("abhijit")
                .with_hint("set abhijit to UTTARA_ASHADHA or SHRAVANA"));
            }
        };
        Native::of_pada(nakshatra, pada)
    }
}

/// A name's first akshara: its first consonant as written, if any, and
/// its vowel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Akshara {
    consonant: Option<char>,
    svara: Svara,
}

/// A refusal of a name's first letter.
fn refuse(what: &str) -> Error {
    Error::invalid_arg(format!("a name {what}"))
        .with_field("name")
        .with_hint("a caller who knows the name's star passes it as a native instead")
}

/// What a Devanagari mark is to a name's first syllable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mark {
    /// A vowel or vowel sign the cakra reads (ṛ as i, C293).
    Vowel(Svara),
    /// A vowel or vowel sign it does not: ai, au, ḷ and the borrowed ones.
    Unread,
    /// No vowel at all.
    Other,
}

fn mark_of(mark: char) -> Mark {
    Mark::Vowel(match mark {
        'अ' | 'आ' | 'ा' => Svara::A,
        'इ' | 'ई' | 'ि' | 'ी' | 'ऋ' | 'ॠ' | 'ृ' | 'ॄ' => Svara::I,
        'उ' | 'ऊ' | 'ु' | 'ू' => Svara::U,
        'ए' | 'े' => Svara::E,
        'ओ' | 'ो' => Svara::O,
        'ऐ' | 'औ' | 'ऌ' | 'ॡ' | 'ै' | 'ौ' | 'ॢ' | 'ॣ' | 'ऍ' | 'ऑ' | 'ॅ' | 'ॉ' | 'ऎ' | 'ऒ' | 'ॆ'
        | 'ॊ' => return Mark::Unread,
        _ => return Mark::Other,
    })
}

/// A Devanagari consonant the vargas name, with a nukta's letter read as
/// its base.
fn consonant_of(letter: char) -> Option<char> {
    // The precomposed nukta letters; a nukta written apart is passed over
    // by the reader.
    let base = match letter {
        '\u{0958}' => 'क',
        '\u{0959}' => 'ख',
        '\u{095A}' => 'ग',
        '\u{095B}' => 'ज',
        '\u{095C}' => 'ड',
        '\u{095D}' => 'ढ',
        '\u{095E}' => 'फ',
        '\u{095F}' => 'य',
        other => other,
    };
    matches!(base, 'क'..='न' | 'प'..='र' | 'ल' | 'व'..='ह').then_some(base)
}

const VIRAMA: char = '्';
const NUKTA: char = '़';

/// The first akshara of a name in Devanagari: a conjunct by its first
/// consonant with the cluster's vowel, the inherent a when none is written.
fn devanagari(name: &str) -> Result<Akshara, Error> {
    let mut letters = name.chars().filter(|&c| c != NUKTA);
    let first = letters.next().ok_or_else(|| refuse("has a first letter"))?;
    let consonant = match mark_of(first) {
        Mark::Vowel(svara) => {
            return Ok(Akshara {
                consonant: None,
                svara,
            });
        }
        Mark::Unread => {
            return Err(refuse(&format!(
                "beginning with {first} is not in the cakra"
            )));
        }
        Mark::Other => consonant_of(first)
            .ok_or_else(|| refuse(&format!("beginning with {first} is not in the cakra")))?,
    };
    // Past the cluster's consonants to its vowel sign.
    let mut joined = false;
    for next in letters {
        match (next, joined) {
            (VIRAMA, _) => joined = true,
            (_, true) if consonant_of(next).is_some() => joined = false,
            _ => {
                let svara = match mark_of(next) {
                    Mark::Vowel(svara) => svara,
                    Mark::Unread => {
                        return Err(refuse(&format!(
                            "whose first vowel is {next} is not in the cakra"
                        )));
                    }
                    Mark::Other => Svara::A,
                };
                return Ok(Akshara {
                    consonant: Some(consonant),
                    svara,
                });
            }
        }
    }
    Ok(Akshara {
        consonant: Some(consonant),
        svara: Svara::A,
    })
}

/// IAST's consonants, the aspirates before the letters they begin with.
const IAST_CONSONANTS: [(&str, char); 33] = [
    ("kh", 'ख'),
    ("gh", 'घ'),
    ("ch", 'छ'),
    ("jh", 'झ'),
    ("ṭh", 'ठ'),
    ("ḍh", 'ढ'),
    ("th", 'थ'),
    ("dh", 'ध'),
    ("ph", 'फ'),
    ("bh", 'भ'),
    ("k", 'क'),
    ("g", 'ग'),
    ("ṅ", 'ङ'),
    ("c", 'च'),
    ("j", 'ज'),
    ("ñ", 'ञ'),
    ("ṭ", 'ट'),
    ("ḍ", 'ड'),
    ("ṇ", 'ण'),
    ("t", 'त'),
    ("d", 'द'),
    ("n", 'न'),
    ("p", 'प'),
    ("b", 'ब'),
    ("m", 'म'),
    ("y", 'य'),
    ("r", 'र'),
    ("l", 'ल'),
    ("v", 'व'),
    ("ś", 'श'),
    ("ṣ", 'ष'),
    ("s", 'स'),
    ("h", 'ह'),
];

/// IAST's vowels, the diphthongs before the vowels they begin with; `None`
/// for the ones the cakra does not read.
const IAST_VOWELS: [(&str, Option<Svara>); 13] = [
    ("ai", None),
    ("au", None),
    ("a", Some(Svara::A)),
    ("ā", Some(Svara::A)),
    ("i", Some(Svara::I)),
    ("ī", Some(Svara::I)),
    ("u", Some(Svara::U)),
    ("ū", Some(Svara::U)),
    ("ṛ", Some(Svara::I)),
    ("ṝ", Some(Svara::I)),
    ("ḷ", None),
    ("e", Some(Svara::E)),
    ("o", Some(Svara::O)),
];

/// The first akshara of a name in IAST, written precomposed.
fn iast(name: &str) -> Result<Akshara, Error> {
    let lower = name.to_lowercase();
    let mut rest = lower.as_str();
    let mut first = None;
    loop {
        if let Some((spelled, svara)) = IAST_VOWELS.iter().find(|(v, _)| rest.starts_with(v)) {
            let svara = svara.ok_or_else(|| {
                refuse(&format!(
                    "whose first vowel is {spelled} is not in the cakra"
                ))
            })?;
            return Ok(Akshara {
                consonant: first,
                svara,
            });
        }
        let Some((spelled, letter)) = IAST_CONSONANTS.iter().find(|(c, _)| rest.starts_with(c))
        else {
            let at = rest.chars().next().map_or_else(String::new, String::from);
            return Err(refuse(&if at.is_empty() {
                "has a vowel".to_owned()
            } else {
                format!("in IAST has no letter {at}; combining marks are written precomposed")
            }));
        };
        first.get_or_insert(*letter);
        rest = &rest[spelled.len()..];
    }
}

/// A name's first syllable under the rules (C291 to C294).
///
/// ```
/// use teistro_core::catalogue::Nakshatra;
/// use teistro_matching::{NameRules, NameVarga, name_syllable};
///
/// // रा: ra, Chitra's third syllable.
/// let ram = name_syllable("राम", NameRules::default())?;
/// assert_eq!((ram.nakshatra, ram.quarter, ram.varga), (Some(Nakshatra::Chitra), 3, NameVarga::Deer));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// A name with no first letter, one beginning with a letter or vowel the
/// cakra does not read, or a Latin spelling while `latin` refuses; each
/// is named `name`.
pub fn name_syllable(name: &str, rules: NameRules) -> Result<NameSyllable, Error> {
    let name = name.trim_start();
    let latin = name.chars().next().is_some_and(char::is_alphabetic)
        && !name
            .chars()
            .next()
            .is_some_and(|c| ('\u{0900}'..='\u{097F}').contains(&c));
    let akshara = match (latin, rules.latin) {
        (false, _) => devanagari(name)?,
        (true, LatinName::Iast) => iast(name)?,
        (true, LatinName::Refuse) => {
            return Err(Error::invalid_arg(
                "a name in Latin letters is read only as IAST, never guessed from English",
            )
            .with_field("name")
            .with_hint(
                "pass the name in Devanagari, or set latin to IAST if it is spelled in IAST",
            ));
        }
    };
    let read = akshara.consonant.map(|letter| {
        PAIRED
            .iter()
            .find(|(lacked, _)| *lacked == letter)
            .map_or(letter, |&(_, paired)| paired)
    });
    let at = cells()
        .position(|cell| match cell {
            Cell::Syllable(head, svara) => head == read && svara == akshara.svara,
            Cell::Letter(letter) => Some(letter) == read,
        })
        .ok_or_else(|| refuse("whose first syllable is in no cell of the cakra"))?;
    let row = at / 4;
    // Krittika's row first; Abhijit's is the 20th.
    let nakshatra = match row {
        19 => None,
        _ => Nakshatra::ALL
            .get((row - usize::from(row > 19) + 2) % 27)
            .copied(),
    };
    Ok(NameSyllable {
        cell: u8::try_from(at).unwrap_or(0),
        nakshatra,
        quarter: u8::try_from(at % 4 + 1).unwrap_or(1),
        varga: NameVarga::of(akshara.consonant),
    })
}

/// *Muhurta Chintamani*'s śatapada table as printed (1954, p. 173, leaf
/// n185, read on the image), from Ashvini, Abhijit after Uttarashadha,
/// with the print's vowel lengths.
pub(crate) const PRINTED: [[&str; 4]; 28] = [
    ["चू", "चे", "चो", "ला"],
    ["ली", "लू", "ले", "लो"],
    ["आ", "ई", "उ", "ए"],
    ["ओ", "वा", "वी", "वू"],
    ["वे", "वो", "का", "की"],
    ["कू", "घ", "ङ", "छा"],
    ["के", "को", "हा", "ही"],
    ["हू", "हे", "हो", "डा"],
    ["डी", "डू", "डे", "डो"],
    ["मा", "मी", "मू", "मे"],
    ["मो", "टा", "टी", "टू"],
    ["टे", "टो", "पा", "पी"],
    ["पू", "ष", "णा", "ठा"],
    ["पे", "पो", "रा", "री"],
    ["रू", "रे", "रो", "ता"],
    ["ती", "तू", "ते", "तो"],
    ["ना", "नी", "नू", "ने"],
    ["नो", "या", "यी", "यू"],
    ["ये", "यो", "भा", "भी"],
    ["भू", "धा", "फा", "ढा"],
    ["भे", "भो", "जा", "जी"],
    ["जू", "जे", "जो", "खा"],
    ["खी", "खू", "खे", "खो"],
    ["गा", "गी", "गू", "गे"],
    ["गो", "सा", "सी", "सू"],
    ["से", "सो", "दा", "दी"],
    ["दू", "थ", "झ", "ञा"],
    ["दे", "दो", "चा", "ची"],
];

/// The syllable a birth names a child by: the cakra's cell for the
/// Moon's pada, as the print spells it (C297 to C299).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct BirthSyllable {
    /// Its place among the cakra's 112 cells, 0 for a, Krittika's first;
    /// never one of Abhijit's four (C298).
    pub cell: u8,
    /// As *Muhurta Chintamani* p. 173 prints it.
    pub devanagari: String,
    /// The same spelling in IAST.
    pub iast: String,
    /// The varga a name beginning with it stands in (VI.35).
    pub varga: NameVarga,
}

/// The syllable a child born with the Moon in `pada` of `nakshatra` is
/// named by: the letter *Svarodaya*'s śatapada cakra gives the birth
/// pada, as *Muhurta Chintamani*'s nāmakarma commentary asks (C297). A
/// birth's pada is one of the 108, so Abhijit's four syllables are never
/// answered (C298).
///
/// ```
/// use teistro_core::catalogue::Nakshatra;
/// use teistro_matching::{NameVarga, birth_syllable};
///
/// let first = birth_syllable(Nakshatra::Ashwini, 1)?;
/// assert_eq!((first.devanagari.as_str(), first.iast.as_str()), ("चू", "cū"));
/// assert_eq!(first.varga, NameVarga::Lion);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// A `pada` outside 1 to 4, named `pada`.
pub fn birth_syllable(nakshatra: Nakshatra, pada: u8) -> Result<BirthSyllable, Error> {
    if !(1..=4).contains(&pada) {
        return Err(Error::invalid_arg(format!("a pada is 1 to 4, not {pada}")).with_field("pada"));
    }
    // Krittika's row first, Abhijit's 20th, as `name_syllable` counts.
    let star = (usize::from(nakshatra.id()) + 25) % 27;
    let row = star + usize::from(star >= 19);
    let printed = PRINTED
        .get((row + 2) % 28)
        .and_then(|four| four.get(usize::from(pada) - 1))
        .copied()
        .unwrap_or_default();
    Ok(BirthSyllable {
        cell: u8::try_from(row * 4 + usize::from(pada) - 1).unwrap_or(0),
        devanagari: printed.to_owned(),
        iast: iast_of(printed),
        varga: NameVarga::of(printed.chars().next().and_then(consonant_of)),
    })
}

/// A printed syllable in IAST, through the letters the IAST reader reads.
fn iast_of(printed: &str) -> String {
    let mut out = String::new();
    let mut inherent = false;
    for letter in printed.chars() {
        if let Some((spelled, _)) = IAST_CONSONANTS.iter().find(|(_, c)| *c == letter) {
            out.push_str(spelled);
            inherent = true;
            continue;
        }
        let vowel = match letter {
            'अ' => "a",
            'आ' | 'ा' => "ā",
            'इ' | 'ि' => "i",
            'ई' | 'ी' => "ī",
            'उ' | 'ु' => "u",
            'ऊ' | 'ू' => "ū",
            'ए' | 'े' => "e",
            'ओ' | 'ो' => "o",
            _ => continue,
        };
        out.push_str(vowel);
        inherent = false;
    }
    if inherent {
        out.push('a');
    }
    out
}

/// The readings two names are matched under.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NaamRules {
    /// How each name is read.
    pub name: NameRules,
    /// The Ashta Koota's.
    pub koota: KootaRules,
    /// The ten considerations'.
    pub porutham: PoruthamRules,
}

/// The match of two names (C296).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct NaamMilan {
    /// The bride's name's first syllable.
    pub bride: NameSyllable,
    /// The groom's.
    pub groom: NameSyllable,
    /// Their vargas (VI.35).
    pub varga: VargaKoota,
    /// The Ashta Koota of the two name stars.
    pub ashta: AshtaKoota,
    /// The ten considerations of the two name stars.
    pub porutham: Porutham,
}

/// Two names matched name star to name star, as Vasishtha asks when a
/// birth is unknown (C296).
///
/// # Errors
///
/// Either name refused by [`name_syllable`] or [`NameSyllable::native`],
/// named under `bride` or `groom`.
pub fn naam_milan(bride: &str, groom: &str, rules: NaamRules) -> Result<NaamMilan, Error> {
    let read = |name: &str, role: MatchRole| {
        name_syllable(name, rules.name)
            .and_then(|syllable| Ok((syllable, syllable.native(rules.name.abhijit)?)))
            .map_err(|error| error.under(role.name()))
    };
    let (bride, bride_star) = read(bride, MatchRole::Bride)?;
    let (groom, groom_star) = read(groom, MatchRole::Groom)?;
    Ok(NaamMilan {
        bride,
        groom,
        varga: varga_koota(bride.varga, groom.varga),
        ashta: ashta_koota(bride_star, groom_star, rules.koota),
        porutham: porutham(bride_star, groom_star, rules.porutham),
    })
}
