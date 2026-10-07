//! Numerology from a name and a civil date (`03-design/numerology.md`).
//!
//! Nothing here reads the sky. A name is read letter by letter through a
//! table transcribed from its source, and a date through the arithmetic
//! that source prints; every intermediate sum is kept, so an answer shows
//! how it was reached as well as where it ended.
//!
//! ```
//! use teistro_numerology::{BirthDate, NumerologyRules, System, name_number, pythagorean_birth};
//!
//! // Balliett's own example (pp. 18-19): Henry 34, so 7; Elder 26, so 8;
//! // the name 15, so 6; and 17 January 1872 is 1 + 8 + 9 = 18, so 9.
//! let rules = NumerologyRules::default();
//! let name = name_number("Henry Elder", System::Pythagorean, &rules)?;
//! assert_eq!(name.reduction.number, 6);
//! let birth = pythagorean_birth(BirthDate::new(1872, 1, 17)?, &rules);
//! assert_eq!(birth.sum.map(|sum| sum.number), Some(9));
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

#![doc(html_no_source)]

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Graha;
use teistro_core::error::Error;

#[cfg(test)]
mod tests;

/// Balliett's cycle (*The Philosophy of Numbers*, 1908, p. 17): a to i,
/// j to r and s to z against 1 to 9.
pub const BALLIETT: [u8; 26] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, // a-i
    1, 2, 3, 4, 5, 6, 7, 8, 9, // j-r
    1, 2, 3, 4, 5, 6, 7, 8, // s-z
];

/// Cheiro's table (*Cheiro's Book of Numbers*, p. 70), the system he
/// calls Chaldean. No letter is 9.
pub const CHEIRO: [u8; 26] = [
    1, 2, 3, 4, 5, 8, 3, 5, 1, 1, 2, 3, 4, 5, 7, 8, 1, 2, 3, 4, 6, 6, 6, 5, 1, 7,
];

/// Which system reads the name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum System {
    /// Balliett's letter cycle, with the stopping numbers of
    /// [`NumerologyRules::masters`].
    Pythagorean,
    /// Cheiro's table, which keeps no number from reducing (p. 35).
    Chaldean,
}

impl System {
    /// The letter table this system reads, A to Z.
    #[must_use]
    pub const fn table(self) -> &'static [u8; 26] {
        match self {
            System::Pythagorean => &BALLIETT,
            System::Chaldean => &CHEIRO,
        }
    }
}

/// The numbers that stop a reduction (C-new-8).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Masters {
    /// Every number reduces to one digit, as Cheiro and Sepharial reduce.
    None,
    /// 11 and 22 stand, 33 does not: Balliett's Orange is 33, so 6 (p. 30).
    #[default]
    ElevenTwentyTwo,
    /// 11, 22 and 33 stand: the baseline engine's reading, which no
    /// public-domain text in hand prints.
    ElevenTwentyTwoThirtyThree,
}

impl Masters {
    fn stops(self, n: u32) -> bool {
        match self {
            Masters::None => false,
            Masters::ElevenTwentyTwo => n == 11 || n == 22,
            Masters::ElevenTwentyTwoThirtyThree => n == 11 || n == 22 || n == 33,
        }
    }
}

/// Where a name of several words is reduced (C-new-17).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NameReduction {
    /// Each word reduced, then the words added and reduced: Balliett pp.
    /// 18-19 and Cheiro p. 72 both print it so.
    #[default]
    ByWord,
    /// Every letter added, then reduced once: the baseline engine's.
    Whole,
}

/// What a Chaldean name's compound number is (C-new-18).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChaldeanCompound {
    /// The sum of the words' single numbers, as Cheiro adds Lloyd's 9 and
    /// George's 7 to 16 (pp. 71-72); a one-word name's is its letters'.
    #[default]
    SumOfSingles,
    /// Every letter added: the baseline engine's.
    LetterTotal,
}

/// How a birth date becomes the Pythagorean birth number (C-new-19).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BirthReduction {
    /// The month, the day and the year reduced apart and then added
    /// (Balliett p. 19); a part that is a master stands apart from the
    /// sum, as her 11 July 1838 is "9, 11" (p. 90).
    #[default]
    ByPart,
    /// The digits of the day, the month and the year added and the total
    /// reduced: the baseline engine's.
    DigitSum,
}

/// What is done with a character that is not one of the 26 Latin letters
/// (C-new-23).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NonLatin {
    /// Refused, naming the character and its place: both tables are of
    /// Latin letters, and a guess at another script's would be invented.
    #[default]
    Refuse,
    /// Left out of every sum: the baseline engine's.
    Skip,
}

