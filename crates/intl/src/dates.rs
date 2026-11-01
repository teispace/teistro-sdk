//! Bikram Sambat and Gregorian dates found in free text, in Nepali,
//! Hindi and English, with Devanagari or Latin digits and the months named
//! in any script or romanised (`docs/03-design/lexicon.md` §2, SR-61).
//!
//! The month names are read through the lexicon's fold
//! ([`crate::lexicon::fold`]), so `Shrawan`, `Shravan`, `श्रावण` and `साउन`
//! name one month. A span that reads two ways is answered with both
//! readings, never one of them.
//!
//! ```
//! use teistro_intl::dates::DatesRequest;
//! use teistro_intl::source::Tree;
//! use teistro_intl::{Intl, sdk_root};
//!
//! let intl = Intl::from_tree(&Tree::load(&sdk_root())?)?;
//! let found = intl.dates(&DatesRequest::from_json(r#"{"text": "बिहे २०८२ कात्तिक १५ गते हो"}"#)?)?;
//! assert_eq!(found.dates[0].text, "२०८२ कात्तिक १५ गते");
//! let reading = &found.dates[0].readings[0];
//! assert_eq!((reading.date.year, reading.date.month, reading.date.day), (2082, 7, 15));
//! let gregorian = reading.gregorian.as_ref().ok_or("a Gregorian day")?;
//! assert_eq!((gregorian.year, gregorian.month, gregorian.day), (2025, 11, 1));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::collections::BTreeMap;
use std::sync::{Arc, PoisonError};

use serde::{Deserialize, Serialize};
use teistro_calendar::{CalendarDate, CalendarDay, CalendarSystem, Gregorian, shipped};
use teistro_core::catalogue::{Calendar, Kind, write_in_full};
use teistro_core::error::Error;

use crate::lexicon::fold;
use crate::{Intl, Value, params};

/// The record's name where a binding sends it, which a refusal is named
/// under.
const DATES: &str = "dates";

/// The calendars a date in text is read in.
const READ: [Calendar; 2] = [Calendar::BikramSambat, Calendar::Gregorian];

/// The spellings of the Bikram Sambat's months the packs do not carry,
/// each a month's number and the spellings: romanised as the Government of
/// Nepal and patros printed in Nepal write them, and Devanagari, vernacular
/// and tatsama (`docs/03-design/lexicon.md` §2.2).
const BIKRAM_SAMBAT_SPELLINGS: [(u8, &[&str]); 12] = [
    (
        1,
        &[
            "Baishakh",
            "Baisakh",
            "Vaishakh",
            "Vaisakh",
            "बैशाख",
            "वैशाख",
            "बैसाख",
        ],
    ),
    (
        2,
        &["Jestha", "Jeth", "Jyeshtha", "Jeshtha", "जेठ", "जेष्ठ", "ज्येष्ठ"],
    ),
    (
        3,
        &[
            "Asar",
            "Ashar",
            "Asadh",
            "Ashadh",
            "Ashad",
            "असार",
            "आषाढ",
            "आषाढ़",
            "अषाढ",
        ],
    ),
    (
        4,
        &[
            "Shrawan",
            "Shravan",
            "Srawan",
            "Saun",
            "Sawan",
            "साउन",
            "श्रावण",
            "सावन",
        ],
    ),
    (5, &["Bhadra", "Bhadau", "Bhado", "भदौ", "भाद्र", "भाद्रपद"]),
    (
        6,
        &["Ashwin", "Asoj", "Ashoj", "Aswin", "असोज", "आश्विन", "असौज"],
    ),
    (7, &["Kartik", "Kattik", "कात्तिक", "कार्तिक"]),
    (
        8,
        &[
            "Mangsir",
            "Mansir",
            "Marga",
            "Margashirsha",
            "मंसिर",
            "मङ्सिर",
            "मार्ग",
            "मार्गशीर्ष",
        ],
    ),
    (
        9,
        &["Poush", "Paush", "Push", "Pus", "पुष", "पौष", "पुस", "पूस"],
    ),
    (10, &["Magh", "माघ"]),
    (11, &["Falgun", "Phalgun", "Fagun", "फागुन", "फाल्गुन"]),
    (12, &["Chaitra", "Chait", "चैत", "चैत्र"]),
];

