//! The syllables a pada, a star or a sign names a child by, with every
//! letter a name may begin with to stand in each (C297 to C299, C770,
//! C771).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Nakshatra, Rashi};
use teistro_core::error::Error;
use teistro_intl::translit::devanagari_to_iast;

use super::NamingRules;
use super::cakra::{
    CELLS, NameVarga, PAIRED, cell, consonant_of, rashi_of_cell, row_of, star_of_row,
};

/// Abhijit's four syllables as *Muhurta Chintamani* p. 173 prints them,
/// the row the catalogue's 27 stars do not hold (C292, C299).
const ABHIJIT_PRINTED: [&str; 4] = ["जू", "जे", "जो", "खा"];

/// The nasals no name begins with, each and the third letter of its own
/// varga that Himal Panchanga VS 2082's rashifal headings print beside it
/// ("ङ=ग", "ण=ड", "ञ=ज", printed pp. 7–8; C770).
const SUBSTITUTES: [(char, char); 3] = [('ङ', 'ग'), ('ण', 'ड'), ('ञ', 'ज')];

/// Whether a pada whose letter no name begins with lists a substitute
/// (C770).
///
/// ```
/// use teistro_core::catalogue::Nakshatra;
/// use teistro_matching::{InitialBy, NamingRules, NasalInitial, SyllablesOf, syllables};
///
/// // Ardra's third pada is ङ: the patro reads it as ग.
/// let ardra = SyllablesOf::Pada { nakshatra: Nakshatra::Ardra, pada: 3 };
/// let listed = syllables(ardra, NamingRules::default())?;
/// assert!(listed[0].initials.iter().any(|i| i.by == InitialBy::Substitute && i.devanagari == "ग"));
/// let bare = NamingRules { nasal: NasalInitial::None, ..NamingRules::default() };
/// assert!(syllables(ardra, bare)?[0].initials.iter().all(|i| i.by != InitialBy::Substitute));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NasalInitial {
    /// ङ, ण and ञ each list their varga's third letter (ग, ड, ज) as a
    /// substitute initial, as a patro printed in Nepal glosses them.
    #[default]
    VargaThird,
    /// No substitute: the cell's own letter only, as *Muhurta
    /// Chintamani*'s commentary gives it.
    None,
}

/// How a letter reaches a cell.
///
/// ```
/// use teistro_core::catalogue::Nakshatra;
/// use teistro_matching::{InitialBy, NamingRules, SyllablesOf, syllables};
///
/// // Rohini's second pada is वा, which बा also reaches (C294).
/// let rohini = SyllablesOf::Pada { nakshatra: Nakshatra::Rohini, pada: 2 };
/// let by: Vec<InitialBy> = syllables(rohini, NamingRules::default())?[0].initials.iter().map(|i| i.by).collect();
/// assert_eq!(by, [InitialBy::Cell, InitialBy::Paired]);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InitialBy {
    /// The cell's own syllable, as the print spells it.
    Cell,
    /// A letter the cakra lacks, read as the one it is paired with: ब as
    /// व, श as स (C294).
    Paired,
    /// A nasal's substitute (C770). A name beginning with it reads in its
    /// own cell, not this one; a check says it was named by substitute.
    Substitute,
}

/// A letter a name may begin with to stand in a cell.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Initial {
    /// In Devanagari.
    pub devanagari: String,
    /// In IAST.
    pub iast: String,
    /// How it reaches the cell.
    pub by: InitialBy,
}

