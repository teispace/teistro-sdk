//! A date or a time of day as text and parts: `:time` and `:date` as
//! calls of their own, through the same locale patterns, for an application
//! that lays a time out rather than writing it into a sentence
//! (`docs/03-design/intl-engine-and-packs.md`, "A date or a time as text
//! and parts").
//!
//! ```
//! use teistro_intl::civil::{CivilPartKind, TimeTextRequest};
//! use teistro_intl::source::Tree;
//! use teistro_intl::{Intl, sdk_root};
//!
//! let intl = Intl::from_tree(&Tree::load(&sdk_root())?)?;
//! let asked = TimeTextRequest::from_json(
//!     r#"{"at": {"hour": 6, "minute": 5}, "hour12": true, "locale": "ne-Deva-NP"}"#,
//! )?;
//! let dawn = intl.time(&asked)?;
//! assert_eq!(dawn.text, "बिहान ६:०५");
//! assert_eq!(dawn.parts[0].kind, CivilPartKind::DayPeriod);
//! assert_eq!(dawn.parts[0].text, "बिहान");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use serde::{Deserialize, Serialize};
use teistro_calendar::{CalendarDay, shipped};
use teistro_core::error::Error;

use crate::mf2::ast::MarkupKind;
use crate::render::{ClockTime, OutPart, digits};
use crate::source::LocaleSource;
use crate::words::WordsDigits;
use crate::{Intl, Rendered};

/// The record's name where a binding sends a time, which a refusal is
/// named under.
const TIME: &str = "time";
/// The record's name where a binding sends a date.
const DATE: &str = "date";

/// How much of a time of day is written.
///
/// ```
/// use teistro_intl::civil::TimeStyle;
///
/// assert_eq!(TimeStyle::default(), TimeStyle::Numeric);
/// assert_eq!(TimeStyle::Long.key(), "long");
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "words-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TimeStyle {
    /// Hours and minutes.
    #[default]
    Numeric,
    /// Hours, minutes and seconds.
    Long,
}

impl TimeStyle {
    /// The style's name in `sdk.calendar.time`.
    ///
    /// ```
    /// use teistro_intl::civil::TimeStyle;
    ///
    /// assert_eq!(TimeStyle::Numeric.key(), "numeric");
    /// ```
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            TimeStyle::Numeric => "numeric",
            TimeStyle::Long => "long",
        }
    }
}

/// How much of a date is written.
///
/// ```
/// use teistro_intl::civil::DateStyle;
///
/// assert_eq!(DateStyle::default(), DateStyle::Numeric);
/// assert_eq!(DateStyle::Full.key(), "full");
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "words-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DateStyle {
    /// In figures.
    #[default]
    Numeric,
    /// With the month's name.
    Long,
    /// With the weekday's name too.
    Full,
}

impl DateStyle {
    /// The style's name in a calendar's `date` patterns.
    ///
    /// ```
    /// use teistro_intl::civil::DateStyle;
    ///
    /// assert_eq!(DateStyle::Long.key(), "long");
    /// ```
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            DateStyle::Numeric => "numeric",
            DateStyle::Long => "long",
            DateStyle::Full => "full",
        }
    }
}

/// What a caller asks to be written: a time of day, and how.
///
/// ```
/// use teistro_intl::civil::TimeTextRequest;
///
/// let asked = TimeTextRequest::from_json(r#"{"at": {"hour": 18, "minute": 30}}"#)?;
/// assert_eq!((asked.at.hour, asked.at.second), (18, 0));
/// assert!(!asked.hour12);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "words-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct TimeTextRequest {
    /// The time of day.
    pub at: ClockTime,
    /// How much of it.
    #[serde(default)]
    pub style: TimeStyle,
    /// The locale's twelve-hour pattern, which writes the part of the day
    /// its readers expect; the twenty-four-hour one when left out.
    #[serde(default)]
    pub hour12: bool,
    /// The locale, a loaded tag (`ne-Deva-NP`); the engine's current one
    /// when left out.
    #[serde(default)]
    pub locale: Option<String>,
    /// The digits; the locale's numbering system when left out.
    #[serde(default)]
    pub digits: Option<WordsDigits>,
}

