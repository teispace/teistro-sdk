//! A name read into the cakra: every cell its spelling reaches, each with
//! its sign and its clauses (C291 to C294, C772, C774).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Rashi;
use teistro_core::error::Error;

use super::cakra::{NameSyllable, devanagari, place, rashi_of_cell};
use super::clauses::{FinalA, GrhyaSutra, NameClauses, name_clauses};
use super::latin::{LatinName, spellings};
use super::pada::NasalInitial;

/// Where a name in Abhijit's row stands when a koota needs one of the 108
/// padas (C292).
///
/// ```
/// use teistro_core::catalogue::Nakshatra;
/// use teistro_matching::{AbhijitPada, NameRules, name_syllable};
///
/// // जू is in Abhijit's row, which is no star's.
/// let ju = name_syllable("जूही", NameRules::default())?;
/// assert_eq!(ju.nakshatra, None);
/// assert_eq!(ju.native(AbhijitPada::Refuse).unwrap_err().field(), Some("abhijit"));
/// assert_eq!(ju.native(AbhijitPada::Shravana)?.nakshatra, Nakshatra::Shravana);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
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
///
/// ```
/// use teistro_matching::{AbhijitPada, LatinName, NameRules};
///
/// assert_eq!(NameRules::default(), NameRules { latin: LatinName::Refuse, abhijit: AbhijitPada::Refuse });
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NameRules {
    /// How a Latin spelling is read.
    pub latin: LatinName,
    /// Where Abhijit's syllables stand.
    pub abhijit: AbhijitPada,
}

/// The rules a name is read and judged under (`03-design/namakarana.md`
/// §4.1); each default is the one no source read contradicts.
///
/// ```
/// use teistro_matching::{FinalA, GrhyaSutra, NameRules, NamingRules, NasalInitial};
///
/// let rules: NamingRules = serde_json::from_str(r#"{"sutra": "ASHVALAYANA"}"#)?;
/// assert_eq!(rules, NamingRules { sutra: GrhyaSutra::Ashvalayana, ..NamingRules::default() });
/// assert_eq!(
///     NamingRules::default(),
///     NamingRules { name: NameRules::default(), nasal: NasalInitial::VargaThird, sutra: GrhyaSutra::Paraskara, final_a: FinalA::Sounded },
/// );
/// assert!(serde_json::from_str::<NamingRules>(r#"{"sutras": "PARASKARA"}"#).is_err());
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NamingRules {
    /// How a name is read: its Latin spelling and Abhijit's place.
    pub name: NameRules,
    /// Whether a nasal's pada lists a substitute initial (C770).
    pub nasal: NasalInitial,
    /// Which gṛhyasūtra's clauses are reported (C774).
    pub sutra: GrhyaSutra,
    /// How syllables are counted (C774).
    pub final_a: FinalA,
}

impl From<NameRules> for NamingRules {
    fn from(name: NameRules) -> NamingRules {
        NamingRules {
            name,
            ..NamingRules::default()
        }
    }
}

/// One cell a name's spelling reaches.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "SpelledName"))]
#[serde(rename_all = "camelCase")]
pub struct Reading {
    /// The Devanagari it was read as: the name itself, or its Latin
    /// spelling transliterated.
    pub spelled: String,
    /// Its first syllable in the cakra.
    pub syllable: NameSyllable,
    /// The sign that syllable's pada stands in; Makara for Abhijit's.
    pub rashi: Rashi,
    /// Its syllables and the clauses of a name in daily use (C774).
    pub clauses: NameClauses,
}

/// A name read: one reading for each distinct first syllable its spelling
/// can reach, in cakra order. One, unless `CASUAL` reads it more ways.
///
/// ```
/// use teistro_core::catalogue::Nakshatra;
/// use teistro_matching::{LatinName, NameRules, NamingRules, read_name};
///
/// let casual = NamingRules::from(NameRules { latin: LatinName::Casual, ..NameRules::default() });
/// // शि or षि: Shatabhisha's 3rd or Hasta's 2nd, both answered.
/// let shiva = read_name("Shiva", casual)?;
/// assert!(shiva.ambiguous);
/// let stars: Vec<_> = shiva.readings.iter().map(|r| (r.syllable.nakshatra, r.syllable.quarter)).collect();
/// assert_eq!(stars, [(Some(Nakshatra::Hasta), 2), (Some(Nakshatra::Shatabhisha), 3)]);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct NameReading {
    /// Each distinct first syllable, with the first spelling that reaches
    /// it.
    pub readings: Vec<Reading>,
    /// Whether there is more than one: the spelling does not settle the
    /// name's cell, or its varga (`CASUAL`'s b reads as व and as ब, one
    /// cell and two vargas).
    pub ambiguous: bool,
}

/// A name read under the rules (C291 to C294, C772, C774): every cell
/// its spelling reaches, each with the spelling that reaches it, its sign
/// and its clauses; a Devanagari name or a declared scheme's reaches one.
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_matching::{NamingRules, read_name};
///
/// let ram = read_name("राम", NamingRules::default())?;
/// assert!(!ram.ambiguous);
/// assert_eq!((ram.readings[0].rashi, ram.readings[0].clauses.syllables), (Rashi::Libra, 2));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// A name with no first letter, one beginning with a letter or vowel the
/// cakra does not read, a Latin spelling while `latin` refuses, or a
/// letter the declared scheme does not spell; each is named `name`.
pub fn read_name(name: &str, rules: NamingRules) -> Result<NameReading, Error> {
    let mut readings: Vec<Reading> = Vec::new();
    for spelled in spellings(name.trim(), rules.name.latin)? {
        let syllable = place(devanagari(&spelled)?)?;
        if readings.iter().all(|seen| seen.syllable != syllable) {
            readings.push(Reading {
                rashi: rashi_of_cell(usize::from(syllable.cell)),
                clauses: name_clauses(&spelled, rules),
                spelled,
                syllable,
            });
        }
    }
    readings.sort_by_key(|reading| reading.syllable.cell);
    Ok(NameReading {
        ambiguous: readings.len() > 1,
        readings,
    })
}

impl NameReading {
    /// The one reading a match or a check needs, or a refusal listing the
    /// spellings when there are several.
    pub(crate) fn one(self) -> Result<Reading, Error> {
        let spellings: Vec<&str> = self
            .readings
            .iter()
            .map(|reading| reading.spelled.as_str())
            .collect();
        let listed = spellings.join(", ");
        let mut readings = self.readings.into_iter();
        match (readings.next(), readings.next()) {
            (Some(one), None) => Ok(one),
            _ => Err(Error::invalid_arg(
                "a name read as CASUAL reaches more than one syllable of the cakra, and one is needed",
            )
            .with_field("name")
            .with_hint(format!(
                "pass the name in Devanagari, as one of its spellings: {listed}"
            ))),
        }
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
/// cakra does not read, a Latin spelling while `latin` refuses or one
/// the declared scheme does not spell, or a `CASUAL` spelling that
/// reaches more than one syllable; each is named `name`.
pub fn name_syllable(name: &str, rules: NameRules) -> Result<NameSyllable, Error> {
    read_name(name, rules.into())?
        .one()
        .map(|reading| reading.syllable)
}