/// The Gregorian months' spellings in Devanagari the packs do not carry:
/// Nepali's and Hindi's, with and without the nukta and the anusvara.
const GREGORIAN_SPELLINGS: [(u8, &[&str]); 12] = [
    (1, &["जनवरी"]),
    (2, &["फेब्रुअरी", "फेब्रुवरी", "फरवरी", "फ़रवरी"]),
    (3, &["मार्च"]),
    (4, &["अप्रिल", "अप्रैल"]),
    (5, &["मे", "मई"]),
    (6, &["जुन", "जून"]),
    (7, &["जुलाई"]),
    (8, &["अगस्ट", "अगस्त"]),
    (9, &["सेप्टेम्बर", "सेप्टेम्बर", "सितम्बर", "सितंबर", "Sept"]),
    (10, &["अक्टोबर", "अक्टूबर", "अक्तूबर"]),
    (11, &["नोभेम्बर", "नवम्बर", "नवंबर"]),
    (12, &["डिसेम्बर", "दिसम्बर", "दिसंबर"]),
];

/// The era markers, each with the calendar it settles: written without
/// their full stops and spaces, compared without regard to case.
const ERAS: [(&str, Calendar); 24] = [
    ("बिसं", Calendar::BikramSambat),
    ("विसं", Calendar::BikramSambat),
    ("बिक्रमसम्बत", Calendar::BikramSambat),
    ("विक्रमसंवत", Calendar::BikramSambat),
    ("विक्रमसम्वत", Calendar::BikramSambat),
    ("बिक्रमसंवत", Calendar::BikramSambat),
    ("संवत", Calendar::BikramSambat),
    ("सम्वत", Calendar::BikramSambat),
    ("bs", Calendar::BikramSambat),
    ("vs", Calendar::BikramSambat),
    ("bikramsambat", Calendar::BikramSambat),
    ("vikramsamvat", Calendar::BikramSambat),
    ("bikramsamvat", Calendar::BikramSambat),
    ("vikramsambat", Calendar::BikramSambat),
    ("ईसं", Calendar::Gregorian),
    ("ई", Calendar::Gregorian),
    ("ईस्वी", Calendar::Gregorian),
    ("ईसवी", Calendar::Gregorian),
    ("इस्वी", Calendar::Gregorian),
    ("सन्", Calendar::Gregorian),
    ("ईस्वीसन्", Calendar::Gregorian),
    ("ad", Calendar::Gregorian),
    ("ce", Calendar::Gregorian),
    ("isvi", Calendar::Gregorian),
];

/// The word a patro printed in Nepal writes after a Bikram Sambat day.
const GATE: &str = "गते";

/// The word Nepali may write after a year.
const SAAL: &str = "साल";

/// The folds of the paksha words that mark a lunar date, which is not
/// read: शुक्ल, कृष्ण, सुदी, बदी, वदी and their romanised spellings.
const LUNAR: [&str; 4] = ["sukl", "krisn", "sudi", "badi"];

/// The English ordinal suffixes a day may carry.
const ORDINALS: [&str; 4] = ["st", "nd", "rd", "th"];

/// What a caller asks: the dates in a text, in the calendars named.
///
/// ```
/// use teistro_core::catalogue::Calendar;
/// use teistro_intl::dates::DatesRequest;
///
/// let asked = DatesRequest::from_json(
///     r#"{"text": "2082/07/15", "calendars": ["calendar.BIKRAM_SAMBAT"]}"#,
/// )?;
/// assert_eq!(asked.calendars, Some(vec![Calendar::BikramSambat]));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "words-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct DatesRequest {
    /// The text.
    pub text: String,
    /// The calendars a reading may be in: `calendar.BIKRAM_SAMBAT`,
    /// `calendar.GREGORIAN`; both when left out.
    #[serde(default)]
    pub calendars: Option<Vec<Calendar>>,
}

impl DatesRequest {
    /// The record a binding sends, as JSON.
    ///
    /// ```
    /// use teistro_intl::dates::DatesRequest;
    ///
    /// let typo = DatesRequest::from_json(r#"{"text": "", "calendar": []}"#);
    /// assert_eq!(typo.unwrap_err().field(), Some("dates.calendar"));
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record or a key it does not
    /// read, named under `dates`.
    pub fn from_json(text: &str) -> Result<DatesRequest, Error> {
        teistro_core::strict::read(text, DATES)
    }
}