impl TimeTextRequest {
    /// The record a binding sends, as JSON.
    ///
    /// ```
    /// use teistro_intl::civil::TimeTextRequest;
    ///
    /// let typo = TimeTextRequest::from_json(r#"{"at": {"hour": 1, "minute": 2}, "hours12": true}"#);
    /// assert_eq!(typo.unwrap_err().field(), Some("time.hours12"));
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record or a key it does not
    /// read, named under `time`.
    pub fn from_json(text: &str) -> Result<TimeTextRequest, Error> {
        teistro_core::strict::read(text, TIME)
    }
}

/// What a caller asks to be written: a date in any calendar, and how.
///
/// ```
/// use teistro_intl::civil::{DateStyle, DateTextRequest};
///
/// let asked = DateTextRequest::from_json(
///     r#"{"at": {"calendar": "calendar.BIKRAM_SAMBAT", "year": 2081, "month": 1, "day": 1}, "style": "LONG"}"#,
/// )?;
/// assert_eq!(asked.style, DateStyle::Long);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "words-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct DateTextRequest {
    /// The date, in its own calendar.
    pub at: CalendarDay,
    /// How much of it.
    #[serde(default)]
    pub style: DateStyle,
    /// The locale, a loaded tag (`ne-Deva-NP`); the engine's current one
    /// when left out.
    #[serde(default)]
    pub locale: Option<String>,
    /// The digits; the locale's numbering system when left out.
    #[serde(default)]
    pub digits: Option<WordsDigits>,
}

impl DateTextRequest {
    /// The record a binding sends, as JSON.
    ///
    /// ```
    /// use teistro_intl::civil::DateTextRequest;
    ///
    /// let late = DateTextRequest::from_json(r#"{"at": {"calendar": "calendar.GREGORIAN"}}"#);
    /// assert!(late.is_err());
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record or a key it does not
    /// read, named under `date`.
    pub fn from_json(text: &str) -> Result<DateTextRequest, Error> {
        teistro_core::strict::read(text, DATE)
    }
}

/// What one part of a date or a time is.
///
/// ```
/// use teistro_intl::civil::CivilPartKind;
///
/// assert_eq!(serde_json::to_string(&CivilPartKind::DayPeriod)?, "\"DAY_PERIOD\"");
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "words-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CivilPartKind {
    /// The year.
    Year,
    /// The month, by number or name; an ISO week date's week.
    Month,
    /// The day of the month.
    Day,
    /// The weekday's name.
    Weekday,
    /// The era's short form.
    Era,
    /// The hour, on the clock asked for.
    Hour,
    /// The minute.
    Minute,
    /// The second.
    Second,
    /// The part of the day, or am and pm.
    DayPeriod,
    /// Whatever the pattern spells itself.
    Literal,
}

impl CivilPartKind {
    /// The part a pattern's variable writes.
    fn of(variable: &str) -> CivilPartKind {
        match variable {
            "year" => CivilPartKind::Year,
            "month" | "monthName" => CivilPartKind::Month,
            "day" => CivilPartKind::Day,
            "weekday" | "weekdayName" => CivilPartKind::Weekday,
            "era" => CivilPartKind::Era,
            "hour" | "hour12" => CivilPartKind::Hour,
            "minute" => CivilPartKind::Minute,
            "second" => CivilPartKind::Second,
            "dayPeriod" | "meridiem" => CivilPartKind::DayPeriod,
            _ => CivilPartKind::Literal,
        }
    }
}

/// One part of a date or a time.
///
/// ```
/// use teistro_intl::civil::{CivilPart, CivilPartKind};
///
/// let part = CivilPart { kind: CivilPartKind::Hour, text: String::from("6") };
/// assert_eq!(serde_json::to_value(&part)?["kind"], "HOUR");
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "words-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct CivilPart {
    /// What it is.
    pub kind: CivilPartKind,
    /// Its text.
    pub text: String,
}

