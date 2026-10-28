//! The shape the gṛhyasūtras give a name in daily use, each clause
//! reported with its source and no verdict (C774,
//! `03-design/namakarana.md` §2.3).

use serde::{Deserialize, Serialize};

use super::cakra::{NUKTA, VIRAMA, consonant_of, is_devanagari};

/// How a name's syllables are counted (C774).
///
/// ```
/// use teistro_matching::{FinalA, NamingRules, name_clauses};
///
/// let text = name_clauses("राम", NamingRules::default());
/// assert_eq!((text.syllables, text.counted), (2, FinalA::Sounded));
/// let spoken = NamingRules { final_a: FinalA::Silent, ..NamingRules::default() };
/// assert_eq!(name_clauses("राम", spoken).syllables, 1);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FinalA {
    /// The text's count: every vowel written or inherent, the final `a`
    /// too (*Dharmasindhu* p. 164, *akṣaram aj svaraḥ*), so राम is rā-ma.
    #[default]
    Sounded,
    /// Speech's: a word's final inherent `a` dropped, as Hindi and Nepali
    /// say "rām", unless it is the word's only vowel.
    Silent,
}

/// Which gṛhyasūtra's clauses are reported (C774).
///
/// ```
/// use teistro_matching::{Clause, GrhyaSutra, NamingRules, name_clauses};
///
/// let ashvalayana = NamingRules { sutra: GrhyaSutra::Ashvalayana, ..NamingRules::default() };
/// let rows = name_clauses("देवः", ashvalayana).rows;
/// assert!(rows.iter().any(|row| row.clause == Clause::EvenForABoy && row.holds));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GrhyaSutra {
    /// *Pāraskara* I.17.2–3, which *Muhurta Chintamani*'s commentary
    /// quotes (p. 173): a girl's name odd and ending in ā.
    #[default]
    Paraskara,
    /// *Āśvalāyana* I.15.4–7 (SBE 29 pp. 182–183): a man's even and a
    /// woman's odd, "in every case".
    Ashvalayana,
}

/// Where a clause is read (C774).
///
/// ```
/// use teistro_matching::{Clause, ClauseSource, NamingRules, name_clauses};
///
/// let rows = name_clauses("कमल्", NamingRules::default()).rows;
/// let final_l = rows.iter().find(|row| row.clause == Clause::NoFinalLOrR).unwrap();
/// assert_eq!((final_l.source, final_l.holds), (ClauseSource::ApastambaHiranyakeshin, false));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClauseSource {
    /// *Pāraskara Gṛhyasūtra* I.17.2–3.
    Paraskara,
    /// *Āśvalāyana Gṛhyasūtra* I.15.4–7.
    Ashvalayana,
    /// *Āpastamba* and *Hiraṇyakeśin*, as *Dharmasindhu* p. 164 reports
    /// them; reported under either sutra, being neither's.
    ApastambaHiranyakeshin,
}

/// One clause of a name in daily use. The kṛt and taddhita clauses are
/// grammar the letters do not show, and the varṇa endings are a rule
/// about the family, so none of them is a row.
///
/// ```
/// use teistro_matching::{Clause, NamingRules, name_clauses};
///
/// // Dharmasindhu's Hari, with its visarga, meets every clause a boy's
/// // name is held to.
/// let rows = name_clauses("हरिः", NamingRules::default()).rows;
/// let girls = [Clause::OddForAGirl, Clause::LongAForAGirl];
/// assert!(rows.iter().filter(|row| !girls.contains(&row.clause)).all(|row| row.holds));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Clause {
    /// Begins with a sonant (*ghoṣavat*): the 3rd, 4th or 5th letter of a
    /// varga, or h.
    SonantInitial,
    /// A semivowel (y r l v) among the letters after the first.
    SemivowelWithin,
    /// Ends in a visarga (*abhiniṣṭhāna*).
    VisargaFinal,
    /// Two syllables (for steadiness) or four (for holy lustre).
    TwoOrFourSyllables,
    /// An even count, for a man.
    EvenForABoy,
    /// An odd count, for a woman.
    OddForAGirl,
    /// Ends in ā, for a girl (*Pāraskara* I.17.3 alone).
    LongAForAGirl,
    /// Does not end in the sound l or r: a last ल or र under a virama, or
    /// with its inherent `a` when speech drops it (`final_a: SILENT`).
    /// *Dharmasindhu*'s Hari, ending in i, meets it.
    NoFinalLOrR,
}