/// The order a date's parts were read in.
///
/// ```
/// use teistro_intl::dates::DateOrder;
///
/// assert_eq!(serde_json::to_string(&DateOrder::Dmy)?, "\"DMY\"");
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "words-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DateOrder {
    /// Year, month, day.
    Ymd,
    /// Day, month, year.
    Dmy,
    /// Month, day, year.
    Mdy,
}

/// One reading of a span: a day its calendar has.
///
/// ```
/// use teistro_calendar::CalendarDay;
/// use teistro_core::catalogue::Calendar;
/// use teistro_intl::dates::{DateOrder, DateReading};
///
/// let date = CalendarDay { calendar: Calendar::Gregorian, year: 2026, month: 10, day: 31, era: None, leap: None };
/// let reading = DateReading { date, order: DateOrder::Ymd, gregorian: None };
/// assert_eq!(serde_json::to_value(&reading)?["order"], "YMD");
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "words-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct DateReading {
    /// The day, in the calendar read.
    pub date: CalendarDay,
    /// The order its parts were read in.
    pub order: DateOrder,
    /// The Gregorian day it is, for a reading in another calendar.
    pub gregorian: Option<CalendarDay>,
}

/// A span of the text that is a date, with every reading of it.
///
/// ```
/// use teistro_intl::dates::FoundDate;
///
/// let found = FoundDate { start: 0, end: 10, text: "2026-10-31".into(), readings: Vec::new() };
/// assert_eq!(found.end, 10);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "words-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct FoundDate {
    /// Where the span begins, counted in Unicode scalar values.
    pub start: usize,
    /// Where it ends, exclusive, in the same count.
    pub end: usize,
    /// The span's text, its era marker included.
    pub text: String,
    /// Every reading, Bikram Sambat first, then in the order read.
    pub readings: Vec<DateReading>,
}

/// The dates found in a text.
///
/// ```
/// use teistro_intl::dates::{DateOrder, DatesRequest};
/// use teistro_intl::source::Tree;
/// use teistro_intl::{Intl, sdk_root};
///
/// let intl = Intl::from_tree(&Tree::load(&sdk_root())?)?;
/// let asked = DatesRequest::from_json(r#"{"text": "on 05/06/2025", "calendars": ["calendar.GREGORIAN"]}"#)?;
/// let orders: Vec<DateOrder> = intl.dates(&asked)?.dates[0].readings.iter().map(|r| r.order).collect();
/// // The fifth of June or the sixth of May: both.
/// assert_eq!(orders, [DateOrder::Dmy, DateOrder::Mdy]);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "words-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct FoundDates {
    /// The spans, in the text's order.
    pub dates: Vec<FoundDate>,
}

impl FoundDates {
    /// Where the answer carries a catalogue member, and of which kind.
    pub const MEMBERS: [(&'static str, Kind); 2] = [
        ("dates.readings.date.calendar", Kind::Calendar),
        ("dates.readings.gregorian.calendar", Kind::Calendar),
    ];

    /// The answer with every catalogue member written as its full key
    /// (`calendar.BIKRAM_SAMBAT`): what the boundary carries.
    ///
    /// ```
    /// use teistro_intl::dates::DatesRequest;
    /// use teistro_intl::source::Tree;
    /// use teistro_intl::{Intl, sdk_root};
    ///
    /// let intl = Intl::from_tree(&Tree::load(&sdk_root())?)?;
    /// let asked = DatesRequest::from_json(r#"{"text": "२०८२/०७/१५ गते"}"#)?;
    /// let full = intl.dates(&asked)?.in_full()?;
    /// let reading = &full["dates"][0]["readings"][0];
    /// assert_eq!(reading["date"]["calendar"], "calendar.BIKRAM_SAMBAT");
    /// assert_eq!(reading["gregorian"]["calendar"], "calendar.GREGORIAN");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INTERNAL` should the answer not serialise.
    pub fn in_full(&self) -> Result<serde_json::Value, Error> {
        let mut value = serde_json::to_value(self)
            .map_err(|why| Error::internal(format!("the dates did not serialise: {why}")))?;
        for (path, kind) in Self::MEMBERS {
            write_in_full(&mut value, path, kind);
        }
        Ok(value)
    }
}

/// Every month name by its fold, with the calendar and month it names.
#[derive(Debug, Default)]
pub(crate) struct Months {
    by_fold: BTreeMap<String, Vec<(Calendar, u8)>>,
}

impl Months {
    fn add(&mut self, spelt: &str, calendar: Calendar, month: u8) {
        let folded = fold(spelt);
        if folded.is_empty() {
            return;
        }
        let slot = self.by_fold.entry(folded).or_default();
        if !slot.contains(&(calendar, month)) {
            slot.push((calendar, month));
        }
    }

