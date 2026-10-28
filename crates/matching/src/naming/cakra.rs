//! *Svarodaya*'s śatapada cakra: its 112 cells, the vargas of VI.35, and
//! the first akshara of a name in Devanagari read into a cell (C291 to
//! C295).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Nakshatra, Rashi};
use teistro_core::error::Error;

use super::read::AbhijitPada;
use crate::Native;

/// The five vowels the cakra reads, length folded (C291).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Svara {
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
pub(crate) enum Cell {
    /// A row's head with one vowel.
    Syllable(Option<char>, Svara),
    /// A pillar's letter, with any vowel.
    Letter(char),
}

impl Cell {
    /// The consonant the cell begins with, `None` for a vowel.
    pub(crate) fn consonant(self) -> Option<char> {
        match self {
            Cell::Syllable(head, _) => head,
            Cell::Letter(letter) => Some(letter),
        }
    }
}

/// The cakra's cells: 28 rows of four, one for each star with Abhijit.
pub(crate) const CELLS: usize = 112;

/// Abhijit's row, the 20th from Krittika's (C292).
const ABHIJIT_ROW: usize = 19;

/// The 112 cells from Krittika's first, in order (v. 8).
pub(crate) fn cells() -> impl Iterator<Item = Cell> {
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

/// The cell at `at`, `at` below [`CELLS`].
pub(crate) fn cell(at: usize) -> Cell {
    cells().nth(at).unwrap_or(Cell::Syllable(None, Svara::A))
}

/// The row a star's four padas fill: Krittika's first, Abhijit's 20th.
pub(crate) fn row_of(nakshatra: Nakshatra) -> usize {
    let star = (usize::from(nakshatra.id()) + 25) % 27;
    star + usize::from(star >= ABHIJIT_ROW)
}

/// The star whose padas fill `row`, `None` for Abhijit's.
pub(crate) fn star_of_row(row: usize) -> Option<Nakshatra> {
    match row {
        ABHIJIT_ROW => None,
        _ => Nakshatra::ALL
            .get((row - usize::from(row > ABHIJIT_ROW) + 2) % 27)
            .copied(),
    }
}

/// The sign a cell's pada stands in: Makara for Abhijit's four, wherever
/// a koota places them (C771).
pub(crate) fn rashi_of_cell(at: usize) -> Rashi {
    star_of_row(at / 4).map_or(Rashi::Capricorn, |nakshatra| {
        let pada = usize::from(nakshatra.id()) * 4 + at % 4;
        Rashi::ALL.get(pada / 9).copied().unwrap_or(Rashi::Aries)
    })
}

/// The letters the cakra lacks, read as the letter *Svarodaya*'s
/// sarvatobhadra v. 22 pairs them with (C294).
pub(crate) const PAIRED: [(char, char); 2] = [('ब', 'व'), ('श', 'स')];

/// Which of the eight letter groups a name begins in, by the animal VI.35
/// gives it (C295).
///
/// ```
/// use teistro_matching::{NameRules, NameVarga, name_syllable};
///
/// assert_eq!(name_syllable("अनिल", NameRules::default())?.varga, NameVarga::Garuda);
/// assert_eq!(name_syllable("कमल", NameRules::default())?.varga, NameVarga::Cat);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
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
    pub(crate) fn of(first: Option<char>) -> NameVarga {
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
    pub(crate) fn place(self) -> usize {
        VARGAS.iter().position(|&v| v == self).unwrap_or(0)
    }
}

/// A name's first syllable as the cakra reads it.
///
/// ```
/// use teistro_matching::{NameRules, name_syllable};
///
/// // अ is the cakra's first cell, Krittika's first syllable.
/// let first = name_syllable("अनिल", NameRules::default())?;
/// assert_eq!((first.cell, first.quarter), (0, 1));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
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
    /// ```
    /// use teistro_core::catalogue::{Nakshatra, Rashi};
    /// use teistro_matching::{AbhijitPada, NameRules, name_syllable};
    ///
    /// // रा, Chitra's third syllable: the Moon in Chitra's third pada, in Libra.
    /// let moon = name_syllable("राम", NameRules::default())?.native(AbhijitPada::Refuse)?;
    /// assert_eq!((moon.nakshatra, moon.pada, moon.rashi), (Nakshatra::Chitra, 3, Rashi::Libra));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
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
pub(crate) struct Akshara {
    pub(crate) consonant: Option<char>,
    svara: Svara,
}

/// A refusal of a name's first letter.
pub(crate) fn refuse(what: &str) -> Error {
    Error::invalid_arg(format!("a name {what}"))
        .with_field("name")
        .with_hint("a caller who knows the name's star passes it as a native instead")
}

/// What a Devanagari mark is to a name's first syllable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Mark {
    /// A vowel or vowel sign the cakra reads (ṛ as i, C293).
    Vowel(Svara),
    /// A vowel or vowel sign it does not: ai, au, ḷ and the borrowed ones.
    Unread,
    /// No vowel at all.
    Other,
}

pub(crate) fn mark_of(mark: char) -> Mark {
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
pub(crate) fn consonant_of(letter: char) -> Option<char> {
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

pub(crate) const VIRAMA: char = '्';
pub(crate) const NUKTA: char = '़';

/// The first akshara of a name in Devanagari: a conjunct by its first
/// consonant with the cluster's vowel, the inherent a when none is written.
pub(crate) fn devanagari(name: &str) -> Result<Akshara, Error> {
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

/// The cell an akshara falls in, with the star and quarter of its row
/// (C291 to C294): ब read as व and श as स.
pub(crate) fn place(akshara: Akshara) -> Result<NameSyllable, Error> {
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
    Ok(NameSyllable {
        cell: u8::try_from(at).unwrap_or(0),
        nakshatra: star_of_row(at / 4),
        quarter: u8::try_from(at % 4 + 1).unwrap_or(1),
        varga: NameVarga::of(akshara.consonant),
    })
}