/// The clauses each sutra reports, in order, with the source each is
/// read in; a clause of neither sutra is reported under both.
const CLAUSES: [(Option<GrhyaSutra>, Clause, ClauseSource); 13] = [
    (
        Some(GrhyaSutra::Paraskara),
        Clause::SonantInitial,
        ClauseSource::Paraskara,
    ),
    (
        Some(GrhyaSutra::Paraskara),
        Clause::SemivowelWithin,
        ClauseSource::Paraskara,
    ),
    (
        Some(GrhyaSutra::Paraskara),
        Clause::VisargaFinal,
        ClauseSource::Paraskara,
    ),
    (
        Some(GrhyaSutra::Paraskara),
        Clause::TwoOrFourSyllables,
        ClauseSource::Paraskara,
    ),
    (
        Some(GrhyaSutra::Paraskara),
        Clause::OddForAGirl,
        ClauseSource::Paraskara,
    ),
    (
        Some(GrhyaSutra::Paraskara),
        Clause::LongAForAGirl,
        ClauseSource::Paraskara,
    ),
    (
        Some(GrhyaSutra::Ashvalayana),
        Clause::SonantInitial,
        ClauseSource::Ashvalayana,
    ),
    (
        Some(GrhyaSutra::Ashvalayana),
        Clause::SemivowelWithin,
        ClauseSource::Ashvalayana,
    ),
    (
        Some(GrhyaSutra::Ashvalayana),
        Clause::VisargaFinal,
        ClauseSource::Ashvalayana,
    ),
    (
        Some(GrhyaSutra::Ashvalayana),
        Clause::TwoOrFourSyllables,
        ClauseSource::Ashvalayana,
    ),
    (
        Some(GrhyaSutra::Ashvalayana),
        Clause::EvenForABoy,
        ClauseSource::Ashvalayana,
    ),
    (
        Some(GrhyaSutra::Ashvalayana),
        Clause::OddForAGirl,
        ClauseSource::Ashvalayana,
    ),
    (
        None,
        Clause::NoFinalLOrR,
        ClauseSource::ApastambaHiranyakeshin,
    ),
];

/// The clauses `sutra` reports, in its order, with their sources.
pub(crate) fn reported(sutra: GrhyaSutra) -> impl Iterator<Item = (Clause, ClauseSource)> {
    CLAUSES
        .iter()
        .filter(move |(of, _, _)| of.is_none_or(|of| of == sutra))
        .map(|&(_, clause, source)| (clause, source))
}

/// The sonants: each varga's 3rd, 4th and 5th letters, and h.
const SONANTS: [char; 16] = [
    'ग', 'घ', 'ङ', 'ज', 'झ', 'ञ', 'ड', 'ढ', 'ण', 'द', 'ध', 'न', 'ब', 'भ', 'म', 'ह',
];

/// The semivowels.
const SEMIVOWELS: [char; 4] = ['य', 'र', 'ल', 'व'];

const VISARGA: char = 'ः';

/// One clause, held or not, and where it is read.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ClauseRow {
    /// The clause.
    pub clause: Clause,
    /// Whether the name meets it.
    pub holds: bool,
    /// The text it is read in.
    pub source: ClauseSource,
}

/// A name's syllables and the clauses of its sutra: facts, not a verdict.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct NameClauses {
    /// Its syllables: its vowels, written or inherent (a conjunct's
    /// consonants count once, a virama's consonant not at all).
    pub syllables: u8,
    /// How they were counted.
    pub counted: FinalA,
    /// The sutra's clauses, in its order.
    pub rows: Vec<ClauseRow>,
}