    fn named(&self, word: &str) -> Option<&[(Calendar, u8)]> {
        self.by_fold.get(&fold(word)).map(Vec::as_slice)
    }
}

/// One token of a text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Token<'t> {
    /// Figures, with their value and how many there are.
    Number(u32, usize),
    /// A word.
    Word(&'t str),
    /// A separator: `/`, `-`, `.`, `,`, or any other mark (`?`).
    Mark(char),
}

#[derive(Clone, Copy, Debug)]
struct Placed<'t> {
    token: Token<'t>,
    start: usize,
    end: usize,
}

/// The value of a digit, Latin or Devanagari.
fn digit(c: char) -> Option<u32> {
    match c {
        '0'..='9' => c.to_digit(10),
        '\u{966}'..='\u{96f}' => Some(u32::from(c) - 0x966),
        _ => None,
    }
}

fn in_word(c: char) -> bool {
    c.is_alphabetic()
        || ('\u{900}'..='\u{963}').contains(&c)
        || ('\u{971}'..='\u{97f}').contains(&c)
        || ('\u{300}'..='\u{36f}').contains(&c)
        || matches!(c, '\u{200c}' | '\u{200d}')
}

/// A text's tokens, each with its span in scalar values.
fn tokens(text: &str) -> Vec<Placed<'_>> {
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut out = Vec::new();
    let mut at = 0;
    while let Some(&(byte, c)) = chars.get(at) {
        let start = at;
        if digit(c).is_some() {
            let mut value: u32 = 0;
            while let Some(d) = chars.get(at).and_then(|&(_, c)| digit(c)) {
                value = value.saturating_mul(10).saturating_add(d);
                at += 1;
            }
            out.push(Placed {
                token: Token::Number(value, at - start),
                start,
                end: at,
            });
        } else if in_word(c) {
            while chars.get(at).is_some_and(|&(_, c)| in_word(c)) {
                at += 1;
            }
            let until = chars.get(at).map_or(text.len(), |&(b, _)| b);
            out.push(Placed {
                token: Token::Word(text.get(byte..until).unwrap_or_default()),
                start,
                end: at,
            });
        } else {
            at += 1;
            if !c.is_whitespace() {
                let mark = if matches!(c, '/' | '-' | '.' | ',') {
                    c
                } else {
                    '?'
                };
                out.push(Placed {
                    token: Token::Mark(mark),
                    start,
                    end: at,
                });
            }
        }
    }
    out
}

/// What a span was read as, before its calendars are settled.
#[derive(Clone, Debug)]
struct Shape {
    /// The token the span begins at and the one after its end.
    from: usize,
    to: usize,
    /// The readings asked of the calendars: (year, month, day, order), and
    /// the calendars a named month allows.
    parts: Vec<(i32, u8, u8, DateOrder)>,
    named: Option<Vec<Calendar>>,
    /// The calendar `गते` settles.
    gate: bool,
}

/// A cursor over the tokens, for the shapes' grammar.
struct Cursor<'a, 't> {
    tokens: &'a [Placed<'t>],
    at: usize,
}

impl<'t> Cursor<'_, 't> {
    fn peek(&self) -> Option<Token<'t>> {
        self.tokens.get(self.at).map(|placed| placed.token)
    }

    /// A number of `digits` digits (one or two, for 0), its ordinal
    /// suffix taken with it when the suffix touches it.
    fn number(&mut self, digits: usize) -> Option<u32> {
        let Some(Token::Number(value, count)) = self.peek() else {
            return None;
        };
        let fits = if digits == 0 {
            (1..=2).contains(&count)
        } else {
            count == digits
        };
        if !fits {
            return None;
        }
        let end = self.tokens.get(self.at).map(|placed| placed.end);
        self.at += 1;
        if let Some(next) = self.tokens.get(self.at)
            && let Token::Word(word) = next.token
            && Some(next.start) == end
            && ORDINALS.contains(&word.to_ascii_lowercase().as_str())
        {
            self.at += 1;
        }
        Some(value)
    }

    fn word(&mut self, expected: &str) -> bool {
        if self.peek() == Some(Token::Word(expected)) {
            self.at += 1;
            true
        } else {
            false
        }
    }

    /// A separator between named parts, if one stands there.
    fn optional_mark(&mut self) {
        if matches!(self.peek(), Some(Token::Mark(',' | '-' | '/'))) {
            self.at += 1;
        }
    }

    fn month(&mut self, months: &Months) -> Option<Vec<(Calendar, u8)>> {
        let Some(Token::Word(word)) = self.peek() else {
            return None;
        };
        let named = months.named(word)?.to_vec();
        self.at += 1;
        Some(named)
    }
}