/// The readings numerology is taken under. Each default is what the
/// text that defines the system prints; [`NumerologyRules::baseline`] is
/// the baseline engine's, value for value.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NumerologyRules {
    /// The Pythagorean stopping numbers; the Chaldean has none.
    pub masters: Masters,
    /// Where a name is reduced.
    pub name_reduction: NameReduction,
    /// A Chaldean name's compound.
    pub chaldean_compound: ChaldeanCompound,
    /// How the Pythagorean birth number is reached.
    pub birth_reduction: BirthReduction,
    /// A character outside A to Z.
    pub non_latin: NonLatin,
}

impl NumerologyRules {
    /// The baseline engine's readings, every one of them; a profile taken
    /// under these also carries the baseline's own numbers
    /// ([`Profile::baseline`]).
    #[must_use]
    pub const fn baseline() -> NumerologyRules {
        NumerologyRules {
            masters: Masters::ElevenTwentyTwoThirtyThree,
            name_reduction: NameReduction::Whole,
            chaldean_compound: ChaldeanCompound::LetterTotal,
            birth_reduction: BirthReduction::DigitSum,
            non_latin: NonLatin::Skip,
        }
    }
}

/// A number reduced: every sum from the first to the last, so that
/// "33, so 6" and "38, so 11" can both be read back.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Reduction {
    /// The sums in order: the number reduced, then each digit sum.
    pub steps: Vec<u32>,
    /// Where it stopped: one digit, or a master the rules keep.
    pub number: u32,
}

/// Reduces a number by adding its digits until one digit is left or a
/// master the rules keep is reached. Zero stays zero.
#[must_use]
pub fn reduce(n: u32, masters: Masters) -> Reduction {
    let mut steps = vec![n];
    let mut at = n;
    while at > 9 && !masters.stops(at) {
        at = digit_sum(at);
        steps.push(at);
    }
    Reduction { steps, number: at }
}

fn digit_sum(mut n: u32) -> u32 {
    let mut sum = 0;
    while n > 0 {
        sum += n % 10;
        n /= 10;
    }
    sum
}

/// One word of a name.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct WordNumber {
    /// The word as written.
    pub text: String,
    /// Each letter read, upper case, with its value.
    pub letters: Vec<(char, u8)>,
    /// The letters added.
    pub total: u32,
    /// The total reduced.
    pub reduction: Reduction,
}

/// A name read under one system.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct NameNumber {
    /// The system read.
    pub system: System,
    /// Every word, in order, so either reduction can be read back.
    pub words: Vec<WordNumber>,
    /// What the final reduction started from: the words' numbers added
    /// ([`NameReduction::ByWord`]) or every letter ([`NameReduction::Whole`]).
    pub total: u32,
    /// Cheiro's compound number, for the Chaldean system only.
    pub compound: Option<u32>,
    /// The name's number.
    pub reduction: Reduction,
}

/// Reads a name under a system.
///
/// Words are separated by spaces, hyphens and apostrophes, which are
/// never refused. Letters are read without regard to case.
///
/// # Errors
///
/// A character outside A to Z under [`NonLatin::Refuse`], named with its
/// place; and a name with no letter that counts, which has no number
/// rather than a zero.
pub fn name_number(
    name: &str,
    system: System,
    rules: &NumerologyRules,
) -> Result<NameNumber, Error> {
    let masters = match system {
        System::Pythagorean => rules.masters,
        System::Chaldean => Masters::None,
    };
    let words = words(name, system.table(), rules.non_latin, masters)?;
    let letters: u32 = words.iter().map(|word| word.total).sum();
    let total = match rules.name_reduction {
        NameReduction::ByWord => words.iter().map(|word| word.reduction.number).sum(),
        NameReduction::Whole => letters,
    };
    let compound =
        (system == System::Chaldean).then(|| match (rules.chaldean_compound, words.as_slice()) {
            (ChaldeanCompound::LetterTotal, _) | (_, [_]) => letters,
            (ChaldeanCompound::SumOfSingles, _) => {
                words.iter().map(|word| word.reduction.number).sum()
            }
        });
    Ok(NameNumber {
        system,
        total,
        compound,
        reduction: reduce(total, masters),
        words,
    })
}