/// One cell of the cakra as a name may begin with it: a pada of the 108,
/// or one of Abhijit's four.
///
/// ```
/// use teistro_core::catalogue::{Nakshatra, Rashi};
/// use teistro_matching::{NamingRules, SyllablesOf, syllables};
///
/// let first = &syllables(SyllablesOf::Pada { nakshatra: Nakshatra::Ashwini, pada: 1 }, NamingRules::default())?[0];
/// assert_eq!((first.devanagari.as_str(), first.iast.as_str(), first.rashi), ("चू", "cū", Rashi::Aries));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PadaSyllable {
    /// Its place among the cakra's 112 cells, 0 for a, Krittika's first.
    pub cell: u8,
    /// Its star, `None` for Abhijit.
    pub nakshatra: Option<Nakshatra>,
    /// Its pada, 1 to 4; `None` for Abhijit's, which is no pada (C298).
    pub pada: Option<u8>,
    /// The sign the pada stands in; Makara for Abhijit's, wherever a
    /// koota places them (C771).
    pub rashi: Rashi,
    /// As *Muhurta Chintamani* p. 173 prints it (C299).
    pub devanagari: String,
    /// The same spelling in IAST.
    pub iast: String,
    /// The varga a name beginning with it stands in (VI.35).
    pub varga: NameVarga,
    /// Every letter that reaches it: its own, a paired letter, and a
    /// nasal's substitute when `nasal` lists one.
    pub initials: Vec<Initial>,
}

/// The print's spelling of a cell: the catalogue's for a star's pada,
/// [`ABHIJIT_PRINTED`] for Abhijit's.
fn printed(at: usize) -> &'static str {
    let row = match star_of_row(at / 4) {
        Some(nakshatra) => nakshatra.attributes().padas.map(|pada| pada.devanagari),
        None => ABHIJIT_PRINTED,
    };
    row.get(at % 4).copied().unwrap_or_default()
}

/// The cell at `at` with its initials under `rules`.
fn pada_syllable(at: usize, rules: NamingRules) -> PadaSyllable {
    let devanagari = printed(at);
    let own = cell(at).consonant();
    let initial = |devanagari: String, by| Initial {
        iast: devanagari_to_iast(&devanagari),
        devanagari,
        by,
    };
    // A paired letter takes the cell letter's place in the print's
    // spelling; a substitute stands alone, being read with any vowel.
    let paired = PAIRED
        .iter()
        .filter(|(_, paired)| own == Some(*paired))
        .map(|(lacked, paired)| {
            initial(
                devanagari.replacen(*paired, &lacked.to_string(), 1),
                InitialBy::Paired,
            )
        });
    let substitute = SUBSTITUTES
        .iter()
        .filter(|(nasal, _)| rules.nasal == NasalInitial::VargaThird && own == Some(*nasal))
        .map(|(_, third)| initial(third.to_string(), InitialBy::Substitute));
    let nakshatra = star_of_row(at / 4);
    PadaSyllable {
        cell: u8::try_from(at).unwrap_or(0),
        nakshatra,
        pada: nakshatra.map(|_| u8::try_from(at % 4 + 1).unwrap_or(1)),
        rashi: rashi_of_cell(at),
        devanagari: devanagari.to_owned(),
        iast: devanagari_to_iast(devanagari),
        varga: NameVarga::of(own),
        initials: std::iter::once(initial(devanagari.to_owned(), InitialBy::Cell))
            .chain(paired)
            .chain(substitute)
            .collect(),
    }
}

/// The letter a substitute initial stands for, when `initial` is one:
/// the first consonant a name beginning with it is spelled with.
pub(crate) fn substitute_letter(initial: &Initial) -> Option<char> {
    (initial.by == InitialBy::Substitute)
        .then(|| initial.devanagari.chars().next().and_then(consonant_of))
        .flatten()
}

/// How a name, spelled in Devanagari, reaches `prescribed`: by the
/// initial whose consonant it begins with, and by the cell's own when no
/// initial's is (a vowel's cell). Asked of a name in the cell or
/// beginning with one of its initials.
pub(crate) fn reached_by(prescribed: &PadaSyllable, spelled: &str) -> InitialBy {
    let first = |text: &str| text.chars().next().and_then(consonant_of);
    let begins = first(spelled);
    prescribed
        .initials
        .iter()
        .find(|initial| begins.is_some() && first(&initial.devanagari) == begins)
        .map_or(InitialBy::Cell, |initial| initial.by)
}