/// A date or a time as text, and in parts.
///
/// ```
/// use teistro_intl::civil::DateTextRequest;
/// use teistro_intl::source::Tree;
/// use teistro_intl::{Intl, sdk_root};
///
/// let intl = Intl::from_tree(&Tree::load(&sdk_root())?)?;
/// let asked = DateTextRequest::from_json(
///     r#"{"at": {"calendar": "calendar.GREGORIAN", "year": 2026, "month": 10, "day": 31}, "style": "FULL"}"#,
/// )?;
/// let day = intl.date(&asked)?;
/// assert_eq!(day.text, "Saturday, 31 October 2026");
/// let joined: String = day.parts.iter().map(|part| part.text.as_str()).collect();
/// assert_eq!(joined, day.text);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "words-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct CivilText {
    /// The whole.
    pub text: String,
    /// The parts, in the locale's order; joined, they are `text`.
    pub parts: Vec<CivilPart>,
    /// The locale asked for.
    pub locale: String,
    /// The locale whose pattern answered.
    pub resolved_from: String,
    /// Whether a fallback locale answered.
    pub is_fallback: bool,
    /// Whether any of it is text no reviewer has signed off.
    pub is_draft: bool,
    /// Every problem met on the way.
    pub warnings: Vec<String>,
}

/// A render's marked parts as typed ones.
fn typed_parts(parts: &[OutPart], system: Option<WordsDigits>) -> Vec<CivilPart> {
    let figures = system.and_then(|asked| digits(asked.system()));
    let mut field = CivilPartKind::Literal;
    let mut out: Vec<CivilPart> = Vec::with_capacity(parts.len());
    for part in parts {
        match part {
            OutPart::Text(text) if text.is_empty() => {}
            OutPart::Text(text) => {
                let text = figures.map_or_else(|| text.clone(), |to| in_digits(text, to));
                match out.last_mut() {
                    Some(last) if last.kind == field => last.text.push_str(&text),
                    _ => out.push(CivilPart { kind: field, text }),
                }
            }
            OutPart::Markup { kind, name, .. } => {
                field = match kind {
                    MarkupKind::Open => CivilPartKind::of(name),
                    MarkupKind::Close | MarkupKind::Standalone => CivilPartKind::Literal,
                };
            }
        }
    }
    out
}

/// Text with its Latin and Devanagari digits in the digits given.
fn in_digits(text: &str, to: [char; 10]) -> String {
    text.chars()
        .map(|c| {
            let digit = match c {
                '0'..='9' => c.to_digit(10),
                '\u{966}'..='\u{96F}' => Some(u32::from(c) - 0x966),
                _ => None,
            };
            digit.and_then(|d| to.get(d as usize).copied()).unwrap_or(c)
        })
        .collect()
}

impl Intl {
    pub(crate) fn loaded_locale(&self, tag: &str) -> Result<&LocaleSource, Error> {
        self.locales.get(tag).map(AsRef::as_ref).ok_or_else(|| {
            let known: Vec<&str> = self.locales.keys().map(String::as_str).collect();
            Error::unsupported(format!("the locale `{tag}` is not loaded"))
                .with_field("locale")
                .with_hint(format!("the locales are {}", known.join(", ")))
        })
    }