/// Every calendar a named month's names allow, and the month in each.
fn by_calendar(named: &[(Calendar, u8)]) -> Vec<(Calendar, u8)> {
    let mut out: Vec<(Calendar, u8)> = Vec::new();
    for &(calendar, month) in named {
        if !out.contains(&(calendar, month)) {
            out.push((calendar, month));
        }
    }
    out
}

/// A date spelt with a month's name, from token `from`.
fn named_shape(tokens: &[Placed<'_>], from: usize, months: &Months) -> Option<Shape> {
    let year = |value: u32| i32::try_from(value).ok();
    let day = |value: u32| u8::try_from(value).ok().filter(|d| (1..=32).contains(d));
    // Year, month, day.
    let mut at = Cursor { tokens, at: from };
    if let Some(y) = at.number(4).and_then(year) {
        at.word(SAAL);
        at.optional_mark();
        if let Some(named) = at.month(months) {
            at.optional_mark();
            if let Some(d) = at.number(0).and_then(day) {
                let gate = at.word(GATE);
                return Some(shaped(from, at.at, &named, y, d, DateOrder::Ymd, gate));
            }
        }
    }
    // Day, month, year.
    let mut at = Cursor { tokens, at: from };
    if let Some(d) = at.number(0).and_then(day) {
        let gate = at.word(GATE);
        at.optional_mark();
        if let Some(named) = at.month(months) {
            at.optional_mark();
            if let Some(y) = at.number(4).and_then(year) {
                return Some(shaped(from, at.at, &named, y, d, DateOrder::Dmy, gate));
            }
        }
    }
    // Month, day, year.
    let mut at = Cursor { tokens, at: from };
    if let Some(named) = at.month(months)
        && let Some(d) = at.number(0).and_then(day)
    {
        let gate = at.word(GATE);
        at.optional_mark();
        if let Some(y) = at.number(4).and_then(year) {
            return Some(shaped(from, at.at, &named, y, d, DateOrder::Mdy, gate));
        }
    }
    None
}

fn shaped(
    from: usize,
    to: usize,
    named: &[(Calendar, u8)],
    year: i32,
    day: u8,
    order: DateOrder,
    gate: bool,
) -> Shape {
    let named = by_calendar(named);
    Shape {
        from,
        to,
        parts: named
            .iter()
            .map(|&(_, month)| (year, month, day, order))
            .collect(),
        named: Some(named.iter().map(|&(calendar, _)| calendar).collect()),
        gate,
    }
}

/// A date in figures, from token `from`: year first, or year last read
/// both ways.
fn numeric_shape(tokens: &[Placed<'_>], from: usize) -> Option<Shape> {
    let number = |at: usize| match tokens.get(at).map(|placed| placed.token) {
        Some(Token::Number(value, digits)) => Some((value, digits)),
        _ => None,
    };
    let mark = |at: usize| match tokens.get(at).map(|placed| placed.token) {
        Some(Token::Mark(mark @ ('/' | '-' | '.'))) => Some(mark),
        _ => None,
    };
    let (first, first_digits) = number(from)?;
    let separator = mark(from + 1)?;
    let (second, second_digits) = number(from + 2)?;
    (mark(from + 3)? == separator).then_some(())?;
    let (third, third_digits) = number(from + 4)?;
    let small = |value: u32, digits: usize| {
        u8::try_from(value)
            .ok()
            .filter(|_| (1..=2).contains(&digits))
    };
    let mut to = from + 5;
    let gate = tokens.get(to).map(|placed| placed.token) == Some(Token::Word(GATE));
    if gate {
        to += 1;
    }
    let parts = if first_digits == 4 {
        let year = i32::try_from(first).ok()?;
        vec![(
            year,
            small(second, second_digits)?,
            small(third, third_digits)?,
            DateOrder::Ymd,
        )]
    } else if third_digits == 4 {
        let year = i32::try_from(third).ok()?;
        let (a, b) = (small(first, first_digits)?, small(second, second_digits)?);
        vec![(year, b, a, DateOrder::Dmy), (year, a, b, DateOrder::Mdy)]
    } else {
        return None;
    };
    Some(Shape {
        from,
        to,
        parts,
        named: None,
        gate,
    })
}

/// An era marker of up to five tokens (words and full stops) from `from`
/// onward, with the token after it.
fn era_after(tokens: &[Placed<'_>], from: usize) -> Option<(Calendar, usize)> {
    let mut written = String::new();
    let mut found = None;
    for (offset, placed) in tokens.get(from..)?.iter().take(5).enumerate() {
        match placed.token {
            Token::Word(word) => written.push_str(&word.to_lowercase()),
            Token::Mark('.') if offset > 0 => {}
            _ => break,
        }
        if let Some(&(_, calendar)) = ERAS.iter().find(|(marker, _)| *marker == written) {
            found = Some((calendar, from + offset + 1));
        }
    }
    // A full stop closing the marker is part of it.
    found.map(|(calendar, to)| {
        let closed = tokens.get(to).map(|placed| placed.token) == Some(Token::Mark('.'));
        (calendar, if closed { to + 1 } else { to })
    })
}

/// An era marker ending just before token `to`, with the token it begins
/// at.
fn era_before(tokens: &[Placed<'_>], to: usize) -> Option<(Calendar, usize)> {
    (1..=6)
        .filter_map(|length| to.checked_sub(length))
        .find_map(|from| match era_after(tokens, from) {
            Some((calendar, end)) if end == to => Some((calendar, from)),
            _ => None,
        })
}

/// Whether a paksha word stands in the span or just after it: a lunar
/// date, not read.
fn lunar(tokens: &[Placed<'_>], from: usize, to: usize) -> bool {
    tokens
        .get(from..=to.min(tokens.len().saturating_sub(1)))
        .unwrap_or_default()
        .iter()
        .any(|placed| matches!(placed.token, Token::Word(word) if LUNAR.contains(&fold(word).as_str())))
}

impl Intl {
    /// Every Bikram Sambat and Gregorian date in a text, each span with
    /// every reading that is a day its calendar has
    /// (`docs/03-design/lexicon.md` §2).
    ///
    /// ```
    /// use teistro_intl::dates::DatesRequest;
    /// use teistro_intl::source::Tree;
    /// use teistro_intl::{Intl, sdk_root};
    ///
    /// let intl = Intl::from_tree(&Tree::load(&sdk_root())?)?;
    /// // Figures alone read in both calendars; a marker settles one.
    /// let both = intl.dates(&DatesRequest::from_json(r#"{"text": "2082-07-15"}"#)?)?;
    /// assert_eq!(both.dates[0].readings.len(), 2);
    /// let settled = intl.dates(&DatesRequest::from_json(r#"{"text": "2082-07-15 B.S."}"#)?)?;
    /// assert_eq!(settled.dates[0].text, "2082-07-15 B.S.");
    /// assert_eq!(settled.dates[0].readings.len(), 1);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `UNSUPPORTED` for a calendar other than the Bikram Sambat and the
    /// Gregorian, named under `dates.calendars`.
    pub fn dates(&self, request: &DatesRequest) -> Result<FoundDates, Error> {
        let calendars = match &request.calendars {
            Some(asked) => {
                if let Some(other) = asked.iter().find(|calendar| !READ.contains(calendar)) {
                    return Err(Error::unsupported(format!(
                        "dates are read in the Bikram Sambat and the Gregorian, not `{}`",
                        other.full_key()
                    ))
                    .with_field("calendars")
                    .under(DATES));
                }
                asked.clone()
            }
            None => READ.to_vec(),
        };
        let months = self.months();
        let tokens = tokens(&request.text);
        let scalars: Vec<char> = request.text.chars().collect();
        let mut found = Vec::new();
        let mut at = 0;
        while at < tokens.len() {
            let Some(shape) =
                named_shape(&tokens, at, &months).or_else(|| numeric_shape(&tokens, at))
            else {
                at += 1;
                continue;
            };
            if lunar(&tokens, shape.from, shape.to) {
                at = shape.to;
                continue;
            }
            let before = era_before(&tokens, shape.from);
            let after = era_after(&tokens, shape.to);
            let first = before.map_or(shape.from, |(_, from)| from);
            let last = after.map_or(shape.to, |(_, to)| to);
            let marked = after.or(before).map(|(calendar, _)| calendar);
            let readings = readings(&shape, &calendars, marked);
            if !readings.is_empty() {
                let start = tokens.get(first).map_or(0, |placed| placed.start);
                let end = last
                    .checked_sub(1)
                    .and_then(|at| tokens.get(at))
                    .map_or(start, |placed| placed.end);
                found.push(FoundDate {
                    start,
                    end,
                    text: scalars
                        .get(start..end)
                        .map(|chars| chars.iter().collect())
                        .unwrap_or_default(),
                    readings,
                });
            }
            at = last.max(at + 1);
        }
        Ok(FoundDates { dates: found })
    }

    /// The month names, built the first time a text is read for dates.
    fn months(&self) -> Arc<Months> {
        if let Some(months) = self
            .months
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .as_ref()
        {
            return Arc::clone(months);
        }
        let mut months = Months::default();
        let tags: Vec<String> = self.locales.keys().cloned().collect();
        for calendar in READ {
            let named = format!("sdk.calendar.{}", calendar.key());
            for tag in &tags {
                for month in 1..=12_u8 {
                    let given = params([("month", Value::Int(i64::from(month)))]);
                    let mut keys = vec![format!("{named}.monthName")];
                    // The Bikram Sambat's short forms (`Man`, `Cha`) are
                    // English words; the Gregorian's (`Oct`) are dates'.
                    if calendar == Calendar::Gregorian {
                        keys.push(format!("{named}.monthShort"));
                    }
                    for key in keys {
                        let rendered = self.render_in(tag, &key, &given);
                        if rendered.resolved_from.is_some() {
                            months.add(rendered.text.trim_end_matches('.'), calendar, month);
                        }
                    }
                }
            }
        }
        for (month, spellings) in BIKRAM_SAMBAT_SPELLINGS {
            for spelt in spellings {
                months.add(spelt, Calendar::BikramSambat, month);
            }
        }
        for (month, spellings) in GREGORIAN_SPELLINGS {
            for spelt in spellings {
                months.add(spelt, Calendar::Gregorian, month);
            }
        }
        let built = Arc::new(months);
        *self.months.write().unwrap_or_else(PoisonError::into_inner) = Some(Arc::clone(&built));
        built
    }
}

/// The readings of a shape that are days of their calendars, in the
/// calendars asked and the one a marker or `गते` settles.
fn readings(shape: &Shape, asked: &[Calendar], marked: Option<Calendar>) -> Vec<DateReading> {
    let settled = marked.or(shape.gate.then_some(Calendar::BikramSambat));
    let mut out = Vec::new();
    for calendar in READ {
        if !asked.contains(&calendar) || settled.is_some_and(|only| only != calendar) {
            continue;
        }
        for (at, &(year, month, day, order)) in shape.parts.iter().enumerate() {
            let named_here = shape
                .named
                .as_ref()
                .is_none_or(|named| named.get(at) == Some(&calendar));
            if !named_here {
                continue;
            }
            let date = CalendarDate::defined(calendar, year, month, day);
            let Some(system) = shipped(calendar) else {
                continue;
            };
            let Ok(fixed) = system.fixed_of(&date) else {
                continue;
            };
            let gregorian = (calendar != Calendar::Gregorian)
                .then(|| Gregorian.date_of(fixed).ok())
                .flatten()
                .map(|day| CalendarDay {
                    era: None,
                    ..CalendarDay::from(&day)
                });
            out.push(DateReading {
                date: CalendarDay::from(&date),
                order,
                gregorian,
            });
        }
    }
    out
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::{Token, tokens};

    #[test]
    fn figures_in_either_script_are_numbers() {
        let read: Vec<Token<'_>> = tokens("२०८२/07-१५ गते.").iter().map(|p| p.token).collect();
        assert_eq!(
            read,
            [
                Token::Number(2082, 4),
                Token::Mark('/'),
                Token::Number(7, 2),
                Token::Mark('-'),
                Token::Number(15, 2),
                Token::Word("गते"),
                Token::Mark('.'),
            ]
        );
    }
}