/// What syllables are asked for (C771).
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_matching::SyllablesOf;
///
/// let asked: SyllablesOf = serde_json::from_str(r#"{"of": "RASHI", "rashi": "CAPRICORN"}"#)?;
/// assert_eq!(asked, SyllablesOf::Rashi { rashi: Rashi::Capricorn });
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "of", rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
pub enum SyllablesOf {
    /// One pada's syllable.
    Pada {
        /// The star.
        nakshatra: Nakshatra,
        /// Its pada, 1 to 4.
        pada: u8,
    },
    /// A star's four.
    Nakshatra {
        /// The star.
        nakshatra: Nakshatra,
    },
    /// A sign's nine padas, and for Makara Abhijit's four as well.
    Rashi {
        /// The sign.
        rashi: Rashi,
    },
}

/// The syllables of a pada, a star or a sign, in cakra order (C297 to
/// C299, C770, C771): one, four, or a sign's nine padas, and Makara's
/// nine with Abhijit's four marked `pada: None`, since every reading
/// places Abhijit in Makara.
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_matching::{NamingRules, SyllablesOf, syllables};
///
/// let makara = syllables(SyllablesOf::Rashi { rashi: Rashi::Capricorn }, NamingRules::default())?;
/// assert_eq!(makara.len(), 13);
/// assert_eq!(makara.iter().filter(|s| s.pada.is_none()).count(), 4);
/// assert_eq!(syllables(SyllablesOf::Rashi { rashi: Rashi::Aries }, NamingRules::default())?.len(), 9);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// A `pada` outside 1 to 4, named `pada`.
pub fn syllables(of: SyllablesOf, rules: NamingRules) -> Result<Vec<PadaSyllable>, Error> {
    let cells = match of {
        SyllablesOf::Pada { nakshatra, pada } => {
            let at = cell_of(nakshatra, pada)?;
            at..at + 1
        }
        SyllablesOf::Nakshatra { nakshatra } => row_of(nakshatra) * 4..row_of(nakshatra) * 4 + 4,
        SyllablesOf::Rashi { .. } => 0..CELLS,
    };
    Ok(cells
        .filter(|&at| match of {
            SyllablesOf::Rashi { rashi } => rashi_of_cell(at) == rashi,
            _ => true,
        })
        .map(|at| pada_syllable(at, rules))
        .collect())
}

/// The cell of a star's pada.
fn cell_of(nakshatra: Nakshatra, pada: u8) -> Result<usize, Error> {
    match pada {
        1..=4 => Ok(row_of(nakshatra) * 4 + usize::from(pada) - 1),
        _ => Err(Error::invalid_arg(format!("a pada is 1 to 4, not {pada}")).with_field("pada")),
    }
}

/// The syllable a birth names a child by: the cakra's cell for the
/// Moon's pada, as the catalogue spells it after the print (C297 to C299).
/// The cell's own syllable alone, without the initials [`syllables`]
/// lists, as a chart's avakahada carries it.
///
/// ```
/// use teistro_core::catalogue::Nakshatra;
/// use teistro_matching::birth_syllable;
///
/// // Krittika's first pada is the cakra's first cell.
/// assert_eq!(birth_syllable(Nakshatra::Krittika, 1)?.cell, 0);
/// assert_eq!(birth_syllable(Nakshatra::Krittika, 5).unwrap_err().field(), Some("pada"));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
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

impl From<PadaSyllable> for BirthSyllable {
    fn from(syllable: PadaSyllable) -> BirthSyllable {
        BirthSyllable {
            cell: syllable.cell,
            devanagari: syllable.devanagari,
            iast: syllable.iast,
            varga: syllable.varga,
        }
    }
}

/// The pada's [`PadaSyllable`]: [`syllables`] of one pada.
pub(crate) fn pada_of(
    nakshatra: Nakshatra,
    pada: u8,
    rules: NamingRules,
) -> Result<PadaSyllable, Error> {
    Ok(pada_syllable(cell_of(nakshatra, pada)?, rules))
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
    pada_of(nakshatra, pada, NamingRules::default()).map(BirthSyllable::from)
}
