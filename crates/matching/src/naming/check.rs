//! A name against a birth, and the names a birth gives (nāmakaraṇa, C297,
//! C334, C770, C773).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Deity, Nakshatra};
use teistro_core::error::Error;

use super::NamingRules;
use super::cakra::{NameSyllable, consonant_of};
use super::clauses::NameClauses;
use super::pada::{PadaSyllable, pada_of, substitute_letter};
use super::read::read_name;

/// A birth, as the names it gives are read from it: the Moon's star and
/// pada.
///
/// ```
/// use teistro_core::catalogue::Nakshatra;
/// use teistro_matching::Birth;
///
/// let birth: Birth = serde_json::from_str(r#"{"nakshatra": "ASHWINI", "pada": 1}"#)?;
/// assert_eq!(birth, Birth { nakshatra: Nakshatra::Ashwini, pada: 1 });
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Birth {
    /// The Moon's nakshatra.
    pub nakshatra: Nakshatra,
    /// Its pada, 1 to 4.
    pub pada: u8,
}

/// A name read against the syllable a birth names a child by
/// (nāmakaraṇa, C297, C334, C770): where the name's first syllable falls
/// in the śatapada cakra, the birth pada's own cell, whether the two are
/// one star and one pada or the name begins with the pada's substitute,
/// and the clauses of a name in daily use. Facts, not a score.
///
/// ```
/// use teistro_core::catalogue::Nakshatra;
/// use teistro_matching::{Birth, NamingRules, name_check};
///
/// let birth = Birth { nakshatra: Nakshatra::Ashwini, pada: 1 };
/// let check = name_check("राम", birth, NamingRules::default())?;
/// // Answered, not refused: the name is Chitra's, the birth Ashvini's.
/// assert!(!check.same_star && !check.same_pada && !check.by_substitute);
/// assert_eq!(check.prescribed.devanagari, "चू");
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct NameCheck {
    /// The name's first syllable, as [`name_syllable`](crate::name_syllable)
    /// reads it.
    pub read: NameSyllable,
    /// The birth pada's syllable, with every initial that reaches it.
    pub prescribed: PadaSyllable,
    /// Whether the name begins in the birth star's row: "the letter of
    /// its star". Never for a syllable in Abhijit's row, which is no
    /// star's.
    pub same_star: bool,
    /// Whether it begins in the birth pada's own cell, the syllable the
    /// rule prescribes.
    pub same_pada: bool,
    /// Whether it begins with the substitute the pada lists for a letter
    /// no name begins with (C770): read in its own cell, and named by the
    /// patro's gloss.
    pub by_substitute: bool,
    /// The name's syllables and clauses (C774).
    pub clauses: NameClauses,
}

/// Reads a name against a birth's nakshatra and pada: the name's first
/// syllable in the śatapada cakra beside the syllable *Muhurta
/// Chintamani*'s nāmakarma commentary prescribes for the pada, whether
/// they share a star and a pada, whether the name took the pada's
/// substitute, and the clauses its sutra gives a name in daily use (C334,
/// C770, C774). A name the cakra reads in another star is answered, not
/// refused; the reader decides.
///
/// ```
/// use teistro_core::catalogue::Nakshatra;
/// use teistro_matching::{Birth, NamingRules, name_check};
///
/// // चू is Ashwini's first pada; चे its second.
/// let birth = Birth { nakshatra: Nakshatra::Ashwini, pada: 1 };
/// assert!(name_check("चूडामणि", birth, NamingRules::default())?.same_pada);
/// let other = name_check("चेतन", birth, NamingRules::default())?;
/// assert!(other.same_star && !other.same_pada);
/// // Ardra's third pada is ङ, which the patro reads as ग.
/// let ardra = Birth { nakshatra: Nakshatra::Ardra, pada: 3 };
/// assert!(name_check("गणेश", ardra, NamingRules::default())?.by_substitute);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// As [`read_name`] for the name, named `name`, and refused there too when
/// a `CASUAL` spelling reaches more than one syllable, since a check
/// reads one; a `pada` outside 1 to 4, named `pada`.
pub fn name_check(name: &str, birth: Birth, rules: NamingRules) -> Result<NameCheck, Error> {
    let prescribed = pada_of(birth.nakshatra, birth.pada, rules)?;
    let reading = read_name(name, rules)?.one()?;
    let read = reading.syllable;
    // A cell is a pada: four to a star's row, so the row is the star.
    let same_star = read.nakshatra.is_some() && read.cell / 4 == prescribed.cell / 4;
    let same_pada = read.cell == prescribed.cell;
    let first = reading.spelled.chars().next().and_then(consonant_of);
    let by_substitute = !same_pada
        && prescribed
            .initials
            .iter()
            .any(|initial| substitute_letter(initial).is_some_and(|letter| Some(letter) == first));
    Ok(NameCheck {
        read,
        prescribed,
        same_star,
        same_pada,
        by_substitute,
        clauses: reading.clauses,
    })
}

/// The names a birth gives, of *Dharmasindhu*'s four that the SDK can
/// compute (C773): the avakahada syllable, the default, and the star's
/// deity, the Śāṅkhāyanas' reading. The star's taddhita and the month's
/// name of Viṣṇu wait on their page images (`03-design/namakarana.md`
/// §8 item 5); the family deity's name is the family's own.
///
/// ```
/// use teistro_core::catalogue::{Deity, Nakshatra};
/// use teistro_matching::{Birth, NamingRules, names_of};
///
/// // Dharmasindhu's Āgniśarman: a Krittika birth named after Agni.
/// let names = names_of(Birth { nakshatra: Nakshatra::Krittika, pada: 1 }, NamingRules::default())?;
/// assert_eq!((names.syllable.devanagari.as_str(), names.deity), ("आ", Deity::Agni));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Names {
    /// The avakahada syllable of the birth pada, with its initials: the
    /// reading *Muhurta Chintamani*'s commentary prescribes and the Nepali
    /// patro prints.
    pub syllable: PadaSyllable,
    /// The star's deity, whose name the Śāṅkhāyanas give.
    pub deity: Deity,
}

/// The names a birth gives (C773): its pada's syllable beside its star's
/// deity, never instead of it.
///
/// ```
/// use teistro_core::catalogue::{Deity, Nakshatra};
/// use teistro_matching::{Birth, NamingRules, names_of};
///
/// let names = names_of(Birth { nakshatra: Nakshatra::Ashwini, pada: 4 }, NamingRules::default())?;
/// assert_eq!((names.syllable.iast.as_str(), names.deity), ("lā", Deity::AshwiniKumara));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// A `pada` outside 1 to 4, named `pada`.
pub fn names_of(birth: Birth, rules: NamingRules) -> Result<Names, Error> {
    Ok(Names {
        syllable: pada_of(birth.nakshatra, birth.pada, rules)?,
        deity: birth.nakshatra.attributes().deity,
    })
}