    /// A time of day as text and parts, through the locale's own pattern
    /// (`sdk.calendar.time.<style>`, or `<style>12`) and parts of the day.
    ///
    /// ```
    /// use teistro_intl::civil::TimeTextRequest;
    /// use teistro_intl::source::Tree;
    /// use teistro_intl::{Intl, sdk_root};
    ///
    /// let intl = Intl::from_tree(&Tree::load(&sdk_root())?)?;
    /// let noon = TimeTextRequest::from_json(r#"{"at": {"hour": 12, "minute": 0}, "hour12": true}"#)?;
    /// assert_eq!(intl.time(&noon)?.text, "12:00 pm");
    /// let hour_25 = TimeTextRequest::from_json(r#"{"at": {"hour": 25, "minute": 0}}"#)?;
    /// assert_eq!(intl.time(&hour_25).unwrap_err().field(), Some("time.at.hour"));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Named under `time`: `INVALID_ARG` for an hour, minute or second out
    /// of range; `UNSUPPORTED` for a locale not loaded.
    pub fn time(&self, request: &TimeTextRequest) -> Result<CivilText, Error> {
        self.time_unnamed(request).map_err(|e| e.under(TIME))
    }

    fn time_unnamed(&self, request: &TimeTextRequest) -> Result<CivilText, Error> {
        let at = request.at;
        for (field, value, most) in [
            ("at.hour", at.hour, 23),
            ("at.minute", at.minute, 59),
            ("at.second", at.second, 60),
        ] {
            if value > most {
                return Err(Error::invalid_arg(format!("{value} is past {most}")).with_field(field));
            }
        }
        let tag = request.locale.as_deref().unwrap_or(self.locale());
        let locale = self.loaded_locale(tag)?;
        let (key, rendered) = self.civil_time(locale, tag, at, request.style.key(), request.hour12);
        answer(tag, &key, rendered, request.digits, "style")
    }

    /// A date as text and parts, in its own calendar, through the locale's
    /// pattern (`sdk.calendar.<CALENDAR>.date.<style>`).
    ///
    /// ```
    /// use teistro_intl::civil::DateTextRequest;
    /// use teistro_intl::source::Tree;
    /// use teistro_intl::{Intl, sdk_root};
    ///
    /// let intl = Intl::from_tree(&Tree::load(&sdk_root())?)?;
    /// let asked = DateTextRequest::from_json(
    ///     r#"{"at": {"calendar": "calendar.GREGORIAN", "year": 2026, "month": 2, "day": 30}}"#,
    /// )?;
    /// assert_eq!(intl.date(&asked).unwrap_err().field().map(|f| f.starts_with("date.at")), Some(true));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Named under `date`: `INVALID_ARG` for a date its calendar does not
    /// hold; `UNSUPPORTED` for a locale not loaded, a calendar not shipped,
    /// or one the locale chain has no pattern for.
    pub fn date(&self, request: &DateTextRequest) -> Result<CivilText, Error> {
        self.date_unnamed(request).map_err(|e| e.under(DATE))
    }

    fn date_unnamed(&self, request: &DateTextRequest) -> Result<CivilText, Error> {
        let date = request.at.date();
        let system = shipped(date.calendar).ok_or_else(|| {
            Error::unsupported(format!(
                "`{}` is not a shipped calendar",
                date.calendar.key()
            ))
            .with_field("at.calendar")
        })?;
        system.fixed_of(&date).map_err(|e| e.under("at"))?;
        let tag = request.locale.as_deref().unwrap_or(self.locale());
        let locale = self.loaded_locale(tag)?;
        let (key, rendered) = self.civil_date(locale, tag, &date, request.style.key());
        answer(tag, &key, rendered, request.digits, "at.calendar")
    }
}

/// The answer from a render, or the refusal of a pattern nothing holds.
fn answer(
    tag: &str,
    key: &str,
    rendered: Rendered,
    digits: Option<WordsDigits>,
    field: &str,
) -> Result<CivilText, Error> {
    let Some(resolved_from) = rendered.resolved_from else {
        return Err(
            Error::unsupported(format!("`{tag}` and its fallbacks have no `{key}`"))
                .with_field(field),
        );
    };
    let parts = typed_parts(&rendered.parts, digits);
    Ok(CivilText {
        text: parts.iter().map(|part| part.text.as_str()).collect(),
        parts,
        locale: tag.to_owned(),
        resolved_from,
        is_fallback: rendered.is_fallback,
        is_draft: rendered.is_draft,
        warnings: rendered.warnings,
    })
}