fn words(
    name: &str,
    table: &[u8; 26],
    non_latin: NonLatin,
    masters: Masters,
) -> Result<Vec<WordNumber>, Error> {
    let mut words = Vec::new();
    for (index, text) in name
        .split(|c: char| c.is_whitespace() || c == '-' || c == '\'' || c == '\u{2019}')
        .enumerate()
        .filter(|(_, text)| !text.is_empty())
    {
        let mut letters = Vec::new();
        for (place, c) in text.chars().enumerate() {
            let upper = c.to_ascii_uppercase();
            match upper {
                'A'..='Z' => {
                    let value = table
                        .get(usize::from(upper as u8 - b'A'))
                        .copied()
                        .unwrap_or_default();
                    letters.push((upper, value));
                }
                _ if non_latin == NonLatin::Skip => {}
                _ => {
                    return Err(Error::invalid_arg(format!(
                        "`{c}` (character {} of word {} of the name) is not one of the 26 Latin \
                         letters both tables are written for",
                        place + 1,
                        index + 1
                    ))
                    .with_field("name")
                    .with_hint(
                        "spell the name in plain Latin letters (é as e), or set nonLatin to \
                         SKIP to leave such characters out as the baseline engine does",
                    ));
                }
            }
        }
        if letters.is_empty() {
            continue;
        }
        let total = letters.iter().map(|&(_, value)| u32::from(value)).sum();
        words.push(WordNumber {
            text: text.to_string(),
            letters,
            total,
            reduction: reduce(total, masters),
        });
    }
    if words.is_empty() {
        return Err(
            Error::invalid_arg("the name has no letter that counts, so it has no number")
                .with_field("name")
                .with_hint("pass a name with at least one Latin letter"),
        );
    }
    Ok(words)
}

/// A Gregorian birth date, validated.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct BirthDate {
    /// The year, astronomical (1 BCE is 0).
    pub year: i32,
    /// The month, 1 to 12.
    pub month: u8,
    /// The day of the month.
    pub day: u8,
}

impl BirthDate {
    /// A Gregorian date, held to the calendar.
    ///
    /// # Errors
    ///
    /// A month outside 1 to 12, a day the month does not have, or a year
    /// before 1: a year's digits are read, and a year 0 or below has none
    /// a source adds.
    pub fn new(year: i32, month: u8, day: u8) -> Result<BirthDate, Error> {
        let Some(length) = teistro_calendar::gregorian::month_length(year, month) else {
            return Err(
                Error::invalid_arg(format!("month {month} is not 1 to 12")).with_field("month")
            );
        };
        if day == 0 || day > length {
            return Err(Error::invalid_arg(format!(
                "day {day} is not in month {month} of {year}, which has {length}"
            ))
            .with_field("day"));
        }
        if year < 1 {
            return Err(
                Error::invalid_arg(format!("year {year} has no digits a source adds"))
                    .with_field("year")
                    .with_hint("give the year of the Common Era, 1 or later"),
            );
        }
        Ok(BirthDate { year, month, day })
    }

    fn year_digits(self) -> u32 {
        self.year.unsigned_abs()
    }
}

/// The Pythagorean birth number.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct BirthNumber {
    /// The month reduced ([`BirthReduction::ByPart`]), or its digits.
    pub month: Reduction,
    /// The day reduced, or its digits.
    pub day: Reduction,
    /// The year reduced, or its digits.
    pub year: Reduction,
    /// The parts that are not masters, added and reduced; `None` when
    /// every part is a master.
    pub sum: Option<Reduction>,
    /// The parts that are masters, which stand apart from the sum
    /// (Balliett p. 90), in the order month, day, year.
    pub apart: Vec<u32>,
}

/// The Pythagorean birth number of a date.
#[must_use]
pub fn pythagorean_birth(date: BirthDate, rules: &NumerologyRules) -> BirthNumber {
    let masters = rules.masters;
    match rules.birth_reduction {
        BirthReduction::ByPart => {
            let month = reduce(u32::from(date.month), masters);
            let day = reduce(u32::from(date.day), masters);
            let year = reduce(date.year_digits(), masters);
            let parts = [month.number, day.number, year.number];
            let apart: Vec<u32> = parts
                .iter()
                .copied()
                .filter(|n| masters.stops(*n))
                .collect();
            let rest: Vec<u32> = parts
                .iter()
                .copied()
                .filter(|n| !masters.stops(*n))
                .collect();
            let sum = (!rest.is_empty()).then(|| reduce(rest.iter().sum(), masters));
            BirthNumber {
                month,
                day,
                year,
                sum,
                apart,
            }
        }
        BirthReduction::DigitSum => {
            let month = digits(u32::from(date.month));
            let day = digits(u32::from(date.day));
            let year = digits(date.year_digits());
            let total = month.number + day.number + year.number;
            BirthNumber {
                month,
                day,
                year,
                sum: Some(reduce(total, masters)),
                apart: Vec::new(),
            }
        }
    }
}

/// One digit sum and no more, which is what the baseline adds.
fn digits(n: u32) -> Reduction {
    let sum = digit_sum(n);
    Reduction {
        steps: if sum == n { vec![n] } else { vec![n, sum] },
        number: sum,
    }
}

/// Cheiro's numbers of a date: the day and the year, each its own, which
/// are "not added together" (p. 93).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ChaldeanDate {
    /// The birth number: the day of the month, reduced.
    pub birth: Reduction,
    /// The year's digits reduced, his "current of destiny".
    pub year: Reduction,
}

