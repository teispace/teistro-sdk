//! Numbers and amounts in words, by CLDR's rule-based number formats
//! (`docs/03-design/amounts-in-words.md`): Nepali, Hindi and English with
//! the Indian grouping (hazar, lakh, crore, arab, kharab), English with the
//! international one too, a currency's units from the locale's messages,
//! and the figures in Latin or Devanagari digits.
//!
//! ```
//! use teistro_intl::source::Tree;
//! use teistro_intl::words::InWordsRequest;
//! use teistro_intl::{Intl, sdk_root};
//!
//! let intl = Intl::from_tree(&Tree::load(&sdk_root())?)?;
//! let asked = InWordsRequest::from_json(
//!     r#"{"amount": {"currency": "NPR", "value": {"minor": 123456}, "only": true}, "locale": "ne-Deva-NP"}"#,
//! )?;
//! let words = intl.in_words(&asked)?;
//! assert_eq!(words.text, "एक हजार दुई सय चौँतिस रुपैयाँ र छपन्न पैसा मात्र");
//! assert_eq!(words.figure, "१,२३४.५६");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use teistro_core::error::Error;

use crate::render::digits;
use crate::{Intl, Params, Value};

#[rustfmt::skip]
mod cldr;
pub mod rbnf;

pub use cldr::{CLDR_RELEASE, SOURCES};

use rbnf::{RuleSet, grouped, spell};

/// The record's name where a binding sends it, which a refusal is named
/// under.
const IN_WORDS: &str = "inWords";

/// One CLDR file's rules, as `cargo xtask gen intl` keeps them: the text
/// of the cardinal rule sets and every set they reach, each rule as CLDR
/// wrote it.
///
/// ```
/// use teistro_intl::words::SOURCES;
///
/// let nepali = SOURCES.iter().find(|source| source.file == "ne").ok_or("Nepali's rules")?;
/// assert!(nepali.rules.contains("100000: << लाख[ >>];"));
/// assert_eq!(nepali.sha256.len(), 64);
/// # Ok::<(), &str>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Source {
    /// The file's name without `.xml`: `ne`, `hi`, `en`, `en_IN`.
    pub file: &'static str,
    /// The SHA-256 of the file in `crates/intl/cldr/rbnf/`, as hex.
    pub sha256: &'static str,
    /// The rule sets kept, in RBNF's own syntax.
    pub rules: &'static str,
}

/// How digits are grouped, and so which scale words a number is counted
/// in.
///
/// ```
/// use teistro_intl::words::WordsGrouping;
///
/// let read: WordsGrouping = serde_json::from_str("\"INDIAN\"")?;
/// assert_eq!(read, WordsGrouping::Indian);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "words-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WordsGrouping {
    /// Every two digits past the thousands: lakh, crore, arab, kharab;
    /// `12,34,567`.
    Indian,
    /// Every three: million, billion; `1,234,567`. English only.
    International,
}

/// The digits a figure is written in.
///
/// ```
/// use teistro_intl::words::WordsDigits;
///
/// assert_eq!(WordsDigits::Deva.system(), "deva");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "words-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WordsDigits {
    /// `0123456789`.
    Latn,
    /// `०१२३४५६७८९`.
    Deva,
}

impl WordsDigits {
    /// The CLDR numbering system's name.
    ///
    /// ```
    /// use teistro_intl::words::WordsDigits;
    ///
    /// assert_eq!(WordsDigits::Latn.system(), "latn");
    /// ```
    #[must_use]
    pub const fn system(self) -> &'static str {
        match self {
            WordsDigits::Latn => "latn",
            WordsDigits::Deva => "deva",
        }
    }
}

/// Which of CLDR's cardinal rule sets spells the words.
///
/// ```
/// use teistro_intl::words::WordsStyle;
///
/// assert_eq!(WordsStyle::default(), WordsStyle::Plain);
/// assert_eq!(WordsStyle::Verbose.rule_set(), "spellout-cardinal-verbose");
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "words-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WordsStyle {
    /// `spellout-cardinal`: "one hundred five".
    #[default]
    Plain,
    /// `spellout-cardinal-verbose`: "one hundred and five". English only.
    Verbose,
}