/// What a Devanagari letter is to a syllable count.
fn is_vowel_letter(letter: char) -> bool {
    matches!(letter, 'अ'..='औ' | 'ॠ'..='ॡ' | 'ॐ')
}

fn is_vowel_sign(letter: char) -> bool {
    matches!(letter, 'ा'..='ौ' | 'ॢ' | 'ॣ' | 'ऺ' | 'ऻ' | 'ॏ' | 'ॕ'..='ॗ')
}

/// The syllables of one word: its vowel letters, its vowel signs, and the
/// inherent `a` of each consonant with neither a sign nor a virama after
/// it; the last one dropped when speech drops it.
fn word_syllables(word: &[char], final_a: FinalA) -> usize {
    let mut count = 0;
    let mut inherent_last = false;
    for (at, &letter) in word.iter().enumerate() {
        let next = word.get(at + 1).copied();
        let inherent = consonant_of(letter).is_some()
            && !next.is_some_and(|next| next == VIRAMA || is_vowel_sign(next));
        count += usize::from(is_vowel_letter(letter) || is_vowel_sign(letter) || inherent);
        inherent_last = inherent && next.is_none();
    }
    let dropped = final_a == FinalA::Silent && inherent_last && count > 1;
    count - usize::from(dropped)
}

/// A name's syllables and the clauses of the sutra `rules` name, read in
/// its Devanagari spelling (C774). Every clause is a fact about the
/// letters; which a parent keeps is theirs to say, and *Dharmasindhu*
/// keeps the count's parity when the rest cannot all be met.
///
/// ```
/// use teistro_matching::{Clause, NamingRules, name_clauses};
///
/// // Dharmasindhu's Rudra: no visarga, and an even count.
/// let rudra = name_clauses("रुद्र", NamingRules::default());
/// let holds = |clause| rudra.rows.iter().any(|row| row.clause == clause && row.holds);
/// assert!(!holds(Clause::VisargaFinal) && holds(Clause::TwoOrFourSyllables));
/// ```
#[must_use]
pub fn name_clauses(spelled: &str, rules: super::NamingRules) -> NameClauses {
    let letters: Vec<char> = spelled.trim().chars().filter(|&c| c != NUKTA).collect();
    let syllables: usize = letters
        .split(|&letter| !is_devanagari(letter))
        .map(|word| word_syllables(word, rules.final_a))
        .sum();
    let last = letters.last().copied();
    // The sound the name ends in, when it is a consonant's: under a
    // virama, or bare when speech drops its `a`.
    let final_consonant = match letters.as_slice() {
        [.., consonant, VIRAMA] => consonant_of(*consonant),
        [.., consonant] if rules.final_a == FinalA::Silent && syllables > 1 => {
            consonant_of(*consonant)
        }
        _ => None,
    };
    let holds = |clause: Clause| match clause {
        Clause::SonantInitial => letters
            .first()
            .and_then(|&first| consonant_of(first))
            .is_some_and(|first| SONANTS.contains(&first)),
        Clause::SemivowelWithin => letters
            .iter()
            .skip(1)
            .any(|&letter| consonant_of(letter).is_some_and(|c| SEMIVOWELS.contains(&c))),
        Clause::VisargaFinal => last == Some(VISARGA),
        Clause::TwoOrFourSyllables => matches!(syllables, 2 | 4),
        Clause::EvenForABoy => syllables % 2 == 0,
        Clause::OddForAGirl => syllables % 2 == 1,
        Clause::LongAForAGirl => matches!(last, Some('ा' | 'आ')),
        Clause::NoFinalLOrR => !matches!(final_consonant, Some('ल' | 'र')),
    };
    NameClauses {
        syllables: u8::try_from(syllables).unwrap_or(u8::MAX),
        counted: rules.final_a,
        rows: reported(rules.sutra)
            .map(|(clause, source)| ClauseRow {
                clause,
                holds: holds(clause),
                source,
            })
            .collect(),
    }
}