/// Cheiro's numbers of a date.
#[must_use]
pub fn chaldean_birth(date: BirthDate) -> ChaldeanDate {
    ChaldeanDate {
        birth: reduce(u32::from(date.day), Masters::None),
        year: reduce(date.year_digits(), Masters::None),
    }
}

/// The compound number whose meaning Cheiro gives a compound (pp. 78-85):
/// 10 to 32 their own, and from 33 the number nine below, but for 37,
/// 43 and 51, which have meanings of their own. `None` outside 10 to 52.
#[must_use]
pub const fn compound_class(n: u32) -> Option<u32> {
    match n {
        10..=32 | 37 | 43 | 51 => Some(n),
        33..=52 => Some(n - 9),
        _ => None,
    }
}

/// The planets Cheiro gives a single number (p. 22): the Sun is 1 and 4,
/// the Moon 2 and 7, Uranus 4 and Neptune 7. Empty outside 1 to 9.
#[must_use]
pub const fn planets_of(n: u32) -> &'static [Graha] {
    match n {
        1 => &[Graha::Sun],
        2 => &[Graha::Moon],
        3 => &[Graha::Jupiter],
        4 => &[Graha::Uranus, Graha::Sun],
        5 => &[Graha::Mercury],
        6 => &[Graha::Venus],
        7 => &[Graha::Neptune, Graha::Moon],
        8 => &[Graha::Saturn],
        9 => &[Graha::Mars],
        _ => &[],
    }
}

/// The baseline engine's number for a graha, which gives Rahu 4 and Ketu
/// 7 where Cheiro gives Uranus and Neptune; no public-domain text in hand
/// prints it. `None` for a graha it does not number.
#[must_use]
pub const fn baseline_anka(graha: Graha) -> Option<u32> {
    match graha {
        Graha::Sun => Some(1),
        Graha::Moon => Some(2),
        Graha::Jupiter => Some(3),
        Graha::Rahu => Some(4),
        Graha::Mercury => Some(5),
        Graha::Venus => Some(6),
        Graha::Ketu => Some(7),
        Graha::Saturn => Some(8),
        Graha::Mars => Some(9),
        _ => None,
    }
}

/// The days of a month whose single number is `n` (Cheiro p. 49: the
/// 5th, 14th and 23rd for 5).
#[must_use]
pub fn days_of(n: u32) -> Vec<u8> {
    (1..=31u8)
        .filter(|day| reduce(u32::from(*day), Masters::None).number == n)
        .collect()
}

/// Everything numerology says of a name and a birth date.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    /// The name under Balliett's cycle.
    pub pythagorean_name: NameNumber,
    /// The birth number under Balliett's arithmetic.
    pub pythagorean_birth: BirthNumber,
    /// The name under Cheiro's table.
    pub chaldean_name: NameNumber,
    /// Cheiro's numbers of the date.
    pub chaldean_birth: ChaldeanDate,
    /// The baseline engine's own numbers, present only under
    /// [`NumerologyRules::baseline`], so they are never mistaken for the
    /// texts'.
    pub baseline: Option<BaselineProfile>,
}

/// The numbers only the baseline engine computes, with no public-domain
/// source in hand.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct BaselineProfile {
    /// The vowels A, E, I, O and U under Balliett's cycle (its "soul").
    pub soul: Reduction,
    /// The other letters (its "personality").
    pub personality: Reduction,
    /// The date's digit sum under Cheiro, which he forbids (p. 93).
    pub chaldean_destiny: Reduction,
}

/// Everything numerology says of a name and a birth date.
///
/// # Errors
///
/// As [`name_number`], for the name.
pub fn profile(name: &str, date: BirthDate, rules: &NumerologyRules) -> Result<Profile, Error> {
    let pythagorean_name = name_number(name, System::Pythagorean, rules)?;
    let chaldean_name = name_number(name, System::Chaldean, rules)?;
    let baseline = (*rules == NumerologyRules::baseline()).then(|| {
        let letters = pythagorean_name
            .words
            .iter()
            .flat_map(|word| word.letters.iter());
        let (vowels, consonants): (Vec<_>, Vec<_>) =
            letters.partition(|(letter, _)| matches!(letter, 'A' | 'E' | 'I' | 'O' | 'U'));
        let sum = |part: &[&(char, u8)]| part.iter().map(|(_, value)| u32::from(*value)).sum();
        let dated = digits(u32::from(date.day)).number
            + digits(u32::from(date.month)).number
            + digits(date.year_digits()).number;
        BaselineProfile {
            soul: reduce(sum(&vowels), rules.masters),
            personality: reduce(sum(&consonants), rules.masters),
            chaldean_destiny: reduce(dated, Masters::None),
        }
    });
    Ok(Profile {
        pythagorean_birth: pythagorean_birth(date, rules),
        chaldean_birth: chaldean_birth(date),
        pythagorean_name,
        chaldean_name,
        baseline,
    })
}