impl WordsStyle {
    /// The CLDR rule set's name.
    ///
    /// ```
    /// use teistro_intl::words::WordsStyle;
    ///
    /// assert_eq!(WordsStyle::Plain.rule_set(), "spellout-cardinal");
    /// ```
    #[must_use]
    pub const fn rule_set(self) -> &'static str {
        match self {
            WordsStyle::Plain => "spellout-cardinal",
            WordsStyle::Verbose => "spellout-cardinal-verbose",
        }
    }
}

/// How a decimal with more digits than the currency's minor unit is
/// brought to it.
///
/// ```
/// use teistro_intl::words::WordsRounding;
///
/// assert_eq!(WordsRounding::default(), WordsRounding::HalfUp);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "words-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WordsRounding {
    /// Half a minor unit or more goes up, less is dropped: India's
    /// banking rule (RBI DBOD.Dir.BC.70/13.01.01/2006-07), and the
    /// convention for NPR (C805).
    #[default]
    HalfUp,
    /// Half a minor unit goes to the even neighbour (banker's rounding).
    HalfEven,
    /// Refuses a decimal whose digits past the minor unit are not zero.
    Exact,
}

/// An amount's value: a count of minor units, or a decimal string.
/// Never a float.
///
/// ```
/// use teistro_intl::words::WordsAmountValue;
///
/// let minor: WordsAmountValue = serde_json::from_str(r#"{"minor": 123456}"#)?;
/// assert_eq!(minor, WordsAmountValue::Minor(123_456));
/// let decimal: WordsAmountValue = serde_json::from_str(r#"{"decimal": "1234.56"}"#)?;
/// assert_eq!(decimal, WordsAmountValue::Decimal(String::from("1234.56")));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "words-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum WordsAmountValue {
    /// The minor units: 123456 paisa is NPR 1,234.56.
    Minor(u64),
    /// ASCII digits with at most one `.`, no sign and no exponent,
    /// rounded to the minor unit by the amount's `rounding`.
    Decimal(String),
}

/// An amount of a currency.
///
/// ```
/// use teistro_intl::words::{WordsAmount, WordsAmountValue, WordsRounding};
///
/// let read: WordsAmount = serde_json::from_str(r#"{"currency": "INR", "value": {"minor": 5}}"#)?;
/// assert_eq!(read.minor_digits, 2);
/// assert_eq!(read.rounding, WordsRounding::HalfUp);
/// assert!(!read.only);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "words-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct WordsAmount {
    /// The ISO 4217 code, three capital letters; its unit words are the
    /// locale's messages `sdk.amount.<CODE>.major` and `.minor`.
    pub currency: String,
    /// The value.
    pub value: WordsAmountValue,
    /// The minor unit's decimal places, 0 to 4: 2 for NPR and INR, 0 for
    /// a currency with no minor unit.
    #[serde(default = "two")]
    pub minor_digits: u8,
    /// How a decimal is brought to the minor unit.
    #[serde(default)]
    pub rounding: WordsRounding,
    /// Whether the closing word ("only", "मात्र") follows, as a cheque's
    /// legal amount has it.
    #[serde(default)]
    pub only: bool,
}

const fn two() -> u8 {
    2
}

/// What a caller asks to be written in words: a number or an amount,
/// and how.
///
/// ```
/// use teistro_intl::words::{WordsGrouping, InWordsRequest};
///
/// let asked = InWordsRequest::from_json(r#"{"number": 100000, "grouping": "INDIAN"}"#)?;
/// assert_eq!(asked.number, Some(100_000));
/// assert_eq!(asked.grouping, Some(WordsGrouping::Indian));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "words-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct InWordsRequest {
    /// A number to spell, negatives by the rule set's own word for
    /// minus; this or `amount`.
    #[serde(default)]
    pub number: Option<i64>,
    /// An amount to spell with its units; this or `number`.
    #[serde(default)]
    pub amount: Option<WordsAmount>,
    /// The locale, a loaded tag (`ne-Deva-NP`); the engine's current one
    /// when left out.
    #[serde(default)]
    pub locale: Option<String>,
    /// The grouping; the locale's own (`_meta.json`) when left out.
    #[serde(default)]
    pub grouping: Option<WordsGrouping>,
    /// The figures' digits; the locale's numbering system when left out.
    #[serde(default)]
    pub digits: Option<WordsDigits>,
    /// The rule set.
    #[serde(default)]
    pub style: WordsStyle,
}

impl InWordsRequest {
    /// The record a binding sends, as JSON.
    ///
    /// ```
    /// use teistro_intl::words::InWordsRequest;
    ///
    /// let typo = InWordsRequest::from_json(r#"{"numbr": 5}"#).unwrap_err();
    /// assert_eq!(typo.field(), Some("inWords.numbr"));
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record or a key it does not
    /// read, named under `inWords`.
    pub fn from_json(text: &str) -> Result<InWordsRequest, Error> {
        teistro_core::strict::read(text, IN_WORDS)
    }
}

/// A number or an amount in words, and in figures.
///
/// ```
/// use teistro_intl::source::Tree;
/// use teistro_intl::words::InWordsRequest;
/// use teistro_intl::{Intl, sdk_root};
///
/// let intl = Intl::from_tree(&Tree::load(&sdk_root())?)?;
/// let words = intl.in_words(&InWordsRequest::from_json(r#"{"number": -42}"#)?)?;
/// assert_eq!(words.text, "minus forty-two");
/// assert_eq!(words.rules, "cldr-48/en/spellout-cardinal");
/// assert!(words.amount.is_none());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "words-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct InWords {
    /// The whole, in words.
    pub text: String,
    /// The figures, grouped, in the digits asked for.
    pub figure: String,
    /// Where the words came from: `cldr-<release>/<file>/<rule set>`.
    pub rules: String,
    /// The locale the words and units are in.
    pub locale: String,
    /// An amount's parts; none for a number.
    pub amount: Option<AmountInWords>,
}

/// An amount's parts, for an application that composes them its own way.
///
/// ```
/// use teistro_intl::source::Tree;
/// use teistro_intl::words::InWordsRequest;
/// use teistro_intl::{Intl, sdk_root};
///
/// let intl = Intl::from_tree(&Tree::load(&sdk_root())?)?;
/// let asked = InWordsRequest::from_json(r#"{"amount": {"currency": "INR", "value": {"decimal": "0.505"}}}"#)?;
/// let parts = intl.in_words(&asked)?.amount.ok_or("an amount's parts")?;
/// assert_eq!((parts.major, parts.minor, parts.rounded), (0, 51, true));
/// assert_eq!(parts.minor_words.as_deref(), Some("fifty-one"));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "words-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct AmountInWords {
    /// The ISO 4217 code.
    pub currency: String,
    /// The major units.
    pub major: u64,
    /// The minor units after them.
    pub minor: u64,
    /// The major units in words.
    pub major_words: String,
    /// The major unit's word for that count.
    pub major_unit: String,
    /// The minor units in words; none for a currency with no minor unit.
    pub minor_words: Option<String>,
    /// The minor unit's word for that count; none with `minor_words`.
    pub minor_unit: Option<String>,
    /// Whether a decimal lost digits to the rounding.
    pub rounded: bool,
}

/// One request's rules, digits and grouping.
struct Speller<'a> {
    file: &'static str,
    sets: &'a [RuleSet],
    index: usize,
    digits: [char; 10],
    /// The first group's size from the right, then every other's.
    groups: (u8, u8),
    decimal: char,
}

impl Speller<'_> {
    fn words(&self, n: i128) -> Result<String, Error> {
        spell(self.sets, self.index, n, &self.digits).map_err(|why| {
            Error::internal(format!("CLDR `{}` could not spell {n}: {why}", self.file))
        })
    }

    fn figure(&self, n: i128) -> String {
        grouped(n, self.groups.0, self.groups.1, &self.digits)
    }
}

/// The CLDR file a language and a grouping read.
fn file_for(language: &str, grouping: WordsGrouping) -> Result<&'static str, Error> {
    match (language, grouping) {
        ("ne", WordsGrouping::Indian) => Ok("ne"),
        ("hi", WordsGrouping::Indian) => Ok("hi"),
        ("en", WordsGrouping::Indian) => Ok("en_IN"),
        ("en", WordsGrouping::International) => Ok("en"),
        ("ne" | "hi", WordsGrouping::International) => Err(Error::unsupported(format!(
            "CLDR {CLDR_RELEASE} gives `{language}` no international grouping's words"
        ))
        .with_field("grouping")
        .with_hint(String::from("`INDIAN`, the only grouping its rules have"))),
        ("sa", _) => Err(Error::unsupported(String::from(
            "Sanskrit numerals are unreviewed: CLDR has no Sanskrit rules, and \
             Whitney's §§475–489 forms wait on a reviewer (C803)",
        ))
        .with_field("locale")
        .with_hint(String::from(
            "docs/03-design/amounts-in-words.md §5; spell in `ne`, `hi` or `en`",
        ))),
        _ => Err(
            Error::unsupported(format!("no number words for the language `{language}`"))
                .with_field("locale")
                .with_hint(String::from("a locale in `ne`, `hi` or `en`")),
        ),
    }
}

/// The rule sets of every CLDR file, parsed once from the generated
/// tables.
fn parsed() -> &'static [Result<Vec<RuleSet>, String>] {
    static PARSED: OnceLock<Vec<Result<Vec<RuleSet>, String>>> = OnceLock::new();
    PARSED.get_or_init(|| {
        SOURCES
            .iter()
            .map(|source| rbnf::parse(source.rules).map_err(|e| e.to_string()))
            .collect()
    })
}

/// A file's rule sets and the index of the set named `name`.
fn rule_set(file: &str, name: &str) -> Result<(&'static [RuleSet], usize), Error> {
    let position = SOURCES
        .iter()
        .position(|source| source.file == file)
        .ok_or_else(|| Error::internal(format!("no CLDR file `{file}` in the tables")))?;
    let sets = parsed()
        .get(position)
        .ok_or_else(|| Error::internal(format!("no parsed tables for `{file}`")))?
        .as_ref()
        .map_err(|why| Error::internal(format!("CLDR `{file}`'s tables do not parse: {why}")))?;
    let index = sets
        .iter()
        .position(|set| set.name == name)
        .ok_or_else(|| {
            Error::unsupported(format!("CLDR {CLDR_RELEASE}'s `{file}` has no `{name}`"))
                .with_field("style")
                .with_hint(String::from("`PLAIN`; `VERBOSE` is English's alone"))
        })?;
    Ok((sets.as_slice(), index))
}

/// An ISO 4217 code: three capital letters, so it can name a message.
fn currency_code(code: &str) -> Result<&str, Error> {
    if code.len() == 3 && code.bytes().all(|b| b.is_ascii_uppercase()) {
        Ok(code)
    } else {
        Err(
            Error::invalid_arg(format!("`{code}` is not an ISO 4217 code"))
                .with_field("amount.currency")
                .with_hint(String::from("three capital letters, as `NPR` or `INR`")),
        )
    }
}

/// A value in minor units, and whether rounding changed it.
fn minor_units(
    value: &WordsAmountValue,
    places: u8,
    rounding: WordsRounding,
) -> Result<(u64, bool), Error> {
    let decimal = match value {
        WordsAmountValue::Minor(units) => return Ok((*units, false)),
        WordsAmountValue::Decimal(decimal) => decimal,
    };
    let refuse = |why: &str| {
        Error::invalid_arg(format!("`{decimal}` {why}"))
            .with_field("amount.value.decimal")
            .with_hint(String::from(
                "ASCII digits with at most one `.`, as `1234.56`; no sign, no exponent",
            ))
    };
    let (whole, fraction) = decimal.split_once('.').unwrap_or((decimal, ""));
    let digits_only = |s: &str| s.bytes().all(|b| b.is_ascii_digit());
    if (whole.is_empty() && fraction.is_empty()) || !digits_only(whole) || !digits_only(fraction) {
        return Err(refuse("is not a decimal this reads"));
    }
    let places = usize::from(places);
    let (kept, dropped) = fraction.split_at(fraction.len().min(places));
    let mut text = String::with_capacity(whole.len() + places);
    text.push_str(whole);
    text.push_str(kept);
    text.extend(std::iter::repeat_n('0', places - kept.len()));
    let significant = text.trim_start_matches('0');
    let units: u64 = if significant.is_empty() {
        0
    } else {
        significant
            .parse::<u64>()
            .map_err(|_| refuse("has more minor units than 2⁶⁴ − 1"))?
    };
    let mut rest = dropped.bytes().map(|b| b - b'0');
    let Some(first) = rest.next().filter(|_| dropped.bytes().any(|b| b != b'0')) else {
        return Ok((units, false));
    };
    let past_half = rest.any(|d| d != 0);
    let up = match rounding {
        WordsRounding::Exact => {
            return Err(refuse(&format!(
                "has digits past the minor unit's {places} places, and `rounding` is `EXACT`"
            )));
        }
        WordsRounding::HalfUp => first >= 5,
        WordsRounding::HalfEven => first > 5 || (first == 5 && (past_half || units % 2 == 1)),
    };
    let units = if up {
        units
            .checked_add(1)
            .ok_or_else(|| refuse("has more minor units than 2⁶⁴ − 1"))?
    } else {
        units
    };
    Ok((units, true))
}

impl Intl {
    /// A number or an amount in words, in a loaded locale, by CLDR's
    /// rules (`docs/03-design/amounts-in-words.md`).
    ///
    /// ```
    /// use teistro_intl::source::Tree;
    /// use teistro_intl::words::InWordsRequest;
    /// use teistro_intl::{Intl, sdk_root};
    ///
    /// let intl = Intl::from_tree(&Tree::load(&sdk_root())?)?;
    /// let lakh = InWordsRequest::from_json(r#"{"number": 150000, "grouping": "INDIAN"}"#)?;
    /// assert_eq!(intl.in_words(&lakh)?.text, "one lakh fifty thousand");
    /// assert_eq!(intl.in_words(&lakh)?.figure, "1,50,000");
    /// let hindi = InWordsRequest::from_json(r#"{"number": 10000000, "locale": "hi-Deva-IN"}"#)?;
    /// assert_eq!(intl.in_words(&hindi)?.text, "एक करोड़");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Named under `inWords`: `INVALID_ARG` for both or neither of
    /// `number` and `amount`, a malformed decimal or currency code, minor
    /// digits past 4, or an `EXACT` decimal that needs rounding;
    /// `UNSUPPORTED` for a locale not loaded, a language without rules
    /// (Sanskrit, unreviewed: C803), a grouping or style the language's
    /// rules lack, or a currency the locale names no units for.
    pub fn in_words(&self, request: &InWordsRequest) -> Result<InWords, Error> {
        self.in_words_unnamed(request)
            .map_err(|e| e.under(IN_WORDS))
    }

    fn in_words_unnamed(&self, request: &InWordsRequest) -> Result<InWords, Error> {
        let tag = request.locale.as_deref().unwrap_or(self.locale());
        let locale = self.locales.get(tag).ok_or_else(|| {
            let known: Vec<&str> = self.locales.keys().map(String::as_str).collect();
            Error::unsupported(format!("the locale `{tag}` is not loaded"))
                .with_field("locale")
                .with_hint(format!("the locales are {}", known.join(", ")))
        })?;
        let language = tag.split('-').next().unwrap_or(tag);
        let grouping = request
            .grouping
            .unwrap_or(if locale.meta.grouping == [3, 2] {
                WordsGrouping::Indian
            } else {
                WordsGrouping::International
            });
        let file = file_for(language, grouping)?;
        let (sets, index) = rule_set(file, request.style.rule_set())?;
        let system: &str = match request.digits {
            Some(asked) => asked.system(),
            None => &locale.meta.numbering_system,
        };
        let speller = Speller {
            file,
            sets,
            index,
            digits: digits(system).unwrap_or(['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']),
            groups: match grouping {
                WordsGrouping::Indian => (3, 2),
                WordsGrouping::International => (3, 3),
            },
            decimal: locale.meta.decimal,
        };
        let rules = format!("cldr-{CLDR_RELEASE}/{file}/{}", request.style.rule_set());
        match (request.number, &request.amount) {
            (Some(number), None) => Ok(InWords {
                text: speller.words(i128::from(number))?,
                figure: speller.figure(i128::from(number)),
                rules,
                locale: tag.to_owned(),
                amount: None,
            }),
            (None, Some(amount)) => {
                let (text, figure, parts) = self.amount(tag, &speller, amount)?;
                Ok(InWords {
                    text,
                    figure,
                    rules,
                    locale: tag.to_owned(),
                    amount: Some(parts),
                })
            }
            (Some(_), Some(_)) => Err(Error::invalid_arg(String::from(
                "both `number` and `amount`; a request writes one",
            ))
            .with_field("amount")),
            (None, None) => Err(Error::invalid_arg(String::from(
                "neither `number` nor `amount`; a request writes one",
            ))
            .with_field("number")),
        }
    }

    /// An amount's words, its figures and its parts.
    fn amount(
        &self,
        tag: &str,
        speller: &Speller<'_>,
        amount: &WordsAmount,
    ) -> Result<(String, String, AmountInWords), Error> {
        let code = currency_code(&amount.currency)?;
        if amount.minor_digits > 4 {
            return Err(Error::invalid_arg(format!(
                "{} minor digits; a currency has 0 to 4",
                amount.minor_digits
            ))
            .with_field("amount.minorDigits"));
        }
        let (units, rounded) = minor_units(&amount.value, amount.minor_digits, amount.rounding)?;
        let scale = 10u64.pow(u32::from(amount.minor_digits));
        let (major, minor) = (units / scale, units % scale);
        let unit = |which: &str, count: u64| self.unit_word(tag, code, which, count);
        let major_words = speller.words(i128::from(major))?;
        let major_unit = unit("major", major)?;
        let (minor_words, minor_unit) = if amount.minor_digits == 0 {
            (None, None)
        } else {
            (
                Some(speller.words(i128::from(minor))?),
                Some(unit("minor", minor)?),
            )
        };
        let shape = match (major, minor) {
            (0, 1..) => "MINOR",
            (_, 1..) => "BOTH",
            _ => "MAJOR",
        };
        let params: Params = [
            ("shape", shape),
            ("majorWords", major_words.as_str()),
            ("majorUnit", major_unit.as_str()),
            ("minorWords", minor_words.as_deref().unwrap_or_default()),
            ("minorUnit", minor_unit.as_deref().unwrap_or_default()),
        ]
        .into_iter()
        .map(|(name, value)| (name.to_owned(), Value::from(value)))
        .collect();
        let mut text = self.message_in(tag, "sdk.amount.words", &params)?;
        if amount.only {
            let only: Params = [(String::from("amount"), Value::from(text))].into();
            text = self.message_in(tag, "sdk.amount.only", &only)?;
        }
        let mut figure = speller.figure(i128::from(major));
        if amount.minor_digits > 0 {
            figure.push(speller.decimal);
            figure.extend(
                format!("{minor:0width$}", width = usize::from(amount.minor_digits))
                    .chars()
                    .map(|c| rbnf::digit(c, &speller.digits)),
            );
        }
        let parts = AmountInWords {
            currency: code.to_owned(),
            major,
            minor,
            major_words,
            major_unit,
            minor_words,
            minor_unit,
            rounded,
        };
        Ok((text, figure, parts))
    }

    /// A currency unit's word for a count, from `sdk.amount.<CODE>.<which>`
    /// in the locale or a fallback in the same language.
    fn unit_word(&self, tag: &str, code: &str, which: &str, count: u64) -> Result<String, Error> {
        let mut params = Params::new();
        params.insert(
            String::from("count"),
            Value::Int(i64::try_from(count).unwrap_or(i64::MAX)),
        );
        self.message_in(tag, &format!("sdk.amount.{code}.{which}"), &params)
            .map_err(|e| e.with_field("amount.currency"))
    }

    /// A message rendered in `tag`, refused where no locale of the same
    /// language has it: an English unit inside Nepali words would be wrong
    /// without a word of warning.
    fn message_in(&self, tag: &str, key: &str, params: &Params) -> Result<String, Error> {
        let language = |t: &str| t.split('-').next().unwrap_or(t).to_owned();
        let rendered = self.render_in(tag, key, params);
        match rendered.resolved_from.as_deref() {
            Some(from) if language(from) == language(tag) && rendered.warnings.is_empty() => {
                Ok(rendered.text)
            }
            Some(from) if language(from) == language(tag) => Err(Error::internal(format!(
                "`{key}` in `{from}` rendered with warnings: {}",
                rendered.warnings.join("; ")
            ))),
            _ => Err(
                Error::unsupported(format!("`{tag}` has no `{key}` in its own language"))
                    .with_hint(format!("add `{key}` to a pack or an override for `{tag}`")),
            ),
        }
    }
}
